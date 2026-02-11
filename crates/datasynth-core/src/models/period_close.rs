//! Period close models.
//!
//! This module provides models for fiscal period management and
//! period-end close processes including:
//! - Fiscal period definitions
//! - Fiscal calendar types (calendar year, custom year start, 4-4-5, 13-period)
//! - Close tasks and workflows
//! - Accrual definitions and schedules
//! - Year-end closing entries

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Type of fiscal calendar used by the organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FiscalCalendarType {
    /// Standard calendar year (Jan 1 - Dec 31).
    #[default]
    CalendarYear,
    /// Custom year start (e.g., July 1 for government fiscal years).
    CustomYearStart {
        /// Month the fiscal year starts (1-12).
        start_month: u8,
        /// Day the fiscal year starts (1-31).
        start_day: u8,
    },
    /// 4-4-5 retail calendar (52/53 week years with 4-4-5, 4-5-4, or 5-4-4 pattern).
    FourFourFive(FourFourFiveConfig),
    /// 13-period calendar (13 equal 4-week periods).
    ThirteenPeriod(ThirteenPeriodConfig),
}

/// Configuration for 4-4-5 retail calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourFourFiveConfig {
    /// Week pattern for each quarter.
    pub pattern: WeekPattern,
    /// Anchor point for determining fiscal year start.
    pub anchor: FourFourFiveAnchor,
    /// Where to place the leap week in 53-week years.
    pub leap_week_placement: LeapWeekPlacement,
}

impl Default for FourFourFiveConfig {
    fn default() -> Self {
        Self {
            pattern: WeekPattern::FourFourFive,
            anchor: FourFourFiveAnchor::LastSaturdayOf(1), // Last Saturday of January
            leap_week_placement: LeapWeekPlacement::Q4Period3,
        }
    }
}

/// Week pattern for 4-4-5 calendar quarters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeekPattern {
    /// 4 weeks, 4 weeks, 5 weeks per quarter.
    FourFourFive,
    /// 4 weeks, 5 weeks, 4 weeks per quarter.
    FourFiveFour,
    /// 5 weeks, 4 weeks, 4 weeks per quarter.
    FiveFourFour,
}

impl WeekPattern {
    /// Returns the number of weeks in each period of a quarter.
    pub fn weeks_per_period(&self) -> [u8; 3] {
        match self {
            WeekPattern::FourFourFive => [4, 4, 5],
            WeekPattern::FourFiveFour => [4, 5, 4],
            WeekPattern::FiveFourFour => [5, 4, 4],
        }
    }
}

/// Anchor point for 4-4-5 fiscal year start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "month", rename_all = "snake_case")]
pub enum FourFourFiveAnchor {
    /// Fiscal year starts on the first Sunday of a month.
    FirstSundayOf(u8),
    /// Fiscal year starts on the last Saturday of a month.
    LastSaturdayOf(u8),
    /// Fiscal year ends on the Saturday nearest to a month end.
    NearestSaturdayTo(u8),
}

/// Where to place the leap week in 53-week years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeapWeekPlacement {
    /// Add leap week to Q4 Period 3 (most common).
    Q4Period3,
    /// Add leap week to Q1 Period 1.
    Q1Period1,
}

/// Configuration for 13-period calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThirteenPeriodConfig {
    /// First day of fiscal year (day of year, 1-366).
    pub year_start_day: u16,
    /// Month containing year start (for display purposes).
    pub year_start_month: u8,
}

impl Default for ThirteenPeriodConfig {
    fn default() -> Self {
        Self {
            year_start_day: 1,   // January 1
            year_start_month: 1, // January
        }
    }
}

/// Fiscal calendar definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiscalCalendar {
    /// Type of fiscal calendar.
    pub calendar_type: FiscalCalendarType,
    /// Name of the fiscal calendar (e.g., "US Federal", "Retail 445").
    pub name: String,
}

impl Default for FiscalCalendar {
    fn default() -> Self {
        Self {
            calendar_type: FiscalCalendarType::CalendarYear,
            name: "Calendar Year".to_string(),
        }
    }
}

impl FiscalCalendar {
    /// Creates a standard calendar year fiscal calendar.
    pub fn calendar_year() -> Self {
        Self::default()
    }

    /// Creates a fiscal calendar with custom year start.
    pub fn custom_year_start(start_month: u8, start_day: u8) -> Self {
        Self {
            calendar_type: FiscalCalendarType::CustomYearStart {
                start_month,
                start_day,
            },
            name: format!("Fiscal Year ({})", month_name(start_month)),
        }
    }

