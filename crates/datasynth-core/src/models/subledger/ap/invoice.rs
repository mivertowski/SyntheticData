//! AP Invoice (vendor invoice) model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{
    ClearingInfo, CurrencyAmount, GLReference, PaymentTerms, SubledgerDocumentStatus, TaxInfo,
};

/// AP Invoice (vendor invoice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APInvoice {
    /// Unique invoice number (internal).
    pub invoice_number: String,
    /// Vendor's invoice number.
    pub vendor_invoice_number: String,
    /// Company code.
    pub company_code: String,
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Invoice date (vendor's date).
    pub invoice_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Document date (receipt date).
    pub document_date: NaiveDate,
    /// Due date.
    pub due_date: NaiveDate,
    /// Baseline date for payment terms.
    pub baseline_date: NaiveDate,
    /// Invoice type.
    pub invoice_type: APInvoiceType,
    /// Invoice status.
    pub status: SubledgerDocumentStatus,
    /// Invoice lines.
    pub lines: Vec<APInvoiceLine>,
    /// Net amount (before tax).
    pub net_amount: CurrencyAmount,
    /// Tax amount.
    pub tax_amount: CurrencyAmount,
    /// Gross amount (after tax).
    pub gross_amount: CurrencyAmount,
    /// Amount paid.
    pub amount_paid: Decimal,
    /// Amount remaining.
    pub amount_remaining: Decimal,
    /// Payment terms.
    pub payment_terms: PaymentTerms,
    /// Tax details.
    pub tax_details: Vec<TaxInfo>,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Clearing information.
    pub clearing_info: Vec<ClearingInfo>,
    /// Three-way match status.
    pub match_status: MatchStatus,
    /// Reference purchase order.
    pub reference_po: Option<String>,
    /// Reference goods receipt.
    pub reference_gr: Option<String>,
    /// Payment block reason.
    pub payment_block: Option<PaymentBlockReason>,
    /// Withholding tax applicable.
    pub withholding_tax: Option<WithholdingTax>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Last modified timestamp.
    pub modified_at: Option<DateTime<Utc>>,
    /// Notes.
    pub notes: Option<String>,
}

