use rocket::http::Header;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::Response;
use serde::{Deserialize, Serialize};

use super::error::*;
use super::password::{PasswordManager, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

use crate::auth::session::AuthCredentials;
use crate::infra::DbConn;
use crate::member::models::{MemberProfile, NewMember};
use crate::member::repository::MemberRepository;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub email: String,
}
impl TryInto<NewMember> for RegisterInput {
    type Error = AuthError;
    fn try_into(self) -> Result<NewMember, AuthError> {
        let password_hash = PasswordManager::validate(&self)?;
        let username = Validator::validate_text(self.username, Some(3), Some(125))
            .map_err(|e| AuthError::InvalidInput(format!("Username {}", e.message())))?;
        Ok(NewMember::new(
            username.to_string(),
            password_hash,
            self.email.to_string(),
        ))
    }
}

impl RegisterInput {
    pub fn validate(self, conn: &mut DbConn) -> Result<NewMember, AuthError> {
        if self.password.ne(&self.confirm_password) {
            return Err(AuthError::PasswordConfirmationMismatch);
        }
        match MemberRepository::identity_exists(conn, &self.username, &self.email) {
            Err(err) => Err(AuthError::InternalError(format!("{err:?}"))),
            Ok(Some(username)) => {
                if username == self.username {
                    Err(AuthError::UsernameTaken)
                } else {
                    Err(AuthError::EmailTaken)
                }
            }
            Ok(None) => {
                let new_member = RegisterInput::try_into(self)?;
                Ok(new_member)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
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
