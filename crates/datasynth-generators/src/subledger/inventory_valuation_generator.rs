//! Inventory Valuation Generator.
//!
//! Produces `InventoryValuationReport` records by applying IAS 2 / ASC 330
//! lower-of-cost-or-NRV logic to the set of `InventoryPosition` entries in
//! the subledger snapshot.
//!
//! ## NRV estimation
//!
//! Because the synthetic dataset does not separately track market prices, the
//! generator models Net Realisable Value (NRV) as:
//!
//! ```text
//! nrv_per_unit = unit_cost * nrv_factor
//! ```
//!
//! where `nrv_factor` is sampled around the configured `avg_nrv_factor` with
//! `nrv_factor_variation` applied symmetrically.  A factor < 1.0 means the
//! position is impaired and triggers a write-down amount:
//!
//! ```text
//! write_down = quantity * max(0, cost - nrv_per_unit)
//! ```

use chrono::NaiveDate;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::subledger::inventory::{InventoryPosition, InventoryValuationReport};

/// Configuration for the inventory valuation generator.
#[derive(Debug, Clone)]
pub struct InventoryValuationGeneratorConfig {
    /// Average ratio of NRV to cost (1.0 = no impairment on average).
    /// Values below 1.0 introduce write-downs.
    pub avg_nrv_factor: f64,
    /// Symmetric variation applied to `avg_nrv_factor` per material.
    pub nrv_factor_variation: f64,
    /// Seed offset used to keep the RNG independent of other generators.
    pub seed_offset: u64,
}

impl Default for InventoryValuationGeneratorConfig {
    fn default() -> Self {
        Self {
            avg_nrv_factor: 1.05, // NRV slightly above cost on average
            nrv_factor_variation: 0.15,
            seed_offset: 900,
        }
    }
}

/// Per-material valuation line including NRV and potential write-down.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryValuationLine {
    /// Material ID.
    pub material_id: String,
    /// Material description.
    pub description: String,
    /// Plant.
    pub plant: String,
    /// Storage location.
    pub storage_location: String,
    /// Quantity on hand.
    pub quantity: Decimal,
    /// Unit of measure.
    pub unit: String,
    /// Cost per unit (from subledger position).
    pub cost_per_unit: Decimal,
    /// Total cost value.
    pub total_cost: Decimal,
    /// Estimated NRV per unit.
    pub nrv_per_unit: Decimal,
    /// Total NRV.
    pub total_nrv: Decimal,
    /// Write-down required (lower of cost vs NRV, per IAS 2).
    /// Zero when NRV >= cost.
    pub write_down_amount: Decimal,
    /// Carrying value after write-down (min of cost and NRV).
    pub carrying_value: Decimal,
    /// Whether the position is impaired (write_down_amount > 0).
    pub is_impaired: bool,
}

/// Inventory valuation report for a company/period, with write-down analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryValuationResult {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Per-material valuation lines.
    pub lines: Vec<InventoryValuationLine>,
    /// Total cost of all positions.
    pub total_cost: Decimal,
    /// Total NRV of all positions.
    pub total_nrv: Decimal,
    /// Total write-down required.
    pub total_write_down: Decimal,
    /// Carrying value after write-downs.
    pub total_carrying_value: Decimal,
    /// Count of impaired positions.
    pub impaired_count: u32,
    /// Underlying valuation report (sorted by value, with ABC analysis).
    pub valuation_report: InventoryValuationReport,
}

/// Generator that applies lower-of-cost-or-NRV valuation to inventory positions.
pub struct InventoryValuationGenerator {
    config: InventoryValuationGeneratorConfig,
    seed: u64,
}

impl InventoryValuationGenerator {
    /// Creates a new generator with the given base seed.
    pub fn new(config: InventoryValuationGeneratorConfig, seed: u64) -> Self {
        Self { config, seed }
    }

    /// Generates an `InventoryValuationResult` for a company as of a date.
    ///
    /// * `company_code` — the company to value (filters positions by `company_code`).
    /// * `positions` — full slice of inventory positions (may contain multiple companies).
    /// * `as_of_date` — valuation date.
    pub fn generate(
        &self,
        company_code: &str,
        positions: &[InventoryPosition],
        as_of_date: NaiveDate,
    ) -> InventoryValuationResult {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed + self.config.seed_offset);

        let company_positions: Vec<&InventoryPosition> = positions
            .iter()
            .filter(|p| p.company_code == company_code)
            .collect();

        let mut lines = Vec::with_capacity(company_positions.len());
        let mut total_cost = Decimal::ZERO;
        let mut total_nrv = Decimal::ZERO;
        let mut total_write_down = Decimal::ZERO;
        let mut impaired_count = 0u32;

        for pos in &company_positions {
            let cost_per_unit = pos.valuation.unit_cost;
            let quantity = pos.quantity_on_hand;
            let total_cost_pos = (quantity * cost_per_unit).round_dp(2);

            // Sample NRV factor for this position.
            let variation: f64 = rng
                .random_range(-self.config.nrv_factor_variation..=self.config.nrv_factor_variation);
            let nrv_factor = (self.config.avg_nrv_factor + variation).max(0.0);
            let nrv_factor_dec = Decimal::try_from(nrv_factor).unwrap_or(dec!(1));

            let nrv_per_unit = (cost_per_unit * nrv_factor_dec).round_dp(4);
            let total_nrv_pos = (quantity * nrv_per_unit).round_dp(2);

            // IAS 2: carrying value = min(cost, NRV)
            let write_down = (total_cost_pos - total_nrv_pos)
                .max(Decimal::ZERO)
                .round_dp(2);
            let carrying_value = total_cost_pos - write_down;
            let is_impaired = write_down > Decimal::ZERO;

            if is_impaired {
                impaired_count += 1;
            }

            total_cost += total_cost_pos;
            total_nrv += total_nrv_pos;
            total_write_down += write_down;

            lines.push(InventoryValuationLine {
                material_id: pos.material_id.clone(),
                description: pos.description.clone(),
                plant: pos.plant.clone(),
                storage_location: pos.storage_location.clone(),
                quantity,
                unit: pos.unit.clone(),
                cost_per_unit,
                total_cost: total_cost_pos,
                nrv_per_unit,
                total_nrv: total_nrv_pos,
                write_down_amount: write_down,
                carrying_value,
                is_impaired,
            });
        }

