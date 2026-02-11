//! Inventory movement model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::subledger::GLReference;

/// Inventory movement (goods receipt, issue, transfer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMovement {
    /// Movement document number.
    pub document_number: String,
    /// Movement item number.
    pub item_number: u32,
    /// Company code.
    pub company_code: String,
    /// Movement date.
    pub movement_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Movement type.
    pub movement_type: MovementType,
    /// Material ID.
    pub material_id: String,
    /// Material description.
    pub description: String,
    /// Plant.
    pub plant: String,
    /// Storage location.
    pub storage_location: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit of measure.
    pub unit: String,
    /// Movement value.
    pub value: Decimal,
    /// Currency.
    pub currency: String,
    /// Unit cost.
    pub unit_cost: Decimal,
    /// Batch number.
    pub batch_number: Option<String>,
    /// Serial numbers.
    pub serial_numbers: Vec<String>,
    /// Reference document type.
    pub reference_doc_type: Option<ReferenceDocType>,
    /// Reference document number.
    pub reference_doc_number: Option<String>,
    /// Reference item.
    pub reference_item: Option<u32>,
    /// Vendor (for receipts).
    pub vendor_id: Option<String>,
    /// Customer (for issues).
    pub customer_id: Option<String>,
    /// Cost center.
    pub cost_center: Option<String>,
    /// GL account.
    pub gl_account: String,
    /// Offset account.
    pub offset_account: String,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Special stock indicator.
    pub special_stock: Option<SpecialStockType>,
    /// Reason code.
    pub reason_code: Option<String>,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Reversed.
    pub is_reversed: bool,
    /// Reversal document.
    pub reversal_doc: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl InventoryMovement {
    /// Creates a new inventory movement.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_number: String,
        item_number: u32,
        company_code: String,
        movement_date: NaiveDate,
        movement_type: MovementType,
        material_id: String,
        description: String,
        plant: String,
        storage_location: String,
        quantity: Decimal,
        unit: String,
        unit_cost: Decimal,
        currency: String,
        created_by: String,
    ) -> Self {
        let value = quantity * unit_cost;
        let (gl_account, offset_account) = movement_type.default_accounts();

        Self {
            document_number,
            item_number,
            company_code,
            movement_date,
            posting_date: movement_date,
            movement_type,
            material_id,
            description,
            plant,
            storage_location,
            quantity,
            unit,
            value,
            currency,
            unit_cost,
            batch_number: None,
            serial_numbers: Vec::new(),
            reference_doc_type: None,
            reference_doc_number: None,
            reference_item: None,
            vendor_id: None,
            customer_id: None,
            cost_center: None,
            gl_account,
            offset_account,
            gl_reference: None,
            special_stock: None,
            reason_code: None,
            created_by,
            created_at: Utc::now(),
            is_reversed: false,
            reversal_doc: None,
            notes: None,
        }
    }

    /// Creates a goods receipt from purchase order.
    #[allow(clippy::too_many_arguments)]
    pub fn goods_receipt_po(
        document_number: String,
        item_number: u32,
        company_code: String,
        movement_date: NaiveDate,
        material_id: String,
        description: String,
        plant: String,
        storage_location: String,
        quantity: Decimal,
        unit: String,
        unit_cost: Decimal,
        currency: String,
        po_number: String,
        po_item: u32,
        vendor_id: String,
        created_by: String,
    ) -> Self {
        let mut movement = Self::new(
            document_number,
            item_number,
            company_code,
            movement_date,
            MovementType::GoodsReceiptPO,
            material_id,
            description,
            plant,
            storage_location,
            quantity,
            unit,
            unit_cost,
            currency,
            created_by,
        );

        movement.reference_doc_type = Some(ReferenceDocType::PurchaseOrder);
        movement.reference_doc_number = Some(po_number);
        movement.reference_item = Some(po_item);
        movement.vendor_id = Some(vendor_id);
        movement
    }

    /// Creates a goods issue to sales order.
    #[allow(clippy::too_many_arguments)]
    pub fn goods_issue_sales(
        document_number: String,
        item_number: u32,
        company_code: String,
        movement_date: NaiveDate,
        material_id: String,
        description: String,
        plant: String,
        storage_location: String,
        quantity: Decimal,
        unit: String,
        unit_cost: Decimal,
        currency: String,
        sales_order: String,
        sales_item: u32,
        customer_id: String,
        created_by: String,
    ) -> Self {
        let mut movement = Self::new(
            document_number,
            item_number,
            company_code,
            movement_date,
            MovementType::GoodsIssueSales,
            material_id,
            description,
            plant,
            storage_location,
            quantity,
            unit,
            unit_cost,
            currency,
            created_by,
        );

        movement.reference_doc_type = Some(ReferenceDocType::SalesOrder);
        movement.reference_doc_number = Some(sales_order);
        movement.reference_item = Some(sales_item);
        movement.customer_id = Some(customer_id);
        movement
    }

    /// Sets batch number.
    pub fn with_batch(mut self, batch_number: String) -> Self {
        self.batch_number = Some(batch_number);
        self
    }

    /// Sets serial numbers.
    pub fn with_serials(mut self, serial_numbers: Vec<String>) -> Self {
        self.serial_numbers = serial_numbers;
        self
    }

    /// Sets cost center.
    pub fn with_cost_center(mut self, cost_center: String) -> Self {
        self.cost_center = Some(cost_center);
        self
    }

    /// Sets reason code.
    pub fn with_reason(mut self, reason_code: String) -> Self {
        self.reason_code = Some(reason_code);
        self
    }

    /// Sets GL reference.
    pub fn with_gl_reference(mut self, reference: GLReference) -> Self {
        self.gl_reference = Some(reference);
        self
    }

    /// Marks as reversed.
    pub fn reverse(&mut self, reversal_doc: String) {
        self.is_reversed = true;
        self.reversal_doc = Some(reversal_doc);
    }

    /// Creates a reversal movement.
    pub fn create_reversal(&self, reversal_doc_number: String, created_by: String) -> Self {
        let mut reversal = Self::new(
            reversal_doc_number,
            self.item_number,
            self.company_code.clone(),
            chrono::Local::now().date_naive(),
            self.movement_type.reversal_type(),
            self.material_id.clone(),
            self.description.clone(),
            self.plant.clone(),
            self.storage_location.clone(),
            self.quantity,
            self.unit.clone(),
            self.unit_cost,
            self.currency.clone(),
            created_by,
        );

        reversal.reference_doc_type = Some(ReferenceDocType::MaterialDocument);
        reversal.reference_doc_number = Some(self.document_number.clone());
        reversal.reference_item = Some(self.item_number);
        reversal.batch_number = self.batch_number.clone();
        reversal.notes = Some(format!(
            "Reversal of {}/{}",
            self.document_number, self.item_number
        ));
        reversal
    }

    /// Gets sign for quantity (positive or negative).
    pub fn quantity_sign(&self) -> i8 {
        self.movement_type.quantity_sign()
    }

    /// Gets signed quantity.
    pub fn signed_quantity(&self) -> Decimal {
        self.quantity * Decimal::from(self.quantity_sign())
    }
}

