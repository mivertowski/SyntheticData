//! Referential integrity evaluation.
//!
//! Validates that all foreign key references point to valid master data entities
//! and that created entities are actually used in transactions.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Results of referential integrity evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferentialIntegrityEvaluation {
    /// Vendor reference integrity.
    pub vendor_integrity: EntityIntegrity,
    /// Customer reference integrity.
    pub customer_integrity: EntityIntegrity,
    /// Material reference integrity.
    pub material_integrity: EntityIntegrity,
    /// Employee/User reference integrity.
    pub employee_integrity: EntityIntegrity,
    /// Account reference integrity.
    pub account_integrity: EntityIntegrity,
    /// Cost center reference integrity.
    pub cost_center_integrity: EntityIntegrity,
    /// Overall integrity score (0.0-1.0).
    pub overall_integrity_score: f64,
    /// Total valid references.
    pub total_valid_references: usize,
    /// Total invalid references.
    pub total_invalid_references: usize,
    /// Total orphaned entities (created but never used).
    pub total_orphaned_entities: usize,
    /// Passes integrity check.
    pub passes: bool,
}

/// Integrity metrics for a single entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityIntegrity {
    /// Entity type name.
    pub entity_type: String,
    /// Total entities defined.
    pub total_entities: usize,
    /// Entities actually referenced.
    pub entities_referenced: usize,
    /// Valid references count.
    pub valid_references: usize,
    /// Invalid references count.
    pub invalid_references: usize,
    /// Orphaned entities (defined but never used).
    pub orphaned_entities: usize,
    /// Integrity score (0.0-1.0).
    pub integrity_score: f64,
    /// Usage rate (entities_referenced / total_entities).
    pub usage_rate: f64,
}

impl Default for EntityIntegrity {
    fn default() -> Self {
        Self {
            entity_type: String::new(),
            total_entities: 0,
            entities_referenced: 0,
            valid_references: 0,
            invalid_references: 0,
            orphaned_entities: 0,
            integrity_score: 1.0,
            usage_rate: 1.0,
        }
    }
}

/// Input data for referential integrity evaluation.
#[derive(Debug, Clone, Default)]
pub struct ReferentialData {
    /// Vendor reference data.
    pub vendors: EntityReferenceData,
    /// Customer reference data.
    pub customers: EntityReferenceData,
    /// Material reference data.
    pub materials: EntityReferenceData,
    /// Employee reference data.
    pub employees: EntityReferenceData,
    /// Account reference data.
    pub accounts: EntityReferenceData,
    /// Cost center reference data.
    pub cost_centers: EntityReferenceData,
}

/// Reference data for a single entity type.
#[derive(Debug, Clone, Default)]
pub struct EntityReferenceData {
    /// Set of all valid entity IDs.
    pub valid_ids: std::collections::HashSet<String>,
    /// List of all references made to this entity type.
    pub references: Vec<String>,
}

impl EntityReferenceData {
    /// Create new entity reference data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a valid entity ID.
    pub fn add_entity(&mut self, id: String) {
        self.valid_ids.insert(id);
    }

    /// Add a reference to this entity type.
    pub fn add_reference(&mut self, id: String) {
        self.references.push(id);
    }
}

/// Evaluator for referential integrity.
pub struct ReferentialIntegrityEvaluator {
    /// Minimum integrity score threshold.
    min_integrity_score: f64,
    /// Minimum usage rate threshold.
    #[allow(dead_code)]
    min_usage_rate: f64,
}

impl ReferentialIntegrityEvaluator {
    /// Create a new evaluator with specified thresholds.
    pub fn new(min_integrity_score: f64, min_usage_rate: f64) -> Self {
        Self {
            min_integrity_score,
            min_usage_rate,
        }
    }

