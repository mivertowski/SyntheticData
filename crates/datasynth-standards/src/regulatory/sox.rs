//! Sarbanes-Oxley Act (SOX) Compliance Models.
//!
//! Implements SOX compliance structures:
//! - Section 302: CEO/CFO Certifications
//! - Section 404: Management's Assessment of Internal Control
//! - Control deficiency classification
//! - Material weakness documentation

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SOX Section 302 CEO/CFO Certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sox302Certification {
    /// Unique certification identifier.
    pub certification_id: Uuid,

    /// Company code.
    pub company_code: String,

    /// Fiscal year.
    pub fiscal_year: u16,

    /// Period end date.
    pub period_end_date: NaiveDate,

    /// Certifier role.
    pub certifier_role: CertifierRole,

    /// Certifier name.
    pub certifier_name: String,

    /// Certification date.
    pub certification_date: NaiveDate,

    /// Report type being certified.
    pub report_type: ReportType,

    // Certification statements (302(a))
    /// Statement (a)(1): Reviewed the report.
    pub reviewed_report: bool,

    /// Statement (a)(2): Report does not contain material misstatement.
    pub no_material_misstatement: bool,

    /// Statement (a)(3): Financial statements fairly present.
    pub fairly_presented: bool,

    // Disclosure controls (302(a)(4))
    /// Disclosure controls and procedures are effective.
    pub disclosure_controls_effective: bool,

    /// Disclosure controls evaluation date.
    pub disclosure_controls_evaluation_date: Option<NaiveDate>,

    // Internal control (302(a)(4)(B))
    /// Internal control designed effectively.
    pub internal_control_designed_effectively: bool,

    // Changes disclosed (302(a)(5))
    /// Significant changes in internal control disclosed.
    pub significant_changes_disclosed: bool,

    /// Description of significant changes.
    pub significant_changes_description: Option<String>,

    // Fraud notifications (302(a)(5))
    /// Any fraud involving management disclosed.
    pub fraud_disclosed: bool,

    /// Description of fraud if any.
    pub fraud_description: Option<String>,

    /// Material weaknesses disclosed.
    pub material_weaknesses: Vec<Uuid>, // References to Finding IDs

    /// Significant deficiencies disclosed.
    pub significant_deficiencies: Vec<Uuid>,

    /// Certification statement text.
    pub certification_text: String,
}

impl Sox302Certification {
    /// Create a new SOX 302 certification.
    pub fn new(
        company_code: impl Into<String>,
        fiscal_year: u16,
        period_end_date: NaiveDate,
        certifier_role: CertifierRole,
        certifier_name: impl Into<String>,
    ) -> Self {
        Self {
            certification_id: Uuid::now_v7(),
            company_code: company_code.into(),
            fiscal_year,
            period_end_date,
            certifier_role,
            certifier_name: certifier_name.into(),
            certification_date: chrono::Utc::now().date_naive(),
            report_type: ReportType::AnnualReport10K,
            reviewed_report: true,
            no_material_misstatement: true,
            fairly_presented: true,
            disclosure_controls_effective: true,
            disclosure_controls_evaluation_date: Some(period_end_date),
            internal_control_designed_effectively: true,
            significant_changes_disclosed: true,
            significant_changes_description: None,
            fraud_disclosed: false,
            fraud_description: None,
            material_weaknesses: Vec::new(),
            significant_deficiencies: Vec::new(),
            certification_text: String::new(),
        }
    }

