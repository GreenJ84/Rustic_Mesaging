use super::error::ConfigError;
use std::env;
use std::env::VarError;

fn get_env_var(name: &str) -> Result<String, VarError> {
    env::var(name).map(|value| value.trim().to_string())
}
/// Reads required and optional string environment variables.
pub(super) mod env_str {

    use super::{get_env_var, ConfigError};

    /// Reads a non-empty environment variable.
    pub fn required(name: &str) -> Result<String, ConfigError> {
        let value = get_env_var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;
        if value.is_empty() {
            return Err(ConfigError::Missing(name.to_string()));
        }
        Ok(value)
    }

    /// Reads an environment variable or returns the supplied default.
    pub fn optional(name: &str, default: String) -> String {
        get_env_var(name).unwrap_or(default)
    }

    #[cfg(test)]
    mod tests {
        use super::{optional, required};
        use crate::infra::config::error::ConfigError;
        use crate::test_support::EnvironmentCheckpoint;

        #[test]
        fn required_reports_missing_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_MISSING_STRING";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn required_rejects_blank_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_BLANK_STRING";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.set(name, "   ");

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn optional_returns_default_when_variable_is_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_STRING";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert_eq!(optional(name, "fallback".to_string()), "fallback");
        }

        #[test]
        fn optional_returns_present_value() {
            let name = "RUSTIC_MESSAGING_INFRA_PRESENT_STRING";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.set(name, "configured");

            assert_eq!(optional(name, "fallback".to_string()), "configured");
        }
    }
}

/// Reads required and optional boolean environment variables.
pub(super) mod env_bool {
    use super::{get_env_var, ConfigError};
    /// Reads and parses a required boolean environment variable.
    pub fn required(name: &str) -> Result<bool, ConfigError> {
        let value = get_env_var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;

        value
            .parse::<bool>()
            .map_err(|_| ConfigError::Invalid(name.to_string()))
    }

    /// Reads an optional boolean, defaulting to `false` when absent.
    pub fn optional(name: &str) -> Result<bool, ConfigError> {
        match get_env_var(name) {
            Ok(value) => parse(&value, name),
            Err(_) => Ok(false),
        }
    }

    /// Parses a boolean value and reports the variable name on failure.
    fn parse(value: &str, name: &str) -> Result<bool, ConfigError> {
        value
            .to_lowercase()
            .parse::<bool>()
            .map_err(|_| ConfigError::Invalid(name.to_string()))
    }

    #[cfg(test)]
    mod tests {
        use super::{optional, parse, required};
        use crate::infra::config::error::ConfigError;
        use crate::test_support::EnvironmentCheckpoint;

        #[test]
        fn parses_boolean_values() {
            assert_eq!(parse("true", "COOKIE_SECURE").unwrap(), true);
            assert_eq!(parse("false", "COOKIE_SECURE").unwrap(), false);
        }

        #[test]
        fn rejects_invalid_boolean_values() {
            assert!(matches!(
                parse("yes", "COOKIE_SECURE"),
                Err(ConfigError::Invalid(name)) if name == "COOKIE_SECURE"
            ));
            assert!(matches!(
                parse("no", "COOKIE_SECURE"),
                Err(ConfigError::Invalid(name)) if name == "COOKIE_SECURE"
            ));
        }

        #[test]
        fn required_reports_missing_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_REQUIRED_BOOL";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn optional_defaults_to_false_when_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_BOOL";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert!(!optional(name).unwrap());
        }
    }
}

/// Reads comma-separated required and optional string lists.
pub(super) mod env_list {
    use super::{get_env_var, ConfigError};

    /// Reads and parses a non-empty comma-separated list.
    pub fn required(name: &str) -> Result<Vec<String>, ConfigError> {
        let value = get_env_var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;
        let values: Vec<String> = parse(value);
        if values.is_empty() {
            return Err(ConfigError::Invalid(name.to_string()));
        }
        Ok(values)
    }

    /// Reads a comma-separated list or returns the default when absent or blank.
    pub fn optional(name: &str, default: Vec<String>) -> Vec<String> {
        get_env_var(name)
            .ok()
            .map(|value| parse(value))
            .filter(|values: &Vec<String>| !values.is_empty())
            .unwrap_or(default)
    }

    /// Splits, trims, and removes empty entries from a list value.
    fn parse(value: String) -> Vec<String> {
        value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::{optional, parse, required};
        use crate::test_support::EnvironmentCheckpoint;

        #[test]
        fn parses_and_trims_comma_separated_values() {
            assert_eq!(
                parse(" http://localhost:3000, https://example.com ,, ".to_string()),
                vec!["http://localhost:3000", "https://example.com"]
            );
        }

        #[test]
        fn empty_values_are_removed() {
            assert!(parse(" , , ".to_string()).is_empty());
        }

        #[test]
        fn parses_a_single_value() {
            assert_eq!(
                parse("https://example.com".to_string()),
                vec!["https://example.com"]
            );
        }

        #[test]
        fn required_reports_missing_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_REQUIRED_LIST";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert!(matches!(
                required(name),
                Err(super::ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn optional_returns_default_when_variable_is_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_LIST";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.remove(name);

            assert_eq!(
                optional(name, vec!["default".to_string()]),
                vec!["default".to_string()]
            );
        }

        #[test]
        fn optional_returns_present_values() {
            let name = "RUSTIC_MESSAGING_INFRA_PRESENT_LIST";
            let environment = EnvironmentCheckpoint::capture(&[name]);
            environment.set(name, "value1, value2");

            assert_eq!(
                optional(name, vec!["default".to_string()]),
                vec!["value1".to_string(), "value2".to_string()]
            );
        }
    }
}
