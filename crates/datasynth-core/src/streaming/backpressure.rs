//! Backpressure handling for streaming generation.
//!
//! Provides strategies and utilities for handling backpressure
//! when consumers cannot keep up with producers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::traits::BackpressureStrategy;

/// Monitors and reports backpressure conditions.
#[derive(Debug)]
pub struct BackpressureMonitor {
    /// Strategy in use.
    strategy: BackpressureStrategy,
    /// Capacity threshold.
    capacity: usize,
    /// Current fill level.
    current_fill: AtomicU64,
    /// High watermark (when to slow down).
    high_watermark: f64,
    /// Low watermark (when to resume full speed).
    low_watermark: f64,
    /// Total items dropped.
    items_dropped: AtomicU64,
    /// Total time spent blocked (nanoseconds).
    blocked_time_ns: AtomicU64,
    /// Number of times backpressure was triggered.
    backpressure_events: AtomicU64,
}

impl BackpressureMonitor {
    /// Creates a new backpressure monitor.
    pub fn new(strategy: BackpressureStrategy, capacity: usize) -> Self {
        Self {
            strategy,
            capacity,
            current_fill: AtomicU64::new(0),
            high_watermark: 0.8,
            low_watermark: 0.5,
            items_dropped: AtomicU64::new(0),
            blocked_time_ns: AtomicU64::new(0),
            backpressure_events: AtomicU64::new(0),
        }
    }

    /// Creates a monitor with custom watermarks.
    pub fn with_watermarks(mut self, high: f64, low: f64) -> Self {
        self.high_watermark = high.clamp(0.0, 1.0);
        self.low_watermark = low.clamp(0.0, self.high_watermark);
        self
    }

    /// Updates the current fill level.
    pub fn update_fill(&self, current: usize) {
        self.current_fill.store(current as u64, Ordering::Relaxed);
    }

    /// Returns the current fill ratio (0.0 to 1.0+).
    pub fn fill_ratio(&self) -> f64 {
        self.current_fill.load(Ordering::Relaxed) as f64 / self.capacity as f64
    }

    /// Returns whether backpressure should be applied.
    pub fn should_apply_backpressure(&self) -> bool {
        self.fill_ratio() >= self.high_watermark
    }

    /// Returns whether the system has recovered from backpressure.
    pub fn has_recovered(&self) -> bool {
        self.fill_ratio() <= self.low_watermark
    }

