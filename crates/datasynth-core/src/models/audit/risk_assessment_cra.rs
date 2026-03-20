//! Combined Risk Assessment (CRA) models per ISA 315.
//!
//! The CRA is the cornerstone of a risk-based audit.  For each account area and
//! financial statement assertion the auditor combines inherent risk (IR) and
//! control risk (CR) into a single CRA level that drives the nature, extent,
//! and timing of planned audit procedures.
//!
//! References:
//! - ISA 315 (Revised 2019) — Identifying and Assessing Risks of Material Misstatement
//! - ISA 315.28 — Significant risks require special audit consideration
//! - ISA 240 — Revenue occurrence is always presumed a significant fraud risk

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core enums
// ---------------------------------------------------------------------------

/// Financial statement assertion being assessed (ISA 315.A129).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAssertion {
    // ---- Transaction-level assertions (ISA 315.A129a) ----
    /// Recorded transactions and events have occurred and pertain to the entity.
    Occurrence,
    /// All transactions and events that should have been recorded have been.
    Completeness,
    /// Amounts and other data have been recorded appropriately.
    Accuracy,
    /// Transactions have been recorded in the correct accounting period.
    Cutoff,
    /// Transactions have been recorded in the proper accounts.
    Classification,
    // ---- Account-balance assertions (ISA 315.A129b) ----
    /// Assets, liabilities and equity interests exist at the period end.
    Existence,
    /// The entity holds or controls the rights to assets; liabilities are obligations.
    RightsAndObligations,
    /// All assets, liabilities and equity interests that should have been recorded are.
    CompletenessBalance,
    /// Assets, liabilities and equity interests are included at appropriate amounts.
    ValuationAndAllocation,
    // ---- Presentation assertions (ISA 315.A129c) ----
    /// All disclosures are appropriately described, disclosed and presented.
    PresentationAndDisclosure,
}

impl std::fmt::Display for AuditAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Occurrence => "Occurrence",
            Self::Completeness => "Completeness",
            Self::Accuracy => "Accuracy",
            Self::Cutoff => "Cutoff",
            Self::Classification => "Classification",
            Self::Existence => "Existence",
            Self::RightsAndObligations => "Rights & Obligations",
            Self::CompletenessBalance => "Completeness (Balance)",
            Self::ValuationAndAllocation => "Valuation & Allocation",
            Self::PresentationAndDisclosure => "Presentation & Disclosure",
        };
        write!(f, "{s}")
    }
}

/// Individual risk rating — used separately for inherent risk and control risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskRating {
    /// Well below the acceptable threshold.
    Low,
    /// Moderate — some risk present but not pervasive.
    Medium,
    /// Elevated — significant susceptibility to misstatement.
    High,
}

impl std::fmt::Display for RiskRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        };
        write!(f, "{s}")
    }
}

/// Combined Risk Assessment level — derived from the IR × CR matrix.
///
/// | IR \ CR  | Low      | Medium   | High  |
/// |----------|----------|----------|-------|
/// | Low      | Minimal  | Low      | Moderate |
/// | Medium   | Low      | Moderate | High  |
/// | High     | Moderate | High     | High  |
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CraLevel {
    /// Low IR + Low CR.
    Minimal,
    /// Low IR + Medium CR, or Medium IR + Low CR.
    Low,
    /// Medium IR + Medium CR, or High IR + Low CR.
    Moderate,
    /// High IR + Medium CR, or High IR + High CR, or Medium IR + High CR.
    High,
}

