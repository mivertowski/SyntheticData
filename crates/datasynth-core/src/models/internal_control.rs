//! Internal Controls System (ICS) definitions for SOX compliance.
//!
//! Provides structures for modeling internal controls, control testing,
//! and SOX 404 compliance markers in synthetic accounting data.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::coso::{ControlScope, CosoComponent, CosoMaturityLevel, CosoPrinciple};
use super::graph_properties::{GraphPropertyValue, ToNodeProperties};
use super::user::UserPersona;

/// Control type based on SOX 404 framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlType {
    /// Prevents errors/fraud before they occur
    Preventive,
    /// Detects errors/fraud after they occur
    Detective,
    /// Continuous monitoring and analytics
    Monitoring,
}

impl std::fmt::Display for ControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preventive => write!(f, "Preventive"),
            Self::Detective => write!(f, "Detective"),
            Self::Monitoring => write!(f, "Monitoring"),
        }
    }
}

/// Control testing frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFrequency {
    /// Applied to every transaction
    Transactional,
    /// Performed daily
    Daily,
    /// Performed weekly
    Weekly,
    /// Performed monthly
    Monthly,
    /// Performed quarterly
    Quarterly,
    /// Performed annually
    Annual,
}

impl std::fmt::Display for ControlFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transactional => write!(f, "Transactional"),
            Self::Daily => write!(f, "Daily"),
            Self::Weekly => write!(f, "Weekly"),
            Self::Monthly => write!(f, "Monthly"),
            Self::Quarterly => write!(f, "Quarterly"),
            Self::Annual => write!(f, "Annual"),
        }
    }
}

/// Risk level for controls and control deficiencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk - minor impact
    Low,
    /// Medium risk - moderate impact
    Medium,
    /// High risk - significant impact
    High,
    /// Critical risk - material impact on financial statements
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// SOX 404 financial statement assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoxAssertion {
    /// Transactions and events have been recorded
    Existence,
    /// All transactions have been recorded
    Completeness,
    /// Amounts are recorded at appropriate values
    Valuation,
    /// Entity has rights to assets and obligations for liabilities
    RightsAndObligations,
    /// Components are properly classified and disclosed
    PresentationAndDisclosure,
}

impl ToNodeProperties for SoxAssertion {
    fn node_type_name(&self) -> &'static str {
        "sox_assertion"
    }
    fn node_type_code(&self) -> u16 {
        502
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert("name".into(), GraphPropertyValue::String(self.to_string()));
        p.insert(
            "code".into(),
            GraphPropertyValue::String(format!("{self:?}")),
        );
        p
    }
}

impl std::fmt::Display for SoxAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Existence => write!(f, "Existence"),
            Self::Completeness => write!(f, "Completeness"),
            Self::Valuation => write!(f, "Valuation"),
            Self::RightsAndObligations => write!(f, "RightsAndObligations"),
            Self::PresentationAndDisclosure => write!(f, "PresentationAndDisclosure"),
        }
    }
}

/// Result of control testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TestResult {
    /// Control test passed — no exceptions found
    Pass,
    /// Control test partially passed — minor exceptions
    Partial,
    /// Control test failed — material exception
    Fail,
    /// Control has not been tested
    #[default]
    NotTested,
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "Pass"),
            Self::Partial => write!(f, "Partial"),
            Self::Fail => write!(f, "Fail"),
            Self::NotTested => write!(f, "NotTested"),
        }
    }
}

/// Derived control effectiveness rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ControlEffectiveness {
    /// Control is operating effectively
    Effective,
    /// Control is partially effective — some deficiencies
    PartiallyEffective,
    /// Control has not been tested for effectiveness
    #[default]
    NotTested,
    /// Control is not effective — material weakness or significant deficiency
    Ineffective,
}

impl std::fmt::Display for ControlEffectiveness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Effective => write!(f, "Effective"),
            Self::PartiallyEffective => write!(f, "PartiallyEffective"),
            Self::NotTested => write!(f, "NotTested"),
            Self::Ineffective => write!(f, "Ineffective"),
        }
    }
}

/// Control status for transaction-level tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    /// Control operating effectively
    #[default]
    Effective,
    /// Control exception/deficiency found
    Exception,
    /// Control not yet tested
    NotTested,
    /// Exception has been remediated
    Remediated,
}

impl std::fmt::Display for ControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Effective => write!(f, "Effective"),
            Self::Exception => write!(f, "Exception"),
            Self::NotTested => write!(f, "NotTested"),
            Self::Remediated => write!(f, "Remediated"),
        }
    }
}

