use super::models::{MessageChangeset, MessageDetailedRow, MessageRow, NewMessage};
use crate::infra::{DbConn, DbError};
use crate::member::models::MemberShortRow;
use crate::schema::{member::dsl as m_dsl, message::dsl as ms_dsl};
use crate::utils::Paginator;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use shared::models::chat::chat::ChatId;
use shared::models::chat::message::MessageId;
use shared::models::member::MemberId;

const MESSAGE_PAGINATOR: Paginator = Paginator::new(None);

pub(in crate::chat) struct MessageRepository;
impl MessageRepository {
    // Create
    pub(in crate::chat) fn create(
        conn: &mut DbConn,
        value: NewMessage,
    ) -> Result<MessageRow, DbError> {
        diesel::insert_into(ms_dsl::message)
            .values(value)
            .returning(MessageRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(in crate::chat) fn read_detailed(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
    ) -> Result<MessageDetailedRow, DbError> {
        ms_dsl::message
            .filter(ms_dsl::id.eq(id))
            .filter(ms_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(ms_dsl::sender_id.eq(m_dsl::id)))
            .select((MessageRow::as_select(), MemberShortRow::as_select()))
            .first(conn)
            .map(|(message, sender)| MessageDetailedRow { message, sender })
            .map_err(DbError::from)
    }
    pub(in crate::chat) fn read_author_id(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
    ) -> Result<MemberId, DbError> {
        ms_dsl::message
            .filter(ms_dsl::id.eq(id))
            .filter(ms_dsl::chat_id.eq(chat_id))
            .select(ms_dsl::sender_id)
            .first(conn)
            .map_err(DbError::from)
    }
    pub(in crate::chat) fn read_many_detailed(
        conn: &mut DbConn,
        chat_id: ChatId,
        page: u32,
    ) -> Result<Vec<MessageDetailedRow>, DbError> {
        let page_size = MESSAGE_PAGINATOR.page_size();
        let page_offset = MESSAGE_PAGINATOR.offset(page);
        ms_dsl::message
            .filter(ms_dsl::chat_id.eq(chat_id))
            .order((ms_dsl::created_at.desc(), ms_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .inner_join(m_dsl::member.on(ms_dsl::sender_id.eq(m_dsl::id)))
            .select((MessageRow::as_select(), MemberShortRow::as_select()))
            .load(conn)
            .map(|messages| {
                messages
                    .into_iter()
                    .map(|(message, sender)| MessageDetailedRow { message, sender })
                    .collect()
            })
            .map_err(DbError::from)
    }

    // Update
    pub(in crate::chat) fn update(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
        changeset: MessageChangeset,
    ) -> Result<MessageRow, DbError> {
        diesel::update(
            ms_dsl::message
                .filter(ms_dsl::id.eq(id))
                .filter(ms_dsl::chat_id.eq(chat_id)),
        )
        .set(changeset)
        .returning(MessageRow::as_returning())
        .get_result::<MessageRow>(conn)
        .map_err(DbError::from)
    }

    // Delete
    pub(in crate::chat) fn delete(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
    ) -> Result<(), DbError> {
        let count = diesel::delete(
            ms_dsl::message
                .filter(ms_dsl::id.eq(id))
                .filter(ms_dsl::chat_id.eq(chat_id)),
        )
        .execute(conn)
        .map_err(DbError::from)?;
        if count == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
