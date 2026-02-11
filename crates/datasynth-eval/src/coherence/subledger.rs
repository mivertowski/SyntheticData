//! Subledger-to-GL reconciliation evaluation.
//!
//! Validates that subledger balances (AR, AP, FA, Inventory) reconcile
//! to their corresponding GL control accounts.

use crate::error::EvalResult;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Results of subledger reconciliation evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerReconciliationEvaluation {
    /// AR reconciliation status.
    pub ar_reconciled: bool,
    /// AR GL balance.
    pub ar_gl_balance: Decimal,
    /// AR subledger balance.
    pub ar_subledger_balance: Decimal,
    /// AR difference.
    pub ar_difference: Decimal,
    /// AP reconciliation status.
    pub ap_reconciled: bool,
    /// AP GL balance.
    pub ap_gl_balance: Decimal,
    /// AP subledger balance.
    pub ap_subledger_balance: Decimal,
    /// AP difference.
    pub ap_difference: Decimal,
    /// FA asset reconciliation status.
    pub fa_asset_reconciled: bool,
    /// FA asset GL balance.
    pub fa_asset_gl_balance: Decimal,
    /// FA asset subledger balance.
    pub fa_asset_subledger_balance: Decimal,
    /// FA asset difference.
    pub fa_asset_difference: Decimal,
    /// FA accumulated depreciation reconciliation status.
    pub fa_accum_depr_reconciled: bool,
    /// FA accumulated depreciation GL balance.
    pub fa_accum_depr_gl_balance: Decimal,
    /// FA accumulated depreciation subledger balance.
    pub fa_accum_depr_subledger_balance: Decimal,
    /// FA accumulated depreciation difference.
    pub fa_accum_depr_difference: Decimal,
    /// Inventory reconciliation status.
    pub inventory_reconciled: bool,
    /// Inventory GL balance.
    pub inventory_gl_balance: Decimal,
    /// Inventory subledger balance.
    pub inventory_subledger_balance: Decimal,
    /// Inventory difference.
    pub inventory_difference: Decimal,
    /// Overall completeness score (0.0-1.0).
    pub completeness_score: f64,
    /// Number of subledgers reconciled.
    pub subledgers_reconciled: usize,
    /// Total subledgers checked.
    pub subledgers_total: usize,
}

/// Input for subledger reconciliation.
#[derive(Debug, Clone, Default)]
pub struct SubledgerData {
    /// AR GL control account balance.
    pub ar_gl_balance: Option<Decimal>,
    /// Sum of AR invoice balances.
    pub ar_subledger_balance: Option<Decimal>,
    /// AP GL control account balance.
    pub ap_gl_balance: Option<Decimal>,
    /// Sum of AP invoice balances.
    pub ap_subledger_balance: Option<Decimal>,
    /// FA asset GL control account balance.
    pub fa_asset_gl_balance: Option<Decimal>,
    /// Sum of FA asset values.
    pub fa_asset_subledger_balance: Option<Decimal>,
    /// FA accumulated depreciation GL balance.
    pub fa_accum_depr_gl_balance: Option<Decimal>,
    /// Sum of FA accumulated depreciation.
    pub fa_accum_depr_subledger_balance: Option<Decimal>,
    /// Inventory GL control account balance.
    pub inventory_gl_balance: Option<Decimal>,
    /// Sum of inventory position values.
    pub inventory_subledger_balance: Option<Decimal>,
}

/// Evaluator for subledger reconciliation.
pub struct SubledgerEvaluator {
    /// Tolerance for reconciliation differences.
    tolerance: Decimal,
}

impl SubledgerEvaluator {
    /// Create a new evaluator with the specified tolerance.
    pub fn new(tolerance: Decimal) -> Self {
        Self { tolerance }
    }

