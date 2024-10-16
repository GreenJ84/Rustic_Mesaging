use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::models::member::Member;
use crate::schema::friend_request;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = friend_request)]
pub struct NewFriendRequest{
    sender_id: i32,
    receiver_id: i32,
    note: String
}
impl NewFriendRequest {
    pub fn sender_id(&self) -> i32 { self.sender_id }
    pub fn receiver_id(&self) -> i32 { self.receiver_id }
    pub fn note(&self) -> &str { &self.note }

    // Order id's to avoid duplication
    pub fn new(sender_id: i32, receiver_id: i32, note: String) -> Self {
        Self { sender_id, receiver_id, note }
    }
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = friend_request)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FriendRequest{
    #[diesel(sql_type = Int4)]
    sender_id: i32,
    #[diesel(sql_type = Int4)]
    receiver_id: i32,
    #[diesel(sql_type = Text)]
    note: Option<String>,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for FriendRequest {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl FriendRequest {
    pub fn sender_id(&self) -> i32 { self.sender_id }
    pub fn receiver_id(&self) -> i32 { self.receiver_id }
    pub fn note(&self) -> &Option<String> { &self.note }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }

    pub fn new(
        sender_id: i32,
        receiver_id: i32,
        note: Option<String>,
        created_at: NaiveDateTime
    ) -> Self {
        Self { sender_id, receiver_id, note, created_at }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RequestDetailed {
    pub(crate) member: Member,
    pub(crate) note: Option<String>,
    pub(crate) created_at: NaiveDateTime,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MultiFriendRequest {
    pub(crate) incoming: Vec<RequestDetailed>,
    pub(crate) outgoing: Vec<RequestDetailed>,
}
impl<'r> Responder<'r, 'r> for MultiFriendRequest {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}