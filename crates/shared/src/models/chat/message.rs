use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::chat::chat::ChatId;
use crate::models::member::{MemberId, MemberShort};

pub type MessageId = i32;

/// Input used to create a message in a chat.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewMessageInput {
    pub content: String,
}

/// Input used to update mutable message fields.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageChangesetInput {
    pub content: Option<String>,
}

/// Message response without an expanded sender projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Message {
    pub id: MessageId,
    pub content: String,
    pub sender_id: MemberId,
    pub chat_id: ChatId,
    pub created_at: NaiveDateTime,
}

/// Detailed message response containing a sender preview.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageDetailed {
    pub id: MessageId,
    pub content: String,
    pub sender: MemberShort,
    pub chat_id: ChatId,
    pub created_at: NaiveDateTime,
}

/// Collection wrapper for message responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageList<T> {
    pub messages: Vec<T>,
}

pub type MultiMessage = MessageList<Message>;
pub type MultiMessageDetailed = MessageList<MessageDetailed>;

#[cfg(test)]
mod tests {
    use super::{MessageChangesetInput, MessageList, NewMessageInput};

    #[test]
    fn creation_input_contains_only_client_owned_content() {
        let value = serde_json::to_value(NewMessageInput {
            content: "hello".into(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "content": "hello" }));
    }

    #[test]
    fn message_collections_use_the_messages_field() {
        let value = serde_json::to_value(MessageList::<serde_json::Value> {
            messages: Vec::new(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "messages": [] }));
    }

    #[test]
    fn changeset_preserves_an_explicit_missing_content() {
        let value = serde_json::to_value(MessageChangesetInput { content: None }).unwrap();

        assert!(value["content"].is_null());
    }

    #[test]
    fn creation_input_deserializes_from_the_wire_shape() {
        let input: NewMessageInput =
            serde_json::from_value(serde_json::json!({ "content": "hello" })).unwrap();

        assert_eq!(input.content, "hello");
    }
}
