use crate::api_error::ApiError;

#[derive(Debug)]
pub enum AuthError {
    // Registration Errors
    UsernameTaken,
    EmailTaken,
    PasswordRejected(PasswordViolation),
    PasswordConfirmationMismatch,

    // Login Errors
    InvalidCredentials,
    AccountNotFound,
    AccountDisabled,

    // Token Errors
    MissingAuthorization,
    MalformedAuthorization,
    InvalidAccessToken,
    ExpiredAccessToken,
    InvalidRefreshToken,
    ExpiredRefreshToken,
    SessionRevoked,
    RefreshTokenReused,

    // Internal Errors
    PersistenceFailure,
    TokenCreationFailure,
    HashingError,
    InternalError(String),
}
impl AuthError {
    pub fn message(&self) -> String {
        match self {
            Self::UsernameTaken => "Username is already taken.".to_string(),
            Self::EmailTaken => "Email is already taken.".to_string(),
            Self::PasswordRejected(violation) => violation.message(),
            Self::PasswordConfirmationMismatch => "Password confirmation does not match.".to_string(),

            Self::AccountNotFound => "Account not found.".to_string(),
            Self::InvalidCredentials => "Invalid username or password.".to_string(),
            Self::AccountDisabled => "Account is disabled.".to_string(),

            Self::MissingAuthorization => "Missing authorization header.".to_string(),
            Self::MalformedAuthorization => "Malformed authorization header.".to_string(),
            Self::InvalidAccessToken => "Invalid access token.".to_string(),
            Self::ExpiredAccessToken => "Access token has expired.".to_string(),

            Self::InvalidRefreshToken => "Invalid refresh token.".to_string(),
            Self::ExpiredRefreshToken => "Refresh token has expired.".to_string(),
            Self::SessionRevoked => "Session has been revoked.".to_string(),
            Self::RefreshTokenReused => "Refresh token has been reused.".to_string(),

            Self::PersistenceFailure => "Persistence failure occurred".to_string(),
            Self::TokenCreationFailure => "Token creation failure occurred".to_string(),
            Self::HashingError => "Hashing error occurred".to_string(),
            Self::InternalError(msg) => format!("Internal error occurred: {}", msg),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::UsernameTaken => ApiError::Conflict(err.message()),
            AuthError::EmailTaken => ApiError::Conflict(err.message()),
            AuthError::PasswordRejected(violation) => ApiError::BadRequest(violation.message()),
            AuthError::PasswordConfirmationMismatch => ApiError::BadRequest(err.message()),

            AuthError::AccountNotFound => ApiError::NotFound(err.message()),
            AuthError::InvalidCredentials => ApiError::Unauthorized(err.message()),
            AuthError::AccountDisabled => ApiError::Forbidden(err.message()),

            AuthError::MissingAuthorization => ApiError::BadRequest(err.message()),
            AuthError::MalformedAuthorization => ApiError::BadRequest(err.message()),
            AuthError::InvalidAccessToken => ApiError::Unauthorized(err.message()),
            AuthError::ExpiredAccessToken => ApiError::Unauthorized(err.message()),

            AuthError::InvalidRefreshToken => ApiError::Unauthorized(err.message()),
            AuthError::ExpiredRefreshToken => ApiError::Unauthorized(err.message()),
            AuthError::SessionRevoked => ApiError::Unauthorized(err.message()),
            AuthError::RefreshTokenReused => ApiError::Unauthorized(err.message()),

            AuthError::PersistenceFailure => ApiError::InternalError,
            AuthError::TokenCreationFailure => ApiError::InternalError,
            AuthError::HashingError => ApiError::InternalError,
            AuthError::InternalError(_) => ApiError::InternalError,
        }
    }
}

/// Password violation rules
pub const MIN_PASSWORD_LENGTH: usize = 15;
pub const MAX_PASSWORD_LENGTH: usize = 128;
#[derive(Debug)]
pub(crate) enum PasswordViolation {
    CommonPassword,
    ContainsIdentity,
}
impl PasswordViolation {
    pub fn message(&self) -> String {
        match self {
            Self::CommonPassword => "Password is too common.".to_string(),
            Self::ContainsIdentity => {
                "Password must not contain your username or email.".to_string()
            }
        }
    }
}