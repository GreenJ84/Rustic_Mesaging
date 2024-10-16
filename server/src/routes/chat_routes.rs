use rocket::form::{Form, Lenient, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::db::{DbPool, get_db_connection};
use crate::models::authentication::JWT;
use crate::models::member_chat::NewMemberChat;
use crate::models::chat::{NewChat, Chat, NewChatWithMembers};
use crate::models::member::MultiMemberPreview;
use crate::models::message::MessageThread;
use crate::routes::CustomResponse;
use crate::service::chat_service::ChatService;
use crate::service::CrudOps;
use crate::service::member_chat_service::MemberChatService;


#[post("/", data = "<form>")]
pub fn create(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewChatWithMembers>>,
) -> CustomResponse<Chat> {
    let mut conn = get_db_connection(pool)?;

    let NewChatWithMembers { name, members} = form.into_inner().into_inner();

    // Create the chat
    let chat = ChatService::create(&mut conn, NewChat { name })
        .map_err(|e|
            status::Custom(
                Status::InternalServerError,
                format!("Chat creation error: {}", e)
        ))?;

    // Add the intended members to the chat
    MemberChatService::join(&mut conn, NewMemberChat::new(chat.id(), token.claims.member_id)).map_err(|_|
    status::Custom(
        Status::FailedDependency,
        String::from("Could not create a chat with specified members")
    ))?;
    for member_id in members {
        MemberChatService::join(&mut conn, NewMemberChat::new(chat.id(), member_id)).map_err(|_|
            status::Custom(
                Status::FailedDependency,
                String::from("Could not create a chat with specified members")
            ))?;
    }

    Ok(status::Custom(
        Status::Created,
        chat
    ))
}

#[get("/<chat_id>")]
pub fn get(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32
) -> CustomResponse<Chat>{
    let mut conn = get_db_connection(pool)?;

    // Get chat details
    let chat = ChatService::read(&mut conn, chat_id).map_err(|e|
        status::Custom(
            Status::NotFound,
            format!("Chat lookup error: {}", e)
        ))?;

    // Make sure member is participant of chat where chat displays
    if !MemberChatService::is_participant(&mut conn, chat.id(), token.claims.member_id){
        return Err::<status::Custom<Chat>, status::Custom<String>>(
            status::Custom(Status::Unauthorized,
                String::from("Only chat participants can view chats within")
        ))
    }

    Ok(status::Custom(
        Status::Ok,
        chat
    ))
}

#[put("/<chat_id>", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewChat>>,
    chat_id: i32
) -> CustomResponse<Chat> {
    let mut conn = get_db_connection(pool)?;

    let chat = ChatService::read(&mut conn, chat_id).map_err(|e|
        status::Custom(Status::FailedDependency, format!("Chat update error {}", e))
    )?;
    // Make sure member is participant of chat where chat displays
    if !MemberChatService::is_participant(&mut conn, chat.id(), token.claims.member_id){
        return Err::<status::Custom<Chat>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only chat participants can update chat details")))
    }

    let input = form.into_inner().into_inner();
    match ChatService::update(&mut conn, chat_id, input){
        Ok(chat) => Ok(
            status::Custom(
                Status::Ok,
                chat
            )),
        Err(e) =>
            return Err::<status::Custom<Chat>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Chat update error: {}", e)
                ))
    }
}

#[delete("/<chat_id>")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32
) -> CustomResponse<String>{
    let mut conn = get_db_connection(pool)?;

    if !token.claims.is_admin {
        return Err::<status::Custom<String>, status::Custom<String>>(
            status::Custom(Status::Unauthorized,
               String::from("Chat deletions not allowed")
        ))
    }

    match ChatService::delete(&mut conn, chat_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, format!("Chat number {} has been deleted.", chat_id)))
        },
        Err(e) => {
            return Err::<status::Custom<String>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Chat deletion error: {}", e)
                ))
        }
    }
}

#[get("/<chat_id>/members")]
pub fn get_chat_members(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32
) -> CustomResponse<MultiMemberPreview>{
    let mut conn = get_db_connection(pool)?;

    if !MemberChatService::is_participant(&mut conn, chat_id, token.claims.member_id){
        return Err::<status::Custom<MultiMemberPreview>, status::Custom<String>>(
            status::Custom(
                Status::Unauthorized,
                String::from("Only participants can view chat members")
            )
        )
    }

    match ChatService::get_chat_members(&mut conn, chat_id){
        Ok(members) => {
            Ok(status::Custom(
                Status::Ok,
                MultiMemberPreview { members }
            ))
        },
        Err(e) => {
            Err(status::Custom(
                Status::InternalServerError,
                format!("Members retrieval error {}", e)
            ))
        }
    }
}

#[get("/<chat_id>/thread")]
pub fn get_chat_thread(
    token: JWT,
    pool: &State<DbPool>,
    chat_id: i32
) -> CustomResponse<MessageThread>{
    let mut conn = get_db_connection(pool)?;

    if !MemberChatService::is_participant(&mut conn, chat_id, token.claims.member_id){
        return Err::<status::Custom<MessageThread>, status::Custom<String>>(
            status::Custom(
                Status::Unauthorized,
                String::from("Only participants can view chat content")
            )
        )
    }

    match ChatService::get_chat_thread(&mut conn, chat_id){
        Ok(thread) => {
            Ok(status::Custom(
                Status::Ok,
                MessageThread { thread }
            ))
        },
        Err(e) => {
            Err(status::Custom(
                Status::InternalServerError,
                format!("Thread retrieval error {}", e)
            ))
        }
    }
}