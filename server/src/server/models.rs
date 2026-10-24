use chrono::NaiveDateTime;
use diesel::query_builder::AsChangeset;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::member::models::{Member, MemberId};
use crate::schema::server;

use super::errors::ServerError;

pub type ServerId = i32;

// ================================
// Server Creation Models
// ================================
#[derive(FromForm)]
pub(crate) struct ServerCreateInput {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) icon: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = server)]
pub(super) struct NewServer {
    name: String,
    description: String,
    icon: Option<String>,
    owner_id: MemberId,
}
impl TryFrom<(MemberId, ServerCreateInput)> for NewServer {
    type Error = ServerError;
    fn try_from(
        (member_id, input): (MemberId, ServerCreateInput),
    ) -> Result<NewServer, Self::Error> {
        let name = input.name.trim().to_owned();
        let description = input.description.trim().to_owned();
        let icon = input
            .icon
            .map(|icon| icon.trim().to_owned())
            .filter(|icon| !icon.is_empty());

        if !(5..=80).contains(&name.chars().count()) {
            return Err(ServerError::InvalidInput(
                "Server name must be between 5 and 80 characters".into(),
            ));
        }
        if !(10..=300).contains(&description.chars().count()) {
            return Err(ServerError::InvalidInput(
                "Server description must be between 10 and 300 characters".into(),
            ));
        }

        Ok(Self {
            name,
            description,
            icon,
            owner_id: member_id,
        })
    }
}

// ================================
// Server Update Models
// ================================
#[derive(FromForm)]
pub(crate) struct ServerChangesetInput {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) icon: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = server)]
pub(super) struct ServerChangeset {
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
}
impl TryFrom<ServerChangesetInput> for ServerChangeset {
    type Error = ServerError;
    fn try_from(input: ServerChangesetInput) -> Result<ServerChangeset, Self::Error> {
        let name = input.name.map(|name| name.trim().to_owned());
        let description = input
            .description
            .map(|description| description.trim().to_owned());
        let icon = validate_server_icon(input.icon).map_err(ServerError::InvalidChangeset)?;

        if name.is_none() && description.is_none() && icon.is_none() {
            return Err(ServerError::InvalidChangeset(
                "At least one field must be provided for update".into(),
            ));
        }
        if let Some(ref name) = name {
            if !(5..=80).contains(&name.chars().count()) {
                return Err(ServerError::InvalidChangeset(
                    "Server name must be between 5 and 80 characters".into(),
                ));
            }
        }
        if let Some(ref description) = description {
            if !(10..=300).contains(&description.chars().count()) {
                return Err(ServerError::InvalidChangeset(
                    "Server description must be between 10 and 300 characters".into(),
                ));
            }
        }

        Ok(Self {
            name,
            description,
            icon,
        })
    }
}

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(table_name = server)]
#[diesel(belongs_to(Member, foreign_key = owner_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct Server {
    id: ServerId,
    name: String,
    description: String,
    icon: String,
    owner_id: MemberId,
    created_at: NaiveDateTime,
}

const SERVER_ICON_SHAPES: &[&str] = &["badge", "building", "bug", "default"];

const SERVER_ICON_COLORS: &[&str] = &[
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

fn validate_server_icon(icon: Option<String>) -> Result<Option<String>, String> {
    let Some(icon) = icon else {
        return Ok(None); // INSERT uses the database default
    };

    let icon = icon.trim();

    let Some((shape, color)) = icon.split_once(':') else {
        return Err("Server icon must use the '<shape>:<color>' format".into());
    };

    if !SERVER_ICON_SHAPES.contains(&shape) {
        return Err("Unsupported server icon".into());
    }

    if !SERVER_ICON_COLORS.contains(&color) {
        return Err("Unsupported server icon color".into());
    }

    Ok(Some(icon.to_owned()))
}
// ================================
// Server DTO Models
// ================================
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerDetails {
    pub id: ServerId,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub owner_id: MemberId,
    pub created_at: NaiveDateTime,
}
impl From<Server> for ServerDetails {
    fn from(server: Server) -> Self {
        Self {
            id: server.id,
            name: server.name,
            description: server.description,
            icon: server.icon,
            owner_id: server.owner_id,
            created_at: server.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerPreview {
    pub id: ServerId,
    pub name: String,
    pub icon: String,
}
impl From<Server> for ServerPreview {
    fn from(server: Server) -> Self {
        Self {
            id: server.id,
            name: server.name,
            icon: server.icon,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerList<T> {
    pub(crate) servers: Vec<T>,
}

pub type MultiServer = ServerList<ServerDetails>;
pub type MultiServerPreview = ServerList<ServerPreview>;
