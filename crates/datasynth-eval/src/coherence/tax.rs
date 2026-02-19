//! Tax coherence evaluator.
//!
//! Validates tax calculation accuracy, VAT/GST return coherence,
//! and withholding tax compliance including treaty rate validation.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for tax evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxThresholds {
    /// Minimum accuracy for tax_amount = taxable_amount * rate.
    pub min_tax_calculation_accuracy: f64,
    /// Tolerance for tax amount comparisons.
    pub rate_tolerance: f64,
    /// Minimum accuracy for return net_payable = output - input.
    pub min_return_accuracy: f64,
    /// Minimum accuracy for withheld_amount = base * applied_rate.
    pub min_withholding_accuracy: f64,
    /// Minimum rate of treaty records where applied_rate <= statutory_rate.
    pub min_treaty_compliance_rate: f64,
}

impl Default for TaxThresholds {
    fn default() -> Self {
        Self {
            min_tax_calculation_accuracy: 0.999,
            rate_tolerance: 0.001,
            min_return_accuracy: 0.95,
            min_withholding_accuracy: 0.999,
            min_treaty_compliance_rate: 0.95,
        }
    }
}

/// Tax line data for calculation validation.
#[derive(Debug, Clone)]
pub struct TaxLineData {
    /// Tax code identifier.
    pub tax_code_id: String,
    /// Taxable amount (base).
    pub taxable_amount: f64,
    /// Computed tax amount.
    pub tax_amount: f64,
    /// Tax rate applied.
    pub rate: f64,
}

/// Tax return data for net payable validation.
#[derive(Debug, Clone)]
pub struct TaxReturnData {
    /// Return identifier.
    pub return_id: String,
    /// Total output tax (collected).
    pub total_output_tax: f64,
    /// Total input tax (paid).
    pub total_input_tax: f64,
    /// Net payable (output - input).
    pub net_payable: f64,
}

/// Withholding tax data for treaty validation.
#[derive(Debug, Clone)]
pub struct WithholdingData {
    /// Record identifier.
    pub record_id: String,
    /// Base amount subject to withholding.
    pub base_amount: f64,
    /// Applied withholding rate.
    pub applied_rate: f64,
    /// Statutory withholding rate.
    pub statutory_rate: f64,
    /// Actual withheld amount.
    pub withheld_amount: f64,
    /// Whether a treaty rate was applied.
    pub has_treaty: bool,
}

/// Results of tax coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxEvaluation {
    /// Fraction of tax lines where tax_amount ≈ taxable_amount * rate.
    pub tax_calculation_accuracy: f64,
    /// Fraction of returns where net_payable ≈ output - input.
    pub return_net_accuracy: f64,
    /// Fraction of withholding records where withheld ≈ base * rate.
    pub withholding_accuracy: f64,
    /// Fraction of treaty records where applied_rate <= statutory_rate.
    pub treaty_compliance_rate: f64,
    /// Total tax lines evaluated.
    pub total_tax_lines: usize,
    /// Total returns evaluated.
    pub total_returns: usize,
    /// Total withholding records evaluated.
    pub total_withholding: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for tax calculation coherence.
pub struct TaxEvaluator {
    thresholds: TaxThresholds,
}

