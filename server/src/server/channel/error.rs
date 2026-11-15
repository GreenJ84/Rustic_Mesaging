use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum ChannelError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    NameTaken,

    ServerNotFound,
    MembershipRequired,
    PermissionDenied,

    PersistenceError(DbError),
}
impl From<DbError> for ChannelError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("channel_server_name_unique") => Self::NameTaken,
                    _ => Self::PersistenceError(error),
                }
            }

            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("channel_server_id_fkey") => Self::ServerNotFound,
                    _ => Self::PersistenceError(error),
                }
            }

            DbError::Conflict(_) | DbError::ConstraintViolation(_) => Self::PersistenceError(error),

            DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => Self::PersistenceError(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelError;
    use crate::api::ApiError;
    use crate::infra::DbError;
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error as DieselError};
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

    fn database_error(kind: DatabaseErrorKind, name: &'static str) -> DbError {
        let error = DieselError::DatabaseError(kind, Box::new(ConstraintInformation(name)));
        DbError::from(error)
    }

    #[test]
    fn channel_constraints_map_to_domain_errors() {
        assert!(matches!(
            ChannelError::from(database_error(
                DatabaseErrorKind::UniqueViolation,
                "channel_server_name_unique"
            )),
            ChannelError::NameTaken
        ));
        assert!(matches!(
            ChannelError::from(database_error(
                DatabaseErrorKind::ForeignKeyViolation,
                "channel_server_id_fkey"
            )),
            ChannelError::ServerNotFound
        ));
    }

    #[test]
    fn unknown_database_errors_remain_internal() {
        assert!(matches!(
            ChannelError::from(database_error(
                DatabaseErrorKind::UniqueViolation,
                "unknown_constraint"
            )),
            ChannelError::PersistenceError(_)
        ));
        assert!(matches!(
            ChannelError::from(DbError::ConnectionError("offline".into())),
            ChannelError::PersistenceError(_)
        ));
    }

    #[test]
    fn channel_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                ChannelError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                ChannelError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (ChannelError::NotFound, Status::NotFound),
            (ChannelError::NameTaken, Status::Conflict),
            (ChannelError::ServerNotFound, Status::NotFound),
            (ChannelError::MembershipRequired, Status::Forbidden),
            (ChannelError::PermissionDenied, Status::Forbidden),
            (
                ChannelError::PersistenceError(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
impl From<DieselError> for ChannelError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<ChannelError> for ApiError {
    fn from(error: ChannelError) -> Self {
        match error {
            ChannelError::InvalidInput(message) => ApiError::BadRequest(message),
            ChannelError::InvalidChangeset(message) => ApiError::BadRequest(message),

            ChannelError::NotFound => ApiError::NotFound("Channel not found".into()),
            ChannelError::NameTaken => {
                ApiError::Conflict("Channel name is already taken for the associated server".into())
            }

            ChannelError::ServerNotFound => {
                ApiError::NotFound("Associated server not found".into())
            }
            ChannelError::MembershipRequired => ApiError::Forbidden(
                "You do not have the required membership authority for this operation".into(),
            ),
            ChannelError::PermissionDenied => ApiError::Forbidden(
                "You do not have the required server permission for this operation".into(),
            ),

            ChannelError::PersistenceError(error) => {
                error.log_error("server channel");
                ApiError::InternalError
            }
        }
    }
}
