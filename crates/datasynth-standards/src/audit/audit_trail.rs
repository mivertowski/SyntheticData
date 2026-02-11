//! Audit Trail for Complete Traceability.
//!
//! Provides structures for maintaining a complete audit trail from
//! risk assessment through to conclusions, enabling traceability
//! across the entire audit process.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::isa_reference::IsaRequirement;

/// Complete audit trail for an assertion or audit area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Unique trail identifier.
    pub trail_id: Uuid,

    /// Engagement ID.
    pub engagement_id: Uuid,

    /// Account or audit area.
    pub account_or_area: String,

    /// Assertion being addressed.
    pub assertion: Assertion,

    /// Risk assessment phase.
    pub risk_assessment: RiskTrailNode,

    /// Planned audit responses.
    pub planned_responses: Vec<ResponseTrailNode>,

    /// Procedures actually performed.
    pub procedures_performed: Vec<ProcedureTrailNode>,

    /// Evidence obtained.
    pub evidence_obtained: Vec<EvidenceTrailNode>,

    /// Conclusion reached.
    pub conclusion: ConclusionTrailNode,

    /// Gaps identified in the audit trail.
    pub gaps_identified: Vec<TrailGap>,

    /// ISA requirements addressed.
    pub isa_coverage: Vec<IsaRequirement>,
}

impl AuditTrail {
    /// Create a new audit trail.
    pub fn new(
        engagement_id: Uuid,
        account_or_area: impl Into<String>,
        assertion: Assertion,
    ) -> Self {
        Self {
            trail_id: Uuid::now_v7(),
            engagement_id,
            account_or_area: account_or_area.into(),
            assertion,
            risk_assessment: RiskTrailNode::default(),
            planned_responses: Vec::new(),
            procedures_performed: Vec::new(),
            evidence_obtained: Vec::new(),
            conclusion: ConclusionTrailNode::default(),
            gaps_identified: Vec::new(),
            isa_coverage: Vec::new(),
        }
    }

    /// Check if trail is complete (no gaps).
    pub fn is_complete(&self) -> bool {
        self.gaps_identified.is_empty()
            && self.conclusion.conclusion_reached
            && !self.evidence_obtained.is_empty()
    }

    /// Identify gaps in the audit trail.
    pub fn identify_gaps(&mut self) {
        self.gaps_identified.clear();

        // Check for risk assessment gaps
        if !self.risk_assessment.risk_identified {
            self.gaps_identified.push(TrailGap {
                gap_type: GapType::RiskAssessment,
                description: "Risk of material misstatement not documented".to_string(),
                severity: GapSeverity::High,
                remediation_required: true,
            });
        }

        // Check for response gaps
        if self.planned_responses.is_empty() {
            self.gaps_identified.push(TrailGap {
                gap_type: GapType::PlannedResponse,
                description: "No audit responses planned".to_string(),
                severity: GapSeverity::High,
                remediation_required: true,
            });
        }

        // Check for procedures gap
        if self.procedures_performed.is_empty() {
            self.gaps_identified.push(TrailGap {
                gap_type: GapType::ProceduresPerformed,
                description: "No audit procedures performed".to_string(),
                severity: GapSeverity::High,
                remediation_required: true,
            });
        }

        // Check for evidence gap
        if self.evidence_obtained.is_empty() {
            self.gaps_identified.push(TrailGap {
                gap_type: GapType::Evidence,
                description: "No audit evidence documented".to_string(),
                severity: GapSeverity::High,
                remediation_required: true,
            });
        }

        // Check for conclusion gap
        if !self.conclusion.conclusion_reached {
            self.gaps_identified.push(TrailGap {
                gap_type: GapType::Conclusion,
                description: "No conclusion documented".to_string(),
                severity: GapSeverity::High,
                remediation_required: true,
            });
        }

        // Check response-to-risk linkage
        for response in &self.planned_responses {
            if !self
                .procedures_performed
                .iter()
                .any(|p| p.response_id == Some(response.response_id))
            {
                self.gaps_identified.push(TrailGap {
                    gap_type: GapType::Linkage,
                    description: format!(
                        "Planned response '{}' not linked to performed procedure",
                        response.response_description
                    ),
                    severity: GapSeverity::Medium,
                    remediation_required: true,
                });
            }
        }
    }
}

