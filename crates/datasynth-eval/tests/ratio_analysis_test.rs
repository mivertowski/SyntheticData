//! Integration tests for the financial ratio analysis evaluator (ISA 520).

#![allow(clippy::unwrap_used)]

use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_eval::coherence::ratio_analysis::{
    analyze, check_reasonableness, compute_ratios, FinancialRatios,
};
use rust_decimal_macros::dec;

fn date(year: i32, month: u32, day: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Build a balanced two-line JE: debit `da`, credit `ca`, amount `amt`.
fn je(
    company: &str,
    debit_account: &str,
    credit_account: &str,
    amount: rust_decimal::Decimal,
) -> JournalEntry {
    let header = JournalEntryHeader::new(company.to_string(), date(2024, 6, 30));
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(
        doc_id,
        1,
        debit_account.to_string(),
        amount,
    ));
    entry.add_line(JournalEntryLine::credit(
        doc_id,
        2,
        credit_account.to_string(),
        amount,
    ));
    entry
}

// ─── current_ratio ────────────────────────────────────────────────────────────

#[test]
fn test_current_ratio_two_to_one() {
    // Assets 20000 (debit 1000), Liabilities 10000 (credit 2000)
    let entries = vec![
        je("C001", "1000", "3000", dec!(20000)), // asset debit
        je("C001", "6000", "2000", dec!(10000)), // liability credit
    ];
    let ratios = compute_ratios(&entries, "C001");
    let cr = ratios.current_ratio.unwrap();
    // assets = 20000 - 10000 (from 6000→3000 debit chain, but let's verify logic)
    // Actually: account 1000 prefix1=1 → assets += 20000 - 0 = 20000
    //           account 3000 prefix1=3 → equity += -(0 - 20000) = 20000
    //           account 6000 prefix1=6 → opex += 10000 - 0 = 10000
    //           account 2000 prefix1=2 → liabilities += -(0 - 10000) = 10000
    // current_ratio = 20000 / 10000 = 2.0
    assert!(
        (cr - dec!(2.0)).abs() < dec!(0.01),
        "Expected 2.0, got {cr}"
    );
}

#[test]
fn test_current_ratio_none_when_no_liabilities() {
    let entries = vec![je("C001", "1000", "3000", dec!(5000))];
    let ratios = compute_ratios(&entries, "C001");
    assert!(
        ratios.current_ratio.is_none(),
        "No liabilities → current_ratio should be None"
    );
}

// ─── DSO ─────────────────────────────────────────────────────────────────────

#[test]
fn test_dso_computation() {
    // AR = 3650, Revenue = 3650 → DSO = 365 days
    let entries = vec![
        je("C001", "1100", "4000", dec!(3650)), // AR debit, Revenue credit
    ];
    let ratios = compute_ratios(&entries, "C001");
    let dso = ratios.dso.unwrap();
    // ar = 3650, revenue = 3650 → 3650/3650*365 = 365
    assert!(
        (dso - dec!(365)).abs() < dec!(0.5),
        "Expected DSO ≈ 365, got {dso}"
    );
}

#[test]
fn test_dso_none_when_no_revenue() {
    let entries = vec![je("C001", "1100", "3000", dec!(1000))]; // AR debit, no revenue
    let ratios = compute_ratios(&entries, "C001");
    // No 4xxx entries → revenue = 0 → DSO = None
    assert!(ratios.dso.is_none(), "No revenue → DSO should be None");
}

// ─── Gross margin ─────────────────────────────────────────────────────────────

#[test]
fn test_gross_margin_forty_percent() {
    // Revenue 10000 (credit 4000), COGS 6000 (debit 5000)
    let entries = vec![
        je("C001", "1000", "4000", dec!(10000)),
        je("C001", "5000", "1000", dec!(6000)),
    ];
    let ratios = compute_ratios(&entries, "C001");
    let gm = ratios.gross_margin.unwrap();
    assert!(
        (gm - dec!(0.40)).abs() < dec!(0.01),
        "Expected gross_margin ≈ 0.40, got {gm}"
    );
}

#[test]
fn test_gross_margin_none_when_no_revenue() {
    let entries = vec![je("C001", "5000", "3000", dec!(1000))]; // COGS, no revenue
    let ratios = compute_ratios(&entries, "C001");
    assert!(
        ratios.gross_margin.is_none(),
        "No revenue → gross_margin should be None"
    );
}

// ─── Leverage ─────────────────────────────────────────────────────────────────

#[test]
fn test_debt_to_equity_and_assets() {
    // Liabilities 4000 (credit 2000), Equity 2000 (credit 3000), Assets 6000 (debit 1000)
    let entries = vec![
        je("C001", "1000", "2000", dec!(4000)), // liability side
        je("C001", "1000", "3000", dec!(2000)), // equity side
    ];
    let ratios = compute_ratios(&entries, "C001");
    // liabilities = 4000, equity = 2000, assets = 6000
    let dte = ratios.debt_to_equity.unwrap();
    let dta = ratios.debt_to_assets.unwrap();
    assert!(
        (dte - dec!(2.0)).abs() < dec!(0.01),
        "D/E = 4000/2000 = 2.0, got {dte}"
    );
    assert!(
        (dta - dec!(0.666666)).abs() < dec!(0.01),
        "D/A = 4000/6000 ≈ 0.667, got {dta}"
    );
}

