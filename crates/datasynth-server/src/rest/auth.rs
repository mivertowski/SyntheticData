//! Authentication middleware for REST API.
//!
//! Provides API key authentication with Argon2id hashing and
//! timing-safe comparison for protecting endpoints.
//!
//! When the `jwt` feature is enabled, also supports JWT validation
//! from external OIDC providers (Keycloak, Auth0, Entra ID).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ===========================================================================
// JWT types (feature-gated)
// ===========================================================================

/// JWT validation configuration for OIDC providers.
#[cfg(feature = "jwt")]
#[derive(Clone, Debug)]
pub struct JwtConfig {
    /// Expected token issuer (e.g., "https://auth.example.com/realms/main").
    pub issuer: String,
    /// Expected audience claim.
    pub audience: String,
    /// PEM-encoded public key for RS256 verification.
    pub public_key_pem: Option<String>,
    /// Allowed algorithms (default: RS256).
    pub allowed_algorithms: Vec<jsonwebtoken::Algorithm>,
}

#[cfg(feature = "jwt")]
impl JwtConfig {
    /// Create a new JWT config with RS256 algorithm.
    pub fn new(issuer: String, audience: String) -> Self {
        Self {
            issuer,
            audience,
            public_key_pem: None,
            allowed_algorithms: vec![jsonwebtoken::Algorithm::RS256],
        }
    }

    /// Set the PEM public key.
    pub fn with_public_key(mut self, pem: String) -> Self {
        self.public_key_pem = Some(pem);
        self
    }
}

/// Claims extracted from a validated JWT.
#[cfg(feature = "jwt")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID).
    pub sub: String,
    /// Email address (optional).
    #[serde(default)]
    pub email: Option<String>,
    /// Roles assigned to the user.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Tenant ID for multi-tenancy (optional).
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Expiration timestamp.
    pub exp: usize,
    /// Issuer.
    pub iss: String,
    /// Audience (can be string or array).
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
}

/// JWT validator that verifies tokens from external OIDC providers.
#[cfg(feature = "jwt")]
#[derive(Clone)]
pub struct JwtValidator {
    config: JwtConfig,
    decoding_key: Option<jsonwebtoken::DecodingKey>,
}

#[cfg(feature = "jwt")]
impl std::fmt::Debug for JwtValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtValidator")
            .field("config", &self.config)
            .field(
                "decoding_key",
                &self.decoding_key.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
}

#[cfg(feature = "jwt")]
impl JwtValidator {
    /// Create a new JWT validator.
    pub fn new(config: JwtConfig) -> Result<Self, String> {
        let decoding_key = if let Some(ref pem) = config.public_key_pem {
            Some(
                jsonwebtoken::DecodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(|e| format!("Invalid RSA PEM key: {}", e))?,
            )
        } else {
            None
        };

        Ok(Self {
            config,
            decoding_key,
        })
    }

    /// Validate a JWT token and extract claims.
    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, String> {
        let decoding_key = self
            .decoding_key
            .as_ref()
            .ok_or_else(|| "No decoding key configured".to_string())?;

        let mut validation = jsonwebtoken::Validation::new(
            *self
                .config
                .allowed_algorithms
                .first()
                .unwrap_or(&jsonwebtoken::Algorithm::RS256),
        );
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = true;

        let token_data = jsonwebtoken::decode::<TokenClaims>(token, decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {}", e))?;

        Ok(token_data.claims)
    }
}

// ===========================================================================
// Authentication configuration
// ===========================================================================

/// Authentication configuration.
#[derive(Clone, Debug)]
pub struct AuthConfig {
    /// Whether authentication is enabled.
    pub enabled: bool,
    /// Argon2id hashed API keys (PHC format strings).
    hashed_keys: Vec<String>,
    /// Paths that don't require authentication (e.g., health checks).
    pub exempt_paths: HashSet<String>,
    /// LRU cache for recently verified keys (fast hash -> expiry).
    cache: Arc<Mutex<Vec<CacheEntry>>>,
    /// JWT validator (only available with `jwt` feature).
    #[cfg(feature = "jwt")]
    pub jwt_validator: Option<JwtValidator>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    /// Fast hash of the submitted key (not the key itself).
    key_hash: u64,
    /// When this cache entry expires.
    expires_at: Instant,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            hashed_keys: Vec::new(),
            exempt_paths: HashSet::from([
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/metrics".to_string(),
            ]),
            cache: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "jwt")]
            jwt_validator: None,
        }
    }
}

