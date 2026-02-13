//! HR/Payroll evaluator.
//!
//! Validates payroll arithmetic coherence including gross-to-net calculations,
//! component sums, run totals, time entry mapping, and expense report consistency.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for HR/payroll evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrPayrollThresholds {
    /// Minimum calculation accuracy (arithmetic should be near-exact).
    pub min_calculation_accuracy: f64,
    /// Tolerance for floating-point comparisons.
    pub tolerance: f64,
}

impl Default for HrPayrollThresholds {
    fn default() -> Self {
        Self {
            min_calculation_accuracy: 0.999,
            tolerance: 0.01,
        }
    }
}

/// Payroll line item data for validation.
#[derive(Debug, Clone)]
pub struct PayrollLineItemData {
    /// Employee identifier.
    pub employee_id: String,
    /// Gross pay.
    pub gross_pay: f64,
    /// Base salary component.
    pub base_pay: f64,
    /// Overtime component.
    pub overtime_pay: f64,
    /// Bonus component.
    pub bonus_pay: f64,
    /// Net pay.
    pub net_pay: f64,
    /// Total deductions.
    pub total_deductions: f64,
    /// Tax deduction.
    pub tax_deduction: f64,
    /// Social security deduction.
    pub social_security: f64,
    /// Health insurance deduction.
    pub health_insurance: f64,
    /// Retirement contribution.
    pub retirement: f64,
    /// Other deductions.
    pub other_deductions: f64,
}

/// Payroll run data for validation.
#[derive(Debug, Clone)]
pub struct PayrollRunData {
    /// Run identifier.
    pub run_id: String,
    /// Reported total net pay for the run.
    pub total_net_pay: f64,
    /// Line items in this run.
    pub line_items: Vec<PayrollLineItemData>,
}

/// Time entry data for validation.
#[derive(Debug, Clone)]
pub struct TimeEntryData {
    /// Employee identifier.
    pub employee_id: String,
    /// Total hours from time entries for the period.
    pub total_hours: f64,
}

/// Payroll hours for an employee from payroll records.
#[derive(Debug, Clone)]
pub struct PayrollHoursData {
    /// Employee identifier.
    pub employee_id: String,
    /// Hours recorded in payroll.
    pub payroll_hours: f64,
}

/// Expense report data.
#[derive(Debug, Clone)]
pub struct ExpenseReportData {
    /// Report identifier.
    pub report_id: String,
    /// Reported total amount.
    pub total_amount: f64,
    /// Sum of line item amounts.
    pub line_items_sum: f64,
    /// Whether the report is approved.
    pub is_approved: bool,
    /// Whether the report has an approver assigned.
    pub has_approver: bool,
}

/// Results of HR/payroll evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrPayrollEvaluation {
    /// Gross-to-net accuracy: fraction of line items where net = gross - deductions.
    pub gross_to_net_accuracy: f64,
    /// Component sum accuracy: fraction where gross = base + OT + bonus.
    pub component_sum_accuracy: f64,
    /// Deduction sum accuracy: fraction where total_deductions = tax + SS + health + retirement + other.
    pub deduction_sum_accuracy: f64,
    /// Run sum accuracy: fraction where run total = SUM(line_items.net_pay).
    pub run_sum_accuracy: f64,
    /// Time-to-payroll mapping rate: fraction of employees with matching hours.
    pub time_to_payroll_mapping_rate: f64,
    /// Expense line item sum accuracy: fraction where report total = SUM(line_items).
    pub expense_line_item_sum_accuracy: f64,
    /// Expense approval consistency: fraction of approved reports with approver.
    pub expense_approval_consistency: f64,
    /// Total line items checked.
    pub total_line_items: usize,
    /// Total runs checked.
    pub total_runs: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for HR/payroll coherence.
pub struct HrPayrollEvaluator {
    thresholds: HrPayrollThresholds,
}

