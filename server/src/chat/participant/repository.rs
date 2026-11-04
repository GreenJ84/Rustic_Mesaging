use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::chat::models::ChatId;
use crate::chat::participant::models::ChatParticipantDetails;
use crate::infra::{DbConn, DbError};
use crate::member::models::{Member, MemberId, MemberShort};
use crate::schema::{member::dsl as m_dsl, chat_participant::dsl as cp_dsl};

use super::models::{ChatParticipant, NewChatParticipant, NewChatParticipants};

pub(crate) struct ChatParticipantRepository;

impl ChatParticipantRepository {
    // Existence
    pub(crate) fn exists(
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(
            cp_dsl::chat_participant.find((chat_id, member_id)),
        ))
        .get_result(conn)
        .map_err(DbError::from)
    }

    // Create
    pub(crate) fn create(
        conn: &mut DbConn,
        new_membership: NewChatParticipant,
    ) -> Result<ChatParticipant, DbError> {
        diesel::insert_into(cp_dsl::chat_participant)
            .values(new_membership)
            .returning(ChatParticipant::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(crate) fn create_many(
        conn: &mut DbConn,
        new_memberships: NewChatParticipants,
    ) -> Result<Vec<ChatParticipant>, DbError> {
        diesel::insert_into(cp_dsl::chat_participant)
            .values(new_memberships.rows())
            .returning(ChatParticipant::as_returning())
            .get_results(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(crate) fn read(
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatParticipantDetails, DbError> {
        cp_dsl::chat_participant
            .find((chat_id, member_id))
            .inner_join(m_dsl::member.on(cp_dsl::member_id.eq(m_dsl::id)))
            .select((ChatParticipant::as_select(), Member::as_select()))
            .first(conn)
            .map(|(chat_member, member)| {
                let member_profile = MemberShort::from(member);
                ChatParticipantDetails::from((chat_member, member_profile))
            })
            .map_err(DbError::from)
    }

    pub(crate) fn read_many(
        conn: &mut DbConn,
        chat_id: ChatId,
    ) -> Result<Vec<ChatParticipantDetails>, DbError> {
        cp_dsl::chat_participant
            .filter(cp_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(cp_dsl::member_id.eq(m_dsl::id)))
            .select((ChatParticipant::as_select(), Member::as_select()))
            .load(conn)
            .map(|memberships| {
                memberships
                    .into_iter()
                    .map(|(chat_member, member)| {
                        let member_profile = MemberShort::from(member);
                        ChatParticipantDetails::from((chat_member, member_profile))
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(crate) fn read_member_counts(conn: &mut DbConn, chat_id: ChatId) -> Result<i64, DbError> {
        cp_dsl::chat_participant
            .filter(cp_dsl::chat_id.eq(chat_id))
            .count()
            .get_result::<i64>(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(crate) fn delete(
        conn: &mut DbConn,
        chat_id: ChatId,
        member_id: MemberId,
    ) -> Result<(), DbError> {
        let deleted = diesel::delete(cp_dsl::chat_participant.find((chat_id, member_id)))
            .execute(conn)
            .map_err(DbError::from)?;
        if deleted == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
