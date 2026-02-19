//! Project accounting coherence evaluator.
//!
//! Validates percentage-of-completion revenue recognition, earned value
//! management (EVM) metrics, and retainage balance calculations.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for project accounting evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAccountingThresholds {
    /// Minimum accuracy for EVM metric calculations.
    pub min_evm_accuracy: f64,
    /// Minimum accuracy for PoC completion percentage.
    pub min_poc_accuracy: f64,
    /// Tolerance for EVM comparisons.
    pub evm_tolerance: f64,
}

impl Default for ProjectAccountingThresholds {
    fn default() -> Self {
        Self {
            min_evm_accuracy: 0.999,
            min_poc_accuracy: 0.99,
            evm_tolerance: 0.01,
        }
    }
}

/// Project revenue data for PoC validation.
#[derive(Debug, Clone)]
pub struct ProjectRevenueData {
    /// Project identifier.
    pub project_id: String,
    /// Costs incurred to date.
    pub costs_to_date: f64,
    /// Estimated total cost at completion.
    pub estimated_total_cost: f64,
    /// Reported completion percentage.
    pub completion_pct: f64,
    /// Total contract value.
    pub contract_value: f64,
    /// Cumulative revenue recognized.
    pub cumulative_revenue: f64,
    /// Amount billed to date.
    pub billed_to_date: f64,
    /// Unbilled revenue balance.
    pub unbilled_revenue: f64,
}

/// Earned value management data for EVM validation.
#[derive(Debug, Clone)]
pub struct EarnedValueData {
    /// Project identifier.
    pub project_id: String,
    /// Planned value (BCWS).
    pub planned_value: f64,
    /// Earned value (BCWP).
    pub earned_value: f64,
    /// Actual cost (ACWP).
    pub actual_cost: f64,
    /// Budget at completion.
    pub bac: f64,
    /// Schedule variance (SV = EV - PV).
    pub schedule_variance: f64,
    /// Cost variance (CV = EV - AC).
    pub cost_variance: f64,
    /// Schedule performance index (SPI = EV / PV).
    pub spi: f64,
    /// Cost performance index (CPI = EV / AC).
    pub cpi: f64,
}

/// Retainage data for balance validation.
#[derive(Debug, Clone)]
pub struct RetainageData {
    /// Retainage identifier.
    pub retainage_id: String,
    /// Total amount held.
    pub total_held: f64,
    /// Amount released.
    pub released_amount: f64,
    /// Current balance held.
    pub balance_held: f64,
}

/// Results of project accounting coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAccountingEvaluation {
    /// Fraction of projects with correct completion_pct.
    pub poc_accuracy: f64,
    /// Fraction of projects with correct cumulative_revenue.
    pub revenue_accuracy: f64,
    /// Fraction of projects with correct unbilled_revenue.
    pub unbilled_accuracy: f64,
    /// Fraction of EVM records with correct SV, CV, SPI, CPI.
    pub evm_accuracy: f64,
    /// Fraction of retainage records with correct balance.
    pub retainage_accuracy: f64,
    /// Total projects evaluated.
    pub total_projects: usize,
    /// Total EVM records evaluated.
    pub total_evm_records: usize,
    /// Total retainage records evaluated.
    pub total_retainage: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for project accounting coherence.
pub struct ProjectAccountingEvaluator {
    thresholds: ProjectAccountingThresholds,
}

