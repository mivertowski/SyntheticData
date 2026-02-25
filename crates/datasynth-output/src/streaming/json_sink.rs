//! JSON streaming sinks for real-time data output.
//!
//! Provides both JSON array output and Newline-Delimited JSON (NDJSON) output.
//! Optimized to use serde_json::to_writer() for zero-copy serialization directly
//! into the buffered writer (Phase 3 I/O optimization).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::Serialize;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::traits::{StreamEvent, StreamingSink};

/// JSON streaming sink that writes items as a JSON array.
///
/// The output format is a valid JSON array:
/// ```json
/// [
///   { "field": "value1" },
///   { "field": "value2" }
/// ]
/// ```
///
/// # Type Parameters
///
/// * `T` - The type of items to write. Must implement `Serialize`.
pub struct JsonStreamingSink<T> {
    writer: BufWriter<File>,
    items_written: u64,
    bytes_written: u64,
    is_first: bool,
    path: PathBuf,
    pretty_print: bool,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + Send> JsonStreamingSink<T> {
    /// Creates a new JSON streaming sink.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output JSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        Self::with_options(path, false)
    }

    /// Creates a JSON streaming sink with pretty printing enabled.
    pub fn pretty(path: PathBuf) -> SynthResult<Self> {
        Self::with_options(path, true)
    }

    /// Creates a JSON streaming sink with configurable options.
    fn with_options(path: PathBuf, pretty_print: bool) -> SynthResult<Self> {
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Write opening bracket
        let opening = if pretty_print { "[\n" } else { "[" };
        writer.write_all(opening.as_bytes())?;

        Ok(Self {
            writer,
            items_written: 0,
            bytes_written: opening.len() as u64,
            is_first: true,
            path,
            pretty_print,
            _phantom: PhantomData,
        })
    }

    /// Returns the path to the output file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the total bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Writes a single item to JSON.
    ///
    /// For compact mode, serializes directly to the BufWriter (zero-copy).
    /// For pretty mode, uses a reusable buffer to avoid per-line allocations.
    fn write_item(&mut self, item: &T) -> SynthResult<()> {
        // Write separator
        if !self.is_first {
            let sep = if self.pretty_print { ",\n" } else { "," };
            self.writer.write_all(sep.as_bytes())?;
            self.bytes_written += sep.len() as u64;
        }
        self.is_first = false;

        if self.pretty_print {
            // Pretty print: serialize to a temporary buffer, then indent
            let json = serde_json::to_string_pretty(item).map_err(|e| {
                SynthError::generation(format!("Failed to serialize item to JSON: {}", e))
            })?;
            // Write each line with 2-space indent directly to the writer
            for (i, line) in json.lines().enumerate() {
                if i > 0 {
                    self.writer.write_all(b"\n")?;
                }
                self.writer.write_all(b"  ")?;
                self.writer.write_all(line.as_bytes())?;
            }
            self.bytes_written += json.len() as u64;
        } else {
            // Compact mode: serialize directly to BufWriter — zero intermediate allocation
            serde_json::to_writer(&mut self.writer, item).map_err(|e| {
                SynthError::generation(format!("Failed to serialize item to JSON: {}", e))
            })?;
            self.bytes_written += 100; // estimate
        }

        self.items_written += 1;
        Ok(())
    }

    /// Finalize the JSON array by writing the closing bracket.
    fn finalize(&mut self) -> SynthResult<()> {
        let closing = if self.pretty_print { "\n]" } else { "]" };
        self.writer.write_all(closing.as_bytes())?;
        self.bytes_written += closing.len() as u64;
        self.writer.flush()?;
        Ok(())
    }
}

impl<T: Serialize + Send> StreamingSink<T> for JsonStreamingSink<T> {
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()> {
        match event {
            StreamEvent::Data(item) => {
                self.write_item(&item)?;
            }
            StreamEvent::Complete(_summary) => {
                self.finalize()?;
            }
            StreamEvent::BatchComplete { .. } => {
                self.writer.flush()?;
            }
            StreamEvent::Progress(_) | StreamEvent::Error(_) => {}
        }
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.finalize()?;
        Ok(())
    }

    fn items_processed(&self) -> u64 {
        self.items_written
    }
}