    /// Creates a 4-4-5 retail calendar.
    pub fn four_four_five(config: FourFourFiveConfig) -> Self {
        Self {
            calendar_type: FiscalCalendarType::FourFourFive(config),
            name: "Retail 4-4-5".to_string(),
        }
    }

    /// Creates a 13-period calendar.
    pub fn thirteen_period(config: ThirteenPeriodConfig) -> Self {
        Self {
            calendar_type: FiscalCalendarType::ThirteenPeriod(config),
            name: "13-Period".to_string(),
        }
    }

    /// Returns the fiscal year for a given date.
    pub fn fiscal_year(&self, date: NaiveDate) -> i32 {
        match &self.calendar_type {
            FiscalCalendarType::CalendarYear => date.year(),
            FiscalCalendarType::CustomYearStart {
                start_month,
                start_day,
            } => {
                let year_start =
                    NaiveDate::from_ymd_opt(date.year(), *start_month as u32, *start_day as u32)
                        .unwrap_or_else(|| {
                            NaiveDate::from_ymd_opt(date.year(), *start_month as u32, 1)
                                .expect("valid date components")
                        });
                if date >= year_start {
                    date.year()
                } else {
                    date.year() - 1
                }
            }
            FiscalCalendarType::FourFourFive(_) | FiscalCalendarType::ThirteenPeriod(_) => {
                // Simplified - would need more complex calculation
                date.year()
            }
        }
    }

    /// Returns the fiscal period number for a given date.
    pub fn fiscal_period(&self, date: NaiveDate) -> u8 {
        match &self.calendar_type {
            FiscalCalendarType::CalendarYear => date.month() as u8,
            FiscalCalendarType::CustomYearStart {
                start_month,
                start_day: _,
            } => {
                let month = date.month() as u8;
                if month >= *start_month {
                    month - start_month + 1
                } else {
                    12 - start_month + month + 1
                }
            }
            FiscalCalendarType::ThirteenPeriod(_) => {
                // Simplified: 28 days per period
                let day_of_year = date.ordinal();
                ((day_of_year - 1) / 28 + 1).min(13) as u8
            }
            FiscalCalendarType::FourFourFive(config) => {
                // Simplified 4-4-5 period calculation
                let weeks = config.pattern.weeks_per_period();
                let week_of_year = (date.ordinal() as u8 - 1) / 7 + 1;
                let mut cumulative = 0u8;
                for (quarter, _) in (0..4).enumerate() {
                    for (period_in_q, &period_weeks) in weeks.iter().enumerate() {
                        cumulative += period_weeks;
                        if week_of_year <= cumulative {
                            return (quarter * 3 + period_in_q + 1) as u8;
                        }
                    }
                }
                12 // Default to period 12
            }
        }
    }
}

/// Helper function to get month name.
fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Fiscal period representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FiscalPeriod {
    /// Fiscal year.
    pub year: i32,
    /// Period number (1-12 for monthly, 1-4 for quarterly).
    pub period: u8,
    /// Period start date.
    pub start_date: NaiveDate,
    /// Period end date.
    pub end_date: NaiveDate,
    /// Period type.
    pub period_type: FiscalPeriodType,
    /// Is this the year-end period?
    pub is_year_end: bool,
    /// Period status.
    pub status: PeriodStatus,
}

impl FiscalPeriod {
    /// Creates a monthly fiscal period.
    pub fn monthly(year: i32, month: u8) -> Self {
        let start_date =
            NaiveDate::from_ymd_opt(year, month as u32, 1).expect("valid date components");
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .expect("valid date components")
                .pred_opt()
                .expect("valid date components")
        } else {
            NaiveDate::from_ymd_opt(year, month as u32 + 1, 1)
                .expect("valid date components")
                .pred_opt()
                .expect("valid date components")
        };

