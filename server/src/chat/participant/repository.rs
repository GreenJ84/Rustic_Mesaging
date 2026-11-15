use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::infra::{DbConn, DbError};
use crate::member::models::MemberShortRow;
use crate::schema::{chat_participant::dsl as cp_dsl, member::dsl as m_dsl};
use shared::models::chat::chat::ChatId;
use shared::models::member::MemberId;

use super::models::{
    ChatParticipantDetailedRow, ChatParticipantRow, NewChatParticipant, NewChatParticipants,
};

pub(in crate::chat) struct ChatParticipantRepository;

impl ChatParticipantRepository {
    // Existence
    pub(in crate::chat) fn exists(
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
    pub(in crate::chat) fn create(
        conn: &mut DbConn,
        new_membership: NewChatParticipant,
    ) -> Result<ChatParticipantRow, DbError> {
        diesel::insert_into(cp_dsl::chat_participant)
            .values(new_membership)
            .returning(ChatParticipantRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(in crate::chat) fn create_many(
        conn: &mut DbConn,
        new_memberships: NewChatParticipants,
    ) -> Result<Vec<ChatParticipantRow>, DbError> {
        diesel::insert_into(cp_dsl::chat_participant)
            .values(new_memberships.rows())
            .returning(ChatParticipantRow::as_returning())
            .get_results(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(in crate::chat) fn read_detailed(
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatParticipantDetailedRow, DbError> {
        cp_dsl::chat_participant
            .find((chat_id, member_id))
            .inner_join(m_dsl::member.on(cp_dsl::member_id.eq(m_dsl::id)))
            .select((ChatParticipantRow::as_select(), MemberShortRow::as_select()))
            .first(conn)
            .map(|(participant, member)| ChatParticipantDetailedRow {
                participant,
                member,
            })
            .map_err(DbError::from)
    }

    pub(in crate::chat) fn read_many_detailed(
        conn: &mut DbConn,
        chat_id: ChatId,
    ) -> Result<Vec<ChatParticipantDetailedRow>, DbError> {
        cp_dsl::chat_participant
            .filter(cp_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(cp_dsl::member_id.eq(m_dsl::id)))
            .select((ChatParticipantRow::as_select(), MemberShortRow::as_select()))
            .load(conn)
            .map(|memberships| {
                memberships
                    .into_iter()
                    .map(|(participant, member)| ChatParticipantDetailedRow {
                        participant,
                        member,
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(in crate::chat) fn read_member_counts(
        conn: &mut DbConn,
        chat_id: ChatId,
    ) -> Result<i64, DbError> {
        cp_dsl::chat_participant
            .filter(cp_dsl::chat_id.eq(chat_id))
            .count()
            .get_result::<i64>(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(in crate::chat) fn delete(
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
