//! PCAOB Standards Integration.
//!
//! Provides PCAOB (Public Company Accounting Oversight Board) standards
//! and their mappings to ISA standards for US public company audits.

use serde::{Deserialize, Serialize};

use super::isa_reference::IsaStandard;

/// PCAOB Auditing Standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PcaobStandard {
    // 1000 Series - General Auditing Standards
    /// AS 1001: Responsibilities and Functions of the Independent Auditor
    As1001,
    /// AS 1005: Independence
    As1005,
    /// AS 1010: Training and Proficiency of the Independent Auditor
    As1010,
    /// AS 1015: Due Professional Care in the Performance of Work
    As1015,

    // 1100 Series - General Concepts
    /// AS 1101: Audit Risk
    As1101,
    /// AS 1105: Audit Evidence
    As1105,
    /// AS 1110: Relationship of Auditing Standards to Quality Control Standards
    As1110,

    // 1200 Series - General Activities
    /// AS 1201: Supervision of the Audit Engagement
    As1201,
    /// AS 1205: Part of Audit Performed by Other Independent Auditors
    As1205,
    /// AS 1210: Using the Work of a Specialist
    As1210,
    /// AS 1215: Audit Documentation
    As1215,
    /// AS 1220: Engagement Quality Review
    As1220,

    // 1300 Series - Auditor Communications
    /// AS 1301: Communications with Audit Committees
    As1301,
    /// AS 1305: Communications About Control Deficiencies
    As1305,

    // 2100 Series - Audit Procedures: General
    /// AS 2101: Audit Planning
    As2101,
    /// AS 2105: Consideration of Materiality in Planning and Performing an Audit
    As2105,
    /// AS 2110: Identifying and Assessing Risks of Material Misstatement
    As2110,

    // 2200 Series - Auditing Internal Control
    /// AS 2201: An Audit of Internal Control Over Financial Reporting
    As2201,

    // 2300 Series - Audit Procedures: Response to Risks
    /// AS 2301: The Auditor's Responses to the Risks of Material Misstatement
    As2301,
    /// AS 2305: Substantive Analytical Procedures
    As2305,
    /// AS 2310: The Confirmation Process
    As2310,
    /// AS 2315: Audit Sampling
    As2315,

    // 2400 Series - Audit Procedures: Fraud and Illegal Acts
    /// AS 2401: Consideration of Fraud in a Financial Statement Audit
    As2401,
    /// AS 2405: Illegal Acts by Clients
    As2405,

    // 2500 Series - Audit Procedures: Specific Areas
    /// AS 2501: Auditing Accounting Estimates
    As2501,
    /// AS 2502: Auditing Fair Value Measurements and Disclosures
    As2502,
    /// AS 2503: Auditing Derivative Instruments
    As2503,
    /// AS 2505: Inquiry of a Client's Lawyer
    As2505,
    /// AS 2510: Auditing Inventories
    As2510,

    // 2600 Series - Using Work of Others
    /// AS 2601: Consideration of an Entity's Use of a Service Organization
    As2601,
    /// AS 2605: Consideration of the Internal Audit Function
    As2605,
    /// AS 2610: Initial Audits—Communications with Predecessor Auditors
    As2610,

    // 2700 Series - Audit Procedures: Other
    /// AS 2701: Auditing Supplemental Information
    As2701,
    /// AS 2705: Required Supplementary Information
    As2705,
    /// AS 2710: Other Information in Documents Containing Audited Financial Statements
    As2710,

    // 2800 Series - Concluding Audit Procedures
    /// AS 2801: Subsequent Events
    As2801,
    /// AS 2805: Management Representations
    As2805,
    /// AS 2810: Evaluating Audit Results
    As2810,
    /// AS 2815: The Meaning of "Present Fairly in Conformity with GAAP"
    As2815,
    /// AS 2820: Evaluating Consistency of Financial Statements
    As2820,

    // 2900 Series - Going Concern
    /// AS 2901: Consideration of an Entity's Ability to Continue as a Going Concern (Rescinded)
    As2901,

    // 3100 Series - Reporting
    /// AS 3101: The Auditor's Report on an Audit of Financial Statements
    As3101,
    /// AS 3105: Departures from Unqualified Opinions and Other Reporting Circumstances
    As3105,
    /// AS 3110: Dating of the Independent Auditor's Report
    As3110,
    /// AS 3305: Special Reports on Regulated Companies
    As3305,
    /// AS 3310: Special Reports on Regulated Entities—Credit Unions
    As3310,
    /// AS 3315: Reporting on Condensed Financial Statements
    As3315,
    /// AS 3320: Association with Financial Statements
    As3320,
}