impl CraLevel {
    /// Compute the CRA level from individual IR and CR ratings.
    ///
    /// | IR \ CR  | Low     | Medium   | High     |
    /// |----------|---------|----------|----------|
    /// | Low      | Minimal | Low      | Moderate |
    /// | Medium   | Low     | Moderate | High     |
    /// | High     | Moderate| High     | High     |
    pub fn from_ratings(ir: RiskRating, cr: RiskRating) -> Self {
        match (ir, cr) {
            (RiskRating::Low, RiskRating::Low) => Self::Minimal,
            (RiskRating::Low, RiskRating::Medium) | (RiskRating::Medium, RiskRating::Low) => {
                Self::Low
            }
            (RiskRating::Medium, RiskRating::Medium)
            | (RiskRating::High, RiskRating::Low)
            | (RiskRating::Low, RiskRating::High) => Self::Moderate,
            _ => Self::High,
        }
    }
}

impl std::fmt::Display for CraLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Minimal => "Minimal",
            Self::Low => "Low",
            Self::Moderate => "Moderate",
            Self::High => "High",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Planned response
// ---------------------------------------------------------------------------

/// Nature of planned audit procedures in response to the CRA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureNature {
    /// Substantive procedures only — no reliance on controls.
    SubstantiveOnly,
    /// Combined approach — tests of controls plus reduced substantive procedures.
    Combined,
    /// Controls reliance — extensive controls testing with minimal substantive.
    ControlsReliance,
}

/// Planned extent of substantive testing (maps to relative sample sizes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingExtent {
    /// Below-standard sample sizes applicable to low-risk areas.
    Reduced,
    /// Standard sample sizes.
    Standard,
    /// Above-standard sample sizes for high-risk or significant-risk areas.
    Extended,
}

/// Timing of planned procedures relative to the period end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureTiming {
    /// Procedures performed at an interim date only.
    Interim,
    /// Procedures performed at or after the period end only.
    YearEnd,
    /// Procedures at both interim and year-end (roll-forward required).
    Both,
}

/// Planned response design driven by the CRA level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraPlannedResponse {
    /// Nature: substantive only, combined, or controls reliance.
    pub nature: ProcedureNature,
    /// Extent: reduced, standard, or extended.
    pub extent: SamplingExtent,
    /// Timing: interim, year-end, or both.
    pub timing: ProcedureTiming,
}

