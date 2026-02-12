//! Cycle count models for warehouse inventory management.
//!
//! These models represent cycle counting activities used to verify
//! inventory accuracy without performing a full physical inventory.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a cycle count through the counting lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleCountStatus {
    /// Count has been planned but not yet started
    #[default]
    Planned,
    /// Count is currently in progress
    InProgress,
    /// Physical count has been completed
    Counted,
    /// Variances have been investigated and reconciled
    Reconciled,
    /// Count has been closed and adjustments posted
    Closed,
}

/// Classification of count variance severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CountVarianceType {
    /// No variance between book and counted quantities
    #[default]
    None,
    /// Minor variance within acceptable tolerance
    Minor,
    /// Major variance requiring investigation
    Major,
    /// Critical variance requiring immediate action
    Critical,
}

/// A cycle count event covering one or more inventory items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCount {
    /// Unique cycle count identifier
    pub count_id: String,
    /// Company code this count belongs to
    pub company_code: String,
    /// Warehouse where the count takes place
    pub warehouse_id: String,
    /// Date the count is performed
    pub count_date: NaiveDate,
    /// Current status of the cycle count
    pub status: CycleCountStatus,
    /// Employee performing the count
    pub counter_id: Option<String>,
    /// Supervisor overseeing the count
    pub supervisor_id: Option<String>,
    /// Individual items counted
    pub items: Vec<CycleCountItem>,
    /// Total number of items counted
    pub total_items_counted: u32,
    /// Total number of items with variances
    pub total_variances: u32,
    /// Overall variance rate (total_variances / total_items_counted)
    pub variance_rate: f64,
}

/// A single item within a cycle count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCountItem {
    /// Material being counted
    pub material_id: String,
    /// Storage location within the warehouse
    pub storage_location: String,
    /// Quantity recorded in the system
    #[serde(with = "rust_decimal::serde::str")]
    pub book_quantity: Decimal,
    /// Quantity physically counted
    #[serde(with = "rust_decimal::serde::str")]
    pub counted_quantity: Decimal,
    /// Difference between counted and book quantities
    #[serde(with = "rust_decimal::serde::str")]
    pub variance_quantity: Decimal,
    /// Unit cost of the material
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_cost: Decimal,
    /// Monetary value of the variance
    #[serde(with = "rust_decimal::serde::str")]
    pub variance_value: Decimal,
    /// Classification of variance severity
    pub variance_type: CountVarianceType,
    /// Whether an inventory adjustment has been posted
    pub adjusted: bool,
    /// Reason for the adjustment (if adjusted)
    pub adjustment_reason: Option<String>,
}
