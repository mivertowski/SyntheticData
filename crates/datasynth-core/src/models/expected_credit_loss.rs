//! Expected Credit Loss (ECL) models — IFRS 9 / ASC 326.
//!
//! This module provides data structures for the simplified approach ECL model
//! applied to trade receivables via a provision matrix based on AR aging.
//!
//! Key IFRS 9 / ASC 326 concepts modelled:
//! - Simplified approach (trade receivables): lifetime ECL at all times
//! - Provision matrix: historical loss rates by aging bucket + forward-looking adjustment
//! - ECL = Exposure × PD × LGD (Stage 1/2/3 for completeness)
//! - Provision movement: opening → new originations → stage transfers → write-offs → closing

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::subledger::ar::AgingBucket;

// ============================================================================
// Enums
// ============================================================================

/// ECL measurement approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EclApproach {
    /// Simplified approach — lifetime ECL at all times (used for trade receivables).
    Simplified,
    /// General approach — 3-stage model based on credit deterioration.
    General,
}

/// IFRS 9 / ASC 326 stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EclStage {
    /// Stage 1 — performing; 12-month ECL.
    Stage1Month12,
    /// Stage 2 — significant credit deterioration; lifetime ECL.
    Stage2Lifetime,
    /// Stage 3 — credit-impaired; lifetime ECL, interest on net carrying amount.
    Stage3CreditImpaired,
}

// ============================================================================
// Core model structs
// ============================================================================

/// Top-level ECL model for one entity / measurement date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclModel {
    /// Unique model ID.
    pub id: String,

    /// Entity (company) code.
    pub entity_code: String,

    /// Measurement approach.
    pub approach: EclApproach,

    /// Measurement date (balance-sheet date).
    pub measurement_date: NaiveDate,

    /// Accounting framework ("IFRS_9" or "ASC_326").
    pub framework: String,

    /// Portfolio segments within this model.
    pub portfolio_segments: Vec<EclPortfolioSegment>,

    /// Provision matrix (simplified approach only).
    pub provision_matrix: Option<ProvisionMatrix>,

    /// Total ECL across all segments.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_ecl: Decimal,

    /// Total gross exposure.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_exposure: Decimal,
}

/// A portfolio segment within the ECL model (e.g. "Trade Receivables", "Intercompany").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclPortfolioSegment {
    /// Segment name.
    pub segment_name: String,

    /// Gross exposure at the measurement date.
    #[serde(with = "rust_decimal::serde::str")]
    pub exposure_at_default: Decimal,

    /// Stage allocations within this segment.
    pub staging: Vec<EclStageAllocation>,

    /// Total ECL for this segment (sum of stage ECLs).
    #[serde(with = "rust_decimal::serde::str")]
    pub total_ecl: Decimal,
}

/// ECL split by IFRS 9 stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclStageAllocation {
    /// IFRS 9 / ASC 326 stage.
    pub stage: EclStage,

    /// Gross exposure in this stage.
    #[serde(with = "rust_decimal::serde::str")]
    pub exposure: Decimal,

    /// Probability of default (0–1).
    #[serde(with = "rust_decimal::serde::str")]
    pub probability_of_default: Decimal,

    /// Loss given default (0–1).
    #[serde(with = "rust_decimal::serde::str")]
    pub loss_given_default: Decimal,

    /// Computed ECL = exposure × PD × LGD × forward_looking_adjustment.
    #[serde(with = "rust_decimal::serde::str")]
    pub ecl_amount: Decimal,

    /// Forward-looking multiplier applied to historical rate (1.0 = no adjustment).
    #[serde(with = "rust_decimal::serde::str")]
    pub forward_looking_adjustment: Decimal,
}

// ============================================================================
// Provision matrix
// ============================================================================

