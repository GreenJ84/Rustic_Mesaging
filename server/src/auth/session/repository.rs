use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::{Db, DbError, DbPool, DieselError};
use crate::id_generator::{self, IdGenerator};
use crate::member::models::MemberId;
use crate::schema::auth_session::dsl as sessions;

use super::SessionClock;
use super::{
    AuthSession, NewAuthSession, RotationResult, SessionFamilyId, SessionId, SessionMetadata,
};

/// Persistence boundary for refresh sessions.
///
/// The repository owns a cheap clone of the application's pool handle. It
/// checks out a connection only for the duration of an operation.
#[derive(Clone)]
pub(super) struct SessionRepository {
    pool: DbPool,
    id_generator: IdGenerator,
}
impl SessionRepository {
    pub(super) fn new(pool: DbPool, id_generator: IdGenerator) -> Self {
        Self { pool, id_generator }
    }

    pub(super) fn create(
        &self,
        member_id: MemberId,
        refresh_token_hash: String,
        refresh_expires_at: NaiveDateTime,
        metadata: SessionMetadata,
    ) -> Result<AuthSession, DbError> {
        let new_session = NewAuthSession {
            id: self.id_generator.generate_uuid(),
            family_id: self.id_generator.generate_uuid(),
            member_id,
            refresh_token_hash,
            refresh_expires_at,
            user_agent: metadata.user_agent,
            ip_address: metadata.ip_address,
        };

        let mut conn = Db::connect(&self.pool)?;
        diesel::insert_into(sessions::auth_session)
            .values(new_session)
            .returning(AuthSession::as_returning())
            .get_result(&mut conn)
            .map_err(|e| {
                let error = DbError::from(e);
                error.log_query_error("Creating new session");
                error
            })
    }

    pub(super) fn find_by_refresh_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AuthSession>, DbError> {
        let mut conn = Db::connect(&self.pool)?;

        sessions::auth_session
            .filter(sessions::refresh_token_hash.eq(token_hash))
            .select(AuthSession::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                let error = DbError::from(e);
                error.log_query_error("Finding session by refresh token hash");
                error
            })
    }

    /// Atomically consumes the current refresh-token generation and inserts
    /// its replacement. Reuse of a consumed or revoked token revokes every
    /// generation in the same family before returning an error.
    pub(super) fn rotate(
        &self,
        token_hash: String,
        replacement_hash: String,
        expires_at: NaiveDateTime,
        metadata: SessionMetadata,
    ) -> Result<RotationResult, DbError> {
        if expires_at <= SessionClock::now_naive() {
            return Ok(RotationResult::InvalidRotation);
        }
        let mut conn = Db::connect(&self.pool)?;

        let result = conn.transaction::<RotationResult, DbError, _>(|conn| {
            // Lock the current session row for update to prevent concurrent rotations
            let current = sessions::auth_session
                .filter(sessions::refresh_token_hash.eq(token_hash))
                .for_update()
                .select(AuthSession::as_select())
                .first::<AuthSession>(conn)
                .map_err(|e| {
                    let error = DbError::from(e);
                    error.log_query_error("Finding current session for rotation");
                    error
                })?;

            // If the current session is already revoked or replaced, revoke all sessions in the same family and return the reused result. This prevents token reuse attacks.
            if current.revoked_at.is_some() || current.replaced_by.is_some() {
                diesel::update(
                    sessions::auth_session
                        .filter(sessions::family_id.eq(current.family_id))
                        .filter(sessions::revoked_at.is_null()),
                )
                .set(sessions::revoked_at.eq(SessionClock::now_naive()))
                .execute(conn)
                .map_err(|e| {
                    let error = DbError::from(e);
                    error.log_query_error("Revoking all sessions in family due to token reuse");
                    error
                })?;

                return Ok(RotationResult::Reused);
            }

            let now = SessionClock::now_naive();
            // If the current session is expired, revoke it and return the expired result.
            if current.refresh_expires_at <= now {
                diesel::update(sessions::auth_session.find(current.id))
                    .set(sessions::revoked_at.eq(now))
                    .execute(conn)
                    .map_err(|e| {
                        let error = DbError::from(e);
                        error.log_query_error("Revoking expired session");
                        error
                    })?;

                return Ok(RotationResult::Expired);
            }

            // Insert the replacement session and mark the current session as replaced
            let new_session = NewAuthSession {
                id: self.id_generator.generate_uuid(),
                family_id: current.family_id,
                member_id: current.member_id,
                refresh_token_hash: replacement_hash,
                refresh_expires_at: expires_at,
                user_agent: metadata.user_agent,
                ip_address: metadata.ip_address,
            };
            let rotated = diesel::insert_into(sessions::auth_session)
                .values(new_session)
                .returning(AuthSession::as_returning())
                .get_result(conn)
                .map_err(|e| {
                    let error = DbError::from(e);
                    error.log_query_error("Inserting replacement session during rotation");
                    error
                })?;

            diesel::update(sessions::auth_session.find(current.id))
                .set((
                    sessions::replaced_by.eq(rotated.id),
                    sessions::revoked_at.eq(now),
                    sessions::consumed_at.eq(now),
                ))
                .execute(conn)
                .map_err(|e| {
                    let error = DbError::from(e);
                    error.log_query_error("Marking current session as replaced during rotation");
                    error
                })?;

            Ok(RotationResult::Rotated(rotated))
        })?;

        Ok(result)
    }

    pub(super) fn is_active(&self, session_id: SessionId, member_id: i32) -> Result<bool, DbError> {
        let mut conn = Db::connect(&self.pool)?;
        let now = SessionClock::now_naive();

        diesel::select(diesel::dsl::exists(
            sessions::auth_session
                .filter(sessions::id.eq(session_id))
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null()),
        ))
        .get_result(&mut conn)
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Checking if session is active");
            error
        })
    }

    pub(super) fn revoke(&self, session_id: SessionId, member_id: i32) -> Result<(), DbError> {
        let mut conn = Db::connect(&self.pool)?;

        diesel::update(
            sessions::auth_session
                .filter(sessions::id.eq(session_id))
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null()),
        )
        .set(sessions::revoked_at.eq(SessionClock::now_naive()))
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Revoking session");
            error
        })
    }

    pub(super) fn revoke_all(&self, member_id: i32) -> Result<(), DbError> {
        let mut conn = Db::connect(&self.pool)?;

        diesel::update(
            sessions::auth_session
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null()),
        )
        .set(sessions::revoked_at.eq(SessionClock::now_naive()))
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Revoking all sessions for member");
            error
        })
    }
}
