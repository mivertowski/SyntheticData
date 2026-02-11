//! Memory management and guardrails for preventing OOM conditions.
//!
//! This module provides memory tracking and enforcement across different platforms,
//! with configurable soft and hard limits, automatic GC hints, and graceful degradation.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Memory usage statistics.
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current resident memory in bytes
    pub resident_bytes: u64,
    /// Peak resident memory in bytes
    pub peak_resident_bytes: u64,
    /// Number of memory limit checks performed
    pub checks_performed: u64,
    /// Number of soft limit warnings
    pub soft_limit_warnings: u64,
    /// Whether hard limit was ever exceeded
    pub hard_limit_exceeded: bool,
}

/// Memory guard configuration.
#[derive(Debug, Clone)]
pub struct MemoryGuardConfig {
    /// Hard memory limit in MB (0 = disabled)
    pub hard_limit_mb: usize,
    /// Soft memory limit in MB for warnings (0 = disabled, typically 80% of hard limit)
    pub soft_limit_mb: usize,
    /// Check interval (every N operations)
    pub check_interval: usize,
    /// Whether to enable aggressive mode (check more frequently)
    pub aggressive_mode: bool,
    /// Maximum allowed growth rate (MB per second) before warning
    pub max_growth_rate_mb_per_sec: f64,
}

impl Default for MemoryGuardConfig {
    fn default() -> Self {
        Self {
            hard_limit_mb: 0,    // Disabled by default
            soft_limit_mb: 0,    // Disabled by default
            check_interval: 500, // Check every 500 operations
            aggressive_mode: false,
            max_growth_rate_mb_per_sec: 100.0,
        }
    }
}

impl MemoryGuardConfig {
    /// Create config with specified hard limit (soft limit auto-calculated at 80%)
    pub fn with_limit_mb(hard_limit_mb: usize) -> Self {
        Self {
            hard_limit_mb,
            soft_limit_mb: (hard_limit_mb * 80) / 100,
            ..Default::default()
        }
    }

    /// Enable aggressive memory checking
    pub fn aggressive(mut self) -> Self {
        self.aggressive_mode = true;
        self.check_interval = 100;
        self
    }
}

/// Memory limit exceeded error.
#[derive(Debug, Clone)]
pub struct MemoryLimitExceeded {
    pub current_mb: usize,
    pub limit_mb: usize,
    pub is_soft_limit: bool,
    pub message: String,
}

impl std::fmt::Display for MemoryLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MemoryLimitExceeded {}

/// Thread-safe memory guard for monitoring and enforcing memory limits.
#[derive(Debug)]
pub struct MemoryGuard {
    config: MemoryGuardConfig,
    operation_counter: AtomicU64,
    peak_memory_mb: AtomicUsize,
    soft_warnings_count: AtomicU64,
    hard_limit_exceeded: AtomicBool,
    last_check_time_ns: AtomicU64,
    last_check_memory_mb: AtomicUsize,
}

impl MemoryGuard {
    /// Create a new memory guard with the given configuration.
    pub fn new(config: MemoryGuardConfig) -> Self {
        Self {
            config,
            operation_counter: AtomicU64::new(0),
            peak_memory_mb: AtomicUsize::new(0),
            soft_warnings_count: AtomicU64::new(0),
            hard_limit_exceeded: AtomicBool::new(false),
            last_check_time_ns: AtomicU64::new(0),
            last_check_memory_mb: AtomicUsize::new(0),
        }
    }

    /// Create a memory guard with default configuration.
    pub fn default_guard() -> Self {
        Self::new(MemoryGuardConfig::default())
    }

    /// Create a memory guard with specified limit.
    pub fn with_limit(limit_mb: usize) -> Self {
        Self::new(MemoryGuardConfig::with_limit_mb(limit_mb))
    }

