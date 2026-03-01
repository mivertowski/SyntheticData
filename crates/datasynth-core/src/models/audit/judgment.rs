//! Professional judgment models per ISA 200.
//!
//! Professional judgment is essential in applying audit standards and
//! making informed decisions throughout the audit process.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};
use super::engagement::RiskLevel;

/// Professional judgment documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalJudgment {
    /// Unique judgment ID
    pub judgment_id: Uuid,
    /// External reference
    pub judgment_ref: String,
    /// Engagement ID
    pub engagement_id: Uuid,
    /// Type of judgment
    pub judgment_type: JudgmentType,
    /// Subject matter of the judgment
    pub subject: String,
    /// Applicable auditing standards
    pub applicable_standards: Vec<String>,

    // === Structured Documentation ===
    /// Issue or matter requiring judgment
    pub issue_description: String,
    /// Information and factors considered
    pub information_considered: Vec<InformationItem>,
    /// Alternatives evaluated
    pub alternatives_evaluated: Vec<AlternativeEvaluation>,
    /// Professional skepticism applied
    pub skepticism_applied: SkepticismDocumentation,

    // === Conclusion ===
    /// Conclusion reached
    pub conclusion: String,
    /// Rationale for conclusion
    pub rationale: String,
    /// Residual risk or uncertainty
    pub residual_risk: String,
    /// Impact on audit approach
    pub impact_on_audit: Option<String>,

    // === Consultation ===
    /// Was consultation required?
    pub consultation_required: bool,
    /// Consultation details
    pub consultation: Option<ConsultationRecord>,

    // === Sign-offs ===
    /// Preparer user ID
    pub preparer_id: String,
    /// Preparer name
    pub preparer_name: String,
    /// Date prepared
    pub preparer_date: NaiveDate,
    /// Reviewer ID
    pub reviewer_id: Option<String>,
    /// Reviewer name
    pub reviewer_name: Option<String>,
    /// Review date
    pub reviewer_date: Option<NaiveDate>,
    /// Partner concurrence required?
    pub partner_concurrence_required: bool,
    /// Partner concurrence ID
    pub partner_concurrence_id: Option<String>,
    /// Partner concurrence date
    pub partner_concurrence_date: Option<NaiveDate>,

    // === Cross-References ===
    /// Related workpaper IDs
    pub workpaper_refs: Vec<Uuid>,
    /// Related evidence IDs
    pub evidence_refs: Vec<Uuid>,

    // === Status ===
    pub status: JudgmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProfessionalJudgment {
    /// Create a new professional judgment document.
    pub fn new(engagement_id: Uuid, judgment_type: JudgmentType, subject: &str) -> Self {
        let now = Utc::now();
        Self {
            judgment_id: Uuid::new_v4(),
            judgment_ref: format!("JDG-{}-{:03}", now.format("%Y"), 1),
            engagement_id,
            judgment_type,
            subject: subject.into(),
            applicable_standards: judgment_type.default_standards(),
            issue_description: String::new(),
            information_considered: Vec::new(),
            alternatives_evaluated: Vec::new(),
            skepticism_applied: SkepticismDocumentation::default(),
            conclusion: String::new(),
            rationale: String::new(),
            residual_risk: String::new(),
            impact_on_audit: None,
            consultation_required: judgment_type.typically_requires_consultation(),
            consultation: None,
            preparer_id: String::new(),
            preparer_name: String::new(),
            preparer_date: now.date_naive(),
            reviewer_id: None,
            reviewer_name: None,
            reviewer_date: None,
            partner_concurrence_required: judgment_type.requires_partner_concurrence(),
            partner_concurrence_id: None,
            partner_concurrence_date: None,
            workpaper_refs: Vec::new(),
            evidence_refs: Vec::new(),
            status: JudgmentStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the issue description.
    pub fn with_issue(mut self, issue: &str) -> Self {
        self.issue_description = issue.into();
        self
    }

    /// Add information considered.
    pub fn add_information(&mut self, item: InformationItem) {
        self.information_considered.push(item);
        self.updated_at = Utc::now();
    }

    /// Add an alternative evaluation.
    pub fn add_alternative(&mut self, alternative: AlternativeEvaluation) {
        self.alternatives_evaluated.push(alternative);
        self.updated_at = Utc::now();
    }

    /// Set skepticism documentation.
    pub fn with_skepticism(mut self, skepticism: SkepticismDocumentation) -> Self {
        self.skepticism_applied = skepticism;
        self
    }

    /// Set conclusion.
    pub fn with_conclusion(
        mut self,
        conclusion: &str,
        rationale: &str,
        residual_risk: &str,
    ) -> Self {
        self.conclusion = conclusion.into();
        self.rationale = rationale.into();
        self.residual_risk = residual_risk.into();
        self
    }

    /// Set preparer.
    pub fn with_preparer(mut self, id: &str, name: &str, date: NaiveDate) -> Self {
        self.preparer_id = id.into();
        self.preparer_name = name.into();
        self.preparer_date = date;
        self
    }

    /// Add reviewer sign-off.
    pub fn add_review(&mut self, id: &str, name: &str, date: NaiveDate) {
        self.reviewer_id = Some(id.into());
        self.reviewer_name = Some(name.into());
        self.reviewer_date = Some(date);
        self.status = JudgmentStatus::Reviewed;
        self.updated_at = Utc::now();
    }

    /// Add partner concurrence.
    pub fn add_partner_concurrence(&mut self, id: &str, date: NaiveDate) {
        self.partner_concurrence_id = Some(id.into());
        self.partner_concurrence_date = Some(date);
        self.status = JudgmentStatus::Approved;
        self.updated_at = Utc::now();
    }

    /// Add consultation record.
    pub fn add_consultation(&mut self, consultation: ConsultationRecord) {
        self.consultation = Some(consultation);
        self.updated_at = Utc::now();
    }

    /// Check if judgment is fully approved.
    pub fn is_approved(&self) -> bool {
        let reviewer_ok = self.reviewer_id.is_some();
        let partner_ok =
            !self.partner_concurrence_required || self.partner_concurrence_id.is_some();
        let consultation_ok = !self.consultation_required || self.consultation.is_some();
        reviewer_ok && partner_ok && consultation_ok
    }
}

impl ToNodeProperties for ProfessionalJudgment {
    fn node_type_name(&self) -> &'static str {
        "professional_judgment"
    }
    fn node_type_code(&self) -> u16 {
        365
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "judgmentId".into(),
            GraphPropertyValue::String(self.judgment_id.to_string()),
        );
        p.insert(
            "judgmentRef".into(),
            GraphPropertyValue::String(self.judgment_ref.clone()),
        );
        p.insert(
            "engagementId".into(),
            GraphPropertyValue::String(self.engagement_id.to_string()),
        );
        p.insert(
            "judgmentType".into(),
            GraphPropertyValue::String(format!("{:?}", self.judgment_type)),
        );
        p.insert(
            "topic".into(),
            GraphPropertyValue::String(self.subject.clone()),
        );
        p.insert(
            "conclusion".into(),
            GraphPropertyValue::String(self.conclusion.clone()),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "alternativesCount".into(),
            GraphPropertyValue::Int(self.alternatives_evaluated.len() as i64),
        );
        p.insert(
            "consultationRequired".into(),
            GraphPropertyValue::Bool(self.consultation_required),
        );
        p.insert(
            "partnerConcurrenceRequired".into(),
            GraphPropertyValue::Bool(self.partner_concurrence_required),
        );
        p.insert(
            "isApproved".into(),
            GraphPropertyValue::Bool(self.is_approved()),
        );
        p
    }
}

