use chrono::NaiveDateTime;
use diesel::dsl::case_when;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};

use super::models::{
    normalize_friendship, Friend, FriendPreview, Friendship, MultiFriend, MultiFriendPreview,
    NewFriend,
};
use crate::infra::{DbConn, DbError};
use crate::member::models::{MemberId, MemberProfile, MemberShort};
use crate::schema::friend::dsl as friend_dsl;
use crate::schema::member::dsl as member_dsl;

pub(in crate::social) struct FriendRepository;
impl FriendRepository {
    // Create
    pub(in crate::social) fn create(
        conn: &mut DbConn,
        new_friendship: NewFriend,
    ) -> Result<Friendship, DbError> {
        diesel::insert_into(friend_dsl::friend)
            .values(&new_friendship)
            .returning(Friendship::as_returning())
            .get_result::<Friendship>(conn)
            .map_err(DbError::from)
    }
    // Get
    pub(in crate::social) fn exists(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<bool, DbError> {
        let (member_id, friend_id) = normalize_friendship(member_id, friend_id);
        diesel::select(diesel::dsl::exists(
            friend_dsl::friend.find((member_id, friend_id)),
        ))
        .get_result::<bool>(conn)
        .map_err(DbError::from)
    }

    pub(in crate::social) fn find(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<Friend, DbError> {
        let other_member_id = case_when(friend_dsl::member_id.eq(member_id), friend_dsl::friend_id)
            .otherwise(friend_dsl::member_id);

        let (member_id, friend_id) = normalize_friendship(member_id, friend_id);
        friend_dsl::friend
            .filter(friend_dsl::member_id.eq(member_id))
            .filter(friend_dsl::friend_id.eq(friend_id))
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberProfile::as_select(), friend_dsl::created_at))
            .first::<(MemberProfile, NaiveDateTime)>(conn)
            .map(|(member, created_at)| Friend {
                friend: member,
                created_at,
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn find_multi(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<MultiFriend, DbError> {
        let other_member_id = case_when(friend_dsl::member_id.eq(member_id), friend_dsl::friend_id)
            .otherwise(friend_dsl::member_id);

        friend_dsl::friend
            .filter(
                friend_dsl::member_id
                    .eq(member_id)
                    .or(friend_dsl::friend_id.eq(member_id)), // Find all friend relationships
            )
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberProfile::as_select(), friend_dsl::created_at))
            .order_by(friend_dsl::created_at.desc())
            .limit(25)
            .load::<(MemberProfile, NaiveDateTime)>(conn)
            .map(|results| MultiFriend {
                friends: results
                    .into_iter()
                    .map(|(member, created_at)| Friend {
                        friend: member,
                        created_at,
                    })
                    .collect::<Vec<Friend>>(),
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn find_multi_preview(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<MultiFriendPreview, DbError> {
        let other_member_id = case_when(friend_dsl::member_id.eq(member_id), friend_dsl::friend_id)
            .otherwise(friend_dsl::member_id);

        friend_dsl::friend
            .filter(
                friend_dsl::member_id
                    .eq(member_id)
                    .or(friend_dsl::friend_id.eq(member_id)), // Find all friend relationships
            )
            .inner_join(member_dsl::member.on(member_dsl::id.eq(other_member_id)))
            .select((MemberShort::as_select(), friend_dsl::created_at))
            .order_by(friend_dsl::created_at.desc())
            .limit(25)
            .load::<(MemberShort, NaiveDateTime)>(conn)
            .map(|results| MultiFriendPreview {
                friends: results
                    .into_iter()
                    .map(|(member, created_at)| FriendPreview {
                        friend: member,
                        created_at,
                    })
                    .collect::<Vec<FriendPreview>>(),
            })
            .map_err(DbError::from)
    }

    // Friendship Updates are not supported.

    // Delete
    pub(in crate::social) fn delete(
        conn: &mut DbConn,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<usize, DbError> {
        let (member_id, friend_id) = normalize_friendship(member_id, friend_id);
        diesel::delete(friend_dsl::friend.find((member_id, friend_id)))
            .execute(conn)
            .map_err(DbError::from)
    }
}
