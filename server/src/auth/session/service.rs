use diesel::Connection;
use uuid::Uuid;

use super::{
    AuthCredentials, AuthenticatedPrincipal, RotationResult, SessionClock, SessionConfig,
    SessionError, SessionMetadata, SessionRepository, SessionTokenCodec,
};

use crate::auth::error::AuthError;
use crate::auth::session::NewAuthSession;
use crate::database::{DbConn, DbError};
use crate::member::models::MemberId;

#[derive(Clone)]
pub(in crate::auth) struct SessionService {
    access_tokens: SessionTokenCodec,
}

impl SessionService {
    pub(in crate::auth) fn new(jwt_secret: &[u8], config: SessionConfig) -> Self {
        Self {
            access_tokens: SessionTokenCodec::new(jwt_secret, config),
        }
    }

    /// Returns whether the service is configured to use secure cookies for access tokens.
    pub(in crate::auth) fn secure_cookies(&self) -> bool {
        self.access_tokens.config.cookie_secure
    }

    /// Issues a new session for a member, returning the access and refresh tokens.
    pub(in crate::auth) fn issue(
        &self,
        conn: &mut DbConn,
        new_session_id: Uuid,
        new_family_id: Uuid,
        member_id: MemberId,
        metadata: SessionMetadata,
    ) -> Result<AuthCredentials, AuthError> {
        conn.transaction::<AuthCredentials, AuthError, _>(|conn| {
            let refresh = self.access_tokens.issue_refresh();
            let new_session = NewAuthSession {
                id: new_session_id,
                family_id: new_family_id,
                member_id,
                refresh_token_hash: refresh.refresh_token_hash.clone(),
                refresh_expires_at: refresh.refresh_expires_at,
                user_agent: metadata.user_agent,
                ip_address: metadata.ip_address,
            };

            let session = SessionRepository::create(conn, new_session)?;

            let access_token = self.access_tokens.issue_access(member_id, session.id)?;
            Ok(AuthCredentials {
                access_token,
                refresh_token: refresh.refresh_token,
            })
        })
    }

    pub(in crate::auth) fn rotate(
        &self,
        conn: &mut DbConn,
        member_id: MemberId,
        refresh_token: String,
        new_session_id: Uuid,
        metadata: SessionMetadata,
    ) -> Result<AuthCredentials, AuthError> {
        let refresh = self.access_tokens.issue_refresh();
        let result = conn.transaction::<RotationResult, AuthError, _>(|conn| {
            // Lock the current session row for update to prevent concurrent rotations
            let current = SessionRepository::find_by_refresh_token_hash_for_update(
                conn,
                SessionTokenCodec::hash_refresh(&refresh_token),
            )?
            .ok_or(DbError::NotFound)?;

            // If the current session is already revoked or replaced, revoke all sessions in the same family and return the reused result. This prevents token reuse attacks.
            if current.consumed_at.is_some()
                || current.revoked_at.is_some()
                || current.replaced_by.is_some()
            {
                SessionRepository::revoke_family(conn, current.family_id)?;
                return Ok(RotationResult::Reused);
            }
            if current.member_id != member_id {
                return Ok(RotationResult::Invalid);
            }
            // If the current session is expired, prevent rotation. Access tokens live until their expiration.
            else if current.refresh_expires_at <= SessionClock::now_naive() {
                return Ok(RotationResult::Expired);
            }

            let new_session = NewAuthSession {
                id: new_session_id,
                family_id: current.family_id,
                member_id: current.member_id,
                refresh_token_hash: refresh.refresh_token_hash.clone(),
                refresh_expires_at: current.refresh_expires_at,
                user_agent: metadata.user_agent,
                ip_address: metadata.ip_address,
            };
            // Insert the replacement session
            let rotated = SessionRepository::create(conn, new_session)?;

            // Mark the current session as replaced and revoked
            SessionRepository::consume_and_replace(conn, current.id, rotated.id)?;
            let access_token = self
                .access_tokens
                .issue_access(rotated.member_id, rotated.id)?;
            Ok(RotationResult::Rotated(AuthCredentials {
                access_token,
                refresh_token: refresh.refresh_token,
            }))
        })?;

        match result {
            RotationResult::Rotated(rotated) => Ok(rotated),
            RotationResult::Invalid => Err(AuthError::from(SessionError::InvalidRefreshToken)),
            RotationResult::Expired => Err(AuthError::from(SessionError::ExpiredRefreshToken)),
            RotationResult::Reused => Err(AuthError::from(SessionError::RefreshTokenReused)),
        }
    }

    pub(in crate::auth) fn authenticate_access_token(
        &self,
        conn: &mut DbConn,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, AuthError> {
        let principal = self.access_tokens.verify_access(token)?;
        if !SessionRepository::is_active(conn, principal.session_id, principal.member_id)? {
            return Err(AuthError::from(SessionError::SessionRevoked));
        }
        Ok(principal)
    }

    pub(in crate::auth) fn logout(&self, conn: &mut DbConn, token: &str) -> Result<(), AuthError> {
        conn.transaction::<(), DbError, _>(|conn| {
            let Some(session) = SessionRepository::find_by_refresh_token_hash_for_update(
                conn,
                SessionTokenCodec::hash_refresh(token),
            )?
            else {
                return Ok(());
            };

            SessionRepository::revoke_family(conn, session.family_id)
        })
        .map_err(|e| AuthError::from(SessionError::from(e)))
    }

    pub(in crate::auth) fn logout_all(
        &self,
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<(), AuthError> {
        Ok(SessionRepository::revoke_all(conn, member_id)?)
    }
}
