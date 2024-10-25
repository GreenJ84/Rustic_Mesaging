use rocket::form::{Form, Lenient, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;
use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::channel::{NewChannel, Channel};
use crate::models::post::MultiPost;
use crate::routes::CustomResponse;
use crate::models::authentication::JWT;
use crate::service::{CrudOps, channel_service::ChannelService};
use crate::service::server_service::ServerService;

pub fn channel_authorized(conn: &mut DbConn, channel_id: i32, jwt: JWT)
                          -> Result<bool, status::Custom<String>>{
    if !jwt.claims.is_admin && !ChannelService::has_authorization(conn, channel_id, jwt.claims.member_id){
        return Err(
            status::Custom(
                Status::Unauthorized,
                String::from("Only server members can access channel details")
            )
        )
    }
    Ok(true)
}

#[post("/", data = "<form>")]
pub fn create(
    _token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewChannel>>,
) -> CustomResponse<Channel> {
    let mut conn = get_db_connection(pool)?;

    let input = form.into_inner().into_inner();
    match ChannelService::create(&mut conn, input){
        Ok(channel) => Ok(
            status::Custom(
                Status::Ok,
                channel
            )),
        Err(e) =>
            return Err::<status::Custom<Channel>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Database creation error: {}", e)
                ))
    }
}

#[get("/<channel_id>")]
pub fn get(
    token: JWT,
    pool: &State<DbPool>,
    channel_id: i32
) -> CustomResponse<Channel>{
    let mut conn = get_db_connection(pool)?;
    channel_authorized(&mut conn, channel_id, token)?;

     match ChannelService::read(&mut conn, channel_id){
        Ok(channel) => Ok(
            status::Custom(
                Status::Ok,
                channel
            )),
        Err(e) => Err::<status::Custom<Channel>, status::Custom<String>>(
            status::Custom(
                Status::NotFound,
                format!("Channel lookup error: {}", e)
            ))
    }
}

#[put("/<channel_id>", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewChannel>>,
    channel_id: i32
) -> CustomResponse<Channel> {
    let mut conn = get_db_connection(pool)?;

    let input = form.into_inner().into_inner();
    if !ServerService::is_owner(&mut conn, input.server_id(), token.claims.member_id){
        return Err::<status::Custom< Channel >, status::Custom<String>>(
            status::Custom(
                Status::Unauthorized,
                String::from("Only server owner can update channels")
            )
        )
    }

    match ChannelService::update(&mut conn, channel_id, input){
        Ok(channel) => Ok(
            status::Custom(
                Status::Ok,
                channel
            )),
        Err(e) =>
            return Err::<status::Custom<Channel>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Channel update error: {}", e)
                ))
    }
}

#[delete("/<channel_id>")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
    channel_id: i32
) -> CustomResponse<String>{
    let mut conn = get_db_connection(pool)?;
    if !token.claims.is_admin && !ChannelService::has_authorization(&mut conn, channel_id, token.claims.member_id) {
        return Err::<status::Custom<String>, status::Custom<String>>(
            status::Custom(
                Status::Unauthorized,
                String::from("Only server owner can delete channels.")
            )
        )
    }

    match ChannelService::delete(&mut conn, channel_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, format!("Channel number {} has been deleted.", channel_id)))
        },
        Err(e) => {
            return Err::<status::Custom<String>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Channel deletion error: {}", e)
                ))
        }
    }
}

#[get("/<channel_id>/posts")]
pub fn get_channel_posts(
    token: JWT,
    pool: &State<DbPool>,
    channel_id: i32
) -> CustomResponse<MultiPost>{
    let mut conn = get_db_connection(pool)?;
    channel_authorized(&mut conn, channel_id, token)?;


    match ChannelService::get_channel_posts(&mut conn, channel_id){
        Ok(posts) => Ok(
            status::Custom(
                Status::Ok,
                posts
            )),
        Err(e) =>
            return Err::<status::Custom<MultiPost>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Channel post lookup error: {}", e)
                ))
    }
}
