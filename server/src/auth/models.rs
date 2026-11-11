use rocket::http::Header;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::Response;
use serde::{Deserialize, Serialize};

use crate::auth::session::AuthCredentials;
use crate::infra::{AppConfig, DbConn};
use crate::member::error::MemberError;
use crate::member::models::{ChangePasswordInput, MemberProfile, NewMember, NewMemberInput};
use crate::member::repository::MemberRepository;
use crate::utils::Validator;

use super::error::*;
use super::password::{PasswordManager, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

impl TryFrom<NewMemberInput> for NewMember {
    type Error = AuthError;
    fn try_from(input: NewMemberInput) -> Result<Self, AuthError> {
        if input.password.ne(&input.confirm_password) {
            return Err(AuthError::PasswordConfirmationMismatch);
        }
        let username = Validator::validate_text(&input.username, Some(3), Some(125))
            .map_err(|e| MemberError::InvalidInput(format!("Username {}", e.message())))?;
        let email = Validator::validate_text(&input.email, Some(6), Some(254))
            .map_err(|e| MemberError::InvalidInput(format!("Email {}", e.message())))?;
        if !email.contains('@') {
            return Err(MemberError::InvalidInput("Email must contain '@'".into()).into());
        }
        let password_hash = PasswordManager::validate(&input)?;
        Ok(Self::new(username, password_hash, email))
    }
}

// TODO: Move LoginInput into Shared crate
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

pub(crate) struct AuthOutcome {
    pub member: MemberProfile,
    pub credentials: AuthCredentials,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::member::models::NewMemberInput;

    fn input() -> NewMemberInput {
        NewMemberInput {
            username: "alice".to_string(),
            password: "a-secure-password-value".to_string(),
            confirm_password: "a-secure-password-value".to_string(),
            email: "alice@example.com".to_string(),
        }
    }

    #[test]
    fn converts_valid_registration_input_into_hashed_member_data() {
        let member = NewMember::try_from(input()).unwrap();

        assert_eq!(member.username(), "alice");
        assert_eq!(member.email(), "alice@example.com");
        assert!(PasswordManager::verify("a-secure-password-value", member.password_hash()).is_ok());
    }

    #[test]
    fn rejects_mismatched_password_confirmation() {
        let mut value = input();
        value.confirm_password = "different-password-value".to_string();

        assert!(matches!(
            NewMember::try_from(value),
            Err(AuthError::PasswordConfirmationMismatch)
        ));
    }

    #[test]
    fn rejects_invalid_email() {
        let mut value = input();
        value.email = "not-an-email".to_string();

        assert!(matches!(
            NewMember::try_from(value),
            Err(AuthError::Member(_))
        ));
    }

    #[test]
    fn rejects_invalid_username() {
        let mut value = input();
        value.username = "ab".to_string();

        assert!(matches!(
            NewMember::try_from(value),
            Err(AuthError::Member(_))
        ));
    }
}
