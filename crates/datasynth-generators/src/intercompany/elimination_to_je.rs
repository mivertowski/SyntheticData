//! Converter from consolidation [`EliminationEntry`] records to GL [`JournalEntry`] records.
//!
//! Elimination entries are generated as domain objects by the [`EliminationGenerator`].
//! This module bridges them into the main journal-entry stream so that consolidated
//! financial statements include the corresponding GL postings.

use datasynth_core::models::intercompany::EliminationEntry;
use datasynth_core::models::{
    BusinessProcess, JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};

/// Convert a slice of [`EliminationEntry`] records into [`JournalEntry`] records.
///
/// Each elimination entry becomes exactly one balanced journal entry.  The header
/// is stamped with:
/// - `document_type = "ELIMINATION"`
/// - `created_by = "CONSOLIDATION"`
/// - `is_elimination = true`
/// - `business_process = Some(BusinessProcess::R2R)`
/// - `source = TransactionSource::Automated`
///
/// Only elimination entries that are themselves balanced (`total_debit == total_credit`)
/// are converted; unbalanced entries are skipped with a debug-level log message to avoid
/// corrupting the GL.
pub fn elimination_to_journal_entries(entries: &[EliminationEntry]) -> Vec<JournalEntry> {
    entries
        .iter()
        .filter_map(|elim| {
            if !elim.is_balanced() {
                // Skip entries that would corrupt the GL.
                // Unbalanced eliminations are a data-quality issue upstream; we log at
                // runtime via the orchestrator's own logging rather than here.
                return None;
            }

            // Build the header using the elimination's date/company/currency.
            let mut header =
                JournalEntryHeader::new(elim.consolidation_entity.clone(), elim.entry_date);
            header.document_type = "ELIMINATION".to_string();
            header.created_by = "CONSOLIDATION".to_string();
            header.user_persona = "system_consolidation".to_string();
            header.currency = elim.currency.clone();
            header.source = TransactionSource::Automated;
            header.business_process = Some(BusinessProcess::R2R);
            header.header_text = Some(elim.description.clone());
            header.reference = Some(elim.entry_id.clone());
            header.is_elimination = true;

            // Parse fiscal year / period from the YYYYMM fiscal_period string when possible.
            if elim.fiscal_period.len() == 6 {
                if let (Ok(year), Ok(month)) = (
                    elim.fiscal_period[..4].parse::<u16>(),
                    elim.fiscal_period[4..].parse::<u8>(),
                ) {
                    header.fiscal_year = year;
                    header.fiscal_period = month;
                }
            }

            let document_id = header.document_id;
            let mut je = JournalEntry::new(header);

            // Convert each elimination line to a JE line.
            for (idx, line) in elim.lines.iter().enumerate() {
                let line_number = (idx as u32) + 1;
                let je_line = if line.is_debit {
                    JournalEntryLine::debit(
                        document_id,
                        line_number,
                        line.account.clone(),
                        line.amount,
                    )
                } else {
                    JournalEntryLine::credit(
                        document_id,
                        line_number,
                        line.account.clone(),
                        line.amount,
                    )
                };
                je.add_line(je_line);
            }

            Some(je)
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::intercompany::EliminationEntry;
    use rust_decimal_macros::dec;

    fn sample_ic_balance_elim() -> EliminationEntry {
        EliminationEntry::create_ic_balance_elimination(
            "ELIM001".to_string(),
            "GROUP".to_string(),
            "202406".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            "C001",
            "C002",
            "1310", // IC receivable
            "2110", // IC payable
            dec!(50000),
            "USD".to_string(),
        )
    }

    #[test]
    fn test_converts_balanced_elimination() {
        let elim = sample_ic_balance_elim();
        let jes = elimination_to_journal_entries(&[elim]);
        assert_eq!(jes.len(), 1);
        let je = &jes[0];
        assert!(je.is_balanced(), "Resulting JE must be balanced");
        assert_eq!(je.header.document_type, "ELIMINATION");
        assert_eq!(je.header.created_by, "CONSOLIDATION");
        assert!(je.header.is_elimination);
        assert_eq!(je.line_count(), 2);
    }

    #[test]
    fn test_fiscal_period_parsed() {
        let elim = sample_ic_balance_elim();
        let jes = elimination_to_journal_entries(&[elim]);
        let je = &jes[0];
        assert_eq!(je.header.fiscal_year, 2024);
        assert_eq!(je.header.fiscal_period, 6);
    }

    #[test]
    fn test_empty_slice_returns_empty() {
        let jes = elimination_to_journal_entries(&[]);
        assert!(jes.is_empty());
    }

    #[test]
    fn test_multiple_entries_converted() {
        let e1 = sample_ic_balance_elim();
        let e2 = EliminationEntry::create_ic_revenue_expense_elimination(
            "ELIM002".to_string(),
            "GROUP".to_string(),
            "202406".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            "C001",
            "C002",
            "4100", // revenue account
            "5100", // expense account
            dec!(120000),
            "USD".to_string(),
        );
        let jes = elimination_to_journal_entries(&[e1, e2]);
        assert_eq!(jes.len(), 2);
        for je in &jes {
            assert!(je.is_balanced());
            assert!(je.header.is_elimination);
        }
    }
}