impl ProjectAccountingEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: ProjectAccountingThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: ProjectAccountingThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate project accounting data coherence.
    pub fn evaluate(
        &self,
        projects: &[ProjectRevenueData],
        evm_records: &[EarnedValueData],
        retainage: &[RetainageData],
    ) -> EvalResult<ProjectAccountingEvaluation> {
        let mut issues = Vec::new();
        let tolerance = self.thresholds.evm_tolerance;

        // 1. PoC: completion_pct ≈ costs_to_date / estimated_total_cost
        let poc_ok = projects
            .iter()
            .filter(|p| {
                if p.estimated_total_cost <= 0.0 {
                    return true;
                }
                let expected = p.costs_to_date / p.estimated_total_cost;
                (p.completion_pct - expected).abs() <= tolerance
            })
            .count();
        let poc_accuracy = if projects.is_empty() {
            1.0
        } else {
            poc_ok as f64 / projects.len() as f64
        };

        // 2. Revenue: cumulative_revenue ≈ contract_value * completion_pct
        let rev_ok = projects
            .iter()
            .filter(|p| {
                let expected = p.contract_value * p.completion_pct;
                (p.cumulative_revenue - expected).abs()
                    <= tolerance * p.contract_value.abs().max(1.0)
            })
            .count();
        let revenue_accuracy = if projects.is_empty() {
            1.0
        } else {
            rev_ok as f64 / projects.len() as f64
        };

        // 3. Unbilled: unbilled_revenue ≈ cumulative_revenue - billed_to_date
        let unbilled_ok = projects
            .iter()
            .filter(|p| {
                let expected = p.cumulative_revenue - p.billed_to_date;
                (p.unbilled_revenue - expected).abs()
                    <= tolerance * p.cumulative_revenue.abs().max(1.0)
            })
            .count();
        let unbilled_accuracy = if projects.is_empty() {
            1.0
        } else {
            unbilled_ok as f64 / projects.len() as f64
        };

        // 4. EVM: SV = EV - PV, CV = EV - AC, SPI = EV/PV, CPI = EV/AC
        let evm_ok = evm_records
            .iter()
            .filter(|e| {
                let sv_expected = e.earned_value - e.planned_value;
                let cv_expected = e.earned_value - e.actual_cost;
                let sv_ok = (e.schedule_variance - sv_expected).abs()
                    <= tolerance * e.earned_value.abs().max(1.0);
                let cv_ok = (e.cost_variance - cv_expected).abs()
                    <= tolerance * e.earned_value.abs().max(1.0);

                let spi_ok = if e.planned_value > 0.0 {
                    let expected = e.earned_value / e.planned_value;
                    (e.spi - expected).abs() <= tolerance
                } else {
                    true
                };
                let cpi_ok = if e.actual_cost > 0.0 {
                    let expected = e.earned_value / e.actual_cost;
                    (e.cpi - expected).abs() <= tolerance
                } else {
                    true
                };

                sv_ok && cv_ok && spi_ok && cpi_ok
            })
            .count();
        let evm_accuracy = if evm_records.is_empty() {
            1.0
        } else {
            evm_ok as f64 / evm_records.len() as f64
        };

        // 5. Retainage: balance_held ≈ total_held - released_amount
        let ret_ok = retainage
            .iter()
            .filter(|r| {
                let expected = r.total_held - r.released_amount;
                (r.balance_held - expected).abs() <= tolerance * r.total_held.abs().max(1.0)
            })
            .count();
        let retainage_accuracy = if retainage.is_empty() {
            1.0
        } else {
            ret_ok as f64 / retainage.len() as f64
        };

        // Check thresholds
        if poc_accuracy < self.thresholds.min_poc_accuracy {
            issues.push(format!(
                "PoC completion accuracy {:.4} < {:.4}",
                poc_accuracy, self.thresholds.min_poc_accuracy
            ));
        }
        if revenue_accuracy < self.thresholds.min_poc_accuracy {
            issues.push(format!(
                "Revenue recognition accuracy {:.4} < {:.4}",
                revenue_accuracy, self.thresholds.min_poc_accuracy
            ));
        }
        if unbilled_accuracy < self.thresholds.min_poc_accuracy {
            issues.push(format!(
                "Unbilled revenue accuracy {:.4} < {:.4}",
                unbilled_accuracy, self.thresholds.min_poc_accuracy
            ));
        }
        if evm_accuracy < self.thresholds.min_evm_accuracy {
            issues.push(format!(
                "EVM metric accuracy {:.4} < {:.4}",
                evm_accuracy, self.thresholds.min_evm_accuracy
            ));
        }
        if retainage_accuracy < self.thresholds.min_evm_accuracy {
            issues.push(format!(
                "Retainage balance accuracy {:.4} < {:.4}",
                retainage_accuracy, self.thresholds.min_evm_accuracy
            ));
        }

        let passes = issues.is_empty();

        Ok(ProjectAccountingEvaluation {
            poc_accuracy,
            revenue_accuracy,
            unbilled_accuracy,
            evm_accuracy,
            retainage_accuracy,
            total_projects: projects.len(),
            total_evm_records: evm_records.len(),
            total_retainage: retainage.len(),
            passes,
            issues,
        })
    }
}

