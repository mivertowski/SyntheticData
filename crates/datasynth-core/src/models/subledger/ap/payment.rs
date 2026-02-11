//! AP Payment model.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::subledger::{CurrencyAmount, GLReference};

/// AP Payment (payment to vendor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APPayment {
    /// Unique payment number.
    pub payment_number: String,
    /// Company code.
    pub company_code: String,
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Payment date.
    pub payment_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Value date (bank value date).
    pub value_date: NaiveDate,
    /// Payment type.
    pub payment_type: APPaymentType,
    /// Payment status.
    pub status: PaymentStatus,
    /// Payment amount.
    pub amount: CurrencyAmount,
    /// Bank charges.
    pub bank_charges: Decimal,
    /// Discount taken.
    pub discount_taken: Decimal,
    /// Withholding tax.
    pub withholding_tax: Decimal,
    /// Net payment amount.
    pub net_payment: Decimal,
    /// Payment method.
    pub payment_method: APPaymentMethod,
    /// House bank.
    pub house_bank: String,
    /// Bank account.
    pub bank_account: String,
    /// Vendor bank account.
    pub vendor_bank_account: Option<String>,
    /// Check number (if check payment).
    pub check_number: Option<String>,
    /// Wire reference.
    pub wire_reference: Option<String>,
    /// Paid invoices.
    pub paid_invoices: Vec<PaymentAllocation>,
    /// GL references.
    pub gl_references: Vec<GLReference>,
    /// Payment run ID.
    pub payment_run_id: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Approved by.
    pub approved_by: Option<String>,
    /// Approval date.
    pub approved_date: Option<NaiveDate>,
    /// Notes.
    pub notes: Option<String>,
}

impl APPayment {
    /// Creates a new AP payment.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_number: String,
        company_code: String,
        vendor_id: String,
        vendor_name: String,
        payment_date: NaiveDate,
        amount: Decimal,
        currency: String,
        payment_method: APPaymentMethod,
        house_bank: String,
        bank_account: String,
    ) -> Self {
        Self {
            payment_number,
            company_code,
            vendor_id,
            vendor_name,
            payment_date,
            posting_date: payment_date,
            value_date: payment_date,
            payment_type: APPaymentType::Standard,
            status: PaymentStatus::Created,
            amount: CurrencyAmount::single_currency(amount, currency),
            bank_charges: Decimal::ZERO,
            discount_taken: Decimal::ZERO,
            withholding_tax: Decimal::ZERO,
            net_payment: amount,
            payment_method,
            house_bank,
            bank_account,
            vendor_bank_account: None,
            check_number: None,
            wire_reference: None,
            paid_invoices: Vec::new(),
            gl_references: Vec::new(),
            payment_run_id: None,
            created_at: Utc::now(),
            created_by: None,
            approved_by: None,
            approved_date: None,
            notes: None,
        }
    }

    /// Allocates payment to an invoice.
    pub fn allocate_to_invoice(
        &mut self,
        invoice_number: String,
        amount_paid: Decimal,
        discount: Decimal,
        withholding: Decimal,
    ) {
        let allocation = PaymentAllocation {
            invoice_number,
            amount_paid,
            discount_taken: discount,
            withholding_tax: withholding,
            allocation_date: self.payment_date,
        };

        self.paid_invoices.push(allocation);
        self.discount_taken += discount;
        self.withholding_tax += withholding;
        self.recalculate_net_payment();
    }

    /// Recalculates net payment amount.
    fn recalculate_net_payment(&mut self) {
        self.net_payment = self.amount.document_amount
            - self.discount_taken
            - self.withholding_tax
            - self.bank_charges;
    }

    /// Sets bank charges.
    pub fn with_bank_charges(mut self, charges: Decimal) -> Self {
        self.bank_charges = charges;
        self.recalculate_net_payment();
        self
    }

    /// Sets check number.
    pub fn with_check(mut self, check_number: String) -> Self {
        self.check_number = Some(check_number);
        self.payment_method = APPaymentMethod::Check;
        self
    }

    /// Sets wire reference.
    pub fn with_wire_reference(mut self, reference: String) -> Self {
        self.wire_reference = Some(reference);
        self
    }

    /// Sets vendor bank account.
    pub fn with_vendor_bank(mut self, bank_account: String) -> Self {
        self.vendor_bank_account = Some(bank_account);
        self
    }

    /// Approves the payment.
    pub fn approve(&mut self, approver: String, approval_date: NaiveDate) {
        self.status = PaymentStatus::Approved;
        self.approved_by = Some(approver);
        self.approved_date = Some(approval_date);
    }

    /// Releases the payment for processing.
    pub fn release(&mut self) {
        if self.status == PaymentStatus::Approved {
            self.status = PaymentStatus::Released;
        }
    }

    /// Confirms payment was sent.
    pub fn confirm_sent(&mut self, reference: Option<String>) {
        self.status = PaymentStatus::Sent;
        if let Some(ref_num) = reference {
            self.wire_reference = Some(ref_num);
        }
    }

    /// Confirms payment cleared the bank.
    pub fn confirm_cleared(&mut self, value_date: NaiveDate) {
        self.status = PaymentStatus::Cleared;
        self.value_date = value_date;
    }

    /// Voids the payment.
    pub fn void(&mut self, reason: String) {
        self.status = PaymentStatus::Voided;
        self.notes = Some(format!(
            "{}Voided: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{}. ", n))
                .unwrap_or_default(),
            reason
        ));
    }

    /// Gets total settlement amount (payment + discount + withholding).
    pub fn total_settlement(&self) -> Decimal {
        self.paid_invoices
            .iter()
            .map(|a| a.total_settlement())
            .sum()
    }

    /// Adds a GL reference.
    pub fn add_gl_reference(&mut self, reference: GLReference) {
        self.gl_references.push(reference);
    }
}

