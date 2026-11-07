pub(crate) use diesel::result::{DatabaseErrorKind, Error as DieselError};

#[derive(Debug)]
pub enum DbError {
    #[allow(dead_code)]
    PoolError(String),
    ConnectionError(String),

    NotFound,
    Conflict(DieselError),
    ConstraintViolation(DieselError),
    RetryableTransaction(DieselError),

    Serialization(DieselError),
    QueryBuilder(DieselError),
    Transaction(DieselError),
    Database(DieselError),
}
impl From<DieselError> for DbError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound,

            DieselError::DatabaseError(kind, _) => match kind {
                DatabaseErrorKind::UniqueViolation => Self::Conflict(error),

                DatabaseErrorKind::ForeignKeyViolation
                | DatabaseErrorKind::NotNullViolation
                | DatabaseErrorKind::CheckViolation => Self::ConstraintViolation(error),

                DatabaseErrorKind::ReadOnlyTransaction => Self::Transaction(error),

                DatabaseErrorKind::SerializationFailure => Self::RetryableTransaction(error),

                DatabaseErrorKind::UnableToSendCommand
                | DatabaseErrorKind::ClosedConnection
                | _ => Self::Database(error),
            },

            DieselError::SerializationError(_) | DieselError::DeserializationError(_) => {
                Self::Serialization(error)
            }

            DieselError::QueryBuilderError(_) | DieselError::InvalidCString(_) => {
                Self::QueryBuilder(error)
            }

            DieselError::RollbackTransaction
            | DieselError::RollbackErrorOnCommit { .. }
            | DieselError::AlreadyInTransaction
            | DieselError::NotInTransaction
            | DieselError::BrokenTransactionManager => Self::Transaction(error),

            _ => Self::Database(error),
        }
    }
}
impl DbError {
    pub(crate) fn log_error(&self, operation: &str) {
        match self {
            Self::ConnectionError(msg) => {
                log::error!(
                    "{msg:?}, operation={operation}"
                );
            }
            Self::NotFound => {
                log::warn!("Database not found: operation={operation}");
            }

            Self::Conflict(error) => {
                log::warn!("Database conflict: operation={operation}, error={error:?}");
            }

            Self::ConstraintViolation(error) => {
                log::error!(
                    "Database constraint violation: operation={operation}, error={error:?}"
                );
            }

            Self::RetryableTransaction(error) => {
                log::warn!(
                    "Database retryable transaction: operation={operation}, error={error:?}"
                );
            }

            Self::Serialization(error) => {
                log::error!(
                    "Database serialization failure: operation={operation}, error={error:?}"
                );
            }

            Self::QueryBuilder(error) => {
                log::error!(
                    "Database query construction failure: operation={operation}, error={error:?}"
                );
            }

            Self::Transaction(error) => {
                log::error!("Database transaction failure: operation={operation}, error={error:?}");
            }

            Self::Database(error) => {
                log::error!("Database operation failure: operation={operation}, error={error:?}");
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use diesel::result::DatabaseErrorKind;

    use super::{DbError, DieselError};

    fn database_error(kind: DatabaseErrorKind) -> DieselError {
        DieselError::DatabaseError(kind, Box::new("test database error".to_string()))
    }

    #[test]
    fn maps_missing_rows_to_not_found() {
        assert!(matches!(DbError::from(DieselError::NotFound), DbError::NotFound));
    }

    #[test]
    fn maps_unique_violation_to_conflict() {
        assert!(matches!(
            DbError::from(database_error(DatabaseErrorKind::UniqueViolation)),
            DbError::Conflict(_)
        ));
    }

    #[test]
    fn maps_integrity_violations_to_constraint_violation() {
        for kind in [
            DatabaseErrorKind::ForeignKeyViolation,
            DatabaseErrorKind::NotNullViolation,
            DatabaseErrorKind::CheckViolation,
        ] {
            assert!(matches!(DbError::from(database_error(kind)), DbError::ConstraintViolation(_)));
        }
    }

    #[test]
    fn maps_transaction_and_retryable_errors() {
        assert!(matches!(
            DbError::from(database_error(DatabaseErrorKind::ReadOnlyTransaction)),
            DbError::Transaction(_)
        ));
        assert!(matches!(
            DbError::from(database_error(DatabaseErrorKind::SerializationFailure)),
            DbError::RetryableTransaction(_)
        ));
    }

    #[test]
    fn maps_unclassified_database_errors_to_database() {
        assert!(matches!(
            DbError::from(database_error(DatabaseErrorKind::UnableToSendCommand)),
            DbError::Database(_)
        ));
    }
}
