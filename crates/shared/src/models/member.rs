use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// Data Transfer Objects ==========================
pub type MemberId = i32;

pub const ALLOWED_AVATARS: &[&str] = &[
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

#[derive(Serialize, Deserialize, Debug)]
pub struct NewMemberInput {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MemberUpdateInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
}
impl MemberUpdateInput {
    pub fn is_empty(&self) -> bool {
        self.username.is_none() && self.email.is_none() && self.avatar.is_none()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemberDetailed {
    pub id: MemberId,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemberShort {
    pub id: MemberId,
    pub username: String,
    pub avatar: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MemberList<T> {
    pub members: Vec<T>,
}

pub type MultiMember = MemberList<MemberDetailed>;
pub type MultiMemberPreview = MemberList<MemberShort>;

#[cfg(test)]
mod tests {
    use super::{MemberList, MemberUpdateInput, NewMemberInput};

    #[test]
    fn update_input_is_empty_only_when_every_field_is_absent() {
        assert!(
            MemberUpdateInput {
                username: None,
                email: None,
                avatar: None,
            }
            .is_empty()
        );

        for input in [
            MemberUpdateInput {
                username: Some(String::new()),
                email: None,
                avatar: None,
            },
            MemberUpdateInput {
                username: None,
                email: Some(String::new()),
                avatar: None,
            },
            MemberUpdateInput {
                username: None,
                email: None,
                avatar: Some(String::new()),
            },
        ] {
            assert!(!input.is_empty());
        }
    }

    #[test]
    fn registration_input_serializes_all_client_owned_fields() {
        let value = serde_json::to_value(NewMemberInput {
            username: "alice".into(),
            password: "secret-password".into(),
            confirm_password: "secret-password".into(),
            email: "alice@example.com".into(),
        })
        .unwrap();

        assert_eq!(value["username"], "alice");
        assert_eq!(value["email"], "alice@example.com");
        assert_eq!(value["password"], "secret-password");
        assert_eq!(value["confirm_password"], "secret-password");
    }

    #[test]
    fn member_list_uses_a_stable_members_envelope() {
        let value = serde_json::to_value(MemberList::<serde_json::Value> {
            members: Vec::new(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "members": [] }));
    }
}