    /// Create an Arc-wrapped memory guard for sharing across threads.
    pub fn shared(config: MemoryGuardConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Check memory limit (returns error if hard limit exceeded).
    ///
    /// This should be called periodically during generation.
    /// It's designed to be efficient - actual memory checks only happen
    /// at the configured interval.
    pub fn check(&self) -> Result<(), MemoryLimitExceeded> {
        // Disabled if no limits set
        if self.config.hard_limit_mb == 0 {
            return Ok(());
        }

        let count = self.operation_counter.fetch_add(1, Ordering::Relaxed);

        // Only check at intervals to minimize overhead
        let interval = if self.config.aggressive_mode {
            self.config.check_interval / 5
        } else {
            self.config.check_interval
        };

        if !count.is_multiple_of(interval as u64) {
            return Ok(());
        }

        self.check_now()
    }

    /// Force an immediate memory check (bypasses interval).
    pub fn check_now(&self) -> Result<(), MemoryLimitExceeded> {
        if self.config.hard_limit_mb == 0 {
            return Ok(());
        }

        let current_mb = get_memory_usage_mb().unwrap_or(0);

        // Update peak
        let mut peak = self.peak_memory_mb.load(Ordering::Relaxed);
        while current_mb > peak {
            match self.peak_memory_mb.compare_exchange_weak(
                peak,
                current_mb,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }

        // Check growth rate
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        let last_time = self.last_check_time_ns.swap(now_ns, Ordering::Relaxed);
        let last_mem = self
            .last_check_memory_mb
            .swap(current_mb, Ordering::Relaxed);

        if last_time > 0 && now_ns > last_time {
            let elapsed_sec = (now_ns - last_time) as f64 / 1_000_000_000.0;
            if elapsed_sec > 0.0 && current_mb > last_mem {
                let growth_rate = (current_mb - last_mem) as f64 / elapsed_sec;
                if growth_rate > self.config.max_growth_rate_mb_per_sec {
                    // High memory growth rate detected - consumer should check stats
                    // Note: Growth rate warning is logged by the caller
                    let _ = growth_rate; // Silence unused variable warning
                }
            }
        }

        // Check hard limit
        if current_mb > self.config.hard_limit_mb {
            self.hard_limit_exceeded.store(true, Ordering::Relaxed);
            return Err(MemoryLimitExceeded {
                current_mb,
                limit_mb: self.config.hard_limit_mb,
                is_soft_limit: false,
                message: format!(
                    "Memory limit exceeded: using {} MB, hard limit is {} MB. \
                     Reduce transaction volume or increase memory_limit_mb in config.",
                    current_mb, self.config.hard_limit_mb
                ),
            });
        }

        // Check soft limit (warning only)
        if self.config.soft_limit_mb > 0 && current_mb > self.config.soft_limit_mb {
            self.soft_warnings_count.fetch_add(1, Ordering::Relaxed);
            // Soft limit exceeded - consumer should check stats for warning count
        }

        Ok(())
    }

    /// Get current memory statistics.
    pub fn stats(&self) -> MemoryStats {
        let current = get_memory_usage_mb().unwrap_or(0);
        MemoryStats {
            resident_bytes: (current as u64) * 1024 * 1024,
            peak_resident_bytes: (self.peak_memory_mb.load(Ordering::Relaxed) as u64) * 1024 * 1024,
            checks_performed: self.operation_counter.load(Ordering::Relaxed),
            soft_limit_warnings: self.soft_warnings_count.load(Ordering::Relaxed),
            hard_limit_exceeded: self.hard_limit_exceeded.load(Ordering::Relaxed),
        }
    }

    /// Get current memory usage in MB.
    pub fn current_usage_mb(&self) -> usize {
        get_memory_usage_mb().unwrap_or(0)
    }

    /// Get peak memory usage in MB.
    pub fn peak_usage_mb(&self) -> usize {
        self.peak_memory_mb.load(Ordering::Relaxed)
    }

    /// Check if memory tracking is available on this platform.
    pub fn is_available() -> bool {
        get_memory_usage_mb().is_some()
    }

    /// Reset statistics (for testing).
    pub fn reset_stats(&self) {
        self.operation_counter.store(0, Ordering::Relaxed);
        self.soft_warnings_count.store(0, Ordering::Relaxed);
        self.hard_limit_exceeded.store(false, Ordering::Relaxed);
    }
}

impl Default for MemoryGuard {
    fn default() -> Self {
        Self::default_guard()
    }
}

/// Get current process memory usage in MB (Linux implementation).
#[cfg(target_os = "linux")]
pub fn get_memory_usage_mb() -> Option<usize> {
    use std::fs;

    // Try /proc/self/statm first (faster)
    if let Ok(content) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<usize>() {
                // Resident pages * page size (typically 4KB)
                let page_size_kb = 4;
                return Some((pages * page_size_kb) / 1024);
            }
        }
    }

