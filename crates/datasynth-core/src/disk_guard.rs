//! Disk space management and guardrails for preventing disk exhaustion.
//!
//! This module provides disk space tracking and enforcement across different platforms,
//! with configurable minimum free space limits and pre-write checks.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Disk space usage statistics.
#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    /// Total disk space in bytes
    pub total_bytes: u64,
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Used disk space in bytes
    pub used_bytes: u64,
    /// Number of disk space checks performed
    pub checks_performed: u64,
    /// Number of soft limit warnings
    pub soft_limit_warnings: u64,
    /// Whether hard limit was ever exceeded
    pub hard_limit_exceeded: bool,
    /// Estimated bytes written this session
    pub estimated_bytes_written: u64,
}

/// Disk space guard configuration.
#[derive(Debug, Clone)]
pub struct DiskSpaceGuardConfig {
    /// Minimum free space required in MB (hard limit)
    pub hard_limit_mb: usize,
    /// Warning threshold in MB (soft limit)
    pub soft_limit_mb: usize,
    /// Check interval (every N write operations)
    pub check_interval: usize,
    /// Reserve buffer to maintain in MB
    pub reserve_buffer_mb: usize,
    /// Path to monitor (defaults to output directory)
    pub monitor_path: Option<PathBuf>,
}

impl Default for DiskSpaceGuardConfig {
    fn default() -> Self {
        Self {
            hard_limit_mb: 100,    // Require at least 100 MB free
            soft_limit_mb: 500,    // Warn when below 500 MB
            check_interval: 500,   // Check every 500 operations
            reserve_buffer_mb: 50, // Keep 50 MB buffer
            monitor_path: None,
        }
    }
}

impl DiskSpaceGuardConfig {
    /// Create config with specified minimum free space.
    pub fn with_min_free_mb(hard_limit_mb: usize) -> Self {
        Self {
            hard_limit_mb,
            soft_limit_mb: hard_limit_mb * 5, // Soft limit at 5x hard limit
            ..Default::default()
        }
    }

    /// Set the path to monitor for disk space.
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.monitor_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the reserve buffer.
    pub fn with_reserve(mut self, reserve_mb: usize) -> Self {
        self.reserve_buffer_mb = reserve_mb;
        self
    }
}

/// Disk space limit exceeded error.
#[derive(Debug, Clone)]
pub struct DiskSpaceExhausted {
    pub available_mb: usize,
    pub required_mb: usize,
    pub is_soft_limit: bool,
    pub message: String,
}

impl std::fmt::Display for DiskSpaceExhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DiskSpaceExhausted {}

/// Thread-safe disk space guard for monitoring and enforcing disk limits.
#[derive(Debug)]
pub struct DiskSpaceGuard {
    config: DiskSpaceGuardConfig,
    operation_counter: AtomicU64,
    soft_warnings_count: AtomicU64,
    hard_limit_exceeded: AtomicBool,
    bytes_written_estimate: AtomicU64,
    last_available_mb: AtomicUsize,
}

impl DiskSpaceGuard {
    /// Create a new disk space guard with the given configuration.
    pub fn new(config: DiskSpaceGuardConfig) -> Self {
        Self {
            config,
            operation_counter: AtomicU64::new(0),
            soft_warnings_count: AtomicU64::new(0),
            hard_limit_exceeded: AtomicBool::new(false),
            bytes_written_estimate: AtomicU64::new(0),
            last_available_mb: AtomicUsize::new(0),
        }
    }

    /// Create a disk space guard with default configuration.
    pub fn default_guard() -> Self {
        Self::new(DiskSpaceGuardConfig::default())
    }

    /// Create a disk space guard with specified minimum free space.
    pub fn with_min_free(min_free_mb: usize) -> Self {
        Self::new(DiskSpaceGuardConfig::with_min_free_mb(min_free_mb))
    }

