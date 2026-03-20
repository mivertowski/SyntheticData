//! Framework Differences Tracking for Dual Reporting.
//!
//! Provides structures for tracking and reporting differences between
//! US GAAP and IFRS accounting treatments for the same transactions.
//! Used when generating synthetic data for dual-reporting scenarios.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Framework difference record for dual reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkDifferenceRecord {
    /// Unique difference identifier.
    pub difference_id: Uuid,

    /// Company code.
    pub company_code: String,

    /// Reporting period end date.
    pub period_date: NaiveDate,

    /// Area of accounting difference.
    pub difference_area: DifferenceArea,

    /// Reference to source transaction/item.
    pub source_reference: String,

    /// Description of the item.
    pub description: String,

    /// US GAAP amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub us_gaap_amount: Decimal,

    /// IFRS amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub ifrs_amount: Decimal,

    /// Difference (IFRS - US GAAP).
    #[serde(with = "rust_decimal::serde::str")]
    pub difference_amount: Decimal,

    /// US GAAP account classification.
    pub us_gaap_classification: String,

    /// IFRS account classification.
    pub ifrs_classification: String,

    /// Explanation of the difference.
    pub explanation: String,

    /// Whether this is a permanent or temporary difference.
    pub difference_type: DifferenceType,

    /// Impact on financial statements.
    pub financial_statement_impact: FinancialStatementImpact,
}

impl FrameworkDifferenceRecord {
    /// Create a new framework difference record.
    pub fn new(
        company_code: impl Into<String>,
        period_date: NaiveDate,
        difference_area: DifferenceArea,
        source_reference: impl Into<String>,
        description: impl Into<String>,
        us_gaap_amount: Decimal,
        ifrs_amount: Decimal,
    ) -> Self {
        let difference_amount = ifrs_amount - us_gaap_amount;
        Self {
            difference_id: Uuid::now_v7(),
            company_code: company_code.into(),
            period_date,
            difference_area,
            source_reference: source_reference.into(),
            description: description.into(),
            us_gaap_amount,
            ifrs_amount,
            difference_amount,
            us_gaap_classification: String::new(),
            ifrs_classification: String::new(),
            explanation: String::new(),
            difference_type: DifferenceType::Temporary,
            financial_statement_impact: FinancialStatementImpact::default(),
        }
    }
}

/// Area of accounting where framework differences occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceArea {
    /// Revenue recognition differences.
    RevenueRecognition,
    /// Lease classification and measurement.
    LeaseAccounting,
    /// Inventory costing method (LIFO).
    InventoryCosting,
    /// Development cost capitalization.
    DevelopmentCosts,
    /// PPE revaluation.
    PropertyRevaluation,
    /// Impairment and reversal.
    Impairment,
    /// Contingent liabilities threshold.
    ContingentLiabilities,
    /// Share-based payment measurement.
    ShareBasedPayment,
    /// Financial instrument classification.
    FinancialInstruments,
    /// Consolidation scope.
    Consolidation,
    /// Joint arrangement classification.
    JointArrangements,
    /// Income taxes.
    IncomeTaxes,
    /// Presentation and disclosure.
    PresentationDisclosure,
    /// Other differences.
    Other,
}

impl std::fmt::Display for DifferenceArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevenueRecognition => write!(f, "Revenue Recognition"),
            Self::LeaseAccounting => write!(f, "Lease Accounting"),
            Self::InventoryCosting => write!(f, "Inventory Costing"),
            Self::DevelopmentCosts => write!(f, "Development Costs"),
            Self::PropertyRevaluation => write!(f, "Property Revaluation"),
            Self::Impairment => write!(f, "Impairment"),
            Self::ContingentLiabilities => write!(f, "Contingent Liabilities"),
            Self::ShareBasedPayment => write!(f, "Share-Based Payment"),
            Self::FinancialInstruments => write!(f, "Financial Instruments"),
            Self::Consolidation => write!(f, "Consolidation"),
            Self::JointArrangements => write!(f, "Joint Arrangements"),
            Self::IncomeTaxes => write!(f, "Income Taxes"),
            Self::PresentationDisclosure => write!(f, "Presentation & Disclosure"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Type of accounting difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceType {
    /// Temporary difference that will reverse over time.
    #[default]
    Temporary,
    /// Permanent difference that will not reverse.
    Permanent,
    /// Classification difference (same total, different line items).
    Classification,
    /// Measurement difference (different amounts).
    Measurement,
    /// Timing difference (same amount, different period).
    Timing,
}

/// Impact on specific financial statements.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialStatementImpact {
    /// Impact on balance sheet assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub assets_impact: Decimal,
    /// Impact on balance sheet liabilities.
    #[serde(with = "rust_decimal::serde::str")]
    pub liabilities_impact: Decimal,
    /// Impact on equity.
    #[serde(with = "rust_decimal::serde::str")]
    pub equity_impact: Decimal,
    /// Impact on revenue.
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_impact: Decimal,
    /// Impact on expenses.
    #[serde(with = "rust_decimal::serde::str")]
    pub expense_impact: Decimal,
    /// Impact on net income.
    #[serde(with = "rust_decimal::serde::str")]
    pub net_income_impact: Decimal,
}

