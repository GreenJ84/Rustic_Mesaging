use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::api::ApiError;
use crate::auth::SystemUser;
use crate::member::models::MemberId;
use crate::ApiResponse;
use shared::models::social::friend::{FriendDetails, MultiFriend, MultiFriendPreview};
use shared::models::social::request::{
    FriendRequestChangesetInput, FriendRequestCreation, FriendRequestDetailed, MultiFriendRequest,
    NewFriendRequestInput,
};

use super::service::SocialService;

#[post("/", data = "<input>")]
fn create_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    input: Json<NewFriendRequestInput>,
) -> ApiResponse<Json<FriendRequestCreation>> {
    let response = social_service.send_friend_request(user.member_id(), input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(response)))
}

#[get("/<other_member_id>")]
fn get_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    other_member_id: MemberId,
) -> ApiResponse<Json<FriendRequestDetailed>> {
    let request = social_service.get_friend_request(user.member_id(), other_member_id)?;
    Ok(status::Custom(Status::Ok, Json(request)))
}

#[get("/")]
fn get_friend_requests(
    user: SystemUser,
    social_service: &State<SocialService>,
) -> ApiResponse<Json<MultiFriendRequest>> {
    let requests = social_service.get_friend_requests(user.member_id())?;
    Ok(status::Custom(Status::Ok, Json(requests)))
}

#[patch("/<receiver_id>", data = "<input>")]
fn update_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    receiver_id: MemberId,
    input: Json<FriendRequestChangesetInput>,
) -> Result<status::NoContent, ApiError> {
    social_service.update_request(user.member_id(), receiver_id, input.into_inner())?;
    Ok(status::NoContent)
}

#[patch("/<sender_id>/accept")]
fn accept_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    sender_id: MemberId,
) -> ApiResponse<Json<FriendDetails>> {
    let friend = social_service.accept_friend_request(user.member_id(), sender_id)?;
    Ok(status::Custom(Status::Ok, Json(friend)))
}

#[delete("/<sender_id>/deny")]
fn deny_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    sender_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    social_service.deny_friend_request(user.member_id(), sender_id)?;
    Ok(status::NoContent)
}

#[delete("/<receiver_id>")]
fn delete_request(
    user: SystemUser,
    social_service: &State<SocialService>,
    receiver_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    social_service.cancel_friend_request(user.member_id(), receiver_id)?;
    Ok(status::NoContent)
}

#[get("/<friend_id>/status")]
fn get_friend_status(
    user: SystemUser,
    social_service: &State<SocialService>,
    friend_id: MemberId,
) -> ApiResponse<Json<bool>> {
    let is_friend = social_service.is_friend(user.member_id(), friend_id)?;
    Ok(status::Custom(Status::Ok, Json(is_friend)))
}

#[get("/<friend_id>")]
fn get_friend(
    user: SystemUser,
    social_service: &State<SocialService>,
    friend_id: MemberId,
) -> ApiResponse<Json<FriendDetails>> {
    let friend = social_service.get_friend(user.member_id(), friend_id)?;
    Ok(status::Custom(Status::Ok, Json(friend)))
}

#[get("/")]
fn get_friend_previews(
    user: SystemUser,
    social_service: &State<SocialService>,
) -> ApiResponse<Json<MultiFriendPreview>> {
    let friends = social_service.get_member_friend_previews(user.member_id())?;
    Ok(status::Custom(Status::Ok, Json(friends)))
}

#[get("/detailed")]
fn get_friends(
    user: SystemUser,
    social_service: &State<SocialService>,
) -> ApiResponse<Json<MultiFriend>> {
    let friends = social_service.get_member_friends(user.member_id())?;
    Ok(status::Custom(Status::Ok, Json(friends)))
}

#[delete("/<friend_id>")]
fn remove_friend(
    user: SystemUser,
    social_service: &State<SocialService>,
    friend_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    social_service.remove_friend(user.member_id(), friend_id)?;
    Ok(status::NoContent)
}

pub(super) fn request_routes() -> Vec<rocket::Route> {
    routes![
        create_request,
        get_request,
        get_friend_requests,
        update_request,
        accept_request,
        deny_request,
        delete_request,
    ]
}

pub(super) fn friend_routes() -> Vec<rocket::Route> {
    routes![
        get_friend_status,
        get_friend,
        get_friend_previews,
        get_friends,
        remove_friend,
    ]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::{friend_routes, request_routes};

    fn signatures(routes: Vec<rocket::Route>) -> Vec<(Method, String)> {
        routes
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect()
    }

    #[test]
    fn request_routes_use_resource_oriented_methods_and_paths() {
        let routes = signatures(request_routes());

        assert!(routes.contains(&(Method::Post, "/".into())));
        assert!(routes.contains(&(Method::Get, "/".into())));
        assert!(routes.contains(&(Method::Get, "/<other_member_id>".into())));
        assert!(routes.contains(&(Method::Patch, "/<receiver_id>".into())));
        assert!(routes.contains(&(Method::Patch, "/<sender_id>/accept".into())));
        assert!(routes.contains(&(Method::Delete, "/<sender_id>/deny".into())));
        assert!(routes.contains(&(Method::Delete, "/<receiver_id>".into())));
        assert_eq!(routes.len(), 7);
    }

    #[test]
    fn friend_routes_do_not_expose_an_unsupported_update() {
        let routes = signatures(friend_routes());

        assert!(routes.contains(&(Method::Get, "/".into())));
        assert!(routes.contains(&(Method::Get, "/detailed".into())));
        assert!(routes.contains(&(Method::Get, "/<friend_id>".into())));
        assert!(routes.contains(&(Method::Get, "/<friend_id>/status".into())));
        assert!(routes.contains(&(Method::Delete, "/<friend_id>".into())));
        assert!(!routes.iter().any(|(method, _)| *method == Method::Put));
        assert_eq!(routes.len(), 5);
    }
}
