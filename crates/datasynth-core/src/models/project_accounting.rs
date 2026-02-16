//! Project Accounting Models.
//!
//! Extends the base [`Project`]/[`WbsElement`] models with accounting-specific types:
//! - Cost tracking by category and source document
//! - Revenue recognition (Percentage-of-Completion / ASC 606)
//! - Project milestones
//! - Change orders and retainage
//! - Earned Value Management (EVM) metrics (BCWS/BCWP/ACWP/SPI/CPI/EAC/ETC/TCPI)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Cost Category and Source
// ---------------------------------------------------------------------------

/// Category of project cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostCategory {
    /// Labor costs (employee time)
    #[default]
    Labor,
    /// Material / parts costs
    Material,
    /// External subcontractor costs
    Subcontractor,
    /// Overhead allocation
    Overhead,
    /// Equipment / machinery usage
    Equipment,
    /// Travel & expense costs
    Travel,
}

/// Type of source document that originated the cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostSourceType {
    /// From a time entry (employee hours)
    #[default]
    TimeEntry,
    /// From an expense report
    ExpenseReport,
    /// From a purchase order
    PurchaseOrder,
    /// From a vendor invoice
    VendorInvoice,
    /// From a manual journal entry
    JournalEntry,
}

// ---------------------------------------------------------------------------
// Project Cost Line
// ---------------------------------------------------------------------------

/// A single cost posting against a project WBS element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCostLine {
    /// Unique cost line ID
    pub id: String,
    /// Project ID this cost belongs to
    pub project_id: String,
    /// WBS element ID for cost assignment
    pub wbs_id: String,
    /// Entity that incurred the cost
    pub entity_id: String,
    /// Date the cost was incurred
    pub posting_date: NaiveDate,
    /// Category of the cost
    pub cost_category: CostCategory,
    /// Type of source document
    pub source_type: CostSourceType,
    /// Reference to the source document
    pub source_document_id: String,
    /// Cost amount (always positive)
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Currency
    pub currency: String,
    /// Hours (for labor costs)
    pub hours: Option<Decimal>,
    /// Description
    pub description: String,
}

impl ProjectCostLine {
    /// Creates a new project cost line.
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        wbs_id: impl Into<String>,
        entity_id: impl Into<String>,
        posting_date: NaiveDate,
        cost_category: CostCategory,
        source_type: CostSourceType,
        source_document_id: impl Into<String>,
        amount: Decimal,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            wbs_id: wbs_id.into(),
            entity_id: entity_id.into(),
            posting_date,
            cost_category,
            source_type,
            source_document_id: source_document_id.into(),
            amount,
            currency: currency.into(),
            hours: None,
            description: String::new(),
        }
    }

    /// Sets the hours (for labor costs).
    pub fn with_hours(mut self, hours: Decimal) -> Self {
        self.hours = Some(hours);
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Computes an effective hourly rate (if hours are available).
    pub fn hourly_rate(&self) -> Option<Decimal> {
        self.hours
            .filter(|h| !h.is_zero())
            .map(|h| (self.amount / h).round_dp(2))
    }
}

// ---------------------------------------------------------------------------
// Revenue Recognition
// ---------------------------------------------------------------------------

/// Method used for revenue recognition on long-term projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RevenueMethod {
    /// Percentage of Completion (ASC 606 input method)
    #[default]
    PercentageOfCompletion,
    /// Completed Contract
    CompletedContract,
    /// Milestone-based recognition (ASC 606 output method)
    MilestoneBased,
}

/// How completion percentage is measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompletionMeasure {
    /// Cost-to-cost (incurred / estimated total)
    #[default]
    CostToCost,
    /// Labor hours (hours worked / estimated total hours)
    LaborHours,
    /// Physical completion (engineering estimate)
    PhysicalCompletion,
}

