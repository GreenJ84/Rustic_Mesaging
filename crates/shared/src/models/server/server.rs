use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::member::MemberId;

pub type ServerId = i32;

pub const SERVER_ICON_SHAPES: &[&str] = &["badge", "building", "bug", "default"];
pub const SERVER_ICON_COLORS: &[&str] = &[
    "limegreen",
    "yellow",
    "lightblue",
    "purple",
    "green",
    "white",
    "red",
    "blue",
    "orange",
    "pink",
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewServerInput {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerChangesetInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub owner_id: MemberId,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerDetailed {
    pub id: ServerId,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub owner_id: MemberId,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerShort {
    pub id: ServerId,
    pub name: String,
    pub icon: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerList<T> {
    pub servers: Vec<T>,
}

pub type MultiServer = ServerList<ServerDetailed>;
pub type MultiServerPreview = ServerList<ServerShort>;

#[cfg(test)]
mod tests {
    use super::{NewServerInput, ServerChangesetInput};

    #[test]
    fn creation_input_omits_server_owned_fields() {
        let value = serde_json::to_value(NewServerInput {
            name: "Rust".into(),
            description: "A Rust community".into(),
            icon: None,
        })
        .unwrap();

        assert_eq!(value["name"], "Rust");
        assert!(value.get("owner_id").is_none());
        assert!(value.get("id").is_none());
    }

    #[test]
    fn changeset_preserves_explicit_optional_fields() {
        let value = serde_json::to_value(ServerChangesetInput {
            name: Some("Updated".into()),
            description: None,
            icon: None,
        })
        .unwrap();

        assert_eq!(value["name"], "Updated");
        assert!(value["description"].is_null());
    }
}
