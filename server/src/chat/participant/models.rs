use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::chat::models::ChatId;
use crate::member::models::{MemberId, MemberShort};
use crate::schema::chat_participant;

use crate::chat::error::ChatError;

#[derive(Insertable, Debug)]
#[diesel(table_name = chat_participant)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewChatParticipant {
    chat_id: ChatId,
    member_id: MemberId,
}
impl From<(ChatId, MemberId)> for NewChatParticipant {
    fn from((chat_id, member_id): (ChatId, MemberId)) -> Self {
        Self { chat_id, member_id }
    }
}

#[derive(Debug)]
pub(crate) struct NewChatParticipants(Vec<NewChatParticipant>);
impl NewChatParticipants {
    pub(crate) fn rows(&self) -> &[NewChatParticipant] {
        &self.0
    }
}
impl TryFrom<(ChatId, Vec<MemberId>)> for NewChatParticipants {
    type Error = ChatError;

    fn try_from((chat_id, member_ids): (ChatId, Vec<MemberId>)) -> Result<Self, Self::Error> {
        let mut unique_ids = member_ids;
        unique_ids.sort_unstable();
        unique_ids.dedup();
        if unique_ids.is_empty() {
            return Err(ChatError::InvalidInput(
                "At least one member is required".into(),
            ));
        }

        Ok(Self(
            unique_ids
                .into_iter()
                .map(|member_id| NewChatParticipant::from((chat_id, member_id)))
                .collect(),
        ))
    }
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = chat_participant)]
#[diesel(primary_key(chat_id, member_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ChatParticipant {
    pub(crate) chat_id: ChatId,
    pub(crate) member_id: MemberId,
    pub(crate) joined_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatParticipantDetails {
    pub(crate) chat_id: ChatId,
    pub(crate) member: MemberShort,
    pub(crate) joined_at: NaiveDateTime,
}
impl From<(ChatParticipant, MemberShort)> for ChatParticipantDetails {
    fn from((chat, member): (ChatParticipant, MemberShort)) -> Self {
        Self {
            chat_id: chat.chat_id,
            member,
            joined_at: chat.joined_at,
        }
    }
}

pub type MultiChatParticipantDetails = Vec<ChatParticipantDetails>;
