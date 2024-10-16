use rocket::form::{Form, Lenient, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::db::{DbPool, get_db_connection};
use crate::models::message::{NewMessage, Message};
use crate::routes::CustomResponse;
use crate::models::authentication::JWT;
use crate::service::{CrudOps, message_service::MessageService};
use crate::service::member_chat_service::MemberChatService;

#[post("/", data = "<form>")]
pub fn create(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewMessage>>,
) -> CustomResponse<Message> {
    let mut conn = get_db_connection(pool)?;

    let input = form.into_inner().into_inner();
    // Make sure member is participant of chat where message will be displayed
    if !MemberChatService::is_participant(&mut conn, input.chat_id(), token.claims.member_id){
        return Err::<status::Custom<Message>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only chat participants can create messages within")))
    }

    match MessageService::create(&mut conn, input){
        Ok(message) => Ok(
            status::Custom(
                Status::Ok,
                message
            )),
        Err(e) =>
            return Err::<status::Custom<Message>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Message creation error: {}", e)
                ))
    }
}

#[get("/<message_id>")]
pub fn get(
    token: JWT,
    pool: &State<DbPool>,
    message_id: i32
) -> CustomResponse<Message>{
    let mut conn = get_db_connection(pool)?;

    // Get message details
    let message = MessageService::read(&mut conn, message_id).map_err(|_|
        status::Custom(Status::InternalServerError, String::from("Database connection error"))
    )?;

    // Make sure member is participant of chat where message displays
    if !MemberChatService::is_participant(&mut conn, message.chat_id(), token.claims.member_id){
        return Err::<status::Custom<Message>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only chat participants can view messages within")))
    }

    match MessageService::read(&mut conn, message_id){
        Ok(message) => Ok(
            status::Custom(
                Status::Ok,
                message
            )),
        Err(e) =>
            return Err::<status::Custom<Message>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Message lookup error: {}", e)
                ))
    }
}

#[put("/<message_id>", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewMessage>>,
    message_id: i32
) -> CustomResponse<Message> {
    let mut conn = get_db_connection(pool)?;

    let message = MessageService::read(&mut conn, message_id).map_err(|_|
        status::Custom(Status::InternalServerError, String::from("Database connection error"))
    )?;
    // Make sure only sender can update their messages
    if !token.claims.member_id == message.sender_id(){
        return Err::<status::Custom<Message>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only message sender can update content")))
    }

    let input = form.into_inner().into_inner();
    match MessageService::update(&mut conn, message_id, input){
        Ok(message) => Ok(
            status::Custom(
                Status::Ok,
                message
            )),
        Err(e) =>
            return Err::<status::Custom<Message>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Message update error: {}", e)
                ))
    }
}

#[delete("/<message_id>")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
    message_id: i32
) -> CustomResponse<String>{
    let mut conn = get_db_connection(pool)?;

    if !token.claims.is_admin {
        return Err::<status::Custom<String>, status::Custom<String>>(
            status::Custom(Status::Unauthorized,
               String::from("Message deletions not allowed")
        ))
    }

    match MessageService::delete(&mut conn, message_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, format!("Message number {} has been deleted.", message_id)))
        },
        Err(e) => {
            return Err::<status::Custom<String>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Message deletion error: {}", e)
                ))
        }
    }
}