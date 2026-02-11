//! Inventory valuation models.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::{InventoryPosition, ValuationMethod};

/// Inventory valuation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryValuationReport {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Valuation method.
    pub valuation_method: ValuationMethod,
    /// Material valuations.
    pub materials: Vec<MaterialValuation>,
    /// Total inventory value.
    pub total_value: Decimal,
    /// Total quantity.
    pub total_quantity: Decimal,
    /// Value by plant.
    pub by_plant: HashMap<String, Decimal>,
    /// Value by material group.
    pub by_material_group: HashMap<String, Decimal>,
    /// Generated at.
    pub generated_at: DateTime<Utc>,
}

impl InventoryValuationReport {
    /// Creates a valuation report from positions.
    pub fn from_positions(
        company_code: String,
        positions: &[InventoryPosition],
        as_of_date: NaiveDate,
    ) -> Self {
        let mut materials = Vec::new();
        let mut total_value = Decimal::ZERO;
        let mut total_quantity = Decimal::ZERO;
        let mut by_plant: HashMap<String, Decimal> = HashMap::new();
        let by_material_group: HashMap<String, Decimal> = HashMap::new();

        for pos in positions.iter().filter(|p| p.company_code == company_code) {
            let value = pos.total_value();
            total_value += value;
            total_quantity += pos.quantity_on_hand;

            *by_plant.entry(pos.plant.clone()).or_default() += value;

            materials.push(MaterialValuation {
                material_id: pos.material_id.clone(),
                description: pos.description.clone(),
                plant: pos.plant.clone(),
                storage_location: pos.storage_location.clone(),
                quantity: pos.quantity_on_hand,
                unit: pos.unit.clone(),
                unit_cost: pos.valuation.unit_cost,
                total_value: value,
                valuation_method: pos.valuation.method,
                standard_cost: pos.valuation.standard_cost,
                price_variance: pos.valuation.price_variance,
            });
        }

        // Sort by value descending
        materials.sort_by(|a, b| b.total_value.cmp(&a.total_value));

        Self {
            company_code,
            as_of_date,
            valuation_method: ValuationMethod::StandardCost,
            materials,
            total_value,
            total_quantity,
            by_plant,
            by_material_group,
            generated_at: Utc::now(),
        }
    }

    /// Gets top N materials by value.
    pub fn top_materials(&self, n: usize) -> Vec<&MaterialValuation> {
        self.materials.iter().take(n).collect()
    }

    /// Gets ABC classification.
    pub fn abc_analysis(&self) -> ABCAnalysis {
        ABCAnalysis::from_valuations(&self.materials, self.total_value)
    }
}

/// Material valuation detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialValuation {
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Plant.
    pub plant: String,
    /// Storage location.
    pub storage_location: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit.
    pub unit: String,
    /// Unit cost.
    pub unit_cost: Decimal,
    /// Total value.
    pub total_value: Decimal,
    /// Valuation method.
    pub valuation_method: ValuationMethod,
    /// Standard cost.
    pub standard_cost: Decimal,
    /// Price variance.
    pub price_variance: Decimal,
}

/// ABC Analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABCAnalysis {
    /// A items (typically 80% of value, 20% of items).
    pub a_items: Vec<ABCItem>,
    /// B items (typically 15% of value, 30% of items).
    pub b_items: Vec<ABCItem>,
    /// C items (typically 5% of value, 50% of items).
    pub c_items: Vec<ABCItem>,
    /// A threshold percentage.
    pub a_threshold: Decimal,
    /// B threshold percentage.
    pub b_threshold: Decimal,
    /// Summary statistics.
    pub summary: ABCSummary,
}

