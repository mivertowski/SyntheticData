//! Expense report models for the Hire-to-Retire (H2R) process.
//!
//! These models represent employee expense reports and their line items,
//! supporting the full expense lifecycle from draft submission through payment.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of an expense report through the approval and payment lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseStatus {
    /// Initial draft, not yet submitted
    #[default]
    Draft,
    /// Submitted for approval
    Submitted,
    /// Approved by manager
    Approved,
    /// Rejected by manager
    Rejected,
    /// Reimbursement paid to employee
    Paid,
}

/// Category of an expense line item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategory {
    /// Airfare, mileage, etc.
    Travel,
    /// Business meals and dining
    Meals,
    /// Hotel and accommodation
    Lodging,
    /// Taxi, rideshare, rental car, parking
    Transportation,
    /// Office supplies and equipment
    Office,
    /// Client entertainment
    Entertainment,
    /// Professional development and training
    Training,
    /// Miscellaneous expenses
    Other,
}

/// An expense report submitted by an employee for reimbursement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseReport {
    /// Unique expense report identifier
    pub report_id: String,
    /// Employee who submitted the report
    pub employee_id: String,
    /// Date the report was submitted
    pub submission_date: NaiveDate,
    /// Overall description/purpose of the expense report
    pub description: String,
    /// Current status of the expense report
    pub status: ExpenseStatus,
    /// Total amount across all line items
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// Individual expense line items
    pub line_items: Vec<ExpenseLineItem>,
    /// Manager who approved/rejected the report
    pub approved_by: Option<String>,
    /// Date the report was approved
    pub approved_date: Option<NaiveDate>,
    /// Date the reimbursement was paid
    pub paid_date: Option<NaiveDate>,
    /// Cost center to charge
    pub cost_center: Option<String>,
    /// Department to charge
    pub department: Option<String>,
    /// List of policy violations flagged on this report
    pub policy_violations: Vec<String>,
}

/// An individual line item within an expense report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseLineItem {
    /// Unique line item identifier
    pub item_id: String,
    /// Expense category
    pub category: ExpenseCategory,
    /// Date the expense was incurred
    pub date: NaiveDate,
    /// Amount of the expense
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// Description of the expense
    pub description: String,
    /// Whether a receipt is attached
    pub receipt_attached: bool,
    /// Merchant or vendor name
    pub merchant: Option<String>,
}
