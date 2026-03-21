//! Audit engagement models per ISA 210 and ISA 220.
//!
//! An audit engagement represents the entire audit project including
//! planning, fieldwork, and reporting phases.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Audit engagement representing an audit project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEngagement {
    /// Unique engagement ID
    pub engagement_id: Uuid,
    /// External engagement reference (e.g., "AUD-2025-001")
    pub engagement_ref: String,
    /// Client entity being audited
    pub client_entity_id: String,
    /// Client name
    pub client_name: String,
    /// Type of engagement
    pub engagement_type: EngagementType,
    /// Fiscal year being audited
    pub fiscal_year: u16,
    /// Fiscal period end date
    pub period_end_date: NaiveDate,

    // === Materiality ===
    /// Overall materiality
    pub materiality: Decimal,
    /// Performance materiality (typically 50-75% of materiality)
    pub performance_materiality: Decimal,
    /// Clearly trivial threshold (typically 3-5% of materiality)
    pub clearly_trivial: Decimal,
    /// Materiality basis (e.g., "Total Revenue", "Total Assets")
    pub materiality_basis: String,
    /// Materiality percentage applied
    pub materiality_percentage: f64,

    // === Timeline ===
    /// Planning phase start date
    pub planning_start: NaiveDate,
    /// Planning phase end date
    pub planning_end: NaiveDate,
    /// Fieldwork start date
    pub fieldwork_start: NaiveDate,
    /// Fieldwork end date
    pub fieldwork_end: NaiveDate,
    /// Completion phase start date
    pub completion_start: NaiveDate,
    /// Expected report date
    pub report_date: NaiveDate,

    // === Team ===
    /// Engagement partner ID
    pub engagement_partner_id: String,
    /// Engagement partner name
    pub engagement_partner_name: String,
    /// Engagement manager ID
    pub engagement_manager_id: String,
    /// Engagement manager name
    pub engagement_manager_name: String,
    /// All team member IDs
    pub team_member_ids: Vec<String>,

    // === Status ===
    /// Current engagement status
    pub status: EngagementStatus,
    /// Current phase
    pub current_phase: EngagementPhase,

    // === Risk Assessment Summary ===
    /// Overall audit risk assessment
    pub overall_audit_risk: RiskLevel,
    /// Number of significant risks identified
    pub significant_risk_count: u32,
    /// Fraud risk assessment level
    pub fraud_risk_level: RiskLevel,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // === Scope ===
    /// Audit scope identifier (FK → AuditScope.id). Populated during planning phase.
    #[serde(default)]
    pub scope_id: Option<String>,
}

impl AuditEngagement {
    /// Create a new audit engagement.
    pub fn new(
        client_entity_id: &str,
        client_name: &str,
        engagement_type: EngagementType,
        fiscal_year: u16,
        period_end_date: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        Self {
            engagement_id: Uuid::new_v4(),
            engagement_ref: format!("AUD-{}-{:03}", fiscal_year, 1),
            client_entity_id: client_entity_id.into(),
            client_name: client_name.into(),
            engagement_type,
            fiscal_year,
            period_end_date,
            materiality: Decimal::ZERO,
            performance_materiality: Decimal::ZERO,
            clearly_trivial: Decimal::ZERO,
            materiality_basis: String::new(),
            materiality_percentage: 0.0,
            planning_start: period_end_date,
            planning_end: period_end_date,
            fieldwork_start: period_end_date,
            fieldwork_end: period_end_date,
            completion_start: period_end_date,
            report_date: period_end_date,
            engagement_partner_id: String::new(),
            engagement_partner_name: String::new(),
            engagement_manager_id: String::new(),
            engagement_manager_name: String::new(),
            team_member_ids: Vec::new(),
            status: EngagementStatus::Planning,
            current_phase: EngagementPhase::Planning,
            overall_audit_risk: RiskLevel::Medium,
            significant_risk_count: 0,
            fraud_risk_level: RiskLevel::Low,
            created_at: now,
            updated_at: now,
            scope_id: None,
        }
    }

    /// Set materiality values.
    pub fn with_materiality(
        mut self,
        materiality: Decimal,
        performance_materiality_factor: f64,
        clearly_trivial_factor: f64,
        basis: &str,
        percentage: f64,
    ) -> Self {
        self.materiality = materiality;
        self.performance_materiality =
            materiality * Decimal::try_from(performance_materiality_factor).unwrap_or_default();
        self.clearly_trivial =
            materiality * Decimal::try_from(clearly_trivial_factor).unwrap_or_default();
        self.materiality_basis = basis.into();
        self.materiality_percentage = percentage;
        self
    }

