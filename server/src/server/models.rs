use chrono::NaiveDateTime;
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};

use super::errors::ServerError;
use crate::member::models::MemberRow;
use crate::schema::server;
use crate::utils::{ValidationError, Validator};
use shared::models::member::MemberId;
use shared::models::server::server::{
    NewServerInput, Server, ServerChangesetInput, ServerDetailed, ServerId, ServerShort,
    SERVER_ICON_COLORS, SERVER_ICON_SHAPES,
};

fn validate_server_name(name: String) -> Result<String, ValidationError> {
    Validator::validate_text(&name, Some(5), Some(80))
}

fn validate_server_description(description: String) -> Result<String, ValidationError> {
    Validator::validate_text(&description, Some(10), Some(300))
}

fn validate_server_icon(icon: Option<String>) -> Result<Option<String>, String> {
    let Some(icon) = icon else {
        return Ok(None);
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

#[derive(Insertable)]
#[diesel(table_name = server)]
pub(in crate::server) struct NewServer {
    name: String,
    description: String,
    icon: Option<String>,
    owner_id: MemberId,
}
impl NewServer {
    pub(in crate::server) fn name(&self) -> &str {
        &self.name
    }
}
impl TryFrom<(MemberId, NewServerInput)> for NewServer {
    type Error = ServerError;

    fn try_from((member_id, input): (MemberId, NewServerInput)) -> Result<Self, Self::Error> {
        let name = validate_server_name(input.name).map_err(|error| {
            ServerError::InvalidInput(format!("Server name {}", error.message()))
        })?;
        let description = validate_server_description(input.description).map_err(|error| {
            ServerError::InvalidInput(format!("Server description {}", error.message()))
        })?;
        let icon = validate_server_icon(input.icon).map_err(ServerError::InvalidInput)?;
        Ok(Self {
            name,
            description,
            icon,
            owner_id: member_id,
        })
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = server)]
pub(in crate::server) struct ServerChangeset {
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
}

impl TryFrom<ServerChangesetInput> for ServerChangeset {
    type Error = ServerError;

    fn try_from(input: ServerChangesetInput) -> Result<Self, Self::Error> {
        if input.name.is_none() && input.description.is_none() && input.icon.is_none() {
            return Err(ServerError::InvalidChangeset(
                "At least one field must be provided for update".into(),
            ));
        }
        let name = input
            .name
            .map(validate_server_name)
            .transpose()
            .map_err(|error| {
                ServerError::InvalidChangeset(format!("Server name {}", error.message()))
            })?;
        let description = input
            .description
            .map(validate_server_description)
            .transpose()
            .map_err(|error| {
                ServerError::InvalidChangeset(format!("Server description {}", error.message()))
            })?;
        let icon = validate_server_icon(input.icon).map_err(ServerError::InvalidChangeset)?;
        Ok(Self {
            name,
            description,
            icon,
        })
    }
}

#[derive(Associations, Identifiable, Queryable, Selectable)]
#[diesel(table_name = server)]
#[diesel(belongs_to(MemberRow, foreign_key = owner_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ServerRow {
    id: ServerId,
    name: String,
    description: String,
    icon: String,
    owner_id: MemberId,
    created_at: NaiveDateTime,
}
impl ServerRow {
    pub(in crate::server) fn id(&self) -> ServerId {
        self.id
    }
    pub(in crate::server) fn name(&self) -> &str {
        &self.name
    }
    pub(in crate::server) fn owner_id(&self) -> MemberId {
        self.owner_id
    }
    pub(in crate::server) fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}
impl From<ServerRow> for Server {
    fn from(row: ServerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            icon: row.icon,
            owner_id: row.owner_id,
            created_at: row.created_at,
        }
    }
}
impl From<ServerRow> for ServerDetailed {
    fn from(row: ServerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            icon: row.icon,
            owner_id: row.owner_id,
            created_at: row.created_at,
        }
    }
}
impl From<ServerRow> for ServerShort {
    fn from(row: ServerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            icon: row.icon,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = server)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ServerDetailedRow {
    id: ServerId,
    name: String,
    description: String,
    icon: String,
    owner_id: MemberId,
    created_at: NaiveDateTime,
}
impl ServerDetailedRow {
    pub(in crate::server) fn id(&self) -> ServerId {
        self.id
    }
    pub(in crate::server) fn name(&self) -> &str {
        &self.name
    }
    pub(in crate::server) fn owner_id(&self) -> MemberId {
        self.owner_id
    }
    pub(in crate::server) fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}
impl From<ServerDetailedRow> for ServerDetailed {
    fn from(row: ServerDetailedRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            icon: row.icon,
            owner_id: row.owner_id,
            created_at: row.created_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = server)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::server) struct ServerShortRow {
    id: ServerId,
    name: String,
    icon: String,
}
impl From<ServerShortRow> for ServerShort {
    fn from(row: ServerShortRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            icon: row.icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NewServer, ServerChangeset, ServerRow, ServerShortRow};
    use crate::server::errors::ServerError;
    use shared::models::server::server::{
        NewServerInput, ServerChangesetInput, ServerDetailed, ServerShort,
    };

    fn create_input(name: &str, description: &str, icon: Option<&str>) -> NewServerInput {
        NewServerInput {
            name: name.into(),
            description: description.into(),
            icon: icon.map(str::to_owned),
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
    fn creation_rejects_invalid_fields() {
        assert!(matches!(
            NewServer::try_from((11, create_input("four", "A valid description", None))),
            Err(ServerError::InvalidInput(_))
        ));
        assert!(matches!(
            NewServer::try_from((
                11,
                create_input("Example", "A valid description", Some("unknown:red"))
            )),
            Err(ServerError::InvalidInput(_))
        ));
    }

    #[test]
    fn changeset_requires_and_normalizes_fields() {
        assert!(matches!(
            ServerChangeset::try_from(ServerChangesetInput {
                name: None,
                description: None,
                icon: None,
            }),
            Err(ServerError::InvalidChangeset(_))
        ));
        let changeset = ServerChangeset::try_from(ServerChangesetInput {
            name: Some("  Updated Server  ".into()),
            description: None,
            icon: None,
        })
        .unwrap();
        assert_eq!(changeset.name.as_deref(), Some("Updated Server"));
    }

    #[test]
    fn rows_convert_to_shared_contracts() {
        let created_at = chrono::NaiveDateTime::default();
        let detailed = ServerDetailed::from(ServerRow {
            id: 7,
            name: "Example Server".into(),
            description: "An example server".into(),
            icon: "badge:green".into(),
            owner_id: 11,
            created_at,
        });
        let short = ServerShort::from(ServerShortRow {
            id: 7,
            name: "Example Server".into(),
            icon: "badge:green".into(),
        });

        assert_eq!(detailed.owner_id, 11);
        assert_eq!(short.icon, "badge:green");
    }
}
