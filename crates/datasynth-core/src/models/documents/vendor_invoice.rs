//! Vendor Invoice document model.
//!
//! Represents vendor invoices in the P2P (Procure-to-Pay) process flow.
//! Vendor invoices create accounting entries: DR Expense/GR-IR, CR AP.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DocumentHeader, DocumentLineItem, DocumentReference, DocumentStatus, DocumentType,
    ReferenceType,
};

/// Vendor Invoice type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorInvoiceType {
    /// Standard invoice against PO
    #[default]
    Standard,
    /// Credit memo from vendor
    CreditMemo,
    /// Subsequent debit/credit
    SubsequentAdjustment,
    /// Down payment request
    DownPaymentRequest,
    /// Invoice plan
    InvoicePlan,
    /// Recurring invoice
    Recurring,
}

/// Invoice verification status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceVerificationStatus {
    /// Not verified
    #[default]
    Unverified,
    /// Three-way match passed
    ThreeWayMatchPassed,
    /// Three-way match failed
    ThreeWayMatchFailed,
    /// Manually approved despite mismatch
    ManuallyApproved,
    /// Blocked for payment
    BlockedForPayment,
}

/// Vendor Invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorInvoiceItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Reference PO number
    pub po_number: Option<String>,

    /// Reference PO item
    pub po_item: Option<u16>,

    /// Reference GR number
    pub gr_number: Option<String>,

    /// Reference GR item
    pub gr_item: Option<u16>,

    /// Invoiced quantity
    pub invoiced_quantity: Decimal,

    /// Three-way match status
    pub match_status: ThreeWayMatchStatus,

    /// Price variance amount
    pub price_variance: Decimal,

    /// Quantity variance
    pub quantity_variance: Decimal,

    /// Tax code
    pub tax_code: Option<String>,

    /// Withholding tax applicable
    pub withholding_tax: bool,

    /// Withholding tax amount
    pub withholding_tax_amount: Decimal,
}

/// Three-way match status for invoice line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThreeWayMatchStatus {
    /// Not applicable (no PO)
    #[default]
    NotApplicable,
    /// Match passed
    Matched,
    /// Price mismatch
    PriceMismatch,
    /// Quantity mismatch
    QuantityMismatch,
    /// Both price and quantity mismatch
    BothMismatch,
    /// GR not received
    GrNotReceived,
}

impl VendorInvoiceItem {
    /// Create a new vendor invoice item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
    ) -> Self {
        let base = DocumentLineItem::new(line_number, description, quantity, unit_price);
        Self {
            base,
            po_number: None,
            po_item: None,
            gr_number: None,
            gr_item: None,
            invoiced_quantity: quantity,
            match_status: ThreeWayMatchStatus::NotApplicable,
            price_variance: Decimal::ZERO,
            quantity_variance: Decimal::ZERO,
            tax_code: None,
            withholding_tax: false,
            withholding_tax_amount: Decimal::ZERO,
        }
    }

    /// Create from PO/GR reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_po_gr(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        po_number: impl Into<String>,
        po_item: u16,
        gr_number: Option<String>,
        gr_item: Option<u16>,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price);
        item.po_number = Some(po_number.into());
        item.po_item = Some(po_item);
        item.gr_number = gr_number;
        item.gr_item = gr_item;
        item
    }

    /// Set GL account.
    pub fn with_gl_account(mut self, account: impl Into<String>) -> Self {
        self.base = self.base.with_gl_account(account);
        self
    }

    /// Set cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.base = self.base.with_cost_center(cost_center);
        self
    }

    /// Set tax.
    pub fn with_tax(mut self, tax_code: impl Into<String>, tax_amount: Decimal) -> Self {
        self.tax_code = Some(tax_code.into());
        self.base = self.base.with_tax(tax_amount);
        self
    }

    /// Set withholding tax.
    pub fn with_withholding_tax(mut self, amount: Decimal) -> Self {
        self.withholding_tax = true;
        self.withholding_tax_amount = amount;
        self
    }

    /// Set match status.
    pub fn with_match_status(mut self, status: ThreeWayMatchStatus) -> Self {
        self.match_status = status;
        self
    }

    /// Calculate price variance.
    pub fn calculate_price_variance(&mut self, po_price: Decimal) {
        self.price_variance = (self.base.unit_price - po_price) * self.base.quantity;
    }

    /// Calculate quantity variance.
    pub fn calculate_quantity_variance(&mut self, gr_quantity: Decimal) {
        self.quantity_variance = self.base.quantity - gr_quantity;
    }
}

