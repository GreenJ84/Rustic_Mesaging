use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::member::models::{MemberId, MemberShort};
use crate::schema::chat;
use crate::utils::{Page, Validator};

use super::error::ChatError;

pub(crate) type ChatId = i32;

#[derive(FromForm, Debug)]
pub(crate) struct NewChatInput {
    pub(crate) name: String,
    pub(crate) member_ids: Vec<MemberId>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewChat {
    name: String,
}
impl NewChat {
    pub fn new(input: NewChatInput) -> Result<(Self, Vec<MemberId>), ChatError> {
        let name = Validator::validate_text(input.name, Some(3), Some(80))
            .map_err(|e| ChatError::InvalidInput(format!("Chat name {}", e.message())))?;

        Ok((Self { name }, input.member_ids))
    }
}

#[derive(Debug, FromForm)]
pub(crate) struct ChatChangesetInput {
    pub(crate) name: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ChatChangeset {
    name: Option<String>,
}
impl TryFrom<ChatChangesetInput> for ChatChangeset {
    type Error = ChatError;

    fn try_from(input: ChatChangesetInput) -> Result<Self, Self::Error> {
        let Some(name) = input.name else {
            return Err(ChatError::InvalidChangeset("No changes provided".into()));
        };
        let name = Validator::validate_text(name, Some(3), Some(80))
            .map_err(|e| ChatError::InvalidChangeset(format!("Chat name {}", e.message())))?;
        Ok(Self { name: Some(name) })
    }
}

#[derive(Identifiable, Queryable, Selectable, Debug)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Chat {
    id: ChatId,
    name: String,
    created_at: NaiveDateTime,
}
impl Chat {
    pub(crate) fn id(&self) -> ChatId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatDetails {
    pub id: ChatId,
    pub name: String,
    pub created_at: NaiveDateTime,
}
impl From<Chat> for ChatDetails {
    fn from(chat: Chat) -> Self {
        Self {
            id: chat.id,
            name: chat.name,
            created_at: chat.created_at,
        }
    }
}

pub(crate) type MultiChatDetails = Vec<ChatDetails>;
pub(crate) type MultiChatPage = Page<MultiChatDetails>;
