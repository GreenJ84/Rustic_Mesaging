use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::api_error::ApiError;
use crate::auth::SystemUser;

use super::models::{MemberProfile, MemberShort, MemberUpdateInput, MultiMemberPreview};
use super::service::MemberService;

use crate::chat::models::MultiChatPreview;
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

#[patch("/", data = "<form>")]
pub fn update(
    user: SystemUser,
    member_service: &State<MemberService>,
    form: Form<Strict<MemberUpdateInput>>,
) -> ApiResponse<MemberProfile> {
    let input = form.into_inner().into_inner();
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

#[get("/chats")]
pub fn get_all_member_chats(
    user: SystemUser,
    member_service: &State<MemberService>,
) -> ApiResponse<MultiChatPreview> {
    let chats = member_service.get_member_chats(user.member_id())?;
    Ok(status::Custom(Status::Ok, chats))
}

#[get("/servers")]
pub fn get_all_member_servers(
    user: SystemUser,
    member_service: &State<MemberService>,
) -> ApiResponse<MultiServer> {
    let servers = member_service.get_all_member_servers(user.member_id())?;
    Ok(status::Custom(Status::Ok, servers))
}

#[get("/friends")]
pub fn get_all_friends(
    user: SystemUser,
    member_service: &State<MemberService>,
) -> ApiResponse<MultiFriend> {
    let friends = member_service.get_member_friends(user.member_id())?;
    Ok(status::Custom(Status::Ok, friends))
}
