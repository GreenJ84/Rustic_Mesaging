use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

use super::models::AuthSession;
use crate::db::DbError;

pub(super) enum RotationResult {
    InvalidRotation,
    Rotated(AuthSession),
    Expired,
    Reused,
}

#[derive(Debug)]
pub(in crate::auth) enum SessionError {
    InvalidAccessToken,
    ExpiredAccessToken,

    InvalidRefreshToken,
    ExpiredRefreshToken,
    SessionRevoked,
    RefreshTokenReused,

    RotationNotAllowed,
    CodecFailure,
    PersistenceFailure,
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
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => SessionError::PersistenceFailure,
        }
    }
}
