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
impl NewServer {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
    pub fn owner_id(&self) -> MemberId {
        self.owner_id
    }
}
impl TryFrom<(MemberId, ServerCreateInput)> for NewServer {
    type Error = ServerError;
    fn try_from(
        (member_id, input): (MemberId, ServerCreateInput),
    ) -> Result<NewServer, Self::Error> {
        let name = input.name.trim().to_owned();
        let description = input.description.trim().to_owned();
        let icon = validate_server_icon(input.icon).map_err(ServerError::InvalidInput)?;

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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{
        NewServer, Server, ServerChangeset, ServerChangesetInput, ServerCreateInput, ServerDetails,
        ServerPreview,
    };
    use crate::server::errors::ServerError;

    fn create_input(name: &str, description: &str, icon: Option<&str>) -> ServerCreateInput {
        ServerCreateInput {
            name: name.into(),
            description: description.into(),
            icon: icon.map(str::to_owned),
        }
    }

    fn update_input(
        name: Option<&str>,
        description: Option<&str>,
        icon: Option<&str>,
    ) -> ServerChangesetInput {
        ServerChangesetInput {
            name: name.map(str::to_owned),
            description: description.map(str::to_owned),
            icon: icon.map(str::to_owned),
        }
    }

    fn server() -> Server {
        Server {
            id: 7,
            name: "Example Server".into(),
            description: "An example server used by model tests".into(),
            icon: "badge:green".into(),
            owner_id: 11,
            created_at: NaiveDate::from_ymd_opt(2026, 1, 2)
                .unwrap()
                .and_hms_opt(3, 4, 5)
                .unwrap(),
        }
    }

    #[test]
    fn creation_normalizes_input_and_binds_owner() {
        let server = NewServer::try_from((
            11,
            create_input(
                "  Example Server  ",
                "  A sufficiently long description  ",
                Some("  badge:green  "),
            ),
        ))
        .unwrap();

        assert_eq!(server.name, "Example Server");
        assert_eq!(server.description, "A sufficiently long description");
        assert_eq!(server.icon.as_deref(), Some("badge:green"));
        assert_eq!(server.owner_id, 11);
    }

    #[test]
    fn creation_omits_icon_to_use_database_default() {
        let server = NewServer::try_from((
            11,
            create_input("Example", "A valid server description", None),
        ))
        .unwrap();

        assert_eq!(server.icon, None);
    }

    #[test]
    fn creation_rejects_invalid_name_and_description_lengths() {
        assert!(matches!(
            NewServer::try_from((11, create_input("four", "A valid description", None))),
            Err(ServerError::InvalidInput(_))
        ));
        assert!(matches!(
            NewServer::try_from((11, create_input("Valid", "too short", None))),
            Err(ServerError::InvalidInput(_))
        ));
    }

    #[test]
    fn creation_rejects_invalid_icons() {
        for icon in ["unknown:red", "badge:unknown", "badge", "   "] {
            assert!(matches!(
                NewServer::try_from((
                    11,
                    create_input("Example", "A valid server description", Some(icon)),
                )),
                Err(ServerError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn update_rejects_empty_changeset() {
        assert!(matches!(
            ServerChangeset::try_from(update_input(None, None, None)),
            Err(ServerError::InvalidChangeset(_))
        ));
    }

    #[test]
    fn update_normalizes_supplied_fields() {
        let changeset = ServerChangeset::try_from(update_input(
            Some("  Updated Server  "),
            Some("  An updated server description  "),
            Some("  building:blue  "),
        ))
        .unwrap();

        assert_eq!(changeset.name.as_deref(), Some("Updated Server"));
        assert_eq!(
            changeset.description.as_deref(),
            Some("An updated server description")
        );
        assert_eq!(changeset.icon.as_deref(), Some("building:blue"));
    }

    #[test]
    fn update_rejects_blank_or_invalid_supplied_fields() {
        for input in [
            update_input(Some("   "), None, None),
            update_input(None, Some("   "), None),
            update_input(None, None, Some("bug:unknown")),
            update_input(None, None, Some("   ")),
        ] {
            assert!(matches!(
                ServerChangeset::try_from(input),
                Err(ServerError::InvalidChangeset(_))
            ));
        }
    }

    #[test]
    fn persistence_model_converts_to_details_and_preview() {
        let details = ServerDetails::from(server());
        assert_eq!(details.id, 7);
        assert_eq!(details.name, "Example Server");
        assert_eq!(details.icon, "badge:green");
        assert_eq!(details.owner_id, 11);

        let preview = ServerPreview::from(server());
        assert_eq!(preview.id, 7);
        assert_eq!(preview.name, "Example Server");
        assert_eq!(preview.icon, "badge:green");
    }
}
