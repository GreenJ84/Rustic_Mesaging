use super::error::ConfigError;
use std::env;

pub(super) mod env_str {
    use super::{env, ConfigError};

    pub fn required(name: &str) -> Result<String, ConfigError> {
        let value = env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;

        if value.trim().is_empty() {
            return Err(ConfigError::Missing(name.to_string()));
        }

        Ok(value)
    }

    pub fn optional(name: &str, default: String) -> String {
        env::var(name).unwrap_or(default)
    }

    #[cfg(test)]
    mod tests {
        use super::{optional, required};
        use crate::infra::config::error::ConfigError;

        #[test]
        fn required_reports_missing_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_MISSING_STRING";
            std::env::remove_var(name);

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn required_rejects_blank_variable() {
            let name = "RUSTIC_MESSAGING_INFRA_BLANK_STRING";
            std::env::set_var(name, "   ");

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
            std::env::remove_var(name);
        }

        #[test]
        fn optional_returns_default_when_variable_is_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_STRING";
            std::env::remove_var(name);

            assert_eq!(optional(name, "fallback".to_string()), "fallback");
        }

        #[test]
        fn optional_returns_present_value() {
            let name = "RUSTIC_MESSAGING_INFRA_PRESENT_STRING";
            std::env::set_var(name, "configured");

            assert_eq!(optional(name, "fallback".to_string()), "configured");
            std::env::remove_var(name);
        }
    }
}

pub(super) mod env_bool {
    use super::{env, ConfigError};
    pub fn required(name: &str) -> Result<bool, ConfigError> {
        let value = env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;

        value
            .parse::<bool>()
            .map_err(|_| ConfigError::Invalid(name.to_string()))
    }

    pub fn optional(name: &str) -> Result<bool, ConfigError> {
        match env::var(name) {
            Ok(value) => parse(&value, name),
            Err(_) => Ok(false),
        }
    }

    fn parse(value: &str, name: &str) -> Result<bool, ConfigError> {
        value
            .parse::<bool>()
            .map_err(|_| ConfigError::Invalid(name.to_string()))
    }

    #[cfg(test)]
    mod tests {
        use super::{parse, optional, required};
        use crate::infra::config::error::ConfigError;

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
            std::env::remove_var(name);

            assert!(matches!(
                required(name),
                Err(ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn optional_defaults_to_false_when_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_BOOL";
            std::env::remove_var(name);

            assert!(!optional(name).unwrap());
        }
    }
}

pub(super) mod env_list {
    use super::{env, ConfigError};

    pub fn required(name: &str) -> Result<Vec<String>, ConfigError> {
        let value = env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))?;

        let values: Vec<String> = parse(value);
        if values.is_empty() {
            return Err(ConfigError::Missing(name.to_string()));
        }

        Ok(values)
    }

    pub fn optional(name: &str, default: Vec<String>) -> Vec<String> {
        env::var(name)
            .ok()
            .map(|value| parse(value))
            .filter(|values: &Vec<String>| !values.is_empty())
            .unwrap_or(default)
    }

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
        use super::{parse, optional, required};

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
            std::env::remove_var(name);

            assert!(matches!(
                required(name),
                Err(super::ConfigError::Missing(value)) if value == name
            ));
        }

        #[test]
        fn optional_returns_default_when_variable_is_missing() {
            let name = "RUSTIC_MESSAGING_INFRA_OPTIONAL_LIST";
            std::env::remove_var(name);

            assert_eq!(
                optional(name, vec!["default".to_string()]),
                vec!["default".to_string()]
            );
        }

        #[test]
        fn optional_returns_present_values() {
            let name = "RUSTIC_MESSAGING_INFRA_PRESENT_LIST";
            std::env::set_var(name, "value1, value2");

            assert_eq!(
                optional(name, vec!["default".to_string()]),
                vec!["value1".to_string(), "value2".to_string()]
            );
        }
    }
}
