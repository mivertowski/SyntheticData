//! AR Credit Memo model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{CurrencyAmount, GLReference, SubledgerDocumentStatus, TaxInfo};

/// AR Credit Memo (reduces customer balance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARCreditMemo {
    /// Unique credit memo number.
    pub credit_memo_number: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Credit memo date.
    pub memo_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Credit memo type.
    pub memo_type: ARCreditMemoType,
    /// Credit memo status.
    pub status: SubledgerDocumentStatus,
    /// Reason code.
    pub reason_code: CreditMemoReason,
    /// Reason description.
    pub reason_description: String,
    /// Credit memo lines.
    pub lines: Vec<ARCreditMemoLine>,
    /// Net amount (before tax).
    pub net_amount: CurrencyAmount,
    /// Tax amount.
    pub tax_amount: CurrencyAmount,
    /// Gross amount (total credit).
    pub gross_amount: CurrencyAmount,
    /// Amount applied to invoices.
    pub amount_applied: Decimal,
    /// Amount remaining.
    pub amount_remaining: Decimal,
    /// Tax details.
    pub tax_details: Vec<TaxInfo>,
    /// Reference invoice (if applicable).
    pub reference_invoice: Option<String>,
    /// Reference return order.
    pub reference_return: Option<String>,
    /// Applied to invoices.
    pub applied_invoices: Vec<CreditMemoApplication>,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Approval status.
    pub approval_status: ApprovalStatus,
    /// Approved by.
    pub approved_by: Option<String>,
    /// Approval date.
    pub approved_date: Option<NaiveDate>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ARCreditMemo {
    /// Creates a new credit memo.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        credit_memo_number: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        memo_date: NaiveDate,
        reason_code: CreditMemoReason,
        reason_description: String,
        currency: String,
    ) -> Self {
        Self {
            credit_memo_number,
            company_code,
            customer_id,
            customer_name,
            memo_date,
            posting_date: memo_date,
            memo_type: ARCreditMemoType::Standard,
            status: SubledgerDocumentStatus::Open,
            reason_code,
            reason_description,
            lines: Vec::new(),
            net_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency.clone()),
            tax_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency.clone()),
            gross_amount: CurrencyAmount::single_currency(Decimal::ZERO, currency),
            amount_applied: Decimal::ZERO,
            amount_remaining: Decimal::ZERO,
            tax_details: Vec::new(),
            reference_invoice: None,
            reference_return: None,
            applied_invoices: Vec::new(),
            gl_reference: None,
            approval_status: ApprovalStatus::Pending,
            approved_by: None,
            approved_date: None,
            created_at: Utc::now(),
            created_by: None,
            notes: None,
        }
    }

    /// Creates credit memo for a specific invoice.
    #[allow(clippy::too_many_arguments)]
    pub fn for_invoice(
        credit_memo_number: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        memo_date: NaiveDate,
        invoice_number: String,
        reason_code: CreditMemoReason,
        reason_description: String,
        currency: String,
    ) -> Self {
        let mut memo = Self::new(
            credit_memo_number,
            company_code,
            customer_id,
            customer_name,
            memo_date,
            reason_code,
            reason_description,
            currency,
        );
        memo.reference_invoice = Some(invoice_number);
        memo
    }

    /// Adds a credit memo line.
    pub fn add_line(&mut self, line: ARCreditMemoLine) {
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
        self.amount_remaining = gross_total - self.amount_applied;
    }

    /// Applies credit memo to an invoice.
    pub fn apply_to_invoice(&mut self, invoice_number: String, amount: Decimal) {
        let application = CreditMemoApplication {
            invoice_number,
            amount_applied: amount,
            application_date: chrono::Local::now().date_naive(),
        };

        self.applied_invoices.push(application);
        self.amount_applied += amount;
        self.amount_remaining = self.gross_amount.document_amount - self.amount_applied;

        if self.amount_remaining <= Decimal::ZERO {
            self.status = SubledgerDocumentStatus::Cleared;
        } else {
            self.status = SubledgerDocumentStatus::PartiallyCleared;
        }
    }

    /// Approves the credit memo.
    pub fn approve(&mut self, approver: String, approval_date: NaiveDate) {
        self.approval_status = ApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_date = Some(approval_date);
    }

    /// Rejects the credit memo.
    pub fn reject(&mut self, reason: String) {
        self.approval_status = ApprovalStatus::Rejected;
        self.notes = Some(format!(
            "{}Rejected: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{n}. "))
                .unwrap_or_default(),
            reason
        ));
    }

    /// Sets the GL reference.
    pub fn set_gl_reference(&mut self, reference: GLReference) {
        self.gl_reference = Some(reference);
    }

    /// Sets reference return order.
    pub fn with_return_order(mut self, return_order: String) -> Self {
        self.reference_return = Some(return_order);
        self.memo_type = ARCreditMemoType::Return;
        self
    }

    /// Requires approval above threshold.
    pub fn requires_approval(&self, threshold: Decimal) -> bool {
        self.gross_amount.document_amount > threshold
    }
}

