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
    pub(crate) fn username(&self) -> &str {
        &self.username
    }
    pub(crate) fn password_hash(&self) -> &str {
        &self.password_hash
    }
    pub(crate) fn email(&self) -> &str {
        &self.email
    }

    pub(crate) fn new(username: String, password_hash: String, email: String) -> Self {
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
impl MemberRow {
    pub(crate) fn id(&self) -> MemberId {
        self.id
    }
    pub(crate) fn username(&self) -> &str {
        &self.username
    }
    pub(crate) fn password_hash(&self) -> &str {
        &self.password_hash
    }
    pub(crate) fn email(&self) -> &str {
        &self.email
    }
    pub(crate) fn avatar(&self) -> &str {
        &self.avatar
    }
    pub(crate) fn created_at(&self) -> &NaiveDateTime {
        &self.created_at
    }
}
impl From<MemberRow> for MemberDetailed {
    fn from(member: MemberRow) -> MemberDetailed {
        MemberDetailed {
            id: member.id,
            username: member.username,
            email: member.email,
            avatar: member.avatar,
            created_at: member.created_at,
        }
    }
}
impl From<MemberRow> for MemberShort {
    fn from(member: MemberRow) -> MemberShort {
        MemberShort {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
        }
    }
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
    use super::{
        MemberChangeset, MemberDetailed, MemberDetailedRow, MemberRow, MemberShort, MemberShortRow,
        MemberUpdateInput, NewMember, ALLOWED_AVATARS,
    };
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
    fn full_rows_expose_persistence_values_and_convert_to_detailed_contracts() {
        let created_at = chrono::NaiveDateTime::default();
        let row = MemberRow {
            id: 7,
            username: "alice".into(),
            password_hash: "password-hash".into(),
            email: "alice@example.com".into(),
            avatar: "robot".into(),
            created_at,
        };

        assert_eq!(row.id(), 7);
        assert_eq!(row.username(), "alice");
        assert_eq!(row.password_hash(), "password-hash");
        assert_eq!(row.email(), "alice@example.com");
        assert_eq!(row.avatar(), "robot");
        assert_eq!(*row.created_at(), created_at);

        let detailed = MemberDetailed::from(row);
        assert_eq!(detailed.id, 7);
        assert_eq!(detailed.email, "alice@example.com");
    }

    #[test]
    fn full_rows_convert_to_short_contracts_without_private_fields() {
        let short = MemberShort::from(MemberRow {
            id: 7,
            username: "alice".into(),
            password_hash: "password-hash".into(),
            email: "alice@example.com".into(),
            avatar: "robot".into(),
            created_at: chrono::NaiveDateTime::default(),
        });

        assert_eq!(short.id, 7);
        assert_eq!(short.username, "alice");
        assert_eq!(short.avatar, "robot");
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
    fn accepts_username_length_boundaries() {
        for username in ["a".repeat(3), "a".repeat(125)] {
            let changeset = MemberChangeset::try_from(MemberUpdateInput {
                username: Some(username.clone()),
                email: None,
                avatar: None,
            })
            .unwrap();
            assert_eq!(changeset.username.as_deref(), Some(username.as_str()));
        }
    }

    #[test]
    fn rejects_invalid_username_lengths() {
        for username in ["  ".to_string(), "ab".to_string(), "a".repeat(126)] {
            let result = MemberChangeset::try_from(MemberUpdateInput {
                username: Some(username),
                email: None,
                avatar: None,
            });
            assert!(matches!(result, Err(MemberError::InvalidChangeset(_))));
        }
    }

    #[test]
    fn accepts_email_length_boundaries() {
        for email in ["a@b.co".to_string(), format!("{}@x", "a".repeat(252))] {
            let changeset = MemberChangeset::try_from(MemberUpdateInput {
                username: None,
                email: Some(email.clone()),
                avatar: None,
            })
            .unwrap();
            assert_eq!(changeset.email.as_deref(), Some(email.as_str()));
        }
    }

    #[test]
    fn rejects_invalid_email_values() {
        for email in [
            "a@b".to_string(),
            "invalid-email".to_string(),
            format!("{}@x", "a".repeat(253)),
        ] {
            let result = MemberChangeset::try_from(MemberUpdateInput {
                username: None,
                email: Some(email),
                avatar: None,
            });
            assert!(matches!(result, Err(MemberError::InvalidChangeset(_))));
        }
    }

    #[test]
    fn accepts_every_supported_avatar() {
        for avatar in ALLOWED_AVATARS {
            let changeset = MemberChangeset::try_from(MemberUpdateInput {
                username: None,
                email: None,
                avatar: Some((*avatar).to_string()),
            })
            .unwrap();
            assert_eq!(changeset.avatar.as_deref(), Some(*avatar));
        }
    }

    #[test]
    fn rejects_unknown_or_empty_avatars() {
        for avatar in ["unknown", "   "] {
            let result = MemberChangeset::try_from(MemberUpdateInput {
                username: None,
                email: None,
                avatar: Some(avatar.to_string()),
            });
            assert!(matches!(result, Err(MemberError::InvalidChangeset(_))));
        }
    }
}
