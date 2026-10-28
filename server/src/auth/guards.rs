use std::convert::Infallible;
use std::env;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::State;

use super::service::AuthService;
use crate::auth::error::AuthError;
use crate::auth::session::{AuthenticatedPrincipal, SessionId, SessionMetadata};
use crate::infra::AdminCredential;
use crate::auth::system_roles::{BuiltInSystemRole, RegistrationRoleKind, SystemRoleError};
use crate::member::models::MemberId;

/// Registration capability selected from the rotating platform secret. An
/// omitted key means ordinary user registration; an invalid supplied key is
/// rejected rather than silently downgraded.
pub(crate) struct RegistrationRole(RegistrationRoleKind);
impl RegistrationRole {
    pub(crate) const fn kind(&self) -> RegistrationRoleKind {
        self.0
    }
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RegistrationRole {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let admin_credentials = match request.guard::<&State<AdminCredential>>().await {
            Outcome::Success(config) => config,
            Outcome::Error(error) => return Outcome::Error((error.0, AuthError::PersistenceFailure)),
            Outcome::Forward(status) => return Outcome::Forward(status),
        };

        let Some(provided) = request.headers().get_one("Administration-Key") else {
            return Outcome::Success(Self(RegistrationRoleKind::User));
        };
        if admin_credentials.matches(provided) {
            Outcome::Success(Self(RegistrationRoleKind::Administrator))
        } else {
            Outcome::Error((Status::Unauthorized, AuthError::InvalidAdministrationKey))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedPrincipal {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let service = match request.rocket().state::<AuthService>() {
            Some(service) => service,
            None => {
                let error = AuthError::PersistenceFailure;
                return Outcome::Error((error.status(), error));
            }
        };
        let token = match bearer_token(request) {
            Ok(token) => token,
            Err(error) => return Outcome::Error((error.status(), error)),
        };
        match service.authenticate(token) {
            Ok(principal) => Outcome::Success(principal),
            Err(error) => Outcome::Error((error.status(), error)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SystemUser(AuthenticatedPrincipal);
#[rocket::async_trait]
impl<'r> FromRequest<'r> for SystemUser {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        attest(request, BuiltInSystemRole::User)
            .await
            .map(SystemUser)
    }
}
impl SystemUser {
    pub(crate) fn principal(&self) -> &AuthenticatedPrincipal {
        &self.0
    }
    pub(crate) fn session_id(&self) -> SessionId {
        self.0.session_id
    }
    pub(crate) fn member_id(&self) -> MemberId {
        self.0.member_id
    }
    pub(crate) fn role(&self) -> BuiltInSystemRole {
        BuiltInSystemRole::User
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SystemModerator(AuthenticatedPrincipal);
#[rocket::async_trait]
impl<'r> FromRequest<'r> for SystemModerator {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        attest(request, BuiltInSystemRole::Moderator)
            .await
            .map(SystemModerator)
    }
}
impl SystemModerator {
    pub(crate) fn principal(&self) -> &AuthenticatedPrincipal {
        &self.0
    }
    pub(crate) fn session_id(&self) -> SessionId {
        self.0.session_id
    }
    pub(crate) fn member_id(&self) -> MemberId {
        self.0.member_id
    }
    pub(crate) fn role(&self) -> BuiltInSystemRole {
        BuiltInSystemRole::Moderator
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SystemAdmin(AuthenticatedPrincipal);
#[rocket::async_trait]
impl<'r> FromRequest<'r> for SystemAdmin {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        attest(request, BuiltInSystemRole::Admin)
            .await
            .map(SystemAdmin)
    }
}
impl SystemAdmin {
    pub(crate) fn principal(&self) -> &AuthenticatedPrincipal {
        &self.0
    }
    pub(crate) fn session_id(&self) -> SessionId {
        self.0.session_id
    }
    pub(crate) fn member_id(&self) -> MemberId {
        self.0.member_id
    }
    pub(crate) fn role(&self) -> BuiltInSystemRole {
        BuiltInSystemRole::Admin
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClientMetadata(pub(crate) SessionMetadata);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientMetadata {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self(SessionMetadata {
            user_agent: request
                .headers()
                .get_one("User-Agent")
                .map(ToOwned::to_owned),
            ip_address: request.client_ip().map(|address| address.to_string()),
        }))
    }
}

async fn attest(
    request: &Request<'_>,
    minimum: BuiltInSystemRole,
) -> Outcome<AuthenticatedPrincipal, AuthError> {
    let principal = match request.guard::<AuthenticatedPrincipal>().await {
        Outcome::Success(principal) => principal,
        Outcome::Error(error) => return Outcome::Error(error),
        Outcome::Forward(status) => return Outcome::Forward(status),
    };
    let service = match request.rocket().state::<AuthService>() {
        Some(service) => service,
        None => {
            let error = AuthError::PersistenceFailure;
            return Outcome::Error((error.status(), error));
        }
    };
    match service.authorize(principal.member_id, minimum) {
        Ok(()) => Outcome::Success(principal),
        Err(error) => Outcome::Error((error.status(), error)),
    }
}

fn bearer_token<'a>(request: &'a Request<'_>) -> Result<&'a str, AuthError> {
    let header = request
        .headers()
        .get_one("Authorization")
        .ok_or(AuthError::MissingAuthorization)?;
    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MalformedAuthorization)?;
    if token.is_empty() || token.chars().any(char::is_whitespace) {
        return Err(AuthError::MalformedAuthorization);
    }
    Ok(token)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn compares_secret_digests() {
        assert!(constant_time_eq(b"same", b"same"));
        assert!(!constant_time_eq(b"same", b"diff"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }
}
