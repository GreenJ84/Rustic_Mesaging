use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

use crate::api::ApiError;
use crate::auth::session::AuthCredentials;
use crate::infra::DbError;

pub(super) enum RotationResult {
    Rotated(AuthCredentials),
    Invalid,
    Expired,
    Reused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionError {
    InvalidAccessToken,
    ExpiredAccessToken,

    InvalidRefreshToken,
    ExpiredRefreshToken,
    SessionRevoked,
    RefreshTokenReused,

    CodecFailure,
    PersistenceFailure,
}
impl SessionError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::InvalidAccessToken => "Invalid access token".to_string(),
            Self::ExpiredAccessToken => "Expired access token".to_string(),
            Self::InvalidRefreshToken => "Invalid refresh token".to_string(),
            Self::ExpiredRefreshToken => "Expired refresh token".to_string(),
            Self::SessionRevoked => "Session has been revoked".to_string(),
            Self::RefreshTokenReused => "Refresh token has been reused".to_string(),
            Self::CodecFailure => "Failed to encode or decode session data".to_string(),
            Self::PersistenceFailure => "Failed to persist session data".to_string(),
        }
    }
}
impl From<JwtError> for SessionError {
    fn from(e: JwtError) -> Self {
        match e.kind() {
            ErrorKind::InvalidToken
            | ErrorKind::InvalidSignature
            | ErrorKind::InvalidIssuer
            | ErrorKind::InvalidAudience
            | ErrorKind::InvalidSubject
            | ErrorKind::ImmatureSignature
            | ErrorKind::MissingRequiredClaim(_) => {
                log::error!("JWT verification error: {e:?}");
                SessionError::InvalidAccessToken
            }
            ErrorKind::ExpiredSignature => SessionError::ExpiredAccessToken,
            _ => {
                log::error!("JWT infrastructure error: {e:?}");
                SessionError::CodecFailure
            }
        }
    }
}
impl From<DbError> for SessionError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => SessionError::InvalidRefreshToken,

            DbError::Conflict(_)
            | DbError::ConstraintViolation(_)
            | DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => SessionError::PersistenceFailure,
        }
    }
}
impl From<SessionError> for ApiError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::InvalidAccessToken
            | SessionError::ExpiredAccessToken
            | SessionError::InvalidRefreshToken
            | SessionError::ExpiredRefreshToken
            | SessionError::SessionRevoked
            | SessionError::RefreshTokenReused => ApiError::Unauthorized(error.message()),

            SessionError::CodecFailure | SessionError::PersistenceFailure => {
                ApiError::InternalError
            }
        }
    }
}
