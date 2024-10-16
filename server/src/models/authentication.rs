use std::env;
use rocket::http::Status;
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use rocket::response::{Responder, status};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header as JWTHeader, Algorithm, Validation, EncodingKey, DecodingKey};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use rocket::http::Header;
use rocket::Response;
use rocket::serde::json::Json;
use crate::db::DbConn;
use crate::models::member::{Member, MemberSafe, NewMember};
use crate::service::member_service::MemberService;

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct Register<'a> {
    #[field(validate = len(3..=250))]
    pub username: &'a str,
    #[field(validate = len(8..=80))]
    pub password: &'a str,
    #[field(validate = len(8..=80))]
    pub confirm_password: &'a str,
    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: &'a str,
}
impl Register<'_>{
    pub fn validate(
        &self,
        conn: &mut DbConn,
    ) -> Result<NewMember, (Status, String)>{

        if self.password.ne(self.confirm_password){
            return Err((Status::BadRequest, String::from("Confirm Password does not match Password")));
        }
        match MemberService::find_by_username(conn, self.username.to_string()){
            Ok(_) => { return Err((Status::Conflict, String::from("User already exists."))); }
            Err(_) => {
                let new_member = NewMember::new(
                    self.username.to_string(),
                    Self::hash_password(self.password),
                    self.email.to_string(),
                );
                Ok(new_member)
            }
        }
    }
    pub fn hash_password(password: &str) -> String {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        return match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hashed_password) => {
                hashed_password.to_string()
            }
            Err(e) => {
                // Handle the error, e.g., log it or return an error response
                eprintln!("Failed to hash password: {}", e);
                String::new()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct Login<'a> {
    #[field(validate = len(3..=250))]
    pub username: &'a str,
    #[field(validate = len(8..=80))]
    pub password: &'a str,
}

// JWT Member Authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub member_id: i32,
    pub is_admin: bool,
    pub exp: usize,
}
#[derive(Debug)]
pub struct JWT {
    pub claims: JwtClaims
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = status::Custom<String>;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("authorization") {
            None => {
                Outcome::Error((Status::Unauthorized, status::Custom(Status::Unauthorized, String::from("Error validating JWT token - No token provided"))))
            },
            Some(key) => match JWT::validate_jwt(key) {
                Ok(claims) => Outcome::Success(JWT {claims}),
                Err(err) => match &err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        Outcome::Error((Status::Unauthorized, status::Custom(Status::Unauthorized, String::from("Error validating JWT token - Expired Token"))))
                    },
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        Outcome::Error((Status::Unauthorized, status::Custom(Status::Unauthorized,  String::from("Error validating JWT token - Invalid Token"))))
                    },
                    _ => {
                        Outcome::Error((Status::Unauthorized, status::Custom(Status::Unauthorized,  format!("Error validating JWT token - {}", err))))
                    }
                }
            },
        }
    }
}
impl JWT {
    pub fn create_jwt(member: &Member) -> Result<String, jsonwebtoken::errors::Error> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        let claims = JwtClaims {
            member_id: member.id(),
            is_admin: member.is_admin(),
            exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
        };
        encode(&JWTHeader::new(Algorithm::HS512), &claims, &encoding_key)
    }

    fn validate_jwt(token: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        let token = token.trim_start_matches("Bearer").trim();

        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = true;
        validation.leeway = 0;

        match decode::<JwtClaims>(
            &token,
            &decoding_key,
            &validation,
        ) {
            Ok(token) => Ok(token.claims),
            Err(err) => Err(err)
        }
    }
}


// Secret Administration passkey
pub struct AdminKey {
    pub(crate) passkey: bool
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
    pub member: MemberSafe,
    pub token: String // String JWT encoding
}
impl<'r> Responder<'r, 'r> for AuthenticationResponse {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self.member).respond_to(req)?)
            .header(Header::new("Authorization", format!("Bearer {}", self.token)))
            .ok()
    }
}

