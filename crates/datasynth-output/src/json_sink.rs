//! JSON/JSONL output sink with optional disk space monitoring.
//!
//! Optimized to use serde_json::to_writer() to serialize directly into the
//! BufWriter, avoiding the intermediate String allocation from to_string()
//! (Phase 3 I/O optimization).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::JournalEntry;
use datasynth_core::traits::Sink;
use datasynth_core::{DiskSpaceGuard, DiskSpaceGuardConfig};

/// JSON Lines sink for journal entry output with optional disk space monitoring.
pub struct JsonLinesSink {
    writer: BufWriter<File>,
    items_written: u64,
    bytes_written: u64,
    /// Optional disk space guard for monitoring available space
    disk_guard: Option<Arc<DiskSpaceGuard>>,
    /// Interval for disk checks (every N items)
    check_interval: u64,
}

impl JsonLinesSink {
    /// Create a new JSON Lines sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::with_capacity(256 * 1024, file),
            items_written: 0,
            bytes_written: 0,
            disk_guard: None,
            check_interval: 500,
        })
    }

    /// Create a new JSON Lines sink with disk space monitoring.
    pub fn with_disk_guard(path: PathBuf, min_free_mb: usize) -> SynthResult<Self> {
        let file = File::create(&path)?;
        let disk_config = DiskSpaceGuardConfig::with_min_free_mb(min_free_mb).with_path(&path);
        let disk_guard = Arc::new(DiskSpaceGuard::new(disk_config));

        Ok(Self {
            writer: BufWriter::with_capacity(256 * 1024, file),
            items_written: 0,
            bytes_written: 0,
            disk_guard: Some(disk_guard),
            check_interval: 500,
        })
    }

    /// Set a custom disk guard.
    pub fn set_disk_guard(&mut self, guard: Arc<DiskSpaceGuard>) {
        self.disk_guard = Some(guard);
    }

    /// Set the disk check interval.
    pub fn set_check_interval(&mut self, interval: u64) {
        self.check_interval = interval;
    }

    /// Check disk space if guard is configured.
    fn check_disk_space(&self) -> SynthResult<()> {
        if let Some(guard) = &self.disk_guard {
            if self.items_written.is_multiple_of(self.check_interval) {
                guard
                    .check()
                    .map_err(|e| SynthError::disk_exhausted(e.available_mb, e.required_mb))?;
            }
        }
        Ok(())
    }

    /// Record bytes written for tracking.
    fn record_write(&self, bytes: u64) {
        if let Some(guard) = &self.disk_guard {
            guard.record_write(bytes);
        }
    }

    /// Get total bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl Sink for JsonLinesSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        // Check disk space periodically
        self.check_disk_space()?;

        // Write directly to the BufWriter — avoids the intermediate String
        // allocation that serde_json::to_string() would create.
        serde_json::to_writer(&mut self.writer, &item)
            .map_err(|e| SynthError::SerializationError(e.to_string()))?;
        self.writer.write_all(b"\n")?;

        // Estimate bytes written (exact tracking would require a counting writer)
        self.bytes_written += 200; // conservative estimate per JSON entry
        self.record_write(200);

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}