impl ABCAnalysis {
    /// Creates ABC analysis from valuations.
    pub fn from_valuations(valuations: &[MaterialValuation], total_value: Decimal) -> Self {
        let a_threshold = dec!(80);
        let b_threshold = dec!(95);

        let mut sorted: Vec<_> = valuations.iter().collect();
        sorted.sort_by(|a, b| b.total_value.cmp(&a.total_value));

        let mut a_items = Vec::new();
        let mut b_items = Vec::new();
        let mut c_items = Vec::new();

        let mut cumulative_value = Decimal::ZERO;

        for val in sorted {
            cumulative_value += val.total_value;
            let cumulative_percent = if total_value > Decimal::ZERO {
                cumulative_value / total_value * dec!(100)
            } else {
                Decimal::ZERO
            };

            let item = ABCItem {
                material_id: val.material_id.clone(),
                description: val.description.clone(),
                value: val.total_value,
                cumulative_percent,
            };

            if cumulative_percent <= a_threshold {
                a_items.push(item);
            } else if cumulative_percent <= b_threshold {
                b_items.push(item);
            } else {
                c_items.push(item);
            }
        }

        let summary = ABCSummary {
            a_count: a_items.len() as u32,
            a_value: a_items.iter().map(|i| i.value).sum(),
            a_percent: if total_value > Decimal::ZERO {
                a_items.iter().map(|i| i.value).sum::<Decimal>() / total_value * dec!(100)
            } else {
                Decimal::ZERO
            },
            b_count: b_items.len() as u32,
            b_value: b_items.iter().map(|i| i.value).sum(),
            b_percent: if total_value > Decimal::ZERO {
                b_items.iter().map(|i| i.value).sum::<Decimal>() / total_value * dec!(100)
            } else {
                Decimal::ZERO
            },
            c_count: c_items.len() as u32,
            c_value: c_items.iter().map(|i| i.value).sum(),
            c_percent: if total_value > Decimal::ZERO {
                c_items.iter().map(|i| i.value).sum::<Decimal>() / total_value * dec!(100)
            } else {
                Decimal::ZERO
            },
        };

        Self {
            a_items,
            b_items,
            c_items,
            a_threshold,
            b_threshold,
            summary,
        }
    }
}

/// Item in ABC classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABCItem {
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Value.
    pub value: Decimal,
    /// Cumulative percentage.
    pub cumulative_percent: Decimal,
}

/// ABC analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABCSummary {
    /// A item count.
    pub a_count: u32,
    /// A total value.
    pub a_value: Decimal,
    /// A percentage.
    pub a_percent: Decimal,
    /// B item count.
    pub b_count: u32,
    /// B total value.
    pub b_value: Decimal,
    /// B percentage.
    pub b_percent: Decimal,
    /// C item count.
    pub c_count: u32,
    /// C total value.
    pub c_value: Decimal,
    /// C percentage.
    pub c_percent: Decimal,
}

/// FIFO valuation layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FIFOLayer {
    /// Receipt date.
    pub receipt_date: NaiveDate,
    /// Receipt document.
    pub receipt_document: String,
    /// Quantity remaining.
    pub quantity: Decimal,
    /// Unit cost.
    pub unit_cost: Decimal,
}

/// FIFO inventory tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FIFOTracker {
    /// Material ID.
    pub material_id: String,
    /// Layers (oldest first).
    pub layers: VecDeque<FIFOLayer>,
    /// Total quantity.
    pub total_quantity: Decimal,
    /// Total value.
    pub total_value: Decimal,
}

impl FIFOTracker {
    /// Creates a new FIFO tracker.
    pub fn new(material_id: String) -> Self {
        Self {
            material_id,
            layers: VecDeque::new(),
            total_quantity: Decimal::ZERO,
            total_value: Decimal::ZERO,
        }
    }

    /// Adds a receipt.
    pub fn receive(
        &mut self,
        date: NaiveDate,
        document: String,
        quantity: Decimal,
        unit_cost: Decimal,
    ) {
        self.layers.push_back(FIFOLayer {
            receipt_date: date,
            receipt_document: document,
            quantity,
            unit_cost,
        });
        self.total_quantity += quantity;
        self.total_value += quantity * unit_cost;
    }

