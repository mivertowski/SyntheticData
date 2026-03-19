//! Materiality-stratified sampling validation.
//!
//! Validates that generated journal entry populations are appropriately distributed
//! across materiality strata and that anomaly coverage meets audit expectations.
//!
//! Stratification follows ISA 530 (Audit Sampling) conventions:
//! - AboveMateriality: full population coverage expected
//! - BetweenPerformanceAndOverall: judgmental sampling
//! - BelowPerformanceMateriality: statistical sampling
//! - ClearlyTrivial: excluded from scope

use datasynth_core::models::JournalEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ─── Result types ─────────────────────────────────────────────────────────────

/// Audit materiality strata per ISA 530.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Stratum {
    /// Amount > overall materiality: full coverage expected.
    AboveMateriality,
    /// Performance materiality < amount ≤ overall materiality.
    BetweenPerformanceAndOverall,
    /// Clearly trivial threshold < amount ≤ performance materiality.
    BelowPerformanceMateriality,
    /// Amount ≤ materiality × 5%: excluded from scope.
    ClearlyTrivial,
}

/// Aggregated results for a single materiality stratum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratumResult {
    /// Which stratum this represents.
    pub stratum: Stratum,
    /// Number of journal entries in this stratum.
    pub item_count: usize,
    /// Sum of debit amounts across all entries in this stratum.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Number of entries flagged as anomaly or fraud.
    pub anomaly_count: usize,
    /// Fraction of entries in this stratum that are anomaly-flagged (0.0–1.0).
    pub anomaly_rate: f64,
}

/// Overall result of materiality-stratified sampling validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingValidationResult {
    /// Total number of journal entries supplied.
    pub total_population: usize,
    /// Per-stratum breakdown.
    pub strata: Vec<StratumResult>,
    /// Fraction of above-materiality items that are anomaly-flagged.
    ///
    /// Pass threshold: ≥ 0.95 (auditors expect near-complete coverage of large items).
    pub above_materiality_coverage: f64,
    /// Fraction of strata (excluding ClearlyTrivial) that contain at least one anomaly.
    pub anomaly_stratum_coverage: f64,
    /// Fraction of unique entity codes (company codes) with at least one anomaly.
    pub entity_coverage: f64,
    /// Fraction of unique fiscal periods with at least one anomaly.
    pub temporal_coverage: f64,
    /// True when above_materiality_coverage ≥ 0.95 (relaxed threshold for synthetic data).
    pub passes: bool,
}

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Compute the sum of debit amounts for a journal entry (representative amount).
fn entry_amount(entry: &JournalEntry) -> Decimal {
    entry.lines.iter().map(|l| l.debit_amount).sum()
}

/// Return `true` if the entry is flagged as an anomaly or fraud.
fn is_anomalous(entry: &JournalEntry) -> bool {
    entry.header.is_anomaly || entry.header.is_fraud
}

