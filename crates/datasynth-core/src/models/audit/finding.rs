//! Audit finding and issue models per ISA 265.
//!
//! Findings represent deficiencies identified during the audit,
//! ranging from control deficiencies to material misstatements.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::engagement::RiskLevel;
use super::workpaper::Assertion;

/// Audit finding representing an identified issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    /// Unique finding ID
    pub finding_id: Uuid,
    /// External reference (e.g., "FIND-2025-001")
    pub finding_ref: String,
    /// Engagement ID
    pub engagement_id: Uuid,
    /// Finding type/classification
    pub finding_type: FindingType,
    /// Severity level
    pub severity: FindingSeverity,
    /// Finding title
    pub title: String,

    // === Finding Details (Condition, Criteria, Cause, Effect) ===
    /// Condition: What we found
    pub condition: String,
    /// Criteria: What it should be (standard/policy)
    pub criteria: String,
    /// Cause: Why it happened
    pub cause: String,
    /// Effect: Impact/consequence
    pub effect: String,

    // === Quantification ===
    /// Monetary impact if quantifiable
    pub monetary_impact: Option<Decimal>,
    /// Is this a known misstatement?
    pub is_misstatement: bool,
    /// Projected misstatement (for sampling)
    pub projected_misstatement: Option<Decimal>,
    /// Factual misstatement
    pub factual_misstatement: Option<Decimal>,
    /// Judgmental misstatement
    pub judgmental_misstatement: Option<Decimal>,

    // === Recommendations ===
    /// Recommendation for remediation
    pub recommendation: String,
    /// Management response
    pub management_response: Option<String>,
    /// Management response date
    pub management_response_date: Option<NaiveDate>,
    /// Does management agree?
    pub management_agrees: Option<bool>,

    // === Remediation ===
    /// Remediation plan
    pub remediation_plan: Option<RemediationPlan>,
    /// Finding status
    pub status: FindingStatus,

    // === Assertions & Accounts ===
    /// Assertions affected
    pub assertions_affected: Vec<Assertion>,
    /// Account IDs affected
    pub accounts_affected: Vec<String>,
    /// Process areas affected
    pub process_areas: Vec<String>,

    // === Relationship Linkage ===
    /// Control IDs related to this finding (populated at generation time)
    pub related_control_ids: Vec<String>,
    /// Risk ID this finding was raised against (populated at generation time)
    pub related_risk_id: Option<String>,
    /// Primary workpaper ID that documents this finding
    pub workpaper_id: Option<String>,

    // === References ===
    /// Supporting workpaper IDs
    pub workpaper_refs: Vec<Uuid>,
    /// Supporting evidence IDs
    pub evidence_refs: Vec<Uuid>,
    /// Related finding IDs (if recurring)
    pub related_findings: Vec<Uuid>,
    /// Prior year finding ID if recurring
    pub prior_year_finding_id: Option<Uuid>,

    // === Reporting ===
    /// Include in management letter?
    pub include_in_management_letter: bool,
    /// Report to those charged with governance?
    pub report_to_governance: bool,
    /// Communicated date
    pub communicated_date: Option<NaiveDate>,

    // === Metadata ===
    /// Identified by user ID
    pub identified_by: String,
    /// Date identified
    pub identified_date: NaiveDate,
    /// Reviewed by
    pub reviewed_by: Option<String>,
    /// Review date
    pub review_date: Option<NaiveDate>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuditFinding {
    /// Create a new audit finding.
    pub fn new(engagement_id: Uuid, finding_type: FindingType, title: &str) -> Self {
        let now = Utc::now();
        Self {
            finding_id: Uuid::new_v4(),
            finding_ref: format!("FIND-{}-{:03}", now.format("%Y"), 1),
            engagement_id,
            finding_type,
            severity: finding_type.default_severity(),
            title: title.into(),
            condition: String::new(),
            criteria: String::new(),
            cause: String::new(),
            effect: String::new(),
            monetary_impact: None,
            is_misstatement: false,
            projected_misstatement: None,
            factual_misstatement: None,
            judgmental_misstatement: None,
            recommendation: String::new(),
            management_response: None,
            management_response_date: None,
            management_agrees: None,
            remediation_plan: None,
            status: FindingStatus::Draft,
            assertions_affected: Vec::new(),
            accounts_affected: Vec::new(),
            process_areas: Vec::new(),
            related_control_ids: Vec::new(),
            related_risk_id: None,
            workpaper_id: None,
            workpaper_refs: Vec::new(),
            evidence_refs: Vec::new(),
            related_findings: Vec::new(),
            prior_year_finding_id: None,
            include_in_management_letter: false,
            report_to_governance: false,
            communicated_date: None,
            identified_by: String::new(),
            identified_date: now.date_naive(),
            reviewed_by: None,
            review_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the finding details (condition, criteria, cause, effect).
    pub fn with_details(
        mut self,
        condition: &str,
        criteria: &str,
        cause: &str,
        effect: &str,
    ) -> Self {
        self.condition = condition.into();
        self.criteria = criteria.into();
        self.cause = cause.into();
        self.effect = effect.into();
        self
    }

    /// Set monetary impact.
    pub fn with_monetary_impact(mut self, impact: Decimal) -> Self {
        self.monetary_impact = Some(impact);
        self.is_misstatement = true;
        self
    }

    /// Set misstatement details.
    pub fn with_misstatement(
        mut self,
        factual: Option<Decimal>,
        projected: Option<Decimal>,
        judgmental: Option<Decimal>,
    ) -> Self {
        self.factual_misstatement = factual;
        self.projected_misstatement = projected;
        self.judgmental_misstatement = judgmental;
        self.is_misstatement = true;
        self
    }

    /// Set recommendation.
    pub fn with_recommendation(mut self, recommendation: &str) -> Self {
        self.recommendation = recommendation.into();
        self
    }

    /// Add management response.
    pub fn add_management_response(&mut self, response: &str, agrees: bool, date: NaiveDate) {
        self.management_response = Some(response.into());
        self.management_agrees = Some(agrees);
        self.management_response_date = Some(date);
        self.status = FindingStatus::ManagementResponse;
        self.updated_at = Utc::now();
    }

    /// Set remediation plan.
    pub fn with_remediation_plan(&mut self, plan: RemediationPlan) {
        self.remediation_plan = Some(plan);
        self.status = FindingStatus::RemediationPlanned;
        self.updated_at = Utc::now();
    }

    /// Mark for reporting.
    pub fn mark_for_reporting(&mut self, management_letter: bool, governance: bool) {
        self.include_in_management_letter = management_letter;
        self.report_to_governance = governance;
        self.updated_at = Utc::now();
    }

    /// Get total misstatement amount.
    pub fn total_misstatement(&self) -> Decimal {
        let factual = self.factual_misstatement.unwrap_or_default();
        let projected = self.projected_misstatement.unwrap_or_default();
        let judgmental = self.judgmental_misstatement.unwrap_or_default();
        factual + projected + judgmental
    }

    /// Check if this is a material weakness (for SOX).
    pub fn is_material_weakness(&self) -> bool {
        matches!(self.finding_type, FindingType::MaterialWeakness)
    }

    /// Check if this requires governance communication per ISA 260.
    pub fn requires_governance_communication(&self) -> bool {
        matches!(
            self.finding_type,
            FindingType::MaterialWeakness | FindingType::SignificantDeficiency
        ) || matches!(
            self.severity,
            FindingSeverity::Critical | FindingSeverity::High
        )
    }
}

/// Type of audit finding per ISA 265.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    /// Material weakness in internal control
    MaterialWeakness,
    /// Significant deficiency in internal control
    SignificantDeficiency,
    /// Control deficiency (not significant)
    #[default]
    ControlDeficiency,
    /// Material misstatement in financial statements
    MaterialMisstatement,
    /// Immaterial misstatement
    ImmaterialMisstatement,
    /// Compliance exception
    ComplianceException,
    /// Other matter for management attention
    OtherMatter,
    /// IT-related deficiency
    ItDeficiency,
    /// Process improvement opportunity
    ProcessImprovement,
}

impl FindingType {
    /// Get the default severity for this finding type.
    pub fn default_severity(&self) -> FindingSeverity {
        match self {
            Self::MaterialWeakness => FindingSeverity::Critical,
            Self::SignificantDeficiency => FindingSeverity::High,
            Self::MaterialMisstatement => FindingSeverity::Critical,
            Self::ControlDeficiency | Self::ImmaterialMisstatement => FindingSeverity::Medium,
            Self::ComplianceException => FindingSeverity::Medium,
            Self::OtherMatter | Self::ProcessImprovement => FindingSeverity::Low,
            Self::ItDeficiency => FindingSeverity::Medium,
        }
    }

    /// Get ISA reference for this finding type.
    pub fn isa_reference(&self) -> &'static str {
        match self {
            Self::MaterialWeakness | Self::SignificantDeficiency | Self::ControlDeficiency => {
                "ISA 265"
            }
            Self::MaterialMisstatement | Self::ImmaterialMisstatement => "ISA 450",
            Self::ComplianceException => "ISA 250",
            _ => "ISA 260",
        }
    }

    /// Check if this type requires SOX reporting.
    pub fn requires_sox_reporting(&self) -> bool {
        matches!(self, Self::MaterialWeakness | Self::SignificantDeficiency)
    }
}

