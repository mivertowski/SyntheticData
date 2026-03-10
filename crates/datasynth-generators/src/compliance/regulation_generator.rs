//! Regulation generator — produces compliance registry snapshot data for output.
//!
//! Resolves the active standards for each configured jurisdiction at the reference
//! date and emits serializable records for CSV/JSON export.

use chrono::NaiveDate;
use serde::Serialize;

use datasynth_standards::registry::StandardRegistry;

/// A flattened compliance-standard record suitable for CSV/JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceStandardRecord {
    pub standard_id: String,
    pub body: String,
    pub number: String,
    pub title: String,
    pub category: String,
    pub domain: String,
    pub jurisdiction: String,
    pub effective_date: String,
    pub version: String,
    pub is_active: bool,
    pub superseded_by: Option<String>,
    /// GL account types this standard applies to.
    pub applicable_account_types: Vec<String>,
    /// Business processes this standard governs.
    pub applicable_processes: Vec<String>,
}

/// A flattened cross-reference record.
#[derive(Debug, Clone, Serialize)]
pub struct CrossReferenceRecord {
    pub from_standard: String,
    pub to_standard: String,
    pub relationship: String,
    pub convergence_level: f64,
    pub description: Option<String>,
}

/// A flattened jurisdiction profile record.
#[derive(Debug, Clone, Serialize)]
pub struct JurisdictionRecord {
    pub country_code: String,
    pub country_name: String,
    pub accounting_framework: String,
    pub audit_framework: String,
    pub standards_body: String,
    pub statutory_tax_rate: f64,
    pub standard_count: usize,
}

/// Generator that produces compliance regulation output data from the registry.
pub struct RegulationGenerator {
    registry: StandardRegistry,
}

impl RegulationGenerator {
    /// Creates a new generator with the built-in registry.
    pub fn new() -> Self {
        Self {
            registry: StandardRegistry::with_built_in(),
        }
    }

    /// Creates a generator with a custom registry.
    pub fn with_registry(registry: StandardRegistry) -> Self {
        Self { registry }
    }

    /// Returns a reference to the underlying registry.
    pub fn registry(&self) -> &StandardRegistry {
        &self.registry
    }

    /// Generates standard records for a set of jurisdictions at a reference date.
    pub fn generate_standard_records(
        &self,
        jurisdictions: &[String],
        reference_date: NaiveDate,
    ) -> Vec<ComplianceStandardRecord> {
        let mut records = Vec::new();

        for jurisdiction in jurisdictions {
            let standards = self
                .registry
                .standards_for_jurisdiction(jurisdiction, reference_date);

            for std in standards {
                let active_version =
                    self.registry
                        .active_version_in(&std.id, jurisdiction, reference_date);

                let (effective_date, version) = if let Some(v) = active_version {
                    (v.effective_from.to_string(), v.version_id.clone())
                } else {
                    ("unknown".to_string(), "unknown".to_string())
                };

                records.push(ComplianceStandardRecord {
                    standard_id: std.id.as_str().to_string(),
                    body: std.id.body().to_string(),
                    number: std.id.number().to_string(),
                    title: std.title.clone(),
                    category: format!("{}", std.category),
                    domain: format!("{}", std.domain),
                    jurisdiction: jurisdiction.clone(),
                    effective_date,
                    version,
                    is_active: true,
                    superseded_by: std.superseded_by.as_ref().map(|s| s.as_str().to_string()),
                    applicable_account_types: std.applicable_account_types.clone(),
                    applicable_processes: std.applicable_processes.clone(),
                });
            }
        }

        records
    }

    /// Generates cross-reference records.
    pub fn generate_cross_reference_records(&self) -> Vec<CrossReferenceRecord> {
        self.registry
            .cross_references()
            .iter()
            .map(|xr| CrossReferenceRecord {
                from_standard: xr.from_standard.as_str().to_string(),
                to_standard: xr.to_standard.as_str().to_string(),
                relationship: format!("{}", xr.relationship),
                convergence_level: xr.convergence_level,
                description: xr.description.clone(),
            })
            .collect()
    }

    /// Generates jurisdiction profile records.
    pub fn generate_jurisdiction_records(
        &self,
        jurisdictions: &[String],
        reference_date: NaiveDate,
    ) -> Vec<JurisdictionRecord> {
        jurisdictions
            .iter()
            .filter_map(|code| {
                self.registry.jurisdiction(code).map(|jp| {
                    let standard_count = self
                        .registry
                        .standards_for_jurisdiction(code, reference_date)
                        .len();

                    JurisdictionRecord {
                        country_code: jp.country_code.clone(),
                        country_name: jp.country_name.clone(),
                        accounting_framework: format!("{:?}", jp.accounting_framework),
                        audit_framework: format!("{:?}", jp.audit_framework),
                        standards_body: jp.accounting_standards_body.clone(),
                        statutory_tax_rate: jp.corporate_tax_rate.unwrap_or(0.0),
                        standard_count,
                    }
                })
            })
            .collect()
    }
}

impl Default for RegulationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_standard_records() {
        let gen = RegulationGenerator::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let records = gen.generate_standard_records(&["US".to_string()], date);
        assert!(!records.is_empty(), "Should have US standards");
    }

    #[test]
    fn test_generate_cross_references() {
        let gen = RegulationGenerator::new();
        let records = gen.generate_cross_reference_records();
        assert!(!records.is_empty(), "Should have cross-references");
    }

    #[test]
    fn test_generate_jurisdiction_records() {
        let gen = RegulationGenerator::new();
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let records =
            gen.generate_jurisdiction_records(&["US".to_string(), "DE".to_string()], date);
        assert_eq!(records.len(), 2);
    }
}