/// Internal control definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalControl {
    /// Unique control identifier (e.g., "C001", "C010")
    pub control_id: String,
    /// Control name/title
    pub control_name: String,
    /// Type of control (Preventive, Detective, Monitoring)
    pub control_type: ControlType,
    /// Control objective description
    pub objective: String,
    /// How often the control is performed
    pub frequency: ControlFrequency,
    /// Role responsible for executing/owning the control
    pub owner_role: UserPersona,
    /// Risk level associated with control failure
    pub risk_level: RiskLevel,
    /// Detailed description of the control procedure
    pub description: String,
    /// Whether this is a SOX 404 key control
    pub is_key_control: bool,
    /// SOX assertion this control addresses
    pub sox_assertion: SoxAssertion,
    /// COSO 2013 component this control maps to
    pub coso_component: CosoComponent,
    /// COSO 2013 principles this control addresses
    pub coso_principles: Vec<CosoPrinciple>,
    /// Control scope (entity-level vs transaction-level)
    pub control_scope: ControlScope,
    /// Control maturity level
    pub maturity_level: CosoMaturityLevel,

    // --- New fields for test history, effectiveness, owner resolution, risk linkage ---

    /// Employee ID of the control owner (resolved from owner_role at generation time)
    pub owner_employee_id: Option<String>,
    /// Display name of the control owner
    pub owner_name: String,

    /// Number of times this control has been tested
    pub test_count: u32,
    /// Date of the most recent test
    pub last_tested_date: Option<NaiveDate>,
    /// Result of the most recent test
    pub test_result: TestResult,

    /// Derived effectiveness rating (from maturity_level + test_result)
    pub effectiveness: ControlEffectiveness,

    /// IDs of risks this control mitigates (populated at generation time)
    pub mitigates_risk_ids: Vec<String>,
    /// Account classes this control covers (derived from sox_assertion)
    pub covers_account_classes: Vec<String>,
}

impl InternalControl {
    /// Create a new internal control.
    pub fn new(
        control_id: impl Into<String>,
        control_name: impl Into<String>,
        control_type: ControlType,
        objective: impl Into<String>,
    ) -> Self {
        Self {
            control_id: control_id.into(),
            control_name: control_name.into(),
            control_type,
            objective: objective.into(),
            frequency: ControlFrequency::Transactional,
            owner_role: UserPersona::Controller,
            risk_level: RiskLevel::Medium,
            description: String::new(),
            is_key_control: false,
            sox_assertion: SoxAssertion::Existence,
            coso_component: CosoComponent::ControlActivities,
            coso_principles: vec![CosoPrinciple::ControlActions],
            control_scope: ControlScope::TransactionLevel,
            maturity_level: CosoMaturityLevel::Defined,
            owner_employee_id: None,
            owner_name: String::new(),
            test_count: 0,
            last_tested_date: None,
            test_result: TestResult::NotTested,
            effectiveness: ControlEffectiveness::NotTested,
            mitigates_risk_ids: Vec::new(),
            covers_account_classes: Vec::new(),
        }
    }

    /// Builder method to set frequency.
    pub fn with_frequency(mut self, frequency: ControlFrequency) -> Self {
        self.frequency = frequency;
        self
    }

    /// Builder method to set owner role.
    pub fn with_owner(mut self, owner: UserPersona) -> Self {
        self.owner_role = owner;
        self
    }

    /// Builder method to set risk level.
    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    /// Builder method to set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder method to mark as key control.
    pub fn as_key_control(mut self) -> Self {
        self.is_key_control = true;
        self
    }

    /// Builder method to set SOX assertion.
    pub fn with_assertion(mut self, assertion: SoxAssertion) -> Self {
        self.sox_assertion = assertion;
        self
    }

    /// Builder method to set COSO component.
    pub fn with_coso_component(mut self, component: CosoComponent) -> Self {
        self.coso_component = component;
        self
    }

    /// Builder method to set COSO principles.
    pub fn with_coso_principles(mut self, principles: Vec<CosoPrinciple>) -> Self {
        self.coso_principles = principles;
        self
    }

    /// Builder method to set control scope.
    pub fn with_control_scope(mut self, scope: ControlScope) -> Self {
        self.control_scope = scope;
        self
    }

    /// Builder method to set maturity level.
    pub fn with_maturity_level(mut self, level: CosoMaturityLevel) -> Self {
        self.maturity_level = level;
        self
    }

    /// Builder method to set owner employee ID and name.
    pub fn with_owner_employee(
        mut self,
        employee_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        self.owner_employee_id = Some(employee_id.into());
        self.owner_name = name.into();
        self
    }

