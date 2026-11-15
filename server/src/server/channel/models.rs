use chrono::NaiveDateTime;
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};

use crate::schema::channel;
use crate::server::models::ServerRow;
use crate::utils::{ValidationError, Validator};
use shared::models::server::channel::channel::{
    Channel, ChannelChangesetInput, ChannelDetailed, ChannelId, ChannelShort, NewChannelInput,
};
use shared::models::server::server::ServerId;

use super::error::ChannelError;

fn validate_channel_name(name: String) -> Result<String, ValidationError> {
    Validator::validate_text(&name, Some(5), Some(10))
}

#[derive(Insertable)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct NewChannel {
    server_id: ServerId,
    name: String,
}
impl NewChannel {
    pub(in crate::server) fn name(&self) -> &str {
        &self.name
    }
}
impl TryFrom<(ServerId, NewChannelInput)> for NewChannel {
    type Error = ChannelError;

    fn try_from((server_id, input): (ServerId, NewChannelInput)) -> Result<Self, Self::Error> {
        let name = validate_channel_name(input.name).map_err(|error| {
            ChannelError::InvalidInput(format!("Channel name {}", error.message()))
        })?;
        Ok(Self { server_id, name })
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ChannelChangeset {
    name: Option<String>,
}
impl ChannelChangeset {
    pub(in crate::server) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
impl TryFrom<ChannelChangesetInput> for ChannelChangeset {
    type Error = ChannelError;

    fn try_from(input: ChannelChangesetInput) -> Result<Self, Self::Error> {
        let Some(name) = input.name else {
            return Err(ChannelError::InvalidChangeset("No changes provided".into()));
        };
        let name = validate_channel_name(name).map_err(|error| {
            ChannelError::InvalidChangeset(format!("Channel name {}", error.message()))
        })?;
        Ok(Self { name: Some(name) })
    }
}

#[derive(Associations, Identifiable, Queryable, Selectable)]
#[diesel(table_name = channel)]
#[diesel(belongs_to(ServerRow, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ChannelRow {
    id: ChannelId,
    server_id: ServerId,
    name: String,
    created_at: NaiveDateTime,
}
impl From<ChannelRow> for Channel {
    fn from(row: ChannelRow) -> Self {
        Self {
            id: row.id,
            server_id: row.server_id,
            name: row.name,
            created_at: row.created_at,
        }
    }
}
impl From<ChannelRow> for ChannelDetailed {
    fn from(row: ChannelRow) -> Self {
        Self {
            id: row.id,
            server_id: row.server_id,
            name: row.name,
            created_at: row.created_at,
        }
    }
}
impl From<ChannelRow> for ChannelShort {
    fn from(row: ChannelRow) -> Self {
        Self {
            id: row.id,
            server_id: row.server_id,
            name: row.name,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ChannelDetailedRow {
    id: ChannelId,
    server_id: ServerId,
    name: String,
    created_at: NaiveDateTime,
}
impl From<ChannelDetailedRow> for ChannelDetailed {
    fn from(row: ChannelDetailedRow) -> Self {
        Self {
            id: row.id,
            server_id: row.server_id,
            name: row.name,
            created_at: row.created_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ChannelShortRow {
    id: ChannelId,
    server_id: ServerId,
    name: String,
}

impl From<ChannelShortRow> for ChannelShort {
    fn from(row: ChannelShortRow) -> Self {
        Self {
            id: row.id,
            server_id: row.server_id,
            name: row.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelChangeset, ChannelDetailedRow, ChannelRow, ChannelShortRow, NewChannel};
    use crate::server::channel::error::ChannelError;
    use shared::models::server::channel::channel::{
        Channel, ChannelChangesetInput, ChannelDetailed, ChannelShort, NewChannelInput,
    };

    #[test]
    fn creation_and_changeset_normalize_names() {
        let channel = NewChannel::try_from((
            3,
            NewChannelInput {
                name: "  general  ".into(),
            },
        ))
        .unwrap();
        let changeset = ChannelChangeset::try_from(ChannelChangesetInput {
            name: Some("  updates  ".into()),
        })
        .unwrap();

        assert_eq!(channel.name, "general");
        assert_eq!(changeset.name.as_deref(), Some("updates"));
    }

    #[test]
    fn changeset_requires_a_name() {
        assert!(matches!(
            ChannelChangeset::try_from(ChannelChangesetInput { name: None }),
            Err(ChannelError::InvalidChangeset(_))
        ));
    }

    #[test]
    fn creation_and_changeset_enforce_name_boundaries() {
        for name in ["a".repeat(5), "é".repeat(10)] {
            assert!(NewChannel::try_from((3, NewChannelInput { name: name.clone() })).is_ok());
            assert!(ChannelChangeset::try_from(ChannelChangesetInput { name: Some(name) }).is_ok());
        }
        for name in ["a".repeat(4), "a".repeat(11)] {
            assert!(matches!(
                NewChannel::try_from((3, NewChannelInput { name: name.clone() })),
                Err(ChannelError::InvalidInput(_))
            ));
            assert!(matches!(
                ChannelChangeset::try_from(ChannelChangesetInput { name: Some(name) }),
                Err(ChannelError::InvalidChangeset(_))
            ));
        }
    }

    #[test]
    fn model_accessors_expose_normalized_values() {
        let channel = NewChannel::try_from((
            3,
            NewChannelInput {
                name: "  general  ".into(),
            },
        ))
        .unwrap();
        assert_eq!(channel.name(), "general");

        let changeset = ChannelChangeset::try_from(ChannelChangesetInput {
            name: Some("  updates  ".into()),
        })
        .unwrap();
        assert_eq!(changeset.name(), Some("updates"));
    }

    #[test]
    fn row_converts_to_shared_channel_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let channel = ChannelDetailed::from(ChannelRow {
            id: 5,
            server_id: 3,
            name: "general".into(),
            created_at,
        });
        assert_eq!(channel.id, 5);
        assert_eq!(channel.server_id, 3);
    }

    #[test]
    fn rows_convert_to_each_shared_projection() {
        let created_at = chrono::NaiveDateTime::default();
        let channel = Channel::from(ChannelRow {
            id: 5,
            server_id: 3,
            name: "general".into(),
            created_at,
        });
        let short = ChannelShort::from(ChannelRow {
            id: 5,
            server_id: 3,
            name: "general".into(),
            created_at,
        });
        let detailed = ChannelDetailed::from(ChannelDetailedRow {
            id: 5,
            server_id: 3,
            name: "general".into(),
            created_at,
        });
        let short_row = ChannelShort::from(ChannelShortRow {
            id: 5,
            server_id: 3,
            name: "general".into(),
        });

        assert_eq!(channel.created_at, created_at);
        assert_eq!(short.name, "general");
        assert_eq!(detailed.server_id, 3);
        assert_eq!(short_row.id, 5);
    }
}
