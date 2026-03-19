//! Service organization and SOC report models per ISA 402.
//!
//! ISA 402 addresses the responsibilities of the user auditor when the user entity
//! uses services provided by a service organization.  When the services form part
//! of the user entity's information system relevant to financial reporting, the
//! auditor must obtain an understanding of how that affects the assessment of
//! risks of material misstatement.
//!
//! SOC 1 reports (Service Organization Control 1) describe the controls at a
//! service organization relevant to user entity's financial reporting.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A service organization used by one or more audited entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOrganization {
    /// Unique identifier
    pub id: String,
    /// Name of the service organization
    pub name: String,
    /// Type of service provided
    pub service_type: ServiceType,
    /// Entity codes of user entities served by this organization
    pub entities_served: Vec<String>,
}

/// Type of service provided by the service organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    /// Payroll processing services
    #[default]
    PayrollProcessor,
    /// Cloud hosting and infrastructure services
    CloudHosting,
    /// Payment processing services
    PaymentProcessor,
    /// IT managed services
    ItManagedServices,
    /// Data centre colocation
    DataCentre,
}

/// A SOC 1 report obtained from or about a service organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocReport {
    /// Unique identifier for this report
    pub id: String,
    /// Reference to the service organization
    pub service_org_id: String,
    /// Type of SOC report
    pub report_type: SocReportType,
    /// Start of the period covered by the report
    pub report_period_start: NaiveDate,
    /// End of the period covered by the report
    pub report_period_end: NaiveDate,
    /// Opinion issued by the service auditor
    pub opinion_type: SocOpinionType,
    /// Control objectives included in the report
    pub control_objectives: Vec<ControlObjective>,
    /// Exceptions noted during testing
    pub exceptions_noted: Vec<SocException>,
}

/// SOC report type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SocReportType {
    /// SOC 1 Type I: design of controls at a point in time
    Soc1Type1,
    /// SOC 1 Type II: design and operating effectiveness over a period
    #[default]
    Soc1Type2,
}

/// Opinion issued by the service auditor on the SOC report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SocOpinionType {
    /// Unmodified opinion — controls are suitably designed and operating effectively
    #[default]
    Unmodified,
    /// Qualified opinion — one or more control objectives have exceptions
    Qualified,
    /// Adverse opinion — controls are not suitably designed or not operating effectively
    Adverse,
}

/// A control objective within a SOC report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlObjective {
    /// Unique identifier for this objective
    pub id: String,
    /// Description of the control objective
    pub description: String,
    /// Number of controls tested against this objective
    pub controls_tested: u32,
    /// Whether all tested controls operated effectively
    pub controls_effective: bool,
}

/// An exception noted during SOC report testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocException {
    /// ID of the control objective to which this exception relates
    pub control_objective_id: String,
    /// Description of the exception
    pub description: String,
    /// Service organization's management response to the exception
    pub management_response: String,
    /// Impact assessment for the user entity
    pub user_entity_impact: String,
}

/// A complementary user entity control (CUEC) mapped to a SOC objective.
///
/// User entities are responsible for implementing certain controls to complement
/// the controls at the service organization.  These are documented by the user
/// auditor per ISA 402 requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntityControl {
    /// Unique identifier
    pub id: String,
    /// Reference to the SOC report this control relates to
    pub soc_report_id: String,
    /// Description of the user entity control
    pub description: String,
    /// ID of the SOC control objective this control maps to
    pub mapped_objective: String,
    /// Whether the control has been implemented
    pub implemented: bool,
    /// Operating effectiveness assessment
    pub operating_effectiveness: ControlEffectiveness,
}

/// Operating effectiveness assessment for a user entity control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ControlEffectiveness {
    /// Control is operating effectively
    #[default]
    Effective,
    /// Control has minor exceptions but is substantially effective
    EffectiveWithExceptions,
    /// Control is not operating effectively
    Ineffective,
    /// Control has not been tested
    NotTested,
}

impl ServiceOrganization {
    /// Create a new service organization.
    pub fn new(
        name: impl Into<String>,
        service_type: ServiceType,
        entities_served: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            service_type,
            entities_served,
        }
    }
}

impl SocReport {
    /// Create a new SOC report.
    pub fn new(
        service_org_id: impl Into<String>,
        report_type: SocReportType,
        report_period_start: NaiveDate,
        report_period_end: NaiveDate,
        opinion_type: SocOpinionType,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            service_org_id: service_org_id.into(),
            report_type,
            report_period_start,
            report_period_end,
            opinion_type,
            control_objectives: Vec::new(),
            exceptions_noted: Vec::new(),
        }
    }
}

impl ControlObjective {
    /// Create a new control objective.
    pub fn new(
        description: impl Into<String>,
        controls_tested: u32,
        controls_effective: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            controls_tested,
            controls_effective,
        }
    }
}

impl UserEntityControl {
    /// Create a new user entity control.
    pub fn new(
        soc_report_id: impl Into<String>,
        description: impl Into<String>,
        mapped_objective: impl Into<String>,
        implemented: bool,
        operating_effectiveness: ControlEffectiveness,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            soc_report_id: soc_report_id.into(),
            description: description.into(),
            mapped_objective: mapped_objective.into(),
            implemented,
            operating_effectiveness,
        }
    }
}
