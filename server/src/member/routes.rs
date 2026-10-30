use rocket::http::Status;
use rocket::response::status;
use rocket::State;
use rocket::serde::json::Json;

use crate::api::ApiError;
use crate::auth::SystemUser;

use super::models::{MemberProfile, MemberShort, MemberUpdateInput, MultiMemberPreview};
use super::service::MemberService;

use crate::server::models::MultiServer;
use crate::ApiResponse;

#[head("/")]
pub fn head(_principal: SystemUser) -> ApiResponse<()> {
    Ok(status::Custom(Status::Ok, ()))
}

/// Member creation ONLY through authenticating Registration route
#[get("/")]
pub fn get(user: SystemUser, member_service: &State<MemberService>) -> ApiResponse<MemberProfile> {
    member_service
        .get(user.member_id())
        .map(|member| status::Custom(Status::Ok, member))
        .map_err(ApiError::from)
}

#[patch("/", data = "<input>")]
pub fn update(
    user: SystemUser,
    member_service: &State<MemberService>,
    input: Json<MemberUpdateInput>,
) -> ApiResponse<MemberProfile> {
    let input = input.into_inner();
    member_service
        .update(user.member_id(), input)
        .map(|member| status::Custom(Status::Ok, member))
        .map_err(ApiError::from)
}

#[delete("/")]
pub fn delete(user: SystemUser, member_service: &State<MemberService>) -> ApiResponse<String> {
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

#[get("/<username>")]
pub fn get_by_username(
    _user: SystemUser,
    member_service: &State<MemberService>,
    username: String,
) -> ApiResponse<MemberShort> {
    match member_service.find_by_username(username)? {
        Some(member) => Ok(status::Custom(Status::Ok, member)),
        None => Err(ApiError::NotFound("Member not found".to_string())),
    }
}

#[get("/search?<search_term>")]
pub fn search_members(
    _user: SystemUser,
    member_service: &State<MemberService>,
    search_term: String,
) -> ApiResponse<MultiMemberPreview> {
    let members = member_service.search_members(search_term)?;
    Ok(status::Custom(Status::Ok, members))
}
