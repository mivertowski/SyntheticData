//! Parquet streaming sink for real-time data output.
//!
//! Writes streaming data to Parquet files with configurable row group sizes.

use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::traits::{StreamEvent, StreamingSink};

/// Parquet streaming sink that buffers items and writes in row groups.
///
/// Items are buffered until a row group is full, then written to the Parquet file.
/// This approach provides efficient compression while supporting streaming input.
///
/// # Type Parameters
///
/// * `T` - The type of items to write. Must implement `ToParquetBatch`.
///
/// # Note
///
/// For custom types, implement the `ToParquetBatch` trait to define how your
/// data maps to Arrow schema and batches.
pub struct ParquetStreamingSink<T: ToParquetBatch + Send> {
    /// Lazily initialized writer (created on first flush to capture actual schema)
    writer: Option<ArrowWriter<std::fs::File>>,
    items_written: u64,
    buffer: Vec<T>,
    row_group_size: usize,
    path: PathBuf,
    /// Lazily set schema from first batch
    schema: Option<Arc<Schema>>,
    writer_created: bool,
}

impl<T: ToParquetBatch + Send> ParquetStreamingSink<T> {
    /// Creates a new Parquet streaming sink.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output Parquet file
    /// * `row_group_size` - Number of rows per row group (default: 10000)
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn new(path: PathBuf, row_group_size: usize) -> SynthResult<Self> {
        Ok(Self {
            writer: None,
            items_written: 0,
            buffer: Vec::with_capacity(row_group_size),
            row_group_size,
            path,
            schema: None,
            writer_created: false,
        })
    }

    /// Creates a Parquet streaming sink with default row group size (10000).
    pub fn with_defaults(path: PathBuf) -> SynthResult<Self> {
        Self::new(path, 10000)
    }

    /// Returns the path to the output file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Creates the writer lazily with the schema from the first batch.
    fn ensure_writer(&mut self, schema: Arc<Schema>) -> SynthResult<()> {
        if self.writer_created {
            return Ok(());
        }

        let file = std::fs::File::create(&self.path)?;

        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .set_max_row_group_size(self.row_group_size)
            .build();

        let writer = ArrowWriter::try_new(file, Arc::clone(&schema), Some(props)).map_err(|e| {
            SynthError::generation(format!("Failed to create Parquet writer: {}", e))
        })?;

        self.writer = Some(writer);
        self.schema = Some(schema);
        self.writer_created = true;
        Ok(())
    }

    /// Flushes the current buffer to the Parquet file.
    fn flush_buffer(&mut self) -> SynthResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Create batch from buffer (this also gives us the schema)
        let dummy_schema = Arc::new(T::schema());
        let batch = T::to_batch(&self.buffer, Arc::clone(&dummy_schema))?;

        // Ensure writer is created with the actual batch schema
        self.ensure_writer(batch.schema())?;

        if let Some(writer) = &mut self.writer {
            writer.write(&batch).map_err(|e| {
                SynthError::generation(format!("Failed to write Parquet batch: {}", e))
            })?;
        }

        self.buffer.clear();
        Ok(())
    }
}

impl<T: ToParquetBatch + Send> StreamingSink<T> for ParquetStreamingSink<T> {
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()> {
        match event {
            StreamEvent::Data(item) => {
                self.buffer.push(item);
                self.items_written += 1;

                // Flush when buffer reaches row group size
                if self.buffer.len() >= self.row_group_size {
                    self.flush_buffer()?;
                }
            }
            StreamEvent::Complete(_summary) => {
                // Flush remaining items and close
                self.flush_buffer()?;
                if let Some(writer) = self.writer.take() {
                    writer.close().map_err(|e| {
                        SynthError::generation(format!("Failed to close Parquet writer: {}", e))
                    })?;
                }
            }
            StreamEvent::BatchComplete { .. } => {
                // Optionally flush on batch complete
                self.flush_buffer()?;
            }
            StreamEvent::Progress(_) | StreamEvent::Error(_) => {}
        }
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.flush_buffer()?;
        if let Some(writer) = &mut self.writer {
            writer.flush().map_err(|e| {
                SynthError::generation(format!("Failed to flush Parquet writer: {}", e))
            })?;
        }
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush_buffer()?;
        if let Some(writer) = self.writer.take() {
            writer.close().map_err(|e| {
                SynthError::generation(format!("Failed to close Parquet writer: {}", e))
            })?;
        }
        Ok(())
    }

