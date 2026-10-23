use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::{Deserialize, Serialize};

use crate::member::models::{MemberShort, MemberId};
use crate::schema::friend_request;

#[derive(Insertable)]
#[diesel(table_name = friend_request)]
pub(in crate::social) struct NewFriendRequest {
    pub(in crate::social) sender_id: MemberId,
    pub(in crate::social) receiver_id: MemberId,
    pub(in crate::social) note: String,
}
impl NewFriendRequest {
    pub(in crate::social) fn sender_id(&self) -> MemberId {
        self.sender_id
    }
    pub(in crate::social) fn receiver_id(&self) -> MemberId {
        self.receiver_id
    }
    pub(in crate::social) fn note(&self) -> &str {
        &self.note
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = friend_request)]
pub(in crate::social) struct FriendRequest {
    sender_id: MemberId,
    receiver_id: MemberId,
    note: Option<String>,
    created_at: NaiveDateTime,
}
impl FriendRequest {
    pub(in crate::social) fn sender_id(&self) -> MemberId {
        self.sender_id
    }
    pub(in crate::social) fn receiver_id(&self) -> MemberId {
        self.receiver_id
    }
    pub(in crate::social) fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
    pub(in crate::social) fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestDetails {
    pub member: MemberShort,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

pub type IncomingRequests = Vec<RequestDetails>;
pub type OutgoingRequests = Vec<RequestDetails>;

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiFriendRequest {
    pub incoming: IncomingRequests,
    pub outgoing: OutgoingRequests,
}
impl<'r> Responder<'r, 'r> for MultiFriendRequest {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}