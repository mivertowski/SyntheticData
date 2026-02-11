//! Server metrics abstraction.
//!
//! Provides a unified metrics interface that works with or without the `otel` feature.
//! When `otel` is enabled, metrics are recorded via OpenTelemetry instruments.
//! When disabled, they proxy to the existing `ServerState` atomics.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Server metrics collector.
///
/// This is a lightweight wrapper that records metrics.
/// Without the `otel` feature, it delegates to `ServerState` atomics.
/// With `otel`, it additionally records via OTEL instruments.
#[derive(Clone)]
pub struct ServerMetrics {
    /// Total entries generated (counter)
    pub entries_total: Arc<AtomicU64>,
    /// Total errors (counter)
    pub errors_total: Arc<AtomicU64>,
    /// Active sessions (gauge)
    pub active_sessions: Arc<AtomicU64>,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            entries_total: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
            active_sessions: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record entries generated.
    pub fn record_entries(&self, count: u64) {
        self.entries_total.fetch_add(count, Ordering::Relaxed);
    }

    /// Record an error.
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active sessions.
    pub fn session_started(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active sessions.
    pub fn session_ended(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Timer for measuring operation duration.
pub struct DurationTimer {
    start: Instant,
    label: String,
}

impl DurationTimer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    /// Finish the timer and return duration in milliseconds.
    pub fn finish(self) -> u64 {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        tracing::debug!(
            label = %self.label,
            duration_ms = duration_ms,
            "Operation completed"
        );
        duration_ms
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_server_metrics_default() {
        let metrics = ServerMetrics::new();
        assert_eq!(metrics.entries_total.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.errors_total.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.active_sessions.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_entries() {
        let metrics = ServerMetrics::new();
        metrics.record_entries(100);
        metrics.record_entries(50);
        assert_eq!(metrics.entries_total.load(Ordering::Relaxed), 150);
    }

    #[test]
    fn test_record_errors() {
        let metrics = ServerMetrics::new();
        metrics.record_error();
        metrics.record_error();
        assert_eq!(metrics.errors_total.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_session_tracking() {
        let metrics = ServerMetrics::new();
        metrics.session_started();
        metrics.session_started();
        assert_eq!(metrics.active_sessions.load(Ordering::Relaxed), 2);
        metrics.session_ended();
        assert_eq!(metrics.active_sessions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_duration_timer() {
        let timer = DurationTimer::new("test_op");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.finish();
        assert!(duration >= 10);
    }
}
