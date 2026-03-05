//! Expense report models for the Hire-to-Retire (H2R) process.
//!
//! These models represent employee expense reports and their line items,
//! supporting the full expense lifecycle from draft submission through payment.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Status of an expense report through the approval and payment lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseStatus {
    /// Initial draft, not yet submitted
    #[default]
    Draft,
    /// Submitted for approval
    Submitted,
    /// Approved by manager
    Approved,
    /// Rejected by manager
    Rejected,
    /// Reimbursement paid to employee
    Paid,
}

/// Category of an expense line item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategory {
    /// Airfare, mileage, etc.
    Travel,
    /// Business meals and dining
    Meals,
    /// Hotel and accommodation
    Lodging,
    /// Taxi, rideshare, rental car, parking
    Transportation,
    /// Office supplies and equipment
    Office,
    /// Client entertainment
    Entertainment,
    /// Professional development and training
    Training,
    /// Miscellaneous expenses
    Other,
}

/// An expense report submitted by an employee for reimbursement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseReport {
    /// Unique expense report identifier
    pub report_id: String,
    /// Employee who submitted the report
    pub employee_id: String,
    /// Date the report was submitted
    pub submission_date: NaiveDate,
    /// Overall description/purpose of the expense report
    pub description: String,
    /// Current status of the expense report
    pub status: ExpenseStatus,
    /// Total amount across all line items
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// Individual expense line items
    pub line_items: SmallVec<[ExpenseLineItem; 4]>,
    /// Manager who approved/rejected the report
    pub approved_by: Option<String>,
    /// Date the report was approved
    pub approved_date: Option<NaiveDate>,
    /// Date the reimbursement was paid
    pub paid_date: Option<NaiveDate>,
    /// Cost center to charge
    pub cost_center: Option<String>,
    /// Department to charge
    pub department: Option<String>,
    /// List of policy violations flagged on this report
    pub policy_violations: Vec<String>,
    /// Employee display name (denormalized, DS-011)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub employee_name: Option<String>,
}

impl ToNodeProperties for ExpenseReport {
    fn node_type_name(&self) -> &'static str {
        "expense_report"
    }
    fn node_type_code(&self) -> u16 {
        332
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "reportId".into(),
            GraphPropertyValue::String(self.report_id.clone()),
        );
        p.insert(
            "employeeId".into(),
            GraphPropertyValue::String(self.employee_id.clone()),
        );
        if let Some(ref name) = self.employee_name {
            p.insert(
                "employeeName".into(),
                GraphPropertyValue::String(name.clone()),
            );
        }
        p.insert(
            "submissionDate".into(),
            GraphPropertyValue::Date(self.submission_date),
        );
        p.insert(
            "totalAmount".into(),
            GraphPropertyValue::Decimal(self.total_amount),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "lineCount".into(),
            GraphPropertyValue::Int(self.line_items.len() as i64),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "isApproved".into(),
            GraphPropertyValue::Bool(matches!(
                self.status,
                ExpenseStatus::Approved | ExpenseStatus::Paid
            )),
        );
        if let Some(ref dept) = self.department {
            p.insert(
                "department".into(),
                GraphPropertyValue::String(dept.clone()),
            );
        }
        p
    }
}

/// An individual line item within an expense report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseLineItem {
    /// Unique line item identifier
    pub item_id: String,
    /// Expense category
    pub category: ExpenseCategory,
    /// Date the expense was incurred
    pub date: NaiveDate,
    /// Amount of the expense
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// Description of the expense
    pub description: String,
    /// Whether a receipt is attached
    pub receipt_attached: bool,
    /// Merchant or vendor name
    pub merchant: Option<String>,
}
