use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub administration_key: String,
    pub cookie_secure: bool,
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        dotenv().ok();

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt_secret: required("JWT_SECRET")?,
            administration_key: required("ADMINISTRATION_KEY")?,
            cookie_secure: optional_bool("COOKIE_SECURE", !cfg!(debug_assertions))?,
            allowed_origins: optional_list(
                "ALLOWED_ORIGINS",
                vec!["http://localhost:3000".to_string()],
            ),
        })
    }
}

fn required(name: &str) -> Result<String, ConfigError> {
    let value = env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;

    if value.trim().is_empty() {
        return Err(ConfigError::Missing(name.to_string()));
    }

    Ok(value)
}

fn optional_bool(name: &str, default: bool) -> Result<bool, ConfigError> {
    match env::var(name) {
        Ok(value) => value
            .parse::<bool>()
            .map_err(|_| ConfigError::Invalid(name.to_string())),
        Err(_) => Ok(default),
    }
}

fn optional_list(name: &str, default: Vec<String>) -> Vec<String> {
    env::var(name)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .filter(|values: &Vec<String>| !values.is_empty())
        .unwrap_or(default)
}