    /// Set the engagement team.
    pub fn with_team(
        mut self,
        partner_id: &str,
        partner_name: &str,
        manager_id: &str,
        manager_name: &str,
        team_members: Vec<String>,
    ) -> Self {
        self.engagement_partner_id = partner_id.into();
        self.engagement_partner_name = partner_name.into();
        self.engagement_manager_id = manager_id.into();
        self.engagement_manager_name = manager_name.into();
        self.team_member_ids = team_members;
        self
    }

    /// Set engagement timeline.
    pub fn with_timeline(
        mut self,
        planning_start: NaiveDate,
        planning_end: NaiveDate,
        fieldwork_start: NaiveDate,
        fieldwork_end: NaiveDate,
        completion_start: NaiveDate,
        report_date: NaiveDate,
    ) -> Self {
        self.planning_start = planning_start;
        self.planning_end = planning_end;
        self.fieldwork_start = fieldwork_start;
        self.fieldwork_end = fieldwork_end;
        self.completion_start = completion_start;
        self.report_date = report_date;
        self
    }

    /// Advance to the next phase.
    pub fn advance_phase(&mut self) {
        self.current_phase = match self.current_phase {
            EngagementPhase::Planning => EngagementPhase::RiskAssessment,
            EngagementPhase::RiskAssessment => EngagementPhase::ControlTesting,
            EngagementPhase::ControlTesting => EngagementPhase::SubstantiveTesting,
            EngagementPhase::SubstantiveTesting => EngagementPhase::Completion,
            EngagementPhase::Completion => EngagementPhase::Reporting,
            EngagementPhase::Reporting => EngagementPhase::Reporting,
        };
        self.updated_at = Utc::now();
    }

    /// Check if the engagement is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            EngagementStatus::Complete | EngagementStatus::Archived
        )
    }

    /// Calculate days until report date.
    pub fn days_until_report(&self, as_of: NaiveDate) -> i64 {
        (self.report_date - as_of).num_days()
    }
}

impl ToNodeProperties for AuditEngagement {
    fn node_type_name(&self) -> &'static str {
        "audit_engagement"
    }
    fn node_type_code(&self) -> u16 {
        360
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "engagementId".into(),
            GraphPropertyValue::String(self.engagement_id.to_string()),
        );
        p.insert(
            "engagementRef".into(),
            GraphPropertyValue::String(self.engagement_ref.clone()),
        );
        p.insert(
            "clientName".into(),
            GraphPropertyValue::String(self.client_name.clone()),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.client_entity_id.clone()),
        );
        p.insert(
            "engagementType".into(),
            GraphPropertyValue::String(format!("{:?}", self.engagement_type)),
        );
        p.insert(
            "fiscalYear".into(),
            GraphPropertyValue::Int(self.fiscal_year as i64),
        );
        p.insert(
            "periodEndDate".into(),
            GraphPropertyValue::Date(self.period_end_date),
        );
        p.insert(
            "materiality".into(),
            GraphPropertyValue::Decimal(self.materiality),
        );
        p.insert(
            "performanceMateriality".into(),
            GraphPropertyValue::Decimal(self.performance_materiality),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "currentPhase".into(),
            GraphPropertyValue::String(self.current_phase.display_name().into()),
        );
        p.insert(
            "overallAuditRisk".into(),
            GraphPropertyValue::String(format!("{:?}", self.overall_audit_risk)),
        );
        p.insert(
            "significantRiskCount".into(),
            GraphPropertyValue::Int(self.significant_risk_count as i64),
        );
        p.insert(
            "teamSize".into(),
            GraphPropertyValue::Int(self.team_member_ids.len() as i64),
        );
        p.insert(
            "isComplete".into(),
            GraphPropertyValue::Bool(self.is_complete()),
        );
        p
    }
}

/// Type of audit engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngagementType {
    /// Annual financial statement audit
    #[default]
    AnnualAudit,
    /// Interim audit procedures
    InterimAudit,
    /// SOX 404 internal control audit
    Sox404,
    /// Integrated audit (financial statements + SOX)
    IntegratedAudit,
    /// Review engagement (limited assurance)
    ReviewEngagement,
    /// Compilation engagement (no assurance)
    CompilationEngagement,
    /// Agreed-upon procedures
    AgreedUponProcedures,
    /// Special purpose audit
    SpecialPurpose,
}

impl EngagementType {
    /// Get the assurance level for this engagement type.
    pub fn assurance_level(&self) -> AssuranceLevel {
        match self {
            Self::AnnualAudit
            | Self::InterimAudit
            | Self::Sox404
            | Self::IntegratedAudit
            | Self::SpecialPurpose => AssuranceLevel::Reasonable,
            Self::ReviewEngagement => AssuranceLevel::Limited,
            Self::CompilationEngagement | Self::AgreedUponProcedures => AssuranceLevel::None,
        }
    }

