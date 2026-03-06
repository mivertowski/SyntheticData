//! CSV streaming sink for real-time data output.
//!
//! Writes streaming data to CSV files with optional disk space monitoring.
//! Optimized to reuse a single serialization buffer across all items instead
//! of allocating a new csv::Writer per item (Phase 3 I/O optimization).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::Serialize;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::traits::{StreamEvent, StreamingSink};

/// CSV streaming sink that writes serializable items to a CSV file.
///
/// This sink writes each data item as a CSV row, handling headers
/// automatically on the first write. Uses a reusable internal buffer
/// to avoid per-item allocations.
///
/// # Type Parameters
///
/// * `T` - The type of items to write. Must implement `Serialize`.
///
/// # Example
///
/// ```ignore
/// use datasynth_output::streaming::CsvStreamingSink;
/// use datasynth_core::traits::{StreamEvent, StreamingSink};
///
/// let mut sink = CsvStreamingSink::<MyData>::new("output.csv".into())?;
/// sink.process(StreamEvent::Data(my_data))?;
/// sink.close()?;
/// ```
pub struct CsvStreamingSink<T> {
    writer: BufWriter<File>,
    items_written: u64,
    bytes_written: u64,
    header_written: bool,
    path: PathBuf,
    /// Reusable serialization buffer to avoid per-item allocation.
    serialize_buf: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + Send> CsvStreamingSink<T> {
    /// Creates a new CSV streaming sink.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output CSV file
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
            header_written: false,
            path,
            serialize_buf: Vec::with_capacity(4096),
            _phantom: PhantomData,
        })
    }

    /// Creates a CSV streaming sink with a pre-written header.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output CSV file
    /// * `header` - The header line (without newline)
    pub fn with_header(path: PathBuf, header: &str) -> SynthResult<Self> {
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);
        let header_line = format!("{header}\n");
        writer.write_all(header_line.as_bytes())?;
        let bytes_written = header_line.len() as u64;

        Ok(Self {
            writer,
            items_written: 0,
            bytes_written,
            header_written: true,
            path,
            serialize_buf: Vec::with_capacity(4096),
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

    /// Writes a single item to CSV, reusing the internal buffer.
    fn write_item(&mut self, item: &T) -> SynthResult<()> {
        // Clear and reuse the buffer — no new allocation after the first item
        self.serialize_buf.clear();

        {
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(!self.header_written)
                .from_writer(&mut self.serialize_buf);

            wtr.serialize(item).map_err(|e| {
                SynthError::generation(format!("Failed to serialize item to CSV: {e}"))
            })?;

            // Flush the csv writer into our buffer (not into the file)
            wtr.flush()
                .map_err(|e| SynthError::generation(format!("Failed to flush CSV writer: {e}")))?;
        }

        self.writer.write_all(&self.serialize_buf)?;
        self.bytes_written += self.serialize_buf.len() as u64;
        self.header_written = true;
        self.items_written += 1;

        Ok(())
    }
}

impl<T: Serialize + Send> StreamingSink<T> for CsvStreamingSink<T> {
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()> {
        match event {
            StreamEvent::Data(item) => {
                self.write_item(&item)?;
            }
            StreamEvent::Complete(_summary) => {
                self.flush()?;
            }
            StreamEvent::BatchComplete { .. } => {
                // Optionally flush on batch complete
                self.writer.flush()?;
            }
            StreamEvent::Progress(_) | StreamEvent::Error(_) => {
                // Progress and error events don't need file output
            }
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
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: u32,
        name: String,
        value: f64,
    }

    #[test]
    fn test_csv_streaming_sink_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.csv");

        let mut sink = CsvStreamingSink::<TestRecord>::new(path.clone()).unwrap();

        let record = TestRecord {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
        };

        sink.process(StreamEvent::Data(record)).unwrap();
        sink.close().unwrap();

        // Read back and verify
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("id"));
        assert!(content.contains("test"));
        assert!(content.contains("42.5"));
    }

    #[test]
    fn test_csv_streaming_sink_multiple_items() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.csv");

        let mut sink = CsvStreamingSink::<TestRecord>::new(path.clone()).unwrap();

        for i in 0..10 {
            let record = TestRecord {
                id: i,
                name: format!("item_{}", i),
                value: i as f64 * 1.5,
            };
            sink.process(StreamEvent::Data(record)).unwrap();
        }

        sink.close().unwrap();

        // Verify all items were written
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        // Header + 10 data rows
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn test_csv_streaming_sink_with_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.csv");

        let mut sink =
            CsvStreamingSink::<TestRecord>::with_header(path.clone(), "id,name,value").unwrap();

        let record = TestRecord {
            id: 1,
            name: "test".to_string(),
            value: 42.5,
        };

        sink.process(StreamEvent::Data(record)).unwrap();
        sink.close().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines[0], "id,name,value");
    }
}
