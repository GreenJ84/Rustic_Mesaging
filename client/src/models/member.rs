use crate::comps::icon::member_avatars;
use crate::models::friend_request::RequestDetailed;
use crate::models::{deserialize_naive_date_time, serialize_naive_date_time};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use yew::{html, Html};

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
    username: Option<String>,
    email: Option<String>,
    avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Member {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) avatar: Option<String>,
    pub(crate) created_at: NaiveDateTime,
}
impl Member {
    pub fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            email: "member.email@provider".to_string(),
            avatar: None,
            created_at: NaiveDateTime::default(),
        }
    }

    pub fn avatar(self, class_name: &str, size: &str) -> Html {
        match self.avatar {
            Some(option) => member_avatars(&class_name, &size)
                .get(&option)
                .unwrap()
                .to_owned(),
            None => member_avatars(&class_name, &size)
                .get("default")
                .unwrap()
                .to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemberShort {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) avatar: Option<String>,
}
impl MemberShort {
    pub fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            avatar: None,
        }
    }
    pub fn avatar(self, class_name: &str, size: &str) -> Html {
        match self.avatar {
            Some(option) => member_avatars(&class_name, &size)
                .get(&option)
                .unwrap()
                .to_owned(),
            None => member_avatars(&class_name, &size)
                .get("default")
                .unwrap()
                .to_owned(),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MultiMember {
    pub(crate) members: Vec<MemberShort>,
}
impl MultiMember {
    pub fn default() -> Self {
        Self { members: Vec::<MemberShort>::new() }
    }
}
