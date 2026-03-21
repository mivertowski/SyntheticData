//! ISA Standard References (International Standards on Auditing).
//!
//! Provides comprehensive ISA standard enumerations and mapping structures
//! for documenting audit procedure compliance with specific ISA requirements.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ISA Standard enumeration covering all major ISA standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IsaStandard {
    // 200 Series - General Principles and Responsibilities
    /// ISA 200: Overall Objectives of the Independent Auditor
    Isa200,
    /// ISA 210: Agreeing the Terms of Audit Engagements
    Isa210,
    /// ISA 220: Quality Management for an Audit of Financial Statements
    Isa220,
    /// ISA 230: Audit Documentation
    Isa230,
    /// ISA 240: The Auditor's Responsibilities Relating to Fraud
    Isa240,
    /// ISA 250: Consideration of Laws and Regulations
    Isa250,
    /// ISA 260: Communication with Those Charged with Governance
    Isa260,
    /// ISA 265: Communicating Deficiencies in Internal Control
    Isa265,

    // 300 Series - Risk Assessment and Response
    /// ISA 300: Planning an Audit of Financial Statements
    Isa300,
    /// ISA 315: Identifying and Assessing Risks of Material Misstatement
    Isa315,
    /// ISA 320: Materiality in Planning and Performing an Audit
    Isa320,
    /// ISA 330: The Auditor's Responses to Assessed Risks
    Isa330,

    // 400 Series - Internal Control
    /// ISA 402: Audit Considerations Relating to Service Organizations
    Isa402,
    /// ISA 450: Evaluation of Misstatements Identified During the Audit
    Isa450,

    // 500 Series - Audit Evidence
    /// ISA 500: Audit Evidence
    Isa500,
    /// ISA 501: Audit Evidence - Specific Considerations
    Isa501,
    /// ISA 505: External Confirmations
    Isa505,
    /// ISA 510: Initial Audit Engagements - Opening Balances
    Isa510,
    /// ISA 520: Analytical Procedures
    Isa520,
    /// ISA 530: Audit Sampling
    Isa530,
    /// ISA 540: Auditing Accounting Estimates and Related Disclosures
    Isa540,
    /// ISA 550: Related Parties
    Isa550,
    /// ISA 560: Subsequent Events
    Isa560,
    /// ISA 570: Going Concern
    Isa570,
    /// ISA 580: Written Representations
    Isa580,

    // 600 Series - Using Work of Others
    /// ISA 600: Special Considerations - Audits of Group Financial Statements
    Isa600,
    /// ISA 610: Using the Work of Internal Auditors
    Isa610,
    /// ISA 620: Using the Work of an Auditor's Expert
    Isa620,

    // 700 Series - Audit Conclusions and Reporting
    /// ISA 700: Forming an Opinion and Reporting on Financial Statements
    Isa700,
    /// ISA 701: Communicating Key Audit Matters
    Isa701,
    /// ISA 705: Modifications to the Opinion
    Isa705,
    /// ISA 706: Emphasis of Matter and Other Matter Paragraphs
    Isa706,
    /// ISA 710: Comparative Information
    Isa710,
    /// ISA 720: The Auditor's Responsibilities Relating to Other Information
    Isa720,
}

