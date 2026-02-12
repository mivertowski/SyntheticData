//! Production order generator for manufacturing processes.
//!
//! Generates realistic production orders with routing operations, costing,
//! yield calculations, and proper order lifecycle status distribution.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::{ManufacturingCostingConfig, ProductionOrderConfig, RoutingConfig};
use datasynth_core::models::{
    OperationStatus, ProductionOrder, ProductionOrderStatus, ProductionOrderType, RoutingOperation,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Work center identifiers used in routing operations.
const WORK_CENTERS: &[&str] = &["WC-100", "WC-200", "WC-300", "WC-400", "WC-500"];

/// Operation descriptions for routing steps.
const OPERATION_DESCRIPTIONS: &[&str] = &[
    "Material Preparation",
    "Cutting",
    "Machining",
    "Assembly",
    "Welding",
    "Heat Treatment",
    "Surface Finishing",
    "Quality Check",
    "Packaging",
    "Final Inspection",
];

/// Generates [`ProductionOrder`] instances with realistic routing operations,
/// costing, and lifecycle status distributions.
pub struct ProductionOrderGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl ProductionOrderGenerator {
    /// Create a new production order generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProductionOrder),
        }
    }

    /// Generate production orders for the given period based on configuration.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company code for all generated orders.
    /// * `material_ids` - Available materials as `(material_id, description)` tuples.
    /// * `period_start` - Start of the generation period.
    /// * `period_end` - End of the generation period.
    /// * `config` - Production order configuration parameters.
    /// * `costing` - Manufacturing costing parameters.
    /// * `routing` - Routing configuration parameters.
    pub fn generate(
        &mut self,
        company_code: &str,
        material_ids: &[(String, String)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &ProductionOrderConfig,
        costing: &ManufacturingCostingConfig,
        routing: &RoutingConfig,
    ) -> Vec<ProductionOrder> {
        if material_ids.is_empty() {
            return Vec::new();
        }

        let mut orders = Vec::new();

        // Iterate month-by-month through the period
        let mut current = period_start;
        while current <= period_end {
            let month_end = end_of_month(current).min(period_end);

            for _ in 0..config.orders_per_month {
                let order = self.generate_one(
                    company_code,
                    material_ids,
                    current,
                    month_end,
                    config,
                    costing,
                    routing,
                );
                orders.push(order);
            }

            // Advance to the first day of the next month
            current = next_month_start(current);
        }

        orders
    }

    /// Generate a single production order.
    fn generate_one(
        &mut self,
        company_code: &str,
        material_ids: &[(String, String)],
        month_start: NaiveDate,
        month_end: NaiveDate,
        config: &ProductionOrderConfig,
        costing: &ManufacturingCostingConfig,
        routing: &RoutingConfig,
    ) -> ProductionOrder {
        let order_id = self.uuid_factory.next().to_string();

        // Pick a random material
        let (material_id, material_description) = material_ids
            .choose(&mut self.rng)
            .map(|(id, desc)| (id.clone(), desc.clone()))
            .unwrap_or_else(|| ("MAT-UNKNOWN".to_string(), "Unknown Material".to_string()));

        // Determine order type
        let order_type = self.pick_order_type(config);

        // Determine status
        let status = self.pick_status();

        // Batch size: avg * random factor (0.5 - 1.5)
        let batch_factor: f64 = self.rng.gen_range(0.5..=1.5);
        let planned_qty_f64 = config.avg_batch_size as f64 * batch_factor;
        let planned_quantity = Decimal::from_f64_retain(planned_qty_f64.round())
            .unwrap_or(Decimal::from(config.avg_batch_size));

        // Yield: actual = planned * yield_rate * random(0.95 - 1.05)
        let yield_factor: f64 = self.rng.gen_range(0.95..=1.05);
        let effective_yield = config.yield_rate * yield_factor;
        let actual_qty_f64 = planned_qty_f64 * effective_yield;
        let actual_quantity =
            Decimal::from_f64_retain(actual_qty_f64.round()).unwrap_or(planned_quantity);
        let scrap_quantity = (planned_quantity - actual_quantity).max(Decimal::ZERO);

        // Dates
        let days_in_month = (month_end - month_start).num_days().max(1);
        let start_offset = self.rng.gen_range(0..days_in_month);
        let planned_start = month_start + chrono::Duration::days(start_offset);
        let production_days = self.rng.gen_range(3..=14);
        let planned_end = planned_start + chrono::Duration::days(production_days);

        // Actual dates depend on status
        let (actual_start, actual_end) = match status {
            ProductionOrderStatus::Planned => (None, None),
            ProductionOrderStatus::Released => (None, None),
            ProductionOrderStatus::InProcess => {
                let offset = self.rng.gen_range(0..=2);
                (Some(planned_start + chrono::Duration::days(offset)), None)
            }
            ProductionOrderStatus::Completed | ProductionOrderStatus::Closed => {
                let start_offset = self.rng.gen_range(0..=2);
                let end_offset = self.rng.gen_range(-1..=3);
                (
                    Some(planned_start + chrono::Duration::days(start_offset)),
                    Some(planned_end + chrono::Duration::days(end_offset)),
                )
            }
            ProductionOrderStatus::Cancelled => (None, None),
        };

        // Work center
        let work_center = WORK_CENTERS
            .choose(&mut self.rng)
            .unwrap_or(&"WC-100")
            .to_string();

        // Routing operations
        let variation: i32 = self.rng.gen_range(-1..=1);
        let num_operations = (routing.avg_operations as i32 + variation).max(1) as u32;
        let operations = self.generate_operations(
            num_operations,
            planned_quantity,
            actual_quantity,
            routing,
            &status,
            planned_start,
            planned_end,
        );

        // Labor hours: sum of operation times * (1 + variation)
        let raw_labor_hours: f64 = operations
            .iter()
            .map(|op| op.setup_time_hours + op.run_time_hours)
            .sum();
        let labor_variation: f64 = self
            .rng
            .gen_range(-routing.run_time_variation..=routing.run_time_variation);
        let labor_hours = raw_labor_hours * (1.0 + labor_variation);

        // Machine hours: 70% of labor hours
        let machine_hours = labor_hours * 0.7;

        // Costing
        let labor_cost = labor_hours * costing.labor_rate_per_hour;
        let overhead_cost = labor_cost * costing.overhead_rate;
        let material_cost = planned_qty_f64 * self.rng.gen_range(5.0..50.0);
        let planned_cost_f64 = labor_cost + overhead_cost + material_cost;
        let planned_cost = Decimal::from_f64_retain(planned_cost_f64)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2);

        // Actual cost: planned * random(0.9 - 1.15)
        let cost_factor: f64 = self.rng.gen_range(0.9..=1.15);
        let actual_cost = Decimal::from_f64_retain(planned_cost_f64 * cost_factor)
            .unwrap_or(planned_cost)
            .round_dp(2);

        let routing_id = Some(format!("RT-{}", order_id.get(..8).unwrap_or("00000000")));
        let batch_number = Some(format!(
            "BATCH-{}-{:04}",
            planned_start.format("%Y%m%d"),
            self.rng.gen_range(1..=9999)
        ));

        ProductionOrder {
            order_id,
            company_code: company_code.to_string(),
            material_id,
            material_description,
            order_type,
            status,
            planned_quantity,
            actual_quantity,
            scrap_quantity,
            planned_start,
            planned_end,
            actual_start,
            actual_end,
            work_center,
            routing_id,
            planned_cost,
            actual_cost,
            labor_hours,
            machine_hours,
            yield_rate: effective_yield,
            batch_number,
            operations,
        }
    }

    /// Pick a production order type based on configured rates.
    fn pick_order_type(&mut self, config: &ProductionOrderConfig) -> ProductionOrderType {
        let roll: f64 = self.rng.gen();
        if roll < config.make_to_order_rate {
            ProductionOrderType::MakeToOrder
        } else if roll < config.make_to_order_rate + config.rework_rate {
            ProductionOrderType::Rework
        } else if self.rng.gen_bool(0.5) {
            ProductionOrderType::Standard
        } else {
            ProductionOrderType::MakeToStock
        }
    }

    /// Pick a production order status with realistic distribution.
    fn pick_status(&mut self) -> ProductionOrderStatus {
        let roll: f64 = self.rng.gen();
        if roll < 0.50 {
            ProductionOrderStatus::Completed
        } else if roll < 0.70 {
            ProductionOrderStatus::InProcess
        } else if roll < 0.85 {
            ProductionOrderStatus::Released
        } else if roll < 0.95 {
            ProductionOrderStatus::Closed
        } else {
            ProductionOrderStatus::Planned
        }
    }

    /// Generate routing operations for a production order.
    fn generate_operations(
        &mut self,
        count: u32,
        planned_quantity: Decimal,
        actual_quantity: Decimal,
        routing: &RoutingConfig,
        order_status: &ProductionOrderStatus,
        planned_start: NaiveDate,
        planned_end: NaiveDate,
    ) -> Vec<RoutingOperation> {
        let total_days = (planned_end - planned_start).num_days().max(1);
        let days_per_op = total_days / count as i64;

        (0..count)
            .map(|i| {
                let op_number = (i + 1) * 10;
                let desc_idx = (i as usize) % OPERATION_DESCRIPTIONS.len();
                let description = OPERATION_DESCRIPTIONS[desc_idx].to_string();

                let wc = WORK_CENTERS
                    .choose(&mut self.rng)
                    .unwrap_or(&"WC-100")
                    .to_string();

                // Setup time with variation
                let setup_variation: f64 = self.rng.gen_range(0.8..=1.2);
                let setup_time_hours = routing.setup_time_hours * setup_variation;

                // Run time based on quantity and variation
                let base_run_hours: f64 = planned_quantity.to_f64().unwrap_or(100.0) * 0.05; // 0.05 hours per unit base
                let run_variation: f64 = self
                    .rng
                    .gen_range(1.0 - routing.run_time_variation..=1.0 + routing.run_time_variation);
                let run_time_hours = base_run_hours * run_variation;

                // Operation status depends on order status and sequence position
                let op_status = match order_status {
                    ProductionOrderStatus::Planned | ProductionOrderStatus::Released => {
                        OperationStatus::Pending
                    }
                    ProductionOrderStatus::InProcess => {
                        if i < count / 2 {
                            OperationStatus::Completed
                        } else if i == count / 2 {
                            OperationStatus::InProcess
                        } else {
                            OperationStatus::Pending
                        }
                    }
                    ProductionOrderStatus::Completed | ProductionOrderStatus::Closed => {
                        OperationStatus::Completed
                    }
                    ProductionOrderStatus::Cancelled => OperationStatus::Cancelled,
                };

                let op_start_offset = i as i64 * days_per_op;
                let started_at = match op_status {
                    OperationStatus::InProcess | OperationStatus::Completed => {
                        Some(planned_start + chrono::Duration::days(op_start_offset))
                    }
                    _ => None,
                };
                let completed_at = match op_status {
                    OperationStatus::Completed => {
                        Some(planned_start + chrono::Duration::days(op_start_offset + days_per_op))
                    }
                    _ => None,
                };

                RoutingOperation {
                    operation_number: op_number,
                    operation_description: description,
                    work_center: wc,
                    setup_time_hours,
                    run_time_hours,
                    planned_quantity,
                    actual_quantity,
                    status: op_status,
                    started_at,
                    completed_at,
                }
            })
            .collect()
    }
}

