//! Customer Invoice document model.
//!
//! Represents customer invoices (billing documents) in the O2C (Order-to-Cash) process flow.
//! Customer invoices create accounting entries: DR Accounts Receivable, CR Revenue.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DocumentHeader, DocumentLineItem, DocumentReference, DocumentStatus, DocumentType,
    ReferenceType,
};

/// Customer Invoice type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomerInvoiceType {
    /// Standard invoice
    #[default]
    Standard,
    /// Credit memo
    CreditMemo,
    /// Debit memo
    DebitMemo,
    /// Pro forma invoice
    ProForma,
    /// Down payment request
    DownPaymentRequest,
    /// Final invoice (settling down payment)
    FinalInvoice,
    /// Intercompany invoice
    Intercompany,
}

impl CustomerInvoiceType {
    /// Check if this type increases AR (debit).
    pub fn is_debit(&self) -> bool {
        matches!(
            self,
            Self::Standard
                | Self::DebitMemo
                | Self::DownPaymentRequest
                | Self::FinalInvoice
                | Self::Intercompany
        )
    }

    /// Check if this type decreases AR (credit).
    pub fn is_credit(&self) -> bool {
        matches!(self, Self::CreditMemo)
    }

    /// Check if this creates revenue.
    pub fn creates_revenue(&self) -> bool {
        matches!(
            self,
            Self::Standard | Self::FinalInvoice | Self::Intercompany
        )
    }
}

/// Customer Invoice payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InvoicePaymentStatus {
    /// Open - not paid
    #[default]
    Open,
    /// Partially paid
    PartiallyPaid,
    /// Fully paid
    Paid,
    /// Cleared (matched and closed)
    Cleared,
    /// Written off
    WrittenOff,
    /// In dispute
    InDispute,
    /// Sent to collection
    InCollection,
}

/// Customer Invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInvoiceItem {
    /// Base line item fields
    #[serde(flatten)]
    pub base: DocumentLineItem,

    /// Reference sales order number
    pub sales_order_id: Option<String>,

    /// Reference SO item
    pub so_item: Option<u16>,

    /// Reference delivery number
    pub delivery_id: Option<String>,

    /// Reference delivery item
    pub delivery_item: Option<u16>,

    /// Revenue account (override from material)
    pub revenue_account: Option<String>,

    /// COGS account (for statistical tracking)
    pub cogs_account: Option<String>,

    /// COGS amount (for margin calculation)
    pub cogs_amount: Decimal,

    /// Discount amount
    pub discount_amount: Decimal,

    /// Is this a service item?
    pub is_service: bool,

    /// Returns reference (if credit memo for returns)
    pub returns_reference: Option<String>,
}

impl CustomerInvoiceItem {
    /// Create a new customer invoice item.
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
            delivery_id: None,
            delivery_item: None,
            revenue_account: None,
            cogs_account: None,
            cogs_amount: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            is_service: false,
            returns_reference: None,
        }
    }

    /// Create from delivery reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_delivery(
        line_number: u16,
        description: impl Into<String>,
        quantity: Decimal,
        unit_price: Decimal,
        delivery_id: impl Into<String>,
        delivery_item: u16,
    ) -> Self {
        let mut item = Self::new(line_number, description, quantity, unit_price);
        item.delivery_id = Some(delivery_id.into());
        item.delivery_item = Some(delivery_item);
        item
    }

    /// Set material.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.base = self.base.with_material(material_id);
        self
    }

    /// Set sales order reference.
    pub fn with_sales_order(mut self, so_id: impl Into<String>, so_item: u16) -> Self {
        self.sales_order_id = Some(so_id.into());
        self.so_item = Some(so_item);
        self
    }

    /// Set COGS amount.
    pub fn with_cogs(mut self, cogs: Decimal) -> Self {
        self.cogs_amount = cogs;
        self
    }

    /// Set revenue account.
    pub fn with_revenue_account(mut self, account: impl Into<String>) -> Self {
        self.revenue_account = Some(account.into());
        self
    }

    /// Set as service item.
    pub fn as_service(mut self) -> Self {
        self.is_service = true;
        self
    }

    /// Set discount.
    pub fn with_discount(mut self, discount: Decimal) -> Self {
        self.discount_amount = discount;
        self
    }

    /// Calculate gross margin.
    pub fn gross_margin(&self) -> Decimal {
        if self.base.net_amount == Decimal::ZERO {
            return Decimal::ZERO;
        }
        ((self.base.net_amount - self.cogs_amount) / self.base.net_amount * Decimal::from(100))
            .round_dp(2)
    }
}

