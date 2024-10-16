use rocket::http::Status;
use rocket::response::status;
use rocket::State;
use rocket::serde::json::Json;
use crate::db::{DbPool, get_db_connection};
use crate::models::{
    server_membership::{NewServerMembership, ServerMembership},
    authentication::JWT
};
use crate::routes::CustomResponse;
use crate::service::server_membership_service::MembershipService;



#[post("/<server_id>/join")]
pub fn join_server(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<ServerMembership> {
    let mut conn = get_db_connection(pool)?;

    let new_membership = NewServerMembership { member_id: token.claims.member_id, server_id };

    match MembershipService::join(&mut conn, new_membership){
        Ok(server) => Ok(status::Custom(
                Status::Created,
                server
            )),
        Err(e) =>
            return Err::<status::Custom<ServerMembership>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Server membership creation error: {}", e)
                ))
    }
}

#[get("/<server_id>/membership")]
pub fn get_membership_status(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<Json<(bool, String)>>{
    let mut conn = get_db_connection(pool)?;

    let is_participant =  MembershipService::has_membership(&mut conn, server_id, token.claims.member_id);
    Ok(status::Custom(
        Status::Ok,
        Json((is_participant, format!("You are {}a server member", if is_participant { "" } else { "NOT " }) ))
    ))
}

#[put("/<_server_id>/membership")]
pub fn update_membership_status(
    _token: JWT,
    _server_id: i32,
) -> CustomResponse<()> {
    Err(status::Custom(Status::NotImplemented, "No ability to update, just join or leave".to_string()))
}

#[delete("/<server_id>/leave")]
pub fn leave_server(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> CustomResponse<()>{
    let mut conn = get_db_connection(pool)?;

    match MembershipService::leave(&mut conn, server_id, token.claims.member_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, ()))
        },
        Err(e) => {
            return Err::<status::Custom<()>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Server membership deletion error: {}", e)
                ))
        }
    }
}