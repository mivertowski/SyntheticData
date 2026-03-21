//! Inventory position model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Inventory position (stock on hand).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryPosition {
    /// Material ID.
    pub material_id: String,
    /// Material description.
    pub description: String,
    /// Plant/warehouse.
    pub plant: String,
    /// Storage location.
    pub storage_location: String,
    /// Company code.
    pub company_code: String,
    /// Quantity on hand.
    pub quantity_on_hand: Decimal,
    /// Unit of measure.
    pub unit: String,
    /// Reserved quantity.
    pub quantity_reserved: Decimal,
    /// Available quantity (on hand - reserved).
    pub quantity_available: Decimal,
    /// Quality inspection quantity.
    pub quantity_in_inspection: Decimal,
    /// Blocked quantity.
    pub quantity_blocked: Decimal,
    /// In-transit quantity.
    pub quantity_in_transit: Decimal,
    /// Valuation data.
    pub valuation: PositionValuation,
    /// Last movement date.
    pub last_movement_date: Option<NaiveDate>,
    /// Last count date.
    pub last_count_date: Option<NaiveDate>,
    /// Minimum stock level.
    pub min_stock: Option<Decimal>,
    /// Maximum stock level.
    pub max_stock: Option<Decimal>,
    /// Reorder point.
    pub reorder_point: Option<Decimal>,
    /// Safety stock.
    pub safety_stock: Option<Decimal>,
    /// Stock status.
    pub status: StockStatus,
    /// Batch/lot tracking.
    pub batches: Vec<BatchStock>,
    /// Serial numbers (if serialized).
    pub serial_numbers: Vec<SerialNumber>,
    /// Last updated.
    pub updated_at: DateTime<Utc>,
}

