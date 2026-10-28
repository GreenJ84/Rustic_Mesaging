use blake3::Hash;
use dotenv::dotenv;
use std::env;

#[derive(Debug)]
pub(crate) enum ConfigError {
    #[allow(dead_code)]
    Missing(String),
    #[allow(dead_code)]
    Invalid(String),
}

#[derive(Clone)]
pub(crate) struct AppSettings {
    database_url: String,
    jwt_secret: String,
    cookie_secure: bool,
    allowed_origins: Vec<String>,
}
impl AppSettings {
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub(crate) fn cookie_secure(&self) -> bool {
        self.cookie_secure
    }

    pub(crate) fn allowed_origins(&self) -> &[String] {
        &self.allowed_origins
    }
}

#[derive(Clone)]
pub(crate) struct AdminCredential {
    administration_key: Hash,
}
impl AdminCredential {
    pub(crate) fn matches(&self, input: &str) -> bool {
        let input = blake3::hash(input.as_bytes());
        let provided = input.as_bytes();
        let key = self.administration_key.as_bytes();
        if key.len() != provided.len() || !Self::constant_time_eq(provided, key) {
            return false;
        }
        true
    }
    fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
        if left.len() != right.len() {
            return false;
        }
        left.iter()
            .zip(right)
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            == 0
    }
}

pub(crate) struct AppConfig {
    pub(crate) env: AppSettings,
    pub(crate) admin_key: AdminCredential,
}
impl AppConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        dotenv().ok();

        Ok(Self {
            env: AppSettings {
                database_url: required("DATABASE_URL")?,
                jwt_secret: required("JWT_SECRET")?,
                cookie_secure: optional_bool("COOKIE_SECURE", !cfg!(debug_assertions))?,
                allowed_origins: optional_list(
                    "ALLOWED_ORIGINS",
                    vec!["http://localhost:3000".to_string()],
                ),
            },
            admin_key: AdminCredential {
                administration_key: blake3::hash(required("ADMINISTRATION_KEY")?.as_bytes()),
            },
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
