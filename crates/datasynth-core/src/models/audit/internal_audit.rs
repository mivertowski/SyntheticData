//! Internal audit function and report models per ISA 610.
//!
//! ISA 610 governs the use of the work of internal auditors. The external auditor
//! must evaluate the internal audit function's objectivity, competence, and whether
//! a systematic and disciplined approach has been applied.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reporting line of the internal audit function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportingLine {
    /// Reports to the Audit Committee
    #[default]
    AuditCommittee,
    /// Reports to the Board of Directors
    Board,
    /// Reports to the Chief Financial Officer
    CFO,
    /// Reports to the Chief Executive Officer
    CEO,
}

/// ISA 610 assessment of the overall effectiveness of the internal audit function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IaAssessment {
    /// Internal audit is fully effective across all dimensions
    FullyEffective,
    /// Internal audit is largely effective with minor gaps
    #[default]
    LargelyEffective,
    /// Internal audit is partially effective with notable gaps
    PartiallyEffective,
    /// Internal audit is not effective
    Ineffective,
}

/// Objectivity rating of internal audit personnel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObjectivityRating {
    /// High objectivity — strong independence safeguards in place
    #[default]
    High,
    /// Moderate objectivity — some independence concerns exist
    Moderate,
    /// Low objectivity — significant independence concerns exist
    Low,
}

/// Competence rating of the internal audit function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompetenceRating {
    /// High competence — well-qualified staff with relevant expertise
    High,
    /// Moderate competence — adequate but some skill gaps exist
    #[default]
    Moderate,
    /// Low competence — significant skill gaps exist
    Low,
}

/// Extent to which the external auditor relies on the internal audit function's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelianceExtent {
    /// No reliance — external auditor performs all work independently
    NoReliance,
    /// Limited reliance — minor use of internal audit work
    #[default]
    LimitedReliance,
    /// Significant reliance — substantial use of internal audit work
    SignificantReliance,
    /// Full reliance — maximum use of internal audit work permitted by ISA 610
    FullReliance,
}

/// Overall rating assigned by the internal auditor to the audited area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IaReportRating {
    /// Area is operating satisfactorily
    #[default]
    Satisfactory,
    /// Area needs improvement in certain respects
    NeedsImprovement,
    /// Area is unsatisfactory with material control weaknesses
    Unsatisfactory,
}

/// Lifecycle status of an internal audit report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IaReportStatus {
    /// Report is in draft form, pending management review
    #[default]
    Draft,
    /// Report has been finalised and issued
    Final,
    /// Report has been retracted
    Retracted,
}

/// Priority level assigned to an internal audit recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationPriority {
    /// Critical — immediate action required to address a significant control failure
    Critical,
    /// High — urgent action required
    High,
    /// Medium — action required within a reasonable timeframe
    #[default]
    Medium,
    /// Low — action desirable but not urgent
    Low,
}

/// Status of a management action plan in response to a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionPlanStatus {
    /// Action plan has not yet been started
    #[default]
    Open,
    /// Action plan is in progress
    InProgress,
    /// Action plan has been fully implemented
    Implemented,
    /// Action plan has passed its target date without implementation
    Overdue,
}

/// External auditor's assessment of the reliability of a specific piece of internal audit work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IaWorkAssessment {
    /// Work is reliable and can be used without significant modification
    Reliable,
    /// Work is partially reliable — some additional procedures required
    #[default]
    PartiallyReliable,
    /// Work is not reliable — cannot be used by the external auditor
    Unreliable,
}

