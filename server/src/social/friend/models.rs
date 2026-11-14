use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};

use crate::member::models::{
    MemberDetailed, MemberDetailedRow, MemberId, MemberShort, MemberShortRow,
};
use crate::schema::friend as friend_schema;
use shared::models::social::friend::{Friend, FriendDetails, FriendShort};

#[derive(Insertable)]
#[diesel(table_name = friend_schema)]
pub(in crate::social) struct NewFriend {
    member_id: MemberId,
    friend_id: MemberId,
}
impl NewFriend {
    pub(in crate::social) fn new(member_id: MemberId, friend_id: MemberId) -> Self {
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
pub(in crate::social) struct FriendRow {
    member_id: MemberId,
    friend_id: MemberId,
    created_at: NaiveDateTime,
}

impl FriendRow {
    pub(in crate::social) fn member_id(&self) -> MemberId {
        self.member_id
    }

    pub(in crate::social) fn friend_id(&self) -> MemberId {
        self.friend_id
    }

    pub(in crate::social) fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}

impl From<FriendRow> for Friend {
    fn from(row: FriendRow) -> Self {
        Self {
            member_id: row.member_id,
            friend_id: row.friend_id,
            created_at: row.created_at,
        }
    }
}

pub(in crate::social) struct FriendDetailedRow {
    pub(super) friend: MemberDetailedRow,
    pub(super) created_at: NaiveDateTime,
}

impl From<FriendDetailedRow> for FriendDetails {
    fn from(row: FriendDetailedRow) -> Self {
        Self {
            friend: MemberDetailed::from(row.friend),
            created_at: row.created_at,
        }
    }
}

pub(in crate::social) struct FriendShortRow {
    pub(super) friend: MemberShortRow,
    pub(super) created_at: NaiveDateTime,
}

impl From<FriendShortRow> for FriendShort {
    fn from(row: FriendShortRow) -> Self {
        Self {
            friend: MemberShort::from(row.friend),
            created_at: row.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_friendship, FriendRow, NewFriend};
    use shared::models::social::friend::Friend;

    #[test]
    fn normalizes_friendship_ids_into_canonical_order() {
        assert_eq!(normalize_friendship(4, 9), (4, 9));
        assert_eq!(normalize_friendship(9, 4), (4, 9));
    }

    #[test]
    fn new_friend_uses_canonical_id_order() {
        let friend = NewFriend::new(9, 4);

        assert_eq!(friend.member_id, 4);
        assert_eq!(friend.friend_id, 9);
    }

    #[test]
    fn row_converts_to_shared_friend_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let friend = Friend::from(FriendRow {
            member_id: 4,
            friend_id: 9,
            created_at,
        });

        assert_eq!(friend.member_id, 4);
        assert_eq!(friend.friend_id, 9);
        assert_eq!(friend.created_at, created_at);
    }
}