impl APInvoice {
    /// Creates a new AP invoice.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invoice_number: String,
        vendor_invoice_number: String,
        company_code: String,
        vendor_id: String,
        vendor_name: String,
        invoice_date: NaiveDate,
        payment_terms: PaymentTerms,
        currency: String,
    ) -> Self {
        let due_date = payment_terms.calculate_due_date(invoice_date);

        Self {
            invoice_number,
            vendor_invoice_number,
            company_code,
            vendor_id,
            vendor_name,
            invoice_date,
            posting_date: invoice_date,
            document_date: invoice_date,
            due_date,
            baseline_date: invoice_date,
            invoice_type: APInvoiceType::Standard,
            status: SubledgerDocumentStatus::Open,
            lines: Vec::new(),
            net_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency.clone()),
            tax_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency.clone()),
            gross_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency),
            amount_paid: Decimal::ZERO,
            amount_remaining: Decimal::ZERO,
            payment_terms,
            tax_details: Vec::new(),
            gl_reference: None,
            clearing_info: Vec::new(),
            match_status: MatchStatus::NotMatched,
            reference_po: None,
            reference_gr: None,
            payment_block: None,
            withholding_tax: None,
            created_at: Utc::now(),
            created_by: None,
            modified_at: None,
            notes: None,
        }
    }

    /// Adds an invoice line.
    pub fn add_line(&mut self, line: APInvoiceLine) {
        self.lines.push(line);
        self.recalculate_totals();
    }

    /// Recalculates totals from lines.
    pub fn recalculate_totals(&mut self) {
        let net_total: Decimal = self.lines.iter().map(|l| l.net_amount).sum();
        let tax_total: Decimal = self.lines.iter().map(|l| l.tax_amount).sum();
        let gross_total = net_total + tax_total;

        self.net_amount.document_amount = net_total;
        self.net_amount.local_amount = net_total * self.net_amount.exchange_rate;
        self.tax_amount.document_amount = tax_total;
        self.tax_amount.local_amount = tax_total * self.tax_amount.exchange_rate;
        self.gross_amount.document_amount = gross_total;
        self.gross_amount.local_amount = gross_total * self.gross_amount.exchange_rate;
        self.amount_remaining = gross_total - self.amount_paid;
    }

    /// Applies a payment to the invoice.
    pub fn apply_payment(&mut self, amount: Decimal, clearing: ClearingInfo) {
        self.amount_paid += amount;
        self.amount_remaining = self.gross_amount.document_amount - self.amount_paid;
        self.clearing_info.push(clearing);

        self.status = if self.amount_remaining <= Decimal::ZERO {
            SubledgerDocumentStatus::Cleared
        } else {
            SubledgerDocumentStatus::PartiallyCleared
        };

        self.modified_at = Some(Utc::now());
    }

    /// Checks if invoice is overdue.
    pub fn is_overdue(&self, as_of_date: NaiveDate) -> bool {
        self.status == SubledgerDocumentStatus::Open && as_of_date > self.due_date
    }

    /// Calculates days overdue.
    pub fn days_overdue(&self, as_of_date: NaiveDate) -> i64 {
        if self.is_overdue(as_of_date) {
            (as_of_date - self.due_date).num_days()
        } else {
            0
        }
    }

    /// Gets discount amount if paid by discount date.
    pub fn available_discount(&self, payment_date: NaiveDate) -> Decimal {
        self.payment_terms.calculate_discount(
            self.gross_amount.document_amount,
            payment_date,
            self.baseline_date,
        )
    }

    /// Sets purchase order reference.
    pub fn with_po_reference(mut self, po_number: String) -> Self {
        self.reference_po = Some(po_number);
        self
    }

    /// Sets goods receipt reference.
    pub fn with_gr_reference(mut self, gr_number: String) -> Self {
        self.reference_gr = Some(gr_number);
        self
    }

    /// Sets payment block.
    pub fn block_payment(&mut self, reason: PaymentBlockReason) {
        self.payment_block = Some(reason);
    }

    /// Removes payment block.
    pub fn unblock_payment(&mut self) {
        self.payment_block = None;
    }

    /// Checks if payment is blocked.
    pub fn is_blocked(&self) -> bool {
        self.payment_block.is_some()
    }

    /// Sets three-way match status.
    pub fn set_match_status(&mut self, status: MatchStatus) {
        let should_block = matches!(
            &status,
            MatchStatus::MatchedWithVariance { .. } | MatchStatus::NotMatched
        );
        self.match_status = status;
        if should_block {
            self.payment_block = Some(PaymentBlockReason::MatchException);
        }
    }

    /// Checks if ready for payment.
    pub fn is_payable(&self) -> bool {
        !self.is_blocked()
            && self.status == SubledgerDocumentStatus::Open
            && matches!(
                self.match_status,
                MatchStatus::Matched | MatchStatus::NotRequired
            )
    }

    /// Sets withholding tax.
    pub fn with_withholding_tax(mut self, wht: WithholdingTax) -> Self {
        self.withholding_tax = Some(wht);
        self
    }

    /// Gets net payable amount (after withholding).
    pub fn net_payable(&self) -> Decimal {
        let wht_amount = self
            .withholding_tax
            .as_ref()
            .map(|w| w.amount)
            .unwrap_or_default();
        self.amount_remaining - wht_amount
    }

    /// Reverses the invoice.
    pub fn reverse(&mut self, reversal_date: NaiveDate, reason: String) {
        self.status = SubledgerDocumentStatus::Reversed;
        self.notes = Some(format!(
            "{}Reversed on {}: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{}. ", n))
                .unwrap_or_default(),
            reversal_date,
            reason
        ));
        self.modified_at = Some(Utc::now());
    }
}

/// Type of AP invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum APInvoiceType {
    /// Standard invoice.
    #[default]
    Standard,
    /// Down payment request.
    DownPayment,
    /// Credit memo (negative invoice).
    CreditMemo,
    /// Recurring invoice.
    Recurring,
    /// Intercompany invoice.
    Intercompany,
    /// Service invoice (no goods receipt).
    Service,
    /// Expense reimbursement.
    Expense,
}