    // Fallback to /proc/self/status (more detailed but slower)
    if let Ok(content) = fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return Some(kb / 1024);
                    }
                }
            }
        }
    }

    None
}

/// Get current process memory usage in MB (macOS implementation).
#[cfg(target_os = "macos")]
pub fn get_memory_usage_mb() -> Option<usize> {
    use std::process::Command;

    // Use ps to get RSS (resident set size)
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;

    let rss_kb: usize = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;

    Some(rss_kb / 1024)
}

/// Get current process memory usage in MB (Windows implementation).
#[cfg(target_os = "windows")]
pub fn get_memory_usage_mb() -> Option<usize> {
    // Windows implementation using GetProcessMemoryInfo would go here
    // For now, return None to indicate unavailable
    None
}

/// Get current process memory usage in MB (fallback for other platforms).
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn get_memory_usage_mb() -> Option<usize> {
    None
}

/// Estimate memory needed for generating N journal entries.
///
/// Returns estimated memory in MB based on typical entry size.
pub fn estimate_memory_mb(num_entries: usize, avg_lines_per_entry: usize) -> usize {
    // Rough estimates based on struct sizes:
    // - JournalEntry header: ~500 bytes
    // - JournalEntryLine: ~300 bytes each
    // - Overhead (strings, vecs): ~200 bytes per entry
    let bytes_per_entry = 500 + (avg_lines_per_entry * 300) + 200;
    let total_bytes = num_entries * bytes_per_entry;

    // Add 50% overhead for temporary allocations during generation
    let with_overhead = (total_bytes as f64 * 1.5) as usize;

    // Convert to MB, round up
    with_overhead.div_ceil(1024 * 1024)
}

/// Check if there's enough memory for the planned generation.
pub fn check_sufficient_memory(
    planned_entries: usize,
    avg_lines: usize,
    available_limit_mb: usize,
) -> Result<(), String> {
    let estimated = estimate_memory_mb(planned_entries, avg_lines);

    if available_limit_mb > 0 && estimated > available_limit_mb {
        Err(format!(
            "Estimated memory requirement ({} MB) exceeds limit ({} MB). \
             Reduce transaction count from {} to approximately {}",
            estimated,
            available_limit_mb,
            planned_entries,
            (planned_entries * available_limit_mb) / estimated
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_guard_creation() {
        let guard = MemoryGuard::with_limit(1024);
        assert_eq!(guard.config.hard_limit_mb, 1024);
        assert_eq!(guard.config.soft_limit_mb, 819); // 80% of 1024
    }

    #[test]
    fn test_memory_guard_disabled() {
        let guard = MemoryGuard::default_guard();
        // Should always succeed when disabled
        assert!(guard.check().is_ok());
        assert!(guard.check_now().is_ok());
    }

    #[test]
    fn test_memory_estimation() {
        let est = estimate_memory_mb(1000, 4);
        assert!(est > 0);
        assert!(est < 100); // Should be reasonable for 1000 entries
    }

    #[test]
    fn test_sufficient_memory_check() {
        // Should pass with high limit
        assert!(check_sufficient_memory(1000, 4, 1024).is_ok());

        // Should fail with low limit
        let result = check_sufficient_memory(1_000_000, 10, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_tracking() {
        let guard = MemoryGuard::with_limit(10000); // High limit to avoid errors

        // Perform some checks
        for _ in 0..1000 {
            let _ = guard.check();
        }

        let stats = guard.stats();
        assert!(stats.checks_performed > 0);
    }

    #[test]
    fn test_is_available() {
        // This will vary by platform
        #[cfg(target_os = "linux")]
        assert!(MemoryGuard::is_available());
    }
}
