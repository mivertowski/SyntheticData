//! Unified rate limiting backend.
//!
//! Provides a single `RateLimitBackend` enum that abstracts over in-memory
//! and Redis-backed rate limiting, allowing the middleware to work
//! transparently with either backend.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::rate_limit::{RateLimitConfig, RateLimiter};
#[cfg(feature = "redis")]
use super::redis_rate_limit::RedisRateLimiter;

/// Unified rate limiting backend that supports both in-memory and Redis
/// implementations.
///
/// When running a single server instance, `InMemory` is sufficient.
/// For distributed deployments with multiple server instances behind a
/// load balancer, use `Redis` to ensure consistent rate limiting across
/// all nodes.
#[derive(Clone)]
pub enum RateLimitBackend {
    /// In-memory rate limiter (single instance only).
    InMemory {
        limiter: RateLimiter,
        config: RateLimitConfig,
    },
    /// Redis-backed rate limiter (distributed, multi-instance).
    #[cfg(feature = "redis")]
    Redis {
        limiter: Box<RedisRateLimiter>,
        config: RateLimitConfig,
    },
}

impl RateLimitBackend {
    /// Create a new in-memory rate limiting backend.
    pub fn in_memory(config: RateLimitConfig) -> Self {
        let limiter = RateLimiter::new(config.clone());
        Self::InMemory { limiter, config }
    }

    /// Create a new Redis-backed rate limiting backend.
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., `redis://127.0.0.1:6379`)
    /// * `config` - Rate limit configuration
    #[cfg(feature = "redis")]
    pub async fn redis(
        redis_url: &str,
        config: RateLimitConfig,
    ) -> Result<Self, redis::RedisError> {
        let limiter = RedisRateLimiter::new(redis_url, config.max_requests, config.window).await?;
        Ok(Self::Redis {
            limiter: Box::new(limiter),
            config,
        })
    }

    /// Get the rate limit configuration.
    pub fn config(&self) -> &RateLimitConfig {
        match self {
            Self::InMemory { config, .. } => config,
            #[cfg(feature = "redis")]
            Self::Redis { config, .. } => config,
        }
    }

    /// Check if a request from the given client should be allowed.
    ///
    /// Returns `true` if the request is allowed.
    pub async fn check_rate_limit(&self, client_key: &str) -> bool {
        match self {
            Self::InMemory { limiter, config } => {
                if !config.enabled {
                    return true;
                }
                limiter.check_rate_limit(client_key).await
            }
            #[cfg(feature = "redis")]
            Self::Redis { limiter, config } => {
                if !config.enabled {
                    return true;
                }
                limiter.check_rate_limit(client_key).await.allowed
            }
        }
    }

    /// Get remaining requests for a client key.
    pub async fn remaining(&self, client_key: &str) -> u32 {
        match self {
            Self::InMemory { limiter, config } => {
                if !config.enabled {
                    return config.max_requests;
                }
                limiter.remaining(client_key).await
            }
            #[cfg(feature = "redis")]
            Self::Redis { limiter, config } => {
                if !config.enabled {
                    return config.max_requests;
                }
                limiter.remaining(client_key).await
            }
        }
    }

    /// Clean up expired records (only applicable to in-memory backend).
    ///
    /// For Redis, TTL-based expiry is handled automatically by Redis.
    pub async fn cleanup_expired(&self) {
        match self {
            Self::InMemory { limiter, .. } => {
                limiter.cleanup_expired().await;
            }
            #[cfg(feature = "redis")]
            Self::Redis { .. } => {
                // Redis handles expiry automatically via TTL
            }
        }
    }

    /// Return a human-readable description of the backend type.
    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::InMemory { .. } => "in-memory",
            #[cfg(feature = "redis")]
            Self::Redis { .. } => "redis",
        }
    }
}

/// Rate limiting middleware that works with any `RateLimitBackend`.
///
/// This replaces the original `rate_limit_middleware` when using the
/// backend abstraction. It is added to the router via
/// `axum::middleware::from_fn(backend_rate_limit_middleware)` and expects
/// `RateLimitBackend` to be available as an `Extension`.
pub async fn backend_rate_limit_middleware(
    axum::Extension(backend): axum::Extension<RateLimitBackend>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = backend.config();

    // Check if rate limiting is enabled
    if !config.enabled {
        return next.run(request).await;
    }

    // Check if path is exempt
    let path = request.uri().path();
    if config.exempt_paths.iter().any(|p| path.starts_with(p)) {
        return next.run(request).await;
    }

    // Get client identifier (IP address or fallback)
    let client_key = extract_client_key(&request);
    let max_requests = config.max_requests;
    let window_secs = config.window.as_secs();

    // Check rate limit
    if backend.check_rate_limit(&client_key).await {
        let remaining = backend.remaining(&client_key).await;
        let mut response = next.run(request).await;

        // Add rate limit headers
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            max_requests.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap(),
        );

        response
    } else {
        // Rate limited
        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("X-RateLimit-Limit", max_requests.to_string()),
                ("X-RateLimit-Remaining", "0".to_string()),
                ("Retry-After", window_secs.to_string()),
            ],
            format!(
                "Rate limit exceeded. Max {} requests per {} seconds.",
                max_requests, window_secs
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

    // Fallback to a default
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

    fn test_router_with_backend(config: RateLimitConfig) -> Router {
        let backend = RateLimitBackend::in_memory(config);
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(backend_rate_limit_middleware))
            .layer(axum::Extension(backend))
    }

    #[tokio::test]
    async fn test_backend_rate_limit_disabled() {
        let config = RateLimitConfig::default(); // disabled by default
        let router = test_router_with_backend(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_backend_rate_limit_allows_under_limit() {
        let config = RateLimitConfig::new(5, 60);
        let router = test_router_with_backend(config);

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
    async fn test_backend_rate_limit_blocks_over_limit() {
        let config = RateLimitConfig::new(2, 60);
        let backend = RateLimitBackend::in_memory(config.clone());

        let router = Router::new()
            .route("/api/test", get(test_handler))
            .layer(middleware::from_fn(backend_rate_limit_middleware))
            .layer(axum::Extension(backend));

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
    async fn test_backend_rate_limit_exempt_path() {
        let config = RateLimitConfig::new(1, 60);
        let backend = RateLimitBackend::in_memory(config);

        let router = Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(backend_rate_limit_middleware))
            .layer(axum::Extension(backend));

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

    #[test]
    fn test_backend_name_in_memory() {
        let config = RateLimitConfig::default();
        let backend = RateLimitBackend::in_memory(config);
        assert_eq!(backend.backend_name(), "in-memory");
    }

    #[tokio::test]
    async fn test_backend_cleanup_in_memory() {
        let config = RateLimitConfig::new(10, 1);
        let backend = RateLimitBackend::in_memory(config);

        // Should not panic for in-memory
        backend.cleanup_expired().await;
    }
}
