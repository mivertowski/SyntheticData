//! Purchase Order document model.
//!
//! Represents purchase orders in the P2P (Procure-to-Pay) process flow.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{DocumentHeader, DocumentLineItem, DocumentStatus, DocumentType};

/// Purchase Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseOrderType {
    /// Standard purchase order for goods
    #[default]
    Standard,
    /// Service purchase order
    Service,
    /// Framework/blanket order
    Framework,
    /// Consignment order
    Consignment,
    /// Stock transfer order
    StockTransfer,
    /// Subcontracting order
    Subcontracting,
}

impl PurchaseOrderType {
    /// Check if this PO type requires goods receipt.
    pub fn requires_goods_receipt(&self) -> bool {
        !matches!(self, Self::Service)
    }

    /// Check if this is an internal order (stock transfer).
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::StockTransfer)
    }
}

/// Purchase Order line item with P2P specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Item category (goods, service, etc.)
    pub item_category: String,

    /// Purchasing group
    pub purchasing_group: Option<String>,

    /// Goods receipt indicator
    pub gr_indicator: bool,

    /// Invoice receipt indicator
    pub ir_indicator: bool,

    /// GR-based invoice verification
    pub gr_based_iv: bool,

    /// Quantity received so far
    pub quantity_received: Decimal,

    /// Quantity invoiced so far
    pub quantity_invoiced: Decimal,

    /// Quantity returned
    pub quantity_returned: Decimal,

    /// Is this line fully received?
    pub is_fully_received: bool,

    /// Is this line fully invoiced?
    pub is_fully_invoiced: bool,

    /// Requested delivery date
    pub requested_date: Option<NaiveDate>,

    /// Confirmed delivery date
    pub confirmed_date: Option<NaiveDate>,

    /// Incoterms
    pub incoterms: Option<String>,

    /// Account assignment category (cost center, asset, etc.)
    pub account_assignment_category: String,
}

impl PurchaseOrderItem {
    /// Create a new purchase order item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        Self {
            base: DocumentLineItem::new(line_number, description, quantity, unit_price),
            item_category: "GOODS".to_string(),
            purchasing_group: None,
            gr_indicator: true,
            ir_indicator: true,
            gr_based_iv: true,
            quantity_received: Decimal::ZERO,
            quantity_invoiced: Decimal::ZERO,
            quantity_returned: Decimal::ZERO,
            is_fully_received: false,
            is_fully_invoiced: false,
            requested_date: None,
            confirmed_date: None,
            incoterms: None,
            account_assignment_category: "K".to_string(), // Cost center
        }
    }

    /// Create a service line item.
    pub fn service(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price);
        item.item_category = "SERVICE".to_string();
        item.gr_indicator = false;
        item.gr_based_iv = false;
        item.base.uom = "HR".to_string();
        item
    }

    /// Set material.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.base = self.base.with_material(material_id);
        self
    }

    /// Set cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.base = self.base.with_cost_center(cost_center);
        self
    }

    /// Set GL account.
    pub fn with_gl_account(mut self, account: impl Into<String>) -> Self {
        self.base = self.base.with_gl_account(account);
        self
    }

    /// Set requested delivery date.
    pub fn with_requested_date(mut self, date: NaiveDate) -> Self {
        self.requested_date = Some(date);
        self.base = self.base.with_delivery_date(date);
        self
    }

    /// Set purchasing group.
    pub fn with_purchasing_group(mut self, group: impl Into<String>) -> Self {
        self.purchasing_group = Some(group.into());
        self
    }

    /// Record goods receipt.
    pub fn record_goods_receipt(&mut self, quantity: Decimal) {
        self.quantity_received += quantity;
        if self.quantity_received >= self.base.quantity {
            self.is_fully_received = true;
        }
    }

    /// Record invoice receipt.
    pub fn record_invoice(&mut self, quantity: Decimal) {
        self.quantity_invoiced += quantity;
        if self.quantity_invoiced >= self.base.quantity {
            self.is_fully_invoiced = true;
        }
    }

    /// Get open quantity for receipt.
    pub fn open_quantity_gr(&self) -> Decimal {
        (self.base.quantity - self.quantity_received - self.quantity_returned).max(Decimal::ZERO)
    }

    /// Get open quantity for invoice.
    pub fn open_quantity_iv(&self) -> Decimal {
        if self.gr_based_iv {
            (self.quantity_received - self.quantity_invoiced).max(Decimal::ZERO)
        } else {
            (self.base.quantity - self.quantity_invoiced).max(Decimal::ZERO)
        }
    }

    /// Get open amount for invoice.
    pub fn open_amount_iv(&self) -> Decimal {
        self.open_quantity_iv() * self.base.unit_price
    }
}

/// Purchase Order document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    /// Document header
    pub header: DocumentHeader,

    /// PO type
    pub po_type: PurchaseOrderType,

    /// Vendor ID
    pub vendor_id: String,

    /// Purchasing organization
    pub purchasing_org: String,

    /// Purchasing group
    pub purchasing_group: String,

    /// Payment terms
    pub payment_terms: String,

    /// Incoterms
    pub incoterms: Option<String>,

    /// Incoterms location
    pub incoterms_location: Option<String>,

    /// Line items
    pub items: Vec<PurchaseOrderItem>,

    /// Total net amount
    pub total_net_amount: Decimal,

    /// Total tax amount
    pub total_tax_amount: Decimal,

    /// Total gross amount
    pub total_gross_amount: Decimal,

    /// Is this PO completely delivered?
    pub is_complete: bool,

    /// Is this PO closed?
    pub is_closed: bool,

    /// Related purchase requisition
    pub requisition_id: Option<String>,

    /// Contract reference
    pub contract_id: Option<String>,

    /// Release status (for framework orders)
    pub release_status: Option<String>,

    /// Output control - PO printed/sent
    pub output_complete: bool,

    /// Vendor display name (denormalized, DS-011)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<String>,
}

