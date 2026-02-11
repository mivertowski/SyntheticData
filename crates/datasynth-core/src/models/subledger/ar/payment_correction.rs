//! Payment correction and adjustment models for AR.
//!
//! This module provides models for:
//! - Payment corrections (NSF, chargebacks, reversals)
//! - Short payments (unauthorized deductions)
//! - On-account payments (unapplied customer payments)

#![allow(clippy::too_many_arguments)]

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A payment correction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCorrection {
    /// Unique correction identifier.
    pub correction_id: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Original payment ID being corrected.
    pub original_payment_id: String,
    /// Type of correction.
    pub correction_type: PaymentCorrectionType,
    /// Original payment amount.
    pub original_amount: Decimal,
    /// Correction amount (positive for reversals, negative for adjustments).
    pub correction_amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Date of correction.
    pub correction_date: NaiveDate,
    /// Reversal journal entry ID if applicable.
    pub reversal_je_id: Option<String>,
    /// Correcting payment ID (new payment if re-processed).
    pub correcting_payment_id: Option<String>,
    /// Related invoice IDs affected.
    pub affected_invoice_ids: Vec<String>,
    /// Status of the correction.
    pub status: CorrectionStatus,
    /// Reason for correction.
    pub reason: Option<String>,
    /// Bank reference (for NSF/chargeback).
    pub bank_reference: Option<String>,
    /// Chargeback code if applicable.
    pub chargeback_code: Option<String>,
    /// Fee amount (bank fees, chargeback fees).
    pub fee_amount: Decimal,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Resolved timestamp.
    pub resolved_at: Option<DateTime<Utc>>,
    /// Notes.
    pub notes: Option<String>,
}

impl PaymentCorrection {
    /// Creates a new payment correction.
    pub fn new(
        correction_id: String,
        company_code: String,
        customer_id: String,
        original_payment_id: String,
        correction_type: PaymentCorrectionType,
        original_amount: Decimal,
        correction_amount: Decimal,
        currency: String,
        correction_date: NaiveDate,
    ) -> Self {
        Self {
            correction_id,
            company_code,
            customer_id,
            original_payment_id,
            correction_type,
            original_amount,
            correction_amount,
            currency,
            correction_date,
            reversal_je_id: None,
            correcting_payment_id: None,
            affected_invoice_ids: Vec::new(),
            status: CorrectionStatus::Pending,
            reason: None,
            bank_reference: None,
            chargeback_code: None,
            fee_amount: Decimal::ZERO,
            created_at: Utc::now(),
            created_by: None,
            resolved_at: None,
            notes: None,
        }
    }

    /// Creates an NSF (Non-Sufficient Funds) correction.
    pub fn nsf(
        correction_id: String,
        company_code: String,
        customer_id: String,
        original_payment_id: String,
        original_amount: Decimal,
        currency: String,
        correction_date: NaiveDate,
        bank_reference: String,
        nsf_fee: Decimal,
    ) -> Self {
        let mut correction = Self::new(
            correction_id,
            company_code,
            customer_id,
            original_payment_id,
            PaymentCorrectionType::NSF,
            original_amount,
            original_amount, // Full reversal
            currency,
            correction_date,
        );
        correction.bank_reference = Some(bank_reference);
        correction.fee_amount = nsf_fee;
        correction.reason = Some("Payment returned - Non-Sufficient Funds".to_string());
        correction
    }

    /// Creates a chargeback correction.
    pub fn chargeback(
        correction_id: String,
        company_code: String,
        customer_id: String,
        original_payment_id: String,
        chargeback_amount: Decimal,
        currency: String,
        correction_date: NaiveDate,
        chargeback_code: String,
        reason: String,
    ) -> Self {
        let mut correction = Self::new(
            correction_id,
            company_code,
            customer_id,
            original_payment_id,
            PaymentCorrectionType::Chargeback,
            chargeback_amount,
            chargeback_amount,
            currency,
            correction_date,
        );
        correction.chargeback_code = Some(chargeback_code);
        correction.reason = Some(reason);
        correction
    }

