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
    pub(crate) avatar: String,
    pub(crate) created_at: NaiveDateTime,
}
impl MemberDefaults for Member {
    fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            email: "member.email@provider".to_string(),
            avatar: String::from("default"),
            created_at: NaiveDateTime::default(),
        }
    }
}
impl AvatarRendering for Member {
    fn avatar(self, class_name: &str, size: &str) -> Html {
        member_avatars(&class_name, &size)
            .get(&self.avatar)
            .unwrap()
            .to_owned()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemberShort {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) avatar: String,
}
impl MemberDefaults for MemberShort {
    fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            avatar: String::from("default"),
        }
    }
}
impl AvatarRendering for MemberShort {
    fn avatar(self, class_name: &str, size: &str) -> Html {
        member_avatars(&class_name, &size)
            .get(&self.avatar)
            .unwrap()
            .to_owned()
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MultiMember {
    pub(crate) members: Vec<MemberShort>,
}
impl MemberDefaults for MultiMember {
    fn default() -> Self {
        Self {
            members: Vec::<MemberShort>::new(),
        }
    }
}

pub(crate) trait MemberDefaults {
    fn default() -> Self;
}
pub(crate) trait AvatarRendering {
    fn avatar(self, class_name: &str, size: &str) -> Html;
}