/// Finding severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Critical - immediate attention required
    Critical,
    /// High - significant impact
    High,
    /// Medium - moderate impact
    #[default]
    Medium,
    /// Low - minor impact
    Low,
    /// Informational only
    Informational,
}

impl FindingSeverity {
    /// Get numeric score for prioritization.
    pub fn score(&self) -> u8 {
        match self {
            Self::Critical => 5,
            Self::High => 4,
            Self::Medium => 3,
            Self::Low => 2,
            Self::Informational => 1,
        }
    }

    /// Convert to risk level.
    pub fn to_risk_level(&self) -> RiskLevel {
        match self {
            Self::Critical => RiskLevel::Significant,
            Self::High => RiskLevel::High,
            Self::Medium => RiskLevel::Medium,
            Self::Low | Self::Informational => RiskLevel::Low,
        }
    }
}

/// Finding status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    /// Draft - being documented
    #[default]
    Draft,
    /// Pending review
    PendingReview,
    /// Awaiting management response
    AwaitingResponse,
    /// Management has responded
    ManagementResponse,
    /// Remediation planned
    RemediationPlanned,
    /// Remediation in progress
    RemediationInProgress,
    /// Remediation complete - pending validation
    PendingValidation,
    /// Validated and closed
    Closed,
    /// Deferred to future period
    Deferred,
    /// Not applicable / withdrawn
    NotApplicable,
}