/// Provision matrix for the simplified approach — one row per aging bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionMatrix {
    /// Entity code.
    pub entity_code: String,

    /// Measurement date.
    pub measurement_date: NaiveDate,

    /// Scenario weights used for forward-looking adjustment.
    pub scenario_weights: ScenarioWeights,

    /// One row per AR aging bucket.
    pub aging_buckets: Vec<ProvisionMatrixRow>,

    /// Sum of all provisions across all buckets.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_provision: Decimal,

    /// Sum of all exposures across all buckets.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_exposure: Decimal,

    /// Blended loss rate = total_provision / total_exposure.
    #[serde(with = "rust_decimal::serde::str")]
    pub blended_loss_rate: Decimal,
}

/// Scenario weights for forward-looking macro adjustment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioWeights {
    /// Weight for base scenario.
    #[serde(with = "rust_decimal::serde::str")]
    pub base: Decimal,

    /// Multiplier applied to historical rates under base scenario (typically 1.0).
    #[serde(with = "rust_decimal::serde::str")]
    pub base_multiplier: Decimal,

    /// Weight for optimistic scenario.
    #[serde(with = "rust_decimal::serde::str")]
    pub optimistic: Decimal,

    /// Multiplier applied to historical rates under optimistic scenario (< 1.0).
    #[serde(with = "rust_decimal::serde::str")]
    pub optimistic_multiplier: Decimal,

    /// Weight for pessimistic scenario.
    #[serde(with = "rust_decimal::serde::str")]
    pub pessimistic: Decimal,

    /// Multiplier applied to historical rates under pessimistic scenario (> 1.0).
    #[serde(with = "rust_decimal::serde::str")]
    pub pessimistic_multiplier: Decimal,

    /// Resulting blended forward-looking multiplier
    /// = base*base_m + optimistic*opt_m + pessimistic*pes_m.
    #[serde(with = "rust_decimal::serde::str")]
    pub blended_multiplier: Decimal,
}

/// One row of the provision matrix, corresponding to an AR aging bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionMatrixRow {
    /// Aging bucket this row covers.
    pub bucket: AgingBucket,

    /// Historical loss rate for this bucket (e.g. 0.005 = 0.5%).
    #[serde(with = "rust_decimal::serde::str")]
    pub historical_loss_rate: Decimal,

    /// Forward-looking adjustment multiplier (scenario-weighted).
    #[serde(with = "rust_decimal::serde::str")]
    pub forward_looking_adjustment: Decimal,

    /// Applied loss rate = historical_loss_rate × forward_looking_adjustment.
    #[serde(with = "rust_decimal::serde::str")]
    pub applied_loss_rate: Decimal,

    /// Gross exposure in this bucket.
    #[serde(with = "rust_decimal::serde::str")]
    pub exposure: Decimal,

    /// Provision = exposure × applied_loss_rate.
    #[serde(with = "rust_decimal::serde::str")]
    pub provision: Decimal,
}

// ============================================================================
// Provision movement
// ============================================================================

/// Provision movement (roll-forward) for one fiscal period.
///
/// Reconciles the opening and closing allowance for doubtful accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EclProvisionMovement {
    /// Unique movement record ID.
    pub id: String,

    /// Entity code.
    pub entity_code: String,

    /// Fiscal period label (e.g. "2024-Q1", "2024-12").
    pub period: String,

    /// Opening allowance balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub opening: Decimal,

    /// New originations charged to P&L (increase in allowance).
    #[serde(with = "rust_decimal::serde::str")]
    pub new_originations: Decimal,

    /// Stage-transfer adjustments (positive = provision increase).
    #[serde(with = "rust_decimal::serde::str")]
    pub stage_transfers: Decimal,

    /// Write-offs charged against the allowance (reduces allowance balance).
    #[serde(with = "rust_decimal::serde::str")]
    pub write_offs: Decimal,

    /// Cash recoveries on previously written-off receivables (increases allowance).
    #[serde(with = "rust_decimal::serde::str")]
    pub recoveries: Decimal,

    /// Closing allowance = opening + new_originations + stage_transfers - write_offs + recoveries.
    #[serde(with = "rust_decimal::serde::str")]
    pub closing: Decimal,

    /// P&L charge for the period = new_originations + stage_transfers + recoveries - write_offs.
    #[serde(with = "rust_decimal::serde::str")]
    pub pl_charge: Decimal,
}
