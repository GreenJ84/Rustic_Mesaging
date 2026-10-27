use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::chat::models::{Chat, ChatId};
use crate::member::models::{Member, MemberId, MemberShort};
use crate::schema::message;
use crate::utils::{Page, Validator};

use super::error::MessageError;

pub(crate) type MessageId = i32;

#[derive(Debug, FromForm)]
pub(crate) struct MessageInput {
    pub(crate) content: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewMessage {
    content: String,
    sender_id: MemberId,
    chat_id: ChatId,
}
impl TryFrom<(ChatId, MemberId, MessageInput)> for NewMessage {
    type Error = MessageError;

    fn try_from(
        (chat_id, sender_id, input): (ChatId, MemberId, MessageInput),
    ) -> Result<Self, Self::Error> {
        let content = Validator::validate_text(input.content, None, Some(2000))
            .map_err(|e| MessageError::InvalidInput(format!("Message content {}", e.message())))?;
        Ok(Self {
            content,
            sender_id,
            chat_id,
        })
    }
}

#[derive(Debug, FromForm)]
pub(crate) struct MessageChangesetInput {
    pub(crate) content: Option<String>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct MessageChangeset {
    content: Option<String>,
}
impl TryFrom<MessageChangesetInput> for MessageChangeset {
    type Error = MessageError;

    fn try_from(input: MessageChangesetInput) -> Result<Self, Self::Error> {
        let Some(content) = input.content else {
            return Err(MessageError::InvalidChangeset("No changes provided".into()));
        };
        let content = Validator::validate_text(content, None, Some(2000)).map_err(|e| {
            MessageError::InvalidChangeset(format!("Message content {}", e.message()))
        })?;
        Ok(Self {
            content: Some(content),
        })
    }
}

#[derive(Identifiable, Queryable, Selectable, Debug)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Member, foreign_key = sender_id))]
pub(crate) struct Message {
    id: MessageId,
    content: String,
    sender_id: MemberId,
    chat_id: ChatId,
    created_at: NaiveDateTime,
}
impl Message {
    pub(crate) fn id(&self) -> MessageId {
        self.id
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct MessageDetails {
    pub(crate) id: MessageId,
    pub(crate) content: String,
    pub(crate) sender: MemberShort,
    pub(crate) chat_id: ChatId,
    pub(crate) created_at: NaiveDateTime,
}

impl From<(Message, MemberShort)> for MessageDetails {
    fn from((message, sender): (Message, MemberShort)) -> Self {
        Self {
            id: message.id,
            content: message.content,
            sender,
            chat_id: message.chat_id,
            created_at: message.created_at,
        }
    }
}

pub type MultiMessageDetails = Vec<MessageDetails>;
pub type MessagePage = Page<MultiMessageDetails>;
