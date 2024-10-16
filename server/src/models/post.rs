use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::schema::post;
use crate::models::{
    channel::Channel,
    member::Member,
    member::MemberShort
};

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = post)]
pub struct NewPost{
    content: String,
    author_id: i32,
    channel_id: i32,
}

impl NewPost {
    pub fn content(&self) -> &str { &self.content }
    pub fn author_id(&self) -> &i32 { &self.author_id }
    pub fn channel_id(&self) -> i32 { self.channel_id }

    pub fn new(content: String, author_id: i32, channel_id: i32,) -> Self {
        Self { content, author_id, channel_id }
    }
}

#[derive(Identifiable, Queryable, Selectable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Channel, foreign_key = channel_id))]
#[diesel(belongs_to(Member, foreign_key = author_id))]
pub struct Post{
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Int4)]
    author_id: i32,
    #[diesel(sql_type = Int4)]
    channel_id: i32,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Post {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl Post {
    pub fn id(&self) -> i32 { self.id }
    pub fn content(&self) -> &str { &self.content }
    pub fn author_id(&self) -> i32 { self.author_id }
    pub fn channel_id(&self) -> i32 { self.channel_id }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDetail{
    id: i32,
    content: String,
    pub(crate) author: MemberShort,
    channel_id: i32,
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for PostDetail {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl PostDetail{
    pub fn new(
        id: i32,
        content: String,
        author: MemberShort,
        channel_id: i32,
        created_at: NaiveDateTime
    ) -> Self{
        Self { id, content, author, channel_id, created_at }
    }
}
#[derive(Serialize, Debug)]
pub struct MultiPost {
    pub(crate) posts: Vec<PostDetail>,
}
impl<'r> Responder<'r, 'r> for MultiPost {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}