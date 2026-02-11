//! Environment variable interpolation for YAML configuration.
//!
//! Supports:
//! - `${VAR_NAME}` - substitute from environment, error if unset
//! - `${VAR_NAME:-default}` - substitute from environment, use default if unset

use regex::Regex;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvInterpolationError {
    #[error("Environment variable '{0}' is not set and no default provided")]
    MissingVariable(String),
}

/// Interpolate environment variables in a string.
///
/// Patterns:
/// - `${VAR}` - required variable (error if not set)
/// - `${VAR:-default}` - optional variable with default
///
/// # Examples
///
/// ```
/// use datasynth_config::env_interpolation::interpolate_env;
///
/// std::env::set_var("TEST_PORT", "8080");
/// let result = interpolate_env("port: ${TEST_PORT}").unwrap();
/// assert_eq!(result, "port: 8080");
///
/// let result = interpolate_env("host: ${MISSING_VAR:-localhost}").unwrap();
/// assert_eq!(result, "host: localhost");
/// std::env::remove_var("TEST_PORT");
/// ```
pub fn interpolate_env(input: &str) -> Result<String, EnvInterpolationError> {
    let re = Regex::new(r"\$\{([^}]+)\}").expect("valid env interpolation regex");
    let mut result = input.to_string();
    let mut errors = Vec::new();

    // Collect all matches first to avoid borrow issues
    let matches: Vec<(String, String)> = re
        .captures_iter(input)
        .map(|cap| {
            let full_match = cap
                .get(0)
                .expect("capture group 0 always exists")
                .as_str()
                .to_string();
            let inner = cap
                .get(1)
                .expect("capture group 1 defined in regex")
                .as_str()
                .to_string();
            (full_match, inner)
        })
        .collect();

    for (full_match, inner) in matches {
        let replacement = if let Some((var_name, default_value)) = inner.split_once(":-") {
            // Pattern: ${VAR:-default}
            match env::var(var_name) {
                Ok(val) => val,
                Err(_) => default_value.to_string(),
            }
        } else {
            // Pattern: ${VAR}
            match env::var(&inner) {
                Ok(val) => val,
                Err(_) => {
                    errors.push(inner.clone());
                    continue;
                }
            }
        };

        result = result.replace(&full_match, &replacement);
    }

    if let Some(first_error) = errors.into_iter().next() {
        return Err(EnvInterpolationError::MissingVariable(first_error));
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_substitution() {
        env::set_var("TEST_INTERP_VAR", "hello");
        let result = interpolate_env("value: ${TEST_INTERP_VAR}").unwrap();
        assert_eq!(result, "value: hello");
        env::remove_var("TEST_INTERP_VAR");
    }

    #[test]
    fn test_default_value() {
        env::remove_var("TEST_INTERP_MISSING");
        let result = interpolate_env("value: ${TEST_INTERP_MISSING:-fallback}").unwrap();
        assert_eq!(result, "value: fallback");
    }

    #[test]
    fn test_default_with_existing_var() {
        env::set_var("TEST_INTERP_EXISTS", "real_value");
        let result = interpolate_env("value: ${TEST_INTERP_EXISTS:-fallback}").unwrap();
        assert_eq!(result, "value: real_value");
        env::remove_var("TEST_INTERP_EXISTS");
    }

    #[test]
    fn test_missing_required_variable() {
        env::remove_var("TEST_INTERP_REQUIRED");
        let result = interpolate_env("value: ${TEST_INTERP_REQUIRED}");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_interpolation_needed() {
        let result = interpolate_env("plain text without variables").unwrap();
        assert_eq!(result, "plain text without variables");
    }

    #[test]
    fn test_multiple_variables() {
        env::set_var("TEST_INTERP_A", "alpha");
        env::set_var("TEST_INTERP_B", "beta");
        let result = interpolate_env("${TEST_INTERP_A} and ${TEST_INTERP_B}").unwrap();
        assert_eq!(result, "alpha and beta");
        env::remove_var("TEST_INTERP_A");
        env::remove_var("TEST_INTERP_B");
    }

    #[test]
    fn test_empty_default() {
        env::remove_var("TEST_INTERP_EMPTY_DEFAULT");
        let result = interpolate_env("value: ${TEST_INTERP_EMPTY_DEFAULT:-}").unwrap();
        assert_eq!(result, "value: ");
    }
}