/// Revenue recognition for a project period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRevenue {
    /// Unique revenue record ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// Entity ID (billing entity)
    pub entity_id: String,
    /// Period start
    pub period_start: NaiveDate,
    /// Period end
    pub period_end: NaiveDate,
    /// Total contract value
    #[serde(with = "rust_decimal::serde::str")]
    pub contract_value: Decimal,
    /// Total estimated cost at completion
    #[serde(with = "rust_decimal::serde::str")]
    pub estimated_total_cost: Decimal,
    /// Costs incurred to date
    #[serde(with = "rust_decimal::serde::str")]
    pub costs_to_date: Decimal,
    /// Completion percentage (0.00 to 1.00)
    #[serde(with = "rust_decimal::serde::str")]
    pub completion_pct: Decimal,
    /// Revenue method
    pub method: RevenueMethod,
    /// Completion measure
    pub measure: CompletionMeasure,
    /// Cumulative revenue recognized to date
    #[serde(with = "rust_decimal::serde::str")]
    pub cumulative_revenue: Decimal,
    /// Revenue recognized in this period
    #[serde(with = "rust_decimal::serde::str")]
    pub period_revenue: Decimal,
    /// Cumulative amount billed to customer
    #[serde(with = "rust_decimal::serde::str")]
    pub billed_to_date: Decimal,
    /// Unbilled revenue (cumulative_revenue - billed_to_date)
    #[serde(with = "rust_decimal::serde::str")]
    pub unbilled_revenue: Decimal,
    /// Estimated gross margin percentage
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_margin_pct: Decimal,
}

impl ProjectRevenue {
    /// Computes the PoC completion percentage (cost-to-cost method).
    pub fn computed_completion_pct(&self) -> Decimal {
        if self.estimated_total_cost.is_zero() {
            return Decimal::ZERO;
        }
        (self.costs_to_date / self.estimated_total_cost).round_dp(4)
    }

    /// Computes the cumulative revenue based on PoC.
    pub fn computed_cumulative_revenue(&self) -> Decimal {
        (self.contract_value * self.completion_pct).round_dp(2)
    }

    /// Computes the unbilled revenue.
    pub fn computed_unbilled_revenue(&self) -> Decimal {
        (self.cumulative_revenue - self.billed_to_date).round_dp(2)
    }

    /// Computes estimated gross margin percentage.
    pub fn computed_gross_margin_pct(&self) -> Decimal {
        if self.contract_value.is_zero() {
            return Decimal::ZERO;
        }
        ((self.contract_value - self.estimated_total_cost) / self.contract_value).round_dp(4)
    }
}

// ---------------------------------------------------------------------------
// Milestones
// ---------------------------------------------------------------------------

/// Status of a project milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    /// Not yet started
    #[default]
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Overdue (past planned date, not completed)
    Overdue,
    /// Cancelled
    Cancelled,
}

/// A project milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMilestone {
    /// Unique milestone ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// WBS element ID (optional)
    pub wbs_id: Option<String>,
    /// Milestone name
    pub name: String,
    /// Planned completion date
    pub planned_date: NaiveDate,
    /// Actual completion date (if completed)
    pub actual_date: Option<NaiveDate>,
    /// Current status
    pub status: MilestoneStatus,
    /// Payment amount tied to milestone (if any)
    #[serde(with = "rust_decimal::serde::str")]
    pub payment_amount: Decimal,
    /// Completion weight for EVM (0.0 to 1.0)
    #[serde(with = "rust_decimal::serde::str")]
    pub weight: Decimal,
    /// Sequence order
    pub sequence: u32,
}

impl ProjectMilestone {
    /// Creates a new milestone.
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        name: impl Into<String>,
        planned_date: NaiveDate,
        sequence: u32,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            wbs_id: None,
            name: name.into(),
            planned_date,
            actual_date: None,
            status: MilestoneStatus::Pending,
            payment_amount: Decimal::ZERO,
            weight: Decimal::ZERO,
            sequence,
        }
    }

    /// Sets the WBS element.
    pub fn with_wbs(mut self, wbs_id: impl Into<String>) -> Self {
        self.wbs_id = Some(wbs_id.into());
        self
    }

    /// Sets the payment amount tied to this milestone.
    pub fn with_payment(mut self, amount: Decimal) -> Self {
        self.payment_amount = amount;
        self
    }

    /// Sets the EVM weight.
    pub fn with_weight(mut self, weight: Decimal) -> Self {
        self.weight = weight;
        self
    }

    /// Returns true if the milestone is overdue on the given date.
    pub fn is_overdue_on(&self, date: NaiveDate) -> bool {
        self.actual_date.is_none() && date > self.planned_date
    }

    /// Returns the number of days late (negative if early or not yet complete).
    pub fn days_variance(&self) -> Option<i64> {
        self.actual_date
            .map(|actual| (actual - self.planned_date).num_days())
    }
}

// ---------------------------------------------------------------------------
// Change Orders
// ---------------------------------------------------------------------------