/// Three-way match status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MatchStatus {
    /// Not yet matched.
    #[default]
    NotMatched,
    /// Fully matched (PO = GR = Invoice).
    Matched,
    /// Matched with variance.
    MatchedWithVariance {
        /// Price variance.
        price_variance: Decimal,
        /// Quantity variance.
        quantity_variance: Decimal,
    },
    /// Two-way match only (no GR required).
    TwoWayMatched,
    /// Match not required (e.g., expense invoice).
    NotRequired,
}

/// Payment block reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentBlockReason {
    /// Quality issue.
    QualityHold,
    /// Price variance.
    PriceVariance,
    /// Quantity variance.
    QuantityVariance,
    /// Missing documentation.
    MissingDocumentation,
    /// Under review.
    Review,
    /// Match exception.
    MatchException,
    /// Duplicate suspected.
    DuplicateSuspect,
    /// Manual block.
    ManualBlock,
    /// Other.
    Other,
}

/// Withholding tax information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithholdingTax {
    /// Withholding tax type.
    pub wht_type: String,
    /// Withholding tax rate.
    pub rate: Decimal,
    /// Base amount.
    pub base_amount: Decimal,
    /// Withholding amount.
    pub amount: Decimal,
}

impl WithholdingTax {
    /// Creates new withholding tax.
    pub fn new(wht_type: String, rate: Decimal, base_amount: Decimal) -> Self {
        let amount = (base_amount * rate / dec!(100)).round_dp(2);
        Self {
            wht_type,
            rate,
            base_amount,
            amount,
        }
    }
}

/// AP invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APInvoiceLine {
    /// Line number.
    pub line_number: u32,
    /// Material/service ID.
    pub material_id: Option<String>,
    /// Description.
    pub description: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit of measure.
    pub unit: String,
    /// Unit price.
    pub unit_price: Decimal,
    /// Net amount.
    pub net_amount: Decimal,
    /// Tax code.
    pub tax_code: Option<String>,
    /// Tax rate.
    pub tax_rate: Decimal,
    /// Tax amount.
    pub tax_amount: Decimal,
    /// Gross amount.
    pub gross_amount: Decimal,
    /// GL account.
    pub gl_account: String,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Internal order.
    pub internal_order: Option<String>,
    /// WBS element (project).
    pub wbs_element: Option<String>,
    /// Asset number (for asset acquisitions).
    pub asset_number: Option<String>,
    /// Reference PO line.
    pub po_line: Option<u32>,
    /// Reference GR line.
    pub gr_line: Option<u32>,
}

impl APInvoiceLine {
    /// Creates a new invoice line.
    pub fn new(
        line_number: u32,
        description: String,
        quantity: Decimal,
        unit: String,
        unit_price: Decimal,
        gl_account: String,
    ) -> Self {
        let net_amount = (quantity * unit_price).round_dp(2);
        Self {
            line_number,
            material_id: None,
            description,
            quantity,
            unit,
            unit_price,
            net_amount,
            tax_code: None,
            tax_rate: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            gross_amount: net_amount,
            gl_account,
            cost_center: None,
            internal_order: None,
            wbs_element: None,
            asset_number: None,
            po_line: None,
            gr_line: None,
        }
    }

    /// Sets tax information.
    pub fn with_tax(mut self, tax_code: String, tax_rate: Decimal) -> Self {
        self.tax_code = Some(tax_code);
        self.tax_rate = tax_rate;
        self.tax_amount = (self.net_amount * tax_rate / dec!(100)).round_dp(2);
        self.gross_amount = self.net_amount + self.tax_amount;
        self
    }

    /// Sets cost center.
    pub fn with_cost_center(mut self, cost_center: String) -> Self {
        self.cost_center = Some(cost_center);
        self
    }

    /// Sets PO reference.
    pub fn with_po_reference(mut self, po_line: u32) -> Self {
        self.po_line = Some(po_line);
        self
    }

    /// Sets asset number.
    pub fn with_asset(mut self, asset_number: String) -> Self {
        self.asset_number = Some(asset_number);
        self
    }
}

