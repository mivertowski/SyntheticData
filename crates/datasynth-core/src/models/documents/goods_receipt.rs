//! Goods Receipt document model.
//!
//! Represents goods receipts in the P2P (Procure-to-Pay) process flow.
//! Goods receipts create accounting entries: DR Inventory, CR GR/IR Clearing.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DocumentHeader, DocumentLineItem, DocumentReference, DocumentStatus, DocumentType,
    ReferenceType,
};

/// Goods Receipt type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GoodsReceiptType {
    /// Standard goods receipt against PO
    #[default]
    PurchaseOrder,
    /// Return to vendor
    ReturnToVendor,
    /// Stock transfer receipt
    StockTransfer,
    /// Production receipt
    Production,
    /// Initial stock entry
    InitialStock,
    /// Subcontracting receipt
    Subcontracting,
}

/// Movement type for inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    /// GR for PO (101)
    #[default]
    GrForPo,
    /// Return to vendor (122)
    ReturnToVendor,
    /// GR for production order (131)
    GrForProduction,
    /// Transfer posting (301)
    TransferPosting,
    /// Initial stock entry (561)
    InitialEntry,
    /// Scrapping (551)
    Scrapping,
    /// Consumption (201)
    Consumption,
}

impl MovementType {
    /// Get the SAP movement type code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::GrForPo => "101",
            Self::ReturnToVendor => "122",
            Self::GrForProduction => "131",
            Self::TransferPosting => "301",
            Self::InitialEntry => "561",
            Self::Scrapping => "551",
            Self::Consumption => "201",
        }
    }

    /// Check if this movement increases inventory.
    pub fn is_receipt(&self) -> bool {
        matches!(
            self,
            Self::GrForPo | Self::GrForProduction | Self::InitialEntry | Self::TransferPosting
        )
    }

    /// Check if this movement decreases inventory.
    pub fn is_issue(&self) -> bool {
        matches!(
            self,
            Self::ReturnToVendor | Self::Scrapping | Self::Consumption
        )
    }
}

/// Goods Receipt line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Movement type
    pub movement_type: MovementType,

    /// Reference PO number
    pub po_number: Option<String>,

    /// Reference PO item
    pub po_item: Option<u16>,

    /// Batch number (if batch managed)
    pub batch: Option<String>,

    /// Serial numbers (if serial managed)
    pub serial_numbers: Vec<String>,

    /// Vendor batch
    pub vendor_batch: Option<String>,

    /// Quantity in base UOM
    pub quantity_base_uom: Decimal,

    /// Valuation type
    pub valuation_type: Option<String>,

    /// Stock type (unrestricted, quality inspection, blocked)
    pub stock_type: StockType,

    /// Reason for movement (for returns, adjustments)
    pub reason_for_movement: Option<String>,

    /// Delivery note reference
    pub delivery_note: Option<String>,

    /// Bill of lading
    pub bill_of_lading: Option<String>,
}

/// Stock type for goods receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StockType {
    /// Unrestricted use stock
    #[default]
    Unrestricted,
    /// Quality inspection stock
    QualityInspection,
    /// Blocked stock
    Blocked,
    /// Returns stock
    Returns,
}

