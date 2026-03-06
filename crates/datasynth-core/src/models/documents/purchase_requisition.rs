//! Purchase Requisition document model.
//!
//! Represents purchase requisitions in the P2P (Procure-to-Pay) process flow.
//! Purchase requisitions are the starting point for procurement and do not create GL entries.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{DocumentHeader, DocumentLineItem, DocumentStatus, DocumentType};

/// Purchase Requisition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseRequisitionType {
    /// Standard PR for goods/services
    #[default]
    Standard,
    /// Emergency/rush PR requiring expedited processing
    Emergency,
    /// Framework PR for blanket release orders
    Framework,
    /// Consignment PR for consignment inventory
    Consignment,
    /// Non-stock material PR (direct expense)
    NonStock,
    /// Service PR for external services
    Service,
}

impl PurchaseRequisitionType {
    /// Check if this PR type requires expedited processing.
    pub fn is_expedited(&self) -> bool {
        matches!(self, Self::Emergency)
    }

    /// Check if this PR type requires goods receipt.
    pub fn requires_goods_receipt(&self) -> bool {
        !matches!(self, Self::Service | Self::NonStock)
    }

    /// Check if this is a service requisition.
    pub fn is_service(&self) -> bool {
        matches!(self, Self::Service)
    }
}

/// Purchase Requisition priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequisitionPriority {
    /// Low priority - can wait
    Low,
    /// Normal priority - standard processing
    #[default]
    Normal,
    /// High priority - expedite
    High,
    /// Urgent - immediate attention required
    Urgent,
}

impl RequisitionPriority {
    /// Get the numeric priority value (higher = more urgent).
    pub fn value(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 2,
            Self::High => 3,
            Self::Urgent => 4,
        }
    }
}

/// Purchase Requisition line item with procurement-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequisitionItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Requester user ID
    pub requester: String,

    /// Approver user ID (when approved)
    pub approver: Option<String>,

    /// Purchasing group to handle this item
    pub purchasing_group: Option<String>,

    /// Preferred vendor ID (if any)
    pub preferred_vendor: Option<String>,

    /// Fixed vendor (must use this vendor)
    pub fixed_vendor: Option<String>,

    /// Budget center for approval
    pub budget_center: Option<String>,

    /// Is this item approved?
    pub is_approved: bool,

    /// Is this item rejected?
    pub is_rejected: bool,

    /// Rejection reason
    pub rejection_reason: Option<String>,

    /// Business justification/reason for request
    pub reason: Option<String>,

    /// Requested delivery date
    pub requested_date: Option<NaiveDate>,

    /// Item category (goods, service, limit, etc.)
    pub item_category: String,

    /// Account assignment category (K=cost center, F=order, etc.)
    pub account_assignment_category: String,

    /// Related Purchase Order ID (once converted)
    pub purchase_order_id: Option<String>,

    /// Related PO item number
    pub purchase_order_item: Option<u16>,

    /// Is this item closed (cannot be converted)?
    pub is_closed: bool,

    /// Tracking number for status updates
    pub tracking_number: Option<String>,
}

impl PurchaseRequisitionItem {
    /// Create a new purchase requisition item.
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            base: DocumentLineItem::new(line_number, description, quantity, unit_price),
            requester: requester.into(),
            approver: None,
            purchasing_group: None,
            preferred_vendor: None,
            fixed_vendor: None,
            budget_center: None,
            is_approved: false,
            is_rejected: false,
            rejection_reason: None,
            reason: None,
            requested_date: None,
            item_category: "GOODS".to_string(),
            account_assignment_category: "K".to_string(),
            purchase_order_id: None,
            purchase_order_item: None,
            is_closed: false,
            tracking_number: None,
        }
    }

    /// Create a service line item.
    pub fn service(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        requester: impl Into<String>,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price, requester);
        item.item_category = "SERVICE".to_string();
        item.base.uom = "HR".to_string();
        item
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

    /// Set material.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.base = self.base.with_material(material_id);
        self
    }

    /// Set purchasing group.
    pub fn with_purchasing_group(mut self, group: impl Into<String>) -> Self {
        self.purchasing_group = Some(group.into());
        self
    }

    /// Set preferred vendor.
    pub fn with_preferred_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.preferred_vendor = Some(vendor.into());
        self
    }

    /// Set fixed vendor (mandatory source).
    pub fn with_fixed_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.fixed_vendor = Some(vendor.into());
        self
    }

    /// Set requested delivery date.
    pub fn with_requested_date(mut self, date: NaiveDate) -> Self {
        self.requested_date = Some(date);
        self.base = self.base.with_delivery_date(date);
        self
    }

    /// Set business justification.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set budget center.
    pub fn with_budget_center(mut self, budget_center: impl Into<String>) -> Self {
        self.budget_center = Some(budget_center.into());
        self
    }

    /// Approve the line item.
    pub fn approve(&mut self, approver: impl Into<String>) {
        self.is_approved = true;
        self.is_rejected = false;
        self.approver = Some(approver.into());
        self.rejection_reason = None;
    }

    /// Reject the line item.
    pub fn reject(&mut self, approver: impl Into<String>, reason: impl Into<String>) {
        self.is_rejected = true;
        self.is_approved = false;
        self.approver = Some(approver.into());
        self.rejection_reason = Some(reason.into());
    }

    /// Convert to PO (mark as converted).
    pub fn convert_to_po(&mut self, po_id: impl Into<String>, po_item: u16) {
        self.purchase_order_id = Some(po_id.into());
        self.purchase_order_item = Some(po_item);
    }

    /// Close the item (cannot be converted).
    pub fn close(&mut self) {
        self.is_closed = true;
    }

    /// Check if item can be converted to PO.
    pub fn can_convert(&self) -> bool {
        self.is_approved && !self.is_rejected && !self.is_closed && self.purchase_order_id.is_none()
    }

    /// Get open quantity (not yet converted to PO).
    pub fn open_quantity(&self) -> Decimal {
        if self.purchase_order_id.is_some() || self.is_closed {
            Decimal::ZERO
        } else {
            self.base.quantity
        }
    }
}

