//! CPU load monitoring for preventing system overload.
//!
//! This module provides CPU load tracking with configurable thresholds
//! and optional auto-throttling to maintain system responsiveness.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// CPU load statistics.
#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    /// Current CPU load (0.0 - 1.0)
    pub current_load: f64,
    /// Average CPU load over sample window
    pub average_load: f64,
    /// Peak CPU load observed
    pub peak_load: f64,
    /// Number of samples collected
    pub samples_collected: u64,
    /// Whether throttling is currently active
    pub is_throttling: bool,
    /// Number of times throttling was triggered
    pub throttle_count: u64,
}

/// CPU monitor configuration.
#[derive(Debug, Clone)]
pub struct CpuMonitorConfig {
    /// Enable CPU monitoring
    pub enabled: bool,
    /// High load threshold (0.0 - 1.0), triggers warning
    pub high_load_threshold: f64,
    /// Critical load threshold (0.0 - 1.0), triggers throttling
    pub critical_load_threshold: f64,
    /// Sample interval in milliseconds
    pub sample_interval_ms: u64,
    /// Number of samples to keep for averaging
    pub sample_window_size: usize,
    /// Enable automatic throttling when critical threshold exceeded
    pub auto_throttle: bool,
    /// Throttle delay in milliseconds (pause between operations)
    pub throttle_delay_ms: u64,
}

impl Default for CpuMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            high_load_threshold: 0.85,
            critical_load_threshold: 0.95,
            sample_interval_ms: 1000,
            sample_window_size: 10,
            auto_throttle: true,
            throttle_delay_ms: 50,
        }
    }
}

