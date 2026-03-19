//! Integration tests for `opening_balance_to_jes`.
//!
//! Verifies that `GeneratedOpeningBalance` is correctly converted into balanced
//! `JournalEntry` records using the `ChartOfAccounts` for account-type lookup.

use datasynth_core::models::balance::{CalculatedRatios, GeneratedOpeningBalance};
use datasynth_core::models::{
    AccountSubType, AccountType, ChartOfAccounts, CoAComplexity, GLAccount, IndustrySector,
};
use datasynth_generators::opening_balance_to_jes;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

// ── helpers ──────────────────────────────────────────────────────────────────

fn small_coa() -> ChartOfAccounts {
    let mut coa = ChartOfAccounts::new(
        "TEST".to_string(),
        "Test CoA".to_string(),
        "US".to_string(),
        IndustrySector::Manufacturing,
        CoAComplexity::Small,
    );

    coa.add_account(GLAccount::new(
        "110000".to_string(),
        "Cash".to_string(),
        AccountType::Asset,
        AccountSubType::Cash,
    ));
    coa.add_account(GLAccount::new(
        "120000".to_string(),
        "Accounts Receivable".to_string(),
        AccountType::Asset,
        AccountSubType::AccountsReceivable,
    ));
    coa.add_account(GLAccount::new(
        "170000".to_string(),
        "Fixed Assets".to_string(),
        AccountType::Asset,
        AccountSubType::FixedAssets,
    ));
    coa.add_account(GLAccount::new(
        "210000".to_string(),
        "Accounts Payable".to_string(),
        AccountType::Liability,
        AccountSubType::AccountsPayable,
    ));
    coa.add_account(GLAccount::new(
        "310000".to_string(),
        "Common Stock".to_string(),
        AccountType::Equity,
        AccountSubType::CommonStock,
    ));
    coa.add_account(GLAccount::new(
        "320000".to_string(),
        "Retained Earnings".to_string(),
        AccountType::Equity,
        AccountSubType::RetainedEarnings,
    ));

    coa
}

fn ob_from_balances(
    company_code: &str,
    balances: HashMap<String, Decimal>,
) -> GeneratedOpeningBalance {
    use chrono::NaiveDate;
    GeneratedOpeningBalance {
        company_code: company_code.to_string(),
        as_of_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        balances,
        total_assets: dec!(0),
        total_liabilities: dec!(0),
        total_equity: dec!(0),
        is_balanced: true,
        calculated_ratios: CalculatedRatios {
            current_ratio: None,
            quick_ratio: None,
            debt_to_equity: None,
            working_capital: dec!(0),
        },
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn empty_ob_produces_no_jes() {
    let coa = small_coa();
    let ob = ob_from_balances("C001", HashMap::new());
    assert!(opening_balance_to_jes(&ob, &coa).is_empty());
}

#[test]
fn all_zero_balances_produces_no_jes() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(0));
    balances.insert("210000".to_string(), dec!(0));
    let ob = ob_from_balances("C001", balances);
    assert!(opening_balance_to_jes(&ob, &coa).is_empty());
}

#[test]
fn basic_balanced_set_produces_one_je() {
    let coa = small_coa();
    // A = L + E: 600,000 = 100,000 + 500,000
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(600000)); // Cash (asset)
    balances.insert("210000".to_string(), dec!(100000)); // AP (liability)
    balances.insert("320000".to_string(), dec!(500000)); // RE (equity)
    let ob = ob_from_balances("C001", balances);

    let jes = opening_balance_to_jes(&ob, &coa);
    assert_eq!(jes.len(), 1, "expected exactly one JE per company");
}

#[test]
fn je_header_fields_are_correct() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(200000));
    balances.insert("320000".to_string(), dec!(200000));
    let ob = ob_from_balances("CORP1", balances);

    let jes = opening_balance_to_jes(&ob, &coa);
    let header = &jes[0].header;

    assert_eq!(header.document_type, "OPENING_BALANCE");
    assert_eq!(header.created_by, "SYSTEM");
    assert_eq!(header.company_code, "CORP1");
}

