//! Payment document models.
//!
//! Represents AP payments and AR receipts in the P2P and O2C flows.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{DocumentHeader, DocumentReference, DocumentStatus, DocumentType, ReferenceType};

/// Payment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    /// Outgoing payment (AP)
    #[default]
    ApPayment,
    /// Incoming payment (AR)
    ArReceipt,
    /// Down payment
    DownPayment,
    /// Advance payment
    Advance,
    /// Refund
    Refund,
    /// Clearing (internal)
    Clearing,
}

/// Payment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// Bank transfer / ACH
    #[default]
    BankTransfer,
    /// Check
    Check,
    /// Wire transfer
    Wire,
    /// Credit card
    CreditCard,
    /// Direct debit
    DirectDebit,
    /// Cash
    Cash,
    /// Letter of credit
    LetterOfCredit,
}

impl PaymentMethod {
    /// Get typical processing days for this method.
    pub fn processing_days(&self) -> u8 {
        match self {
            Self::Wire | Self::Cash => 0,
            Self::BankTransfer | Self::DirectDebit => 1,
            Self::CreditCard => 2,
            Self::Check => 5,
            Self::LetterOfCredit => 7,
        }
    }
}

/// Payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Created/pending
    #[default]
    Pending,
    /// Approved
    Approved,
    /// Sent to bank
    Sent,
    /// Cleared by bank
    Cleared,
    /// Rejected
    Rejected,
    /// Returned
    Returned,
    /// Cancelled
    Cancelled,
}

/// Payment allocation to an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAllocation {
    /// Invoice document ID
    pub invoice_id: String,
    /// Invoice type
    pub invoice_type: DocumentType,
    /// Allocated amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Discount taken
    #[serde(with = "rust_decimal::serde::str")]
    pub discount_taken: Decimal,
    /// Write-off amount
    #[serde(with = "rust_decimal::serde::str")]
    pub write_off: Decimal,
    /// Withholding tax
    #[serde(with = "rust_decimal::serde::str")]
    pub withholding_tax: Decimal,
    /// Is this allocation cleared?
    pub is_cleared: bool,
}

impl PaymentAllocation {
    /// Create a new allocation.
    pub fn new(invoice_id: impl Into<String>, invoice_type: DocumentType, amount: Decimal) -> Self {
        Self {
            invoice_id: invoice_id.into(),
            invoice_type,
            amount,
            discount_taken: Decimal::ZERO,
            write_off: Decimal::ZERO,
            withholding_tax: Decimal::ZERO,
            is_cleared: false,
        }
    }

    /// Set discount taken.
    pub fn with_discount(mut self, discount: Decimal) -> Self {
        self.discount_taken = discount;
        self
    }

    /// Total applied amount (including discount).
    pub fn total_applied(&self) -> Decimal {
        self.amount + self.discount_taken + self.write_off
    }
}

/// Payment document (AP Payment or AR Receipt).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Document header
    pub header: DocumentHeader,

    /// Payment type
    pub payment_type: PaymentType,

    /// Business partner ID (vendor or customer)
    pub business_partner_id: String,

    /// Is this a vendor (true) or customer (false)?
    pub is_vendor: bool,

    /// Payment method
    pub payment_method: PaymentMethod,

    /// Payment status
    pub payment_status: PaymentStatus,

    /// Payment amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,

    /// Payment currency
    pub currency: String,

    /// Bank account (house bank)
    pub house_bank: String,

    /// Bank account ID
    pub bank_account_id: String,

    /// Partner bank details
    pub partner_bank_account: Option<String>,

    /// Value date (when funds are available)
    pub value_date: NaiveDate,

    /// Check number (if check payment)
    pub check_number: Option<String>,

    /// Wire reference
    pub wire_reference: Option<String>,

    /// Allocations to invoices
    pub allocations: Vec<PaymentAllocation>,

    /// Total discount taken
    #[serde(with = "rust_decimal::serde::str")]
    pub total_discount: Decimal,

    /// Total write-off
    #[serde(with = "rust_decimal::serde::str")]
    pub total_write_off: Decimal,

    /// Bank charges
    #[serde(with = "rust_decimal::serde::str")]
    pub bank_charges: Decimal,

    /// Exchange rate (if foreign currency)
    #[serde(with = "rust_decimal::serde::str")]
    pub exchange_rate: Decimal,

    /// FX gain/loss
    #[serde(with = "rust_decimal::serde::str")]
    pub fx_gain_loss: Decimal,

    /// Payment run ID (if from automatic payment run)
    pub payment_run_id: Option<String>,

    /// Is this payment cleared by bank?
    pub is_bank_cleared: bool,

    /// Bank statement reference
    pub bank_statement_ref: Option<String>,

    /// Cleared date
    pub cleared_date: Option<NaiveDate>,

    /// Is this payment voided?
    pub is_voided: bool,

    /// Void reason
    pub void_reason: Option<String>,
}

