//! Materiality benchmark calculation models.
//!
//! Materiality is set at the planning stage per ISA 320 and is used to:
//! - Design audit procedures (performance materiality drives sample sizes)
//! - Evaluate whether uncorrected misstatements are material (SAD threshold)
//! - Determine whether items are clearly trivial (no further consideration)
//!
//! References:
//! - ISA 320 — Materiality in Planning and Performing an Audit
//! - ISA 450 — Evaluation of Misstatements Identified during the Audit

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Benchmark selection
// ---------------------------------------------------------------------------

/// Benchmark used to derive overall materiality.
///
/// The appropriate benchmark depends on the entity's nature, the users of
/// the financial statements, and the stability/relevance of the benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialityBenchmark {
    /// Pre-tax income (profit-making entities, 3–7% range).
    PretaxIncome,
    /// Revenue (thin-margin entities or revenue-focused users, 0.5–1% range).
    Revenue,
    /// Total assets (asset-heavy industries, 0.5–1% range).
    TotalAssets,
    /// Equity (equity-focused users or non-profit entities, 1–2% range).
    Equity,
    /// Gross profit (manufacturing/retail with thin net margins).
    GrossProfit,
}

impl std::fmt::Display for MaterialityBenchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PretaxIncome => "Pre-tax Income",
            Self::Revenue => "Revenue",
            Self::TotalAssets => "Total Assets",
            Self::Equity => "Equity",
            Self::GrossProfit => "Gross Profit",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Normalized earnings
// ---------------------------------------------------------------------------

/// Type of normalization adjustment applied to reported earnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    /// Non-recurring item (restructuring, write-off, etc.).
    NonRecurring,
    /// Extraordinary item (rare, unusual, material by nature).
    Extraordinary,
    /// Reclassification between income statement line items.
    Reclassification,
}

/// A single normalization adjustment to reported earnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationAdjustment {
    /// Human-readable description (e.g. "Restructuring charge — one-time Q3").
    pub description: String,
    /// Amount of the adjustment (positive = increases earnings, negative = decreases).
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Category of adjustment.
    pub adjustment_type: AdjustmentType,
}

/// Normalized earnings schedule — strips non-recurring items from reported
/// earnings to arrive at a "run-rate" figure used as the materiality base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEarnings {
    /// Reported (unadjusted) earnings from the income statement.
    #[serde(with = "rust_decimal::serde::str")]
    pub reported_earnings: Decimal,
    /// Adjustments applied to arrive at normalized earnings.
    pub adjustments: Vec<NormalizationAdjustment>,
    /// Normalized earnings = reported + sum(adjustments).
    #[serde(with = "rust_decimal::serde::str")]
    pub normalized_amount: Decimal,
}

impl NormalizedEarnings {
    /// Construct and verify the normalized total from reported earnings and adjustments.
    pub fn new(reported_earnings: Decimal, adjustments: Vec<NormalizationAdjustment>) -> Self {
        let adj_total: Decimal = adjustments.iter().map(|a| a.amount).sum();
        let normalized_amount = reported_earnings + adj_total;
        Self {
            reported_earnings,
            adjustments,
            normalized_amount,
        }
    }
}

// ---------------------------------------------------------------------------
// Main struct
// ---------------------------------------------------------------------------

/// Materiality calculation for a single entity and reporting period.
///
/// Generated once per entity per period.  All monetary amounts are in the
/// entity's functional currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialityCalculation {
    /// Entity / company code.
    pub entity_code: String,
    /// Human-readable period descriptor (e.g. "FY2024").
    pub period: String,
    /// Benchmark selected for this entity.
    pub benchmark: MaterialityBenchmark,
    /// Raw benchmark amount drawn from financial data.
    #[serde(with = "rust_decimal::serde::str")]
    pub benchmark_amount: Decimal,
    /// Percentage applied to the benchmark (e.g. 0.05 for 5%).
    #[serde(with = "rust_decimal::serde::str")]
    pub benchmark_percentage: Decimal,
    /// Overall materiality = benchmark_amount × benchmark_percentage.
    #[serde(with = "rust_decimal::serde::str")]
    pub overall_materiality: Decimal,
    /// Performance materiality (typically 50–75% of overall; default 65%).
    /// Used to reduce the risk that aggregate uncorrected misstatements exceed
    /// overall materiality (ISA 320.11).
    #[serde(with = "rust_decimal::serde::str")]
    pub performance_materiality: Decimal,
    /// Clearly trivial threshold (typically 5% of overall).
    /// Misstatements below this amount need not be accumulated (ISA 450.A2).
    #[serde(with = "rust_decimal::serde::str")]
    pub clearly_trivial: Decimal,
    /// Tolerable error — equals performance materiality for sampling purposes.
    #[serde(with = "rust_decimal::serde::str")]
    pub tolerable_error: Decimal,
    /// Summary of Audit Differences (SAD) nominal threshold — misstatements
    /// below this amount need not be individually tracked in the SAD schedule.
    /// Set to 5% of overall materiality per common practice (ISA 450).
    #[serde(with = "rust_decimal::serde::str")]
    pub sad_nominal: Decimal,
    /// Optional normalized earnings schedule (generated when reported earnings
    /// are unusual or volatile).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_earnings: Option<NormalizedEarnings>,
    /// Auditor's narrative rationale for the benchmark choice.
    pub rationale: String,
}

