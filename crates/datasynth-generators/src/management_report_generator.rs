//! Management report generator (WI-7).
//!
//! Generates realistic management packs, board reports, flash reports, and
//! forecast packs that aggregate KPI performance and budget variance data
//! into period-level narrative documents used by auditors as analytical
//! evidence (ISA 520).

use chrono::NaiveDate;
use datasynth_core::models::{BudgetVarianceLine, KpiSummaryLine, ManagementReport};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

// ---------------------------------------------------------------------------
// Template pools
// ---------------------------------------------------------------------------

/// KPI metric names used in management packs.
const KPI_METRICS: &[&str] = &[
    "Revenue Growth Rate",
    "Gross Margin",
    "Operating Margin",
    "EBITDA Margin",
    "Current Ratio",
    "Days Sales Outstanding",
    "Inventory Turnover",
    "Order Fulfillment Rate",
    "Customer Satisfaction Score",
    "Employee Turnover Rate",
    "Net Promoter Score",
    "Return on Assets",
];

/// GL account categories used for budget variance lines.
const BUDGET_ACCOUNTS: &[(&str, f64, f64)] = &[
    // (label, budget_base_min, budget_base_max) all amounts in thousands
    ("Revenue", 500.0, 5_000.0),
    ("Cost of Goods Sold", 200.0, 3_000.0),
    ("Gross Profit", 150.0, 2_000.0),
    ("Salaries & Benefits", 100.0, 1_500.0),
    ("Rent & Facilities", 20.0, 200.0),
    ("Marketing & Advertising", 15.0, 300.0),
    ("Research & Development", 10.0, 500.0),
    ("Depreciation & Amortisation", 5.0, 100.0),
    ("Interest Expense", 2.0, 50.0),
    ("General & Administrative", 10.0, 150.0),
    ("Travel & Entertainment", 5.0, 80.0),
    ("IT & Software", 8.0, 120.0),
    ("Professional Fees", 5.0, 60.0),
    ("Taxes", 10.0, 200.0),
    ("Capital Expenditure", 20.0, 400.0),
];

/// Commentary templates keyed on overall budget position.
const POSITIVE_COMMENTARY: &[&str] = &[
    "Revenue exceeded target for the period, driven by strong demand in the core product segment.",
    "Gross margin improvement reflects continued procurement savings and favourable product mix.",
    "Operating expenses were well-controlled; all major cost lines came in on or below budget.",
    "Strong cash collections in the period resulted in DSO improvement versus prior year.",
    "Operating profit was ahead of plan, supported by one-off cost savings in facilities.",
];

const NEGATIVE_COMMENTARY: &[&str] = &[
    "Revenue fell short of target due to delayed customer onboarding and a weaker macro environment.",
    "Cost overruns in the Engineering department require remediation action in the next period.",
    "Supply chain disruptions led to higher-than-budgeted COGS; management is reviewing sourcing strategy.",
    "Margin compression was observed as a result of increased input costs not yet passed on to customers.",
    "Operating expenses exceeded budget primarily in Marketing; a revised spend plan is being developed.",
];

