//! Dunning (Mahnungen) models for AR collections.
//!
//! This module provides models for the dunning process including:
//! - Dunning runs (batch processing of overdue invoices)
//! - Dunning letters (reminders sent to customers)
//! - Dunning items (individual invoices included in a letter)

#![allow(clippy::too_many_arguments)]

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A dunning run represents a batch execution of the dunning process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningRun {
    /// Unique run identifier.
    pub run_id: String,
    /// Company code.
    pub company_code: String,
    /// Run execution date.
    pub run_date: NaiveDate,
    /// Dunning date used for calculations.
    pub dunning_date: NaiveDate,
    /// Number of customers evaluated.
    pub customers_evaluated: u32,
    /// Number of customers with letters generated.
    pub customers_with_letters: u32,
    /// Number of letters generated.
    pub letters_generated: u32,
    /// Total amount dunned across all letters.
    pub total_amount_dunned: Decimal,
    /// Total dunning charges applied.
    pub total_dunning_charges: Decimal,
    /// Total interest calculated.
    pub total_interest_amount: Decimal,
    /// Run status.
    pub status: DunningRunStatus,
    /// Letters generated in this run.
    pub letters: Vec<DunningLetter>,
    /// Started timestamp.
    pub started_at: DateTime<Utc>,
    /// Completed timestamp.
    pub completed_at: Option<DateTime<Utc>>,
    /// User who initiated the run.
    pub created_by: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl DunningRun {
    /// Creates a new dunning run.
    pub fn new(run_id: String, company_code: String, run_date: NaiveDate) -> Self {
        Self {
            run_id,
            company_code,
            run_date,
            dunning_date: run_date,
            customers_evaluated: 0,
            customers_with_letters: 0,
            letters_generated: 0,
            total_amount_dunned: Decimal::ZERO,
            total_dunning_charges: Decimal::ZERO,
            total_interest_amount: Decimal::ZERO,
            status: DunningRunStatus::Pending,
            letters: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            created_by: None,
            notes: None,
        }
    }

    /// Adds a dunning letter to the run.
    pub fn add_letter(&mut self, letter: DunningLetter) {
        self.total_amount_dunned += letter.total_dunned_amount;
        self.total_dunning_charges += letter.dunning_charges;
        self.total_interest_amount += letter.interest_amount;
        self.letters_generated += 1;
        self.letters.push(letter);
    }

    /// Marks the run as started.
    pub fn start(&mut self) {
        self.status = DunningRunStatus::InProgress;
        self.started_at = Utc::now();
    }

    /// Marks the run as completed.
    pub fn complete(&mut self) {
        self.status = DunningRunStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.customers_with_letters = self
            .letters
            .iter()
            .map(|l| l.customer_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;
    }

    /// Marks the run as failed.
    pub fn fail(&mut self, reason: String) {
        self.status = DunningRunStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.notes = Some(reason);
    }
}

/// Status of a dunning run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DunningRunStatus {
    /// Run is pending execution.
    #[default]
    Pending,
    /// Run is in progress.
    InProgress,
    /// Run completed successfully.
    Completed,
    /// Run failed.
    Failed,
    /// Run was cancelled.
    Cancelled,
}

/// A dunning letter sent to a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetter {
    /// Unique letter identifier.
    pub letter_id: String,
    /// Reference to the dunning run.
    pub dunning_run_id: String,
    /// Company code.
    pub company_code: String,
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Dunning level (1-4).
    pub dunning_level: u8,
    /// Date of the dunning letter.
    pub dunning_date: NaiveDate,
    /// Items included in this letter.
    pub dunning_items: Vec<DunningItem>,
    /// Total amount being dunned.
    pub total_dunned_amount: Decimal,
    /// Dunning charges applied.
    pub dunning_charges: Decimal,
    /// Interest amount calculated.
    pub interest_amount: Decimal,
    /// Total amount due (principal + charges + interest).
    pub total_amount_due: Decimal,
    /// Currency.
    pub currency: String,
    /// Payment deadline.
    pub payment_deadline: NaiveDate,
    /// Whether the letter has been sent.
    pub is_sent: bool,
    /// Date sent.
    pub sent_date: Option<NaiveDate>,
    /// Response received from customer.
    pub response_type: Option<DunningResponseType>,
    /// Response date.
    pub response_date: Option<NaiveDate>,
    /// Status of the letter.
    pub status: DunningLetterStatus,
    /// Customer contact address.
    pub contact_address: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl DunningLetter {
    /// Creates a new dunning letter.
    pub fn new(
        letter_id: String,
        dunning_run_id: String,
        company_code: String,
        customer_id: String,
        customer_name: String,
        dunning_level: u8,
        dunning_date: NaiveDate,
        payment_deadline: NaiveDate,
        currency: String,
    ) -> Self {
        Self {
            letter_id,
            dunning_run_id,
            company_code,
            customer_id,
            customer_name,
            dunning_level,
            dunning_date,
            dunning_items: Vec::new(),
            total_dunned_amount: Decimal::ZERO,
            dunning_charges: Decimal::ZERO,
            interest_amount: Decimal::ZERO,
            total_amount_due: Decimal::ZERO,
            currency,
            payment_deadline,
            is_sent: false,
            sent_date: None,
            response_type: None,
            response_date: None,
            status: DunningLetterStatus::Created,
            contact_address: None,
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Adds a dunning item to the letter.
    pub fn add_item(&mut self, item: DunningItem) {
        self.total_dunned_amount += item.open_amount;
        self.dunning_items.push(item);
        self.recalculate_totals();
    }

    /// Sets dunning charges.
    pub fn set_charges(&mut self, charges: Decimal) {
        self.dunning_charges = charges;
        self.recalculate_totals();
    }

    /// Sets interest amount.
    pub fn set_interest(&mut self, interest: Decimal) {
        self.interest_amount = interest;
        self.recalculate_totals();
    }

    /// Recalculates total amount due.
    fn recalculate_totals(&mut self) {
        self.total_amount_due =
            self.total_dunned_amount + self.dunning_charges + self.interest_amount;
    }

    /// Marks the letter as sent.
    pub fn mark_sent(&mut self, sent_date: NaiveDate) {
        self.is_sent = true;
        self.sent_date = Some(sent_date);
        self.status = DunningLetterStatus::Sent;
    }

    /// Records customer response.
    pub fn record_response(&mut self, response: DunningResponseType, response_date: NaiveDate) {
        self.response_type = Some(response);
        self.response_date = Some(response_date);
        self.status = match response {
            DunningResponseType::PaymentPromise | DunningResponseType::Paid => {
                DunningLetterStatus::Resolved
            }
            DunningResponseType::Dispute | DunningResponseType::PartialDispute => {
                DunningLetterStatus::Disputed
            }
            DunningResponseType::NoResponse => DunningLetterStatus::Sent,
            DunningResponseType::PaymentPlan => DunningLetterStatus::Resolved,
            DunningResponseType::Bankruptcy => DunningLetterStatus::WrittenOff,
        };
    }

    /// Marks as escalated to collection.
    pub fn escalate_to_collection(&mut self) {
        self.status = DunningLetterStatus::EscalatedToCollection;
    }
}

/// Status of a dunning letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DunningLetterStatus {
    /// Letter created but not sent.
    #[default]
    Created,
    /// Letter sent to customer.
    Sent,
    /// Customer disputed the charges.
    Disputed,
    /// Matter resolved (paid or payment plan).
    Resolved,
    /// Escalated to collection agency.
    EscalatedToCollection,
    /// Written off as bad debt.
    WrittenOff,
    /// Letter cancelled.
    Cancelled,
}

/// Type of response to a dunning letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DunningResponseType {
    /// No response received.
    NoResponse,
    /// Customer promised to pay.
    PaymentPromise,
    /// Customer paid the amount.
    Paid,
    /// Customer disputes the full amount.
    Dispute,
    /// Customer disputes part of the amount.
    PartialDispute,
    /// Customer requests payment plan.
    PaymentPlan,
    /// Customer filed for bankruptcy.
    Bankruptcy,
}

/// An individual invoice/item included in a dunning letter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningItem {
    /// Reference to the AR invoice.
    pub invoice_number: String,
    /// Invoice date.
    pub invoice_date: NaiveDate,
    /// Due date.
    pub due_date: NaiveDate,
    /// Original invoice amount.
    pub original_amount: Decimal,
    /// Open/remaining amount.
    pub open_amount: Decimal,
    /// Days overdue.
    pub days_overdue: u32,
    /// Interest calculated on this item.
    pub interest_amount: Decimal,
    /// Previous dunning level before this run.
    pub previous_dunning_level: u8,
    /// New dunning level after this run.
    pub new_dunning_level: u8,
    /// Whether this item was blocked from dunning.
    pub is_blocked: bool,
    /// Block reason if blocked.
    pub block_reason: Option<String>,
}

