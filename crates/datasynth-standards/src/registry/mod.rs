//! Standard Registry — Central catalog of all compliance standards.
//!
//! The registry provides:
//! - Canonical lookup by [`StandardId`]
//! - Temporal resolution (active version at a given date)
//! - Cross-reference traversal
//! - Jurisdiction-aware standard filtering
//! - Supersession chain resolution

mod built_in;

use chrono::NaiveDate;
use std::collections::HashMap;

use datasynth_core::models::compliance::{
    ComplianceDomain, ComplianceStandard, CrossReference, JurisdictionProfile, StandardCategory,
    StandardId, TemporalVersion,
};

pub use built_in::register_built_in_standards;

/// The central standard registry.
#[derive(Debug, Default)]
pub struct StandardRegistry {
    /// All registered standards indexed by canonical ID.
    standards: HashMap<StandardId, ComplianceStandard>,
    /// Jurisdiction profiles indexed by country code.
    jurisdictions: HashMap<String, JurisdictionProfile>,
    /// Cross-reference index: standard → related standards.
    cross_references: Vec<CrossReference>,
}

impl StandardRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry pre-populated with built-in standards.
    pub fn with_built_in() -> Self {
        let mut registry = Self::new();
        built_in::register_built_in_standards(&mut registry);
        registry
    }

    /// Registers a compliance standard.
    pub fn register_standard(&mut self, standard: ComplianceStandard) {
        self.standards.insert(standard.id.clone(), standard);
    }

    /// Registers a jurisdiction profile.
    pub fn register_jurisdiction(&mut self, profile: JurisdictionProfile) {
        self.jurisdictions
            .insert(profile.country_code.clone(), profile);
    }

    /// Adds a cross-reference between two standards.
    pub fn add_cross_reference(&mut self, xref: CrossReference) {
        self.cross_references.push(xref);
    }

    /// Looks up a standard by ID.
    pub fn get(&self, id: &StandardId) -> Option<&ComplianceStandard> {
        self.standards.get(id)
    }

    /// Looks up a jurisdiction profile by country code.
    pub fn jurisdiction(&self, country_code: &str) -> Option<&JurisdictionProfile> {
        self.jurisdictions.get(country_code)
    }

    /// Returns all registered standards.
    pub fn all_standards(&self) -> impl Iterator<Item = &ComplianceStandard> {
        self.standards.values()
    }

    /// Returns all registered jurisdiction profiles.
    pub fn all_jurisdictions(&self) -> impl Iterator<Item = &JurisdictionProfile> {
        self.jurisdictions.values()
    }

    /// Returns the number of registered standards.
    pub fn standard_count(&self) -> usize {
        self.standards.len()
    }

    /// Returns the number of registered jurisdictions.
    pub fn jurisdiction_count(&self) -> usize {
        self.jurisdictions.len()
    }

    /// Returns all cross-references.
    pub fn cross_references(&self) -> &[CrossReference] {
        &self.cross_references
    }

    /// Gets the active version of a standard at a given date.
    pub fn active_version(&self, id: &StandardId, at: NaiveDate) -> Option<&TemporalVersion> {
        self.standards
            .get(id)
            .and_then(|std| std.versions.iter().find(|v| v.is_active_at(at)))
    }

    /// Gets the active version for a specific jurisdiction at a given date.
    pub fn active_version_in(
        &self,
        id: &StandardId,
        country: &str,
        at: NaiveDate,
    ) -> Option<&TemporalVersion> {
        self.standards
            .get(id)
            .and_then(|std| std.versions.iter().find(|v| v.is_active_at_in(at, country)))
    }

    /// Returns all standards applicable in a jurisdiction at a given date.
    pub fn standards_for_jurisdiction(
        &self,
        country: &str,
        at: NaiveDate,
    ) -> Vec<&ComplianceStandard> {
        self.standards
            .values()
            .filter(|std| {
                // Standard is mandatory in this jurisdiction
                let is_mandatory = std.mandatory_jurisdictions.contains(&country.to_string())
                    || std.mandatory_jurisdictions.contains(&"*".to_string());

                // Standard has an active version at this date
                let has_active = std.versions.iter().any(|v| v.is_active_at_in(at, country));

                is_mandatory && has_active
            })
            .collect()
    }

    /// Gets cross-references for a specific standard.
    pub fn cross_references_for(&self, id: &StandardId) -> Vec<&CrossReference> {
        self.cross_references
            .iter()
            .filter(|xr| xr.from_standard == *id || xr.to_standard == *id)
            .collect()
    }

    /// Gets the supersession chain for a standard (oldest → newest).
    pub fn supersession_chain(&self, id: &StandardId) -> Vec<&ComplianceStandard> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Walk backwards to find predecessors
        let mut current = id.clone();
        loop {
            if !visited.insert(current.clone()) {
                break; // Cycle detected
            }
            if self.standards.contains_key(&current) {
                let predecessor = self
                    .standards
                    .values()
                    .find(|s| s.superseded_by.as_ref() == Some(&current));
                if let Some(pred) = predecessor {
                    current = pred.id.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Now walk forward from the oldest
        visited.clear();
        while let Some(std) = self.standards.get(&current) {
            if !visited.insert(current.clone()) {
                break; // Cycle detected
            }
            chain.push(std);
            if let Some(ref successor) = std.superseded_by {
                current = successor.clone();
            } else {
                break;
            }
        }

        chain
    }

    /// Queries standards by category.
    pub fn by_category(&self, category: StandardCategory) -> Vec<&ComplianceStandard> {
        self.standards
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Queries standards by domain.
    pub fn by_domain(&self, domain: ComplianceDomain) -> Vec<&ComplianceStandard> {
        self.standards
            .values()
            .filter(|s| s.domain == domain)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_with_built_in() {
        let registry = StandardRegistry::with_built_in();
        assert!(
            registry.standard_count() > 0,
            "Should have built-in standards"
        );
        assert!(
            registry.jurisdiction_count() > 0,
            "Should have jurisdiction profiles"
        );
    }

    #[test]
    fn test_active_version_lookup() {
        let registry = StandardRegistry::with_built_in();
        let ifrs16 = StandardId::new("IFRS", "16");
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).expect("valid");
        let version = registry.active_version(&ifrs16, date);
        assert!(version.is_some(), "IFRS 16 should be active in 2025");
    }

    #[test]
    fn test_cross_references() {
        let registry = StandardRegistry::with_built_in();
        let ifrs15 = StandardId::new("IFRS", "15");
        let xrefs = registry.cross_references_for(&ifrs15);
        assert!(!xrefs.is_empty(), "IFRS 15 should have cross-references");
    }

    #[test]
    fn test_supersession_chain() {
        let registry = StandardRegistry::with_built_in();
        let ifrs16 = StandardId::new("IFRS", "16");
        let chain = registry.supersession_chain(&ifrs16);
        assert!(chain.len() >= 2, "Should have IAS 17 → IFRS 16 chain");
    }
}
