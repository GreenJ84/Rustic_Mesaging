use chrono::NaiveDateTime;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};

use super::models::{FriendRequest, NewFriendRequest};
use crate::infra::{DbConn, DbError};
use crate::member::models::{MemberId, MemberShort};
use crate::schema::{friend_request::dsl as f_dsl, member::dsl as m_dsl};
use crate::social::request::models::{IncomingRequests, OutgoingRequests, RequestDetails};

/// Request Direction relative to the passed sender_id and receiver_id.
pub(in crate::social) enum RequestType {
    Incoming,
    Outgoing,
}
pub(in crate::social) struct RequestRepository;
impl RequestRepository {
    // Create
    pub(in crate::social) fn create_request(
        conn: &mut DbConn,
        new_request: NewFriendRequest,
    ) -> Result<FriendRequest, DbError> {
        diesel::insert_into(f_dsl::friend_request)
            .values(&new_request)
            .get_result::<FriendRequest>(conn)
            .map_err(DbError::from)
    }

    // Get
    pub(in crate::social) fn is_requested(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<RequestType, DbError> {
        let request = f_dsl::friend_request
            .filter(
                f_dsl::sender_id
                    .eq(sender_id)
                    .and(f_dsl::receiver_id.eq(receiver_id))
                    .or(f_dsl::sender_id
                        .eq(receiver_id)
                        .and(f_dsl::receiver_id.eq(sender_id))),
            )
            .select(f_dsl::sender_id)
            .first::<MemberId>(conn)
            .map_err(DbError::from)?;

        if sender_id == request {
            Ok(RequestType::Outgoing)
        } else {
            Ok(RequestType::Incoming)
        }
    }

    pub(in crate::social) fn get_request(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<RequestDetails, DbError> {
        f_dsl::friend_request
            .find((sender_id, receiver_id))
            .inner_join(m_dsl::member.on(m_dsl::id.eq(f_dsl::sender_id)))
            .select((MemberShort::as_select(), f_dsl::note, f_dsl::created_at))
            .first::<(MemberShort, Option<String>, NaiveDateTime)>(conn)
            .map(|(member, note, created_at)| RequestDetails {
                member,
                note,
                created_at,
            })
            .map_err(DbError::from)
    }
    pub(in crate::social) fn get_incoming(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<IncomingRequests, DbError> {
        f_dsl::friend_request
            .filter(f_dsl::receiver_id.eq(member_id))
            .inner_join(m_dsl::member.on(m_dsl::id.eq(f_dsl::sender_id)))
            .select((MemberShort::as_select(), f_dsl::note, f_dsl::created_at))
            .limit(25)
            .order_by(f_dsl::created_at.desc())
            .load::<(MemberShort, Option<String>, NaiveDateTime)>(conn)
            .map(|requests| {
                requests
                    .into_iter()
                    .map(|(member, note, created_at)| RequestDetails {
                        member,
                        note,
                        created_at,
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(in crate::social) fn get_outgoing(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<OutgoingRequests, DbError> {
        f_dsl::friend_request
            .filter(f_dsl::sender_id.eq(member_id))
            .inner_join(m_dsl::member.on(m_dsl::id.eq(f_dsl::receiver_id)))
            .select((MemberShort::as_select(), f_dsl::note, f_dsl::created_at))
            .limit(25)
            .order_by(f_dsl::created_at.desc())
            .load::<(MemberShort, Option<String>, NaiveDateTime)>(conn)
            .map(|requests| {
                requests
                    .into_iter()
                    .map(|(member, note, created_at)| RequestDetails {
                        member,
                        note,
                        created_at,
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    // Update
    pub(in crate::social) fn update_request(
        conn: &mut DbConn,
        update: NewFriendRequest,
    ) -> Result<FriendRequest, DbError> {
        diesel::update(f_dsl::friend_request.find((update.sender_id, update.receiver_id)))
            .set(f_dsl::note.eq(update.note))
            .get_result::<FriendRequest>(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(in crate::social) fn delete_request(
        conn: &mut DbConn,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<usize, DbError> {
        diesel::delete(f_dsl::friend_request.find((sender_id, receiver_id)))
            .execute(conn)
            .map_err(DbError::from)
    }
}
