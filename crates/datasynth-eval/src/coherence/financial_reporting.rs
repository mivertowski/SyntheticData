//! Financial reporting evaluator.
//!
//! Validates financial statement coherence including balance sheet equation,
//! statement-to-trial-balance tie-back, cash flow reconciliation,
//! KPI derivation accuracy, and budget variance realism.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for financial reporting evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReportingThresholds {
    /// Minimum rate at which statement line items tie back to trial balance.
    pub min_statement_tb_tie_rate: f64,
    /// Minimum KPI derivation accuracy.
    pub min_kpi_accuracy: f64,
    /// Maximum budget variance standard deviation (as fraction of budget).
    pub max_budget_variance_std: f64,
    /// Tolerance for balance equation checks.
    pub balance_tolerance: f64,
}

impl Default for FinancialReportingThresholds {
    fn default() -> Self {
        Self {
            min_statement_tb_tie_rate: 0.99,
            min_kpi_accuracy: 0.95,
            max_budget_variance_std: 0.50,
            balance_tolerance: 0.01,
        }
    }
}

/// Input data for a financial statement period.
#[derive(Debug, Clone)]
pub struct FinancialStatementData {
    /// Period identifier (e.g., "2024-Q1").
    pub period: String,
    /// Total assets from balance sheet.
    pub total_assets: f64,
    /// Total liabilities from balance sheet.
    pub total_liabilities: f64,
    /// Total equity from balance sheet.
    pub total_equity: f64,
    /// Statement line item totals by GL account.
    pub line_item_totals: Vec<(String, f64)>,
    /// Trial balance totals by GL account for the same period.
    pub trial_balance_totals: Vec<(String, f64)>,
    /// Operating cash flow.
    pub cash_flow_operating: f64,
    /// Investing cash flow.
    pub cash_flow_investing: f64,
    /// Financing cash flow.
    pub cash_flow_financing: f64,
    /// Beginning cash balance.
    pub cash_beginning: f64,
    /// Ending cash balance.
    pub cash_ending: f64,
}

/// Input data for KPI validation.
#[derive(Debug, Clone)]
pub struct KpiData {
    /// KPI name.
    pub name: String,
    /// Reported KPI value.
    pub reported_value: f64,
    /// Computed KPI value from underlying GL data.
    pub computed_value: f64,
}

/// Input data for budget variance.
#[derive(Debug, Clone)]
pub struct BudgetVarianceData {
    /// Budget line item name.
    pub line_item: String,
    /// Budgeted amount.
    pub budget_amount: f64,
    /// Actual amount.
    pub actual_amount: f64,
}

/// Per-period balance sheet result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodBsResult {
    /// Period identifier.
    pub period: String,
    /// Whether A = L + E within tolerance.
    pub balanced: bool,
    /// Imbalance amount.
    pub imbalance: f64,
}

/// Per-period cash flow result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowResult {
    /// Period identifier.
    pub period: String,
    /// Whether cash flow reconciles.
    pub reconciled: bool,
    /// Discrepancy amount.
    pub discrepancy: f64,
}

/// Results of financial reporting evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReportingEvaluation {
    /// Whether all balance sheets are balanced (A = L + E).
    pub bs_equation_balanced: bool,
    /// Per-period balance sheet results.
    pub period_bs_results: Vec<PeriodBsResult>,
    /// Rate at which statement line items tie to trial balance.
    pub statement_tb_tie_rate: f64,
    /// Number of tie-back mismatches.
    pub tie_back_mismatches: usize,
    /// Whether all cash flow statements reconcile.
    pub cash_flow_reconciled: bool,
    /// Per-period cash flow results.
    pub period_cf_results: Vec<CashFlowResult>,
    /// KPI derivation accuracy (fraction of KPIs within tolerance).
    pub kpi_derivation_accuracy: f64,
    /// Number of KPI derivation mismatches.
    pub kpi_mismatches: usize,
    /// Budget variance standard deviation (normalized).
    pub budget_variance_std: f64,
    /// Whether budget variance is within bounds.
    pub budget_variance_within_bounds: bool,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for financial reporting coherence.
pub struct FinancialReportingEvaluator {
    thresholds: FinancialReportingThresholds,
}