/// Remediation plan for a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationPlan {
    /// Plan ID
    pub plan_id: Uuid,
    /// Finding ID this plan addresses
    pub finding_id: Uuid,
    /// Plan description
    pub description: String,
    /// Responsible party
    pub responsible_party: String,
    /// Target completion date
    pub target_date: NaiveDate,
    /// Actual completion date
    pub actual_completion_date: Option<NaiveDate>,
    /// Plan status
    pub status: RemediationStatus,
    /// Validation approach
    pub validation_approach: String,
    /// Validated by auditor ID
    pub validated_by: Option<String>,
    /// Validation date
    pub validated_date: Option<NaiveDate>,
    /// Validation result
    pub validation_result: Option<ValidationResult>,
    /// Milestones
    pub milestones: Vec<RemediationMilestone>,
    /// Notes
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RemediationPlan {
    /// Create a new remediation plan.
    pub fn new(
        finding_id: Uuid,
        description: &str,
        responsible_party: &str,
        target_date: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        Self {
            plan_id: Uuid::new_v4(),
            finding_id,
            description: description.into(),
            responsible_party: responsible_party.into(),
            target_date,
            actual_completion_date: None,
            status: RemediationStatus::Planned,
            validation_approach: String::new(),
            validated_by: None,
            validated_date: None,
            validation_result: None,
            milestones: Vec::new(),
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a milestone.
    pub fn add_milestone(&mut self, description: &str, target_date: NaiveDate) {
        self.milestones.push(RemediationMilestone {
            milestone_id: Uuid::new_v4(),
            description: description.into(),
            target_date,
            completion_date: None,
            status: MilestoneStatus::Pending,
        });
        self.updated_at = Utc::now();
    }

    /// Mark as complete.
    pub fn mark_complete(&mut self, completion_date: NaiveDate) {
        self.actual_completion_date = Some(completion_date);
        self.status = RemediationStatus::Complete;
        self.updated_at = Utc::now();
    }

    /// Add validation result.
    pub fn validate(&mut self, validator: &str, date: NaiveDate, result: ValidationResult) {
        self.validated_by = Some(validator.into());
        self.validated_date = Some(date);
        self.validation_result = Some(result);
        self.status = match result {
            ValidationResult::Effective => RemediationStatus::Validated,
            ValidationResult::PartiallyEffective => RemediationStatus::PartiallyValidated,
            ValidationResult::Ineffective => RemediationStatus::Failed,
        };
        self.updated_at = Utc::now();
    }

    /// Check if plan is overdue.
    pub fn is_overdue(&self) -> bool {
        self.actual_completion_date.is_none()
            && Utc::now().date_naive() > self.target_date
            && !matches!(
                self.status,
                RemediationStatus::Complete | RemediationStatus::Validated
            )
    }
}

/// Remediation plan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    /// Planned but not started
    #[default]
    Planned,
    /// In progress
    InProgress,
    /// Complete - pending validation
    Complete,
    /// Validated as effective
    Validated,
    /// Partially validated
    PartiallyValidated,
    /// Failed validation
    Failed,
    /// Deferred
    Deferred,
}

/// Validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationResult {
    /// Remediation is effective
    Effective,
    /// Partially effective
    PartiallyEffective,
    /// Not effective
    Ineffective,
}

