use std::{error::Error, fmt::Display};

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
impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoolError(msg) => write!(f, "Database pool error: {msg}"),
            Self::ConnectionError(msg) => write!(f, "Database connection error: {msg}"),
            Self::NotFound => write!(f, "Database not found"),
            Self::Conflict(_) => write!(f, "Database conflict"),
            Self::ConstraintViolation(_) => write!(f, "Database constraint violation"),
            Self::RetryableTransaction(_) => write!(f, "Database retryable transaction"),
            Self::Serialization(_) => write!(f, "Database serialization failure"),
            Self::QueryBuilder(_) => write!(f, "Database query construction failure"),
            Self::Transaction(_) => write!(f, "Database transaction failure"),
            Self::Database(_) => write!(f, "Database operation failure"),
        }
    }
}
impl Error for DbError {}

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
        log::error!("{self:?} operation={operation}");
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
        assert!(matches!(
            DbError::from(DieselError::NotFound),
            DbError::NotFound
        ));
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
            assert!(matches!(
                DbError::from(database_error(kind)),
                DbError::ConstraintViolation(_)
            ));
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
