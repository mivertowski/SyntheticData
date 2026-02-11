//! Delivery document model.
//!
//! Represents outbound deliveries in the O2C (Order-to-Cash) process flow.
//! Deliveries create accounting entries: DR COGS, CR Inventory.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DocumentHeader, DocumentLineItem, DocumentReference, DocumentStatus, DocumentType,
    ReferenceType,
};

/// Delivery type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    /// Standard outbound delivery
    #[default]
    Outbound,
    /// Return delivery (from customer)
    Return,
    /// Stock transfer delivery
    StockTransfer,
    /// Replenishment delivery
    Replenishment,
    /// Consignment issue
    ConsignmentIssue,
    /// Consignment return
    ConsignmentReturn,
}

/// Delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    /// Created - not yet picked
    #[default]
    Created,
    /// Pick released
    PickReleased,
    /// Picking in progress
    Picking,
    /// Picked and ready for packing
    Picked,
    /// Packed
    Packed,
    /// Goods issued
    GoodsIssued,
    /// In transit
    InTransit,
    /// Delivered
    Delivered,
    /// Partially delivered
    PartiallyDelivered,
    /// Cancelled
    Cancelled,
}

/// Delivery item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Reference SO number
    pub sales_order_id: Option<String>,

    /// Reference SO item
    pub so_item: Option<u16>,

    /// Picked quantity
    pub quantity_picked: Decimal,

    /// Packed quantity
    pub quantity_packed: Decimal,

    /// Goods issued quantity
    pub quantity_issued: Decimal,

    /// Is this line fully picked?
    pub is_fully_picked: bool,

    /// Is this line fully issued?
    pub is_fully_issued: bool,

    /// Batch number (if batch managed)
    pub batch: Option<String>,

    /// Serial numbers (if serial managed)
    pub serial_numbers: Vec<String>,

    /// Pick location (bin)
    pub pick_location: Option<String>,

    /// Handling unit
    pub handling_unit: Option<String>,

    /// Weight in kg
    pub weight: Option<Decimal>,

    /// Volume in m³
    pub volume: Option<Decimal>,

    /// COGS amount (cost of goods sold)
    pub cogs_amount: Decimal,
}

impl DeliveryItem {
    /// Create a new delivery item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        Self {
            base: DocumentLineItem::new(line_number, description, quantity, unit_price),
            sales_order_id: None,
            so_item: None,
            quantity_picked: Decimal::ZERO,
            quantity_packed: Decimal::ZERO,
            quantity_issued: Decimal::ZERO,
            is_fully_picked: false,
            is_fully_issued: false,
            batch: None,
            serial_numbers: Vec::new(),
            pick_location: None,
            handling_unit: None,
            weight: None,
            volume: None,
            cogs_amount: Decimal::ZERO,
        }
    }

    /// Create from SO reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_sales_order(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        sales_order_id: impl Into<String>,
        so_item: u16,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price);
        item.sales_order_id = Some(sales_order_id.into());
        item.so_item = Some(so_item);
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

    /// Set COGS amount (cost).
    pub fn with_cogs(mut self, cogs: Decimal) -> Self {
        self.cogs_amount = cogs;
        self
    }

    /// Set location.
    pub fn with_location(
        mut self,
        plant: impl Into<String>,
        storage_location: impl Into<String>,
    ) -> Self {
        self.base.plant = Some(plant.into());
        self.base.storage_location = Some(storage_location.into());
        self
    }

    /// Set weight and volume.
    pub fn with_dimensions(mut self, weight: Decimal, volume: Decimal) -> Self {
        self.weight = Some(weight);
        self.volume = Some(volume);
        self
    }

    /// Add serial number.
    pub fn add_serial_number(&mut self, serial: impl Into<String>) {
        self.serial_numbers.push(serial.into());
    }

    /// Record pick.
    pub fn record_pick(&mut self, quantity: Decimal) {
        self.quantity_picked += quantity;
        if self.quantity_picked >= self.base.quantity {
            self.is_fully_picked = true;
        }
    }

    /// Record pack.
    pub fn record_pack(&mut self, quantity: Decimal) {
        self.quantity_packed += quantity;
    }

    /// Record goods issue.
    pub fn record_goods_issue(&mut self, quantity: Decimal) {
        self.quantity_issued += quantity;
        if self.quantity_issued >= self.base.quantity {
            self.is_fully_issued = true;
        }
    }

    /// Get open quantity for picking.
    pub fn open_quantity_pick(&self) -> Decimal {
        (self.base.quantity - self.quantity_picked).max(Decimal::ZERO)
    }

    /// Get open quantity for goods issue.
    pub fn open_quantity_gi(&self) -> Decimal {
        (self.quantity_picked - self.quantity_issued).max(Decimal::ZERO)
    }
}

