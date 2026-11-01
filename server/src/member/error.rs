use rocket::http::Status;

use crate::api::ApiError;
use crate::auth::system_roles::SystemRoleError;
use crate::infra::{DbError, DieselError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeletionBlock {
    LastAdministrator,
}
impl DeletionBlock {
    pub fn message(&self) -> &str {
        match self {
            DeletionBlock::LastAdministrator => "Cannot delete the last administrator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberError {
    NotFound,

    UsernameTaken,
    EmailTaken,
    InvalidInput(String),
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    SystemRoleError(SystemRoleError),
    DeletionBlocked(DeletionBlock),

    PersistenceFailure,
}
impl MemberError {
    pub fn message(&self) -> String {
        match self {
            Self::NotFound => "Member not found".to_string(),
            Self::UsernameTaken => "Username is already taken".to_string(),
            Self::EmailTaken => "Email is already taken".to_string(),
            Self::InvalidInput(msg) => msg.to_string(),
            Self::InvalidChangeset(msg) => msg.to_string(),
            Self::InvalidSearchTerm(msg) => msg.to_string(),
            Self::SystemRoleError(e) => e.message().to_string(),
            Self::DeletionBlocked(block) => block.message().to_string(),
            Self::PersistenceFailure => "Database error".to_string(),
        }
    }

    pub fn status(&self) -> Status {
        match self {
            MemberError::NotFound => Status::NotFound,
            MemberError::UsernameTaken
            | MemberError::EmailTaken
            | MemberError::DeletionBlocked(_) => Status::Conflict,
            MemberError::InvalidInput(_)
            | MemberError::InvalidChangeset(_)
            | MemberError::InvalidSearchTerm(_) => Status::BadRequest,
            MemberError::SystemRoleError(error) => match error {
                SystemRoleError::MemberNotFound => Status::NotFound,
                _ => Status::InternalServerError,
            },
            MemberError::PersistenceFailure => Status::InternalServerError,
        }
    }
}
impl From<DbError> for MemberError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("member_username_key") => Self::UsernameTaken,
                    Some("member_email_key") => Self::EmailTaken,
                    _ => Self::PersistenceFailure,
                }
            }

            _ => Self::PersistenceFailure,
        }
    }
}
impl From<DieselError> for MemberError {
    fn from(error: DieselError) -> Self {
        DbError::from(error).into()
    }
}
impl From<SystemRoleError> for MemberError {
    fn from(error: SystemRoleError) -> Self {
        match error {
            SystemRoleError::MemberNotFound => Self::NotFound,
            _ => Self::SystemRoleError(error),
        }
    }
}
impl From<MemberError> for ApiError {
    fn from(error: MemberError) -> Self {
        match error {
            MemberError::NotFound => ApiError::NotFound(error.message()),
            MemberError::UsernameTaken => ApiError::Conflict(error.message()),
            MemberError::EmailTaken => ApiError::Conflict(error.message()),
            MemberError::InvalidInput(msg) => ApiError::BadRequest(msg),
            MemberError::InvalidChangeset(msg) => ApiError::BadRequest(msg),
            MemberError::InvalidSearchTerm(msg) => ApiError::BadRequest(msg),
            MemberError::SystemRoleError(e) => e.into(),
            MemberError::DeletionBlocked(block) => ApiError::Conflict(block.message().to_string()),
            MemberError::PersistenceFailure => ApiError::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DeletionBlock, MemberError};
    use crate::api::ApiError;
    use crate::infra::DbError;
    use rocket::http::Status;

    #[test]
    fn deletion_blocks_have_stable_messages() {
        assert_eq!(
            DeletionBlock::LastAdministrator.message(),
            "Cannot delete the last administrator"
        );
    }

    #[test]
    fn deletion_blocks_map_to_conflict() {
        let error = ApiError::from(MemberError::DeletionBlocked(
            DeletionBlock::LastAdministrator,
        ));
        assert_eq!(error.status(), Status::Conflict);
    }

    #[test]
    fn missing_database_rows_map_to_member_not_found() {
        assert_eq!(MemberError::from(DbError::NotFound), MemberError::NotFound);
    }
}