impl PcaobStandard {
    /// Get the standard number (e.g., "2110").
    pub fn number(&self) -> &'static str {
        match self {
            Self::As1001 => "1001",
            Self::As1005 => "1005",
            Self::As1010 => "1010",
            Self::As1015 => "1015",
            Self::As1101 => "1101",
            Self::As1105 => "1105",
            Self::As1110 => "1110",
            Self::As1201 => "1201",
            Self::As1205 => "1205",
            Self::As1210 => "1210",
            Self::As1215 => "1215",
            Self::As1220 => "1220",
            Self::As1301 => "1301",
            Self::As1305 => "1305",
            Self::As2101 => "2101",
            Self::As2105 => "2105",
            Self::As2110 => "2110",
            Self::As2201 => "2201",
            Self::As2301 => "2301",
            Self::As2305 => "2305",
            Self::As2310 => "2310",
            Self::As2315 => "2315",
            Self::As2401 => "2401",
            Self::As2405 => "2405",
            Self::As2501 => "2501",
            Self::As2502 => "2502",
            Self::As2503 => "2503",
            Self::As2505 => "2505",
            Self::As2510 => "2510",
            Self::As2601 => "2601",
            Self::As2605 => "2605",
            Self::As2610 => "2610",
            Self::As2701 => "2701",
            Self::As2705 => "2705",
            Self::As2710 => "2710",
            Self::As2801 => "2801",
            Self::As2805 => "2805",
            Self::As2810 => "2810",
            Self::As2815 => "2815",
            Self::As2820 => "2820",
            Self::As2901 => "2901",
            Self::As3101 => "3101",
            Self::As3105 => "3105",
            Self::As3110 => "3110",
            Self::As3305 => "3305",
            Self::As3310 => "3310",
            Self::As3315 => "3315",
            Self::As3320 => "3320",
        }
    }

    /// Get the full title of the standard.
    pub fn title(&self) -> &'static str {
        match self {
            Self::As1001 => "Responsibilities and Functions of the Independent Auditor",
            Self::As1005 => "Independence",
            Self::As1010 => "Training and Proficiency of the Independent Auditor",
            Self::As1015 => "Due Professional Care in the Performance of Work",
            Self::As1101 => "Audit Risk",
            Self::As1105 => "Audit Evidence",
            Self::As1110 => "Relationship of Auditing Standards to Quality Control Standards",
            Self::As1201 => "Supervision of the Audit Engagement",
            Self::As1205 => "Part of Audit Performed by Other Independent Auditors",
            Self::As1210 => "Using the Work of a Specialist",
            Self::As1215 => "Audit Documentation",
            Self::As1220 => "Engagement Quality Review",
            Self::As1301 => "Communications with Audit Committees",
            Self::As1305 => "Communications About Control Deficiencies",
            Self::As2101 => "Audit Planning",
            Self::As2105 => "Consideration of Materiality in Planning and Performing an Audit",
            Self::As2110 => "Identifying and Assessing Risks of Material Misstatement",
            Self::As2201 => "An Audit of Internal Control Over Financial Reporting",
            Self::As2301 => "The Auditor's Responses to the Risks of Material Misstatement",
            Self::As2305 => "Substantive Analytical Procedures",
            Self::As2310 => "The Confirmation Process",
            Self::As2315 => "Audit Sampling",
            Self::As2401 => "Consideration of Fraud in a Financial Statement Audit",
            Self::As2405 => "Illegal Acts by Clients",
            Self::As2501 => "Auditing Accounting Estimates",
            Self::As2502 => "Auditing Fair Value Measurements and Disclosures",
            Self::As2503 => "Auditing Derivative Instruments",
            Self::As2505 => "Inquiry of a Client's Lawyer",
            Self::As2510 => "Auditing Inventories",
            Self::As2601 => "Consideration of an Entity's Use of a Service Organization",
            Self::As2605 => "Consideration of the Internal Audit Function",
            Self::As2610 => "Initial Audits—Communications with Predecessor Auditors",
            Self::As2701 => "Auditing Supplemental Information",
            Self::As2705 => "Required Supplementary Information",
            Self::As2710 => {
                "Other Information in Documents Containing Audited Financial Statements"
            }
            Self::As2801 => "Subsequent Events",
            Self::As2805 => "Management Representations",
            Self::As2810 => "Evaluating Audit Results",
            Self::As2815 => "The Meaning of 'Present Fairly in Conformity with GAAP'",
            Self::As2820 => "Evaluating Consistency of Financial Statements",
            Self::As2901 => "Consideration of an Entity's Ability to Continue as a Going Concern",
            Self::As3101 => "The Auditor's Report on an Audit of Financial Statements",
            Self::As3105 => {
                "Departures from Unqualified Opinions and Other Reporting Circumstances"
            }
            Self::As3110 => "Dating of the Independent Auditor's Report",
            Self::As3305 => "Special Reports on Regulated Companies",
            Self::As3310 => "Special Reports on Regulated Entities—Credit Unions",
            Self::As3315 => "Reporting on Condensed Financial Statements",
            Self::As3320 => "Association with Financial Statements",
        }
    }

    /// Get the series this standard belongs to.
    pub fn series(&self) -> PcaobSeries {
        let num: u32 = self.number().parse().unwrap_or(0);
        match num / 100 {
            10..=11 => PcaobSeries::GeneralStandards,
            12..=13 => PcaobSeries::GeneralActivities,
            21 => PcaobSeries::AuditProceduresGeneral,
            22 => PcaobSeries::AuditingInternalControl,
            23 => PcaobSeries::AuditProceduresResponse,
            24 => PcaobSeries::FraudAndIllegalActs,
            25 => PcaobSeries::SpecificAreas,
            26 => PcaobSeries::UsingWorkOfOthers,
            27 => PcaobSeries::OtherProcedures,
            28 => PcaobSeries::ConcludingProcedures,
            29 => PcaobSeries::GoingConcern,
            31..=33 => PcaobSeries::Reporting,
            _ => PcaobSeries::GeneralStandards,
        }
    }
}