    /// Generate standard certification text.
    pub fn generate_certification_text(&mut self) {
        let report_name = match self.report_type {
            ReportType::AnnualReport10K => "annual report on Form 10-K",
            ReportType::QuarterlyReport10Q => "quarterly report on Form 10-Q",
        };

        self.certification_text = format!(
            "I, {}, certify that:\n\n\
             1. I have reviewed this {} of {};\n\n\
             2. Based on my knowledge, this report does not contain any untrue statement of a \
                material fact or omit to state a material fact necessary to make the statements \
                made, in light of the circumstances under which such statements were made, not \
                misleading with respect to the period covered by this report;\n\n\
             3. Based on my knowledge, the financial statements, and other financial information \
                included in this report, fairly present in all material respects the financial \
                condition, results of operations and cash flows of the registrant as of, and for, \
                the periods presented in this report;\n\n\
             4. The registrant's other certifying officer and I are responsible for establishing \
                and maintaining disclosure controls and procedures (as defined in Exchange Act \
                Rules 13a-15(e) and 15d-15(e)) and internal control over financial reporting \
                (as defined in Exchange Act Rules 13a-15(f) and 15d-15(f)) for the registrant \
                and have:\n\n\
                (a) Designed such disclosure controls and procedures, or caused such disclosure \
                    controls and procedures to be designed under our supervision, to ensure that \
                    material information relating to the registrant is made known to us;\n\n\
                (b) Designed such internal control over financial reporting, or caused such \
                    internal control over financial reporting to be designed under our supervision, \
                    to provide reasonable assurance regarding the reliability of financial reporting;\n\n\
                (c) Evaluated the effectiveness of the registrant's disclosure controls and \
                    procedures and presented in this report our conclusions about the effectiveness \
                    of the disclosure controls and procedures;\n\n\
                (d) Disclosed in this report any change in the registrant's internal control over \
                    financial reporting that occurred during the registrant's most recent fiscal \
                    quarter that has materially affected, or is reasonably likely to materially \
                    affect, the registrant's internal control over financial reporting;\n\n\
             5. The registrant's other certifying officer and I have disclosed, based on our most \
                recent evaluation of internal control over financial reporting, to the registrant's \
                auditors and the audit committee of the registrant's board of directors:\n\n\
                (a) All significant deficiencies and material weaknesses in the design or operation \
                    of internal control over financial reporting which are reasonably likely to \
                    adversely affect the registrant's ability to record, process, summarize and \
                    report financial information; and\n\n\
                (b) Any fraud, whether or not material, that involves management or other employees \
                    who have a significant role in the registrant's internal control over financial \
                    reporting.",
            self.certifier_name,
            report_name,
            self.company_code
        );
    }
}

/// Certifier role for SOX certifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertifierRole {
    /// Chief Executive Officer.
    Ceo,
    /// Chief Financial Officer.
    Cfo,
    /// Principal Executive Officer (if not CEO).
    PrincipalExecutiveOfficer,
    /// Principal Financial Officer (if not CFO).
    PrincipalFinancialOfficer,
}

impl std::fmt::Display for CertifierRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ceo => write!(f, "Chief Executive Officer"),
            Self::Cfo => write!(f, "Chief Financial Officer"),
            Self::PrincipalExecutiveOfficer => write!(f, "Principal Executive Officer"),
            Self::PrincipalFinancialOfficer => write!(f, "Principal Financial Officer"),
        }
    }
}

/// Type of report being certified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// Form 10-K Annual Report.
    #[default]
    AnnualReport10K,
    /// Form 10-Q Quarterly Report.
    QuarterlyReport10Q,
}

/// SOX Section 404 Management's Assessment of ICFR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sox404Assessment {
    /// Unique assessment identifier.
    pub assessment_id: Uuid,

    /// Company code.
    pub company_code: String,

    /// Fiscal year being assessed.
    pub fiscal_year: u16,

    /// Assessment date.
    pub assessment_date: NaiveDate,

    /// Framework used for assessment.
    pub framework: IcfrFramework,

    /// Overall ICFR effectiveness conclusion.
    pub icfr_effective: bool,

    /// Scope of the assessment.
    pub scope: Vec<ScopedEntity>,

    /// Materiality threshold used.
    #[serde(with = "rust_decimal::serde::str")]
    pub materiality_threshold: Decimal,

    /// Number of key controls tested.
    pub key_controls_tested: usize,

    /// Number of key controls found effective.
    pub key_controls_effective: usize,

    /// Deficiency classification results.
    pub deficiency_classification: DeficiencyClassificationSummary,

    /// Material weaknesses identified.
    pub material_weaknesses: Vec<MaterialWeakness>,

    /// Significant deficiencies identified.
    pub significant_deficiencies: Vec<SignificantDeficiency>,

    /// Control deficiencies identified.
    pub control_deficiencies: Vec<ControlDeficiency>,

    /// Remediation actions planned/completed.
    pub remediation_actions: Vec<RemediationAction>,

    /// Management's conclusion statement.
    pub management_conclusion: String,

    /// Date of management's report.
    pub management_report_date: NaiveDate,
}