    /// Records a backpressure event.
    pub fn record_backpressure(&self) {
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Records dropped items.
    pub fn record_dropped(&self, count: u64) {
        self.items_dropped.fetch_add(count, Ordering::Relaxed);
    }

    /// Records blocked time.
    pub fn record_blocked_time(&self, duration: Duration) {
        self.blocked_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Returns backpressure statistics.
    pub fn stats(&self) -> BackpressureStats {
        BackpressureStats {
            strategy: self.strategy,
            fill_ratio: self.fill_ratio(),
            items_dropped: self.items_dropped.load(Ordering::Relaxed),
            blocked_time_ms: self.blocked_time_ns.load(Ordering::Relaxed) / 1_000_000,
            backpressure_events: self.backpressure_events.load(Ordering::Relaxed),
            is_under_pressure: self.should_apply_backpressure(),
        }
    }

    /// Returns the configured strategy.
    pub fn strategy(&self) -> BackpressureStrategy {
        self.strategy
    }
}

/// Statistics about backpressure handling.
#[derive(Debug, Clone)]
pub struct BackpressureStats {
    /// Strategy in use.
    pub strategy: BackpressureStrategy,
    /// Current fill ratio.
    pub fill_ratio: f64,
    /// Total items dropped.
    pub items_dropped: u64,
    /// Total time spent blocked (milliseconds).
    pub blocked_time_ms: u64,
    /// Number of backpressure events.
    pub backpressure_events: u64,
    /// Currently under backpressure.
    pub is_under_pressure: bool,
}

/// Adaptive backpressure controller that adjusts generation rate.
#[derive(Debug)]
pub struct AdaptiveBackpressure {
    /// Target fill ratio to maintain.
    target_fill: f64,
    /// Minimum delay between items (nanoseconds).
    min_delay_ns: u64,
    /// Maximum delay between items (nanoseconds).
    max_delay_ns: u64,
    /// Current delay.
    current_delay_ns: AtomicU64,
    /// Last adjustment time.
    last_adjustment: std::sync::Mutex<Instant>,
    /// Adjustment interval.
    adjustment_interval: Duration,
}

impl AdaptiveBackpressure {
    /// Creates a new adaptive backpressure controller.
    pub fn new() -> Self {
        Self {
            target_fill: 0.7,
            min_delay_ns: 0,
            max_delay_ns: 10_000_000, // 10ms
            current_delay_ns: AtomicU64::new(0),
            last_adjustment: std::sync::Mutex::new(Instant::now()),
            adjustment_interval: Duration::from_millis(100),
        }
    }

    /// Sets the target fill ratio.
    pub fn with_target_fill(mut self, target: f64) -> Self {
        self.target_fill = target.clamp(0.1, 0.9);
        self
    }

    /// Sets the delay bounds.
    pub fn with_delay_bounds(mut self, min: Duration, max: Duration) -> Self {
        self.min_delay_ns = min.as_nanos() as u64;
        self.max_delay_ns = max.as_nanos() as u64;
        self
    }

    /// Adjusts the delay based on current fill ratio.
    pub fn adjust(&self, current_fill: f64) {
        let mut last_adj = self.last_adjustment.lock().expect("mutex poisoned");
        if last_adj.elapsed() < self.adjustment_interval {
            return;
        }
        *last_adj = Instant::now();
        drop(last_adj);

        let current_delay = self.current_delay_ns.load(Ordering::Relaxed);
        let error = current_fill - self.target_fill;

        // Simple proportional control
        // When current delay is 0 and we need to increase, use a minimum step
        let new_delay = if current_delay == 0 && error > 0.0 {
            // Start with 1 microsecond (1000 nanoseconds) when at zero
            let step = (self.max_delay_ns / 10).max(1000);
            (step as f64 * error * 2.0) as u64
        } else {
            let adjustment_factor = 1.0 + error * 0.5;
            (current_delay as f64 * adjustment_factor) as u64
        };

        // Clamp to bounds
        let clamped_delay = new_delay.clamp(self.min_delay_ns, self.max_delay_ns);
        self.current_delay_ns
            .store(clamped_delay, Ordering::Relaxed);
    }

    /// Returns the current delay to apply.
    pub fn current_delay(&self) -> Duration {
        Duration::from_nanos(self.current_delay_ns.load(Ordering::Relaxed))
    }

    /// Resets the delay to minimum.
    pub fn reset(&self) {
        self.current_delay_ns
            .store(self.min_delay_ns, Ordering::Relaxed);
    }
}

impl Default for AdaptiveBackpressure {
    fn default() -> Self {
        Self::new()
    }
}

/// A rate-aware producer that respects backpressure.
pub struct BackpressureAwareProducer {
    /// Backpressure monitor.
    monitor: BackpressureMonitor,
    /// Adaptive controller.
    adaptive: Option<AdaptiveBackpressure>,
    /// Current state.
    state: BackpressureState,
}

/// State of the backpressure system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureState {
    /// Normal operation.
    Normal,
    /// Slowing down due to high fill.
    SlowingDown,
    /// Fully blocked waiting for space.
    Blocked,
    /// Recovering from backpressure.
    Recovering,
}

impl BackpressureAwareProducer {
    /// Creates a new backpressure-aware producer.
    pub fn new(strategy: BackpressureStrategy, capacity: usize) -> Self {
        Self {
            monitor: BackpressureMonitor::new(strategy, capacity),
            adaptive: None,
            state: BackpressureState::Normal,
        }
    }