impl AuthConfig {
    /// Create a new auth config with API key authentication enabled.
    ///
    /// Keys are hashed with Argon2id at construction time.
    pub fn with_api_keys(api_keys: Vec<String>) -> Self {
        let argon2 = Argon2::default();
        let hashed_keys: Vec<String> = api_keys
            .iter()
            .map(|key| {
                let salt = SaltString::generate(&mut OsRng);
                argon2
                    .hash_password(key.as_bytes(), &salt)
                    .expect("Argon2id hashing should not fail")
                    .to_string()
            })
            .collect();

        Self {
            enabled: true,
            hashed_keys,
            exempt_paths: HashSet::from([
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/metrics".to_string(),
            ]),
            cache: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "jwt")]
            jwt_validator: None,
        }
    }

    /// Create a new auth config with pre-hashed keys (PHC format).
    ///
    /// Use this when keys are already hashed (e.g., loaded from config).
    pub fn with_prehashed_keys(hashed_keys: Vec<String>) -> Self {
        Self {
            enabled: true,
            hashed_keys,
            exempt_paths: HashSet::from([
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/metrics".to_string(),
            ]),
            cache: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "jwt")]
            jwt_validator: None,
        }
    }

    /// Add JWT validation support.
    #[cfg(feature = "jwt")]
    pub fn with_jwt(mut self, config: JwtConfig) -> Result<Self, String> {
        let validator = JwtValidator::new(config)?;
        self.jwt_validator = Some(validator);
        self.enabled = true;
        Ok(self)
    }

    /// Add exempt paths that don't require authentication.
    pub fn with_exempt_paths(mut self, paths: Vec<String>) -> Self {
        for path in paths {
            self.exempt_paths.insert(path);
        }
        self
    }

    /// Verify an API key against all stored hashes.
    ///
    /// Iterates ALL hashes to prevent timing side-channels on which
    /// key matched or how many keys exist.
    fn verify_key(&self, submitted_key: &str) -> bool {
        let key_hash = fast_hash(submitted_key);

        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            let now = Instant::now();
            for entry in cache.iter() {
                if entry.key_hash == key_hash && entry.expires_at > now {
                    return true;
                }
            }
        }

        // Verify against all hashed keys (no short-circuit)
        let argon2 = Argon2::default();
        let mut any_match = false;

        for stored_hash in &self.hashed_keys {
            if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
                if argon2
                    .verify_password(submitted_key.as_bytes(), &parsed_hash)
                    .is_ok()
                {
                    any_match = true;
                }
            }
        }

        // Cache on success
        if any_match {
            let mut cache = self.cache.lock().unwrap();
            // Evict expired entries
            let now = Instant::now();
            cache.retain(|e| e.expires_at > now);
            // Add new entry with 5s TTL
            cache.push(CacheEntry {
                key_hash,
                expires_at: now + Duration::from_secs(5),
            });
        }

        any_match
    }

    /// Try to validate a Bearer token as JWT first, then fall back to API key.
    fn verify_bearer(&self, token: &str) -> AuthResult {
        // Try JWT first (if feature enabled and configured)
        #[cfg(feature = "jwt")]
        if let Some(ref validator) = self.jwt_validator {
            match validator.validate_token(token) {
                Ok(_claims) => return AuthResult::Authenticated,
                Err(_) => {
                    // JWT validation failed — fall through to API key check
                }
            }
        }

        // Fall back to API key verification
        if self.verify_key(token) {
            AuthResult::Authenticated
        } else {
            AuthResult::InvalidCredentials
        }
    }
}

/// Result of an authentication attempt.
enum AuthResult {
    Authenticated,
    InvalidCredentials,
}

