use diesel::Connection;

use crate::infra::Database;
use crate::member::models::MemberId;
use crate::server::channel::models::{ChannelChangeset, MultiChannelDetails};
use crate::server::membership::repository::ServerMembershipRepository;
use crate::server::models::ServerId;
use crate::server::repository::ServerRepository;

use super::error::ChannelError;
use super::models::{
    ChannelChangesetInput, ChannelDetails, ChannelId, ChannelPreview, MultiChannelPreview,
    NewChannel, NewChannelInput,
};
use super::repository::ChannelRepository;

pub struct ChannelService {
    database: Database,
}
impl ChannelService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    // Create
    pub fn create_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        input: NewChannelInput,
    ) -> Result<ChannelDetails, ChannelError> {
        let new_channel = NewChannel::try_from((server_id, input))?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetails, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ChannelError::PermissionDenied);
            } else if ChannelRepository::exists_by_name(conn, server_id, new_channel.name())? {
                return Err(ChannelError::NameTaken);
            }

            ChannelRepository::create(conn, new_channel)
                .map(ChannelDetails::from)
                .map_err(ChannelError::from)
        })
    }

    // Read
    pub fn get_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<ChannelDetails, ChannelError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetails, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ChannelError::MembershipRequired);
            } else if !ChannelRepository::exists(conn, server_id, channel_id)? {
                return Err(ChannelError::NotFound);
            }

            ChannelRepository::read(conn, channel_id)
                .map(ChannelDetails::from)
                .map_err(ChannelError::from)
        })
    }
    pub fn get_server_channels(
        &self,
        member_id: MemberId,
        server_id: ServerId,
    ) -> Result<Vec<ChannelDetails>, ChannelError> {
        let mut conn = self.database.get_connection()?;

        conn.transaction::<MultiChannelDetails, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ChannelError::MembershipRequired);
            }

            ChannelRepository::get_multi(conn, server_id)
                .map(|channels| channels.into_iter().map(ChannelDetails::from).collect())
                .map_err(ChannelError::from)
        })
    }
    pub fn get_server_channel_previews(
        &self,
        member_id: MemberId,
        server_id: ServerId,
    ) -> Result<MultiChannelPreview, ChannelError> {
        let mut conn = self.database.get_connection()?;

        conn.transaction::<MultiChannelPreview, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ChannelError::MembershipRequired);
            }

            ChannelRepository::get_multi(conn, server_id)
                .map(|channels| channels.into_iter().map(ChannelPreview::from).collect())
                .map_err(ChannelError::from)
        })
    }

    // Update
    pub fn update_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        input: ChannelChangesetInput,
    ) -> Result<ChannelDetails, ChannelError> {
        let changeset = ChannelChangeset::try_from(input)?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetails, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ChannelError::PermissionDenied);
            }
            if !ChannelRepository::exists(conn, server_id, channel_id)? {
                return Err(ChannelError::NotFound);
            } else if let Some(name) = changeset.name() {
                if ChannelRepository::exists_by_name_excluding(conn, server_id, channel_id, name)? {
                    return Err(ChannelError::NameTaken);
                }
            }

            ChannelRepository::update(conn, channel_id, changeset)
                .map(ChannelDetails::from)
                .map_err(ChannelError::from)
        })
    }

    // Delete
    pub fn delete_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<(), ChannelError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), ChannelError, _>(|conn| {
            if !ChannelRepository::exists(conn, server_id, channel_id)? {
                return Err(ChannelError::NotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ChannelError::PermissionDenied);
            }

            ChannelRepository::delete(conn, channel_id).map_err(ChannelError::from)
        })
    }
}