impl IsaStandard {
    /// Get the standard number as a string (e.g., "315").
    pub fn number(&self) -> &'static str {
        match self {
            Self::Isa200 => "200",
            Self::Isa210 => "210",
            Self::Isa220 => "220",
            Self::Isa230 => "230",
            Self::Isa240 => "240",
            Self::Isa250 => "250",
            Self::Isa260 => "260",
            Self::Isa265 => "265",
            Self::Isa300 => "300",
            Self::Isa315 => "315",
            Self::Isa320 => "320",
            Self::Isa330 => "330",
            Self::Isa402 => "402",
            Self::Isa450 => "450",
            Self::Isa500 => "500",
            Self::Isa501 => "501",
            Self::Isa505 => "505",
            Self::Isa510 => "510",
            Self::Isa520 => "520",
            Self::Isa530 => "530",
            Self::Isa540 => "540",
            Self::Isa550 => "550",
            Self::Isa560 => "560",
            Self::Isa570 => "570",
            Self::Isa580 => "580",
            Self::Isa600 => "600",
            Self::Isa610 => "610",
            Self::Isa620 => "620",
            Self::Isa700 => "700",
            Self::Isa701 => "701",
            Self::Isa705 => "705",
            Self::Isa706 => "706",
            Self::Isa710 => "710",
            Self::Isa720 => "720",
        }
    }

    /// Get the full title of the standard.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Isa200 => "Overall Objectives of the Independent Auditor",
            Self::Isa210 => "Agreeing the Terms of Audit Engagements",
            Self::Isa220 => "Quality Management for an Audit of Financial Statements",
            Self::Isa230 => "Audit Documentation",
            Self::Isa240 => "The Auditor's Responsibilities Relating to Fraud",
            Self::Isa250 => "Consideration of Laws and Regulations",
            Self::Isa260 => "Communication with Those Charged with Governance",
            Self::Isa265 => "Communicating Deficiencies in Internal Control",
            Self::Isa300 => "Planning an Audit of Financial Statements",
            Self::Isa315 => "Identifying and Assessing Risks of Material Misstatement",
            Self::Isa320 => "Materiality in Planning and Performing an Audit",
            Self::Isa330 => "The Auditor's Responses to Assessed Risks",
            Self::Isa402 => "Audit Considerations Relating to Service Organizations",
            Self::Isa450 => "Evaluation of Misstatements Identified During the Audit",
            Self::Isa500 => "Audit Evidence",
            Self::Isa501 => "Audit Evidence - Specific Considerations",
            Self::Isa505 => "External Confirmations",
            Self::Isa510 => "Initial Audit Engagements - Opening Balances",
            Self::Isa520 => "Analytical Procedures",
            Self::Isa530 => "Audit Sampling",
            Self::Isa540 => "Auditing Accounting Estimates and Related Disclosures",
            Self::Isa550 => "Related Parties",
            Self::Isa560 => "Subsequent Events",
            Self::Isa570 => "Going Concern",
            Self::Isa580 => "Written Representations",
            Self::Isa600 => "Special Considerations - Audits of Group Financial Statements",
            Self::Isa610 => "Using the Work of Internal Auditors",
            Self::Isa620 => "Using the Work of an Auditor's Expert",
            Self::Isa700 => "Forming an Opinion and Reporting on Financial Statements",
            Self::Isa701 => "Communicating Key Audit Matters",
            Self::Isa705 => "Modifications to the Opinion",
            Self::Isa706 => "Emphasis of Matter and Other Matter Paragraphs",
            Self::Isa710 => "Comparative Information",
            Self::Isa720 => "The Auditor's Responsibilities Relating to Other Information",
        }
    }

    /// Get the ISA series this standard belongs to.
    pub fn series(&self) -> IsaSeries {
        match self {
            Self::Isa200
            | Self::Isa210
            | Self::Isa220
            | Self::Isa230
            | Self::Isa240
            | Self::Isa250
            | Self::Isa260
            | Self::Isa265 => IsaSeries::GeneralPrinciples,
            Self::Isa300 | Self::Isa315 | Self::Isa320 | Self::Isa330 => IsaSeries::RiskAssessment,
            Self::Isa402 | Self::Isa450 => IsaSeries::InternalControl,
            Self::Isa500
            | Self::Isa501
            | Self::Isa505
            | Self::Isa510
            | Self::Isa520
            | Self::Isa530
            | Self::Isa540
            | Self::Isa550
            | Self::Isa560
            | Self::Isa570
            | Self::Isa580 => IsaSeries::AuditEvidence,
            Self::Isa600 | Self::Isa610 | Self::Isa620 => IsaSeries::UsingWorkOfOthers,
            Self::Isa700
            | Self::Isa701
            | Self::Isa705
            | Self::Isa706
            | Self::Isa710
            | Self::Isa720 => IsaSeries::Reporting,
        }
    }

    /// Returns all ISA standards as a vector.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Isa200,
            Self::Isa210,
            Self::Isa220,
            Self::Isa230,
            Self::Isa240,
            Self::Isa250,
            Self::Isa260,
            Self::Isa265,
            Self::Isa300,
            Self::Isa315,
            Self::Isa320,
            Self::Isa330,
            Self::Isa402,
            Self::Isa450,
            Self::Isa500,
            Self::Isa501,
            Self::Isa505,
            Self::Isa510,
            Self::Isa520,
            Self::Isa530,
            Self::Isa540,
            Self::Isa550,
            Self::Isa560,
            Self::Isa570,
            Self::Isa580,
            Self::Isa600,
            Self::Isa610,
            Self::Isa620,
            Self::Isa700,
            Self::Isa701,
            Self::Isa705,
            Self::Isa706,
            Self::Isa710,
            Self::Isa720,
        ]
    }
}

impl std::fmt::Display for IsaStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ISA {}", self.number())
    }
}

/// ISA Series groupings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsaSeries {
    /// 200 Series: General Principles and Responsibilities
    GeneralPrinciples,
    /// 300 Series: Risk Assessment and Response
    RiskAssessment,
    /// 400 Series: Internal Control
    InternalControl,
    /// 500 Series: Audit Evidence
    AuditEvidence,
    /// 600 Series: Using Work of Others
    UsingWorkOfOthers,
    /// 700 Series: Audit Conclusions and Reporting
    Reporting,
}