/// Vendor Invoice document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorInvoice {
    /// Document header
    pub header: DocumentHeader,

    /// Invoice type
    pub invoice_type: VendorInvoiceType,

    /// Vendor ID
    pub vendor_id: String,

    /// Vendor invoice number (external reference)
    pub vendor_invoice_number: String,

    /// Invoice date from vendor
    pub invoice_date: NaiveDate,

    /// Line items
    pub items: Vec<VendorInvoiceItem>,

    /// Net amount
    pub net_amount: Decimal,

    /// Tax amount
    pub tax_amount: Decimal,

    /// Gross amount
    pub gross_amount: Decimal,

    /// Withholding tax amount
    pub withholding_tax_amount: Decimal,

    /// Amount to pay (gross - withholding)
    pub payable_amount: Decimal,

    /// Payment terms
    pub payment_terms: String,

    /// Due date for payment
    pub due_date: NaiveDate,

    /// Discount due date (for early payment)
    pub discount_due_date: Option<NaiveDate>,

    /// Cash discount percentage
    pub cash_discount_percent: Decimal,

    /// Cash discount amount
    pub cash_discount_amount: Decimal,

    /// Verification status
    pub verification_status: InvoiceVerificationStatus,

    /// Is blocked for payment?
    pub payment_block: bool,

    /// Payment block reason
    pub payment_block_reason: Option<String>,

    /// Reference PO (primary)
    pub purchase_order_id: Option<String>,

    /// Reference GR (primary)
    pub goods_receipt_id: Option<String>,

    /// Is this invoice paid?
    pub is_paid: bool,

    /// Amount paid
    pub amount_paid: Decimal,

    /// Remaining balance
    pub balance: Decimal,

    /// Payment document references
    pub payment_references: Vec<String>,

    /// Baseline date for payment
    pub baseline_date: NaiveDate,
}

