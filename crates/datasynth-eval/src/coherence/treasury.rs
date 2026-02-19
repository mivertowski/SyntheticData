//! Treasury coherence evaluator.
//!
//! Validates cash position balance equations, hedge effectiveness ranges,
//! covenant compliance logic, and intercompany netting calculations.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for treasury evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryThresholds {
    /// Minimum accuracy for closing = opening + inflows - outflows.
    pub min_balance_accuracy: f64,
    /// Tolerance for balance comparisons.
    pub balance_tolerance: f64,
    /// Minimum rate of hedges with correct effectiveness classification.
    pub min_hedge_effectiveness_rate: f64,
    /// Minimum rate of covenants with correct compliance classification.
    pub min_covenant_compliance_rate: f64,
    /// Minimum accuracy for netting settlement calculations.
    pub min_netting_accuracy: f64,
}

impl Default for TreasuryThresholds {
    fn default() -> Self {
        Self {
            min_balance_accuracy: 0.999,
            balance_tolerance: 0.01,
            min_hedge_effectiveness_rate: 0.95,
            min_covenant_compliance_rate: 0.95,
            min_netting_accuracy: 0.999,
        }
    }
}

/// Cash position data for balance validation.
#[derive(Debug, Clone)]
pub struct CashPositionData {
    /// Position identifier.
    pub position_id: String,
    /// Opening balance.
    pub opening_balance: f64,
    /// Total inflows.
    pub inflows: f64,
    /// Total outflows.
    pub outflows: f64,
    /// Closing balance.
    pub closing_balance: f64,
}

/// Hedge effectiveness data for range validation.
#[derive(Debug, Clone)]
pub struct HedgeEffectivenessData {
    /// Hedge identifier.
    pub hedge_id: String,
    /// Effectiveness ratio (should be 0.80-1.25 for effective hedges).
    pub effectiveness_ratio: f64,
    /// Whether classified as effective.
    pub is_effective: bool,
}

/// Covenant data for compliance validation.
#[derive(Debug, Clone)]
pub struct CovenantData {
    /// Covenant identifier.
    pub covenant_id: String,
    /// Covenant threshold value.
    pub threshold: f64,
    /// Actual measured value.
    pub actual_value: f64,
    /// Whether classified as compliant.
    pub is_compliant: bool,
    /// Whether this is a maximum covenant (actual must be <= threshold).
    /// If false, it's a minimum covenant (actual must be >= threshold).
    pub is_max_covenant: bool,
}

/// Netting data for settlement validation.
#[derive(Debug, Clone)]
pub struct NettingData {
    /// Netting run identifier.
    pub run_id: String,
    /// Gross receivables.
    pub gross_receivables: f64,
    /// Gross payables.
    pub gross_payables: f64,
    /// Net settlement amount.
    pub net_settlement: f64,
}

/// Results of treasury coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryEvaluation {
    /// Fraction of positions where closing ≈ opening + inflows - outflows.
    pub balance_accuracy: f64,
    /// Fraction of hedges with correct effectiveness classification.
    pub hedge_effectiveness_accuracy: f64,
    /// Fraction of covenants with correct compliance classification.
    pub covenant_compliance_accuracy: f64,
    /// Fraction of netting runs where net ≈ |receivables - payables|.
    pub netting_accuracy: f64,
    /// Total cash positions evaluated.
    pub total_positions: usize,
    /// Total hedges evaluated.
    pub total_hedges: usize,
    /// Total covenants evaluated.
    pub total_covenants: usize,
    /// Total netting runs evaluated.
    pub total_netting_runs: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for treasury coherence.
pub struct TreasuryEvaluator {
    thresholds: TreasuryThresholds,
}