    /// Create an Arc-wrapped disk space guard for sharing across threads.
    pub fn shared(config: DiskSpaceGuardConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Check disk space limit (returns error if hard limit exceeded).
    ///
    /// This should be called periodically during file writes.
    /// It's designed to be efficient - actual disk checks only happen
    /// at the configured interval.
    pub fn check(&self) -> Result<(), DiskSpaceExhausted> {
        // Disabled if no limits set
        if self.config.hard_limit_mb == 0 {
            return Ok(());
        }

        let count = self.operation_counter.fetch_add(1, Ordering::Relaxed);

        // Only check at intervals to minimize overhead
        if !count.is_multiple_of(self.config.check_interval as u64) {
            return Ok(());
        }

        self.check_now()
    }

    /// Force an immediate disk space check (bypasses interval).
    pub fn check_now(&self) -> Result<(), DiskSpaceExhausted> {
        if self.config.hard_limit_mb == 0 {
            return Ok(());
        }

        let path = self
            .config
            .monitor_path
            .as_deref()
            .unwrap_or(Path::new("."));

        let available_mb = get_available_space_mb(path).unwrap_or(usize::MAX);
        self.last_available_mb
            .store(available_mb, Ordering::Relaxed);

        // Check hard limit (minimum free space required)
        let required_mb = self.config.hard_limit_mb + self.config.reserve_buffer_mb;
        if available_mb < required_mb {
            self.hard_limit_exceeded.store(true, Ordering::Relaxed);
            return Err(DiskSpaceExhausted {
                available_mb,
                required_mb,
                is_soft_limit: false,
                message: format!(
                    "Disk space exhausted: only {} MB available, need at least {} MB. \
                     Free up disk space or reduce output volume.",
                    available_mb, required_mb
                ),
            });
        }

        // Check soft limit (warning only)
        if available_mb < self.config.soft_limit_mb {
            self.soft_warnings_count.fetch_add(1, Ordering::Relaxed);
            // Soft limit exceeded - consumer should check stats for warning count
        }

        Ok(())
    }

    /// Pre-check if there's enough space for an estimated write.
    pub fn check_before_write(&self, estimated_bytes: u64) -> Result<(), DiskSpaceExhausted> {
        if self.config.hard_limit_mb == 0 {
            return Ok(());
        }

        let path = self
            .config
            .monitor_path
            .as_deref()
            .unwrap_or(Path::new("."));

        let available_mb = get_available_space_mb(path).unwrap_or(usize::MAX);
        let estimated_mb = (estimated_bytes / (1024 * 1024)) as usize;
        let required_mb = self.config.hard_limit_mb + self.config.reserve_buffer_mb + estimated_mb;

        if available_mb < required_mb {
            return Err(DiskSpaceExhausted {
                available_mb,
                required_mb,
                is_soft_limit: false,
                message: format!(
                    "Insufficient disk space for write: {} MB available, need {} MB \
                     (estimated write: {} MB, reserve: {} MB).",
                    available_mb, required_mb, estimated_mb, self.config.reserve_buffer_mb
                ),
            });
        }

        Ok(())
    }

    /// Record bytes written (for estimation).
    pub fn record_write(&self, bytes: u64) {
        self.bytes_written_estimate
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get current disk space statistics.
    pub fn stats(&self) -> DiskStats {
        let path = self
            .config
            .monitor_path
            .as_deref()
            .unwrap_or(Path::new("."));

        let (total, available) = get_disk_space(path).unwrap_or((0, 0));

        DiskStats {
            total_bytes: total,
            available_bytes: available,
            used_bytes: total.saturating_sub(available),
            checks_performed: self.operation_counter.load(Ordering::Relaxed),
            soft_limit_warnings: self.soft_warnings_count.load(Ordering::Relaxed),
            hard_limit_exceeded: self.hard_limit_exceeded.load(Ordering::Relaxed),
            estimated_bytes_written: self.bytes_written_estimate.load(Ordering::Relaxed),
        }
    }

    /// Get current available space in MB.
    pub fn available_space_mb(&self) -> usize {
        let path = self
            .config
            .monitor_path
            .as_deref()
            .unwrap_or(Path::new("."));
        get_available_space_mb(path).unwrap_or(0)
    }

    /// Check if disk space tracking is available on this platform.
    pub fn is_available() -> bool {
        get_available_space_mb(Path::new(".")).is_some()
    }

    /// Reset statistics (for testing).
    pub fn reset_stats(&self) {
        self.operation_counter.store(0, Ordering::Relaxed);
        self.soft_warnings_count.store(0, Ordering::Relaxed);
        self.hard_limit_exceeded.store(false, Ordering::Relaxed);
        self.bytes_written_estimate.store(0, Ordering::Relaxed);
    }
}

impl Default for DiskSpaceGuard {
    fn default() -> Self {
        Self::default_guard()
    }
}

/// Get available disk space in MB (Linux/macOS implementation using statvfs).
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)] // Casts needed for cross-platform compatibility
pub fn get_available_space_mb(path: &Path) -> Option<usize> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_cstr = CString::new(path.as_os_str().as_bytes()).ok()?;

    #[repr(C)]
    struct Statvfs {
        f_bsize: libc::c_ulong,
        f_frsize: libc::c_ulong,
        f_blocks: libc::fsblkcnt_t,
        f_bfree: libc::fsblkcnt_t,
        f_bavail: libc::fsblkcnt_t,
        // Remaining fields are not needed
        _rest: [u8; 128],
    }

    let mut stat: Statvfs = unsafe { std::mem::zeroed() };

    let result = unsafe { libc::statvfs(path_cstr.as_ptr(), &mut stat as *mut _ as *mut _) };

    if result == 0 {
        let block_size = stat.f_frsize as u64;
        let available_blocks = stat.f_bavail as u64;
        let available_bytes = available_blocks * block_size;
        Some((available_bytes / (1024 * 1024)) as usize)
    } else {
        None
    }
}

