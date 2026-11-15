use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::infra::{DbConn, DbError};
use crate::schema::{chat::dsl as c_dsl, chat_participant::dsl as cp_dsl};
use shared::models::chat::chat::ChatId;
use shared::models::member::MemberId;

use super::models::{ChatChangeset, ChatRow, NewChat};

pub(in crate::chat) struct ChatRepository;
impl ChatRepository {
    // Existence
    pub(in crate::chat) fn exists(conn: &mut DbConn, chat_id: ChatId) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(c_dsl::chat.find(chat_id)))
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Create
    pub(in crate::chat) fn create(
        conn: &mut DbConn,
        new_chat: NewChat,
    ) -> Result<ChatRow, DbError> {
        diesel::insert_into(c_dsl::chat)
            .values(new_chat)
            .returning(ChatRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(in crate::chat) fn read(conn: &mut DbConn, chat_id: ChatId) -> Result<ChatRow, DbError> {
        c_dsl::chat
            .find(chat_id)
            .select(ChatRow::as_select())
            .first(conn)
            .map_err(DbError::from)
    }

    pub(in crate::chat) fn read_many_by_member(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Vec<ChatRow>, DbError> {
        c_dsl::chat
            .inner_join(cp_dsl::chat_participant.on(cp_dsl::chat_id.eq(c_dsl::id)))
            .filter(cp_dsl::member_id.eq(member_id))
            .order((c_dsl::created_at.desc(), c_dsl::id.desc()))
            .select(ChatRow::as_select())
            .load(conn)
            .map_err(DbError::from)
    }

    // Update
    pub(in crate::chat) fn update(
        conn: &mut DbConn,
        chat_id: ChatId,
        changeset: ChatChangeset,
    ) -> Result<ChatRow, DbError> {
        diesel::update(c_dsl::chat.find(chat_id))
            .set(changeset)
            .returning(ChatRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(in crate::chat) fn delete(conn: &mut DbConn, chat_id: ChatId) -> Result<(), DbError> {
        let deleted = diesel::delete(c_dsl::chat.find(chat_id))
            .execute(conn)
            .map_err(DbError::from)?;
        if deleted == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
