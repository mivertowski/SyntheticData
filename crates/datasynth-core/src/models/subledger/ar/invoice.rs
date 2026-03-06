//! AR Invoice model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{
    ClearingInfo, CurrencyAmount, DunningInfo, GLReference, PaymentTerms, SubledgerDocumentStatus,
    TaxInfo,
};

/// AR Invoice (customer invoice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARInvoice {
    /// Unique invoice number.
    pub invoice_number: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Invoice date.
    pub invoice_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Due date.
    pub due_date: NaiveDate,
    /// Baseline date for payment terms.
    pub baseline_date: NaiveDate,
    /// Invoice type.
    pub invoice_type: ARInvoiceType,
    /// Invoice status.
    pub status: SubledgerDocumentStatus,
    /// Invoice lines.
    pub lines: Vec<ARInvoiceLine>,
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
    /// Dunning information.
    pub dunning_info: DunningInfo,
    /// Reference documents (sales order, delivery).
    pub reference_documents: Vec<ARDocumentReference>,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Profit center.
    pub profit_center: Option<String>,
    /// Sales organization.
    pub sales_org: Option<String>,
    /// Distribution channel.
    pub distribution_channel: Option<String>,
    /// Division.
    pub division: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Last modified timestamp.
    pub modified_at: Option<DateTime<Utc>>,
    /// Notes.
    pub notes: Option<String>,
}

impl ARInvoice {
    /// Creates a new AR invoice.
    pub fn new(
        invoice_number: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        invoice_date: NaiveDate,
        payment_terms: PaymentTerms,
        currency: String,
    ) -> Self {
        let due_date = payment_terms.calculate_due_date(invoice_date);

        Self {
            invoice_number,
            company_code,
            customer_id,
            customer_name,
            invoice_date,
            posting_date: invoice_date,
            due_date,
            baseline_date: invoice_date,
            invoice_type: ARInvoiceType::Standard,
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
            dunning_info: DunningInfo::default(),
            reference_documents: Vec::new(),
            cost_center: None,
            profit_center: None,
            sales_org: None,
            distribution_channel: None,
            division: None,
            created_at: Utc::now(),
            created_by: None,
            modified_at: None,
            notes: None,
        }
    }

    /// Adds an invoice line.
    pub fn add_line(&mut self, line: ARInvoiceLine) {
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

    /// Sets the GL reference.
    pub fn set_gl_reference(&mut self, reference: GLReference) {
        self.gl_reference = Some(reference);
    }

    /// Adds a reference document.
    pub fn add_reference(&mut self, reference: ARDocumentReference) {
        self.reference_documents.push(reference);
    }

    /// Reverses the invoice.
    pub fn reverse(&mut self, reversal_date: NaiveDate, reason: String) {
        self.status = SubledgerDocumentStatus::Reversed;
        self.notes = Some(format!(
            "{}Reversed on {}: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{n}. "))
                .unwrap_or_default(),
            reversal_date,
            reason
        ));
        self.modified_at = Some(Utc::now());
    }
}

/// Type of AR invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ARInvoiceType {
    /// Standard invoice.
    #[default]
    Standard,
    /// Down payment request.
    DownPaymentRequest,
    /// Recurring invoice.
    Recurring,
    /// Credit invoice (negative).
    CreditInvoice,
    /// Debit invoice (adjustment).
    DebitInvoice,
    /// Pro forma invoice.
    ProForma,
    /// Intercompany invoice.
    Intercompany,
}

/// AR invoice line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARInvoiceLine {
    /// Line number.
    pub line_number: u32,
    /// Material/product ID.
    pub material_id: Option<String>,
    /// Description.
    pub description: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit of measure.
    pub unit: String,
    /// Unit price.
    pub unit_price: Decimal,
    /// Net amount (quantity * unit_price).
    pub net_amount: Decimal,
    /// Tax code.
    pub tax_code: Option<String>,
    /// Tax rate.
    pub tax_rate: Decimal,
    /// Tax amount.
    pub tax_amount: Decimal,
    /// Gross amount.
    pub gross_amount: Decimal,
    /// Revenue account.
    pub revenue_account: String,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Profit center.
    pub profit_center: Option<String>,
    /// Reference (sales order line).
    pub reference: Option<String>,
}

impl ARInvoiceLine {
    /// Creates a new invoice line.
    pub fn new(
        line_number: u32,
        description: String,
        quantity: Decimal,
        unit: String,
        unit_price: Decimal,
        revenue_account: String,
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
            revenue_account,
            cost_center: None,
            profit_center: None,
            reference: None,
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

    /// Sets material ID.
    pub fn with_material(mut self, material_id: String) -> Self {
        self.material_id = Some(material_id);
        self
    }

    /// Sets cost/profit center.
    pub fn with_cost_center(mut self, cost_center: String, profit_center: Option<String>) -> Self {
        self.cost_center = Some(cost_center);
        self.profit_center = profit_center;
        self
    }
}

/// Reference to related documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARDocumentReference {
    /// Reference document type.
    pub document_type: ARReferenceDocType,
    /// Document number.
    pub document_number: String,
    /// Document date.
    pub document_date: NaiveDate,
}