impl CpuMonitorConfig {
    /// Create config with specified thresholds.
    pub fn with_thresholds(high: f64, critical: f64) -> Self {
        Self {
            enabled: true,
            high_load_threshold: high.clamp(0.0, 1.0),
            critical_load_threshold: critical.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Enable auto-throttling.
    pub fn with_auto_throttle(mut self, delay_ms: u64) -> Self {
        self.auto_throttle = true;
        self.throttle_delay_ms = delay_ms;
        self
    }

    /// Disable auto-throttling.
    pub fn without_auto_throttle(mut self) -> Self {
        self.auto_throttle = false;
        self
    }
}

/// CPU load exceeded error.
#[derive(Debug, Clone)]
pub struct CpuOverloaded {
    pub current_load: f64,
    pub threshold: f64,
    pub is_critical: bool,
    pub message: String,
}

impl std::fmt::Display for CpuOverloaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CpuOverloaded {}

/// Thread-safe CPU load monitor.
#[derive(Debug)]
pub struct CpuMonitor {
    config: CpuMonitorConfig,
    load_history: Arc<RwLock<VecDeque<f64>>>,
    current_load_raw: AtomicU64,
    peak_load_raw: AtomicU64,
    is_throttling: AtomicBool,
    throttle_count: AtomicU64,
    samples_collected: AtomicU64,
    last_sample_time: Arc<RwLock<Option<Instant>>>,
    // CPU time tracking for load calculation
    #[cfg(target_os = "linux")]
    last_cpu_times: Arc<RwLock<Option<(u64, u64)>>>,
}

impl CpuMonitor {
    /// Create a new CPU monitor with the given configuration.
    pub fn new(config: CpuMonitorConfig) -> Self {
        Self {
            config,
            load_history: Arc::new(RwLock::new(VecDeque::new())),
            current_load_raw: AtomicU64::new(0),
            peak_load_raw: AtomicU64::new(0),
            is_throttling: AtomicBool::new(false),
            throttle_count: AtomicU64::new(0),
            samples_collected: AtomicU64::new(0),
            last_sample_time: Arc::new(RwLock::new(None)),
            #[cfg(target_os = "linux")]
            last_cpu_times: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a disabled CPU monitor.
    pub fn disabled() -> Self {
        Self::new(CpuMonitorConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Create an Arc-wrapped CPU monitor for sharing across threads.
    pub fn shared(config: CpuMonitorConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Check if monitoring is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Sample current CPU load and update statistics.
    pub fn sample(&self) -> Option<f64> {
        if !self.config.enabled {
            return None;
        }

        // Check if enough time has passed since last sample
        {
            let mut last_time = self.last_sample_time.write().ok()?;
            let now = Instant::now();
            if let Some(last) = *last_time {
                if now.duration_since(last).as_millis() < self.config.sample_interval_ms as u128 {
                    // Return current load without sampling
                    return Some(f64::from_bits(
                        self.current_load_raw.load(Ordering::Relaxed),
                    ));
                }
            }
            *last_time = Some(now);
        }

        let load = self.get_cpu_load()?;

        // Update current load
        self.current_load_raw
            .store(load.to_bits(), Ordering::Relaxed);

        // Update peak
        let mut peak = f64::from_bits(self.peak_load_raw.load(Ordering::Relaxed));
        while load > peak {
            match self.peak_load_raw.compare_exchange_weak(
                peak.to_bits(),
                load.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = f64::from_bits(p),
            }
        }

        // Update history
        if let Ok(mut history) = self.load_history.write() {
            history.push_back(load);
            while history.len() > self.config.sample_window_size {
                history.pop_front();
            }
        }

        self.samples_collected.fetch_add(1, Ordering::Relaxed);

        // Check thresholds and apply throttling
        if load >= self.config.critical_load_threshold {
            if self.config.auto_throttle && !self.is_throttling.load(Ordering::Relaxed) {
                self.is_throttling.store(true, Ordering::Relaxed);
                self.throttle_count.fetch_add(1, Ordering::Relaxed);
            }
        } else if load < self.config.high_load_threshold {
            self.is_throttling.store(false, Ordering::Relaxed);
        }

        Some(load)
    }

    /// Check CPU load and return error if threshold exceeded.
    pub fn check(&self) -> Result<(), CpuOverloaded> {
        if !self.config.enabled {
            return Ok(());
        }

        let load = self.sample().unwrap_or(0.0);

        if load >= self.config.critical_load_threshold {
            return Err(CpuOverloaded {
                current_load: load,
                threshold: self.config.critical_load_threshold,
                is_critical: true,
                message: format!(
                    "Critical CPU load: {:.1}% exceeds critical threshold of {:.1}%. \
                     Reduce parallel workers or enable throttling.",
                    load * 100.0,
                    self.config.critical_load_threshold * 100.0
                ),
            });
        }

        Ok(())
    }

    /// Apply throttle delay if throttling is active.
    pub fn maybe_throttle(&self) {
        if self.config.auto_throttle && self.is_throttling.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(self.config.throttle_delay_ms));
        }
    }

    /// Get current statistics.
    pub fn stats(&self) -> CpuStats {
        let current = f64::from_bits(self.current_load_raw.load(Ordering::Relaxed));
        let peak = f64::from_bits(self.peak_load_raw.load(Ordering::Relaxed));

        let average = if let Ok(history) = self.load_history.read() {
            if history.is_empty() {
                0.0
            } else {
                history.iter().sum::<f64>() / history.len() as f64
            }
        } else {
            current
        };

        CpuStats {
            current_load: current,
            average_load: average,
            peak_load: peak,
            samples_collected: self.samples_collected.load(Ordering::Relaxed),
            is_throttling: self.is_throttling.load(Ordering::Relaxed),
            throttle_count: self.throttle_count.load(Ordering::Relaxed),
        }
    }

    /// Get current CPU load.
    pub fn current_load(&self) -> f64 {
        f64::from_bits(self.current_load_raw.load(Ordering::Relaxed))
    }

    /// Check if throttling is currently active.
    pub fn is_throttling(&self) -> bool {
        self.is_throttling.load(Ordering::Relaxed)
    }

    /// Check if CPU monitoring is available on this platform.
    pub fn is_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/stat").is_ok()
        }
        #[cfg(target_os = "macos")]
        {
            true // Uses top -l 1
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            false
        }
    }

    /// Reset statistics (for testing).
    pub fn reset_stats(&self) {
        self.current_load_raw.store(0, Ordering::Relaxed);
        self.peak_load_raw.store(0, Ordering::Relaxed);
        self.is_throttling.store(false, Ordering::Relaxed);
        self.throttle_count.store(0, Ordering::Relaxed);
        self.samples_collected.store(0, Ordering::Relaxed);
        if let Ok(mut history) = self.load_history.write() {
            history.clear();
        }
    }

    /// Get CPU load (platform-specific implementation).
    #[cfg(target_os = "linux")]
    fn get_cpu_load(&self) -> Option<f64> {
        use std::fs;

        let content = fs::read_to_string("/proc/stat").ok()?;
        let line = content.lines().next()?;

        // Parse: cpu user nice system idle iowait irq softirq steal guest guest_nice
        let parts: Vec<u64> = line
            .split_whitespace()
            .skip(1) // Skip "cpu"
            .take(7)
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() < 4 {
            return None;
        }

        let idle = parts[3];
        let total: u64 = parts.iter().sum();

        // Get previous values
        let mut last_times = self.last_cpu_times.write().ok()?;

        let load = if let Some((last_idle, last_total)) = *last_times {
            let idle_delta = idle.saturating_sub(last_idle);
            let total_delta = total.saturating_sub(last_total);

            if total_delta > 0 {
                1.0 - (idle_delta as f64 / total_delta as f64)
            } else {
                0.0
            }
        } else {
            0.0
        };

        *last_times = Some((idle, total));

        Some(load.clamp(0.0, 1.0))
    }

    #[cfg(target_os = "macos")]
    fn get_cpu_load(&self) -> Option<f64> {
        use std::process::Command;

        // Use top -l 1 to get CPU usage
        let output = Command::new("top")
            .args(["-l", "1", "-n", "0"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse "CPU usage: X% user, Y% sys, Z% idle"
        for line in stdout.lines() {
            if line.contains("CPU usage:") {
                // Extract idle percentage
                if let Some(idle_start) = line.find("idle") {
                    let before_idle = &line[..idle_start];
                    let parts: Vec<&str> = before_idle.split_whitespace().collect();
                    if let Some(idle_str) = parts.last() {
                        let idle_str = idle_str.trim_end_matches('%').trim_end_matches(',');
                        if let Ok(idle) = idle_str.parse::<f64>() {
                            return Some((100.0 - idle) / 100.0);
                        }
                    }
                }
            }
        }

        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_cpu_load(&self) -> Option<f64> {
        None
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Simple system load check (returns average load if available).
#[cfg(unix)]
pub fn get_system_load() -> Option<f64> {
    use std::fs;

    // Try /proc/loadavg on Linux
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(load) = parts[0].parse::<f64>() {
                // Convert load average to percentage (assuming single core)
                // For multi-core, divide by number of cores
                let num_cpus = num_cpus::get() as f64;
                return Some((load / num_cpus).clamp(0.0, 1.0));
            }
        }
    }

    None
}

#[cfg(not(unix))]
pub fn get_system_load() -> Option<f64> {
    None
}

/// Get number of CPU cores (fallback for non-unix).
#[cfg(not(unix))]
mod num_cpus {
    pub fn get() -> usize {
        1
    }
}

#[cfg(unix)]
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_monitor_creation() {
        let config = CpuMonitorConfig::with_thresholds(0.80, 0.95);
        let monitor = CpuMonitor::new(config);
        assert!(monitor.is_enabled());
    }

    #[test]
    fn test_cpu_monitor_disabled() {
        let monitor = CpuMonitor::disabled();
        assert!(!monitor.is_enabled());
        assert!(monitor.check().is_ok());
    }

    #[test]
    fn test_stats_tracking() {
        let config = CpuMonitorConfig {
            enabled: true,
            sample_interval_ms: 0, // No delay for testing
            ..Default::default()
        };
        let monitor = CpuMonitor::new(config);

        // Sample a few times
        for _ in 0..5 {
            let _ = monitor.sample();
        }

        let stats = monitor.stats();
        // On supported platforms, we should have samples
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        assert!(stats.samples_collected > 0);
    }

    #[test]
    fn test_is_available() {
        #[cfg(target_os = "linux")]
        assert!(CpuMonitor::is_available());
    }

    #[test]
    fn test_throttling_flag() {
        let monitor = CpuMonitor::disabled();
        assert!(!monitor.is_throttling());
    }

    #[test]
    fn test_config_builders() {
        let config = CpuMonitorConfig::with_thresholds(0.7, 0.9).with_auto_throttle(100);
        assert!(config.auto_throttle);
        assert_eq!(config.throttle_delay_ms, 100);

        let config2 = CpuMonitorConfig::with_thresholds(0.7, 0.9).without_auto_throttle();
        assert!(!config2.auto_throttle);
    }
}
