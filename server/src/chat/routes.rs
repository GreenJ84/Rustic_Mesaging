use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::ApiResponse;
use shared::models::chat::chat::{Chat, ChatChangesetInput, ChatId, NewChatInput};
use shared::models::chat::participant::{
    ChatParticipant, MultiChatParticipant, MultiChatParticipantDetailed, NewChatParticipantsInput,
};

use super::service::ChatService;

#[post("/", data = "<input>")]
fn create(
    user: SystemUser,
    chat_service: &State<ChatService>,
    input: Json<NewChatInput>,
) -> ApiResponse<Json<Chat>> {
    let chat = chat_service.create_chat(user.member_id(), input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(chat)))
}

#[get("/<chat_id>")]
fn get(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<Chat>> {
    let chat = chat_service.get_chat(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Ok, Json(chat)))
}

#[patch("/<chat_id>", data = "<input>")]
fn update(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
    input: Json<ChatChangesetInput>,
) -> ApiResponse<Json<Chat>> {
    let chat = chat_service.update_chat(user.member_id(), chat_id, input.into_inner())?;
    Ok(status::Custom(Status::Ok, Json(chat)))
}

#[post("/<chat_id>/members")]
fn join(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<ChatParticipant>> {
    let members = chat_service.join(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Created, Json(members)))
}

#[post("/<chat_id>/members/many", data = "<input>")]
fn add_members(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
    input: Json<NewChatParticipantsInput>,
) -> ApiResponse<Json<MultiChatParticipant>> {
    let members = chat_service.add_members(user.member_id(), input.into_inner(), chat_id)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[get("/<chat_id>/members")]
fn get_chat_members(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<MultiChatParticipantDetailed>> {
    let members = chat_service.get_chat_members(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[delete("/<chat_id>/members")]
fn leave(user: SystemUser, chat_service: &State<ChatService>, chat_id: ChatId) -> ApiResponse<()> {
    chat_service.leave(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        create,
        get,
        update,
        join,
        add_members,
        get_chat_members,
        leave,
    ]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::routes;

    #[test]
    fn chat_routes_use_resource_oriented_methods_and_paths() {
        let signatures = routes()
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(signatures.contains(&(Method::Post, "/".into())));
        assert!(signatures.contains(&(Method::Get, "/<chat_id>".into())));
        assert!(signatures.contains(&(Method::Patch, "/<chat_id>".into())));
        assert!(signatures.contains(&(Method::Post, "/<chat_id>/members".into())));
        assert!(signatures.contains(&(Method::Post, "/<chat_id>/members/many".into())));
        assert!(signatures.contains(&(Method::Get, "/<chat_id>/members".into())));
        assert!(signatures.contains(&(Method::Delete, "/<chat_id>/members".into())));
        assert_eq!(signatures.len(), 7);
    }
}