impl Payment {
    /// Create a new AP payment.
    #[allow(clippy::too_many_arguments)]
    pub fn new_ap_payment(
        payment_id: impl Into<String>,
        company_code: impl Into<String>,
        vendor_id: impl Into<String>,
        amount: Decimal,
        fiscal_year: u16,
        fiscal_period: u8,
        payment_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            payment_id,
            DocumentType::ApPayment,
            company_code,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_currency("USD");

        Self {
            header,
            payment_type: PaymentType::ApPayment,
            business_partner_id: vendor_id.into(),
            is_vendor: true,
            payment_method: PaymentMethod::BankTransfer,
            payment_status: PaymentStatus::Pending,
            amount,
            currency: "USD".to_string(),
            house_bank: "BANK01".to_string(),
            bank_account_id: "001".to_string(),
            partner_bank_account: None,
            value_date: payment_date,
            check_number: None,
            wire_reference: None,
            allocations: Vec::new(),
            total_discount: Decimal::ZERO,
            total_write_off: Decimal::ZERO,
            bank_charges: Decimal::ZERO,
            exchange_rate: Decimal::ONE,
            fx_gain_loss: Decimal::ZERO,
            payment_run_id: None,
            is_bank_cleared: false,
            bank_statement_ref: None,
            cleared_date: None,
            is_voided: false,
            void_reason: None,
        }
    }

