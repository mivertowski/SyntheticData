//! Trend plausibility evaluator.
//!
//! Validates that multi-period journal entry data exhibits internally consistent
//! financial trends. Checks revenue stability, expense ratio stability,
//! balance sheet growth consistency, and directional consistency between
//! revenue and accounts receivable.
//!
//! Accounts are classified by GL account prefix (first character):
//! - 1xxx → Assets
//! - 2xxx → Liabilities
//! - 4xxx → Revenue
//! - 5xxx–8xxx → Expenses

use datasynth_core::models::JournalEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─── Result types ─────────────────────────────────────────────────────────────

/// Result of a single trend consistency check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConsistencyCheck {
    /// Name of the check (e.g. "RevenueStability").
    pub check_type: String,
    /// Number of consecutive period pairs analyzed.
    pub periods_analyzed: usize,
    /// True when the check passes.
    pub is_consistent: bool,
    /// Human-readable explanation of the result.
    pub details: String,
}

/// Aggregate result of the trend plausibility evaluator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPlausibilityResult {
    /// Number of unique fiscal periods found in the data.
    pub period_count: usize,
    /// Individual check results.
    pub consistency_checks: Vec<TrendConsistencyCheck>,
    /// Fraction of checks that pass (0.0–1.0).
    pub overall_plausibility_score: f64,
    /// True when overall_plausibility_score ≥ 0.75.
    pub passes: bool,
}

// ─── Per-period aggregates ────────────────────────────────────────────────────

/// Financial totals for a single fiscal period.
#[derive(Debug, Default, Clone)]
struct PeriodTotals {
    revenue: Decimal,
    expenses: Decimal,
    assets: Decimal,
    liabilities: Decimal,
    /// Net credit to AR accounts (1100–1199 range).
    ar_net: Decimal,
}

/// Fiscal period key: (fiscal_year, fiscal_period).
type PeriodKey = (u16, u8);

// ─── Account classification helpers ──────────────────────────────────────────

fn is_revenue(account: &str) -> bool {
    account.starts_with('4')
}

fn is_expense(account: &str) -> bool {
    matches!(
        account.chars().next(),
        Some('5') | Some('6') | Some('7') | Some('8')
    )
}

fn is_asset(account: &str) -> bool {
    account.starts_with('1')
}

fn is_liability(account: &str) -> bool {
    account.starts_with('2')
}

/// Accounts Receivable: GL codes 1100–1199.
fn is_ar(account: &str) -> bool {
    account.starts_with("11")
}

// ─── Aggregation ──────────────────────────────────────────────────────────────

fn aggregate_by_period(entries: &[JournalEntry]) -> BTreeMap<PeriodKey, PeriodTotals> {
    let mut map: BTreeMap<PeriodKey, PeriodTotals> = BTreeMap::new();

    for entry in entries {
        let key = (entry.header.fiscal_year, entry.header.fiscal_period);
        let totals = map.entry(key).or_default();

        for line in &entry.lines {
            let account = &line.gl_account;
            let net = line.debit_amount - line.credit_amount;

            if is_revenue(account) {
                // Revenue is credited; net negative means credit balance → more revenue
                totals.revenue += line.credit_amount - line.debit_amount;
            }
            if is_expense(account) {
                totals.expenses += line.debit_amount - line.credit_amount;
            }
            if is_asset(account) {
                totals.assets += net;
            }
            if is_liability(account) {
                totals.liabilities += net;
            }
            if is_ar(account) {
                totals.ar_net += net;
            }
        }
    }

    map
}

// ─── Numeric helpers ──────────────────────────────────────────────────────────

fn to_f64(d: Decimal) -> f64 {
    d.to_string().parse::<f64>().unwrap_or(0.0)
}

/// Coefficient of variation: std_dev / mean (returns 0.0 when mean ≈ 0).
fn coefficient_of_variation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean.abs() < 1e-9 {
        return 0.0;
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt() / mean.abs()
}

// ─── Individual checks ────────────────────────────────────────────────────────

/// Check 1: Revenue doesn't swing > 50% period-over-period.
fn check_revenue_stability(periods: &[&PeriodTotals]) -> TrendConsistencyCheck {
    let check_type = "RevenueStability".to_string();

    if periods.len() < 2 {
        return TrendConsistencyCheck {
            check_type,
            periods_analyzed: periods.len().saturating_sub(1),
            is_consistent: true,
            details: "Insufficient periods for comparison".to_string(),
        };
    }

    let mut violations = 0usize;
    let mut comparisons = 0usize;

    for window in periods.windows(2) {
        let prev = to_f64(window[0].revenue);
        let curr = to_f64(window[1].revenue);

        if prev.abs() < 1.0 {
            // Skip if previous period revenue is essentially zero
            continue;
        }

        comparisons += 1;
        let change = ((curr - prev) / prev.abs()).abs();
        if change > 0.50 {
            violations += 1;
        }
    }

    let is_consistent = violations == 0 || comparisons == 0;
    let details = if comparisons == 0 {
        "All revenue values near zero; check vacuously passes".to_string()
    } else {
        format!("{violations} of {comparisons} period-over-period revenue swings exceeded 50%")
    };

    TrendConsistencyCheck {
        check_type,
        periods_analyzed: comparisons,
        is_consistent,
        details,
    }
}

