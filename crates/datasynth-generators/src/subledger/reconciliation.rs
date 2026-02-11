//! GL-to-Subledger reconciliation module.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::accounts::control_accounts;
use datasynth_core::models::subledger::ap::APInvoice;
use datasynth_core::models::subledger::ar::ARInvoice;
use datasynth_core::models::subledger::fa::FixedAssetRecord;
use datasynth_core::models::subledger::inventory::InventoryPosition;
use datasynth_core::models::subledger::SubledgerType;

/// Local status enum for reconciliation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconStatus {
    /// Fully reconciled within tolerance.
    Reconciled,
    /// Partially reconciled (some items identified).
    PartiallyReconciled,
    /// Not reconciled.
    Unreconciled,
    /// Reconciliation in progress.
    InProgress,
}

/// An item that doesn't reconcile.
#[derive(Debug, Clone)]
pub struct UnreconciledEntry {
    /// Type of discrepancy.
    pub entry_type: String,
    /// Document number.
    pub document_number: String,
    /// Amount.
    pub amount: Decimal,
    /// Description of discrepancy.
    pub description: String,
}

/// Result of a GL-to-subledger reconciliation.
#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    /// Reconciliation ID.
    pub reconciliation_id: String,
    /// Company code.
    pub company_code: String,
    /// Subledger type.
    pub subledger_type: SubledgerType,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// GL account code.
    pub gl_account: String,
    /// GL balance.
    pub gl_balance: Decimal,
    /// Subledger balance.
    pub subledger_balance: Decimal,
    /// Difference.
    pub difference: Decimal,
    /// Status.
    pub status: ReconStatus,
    /// Unreconciled items.
    pub unreconciled_items: Vec<UnreconciledEntry>,
    /// Reconciliation date.
    pub reconciliation_date: NaiveDate,
    /// Reconciled by.
    pub reconciled_by: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ReconciliationResult {
    /// Returns true if the reconciliation is balanced.
    pub fn is_balanced(&self) -> bool {
        self.difference.abs() < dec!(0.01)
    }
}

/// Configuration for reconciliation.
#[derive(Debug, Clone)]
pub struct ReconciliationConfig {
    /// Tolerance amount for auto-reconciliation.
    pub tolerance_amount: Decimal,
    /// AR control account.
    pub ar_control_account: String,
    /// AP control account.
    pub ap_control_account: String,
    /// FA control account.
    pub fa_control_account: String,
    /// Inventory control account.
    pub inventory_control_account: String,
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            tolerance_amount: dec!(0.01),
            ar_control_account: control_accounts::AR_CONTROL.to_string(),
            ap_control_account: control_accounts::AP_CONTROL.to_string(),
            fa_control_account: control_accounts::FIXED_ASSETS.to_string(),
            inventory_control_account: control_accounts::INVENTORY.to_string(),
        }
    }
}

/// Reconciliation engine for GL-to-subledger matching.
pub struct ReconciliationEngine {
    config: ReconciliationConfig,
    reconciliation_counter: u64,
}

impl ReconciliationEngine {
    /// Creates a new reconciliation engine.
    pub fn new(config: ReconciliationConfig) -> Self {
        Self {
            config,
            reconciliation_counter: 0,
        }
    }

    /// Reconciles AR subledger to GL.
    pub fn reconcile_ar(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        gl_balance: Decimal,
        ar_invoices: &[&ARInvoice],
    ) -> ReconciliationResult {
        self.reconciliation_counter += 1;
        let reconciliation_id = format!("RECON-AR-{:08}", self.reconciliation_counter);

        let subledger_balance: Decimal = ar_invoices.iter().map(|inv| inv.amount_remaining).sum();

        let difference = gl_balance - subledger_balance;

        let mut unreconciled_items = Vec::new();

        // Check for timing differences or mismatches
        if difference.abs() >= self.config.tolerance_amount {
            // Find potential unreconciled items
            for invoice in ar_invoices {
                if invoice.posting_date > as_of_date {
                    unreconciled_items.push(UnreconciledEntry {
                        entry_type: "Timing Difference".to_string(),
                        document_number: invoice.invoice_number.clone(),
                        amount: invoice.amount_remaining,
                        description: format!(
                            "Invoice posted after reconciliation date: {}",
                            invoice.posting_date
                        ),
                    });
                }
            }
        }

        let status = if difference.abs() < self.config.tolerance_amount {
            ReconStatus::Reconciled
        } else if !unreconciled_items.is_empty() {
            ReconStatus::PartiallyReconciled
        } else {
            ReconStatus::Unreconciled
        };

        ReconciliationResult {
            reconciliation_id,
            company_code: company_code.to_string(),
            subledger_type: SubledgerType::AR,
            as_of_date,
            gl_account: self.config.ar_control_account.clone(),
            gl_balance,
            subledger_balance,
            difference,
            status,
            unreconciled_items,
            reconciliation_date: as_of_date,
            reconciled_by: None,
            notes: None,
        }
    }

