use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::member::{MemberId, MemberShort};
use crate::models::social::friend::FriendDetails;

/// Input used to create a friend request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewFriendRequestInput {
    pub receiver_id: MemberId,
    pub note: Option<String>,
}

/// Input used to update the note on an outgoing friend request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendRequestChangesetInput {
    pub note: Option<String>,
}

/// Detailed friend-request response relative to the requesting member.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendRequestDetailed {
    pub member: MemberShort,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Stored friend-request response without an expanded member projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendRequest {
    pub sender_id: MemberId,
    pub receiver_id: MemberId,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Incoming and outgoing friend requests for one member.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MultiFriendRequest {
    pub incoming: Vec<FriendRequestDetailed>,
    pub outgoing: Vec<FriendRequestDetailed>,
}

/// Collection wrapper for friend-request responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FriendRequestList<T> {
    pub requests: Vec<T>,
}

pub type MultiFriendRequestPreview = FriendRequestList<FriendRequestDetailed>;

/// Result of creating a request when an inverse request may be auto-accepted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum FriendRequestCreation {
    Friend(FriendDetails),
    Request(FriendRequestDetailed),
}
