//! Sales quote coherence evaluator.
//!
//! Validates line amount arithmetic, total consistency,
//! and status-dependent field requirements (Won → sales_order_id, Lost → lost_reason).

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for sales quote evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuoteThresholds {
    /// Minimum accuracy for line_amount = qty * unit_price.
    pub min_line_amount_accuracy: f64,
    /// Minimum accuracy for total = sum(line_amounts).
    pub min_total_accuracy: f64,
    /// Minimum status consistency (Won has order, Lost has reason).
    pub min_status_consistency: f64,
    /// Tolerance for amount comparisons.
    pub tolerance: f64,
}

impl Default for SalesQuoteThresholds {
    fn default() -> Self {
        Self {
            min_line_amount_accuracy: 0.999,
            min_total_accuracy: 0.999,
            min_status_consistency: 0.95,
            tolerance: 0.001,
        }
    }
}

/// Quote line item data.
#[derive(Debug, Clone)]
pub struct QuoteLineData {
    /// Line item number.
    pub item_number: u32,
    /// Quantity.
    pub quantity: f64,
    /// Unit price.
    pub unit_price: f64,
    /// Line amount (should be qty * unit_price).
    pub line_amount: f64,
}

/// Sales quote data for validation.
#[derive(Debug, Clone)]
pub struct SalesQuoteData {
    /// Quote identifier.
    pub quote_id: String,
    /// Quote status (e.g., "Draft", "Won", "Lost", "Expired").
    pub status: String,
    /// Line items.
    pub line_items: Vec<QuoteLineData>,
    /// Total quote amount.
    pub total_amount: f64,
    /// Whether a sales order reference exists.
    pub has_sales_order_id: bool,
    /// Whether a lost reason is provided.
    pub has_lost_reason: bool,
}

/// Results of sales quote coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuoteEvaluation {
    /// Fraction of line items with correct line_amount.
    pub line_amount_accuracy: f64,
    /// Fraction of quotes with correct total_amount.
    pub total_accuracy: f64,
    /// Fraction of Won/Lost quotes with required fields.
    pub status_consistency: f64,
    /// Total quotes evaluated.
    pub total_quotes: usize,
    /// Total line items evaluated.
    pub total_line_items: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for sales quote coherence.
pub struct SalesQuoteEvaluator {
    thresholds: SalesQuoteThresholds,
}

impl SalesQuoteEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: SalesQuoteThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: SalesQuoteThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate sales quote data coherence.
    pub fn evaluate(&self, quotes: &[SalesQuoteData]) -> EvalResult<SalesQuoteEvaluation> {
        let mut issues = Vec::new();
        let tolerance = self.thresholds.tolerance;

        // 1. Line amount accuracy: line_amount ≈ quantity * unit_price
        let all_lines: Vec<&QuoteLineData> =
            quotes.iter().flat_map(|q| q.line_items.iter()).collect();
        let line_ok = all_lines
            .iter()
            .filter(|l| {
                let expected = l.quantity * l.unit_price;
                (l.line_amount - expected).abs() <= tolerance * expected.abs().max(1.0)
            })
            .count();
        let line_amount_accuracy = if all_lines.is_empty() {
            1.0
        } else {
            line_ok as f64 / all_lines.len() as f64
        };

        // 2. Total accuracy: total_amount ≈ sum(line_amounts)
        let total_ok = quotes
            .iter()
            .filter(|q| {
                if q.line_items.is_empty() {
                    return true;
                }
                let sum: f64 = q.line_items.iter().map(|l| l.line_amount).sum();
                (q.total_amount - sum).abs() <= tolerance * sum.abs().max(1.0)
            })
            .count();
        let total_accuracy = if quotes.is_empty() {
            1.0
        } else {
            total_ok as f64 / quotes.len() as f64
        };

        // 3. Status consistency: Won → has_sales_order_id, Lost → has_lost_reason
        let status_relevant: Vec<_> = quotes
            .iter()
            .filter(|q| q.status == "Won" || q.status == "Lost")
            .collect();
        let status_ok = status_relevant
            .iter()
            .filter(|q| {
                if q.status == "Won" {
                    q.has_sales_order_id
                } else {
                    q.has_lost_reason
                }
            })
            .count();
        let status_consistency = if status_relevant.is_empty() {
            1.0
        } else {
            status_ok as f64 / status_relevant.len() as f64
        };

        // Check thresholds
        if line_amount_accuracy < self.thresholds.min_line_amount_accuracy {
            issues.push(format!(
                "Line amount accuracy {:.4} < {:.4}",
                line_amount_accuracy, self.thresholds.min_line_amount_accuracy
            ));
        }
        if total_accuracy < self.thresholds.min_total_accuracy {
            issues.push(format!(
                "Quote total accuracy {:.4} < {:.4}",
                total_accuracy, self.thresholds.min_total_accuracy
            ));
        }
        if status_consistency < self.thresholds.min_status_consistency {
            issues.push(format!(
                "Status consistency {:.4} < {:.4}",
                status_consistency, self.thresholds.min_status_consistency
            ));
        }

        let passes = issues.is_empty();

        Ok(SalesQuoteEvaluation {
            line_amount_accuracy,
            total_accuracy,
            status_consistency,
            total_quotes: quotes.len(),
            total_line_items: all_lines.len(),
            passes,
            issues,
        })
    }
}

