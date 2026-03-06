//! Intercompany matching evaluation.
//!
//! Validates that intercompany transactions are properly matched
//! between company pairs.

use crate::error::EvalResult;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Results of intercompany matching evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICMatchingEvaluation {
    /// Total company pairs with IC transactions.
    pub total_pairs: usize,
    /// Number of pairs fully matched.
    pub matched_pairs: usize,
    /// Match rate (0.0-1.0).
    pub match_rate: f64,
    /// Total intercompany receivables.
    pub total_receivables: Decimal,
    /// Total intercompany payables.
    pub total_payables: Decimal,
    /// Total unmatched amount.
    pub total_unmatched: Decimal,
    /// Net position (receivables - payables).
    pub net_position: Decimal,
    /// Number of discrepancies (outside tolerance).
    pub discrepancy_count: usize,
    /// Number of unmatched items within tolerance.
    pub within_tolerance_count: usize,
    /// Number of unmatched items outside tolerance.
    pub outside_tolerance_count: usize,
    /// Netting efficiency if applicable.
    pub netting_efficiency: Option<f64>,
}

/// Input for IC matching evaluation.
#[derive(Debug, Clone)]
pub struct ICMatchingData {
    /// Total company pairs.
    pub total_pairs: usize,
    /// Matched company pairs.
    pub matched_pairs: usize,
    /// Total receivables amount.
    pub total_receivables: Decimal,
    /// Total payables amount.
    pub total_payables: Decimal,
    /// Unmatched items details.
    pub unmatched_items: Vec<UnmatchedICItem>,
    /// Gross IC volume (for netting calculation).
    pub gross_volume: Option<Decimal>,
    /// Net settlement amount (for netting calculation).
    pub net_settlement: Option<Decimal>,
}

/// An unmatched IC item.
#[derive(Debug, Clone)]
pub struct UnmatchedICItem {
    /// Company code.
    pub company: String,
    /// Counterparty company code.
    pub counterparty: String,
    /// Amount.
    pub amount: Decimal,
    /// Whether this is a receivable (true) or payable (false).
    pub is_receivable: bool,
}

/// Evaluator for intercompany matching.
pub struct ICMatchingEvaluator {
    /// Tolerance for classifying unmatched items as within/outside tolerance.
    tolerance: Decimal,
}

impl ICMatchingEvaluator {
    /// Create a new evaluator with the specified tolerance.
    pub fn new(tolerance: Decimal) -> Self {
        Self { tolerance }
    }

    /// Evaluate IC matching results.
    pub fn evaluate(&self, data: &ICMatchingData) -> EvalResult<ICMatchingEvaluation> {
        let match_rate = if data.total_pairs > 0 {
            data.matched_pairs as f64 / data.total_pairs as f64
        } else {
            1.0
        };

        let total_unmatched: Decimal = data.unmatched_items.iter().map(|i| i.amount.abs()).sum();
        let net_position = data.total_receivables - data.total_payables;

        // Classify unmatched items by tolerance
        let within_tolerance_count = data
            .unmatched_items
            .iter()
            .filter(|item| item.amount.abs() <= self.tolerance)
            .count();
        let outside_tolerance_count = data.unmatched_items.len() - within_tolerance_count;
        // Only outside-tolerance items count as true discrepancies
        let discrepancy_count = outside_tolerance_count;

        // Calculate netting efficiency if data available
        let netting_efficiency = match (data.gross_volume, data.net_settlement) {
            (Some(gross), Some(net)) if gross > Decimal::ZERO => {
                Some(1.0 - (net / gross).to_f64().unwrap_or(0.0))
            }
            _ => None,
        };

        Ok(ICMatchingEvaluation {
            total_pairs: data.total_pairs,
            matched_pairs: data.matched_pairs,
            match_rate,
            total_receivables: data.total_receivables,
            total_payables: data.total_payables,
            total_unmatched,
            net_position,
            discrepancy_count,
            within_tolerance_count,
            outside_tolerance_count,
            netting_efficiency,
        })
    }
}

impl Default for ICMatchingEvaluator {
    fn default() -> Self {
        Self::new(Decimal::new(1, 2)) // 0.01 tolerance
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_fully_matched_ic() {
        let data = ICMatchingData {
            total_pairs: 5,
            matched_pairs: 5,
            total_receivables: Decimal::new(100000, 2),
            total_payables: Decimal::new(100000, 2),
            unmatched_items: vec![],
            gross_volume: Some(Decimal::new(200000, 2)),
            net_settlement: Some(Decimal::new(20000, 2)),
        };

        let evaluator = ICMatchingEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.match_rate, 1.0);
        assert_eq!(result.total_unmatched, Decimal::ZERO);
        assert_eq!(result.net_position, Decimal::ZERO);
        assert!(result.netting_efficiency.unwrap() > 0.8);
    }

    #[test]
    fn test_partial_match() {
        let data = ICMatchingData {
            total_pairs: 10,
            matched_pairs: 8,
            total_receivables: Decimal::new(100000, 2),
            total_payables: Decimal::new(95000, 2),
            unmatched_items: vec![UnmatchedICItem {
                company: "1000".to_string(),
                counterparty: "2000".to_string(),
                amount: Decimal::new(5000, 2),
                is_receivable: true,
            }],
            gross_volume: None,
            net_settlement: None,
        };

        let evaluator = ICMatchingEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.match_rate, 0.8);
        assert_eq!(result.discrepancy_count, 1);
        assert_eq!(result.net_position, Decimal::new(5000, 2));
    }

    #[test]
    fn test_no_ic_transactions() {
        let data = ICMatchingData {
            total_pairs: 0,
            matched_pairs: 0,
            total_receivables: Decimal::ZERO,
            total_payables: Decimal::ZERO,
            unmatched_items: vec![],
            gross_volume: None,
            net_settlement: None,
        };

        let evaluator = ICMatchingEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.match_rate, 1.0); // No IC = 100% matched
    }
}
