//! Financial statement models for period-end reporting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of financial statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatementType {
    /// Balance Sheet (Statement of Financial Position)
    BalanceSheet,
    /// Income Statement (Profit & Loss)
    IncomeStatement,
    /// Cash Flow Statement
    CashFlowStatement,
    /// Statement of Changes in Equity
    ChangesInEquity,
}

/// Basis of accounting for the statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StatementBasis {
    /// US GAAP
    #[default]
    UsGaap,
    /// IFRS
    Ifrs,
    /// Statutory/local GAAP
    Statutory,
}

/// Cash flow category for cash flow statement items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CashFlowCategory {
    /// Operating activities
    Operating,
    /// Investing activities
    Investing,
    /// Financing activities
    Financing,
}

/// A line item on a financial statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementLineItem {
    /// Line item code (e.g., "BS-CASH", "IS-REV")
    pub line_code: String,
    /// Display label
    pub label: String,
    /// Statement section (e.g., "Current Assets", "Revenue")
    pub section: String,
    /// Sort order within section
    pub sort_order: u32,
    /// Current period amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Prior period amount (for comparison)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_prior: Option<Decimal>,
    /// Indentation level for display hierarchy
    pub indent_level: u8,
    /// Whether this is a subtotal/total line
    pub is_total: bool,
    /// GL accounts that roll up to this line
    pub gl_accounts: Vec<String>,
}

/// A cash flow item (indirect method).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowItem {
    /// Item code
    pub item_code: String,
    /// Display label
    pub label: String,
    /// Cash flow category
    pub category: CashFlowCategory,
    /// Amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Prior period amount
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_prior: Option<Decimal>,
    /// Sort order
    pub sort_order: u32,
    /// Is this a subtotal line
    pub is_total: bool,
}

/// A complete financial statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatement {
    /// Unique statement identifier
    pub statement_id: String,
    /// Company code
    pub company_code: String,
    /// Statement type
    pub statement_type: StatementType,
    /// Accounting basis
    pub basis: StatementBasis,
    /// Reporting period start
    pub period_start: NaiveDate,
    /// Reporting period end
    pub period_end: NaiveDate,
    /// Fiscal year
    pub fiscal_year: u16,
    /// Fiscal period
    pub fiscal_period: u8,
    /// Line items
    pub line_items: Vec<FinancialStatementLineItem>,
    /// Cash flow items (only for CashFlowStatement)
    pub cash_flow_items: Vec<CashFlowItem>,
    /// Currency
    pub currency: String,
    /// Whether this is a consolidated statement
    pub is_consolidated: bool,
    /// Preparer ID
    pub preparer_id: String,
}
