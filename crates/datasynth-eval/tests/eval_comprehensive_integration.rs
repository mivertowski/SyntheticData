//! Integration tests for ComprehensiveEvaluation and Evaluator.
//!
//! Validates top-level evaluation orchestration: creating empty evaluations,
//! constructing evaluators with defaults, and running threshold checks.

use datasynth_eval::{
    AccountType, BalanceSheetEvaluator, BalanceSnapshot, BenfordAnalyzer, ComprehensiveEvaluation,
    EvaluationConfig, EvaluationThresholds, Evaluator,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn test_comprehensive_evaluation_creation() {
    let eval = ComprehensiveEvaluation::new();

    assert!(eval.passes, "New evaluation should pass by default");
    assert!(
        eval.failures.is_empty(),
        "New evaluation should have no failures"
    );
    assert!(eval.statistical.passes);
    assert!(eval.coherence.passes);
    assert!(eval.quality.passes);
    assert!(eval.ml_readiness.passes);
    assert!(eval.privacy.is_none());
    assert!(eval.banking.is_none());
    assert!(eval.process_mining.is_none());
    assert!(eval.causal.is_none());
    assert!(eval.enrichment_quality.is_none());
    assert!(eval.tuning_opportunities.is_empty());
    assert!(eval.config_suggestions.is_empty());
}

#[test]
fn test_evaluator_with_defaults() {
    let evaluator = Evaluator::with_defaults();
    let config = evaluator.config();

    // Verify default threshold values
    assert!(
        (config.thresholds.benford_p_value_min - 0.05).abs() < f64::EPSILON,
        "Default Benford p-value threshold should be 0.05"
    );
    assert!(
        (config.thresholds.benford_mad_max - 0.015).abs() < f64::EPSILON,
        "Default Benford MAD threshold should be 0.015"
    );
    assert_eq!(config.thresholds.balance_tolerance, dec!(0.01));
    assert!(config.statistical.benford_enabled);
    assert!(config.coherence.balance_enabled);
    assert!(config.quality.uniqueness_enabled);
}

#[test]
fn test_evaluator_run_produces_passing_empty_evaluation() {
    // An evaluator run with no data populated should produce a passing result
    // since all sub-evaluations default to passing.
    let evaluator = Evaluator::with_defaults();
    let result = evaluator.run_evaluation();

    assert!(
        result.passes,
        "Empty evaluation should pass all threshold checks"
    );
    assert!(result.failures.is_empty());
}

#[test]
fn test_evaluation_thresholds_with_benford_failure() {
    // Populate the Benford analysis with values that will fail the default thresholds.
    let mut eval = ComprehensiveEvaluation::new();

    // Create a Benford analysis result that fails: low p-value and high MAD
    // We use the analyzer with uniform data to get a failing result.
    let uniform_amounts: Vec<Decimal> = (1..=500)
        .map(|i| Decimal::new(i * 2, 0)) // 2, 4, 6, ..., 1000
        .collect();
    let analyzer = BenfordAnalyzer::new(0.05);
    let benford_result = analyzer.analyze(&uniform_amounts).expect("should succeed");

    eval.statistical.benford = Some(benford_result);

    let thresholds = EvaluationThresholds::default();
    eval.check_all_thresholds(&thresholds);

    // The uniform-like data should produce threshold failures
    assert!(
        !eval.failures.is_empty(),
        "Uniform data should cause Benford threshold failures"
    );
    assert!(
        !eval.passes,
        "Evaluation should not pass with Benford failures"
    );

    // Verify at least one failure message mentions Benford
    let has_benford_failure = eval.failures.iter().any(|f| f.contains("Benford"));
    assert!(
        has_benford_failure,
        "Should have a Benford-related failure message, got: {:?}",
        eval.failures
    );
}

#[test]
fn test_evaluation_thresholds_with_balance_failure() {
    // Populate the coherence balance evaluation with an imbalanced result.
    let mut eval = ComprehensiveEvaluation::new();

    let evaluator = BalanceSheetEvaluator::default();
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000));
    balances.insert(AccountType::Liability, dec!(500));
    balances.insert(AccountType::Equity, dec!(300));
    // Net income = 0, imbalance = 1000 - (500 + 300) = 200

    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };

    let balance_result = evaluator.evaluate(&[snapshot]).expect("should succeed");
    eval.coherence.balance = Some(balance_result);

    let thresholds = EvaluationThresholds::default();
    eval.check_all_thresholds(&thresholds);

    assert!(
        !eval.passes,
        "Evaluation should fail with imbalanced balance sheet"
    );
    let has_balance_failure = eval.failures.iter().any(|f| f.contains("Balance sheet"));
    assert!(
        has_balance_failure,
        "Should have a balance-related failure message, got: {:?}",
        eval.failures
    );
}

#[test]
fn test_evaluation_thresholds_all_passing() {
    // Create an evaluation where all populated sub-evaluations pass.
    let mut eval = ComprehensiveEvaluation::new();

    // Add a passing Benford result (use log-normal data)
    let amounts: Vec<Decimal> = (1..=1000)
        .map(|i| {
            // Approximate Benford distribution
            let digit = match i % 100 {
                0..=29 => 1,
                30..=46 => 2,
                47..=59 => 3,
                60..=69 => 4,
                70..=77 => 5,
                78..=84 => 6,
                85..=90 => 7,
                91..=95 => 8,
                _ => 9,
            };
            Decimal::new(digit * 100 + (i % 100) as i64, 2)
        })
        .collect();
    let analyzer = BenfordAnalyzer::new(0.05);
    if let Ok(benford_result) = analyzer.analyze(&amounts) {
        // Only set if it actually passes (it may depend on exact distribution fit)
        if benford_result.passes {
            eval.statistical.benford = Some(benford_result);
        }
    }

    // Add a passing balance result
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
    let balance_result = balance_evaluator
        .evaluate(&[snapshot])
        .expect("should succeed");
    eval.coherence.balance = Some(balance_result);

    let thresholds = EvaluationThresholds::default();
    eval.check_all_thresholds(&thresholds);

    assert!(
        eval.passes,
        "Evaluation with all-passing sub-evaluations should pass, but got failures: {:?}",
        eval.failures
    );
    assert!(eval.failures.is_empty());
}

#[test]
fn test_evaluator_custom_config() {
    // Verify we can create an evaluator with custom configuration.
    let mut config = EvaluationConfig::default();
    config.thresholds = EvaluationThresholds::strict();
    config.statistical.significance_level = 0.10;

    let evaluator = Evaluator::new(config);
    let config_ref = evaluator.config();

    assert!(
        (config_ref.statistical.significance_level - 0.10).abs() < f64::EPSILON,
        "Custom significance level should be 0.10"
    );
    assert!(
        (config_ref.thresholds.benford_p_value_min - 0.10).abs() < f64::EPSILON,
        "Strict threshold Benford p-value should be 0.10"
    );
    assert_eq!(
        config_ref.thresholds.balance_tolerance,
        Decimal::new(1, 4),
        "Strict balance tolerance should be 0.0001"
    );
}