    /// Sets the reversal journal entry.
    pub fn with_reversal_je(mut self, je_id: String) -> Self {
        self.reversal_je_id = Some(je_id);
        self
    }

    /// Adds an affected invoice.
    pub fn add_affected_invoice(&mut self, invoice_id: String) {
        self.affected_invoice_ids.push(invoice_id);
    }

    /// Processes the correction.
    pub fn process(&mut self, reversal_je_id: Option<String>) {
        self.status = CorrectionStatus::Processed;
        self.reversal_je_id = reversal_je_id;
    }

    /// Resolves the correction (e.g., customer re-paid).
    pub fn resolve(&mut self, correcting_payment_id: Option<String>) {
        self.status = CorrectionStatus::Resolved;
        self.correcting_payment_id = correcting_payment_id;
        self.resolved_at = Some(Utc::now());
    }

    /// Writes off the correction as bad debt.
    pub fn write_off(&mut self) {
        self.status = CorrectionStatus::WrittenOff;
        self.resolved_at = Some(Utc::now());
    }
}

/// Type of payment correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentCorrectionType {
    /// Non-Sufficient Funds (bounced check).
    NSF,
    /// Credit card chargeback.
    Chargeback,
    /// Wrong amount applied.
    WrongAmount,
    /// Payment applied to wrong customer.
    WrongCustomer,
    /// Duplicate payment received.
    DuplicatePayment,
    /// Payment reversal requested by customer.
    CustomerReversal,
    /// Bank error.
    BankError,
    /// System error.
    SystemError,
}

/// Status of a payment correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CorrectionStatus {
    /// Correction is pending processing.
    #[default]
    Pending,
    /// Correction has been processed (reversal posted).
    Processed,
    /// Correction has been resolved (re-payment received or written off).
    Resolved,
    /// Amount written off as bad debt.
    WrittenOff,
    /// Correction was cancelled/voided.
    Cancelled,
}

/// A short payment record (customer paid less than owed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPayment {
    /// Unique short payment identifier.
    pub short_payment_id: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Payment ID that was short.
    pub payment_id: String,
    /// Invoice ID that was shorted.
    pub invoice_id: String,
    /// Expected payment amount.
    pub expected_amount: Decimal,
    /// Actual paid amount.
    pub paid_amount: Decimal,
    /// Short amount (expected - paid).
    pub short_amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Payment date.
    pub payment_date: NaiveDate,
    /// Reason code for the short payment.
    pub reason_code: ShortPaymentReasonCode,
    /// Reason description.
    pub reason_description: Option<String>,
    /// Disposition of the short payment.
    pub disposition: ShortPaymentDisposition,
    /// Credit memo ID if issued.
    pub credit_memo_id: Option<String>,
    /// Write-off JE ID if written off.
    pub write_off_je_id: Option<String>,
    /// Re-bill invoice ID if re-billed.
    pub rebill_invoice_id: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Resolved timestamp.
    pub resolved_at: Option<DateTime<Utc>>,
    /// Notes.
    pub notes: Option<String>,
}

impl ShortPayment {
    /// Creates a new short payment record.
    pub fn new(
        short_payment_id: String,
        company_code: String,
        customer_id: String,
        payment_id: String,
        invoice_id: String,
        expected_amount: Decimal,
        paid_amount: Decimal,
        currency: String,
        payment_date: NaiveDate,
        reason_code: ShortPaymentReasonCode,
    ) -> Self {
        Self {
            short_payment_id,
            company_code,
            customer_id,
            payment_id,
            invoice_id,
            expected_amount,
            paid_amount,
            short_amount: expected_amount - paid_amount,
            currency,
            payment_date,
            reason_code,
            reason_description: None,
            disposition: ShortPaymentDisposition::Pending,
            credit_memo_id: None,
            write_off_je_id: None,
            rebill_invoice_id: None,
            created_at: Utc::now(),
            created_by: None,
            resolved_at: None,
            notes: None,
        }
    }

    /// Sets the reason description.
    pub fn with_reason(mut self, description: String) -> Self {
        self.reason_description = Some(description);
        self
    }

