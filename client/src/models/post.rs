use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};
use crate::models::friend_request::RequestDetailed;
use crate::models::member::MemberShort;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewPost{
    content: String,
    author_id: i32,
    channel_id: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Post{
    id: i32,
    pub(crate) content: String,
    author_id: i32,
    channel_id: i32,
    #[serde(serialize_with ="serialize_naive_date_time", deserialize_with = "deserialize_naive_date_time")]
    created_at: NaiveDateTime,
}

impl Post {
    pub fn created_at(self: &Self) -> String{
        self.created_at.format("%m/%d/%Y %I:%M %p").to_string()
    }
}

impl Post{
    pub fn default() -> Self{
        Self {
            id: 0,
            content: "Post".to_string(),
            author_id: 0,
            channel_id: 0,
            created_at: NaiveDateTime::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PostDetail{
    pub(crate) id: i32,
    pub(crate) content: String,
    pub(crate) author: MemberShort,
    pub(crate) channel_id: i32,
    pub(crate) created_at: NaiveDateTime,
}
impl PostDetail{
    pub fn created_at(self: &Self) -> String{
        self.created_at.format("%m/%d/%Y %I:%M %p").to_string()
    }
    pub fn new(
        id: i32,
        content: String,
        author: MemberShort,
        channel_id: i32,
        created_at: NaiveDateTime
    ) -> Self{
        Self { id, content, author, channel_id, created_at }
    }
    pub fn default() -> Self{
        Self {
            id: 0,
            content: "Post".to_string(),
            author: MemberShort::default(),
            channel_id: 0,
            created_at: NaiveDateTime::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MultiPost {
    pub(crate) posts: Vec<PostDetail>,
}
impl MultiPost {
    pub fn new(
        posts: Vec<PostDetail>,
    ) -> Self {
        Self {
            posts
        }
    }
    pub fn default() -> Self {
        Self::new(
            Vec::<PostDetail>::new()
        )

    }
}