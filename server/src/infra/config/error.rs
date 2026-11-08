use std::{error::Error, fmt::Display};

/// Errors produced while loading or parsing application configuration.
#[derive(Debug)]
pub(crate) enum ConfigError {
    #[allow(dead_code)]
    /// A required configuration value is missing or blank.
    Missing(String),
    #[allow(dead_code)]
    /// A configuration value has an invalid format or unsupported value.
    Invalid(String),
    #[allow(dead_code)]
    /// The test environment points at a database that is not explicitly test-only.
    UnsafeTestDatabase,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(msg) => write!(f, "Missing configuration value: {}", msg),
            Self::Invalid(msg) => write!(f, "Invalid configuration value: {}", msg),
            Self::UnsafeTestDatabase => write!(
                f,
                "Unsafe test database: the configured database is not explicitly test-only"
            ),
        }
    }
}
impl Error for ConfigError {}
