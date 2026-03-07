//! Coherence integration tests for the datasynth-eval crate.
//!
//! Tests IC matching with tolerance, document chain validation,
//! and balance sheet equation checks working together.

#![allow(clippy::unwrap_used)]

use datasynth_eval::{
    AccountType, BalanceSheetEvaluator, BalanceSnapshot, CoherenceEvaluation,
    DocumentChainEvaluator, DocumentReferenceData, EvaluationThresholds, ICMatchingData,
    ICMatchingEvaluator, O2CChainData, P2PChainData, UnmatchedICItem,
};
use rust_decimal_macros::dec;
use std::collections::HashMap;

// ============================================================================
// Intercompany Matching with Tolerance
// ============================================================================

#[test]
fn test_ic_matching_within_tolerance() {
    let evaluator = ICMatchingEvaluator::new(dec!(1.00)); // $1 tolerance

    let data = ICMatchingData {
        total_pairs: 5,
        matched_pairs: 4,
        total_receivables: dec!(50000),
        total_payables: dec!(49999.50),
        unmatched_items: vec![UnmatchedICItem {
            company: "1000".to_string(),
            counterparty: "2000".to_string(),
            amount: dec!(0.50), // Within $1 tolerance
            is_receivable: true,
        }],
        gross_volume: None,
        net_settlement: None,
    };

    let result = evaluator.evaluate(&data).unwrap();
    assert_eq!(result.within_tolerance_count, 1);
    assert_eq!(result.outside_tolerance_count, 0);
    assert_eq!(
        result.discrepancy_count, 0,
        "Within-tolerance items should not count as discrepancies"
    );
}

#[test]
fn test_ic_matching_outside_tolerance() {
    let evaluator = ICMatchingEvaluator::new(dec!(0.01)); // $0.01 tolerance

    let data = ICMatchingData {
        total_pairs: 3,
        matched_pairs: 1,
        total_receivables: dec!(30000),
        total_payables: dec!(25000),
        unmatched_items: vec![UnmatchedICItem {
            company: "1000".to_string(),
            counterparty: "3000".to_string(),
            amount: dec!(5000), // Way outside $0.01 tolerance
            is_receivable: true,
        }],
        gross_volume: None,
        net_settlement: None,
    };

    let result = evaluator.evaluate(&data).unwrap();
    assert_eq!(result.outside_tolerance_count, 1);
    assert_eq!(result.discrepancy_count, 1);
    assert_eq!(result.net_position, dec!(5000));
}

#[test]
fn test_ic_netting_efficiency() {
    let evaluator = ICMatchingEvaluator::default();

    let data = ICMatchingData {
        total_pairs: 10,
        matched_pairs: 10,
        total_receivables: dec!(100000),
        total_payables: dec!(100000),
        unmatched_items: vec![],
        gross_volume: Some(dec!(200000)),
        net_settlement: Some(dec!(20000)), // 90% netting efficiency
    };

    let result = evaluator.evaluate(&data).unwrap();
    let efficiency = result.netting_efficiency.unwrap();
    assert!(
        (efficiency - 0.9).abs() < 0.01,
        "Netting efficiency should be ~90%, got {efficiency}"
    );
}

// ============================================================================
// Document Chain Validation
// ============================================================================