/// Financial statement assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Assertion {
    // Transaction assertions
    /// Transactions and events occurred.
    #[default]
    Occurrence,
    /// All transactions are recorded (none omitted).
    Completeness,
    /// Transactions are recorded in the correct period.
    Cutoff,
    /// Transactions are recorded at correct amounts.
    Accuracy,
    /// Transactions are recorded in proper accounts.
    Classification,

    // Balance assertions
    /// Assets and liabilities exist.
    Existence,
    /// Entity has rights to assets and obligations for liabilities.
    RightsAndObligations,
    /// Assets and liabilities are recorded at appropriate amounts.
    Valuation,

    // Disclosure assertions
    /// Disclosures are understandable.
    Understandability,
    /// Information is appropriately classified and described.
    ClassificationAndUnderstandability,
    /// Amounts are accurate and appropriately measured.
    AccuracyAndValuation,
}

impl std::fmt::Display for Assertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Occurrence => write!(f, "Occurrence"),
            Self::Completeness => write!(f, "Completeness"),
            Self::Cutoff => write!(f, "Cutoff"),
            Self::Accuracy => write!(f, "Accuracy"),
            Self::Classification => write!(f, "Classification"),
            Self::Existence => write!(f, "Existence"),
            Self::RightsAndObligations => write!(f, "Rights and Obligations"),
            Self::Valuation => write!(f, "Valuation"),
            Self::Understandability => write!(f, "Understandability"),
            Self::ClassificationAndUnderstandability => {
                write!(f, "Classification and Understandability")
            }
            Self::AccuracyAndValuation => write!(f, "Accuracy and Valuation"),
        }
    }
}

/// Risk assessment trail node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskTrailNode {
    /// Risk identified.
    pub risk_identified: bool,

    /// Risk description.
    pub risk_description: String,

    /// Risk level (inherent risk).
    pub inherent_risk_level: AuditRiskLevel,

    /// Control risk level.
    pub control_risk_level: AuditRiskLevel,

    /// Combined assessment (RoMM).
    pub romm_level: AuditRiskLevel,

    /// Significant risk designation.
    pub is_significant_risk: bool,

    /// Fraud risk identified.
    pub fraud_risk_identified: bool,

    /// Understanding of entity obtained.
    pub understanding_documented: bool,

    /// Internal controls evaluated.
    pub controls_evaluated: bool,

    /// Risk assessment workpaper reference.
    pub workpaper_reference: Option<String>,
}

/// Risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuditRiskLevel {
    Low,
    #[default]
    Medium,
    High,
    Maximum,
}

impl std::fmt::Display for AuditRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Maximum => write!(f, "Maximum"),
        }
    }
}

/// Planned response trail node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTrailNode {
    /// Response ID.
    pub response_id: Uuid,

    /// Response description.
    pub response_description: String,

    /// Type of response.
    pub response_type: ResponseType,

    /// Risk being addressed.
    pub risk_addressed: String,

    /// Nature of procedure.
    pub procedure_nature: ProcedureNature,

    /// Timing of procedure.
    pub procedure_timing: ProcedureTiming,

    /// Extent of procedure.
    pub procedure_extent: String,

    /// Staff assigned.
    pub staff_assigned: Vec<String>,

    /// Budgeted hours.
    pub budgeted_hours: Option<f64>,
}

impl ResponseTrailNode {
    /// Create a new response trail node.
    pub fn new(response_description: impl Into<String>, response_type: ResponseType) -> Self {
        Self {
            response_id: Uuid::now_v7(),
            response_description: response_description.into(),
            response_type,
            risk_addressed: String::new(),
            procedure_nature: ProcedureNature::Substantive,
            procedure_timing: ProcedureTiming::YearEnd,
            procedure_extent: String::new(),
            staff_assigned: Vec::new(),
            budgeted_hours: None,
        }
    }
}

/// Type of audit response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// Test of controls.
    TestOfControls,
    /// Substantive procedures.
    #[default]
    Substantive,
    /// Combined approach.
    Combined,
    /// Overall response.
    Overall,
}

/// Nature of audit procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureNature {
    /// Inspection of records/documents.
    Inspection,
    /// Physical observation.
    Observation,
    /// External confirmation.
    Confirmation,
    /// Recalculation.
    Recalculation,
    /// Reperformance.
    Reperformance,
    /// Analytical procedures.
    Analytical,
    /// Inquiry.
    Inquiry,
    #[default]
    /// Substantive testing.
    Substantive,
}

