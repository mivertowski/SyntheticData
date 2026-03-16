//! Financial statement models for period-end reporting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// A single line in a consolidation schedule, showing per-entity amounts plus
/// pre-elimination total, elimination adjustments, and post-elimination total.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationLineItem {
    /// Account category (e.g. "Revenue", "Cash", "Payables")
    pub account_category: String,
    /// Per-entity amounts: entity_code → net balance
    #[serde(default)]
    pub entity_amounts: HashMap<String, Decimal>,
    /// Sum of all entity amounts before eliminations
    #[serde(with = "rust_decimal::serde::str")]
    pub pre_elimination_total: Decimal,
    /// Net elimination adjustment (positive = increases total, negative = decreases)
    #[serde(with = "rust_decimal::serde::str")]
    pub elimination_adjustments: Decimal,
    /// post_elimination_total = pre_elimination_total + elimination_adjustments
    #[serde(with = "rust_decimal::serde::str")]
    pub post_elimination_total: Decimal,
}

/// A consolidation schedule showing how individual entity amounts roll up into
/// the consolidated group total with elimination entries applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationSchedule {
    /// Fiscal period label, e.g. "2024-Q1" or "2024-03"
    pub period: String,
    /// One line per account category
    pub line_items: Vec<ConsolidationLineItem>,
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