impl GoodsReceiptItem {
    /// Create a new goods receipt item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        Self {
            base: DocumentLineItem::new(line_number, description, quantity, unit_price),
            movement_type: MovementType::GrForPo,
            po_number: None,
            po_item: None,
            batch: None,
            serial_numbers: Vec::new(),
            vendor_batch: None,
            quantity_base_uom: quantity,
            valuation_type: None,
            stock_type: StockType::Unrestricted,
            reason_for_movement: None,
            delivery_note: None,
            bill_of_lading: None,
        }
    }

    /// Create from PO reference.
    pub fn from_po(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        po_number: impl Into<String>,
        po_item: u16,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price);
        item.po_number = Some(po_number.into());
        item.po_item = Some(po_item);
        item
    }

    /// Set material.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.base = self.base.with_material(material_id);
        self
    }

    /// Set batch.
    pub fn with_batch(mut self, batch: impl Into<String>) -> Self {
        self.batch = Some(batch.into());
        self
    }

    /// Set stock type.
    pub fn with_stock_type(mut self, stock_type: StockType) -> Self {
        self.stock_type = stock_type;
        self
    }

    /// Set movement type.
    pub fn with_movement_type(mut self, movement_type: MovementType) -> Self {
        self.movement_type = movement_type;
        self
    }

    /// Set plant and storage location.
    pub fn with_location(
        mut self,
        plant: impl Into<String>,
        storage_location: impl Into<String>,
    ) -> Self {
        self.base.plant = Some(plant.into());
        self.base.storage_location = Some(storage_location.into());
        self
    }

    /// Set delivery note.
    pub fn with_delivery_note(mut self, note: impl Into<String>) -> Self {
        self.delivery_note = Some(note.into());
        self
    }

    /// Add serial number.
    pub fn add_serial_number(&mut self, serial: impl Into<String>) {
        self.serial_numbers.push(serial.into());
    }
}

/// Goods Receipt document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceipt {
    /// Document header
    pub header: DocumentHeader,

    /// GR type
    pub gr_type: GoodsReceiptType,

    /// Line items
    pub items: Vec<GoodsReceiptItem>,

    /// Total quantity received
    pub total_quantity: Decimal,

    /// Total value
    pub total_value: Decimal,

    /// Reference PO (primary)
    pub purchase_order_id: Option<String>,

    /// Vendor (for info)
    pub vendor_id: Option<String>,

    /// Bill of lading
    pub bill_of_lading: Option<String>,

    /// Delivery note from vendor
    pub delivery_note: Option<String>,

    /// Receiving plant
    pub plant: String,

    /// Receiving storage location
    pub storage_location: String,

    /// Material document year
    pub material_doc_year: u16,

    /// Is this GR posted?
    pub is_posted: bool,

    /// Is this GR cancelled/reversed?
    pub is_cancelled: bool,

    /// Cancellation GR reference
    pub cancellation_doc: Option<String>,
}

