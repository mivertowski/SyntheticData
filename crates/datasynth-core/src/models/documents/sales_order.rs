//! Sales Order document model.
//!
//! Represents sales orders in the O2C (Order-to-Cash) process flow.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{DocumentHeader, DocumentLineItem, DocumentStatus, DocumentType};

/// Sales Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SalesOrderType {
    /// Standard sales order
    #[default]
    Standard,
    /// Rush order
    Rush,
    /// Cash sale
    CashSale,
    /// Return order
    Return,
    /// Free of charge delivery
    FreeOfCharge,
    /// Consignment order
    Consignment,
    /// Service order
    Service,
    /// Credit memo request
    CreditMemoRequest,
    /// Debit memo request
    DebitMemoRequest,
}

impl SalesOrderType {
    /// Check if this order type requires delivery.
    pub fn requires_delivery(&self) -> bool {
        !matches!(
            self,
            Self::Service | Self::CreditMemoRequest | Self::DebitMemoRequest
        )
    }

    /// Check if this creates revenue.
    pub fn creates_revenue(&self) -> bool {
        !matches!(
            self,
            Self::FreeOfCharge | Self::Return | Self::CreditMemoRequest
        )
    }
}

/// Sales Order item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Item category
    pub item_category: String,

    /// Schedule line (delivery schedule)
    pub schedule_lines: Vec<ScheduleLine>,

    /// Quantity confirmed
    pub quantity_confirmed: Decimal,

    /// Quantity delivered
    pub quantity_delivered: Decimal,

    /// Quantity invoiced
    pub quantity_invoiced: Decimal,

    /// Is this line fully delivered?
    pub is_fully_delivered: bool,

    /// Is this line fully invoiced?
    pub is_fully_invoiced: bool,

    /// Rejection reason (if rejected)
    pub rejection_reason: Option<String>,

    /// Is this line rejected?
    pub is_rejected: bool,

    /// Route
    pub route: Option<String>,

    /// Shipping point
    pub shipping_point: Option<String>,
}

/// Schedule line for delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleLine {
    /// Schedule line number
    pub schedule_number: u16,
    /// Requested delivery date
    pub requested_date: NaiveDate,
    /// Confirmed delivery date
    pub confirmed_date: Option<NaiveDate>,
    /// Scheduled quantity
    pub quantity: Decimal,
    /// Delivered quantity
    pub delivered_quantity: Decimal,
}

impl SalesOrderItem {
    /// Create a new sales order item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        Self {
            base: DocumentLineItem::new(line_number, description, quantity, unit_price),
            item_category: "TAN".to_string(), // Standard item
            schedule_lines: Vec::new(),
            quantity_confirmed: Decimal::ZERO,
            quantity_delivered: Decimal::ZERO,
            quantity_invoiced: Decimal::ZERO,
            is_fully_delivered: false,
            is_fully_invoiced: false,
            rejection_reason: None,
            is_rejected: false,
            route: None,
            shipping_point: None,
        }
    }

    /// Set material.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.base = self.base.with_material(material_id);
        self
    }

    /// Set plant.
    pub fn with_plant(mut self, plant: impl Into<String>) -> Self {
        self.base.plant = Some(plant.into());
        self
    }

    /// Add a schedule line.
    pub fn add_schedule_line(&mut self, requested_date: NaiveDate, quantity: Decimal) {
        let schedule_number = (self.schedule_lines.len() + 1) as u16;
        self.schedule_lines.push(ScheduleLine {
            schedule_number,
            requested_date,
            confirmed_date: None,
            quantity,
            delivered_quantity: Decimal::ZERO,
        });
    }

    /// Confirm the schedule line.
    pub fn confirm_schedule(&mut self, schedule_number: u16, confirmed_date: NaiveDate) {
        if let Some(line) = self
            .schedule_lines
            .iter_mut()
            .find(|l| l.schedule_number == schedule_number)
        {
            line.confirmed_date = Some(confirmed_date);
            self.quantity_confirmed += line.quantity;
        }
    }

    /// Record delivery.
    pub fn record_delivery(&mut self, quantity: Decimal) {
        self.quantity_delivered += quantity;
        if self.quantity_delivered >= self.base.quantity {
            self.is_fully_delivered = true;
        }
    }

    /// Record invoice.
    pub fn record_invoice(&mut self, quantity: Decimal) {
        self.quantity_invoiced += quantity;
        if self.quantity_invoiced >= self.base.quantity {
            self.is_fully_invoiced = true;
        }
    }

    /// Open quantity for delivery.
    pub fn open_quantity_delivery(&self) -> Decimal {
        (self.base.quantity - self.quantity_delivered).max(Decimal::ZERO)
    }

    /// Open quantity for billing.
    pub fn open_quantity_billing(&self) -> Decimal {
        (self.quantity_delivered - self.quantity_invoiced).max(Decimal::ZERO)
    }

    /// Reject the line.
    pub fn reject(&mut self, reason: impl Into<String>) {
        self.is_rejected = true;
        self.rejection_reason = Some(reason.into());
    }
}

