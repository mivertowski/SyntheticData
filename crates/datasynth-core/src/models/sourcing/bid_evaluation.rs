//! Bid evaluation and award recommendation models.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// An individual criterion score entry for a bid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidEvaluationEntry {
    /// Criterion name
    pub criterion_name: String,
    /// Raw score (0-100)
    pub raw_score: f64,
    /// Weight from RFx criteria
    pub weight: f64,
    /// Weighted score
    pub weighted_score: f64,
}

/// A ranked bid with overall scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedBid {
    /// Bid ID
    pub bid_id: String,
    /// Vendor ID
    pub vendor_id: String,
    /// Rank (1 = best)
    pub rank: u32,
    /// Total weighted score
    pub total_score: f64,
    /// Price score component
    pub price_score: f64,
    /// Technical/quality score component
    pub quality_score: f64,
    /// Total bid amount
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Individual criterion scores
    pub criterion_scores: Vec<BidEvaluationEntry>,
}

/// Award recommendation from bid evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AwardRecommendation {
    /// Award to this vendor
    Award,
    /// Consider as backup
    Backup,
    /// Do not award
    Reject,
}

/// Bid evaluation for an RFx event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidEvaluation {
    /// Unique evaluation ID
    pub evaluation_id: String,
    /// RFx event ID
    pub rfx_id: String,
    /// Company code
    pub company_code: String,
    /// Evaluator ID
    pub evaluator_id: String,
    /// Ranked bids (sorted by total_score descending)
    pub ranked_bids: Vec<RankedBid>,
    /// Recommended award
    pub recommendation: AwardRecommendation,
    /// Recommended vendor ID
    pub recommended_vendor_id: Option<String>,
    /// Recommended bid ID
    pub recommended_bid_id: Option<String>,
    /// Evaluation notes
    pub notes: Option<String>,
    /// Is the evaluation finalized
    pub is_finalized: bool,
}

impl ToNodeProperties for BidEvaluation {
    fn node_type_name(&self) -> &'static str {
        "bid_evaluation"
    }
    fn node_type_code(&self) -> u16 {
        323
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "evaluationId".into(),
            GraphPropertyValue::String(self.evaluation_id.clone()),
        );
        p.insert(
            "rfxId".into(),
            GraphPropertyValue::String(self.rfx_id.clone()),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "evaluatorId".into(),
            GraphPropertyValue::String(self.evaluator_id.clone()),
        );
        p.insert(
            "recommendation".into(),
            GraphPropertyValue::String(format!("{:?}", self.recommendation)),
        );
        p.insert(
            "bidCount".into(),
            GraphPropertyValue::Int(self.ranked_bids.len() as i64),
        );
        if let Some(top) = self.ranked_bids.first() {
            p.insert(
                "topBidScore".into(),
                GraphPropertyValue::Float(top.total_score),
            );
            p.insert(
                "topBidAmount".into(),
                GraphPropertyValue::Decimal(top.total_amount),
            );
        }
        if let Some(ref vendor_id) = self.recommended_vendor_id {
            p.insert(
                "recommendedVendorId".into(),
                GraphPropertyValue::String(vendor_id.clone()),
            );
        }
        p.insert(
            "isFinalized".into(),
            GraphPropertyValue::Bool(self.is_finalized),
        );
        p
    }
}
