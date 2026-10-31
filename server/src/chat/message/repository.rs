use super::models::{Message, MessageChangeset, MessageDetails, MessageId, NewMessage};
use crate::chat::message::models::MultiMessageDetails;
use crate::chat::models::ChatId;
use crate::infra::{DbConn, DbError};
use crate::member::models::{Member, MemberShort};
use crate::schema::{member::dsl as m_dsl, message::dsl as ms_dsl};
use crate::utils::Paginator;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

const MESSAGE_PAGINATOR: Paginator = Paginator::new(None);

pub(crate) struct MessageRepository;
impl MessageRepository {
    // Create
    pub(crate) fn create(conn: &mut DbConn, value: NewMessage) -> Result<Message, DbError> {
        diesel::insert_into(ms_dsl::message)
            .values(value)
            .returning(Message::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(crate) fn read(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
    ) -> Result<MessageDetails, DbError> {
        ms_dsl::message
            .filter(ms_dsl::id.eq(id))
            .filter(ms_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(ms_dsl::sender_id.eq(m_dsl::id)))
            .select((Message::as_select(), Member::as_select()))
            .first(conn)
            .map(|(message, member)| {
                let member_profile = MemberShort::from(member);
                MessageDetails::from((message, member_profile))
            })
            .map_err(DbError::from)
    }
    pub(crate) fn read_author_id(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
    ) -> Result<i32, DbError> {
        ms_dsl::message
            .filter(ms_dsl::id.eq(id))
            .filter(ms_dsl::chat_id.eq(chat_id))
            .select(ms_dsl::sender_id)
            .first(conn)
            .map_err(DbError::from)
    }
    pub(crate) fn read_many(
        conn: &mut DbConn,
        chat_id: ChatId,
        page: u32,
    ) -> Result<MultiMessageDetails, DbError> {
        let page_size = MESSAGE_PAGINATOR.page_size();
        let page_offset = MESSAGE_PAGINATOR.offset(page);
        ms_dsl::message
            .filter(ms_dsl::chat_id.eq(chat_id))
            .order((ms_dsl::created_at.desc(), ms_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .inner_join(m_dsl::member.on(ms_dsl::sender_id.eq(m_dsl::id)))
            .select((Message::as_select(), Member::as_select()))
            .load(conn)
            .map(|messages| {
                messages
                    .into_iter()
                    .map(|(message, member)| {
                        let member_profile = MemberShort::from(member);
                        MessageDetails::from((message, member_profile))
                    })
                    .collect()
            })
            .map_err(DbError::from)
    }

    // Update
    pub(crate) fn update(
        conn: &mut DbConn,
        chat_id: ChatId,
        id: MessageId,
        changeset: MessageChangeset,
    ) -> Result<MessageDetails, DbError> {
        diesel::update(
            ms_dsl::message
                .filter(ms_dsl::id.eq(id))
                .filter(ms_dsl::chat_id.eq(chat_id)),
        )
        .set(changeset)
        .returning(Message::as_returning())
        .get_result::<Message>(conn)
        .map_err(DbError::from)
        .and_then(|message| Self::read(conn, chat_id, message.id()))
    }

    // Delete
    pub(crate) fn delete(conn: &mut DbConn, chat_id: ChatId, id: MessageId) -> Result<(), DbError> {
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
