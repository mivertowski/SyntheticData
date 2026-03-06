//! COSO 2013 Internal Control-Integrated Framework definitions.
//!
//! Provides structures for modeling the COSO framework's 5 components
//! and 17 principles, along with control scope and maturity levels.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// COSO 2013 Framework - 5 Components of Internal Control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CosoComponent {
    /// The set of standards, processes, and structures that provide the basis
    /// for carrying out internal control across the organization.
    ControlEnvironment,
    /// A dynamic and iterative process for identifying and assessing risks
    /// to achievement of objectives.
    RiskAssessment,
    /// Actions established through policies and procedures that help ensure
    /// that management's directives are carried out.
    ControlActivities,
    /// Information is necessary for the entity to carry out internal control
    /// responsibilities. Communication is the ongoing process of providing,
    /// sharing, and obtaining necessary information.
    InformationCommunication,
    /// Ongoing evaluations, separate evaluations, or some combination of
    /// the two are used to ascertain whether each of the five components
    /// of internal control is present and functioning.
    MonitoringActivities,
}

impl std::fmt::Display for CosoComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlEnvironment => write!(f, "Control Environment"),
            Self::RiskAssessment => write!(f, "Risk Assessment"),
            Self::ControlActivities => write!(f, "Control Activities"),
            Self::InformationCommunication => write!(f, "Information & Communication"),
            Self::MonitoringActivities => write!(f, "Monitoring Activities"),
        }
    }
}

/// COSO 2013 Framework - 17 Principles of Internal Control.
///
/// Each principle maps to one of the 5 COSO components:
/// - Control Environment: Principles 1-5
/// - Risk Assessment: Principles 6-9
/// - Control Activities: Principles 10-12
/// - Information & Communication: Principles 13-15
/// - Monitoring Activities: Principles 16-17
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CosoPrinciple {
    // Control Environment (Principles 1-5)
    /// Principle 1: The organization demonstrates a commitment to integrity
    /// and ethical values.
    IntegrityAndEthics,
    /// Principle 2: The board of directors demonstrates independence from
    /// management and exercises oversight of internal control.
    BoardOversight,
    /// Principle 3: Management establishes structures, reporting lines, and
    /// appropriate authorities and responsibilities.
    OrganizationalStructure,
    /// Principle 4: The organization demonstrates a commitment to attract,
    /// develop, and retain competent individuals.
    CommitmentToCompetence,
    /// Principle 5: The organization holds individuals accountable for their
    /// internal control responsibilities.
    Accountability,

    // Risk Assessment (Principles 6-9)
    /// Principle 6: The organization specifies objectives with sufficient
    /// clarity to enable the identification and assessment of risks.
    ClearObjectives,
    /// Principle 7: The organization identifies risks to the achievement
    /// of its objectives and analyzes risks as a basis for determining
    /// how the risks should be managed.
    IdentifyRisks,
    /// Principle 8: The organization considers the potential for fraud
    /// in assessing risks to the achievement of objectives.
    FraudRisk,
    /// Principle 9: The organization identifies and assesses changes that
    /// could significantly impact the system of internal control.
    ChangeIdentification,

    // Control Activities (Principles 10-12)
    /// Principle 10: The organization selects and develops control activities
    /// that contribute to the mitigation of risks.
    ControlActions,
    /// Principle 11: The organization selects and develops general control
    /// activities over technology to support the achievement of objectives.
    TechnologyControls,
    /// Principle 12: The organization deploys control activities through
    /// policies that establish what is expected and procedures that put
    /// policies into action.
    PoliciesAndProcedures,

    // Information & Communication (Principles 13-15)
    /// Principle 13: The organization obtains or generates and uses relevant,
    /// quality information to support the functioning of internal control.
    QualityInformation,
    /// Principle 14: The organization internally communicates information,
    /// including objectives and responsibilities for internal control.
    InternalCommunication,
    /// Principle 15: The organization communicates with external parties
    /// regarding matters affecting the functioning of internal control.
    ExternalCommunication,

    // Monitoring Activities (Principles 16-17)
    /// Principle 16: The organization selects, develops, and performs ongoing
    /// and/or separate evaluations to ascertain whether the components of
    /// internal control are present and functioning.
    OngoingMonitoring,
    /// Principle 17: The organization evaluates and communicates internal
    /// control deficiencies in a timely manner to those parties responsible
    /// for taking corrective action.
    DeficiencyEvaluation,
}

impl CosoPrinciple {
    /// Returns the COSO component that this principle belongs to.
    pub fn component(&self) -> CosoComponent {
        match self {
            // Control Environment (Principles 1-5)
            Self::IntegrityAndEthics
            | Self::BoardOversight
            | Self::OrganizationalStructure
            | Self::CommitmentToCompetence
            | Self::Accountability => CosoComponent::ControlEnvironment,

            // Risk Assessment (Principles 6-9)
            Self::ClearObjectives
            | Self::IdentifyRisks
            | Self::FraudRisk
            | Self::ChangeIdentification => CosoComponent::RiskAssessment,

            // Control Activities (Principles 10-12)
            Self::ControlActions | Self::TechnologyControls | Self::PoliciesAndProcedures => {
                CosoComponent::ControlActivities
            }

            // Information & Communication (Principles 13-15)
            Self::QualityInformation
            | Self::InternalCommunication
            | Self::ExternalCommunication => CosoComponent::InformationCommunication,

            // Monitoring Activities (Principles 16-17)
            Self::OngoingMonitoring | Self::DeficiencyEvaluation => {
                CosoComponent::MonitoringActivities
            }
        }
    }