/// Purchase Requisition document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequisition {
    /// Document header
    pub header: DocumentHeader,

    /// PR type
    pub pr_type: PurchaseRequisitionType,

    /// Priority
    pub priority: RequisitionPriority,

    /// Requester user ID (document-level)
    pub requester_id: String,

    /// Requester name
    pub requester_name: Option<String>,

    /// Requester email
    pub requester_email: Option<String>,

    /// Requester department
    pub requester_department: Option<String>,

    /// Approver user ID (document-level)
    pub approver_id: Option<String>,

    /// Purchasing organization
    pub purchasing_org: String,

    /// Default purchasing group
    pub purchasing_group: Option<String>,

    /// Line items
    pub items: Vec<PurchaseRequisitionItem>,

    /// Total net amount
    pub total_net_amount: Decimal,

    /// Total tax amount
    pub total_tax_amount: Decimal,

    /// Total gross amount
    pub total_gross_amount: Decimal,

    /// Is this PR fully approved?
    pub is_approved: bool,

    /// Is this PR fully converted to PO(s)?
    pub is_converted: bool,

    /// Is this PR closed?
    pub is_closed: bool,

    /// Related PO IDs (multiple POs can be created from one PR)
    pub purchase_order_ids: Vec<String>,

    /// Business justification (document-level)
    pub justification: Option<String>,

    /// Budget code for approval routing
    pub budget_code: Option<String>,

    /// Approval workflow ID
    pub workflow_id: Option<String>,

    /// Notes/comments
    pub notes: Option<String>,

    /// Desired vendor (document-level preference)
    pub desired_vendor: Option<String>,
}

impl PurchaseRequisition {
    /// Create a new purchase requisition.
    pub fn new(
        pr_id: impl Into<String>,
        company_code: impl Into<String>,
        requester_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            pr_id,
            DocumentType::PurchaseRequisition,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        );

