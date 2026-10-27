use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::chat::models::ChatId;
use crate::ApiResponse;

use super::models::{
    MessageChangesetInput, MessageDetails, MessageId, MessageInput, MultiMessageDetails,
};
use super::service::MessageService;

#[post("/<chat_id>/messages", data = "<form>")]
pub(crate) fn create(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    form: Form<Strict<MessageInput>>,
) -> ApiResponse<Json<MessageDetails>> {
    let message = service.create(user.member_id(), chat_id, form.into_inner().into_inner())?;
    Ok(status::Custom(Status::Created, Json(message)))
}

#[get("/<chat_id>/messages/<message_id>")]
pub(crate) fn get(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
) -> ApiResponse<Json<MessageDetails>> {
    Ok(status::Custom(
        Status::Ok,
        Json(service.get(user.member_id(), chat_id, message_id)?),
    ))
}
#[get("/<chat_id>/messages?<page>")]
pub(crate) fn get_thread(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiMessageDetails>> {
    let page = page.unwrap_or(0);
    Ok(status::Custom(
        Status::Ok,
        Json(service.get_thread(user.member_id(), chat_id, page)?),
    ))
}

#[patch("/<chat_id>/messages/<message_id>", data = "<form>")]
pub(crate) fn update(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
    form: Form<Strict<MessageChangesetInput>>,
) -> ApiResponse<Json<MessageDetails>> {
    Ok(status::Custom(
        Status::Ok,
        Json(service.update(
            user.member_id(),
            chat_id,
            message_id,
            form.into_inner().into_inner(),
        )?),
    ))
}

#[delete("/<chat_id>/messages/<message_id>")]
pub(crate) fn delete(
    user: SystemUser,
    service: &State<MessageService>,
    chat_id: ChatId,
    message_id: MessageId,
) -> ApiResponse<()> {
    service.delete(user.member_id(), chat_id, message_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}