impl std::fmt::Display for PcaobStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AS {}", self.number())
    }
}

/// PCAOB Series groupings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PcaobSeries {
    /// 1000-1100 Series: General Auditing Standards
    GeneralStandards,
    /// 1200-1300 Series: General Activities
    GeneralActivities,
    /// 2100 Series: Audit Procedures - General
    AuditProceduresGeneral,
    /// 2200 Series: Auditing Internal Control
    AuditingInternalControl,
    /// 2300 Series: Response to Risks
    AuditProceduresResponse,
    /// 2400 Series: Fraud and Illegal Acts
    FraudAndIllegalActs,
    /// 2500 Series: Specific Areas
    SpecificAreas,
    /// 2600 Series: Using Work of Others
    UsingWorkOfOthers,
    /// 2700 Series: Other Procedures
    OtherProcedures,
    /// 2800 Series: Concluding Procedures
    ConcludingProcedures,
    /// 2900 Series: Going Concern
    GoingConcern,
    /// 3100-3300 Series: Reporting
    Reporting,
}

impl std::fmt::Display for PcaobSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneralStandards => write!(f, "General Auditing Standards"),
            Self::GeneralActivities => write!(f, "General Activities"),
            Self::AuditProceduresGeneral => write!(f, "Audit Procedures - General"),
            Self::AuditingInternalControl => write!(f, "Auditing Internal Control"),
            Self::AuditProceduresResponse => write!(f, "Response to Risks"),
            Self::FraudAndIllegalActs => write!(f, "Fraud and Illegal Acts"),
            Self::SpecificAreas => write!(f, "Specific Areas"),
            Self::UsingWorkOfOthers => write!(f, "Using Work of Others"),
            Self::OtherProcedures => write!(f, "Other Procedures"),
            Self::ConcludingProcedures => write!(f, "Concluding Procedures"),
            Self::GoingConcern => write!(f, "Going Concern"),
            Self::Reporting => write!(f, "Reporting"),
        }
    }
}

