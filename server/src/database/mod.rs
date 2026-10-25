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

    pub const PAGE_SIZE: i64 = 25;
    pub fn page_offset(page: u32) -> i64 {
        i64::from(page) * Self::PAGE_SIZE
    }

    pub fn escape_search_pattern(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            if matches!(character, '\\' | '%' | '_') {
                escaped.push('\\');
            }
            escaped.push(character);
        }
        escaped
    }

    pub fn normalize_search_term(term: String) -> Result<String, SearchTermError> {
        let search = term.trim().to_lowercase();
        if search.is_empty() {
            return Err(SearchTermError::Empty);
        } else if search.len() > 100 {
            return Err(SearchTermError::TooLong);
        }
        Ok(search)
    }
}

#[derive(Debug)]
pub enum SearchTermError {
    Empty,
    TooLong,
}
impl SearchTermError {
    pub fn message(&self) -> &str {
        match self {
            SearchTermError::Empty => "Search term is empty",
            SearchTermError::TooLong => "Search term too long",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::database::SearchTermError;

use super::Database;

    #[test]
    fn escapes_like_wildcards_and_escape_characters() {
        assert_eq!(
            Database::escape_search_pattern(r"50%_off\today"),
            r"50\%\_off\\today"
        );
    }

    #[test]
    fn leaves_plain_search_terms_unchanged() {
        assert_eq!(
            Database::escape_search_pattern("rust server"),
            "rust server"
        );
    }

    #[test]
    fn calculates_zero_based_page_offsets() {
        assert_eq!(Database::page_offset(0), 0);
        assert_eq!(Database::page_offset(1), 25);
        assert_eq!(Database::page_offset(3), 75);
        assert_eq!(Database::page_offset(u32::MAX), i64::from(u32::MAX) * 25);
    }

    #[test]
    fn normalizes_search_terms() {
        assert_eq!(Database::normalize_search_term("  Alice  ".to_string()).unwrap(), "Alice");
    }

    #[test]
    fn rejects_empty_search_terms() {
        assert!(matches!(
            Database::normalize_search_term(" \t\n ".to_string()),
            Err(SearchTermError::Empty)
        ));
    }

    #[test]
    fn limits_search_terms_by_character_count() {
        assert!(Database::normalize_search_term("é".repeat(125)).is_ok());
        assert!(matches!(
            Database::normalize_search_term("é".repeat(126)),
            Err(SearchTermError::TooLong)
        ));
    }
}

