use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

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
pub struct MemberProfile {
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

pub type MultiMember = MemberList<MemberProfile>;
pub type MultiMemberPreview = MemberList<MemberShort>;
