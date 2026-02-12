//! Spend analysis models for procurement category management.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Spend analysis for a procurement category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendAnalysis {
    /// Category identifier (e.g., GL account group or material group)
    pub category_id: String,
    /// Category description
    pub category_name: String,
    /// Company code
    pub company_code: String,
    /// Total spend in the analysis period
    #[serde(with = "rust_decimal::serde::str")]
    pub total_spend: Decimal,
    /// Number of active vendors in this category
    pub vendor_count: u32,
    /// Number of transactions in this category
    pub transaction_count: u32,
    /// Herfindahl-Hirschman Index (0-10000) measuring vendor concentration
    pub hhi_index: f64,
    /// Top vendor spend shares
    pub vendor_shares: Vec<VendorSpendShare>,
    /// Percentage of spend under contract
    pub contract_coverage: f64,
    /// Percentage of spend through preferred vendors
    pub preferred_vendor_coverage: f64,
    /// Average unit price trend (year-over-year change)
    pub price_trend_pct: f64,
    /// Analysis period (fiscal year)
    pub fiscal_year: u16,
}

/// Vendor's share of category spend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSpendShare {
    /// Vendor ID
    pub vendor_id: String,
    /// Vendor name
    pub vendor_name: String,
    /// Spend amount
    #[serde(with = "rust_decimal::serde::str")]
    pub spend_amount: Decimal,
    /// Share of category spend (0.0 to 1.0)
    pub share: f64,
    /// Whether this is a preferred/contracted vendor
    pub is_preferred: bool,
}