    /// Returns the principle number (1-17) in the COSO framework.
    pub fn principle_number(&self) -> u8 {
        match self {
            Self::IntegrityAndEthics => 1,
            Self::BoardOversight => 2,
            Self::OrganizationalStructure => 3,
            Self::CommitmentToCompetence => 4,
            Self::Accountability => 5,
            Self::ClearObjectives => 6,
            Self::IdentifyRisks => 7,
            Self::FraudRisk => 8,
            Self::ChangeIdentification => 9,
            Self::ControlActions => 10,
            Self::TechnologyControls => 11,
            Self::PoliciesAndProcedures => 12,
            Self::QualityInformation => 13,
            Self::InternalCommunication => 14,
            Self::ExternalCommunication => 15,
            Self::OngoingMonitoring => 16,
            Self::DeficiencyEvaluation => 17,
        }
    }
}

impl std::fmt::Display for CosoPrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntegrityAndEthics => write!(f, "Integrity and Ethics"),
            Self::BoardOversight => write!(f, "Board Oversight"),
            Self::OrganizationalStructure => write!(f, "Organizational Structure"),
            Self::CommitmentToCompetence => write!(f, "Commitment to Competence"),
            Self::Accountability => write!(f, "Accountability"),
            Self::ClearObjectives => write!(f, "Clear Objectives"),
            Self::IdentifyRisks => write!(f, "Identify Risks"),
            Self::FraudRisk => write!(f, "Fraud Risk"),
            Self::ChangeIdentification => write!(f, "Change Identification"),
            Self::ControlActions => write!(f, "Control Actions"),
            Self::TechnologyControls => write!(f, "Technology Controls"),
            Self::PoliciesAndProcedures => write!(f, "Policies and Procedures"),
            Self::QualityInformation => write!(f, "Quality Information"),
            Self::InternalCommunication => write!(f, "Internal Communication"),
            Self::ExternalCommunication => write!(f, "External Communication"),
            Self::OngoingMonitoring => write!(f, "Ongoing Monitoring"),
            Self::DeficiencyEvaluation => write!(f, "Deficiency Evaluation"),
        }
    }
}

impl ToNodeProperties for CosoComponent {
    fn node_type_name(&self) -> &'static str {
        "coso_component"
    }
    fn node_type_code(&self) -> u16 {
        500
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert("name".into(), GraphPropertyValue::String(self.to_string()));
        p.insert(
            "code".into(),
            GraphPropertyValue::String(format!("{self:?}")),
        );
        // Number of principles in this component
        let principle_count = match self {
            CosoComponent::ControlEnvironment => 5,
            CosoComponent::RiskAssessment => 4,
            CosoComponent::ControlActivities => 3,
            CosoComponent::InformationCommunication => 3,
            CosoComponent::MonitoringActivities => 2,
        };
        p.insert(
            "principleCount".into(),
            GraphPropertyValue::Int(principle_count),
        );
        p
    }
}

impl ToNodeProperties for CosoPrinciple {
    fn node_type_name(&self) -> &'static str {
        "coso_principle"
    }
    fn node_type_code(&self) -> u16 {
        501
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert("name".into(), GraphPropertyValue::String(self.to_string()));
        p.insert(
            "code".into(),
            GraphPropertyValue::String(format!("{self:?}")),
        );
        p.insert(
            "number".into(),
            GraphPropertyValue::Int(self.principle_number() as i64),
        );
        p.insert(
            "componentId".into(),
            GraphPropertyValue::String(format!("{:?}", self.component())),
        );
        p.insert(
            "componentName".into(),
            GraphPropertyValue::String(self.component().to_string()),
        );
        p
    }
}

/// Control scope distinguishing entity-level from transaction-level controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlScope {
    /// Entity-level controls operate across the organization and are typically
    /// more pervasive in nature (e.g., tone at the top, code of conduct).
    EntityLevel,
    /// Transaction-level controls operate at the process or transaction level
    /// and are typically more specific (e.g., three-way match, approvals).
    TransactionLevel,
    /// IT General Controls (ITGCs) are controls over the IT environment that
    /// support the effective functioning of application controls.
    ItGeneralControl,
    /// IT Application Controls are automated controls embedded within
    /// applications to ensure data integrity and proper authorization.
    ItApplicationControl,
}

impl std::fmt::Display for ControlScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityLevel => write!(f, "Entity Level"),
            Self::TransactionLevel => write!(f, "Transaction Level"),
            Self::ItGeneralControl => write!(f, "IT General Control"),
            Self::ItApplicationControl => write!(f, "IT Application Control"),
        }
    }
}

