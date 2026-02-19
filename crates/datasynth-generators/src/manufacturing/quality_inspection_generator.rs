//! Quality inspection generator for manufacturing processes.
//!
//! Generates realistic quality inspections linked to production orders,
//! with multiple inspection characteristics, defect tracking, and
//! disposition assignment.

use chrono::NaiveDate;
use datasynth_core::models::{
    InspectionCharacteristic, InspectionResult, InspectionType, QualityInspection,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Characteristic names used in quality inspections.
const CHARACTERISTIC_NAMES: &[&str] = &[
    "Dimension A",
    "Weight",
    "Surface Finish",
    "Tensile Strength",
    "Hardness",
    "Thickness",
    "Diameter",
    "Flatness",
    "Concentricity",
    "Color Consistency",
];

/// Disposition actions for inspection results.
const DISPOSITIONS_ACCEPTED: &[&str] = &["use_as_is", "stock"];
const DISPOSITIONS_CONDITIONAL: &[&str] = &["use_as_is", "rework", "downgrade"];
const DISPOSITIONS_REJECTED: &[&str] = &["return_to_vendor", "scrap", "rework"];

/// Generates [`QualityInspection`] instances linked to production orders
/// with realistic inspection characteristics and defect rates.
pub struct QualityInspectionGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl QualityInspectionGenerator {
    /// Create a new quality inspection generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::QualityInspection),
        }
    }

    /// Generate quality inspections for a set of production orders.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company code for all generated inspections.
    /// * `production_orders` - Tuples of `(order_id, material_id, material_description)`.
    /// * `inspection_date` - Date for all generated inspections.
    pub fn generate(
        &mut self,
        company_code: &str,
        production_orders: &[(String, String, String)],
        inspection_date: NaiveDate,
    ) -> Vec<QualityInspection> {
        debug!(company_code, order_count = production_orders.len(), %inspection_date, "Generating quality inspections");
        production_orders
            .iter()
            .map(|(order_id, material_id, material_desc)| {
                self.generate_one(
                    company_code,
                    order_id,
                    material_id,
                    material_desc,
                    inspection_date,
                )
            })
            .collect()
    }

    /// Generate a single quality inspection for a production order.
    fn generate_one(
        &mut self,
        company_code: &str,
        order_id: &str,
        material_id: &str,
        material_description: &str,
        inspection_date: NaiveDate,
    ) -> QualityInspection {
        let inspection_id = self.uuid_factory.next().to_string();

        // Inspection type distribution: 40% Final, 25% InProcess, 20% Incoming, 10% Random, 5% Periodic
        let inspection_type = self.pick_inspection_type();

        // Lot size based on typical production order quantity
        let lot_size_f64: f64 = self.rng.gen_range(50.0..=1000.0);
        let lot_size = Decimal::from_f64_retain(lot_size_f64.round()).unwrap_or(Decimal::from(100));

        // Sample size: 10-30% of lot
        let sample_pct: f64 = self.rng.gen_range(0.10..=0.30);
        let sample_size_f64 = (lot_size_f64 * sample_pct).round().max(1.0);
        let sample_size = Decimal::from_f64_retain(sample_size_f64).unwrap_or(Decimal::from(10));

        // Generate 2-5 inspection characteristics
        let num_characteristics: usize = self.rng.gen_range(2..=5);
        let characteristics = self.generate_characteristics(num_characteristics);

        // Count defects (failed characteristics)
        let defect_count = characteristics.iter().filter(|c| !c.passed).count() as u32;
        let defect_rate = if sample_size_f64 > 0.0 {
            defect_count as f64 / sample_size_f64
        } else {
            0.0
        };

        // Inspection result: 80% Accepted, 10% Conditionally, 7% Rejected, 3% Pending
        let result = self.pick_result();

        // Inspector
        let inspector_id = Some(format!("QC-{:02}", self.rng.gen_range(1..=20)));

        // Disposition based on result
        let disposition = match result {
            InspectionResult::Accepted => DISPOSITIONS_ACCEPTED
                .choose(&mut self.rng)
                .map(|s| s.to_string()),
            InspectionResult::Conditionally => DISPOSITIONS_CONDITIONAL
                .choose(&mut self.rng)
                .map(|s| s.to_string()),
            InspectionResult::Rejected => DISPOSITIONS_REJECTED
                .choose(&mut self.rng)
                .map(|s| s.to_string()),
            InspectionResult::Pending => None,
        };

        // Notes for non-accepted results
        let notes = match result {
            InspectionResult::Rejected => Some(format!(
                "{} defects found in {} characteristics. Material held for disposition.",
                defect_count, num_characteristics
            )),
            InspectionResult::Conditionally => Some(format!(
                "Minor deviations noted. {} characteristic(s) marginally out of spec.",
                defect_count
            )),
            _ => None,
        };

        QualityInspection {
            inspection_id,
            company_code: company_code.to_string(),
            reference_type: "production_order".to_string(),
            reference_id: order_id.to_string(),
            material_id: material_id.to_string(),
            material_description: material_description.to_string(),
            inspection_type,
            inspection_date,
            inspector_id,
            lot_size,
            sample_size,
            defect_count,
            defect_rate,
            result,
            characteristics,
            disposition,
            notes,
        }
    }

    /// Pick an inspection type based on distribution.
    fn pick_inspection_type(&mut self) -> InspectionType {
        let roll: f64 = self.rng.gen();
        if roll < 0.40 {
            InspectionType::Final
        } else if roll < 0.65 {
            InspectionType::InProcess
        } else if roll < 0.85 {
            InspectionType::Incoming
        } else if roll < 0.95 {
            InspectionType::Random
        } else {
            InspectionType::Periodic
        }
    }

    /// Pick an inspection result based on distribution.
    fn pick_result(&mut self) -> InspectionResult {
        let roll: f64 = self.rng.gen();
        if roll < 0.80 {
            InspectionResult::Accepted
        } else if roll < 0.90 {
            InspectionResult::Conditionally
        } else if roll < 0.97 {
            InspectionResult::Rejected
        } else {
            InspectionResult::Pending
        }
    }

    /// Generate inspection characteristics with target/actual values and limits.
    fn generate_characteristics(&mut self, count: usize) -> Vec<InspectionCharacteristic> {
        // Shuffle and pick `count` characteristic names
        let mut indices: Vec<usize> = (0..CHARACTERISTIC_NAMES.len()).collect();
        indices.shuffle(&mut self.rng);
        let selected_count = count.min(indices.len());

        indices[..selected_count]
            .iter()
            .map(|&idx| {
                let name = CHARACTERISTIC_NAMES[idx].to_string();

                // Target value: random 10.0 - 100.0
                let target_value: f64 = self.rng.gen_range(10.0..=100.0);

                // Limits: ± 5-15% of target
                let tolerance_pct: f64 = self.rng.gen_range(0.05..=0.15);
                let lower_limit = target_value * (1.0 - tolerance_pct);
                let upper_limit = target_value * (1.0 + tolerance_pct);

                // Actual value: target * random(0.95 - 1.05)
                let actual_factor: f64 = self.rng.gen_range(0.95..=1.05);
                let actual_value = target_value * actual_factor;

                let passed = actual_value >= lower_limit && actual_value <= upper_limit;

                InspectionCharacteristic {
                    name,
                    target_value,
                    actual_value,
                    lower_limit,
                    upper_limit,
                    passed,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_orders() -> Vec<(String, String, String)> {
        vec![
            (
                "PO-001".to_string(),
                "MAT-001".to_string(),
                "Widget Alpha".to_string(),
            ),
            (
                "PO-002".to_string(),
                "MAT-002".to_string(),
                "Widget Beta".to_string(),
            ),
            (
                "PO-003".to_string(),
                "MAT-003".to_string(),
                "Widget Gamma".to_string(),
            ),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = QualityInspectionGenerator::new(42);
        let orders = sample_orders();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let inspections = gen.generate("C001", &orders, date);

        assert_eq!(inspections.len(), orders.len());
        for insp in &inspections {
            assert_eq!(insp.company_code, "C001");
            assert_eq!(insp.inspection_date, date);
            assert!(!insp.inspection_id.is_empty());
            assert_eq!(insp.reference_type, "production_order");
            assert!(insp.lot_size > Decimal::ZERO);
            assert!(insp.sample_size > Decimal::ZERO);
            assert!(insp.sample_size <= insp.lot_size);
            assert!(!insp.characteristics.is_empty());
            assert!(insp.characteristics.len() >= 2 && insp.characteristics.len() <= 5);
            assert!(insp.inspector_id.is_some());
        }
    }

    #[test]
    fn test_deterministic() {
        let orders = sample_orders();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let mut gen1 = QualityInspectionGenerator::new(12345);
        let insp1 = gen1.generate("C001", &orders, date);

        let mut gen2 = QualityInspectionGenerator::new(12345);
        let insp2 = gen2.generate("C001", &orders, date);

        assert_eq!(insp1.len(), insp2.len());
        for (i1, i2) in insp1.iter().zip(insp2.iter()) {
            assert_eq!(i1.inspection_id, i2.inspection_id);
            assert_eq!(i1.lot_size, i2.lot_size);
            assert_eq!(i1.sample_size, i2.sample_size);
            assert_eq!(i1.defect_count, i2.defect_count);
            assert_eq!(i1.characteristics.len(), i2.characteristics.len());
        }
    }

    #[test]
    fn test_characteristics_limits() {
        let mut gen = QualityInspectionGenerator::new(99);
        let orders = sample_orders();
        let date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let inspections = gen.generate("C001", &orders, date);

        for insp in &inspections {
            for char in &insp.characteristics {
                // Lower limit must be below target
                assert!(
                    char.lower_limit < char.target_value,
                    "Lower limit {} should be below target {}",
                    char.lower_limit,
                    char.target_value,
                );
                // Upper limit must be above target
                assert!(
                    char.upper_limit > char.target_value,
                    "Upper limit {} should be above target {}",
                    char.upper_limit,
                    char.target_value,
                );
                // Passed should be consistent with limits
                let within_limits =
                    char.actual_value >= char.lower_limit && char.actual_value <= char.upper_limit;
                assert_eq!(
                    char.passed, within_limits,
                    "Passed flag ({}) inconsistent with limits for {}",
                    char.passed, char.name,
                );
            }
        }
    }
}
