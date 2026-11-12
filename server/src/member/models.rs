use chrono::NaiveDateTime;
use diesel::AsChangeset;
use diesel::{Identifiable, Insertable, Queryable, Selectable};

pub(crate) use shared::models::member::*;

use super::error::MemberError;
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

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = member)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct MemberRow {
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

/// Persistence projection for member profile responses.
#[derive(Queryable, Selectable)]
#[diesel(table_name = member)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct MemberDetailedRow {
    id: MemberId,
    username: String,
    email: String,
    avatar: String,
    created_at: NaiveDateTime,
}
impl From<MemberDetailedRow> for MemberDetailed {
    fn from(member: MemberDetailedRow) -> MemberDetailed {
        MemberDetailed {
            id: member.id,
            username: member.username,
            email: member.email,
            avatar: member.avatar,
            created_at: member.created_at,
        }
    }
}

/// Persistence projection for public member previews.
#[derive(Queryable, Selectable)]
#[diesel(table_name = member)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct MemberShortRow {
    id: MemberId,
    username: String,
    avatar: String,
}
impl From<MemberShortRow> for MemberShort {
    fn from(member: MemberShortRow) -> MemberShort {
        MemberShort {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
        }
    }
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = member)]
pub(crate) struct MemberChangeset {
    pub(super) username: Option<String>,
    pub(super) email: Option<String>,
    pub(super) avatar: Option<String>,
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
            let username = Validator::validate_text(&username, Some(3), Some(125))
                .map_err(|e| MemberError::InvalidChangeset(format!("Username {}", e.message())))?;
            Some(username)
        } else {
            None
        };

        let email = if let Some(email) = input.email {
            let email = Validator::validate_text(&email, Some(6), Some(254))
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

#[cfg(test)]
mod tests {
    use super::{MemberChangeset, MemberDetailedRow, MemberRow, MemberShortRow, NewMember, MemberDetailed, MemberShort, MemberUpdateInput};
    use crate::member::error::MemberError;

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
    fn detailed_rows_convert_to_detailed_contracts() {
        let created_at = chrono::NaiveDateTime::default();
        let detailed = MemberDetailed::from(MemberDetailedRow {
            id: 7,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            avatar: "robot".to_string(),
            created_at,
        });

        assert_eq!(detailed.id, 7);
        assert_eq!(detailed.username, "alice");
        assert_eq!(detailed.email, "alice@example.com");
        assert_eq!(detailed.avatar, "robot");
        assert_eq!(detailed.created_at, created_at);
    }

    #[test]
    fn short_rows_convert_to_short_contracts() {
        let short = MemberShort::from(MemberShortRow {
            id: 7,
            username: "alice".to_string(),
            avatar: "robot".to_string(),
        });

        assert_eq!(short.id, 7);
        assert_eq!(short.username, "alice");
        assert_eq!(short.avatar, "robot");
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
