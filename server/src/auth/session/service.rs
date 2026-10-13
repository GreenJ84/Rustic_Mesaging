use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    AuthCredentials, AuthenticatedPrincipal, RotationResult, SessionClock, SessionConfig,
    SessionError, SessionId, SessionMetadata, SessionRepository, SessionTokenCodec,
};

use crate::db::DbPool;
use crate::id_generator::IdGenerator;
use crate::member::models::MemberId;

#[derive(Clone)]
pub(in crate::auth) struct SessionService {
    repository: SessionRepository,
    access_tokens: SessionTokenCodec,
}

impl SessionService {
    pub(in crate::auth) fn new(
        pool: DbPool,
        id_generator: IdGenerator,
        jwt_secret: &[u8],
        config: SessionConfig,
    ) -> Self {
        Self {
            repository: SessionRepository::new(pool, id_generator),
            access_tokens: SessionTokenCodec::new(jwt_secret, config),
        }
    }

    pub(in crate::auth) fn secure_cookies(&self) -> bool {
        self.access_tokens.config.cookie_secure
    }

    pub(in crate::auth) fn issue(
        &self,
        member_id: MemberId,
        metadata: SessionMetadata,
    ) -> Result<AuthCredentials, SessionError> {
        let refresh = self.access_tokens.issue_refresh();

        let session = self.repository.create(
            member_id,
            refresh.refresh_token_hash,
            refresh.refresh_expires_at,
            metadata,
        )?;

        let access_token = self.access_tokens.issue_access(member_id, session.id)?;

        Ok(AuthCredentials {
            access_token,
            refresh_token: refresh.refresh_token,
        })
    }

    pub(in crate::auth) fn rotate(
        &self,
        token: &str,
        metadata: SessionMetadata,
    ) -> Result<AuthCredentials, SessionError> {
        let token_hash = SessionTokenCodec::hash_refresh(token);
        let refresh = self.access_tokens.issue_refresh();

        match self.repository.rotate(
            token_hash,
            refresh.refresh_token_hash,
            refresh.refresh_expires_at,
            metadata,
        ) {
            Ok(rotation_result) => match rotation_result {
                RotationResult::InvalidRotation => Err(SessionError::RotationNotAllowed),
                RotationResult::Reused => Err(SessionError::RefreshTokenReused),
                RotationResult::Expired => Err(SessionError::ExpiredRefreshToken),
                RotationResult::Rotated(session) => {
                    let access_token = self
                        .access_tokens
                        .issue_access(session.member_id, session.id)?;

                    Ok(AuthCredentials {
                        access_token,
                        refresh_token: refresh.refresh_token,
                    })
                }
            },
            Err(error) => Err(SessionError::from(error)),
        }
    }

    pub(in crate::auth) fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, SessionError> {
        let principal = self.access_tokens.verify_access(token)?;
        if !self
            .repository
            .is_active(principal.session_id, principal.member_id)?
        {
            return Err(SessionError::SessionRevoked);
        }
        Ok(principal)
    }

    pub(in crate::auth) fn revoke(
        &self,
        principal: &AuthenticatedPrincipal,
    ) -> Result<(), SessionError> {
        Ok(self
            .repository
            .revoke(principal.session_id, principal.member_id)?)
    }

    pub(in crate::auth) fn revoke_all(&self, member_id: MemberId) -> Result<(), SessionError> {
        Ok(self.repository.revoke_all(member_id)?)
    }
}