const NEUTRAL_COMMENTARY: &[&str] = &[
    "Performance was broadly in line with the annual operating plan.",
    "No material variances were identified; the business is on track to deliver the full-year budget.",
    "Minor timing differences between actual and budget are expected to reverse in subsequent periods.",
    "The period results reflect normal seasonal patterns consistent with the prior year.",
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates [`ManagementReport`] instances representing monthly packs,
/// quarterly board reports, and flash reports for a given entity.
pub struct ManagementReportGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl ManagementReportGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ManagementReport),
        }
    }

    /// Generate management reports for the given entity and fiscal year.
    ///
    /// Produces:
    /// - 12 monthly flash reports (prepared on day 5 of the following month)
    /// - 12 monthly packs (prepared on day 15 of the following month)
    /// - 4 quarterly board reports (prepared ~20 days after quarter end)
    ///
    /// # Arguments
    ///
    /// * `entity_code` – The entity code these reports belong to.
    /// * `fiscal_year` – The four-digit fiscal year (e.g., 2025).
    /// * `period_months` – Number of months to generate (1–12).
    pub fn generate_reports(
        &mut self,
        entity_code: &str,
        fiscal_year: u32,
        period_months: u32,
    ) -> Vec<ManagementReport> {
        let months = period_months.clamp(1, 12);
        let mut reports = Vec::with_capacity(months as usize * 2 + 4);

        for month in 1..=months {
            let period_label = format!("{fiscal_year}-{month:02}");

            // Flash report — quick preliminary numbers, prepared on day 5
            let flash_date = next_month_day(fiscal_year, month, 5);
            reports.push(self.generate_single(
                entity_code,
                "flash_report",
                &period_label,
                flash_date,
                6..=8,
                8..=10,
            ));

            // Monthly pack — full management pack, prepared on day 15
            let pack_date = next_month_day(fiscal_year, month, 15);
            reports.push(self.generate_single(
                entity_code,
                "monthly_pack",
                &period_label,
                pack_date,
                8..=10,
                10..=13,
            ));

            // Board report — one per quarter, prepared ~20 days after quarter end
            if month % 3 == 0 {
                let quarter = month / 3;
                let period_q = format!("{fiscal_year}-Q{quarter}");
                let board_date = next_month_day(fiscal_year, month, 20);
                reports.push(self.generate_single(
                    entity_code,
                    "board_report",
                    &period_q,
                    board_date,
                    8..=10,
                    12..=15,
                ));
            }
        }

        reports
    }

    /// Generate a single [`ManagementReport`].
    fn generate_single(
        &mut self,
        entity_code: &str,
        report_type: &str,
        period: &str,
        prepared_date: NaiveDate,
        kpi_range: std::ops::RangeInclusive<usize>,
        variance_range: std::ops::RangeInclusive<usize>,
    ) -> ManagementReport {
        let report_id = self.uuid_factory.next();

        let kpi_count = self.rng.random_range(*kpi_range.start()..=*kpi_range.end());
        let variance_count = self
            .rng
            .random_range(*variance_range.start()..=*variance_range.end());

        let kpi_summary = self.generate_kpi_summary(kpi_count);
        let budget_variances = self.generate_budget_variances(variance_count);
        let commentary = self.generate_commentary(&budget_variances);

        let preparer_num: u32 = self.rng.random_range(1..=5);
        let prepared_by = format!("FIN-ANALYST-{preparer_num:03}");

        ManagementReport {
            report_id,
            report_type: report_type.to_string(),
            period: period.to_string(),
            entity_code: entity_code.to_string(),
            prepared_by,
            prepared_date,
            kpi_summary,
            budget_variances,
            commentary,
        }
    }

    /// Generate KPI summary lines with RAG statuses.
    fn generate_kpi_summary(&mut self, count: usize) -> Vec<KpiSummaryLine> {
        // Pick a random subset of the available metric pool
        let mut indices: Vec<usize> = (0..KPI_METRICS.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);

        indices
            .into_iter()
            .map(|i| {
                let metric = KPI_METRICS[i];

                // Target in a realistic range (10–100)
                let target_raw: f64 = self.rng.random_range(10.0..100.0);
                let target = safe_decimal(target_raw, 2);

                // Actual = target * (1 + variance)
                // Distribution: 60% small (<5%), 30% medium (<10%), 10% large (>=10%)
                let variance_pct = self.sample_variance_pct();
                let actual_raw = target_raw * (1.0 + variance_pct / 100.0);
                let actual = safe_decimal(actual_raw, 2);

                let rag_status = rag_from_variance(variance_pct);

                KpiSummaryLine {
                    metric: metric.to_string(),
                    actual,
                    target,
                    variance_pct: (variance_pct * 100.0).round() / 100.0,
                    rag_status,
                }
            })
            .collect()
    }

    /// Generate budget variance lines.
    fn generate_budget_variances(&mut self, count: usize) -> Vec<BudgetVarianceLine> {
        let count = count.min(BUDGET_ACCOUNTS.len());
        let mut indices: Vec<usize> = (0..BUDGET_ACCOUNTS.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);

        indices
            .into_iter()
            .map(|i| {
                let (label, min_k, max_k) = BUDGET_ACCOUNTS[i];

                let budget_raw: f64 = self.rng.random_range(min_k..max_k) * 1_000.0;
                let budget_amount = safe_decimal(budget_raw, 2);

                let variance_pct = self.sample_variance_pct();
                let actual_raw = budget_raw * (1.0 + variance_pct / 100.0);
                let actual_amount = safe_decimal(actual_raw, 2);

                let variance = actual_amount - budget_amount;

                BudgetVarianceLine {
                    account: label.to_string(),
                    budget_amount,
                    actual_amount,
                    variance,
                    variance_pct: (variance_pct * 100.0).round() / 100.0,
                }
            })
            .collect()
    }

    /// Pick a variance percentage with a realistic distribution.
    ///
    /// 60% small (|v| < 5%), 30% medium (5% ≤ |v| < 10%), 10% large (|v| ≥ 10%)
    fn sample_variance_pct(&mut self) -> f64 {
        let bucket: f64 = self.rng.random();
        let sign: f64 = if self.rng.random_bool(0.5) { 1.0 } else { -1.0 };

        if bucket < 0.60 {
            sign * self.rng.random_range(0.0_f64..5.0)
        } else if bucket < 0.90 {
            sign * self.rng.random_range(5.0_f64..10.0)
        } else {
            sign * self.rng.random_range(10.0_f64..25.0)
        }
    }

    /// Generate a narrative commentary sentence based on overall budget position.
    fn generate_commentary(&mut self, variances: &[BudgetVarianceLine]) -> String {
        // Calculate average variance across all lines
        let avg_pct = if variances.is_empty() {
            0.0
        } else {
            variances.iter().map(|v| v.variance_pct).sum::<f64>() / variances.len() as f64
        };

        let pool = if avg_pct > 2.0 {
            POSITIVE_COMMENTARY
        } else if avg_pct < -2.0 {
            NEGATIVE_COMMENTARY
        } else {
            NEUTRAL_COMMENTARY
        };

        let idx = self.rng.random_range(0..pool.len());
        pool[idx].to_string()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Determine the RAG status from a variance percentage.
///
/// - green  : |variance| < 5%
/// - amber  : 5% ≤ |variance| < 10%
/// - red    : |variance| ≥ 10%
fn rag_from_variance(variance_pct: f64) -> String {
    let abs_v = variance_pct.abs();
    if abs_v < 5.0 {
        "green".to_string()
    } else if abs_v < 10.0 {
        "amber".to_string()
    } else {
        "red".to_string()
    }
}

/// Return a NaiveDate for `day` in the month following `(fiscal_year, month)`.
/// If `month == 12` the returned date is in January of `fiscal_year + 1`.
fn next_month_day(fiscal_year: u32, month: u32, day: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (fiscal_year as i32 + 1, 1u32)
    } else {
        (fiscal_year as i32, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, day)
        .or_else(|| NaiveDate::from_ymd_opt(y, m, 28))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(y, m, 1).unwrap_or_default())
}

