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

// ============================================================================
// IFRS 8 / ASC 280 — Operating Segment Reporting
// ============================================================================

/// Basis for how the entity defines its operating segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    /// Segments defined by geographic region (country / continent)
    Geographic,
    /// Segments defined by product or service line
    ProductLine,
    /// Segments that correspond to separate legal entities
    LegalEntity,
}

/// A single IFRS 8 / ASC 280 reportable operating segment.
///
/// All monetary fields are expressed in the entity's reporting currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSegment {
    /// Unique identifier for this segment record (deterministic UUID)
    pub segment_id: String,
    /// Human-readable segment name (e.g. "North America", "Software Products")
    pub name: String,
    /// Basis on which the segment is identified
    pub segment_type: SegmentType,
    /// Revenue from transactions with external customers
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_external: Decimal,
    /// Revenue from transactions with other operating segments (eliminated on consolidation)
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_intersegment: Decimal,
    /// Segment operating profit (before corporate overhead and group tax)
    #[serde(with = "rust_decimal::serde::str")]
    pub operating_profit: Decimal,
    /// Total assets allocated to this segment
    #[serde(with = "rust_decimal::serde::str")]
    pub total_assets: Decimal,
    /// Total liabilities allocated to this segment
    #[serde(with = "rust_decimal::serde::str")]
    pub total_liabilities: Decimal,
    /// Capital expenditure (additions to PP&E and intangibles) in the period
    #[serde(with = "rust_decimal::serde::str")]
    pub capital_expenditure: Decimal,
    /// Depreciation and amortisation charged in the period
    #[serde(with = "rust_decimal::serde::str")]
    pub depreciation_amortization: Decimal,
    /// Fiscal period label for which these figures are reported (e.g. "2024-03")
    pub period: String,
    /// Company / group these segments belong to
    pub company_code: String,
}

/// Reconciliation of segment totals to the consolidated financial statements
/// as required by IFRS 8 para. 28 and ASC 280-10-50-30.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentReconciliation {
    /// Fiscal period label (e.g. "2024-03")
    pub period: String,
    /// Company / group code
    pub company_code: String,
    /// Sum of all reportable segment revenues (external + intersegment)
    #[serde(with = "rust_decimal::serde::str")]
    pub segment_revenue_total: Decimal,
    /// Elimination of intersegment revenues (typically negative)
    #[serde(with = "rust_decimal::serde::str")]
    pub intersegment_eliminations: Decimal,
    /// Consolidated external revenue = segment_revenue_total + intersegment_eliminations
    #[serde(with = "rust_decimal::serde::str")]
    pub consolidated_revenue: Decimal,
    /// Sum of all reportable segment operating profits
    #[serde(with = "rust_decimal::serde::str")]
    pub segment_profit_total: Decimal,
    /// Unallocated corporate overhead (negative amount)
    #[serde(with = "rust_decimal::serde::str")]
    pub corporate_overhead: Decimal,
    /// Consolidated operating profit = segment_profit_total + corporate_overhead
    #[serde(with = "rust_decimal::serde::str")]
    pub consolidated_profit: Decimal,
    /// Sum of all reportable segment assets
    #[serde(with = "rust_decimal::serde::str")]
    pub segment_assets_total: Decimal,
    /// Unallocated corporate / group assets (e.g. deferred tax, goodwill)
    #[serde(with = "rust_decimal::serde::str")]
    pub unallocated_assets: Decimal,
    /// Consolidated total assets = segment_assets_total + unallocated_assets
    #[serde(with = "rust_decimal::serde::str")]
    pub consolidated_assets: Decimal,
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
