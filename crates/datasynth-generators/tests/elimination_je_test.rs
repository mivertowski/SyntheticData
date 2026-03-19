//! Integration tests for the elimination-to-journal-entry converter.

use chrono::NaiveDate;
use datasynth_core::models::intercompany::EliminationEntry;
use datasynth_generators::elimination_to_journal_entries;
use rust_decimal_macros::dec;

fn make_ic_balance_elim(entry_id: &str, amount: rust_decimal::Decimal) -> EliminationEntry {
    EliminationEntry::create_ic_balance_elimination(
        entry_id.to_string(),
        "GROUP".to_string(),
        "202406".to_string(),
        NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
        "C001",
        "C002",
        "1310",
        "2110",
        amount,
        "USD".to_string(),
    )
}

fn make_ic_rev_exp_elim(entry_id: &str, amount: rust_decimal::Decimal) -> EliminationEntry {
    EliminationEntry::create_ic_revenue_expense_elimination(
        entry_id.to_string(),
        "GROUP".to_string(),
        "202406".to_string(),
        NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
        "C001",
        "C002",
        "4100",
        "5100",
        amount,
        "EUR".to_string(),
    )
}

#[test]
fn test_elimination_produces_balanced_je() {
    let elim = make_ic_balance_elim("ELIM001", dec!(50000));
    let jes = elimination_to_journal_entries(&[elim]);

    assert_eq!(jes.len(), 1, "One elimination entry → one JE");
    let je = &jes[0];
    assert!(je.is_balanced(), "JE must be balanced");
    assert_eq!(je.total_debit(), dec!(50000));
    assert_eq!(je.total_credit(), dec!(50000));
}

#[test]
fn test_elimination_je_header_flags() {
    let elim = make_ic_balance_elim("ELIM002", dec!(25000));
    let jes = elimination_to_journal_entries(&[elim]);
    let je = &jes[0];

    assert!(je.header.is_elimination, "is_elimination must be true");
    assert_eq!(je.header.document_type, "ELIMINATION");
    assert_eq!(je.header.created_by, "CONSOLIDATION");
    assert!(!je.header.is_fraud);
    assert!(!je.header.is_anomaly);
}

#[test]
fn test_fiscal_period_extracted_from_yyyymm() {
    let elim = make_ic_balance_elim("ELIM003", dec!(1000));
    let jes = elimination_to_journal_entries(&[elim]);
    let je = &jes[0];

    assert_eq!(je.header.fiscal_year, 2024);
    assert_eq!(je.header.fiscal_period, 6);
}

#[test]
fn test_currency_propagated() {
    let elim = make_ic_rev_exp_elim("ELIM004", dec!(80000));
    let jes = elimination_to_journal_entries(&[elim]);
    let je = &jes[0];

    assert_eq!(je.header.currency, "EUR");
}

#[test]
fn test_line_accounts_match_elimination() {
    let elim = make_ic_balance_elim("ELIM005", dec!(10000));
    let jes = elimination_to_journal_entries(&[elim]);
    let je = &jes[0];

    let accounts: Vec<&str> = je.lines.iter().map(|l| l.gl_account.as_str()).collect();
    // IC payable (debit line) + IC receivable (credit line)
    assert!(
        accounts.contains(&"2110"),
        "payable account must be present"
    );
    assert!(
        accounts.contains(&"1310"),
        "receivable account must be present"
    );
}

#[test]
fn test_multiple_elimination_entries_all_converted() {
    let elims = vec![
        make_ic_balance_elim("ELIM010", dec!(100000)),
        make_ic_rev_exp_elim("ELIM011", dec!(60000)),
        make_ic_balance_elim("ELIM012", dec!(30000)),
    ];
    let jes = elimination_to_journal_entries(&elims);

    assert_eq!(jes.len(), 3);
    for je in &jes {
        assert!(je.is_balanced(), "Every converted JE must be balanced");
        assert!(je.header.is_elimination);
    }
}

#[test]
fn test_empty_input_returns_empty_vec() {
    let jes = elimination_to_journal_entries(&[]);
    assert!(jes.is_empty());
}
