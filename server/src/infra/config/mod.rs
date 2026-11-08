//! Application configuration loading and credential handling.

mod error;
mod parsing;

use blake3::Hash;
use dotenv::dotenv;
use std::env;

pub(crate) use error::ConfigError;
use parsing::{env_bool, env_list, env_str};

/// Runtime settings shared by application components.
#[derive(Clone)]
pub(crate) struct AppSettings {
    /// Database connection URL selected for the current environment.
    database_url: String,
    /// Secret used to sign and verify access tokens.
    jwt_secret: String,
    /// Whether browser cookies must be marked `Secure`.
    cookie_secure: bool,
    /// Exact browser origins permitted by the CORS policy.
    allowed_origins: Vec<String>,
}
impl AppSettings {
    /// Returns the configured database connection URL.
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Returns the JWT signing secret.
    pub(crate) fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    /// Indicates whether secure cookies are required.
    pub(crate) fn cookie_secure(&self) -> bool {
        self.cookie_secure
    }

    /// Returns the exact origins allowed by CORS.
    pub(crate) fn allowed_origins(&self) -> &[String] {
        &self.allowed_origins
    }
}

/// Hashed platform administration credential for privileged registration.
#[derive(Clone)]
pub(crate) struct AdminCredential {
    /// Blake3 digest of the platform administration key.
    administration_key: Hash,
}
impl AdminCredential {
    /// Compares a supplied administration key against its stored digest.
    pub(crate) fn matches(&self, input: &str) -> bool {
        let input = blake3::hash(input.as_bytes());
        let provided = input.as_bytes();
        let key = self.administration_key.as_bytes();
        if key.len() != provided.len() || !Self::constant_time_eq(provided, key) {
            return false;
        }
        true
    }
    /// Compares equal-length byte slices without content-based early exits.
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

/// Complete application configuration loaded from the selected environment.
pub(crate) struct AppConfig {
    /// General runtime settings.
    pub(crate) env: AppSettings,
    /// Hashed credential used for privileged registration.
    pub(crate) admin_key: AdminCredential,
}
impl AppConfig {
    /// Validates and normalizes one configured CORS origin.
    ///
    /// CORS origins must use HTTP(S), identify a host, and contain no
    /// credentials, path, query, or fragment components.
    fn validate_origin(origin: &str) -> Result<String, ConfigError> {
        let parsed = url::Url::parse(origin)
            .map_err(|_| ConfigError::Invalid(format!("Invalid CORS origin: {origin}")))?;

        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || (parsed.path() != "" && parsed.path() != "/")
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(ConfigError::Invalid(format!(
                "Invalid CORS origin: {origin}"
            )));
        }

        Ok(parsed.origin().ascii_serialization())
    }

    /// Loads and validates development, test, or production configuration.
    pub(crate) fn load() -> Result<Self, ConfigError> {
        let app_env = env_str::optional("APP_ENV", "development".to_string());

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
        if app_env == "test" && !database_url.contains("test") {
            return Err(ConfigError::UnsafeTestDatabase);
        }

        let configured_origins: Vec<String> = match app_env.as_str() {
            "production" => env_list::required("ALLOWED_ORIGINS")?,
            _ => env_list::optional("ALLOWED_ORIGINS", vec!["http://localhost:3000".to_string()]),
        };
        let allowed_origins: Vec<String> = configured_origins
            .iter()
            .map(|origin| Self::validate_origin(origin))
            .collect::<Result<_, _>>()?;
        if allowed_origins.is_empty() {
            return Err(ConfigError::Invalid(
                "ALLOWED_ORIGINS must contain at least one valid URL".to_string(),
            ));
        }

        Ok(Self {
            env: AppSettings {
                database_url,
                jwt_secret: env_str::required("JWT_SECRET")?,
                cookie_secure: match app_env.as_str() {
                    "production" => env_bool::required("COOKIE_SECURE")?,
                    _ => env_bool::optional("COOKIE_SECURE")?,
                },
                allowed_origins,
            },
            admin_key: AdminCredential {
                administration_key: blake3::hash(
                    env_str::required("ADMINISTRATION_KEY")?.as_bytes(),
                ),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AdminCredential, AppConfig, ConfigError};

    #[test]
    fn accepts_http_and_https_origins() {
        assert_eq!(
            AppConfig::validate_origin("http://localhost:3000").unwrap(),
            "http://localhost:3000"
        );
        assert_eq!(
            AppConfig::validate_origin("https://example.com/").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn rejects_non_origin_urls() {
        for origin in [
            "ftp://example.com",
            "http://",
            "http://user@example.com",
            "https://example.com/path",
            "https://example.com?query=value",
            "https://example.com#fragment",
            "not-an-origin",
        ] {
            assert!(matches!(
                AppConfig::validate_origin(origin),
                Err(ConfigError::Invalid(message)) if message.contains(origin)
            ));
        }
    }

    #[test]
    fn administration_credentials_match_only_the_original_secret() {
        let credential = AdminCredential {
            administration_key: blake3::hash(b"test-administration-key"),
        };

        assert!(credential.matches("test-administration-key"));
        assert!(!credential.matches("wrong-key"));
        assert!(!credential.matches(""));
    }

    #[test]
    fn constant_time_comparison_rejects_different_lengths() {
        assert!(AdminCredential::constant_time_eq(b"same", b"same"));
        assert!(!AdminCredential::constant_time_eq(b"same", b"different"));
    }
}