impl Sox404Assessment {
    /// Create a new SOX 404 assessment.
    pub fn new(
        company_code: impl Into<String>,
        fiscal_year: u16,
        assessment_date: NaiveDate,
    ) -> Self {
        Self {
            assessment_id: Uuid::now_v7(),
            company_code: company_code.into(),
            fiscal_year,
            assessment_date,
            framework: IcfrFramework::Coso2013,
            icfr_effective: true,
            scope: Vec::new(),
            materiality_threshold: Decimal::ZERO,
            key_controls_tested: 0,
            key_controls_effective: 0,
            deficiency_classification: DeficiencyClassificationSummary::default(),
            material_weaknesses: Vec::new(),
            significant_deficiencies: Vec::new(),
            control_deficiencies: Vec::new(),
            remediation_actions: Vec::new(),
            management_conclusion: String::new(),
            management_report_date: assessment_date,
        }
    }

    /// Determine if ICFR is effective based on material weaknesses.
    pub fn evaluate_effectiveness(&mut self) {
        // ICFR cannot be effective if any material weaknesses exist
        self.icfr_effective = self.material_weaknesses.is_empty();
    }

    /// Calculate control testing effectiveness rate.
    pub fn effectiveness_rate(&self) -> f64 {
        if self.key_controls_tested == 0 {
            return 0.0;
        }
        (self.key_controls_effective as f64 / self.key_controls_tested as f64) * 100.0
    }
}

/// Framework used for ICFR assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IcfrFramework {
    /// COSO 2013 Internal Control-Integrated Framework.
    #[default]
    Coso2013,
    /// COSO 1992 (legacy).
    Coso1992,
    /// Other recognized framework.
    Other,
}

impl std::fmt::Display for IcfrFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coso2013 => write!(f, "COSO 2013"),
            Self::Coso1992 => write!(f, "COSO 1992"),
            Self::Other => write!(f, "Other Framework"),
        }
    }
}

/// Entity in scope for SOX 404 assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedEntity {
    /// Entity code.
    pub entity_code: String,

    /// Entity name.
    pub entity_name: String,

    /// Percentage of consolidated revenue.
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_percent: Decimal,

    /// Percentage of consolidated assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub assets_percent: Decimal,

    /// Scoping conclusion.
    pub scope_conclusion: ScopeConclusion,

    /// Significant accounts at this entity.
    pub significant_accounts: Vec<String>,
}

/// Scoping conclusion for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScopeConclusion {
    /// Entity is in scope - full testing required.
    #[default]
    InScope,
    /// Entity is out of scope - immaterial.
    OutOfScope,
    /// Specific accounts only in scope.
    SpecificAccountsOnly,
    /// Under common control - reliance on group.
    CommonControl,
}

/// Deficiency classification summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeficiencyClassificationSummary {
    /// Number of deficiencies initially identified.
    pub deficiencies_identified: u32,

    /// Number classified as control deficiency.
    pub control_deficiencies: u32,

    /// Number classified as significant deficiency.
    pub significant_deficiencies: u32,

    /// Number classified as material weakness.
    pub material_weaknesses: u32,

    /// Number remediated before year-end.
    pub remediated: u32,
}

/// Deficiency classification matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeficiencyMatrix {
    /// Likelihood of misstatement occurring.
    pub likelihood: DeficiencyLikelihood,

    /// Magnitude of potential misstatement.
    pub magnitude: DeficiencyMagnitude,

    /// Resulting classification.
    pub classification: DeficiencyClassification,
}

impl DeficiencyMatrix {
    /// Determine classification based on likelihood and magnitude.
    pub fn classify(
        likelihood: DeficiencyLikelihood,
        magnitude: DeficiencyMagnitude,
    ) -> DeficiencyClassification {
        match (likelihood, magnitude) {
            // Material weakness: reasonably possible + material OR probable + more than inconsequential
            (DeficiencyLikelihood::Probable, DeficiencyMagnitude::Material) => {
                DeficiencyClassification::MaterialWeakness
            }
            (DeficiencyLikelihood::ReasonablyPossible, DeficiencyMagnitude::Material) => {
                DeficiencyClassification::MaterialWeakness
            }
            (DeficiencyLikelihood::Probable, DeficiencyMagnitude::MoreThanInconsequential) => {
                DeficiencyClassification::MaterialWeakness
            }

            // Significant deficiency: reasonably possible + more than inconsequential
            (
                DeficiencyLikelihood::ReasonablyPossible,
                DeficiencyMagnitude::MoreThanInconsequential,
            ) => DeficiencyClassification::SignificantDeficiency,
            (DeficiencyLikelihood::Probable, DeficiencyMagnitude::Inconsequential) => {
                DeficiencyClassification::SignificantDeficiency
            }

            // Control deficiency: remote likelihood or inconsequential magnitude
            _ => DeficiencyClassification::ControlDeficiency,
        }
    }
}