    fn items_processed(&self) -> u64 {
        self.items_written
    }
}

/// Trait for types that can be converted to Parquet batches.
///
/// Implement this trait to enable streaming output to Parquet files.
pub trait ToParquetBatch {
    /// Returns the Arrow schema for this type.
    fn schema() -> Schema;

    /// Converts a batch of items to an Arrow RecordBatch.
    fn to_batch(items: &[Self], schema: Arc<Schema>) -> SynthResult<RecordBatch>
    where
        Self: Sized;
}

/// A generic string-based Parquet record for simple use cases.
///
/// This type stores all fields as strings and can be used when schema
/// is determined at runtime.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenericParquetRecord {
    /// Field names
    pub field_names: Vec<String>,
    /// Field values (as strings)
    pub values: Vec<String>,
}

impl GenericParquetRecord {
    /// Creates a new generic record.
    #[allow(dead_code)]
    pub fn new(field_names: Vec<String>, values: Vec<String>) -> Self {
        Self {
            field_names,
            values,
        }
    }
}

impl ToParquetBatch for GenericParquetRecord {
    fn schema() -> Schema {
        // Default schema with common fields
        Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("type", DataType::Utf8, true),
            Field::new("data", DataType::Utf8, true),
        ])
    }

    fn to_batch(items: &[Self], schema: Arc<Schema>) -> SynthResult<RecordBatch> {
        if items.is_empty() {
            return RecordBatch::try_new_with_options(
                schema,
                vec![],
                &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
            )
            .map_err(|e| SynthError::generation(format!("Failed to create empty batch: {}", e)));
        }

        // Use the field names from the first item
        let field_names = &items[0].field_names;
        let num_fields = field_names.len();

        // Create arrays for each field
        let mut arrays: Vec<ArrayRef> = Vec::with_capacity(num_fields);

        for field_idx in 0..num_fields {
            let values: Vec<&str> = items
                .iter()
                .map(|item| item.values.get(field_idx).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            arrays.push(Arc::new(StringArray::from(values)));
        }

        // Create schema from field names
        let fields: Vec<Field> = field_names
            .iter()
            .map(|name| Field::new(name, DataType::Utf8, true))
            .collect();
        let dynamic_schema = Arc::new(Schema::new(fields));

        RecordBatch::try_new(dynamic_schema, arrays)
            .map_err(|e| SynthError::generation(format!("Failed to create record batch: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::traits::StreamSummary;
    use tempfile::tempdir;

    #[test]
    fn test_parquet_streaming_sink_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        let mut sink =
            ParquetStreamingSink::<GenericParquetRecord>::new(path.clone(), 100).unwrap();

        let record = GenericParquetRecord::new(
            vec!["id".to_string(), "name".to_string()],
            vec!["1".to_string(), "test".to_string()],
        );

        sink.process(StreamEvent::Data(record)).unwrap();
        sink.process(StreamEvent::Complete(StreamSummary::new(1, 100)))
            .unwrap();

        // Verify file exists and has content
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }

    #[test]
    fn test_parquet_streaming_sink_row_group_flush() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        // Small row group size to trigger flush
        let mut sink = ParquetStreamingSink::<GenericParquetRecord>::new(path.clone(), 5).unwrap();

        for i in 0..12 {
            let record = GenericParquetRecord::new(
                vec!["id".to_string(), "value".to_string()],
                vec![i.to_string(), format!("value_{}", i)],
            );
            sink.process(StreamEvent::Data(record)).unwrap();
        }

        sink.process(StreamEvent::Complete(StreamSummary::new(12, 100)))
            .unwrap();

        assert_eq!(sink.items_processed(), 12);
    }

    #[test]
    fn test_parquet_items_processed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        let mut sink = ParquetStreamingSink::<GenericParquetRecord>::new(path, 100).unwrap();

        for i in 0..25 {
            let record = GenericParquetRecord::new(vec!["id".to_string()], vec![i.to_string()]);
            sink.process(StreamEvent::Data(record)).unwrap();
        }

        assert_eq!(sink.items_processed(), 25);
    }
}