/// Assign a stratum based on the entry's representative amount.
fn classify(amount: Decimal, materiality: Decimal, performance_materiality: Decimal) -> Stratum {
    let clearly_trivial_threshold = materiality * Decimal::new(5, 2); // 5% of materiality
    if amount > materiality {
        Stratum::AboveMateriality
    } else if amount > performance_materiality {
        Stratum::BetweenPerformanceAndOverall
    } else if amount > clearly_trivial_threshold {
        Stratum::BelowPerformanceMateriality
    } else {
        Stratum::ClearlyTrivial
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Validate materiality-stratified sampling of journal entries.
///
/// # Arguments
/// - `entries`: All journal entries in the population.
/// - `materiality`: Overall materiality threshold.
/// - `performance_materiality`: Performance (tolerable error) materiality threshold.
///   Typically 50–75% of overall materiality.
///
/// # Returns
/// A `SamplingValidationResult` describing stratum coverage and pass/fail status.
pub fn validate_sampling(
    entries: &[JournalEntry],
    materiality: Decimal,
    performance_materiality: Decimal,
) -> SamplingValidationResult {
    let total_population = entries.len();

    // ─── Per-stratum accumulators ─────────────────────────────────────────────
    let strata_order = [
        Stratum::AboveMateriality,
        Stratum::BetweenPerformanceAndOverall,
        Stratum::BelowPerformanceMateriality,
        Stratum::ClearlyTrivial,
    ];

    let mut counts = [0usize; 4];
    let mut totals = [Decimal::ZERO; 4];
    let mut anomaly_counts = [0usize; 4];

    // ─── Entity / temporal coverage tracking ──────────────────────────────────
    let mut all_entities: HashSet<String> = HashSet::new();
    let mut anomaly_entities: HashSet<String> = HashSet::new();
    // fiscal period key: (fiscal_year, fiscal_period)
    let mut all_periods: HashSet<(u16, u8)> = HashSet::new();
    let mut anomaly_periods: HashSet<(u16, u8)> = HashSet::new();

    for entry in entries {
        let amount = entry_amount(entry);
        let stratum = classify(amount, materiality, performance_materiality);
        let idx = match stratum {
            Stratum::AboveMateriality => 0,
            Stratum::BetweenPerformanceAndOverall => 1,
            Stratum::BelowPerformanceMateriality => 2,
            Stratum::ClearlyTrivial => 3,
        };

        counts[idx] += 1;
        totals[idx] += amount;

        let entity_key = entry.header.company_code.clone();
        let period_key = (entry.header.fiscal_year, entry.header.fiscal_period);

        all_entities.insert(entity_key.clone());
        all_periods.insert(period_key);

        if is_anomalous(entry) {
            anomaly_counts[idx] += 1;
            anomaly_entities.insert(entity_key);
            anomaly_periods.insert(period_key);
        }
    }

    // ─── Build stratum results ────────────────────────────────────────────────
    let strata: Vec<StratumResult> = strata_order
        .iter()
        .enumerate()
        .map(|(i, stratum)| {
            let count = counts[i];
            let anomaly_count = anomaly_counts[i];
            let anomaly_rate = if count > 0 {
                anomaly_count as f64 / count as f64
            } else {
                0.0
            };
            StratumResult {
                stratum: stratum.clone(),
                item_count: count,
                total_amount: totals[i],
                anomaly_count,
                anomaly_rate,
            }
        })
        .collect();

    // ─── above_materiality_coverage ───────────────────────────────────────────
    let above_mat_count = counts[0];
    let above_mat_anomaly = anomaly_counts[0];
    let above_materiality_coverage = if above_mat_count > 0 {
        above_mat_anomaly as f64 / above_mat_count as f64
    } else {
        // No items above materiality → vacuously pass
        1.0
    };

    // ─── anomaly_stratum_coverage ─────────────────────────────────────────────
    // Count strata that contain anomalies (exclude ClearlyTrivial index=3).
    let non_trivial_strata = 3usize; // AboveMateriality, Between, Below
    let strata_with_anomalies = anomaly_counts[0..3].iter().filter(|&&c| c > 0).count();
    let anomaly_stratum_coverage = if non_trivial_strata > 0 {
        strata_with_anomalies as f64 / non_trivial_strata as f64
    } else {
        1.0
    };

    // ─── entity_coverage ──────────────────────────────────────────────────────
    let entity_coverage = if all_entities.is_empty() {
        1.0
    } else {
        anomaly_entities.len() as f64 / all_entities.len() as f64
    };

    // ─── temporal_coverage ────────────────────────────────────────────────────
    let temporal_coverage = if all_periods.is_empty() {
        1.0
    } else {
        anomaly_periods.len() as f64 / all_periods.len() as f64
    };

    // ─── Pass/fail ────────────────────────────────────────────────────────────
    // Relaxed threshold: above-materiality items must have ≥ 95% anomaly coverage.
    let passes = above_materiality_coverage >= 0.95;

    SamplingValidationResult {
        total_population,
        strata,
        above_materiality_coverage,
        anomaly_stratum_coverage,
        entity_coverage,
        temporal_coverage,
        passes,
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_entry(amount: Decimal, anomaly: bool, company: &str, period: u8) -> JournalEntry {
        let posting_date = date(2024, period as u32, 1);
        let mut header = JournalEntryHeader::new(company.to_string(), posting_date);
        header.fiscal_period = period;
        header.is_anomaly = anomaly;
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);
        entry.add_line(JournalEntryLine::debit(doc_id, 1, "6000".to_string(), amount));
        entry.add_line(JournalEntryLine::credit(doc_id, 2, "2000".to_string(), amount));
        entry
    }

    #[test]
    fn test_stratum_classification() {
        // materiality = 100_000, performance_materiality = 60_000
        // clearly_trivial = 5_000
        let mat = dec!(100_000);
        let perf = dec!(60_000);

        assert_eq!(classify(dec!(200_000), mat, perf), Stratum::AboveMateriality);
        assert_eq!(classify(dec!(100_001), mat, perf), Stratum::AboveMateriality);
        assert_eq!(
            classify(dec!(80_000), mat, perf),
            Stratum::BetweenPerformanceAndOverall
        );
        assert_eq!(
            classify(dec!(60_001), mat, perf),
            Stratum::BetweenPerformanceAndOverall
        );
        assert_eq!(
            classify(dec!(10_000), mat, perf),
            Stratum::BelowPerformanceMateriality
        );
        assert_eq!(classify(dec!(1_000), mat, perf), Stratum::ClearlyTrivial);
        assert_eq!(classify(dec!(0), mat, perf), Stratum::ClearlyTrivial);
    }

    #[test]
    fn test_empty_entries() {
        let result = validate_sampling(&[], dec!(100_000), dec!(60_000));
        assert_eq!(result.total_population, 0);
        // Vacuously passes
        assert!(result.passes);
        assert!((result.above_materiality_coverage - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_above_materiality_coverage_full() {
        // All above-materiality items are anomalous → coverage = 1.0 → passes
        let entries: Vec<JournalEntry> = (0..5)
            .map(|_| make_entry(dec!(200_000), true, "C001", 1))
            .collect();
        let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
        assert!((result.above_materiality_coverage - 1.0).abs() < 1e-9);
        assert!(result.passes);
    }

    #[test]
    fn test_above_materiality_coverage_zero() {
        // No above-materiality items are anomalous → coverage = 0.0 → fails
        let entries: Vec<JournalEntry> = (0..5)
            .map(|_| make_entry(dec!(200_000), false, "C001", 1))
            .collect();
        let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
        assert!((result.above_materiality_coverage - 0.0).abs() < 1e-9);
        assert!(!result.passes);
    }

    #[test]
    fn test_entity_coverage() {
        // Two companies, one has anomaly, other does not
        let mut entries = vec![
            make_entry(dec!(50_000), true, "C001", 1),
            make_entry(dec!(50_000), false, "C002", 1),
        ];
        // Add above-materiality anomaly to pass the main threshold
        entries.push(make_entry(dec!(200_000), true, "C001", 1));
        let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
        // C001 has anomaly, C002 does not → 1/2 = 0.5
        assert!((result.entity_coverage - 0.5).abs() < 1e-9);
        assert!(result.passes);
    }

    #[test]
    fn test_temporal_coverage() {
        // 3 periods, anomalies only in 2
        let mut entries: Vec<JournalEntry> = Vec::new();
        // period 1: anomaly (above materiality)
        entries.push(make_entry(dec!(200_000), true, "C001", 1));
        // period 2: anomaly
        entries.push(make_entry(dec!(50_000), true, "C001", 2));
        // period 3: no anomaly
        entries.push(make_entry(dec!(50_000), false, "C001", 3));
        let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
        // 2 out of 3 periods have anomalies
        assert!((result.temporal_coverage - 2.0 / 3.0).abs() < 1e-9);
        assert!(result.passes);
    }

    #[test]
    fn test_stratum_counts() {
        let entries = vec![
            make_entry(dec!(200_000), true, "C001", 1),  // AboveMateriality
            make_entry(dec!(80_000), false, "C001", 2),  // Between
            make_entry(dec!(10_000), false, "C001", 3),  // Below
            make_entry(dec!(500), false, "C001", 4),     // ClearlyTrivial
        ];
        let result = validate_sampling(&entries, dec!(100_000), dec!(60_000));
        assert_eq!(result.total_population, 4);
        let above = result.strata.iter().find(|s| s.stratum == Stratum::AboveMateriality).unwrap();
        let between = result.strata.iter().find(|s| s.stratum == Stratum::BetweenPerformanceAndOverall).unwrap();
        let below = result.strata.iter().find(|s| s.stratum == Stratum::BelowPerformanceMateriality).unwrap();
        let trivial = result.strata.iter().find(|s| s.stratum == Stratum::ClearlyTrivial).unwrap();
        assert_eq!(above.item_count, 1);
        assert_eq!(between.item_count, 1);
        assert_eq!(below.item_count, 1);
        assert_eq!(trivial.item_count, 1);
    }
}
