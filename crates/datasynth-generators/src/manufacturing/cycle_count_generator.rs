//! Cycle count generator for warehouse inventory management.
//!
//! Generates realistic cycle counts with variance distributions matching
//! typical warehouse accuracy patterns: most items match, a few have minor
//! variances, and rare items show major or critical discrepancies.

use chrono::NaiveDate;
use datasynth_core::models::{CountVarianceType, CycleCount, CycleCountItem, CycleCountStatus};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Adjustment reason strings for items that are adjusted.
const ADJUSTMENT_REASONS: &[&str] = &[
    "Physical recount confirmed",
    "Damaged goods written off",
    "Misplaced inventory located",
    "Unit of measure correction",
    "System sync error resolved",
    "Picking discrepancy",
    "Receiving error",
    "Scrap not recorded",
];

/// Generates [`CycleCount`] instances with realistic variance distributions
/// and adjustment patterns.
pub struct CycleCountGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    employee_ids_pool: Vec<String>,
    /// Mapping of material_id → description for denormalization (DS-011).
    material_descriptions: HashMap<String, String>,
}

impl CycleCountGenerator {
    /// Create a new cycle count generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::CycleCount),
            employee_ids_pool: Vec::new(),
            material_descriptions: HashMap::new(),
        }
    }

    /// Set the employee ID pool used for counter and supervisor IDs.
    ///
    /// When non-empty, `counter_id` and `supervisor_id` are picked from this
    /// pool instead of fabricated `WH-{:02}` / `SUP-{:02}` strings.
    pub fn with_employee_pool(mut self, employee_ids: Vec<String>) -> Self {
        self.employee_ids_pool = employee_ids;
        self
    }

    /// Set the material description mapping for denormalization (DS-011).
    ///
    /// Maps material IDs to their descriptions so that generated cycle count
    /// items include the material description for graph export convenience.
    pub fn with_material_descriptions(mut self, descriptions: HashMap<String, String>) -> Self {
        self.material_descriptions = descriptions;
        self
    }

    /// Generate a single cycle count event covering the specified materials.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company code for the cycle count.
    /// * `material_ids` - Available materials as `(material_id, storage_location)` tuples.
    /// * `count_date` - Date the count is performed.
    /// * `items_per_count` - Number of items to include in this count.
    pub fn generate(
        &mut self,
        company_code: &str,
        material_ids: &[(String, String)],
        count_date: NaiveDate,
        items_per_count: usize,
    ) -> CycleCount {
        let count_id = self.uuid_factory.next().to_string();

        // Select items: pick `items_per_count` random materials (or all if fewer)
        let selected_count = items_per_count.min(material_ids.len());
        let mut indices: Vec<usize> = (0..material_ids.len()).collect();
        indices.shuffle(&mut self.rng);
        let selected_indices = &indices[..selected_count];

        // Generate items
        let items: Vec<CycleCountItem> = selected_indices
            .iter()
            .map(|&idx| {
                let (material_id, storage_location) = &material_ids[idx];
                self.generate_item(material_id, storage_location)
            })
            .collect();

        // Compute totals
        let total_items_counted = items.len() as u32;
        let total_variances = items
            .iter()
            .filter(|item| !matches!(item.variance_type, CountVarianceType::None))
            .count() as u32;
        let variance_rate = if total_items_counted > 0 {
            total_variances as f64 / total_items_counted as f64
        } else {
            0.0
        };

        // Status: 40% Reconciled, 30% Closed, 20% Counted, 10% InProgress
        let status = self.pick_status();

        // Counter and supervisor – use real employee IDs when available
        let counter_id = if self.employee_ids_pool.is_empty() {
            Some(format!("WH-{:02}", self.rng.random_range(1..=10)))
        } else {
            self.employee_ids_pool.choose(&mut self.rng).cloned()
        };
        let supervisor_id = if self.employee_ids_pool.is_empty() {
            Some(format!("SUP-{:02}", self.rng.random_range(1..=5)))
        } else {
            self.employee_ids_pool.choose(&mut self.rng).cloned()
        };

        // Warehouse ID
        let warehouse_id = format!("WH-{:03}", self.rng.random_range(1..=10));

        CycleCount {
            count_id,
            company_code: company_code.to_string(),
            warehouse_id,
            count_date,
            status,
            counter_id,
            supervisor_id,
            items,
            total_items_counted,
            total_variances,
            variance_rate,
        }
    }

    /// Generate a single cycle count item with realistic variance patterns.
    fn generate_item(&mut self, material_id: &str, storage_location: &str) -> CycleCountItem {
        // Book quantity: random 100 - 10,000
        let book_qty_f64: f64 = self.rng.random_range(100.0..=10_000.0);
        let book_quantity =
            Decimal::from_f64_retain(book_qty_f64.round()).unwrap_or(Decimal::from(100));

        // Unit cost: random 5.0 - 500.0
        let unit_cost_f64: f64 = self.rng.random_range(5.0..=500.0);
        let unit_cost = Decimal::from_f64_retain(unit_cost_f64)
            .unwrap_or(Decimal::from(10))
            .round_dp(2);

        // Determine variance type and counted quantity
        // 85% match, 10% minor (±1-3%), 4% major (±5-15%), 1% critical (±20-50%)
        let roll: f64 = self.rng.random();
        let (variance_type, counted_quantity) = if roll < 0.85 {
            // Exact match
            (CountVarianceType::None, book_quantity)
        } else if roll < 0.95 {
            // Minor variance: ±1-3%
            let pct: f64 = self.rng.random_range(0.01..=0.03);
            let sign = if self.rng.random_bool(0.5) { 1.0 } else { -1.0 };
            let counted_f64 = book_qty_f64 * (1.0 + sign * pct);
            let counted = Decimal::from_f64_retain(counted_f64.round()).unwrap_or(book_quantity);
            (CountVarianceType::Minor, counted)
        } else if roll < 0.99 {
            // Major variance: ±5-15%
            let pct: f64 = self.rng.random_range(0.05..=0.15);
            let sign = if self.rng.random_bool(0.5) { 1.0 } else { -1.0 };
            let counted_f64 = book_qty_f64 * (1.0 + sign * pct);
            let counted =
                Decimal::from_f64_retain(counted_f64.round().max(0.0)).unwrap_or(book_quantity);
            (CountVarianceType::Major, counted)
        } else {
            // Critical variance: ±20-50%
            let pct: f64 = self.rng.random_range(0.20..=0.50);
            let sign = if self.rng.random_bool(0.5) { 1.0 } else { -1.0 };
            let counted_f64 = book_qty_f64 * (1.0 + sign * pct);
            let counted =
                Decimal::from_f64_retain(counted_f64.round().max(0.0)).unwrap_or(book_quantity);
            (CountVarianceType::Critical, counted)
        };

        let variance_quantity = counted_quantity - book_quantity;
        let variance_value = (variance_quantity * unit_cost).round_dp(2);

        // Adjusted: 80% of items with variance get adjusted
        let has_variance = !matches!(variance_type, CountVarianceType::None);
        let adjusted = has_variance && self.rng.random_bool(0.80);

        let adjustment_reason = if adjusted {
            ADJUSTMENT_REASONS
                .choose(&mut self.rng)
                .map(|s| s.to_string())
        } else {
            None
        };

        CycleCountItem {
            material_id: material_id.to_string(),
            material_description: self.material_descriptions.get(material_id).cloned(),
            storage_location: storage_location.to_string(),
            book_quantity,
            counted_quantity,
            variance_quantity,
            unit_cost,
            variance_value,
            variance_type,
            adjusted,
            adjustment_reason,
        }
    }

    /// Pick a cycle count status based on distribution.
    fn pick_status(&mut self) -> CycleCountStatus {
        let roll: f64 = self.rng.random();
        if roll < 0.40 {
            CycleCountStatus::Reconciled
        } else if roll < 0.70 {
            CycleCountStatus::Closed
        } else if roll < 0.90 {
            CycleCountStatus::Counted
        } else {
            CycleCountStatus::InProgress
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

    fn sample_materials() -> Vec<(String, String)> {
        vec![
            ("MAT-001".to_string(), "SL-A01".to_string()),
            ("MAT-002".to_string(), "SL-A02".to_string()),
            ("MAT-003".to_string(), "SL-B01".to_string()),
            ("MAT-004".to_string(), "SL-B02".to_string()),
            ("MAT-005".to_string(), "SL-C01".to_string()),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = CycleCountGenerator::new(42);
        let materials = sample_materials();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let count = gen.generate("C001", &materials, date, 5);

        assert_eq!(count.company_code, "C001");
        assert_eq!(count.count_date, date);
        assert!(!count.count_id.is_empty());
        assert_eq!(count.items.len(), 5);
        assert_eq!(count.total_items_counted, 5);
        assert!(count.counter_id.is_some());
        assert!(count.supervisor_id.is_some());

        for item in &count.items {
            assert!(item.book_quantity > Decimal::ZERO);
            assert!(item.counted_quantity >= Decimal::ZERO);
            assert!(item.unit_cost > Decimal::ZERO);
            // Variance value should equal variance_quantity * unit_cost (within rounding)
            let expected_variance = (item.variance_quantity * item.unit_cost).round_dp(2);
            assert_eq!(item.variance_value, expected_variance);
        }
    }

    #[test]
    fn test_deterministic() {
        let materials = sample_materials();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let mut gen1 = CycleCountGenerator::new(12345);
        let count1 = gen1.generate("C001", &materials, date, 5);

        let mut gen2 = CycleCountGenerator::new(12345);
        let count2 = gen2.generate("C001", &materials, date, 5);

        assert_eq!(count1.count_id, count2.count_id);
        assert_eq!(count1.items.len(), count2.items.len());
        assert_eq!(count1.total_variances, count2.total_variances);
        for (i1, i2) in count1.items.iter().zip(count2.items.iter()) {
            assert_eq!(i1.material_id, i2.material_id);
            assert_eq!(i1.book_quantity, i2.book_quantity);
            assert_eq!(i1.counted_quantity, i2.counted_quantity);
            assert_eq!(i1.variance_value, i2.variance_value);
        }
    }

    #[test]
    fn test_variance_distribution() {
        let mut gen = CycleCountGenerator::new(77);
        let materials: Vec<(String, String)> = (0..100)
            .map(|i| (format!("MAT-{:03}", i), format!("SL-{:03}", i)))
            .collect();
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

        let count = gen.generate("C001", &materials, date, 100);

        let none_count = count
            .items
            .iter()
            .filter(|i| matches!(i.variance_type, CountVarianceType::None))
            .count();
        let minor_count = count
            .items
            .iter()
            .filter(|i| matches!(i.variance_type, CountVarianceType::Minor))
            .count();

        // With 100 items and 85% match rate, expect ~80-95 exact matches
        assert!(
            none_count >= 70 && none_count <= 98,
            "Expected ~85% exact matches, got {}/100",
            none_count,
        );
        // Minor should be present but not dominant
        assert!(minor_count > 0, "Expected at least some minor variances");
    }

    #[test]
    fn test_items_per_count_cap() {
        let mut gen = CycleCountGenerator::new(55);
        let materials = sample_materials(); // 5 materials
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Request more items than available materials
        let count = gen.generate("C001", &materials, date, 20);

        assert_eq!(
            count.items.len(),
            5,
            "Items should be capped at available material count"
        );
    }
}
