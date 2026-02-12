//! Budget models for financial planning and variance analysis.
//!
//! These models represent organizational budgets and their line items,
//! supporting budget-vs-actual comparison and variance reporting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a budget through the planning and approval lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BudgetStatus {
    /// Initial draft, still being prepared
    #[default]
    Draft,
    /// Submitted for management approval
    Submitted,
    /// Approved by management
    Approved,
    /// Budget has been revised after initial approval
    Revised,
    /// Budget period has ended and the budget is closed
    Closed,
}

/// An individual line item within a budget, representing a single account/cost center allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLineItem {
    /// Unique line item identifier
    pub line_id: String,
    /// Parent budget identifier
    pub budget_id: String,
    /// GL account code
    pub account_code: String,
    /// GL account name
    pub account_name: String,
    /// Department this line applies to
    pub department: Option<String>,
    /// Cost center this line applies to
    pub cost_center: Option<String>,
    /// Budgeted amount for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub budget_amount: Decimal,
    /// Actual amount recorded for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_amount: Decimal,
    /// Variance (actual - budget)
    #[serde(with = "rust_decimal::serde::str")]
    pub variance: Decimal,
    /// Variance as a percentage of budget
    pub variance_percent: f64,
    /// Start of the budget period for this line
    pub period_start: NaiveDate,
    /// End of the budget period for this line
    pub period_end: NaiveDate,
    /// Free-text notes or explanations for variances
    pub notes: Option<String>,
}

/// A budget representing planned financial targets for a fiscal year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Unique budget identifier
    pub budget_id: String,
    /// Company code this budget belongs to
    pub company_code: String,
    /// Fiscal year the budget covers
    pub fiscal_year: u32,
    /// Human-readable name of the budget (e.g., "FY2025 Operating Budget")
    pub name: String,
    /// Current status of the budget
    pub status: BudgetStatus,
    /// Total budgeted amount across all line items
    #[serde(with = "rust_decimal::serde::str")]
    pub total_budget: Decimal,
    /// Total actual amount across all line items
    #[serde(with = "rust_decimal::serde::str")]
    pub total_actual: Decimal,
    /// Total variance across all line items
    #[serde(with = "rust_decimal::serde::str")]
    pub total_variance: Decimal,
    /// Individual budget line items
    pub line_items: Vec<BudgetLineItem>,
    /// Person who approved the budget
    pub approved_by: Option<String>,
    /// Date the budget was approved
    pub approved_date: Option<NaiveDate>,
}
