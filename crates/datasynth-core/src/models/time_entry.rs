//! Time entry models for the Hire-to-Retire (H2R) process.
//!
//! These models represent employee time tracking entries including regular hours,
//! overtime, PTO, and sick leave with an approval workflow.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Approval status of a time entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeApprovalStatus {
    /// Awaiting manager approval
    #[default]
    Pending,
    /// Approved by manager
    Approved,
    /// Rejected by manager
    Rejected,
}

/// A time entry recording hours worked or leave taken by an employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    /// Unique time entry identifier
    pub entry_id: String,
    /// Employee who recorded the time
    pub employee_id: String,
    /// Date the time entry applies to
    pub date: NaiveDate,
    /// Regular hours worked
    pub hours_regular: f64,
    /// Overtime hours worked
    pub hours_overtime: f64,
    /// Paid time off hours used
    pub hours_pto: f64,
    /// Sick leave hours used
    pub hours_sick: f64,
    /// Project the time was charged to
    pub project_id: Option<String>,
    /// Cost center allocation
    pub cost_center: Option<String>,
    /// Description of work performed
    pub description: Option<String>,
    /// Current approval status
    pub approval_status: TimeApprovalStatus,
    /// Manager who approved/rejected the entry
    pub approved_by: Option<String>,
    /// Date the entry was submitted for approval
    pub submitted_at: Option<NaiveDate>,
    /// Employee display name (denormalized, DS-011)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub employee_name: Option<String>,
    /// Whether this time is billable (DS-012)
    #[serde(default)]
    pub billable: bool,
}

impl ToNodeProperties for TimeEntry {
    fn node_type_name(&self) -> &'static str {
        "time_entry"
    }
    fn node_type_code(&self) -> u16 {
        331
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entryId".into(),
            GraphPropertyValue::String(self.entry_id.clone()),
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
        p.insert("date".into(), GraphPropertyValue::Date(self.date));
        p.insert(
            "hours".into(),
            GraphPropertyValue::Float(self.hours_regular + self.hours_overtime),
        );
        p.insert(
            "hoursRegular".into(),
            GraphPropertyValue::Float(self.hours_regular),
        );
        p.insert(
            "hoursOvertime".into(),
            GraphPropertyValue::Float(self.hours_overtime),
        );
        p.insert("billable".into(), GraphPropertyValue::Bool(self.billable));
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.approval_status)),
        );
        if let Some(ref proj) = self.project_id {
            p.insert("projectId".into(), GraphPropertyValue::String(proj.clone()));
        }
        p
    }
}