    /// Reconciles AP subledger to GL.
    pub fn reconcile_ap(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        gl_balance: Decimal,
        ap_invoices: &[&APInvoice],
    ) -> ReconciliationResult {
        self.reconciliation_counter += 1;
        let reconciliation_id = format!("RECON-AP-{:08}", self.reconciliation_counter);

        let subledger_balance: Decimal = ap_invoices.iter().map(|inv| inv.amount_remaining).sum();

        let difference = gl_balance - subledger_balance;

        let mut unreconciled_items = Vec::new();

        if difference.abs() >= self.config.tolerance_amount {
            for invoice in ap_invoices {
                if invoice.posting_date > as_of_date {
                    unreconciled_items.push(UnreconciledEntry {
                        entry_type: "Timing Difference".to_string(),
                        document_number: invoice.invoice_number.clone(),
                        amount: invoice.amount_remaining,
                        description: format!(
                            "Invoice posted after reconciliation date: {}",
                            invoice.posting_date
                        ),
                    });
                }
            }
        }

        let status = if difference.abs() < self.config.tolerance_amount {
            ReconStatus::Reconciled
        } else if !unreconciled_items.is_empty() {
            ReconStatus::PartiallyReconciled
        } else {
            ReconStatus::Unreconciled
        };

        ReconciliationResult {
            reconciliation_id,
            company_code: company_code.to_string(),
            subledger_type: SubledgerType::AP,
            as_of_date,
            gl_account: self.config.ap_control_account.clone(),
            gl_balance,
            subledger_balance,
            difference,
            status,
            unreconciled_items,
            reconciliation_date: as_of_date,
            reconciled_by: None,
            notes: None,
        }
    }

    /// Reconciles FA subledger to GL.
    pub fn reconcile_fa(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        gl_asset_balance: Decimal,
        gl_accum_depr_balance: Decimal,
        assets: &[&FixedAssetRecord],
    ) -> (ReconciliationResult, ReconciliationResult) {
        // Asset reconciliation
        self.reconciliation_counter += 1;
        let asset_recon_id = format!("RECON-FA-{:08}", self.reconciliation_counter);

        let subledger_asset_balance: Decimal =
            assets.iter().map(|a| a.current_acquisition_cost()).sum();

        let asset_difference = gl_asset_balance - subledger_asset_balance;

        let asset_status = if asset_difference.abs() < self.config.tolerance_amount {
            ReconStatus::Reconciled
        } else {
            ReconStatus::Unreconciled
        };

        let asset_result = ReconciliationResult {
            reconciliation_id: asset_recon_id,
            company_code: company_code.to_string(),
            subledger_type: SubledgerType::FA,
            as_of_date,
            gl_account: self.config.fa_control_account.clone(),
            gl_balance: gl_asset_balance,
            subledger_balance: subledger_asset_balance,
            difference: asset_difference,
            status: asset_status,
            unreconciled_items: Vec::new(),
            reconciliation_date: as_of_date,
            reconciled_by: None,
            notes: Some("Fixed Asset - Acquisition Cost".to_string()),
        };

        // Accumulated depreciation reconciliation
        self.reconciliation_counter += 1;
        let depr_recon_id = format!("RECON-FA-{:08}", self.reconciliation_counter);

        let subledger_accum_depr: Decimal = assets.iter().map(|a| a.accumulated_depreciation).sum();

        let depr_difference = gl_accum_depr_balance - subledger_accum_depr;

        let depr_status = if depr_difference.abs() < self.config.tolerance_amount {
            ReconStatus::Reconciled
        } else {
            ReconStatus::Unreconciled
        };

        let depr_result = ReconciliationResult {
            reconciliation_id: depr_recon_id,
            company_code: company_code.to_string(),
            subledger_type: SubledgerType::FA,
            as_of_date,
            gl_account: format!("{}-ACCUM", self.config.fa_control_account),
            gl_balance: gl_accum_depr_balance,
            subledger_balance: subledger_accum_depr,
            difference: depr_difference,
            status: depr_status,
            unreconciled_items: Vec::new(),
            reconciliation_date: as_of_date,
            reconciled_by: None,
            notes: Some("Fixed Asset - Accumulated Depreciation".to_string()),
        };

        (asset_result, depr_result)
    }