/// Movement type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    /// Goods receipt from purchase order.
    GoodsReceiptPO,
    /// Goods receipt from production.
    GoodsReceiptProduction,
    /// Goods receipt without reference.
    GoodsReceiptOther,
    /// Goods receipt (generic).
    GoodsReceipt,
    /// Return to vendor.
    ReturnToVendor,
    /// Goods issue for sales order.
    GoodsIssueSales,
    /// Goods issue for production.
    GoodsIssueProduction,
    /// Goods issue for cost center.
    GoodsIssueCostCenter,
    /// Goods issue scrapping.
    GoodsIssueScrapping,
    /// Goods issue (generic).
    GoodsIssue,
    /// Scrap (alias for GoodsIssueScrapping).
    Scrap,
    /// Transfer posting between plants.
    TransferPlant,
    /// Transfer posting between storage locations.
    TransferStorageLocation,
    /// Transfer in.
    TransferIn,
    /// Transfer out.
    TransferOut,
    /// Transfer to quality inspection.
    TransferToInspection,
    /// Transfer from quality inspection.
    TransferFromInspection,
    /// Physical inventory difference.
    PhysicalInventory,
    /// Inventory adjustment in.
    InventoryAdjustmentIn,
    /// Inventory adjustment out.
    InventoryAdjustmentOut,
    /// Initial stock entry.
    InitialStock,
    /// Reversal of goods receipt.
    ReversalGoodsReceipt,
    /// Reversal of goods issue.
    ReversalGoodsIssue,
}

impl MovementType {
    /// Gets quantity sign (1 for receipts, -1 for issues).
    pub fn quantity_sign(&self) -> i8 {
        match self {
            MovementType::GoodsReceiptPO
            | MovementType::GoodsReceiptProduction
            | MovementType::GoodsReceiptOther
            | MovementType::GoodsReceipt
            | MovementType::TransferFromInspection
            | MovementType::TransferIn
            | MovementType::InventoryAdjustmentIn
            | MovementType::InitialStock
            | MovementType::ReversalGoodsIssue => 1,

            MovementType::ReturnToVendor
            | MovementType::GoodsIssueSales
            | MovementType::GoodsIssueProduction
            | MovementType::GoodsIssueCostCenter
            | MovementType::GoodsIssueScrapping
            | MovementType::GoodsIssue
            | MovementType::Scrap
            | MovementType::TransferOut
            | MovementType::InventoryAdjustmentOut
            | MovementType::TransferToInspection
            | MovementType::ReversalGoodsReceipt => -1,

            MovementType::TransferPlant
            | MovementType::TransferStorageLocation
            | MovementType::PhysicalInventory => 0, // Neutral or depends on context
        }
    }