/// Check 2: Expense/revenue ratio CV < 0.30.
fn check_expense_ratio_stability(periods: &[&PeriodTotals]) -> TrendConsistencyCheck {
    let check_type = "ExpenseRatioStability".to_string();

    if periods.len() < 2 {
        return TrendConsistencyCheck {
            check_type,
            periods_analyzed: 0,
            is_consistent: true,
            details: "Insufficient periods for comparison".to_string(),
        };
    }

    let ratios: Vec<f64> = periods
        .iter()
        .filter_map(|p| {
            let rev = to_f64(p.revenue);
            let exp = to_f64(p.expenses);
            if rev.abs() < 1.0 {
                None
            } else {
                Some(exp / rev)
            }
        })
        .collect();

    if ratios.len() < 2 {
        return TrendConsistencyCheck {
            check_type,
            periods_analyzed: 0,
            is_consistent: true,
            details: "Insufficient non-zero revenue periods for ratio analysis".to_string(),
        };
    }

    let cv = coefficient_of_variation(&ratios);
    let is_consistent = cv < 0.30;

    TrendConsistencyCheck {
        check_type,
        periods_analyzed: ratios.len(),
        is_consistent,
        details: format!("Expense/revenue ratio CV = {cv:.3} (threshold: < 0.30)"),
    }
}

/// Check 3: Asset growth ≈ liability growth (within 25%).
fn check_balance_sheet_growth_consistency(periods: &[&PeriodTotals]) -> TrendConsistencyCheck {
    let check_type = "BalanceSheetGrowthConsistency".to_string();

    if periods.len() < 2 {
        return TrendConsistencyCheck {
            check_type,
            periods_analyzed: 0,
            is_consistent: true,
            details: "Insufficient periods for comparison".to_string(),
        };
    }

    let mut violations = 0usize;
    let mut comparisons = 0usize;

    for window in periods.windows(2) {
        let asset_prev = to_f64(window[0].assets);
        let asset_curr = to_f64(window[1].assets);
        let liab_prev = to_f64(window[0].liabilities);
        let liab_curr = to_f64(window[1].liabilities);

        if asset_prev.abs() < 1.0 && liab_prev.abs() < 1.0 {
            continue;
        }

        comparisons += 1;
        let asset_growth = if asset_prev.abs() > 1.0 {
            (asset_curr - asset_prev) / asset_prev.abs()
        } else {
            0.0
        };
        let liab_growth = if liab_prev.abs() > 1.0 {
            (liab_curr - liab_prev) / liab_prev.abs()
        } else {
            0.0
        };

        if (asset_growth - liab_growth).abs() > 0.25 {
            violations += 1;
        }
    }

    let is_consistent = violations == 0 || comparisons == 0;
    TrendConsistencyCheck {
        check_type,
        periods_analyzed: comparisons,
        is_consistent,
        details: format!(
            "{violations} of {comparisons} periods showed asset/liability growth divergence > 25%"
        ),
    }
}

