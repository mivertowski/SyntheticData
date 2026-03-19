//! Integration tests for materiality-stratified sampling validation.

#![allow(clippy::unwrap_used)]

use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_eval::coherence::sampling_validation::{validate_sampling, Stratum};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn make_entry(
    amount: Decimal,
    anomaly: bool,
    company: &str,
    period: u8,
) -> JournalEntry {
    let posting_date = date(2024, period.clamp(1, 12) as u32, 1);
    let mut header = JournalEntryHeader::new(company.to_string(), posting_date);
    header.fiscal_period = period;
    header.is_anomaly = anomaly;
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(doc_id, 1, "6000".to_string(), amount));
    entry.add_line(JournalEntryLine::credit(doc_id, 2, "2000".to_string(), amount));
    entry
}

fn make_fraud_entry(amount: Decimal, company: &str, period: u8) -> JournalEntry {
    let posting_date = date(2024, period.clamp(1, 12) as u32, 1);
    let mut header = JournalEntryHeader::new(company.to_string(), posting_date);
    header.fiscal_period = period;
    header.is_fraud = true;
    let doc_id = header.document_id;
    let mut entry = JournalEntry::new(header);
    entry.add_line(JournalEntryLine::debit(doc_id, 1, "6000".to_string(), amount));
    entry.add_line(JournalEntryLine::credit(doc_id, 2, "2000".to_string(), amount));
    entry
}

// ─── Strata assignment ────────────────────────────────────────────────────────

#[test]
fn test_above_materiality_stratum() {
    // materiality = 100_000, performance_materiality = 60_000
    let entries = vec![make_entry(dec!(200_000), false, "C001", 1)];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let above = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::AboveMateriality)
        .unwrap();
    assert_eq!(above.item_count, 1);
    assert_eq!(result.total_population, 1);
}

#[test]
fn test_between_stratum() {
    let entries = vec![make_entry(dec!(80_000), false, "C001", 1)];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let between = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::BetweenPerformanceAndOverall)
        .unwrap();
    assert_eq!(between.item_count, 1);
}

#[test]
fn test_below_performance_stratum() {
    // 5% × 100_000 = 5_000; below performance (60_000) but above clearly trivial (5_000)
    let entries = vec![make_entry(dec!(10_000), false, "C001", 1)];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let below = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::BelowPerformanceMateriality)
        .unwrap();
    assert_eq!(below.item_count, 1);
}

#[test]
fn test_clearly_trivial_stratum() {
    // Amount ≤ 5_000 (5% of 100_000)
    let entries = vec![make_entry(dec!(100), false, "C001", 1)];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let trivial = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::ClearlyTrivial)
        .unwrap();
    assert_eq!(trivial.item_count, 1);
}

#[test]
fn test_all_strata_present() {
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),  // AboveMateriality
        make_entry(dec!(80_000), false, "C001", 2),  // BetweenPerformanceAndOverall
        make_entry(dec!(10_000), false, "C001", 3),  // BelowPerformanceMateriality
        make_entry(dec!(500), false, "C001", 4),     // ClearlyTrivial
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert_eq!(result.total_population, 4);
    assert_eq!(result.strata.len(), 4);
    for stratum_result in &result.strata {
        assert_eq!(stratum_result.item_count, 1);
    }
}

// ─── Coverage calculations ────────────────────────────────────────────────────

#[test]
fn test_above_materiality_coverage_all_anomalous() {
    let entries: Vec<JournalEntry> = (0..5)
        .map(|_| make_entry(dec!(200_000), true, "C001", 1))
        .collect();
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.above_materiality_coverage - 1.0).abs() < 1e-9);
    assert!(result.passes);
}

#[test]
fn test_above_materiality_coverage_none_anomalous() {
    let entries: Vec<JournalEntry> = (0..5)
        .map(|_| make_entry(dec!(200_000), false, "C001", 1))
        .collect();
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.above_materiality_coverage - 0.0).abs() < 1e-9);
    assert!(!result.passes);
}