    /// Issues quantity using FIFO.
    pub fn issue(&mut self, quantity: Decimal) -> Option<Decimal> {
        if quantity > self.total_quantity {
            return None;
        }

        let mut remaining = quantity;
        let mut total_cost = Decimal::ZERO;

        while remaining > Decimal::ZERO && !self.layers.is_empty() {
            let front = self
                .layers
                .front_mut()
                .expect("FIFO layer exists when remaining > 0");

            if front.quantity <= remaining {
                // Consume entire layer
                total_cost += front.quantity * front.unit_cost;
                remaining -= front.quantity;
                self.layers.pop_front();
            } else {
                // Partial consumption
                total_cost += remaining * front.unit_cost;
                front.quantity -= remaining;
                remaining = Decimal::ZERO;
            }
        }

        self.total_quantity -= quantity;
        self.total_value -= total_cost;

        Some(total_cost)
    }

    /// Gets weighted average cost.
    pub fn weighted_average_cost(&self) -> Decimal {
        if self.total_quantity > Decimal::ZERO {
            self.total_value / self.total_quantity
        } else {
            Decimal::ZERO
        }
    }
}

/// Standard cost variance analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostVarianceAnalysis {
    /// Company code.
    pub company_code: String,
    /// Period.
    pub period: String,
    /// Material variances.
    pub variances: Vec<MaterialCostVariance>,
    /// Total price variance.
    pub total_price_variance: Decimal,
    /// Total quantity variance.
    pub total_quantity_variance: Decimal,
    /// Generated at.
    pub generated_at: DateTime<Utc>,
}

/// Material cost variance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCostVariance {
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Standard cost.
    pub standard_cost: Decimal,
    /// Actual cost.
    pub actual_cost: Decimal,
    /// Quantity.
    pub quantity: Decimal,
    /// Price variance.
    pub price_variance: Decimal,
    /// Variance percentage.
    pub variance_percent: Decimal,
    /// Is favorable.
    pub is_favorable: bool,
}

impl MaterialCostVariance {
    /// Creates a new variance record.
    pub fn new(
        material_id: String,
        description: String,
        standard_cost: Decimal,
        actual_cost: Decimal,
        quantity: Decimal,
    ) -> Self {
        let price_variance = (standard_cost - actual_cost) * quantity;
        let variance_percent = if actual_cost > Decimal::ZERO {
            ((standard_cost - actual_cost) / actual_cost * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };
        let is_favorable = price_variance > Decimal::ZERO;

        Self {
            material_id,
            description,
            standard_cost,
            actual_cost,
            quantity,
            price_variance,
            variance_percent,
            is_favorable,
        }
    }
}

/// Inventory turnover analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTurnover {
    /// Company code.
    pub company_code: String,
    /// Period start.
    pub period_start: NaiveDate,
    /// Period end.
    pub period_end: NaiveDate,
    /// Average inventory.
    pub average_inventory: Decimal,
    /// Cost of goods sold.
    pub cogs: Decimal,
    /// Turnover ratio.
    pub turnover_ratio: Decimal,
    /// Days inventory outstanding.
    pub dio_days: Decimal,
    /// By material.
    pub by_material: Vec<MaterialTurnover>,
}

impl InventoryTurnover {
    /// Calculates turnover from COGS and inventory levels.
    pub fn calculate(
        company_code: String,
        period_start: NaiveDate,
        period_end: NaiveDate,
        beginning_inventory: Decimal,
        ending_inventory: Decimal,
        cogs: Decimal,
    ) -> Self {
        let average_inventory = (beginning_inventory + ending_inventory) / dec!(2);
        let days_in_period = (period_end - period_start).num_days() as i32;

        let turnover_ratio = if average_inventory > Decimal::ZERO {
            (cogs / average_inventory).round_dp(2)
        } else {
            Decimal::ZERO
        };

        let dio_days = if cogs > Decimal::ZERO {
            (average_inventory / cogs * Decimal::from(days_in_period)).round_dp(1)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code,
            period_start,
            period_end,
            average_inventory,
            cogs,
            turnover_ratio,
            dio_days,
            by_material: Vec::new(),
        }
    }
}