impl HrPayrollEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: HrPayrollThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: HrPayrollThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate payroll data.
    pub fn evaluate(
        &self,
        runs: &[PayrollRunData],
        time_entries: &[TimeEntryData],
        payroll_hours: &[PayrollHoursData],
        expense_reports: &[ExpenseReportData],
    ) -> EvalResult<HrPayrollEvaluation> {
        let mut issues = Vec::new();
        let tol = self.thresholds.tolerance;

        // Collect all line items
        let all_items: Vec<&PayrollLineItemData> =
            runs.iter().flat_map(|r| r.line_items.iter()).collect();
        let total_line_items = all_items.len();

        // 1. Gross-to-net: net = gross - deductions
        let gross_to_net_ok = all_items
            .iter()
            .filter(|li| (li.net_pay - (li.gross_pay - li.total_deductions)).abs() <= tol)
            .count();
        let gross_to_net_accuracy = if total_line_items > 0 {
            gross_to_net_ok as f64 / total_line_items as f64
        } else {
            1.0
        };

        // 2. Component sums: gross = base + OT + bonus
        let component_ok = all_items
            .iter()
            .filter(|li| {
                (li.gross_pay - (li.base_pay + li.overtime_pay + li.bonus_pay)).abs() <= tol
            })
            .count();
        let component_sum_accuracy = if total_line_items > 0 {
            component_ok as f64 / total_line_items as f64
        } else {
            1.0
        };

        // 3. Deduction sums
        let deduction_ok = all_items
            .iter()
            .filter(|li| {
                let computed = li.tax_deduction
                    + li.social_security
                    + li.health_insurance
                    + li.retirement
                    + li.other_deductions;
                (li.total_deductions - computed).abs() <= tol
            })
            .count();
        let deduction_sum_accuracy = if total_line_items > 0 {
            deduction_ok as f64 / total_line_items as f64
        } else {
            1.0
        };

        // 4. Run totals
        let total_runs = runs.len();
        let run_ok = runs
            .iter()
            .filter(|run| {
                let computed_total: f64 = run.line_items.iter().map(|li| li.net_pay).sum();
                (run.total_net_pay - computed_total).abs() <= tol
            })
            .count();
        let run_sum_accuracy = if total_runs > 0 {
            run_ok as f64 / total_runs as f64
        } else {
            1.0
        };

        // 5. Time entry mapping
        let time_map: std::collections::HashMap<&str, f64> = time_entries
            .iter()
            .map(|te| (te.employee_id.as_str(), te.total_hours))
            .collect();
        let mapped_count = payroll_hours
            .iter()
            .filter(|ph| {
                time_map
                    .get(ph.employee_id.as_str())
                    .map(|&hours| (hours - ph.payroll_hours).abs() <= 1.0)
                    .unwrap_or(false)
            })
            .count();
        let time_to_payroll_mapping_rate = if payroll_hours.is_empty() {
            1.0
        } else {
            mapped_count as f64 / payroll_hours.len() as f64
        };

        // 6. Expense reports
        let expense_sum_ok = expense_reports
            .iter()
            .filter(|er| (er.total_amount - er.line_items_sum).abs() <= tol)
            .count();
        let expense_line_item_sum_accuracy = if expense_reports.is_empty() {
            1.0
        } else {
            expense_sum_ok as f64 / expense_reports.len() as f64
        };

        let approved_reports: Vec<&ExpenseReportData> =
            expense_reports.iter().filter(|er| er.is_approved).collect();
        let approval_consistent = approved_reports.iter().filter(|er| er.has_approver).count();
        let expense_approval_consistency = if approved_reports.is_empty() {
            1.0
        } else {
            approval_consistent as f64 / approved_reports.len() as f64
        };

        // Check thresholds
        let min_acc = self.thresholds.min_calculation_accuracy;
        if gross_to_net_accuracy < min_acc {
            issues.push(format!(
                "Gross-to-net accuracy {:.4} < {:.4}",
                gross_to_net_accuracy, min_acc
            ));
        }
        if component_sum_accuracy < min_acc {
            issues.push(format!(
                "Component sum accuracy {:.4} < {:.4}",
                component_sum_accuracy, min_acc
            ));
        }
        if deduction_sum_accuracy < min_acc {
            issues.push(format!(
                "Deduction sum accuracy {:.4} < {:.4}",
                deduction_sum_accuracy, min_acc
            ));
        }
        if run_sum_accuracy < min_acc {
            issues.push(format!(
                "Run sum accuracy {:.4} < {:.4}",
                run_sum_accuracy, min_acc
            ));
        }

        let passes = issues.is_empty();

        Ok(HrPayrollEvaluation {
            gross_to_net_accuracy,
            component_sum_accuracy,
            deduction_sum_accuracy,
            run_sum_accuracy,
            time_to_payroll_mapping_rate,
            expense_line_item_sum_accuracy,
            expense_approval_consistency,
            total_line_items,
            total_runs,
            passes,
            issues,
        })
    }
}

impl Default for HrPayrollEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn valid_line_item() -> PayrollLineItemData {
        PayrollLineItemData {
            employee_id: "EMP001".to_string(),
            gross_pay: 5000.0,
            base_pay: 4000.0,
            overtime_pay: 500.0,
            bonus_pay: 500.0,
            net_pay: 3500.0,
            total_deductions: 1500.0,
            tax_deduction: 800.0,
            social_security: 300.0,
            health_insurance: 200.0,
            retirement: 150.0,
            other_deductions: 50.0,
        }
    }

    #[test]
    fn test_valid_payroll() {
        let evaluator = HrPayrollEvaluator::new();
        let runs = vec![PayrollRunData {
            run_id: "PR001".to_string(),
            total_net_pay: 3500.0,
            line_items: vec![valid_line_item()],
        }];

        let result = evaluator.evaluate(&runs, &[], &[], &[]).unwrap();
        assert!(result.passes);
        assert_eq!(result.gross_to_net_accuracy, 1.0);
        assert_eq!(result.component_sum_accuracy, 1.0);
        assert_eq!(result.run_sum_accuracy, 1.0);
    }

    #[test]
    fn test_broken_gross_to_net() {
        let evaluator = HrPayrollEvaluator::new();
        let mut item = valid_line_item();
        item.net_pay = 4000.0; // Wrong: should be 3500

        let runs = vec![PayrollRunData {
            run_id: "PR001".to_string(),
            total_net_pay: 4000.0,
            line_items: vec![item],
        }];

        let result = evaluator.evaluate(&runs, &[], &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.gross_to_net_accuracy < 1.0);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = HrPayrollEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[], &[]).unwrap();
        assert!(result.passes);
    }

    #[test]
    fn test_expense_report_consistency() {
        let evaluator = HrPayrollEvaluator::new();
        let expenses = vec![
            ExpenseReportData {
                report_id: "ER001".to_string(),
                total_amount: 500.0,
                line_items_sum: 500.0,
                is_approved: true,
                has_approver: true,
            },
            ExpenseReportData {
                report_id: "ER002".to_string(),
                total_amount: 300.0,
                line_items_sum: 300.0,
                is_approved: true,
                has_approver: false, // Approved but no approver
            },
        ];

        let result = evaluator.evaluate(&[], &[], &[], &expenses).unwrap();
        assert_eq!(result.expense_line_item_sum_accuracy, 1.0);
        assert_eq!(result.expense_approval_consistency, 0.5);
    }
}
