use std::env;
use super::error::ConfigError;

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
}