    /// Enables adaptive rate control.
    pub fn with_adaptive(mut self) -> Self {
        self.adaptive = Some(AdaptiveBackpressure::new());
        self
    }

    /// Updates the fill level and adjusts state.
    pub fn update(&mut self, fill_level: usize) {
        self.monitor.update_fill(fill_level);
        let ratio = self.monitor.fill_ratio();

        // Update adaptive controller
        if let Some(ref adaptive) = self.adaptive {
            adaptive.adjust(ratio);
        }

        // Update state
        self.state = if ratio >= 1.0 {
            BackpressureState::Blocked
        } else if self.monitor.should_apply_backpressure() {
            if self.state == BackpressureState::Normal {
                self.monitor.record_backpressure();
            }
            BackpressureState::SlowingDown
        } else if self.monitor.has_recovered() {
            BackpressureState::Normal
        } else if self.state == BackpressureState::SlowingDown {
            BackpressureState::Recovering
        } else {
            self.state
        };
    }

    /// Returns the current state.
    pub fn state(&self) -> BackpressureState {
        self.state
    }

    /// Returns the recommended delay before the next send.
    pub fn recommended_delay(&self) -> Duration {
        match self.state {
            BackpressureState::Normal => Duration::ZERO,
            BackpressureState::SlowingDown | BackpressureState::Recovering => self
                .adaptive
                .as_ref()
                .map(|a| a.current_delay())
                .unwrap_or(Duration::from_micros(100)),
            BackpressureState::Blocked => Duration::from_millis(1),
        }
    }

    /// Records dropped items.
    pub fn record_dropped(&self, count: u64) {
        self.monitor.record_dropped(count);
    }

    /// Returns statistics.
    pub fn stats(&self) -> BackpressureStats {
        self.monitor.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_monitor() {
        let monitor = BackpressureMonitor::new(BackpressureStrategy::Block, 100);

        monitor.update_fill(50);
        assert!(!monitor.should_apply_backpressure());

        monitor.update_fill(80);
        assert!(monitor.should_apply_backpressure());

        monitor.update_fill(40);
        assert!(monitor.has_recovered());
    }

    #[test]
    fn test_backpressure_monitor_stats() {
        let monitor = BackpressureMonitor::new(BackpressureStrategy::DropOldest, 100);

        monitor.record_dropped(5);
        monitor.record_backpressure();
        monitor.record_blocked_time(Duration::from_millis(100));

        let stats = monitor.stats();
        assert_eq!(stats.items_dropped, 5);
        assert_eq!(stats.backpressure_events, 1);
        assert!(stats.blocked_time_ms >= 100);
    }

    #[test]
    fn test_adaptive_backpressure() {
        let adaptive = AdaptiveBackpressure::new()
            .with_target_fill(0.5)
            .with_delay_bounds(Duration::ZERO, Duration::from_millis(100));

        assert_eq!(adaptive.current_delay(), Duration::ZERO);

        // Simulate high fill - should increase delay
        for _ in 0..10 {
            adaptive.adjust(0.9);
            std::thread::sleep(Duration::from_millis(110));
        }

        // Delay should have increased
        assert!(adaptive.current_delay() > Duration::ZERO);
    }

    #[test]
    fn test_backpressure_aware_producer() {
        let mut producer = BackpressureAwareProducer::new(BackpressureStrategy::Block, 100);

        producer.update(50);
        assert_eq!(producer.state(), BackpressureState::Normal);

        producer.update(85);
        assert_eq!(producer.state(), BackpressureState::SlowingDown);

        producer.update(40);
        assert_eq!(producer.state(), BackpressureState::Normal);
    }

    #[test]
    fn test_backpressure_state_blocked() {
        let mut producer = BackpressureAwareProducer::new(BackpressureStrategy::Block, 100);

        producer.update(100);
        assert_eq!(producer.state(), BackpressureState::Blocked);
    }
}