    /// Check if this requires SOX compliance testing.
    pub fn requires_sox_testing(&self) -> bool {
        matches!(self, Self::Sox404 | Self::IntegratedAudit)
    }
}

/// Level of assurance provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    /// Reasonable assurance (audit)
    Reasonable,
    /// Limited assurance (review)
    Limited,
    /// No assurance (compilation, AUP)
    None,
}

/// Engagement status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngagementStatus {
    /// Planning in progress
    #[default]
    Planning,
    /// Active fieldwork
    InProgress,
    /// Under review
    UnderReview,
    /// Pending partner sign-off
    PendingSignOff,
    /// Complete
    Complete,
    /// Archived
    Archived,
    /// On hold
    OnHold,
}

/// Current engagement phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngagementPhase {
    /// Planning and understanding the entity
    #[default]
    Planning,
    /// Risk assessment procedures
    RiskAssessment,
    /// Testing of controls
    ControlTesting,
    /// Substantive testing
    SubstantiveTesting,
    /// Completion procedures
    Completion,
    /// Report issuance
    Reporting,
}

impl EngagementPhase {
    /// Get the phase name for display.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Planning => "Planning",
            Self::RiskAssessment => "Risk Assessment",
            Self::ControlTesting => "Control Testing",
            Self::SubstantiveTesting => "Substantive Testing",
            Self::Completion => "Completion",
            Self::Reporting => "Reporting",
        }
    }

    /// Get the ISA reference for this phase.
    pub fn isa_reference(&self) -> &'static str {
        match self {
            Self::Planning => "ISA 300",
            Self::RiskAssessment => "ISA 315",
            Self::ControlTesting => "ISA 330",
            Self::SubstantiveTesting => "ISA 330, ISA 500",
            Self::Completion => "ISA 450, ISA 560",
            Self::Reporting => "ISA 700",
        }
    }
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    #[default]
    Medium,
    /// High risk
    High,
    /// Significant risk (per ISA 315)
    Significant,
}

impl RiskLevel {
    /// Get numeric score for calculations.
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Significant => 4,
        }
    }

    /// Create from numeric score.
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=1 => Self::Low,
            2 => Self::Medium,
            3 => Self::High,
            _ => Self::Significant,
        }
    }
}

/// Engagement team member role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementTeamMember {
    /// Team member ID
    pub member_id: String,
    /// Team member name
    pub name: String,
    /// Role on the engagement
    pub role: TeamMemberRole,
    /// Allocated hours
    pub allocated_hours: f64,
    /// Actual hours worked
    pub actual_hours: f64,
    /// Sections assigned
    pub assigned_sections: Vec<String>,
}

/// Role of a team member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamMemberRole {
    EngagementPartner,
    EngagementQualityReviewer,
    EngagementManager,
    Senior,
    Staff,
    Specialist,
    ITAuditor,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_creation() {
        let engagement = AuditEngagement::new(
            "ENTITY001",
            "Test Company Inc.",
            EngagementType::AnnualAudit,
            2025,
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        );

        assert_eq!(engagement.fiscal_year, 2025);
        assert_eq!(engagement.engagement_type, EngagementType::AnnualAudit);
        assert_eq!(engagement.status, EngagementStatus::Planning);
    }

    #[test]
    fn test_engagement_with_materiality() {
        let engagement = AuditEngagement::new(
            "ENTITY001",
            "Test Company Inc.",
            EngagementType::AnnualAudit,
            2025,
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .with_materiality(
            Decimal::new(1_000_000, 0),
            0.75,
            0.05,
            "Total Revenue",
            0.005,
        );

        assert_eq!(engagement.materiality, Decimal::new(1_000_000, 0));
        assert_eq!(engagement.performance_materiality, Decimal::new(750_000, 0));
        assert_eq!(engagement.clearly_trivial, Decimal::new(50_000, 0));
    }

    #[test]
    fn test_phase_advancement() {
        let mut engagement = AuditEngagement::new(
            "ENTITY001",
            "Test Company Inc.",
            EngagementType::AnnualAudit,
            2025,
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        );

        assert_eq!(engagement.current_phase, EngagementPhase::Planning);
        engagement.advance_phase();
        assert_eq!(engagement.current_phase, EngagementPhase::RiskAssessment);
        engagement.advance_phase();
        assert_eq!(engagement.current_phase, EngagementPhase::ControlTesting);
    }

    #[test]
    fn test_risk_level_score() {
        assert_eq!(RiskLevel::Low.score(), 1);
        assert_eq!(RiskLevel::Significant.score(), 4);
        assert_eq!(RiskLevel::from_score(3), RiskLevel::High);
    }
}
