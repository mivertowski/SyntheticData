//! Token bucket rate limiter implementation.
//!
//! Provides a token bucket algorithm for rate limiting with support for
//! burst capacity and multiple backpressure strategies.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Configuration for rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Target entities per second.
    pub entities_per_second: f64,
    /// Burst size (maximum tokens in bucket).
    pub burst_size: u32,
    /// Backpressure strategy when rate is exceeded.
    pub backpressure: RateLimitBackpressure,
    /// Whether rate limiting is enabled.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            entities_per_second: 1000.0,
            burst_size: 100,
            backpressure: RateLimitBackpressure::Block,
            enabled: true,
        }
    }
}

/// Backpressure strategy when rate limit is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitBackpressure {
    /// Block until tokens are available.
    #[default]
    Block,
    /// Drop excess items.
    Drop,
    /// Buffer items up to a limit, then block.
    Buffer {
        /// Maximum number of items to buffer.
        max_buffered: usize,
    },
}

/// Result of a rate limit acquisition.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitAction {
    /// Request can proceed immediately.
    Proceed,
    /// Request was dropped due to rate limiting.
    Dropped,
    /// Request was buffered.
    Buffered {
        /// Position in buffer.
        position: usize,
    },
    /// Request waited before proceeding.
    Waited {
        /// Time waited in milliseconds.
        wait_time_ms: u64,
    },
}

/// Statistics for the rate limiter.
#[derive(Debug, Clone, Default)]
pub struct RateLimiterStats {
    /// Total acquisitions attempted.
    pub total_acquisitions: u64,
    /// Acquisitions that proceeded immediately.
    pub immediate_proceeds: u64,
    /// Acquisitions that required waiting.
    pub waits: u64,
    /// Acquisitions that were dropped.
    pub drops: u64,
    /// Acquisitions that were buffered.
    pub buffers: u64,
    /// Total wait time in milliseconds.
    pub total_wait_time_ms: u64,
    /// Current tokens available.
    pub current_tokens: f64,
    /// Current buffer size.
    pub buffer_size: usize,
}

