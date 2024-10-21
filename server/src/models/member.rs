use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use rocket::http::Status;
use rocket::Response;
use rocket::serde::json::Json;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use diesel::sql_types::{Text, Timestamp, Integer, Nullable, Bool};

use crate::schema::member;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = member)]
pub struct NewMember {
    username: String,
    password: String,
    email: String,
    avatar: Option<String>,
    is_admin: bool,
}
impl NewMember {
    pub fn username(&self) -> &str { &self.username }
    pub fn password(&self) -> &str { &self.password }
    pub fn email(&self) -> &str { &self.email }
    pub fn avatar(&self) -> &Option<String> { &self.avatar }
    pub fn is_admin(&self) -> &bool { &self.is_admin }
    pub fn set_admin(&mut self) { self.is_admin = true; }

    pub fn new(username: String, password: String, email: String) -> Self {
        Self {
            username,
            password,
            email,
            avatar: None,
            is_admin: false,
        }
    }
}

#[derive(Identifiable, Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = member)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Text)]
    password: String,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Nullable<Text>)]
    avatar: Option<String>,
    #[diesel(sql_type = Bool)]
    is_admin: bool,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Member {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl Member {
    // Encapsulation (Getter for id)
    pub fn id(&self) -> i32 { self.id }
    pub fn username(&self) -> &str { &self.username }
    pub fn password(&self) -> &str { &self.password }
    pub fn email(&self) -> &str { &self.email }
    pub fn avatar(&self) -> &Option<String> { &self.avatar }
    pub fn is_admin(&self) -> bool { self.is_admin }

    pub fn validate_password(&self, password: &str) -> Result<(), (Status, String)>{
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&self.password)
            .map_err(|_| { (Status::InternalServerError, String::from("Hashing error.")) })?;

        argon2.verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| { (Status::Unauthorized, String::from("Username or Password is incorrect.")) })?;

        Ok(())
    }
    pub fn into_safe(self) -> MemberSafe {
        MemberSafe {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            avatar: self.avatar.clone(),
            is_admin: self.is_admin,
            created_at: self.created_at
        }
    }
    pub fn into_short(self) -> MemberShort {
        MemberShort {
            id: self.id,
            username: self.username.clone(),
            avatar: self.avatar.clone(),
        }
    }
}

// API Response Types
#[derive(Serialize, QueryableByName, Deserialize, Debug, Clone)]
pub struct MemberSafe {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Nullable<Text>)]
    avatar: Option<String>,
    #[diesel(sql_type = Bool)]
    is_admin: bool,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for MemberSafe {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
#[derive(Serialize, Debug)]
pub struct MultiMember {
    pub(crate) members: Vec<MemberSafe>,
}
impl<'r> Responder<'r, 'r> for MultiMember {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberShort {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) avatar: Option<String>,
}
impl<'r> Responder<'r, 'r> for MemberShort {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
#[derive(Serialize, Debug)]
pub struct MultiMemberPreview {
    pub(crate) members: Vec<MemberShort>,
}
impl<'r> Responder<'r, 'r> for MultiMemberPreview {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
