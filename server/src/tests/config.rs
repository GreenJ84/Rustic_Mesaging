use crate::infra::{AppConfig, ConfigError};
use crate::test_support::{EnvironmentCheckpoint, CONFIG_ENVIRONMENT};

#[test]
fn test_environment_loads_expected_configuration() {
    let environment = EnvironmentCheckpoint::capture(CONFIG_ENVIRONMENT);
    environment.set("APP_ENV", "test");
    environment.remove("DATABASE_URL");
    environment.remove("JWT_SECRET");
    environment.remove("ADMINISTRATION_KEY");
    environment.remove("COOKIE_SECURE");
    environment.remove("ALLOWED_ORIGINS");

    let config = AppConfig::load().expect(".env.test should provide valid configuration");

    assert!(config.env.database_url().contains("_test"));
    assert_eq!(config.env.jwt_secret(), "test-only-jwt-secret");
    assert!(!config.env.cookie_secure());
    assert_eq!(config.env.allowed_origins().len(), 2);
    assert!(config.admin_key.matches("test-only-administration-key"));
}

#[test]
fn unsupported_environment_is_rejected() {
    let environment = EnvironmentCheckpoint::capture(CONFIG_ENVIRONMENT);
    environment.set("APP_ENV", "invalid-environment");

    assert!(matches!(
        AppConfig::load(),
        Err(ConfigError::Invalid(message)) if message.contains("invalid-environment")
    ));
}

#[test]
fn test_environment_rejects_non_test_database() {
    let environment = EnvironmentCheckpoint::capture(CONFIG_ENVIRONMENT);
    environment.set("APP_ENV", "test");
    environment.set("DATABASE_URL", "postgres://localhost/rustic_messaging");

    assert!(matches!(
        AppConfig::load(),
        Err(ConfigError::UnsafeTestDatabase)
    ));
}
