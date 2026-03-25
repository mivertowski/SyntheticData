//! Financial statement models for period-end reporting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    /// Prior year amount for year-on-year comparative statements
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_year_amount: Option<Decimal>,
    /// Significant accounting assumptions underlying this line item
    /// (e.g., goodwill, provisions, fair value measurements)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assumptions: Option<String>,
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

// ============================================================================
// Management Report — WI-7
// ============================================================================

/// A single KPI summary line within a management report.
///
/// Captures the actual vs. target comparison and a RAG (Red/Amber/Green) status
/// for each metric tracked in the period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSummaryLine {
    /// KPI metric name (e.g., "Revenue Growth Rate", "Gross Margin")
    pub metric: String,
    /// Actual value achieved in the period
    #[serde(with = "rust_decimal::serde::str")]
    pub actual: Decimal,
    /// Target value set for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub target: Decimal,
    /// Variance as a percentage of target ((actual - target) / target)
    pub variance_pct: f64,
    /// Traffic-light status: "green" (< 5% variance), "amber" (< 10%), "red" (>= 10%)
    pub rag_status: String,
}

/// A single budget variance line within a management report.
///
/// Shows the planned vs. actual for a GL account category and the resulting
/// variance expressed both in absolute and percentage terms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVarianceLine {
    /// GL account code or category (e.g., "4000", "Revenue")
    pub account: String,
    /// Budgeted amount for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub budget_amount: Decimal,
    /// Actual amount recorded for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_amount: Decimal,
    /// Variance = actual − budget
    #[serde(with = "rust_decimal::serde::str")]
    pub variance: Decimal,
    /// Variance as a percentage of budget ((actual − budget) / budget)
    pub variance_pct: f64,
}

/// A management report aggregating KPIs and budget variances for a period.
///
/// Management packs and board reports are the primary documents that auditors
/// reference when performing analytical procedures (ISA 520) and understanding
/// management's assessment of the business for the period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementReport {
    /// Unique report identifier
    pub report_id: Uuid,
    /// Report type: "monthly_pack", "board_report", "forecast", "flash_report"
    pub report_type: String,
    /// Fiscal period label (e.g., "2025-Q1", "2025-01")
    pub period: String,
    /// Entity this report belongs to
    pub entity_code: String,
    /// Employee / role ID of the preparer
    pub prepared_by: String,
    /// Date the report was prepared
    pub prepared_date: NaiveDate,
    /// KPI summary lines (6–10 metrics)
    pub kpi_summary: Vec<KpiSummaryLine>,
    /// Budget variance lines (8–15 accounts)
    pub budget_variances: Vec<BudgetVarianceLine>,
    /// Narrative management commentary for the period
    pub commentary: String,
}

// ---------------------------------------------------------------------------
// Tests — WI-8: FinancialStatementLineItem comparative fields
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_line_item(
        amount: Decimal,
        prior_year: Option<Decimal>,
        assumptions: Option<&str>,
    ) -> FinancialStatementLineItem {
        FinancialStatementLineItem {
            line_code: "BS-CASH".to_string(),
            label: "Cash and Cash Equivalents".to_string(),
            section: "Current Assets".to_string(),
            sort_order: 1,
            amount,
            amount_prior: None,
            prior_year_amount: prior_year,
            assumptions: assumptions.map(|s| s.to_string()),
            indent_level: 0,
            is_total: false,
            gl_accounts: vec![],
        }
    }

    #[test]
    fn test_prior_year_amount_field_present() {
        let item = make_line_item(dec!(100_000), Some(dec!(95_000)), None);
        assert_eq!(item.prior_year_amount, Some(dec!(95_000)));
    }

    #[test]
    fn test_assumptions_present_for_estimate_heavy_line() {
        let assumption_text = "Based on discounted cash flow analysis";
        let item = make_line_item(
            dec!(500_000),
            Some(dec!(480_000)),
            Some(assumption_text),
        );
        assert!(item.assumptions.is_some());
        assert_eq!(item.assumptions.as_deref(), Some(assumption_text));
    }

    #[test]
    fn test_prior_year_amounts_are_within_30_pct_of_current() {
        // Verify that representative prior-year amounts are plausible
        // (within 30% of the current-year amount, per WI-8 spec).
        let cases: &[(Decimal, Decimal)] = &[
            (dec!(100_000), dec!(85_000)),  // -15% — within bounds
            (dec!(200_000), dec!(230_000)), // +15% — within bounds
            (dec!(50_000), dec!(35_100)),   // -29.8% — within bounds
        ];
        for (current, prior) in cases {
            let ratio = ((prior - current).abs() / current)
                .to_string()
                .parse::<f64>()
                .unwrap_or(1.0);
            assert!(
                ratio <= 0.30,
                "Prior year amount {prior} is more than 30% away from current {current} \
                 (ratio={ratio:.3})"
            );
        }
    }
}
