use crate::{
    api_error::ApiError,
    auth::system_roles::SystemRoleError,
    database::{DbError, DieselError},
};

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
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    SystemRoleError(SystemRoleError),
    DeletionBlocked(DeletionBlock),

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
            MemberError::DeletionBlocked(block) => ApiError::Conflict(block.message().to_string()),
            MemberError::PersistenceFailure => ApiError::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DeletionBlock, MemberError};
    use crate::api_error::ApiError;
    use crate::database::DbError;
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