impl InventoryPosition {
    /// Creates a new inventory position.
    pub fn new(
        material_id: String,
        description: String,
        plant: String,
        storage_location: String,
        company_code: String,
        unit: String,
    ) -> Self {
        Self {
            material_id,
            description,
            plant,
            storage_location,
            company_code,
            quantity_on_hand: Decimal::ZERO,
            unit,
            quantity_reserved: Decimal::ZERO,
            quantity_available: Decimal::ZERO,
            quantity_in_inspection: Decimal::ZERO,
            quantity_blocked: Decimal::ZERO,
            quantity_in_transit: Decimal::ZERO,
            valuation: PositionValuation::default(),
            last_movement_date: None,
            last_count_date: None,
            min_stock: None,
            max_stock: None,
            reorder_point: None,
            safety_stock: None,
            status: StockStatus::Normal,
            batches: Vec::new(),
            serial_numbers: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Calculates available quantity.
    pub fn calculate_available(&mut self) {
        self.quantity_available = self.quantity_on_hand
            - self.quantity_reserved
            - self.quantity_in_inspection
            - self.quantity_blocked;
    }

    /// Adds quantity to position.
    pub fn add_quantity(&mut self, quantity: Decimal, cost: Decimal, date: NaiveDate) {
        self.quantity_on_hand += quantity;
        self.valuation.update_on_receipt(quantity, cost);
        self.last_movement_date = Some(date);
        self.calculate_available();
        self.update_status();
        self.updated_at = Utc::now();
    }

    /// Removes quantity from position.
    pub fn remove_quantity(&mut self, quantity: Decimal, date: NaiveDate) -> Option<Decimal> {
        if quantity > self.quantity_available {
            return None;
        }

        let cost = self.valuation.calculate_issue_cost(quantity);
        self.quantity_on_hand -= quantity;
        self.last_movement_date = Some(date);
        self.calculate_available();
        self.update_status();
        self.updated_at = Utc::now();

        Some(cost)
    }

    /// Reserves quantity.
    pub fn reserve(&mut self, quantity: Decimal) -> bool {
        if quantity > self.quantity_available {
            return false;
        }
        self.quantity_reserved += quantity;
        self.calculate_available();
        self.updated_at = Utc::now();
        true
    }

    /// Releases reservation.
    pub fn release_reservation(&mut self, quantity: Decimal) {
        self.quantity_reserved = (self.quantity_reserved - quantity).max(Decimal::ZERO);
        self.calculate_available();
        self.updated_at = Utc::now();
    }

    /// Blocks quantity.
    pub fn block(&mut self, quantity: Decimal) {
        self.quantity_blocked += quantity;
        self.calculate_available();
        self.updated_at = Utc::now();
    }

    /// Unblocks quantity.
    pub fn unblock(&mut self, quantity: Decimal) {
        self.quantity_blocked = (self.quantity_blocked - quantity).max(Decimal::ZERO);
        self.calculate_available();
        self.updated_at = Utc::now();
    }

    /// Updates stock status based on levels.
    fn update_status(&mut self) {
        if self.quantity_on_hand <= Decimal::ZERO {
            self.status = StockStatus::OutOfStock;
        } else if let Some(safety) = self.safety_stock {
            if self.quantity_on_hand <= safety {
                self.status = StockStatus::BelowSafety;
            } else if let Some(reorder) = self.reorder_point {
                if self.quantity_on_hand <= reorder {
                    self.status = StockStatus::BelowReorder;
                } else {
                    self.status = StockStatus::Normal;
                }
            } else {
                self.status = StockStatus::Normal;
            }
        } else {
            self.status = if self.quantity_on_hand > Decimal::ZERO {
                StockStatus::Normal
            } else {
                StockStatus::OutOfStock
            };
        }
    }

    /// Sets stock level parameters.
    pub fn with_stock_levels(
        mut self,
        min: Decimal,
        max: Decimal,
        reorder: Decimal,
        safety: Decimal,
    ) -> Self {
        self.min_stock = Some(min);
        self.max_stock = Some(max);
        self.reorder_point = Some(reorder);
        self.safety_stock = Some(safety);
        self.update_status();
        self
    }

    /// Gets total inventory value.
    pub fn total_value(&self) -> Decimal {
        self.quantity_on_hand * self.valuation.unit_cost
    }

    /// Checks if reorder is needed.
    pub fn needs_reorder(&self) -> bool {
        self.reorder_point
            .map(|rp| self.quantity_available <= rp)
            .unwrap_or(false)
    }

    /// Gets days of supply based on average usage.
    pub fn days_of_supply(&self, average_daily_usage: Decimal) -> Option<Decimal> {
        if average_daily_usage > Decimal::ZERO {
            Some((self.quantity_available / average_daily_usage).round_dp(1))
        } else {
            None
        }
    }
}

/// Valuation data for inventory position.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionValuation {
    /// Valuation method.
    pub method: ValuationMethod,
    /// Standard cost (if standard costing).
    pub standard_cost: Decimal,
    /// Moving average unit cost.
    pub unit_cost: Decimal,
    /// Total value.
    pub total_value: Decimal,
    /// Price variance (standard vs actual).
    pub price_variance: Decimal,
    /// Last price change date.
    pub last_price_change: Option<NaiveDate>,
}

impl PositionValuation {
    /// Updates valuation on receipt using the weighted-average cost formula.
    ///
    /// The `existing_qty` parameter is the quantity *before* this receipt is applied.
    /// For moving-average valuation, the new unit cost is:
    ///
    /// ```text
    /// new_avg_cost = (existing_qty × existing_unit_cost + receipt_qty × receipt_unit_cost)
    ///                / (existing_qty + receipt_qty)
    /// ```
    pub fn update_on_receipt(&mut self, receipt_qty: Decimal, cost: Decimal) {
        match self.method {
            ValuationMethod::StandardCost => {
                let actual_cost = cost;
                let standard_cost = receipt_qty * self.standard_cost;
                self.price_variance += actual_cost - standard_cost;
                self.total_value += standard_cost;
            }
            ValuationMethod::MovingAverage => {
                // cost is the total receipt value (receipt_qty × receipt_unit_cost).
                // Reconstruct the existing quantity from total_value / unit_cost, then
                // apply the weighted-average formula:
                //   new_unit_cost = new_total_value / (existing_qty + receipt_qty)
                let existing_qty = if self.unit_cost > Decimal::ZERO {
                    self.total_value / self.unit_cost
                } else {
                    Decimal::ZERO
                };
                let new_qty = existing_qty + receipt_qty;
                self.total_value += cost;
                if new_qty > Decimal::ZERO {
                    self.unit_cost = (self.total_value / new_qty).round_dp(4);
                }
            }
            ValuationMethod::FIFO | ValuationMethod::LIFO => {
                self.total_value += cost;
            }
        }
    }

