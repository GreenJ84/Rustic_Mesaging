use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::member::{MemberDetailed, MemberId, MemberShort};

/// Stored friendship response without an expanded member projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Friend {
    pub member_id: MemberId,
    pub friend_id: MemberId,
    pub created_at: NaiveDateTime,
}

/// Detailed friendship response containing the friend's profile.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendDetails {
    pub friend: MemberDetailed,
    pub created_at: NaiveDateTime,
}

/// Preview friendship response containing the friend's short identity.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendShort {
    pub friend: MemberShort,
    pub created_at: NaiveDateTime,
}

/// Collection wrapper for friendship responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendList<T> {
    pub friends: Vec<T>,
}

pub type MultiFriend = FriendList<FriendDetails>;
pub type MultiFriendPreview = FriendList<FriendShort>;