/// Mapping between PCAOB and ISA standards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcaobIsaMapping {
    /// PCAOB standard.
    pub pcaob_standard: PcaobStandard,

    /// Corresponding ISA standard(s).
    pub isa_standards: Vec<IsaStandard>,

    /// Key differences between the standards.
    pub key_differences: Vec<String>,

    /// Areas of substantial similarity.
    pub similarities: Vec<String>,

    /// PCAOB-specific requirements not in ISA.
    pub pcaob_specific: Vec<String>,

    /// Notes on practical application.
    pub application_notes: String,
}

impl PcaobIsaMapping {
    /// Create a new mapping.
    pub fn new(pcaob_standard: PcaobStandard) -> Self {
        Self {
            pcaob_standard,
            isa_standards: Vec::new(),
            key_differences: Vec::new(),
            similarities: Vec::new(),
            pcaob_specific: Vec::new(),
            application_notes: String::new(),
        }
    }

    /// Get standard PCAOB-ISA mappings.
    pub fn standard_mappings() -> Vec<Self> {
        vec![
            // Risk Assessment
            Self {
                pcaob_standard: PcaobStandard::As2110,
                isa_standards: vec![IsaStandard::Isa315],
                key_differences: vec![
                    "PCAOB requires documentation of understanding of internal control".to_string(),
                    "PCAOB has specific requirements for fraud risk factors".to_string(),
                ],
                similarities: vec![
                    "Both require understanding the entity and its environment".to_string(),
                    "Both require identifying and assessing RoMM".to_string(),
                ],
                pcaob_specific: vec!["More detailed documentation requirements".to_string()],
                application_notes: "Substantially aligned with differences in documentation"
                    .to_string(),
            },
            // Audit Evidence
            Self {
                pcaob_standard: PcaobStandard::As1105,
                isa_standards: vec![IsaStandard::Isa500],
                key_differences: vec![
                    "PCAOB emphasizes sufficiency and appropriateness".to_string()
                ],
                similarities: vec!["Similar requirements for audit evidence".to_string()],
                pcaob_specific: vec![],
                application_notes: "Closely aligned".to_string(),
            },
            // Confirmations
            Self {
                pcaob_standard: PcaobStandard::As2310,
                isa_standards: vec![IsaStandard::Isa505],
                key_differences: vec![
                    "Similar requirements with minor wording differences".to_string()
                ],
                similarities: vec!["Both address external confirmation procedures".to_string()],
                pcaob_specific: vec![],
                application_notes: "Substantially converged".to_string(),
            },
            // Audit Documentation
            Self {
                pcaob_standard: PcaobStandard::As1215,
                isa_standards: vec![IsaStandard::Isa230],
                key_differences: vec![
                    "PCAOB has specific retention requirements (7 years)".to_string(),
                    "PCAOB 45-day assembly deadline".to_string(),
                ],
                similarities: vec![
                    "Both require documentation of procedures and conclusions".to_string()
                ],
                pcaob_specific: vec!["Specific archive requirements".to_string()],
                application_notes: "PCAOB has more prescriptive documentation requirements"
                    .to_string(),
            },
            // Audit Report
            Self {
                pcaob_standard: PcaobStandard::As3101,
                isa_standards: vec![IsaStandard::Isa700, IsaStandard::Isa701],
                key_differences: vec![
                    "PCAOB requires Critical Audit Matters (CAMs)".to_string(),
                    "PCAOB requires auditor tenure disclosure".to_string(),
                    "Different report formatting requirements".to_string(),
                ],
                similarities: vec![
                    "Both address auditor's opinion on financial statements".to_string(),
                    "CAMs are similar to ISA 701 Key Audit Matters".to_string(),
                ],
                pcaob_specific: vec![
                    "PCAOB requires ICFR opinion for accelerated filers".to_string()
                ],
                application_notes: "Significant differences in report content and format"
                    .to_string(),
            },
            // ICFR (PCAOB-specific)
            Self {
                pcaob_standard: PcaobStandard::As2201,
                isa_standards: vec![], // No direct ISA equivalent
                key_differences: vec![
                    "No direct ISA equivalent - PCAOB-specific requirement".to_string()
                ],
                similarities: vec![],
                pcaob_specific: vec![
                    "Integrated audit of ICFR required for US public companies".to_string(),
                    "Opinion on effectiveness of internal control".to_string(),
                    "Material weakness identification and reporting".to_string(),
                ],
                application_notes: "US-specific requirement under SOX 404".to_string(),
            },
            // Fraud
            Self {
                pcaob_standard: PcaobStandard::As2401,
                isa_standards: vec![IsaStandard::Isa240],
                key_differences: vec!["PCAOB has more detailed fraud risk factors".to_string()],
                similarities: vec![
                    "Both require consideration of fraud in every audit".to_string(),
                    "Both require professional skepticism".to_string(),
                ],
                pcaob_specific: vec![],
                application_notes: "Substantially similar with different organization".to_string(),
            },
            // Analytical Procedures
            Self {
                pcaob_standard: PcaobStandard::As2305,
                isa_standards: vec![IsaStandard::Isa520],
                key_differences: vec!["Similar requirements".to_string()],
                similarities: vec![
                    "Both require analytical procedures at planning and final review".to_string(),
                ],
                pcaob_specific: vec![],
                application_notes: "Closely aligned".to_string(),
            },
        ]
    }
}

