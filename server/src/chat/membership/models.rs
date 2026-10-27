use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::chat::models::ChatId;
use crate::member::models::{MemberId, MemberShort};
use crate::schema::member_chat;

use super::error::ChatMembershipError;

#[derive(Debug, FromForm)]
pub(crate) struct ChatMemberInput {
    pub(crate) member_ids: Vec<MemberId>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = member_chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewChatMember {
    chat_id: ChatId,
    member_id: MemberId,
}
impl From<(ChatId, MemberId)> for NewChatMember {
    fn from((chat_id, member_id): (ChatId, MemberId)) -> Self {
        Self { chat_id, member_id }
    }
}

#[derive(Debug)]
pub(crate) struct NewChatMembers(Vec<NewChatMember>);
impl NewChatMembers {
    pub(crate) fn rows(&self) -> &[NewChatMember] {
        &self.0
    }
}
impl TryFrom<(ChatId, Vec<MemberId>)> for NewChatMembers {
    type Error = ChatMembershipError;

    fn try_from((chat_id, member_ids): (ChatId, Vec<MemberId>)) -> Result<Self, Self::Error> {
        let mut unique_ids = member_ids;
        unique_ids.sort_unstable();
        unique_ids.dedup();
        if unique_ids.is_empty() {
            return Err(ChatMembershipError::InvalidInput(
                "At least one member is required".into(),
            ));
        }

        Ok(Self(
            unique_ids
                .into_iter()
                .map(|member_id| NewChatMember::from((chat_id, member_id)))
                .collect(),
        ))
    }
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = member_chat)]
#[diesel(primary_key(chat_id, member_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ChatMember {
    pub(crate) chat_id: ChatId,
    pub(crate) member_id: MemberId,
    pub(crate) joined_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatMemberDetails {
    pub(crate) chat_id: ChatId,
    pub(crate) member: MemberShort,
    pub(crate) joined_at: NaiveDateTime,
}
impl From<(ChatMember, MemberShort)> for ChatMemberDetails {
    fn from((chat, member): (ChatMember, MemberShort)) -> Self {
        Self {
            chat_id: chat.chat_id,
            member,
            joined_at: chat.joined_at,
        }
    }
}