/// Summary of open AP items for a vendor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAPSummary {
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Total open amount.
    pub total_open: Decimal,
    /// Total overdue amount.
    pub total_overdue: Decimal,
    /// Number of open invoices.
    pub open_invoice_count: u32,
    /// Amount coming due (next 7 days).
    pub coming_due_7d: Decimal,
    /// Amount coming due (next 30 days).
    pub coming_due_30d: Decimal,
    /// Available discount amount.
    pub available_discount: Decimal,
}

impl VendorAPSummary {
    /// Creates from a list of invoices.
    pub fn from_invoices(
        vendor_id: String,
        vendor_name: String,
        invoices: &[APInvoice],
        as_of_date: NaiveDate,
    ) -> Self {
        let open_invoices: Vec<_> = invoices
            .iter()
            .filter(|i| {
                i.vendor_id == vendor_id
                    && matches!(
                        i.status,
                        SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                    )
            })
            .collect();

        let total_open: Decimal = open_invoices.iter().map(|i| i.amount_remaining).sum();
        let total_overdue: Decimal = open_invoices
            .iter()
            .filter(|i| i.is_overdue(as_of_date))
            .map(|i| i.amount_remaining)
            .sum();

        let due_7d = as_of_date + chrono::Duration::days(7);
        let due_30d = as_of_date + chrono::Duration::days(30);

        let coming_due_7d: Decimal = open_invoices
            .iter()
            .filter(|i| i.due_date <= due_7d && i.due_date > as_of_date)
            .map(|i| i.amount_remaining)
            .sum();

        let coming_due_30d: Decimal = open_invoices
            .iter()
            .filter(|i| i.due_date <= due_30d && i.due_date > as_of_date)
            .map(|i| i.amount_remaining)
            .sum();

        let available_discount: Decimal = open_invoices
            .iter()
            .map(|i| i.available_discount(as_of_date))
            .sum();

        Self {
            vendor_id,
            vendor_name,
            total_open,
            total_overdue,
            open_invoice_count: open_invoices.len() as u32,
            coming_due_7d,
            coming_due_30d,
            available_discount,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_invoice() -> APInvoice {
        let mut invoice = APInvoice::new(
            "AP001".to_string(),
            "VINV-2024-001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            PaymentTerms::two_ten_net_30(),
            "USD".to_string(),
        );

        let line = APInvoiceLine::new(
            1,
            "Office Supplies".to_string(),
            dec!(100),
            "EA".to_string(),
            dec!(10),
            "5000".to_string(),
        )
        .with_tax("VAT".to_string(), dec!(10));

        invoice.add_line(line);
        invoice
    }

    #[test]
    fn test_invoice_totals() {
        let invoice = create_test_invoice();
        assert_eq!(invoice.net_amount.document_amount, dec!(1000));
        assert_eq!(invoice.tax_amount.document_amount, dec!(100));
        assert_eq!(invoice.gross_amount.document_amount, dec!(1100));
    }

    #[test]
    fn test_discount_calculation() {
        let invoice = create_test_invoice();
        let early_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let late_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();

        let early_discount = invoice.available_discount(early_date);
        let late_discount = invoice.available_discount(late_date);

        assert_eq!(early_discount, dec!(22)); // 2% of 1100
        assert_eq!(late_discount, Decimal::ZERO);
    }

    #[test]
    fn test_payment_block() {
        let mut invoice = create_test_invoice();
        // Set match status to NotRequired so invoice is payable
        invoice.set_match_status(MatchStatus::NotRequired);
        assert!(invoice.is_payable());

        invoice.block_payment(PaymentBlockReason::QualityHold);
        assert!(!invoice.is_payable());
        assert!(invoice.is_blocked());

        invoice.unblock_payment();
        assert!(invoice.is_payable());
    }

    #[test]
    fn test_withholding_tax() {
        let invoice = create_test_invoice().with_withholding_tax(WithholdingTax::new(
            "WHT10".to_string(),
            dec!(10),
            dec!(1000),
        ));

        assert_eq!(invoice.withholding_tax.as_ref().unwrap().amount, dec!(100));
        assert_eq!(invoice.net_payable(), dec!(1000)); // 1100 - 100 WHT
    }
}