    /// Issues a credit memo for the short amount.
    pub fn issue_credit_memo(&mut self, credit_memo_id: String) {
        self.credit_memo_id = Some(credit_memo_id);
        self.disposition = ShortPaymentDisposition::CreditMemoIssued;
        self.resolved_at = Some(Utc::now());
    }

    /// Writes off the short amount.
    pub fn write_off(&mut self, write_off_je_id: String) {
        self.write_off_je_id = Some(write_off_je_id);
        self.disposition = ShortPaymentDisposition::WrittenOff;
        self.resolved_at = Some(Utc::now());
    }

    /// Re-bills the customer for the short amount.
    pub fn rebill(&mut self, rebill_invoice_id: String) {
        self.rebill_invoice_id = Some(rebill_invoice_id);
        self.disposition = ShortPaymentDisposition::Rebilled;
        self.resolved_at = Some(Utc::now());
    }

    /// Accepts the short amount (customer was correct).
    pub fn accept(&mut self, credit_memo_id: Option<String>) {
        self.credit_memo_id = credit_memo_id;
        self.disposition = ShortPaymentDisposition::Accepted;
        self.resolved_at = Some(Utc::now());
    }
}

/// Reason code for short payments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortPaymentReasonCode {
    /// Customer disputes the price.
    PricingDispute,
    /// Quality issue with goods/services.
    QualityIssue,
    /// Quantity discrepancy.
    QuantityDiscrepancy,
    /// Unauthorized deduction by customer.
    UnauthorizedDeduction,
    /// Early payment discount taken incorrectly.
    IncorrectDiscount,
    /// Freight/shipping dispute.
    FreightDispute,
    /// Tax dispute.
    TaxDispute,
    /// Return/allowance claim.
    ReturnAllowance,
    /// Co-op advertising deduction.
    CoopDeduction,
    /// Other/unspecified.
    Other,
}

/// Disposition of a short payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShortPaymentDisposition {
    /// Pending review.
    #[default]
    Pending,
    /// Under investigation.
    UnderInvestigation,
    /// Credit memo issued to customer.
    CreditMemoIssued,
    /// Amount written off.
    WrittenOff,
    /// Customer re-billed for amount.
    Rebilled,
    /// Short amount accepted (customer was correct).
    Accepted,
    /// Dispute rejected, pursuing collection.
    DisputeRejected,
}

/// An on-account payment (unapplied customer payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnAccountPayment {
    /// Unique identifier.
    pub on_account_id: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Original payment ID.
    pub payment_id: String,
    /// On-account amount.
    pub amount: Decimal,
    /// Remaining unapplied amount.
    pub remaining_amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Date received.
    pub received_date: NaiveDate,
    /// Status.
    pub status: OnAccountStatus,
    /// Applications of this on-account payment.
    pub applications: Vec<OnAccountApplication>,
    /// Reason for on-account posting.
    pub reason: Option<OnAccountReason>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl OnAccountPayment {
    /// Creates a new on-account payment.
    pub fn new(
        on_account_id: String,
        company_code: String,
        customer_id: String,
        payment_id: String,
        amount: Decimal,
        currency: String,
        received_date: NaiveDate,
    ) -> Self {
        Self {
            on_account_id,
            company_code,
            customer_id,
            payment_id,
            amount,
            remaining_amount: amount,
            currency,
            received_date,
            status: OnAccountStatus::Unapplied,
            applications: Vec::new(),
            reason: None,
            created_at: Utc::now(),
            created_by: None,
            notes: None,
        }
    }

    /// Sets the reason.
    pub fn with_reason(mut self, reason: OnAccountReason) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Applies a portion to an invoice.
    pub fn apply_to_invoice(
        &mut self,
        invoice_id: String,
        amount: Decimal,
        application_date: NaiveDate,
    ) -> bool {
        if amount > self.remaining_amount {
            return false;
        }

        self.applications.push(OnAccountApplication {
            invoice_id,
            amount,
            application_date,
        });

        self.remaining_amount -= amount;

        if self.remaining_amount <= Decimal::ZERO {
            self.status = OnAccountStatus::FullyApplied;
        } else {
            self.status = OnAccountStatus::PartiallyApplied;
        }

        true
    }

    /// Refunds the remaining amount.
    pub fn refund(&mut self) {
        self.status = OnAccountStatus::Refunded;
        self.remaining_amount = Decimal::ZERO;
    }

    /// Writes off the remaining amount.
    pub fn write_off(&mut self) {
        self.status = OnAccountStatus::WrittenOff;
        self.remaining_amount = Decimal::ZERO;
    }
}

