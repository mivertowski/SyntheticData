//! Expense report generator for the Hire-to-Retire (H2R) process.
//!
//! Generates employee expense reports with realistic line items across categories
//! (travel, meals, lodging, transportation, etc.), policy violation detection,
//! and approval workflow statuses.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::ExpenseConfig;
use datasynth_core::models::{ExpenseCategory, ExpenseLineItem, ExpenseReport, ExpenseStatus};
use datasynth_core::utils::{sample_decimal_range, seeded_rng};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::debug;

/// Generates [`ExpenseReport`] records for employees over a period.
pub struct ExpenseReportGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    item_uuid_factory: DeterministicUuidFactory,
    // Stored for future use by with_config(); generate() currently takes config as a parameter.
    #[allow(dead_code)]
    config: ExpenseConfig,
    /// Pool of real employee IDs for approved_by references.
    employee_ids_pool: Vec<String>,
    /// Pool of real cost center IDs.
    cost_center_ids_pool: Vec<String>,
    /// Mapping of employee_id → employee_name for denormalization (DS-011).
    employee_names: HashMap<String, String>,
}

impl ExpenseReportGenerator {
    /// Create a new expense report generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ExpenseReport),
            item_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::ExpenseReport,
                1,
            ),
            config: ExpenseConfig::default(),
            employee_ids_pool: Vec::new(),
            cost_center_ids_pool: Vec::new(),
            employee_names: HashMap::new(),
        }
    }

    /// Create an expense report generator with custom configuration.
    pub fn with_config(seed: u64, config: ExpenseConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ExpenseReport),
            item_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::ExpenseReport,
                1,
            ),
            config,
            employee_ids_pool: Vec::new(),
            cost_center_ids_pool: Vec::new(),
            employee_names: HashMap::new(),
        }
    }

    /// Set ID pools for cross-reference coherence.
    ///
    /// When pools are non-empty, the generator selects `approved_by` from
    /// `employee_ids` and `cost_center` from `cost_center_ids` instead of
    /// fabricating placeholder IDs.
    pub fn with_pools(mut self, employee_ids: Vec<String>, cost_center_ids: Vec<String>) -> Self {
        self.employee_ids_pool = employee_ids;
        self.cost_center_ids_pool = cost_center_ids;
        self
    }

    /// Set the employee name mapping for denormalization (DS-011).
    ///
    /// Maps employee IDs to their display names so that generated expense
    /// reports include the employee name for graph export convenience.
    pub fn with_employee_names(mut self, names: HashMap<String, String>) -> Self {
        self.employee_names = names;
        self
    }

    /// Generate expense reports for employees over the given period.
    ///
    /// Only `config.submission_rate` fraction of employees submit reports each
    /// month within the period.
    ///
    /// # Arguments
    ///
    /// * `employee_ids` - Slice of employee identifiers
    /// * `period_start` - Start of the period (inclusive)
    /// * `period_end` - End of the period (inclusive)
    /// * `config` - Expense management configuration
    pub fn generate(
        &mut self,
        employee_ids: &[String],
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &ExpenseConfig,
    ) -> Vec<ExpenseReport> {
        self.generate_with_currency(employee_ids, period_start, period_end, config, "USD")
    }

    /// Generate expense reports with a specific company currency.
    pub fn generate_with_currency(
        &mut self,
        employee_ids: &[String],
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &ExpenseConfig,
        currency: &str,
    ) -> Vec<ExpenseReport> {
        debug!(employee_count = employee_ids.len(), %period_start, %period_end, currency, "Generating expense reports");
        let mut reports = Vec::new();

        // Iterate over each month in the period
        let mut current_month_start = period_start;
        while current_month_start <= period_end {
            let month_end = self.month_end(current_month_start).min(period_end);

            for employee_id in employee_ids {
                // Only submission_rate fraction of employees submit per month
                if self.rng.random_bool(config.submission_rate.min(1.0)) {
                    let report = self.generate_report(
                        employee_id,
                        current_month_start,
                        month_end,
                        config,
                        currency,
                    );
                    reports.push(report);
                }
            }

            // Advance to next month
            current_month_start = self.next_month_start(current_month_start);
        }

        reports
    }

    /// Generate a single expense report for an employee within a date range.
    fn generate_report(
        &mut self,
        employee_id: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &ExpenseConfig,
        currency: &str,
    ) -> ExpenseReport {
        let report_id = self.uuid_factory.next().to_string();

        // 1-5 line items per report
        let item_count = self.rng.random_range(1..=5);
        let mut line_items = Vec::with_capacity(item_count);
        let mut total_amount = Decimal::ZERO;

        for _ in 0..item_count {
            let item = self.generate_line_item(period_start, period_end, currency);
            total_amount += item.amount;
            line_items.push(item);
        }

        // Submission date: usually within a few days after the last expense
        let max_expense_date = line_items
            .iter()
            .map(|li| li.date)
            .max()
            .unwrap_or(period_end);
        let submission_lag = self.rng.random_range(0..=5);
        let submission_date = max_expense_date + chrono::Duration::days(submission_lag);

        // Trip/purpose descriptions
        let descriptions = [
            "Client site visit",
            "Conference attendance",
            "Team offsite meeting",
            "Customer presentation",
            "Training workshop",
            "Quarterly review travel",
            "Sales meeting",
            "Project kickoff",
        ];
        let description = descriptions[self.rng.random_range(0..descriptions.len())].to_string();

        // Status distribution: 70% Approved, 10% Paid, 10% Submitted, 5% Rejected, 5% Draft
        let status_roll: f64 = self.rng.random();
        let status = if status_roll < 0.70 {
            ExpenseStatus::Approved
        } else if status_roll < 0.80 {
            ExpenseStatus::Paid
        } else if status_roll < 0.90 {
            ExpenseStatus::Submitted
        } else if status_roll < 0.95 {
            ExpenseStatus::Rejected
        } else {
            ExpenseStatus::Draft
        };

        let approved_by = if matches!(status, ExpenseStatus::Approved | ExpenseStatus::Paid) {
            if !self.employee_ids_pool.is_empty() {
                let idx = self.rng.random_range(0..self.employee_ids_pool.len());
                Some(self.employee_ids_pool[idx].clone())
            } else {
                Some(format!("MGR-{:04}", self.rng.random_range(1..=100)))
            }
        } else {
            None
        };

        let approved_date = if matches!(status, ExpenseStatus::Approved | ExpenseStatus::Paid) {
            let approval_lag = self.rng.random_range(1..=7);
            Some(submission_date + chrono::Duration::days(approval_lag))
        } else {
            None
        };

        let paid_date = if status == ExpenseStatus::Paid {
            approved_date.map(|ad| ad + chrono::Duration::days(self.rng.random_range(3..=14)))
        } else {
            None
        };

        // Cost center and department
        let cost_center = if self.rng.random_bool(0.70) {
            if !self.cost_center_ids_pool.is_empty() {
                let idx = self.rng.random_range(0..self.cost_center_ids_pool.len());
                Some(self.cost_center_ids_pool[idx].clone())
            } else {
                Some(format!("CC-{:03}", self.rng.random_range(100..=500)))
            }
        } else {
            None
        };

        let department = if self.rng.random_bool(0.80) {
            let departments = [
                "Engineering",
                "Sales",
                "Marketing",
                "Finance",
                "HR",
                "Operations",
                "Legal",
                "IT",
                "Executive",
            ];
            Some(departments[self.rng.random_range(0..departments.len())].to_string())
        } else {
            None
        };

        // Policy violations: based on config.policy_violation_rate per line item
        let policy_violation_rate = config.policy_violation_rate;
        let mut policy_violations = Vec::new();
        for item in &line_items {
            if self.rng.random_bool(policy_violation_rate.min(1.0)) {
                let violation = self.pick_violation(item);
                policy_violations.push(violation);
            }
        }

        ExpenseReport {
            report_id,
            employee_id: employee_id.to_string(),
            submission_date,
            description,
            status,
            total_amount,
            currency: currency.to_string(),
            line_items,
            approved_by,
            approved_date,
            paid_date,
            cost_center,
            department,
            policy_violations,
            employee_name: self.employee_names.get(employee_id).cloned(),
        }
    }

    /// Generate a single expense line item with a random category and amount.
    fn generate_line_item(
        &mut self,
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
    ) -> ExpenseLineItem {
        let item_id = self.item_uuid_factory.next().to_string();

        // Pick a category and generate an appropriate amount range
        let (category, amount_min, amount_max, desc, merchant) = self.pick_category();

        let amount = sample_decimal_range(
            &mut self.rng,
            Decimal::from_f64_retain(amount_min).unwrap_or(Decimal::ONE),
            Decimal::from_f64_retain(amount_max).unwrap_or(Decimal::ONE),
        )
        .round_dp(2);

        // Date within the period
        let days_in_period = (period_end - period_start).num_days().max(1);
        let offset = self.rng.random_range(0..=days_in_period);
        let date = period_start + chrono::Duration::days(offset);

        // Receipt attached: 85% of the time
        let receipt_attached = self.rng.random_bool(0.85);

        ExpenseLineItem {
            item_id,
            category,
            date,
            amount,
            currency: currency.to_string(),
            description: desc,
            receipt_attached,
            merchant,
        }
    }

    /// Pick an expense category with corresponding amount range, description, and merchant.
    fn pick_category(&mut self) -> (ExpenseCategory, f64, f64, String, Option<String>) {
        let roll: f64 = self.rng.random();

        if roll < 0.20 {
            let merchants = [
                "Delta Airlines",
                "United Airlines",
                "American Airlines",
                "Southwest",
            ];
            let merchant = merchants[self.rng.random_range(0..merchants.len())].to_string();
            (
                ExpenseCategory::Travel,
                200.0,
                2000.0,
                "Airfare - business travel".to_string(),
                Some(merchant),
            )
        } else if roll < 0.40 {
            let merchants = [
                "Restaurant ABC",
                "Cafe Express",
                "Business Lunch Co",
                "Steakhouse Prime",
                "Sushi Palace",
            ];
            let merchant = merchants[self.rng.random_range(0..merchants.len())].to_string();
            (
                ExpenseCategory::Meals,
                20.0,
                100.0,
                "Business meal".to_string(),
                Some(merchant),
            )
        } else if roll < 0.55 {
            let merchants = ["Marriott", "Hilton", "Hyatt", "Holiday Inn", "Best Western"];
            let merchant = merchants[self.rng.random_range(0..merchants.len())].to_string();
            (
                ExpenseCategory::Lodging,
                100.0,
                500.0,
                "Hotel accommodation".to_string(),
                Some(merchant),
            )
        } else if roll < 0.70 {
            let merchants = ["Uber", "Lyft", "Hertz", "Enterprise", "Airport Parking"];
            let merchant = merchants[self.rng.random_range(0..merchants.len())].to_string();
            (
                ExpenseCategory::Transportation,
                10.0,
                200.0,
                "Ground transportation".to_string(),
                Some(merchant),
            )
        } else if roll < 0.80 {
            (
                ExpenseCategory::Office,
                15.0,
                300.0,
                "Office supplies".to_string(),
                Some("Office Depot".to_string()),
            )
        } else if roll < 0.88 {
            (
                ExpenseCategory::Entertainment,
                50.0,
                500.0,
                "Client entertainment".to_string(),
                None,
            )
        } else if roll < 0.95 {
            (
                ExpenseCategory::Training,
                100.0,
                1500.0,
                "Professional development".to_string(),
                None,
            )
        } else {
            (
                ExpenseCategory::Other,
                10.0,
                200.0,
                "Miscellaneous expense".to_string(),
                None,
            )
        }
    }

    /// Generate a policy violation description for a given line item.
    fn pick_violation(&mut self, item: &ExpenseLineItem) -> String {
        let violations = match item.category {
            ExpenseCategory::Meals => vec![
                "Exceeds daily meal limit",
                "Alcohol included without approval",
                "Missing itemized receipt",
            ],
            ExpenseCategory::Travel => vec![
                "Booked outside preferred vendor",
                "Class upgrade not pre-approved",
                "Booking made less than 7 days in advance",
            ],
            ExpenseCategory::Lodging => vec![
                "Exceeds nightly rate limit",
                "Extended stay without approval",
                "Non-preferred hotel chain",
            ],
            _ => vec![
                "Missing receipt",
                "Insufficient business justification",
                "Exceeds category spending limit",
            ],
        };

        violations[self.rng.random_range(0..violations.len())].to_string()
    }

    /// Get the last day of the month for a given date.
    fn month_end(&self, date: NaiveDate) -> NaiveDate {
        let (year, month) = if date.month() == 12 {
            (date.year() + 1, 1)
        } else {
            (date.year(), date.month() + 1)
        };
        NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap_or(date)
            .pred_opt()
            .unwrap_or(date)
    }

    /// Get the first day of the next month.
    fn next_month_start(&self, date: NaiveDate) -> NaiveDate {
        let (year, month) = if date.month() == 12 {
            (date.year() + 1, 1)
        } else {
            (date.year(), date.month() + 1)
        };
        NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_employee_ids() -> Vec<String> {
        (1..=10).map(|i| format!("EMP-{:04}", i)).collect()
    }

    #[test]
    fn test_basic_expense_generation() {
        let mut gen = ExpenseReportGenerator::new(42);
        let employees = test_employee_ids();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let config = ExpenseConfig::default();

        let reports = gen.generate(&employees, period_start, period_end, &config);

        // With 30% submission rate and 10 employees, expect ~3 reports per month
        assert!(!reports.is_empty());
        assert!(
            reports.len() <= employees.len(),
            "Should not exceed employee count for a single month"
        );

        for report in &reports {
            assert!(!report.report_id.is_empty());
            assert!(!report.employee_id.is_empty());
            assert!(report.total_amount > Decimal::ZERO);
            assert!(!report.line_items.is_empty());
            assert!(report.line_items.len() <= 5);

            // Total should equal sum of line items
            let line_sum: Decimal = report.line_items.iter().map(|li| li.amount).sum();
            assert_eq!(report.total_amount, line_sum);

            for item in &report.line_items {
                assert!(!item.item_id.is_empty());
                assert!(item.amount > Decimal::ZERO);
            }
        }
    }

    #[test]
    fn test_deterministic_expenses() {
        let employees = test_employee_ids();
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let config = ExpenseConfig::default();

        let mut gen1 = ExpenseReportGenerator::new(42);
        let reports1 = gen1.generate(&employees, period_start, period_end, &config);

        let mut gen2 = ExpenseReportGenerator::new(42);
        let reports2 = gen2.generate(&employees, period_start, period_end, &config);

        assert_eq!(reports1.len(), reports2.len());
        for (a, b) in reports1.iter().zip(reports2.iter()) {
            assert_eq!(a.report_id, b.report_id);
            assert_eq!(a.employee_id, b.employee_id);
            assert_eq!(a.total_amount, b.total_amount);
            assert_eq!(a.status, b.status);
            assert_eq!(a.line_items.len(), b.line_items.len());
        }
    }

    #[test]
    fn test_expense_status_and_violations() {
        let mut gen = ExpenseReportGenerator::new(99);
        // Use more employees for a broader sample
        let employees: Vec<String> = (1..=30).map(|i| format!("EMP-{:04}", i)).collect();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let config = ExpenseConfig::default();

        let reports = gen.generate(&employees, period_start, period_end, &config);

        // With 30 employees over 6 months, we should have a decent sample
        assert!(
            reports.len() > 10,
            "Expected multiple reports, got {}",
            reports.len()
        );

        let approved = reports
            .iter()
            .filter(|r| r.status == ExpenseStatus::Approved)
            .count();
        let paid = reports
            .iter()
            .filter(|r| r.status == ExpenseStatus::Paid)
            .count();
        let submitted = reports
            .iter()
            .filter(|r| r.status == ExpenseStatus::Submitted)
            .count();
        let rejected = reports
            .iter()
            .filter(|r| r.status == ExpenseStatus::Rejected)
            .count();
        let draft = reports
            .iter()
            .filter(|r| r.status == ExpenseStatus::Draft)
            .count();

        // Approved should be the majority
        assert!(approved > 0, "Expected at least some approved reports");
        // Check that we have a mix of statuses
        assert!(
            paid + submitted + rejected + draft > 0,
            "Expected a mix of statuses beyond approved"
        );

        // Check policy violations exist somewhere
        let total_violations: usize = reports.iter().map(|r| r.policy_violations.len()).sum();
        assert!(
            total_violations > 0,
            "Expected at least some policy violations across {} reports",
            reports.len()
        );
    }
}
