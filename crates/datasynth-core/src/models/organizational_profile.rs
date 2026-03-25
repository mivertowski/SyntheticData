//! Organizational profile models for entity-level understanding.
//!
//! These models describe the IT landscape, regulatory environment, and
//! structural characteristics of an audited entity (ISA 315 risk assessment).

use serde::{Deserialize, Serialize};

/// An IT system in the entity's technology landscape.
///
/// Auditors assess IT general controls (ITGCs) and application controls
/// around each significant system that processes financial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItSystem {
    /// System name (e.g., "SAP S/4HANA", "Oracle EBS")
    pub name: String,
    /// Vendor (e.g., "SAP", "Oracle", "Microsoft")
    pub vendor: String,
    /// Functional module (e.g., "ERP", "CRM", "HCM", "Treasury")
    pub module: String,
    /// Category: "core_financial", "operational", or "reporting"
    pub category: String,
}

/// High-level organizational profile of an audited entity.
///
/// Captures the IT systems, regulatory environment, prior auditor, and
/// organizational structure that inform the auditor's risk assessment
/// under ISA 315 and engagement acceptance under ISA 220.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalProfile {
    /// Entity / company code this profile describes
    pub entity_code: String,
    /// IT systems in use across the entity
    #[serde(default)]
    pub it_systems: Vec<ItSystem>,
    /// Applicable regulatory regimes (e.g., "SOX", "GDPR", "Basel III")
    #[serde(default)]
    pub regulatory_environment: Vec<String>,
    /// Name of the predecessor audit firm, if any
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_auditor: Option<String>,
    /// Narrative description of the organizational structure
    #[serde(default)]
    pub org_structure_description: String,
}