/// Sales Order document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    /// Document header
    pub header: DocumentHeader,

    /// SO type
    pub so_type: SalesOrderType,

    /// Customer ID
    pub customer_id: String,

    /// Sold-to party (if different from customer)
    pub sold_to: Option<String>,

    /// Ship-to party
    pub ship_to: Option<String>,

    /// Bill-to party
    pub bill_to: Option<String>,

    /// Payer
    pub payer: Option<String>,

    /// Sales organization
    pub sales_org: String,

    /// Distribution channel
    pub distribution_channel: String,

    /// Division
    pub division: String,

    /// Sales office
    pub sales_office: Option<String>,

    /// Sales group
    pub sales_group: Option<String>,

    /// Line items
    pub items: Vec<SalesOrderItem>,

    /// Total net amount
    pub total_net_amount: Decimal,

    /// Total tax amount
    pub total_tax_amount: Decimal,

    /// Total gross amount
    pub total_gross_amount: Decimal,

    /// Payment terms
    pub payment_terms: String,

    /// Incoterms
    pub incoterms: Option<String>,

    /// Shipping condition
    pub shipping_condition: Option<String>,

    /// Requested delivery date
    pub requested_delivery_date: Option<NaiveDate>,

    /// Customer PO number
    pub customer_po_number: Option<String>,

    /// Is this order complete?
    pub is_complete: bool,

    /// Credit status
    pub credit_status: CreditStatus,

    /// Credit block reason
    pub credit_block_reason: Option<String>,

    /// Is order released for delivery?
    pub is_delivery_released: bool,

    /// Is order released for billing?
    pub is_billing_released: bool,

    /// Related quote (if from quote)
    pub quote_id: Option<String>,

    /// Contract reference
    pub contract_id: Option<String>,
}

/// Credit check status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CreditStatus {
    /// Not checked
    #[default]
    NotChecked,
    /// Passed
    Passed,
    /// Failed - blocked
    Failed,
    /// Manually released
    Released,
}

impl SalesOrder {
    /// Create a new sales order.
    pub fn new(
        so_id: impl Into<String>,
        company_code: impl Into<String>,
        customer_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            so_id,
            DocumentType::SalesOrder,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );

