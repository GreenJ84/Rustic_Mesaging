use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::server::server::ServerId;

pub type ChannelId = i32;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewChannelInput {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelChangesetInput {
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Channel {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelDetailed {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelShort {
    pub id: ChannelId,
    pub server_id: ServerId,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelList<T> {
    pub channels: Vec<T>,
}

pub type MultiChannel = ChannelList<ChannelDetailed>;
pub type MultiChannelPreview = ChannelList<ChannelShort>;

#[cfg(test)]
mod tests {
    use super::{ChannelChangesetInput, ChannelList, NewChannelInput};

    #[test]
    fn creation_input_contains_only_the_channel_name() {
        let value = serde_json::to_value(NewChannelInput {
            name: "general".into(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "name": "general" }));
    }

    #[test]
    fn channel_collections_use_the_channels_field() {
        let value = serde_json::to_value(ChannelList::<serde_json::Value> {
            channels: Vec::new(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "channels": [] }));
    }

    #[test]
    fn changeset_preserves_an_explicit_missing_name() {
        let value = serde_json::to_value(ChannelChangesetInput { name: None }).unwrap();

        assert!(value["name"].is_null());
    }
}
