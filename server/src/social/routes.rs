use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::api_error::ApiError;
use crate::auth::SystemUser;
use crate::member::models::MemberId;
use crate::ApiResponse;

use super::error::SocialError;
use super::friend::models::Friend;
use super::friend::models::MultiFriend;
use super::request::models::MultiFriendRequest;
use super::request::models::{FriendRequest, RequestDetails};
use super::service::{RequestCreation, SocialService};

#[post("/<receiver_id>", data = "<form>")]
pub(crate) fn create_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    receiver_id: MemberId,
    form: Form<Strict<String>>,
) -> ApiResponse<Json<RequestCreation>> {
    let note = form.into_inner().into_inner();

    let response = social_service.send_friend_request(user.member_id(), receiver_id, note)?;
    Ok(status::Custom(Status::Created, Json(response)))
}

#[get("/<sender_id>/<receiver_id>")]
pub(crate) fn get_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    sender_id: i32,
    receiver_id: i32,
) -> ApiResponse<Json<RequestDetails>> {
    if user.member_id() != sender_id && user.member_id() != receiver_id {
        return Err(ApiError::from(SocialError::RequestNotFound));
    }
    let request = social_service.get_friend_request(sender_id, receiver_id)?;

    Ok(status::Custom(Status::Ok, Json(request)))
}

#[put("/<receiver_id>", data = "<form>")]
pub(crate) fn update_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    receiver_id: MemberId,
    form: Form<Strict<String>>,
) -> ApiResponse<()> {
    let note = form.into_inner().into_inner();
    let request = social_service.update_request(user.member_id(), receiver_id, note)?;
    Ok(status::Custom(Status::Ok, ()))
}

#[delete("/<sender_id>/accept")]
pub(crate) fn accept_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    sender_id: MemberId,
) -> ApiResponse<Json<Friend>> {
    let friend = social_service.accept_friend_request(user.member_id(), sender_id)?;
    Ok(status::Custom(Status::Ok, Json(friend)))
}
#[delete("/<sender_id>/deny")]
pub(crate) fn deny_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    sender_id: i32,
) -> ApiResponse<()> {
    social_service.deny_friend_request(user.member_id(), sender_id)?;
    Ok(status::Custom(Status::Ok, ()))
}
#[delete("/<receiver_id>")]
pub(crate) fn delete_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    receiver_id: MemberId,
) -> ApiResponse<()> {
    social_service.cancel_friend_request(user.member_id(), receiver_id)?;
    Ok(status::Custom(Status::Ok, ()))
}

#[get("/requests")]
pub(crate) fn get_friend_requests(
    user: SystemUser,
    social_service: &State<SocialService>,
) -> ApiResponse<MultiFriendRequest> {
    let requests = social_service.get_friend_requests(user.member_id())?;
    Ok(status::Custom(Status::Ok, requests))
}
