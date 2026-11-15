use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};

use crate::chat::error::ChatError;
use crate::member::models::MemberShortRow;
use crate::schema::chat_participant;
use shared::models::chat::chat::ChatId;
use shared::models::chat::participant::{ChatParticipant, ChatParticipantDetailed};
use shared::models::member::{MemberId, MemberShort};

#[derive(Insertable)]
#[diesel(table_name = chat_participant)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct NewChatParticipant {
    chat_id: ChatId,
    member_id: MemberId,
}

impl From<(ChatId, MemberId)> for NewChatParticipant {
    fn from((chat_id, member_id): (ChatId, MemberId)) -> Self {
        Self { chat_id, member_id }
    }
}

pub(in crate::chat) struct NewChatParticipants(Vec<NewChatParticipant>);

impl NewChatParticipants {
    pub(in crate::chat) fn rows(&self) -> &[NewChatParticipant] {
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

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = chat_participant)]
#[diesel(primary_key(chat_id, member_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::chat) struct ChatParticipantRow {
    chat_id: ChatId,
    member_id: MemberId,
    joined_at: NaiveDateTime,
}

impl From<ChatParticipantRow> for ChatParticipant {
    fn from(row: ChatParticipantRow) -> Self {
        Self {
            chat_id: row.chat_id,
            member_id: row.member_id,
            joined_at: row.joined_at,
        }
    }
}

pub(in crate::chat) struct ChatParticipantDetailedRow {
    pub(super) participant: ChatParticipantRow,
    pub(super) member: MemberShortRow,
}

impl From<ChatParticipantDetailedRow> for ChatParticipantDetailed {
    fn from(row: ChatParticipantDetailedRow) -> Self {
        Self {
            chat_id: row.participant.chat_id,
            member: MemberShort::from(row.member),
            joined_at: row.participant.joined_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatParticipantRow, NewChatParticipant, NewChatParticipants};
    use shared::models::chat::participant::ChatParticipant;

    #[test]
    fn participant_batch_sorts_and_deduplicates_member_ids() {
        let participants = NewChatParticipants::try_from((3, vec![8, 4, 8])).unwrap();
        let member_ids = participants
            .rows()
            .iter()
            .map(|participant| participant.member_id)
            .collect::<Vec<_>>();

        assert_eq!(member_ids, vec![4, 8]);
        assert!(participants
            .rows()
            .iter()
            .all(|participant| participant.chat_id == 3));
    }

    #[test]
    fn individual_participant_uses_tuple_ownership() {
        let participant = NewChatParticipant::from((3, 8));

        assert_eq!(participant.chat_id, 3);
        assert_eq!(participant.member_id, 8);
    }

    #[test]
    fn participant_batch_rejects_an_empty_member_set() {
        assert!(NewChatParticipants::try_from((3, Vec::new())).is_err());
    }

    #[test]
    fn row_converts_to_shared_participant_contract() {
        let joined_at = chrono::NaiveDateTime::default();
        let participant = ChatParticipant::from(ChatParticipantRow {
            chat_id: 3,
            member_id: 8,
            joined_at,
        });

        assert_eq!(participant.chat_id, 3);
        assert_eq!(participant.member_id, 8);
        assert_eq!(participant.joined_at, joined_at);
    }
}
