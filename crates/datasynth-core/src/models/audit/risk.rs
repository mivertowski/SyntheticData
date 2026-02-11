//! Risk assessment models per ISA 315 and ISA 330.
//!
//! Risk assessment is the foundation of a risk-based audit approach,
//! identifying risks of material misstatement at both the financial
//! statement level and assertion level.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::engagement::RiskLevel;
use super::workpaper::Assertion;

/// Risk assessment for an account or process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Unique risk ID
    pub risk_id: Uuid,
    /// External reference
    pub risk_ref: String,
    /// Engagement ID
    pub engagement_id: Uuid,
    /// Risk category
    pub risk_category: RiskCategory,
    /// Account or process being assessed
    pub account_or_process: String,
    /// Specific assertion if applicable
    pub assertion: Option<Assertion>,
    /// Risk description
    pub description: String,

    // === Risk Assessment ===
    /// Inherent risk assessment
    pub inherent_risk: RiskLevel,
    /// Control risk assessment
    pub control_risk: RiskLevel,
    /// Combined risk of material misstatement
    pub risk_of_material_misstatement: RiskLevel,
    /// Is this a significant risk per ISA 315?
    pub is_significant_risk: bool,
    /// Rationale for significant risk designation
    pub significant_risk_rationale: Option<String>,

    // === Fraud Risk ===
    /// Fraud risk factors identified
    pub fraud_risk_factors: Vec<FraudRiskFactor>,
    /// Presumed fraud risk in revenue recognition?
    pub presumed_revenue_fraud_risk: bool,
    /// Presumed management override risk?
    pub presumed_management_override: bool,

    // === Response ===
    /// Planned audit response
    pub planned_response: Vec<PlannedResponse>,
    /// Nature of procedures (substantive, control, combined)
    pub response_nature: ResponseNature,
    /// Extent (sample size considerations)
    pub response_extent: String,
    /// Timing (interim, year-end, subsequent)
    pub response_timing: ResponseTiming,

    // === Assessment Details ===
    /// Assessed by user ID
    pub assessed_by: String,
    /// Assessment date
    pub assessed_date: NaiveDate,
    /// Review status
    pub review_status: RiskReviewStatus,
    /// Reviewer ID
    pub reviewer_id: Option<String>,
    /// Review date
    pub review_date: Option<NaiveDate>,

    // === Cross-References ===
    /// Related workpaper IDs
    pub workpaper_refs: Vec<Uuid>,
    /// Related control IDs
    pub related_controls: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RiskAssessment {
    /// Create a new risk assessment.
    pub fn new(
        engagement_id: Uuid,
        risk_category: RiskCategory,
        account_or_process: &str,
        description: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            risk_id: Uuid::new_v4(),
            risk_ref: format!(
                "RISK-{}",
                Uuid::new_v4().simple().to_string()[..8].to_uppercase()
            ),
            engagement_id,
            risk_category,
            account_or_process: account_or_process.into(),
            assertion: None,
            description: description.into(),
            inherent_risk: RiskLevel::Medium,
            control_risk: RiskLevel::Medium,
            risk_of_material_misstatement: RiskLevel::Medium,
            is_significant_risk: false,
            significant_risk_rationale: None,
            fraud_risk_factors: Vec::new(),
            presumed_revenue_fraud_risk: false,
            presumed_management_override: true,
            planned_response: Vec::new(),
            response_nature: ResponseNature::Combined,
            response_extent: String::new(),
            response_timing: ResponseTiming::YearEnd,
            assessed_by: String::new(),
            assessed_date: now.date_naive(),
            review_status: RiskReviewStatus::Draft,
            reviewer_id: None,
            review_date: None,
            workpaper_refs: Vec::new(),
            related_controls: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the assertion being assessed.
    pub fn with_assertion(mut self, assertion: Assertion) -> Self {
        self.assertion = Some(assertion);
        self
    }

    /// Set risk levels.
    pub fn with_risk_levels(mut self, inherent: RiskLevel, control: RiskLevel) -> Self {
        self.inherent_risk = inherent;
        self.control_risk = control;
        self.risk_of_material_misstatement = self.calculate_romm();
        self
    }

    /// Mark as significant risk.
    pub fn mark_significant(mut self, rationale: &str) -> Self {
        self.is_significant_risk = true;
        self.significant_risk_rationale = Some(rationale.into());
        self
    }

    /// Add a fraud risk factor.
    pub fn add_fraud_factor(&mut self, factor: FraudRiskFactor) {
        self.fraud_risk_factors.push(factor);
        self.updated_at = Utc::now();
    }

    /// Add a planned response.
    pub fn add_response(&mut self, response: PlannedResponse) {
        self.planned_response.push(response);
        self.updated_at = Utc::now();
    }

    /// Set who assessed this risk.
    pub fn with_assessed_by(mut self, user_id: &str, date: NaiveDate) -> Self {
        self.assessed_by = user_id.into();
        self.assessed_date = date;
        self
    }

    /// Calculate risk of material misstatement from IR and CR.
    fn calculate_romm(&self) -> RiskLevel {
        let ir_score = self.inherent_risk.score();
        let cr_score = self.control_risk.score();
        let combined = (ir_score + cr_score) / 2;
        RiskLevel::from_score(combined)
    }

    /// Get the detection risk needed to achieve acceptable audit risk.
    pub fn required_detection_risk(&self) -> DetectionRisk {
        match self.risk_of_material_misstatement {
            RiskLevel::Low => DetectionRisk::High,
            RiskLevel::Medium => DetectionRisk::Medium,
            RiskLevel::High | RiskLevel::Significant => DetectionRisk::Low,
        }
    }

    /// Check if this risk requires special audit consideration.
    pub fn requires_special_consideration(&self) -> bool {
        self.is_significant_risk
            || matches!(
                self.risk_of_material_misstatement,
                RiskLevel::High | RiskLevel::Significant
            )
            || !self.fraud_risk_factors.is_empty()
    }
}

/// Risk category per ISA 315.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    /// Risk at the financial statement level
    FinancialStatementLevel,
    /// Risk at the assertion level
    #[default]
    AssertionLevel,
    /// Fraud risk
    FraudRisk,
    /// Going concern risk
    GoingConcern,
    /// Related party risk
    RelatedParty,
    /// Accounting estimate risk
    EstimateRisk,
    /// IT general control risk
    ItGeneralControl,
    /// Regulatory compliance risk
    RegulatoryCompliance,
}

