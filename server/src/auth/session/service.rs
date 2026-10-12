use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::{timestamp::Timestamp, ContextV7};

use crate::member::models::MemberId;
use super::{AccessTokenCodec, RefreshTokenCodec};
use super::repository::SessionRepository;
use super::{
    AuthTokens, AuthenticatedPrincipal, IssuedSessionData, SessionConfig, SessionError, SessionId,
    SessionMetadata, SessionClock, RotationResult
};

use crate::db::DbPool;
use crate::id_generator::IdGenerator;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessClaims {
    sub: String,
    sid: SessionId,
    iss: String,
    aud: String,
    iat: usize,
    exp: usize,
}

#[derive(Clone)]
pub(crate) struct SessionService {
    repository: SessionRepository,
    access_tokens: AccessTokenCodec,
}

impl SessionService {
    pub(crate) fn new(
        pool: DbPool,
        id_generator: IdGenerator,
        jwt_secret: &[u8],
        config: SessionConfig,
    ) -> Self {
        Self {
            repository: SessionRepository::new(pool, id_generator),
            access_tokens: AccessTokenCodec::new(jwt_secret, config),
        }
    }

    pub(crate) fn secure_cookies(&self) -> bool {
        self.access_tokens.config.cookie_secure
    }

    pub(crate) fn issue(
        &self,
        AuthenticatedPrincipal {
            member_id,
            session_id,
        }: &AuthenticatedPrincipal,
        metadata: SessionMetadata,
    ) -> Result<AuthTokens, SessionError> {
        let (refresh_token, refresh_token_hash) = RefreshTokenCodec::issue();
        let (access_token, access_expires_at) =
            self.access_tokens.issue(*member_id, session_id.clone())?;

        let session = self
            .repository
            .create(*member_id, refresh_token_hash, access_expires_at, metadata)?;

        Ok(AuthTokens {
            access_token,
            access_expires_at: access_expires_at.and_utc().timestamp(),
            refresh_token,
        })
    }

    pub(crate) fn rotate(
        &self,
        token: &str,
        metadata: SessionMetadata,
    ) -> Result<IssuedSessionData, SessionError> {
        let token_hash = RefreshTokenCodec::hash(token);
        let (refresh_token, refresh_token_hash) = RefreshTokenCodec::issue();
        let (now, expires_at) = SessionClock::expires_in_naive(self.access_tokens.config.refresh_ttl);

        match self.repository.rotate(token_hash, refresh_token_hash, expires_at, metadata) {
            Ok(rotation_result) => {
                match rotation_result {
                    RotationResult::InvalidRotation => Err(SessionError::RotationNotAllowed),
                    RotationResult::Reused => Err(SessionError::RefreshTokenReused),
                    RotationResult::Expired => Err(SessionError::ExpiredRefreshToken),
                    RotationResult::Rotated(session) => Ok(IssuedSessionData {
                        session_id: session.id,
                        member_id: session.member_id,
                        refresh_token,
                        expires_at,
                    }),
                }
            }
            Err(error) => Err(SessionError::from(error)),
        }
    }

    pub(crate) fn issue_access_for_rotated(
        &self,
        session: &IssuedSessionData,
    ) -> Result<(String, NaiveDateTime), SessionError> {
        self.access_tokens
            .issue(session.member_id, session.session_id)
    }

    pub(crate) fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, SessionError> {
        let principal = self.access_tokens.verify(token)?;
        if !self
            .repository
            .is_active(principal.session_id, principal.member_id)?
        {
            return Err(SessionError::SessionRevoked);
        }
        Ok(principal)
    }

    pub(crate) fn revoke(&self, principal: &AuthenticatedPrincipal) -> Result<(), SessionError> {
        Ok(self.repository
            .revoke(principal.session_id, principal.member_id)?)
    }

    pub(crate) fn revoke_all(&self, member_id: MemberId) -> Result<(), SessionError> {
        Ok(self.repository.revoke_all(member_id)?)
    }
}