/// Customer Invoice document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInvoice {
    /// Document header
    pub header: DocumentHeader,

    /// Invoice type
    pub invoice_type: CustomerInvoiceType,

    /// Line items
    pub items: Vec<CustomerInvoiceItem>,

    /// Customer ID
    pub customer_id: String,

    /// Bill-to party (if different)
    pub bill_to: Option<String>,

    /// Payer (if different)
    pub payer: Option<String>,

    /// Sales organization
    pub sales_org: String,

    /// Distribution channel
    pub distribution_channel: String,

    /// Division
    pub division: String,

    /// Total net amount
    pub total_net_amount: Decimal,

    /// Total tax amount
    pub total_tax_amount: Decimal,

    /// Total gross amount
    pub total_gross_amount: Decimal,

    /// Total discount amount
    pub total_discount: Decimal,

    /// Total COGS
    pub total_cogs: Decimal,

    /// Payment terms
    pub payment_terms: String,

    /// Due date
    pub due_date: NaiveDate,

    /// Cash discount date 1
    pub discount_date_1: Option<NaiveDate>,

    /// Cash discount percent 1
    pub discount_percent_1: Option<Decimal>,

    /// Cash discount date 2
    pub discount_date_2: Option<NaiveDate>,

    /// Cash discount percent 2
    pub discount_percent_2: Option<Decimal>,

    /// Amount paid
    pub amount_paid: Decimal,

    /// Amount open (remaining)
    pub amount_open: Decimal,

    /// Payment status
    pub payment_status: InvoicePaymentStatus,

    /// Reference sales order (primary)
    pub sales_order_id: Option<String>,

    /// Reference delivery (primary)
    pub delivery_id: Option<String>,

    /// External invoice number (for customer)
    pub external_reference: Option<String>,

    /// Customer PO number
    pub customer_po_number: Option<String>,

    /// Is invoice posted?
    pub is_posted: bool,

    /// Is invoice printed/sent?
    pub is_output_complete: bool,

    /// Is this an intercompany invoice?
    pub is_intercompany: bool,

    /// Intercompany partner (company code)
    pub ic_partner: Option<String>,

    /// Dispute reason (if in dispute)
    pub dispute_reason: Option<String>,

    /// Write-off amount
    pub write_off_amount: Decimal,

    /// Write-off reason
    pub write_off_reason: Option<String>,

    /// Dunning level (0 = not dunned)
    pub dunning_level: u8,

    /// Last dunning date
    pub last_dunning_date: Option<NaiveDate>,

    /// Is invoice cancelled/reversed?
    pub is_cancelled: bool,

    /// Cancellation invoice reference
    pub cancellation_invoice: Option<String>,
}

