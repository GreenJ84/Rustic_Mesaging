use crate::api::ApiError;
use crate::infra::{DbError, DieselError};
use crate::member::error::MemberError;

use super::membership::error::MembershipError;

/// Failures produced by server-entity validation, authorization, and persistence.
#[derive(Debug)]
pub(crate) enum ServerError {
    InvalidInput(String),
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    NotFound,
    NameTaken,

    NotOwner,
    MissingAuthority,

    Membership(MembershipError),
    Member(MemberError),
    Persistence(DbError),
}

impl From<DbError> for ServerError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("server_name_key") => Self::NameTaken,
                    Some("server_membership_pkey") => {
                        ServerError::Membership(MembershipError::AlreadyMember)
                    }
                    _ => Self::Persistence(error),
                }
            }

            DbError::ConstraintViolation(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("server_owner_id_fkey") => Self::Member(MemberError::NotFound),
                    Some("server_membership_server_id_fkey") => Self::NotFound,
                    Some("server_membership_member_id_fkey") => Self::Member(MemberError::NotFound),
                    _ => Self::Persistence(error),
                }
            }

            DbError::Conflict(_)
            | DbError::ConstraintViolation(_)
            | DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => Self::Persistence(error),
        }
    }
}
impl From<DieselError> for ServerError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<MembershipError> for ServerError {
    fn from(error: MembershipError) -> Self {
        Self::Membership(error)
    }
}

impl From<ServerError> for ApiError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::InvalidInput(message)
            | ServerError::InvalidChangeset(message)
            | ServerError::InvalidSearchTerm(message) => ApiError::BadRequest(message),

            ServerError::NotFound => ApiError::NotFound("Server not found".into()),

            ServerError::NameTaken => {
                ApiError::Conflict("A server with that name already exists".into())
            }

            ServerError::NotOwner => {
                ApiError::Forbidden("Server ownership is required for this operation".into())
            }
            ServerError::MissingAuthority => ApiError::Forbidden(
                "You do not have the required authority for this operation".into(),
            ),

            ServerError::Membership(error) => ApiError::from(error),
            ServerError::Member(error) => ApiError::from(error),
            ServerError::Persistence(error) => {
                error.log_error("server domain");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerError;
    use crate::api::ApiError;
    use crate::infra::{DbError, DieselError};
    use crate::member::error::MemberError;
    use crate::server::membership::error::MembershipError;
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind};
    use rocket::http::Status;

    struct ConstraintInformation(&'static str);

    impl DatabaseErrorInformation for ConstraintInformation {
        fn message(&self) -> &str {
            "test database error"
        }
        fn details(&self) -> Option<&str> {
            None
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            None
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

    fn conflict(name: &'static str) -> DbError {
        DbError::Conflict(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    fn constraint(name: &'static str) -> DbError {
        DbError::ConstraintViolation(DieselError::DatabaseError(
            DatabaseErrorKind::ForeignKeyViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn missing_rows_map_to_server_not_found() {
        assert!(matches!(
            ServerError::from(DieselError::NotFound),
            ServerError::NotFound
        ));
        assert!(matches!(
            ServerError::from(DbError::NotFound),
            ServerError::NotFound
        ));
    }

    #[test]
    fn unique_constraints_map_to_domain_conflicts() {
        assert!(matches!(
            ServerError::from(conflict("server_name_key")),
            ServerError::NameTaken
        ));
        assert!(matches!(
            ServerError::from(conflict("server_membership_pkey")),
            ServerError::Membership(MembershipError::AlreadyMember)
        ));
    }

    #[test]
    fn foreign_keys_map_to_missing_domain_resources() {
        assert!(matches!(
            ServerError::from(constraint("server_owner_id_fkey")),
            ServerError::Member(MemberError::NotFound)
        ));
        assert!(matches!(
            ServerError::from(constraint("server_membership_member_id_fkey")),
            ServerError::Member(MemberError::NotFound)
        ));
        assert!(matches!(
            ServerError::from(constraint("server_membership_server_id_fkey")),
            ServerError::NotFound
        ));
    }

    #[test]
    fn unknown_database_errors_remain_internal() {
        assert!(matches!(
            ServerError::from(conflict("unknown_constraint")),
            ServerError::Persistence(_)
        ));
        assert!(matches!(
            ServerError::from(DbError::ConnectionError("offline".into())),
            ServerError::Persistence(_)
        ));
    }

    #[test]
    fn server_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                ServerError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                ServerError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (
                ServerError::InvalidSearchTerm("invalid".into()),
                Status::BadRequest,
            ),
            (ServerError::NotFound, Status::NotFound),
            (ServerError::NameTaken, Status::Conflict),
            (ServerError::NotOwner, Status::Forbidden),
            (ServerError::MissingAuthority, Status::Forbidden),
            (
                ServerError::Persistence(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
