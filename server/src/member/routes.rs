use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::api::ApiError;
use crate::auth::SystemUser;

use super::models::{MemberDetailed, MemberShort, MemberUpdateInput, MultiMemberPreview};
use super::service::MemberService;

use crate::ApiResponse;

#[head("/")]
/// Health check endpoint to verify the service is running.
fn head(_principal: SystemUser) -> ApiResponse<()> {
    Ok(status::Custom(Status::Ok, ()))
}

#[get("/")]
/// Returns the detailed information of the authenticated member.
fn get(user: SystemUser, member_service: &State<MemberService>) -> ApiResponse<Json<MemberDetailed>> {
    member_service
        .get_detailed(user.member_id())
        .map(|member| status::Custom(Status::Ok, Json(member)))
        .map_err(ApiError::from)
}

#[get("/search?<search_term>")]
/// Searches for members based on the provided search term.
fn search_members(
    member_service: &State<MemberService>,
    search_term: String,
) -> ApiResponse<Json<MultiMemberPreview>> {
    let members = member_service.search(search_term)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[patch("/", data = "<input>")]
/// Updates the authenticated member's information based on the provided input.
fn update(
    user: SystemUser,
    member_service: &State<MemberService>,
    input: Json<MemberUpdateInput>,
) -> ApiResponse<Json<MemberDetailed>> {
    let input = input.into_inner();
    member_service
        .update(user.member_id(), input)
        .map(|member| status::Custom(Status::Ok, Json(member)))
        .map_err(ApiError::from)
}

#[delete("/")]
// TODO(Severe): Route account deletion through the authenticated account lifecycle guard.
fn delete(user: SystemUser, member_service: &State<MemberService>) -> ApiResponse<String> {
    let member_id = user.member_id();
    member_service
        .delete(member_id)
        .map_err(ApiError::from)
        .map(|_| {
            status::Custom(
                Status::Ok,
                format!("Member number {member_id} has been deleted."),
            )
        })
}

/// Registers the private member HTTP handlers.
pub(super) fn routes() -> Vec<rocket::Route> {
    routes![head, get, search_members, update, delete]
}
