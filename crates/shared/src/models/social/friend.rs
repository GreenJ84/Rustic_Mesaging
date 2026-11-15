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

#[cfg(test)]
mod tests {
    use super::{Friend, FriendList};

    #[test]
    fn friendship_contract_preserves_canonical_member_ids() {
        let created_at = chrono::NaiveDateTime::default();
        let value = serde_json::to_value(Friend {
            member_id: 4,
            friend_id: 9,
            created_at,
        })
        .unwrap();

        assert_eq!(value["member_id"], 4);
        assert_eq!(value["friend_id"], 9);
        assert_eq!(value["created_at"], serde_json::json!(created_at));
    }

    #[test]
    fn friend_list_uses_a_stable_friends_envelope() {
        let value = serde_json::to_value(FriendList::<serde_json::Value> {
            friends: Vec::new(),
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "friends": [] }));
    }
}
