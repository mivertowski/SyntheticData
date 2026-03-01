//! Integration tests for BalanceSheetEvaluator.
//!
//! Validates the balance sheet equation (Assets = Liabilities + Equity + Net Income)
//! across balanced snapshots, imbalanced snapshots, multi-period scenarios,
//! and tolerance edge cases.

use datasynth_eval::{AccountType, BalanceSheetEvaluator, BalanceSnapshot};
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Helper: create a balanced snapshot where
/// Assets(1000) = Liabilities(600) + Equity(400) + NetIncome(0).
fn make_balanced_snapshot(company: &str, year: u16, period: u8) -> BalanceSnapshot {
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000));
    balances.insert(AccountType::Liability, dec!(600));
    balances.insert(AccountType::Equity, dec!(400));
    // Revenue = Expense => NetIncome = 0, so equation holds
    balances.insert(AccountType::Revenue, dec!(200));
    balances.insert(AccountType::Expense, dec!(200));

    BalanceSnapshot {
        company_code: company.to_string(),
        fiscal_year: year,
        fiscal_period: period,
        balances,
    }
}

#[test]
fn test_balanced_snapshot() {
    let evaluator = BalanceSheetEvaluator::default();
    let snapshot = make_balanced_snapshot("C001", 2024, 1);
    let result = evaluator
        .evaluate(&[snapshot])
        .expect("evaluation should succeed");

    assert!(
        result.equation_balanced,
        "Snapshot with Assets=L+E+NI should be balanced"
    );
    assert_eq!(result.periods_evaluated, 1);
    assert_eq!(result.periods_imbalanced, 0);
    assert_eq!(result.companies_evaluated, 1);
    assert_eq!(result.max_imbalance, dec!(0));

    let period = &result.period_results[0];
    assert!(period.is_balanced);
    assert_eq!(period.imbalance, dec!(0));
    assert_eq!(period.total_assets, dec!(1000));
    assert_eq!(period.total_liabilities, dec!(600));
    assert_eq!(period.total_equity, dec!(400));
    assert_eq!(period.net_income, dec!(0));
}

#[test]
fn test_imbalanced_snapshot() {
    let evaluator = BalanceSheetEvaluator::default(); // tolerance = 0.01
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000));
    balances.insert(AccountType::Liability, dec!(600));
    balances.insert(AccountType::Equity, dec!(300)); // Missing 100 to balance

    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };

    let result = evaluator
        .evaluate(&[snapshot])
        .expect("evaluation should succeed");

    assert!(
        !result.equation_balanced,
        "Snapshot with Assets != L+E should be imbalanced"
    );
    assert_eq!(result.periods_imbalanced, 1);
    assert_eq!(result.max_imbalance, dec!(100));

    let period = &result.period_results[0];
    assert!(!period.is_balanced);
    assert_eq!(period.imbalance, dec!(100));
}

#[test]
fn test_multiple_periods() {
    let evaluator = BalanceSheetEvaluator::default();
    let snapshots = vec![
        make_balanced_snapshot("C001", 2024, 1),
        make_balanced_snapshot("C001", 2024, 2),
        make_balanced_snapshot("C001", 2024, 3),
    ];

    let result = evaluator
        .evaluate(&snapshots)
        .expect("evaluation should succeed");

    assert!(result.equation_balanced);
    assert_eq!(result.periods_evaluated, 3);
    assert_eq!(result.periods_imbalanced, 0);
    assert_eq!(result.companies_evaluated, 1);

    for (i, period) in result.period_results.iter().enumerate() {
        assert!(
            period.is_balanced,
            "Period {} should be balanced",
            i + 1
        );
    }
}

#[test]
fn test_tolerance_within_threshold() {
    // Default tolerance is 0.01. An imbalance of 0.005 should pass.
    let evaluator = BalanceSheetEvaluator::default();
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000.005));
    balances.insert(AccountType::Liability, dec!(600));
    balances.insert(AccountType::Equity, dec!(400));
    // Net income = 0, so imbalance = 1000.005 - (600 + 400 + 0) = 0.005

    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };

    let result = evaluator
        .evaluate(&[snapshot])
        .expect("evaluation should succeed");

    assert!(
        result.equation_balanced,
        "Imbalance of 0.005 should be within default tolerance of 0.01"
    );
    assert_eq!(result.periods_imbalanced, 0);
    // max_imbalance should be 0.005
    assert_eq!(result.max_imbalance, dec!(0.005));
}

#[test]
fn test_tolerance_exceeded() {
    // Default tolerance is 0.01. An imbalance of 0.05 should fail.
    let evaluator = BalanceSheetEvaluator::default();
    let mut balances = HashMap::new();
    balances.insert(AccountType::Asset, dec!(1000.05));
    balances.insert(AccountType::Liability, dec!(600));
    balances.insert(AccountType::Equity, dec!(400));
    // Net income = 0, so imbalance = 1000.05 - (600 + 400 + 0) = 0.05

    let snapshot = BalanceSnapshot {
        company_code: "C001".to_string(),
        fiscal_year: 2024,
        fiscal_period: 1,
        balances,
    };

    let result = evaluator
        .evaluate(&[snapshot])
        .expect("evaluation should succeed");

    assert!(
        !result.equation_balanced,
        "Imbalance of 0.05 should exceed default tolerance of 0.01"
    );
    assert_eq!(result.periods_imbalanced, 1);
    assert_eq!(result.max_imbalance, dec!(0.05));

    let period = &result.period_results[0];
    assert!(!period.is_balanced);
    assert_eq!(period.imbalance, dec!(0.05));
}