impl CraPlannedResponse {
    /// Derive a planned response from the CRA level per ISA 330 guidance.
    ///
    /// | CRA level | Nature           | Extent   | Timing   |
    /// |-----------|------------------|----------|----------|
    /// | Minimal   | SubstantiveOnly  | Reduced  | YearEnd  |
    /// | Low       | SubstantiveOnly  | Reduced  | YearEnd  |
    /// | Moderate  | Combined         | Standard | YearEnd  |
    /// | High      | SubstantiveOnly  | Extended | Both     |
    pub fn from_cra_level(level: CraLevel) -> Self {
        match level {
            CraLevel::Minimal | CraLevel::Low => Self {
                nature: ProcedureNature::SubstantiveOnly,
                extent: SamplingExtent::Reduced,
                timing: ProcedureTiming::YearEnd,
            },
            CraLevel::Moderate => Self {
                nature: ProcedureNature::Combined,
                extent: SamplingExtent::Standard,
                timing: ProcedureTiming::YearEnd,
            },
            CraLevel::High => Self {
                nature: ProcedureNature::SubstantiveOnly,
                extent: SamplingExtent::Extended,
                timing: ProcedureTiming::Both,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Main struct
// ---------------------------------------------------------------------------

/// Combined Risk Assessment for a single account area / assertion pair.
///
/// One `CombinedRiskAssessment` is generated for each (account area, assertion)
/// combination that the auditor scopes into the engagement.  The CRA drives the
/// design of audit procedures (ISA 330) and the allocation of materiality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedRiskAssessment {
    /// Unique identifier (deterministic slug).
    pub id: String,
    /// Entity / company code the assessment belongs to.
    pub entity_code: String,
    /// Account area (e.g. "Revenue", "Trade Receivables", "Inventory").
    pub account_area: String,
    /// The specific assertion being assessed.
    pub assertion: AuditAssertion,
    /// Inherent risk rating.
    pub inherent_risk: RiskRating,
    /// Control risk rating (effectiveness of related internal controls).
    pub control_risk: RiskRating,
    /// Combined risk level derived from the IR × CR matrix.
    pub combined_risk: CraLevel,
    /// Whether this is a significant risk per ISA 315.28 requiring special consideration.
    pub significant_risk: bool,
    /// Descriptive risk factors supporting the assessment.
    pub risk_factors: Vec<String>,
    /// Planned audit response designed for this CRA.
    pub planned_response: CraPlannedResponse,
}

impl CombinedRiskAssessment {
    /// Build a `CombinedRiskAssessment` and derive the CRA level and response.
    pub fn new(
        entity_code: &str,
        account_area: &str,
        assertion: AuditAssertion,
        inherent_risk: RiskRating,
        control_risk: RiskRating,
        significant_risk: bool,
        risk_factors: Vec<String>,
    ) -> Self {
        let combined_risk = CraLevel::from_ratings(inherent_risk, control_risk);
        let planned_response = CraPlannedResponse::from_cra_level(combined_risk);
        let id = format!(
            "CRA-{}-{}-{}",
            entity_code,
            account_area.replace(' ', "_").to_uppercase(),
            format!("{assertion:?}").to_uppercase(),
        );

        Self {
            id,
            entity_code: entity_code.to_string(),
            account_area: account_area.to_string(),
            assertion,
            inherent_risk,
            control_risk,
            combined_risk,
            significant_risk,
            risk_factors,
            planned_response,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn cra_matrix_low_low_is_minimal() {
        let level = CraLevel::from_ratings(RiskRating::Low, RiskRating::Low);
        assert_eq!(level, CraLevel::Minimal);
    }

    #[test]
    fn cra_matrix_high_high_is_high() {
        let level = CraLevel::from_ratings(RiskRating::High, RiskRating::High);
        assert_eq!(level, CraLevel::High);
    }

    #[test]
    fn cra_matrix_medium_medium_is_moderate() {
        let level = CraLevel::from_ratings(RiskRating::Medium, RiskRating::Medium);
        assert_eq!(level, CraLevel::Moderate);
    }

    #[test]
    fn cra_matrix_high_low_is_moderate() {
        let level = CraLevel::from_ratings(RiskRating::High, RiskRating::Low);
        assert_eq!(level, CraLevel::Moderate);
    }

    #[test]
    fn cra_matrix_low_high_is_moderate() {
        // Low IR caps the combined level even when controls provide no assurance.
        let level = CraLevel::from_ratings(RiskRating::Low, RiskRating::High);
        assert_eq!(level, CraLevel::Moderate);
    }

    #[test]
    fn cra_matrix_medium_high_is_high() {
        let level = CraLevel::from_ratings(RiskRating::Medium, RiskRating::High);
        assert_eq!(level, CraLevel::High);
    }

    #[test]
    fn planned_response_high_cra_is_extended_both() {
        let resp = CraPlannedResponse::from_cra_level(CraLevel::High);
        assert_eq!(resp.extent, SamplingExtent::Extended);
        assert_eq!(resp.timing, ProcedureTiming::Both);
        assert_eq!(resp.nature, ProcedureNature::SubstantiveOnly);
    }

    #[test]
    fn planned_response_moderate_cra_is_combined_standard() {
        let resp = CraPlannedResponse::from_cra_level(CraLevel::Moderate);
        assert_eq!(resp.nature, ProcedureNature::Combined);
        assert_eq!(resp.extent, SamplingExtent::Standard);
    }

    #[test]
    fn cra_new_derives_combined_risk() {
        let cra = CombinedRiskAssessment::new(
            "C001",
            "Revenue",
            AuditAssertion::Occurrence,
            RiskRating::High,
            RiskRating::Medium,
            true,
            vec!["Presumed fraud risk per ISA 240".into()],
        );
        assert_eq!(cra.combined_risk, CraLevel::High);
        assert!(cra.significant_risk);
    }
}