impl DunningItem {
    /// Creates a new dunning item.
    pub fn new(
        invoice_number: String,
        invoice_date: NaiveDate,
        due_date: NaiveDate,
        original_amount: Decimal,
        open_amount: Decimal,
        days_overdue: u32,
        previous_dunning_level: u8,
        new_dunning_level: u8,
    ) -> Self {
        Self {
            invoice_number,
            invoice_date,
            due_date,
            original_amount,
            open_amount,
            days_overdue,
            interest_amount: Decimal::ZERO,
            previous_dunning_level,
            new_dunning_level,
            is_blocked: false,
            block_reason: None,
        }
    }

    /// Sets the interest amount.
    pub fn with_interest(mut self, interest: Decimal) -> Self {
        self.interest_amount = interest;
        self
    }

    /// Blocks the item from dunning.
    pub fn block(mut self, reason: String) -> Self {
        self.is_blocked = true;
        self.block_reason = Some(reason);
        self
    }
}

/// Summary of dunning activity for a customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerDunningSummary {
    /// Customer ID.
    pub customer_id: String,
    /// Customer name.
    pub customer_name: String,
    /// Current highest dunning level.
    pub current_dunning_level: u8,
    /// Number of dunning letters sent.
    pub letters_sent: u32,
    /// Total amount currently dunned.
    pub total_dunned_amount: Decimal,
    /// Total dunning charges.
    pub total_charges: Decimal,
    /// Total interest accrued.
    pub total_interest: Decimal,
    /// Last dunning date.
    pub last_dunning_date: Option<NaiveDate>,
    /// Whether customer is blocked from dunning.
    pub is_blocked: bool,
    /// Whether customer is in collection.
    pub in_collection: bool,
}

