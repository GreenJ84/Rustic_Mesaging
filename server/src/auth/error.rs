use crate::api::ApiError;
use crate::auth::password::PasswordError;
use crate::auth::session::SessionError;
use crate::auth::system_roles::SystemRoleError;
use crate::database::{DbError, DieselError};
use rocket::http::Status;

#[derive(Debug)]
pub enum AuthError {
    // Registration Errors
    InvalidInput(String),
    UsernameTaken,
    EmailTaken,
    PasswordRejected(PasswordError),
    PasswordConfirmationMismatch,
    InvalidAdministrationKey,

    // Login Errors
    InvalidCredentials,
    AccountNotFound,
    AccountDisabled,

    // Token Errors
    MissingAuthorization,
    MalformedAuthorization,
    FailedAuthorization,
    InsufficientPrivileges,

    // Subdomain Errors
    Session(SessionError),
    SystemRole(SystemRoleError),

    // Internal Errors
    PersistenceFailure,
    InternalError(String),
}
impl AuthError {
    pub fn message(&self) -> String {
        match self {
            Self::InvalidInput(message) => message.to_string(),
            Self::UsernameTaken => "Username is already taken.".to_string(),
            Self::EmailTaken => "Email is already taken.".to_string(),
            Self::PasswordRejected(violation) => violation.message(),
            Self::PasswordConfirmationMismatch => {
                "Password confirmation does not match.".to_string()
            }
            Self::InvalidAdministrationKey => "Invalid administration key.".to_string(),

            Self::AccountNotFound => "Account not found.".to_string(),
            Self::InvalidCredentials => "Invalid username or password.".to_string(),
            Self::AccountDisabled => "Account is disabled.".to_string(),

            Self::MissingAuthorization => "Missing authorization header.".to_string(),
            Self::MalformedAuthorization => "Malformed authorization header.".to_string(),
            Self::FailedAuthorization => "Failed to authorize request.".to_string(),
            Self::InsufficientPrivileges => {
                "You do not have permission to perform this operation.".to_string()
            }

            Self::Session(session_error) => session_error.message(),
            Self::SystemRole(system_role_error) => system_role_error.message(),
            Self::PersistenceFailure => "Persistence failure occurred".to_string(),
            Self::InternalError(msg) => format!("Internal error occurred: {}", msg),
        }
    }

    pub(crate) fn status(&self) -> Status {
        match self {
            Self::InvalidInput(_) => Status::BadRequest,
            Self::UsernameTaken | Self::EmailTaken => Status::Conflict,
            Self::PasswordRejected(_) | Self::PasswordConfirmationMismatch => Status::BadRequest,
            Self::InvalidAdministrationKey => Status::Unauthorized,
            Self::InvalidCredentials
            | Self::MissingAuthorization
            | Self::MalformedAuthorization
            | Self::FailedAuthorization => Status::Unauthorized,
            Self::InsufficientPrivileges => Status::Forbidden,
            Self::AccountNotFound => Status::NotFound,
            Self::AccountDisabled => Status::Forbidden,
            Self::SystemRole(e) => ApiError::from(e.to_owned()).status(),
            Self::Session(e) => ApiError::from(e.to_owned()).status(),
            Self::PersistenceFailure | Self::InternalError(_) => Status::InternalServerError,
        }
    }
}

impl From<PasswordError> for AuthError {
    fn from(err: PasswordError) -> Self {
        match err {
            PasswordError::InvalidLength => AuthError::PasswordRejected(err),
            PasswordError::CommonPassword => AuthError::PasswordRejected(err),
            PasswordError::ContainsIdentity => AuthError::PasswordRejected(err),
            PasswordError::HashingError => AuthError::InternalError(err.message()),
            PasswordError::PasswordMismatch => AuthError::InvalidCredentials,
        }
    }
}
impl From<DbError> for AuthError {
    fn from(_error: DbError) -> Self {
        Self::PersistenceFailure
    }
}
impl From<DieselError> for AuthError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}
impl From<SessionError> for AuthError {
    fn from(error: SessionError) -> Self {
        Self::Session(error)
    }
}
impl From<SystemRoleError> for AuthError {
    fn from(error: SystemRoleError) -> Self {
        Self::SystemRole(error)
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidInput(msg) => ApiError::BadRequest(msg),
            AuthError::UsernameTaken => ApiError::Conflict(err.message()),
            AuthError::EmailTaken => ApiError::Conflict(err.message()),
            AuthError::PasswordRejected(violation) => ApiError::BadRequest(violation.message()),
            AuthError::InvalidAdministrationKey => ApiError::Unauthorized(err.message()),
            AuthError::PasswordConfirmationMismatch => ApiError::BadRequest(err.message()),

            AuthError::AccountNotFound => ApiError::NotFound(err.message()),
            AuthError::InvalidCredentials => ApiError::Unauthorized(err.message()),
            AuthError::AccountDisabled => ApiError::Forbidden(err.message()),

            AuthError::MissingAuthorization => ApiError::Unauthorized(err.message()),
            AuthError::MalformedAuthorization => ApiError::Unauthorized(err.message()),
            AuthError::FailedAuthorization => ApiError::Unauthorized(err.message()),
            AuthError::InsufficientPrivileges => ApiError::Forbidden(err.message()),

            AuthError::Session(session_error) => ApiError::from(session_error),
            AuthError::SystemRole(system_role_error) => ApiError::from(system_role_error),

            AuthError::PersistenceFailure => ApiError::InternalError,
            AuthError::InternalError(_) => ApiError::InternalError,
        }
    }
}
