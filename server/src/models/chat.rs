use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::schema::chat;

#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct NewChatWithMembers {
    pub name: String,
    pub members: Vec<i32>,
}
#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = chat)]
pub struct NewChat{
    pub name: String
}
impl NewChat {
    pub fn name(&self) -> &str { &self.name }

    pub fn new(name: String) -> Self { Self { name } }
}

#[derive(Identifiable, Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chat{
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Chat {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl Chat {
    pub fn id(&self) -> i32 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }
}

#[derive(Serialize, Debug)]
pub struct MultiChatPreview {
    pub(crate) chat_previews: Vec<Chat>,
}
impl<'r> Responder<'r, 'r> for MultiChatPreview {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}