/// Delivery document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delivery {
    /// Document header
    pub header: DocumentHeader,

    /// Delivery type
    pub delivery_type: DeliveryType,

    /// Delivery status
    pub delivery_status: DeliveryStatus,

    /// Line items
    pub items: Vec<DeliveryItem>,

    /// Total quantity
    pub total_quantity: Decimal,

    /// Total weight (kg)
    pub total_weight: Decimal,

    /// Total volume (m³)
    pub total_volume: Decimal,

    /// Customer ID
    pub customer_id: String,

    /// Ship-to party (if different)
    pub ship_to: Option<String>,

    /// Reference sales order (primary)
    pub sales_order_id: Option<String>,

    /// Shipping point
    pub shipping_point: String,

    /// Route
    pub route: Option<String>,

    /// Carrier/forwarder
    pub carrier: Option<String>,

    /// Shipping condition
    pub shipping_condition: Option<String>,

    /// Incoterms
    pub incoterms: Option<String>,

    /// Planned goods issue date
    pub planned_gi_date: NaiveDate,

    /// Actual goods issue date
    pub actual_gi_date: Option<NaiveDate>,

    /// Delivery date
    pub delivery_date: Option<NaiveDate>,

    /// Proof of delivery
    pub pod_date: Option<NaiveDate>,

    /// POD signed by
    pub pod_signed_by: Option<String>,

    /// Bill of lading
    pub bill_of_lading: Option<String>,

    /// Tracking number
    pub tracking_number: Option<String>,

    /// Number of packages
    pub number_of_packages: u32,

    /// Is goods issued?
    pub is_goods_issued: bool,

    /// Is delivery complete?
    pub is_complete: bool,

    /// Is this delivery cancelled/reversed?
    pub is_cancelled: bool,

    /// Cancellation document reference
    pub cancellation_doc: Option<String>,

    /// Total COGS (for GL posting)
    pub total_cogs: Decimal,
}