impl Default for ProjectAccountingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_accounting() {
        let evaluator = ProjectAccountingEvaluator::new();
        let projects = vec![ProjectRevenueData {
            project_id: "PRJ001".to_string(),
            costs_to_date: 500_000.0,
            estimated_total_cost: 1_000_000.0,
            completion_pct: 0.50,
            contract_value: 1_200_000.0,
            cumulative_revenue: 600_000.0,
            billed_to_date: 550_000.0,
            unbilled_revenue: 50_000.0,
        }];
        let evm = vec![EarnedValueData {
            project_id: "PRJ001".to_string(),
            planned_value: 600_000.0,
            earned_value: 500_000.0,
            actual_cost: 520_000.0,
            bac: 1_000_000.0,
            schedule_variance: -100_000.0,
            cost_variance: -20_000.0,
            spi: 500_000.0 / 600_000.0,
            cpi: 500_000.0 / 520_000.0,
        }];
        let retainage = vec![RetainageData {
            retainage_id: "RET001".to_string(),
            total_held: 60_000.0,
            released_amount: 10_000.0,
            balance_held: 50_000.0,
        }];

        let result = evaluator.evaluate(&projects, &evm, &retainage).unwrap();
        assert!(result.passes);
        assert_eq!(result.total_projects, 1);
        assert_eq!(result.total_evm_records, 1);
    }

    #[test]
    fn test_wrong_completion_pct() {
        let evaluator = ProjectAccountingEvaluator::new();
        let projects = vec![ProjectRevenueData {
            project_id: "PRJ001".to_string(),
            costs_to_date: 500_000.0,
            estimated_total_cost: 1_000_000.0,
            completion_pct: 0.80, // Wrong: should be 0.50
            contract_value: 1_200_000.0,
            cumulative_revenue: 960_000.0,
            billed_to_date: 900_000.0,
            unbilled_revenue: 60_000.0,
        }];

        let result = evaluator.evaluate(&projects, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("PoC completion")));
    }

    #[test]
    fn test_wrong_evm_metrics() {
        let evaluator = ProjectAccountingEvaluator::new();
        let evm = vec![EarnedValueData {
            project_id: "PRJ001".to_string(),
            planned_value: 600_000.0,
            earned_value: 500_000.0,
            actual_cost: 520_000.0,
            bac: 1_000_000.0,
            schedule_variance: 0.0, // Wrong: should be -100,000
            cost_variance: -20_000.0,
            spi: 500_000.0 / 600_000.0,
            cpi: 500_000.0 / 520_000.0,
        }];

        let result = evaluator.evaluate(&[], &evm, &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("EVM metric")));
    }

    #[test]
    fn test_wrong_retainage_balance() {
        let evaluator = ProjectAccountingEvaluator::new();
        let retainage = vec![RetainageData {
            retainage_id: "RET001".to_string(),
            total_held: 60_000.0,
            released_amount: 10_000.0,
            balance_held: 60_000.0, // Wrong: should be 50,000
        }];

        let result = evaluator.evaluate(&[], &[], &retainage).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Retainage")));
    }

    #[test]
    fn test_wrong_cumulative_revenue() {
        let evaluator = ProjectAccountingEvaluator::new();
        let projects = vec![ProjectRevenueData {
            project_id: "PRJ001".to_string(),
            costs_to_date: 500_000.0,
            estimated_total_cost: 1_000_000.0,
            completion_pct: 0.50,
            contract_value: 1_200_000.0,
            cumulative_revenue: 900_000.0, // Wrong: should be 600,000
            billed_to_date: 550_000.0,
            unbilled_revenue: 350_000.0,
        }];

        let result = evaluator.evaluate(&projects, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Revenue recognition")));
    }

    #[test]
    fn test_wrong_unbilled_revenue() {
        let evaluator = ProjectAccountingEvaluator::new();
        let projects = vec![ProjectRevenueData {
            project_id: "PRJ001".to_string(),
            costs_to_date: 500_000.0,
            estimated_total_cost: 1_000_000.0,
            completion_pct: 0.50,
            contract_value: 1_200_000.0,
            cumulative_revenue: 600_000.0,
            billed_to_date: 550_000.0,
            unbilled_revenue: 200_000.0, // Wrong: should be 50,000
        }];

        let result = evaluator.evaluate(&projects, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Unbilled revenue")));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = ProjectAccountingEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[]).unwrap();
        assert!(result.passes);
    }
}
