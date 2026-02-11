//! Property-based tests for balance coherence.

use proptest::prelude::*;
use rust_decimal::Decimal;

use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn journal_entry_always_balanced(
        amount_cents in 1i64..1_000_000_000,
        company_code in "[A-Z]{4}",
    ) {
        let amount = Decimal::new(amount_cents, 2);
        let posting_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Create a balanced journal entry
        let header = JournalEntryHeader::new(company_code, posting_date);
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);

        entry.add_line(JournalEntryLine::debit(doc_id, 1, "1100".to_string(), amount));
        entry.add_line(JournalEntryLine::credit(doc_id, 2, "2000".to_string(), amount));

        // Balance is enforced: sum(debits) == sum(credits)
        let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();

        prop_assert_eq!(
            total_debits, total_credits,
            "Journal entry must be balanced: debits={} credits={}",
            total_debits, total_credits
        );
    }

    #[test]
    fn multi_line_journal_entry_balanced(
        n_lines in 2usize..10,
        base_amount_cents in 100i64..100_000,
        company_code in "[A-Z]{4}",
    ) {
        let base_amount = Decimal::new(base_amount_cents, 2);
        let total = base_amount * Decimal::new(n_lines as i64, 0);
        let posting_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let header = JournalEntryHeader::new(company_code, posting_date);
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);

        // Multiple debit lines
        for i in 0..n_lines {
            entry.add_line(JournalEntryLine::debit(
                doc_id,
                (i + 1) as u32,
                format!("5{:03}", i),
                base_amount,
            ));
        }
        // One credit line for the total
        entry.add_line(JournalEntryLine::credit(
            doc_id,
            (n_lines + 1) as u32,
            "1000".to_string(),
            total,
        ));

        let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();

        prop_assert_eq!(total_debits, total_credits);
    }
}
