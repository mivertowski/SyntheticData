//! gRPC authentication interceptor.
//!
//! Validates API keys or JWT tokens from the `authorization` metadata key.

use tonic::{Request, Status};

/// API key validator function type for gRPC.
pub type ApiKeyValidator = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// gRPC authentication interceptor configuration.
#[derive(Clone)]
pub struct GrpcAuthConfig {
    /// Whether authentication is enabled.
    pub enabled: bool,
    /// Valid API keys (plaintext for gRPC — production should use hashed).
    api_keys: Vec<String>,
}

impl GrpcAuthConfig {
    /// Create auth config with specified API keys.
    pub fn new(api_keys: Vec<String>) -> Self {
        Self {
            enabled: !api_keys.is_empty(),
            api_keys,
        }
    }

    /// Create disabled auth config.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            api_keys: Vec::new(),
        }
    }

    /// Validate a token against configured keys.
    pub fn validate_token(&self, token: &str) -> bool {
        if !self.enabled {
            return true;
        }
        self.api_keys.iter().any(|k| k == token)
    }
}

/// Intercept gRPC requests to validate authentication.
///
/// Checks for `authorization` metadata key with `Bearer <token>` format.
#[allow(clippy::result_large_err)]
pub fn auth_interceptor(
    config: &GrpcAuthConfig,
    request: &Request<()>,
) -> Result<(), Status> {
    if !config.enabled {
        return Ok(());
    }

    let token = request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match token {
        Some(t) if config.validate_token(t) => Ok(()),
        Some(_) => Err(Status::unauthenticated("Invalid credentials")),
        None => Err(Status::unauthenticated(
            "Missing authorization metadata. Provide 'authorization: Bearer <token>'",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_auth_passes() {
        let config = GrpcAuthConfig::disabled();
        let request = Request::new(());
        assert!(auth_interceptor(&config, &request).is_ok());
    }

    #[test]
    fn test_missing_token_fails() {
        let config = GrpcAuthConfig::new(vec!["secret".to_string()]);
        let request = Request::new(());
        assert!(auth_interceptor(&config, &request).is_err());
    }

    #[test]
    fn test_valid_token_passes() {
        let config = GrpcAuthConfig::new(vec!["my-key".to_string()]);
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            "Bearer my-key".parse().unwrap(),
        );
        assert!(auth_interceptor(&config, &request).is_ok());
    }

    #[test]
    fn test_invalid_token_fails() {
        let config = GrpcAuthConfig::new(vec!["my-key".to_string()]);
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            "Bearer wrong-key".parse().unwrap(),
        );
        assert!(auth_interceptor(&config, &request).is_err());
    }
}
