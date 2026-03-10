//! Compliance standard metadata.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::cross_reference::CrossReference;
use super::standard_id::StandardId;
use super::temporal::TemporalVersion;
use crate::models::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Category of compliance standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardCategory {
    /// Accounting standard (IFRS, US GAAP, local GAAP)
    AccountingStandard,
    /// Auditing standard (ISA, PCAOB AS)
    AuditingStandard,
    /// Regulatory requirement (SOX, EU Audit Regulation)
    RegulatoryRequirement,
    /// Reporting standard (XBRL, ESEF)
    ReportingStandard,
    /// Prudential regulation (Basel III/IV, Solvency II)
    PrudentialRegulation,
    /// Tax regulation (BEPS, CRS, FATCA)
    TaxRegulation,
    /// Data protection (GDPR, CCPA)
    DataProtection,
    /// Sustainability standard (CSRD, ISSB, GRI)
    SustainabilityStandard,
}

impl std::fmt::Display for StandardCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountingStandard => write!(f, "Accounting Standard"),
            Self::AuditingStandard => write!(f, "Auditing Standard"),
            Self::RegulatoryRequirement => write!(f, "Regulatory Requirement"),
            Self::ReportingStandard => write!(f, "Reporting Standard"),
            Self::PrudentialRegulation => write!(f, "Prudential Regulation"),
            Self::TaxRegulation => write!(f, "Tax Regulation"),
            Self::DataProtection => write!(f, "Data Protection"),
            Self::SustainabilityStandard => write!(f, "Sustainability Standard"),
        }
    }
}

/// Domain within financial compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceDomain {
    /// Financial reporting and accounting
    FinancialReporting,
    /// Internal control over financial reporting
    InternalControl,
    /// External audit procedures
    ExternalAudit,
    /// Tax compliance and reporting
    TaxCompliance,
    /// Regulatory reporting to authorities
    RegulatoryReporting,
    /// Enterprise risk management
    RiskManagement,
    /// Data governance and protection
    DataGovernance,
    /// ESG and sustainability reporting
    Sustainability,
    /// Anti-money laundering
    AntiMoneyLaundering,
    /// Prudential capital and liquidity
    PrudentialCapital,
}

impl std::fmt::Display for ComplianceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FinancialReporting => write!(f, "Financial Reporting"),
            Self::InternalControl => write!(f, "Internal Control"),
            Self::ExternalAudit => write!(f, "External Audit"),
            Self::TaxCompliance => write!(f, "Tax Compliance"),
            Self::RegulatoryReporting => write!(f, "Regulatory Reporting"),
            Self::RiskManagement => write!(f, "Risk Management"),
            Self::DataGovernance => write!(f, "Data Governance"),
            Self::Sustainability => write!(f, "Sustainability"),
            Self::AntiMoneyLaundering => write!(f, "Anti-Money Laundering"),
            Self::PrudentialCapital => write!(f, "Prudential Capital"),
        }
    }
}

/// Issuing body for a compliance standard.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuingBody {
    /// International Accounting Standards Board
    Iasb,
    /// International Auditing and Assurance Standards Board
    Iaasb,
    /// Financial Accounting Standards Board (US)
    Fasb,
    /// Public Company Accounting Oversight Board (US)
    Pcaob,
    /// Securities and Exchange Commission (US)
    Sec,
    /// Basel Committee on Banking Supervision
    Bcbs,
    /// European Commission / Parliament
    EuropeanUnion,
    /// Financial Reporting Council (UK)
    Frc,
    /// Deutsches Rechnungslegungs Standards Committee
    Drsc,
    /// Institut der Wirtschaftsprüfer (Germany)
    Idw,
    /// Autorité des Normes Comptables (France)
    Anc,
    /// Accounting Standards Board of Japan
    Asbj,
    /// Institute of Chartered Accountants of India
    Icai,
    /// International Sustainability Standards Board
    Issb,
    /// Organisation for Economic Cooperation and Development
    Oecd,
    /// User-defined or other body
    Custom(String),
}

impl std::fmt::Display for IssuingBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iasb => write!(f, "IASB"),
            Self::Iaasb => write!(f, "IAASB"),
            Self::Fasb => write!(f, "FASB"),
            Self::Pcaob => write!(f, "PCAOB"),
            Self::Sec => write!(f, "SEC"),
            Self::Bcbs => write!(f, "BCBS"),
            Self::EuropeanUnion => write!(f, "EU"),
            Self::Frc => write!(f, "FRC"),
            Self::Drsc => write!(f, "DRSC"),
            Self::Idw => write!(f, "IDW"),
            Self::Anc => write!(f, "ANC"),
            Self::Asbj => write!(f, "ASBJ"),
            Self::Icai => write!(f, "ICAI"),
            Self::Issb => write!(f, "ISSB"),
            Self::Oecd => write!(f, "OECD"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// A requirement within a compliance standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRequirement {
    /// Requirement identifier (e.g., "ISA-315.R14")
    pub id: String,
    /// Requirement title
    pub title: String,
    /// Description of the requirement
    pub description: String,
    /// Related audit assertions
    pub assertions: Vec<String>,
}