    /// Builder method to set test history.
    pub fn with_test_history(
        mut self,
        test_count: u32,
        last_tested_date: Option<NaiveDate>,
        test_result: TestResult,
    ) -> Self {
        self.test_count = test_count;
        self.last_tested_date = last_tested_date;
        self.test_result = test_result;
        self
    }

    /// Builder method to set effectiveness.
    pub fn with_effectiveness(mut self, effectiveness: ControlEffectiveness) -> Self {
        self.effectiveness = effectiveness;
        self
    }

    /// Builder method to set mitigated risk IDs.
    pub fn with_mitigates_risk_ids(mut self, risk_ids: Vec<String>) -> Self {
        self.mitigates_risk_ids = risk_ids;
        self
    }

    /// Builder method to set covered account classes.
    pub fn with_covers_account_classes(mut self, classes: Vec<String>) -> Self {
        self.covers_account_classes = classes;
        self
    }

    /// Derive test history and effectiveness from the current `maturity_level`.
    ///
    /// Call this after setting `maturity_level` to populate `test_count`,
    /// `last_tested_date`, `test_result`, and `effectiveness`.
    ///
    /// - Maturity >= 4 (Managed/Optimized): tested multiple times, Pass, Effective
    /// - Maturity == 3 (Defined): tested once, Partial, PartiallyEffective
    /// - Maturity <= 2 (NonExistent/AdHoc/Repeatable): not tested
    ///
    /// `assessed_date` is the reference date from which `last_tested_date` is
    /// back-computed: `assessed_date - 30 * (5 - maturity_level)` days.
    pub fn derive_from_maturity(&mut self, assessed_date: NaiveDate) {
        let level = self.maturity_level.level();

        if level >= 4 {
            // Managed (4) or Optimized (5)
            self.test_count = (level as u32).saturating_sub(2); // 2 or 3
            let days_back = 30_i64 * (5 - level as i64);
            self.last_tested_date =
                assessed_date.checked_sub_signed(chrono::Duration::days(days_back));
            self.test_result = TestResult::Pass;
            self.effectiveness = ControlEffectiveness::Effective;
        } else if level == 3 {
            // Defined
            self.test_count = 1;
            let days_back = 30_i64 * (5 - 3);
            self.last_tested_date =
                assessed_date.checked_sub_signed(chrono::Duration::days(days_back));
            self.test_result = TestResult::Partial;
            self.effectiveness = ControlEffectiveness::PartiallyEffective;
        } else {
            // NonExistent (0), AdHoc (1), Repeatable (2)
            self.test_count = 0;
            self.last_tested_date = None;
            self.test_result = TestResult::NotTested;
            self.effectiveness = ControlEffectiveness::NotTested;
        }
    }

    /// Derive `covers_account_classes` from `sox_assertion`.
    ///
    /// Maps SOX assertions to the account classes they cover:
    /// - Existence -> Assets
    /// - Completeness -> Revenue, Liabilities
    /// - Valuation -> Assets, Liabilities, Equity, Revenue, Expenses
    /// - RightsAndObligations -> Assets, Liabilities
    /// - PresentationAndDisclosure -> Revenue, Equity
    pub fn derive_account_classes(&mut self) {
        self.covers_account_classes = match self.sox_assertion {
            SoxAssertion::Existence => vec!["Assets".into()],
            SoxAssertion::Completeness => vec!["Revenue".into(), "Liabilities".into()],
            SoxAssertion::Valuation => vec![
                "Assets".into(),
                "Liabilities".into(),
                "Equity".into(),
                "Revenue".into(),
                "Expenses".into(),
            ],
            SoxAssertion::RightsAndObligations => vec!["Assets".into(), "Liabilities".into()],
            SoxAssertion::PresentationAndDisclosure => vec!["Revenue".into(), "Equity".into()],
        };
    }