/// Likelihood of misstatement for deficiency classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeficiencyLikelihood {
    /// Remote - chance is slight.
    Remote,
    /// Reasonably possible - more than remote but less than likely.
    #[default]
    ReasonablyPossible,
    /// Probable - likely to occur.
    Probable,
}

impl std::fmt::Display for DeficiencyLikelihood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remote => write!(f, "Remote"),
            Self::ReasonablyPossible => write!(f, "Reasonably Possible"),
            Self::Probable => write!(f, "Probable"),
        }
    }
}

/// Magnitude of potential misstatement for deficiency classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeficiencyMagnitude {
    /// Inconsequential - clearly trivial.
    Inconsequential,
    /// More than inconsequential but less than material.
    #[default]
    MoreThanInconsequential,
    /// Material - would influence decisions of users.
    Material,
}

impl std::fmt::Display for DeficiencyMagnitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inconsequential => write!(f, "Inconsequential"),
            Self::MoreThanInconsequential => write!(f, "More Than Inconsequential"),
            Self::Material => write!(f, "Material"),
        }
    }
}

/// Deficiency classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeficiencyClassification {
    /// Control deficiency - does not rise to significant or material.
    #[default]
    ControlDeficiency,
    /// Significant deficiency - less severe than material weakness.
    SignificantDeficiency,
    /// Material weakness - reasonable possibility of material misstatement.
    MaterialWeakness,
}

impl std::fmt::Display for DeficiencyClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlDeficiency => write!(f, "Control Deficiency"),
            Self::SignificantDeficiency => write!(f, "Significant Deficiency"),
            Self::MaterialWeakness => write!(f, "Material Weakness"),
        }
    }
}

/// Material weakness record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialWeakness {
    /// Unique identifier.
    pub weakness_id: Uuid,

    /// Description of the material weakness.
    pub description: String,

    /// Affected controls.
    pub affected_controls: Vec<String>,

    /// Affected significant accounts.
    pub affected_accounts: Vec<String>,

    /// Related assertions impacted.
    pub related_assertions: Vec<String>,

    /// Root cause.
    pub root_cause: String,

    /// Potential misstatement amount.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub potential_misstatement: Option<Decimal>,

    /// Likelihood assessment.
    pub likelihood: DeficiencyLikelihood,

    /// Magnitude assessment.
    pub magnitude: DeficiencyMagnitude,

    /// Identification date.
    pub identification_date: NaiveDate,

    /// Whether remediated by year-end.
    pub remediated_by_year_end: bool,

    /// Remediation date if remediated.
    pub remediation_date: Option<NaiveDate>,

    /// Reference to related findings.
    pub related_finding_ids: Vec<Uuid>,
}

impl MaterialWeakness {
    /// Create a new material weakness.
    pub fn new(description: impl Into<String>, identification_date: NaiveDate) -> Self {
        Self {
            weakness_id: Uuid::now_v7(),
            description: description.into(),
            affected_controls: Vec::new(),
            affected_accounts: Vec::new(),
            related_assertions: Vec::new(),
            root_cause: String::new(),
            potential_misstatement: None,
            likelihood: DeficiencyLikelihood::Probable,
            magnitude: DeficiencyMagnitude::Material,
            identification_date,
            remediated_by_year_end: false,
            remediation_date: None,
            related_finding_ids: Vec::new(),
        }
    }
}

