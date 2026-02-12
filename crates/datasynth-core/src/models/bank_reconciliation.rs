//! Bank reconciliation models for matching payments to bank statement lines.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a bank reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationStatus {
    /// Reconciliation in progress
    #[default]
    InProgress,
    /// Reconciliation completed and balanced
    Completed,
    /// Reconciliation completed with unresolved items
    CompletedWithExceptions,
}

/// Direction of a bank statement line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Money coming in (credit to bank)
    Inflow,
    /// Money going out (debit to bank)
    Outflow,
}

/// Match status for a bank statement line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    /// Not yet matched
    #[default]
    Unmatched,
    /// Auto-matched to a payment/receipt
    AutoMatched,
    /// Manually matched
    ManuallyMatched,
    /// Bank charge (no matching payment expected)
    BankCharge,
    /// Interest (no matching payment expected)
    Interest,
}

/// A bank statement line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementLine {
    /// Unique statement line identifier
    pub line_id: String,
    /// Bank account ID
    pub bank_account_id: String,
    /// Statement date
    pub statement_date: NaiveDate,
    /// Value date
    pub value_date: NaiveDate,
    /// Transaction amount (positive = inflow, negative = outflow)
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Direction
    pub direction: Direction,
    /// Transaction description from bank
    pub description: String,
    /// Bank reference number
    pub bank_reference: String,
    /// Match status
    pub match_status: MatchStatus,
    /// Matched internal document ID (payment/receipt)
    pub matched_document_id: Option<String>,
    /// Company code
    pub company_code: String,
}

/// Type of reconciling item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcilingItemType {
    /// Check written but not yet cleared at bank
    OutstandingCheck,
    /// Deposit recorded but not yet credited by bank
    DepositInTransit,
    /// Bank charge not yet recorded in books
    BankCharge,
    /// Interest earned not yet recorded
    InterestEarned,
    /// NSF/returned check
    ReturnedCheck,
    /// Error correction
    ErrorCorrection,
}

/// A reconciling item that explains the difference between book and bank balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcilingItem {
    /// Item identifier
    pub item_id: String,
    /// Type of reconciling item
    pub item_type: ReconcilingItemType,
    /// Related document ID
    pub document_id: Option<String>,
    /// Amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Date of the item
    pub date: NaiveDate,
    /// Description
    pub description: String,
    /// Expected clearing date
    pub expected_clearing_date: Option<NaiveDate>,
}

/// A bank reconciliation for a specific account and period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReconciliation {
    /// Unique reconciliation identifier
    pub reconciliation_id: String,
    /// Bank account ID
    pub bank_account_id: String,
    /// Company code
    pub company_code: String,
    /// Reconciliation date (as-of date)
    pub reconciliation_date: NaiveDate,
    /// Status
    pub status: ReconciliationStatus,
    /// Bank statement ending balance
    #[serde(with = "rust_decimal::serde::str")]
    pub bank_ending_balance: Decimal,
    /// Book (GL) ending balance
    #[serde(with = "rust_decimal::serde::str")]
    pub book_ending_balance: Decimal,
    /// Bank statement lines for this period
    pub statement_lines: Vec<BankStatementLine>,
    /// Reconciling items
    pub reconciling_items: Vec<ReconcilingItem>,
    /// Net difference after reconciling items (should be zero)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_difference: Decimal,
    /// Opening bank balance
    #[serde(with = "rust_decimal::serde::str")]
    pub opening_balance: Decimal,
    /// Preparer ID
    pub preparer_id: String,
    /// Reviewer ID
    pub reviewer_id: Option<String>,
}
