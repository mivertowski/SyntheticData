//! Inventory generator.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::subledger::inventory::{
    InventoryMovement, InventoryPosition, MovementType, PositionValuation, ReferenceDocType,
    ValuationMethod,
};
use datasynth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for inventory generation.
#[derive(Debug, Clone)]
pub struct InventoryGeneratorConfig {
    /// Default valuation method.
    pub default_valuation_method: ValuationMethod,
    /// Average unit cost.
    pub avg_unit_cost: Decimal,
    /// Unit cost variation.
    pub cost_variation: Decimal,
    /// Average movement quantity.
    pub avg_movement_quantity: Decimal,
    /// Quantity variation.
    pub quantity_variation: Decimal,
}

impl Default for InventoryGeneratorConfig {
    fn default() -> Self {
        Self {
            default_valuation_method: ValuationMethod::MovingAverage,
            avg_unit_cost: dec!(100),
            cost_variation: dec!(0.5),
            avg_movement_quantity: dec!(50),
            quantity_variation: dec!(0.8),
        }
    }
}

/// Generator for inventory transactions.
pub struct InventoryGenerator {
    config: InventoryGeneratorConfig,
    rng: ChaCha8Rng,
    movement_counter: u64,
    #[allow(dead_code)]
    position_counter: u64,
}