/// Return the last day of the month for the given date.
fn end_of_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap_or(date)
        .pred_opt()
        .unwrap_or(date)
}

/// Return the first day of the month following the given date.
fn next_month_start(date: NaiveDate) -> NaiveDate {
    let (y, m) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(date)
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
            ("MAT-001".to_string(), "Widget Alpha".to_string()),
            ("MAT-002".to_string(), "Widget Beta".to_string()),
            ("MAT-003".to_string(), "Widget Gamma".to_string()),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = ProductionOrderGenerator::new(42);
        let materials = sample_materials();
        let config = ProductionOrderConfig::default();
        let costing = ManufacturingCostingConfig::default();
        let routing = RoutingConfig::default();

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let orders = gen.generate("C001", &materials, start, end, &config, &costing, &routing);

        assert_eq!(orders.len(), config.orders_per_month as usize);
        for order in &orders {
            assert_eq!(order.company_code, "C001");
            assert!(!order.order_id.is_empty());
            assert!(!order.material_id.is_empty());
            assert!(order.planned_quantity > Decimal::ZERO);
            assert!(order.planned_cost > Decimal::ZERO);
            assert!(!order.operations.is_empty());
            assert!(order.labor_hours > 0.0);
            assert!(order.machine_hours > 0.0);
        }
    }

    #[test]
    fn test_deterministic() {
        let materials = sample_materials();
        let config = ProductionOrderConfig::default();
        let costing = ManufacturingCostingConfig::default();
        let routing = RoutingConfig::default();

        let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let mut gen1 = ProductionOrderGenerator::new(12345);
        let orders1 = gen1.generate("C001", &materials, start, end, &config, &costing, &routing);

        let mut gen2 = ProductionOrderGenerator::new(12345);
        let orders2 = gen2.generate("C001", &materials, start, end, &config, &costing, &routing);

        assert_eq!(orders1.len(), orders2.len());
        for (o1, o2) in orders1.iter().zip(orders2.iter()) {
            assert_eq!(o1.order_id, o2.order_id);
            assert_eq!(o1.planned_quantity, o2.planned_quantity);
            assert_eq!(o1.actual_quantity, o2.actual_quantity);
            assert_eq!(o1.planned_cost, o2.planned_cost);
            assert_eq!(o1.actual_cost, o2.actual_cost);
        }
    }

    #[test]
    fn test_yield_and_scrap() {
        let mut gen = ProductionOrderGenerator::new(99);
        let materials = sample_materials();
        let config = ProductionOrderConfig {
            orders_per_month: 200,
            yield_rate: 0.90,
            ..Default::default()
        };
        let costing = ManufacturingCostingConfig::default();
        let routing = RoutingConfig::default();

        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let orders = gen.generate("C001", &materials, start, end, &config, &costing, &routing);

        // With a 90% yield rate, on average actual < planned
        let total_planned: Decimal = orders.iter().map(|o| o.planned_quantity).sum();
        let total_actual: Decimal = orders.iter().map(|o| o.actual_quantity).sum();
        assert!(
            total_actual < total_planned,
            "With 90% yield, total actual ({}) should be less than planned ({})",
            total_actual,
            total_planned,
        );

        // Scrap should be non-negative for all orders
        for order in &orders {
            assert!(
                order.scrap_quantity >= Decimal::ZERO,
                "Scrap quantity should be non-negative, got {}",
                order.scrap_quantity,
            );
        }
    }

    #[test]
    fn test_multi_month_period() {
        let mut gen = ProductionOrderGenerator::new(77);
        let materials = sample_materials();
        let config = ProductionOrderConfig {
            orders_per_month: 10,
            ..Default::default()
        };
        let costing = ManufacturingCostingConfig::default();
        let routing = RoutingConfig::default();

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let orders = gen.generate("C001", &materials, start, end, &config, &costing, &routing);

        // 3 months * 10 orders/month = 30 orders
        assert_eq!(orders.len(), 30);
    }
}
