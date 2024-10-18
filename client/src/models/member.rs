
use std::borrow::Borrow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UpdateMember {
    username: String,
    password: String,
    email: String,
    avatar: Option<String>,
    is_admin: bool,
}
impl UpdateMember {
    pub fn new(
        username: String,
        password: String,
        email: String,
        is_admin: bool,
        avatar: Option<String>,
    ) -> Self {
        Self {
            username,
            password,
            email,
            avatar,
            is_admin
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Member {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) avatar: Option<String>,
    pub(crate) is_admin: bool,
    #[serde(serialize_with ="serialize_naive_date_time", deserialize_with = "deserialize_naive_date_time")]
    pub(crate) created_at: NaiveDateTime
}
impl Member {
    pub fn new(
        id: i32,
        username: String,
        email: String,
        avatar: Option<String>,
        is_admin: bool,
        created_at: NaiveDateTime
    ) -> Self {
        Self {
            id,
            username,
            email,
            avatar,
            is_admin,
            created_at
        }
    }
    pub fn default() -> Self {
        Self::new(
            0,
            "guest".to_string(),
            "member.email@provider".to_string(),
            None,
            false,
            NaiveDateTime::default()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemberShort {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) avatar: Option<String>,
}
impl MemberShort {
    pub fn new(
        id: i32,
        username: String,
        avatar: Option<String>,
    ) -> Self {
        Self {
            id,
            username,
            avatar,
        }
    }
    pub fn default() -> Self {
        Self::new(
            0,
            "guest".to_string(),
            None,
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct MultiMember {
    pub(crate) members: Vec<MemberShort>,
}
impl MultiMember {
    pub fn new(
        members: Vec<MemberShort>
    ) -> Self {
        Self {
            members
        }
    }
    pub fn default() -> Self {
        Self::new(
            (1..15).into_iter().map(|_| { MemberShort::default() }).collect()
        )
    }
}