/// Type of AP payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum APPaymentType {
    /// Standard payment.
    #[default]
    Standard,
    /// Down payment.
    DownPayment,
    /// Partial payment.
    Partial,
    /// Final payment.
    Final,
    /// Urgent/rush payment.
    Urgent,
    /// Intercompany payment.
    Intercompany,
}

/// Payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaymentStatus {
    /// Created, awaiting approval.
    #[default]
    Created,
    /// Approved, awaiting release.
    Approved,
    /// Released for processing.
    Released,
    /// Sent to bank.
    Sent,
    /// Cleared/reconciled.
    Cleared,
    /// Voided.
    Voided,
    /// Returned/rejected.
    Returned,
}

/// Payment method for AP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum APPaymentMethod {
    /// Wire transfer.
    #[default]
    WireTransfer,
    /// ACH/Direct debit.
    ACH,
    /// Check.
    Check,
    /// SEPA transfer.
    SEPA,
    /// Credit card.
    CreditCard,
    /// Virtual card.
    VirtualCard,
    /// Intercompany netting.
    Netting,
    /// Letter of credit.
    LetterOfCredit,
}

/// Allocation of payment to an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAllocation {
    /// Invoice number.
    pub invoice_number: String,
    /// Amount paid.
    pub amount_paid: Decimal,
    /// Discount taken.
    pub discount_taken: Decimal,
    /// Withholding tax.
    pub withholding_tax: Decimal,
    /// Allocation date.
    pub allocation_date: NaiveDate,
}

impl PaymentAllocation {
    /// Total settlement for this allocation.
    pub fn total_settlement(&self) -> Decimal {
        self.amount_paid + self.discount_taken + self.withholding_tax
    }
}

/// Payment proposal (before actual payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProposal {
    /// Proposal ID.
    pub proposal_id: String,
    /// Company code.
    pub company_code: String,
    /// Run date.
    pub run_date: NaiveDate,
    /// Payment date.
    pub payment_date: NaiveDate,
    /// Proposal status.
    pub status: ProposalStatus,
    /// Payment method.
    pub payment_method: APPaymentMethod,
    /// Proposed payments.
    pub proposed_payments: Vec<ProposedPayment>,
    /// Total payment amount.
    pub total_amount: Decimal,
    /// Total discount available.
    pub total_discount: Decimal,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
}

impl PaymentProposal {
    /// Creates a new payment proposal.
    pub fn new(
        proposal_id: String,
        company_code: String,
        run_date: NaiveDate,
        payment_date: NaiveDate,
        payment_method: APPaymentMethod,
        created_by: String,
    ) -> Self {
        Self {
            proposal_id,
            company_code,
            run_date,
            payment_date,
            status: ProposalStatus::Draft,
            payment_method,
            proposed_payments: Vec::new(),
            total_amount: Decimal::ZERO,
            total_discount: Decimal::ZERO,
            created_by,
            created_at: Utc::now(),
        }
    }

    /// Adds a proposed payment.
    pub fn add_payment(&mut self, payment: ProposedPayment) {
        self.total_amount += payment.amount;
        self.total_discount += payment.discount;
        self.proposed_payments.push(payment);
    }

    /// Gets count of proposed payments.
    pub fn payment_count(&self) -> usize {
        self.proposed_payments.len()
    }

    /// Gets count of invoices.
    pub fn invoice_count(&self) -> usize {
        self.proposed_payments
            .iter()
            .map(|p| p.invoices.len())
            .sum()
    }

    /// Submits for approval.
    pub fn submit(&mut self) {
        self.status = ProposalStatus::Submitted;
    }

    /// Approves the proposal.
    pub fn approve(&mut self) {
        self.status = ProposalStatus::Approved;
    }

    /// Executes the proposal (creates actual payments).
    pub fn execute(&mut self) {
        self.status = ProposalStatus::Executed;
    }
}

/// Status of payment proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Draft, can be modified.
    Draft,
    /// Submitted for approval.
    Submitted,
    /// Approved, ready to execute.
    Approved,
    /// Executed (payments created).
    Executed,
    /// Cancelled.
    Cancelled,
}

