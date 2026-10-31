use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use super::service::AuthService;
use crate::api::ApiError;
use crate::auth::guards::{ClientMetadata, RegistrationRole};
use crate::auth::models::{
    AuthOutcome, AuthenticationResponse, ChangePasswordInput, LoginInput, RegisterInput,
};
use crate::auth::session::AuthenticatedPrincipal;
use crate::member::models::{MemberId, MemberProfile};
use crate::ApiResponse;

const REFRESH_COOKIE: &str = "refresh_token";

#[post("/register", data = "<input>")]
pub(crate) fn register(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
    metadata: ClientMetadata,
    registration_role: RegistrationRole,
    input: Json<RegisterInput>,
) -> ApiResponse<AuthenticationResponse> {
    let outcome = service.register(input.into_inner(), registration_role.kind(), metadata.0)?;
    let response = into_response(cookies, service, outcome);
    Ok(status::Custom(Status::Created, response))
}

#[post("/login", data = "<input>")]
pub(crate) fn login(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
    metadata: ClientMetadata,
    input: Json<LoginInput>,
) -> ApiResponse<AuthenticationResponse> {
    let outcome = service.login(input.into_inner(), metadata.0)?;
    let response = into_response(cookies, service, outcome);
    Ok(status::Custom(Status::Ok, response))
}

#[post("/refresh/<member_id>")]
pub(crate) fn refresh(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
    metadata: ClientMetadata,
    member_id: MemberId,
) -> ApiResponse<AuthenticationResponse> {
    let refresh_token = cookies
        .get(REFRESH_COOKIE)
        .map(|cookie| cookie.value().to_owned())
        .ok_or_else(|| ApiError::Unauthorized("A refresh credential is required.".to_string()))?;

    match service.refresh(member_id, refresh_token, metadata.0) {
        Ok(outcome) => {
            let response = into_response(cookies, service, outcome);
            Ok(status::Custom(Status::Ok, response))
        }
        Err(error) => {
            clear_refresh_cookie(cookies);
            Err(error.into())
        }
    }
}

#[post("/logout")]
pub(crate) fn logout(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
) -> Result<status::NoContent, ApiError> {
    match cookies.get(REFRESH_COOKIE) {
        Some(cookie) => service.logout(cookie.value()),
        None => Ok(()),
    }?;
    clear_refresh_cookie(cookies);
    Ok(status::NoContent)
}

#[post("/logout-all")]
pub(crate) fn logout_all(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
    principal: AuthenticatedPrincipal,
) -> Result<status::NoContent, ApiError> {
    service.logout_all(principal.member_id)?;
    clear_refresh_cookie(cookies);
    Ok(status::NoContent)
}

#[put("/password", data = "<input>")]
pub(crate) fn update_password(
    service: &State<AuthService>,
    cookies: &CookieJar<'_>,
    principal: AuthenticatedPrincipal,
    input: Json<ChangePasswordInput>,
) -> Result<status::NoContent, ApiError> {
    service.change_password(&principal, input.into_inner())?;
    clear_refresh_cookie(cookies);
    Ok(status::NoContent)
}

#[get("/member")]
pub(crate) fn current_member(
    service: &State<AuthService>,
    principal: AuthenticatedPrincipal,
) -> ApiResponse<Json<MemberProfile>> {
    let response = service.current_member(&principal).map_err(ApiError::from)?;
    Ok(status::Custom(Status::Ok, Json(response)))
}

fn into_response(
    cookies: &CookieJar<'_>,
    service: &AuthService,
    outcome: AuthOutcome,
) -> AuthenticationResponse {
    set_refresh_cookie(
        cookies,
        outcome.credentials.refresh_token,
        service.secure_cookies(),
    );
    AuthenticationResponse {
        member: outcome.member,
        token: outcome.credentials.access_token,
    }
}

fn set_refresh_cookie(cookies: &CookieJar<'_>, token: String, secure: bool) {
    cookies.add(
        Cookie::build((REFRESH_COOKIE, token))
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Strict)
            .path("/")
            .build(),
    );
}

fn clear_refresh_cookie(cookies: &CookieJar<'_>) {
    cookies.remove(Cookie::build(REFRESH_COOKIE).path("/").build());
}
