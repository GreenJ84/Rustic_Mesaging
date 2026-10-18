use dotenv::dotenv;
use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
pub(crate) use diesel::result::{DatabaseErrorKind, Error as DieselError};
use std::sync::Arc;

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

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

                DatabaseErrorKind::SerializationFailure => Self::RetryableTransaction(error),

                DatabaseErrorKind::UnableToSendCommand | DatabaseErrorKind::ReadOnlyTransaction => {
                    Self::Transaction(error)
                }

                DatabaseErrorKind::ClosedConnection | _ => Self::Database(error),
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

pub(crate) struct Db;
impl Db {
    pub fn establish_pool() -> Result<DbPool, DbError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Ok(Arc::new(Pool::builder().build(manager).map_err(|e| {
            log::error!("Failed to create database connection pool: {}", e);
            DbError::PoolError(format!("Failed to create pool: {}", e))
        })?))
    }
    pub fn connect(pool: &DbPool) -> Result<DbConn, DbError> {
        match pool.get() {
            Ok(conn) => Ok(conn),
            // Logged at the Application module conversion level to provide more context for debugging
            Err(e) => Err(DbError::ConnectionError(format!(
                "Failed to get database connection from pool: {}",
                e
            ))),
        }
    }
}

// ================
// Retired Db connection functions, now handled by Db struct
// ================
use rocket::http::Status;
use rocket::response::status;
use rocket::State;
pub fn establish_db_connection() -> DbPool {
    Db::establish_pool().expect("Failed to establish database connection pool")
}
pub fn get_db_connection(pool: &State<DbPool>) -> Result<DbConn, status::Custom<String>> {
    Db::connect(pool).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            String::from("Database connection error"),
        )
    })
}

use crate::api_error::ApiError;
impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::PoolError(msg) => {
                log::error!("Application state: {}", msg);
                ApiError::InternalError
            }
            _ => ApiError::InternalError,
        }
    }
}
