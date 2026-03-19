//! Going concern assessment models — ISA 570 / ASC 205-40.
//!
//! Going concern is a fundamental assumption underlying the preparation of
//! financial statements.  Auditors are required (ISA 570) to evaluate whether
//! there is a material uncertainty about the entity's ability to continue as a
//! going concern over the next twelve months.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Type of going concern indicator identified during the assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoingConcernIndicatorType {
    /// Entity has sustained operating losses over multiple periods.
    RecurringOperatingLosses,
    /// Net cash outflows from operations over one or more periods.
    NegativeOperatingCashFlow,
    /// Current liabilities exceed current assets (negative working capital).
    WorkingCapitalDeficiency,
    /// One or more financial covenants have been breached.
    DebtCovenantBreach,
    /// Departure of a major customer materially impacting revenue.
    LossOfKeyCustomer,
    /// Regulatory action threatening the entity's licence to operate.
    RegulatoryAction,
    /// Material litigation exposure that could threaten solvency.
    LitigationExposure,
    /// Entity has been unable to refinance or obtain new credit facilities.
    InabilityToObtainFinancing,
}

impl std::fmt::Display for GoingConcernIndicatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::RecurringOperatingLosses => "Recurring Operating Losses",
            Self::NegativeOperatingCashFlow => "Negative Operating Cash Flow",
            Self::WorkingCapitalDeficiency => "Working Capital Deficiency",
            Self::DebtCovenantBreach => "Debt Covenant Breach",
            Self::LossOfKeyCustomer => "Loss of Key Customer",
            Self::RegulatoryAction => "Regulatory Action",
            Self::LitigationExposure => "Litigation Exposure",
            Self::InabilityToObtainFinancing => "Inability to Obtain Financing",
        };
        write!(f, "{s}")
    }
}

/// Severity classification of a going concern indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoingConcernSeverity {
    /// Indicator present but manageable; unlikely to threaten continuity alone.
    Low,
    /// Indicator creates meaningful doubt; mitigating plans are required.
    Medium,
    /// Indicator poses a serious threat to the entity's continuity.
    High,
}

/// Auditor's overall conclusion on going concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoingConcernConclusion {
    /// No material uncertainty exists; going concern basis is appropriate.
    #[default]
    NoMaterialUncertainty,
    /// A material uncertainty exists and is adequately disclosed (ISA 570.22).
    MaterialUncertaintyExists,
    /// Significant doubt that the entity is a going concern (ASC 205-40).
    GoingConcernDoubt,
}

// ---------------------------------------------------------------------------
// Indicator
// ---------------------------------------------------------------------------

/// A single going concern indicator identified during the assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoingConcernIndicator {
    /// Nature of the indicator.
    pub indicator_type: GoingConcernIndicatorType,
    /// Assessed severity.
    pub severity: GoingConcernSeverity,
    /// Narrative description of the specific circumstances.
    pub description: String,
    /// Quantitative measure associated with the indicator (e.g. net loss amount).
    #[serde(
        with = "rust_decimal::serde::str_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub quantitative_measure: Option<Decimal>,
    /// Threshold at which the indicator becomes critical (e.g. covenant limit).
    #[serde(
        with = "rust_decimal::serde::str_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub threshold: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// Assessment
// ---------------------------------------------------------------------------

/// Going concern assessment prepared for a single entity and reporting period.
///
/// One assessment is generated per entity per period.  The auditor's
/// conclusion is driven by the number and severity of indicators identified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoingConcernAssessment {
    /// Entity code of the assessed entity.
    pub entity_code: String,
    /// Date on which the assessment was finalised.
    pub assessment_date: NaiveDate,
    /// Human-readable period descriptor (e.g. "FY2024").
    pub assessment_period: String,
    /// Indicators identified during the assessment (may be empty).
    pub indicators: Vec<GoingConcernIndicator>,
    /// Management's plans to address the identified indicators.
    pub management_plans: Vec<String>,
    /// Auditor's overall conclusion.
    pub auditor_conclusion: GoingConcernConclusion,
    /// Whether a material uncertainty paragraph is required in the audit report.
    pub material_uncertainty_exists: bool,
}

impl GoingConcernAssessment {
    /// Derive the conclusion and `material_uncertainty_exists` flag from the
    /// number of indicators present.
    ///
    /// | Indicator count | Conclusion                  | Material uncertainty |
    /// |-----------------|-----------------------------|----------------------|
    /// | 0               | `NoMaterialUncertainty`     | false                |
    /// | 1–2             | `MaterialUncertaintyExists` | true                 |
    /// | 3+              | `GoingConcernDoubt`         | true                 |
    pub fn conclude_from_indicators(mut self) -> Self {
        let n = self.indicators.len();
        self.auditor_conclusion = match n {
            0 => GoingConcernConclusion::NoMaterialUncertainty,
            1..=2 => GoingConcernConclusion::MaterialUncertaintyExists,
            _ => GoingConcernConclusion::GoingConcernDoubt,
        };
        self.material_uncertainty_exists = n > 0;
        self
    }
}