/// Type of professional judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentType {
    /// Materiality determination
    MaterialityDetermination,
    /// Risk assessment judgment
    #[default]
    RiskAssessment,
    /// Control evaluation
    ControlEvaluation,
    /// Accounting estimate evaluation
    EstimateEvaluation,
    /// Going concern assessment
    GoingConcern,
    /// Misstatement evaluation
    MisstatementEvaluation,
    /// Audit report modification decision
    ReportingDecision,
    /// Sampling design judgment
    SamplingDesign,
    /// Related party judgment
    RelatedPartyAssessment,
    /// Subsequent events evaluation
    SubsequentEvents,
    /// Fraud risk assessment
    FraudRiskAssessment,
}

impl JudgmentType {
    /// Get default applicable standards.
    pub fn default_standards(&self) -> Vec<String> {
        match self {
            Self::MaterialityDetermination => vec!["ISA 320".into(), "ISA 450".into()],
            Self::RiskAssessment => vec!["ISA 315".into()],
            Self::ControlEvaluation => vec!["ISA 330".into(), "ISA 265".into()],
            Self::EstimateEvaluation => vec!["ISA 540".into()],
            Self::GoingConcern => vec!["ISA 570".into()],
            Self::MisstatementEvaluation => vec!["ISA 450".into()],
            Self::ReportingDecision => vec!["ISA 700".into(), "ISA 705".into(), "ISA 706".into()],
            Self::SamplingDesign => vec!["ISA 530".into()],
            Self::RelatedPartyAssessment => vec!["ISA 550".into()],
            Self::SubsequentEvents => vec!["ISA 560".into()],
            Self::FraudRiskAssessment => vec!["ISA 240".into()],
        }
    }