/// Remediation milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationMilestone {
    pub milestone_id: Uuid,
    pub description: String,
    pub target_date: NaiveDate,
    pub completion_date: Option<NaiveDate>,
    pub status: MilestoneStatus,
}

/// Milestone status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    #[default]
    Pending,
    InProgress,
    Complete,
    Overdue,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let finding = AuditFinding::new(
            Uuid::new_v4(),
            FindingType::ControlDeficiency,
            "Inadequate segregation of duties",
        )
        .with_details(
            "Same person can create and approve POs",
            "SOD policy requires separation",
            "Staffing constraints",
            "Risk of unauthorized purchases",
        );

        assert_eq!(finding.finding_type, FindingType::ControlDeficiency);
        assert!(!finding.condition.is_empty());
    }

    #[test]
    fn test_material_weakness() {
        let finding = AuditFinding::new(
            Uuid::new_v4(),
            FindingType::MaterialWeakness,
            "Lack of revenue cut-off controls",
        );

        assert!(finding.is_material_weakness());
        assert!(finding.requires_governance_communication());
        assert_eq!(finding.severity, FindingSeverity::Critical);
    }

    #[test]
    fn test_remediation_plan() {
        let mut plan = RemediationPlan::new(
            Uuid::new_v4(),
            "Implement automated SOD controls",
            "IT Manager",
            NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        );

        plan.add_milestone(
            "Complete requirements gathering",
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );

        assert_eq!(plan.milestones.len(), 1);
        assert_eq!(plan.status, RemediationStatus::Planned);
    }

    #[test]
    fn test_misstatement_total() {
        let finding = AuditFinding::new(
            Uuid::new_v4(),
            FindingType::ImmaterialMisstatement,
            "Revenue overstatement",
        )
        .with_misstatement(
            Some(Decimal::new(10000, 0)),
            Some(Decimal::new(5000, 0)),
            Some(Decimal::new(2000, 0)),
        );

        assert_eq!(finding.total_misstatement(), Decimal::new(17000, 0));
    }
}
