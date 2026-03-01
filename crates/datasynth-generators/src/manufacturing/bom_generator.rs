//! Bill of Materials (BOM) generator for manufacturing processes.
//!
//! Generates multi-level BOM structures for finished and semi-finished
//! materials, creating parent-child component relationships with realistic
//! quantities, scrap rates, and phantom assembly flags.

use datasynth_core::models::BomComponent;
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Component descriptions for generated BOM items.
const COMPONENT_DESCRIPTIONS: &[&str] = &[
    "Steel plate",
    "Aluminum extrusion",
    "Bearing assembly",
    "Electronic module",
    "Fastener set",
    "Gasket kit",
    "Wire harness",
    "Plastic housing",
    "Rubber seal",
    "Circuit board",
    "Motor unit",
    "Sensor module",
    "Filter element",
    "Bracket assembly",
    "Spring set",
];

/// Generates [`BomComponent`] records linking parent materials to their components.
pub struct BomGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BomGenerator {
    /// Create a new BOM generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BomComponent),
        }
    }

    /// Generate BOM components for the given materials.
    ///
    /// For each finished or semi-finished material (those with an even index
    /// in the provided list, as a simple heuristic), generates 2-8 BOM
    /// components drawn from the remaining materials.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company / entity code.
    /// * `material_ids` - Available materials as `(material_id, description)` tuples.
    pub fn generate(
        &mut self,
        company_code: &str,
        material_ids: &[(String, String)],
    ) -> Vec<BomComponent> {
        debug!(
            company_code,
            material_count = material_ids.len(),
            "Generating BOM components"
        );
        if material_ids.len() < 3 {
            return Vec::new();
        }

        let mut components = Vec::new();

        // Treat ~40% of materials as finished goods with BOMs
        let parent_count = (material_ids.len() * 2 / 5).max(1);

        for parent_idx in 0..parent_count {
            let (parent_id, _parent_desc) = &material_ids[parent_idx];
            let comp_count = self.rng.random_range(2..=8).min(material_ids.len() - 1);

            // Select component materials (skip the parent itself)
            let mut candidate_indices: Vec<usize> = (0..material_ids.len())
                .filter(|&i| i != parent_idx)
                .collect();
            candidate_indices.shuffle(&mut self.rng);
            let selected = &candidate_indices[..comp_count.min(candidate_indices.len())];

            for (pos, &comp_idx) in selected.iter().enumerate() {
                let (comp_id, comp_desc) = &material_ids[comp_idx];
                let bom_id = self.uuid_factory.next().to_string();

                let quantity_per = Decimal::from(self.rng.random_range(1..=100));
                let scrap_pct: f64 = self.rng.random_range(0.01..=0.10);
                let scrap_percentage =
                    Decimal::from_f64_retain(scrap_pct).unwrap_or(Decimal::new(2, 2));
                let is_phantom = self.rng.random_bool(0.10);
                let level = if pos < 2 { 1 } else { 2 };

                let mut comp =
                    BomComponent::new(comp_id, quantity_per, "EA").with_scrap(scrap_percentage);
                comp.position = (pos + 1) as u16 * 10;
                comp.id = Some(bom_id);
                comp.entity_code = Some(company_code.to_string());
                comp.parent_material = Some(parent_id.clone());
                comp.component_description = Some(if comp_desc.is_empty() {
                    COMPONENT_DESCRIPTIONS[self.rng.random_range(0..COMPONENT_DESCRIPTIONS.len())]
                        .to_string()
                } else {
                    comp_desc.clone()
                });
                comp.level = Some(level);
                comp.is_phantom = is_phantom;

                components.push(comp);
            }
        }

        components
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_materials() -> Vec<(String, String)> {
        (0..10)
            .map(|i| (format!("MAT-{:03}", i), format!("Material {}", i)))
            .collect()
    }

    #[test]
    fn test_bom_generation() {
        let mut gen = BomGenerator::new(42);
        let materials = test_materials();
        let bom = gen.generate("C001", &materials);

        assert!(!bom.is_empty(), "Should generate BOM components");
        for comp in &bom {
            assert!(comp.quantity > Decimal::ZERO);
            assert!(comp.id.is_some());
            assert!(comp.entity_code.is_some());
            assert!(comp.parent_material.is_some());
            assert!(comp.component_description.is_some());
            assert!(comp.level.is_some());
            assert!(comp.position > 0);
        }
    }

    #[test]
    fn test_bom_has_phantoms() {
        let mut gen = BomGenerator::new(77);
        let materials: Vec<(String, String)> = (0..30)
            .map(|i| (format!("MAT-{:03}", i), format!("Material {}", i)))
            .collect();
        let bom = gen.generate("C001", &materials);

        // With 30 materials and ~10% phantom rate, expect at least one
        let phantom_count = bom.iter().filter(|c| c.is_phantom).count();
        assert!(
            phantom_count > 0 || bom.len() < 10,
            "Expected some phantom assemblies in a large BOM set"
        );
    }

    #[test]
    fn test_bom_deterministic() {
        let materials = test_materials();
        let mut gen1 = BomGenerator::new(12345);
        let bom1 = gen1.generate("C001", &materials);
        let mut gen2 = BomGenerator::new(12345);
        let bom2 = gen2.generate("C001", &materials);

        assert_eq!(bom1.len(), bom2.len());
        for (a, b) in bom1.iter().zip(bom2.iter()) {
            assert_eq!(a.component_material_id, b.component_material_id);
            assert_eq!(a.quantity, b.quantity);
        }
    }

    #[test]
    fn test_bom_too_few_materials() {
        let mut gen = BomGenerator::new(42);
        let materials = vec![("MAT-001".to_string(), "M1".to_string())];
        let bom = gen.generate("C001", &materials);
        assert!(bom.is_empty(), "Should return empty for < 3 materials");
    }
}
