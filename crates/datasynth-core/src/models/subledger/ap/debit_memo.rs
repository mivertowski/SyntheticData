//! AP Debit Memo model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{CurrencyAmount, GLReference, SubledgerDocumentStatus, TaxInfo};

/// AP Debit Memo (reduces amount owed to vendor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APDebitMemo {
    /// Unique debit memo number.
    pub debit_memo_number: String,
    /// Company code.
    pub company_code: String,
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Debit memo date.
    pub memo_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Debit memo type.
    pub memo_type: APDebitMemoType,
    /// Debit memo status.
    pub status: SubledgerDocumentStatus,
    /// Reason code.
    pub reason_code: DebitMemoReason,
    /// Reason description.
    pub reason_description: String,
    /// Debit memo lines.
    pub lines: Vec<APDebitMemoLine>,
    /// Net amount (before tax).
    pub net_amount: CurrencyAmount,
    /// Tax amount.
    pub tax_amount: CurrencyAmount,
    /// Gross amount (total credit to AP).
    pub gross_amount: CurrencyAmount,
    /// Amount applied to invoices.
    pub amount_applied: Decimal,
    /// Amount remaining.
    pub amount_remaining: Decimal,
    /// Tax details.
    pub tax_details: Vec<TaxInfo>,
    /// Reference invoice (if applicable).
    pub reference_invoice: Option<String>,
    /// Reference purchase order.
    pub reference_po: Option<String>,
    /// Reference goods receipt.
    pub reference_gr: Option<String>,
    /// Applied to invoices.
    pub applied_invoices: Vec<DebitMemoApplication>,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Requires approval.
    pub requires_approval: bool,
    /// Approval status.
    pub approval_status: APApprovalStatus,
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

impl APDebitMemo {
    /// Creates a new debit memo.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        debit_memo_number: String,
        company_code: String,
        vendor_id: String,
        vendor_name: String,
        memo_date: NaiveDate,
        reason_code: DebitMemoReason,
        reason_description: String,
        currency: String,
    ) -> Self {
        Self {
            debit_memo_number,
            company_code,
            vendor_id,
            vendor_name,
            memo_date,
            posting_date: memo_date,
            memo_type: APDebitMemoType::Standard,
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
            reference_po: None,
            reference_gr: None,
            applied_invoices: Vec::new(),
            gl_reference: None,
            requires_approval: true,
            approval_status: APApprovalStatus::Pending,
            approved_by: None,
            approved_date: None,
            created_at: Utc::now(),
            created_by: None,
            notes: None,
        }
    }

    /// Creates debit memo for a specific invoice.
    #[allow(clippy::too_many_arguments)]
    pub fn for_invoice(
        debit_memo_number: String,
        company_code: String,
        vendor_id: String,
        vendor_name: String,
        memo_date: NaiveDate,
        invoice_number: String,
        reason_code: DebitMemoReason,
        reason_description: String,
        currency: String,
    ) -> Self {
        let mut memo = Self::new(
            debit_memo_number,
            company_code,
            vendor_id,
            vendor_name,
            memo_date,
            reason_code,
            reason_description,
            currency,
        );
        memo.reference_invoice = Some(invoice_number);
        memo
    }

    /// Adds a debit memo line.
    pub fn add_line(&mut self, line: APDebitMemoLine) {
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

    /// Applies debit memo to an invoice.
    pub fn apply_to_invoice(&mut self, invoice_number: String, amount: Decimal) {
        let application = DebitMemoApplication {
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

    /// Sets purchase order reference.
    pub fn with_po_reference(mut self, po_number: String) -> Self {
        self.reference_po = Some(po_number);
        self
    }

    /// Sets goods receipt reference.
    pub fn with_gr_reference(mut self, gr_number: String) -> Self {
        self.reference_gr = Some(gr_number);
        self.memo_type = APDebitMemoType::Return;
        self
    }

    /// Approves the debit memo.
    pub fn approve(&mut self, approver: String, approval_date: NaiveDate) {
        self.approval_status = APApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_date = Some(approval_date);
    }

    /// Rejects the debit memo.
    pub fn reject(&mut self, reason: String) {
        self.approval_status = APApprovalStatus::Rejected;
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

    /// Checks if approval is required based on threshold.
    pub fn check_approval_threshold(&mut self, threshold: Decimal) {
        self.requires_approval = self.gross_amount.document_amount > threshold;
        if !self.requires_approval {
            self.approval_status = APApprovalStatus::NotRequired;
        }
    }
}

/// Type of debit memo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum APDebitMemoType {
    /// Standard debit memo.
    #[default]
    Standard,
    /// Return to vendor.
    Return,
    /// Price adjustment.
    PriceAdjustment,
    /// Quantity adjustment.
    QuantityAdjustment,
    /// Rebate/volume discount.
    Rebate,
    /// Quality claim.
    QualityClaim,
    /// Cancellation.
    Cancellation,
}

/// Reason code for debit memo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DebitMemoReason {
    /// Goods returned.
    Return,
    /// Damaged goods received.
    Damaged,
    /// Wrong item received.
    WrongItem,
    /// Price overcharge.
    PriceOvercharge,
    /// Quantity shortage.
    QuantityShortage,
    /// Quality issue.
    QualityIssue,
    /// Late delivery penalty.
    LateDeliveryPenalty,
    /// Duplicate invoice.
    DuplicateInvoice,
    /// Service not performed.
    ServiceNotPerformed,
    /// Contract adjustment.
    ContractAdjustment,
    /// Other.
    #[default]
    Other,
}

/// Debit memo line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APDebitMemoLine {
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
    /// Reference invoice line.
    pub reference_invoice_line: Option<u32>,
    /// Cost center.
    pub cost_center: Option<String>,
}