/// Type of credit memo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ARCreditMemoType {
    /// Standard credit memo.
    #[default]
    Standard,
    /// Return credit memo.
    Return,
    /// Price adjustment.
    PriceAdjustment,
    /// Quantity adjustment.
    QuantityAdjustment,
    /// Rebate/volume discount.
    Rebate,
    /// Promotional credit.
    Promotional,
    /// Cancellation credit.
    Cancellation,
}

/// Reason code for credit memo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CreditMemoReason {
    /// Goods returned.
    Return,
    /// Damaged goods.
    Damaged,
    /// Wrong item shipped.
    WrongItem,
    /// Price error.
    PriceError,
    /// Quantity error.
    QuantityError,
    /// Quality issue.
    QualityIssue,
    /// Late delivery.
    LateDelivery,
    /// Promotional discount.
    Promotional,
    /// Volume rebate.
    VolumeRebate,
    /// Customer goodwill.
    Goodwill,
    /// Billing error.
    BillingError,
    /// Contract adjustment.
    ContractAdjustment,
    /// Other.
    #[default]
    Other,
}

/// Credit memo line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARCreditMemoLine {
    /// Line number.
    pub line_number: u32,
    /// Material/product ID.
    pub material_id: Option<String>,
    /// Description.
    pub description: String,
    /// Quantity credited.
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
    /// Revenue account (credit).
    pub revenue_account: String,
    /// Reference invoice line.
    pub reference_invoice_line: Option<u32>,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Profit center.
    pub profit_center: Option<String>,
}

impl ARCreditMemoLine {
    /// Creates a new credit memo line.
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
            reference_invoice_line: None,
            cost_center: None,
            profit_center: None,
        }
    }

    /// Sets tax information.
    pub fn with_tax(mut self, tax_code: String, tax_rate: Decimal) -> Self {
        self.tax_code = Some(tax_code);
        self.tax_rate = tax_rate;
        self.tax_amount = (self.net_amount * tax_rate / rust_decimal_macros::dec!(100)).round_dp(2);
        self.gross_amount = self.net_amount + self.tax_amount;
        self
    }

    /// Sets reference to original invoice line.
    pub fn with_invoice_reference(mut self, line_number: u32) -> Self {
        self.reference_invoice_line = Some(line_number);
        self
    }
}

/// Application of credit memo to invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditMemoApplication {
    /// Invoice number.
    pub invoice_number: String,
    /// Amount applied.
    pub amount_applied: Decimal,
    /// Application date.
    pub application_date: NaiveDate,
}

/// Approval status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ApprovalStatus {
    /// Pending approval.
    #[default]
    Pending,
    /// Approved.
    Approved,
    /// Rejected.
    Rejected,
    /// Not required (under threshold).
    NotRequired,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_credit_memo_creation() {
        let memo = ARCreditMemo::new(
            "CM001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            CreditMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        assert_eq!(memo.status, SubledgerDocumentStatus::Open);
        assert_eq!(memo.approval_status, ApprovalStatus::Pending);
    }

    #[test]
    fn test_credit_memo_totals() {
        let mut memo = ARCreditMemo::new(
            "CM001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            CreditMemoReason::PriceError,
            "Price correction".to_string(),
            "USD".to_string(),
        );

        let line = ARCreditMemoLine::new(
            1,
            "Product A".to_string(),
            dec!(5),
            "EA".to_string(),
            dec!(100),
            "4000".to_string(),
        )
        .with_tax("VAT".to_string(), dec!(20));

        memo.add_line(line);

        assert_eq!(memo.net_amount.document_amount, dec!(500));
        assert_eq!(memo.tax_amount.document_amount, dec!(100));
        assert_eq!(memo.gross_amount.document_amount, dec!(600));
    }

    #[test]
    fn test_apply_to_invoice() {
        let mut memo = ARCreditMemo::new(
            "CM001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            CreditMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        let line = ARCreditMemoLine::new(
            1,
            "Product A".to_string(),
            dec!(10),
            "EA".to_string(),
            dec!(50),
            "4000".to_string(),
        );
        memo.add_line(line);

        memo.apply_to_invoice("INV001".to_string(), dec!(300));

        assert_eq!(memo.amount_applied, dec!(300));
        assert_eq!(memo.amount_remaining, dec!(200));
        assert_eq!(memo.status, SubledgerDocumentStatus::PartiallyCleared);
    }

    #[test]
    fn test_approval_workflow() {
        let mut memo = ARCreditMemo::new(
            "CM001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            CreditMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        memo.approve(
            "MANAGER1".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
        );

        assert_eq!(memo.approval_status, ApprovalStatus::Approved);
        assert_eq!(memo.approved_by, Some("MANAGER1".to_string()));
    }
}