/// Status of a change order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOrderStatus {
    /// Change order submitted
    #[default]
    Submitted,
    /// Under review
    UnderReview,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Withdrawn
    Withdrawn,
}

/// Reason for the change order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChangeReason {
    /// Client-requested scope change
    #[default]
    ScopeChange,
    /// Unforeseen site conditions
    UnforeseenConditions,
    /// Design error or omission
    DesignError,
    /// Regulatory requirement change
    RegulatoryChange,
    /// Value engineering (cost reduction)
    ValueEngineering,
    /// Schedule acceleration
    ScheduleAcceleration,
}

/// A change order that modifies a project's scope, cost, or schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeOrder {
    /// Unique change order ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// Change order number (sequential within project)
    pub number: u32,
    /// Date submitted
    pub submitted_date: NaiveDate,
    /// Date approved (if approved)
    pub approved_date: Option<NaiveDate>,
    /// Status
    pub status: ChangeOrderStatus,
    /// Reason for the change
    pub reason: ChangeReason,
    /// Description of the change
    pub description: String,
    /// Impact on contract value (positive = increase, negative = decrease)
    #[serde(with = "rust_decimal::serde::str")]
    pub cost_impact: Decimal,
    /// Impact on estimated total cost
    #[serde(with = "rust_decimal::serde::str")]
    pub estimated_cost_impact: Decimal,
    /// Schedule impact in calendar days (positive = delay)
    pub schedule_impact_days: i32,
}

impl ChangeOrder {
    /// Creates a new change order.
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        number: u32,
        submitted_date: NaiveDate,
        reason: ChangeReason,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            number,
            submitted_date,
            approved_date: None,
            status: ChangeOrderStatus::Submitted,
            reason,
            description: description.into(),
            cost_impact: Decimal::ZERO,
            estimated_cost_impact: Decimal::ZERO,
            schedule_impact_days: 0,
        }
    }

    /// Sets the cost impact.
    pub fn with_cost_impact(mut self, contract_impact: Decimal, estimated_impact: Decimal) -> Self {
        self.cost_impact = contract_impact;
        self.estimated_cost_impact = estimated_impact;
        self
    }

    /// Sets the schedule impact.
    pub fn with_schedule_impact(mut self, days: i32) -> Self {
        self.schedule_impact_days = days;
        self
    }

    /// Approves the change order.
    pub fn approve(mut self, date: NaiveDate) -> Self {
        self.status = ChangeOrderStatus::Approved;
        self.approved_date = Some(date);
        self
    }

    /// Returns true if the change order is approved.
    pub fn is_approved(&self) -> bool {
        self.status == ChangeOrderStatus::Approved
    }

    /// Returns the net cost impact (only if approved).
    pub fn net_cost_impact(&self) -> Decimal {
        if self.is_approved() {
            self.cost_impact
        } else {
            Decimal::ZERO
        }
    }
}

// ---------------------------------------------------------------------------
// Retainage
// ---------------------------------------------------------------------------

/// Status of a retainage hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RetainageStatus {
    /// Retainage is being held
    #[default]
    Held,
    /// Partial release
    PartiallyReleased,
    /// Fully released
    Released,
    /// Forfeited (e.g., for defective work)
    Forfeited,
}

/// Retainage record — a portion of each payment withheld until project completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retainage {
    /// Unique retainage record ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// Entity ID
    pub entity_id: String,
    /// Vendor/subcontractor ID this retainage relates to
    pub vendor_id: String,
    /// Retainage percentage (e.g., 0.10 for 10%)
    #[serde(with = "rust_decimal::serde::str")]
    pub retainage_pct: Decimal,
    /// Total retainage held
    #[serde(with = "rust_decimal::serde::str")]
    pub total_held: Decimal,
    /// Amount released to date
    #[serde(with = "rust_decimal::serde::str")]
    pub released_amount: Decimal,
    /// Current status
    pub status: RetainageStatus,
    /// Date retainage was first held
    pub inception_date: NaiveDate,
    /// Date of last release (if any)
    pub last_release_date: Option<NaiveDate>,
}

impl Retainage {
    /// Creates a new retainage record.
    pub fn new(
        id: impl Into<String>,
        project_id: impl Into<String>,
        entity_id: impl Into<String>,
        vendor_id: impl Into<String>,
        retainage_pct: Decimal,
        inception_date: NaiveDate,
    ) -> Self {
        Self {
            id: id.into(),
            project_id: project_id.into(),
            entity_id: entity_id.into(),
            vendor_id: vendor_id.into(),
            retainage_pct,
            total_held: Decimal::ZERO,
            released_amount: Decimal::ZERO,
            status: RetainageStatus::Held,
            inception_date,
            last_release_date: None,
        }
    }