/// Timing of audit procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureTiming {
    /// At interim date.
    Interim,
    /// At year-end.
    #[default]
    YearEnd,
    /// Roll-forward from interim to year-end.
    RollForward,
    /// Throughout the period.
    Continuous,
}

/// Procedure performed trail node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTrailNode {
    /// Procedure ID.
    pub procedure_id: Uuid,

    /// Linked response ID.
    pub response_id: Option<Uuid>,

    /// Procedure description.
    pub procedure_description: String,

    /// Date performed.
    pub date_performed: chrono::NaiveDate,

    /// Performed by.
    pub performed_by: String,

    /// Reviewed by.
    pub reviewed_by: Option<String>,

    /// Hours spent.
    pub hours_spent: Option<f64>,

    /// Population tested.
    pub population_size: Option<u64>,

    /// Sample size.
    pub sample_size: Option<u64>,

    /// Exceptions found.
    pub exceptions_found: u32,

    /// Results summary.
    pub results_summary: String,

    /// Workpaper reference.
    pub workpaper_reference: Option<String>,
}

impl ProcedureTrailNode {
    /// Create a new procedure trail node.
    pub fn new(
        procedure_description: impl Into<String>,
        date_performed: chrono::NaiveDate,
        performed_by: impl Into<String>,
    ) -> Self {
        Self {
            procedure_id: Uuid::now_v7(),
            response_id: None,
            procedure_description: procedure_description.into(),
            date_performed,
            performed_by: performed_by.into(),
            reviewed_by: None,
            hours_spent: None,
            population_size: None,
            sample_size: None,
            exceptions_found: 0,
            results_summary: String::new(),
            workpaper_reference: None,
        }
    }
}

/// Evidence trail node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTrailNode {
    /// Evidence ID.
    pub evidence_id: Uuid,

    /// Linked procedure ID.
    pub procedure_id: Option<Uuid>,

    /// Evidence type.
    pub evidence_type: EvidenceType,

    /// Evidence description.
    pub evidence_description: String,

    /// Source of evidence.
    pub source: EvidenceSource,

    /// Reliability assessment.
    pub reliability: EvidenceReliability,

    /// Relevance to assertion.
    pub relevance: EvidenceRelevance,

    /// Document reference.
    pub document_reference: Option<String>,

    /// Date obtained.
    pub date_obtained: chrono::NaiveDate,
}

impl EvidenceTrailNode {
    /// Create a new evidence trail node.
    pub fn new(
        evidence_type: EvidenceType,
        evidence_description: impl Into<String>,
        source: EvidenceSource,
    ) -> Self {
        Self {
            evidence_id: Uuid::now_v7(),
            procedure_id: None,
            evidence_type,
            evidence_description: evidence_description.into(),
            source,
            reliability: EvidenceReliability::Moderate,
            relevance: EvidenceRelevance::Relevant,
            document_reference: None,
            date_obtained: chrono::Utc::now().date_naive(),
        }
    }
}

/// Type of audit evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Physical examination.
    Physical,
    /// External confirmation.
    Confirmation,
    /// Documentary - external source.
    DocumentaryExternal,
    /// Documentary - internal source.
    DocumentaryInternal,
    /// Mathematical recalculation.
    Recalculation,
    /// Analytical evidence.
    Analytical,
    /// Management representation.
    Representation,
    /// Observation.
    Observation,
    /// Inquiry response.
    Inquiry,
}

/// Source of audit evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// External third party.
    ExternalThirdParty,
    /// External - client's records of external transactions.
    ExternalClientRecords,
    /// Internal to the entity.
    #[default]
    Internal,
    /// Auditor-generated.
    AuditorGenerated,
}

/// Reliability of audit evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceReliability {
    /// Low reliability.
    Low,
    /// Moderate reliability.
    #[default]
    Moderate,
    /// High reliability.
    High,
}

/// Relevance of audit evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelevance {
    /// Not relevant.
    NotRelevant,
    /// Partially relevant.
    PartiallyRelevant,
    /// Relevant.
    #[default]
    Relevant,
    /// Directly relevant.
    DirectlyRelevant,
}