/// Get total and available disk space in bytes (Linux/macOS).
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)] // Casts needed for cross-platform compatibility
pub fn get_disk_space(path: &Path) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_cstr = CString::new(path.as_os_str().as_bytes()).ok()?;

    #[repr(C)]
    struct Statvfs {
        f_bsize: libc::c_ulong,
        f_frsize: libc::c_ulong,
        f_blocks: libc::fsblkcnt_t,
        f_bfree: libc::fsblkcnt_t,
        f_bavail: libc::fsblkcnt_t,
        _rest: [u8; 128],
    }

    let mut stat: Statvfs = unsafe { std::mem::zeroed() };

    let result = unsafe { libc::statvfs(path_cstr.as_ptr(), &mut stat as *mut _ as *mut _) };

    if result == 0 {
        let block_size = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        Some((total, available))
    } else {
        None
    }
}

/// Get available disk space in MB (Windows implementation).
#[cfg(target_os = "windows")]
pub fn get_available_space_mb(path: &Path) -> Option<usize> {
    use std::os::windows::ffi::OsStrExt;

    // Convert path to wide string
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if result != 0 {
        Some((free_bytes_available / (1024 * 1024)) as usize)
    } else {
        None
    }
}

/// Get total and available disk space in bytes (Windows).
#[cfg(target_os = "windows")]
pub fn get_disk_space(path: &Path) -> Option<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;

    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if result != 0 {
        Some((total_bytes, free_bytes_available))
    } else {
        None
    }
}

/// Fallback for unsupported platforms.
#[cfg(not(any(unix, target_os = "windows")))]
pub fn get_available_space_mb(_path: &Path) -> Option<usize> {
    None
}

#[cfg(not(any(unix, target_os = "windows")))]
pub fn get_disk_space(_path: &Path) -> Option<(u64, u64)> {
    None
}

/// Estimate output size in MB for planned generation.
pub fn estimate_output_size_mb(
    num_entries: usize,
    formats: &[OutputFormat],
    compression: bool,
) -> usize {
    // Average bytes per journal entry by format
    let base_bytes_per_entry = |format: &OutputFormat| -> usize {
        match format {
            OutputFormat::Csv => 400,     // CSV is compact
            OutputFormat::Json => 800,    // JSON has field names
            OutputFormat::Parquet => 200, // Parquet is compressed columnar
        }
    };

    let total: usize = formats
        .iter()
        .map(|f| num_entries * base_bytes_per_entry(f))
        .sum();

    let with_compression = if compression {
        total / 5 // ~5x compression ratio
    } else {
        total
    };

    // Add overhead for master data, indexes, etc.
    let with_overhead = (with_compression as f64 * 1.3) as usize;

    with_overhead.div_ceil(1024 * 1024)
}

/// Output format for size estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Csv,
    Json,
    Parquet,
}

/// Check if there's enough disk space for planned output.
pub fn check_sufficient_disk_space(
    path: &Path,
    planned_entries: usize,
    formats: &[OutputFormat],
    compression: bool,
    min_free_mb: usize,
) -> Result<(), String> {
    let estimated = estimate_output_size_mb(planned_entries, formats, compression);
    let available = get_available_space_mb(path)
        .ok_or_else(|| "Unable to determine available disk space on this platform".to_string())?;

    let required = estimated + min_free_mb;

    if available < required {
        Err(format!(
            "Insufficient disk space: {} MB available, need {} MB \
             (estimated output: {} MB, minimum free: {} MB). \
             Reduce output volume or free up disk space.",
            available, required, estimated, min_free_mb
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
    fn test_disk_guard_creation() {
        let guard = DiskSpaceGuard::with_min_free(100);
        assert_eq!(guard.config.hard_limit_mb, 100);
        assert_eq!(guard.config.soft_limit_mb, 500);
    }

    #[test]
    fn test_disk_guard_disabled() {
        let config = DiskSpaceGuardConfig {
            hard_limit_mb: 0,
            ..Default::default()
        };
        let guard = DiskSpaceGuard::new(config);
        // Should always succeed when disabled
        assert!(guard.check().is_ok());
        assert!(guard.check_now().is_ok());
    }

    #[test]
    fn test_output_size_estimation() {
        let formats = vec![OutputFormat::Csv, OutputFormat::Json];
        let est = estimate_output_size_mb(1000, &formats, false);
        assert!(est > 0);
        assert!(est < 10); // Should be reasonable for 1000 entries

        let est_compressed = estimate_output_size_mb(1000, &formats, true);
        assert!(est_compressed < est); // Compressed should be smaller
    }

    #[test]
    fn test_stats_tracking() {
        let guard = DiskSpaceGuard::with_min_free(1);

        for _ in 0..1000 {
            let _ = guard.check();
        }

        guard.record_write(1024 * 1024);

        let stats = guard.stats();
        assert!(stats.checks_performed > 0);
        assert_eq!(stats.estimated_bytes_written, 1024 * 1024);
    }

    #[test]
    fn test_is_available() {
        #[cfg(unix)]
        assert!(DiskSpaceGuard::is_available());
    }

    #[test]
    fn test_check_before_write() {
        let guard = DiskSpaceGuard::with_min_free(1);
        // This should succeed for a small write on most systems
        let result = guard.check_before_write(1024);
        assert!(result.is_ok());
    }
}
