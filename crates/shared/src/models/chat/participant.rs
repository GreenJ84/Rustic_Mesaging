use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::chat::chat::ChatId;
use crate::models::member::{MemberId, MemberShort};

/// Input used to add multiple members to an existing chat.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewChatParticipantsInput {
    pub member_ids: Vec<MemberId>,
}

/// Stored chat-participant response without an expanded member projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatParticipant {
    pub chat_id: ChatId,
    pub member_id: MemberId,
    pub joined_at: NaiveDateTime,
}

/// Detailed chat-participant response containing a member preview.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatParticipantDetailed {
    pub chat_id: ChatId,
    pub member: MemberShort,
    pub joined_at: NaiveDateTime,
}

/// Collection wrapper for chat-participant responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatParticipantList<T> {
    pub participants: Vec<T>,
}

pub type MultiChatParticipant = ChatParticipantList<ChatParticipant>;
pub type MultiChatParticipantDetailed = ChatParticipantList<ChatParticipantDetailed>;

#[cfg(test)]
mod tests {
    use super::NewChatParticipantsInput;

    #[test]
    fn batch_creation_input_uses_member_ids() {
        let value = serde_json::to_value(NewChatParticipantsInput {
            member_ids: vec![4, 9],
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "member_ids": [4, 9] }));
    }
}
