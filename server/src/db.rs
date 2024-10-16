use std::env;
use std::sync::Arc;
use dotenv::dotenv;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use r2d2::PooledConnection;
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_db_connection() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Arc::new(Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
    )
}

pub fn get_db_connection(pool: &State<DbPool>) -> Result<DbConn, status::Custom<String>>{
    match pool.get() {
        Ok(conn) => Ok(conn),
        Err(_) => Err(status::Custom(Status::InternalServerError, String::from("Database connection error")))
    }
}