    /// Evaluate referential integrity.
    pub fn evaluate(&self, data: &ReferentialData) -> EvalResult<ReferentialIntegrityEvaluation> {
        let vendor_integrity = self.evaluate_entity("Vendor", &data.vendors);
        let customer_integrity = self.evaluate_entity("Customer", &data.customers);
        let material_integrity = self.evaluate_entity("Material", &data.materials);
        let employee_integrity = self.evaluate_entity("Employee", &data.employees);
        let account_integrity = self.evaluate_entity("Account", &data.accounts);
        let cost_center_integrity = self.evaluate_entity("CostCenter", &data.cost_centers);

        // Aggregate totals
        let integrities = [
            &vendor_integrity,
            &customer_integrity,
            &material_integrity,
            &employee_integrity,
            &account_integrity,
            &cost_center_integrity,
        ];

        let total_valid_references: usize = integrities.iter().map(|i| i.valid_references).sum();
        let total_invalid_references: usize =
            integrities.iter().map(|i| i.invalid_references).sum();
        let total_orphaned_entities: usize = integrities.iter().map(|i| i.orphaned_entities).sum();

        // Calculate overall integrity score (weighted by reference count)
        let total_refs = total_valid_references + total_invalid_references;
        let overall_integrity_score = if total_refs > 0 {
            total_valid_references as f64 / total_refs as f64
        } else {
            1.0
        };

        let passes = overall_integrity_score >= self.min_integrity_score;

        Ok(ReferentialIntegrityEvaluation {
            vendor_integrity,
            customer_integrity,
            material_integrity,
            employee_integrity,
            account_integrity,
            cost_center_integrity,
            overall_integrity_score,
            total_valid_references,
            total_invalid_references,
            total_orphaned_entities,
            passes,
        })
    }

    /// Evaluate a single entity type.
    fn evaluate_entity(&self, entity_type: &str, data: &EntityReferenceData) -> EntityIntegrity {
        let total_entities = data.valid_ids.len();

        // Count valid and invalid references
        let mut valid_references = 0;
        let mut invalid_references = 0;
        let mut referenced_ids = std::collections::HashSet::new();

        for reference in &data.references {
            if data.valid_ids.contains(reference) {
                valid_references += 1;
                referenced_ids.insert(reference.clone());
            } else {
                invalid_references += 1;
            }
        }

        let entities_referenced = referenced_ids.len();
        let orphaned_entities = total_entities.saturating_sub(entities_referenced);

        let total_refs = valid_references + invalid_references;
        let integrity_score = if total_refs > 0 {
            valid_references as f64 / total_refs as f64
        } else {
            1.0
        };

        let usage_rate = if total_entities > 0 {
            entities_referenced as f64 / total_entities as f64
        } else {
            1.0
        };

        EntityIntegrity {
            entity_type: entity_type.to_string(),
            total_entities,
            entities_referenced,
            valid_references,
            invalid_references,
            orphaned_entities,
            integrity_score,
            usage_rate,
        }
    }
}

impl Default for ReferentialIntegrityEvaluator {
    fn default() -> Self {
        Self::new(0.99, 0.80) // 99% integrity, 80% usage
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_integrity() {
        let mut data = ReferentialData::default();

        // Add vendors
        data.vendors.add_entity("V001".to_string());
        data.vendors.add_entity("V002".to_string());

        // Add references to valid vendors
        data.vendors.add_reference("V001".to_string());
        data.vendors.add_reference("V002".to_string());
        data.vendors.add_reference("V001".to_string());

        let evaluator = ReferentialIntegrityEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.vendor_integrity.integrity_score, 1.0);
        assert_eq!(result.vendor_integrity.valid_references, 3);
        assert_eq!(result.vendor_integrity.invalid_references, 0);
        assert_eq!(result.vendor_integrity.orphaned_entities, 0);
    }

    #[test]
    fn test_invalid_references() {
        let mut data = ReferentialData::default();

        data.vendors.add_entity("V001".to_string());

        // Reference both valid and invalid
        data.vendors.add_reference("V001".to_string());
        data.vendors.add_reference("V999".to_string()); // Invalid

        let evaluator = ReferentialIntegrityEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.vendor_integrity.valid_references, 1);
        assert_eq!(result.vendor_integrity.invalid_references, 1);
        assert_eq!(result.vendor_integrity.integrity_score, 0.5);
    }

    #[test]
    fn test_orphaned_entities() {
        let mut data = ReferentialData::default();

        // Add vendors but only reference one
        data.vendors.add_entity("V001".to_string());
        data.vendors.add_entity("V002".to_string());
        data.vendors.add_entity("V003".to_string());

        data.vendors.add_reference("V001".to_string());

        let evaluator = ReferentialIntegrityEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.vendor_integrity.entities_referenced, 1);
        assert_eq!(result.vendor_integrity.orphaned_entities, 2);
        assert!(result.vendor_integrity.usage_rate < 0.5);
    }

    #[test]
    fn test_empty_data() {
        let data = ReferentialData::default();
        let evaluator = ReferentialIntegrityEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.overall_integrity_score, 1.0);
        assert!(result.passes);
    }
}
