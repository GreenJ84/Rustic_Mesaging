use rocket::http::Header;
use rocket::request::{FromRequest, Outcome};
use rocket::response::status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::Response;
use serde::{Deserialize, Serialize};
use std::env;

use crate::auth::session::AuthCredentials;
use crate::infra::DbConn;
use crate::member::error::MemberError;
use crate::member::models::{ChangePasswordInput, MemberProfile, NewMember, NewMemberInput};
use crate::member::repository::MemberRepository;
use crate::utils::Validator;

use super::error::*;
use super::password::{PasswordManager, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

impl TryFrom<NewMemberInput> for NewMember {
    type Error = AuthError;
    fn try_from(input: NewMemberInput) -> Result<Self, AuthError> {
        let password_hash = PasswordManager::validate(&input)?;
        let username = Validator::validate_text(input.username, Some(3), Some(125))
            .map_err(|e| MemberError::InvalidInput(format!("Username {}", e.message())))?;
        let email = Validator::validate_text(input.email, Some(6), Some(254))
            .map_err(|e| MemberError::InvalidInput(format!("Email {}", e.message())))?;

        if !email.contains('@') {
            return Err(MemberError::InvalidInput("Email must contain '@'".into()).into());
        }
        Ok(Self::new(username, password_hash, email))
    }
}

pub trait ValidateRegistration {
    fn validate(self, conn: &mut DbConn) -> Result<NewMember, AuthError>;
}
impl ValidateRegistration for NewMemberInput {
    fn validate(self, conn: &mut DbConn) -> Result<NewMember, AuthError> {
        if self.password.ne(&self.confirm_password) {
            return Err(AuthError::PasswordConfirmationMismatch);
        }
        let new_member = NewMember::try_from(self)?;
        match MemberRepository::identity_exists(conn, new_member.username(), new_member.email()) {
            Err(err) => Err(AuthError::InternalError(format!("{err:?}"))),
            Ok(Some(username)) => {
                if username == new_member.username() {
                    Err(MemberError::UsernameTaken.into())
                } else {
                    Err(MemberError::EmailTaken.into())
                }
            }
            Ok(None) => Ok(new_member),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

pub(crate) struct AuthOutcome {
    pub member: MemberProfile,
    pub credentials: AuthCredentials,
}

// Secret Administration passkey
pub struct AdminKey {
    pub(crate) passkey: bool,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminKey {
    type Error = status::Custom<String>;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let admin_secret_key = env::var("ADMIN_SECRET_KEY").expect("ADMIN_SECRET_KEY not set");
        return if let Some(header_value) = req.headers().get_one("Admin-Key") {
            if header_value == admin_secret_key {
                Outcome::Success(AdminKey { passkey: true })
            } else {
                Outcome::Success(AdminKey { passkey: false })
            }
        } else {
            Outcome::Success(AdminKey { passkey: false })
        };
    }
}

// Response
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationResponse {
    pub member: MemberProfile,
    pub token: String, // String JWT encoding
}
impl<'r> Responder<'r, 'r> for AuthenticationResponse {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self.member).respond_to(req)?)
            .header(Header::new(
                "Authorization",
                format!("Bearer {}", self.token),
            ))
            .ok()
    }
}