        Self {
            header,
            pr_type: PurchaseRequisitionType::Standard,
            priority: RequisitionPriority::Normal,
            requester_id: requester_id.into(),
            requester_name: None,
            requester_email: None,
            requester_department: None,
            approver_id: None,
            purchasing_org: "1000".to_string(),
            purchasing_group: None,
            items: Vec::new(),
            total_net_amount: Decimal::ZERO,
            total_tax_amount: Decimal::ZERO,
            total_gross_amount: Decimal::ZERO,
            is_approved: false,
            is_converted: false,
            is_closed: false,
            purchase_order_ids: Vec::new(),
            justification: None,
            budget_code: None,
            workflow_id: None,
            notes: None,
            desired_vendor: None,
        }
    }

    /// Set PR type.
    pub fn with_pr_type(mut self, pr_type: PurchaseRequisitionType) -> Self {
        self.pr_type = pr_type;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: RequisitionPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set purchasing organization.
    pub fn with_purchasing_org(mut self, org: impl Into<String>) -> Self {
        self.purchasing_org = org.into();
        self
    }

    /// Set purchasing group.
    pub fn with_purchasing_group(mut self, group: impl Into<String>) -> Self {
        self.purchasing_group = Some(group.into());
        self
    }

    /// Set requester details.
    pub fn with_requester_details(
        mut self,
        name: impl Into<String>,
        email: impl Into<String>,
        department: impl Into<String>,
    ) -> Self {
        self.requester_name = Some(name.into());
        self.requester_email = Some(email.into());
        self.requester_department = Some(department.into());
        self
    }

    /// Set justification.
    pub fn with_justification(mut self, justification: impl Into<String>) -> Self {
        self.justification = Some(justification.into());
        self
    }

    /// Set budget code.
    pub fn with_budget_code(mut self, code: impl Into<String>) -> Self {
        self.budget_code = Some(code.into());
        self
    }

    /// Set desired vendor.
    pub fn with_desired_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.desired_vendor = Some(vendor.into());
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: PurchaseRequisitionItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals from items.
    pub fn recalculate_totals(&mut self) {
        self.total_net_amount = self.items.iter().map(|i| i.base.net_amount).sum();
        self.total_tax_amount = self.items.iter().map(|i| i.base.tax_amount).sum();
        self.total_gross_amount = self.items.iter().map(|i| i.base.gross_amount).sum();
    }

    /// Submit the PR for approval.
    pub fn submit(&mut self, user: impl Into<String>) {
        self.header.update_status(DocumentStatus::Submitted, user);
    }

    /// Approve the entire PR.
    pub fn approve(&mut self, approver: impl Into<String>) {
        let approver_str: String = approver.into();
        self.is_approved = true;
        self.approver_id = Some(approver_str.clone());

        // Approve all pending items
        for item in &mut self.items {
            if !item.is_rejected && !item.is_approved {
                item.approve(approver_str.clone());
            }
        }

        self.header
            .update_status(DocumentStatus::Approved, approver_str);
    }

    /// Reject the entire PR.
    pub fn reject(&mut self, approver: impl Into<String>, reason: impl Into<String>) {
        let approver_str: String = approver.into();
        let reason_str: String = reason.into();

        self.approver_id = Some(approver_str.clone());

        // Reject all non-approved items
        for item in &mut self.items {
            if !item.is_approved {
                item.reject(approver_str.clone(), reason_str.clone());
            }
        }

        self.header
            .update_status(DocumentStatus::Rejected, approver_str);
    }

    /// Release the PR for conversion to PO.
    pub fn release(&mut self, user: impl Into<String>) {
        if self.is_approved {
            self.header.update_status(DocumentStatus::Released, user);
        }
    }

    /// Convert to PO (mark as converted).
    pub fn convert_to_po(&mut self, po_id: impl Into<String>, user: impl Into<String>) {
        let po_id_str = po_id.into();
        self.purchase_order_ids.push(po_id_str.clone());

        // Check if all items are converted
        let all_converted = self
            .items
            .iter()
            .all(|i| i.purchase_order_id.is_some() || i.is_closed || i.is_rejected);

        if all_converted {
            self.is_converted = true;
            self.header.update_status(DocumentStatus::Completed, user);
        } else {
            self.header
                .update_status(DocumentStatus::PartiallyProcessed, user);
        }
    }

    /// Close the PR.
    pub fn close(&mut self, user: impl Into<String>) {
        self.is_closed = true;
        for item in &mut self.items {
            if item.purchase_order_id.is_none() {
                item.close();
            }
        }
        self.header.update_status(DocumentStatus::Completed, user);
    }

    /// Get total open amount (not yet converted to PO).
    pub fn open_amount(&self) -> Decimal {
        self.items
            .iter()
            .filter(|i| i.can_convert())
            .map(|i| i.base.net_amount)
            .sum()
    }

    /// Get count of approved items.
    pub fn approved_item_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_approved).count()
    }

    /// Get count of rejected items.
    pub fn rejected_item_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_rejected).count()
    }

    /// Get count of converted items.
    pub fn converted_item_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.purchase_order_id.is_some())
            .count()
    }

    /// Check if all items are approved.
    pub fn all_items_approved(&self) -> bool {
        !self.items.is_empty()
            && self
                .items
                .iter()
                .all(|i| i.is_approved || i.is_rejected || i.is_closed)
    }

    /// Check if any items can be converted.
    pub fn has_convertible_items(&self) -> bool {
        self.items.iter().any(PurchaseRequisitionItem::can_convert)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_requisition_creation() {
        let pr = PurchaseRequisition::new(
            "PR-1000-0000000001",
            "1000",
            "EMP-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(pr.requester_id, "EMP-001");
        assert_eq!(pr.header.status, DocumentStatus::Draft);
        assert!(!pr.is_approved);
        assert_eq!(pr.pr_type, PurchaseRequisitionType::Standard);
        assert_eq!(pr.priority, RequisitionPriority::Normal);
    }

    #[test]
    fn test_purchase_requisition_items() {
        let mut pr = PurchaseRequisition::new(
            "PR-1000-0000000001",
            "1000",
            "EMP-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        pr.add_item(
            PurchaseRequisitionItem::new(
                1,
                "Office Supplies",
                Decimal::from(10),
                Decimal::from(25),
                "EMP-001",
            )
            .with_cost_center("CC-1000"),
        );

        pr.add_item(
            PurchaseRequisitionItem::new(
                2,
                "Computer Equipment",
                Decimal::from(5),
                Decimal::from(500),
                "EMP-001",
            )
            .with_cost_center("CC-1000"),
        );

        assert_eq!(pr.items.len(), 2);
        assert_eq!(pr.total_net_amount, Decimal::from(2750)); // 250 + 2500
    }

    #[test]
    fn test_pr_approval_workflow() {
        let mut pr = PurchaseRequisition::new(
            "PR-1000-0000000001",
            "1000",
            "EMP-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        pr.add_item(PurchaseRequisitionItem::new(
            1,
            "Test Item",
            Decimal::from(10),
            Decimal::from(100),
            "EMP-001",
        ));

        // Submit
        pr.submit("JSMITH");
        assert_eq!(pr.header.status, DocumentStatus::Submitted);

        // Approve
        pr.approve("MANAGER");
        assert!(pr.is_approved);
        assert_eq!(pr.header.status, DocumentStatus::Approved);
        assert!(pr.items[0].is_approved);
        assert_eq!(pr.items[0].approver, Some("MANAGER".to_string()));
    }

    #[test]
    fn test_pr_rejection() {
        let mut pr = PurchaseRequisition::new(
            "PR-1000-0000000001",
            "1000",
            "EMP-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        pr.add_item(PurchaseRequisitionItem::new(
            1,
            "Expensive Item",
            Decimal::from(1),
            Decimal::from(10000),
            "EMP-001",
        ));

        pr.submit("JSMITH");
        pr.reject("MANAGER", "Budget exceeded");

        assert_eq!(pr.header.status, DocumentStatus::Rejected);
        assert!(pr.items[0].is_rejected);
        assert_eq!(
            pr.items[0].rejection_reason,
            Some("Budget exceeded".to_string())
        );
    }

    #[test]
    fn test_pr_conversion_to_po() {
        let mut pr = PurchaseRequisition::new(
            "PR-1000-0000000001",
            "1000",
            "EMP-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        pr.add_item(PurchaseRequisitionItem::new(
            1,
            "Test Item",
            Decimal::from(10),
            Decimal::from(100),
            "EMP-001",
        ));

        pr.approve("MANAGER");
        pr.release("BUYER");

        // Convert item to PO
        pr.items[0].convert_to_po("PO-1000-0000000001", 1);
        pr.convert_to_po("PO-1000-0000000001", "BUYER");

        assert!(pr.is_converted);
        assert_eq!(pr.header.status, DocumentStatus::Completed);
        assert_eq!(
            pr.items[0].purchase_order_id,
            Some("PO-1000-0000000001".to_string())
        );
    }

    #[test]
    fn test_pr_item_can_convert() {
        let mut item = PurchaseRequisitionItem::new(
            1,
            "Test Item",
            Decimal::from(10),
            Decimal::from(100),
            "EMP-001",
        );

        // Cannot convert if not approved
        assert!(!item.can_convert());

        // Can convert after approval
        item.approve("MANAGER");
        assert!(item.can_convert());

        // Cannot convert after conversion
        item.convert_to_po("PO-001", 1);
        assert!(!item.can_convert());
    }

    #[test]
    fn test_service_requisition() {
        let item = PurchaseRequisitionItem::service(
            1,
            "Consulting Services",
            Decimal::from(40),
            Decimal::from(150),
            "EMP-001",
        );

        assert_eq!(item.item_category, "SERVICE");
        assert_eq!(item.base.uom, "HR");
    }

    #[test]
    fn test_pr_type_properties() {
        assert!(PurchaseRequisitionType::Emergency.is_expedited());
        assert!(!PurchaseRequisitionType::Standard.is_expedited());
        assert!(PurchaseRequisitionType::Service.is_service());
        assert!(!PurchaseRequisitionType::Service.requires_goods_receipt());
        assert!(!PurchaseRequisitionType::NonStock.requires_goods_receipt());
        assert!(PurchaseRequisitionType::Standard.requires_goods_receipt());
        assert!(PurchaseRequisitionType::Framework.requires_goods_receipt());
    }

    #[test]
    fn test_priority_values() {
        assert!(RequisitionPriority::Urgent.value() > RequisitionPriority::High.value());
        assert!(RequisitionPriority::High.value() > RequisitionPriority::Normal.value());
        assert!(RequisitionPriority::Normal.value() > RequisitionPriority::Low.value());
    }
}
