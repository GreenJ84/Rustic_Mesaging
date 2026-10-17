use diesel::prelude::*;

use crate::db::{DbConn, DbError};
use crate::member::models::MemberId;
use crate::schema::auth_session::dsl as sessions;

use super::SessionClock;
use super::{AuthSession, NewAuthSession, SessionFamilyId, SessionId};

/// Persistence boundary for refresh sessions.
///
/// Every operation uses a caller-provided connection so the owning service can
/// define the transaction boundary.
#[derive(Clone)]
pub(super) struct SessionRepository;
impl SessionRepository {
    // Create ======================================
    /// Inserts a new session into the database.
    pub(super) fn create(
        conn: &mut DbConn,
        new_session: NewAuthSession,
    ) -> Result<AuthSession, DbError> {
        diesel::insert_into(sessions::auth_session)
            .values(new_session)
            .returning(AuthSession::as_returning())
            .get_result(conn)
            .map_err(|e| {
                let error = DbError::from(e);
                error.log_query_error("Creating new session");
                error
            })
    }

    // Read ======================================
    /// Finds a session by its refresh token hash, locking it for update.
    pub(super) fn find_by_refresh_token_hash_for_update(
        conn: &mut DbConn,
        token_hash: String,
    ) -> Result<Option<AuthSession>, DbError> {
        sessions::auth_session
            .filter(sessions::refresh_token_hash.eq(token_hash))
            .for_update()
            .select(AuthSession::as_select())
            .first(conn)
            .optional()
            .map_err(|e| {
                let error = DbError::from(e);
                error.log_query_error("Locking session for refresh rotation");
                error
            })
    }

    /// Checks if a session is active (not revoked) for a given member.
    pub(super) fn is_active(
        conn: &mut DbConn,
        session_id: SessionId,
        member_id: i32,
    ) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(
            sessions::auth_session
                .filter(sessions::id.eq(session_id))
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null())
        ))
        .get_result(conn)
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Checking if session is active");
            error
        })
    }

    /// Atomically consumes the current refresh-token generation.
    /// Reuse of a consumed or revoked token revokes every
    /// generation in the same family before returning an error.
    pub(super) fn consume_and_replace(
        conn: &mut DbConn,
        old_id: SessionId,
        new_id: SessionId,
    ) -> Result<(), DbError> {
        let now = SessionClock::now_naive();
        // Mark the current session as replaced and revoked
        let updated = diesel::update(
            sessions::auth_session
                .find(old_id)
                .filter(sessions::consumed_at.is_null())
                .filter(sessions::revoked_at.is_null())
                .filter(sessions::replaced_by.is_null()),
        )
        .set((
            sessions::consumed_at.eq(now),
            sessions::revoked_at.eq(now),
            sessions::replaced_by.eq(new_id),
        ))
        .execute(conn)
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Marking current session as replaced during rotation");
            error
        })?;

        if updated == 1 {
            Ok(())
        } else {
            Err(DbError::NotFound)
        }
    }

    /// Revokes a specific member's session by its ID.
    pub(super) fn revoke_session(
        conn: &mut DbConn,
        session_id: SessionId,
        member_id: i32,
    ) -> Result<(), DbError> {
        diesel::update(
            sessions::auth_session
                .filter(sessions::id.eq(session_id))
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null()),
        )
        .set(sessions::revoked_at.eq(SessionClock::now_naive()))
        .execute(conn)
        .map(|_| ())
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Revoking session");
            error
        })
    }

    /// Revokes all sessions in a family, typically used when a refresh token is reused.
    pub(super) fn revoke_family(
        conn: &mut DbConn,
        family_id: SessionFamilyId,
    ) -> Result<(), DbError> {
        diesel::update(
            sessions::auth_session
                .filter(sessions::family_id.eq(family_id))
                .filter(sessions::revoked_at.is_null()),
        )
        .set(sessions::revoked_at.eq(SessionClock::now_naive()))
        .execute(conn)
        .map(|_| ())
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Revoking session");
            error
        })
    }

    /// Revokes all sessions for a specific member, typically used when a password is changed or an all device logout is requested.
    pub(super) fn revoke_all(conn: &mut DbConn, member_id: i32) -> Result<(), DbError> {
        diesel::update(
            sessions::auth_session
                .filter(sessions::member_id.eq(member_id))
                .filter(sessions::revoked_at.is_null()),
        )
        .set(sessions::revoked_at.eq(SessionClock::now_naive()))
        .execute(conn)
        .map(|_| ())
        .map_err(|e| {
            let error = DbError::from(e);
            error.log_query_error("Revoking all sessions for member");
            error
        })
    }
}
