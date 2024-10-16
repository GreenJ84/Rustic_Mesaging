use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::form::FromForm;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::schema::channel;
use crate::models::server::Server;


#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = channel)]
pub struct NewChannel{
    server_id: i32,
    name: String,
}

impl NewChannel {
    pub fn name(&self) -> &str { &self.name }
    pub fn server_id(&self) -> i32 { self.server_id }

    pub fn new(name: String, server_id: i32) -> Self {
        Self { name, server_id }
    }
}

#[derive(Identifiable, Queryable, Selectable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Server, foreign_key = server_id))]
pub struct Channel{
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Int4)]
    server_id: i32,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Channel {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}

impl Channel {
    pub fn id(&self) -> i32 { self.id }
    pub fn server_id(&self) -> i32 { self.server_id }
    pub fn name(&self) -> &str { &self.name }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }
}

#[derive(Serialize, Debug)]
pub struct MultiChannel {
    pub(crate) channels: Vec<Channel>,
}
impl<'r> Responder<'r, 'r> for MultiChannel {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}