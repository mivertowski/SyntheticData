//! Management KPI models for executive dashboards and reporting.
//!
//! These models represent key performance indicators tracked by management,
//! supporting period-over-period comparison and trend analysis.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Category of a management KPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KpiCategory {
    /// Financial metrics (revenue, margin, EBITDA, etc.)
    #[default]
    Financial,
    /// Operational metrics (throughput, cycle time, utilization, etc.)
    Operational,
    /// Customer metrics (NPS, retention, acquisition cost, etc.)
    Customer,
    /// Employee metrics (headcount, turnover, engagement, etc.)
    Employee,
    /// Quality metrics (defect rate, compliance, SLA adherence, etc.)
    Quality,
}

/// Trend direction for a KPI relative to prior periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KpiTrend {
    /// KPI is moving in a favorable direction
    #[default]
    Improving,
    /// KPI is holding steady
    Stable,
    /// KPI is moving in an unfavorable direction
    Declining,
}

/// A management key performance indicator for a given period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementKpi {
    /// Unique KPI identifier
    pub kpi_id: String,
    /// Company code this KPI belongs to
    pub company_code: String,
    /// Human-readable name of the KPI (e.g., "Gross Margin")
    pub name: String,
    /// Category of the KPI
    pub category: KpiCategory,
    /// Start of the measurement period
    pub period_start: NaiveDate,
    /// End of the measurement period
    pub period_end: NaiveDate,
    /// Actual measured value for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// Target value for the period
    #[serde(with = "rust_decimal::serde::str")]
    pub target: Decimal,
    /// Unit of measure (e.g., "%", "days", "USD")
    pub unit: String,
    /// Trend direction relative to prior periods
    pub trend: KpiTrend,
    /// Year-over-year percentage change (e.g., 0.05 = +5%)
    pub year_over_year_change: Option<f64>,
    /// Value from the prior period for comparison
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub prior_period_value: Option<Decimal>,
}
