//! Graceful degradation system for handling resource pressure.
//!
//! This module provides a degradation level system that allows the generator
//! to progressively reduce functionality when system resources become constrained,
//! rather than failing outright.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Degradation level indicating current system resource state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[derive(Default)]
pub enum DegradationLevel {
    /// Normal operation - all features enabled
    #[default]
    Normal = 0,
    /// Reduced operation - skip optional features, reduce batch sizes
    Reduced = 1,
    /// Minimal operation - essential data only, disable injections
    Minimal = 2,
    /// Emergency - flush and terminate gracefully
    Emergency = 3,
}

impl DegradationLevel {
    /// Check if data quality injection should be skipped at this level.
    pub fn skip_data_quality(&self) -> bool {
        *self >= DegradationLevel::Reduced
    }

    /// Check if anomaly injection should be skipped at this level.
    pub fn skip_anomaly_injection(&self) -> bool {
        *self >= DegradationLevel::Minimal
    }

    /// Check if optional fields should be omitted at this level.
    pub fn skip_optional_fields(&self) -> bool {
        *self >= DegradationLevel::Minimal
    }

    /// Check if immediate flush is required at this level.
    pub fn requires_immediate_flush(&self) -> bool {
        *self >= DegradationLevel::Emergency
    }

    /// Check if generation should terminate at this level.
    pub fn should_terminate(&self) -> bool {
        *self == DegradationLevel::Emergency
    }

    /// Get recommended batch size multiplier (1.0 = normal, 0.5 = half, etc.)
    pub fn batch_size_multiplier(&self) -> f64 {
        match self {
            DegradationLevel::Normal => 1.0,
            DegradationLevel::Reduced => 0.5,
            DegradationLevel::Minimal => 0.25,
            DegradationLevel::Emergency => 0.0,
        }
    }

    /// Get recommended anomaly injection rate multiplier.
    pub fn anomaly_rate_multiplier(&self) -> f64 {
        match self {
            DegradationLevel::Normal => 1.0,
            DegradationLevel::Reduced => 0.5,
            DegradationLevel::Minimal => 0.0,
            DegradationLevel::Emergency => 0.0,
        }
    }

    /// Get display name for this level.
    pub fn name(&self) -> &'static str {
        match self {
            DegradationLevel::Normal => "Normal",
            DegradationLevel::Reduced => "Reduced",
            DegradationLevel::Minimal => "Minimal",
            DegradationLevel::Emergency => "Emergency",
        }
    }

    /// Get description of what happens at this level.
    pub fn description(&self) -> &'static str {
        match self {
            DegradationLevel::Normal => "Full operation with all features enabled",
            DegradationLevel::Reduced => {
                "Reduced batch sizes, skip data quality injection, 50% anomaly rate"
            }
            DegradationLevel::Minimal => "Essential data only, no injections, minimal batch sizes",
            DegradationLevel::Emergency => {
                "Flush pending writes, save checkpoint, terminate gracefully"
            }
        }
    }

    /// Convert from u8.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => DegradationLevel::Normal,
            1 => DegradationLevel::Reduced,
            2 => DegradationLevel::Minimal,
            _ => DegradationLevel::Emergency,
        }
    }
}

impl std::fmt::Display for DegradationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration for degradation thresholds.
#[derive(Debug, Clone)]
pub struct DegradationConfig {
    /// Enable graceful degradation
    pub enabled: bool,
    /// Memory usage threshold for Reduced level (0.0 - 1.0)
    pub reduced_memory_threshold: f64,
    /// Memory usage threshold for Minimal level (0.0 - 1.0)
    pub minimal_memory_threshold: f64,
    /// Memory usage threshold for Emergency level (0.0 - 1.0)
    pub emergency_memory_threshold: f64,
    /// Disk space threshold for Reduced level (MB remaining)
    pub reduced_disk_threshold_mb: usize,
    /// Disk space threshold for Minimal level (MB remaining)
    pub minimal_disk_threshold_mb: usize,
    /// Disk space threshold for Emergency level (MB remaining)
    pub emergency_disk_threshold_mb: usize,
    /// CPU threshold for Reduced level (0.0 - 1.0)
    pub reduced_cpu_threshold: f64,
    /// CPU threshold for Minimal level (0.0 - 1.0)
    pub minimal_cpu_threshold: f64,
    /// Enable auto-recovery when resources improve
    pub auto_recovery: bool,
    /// Recovery hysteresis (must improve by this much before recovering)
    pub recovery_hysteresis: f64,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Memory thresholds (percentage of limit)
            reduced_memory_threshold: 0.70,
            minimal_memory_threshold: 0.85,
            emergency_memory_threshold: 0.95,
            // Disk thresholds (MB remaining)
            reduced_disk_threshold_mb: 1000,
            minimal_disk_threshold_mb: 500,
            emergency_disk_threshold_mb: 100,
            // CPU thresholds
            reduced_cpu_threshold: 0.80,
            minimal_cpu_threshold: 0.90,
            // Recovery
            auto_recovery: true,
            recovery_hysteresis: 0.05,
        }
    }
}