        Self {
            year,
            period: month,
            start_date,
            end_date,
            period_type: FiscalPeriodType::Monthly,
            is_year_end: month == 12,
            status: PeriodStatus::Open,
        }
    }

    /// Creates a quarterly fiscal period.
    pub fn quarterly(year: i32, quarter: u8) -> Self {
        let start_month = (quarter - 1) * 3 + 1;
        let end_month = quarter * 3;

        let start_date =
            NaiveDate::from_ymd_opt(year, start_month as u32, 1).expect("valid date components");
        let end_date = if end_month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .expect("valid date components")
                .pred_opt()
                .expect("valid date components")
        } else {
            NaiveDate::from_ymd_opt(year, end_month as u32 + 1, 1)
                .expect("valid date components")
                .pred_opt()
                .expect("valid date components")
        };

        Self {
            year,
            period: quarter,
            start_date,
            end_date,
            period_type: FiscalPeriodType::Quarterly,
            is_year_end: quarter == 4,
            status: PeriodStatus::Open,
        }
    }

    /// Returns the number of days in the period.
    pub fn days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }

    /// Returns the period key (e.g., "2024-01" for monthly).
    pub fn key(&self) -> String {
        format!("{}-{:02}", self.year, self.period)
    }

    /// Checks if a date falls within this period.
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }

    /// Creates a fiscal period from a calendar and date.
    ///
    /// This method determines the correct fiscal period for a given date
    /// based on the fiscal calendar configuration.
    pub fn from_calendar(calendar: &FiscalCalendar, date: NaiveDate) -> Self {
        let fiscal_year = calendar.fiscal_year(date);
        let period_num = calendar.fiscal_period(date);

        match &calendar.calendar_type {
            FiscalCalendarType::CalendarYear => Self::monthly(fiscal_year, period_num),
            FiscalCalendarType::CustomYearStart {
                start_month,
                start_day,
            } => {
                // Calculate the actual start and end dates for the period
                let period_start_month = if *start_month + period_num - 1 > 12 {
                    start_month + period_num - 1 - 12
                } else {
                    start_month + period_num - 1
                };
                let period_year = if *start_month + period_num - 1 > 12 {
                    fiscal_year + 1
                } else {
                    fiscal_year
                };

                let start_date = if period_num == 1 {
                    NaiveDate::from_ymd_opt(fiscal_year, *start_month as u32, *start_day as u32)
                        .unwrap_or_else(|| {
                            NaiveDate::from_ymd_opt(fiscal_year, *start_month as u32, 1)
                                .expect("valid date components")
                        })
                } else {
                    NaiveDate::from_ymd_opt(period_year, period_start_month as u32, 1)
                        .expect("valid date components")
                };

                let end_date = if period_num == 12 {
                    // Last period ends day before fiscal year start
                    NaiveDate::from_ymd_opt(fiscal_year + 1, *start_month as u32, *start_day as u32)
                        .unwrap_or_else(|| {
                            NaiveDate::from_ymd_opt(fiscal_year + 1, *start_month as u32, 1)
                                .expect("valid date components")
                        })
                        .pred_opt()
                        .expect("valid date components")
                } else {
                    let next_month = if period_start_month == 12 {
                        1
                    } else {
                        period_start_month + 1
                    };
                    let next_year = if period_start_month == 12 {
                        period_year + 1
                    } else {
                        period_year
                    };
                    NaiveDate::from_ymd_opt(next_year, next_month as u32, 1)
                        .expect("valid date components")
                        .pred_opt()
                        .expect("valid date components")
                };

                Self {
                    year: fiscal_year,
                    period: period_num,
                    start_date,
                    end_date,
                    period_type: FiscalPeriodType::Monthly,
                    is_year_end: period_num == 12,
                    status: PeriodStatus::Open,
                }
            }
            FiscalCalendarType::FourFourFive(config) => {
                // 4-4-5 calendar: 12 periods of 4 or 5 weeks each
                let weeks = config.pattern.weeks_per_period();
                let quarter = (period_num - 1) / 3;
                let period_in_quarter = (period_num - 1) % 3;
                let period_weeks = weeks[period_in_quarter as usize];

                // Calculate start of fiscal year (simplified)
                let year_start =
                    NaiveDate::from_ymd_opt(fiscal_year, 1, 1).expect("valid date components");

                // Calculate period start by summing previous period weeks
                let mut weeks_before = 0u32;
                for _ in 0..quarter {
                    for &w in &weeks {
                        weeks_before += w as u32;
                    }
                }
                for p in 0..period_in_quarter {
                    weeks_before += weeks[p as usize] as u32;
                }

                let start_date = year_start + chrono::Duration::weeks(weeks_before as i64);
                let end_date = start_date + chrono::Duration::weeks(period_weeks as i64)
                    - chrono::Duration::days(1);

                Self {
                    year: fiscal_year,
                    period: period_num,
                    start_date,
                    end_date,
                    period_type: FiscalPeriodType::FourWeek,
                    is_year_end: period_num == 12,
                    status: PeriodStatus::Open,
                }
            }
            FiscalCalendarType::ThirteenPeriod(_) => {
                // 13 periods of 28 days each (4 weeks)
                let year_start =
                    NaiveDate::from_ymd_opt(fiscal_year, 1, 1).expect("valid date components");
                let start_date = year_start + chrono::Duration::days((period_num as i64 - 1) * 28);
                let end_date = start_date + chrono::Duration::days(27);

                Self {
                    year: fiscal_year,
                    period: period_num,
                    start_date,
                    end_date,
                    period_type: FiscalPeriodType::FourWeek,
                    is_year_end: period_num == 13,
                    status: PeriodStatus::Open,
                }
            }
        }
    }
}

