//! Document chain completeness evaluation.
//!
//! Validates that P2P (Procure-to-Pay) and O2C (Order-to-Cash) document
//! chains are complete and maintain referential integrity.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Results of document chain evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChainEvaluation {
    /// Total P2P chains.
    pub p2p_total: usize,
    /// Complete P2P chains.
    pub p2p_complete: usize,
    /// P2P completion rate.
    pub p2p_completion_rate: f64,
    /// P2P chains with PO created.
    pub p2p_po_created: usize,
    /// P2P chains with GR created.
    pub p2p_gr_created: usize,
    /// P2P chains with invoice created.
    pub p2p_invoice_created: usize,
    /// P2P chains with payment created.
    pub p2p_payment_created: usize,
    /// P2P chains where three-way match passed.
    pub p2p_three_way_match_passed: usize,
    /// P2P three-way match rate.
    pub p2p_three_way_match_rate: f64,
    /// Total O2C chains.
    pub o2c_total: usize,
    /// Complete O2C chains.
    pub o2c_complete: usize,
    /// O2C completion rate.
    pub o2c_completion_rate: f64,
    /// O2C chains with SO created.
    pub o2c_so_created: usize,
    /// O2C chains with delivery created.
    pub o2c_delivery_created: usize,
    /// O2C chains with invoice created.
    pub o2c_invoice_created: usize,
    /// O2C chains with receipt created.
    pub o2c_receipt_created: usize,
    /// O2C chains where credit check passed.
    pub o2c_credit_check_passed: usize,
    /// O2C credit check pass rate.
    pub o2c_credit_check_rate: f64,
    /// Number of orphan documents (no chain reference).
    pub orphan_documents: usize,
    /// Number of broken references.
    pub broken_references: usize,
    /// Reference integrity score (0.0-1.0).
    pub reference_integrity_score: f64,
}

/// Input for P2P chain evaluation.
#[derive(Debug, Clone)]
pub struct P2PChainData {
    /// Whether chain is complete (all documents present).
    pub is_complete: bool,
    /// Whether PO was created.
    pub has_po: bool,
    /// Whether GR was created.
    pub has_gr: bool,
    /// Whether invoice was created.
    pub has_invoice: bool,
    /// Whether payment was created.
    pub has_payment: bool,
    /// Whether three-way match passed.
    pub three_way_match_passed: bool,
}

/// Input for O2C chain evaluation.
#[derive(Debug, Clone)]
pub struct O2CChainData {
    /// Whether chain is complete.
    pub is_complete: bool,
    /// Whether SO was created.
    pub has_so: bool,
    /// Whether delivery was created.
    pub has_delivery: bool,
    /// Whether invoice was created.
    pub has_invoice: bool,
    /// Whether receipt was created.
    pub has_receipt: bool,
    /// Whether credit check passed.
    pub credit_check_passed: bool,
}

/// Document reference data.
#[derive(Debug, Clone)]
pub struct DocumentReferenceData {
    /// Total document references.
    pub total_references: usize,
    /// Valid references (target document exists).
    pub valid_references: usize,
    /// Orphan documents (no incoming references).
    pub orphan_count: usize,
}

/// Evaluator for document chain completeness.
pub struct DocumentChainEvaluator;

