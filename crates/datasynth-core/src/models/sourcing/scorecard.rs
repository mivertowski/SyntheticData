//! Supplier scorecard models for vendor performance tracking.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Trend direction for scorecard metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreboardTrend {
    /// Improving performance
    Improving,
    /// Stable performance
    Stable,
    /// Declining performance
    Declining,
}

/// Recommendation from scorecard review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorecardRecommendation {
    /// Maintain current relationship
    Maintain,
    /// Expand relationship (more categories/volume)
    Expand,
    /// Place on probation with improvement plan
    Probation,
    /// Initiate replacement sourcing
    Replace,
}

/// Contract compliance metrics within a scorecard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractComplianceMetrics {
    /// Contract ID
    pub contract_id: String,
    /// Contract utilization (consumed / total value)
    pub utilization_pct: f64,
    /// Number of SLA breaches
    pub sla_breach_count: u32,
    /// Price compliance (% of orders at contract price)
    pub price_compliance_pct: f64,
    /// Number of contract amendments
    pub amendment_count: u32,
}

/// Supplier scorecard for a review period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierScorecard {
    /// Unique scorecard identifier
    pub scorecard_id: String,
    /// Vendor ID
    pub vendor_id: String,
    /// Company code
    pub company_code: String,
    /// Review period start
    pub period_start: NaiveDate,
    /// Review period end
    pub period_end: NaiveDate,
    /// On-time delivery rate (0.0 to 1.0)
    pub on_time_delivery_rate: f64,
    /// Quality acceptance rate (0.0 to 1.0)
    pub quality_rate: f64,
    /// Price competitiveness score (0.0 to 100.0)
    pub price_score: f64,
    /// Responsiveness score (0.0 to 100.0)
    pub responsiveness_score: f64,
    /// Overall weighted score (0.0 to 100.0)
    pub overall_score: f64,
    /// Letter grade (A, B, C, D, F)
    pub grade: String,
    /// Trend direction
    pub trend: ScoreboardTrend,
    /// Contract compliance details
    pub contract_compliance: Vec<ContractComplianceMetrics>,
    /// Recommendation
    pub recommendation: ScorecardRecommendation,
    /// Reviewer ID
    pub reviewer_id: String,
    /// Review comments
    pub comments: Option<String>,
}