impl CustomerDunningSummary {
    /// Creates from dunning letters.
    pub fn from_letters(
        customer_id: String,
        customer_name: String,
        letters: &[DunningLetter],
    ) -> Self {
        let customer_letters: Vec<_> = letters
            .iter()
            .filter(|l| l.customer_id == customer_id)
            .collect();

        let current_dunning_level = customer_letters
            .iter()
            .map(|l| l.dunning_level)
            .max()
            .unwrap_or(0);

        let total_dunned_amount: Decimal = customer_letters
            .iter()
            .filter(|l| l.status != DunningLetterStatus::Resolved)
            .map(|l| l.total_dunned_amount)
            .sum();

        let total_charges: Decimal = customer_letters.iter().map(|l| l.dunning_charges).sum();

        let total_interest: Decimal = customer_letters.iter().map(|l| l.interest_amount).sum();

        let last_dunning_date = customer_letters.iter().map(|l| l.dunning_date).max();

        let in_collection = customer_letters
            .iter()
            .any(|l| l.status == DunningLetterStatus::EscalatedToCollection);

        Self {
            customer_id,
            customer_name,
            current_dunning_level,
            letters_sent: customer_letters.iter().filter(|l| l.is_sent).count() as u32,
            total_dunned_amount,
            total_charges,
            total_interest,
            last_dunning_date,
            is_blocked: false,
            in_collection,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_dunning_run_creation() {
        let run = DunningRun::new(
            "DR-2024-001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        );

        assert_eq!(run.status, DunningRunStatus::Pending);
        assert_eq!(run.letters_generated, 0);
    }

    #[test]
    fn test_dunning_letter_creation() {
        let letter = DunningLetter::new(
            "DL-2024-001".to_string(),
            "DR-2024-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            1,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 29).unwrap(),
            "USD".to_string(),
        );

        assert_eq!(letter.dunning_level, 1);
        assert!(!letter.is_sent);
        assert_eq!(letter.status, DunningLetterStatus::Created);
    }

    #[test]
    fn test_dunning_letter_items() {
        let mut letter = DunningLetter::new(
            "DL-2024-001".to_string(),
            "DR-2024-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            1,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 29).unwrap(),
            "USD".to_string(),
        );

        let item = DunningItem::new(
            "INV-001".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            Decimal::from(1000),
            Decimal::from(1000),
            30,
            0,
            1,
        );

        letter.add_item(item);
        letter.set_charges(Decimal::from(25));
        letter.set_interest(Decimal::from(7)); // ~9% annual on $1000 for 30 days

        assert_eq!(letter.total_dunned_amount, Decimal::from(1000));
        assert_eq!(letter.dunning_charges, Decimal::from(25));
        assert_eq!(letter.interest_amount, Decimal::from(7));
        assert_eq!(letter.total_amount_due, Decimal::from(1032));
    }

    #[test]
    fn test_dunning_run_with_letters() {
        let mut run = DunningRun::new(
            "DR-2024-001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        );

        run.start();
        assert_eq!(run.status, DunningRunStatus::InProgress);

        let mut letter = DunningLetter::new(
            "DL-2024-001".to_string(),
            "DR-2024-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            1,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 29).unwrap(),
            "USD".to_string(),
        );

        letter.add_item(DunningItem::new(
            "INV-001".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            Decimal::from(1000),
            Decimal::from(1000),
            30,
            0,
            1,
        ));
        letter.set_charges(Decimal::from(25));

        run.add_letter(letter);
        run.complete();

        assert_eq!(run.status, DunningRunStatus::Completed);
        assert_eq!(run.letters_generated, 1);
        assert_eq!(run.total_amount_dunned, Decimal::from(1000));
        assert_eq!(run.total_dunning_charges, Decimal::from(25));
    }

    #[test]
    fn test_letter_response() {
        let mut letter = DunningLetter::new(
            "DL-2024-001".to_string(),
            "DR-2024-001".to_string(),
            "1000".to_string(),
            "CUST001".to_string(),
            "Test Customer".to_string(),
            1,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 29).unwrap(),
            "USD".to_string(),
        );

        letter.mark_sent(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
        assert!(letter.is_sent);
        assert_eq!(letter.status, DunningLetterStatus::Sent);

        letter.record_response(
            DunningResponseType::PaymentPromise,
            NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
        );
        assert_eq!(letter.status, DunningLetterStatus::Resolved);
    }
}
