use crate::{
    api_error::ApiError,
    auth::system_roles::SystemRoleError,
    db::{DbError, DieselError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberError {
    NotFound,

    UsernameTaken,
    EmailTaken,
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    SystemRoleError(SystemRoleError),
    CannotDeleteLastAdministrator,
    DeletionBlocked,

    PersistenceFailure,
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
impl From<SystemRoleError> for MemberError {
    fn from(error: SystemRoleError) -> Self {
        Self::SystemRoleError(error)
    }
}
impl From<DieselError> for MemberError {
    fn from(error: DieselError) -> Self {
        DbError::from(error).into()
    }
}
impl From<MemberError> for ApiError {
    fn from(error: MemberError) -> Self {
        match error {
            MemberError::NotFound => ApiError::NotFound("Member not found".to_string()),
            MemberError::UsernameTaken => ApiError::Conflict("Username already taken".to_string()),
            MemberError::EmailTaken => ApiError::Conflict("Email already taken".to_string()),
            MemberError::InvalidChangeset(msg) => ApiError::BadRequest(msg),
            MemberError::InvalidSearchTerm(msg) => ApiError::BadRequest(msg),
            MemberError::SystemRoleError(e) => e.into(),
            MemberError::CannotDeleteLastAdministrator => {
                ApiError::BadRequest("Cannot delete the last administrator".to_string())
            }
            MemberError::DeletionBlocked => {
                ApiError::BadRequest("Member deletion is blocked".to_string())
            }
            MemberError::PersistenceFailure => ApiError::InternalError,
        }
    }
}
