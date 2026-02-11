//! Streaming traits for real-time data generation.
//!
//! This module provides traits for streaming generation with backpressure,
//! progress reporting, and cancellation support.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::SynthResult;

/// Events emitted during streaming generation.
#[derive(Debug, Clone)]
pub enum StreamEvent<T> {
    /// A data item was generated.
    Data(T),
    /// Progress update.
    Progress(StreamProgress),
    /// A batch of items was completed.
    BatchComplete {
        /// Batch identifier.
        batch_id: u64,
        /// Number of items in the batch.
        count: usize,
    },
    /// An error occurred (non-fatal, generation continues).
    Error(StreamError),
    /// Generation is complete.
    Complete(StreamSummary),
}

impl<T> StreamEvent<T> {
    /// Returns true if this is a data event.
    pub fn is_data(&self) -> bool {
        matches!(self, StreamEvent::Data(_))
    }

    /// Returns true if this is a completion event.
    pub fn is_complete(&self) -> bool {
        matches!(self, StreamEvent::Complete(_))
    }

    /// Returns true if this is an error event.
    pub fn is_error(&self) -> bool {
        matches!(self, StreamEvent::Error(_))
    }

    /// Extracts data from a Data event.
    pub fn into_data(self) -> Option<T> {
        match self {
            StreamEvent::Data(data) => Some(data),
            _ => None,
        }
    }
}

/// Progress information during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProgress {
    /// Total items generated so far.
    pub items_generated: u64,
    /// Generation rate (items per second).
    pub items_per_second: f64,
    /// Elapsed time in milliseconds.
    pub elapsed_ms: u64,
    /// Current phase/stage name.
    pub phase: String,
    /// Memory usage in MB (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_mb: Option<u64>,
    /// Buffer fill level (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_fill_ratio: Option<f64>,
    /// Estimated items remaining.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_remaining: Option<u64>,
}

impl StreamProgress {
    /// Creates a new progress tracker.
    pub fn new(phase: impl Into<String>) -> Self {
        Self {
            items_generated: 0,
            items_per_second: 0.0,
            elapsed_ms: 0,
            phase: phase.into(),
            memory_usage_mb: None,
            buffer_fill_ratio: None,
            items_remaining: None,
        }
    }

    /// Updates the progress with new values.
    pub fn update(&mut self, items_generated: u64, elapsed_ms: u64) {
        self.items_generated = items_generated;
        self.elapsed_ms = elapsed_ms;
        if elapsed_ms > 0 {
            self.items_per_second = (items_generated as f64) / (elapsed_ms as f64 / 1000.0);
        }
    }

    /// Calculates estimated time remaining in milliseconds.
    pub fn eta_ms(&self) -> Option<u64> {
        self.items_remaining.map(|remaining| {
            if self.items_per_second > 0.0 {
                ((remaining as f64 / self.items_per_second) * 1000.0) as u64
            } else {
                0
            }
        })
    }
}

/// Error during streaming (non-fatal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamError {
    /// Error message.
    pub message: String,
    /// Error category.
    pub category: StreamErrorCategory,
    /// Whether the error is recoverable.
    pub recoverable: bool,
    /// Number of items affected.
    pub items_affected: Option<usize>,
}

impl StreamError {
    /// Creates a new stream error.
    pub fn new(message: impl Into<String>, category: StreamErrorCategory) -> Self {
        Self {
            message: message.into(),
            category,
            recoverable: true,
            items_affected: None,
        }
    }

    /// Marks this error as non-recoverable.
    pub fn non_recoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }

    /// Sets the number of affected items.
    pub fn with_affected_items(mut self, count: usize) -> Self {
        self.items_affected = Some(count);
        self
    }
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.category, self.message)
    }
}

impl std::error::Error for StreamError {}

/// Categories of streaming errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamErrorCategory {
    /// Configuration error.
    Configuration,
    /// Generation error.
    Generation,
    /// Output/sink error.
    Output,
    /// Resource exhaustion.
    Resource,
    /// Validation error.
    Validation,
    /// Network error (for streaming to remote).
    Network,
    /// Internal error.
    Internal,
}

/// Summary of a completed stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSummary {
    /// Total items generated.
    pub total_items: u64,
    /// Total time taken in milliseconds.
    pub total_time_ms: u64,
    /// Average generation rate (items per second).
    pub avg_items_per_second: f64,
    /// Number of errors encountered.
    pub error_count: u64,
    /// Number of items dropped due to backpressure.
    pub dropped_count: u64,
    /// Peak memory usage in MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_memory_mb: Option<u64>,
    /// Generation phases completed.
    pub phases_completed: Vec<String>,
}

impl StreamSummary {
    /// Creates a new stream summary.
    pub fn new(total_items: u64, total_time_ms: u64) -> Self {
        let avg_items_per_second = if total_time_ms > 0 {
            (total_items as f64) / (total_time_ms as f64 / 1000.0)
        } else {
            0.0
        };

        Self {
            total_items,
            total_time_ms,
            avg_items_per_second,
            error_count: 0,
            dropped_count: 0,
            peak_memory_mb: None,
            phases_completed: Vec::new(),
        }
    }
}

/// Configuration for streaming generation.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for the output channel.
    pub buffer_size: usize,
    /// Enable progress reporting.
    pub enable_progress: bool,
    /// Interval for progress updates (in items).
    pub progress_interval: u64,
    /// Backpressure strategy.
    pub backpressure: BackpressureStrategy,
    /// Timeout for blocking operations.
    pub timeout: Option<Duration>,
    /// Batch size for generation.
    pub batch_size: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            enable_progress: true,
            progress_interval: 100,
            backpressure: BackpressureStrategy::Block,
            timeout: None,
            batch_size: 100,
        }
    }
}

