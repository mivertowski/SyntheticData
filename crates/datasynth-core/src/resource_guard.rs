//! Unified resource guard combining memory, disk, and CPU monitoring.
//!
//! This module provides a single interface for checking all system resources
//! and coordinating graceful degradation when resources become constrained.

use std::path::Path;
use std::sync::Arc;

use crate::cpu_monitor::{CpuMonitor, CpuMonitorConfig, CpuStats};
use crate::degradation::{
    DegradationActions, DegradationConfig, DegradationController, DegradationLevel, ResourceStatus,
};
use crate::disk_guard::{DiskSpaceGuard, DiskSpaceGuardConfig, DiskStats};
use crate::error::{SynthError, SynthResult};
use crate::memory_guard::{MemoryGuard, MemoryGuardConfig, MemoryStats};

/// Combined resource statistics.
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    /// Memory statistics
    pub memory: MemoryStats,
    /// Disk space statistics
    pub disk: DiskStats,
    /// CPU statistics
    pub cpu: CpuStats,
    /// Current degradation level
    pub degradation_level: DegradationLevel,
    /// Number of resource checks performed
    pub checks_performed: u64,
}

/// Configuration for the unified resource guard.
#[derive(Debug, Clone)]
pub struct ResourceGuardConfig {
    /// Memory guard configuration
    pub memory: MemoryGuardConfig,
    /// Disk space guard configuration
    pub disk: DiskSpaceGuardConfig,
    /// CPU monitor configuration
    pub cpu: CpuMonitorConfig,
    /// Degradation configuration
    pub degradation: DegradationConfig,
    /// Check interval (every N operations)
    pub check_interval: usize,
}

impl Default for ResourceGuardConfig {
    fn default() -> Self {
        Self {
            memory: MemoryGuardConfig::default(),
            disk: DiskSpaceGuardConfig::default(),
            cpu: CpuMonitorConfig::default(),
            degradation: DegradationConfig::default(),
            check_interval: 500,
        }
    }
}

impl ResourceGuardConfig {
    /// Create config with specified memory limit.
    pub fn with_memory_limit(limit_mb: usize) -> Self {
        Self {
            memory: MemoryGuardConfig::with_limit_mb(limit_mb),
            ..Default::default()
        }
    }

    /// Set output path for disk monitoring.
    pub fn with_output_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.disk.monitor_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable CPU monitoring with thresholds.
    pub fn with_cpu_monitoring(mut self, high_threshold: f64, critical_threshold: f64) -> Self {
        self.cpu.enabled = true;
        self.cpu.high_load_threshold = high_threshold;
        self.cpu.critical_load_threshold = critical_threshold;
        self
    }

    /// Use conservative degradation thresholds.
    pub fn conservative(mut self) -> Self {
        self.degradation = DegradationConfig::conservative();
        self
    }

    /// Use aggressive degradation thresholds.
    pub fn aggressive(mut self) -> Self {
        self.degradation = DegradationConfig::aggressive();
        self
    }

    /// Disable all monitoring (for testing or when resources are managed externally).
    pub fn disabled() -> Self {
        Self {
            memory: MemoryGuardConfig {
                hard_limit_mb: 0,
                ..Default::default()
            },
            disk: DiskSpaceGuardConfig {
                hard_limit_mb: 0,
                ..Default::default()
            },
            cpu: CpuMonitorConfig {
                enabled: false,
                ..Default::default()
            },
            degradation: DegradationConfig::disabled(),
            check_interval: 1000,
        }
    }
}

/// Unified resource guard for monitoring all system resources.
#[derive(Debug)]
pub struct ResourceGuard {
    config: ResourceGuardConfig,
    memory_guard: MemoryGuard,
    disk_guard: DiskSpaceGuard,
    cpu_monitor: CpuMonitor,
    degradation_controller: DegradationController,
    check_counter: std::sync::atomic::AtomicU64,
}

impl ResourceGuard {
    /// Create a new resource guard with the given configuration.
    pub fn new(config: ResourceGuardConfig) -> Self {
        Self {
            memory_guard: MemoryGuard::new(config.memory.clone()),
            disk_guard: DiskSpaceGuard::new(config.disk.clone()),
            cpu_monitor: CpuMonitor::new(config.cpu.clone()),
            degradation_controller: DegradationController::new(config.degradation.clone()),
            check_counter: std::sync::atomic::AtomicU64::new(0),
            config,
        }
    }

    /// Create a resource guard with default configuration.
    pub fn default_guard() -> Self {
        Self::new(ResourceGuardConfig::default())
    }

    /// Create a resource guard with specified memory limit.
    pub fn with_memory_limit(limit_mb: usize) -> Self {
        Self::new(ResourceGuardConfig::with_memory_limit(limit_mb))
    }

    /// Create a disabled resource guard.
    pub fn disabled() -> Self {
        Self::new(ResourceGuardConfig::disabled())
    }

