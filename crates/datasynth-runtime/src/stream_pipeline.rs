//! Phase-aware streaming pipeline for real-time data emission.
//!
//! [`StreamPipeline`] implements the [`PhaseSink`] trait, allowing generated
//! data to be streamed to files or HTTP endpoints as it is produced, rather
//! than buffering everything in memory.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Trait for sinks that receive generated items phase-by-phase.
pub trait PhaseSink: Send + Sync {
    /// Emit a single generated item.
    fn emit(
        &self,
        phase: &str,
        item_type: &str,
        item: &serde_json::Value,
    ) -> Result<(), StreamError>;

    /// Signal that a generation phase has completed.
    fn phase_complete(&self, phase: &str) -> Result<(), StreamError>;

    /// Flush any buffered data to the underlying sink.
    fn flush(&self) -> Result<(), StreamError>;

    /// Return current streaming statistics.
    fn stats(&self) -> StreamStats;
}

/// Accumulated statistics for a streaming pipeline.
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total items emitted across all phases.
    pub items_emitted: u64,
    /// Total bytes written/sent.
    pub bytes_sent: u64,
    /// Number of errors encountered.
    pub errors: u64,
    /// Number of phases that have completed.
    pub phases_completed: u64,
}

/// Errors that can occur during streaming.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Connection to the remote endpoint failed or was lost.
    #[error("Connection error: {0}")]
    Connection(String),

    /// The internal buffer is full and the backpressure strategy is to reject.
    #[error("Backpressure: buffer full")]
    BackpressureFull,
}

/// Where the stream sends its data.
#[derive(Debug, Clone)]
pub enum StreamTarget {
    /// Send JSONL to an HTTP endpoint.
    Http {
        /// Target URL.
        url: String,
        /// Optional API key for authentication.
        api_key: Option<String>,
        /// Number of items to batch before sending.
        batch_size: usize,
    },
    /// Append JSONL to a local file.
    File {
        /// Path to the output file.
        path: PathBuf,
    },
    /// Discard all output (no-op sink).
    None,
}

/// Strategy for handling back-pressure when the sink cannot keep up.
#[derive(Debug, Clone, Default)]
pub enum BackpressureStrategy {
    /// Block the producer until the sink is ready.
    #[default]
    Block,
    /// Drop the oldest buffered items to make room.
    DropOldest,
    /// Buffer up to `max_items` before applying back-pressure.
    Buffer {
        /// Maximum number of items to buffer.
        max_items: usize,
    },
}

/// A streaming pipeline that writes generated data as JSONL envelopes.
pub struct StreamPipeline {
    target: StreamTarget,
    stats: Arc<Mutex<StreamStats>>,
    writer: Mutex<Option<Box<dyn std::io::Write + Send>>>,
}

impl StreamPipeline {
    /// Create a new pipeline for the given target.
    pub fn new(target: StreamTarget) -> Result<Self, StreamError> {
        let writer: Option<Box<dyn std::io::Write + Send>> = match &target {
            StreamTarget::File { path } => {
                let file = std::fs::File::create(path)?;
                Some(Box::new(std::io::BufWriter::new(file)))
            }
            StreamTarget::Http { .. } => None,
            StreamTarget::None => None,
        };
        Ok(Self {
            target,
            stats: Arc::new(Mutex::new(StreamStats::default())),
            writer: Mutex::new(writer),
        })
    }

    /// Create a no-op pipeline that discards all output.
    pub fn none() -> Self {
        Self {
            target: StreamTarget::None,
            stats: Arc::new(Mutex::new(StreamStats::default())),
            writer: Mutex::new(None),
        }
    }

    /// Returns `true` if this pipeline will actually emit data.
    pub fn is_active(&self) -> bool {
        !matches!(self.target, StreamTarget::None)
    }
}

