use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::{Deserialize, Serialize};

use crate::member::models::{MemberId, MemberProfile, MemberShort};
use crate::schema::friend as friend_schema;

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
        let (member_id, friend_id) = normalize_friendship(member_id, friend_id);
        Self {
            member_id,
            friend_id,
        }
    }
}

pub(super) fn normalize_friendship(
    member_id: MemberId,
    friend_id: MemberId,
) -> (MemberId, MemberId) {
    if member_id < friend_id {
        (member_id, friend_id)
    } else {
        (friend_id, member_id)
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Friend {
    pub friend: MemberProfile,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FriendPreview {
    pub friend: MemberShort,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct FriendList<T> {
    pub friends: Vec<T>,
}

pub type MultiFriend = FriendList<Friend>;
pub type MultiFriendPreview = FriendList<FriendPreview>;