    /// Create a new AR receipt.
    #[allow(clippy::too_many_arguments)]
    pub fn new_ar_receipt(
        payment_id: impl Into<String>,
        company_code: impl Into<String>,
        customer_id: impl Into<String>,
        amount: Decimal,
        fiscal_year: u16,
        fiscal_period: u8,
        payment_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let header = DocumentHeader::new(
            payment_id,
            DocumentType::CustomerReceipt,
            company_code,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_currency("USD");

        Self {
            header,
            payment_type: PaymentType::ArReceipt,
            business_partner_id: customer_id.into(),
            is_vendor: false,
            payment_method: PaymentMethod::BankTransfer,
            payment_status: PaymentStatus::Pending,
            amount,
            currency: "USD".to_string(),
            house_bank: "BANK01".to_string(),
            bank_account_id: "001".to_string(),
            partner_bank_account: None,
            value_date: payment_date,
            check_number: None,
            wire_reference: None,
            allocations: Vec::new(),
            total_discount: Decimal::ZERO,
            total_write_off: Decimal::ZERO,
            bank_charges: Decimal::ZERO,
            exchange_rate: Decimal::ONE,
            fx_gain_loss: Decimal::ZERO,
            payment_run_id: None,
            is_bank_cleared: false,
            bank_statement_ref: None,
            cleared_date: None,
            is_voided: false,
            void_reason: None,
        }
    }

    /// Set payment method.
    pub fn with_payment_method(mut self, method: PaymentMethod) -> Self {
        self.payment_method = method;
        self
    }

    /// Set house bank.
    pub fn with_bank(
        mut self,
        house_bank: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Self {
        self.house_bank = house_bank.into();
        self.bank_account_id = account_id.into();
        self
    }

    /// Set check number.
    pub fn with_check_number(mut self, check_number: impl Into<String>) -> Self {
        self.check_number = Some(check_number.into());
        self.payment_method = PaymentMethod::Check;
        self
    }

    /// Set value date.
    pub fn with_value_date(mut self, date: NaiveDate) -> Self {
        self.value_date = date;
        self
    }

    /// Add an allocation.
    pub fn add_allocation(&mut self, allocation: PaymentAllocation) {
        // Add reference to the invoice
        self.header.add_reference(
            DocumentReference::new(
                allocation.invoice_type,
                allocation.invoice_id.clone(),
                self.header.document_type,
                self.header.document_id.clone(),
                ReferenceType::Payment,
                self.header.company_code.clone(),
                self.header.document_date,
            )
            .with_amount(allocation.amount),
        );

        self.allocations.push(allocation);
        self.recalculate_totals();
    }

    /// Allocate to an invoice.
    pub fn allocate_to_invoice(
        &mut self,
        invoice_id: impl Into<String>,
        invoice_type: DocumentType,
        amount: Decimal,
        discount: Decimal,
    ) {
        let allocation =
            PaymentAllocation::new(invoice_id, invoice_type, amount).with_discount(discount);
        self.add_allocation(allocation);
    }

    /// Recalculate totals.
    pub fn recalculate_totals(&mut self) {
        self.total_discount = self.allocations.iter().map(|a| a.discount_taken).sum();
        self.total_write_off = self.allocations.iter().map(|a| a.write_off).sum();
    }

    /// Total allocated amount.
    pub fn total_allocated(&self) -> Decimal {
        self.allocations.iter().map(|a| a.amount).sum()
    }

    /// Unallocated amount.
    pub fn unallocated(&self) -> Decimal {
        self.amount - self.total_allocated()
    }

    /// Approve the payment.
    pub fn approve(&mut self, user: impl Into<String>) {
        self.payment_status = PaymentStatus::Approved;
        self.header.update_status(DocumentStatus::Approved, user);
    }

    /// Send to bank.
    pub fn send_to_bank(&mut self, user: impl Into<String>) {
        self.payment_status = PaymentStatus::Sent;
        self.header.update_status(DocumentStatus::Released, user);
    }

    /// Record bank clearing.
    pub fn clear(&mut self, clear_date: NaiveDate, statement_ref: impl Into<String>) {
        self.is_bank_cleared = true;
        self.cleared_date = Some(clear_date);
        self.bank_statement_ref = Some(statement_ref.into());
        self.payment_status = PaymentStatus::Cleared;
        self.header.update_status(DocumentStatus::Cleared, "SYSTEM");

        // Mark all allocations as cleared
        for allocation in &mut self.allocations {
            allocation.is_cleared = true;
        }
    }

    /// Void the payment.
    pub fn void(&mut self, reason: impl Into<String>, user: impl Into<String>) {
        self.is_voided = true;
        self.void_reason = Some(reason.into());
        self.payment_status = PaymentStatus::Cancelled;
        self.header.update_status(DocumentStatus::Cancelled, user);
    }

    /// Post the payment.
    pub fn post(&mut self, user: impl Into<String>, posting_date: NaiveDate) {
        self.header.posting_date = Some(posting_date);
        self.header.update_status(DocumentStatus::Posted, user);
    }

    /// Generate GL entries for payment.
    pub fn generate_gl_entries(&self) -> Vec<(String, Decimal, Decimal)> {
        let mut entries = Vec::new();

        if self.is_vendor {
            // AP Payment: DR AP, CR Bank
            entries.push(("210000".to_string(), self.amount, Decimal::ZERO)); // AP
            entries.push(("110000".to_string(), Decimal::ZERO, self.amount)); // Bank

            if self.total_discount > Decimal::ZERO {
                entries.push(("740000".to_string(), Decimal::ZERO, self.total_discount));
                // Purchase discount
            }
        } else {
            // AR Receipt: DR Bank, CR AR
            entries.push(("110000".to_string(), self.amount, Decimal::ZERO)); // Bank
            entries.push(("120000".to_string(), Decimal::ZERO, self.amount)); // AR

            if self.total_discount > Decimal::ZERO {
                entries.push(("440000".to_string(), self.total_discount, Decimal::ZERO));
                // Sales discount
            }
        }

        entries
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_ap_payment_creation() {
        let payment = Payment::new_ap_payment(
            "PAY-1000-0000000001",
            "1000",
            "V-000001",
            Decimal::from(1000),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(payment.amount, Decimal::from(1000));
        assert!(payment.is_vendor);
        assert_eq!(payment.payment_type, PaymentType::ApPayment);
    }

    #[test]
    fn test_ar_receipt_creation() {
        let payment = Payment::new_ar_receipt(
            "REC-1000-0000000001",
            "1000",
            "C-000001",
            Decimal::from(5000),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        assert_eq!(payment.amount, Decimal::from(5000));
        assert!(!payment.is_vendor);
        assert_eq!(payment.payment_type, PaymentType::ArReceipt);
    }

    #[test]
    fn test_payment_allocation() {
        let mut payment = Payment::new_ap_payment(
            "PAY-1000-0000000001",
            "1000",
            "V-000001",
            Decimal::from(1000),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        payment.allocate_to_invoice(
            "VI-1000-0000000001",
            DocumentType::VendorInvoice,
            Decimal::from(980),
            Decimal::from(20),
        );

        assert_eq!(payment.total_allocated(), Decimal::from(980));
        assert_eq!(payment.total_discount, Decimal::from(20));
        assert_eq!(payment.unallocated(), Decimal::from(20));
    }

    #[test]
    fn test_payment_workflow() {
        let mut payment = Payment::new_ap_payment(
            "PAY-1000-0000000001",
            "1000",
            "V-000001",
            Decimal::from(1000),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        payment.approve("MANAGER");
        assert_eq!(payment.payment_status, PaymentStatus::Approved);

        payment.send_to_bank("TREASURY");
        assert_eq!(payment.payment_status, PaymentStatus::Sent);

        payment.clear(
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            "STMT-2024-01-17-001",
        );
        assert!(payment.is_bank_cleared);
        assert_eq!(payment.payment_status, PaymentStatus::Cleared);
    }
}
