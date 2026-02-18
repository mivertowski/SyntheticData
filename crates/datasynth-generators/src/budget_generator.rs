//! Budget generator.
//!
//! Generates realistic budgets with line items for each GL account,
//! budget-vs-actual variance analysis, and approval workflows.

use chrono::NaiveDate;
use datasynth_config::schema::BudgetConfig;
use datasynth_core::models::{Budget, BudgetLineItem, BudgetStatus};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Departments cycled through for budget line items.
const DEPARTMENTS: &[&str] = &["Finance", "Sales", "Engineering", "Operations", "HR"];

/// Cost center codes corresponding to departments.
const COST_CENTERS: &[&str] = &["CC-100", "CC-200", "CC-300", "CC-400", "CC-500"];

/// Generates [`Budget`] instances with line items, variance analysis,
/// and realistic approval workflows.
pub struct BudgetGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    line_uuid_factory: DeterministicUuidFactory,
}

impl BudgetGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BudgetLine),
            line_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::BudgetLine,
                1,
            ),
        }
    }

    /// Generate a budget for the given fiscal year and accounts.
    ///
    /// # Arguments
    ///
    /// * `company_code` - The company code this budget belongs to.
    /// * `fiscal_year` - The fiscal year the budget covers.
    /// * `account_codes` - Slice of (account_code, account_name) tuples.
    /// * `config` - Budget configuration knobs.
    pub fn generate(
        &mut self,
        company_code: &str,
        fiscal_year: u32,
        account_codes: &[(String, String)],
        config: &BudgetConfig,
    ) -> Budget {
        let budget_id = self.uuid_factory.next().to_string();

        let mut line_items = Vec::new();
        let mut total_budget = Decimal::ZERO;
        let mut total_actual = Decimal::ZERO;

        for (idx, (account_code, account_name)) in account_codes.iter().enumerate() {
            // Cycle through departments
            let dept_idx = idx % DEPARTMENTS.len();
            let department = DEPARTMENTS[dept_idx];
            let cost_center = COST_CENTERS[dept_idx];

            // Generate monthly line items for the fiscal year (Jan-Dec)
            for month in 1..=12u32 {
                let line = self.generate_line_item(
                    &budget_id,
                    account_code,
                    account_name,
                    department,
                    cost_center,
                    fiscal_year,
                    month,
                    config,
                );
                total_budget += line.budget_amount;
                total_actual += line.actual_amount;
                line_items.push(line);
            }
        }

        let total_variance = total_actual - total_budget;

        // Status: 60% Approved, 20% Closed, 15% Revised, 5% Submitted
        let status_roll: f64 = self.rng.gen();
        let status = if status_roll < 0.60 {
            BudgetStatus::Approved
        } else if status_roll < 0.80 {
            BudgetStatus::Closed
        } else if status_roll < 0.95 {
            BudgetStatus::Revised
        } else {
            BudgetStatus::Submitted
        };

        // Approved/Closed budgets get an approver
        let (approved_by, approved_date) =
            if matches!(status, BudgetStatus::Approved | BudgetStatus::Closed) {
                let approver = if self.rng.gen_bool(0.5) {
                    "CFO-001".to_string()
                } else {
                    "VP-FIN-001".to_string()
                };
                // Approved before the fiscal year starts
                let approve_date =
                    NaiveDate::from_ymd_opt(fiscal_year.saturating_sub(1) as i32, 12, 15)
                        .or_else(|| NaiveDate::from_ymd_opt(fiscal_year as i32, 1, 1));
                (Some(approver), approve_date)
            } else {
                (None, None)
            };

        Budget {
            budget_id,
            company_code: company_code.to_string(),
            fiscal_year,
            name: format!("FY{} Operating Budget", fiscal_year),
            status,
            total_budget: total_budget.round_dp(2),
            total_actual: total_actual.round_dp(2),
            total_variance: total_variance.round_dp(2),
            line_items,
            approved_by,
            approved_date,
        }
    }

    /// Generate a single budget line item for an account/month.
    #[allow(clippy::too_many_arguments)]
    fn generate_line_item(
        &mut self,
        budget_id: &str,
        account_code: &str,
        account_name: &str,
        department: &str,
        cost_center: &str,
        fiscal_year: u32,
        month: u32,
        config: &BudgetConfig,
    ) -> BudgetLineItem {
        let line_id = self.line_uuid_factory.next().to_string();

        // Budget amount: random 10000 - 500000, applying revenue growth rate
        let base_amount: f64 = self.rng.gen_range(10_000.0..500_000.0);
        let growth_adjusted = base_amount * (1.0 + config.revenue_growth_rate);
        let budget_amount = Decimal::from_f64_retain(growth_adjusted)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2);

        // Actual amount: budget * (1 + random(-variance_noise, +variance_noise))
        let variance_factor: f64 = self
            .rng
            .gen_range(-config.variance_noise..config.variance_noise);
        let actual_raw = growth_adjusted * (1.0 + variance_factor);
        let actual_amount = Decimal::from_f64_retain(actual_raw)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2);

        // Variance = actual - budget
        let variance = actual_amount - budget_amount;

        // Variance percent = variance / budget * 100
        let variance_percent = if budget_amount != Decimal::ZERO {
            let pct = variance.to_string().parse::<f64>().unwrap_or(0.0) / growth_adjusted * 100.0;
            (pct * 100.0).round() / 100.0
        } else {
            0.0
        };

        // Period dates for this month
        let period_start =
            NaiveDate::from_ymd_opt(fiscal_year as i32, month, 1).unwrap_or_else(|| {
                NaiveDate::from_ymd_opt(fiscal_year as i32, 1, 1)
                    .unwrap_or(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap_or_default())
            });

        let period_end = {
            let (next_year, next_month) = if month == 12 {
                (fiscal_year as i32 + 1, 1)
            } else {
                (fiscal_year as i32, month + 1)
            };
            NaiveDate::from_ymd_opt(next_year, next_month, 1)
                .and_then(|d| d.pred_opt())
                .unwrap_or(period_start)
        };

        // Add a note for large variances
        let notes = if variance_percent.abs() > 5.0 {
            Some(format!(
                "Variance of {:.1}% requires management review",
                variance_percent
            ))
        } else {
            None
        };

        BudgetLineItem {
            line_id,
            budget_id: budget_id.to_string(),
            account_code: account_code.to_string(),
            account_name: account_name.to_string(),
            department: Some(department.to_string()),
            cost_center: Some(cost_center.to_string()),
            budget_amount,
            actual_amount,
            variance,
            variance_percent,
            period_start,
            period_end,
            notes,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_accounts() -> Vec<(String, String)> {
        vec![
            ("4000".to_string(), "Revenue".to_string()),
            ("5000".to_string(), "Cost of Goods Sold".to_string()),
            ("6100".to_string(), "Salaries Expense".to_string()),
            ("6200".to_string(), "Rent Expense".to_string()),
            ("6300".to_string(), "Utilities Expense".to_string()),
        ]
    }

    fn default_config() -> BudgetConfig {
        BudgetConfig {
            enabled: true,
            revenue_growth_rate: 0.05,
            expense_inflation_rate: 0.03,
            variance_noise: 0.10,
        }
    }

    #[test]
    fn test_basic_generation_produces_expected_structure() {
        let mut gen = BudgetGenerator::new(42);
        let accounts = sample_accounts();
        let config = default_config();

        let budget = gen.generate("C001", 2024, &accounts, &config);

        // Basic field checks
        assert!(!budget.budget_id.is_empty());
        assert_eq!(budget.company_code, "C001");
        assert_eq!(budget.fiscal_year, 2024);
        assert_eq!(budget.name, "FY2024 Operating Budget");

        // 5 accounts * 12 months = 60 line items
        assert_eq!(budget.line_items.len(), 60);

        // Totals should be consistent with line items
        let sum_budget: Decimal = budget.line_items.iter().map(|l| l.budget_amount).sum();
        let sum_actual: Decimal = budget.line_items.iter().map(|l| l.actual_amount).sum();
        assert_eq!(budget.total_budget, sum_budget.round_dp(2));
        assert_eq!(budget.total_actual, sum_actual.round_dp(2));

        // Variance = actual - budget
        let expected_variance = budget.total_actual - budget.total_budget;
        assert_eq!(budget.total_variance, expected_variance);

        // All line items should have departments and cost centers
        for line in &budget.line_items {
            assert!(line.department.is_some());
            assert!(line.cost_center.is_some());
            assert!(line.budget_amount > Decimal::ZERO);
            assert!(line.actual_amount > Decimal::ZERO);
        }

        // Approved/Closed should have approver, Submitted should not
        if matches!(budget.status, BudgetStatus::Approved | BudgetStatus::Closed) {
            assert!(budget.approved_by.is_some());
            assert!(budget.approved_date.is_some());
        }
    }

    #[test]
    fn test_deterministic_output_with_same_seed() {
        let accounts = sample_accounts();
        let config = default_config();

        let mut gen1 = BudgetGenerator::new(12345);
        let budget1 = gen1.generate("C001", 2025, &accounts, &config);

        let mut gen2 = BudgetGenerator::new(12345);
        let budget2 = gen2.generate("C001", 2025, &accounts, &config);

        assert_eq!(budget1.budget_id, budget2.budget_id);
        assert_eq!(budget1.total_budget, budget2.total_budget);
        assert_eq!(budget1.total_actual, budget2.total_actual);
        assert_eq!(budget1.line_items.len(), budget2.line_items.len());

        for (l1, l2) in budget1.line_items.iter().zip(budget2.line_items.iter()) {
            assert_eq!(l1.line_id, l2.line_id);
            assert_eq!(l1.budget_amount, l2.budget_amount);
            assert_eq!(l1.actual_amount, l2.actual_amount);
            assert_eq!(l1.variance, l2.variance);
        }
    }

    #[test]
    fn test_variance_within_noise_bounds() {
        let mut gen = BudgetGenerator::new(777);
        let accounts = sample_accounts();
        let config = BudgetConfig {
            enabled: true,
            revenue_growth_rate: 0.0,
            expense_inflation_rate: 0.0,
            variance_noise: 0.10,
        };

        let budget = gen.generate("C002", 2024, &accounts, &config);

        // Each line item's variance should be within +-10% of budget
        for line in &budget.line_items {
            let ratio = if line.budget_amount != Decimal::ZERO {
                (line.actual_amount - line.budget_amount).abs() / line.budget_amount
            } else {
                Decimal::ZERO
            };
            // Allow small rounding slack
            assert!(
                ratio <= Decimal::from_f64_retain(0.11).unwrap_or(Decimal::ONE),
                "Variance ratio {} exceeds noise bound for account {}",
                ratio,
                line.account_code
            );
        }
    }
}
