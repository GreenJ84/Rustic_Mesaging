use chrono::NaiveDateTime;
use diesel::query_builder::AsChangeset;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::channel;
use crate::server::models::{Server, ServerId};
use crate::utils::{Page, ValidationError, Validator};

use super::error::ChannelError;

pub type ChannelId = i32;

fn validate_channel_name(name: String) -> Result<String, ValidationError> {
    Validator::validate_text(name, Some(5), Some(10))
}

#[derive(Deserialize, Serialize, Debug, FromForm)]
pub struct NewChannelInput {
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct NewChannel {
    server_id: ServerId,
    name: String,
}
impl NewChannel {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}
impl TryFrom<(ServerId, NewChannelInput)> for NewChannel {
    type Error = ChannelError;
    fn try_from((server_id, input): (ServerId, NewChannelInput)) -> Result<Self, Self::Error> {
        let name = validate_channel_name(input.name)
            .map_err(|e| ChannelError::InvalidInput(format!("Channel name {}", e.message())))?;
        Ok(Self { server_id, name })
    }
}

#[derive(Deserialize, Serialize, Debug, FromForm)]
pub struct ChannelChangesetInput {
    pub name: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct ChannelChangeset {
    name: Option<String>,
}
impl ChannelChangeset {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
impl TryFrom<ChannelChangesetInput> for ChannelChangeset {
    type Error = ChannelError;
    fn try_from(input: ChannelChangesetInput) -> Result<Self, Self::Error> {
        // Empty changeset check
        if input.name.is_none() {
            return Err(ChannelError::InvalidChangeset(
                "No changes provided".to_string(),
            ));
        }
        // Field validation
        let name = if let Some(name) = input.name {
            let name = validate_channel_name(name).map_err(|e| {
                ChannelError::InvalidChangeset(format!("Channel name {}", e.message()))
            })?;
            Some(name)
        } else {
            None
        };
        Ok(Self { name })
    }
}
#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(table_name = channel)]
#[diesel(belongs_to(Server, foreign_key = server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct Channel {
    id: ChannelId,
    server_id: ServerId,
    name: String,
    created_at: NaiveDateTime,
}
impl Channel {
    pub fn id(&self) -> ChannelId {
        self.id
    }
    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelDetails {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
    pub created_at: NaiveDateTime,
}
impl From<Channel> for ChannelDetails {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id,
            server_id: channel.server_id,
            name: channel.name,
            created_at: channel.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelPreview {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
}
impl From<Channel> for ChannelPreview {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id,
            server_id: channel.server_id,
            name: channel.name.to_owned(),
        }
    }
}

pub type MultiChannelPreview = Vec<ChannelPreview>;
pub type ChannelPreviewPage = Page<MultiChannelPreview>;

pub type MultiChannelDetails = Vec<ChannelDetails>;
pub type ChannelDetailsPage = Page<MultiChannelDetails>;
