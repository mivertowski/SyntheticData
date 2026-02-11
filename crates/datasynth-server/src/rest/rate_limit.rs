//! Rate limiting middleware for REST API.
//!
//! Provides configurable rate limiting to prevent abuse.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiting configuration.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled.
    pub enabled: bool,
    /// Maximum requests per window.
    pub max_requests: u32,
    /// Time window duration.
    pub window: Duration,
    /// Paths exempt from rate limiting.
    pub exempt_paths: Vec<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests: 100,
            window: Duration::from_secs(60), // 100 requests per minute
            exempt_paths: vec![
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
            ],
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit config with custom limits.
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            enabled: true,
            max_requests,
            window: Duration::from_secs(window_secs),
            exempt_paths: vec![
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
            ],
        }
    }

    /// Add exempt paths.
    pub fn with_exempt_paths(mut self, paths: Vec<String>) -> Self {
        self.exempt_paths.extend(paths);
        self
    }
}

/// Request record for rate limiting.
#[derive(Clone)]
struct RequestRecord {
    count: u32,
    window_start: Instant,
}

/// Shared rate limiter state.
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    records: Arc<RwLock<HashMap<String, RequestRecord>>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request should be allowed.
    pub async fn check_rate_limit(&self, key: &str) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut records = self.records.write().await;
        let now = Instant::now();

        match records.get_mut(key) {
            Some(record) => {
                // Check if we're in a new window
                if now.duration_since(record.window_start) >= self.config.window {
                    // Reset for new window
                    record.count = 1;
                    record.window_start = now;
                    true
                } else if record.count < self.config.max_requests {
                    // Within window and under limit
                    record.count += 1;
                    true
                } else {
                    // Rate limited
                    false
                }
            }
            None => {
                // First request from this client
                records.insert(
                    key.to_string(),
                    RequestRecord {
                        count: 1,
                        window_start: now,
                    },
                );
                true
            }
        }
    }

    /// Get remaining requests for a key.
    pub async fn remaining(&self, key: &str) -> u32 {
        if !self.config.enabled {
            return self.config.max_requests;
        }

        let records = self.records.read().await;
        match records.get(key) {
            Some(record) => {
                let now = Instant::now();
                if now.duration_since(record.window_start) >= self.config.window {
                    self.config.max_requests
                } else {
                    self.config.max_requests.saturating_sub(record.count)
                }
            }
            None => self.config.max_requests,
        }
    }

    /// Clean up expired records.
    pub async fn cleanup_expired(&self) {
        let mut records = self.records.write().await;
        let now = Instant::now();
        records.retain(|_, record| now.duration_since(record.window_start) < self.config.window);
    }
}

/// Rate limiting middleware.
pub async fn rate_limit_middleware(
    axum::Extension(limiter): axum::Extension<RateLimiter>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Check if path is exempt
    let path = request.uri().path();
    if limiter
        .config
        .exempt_paths
        .iter()
        .any(|p| path.starts_with(p))
    {
        return next.run(request).await;
    }

    // Get client identifier (IP address or fallback)
    let client_key = extract_client_key(&request);

    // Check rate limit
    if limiter.check_rate_limit(&client_key).await {
        let remaining = limiter.remaining(&client_key).await;
        let mut response = next.run(request).await;

        // Add rate limit headers
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            limiter.config.max_requests.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap(),
        );

        response
    } else {
        // Rate limited
        let window_secs = limiter.config.window.as_secs();
        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("X-RateLimit-Limit", limiter.config.max_requests.to_string()),
                ("X-RateLimit-Remaining", "0".to_string()),
                ("Retry-After", window_secs.to_string()),
            ],
            format!(
                "Rate limit exceeded. Max {} requests per {} seconds.",
                limiter.config.max_requests, window_secs
            ),
        )
            .into_response()
    }
}

/// Extract client identifier from request.
fn extract_client_key(request: &Request<Body>) -> String {
    // Try X-Forwarded-For header (for proxied requests)
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(ip) = s.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(s) = real_ip.to_str() {
            return s.to_string();
        }
    }

    // Fallback to a default (in production, you'd want to extract from connection info)
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    fn test_router(config: RateLimitConfig) -> Router {
        let limiter = RateLimiter::new(config);
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(rate_limit_middleware))
            .layer(axum::Extension(limiter))
    }

    #[tokio::test]
    async fn test_rate_limit_disabled() {
        let config = RateLimitConfig::default();
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_allows_under_limit() {
        let config = RateLimitConfig::new(5, 60);
        let router = test_router(config);

        // Make 3 requests - should all succeed
        for _ in 0..3 {
            let router = router.clone();
            let request = Request::builder()
                .uri("/api/test")
                .header("X-Forwarded-For", "192.168.1.1")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_over_limit() {
        let config = RateLimitConfig::new(2, 60);
        let limiter = RateLimiter::new(config.clone());

        let router = Router::new()
            .route("/api/test", get(test_handler))
            .layer(middleware::from_fn(rate_limit_middleware))
            .layer(axum::Extension(limiter.clone()));

        // Make requests until rate limited
        for i in 0..3 {
            let router = router.clone();
            let request = Request::builder()
                .uri("/api/test")
                .header("X-Forwarded-For", "192.168.1.100")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            if i < 2 {
                assert_eq!(response.status(), StatusCode::OK);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            }
        }
    }

    #[tokio::test]
    async fn test_rate_limit_exempt_path() {
        let config = RateLimitConfig::new(1, 60);
        let limiter = RateLimiter::new(config);

        let router = Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(rate_limit_middleware))
            .layer(axum::Extension(limiter));

        // Exhaust rate limit on /api/test
        let request = Request::builder()
            .uri("/api/test")
            .header("X-Forwarded-For", "192.168.1.200")
            .body(Body::empty())
            .unwrap();
        let _ = router.clone().oneshot(request).await.unwrap();

        // /health should still work (exempt)
        let request = Request::builder()
            .uri("/health")
            .header("X-Forwarded-For", "192.168.1.200")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let config = RateLimitConfig::new(10, 1); // 1 second window
        let limiter = RateLimiter::new(config);

        // Make a request
        limiter.check_rate_limit("test-client").await;

        // Records should exist
        assert!(limiter.records.read().await.contains_key("test-client"));

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Clean up expired
        limiter.cleanup_expired().await;

        // Records should be removed
        assert!(!limiter.records.read().await.contains_key("test-client"));
    }
}
