//! Compliance findings and deficiency classification.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::assertion::ComplianceAssertion;
use super::standard_id::StandardId;

/// Deficiency severity level per SOX/ISA classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeficiencyLevel {
    /// Reasonable possibility of material misstatement not being prevented/detected
    MaterialWeakness,
    /// Important enough to merit attention of those charged with governance
    SignificantDeficiency,
    /// Design or operation deficiency that does not rise to significant deficiency
    ControlDeficiency,
}

impl DeficiencyLevel {
    /// Returns a numeric severity score for ML features.
    pub fn severity_score(&self) -> f64 {
        match self {
            Self::MaterialWeakness => 1.0,
            Self::SignificantDeficiency => 0.66,
            Self::ControlDeficiency => 0.33,
        }
    }
}

impl std::fmt::Display for DeficiencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaterialWeakness => write!(f, "Material Weakness"),
            Self::SignificantDeficiency => write!(f, "Significant Deficiency"),
            Self::ControlDeficiency => write!(f, "Control Deficiency"),
        }
    }
}

/// Finding severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// High severity — likely material
    High,
    /// Moderate severity — potentially significant
    Moderate,
    /// Low severity — minor issue
    Low,
}

impl FindingSeverity {
    /// Returns a numeric score for ML features.
    pub fn score(&self) -> f64 {
        match self {
            Self::High => 1.0,
            Self::Moderate => 0.66,
            Self::Low => 0.33,
        }
    }
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Low => write!(f, "Low"),
        }
    }
}

/// Remediation status of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    /// Finding is open; no action taken
    Open,
    /// Remediation is in progress
    InProgress,
    /// Finding has been remediated and retested
    Remediated,
}

impl std::fmt::Display for RemediationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Remediated => write!(f, "Remediated"),
        }
    }
}

/// A compliance finding from an audit procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    /// Unique finding identifier
    pub finding_id: Uuid,
    /// Company code
    pub company_code: String,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Finding severity
    pub severity: FindingSeverity,
    /// Deficiency classification (SOX)
    pub deficiency_level: DeficiencyLevel,
    /// Control ID where finding was identified
    pub control_id: Option<String>,
    /// Procedure that identified this finding
    pub procedure_id: Option<String>,
    /// Affected audit assertions
    pub affected_assertions: Vec<ComplianceAssertion>,
    /// Related standards
    pub related_standards: Vec<StandardId>,
    /// Date finding was identified
    pub identified_date: NaiveDate,
    /// Remediation status
    pub remediation_status: RemediationStatus,
    /// Estimated financial impact
    pub financial_impact: Option<Decimal>,
    /// Whether this is a repeat finding from a prior period
    pub is_repeat: bool,
    /// Account codes affected
    pub affected_accounts: Vec<String>,
    /// Fiscal year
    pub fiscal_year: i32,
}

impl ComplianceFinding {
    /// Creates a new compliance finding.
    pub fn new(
        company_code: impl Into<String>,
        title: impl Into<String>,
        severity: FindingSeverity,
        deficiency_level: DeficiencyLevel,
        identified_date: NaiveDate,
    ) -> Self {
        Self {
            finding_id: Uuid::new_v4(),
            company_code: company_code.into(),
            title: title.into(),
            description: String::new(),
            severity,
            deficiency_level,
            control_id: None,
            procedure_id: None,
            affected_assertions: Vec::new(),
            related_standards: Vec::new(),
            identified_date,
            remediation_status: RemediationStatus::Open,
            financial_impact: None,
            is_repeat: false,
            affected_accounts: Vec::new(),
            fiscal_year: identified_date.year(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Links to a control.
    pub fn on_control(mut self, control_id: impl Into<String>) -> Self {
        self.control_id = Some(control_id.into());
        self
    }

    /// Links to a procedure.
    pub fn identified_by(mut self, procedure_id: impl Into<String>) -> Self {
        self.procedure_id = Some(procedure_id.into());
        self
    }

    /// Adds an affected assertion.
    pub fn with_assertion(mut self, assertion: ComplianceAssertion) -> Self {
        self.affected_assertions.push(assertion);
        self
    }

    /// Adds a related standard.
    pub fn with_standard(mut self, id: StandardId) -> Self {
        self.related_standards.push(id);
        self
    }

    /// Sets the remediation status.
    pub fn with_remediation(mut self, status: RemediationStatus) -> Self {
        self.remediation_status = status;
        self
    }

    /// Marks as a repeat finding.
    pub fn as_repeat(mut self) -> Self {
        self.is_repeat = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).expect("valid date");
        let finding = ComplianceFinding::new(
            "C001",
            "Three-way match exception",
            FindingSeverity::Moderate,
            DeficiencyLevel::SignificantDeficiency,
            date,
        )
        .on_control("C010")
        .with_assertion(ComplianceAssertion::Occurrence)
        .with_standard(StandardId::new("SOX", "404"));

        assert_eq!(finding.severity, FindingSeverity::Moderate);
        assert_eq!(finding.control_id.as_deref(), Some("C010"));
        assert_eq!(finding.related_standards.len(), 1);
    }

    #[test]
    fn test_deficiency_severity_ordering() {
        assert!(
            DeficiencyLevel::MaterialWeakness.severity_score()
                > DeficiencyLevel::SignificantDeficiency.severity_score()
        );
        assert!(
            DeficiencyLevel::SignificantDeficiency.severity_score()
                > DeficiencyLevel::ControlDeficiency.severity_score()
        );
    }
}
