//! Cross-process link evaluator.
//!
//! Validates cross-process linkage including P2P-O2C via inventory,
//! payment-bank reconciliation links, intercompany bilateral tracing,
//! and overall lineage completeness.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for cross-process link evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProcessThresholds {
    /// Minimum link rate for any cross-process category.
    pub min_link_rate: f64,
}

impl Default for CrossProcessThresholds {
    fn default() -> Self {
        Self {
            min_link_rate: 0.80,
        }
    }
}

/// Cross-process link data.
#[derive(Debug, Clone)]
pub struct CrossProcessLinkData {
    /// Inventory P2P↔O2C links: total GoodsReceipt→Delivery candidates.
    pub inventory_total: usize,
    /// Inventory P2P↔O2C links: successfully linked.
    pub inventory_linked: usize,
    /// Payment↔BankReconciliation: total payment candidates.
    pub payment_total: usize,
    /// Payment↔BankReconciliation: successfully linked.
    pub payment_linked: usize,
    /// IC bilateral: total IC transaction pairs.
    pub ic_bilateral_total: usize,
    /// IC bilateral: pairs traced end-to-end.
    pub ic_bilateral_traced: usize,
    /// Total lineage entities.
    pub lineage_total: usize,
    /// Entities with complete lineage.
    pub lineage_complete: usize,
}

/// Results of cross-process link evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProcessEvaluation {
    /// Inventory P2P↔O2C link rate.
    pub inventory_p2p_o2c_link_rate: f64,
    /// Payment↔BankReconciliation link rate.
    pub payment_bank_link_rate: f64,
    /// IC bilateral trace rate.
    pub ic_bilateral_trace_rate: f64,
    /// Overall lineage completeness.
    pub overall_lineage_completeness: f64,
    /// Combined cross-process score (average of all link rates).
    pub combined_score: f64,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for cross-process links.
pub struct CrossProcessEvaluator {
    thresholds: CrossProcessThresholds,
}

impl CrossProcessEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: CrossProcessThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: CrossProcessThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate cross-process link data.
    pub fn evaluate(&self, data: &CrossProcessLinkData) -> EvalResult<CrossProcessEvaluation> {
        let mut issues = Vec::new();

        let inventory_rate = if data.inventory_total > 0 {
            data.inventory_linked as f64 / data.inventory_total as f64
        } else {
            1.0
        };

        let payment_rate = if data.payment_total > 0 {
            data.payment_linked as f64 / data.payment_total as f64
        } else {
            1.0
        };

        let ic_rate = if data.ic_bilateral_total > 0 {
            data.ic_bilateral_traced as f64 / data.ic_bilateral_total as f64
        } else {
            1.0
        };

        let lineage_rate = if data.lineage_total > 0 {
            data.lineage_complete as f64 / data.lineage_total as f64
        } else {
            1.0
        };

        // Compute combined score from available rates
        let mut rates = Vec::new();
        if data.inventory_total > 0 {
            rates.push(inventory_rate);
        }
        if data.payment_total > 0 {
            rates.push(payment_rate);
        }
        if data.ic_bilateral_total > 0 {
            rates.push(ic_rate);
        }
        if data.lineage_total > 0 {
            rates.push(lineage_rate);
        }
        let combined_score = if rates.is_empty() {
            1.0
        } else {
            rates.iter().sum::<f64>() / rates.len() as f64
        };

        let min_rate = self.thresholds.min_link_rate;
        if data.inventory_total > 0 && inventory_rate < min_rate {
            issues.push(format!(
                "Inventory P2P↔O2C link rate {inventory_rate:.3} < {min_rate:.3}"
            ));
        }
        if data.payment_total > 0 && payment_rate < min_rate {
            issues.push(format!(
                "Payment↔Bank link rate {payment_rate:.3} < {min_rate:.3}"
            ));
        }
        if data.ic_bilateral_total > 0 && ic_rate < min_rate {
            issues.push(format!(
                "IC bilateral trace rate {ic_rate:.3} < {min_rate:.3}"
            ));
        }

        let passes = issues.is_empty();

        Ok(CrossProcessEvaluation {
            inventory_p2p_o2c_link_rate: inventory_rate,
            payment_bank_link_rate: payment_rate,
            ic_bilateral_trace_rate: ic_rate,
            overall_lineage_completeness: lineage_rate,
            combined_score,
            passes,
            issues,
        })
    }
}

impl Default for CrossProcessEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_fully_linked() {
        let evaluator = CrossProcessEvaluator::new();
        let data = CrossProcessLinkData {
            inventory_total: 100,
            inventory_linked: 95,
            payment_total: 200,
            payment_linked: 190,
            ic_bilateral_total: 50,
            ic_bilateral_traced: 48,
            lineage_total: 300,
            lineage_complete: 280,
        };

        let result = evaluator.evaluate(&data).unwrap();
        assert!(result.passes);
        assert!(result.combined_score > 0.9);
    }

    #[test]
    fn test_low_link_rates() {
        let evaluator = CrossProcessEvaluator::new();
        let data = CrossProcessLinkData {
            inventory_total: 100,
            inventory_linked: 50,
            payment_total: 100,
            payment_linked: 40,
            ic_bilateral_total: 0,
            ic_bilateral_traced: 0,
            lineage_total: 0,
            lineage_complete: 0,
        };

        let result = evaluator.evaluate(&data).unwrap();
        assert!(!result.passes);
        assert_eq!(result.inventory_p2p_o2c_link_rate, 0.5);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = CrossProcessEvaluator::new();
        let data = CrossProcessLinkData {
            inventory_total: 0,
            inventory_linked: 0,
            payment_total: 0,
            payment_linked: 0,
            ic_bilateral_total: 0,
            ic_bilateral_traced: 0,
            lineage_total: 0,
            lineage_complete: 0,
        };

        let result = evaluator.evaluate(&data).unwrap();
        assert!(result.passes);
        assert_eq!(result.combined_score, 1.0);
    }
}
