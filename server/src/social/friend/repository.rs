use diesel::dsl::case_when;
use diesel::prelude::*;

use super::models::{
    normalize_friendship, FriendDetailedRow, FriendRow, FriendShortRow, NewFriend,
};
use crate::infra::{DbConn, DbError};
use crate::member::models::{MemberDetailedRow, MemberId, MemberShortRow};
use crate::schema::friend::dsl as friend_dsl;
use crate::schema::member::dsl as member_dsl;

pub(crate) struct FriendRepository;

impl FriendRepository {
    pub(in crate::social) fn create(
        conn: &mut DbConn,
        new_friend: NewFriend,
    ) -> Result<FriendRow, DbError> {
        diesel::insert_into(friend_dsl::friend)
            .values(&new_friend)
            .returning(FriendRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(crate) fn exists(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<bool, DbError> {
        let pair = normalize_friendship(member_id, friend_id);
        diesel::select(diesel::dsl::exists(friend_dsl::friend.find(pair)))
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<Option<FriendRow>, DbError> {
        let pair = normalize_friendship(member_id, friend_id);
        friend_dsl::friend
            .find(pair)
            .select(FriendRow::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_detailed(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<Option<FriendDetailedRow>, DbError> {
        let requester_id = member_id;
        let pair = normalize_friendship(member_id, friend_id);
        let other_member_id = case_when(
            friend_dsl::member_id.eq(requester_id),
            friend_dsl::friend_id,
        )
        .otherwise(friend_dsl::member_id);

        friend_dsl::friend
            .find(pair)
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberDetailedRow::as_select(), friend_dsl::created_at))
            .first(conn)
            .optional()
            .map(|row| row.map(|(friend, created_at)| FriendDetailedRow { friend, created_at }))
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_many_detailed(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Vec<FriendDetailedRow>, DbError> {
        let other_member_id = case_when(friend_dsl::member_id.eq(member_id), friend_dsl::friend_id)
            .otherwise(friend_dsl::member_id);

        friend_dsl::friend
            .filter(
                friend_dsl::member_id
                    .eq(member_id)
                    .or(friend_dsl::friend_id.eq(member_id)),
            )
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberDetailedRow::as_select(), friend_dsl::created_at))
            .order_by((
                friend_dsl::created_at.desc(),
                friend_dsl::member_id.asc(),
                friend_dsl::friend_id.asc(),
            ))
            .limit(25)
            .load(conn)
            .map(|rows: Vec<(MemberDetailedRow, _)>| {
                rows.into_iter()
                    .map(|(friend, created_at)| FriendDetailedRow { friend, created_at })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn read_many_short(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Vec<FriendShortRow>, DbError> {
        let other_member_id = case_when(friend_dsl::member_id.eq(member_id), friend_dsl::friend_id)
            .otherwise(friend_dsl::member_id);

        friend_dsl::friend
            .filter(
                friend_dsl::member_id
                    .eq(member_id)
                    .or(friend_dsl::friend_id.eq(member_id)),
            )
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberShortRow::as_select(), friend_dsl::created_at))
            .order_by((
                friend_dsl::created_at.desc(),
                friend_dsl::member_id.asc(),
                friend_dsl::friend_id.asc(),
            ))
            .limit(25)
            .load(conn)
            .map(|rows: Vec<(MemberShortRow, _)>| {
                rows.into_iter()
                    .map(|(friend, created_at)| FriendShortRow { friend, created_at })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn delete(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<bool, DbError> {
        let pair = normalize_friendship(member_id, friend_id);
        diesel::delete(friend_dsl::friend.find(pair))
            .execute(conn)
            .map(|deleted| deleted == 1)
            .map_err(DbError::from)
    }
}
