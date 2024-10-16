use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::db::{DbPool, get_db_connection};
use crate::models::authentication::JWT;
use crate::models::member_chat::{MemberChat, NewMemberChat};
use crate::routes::CustomResponse;
use crate::service::member_chat_service::MemberChatService;


#[post("/<chat_id>/join")]
pub fn join_chat(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32,
) -> CustomResponse<MemberChat> {
    let mut conn = get_db_connection(pool)?;

    let member_chat = NewMemberChat::new(chat_id, token.claims.member_id);

    match MemberChatService::join(&mut conn, member_chat) {
        Ok(member_chat) => Ok(status::Custom(
            Status::Created,
            member_chat
        )),
        Err(e) =>
            return Err::<status::Custom<MemberChat>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Chat member creation error: {}", e)
                ))
    }

}

#[get("/<chat_id>/status")]
pub fn get_membership_status(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32,
) -> CustomResponse<Json<(bool, String)>>{
    let mut conn = get_db_connection(pool)?;

    let is_participant = MemberChatService::is_participant(&mut conn, chat_id, token.claims.member_id);
    Ok(status::Custom(
        Status::Ok,
        Json((is_participant, format!("You are {}a chat participant", if is_participant { "" } else { "NOT " } )))
    ))
}

#[put("/<_chat_id>/status")]
pub fn update_membership_status(
    _token: JWT,
    _chat_id: i32,
) -> CustomResponse<()> {
    Err(status::Custom(Status::NotImplemented, "No ability to update, just join or leave".to_string()))
}

#[delete("/<chat_id>/leave")]
pub fn leave_chat(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32,
) -> CustomResponse<()>{
    let mut conn = get_db_connection(pool)?;

    match MemberChatService::leave(&mut conn, chat_id, token.claims.member_id){
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
                    format!("Chat Member deletion error: {}", e)
                ))
        }
    }
}