impl VendorInvoice {
    /// Create a new vendor invoice.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invoice_id: impl Into<String>,
        company_code: impl Into<String>,
        vendor_id: impl Into<String>,
        vendor_invoice_number: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        invoice_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            invoice_id,
            DocumentType::VendorInvoice,
            company_code,
            fiscal_year,
            fiscal_period,
            invoice_date,
            created_by,
        );

        let due_date = invoice_date + chrono::Duration::days(30);

        Self {
            header,
            invoice_type: VendorInvoiceType::Standard,
            vendor_id: vendor_id.into(),
            vendor_invoice_number: vendor_invoice_number.into(),
            invoice_date,
            items: Vec::new(),
            net_amount: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            gross_amount: Decimal::ZERO,
            withholding_tax_amount: Decimal::ZERO,
            payable_amount: Decimal::ZERO,
            payment_terms: "NET30".to_string(),
            due_date,
            discount_due_date: None,
            cash_discount_percent: Decimal::ZERO,
            cash_discount_amount: Decimal::ZERO,
            verification_status: InvoiceVerificationStatus::Unverified,
            payment_block: false,
            payment_block_reason: None,
            purchase_order_id: None,
            goods_receipt_id: None,
            is_paid: false,
            amount_paid: Decimal::ZERO,
            balance: Decimal::ZERO,
            payment_references: Vec::new(),
            baseline_date: invoice_date,
        }
    }

    /// Create invoice referencing PO and GR.
    #[allow(clippy::too_many_arguments)]
    pub fn from_po_gr(
        invoice_id: impl Into<String>,
        company_code: impl Into<String>,
        vendor_id: impl Into<String>,
        vendor_invoice_number: impl Into<String>,
        po_id: impl Into<String>,
        gr_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        invoice_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let po = po_id.into();
        let gr = gr_id.into();
        let cc = company_code.into();

        let mut invoice = Self::new(
            invoice_id,
            &cc,
            vendor_id,
            vendor_invoice_number,
            fiscal_year,
            fiscal_period,
            invoice_date,
            created_by,
        );

        invoice.purchase_order_id = Some(po.clone());
        invoice.goods_receipt_id = Some(gr.clone());

        // Add references
        invoice.header.add_reference(DocumentReference::new(
            DocumentType::PurchaseOrder,
            po,
            DocumentType::VendorInvoice,
            invoice.header.document_id.clone(),
            ReferenceType::FollowOn,
            &cc,
            invoice_date,
        ));

        invoice.header.add_reference(DocumentReference::new(
            DocumentType::GoodsReceipt,
            gr,
            DocumentType::VendorInvoice,
            invoice.header.document_id.clone(),
            ReferenceType::FollowOn,
            cc,
            invoice_date,
        ));

        invoice
    }

    /// Set invoice type.
    pub fn with_invoice_type(mut self, invoice_type: VendorInvoiceType) -> Self {
        self.invoice_type = invoice_type;
        self
    }

    /// Set payment terms.
    pub fn with_payment_terms(mut self, terms: impl Into<String>, due_days: i64) -> Self {
        self.payment_terms = terms.into();
        self.due_date = self.invoice_date + chrono::Duration::days(due_days);
        self
    }

    /// Set cash discount.
    pub fn with_cash_discount(mut self, percent: Decimal, discount_days: i64) -> Self {
        self.cash_discount_percent = percent;
        self.discount_due_date = Some(self.invoice_date + chrono::Duration::days(discount_days));
        self
    }

    /// Block payment.
    pub fn block_payment(&mut self, reason: impl Into<String>) {
        self.payment_block = true;
        self.payment_block_reason = Some(reason.into());
        self.verification_status = InvoiceVerificationStatus::BlockedForPayment;
    }

    /// Unblock payment.
    pub fn unblock_payment(&mut self) {
        self.payment_block = false;
        self.payment_block_reason = None;
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: VendorInvoiceItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.net_amount = self.items.iter().map(|i| i.base.net_amount).sum();
        self.tax_amount = self.items.iter().map(|i| i.base.tax_amount).sum();
        self.withholding_tax_amount = self.items.iter().map(|i| i.withholding_tax_amount).sum();
        self.gross_amount = self.net_amount + self.tax_amount;
        self.payable_amount = self.gross_amount - self.withholding_tax_amount;
        self.cash_discount_amount =
            self.net_amount * self.cash_discount_percent / Decimal::from(100);
        self.balance = self.payable_amount - self.amount_paid;
    }

    /// Record payment.
    pub fn record_payment(&mut self, amount: Decimal, payment_doc_id: impl Into<String>) {
        self.amount_paid += amount;
        self.balance = self.payable_amount - self.amount_paid;
        self.payment_references.push(payment_doc_id.into());

        if self.balance <= Decimal::ZERO {
            self.is_paid = true;
            self.header.update_status(DocumentStatus::Cleared, "SYSTEM");
        }
    }

    /// Post the invoice.
    pub fn post(&mut self, user: impl Into<String>, posting_date: NaiveDate) {
        self.header.posting_date = Some(posting_date);
        self.header.update_status(DocumentStatus::Posted, user);
    }

    /// Verify the invoice (three-way match).
    pub fn verify(&mut self, passed: bool) {
        self.verification_status = if passed {
            InvoiceVerificationStatus::ThreeWayMatchPassed
        } else {
            InvoiceVerificationStatus::ThreeWayMatchFailed
        };
    }

    /// Check if discount is still available.
    pub fn discount_available(&self, as_of_date: NaiveDate) -> bool {
        self.discount_due_date.is_some_and(|d| as_of_date <= d)
    }

    /// Get amount with discount.
    pub fn discounted_amount(&self, as_of_date: NaiveDate) -> Decimal {
        if self.discount_available(as_of_date) {
            self.payable_amount - self.cash_discount_amount
        } else {
            self.payable_amount
        }
    }

    /// Generate GL entries for invoice posting.
    /// DR Expense/GR-IR Clearing
    /// CR AP (Vendor)
    pub fn generate_gl_entries(&self) -> Vec<(String, Decimal, Decimal, Option<String>)> {
        let mut entries = Vec::new();

        // Debit entries (expenses or GR/IR clearing)
        for item in &self.items {
            let account = if item.po_number.is_some() && item.gr_number.is_some() {
                "290000".to_string() // GR/IR Clearing
            } else {
                item.base
                    .gl_account
                    .clone()
                    .unwrap_or_else(|| "600000".to_string())
            };

            entries.push((
                account,
                item.base.net_amount,
                Decimal::ZERO,
                item.base.cost_center.clone(),
            ));
        }

        // Tax entry (if applicable)
        if self.tax_amount > Decimal::ZERO {
            entries.push((
                "154000".to_string(), // Input VAT
                self.tax_amount,
                Decimal::ZERO,
                None,
            ));
        }

        // Credit entry (AP)
        entries.push((
            "210000".to_string(), // AP
            Decimal::ZERO,
            self.gross_amount,
            None,
        ));

        entries
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_invoice_creation() {
        let invoice = VendorInvoice::new(
            "VI-1000-0000000001",
            "1000",
            "V-000001",
            "INV-2024-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(invoice.vendor_id, "V-000001");
        assert_eq!(
            invoice.due_date,
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()
        );
    }

    #[test]
    fn test_vendor_invoice_with_items() {
        let mut invoice = VendorInvoice::new(
            "VI-1000-0000000001",
            "1000",
            "V-000001",
            "INV-2024-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        invoice.add_item(
            VendorInvoiceItem::new(1, "Office Supplies", Decimal::from(10), Decimal::from(25))
                .with_tax("VAT10", Decimal::from(25)),
        );

        assert_eq!(invoice.net_amount, Decimal::from(250));
        assert_eq!(invoice.tax_amount, Decimal::from(25));
        assert_eq!(invoice.gross_amount, Decimal::from(275));
    }

    #[test]
    fn test_payment_recording() {
        let mut invoice = VendorInvoice::new(
            "VI-1000-0000000001",
            "1000",
            "V-000001",
            "INV-2024-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        invoice.add_item(VendorInvoiceItem::new(
            1,
            "Test",
            Decimal::from(1),
            Decimal::from(1000),
        ));

        invoice.record_payment(Decimal::from(500), "PAY-001");
        assert_eq!(invoice.balance, Decimal::from(500));
        assert!(!invoice.is_paid);

        invoice.record_payment(Decimal::from(500), "PAY-002");
        assert_eq!(invoice.balance, Decimal::ZERO);
        assert!(invoice.is_paid);
    }

    #[test]
    fn test_cash_discount() {
        let invoice = VendorInvoice::new(
            "VI-1000-0000000001",
            "1000",
            "V-000001",
            "INV-2024-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        )
        .with_cash_discount(Decimal::from(2), 10);

        let early_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let late_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();

        assert!(invoice.discount_available(early_date));
        assert!(!invoice.discount_available(late_date));
    }
}
