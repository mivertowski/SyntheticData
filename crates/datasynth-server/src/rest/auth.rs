//! Authentication middleware for REST API.
//!
//! Provides API key authentication with Argon2id hashing and
//! timing-safe comparison for protecting endpoints.

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
        }
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

/// Authentication middleware that checks for valid API key.
///
/// Checks for API key in:
/// 1. `Authorization: Bearer <key>` header
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

    // Extract API key from headers
    let api_key = extract_api_key(&request);

    match api_key {
        Some(key) if config.verify_key(&key) => {
            // Valid API key, proceed
            next.run(request).await
        }
        Some(_) => {
            // Invalid API key
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Invalid API key",
            )
                .into_response()
        }
        None => {
            // No API key provided
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "API key required. Provide via 'Authorization: Bearer <key>' or 'X-API-Key' header",
            )
                .into_response()
        }
    }
}

/// Extract API key from request headers.
fn extract_api_key(request: &Request<Body>) -> Option<String> {
    // Try Authorization: Bearer <key>
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return Some(key.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = request.headers().get("X-API-Key") {
        if let Ok(key) = api_key_header.to_str() {
            return Some(key.to_string());
        }
    }

    None
}

#[cfg(test)]
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
}
