use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::models::member::{Member, MemberShort};
use crate::schema::friend;

#[derive(Insertable, Serialize, Deserialize, Debug, FromForm)]
#[diesel(table_name = friend)]
pub struct NewFriend{
    member_id: i32,
    friend_id: i32
}
impl NewFriend {
    pub fn member_id(&self) -> i32 { self.member_id }
    pub fn friend_id(&self) -> i32 { self.friend_id }

    // Order id's to avoid duplication
    pub fn new(member_id: i32, friend_id: i32) -> Self {
        if member_id < friend_id{
            Self { member_id, friend_id }
        } else {
            Self { member_id: friend_id, friend_id: member_id}
        }
    }
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = friend)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Friend{
    #[diesel(sql_type = Int4)]
    member_id: i32,
    #[diesel(sql_type = Int4)]
    friend_id: i32,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Friend {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
impl Friend {
    pub fn member_id(&self) -> i32 { self.member_id }
    pub fn friend_id(&self) -> i32 { self.friend_id }
    pub fn created_at(&self) -> NaiveDateTime { self.created_at }

    pub fn new(
        member_id: i32,
        friend_id: i32,
        created_at: NaiveDateTime
    ) -> Self {
        Self { member_id, friend_id, created_at }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullFriend{
    pub(crate) member: MemberShort,
    pub(crate) created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for FullFriend {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}
#[derive(Serialize, Debug)]
pub struct MultiFriend {
    pub(crate) friends: Vec<FullFriend>,
}
impl<'r> Responder<'r, 'r> for MultiFriend {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}