/// Status of an on-account payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OnAccountStatus {
    /// Payment is unapplied.
    #[default]
    Unapplied,
    /// Payment is partially applied.
    PartiallyApplied,
    /// Payment is fully applied.
    FullyApplied,
    /// Payment has been refunded.
    Refunded,
    /// Remaining amount written off.
    WrittenOff,
}

/// Reason for on-account posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnAccountReason {
    /// No invoice reference provided.
    NoInvoiceReference,
    /// Customer overpaid.
    Overpayment,
    /// Prepayment for future invoice.
    Prepayment,
    /// Invoice not yet created.
    InvoicePending,
    /// Remittance unclear.
    UnclearRemittance,
    /// Other reason.
    Other,
}

/// Application of on-account payment to an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnAccountApplication {
    /// Invoice ID applied to.
    pub invoice_id: String,
    /// Amount applied.
    pub amount: Decimal,
    /// Date of application.
    pub application_date: NaiveDate,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_nsf_correction() {
        let correction = PaymentCorrection::nsf(
            "CORR-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "PAY-001".to_string(),
            Decimal::from(1000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            "BANK-REF-123".to_string(),
            Decimal::from(35),
        );

        assert_eq!(correction.correction_type, PaymentCorrectionType::NSF);
        assert_eq!(correction.correction_amount, Decimal::from(1000));
        assert_eq!(correction.fee_amount, Decimal::from(35));
        assert_eq!(correction.status, CorrectionStatus::Pending);
    }

    #[test]
    fn test_short_payment() {
        let mut short = ShortPayment::new(
            "SHORT-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "PAY-001".to_string(),
            "INV-001".to_string(),
            Decimal::from(1000),
            Decimal::from(950),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            ShortPaymentReasonCode::PricingDispute,
        );

        assert_eq!(short.short_amount, Decimal::from(50));
        assert_eq!(short.disposition, ShortPaymentDisposition::Pending);

        short.issue_credit_memo("CM-001".to_string());
        assert_eq!(short.disposition, ShortPaymentDisposition::CreditMemoIssued);
        assert!(short.credit_memo_id.is_some());
    }

    #[test]
    fn test_on_account_payment() {
        let mut on_account = OnAccountPayment::new(
            "OA-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "PAY-001".to_string(),
            Decimal::from(500),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        );

        assert_eq!(on_account.status, OnAccountStatus::Unapplied);
        assert_eq!(on_account.remaining_amount, Decimal::from(500));

        // Apply partial amount
        let applied = on_account.apply_to_invoice(
            "INV-001".to_string(),
            Decimal::from(300),
            NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
        );

        assert!(applied);
        assert_eq!(on_account.status, OnAccountStatus::PartiallyApplied);
        assert_eq!(on_account.remaining_amount, Decimal::from(200));

        // Apply remaining
        let applied = on_account.apply_to_invoice(
            "INV-002".to_string(),
            Decimal::from(200),
            NaiveDate::from_ymd_opt(2024, 3, 25).unwrap(),
        );

        assert!(applied);
        assert_eq!(on_account.status, OnAccountStatus::FullyApplied);
        assert_eq!(on_account.remaining_amount, Decimal::ZERO);
        assert_eq!(on_account.applications.len(), 2);
    }

    #[test]
    fn test_on_account_overapply_fails() {
        let mut on_account = OnAccountPayment::new(
            "OA-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "PAY-001".to_string(),
            Decimal::from(500),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        );

        let applied = on_account.apply_to_invoice(
            "INV-001".to_string(),
            Decimal::from(600), // More than available
            NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
        );

        assert!(!applied);
        assert_eq!(on_account.status, OnAccountStatus::Unapplied);
        assert_eq!(on_account.remaining_amount, Decimal::from(500));
    }
}
