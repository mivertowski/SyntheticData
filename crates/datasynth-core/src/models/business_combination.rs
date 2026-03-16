//! Business Combination models (IFRS 3 / ASC 805).
//!
//! Provides data structures for:
//! - Acquisition consideration (cash, shares, contingent)
//! - Purchase price allocation (PPA) with fair value adjustments
//! - Goodwill computation
//! - Day 1 journal entries and subsequent amortization of acquired intangibles

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A business combination transaction representing an acquisition accounted for
/// under IFRS 3 (acquisition method) or ASC 805.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessCombination {
    /// Unique identifier for this acquisition
    pub id: String,

    /// Company code of the acquirer entity
    pub acquirer_entity: String,

    /// Name of the acquiree (target company)
    pub acquiree_name: String,

    /// Date control was obtained (acquisition date)
    pub acquisition_date: NaiveDate,

    /// Total consideration paid or transferred
    pub consideration: AcquisitionConsideration,

    /// Purchase price allocation at acquisition date
    pub purchase_price_allocation: AcquisitionPpa,

    /// Goodwill recognised (consideration minus net identifiable assets at FV).
    /// Zero when consideration < net identifiable assets (bargain purchase).
    #[serde(with = "rust_decimal::serde::str")]
    pub goodwill: Decimal,

    /// Accounting framework applied: "IFRS" or "US_GAAP"
    pub framework: String,
}

/// Consideration transferred in a business combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionConsideration {
    /// Cash and cash equivalents paid
    #[serde(with = "rust_decimal::serde::str")]
    pub cash: Decimal,

    /// Fair value of equity instruments issued by the acquirer
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub shares_issued_value: Option<Decimal>,

    /// Fair value of contingent consideration (earn-out) at acquisition date
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub contingent_consideration: Option<Decimal>,

    /// Total consideration (sum of cash + shares + contingent)
    #[serde(with = "rust_decimal::serde::str")]
    pub total: Decimal,
}

/// Purchase price allocation mapping the consideration to identifiable
/// net assets at fair value, with the residual as goodwill.
///
/// Named `AcquisitionPpa` (not `PurchasePriceAllocation`) to avoid
/// a name collision with the same-named struct in `organizational_event`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionPpa {
    /// Identifiable assets acquired at fair value
    pub identifiable_assets: Vec<AcquisitionFvAdjustment>,

    /// Identifiable liabilities assumed at fair value
    pub identifiable_liabilities: Vec<AcquisitionFvAdjustment>,

    /// Net identifiable assets at fair value
    /// = sum(asset FVs) - sum(liability FVs)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_identifiable_assets_fv: Decimal,
}

/// A single asset or liability line within the purchase price allocation,
/// showing book value, fair value step-up, and useful life for intangibles.
///
/// Named `AcquisitionFvAdjustment` to avoid a name collision with
/// `FairValueAdjustment` in `organizational_event`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionFvAdjustment {
    /// Description of the asset or liability (e.g. "Customer Relationships")
    pub asset_or_liability: String,

    /// Carrying amount in the acquiree's books at acquisition date
    #[serde(with = "rust_decimal::serde::str")]
    pub book_value: Decimal,

    /// Fair value assigned in the PPA
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value: Decimal,

    /// Step-up amount (fair_value - book_value; may be negative for liabilities)
    #[serde(with = "rust_decimal::serde::str")]
    pub step_up: Decimal,

    /// Useful life in years for finite-lived intangibles; None for PP&E and indefinite-lived assets
    #[serde(default)]
    pub useful_life_years: Option<u32>,
}