/// Fraud risk factor per the fraud triangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudRiskFactor {
    /// Factor ID
    pub factor_id: Uuid,
    /// Element of fraud triangle
    pub factor_type: FraudTriangleElement,
    /// Specific indicator description
    pub indicator: String,
    /// Risk score (0-100)
    pub score: u8,
    /// Trend direction
    pub trend: Trend,
    /// Source of information
    pub source: String,
    /// Date identified
    pub identified_date: NaiveDate,
}

impl FraudRiskFactor {
    /// Create a new fraud risk factor.
    pub fn new(
        factor_type: FraudTriangleElement,
        indicator: &str,
        score: u8,
        source: &str,
    ) -> Self {
        Self {
            factor_id: Uuid::new_v4(),
            factor_type,
            indicator: indicator.into(),
            score: score.min(100),
            trend: Trend::Stable,
            source: source.into(),
            identified_date: Utc::now().date_naive(),
        }
    }

    /// Set the trend.
    pub fn with_trend(mut self, trend: Trend) -> Self {
        self.trend = trend;
        self
    }
}

/// Elements of the fraud triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FraudTriangleElement {
    /// Opportunity to commit fraud
    Opportunity,
    /// Incentive/pressure to commit fraud
    Pressure,
    /// Rationalization/attitude
    Rationalization,
}

impl FraudTriangleElement {
    /// Get a description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Opportunity => "Circumstances providing opportunity to commit fraud",
            Self::Pressure => "Incentives or pressures to commit fraud",
            Self::Rationalization => "Attitude or rationalization to justify fraud",
        }
    }
}

/// Trend direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Trend {
    /// Increasing
    Increasing,
    /// Stable
    #[default]
    Stable,
    /// Decreasing
    Decreasing,
}