impl GoodsReceipt {
    /// Create a new goods receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gr_id: impl Into<String>,
        company_code: impl Into<String>,
        plant: impl Into<String>,
        storage_location: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            gr_id,
            DocumentType::GoodsReceipt,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );

        Self {
            header,
            gr_type: GoodsReceiptType::PurchaseOrder,
            items: Vec::new(),
            total_quantity: Decimal::ZERO,
            total_value: Decimal::ZERO,
            purchase_order_id: None,
            vendor_id: None,
            bill_of_lading: None,
            delivery_note: None,
            plant: plant.into(),
            storage_location: storage_location.into(),
            material_doc_year: fiscal_year,
            is_posted: false,
            is_cancelled: false,
            cancellation_doc: None,
        }
    }

    /// Create from PO reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_purchase_order(
        gr_id: impl Into<String>,
        company_code: impl Into<String>,
        purchase_order_id: impl Into<String>,
        vendor_id: impl Into<String>,
        plant: impl Into<String>,
        storage_location: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let po_id = purchase_order_id.into();
        let mut gr = Self::new(
            gr_id,
            company_code,
            plant,
            storage_location,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );
        gr.purchase_order_id = Some(po_id.clone());
        gr.vendor_id = Some(vendor_id.into());

        // Add reference to PO
        gr.header.add_reference(DocumentReference::new(
            DocumentType::PurchaseOrder,
            po_id,
            DocumentType::GoodsReceipt,
            gr.header.document_id.clone(),
            ReferenceType::FollowOn,
            gr.header.company_code.clone(),
            document_date,
        ));

        gr
    }

    /// Set GR type.
    pub fn with_gr_type(mut self, gr_type: GoodsReceiptType) -> Self {
        self.gr_type = gr_type;
        self
    }

    /// Set delivery note.
    pub fn with_delivery_note(mut self, note: impl Into<String>) -> Self {
        self.delivery_note = Some(note.into());
        self
    }

    /// Set bill of lading.
    pub fn with_bill_of_lading(mut self, bol: impl Into<String>) -> Self {
        self.bill_of_lading = Some(bol.into());
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, mut item: GoodsReceiptItem) {
        item.base.plant = Some(self.plant.clone());
        item.base.storage_location = Some(self.storage_location.clone());
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.total_quantity = self.items.iter().map(|i| i.base.quantity).sum();
        self.total_value = self.items.iter().map(|i| i.base.net_amount).sum();
    }

    /// Post the GR and generate GL entry.
    pub fn post(&mut self, user: impl Into<String>, posting_date: NaiveDate) {
        self.header.posting_date = Some(posting_date);
        self.header.update_status(DocumentStatus::Posted, user);
        self.is_posted = true;
    }

    /// Cancel/reverse the GR.
    pub fn cancel(&mut self, user: impl Into<String>, cancellation_doc: impl Into<String>) {
        self.is_cancelled = true;
        self.cancellation_doc = Some(cancellation_doc.into());
        self.header.update_status(DocumentStatus::Cancelled, user);
    }

    /// Generate the GL journal entry for this GR.
    /// DR Inventory (or GR/IR for services)
    /// CR GR/IR Clearing
    pub fn generate_gl_entries(&self) -> Vec<(String, Decimal, Decimal)> {
        let mut entries = Vec::new();

        for item in &self.items {
            if item.movement_type.is_receipt() {
                // Debit: Inventory or Expense account
                let debit_account = item
                    .base
                    .gl_account
                    .clone()
                    .unwrap_or_else(|| "140000".to_string()); // Default inventory

                // Credit: GR/IR Clearing
                let credit_account = "290000".to_string();

                entries.push((debit_account, item.base.net_amount, Decimal::ZERO));
                entries.push((credit_account, Decimal::ZERO, item.base.net_amount));
            }
        }

        entries
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_goods_receipt_creation() {
        let gr = GoodsReceipt::new(
            "GR-1000-0000000001",
            "1000",
            "1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(gr.plant, "1000");
        assert_eq!(gr.header.status, DocumentStatus::Draft);
    }

    #[test]
    fn test_goods_receipt_from_po() {
        let gr = GoodsReceipt::from_purchase_order(
            "GR-1000-0000000001",
            "1000",
            "PO-1000-0000000001",
            "V-000001",
            "1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(gr.purchase_order_id, Some("PO-1000-0000000001".to_string()));
        assert_eq!(gr.header.document_references.len(), 1);
    }

    #[test]
    fn test_goods_receipt_items() {
        let mut gr = GoodsReceipt::new(
            "GR-1000-0000000001",
            "1000",
            "1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        gr.add_item(
            GoodsReceiptItem::from_po(
                1,
                "Raw Materials",
                Decimal::from(100),
                Decimal::from(10),
                "PO-1000-0000000001",
                1,
            )
            .with_material("MAT-001"),
        );

        assert_eq!(gr.total_quantity, Decimal::from(100));
        assert_eq!(gr.total_value, Decimal::from(1000));
    }

    #[test]
    fn test_movement_types() {
        assert!(MovementType::GrForPo.is_receipt());
        assert!(!MovementType::GrForPo.is_issue());
        assert!(MovementType::ReturnToVendor.is_issue());
        assert_eq!(MovementType::GrForPo.code(), "101");
    }

    #[test]
    fn test_gl_entry_generation() {
        let mut gr = GoodsReceipt::new(
            "GR-1000-0000000001",
            "1000",
            "1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        gr.add_item(GoodsReceiptItem::new(
            1,
            "Test Item",
            Decimal::from(10),
            Decimal::from(100),
        ));

        let entries = gr.generate_gl_entries();
        assert_eq!(entries.len(), 2);
        // First entry: DR Inventory
        assert_eq!(entries[0].1, Decimal::from(1000));
        // Second entry: CR GR/IR
        assert_eq!(entries[1].2, Decimal::from(1000));
    }
}