impl MaterialityCalculation {
    /// Derive the computed amounts from the supplied inputs.
    ///
    /// # Arguments
    /// * `entity_code` — Entity identifier.
    /// * `period` — Period descriptor.
    /// * `benchmark` — Chosen benchmark type.
    /// * `benchmark_amount` — Raw benchmark figure.
    /// * `benchmark_percentage` — Decimal fraction to apply (e.g. `dec!(0.05)` for 5%).
    /// * `pm_percentage` — Performance materiality as fraction of overall (e.g. `dec!(0.65)`).
    /// * `normalized_earnings` — Optional normalized earnings schedule.
    /// * `rationale` — Free-text rationale for the benchmark selection.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_code: &str,
        period: &str,
        benchmark: MaterialityBenchmark,
        benchmark_amount: Decimal,
        benchmark_percentage: Decimal,
        pm_percentage: Decimal,
        normalized_earnings: Option<NormalizedEarnings>,
        rationale: &str,
    ) -> Self {
        let overall_materiality = benchmark_amount * benchmark_percentage;
        let performance_materiality = overall_materiality * pm_percentage;
        let clearly_trivial = overall_materiality * Decimal::new(5, 2); // 5%
        let tolerable_error = performance_materiality;
        // SAD nominal = 5% of overall materiality (common professional practice).
        // Misstatements below this threshold need not be individually accumulated
        // in the Summary of Audit Differences schedule.
        let sad_nominal = overall_materiality * Decimal::new(5, 2); // 5% of OM

        Self {
            entity_code: entity_code.to_string(),
            period: period.to_string(),
            benchmark,
            benchmark_amount,
            benchmark_percentage,
            overall_materiality,
            performance_materiality,
            clearly_trivial,
            tolerable_error,
            sad_nominal,
            normalized_earnings,
            rationale: rationale.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn materiality_basic_calculation() {
        let calc = MaterialityCalculation::new(
            "C001",
            "FY2024",
            MaterialityBenchmark::PretaxIncome,
            dec!(1_000_000),
            dec!(0.05),
            dec!(0.65),
            None,
            "5% of pre-tax income — profit-making entity",
        );
        assert_eq!(calc.overall_materiality, dec!(50_000));
        assert_eq!(calc.performance_materiality, dec!(32_500));
        assert_eq!(calc.clearly_trivial, dec!(2_500));
        assert_eq!(calc.tolerable_error, dec!(32_500));
        // SAD nominal = 5% of overall materiality = 2,500
        assert_eq!(calc.sad_nominal, dec!(2_500));
    }

    #[test]
    fn pm_between_50_and_75_percent_of_overall() {
        let calc = MaterialityCalculation::new(
            "C001",
            "FY2024",
            MaterialityBenchmark::Revenue,
            dec!(10_000_000),
            dec!(0.005),
            dec!(0.65),
            None,
            "0.5% of revenue",
        );
        let overall = calc.overall_materiality;
        let pm = calc.performance_materiality;
        let ratio = pm / overall;
        assert!(ratio >= dec!(0.50), "PM should be >= 50% of overall");
        assert!(ratio <= dec!(0.75), "PM should be <= 75% of overall");
    }

    #[test]
    fn clearly_trivial_is_five_percent_of_overall() {
        let calc = MaterialityCalculation::new(
            "C001",
            "FY2024",
            MaterialityBenchmark::TotalAssets,
            dec!(5_000_000),
            dec!(0.005),
            dec!(0.65),
            None,
            "0.5% of total assets",
        );
        let expected_ct = calc.overall_materiality * dec!(0.05);
        assert_eq!(calc.clearly_trivial, expected_ct);
    }

    #[test]
    fn normalized_earnings_adjustments_sum_correctly() {
        let adjustments = vec![
            NormalizationAdjustment {
                description: "Restructuring charge".into(),
                amount: dec!(200_000),
                adjustment_type: AdjustmentType::NonRecurring,
            },
            NormalizationAdjustment {
                description: "Asset write-off".into(),
                amount: dec!(-50_000),
                adjustment_type: AdjustmentType::Extraordinary,
            },
        ];
        let ne = NormalizedEarnings::new(dec!(800_000), adjustments);
        assert_eq!(ne.normalized_amount, dec!(950_000));
    }
}