/// Token bucket rate limiter.
///
/// Implements the token bucket algorithm for rate limiting:
/// - Tokens are added at a steady rate up to a maximum (burst) capacity
/// - Each operation consumes one token
/// - If no tokens are available, behavior depends on backpressure strategy
pub struct RateLimiter {
    config: RateLimitConfig,
    /// Current number of tokens in the bucket.
    tokens: f64,
    /// Time of last token refill.
    last_refill: Instant,
    /// Buffer for items waiting due to rate limiting.
    buffer: VecDeque<Instant>,
    /// Statistics.
    stats: RateLimiterStats,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.burst_size as f64,
            last_refill: Instant::now(),
            buffer: VecDeque::new(),
            stats: RateLimiterStats {
                current_tokens: config.burst_size as f64,
                ..Default::default()
            },
            config,
        }
    }

    /// Creates a rate limiter with a simple rate (entities per second).
    pub fn with_rate(entities_per_second: f64) -> Self {
        Self::new(RateLimitConfig {
            entities_per_second,
            ..Default::default()
        })
    }

    /// Creates a disabled rate limiter (always allows).
    pub fn disabled() -> Self {
        Self::new(RateLimitConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Acquires a token, blocking if necessary.
    ///
    /// Returns the action taken to acquire the token.
    pub fn acquire(&mut self) -> RateLimitAction {
        if !self.config.enabled {
            self.stats.total_acquisitions += 1;
            self.stats.immediate_proceeds += 1;
            return RateLimitAction::Proceed;
        }

        self.stats.total_acquisitions += 1;
        self.refill_tokens();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.stats.current_tokens = self.tokens;
            self.stats.immediate_proceeds += 1;
            return RateLimitAction::Proceed;
        }

        // No tokens available, apply backpressure strategy
        match self.config.backpressure {
            RateLimitBackpressure::Block => {
                let wait_time = self.wait_for_token();
                self.stats.waits += 1;
                self.stats.total_wait_time_ms += wait_time;
                RateLimitAction::Waited {
                    wait_time_ms: wait_time,
                }
            }
            RateLimitBackpressure::Drop => {
                self.stats.drops += 1;
                RateLimitAction::Dropped
            }
            RateLimitBackpressure::Buffer { max_buffered } => {
                if self.buffer.len() < max_buffered {
                    self.buffer.push_back(Instant::now());
                    self.stats.buffers += 1;
                    self.stats.buffer_size = self.buffer.len();
                    RateLimitAction::Buffered {
                        position: self.buffer.len(),
                    }
                } else {
                    // Buffer full, block
                    let wait_time = self.wait_for_token();
                    self.stats.waits += 1;
                    self.stats.total_wait_time_ms += wait_time;
                    RateLimitAction::Waited {
                        wait_time_ms: wait_time,
                    }
                }
            }
        }
    }

    /// Tries to acquire a token without blocking.
    ///
    /// Returns `Some(action)` if a token was acquired, `None` if rate limited.
    pub fn try_acquire(&mut self) -> Option<RateLimitAction> {
        if !self.config.enabled {
            self.stats.total_acquisitions += 1;
            self.stats.immediate_proceeds += 1;
            return Some(RateLimitAction::Proceed);
        }

        self.refill_tokens();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.stats.current_tokens = self.tokens;
            self.stats.total_acquisitions += 1;
            self.stats.immediate_proceeds += 1;
            Some(RateLimitAction::Proceed)
        } else {
            None
        }
    }

    /// Acquires a token with a timeout.
    ///
    /// Returns the action taken, or `None` if the timeout was exceeded.
    pub fn acquire_timeout(&mut self, timeout: Duration) -> Option<RateLimitAction> {
        if !self.config.enabled {
            self.stats.total_acquisitions += 1;
            self.stats.immediate_proceeds += 1;
            return Some(RateLimitAction::Proceed);
        }

        self.stats.total_acquisitions += 1;
        self.refill_tokens();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.stats.current_tokens = self.tokens;
            self.stats.immediate_proceeds += 1;
            return Some(RateLimitAction::Proceed);
        }

        // Calculate time needed for one token
        let tokens_needed = 1.0 - self.tokens;
        let time_needed = Duration::from_secs_f64(tokens_needed / self.config.entities_per_second);

        if time_needed > timeout {
            // Timeout exceeded
            match self.config.backpressure {
                RateLimitBackpressure::Drop => {
                    self.stats.drops += 1;
                    Some(RateLimitAction::Dropped)
                }
                _ => None,
            }
        } else {
            std::thread::sleep(time_needed);
            self.refill_tokens();
            self.tokens -= 1.0;
            self.stats.current_tokens = self.tokens;
            self.stats.waits += 1;
            self.stats.total_wait_time_ms += time_needed.as_millis() as u64;
            Some(RateLimitAction::Waited {
                wait_time_ms: time_needed.as_millis() as u64,
            })
        }
    }

    /// Returns the current statistics.
    pub fn stats(&self) -> RateLimiterStats {
        let mut stats = self.stats.clone();
        stats.current_tokens = self.tokens;
        stats.buffer_size = self.buffer.len();
        stats
    }

    /// Resets the rate limiter to initial state.
    pub fn reset(&mut self) {
        self.tokens = self.config.burst_size as f64;
        self.last_refill = Instant::now();
        self.buffer.clear();
        self.stats = RateLimiterStats {
            current_tokens: self.tokens,
            ..Default::default()
        };
    }

    /// Returns the current number of available tokens.
    pub fn available_tokens(&self) -> f64 {
        self.tokens
    }

    /// Returns the configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Updates the rate limit.
    pub fn set_rate(&mut self, entities_per_second: f64) {
        self.config.entities_per_second = entities_per_second;
    }

    /// Enables or disables the rate limiter.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Refills tokens based on elapsed time.
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = elapsed.as_secs_f64() * self.config.entities_per_second;

        self.tokens = (self.tokens + new_tokens).min(self.config.burst_size as f64);
        self.last_refill = now;
    }

    /// Waits until a token is available.
    fn wait_for_token(&mut self) -> u64 {
        let tokens_needed = 1.0 - self.tokens;
        let wait_secs = tokens_needed / self.config.entities_per_second;
        let wait_duration = Duration::from_secs_f64(wait_secs);

        std::thread::sleep(wait_duration);

        self.refill_tokens();
        self.tokens -= 1.0;
        self.stats.current_tokens = self.tokens;

        wait_duration.as_millis() as u64
    }

    /// Processes the buffer, releasing items as tokens become available.
    pub fn process_buffer(&mut self) -> Vec<Duration> {
        self.refill_tokens();

        let mut wait_times = Vec::new();

        while !self.buffer.is_empty() && self.tokens >= 1.0 {
            if let Some(enqueue_time) = self.buffer.pop_front() {
                let wait_time = enqueue_time.elapsed();
                wait_times.push(wait_time);
                self.tokens -= 1.0;
            }
        }

        self.stats.buffer_size = self.buffer.len();
        self.stats.current_tokens = self.tokens;

        wait_times
    }
}

/// A rate-limited wrapper for any iterator.
pub struct RateLimitedIterator<I> {
    inner: I,
    limiter: RateLimiter,
}

impl<I> RateLimitedIterator<I> {
    /// Creates a new rate-limited iterator.
    pub fn new(inner: I, limiter: RateLimiter) -> Self {
        Self { inner, limiter }
    }

    /// Creates a rate-limited iterator with a simple rate.
    pub fn with_rate(inner: I, entities_per_second: f64) -> Self {
        Self::new(inner, RateLimiter::with_rate(entities_per_second))
    }

    /// Returns the limiter statistics.
    pub fn stats(&self) -> RateLimiterStats {
        self.limiter.stats()
    }
}