    /// Reconciles inventory subledger to GL.
    pub fn reconcile_inventory(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        gl_balance: Decimal,
        positions: &[&InventoryPosition],
    ) -> ReconciliationResult {
        self.reconciliation_counter += 1;
        let reconciliation_id = format!("RECON-INV-{:08}", self.reconciliation_counter);

        let subledger_balance: Decimal = positions.iter().map(|p| p.valuation.total_value).sum();

        let difference = gl_balance - subledger_balance;

        let mut unreconciled_items = Vec::new();

        if difference.abs() >= self.config.tolerance_amount {
            // Check for positions with zero value but quantity
            for position in positions {
                if position.quantity_on_hand > Decimal::ZERO
                    && position.valuation.total_value == Decimal::ZERO
                {
                    unreconciled_items.push(UnreconciledEntry {
                        entry_type: "Valuation Issue".to_string(),
                        document_number: position.material_id.clone(),
                        amount: Decimal::ZERO,
                        description: format!(
                            "Material {} has quantity {} but zero value",
                            position.material_id, position.quantity_on_hand
                        ),
                    });
                }
            }
        }

        let status = if difference.abs() < self.config.tolerance_amount {
            ReconStatus::Reconciled
        } else if !unreconciled_items.is_empty() {
            ReconStatus::PartiallyReconciled
        } else {
            ReconStatus::Unreconciled
        };

        ReconciliationResult {
            reconciliation_id,
            company_code: company_code.to_string(),
            subledger_type: SubledgerType::Inventory,
            as_of_date,
            gl_account: self.config.inventory_control_account.clone(),
            gl_balance,
            subledger_balance,
            difference,
            status,
            unreconciled_items,
            reconciliation_date: as_of_date,
            reconciled_by: None,
            notes: None,
        }
    }

    /// Performs full reconciliation for all subledgers.
    pub fn full_reconciliation(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        gl_balances: &HashMap<String, Decimal>,
        ar_invoices: &[&ARInvoice],
        ap_invoices: &[&APInvoice],
        assets: &[&FixedAssetRecord],
        inventory_positions: &[&InventoryPosition],
    ) -> FullReconciliationReport {
        let ar_result = self.reconcile_ar(
            company_code,
            as_of_date,
            *gl_balances
                .get(&self.config.ar_control_account)
                .unwrap_or(&Decimal::ZERO),
            ar_invoices,
        );

        let ap_result = self.reconcile_ap(
            company_code,
            as_of_date,
            *gl_balances
                .get(&self.config.ap_control_account)
                .unwrap_or(&Decimal::ZERO),
            ap_invoices,
        );

        let fa_asset_balance = *gl_balances
            .get(&self.config.fa_control_account)
            .unwrap_or(&Decimal::ZERO);
        let fa_depr_balance = *gl_balances
            .get(&format!("{}-ACCUM", self.config.fa_control_account))
            .unwrap_or(&Decimal::ZERO);

        let (fa_asset_result, fa_depr_result) = self.reconcile_fa(
            company_code,
            as_of_date,
            fa_asset_balance,
            fa_depr_balance,
            assets,
        );

        let inventory_result = self.reconcile_inventory(
            company_code,
            as_of_date,
            *gl_balances
                .get(&self.config.inventory_control_account)
                .unwrap_or(&Decimal::ZERO),
            inventory_positions,
        );

        let all_reconciled = ar_result.is_balanced()
            && ap_result.is_balanced()
            && fa_asset_result.is_balanced()
            && fa_depr_result.is_balanced()
            && inventory_result.is_balanced();

        let total_difference = ar_result.difference.abs()
            + ap_result.difference.abs()
            + fa_asset_result.difference.abs()
            + fa_depr_result.difference.abs()
            + inventory_result.difference.abs();

        FullReconciliationReport {
            company_code: company_code.to_string(),
            as_of_date,
            ar: ar_result,
            ap: ap_result,
            fa_assets: fa_asset_result,
            fa_depreciation: fa_depr_result,
            inventory: inventory_result,
            all_reconciled,
            total_difference,
        }
    }
}

