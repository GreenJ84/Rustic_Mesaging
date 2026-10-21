use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::sync::Arc;

mod error;

pub(crate) use error::{DbError, DieselError};

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub(crate) struct Database {
    pool: DbPool,
}

impl Database {
    pub(crate) fn new(database_url: &str) -> Result<Self, DbError> {
        Ok(Self {
            pool: Self::establish_pool(database_url)?,
        })
    }

    fn establish_pool(database_url: &str) -> Result<DbPool, DbError> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Ok(Arc::new(Pool::builder().build(manager).map_err(|e| {
            log::error!("Failed to create database connection pool: {}", e);
            DbError::PoolError(format!("Failed to create pool: {}", e))
        })?))
    }

    pub(crate) fn get_connection(&self) -> Result<DbConn, DbError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            // Logged at the Application module conversion level to provide more context for debugging
            Err(e) => Err(DbError::ConnectionError(format!(
                "Failed to get database connection from pool: {}",
                e
            ))),
        }
    }
}
