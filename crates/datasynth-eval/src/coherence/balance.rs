//! Balance sheet equation validation.
//!
//! Validates that Assets = Liabilities + Equity + Net Income across all periods.

use crate::error::{EvalError, EvalResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of balance sheet evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetEvaluation {
    /// Whether the balance sheet equation holds.
    pub equation_balanced: bool,
    /// Maximum imbalance observed across all periods.
    pub max_imbalance: Decimal,
    /// Number of periods evaluated.
    pub periods_evaluated: usize,
    /// Number of periods with imbalance.
    pub periods_imbalanced: usize,
    /// Per-period results.
    pub period_results: Vec<PeriodBalanceResult>,
    /// Companies evaluated.
    pub companies_evaluated: usize,
}

/// Balance result for a single period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodBalanceResult {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: u16,
    /// Fiscal period (month).
    pub fiscal_period: u8,
    /// Total assets.
    pub total_assets: Decimal,
    /// Total liabilities.
    pub total_liabilities: Decimal,
    /// Total equity.
    pub total_equity: Decimal,
    /// Net income (Revenue - Expenses).
    pub net_income: Decimal,
    /// Imbalance amount (should be zero).
    pub imbalance: Decimal,
    /// Whether this period is balanced.
    pub is_balanced: bool,
}

/// Input for balance sheet evaluation.
#[derive(Debug, Clone)]
pub struct BalanceSnapshot {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: u16,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Account balances by account type.
    pub balances: HashMap<AccountType, Decimal>,
}

/// Account types for balance sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    Asset,
    ContraAsset,
    Liability,
    ContraLiability,
    Equity,
    ContraEquity,
    Revenue,
    Expense,
}

/// Evaluator for balance sheet equations.
pub struct BalanceSheetEvaluator {
    /// Tolerance for balance differences.
    tolerance: Decimal,
}

impl BalanceSheetEvaluator {
    /// Create a new evaluator with the specified tolerance.
    pub fn new(tolerance: Decimal) -> Self {
        Self { tolerance }
    }

    /// Evaluate balance sheet equation across all snapshots.
    pub fn evaluate(&self, snapshots: &[BalanceSnapshot]) -> EvalResult<BalanceSheetEvaluation> {
        if snapshots.is_empty() {
            return Err(EvalError::MissingData(
                "No balance snapshots provided".to_string(),
            ));
        }

        let mut period_results = Vec::new();
        let mut max_imbalance = Decimal::ZERO;
        let mut periods_imbalanced = 0;
        let companies: std::collections::HashSet<_> =
            snapshots.iter().map(|s| &s.company_code).collect();

        for snapshot in snapshots {
            let result = self.evaluate_snapshot(snapshot);
            if !result.is_balanced {
                periods_imbalanced += 1;
            }
            if result.imbalance.abs() > max_imbalance {
                max_imbalance = result.imbalance.abs();
            }
            period_results.push(result);
        }

        let equation_balanced = periods_imbalanced == 0;

        Ok(BalanceSheetEvaluation {
            equation_balanced,
            max_imbalance,
            periods_evaluated: snapshots.len(),
            periods_imbalanced,
            period_results,
            companies_evaluated: companies.len(),
        })
    }

    /// Evaluate a single balance snapshot.
    fn evaluate_snapshot(&self, snapshot: &BalanceSnapshot) -> PeriodBalanceResult {
        let get_balance = |account_type: AccountType| {
            snapshot
                .balances
                .get(&account_type)
                .copied()
                .unwrap_or(Decimal::ZERO)
        };

        // Calculate totals
        let total_assets = get_balance(AccountType::Asset) - get_balance(AccountType::ContraAsset);
        let total_liabilities =
            get_balance(AccountType::Liability) - get_balance(AccountType::ContraLiability);
        let total_equity =
            get_balance(AccountType::Equity) - get_balance(AccountType::ContraEquity);
        let net_income = get_balance(AccountType::Revenue) - get_balance(AccountType::Expense);

        // Balance equation: Assets = Liabilities + Equity + Net Income
        let imbalance = total_assets - (total_liabilities + total_equity + net_income);
        let is_balanced = imbalance.abs() <= self.tolerance;

        PeriodBalanceResult {
            company_code: snapshot.company_code.clone(),
            fiscal_year: snapshot.fiscal_year,
            fiscal_period: snapshot.fiscal_period,
            total_assets,
            total_liabilities,
            total_equity,
            net_income,
            imbalance,
            is_balanced,
        }
    }
}

impl Default for BalanceSheetEvaluator {
    fn default() -> Self {
        Self::new(Decimal::new(1, 2)) // 0.01 tolerance
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_balanced_snapshot() -> BalanceSnapshot {
        let mut balances = HashMap::new();
        balances.insert(AccountType::Asset, Decimal::new(100000, 2));
        balances.insert(AccountType::Liability, Decimal::new(40000, 2));
        balances.insert(AccountType::Equity, Decimal::new(50000, 2));
        balances.insert(AccountType::Revenue, Decimal::new(20000, 2));
        balances.insert(AccountType::Expense, Decimal::new(10000, 2));
        // Assets (1000) = Liabilities (400) + Equity (500) + Net Income (200-100=100)

        BalanceSnapshot {
            company_code: "1000".to_string(),
            fiscal_year: 2024,
            fiscal_period: 1,
            balances,
        }
    }

    #[test]
    fn test_balanced_snapshot() {
        let evaluator = BalanceSheetEvaluator::default();
        let snapshot = create_balanced_snapshot();
        let result = evaluator.evaluate(&[snapshot]).unwrap();

        assert!(result.equation_balanced);
        assert_eq!(result.periods_imbalanced, 0);
    }

    #[test]
    fn test_imbalanced_snapshot() {
        let mut snapshot = create_balanced_snapshot();
        snapshot
            .balances
            .insert(AccountType::Asset, Decimal::new(110000, 2)); // Add 100 to assets

        let evaluator = BalanceSheetEvaluator::default();
        let result = evaluator.evaluate(&[snapshot]).unwrap();

        assert!(!result.equation_balanced);
        assert_eq!(result.periods_imbalanced, 1);
        assert!(result.max_imbalance > Decimal::ZERO);
    }

    #[test]
    fn test_empty_snapshots() {
        let evaluator = BalanceSheetEvaluator::default();
        let result = evaluator.evaluate(&[]);
        assert!(matches!(result, Err(EvalError::MissingData(_))));
    }
}