/// Type of fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FiscalPeriodType {
    /// Monthly period.
    Monthly,
    /// Quarterly period.
    Quarterly,
    /// Four-week period (used in 4-4-5 and 13-period calendars).
    FourWeek,
    /// Special period (13th period, adjustments).
    Special,
}

/// Status of a fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeriodStatus {
    /// Period is open for posting.
    Open,
    /// Soft close - limited posting allowed.
    SoftClosed,
    /// Hard close - no posting allowed.
    Closed,
    /// Period is locked for audit.
    Locked,
}

/// Close task types for period-end processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CloseTask {
    /// Run depreciation for fixed assets.
    RunDepreciation,
    /// Post inventory revaluation adjustments.
    PostInventoryRevaluation,
    /// Reconcile AR subledger to GL.
    ReconcileArToGl,
    /// Reconcile AP subledger to GL.
    ReconcileApToGl,
    /// Reconcile FA subledger to GL.
    ReconcileFaToGl,
    /// Reconcile Inventory to GL.
    ReconcileInventoryToGl,
    /// Post accrued expenses.
    PostAccruedExpenses,
    /// Post accrued revenue.
    PostAccruedRevenue,
    /// Post prepaid expense amortization.
    PostPrepaidAmortization,
    /// Allocate corporate overhead.
    AllocateCorporateOverhead,
    /// Post intercompany settlements.
    PostIntercompanySettlements,
    /// Revalue foreign currency balances.
    RevalueForeignCurrency,
    /// Calculate and post tax provision.
    CalculateTaxProvision,
    /// Translate foreign subsidiary trial balances.
    TranslateForeignSubsidiaries,
    /// Eliminate intercompany balances.
    EliminateIntercompany,
    /// Generate trial balance.
    GenerateTrialBalance,
    /// Generate financial statements.
    GenerateFinancialStatements,
    /// Close income statement accounts (year-end).
    CloseIncomeStatement,
    /// Post retained earnings rollforward (year-end).
    PostRetainedEarningsRollforward,
    /// Custom task.
    Custom(String),
}

impl CloseTask {
    /// Returns true if this is a year-end only task.
    pub fn is_year_end_only(&self) -> bool {
        matches!(
            self,
            CloseTask::CloseIncomeStatement | CloseTask::PostRetainedEarningsRollforward
        )
    }

    /// Returns the task name.
    pub fn name(&self) -> &str {
        match self {
            CloseTask::RunDepreciation => "Run Depreciation",
            CloseTask::PostInventoryRevaluation => "Post Inventory Revaluation",
            CloseTask::ReconcileArToGl => "Reconcile AR to GL",
            CloseTask::ReconcileApToGl => "Reconcile AP to GL",
            CloseTask::ReconcileFaToGl => "Reconcile FA to GL",
            CloseTask::ReconcileInventoryToGl => "Reconcile Inventory to GL",
            CloseTask::PostAccruedExpenses => "Post Accrued Expenses",
            CloseTask::PostAccruedRevenue => "Post Accrued Revenue",
            CloseTask::PostPrepaidAmortization => "Post Prepaid Amortization",
            CloseTask::AllocateCorporateOverhead => "Allocate Corporate Overhead",
            CloseTask::PostIntercompanySettlements => "Post IC Settlements",
            CloseTask::RevalueForeignCurrency => "Revalue Foreign Currency",
            CloseTask::CalculateTaxProvision => "Calculate Tax Provision",
            CloseTask::TranslateForeignSubsidiaries => "Translate Foreign Subs",
            CloseTask::EliminateIntercompany => "Eliminate Intercompany",
            CloseTask::GenerateTrialBalance => "Generate Trial Balance",
            CloseTask::GenerateFinancialStatements => "Generate Financials",
            CloseTask::CloseIncomeStatement => "Close Income Statement",
            CloseTask::PostRetainedEarningsRollforward => "Post RE Rollforward",
            CloseTask::Custom(name) => name,
        }
    }
}

