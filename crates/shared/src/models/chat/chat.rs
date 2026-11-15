use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::member::MemberId;

pub type ChatId = i32;

/// Input used to create a chat and its initial participant set.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewChatInput {
    pub name: String,
    pub member_ids: Vec<MemberId>,
}

/// Input used to update mutable chat fields.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatChangesetInput {
    pub name: Option<String>,
}

/// Chat response without expanded participant or message data.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Chat {
    pub id: ChatId,
    pub name: String,
    pub created_at: NaiveDateTime,
}

/// Collection wrapper for chat responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatList<T> {
    pub chats: Vec<T>,
}

pub type MultiChat = ChatList<Chat>;

#[cfg(test)]
mod tests {
    use super::{ChatChangesetInput, NewChatInput};

    #[test]
    fn creation_input_uses_the_stable_json_shape() {
        let value = serde_json::to_value(NewChatInput {
            name: "Project room".into(),
            member_ids: vec![4, 9],
        })
        .unwrap();

        assert_eq!(value["name"], "Project room");
        assert_eq!(value["member_ids"], serde_json::json!([4, 9]));
    }

    #[test]
    fn changeset_input_preserves_an_explicit_missing_name() {
        let value = serde_json::to_value(ChatChangesetInput { name: None }).unwrap();

        assert!(value["name"].is_null());
    }
}
