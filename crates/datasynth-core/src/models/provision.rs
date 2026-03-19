//! Provisions and contingencies models — IAS 37 / ASC 450.
//!
//! This module provides data models for provisions (recognised liabilities of
//! uncertain timing or amount), contingent liabilities (disclosed but not
//! recognised), and provision movement roll-forwards.
//!
//! # Framework differences
//!
//! | Criterion          | IAS 37 (IFRS)              | ASC 450 (US GAAP)          |
//! |--------------------|----------------------------|----------------------------|
//! | Recognition        | Probable (>50%)            | Probable (>75%)            |
//! | Measurement        | Best estimate              | Lower end of range          |
//! | Discounting        | Required when material     | Permitted                   |
//! | Contingent liab.   | Possible — disclose only   | Possible — disclose only    |
//! | Remote             | No disclosure required     | No disclosure required      |

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Category of provision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionType {
    /// Product or service warranty obligations.
    Warranty,
    /// Restructuring costs (redundancy, lease termination, onerous obligations).
    Restructuring,
    /// Pending or threatened litigation claims.
    LegalClaim,
    /// Environmental clean-up or remediation obligations.
    EnvironmentalRemediation,
    /// Contracts where unavoidable costs exceed expected economic benefits.
    OnerousContract,
    /// Asset retirement / decommissioning obligations.
    Decommissioning,
}

impl std::fmt::Display for ProvisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Warranty => "Warranty",
            Self::Restructuring => "Restructuring",
            Self::LegalClaim => "Legal Claim",
            Self::EnvironmentalRemediation => "Environmental Remediation",
            Self::OnerousContract => "Onerous Contract",
            Self::Decommissioning => "Decommissioning",
        };
        write!(f, "{s}")
    }
}

/// Probability level for contingent items.
///
/// Drives both recognition (provisions) and disclosure (contingent liabilities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContingentProbability {
    /// Remote — no disclosure required (<10%).
    Remote,
    /// Possible — disclose but do not recognise (10%–50% under IFRS; 10%–75% under US GAAP).
    Possible,
    /// Probable — recognise as provision (>50% under IFRS; >75% under US GAAP).
    Probable,
}

// ---------------------------------------------------------------------------
// Provision
// ---------------------------------------------------------------------------

/// A recognised provision per IAS 37 / ASC 450.
///
/// A provision is recognised when there is a present obligation, an outflow of
/// resources is probable, and a reliable estimate can be made.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provision {
    /// Unique provision identifier.
    pub id: String,
    /// Company / entity code.
    pub entity_code: String,
    /// Provision category.
    pub provision_type: ProvisionType,
    /// Description of the obligation (e.g. "Product warranty — FY2024 sales").
    pub description: String,
    /// Best estimate of the expenditure required to settle the obligation.
    #[serde(with = "rust_decimal::serde::str")]
    pub best_estimate: Decimal,
    /// Lower end of the estimated range.
    #[serde(with = "rust_decimal::serde::str")]
    pub range_low: Decimal,
    /// Upper end of the estimated range.
    #[serde(with = "rust_decimal::serde::str")]
    pub range_high: Decimal,
    /// Discount rate applied to long-term provisions (e.g. 0.04 = 4%).
    /// `None` for provisions expected to be settled within 12 months.
    #[serde(
        with = "rust_decimal::serde::str_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub discount_rate: Option<Decimal>,
    /// Expected date of cash outflow / settlement.
    pub expected_utilization_date: NaiveDate,
    /// Accounting framework governing recognition: `"IFRS"` or `"US_GAAP"`.
    pub framework: String,
    /// Reporting currency code.
    pub currency: String,
}

// ---------------------------------------------------------------------------
// Contingent Liability
// ---------------------------------------------------------------------------

/// A contingent liability disclosed in the notes per IAS 37.86 / ASC 450-20-50.
///
/// Contingent liabilities are **not** recognised on the balance sheet but are
/// disclosed when the probability is Possible (or Probable with uncertain amount).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingentLiability {
    /// Unique identifier.
    pub id: String,
    /// Company / entity code.
    pub entity_code: String,
    /// Nature of the contingency (e.g. "Pending patent infringement lawsuit").
    pub nature: String,
    /// Assessed probability level.
    pub probability: ContingentProbability,
    /// Best estimate of the potential exposure (if determinable).
    #[serde(
        with = "rust_decimal::serde::str_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub estimated_amount: Option<Decimal>,
    /// Whether this item requires disclosure only (true) or could be recognised (false).
    ///
    /// Always `true` for Possible items; `false` would indicate the entity is
    /// still evaluating whether recognition criteria are met.
    pub disclosure_only: bool,
    /// Reporting currency code.
    pub currency: String,
}

// ---------------------------------------------------------------------------
// Provision Movement
// ---------------------------------------------------------------------------

/// Roll-forward of a provision balance for one reporting period.
///
/// Identity: `opening + additions − utilizations − reversals + unwinding_of_discount = closing`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionMovement {
    /// Reference to the parent `Provision.id`.
    pub provision_id: String,
    /// Period label (e.g. "2024-Q4" or "FY2024").
    pub period: String,
    /// Provision balance at start of period.
    #[serde(with = "rust_decimal::serde::str")]
    pub opening: Decimal,
    /// New provisions recognised during the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub additions: Decimal,
    /// Amounts utilised (actual cash payments) during the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub utilizations: Decimal,
    /// Provisions reversed (no longer required) during the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub reversals: Decimal,
    /// Unwinding of discount on long-term provisions (finance cost).
    #[serde(with = "rust_decimal::serde::str")]
    pub unwinding_of_discount: Decimal,
    /// Provision balance at end of period.
    /// `opening + additions − utilizations − reversals + unwinding_of_discount`
    #[serde(with = "rust_decimal::serde::str")]
    pub closing: Decimal,
}
