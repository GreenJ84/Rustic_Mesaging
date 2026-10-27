use crate::database::{Database, DbConn};
use crate::member::models::MemberId;

use crate::chat::membership::repository::ChatMembershipRepository;
use crate::chat::models::ChatId;
use crate::chat::repository::ChatRepository;

use super::error::MessageError;
use super::models::{
    MessageChangeset, MessageChangesetInput, MessageDetails, MessageId, MessageInput,
    MultiMessageDetails, NewMessage,
};
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
        if !ChatMembershipRepository::exists(conn, member_id, chat_id)? {
            return Err(MessageError::MembershipRequired);
        }
        Ok(())
    }

    // Create
    pub(crate) fn create(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        input: MessageInput,
    ) -> Result<MessageDetails, MessageError> {
        let new_message = NewMessage::try_from((chat_id, member_id, input))?;
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        let message = MessageRepository::create(&mut conn, new_message)?;
        MessageRepository::read(&mut conn, chat_id, message.id()).map_err(MessageError::from)
    }

    // Read
    pub(crate) fn get(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> Result<MessageDetails, MessageError> {
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        MessageRepository::read(&mut conn, chat_id, message_id).map_err(MessageError::from)
    }
    pub(crate) fn get_thread(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        page: u32,
    ) -> Result<MultiMessageDetails, MessageError> {
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        MessageRepository::read_many(&mut conn, chat_id, page).map_err(MessageError::from)
    }

    // Update
    pub(crate) fn update(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        message_id: MessageId,
        input: MessageChangesetInput,
    ) -> Result<MessageDetails, MessageError> {
        let changeset = MessageChangeset::try_from(input)?;
        let mut conn = self.database.get_connection()?;
        self.authorize(&mut conn, member_id, chat_id)?;
        if MessageRepository::read_author_id(&mut conn, chat_id, message_id)? != member_id {
            return Err(MessageError::PermissionDenied);
        }
        MessageRepository::update(&mut conn, chat_id, message_id, changeset)
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
