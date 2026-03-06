//! AR Receipt (customer payment) model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{CurrencyAmount, GLReference, SubledgerDocumentStatus};

/// AR Receipt (payment from customer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARReceipt {
    /// Unique receipt number.
    pub receipt_number: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Receipt date.
    pub receipt_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Value date (bank value date).
    pub value_date: NaiveDate,
    /// Receipt type.
    pub receipt_type: ARReceiptType,
    /// Receipt status.
    pub status: SubledgerDocumentStatus,
    /// Receipt amount.
    pub amount: CurrencyAmount,
    /// Bank charges deducted.
    pub bank_charges: Decimal,
    /// Discount taken.
    pub discount_taken: Decimal,
    /// Write-off amount.
    pub write_off_amount: Decimal,
    /// Net amount applied to invoices.
    pub net_applied: Decimal,
    /// Unapplied amount.
    pub unapplied_amount: Decimal,
    /// Payment method.
    pub payment_method: PaymentMethod,
    /// Bank account.
    pub bank_account: String,
    /// Bank reference.
    pub bank_reference: Option<String>,
    /// Check number (if check payment).
    pub check_number: Option<String>,
    /// Applied invoices.
    pub applied_invoices: Vec<ReceiptApplication>,
    /// GL references.
    pub gl_references: Vec<GLReference>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ARReceipt {
    /// Creates a new AR receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        receipt_number: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        receipt_date: NaiveDate,
        amount: Decimal,
        currency: String,
        payment_method: PaymentMethod,
        bank_account: String,
    ) -> Self {
        Self {
            receipt_number,
            company_code,
            customer_id,
            customer_name,
            receipt_date,
            posting_date: receipt_date,
            value_date: receipt_date,
            receipt_type: ARReceiptType::Standard,
            status: SubledgerDocumentStatus::Open,
            amount: CurrencyAmount::single_currency(amount, currency),
            bank_charges: Decimal::ZERO,
            discount_taken: Decimal::ZERO,
            write_off_amount: Decimal::ZERO,
            net_applied: Decimal::ZERO,
            unapplied_amount: amount,
            payment_method,
            bank_account,
            bank_reference: None,
            check_number: None,
            applied_invoices: Vec::new(),
            gl_references: Vec::new(),
            created_at: Utc::now(),
            created_by: None,
            notes: None,
        }
    }

    /// Applies receipt to an invoice.
    pub fn apply_to_invoice(
        &mut self,
        invoice_number: String,
        amount_applied: Decimal,
        discount: Decimal,
    ) {
        let application = ReceiptApplication {
            invoice_number,
            amount_applied,
            discount_taken: discount,
            write_off: Decimal::ZERO,
            application_date: self.receipt_date,
        };

        self.applied_invoices.push(application);
        self.net_applied += amount_applied;
        self.discount_taken += discount;
        self.unapplied_amount = self.amount.document_amount - self.net_applied;

        if self.unapplied_amount <= Decimal::ZERO {
            self.status = SubledgerDocumentStatus::Cleared;
        }
    }

    /// Sets bank charges.
    pub fn with_bank_charges(mut self, charges: Decimal) -> Self {
        self.bank_charges = charges;
        self.unapplied_amount -= charges;
        self
    }

    /// Sets check number.
    pub fn with_check(mut self, check_number: String) -> Self {
        self.check_number = Some(check_number);
        self.payment_method = PaymentMethod::Check;
        self
    }

    /// Sets bank reference.
    pub fn with_bank_reference(mut self, reference: String) -> Self {
        self.bank_reference = Some(reference);
        self
    }

    /// Adds a GL reference.
    pub fn add_gl_reference(&mut self, reference: GLReference) {
        self.gl_references.push(reference);
    }

    /// Gets total amount including discount.
    pub fn total_settlement(&self) -> Decimal {
        self.net_applied + self.discount_taken + self.write_off_amount
    }

    /// Reverses the receipt.
    pub fn reverse(&mut self, reason: String) {
        self.status = SubledgerDocumentStatus::Reversed;
        self.notes = Some(format!(
            "{}Reversed: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{n}. "))
                .unwrap_or_default(),
            reason
        ));
    }

    /// Creates on-account receipt (not applied to specific invoices).
    #[allow(clippy::too_many_arguments)]
    pub fn on_account(
        receipt_number: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        receipt_date: NaiveDate,
        amount: Decimal,
        currency: String,
        payment_method: PaymentMethod,
        bank_account: String,
    ) -> Self {
        let mut receipt = Self::new(
            receipt_number,
            company_code,
            customer_id,
            customer_name,
            receipt_date,
            amount,
            currency,
            payment_method,
            bank_account,
        );
        receipt.receipt_type = ARReceiptType::OnAccount;
        receipt
    }
}

/// Type of AR receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ARReceiptType {
    /// Standard receipt applied to invoices.
    #[default]
    Standard,
    /// On-account receipt (unapplied).
    OnAccount,
    /// Down payment receipt.
    DownPayment,
    /// Refund (negative receipt).
    Refund,
    /// Write-off receipt.
    WriteOff,
    /// Netting (AR against AP).
    Netting,
}

/// Payment method for receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaymentMethod {
    /// Wire transfer.
    #[default]
    WireTransfer,
    /// Check.
    Check,
    /// ACH/Direct debit.
    ACH,
    /// Credit card.
    CreditCard,
    /// Cash.
    Cash,
    /// Letter of credit.
    LetterOfCredit,
    /// Netting.
    Netting,
    /// Other.
    Other,
}