impl Default for SalesQuoteEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sales_quotes() {
        let evaluator = SalesQuoteEvaluator::new();
        let quotes = vec![
            SalesQuoteData {
                quote_id: "SQ001".to_string(),
                status: "Won".to_string(),
                line_items: vec![
                    QuoteLineData {
                        item_number: 1,
                        quantity: 10.0,
                        unit_price: 100.0,
                        line_amount: 1000.0,
                    },
                    QuoteLineData {
                        item_number: 2,
                        quantity: 5.0,
                        unit_price: 200.0,
                        line_amount: 1000.0,
                    },
                ],
                total_amount: 2000.0,
                has_sales_order_id: true,
                has_lost_reason: false,
            },
            SalesQuoteData {
                quote_id: "SQ002".to_string(),
                status: "Lost".to_string(),
                line_items: vec![QuoteLineData {
                    item_number: 1,
                    quantity: 3.0,
                    unit_price: 500.0,
                    line_amount: 1500.0,
                }],
                total_amount: 1500.0,
                has_sales_order_id: false,
                has_lost_reason: true,
            },
        ];

        let result = evaluator.evaluate(&quotes).unwrap();
        assert!(result.passes);
        assert_eq!(result.total_quotes, 2);
        assert_eq!(result.total_line_items, 3);
    }

    #[test]
    fn test_wrong_line_amount() {
        let evaluator = SalesQuoteEvaluator::new();
        let quotes = vec![SalesQuoteData {
            quote_id: "SQ001".to_string(),
            status: "Draft".to_string(),
            line_items: vec![QuoteLineData {
                item_number: 1,
                quantity: 10.0,
                unit_price: 100.0,
                line_amount: 500.0, // Wrong: should be 1000
            }],
            total_amount: 500.0,
            has_sales_order_id: false,
            has_lost_reason: false,
        }];

        let result = evaluator.evaluate(&quotes).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Line amount")));
    }

    #[test]
    fn test_wrong_total() {
        let evaluator = SalesQuoteEvaluator::new();
        let quotes = vec![SalesQuoteData {
            quote_id: "SQ001".to_string(),
            status: "Draft".to_string(),
            line_items: vec![
                QuoteLineData {
                    item_number: 1,
                    quantity: 10.0,
                    unit_price: 100.0,
                    line_amount: 1000.0,
                },
                QuoteLineData {
                    item_number: 2,
                    quantity: 5.0,
                    unit_price: 200.0,
                    line_amount: 1000.0,
                },
            ],
            total_amount: 3000.0, // Wrong: should be 2000
            has_sales_order_id: false,
            has_lost_reason: false,
        }];

        let result = evaluator.evaluate(&quotes).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Quote total")));
    }

    #[test]
    fn test_won_without_order() {
        let evaluator = SalesQuoteEvaluator::new();
        let quotes = vec![SalesQuoteData {
            quote_id: "SQ001".to_string(),
            status: "Won".to_string(),
            line_items: vec![QuoteLineData {
                item_number: 1,
                quantity: 1.0,
                unit_price: 100.0,
                line_amount: 100.0,
            }],
            total_amount: 100.0,
            has_sales_order_id: false, // Missing for Won status
            has_lost_reason: false,
        }];

        let result = evaluator.evaluate(&quotes).unwrap();
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Status consistency")));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = SalesQuoteEvaluator::new();
        let result = evaluator.evaluate(&[]).unwrap();
        assert!(result.passes);
    }
}
