//! Integration tests for the CsvSink output writer.

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_core::traits::Sink;
use datasynth_output::CsvSink;
use rust_decimal::Decimal;
use tempfile::TempDir;

/// Create a balanced JournalEntry with a single debit/credit pair.
fn make_journal_entry(company_code: &str, debit_amount: Decimal) -> JournalEntry {
    let posting_date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
    let header = JournalEntryHeader::new(company_code.to_string(), posting_date);
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(
        doc_id,
        1,
        "100000".to_string(),
        debit_amount,
    ));
    entry.add_line(JournalEntryLine::credit(
        doc_id,
        2,
        "200000".to_string(),
        debit_amount,
    ));
    entry
}

#[test]
fn test_csv_write_and_readback() {
    let tmp_dir = TempDir::new().unwrap();
    let csv_path = tmp_dir.path().join("journal_entries.csv");

    let entry = make_journal_entry("C001", Decimal::from(1500));
    let expected_doc_id = entry.header.document_id.to_string();

    let mut sink = CsvSink::new(csv_path.clone()).unwrap();
    sink.write(entry).unwrap();
    sink.flush().unwrap();

    let content = std::fs::read_to_string(&csv_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Header line + 2 data lines (one debit, one credit)
    assert!(
        lines.len() >= 3,
        "Expected at least 3 lines (header + 2 data), got {}",
        lines.len()
    );

    // Verify header contains expected column names
    let header_line = lines[0];
    assert!(
        header_line.contains("document_id"),
        "Header should contain 'document_id'"
    );
    assert!(
        header_line.contains("gl_account"),
        "Header should contain 'gl_account'"
    );
    assert!(
        header_line.contains("debit_amount"),
        "Header should contain 'debit_amount'"
    );
    assert!(
        header_line.contains("credit_amount"),
        "Header should contain 'credit_amount'"
    );

    // Verify data rows contain the document ID
    assert!(
        lines[1].contains(&expected_doc_id),
        "First data line should contain the document ID"
    );
    assert!(
        lines[2].contains(&expected_doc_id),
        "Second data line should contain the document ID"
    );

    // Verify GL accounts appear
    assert!(
        content.contains("100000"),
        "Output should contain debit GL account 100000"
    );
    assert!(
        content.contains("200000"),
        "Output should contain credit GL account 200000"
    );
}

#[test]
fn test_csv_multiple_entries() {
    let tmp_dir = TempDir::new().unwrap();
    let csv_path = tmp_dir.path().join("journal_entries.csv");

    let mut sink = CsvSink::new(csv_path.clone()).unwrap();

    for i in 0..10 {
        let entry = make_journal_entry("C001", Decimal::from(100 + i));
        sink.write(entry).unwrap();
    }
    sink.flush().unwrap();

    let content = std::fs::read_to_string(&csv_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // 1 header + 10 entries * 2 lines each = 21 lines
    assert_eq!(
        lines.len(),
        21,
        "Expected 21 lines (1 header + 10*2 data lines), got {}",
        lines.len()
    );

    assert_eq!(sink.items_written(), 10, "items_written should be 10");
}

#[test]
fn test_csv_bytes_written() {
    let tmp_dir = TempDir::new().unwrap();
    let csv_path = tmp_dir.path().join("journal_entries.csv");

    let mut sink = CsvSink::new(csv_path).unwrap();

    let entry = make_journal_entry("C001", Decimal::from(5000));
    sink.write(entry).unwrap();
    sink.flush().unwrap();

    assert!(
        sink.bytes_written() > 0,
        "bytes_written should be greater than 0 after writing an entry"
    );
}

#[test]
fn test_csv_empty_file() {
    let tmp_dir = TempDir::new().unwrap();
    let csv_path = tmp_dir.path().join("journal_entries.csv");

    let sink = CsvSink::new(csv_path.clone()).unwrap();
    // Don't write anything, just close
    sink.close().unwrap();

    // File should exist (created by File::create)
    assert!(
        csv_path.exists(),
        "CSV file should exist even without writes"
    );

    let content = std::fs::read_to_string(&csv_path).unwrap();
    // No header should be written if no items were written
    assert!(
        content.is_empty(),
        "File should be empty when no items are written, got: {:?}",
        content
    );
}