    /// Check if this type typically requires consultation.
    pub fn typically_requires_consultation(&self) -> bool {
        matches!(
            self,
            Self::GoingConcern
                | Self::ReportingDecision
                | Self::FraudRiskAssessment
                | Self::EstimateEvaluation
        )
    }

    /// Check if this type requires partner concurrence.
    pub fn requires_partner_concurrence(&self) -> bool {
        matches!(
            self,
            Self::MaterialityDetermination
                | Self::GoingConcern
                | Self::ReportingDecision
                | Self::FraudRiskAssessment
        )
    }
}

/// Information item considered in judgment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationItem {
    /// Item ID
    pub item_id: Uuid,
    /// Description of information
    pub description: String,
    /// Source of information
    pub source: String,
    /// Reliability assessment
    pub reliability: InformationReliability,
    /// Relevance to the judgment
    pub relevance: String,
    /// Weight given in analysis
    pub weight: InformationWeight,
}

impl InformationItem {
    /// Create a new information item.
    pub fn new(
        description: &str,
        source: &str,
        reliability: InformationReliability,
        relevance: &str,
    ) -> Self {
        Self {
            item_id: Uuid::new_v4(),
            description: description.into(),
            source: source.into(),
            reliability,
            relevance: relevance.into(),
            weight: InformationWeight::Moderate,
        }
    }

    /// Set the weight.
    pub fn with_weight(mut self, weight: InformationWeight) -> Self {
        self.weight = weight;
        self
    }
}

/// Reliability of information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InformationReliability {
    /// High reliability
    High,
    /// Medium reliability
    #[default]
    Medium,
    /// Low reliability
    Low,
}

/// Weight given to information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InformationWeight {
    /// High weight - primary factor
    High,
    /// Moderate weight
    #[default]
    Moderate,
    /// Low weight - secondary factor
    Low,
    /// Not weighted - for context only
    Context,
}

/// Alternative evaluation in judgment process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeEvaluation {
    /// Alternative ID
    pub alternative_id: Uuid,
    /// Description of alternative
    pub description: String,
    /// Pros/advantages
    pub pros: Vec<String>,
    /// Cons/disadvantages
    pub cons: Vec<String>,
    /// Risk level if chosen
    pub risk_level: RiskLevel,
    /// Was this alternative selected?
    pub selected: bool,
    /// Reason if not selected
    pub rejection_reason: Option<String>,
}

impl AlternativeEvaluation {
    /// Create a new alternative evaluation.
    pub fn new(description: &str, pros: Vec<String>, cons: Vec<String>) -> Self {
        Self {
            alternative_id: Uuid::new_v4(),
            description: description.into(),
            pros,
            cons,
            risk_level: RiskLevel::Medium,
            selected: false,
            rejection_reason: None,
        }
    }

    /// Mark as selected.
    pub fn select(mut self) -> Self {
        self.selected = true;
        self
    }

    /// Mark as rejected with reason.
    pub fn reject(mut self, reason: &str) -> Self {
        self.selected = false;
        self.rejection_reason = Some(reason.into());
        self
    }
}

/// Documentation of professional skepticism.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkepticismDocumentation {
    /// Contradictory evidence considered
    pub contradictory_evidence_considered: Vec<String>,
    /// Management bias indicators evaluated
    pub management_bias_indicators: Vec<String>,
    /// Alternative explanations explored
    pub alternative_explanations: Vec<String>,
    /// Challenging questions asked
    pub challenging_questions: Vec<String>,
    /// Corroboration obtained
    pub corroboration_obtained: String,
    /// Overall skepticism assessment
    pub skepticism_assessment: String,
}

impl SkepticismDocumentation {
    /// Create skepticism documentation.
    pub fn new(assessment: &str) -> Self {
        Self {
            skepticism_assessment: assessment.into(),
            ..Default::default()
        }
    }

    /// Add contradictory evidence.
    pub fn with_contradictory_evidence(mut self, evidence: Vec<String>) -> Self {
        self.contradictory_evidence_considered = evidence;
        self
    }

