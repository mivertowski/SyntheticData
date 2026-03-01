//! Integration tests for the JsonLinesSink output writer.

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_core::traits::Sink;
use datasynth_output::JsonLinesSink;
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
fn test_json_write_and_readback() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("journal_entries.jsonl");

    let entry = make_journal_entry("C001", Decimal::from(2500));
    let expected_doc_id = entry.header.document_id.to_string();

    let mut sink = JsonLinesSink::new(json_path.clone()).unwrap();
    sink.write(entry).unwrap();
    sink.flush().unwrap();

    let content = std::fs::read_to_string(&json_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 1, "Expected exactly 1 JSON line, got {}", lines.len());

    // Parse as valid JSON
    let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert!(parsed.is_object(), "Each line should be a JSON object");

    // Verify the document_id is present in the header
    let header = parsed.get("header").expect("Should have 'header' field");
    let doc_id_value = header
        .get("document_id")
        .expect("Header should have 'document_id'");
    assert_eq!(
        doc_id_value.as_str().unwrap(),
        expected_doc_id,
        "document_id should match"
    );

    // Verify company_code
    let company_code = header
        .get("company_code")
        .expect("Header should have 'company_code'");
    assert_eq!(company_code.as_str().unwrap(), "C001");
}

#[test]
fn test_json_multiple_entries() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("journal_entries.jsonl");

    let mut sink = JsonLinesSink::new(json_path.clone()).unwrap();

    for i in 0..5 {
        let entry = make_journal_entry("C002", Decimal::from(1000 + i));
        sink.write(entry).unwrap();
    }
    sink.flush().unwrap();

    let content = std::fs::read_to_string(&json_path).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();

    assert_eq!(lines.len(), 5, "Expected 5 JSON lines, got {}", lines.len());

    // Verify each line is valid JSON
    for (i, line) in lines.iter().enumerate() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(
            parsed.is_ok(),
            "Line {} should be valid JSON, got error: {:?}",
            i,
            parsed.err()
        );
    }

    assert_eq!(sink.items_written(), 5, "items_written should be 5");
}

#[test]
fn test_json_bytes_written() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("journal_entries.jsonl");

    let mut sink = JsonLinesSink::new(json_path).unwrap();

    let entry = make_journal_entry("C001", Decimal::from(9999));
    sink.write(entry).unwrap();
    sink.flush().unwrap();

    assert!(
        sink.bytes_written() > 0,
        "bytes_written should be greater than 0 after writing an entry"
    );
}

#[test]
fn test_json_decimal_as_string() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("journal_entries.jsonl");

    let known_amount = Decimal::new(123456, 2); // 1234.56
    let entry = make_journal_entry("C001", known_amount);

    let mut sink = JsonLinesSink::new(json_path.clone()).unwrap();
    sink.write(entry).unwrap();
    sink.flush().unwrap();

    let content = std::fs::read_to_string(&json_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content.lines().next().unwrap()).unwrap();

    // Navigate to lines array and check the debit_amount on the first line
    let lines = parsed.get("lines").expect("Should have 'lines' field");
    let first_line = lines.get(0).expect("Should have at least one line");
    let debit_amount = first_line
        .get("debit_amount")
        .expect("Line should have 'debit_amount'");

    // rust_decimal with serde::str serializes as a string, not a float
    assert!(
        debit_amount.is_string(),
        "debit_amount should be serialized as a string (not a float), got: {:?}",
        debit_amount
    );
    assert_eq!(
        debit_amount.as_str().unwrap(),
        "1234.56",
        "debit_amount string value should be '1234.56'"
    );

    // Also check credit_amount on the second line
    let second_line = lines.get(1).expect("Should have a second line");
    let credit_amount = second_line
        .get("credit_amount")
        .expect("Line should have 'credit_amount'");
    assert!(
        credit_amount.is_string(),
        "credit_amount should be serialized as a string (not a float), got: {:?}",
        credit_amount
    );
    assert_eq!(
        credit_amount.as_str().unwrap(),
        "1234.56",
        "credit_amount string value should be '1234.56'"
    );
}
