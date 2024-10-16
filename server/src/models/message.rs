use diesel::{Identifiable, Insertable, Queryable, Selectable};
use chrono::NaiveDateTime;
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::models::member::MemberShort;
use crate::schema::message;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = message)]
pub struct NewMessage{
   content: String,
   sender_id: i32,
   chat_id: i32,
}
impl NewMessage {
    pub fn content(&self) -> &str { &self.content }
    pub fn sender_id(&self) -> &i32 { &self.sender_id }
    pub fn chat_id(&self) -> i32 { self.chat_id }

    pub fn new(content: String, sender_id: i32, chat_id: i32) -> Self {
        Self { content, sender_id, chat_id }
    }
}

#[derive(Identifiable, Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Chat, foreign_key = chat_id))]
#[diesel(belongs_to(Member, foreign_key = sender_id))]
pub struct Message{
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Int4)]
    sender_id: i32,
    #[diesel(sql_type = Int4)]
    chat_id: i32,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Message {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl Message {
    pub fn id(&self) -> i32 { self.id }
    pub fn content(&self) -> &str { &self.content }
    pub fn sender_id(&self) -> i32 { self.sender_id }
    pub fn chat_id(&self) -> i32 { self.chat_id }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }
}


#[derive(Serialize, Debug)]
pub struct ResponseMessage {
    pub(crate) id: i32,
    pub(crate) content: String,
    pub(crate) sender: MemberShort,
    pub(crate) created_at: NaiveDateTime,
}
#[derive(Serialize, Debug)]
pub struct MessageThread {
    pub(crate) thread: Vec<ResponseMessage>,
}
impl<'r> Responder<'r, 'r> for MessageThread {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}