/// Convert an f64 to Decimal with NaN/Inf safety, rounded to `dp` decimal places.
fn safe_decimal(raw: f64, dp: u32) -> Decimal {
    if raw.is_finite() {
        Decimal::from_f64_retain(raw)
            .unwrap_or(Decimal::ZERO)
            .round_dp(dp)
    } else {
        Decimal::ZERO
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_reports_generated_for_12_month_period() {
        let mut gen = ManagementReportGenerator::new(42);
        let reports = gen.generate_reports("C001", 2025, 12);

        // 12 flash + 12 monthly_pack + 4 board = 28 total
        assert_eq!(reports.len(), 28);

        // All reports must have non-empty required fields
        for r in &reports {
            assert!(!r.entity_code.is_empty());
            assert!(!r.period.is_empty());
            assert!(!r.report_type.is_empty());
            assert!(!r.prepared_by.is_empty());
            assert!(!r.commentary.is_empty());
        }
    }

    #[test]
    fn test_monthly_and_quarterly_report_types_present() {
        let mut gen = ManagementReportGenerator::new(99);
        let reports = gen.generate_reports("ENTITY_A", 2025, 12);

        let types: std::collections::HashSet<&str> =
            reports.iter().map(|r| r.report_type.as_str()).collect();

        assert!(types.contains("flash_report"), "Missing flash_report");
        assert!(types.contains("monthly_pack"), "Missing monthly_pack");
        assert!(types.contains("board_report"), "Missing board_report");

        // Counts
        let flash_count = reports
            .iter()
            .filter(|r| r.report_type == "flash_report")
            .count();
        let pack_count = reports
            .iter()
            .filter(|r| r.report_type == "monthly_pack")
            .count();
        let board_count = reports
            .iter()
            .filter(|r| r.report_type == "board_report")
            .count();
        assert_eq!(flash_count, 12);
        assert_eq!(pack_count, 12);
        assert_eq!(board_count, 4);
    }

    #[test]
    fn test_kpi_rag_status_consistent_with_variance() {
        let mut gen = ManagementReportGenerator::new(7);
        let reports = gen.generate_reports("C002", 2025, 3);

        for report in &reports {
            for kpi in &report.kpi_summary {
                let abs_v = kpi.variance_pct.abs();
                let expected_rag = if abs_v < 5.0 {
                    "green"
                } else if abs_v < 10.0 {
                    "amber"
                } else {
                    "red"
                };
                assert_eq!(
                    kpi.rag_status, expected_rag,
                    "RAG mismatch for metric '{}': variance_pct={:.2}, got '{}', expected '{}'",
                    kpi.metric, kpi.variance_pct, kpi.rag_status, expected_rag
                );
            }
        }
    }

    #[test]
    fn test_budget_variances_sum_correctly() {
        let mut gen = ManagementReportGenerator::new(1234);
        let reports = gen.generate_reports("C003", 2025, 1);

        for report in &reports {
            for line in &report.budget_variances {
                let expected = line.actual_amount - line.budget_amount;
                // Allow small rounding difference (< 0.01)
                let diff = (line.variance - expected).abs();
                assert!(
                    diff <= Decimal::from_f64_retain(0.01).unwrap_or(Decimal::ZERO),
                    "Variance arithmetic mismatch for account '{}': variance={}, expected={}",
                    line.account,
                    line.variance,
                    expected
                );
            }
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = ManagementReportGenerator::new(555);
        let reports = gen.generate_reports("C004", 2025, 1);

        assert!(!reports.is_empty());
        let report = &reports[0];

        let json = serde_json::to_string(report).expect("serialization failed");
        let roundtripped: ManagementReport =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(report.report_id, roundtripped.report_id);
        assert_eq!(report.report_type, roundtripped.report_type);
        assert_eq!(report.period, roundtripped.period);
        assert_eq!(report.entity_code, roundtripped.entity_code);
        assert_eq!(report.kpi_summary.len(), roundtripped.kpi_summary.len());
        assert_eq!(
            report.budget_variances.len(),
            roundtripped.budget_variances.len()
        );
        assert_eq!(report.commentary, roundtripped.commentary);
    }
}
