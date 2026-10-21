use rocket::http::Header;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::Response;
use serde::{Deserialize, Serialize};

use super::error::*;
use super::password::{PasswordManager, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};
use super::service::AuthService;
use crate::auth::session::{AuthCredentials, AuthenticatedPrincipal};
use crate::database::DbConn;
use crate::member::models::{MemberProfile, NewMember};
use crate::member::repository::MemberRepository;

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct RegisterInput {
    #[field(validate = len(3..=125).or_else(msg!("Username must be between 3 and 125 characters long.")))]
    pub username: String,
    #[field(validate = len(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).or_else(msg!(format!("Password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters long."))))]
    pub password: String,
    pub confirm_password: String,
    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: String,
}
impl TryInto<NewMember> for RegisterInput {
    type Error = AuthError;
    fn try_into(self) -> Result<NewMember, AuthError> {
        let password_hash = PasswordManager::validate(&self)?;
        Ok(NewMember::new(
            self.username.to_string(),
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

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct LoginInput {
    #[field(validate = len(3..=125).or_else(msg!("Usernames are between 3 and 125 characters long.")))]
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct ChangePasswordInput {
    pub current_password: String,
    #[field(validate = len(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).or_else(msg!(format!("Password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters long."))))]
    pub new_password: String,
    pub confirm_password: String,
}

pub(crate) struct AuthOutcome {
    pub member: MemberProfile,
    pub credentials: AuthCredentials,
}

// ======================================================
// Old Implementations
// ======================================================
use crate::member::models::Member;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header as JWTHeader, Validation,
};
use rocket::http::Status;
use rocket::response::status;
use std::env;

// JWT Member Authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub member_id: i32,
    pub is_admin: bool,
    pub exp: usize,
}
#[derive(Debug)]
pub struct JWT {
    pub claims: JwtClaims,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = status::Custom<String>;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("authorization") {
            None => Outcome::Error((
                Status::Unauthorized,
                status::Custom(
                    Status::Unauthorized,
                    String::from("Error validating JWT token - No token provided"),
                ),
            )),
            Some(key) => match JWT::validate_jwt(key) {
                Ok(claims) => Outcome::Success(JWT { claims }),
                Err(err) => match &err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => Outcome::Error((
                        Status::Unauthorized,
                        status::Custom(
                            Status::Unauthorized,
                            String::from("Error validating JWT token - Expired Token"),
                        ),
                    )),
                    jsonwebtoken::errors::ErrorKind::InvalidToken => Outcome::Error((
                        Status::Unauthorized,
                        status::Custom(
                            Status::Unauthorized,
                            String::from("Error validating JWT token - Invalid Token"),
                        ),
                    )),
                    _ => Outcome::Error((
                        Status::Unauthorized,
                        status::Custom(
                            Status::Unauthorized,
                            format!("Error validating JWT token - {}", err),
                        ),
                    )),
                },
            },
        }
    }
}
impl JWT {
    pub fn create_jwt(member: &Member) -> Result<String, AuthError> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        let claims = JwtClaims {
            member_id: member.id(),
            is_admin: false,
            exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
        };
        encode(&JWTHeader::new(Algorithm::HS512), &claims, &encoding_key).map_err(|e| {
            log::error!("Error creating JWT token: {}", e);
            AuthError::InternalError("Token creation failure".to_string())
        })
    }

    fn validate_jwt(token: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        let token = token.trim_start_matches("Bearer").trim();

        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = true;
        validation.leeway = 0;

        match decode::<JwtClaims>(&token, &decoding_key, &validation) {
            Ok(token) => Ok(token.claims),
            Err(err) => Err(err),
        }
    }
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