/// Audit framework selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuditFramework {
    /// International Standards on Auditing only.
    #[default]
    IsaOnly,
    /// PCAOB standards only.
    PcaobOnly,
    /// Dual framework - both ISA and PCAOB.
    Dual,
}

impl std::fmt::Display for AuditFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsaOnly => write!(f, "ISA"),
            Self::PcaobOnly => write!(f, "PCAOB"),
            Self::Dual => write!(f, "ISA + PCAOB"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_pcaob_standard_number() {
        assert_eq!(PcaobStandard::As2110.number(), "2110");
        assert_eq!(PcaobStandard::As2201.number(), "2201");
        assert_eq!(PcaobStandard::As3101.number(), "3101");
    }

    #[test]
    fn test_pcaob_standard_series() {
        assert_eq!(
            PcaobStandard::As1015.series(),
            PcaobSeries::GeneralStandards
        );
        assert_eq!(
            PcaobStandard::As2110.series(),
            PcaobSeries::AuditProceduresGeneral
        );
        assert_eq!(
            PcaobStandard::As2201.series(),
            PcaobSeries::AuditingInternalControl
        );
        assert_eq!(PcaobStandard::As3101.series(), PcaobSeries::Reporting);
    }

    #[test]
    fn test_pcaob_isa_mapping() {
        let mappings = PcaobIsaMapping::standard_mappings();

        // Check AS 2110 -> ISA 315 mapping exists
        let risk_mapping = mappings
            .iter()
            .find(|m| m.pcaob_standard == PcaobStandard::As2110);
        assert!(risk_mapping.is_some());
        assert!(risk_mapping
            .unwrap()
            .isa_standards
            .contains(&IsaStandard::Isa315));

        // Check AS 2201 has no ISA equivalent
        let icfr_mapping = mappings
            .iter()
            .find(|m| m.pcaob_standard == PcaobStandard::As2201);
        assert!(icfr_mapping.is_some());
        assert!(icfr_mapping.unwrap().isa_standards.is_empty());
    }
}