impl<I: Iterator> Iterator for RateLimitedIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.limiter.acquire();
        self.inner.next()
    }
}

/// Extension trait to add rate limiting to any iterator.
pub trait RateLimitExt: Iterator + Sized {
    /// Applies rate limiting to this iterator.
    fn rate_limit(self, entities_per_second: f64) -> RateLimitedIterator<Self> {
        RateLimitedIterator::with_rate(self, entities_per_second)
    }

    /// Applies rate limiting with custom config.
    fn rate_limit_with(self, config: RateLimitConfig) -> RateLimitedIterator<Self> {
        RateLimitedIterator::new(self, RateLimiter::new(config))
    }
}

impl<I: Iterator> RateLimitExt for I {}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_immediate_proceed() {
        let config = RateLimitConfig {
            entities_per_second: 1000.0,
            burst_size: 10,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // First 10 should proceed immediately (burst capacity)
        for _ in 0..10 {
            let action = limiter.acquire();
            assert_eq!(action, RateLimitAction::Proceed);
        }

        let stats = limiter.stats();
        assert_eq!(stats.total_acquisitions, 10);
        assert_eq!(stats.immediate_proceeds, 10);
    }

    #[test]
    fn test_rate_limiter_blocking() {
        let config = RateLimitConfig {
            entities_per_second: 1000.0,
            burst_size: 1,
            backpressure: RateLimitBackpressure::Block,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // First should proceed
        let action1 = limiter.acquire();
        assert_eq!(action1, RateLimitAction::Proceed);

        // Second should wait
        let action2 = limiter.acquire();
        assert!(matches!(action2, RateLimitAction::Waited { .. }));
    }

    #[test]
    fn test_rate_limiter_drop() {
        let config = RateLimitConfig {
            entities_per_second: 10.0,
            burst_size: 1,
            backpressure: RateLimitBackpressure::Drop,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // First should proceed
        let action1 = limiter.acquire();
        assert_eq!(action1, RateLimitAction::Proceed);

        // Second should be dropped (no time to refill)
        let action2 = limiter.acquire();
        assert_eq!(action2, RateLimitAction::Dropped);

        let stats = limiter.stats();
        assert_eq!(stats.drops, 1);
    }

    #[test]
    fn test_rate_limiter_buffer() {
        let config = RateLimitConfig {
            entities_per_second: 10.0,
            burst_size: 1,
            backpressure: RateLimitBackpressure::Buffer { max_buffered: 5 },
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // First should proceed
        let action1 = limiter.acquire();
        assert_eq!(action1, RateLimitAction::Proceed);

        // Next should be buffered
        let action2 = limiter.acquire();
        assert!(matches!(action2, RateLimitAction::Buffered { position: 1 }));

        let stats = limiter.stats();
        assert_eq!(stats.buffers, 1);
        assert_eq!(stats.buffer_size, 1);
    }

    #[test]
    fn test_rate_limiter_try_acquire() {
        let config = RateLimitConfig {
            entities_per_second: 10.0,
            burst_size: 1,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // First should succeed
        assert!(limiter.try_acquire().is_some());

        // Second should fail (no time to refill)
        assert!(limiter.try_acquire().is_none());
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let mut limiter = RateLimiter::disabled();

        // All should proceed immediately
        for _ in 0..100 {
            let action = limiter.acquire();
            assert_eq!(action, RateLimitAction::Proceed);
        }
    }

    #[test]
    fn test_rate_limiter_reset() {
        let config = RateLimitConfig {
            entities_per_second: 10.0,
            burst_size: 5,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // Consume some tokens
        for _ in 0..5 {
            limiter.acquire();
        }

        assert!(limiter.available_tokens() < 1.0);

        limiter.reset();

        assert_eq!(limiter.available_tokens(), 5.0);
    }

    #[test]
    fn test_rate_limited_iterator() {
        let items = vec![1, 2, 3, 4, 5];
        let rate_limited: Vec<_> = items
            .into_iter()
            .rate_limit_with(RateLimitConfig {
                entities_per_second: 10000.0,
                burst_size: 100,
                ..Default::default()
            })
            .collect();

        assert_eq!(rate_limited, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_rate_limiter_refill() {
        let config = RateLimitConfig {
            entities_per_second: 100.0, // 100 per second = 1 per 10ms
            burst_size: 10,
            ..Default::default()
        };
        let mut limiter = RateLimiter::new(config);

        // Consume all tokens
        for _ in 0..10 {
            limiter.try_acquire();
        }
        assert!(limiter.available_tokens() < 1.0);

        // Wait for refill (20ms should give ~2 tokens)
        std::thread::sleep(Duration::from_millis(25));

        // Should have some tokens now
        assert!(limiter.try_acquire().is_some());
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.entities_per_second, 1000.0);
        assert_eq!(config.burst_size, 100);
    }
}