/// Check 4: If revenue grows, AR should grow in the same direction.
fn check_directional_consistency(periods: &[&PeriodTotals]) -> TrendConsistencyCheck {
    let check_type = "DirectionalConsistency".to_string();

    if periods.len() < 2 {
        return TrendConsistencyCheck {
            check_type,
            periods_analyzed: 0,
            is_consistent: true,
            details: "Insufficient periods for comparison".to_string(),
        };
    }

    let mut violations = 0usize;
    let mut comparisons = 0usize;

    for window in periods.windows(2) {
        let rev_delta = to_f64(window[1].revenue) - to_f64(window[0].revenue);
        let ar_delta = to_f64(window[1].ar_net) - to_f64(window[0].ar_net);

        // Only test if both revenue and AR are non-trivially present
        let rev_magnitude = to_f64(window[0].revenue)
            .abs()
            .max(to_f64(window[1].revenue).abs());
        if rev_magnitude < 1.0 {
            continue;
        }

        comparisons += 1;
        // Directional mismatch: revenue grows but AR shrinks, or vice versa (significant change)
        let significant_rev_change = rev_delta.abs() > rev_magnitude * 0.10;
        if significant_rev_change && rev_delta * ar_delta < 0.0 {
            violations += 1;
        }
    }

    let is_consistent = violations == 0 || comparisons == 0;
    TrendConsistencyCheck {
        check_type,
        periods_analyzed: comparisons,
        is_consistent,
        details: format!(
            "{violations} of {comparisons} periods showed revenue/AR directional mismatch"
        ),
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Analyze trend plausibility across all journal entries.
///
/// Groups entries by fiscal period, computes per-period account totals, and
/// evaluates four trend consistency checks.
///
/// # Returns
/// A `TrendPlausibilityResult` with individual check results and an aggregate
/// plausibility score. Passes when ≥ 75% of checks are consistent.
pub fn analyze_trends(entries: &[JournalEntry]) -> TrendPlausibilityResult {
    let period_map = aggregate_by_period(entries);
    let period_count = period_map.len();

    // Collect period totals in chronological order (BTreeMap is sorted by key).
    let ordered: Vec<&PeriodTotals> = period_map.values().collect();

    let checks = vec![
        check_revenue_stability(&ordered),
        check_expense_ratio_stability(&ordered),
        check_balance_sheet_growth_consistency(&ordered),
        check_directional_consistency(&ordered),
    ];

    let passing = checks.iter().filter(|c| c.is_consistent).count();
    let total = checks.len();
    let overall_plausibility_score = if total > 0 {
        passing as f64 / total as f64
    } else {
        1.0
    };

    let passes = overall_plausibility_score >= 0.75;

    TrendPlausibilityResult {
        period_count,
        consistency_checks: checks,
        overall_plausibility_score,
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

    /// Create a JE with one debit and one credit line in a given fiscal period.
    fn make_je(period: u8, debit_acct: &str, credit_acct: &str, amount: Decimal) -> JournalEntry {
        // Use month = period for simplicity (valid for periods 1–12)
        let m = period.clamp(1, 12);
        let posting_date = date(2024, m as u32, 1);
        let mut header = JournalEntryHeader::new("C001".to_string(), posting_date);
        header.fiscal_period = period;
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);
        entry.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            debit_acct.to_string(),
            amount,
        ));
        entry.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            credit_acct.to_string(),
            amount,
        ));
        entry
    }

    /// Helper: build stable revenue across N periods (identical amount).
    fn stable_revenue_entries(periods: u8, amount: Decimal) -> Vec<JournalEntry> {
        (1..=periods)
            .map(|p| make_je(p, "1100", "4000", amount)) // debit AR, credit Revenue
            .collect()
    }

    #[test]
    fn test_empty_entries() {
        let result = analyze_trends(&[]);
        assert_eq!(result.period_count, 0);
        // All checks vacuously pass (insufficient periods)
        assert!(result.passes);
    }

    #[test]
    fn test_single_period() {
        let entries = stable_revenue_entries(1, dec!(100_000));
        let result = analyze_trends(&entries);
        assert_eq!(result.period_count, 1);
        // All checks vacuously pass
        assert!(result.passes);
    }

    #[test]
    fn test_stable_revenue_passes() {
        // Identical revenue each period → zero variance → passes
        let entries = stable_revenue_entries(6, dec!(100_000));
        let result = analyze_trends(&entries);
        assert_eq!(result.period_count, 6);
        let rev_check = result
            .consistency_checks
            .iter()
            .find(|c| c.check_type == "RevenueStability")
            .unwrap();
        assert!(rev_check.is_consistent, "{}", rev_check.details);
    }

    #[test]
    fn test_volatile_revenue_fails() {
        // Double revenue each period → > 50% swing
        let mut entries = Vec::new();
        let mut amount = dec!(10_000);
        for period in 1u8..=4 {
            entries.push(make_je(period, "1100", "4000", amount));
            amount *= dec!(3); // 200% increase → far above 50% threshold
        }
        let result = analyze_trends(&entries);
        let rev_check = result
            .consistency_checks
            .iter()
            .find(|c| c.check_type == "RevenueStability")
            .unwrap();
        assert!(!rev_check.is_consistent, "3× revenue growth should fail");
    }

    #[test]
    fn test_plausibility_score_range() {
        let entries = stable_revenue_entries(4, dec!(50_000));
        let result = analyze_trends(&entries);
        assert!(
            result.overall_plausibility_score >= 0.0 && result.overall_plausibility_score <= 1.0
        );
    }

    #[test]
    fn test_passes_threshold() {
        // Stable data should have score ≥ 0.75
        let entries = stable_revenue_entries(6, dec!(100_000));
        let result = analyze_trends(&entries);
        assert!(
            result.passes,
            "Stable data should pass. Score: {}",
            result.overall_plausibility_score
        );
    }

    #[test]
    fn test_period_count_correct() {
        let entries = stable_revenue_entries(3, dec!(50_000));
        let result = analyze_trends(&entries);
        assert_eq!(result.period_count, 3);
    }

    #[test]
    fn test_check_count() {
        let entries = stable_revenue_entries(4, dec!(100_000));
        let result = analyze_trends(&entries);
        assert_eq!(result.consistency_checks.len(), 4);
        let names: Vec<&str> = result
            .consistency_checks
            .iter()
            .map(|c| c.check_type.as_str())
            .collect();
        assert!(names.contains(&"RevenueStability"));
        assert!(names.contains(&"ExpenseRatioStability"));
        assert!(names.contains(&"BalanceSheetGrowthConsistency"));
        assert!(names.contains(&"DirectionalConsistency"));
    }

    #[test]
    fn test_cv_calculation() {
        // Test coefficient_of_variation directly
        let values = vec![1.0, 1.0, 1.0, 1.0];
        assert!((coefficient_of_variation(&values) - 0.0).abs() < 1e-9);

        let values2 = vec![1.0, 2.0, 3.0, 4.0];
        let cv = coefficient_of_variation(&values2);
        assert!(cv > 0.0);
    }
}