/// A proposed payment to a vendor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedPayment {
    /// Vendor ID.
    pub vendor_id: String,
    /// Vendor name.
    pub vendor_name: String,
    /// Payment amount.
    pub amount: Decimal,
    /// Discount amount.
    pub discount: Decimal,
    /// Withholding tax.
    pub withholding_tax: Decimal,
    /// Net payment.
    pub net_payment: Decimal,
    /// Currency.
    pub currency: String,
    /// Invoices included.
    pub invoices: Vec<ProposedInvoice>,
    /// Is selected for payment.
    pub is_selected: bool,
}

/// Invoice in a payment proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedInvoice {
    /// Invoice number.
    pub invoice_number: String,
    /// Invoice date.
    pub invoice_date: NaiveDate,
    /// Due date.
    pub due_date: NaiveDate,
    /// Open amount.
    pub open_amount: Decimal,
    /// Proposed payment.
    pub payment_amount: Decimal,
    /// Discount available.
    pub discount: Decimal,
    /// Days until due (negative if overdue).
    pub days_until_due: i32,
}

/// Payment run configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRunConfig {
    /// Company codes to include.
    pub company_codes: Vec<String>,
    /// Payment methods to use.
    pub payment_methods: Vec<APPaymentMethod>,
    /// Due date cutoff (pay items due by this date).
    pub due_date_cutoff: NaiveDate,
    /// Include discount items (pay early for discount).
    pub include_discount_items: bool,
    /// Discount date cutoff.
    pub discount_date_cutoff: Option<NaiveDate>,
    /// Maximum payment amount per vendor.
    pub max_amount_per_vendor: Option<Decimal>,
    /// Minimum payment amount.
    pub min_payment_amount: Decimal,
    /// Exclude blocked items.
    pub exclude_blocked: bool,
    /// Vendor filter (if empty, all vendors).
    pub vendor_filter: Vec<String>,
}

impl Default for PaymentRunConfig {
    fn default() -> Self {
        Self {
            company_codes: Vec::new(),
            payment_methods: vec![APPaymentMethod::WireTransfer, APPaymentMethod::Check],
            due_date_cutoff: chrono::Local::now().date_naive() + chrono::Duration::days(7),
            include_discount_items: true,
            discount_date_cutoff: None,
            max_amount_per_vendor: None,
            min_payment_amount: dec!(0.01),
            exclude_blocked: true,
            vendor_filter: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_creation() {
        let payment = APPayment::new(
            "PAY001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec!(1000),
            "USD".to_string(),
            APPaymentMethod::WireTransfer,
            "BANK01".to_string(),
            "100001".to_string(),
        );

        assert_eq!(payment.amount.document_amount, dec!(1000));
        assert_eq!(payment.status, PaymentStatus::Created);
    }

    #[test]
    fn test_payment_allocation() {
        let mut payment = APPayment::new(
            "PAY001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec!(1000),
            "USD".to_string(),
            APPaymentMethod::WireTransfer,
            "BANK01".to_string(),
            "100001".to_string(),
        );

        payment.allocate_to_invoice(
            "INV001".to_string(),
            dec!(980),
            dec!(20), // 2% discount
            Decimal::ZERO,
        );

        assert_eq!(payment.discount_taken, dec!(20));
        assert_eq!(payment.total_settlement(), dec!(1000));
        assert_eq!(payment.paid_invoices.len(), 1);
    }

    #[test]
    fn test_payment_workflow() {
        let mut payment = APPayment::new(
            "PAY001".to_string(),
            "1000".to_string(),
            "VEND001".to_string(),
            "Test Vendor".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec!(1000),
            "USD".to_string(),
            APPaymentMethod::Check,
            "BANK01".to_string(),
            "100001".to_string(),
        )
        .with_check("CHK12345".to_string());

        assert_eq!(payment.status, PaymentStatus::Created);

        payment.approve(
            "APPROVER1".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        );
        assert_eq!(payment.status, PaymentStatus::Approved);

        payment.release();
        assert_eq!(payment.status, PaymentStatus::Released);

        payment.confirm_sent(None);
        assert_eq!(payment.status, PaymentStatus::Sent);

        payment.confirm_cleared(NaiveDate::from_ymd_opt(2024, 2, 18).unwrap());
        assert_eq!(payment.status, PaymentStatus::Cleared);
    }

    #[test]
    fn test_payment_proposal() {
        let mut proposal = PaymentProposal::new(
            "PROP001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            APPaymentMethod::WireTransfer,
            "USER1".to_string(),
        );

        let payment = ProposedPayment {
            vendor_id: "VEND001".to_string(),
            vendor_name: "Test Vendor".to_string(),
            amount: dec!(5000),
            discount: dec!(100),
            withholding_tax: Decimal::ZERO,
            net_payment: dec!(4900),
            currency: "USD".to_string(),
            invoices: vec![ProposedInvoice {
                invoice_number: "INV001".to_string(),
                invoice_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                due_date: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
                open_amount: dec!(5000),
                payment_amount: dec!(4900),
                discount: dec!(100),
                days_until_due: 0,
            }],
            is_selected: true,
        };

        proposal.add_payment(payment);

        assert_eq!(proposal.payment_count(), 1);
        assert_eq!(proposal.invoice_count(), 1);
        assert_eq!(proposal.total_amount, dec!(5000));
        assert_eq!(proposal.total_discount, dec!(100));
    }
}