impl PhaseSink for StreamPipeline {
    fn emit(
        &self,
        phase: &str,
        item_type: &str,
        item: &serde_json::Value,
    ) -> Result<(), StreamError> {
        if !self.is_active() {
            return Ok(());
        }

        let envelope = serde_json::json!({
            "phase": phase,
            "item_type": item_type,
            "data": item,
        });
        let json = serde_json::to_string(&envelope)
            .map_err(|e| StreamError::Serialization(e.to_string()))?;
        let bytes = json.len() as u64 + 1; // +1 for newline

        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(writer) = writer_guard.as_mut() {
                use std::io::Write;
                writeln!(writer, "{json}")?;
            }
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.items_emitted += 1;
            stats.bytes_sent += bytes;
        }
        Ok(())
    }

    fn phase_complete(&self, _phase: &str) -> Result<(), StreamError> {
        if let Ok(mut stats) = self.stats.lock() {
            stats.phases_completed += 1;
        }
        self.flush()
    }

    fn flush(&self) -> Result<(), StreamError> {
        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(writer) = writer_guard.as_mut() {
                use std::io::Write;
                writer.flush()?;
            }
        }
        Ok(())
    }

    fn stats(&self) -> StreamStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_none_pipeline_is_inactive() {
        let pipeline = StreamPipeline::none();
        assert!(!pipeline.is_active());
    }

    #[test]
    fn test_none_pipeline_emit_is_noop() {
        let pipeline = StreamPipeline::none();
        let item = serde_json::json!({"id": "noop"});
        pipeline.emit("phase", "Type", &item).unwrap();
        let stats = pipeline.stats();
        assert_eq!(stats.items_emitted, 0);
    }

    #[test]
    fn test_file_pipeline_writes_jsonl() {
        let tmp = std::env::temp_dir().join("test_stream_pipeline_writes.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();
        assert!(pipeline.is_active());
        let item = serde_json::json!({"id": "test-001", "amount": 100.0});
        pipeline
            .emit("journal_entries", "JournalEntry", &item)
            .unwrap();
        pipeline.flush().unwrap();
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("test-001"));
        assert!(content.contains("journal_entries"));
        assert!(content.contains("JournalEntry"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_stats_increment() {
        let tmp = std::env::temp_dir().join("test_stream_pipeline_stats.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();
        let item = serde_json::json!({"id": 1});
        pipeline.emit("phase1", "Item", &item).unwrap();
        pipeline.emit("phase1", "Item", &item).unwrap();
        pipeline.phase_complete("phase1").unwrap();
        let stats = pipeline.stats();
        assert_eq!(stats.items_emitted, 2);
        assert_eq!(stats.phases_completed, 1);
        assert!(stats.bytes_sent > 0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_multiple_phases() {
        let tmp = std::env::temp_dir().join("test_stream_pipeline_phases.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();
        let item = serde_json::json!({"id": 1});
        pipeline.emit("phase1", "A", &item).unwrap();
        pipeline.phase_complete("phase1").unwrap();
        pipeline.emit("phase2", "B", &item).unwrap();
        pipeline.phase_complete("phase2").unwrap();
        let stats = pipeline.stats();
        assert_eq!(stats.items_emitted, 2);
        assert_eq!(stats.phases_completed, 2);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_file_output_is_valid_jsonl() {
        let tmp = std::env::temp_dir().join("test_stream_pipeline_valid_jsonl.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();
        let item1 = serde_json::json!({"id": "a"});
        let item2 = serde_json::json!({"id": "b"});
        pipeline.emit("p", "T", &item1).unwrap();
        pipeline.emit("p", "T", &item2).unwrap();
        pipeline.flush().unwrap();
        let content = std::fs::read_to_string(&tmp).unwrap();
        for line in content.lines() {
            let parsed: serde_json::Value =
                serde_json::from_str(line).expect("each line should be valid JSON");
            assert!(parsed.get("phase").is_some());
            assert!(parsed.get("item_type").is_some());
            assert!(parsed.get("data").is_some());
        }
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_backpressure_strategy_default() {
        let strategy = BackpressureStrategy::default();
        assert!(matches!(strategy, BackpressureStrategy::Block));
    }

    /// A mock PhaseSink that records all emitted items for testing.
    pub struct MockPhaseSink {
        pub items: Mutex<Vec<(String, String, serde_json::Value)>>,
        pub completed_phases: Mutex<Vec<String>>,
        pub flushed: Mutex<bool>,
    }

    impl MockPhaseSink {
        pub fn new() -> Self {
            Self {
                items: Mutex::new(Vec::new()),
                completed_phases: Mutex::new(Vec::new()),
                flushed: Mutex::new(false),
            }
        }
    }

    impl PhaseSink for MockPhaseSink {
        fn emit(
            &self,
            phase: &str,
            item_type: &str,
            item: &serde_json::Value,
        ) -> Result<(), StreamError> {
            self.items.lock().unwrap().push((
                phase.to_string(),
                item_type.to_string(),
                item.clone(),
            ));
            Ok(())
        }

        fn phase_complete(&self, phase: &str) -> Result<(), StreamError> {
            self.completed_phases
                .lock()
                .unwrap()
                .push(phase.to_string());
            Ok(())
        }

        fn flush(&self) -> Result<(), StreamError> {
            *self.flushed.lock().unwrap() = true;
            Ok(())
        }

        fn stats(&self) -> StreamStats {
            let items = self.items.lock().unwrap();
            let phases = self.completed_phases.lock().unwrap();
            StreamStats {
                items_emitted: items.len() as u64,
                phases_completed: phases.len() as u64,
                bytes_sent: 0,
                errors: 0,
            }
        }
    }

    #[test]
    fn test_mock_phase_sink_records_emissions() {
        let mock = MockPhaseSink::new();
        let item1 = serde_json::json!({"id": "V001", "name": "Acme Corp"});
        let item2 = serde_json::json!({"id": "V002", "name": "Global Parts"});
        mock.emit("master_data", "Vendor", &item1).unwrap();
        mock.emit("master_data", "Vendor", &item2).unwrap();
        mock.phase_complete("master_data").unwrap();

        let items = mock.items.lock().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, "master_data");
        assert_eq!(items[0].1, "Vendor");
        assert_eq!(items[1].2["name"], "Global Parts");

        let phases = mock.completed_phases.lock().unwrap();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0], "master_data");
    }

    #[test]
    fn test_mock_phase_sink_multi_phase_emission() {
        let mock = MockPhaseSink::new();
        let je = serde_json::json!({"entry_id": "JE-001"});
        let anomaly = serde_json::json!({"label": "DuplicateEntry"});

        mock.emit("journal_entries", "JournalEntry", &je).unwrap();
        mock.phase_complete("journal_entries").unwrap();
        mock.emit("anomaly_injection", "LabeledAnomaly", &anomaly)
            .unwrap();
        mock.phase_complete("anomaly_injection").unwrap();
        mock.flush().unwrap();

        let stats = mock.stats();
        assert_eq!(stats.items_emitted, 2);
        assert_eq!(stats.phases_completed, 2);
        assert!(*mock.flushed.lock().unwrap());

        let items = mock.items.lock().unwrap();
        // Verify items from different phases are properly tagged
        assert_eq!(items[0].0, "journal_entries");
        assert_eq!(items[1].0, "anomaly_injection");
    }
}