impl CustomerInvoice {
    /// Create a new customer invoice.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invoice_id: impl Into<String>,
        company_code: impl Into<String>,
        customer_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        due_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            invoice_id,
            DocumentType::CustomerInvoice,
            company_code,
            fiscal_year,
            fiscal_period,
            document_date,
            created_by,
        )
        .with_currency("USD");

        Self {
            header,
            invoice_type: CustomerInvoiceType::Standard,
            items: Vec::new(),
            customer_id: customer_id.into(),
            bill_to: None,
            payer: None,
            sales_org: "1000".to_string(),
            distribution_channel: "10".to_string(),
            division: "00".to_string(),
            total_net_amount: Decimal::ZERO,
            total_tax_amount: Decimal::ZERO,
            total_gross_amount: Decimal::ZERO,
            total_discount: Decimal::ZERO,
            total_cogs: Decimal::ZERO,
            payment_terms: "NET30".to_string(),
            due_date,
            discount_date_1: None,
            discount_percent_1: None,
            discount_date_2: None,
            discount_percent_2: None,
            amount_paid: Decimal::ZERO,
            amount_open: Decimal::ZERO,
            payment_status: InvoicePaymentStatus::Open,
            sales_order_id: None,
            delivery_id: None,
            external_reference: None,
            customer_po_number: None,
            is_posted: false,
            is_output_complete: false,
            is_intercompany: false,
            ic_partner: None,
            dispute_reason: None,
            write_off_amount: Decimal::ZERO,
            write_off_reason: None,
            dunning_level: 0,
            last_dunning_date: None,
            is_cancelled: false,
            cancellation_invoice: None,
        }
    }

    /// Create from delivery reference.
    #[allow(clippy::too_many_arguments)]
    pub fn from_delivery(
        invoice_id: impl Into<String>,
        company_code: impl Into<String>,
        delivery_id: impl Into<String>,
        customer_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        due_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let dlv_id = delivery_id.into();
        let mut invoice = Self::new(
            invoice_id,
            company_code,
            customer_id,
            fiscal_year,
            fiscal_period,
            document_date,
            due_date,
            created_by,
        );
        invoice.delivery_id = Some(dlv_id.clone());

        // Add reference to delivery
        invoice.header.add_reference(DocumentReference::new(
            DocumentType::Delivery,
            dlv_id,
            DocumentType::CustomerInvoice,
            invoice.header.document_id.clone(),
            ReferenceType::FollowOn,
            invoice.header.company_code.clone(),
            document_date,
        ));

        invoice
    }

    /// Create a credit memo.
    pub fn credit_memo(
        invoice_id: impl Into<String>,
        company_code: impl Into<String>,
        customer_id: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let mut invoice = Self::new(
            invoice_id,
            company_code,
            customer_id,
            fiscal_year,
            fiscal_period,
            document_date,
            document_date, // Due immediately
            created_by,
        );
        invoice.invoice_type = CustomerInvoiceType::CreditMemo;
        invoice.header.document_type = DocumentType::CreditMemo;
        invoice
    }

    /// Set invoice type.
    pub fn with_invoice_type(mut self, invoice_type: CustomerInvoiceType) -> Self {
        self.invoice_type = invoice_type;
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
    pub fn with_partners(mut self, bill_to: impl Into<String>, payer: impl Into<String>) -> Self {
        self.bill_to = Some(bill_to.into());
        self.payer = Some(payer.into());
        self
    }

    /// Set payment terms with cash discount.
    pub fn with_payment_terms(
        mut self,
        terms: impl Into<String>,
        discount_days_1: Option<u16>,
        discount_percent_1: Option<Decimal>,
    ) -> Self {
        self.payment_terms = terms.into();
        if let (Some(days), Some(pct)) = (discount_days_1, discount_percent_1) {
            self.discount_date_1 =
                Some(self.header.document_date + chrono::Duration::days(days as i64));
            self.discount_percent_1 = Some(pct);
        }
        self
    }

    /// Set customer PO reference.
    pub fn with_customer_po(mut self, po_number: impl Into<String>) -> Self {
        self.customer_po_number = Some(po_number.into());
        self
    }

    /// Set as intercompany.
    pub fn as_intercompany(mut self, partner_company: impl Into<String>) -> Self {
        self.is_intercompany = true;
        self.ic_partner = Some(partner_company.into());
        self.invoice_type = CustomerInvoiceType::Intercompany;
        self
    }

    /// Add a line item.
    pub fn add_item(&mut self, item: CustomerInvoiceItem) {
        self.items.push(item);
        self.recalculate_totals();
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.total_net_amount = self.items.iter().map(|i| i.base.net_amount).sum();
        self.total_tax_amount = self.items.iter().map(|i| i.base.tax_amount).sum();
        self.total_gross_amount = self.total_net_amount + self.total_tax_amount;
        self.total_discount = self.items.iter().map(|i| i.discount_amount).sum();
        self.total_cogs = self.items.iter().map(|i| i.cogs_amount).sum();
        self.amount_open = self.total_gross_amount - self.amount_paid - self.write_off_amount;
    }

    /// Post the invoice.
    pub fn post(&mut self, user: impl Into<String>, posting_date: NaiveDate) {
        self.is_posted = true;
        self.header.posting_date = Some(posting_date);
        self.header.update_status(DocumentStatus::Posted, user);
        self.recalculate_totals();
    }

    /// Record a payment.
    pub fn record_payment(&mut self, amount: Decimal, discount_taken: Decimal) {
        self.amount_paid += amount + discount_taken;
        self.amount_open = self.total_gross_amount - self.amount_paid - self.write_off_amount;

        if self.amount_open <= Decimal::ZERO {
            self.payment_status = InvoicePaymentStatus::Paid;
        } else if self.amount_paid > Decimal::ZERO {
            self.payment_status = InvoicePaymentStatus::PartiallyPaid;
        }
    }

    /// Clear the invoice.
    pub fn clear(&mut self) {
        self.payment_status = InvoicePaymentStatus::Cleared;
        self.amount_open = Decimal::ZERO;
        self.header.update_status(DocumentStatus::Cleared, "SYSTEM");
    }

    /// Put invoice in dispute.
    pub fn dispute(&mut self, reason: impl Into<String>) {
        self.payment_status = InvoicePaymentStatus::InDispute;
        self.dispute_reason = Some(reason.into());
    }

    /// Resolve dispute.
    pub fn resolve_dispute(&mut self) {
        self.dispute_reason = None;
        if self.amount_open > Decimal::ZERO {
            self.payment_status = if self.amount_paid > Decimal::ZERO {
                InvoicePaymentStatus::PartiallyPaid
            } else {
                InvoicePaymentStatus::Open
            };
        } else {
            self.payment_status = InvoicePaymentStatus::Paid;
        }
    }

    /// Write off remaining amount.
    pub fn write_off(&mut self, amount: Decimal, reason: impl Into<String>) {
        self.write_off_amount = amount;
        self.write_off_reason = Some(reason.into());
        self.amount_open = self.total_gross_amount - self.amount_paid - self.write_off_amount;

        if self.amount_open <= Decimal::ZERO {
            self.payment_status = InvoicePaymentStatus::WrittenOff;
        }
    }

    /// Record dunning.
    pub fn record_dunning(&mut self, dunning_date: NaiveDate) {
        self.dunning_level += 1;
        self.last_dunning_date = Some(dunning_date);

        if self.dunning_level >= 4 {
            self.payment_status = InvoicePaymentStatus::InCollection;
        }
    }

    /// Cancel the invoice.
    pub fn cancel(&mut self, user: impl Into<String>, cancellation_invoice: impl Into<String>) {
        self.is_cancelled = true;
        self.cancellation_invoice = Some(cancellation_invoice.into());
        self.header.update_status(DocumentStatus::Cancelled, user);
    }

    /// Check if invoice is overdue.
    pub fn is_overdue(&self, as_of_date: NaiveDate) -> bool {
        self.payment_status == InvoicePaymentStatus::Open && as_of_date > self.due_date
    }

    /// Days past due.
    pub fn days_past_due(&self, as_of_date: NaiveDate) -> i64 {
        if as_of_date <= self.due_date {
            0
        } else {
            (as_of_date - self.due_date).num_days()
        }
    }

    /// Get aging bucket.
    pub fn aging_bucket(&self, as_of_date: NaiveDate) -> AgingBucket {
        let days = self.days_past_due(as_of_date);
        match days {
            d if d <= 0 => AgingBucket::Current,
            1..=30 => AgingBucket::Days1To30,
            31..=60 => AgingBucket::Days31To60,
            61..=90 => AgingBucket::Days61To90,
            _ => AgingBucket::Over90,
        }
    }

    /// Cash discount available.
    pub fn cash_discount_available(&self, as_of_date: NaiveDate) -> Decimal {
        if let (Some(date1), Some(pct1)) = (self.discount_date_1, self.discount_percent_1) {
            if as_of_date <= date1 {
                return self.amount_open * pct1 / Decimal::from(100);
            }
        }
        if let (Some(date2), Some(pct2)) = (self.discount_date_2, self.discount_percent_2) {
            if as_of_date <= date2 {
                return self.amount_open * pct2 / Decimal::from(100);
            }
        }
        Decimal::ZERO
    }

    /// Calculate gross margin.
    pub fn gross_margin(&self) -> Decimal {
        if self.total_net_amount == Decimal::ZERO {
            return Decimal::ZERO;
        }
        ((self.total_net_amount - self.total_cogs) / self.total_net_amount * Decimal::from(100))
            .round_dp(2)
    }

    /// Generate GL entries.
    /// DR Accounts Receivable, CR Revenue, CR Tax Payable
    pub fn generate_gl_entries(&self) -> Vec<(String, Decimal, Decimal)> {
        let mut entries = Vec::new();

        let sign = if self.invoice_type.is_debit() { 1 } else { -1 };

        // DR AR (or CR for credit memo)
        let ar_account = "120000".to_string();
        if sign > 0 {
            entries.push((ar_account, self.total_gross_amount, Decimal::ZERO));
        } else {
            entries.push((ar_account, Decimal::ZERO, self.total_gross_amount));
        }

        // CR Revenue per item (or DR for credit memo)
        for item in &self.items {
            let revenue_account = item
                .revenue_account
                .clone()
                .or_else(|| item.base.gl_account.clone())
                .unwrap_or_else(|| "400000".to_string());

            if sign > 0 {
                entries.push((revenue_account, Decimal::ZERO, item.base.net_amount));
            } else {
                entries.push((revenue_account, item.base.net_amount, Decimal::ZERO));
            }
        }

        // CR Tax (or DR for credit memo)
        if self.total_tax_amount > Decimal::ZERO {
            let tax_account = "220000".to_string();
            if sign > 0 {
                entries.push((tax_account, Decimal::ZERO, self.total_tax_amount));
            } else {
                entries.push((tax_account, self.total_tax_amount, Decimal::ZERO));
            }
        }

        entries
    }
}

