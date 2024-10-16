use diesel::Connection;
use diesel::result::Error;
use rocket::form::{Form, Lenient, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::channel::MultiChannel;
use crate::models::member::{MemberSafe, MultiMember};
use crate::models::server::{MultiServer, NewServer, Server};
use crate::routes::CustomResponse;
use crate::models::authentication::JWT;
use crate::models::server_membership::NewServerMembership;
use crate::service::{CrudOps, server_service::ServerService};
use crate::service::server_membership_service::MembershipService;

#[post("/", data = "<form>")]
pub fn create(
    _token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewServer>>,
) -> CustomResponse<Server> {
    let mut conn = get_db_connection(pool)?;

    let input = form.into_inner().into_inner();
    match conn.transaction::<Server, Error, _>(move |mut conn| {
        let server = ServerService::create(&mut conn, input)?;
        MembershipService::join(&mut conn, NewServerMembership {
                member_id: server.owner_id(),
                server_id:  server.id()
            })?;
        Ok(server)
    }){
        Ok(server) => Ok(status::Custom(
            Status::Created,
            server
        )),
        Err(e) => Err(status::Custom(Status::InternalServerError, format!("Server entity creation error: {}", e))),
    }
}

#[get("/<server_id>")]
pub fn get(
    _token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<Server>{
    let mut conn = get_db_connection(pool)?;

    match ServerService::read(&mut conn, server_id){
        Ok(server) => Ok(
            status::Custom(
                Status::Ok,
                server
            )),
        Err(e) =>
            return Err::<status::Custom<Server>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Server lookup error: {}", e)
            ))
    }
}

#[put("/<server_id>", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewServer>>,
    server_id: i32
) -> CustomResponse<Server> {
    let mut conn = get_db_connection(pool)?;

    let server = ServerService::read(&mut conn, server_id).map_err(|_| {
        status::Custom(Status::FailedDependency, String::from("Server update error"))
    })?;
    if token.claims.member_id != server.owner_id() && !token.claims.is_admin{
        return Err::<status::Custom<Server>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only Admin and Owners can update servers.")))
    }

    let input = form.into_inner().into_inner();
    match ServerService::update(&mut conn, server_id, input){
        Ok(server) => Ok(
            status::Custom(
                Status::Ok,
                server
            )),
        Err(e) =>
            return Err::<status::Custom<Server>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Server update error: {}", e)
                ))
    }
}

#[delete("/<server_id>")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<()>{
    let mut conn = get_db_connection(pool)?;

    let server = ServerService::read(&mut conn, server_id).map_err(|_| {
        status::Custom(Status::InternalServerError, String::from("Database connection error"))
    })?;
    if token.claims.member_id != server.owner_id() && !token.claims.is_admin{
        return Err::<status::Custom<()>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only Admin and Owners can update servers.")))
    }

    match ServerService::delete(&mut conn, server_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, ()))
        },
        Err(e) => {
            return Err::<status::Custom<()>, status::Custom<String>>(
                status::Custom(
                    Status::FailedDependency,
                    format!("Server deletion error: {}", e)
                ))
        }
    }
}

#[get("/search?<search_term>")]
pub fn search_servers(
    _token: JWT,
    pool: &State<DbPool>,
    search_term: &str
) -> CustomResponse<MultiServer>{
    let mut conn = get_db_connection(pool)?;

    let results = ServerService::search_servers(&mut conn, search_term.to_string())
        .map_err(|_| status::Custom(Status::InternalServerError, String::from("Server search error")))?;

    Ok(status::Custom(
        Status::Ok,
        MultiServer { servers: results }
    ))
}

#[get("/all")]
pub fn get_all_servers(
    _token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MultiServer>{
    let mut conn = get_db_connection(pool)?;

    match ServerService::get_all_servers(&mut conn){
        Ok(servers) => {
            Ok(status::Custom(
                Status::Ok,
                MultiServer { servers },
            ))
        },
        Err(e) =>
            return Err::<status::Custom<MultiServer>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Servers lookup error: {}", e)
                ))
    }
}

#[get("/<server_id>/members")]
pub fn get_server_members(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<MultiMember>{
    let mut conn = get_db_connection(pool)?;
    if !MembershipService::has_membership(&mut conn, server_id, token.claims.member_id){
        return Err::<status::Custom<MultiMember>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only server members can view other members within")))
    }

    match ServerService::get_server_members(&mut conn, server_id){
        Ok(members) => {
            let safe_members = members.into_iter().map(|member| member.into_safe()).collect::<Vec<MemberSafe>>();
            Ok(status::Custom(
                Status::Ok,
                MultiMember { members: safe_members },
            ))
        },
        Err(e) =>
            return Err::<status::Custom<MultiMember>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Server members lookup error: {}", e)
                ))
    }
}

#[get("/<server_id>/channels")]
pub fn get_server_channels(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<MultiChannel>{
    let mut conn = get_db_connection(pool)?;
    if !MembershipService::has_membership(&mut conn, server_id, token.claims.member_id){
        return Err::<status::Custom<MultiChannel>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only server members can view channels within")))
    }


    match ServerService::get_server_channels(&mut conn, server_id){
        Ok(channels) => Ok(
            status::Custom(
                Status::Ok,
                MultiChannel { channels }
            )),
        Err(e) =>
            return Err::<status::Custom<MultiChannel>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Server channels lookup error: {}", e)
                ))
    }
}