    /// Calculates cost for issue.
    pub fn calculate_issue_cost(&mut self, quantity: Decimal) -> Decimal {
        let cost = quantity * self.unit_cost;
        self.total_value = (self.total_value - cost).max(Decimal::ZERO);
        cost
    }
}

/// Valuation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ValuationMethod {
    /// Standard cost.
    #[default]
    StandardCost,
    /// Moving average.
    MovingAverage,
    /// First-in, first-out.
    FIFO,
    /// Last-in, first-out.
    LIFO,
}

/// Stock status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StockStatus {
    /// Normal stock level.
    #[default]
    Normal,
    /// Below reorder point.
    BelowReorder,
    /// Below safety stock.
    BelowSafety,
    /// Out of stock.
    OutOfStock,
    /// Over maximum.
    OverMax,
    /// Obsolete/slow moving.
    Obsolete,
}

/// Batch/lot stock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStock {
    /// Batch number.
    pub batch_number: String,
    /// Quantity in batch.
    pub quantity: Decimal,
    /// Manufacturing date.
    pub manufacture_date: Option<NaiveDate>,
    /// Expiration date.
    pub expiration_date: Option<NaiveDate>,
    /// Supplier batch.
    pub supplier_batch: Option<String>,
    /// Status.
    pub status: BatchStatus,
    /// Unit cost for this batch.
    pub unit_cost: Decimal,
}

impl BatchStock {
    /// Creates a new batch.
    pub fn new(batch_number: String, quantity: Decimal, unit_cost: Decimal) -> Self {
        Self {
            batch_number,
            quantity,
            manufacture_date: None,
            expiration_date: None,
            supplier_batch: None,
            status: BatchStatus::Unrestricted,
            unit_cost,
        }
    }

    /// Checks if batch is expired.
    pub fn is_expired(&self, as_of_date: NaiveDate) -> bool {
        self.expiration_date
            .map(|exp| as_of_date > exp)
            .unwrap_or(false)
    }

    /// Checks if batch is expiring soon (within days).
    pub fn is_expiring_soon(&self, as_of_date: NaiveDate, days: i64) -> bool {
        self.expiration_date
            .map(|exp| {
                let threshold = as_of_date + chrono::Duration::days(days);
                as_of_date <= exp && exp <= threshold
            })
            .unwrap_or(false)
    }
}

/// Batch status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Available for use.
    #[default]
    Unrestricted,
    /// Quality inspection.
    InInspection,
    /// Blocked.
    Blocked,
    /// Expired.
    Expired,
    /// Reserved.
    Reserved,
}

/// Serial number tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialNumber {
    /// Serial number.
    pub serial_number: String,
    /// Status.
    pub status: SerialStatus,
    /// Receipt date.
    pub receipt_date: NaiveDate,
    /// Issue date (if issued).
    pub issue_date: Option<NaiveDate>,
    /// Customer (if sold).
    pub customer_id: Option<String>,
    /// Warranty expiration.
    pub warranty_expiration: Option<NaiveDate>,
}

/// Serial number status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SerialStatus {
    /// In stock.
    #[default]
    InStock,
    /// Reserved.
    Reserved,
    /// Issued/sold.
    Issued,
    /// In repair.
    InRepair,
    /// Scrapped.
    Scrapped,
}

/// Inventory summary by plant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Summary by plant.
    pub by_plant: HashMap<String, PlantInventorySummary>,
    /// Total inventory value.
    pub total_value: Decimal,
    /// Total SKU count.
    pub total_sku_count: u32,
    /// Items below reorder point.
    pub below_reorder_count: u32,
    /// Out of stock count.
    pub out_of_stock_count: u32,
}

