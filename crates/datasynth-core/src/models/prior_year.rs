//! Prior-year comparative data models for year-over-year audit analysis.
//!
//! Supports ISA 315 (Understanding the Entity) and ISA 520 (Analytical Procedures)
//! by providing prior-year balances, variances, and prior-year audit findings
//! for follow-up procedures.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Prior-year comparative data for year-over-year analysis.
///
/// Each record pairs a current-year account balance with its prior-year
/// counterpart and computes the variance (absolute and percentage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorYearComparative {
    /// GL account code (e.g., "1100", "4000")
    pub account_code: String,
    /// Account description
    pub account_name: String,
    /// Current-year closing balance
    #[serde(with = "rust_decimal::serde::str")]
    pub current_year_amount: Decimal,
    /// Prior-year closing balance (derived with realistic variance)
    #[serde(with = "rust_decimal::serde::str")]
    pub prior_year_amount: Decimal,
    /// Absolute variance: current_year_amount - prior_year_amount
    #[serde(with = "rust_decimal::serde::str")]
    pub variance: Decimal,
    /// Variance as percentage: (current - prior) / |prior| * 100
    pub variance_pct: f64,
    /// Entity / company code
    pub entity_code: String,
    /// Fiscal period label (e.g., "2025-12", "2025-Q4")
    pub period: String,
}

/// Prior-year audit finding for follow-up procedures.
///
/// Findings from the prior-year engagement that may require current-year
/// follow-up, re-testing, or disclosure in the current audit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorYearFinding {
    /// Unique finding identifier (deterministic UUID)
    pub finding_id: Uuid,
    /// Fiscal year the finding was originally raised
    pub fiscal_year: i32,
    /// Finding classification: "control_deficiency", "misstatement",
    /// "significant_deficiency", "material_weakness"
    pub finding_type: String,
    /// Narrative description of the finding
    pub description: String,
    /// Current remediation status: "remediated", "open", "recurring",
    /// "partially_remediated"
    pub status: String,
    /// Risk area: "revenue", "receivables", "payables", "inventory", "estimates"
    pub risk_area: String,
    /// Original monetary amount of the finding (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_amount: Option<Decimal>,
    /// Date on which remediation was completed (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation_date: Option<NaiveDate>,
    /// Whether the current-year audit team must follow up
    pub follow_up_required: bool,
}

/// Prior-year audit engagement summary.
///
/// An aggregate record that bundles the prior-year opinion, materiality,
/// comparatives, and findings into a single envelope for the current-year
/// audit team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorYearSummary {
    /// Prior fiscal year (e.g., 2024 if current year is 2025)
    pub fiscal_year: i32,
    /// Entity / company code
    pub entity_code: String,
    /// Audit opinion issued: "unmodified", "qualified", "adverse", "disclaimer"
    pub opinion_type: String,
    /// Prior-year materiality threshold
    #[serde(with = "rust_decimal::serde::str")]
    pub materiality: Decimal,
    /// Total number of findings raised
    pub total_findings: usize,
    /// Number of findings still open or recurring
    pub open_findings: usize,
    /// Key audit matters from the prior-year report
    pub key_audit_matters: Vec<String>,
    /// Per-account comparative data
    pub comparatives: Vec<PriorYearComparative>,
    /// Prior-year findings carried forward
    pub findings: Vec<PriorYearFinding>,
}
