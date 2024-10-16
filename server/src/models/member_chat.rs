use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::schema::member_chat;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = member_chat)]
pub struct NewMemberChat{
    chat_id: i32,
    member_id: i32,
}
impl NewMemberChat {
    pub fn chat_id(&self) -> i32 { self.chat_id }
    pub fn member_id(&self) -> i32 { self.member_id }

    // Order id's to avoid duplication
    pub fn new(chat_id: i32, member_id: i32) -> Self {
        Self { chat_id, member_id }
    }
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = member_chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberChat{
    #[diesel(sql_type = Int4)]
    chat_id: i32,
    #[diesel(sql_type = Int4)]
    member_id: i32,
    #[diesel(sql_type = Timestamp)]
    joined_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for MemberChat {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl MemberChat {
    pub fn chat_id(&self) -> i32 { self.chat_id }
    pub fn member_id(&self) -> i32 { self.member_id }
    pub fn joined_at(&self) -> NaiveDateTime { self.joined_at }
}