    /// Gets default GL accounts.
    pub fn default_accounts(&self) -> (String, String) {
        match self {
            MovementType::GoodsReceiptPO => ("1200".to_string(), "2100".to_string()), // Inventory, GR/IR
            MovementType::GoodsReceiptProduction => ("1200".to_string(), "1300".to_string()), // Inventory, WIP
            MovementType::GoodsReceiptOther => ("1200".to_string(), "1299".to_string()), // Inventory, Clearing
            MovementType::GoodsReceipt => ("1200".to_string(), "1299".to_string()), // Generic receipt
            MovementType::ReturnToVendor => ("2100".to_string(), "1200".to_string()), // GR/IR, Inventory
            MovementType::GoodsIssueSales => ("5000".to_string(), "1200".to_string()), // COGS, Inventory
            MovementType::GoodsIssueProduction => ("1300".to_string(), "1200".to_string()), // WIP, Inventory
            MovementType::GoodsIssueCostCenter => ("7000".to_string(), "1200".to_string()), // Expense, Inventory
            MovementType::GoodsIssueScrapping => ("7900".to_string(), "1200".to_string()), // Loss, Inventory
            MovementType::GoodsIssue => ("7000".to_string(), "1200".to_string()), // Generic issue
            MovementType::Scrap => ("7900".to_string(), "1200".to_string()), // Loss, Inventory (alias)
            MovementType::TransferPlant => ("1200".to_string(), "1200".to_string()), // Inventory to Inventory
            MovementType::TransferStorageLocation => ("1200".to_string(), "1200".to_string()),
            MovementType::TransferIn => ("1200".to_string(), "1299".to_string()), // Inventory in
            MovementType::TransferOut => ("1299".to_string(), "1200".to_string()), // Inventory out
            MovementType::TransferToInspection => ("1210".to_string(), "1200".to_string()),
            MovementType::TransferFromInspection => ("1200".to_string(), "1210".to_string()),
            MovementType::PhysicalInventory => ("7910".to_string(), "1200".to_string()), // Gain/Loss, Inventory
            MovementType::InventoryAdjustmentIn => ("1200".to_string(), "7910".to_string()), // Inventory, Gain
            MovementType::InventoryAdjustmentOut => ("7910".to_string(), "1200".to_string()), // Loss, Inventory
            MovementType::InitialStock => ("1200".to_string(), "3000".to_string()), // Inventory, Equity
            MovementType::ReversalGoodsReceipt => ("2100".to_string(), "1200".to_string()),
            MovementType::ReversalGoodsIssue => ("1200".to_string(), "5000".to_string()),
        }
    }

    /// Gets the reversal movement type.
    pub fn reversal_type(&self) -> MovementType {
        match self {
            MovementType::GoodsReceiptPO
            | MovementType::GoodsReceiptProduction
            | MovementType::GoodsReceiptOther
            | MovementType::GoodsReceipt
            | MovementType::TransferIn
            | MovementType::InventoryAdjustmentIn => MovementType::ReversalGoodsReceipt,

            MovementType::GoodsIssueSales
            | MovementType::GoodsIssueProduction
            | MovementType::GoodsIssueCostCenter
            | MovementType::GoodsIssue
            | MovementType::Scrap
            | MovementType::TransferOut
            | MovementType::InventoryAdjustmentOut
            | MovementType::GoodsIssueScrapping => MovementType::ReversalGoodsIssue,

            _ => *self, // Others reverse themselves
        }
    }
}

/// Reference document type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceDocType {
    /// Purchase order.
    PurchaseOrder,
    /// Sales order.
    SalesOrder,
    /// Production order.
    ProductionOrder,
    /// Delivery.
    Delivery,
    /// Material document.
    MaterialDocument,
    /// Reservation.
    Reservation,
    /// Physical inventory document.
    PhysicalInventoryDoc,
}

/// Special stock type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialStockType {
    /// Consignment from vendor.
    VendorConsignment,
    /// Consignment at customer.
    CustomerConsignment,
    /// Project stock.
    ProjectStock,
    /// Sales order stock.
    SalesOrderStock,
    /// Subcontracting stock.
    Subcontracting,
}