/// Planned audit response to identified risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedResponse {
    /// Response ID
    pub response_id: Uuid,
    /// Procedure description
    pub procedure: String,
    /// Procedure type
    pub procedure_type: ResponseProcedureType,
    /// Assertion addressed
    pub assertion_addressed: Assertion,
    /// Assigned to user ID
    pub assigned_to: String,
    /// Target completion date
    pub target_date: NaiveDate,
    /// Status
    pub status: ResponseStatus,
    /// Workpaper reference when complete
    pub workpaper_ref: Option<Uuid>,
}

impl PlannedResponse {
    /// Create a new planned response.
    pub fn new(
        procedure: &str,
        procedure_type: ResponseProcedureType,
        assertion: Assertion,
        assigned_to: &str,
        target_date: NaiveDate,
    ) -> Self {
        Self {
            response_id: Uuid::new_v4(),
            procedure: procedure.into(),
            procedure_type,
            assertion_addressed: assertion,
            assigned_to: assigned_to.into(),
            target_date,
            status: ResponseStatus::Planned,
            workpaper_ref: None,
        }
    }
}

/// Type of response procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseProcedureType {
    /// Test of controls
    TestOfControls,
    /// Substantive analytical procedure
    AnalyticalProcedure,
    /// Substantive test of details
    #[default]
    TestOfDetails,
    /// External confirmation
    Confirmation,
    /// Physical inspection
    PhysicalInspection,
    /// Inquiry
    Inquiry,
}

/// Nature of audit response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseNature {
    /// Substantive procedures only
    SubstantiveOnly,
    /// Controls reliance with reduced substantive
    ControlsReliance,
    /// Combined approach
    #[default]
    Combined,
}

/// Timing of audit response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseTiming {
    /// Interim testing
    Interim,
    /// Year-end testing
    #[default]
    YearEnd,
    /// Roll-forward from interim
    RollForward,
    /// Subsequent events testing
    Subsequent,
}

/// Status of planned response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// Planned but not started
    #[default]
    Planned,
    /// In progress
    InProgress,
    /// Complete
    Complete,
    /// Deferred
    Deferred,
    /// Not required
    NotRequired,
}

/// Risk review status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskReviewStatus {
    /// Draft assessment
    #[default]
    Draft,
    /// Pending review
    PendingReview,
    /// Reviewed and approved
    Approved,
    /// Requires revision
    RequiresRevision,
}

/// Detection risk level (inverse of ROMM for achieving acceptable AR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionRisk {
    /// Can accept high detection risk (less testing)
    High,
    /// Medium detection risk
    Medium,
    /// Low detection risk required (more testing)
    Low,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_assessment_creation() {
        let risk = RiskAssessment::new(
            Uuid::new_v4(),
            RiskCategory::AssertionLevel,
            "Revenue",
            "Risk of fictitious revenue recognition",
        )
        .with_assertion(Assertion::Occurrence)
        .with_risk_levels(RiskLevel::High, RiskLevel::Medium);

        assert!(risk.inherent_risk == RiskLevel::High);
        assert!(
            risk.requires_special_consideration()
                || risk.risk_of_material_misstatement != RiskLevel::Low
        );
    }

    #[test]
    fn test_significant_risk() {
        let risk = RiskAssessment::new(
            Uuid::new_v4(),
            RiskCategory::FraudRisk,
            "Revenue",
            "Fraud risk in revenue recognition",
        )
        .mark_significant("Presumed fraud risk per ISA 240");

        assert!(risk.is_significant_risk);
        assert!(risk.requires_special_consideration());
    }

    #[test]
    fn test_fraud_risk_factor() {
        let factor = FraudRiskFactor::new(
            FraudTriangleElement::Pressure,
            "Management bonus tied to revenue targets",
            75,
            "Bonus plan review",
        )
        .with_trend(Trend::Increasing);

        assert_eq!(factor.factor_type, FraudTriangleElement::Pressure);
        assert_eq!(factor.score, 75);
    }

    #[test]
    fn test_detection_risk() {
        let risk = RiskAssessment::new(
            Uuid::new_v4(),
            RiskCategory::AssertionLevel,
            "Cash",
            "Low risk account",
        )
        .with_risk_levels(RiskLevel::Low, RiskLevel::Low);

        assert_eq!(risk.required_detection_risk(), DetectionRisk::High);
    }
}
