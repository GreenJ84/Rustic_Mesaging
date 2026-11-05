#[derive(Debug)]
pub(crate) enum ConfigError {
    #[allow(dead_code)]
    Missing(String),
    #[allow(dead_code)]
    Invalid(String),
    #[allow(dead_code)]
    UnsafeTestDatabase,
}