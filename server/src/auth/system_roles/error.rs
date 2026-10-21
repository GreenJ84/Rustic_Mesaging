use crate::api_error::ApiError;
use crate::db::{DbError, DieselError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SystemRoleError {
    InsufficientPrivileges,

    InvalidRoleName,
    InvalidAuthorityLevel(String),
    RoleAlreadyExists,
    AssignmentAlreadyExists,

    MemberNotFound,
    RoleNotFound,
    AssignmentNotFound,

    CannotModifyBuiltInRole,
    CannotDeleteBuiltInRole,
    CannotRevokeLastAdministrator,

    ConfigurationFailure,
    PersistenceFailure,
}

impl SystemRoleError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::InsufficientPrivileges => {
                "Administrator privileges are required for this operation.".to_string()
            }

            Self::InvalidRoleName => {
                "Role names must contain 3 to 32 lowercase letters, numbers, '-' or '_'."
                    .to_string()
            }
            Self::InvalidAuthorityLevel(reason) => {
                format!("Authority level is invalid: {reason}.")
            }
            Self::RoleAlreadyExists => {
                "A system role with that name or authority already exists.".to_string()
            }
            Self::AssignmentAlreadyExists => {
                "A system role assignment for this member already exists.".to_string()
            }

            Self::MemberNotFound => "The requested member does not exist.".to_string(),
            Self::RoleNotFound => "The requested system role does not exist.".to_string(),
            Self::AssignmentNotFound => {
                "The requested system role assignment does not exist.".to_string()
            }

            Self::CannotDeleteBuiltInRole => "Built-in system roles cannot be deleted.".to_string(),
            Self::CannotRevokeLastAdministrator => {
                "The final platform administrator assignment cannot be revoked.".to_string()
            }
            Self::CannotModifyBuiltInRole => {
                "Built-in system roles cannot be modified or shadowed.".to_string()
            }

            Self::ConfigurationFailure | Self::PersistenceFailure => {
                "The system role operation could not be completed.".to_string()
            }
        }
    }
}

impl From<DbError> for SystemRoleError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::RoleNotFound,
            DbError::Conflict(_)
            | DbError::ConstraintViolation(_)
            | DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => Self::PersistenceFailure,
        }
    }
}

impl From<DieselError> for SystemRoleError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<SystemRoleError> for ApiError {
    fn from(error: SystemRoleError) -> Self {
        match error {
            SystemRoleError::InvalidRoleName => ApiError::BadRequest(error.message()),
            SystemRoleError::InvalidAuthorityLevel(_) => ApiError::BadRequest(error.message()),
            SystemRoleError::InsufficientPrivileges => ApiError::Forbidden(error.message()),

            SystemRoleError::MemberNotFound
            | SystemRoleError::RoleNotFound
            | SystemRoleError::AssignmentNotFound => ApiError::NotFound(error.message()),

            SystemRoleError::RoleAlreadyExists
            | SystemRoleError::AssignmentAlreadyExists
            | SystemRoleError::CannotModifyBuiltInRole
            | SystemRoleError::CannotDeleteBuiltInRole
            | SystemRoleError::CannotRevokeLastAdministrator => ApiError::Conflict(error.message()),
            SystemRoleError::ConfigurationFailure | SystemRoleError::PersistenceFailure => {
                ApiError::InternalError
            }
        }
    }
}
