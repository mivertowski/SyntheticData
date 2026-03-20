//! Analytical relationship models for audit support data — ISA 520.
//!
//! Analytical procedures (ISA 520) require the auditor to develop expectations
//! about plausible relationships between financial and non-financial data.
//! This module captures those relationships in a structured form so that they
//! can be output as training data and used by AI-assisted audit tools.
//!
//! Standard relationships computed per entity per period:
//! - DSO (Days Sales Outstanding)
//! - DPO (Days Payable Outstanding)
//! - Inventory Turnover
//! - Gross Margin
//! - Payroll to Revenue
//! - Depreciation to Gross Fixed Assets
//! - Revenue Growth (period-on-period)
//! - Operating Expense Ratio

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The mathematical / statistical nature of the analytical relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// A ratio between two financial line items (e.g. DSO = AR / Revenue × 365).
    Ratio,
    /// A period-on-period trend (e.g. revenue growth rate).
    Trend,
    /// A correlation between two time-series variables (e.g. revenue vs AR).
    Correlation,
    /// A reasonableness check — does the value fall within an expected range?
    Reasonableness,
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ratio => "Ratio",
            Self::Trend => "Trend",
            Self::Correlation => "Correlation",
            Self::Reasonableness => "Reasonableness",
        };
        write!(f, "{s}")
    }
}

/// Reliability of the underlying data used to compute the relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataReliability {
    /// Data comes from a routine, fully-automated, previously-audited source.
    High,
    /// Data is semi-automated or subject to internal review but not audit.
    Medium,
    /// Data is manually compiled or unverified.
    Low,
}

impl std::fmt::Display for DataReliability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Supporting sub-structures
// ---------------------------------------------------------------------------

/// A single period's computed value for an analytical relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodDataPoint {
    /// Human-readable period label (e.g. "FY2024-Q3", "FY2023").
    pub period: String,
    /// Computed value for the relationship in this period.
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// Whether this is the current (under-audit) period.
    pub is_current: bool,
}

/// A non-financial or operational metric that supports the analytical relationship.
///
/// Examples: headcount for payroll ratios, units shipped for revenue reasonableness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingMetric {
    /// Name of the metric (e.g. "Employee headcount", "Units shipped").
    pub metric_name: String,
    /// Metric value for the current period.
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// System or process from which the metric was sourced.
    pub source: String,
}

// ---------------------------------------------------------------------------
// Main struct
// ---------------------------------------------------------------------------

/// An analytical relationship computed from actual journal entry data.
///
/// Each relationship captures the formula, historical trend, expected range,
/// and any variance explanation — providing the auditor with structured
/// evidence to support or challenge the recorded amounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalRelationship {
    /// Unique identifier for this relationship record.
    pub id: String,
    /// Entity / company code this relationship relates to.
    pub entity_code: String,
    /// Human-readable name (e.g. "Days Sales Outstanding", "Gross Margin").
    pub relationship_name: String,
    /// The account area or financial statement section (e.g. "Receivables", "Revenue").
    pub account_area: String,
    /// Mathematical / statistical type of the relationship.
    pub relationship_type: RelationshipType,
    /// Plain-English formula showing how the value is calculated.
    /// Example: `"AR / Revenue * 365 = DSO"`
    pub formula: String,
    /// Computed values for the current period and 2–3 prior comparison periods.
    pub periods: Vec<PeriodDataPoint>,
    /// Expected range `(lower_bound, upper_bound)` based on industry norms.
    /// The value is expressed in the same units as the ratio (e.g. days, %, ×).
    pub expected_range: (String, String),
    /// Explanation of why the current value is outside the expected range,
    /// or `None` if it falls within range.
    pub variance_explanation: Option<String>,
    /// Non-financial supporting metrics used to corroborate the relationship.
    pub supporting_metrics: Vec<SupportingMetric>,
    /// Reliability of the data underlying this relationship.
    pub reliability: DataReliability,
    /// Whether the current period value is within the expected range.
    pub within_expected_range: bool,
}

impl AnalyticalRelationship {
    /// Return the current period data point (the one being audited).
    pub fn current_period(&self) -> Option<&PeriodDataPoint> {
        self.periods.iter().find(|p| p.is_current)
    }
}