    /// Adds retainage from a payment.
    pub fn add_from_payment(&mut self, payment_amount: Decimal) {
        let held = (payment_amount * self.retainage_pct).round_dp(2);
        self.total_held += held;
    }

    /// Returns the balance still held.
    pub fn balance_held(&self) -> Decimal {
        (self.total_held - self.released_amount).round_dp(2)
    }

    /// Releases a specified amount.
    pub fn release(&mut self, amount: Decimal, date: NaiveDate) {
        let release = amount.min(self.balance_held());
        self.released_amount += release;
        self.last_release_date = Some(date);
        if self.balance_held().is_zero() {
            self.status = RetainageStatus::Released;
        } else {
            self.status = RetainageStatus::PartiallyReleased;
        }
    }
}

// ---------------------------------------------------------------------------
// Earned Value Management (EVM)
// ---------------------------------------------------------------------------

/// Earned Value Management metrics for a project at a point in time.
///
/// EVM formulas:
/// - SV (Schedule Variance) = EV - PV
/// - CV (Cost Variance) = EV - AC
/// - SPI (Schedule Performance Index) = EV / PV
/// - CPI (Cost Performance Index) = EV / AC
/// - EAC (Estimate at Completion) = BAC / CPI
/// - ETC (Estimate to Complete) = EAC - AC
/// - TCPI (To-Complete Performance Index) = (BAC - EV) / (BAC - AC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedValueMetric {
    /// Unique metric ID
    pub id: String,
    /// Project ID
    pub project_id: String,
    /// Measurement date
    pub measurement_date: NaiveDate,
    /// Budget at Completion (total baseline budget)
    #[serde(with = "rust_decimal::serde::str")]
    pub bac: Decimal,
    /// Planned Value (BCWS — Budgeted Cost of Work Scheduled)
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_value: Decimal,
    /// Earned Value (BCWP — Budgeted Cost of Work Performed)
    #[serde(with = "rust_decimal::serde::str")]
    pub earned_value: Decimal,
    /// Actual Cost (ACWP — Actual Cost of Work Performed)
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_cost: Decimal,
    /// Schedule Variance (EV - PV)
    #[serde(with = "rust_decimal::serde::str")]
    pub schedule_variance: Decimal,
    /// Cost Variance (EV - AC)
    #[serde(with = "rust_decimal::serde::str")]
    pub cost_variance: Decimal,
    /// Schedule Performance Index (EV / PV)
    #[serde(with = "rust_decimal::serde::str")]
    pub spi: Decimal,
    /// Cost Performance Index (EV / AC)
    #[serde(with = "rust_decimal::serde::str")]
    pub cpi: Decimal,
    /// Estimate at Completion (BAC / CPI)
    #[serde(with = "rust_decimal::serde::str")]
    pub eac: Decimal,
    /// Estimate to Complete (EAC - AC)
    #[serde(with = "rust_decimal::serde::str")]
    pub etc: Decimal,
    /// To-Complete Performance Index ((BAC - EV) / (BAC - AC))
    #[serde(with = "rust_decimal::serde::str")]
    pub tcpi: Decimal,
}

impl EarnedValueMetric {
    /// Creates a new EVM metric by computing all derived values from
    /// the three fundamental inputs: PV, EV, AC, and BAC.
    pub fn compute(
        id: impl Into<String>,
        project_id: impl Into<String>,
        measurement_date: NaiveDate,
        bac: Decimal,
        planned_value: Decimal,
        earned_value: Decimal,
        actual_cost: Decimal,
    ) -> Self {
        let sv = (earned_value - planned_value).round_dp(2);
        let cv = (earned_value - actual_cost).round_dp(2);
        let spi = if planned_value.is_zero() {
            dec!(1.00)
        } else {
            (earned_value / planned_value).round_dp(4)
        };
        let cpi = if actual_cost.is_zero() {
            dec!(1.00)
        } else {
            (earned_value / actual_cost).round_dp(4)
        };
        let eac = if cpi.is_zero() {
            bac
        } else {
            (bac / cpi).round_dp(2)
        };
        let etc = (eac - actual_cost).round_dp(2);
        let remaining_budget = bac - actual_cost;
        let remaining_work = bac - earned_value;
        let tcpi = if remaining_budget.is_zero() {
            dec!(1.00)
        } else {
            (remaining_work / remaining_budget).round_dp(4)
        };

        Self {
            id: id.into(),
            project_id: project_id.into(),
            measurement_date,
            bac,
            planned_value,
            earned_value,
            actual_cost,
            schedule_variance: sv,
            cost_variance: cv,
            spi,
            cpi,
            eac,
            etc,
            tcpi,
        }
    }

