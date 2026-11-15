use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::ApiResponse;
use shared::models::chat::chat::ChatId;
use shared::models::chat::message::{
    MessageChangesetInput, MessageDetailed, MessageId, MultiMessageDetailed, NewMessageInput,
};

use super::service::MessageService;

#[post("/<chat_id>/messages", data = "<input>")]
fn create(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    input: Json<NewMessageInput>,
) -> ApiResponse<Json<MessageDetailed>> {
    let message = service.create(user.member_id(), chat_id, input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(message)))
}

#[get("/<chat_id>/messages/<message_id>")]
fn get(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
) -> ApiResponse<Json<MessageDetailed>> {
    Ok(status::Custom(
        Status::Ok,
        Json(service.get(user.member_id(), chat_id, message_id)?),
    ))
}
#[get("/<chat_id>/messages?<page>")]
fn get_thread(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiMessageDetailed>> {
    let page = page.unwrap_or(0);
    Ok(status::Custom(
        Status::Ok,
        Json(service.get_thread(user.member_id(), chat_id, page)?),
    ))
}

#[patch("/<chat_id>/messages/<message_id>", data = "<input>")]
fn update(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
    input: Json<MessageChangesetInput>,
) -> ApiResponse<Json<MessageDetailed>> {
    Ok(status::Custom(
        Status::Ok,
        Json(service.update(user.member_id(), chat_id, message_id, input.into_inner())?),
    ))
}

#[delete("/<chat_id>/messages/<message_id>")]
fn delete(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
) -> ApiResponse<()> {
    service.delete(user.member_id(), chat_id, message_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}

pub(in crate::chat) fn routes() -> Vec<rocket::Route> {
    routes![create, get, get_thread, update, delete]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::routes;

    #[test]
    fn message_routes_include_thread_and_resource_operations() {
        let signatures = routes()
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(signatures.contains(&(Method::Post, "/<chat_id>/messages".into())));
        assert!(signatures.contains(&(Method::Get, "/<chat_id>/messages/<message_id>".into())));
        assert!(signatures.contains(&(Method::Get, "/<chat_id>/messages?<page>".into())));
        assert!(signatures.contains(&(Method::Patch, "/<chat_id>/messages/<message_id>".into())));
        assert!(signatures.contains(&(Method::Delete, "/<chat_id>/messages/<message_id>".into())));
        assert_eq!(signatures.len(), 5);
    }
}
