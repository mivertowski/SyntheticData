//! Jurisdiction-specific compliance profiles.

use serde::{Deserialize, Serialize};

use super::standard_id::StandardId;

/// Supranational body membership that propagates regulatory obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupranationalBody {
    /// European Union
    Eu,
    /// European Economic Area
    Eea,
    /// Eurozone (single currency)
    Eurozone,
    /// Association of Southeast Asian Nations
    Asean,
    /// Gulf Cooperation Council
    Gcc,
    /// Southern Common Market
    Mercosur,
}

impl std::fmt::Display for SupranationalBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eu => write!(f, "EU"),
            Self::Eea => write!(f, "EEA"),
            Self::Eurozone => write!(f, "Eurozone"),
            Self::Asean => write!(f, "ASEAN"),
            Self::Gcc => write!(f, "GCC"),
            Self::Mercosur => write!(f, "Mercosur"),
        }
    }
}

/// Accounting framework for a jurisdiction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JurisdictionAccountingFramework {
    /// US GAAP (ASC Codification)
    UsGaap,
    /// Full IFRS adoption
    Ifrs,
    /// IFRS-converged local GAAP (e.g., Ind AS)
    IfrsConverged,
    /// Local GAAP with IFRS for listed entities
    LocalGaapWithIfrs,
    /// Local GAAP only
    LocalGaap,
}

/// Audit framework for a jurisdiction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditFramework {
    /// ISA (International Standards on Auditing) adopted directly
    Isa,
    /// ISA with local modifications (e.g., ISA (UK), ISA [DE])
    IsaLocal,
    /// PCAOB standards (US public companies)
    Pcaob,
    /// Local auditing standards (ISA-based)
    LocalIsaBased,
}

/// Entity type for applicability criteria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// SEC-registered (US)
    SecRegistrant,
    /// Accelerated filer (US)
    AcceleratedFiler,
    /// Large accelerated filer (US)
    LargeAcceleratedFiler,
    /// Public interest entity (EU/intl)
    PublicInterestEntity,
    /// Listed on stock exchange
    ListedEntity,
    /// Large entity (above thresholds)
    LargeEntity,
    /// Small/medium enterprise
    Sme,
    /// Micro entity
    MicroEntity,
    /// Financial institution / bank
    FinancialInstitution,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SecRegistrant => write!(f, "SEC Registrant"),
            Self::AcceleratedFiler => write!(f, "Accelerated Filer"),
            Self::LargeAcceleratedFiler => write!(f, "Large Accelerated Filer"),
            Self::PublicInterestEntity => write!(f, "Public Interest Entity"),
            Self::ListedEntity => write!(f, "Listed Entity"),
            Self::LargeEntity => write!(f, "Large Entity"),
            Self::Sme => write!(f, "SME"),
            Self::MicroEntity => write!(f, "Micro Entity"),
            Self::FinancialInstitution => write!(f, "Financial Institution"),
        }
    }
}

/// How a standard applies in a specific jurisdiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionStandard {
    /// Standard identifier
    pub standard_id: StandardId,
    /// Local effective date (may differ from global)
    pub local_effective_date: Option<String>,
    /// Local name or designation (e.g., "Ind AS 116" for IFRS 16 in India)
    pub local_designation: Option<String>,
    /// Applicability scope
    pub applicability: Vec<EntityType>,
}

/// A jurisdiction's complete compliance profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionProfile {
    /// ISO 3166-1 alpha-2 country code
    pub country_code: String,
    /// Jurisdiction display name
    pub country_name: String,
    /// Supranational memberships
    pub memberships: Vec<SupranationalBody>,
    /// Primary accounting framework
    pub accounting_framework: JurisdictionAccountingFramework,
    /// Primary audit framework
    pub audit_framework: AuditFramework,
    /// Standards body name (e.g., "FASB", "DRSC", "ANC")
    pub accounting_standards_body: String,
    /// Audit oversight body (e.g., "PCAOB", "IDW", "H3C")
    pub audit_oversight_body: String,
    /// Securities regulator (e.g., "SEC", "BaFin", "AMF")
    pub securities_regulator: Option<String>,
    /// Stock exchanges
    pub stock_exchanges: Vec<String>,
    /// Mandatory standards in this jurisdiction
    pub mandatory_standards: Vec<JurisdictionStandard>,
    /// Corporate tax rate
    pub corporate_tax_rate: Option<f64>,
    /// Currency code (ISO 4217)
    pub currency: String,
    /// Whether IFRS is required for listed entities
    pub ifrs_required_for_listed: bool,
    /// Whether e-invoicing is mandatory
    pub e_invoicing_mandatory: bool,
    /// Required audit export format (e.g., "fec", "gobd")
    pub audit_export_format: Option<String>,
}

impl JurisdictionProfile {
    /// Creates a minimal jurisdiction profile.
    pub fn new(
        country_code: impl Into<String>,
        country_name: impl Into<String>,
        accounting_framework: JurisdictionAccountingFramework,
        audit_framework: AuditFramework,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            country_code: country_code.into(),
            country_name: country_name.into(),
            memberships: Vec::new(),
            accounting_framework,
            audit_framework,
            accounting_standards_body: String::new(),
            audit_oversight_body: String::new(),
            securities_regulator: None,
            stock_exchanges: Vec::new(),
            mandatory_standards: Vec::new(),
            corporate_tax_rate: None,
            currency: currency.into(),
            ifrs_required_for_listed: false,
            e_invoicing_mandatory: false,
            audit_export_format: None,
        }
    }

    /// Returns true if this jurisdiction is an EU member.
    pub fn is_eu_member(&self) -> bool {
        self.memberships.contains(&SupranationalBody::Eu)
    }

    /// Returns true if this jurisdiction is in the Eurozone.
    pub fn is_eurozone(&self) -> bool {
        self.memberships.contains(&SupranationalBody::Eurozone)
    }

    /// Adds a supranational membership.
    pub fn with_membership(mut self, body: SupranationalBody) -> Self {
        self.memberships.push(body);
        self
    }

    /// Adds a mandatory standard.
    pub fn with_mandatory_standard(mut self, js: JurisdictionStandard) -> Self {
        self.mandatory_standards.push(js);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jurisdiction_profile_creation() {
        let profile = JurisdictionProfile::new(
            "DE",
            "Federal Republic of Germany",
            JurisdictionAccountingFramework::LocalGaapWithIfrs,
            AuditFramework::IsaLocal,
            "EUR",
        )
        .with_membership(SupranationalBody::Eu)
        .with_membership(SupranationalBody::Eurozone);

        assert!(profile.is_eu_member());
        assert!(profile.is_eurozone());
        assert_eq!(profile.country_code, "DE");
    }
}