#[test]
fn test_above_materiality_coverage_partial() {
    // 4 out of 5 above-materiality items are anomalous → 80% < 95% → fails
    let mut entries: Vec<JournalEntry> = (0..4)
        .map(|_| make_entry(dec!(200_000), true, "C001", 1))
        .collect();
    entries.push(make_entry(dec!(200_000), false, "C001", 1));
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.above_materiality_coverage - 0.8).abs() < 1e-9);
    assert!(!result.passes);
}

#[test]
fn test_no_above_materiality_items_passes_vacuously() {
    // All items below materiality → no above-materiality items → coverage = 1.0
    let entries: Vec<JournalEntry> = (0..5)
        .map(|_| make_entry(dec!(50_000), false, "C001", 1))
        .collect();
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.above_materiality_coverage - 1.0).abs() < 1e-9);
    assert!(result.passes);
}

#[test]
fn test_entity_coverage_single_company() {
    // Single company with anomalies → entity_coverage = 1.0
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),
        make_entry(dec!(50_000), false, "C001", 2),
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.entity_coverage - 1.0).abs() < 1e-9);
}

#[test]
fn test_entity_coverage_partial() {
    // Two companies, one has anomaly, one does not
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),  // C001 anomaly + above mat
        make_entry(dec!(50_000), false, "C002", 1),  // C002 clean
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    // C001: has anomaly; C002: no anomaly → 1/2 = 0.5
    assert!((result.entity_coverage - 0.5).abs() < 1e-9);
}

#[test]
fn test_temporal_coverage() {
    // 3 fiscal periods, anomalies in 2 of them
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),  // period 1 has anomaly + above mat
        make_entry(dec!(50_000), true, "C001", 2),   // period 2 has anomaly
        make_entry(dec!(50_000), false, "C001", 3),  // period 3 no anomaly
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.temporal_coverage - 2.0 / 3.0).abs() < 1e-9);
}

#[test]
fn test_anomaly_stratum_coverage() {
    // Anomalies in all 3 non-trivial strata → coverage = 1.0
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),  // AboveMateriality
        make_entry(dec!(80_000), true, "C001", 2),   // Between
        make_entry(dec!(10_000), true, "C001", 3),   // Below
        make_entry(dec!(100), false, "C001", 4),     // ClearlyTrivial (excluded)
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.anomaly_stratum_coverage - 1.0).abs() < 1e-9);
}

#[test]
fn test_fraud_flag_also_counted() {
    // is_fraud should also be treated as anomalous
    let entries = vec![make_fraud_entry(dec!(200_000), "C001", 1)];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    assert!((result.above_materiality_coverage - 1.0).abs() < 1e-9);
    assert!(result.passes);
}

// ─── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn test_empty_population() {
    let result = validate_sampling(&[], dec!(100_000), dec!(60_000));
    assert_eq!(result.total_population, 0);
    assert!(result.passes);
}

#[test]
fn test_zero_amount_entries() {
    // Zero-amount entries fall into ClearlyTrivial
    let entries: Vec<JournalEntry> = (0..3)
        .map(|_| make_entry(dec!(0), false, "C001", 1))
        .collect();
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let trivial = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::ClearlyTrivial)
        .unwrap();
    assert_eq!(trivial.item_count, 3);
    // No above-materiality items → vacuously passes
    assert!(result.passes);
}

#[test]
fn test_anomaly_rate_in_stratum() {
    let entries = vec![
        make_entry(dec!(200_000), true, "C001", 1),
        make_entry(dec!(200_000), false, "C001", 2),
    ];
    let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
    let above = result
        .strata
        .iter()
        .find(|s| s.stratum == Stratum::AboveMateriality)
        .unwrap();
    assert_eq!(above.item_count, 2);
    assert_eq!(above.anomaly_count, 1);
    assert!((above.anomaly_rate - 0.5).abs() < 1e-9);
}