impl InventoryGenerator {
    /// Creates a new inventory generator.
    pub fn new(config: InventoryGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            movement_counter: 0,
            position_counter: 0,
        }
    }

    /// Generates an initial inventory position.
    pub fn generate_position(
        &mut self,
        company_code: &str,
        plant: &str,
        storage_location: &str,
        material_id: &str,
        description: &str,
        initial_quantity: Decimal,
        unit_cost: Option<Decimal>,
        _currency: &str,
    ) -> InventoryPosition {
        let cost = unit_cost.unwrap_or_else(|| self.generate_unit_cost());
        let total_value = (initial_quantity * cost).round_dp(2);

        let mut position = InventoryPosition::new(
            material_id.to_string(),
            description.to_string(),
            plant.to_string(),
            storage_location.to_string(),
            company_code.to_string(),
            "EA".to_string(),
        );

        position.quantity_on_hand = initial_quantity;
        position.quantity_available = initial_quantity;
        position.valuation = PositionValuation {
            method: self.config.default_valuation_method,
            standard_cost: cost,
            unit_cost: cost,
            total_value,
            price_variance: Decimal::ZERO,
            last_price_change: None,
        };
        position.min_stock = Some(dec!(10));
        position.max_stock = Some(dec!(1000));
        position.reorder_point = Some(dec!(50));

        position
    }

    /// Generates a goods receipt (inventory increase).
    pub fn generate_goods_receipt(
        &mut self,
        position: &InventoryPosition,
        receipt_date: NaiveDate,
        quantity: Decimal,
        unit_cost: Decimal,
        po_number: Option<&str>,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let document_number = format!("INVMV{:08}", self.movement_counter);
        let batch_number = format!("BATCH{:06}", self.rng.gen::<u32>() % 1000000);

        let mut movement = InventoryMovement::new(
            document_number,
            1, // item_number
            position.company_code.clone(),
            receipt_date,
            MovementType::GoodsReceipt,
            position.material_id.clone(),
            position.description.clone(),
            position.plant.clone(),
            position.storage_location.clone(),
            quantity,
            position.unit.clone(),
            unit_cost,
            "USD".to_string(),
            "SYSTEM".to_string(),
        );

        movement.batch_number = Some(batch_number);
        if let Some(po) = po_number {
            movement.reference_doc_type = Some(ReferenceDocType::PurchaseOrder);
            movement.reference_doc_number = Some(po.to_string());
        }
        movement.reason_code = Some("Goods Receipt from PO".to_string());

        let je = self.generate_goods_receipt_je(&movement);
        (movement, je)
    }

    /// Generates a goods issue (inventory decrease).
    pub fn generate_goods_issue(
        &mut self,
        position: &InventoryPosition,
        issue_date: NaiveDate,
        quantity: Decimal,
        cost_center: Option<&str>,
        production_order: Option<&str>,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let document_number = format!("INVMV{:08}", self.movement_counter);

        let unit_cost = position.valuation.unit_cost;

        let mut movement = InventoryMovement::new(
            document_number,
            1, // item_number
            position.company_code.clone(),
            issue_date,
            MovementType::GoodsIssue,
            position.material_id.clone(),
            position.description.clone(),
            position.plant.clone(),
            position.storage_location.clone(),
            quantity,
            position.unit.clone(),
            unit_cost,
            "USD".to_string(),
            "SYSTEM".to_string(),
        );

        movement.cost_center = cost_center.map(|s| s.to_string());
        if let Some(po) = production_order {
            movement.reference_doc_type = Some(ReferenceDocType::ProductionOrder);
            movement.reference_doc_number = Some(po.to_string());
        }
        movement.reason_code = Some("Goods Issue to Production".to_string());

        let je = self.generate_goods_issue_je(&movement);
        (movement, je)
    }

    /// Generates a stock transfer between locations.
    pub fn generate_transfer(
        &mut self,
        position: &InventoryPosition,
        transfer_date: NaiveDate,
        quantity: Decimal,
        to_plant: &str,
        to_storage_location: &str,
    ) -> (InventoryMovement, InventoryMovement, JournalEntry) {
        // Issue from source
        self.movement_counter += 1;
        let issue_id = format!("INVMV{:08}", self.movement_counter);

        // Receipt at destination
        self.movement_counter += 1;
        let receipt_id = format!("INVMV{:08}", self.movement_counter);

        let unit_cost = position.valuation.unit_cost;

        let mut issue = InventoryMovement::new(
            issue_id,
            1, // item_number
            position.company_code.clone(),
            transfer_date,
            MovementType::TransferOut,
            position.material_id.clone(),
            position.description.clone(),
            position.plant.clone(),
            position.storage_location.clone(),
            quantity,
            position.unit.clone(),
            unit_cost,
            "USD".to_string(),
            "SYSTEM".to_string(),
        );
        issue.reference_doc_type = Some(ReferenceDocType::MaterialDocument);
        issue.reference_doc_number = Some(receipt_id.clone());
        issue.reason_code = Some(format!("Transfer to {}/{}", to_plant, to_storage_location));

        let mut receipt = InventoryMovement::new(
            receipt_id,
            1, // item_number
            position.company_code.clone(),
            transfer_date,
            MovementType::TransferIn,
            position.material_id.clone(),
            position.description.clone(),
            to_plant.to_string(),
            to_storage_location.to_string(),
            quantity,
            position.unit.clone(),
            unit_cost,
            "USD".to_string(),
            "SYSTEM".to_string(),
        );
        receipt.reference_doc_type = Some(ReferenceDocType::MaterialDocument);
        receipt.reference_doc_number = Some(issue.document_number.clone());
        receipt.reason_code = Some(format!(
            "Transfer from {}/{}",
            position.plant, position.storage_location
        ));

        // For intra-company transfer, no GL impact unless different plants have different valuations
        let je = self.generate_transfer_je(&issue, &receipt);

        (issue, receipt, je)
    }

    /// Generates an inventory adjustment.
    pub fn generate_adjustment(
        &mut self,
        position: &InventoryPosition,
        adjustment_date: NaiveDate,
        quantity_change: Decimal,
        reason: &str,
    ) -> (InventoryMovement, JournalEntry) {
        self.movement_counter += 1;
        let document_number = format!("INVMV{:08}", self.movement_counter);

        let movement_type = if quantity_change > Decimal::ZERO {
            MovementType::InventoryAdjustmentIn
        } else {
            MovementType::InventoryAdjustmentOut
        };

        let unit_cost = position.valuation.unit_cost;

        let mut movement = InventoryMovement::new(
            document_number,
            1, // item_number
            position.company_code.clone(),
            adjustment_date,
            movement_type,
            position.material_id.clone(),
            position.description.clone(),
            position.plant.clone(),
            position.storage_location.clone(),
            quantity_change.abs(),
            position.unit.clone(),
            unit_cost,
            "USD".to_string(),
            "SYSTEM".to_string(),
        );
        movement.reference_doc_type = Some(ReferenceDocType::PhysicalInventoryDoc);
        movement.reference_doc_number = Some(format!("PI{:08}", self.movement_counter));
        movement.reason_code = Some(reason.to_string());

        let je = self.generate_adjustment_je(&movement, quantity_change > Decimal::ZERO);
        (movement, je)
    }

    fn generate_unit_cost(&mut self) -> Decimal {
        let base = self.config.avg_unit_cost;
        let variation = base * self.config.cost_variation;
        let random: f64 = self.rng.gen_range(-1.0..1.0);
        (base + variation * Decimal::try_from(random).unwrap_or_default())
            .max(dec!(1))
            .round_dp(2)
    }

    fn generate_goods_receipt_je(&self, movement: &InventoryMovement) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", movement.document_number),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Goods Receipt {}", movement.material_id),
        );

        // Debit Inventory
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: "1300".to_string(),
            debit_amount: movement.value,
            cost_center: movement.cost_center.clone(),
            profit_center: None,
            reference: Some(movement.document_number.clone()),
            assignment: Some(movement.material_id.clone()),
            text: Some(movement.description.clone()),
            quantity: Some(movement.quantity),
            unit: Some(movement.unit.clone()),
            ..Default::default()
        });

        // Credit GR/IR Clearing
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "2100".to_string(),
            credit_amount: movement.value,
            reference: movement.reference_doc_number.clone(),
            ..Default::default()
        });

        je
    }

    fn generate_goods_issue_je(&self, movement: &InventoryMovement) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", movement.document_number),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Goods Issue {}", movement.material_id),
        );

        // Debit Cost of Goods Sold or WIP
        let debit_account =
            if movement.reference_doc_type == Some(ReferenceDocType::ProductionOrder) {
                "1350".to_string() // WIP
            } else {
                "5100".to_string() // COGS
            };

        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: debit_account,
            debit_amount: movement.value,
            cost_center: movement.cost_center.clone(),
            profit_center: None,
            reference: Some(movement.document_number.clone()),
            assignment: Some(movement.material_id.clone()),
            text: Some(movement.description.clone()),
            quantity: Some(movement.quantity),
            unit: Some(movement.unit.clone()),
            ..Default::default()
        });

        // Credit Inventory
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1300".to_string(),
            credit_amount: movement.value,
            reference: Some(movement.document_number.clone()),
            assignment: Some(movement.material_id.clone()),
            quantity: Some(movement.quantity),
            unit: Some(movement.unit.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_transfer_je(
        &self,
        issue: &InventoryMovement,
        _receipt: &InventoryMovement,
    ) -> JournalEntry {
        // For intra-company transfer with same valuation, this might be a memo entry
        // or could involve plant-specific inventory accounts
        let mut je = JournalEntry::new_simple(
            format!("JE-XFER-{}", issue.document_number),
            issue.company_code.clone(),
            issue.posting_date,
            format!("Stock Transfer {}", issue.material_id),
        );

        // Debit Inventory at destination (using same account for simplicity)
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: "1300".to_string(),
            debit_amount: issue.value,
            reference: Some(issue.document_number.clone()),
            assignment: Some(issue.material_id.clone()),
            quantity: Some(issue.quantity),
            unit: Some(issue.unit.clone()),
            ..Default::default()
        });

        // Credit Inventory at source
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1300".to_string(),
            credit_amount: issue.value,
            reference: Some(issue.document_number.clone()),
            assignment: Some(issue.material_id.clone()),
            quantity: Some(issue.quantity),
            unit: Some(issue.unit.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_adjustment_je(
        &self,
        movement: &InventoryMovement,
        is_increase: bool,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", movement.document_number),
            movement.company_code.clone(),
            movement.posting_date,
            format!("Inventory Adjustment {}", movement.material_id),
        );

        if is_increase {
            // Debit Inventory
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: "1300".to_string(),
                debit_amount: movement.value,
                reference: Some(movement.document_number.clone()),
                assignment: Some(movement.material_id.clone()),
                text: Some(movement.reason_code.clone().unwrap_or_default()),
                quantity: Some(movement.quantity),
                unit: Some(movement.unit.clone()),
                ..Default::default()
            });

            // Credit Inventory Adjustment Account
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: "4950".to_string(),
                credit_amount: movement.value,
                cost_center: movement.cost_center.clone(),
                reference: Some(movement.document_number.clone()),
                ..Default::default()
            });
        } else {
            // Debit Inventory Adjustment Account (expense)
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: "6950".to_string(),
                debit_amount: movement.value,
                cost_center: movement.cost_center.clone(),
                reference: Some(movement.document_number.clone()),
                text: Some(movement.reason_code.clone().unwrap_or_default()),
                ..Default::default()
            });

            // Credit Inventory
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: "1300".to_string(),
                credit_amount: movement.value,
                reference: Some(movement.document_number.clone()),
                assignment: Some(movement.material_id.clone()),
                quantity: Some(movement.quantity),
                unit: Some(movement.unit.clone()),
                ..Default::default()
            });
        }

        je
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generate_position() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            None,
            "USD",
        );

        assert_eq!(position.quantity_on_hand, dec!(100));
        assert!(position.valuation.unit_cost > Decimal::ZERO);
    }

    #[test]
    fn test_generate_goods_receipt() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            Some(dec!(50)),
            "USD",
        );

        let (movement, je) = generator.generate_goods_receipt(
            &position,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(50),
            dec!(50),
            Some("PO001"),
        );

        assert_eq!(movement.movement_type, MovementType::GoodsReceipt);
        assert_eq!(movement.quantity, dec!(50));
        assert!(je.is_balanced());
    }

    #[test]
    fn test_generate_goods_issue() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = InventoryGenerator::new(InventoryGeneratorConfig::default(), rng);

        let position = generator.generate_position(
            "1000",
            "PLANT01",
            "WH01",
            "MAT001",
            "Raw Material A",
            dec!(100),
            Some(dec!(50)),
            "USD",
        );

        let (movement, je) = generator.generate_goods_issue(
            &position,
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            dec!(30),
            Some("CC100"),
            None,
        );

        assert_eq!(movement.movement_type, MovementType::GoodsIssue);
        assert_eq!(movement.quantity, dec!(30));
        assert!(je.is_balanced());
    }
}