// ─── Reasonableness checks ────────────────────────────────────────────────────

#[test]
fn test_reasonableness_flags_low_current_ratio_retail() {
    let ratios = FinancialRatios {
        current_ratio: Some(dec!(0.5)), // below retail min 1.0
        ..Default::default()
    };
    let checks = check_reasonableness(&ratios, "retail");
    let cr = checks
        .iter()
        .find(|c| c.ratio_name == "current_ratio")
        .unwrap();
    assert!(!cr.is_reasonable, "0.5 < 1.0 min for retail → unreasonable");
}

#[test]
fn test_reasonableness_passes_normal_ratios() {
    let ratios = FinancialRatios {
        current_ratio: Some(dec!(1.8)),
        gross_margin: Some(dec!(0.30)),
        debt_to_assets: Some(dec!(0.40)),
        ..Default::default()
    };
    let checks = check_reasonableness(&ratios, "retail");
    for check in &checks {
        assert!(
            check.is_reasonable,
            "{} = {:?} should be within retail bounds [{}, {}]",
            check.ratio_name, check.value, check.industry_min, check.industry_max
        );
    }
}

#[test]
fn test_reasonableness_none_ratios_vacuously_pass() {
    let ratios = FinancialRatios::default(); // all None
    let checks = check_reasonableness(&ratios, "manufacturing");
    assert!(
        checks.iter().all(|c| c.is_reasonable),
        "All-None ratios should vacuously pass"
    );
}

#[test]
fn test_reasonableness_manufacturing_bounds() {
    let ratios = FinancialRatios {
        current_ratio: Some(dec!(1.5)),      // within 1.2–3.0
        inventory_turnover: Some(dec!(8.0)), // within 3.0–20.0
        ..Default::default()
    };
    let checks = check_reasonableness(&ratios, "manufacturing");
    for check in &checks {
        assert!(
            check.is_reasonable,
            "{} should be within manufacturing bounds",
            check.ratio_name
        );
    }
}

#[test]
fn test_reasonableness_high_dso_flagged() {
    let ratios = FinancialRatios {
        dso: Some(dec!(200)), // way above any industry max
        ..Default::default()
    };
    let checks = check_reasonableness(&ratios, "retail");
    let dso_check = checks.iter().find(|c| c.ratio_name == "dso").unwrap();
    assert!(
        !dso_check.is_reasonable,
        "DSO 200 > retail max 45 → unreasonable"
    );
}

// ─── Entity isolation ─────────────────────────────────────────────────────────

#[test]
fn test_entity_filter_isolates_companies() {
    // C001: revenue=5000 COGS=2000 → gross_margin = (5000-2000)/5000 = 0.60
    // C002: revenue=5000 COGS=4000 → gross_margin = (5000-4000)/5000 = 0.20
    let entries = vec![
        je("C001", "1000", "4000", dec!(5000)), // C001 revenue
        je("C001", "5000", "1000", dec!(2000)), // C001 COGS
        je("C002", "1000", "4000", dec!(5000)), // C002 revenue
        je("C002", "5000", "1000", dec!(4000)), // C002 COGS (higher)
    ];
    let r1 = compute_ratios(&entries, "C001");
    let r2 = compute_ratios(&entries, "C002");
    // Different COGS → different gross margins
    assert_ne!(
        r1.gross_margin, r2.gross_margin,
        "Per-entity isolation failed"
    );
}

// ─── analyze end-to-end ───────────────────────────────────────────────────────

#[test]
fn test_analyze_returns_complete_result() {
    let entries = vec![
        je("C001", "1000", "4000", dec!(10000)),
        je("C001", "5000", "1000", dec!(6000)),
        je("C001", "6100", "2000", dec!(1500)),
    ];
    let result = analyze(&entries, "C001", "2024-H1", "retail");
    assert_eq!(result.entity_code, "C001");
    assert_eq!(result.period, "2024-H1");
    assert_eq!(
        result.reasonableness_checks.len(),
        12,
        "Should check all 12 ratios"
    );
}

#[test]
fn test_analyze_passes_reasonable_data() {
    // Construct ratios that are within retail bounds
    let entries = vec![
        je("C001", "1000", "4000", dec!(20000)), // assets + revenue
        je("C001", "5000", "1000", dec!(12000)), // COGS
        je("C001", "6000", "2000", dec!(4000)),  // opex + liabilities
        je("C001", "1000", "3000", dec!(4000)),  // equity
    ];
    let result = analyze(&entries, "C001", "2024", "retail");
    // Check that the analysis completes without panic
    assert!(!result.reasonableness_checks.is_empty());
}
