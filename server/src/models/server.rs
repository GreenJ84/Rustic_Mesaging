use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::schema::server;
use crate::models::member::Member;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = server)]
pub struct NewServer{
    #[field(validate = len(3..=80).or_else(msg!("Name too long.")))]
    name: String,
    #[field(validate = len(3..=200).or_else(msg!("Description too long.")))]
    description: String,
    icon: Option<String>,
    #[field(validate = range(1..).or_else(msg!("Invalid owner_id.")))]
    owner_id: i32,
}

impl NewServer {
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn icon(&self) -> &Option<String> { &self.icon }
    pub fn owner_id(&self) -> i32 { self.owner_id }

    pub fn new(name: String, description: String, icon: Option<String>, owner_id: i32) -> Self {
        Self { name, description, icon, owner_id }
    }
}

#[derive(Identifiable, Queryable, Selectable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = server)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Member, foreign_key = owner_id))]
pub struct Server{
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Text)]
    description: String,
    #[diesel(sql_type = Nullable<TexT>)]
    icon: Option<String>,
    #[diesel(sql_type = Int4)]
    owner_id: i32,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Server {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}

impl Server {
    pub fn id(&self) -> i32 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn icon(&self) -> &Option<String> { &self.icon }
    pub fn owner_id(&self) -> i32 { self.owner_id }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }

    pub fn new(
        id: i32,
        name: String,
        description: String,
        icon: Option<String>,
        owner_id: i32,
        created_at: NaiveDateTime
    ) -> Self {
        Self { id, name, description, icon, owner_id, created_at }
    }
}

#[derive(Serialize, Debug)]
pub struct MultiServer {
    pub(crate) servers: Vec<Server>,
}
impl<'r> Responder<'r, 'r> for MultiServer {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}