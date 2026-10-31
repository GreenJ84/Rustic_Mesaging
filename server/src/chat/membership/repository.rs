use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::chat::membership::models::ChatMemberDetails;
use crate::chat::models::ChatId;
use crate::infra::{DbConn, DbError};
use crate::member::models::{Member, MemberId, MemberShort};
use crate::schema::{member::dsl as m_dsl, member_chat::dsl as mc_dsl};

use super::models::{ChatMember, NewChatMember, NewChatMembers};

pub(crate) struct ChatMembershipRepository;

impl ChatMembershipRepository {
    // Existence
    pub(crate) fn exists(
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(
            mc_dsl::member_chat.find((chat_id, member_id)),
        ))
        .get_result(conn)
        .map_err(DbError::from)
    }

    // Create
    pub(crate) fn create(
        conn: &mut DbConn,
        new_membership: NewChatMember,
    ) -> Result<ChatMember, DbError> {
        diesel::insert_into(mc_dsl::member_chat)
            .values(new_membership)
            .returning(ChatMember::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    pub(crate) fn create_many(
        conn: &mut DbConn,
        new_memberships: NewChatMembers,
    ) -> Result<Vec<ChatMember>, DbError> {
        diesel::insert_into(mc_dsl::member_chat)
            .values(new_memberships.rows())
            .returning(ChatMember::as_returning())
            .get_results(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(crate) fn read(
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatMemberDetails, DbError> {
        mc_dsl::member_chat
            .find((chat_id, member_id))
            .inner_join(m_dsl::member.on(mc_dsl::member_id.eq(m_dsl::id)))
            .select((ChatMember::as_select(), Member::as_select()))
            .first(conn)
            .map(|(chat_member, member)| {
                let member_profile = MemberShort::from(member);
                ChatMemberDetails::from((chat_member, member_profile))
            })
            .map_err(DbError::from)
    }

    pub(crate) fn read_many(
        conn: &mut DbConn,
        chat_id: ChatId,
    ) -> Result<Vec<ChatMemberDetails>, DbError> {
        mc_dsl::member_chat
            .filter(mc_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(mc_dsl::member_id.eq(m_dsl::id)))
            .select((ChatMember::as_select(), Member::as_select()))
            .load(conn)
            .map(|memberships| {
                memberships
                    .into_iter()
                    .map(|(chat_member, member)| {
                        let member_profile = MemberShort::from(member);
                        ChatMemberDetails::from((chat_member, member_profile))
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    pub(crate) fn read_member_counts(conn: &mut DbConn, chat_id: ChatId) -> Result<i64, DbError> {
        mc_dsl::member_chat
            .filter(mc_dsl::chat_id.eq(chat_id))
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
        let deleted = diesel::delete(mc_dsl::member_chat.find((chat_id, member_id)))
            .execute(conn)
            .map_err(DbError::from)?;
        if deleted == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
