use diesel::{dsl, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::infra::{sql::lower, DbConn, DbError};
use crate::schema::channel::dsl as c_dsl;
use shared::models::server::channel::channel::ChannelId;
use shared::models::server::server::ServerId;

use super::models::{ChannelChangeset, ChannelRow, ChannelShortRow, NewChannel};

pub(in crate::server) struct ChannelRepository;

impl ChannelRepository {
    // Existence
    pub(super) fn exists(
        conn: &mut DbConn,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<bool, DbError> {
        dsl::select(dsl::exists(
            c_dsl::channel
                .find(channel_id)
                .filter(c_dsl::server_id.eq(server_id)),
        ))
        .get_result::<bool>(conn)
        .map_err(DbError::from)
    }
    pub(super) fn exists_by_name(
        conn: &mut DbConn,
        server_id: ServerId,
        name: &str,
    ) -> Result<bool, DbError> {
        let normalized_name = name.to_lowercase();

        dsl::select(dsl::exists(
            c_dsl::channel
                .filter(c_dsl::server_id.eq(server_id))
                .filter(lower(c_dsl::name).eq(normalized_name)),
        ))
        .get_result(conn)
        .map_err(DbError::from)
    }
    pub(super) fn exists_by_name_excluding(
        conn: &mut DbConn,
        server_id: ServerId,
        channel_id: ChannelId,
        name: &str,
    ) -> Result<bool, DbError> {
        let normalized_name = name.to_lowercase();

        dsl::select(dsl::exists(
            c_dsl::channel
                .filter(c_dsl::server_id.eq(server_id))
                .filter(c_dsl::id.ne(channel_id))
                .filter(lower(c_dsl::name).eq(normalized_name)),
        ))
        .get_result(conn)
        .map_err(DbError::from)
    }

    // Create
    pub(in crate::server) fn create(
        conn: &mut DbConn,
        new_channel: NewChannel,
    ) -> Result<ChannelRow, DbError> {
        diesel::insert_into(c_dsl::channel)
            .values(&new_channel)
            .returning(ChannelRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }
    pub(in crate::server) fn read(conn: &mut DbConn, id: ChannelId) -> Result<ChannelRow, DbError> {
        c_dsl::channel
            .find(id)
            .select(ChannelRow::as_select())
            .first(conn)
            .map_err(DbError::from)
    }
    pub(in crate::server) fn read_many(
        conn: &mut DbConn,
        server_id: ServerId,
    ) -> Result<Vec<ChannelRow>, DbError> {
        c_dsl::channel
            .filter(c_dsl::server_id.eq(server_id))
            .order_by(c_dsl::name.asc())
            .select(ChannelRow::as_select())
            .load(conn)
            .map_err(DbError::from)
    }

    pub(in crate::server) fn read_many_short(
        conn: &mut DbConn,
        server_id: ServerId,
    ) -> Result<Vec<ChannelShortRow>, DbError> {
        c_dsl::channel
            .filter(c_dsl::server_id.eq(server_id))
            .order_by(c_dsl::name.asc())
            .select(ChannelShortRow::as_select())
            .load(conn)
            .map_err(DbError::from)
    }

    // Update
    pub(in crate::server) fn update(
        conn: &mut DbConn,
        id: ChannelId,
        changeset: ChannelChangeset,
    ) -> Result<ChannelRow, DbError> {
        diesel::update(c_dsl::channel.find(id))
            .set(changeset)
            .returning(ChannelRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(super) fn delete(conn: &mut DbConn, id: ChannelId) -> Result<(), DbError> {
        let size = diesel::delete(c_dsl::channel.find(id))
            .execute(conn)
            .map_err(DbError::from)?;

        if size == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }
}