    /// Generate standard controls for a typical organization.
    ///
    /// Includes both transaction-level controls (C001-C060) and
    /// entity-level controls (C070-C081) with full COSO 2013 mappings.
    pub fn standard_controls() -> Vec<Self> {
        vec![
            // ========================================
            // TRANSACTION-LEVEL CONTROLS (C001-C060)
            // ========================================

            // Cash controls
            Self::new(
                "C001",
                "Cash Account Daily Review",
                ControlType::Detective,
                "Review all cash transactions daily for unauthorized activity",
            )
            .with_frequency(ControlFrequency::Daily)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Daily reconciliation of cash accounts with bank statements and review of unusual transactions",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::OngoingMonitoring,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Large transaction approval
            Self::new(
                "C002",
                "Large Transaction Multi-Level Approval",
                ControlType::Preventive,
                "Transactions over $10,000 require additional approval levels",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::Manager)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Multi-level approval workflow for transactions exceeding defined thresholds",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::PoliciesAndProcedures,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // P2P Three-Way Match
            Self::new(
                "C010",
                "Three-Way Match",
                ControlType::Preventive,
                "Match purchase order, receipt, and invoice before payment",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::Medium)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Automated matching of PO, goods receipt, and vendor invoice prior to payment release",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::TechnologyControls,
            ])
            .with_control_scope(ControlScope::ItApplicationControl)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Vendor Master Maintenance
            Self::new(
                "C011",
                "Vendor Master Data Maintenance",
                ControlType::Preventive,
                "Segregated access for vendor master data changes",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Restricted access to vendor master data with dual-approval for bank account changes",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::FraudRisk,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // O2C Revenue Recognition
            Self::new(
                "C020",
                "Revenue Recognition Review",
                ControlType::Detective,
                "Review revenue entries for proper timing and classification",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Critical)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Monthly review of revenue recognition to ensure compliance with ASC 606",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::ClearObjectives,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Credit Limit Enforcement
            Self::new(
                "C021",
                "Customer Credit Limit Check",
                ControlType::Preventive,
                "Automatic credit limit check before order acceptance",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::AutomatedSystem)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "System-enforced credit limit validation at order entry",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::TechnologyControls,
                CosoPrinciple::ControlActions,
            ])
            .with_control_scope(ControlScope::ItApplicationControl)
            .with_maturity_level(CosoMaturityLevel::Optimized),

            // GL Account Reconciliation
            Self::new(
                "C030",
                "GL Account Reconciliation",
                ControlType::Detective,
                "Monthly reconciliation of all balance sheet accounts",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Complete reconciliation of all balance sheet accounts with supporting documentation",
            )
            .with_coso_component(CosoComponent::MonitoringActivities)
            .with_coso_principles(vec![CosoPrinciple::OngoingMonitoring])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Journal Entry Review
            Self::new(
                "C031",
                "Manual Journal Entry Review",
                ControlType::Detective,
                "Review of all manual journal entries over threshold",
            )
            .with_frequency(ControlFrequency::Daily)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Daily review of manual journal entries with supporting documentation",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::FraudRisk,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Period Close Review
            Self::new(
                "C032",
                "Period Close Checklist",
                ControlType::Detective,
                "Comprehensive checklist for period-end close procedures",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Standardized period-end close checklist ensuring all procedures completed",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::PoliciesAndProcedures,
                CosoPrinciple::ControlActions,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // Payroll Processing
            Self::new(
                "C040",
                "Payroll Processing Review",
                ControlType::Detective,
                "Review of payroll processing for accuracy",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Monthly review of payroll journals and reconciliation to HR records",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::FraudRisk,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Fixed Asset Additions
            Self::new(
                "C050",
                "Fixed Asset Addition Approval",
                ControlType::Preventive,
                "Multi-level approval for capital expenditures",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::Manager)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Approval workflow for capital asset additions based on dollar thresholds",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::ControlActions,
                CosoPrinciple::PoliciesAndProcedures,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // Intercompany Reconciliation
            Self::new(
                "C060",
                "Intercompany Balance Reconciliation",
                ControlType::Detective,
                "Monthly reconciliation of intercompany balances",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Full reconciliation of intercompany accounts between all entities",
            )
            .with_coso_component(CosoComponent::MonitoringActivities)
            .with_coso_principles(vec![
                CosoPrinciple::OngoingMonitoring,
                CosoPrinciple::DeficiencyEvaluation,
            ])
            .with_control_scope(ControlScope::TransactionLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // ========================================
            // ENTITY-LEVEL CONTROLS (C070-C081)
            // ========================================

            // Code of Conduct
            Self::new(
                "C070",
                "Code of Conduct and Ethics",
                ControlType::Preventive,
                "Establish and communicate ethical values and standards of conduct",
            )
            .with_frequency(ControlFrequency::Annual)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::PresentationAndDisclosure)
            .with_description(
                "Annual review and acknowledgment of code of conduct by all employees; \
                 includes ethics hotline and whistleblower protections",
            )
            .with_coso_component(CosoComponent::ControlEnvironment)
            .with_coso_principles(vec![
                CosoPrinciple::IntegrityAndEthics,
                CosoPrinciple::Accountability,
            ])
            .with_control_scope(ControlScope::EntityLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Audit Committee Oversight
            Self::new(
                "C071",
                "Audit Committee Oversight",
                ControlType::Monitoring,
                "Board and audit committee exercise independent oversight of internal control",
            )
            .with_frequency(ControlFrequency::Quarterly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Critical)
            .as_key_control()
            .with_assertion(SoxAssertion::PresentationAndDisclosure)
            .with_description(
                "Quarterly audit committee meetings with review of internal control effectiveness, \
                 external auditor findings, and management representations",
            )
            .with_coso_component(CosoComponent::ControlEnvironment)
            .with_coso_principles(vec![
                CosoPrinciple::BoardOversight,
                CosoPrinciple::OrganizationalStructure,
            ])
            .with_control_scope(ControlScope::EntityLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Risk Assessment Process
            Self::new(
                "C075",
                "Enterprise Risk Assessment",
                ControlType::Detective,
                "Identify and assess risks to achievement of organizational objectives",
            )
            .with_frequency(ControlFrequency::Annual)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Annual enterprise risk assessment process including fraud risk evaluation; \
                 risk register maintained and updated quarterly",
            )
            .with_coso_component(CosoComponent::RiskAssessment)
            .with_coso_principles(vec![
                CosoPrinciple::IdentifyRisks,
                CosoPrinciple::FraudRisk,
                CosoPrinciple::ChangeIdentification,
            ])
            .with_control_scope(ControlScope::EntityLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // IT General Controls
            Self::new(
                "C077",
                "IT General Controls Program",
                ControlType::Preventive,
                "General controls over IT environment supporting financial reporting systems",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::AutomatedSystem)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "IT general controls including access management, change management, \
                 computer operations, and program development for systems supporting \
                 financial reporting",
            )
            .with_coso_component(CosoComponent::ControlActivities)
            .with_coso_principles(vec![
                CosoPrinciple::TechnologyControls,
                CosoPrinciple::PoliciesAndProcedures,
            ])
            .with_control_scope(ControlScope::ItGeneralControl)
            .with_maturity_level(CosoMaturityLevel::Managed),

            // Information Quality
            Self::new(
                "C078",
                "Financial Information Quality",
                ControlType::Detective,
                "Obtain and use quality information for internal control",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Monthly data quality reviews for key financial reports; validation of \
                 data inputs, processing, and outputs supporting management decisions",
            )
            .with_coso_component(CosoComponent::InformationCommunication)
            .with_coso_principles(vec![
                CosoPrinciple::QualityInformation,
                CosoPrinciple::InternalCommunication,
            ])
            .with_control_scope(ControlScope::EntityLevel)
            .with_maturity_level(CosoMaturityLevel::Defined),

            // Monitoring Program
            Self::new(
                "C081",
                "Internal Control Monitoring Program",
                ControlType::Monitoring,
                "Ongoing and periodic evaluations of internal control effectiveness",
            )
            .with_frequency(ControlFrequency::Quarterly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Continuous monitoring program with quarterly control testing, \
                 deficiency tracking, and remediation management; annual SOX 404 \
                 assessment and certification",
            )
            .with_coso_component(CosoComponent::MonitoringActivities)
            .with_coso_principles(vec![
                CosoPrinciple::OngoingMonitoring,
                CosoPrinciple::DeficiencyEvaluation,
            ])
            .with_control_scope(ControlScope::EntityLevel)
            .with_maturity_level(CosoMaturityLevel::Managed),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_control_creation() {
        let control = InternalControl::new(
            "TEST001",
            "Test Control",
            ControlType::Preventive,
            "Test objective",
        )
        .with_frequency(ControlFrequency::Daily)
        .with_risk_level(RiskLevel::High)
        .as_key_control();

        assert_eq!(control.control_id, "TEST001");
        assert_eq!(control.control_type, ControlType::Preventive);
        assert_eq!(control.frequency, ControlFrequency::Daily);
        assert_eq!(control.risk_level, RiskLevel::High);
        assert!(control.is_key_control);
    }

    #[test]
    fn test_standard_controls() {
        let controls = InternalControl::standard_controls();
        assert!(!controls.is_empty());

        // Verify key controls exist
        let key_controls: Vec<_> = controls.iter().filter(|c| c.is_key_control).collect();
        assert!(key_controls.len() >= 5);

        // Verify different control types exist
        let preventive: Vec<_> = controls
            .iter()
            .filter(|c| c.control_type == ControlType::Preventive)
            .collect();
        let detective: Vec<_> = controls
            .iter()
            .filter(|c| c.control_type == ControlType::Detective)
            .collect();

        assert!(!preventive.is_empty());
        assert!(!detective.is_empty());
    }

    #[test]
    fn test_control_status_display() {
        assert_eq!(ControlStatus::Effective.to_string(), "Effective");
        assert_eq!(ControlStatus::Exception.to_string(), "Exception");
    }
}
