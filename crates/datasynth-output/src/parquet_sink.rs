//! Parquet output sink for journal entry data.
//!
//! Writes journal entries to Parquet files using Apache Arrow columnar format
//! with Zstd compression. Each journal entry line item becomes one row in the
//! output, with header fields denormalized across all rows for that entry.
//!
//! Decimal amounts and UUIDs are stored as UTF-8 strings per project convention.

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{BooleanArray, Int32Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::JournalEntry;
use datasynth_core::traits::Sink;

/// Default number of rows to buffer before writing a RecordBatch.
const DEFAULT_BATCH_SIZE: usize = 10_000;

/// Parquet sink for journal entry output.
///
/// Buffers denormalized journal entry rows (one per line item) and writes
/// them as Arrow RecordBatches when the buffer reaches `batch_size`. Uses
/// Zstd compression for efficient storage.
///
/// # Schema
///
/// The 15-column schema stores one row per journal entry line item:
///
/// | Column         | Arrow Type | Description                        |
/// |----------------|------------|------------------------------------|
/// | document_id    | Utf8       | UUID as string                     |
/// | company_code   | Utf8       | Company code from header           |
/// | fiscal_year    | Int32      | Fiscal year (4-digit)              |
/// | fiscal_period  | Int32      | Fiscal period (1-16)               |
/// | posting_date   | Utf8       | ISO 8601 date string               |
/// | document_type  | Utf8       | Document type code                 |
/// | currency       | Utf8       | ISO 4217 currency code             |
/// | created_by     | Utf8       | User who created the entry         |
/// | is_anomaly     | Boolean    | Anomaly flag from header           |
/// | is_fraud       | Boolean    | Fraud flag from header             |
/// | line_number    | Int32      | Line item number within document   |
/// | gl_account     | Utf8       | GL account number                  |
/// | debit_amount   | Utf8       | Decimal as string (precision safe) |
/// | credit_amount  | Utf8       | Decimal as string (precision safe) |
/// | cost_center    | Utf8       | Cost center (nullable)             |
///
/// # Example
///
/// ```ignore
/// use std::path::PathBuf;
/// use datasynth_output::ParquetSink;
///
/// let mut sink = ParquetSink::new(PathBuf::from("output.parquet"), 5000)?;
/// sink.write(journal_entry)?;
/// sink.close()?;
/// ```
#[derive(Debug)]
pub struct ParquetSink {
    /// Arrow writer wrapping the output file.
    writer: ArrowWriter<File>,
    /// Arrow schema for the output.
    schema: Arc<Schema>,
    /// Buffered rows awaiting batch write.
    buffer: Vec<BufferedRow>,
    /// Number of rows to accumulate before writing a RecordBatch.
    batch_size: usize,
    /// Total journal entries written (not rows -- each entry may produce multiple rows).
    items_written: u64,
}

/// A single denormalized row in the buffer, representing one journal entry line item
/// with its parent header fields.
#[derive(Debug, Clone)]
struct BufferedRow {
    document_id: String,
    company_code: String,
    fiscal_year: i32,
    fiscal_period: i32,
    posting_date: String,
    document_type: String,
    currency: String,
    created_by: String,
    is_anomaly: bool,
    is_fraud: bool,
    line_number: i32,
    gl_account: String,
    debit_amount: String,
    credit_amount: String,
    cost_center: Option<String>,
}

/// Build the 15-column Arrow schema for journal entry line items.
fn journal_entry_schema() -> Schema {
    Schema::new(vec![
        Field::new("document_id", DataType::Utf8, false),
        Field::new("company_code", DataType::Utf8, false),
        Field::new("fiscal_year", DataType::Int32, false),
        Field::new("fiscal_period", DataType::Int32, false),
        Field::new("posting_date", DataType::Utf8, false),
        Field::new("document_type", DataType::Utf8, false),
        Field::new("currency", DataType::Utf8, false),
        Field::new("created_by", DataType::Utf8, false),
        Field::new("is_anomaly", DataType::Boolean, false),
        Field::new("is_fraud", DataType::Boolean, false),
        Field::new("line_number", DataType::Int32, false),
        Field::new("gl_account", DataType::Utf8, false),
        Field::new("debit_amount", DataType::Utf8, false),
        Field::new("credit_amount", DataType::Utf8, false),
        Field::new("cost_center", DataType::Utf8, true),
    ])
}

impl ParquetSink {
    /// Create a new Parquet sink writing to the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Output file path (will be created or truncated)
    /// * `batch_size` - Number of rows to buffer before writing a RecordBatch.
    ///   Use `0` to get the default (10,000).
    ///
    /// # Errors
    ///
    /// Returns an error if the output file cannot be created or the Parquet
    /// writer cannot be initialized.
    pub fn new(path: PathBuf, batch_size: usize) -> SynthResult<Self> {
        let batch_size = if batch_size == 0 {
            DEFAULT_BATCH_SIZE
        } else {
            batch_size
        };

        let schema = Arc::new(journal_entry_schema());

        let file = File::create(&path).map_err(|e| {
            SynthError::generation(format!(
                "Failed to create Parquet output file '{}': {}",
                path.display(),
                e
            ))
        })?;

        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).map_err(|e| {
                SynthError::generation(format!("Failed to create Zstd compression level: {}", e))
            })?))
            .set_max_row_group_size(batch_size)
            .build();

        let writer = ArrowWriter::try_new(file, Arc::clone(&schema), Some(props)).map_err(|e| {
            SynthError::generation(format!("Failed to create Parquet writer: {}", e))
        })?;

        Ok(Self {
            writer,
            schema,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
            items_written: 0,
        })
    }

    /// Flush buffered rows to the Parquet file as a RecordBatch.
    fn flush_buffer(&mut self) -> SynthResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let batch = self.build_record_batch()?;
        self.writer
            .write(&batch)
            .map_err(|e| SynthError::generation(format!("Failed to write Parquet batch: {}", e)))?;
        self.buffer.clear();
        Ok(())
    }

    /// Convert the current buffer into an Arrow RecordBatch.
    fn build_record_batch(&self) -> SynthResult<RecordBatch> {
        let rows = &self.buffer;

        let document_id: Vec<&str> = rows.iter().map(|r| r.document_id.as_str()).collect();
        let company_code: Vec<&str> = rows.iter().map(|r| r.company_code.as_str()).collect();
        let fiscal_year: Vec<i32> = rows.iter().map(|r| r.fiscal_year).collect();
        let fiscal_period: Vec<i32> = rows.iter().map(|r| r.fiscal_period).collect();
        let posting_date: Vec<&str> = rows.iter().map(|r| r.posting_date.as_str()).collect();
        let document_type: Vec<&str> = rows.iter().map(|r| r.document_type.as_str()).collect();
        let currency: Vec<&str> = rows.iter().map(|r| r.currency.as_str()).collect();
        let created_by: Vec<&str> = rows.iter().map(|r| r.created_by.as_str()).collect();
        let is_anomaly: Vec<bool> = rows.iter().map(|r| r.is_anomaly).collect();
        let is_fraud: Vec<bool> = rows.iter().map(|r| r.is_fraud).collect();
        let line_number: Vec<i32> = rows.iter().map(|r| r.line_number).collect();
        let gl_account: Vec<&str> = rows.iter().map(|r| r.gl_account.as_str()).collect();
        let debit_amount: Vec<&str> = rows.iter().map(|r| r.debit_amount.as_str()).collect();
        let credit_amount: Vec<&str> = rows.iter().map(|r| r.credit_amount.as_str()).collect();
        let cost_center: Vec<Option<&str>> =
            rows.iter().map(|r| r.cost_center.as_deref()).collect();

        let columns: Vec<Arc<dyn arrow::array::Array>> = vec![
            Arc::new(StringArray::from(document_id)),
            Arc::new(StringArray::from(company_code)),
            Arc::new(Int32Array::from(fiscal_year)),
            Arc::new(Int32Array::from(fiscal_period)),
            Arc::new(StringArray::from(posting_date)),
            Arc::new(StringArray::from(document_type)),
            Arc::new(StringArray::from(currency)),
            Arc::new(StringArray::from(created_by)),
            Arc::new(BooleanArray::from(is_anomaly)),
            Arc::new(BooleanArray::from(is_fraud)),
            Arc::new(Int32Array::from(line_number)),
            Arc::new(StringArray::from(gl_account)),
            Arc::new(StringArray::from(debit_amount)),
            Arc::new(StringArray::from(credit_amount)),
            Arc::new(StringArray::from(cost_center)),
        ];

        RecordBatch::try_new(Arc::clone(&self.schema), columns).map_err(|e| {
            SynthError::generation(format!("Failed to build Arrow RecordBatch: {}", e))
        })
    }
}

