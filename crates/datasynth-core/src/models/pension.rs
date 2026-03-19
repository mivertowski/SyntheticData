//! Pension models — IAS 19 / ASC 715.
//!
//! This module provides data models for defined benefit pension plans,
//! including actuarial assumptions, defined benefit obligation (DBO)
//! roll-forwards, plan asset roll-forwards, and pension disclosures.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Type of pension plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PensionPlanType {
    /// Traditional defined benefit — benefit formula-driven obligation.
    #[default]
    DefinedBenefit,
    /// Hybrid cash balance — lump sum account with interest credits.
    HybridCashBalance,
}

// ---------------------------------------------------------------------------
// Actuarial Assumptions
// ---------------------------------------------------------------------------

/// Actuarial assumptions used to measure the defined benefit obligation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuarialAssumptions {
    /// Discount rate used to present-value future benefit payments (e.g. 0.04 = 4%).
    #[serde(with = "rust_decimal::serde::str")]
    pub discount_rate: Decimal,
    /// Expected annual salary growth rate (e.g. 0.03 = 3%).
    #[serde(with = "rust_decimal::serde::str")]
    pub salary_growth_rate: Decimal,
    /// Expected annual pension increase rate post-retirement (e.g. 0.02 = 2%).
    #[serde(with = "rust_decimal::serde::str")]
    pub pension_increase_rate: Decimal,
    /// Long-term expected return on plan assets (e.g. 0.06 = 6%).
    #[serde(with = "rust_decimal::serde::str")]
    pub expected_return_on_plan_assets: Decimal,
}

// ---------------------------------------------------------------------------
// Defined Benefit Plan
// ---------------------------------------------------------------------------

/// A defined benefit pension plan sponsored by an entity.
///
/// One plan is generated per reporting entity.  The plan references its
/// obligation and asset roll-forwards via `plan_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinedBenefitPlan {
    /// Unique plan identifier (e.g. "PLAN-1000-DB").
    pub id: String,
    /// Company / entity code that sponsors this plan.
    pub entity_code: String,
    /// Human-readable plan name (e.g. "Acme Corp Retirement Plan").
    pub plan_name: String,
    /// Plan type — defined benefit or hybrid cash balance.
    pub plan_type: PensionPlanType,
    /// Number of active plan participants (employees enrolled).
    pub participant_count: u32,
    /// Actuarial assumptions used for valuation.
    pub assumptions: ActuarialAssumptions,
    /// Reporting currency code (e.g. "USD").
    pub currency: String,
}

// ---------------------------------------------------------------------------
// DBO Roll-forward
// ---------------------------------------------------------------------------

/// Defined Benefit Obligation (DBO) roll-forward for one reporting period.
///
/// Reconciles opening to closing DBO per IAS 19.140 / ASC 715-20-50.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PensionObligation {
    /// Reference to the parent `DefinedBenefitPlan.id`.
    pub plan_id: String,
    /// Period label (e.g. "2024-01" or "FY2024").
    pub period: String,
    /// DBO at start of period.
    #[serde(with = "rust_decimal::serde::str")]
    pub dbo_opening: Decimal,
    /// Current service cost — present value of benefits earned by employees
    /// during the current period.
    #[serde(with = "rust_decimal::serde::str")]
    pub service_cost: Decimal,
    /// Interest cost — unwinding of the discount on the obligation
    /// (`dbo_opening × discount_rate`).
    #[serde(with = "rust_decimal::serde::str")]
    pub interest_cost: Decimal,
    /// Actuarial gains (negative) or losses (positive) arising from
    /// changes in assumptions or experience adjustments.
    #[serde(with = "rust_decimal::serde::str")]
    pub actuarial_gains_losses: Decimal,
    /// Benefits paid to retirees during the period (reduces DBO).
    #[serde(with = "rust_decimal::serde::str")]
    pub benefits_paid: Decimal,
    /// DBO at end of period.
    /// Identity: `dbo_opening + service_cost + interest_cost + actuarial_gains_losses − benefits_paid`
    #[serde(with = "rust_decimal::serde::str")]
    pub dbo_closing: Decimal,
}

// ---------------------------------------------------------------------------
// Plan Assets Roll-forward
// ---------------------------------------------------------------------------

/// Plan assets roll-forward for one reporting period.
///
/// Tracks the fair value of assets held in trust to fund pension obligations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAssets {
    /// Reference to the parent `DefinedBenefitPlan.id`.
    pub plan_id: String,
    /// Period label.
    pub period: String,
    /// Fair value of plan assets at start of period.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_opening: Decimal,
    /// Expected return on plan assets
    /// (`fair_value_opening × expected_return_on_plan_assets`).
    #[serde(with = "rust_decimal::serde::str")]
    pub expected_return: Decimal,
    /// Actuarial gain (positive) or loss (negative) on plan assets
    /// (actual return vs. expected return).
    #[serde(with = "rust_decimal::serde::str")]
    pub actuarial_gain_loss: Decimal,
    /// Employer contributions paid into the plan trust.
    #[serde(with = "rust_decimal::serde::str")]
    pub employer_contributions: Decimal,
    /// Benefits paid out of the plan trust during the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub benefits_paid: Decimal,
    /// Fair value of plan assets at end of period.
    /// Identity: `fair_value_opening + expected_return + actuarial_gain_loss + employer_contributions − benefits_paid`
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_closing: Decimal,
}

// ---------------------------------------------------------------------------
// Pension Disclosure
// ---------------------------------------------------------------------------

/// Summary pension disclosure amounts for a reporting period.
///
/// Provides the key balance-sheet and income-statement figures required by
/// IAS 19 / ASC 715 disclosures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PensionDisclosure {
    /// Reference to the parent `DefinedBenefitPlan.id`.
    pub plan_id: String,
    /// Period label.
    pub period: String,
    /// Net pension liability recognised on the balance sheet
    /// (`dbo_closing − fair_value_closing`).
    /// Positive = under-funded (liability); negative = over-funded (asset).
    #[serde(with = "rust_decimal::serde::str")]
    pub net_pension_liability: Decimal,
    /// Total pension expense recognised in profit or loss
    /// (`service_cost + interest_cost − expected_return`).
    #[serde(with = "rust_decimal::serde::str")]
    pub pension_expense: Decimal,
    /// Remeasurements recognised in Other Comprehensive Income (OCI).
    /// Combines obligation actuarial gains/losses and plan asset actuarial gains/losses.
    /// Negative = gain in OCI; positive = loss recognised in OCI.
    #[serde(with = "rust_decimal::serde::str")]
    pub oci_remeasurements: Decimal,
    /// Funding ratio: `fair_value_closing / dbo_closing` (expressed as a decimal, e.g. 0.95).
    /// Zero when DBO is zero.
    #[serde(with = "rust_decimal::serde::str")]
    pub funding_ratio: Decimal,
}