impl std::fmt::Display for IsaSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneralPrinciples => write!(f, "General Principles and Responsibilities"),
            Self::RiskAssessment => write!(f, "Risk Assessment and Response"),
            Self::InternalControl => write!(f, "Internal Control"),
            Self::AuditEvidence => write!(f, "Audit Evidence"),
            Self::UsingWorkOfOthers => write!(f, "Using Work of Others"),
            Self::Reporting => write!(f, "Audit Conclusions and Reporting"),
        }
    }
}

/// Specific ISA requirement reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaRequirement {
    /// The ISA standard.
    pub standard: IsaStandard,

    /// Specific paragraph number (e.g., "25" for ISA 315.25).
    pub paragraph: String,

    /// Type of requirement.
    pub requirement_type: IsaRequirementType,

    /// Brief description of the requirement.
    pub description: String,

    /// Whether this is a mandatory ("shall") requirement.
    pub is_mandatory: bool,
}

impl IsaRequirement {
    /// Create a new ISA requirement reference.
    pub fn new(
        standard: IsaStandard,
        paragraph: impl Into<String>,
        requirement_type: IsaRequirementType,
        description: impl Into<String>,
    ) -> Self {
        let is_mandatory = matches!(requirement_type, IsaRequirementType::Requirement);
        Self {
            standard,
            paragraph: paragraph.into(),
            requirement_type,
            description: description.into(),
            is_mandatory,
        }
    }

    /// Get the full reference string (e.g., "ISA 315.25").
    pub fn reference(&self) -> String {
        format!("ISA {}.{}", self.standard.number(), self.paragraph)
    }
}

/// Type of ISA requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsaRequirementType {
    /// Objective of the standard.
    Objective,
    /// Mandatory requirement ("shall").
    Requirement,
    /// Application guidance (non-mandatory).
    ApplicationGuidance,
    /// Definition.
    Definition,
}

impl std::fmt::Display for IsaRequirementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Objective => write!(f, "Objective"),
            Self::Requirement => write!(f, "Requirement"),
            Self::ApplicationGuidance => write!(f, "Application Guidance"),
            Self::Definition => write!(f, "Definition"),
        }
    }
}

/// Mapping of audit procedure to ISA requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaProcedureMapping {
    /// Unique mapping identifier.
    pub mapping_id: Uuid,

    /// Audit procedure ID being mapped.
    pub procedure_id: Uuid,

    /// Procedure description.
    pub procedure_description: String,

    /// ISA requirements addressed by this procedure.
    pub isa_requirements: Vec<IsaRequirement>,

    /// Compliance status.
    pub compliance_status: ComplianceStatus,

    /// Documentation reference.
    pub documentation_reference: Option<String>,

    /// Notes on compliance.
    pub compliance_notes: String,
}

impl IsaProcedureMapping {
    /// Create a new procedure mapping.
    pub fn new(procedure_id: Uuid, procedure_description: impl Into<String>) -> Self {
        Self {
            mapping_id: Uuid::now_v7(),
            procedure_id,
            procedure_description: procedure_description.into(),
            isa_requirements: Vec::new(),
            compliance_status: ComplianceStatus::NotAssessed,
            documentation_reference: None,
            compliance_notes: String::new(),
        }
    }

    /// Add an ISA requirement.
    pub fn add_requirement(&mut self, requirement: IsaRequirement) {
        self.isa_requirements.push(requirement);
    }

    /// Get unique standards covered by this mapping.
    pub fn standards_covered(&self) -> Vec<IsaStandard> {
        let mut standards: Vec<_> = self.isa_requirements.iter().map(|r| r.standard).collect();
        standards.sort_by_key(IsaStandard::number);
        standards.dedup();
        standards
    }
}

/// ISA compliance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Not yet assessed.
    #[default]
    NotAssessed,
    /// Fully compliant.
    Compliant,
    /// Partially compliant.
    PartiallyCompliant,
    /// Non-compliant.
    NonCompliant,
    /// Not applicable.
    NotApplicable,
}

impl std::fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAssessed => write!(f, "Not Assessed"),
            Self::Compliant => write!(f, "Compliant"),
            Self::PartiallyCompliant => write!(f, "Partially Compliant"),
            Self::NonCompliant => write!(f, "Non-Compliant"),
            Self::NotApplicable => write!(f, "Not Applicable"),
        }
    }
}

/// ISA coverage summary for an engagement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaCoverageSummary {
    /// Engagement ID.
    pub engagement_id: Uuid,

    /// Standards explicitly addressed.
    pub standards_addressed: Vec<IsaStandard>,

    /// Coverage by series.
    pub coverage_by_series: Vec<SeriesCoverage>,

    /// Overall compliance percentage.
    pub overall_compliance_percent: f64,

    /// Standards not covered.
    pub gaps: Vec<IsaStandard>,

    /// Mandatory requirements not addressed.
    pub unaddressed_mandatory: Vec<IsaRequirement>,
}

