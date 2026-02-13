//! Bank reconciliation evaluator.
//!
//! Validates bank reconciliation coherence including balance equations,
//! completed reconciliation status, match rates, and reconciling item completeness.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for bank reconciliation evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReconciliationThresholds {
    /// Minimum balance accuracy (fraction of reconciliations within tolerance).
    pub min_balance_accuracy: f64,
    /// Minimum statement line match rate.
    pub min_match_rate: f64,
    /// Tolerance for balance comparisons.
    pub balance_tolerance: f64,
}

impl Default for BankReconciliationThresholds {
    fn default() -> Self {
        Self {
            min_balance_accuracy: 0.99,
            min_match_rate: 0.85,
            balance_tolerance: 0.01,
        }
    }
}

/// Bank reconciliation data for validation.
#[derive(Debug, Clone)]
pub struct ReconciliationData {
    /// Reconciliation identifier.
    pub reconciliation_id: String,
    /// Bank ending balance.
    pub bank_ending_balance: f64,
    /// Book ending balance.
    pub book_ending_balance: f64,
    /// Sum of reconciling items (adjustments from bank to book).
    pub reconciling_items_sum: f64,
    /// Whether the reconciliation is marked as completed.
    pub is_completed: bool,
    /// Total statement lines in the period.
    pub total_statement_lines: usize,
    /// Matched statement lines.
    pub matched_statement_lines: usize,
    /// Number of reconciling items.
    pub reconciling_item_count: usize,
    /// Number of reconciling items with descriptions.
    pub items_with_descriptions: usize,
}

/// Results of bank reconciliation evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReconciliationEvaluation {
    /// Balance accuracy: fraction of reconciliations where bank_ending + reconciling_items = book_ending.
    pub balance_accuracy: f64,
    /// Fraction of completed reconciliations with zero net difference.
    pub completed_zero_difference_rate: f64,
    /// Average statement line match rate.
    pub match_rate: f64,
    /// Reconciling item completeness (fraction with descriptions).
    pub reconciling_item_completeness: f64,
    /// Total reconciliations evaluated.
    pub total_reconciliations: usize,
    /// Reconciliations with balance equation satisfied.
    pub balanced_count: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for bank reconciliation coherence.
pub struct BankReconciliationEvaluator {
    thresholds: BankReconciliationThresholds,
}

impl BankReconciliationEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: BankReconciliationThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: BankReconciliationThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate bank reconciliation data.
    pub fn evaluate(
        &self,
        reconciliations: &[ReconciliationData],
    ) -> EvalResult<BankReconciliationEvaluation> {
        let mut issues = Vec::new();
        let tol = self.thresholds.balance_tolerance;
        let total = reconciliations.len();

        // 1. Balance equation: bank_ending + reconciling_items ≈ book_ending
        let balanced_count = reconciliations
            .iter()
            .filter(|r| {
                let adjusted = r.bank_ending_balance + r.reconciling_items_sum;
                (adjusted - r.book_ending_balance).abs() <= tol
            })
            .count();
        let balance_accuracy = if total > 0 {
            balanced_count as f64 / total as f64
        } else {
            1.0
        };

        // 2. Completed reconciliations should have zero net difference
        let completed: Vec<&ReconciliationData> =
            reconciliations.iter().filter(|r| r.is_completed).collect();
        let completed_zero = completed
            .iter()
            .filter(|r| {
                let adjusted = r.bank_ending_balance + r.reconciling_items_sum;
                (adjusted - r.book_ending_balance).abs() <= tol
            })
            .count();
        let completed_zero_difference_rate = if completed.is_empty() {
            1.0
        } else {
            completed_zero as f64 / completed.len() as f64
        };

        // 3. Match rate
        let total_lines: usize = reconciliations
            .iter()
            .map(|r| r.total_statement_lines)
            .sum();
        let matched_lines: usize = reconciliations
            .iter()
            .map(|r| r.matched_statement_lines)
            .sum();
        let match_rate = if total_lines > 0 {
            matched_lines as f64 / total_lines as f64
        } else {
            1.0
        };

        // 4. Reconciling item completeness
        let total_items: usize = reconciliations
            .iter()
            .map(|r| r.reconciling_item_count)
            .sum();
        let items_with_desc: usize = reconciliations
            .iter()
            .map(|r| r.items_with_descriptions)
            .sum();
        let reconciling_item_completeness = if total_items > 0 {
            items_with_desc as f64 / total_items as f64
        } else {
            1.0
        };

        // Check thresholds
        if balance_accuracy < self.thresholds.min_balance_accuracy {
            issues.push(format!(
                "Balance accuracy {:.3} < {:.3}",
                balance_accuracy, self.thresholds.min_balance_accuracy
            ));
        }
        if match_rate < self.thresholds.min_match_rate {
            issues.push(format!(
                "Match rate {:.3} < {:.3}",
                match_rate, self.thresholds.min_match_rate
            ));
        }

        let passes = issues.is_empty();

        Ok(BankReconciliationEvaluation {
            balance_accuracy,
            completed_zero_difference_rate,
            match_rate,
            reconciling_item_completeness,
            total_reconciliations: total,
            balanced_count,
            passes,
            issues,
        })
    }
}

impl Default for BankReconciliationEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_reconciliation() {
        let evaluator = BankReconciliationEvaluator::new();
        let data = vec![ReconciliationData {
            reconciliation_id: "BR001".to_string(),
            bank_ending_balance: 100_000.0,
            book_ending_balance: 100_500.0,
            reconciling_items_sum: 500.0,
            is_completed: true,
            total_statement_lines: 50,
            matched_statement_lines: 48,
            reconciling_item_count: 5,
            items_with_descriptions: 5,
        }];

        let result = evaluator.evaluate(&data).unwrap();
        assert!(result.passes);
        assert_eq!(result.balance_accuracy, 1.0);
        assert_eq!(result.completed_zero_difference_rate, 1.0);
    }

    #[test]
    fn test_imbalanced_reconciliation() {
        let evaluator = BankReconciliationEvaluator::new();
        let data = vec![ReconciliationData {
            reconciliation_id: "BR001".to_string(),
            bank_ending_balance: 100_000.0,
            book_ending_balance: 110_000.0,
            reconciling_items_sum: 500.0, // Doesn't bridge the gap
            is_completed: true,
            total_statement_lines: 50,
            matched_statement_lines: 48,
            reconciling_item_count: 5,
            items_with_descriptions: 5,
        }];

        let result = evaluator.evaluate(&data).unwrap();
        assert!(!result.passes);
        assert_eq!(result.balance_accuracy, 0.0);
    }

    #[test]
    fn test_low_match_rate() {
        let evaluator = BankReconciliationEvaluator::new();
        let data = vec![ReconciliationData {
            reconciliation_id: "BR001".to_string(),
            bank_ending_balance: 100_000.0,
            book_ending_balance: 100_000.0,
            reconciling_items_sum: 0.0,
            is_completed: true,
            total_statement_lines: 100,
            matched_statement_lines: 50, // Only 50% matched
            reconciling_item_count: 0,
            items_with_descriptions: 0,
        }];

        let result = evaluator.evaluate(&data).unwrap();
        assert!(!result.passes);
        assert_eq!(result.match_rate, 0.5);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = BankReconciliationEvaluator::new();
        let result = evaluator.evaluate(&[]).unwrap();
        assert!(result.passes);
    }
}