/// AR aging bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgingBucket {
    /// Not yet due
    Current,
    /// 1-30 days past due
    Days1To30,
    /// 31-60 days past due
    Days31To60,
    /// 61-90 days past due
    Days61To90,
    /// Over 90 days past due
    Over90,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_invoice_creation() {
        let invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        assert_eq!(invoice.customer_id, "C-000001");
        assert_eq!(invoice.payment_status, InvoicePaymentStatus::Open);
    }

    #[test]
    fn test_customer_invoice_from_delivery() {
        let invoice = CustomerInvoice::from_delivery(
            "CI-1000-0000000001",
            "1000",
            "DLV-1000-0000000001",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        assert_eq!(invoice.delivery_id, Some("DLV-1000-0000000001".to_string()));
        assert_eq!(invoice.header.document_references.len(), 1);
    }

    #[test]
    fn test_invoice_items() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        let item = CustomerInvoiceItem::from_delivery(
            1,
            "Product A",
            Decimal::from(100),
            Decimal::from(50),
            "DLV-1000-0000000001",
            1,
        )
        .with_material("MAT-001")
        .with_cogs(Decimal::from(3000));

        invoice.add_item(item);

        assert_eq!(invoice.total_net_amount, Decimal::from(5000));
        assert_eq!(invoice.total_cogs, Decimal::from(3000));
        assert_eq!(invoice.gross_margin(), Decimal::from(40)); // (5000-3000)/5000 * 100 = 40%
    }

    #[test]
    fn test_payment_recording() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        let item = CustomerInvoiceItem::new(1, "Product", Decimal::from(10), Decimal::from(100));
        invoice.add_item(item);
        invoice.post("BILLING", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        assert_eq!(invoice.amount_open, Decimal::from(1000));

        // Partial payment
        invoice.record_payment(Decimal::from(500), Decimal::ZERO);
        assert_eq!(invoice.amount_paid, Decimal::from(500));
        assert_eq!(invoice.amount_open, Decimal::from(500));
        assert_eq!(invoice.payment_status, InvoicePaymentStatus::PartiallyPaid);

        // Final payment
        invoice.record_payment(Decimal::from(500), Decimal::ZERO);
        assert_eq!(invoice.payment_status, InvoicePaymentStatus::Paid);
    }

    #[test]
    fn test_cash_discount() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        )
        .with_payment_terms("2/10 NET 30", Some(10), Some(Decimal::from(2)));

        let item = CustomerInvoiceItem::new(1, "Product", Decimal::from(10), Decimal::from(100));
        invoice.add_item(item);
        invoice.post("BILLING", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Within discount period
        let discount =
            invoice.cash_discount_available(NaiveDate::from_ymd_opt(2024, 1, 20).unwrap());
        assert_eq!(discount, Decimal::from(20)); // 2% of 1000

        // After discount period
        let discount =
            invoice.cash_discount_available(NaiveDate::from_ymd_opt(2024, 1, 30).unwrap());
        assert_eq!(discount, Decimal::ZERO);
    }

    #[test]
    fn test_aging() {
        let invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        // Not overdue
        assert!(!invoice.is_overdue(NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()));
        assert_eq!(
            invoice.aging_bucket(NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()),
            AgingBucket::Current
        );

        // 15 days overdue
        assert!(invoice.is_overdue(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()));
        assert_eq!(
            invoice.aging_bucket(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
            AgingBucket::Days1To30
        );

        // 45 days overdue
        assert_eq!(
            invoice.aging_bucket(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            AgingBucket::Days31To60
        );

        // 100 days overdue
        assert_eq!(
            invoice.aging_bucket(NaiveDate::from_ymd_opt(2024, 5, 25).unwrap()),
            AgingBucket::Over90
        );
    }

    #[test]
    fn test_gl_entry_generation() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        let mut item =
            CustomerInvoiceItem::new(1, "Product", Decimal::from(10), Decimal::from(100));
        item.base.tax_amount = Decimal::from(100);
        invoice.add_item(item);
        invoice.recalculate_totals();

        let entries = invoice.generate_gl_entries();
        assert_eq!(entries.len(), 3);

        // DR AR
        assert_eq!(entries[0].0, "120000");
        assert_eq!(entries[0].1, Decimal::from(1100)); // 1000 net + 100 tax

        // CR Revenue
        assert_eq!(entries[1].0, "400000");
        assert_eq!(entries[1].2, Decimal::from(1000));

        // CR Tax
        assert_eq!(entries[2].0, "220000");
        assert_eq!(entries[2].2, Decimal::from(100));
    }

    #[test]
    fn test_credit_memo_gl_entries() {
        let mut invoice = CustomerInvoice::credit_memo(
            "CM-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let item = CustomerInvoiceItem::new(1, "Return", Decimal::from(5), Decimal::from(100));
        invoice.add_item(item);

        let entries = invoice.generate_gl_entries();

        // CR AR (credit reduces AR)
        assert_eq!(entries[0].0, "120000");
        assert_eq!(entries[0].2, Decimal::from(500)); // Credit side

        // DR Revenue (credit reduces revenue)
        assert_eq!(entries[1].0, "400000");
        assert_eq!(entries[1].1, Decimal::from(500)); // Debit side
    }

    #[test]
    fn test_write_off() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        let item = CustomerInvoiceItem::new(1, "Product", Decimal::from(10), Decimal::from(100));
        invoice.add_item(item);
        invoice.post("BILLING", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        invoice.record_payment(Decimal::from(900), Decimal::ZERO);
        invoice.write_off(Decimal::from(100), "Small balance write-off");

        assert_eq!(invoice.write_off_amount, Decimal::from(100));
        assert_eq!(invoice.amount_open, Decimal::ZERO);
        assert_eq!(invoice.payment_status, InvoicePaymentStatus::WrittenOff);
    }

    #[test]
    fn test_dunning() {
        let mut invoice = CustomerInvoice::new(
            "CI-1000-0000000001",
            "1000",
            "C-000001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "JSMITH",
        );

        invoice.record_dunning(NaiveDate::from_ymd_opt(2024, 2, 20).unwrap());
        assert_eq!(invoice.dunning_level, 1);

        invoice.record_dunning(NaiveDate::from_ymd_opt(2024, 3, 5).unwrap());
        invoice.record_dunning(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap());
        invoice.record_dunning(NaiveDate::from_ymd_opt(2024, 4, 5).unwrap());

        assert_eq!(invoice.dunning_level, 4);
        assert_eq!(invoice.payment_status, InvoicePaymentStatus::InCollection);
    }
}