impl IsaCoverageSummary {
    /// Create a new coverage summary.
    pub fn new(engagement_id: Uuid) -> Self {
        Self {
            engagement_id,
            standards_addressed: Vec::new(),
            coverage_by_series: Vec::new(),
            overall_compliance_percent: 0.0,
            gaps: Vec::new(),
            unaddressed_mandatory: Vec::new(),
        }
    }

    /// Calculate coverage from procedure mappings.
    pub fn calculate_coverage(&mut self, mappings: &[IsaProcedureMapping]) {
        self.standards_addressed = mappings
            .iter()
            .flat_map(IsaProcedureMapping::standards_covered)
            .collect();
        self.standards_addressed.sort_by_key(IsaStandard::number);
        self.standards_addressed.dedup();

        // Calculate gaps
        self.gaps = IsaStandard::all()
            .into_iter()
            .filter(|s| !self.standards_addressed.contains(s))
            .collect();

        // Calculate overall compliance
        let total = IsaStandard::all().len();
        let covered = self.standards_addressed.len();
        self.overall_compliance_percent = (covered as f64 / total as f64) * 100.0;
    }
}

/// Coverage summary by ISA series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesCoverage {
    /// ISA series.
    pub series: IsaSeries,

    /// Standards in series.
    pub total_standards: usize,

    /// Standards addressed.
    pub addressed_standards: usize,

    /// Coverage percentage.
    pub coverage_percent: f64,
}

/// A flat, serializable entry for a single ISA standard (used for `isa_mappings.json` output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaStandardEntry {
    /// Standard number (e.g., "315").
    pub number: String,
    /// Full title (e.g., "Identifying and Assessing Risks of Material Misstatement").
    pub title: String,
    /// ISA series label (e.g., "Risk Assessment and Response").
    pub series: String,
    /// Canonical display name (e.g., "ISA 315").
    pub display_name: String,
}

impl IsaStandardEntry {
    /// Build from an [`IsaStandard`] enum variant.
    pub fn from_standard(standard: IsaStandard) -> Self {
        Self {
            number: standard.number().to_string(),
            title: standard.title().to_string(),
            series: standard.series().to_string(),
            display_name: standard.to_string(),
        }
    }
}

impl IsaStandard {
    /// Return all ISA standards as flat, serializable [`IsaStandardEntry`] records.
    ///
    /// Suitable for direct JSON output (e.g., `audit/isa_mappings.json`).
    pub fn standard_entries() -> Vec<IsaStandardEntry> {
        Self::all()
            .into_iter()
            .map(IsaStandardEntry::from_standard)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_isa_standard_number() {
        assert_eq!(IsaStandard::Isa315.number(), "315");
        assert_eq!(IsaStandard::Isa700.number(), "700");
    }

    #[test]
    fn test_isa_standard_title() {
        assert_eq!(
            IsaStandard::Isa315.title(),
            "Identifying and Assessing Risks of Material Misstatement"
        );
    }

    #[test]
    fn test_isa_standard_series() {
        assert_eq!(IsaStandard::Isa200.series(), IsaSeries::GeneralPrinciples);
        assert_eq!(IsaStandard::Isa315.series(), IsaSeries::RiskAssessment);
        assert_eq!(IsaStandard::Isa500.series(), IsaSeries::AuditEvidence);
        assert_eq!(IsaStandard::Isa700.series(), IsaSeries::Reporting);
    }

    #[test]
    fn test_isa_requirement_reference() {
        let req = IsaRequirement::new(
            IsaStandard::Isa315,
            "25",
            IsaRequirementType::Requirement,
            "Identify and assess risks of material misstatement",
        );

        assert_eq!(req.reference(), "ISA 315.25");
        assert!(req.is_mandatory);
    }

    #[test]
    fn test_procedure_mapping() {
        let mut mapping =
            IsaProcedureMapping::new(Uuid::now_v7(), "Test risk assessment procedures");

        mapping.add_requirement(IsaRequirement::new(
            IsaStandard::Isa315,
            "25",
            IsaRequirementType::Requirement,
            "Risk assessment",
        ));

        mapping.add_requirement(IsaRequirement::new(
            IsaStandard::Isa330,
            "5",
            IsaRequirementType::Requirement,
            "Audit responses",
        ));

        let standards = mapping.standards_covered();
        assert_eq!(standards.len(), 2);
        assert!(standards.contains(&IsaStandard::Isa315));
        assert!(standards.contains(&IsaStandard::Isa330));
    }

    #[test]
    fn test_all_standards() {
        let all = IsaStandard::all();
        assert_eq!(all.len(), 34); // 34 ISA standards
    }
}