    /// Create an Arc-wrapped resource guard for sharing across threads.
    pub fn shared(config: ResourceGuardConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Check all resources (memory, disk, CPU).
    /// Returns Ok with current degradation level or Err if hard limits exceeded.
    pub fn check(&self) -> SynthResult<DegradationLevel> {
        let count = self
            .check_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Only perform actual checks at intervals
        if !count.is_multiple_of(self.config.check_interval as u64) {
            return Ok(self.degradation_controller.current_level());
        }

        self.check_now()
    }

    /// Force an immediate check of all resources (bypasses interval).
    pub fn check_now(&self) -> SynthResult<DegradationLevel> {
        // Check memory
        if let Err(e) = self.memory_guard.check_now() {
            return Err(SynthError::memory_exhausted(e.current_mb, e.limit_mb));
        }

        // Check disk
        if let Err(e) = self.disk_guard.check_now() {
            return Err(SynthError::disk_exhausted(e.available_mb, e.required_mb));
        }

        // Sample CPU
        let _ = self.cpu_monitor.sample();

        // Update degradation level
        let status = self.build_resource_status();
        let (level, _changed) = self.degradation_controller.update(&status);

        // If at Emergency level, return error to trigger graceful shutdown
        if level == DegradationLevel::Emergency {
            return Err(SynthError::degradation(
                level.name(),
                "Resource limits critically exceeded, initiating graceful shutdown",
            ));
        }

        Ok(level)
    }

    /// Build resource status for degradation calculation.
    fn build_resource_status(&self) -> ResourceStatus {
        let memory_usage = if self.config.memory.hard_limit_mb > 0 {
            let current = self.memory_guard.current_usage_mb();
            Some(current as f64 / self.config.memory.hard_limit_mb as f64)
        } else {
            None
        };

        let disk_available = if self.config.disk.hard_limit_mb > 0 {
            Some(self.disk_guard.available_space_mb())
        } else {
            None
        };

        let cpu_load = if self.cpu_monitor.is_enabled() {
            Some(self.cpu_monitor.current_load())
        } else {
            None
        };

        ResourceStatus::new(memory_usage, disk_available, cpu_load)
    }

    /// Get actions to take based on current degradation level.
    pub fn get_actions(&self) -> DegradationActions {
        DegradationActions::for_level(self.degradation_controller.current_level())
    }

    /// Check if currently degraded (not Normal).
    pub fn is_degraded(&self) -> bool {
        self.degradation_controller.is_degraded()
    }

    /// Get current degradation level.
    pub fn degradation_level(&self) -> DegradationLevel {
        self.degradation_controller.current_level()
    }

    /// Get combined resource statistics.
    pub fn stats(&self) -> ResourceStats {
        ResourceStats {
            memory: self.memory_guard.stats(),
            disk: self.disk_guard.stats(),
            cpu: self.cpu_monitor.stats(),
            degradation_level: self.degradation_controller.current_level(),
            checks_performed: self
                .check_counter
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Pre-check before a potentially expensive operation.
    /// Returns recommended action based on current resource state.
    pub fn pre_check(&self) -> PreCheckResult {
        let level = self.degradation_controller.current_level();
        let actions = DegradationActions::for_level(level);

        if actions.terminate {
            PreCheckResult::Abort("Resources critically low, cannot proceed")
        } else if actions.immediate_flush {
            PreCheckResult::ProceedWithCaution("Resources constrained, reduce batch size")
        } else if level != DegradationLevel::Normal {
            PreCheckResult::Reduced("Operating in degraded mode")
        } else {
            PreCheckResult::Proceed
        }
    }

    /// Pre-check before writing data.
    pub fn check_before_write(&self, estimated_bytes: u64) -> SynthResult<()> {
        self.disk_guard
            .check_before_write(estimated_bytes)
            .map_err(|e| SynthError::disk_exhausted(e.available_mb, e.required_mb))
    }

    /// Record bytes written (for tracking).
    pub fn record_write(&self, bytes: u64) {
        self.disk_guard.record_write(bytes);
    }

    /// Get reference to memory guard.
    pub fn memory(&self) -> &MemoryGuard {
        &self.memory_guard
    }

    /// Get reference to disk guard.
    pub fn disk(&self) -> &DiskSpaceGuard {
        &self.disk_guard
    }

    /// Get reference to CPU monitor.
    pub fn cpu(&self) -> &CpuMonitor {
        &self.cpu_monitor
    }

    /// Get reference to degradation controller.
    pub fn degradation(&self) -> &DegradationController {
        &self.degradation_controller
    }

    /// Apply throttle delay if CPU is overloaded.
    pub fn maybe_throttle(&self) {
        self.cpu_monitor.maybe_throttle();
    }

    /// Reset all statistics (for testing).
    pub fn reset_stats(&self) {
        self.memory_guard.reset_stats();
        self.disk_guard.reset_stats();
        self.cpu_monitor.reset_stats();
        self.degradation_controller.reset();
        self.check_counter
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if resource monitoring is available on this platform.
    pub fn is_available() -> bool {
        MemoryGuard::is_available() || DiskSpaceGuard::is_available() || CpuMonitor::is_available()
    }

    /// Get current memory usage in MB.
    pub fn current_memory_mb(&self) -> usize {
        self.memory_guard.current_usage_mb()
    }

    /// Get current available disk space in MB.
    pub fn available_disk_mb(&self) -> usize {
        self.disk_guard.available_space_mb()
    }

    /// Get current CPU load.
    pub fn current_cpu_load(&self) -> f64 {
        self.cpu_monitor.current_load()
    }
}

impl Default for ResourceGuard {
    fn default() -> Self {
        Self::default_guard()
    }
}

/// Result of a pre-check before an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreCheckResult {
    /// Proceed normally
    Proceed,
    /// Proceed with reduced functionality
    Reduced(&'static str),
    /// Proceed but with caution (flush more frequently, reduce batch size)
    ProceedWithCaution(&'static str),
    /// Abort the operation
    Abort(&'static str),
}

impl PreCheckResult {
    /// Check if operation should proceed (any variant except Abort).
    pub fn should_proceed(&self) -> bool {
        !matches!(self, PreCheckResult::Abort(_))
    }

    /// Get the message if any.
    pub fn message(&self) -> Option<&'static str> {
        match self {
            PreCheckResult::Proceed => None,
            PreCheckResult::Reduced(msg) => Some(msg),
            PreCheckResult::ProceedWithCaution(msg) => Some(msg),
            PreCheckResult::Abort(msg) => Some(msg),
        }
    }
}

/// Builder for creating a ResourceGuard with a fluent interface.
#[derive(Debug, Clone, Default)]
pub struct ResourceGuardBuilder {
    config: ResourceGuardConfig,
}

impl ResourceGuardBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set memory limit in MB.
    pub fn memory_limit(mut self, limit_mb: usize) -> Self {
        self.config.memory = MemoryGuardConfig::with_limit_mb(limit_mb);
        self
    }

    /// Set minimum free disk space in MB.
    pub fn min_free_disk(mut self, min_free_mb: usize) -> Self {
        self.config.disk = DiskSpaceGuardConfig::with_min_free_mb(min_free_mb);
        self
    }

    /// Set output path for disk monitoring.
    pub fn output_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.disk.monitor_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable CPU monitoring.
    pub fn cpu_monitoring(mut self, high_threshold: f64, critical_threshold: f64) -> Self {
        self.config.cpu.enabled = true;
        self.config.cpu.high_load_threshold = high_threshold;
        self.config.cpu.critical_load_threshold = critical_threshold;
        self
    }

    /// Enable auto-throttling when CPU is overloaded.
    pub fn auto_throttle(mut self, delay_ms: u64) -> Self {
        self.config.cpu.auto_throttle = true;
        self.config.cpu.throttle_delay_ms = delay_ms;
        self
    }

    /// Set degradation thresholds.
    pub fn degradation_config(mut self, config: DegradationConfig) -> Self {
        self.config.degradation = config;
        self
    }

    /// Use conservative degradation settings.
    pub fn conservative(mut self) -> Self {
        self.config.degradation = DegradationConfig::conservative();
        self
    }

    /// Use aggressive degradation settings.
    pub fn aggressive(mut self) -> Self {
        self.config.degradation = DegradationConfig::aggressive();
        self
    }

    /// Set check interval.
    pub fn check_interval(mut self, interval: usize) -> Self {
        self.config.check_interval = interval;
        self
    }

    /// Build the ResourceGuard.
    pub fn build(self) -> ResourceGuard {
        ResourceGuard::new(self.config)
    }

    /// Build an Arc-wrapped ResourceGuard.
    pub fn build_shared(self) -> Arc<ResourceGuard> {
        Arc::new(ResourceGuard::new(self.config))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_guard_creation() {
        let guard = ResourceGuard::with_memory_limit(1024);
        assert_eq!(guard.config.memory.hard_limit_mb, 1024);
    }

    #[test]
    fn test_resource_guard_disabled() {
        let guard = ResourceGuard::disabled();
        assert!(guard.check().is_ok());
        assert_eq!(guard.degradation_level(), DegradationLevel::Normal);
    }

    #[test]
    fn test_builder() {
        let guard = ResourceGuardBuilder::new()
            .memory_limit(2048)
            .min_free_disk(500)
            .cpu_monitoring(0.8, 0.95)
            .conservative()
            .build();

        assert_eq!(guard.config.memory.hard_limit_mb, 2048);
        assert_eq!(guard.config.disk.hard_limit_mb, 500);
        assert!(guard.config.cpu.enabled);
    }

    #[test]
    fn test_pre_check() {
        let guard = ResourceGuard::disabled();
        let result = guard.pre_check();
        assert!(result.should_proceed());
        assert_eq!(result, PreCheckResult::Proceed);
    }

    #[test]
    fn test_stats() {
        let guard = ResourceGuard::default_guard();
        let stats = guard.stats();
        assert_eq!(stats.degradation_level, DegradationLevel::Normal);
    }

    #[test]
    fn test_pre_check_messages() {
        assert!(PreCheckResult::Proceed.message().is_none());
        assert!(PreCheckResult::Abort("test").message().is_some());
    }

    #[test]
    fn test_is_available() {
        // Should be true on at least one of the monitored resources
        #[cfg(unix)]
        assert!(ResourceGuard::is_available());
    }
}