impl TaxEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: TaxThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: TaxThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate tax data coherence.
    pub fn evaluate(
        &self,
        tax_lines: &[TaxLineData],
        returns: &[TaxReturnData],
        withholding: &[WithholdingData],
    ) -> EvalResult<TaxEvaluation> {
        let mut issues = Vec::new();
        let tolerance = self.thresholds.rate_tolerance;

        // 1. Tax calculation accuracy: tax_amount ≈ taxable_amount * rate
        let tax_ok = tax_lines
            .iter()
            .filter(|t| {
                let expected = t.taxable_amount * t.rate;
                (t.tax_amount - expected).abs() <= tolerance * t.taxable_amount.abs().max(1.0)
            })
            .count();
        let tax_calculation_accuracy = if tax_lines.is_empty() {
            1.0
        } else {
            tax_ok as f64 / tax_lines.len() as f64
        };

        // 2. Return net payable: net_payable ≈ output - input
        let return_ok = returns
            .iter()
            .filter(|r| {
                let expected = r.total_output_tax - r.total_input_tax;
                (r.net_payable - expected).abs() <= tolerance * r.total_output_tax.abs().max(1.0)
            })
            .count();
        let return_net_accuracy = if returns.is_empty() {
            1.0
        } else {
            return_ok as f64 / returns.len() as f64
        };

        // 3. Withholding accuracy: withheld ≈ base * applied_rate
        let wh_ok = withholding
            .iter()
            .filter(|w| {
                let expected = w.base_amount * w.applied_rate;
                (w.withheld_amount - expected).abs() <= tolerance * w.base_amount.abs().max(1.0)
            })
            .count();
        let withholding_accuracy = if withholding.is_empty() {
            1.0
        } else {
            wh_ok as f64 / withholding.len() as f64
        };

        // 4. Treaty compliance: has_treaty implies applied_rate <= statutory_rate
        let treaty_records: Vec<_> = withholding.iter().filter(|w| w.has_treaty).collect();
        let treaty_ok = treaty_records
            .iter()
            .filter(|w| w.applied_rate <= w.statutory_rate + tolerance)
            .count();
        let treaty_compliance_rate = if treaty_records.is_empty() {
            1.0
        } else {
            treaty_ok as f64 / treaty_records.len() as f64
        };

        // Check thresholds
        if tax_calculation_accuracy < self.thresholds.min_tax_calculation_accuracy {
            issues.push(format!(
                "Tax calculation accuracy {:.4} < {:.4}",
                tax_calculation_accuracy, self.thresholds.min_tax_calculation_accuracy
            ));
        }
        if return_net_accuracy < self.thresholds.min_return_accuracy {
            issues.push(format!(
                "Return net payable accuracy {:.4} < {:.4}",
                return_net_accuracy, self.thresholds.min_return_accuracy
            ));
        }
        if withholding_accuracy < self.thresholds.min_withholding_accuracy {
            issues.push(format!(
                "Withholding accuracy {:.4} < {:.4}",
                withholding_accuracy, self.thresholds.min_withholding_accuracy
            ));
        }
        if treaty_compliance_rate < self.thresholds.min_treaty_compliance_rate {
            issues.push(format!(
                "Treaty compliance rate {:.4} < {:.4}",
                treaty_compliance_rate, self.thresholds.min_treaty_compliance_rate
            ));
        }

        let passes = issues.is_empty();

        Ok(TaxEvaluation {
            tax_calculation_accuracy,
            return_net_accuracy,
            withholding_accuracy,
            treaty_compliance_rate,
            total_tax_lines: tax_lines.len(),
            total_returns: returns.len(),
            total_withholding: withholding.len(),
            passes,
            issues,
        })
    }
}

impl Default for TaxEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tax_calculations() {
        let evaluator = TaxEvaluator::new();
        let lines = vec![
            TaxLineData {
                tax_code_id: "VAT20".to_string(),
                taxable_amount: 1000.0,
                tax_amount: 200.0,
                rate: 0.20,
            },
            TaxLineData {
                tax_code_id: "VAT10".to_string(),
                taxable_amount: 500.0,
                tax_amount: 50.0,
                rate: 0.10,
            },
        ];
        let returns = vec![TaxReturnData {
            return_id: "RET001".to_string(),
            total_output_tax: 250.0,
            total_input_tax: 100.0,
            net_payable: 150.0,
        }];
        let withholding = vec![WithholdingData {
            record_id: "WH001".to_string(),
            base_amount: 10000.0,
            applied_rate: 0.10,
            statutory_rate: 0.15,
            withheld_amount: 1000.0,
            has_treaty: true,
        }];

        let result = evaluator.evaluate(&lines, &returns, &withholding).unwrap();
        assert!(result.passes);
        assert_eq!(result.total_tax_lines, 2);
        assert_eq!(result.total_returns, 1);
        assert_eq!(result.treaty_compliance_rate, 1.0);
    }

    #[test]
    fn test_wrong_tax_amount() {
        let evaluator = TaxEvaluator::new();
        let lines = vec![TaxLineData {
            tax_code_id: "VAT20".to_string(),
            taxable_amount: 1000.0,
            tax_amount: 300.0, // Wrong: should be 200.0
            rate: 0.20,
        }];

        let result = evaluator.evaluate(&lines, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Tax calculation accuracy"));
    }

    #[test]
    fn test_wrong_net_payable() {
        let evaluator = TaxEvaluator::new();
        let returns = vec![TaxReturnData {
            return_id: "RET001".to_string(),
            total_output_tax: 250.0,
            total_input_tax: 100.0,
            net_payable: 200.0, // Wrong: should be 150.0
        }];

        let result = evaluator.evaluate(&[], &returns, &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Return net payable"));
    }

    #[test]
    fn test_treaty_violation() {
        let evaluator = TaxEvaluator::new();
        let withholding = vec![WithholdingData {
            record_id: "WH001".to_string(),
            base_amount: 10000.0,
            applied_rate: 0.20, // Higher than statutory
            statutory_rate: 0.15,
            withheld_amount: 2000.0,
            has_treaty: true,
        }];

        let result = evaluator.evaluate(&[], &[], &withholding).unwrap();
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Treaty compliance")));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = TaxEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[]).unwrap();
        assert!(result.passes);
    }
}