/// Fast non-cryptographic hash for cache key lookup.
fn fast_hash(s: &str) -> u64 {
    // FNV-1a hash
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Authentication middleware that checks for valid API key or JWT.
///
/// Checks for credentials in:
/// 1. `Authorization: Bearer <key_or_jwt>` header
/// 2. `X-API-Key: <key>` header
pub async fn auth_middleware(
    axum::Extension(config): axum::Extension<AuthConfig>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip if auth is disabled
    if !config.enabled {
        return next.run(request).await;
    }

    // Check if path is exempt
    let path = request.uri().path();
    if config.exempt_paths.contains(path) {
        return next.run(request).await;
    }

    // Extract credential from headers
    let bearer_token = extract_bearer_token(&request);
    let api_key = extract_x_api_key(&request);

    // Try Bearer token first (supports both JWT and API key)
    if let Some(token) = bearer_token {
        return match config.verify_bearer(&token) {
            AuthResult::Authenticated => next.run(request).await,
            AuthResult::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Invalid credentials",
            )
                .into_response(),
        };
    }

    // Try X-API-Key header
    if let Some(key) = api_key {
        if config.verify_key(&key) {
            return next.run(request).await;
        }
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "Invalid API key",
        )
            .into_response();
    }

    // No credentials provided
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        "API key required. Provide via 'Authorization: Bearer <key>' or 'X-API-Key' header",
    )
        .into_response()
}

/// Extract Bearer token from Authorization header.
fn extract_bearer_token(request: &Request<Body>) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Extract API key from X-API-Key header.
fn extract_x_api_key(request: &Request<Body>) -> Option<String> {
    request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    fn test_router(config: AuthConfig) -> Router {
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(config))
    }

    #[tokio::test]
    async fn test_auth_disabled() {
        let config = AuthConfig::default();
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_bearer_token() {
        let config = AuthConfig::with_api_keys(vec!["test-key-123".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer test-key-123")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_x_api_key() {
        let config = AuthConfig::with_api_keys(vec!["test-key-456".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("X-API-Key", "test-key-456")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer wrong-key")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_exempt_path() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_prehashed_keys() {
        // Hash a key manually
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(b"pre-hashed-key", &salt)
            .unwrap()
            .to_string();

        let config = AuthConfig::with_prehashed_keys(vec![hash]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer pre-hashed-key")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let config = AuthConfig::with_api_keys(vec!["cached-key".to_string()]);

        // First request - populates cache
        let router1 = test_router(config.clone());
        let request1 = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer cached-key")
            .body(Body::empty())
            .unwrap();
        let response1 = router1.oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        // Second request - should hit cache
        let router2 = test_router(config);
        let request2 = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer cached-key")
            .body(Body::empty())
            .unwrap();
        let response2 = router2.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_key_fallback_still_works() {
        // Even without JWT feature, API key auth should work
        let config = AuthConfig::with_api_keys(vec!["my-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer my-key")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[cfg(feature = "jwt")]
    mod jwt_tests {
        use super::*;

        #[test]
        fn test_jwt_config_creation() {
            let config =
                JwtConfig::new("https://auth.example.com".to_string(), "my-api".to_string());
            assert_eq!(config.issuer, "https://auth.example.com");
            assert_eq!(config.audience, "my-api");
            assert!(config.public_key_pem.is_none());
            assert_eq!(
                config.allowed_algorithms,
                vec![jsonwebtoken::Algorithm::RS256]
            );
        }

        #[test]
        fn test_jwt_validator_requires_key() {
            let config = JwtConfig::new("issuer".to_string(), "audience".to_string());
            let validator = JwtValidator::new(config).expect("should create");
            let result = validator.validate_token("some.invalid.token");
            assert!(result.is_err());
        }

        #[test]
        fn test_token_claims_deserialization() {
            let json = r#"{
                "sub": "user123",
                "email": "user@example.com",
                "roles": ["admin", "operator"],
                "tenant_id": "tenant1",
                "exp": 9999999999,
                "iss": "https://auth.example.com"
            }"#;
            let claims: TokenClaims = serde_json::from_str(json).unwrap();
            assert_eq!(claims.sub, "user123");
            assert_eq!(claims.email, Some("user@example.com".to_string()));
            assert_eq!(claims.roles, vec!["admin", "operator"]);
            assert_eq!(claims.tenant_id, Some("tenant1".to_string()));
        }
    }
}