    /// Returns true if the project is ahead of schedule (SPI > 1.0).
    pub fn is_ahead_of_schedule(&self) -> bool {
        self.spi > dec!(1.00)
    }

    /// Returns true if the project is under budget (CPI > 1.0).
    pub fn is_under_budget(&self) -> bool {
        self.cpi > dec!(1.00)
    }

    /// Returns true if the project is both on schedule and on budget.
    pub fn is_healthy(&self) -> bool {
        self.spi >= dec!(0.90) && self.cpi >= dec!(0.90)
    }

    /// Returns the variance at completion (BAC - EAC).
    pub fn variance_at_completion(&self) -> Decimal {
        (self.bac - self.eac).round_dp(2)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    // -- Cost Lines --

    #[test]
    fn test_cost_line_creation() {
        let line = ProjectCostLine::new(
            "PCL-001",
            "PRJ-001",
            "PRJ-001.01",
            "C001",
            d("2025-03-15"),
            CostCategory::Labor,
            CostSourceType::TimeEntry,
            "TE-001",
            dec!(1500),
            "USD",
        )
        .with_hours(dec!(20))
        .with_description("Developer time - sprint 5");

        assert_eq!(line.cost_category, CostCategory::Labor);
        assert_eq!(line.source_type, CostSourceType::TimeEntry);
        assert_eq!(line.amount, dec!(1500));
        assert_eq!(line.hours, Some(dec!(20)));
        assert_eq!(line.hourly_rate(), Some(dec!(75.00)));
    }

    #[test]
    fn test_cost_line_hourly_rate_no_hours() {
        let line = ProjectCostLine::new(
            "PCL-002",
            "PRJ-001",
            "PRJ-001.02",
            "C001",
            d("2025-03-15"),
            CostCategory::Material,
            CostSourceType::PurchaseOrder,
            "PO-001",
            dec!(5000),
            "USD",
        );
        assert_eq!(line.hourly_rate(), None);
    }

    // -- Revenue Recognition --

    #[test]
    fn test_revenue_poc_completion() {
        let rev = ProjectRevenue {
            id: "REV-001".to_string(),
            project_id: "PRJ-001".to_string(),
            entity_id: "C001".to_string(),
            period_start: d("2025-01-01"),
            period_end: d("2025-03-31"),
            contract_value: dec!(1000000),
            estimated_total_cost: dec!(800000),
            costs_to_date: dec!(400000),
            completion_pct: dec!(0.50),
            method: RevenueMethod::PercentageOfCompletion,
            measure: CompletionMeasure::CostToCost,
            cumulative_revenue: dec!(500000),
            period_revenue: dec!(200000),
            billed_to_date: dec!(400000),
            unbilled_revenue: dec!(100000),
            gross_margin_pct: dec!(0.20),
        };

        // PoC: 400000 / 800000 = 0.50
        assert_eq!(rev.computed_completion_pct(), dec!(0.5000));
        // Cumulative revenue: 1000000 * 0.50 = 500000
        assert_eq!(rev.computed_cumulative_revenue(), dec!(500000.00));
        // Unbilled: 500000 - 400000 = 100000
        assert_eq!(rev.computed_unbilled_revenue(), dec!(100000.00));
        // Margin: (1000000 - 800000) / 1000000 = 0.20
        assert_eq!(rev.computed_gross_margin_pct(), dec!(0.2000));
    }

    #[test]
    fn test_revenue_zero_estimated_cost() {
        let rev = ProjectRevenue {
            id: "REV-002".to_string(),
            project_id: "PRJ-002".to_string(),
            entity_id: "C001".to_string(),
            period_start: d("2025-01-01"),
            period_end: d("2025-03-31"),
            contract_value: dec!(100000),
            estimated_total_cost: Decimal::ZERO,
            costs_to_date: Decimal::ZERO,
            completion_pct: Decimal::ZERO,
            method: RevenueMethod::PercentageOfCompletion,
            measure: CompletionMeasure::CostToCost,
            cumulative_revenue: Decimal::ZERO,
            period_revenue: Decimal::ZERO,
            billed_to_date: Decimal::ZERO,
            unbilled_revenue: Decimal::ZERO,
            gross_margin_pct: Decimal::ZERO,
        };
        assert_eq!(rev.computed_completion_pct(), Decimal::ZERO);
    }

    // -- Milestones --

    #[test]
    fn test_milestone_creation() {
        let ms = ProjectMilestone::new("MS-001", "PRJ-001", "Foundation Complete", d("2025-06-30"), 1)
            .with_wbs("PRJ-001.02")
            .with_payment(dec!(50000))
            .with_weight(dec!(0.25));

        assert_eq!(ms.status, MilestoneStatus::Pending);
        assert_eq!(ms.payment_amount, dec!(50000));
        assert_eq!(ms.weight, dec!(0.25));
        assert!(ms.is_overdue_on(d("2025-07-01")));
        assert!(!ms.is_overdue_on(d("2025-06-15")));
    }

    #[test]
    fn test_milestone_variance() {
        let mut ms = ProjectMilestone::new("MS-002", "PRJ-001", "Testing", d("2025-09-30"), 3);
        assert_eq!(ms.days_variance(), None);

        // Complete 5 days late
        ms.actual_date = Some(d("2025-10-05"));
        ms.status = MilestoneStatus::Completed;
        assert_eq!(ms.days_variance(), Some(5));

        // Complete 3 days early
        let mut ms2 = ProjectMilestone::new("MS-003", "PRJ-001", "Delivery", d("2025-12-31"), 4);
        ms2.actual_date = Some(d("2025-12-28"));
        ms2.status = MilestoneStatus::Completed;
        assert_eq!(ms2.days_variance(), Some(-3));
    }

    // -- Change Orders --

    #[test]
    fn test_change_order_approval() {
        let co = ChangeOrder::new(
            "CO-001",
            "PRJ-001",
            1,
            d("2025-04-15"),
            ChangeReason::ScopeChange,
            "Add additional floor to building",
        )
        .with_cost_impact(dec!(200000), dec!(180000))
        .with_schedule_impact(30);

        assert!(!co.is_approved());
        assert_eq!(co.net_cost_impact(), Decimal::ZERO);

        let co_approved = co.approve(d("2025-04-25"));
        assert!(co_approved.is_approved());
        assert_eq!(co_approved.net_cost_impact(), dec!(200000));
        assert_eq!(co_approved.schedule_impact_days, 30);
    }

    // -- Retainage --

    #[test]
    fn test_retainage_hold_and_release() {
        let mut ret = Retainage::new(
            "RET-001",
            "PRJ-001",
            "C001",
            "V-001",
            dec!(0.10), // 10%
            d("2025-01-15"),
        );

        // Add retainage from three payments
        ret.add_from_payment(dec!(100000));
        ret.add_from_payment(dec!(150000));
        ret.add_from_payment(dec!(75000));

        // Total held: (100000 + 150000 + 75000) * 0.10 = 32500
        assert_eq!(ret.total_held, dec!(32500.00));
        assert_eq!(ret.balance_held(), dec!(32500.00));
        assert_eq!(ret.status, RetainageStatus::Held);

        // Partial release
        ret.release(dec!(15000), d("2025-06-30"));
        assert_eq!(ret.balance_held(), dec!(17500.00));
        assert_eq!(ret.status, RetainageStatus::PartiallyReleased);

        // Full release
        ret.release(dec!(17500), d("2025-12-31"));
        assert_eq!(ret.balance_held(), dec!(0.00));
        assert_eq!(ret.status, RetainageStatus::Released);
    }

    #[test]
    fn test_retainage_release_capped() {
        let mut ret = Retainage::new(
            "RET-002", "PRJ-001", "C001", "V-001",
            dec!(0.10), d("2025-01-15"),
        );
        ret.add_from_payment(dec!(100000)); // held = 10000

        // Try to release more than held
        ret.release(dec!(50000), d("2025-12-31"));
        assert_eq!(ret.released_amount, dec!(10000.00)); // capped
        assert_eq!(ret.balance_held(), dec!(0.00));
        assert_eq!(ret.status, RetainageStatus::Released);
    }

    // -- Earned Value Management --

    #[test]
    fn test_evm_formulas() {
        // Project: BAC=1,000,000, 50% scheduled, 40% earned, 450,000 spent
        let evm = EarnedValueMetric::compute(
            "EVM-001",
            "PRJ-001",
            d("2025-06-30"),
            dec!(1000000),   // BAC
            dec!(500000),    // PV (50% scheduled)
            dec!(400000),    // EV (40% earned)
            dec!(450000),    // AC (450k spent)
        );

        // SV = EV - PV = 400000 - 500000 = -100000 (behind schedule)
        assert_eq!(evm.schedule_variance, dec!(-100000.00));
        // CV = EV - AC = 400000 - 450000 = -50000 (over budget)
        assert_eq!(evm.cost_variance, dec!(-50000.00));
        // SPI = EV / PV = 400000 / 500000 = 0.80
        assert_eq!(evm.spi, dec!(0.8000));
        // CPI = EV / AC = 400000 / 450000 = 0.8889
        assert_eq!(evm.cpi, dec!(0.8889));
        // EAC = BAC / CPI = 1000000 / 0.8889 ≈ 1124972.44
        let expected_eac = (dec!(1000000) / dec!(0.8889)).round_dp(2);
        assert_eq!(evm.eac, expected_eac);
        // ETC = EAC - AC
        assert_eq!(evm.etc, (evm.eac - dec!(450000)).round_dp(2));
        // TCPI = (BAC - EV) / (BAC - AC) = 600000 / 550000 ≈ 1.0909
        assert_eq!(evm.tcpi, dec!(1.0909));

        assert!(!evm.is_ahead_of_schedule());
        assert!(!evm.is_under_budget());
        assert!(!evm.is_healthy());
    }

    #[test]
    fn test_evm_healthy_project() {
        let evm = EarnedValueMetric::compute(
            "EVM-002",
            "PRJ-002",
            d("2025-06-30"),
            dec!(500000),    // BAC
            dec!(250000),    // PV (50% scheduled)
            dec!(275000),    // EV (55% earned — ahead)
            dec!(240000),    // AC (240k — under budget)
        );

        // SPI = 275000 / 250000 = 1.10
        assert_eq!(evm.spi, dec!(1.1000));
        // CPI = 275000 / 240000 ≈ 1.1458
        assert_eq!(evm.cpi, dec!(1.1458));

        assert!(evm.is_ahead_of_schedule());
        assert!(evm.is_under_budget());
        assert!(evm.is_healthy());

        // VAC = BAC - EAC = 500000 - (500000 / 1.1458)
        assert!(evm.variance_at_completion() > Decimal::ZERO);
    }

    #[test]
    fn test_evm_zero_inputs() {
        // Edge case: all zeros (project just started)
        let evm = EarnedValueMetric::compute(
            "EVM-003",
            "PRJ-003",
            d("2025-01-01"),
            dec!(1000000),   // BAC
            Decimal::ZERO,   // PV
            Decimal::ZERO,   // EV
            Decimal::ZERO,   // AC
        );

        // With PV=0 and AC=0, SPI and CPI default to 1.0
        assert_eq!(evm.spi, dec!(1.00));
        assert_eq!(evm.cpi, dec!(1.00));
        assert_eq!(evm.eac, dec!(1000000));
    }

    // -- Serde roundtrip --

    #[test]
    fn test_cost_line_serde_roundtrip() {
        let line = ProjectCostLine::new(
            "PCL-100",
            "PRJ-001",
            "PRJ-001.01",
            "C001",
            d("2025-03-15"),
            CostCategory::Subcontractor,
            CostSourceType::VendorInvoice,
            "VI-099",
            dec!(25000),
            "EUR",
        );
        let json = serde_json::to_string(&line).unwrap();
        let deserialized: ProjectCostLine = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "PCL-100");
        assert_eq!(deserialized.cost_category, CostCategory::Subcontractor);
        assert_eq!(deserialized.amount, dec!(25000));
    }

    #[test]
    fn test_evm_serde_roundtrip() {
        let evm = EarnedValueMetric::compute(
            "EVM-100", "PRJ-001", d("2025-06-30"),
            dec!(1000000), dec!(500000), dec!(400000), dec!(450000),
        );
        let json = serde_json::to_string(&evm).unwrap();
        let deserialized: EarnedValueMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.spi, evm.spi);
        assert_eq!(deserialized.cpi, evm.cpi);
        assert_eq!(deserialized.eac, evm.eac);
    }
}
