//! Audit sampling methodology models per ISA 530.
//!
//! ISA 530 (Audit Sampling) requires the auditor to design and perform audit
//! procedures to obtain sufficient appropriate audit evidence.  The sampling
//! plan documents the population, key items, methodology, and sample drawn.
//!
//! Key concepts:
//! - **Key items**: 100% tested; amounts ≥ tolerable error, unusual or high-risk items.
//! - **Representative sample**: drawn from the residual population using a statistical
//!   or non-statistical method proportional to the CRA level.
//! - **Monetary Unit Sampling (MUS)**: preferred for balance testing (existence,
//!   valuation) — each monetary unit has an equal probability of selection.
//! - **Systematic selection**: preferred for transaction testing — fixed interval
//!   with random start.
//!
//! References:
//! - ISA 530 — Audit Sampling
//! - ISA 315 — links CRA level to sampling extent
//! - ISA 320 / ISA 450 — tolerable error equals performance materiality

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Sampling methodology
// ---------------------------------------------------------------------------

/// Sampling methodology chosen for the plan per ISA 530.A5–A8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingMethodology {
    /// Monetary Unit Sampling (MUS) — each currency unit has equal selection probability.
    /// Preferred for balance testing (existence, valuation assertions).
    MonetaryUnitSampling,
    /// Simple random selection — each item has an equal probability of selection.
    RandomSelection,
    /// Systematic selection — fixed interval with a random start point.
    /// Preferred for transaction testing (occurrence, completeness assertions).
    SystematicSelection,
    /// Haphazard (non-statistical) selection — for low-risk areas where a formal
    /// statistical inference is not required (ISA 530.A8).
    HaphazardSelection,
}

impl std::fmt::Display for SamplingMethodology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MonetaryUnitSampling => "Monetary Unit Sampling (MUS)",
            Self::RandomSelection => "Random Selection",
            Self::SystematicSelection => "Systematic Selection",
            Self::HaphazardSelection => "Haphazard Selection",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Key items
// ---------------------------------------------------------------------------

/// Reason why an item was designated as a key item (100% tested).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyItemReason {
    /// Amount ≥ tolerable error — automatically selected per ISA 530.A14.
    AboveTolerableError,
    /// Unusual nature — related party, unusual counterparty, or non-routine transaction.
    UnusualNature,
    /// High-risk item originating from a significant risk area per ISA 315.28.
    HighRisk,
    /// Manual journal entry to an automated account — management override indicator.
    ManagementOverride,
}

impl std::fmt::Display for KeyItemReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AboveTolerableError => "Amount above tolerable error",
            Self::UnusualNature => "Unusual nature (related party / unusual counterparty)",
            Self::HighRisk => "High-risk area (significant risk per ISA 315.28)",
            Self::ManagementOverride => "Management override — manual JE to automated account",
        };
        write!(f, "{s}")
    }
}

/// A key item selected for 100% testing outside the representative sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyItem {
    /// Reference ID — JE document_id, subledger record ID, etc.
    pub item_id: String,
    /// Monetary amount of the item.
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Reason this item was designated as a key item.
    pub reason: KeyItemReason,
}

// ---------------------------------------------------------------------------
// Sampled items
// ---------------------------------------------------------------------------

/// Whether an item was selected as a key item or a representative sample item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionType {
    /// 100%-tested key item (above TE, unusual, high-risk, or management override).
    KeyItem,
    /// Representative sample item drawn from the residual population.
    Representative,
}

/// A single item selected into the audit sample (key item or representative).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampledItem {
    /// Reference ID — links to JE document_id, subledger record, etc.
    pub item_id: String,
    /// Monetary amount of the item.
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// How the item was selected into the sample.
    pub selection_type: SelectionType,
    /// Whether the auditor has completed testing on this item.
    pub tested: bool,
    /// Whether a misstatement was found during testing.
    pub misstatement_found: bool,
    /// Monetary amount of any misstatement identified (None if none found).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub misstatement_amount: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// Main sampling plan
// ---------------------------------------------------------------------------

/// Audit sampling plan for a single account area / assertion combination.
///
/// One plan is generated for each CRA at Moderate or High level, documenting
/// the full ISA 530-compliant sampling design and execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingPlan {
    /// Unique identifier for this sampling plan (deterministic slug).
    pub id: String,
    /// Entity / company code.
    pub entity_code: String,
    /// Account area being tested (e.g. "Trade Receivables", "Revenue").
    pub account_area: String,
    /// Financial statement assertion being tested (e.g. "Existence").
    pub assertion: String,
    /// Sampling methodology chosen for this plan.
    pub methodology: SamplingMethodology,
    /// Total number of items in the population before key item extraction.
    pub population_size: usize,
    /// Total monetary value of the population.
    #[serde(with = "rust_decimal::serde::str")]
    pub population_value: Decimal,
    /// Key items identified and extracted for 100% testing.
    pub key_items: Vec<KeyItem>,
    /// Total monetary value of all key items.
    #[serde(with = "rust_decimal::serde::str")]
    pub key_items_value: Decimal,
    /// Monetary value of the residual population (population_value − key_items_value).
    #[serde(with = "rust_decimal::serde::str")]
    pub remaining_population_value: Decimal,
    /// Number of representative sample items drawn from the residual population.
    pub sample_size: usize,
    /// Sampling interval = remaining_population_value / sample_size (for MUS / systematic).
    #[serde(with = "rust_decimal::serde::str")]
    pub sampling_interval: Decimal,
    /// CRA level that drove this plan (links to `CombinedRiskAssessment.combined_risk`).
    pub cra_level: String,
    /// Tolerable error for this population (equals performance materiality from ISA 320).
    #[serde(with = "rust_decimal::serde::str")]
    pub tolerable_error: Decimal,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn key_item_reason_display() {
        assert_eq!(
            KeyItemReason::AboveTolerableError.to_string(),
            "Amount above tolerable error"
        );
    }

    #[test]
    fn sampling_methodology_display() {
        assert_eq!(
            SamplingMethodology::MonetaryUnitSampling.to_string(),
            "Monetary Unit Sampling (MUS)"
        );
        assert_eq!(
            SamplingMethodology::SystematicSelection.to_string(),
            "Systematic Selection"
        );
    }

    #[test]
    fn sampling_plan_structure() {
        let plan = SamplingPlan {
            id: "SP-C001-TRADE_RECEIVABLES-Existence".into(),
            entity_code: "C001".into(),
            account_area: "Trade Receivables".into(),
            assertion: "Existence".into(),
            methodology: SamplingMethodology::MonetaryUnitSampling,
            population_size: 500,
            population_value: dec!(1_000_000),
            key_items: vec![KeyItem {
                item_id: "JE-001".into(),
                amount: dec!(50_000),
                reason: KeyItemReason::AboveTolerableError,
            }],
            key_items_value: dec!(50_000),
            remaining_population_value: dec!(950_000),
            sample_size: 25,
            sampling_interval: dec!(38_000),
            cra_level: "Moderate".into(),
            tolerable_error: dec!(32_500),
        };

        assert_eq!(plan.population_value - plan.key_items_value, plan.remaining_population_value);
        assert_eq!(plan.key_items.len(), 1);
        assert_eq!(plan.key_items[0].reason, KeyItemReason::AboveTolerableError);
    }
}
