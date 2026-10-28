use diesel::Connection;

use crate::infra::Database;
use crate::member::models::{MemberId, MultiMemberPreview};
use crate::server::membership::error::MembershipError;
use crate::server::membership::repository::ServerMembershipRepository;
use crate::utils::Validator;

use super::errors::ServerError;
use super::membership::models::NewServerMembership;
use super::models::{
    MultiServerPreview, NewServer, ServerChangeset, ServerChangesetInput, ServerCreateInput,
    ServerDetails, ServerId, ServerPreview,
};
use super::repository::ServerRepository;

pub struct ServerService {
    database: Database,
}
impl ServerService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    // ===============================
    // Server Services
    // ===============================
    // Create
    pub fn create_server(
        &self,
        member_id: MemberId,
        input: ServerCreateInput,
    ) -> Result<ServerDetails, ServerError> {
        let new_server = NewServer::try_from((member_id, input))?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ServerDetails, ServerError, _>(|conn| {
            if ServerRepository::name_taken(conn, new_server.name())? {
                return Err(ServerError::NameTaken);
            }
            let new_server = ServerRepository::create(conn, new_server)
                .map(ServerDetails::from)
                .map_err(ServerError::from)?;
            ServerMembershipRepository::create(
                conn,
                NewServerMembership {
                    member_id,
                    server_id: new_server.id,
                },
            )?;
            Ok(new_server.into())
        })
    }

    // Read
    pub fn get_server(&self, server_id: ServerId) -> Result<ServerDetails, ServerError> {
        let mut conn = self.database.get_connection()?;
        ServerRepository::read(&mut conn, server_id)
            .map(ServerDetails::from)
            .map_err(ServerError::from)
    }

    pub fn get_servers_list(&self, page: u32) -> Result<MultiServerPreview, ServerError> {
        let mut conn = self.database.get_connection()?;
        ServerRepository::read_page(&mut conn, page)
            .map(|servers| MultiServerPreview {
                servers: servers.into_iter().map(ServerPreview::from).collect(),
            })
            .map_err(ServerError::from)
    }

    pub fn search_servers(&self, search_term: String) -> Result<MultiServerPreview, ServerError> {
        let search = Validator::validate_search_term(search_term).map_err(|e| {
            ServerError::InvalidSearchTerm(format!("Servers search term {}", e.message()))
        })?;

        let mut conn = self.database.get_connection()?;
        ServerRepository::search(&mut conn, search)
            .map(|servers| MultiServerPreview {
                servers: servers.into_iter().map(ServerPreview::from).collect(),
            })
            .map_err(ServerError::from)
    }

    // Update
    pub fn update_server(
        &self,
        server_id: ServerId,
        member_id: MemberId,
        input: ServerChangesetInput,
    ) -> Result<ServerDetails, ServerError> {
        let changeset = ServerChangeset::try_from(input)?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ServerDetails, ServerError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ServerError::NotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ServerError::NotOwner);
            }
            ServerRepository::update(conn, server_id, changeset)
                .map(ServerDetails::from)
                .map_err(ServerError::from)
        })
    }

    // Delete
    pub fn delete_server(
        &self,
        server_id: ServerId,
        member_id: MemberId,
    ) -> Result<(), ServerError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), ServerError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ServerError::NotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ServerError::NotOwner);
            }
            ServerRepository::delete(conn, server_id).map_err(ServerError::from)
        })
    }

    // ===============================
    // Server Membership Services
    // ===============================
    // Create
    pub fn join_server(&self, server_id: ServerId, member_id: MemberId) -> Result<(), ServerError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), ServerError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ServerError::NotFound);
            }
            ServerMembershipRepository::create(
                conn,
                NewServerMembership {
                    member_id,
                    server_id,
                },
            )
            .map_err(ServerError::from)
        })
    }

    // Read
    pub fn get_member_servers(
        &self,
        member_id: MemberId,
        page: u32,
    ) -> Result<MultiServerPreview, ServerError> {
        let mut conn = self.database.get_connection()?;
        ServerMembershipRepository::get_member_servers(&mut conn, member_id, page)
            .map(|servers| MultiServerPreview { servers })
            .map_err(ServerError::from)
    }
    pub fn get_server_members(
        &self,
        server_id: ServerId,
        member_id: MemberId,
        page: u32,
    ) -> Result<MultiMemberPreview, ServerError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<MultiMemberPreview, ServerError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ServerError::NotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ServerError::Membership(MembershipError::MembershipRequired));
            }
            ServerMembershipRepository::get_server_members(conn, server_id, page)
                .map(|members| MultiMemberPreview { members })
                .map_err(ServerError::from)
        })
    }

    // Delete
    pub fn leave_server(
        &self,
        server_id: ServerId,
        member_id: MemberId,
    ) -> Result<(), ServerError> {
        let mut conn = self.database.get_connection()?;

        conn.transaction::<(), ServerError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ServerError::NotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ServerError::Membership(MembershipError::MembershipNotFound));
            } else if ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ServerError::Membership(MembershipError::OwnerCannotLeave));
            }
            ServerMembershipRepository::delete(conn, server_id, member_id)
                .map_err(|err| MembershipError::from(err).into())
        })
    }
}
