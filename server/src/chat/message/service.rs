use crate::chat::participant::repository::ChatParticipantRepository;
use crate::chat::repository::ChatRepository;
use crate::infra::{Database, DbConn};
use shared::models::chat::chat::ChatId;
use shared::models::chat::message::{
    MessageChangesetInput, MessageDetailed, MessageId, MultiMessageDetailed, NewMessageInput,
};
use shared::models::member::MemberId;

use super::error::MessageError;
use super::models::{MessageChangeset, NewMessage};
use super::repository::MessageRepository;

pub(crate) struct MessageService {
    database: Database,
}
impl MessageService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    fn authorize(
        &self,
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<(), MessageError> {
        if !ChatRepository::exists(conn, chat_id)? {
            return Err(MessageError::ChatNotFound);
        }
        if !ChatParticipantRepository::exists(conn, member_id, chat_id)? {
            return Err(MessageError::MembershipRequired);
        }
        Ok(())
    }

    // Create
    pub(crate) fn create(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        input: NewMessageInput,
    ) -> Result<MessageDetailed, MessageError> {
        let new_message = NewMessage::try_from((chat_id, member_id, input))?;
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        let message = MessageRepository::create(&mut conn, new_message)?;
        MessageRepository::read_detailed(&mut conn, chat_id, message.id())
            .map(MessageDetailed::from)
            .map_err(MessageError::from)
    }

    // Read
    pub(crate) fn get(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> Result<MessageDetailed, MessageError> {
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        MessageRepository::read_detailed(&mut conn, chat_id, message_id)
            .map(MessageDetailed::from)
            .map_err(MessageError::from)
    }
    pub(crate) fn get_thread(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        page: u32,
    ) -> Result<MultiMessageDetailed, MessageError> {
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        let messages = MessageRepository::read_many_detailed(&mut conn, chat_id, page)?
            .into_iter()
            .map(MessageDetailed::from)
            .collect();
        Ok(MultiMessageDetailed { messages })
    }

    // Update
    pub(crate) fn update(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        message_id: MessageId,
        input: MessageChangesetInput,
    ) -> Result<MessageDetailed, MessageError> {
        let changeset = MessageChangeset::try_from(input)?;
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        if MessageRepository::read_author_id(&mut conn, chat_id, message_id)? != member_id {
            return Err(MessageError::PermissionDenied);
        }
        MessageRepository::update(&mut conn, chat_id, message_id, changeset)?;
        MessageRepository::read_detailed(&mut conn, chat_id, message_id)
            .map(MessageDetailed::from)
            .map_err(MessageError::from)
    }

    // Delete
    pub(crate) fn delete(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> Result<(), MessageError> {
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        if MessageRepository::read_author_id(&mut conn, chat_id, message_id)? != member_id {
            return Err(MessageError::PermissionDenied);
        }
        MessageRepository::delete(&mut conn, chat_id, message_id).map_err(MessageError::from)
    }
}