/// A compliance standard with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    /// Canonical identifier
    pub id: StandardId,
    /// Human-readable title
    pub title: String,
    /// Issuing body
    pub issuing_body: IssuingBody,
    /// Standard category
    pub category: StandardCategory,
    /// Domain area
    pub domain: ComplianceDomain,
    /// All known versions with effective dates
    pub versions: Vec<TemporalVersion>,
    /// Standards this one supersedes
    pub supersedes: Vec<StandardId>,
    /// Standard this one is superseded by (if any)
    pub superseded_by: Option<StandardId>,
    /// Cross-references to related standards
    pub cross_references: Vec<CrossReference>,
    /// Jurisdictions where this standard is mandatory (ISO 3166-1 alpha-2)
    pub mandatory_jurisdictions: Vec<String>,
    /// Jurisdictions where this standard is permitted but optional
    pub permitted_jurisdictions: Vec<String>,
    /// Key requirements within this standard
    pub requirements: Vec<StandardRequirement>,
    /// GL account types this standard applies to (e.g., "Revenue", "Leases", "PP&E")
    #[serde(default)]
    pub applicable_account_types: Vec<String>,
    /// Business processes this standard governs (e.g., "O2C", "P2P", "R2R")
    #[serde(default)]
    pub applicable_processes: Vec<String>,
}

impl ComplianceStandard {
    /// Creates a new compliance standard with minimal required fields.
    pub fn new(
        id: StandardId,
        title: impl Into<String>,
        issuing_body: IssuingBody,
        category: StandardCategory,
        domain: ComplianceDomain,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            issuing_body,
            category,
            domain,
            versions: Vec::new(),
            supersedes: Vec::new(),
            superseded_by: None,
            cross_references: Vec::new(),
            mandatory_jurisdictions: Vec::new(),
            permitted_jurisdictions: Vec::new(),
            requirements: Vec::new(),
            applicable_account_types: Vec::new(),
            applicable_processes: Vec::new(),
        }
    }

    /// Adds a temporal version.
    pub fn with_version(mut self, version: TemporalVersion) -> Self {
        self.versions.push(version);
        self
    }

    /// Adds a superseded standard.
    pub fn supersedes_standard(mut self, id: StandardId) -> Self {
        self.supersedes.push(id);
        self
    }

    /// Sets the superseding standard.
    pub fn superseded_by_standard(mut self, id: StandardId) -> Self {
        self.superseded_by = Some(id);
        self
    }

    /// Adds a cross-reference.
    pub fn with_cross_reference(mut self, xref: CrossReference) -> Self {
        self.cross_references.push(xref);
        self
    }

    /// Adds a mandatory jurisdiction.
    pub fn mandatory_in(mut self, country_code: &str) -> Self {
        self.mandatory_jurisdictions.push(country_code.to_string());
        self
    }

    /// Adds a requirement.
    pub fn with_requirement(mut self, req: StandardRequirement) -> Self {
        self.requirements.push(req);
        self
    }

    /// Adds an applicable GL account type.
    pub fn with_account_type(mut self, account_type: &str) -> Self {
        self.applicable_account_types.push(account_type.to_string());
        self
    }

    /// Adds applicable GL account types in bulk.
    pub fn with_account_types(mut self, types: &[&str]) -> Self {
        self.applicable_account_types
            .extend(types.iter().map(|s| s.to_string()));
        self
    }

    /// Adds an applicable business process.
    pub fn with_process(mut self, process: &str) -> Self {
        self.applicable_processes.push(process.to_string());
        self
    }

    /// Adds applicable business processes in bulk.
    pub fn with_processes(mut self, processes: &[&str]) -> Self {
        self.applicable_processes
            .extend(processes.iter().map(|s| s.to_string()));
        self
    }

    /// Returns true if this standard is currently superseded.
    pub fn is_superseded(&self) -> bool {
        self.superseded_by.is_some()
    }
}

impl ToNodeProperties for ComplianceStandard {
    fn node_type_name(&self) -> &'static str {
        "compliance_standard"
    }
    fn node_type_code(&self) -> u16 {
        510
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "standardId".into(),
            GraphPropertyValue::String(self.id.as_str().to_string()),
        );
        p.insert(
            "title".into(),
            GraphPropertyValue::String(self.title.clone()),
        );
        p.insert(
            "issuingBody".into(),
            GraphPropertyValue::String(self.issuing_body.to_string()),
        );
        p.insert(
            "category".into(),
            GraphPropertyValue::String(self.category.to_string()),
        );
        p.insert(
            "domain".into(),
            GraphPropertyValue::String(self.domain.to_string()),
        );
        p.insert(
            "isSuperseded".into(),
            GraphPropertyValue::Bool(self.is_superseded()),
        );
        p.insert(
            "mandatoryJurisdictions".into(),
            GraphPropertyValue::StringList(self.mandatory_jurisdictions.clone()),
        );
        if !self.applicable_account_types.is_empty() {
            p.insert(
                "applicableAccountTypes".into(),
                GraphPropertyValue::StringList(self.applicable_account_types.clone()),
            );
        }
        if !self.applicable_processes.is_empty() {
            p.insert(
                "applicableProcesses".into(),
                GraphPropertyValue::StringList(self.applicable_processes.clone()),
            );
        }
        p.insert(
            "requirementCount".into(),
            GraphPropertyValue::Int(self.requirements.len() as i64),
        );
        p.insert(
            "versionCount".into(),
            GraphPropertyValue::Int(self.versions.len() as i64),
        );
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_standard_creation() {
        let std = ComplianceStandard::new(
            StandardId::new("IFRS", "16"),
            "Leases",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .mandatory_in("DE")
        .mandatory_in("GB");

        assert_eq!(std.id.as_str(), "IFRS-16");
        assert_eq!(std.mandatory_jurisdictions.len(), 2);
        assert!(!std.is_superseded());
    }
}
