//! ISA 600 Group Audit — component auditor models.
//!
//! Supports simulation of group audit engagements following ISA 600 (Special
//! Considerations — Audits of Group Financial Statements), including:
//!
//! - Component auditor assignment and competence assessment
//! - Group audit planning and materiality allocation
//! - Component instructions issued by the group engagement team
//! - Component auditor reporting back to the group

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A component auditor firm assigned to one or more group entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAuditor {
    /// Unique identifier.
    pub id: String,
    /// Audit firm name (e.g., "Audit Firm DE").
    pub firm_name: String,
    /// Jurisdiction / country code the firm operates in.
    pub jurisdiction: String,
    /// Whether independence has been confirmed by the group engagement team.
    pub independence_confirmed: bool,
    /// Group engagement team's assessment of competence.
    pub competence_assessment: CompetenceLevel,
    /// Entity codes this component auditor is responsible for.
    pub assigned_entities: Vec<String>,
}

/// Competence assessment of a component auditor by the group team.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompetenceLevel {
    /// Competence is satisfactory — no additional supervision required.
    Satisfactory,
    /// Satisfactory with conditions — additional supervision or procedures required.
    RequiresSupervision,
    /// Unsatisfactory — alternative arrangements must be made.
    Unsatisfactory,
}

/// Overall group audit plan covering all components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAuditPlan {
    /// Reference to the parent audit engagement.
    pub engagement_id: String,
    /// Group-level materiality (applied to group financial statements).
    #[serde(with = "rust_decimal::serde::str")]
    pub group_materiality: Decimal,
    /// Materiality allocation per component entity.
    pub component_allocations: Vec<ComponentMaterialityAllocation>,
    /// Aggregation risk level (risk that uncorrected misstatements in components
    /// would in aggregate exceed group materiality).
    pub aggregation_risk: GroupRiskLevel,
    /// Entity codes identified as significant components.
    pub significant_components: Vec<String>,
    /// Consolidation-level audit procedures performed by the group team.
    pub consolidation_audit_procedures: Vec<String>,
}

/// Aggregation risk level for group audits (ISA 600).
///
/// Named `GroupRiskLevel` to avoid conflict with the engagement-level
/// `RiskLevel` enum used in ISA 315/330 risk assessments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GroupRiskLevel {
    Low,
    Medium,
    High,
}

/// Materiality allocation for a single component entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMaterialityAllocation {
    /// Entity code this allocation applies to.
    pub entity_code: String,
    /// Component materiality threshold (lower than group materiality).
    #[serde(with = "rust_decimal::serde::str")]
    pub component_materiality: Decimal,
    /// Clearly-trivial threshold (items below this need not be aggregated).
    #[serde(with = "rust_decimal::serde::str")]
    pub clearly_trivial: Decimal,
    /// Basis used to allocate materiality.
    pub allocation_basis: AllocationBasis,
}

/// Basis for allocating group materiality to a component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationBasis {
    /// Proportional to component's share of group revenue.
    RevenueProportional,
    /// Proportional to component's share of group total assets.
    AssetProportional,
    /// Based on assessed risk rather than financial size.
    RiskBased,
}

/// Scope of work required from a component auditor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ComponentScope {
    /// Full audit of the component financial statements.
    FullScope,
    /// Audit of specific account areas only.
    SpecificScope {
        /// Account areas subject to audit procedures.
        account_areas: Vec<String>,
    },
    /// Limited agreed-upon procedures.
    LimitedProcedures,
    /// Analytical procedures only — typically for non-significant components.
    AnalyticalOnly,
}

/// Instruction issued by the group engagement team to a component auditor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInstruction {
    /// Unique identifier.
    pub id: String,
    /// ID of the component auditor receiving the instruction.
    pub component_auditor_id: String,
    /// Entity code this instruction relates to.
    pub entity_code: String,
    /// Scope of work required.
    pub scope: ComponentScope,
    /// Materiality allocated for this instruction (from GroupAuditPlan).
    #[serde(with = "rust_decimal::serde::str")]
    pub materiality_allocated: Decimal,
    /// Deadline by which the component auditor must report back.
    pub reporting_deadline: NaiveDate,
    /// Specific audit procedures the component auditor is required to perform.
    pub specific_procedures: Vec<String>,
    /// Account areas or risk areas the group team wishes to focus on.
    pub areas_of_focus: Vec<String>,
}

/// Report returned by a component auditor to the group engagement team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAuditorReport {
    /// Unique identifier.
    pub id: String,
    /// ID of the instruction this report responds to.
    pub instruction_id: String,
    /// ID of the component auditor submitting the report.
    pub component_auditor_id: String,
    /// Entity code this report covers.
    pub entity_code: String,
    /// Misstatements identified during the component audit.
    pub misstatements_identified: Vec<Misstatement>,
    /// Any limitations on the scope of work performed.
    pub scope_limitations: Vec<String>,
    /// Findings significant enough to communicate to the group team.
    pub significant_findings: Vec<String>,
    /// Overall conclusion of the component auditor (plain text).
    pub conclusion: String,
}

/// A misstatement identified by a component auditor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Misstatement {
    /// Description of the misstatement.
    pub description: String,
    /// Monetary amount of the misstatement.
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Classification of the misstatement.
    pub classification: MisstatementType,
    /// Account area or financial statement line where the misstatement was found.
    pub account_area: String,
    /// Whether the misstatement was corrected by management.
    pub corrected: bool,
}

/// Classification of an audit misstatement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MisstatementType {
    /// A specific misstatement that can be precisely measured.
    Factual,
    /// A misstatement arising from a difference in accounting judgment.
    Judgmental,
    /// An extrapolation from a sample to the full population.
    Projected,
}

/// The full output of group audit / component audit generation.
#[derive(Debug, Clone, Default)]
pub struct ComponentAuditSnapshot {
    /// One auditor record per jurisdiction.
    pub component_auditors: Vec<ComponentAuditor>,
    /// The overall group audit plan.
    pub group_audit_plan: Option<GroupAuditPlan>,
    /// One instruction per entity.
    pub component_instructions: Vec<ComponentInstruction>,
    /// One report per entity.
    pub component_reports: Vec<ComponentAuditorReport>,
}