impl PurchaseOrder {
    /// Create a new purchase order.
    pub fn new(
        po_id: impl Into<String>,
        company_code: impl Into<String>,
        vendor_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            po_id,
            DocumentType::PurchaseOrder,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );

        Self {
            header,
            po_type: PurchaseOrderType::Standard,
            vendor_id: vendor_id.into(),
            purchasing_org: "1000".to_string(),
            purchasing_group: "001".to_string(),
            payment_terms: "NET30".to_string(),
            incoterms: None,
            incoterms_location: None,
            items: Vec::new(),
            total_net_amount: Decimal::ZERO,
            total_tax_amount: Decimal::ZERO,
            total_gross_amount: Decimal::ZERO,
            is_complete: false,
            is_closed: false,
            requisition_id: None,
            contract_id: None,
            release_status: None,
            output_complete: false,
            vendor_name: None,
        }
    }

    /// Set PO type.
    pub fn with_po_type(mut self, po_type: PurchaseOrderType) -> Self {
        self.po_type = po_type;
        self
    }

    /// Set purchasing organization.
    pub fn with_purchasing_org(mut self, org: impl Into<String>) -> Self {
        self.purchasing_org = org.into();
        self
    }

    /// Set purchasing group.
    pub fn with_purchasing_group(mut self, group: impl Into<String>) -> Self {
        self.purchasing_group = group.into();
        self
    }

    /// Set payment terms.
    pub fn with_payment_terms(mut self, terms: impl Into<String>) -> Self {
        self.payment_terms = terms.into();
        self
    }

    /// Set incoterms.
    pub fn with_incoterms(
        mut self,
        incoterms: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        self.incoterms = Some(incoterms.into());
        self.incoterms_location = Some(location.into());
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: PurchaseOrderItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals from items.
    pub fn recalculate_totals(&mut self) {
        self.total_net_amount = self.items.iter().map(|i| i.base.net_amount).sum();
        self.total_tax_amount = self.items.iter().map(|i| i.base.tax_amount).sum();
        self.total_gross_amount = self.items.iter().map(|i| i.base.gross_amount).sum();
    }

    /// Release the PO for processing.
    pub fn release(&mut self, user: impl Into<String>) {
        self.header.update_status(DocumentStatus::Released, user);
    }

    /// Check if all items are fully received.
    pub fn check_complete(&mut self) {
        self.is_complete = self
            .items
            .iter()
            .all(|i| !i.gr_indicator || i.is_fully_received)
            && self
                .items
                .iter()
                .all(|i| !i.ir_indicator || i.is_fully_invoiced);
    }

    /// Get total open amount for goods receipt.
    pub fn open_gr_amount(&self) -> Decimal {
        self.items
            .iter()
            .filter(|i| i.gr_indicator)
            .map(|i| i.open_quantity_gr() * i.base.unit_price)
            .sum()
    }

    /// Get total open amount for invoice.
    pub fn open_iv_amount(&self) -> Decimal {
        self.items
            .iter()
            .filter(|i| i.ir_indicator)
            .map(PurchaseOrderItem::open_amount_iv)
            .sum()
    }

    /// Close the PO.
    pub fn close(&mut self, user: impl Into<String>) {
        self.is_closed = true;
        self.header.update_status(DocumentStatus::Completed, user);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_order_creation() {
        let po = PurchaseOrder::new(
            "PO-1000-0000000001",
            "1000",
            "V-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(po.vendor_id, "V-000001");
        assert_eq!(po.header.status, DocumentStatus::Draft);
    }

    #[test]
    fn test_purchase_order_items() {
        let mut po = PurchaseOrder::new(
            "PO-1000-0000000001",
            "1000",
            "V-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        po.add_item(
            PurchaseOrderItem::new(1, "Office Supplies", Decimal::from(10), Decimal::from(25))
                .with_cost_center("CC-1000"),
        );

        po.add_item(
            PurchaseOrderItem::new(
                2,
                "Computer Equipment",
                Decimal::from(5),
                Decimal::from(500),
            )
            .with_cost_center("CC-1000"),
        );

        assert_eq!(po.items.len(), 2);
        assert_eq!(po.total_net_amount, Decimal::from(2750)); // 250 + 2500
    }

    #[test]
    fn test_goods_receipt_tracking() {
        let mut item =
            PurchaseOrderItem::new(1, "Test Item", Decimal::from(100), Decimal::from(10));

        assert_eq!(item.open_quantity_gr(), Decimal::from(100));

        item.record_goods_receipt(Decimal::from(60));
        assert_eq!(item.open_quantity_gr(), Decimal::from(40));
        assert!(!item.is_fully_received);

        item.record_goods_receipt(Decimal::from(40));
        assert_eq!(item.open_quantity_gr(), Decimal::ZERO);
        assert!(item.is_fully_received);
    }

    #[test]
    fn test_service_order() {
        let item = PurchaseOrderItem::service(
            1,
            "Consulting Services",
            Decimal::from(40),
            Decimal::from(150),
        );

        assert_eq!(item.item_category, "SERVICE");
        assert!(!item.gr_indicator);
        assert_eq!(item.base.uom, "HR");
    }
}
