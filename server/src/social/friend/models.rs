use crate::member::models::{Member, MemberId, MemberProfile, MemberShort};
use crate::schema::friend as friend_schema;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::{Deserialize, Serialize};

#[derive(Insertable)]
#[diesel(table_name = friend_schema)]
pub struct NewFriend {
    member_id: MemberId,
    friend_id: MemberId,
}
impl NewFriend {
    pub fn member_id(&self) -> MemberId {
        self.member_id
    }
    pub fn friend_id(&self) -> MemberId {
        self.friend_id
    }

    // Order id's to avoid duplication
    pub fn new(member_id: MemberId, friend_id: MemberId) -> Self {
        if member_id < friend_id {
            Self {
                member_id,
                friend_id,
            }
        } else {
            Self {
                member_id: friend_id,
                friend_id: member_id,
            }
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = friend_schema)]
pub struct Friendship {
    member_id: MemberId,
    friend_id: MemberId,
    created_at: NaiveDateTime,
}
impl Friendship {
    pub fn member_id(&self) -> MemberId {
        self.member_id
    }
    pub fn friend_id(&self) -> MemberId {
        self.friend_id
    }
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}

pub struct FullFriend {
    friend: Member,
    created_at: NaiveDateTime,
}
impl FullFriend {
    pub fn friend(&self) -> &Member {
        &self.friend
    }
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    pub fn into_full(self) -> Friend {
        Friend {
            member: self.friend.into_profile(),
            created_at: self.created_at,
        }
    }
    pub fn into_preview(self) -> FriendPreview {
        FriendPreview {
            member: self.friend.into_short(),
            created_at: self.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Friend {
    pub(crate) member: MemberProfile,
    pub(crate) created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for Friend {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FriendPreview {
    pub(crate) member: MemberShort,
    pub(crate) created_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for FriendPreview {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct FriendList<T> {
    pub(crate) friends: Vec<T>,
}
impl<'r, T> Responder<'r, 'r> for FriendList<T>
where
    T: Serialize,
{
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?).ok()
    }
}
pub type MultiFriend = FriendList<Friend>;
pub type MultiFriendPreview = FriendList<FriendPreview>;
