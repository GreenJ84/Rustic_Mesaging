use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum PostError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,

    ServerNotFound,
    ChannelNotFound,
    MemberNotFound,
    MembershipRequired,
    PermissionDenied,

    PersistenceError(DbError),
}
impl From<DbError> for PostError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("post_channel_id_fkey") => Self::ChannelNotFound,
                    Some("post_author_id_fkey") => Self::MemberNotFound,
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
impl From<DieselError> for PostError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<PostError> for ApiError {
    fn from(error: PostError) -> Self {
        match error {
            PostError::InvalidInput(message) => ApiError::BadRequest(message),
            PostError::InvalidChangeset(message) => ApiError::BadRequest(message),

            PostError::NotFound => ApiError::NotFound("Post not found".into()),

            PostError::ServerNotFound => ApiError::NotFound("Server not found".into()),
            PostError::ChannelNotFound => ApiError::NotFound("Channel not found".into()),
            PostError::MemberNotFound => ApiError::NotFound("Member not found".into()),
            PostError::MembershipRequired => {
                ApiError::Forbidden("Membership required to perform this action".into())
            }
            PostError::PermissionDenied => {
                ApiError::Forbidden("You do not have permission to perform this action".into())
            }

            PostError::PersistenceError(error) => {
                error.log_error("server channel post");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PostError;
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

    fn constraint(name: &'static str) -> DbError {
        DbError::ConstraintViolation(DieselError::DatabaseError(
            DatabaseErrorKind::ForeignKeyViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn post_foreign_keys_map_to_missing_resources() {
        assert!(matches!(
            PostError::from(constraint("post_channel_id_fkey")),
            PostError::ChannelNotFound
        ));
        assert!(matches!(
            PostError::from(constraint("post_author_id_fkey")),
            PostError::MemberNotFound
        ));
    }

    #[test]
    fn unknown_database_errors_remain_internal() {
        assert!(matches!(
            PostError::from(constraint("unknown_constraint")),
            PostError::PersistenceError(_)
        ));
        assert!(matches!(
            PostError::from(DbError::ConnectionError("offline".into())),
            PostError::PersistenceError(_)
        ));
    }

    #[test]
    fn post_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                PostError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                PostError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (PostError::NotFound, Status::NotFound),
            (PostError::ServerNotFound, Status::NotFound),
            (PostError::ChannelNotFound, Status::NotFound),
            (PostError::MemberNotFound, Status::NotFound),
            (PostError::MembershipRequired, Status::Forbidden),
            (PostError::PermissionDenied, Status::Forbidden),
            (
                PostError::PersistenceError(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
