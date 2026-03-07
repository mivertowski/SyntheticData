//! Integration tests for Parquet output with roundtrip verification.

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_core::traits::Sink;
use datasynth_output::ParquetSink;
use parquet::file::reader::{FileReader, SerializedFileReader};
use rust_decimal::Decimal;
use std::fs::File;
use tempfile::TempDir;

fn make_journal_entry(company_code: &str, amount: i64) -> JournalEntry {
    let posting_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let header = JournalEntryHeader::new(company_code.to_string(), posting_date);
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(
        doc_id,
        1,
        "100000".to_string(),
        Decimal::from(amount),
    ));
    entry.add_line(JournalEntryLine::credit(
        doc_id,
        2,
        "200000".to_string(),
        Decimal::from(amount),
    ));
    entry
}

#[test]
fn test_parquet_roundtrip_row_count() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("journal_entries.parquet");

    let mut sink = ParquetSink::new(path.clone(), 100).unwrap();
    for i in 0..5 {
        sink.write(make_journal_entry("C001", 1000 + i)).unwrap();
    }
    sink.close().unwrap();

    let file = File::open(&path).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let metadata = reader.metadata();
    let total_rows: i64 = (0..metadata.num_row_groups())
        .map(|i| metadata.row_group(i).num_rows())
        .sum();

    // 5 entries * 2 lines each = 10 rows
    assert_eq!(total_rows, 10, "Expected 10 rows (5 entries * 2 lines)");
}

#[test]
fn test_parquet_schema_columns() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("schema_test.parquet");

    let mut sink = ParquetSink::new(path.clone(), 100).unwrap();
    sink.write(make_journal_entry("C001", 5000)).unwrap();
    sink.close().unwrap();

    let file = File::open(&path).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let schema = reader.metadata().file_metadata().schema();

    let field_names: Vec<&str> = schema
        .get_fields()
        .iter()
        .map(|f| f.name())
        .collect();

    assert!(field_names.contains(&"document_id"), "Missing document_id column");
    assert!(field_names.contains(&"company_code"), "Missing company_code column");
    assert!(field_names.contains(&"gl_account"), "Missing gl_account column");
    assert!(field_names.contains(&"debit_amount"), "Missing debit_amount column");
    assert!(field_names.contains(&"credit_amount"), "Missing credit_amount column");
    assert!(field_names.contains(&"is_anomaly"), "Missing is_anomaly column");
    assert!(field_names.contains(&"is_fraud"), "Missing is_fraud column");
}

#[test]
fn test_parquet_multiple_batches() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("batches.parquet");

    // Small batch size to force multiple row groups
    let mut sink = ParquetSink::new(path.clone(), 4).unwrap();
    for i in 0..10 {
        sink.write(make_journal_entry("C001", 100 + i)).unwrap();
    }
    sink.close().unwrap();

    let file = File::open(&path).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let metadata = reader.metadata();
    let total_rows: i64 = (0..metadata.num_row_groups())
        .map(|i| metadata.row_group(i).num_rows())
        .sum();

    // 10 entries * 2 lines = 20 rows
    assert_eq!(total_rows, 20);
    // With batch size 4, we should have multiple row groups
    assert!(
        metadata.num_row_groups() > 1,
        "Expected multiple row groups with batch_size=4 and 20 rows, got {}",
        metadata.num_row_groups()
    );
}

#[test]
fn test_parquet_items_written_count() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("count.parquet");

    let mut sink = ParquetSink::new(path, 100).unwrap();
    for _ in 0..7 {
        sink.write(make_journal_entry("C001", 500)).unwrap();
    }
    assert_eq!(sink.items_written(), 7, "items_written should track entries, not rows");
    sink.close().unwrap();
}

#[test]
fn test_parquet_empty_file_valid() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("empty.parquet");

    let sink = ParquetSink::new(path.clone(), 100).unwrap();
    sink.close().unwrap();

    // File should exist and be a valid Parquet file (with footer)
    assert!(path.exists());
    let file = File::open(&path).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let total_rows: i64 = (0..reader.metadata().num_row_groups())
        .map(|i| reader.metadata().row_group(i).num_rows())
        .sum();
    assert_eq!(total_rows, 0);
}