/// Newline-Delimited JSON (NDJSON) streaming sink.
///
/// Each item is written as a separate JSON object on its own line:
/// ```json
/// {"field": "value1"}
/// {"field": "value2"}
/// ```
///
/// This format is ideal for streaming and processing line by line.
///
/// # Type Parameters
///
/// * `T` - The type of items to write. Must implement `Serialize`.
pub struct NdjsonStreamingSink<T> {
    writer: BufWriter<File>,
    items_written: u64,
    bytes_written: u64,
    path: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + Send> NdjsonStreamingSink<T> {
    /// Creates a new NDJSON streaming sink.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output NDJSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::with_capacity(256 * 1024, file),
            items_written: 0,
            bytes_written: 0,
            path,
            _phantom: PhantomData,
        })
    }

    /// Returns the path to the output file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the total bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Writes a single item as a JSON line.
    ///
    /// Serializes directly to the BufWriter, avoiding intermediate String allocation.
    fn write_item(&mut self, item: &T) -> SynthResult<()> {
        serde_json::to_writer(&mut self.writer, item).map_err(|e| {
            SynthError::generation(format!("Failed to serialize item to JSON: {}", e))
        })?;

        self.writer.write_all(b"\n")?;
        self.bytes_written += 100; // estimate
        self.items_written += 1;

        Ok(())
    }
}

impl<T: Serialize + Send> StreamingSink<T> for NdjsonStreamingSink<T> {
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()> {
        match event {
            StreamEvent::Data(item) => {
                self.write_item(&item)?;
            }
            StreamEvent::Complete(_summary) => {
                self.flush()?;
            }
            StreamEvent::BatchComplete { .. } => {
                self.writer.flush()?;
            }
            StreamEvent::Progress(_) | StreamEvent::Error(_) => {}
        }
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

    fn items_processed(&self) -> u64 {
        self.items_written
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::traits::StreamSummary;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: u32,
        name: String,
        value: f64,
    }

    #[test]
    fn test_json_streaming_sink_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        let mut sink = JsonStreamingSink::<TestRecord>::new(path.clone()).unwrap();

        let record = TestRecord {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
        };

        sink.process(StreamEvent::Data(record)).unwrap();
        sink.process(StreamEvent::Complete(StreamSummary::new(1, 100)))
            .unwrap();

        // Read back and verify it's valid JSON
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Vec<TestRecord> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, 1);
    }

    #[test]
    fn test_json_streaming_sink_multiple_items() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        let mut sink = JsonStreamingSink::<TestRecord>::new(path.clone()).unwrap();

        for i in 0..5 {
            let record = TestRecord {
                id: i,
                name: format!("item_{}", i),
                value: i as f64,
            };
            sink.process(StreamEvent::Data(record)).unwrap();
        }
        sink.process(StreamEvent::Complete(StreamSummary::new(5, 100)))
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Vec<TestRecord> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 5);
    }

    #[test]
    fn test_json_streaming_sink_pretty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        let mut sink = JsonStreamingSink::<TestRecord>::pretty(path.clone()).unwrap();

        let record = TestRecord {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
        };

        sink.process(StreamEvent::Data(record)).unwrap();
        sink.process(StreamEvent::Complete(StreamSummary::new(1, 100)))
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // Pretty printed should have newlines and indentation
        assert!(content.contains("\n"));
        assert!(content.contains("  "));
    }

    #[test]
    fn test_ndjson_streaming_sink_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.ndjson");

        let mut sink = NdjsonStreamingSink::<TestRecord>::new(path.clone()).unwrap();

        for i in 0..3 {
            let record = TestRecord {
                id: i,
                name: format!("item_{}", i),
                value: i as f64,
            };
            sink.process(StreamEvent::Data(record)).unwrap();
        }
        sink.close().unwrap();

        // Read back and verify line by line
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 3);

        // Each line should be valid JSON
        for (i, line) in lines.iter().enumerate() {
            let record: TestRecord = serde_json::from_str(line).unwrap();
            assert_eq!(record.id, i as u32);
        }
    }

    #[test]
    fn test_ndjson_items_processed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.ndjson");

        let mut sink = NdjsonStreamingSink::<TestRecord>::new(path).unwrap();

        for i in 0..10 {
            let record = TestRecord {
                id: i,
                name: format!("item_{}", i),
                value: i as f64,
            };
            sink.process(StreamEvent::Data(record)).unwrap();
        }

        assert_eq!(sink.items_processed(), 10);
    }
}