/// Significant deficiency record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantDeficiency {
    /// Unique identifier.
    pub deficiency_id: Uuid,

    /// Description.
    pub description: String,

    /// Affected controls.
    pub affected_controls: Vec<String>,

    /// Affected accounts.
    pub affected_accounts: Vec<String>,

    /// Likelihood assessment.
    pub likelihood: DeficiencyLikelihood,

    /// Magnitude assessment.
    pub magnitude: DeficiencyMagnitude,

    /// Identification date.
    pub identification_date: NaiveDate,

    /// Remediation status.
    pub remediated: bool,
}

impl SignificantDeficiency {
    /// Create a new significant deficiency.
    pub fn new(description: impl Into<String>, identification_date: NaiveDate) -> Self {
        Self {
            deficiency_id: Uuid::now_v7(),
            description: description.into(),
            affected_controls: Vec::new(),
            affected_accounts: Vec::new(),
            likelihood: DeficiencyLikelihood::ReasonablyPossible,
            magnitude: DeficiencyMagnitude::MoreThanInconsequential,
            identification_date,
            remediated: false,
        }
    }
}

/// Control deficiency record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDeficiency {
    /// Unique identifier.
    pub deficiency_id: Uuid,

    /// Description.
    pub description: String,

    /// Affected control.
    pub affected_control: String,

    /// Identification date.
    pub identification_date: NaiveDate,

    /// Remediation status.
    pub remediated: bool,
}

/// Remediation action record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAction {
    /// Action identifier.
    pub action_id: Uuid,

    /// Related deficiency ID.
    pub deficiency_id: Uuid,

    /// Action description.
    pub description: String,

    /// Responsible party.
    pub responsible_party: String,

    /// Target completion date.
    pub target_date: NaiveDate,

    /// Actual completion date.
    pub completion_date: Option<NaiveDate>,

    /// Status.
    pub status: RemediationStatus,

    /// Testing of remediation performed.
    pub remediation_tested: bool,

    /// Remediation effective.
    pub remediation_effective: bool,
}

/// Remediation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    /// Not started.
    #[default]
    NotStarted,
    /// In progress.
    InProgress,
    /// Completed.
    Completed,
    /// Deferred to next period.
    Deferred,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_sox_302_certification() {
        let mut cert = Sox302Certification::new(
            "ABC Corp",
            2024,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            CertifierRole::Ceo,
            "John Smith",
        );

        cert.generate_certification_text();

        assert!(!cert.certification_text.is_empty());
        assert!(cert.certification_text.contains("John Smith"));
        assert!(cert.disclosure_controls_effective);
    }

    #[test]
    fn test_sox_404_assessment() {
        let mut assessment = Sox404Assessment::new(
            "ABC Corp",
            2024,
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
        );

        assessment.key_controls_tested = 100;
        assessment.key_controls_effective = 95;
        assessment.materiality_threshold = dec!(100000);

        assert_eq!(assessment.effectiveness_rate(), 95.0);
        assert!(assessment.icfr_effective);
    }

    #[test]
    fn test_sox_404_with_material_weakness() {
        let mut assessment = Sox404Assessment::new(
            "ABC Corp",
            2024,
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
        );

        assessment.material_weaknesses.push(MaterialWeakness::new(
            "Inadequate segregation of duties in accounts payable",
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
        ));

        assessment.evaluate_effectiveness();

        assert!(!assessment.icfr_effective);
    }

    #[test]
    fn test_deficiency_matrix_classification() {
        // Material weakness cases
        assert_eq!(
            DeficiencyMatrix::classify(
                DeficiencyLikelihood::Probable,
                DeficiencyMagnitude::Material
            ),
            DeficiencyClassification::MaterialWeakness
        );
        assert_eq!(
            DeficiencyMatrix::classify(
                DeficiencyLikelihood::ReasonablyPossible,
                DeficiencyMagnitude::Material
            ),
            DeficiencyClassification::MaterialWeakness
        );

        // Significant deficiency
        assert_eq!(
            DeficiencyMatrix::classify(
                DeficiencyLikelihood::ReasonablyPossible,
                DeficiencyMagnitude::MoreThanInconsequential
            ),
            DeficiencyClassification::SignificantDeficiency
        );

        // Control deficiency
        assert_eq!(
            DeficiencyMatrix::classify(
                DeficiencyLikelihood::Remote,
                DeficiencyMagnitude::Inconsequential
            ),
            DeficiencyClassification::ControlDeficiency
        );
    }
}
