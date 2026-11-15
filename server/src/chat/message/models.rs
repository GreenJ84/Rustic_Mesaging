use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};

use crate::member::models::MemberShortRow;
use crate::schema::message;
use crate::utils::Validator;
use shared::models::chat::chat::ChatId;
use shared::models::chat::message::{
    Message, MessageChangesetInput, MessageDetailed, MessageId, NewMessageInput,
};
use shared::models::member::{MemberId, MemberShort};

use super::error::MessageError;

#[derive(Insertable)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct NewMessage {
    content: String,
    sender_id: MemberId,
    chat_id: ChatId,
}

impl TryFrom<(ChatId, MemberId, NewMessageInput)> for NewMessage {
    type Error = MessageError;

    fn try_from(
        (chat_id, sender_id, input): (ChatId, MemberId, NewMessageInput),
    ) -> Result<Self, Self::Error> {
        let content =
            Validator::validate_text(&input.content, Some(1), Some(2000)).map_err(|error| {
                MessageError::InvalidInput(format!("Message content {}", error.message()))
            })?;
        Ok(Self {
            content,
            sender_id,
            chat_id,
        })
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct MessageChangeset {
    content: Option<String>,
}

impl TryFrom<MessageChangesetInput> for MessageChangeset {
    type Error = MessageError;

    fn try_from(input: MessageChangesetInput) -> Result<Self, Self::Error> {
        let Some(content) = input.content else {
            return Err(MessageError::InvalidChangeset("No changes provided".into()));
        };
        let content = Validator::validate_text(&content, Some(1), Some(2000)).map_err(|error| {
            MessageError::InvalidChangeset(format!("Message content {}", error.message()))
        })?;
        Ok(Self {
            content: Some(content),
        })
    }
}

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct MessageRow {
    id: MessageId,
    content: String,
    sender_id: MemberId,
    chat_id: ChatId,
    created_at: NaiveDateTime,
}

impl MessageRow {
    pub(in crate::chat) fn id(&self) -> MessageId {
        self.id
    }
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            content: row.content,
            sender_id: row.sender_id,
            chat_id: row.chat_id,
            created_at: row.created_at,
        }
    }
}

pub(in crate::chat) struct MessageDetailedRow {
    pub(super) message: MessageRow,
    pub(super) sender: MemberShortRow,
}

impl From<MessageDetailedRow> for MessageDetailed {
    fn from(row: MessageDetailedRow) -> Self {
        Self {
            id: row.message.id,
            content: row.message.content,
            sender: MemberShort::from(row.sender),
            chat_id: row.message.chat_id,
            created_at: row.message.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageChangeset, MessageRow, NewMessage};
    use crate::chat::message::error::MessageError;
    use shared::models::chat::message::{Message, MessageChangesetInput, NewMessageInput};

    #[test]
    fn new_message_normalizes_content_and_sets_ownership() {
        let message = NewMessage::try_from((
            4,
            9,
            NewMessageInput {
                content: "  hello  ".into(),
            },
        ))
        .unwrap();

        assert_eq!(message.content, "hello");
        assert_eq!(message.chat_id, 4);
        assert_eq!(message.sender_id, 9);
    }

    #[test]
    fn new_message_accepts_content_length_boundaries() {
        for content in ["a".to_string(), "é".repeat(2000)] {
            let message = NewMessage::try_from((
                4,
                9,
                NewMessageInput {
                    content: format!("  {content}  "),
                },
            ))
            .unwrap();
            assert_eq!(message.content, content);
        }
    }

    #[test]
    fn new_message_rejects_blank_and_oversized_content() {
        for content in ["   ".to_string(), "a".repeat(2001)] {
            assert!(matches!(
                NewMessage::try_from((4, 9, NewMessageInput { content })),
                Err(MessageError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn changeset_requires_content() {
        assert!(matches!(
            MessageChangeset::try_from(MessageChangesetInput { content: None }),
            Err(MessageError::InvalidChangeset(_))
        ));
    }

    #[test]
    fn changeset_normalizes_content_and_enforces_boundaries() {
        let changeset = MessageChangeset::try_from(MessageChangesetInput {
            content: Some("  x  ".into()),
        })
        .unwrap();
        assert_eq!(changeset.content.as_deref(), Some("x"));

        for content in ["   ".to_string(), "a".repeat(2001)] {
            assert!(matches!(
                MessageChangeset::try_from(MessageChangesetInput {
                    content: Some(content),
                }),
                Err(MessageError::InvalidChangeset(_))
            ));
        }
    }

    #[test]
    fn row_converts_to_shared_message_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let message = Message::from(MessageRow {
            id: 2,
            content: "hello".into(),
            sender_id: 9,
            chat_id: 4,
            created_at,
        });

        assert_eq!(message.id, 2);
        assert_eq!(message.sender_id, 9);
        assert_eq!(message.chat_id, 4);
        assert_eq!(message.created_at, created_at);
    }

    #[test]
    fn row_exposes_its_identifier() {
        let row = MessageRow {
            id: 2,
            content: "hello".into(),
            sender_id: 9,
            chat_id: 4,
            created_at: chrono::NaiveDateTime::default(),
        };

        assert_eq!(row.id(), 2);
    }
}
