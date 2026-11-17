use rocket::http::Status;
use rocket::response::status;
use rocket::response::Redirect;
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
/// Redirects the authenticated member to their own profile page.
fn get() -> Redirect {
    Redirect::to(uri!(crate::auth::routes::current_member))
}

#[get("/<username>")]
/// Returns the public profile projection for an exact username.
fn get_by_username(
    _user: SystemUser,
    member_service: &State<MemberService>,
    username: String,
) -> ApiResponse<Json<MemberShort>> {
    let member = member_service.get_by_username(username)?;
    Ok(status::Custom(Status::Ok, Json(member)))
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

/// Registers the private member HTTP handlers.
pub(super) fn routes() -> Vec<rocket::Route> {
    routes![head, get, get_by_username, search_members, update]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::routes;

    #[test]
    fn member_routes_register_expected_methods_and_resource_paths() {
        let signatures = routes()
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(signatures.contains(&(Method::Head, "/".into())));
        assert!(signatures.contains(&(Method::Get, "/".into())));
        assert!(signatures.contains(&(Method::Get, "/<username>".into())));
        assert!(signatures.contains(&(Method::Get, "/search?<search_term>".into())));
        assert!(signatures.contains(&(Method::Patch, "/".into())));
        assert_eq!(signatures.len(), 5);
    }
}
