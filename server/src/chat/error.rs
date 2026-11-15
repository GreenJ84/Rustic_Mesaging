use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum ChatError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    MemberNotFound,

    AlreadyMember,
    NotMember,

    PersistenceError(DbError),
}

impl From<DbError> for ChatError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            // Chat currently has no unique constraint other than participant
            // membership. Treat a database unique violation without constraint
            // metadata as the same conflict instead of leaking a 500 response.
            DbError::Conflict(DieselError::DatabaseError(_, _)) => Self::AlreadyMember,
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("chat_participant_member_id_fkey") => Self::MemberNotFound,
                    Some("chat_participant_chat_id_fkey") => Self::NotFound,
                    Some("chat_name_not_blank") => {
                        Self::InvalidInput("Chat name cannot be blank".into())
                    }
                    Some("chat_name_length") => {
                        Self::InvalidInput("Chat name must be between 3 and 80 characters".into())
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
impl From<DieselError> for ChatError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<ChatError> for ApiError {
    fn from(error: ChatError) -> Self {
        match error {
            ChatError::InvalidInput(message) | ChatError::InvalidChangeset(message) => {
                ApiError::BadRequest(message)
            }
            ChatError::NotFound => ApiError::NotFound("Chat not found".into()),
            ChatError::MemberNotFound => ApiError::NotFound("Member not found".into()),

            ChatError::AlreadyMember => ApiError::Conflict("Member is already in this chat".into()),
            ChatError::NotMember => ApiError::NotFound("Chat membership not found".into()),
            ChatError::PersistenceError(error) => {
                error.log_error("chat domain");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChatError;
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

    fn conflict(name: &'static str) -> DbError {
        DbError::Conflict(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    fn constraint(name: &'static str) -> DbError {
        DbError::ConstraintViolation(DieselError::DatabaseError(
            DatabaseErrorKind::CheckViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn participant_primary_key_maps_to_already_member() {
        assert!(matches!(
            ChatError::from(conflict("chat_participant_pkey")),
            ChatError::AlreadyMember
        ));
    }

    #[test]
    fn participant_foreign_keys_map_to_missing_resources() {
        assert!(matches!(
            ChatError::from(constraint("chat_participant_member_id_fkey")),
            ChatError::MemberNotFound
        ));
        assert!(matches!(
            ChatError::from(constraint("chat_participant_chat_id_fkey")),
            ChatError::NotFound
        ));
    }

    #[test]
    fn chat_checks_map_to_invalid_input() {
        for name in ["chat_name_not_blank", "chat_name_length"] {
            assert!(matches!(
                ChatError::from(constraint(name)),
                ChatError::InvalidInput(_)
            ));
        }
    }

    #[test]
    fn unrecognized_unique_conflicts_remain_membership_conflicts() {
        assert!(matches!(
            ChatError::from(conflict("unknown_constraint")),
            ChatError::AlreadyMember
        ));
    }

    #[test]
    fn non_constraint_database_errors_remain_internal() {
        assert!(matches!(
            ChatError::from(DbError::ConnectionError("offline".into())),
            ChatError::PersistenceError(_)
        ));
    }

    #[test]
    fn chat_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                ChatError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (
                ChatError::InvalidChangeset("invalid".into()),
                Status::BadRequest,
            ),
            (ChatError::NotFound, Status::NotFound),
            (ChatError::MemberNotFound, Status::NotFound),
            (ChatError::AlreadyMember, Status::Conflict),
            (ChatError::NotMember, Status::NotFound),
            (
                ChatError::PersistenceError(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