        // Sort lines by write-down descending (most impaired first).
        lines.sort_by(|a, b| b.write_down_amount.cmp(&a.write_down_amount));

        let total_carrying_value = total_cost - total_write_down;

        // Build the standard InventoryValuationReport as well.
        let valuation_report = InventoryValuationReport::from_positions(
            company_code.to_string(),
            positions,
            as_of_date,
        );

        InventoryValuationResult {
            company_code: company_code.to_string(),
            as_of_date,
            lines,
            total_cost,
            total_nrv,
            total_write_down,
            total_carrying_value,
            impaired_count,
            valuation_report,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::subledger::inventory::{
        InventoryPosition, PositionValuation, ValuationMethod,
    };
    use rust_decimal_macros::dec;

    fn make_position(
        material_id: &str,
        company: &str,
        qty: Decimal,
        unit_cost: Decimal,
    ) -> InventoryPosition {
        let mut pos = InventoryPosition::new(
            material_id.to_string(),
            format!("Material {material_id}"),
            "PLANT01".to_string(),
            "SL001".to_string(),
            company.to_string(),
            "EA".to_string(),
        );
        pos.quantity_on_hand = qty;
        pos.quantity_available = qty;
        pos.valuation = PositionValuation {
            method: ValuationMethod::StandardCost,
            standard_cost: unit_cost,
            unit_cost,
            total_value: qty * unit_cost,
            price_variance: Decimal::ZERO,
            last_price_change: None,
        };
        pos
    }

    #[test]
    fn test_nrv_write_down_when_cost_exceeds_nrv() {
        // Set avg_nrv_factor = 0.8 so NRV is always below cost → write-down expected.
        let cfg = InventoryValuationGeneratorConfig {
            avg_nrv_factor: 0.8,
            nrv_factor_variation: 0.0, // deterministic
            seed_offset: 0,
        };
        let gen = InventoryValuationGenerator::new(cfg, 42);
        let positions = vec![make_position("MAT001", "1000", dec!(100), dec!(10))];
        let result = gen.generate(
            "1000",
            &positions,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        // cost = 100 * 10 = 1000, nrv = 1000 * 0.8 = 800, write-down = 200
        assert_eq!(result.lines.len(), 1);
        assert!(result.lines[0].is_impaired, "Position should be impaired");
        assert_eq!(result.lines[0].write_down_amount, dec!(200));
        assert_eq!(result.total_write_down, dec!(200));
        assert_eq!(result.impaired_count, 1);
    }

    #[test]
    fn test_no_write_down_when_nrv_exceeds_cost() {
        // Set avg_nrv_factor = 1.2 so NRV > cost → no write-down.
        let cfg = InventoryValuationGeneratorConfig {
            avg_nrv_factor: 1.2,
            nrv_factor_variation: 0.0,
            seed_offset: 1,
        };
        let gen = InventoryValuationGenerator::new(cfg, 77);
        let positions = vec![make_position("MAT002", "1000", dec!(50), dec!(20))];
        let result = gen.generate(
            "1000",
            &positions,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        assert_eq!(result.lines.len(), 1);
        assert!(
            !result.lines[0].is_impaired,
            "Position should not be impaired"
        );
        assert_eq!(result.total_write_down, Decimal::ZERO);
        assert_eq!(result.impaired_count, 0);
    }

    #[test]
    fn test_carrying_value_equals_cost_minus_writedown() {
        let cfg = InventoryValuationGeneratorConfig {
            avg_nrv_factor: 0.9,
            nrv_factor_variation: 0.0,
            seed_offset: 2,
        };
        let gen = InventoryValuationGenerator::new(cfg, 55);
        let positions = vec![make_position("MAT003", "1000", dec!(200), dec!(5))];
        let result = gen.generate(
            "1000",
            &positions,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        let line = &result.lines[0];
        assert_eq!(
            line.carrying_value,
            line.total_cost - line.write_down_amount,
            "carrying_value = total_cost - write_down"
        );
        assert_eq!(
            result.total_carrying_value,
            result.total_cost - result.total_write_down,
        );
    }

    #[test]
    fn test_filters_to_company() {
        let positions = vec![
            make_position("MAT010", "1000", dec!(10), dec!(100)),
            make_position("MAT011", "2000", dec!(20), dec!(50)), // different company
        ];
        let cfg = InventoryValuationGeneratorConfig::default();
        let gen = InventoryValuationGenerator::new(cfg, 1);
        let result = gen.generate(
            "1000",
            &positions,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        assert_eq!(result.lines.len(), 1, "Only MAT010 belongs to company 1000");
        assert_eq!(result.lines[0].material_id, "MAT010");
    }

    #[test]
    fn test_empty_positions_returns_zero_totals() {
        let cfg = InventoryValuationGeneratorConfig::default();
        let gen = InventoryValuationGenerator::new(cfg, 0);
        let result = gen.generate("1000", &[], NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        assert!(result.lines.is_empty());
        assert_eq!(result.total_cost, Decimal::ZERO);
        assert_eq!(result.total_write_down, Decimal::ZERO);
        assert_eq!(result.impaired_count, 0);
    }
}
