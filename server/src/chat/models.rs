use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};

use crate::schema::chat;
use crate::utils::Validator;
use shared::models::chat::chat::{Chat, ChatChangesetInput, ChatId, NewChatInput};
use shared::models::member::MemberId;

use super::error::ChatError;

#[derive(Insertable)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct NewChat {
    name: String,
}

impl NewChat {
    pub(in crate::chat) fn new(input: NewChatInput) -> Result<(Self, Vec<MemberId>), ChatError> {
        let name = Validator::validate_text(&input.name, Some(3), Some(80))
            .map_err(|error| ChatError::InvalidInput(format!("Chat name {}", error.message())))?;

        Ok((Self { name }, input.member_ids))
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct ChatChangeset {
    name: Option<String>,
}

impl TryFrom<ChatChangesetInput> for ChatChangeset {
    type Error = ChatError;

    fn try_from(input: ChatChangesetInput) -> Result<Self, Self::Error> {
        let Some(name) = input.name else {
            return Err(ChatError::InvalidChangeset("No changes provided".into()));
        };
        let name = Validator::validate_text(&name, Some(3), Some(80)).map_err(|error| {
            ChatError::InvalidChangeset(format!("Chat name {}", error.message()))
        })?;
        Ok(Self { name: Some(name) })
    }
}

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct ChatRow {
    id: ChatId,
    name: String,
    created_at: NaiveDateTime,
}

impl ChatRow {
    pub(in crate::chat) fn id(&self) -> ChatId {
        self.id
    }
}

impl From<ChatRow> for Chat {
    fn from(row: ChatRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
        }
    }
}

pub(crate) type MultiChatDetails = Vec<ChatDetails>;
pub(crate) type MultiChatPage = Page<MultiChatDetails>;
