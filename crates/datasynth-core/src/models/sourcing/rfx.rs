//! RFx (Request for Information/Proposal/Quotation) models.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of RFx event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RfxType {
    /// Request for Information (discovery)
    Rfi,
    /// Request for Proposal (complex requirements)
    #[default]
    Rfp,
    /// Request for Quotation (price-focused)
    Rfq,
}

/// Status of an RFx event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RfxStatus {
    /// RFx created, not yet published
    #[default]
    Draft,
    /// Published and open for responses
    Published,
    /// Response period ended, under evaluation
    Closed,
    /// Evaluation complete, winner selected
    Awarded,
    /// RFx cancelled
    Cancelled,
    /// No suitable bids received
    NoAward,
}

/// Scoring method for bid evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScoringMethod {
    /// Lowest price wins
    LowestPrice,
    /// Best value (weighted price + quality)
    #[default]
    BestValue,
    /// Quality-based selection (price secondary)
    QualityBased,
}

/// Evaluation criterion for an RFx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxEvaluationCriterion {
    /// Criterion name
    pub name: String,
    /// Weight (0.0 to 1.0, all criteria sum to 1.0)
    pub weight: f64,
    /// Description of what is being evaluated
    pub description: String,
}

/// Line item within an RFx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxLineItem {
    /// Item number
    pub item_number: u16,
    /// Material/service description
    pub description: String,
    /// Material ID (if applicable)
    pub material_id: Option<String>,
    /// Required quantity
    pub quantity: Decimal,
    /// Unit of measure
    pub uom: String,
    /// Target unit price (budget)
    pub target_price: Option<Decimal>,
}

/// An RFx (Request for x) event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxEvent {
    /// Unique RFx identifier
    pub rfx_id: String,
    /// RFx type (RFI/RFP/RFQ)
    pub rfx_type: RfxType,
    /// Company code
    pub company_code: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Current status
    pub status: RfxStatus,
    /// Sourcing project ID
    pub sourcing_project_id: String,
    /// Spend category
    pub category_id: String,
    /// Scoring method
    pub scoring_method: ScoringMethod,
    /// Evaluation criteria
    pub criteria: Vec<RfxEvaluationCriterion>,
    /// Line items
    pub line_items: Vec<RfxLineItem>,
    /// Invited vendor IDs
    pub invited_vendors: Vec<String>,
    /// Publication date
    pub publish_date: NaiveDate,
    /// Response deadline
    pub response_deadline: NaiveDate,
    /// Number of bids received
    pub bid_count: u32,
    /// Owner (sourcing manager)
    pub owner_id: String,
    /// Awarded vendor ID
    pub awarded_vendor_id: Option<String>,
    /// Awarded bid ID
    pub awarded_bid_id: Option<String>,
}