#[test]
fn test_complete_document_chains() {
    let evaluator = DocumentChainEvaluator::new();

    let p2p = vec![
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

    let o2c = vec![O2CChainData {
        is_complete: true,
        has_so: true,
        has_delivery: true,
        has_invoice: true,
        has_receipt: true,
        credit_check_passed: true,
    }];

    let refs = DocumentReferenceData {
        total_references: 10,
        valid_references: 10,
        orphan_count: 0,
    };

    let result = evaluator.evaluate(&p2p, &o2c, &refs).unwrap();
    assert_eq!(result.p2p_completion_rate, 1.0);
    assert_eq!(result.o2c_completion_rate, 1.0);
    assert_eq!(result.p2p_three_way_match_rate, 1.0);
    assert_eq!(result.reference_integrity_score, 1.0);
    assert_eq!(result.broken_references, 0);
}

#[test]
fn test_incomplete_document_chains() {
    let evaluator = DocumentChainEvaluator::new();

    let p2p = vec![
        P2PChainData {
            is_complete: false,
            has_po: true,
            has_gr: true,
            has_invoice: false, // Missing invoice
            has_payment: false, // Missing payment
            three_way_match_passed: false,
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

    let o2c: Vec<O2CChainData> = vec![];

    let refs = DocumentReferenceData {
        total_references: 8,
        valid_references: 6,
        orphan_count: 1,
    };

    let result = evaluator.evaluate(&p2p, &o2c, &refs).unwrap();
    assert_eq!(result.p2p_completion_rate, 0.5);
    assert_eq!(result.p2p_three_way_match_rate, 0.5);
    assert_eq!(result.broken_references, 2);
    assert!(result.reference_integrity_score < 1.0);
    // O2C with no chains should be 100% (vacuously true)
    assert_eq!(result.o2c_completion_rate, 1.0);
}

// ============================================================================
// Combined Coherence Evaluation
// ============================================================================

#[test]
fn test_coherence_all_passing() {
    let mut eval = CoherenceEvaluation::new();

    // Add passing balance
    let balance_evaluator = BalanceSheetEvaluator::default();
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000));
    balances.insert(AccountType::Liability, dec!(600));
    balances.insert(AccountType::Equity, dec!(400));
    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };
    eval.balance = Some(balance_evaluator.evaluate(&[snapshot]).unwrap());

    // Add passing IC
    let ic_evaluator = ICMatchingEvaluator::default();
    let ic_data = ICMatchingData {
        total_pairs: 5,
        matched_pairs: 5,
        total_receivables: dec!(50000),
        total_payables: dec!(50000),
        unmatched_items: vec![],
        gross_volume: None,
        net_settlement: None,
    };
    eval.intercompany = Some(ic_evaluator.evaluate(&ic_data).unwrap());

    // Add passing document chain
    let doc_evaluator = DocumentChainEvaluator::new();
    let p2p = vec![P2PChainData {
        is_complete: true,
        has_po: true,
        has_gr: true,
        has_invoice: true,
        has_payment: true,
        three_way_match_passed: true,
    }];
    let o2c = vec![O2CChainData {
        is_complete: true,
        has_so: true,
        has_delivery: true,
        has_invoice: true,
        has_receipt: true,
        credit_check_passed: true,
    }];
    let refs = DocumentReferenceData {
        total_references: 5,
        valid_references: 5,
        orphan_count: 0,
    };
    eval.document_chain = Some(doc_evaluator.evaluate(&p2p, &o2c, &refs).unwrap());

    let thresholds = EvaluationThresholds::default();
    eval.check_thresholds(&thresholds);

    assert!(
        eval.passes,
        "All evaluations passing should yield overall pass"
    );
    assert!(
        eval.failures.is_empty(),
        "No failures expected: {:?}",
        eval.failures
    );
}

#[test]
fn test_coherence_balance_failure_propagates() {
    let mut eval = CoherenceEvaluation::new();

    // Intentionally imbalanced
    let balance_evaluator = BalanceSheetEvaluator::default();
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000));
    balances.insert(AccountType::Liability, dec!(500));
    balances.insert(AccountType::Equity, dec!(200)); // Missing 300
    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };
    eval.balance = Some(balance_evaluator.evaluate(&[snapshot]).unwrap());

    let thresholds = EvaluationThresholds::default();
    eval.check_thresholds(&thresholds);

    assert!(!eval.passes, "Imbalanced balance sheet should fail");
    assert!(
        eval.failures.iter().any(|f| f.contains("Balance sheet")),
        "Should have balance sheet failure message"
    );
}

#[test]
fn test_coherence_ic_failure_propagates() {
    let mut eval = CoherenceEvaluation::new();

    // Low IC match rate
    let ic_evaluator = ICMatchingEvaluator::default();
    let ic_data = ICMatchingData {
        total_pairs: 10,
        matched_pairs: 2, // Only 20% matched
        total_receivables: dec!(100000),
        total_payables: dec!(80000),
        unmatched_items: vec![],
        gross_volume: None,
        net_settlement: None,
    };
    eval.intercompany = Some(ic_evaluator.evaluate(&ic_data).unwrap());

    let thresholds = EvaluationThresholds::default();
    eval.check_thresholds(&thresholds);

    assert!(
        !eval.passes,
        "Low IC match rate should fail (20% < threshold)"
    );
    assert!(
        eval.failures.iter().any(|f| f.contains("IC match rate")),
        "Should have IC match rate failure"
    );
}