/// Application of receipt to an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptApplication {
    /// Invoice number.
    pub invoice_number: String,
    /// Amount applied.
    pub amount_applied: Decimal,
    /// Discount taken.
    pub discount_taken: Decimal,
    /// Write-off amount.
    pub write_off: Decimal,
    /// Application date.
    pub application_date: NaiveDate,
}

impl ReceiptApplication {
    /// Total settlement for this application.
    pub fn total_settlement(&self) -> Decimal {
        self.amount_applied + self.discount_taken + self.write_off
    }
}

/// Batch of receipts for processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARReceiptBatch {
    /// Batch ID.
    pub batch_id: String,
    /// Company code.
    pub company_code: String,
    /// Batch date.
    pub batch_date: NaiveDate,
    /// Receipts in batch.
    pub receipts: Vec<ARReceipt>,
    /// Total batch amount.
    pub total_amount: Decimal,
    /// Batch status.
    pub status: BatchStatus,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
}

impl ARReceiptBatch {
    /// Creates a new receipt batch.
    pub fn new(
        batch_id: String,
        company_code: String,
        batch_date: NaiveDate,
        created_by: String,
    ) -> Self {
        Self {
            batch_id,
            company_code,
            batch_date,
            receipts: Vec::new(),
            total_amount: Decimal::ZERO,
            status: BatchStatus::Open,
            created_by,
            created_at: Utc::now(),
        }
    }

    /// Adds a receipt to the batch.
    pub fn add_receipt(&mut self, receipt: ARReceipt) {
        self.total_amount += receipt.amount.document_amount;
        self.receipts.push(receipt);
    }

    /// Posts the batch.
    pub fn post(&mut self) {
        self.status = BatchStatus::Posted;
    }

    /// Gets receipt count.
    pub fn count(&self) -> usize {
        self.receipts.len()
    }
}

/// Status of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch is open for additions.
    Open,
    /// Batch is submitted for approval.
    Submitted,
    /// Batch is approved.
    Approved,
    /// Batch is posted.
    Posted,
    /// Batch is cancelled.
    Cancelled,
}

/// Bank statement line for automatic matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementLine {
    /// Statement line ID.
    pub line_id: String,
    /// Bank account.
    pub bank_account: String,
    /// Statement date.
    pub statement_date: NaiveDate,
    /// Value date.
    pub value_date: NaiveDate,
    /// Amount (positive = receipt, negative = payment).
    pub amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Bank reference.
    pub bank_reference: String,
    /// Payer/payee name.
    pub counterparty_name: Option<String>,
    /// Payer/payee account.
    pub counterparty_account: Option<String>,
    /// Payment reference/remittance info.
    pub payment_reference: Option<String>,
    /// Is matched.
    pub is_matched: bool,
    /// Matched receipt number.
    pub matched_receipt: Option<String>,
}

impl BankStatementLine {
    /// Checks if this is an incoming payment.
    pub fn is_receipt(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    /// Marks as matched.
    pub fn match_to_receipt(&mut self, receipt_number: String) {
        self.is_matched = true;
        self.matched_receipt = Some(receipt_number);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_receipt_creation() {
        let receipt = ARReceipt::new(
            "REC001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec!(1000),
            "USD".to_string(),
            PaymentMethod::WireTransfer,
            "1000".to_string(),
        );

        assert_eq!(receipt.amount.document_amount, dec!(1000));
        assert_eq!(receipt.unapplied_amount, dec!(1000));
        assert_eq!(receipt.status, SubledgerDocumentStatus::Open);
    }

    #[test]
    fn test_apply_to_invoice() {
        let mut receipt = ARReceipt::new(
            "REC001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec!(1000),
            "USD".to_string(),
            PaymentMethod::WireTransfer,
            "1000".to_string(),
        );

        receipt.apply_to_invoice("INV001".to_string(), dec!(800), dec!(20));

        assert_eq!(receipt.net_applied, dec!(800));
        assert_eq!(receipt.discount_taken, dec!(20));
        assert_eq!(receipt.unapplied_amount, dec!(200));
        assert_eq!(receipt.applied_invoices.len(), 1);
    }

    #[test]
    fn test_receipt_fully_applied() {
        let mut receipt = ARReceipt::new(
            "REC001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec!(1000),
            "USD".to_string(),
            PaymentMethod::WireTransfer,
            "1000".to_string(),
        );

        receipt.apply_to_invoice("INV001".to_string(), dec!(1000), Decimal::ZERO);

        assert_eq!(receipt.status, SubledgerDocumentStatus::Cleared);
        assert_eq!(receipt.unapplied_amount, Decimal::ZERO);
    }

    #[test]
    fn test_batch_totals() {
        let mut batch = ARReceiptBatch::new(
            "BATCH001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            "USER1".to_string(),
        );

        let receipt1 = ARReceipt::new(
            "REC001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Customer 1".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec!(500),
            "USD".to_string(),
            PaymentMethod::WireTransfer,
            "1000".to_string(),
        );

        let receipt2 = ARReceipt::new(
            "REC002".to_string(),
            "1000".to_string(),
            "CUST002".to_string(),
            "Customer 2".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec!(750),
            "USD".to_string(),
            PaymentMethod::Check,
            "1000".to_string(),
        );

        batch.add_receipt(receipt1);
        batch.add_receipt(receipt2);

        assert_eq!(batch.count(), 2);
        assert_eq!(batch.total_amount, dec!(1250));
    }
}
