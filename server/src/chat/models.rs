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

#[cfg(test)]
mod tests {
    use super::{ChatChangeset, ChatRow, NewChat};
    use crate::chat::error::ChatError;
    use shared::models::chat::chat::{Chat, ChatChangesetInput, NewChatInput};

    #[test]
    fn new_chat_normalizes_name_and_preserves_participants() {
        let (chat, member_ids) = NewChat::new(NewChatInput {
            name: "  Project room  ".into(),
            member_ids: vec![3, 7],
        })
        .unwrap();

        assert_eq!(chat.name, "Project room");
        assert_eq!(member_ids, vec![3, 7]);
    }

    #[test]
    fn new_chat_accepts_name_length_boundaries() {
        for name in ["a".repeat(3), "é".repeat(80)] {
            let (chat, _) = NewChat::new(NewChatInput {
                name: format!("  {name}  "),
                member_ids: Vec::new(),
            })
            .unwrap();
            assert_eq!(chat.name, name);
        }
    }

    #[test]
    fn new_chat_rejects_blank_short_and_oversized_names() {
        for name in ["   ".to_string(), "ab".into(), "a".repeat(81)] {
            assert!(matches!(
                NewChat::new(NewChatInput {
                    name,
                    member_ids: Vec::new(),
                }),
                Err(ChatError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn changeset_requires_a_valid_name() {
        assert!(matches!(
            ChatChangeset::try_from(ChatChangesetInput { name: None }),
            Err(ChatError::InvalidChangeset(_))
        ));
        assert!(matches!(
            ChatChangeset::try_from(ChatChangesetInput {
                name: Some("  ".into())
            }),
            Err(ChatError::InvalidChangeset(_))
        ));
    }

    #[test]
    fn changeset_normalizes_valid_names_and_enforces_boundaries() {
        let changeset = ChatChangeset::try_from(ChatChangesetInput {
            name: Some("  Renamed room  ".into()),
        })
        .unwrap();
        assert_eq!(changeset.name.as_deref(), Some("Renamed room"));

        for name in ["ab".to_string(), "a".repeat(81)] {
            assert!(matches!(
                ChatChangeset::try_from(ChatChangesetInput { name: Some(name) }),
                Err(ChatError::InvalidChangeset(_))
            ));
        }
    }

    #[test]
    fn row_converts_to_shared_chat_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let chat = Chat::from(ChatRow {
            id: 12,
            name: "Project room".into(),
            created_at,
        });

        assert_eq!(chat.id, 12);
        assert_eq!(chat.name, "Project room");
        assert_eq!(chat.created_at, created_at);
    }

    #[test]
    fn row_exposes_its_identifier() {
        let row = ChatRow {
            id: 12,
            name: "Project room".into(),
            created_at: chrono::NaiveDateTime::default(),
        };

        assert_eq!(row.id(), 12);
    }
}