/// Reconciliation summary between frameworks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkReconciliation {
    /// Company code.
    pub company_code: String,

    /// Period end date.
    pub period_date: NaiveDate,

    /// US GAAP net income.
    #[serde(with = "rust_decimal::serde::str")]
    pub us_gaap_net_income: Decimal,

    /// IFRS net income.
    #[serde(with = "rust_decimal::serde::str")]
    pub ifrs_net_income: Decimal,

    /// US GAAP total equity.
    #[serde(with = "rust_decimal::serde::str")]
    pub us_gaap_equity: Decimal,

    /// IFRS total equity.
    #[serde(with = "rust_decimal::serde::str")]
    pub ifrs_equity: Decimal,

    /// US GAAP total assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub us_gaap_assets: Decimal,

    /// IFRS total assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub ifrs_assets: Decimal,

    /// Reconciling items.
    pub reconciling_items: Vec<ReconcilingItem>,
}

impl FrameworkReconciliation {
    /// Create a new reconciliation.
    pub fn new(company_code: impl Into<String>, period_date: NaiveDate) -> Self {
        Self {
            company_code: company_code.into(),
            period_date,
            us_gaap_net_income: Decimal::ZERO,
            ifrs_net_income: Decimal::ZERO,
            us_gaap_equity: Decimal::ZERO,
            ifrs_equity: Decimal::ZERO,
            us_gaap_assets: Decimal::ZERO,
            ifrs_assets: Decimal::ZERO,
            reconciling_items: Vec::new(),
        }
    }

    /// Calculate totals from reconciling items.
    pub fn calculate_totals(&mut self) {
        let mut income_adjustment = Decimal::ZERO;
        let mut equity_adjustment = Decimal::ZERO;
        let mut asset_adjustment = Decimal::ZERO;

        for item in &self.reconciling_items {
            income_adjustment += item.net_income_impact;
            equity_adjustment += item.equity_impact;
            asset_adjustment += item.asset_impact;
        }

        self.ifrs_net_income = self.us_gaap_net_income + income_adjustment;
        self.ifrs_equity = self.us_gaap_equity + equity_adjustment;
        self.ifrs_assets = self.us_gaap_assets + asset_adjustment;
    }
}

/// Individual reconciling item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcilingItem {
    /// Description of the adjustment.
    pub description: String,

    /// Difference area.
    pub difference_area: DifferenceArea,

    /// Impact on net income.
    #[serde(with = "rust_decimal::serde::str")]
    pub net_income_impact: Decimal,

    /// Impact on equity.
    #[serde(with = "rust_decimal::serde::str")]
    pub equity_impact: Decimal,

    /// Impact on assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub asset_impact: Decimal,

    /// Impact on liabilities.
    #[serde(with = "rust_decimal::serde::str")]
    pub liability_impact: Decimal,

    /// Detailed explanation.
    pub explanation: String,
}

impl ReconcilingItem {
    /// Create a new reconciling item with a simplified equity-impact assumption.
    ///
    /// # Limitation — `equity_impact` approximation
    ///
    /// This constructor sets `equity_impact = net_income_impact`, assuming that every
    /// income-statement difference flows through to retained earnings.  This is correct
    /// for the majority of US GAAP ↔ IFRS differences (e.g. revenue recognition timing,
    /// inventory cost-flow assumptions, lease capitalisation under IFRS 16 vs. ASC 842).
    ///
    /// However, several GAAP/IFRS differences bypass the income statement and are
    /// recognised directly in **Other Comprehensive Income (OCI)**, causing
    /// `equity_impact ≠ net_income_impact`:
    ///
    /// | Difference area | US GAAP treatment | IFRS treatment |
    /// |---|---|---|
    /// | Pension remeasurements | Amortised through P&L (corridor method) | OCI only (IAS 19R) |
    /// | Unrealised FX on monetary items | P&L | P&L or OCI depending on hedge designation |
    /// | Available-for-sale / FVOCI equity securities | OCI | OCI (IFRS 9) |
    /// | Revaluation surplus (PP&E / intangibles) | Not permitted | OCI (IAS 16/38) |
    ///
    /// Callers that generate OCI-related reconciling items **must** override
    /// `equity_impact` after construction and set `net_income_impact` to `Decimal::ZERO`
    /// (or to only the reclassification portion).
    pub fn new(
        description: impl Into<String>,
        difference_area: DifferenceArea,
        net_income_impact: Decimal,
    ) -> Self {
        Self {
            description: description.into(),
            difference_area,
            net_income_impact,
            // Simplified: assumes all income differences flow to retained earnings.
            // Override this field for OCI-related differences — see doc comment above.
            equity_impact: net_income_impact,
            asset_impact: Decimal::ZERO,
            liability_impact: Decimal::ZERO,
            explanation: String::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_framework_difference_record() {
        let record = FrameworkDifferenceRecord::new(
            "1000",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            DifferenceArea::DevelopmentCosts,
            "RD001",
            "Software development costs",
            dec!(0),      // US GAAP: expensed
            dec!(100000), // IFRS: capitalized
        );

        assert_eq!(record.difference_amount, dec!(100000));
        assert_eq!(record.difference_area, DifferenceArea::DevelopmentCosts);
    }

    #[test]
    fn test_framework_reconciliation() {
        let mut recon =
            FrameworkReconciliation::new("1000", NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        recon.us_gaap_net_income = dec!(1000000);
        recon.us_gaap_equity = dec!(5000000);
        recon.us_gaap_assets = dec!(10000000);

        // Add reconciling items
        recon.reconciling_items.push(ReconcilingItem::new(
            "Development cost capitalization",
            DifferenceArea::DevelopmentCosts,
            dec!(100000), // Higher income under IFRS
        ));

        recon.reconciling_items.push(ReconcilingItem::new(
            "Impairment reversal",
            DifferenceArea::Impairment,
            dec!(50000), // Reversal permitted under IFRS
        ));

        recon.calculate_totals();

        assert_eq!(recon.ifrs_net_income, dec!(1150000));
        assert_eq!(recon.ifrs_equity, dec!(5150000));
    }
}