impl APDebitMemoLine {
    /// Creates a new debit memo line.
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
            reference_invoice_line: None,
            cost_center: None,
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

    /// Sets reference to original invoice line.
    pub fn with_invoice_reference(mut self, line_number: u32) -> Self {
        self.reference_invoice_line = Some(line_number);
        self
    }
}

/// Application of debit memo to invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebitMemoApplication {
    /// Invoice number.
    pub invoice_number: String,
    /// Amount applied.
    pub amount_applied: Decimal,
    /// Application date.
    pub application_date: NaiveDate,
}

/// Approval status for AP documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum APApprovalStatus {
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

    #[test]
    fn test_debit_memo_creation() {
        let memo = APDebitMemo::new(
            "DM001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            DebitMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        assert_eq!(memo.status, SubledgerDocumentStatus::Open);
        assert_eq!(memo.approval_status, APApprovalStatus::Pending);
    }

    #[test]
    fn test_debit_memo_totals() {
        let mut memo = APDebitMemo::new(
            "DM001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            DebitMemoReason::PriceOvercharge,
            "Price correction".to_string(),
            "USD".to_string(),
        );

        let line = APDebitMemoLine::new(
            1,
            "Product A".to_string(),
            dec!(10),
            "EA".to_string(),
            dec!(50),
            "5000".to_string(),
        )
        .with_tax("VAT".to_string(), dec!(10));

        memo.add_line(line);

        assert_eq!(memo.net_amount.document_amount, dec!(500));
        assert_eq!(memo.tax_amount.document_amount, dec!(50));
        assert_eq!(memo.gross_amount.document_amount, dec!(550));
    }

    #[test]
    fn test_apply_to_invoice() {
        let mut memo = APDebitMemo::new(
            "DM001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            DebitMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        let line = APDebitMemoLine::new(
            1,
            "Product A".to_string(),
            dec!(10),
            "EA".to_string(),
            dec!(100),
            "5000".to_string(),
        );
        memo.add_line(line);

        memo.apply_to_invoice("INV001".to_string(), dec!(500));

        assert_eq!(memo.amount_applied, dec!(500));
        assert_eq!(memo.amount_remaining, dec!(500));
        assert_eq!(memo.status, SubledgerDocumentStatus::PartiallyCleared);
    }

    #[test]
    fn test_approval_workflow() {
        let mut memo = APDebitMemo::new(
            "DM001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            DebitMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        memo.approve(
            "MANAGER1".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
        );

        assert_eq!(memo.approval_status, APApprovalStatus::Approved);
        assert_eq!(memo.approved_by, Some("MANAGER1".to_string()));
    }

    #[test]
    fn test_approval_threshold() {
        let mut memo = APDebitMemo::new(
            "DM001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            DebitMemoReason::Return,
            "Goods returned".to_string(),
            "USD".to_string(),
        );

        let line = APDebitMemoLine::new(
            1,
            "Product A".to_string(),
            dec!(1),
            "EA".to_string(),
            dec!(50),
            "5000".to_string(),
        );
        memo.add_line(line);

        // Under threshold
        memo.check_approval_threshold(dec!(100));
        assert_eq!(memo.approval_status, APApprovalStatus::NotRequired);
        assert!(!memo.requires_approval);
    }
}
