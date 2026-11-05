mod error;
mod parsing;

use blake3::Hash;
use dotenv::dotenv;
use std::env;

use error::ConfigError;
use parsing::{env_bool, env_list, env_str};

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
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let env_file = match app_env.as_str() {
            "test" => ".env.test",
            "production" => ".env.prod",
            "development" => ".env",
            value => {
                return Err(ConfigError::Invalid(format!(
                    "Unsupported APP_ENV: {value}"
                )));
            }
        };

        dotenv::from_filename(env_file)
            .map_err(|error| ConfigError::Missing(format!("Failed to load {env_file}: {error}")))?;

        let database_url = env_str::required("DATABASE_URL")?;
        if app_env == "test" && !database_url.contains("_test") {
            return Err(ConfigError::UnsafeTestDatabase);
        }

        Ok(Self {
            env: AppSettings {
                database_url,
                jwt_secret: env_str::required("JWT_SECRET")?,
                cookie_secure: match app_env.as_str() {
                    "production" => env_bool::required("COOKIE_SECURE")?,
                    _ => env_bool::optional("COOKIE_SECURE")?,
                },
                allowed_origins: match app_env.as_str() {
                    "production" => env_list::required("ALLOWED_ORIGINS")?,
                    _ => env_list::optional(
                        "ALLOWED_ORIGINS",
                        vec!["http://localhost:3000".to_string()],
                    ),
                },
            },
            admin_key: AdminCredential {
                administration_key: blake3::hash(
                    env_str::required("ADMINISTRATION_KEY")?.as_bytes(),
                ),
            },
        })
    }
}