impl TreasuryEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: TreasuryThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: TreasuryThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate treasury data coherence.
    pub fn evaluate(
        &self,
        positions: &[CashPositionData],
        hedges: &[HedgeEffectivenessData],
        covenants: &[CovenantData],
        netting_runs: &[NettingData],
    ) -> EvalResult<TreasuryEvaluation> {
        let mut issues = Vec::new();
        let tolerance = self.thresholds.balance_tolerance;

        // 1. Cash position balance: closing ≈ opening + inflows - outflows
        let balance_ok = positions
            .iter()
            .filter(|p| {
                let expected = p.opening_balance + p.inflows - p.outflows;
                (p.closing_balance - expected).abs() <= tolerance * p.opening_balance.abs().max(1.0)
            })
            .count();
        let balance_accuracy = if positions.is_empty() {
            1.0
        } else {
            balance_ok as f64 / positions.len() as f64
        };

        // 2. Hedge effectiveness: is_effective iff ratio in [0.80, 1.25]
        let hedge_ok = hedges
            .iter()
            .filter(|h| {
                let in_range = h.effectiveness_ratio >= 0.80 && h.effectiveness_ratio <= 1.25;
                h.is_effective == in_range
            })
            .count();
        let hedge_effectiveness_accuracy = if hedges.is_empty() {
            1.0
        } else {
            hedge_ok as f64 / hedges.len() as f64
        };

        // 3. Covenant compliance: is_compliant iff actual meets threshold
        let covenant_ok = covenants
            .iter()
            .filter(|c| {
                let should_comply = if c.is_max_covenant {
                    c.actual_value <= c.threshold
                } else {
                    c.actual_value >= c.threshold
                };
                c.is_compliant == should_comply
            })
            .count();
        let covenant_compliance_accuracy = if covenants.is_empty() {
            1.0
        } else {
            covenant_ok as f64 / covenants.len() as f64
        };

        // 4. Netting: net_settlement ≈ |gross_receivables - gross_payables|
        let netting_ok = netting_runs
            .iter()
            .filter(|n| {
                let expected = (n.gross_receivables - n.gross_payables).abs();
                (n.net_settlement - expected).abs()
                    <= tolerance * n.gross_receivables.abs().max(1.0)
            })
            .count();
        let netting_accuracy = if netting_runs.is_empty() {
            1.0
        } else {
            netting_ok as f64 / netting_runs.len() as f64
        };

        // Check thresholds
        if balance_accuracy < self.thresholds.min_balance_accuracy {
            issues.push(format!(
                "Cash position balance accuracy {:.4} < {:.4}",
                balance_accuracy, self.thresholds.min_balance_accuracy
            ));
        }
        if hedge_effectiveness_accuracy < self.thresholds.min_hedge_effectiveness_rate {
            issues.push(format!(
                "Hedge effectiveness accuracy {:.4} < {:.4}",
                hedge_effectiveness_accuracy, self.thresholds.min_hedge_effectiveness_rate
            ));
        }
        if covenant_compliance_accuracy < self.thresholds.min_covenant_compliance_rate {
            issues.push(format!(
                "Covenant compliance accuracy {:.4} < {:.4}",
                covenant_compliance_accuracy, self.thresholds.min_covenant_compliance_rate
            ));
        }
        if netting_accuracy < self.thresholds.min_netting_accuracy {
            issues.push(format!(
                "Netting accuracy {:.4} < {:.4}",
                netting_accuracy, self.thresholds.min_netting_accuracy
            ));
        }

        let passes = issues.is_empty();

        Ok(TreasuryEvaluation {
            balance_accuracy,
            hedge_effectiveness_accuracy,
            covenant_compliance_accuracy,
            netting_accuracy,
            total_positions: positions.len(),
            total_hedges: hedges.len(),
            total_covenants: covenants.len(),
            total_netting_runs: netting_runs.len(),
            passes,
            issues,
        })
    }
}

impl Default for TreasuryEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_treasury_data() {
        let evaluator = TreasuryEvaluator::new();
        let positions = vec![CashPositionData {
            position_id: "CP001".to_string(),
            opening_balance: 100_000.0,
            inflows: 50_000.0,
            outflows: 30_000.0,
            closing_balance: 120_000.0,
        }];
        let hedges = vec![
            HedgeEffectivenessData {
                hedge_id: "H001".to_string(),
                effectiveness_ratio: 0.95,
                is_effective: true,
            },
            HedgeEffectivenessData {
                hedge_id: "H002".to_string(),
                effectiveness_ratio: 0.70,
                is_effective: false,
            },
        ];
        let covenants = vec![CovenantData {
            covenant_id: "COV001".to_string(),
            threshold: 3.0,
            actual_value: 2.5,
            is_compliant: true,
            is_max_covenant: true,
        }];
        let netting = vec![NettingData {
            run_id: "NET001".to_string(),
            gross_receivables: 50_000.0,
            gross_payables: 30_000.0,
            net_settlement: 20_000.0,
        }];

        let result = evaluator
            .evaluate(&positions, &hedges, &covenants, &netting)
            .unwrap();
        assert!(result.passes);
        assert_eq!(result.total_positions, 1);
        assert_eq!(result.total_hedges, 2);
    }

    #[test]
    fn test_wrong_closing_balance() {
        let evaluator = TreasuryEvaluator::new();
        let positions = vec![CashPositionData {
            position_id: "CP001".to_string(),
            opening_balance: 100_000.0,
            inflows: 50_000.0,
            outflows: 30_000.0,
            closing_balance: 200_000.0, // Wrong: should be 120,000
        }];

        let result = evaluator.evaluate(&positions, &[], &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Cash position balance"));
    }

    #[test]
    fn test_wrong_hedge_classification() {
        let evaluator = TreasuryEvaluator::new();
        let hedges = vec![HedgeEffectivenessData {
            hedge_id: "H001".to_string(),
            effectiveness_ratio: 0.70, // Out of range
            is_effective: true,        // Wrong: should be false
        }];

        let result = evaluator.evaluate(&[], &hedges, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Hedge effectiveness"));
    }

    #[test]
    fn test_wrong_covenant_compliance() {
        let evaluator = TreasuryEvaluator::new();
        let covenants = vec![CovenantData {
            covenant_id: "COV001".to_string(),
            threshold: 3.0,
            actual_value: 4.0,  // Exceeds max covenant
            is_compliant: true, // Wrong: should be false
            is_max_covenant: true,
        }];

        let result = evaluator.evaluate(&[], &[], &covenants, &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Covenant compliance"));
    }

    #[test]
    fn test_wrong_netting() {
        let evaluator = TreasuryEvaluator::new();
        let netting = vec![NettingData {
            run_id: "NET001".to_string(),
            gross_receivables: 50_000.0,
            gross_payables: 30_000.0,
            net_settlement: 5_000.0, // Wrong: should be 20,000
        }];

        let result = evaluator.evaluate(&[], &[], &[], &netting).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Netting accuracy"));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = TreasuryEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[], &[]).unwrap();
        assert!(result.passes);
    }
}