/// Material-level turnover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTurnover {
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Average inventory.
    pub average_inventory: Decimal,
    /// Usage/COGS.
    pub usage: Decimal,
    /// Turnover ratio.
    pub turnover_ratio: Decimal,
    /// Days of supply.
    pub days_of_supply: Decimal,
    /// Classification (fast/slow/dead).
    pub classification: TurnoverClassification,
}

/// Turnover classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnoverClassification {
    /// Fast moving.
    FastMoving,
    /// Normal.
    Normal,
    /// Slow moving.
    SlowMoving,
    /// Dead/obsolete stock.
    Dead,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_tracker() {
        let mut tracker = FIFOTracker::new("MAT001".to_string());

        // Receive 100 @ $10
        tracker.receive(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            "GR001".to_string(),
            dec!(100),
            dec!(10),
        );

        // Receive 100 @ $12
        tracker.receive(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "GR002".to_string(),
            dec!(100),
            dec!(12),
        );

        assert_eq!(tracker.total_quantity, dec!(200));
        assert_eq!(tracker.total_value, dec!(2200)); // 1000 + 1200

        // Issue 150 (should use FIFO - 100 @ $10 + 50 @ $12)
        let cost = tracker.issue(dec!(150)).unwrap();
        assert_eq!(cost, dec!(1600)); // (100 * 10) + (50 * 12)
        assert_eq!(tracker.total_quantity, dec!(50));
    }

    #[test]
    fn test_abc_analysis() {
        let valuations = vec![
            MaterialValuation {
                material_id: "A".to_string(),
                description: "High value".to_string(),
                plant: "P1".to_string(),
                storage_location: "S1".to_string(),
                quantity: dec!(10),
                unit: "EA".to_string(),
                unit_cost: dec!(100),
                total_value: dec!(1000),
                valuation_method: ValuationMethod::StandardCost,
                standard_cost: dec!(100),
                price_variance: Decimal::ZERO,
            },
            MaterialValuation {
                material_id: "B".to_string(),
                description: "Medium value".to_string(),
                plant: "P1".to_string(),
                storage_location: "S1".to_string(),
                quantity: dec!(50),
                unit: "EA".to_string(),
                unit_cost: dec!(10),
                total_value: dec!(500),
                valuation_method: ValuationMethod::StandardCost,
                standard_cost: dec!(10),
                price_variance: Decimal::ZERO,
            },
            MaterialValuation {
                material_id: "C".to_string(),
                description: "Low value".to_string(),
                plant: "P1".to_string(),
                storage_location: "S1".to_string(),
                quantity: dec!(100),
                unit: "EA".to_string(),
                unit_cost: dec!(1),
                total_value: dec!(100),
                valuation_method: ValuationMethod::StandardCost,
                standard_cost: dec!(1),
                price_variance: Decimal::ZERO,
            },
        ];

        let total = dec!(1600);
        let analysis = ABCAnalysis::from_valuations(&valuations, total);

        // Material A (1000/1600 = 62.5%) should be A
        assert!(!analysis.a_items.is_empty());
    }

    #[test]
    fn test_inventory_turnover() {
        let turnover = InventoryTurnover::calculate(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(100_000),
            dec!(120_000),
            dec!(1_000_000),
        );

        // Average inventory = 110,000
        // Turnover = 1,000,000 / 110,000 = 9.09
        assert!(turnover.turnover_ratio > dec!(9));

        // DIO = 110,000 / 1,000,000 * 365 ≈ 40 days
        assert!(turnover.dio_days > dec!(30) && turnover.dio_days < dec!(50));
    }

    #[test]
    fn test_cost_variance() {
        let variance = MaterialCostVariance::new(
            "MAT001".to_string(),
            "Test Material".to_string(),
            dec!(10),  // Standard
            dec!(11),  // Actual (higher)
            dec!(100), // Quantity
        );

        // Variance = (10 - 11) * 100 = -100 (unfavorable)
        assert_eq!(variance.price_variance, dec!(-100));
        assert!(!variance.is_favorable);
    }
}