/// Status of a close task execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseTaskStatus {
    /// Not started.
    Pending,
    /// In progress.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Completed with warnings.
    CompletedWithWarnings(Vec<String>),
    /// Failed.
    Failed(String),
    /// Skipped.
    Skipped(String),
}

/// Result of executing a close task.
#[derive(Debug, Clone)]
pub struct CloseTaskResult {
    /// Task that was executed.
    pub task: CloseTask,
    /// Company code.
    pub company_code: String,
    /// Fiscal period.
    pub fiscal_period: FiscalPeriod,
    /// Status.
    pub status: CloseTaskStatus,
    /// Start time.
    pub started_at: Option<NaiveDate>,
    /// End time.
    pub completed_at: Option<NaiveDate>,
    /// Journal entries created.
    pub journal_entries_created: u32,
    /// Total amount posted.
    pub total_amount: Decimal,
    /// Execution notes.
    pub notes: Vec<String>,
}

impl CloseTaskResult {
    /// Creates a new task result.
    pub fn new(task: CloseTask, company_code: String, fiscal_period: FiscalPeriod) -> Self {
        Self {
            task,
            company_code,
            fiscal_period,
            status: CloseTaskStatus::Pending,
            started_at: None,
            completed_at: None,
            journal_entries_created: 0,
            total_amount: Decimal::ZERO,
            notes: Vec::new(),
        }
    }

    /// Returns true if the task completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            CloseTaskStatus::Completed | CloseTaskStatus::CompletedWithWarnings(_)
        )
    }
}

/// Accrual definition for recurring period-end entries.
#[derive(Debug, Clone)]
pub struct AccrualDefinition {
    /// Accrual ID.
    pub accrual_id: String,
    /// Company code.
    pub company_code: String,
    /// Description.
    pub description: String,
    /// Accrual type.
    pub accrual_type: AccrualType,
    /// Expense/Revenue account to debit/credit.
    pub expense_revenue_account: String,
    /// Accrual liability/asset account.
    pub accrual_account: String,
    /// Calculation method.
    pub calculation_method: AccrualCalculationMethod,
    /// Fixed amount (if applicable).
    pub fixed_amount: Option<Decimal>,
    /// Percentage rate (if applicable).
    pub percentage_rate: Option<Decimal>,
    /// Base account for percentage calculation.
    pub base_account: Option<String>,
    /// Frequency.
    pub frequency: AccrualFrequency,
    /// Auto-reverse on first day of next period.
    pub auto_reverse: bool,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Active flag.
    pub is_active: bool,
    /// Start date.
    pub effective_from: NaiveDate,
    /// End date (if defined).
    pub effective_to: Option<NaiveDate>,
}

impl AccrualDefinition {
    /// Creates a new accrual definition.
    pub fn new(
        accrual_id: String,
        company_code: String,
        description: String,
        accrual_type: AccrualType,
        expense_revenue_account: String,
        accrual_account: String,
    ) -> Self {
        Self {
            accrual_id,
            company_code,
            description,
            accrual_type,
            expense_revenue_account,
            accrual_account,
            calculation_method: AccrualCalculationMethod::FixedAmount,
            fixed_amount: None,
            percentage_rate: None,
            base_account: None,
            frequency: AccrualFrequency::Monthly,
            auto_reverse: true,
            cost_center: None,
            is_active: true,
            effective_from: NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid date components"),
            effective_to: None,
        }
    }

    /// Sets the fixed amount.
    pub fn with_fixed_amount(mut self, amount: Decimal) -> Self {
        self.calculation_method = AccrualCalculationMethod::FixedAmount;
        self.fixed_amount = Some(amount);
        self
    }

    /// Sets percentage-based calculation.
    pub fn with_percentage(mut self, rate: Decimal, base_account: &str) -> Self {
        self.calculation_method = AccrualCalculationMethod::PercentageOfBase;
        self.percentage_rate = Some(rate);
        self.base_account = Some(base_account.to_string());
        self
    }

    /// Checks if the accrual is effective for a given date.
    pub fn is_effective_on(&self, date: NaiveDate) -> bool {
        if !self.is_active {
            return false;
        }
        if date < self.effective_from {
            return false;
        }
        if let Some(end) = self.effective_to {
            if date > end {
                return false;
            }
        }
        true
    }
}

/// Type of accrual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualType {
    /// Accrued expense (debit expense, credit liability).
    AccruedExpense,
    /// Accrued revenue (debit asset, credit revenue).
    AccruedRevenue,
    /// Prepaid expense (debit expense, credit asset).
    PrepaidExpense,
    /// Deferred revenue (debit liability, credit revenue).
    DeferredRevenue,
}