impl DocumentChainEvaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate document chains.
    pub fn evaluate(
        &self,
        p2p_chains: &[P2PChainData],
        o2c_chains: &[O2CChainData],
        references: &DocumentReferenceData,
    ) -> EvalResult<DocumentChainEvaluation> {
        // P2P analysis
        let p2p_total = p2p_chains.len();
        let p2p_complete = p2p_chains.iter().filter(|c| c.is_complete).count();
        let p2p_completion_rate = if p2p_total > 0 {
            p2p_complete as f64 / p2p_total as f64
        } else {
            1.0
        };

        let p2p_po_created = p2p_chains.iter().filter(|c| c.has_po).count();
        let p2p_gr_created = p2p_chains.iter().filter(|c| c.has_gr).count();
        let p2p_invoice_created = p2p_chains.iter().filter(|c| c.has_invoice).count();
        let p2p_payment_created = p2p_chains.iter().filter(|c| c.has_payment).count();
        let p2p_three_way_match_passed = p2p_chains
            .iter()
            .filter(|c| c.three_way_match_passed)
            .count();
        let p2p_three_way_match_rate = if p2p_total > 0 {
            p2p_three_way_match_passed as f64 / p2p_total as f64
        } else {
            1.0
        };

        // O2C analysis
        let o2c_total = o2c_chains.len();
        let o2c_complete = o2c_chains.iter().filter(|c| c.is_complete).count();
        let o2c_completion_rate = if o2c_total > 0 {
            o2c_complete as f64 / o2c_total as f64
        } else {
            1.0
        };

        let o2c_so_created = o2c_chains.iter().filter(|c| c.has_so).count();
        let o2c_delivery_created = o2c_chains.iter().filter(|c| c.has_delivery).count();
        let o2c_invoice_created = o2c_chains.iter().filter(|c| c.has_invoice).count();
        let o2c_receipt_created = o2c_chains.iter().filter(|c| c.has_receipt).count();
        let o2c_credit_check_passed = o2c_chains.iter().filter(|c| c.credit_check_passed).count();
        let o2c_credit_check_rate = if o2c_total > 0 {
            o2c_credit_check_passed as f64 / o2c_total as f64
        } else {
            1.0
        };

        // Reference integrity
        let broken_references = references.total_references - references.valid_references;
        let reference_integrity_score = if references.total_references > 0 {
            references.valid_references as f64 / references.total_references as f64
        } else {
            1.0
        };

        Ok(DocumentChainEvaluation {
            p2p_total,
            p2p_complete,
            p2p_completion_rate,
            p2p_po_created,
            p2p_gr_created,
            p2p_invoice_created,
            p2p_payment_created,
            p2p_three_way_match_passed,
            p2p_three_way_match_rate,
            o2c_total,
            o2c_complete,
            o2c_completion_rate,
            o2c_so_created,
            o2c_delivery_created,
            o2c_invoice_created,
            o2c_receipt_created,
            o2c_credit_check_passed,
            o2c_credit_check_rate,
            orphan_documents: references.orphan_count,
            broken_references,
            reference_integrity_score,
        })
    }
}

impl Default for DocumentChainEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_p2p_chains() {
        let p2p_chains = vec![
            P2PChainData {
                is_complete: true,
                has_po: true,
                has_gr: true,
                has_invoice: true,
                has_payment: true,
                three_way_match_passed: true,
            },
            P2PChainData {
                is_complete: true,
                has_po: true,
                has_gr: true,
                has_invoice: true,
                has_payment: true,
                three_way_match_passed: true,
            },
        ];

        let o2c_chains = vec![];
        let references = DocumentReferenceData {
            total_references: 10,
            valid_references: 10,
            orphan_count: 0,
        };

        let evaluator = DocumentChainEvaluator::new();
        let result = evaluator
            .evaluate(&p2p_chains, &o2c_chains, &references)
            .unwrap();

        assert_eq!(result.p2p_total, 2);
        assert_eq!(result.p2p_complete, 2);
        assert_eq!(result.p2p_completion_rate, 1.0);
        assert_eq!(result.p2p_three_way_match_rate, 1.0);
    }

    #[test]
    fn test_incomplete_chains() {
        let p2p_chains = vec![P2PChainData {
            is_complete: false,
            has_po: true,
            has_gr: true,
            has_invoice: false,
            has_payment: false,
            three_way_match_passed: false,
        }];

        let o2c_chains = vec![];
        let references = DocumentReferenceData {
            total_references: 5,
            valid_references: 4,
            orphan_count: 1,
        };

        let evaluator = DocumentChainEvaluator::new();
        let result = evaluator
            .evaluate(&p2p_chains, &o2c_chains, &references)
            .unwrap();

        assert_eq!(result.p2p_completion_rate, 0.0);
        assert_eq!(result.broken_references, 1);
        assert_eq!(result.reference_integrity_score, 0.8);
    }
}