/// The internal audit function of the entity being audited (ISA 610).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuditFunction {
    /// Unique function ID
    pub function_id: Uuid,
    /// Human-readable reference (format: "IAF-{first 8 hex chars of function_id}")
    pub function_ref: String,
    /// Engagement this assessment relates to
    pub engagement_id: Uuid,

    // === Organisation ===
    /// Name of the internal audit department
    pub department_name: String,
    /// Reporting line of the head of internal audit
    pub reporting_line: ReportingLine,
    /// Name of the head of internal audit
    pub head_of_ia: String,
    /// Professional qualifications held by the head of internal audit
    pub head_of_ia_qualifications: Vec<String>,
    /// Number of internal audit staff (FTE)
    pub staff_count: u32,
    /// Percentage of total risk universe covered by the annual audit plan
    pub annual_plan_coverage: f64,
    /// Whether a formal quality assurance and improvement programme exists
    pub quality_assurance: bool,

    // === ISA 610 Assessment ===
    /// Overall assessment of the internal audit function per ISA 610
    pub isa_610_assessment: IaAssessment,
    /// Assessment of objectivity
    pub objectivity_rating: ObjectivityRating,
    /// Assessment of technical competence
    pub competence_rating: CompetenceRating,
    /// Whether a systematic and disciplined approach is applied
    pub systematic_discipline: bool,

    // === Reliance Decision ===
    /// Extent to which the external auditor plans to rely on internal audit work
    pub reliance_extent: RelianceExtent,
    /// Specific audit areas where reliance will be placed
    pub reliance_areas: Vec<String>,
    /// Whether direct assistance from internal audit staff will be used
    pub direct_assistance: bool,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InternalAuditFunction {
    /// Create a new internal audit function record with sensible defaults.
    pub fn new(
        engagement_id: Uuid,
        department_name: impl Into<String>,
        head_of_ia: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let function_ref = format!("IAF-{}", &id.simple().to_string()[..8]);
        Self {
            function_id: id,
            function_ref,
            engagement_id,
            department_name: department_name.into(),
            reporting_line: ReportingLine::AuditCommittee,
            head_of_ia: head_of_ia.into(),
            head_of_ia_qualifications: Vec::new(),
            staff_count: 0,
            annual_plan_coverage: 0.0,
            quality_assurance: false,
            isa_610_assessment: IaAssessment::LargelyEffective,
            objectivity_rating: ObjectivityRating::High,
            competence_rating: CompetenceRating::Moderate,
            systematic_discipline: true,
            reliance_extent: RelianceExtent::LimitedReliance,
            reliance_areas: Vec::new(),
            direct_assistance: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A recommendation raised in an internal audit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IaRecommendation {
    /// Unique recommendation ID
    pub recommendation_id: Uuid,
    /// Description of the recommendation
    pub description: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Management's response to the recommendation
    pub management_response: Option<String>,
}

/// Management's action plan in response to an internal audit recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// Unique plan ID
    pub plan_id: Uuid,
    /// The recommendation this plan addresses
    pub recommendation_id: Uuid,
    /// Description of the planned action
    pub description: String,
    /// Party responsible for implementing the action
    pub responsible_party: String,
    /// Target implementation date
    pub target_date: NaiveDate,
    /// Current status of the action plan
    pub status: ActionPlanStatus,
}

/// An internal audit report for a specific audit area (ISA 610).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuditReport {
    /// Unique report ID
    pub report_id: Uuid,
    /// Human-readable reference (format: "IAR-{first 8 hex chars of report_id}")
    pub report_ref: String,
    /// Engagement this report is associated with
    pub engagement_id: Uuid,
    /// The internal audit function that produced this report
    pub ia_function_id: Uuid,

    // === Report Header ===
    /// Report title
    pub report_title: String,
    /// Audit area covered by the report
    pub audit_area: String,
    /// Date the report was issued
    pub report_date: NaiveDate,
    /// Start of the period covered by the audit
    pub period_start: NaiveDate,
    /// End of the period covered by the audit
    pub period_end: NaiveDate,

    // === Scope & Methodology ===
    /// Description of the audit scope
    pub scope_description: String,
    /// Methodology applied during the audit
    pub methodology: String,

    // === Findings & Ratings ===
    /// Overall rating of the audited area
    pub overall_rating: IaReportRating,
    /// Total number of findings raised
    pub findings_count: u32,
    /// Number of high-risk findings
    pub high_risk_findings: u32,
    /// Recommendations raised in the report
    pub recommendations: Vec<IaRecommendation>,
    /// Management action plans in response to the recommendations
    pub management_action_plans: Vec<ActionPlan>,

    // === Status ===
    /// Current lifecycle status of the report
    pub status: IaReportStatus,

    // === External Auditor's Assessment ===
    /// External auditor's assessment of the reliability of this report's work
    pub external_auditor_assessment: Option<IaWorkAssessment>,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InternalAuditReport {
    /// Create a new internal audit report.
    pub fn new(
        engagement_id: Uuid,
        ia_function_id: Uuid,
        report_title: impl Into<String>,
        audit_area: impl Into<String>,
        report_date: NaiveDate,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let report_ref = format!("IAR-{}", &id.simple().to_string()[..8]);
        Self {
            report_id: id,
            report_ref,
            engagement_id,
            ia_function_id,
            report_title: report_title.into(),
            audit_area: audit_area.into(),
            report_date,
            period_start,
            period_end,
            scope_description: String::new(),
            methodology: String::new(),
            overall_rating: IaReportRating::Satisfactory,
            findings_count: 0,
            high_risk_findings: 0,
            recommendations: Vec::new(),
            management_action_plans: Vec::new(),
            status: IaReportStatus::Draft,
            external_auditor_assessment: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_new_ia_function() {
        let eng = Uuid::new_v4();
        let iaf = InternalAuditFunction::new(eng, "Group Internal Audit", "Jane Smith");

        assert_eq!(iaf.engagement_id, eng);
        assert_eq!(iaf.department_name, "Group Internal Audit");
        assert_eq!(iaf.head_of_ia, "Jane Smith");
        assert_eq!(iaf.reporting_line, ReportingLine::AuditCommittee);
        assert_eq!(iaf.isa_610_assessment, IaAssessment::LargelyEffective);
        assert_eq!(iaf.objectivity_rating, ObjectivityRating::High);
        assert_eq!(iaf.competence_rating, CompetenceRating::Moderate);
        assert_eq!(iaf.reliance_extent, RelianceExtent::LimitedReliance);
        assert!(iaf.systematic_discipline);
        assert!(!iaf.direct_assistance);
        assert!(iaf.function_ref.starts_with("IAF-"));
        assert_eq!(iaf.function_ref.len(), 12); // "IAF-" + 8 hex chars
    }

    #[test]
    fn test_new_ia_report() {
        let eng = Uuid::new_v4();
        let func = Uuid::new_v4();
        let report = InternalAuditReport::new(
            eng,
            func,
            "Procurement Process Review",
            "Procurement",
            sample_date(2025, 3, 31),
            sample_date(2025, 1, 1),
            sample_date(2025, 12, 31),
        );

        assert_eq!(report.engagement_id, eng);
        assert_eq!(report.ia_function_id, func);
        assert_eq!(report.report_title, "Procurement Process Review");
        assert_eq!(report.audit_area, "Procurement");
        assert_eq!(report.overall_rating, IaReportRating::Satisfactory);
        assert_eq!(report.status, IaReportStatus::Draft);
        assert_eq!(report.findings_count, 0);
        assert!(report.recommendations.is_empty());
        assert!(report.external_auditor_assessment.is_none());
        assert!(report.report_ref.starts_with("IAR-"));
        assert_eq!(report.report_ref.len(), 12); // "IAR-" + 8 hex chars
    }

    #[test]
    fn test_reporting_line_serde() {
        let variants = [
            ReportingLine::AuditCommittee,
            ReportingLine::Board,
            ReportingLine::CFO,
            ReportingLine::CEO,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: ReportingLine = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&ReportingLine::AuditCommittee).unwrap(),
            "\"audit_committee\""
        );
    }

    #[test]
    fn test_ia_assessment_serde() {
        let variants = [
            IaAssessment::FullyEffective,
            IaAssessment::LargelyEffective,
            IaAssessment::PartiallyEffective,
            IaAssessment::Ineffective,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: IaAssessment = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&IaAssessment::FullyEffective).unwrap(),
            "\"fully_effective\""
        );
    }

    #[test]
    fn test_reliance_extent_serde() {
        let variants = [
            RelianceExtent::NoReliance,
            RelianceExtent::LimitedReliance,
            RelianceExtent::SignificantReliance,
            RelianceExtent::FullReliance,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: RelianceExtent = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&RelianceExtent::SignificantReliance).unwrap(),
            "\"significant_reliance\""
        );
    }

    #[test]
    fn test_ia_report_status_serde() {
        let variants = [
            IaReportStatus::Draft,
            IaReportStatus::Final,
            IaReportStatus::Retracted,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: IaReportStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&IaReportStatus::Final).unwrap(),
            "\"final\""
        );
    }

    #[test]
    fn test_ia_report_rating_serde() {
        let variants = [
            IaReportRating::Satisfactory,
            IaReportRating::NeedsImprovement,
            IaReportRating::Unsatisfactory,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: IaReportRating = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&IaReportRating::NeedsImprovement).unwrap(),
            "\"needs_improvement\""
        );
    }

    #[test]
    fn test_recommendation_priority_serde() {
        let variants = [
            RecommendationPriority::Critical,
            RecommendationPriority::High,
            RecommendationPriority::Medium,
            RecommendationPriority::Low,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: RecommendationPriority = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&RecommendationPriority::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn test_action_plan_status_serde() {
        let variants = [
            ActionPlanStatus::Open,
            ActionPlanStatus::InProgress,
            ActionPlanStatus::Implemented,
            ActionPlanStatus::Overdue,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: ActionPlanStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&ActionPlanStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
    }

    #[test]
    fn test_ia_work_assessment_serde() {
        let variants = [
            IaWorkAssessment::Reliable,
            IaWorkAssessment::PartiallyReliable,
            IaWorkAssessment::Unreliable,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: IaWorkAssessment = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&IaWorkAssessment::PartiallyReliable).unwrap(),
            "\"partially_reliable\""
        );
    }
}