/// Calculation method for accruals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualCalculationMethod {
    /// Fixed amount each period.
    FixedAmount,
    /// Percentage of a base account balance.
    PercentageOfBase,
    /// Days-based proration.
    DaysBased,
    /// Calculated externally (manual entry).
    Manual,
}

/// Frequency for accrual posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualFrequency {
    /// Every month.
    Monthly,
    /// Every quarter.
    Quarterly,
    /// Every year.
    Annually,
}

/// Corporate overhead allocation definition.
#[derive(Debug, Clone)]
pub struct OverheadAllocation {
    /// Allocation ID.
    pub allocation_id: String,
    /// Source company code (corporate).
    pub source_company: String,
    /// Source cost center.
    pub source_cost_center: String,
    /// Source account.
    pub source_account: String,
    /// Allocation basis.
    pub allocation_basis: AllocationBasis,
    /// Target allocations.
    pub targets: Vec<AllocationTarget>,
    /// Description.
    pub description: String,
    /// Active flag.
    pub is_active: bool,
}

/// Basis for overhead allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationBasis {
    /// Based on revenue.
    Revenue,
    /// Based on headcount.
    Headcount,
    /// Based on direct costs.
    DirectCosts,
    /// Based on square footage.
    SquareFootage,
    /// Fixed percentages.
    FixedPercentage,
    /// Custom formula.
    Custom(String),
}

/// Target for overhead allocation.
#[derive(Debug, Clone)]
pub struct AllocationTarget {
    /// Target company code.
    pub company_code: String,
    /// Target cost center.
    pub cost_center: String,
    /// Target account.
    pub account: String,
    /// Allocation percentage (for fixed percentage basis).
    pub percentage: Option<Decimal>,
    /// Allocation driver value (for calculated basis).
    pub driver_value: Option<Decimal>,
}

/// Period close schedule defining the order of tasks.
#[derive(Debug, Clone)]
pub struct CloseSchedule {
    /// Schedule ID.
    pub schedule_id: String,
    /// Company code (or "ALL" for all companies).
    pub company_code: String,
    /// Period type this schedule applies to.
    pub period_type: FiscalPeriodType,
    /// Ordered list of tasks.
    pub tasks: Vec<ScheduledCloseTask>,
    /// Whether this is for year-end.
    pub is_year_end: bool,
}

impl CloseSchedule {
    /// Creates a standard monthly close schedule.
    pub fn standard_monthly(company_code: &str) -> Self {
        Self {
            schedule_id: format!("MONTHLY-{}", company_code),
            company_code: company_code.to_string(),
            period_type: FiscalPeriodType::Monthly,
            tasks: vec![
                ScheduledCloseTask::new(CloseTask::RunDepreciation, 1),
                ScheduledCloseTask::new(CloseTask::PostInventoryRevaluation, 2),
                ScheduledCloseTask::new(CloseTask::PostAccruedExpenses, 3),
                ScheduledCloseTask::new(CloseTask::PostAccruedRevenue, 4),
                ScheduledCloseTask::new(CloseTask::PostPrepaidAmortization, 5),
                ScheduledCloseTask::new(CloseTask::RevalueForeignCurrency, 6),
                ScheduledCloseTask::new(CloseTask::ReconcileArToGl, 7),
                ScheduledCloseTask::new(CloseTask::ReconcileApToGl, 8),
                ScheduledCloseTask::new(CloseTask::ReconcileFaToGl, 9),
                ScheduledCloseTask::new(CloseTask::ReconcileInventoryToGl, 10),
                ScheduledCloseTask::new(CloseTask::PostIntercompanySettlements, 11),
                ScheduledCloseTask::new(CloseTask::AllocateCorporateOverhead, 12),
                ScheduledCloseTask::new(CloseTask::TranslateForeignSubsidiaries, 13),
                ScheduledCloseTask::new(CloseTask::EliminateIntercompany, 14),
                ScheduledCloseTask::new(CloseTask::GenerateTrialBalance, 15),
            ],
            is_year_end: false,
        }
    }

    /// Creates a year-end close schedule.
    pub fn year_end(company_code: &str) -> Self {
        let mut schedule = Self::standard_monthly(company_code);
        schedule.schedule_id = format!("YEAREND-{}", company_code);
        schedule.is_year_end = true;

        // Add year-end specific tasks
        let next_seq = schedule.tasks.len() as u32 + 1;
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::CalculateTaxProvision,
            next_seq,
        ));
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::CloseIncomeStatement,
            next_seq + 1,
        ));
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::PostRetainedEarningsRollforward,
            next_seq + 2,
        ));
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::GenerateFinancialStatements,
            next_seq + 3,
        ));

        schedule
    }
}

