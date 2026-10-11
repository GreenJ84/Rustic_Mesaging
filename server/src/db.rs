use dotenv::dotenv;
use std::env;

use rocket::response::status;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::sync::Arc;

use crate::api_error::ApiError;

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    PoolError(String),
}
impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::ConnectionError(msg) => ApiError::InternalError,
            DbError::PoolError(msg) => {
                log::error!("Application state: {}", msg);
                ApiError::InternalError
            }
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