impl DegradationConfig {
    /// Create a conservative configuration (triggers earlier).
    pub fn conservative() -> Self {
        Self {
            reduced_memory_threshold: 0.60,
            minimal_memory_threshold: 0.75,
            emergency_memory_threshold: 0.90,
            reduced_disk_threshold_mb: 2000,
            minimal_disk_threshold_mb: 1000,
            emergency_disk_threshold_mb: 500,
            reduced_cpu_threshold: 0.70,
            minimal_cpu_threshold: 0.85,
            ..Default::default()
        }
    }

    /// Create an aggressive configuration (triggers later, maximizes throughput).
    pub fn aggressive() -> Self {
        Self {
            reduced_memory_threshold: 0.80,
            minimal_memory_threshold: 0.90,
            emergency_memory_threshold: 0.98,
            reduced_disk_threshold_mb: 500,
            minimal_disk_threshold_mb: 200,
            emergency_disk_threshold_mb: 50,
            reduced_cpu_threshold: 0.90,
            minimal_cpu_threshold: 0.95,
            ..Default::default()
        }
    }

    /// Disable graceful degradation.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// Current resource status for degradation decisions.
#[derive(Debug, Clone, Default)]
pub struct ResourceStatus {
    /// Memory usage as percentage of limit (0.0 - 1.0), None if no limit
    pub memory_usage: Option<f64>,
    /// Available disk space in MB
    pub disk_available_mb: Option<usize>,
    /// CPU load (0.0 - 1.0)
    pub cpu_load: Option<f64>,
}

impl ResourceStatus {
    /// Create status from individual measurements.
    pub fn new(
        memory_usage: Option<f64>,
        disk_available_mb: Option<usize>,
        cpu_load: Option<f64>,
    ) -> Self {
        Self {
            memory_usage,
            disk_available_mb,
            cpu_load,
        }
    }
}

/// Thread-safe degradation controller.
#[derive(Debug)]
pub struct DegradationController {
    config: DegradationConfig,
    current_level: AtomicU8,
    level_change_count: std::sync::atomic::AtomicU64,
}

impl DegradationController {
    /// Create a new degradation controller with the given configuration.
    pub fn new(config: DegradationConfig) -> Self {
        Self {
            config,
            current_level: AtomicU8::new(DegradationLevel::Normal as u8),
            level_change_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Create a controller with default configuration.
    pub fn default_controller() -> Self {
        Self::new(DegradationConfig::default())
    }

    /// Create a disabled controller (always returns Normal).
    pub fn disabled() -> Self {
        Self::new(DegradationConfig::disabled())
    }

    /// Create an Arc-wrapped controller for sharing across threads.
    pub fn shared(config: DegradationConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Get current degradation level.
    pub fn current_level(&self) -> DegradationLevel {
        DegradationLevel::from_u8(self.current_level.load(Ordering::Relaxed))
    }

    /// Update degradation level based on resource status.
    /// Returns the new level and whether it changed.
    pub fn update(&self, status: &ResourceStatus) -> (DegradationLevel, bool) {
        if !self.config.enabled {
            return (DegradationLevel::Normal, false);
        }

        let new_level = self.calculate_level(status);
        let old_level = self.current_level.swap(new_level as u8, Ordering::Relaxed);
        let changed = old_level != new_level as u8;

        if changed {
            self.level_change_count.fetch_add(1, Ordering::Relaxed);
        }

        (new_level, changed)
    }

    /// Calculate appropriate degradation level based on resource status.
    fn calculate_level(&self, status: &ResourceStatus) -> DegradationLevel {
        let current = self.current_level();

        // Check each resource type and find the highest degradation level needed
        let mut level = DegradationLevel::Normal;

        // Memory check
        if let Some(mem_usage) = status.memory_usage {
            let mem_level = if mem_usage >= self.config.emergency_memory_threshold {
                DegradationLevel::Emergency
            } else if mem_usage >= self.config.minimal_memory_threshold {
                DegradationLevel::Minimal
            } else if mem_usage >= self.config.reduced_memory_threshold {
                DegradationLevel::Reduced
            } else {
                DegradationLevel::Normal
            };
            level = level.max(mem_level);
        }

        // Disk check
        if let Some(disk_mb) = status.disk_available_mb {
            let disk_level = if disk_mb <= self.config.emergency_disk_threshold_mb {
                DegradationLevel::Emergency
            } else if disk_mb <= self.config.minimal_disk_threshold_mb {
                DegradationLevel::Minimal
            } else if disk_mb <= self.config.reduced_disk_threshold_mb {
                DegradationLevel::Reduced
            } else {
                DegradationLevel::Normal
            };
            level = level.max(disk_level);
        }

        // CPU check
        if let Some(cpu) = status.cpu_load {
            let cpu_level = if cpu >= self.config.minimal_cpu_threshold {
                DegradationLevel::Minimal
            } else if cpu >= self.config.reduced_cpu_threshold {
                DegradationLevel::Reduced
            } else {
                DegradationLevel::Normal
            };
            // CPU doesn't trigger Emergency - only memory and disk do
            level = level.max(cpu_level);
        }

        // Apply hysteresis for recovery (only allow stepping down one level at a time)
        if self.config.auto_recovery && level < current {
            // Allow recovery only if significantly improved
            let can_recover = if let Some(mem) = status.memory_usage {
                match current {
                    DegradationLevel::Emergency => {
                        mem < self.config.emergency_memory_threshold
                            - self.config.recovery_hysteresis
                    }
                    DegradationLevel::Minimal => {
                        mem < self.config.minimal_memory_threshold - self.config.recovery_hysteresis
                    }
                    DegradationLevel::Reduced => {
                        mem < self.config.reduced_memory_threshold - self.config.recovery_hysteresis
                    }
                    DegradationLevel::Normal => true,
                }
            } else {
                true
            };

            if can_recover {
                // Step down one level at a time for smooth recovery
                level = level.max(match current {
                    DegradationLevel::Emergency => DegradationLevel::Minimal,
                    DegradationLevel::Minimal => DegradationLevel::Reduced,
                    _ => DegradationLevel::Normal,
                });
            } else {
                level = current;
            }
        }

        level
    }

    /// Force a specific degradation level (for testing or manual intervention).
    pub fn force_level(&self, level: DegradationLevel) {
        self.current_level.store(level as u8, Ordering::Relaxed);
        self.level_change_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset to Normal level.
    pub fn reset(&self) {
        self.current_level
            .store(DegradationLevel::Normal as u8, Ordering::Relaxed);
    }

    /// Get number of level changes.
    pub fn level_change_count(&self) -> u64 {
        self.level_change_count.load(Ordering::Relaxed)
    }

    /// Check if currently degraded (not Normal).
    pub fn is_degraded(&self) -> bool {
        self.current_level() != DegradationLevel::Normal
    }

    /// Get the configuration.
    pub fn config(&self) -> &DegradationConfig {
        &self.config
    }
}

impl Default for DegradationController {
    fn default() -> Self {
        Self::default_controller()
    }
}

/// Actions to take at each degradation level.
#[derive(Debug, Clone)]
pub struct DegradationActions {
    /// Skip data quality injection
    pub skip_data_quality: bool,
    /// Skip anomaly injection
    pub skip_anomaly_injection: bool,
    /// Skip optional fields in output
    pub skip_optional_fields: bool,
    /// Reduce batch size by this factor
    pub batch_size_factor: f64,
    /// Reduce anomaly injection rate by this factor
    pub anomaly_rate_factor: f64,
    /// Use compact output format
    pub use_compact_output: bool,
    /// Flush output immediately after each batch
    pub immediate_flush: bool,
    /// Terminate generation
    pub terminate: bool,
}

impl DegradationActions {
    /// Get actions for a given degradation level.
    pub fn for_level(level: DegradationLevel) -> Self {
        match level {
            DegradationLevel::Normal => Self {
                skip_data_quality: false,
                skip_anomaly_injection: false,
                skip_optional_fields: false,
                batch_size_factor: 1.0,
                anomaly_rate_factor: 1.0,
                use_compact_output: false,
                immediate_flush: false,
                terminate: false,
            },
            DegradationLevel::Reduced => Self {
                skip_data_quality: true,
                skip_anomaly_injection: false,
                skip_optional_fields: false,
                batch_size_factor: 0.5,
                anomaly_rate_factor: 0.5,
                use_compact_output: true,
                immediate_flush: false,
                terminate: false,
            },
            DegradationLevel::Minimal => Self {
                skip_data_quality: true,
                skip_anomaly_injection: true,
                skip_optional_fields: true,
                batch_size_factor: 0.25,
                anomaly_rate_factor: 0.0,
                use_compact_output: true,
                immediate_flush: true,
                terminate: false,
            },
            DegradationLevel::Emergency => Self {
                skip_data_quality: true,
                skip_anomaly_injection: true,
                skip_optional_fields: true,
                batch_size_factor: 0.0,
                anomaly_rate_factor: 0.0,
                use_compact_output: true,
                immediate_flush: true,
                terminate: true,
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_degradation_level_ordering() {
        assert!(DegradationLevel::Normal < DegradationLevel::Reduced);
        assert!(DegradationLevel::Reduced < DegradationLevel::Minimal);
        assert!(DegradationLevel::Minimal < DegradationLevel::Emergency);
    }

    #[test]
    fn test_level_behavior_flags() {
        assert!(!DegradationLevel::Normal.skip_data_quality());
        assert!(DegradationLevel::Reduced.skip_data_quality());
        assert!(DegradationLevel::Minimal.skip_data_quality());

        assert!(!DegradationLevel::Normal.skip_anomaly_injection());
        assert!(!DegradationLevel::Reduced.skip_anomaly_injection());
        assert!(DegradationLevel::Minimal.skip_anomaly_injection());

        assert!(!DegradationLevel::Minimal.should_terminate());
        assert!(DegradationLevel::Emergency.should_terminate());
    }

    #[test]
    fn test_controller_creation() {
        let controller = DegradationController::default_controller();
        assert_eq!(controller.current_level(), DegradationLevel::Normal);
    }

    #[test]
    fn test_controller_disabled() {
        let controller = DegradationController::disabled();
        let status = ResourceStatus::new(Some(0.99), Some(10), Some(0.99));
        let (level, _) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Normal);
    }

    #[test]
    fn test_memory_degradation() {
        let controller = DegradationController::default_controller();

        // High memory usage should trigger Reduced
        let status = ResourceStatus::new(Some(0.75), None, None);
        let (level, changed) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Reduced);
        assert!(changed);

        // Very high memory usage should trigger Minimal
        let status = ResourceStatus::new(Some(0.90), None, None);
        let (level, _) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Minimal);

        // Critical memory usage should trigger Emergency
        let status = ResourceStatus::new(Some(0.96), None, None);
        let (level, _) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Emergency);
    }

    #[test]
    fn test_disk_degradation() {
        let controller = DegradationController::default_controller();

        // Low disk space should trigger Reduced
        let status = ResourceStatus::new(None, Some(800), None);
        let (level, _) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Reduced);

        // Very low disk space should trigger Emergency
        let status = ResourceStatus::new(None, Some(50), None);
        let (level, _) = controller.update(&status);
        assert_eq!(level, DegradationLevel::Emergency);
    }

    #[test]
    fn test_force_level() {
        let controller = DegradationController::default_controller();
        controller.force_level(DegradationLevel::Minimal);
        assert_eq!(controller.current_level(), DegradationLevel::Minimal);
    }

    #[test]
    fn test_level_change_count() {
        let controller = DegradationController::default_controller();
        assert_eq!(controller.level_change_count(), 0);

        controller.force_level(DegradationLevel::Reduced);
        assert_eq!(controller.level_change_count(), 1);

        controller.force_level(DegradationLevel::Normal);
        assert_eq!(controller.level_change_count(), 2);
    }

    #[test]
    fn test_actions_for_level() {
        let normal_actions = DegradationActions::for_level(DegradationLevel::Normal);
        assert!(!normal_actions.skip_data_quality);
        assert!(!normal_actions.terminate);
        assert_eq!(normal_actions.batch_size_factor, 1.0);

        let emergency_actions = DegradationActions::for_level(DegradationLevel::Emergency);
        assert!(emergency_actions.skip_data_quality);
        assert!(emergency_actions.terminate);
        assert_eq!(emergency_actions.batch_size_factor, 0.0);
    }
}
