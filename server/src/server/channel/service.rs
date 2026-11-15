use diesel::Connection;

use crate::infra::Database;
use crate::server::channel::models::ChannelChangeset;
use crate::server::membership::repository::ServerMembershipRepository;
use crate::server::repository::ServerRepository;
use shared::models::member::MemberId;
use shared::models::server::channel::channel::{
    ChannelChangesetInput, ChannelDetailed, ChannelId, ChannelShort, MultiChannel,
    MultiChannelPreview, NewChannelInput,
};
use shared::models::server::server::ServerId;

use super::error::ChannelError;
use super::models::NewChannel;
use super::repository::ChannelRepository;

pub(crate) struct ChannelService {
    database: Database,
}
impl ChannelService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    // Create
    pub(crate) fn create_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        input: NewChannelInput,
    ) -> Result<ChannelDetailed, ChannelError> {
        let new_channel = NewChannel::try_from((server_id, input))?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetailed, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerRepository::is_owner(conn, server_id, member_id)? {
                return Err(ChannelError::PermissionDenied);
            } else if ChannelRepository::exists_by_name(conn, server_id, new_channel.name())? {
                return Err(ChannelError::NameTaken);
            }

            ChannelRepository::create(conn, new_channel)
                .map(ChannelDetailed::from)
                .map_err(ChannelError::from)
        })
    }

    // Read
    pub(crate) fn get_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<ChannelDetailed, ChannelError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetailed, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ChannelError::MembershipRequired);
            } else if !ChannelRepository::exists(conn, server_id, channel_id)? {
                return Err(ChannelError::NotFound);
            }

            ChannelRepository::read(conn, channel_id)
                .map(ChannelDetailed::from)
                .map_err(ChannelError::from)
        })
    }
    pub(crate) fn get_server_channels(
        &self,
        member_id: MemberId,
        server_id: ServerId,
    ) -> Result<MultiChannel, ChannelError> {
        let mut conn = self.database.get_connection()?;

        conn.transaction::<MultiChannel, ChannelError, _>(|conn| {
            if !ServerRepository::exists(conn, server_id)? {
                return Err(ChannelError::ServerNotFound);
            } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
                return Err(ChannelError::MembershipRequired);
            }

            ChannelRepository::read_many(conn, server_id)
                .map(|channels| MultiChannel {
                    channels: channels.into_iter().map(ChannelDetailed::from).collect(),
                })
                .map_err(ChannelError::from)
        })
    }
    pub(crate) fn get_server_channel_previews(
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

            ChannelRepository::read_many_short(conn, server_id)
                .map(|channels| MultiChannelPreview {
                    channels: channels.into_iter().map(ChannelShort::from).collect(),
                })
                .map_err(ChannelError::from)
        })
    }

    // Update
    pub(crate) fn update_channel(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        input: ChannelChangesetInput,
    ) -> Result<ChannelDetailed, ChannelError> {
        let changeset = ChannelChangeset::try_from(input)?;

        let mut conn = self.database.get_connection()?;
        conn.transaction::<ChannelDetailed, ChannelError, _>(|conn| {
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
                .map(ChannelDetailed::from)
                .map_err(ChannelError::from)
        })
    }

    // Delete
    pub(crate) fn delete_channel(
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