/// Conclusion trail node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConclusionTrailNode {
    /// Conclusion reached.
    pub conclusion_reached: bool,

    /// Conclusion text.
    pub conclusion_text: String,

    /// Conclusion type.
    pub conclusion_type: ConclusionType,

    /// Misstatements identified.
    pub misstatements_identified: Vec<MisstatementReference>,

    /// Sufficient appropriate evidence obtained.
    pub sufficient_evidence: bool,

    /// Further procedures required.
    pub further_procedures_required: bool,

    /// Reference to summary memo.
    pub summary_memo_reference: Option<String>,

    /// Preparer.
    pub prepared_by: String,

    /// Reviewer.
    pub reviewed_by: Option<String>,

    /// Date concluded.
    pub conclusion_date: Option<chrono::NaiveDate>,
}

/// Type of conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionType {
    /// No exceptions noted.
    #[default]
    Satisfactory,
    /// Minor issues, not material.
    SatisfactoryWithMinorIssues,
    /// Potential misstatement identified.
    PotentialMisstatement,
    /// Misstatement identified.
    MisstatementIdentified,
    /// Unable to conclude.
    UnableToConclude,
}

/// Misstatement reference in conclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisstatementReference {
    /// Misstatement ID.
    pub misstatement_id: Uuid,

    /// Description.
    pub description: String,

    /// Amount (if quantified).
    pub amount: Option<rust_decimal::Decimal>,

    /// Is it factual, judgmental, or projected.
    pub misstatement_type: MisstatementType,
}

/// Type of misstatement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MisstatementType {
    /// Factual misstatement - no doubt.
    Factual,
    /// Judgmental misstatement - differences in estimates.
    Judgmental,
    /// Projected misstatement - extrapolated from sample.
    Projected,
}

/// Gap in the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailGap {
    /// Type of gap.
    pub gap_type: GapType,

    /// Description of the gap.
    pub description: String,

    /// Severity of the gap.
    pub severity: GapSeverity,

    /// Whether remediation is required.
    pub remediation_required: bool,
}

/// Type of audit trail gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    /// Gap in risk assessment.
    RiskAssessment,
    /// Gap in planned responses.
    PlannedResponse,
    /// Gap in procedures performed.
    ProceduresPerformed,
    /// Gap in evidence.
    Evidence,
    /// Gap in conclusion.
    Conclusion,
    /// Gap in linkage between elements.
    Linkage,
    /// Gap in documentation.
    Documentation,
}

/// Severity of audit trail gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    /// Low severity - documentation issue only.
    Low,
    /// Medium severity - could affect conclusions.
    Medium,
    /// High severity - significant audit quality concern.
    High,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_trail_creation() {
        let trail = AuditTrail::new(Uuid::now_v7(), "Revenue", Assertion::Occurrence);

        assert_eq!(trail.account_or_area, "Revenue");
        assert_eq!(trail.assertion, Assertion::Occurrence);
        assert!(!trail.is_complete());
    }

    #[test]
    fn test_gap_identification() {
        let mut trail = AuditTrail::new(Uuid::now_v7(), "Inventory", Assertion::Existence);

        trail.identify_gaps();

        // Should have gaps for all elements
        assert!(!trail.gaps_identified.is_empty());
        assert!(trail
            .gaps_identified
            .iter()
            .any(|g| matches!(g.gap_type, GapType::RiskAssessment)));
        assert!(trail
            .gaps_identified
            .iter()
            .any(|g| matches!(g.gap_type, GapType::Evidence)));
    }

    #[test]
    fn test_complete_trail() {
        let mut trail = AuditTrail::new(Uuid::now_v7(), "Cash", Assertion::Existence);

        // Populate all elements
        trail.risk_assessment.risk_identified = true;
        trail.risk_assessment.risk_description = "Risk of misappropriation".to_string();

        let response =
            ResponseTrailNode::new("Perform bank reconciliation", ResponseType::Substantive);
        let response_id = response.response_id;
        trail.planned_responses.push(response);

        let mut procedure = ProcedureTrailNode::new(
            "Reconciled bank to GL",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            "Auditor A",
        );
        procedure.response_id = Some(response_id);
        trail.procedures_performed.push(procedure);

        trail.evidence_obtained.push(EvidenceTrailNode::new(
            EvidenceType::DocumentaryExternal,
            "Bank statement obtained",
            EvidenceSource::ExternalThirdParty,
        ));

        trail.conclusion.conclusion_reached = true;
        trail.conclusion.conclusion_type = ConclusionType::Satisfactory;
        trail.conclusion.sufficient_evidence = true;

        trail.identify_gaps();

        assert!(trail.is_complete());
        assert!(trail.gaps_identified.is_empty());
    }
}