    /// Add management bias indicators.
    pub fn with_bias_indicators(mut self, indicators: Vec<String>) -> Self {
        self.management_bias_indicators = indicators;
        self
    }

    /// Add alternative explanations.
    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternative_explanations = alternatives;
        self
    }
}

/// Consultation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationRecord {
    /// Consultation ID
    pub consultation_id: Uuid,
    /// Consultant (internal or external)
    pub consultant: String,
    /// Consultant title/role
    pub consultant_role: String,
    /// Is external consultant?
    pub is_external: bool,
    /// Date of consultation
    pub consultation_date: NaiveDate,
    /// Issue presented
    pub issue_presented: String,
    /// Advice received
    pub advice_received: String,
    /// How advice was applied
    pub advice_application: String,
    /// Consultation conclusion
    pub conclusion: String,
}

impl ConsultationRecord {
    /// Create a new consultation record.
    pub fn new(consultant: &str, role: &str, is_external: bool, date: NaiveDate) -> Self {
        Self {
            consultation_id: Uuid::new_v4(),
            consultant: consultant.into(),
            consultant_role: role.into(),
            is_external,
            consultation_date: date,
            issue_presented: String::new(),
            advice_received: String::new(),
            advice_application: String::new(),
            conclusion: String::new(),
        }
    }

    /// Set the issue and advice.
    pub fn with_content(
        mut self,
        issue: &str,
        advice: &str,
        application: &str,
        conclusion: &str,
    ) -> Self {
        self.issue_presented = issue.into();
        self.advice_received = advice.into();
        self.advice_application = application.into();
        self.conclusion = conclusion.into();
        self
    }
}

/// Judgment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentStatus {
    /// Draft
    #[default]
    Draft,
    /// Pending review
    PendingReview,
    /// Reviewed
    Reviewed,
    /// Pending consultation
    PendingConsultation,
    /// Pending partner concurrence
    PendingPartnerConcurrence,
    /// Approved
    Approved,
    /// Superseded
    Superseded,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_judgment_creation() {
        let judgment = ProfessionalJudgment::new(
            Uuid::new_v4(),
            JudgmentType::MaterialityDetermination,
            "Overall audit materiality",
        )
        .with_issue("Determination of materiality for the 2025 audit")
        .with_conclusion(
            "Materiality set at $1M based on 0.5% of revenue",
            "Revenue is stable metric and primary KPI for stakeholders",
            "Risk of material misstatement below $1M not individually evaluated",
        );

        assert_eq!(
            judgment.judgment_type,
            JudgmentType::MaterialityDetermination
        );
        assert!(judgment.partner_concurrence_required);
    }

    #[test]
    fn test_information_item() {
        let item = InformationItem::new(
            "Prior year financial statements",
            "Audited FS",
            InformationReliability::High,
            "Baseline for trend analysis",
        )
        .with_weight(InformationWeight::High);

        assert_eq!(item.reliability, InformationReliability::High);
        assert_eq!(item.weight, InformationWeight::High);
    }

    #[test]
    fn test_alternative_evaluation() {
        let selected = AlternativeEvaluation::new(
            "Use revenue as materiality base",
            vec!["Stable metric".into(), "Primary KPI".into()],
            vec!["May not capture asset-focused risks".into()],
        )
        .select();

        let rejected = AlternativeEvaluation::new(
            "Use total assets as materiality base",
            vec!["Captures balance sheet risks".into()],
            vec!["Assets less stable".into()],
        )
        .reject("Revenue more relevant to stakeholders");

        assert!(selected.selected);
        assert!(!rejected.selected);
        assert!(rejected.rejection_reason.is_some());
    }

    #[test]
    fn test_judgment_approval() {
        let mut judgment = ProfessionalJudgment::new(
            Uuid::new_v4(),
            JudgmentType::RiskAssessment,
            "Overall risk assessment",
        );

        // Not approved initially
        assert!(!judgment.is_approved());

        // Add reviewer
        judgment.add_review(
            "reviewer1",
            "Senior Manager",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );

        // Risk assessment doesn't require partner concurrence
        assert!(judgment.is_approved());
    }

    #[test]
    fn test_judgment_types() {
        assert!(JudgmentType::GoingConcern.requires_partner_concurrence());
        assert!(JudgmentType::GoingConcern.typically_requires_consultation());
        assert!(!JudgmentType::SamplingDesign.requires_partner_concurrence());
    }
}
