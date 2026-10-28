use chrono::NaiveDateTime;
use diesel::AsChangeset;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::{Deserialize, Serialize};

use crate::member::error::MemberError;
use crate::schema::member;
use crate::utils::Validator;

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
#[derive(FromForm)]
pub(crate) struct MemberUpdateInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
}
impl MemberUpdateInput {
    pub fn is_empty(&self) -> bool {
        self.username.is_none() && self.email.is_none() && self.avatar.is_none()
    }
}
#[derive(AsChangeset, Debug)]
#[diesel(table_name = member)]
pub(crate) struct MemberChangeset {
    pub username: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
}
impl TryFrom<MemberUpdateInput> for MemberChangeset {
    type Error = MemberError;

    fn try_from(input: MemberUpdateInput) -> Result<MemberChangeset, Self::Error> {
        if input.is_empty() {
            return Err(MemberError::InvalidChangeset(
                "No changes provided".to_string(),
            ));
        }
        let username = if let Some(username) = input.username {
            let username = Validator::validate_text(username, Some(3), Some(125))
                .map_err(|e| MemberError::InvalidChangeset(format!("Username {}", e.message())))?;
            Some(username)
        } else {
            None
        };

        let email = if let Some(email) = input.email {
            let email = Validator::validate_text(email, Some(6), Some(254))
                .map_err(|e| MemberError::InvalidChangeset(format!("Email {}", e.message())))?;
            if !email.contains('@') {
                return Err(MemberError::InvalidChangeset(
                    "Email must contain '@'".to_string(),
                ));
            }
            Some(email)
        } else {
            None
        };

        let avatar = if let Some(avatar) = input.avatar {
            let avatar = avatar.trim().to_owned();
            if !ALLOWED_AVATARS.contains(&avatar.as_str()) {
                return Err(MemberError::InvalidChangeset("Invalid avatar value".into()));
            }
            Some(avatar)
        } else {
            None
        };
        Ok(MemberChangeset {
            username,
            email,
            avatar,
        })
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

#[cfg(test)]
mod tests {
    use super::{MemberChangeset, NewMember};
    use crate::member::{error::MemberError, models::MemberUpdateInput};

    #[test]
    fn new_member_exposes_its_registration_values() {
        let member = NewMember::new(
            "Alice".to_string(),
            "password-hash".to_string(),
            "alice@example.com".to_string(),
        );

        assert_eq!(member.username(), "Alice");
        assert_eq!(member.password_hash(), "password-hash");
        assert_eq!(member.email(), "alice@example.com");
    }

    #[test]
    fn changesets_are_trimmed_without_changing_case() {
        let changeset: MemberChangeset = MemberUpdateInput {
            username: Some("  Alice  ".to_string()),
            email: Some("  Alice@Example.com  ".to_string()),
            avatar: Some("  robot  ".to_string()),
        }
        .try_into()
        .unwrap();

        assert_eq!(changeset.username.as_deref(), Some("Alice"));
        assert_eq!(changeset.email.as_deref(), Some("Alice@Example.com"));
        assert_eq!(changeset.avatar.as_deref(), Some("robot"));
    }

    #[test]
    fn rejects_empty_changesets() {
        let result: Result<MemberChangeset, MemberError> = MemberUpdateInput {
            username: None,
            email: None,
            avatar: None,
        }
        .try_into();

        assert!(matches!(result, Err(MemberError::InvalidChangeset(_))));
    }

    #[test]
    fn rejects_invalid_member_values() {
        for changeset in [
            MemberUpdateInput {
                username: Some("ab".to_string()),
                email: None,
                avatar: None,
            },
            MemberUpdateInput {
                username: None,
                email: Some("invalid".to_string()),
                avatar: None,
            },
            MemberUpdateInput {
                username: None,
                email: None,
                avatar: Some("unknown".to_string()),
            },
        ] {
            let change_set_result: Result<MemberChangeset, MemberError> = changeset.try_into();
            assert!(matches!(
                change_set_result,
                Err(MemberError::InvalidChangeset(_))
            ));
        }
    }
}
