pub(crate) use diesel::result::{DatabaseErrorKind, Error as DieselError};

#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    PoolError(String),

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
    pub(crate) fn log_query_error(&self, operation: &str) {
        match self {
            Self::Conflict(error) => {
                log::warn!("Database conflict: operation={operation}, error={error:?}");
            }

            Self::ConstraintViolation(error) => {
                log::error!(
                    "Database constraint violation: operation={operation}, error={error:?}"
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
