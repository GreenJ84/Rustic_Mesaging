use jsonwebtoken::errors::{Error as JwtError, ErrorKind};

#[derive(Debug)]
pub(super) enum SessionError {
    InvalidAccessToken,
    ExpiredAccessToken,
    TokenCreationFailure,
    TokenVerificationFailure,

    InvalidRefreshToken,
    ExpiredRefreshToken,
    SessionRevoked,
    RefreshTokenReused,

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
            | ErrorKind::MissingRequiredClaim(_)
            | ErrorKind::InvalidAlgorithm
            | ErrorKind::MissingAlgorithm => {
                log::error!("JWT verification error: {e:?}");
                SessionError::InvalidAccessToken
            },
            ErrorKind::ExpiredSignature => SessionError::ExpiredAccessToken,
            _ => {
                log::error!("JWT infrastructure error: {e:?}");
                SessionError::TokenVerificationFailure
            }
        }
    }
}