impl FinancialReportingEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: FinancialReportingThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: FinancialReportingThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate financial reporting data.
    pub fn evaluate(
        &self,
        statements: &[FinancialStatementData],
        kpis: &[KpiData],
        budget_variances: &[BudgetVarianceData],
    ) -> EvalResult<FinancialReportingEvaluation> {
        let mut issues = Vec::new();

        // 1. Balance sheet equation: A = L + E
        let mut period_bs_results = Vec::new();
        let mut all_balanced = true;
        for stmt in statements {
            let imbalance = stmt.total_assets - (stmt.total_liabilities + stmt.total_equity);
            let balanced = imbalance.abs() <= self.thresholds.balance_tolerance;
            if !balanced {
                all_balanced = false;
                issues.push(format!(
                    "BS imbalance in {}: {:.2} (A={:.2}, L={:.2}, E={:.2})",
                    stmt.period,
                    imbalance,
                    stmt.total_assets,
                    stmt.total_liabilities,
                    stmt.total_equity
                ));
            }
            period_bs_results.push(PeriodBsResult {
                period: stmt.period.clone(),
                balanced,
                imbalance,
            });
        }

        // 2. Statement-to-trial-balance tie-back
        let mut total_line_items = 0usize;
        let mut matched_line_items = 0usize;
        for stmt in statements {
            let tb_map: std::collections::HashMap<&str, f64> = stmt
                .trial_balance_totals
                .iter()
                .map(|(k, v)| (k.as_str(), *v))
                .collect();
            for (account, amount) in &stmt.line_item_totals {
                total_line_items += 1;
                if let Some(&tb_amount) = tb_map.get(account.as_str()) {
                    if (amount - tb_amount).abs() <= self.thresholds.balance_tolerance {
                        matched_line_items += 1;
                    }
                }
            }
        }
        let statement_tb_tie_rate = if total_line_items > 0 {
            matched_line_items as f64 / total_line_items as f64
        } else {
            1.0
        };
        let tie_back_mismatches = total_line_items - matched_line_items;
        if statement_tb_tie_rate < self.thresholds.min_statement_tb_tie_rate {
            issues.push(format!(
                "Statement-TB tie rate {:.3} < {:.3} threshold ({} mismatches)",
                statement_tb_tie_rate,
                self.thresholds.min_statement_tb_tie_rate,
                tie_back_mismatches
            ));
        }

        // 3. Cash flow reconciliation
        let mut period_cf_results = Vec::new();
        let mut all_reconciled = true;
        for stmt in statements {
            let computed_ending = stmt.cash_beginning
                + stmt.cash_flow_operating
                + stmt.cash_flow_investing
                + stmt.cash_flow_financing;
            let discrepancy = (stmt.cash_ending - computed_ending).abs();
            let reconciled = discrepancy <= self.thresholds.balance_tolerance;
            if !reconciled {
                all_reconciled = false;
                issues.push(format!(
                    "Cash flow not reconciled in {}: discrepancy {:.2}",
                    stmt.period, discrepancy
                ));
            }
            period_cf_results.push(CashFlowResult {
                period: stmt.period.clone(),
                reconciled,
                discrepancy,
            });
        }

        // 4. KPI derivation accuracy
        let mut kpi_matches = 0usize;
        for kpi in kpis {
            let denominator = if kpi.computed_value.abs() > f64::EPSILON {
                kpi.computed_value.abs()
            } else {
                1.0
            };
            let error = (kpi.reported_value - kpi.computed_value).abs() / denominator;
            if error <= 0.05 {
                kpi_matches += 1;
            }
        }
        let kpi_derivation_accuracy = if kpis.is_empty() {
            1.0
        } else {
            kpi_matches as f64 / kpis.len() as f64
        };
        let kpi_mismatches = kpis.len() - kpi_matches;
        if kpi_derivation_accuracy < self.thresholds.min_kpi_accuracy {
            issues.push(format!(
                "KPI derivation accuracy {:.3} < {:.3} threshold ({} mismatches)",
                kpi_derivation_accuracy, self.thresholds.min_kpi_accuracy, kpi_mismatches
            ));
        }

        // 5. Budget variance realism
        let variance_ratios: Vec<f64> = budget_variances
            .iter()
            .filter(|bv| bv.budget_amount.abs() > f64::EPSILON)
            .map(|bv| (bv.actual_amount - bv.budget_amount) / bv.budget_amount)
            .collect();

        let budget_variance_std = if variance_ratios.len() >= 2 {
            let mean = variance_ratios.iter().sum::<f64>() / variance_ratios.len() as f64;
            let variance = variance_ratios
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / (variance_ratios.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        let budget_variance_within_bounds =
            budget_variance_std <= self.thresholds.max_budget_variance_std;
        if !budget_variance_within_bounds && !variance_ratios.is_empty() {
            issues.push(format!(
                "Budget variance std {:.3} > {:.3} threshold",
                budget_variance_std, self.thresholds.max_budget_variance_std
            ));
        }

        let passes = issues.is_empty();

        Ok(FinancialReportingEvaluation {
            bs_equation_balanced: all_balanced,
            period_bs_results,
            statement_tb_tie_rate,
            tie_back_mismatches,
            cash_flow_reconciled: all_reconciled,
            period_cf_results,
            kpi_derivation_accuracy,
            kpi_mismatches,
            budget_variance_std,
            budget_variance_within_bounds,
            passes,
            issues,
        })
    }
}

impl Default for FinancialReportingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_statement() -> FinancialStatementData {
        FinancialStatementData {
            period: "2024-Q1".to_string(),
            total_assets: 1_000_000.0,
            total_liabilities: 600_000.0,
            total_equity: 400_000.0,
            line_item_totals: vec![
                ("1100".to_string(), 500_000.0),
                ("2000".to_string(), 300_000.0),
            ],
            trial_balance_totals: vec![
                ("1100".to_string(), 500_000.0),
                ("2000".to_string(), 300_000.0),
            ],
            cash_flow_operating: 50_000.0,
            cash_flow_investing: -20_000.0,
            cash_flow_financing: -10_000.0,
            cash_beginning: 100_000.0,
            cash_ending: 120_000.0,
        }
    }

    #[test]
    fn test_valid_financial_reporting() {
        let evaluator = FinancialReportingEvaluator::new();
        let stmts = vec![valid_statement()];
        let kpis = vec![KpiData {
            name: "ROA".to_string(),
            reported_value: 0.05,
            computed_value: 0.05,
        }];
        let budgets = vec![
            BudgetVarianceData {
                line_item: "Revenue".to_string(),
                budget_amount: 100_000.0,
                actual_amount: 105_000.0,
            },
            BudgetVarianceData {
                line_item: "COGS".to_string(),
                budget_amount: 60_000.0,
                actual_amount: 58_000.0,
            },
        ];

        let result = evaluator.evaluate(&stmts, &kpis, &budgets).unwrap();
        assert!(result.passes);
        assert!(result.bs_equation_balanced);
        assert!(result.cash_flow_reconciled);
        assert_eq!(result.statement_tb_tie_rate, 1.0);
        assert_eq!(result.kpi_derivation_accuracy, 1.0);
    }

    #[test]
    fn test_imbalanced_balance_sheet() {
        let evaluator = FinancialReportingEvaluator::new();
        let mut stmt = valid_statement();
        stmt.total_assets = 1_000_000.0;
        stmt.total_liabilities = 500_000.0;
        stmt.total_equity = 400_000.0; // 100k gap

        let result = evaluator.evaluate(&[stmt], &[], &[]).unwrap();
        assert!(!result.bs_equation_balanced);
        assert!(!result.passes);
    }

    #[test]
    fn test_cash_flow_mismatch() {
        let evaluator = FinancialReportingEvaluator::new();
        let mut stmt = valid_statement();
        stmt.cash_ending = 200_000.0; // Wrong

        let result = evaluator.evaluate(&[stmt], &[], &[]).unwrap();
        assert!(!result.cash_flow_reconciled);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = FinancialReportingEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[]).unwrap();
        assert!(result.passes);
        assert_eq!(result.kpi_derivation_accuracy, 1.0);
    }

    #[test]
    fn test_kpi_mismatch() {
        let evaluator = FinancialReportingEvaluator::new();
        let kpis = vec![
            KpiData {
                name: "ROA".to_string(),
                reported_value: 0.10,
                computed_value: 0.05, // 100% error
            },
            KpiData {
                name: "ROE".to_string(),
                reported_value: 0.15,
                computed_value: 0.15, // exact match
            },
        ];

        let result = evaluator.evaluate(&[], &kpis, &[]).unwrap();
        assert_eq!(result.kpi_derivation_accuracy, 0.5);
        assert_eq!(result.kpi_mismatches, 1);
    }
}
