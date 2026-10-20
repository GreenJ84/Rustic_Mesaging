use chrono::NaiveDateTime;
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::AsChangeset;
use diesel::{Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::{Deserialize, Serialize};

use crate::member::error::MemberError;
use crate::schema::member;

// Data Models ==========================
#[derive(Insertable)]
#[diesel(table_name = member)]
pub(crate) struct NewMember {
    username: String,
    password_hash: String,
    email: String,
}
impl NewMember {
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn new(username: String, password_hash: String, email: String) -> Self {
        Self {
            username,
            password_hash,
            email,
        }
    }
}

pub type MemberId = i32;

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = member)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Member {
    #[diesel(sql_type = Int4)]
    id: MemberId,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Text)]
    password_hash: String,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    avatar: String,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl Member {
    // Encapsulation (Getter for id)
    pub fn id(&self) -> MemberId {
        self.id
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn avatar(&self) -> &str {
        &self.avatar
    }

    pub fn into_profile(self) -> MemberProfile {
        MemberProfile {
            id: self.id,
            username: self.username,
            email: self.email,
            avatar: self.avatar,
            created_at: self.created_at,
        }
    }
    pub fn into_short(self) -> MemberShort {
        MemberShort {
            id: self.id,
            username: self.username,
            avatar: self.avatar,
        }
    }
}

const ALLOWED_AVATARS: &[&str] = &[
    "mouse",
    "panda",
    "bear",
    "bunny",
    "dog",
    "gorilla",
    "koala",
    "robot",
    "astronaut",
    "cat",
    "eagle",
    "pikachu",
    "default",
];
#[derive(AsChangeset, Debug, FromForm)]
#[diesel(table_name = member)]
pub(crate) struct MemberChangeset {
    #[field(validate = len(3..=125).or_else(msg!("Username must be between 3 and 125 characters long.")))]
    pub username: Option<String>,
    #[field(validate = contains('@').or_else(msg!("Invalid email address")))]
    pub email: Option<String>,
    pub avatar: Option<String>,
}
impl MemberChangeset {
    pub fn is_empty(&self) -> bool {
        self.username.is_none() && self.email.is_none() && self.avatar.is_none()
    }
    pub(crate) fn normalize_and_validate(mut self) -> Result<Self, MemberError> {
        if self.is_empty() {
            return Err(MemberError::InvalidChangeset("No changes provided".into()));
        }
        if let Some(username) = self.username.take() {
            let username = username.trim().to_owned();

            if !(3..=125).contains(&username.len()) {
                return Err(MemberError::InvalidChangeset(
                    "Username must be between 3 and 125 characters".into(),
                ));
            }
            self.username = Some(username);
        }

        if let Some(email) = self.email.take() {
            let email = email.trim().to_owned();
            if email.len() > 254 || !email.contains('@') {
                return Err(MemberError::InvalidChangeset(
                    "Invalid email address".into(),
                ));
            }

            self.email = Some(email);
        }

        if let Some(avatar) = self.avatar.take() {
            let avatar = avatar.trim().to_owned();
            if !ALLOWED_AVATARS.contains(&avatar.as_str()) {
                return Err(MemberError::InvalidChangeset("Invalid avatar value".into()));
            }
            self.avatar = Some(avatar);
        }

        Ok(self)
    }
}

// Data Transfer Objects ==========================
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = member)]
pub struct MemberProfile {
    pub id: MemberId,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for MemberProfile {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = member)]
pub struct MemberShort {
    pub id: MemberId,
    pub username: String,
    pub avatar: String,
}
impl<'r> Responder<'r, 'r> for MemberShort {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemberList<T> {
    pub members: Vec<T>,
}
impl<'r, T: Serialize> Responder<'r, 'r> for MemberList<T> {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}
pub type MultiMember = MemberList<MemberProfile>;
pub type MultiMemberPreview = MemberList<MemberShort>;