    /// Evaluate subledger reconciliation.
    pub fn evaluate(&self, data: &SubledgerData) -> EvalResult<SubledgerReconciliationEvaluation> {
        let mut subledgers_reconciled = 0;
        let mut subledgers_total = 0;

        // AR reconciliation
        let (ar_reconciled, ar_gl, ar_sub, ar_diff) = self.check_reconciliation(
            data.ar_gl_balance,
            data.ar_subledger_balance,
            &mut subledgers_reconciled,
            &mut subledgers_total,
        );

        // AP reconciliation
        let (ap_reconciled, ap_gl, ap_sub, ap_diff) = self.check_reconciliation(
            data.ap_gl_balance,
            data.ap_subledger_balance,
            &mut subledgers_reconciled,
            &mut subledgers_total,
        );

        // FA asset reconciliation
        let (fa_asset_reconciled, fa_asset_gl, fa_asset_sub, fa_asset_diff) = self
            .check_reconciliation(
                data.fa_asset_gl_balance,
                data.fa_asset_subledger_balance,
                &mut subledgers_reconciled,
                &mut subledgers_total,
            );

        // FA accumulated depreciation reconciliation
        let (fa_accum_reconciled, fa_accum_gl, fa_accum_sub, fa_accum_diff) = self
            .check_reconciliation(
                data.fa_accum_depr_gl_balance,
                data.fa_accum_depr_subledger_balance,
                &mut subledgers_reconciled,
                &mut subledgers_total,
            );

        // Inventory reconciliation
        let (inv_reconciled, inv_gl, inv_sub, inv_diff) = self.check_reconciliation(
            data.inventory_gl_balance,
            data.inventory_subledger_balance,
            &mut subledgers_reconciled,
            &mut subledgers_total,
        );

        let completeness_score = if subledgers_total > 0 {
            subledgers_reconciled as f64 / subledgers_total as f64
        } else {
            1.0 // No subledgers to reconcile = 100% complete
        };

        Ok(SubledgerReconciliationEvaluation {
            ar_reconciled,
            ar_gl_balance: ar_gl,
            ar_subledger_balance: ar_sub,
            ar_difference: ar_diff,
            ap_reconciled,
            ap_gl_balance: ap_gl,
            ap_subledger_balance: ap_sub,
            ap_difference: ap_diff,
            fa_asset_reconciled,
            fa_asset_gl_balance: fa_asset_gl,
            fa_asset_subledger_balance: fa_asset_sub,
            fa_asset_difference: fa_asset_diff,
            fa_accum_depr_reconciled: fa_accum_reconciled,
            fa_accum_depr_gl_balance: fa_accum_gl,
            fa_accum_depr_subledger_balance: fa_accum_sub,
            fa_accum_depr_difference: fa_accum_diff,
            inventory_reconciled: inv_reconciled,
            inventory_gl_balance: inv_gl,
            inventory_subledger_balance: inv_sub,
            inventory_difference: inv_diff,
            completeness_score,
            subledgers_reconciled,
            subledgers_total,
        })
    }

    /// Check reconciliation for a single subledger.
    fn check_reconciliation(
        &self,
        gl_balance: Option<Decimal>,
        subledger_balance: Option<Decimal>,
        reconciled_count: &mut usize,
        total_count: &mut usize,
    ) -> (bool, Decimal, Decimal, Decimal) {
        match (gl_balance, subledger_balance) {
            (Some(gl), Some(sub)) => {
                *total_count += 1;
                let diff = gl - sub;
                let is_reconciled = diff.abs() <= self.tolerance;
                if is_reconciled {
                    *reconciled_count += 1;
                }
                (is_reconciled, gl, sub, diff)
            }
            _ => (true, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        }
    }
}

impl Default for SubledgerEvaluator {
    fn default() -> Self {
        Self::new(Decimal::new(1, 2)) // 0.01 tolerance
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciled_subledgers() {
        let data = SubledgerData {
            ar_gl_balance: Some(Decimal::new(100000, 2)),
            ar_subledger_balance: Some(Decimal::new(100000, 2)),
            ap_gl_balance: Some(Decimal::new(50000, 2)),
            ap_subledger_balance: Some(Decimal::new(50000, 2)),
            ..Default::default()
        };

        let evaluator = SubledgerEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert!(result.ar_reconciled);
        assert!(result.ap_reconciled);
        assert_eq!(result.completeness_score, 1.0);
    }

    #[test]
    fn test_unreconciled_subledger() {
        let data = SubledgerData {
            ar_gl_balance: Some(Decimal::new(100000, 2)),
            ar_subledger_balance: Some(Decimal::new(99000, 2)), // 10.00 difference
            ..Default::default()
        };

        let evaluator = SubledgerEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert!(!result.ar_reconciled);
        assert_eq!(result.ar_difference, Decimal::new(1000, 2));
    }

    #[test]
    fn test_no_subledger_data() {
        let data = SubledgerData::default();
        let evaluator = SubledgerEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        // With no data, should be considered complete
        assert_eq!(result.completeness_score, 1.0);
    }
}
