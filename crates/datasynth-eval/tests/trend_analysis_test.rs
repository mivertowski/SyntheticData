//! Integration tests for the trend plausibility evaluator.

#![allow(clippy::unwrap_used)]

use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_eval::coherence::trend_analysis::analyze_trends;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Create a journal entry for the given fiscal period.
/// Debit `debit_acct`, credit `credit_acct`.
fn make_je(period: u8, debit_acct: &str, credit_acct: &str, amount: Decimal) -> JournalEntry {
    let m = period.clamp(1, 12);
    let posting_date = date(2024, m as u32, 1);
    let mut header = JournalEntryHeader::new("C001".to_string(), posting_date);
    header.fiscal_period = period;
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(doc_id, 1, debit_acct.to_string(), amount));
    entry.add_line(JournalEntryLine::credit(doc_id, 2, credit_acct.to_string(), amount));
    entry
}

/// Build N periods of identical revenue/expense entries.
/// debit AR (1100), credit Revenue (4000) → revenue.
/// debit Expense (6000), credit AP (2000) → expense.
fn stable_full_entries(periods: u8, revenue: Decimal, expense: Decimal) -> Vec<JournalEntry> {
    let mut entries = Vec::new();
    for p in 1..=periods {
        entries.push(make_je(p, "1100", "4000", revenue));
        entries.push(make_je(p, "6000", "2000", expense));
    }
    entries
}

// ─── Basic behavior ───────────────────────────────────────────────────────────

#[test]
fn test_empty_entries() {
    let result = analyze_trends(&[]);
    assert_eq!(result.period_count, 0);
    assert!(result.passes, "Empty data should vacuously pass");
}

#[test]
fn test_single_period() {
    let entries = vec![make_je(1, "1100", "4000", dec!(100_000))];
    let result = analyze_trends(&entries);
    assert_eq!(result.period_count, 1);
    assert!(result.passes, "Single period should pass vacuously");
}

#[test]
fn test_period_count_correct() {
    let entries = stable_full_entries(6, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    assert_eq!(result.period_count, 6);
}

#[test]
fn test_exactly_four_checks() {
    let entries = stable_full_entries(4, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    assert_eq!(result.consistency_checks.len(), 4);
}

#[test]
fn test_check_names_present() {
    let entries = stable_full_entries(4, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    let names: Vec<&str> = result
        .consistency_checks
        .iter()
        .map(|c| c.check_type.as_str())
        .collect();
    assert!(names.contains(&"RevenueStability"), "Missing RevenueStability");
    assert!(names.contains(&"ExpenseRatioStability"), "Missing ExpenseRatioStability");
    assert!(
        names.contains(&"BalanceSheetGrowthConsistency"),
        "Missing BalanceSheetGrowthConsistency"
    );
    assert!(names.contains(&"DirectionalConsistency"), "Missing DirectionalConsistency");
}

// ─── Revenue stability ────────────────────────────────────────────────────────

#[test]
fn test_stable_revenue_passes() {
    // Identical revenue each period → no swing → passes
    let entries = stable_full_entries(6, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "RevenueStability")
        .unwrap();
    assert!(check.is_consistent, "Stable revenue should pass. {}", check.details);
}

#[test]
fn test_volatile_revenue_fails() {
    // Revenue triples each period → > 50% swing → fails
    let mut entries = Vec::new();
    let mut amount = dec!(10_000);
    for p in 1u8..=4 {
        entries.push(make_je(p, "1100", "4000", amount));
        amount *= dec!(3);
    }
    let result = analyze_trends(&entries);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "RevenueStability")
        .unwrap();
    assert!(!check.is_consistent, "3× revenue growth should fail. {}", check.details);
}

#[test]
fn test_moderate_revenue_growth_passes() {
    // 20% growth each period — within 50% threshold
    let mut entries = Vec::new();
    let mut amount = dec!(100_000);
    for p in 1u8..=6 {
        entries.push(make_je(p, "1100", "4000", amount));
        // 20% growth: multiply by 1.2 (use integer arithmetic to stay in Decimal)
        amount = amount * dec!(1.2);
    }
    let result = analyze_trends(&entries);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "RevenueStability")
        .unwrap();
    assert!(check.is_consistent, "20% growth should pass. {}", check.details);
}

// ─── Expense ratio stability ──────────────────────────────────────────────────

#[test]
fn test_stable_expense_ratio_passes() {
    // Constant expense/revenue ratio → CV = 0 → passes
    let entries = stable_full_entries(6, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "ExpenseRatioStability")
        .unwrap();
    assert!(check.is_consistent, "Constant ratio should pass. {}", check.details);
}

// ─── Plausibility score and pass threshold ────────────────────────────────────

#[test]
fn test_plausibility_score_range() {
    let entries = stable_full_entries(4, dec!(50_000), dec!(30_000));
    let result = analyze_trends(&entries);
    assert!(
        result.overall_plausibility_score >= 0.0 && result.overall_plausibility_score <= 1.0,
        "Score {} out of [0,1]",
        result.overall_plausibility_score
    );
}

#[test]
fn test_stable_data_passes() {
    // Fully stable dataset should have score ≥ 0.75
    let entries = stable_full_entries(6, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    assert!(
        result.passes,
        "Stable data should pass. Score: {}, checks: {:?}",
        result.overall_plausibility_score,
        result.consistency_checks.iter().map(|c| (&c.check_type, c.is_consistent)).collect::<Vec<_>>()
    );
}

#[test]
fn test_score_is_fraction_of_passing_checks() {
    let entries = stable_full_entries(4, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    let passing = result.consistency_checks.iter().filter(|c| c.is_consistent).count();
    let expected_score = passing as f64 / result.consistency_checks.len() as f64;
    assert!(
        (result.overall_plausibility_score - expected_score).abs() < 1e-9,
        "Score should equal fraction of passing checks"
    );
}

// ─── Multi-company / period keying ───────────────────────────────────────────

#[test]
fn test_multiple_entries_same_period_aggregated() {
    // Two revenue entries in the same period should be aggregated together.
    let mut entries = Vec::new();
    for _ in 0..3 {
        entries.push(make_je(1, "1100", "4000", dec!(50_000)));
        entries.push(make_je(2, "1100", "4000", dec!(50_000)));
    }
    let result = analyze_trends(&entries);
    // Periods 1 and 2 each have 3 × 50_000 = 150_000 revenue; identical → stable
    assert_eq!(result.period_count, 2);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "RevenueStability")
        .unwrap();
    assert!(check.is_consistent, "Aggregated stable revenue should pass. {}", check.details);
}

// ─── Periods analyzed field ───────────────────────────────────────────────────

#[test]
fn test_periods_analyzed_for_two_periods() {
    // With 2 periods, revenue stability analyzes 1 consecutive pair.
    let entries = stable_full_entries(2, dec!(100_000), dec!(60_000));
    let result = analyze_trends(&entries);
    let check = result
        .consistency_checks
        .iter()
        .find(|c| c.check_type == "RevenueStability")
        .unwrap();
    assert_eq!(check.periods_analyzed, 1);
}
