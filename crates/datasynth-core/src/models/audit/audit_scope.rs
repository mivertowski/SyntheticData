//! Audit scope model (ISA 220 / ISA 300 planning).
//!
//! An [`AuditScope`] defines the boundaries of an audit engagement:
//! which entity is in scope, which financial statement areas are covered,
//! and what the applicable materiality threshold is.
//!
//! Each scope record links to an [`AuditEngagement`] and is referenced
//! by [`CombinedRiskAssessment`] records to indicate the planning boundary.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Minimal audit scope record describing what the engagement covers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditScope {
    /// Unique scope identifier (e.g. "SCOPE-AUD-2025-001-C001").
    pub id: String,
    /// Engagement this scope belongs to (FK → AuditEngagement.engagement_id).
    pub engagement_id: String,
    /// Entity / company code in scope.
    pub entity_code: String,
    /// Financial statement areas / audit focus areas covered by this scope.
    pub scope_areas: Vec<String>,
    /// Planning materiality threshold applicable to this scope.
    pub materiality: Decimal,
}

impl AuditScope {
    /// Standard scope areas used when no specific areas are configured.
    pub const DEFAULT_SCOPE_AREAS: &'static [&'static str] = &[
        "Revenue",
        "Accounts Receivable",
        "Inventory",
        "Property Plant & Equipment",
        "Accounts Payable",
        "Payroll",
        "Cash & Bank",
        "Financial Reporting Close",
    ];

    /// Create a new `AuditScope` with default scope areas.
    pub fn new(
        id: impl Into<String>,
        engagement_id: impl Into<String>,
        entity_code: impl Into<String>,
        materiality: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            engagement_id: engagement_id.into(),
            entity_code: entity_code.into(),
            scope_areas: Self::DEFAULT_SCOPE_AREAS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            materiality,
        }
    }

    /// Create an `AuditScope` with explicit scope areas.
    pub fn with_areas(
        id: impl Into<String>,
        engagement_id: impl Into<String>,
        entity_code: impl Into<String>,
        scope_areas: Vec<String>,
        materiality: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            engagement_id: engagement_id.into(),
            entity_code: entity_code.into(),
            scope_areas,
            materiality,
        }
    }
}