impl Delivery {
    /// Create a new delivery.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        delivery_id: impl Into<String>,
        company_code: impl Into<String>,
        customer_id: impl Into<String>,
        shipping_point: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            delivery_id,
            DocumentType::Delivery,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );

        Self {
            header,
            delivery_type: DeliveryType::Outbound,
            delivery_status: DeliveryStatus::Created,
            items: Vec::new(),
            total_quantity: Decimal::ZERO,
            total_weight: Decimal::ZERO,
            total_volume: Decimal::ZERO,
            customer_id: customer_id.into(),
            ship_to: None,
            sales_order_id: None,
            shipping_point: shipping_point.into(),
            route: None,
            carrier: None,
            shipping_condition: None,
            incoterms: None,
            planned_gi_date: document_date,
            actual_gi_date: None,
            delivery_date: None,
            pod_date: None,
            pod_signed_by: None,
            bill_of_lading: None,
            tracking_number: None,
            number_of_packages: 0,
            is_goods_issued: false,
            is_complete: false,
            is_cancelled: false,
            cancellation_doc: None,
            total_cogs: Decimal::ZERO,
        }
    }

    /// Create from sales order reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_sales_order(
        delivery_id: impl Into<String>,
        company_code: impl Into<String>,
        sales_order_id: impl Into<String>,
        customer_id: impl Into<String>,
        shipping_point: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let so_id = sales_order_id.into();
        let mut delivery = Self::new(
            delivery_id,
            company_code,
            customer_id,
            shipping_point,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );
        delivery.sales_order_id = Some(so_id.clone());

        // Add reference to SO
        delivery.header.add_reference(DocumentReference::new(
            DocumentType::SalesOrder,
            so_id,
            DocumentType::Delivery,
            delivery.header.document_id.clone(),
            ReferenceType::FollowOn,
            delivery.header.company_code.clone(),
            document_date,
        ));

        delivery
    }

    /// Set delivery type.
    pub fn with_delivery_type(mut self, delivery_type: DeliveryType) -> Self {
        self.delivery_type = delivery_type;
        self
    }

    /// Set ship-to party.
    pub fn with_ship_to(mut self, ship_to: impl Into<String>) -> Self {
        self.ship_to = Some(ship_to.into());
        self
    }

    /// Set carrier.
    pub fn with_carrier(mut self, carrier: impl Into<String>) -> Self {
        self.carrier = Some(carrier.into());
        self
    }

    /// Set route.
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Set planned GI date.
    pub fn with_planned_gi_date(mut self, date: NaiveDate) -> Self {
        self.planned_gi_date = date;
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: DeliveryItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.total_quantity = self.items.iter().map(|i| i.base.quantity).sum();
        self.total_weight = self.items.iter().filter_map(|i| i.weight).sum();
        self.total_volume = self.items.iter().filter_map(|i| i.volume).sum();
        self.total_cogs = self.items.iter().map(|i| i.cogs_amount).sum();
    }

    /// Release for picking.
    pub fn release_for_picking(&mut self, user: impl Into<String>) {
        self.delivery_status = DeliveryStatus::PickReleased;
        self.header.update_status(DocumentStatus::Released, user);
    }

    /// Start picking.
    pub fn start_picking(&mut self) {
        self.delivery_status = DeliveryStatus::Picking;
    }

    /// Confirm pick complete.
    pub fn confirm_pick(&mut self) {
        if self.items.iter().all(|i| i.is_fully_picked) {
            self.delivery_status = DeliveryStatus::Picked;
        }
    }

    /// Confirm packing.
    pub fn confirm_pack(&mut self, num_packages: u32) {
        self.delivery_status = DeliveryStatus::Packed;
        self.number_of_packages = num_packages;
    }

    /// Post goods issue.
    pub fn post_goods_issue(&mut self, user: impl Into<String>, gi_date: NaiveDate) {
        self.actual_gi_date = Some(gi_date);
        self.is_goods_issued = true;
        self.delivery_status = DeliveryStatus::GoodsIssued;
        self.header.posting_date = Some(gi_date);
        self.header.update_status(DocumentStatus::Posted, user);

        // Mark all items as fully issued
        for item in &mut self.items {
            item.quantity_issued = item.quantity_picked;
            item.is_fully_issued = true;
        }
    }

    /// Confirm delivery.
    pub fn confirm_delivery(&mut self, delivery_date: NaiveDate) {
        self.delivery_date = Some(delivery_date);
        self.delivery_status = DeliveryStatus::Delivered;
    }

    /// Record proof of delivery.
    pub fn record_pod(&mut self, pod_date: NaiveDate, signed_by: impl Into<String>) {
        self.pod_date = Some(pod_date);
        self.pod_signed_by = Some(signed_by.into());
        self.is_complete = true;
        self.header
            .update_status(DocumentStatus::Completed, "SYSTEM");
    }

    /// Cancel the delivery.
    pub fn cancel(&mut self, user: impl Into<String>, reason: impl Into<String>) {
        self.is_cancelled = true;
        self.delivery_status = DeliveryStatus::Cancelled;
        self.header.header_text = Some(reason.into());
        self.header.update_status(DocumentStatus::Cancelled, user);
    }

    /// Generate GL entries for goods issue.
    /// DR COGS (or expense), CR Inventory
    pub fn generate_gl_entries(&self) -> Vec<(String, Decimal, Decimal)> {
        let mut entries = Vec::new();

        if !self.is_goods_issued {
            return entries;
        }

        for item in &self.items {
            if item.cogs_amount > Decimal::ZERO {
                // Debit: COGS
                let cogs_account = item
                    .base
                    .gl_account
                    .clone()
                    .unwrap_or_else(|| "500000".to_string());

                // Credit: Inventory
                let inventory_account = "140000".to_string();

                entries.push((cogs_account, item.cogs_amount, Decimal::ZERO));
                entries.push((inventory_account, Decimal::ZERO, item.cogs_amount));
            }
        }

        entries
    }

    /// Get total value (sales value).
    pub fn total_value(&self) -> Decimal {
        self.items.iter().map(|i| i.base.net_amount).sum()
    }

    /// Check if all items are picked.
    pub fn is_fully_picked(&self) -> bool {
        self.items.iter().all(|i| i.is_fully_picked)
    }

    /// Check if all items are issued.
    pub fn is_fully_issued(&self) -> bool {
        self.items.iter().all(|i| i.is_fully_issued)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_creation() {
        let delivery = Delivery::new(
            "DLV-1000-0000000001",
            "1000",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(delivery.customer_id, "C-000001");
        assert_eq!(delivery.shipping_point, "SP01");
        assert_eq!(delivery.delivery_status, DeliveryStatus::Created);
    }

    #[test]
    fn test_delivery_from_sales_order() {
        let delivery = Delivery::from_sales_order(
            "DLV-1000-0000000001",
            "1000",
            "SO-1000-0000000001",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(
            delivery.sales_order_id,
            Some("SO-1000-0000000001".to_string())
        );
        assert_eq!(delivery.header.document_references.len(), 1);
    }

    #[test]
    fn test_delivery_items() {
        let mut delivery = Delivery::new(
            "DLV-1000-0000000001",
            "1000",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let item = DeliveryItem::from_sales_order(
            1,
            "Product A",
            Decimal::from(100),
            Decimal::from(50),
            "SO-1000-0000000001",
            1,
        )
        .with_material("MAT-001")
        .with_cogs(Decimal::from(3000)); // 100 units * $30 cost

        delivery.add_item(item);

        assert_eq!(delivery.total_quantity, Decimal::from(100));
        assert_eq!(delivery.total_cogs, Decimal::from(3000));
    }

    #[test]
    fn test_pick_process() {
        let mut item = DeliveryItem::new(1, "Product A", Decimal::from(100), Decimal::from(50));

        assert_eq!(item.open_quantity_pick(), Decimal::from(100));

        item.record_pick(Decimal::from(60));
        assert_eq!(item.open_quantity_pick(), Decimal::from(40));
        assert!(!item.is_fully_picked);

        item.record_pick(Decimal::from(40));
        assert!(item.is_fully_picked);
    }

    #[test]
    fn test_goods_issue_process() {
        let mut delivery = Delivery::new(
            "DLV-1000-0000000001",
            "1000",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let mut item = DeliveryItem::new(1, "Product A", Decimal::from(100), Decimal::from(50))
            .with_cogs(Decimal::from(3000));

        item.record_pick(Decimal::from(100));
        delivery.add_item(item);

        delivery.release_for_picking("PICKER");
        delivery.confirm_pick();
        delivery.confirm_pack(5);
        delivery.post_goods_issue("SHIPPER", NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());

        assert!(delivery.is_goods_issued);
        assert_eq!(delivery.delivery_status, DeliveryStatus::GoodsIssued);
        assert!(delivery.is_fully_issued());
    }

    #[test]
    fn test_gl_entry_generation() {
        let mut delivery = Delivery::new(
            "DLV-1000-0000000001",
            "1000",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let mut item = DeliveryItem::new(1, "Product A", Decimal::from(100), Decimal::from(50))
            .with_cogs(Decimal::from(3000));

        item.record_pick(Decimal::from(100));
        delivery.add_item(item);
        delivery.post_goods_issue("SHIPPER", NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());

        let entries = delivery.generate_gl_entries();
        assert_eq!(entries.len(), 2);
        // DR COGS
        assert_eq!(entries[0].1, Decimal::from(3000));
        // CR Inventory
        assert_eq!(entries[1].2, Decimal::from(3000));
    }

    #[test]
    fn test_delivery_complete() {
        let mut delivery = Delivery::new(
            "DLV-1000-0000000001",
            "1000",
            "C-000001",
            "SP01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        delivery.post_goods_issue("SHIPPER", NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        delivery.confirm_delivery(NaiveDate::from_ymd_opt(2024, 1, 18).unwrap());
        delivery.record_pod(NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(), "John Doe");

        assert!(delivery.is_complete);
        assert_eq!(delivery.delivery_status, DeliveryStatus::Delivered);
        assert_eq!(delivery.pod_signed_by, Some("John Doe".to_string()));
    }
}
