//! Payroll models for the Hire-to-Retire (H2R) process.
//!
//! These models represent payroll runs and individual employee pay line items,
//! supporting the full payroll cycle from draft calculation through posting.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

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

impl ToNodeProperties for PayrollRun {
    fn node_type_name(&self) -> &'static str {
        "payroll_run"
    }
    fn node_type_code(&self) -> u16 {
        330
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "payrollId".into(),
            GraphPropertyValue::String(self.payroll_id.clone()),
        );
        p.insert(
            "periodStart".into(),
            GraphPropertyValue::Date(self.pay_period_start),
        );
        p.insert(
            "periodEnd".into(),
            GraphPropertyValue::Date(self.pay_period_end),
        );
        p.insert("runDate".into(), GraphPropertyValue::Date(self.run_date));
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "employeeCount".into(),
            GraphPropertyValue::Int(self.employee_count as i64),
        );
        p.insert(
            "grossPay".into(),
            GraphPropertyValue::Decimal(self.total_gross),
        );
        p.insert("netPay".into(), GraphPropertyValue::Decimal(self.total_net));
        p.insert(
            "taxWithheld".into(),
            GraphPropertyValue::Decimal(self.total_deductions),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "isApproved".into(),
            GraphPropertyValue::Bool(matches!(
                self.status,
                PayrollRunStatus::Approved | PayrollRunStatus::Posted
            )),
        );
        p
    }
}

// ---------------------------------------------------------------------------
// Benefit enrollment models
// ---------------------------------------------------------------------------

/// Type of benefit plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BenefitPlanType {
    #[default]
    Health,
    Dental,
    Vision,
    Retirement401k,
    StockPurchase,
    LifeInsurance,
    Disability,
}

/// Status of a benefit enrollment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BenefitStatus {
    #[default]
    Active,
    Pending,
    Terminated,
    OnLeave,
}

/// An employee's enrollment in a benefit plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenefitEnrollment {
    /// Unique enrollment identifier
    pub id: String,
    /// Company / entity code
    pub entity_code: String,
    /// Employee ID
    pub employee_id: String,
    /// Employee display name (denormalized)
    pub employee_name: String,
    /// Benefit plan type
    pub plan_type: BenefitPlanType,
    /// Plan name (e.g. "Blue Cross PPO")
    pub plan_name: String,
    /// Date enrollment was submitted
    pub enrollment_date: NaiveDate,
    /// Coverage effective date
    pub effective_date: NaiveDate,
    /// Fiscal period (e.g. "2024-06")
    pub period: String,
    /// Employee contribution amount per period
    #[serde(with = "rust_decimal::serde::str")]
    pub employee_contribution: Decimal,
    /// Employer contribution amount per period
    #[serde(with = "rust_decimal::serde::str")]
    pub employer_contribution: Decimal,
    /// Currency code
    pub currency: String,
    /// Current enrollment status
    pub status: BenefitStatus,
    /// Whether enrollment is currently active
    pub is_active: bool,
}

impl BenefitEnrollment {
    /// Create a new benefit enrollment.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_code: impl Into<String>,
        employee_id: impl Into<String>,
        employee_name: impl Into<String>,
        plan_type: BenefitPlanType,
        plan_name: impl Into<String>,
        enrollment_date: NaiveDate,
        effective_date: NaiveDate,
        period: impl Into<String>,
        employee_contribution: Decimal,
        employer_contribution: Decimal,
        currency: impl Into<String>,
        status: BenefitStatus,
        is_active: bool,
    ) -> Self {
        Self {
            id: id.into(),
            entity_code: entity_code.into(),
            employee_id: employee_id.into(),
            employee_name: employee_name.into(),
            plan_type,
            plan_name: plan_name.into(),
            enrollment_date,
            effective_date,
            period: period.into(),
            employee_contribution,
            employer_contribution,
            currency: currency.into(),
            status,
            is_active,
        }
    }
}

impl ToNodeProperties for BenefitEnrollment {
    fn node_type_name(&self) -> &'static str {
        "benefit_enrollment"
    }
    fn node_type_code(&self) -> u16 {
        333
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.entity_code.clone()),
        );
        p.insert(
            "employeeId".into(),
            GraphPropertyValue::String(self.employee_id.clone()),
        );
        p.insert(
            "employeeName".into(),
            GraphPropertyValue::String(self.employee_name.clone()),
        );
        p.insert(
            "planType".into(),
            GraphPropertyValue::String(format!("{:?}", self.plan_type)),
        );
        p.insert(
            "planName".into(),
            GraphPropertyValue::String(self.plan_name.clone()),
        );
        p.insert(
            "enrollmentDate".into(),
            GraphPropertyValue::Date(self.enrollment_date),
        );
        p.insert(
            "effectiveDate".into(),
            GraphPropertyValue::Date(self.effective_date),
        );
        p.insert(
            "period".into(),
            GraphPropertyValue::String(self.period.clone()),
        );
        p.insert(
            "employeeContribution".into(),
            GraphPropertyValue::Decimal(self.employee_contribution),
        );
        p.insert(
            "employerContribution".into(),
            GraphPropertyValue::Decimal(self.employer_contribution),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert("isActive".into(), GraphPropertyValue::Bool(self.is_active));
        p
    }
}
