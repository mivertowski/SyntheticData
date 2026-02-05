//! Shared test helpers for the datasynth-output crate.

use chrono::NaiveDate;
use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use rust_decimal::Decimal;

/// Create a standard test journal entry with company code "1000", posting date 2024-06-15,
/// and a balanced debit/credit pair of 5000 on accounts 100000/200000.
pub(crate) fn create_test_je() -> JournalEntry {
    let header = JournalEntryHeader::new(
        "1000".to_string(),
        NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
    );
    let mut je = JournalEntry::new(header);

    je.add_line(JournalEntryLine::debit(
        je.header.document_id,
        1,
        "100000".to_string(),
        Decimal::from(5000),
    ));
    je.add_line(JournalEntryLine::credit(
        je.header.document_id,
        2,
        "200000".to_string(),
        Decimal::from(5000),
    ));

    je
}
