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

#[cfg(test)]
mod tests {
    use super::{
        FriendRequestChangesetInput, FriendRequestCreation, FriendRequestDetailed,
        FriendRequestList, NewFriendRequestInput,
    };
    use crate::models::member::MemberShort;

    #[test]
    fn creation_input_contains_only_receiver_and_optional_note() {
        let value = serde_json::to_value(NewFriendRequestInput {
            receiver_id: 9,
            note: Some("hello".into()),
        })
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({ "receiver_id": 9, "note": "hello" })
        );
    }

    #[test]
    fn changeset_preserves_null_as_the_note_clear_operation() {
        let value = serde_json::to_value(FriendRequestChangesetInput { note: None }).unwrap();
        assert_eq!(value, serde_json::json!({ "note": null }));
    }

    #[test]
    fn request_list_uses_a_stable_requests_envelope() {
        let value = serde_json::to_value(FriendRequestList::<serde_json::Value> {
            requests: Vec::new(),
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "requests": [] }));
    }

    #[test]
    fn creation_result_uses_stable_externally_tagged_variants() {
        let value = serde_json::to_value(FriendRequestCreation::Request(FriendRequestDetailed {
            member: MemberShort {
                id: 9,
                username: "bob".into(),
                avatar: "default".into(),
            },
            note: None,
            created_at: chrono::NaiveDateTime::default(),
        }))
        .unwrap();

        assert!(value.get("Request").is_some());
        assert!(value.get("Friend").is_none());
    }
}
