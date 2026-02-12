//! Time entry models for the Hire-to-Retire (H2R) process.
//!
//! These models represent employee time tracking entries including regular hours,
//! overtime, PTO, and sick leave with an approval workflow.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
}
