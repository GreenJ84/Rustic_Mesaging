use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum MessageError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    ChatNotFound,

    MembershipRequired,
    PermissionDenied,
    SenderNotFound,

    PersistenceError(DbError),
}

impl From<DbError> for MessageError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("message_chat_id_fkey") => Self::ChatNotFound,
                    Some("message_sender_id_fkey") => Self::SenderNotFound,
                    Some("message_content_not_blank") => {
                        Self::InvalidInput("Message content cannot be blank".into())
                    }
                    Some("message_content_length") => {
                        Self::InvalidInput("Message content must be at most 2000 characters".into())
                    }
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

impl From<DieselError> for MessageError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<MessageError> for ApiError {
    fn from(error: MessageError) -> Self {
        match error {
            MessageError::InvalidInput(message) | MessageError::InvalidChangeset(message) => {
                ApiError::BadRequest(message)
            }
            MessageError::NotFound => ApiError::NotFound("Message not found".into()),
            MessageError::ChatNotFound => ApiError::NotFound("Chat not found".into()),
            MessageError::SenderNotFound => ApiError::NotFound("Member not found".into()),
            MessageError::MembershipRequired | MessageError::PermissionDenied => {
                ApiError::Forbidden("You cannot perform this message action".into())
            }
            MessageError::PersistenceError(error) => {
                error.log_error("chat message domain");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MessageError;
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
            DatabaseErrorKind::CheckViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn message_foreign_keys_map_to_missing_resources() {
        assert!(matches!(
            MessageError::from(constraint("message_chat_id_fkey")),
            MessageError::ChatNotFound
        ));
        assert!(matches!(
            MessageError::from(constraint("message_sender_id_fkey")),
            MessageError::SenderNotFound
        ));
    }

    #[test]
    fn message_checks_map_to_invalid_input() {
        for name in ["message_content_not_blank", "message_content_length"] {
            assert!(matches!(
                MessageError::from(constraint(name)),
                MessageError::InvalidInput(_)
            ));
        }
    }

    #[test]
    fn unknown_database_errors_remain_internal() {
        assert!(matches!(
            MessageError::from(constraint("unknown_constraint")),
            MessageError::PersistenceError(_)
        ));
        assert!(matches!(
            MessageError::from(DbError::ConnectionError("offline".into())),
            MessageError::PersistenceError(_)
        ));
    }

    #[test]
    fn message_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                MessageError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                MessageError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (MessageError::NotFound, Status::NotFound),
            (MessageError::ChatNotFound, Status::NotFound),
            (MessageError::SenderNotFound, Status::NotFound),
            (MessageError::MembershipRequired, Status::Forbidden),
            (MessageError::PermissionDenied, Status::Forbidden),
            (
                MessageError::PersistenceError(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