#[test]
fn asset_account_is_debited() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(500000)); // Cash
    balances.insert("320000".to_string(), dec!(500000)); // RE
    let ob = ob_from_balances("C001", balances);

    let je = &opening_balance_to_jes(&ob, &coa)[0];
    let cash = je
        .lines
        .iter()
        .find(|l| l.gl_account == "110000")
        .expect("cash line missing");

    assert_eq!(cash.debit_amount, dec!(500000));
    assert_eq!(cash.credit_amount, dec!(0));
}

#[test]
fn liability_account_is_credited() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(700000)); // Cash
    balances.insert("210000".to_string(), dec!(200000)); // AP
    balances.insert("320000".to_string(), dec!(500000)); // RE
    let ob = ob_from_balances("C001", balances);

    let je = &opening_balance_to_jes(&ob, &coa)[0];
    let ap = je
        .lines
        .iter()
        .find(|l| l.gl_account == "210000")
        .expect("AP line missing");

    assert_eq!(ap.credit_amount, dec!(200000));
    assert_eq!(ap.debit_amount, dec!(0));
}

#[test]
fn equity_account_is_credited() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(1000000)); // Cash
    balances.insert("310000".to_string(), dec!(300000)); // Common Stock
    balances.insert("320000".to_string(), dec!(700000)); // RE
    let ob = ob_from_balances("C001", balances);

    let je = &opening_balance_to_jes(&ob, &coa)[0];

    for code in &["310000", "320000"] {
        let line = je
            .lines
            .iter()
            .find(|l| &l.gl_account.as_str() == code)
            .unwrap_or_else(|| panic!("{code} line missing"));
        assert!(line.credit_amount > dec!(0), "{code} should be credited");
        assert_eq!(line.debit_amount, dec!(0));
    }
}

#[test]
fn unknown_account_uses_prefix_heuristic() {
    let coa = small_coa();

    // "190000" is not in the CoA → first digit '1' → debit (asset heuristic)
    // "290000" is not in the CoA → first digit '2' → credit (liability heuristic)
    let mut balances = HashMap::new();
    balances.insert("190000".to_string(), dec!(400000)); // unknown asset
    balances.insert("290000".to_string(), dec!(400000)); // unknown liability
    let ob = ob_from_balances("C001", balances);

    let je = &opening_balance_to_jes(&ob, &coa)[0];

    let asset_line = je
        .lines
        .iter()
        .find(|l| l.gl_account == "190000")
        .expect("190000 missing");
    assert_eq!(asset_line.debit_amount, dec!(400000));

    let liab_line = je
        .lines
        .iter()
        .find(|l| l.gl_account == "290000")
        .expect("290000 missing");
    assert_eq!(liab_line.credit_amount, dec!(400000));
}

#[test]
fn multiple_companies_produce_one_je_each() {
    let coa = small_coa();

    let make = |code: &str| {
        let mut b = HashMap::new();
        b.insert("110000".to_string(), dec!(100));
        b.insert("320000".to_string(), dec!(100));
        ob_from_balances(code, b)
    };

    let obs = vec![make("C001"), make("C002"), make("C003")];
    let jes: Vec<_> = obs
        .iter()
        .flat_map(|ob| opening_balance_to_jes(ob, &coa))
        .collect();

    assert_eq!(jes.len(), 3);
    let codes: Vec<_> = jes
        .iter()
        .map(|je| je.header.company_code.as_str())
        .collect();
    assert!(codes.contains(&"C001"));
    assert!(codes.contains(&"C002"));
    assert!(codes.contains(&"C003"));
}

#[test]
fn document_id_is_consistent_across_all_lines() {
    let coa = small_coa();
    let mut balances = HashMap::new();
    balances.insert("110000".to_string(), dec!(500000));
    balances.insert("120000".to_string(), dec!(200000));
    balances.insert("210000".to_string(), dec!(300000));
    balances.insert("320000".to_string(), dec!(400000));
    let ob = ob_from_balances("C001", balances);

    let je = &opening_balance_to_jes(&ob, &coa)[0];
    let doc_id = je.header.document_id;
    for line in je.lines.iter() {
        assert_eq!(
            line.document_id, doc_id,
            "line document_id mismatch on {}",
            line.gl_account
        );
    }
}
