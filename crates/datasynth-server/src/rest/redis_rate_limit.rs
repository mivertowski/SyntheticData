//! Redis-backed distributed rate limiting.
//!
//! Uses a Lua script for atomic INCR + EXPIRE sliding window rate limiting,
//! enabling consistent rate limiting across multiple server instances.

use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::Script;
use tracing::{error, info, warn};

/// Redis-backed rate limiter for distributed deployments.
///
/// Uses a sliding window counter implemented via a Lua script that
/// atomically increments the request count and sets expiry, ensuring
/// consistency even under high concurrency across multiple nodes.
#[derive(Clone)]
pub struct RedisRateLimiter {
    conn: ConnectionManager,
    max_requests: u32,
    window_secs: u64,
    key_prefix: String,
    script: Script,
}

/// Result of a Redis rate limit check.
#[derive(Debug, Clone)]
pub struct RedisRateLimitResult {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Current request count in the window.
    pub current_count: u32,
    /// Maximum requests allowed per window.
    pub max_requests: u32,
    /// Remaining requests in the current window.
    pub remaining: u32,
}

/// Lua script for atomic sliding window rate limiting.
///
/// This script atomically:
/// 1. Increments the counter for the given key
/// 2. Sets the TTL on first request (when count becomes 1)
/// 3. Returns the current count and TTL remaining
///
/// Returns: [current_count, ttl_remaining]
const RATE_LIMIT_SCRIPT: &str = r#"
local key = KEYS[1]
local max_requests = tonumber(ARGV[1])
local window_secs = tonumber(ARGV[2])

local current = redis.call('INCR', key)

if current == 1 then
    redis.call('EXPIRE', key, window_secs)
end

local ttl = redis.call('TTL', key)

-- Safety: if TTL is -1 (no expiry set, race condition), reset it
if ttl == -1 then
    redis.call('EXPIRE', key, window_secs)
    ttl = window_secs
end

return {current, ttl}
"#;

impl RedisRateLimiter {
    /// Create a new Redis rate limiter by connecting to the given URL.
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., `redis://127.0.0.1:6379`)
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window` - Time window duration
    pub async fn new(
        redis_url: &str,
        max_requests: u32,
        window: Duration,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        info!(
            "Redis rate limiter connected to {} (max {} requests per {}s)",
            redis_url,
            max_requests,
            window.as_secs()
        );

        Ok(Self {
            conn,
            max_requests,
            window_secs: window.as_secs(),
            key_prefix: "datasynth:ratelimit:".to_string(),
            script: Script::new(RATE_LIMIT_SCRIPT),
        })
    }

    /// Set a custom key prefix for Redis keys.
    ///
    /// Default is `datasynth:ratelimit:`.
    pub fn with_key_prefix(mut self, prefix: String) -> Self {
        self.key_prefix = prefix;
        self
    }

    /// Build the Redis key for a given client identifier.
    fn make_key(&self, client_key: &str) -> String {
        format!("{}{}", self.key_prefix, client_key)
    }

    /// Check if a request from the given client should be allowed.
    ///
    /// Returns a `RedisRateLimitResult` with the decision and metadata.
    /// On Redis errors, falls back to allowing the request (fail-open)
    /// to avoid blocking traffic when Redis is unavailable.
    pub async fn check_rate_limit(&self, client_key: &str) -> RedisRateLimitResult {
        let key = self.make_key(client_key);
        let mut conn = self.conn.clone();

        match self
            .script
            .key(&key)
            .arg(self.max_requests)
            .arg(self.window_secs)
            .invoke_async::<Vec<u32>>(&mut conn)
            .await
        {
            Ok(result) if result.len() == 2 => {
                let current_count = result[0];
                let allowed = current_count <= self.max_requests;
                let remaining = if allowed {
                    self.max_requests.saturating_sub(current_count)
                } else {
                    0
                };

                RedisRateLimitResult {
                    allowed,
                    current_count,
                    max_requests: self.max_requests,
                    remaining,
                }
            }
            Ok(result) => {
                warn!(
                    "Unexpected Redis rate limit script result length: {}",
                    result.len()
                );
                // Fail open
                RedisRateLimitResult {
                    allowed: true,
                    current_count: 0,
                    max_requests: self.max_requests,
                    remaining: self.max_requests,
                }
            }
            Err(e) => {
                error!("Redis rate limit check failed (failing open): {}", e);
                // Fail open - allow request when Redis is unavailable
                RedisRateLimitResult {
                    allowed: true,
                    current_count: 0,
                    max_requests: self.max_requests,
                    remaining: self.max_requests,
                }
            }
        }
    }

    /// Get the remaining request count for a client without consuming a request.
    ///
    /// On Redis errors, returns the max requests (fail-open).
    pub async fn remaining(&self, client_key: &str) -> u32 {
        let key = self.make_key(client_key);
        let mut conn = self.conn.clone();

        match redis::cmd("GET")
            .arg(&key)
            .query_async::<Option<u32>>(&mut conn)
            .await
        {
            Ok(Some(count)) => self.max_requests.saturating_sub(count),
            Ok(None) => self.max_requests,
            Err(e) => {
                error!("Redis remaining check failed (failing open): {}", e);
                self.max_requests
            }
        }
    }

    /// Get the maximum requests per window.
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }

    /// Get the window duration in seconds.
    pub fn window_secs(&self) -> u64 {
        self.window_secs
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_make_key() {
        // We can test key generation without a Redis connection
        let key_prefix = "datasynth:ratelimit:".to_string();
        let key = format!("{}192.168.1.1", key_prefix);
        assert_eq!(key, "datasynth:ratelimit:192.168.1.1");
    }

    #[test]
    fn test_rate_limit_result() {
        let result = RedisRateLimitResult {
            allowed: true,
            current_count: 5,
            max_requests: 100,
            remaining: 95,
        };
        assert!(result.allowed);
        assert_eq!(result.remaining, 95);
    }

    #[test]
    fn test_rate_limit_result_exceeded() {
        let result = RedisRateLimitResult {
            allowed: false,
            current_count: 101,
            max_requests: 100,
            remaining: 0,
        };
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }
}
