use diesel::Connection;

use crate::infra::{Database, DbConn, DbError};
use crate::member::models::{MemberId, MemberShort};

use super::error::ChatError;
use super::participant::models::{
    ChatParticipant, ChatParticipantDetails, MultiChatParticipantDetails, NewChatParticipant, NewChatParticipants,
};
use super::participant::repository::ChatParticipantRepository;
use super::models::{
    ChatChangeset, ChatChangesetInput, ChatDetails, ChatId, MultiChatDetails, NewChat, NewChatInput,
};
use super::repository::ChatRepository;

pub(crate) struct ChatService {
    database: Database,
}

impl ChatService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    // ===============================
    // Chat Operations
    // ===============================
    // Create
    pub(crate) fn create_chat(
        &self,
        member_id: MemberId,
        input: NewChatInput,
    ) -> Result<ChatDetails, ChatError> {
        let (new_chat, mut added_members) = NewChat::new(input)?;
        added_members.push(member_id);

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChatDetails, ChatError, _>(|conn| {
            let chat = ChatRepository::create(conn, new_chat)?;
            let memberships = NewChatParticipants::try_from((chat.id(), added_members))?;

            ChatParticipantRepository::create_many(conn, memberships)?;
            Ok(chat.into())
        })
    }

    // Read
    pub(crate) fn get_chat(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatDetails, ChatError> {
        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, member_id, chat_id)?;
        ChatRepository::read(&mut conn, chat_id)
            .map(ChatDetails::from)
            .map_err(ChatError::from)
    }

    // Update
    pub(crate) fn update_chat(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
        input: ChatChangesetInput,
    ) -> Result<ChatDetails, ChatError> {
        let changeset = ChatChangeset::try_from(input)?;
        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, member_id, chat_id)?;
        ChatRepository::update(&mut conn, chat_id, changeset)
            .map(ChatDetails::from)
            .map_err(ChatError::from)
    }

    // ===============================
    // Chat Membership Operations
    // ===============================
    fn require_participant(
        &self,
        conn: &mut DbConn,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<(), ChatError> {
        if !ChatRepository::exists(conn, chat_id)? {
            return Err(ChatError::NotFound);
        }
        if !ChatParticipantRepository::exists(conn, member_id, chat_id)? {
            return Err(ChatError::NotMember);
        }
        Ok(())
    }

    // Create
    pub(crate) fn join(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatParticipant, ChatError> {
        let memberships = NewChatParticipant::from((chat_id, member_id));

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChatParticipant, ChatError, _>(|conn| {
            if !ChatRepository::exists(conn, chat_id)? {
                return Err(ChatError::NotFound);
            }
            ChatParticipantRepository::create(conn, memberships).map_err(ChatError::from)
        })
    }

    pub(crate) fn add_members(
        &self,
        member_id: MemberId,
        member_ids: Vec<MemberId>,
        chat_id: ChatId,
    ) -> Result<Vec<ChatParticipant>, ChatError> {
        let memberships = NewChatParticipants::try_from((chat_id, member_ids))?;

        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, member_id, chat_id)?;
        ChatParticipantRepository::create_many(&mut conn, memberships).map_err(ChatError::from)
    }

    // Read
    pub(crate) fn get_chat_members(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<MultiChatParticipantDetails, ChatError> {
        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, member_id, chat_id)?;
        ChatParticipantRepository::read_many(&mut conn, chat_id).map_err(ChatError::from)
    }

    // Delete
    pub(crate) fn leave(&self, member_id: MemberId, chat_id: ChatId) -> Result<(), ChatError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), ChatError, _>(|conn| {
            self.require_participant(conn, member_id, chat_id)?;
            ChatParticipantRepository::delete(conn, chat_id, member_id).map_err(
                |error| match error {
                    DbError::NotFound => ChatError::NotMember,
                    error => ChatError::from(error),
                },
            )?;
            if ChatParticipantRepository::read_member_counts(conn, chat_id)? == 0 {
                ChatRepository::delete(conn, chat_id)?;
            }
            Ok(())
        })
    }
}