/// A scheduled close task with sequence and dependencies.
#[derive(Debug, Clone)]
pub struct ScheduledCloseTask {
    /// The task to execute.
    pub task: CloseTask,
    /// Sequence number (execution order).
    pub sequence: u32,
    /// Tasks that must complete before this one.
    pub depends_on: Vec<CloseTask>,
    /// Is this task mandatory?
    pub is_mandatory: bool,
    /// Can this task run in parallel with others at same sequence?
    pub can_parallelize: bool,
}

impl ScheduledCloseTask {
    /// Creates a new scheduled task.
    pub fn new(task: CloseTask, sequence: u32) -> Self {
        Self {
            task,
            sequence,
            depends_on: Vec::new(),
            is_mandatory: true,
            can_parallelize: false,
        }
    }

    /// Adds a dependency.
    pub fn depends_on(mut self, task: CloseTask) -> Self {
        self.depends_on.push(task);
        self
    }

    /// Marks as optional.
    pub fn optional(mut self) -> Self {
        self.is_mandatory = false;
        self
    }

    /// Allows parallel execution.
    pub fn parallelizable(mut self) -> Self {
        self.can_parallelize = true;
        self
    }
}

/// Year-end closing entry specification.
#[derive(Debug, Clone)]
pub struct YearEndClosingSpec {
    /// Company code.
    pub company_code: String,
    /// Fiscal year being closed.
    pub fiscal_year: i32,
    /// Revenue accounts to close.
    pub revenue_accounts: Vec<String>,
    /// Expense accounts to close.
    pub expense_accounts: Vec<String>,
    /// Income summary account (temporary).
    pub income_summary_account: String,
    /// Retained earnings account.
    pub retained_earnings_account: String,
    /// Dividend account (if applicable).
    pub dividend_account: Option<String>,
}

impl Default for YearEndClosingSpec {
    fn default() -> Self {
        Self {
            company_code: String::new(),
            fiscal_year: 0,
            revenue_accounts: vec!["4".to_string()], // All accounts starting with 4
            expense_accounts: vec!["5".to_string(), "6".to_string()], // Accounts starting with 5, 6
            income_summary_account: "3500".to_string(),
            retained_earnings_account: "3300".to_string(),
            dividend_account: Some("3400".to_string()),
        }
    }
}

/// Tax provision calculation inputs.
#[derive(Debug, Clone)]
pub struct TaxProvisionInput {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Pre-tax book income.
    pub pretax_income: Decimal,
    /// Permanent differences (add back).
    pub permanent_differences: Vec<TaxAdjustment>,
    /// Temporary differences (timing).
    pub temporary_differences: Vec<TaxAdjustment>,
    /// Statutory tax rate.
    pub statutory_rate: Decimal,
    /// Tax credits available.
    pub tax_credits: Decimal,
    /// Prior year over/under provision.
    pub prior_year_adjustment: Decimal,
}

/// Tax adjustment item.
#[derive(Debug, Clone)]
pub struct TaxAdjustment {
    /// Description.
    pub description: String,
    /// Amount.
    pub amount: Decimal,
    /// Is this a deduction (negative) or addition (positive)?
    pub is_addition: bool,
}

/// Tax provision result.
#[derive(Debug, Clone)]
pub struct TaxProvisionResult {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Pre-tax book income.
    pub pretax_income: Decimal,
    /// Total permanent differences.
    pub permanent_differences: Decimal,
    /// Taxable income.
    pub taxable_income: Decimal,
    /// Current tax expense.
    pub current_tax_expense: Decimal,
    /// Deferred tax expense (benefit).
    pub deferred_tax_expense: Decimal,
    /// Total tax expense.
    pub total_tax_expense: Decimal,
    /// Effective tax rate.
    pub effective_rate: Decimal,
}

