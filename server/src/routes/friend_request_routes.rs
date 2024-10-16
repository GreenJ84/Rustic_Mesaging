use rocket::http::Status;
use rocket::response::status;
use rocket::State;
use rocket::form::{Form, Lenient, Strict};
use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::{
    friend::Friend,
    authentication::JWT
};
use crate::models::friend_request::{FriendRequest, NewFriendRequest};
use crate::routes::CustomResponse;
use crate::service::friend_request_service::FriendRequestService;
use crate::service::friend_request_service::RequestResolve;
use crate::service::friend_request_service::RequestResolve::{NotResolved, Resolved};

#[post("/", data="<form>")]
pub fn create_request(
    _token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewFriendRequest>>
) -> CustomResponse<RequestResolve> {
    let mut conn = get_db_connection(pool)?;

    let new_request = form.into_inner().into_inner();

    match FriendRequestService::create_request(&mut conn, new_request){
        Ok(NotResolved(request)) => Ok(status::Custom(
            Status::Created,
            NotResolved(request)
        )),
        Ok(Resolved(friend)) => Ok(status::Custom(
            Status::Ok,
            Resolved(friend)
        )),
        Err(e) =>
            return Err::<status::Custom<RequestResolve>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Friendship creation error: {}", e)
                ))
    }
}

#[get("/<sender_id>/<receiver_id>")]
pub fn get_request(
    _token: JWT,
    pool: &State<DbPool>,
    sender_id: i32,
    receiver_id: i32
) -> CustomResponse<FriendRequest>{
    let mut conn = get_db_connection(pool)?;

    let request = FriendRequestService::get_request(&mut conn, sender_id, receiver_id)
        .map_err(|e| status::Custom(
            Status::NotFound,
            format!("Chat lookup error: {}", e)
        ))?;

    Ok(status::Custom(
        Status::Ok,
        request
    ))
}

#[put("/", data = "<form>")]
pub fn update_request(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewFriendRequest>>
) -> CustomResponse<FriendRequest> {
    let mut conn = get_db_connection(pool)?;
    let input = form.into_inner().into_inner();
    if token.claims.member_id != input.sender_id(){
        return Err::<status::Custom<FriendRequest>, status::Custom<String>>(status::Custom(
            Status::Unauthorized,
            String::from("Only request sender can update the request.")
        ));
    }

    let request = FriendRequestService::update_request(&mut conn, input)
        .map_err(|e| status::Custom(
            Status::FailedDependency,
            format!("Chat update error {}", e)
        ))?;
    Ok(status::Custom(Status::Ok, request))
}


fn has_request (
    conn: &mut DbConn, sender_id: i32, receiver_id: i32
) -> Result<bool, status::Custom<String>> {
    if !FriendRequestService::is_requested(conn, sender_id, receiver_id){
        return Err::<bool, status::Custom<String>>(
            status::Custom(
                Status::Conflict,
                format!("There is no friend request from Member Id: {} to Member ID: {}. Cannot remove.", sender_id, receiver_id)
            )
        )
    }
    Ok(true)
}
#[delete("/<sender_id>/<receiver_id>/accept")]
pub fn accept_request(
    _token: JWT,
    pool: &State<DbPool>,
    sender_id: i32,
    receiver_id: i32
) -> CustomResponse<Friend>{
    let mut conn = get_db_connection(pool)?;
    has_request(&mut conn, sender_id, receiver_id)?;

    match FriendRequestService::accept_request(&mut conn, sender_id, receiver_id){
        Ok(friend) => {
            Ok(status::Custom(Status::Ok, friend))
        },
        Err(e) => {
            return Err::<status::Custom<Friend>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Database deletion error: {}", e)
                ))
        }
    }
}
#[delete("/<sender_id>/<receiver_id>/deny")]
pub fn deny_request(
    _token: JWT,
    pool: &State<DbPool>,
    sender_id: i32,
    receiver_id: i32
) -> CustomResponse<()>{
    let mut conn = get_db_connection(pool)?;
    has_request(&mut conn, sender_id, receiver_id)?;

    match FriendRequestService::deny_request(&mut conn, sender_id, receiver_id){
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
                    format!("Database deletion error: {}", e)
                ))
        }
    }
}