/// Type of reference document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ARReferenceDocType {
    /// Sales order.
    SalesOrder,
    /// Delivery.
    Delivery,
    /// Contract.
    Contract,
    /// Quotation.
    Quotation,
    /// Return order.
    ReturnOrder,
}

/// Summary of open AR items for a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerARSummary {
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Total open amount.
    pub total_open: Decimal,
    /// Total overdue amount.
    pub total_overdue: Decimal,
    /// Number of open invoices.
    pub open_invoice_count: u32,
    /// Number of overdue invoices.
    pub overdue_invoice_count: u32,
    /// Oldest open invoice date.
    pub oldest_open_date: Option<NaiveDate>,
    /// Credit limit.
    pub credit_limit: Option<Decimal>,
    /// Credit utilization percentage.
    pub credit_utilization: Option<Decimal>,
    /// Payment behavior score.
    pub payment_score: Option<Decimal>,
}

impl CustomerARSummary {
    /// Creates from a list of invoices.
    pub fn from_invoices(
        customer_id: String,
        customer_name: String,
        invoices: &[ARInvoice],
        as_of_date: NaiveDate,
        credit_limit: Option<Decimal>,
    ) -> Self {
        let open_invoices: Vec<_> = invoices
            .iter()
            .filter(|i| {
                i.customer_id == customer_id
                    && matches!(
                        i.status,
                        SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                    )
            })
            .collect();

        let total_open: Decimal = open_invoices.iter().map(|i| i.amount_remaining).sum();
        let overdue_invoices: Vec<_> = open_invoices
            .iter()
            .filter(|i| i.is_overdue(as_of_date))
            .collect();
        let total_overdue: Decimal = overdue_invoices.iter().map(|i| i.amount_remaining).sum();

        let oldest_open_date = open_invoices.iter().map(|i| i.invoice_date).min();

        let credit_utilization = credit_limit.map(|limit| {
            if limit > Decimal::ZERO {
                (total_open / limit * dec!(100)).round_dp(2)
            } else {
                Decimal::ZERO
            }
        });

        Self {
            customer_id,
            customer_name,
            total_open,
            total_overdue,
            open_invoice_count: open_invoices.len() as u32,
            overdue_invoice_count: overdue_invoices.len() as u32,
            oldest_open_date,
            credit_limit,
            credit_utilization,
            payment_score: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_invoice() -> ARInvoice {
        let mut invoice = ARInvoice::new(
            "INV001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            PaymentTerms::net_30(),
            "USD".to_string(),
        );

        let line = ARInvoiceLine::new(
            1,
            "Product A".to_string(),
            dec!(10),
            "EA".to_string(),
            dec!(100),
            "4000".to_string(),
        )
        .with_tax("VAT".to_string(), dec!(20));

        invoice.add_line(line);
        invoice
    }

    #[test]
    fn test_invoice_totals() {
        let invoice = create_test_invoice();
        assert_eq!(invoice.net_amount.document_amount, dec!(1000));
        assert_eq!(invoice.tax_amount.document_amount, dec!(200));
        assert_eq!(invoice.gross_amount.document_amount, dec!(1200));
    }

    #[test]
    fn test_invoice_due_date() {
        let invoice = create_test_invoice();
        assert_eq!(
            invoice.due_date,
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()
        );
    }

    #[test]
    fn test_invoice_overdue() {
        let invoice = create_test_invoice();
        let before_due = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
        let after_due = NaiveDate::from_ymd_opt(2024, 2, 20).unwrap();

        assert!(!invoice.is_overdue(before_due));
        assert!(invoice.is_overdue(after_due));
        assert_eq!(invoice.days_overdue(after_due), 6);
    }

    #[test]
    fn test_apply_payment() {
        let mut invoice = create_test_invoice();
        let clearing = ClearingInfo {
            clearing_document: "PAY001".to_string(),
            clearing_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            clearing_amount: dec!(600),
            clearing_type: crate::models::subledger::ClearingType::Payment,
        };

        invoice.apply_payment(dec!(600), clearing);
        assert_eq!(invoice.amount_paid, dec!(600));
        assert_eq!(invoice.amount_remaining, dec!(600));
        assert_eq!(invoice.status, SubledgerDocumentStatus::PartiallyCleared);
    }
}