        Self {
            header,
            so_type: SalesOrderType::Standard,
            customer_id: customer_id.into(),
            sold_to: None,
            ship_to: None,
            bill_to: None,
            payer: None,
            sales_org: "1000".to_string(),
            distribution_channel: "10".to_string(),
            division: "00".to_string(),
            sales_office: None,
            sales_group: None,
            items: Vec::new(),
            total_net_amount: Decimal::ZERO,
            total_tax_amount: Decimal::ZERO,
            total_gross_amount: Decimal::ZERO,
            payment_terms: "NET30".to_string(),
            incoterms: None,
            shipping_condition: None,
            requested_delivery_date: None,
            customer_po_number: None,
            is_complete: false,
            credit_status: CreditStatus::NotChecked,
            credit_block_reason: None,
            is_delivery_released: false,
            is_billing_released: false,
            quote_id: None,
            contract_id: None,
        }
    }

    /// Set SO type.
    pub fn with_so_type(mut self, so_type: SalesOrderType) -> Self {
        self.so_type = so_type;
        self
    }

    /// Set sales organization.
    pub fn with_sales_org(
        mut self,
        sales_org: impl Into<String>,
        dist_channel: impl Into<String>,
        division: impl Into<String>,
    ) -> Self {
        self.sales_org = sales_org.into();
        self.distribution_channel = dist_channel.into();
        self.division = division.into();
        self
    }

    /// Set partner functions.
    pub fn with_partners(
        mut self,
        sold_to: impl Into<String>,
        ship_to: impl Into<String>,
        bill_to: impl Into<String>,
    ) -> Self {
        self.sold_to = Some(sold_to.into());
        self.ship_to = Some(ship_to.into());
        self.bill_to = Some(bill_to.into());
        self
    }

    /// Set customer PO.
    pub fn with_customer_po(mut self, po_number: impl Into<String>) -> Self {
        self.customer_po_number = Some(po_number.into());
        self
    }

    /// Set requested delivery date.
    pub fn with_requested_delivery_date(mut self, date: NaiveDate) -> Self {
        self.requested_delivery_date = Some(date);
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: SalesOrderItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.total_net_amount = self
            .items
            .iter()
            .filter(|i| !i.is_rejected)
            .map(|i| i.base.net_amount)
            .sum();
        self.total_tax_amount = self
            .items
            .iter()
            .filter(|i| !i.is_rejected)
            .map(|i| i.base.tax_amount)
            .sum();
        self.total_gross_amount = self.total_net_amount + self.total_tax_amount;
    }

    /// Perform credit check.
    pub fn check_credit(&mut self, passed: bool, block_reason: Option<String>) {
        if passed {
            self.credit_status = CreditStatus::Passed;
            self.credit_block_reason = None;
        } else {
            self.credit_status = CreditStatus::Failed;
            self.credit_block_reason = block_reason;
        }
    }

    /// Release credit block.
    pub fn release_credit_block(&mut self, user: impl Into<String>) {
        self.credit_status = CreditStatus::Released;
        self.credit_block_reason = None;
        self.header.update_status(DocumentStatus::Released, user);
    }

    /// Release for delivery.
    pub fn release_for_delivery(&mut self) {
        self.is_delivery_released = true;
    }

    /// Release for billing.
    pub fn release_for_billing(&mut self) {
        self.is_billing_released = true;
    }

    /// Check if order is complete.
    pub fn check_complete(&mut self) {
        self.is_complete = self
            .items
            .iter()
            .all(|i| i.is_rejected || i.is_fully_invoiced);
    }

    /// Get total open delivery value.
    pub fn open_delivery_value(&self) -> Decimal {
        self.items
            .iter()
            .filter(|i| !i.is_rejected)
            .map(|i| i.open_quantity_delivery() * i.base.unit_price)
            .sum()
    }

    /// Get total open billing value.
    pub fn open_billing_value(&self) -> Decimal {
        self.items
            .iter()
            .filter(|i| !i.is_rejected)
            .map(|i| i.open_quantity_billing() * i.base.unit_price)
            .sum()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sales_order_creation() {
        let so = SalesOrder::new(
            "SO-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(so.customer_id, "C-000001");
        assert_eq!(so.header.status, DocumentStatus::Draft);
    }

    #[test]
    fn test_sales_order_items() {
        let mut so = SalesOrder::new(
            "SO-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let mut item = SalesOrderItem::new(1, "Product A", Decimal::from(10), Decimal::from(100));
        item.add_schedule_line(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            Decimal::from(10),
        );

        so.add_item(item);

        assert_eq!(so.total_net_amount, Decimal::from(1000));
        assert_eq!(so.items[0].schedule_lines.len(), 1);
    }

    #[test]
    fn test_delivery_tracking() {
        let mut item = SalesOrderItem::new(1, "Product A", Decimal::from(100), Decimal::from(10));

        assert_eq!(item.open_quantity_delivery(), Decimal::from(100));

        item.record_delivery(Decimal::from(60));
        assert_eq!(item.open_quantity_delivery(), Decimal::from(40));
        assert!(!item.is_fully_delivered);

        item.record_delivery(Decimal::from(40));
        assert!(item.is_fully_delivered);
    }
}
