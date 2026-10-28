use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

use crate::infra::{DbConn, DbError};
use crate::schema::chat::dsl as c_dsl;

use super::models::{Chat, ChatChangeset, ChatId, NewChat};

pub(crate) struct ChatRepository;
impl ChatRepository {
    // Existence
    pub(crate) fn exists(conn: &mut DbConn, chat_id: ChatId) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(c_dsl::chat.find(chat_id)))
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Create
    pub(crate) fn create(conn: &mut DbConn, new_chat: NewChat) -> Result<Chat, DbError> {
        diesel::insert_into(c_dsl::chat)
            .values(new_chat)
            .returning(Chat::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(crate) fn read(conn: &mut DbConn, chat_id: ChatId) -> Result<Chat, DbError> {
        c_dsl::chat
            .find(chat_id)
            .select(Chat::as_select())
            .first(conn)
            .map_err(DbError::from)
    }

    // Update
    pub(crate) fn update(
        conn: &mut DbConn,
        chat_id: ChatId,
        changeset: ChatChangeset,
    ) -> Result<Chat, DbError> {
        diesel::update(c_dsl::chat.find(chat_id))
            .set(changeset)
            .returning(Chat::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(crate) fn delete(conn: &mut DbConn, chat_id: ChatId) -> Result<(), DbError> {
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
