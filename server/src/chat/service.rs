use diesel::Connection;

use crate::database::{Database, DbConn, DbError};
use crate::member::models::{MemberId, MemberShort};

use super::error::ChatError;
use super::membership::models::{
    ChatMember, ChatMemberDetails, MultiChatMemberDetails, NewChatMember, NewChatMembers,
};
use super::membership::repository::ChatMembershipRepository;
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
        let (new_chat, added_members) = NewChat::new(input)?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChatDetails, ChatError, _>(|conn| {
            let chat = ChatRepository::create(conn, new_chat)?;
            let memberships = NewChatMembers::try_from((chat.id(), added_members))?;

            ChatMembershipRepository::create_many(conn, memberships)?;
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
        self.require_participant(&mut conn, chat_id, member_id)?;
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
        self.require_participant(&mut conn, chat_id, member_id)?;
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
        if !ChatMembershipRepository::exists(conn, member_id, chat_id)? {
            return Err(ChatError::from(ChatError::NotMember));
        }
        Ok(())
    }

    // Create
    pub(crate) fn join(
        &self,
        member_ids: MemberId,
        chat_id: ChatId,
    ) -> Result<ChatMember, ChatError> {
        let memberships = NewChatMember::from((chat_id, member_ids));

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChatMember, ChatError, _>(|conn| {
            if !ChatRepository::exists(conn, chat_id)? {
                return Err(ChatError::NotFound);
            }
            ChatMembershipRepository::create(conn, memberships).map_err(ChatError::from)
        })
    }

    pub(crate) fn add_members(
        &self,
        member_ids: Vec<MemberId>,
        chat_id: ChatId,
    ) -> Result<Vec<ChatMember>, ChatError> {
        let memberships = NewChatMembers::try_from((chat_id, member_ids))?;

        let mut conn = self.database.get_connection()?;
        ChatMembershipRepository::create_many(&mut conn, memberships).map_err(ChatError::from)
    }

    // Read
    pub(crate) fn get_chat_members(
        &self,
        member_id: MemberId,
        chat_id: ChatId,
    ) -> Result<MultiChatMemberDetails, ChatError> {
        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, chat_id, member_id)?;
        ChatMembershipRepository::read_many(&mut conn, chat_id).map_err(ChatError::from)
    }

    // Delete
    pub(crate) fn leave(&self, member_id: MemberId, chat_id: ChatId) -> Result<(), ChatError> {
        let mut conn = self.database.get_connection()?;
        self.require_participant(&mut conn, chat_id, member_id)?;
        ChatMembershipRepository::delete(&mut conn, chat_id, member_id).map_err(
            |error| match error {
                DbError::NotFound => ChatError::NotMember,
                error => ChatError::from(error),
            },
        )?;
        if ChatMembershipRepository::read_member_counts(&mut conn, chat_id)? == 0 {
            ChatRepository::delete(&mut conn, chat_id)?;
        }
        Ok(())
    }
}
