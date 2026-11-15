use rocket::http::Status;

use crate::api::ApiError;
use crate::auth::system_roles::SystemRoleError;
use crate::infra::{DbError, DieselError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeletionBlock {
    LastAdministrator,
    ServerOwner,
    UnderInvestigation,
}
impl DeletionBlock {
    pub(crate) fn message(&self) -> &str {
        match self {
            DeletionBlock::LastAdministrator => "Cannot delete the last administrator",
            DeletionBlock::ServerOwner => "Cannot delete the server owner",
            DeletionBlock::UnderInvestigation => "Cannot delete a member under investigation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MemberError {
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
    pub(crate) fn message(&self) -> String {
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

    pub(crate) fn status(&self) -> Status {
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
    use crate::auth::system_roles::SystemRoleError;
    use crate::infra::{DbError, DieselError};
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind};
    use rocket::http::Status;

    struct ConstraintInformation(&'static str);

    impl DatabaseErrorInformation for ConstraintInformation {
        fn message(&self) -> &str {
            "test constraint violation"
        }
        fn details(&self) -> Option<&str> {
            None
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            Some("member")
        }
        fn column_name(&self) -> Option<&str> {
            None
        }
        fn constraint_name(&self) -> Option<&str> {
            Some(self.0)
        }
        fn statement_position(&self) -> Option<i32> {
            None
        }
    }

    fn unique_constraint(name: &'static str) -> DbError {
        DbError::Conflict(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn deletion_blocks_have_stable_messages() {
        let cases = [
            (
                DeletionBlock::LastAdministrator,
                "Cannot delete the last administrator",
            ),
            (DeletionBlock::ServerOwner, "Cannot delete the server owner"),
            (
                DeletionBlock::UnderInvestigation,
                "Cannot delete a member under investigation",
            ),
        ];

        for (block, expected) in cases {
            assert_eq!(block.message(), expected);
        }
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

    #[test]
    fn member_identity_constraints_map_to_specific_conflicts() {
        assert_eq!(
            MemberError::from(unique_constraint("member_username_key")),
            MemberError::UsernameTaken
        );
        assert_eq!(
            MemberError::from(unique_constraint("member_email_key")),
            MemberError::EmailTaken
        );
    }

    #[test]
    fn unknown_constraints_and_database_failures_are_internal() {
        assert_eq!(
            MemberError::from(unique_constraint("unexpected_constraint")),
            MemberError::PersistenceFailure
        );
        assert_eq!(
            MemberError::from(DbError::ConnectionError("offline".into())),
            MemberError::PersistenceFailure
        );
    }

    #[test]
    fn system_role_member_not_found_is_normalized_to_member_not_found() {
        assert_eq!(
            MemberError::from(SystemRoleError::MemberNotFound),
            MemberError::NotFound
        );
        assert!(matches!(
            MemberError::from(SystemRoleError::ConfigurationFailure),
            MemberError::SystemRoleError(SystemRoleError::ConfigurationFailure)
        ));
    }

    #[test]
    fn member_error_statuses_cover_each_public_error_class() {
        let cases = [
            (MemberError::NotFound, Status::NotFound),
            (MemberError::UsernameTaken, Status::Conflict),
            (MemberError::EmailTaken, Status::Conflict),
            (
                MemberError::DeletionBlocked(DeletionBlock::ServerOwner),
                Status::Conflict,
            ),
            (
                MemberError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                MemberError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (
                MemberError::InvalidSearchTerm("invalid".into()),
                Status::BadRequest,
            ),
            (
                MemberError::SystemRoleError(SystemRoleError::ConfigurationFailure),
                Status::InternalServerError,
            ),
            (MemberError::PersistenceFailure, Status::InternalServerError),
        ];

        for (error, expected) in cases {
            assert_eq!(error.status(), expected);
            assert_eq!(ApiError::from(error).status(), expected);
        }
    }
}
