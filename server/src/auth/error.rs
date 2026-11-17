use rocket::http::Status;

use crate::api::ApiError;
use crate::infra::{DbError, DieselError};
use crate::member::error::MemberError;

use super::deletion::AccountDeletionError;
use super::password::PasswordError;
use super::session::SessionError;
use super::system_roles::SystemRoleError;

#[derive(Debug)]
pub enum AuthError {
    // Registration Errors
    Member(MemberError),
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
    AccountDeletion(AccountDeletionError),

    // Internal Errors
    PersistenceFailure,
    InternalError(String),
}
impl AuthError {
    pub fn message(&self) -> String {
        match self {
            Self::Member(error) => error.message(),
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
            Self::AccountDeletion(error) => error.message().to_string(),
            Self::PersistenceFailure => "Persistence failure occurred".to_string(),
            Self::InternalError(msg) => format!("Internal error occurred: {}", msg),
        }
    }

    pub(crate) fn status(&self) -> Status {
        match self {
            Self::Member(error) => error.status(),
            Self::PasswordRejected(_) | Self::PasswordConfirmationMismatch => Status::BadRequest,
            Self::InvalidAdministrationKey => Status::Unauthorized,
            Self::InvalidCredentials
            | Self::MissingAuthorization
            | Self::MalformedAuthorization
            | Self::FailedAuthorization => Status::Unauthorized,
            Self::InsufficientPrivileges => Status::Forbidden,
            Self::AccountNotFound => Status::NotFound,
            Self::AccountDisabled => Status::Forbidden,
            Self::SystemRole(error) => error.status(),
            Self::Session(error) => error.status(),
            Self::AccountDeletion(error) => error.status(),
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
    fn from(error: DbError) -> Self {
        match AccountDeletionError::from(error) {
            AccountDeletionError::ServerOwner => {
                Self::AccountDeletion(AccountDeletionError::ServerOwner)
            }
            AccountDeletionError::ChatOwner => {
                Self::AccountDeletion(AccountDeletionError::ChatOwner)
            }
            AccountDeletionError::Persistence(_) => Self::PersistenceFailure,
            deletion_error => Self::AccountDeletion(deletion_error),
        }
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
impl From<MemberError> for AuthError {
    fn from(error: MemberError) -> Self {
        Self::Member(error)
    }
}
impl From<AccountDeletionError> for AuthError {
    fn from(error: AccountDeletionError) -> Self {
        Self::AccountDeletion(error)
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::Member(error) => ApiError::from(error),
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
            AuthError::AccountDeletion(error) => match error {
                AccountDeletionError::Persistence(error) => {
                    error.log_error("account deletion");
                    ApiError::InternalError
                }
                error => ApiError::Conflict(error.message().to_string()),
            },

            AuthError::PersistenceFailure => ApiError::InternalError,
            AuthError::InternalError(_) => ApiError::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthError;
    use crate::api::ApiError;
    use rocket::http::Status;

    #[test]
    fn maps_authentication_failures_to_expected_statuses() {
        let cases = [
            (AuthError::InvalidCredentials, Status::Unauthorized),
            (AuthError::MissingAuthorization, Status::Unauthorized),
            (AuthError::MalformedAuthorization, Status::Unauthorized),
            (AuthError::InsufficientPrivileges, Status::Forbidden),
            (AuthError::AccountNotFound, Status::NotFound),
            (AuthError::AccountDisabled, Status::Forbidden),
            (AuthError::PersistenceFailure, Status::InternalServerError),
        ];

        for (error, status) in cases {
            assert_eq!(error.status(), status);
        }
    }

    #[test]
    fn converts_authentication_failures_to_api_errors() {
        assert!(matches!(
            ApiError::from(AuthError::InvalidCredentials),
            ApiError::Unauthorized(_)
        ));
        assert!(matches!(
            ApiError::from(AuthError::InsufficientPrivileges),
            ApiError::Forbidden(_)
        ));
        assert!(matches!(
            ApiError::from(AuthError::AccountNotFound),
            ApiError::NotFound(_)
        ));
        assert!(matches!(
            ApiError::from(AuthError::PersistenceFailure),
            ApiError::InternalError
        ));
    }

    #[test]
    fn does_not_expose_internal_error_details_in_api_conversion() {
        let error = AuthError::InternalError("database password".to_string());
        assert_eq!(error.status(), Status::InternalServerError);
        assert!(matches!(ApiError::from(error), ApiError::InternalError));
    }
}