impl TaxProvisionResult {
    /// Calculates the tax provision from inputs.
    pub fn calculate(input: &TaxProvisionInput) -> Self {
        let permanent_diff: Decimal = input
            .permanent_differences
            .iter()
            .map(|d| if d.is_addition { d.amount } else { -d.amount })
            .sum();

        let temporary_diff: Decimal = input
            .temporary_differences
            .iter()
            .map(|d| if d.is_addition { d.amount } else { -d.amount })
            .sum();

        let taxable_income = input.pretax_income + permanent_diff;
        let current_tax = (taxable_income * input.statutory_rate / dec!(100)).round_dp(2);
        let deferred_tax = (temporary_diff * input.statutory_rate / dec!(100)).round_dp(2);

        let total_tax =
            current_tax + deferred_tax - input.tax_credits + input.prior_year_adjustment;

        let effective_rate = if input.pretax_income != Decimal::ZERO {
            (total_tax / input.pretax_income * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code: input.company_code.clone(),
            fiscal_year: input.fiscal_year,
            pretax_income: input.pretax_income,
            permanent_differences: permanent_diff,
            taxable_income,
            current_tax_expense: current_tax,
            deferred_tax_expense: deferred_tax,
            total_tax_expense: total_tax,
            effective_rate,
        }
    }
}

/// Period close run status.
#[derive(Debug, Clone)]
pub struct PeriodCloseRun {
    /// Run ID.
    pub run_id: String,
    /// Company code.
    pub company_code: String,
    /// Fiscal period.
    pub fiscal_period: FiscalPeriod,
    /// Status.
    pub status: PeriodCloseStatus,
    /// Task results.
    pub task_results: Vec<CloseTaskResult>,
    /// Started at.
    pub started_at: Option<NaiveDate>,
    /// Completed at.
    pub completed_at: Option<NaiveDate>,
    /// Total journal entries created.
    pub total_journal_entries: u32,
    /// Errors encountered.
    pub errors: Vec<String>,
}

/// Status of a period close run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodCloseStatus {
    /// Not started.
    NotStarted,
    /// In progress.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Completed with errors.
    CompletedWithErrors,
    /// Failed.
    Failed,
}

impl PeriodCloseRun {
    /// Creates a new period close run.
    pub fn new(run_id: String, company_code: String, fiscal_period: FiscalPeriod) -> Self {
        Self {
            run_id,
            company_code,
            fiscal_period,
            status: PeriodCloseStatus::NotStarted,
            task_results: Vec::new(),
            started_at: None,
            completed_at: None,
            total_journal_entries: 0,
            errors: Vec::new(),
        }
    }

    /// Returns true if all tasks completed successfully.
    pub fn is_success(&self) -> bool {
        self.status == PeriodCloseStatus::Completed
    }

    /// Returns the number of failed tasks.
    pub fn failed_task_count(&self) -> usize {
        self.task_results
            .iter()
            .filter(|r| matches!(r.status, CloseTaskStatus::Failed(_)))
            .count()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_fiscal_period_monthly() {
        let period = FiscalPeriod::monthly(2024, 1);
        assert_eq!(
            period.start_date,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        assert_eq!(
            period.end_date,
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()
        );
        assert_eq!(period.days(), 31);
        assert!(!period.is_year_end);

        let dec_period = FiscalPeriod::monthly(2024, 12);
        assert!(dec_period.is_year_end);
    }

    #[test]
    fn test_fiscal_period_quarterly() {
        let q1 = FiscalPeriod::quarterly(2024, 1);
        assert_eq!(q1.start_date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(q1.end_date, NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());

        let q4 = FiscalPeriod::quarterly(2024, 4);
        assert!(q4.is_year_end);
    }

    #[test]
    fn test_close_schedule() {
        let schedule = CloseSchedule::standard_monthly("1000");
        assert!(!schedule.is_year_end);
        assert!(!schedule.tasks.is_empty());

        let year_end = CloseSchedule::year_end("1000");
        assert!(year_end.is_year_end);
        assert!(year_end.tasks.len() > schedule.tasks.len());
    }

    #[test]
    fn test_tax_provision() {
        let input = TaxProvisionInput {
            company_code: "1000".to_string(),
            fiscal_year: 2024,
            pretax_income: dec!(1000000),
            permanent_differences: vec![TaxAdjustment {
                description: "Meals & Entertainment".to_string(),
                amount: dec!(10000),
                is_addition: true,
            }],
            temporary_differences: vec![TaxAdjustment {
                description: "Depreciation Timing".to_string(),
                amount: dec!(50000),
                is_addition: false,
            }],
            statutory_rate: dec!(21),
            tax_credits: dec!(5000),
            prior_year_adjustment: Decimal::ZERO,
        };

        let result = TaxProvisionResult::calculate(&input);
        assert_eq!(result.taxable_income, dec!(1010000)); // 1M + 10K permanent
        assert!(result.current_tax_expense > Decimal::ZERO);
    }
}