/// Plant-level inventory summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantInventorySummary {
    /// Plant code.
    pub plant: String,
    /// Total value.
    pub total_value: Decimal,
    /// SKU count.
    pub sku_count: u32,
    /// Below reorder count.
    pub below_reorder_count: u32,
    /// Out of stock count.
    pub out_of_stock_count: u32,
    /// Total quantity.
    pub total_quantity: Decimal,
}

impl InventorySummary {
    /// Creates summary from positions.
    pub fn from_positions(
        company_code: String,
        positions: &[InventoryPosition],
        as_of_date: NaiveDate,
    ) -> Self {
        let mut by_plant: HashMap<String, PlantInventorySummary> = HashMap::new();
        let mut total_value = Decimal::ZERO;
        let mut total_sku_count = 0u32;
        let mut below_reorder_count = 0u32;
        let mut out_of_stock_count = 0u32;

        for pos in positions.iter().filter(|p| p.company_code == company_code) {
            let plant_summary =
                by_plant
                    .entry(pos.plant.clone())
                    .or_insert_with(|| PlantInventorySummary {
                        plant: pos.plant.clone(),
                        total_value: Decimal::ZERO,
                        sku_count: 0,
                        below_reorder_count: 0,
                        out_of_stock_count: 0,
                        total_quantity: Decimal::ZERO,
                    });

            let value = pos.total_value();
            plant_summary.total_value += value;
            plant_summary.sku_count += 1;
            plant_summary.total_quantity += pos.quantity_on_hand;

            total_value += value;
            total_sku_count += 1;

            match pos.status {
                StockStatus::BelowReorder | StockStatus::BelowSafety => {
                    plant_summary.below_reorder_count += 1;
                    below_reorder_count += 1;
                }
                StockStatus::OutOfStock => {
                    plant_summary.out_of_stock_count += 1;
                    out_of_stock_count += 1;
                }
                _ => {}
            }
        }

        Self {
            company_code,
            as_of_date,
            by_plant,
            total_value,
            total_sku_count,
            below_reorder_count,
            out_of_stock_count,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_position() -> InventoryPosition {
        InventoryPosition::new(
            "MAT001".to_string(),
            "Test Material".to_string(),
            "PLANT01".to_string(),
            "SLOC01".to_string(),
            "1000".to_string(),
            "EA".to_string(),
        )
    }

    #[test]
    fn test_add_quantity() {
        let mut pos = create_test_position();
        pos.valuation.unit_cost = dec!(10);

        pos.add_quantity(
            dec!(100),
            dec!(1000),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        assert_eq!(pos.quantity_on_hand, dec!(100));
        assert_eq!(pos.quantity_available, dec!(100));
    }

    #[test]
    fn test_reserve_quantity() {
        let mut pos = create_test_position();
        pos.quantity_on_hand = dec!(100);
        pos.calculate_available();

        assert!(pos.reserve(dec!(30)));
        assert_eq!(pos.quantity_reserved, dec!(30));
        assert_eq!(pos.quantity_available, dec!(70));

        // Try to reserve more than available
        assert!(!pos.reserve(dec!(80)));
    }

    #[test]
    fn test_stock_status() {
        let mut pos =
            create_test_position().with_stock_levels(dec!(10), dec!(200), dec!(50), dec!(20));

        // Use add_quantity to properly update status
        pos.add_quantity(
            dec!(100),
            dec!(1000),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        assert_eq!(pos.status, StockStatus::Normal);

        // Remove quantity to go below reorder point (50) but above safety (20)
        let _ = pos.remove_quantity(dec!(70), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        // Now quantity is 30, which is below reorder (50) but above safety (20)
        assert_eq!(pos.status, StockStatus::BelowReorder);
    }

    #[test]
    fn test_batch_expiration() {
        let batch = BatchStock {
            batch_number: "BATCH001".to_string(),
            quantity: dec!(100),
            manufacture_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            expiration_date: Some(NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()),
            supplier_batch: None,
            status: BatchStatus::Unrestricted,
            unit_cost: dec!(10),
        };

        let before = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let after = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();

        assert!(!batch.is_expired(before));
        assert!(batch.is_expired(after));
        assert!(batch.is_expiring_soon(before, 30));
    }
}
