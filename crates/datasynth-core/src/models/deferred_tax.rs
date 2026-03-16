//! Deferred tax models (IAS 12 / ASC 740).
//!
//! This module provides data models for:
//! - Temporary differences (book vs. tax basis) that give rise to DTA/DTL
//! - ETR (effective tax rate) reconciliation from statutory to effective rate
//! - Deferred tax rollforward schedules tracking opening/closing DTA and DTL

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Whether a temporary difference gives rise to a deferred tax asset or liability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferredTaxType {
    /// Deferred Tax Asset – book basis exceeds tax basis (e.g. accruals, bad debt).
    Asset,
    /// Deferred Tax Liability – tax basis exceeds book basis (e.g. accelerated depreciation).
    Liability,
}

// ---------------------------------------------------------------------------
// Temporary Difference
// ---------------------------------------------------------------------------

/// A single temporary difference between book (GAAP/IFRS) and tax bases.
///
/// Under IAS 12 / ASC 740 a temporary difference arises when the carrying
/// amount of an asset or liability differs from its tax base.  The deferred
/// tax effect equals `difference × statutory_rate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryDifference {
    /// Unique identifier for this temporary difference record.
    pub id: String,
    /// Company / entity code this difference relates to.
    pub entity_code: String,
    /// GL account code associated with the underlying asset or liability.
    pub account: String,
    /// Human-readable description (e.g. "Accelerated depreciation – MACRS").
    pub description: String,
    /// Book (GAAP/IFRS) carrying amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub book_basis: Decimal,
    /// Tax basis of the same asset or liability.
    #[serde(with = "rust_decimal::serde::str")]
    pub tax_basis: Decimal,
    /// `book_basis − tax_basis`; positive = DTA, negative = DTL (before type override).
    #[serde(with = "rust_decimal::serde::str")]
    pub difference: Decimal,
    /// Whether this difference yields a DTA or DTL.
    pub deferred_type: DeferredTaxType,
    /// Accounting standard that created this difference (e.g. "ASC 842", "IAS 16").
    pub originating_standard: Option<String>,
}

// ---------------------------------------------------------------------------
// ETR Reconciliation
// ---------------------------------------------------------------------------

/// A single permanent difference item in the ETR reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentDifference {
    /// Description of the permanent difference (e.g. "Meals & entertainment (50% disallowed)").
    pub description: String,
    /// Pre-tax amount of the difference (positive = adds to taxable income).
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Tax effect = `amount × statutory_rate` (positive = increases tax expense).
    #[serde(with = "rust_decimal::serde::str")]
    pub tax_effect: Decimal,
}

/// Effective tax rate reconciliation for a reporting period.
///
/// Bridges from the statutory rate to the effective rate by listing all
/// permanent differences that cause the two rates to diverge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRateReconciliation {
    /// Company / entity code.
    pub entity_code: String,
    /// Period label (e.g. "FY2024", "2024-Q4").
    pub period: String,
    /// Pre-tax income (profit before income tax).
    #[serde(with = "rust_decimal::serde::str")]
    pub pre_tax_income: Decimal,
    /// Statutory (nominal) corporate income tax rate.
    #[serde(with = "rust_decimal::serde::str")]
    pub statutory_rate: Decimal,
    /// `pre_tax_income × statutory_rate` (expected tax at statutory rate).
    #[serde(with = "rust_decimal::serde::str")]
    pub expected_tax: Decimal,
    /// Permanent differences that bridge expected → actual tax.
    pub permanent_differences: Vec<PermanentDifference>,
    /// `actual_tax / pre_tax_income`.
    #[serde(with = "rust_decimal::serde::str")]
    pub effective_rate: Decimal,
    /// Actual income tax expense for the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_tax: Decimal,
}

// ---------------------------------------------------------------------------
// Deferred Tax Rollforward
// ---------------------------------------------------------------------------

/// Period-over-period rollforward of deferred tax asset and liability balances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredTaxRollforward {
    /// Company / entity code.
    pub entity_code: String,
    /// Period label.
    pub period: String,
    /// Opening Deferred Tax Asset balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub opening_dta: Decimal,
    /// Opening Deferred Tax Liability balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub opening_dtl: Decimal,
    /// Net movement during the period (DTA creation less reversal, net of DTL movement).
    #[serde(with = "rust_decimal::serde::str")]
    pub current_year_movement: Decimal,
    /// Closing Deferred Tax Asset balance (`opening_dta + dta_movement`).
    #[serde(with = "rust_decimal::serde::str")]
    pub closing_dta: Decimal,
    /// Closing Deferred Tax Liability balance (`opening_dtl + dtl_movement`).
    #[serde(with = "rust_decimal::serde::str")]
    pub closing_dtl: Decimal,
}