/// Backpressure handling strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackpressureStrategy {
    /// Block until space is available in the buffer.
    #[default]
    Block,
    /// Drop the oldest items in the buffer.
    DropOldest,
    /// Don't add new items to a full buffer.
    DropNewest,
    /// Buffer additional items before blocking.
    Buffer {
        /// Maximum overflow buffer size.
        max_overflow: usize,
    },
}

/// Handle for controlling an active stream.
///
/// Provides methods to pause, resume, and cancel streaming.
#[derive(Debug)]
pub struct StreamControl {
    /// Whether the stream should be cancelled.
    cancelled: std::sync::atomic::AtomicBool,
    /// Whether the stream is paused.
    paused: std::sync::atomic::AtomicBool,
}

impl StreamControl {
    /// Creates a new stream control handle.
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(false),
            paused: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Requests cancellation of the stream.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Pauses the stream.
    pub fn pause(&self) {
        self.paused.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Resumes a paused stream.
    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Checks if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Checks if the stream is paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for StreamControl {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StreamControl {
    fn clone(&self) -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(self.is_cancelled()),
            paused: std::sync::atomic::AtomicBool::new(self.is_paused()),
        }
    }
}

/// Trait for generators that support streaming output.
///
/// Extends the basic Generator trait with streaming capabilities,
/// including backpressure handling and progress reporting.
#[allow(clippy::type_complexity)]
pub trait StreamingGenerator {
    /// The type of items this generator produces.
    type Item: Clone + Send + 'static;

    /// Starts streaming generation.
    ///
    /// Returns a receiver for stream events and a control handle.
    fn stream(
        &mut self,
        config: StreamConfig,
    ) -> SynthResult<(
        std::sync::mpsc::Receiver<StreamEvent<Self::Item>>,
        std::sync::Arc<StreamControl>,
    )>;

    /// Streams generation with a custom progress callback.
    fn stream_with_progress<F>(
        &mut self,
        config: StreamConfig,
        on_progress: F,
    ) -> SynthResult<(
        std::sync::mpsc::Receiver<StreamEvent<Self::Item>>,
        std::sync::Arc<StreamControl>,
    )>
    where
        F: Fn(&StreamProgress) + Send + Sync + 'static;
}

/// Trait for output sinks that support streaming input.
pub trait StreamingSink<T>: Send {
    /// Processes a stream event.
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()>;

    /// Flushes any buffered data.
    fn flush(&mut self) -> SynthResult<()>;

    /// Closes the sink and releases resources.
    fn close(self) -> SynthResult<()>;

    /// Returns the number of items processed.
    fn items_processed(&self) -> u64;
}

/// A simple collector sink that stores all items in memory.
pub struct CollectorSink<T> {
    items: Vec<T>,
    errors: Vec<StreamError>,
    summary: Option<StreamSummary>,
}

impl<T> CollectorSink<T> {
    /// Creates a new collector sink.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            errors: Vec::new(),
            summary: None,
        }
    }

    /// Creates a collector with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            errors: Vec::new(),
            summary: None,
        }
    }

    /// Returns the collected items.
    pub fn into_items(self) -> Vec<T> {
        self.items
    }

    /// Returns references to collected items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Returns collected errors.
    pub fn errors(&self) -> &[StreamError] {
        &self.errors
    }

    /// Returns the stream summary if generation completed.
    pub fn summary(&self) -> Option<&StreamSummary> {
        self.summary.as_ref()
    }
}

impl<T> Default for CollectorSink<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send> StreamingSink<T> for CollectorSink<T> {
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()> {
        match event {
            StreamEvent::Data(item) => {
                self.items.push(item);
            }
            StreamEvent::Error(error) => {
                self.errors.push(error);
            }
            StreamEvent::Complete(summary) => {
                self.summary = Some(summary);
            }
            _ => {}
        }
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        Ok(())
    }

    fn close(self) -> SynthResult<()> {
        Ok(())
    }

    fn items_processed(&self) -> u64 {
        self.items.len() as u64
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_progress() {
        let mut progress = StreamProgress::new("test_phase");
        progress.update(1000, 2000);

        assert_eq!(progress.items_generated, 1000);
        assert_eq!(progress.items_per_second, 500.0);
    }

    #[test]
    fn test_stream_error() {
        let error =
            StreamError::new("test error", StreamErrorCategory::Generation).with_affected_items(5);

        assert_eq!(error.message, "test error");
        assert_eq!(error.items_affected, Some(5));
        assert!(error.recoverable);
    }

    #[test]
    fn test_stream_summary() {
        let summary = StreamSummary::new(10000, 5000);

        assert_eq!(summary.total_items, 10000);
        assert_eq!(summary.avg_items_per_second, 2000.0);
    }

    #[test]
    fn test_stream_control() {
        let control = StreamControl::new();

        assert!(!control.is_cancelled());
        assert!(!control.is_paused());

        control.pause();
        assert!(control.is_paused());

        control.resume();
        assert!(!control.is_paused());

        control.cancel();
        assert!(control.is_cancelled());
    }

    #[test]
    fn test_collector_sink() {
        let mut sink = CollectorSink::new();

        sink.process(StreamEvent::Data(1)).unwrap();
        sink.process(StreamEvent::Data(2)).unwrap();
        sink.process(StreamEvent::Data(3)).unwrap();

        assert_eq!(sink.items(), &[1, 2, 3]);
        assert_eq!(sink.items_processed(), 3);
    }

    #[test]
    fn test_backpressure_strategy_default() {
        let strategy = BackpressureStrategy::default();
        assert_eq!(strategy, BackpressureStrategy::Block);
    }

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 1000);
        assert!(config.enable_progress);
        assert_eq!(config.progress_interval, 100);
    }
}
