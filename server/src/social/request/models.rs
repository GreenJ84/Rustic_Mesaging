use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};

use crate::member::models::{MemberId, MemberShort, MemberShortRow};
use crate::schema::friend_request;
use shared::models::social::request::{FriendRequest, FriendRequestDetailed};

#[derive(Insertable)]
#[diesel(table_name = friend_request)]
pub(in crate::social) struct NewFriendRequest {
    pub(in crate::social) sender_id: MemberId,
    pub(in crate::social) receiver_id: MemberId,
    pub(in crate::social) note: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = friend_request)]
#[diesel(treat_none_as_null = true)]
pub(in crate::social) struct FriendRequestChangeset {
    pub(in crate::social) note: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = friend_request)]
pub(in crate::social) struct FriendRequestRow {
    sender_id: MemberId,
    receiver_id: MemberId,
    note: Option<String>,
    created_at: NaiveDateTime,
}
impl FriendRequestRow {
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
impl From<FriendRequestRow> for FriendRequest {
    fn from(row: FriendRequestRow) -> Self {
        Self {
            sender_id: row.sender_id,
            receiver_id: row.receiver_id,
            note: row.note,
            created_at: row.created_at,
        }
    }
}

pub(in crate::social) struct FriendRequestDetailedRow {
    pub(super) member: MemberShortRow,
    pub(super) note: Option<String>,
    pub(super) created_at: NaiveDateTime,
}

impl From<FriendRequestDetailedRow> for FriendRequestDetailed {
    fn from(row: FriendRequestDetailedRow) -> Self {
        Self {
            member: MemberShort::from(row.member),
            note: row.note,
            created_at: row.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FriendRequestChangeset, FriendRequestRow, NewFriendRequest};
    use shared::models::social::request::FriendRequest;

    #[test]
    fn row_converts_to_shared_request_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let request = FriendRequest::from(FriendRequestRow {
            sender_id: 4,
            receiver_id: 9,
            note: Some("hello".into()),
            created_at,
        });

        assert_eq!(request.sender_id, 4);
        assert_eq!(request.receiver_id, 9);
        assert_eq!(request.note.as_deref(), Some("hello"));
        assert_eq!(request.created_at, created_at);
    }

    #[test]
    fn new_request_preserves_direction_and_optional_note() {
        let request = NewFriendRequest {
            sender_id: 4,
            receiver_id: 9,
            note: Some("hello".into()),
        };

        assert_eq!(request.sender_id, 4);
        assert_eq!(request.receiver_id, 9);
        assert_eq!(request.note.as_deref(), Some("hello"));
    }

    #[test]
    fn changeset_none_represents_clearing_the_note() {
        let changeset = FriendRequestChangeset { note: None };
        assert!(changeset.note.is_none());
    }

    #[test]
    fn row_accessors_expose_each_persistence_field() {
        let created_at = chrono::NaiveDateTime::default();
        let row = FriendRequestRow {
            sender_id: 4,
            receiver_id: 9,
            note: Some("hello".into()),
            created_at,
        };

        assert_eq!(row.sender_id(), 4);
        assert_eq!(row.receiver_id(), 9);
        assert_eq!(row.note(), Some("hello"));
        assert_eq!(row.created_at(), created_at);
    }
}
