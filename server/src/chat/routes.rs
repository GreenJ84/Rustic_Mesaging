use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::chat::participant::models::{ChatParticipant, ChatParticipantDetails, MultiChatParticipantDetails};
use crate::member::models::{MemberId, MultiMemberPreview};
use crate::ApiResponse;

use super::models::{ChatChangesetInput, ChatDetails, ChatId, MultiChatDetails, NewChatInput};
use super::service::ChatService;

#[post("/", data = "<form>")]
pub(crate) fn create(
    user: SystemUser,
    chat_service: &State<ChatService>,
    form: Form<Strict<NewChatInput>>,
) -> ApiResponse<Json<ChatDetails>> {
    let chat = chat_service.create_chat(user.member_id(), form.into_inner().into_inner())?;
    Ok(status::Custom(Status::Created, Json(chat)))
}

#[get("/<chat_id>")]
pub(crate) fn get(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<ChatDetails>> {
    let chat = chat_service.get_chat(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Ok, Json(chat)))
}

#[put("/<chat_id>", data = "<form>")]
pub(crate) fn update(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
    form: Form<Strict<ChatChangesetInput>>,
) -> ApiResponse<Json<ChatDetails>> {
    let chat =
        chat_service.update_chat(user.member_id(), chat_id, form.into_inner().into_inner())?;
    Ok(status::Custom(Status::Ok, Json(chat)))
}

#[post("/<chat_id>/members")]
pub(crate) fn join(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<ChatParticipant>> {
    let members = chat_service.join(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Created, Json(members)))
}

#[post("/<chat_id>/members/many", data = "<form>")]
pub(crate) fn add_members(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
    form: Form<Strict<Vec<MemberId>>>,
) -> ApiResponse<Json<Vec<ChatParticipant>>> {
    let input = form.into_inner().into_inner();
    let members = chat_service.add_members(user.member_id(), input, chat_id)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[get("/<chat_id>/members")]
pub(crate) fn get_chat_members(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<Json<MultiChatParticipantDetails>> {
    let members = chat_service.get_chat_members(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[delete("/<chat_id>/members")]
pub(crate) fn leave(
    user: SystemUser,
    chat_service: &State<ChatService>,
    chat_id: ChatId,
) -> ApiResponse<()> {
    chat_service.leave(user.member_id(), chat_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}
