//! RFx (Request for Information/Proposal/Quotation) models.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};

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

impl ToNodeProperties for RfxEvent {
    fn node_type_name(&self) -> &'static str {
        "rfx_event"
    }
    fn node_type_code(&self) -> u16 {
        321
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "rfxId".into(),
            GraphPropertyValue::String(self.rfx_id.clone()),
        );
        p.insert(
            "rfxType".into(),
            GraphPropertyValue::String(format!("{:?}", self.rfx_type)),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "title".into(),
            GraphPropertyValue::String(self.title.clone()),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "scoringMethod".into(),
            GraphPropertyValue::String(format!("{:?}", self.scoring_method)),
        );
        p.insert(
            "publishDate".into(),
            GraphPropertyValue::Date(self.publish_date),
        );
        p.insert(
            "responseDeadline".into(),
            GraphPropertyValue::Date(self.response_deadline),
        );
        p.insert(
            "bidCount".into(),
            GraphPropertyValue::Int(self.bid_count as i64),
        );
        p.insert(
            "invitedVendorCount".into(),
            GraphPropertyValue::Int(self.invited_vendors.len() as i64),
        );
        p.insert(
            "isAwarded".into(),
            GraphPropertyValue::Bool(matches!(self.status, RfxStatus::Awarded)),
        );
        p
    }
}
