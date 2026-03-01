//! Inventory movement generator for manufacturing processes.
//!
//! Generates stock movements (goods receipts, goods issues, transfers,
//! returns, scrap, and adjustments) tied to production orders and purchase
//! orders for realistic warehouse flow simulation.

use chrono::NaiveDate;
use datasynth_core::models::{InventoryMovement, MovementType};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Storage locations for movement generation.
const STORAGE_LOCATIONS: &[&str] = &[
    "WH01-A01", "WH01-A02", "WH01-B01", "WH02-A01", "WH02-B01", "WH03-A01",
];

/// Generates [`InventoryMovement`] records for warehouse stock flow.
pub struct InventoryMovementGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl InventoryMovementGenerator {
    /// Create a new inventory movement generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::InventoryMovement),
        }
    }

    /// Generate inventory movements for the given period.
    ///
    /// Creates a mix of movement types distributed across the period for each
    /// material in the pool.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company / entity code.
    /// * `material_ids` - Available materials as `(material_id, description)` tuples.
    /// * `period_start` - Start of the generation period.
    /// * `period_end` - End of the generation period.
    /// * `movements_per_material` - Average number of movements per material.
    /// * `currency` - Currency code for value calculations.
    pub fn generate(
        &mut self,
        company_code: &str,
        material_ids: &[(String, String)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        movements_per_material: u32,
        currency: &str,
    ) -> Vec<InventoryMovement> {
        debug!(
            company_code,
            material_count = material_ids.len(),
            %period_start,
            %period_end,
            movements_per_material,
            "Generating inventory movements"
        );

        let mut movements = Vec::new();
        let period_days = (period_end - period_start).num_days().max(1) as u64;
        let period_str = format!(
            "{}-{:02}",
            period_start.format("%Y"),
            period_start.format("%m")
        );

        for (material_id, material_desc) in material_ids {
            let count = self.rng.random_range(1..=movements_per_material.max(1) * 2);

            for _ in 0..count {
                let mv_id = self.uuid_factory.next().to_string();
                let day_offset = self.rng.random_range(0..period_days);
                let movement_date = period_start + chrono::Duration::days(day_offset as i64);

                let movement_type = self.pick_movement_type();
                let quantity = Decimal::from(self.rng.random_range(1..=500));
                let unit_cost: f64 = self.rng.random_range(5.0..=200.0);
                let value = (quantity
                    * Decimal::from_f64_retain(unit_cost).unwrap_or(Decimal::from(10)))
                .round_dp(2);

                let storage_location = STORAGE_LOCATIONS
                    [self.rng.random_range(0..STORAGE_LOCATIONS.len())]
                .to_string();

                let reference_doc = match movement_type {
                    MovementType::GoodsReceipt => {
                        format!("PO-{:08}", self.rng.random_range(10000..99999))
                    }
                    MovementType::GoodsIssue => {
                        format!("PRD-{:08}", self.rng.random_range(10000..99999))
                    }
                    MovementType::Transfer => {
                        format!("TR-{:08}", self.rng.random_range(10000..99999))
                    }
                    MovementType::Return => {
                        format!("RET-{:08}", self.rng.random_range(10000..99999))
                    }
                    MovementType::Scrap => {
                        format!("QI-{:08}", self.rng.random_range(10000..99999))
                    }
                    MovementType::Adjustment => {
                        format!("CC-{:08}", self.rng.random_range(10000..99999))
                    }
                };

                let mv = InventoryMovement::new(
                    mv_id,
                    company_code,
                    material_id,
                    material_desc,
                    movement_date,
                    &period_str,
                    movement_type,
                    quantity,
                    "EA",
                    value,
                    currency,
                    &storage_location,
                    &reference_doc,
                );
                movements.push(mv);
            }
        }

        movements
    }

    /// Pick a movement type based on realistic distribution.
    ///
    /// 35% GoodsReceipt, 30% GoodsIssue, 15% Transfer, 8% Return, 7% Scrap, 5% Adjustment
    fn pick_movement_type(&mut self) -> MovementType {
        let roll: f64 = self.rng.random();
        if roll < 0.35 {
            MovementType::GoodsReceipt
        } else if roll < 0.65 {
            MovementType::GoodsIssue
        } else if roll < 0.80 {
            MovementType::Transfer
        } else if roll < 0.88 {
            MovementType::Return
        } else if roll < 0.95 {
            MovementType::Scrap
        } else {
            MovementType::Adjustment
        }
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
        vec![
            ("MAT-001".to_string(), "Widget A".to_string()),
            ("MAT-002".to_string(), "Widget B".to_string()),
            ("MAT-003".to_string(), "Widget C".to_string()),
        ]
    }

    #[test]
    fn test_movement_generation() {
        let mut gen = InventoryMovementGenerator::new(42);
        let materials = test_materials();
        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let movements = gen.generate("C001", &materials, start, end, 5, "USD");

        assert!(!movements.is_empty());
        for mv in &movements {
            assert_eq!(mv.entity_code, "C001");
            assert_eq!(mv.currency, "USD");
            assert!(mv.quantity > Decimal::ZERO);
            assert!(mv.value > Decimal::ZERO);
            assert!(mv.movement_date >= start);
            assert!(mv.movement_date <= end);
            assert!(!mv.reference_doc.is_empty());
        }
    }

    #[test]
    fn test_movement_type_distribution() {
        let mut gen = InventoryMovementGenerator::new(77);
        let materials: Vec<(String, String)> = (0..20)
            .map(|i| (format!("MAT-{:03}", i), format!("M-{}", i)))
            .collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let movements = gen.generate("C001", &materials, start, end, 10, "USD");

        let receipts = movements
            .iter()
            .filter(|m| matches!(m.movement_type, MovementType::GoodsReceipt))
            .count();
        let issues = movements
            .iter()
            .filter(|m| matches!(m.movement_type, MovementType::GoodsIssue))
            .count();

        assert!(receipts > 0, "Should have goods receipts");
        assert!(issues > 0, "Should have goods issues");
    }

    #[test]
    fn test_movement_deterministic() {
        let materials = test_materials();
        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let mut gen1 = InventoryMovementGenerator::new(12345);
        let mv1 = gen1.generate("C001", &materials, start, end, 3, "USD");
        let mut gen2 = InventoryMovementGenerator::new(12345);
        let mv2 = gen2.generate("C001", &materials, start, end, 3, "USD");

        assert_eq!(mv1.len(), mv2.len());
        for (a, b) in mv1.iter().zip(mv2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.material_code, b.material_code);
            assert_eq!(a.quantity, b.quantity);
            assert_eq!(a.value, b.value);
        }
    }
}