/// Stock transfer between locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTransfer {
    /// Transfer document number.
    pub document_number: String,
    /// Company code.
    pub company_code: String,
    /// Transfer date.
    pub transfer_date: NaiveDate,
    /// From plant.
    pub from_plant: String,
    /// From storage location.
    pub from_storage_location: String,
    /// To plant.
    pub to_plant: String,
    /// To storage location.
    pub to_storage_location: String,
    /// Items.
    pub items: Vec<TransferItem>,
    /// Status.
    pub status: TransferStatus,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
}

/// Transfer item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    /// Item number.
    pub item_number: u32,
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit.
    pub unit: String,
    /// Batch number.
    pub batch_number: Option<String>,
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Draft.
    Draft,
    /// In transit.
    InTransit,
    /// Partially received.
    PartiallyReceived,
    /// Completed.
    Completed,
    /// Cancelled.
    Cancelled,
}

/// Physical inventory count document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalInventoryDoc {
    /// Document number.
    pub document_number: String,
    /// Company code.
    pub company_code: String,
    /// Plant.
    pub plant: String,
    /// Storage location.
    pub storage_location: String,
    /// Planned count date.
    pub planned_date: NaiveDate,
    /// Actual count date.
    pub count_date: Option<NaiveDate>,
    /// Status.
    pub status: PIStatus,
    /// Items.
    pub items: Vec<PIItem>,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Posted.
    pub posted: bool,
    /// Posted at.
    pub posted_at: Option<DateTime<Utc>>,
}

/// Physical inventory status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PIStatus {
    /// Created.
    Created,
    /// Active (counting in progress).
    Active,
    /// Counted.
    Counted,
    /// Posted.
    Posted,
    /// Cancelled.
    Cancelled,
}

/// Physical inventory item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIItem {
    /// Item number.
    pub item_number: u32,
    /// Material ID.
    pub material_id: String,
    /// Description.
    pub description: String,
    /// Book quantity.
    pub book_quantity: Decimal,
    /// Counted quantity.
    pub counted_quantity: Option<Decimal>,
    /// Difference.
    pub difference: Option<Decimal>,
    /// Unit.
    pub unit: String,
    /// Batch number.
    pub batch_number: Option<String>,
    /// Is zero count.
    pub zero_count: bool,
    /// Difference reason.
    pub difference_reason: Option<String>,
}

impl PIItem {
    /// Calculates difference.
    pub fn calculate_difference(&mut self) {
        if let Some(counted) = self.counted_quantity {
            self.difference = Some(counted - self.book_quantity);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_goods_receipt_po() {
        let movement = InventoryMovement::goods_receipt_po(
            "MBLNR001".to_string(),
            1,
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "MAT001".to_string(),
            "Test Material".to_string(),
            "PLANT01".to_string(),
            "SLOC01".to_string(),
            dec!(100),
            "EA".to_string(),
            dec!(10),
            "USD".to_string(),
            "PO001".to_string(),
            10,
            "VEND001".to_string(),
            "USER1".to_string(),
        );

        assert_eq!(movement.movement_type, MovementType::GoodsReceiptPO);
        assert_eq!(movement.quantity, dec!(100));
        assert_eq!(movement.value, dec!(1000));
        assert_eq!(movement.quantity_sign(), 1);
    }

    #[test]
    fn test_goods_issue_sales() {
        let movement = InventoryMovement::goods_issue_sales(
            "MBLNR002".to_string(),
            1,
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "MAT001".to_string(),
            "Test Material".to_string(),
            "PLANT01".to_string(),
            "SLOC01".to_string(),
            dec!(50),
            "EA".to_string(),
            dec!(10),
            "USD".to_string(),
            "SO001".to_string(),
            10,
            "CUST001".to_string(),
            "USER1".to_string(),
        );

        assert_eq!(movement.movement_type, MovementType::GoodsIssueSales);
        assert_eq!(movement.quantity_sign(), -1);
        assert_eq!(movement.signed_quantity(), dec!(-50));
    }

    #[test]
    fn test_create_reversal() {
        let original = InventoryMovement::goods_receipt_po(
            "MBLNR001".to_string(),
            1,
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "MAT001".to_string(),
            "Test Material".to_string(),
            "PLANT01".to_string(),
            "SLOC01".to_string(),
            dec!(100),
            "EA".to_string(),
            dec!(10),
            "USD".to_string(),
            "PO001".to_string(),
            10,
            "VEND001".to_string(),
            "USER1".to_string(),
        );

        let reversal = original.create_reversal("MBLNR002".to_string(), "USER2".to_string());

        assert_eq!(reversal.movement_type, MovementType::ReversalGoodsReceipt);
        assert_eq!(reversal.reference_doc_number, Some("MBLNR001".to_string()));
    }
}
