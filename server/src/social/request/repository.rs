use diesel::dsl::case_when;
use diesel::prelude::*;

use super::models::{
    FriendRequestChangeset, FriendRequestDetailedRow, FriendRequestRow, NewFriendRequest,
};
use crate::infra::{DbConn, DbError};
use crate::member::models::{MemberId, MemberShortRow};
use crate::schema::{friend_request::dsl as request_dsl, member::dsl as member_dsl};

/// Direction of a request relative to the first member passed to the repository.
pub(in crate::social) enum FriendRequestDirection {
    Incoming,
    Outgoing,
}

pub(in crate::social) struct FriendRequestRepository;

impl FriendRequestRepository {
    pub(in crate::social) fn create(
        conn: &mut DbConn,
        new_request: NewFriendRequest,
    ) -> Result<FriendRequestRow, DbError> {
        diesel::insert_into(request_dsl::friend_request)
            .values(&new_request)
            .returning(FriendRequestRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_direction_for_update(
        conn: &mut DbConn,
        member_id: MemberId,
        other_member_id: MemberId,
    ) -> Result<Option<FriendRequestDirection>, DbError> {
        request_dsl::friend_request
            .filter(
                request_dsl::sender_id
                    .eq(member_id)
                    .and(request_dsl::receiver_id.eq(other_member_id))
                    .or(request_dsl::sender_id
                        .eq(other_member_id)
                        .and(request_dsl::receiver_id.eq(member_id))),
            )
            .for_update()
            .select(request_dsl::sender_id)
            .first::<MemberId>(conn)
            .optional()
            .map(|sender_id| {
                sender_id.map(|sender_id| {
                    if sender_id == member_id {
                        FriendRequestDirection::Outgoing
                    } else {
                        FriendRequestDirection::Incoming
                    }
                })
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<Option<FriendRequestRow>, DbError> {
        request_dsl::friend_request
            .find((sender_id, receiver_id))
            .select(FriendRequestRow::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_detailed(
        conn: &mut DbConn,
        member_id: MemberId,
        other_member_id: MemberId,
    ) -> Result<Option<FriendRequestDetailedRow>, DbError> {
        let related_member_id = case_when(
            request_dsl::sender_id.eq(member_id),
            request_dsl::receiver_id,
        )
        .otherwise(request_dsl::sender_id);

        request_dsl::friend_request
            .filter(
                request_dsl::sender_id
                    .eq(member_id)
                    .and(request_dsl::receiver_id.eq(other_member_id))
                    .or(request_dsl::sender_id
                        .eq(other_member_id)
                        .and(request_dsl::receiver_id.eq(member_id))),
            )
            .inner_join(member_dsl::member.on(member_dsl::id.eq(related_member_id)))
            .select((
                MemberShortRow::as_select(),
                request_dsl::note,
                request_dsl::created_at,
            ))
            .first(conn)
            .optional()
            .map(|row| {
                row.map(|(member, note, created_at)| FriendRequestDetailedRow {
                    member,
                    note,
                    created_at,
                })
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_incoming(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Vec<FriendRequestDetailedRow>, DbError> {
        request_dsl::friend_request
            .filter(request_dsl::receiver_id.eq(member_id))
            .inner_join(member_dsl::member.on(member_dsl::id.eq(request_dsl::sender_id)))
            .select((
                MemberShortRow::as_select(),
                request_dsl::note,
                request_dsl::created_at,
            ))
            .order_by((request_dsl::created_at.desc(), request_dsl::sender_id.asc()))
            .limit(25)
            .load(conn)
            .map(map_detailed_rows)
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_outgoing(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Vec<FriendRequestDetailedRow>, DbError> {
        request_dsl::friend_request
            .filter(request_dsl::sender_id.eq(member_id))
            .inner_join(member_dsl::member.on(member_dsl::id.eq(request_dsl::receiver_id)))
            .select((
                MemberShortRow::as_select(),
                request_dsl::note,
                request_dsl::created_at,
            ))
            .order_by((
                request_dsl::created_at.desc(),
                request_dsl::receiver_id.asc(),
            ))
            .limit(25)
            .load(conn)
            .map(map_detailed_rows)
            .map_err(DbError::from)
    }

    pub(in crate::social) fn update(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
        changeset: FriendRequestChangeset,
    ) -> Result<Option<FriendRequestRow>, DbError> {
        diesel::update(request_dsl::friend_request.find((sender_id, receiver_id)))
            .set(changeset)
            .returning(FriendRequestRow::as_returning())
            .get_result(conn)
            .optional()
            .map_err(DbError::from)
    }

    pub(in crate::social) fn delete(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<bool, DbError> {
        diesel::delete(request_dsl::friend_request.find((sender_id, receiver_id)))
            .execute(conn)
            .map(|deleted| deleted == 1)
            .map_err(DbError::from)
    }
}

fn map_detailed_rows(
    rows: Vec<(MemberShortRow, Option<String>, chrono::NaiveDateTime)>,
) -> Vec<FriendRequestDetailedRow> {
    rows.into_iter()
        .map(|(member, note, created_at)| FriendRequestDetailedRow {
            member,
            note,
            created_at,
        })
        .collect()
}
