//! Payroll models for the Hire-to-Retire (H2R) process.
//!
//! These models represent payroll runs and individual employee pay line items,
//! supporting the full payroll cycle from draft calculation through posting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a payroll run through the processing lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PayrollRunStatus {
    /// Initial draft state before calculation
    #[default]
    Draft,
    /// Payroll has been calculated but not yet approved
    Calculated,
    /// Payroll approved for posting
    Approved,
    /// Payroll posted to GL
    Posted,
    /// Payroll run has been reversed
    Reversed,
}

/// A payroll run representing a complete pay cycle for a company.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRun {
    /// Company code
    pub company_code: String,
    /// Unique payroll run identifier
    pub payroll_id: String,
    /// Start of the pay period
    pub pay_period_start: NaiveDate,
    /// End of the pay period
    pub pay_period_end: NaiveDate,
    /// Date the payroll was run/processed
    pub run_date: NaiveDate,
    /// Current status of the payroll run
    pub status: PayrollRunStatus,
    /// Total gross pay across all employees
    #[serde(with = "rust_decimal::serde::str")]
    pub total_gross: Decimal,
    /// Total deductions across all employees
    #[serde(with = "rust_decimal::serde::str")]
    pub total_deductions: Decimal,
    /// Total net pay across all employees
    #[serde(with = "rust_decimal::serde::str")]
    pub total_net: Decimal,
    /// Total employer cost (gross + employer-side taxes/benefits)
    #[serde(with = "rust_decimal::serde::str")]
    pub total_employer_cost: Decimal,
    /// Number of employees included in this run
    pub employee_count: u32,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// User who posted the payroll
    pub posted_by: Option<String>,
    /// User who approved the payroll
    pub approved_by: Option<String>,
}

/// An individual employee's payroll line item within a payroll run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollLineItem {
    /// Reference to the parent payroll run
    pub payroll_id: String,
    /// Employee identifier
    pub employee_id: String,
    /// Unique line item identifier
    pub line_id: String,
    /// Total gross pay for this employee
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_pay: Decimal,
    /// Base salary component
    #[serde(with = "rust_decimal::serde::str")]
    pub base_salary: Decimal,
    /// Overtime pay component
    #[serde(with = "rust_decimal::serde::str")]
    pub overtime_pay: Decimal,
    /// Bonus component
    #[serde(with = "rust_decimal::serde::str")]
    pub bonus: Decimal,
    /// Federal/state tax withholding
    #[serde(with = "rust_decimal::serde::str")]
    pub tax_withholding: Decimal,
    /// Social security / FICA deduction
    #[serde(with = "rust_decimal::serde::str")]
    pub social_security: Decimal,
    /// Health insurance deduction
    #[serde(with = "rust_decimal::serde::str")]
    pub health_insurance: Decimal,
    /// Retirement plan contribution (employee side)
    #[serde(with = "rust_decimal::serde::str")]
    pub retirement_contribution: Decimal,
    /// Other deductions (garnishments, voluntary deductions, etc.)
    #[serde(with = "rust_decimal::serde::str")]
    pub other_deductions: Decimal,
    /// Net pay after all deductions
    #[serde(with = "rust_decimal::serde::str")]
    pub net_pay: Decimal,
    /// Regular hours worked in the period
    pub hours_worked: f64,
    /// Overtime hours worked in the period
    pub overtime_hours: f64,
    /// Date payment is issued
    pub pay_date: NaiveDate,
    /// Cost center allocation
    pub cost_center: Option<String>,
    /// Department allocation
    pub department: Option<String>,

    // -- Country-pack deduction labels ----------------------------------------
    // When a country pack is available these carry the localized deduction names
    // (e.g. "Lohnsteuer" instead of "Federal Income Tax"). When no pack is set
    // the fields are `None` and the implicit US-centric names apply.
    /// Localized label for the tax withholding deduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_withholding_label: Option<String>,
    /// Localized label for the social security / FICA deduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub social_security_label: Option<String>,
    /// Localized label for the health insurance deduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_insurance_label: Option<String>,
    /// Localized label for the retirement / pension contribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retirement_contribution_label: Option<String>,
    /// Localized label(s) for employer contributions (semicolon-separated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub employer_contribution_label: Option<String>,
}