/// Full reconciliation report covering all subledgers.
#[derive(Debug, Clone)]
pub struct FullReconciliationReport {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// AR reconciliation result.
    pub ar: ReconciliationResult,
    /// AP reconciliation result.
    pub ap: ReconciliationResult,
    /// FA assets reconciliation result.
    pub fa_assets: ReconciliationResult,
    /// FA depreciation reconciliation result.
    pub fa_depreciation: ReconciliationResult,
    /// Inventory reconciliation result.
    pub inventory: ReconciliationResult,
    /// Whether all subledgers are reconciled.
    pub all_reconciled: bool,
    /// Total unreconciled difference across all subledgers.
    pub total_difference: Decimal,
}

impl FullReconciliationReport {
    /// Returns a summary of the reconciliation status.
    pub fn summary(&self) -> String {
        format!(
            "Reconciliation Report for {} as of {}\n\
             AR: {} (diff: {})\n\
             AP: {} (diff: {})\n\
             FA Assets: {} (diff: {})\n\
             FA Depreciation: {} (diff: {})\n\
             Inventory: {} (diff: {})\n\
             Overall: {} (total diff: {})",
            self.company_code,
            self.as_of_date,
            status_str(&self.ar.status),
            self.ar.difference,
            status_str(&self.ap.status),
            self.ap.difference,
            status_str(&self.fa_assets.status),
            self.fa_assets.difference,
            status_str(&self.fa_depreciation.status),
            self.fa_depreciation.difference,
            status_str(&self.inventory.status),
            self.inventory.difference,
            if self.all_reconciled {
                "RECONCILED"
            } else {
                "UNRECONCILED"
            },
            self.total_difference
        )
    }
}

fn status_str(status: &ReconStatus) -> &'static str {
    match status {
        ReconStatus::Reconciled => "RECONCILED",
        ReconStatus::Unreconciled => "UNRECONCILED",
        ReconStatus::PartiallyReconciled => "PARTIAL",
        ReconStatus::InProgress => "IN PROGRESS",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_balanced() {
        let result = ReconciliationResult {
            reconciliation_id: "TEST-001".to_string(),
            company_code: "1000".to_string(),
            subledger_type: SubledgerType::AR,
            as_of_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            gl_account: "1200".to_string(),
            gl_balance: dec!(10000),
            subledger_balance: dec!(10000),
            difference: Decimal::ZERO,
            status: ReconStatus::Reconciled,
            unreconciled_items: Vec::new(),
            reconciliation_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            reconciled_by: None,
            notes: None,
        };

        assert!(result.is_balanced());
    }

    #[test]
    fn test_reconciliation_unbalanced() {
        let result = ReconciliationResult {
            reconciliation_id: "TEST-002".to_string(),
            company_code: "1000".to_string(),
            subledger_type: SubledgerType::AR,
            as_of_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            gl_account: "1200".to_string(),
            gl_balance: dec!(10000),
            subledger_balance: dec!(9500),
            difference: dec!(500),
            status: ReconStatus::Unreconciled,
            unreconciled_items: Vec::new(),
            reconciliation_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            reconciled_by: None,
            notes: None,
        };

        assert!(!result.is_balanced());
    }
}
