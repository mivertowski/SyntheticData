//! Industry benchmark models for comparative financial analysis.
//!
//! These models represent synthetic industry-average metrics that auditors
//! and analysts use to benchmark an entity's performance against peers.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An industry benchmark metric representing a synthetic peer-group average.
///
/// Auditors use industry benchmarks (ISA 520 analytical procedures) to compare
/// an entity's financial ratios and KPIs against sector norms. These benchmarks
/// are entirely synthetic and labeled as such.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryBenchmark {
    /// Industry this benchmark applies to (e.g., "retail", "manufacturing")
    pub industry: String,
    /// Metric name (e.g., "gross_margin_pct", "current_ratio")
    pub metric: String,
    /// The benchmark value
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// Source attribution — always synthetic
    pub source: String,
    /// Fiscal period label (e.g., "FY2025")
    pub period: String,
}