impl Sink for ParquetSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        let header = &item.header;
        let doc_id = header.document_id.to_string();
        let posting_date = header.posting_date.to_string();

        for line in &item.lines {
            self.buffer.push(BufferedRow {
                document_id: doc_id.clone(),
                company_code: header.company_code.clone(),
                fiscal_year: header.fiscal_year as i32,
                fiscal_period: header.fiscal_period as i32,
                posting_date: posting_date.clone(),
                document_type: header.document_type.clone(),
                currency: header.currency.clone(),
                created_by: header.created_by.clone(),
                is_anomaly: header.is_anomaly,
                is_fraud: header.is_fraud,
                line_number: line.line_number as i32,
                gl_account: line.gl_account.clone(),
                debit_amount: line.debit_amount.to_string(),
                credit_amount: line.credit_amount.to_string(),
                cost_center: line.cost_center.clone(),
            });

            if self.buffer.len() >= self.batch_size {
                self.flush_buffer()?;
            }
        }

        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.flush_buffer()?;
        self.writer.flush().map_err(|e| {
            SynthError::generation(format!("Failed to flush Parquet writer: {}", e))
        })?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush_buffer()?;
        self.writer.close().map_err(|e| {
            SynthError::generation(format!("Failed to close Parquet writer: {}", e))
        })?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;
    use chrono::NaiveDate;
    use datasynth_core::models::{JournalEntryHeader, JournalEntryLine};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use rust_decimal::Decimal;
    use tempfile::TempDir;

    /// Helper: create a balanced journal entry with the given number of line pairs.
    fn make_journal_entry(
        company: &str,
        date: NaiveDate,
        pairs: usize,
        amount: Decimal,
    ) -> JournalEntry {
        let header = JournalEntryHeader::new(company.to_string(), date);
        let mut entry = JournalEntry::new(header);

        for i in 0..pairs {
            let base = (i as u32) * 2 + 1;
            entry.add_line(JournalEntryLine::debit(
                entry.header.document_id,
                base,
                "100000".to_string(),
                amount,
            ));
            let mut credit_line = JournalEntryLine::credit(
                entry.header.document_id,
                base + 1,
                "200000".to_string(),
                amount,
            );
            credit_line.cost_center = Some("CC100".to_string());
            entry.add_line(credit_line);
        }

        entry
    }

    #[test]
    fn test_creation_and_basic_write() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("basic.parquet");

        let mut sink = ParquetSink::new(path.clone(), 100).unwrap();

        let entry = make_journal_entry(
            "1000",
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            1,
            Decimal::from(500),
        );

        sink.write(entry).unwrap();
        assert_eq!(sink.items_written(), 1);
        sink.close().unwrap();

        // File should exist and be non-empty
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }

    #[test]
    fn test_roundtrip_write_and_read() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("roundtrip.parquet");

        let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let amount = Decimal::new(12345, 2); // 123.45

        let entry = make_journal_entry("2000", date, 1, amount);
        let doc_id = entry.header.document_id.to_string();

        {
            let mut sink = ParquetSink::new(path.clone(), 100).unwrap();
            sink.write(entry).unwrap();
            sink.close().unwrap();
        }

        // Read back with the parquet reader
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let batches: Vec<RecordBatch> = reader.map(|b| b.unwrap()).collect();
        assert_eq!(batches.len(), 1);

        let batch = &batches[0];
        assert_eq!(batch.num_rows(), 2); // 1 debit + 1 credit

        // Verify schema has 15 columns
        assert_eq!(batch.num_columns(), 15);

        // Verify document_id column
        let doc_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(doc_ids.value(0), doc_id);
        assert_eq!(doc_ids.value(1), doc_id);

        // Verify company_code
        let companies = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(companies.value(0), "2000");

        // Verify fiscal_year
        let years = batch
            .column(2)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(years.value(0), 2024);

        // Verify fiscal_period
        let periods = batch
            .column(3)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(periods.value(0), 6);

        // Verify posting_date
        let dates = batch
            .column(4)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(dates.value(0), "2024-06-01");

        // Verify debit_amount and credit_amount
        let debits = batch
            .column(12)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let credits = batch
            .column(13)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        // Line 1: debit
        assert_eq!(debits.value(0), "123.45");
        assert_eq!(credits.value(0), "0");
        // Line 2: credit
        assert_eq!(debits.value(1), "0");
        assert_eq!(credits.value(1), "123.45");

        // Verify cost_center nullable column
        let cost_centers = batch
            .column(14)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert!(cost_centers.is_null(0)); // debit line has no cost center
        assert_eq!(cost_centers.value(1), "CC100"); // credit line has cost center

        // Verify is_anomaly and is_fraud
        let anomaly = batch
            .column(8)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap();
        assert!(!anomaly.value(0));
        let fraud = batch
            .column(9)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap();
        assert!(!fraud.value(0));
    }

    #[test]
    fn test_batch_flushing_behavior() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("batch_flush.parquet");

        // Batch size of 5 rows -- each entry has 2 lines, so 3 entries = 6 rows
        // which should trigger one flush mid-write.
        let mut sink = ParquetSink::new(path.clone(), 5).unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        for _ in 0..3 {
            let entry = make_journal_entry("3000", date, 1, Decimal::from(100));
            sink.write(entry).unwrap();
        }

        assert_eq!(sink.items_written(), 3);
        sink.close().unwrap();

        // Read back and verify all 6 rows survived
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let total_rows: usize = reader.map(|b| b.unwrap().num_rows()).sum();
        assert_eq!(total_rows, 6);
    }

    #[test]
    fn test_empty_file_handling() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.parquet");

        let sink = ParquetSink::new(path.clone(), 100).unwrap();
        assert_eq!(sink.items_written(), 0);
        sink.close().unwrap();

        // File should exist (Parquet footer is written on close)
        assert!(path.exists());

        // Read back -- should have zero rows but valid schema
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let total_rows: usize = reader.map(|b| b.unwrap().num_rows()).sum();
        assert_eq!(total_rows, 0);
    }

    #[test]
    fn test_multiple_batches() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("multi_batch.parquet");

        // Small batch size so we get multiple row groups
        let mut sink = ParquetSink::new(path.clone(), 4).unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        // 10 entries * 2 lines each = 20 rows => 5 full batches of 4
        for i in 0..10 {
            let amount = Decimal::from(100 * (i + 1));
            let entry = make_journal_entry("4000", date, 1, amount);
            sink.write(entry).unwrap();
        }

        assert_eq!(sink.items_written(), 10);
        sink.close().unwrap();

        // Read back and verify all 20 rows
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let batches: Vec<RecordBatch> = reader.map(|b| b.unwrap()).collect();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 20);

        // Verify amounts are correct for first and last rows
        let first_batch = &batches[0];
        let debits = first_batch
            .column(12)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(debits.value(0), "100"); // First entry, debit line
    }

    #[test]
    fn test_default_batch_size() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("default_batch.parquet");

        // Passing 0 should use the default batch size
        let sink = ParquetSink::new(path, 0).unwrap();
        assert_eq!(sink.batch_size, DEFAULT_BATCH_SIZE);
        sink.close().unwrap();
    }

    #[test]
    fn test_write_batch_trait_method() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("write_batch.parquet");

        let mut sink = ParquetSink::new(path.clone(), 100).unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 7, 4).unwrap();
        let entries: Vec<JournalEntry> = (0..5)
            .map(|i| make_journal_entry("5000", date, 1, Decimal::from(200 * (i + 1))))
            .collect();

        sink.write_batch(entries).unwrap();
        assert_eq!(sink.items_written(), 5);
        sink.close().unwrap();

        // Read back
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let total_rows: usize = reader.map(|b| b.unwrap().num_rows()).sum();
        assert_eq!(total_rows, 10); // 5 entries * 2 lines each
    }

    #[test]
    fn test_entry_with_multiple_line_pairs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("multi_lines.parquet");

        let mut sink = ParquetSink::new(path.clone(), 100).unwrap();

        // Entry with 3 debit/credit pairs = 6 lines
        let entry = make_journal_entry(
            "6000",
            NaiveDate::from_ymd_opt(2024, 2, 28).unwrap(),
            3,
            Decimal::from(999),
        );

        sink.write(entry).unwrap();
        assert_eq!(sink.items_written(), 1);
        sink.close().unwrap();

        // Read back
        let file = File::open(&path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();

        let total_rows: usize = reader.map(|b| b.unwrap().num_rows()).sum();
        assert_eq!(total_rows, 6);
    }
}