/// Control maturity level based on capability maturity models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CosoMaturityLevel {
    /// Level 0: Control processes do not exist or are not recognized.
    NonExistent,
    /// Level 1: Processes are ad hoc and chaotic; success depends on
    /// individual effort.
    AdHoc,
    /// Level 2: Basic processes exist and are repeated; discipline exists
    /// to maintain basic consistency.
    Repeatable,
    /// Level 3: Processes are documented, standardized, and integrated
    /// into the organization.
    Defined,
    /// Level 4: Processes are measured and controlled using metrics;
    /// performance is predictable.
    Managed,
    /// Level 5: Continuous improvement is enabled through feedback and
    /// innovative ideas.
    Optimized,
}

impl std::fmt::Display for CosoMaturityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonExistent => write!(f, "Non-Existent"),
            Self::AdHoc => write!(f, "Ad Hoc"),
            Self::Repeatable => write!(f, "Repeatable"),
            Self::Defined => write!(f, "Defined"),
            Self::Managed => write!(f, "Managed"),
            Self::Optimized => write!(f, "Optimized"),
        }
    }
}

impl CosoMaturityLevel {
    /// Returns the numeric level (0-5).
    pub fn level(&self) -> u8 {
        match self {
            Self::NonExistent => 0,
            Self::AdHoc => 1,
            Self::Repeatable => 2,
            Self::Defined => 3,
            Self::Managed => 4,
            Self::Optimized => 5,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_principle_component_mapping() {
        // Control Environment principles
        assert_eq!(
            CosoPrinciple::IntegrityAndEthics.component(),
            CosoComponent::ControlEnvironment
        );
        assert_eq!(
            CosoPrinciple::Accountability.component(),
            CosoComponent::ControlEnvironment
        );

        // Risk Assessment principles
        assert_eq!(
            CosoPrinciple::FraudRisk.component(),
            CosoComponent::RiskAssessment
        );

        // Control Activities principles
        assert_eq!(
            CosoPrinciple::ControlActions.component(),
            CosoComponent::ControlActivities
        );
        assert_eq!(
            CosoPrinciple::TechnologyControls.component(),
            CosoComponent::ControlActivities
        );

        // Information & Communication principles
        assert_eq!(
            CosoPrinciple::QualityInformation.component(),
            CosoComponent::InformationCommunication
        );

        // Monitoring Activities principles
        assert_eq!(
            CosoPrinciple::OngoingMonitoring.component(),
            CosoComponent::MonitoringActivities
        );
        assert_eq!(
            CosoPrinciple::DeficiencyEvaluation.component(),
            CosoComponent::MonitoringActivities
        );
    }

    #[test]
    fn test_principle_numbers() {
        assert_eq!(CosoPrinciple::IntegrityAndEthics.principle_number(), 1);
        assert_eq!(CosoPrinciple::Accountability.principle_number(), 5);
        assert_eq!(CosoPrinciple::ClearObjectives.principle_number(), 6);
        assert_eq!(CosoPrinciple::ControlActions.principle_number(), 10);
        assert_eq!(CosoPrinciple::QualityInformation.principle_number(), 13);
        assert_eq!(CosoPrinciple::DeficiencyEvaluation.principle_number(), 17);
    }

    #[test]
    fn test_maturity_level_ordering() {
        assert!(CosoMaturityLevel::NonExistent < CosoMaturityLevel::AdHoc);
        assert!(CosoMaturityLevel::AdHoc < CosoMaturityLevel::Repeatable);
        assert!(CosoMaturityLevel::Repeatable < CosoMaturityLevel::Defined);
        assert!(CosoMaturityLevel::Defined < CosoMaturityLevel::Managed);
        assert!(CosoMaturityLevel::Managed < CosoMaturityLevel::Optimized);
    }

    #[test]
    fn test_maturity_level_numeric() {
        assert_eq!(CosoMaturityLevel::NonExistent.level(), 0);
        assert_eq!(CosoMaturityLevel::Optimized.level(), 5);
    }

    #[test]
    fn test_display_implementations() {
        assert_eq!(
            CosoComponent::ControlEnvironment.to_string(),
            "Control Environment"
        );
        assert_eq!(
            CosoPrinciple::IntegrityAndEthics.to_string(),
            "Integrity and Ethics"
        );
        assert_eq!(ControlScope::EntityLevel.to_string(), "Entity Level");
        assert_eq!(CosoMaturityLevel::Defined.to_string(), "Defined");
    }

    #[test]
    fn test_serde_roundtrip() {
        let component = CosoComponent::ControlActivities;
        let json = serde_json::to_string(&component).unwrap();
        let deserialized: CosoComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(component, deserialized);

        let principle = CosoPrinciple::FraudRisk;
        let json = serde_json::to_string(&principle).unwrap();
        let deserialized: CosoPrinciple = serde_json::from_str(&json).unwrap();
        assert_eq!(principle, deserialized);
    }
}
