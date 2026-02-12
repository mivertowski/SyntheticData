//! Payroll generator for the Hire-to-Retire (H2R) process.
//!
//! Generates payroll runs with individual employee line items, computing
//! gross pay (base salary + overtime + bonus), deductions (tax, social security,
//! health insurance, retirement), and net pay.

use chrono::NaiveDate;
use datasynth_config::schema::PayrollConfig;
use datasynth_core::models::{PayrollLineItem, PayrollRun, PayrollRunStatus};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates [`PayrollRun`] and [`PayrollLineItem`] records from employee data.
pub struct PayrollGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    line_uuid_factory: DeterministicUuidFactory,
    config: PayrollConfig,
}

impl PayrollGenerator {
    /// Create a new payroll generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::PayrollRun),
            line_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::PayrollRun,
                1,
            ),
            config: PayrollConfig::default(),
        }
    }

    /// Create a payroll generator with custom configuration.
    pub fn with_config(seed: u64, config: PayrollConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::PayrollRun),
            line_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::PayrollRun,
                1,
            ),
            config,
        }
    }

    /// Generate a payroll run and line items for the given employees and period.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company code owning the payroll
    /// * `employees` - Slice of (employee_id, base_salary, cost_center, department)
    /// * `period_start` - Start of the pay period (inclusive)
    /// * `period_end` - End of the pay period (inclusive)
    /// * `currency` - ISO 4217 currency code
    pub fn generate(
        &mut self,
        company_code: &str,
        employees: &[(String, Decimal, Option<String>, Option<String>)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
    ) -> (PayrollRun, Vec<PayrollLineItem>) {
        let payroll_id = self.uuid_factory.next().to_string();

        let mut line_items = Vec::with_capacity(employees.len());
        let mut total_gross = Decimal::ZERO;
        let mut total_deductions = Decimal::ZERO;
        let mut total_net = Decimal::ZERO;
        let mut total_employer_cost = Decimal::ZERO;

        // Tax rates from config
        let federal_rate = Decimal::from_f64_retain(self.config.tax_rates.federal_effective)
            .unwrap_or(Decimal::ZERO);
        let state_rate = Decimal::from_f64_retain(self.config.tax_rates.state_effective)
            .unwrap_or(Decimal::ZERO);
        let fica_rate =
            Decimal::from_f64_retain(self.config.tax_rates.fica).unwrap_or(Decimal::ZERO);

        // Combined income tax rate (federal + state)
        let income_tax_rate = federal_rate + state_rate;

        // Standard deduction rates for health and retirement
        let health_rate = Decimal::from_f64_retain(0.03).unwrap_or(Decimal::ZERO);
        let retirement_rate = Decimal::from_f64_retain(0.05).unwrap_or(Decimal::ZERO);

        let benefits_enrolled = self.config.benefits_enrollment_rate;
        let retirement_participating = self.config.retirement_participation_rate;

        for (employee_id, base_salary, cost_center, department) in employees {
            let line_id = self.line_uuid_factory.next().to_string();

            // Monthly base component (annual salary / 12)
            let monthly_base = (*base_salary / Decimal::from(12)).round_dp(2);

            // Overtime: 10% chance, 1-20 hours at 1.5x hourly rate
            let (overtime_pay, overtime_hours) = if self.rng.gen_bool(0.10) {
                let ot_hours = self.rng.gen_range(1.0..=20.0);
                // Hourly rate = annual salary / (52 weeks * 40 hours)
                let hourly_rate = *base_salary / Decimal::from(2080);
                let ot_rate = hourly_rate * Decimal::from_f64_retain(1.5).unwrap_or(Decimal::ONE);
                let ot_pay = (ot_rate
                    * Decimal::from_f64_retain(ot_hours).unwrap_or(Decimal::ZERO))
                .round_dp(2);
                (ot_pay, ot_hours)
            } else {
                (Decimal::ZERO, 0.0)
            };

            // Bonus: 5% chance for a monthly bonus (1-10% of monthly base)
            let bonus = if self.rng.gen_bool(0.05) {
                let pct = self.rng.gen_range(0.01..=0.10);
                (monthly_base * Decimal::from_f64_retain(pct).unwrap_or(Decimal::ZERO)).round_dp(2)
            } else {
                Decimal::ZERO
            };

            let gross_pay = monthly_base + overtime_pay + bonus;

            // Deductions
            let tax_withholding = (gross_pay * income_tax_rate).round_dp(2);
            let social_security = (gross_pay * fica_rate).round_dp(2);

            let health_insurance = if self.rng.gen_bool(benefits_enrolled) {
                (gross_pay * health_rate).round_dp(2)
            } else {
                Decimal::ZERO
            };

            let retirement_contribution = if self.rng.gen_bool(retirement_participating) {
                (gross_pay * retirement_rate).round_dp(2)
            } else {
                Decimal::ZERO
            };

            // Small random other deductions (garnishments, etc.): ~3% chance
            let other_deductions = if self.rng.gen_bool(0.03) {
                let raw = self.rng.gen_range(50.0..=500.0);
                Decimal::from_f64_retain(raw)
                    .unwrap_or(Decimal::ZERO)
                    .round_dp(2)
            } else {
                Decimal::ZERO
            };

            let total_ded = tax_withholding
                + social_security
                + health_insurance
                + retirement_contribution
                + other_deductions;
            let net_pay = gross_pay - total_ded;

            // Standard 160 regular hours per month (8h * 20 business days)
            let hours_worked = 160.0;

            // Employer-side cost: gross + employer FICA match
            let employer_fica = (gross_pay * fica_rate).round_dp(2);
            let employer_cost = gross_pay + employer_fica;

            total_gross += gross_pay;
            total_deductions += total_ded;
            total_net += net_pay;
            total_employer_cost += employer_cost;

            line_items.push(PayrollLineItem {
                payroll_id: payroll_id.clone(),
                employee_id: employee_id.clone(),
                line_id,
                gross_pay,
                base_salary: monthly_base,
                overtime_pay,
                bonus,
                tax_withholding,
                social_security,
                health_insurance,
                retirement_contribution,
                other_deductions,
                net_pay,
                hours_worked,
                overtime_hours,
                pay_date: period_end,
                cost_center: cost_center.clone(),
                department: department.clone(),
            });
        }

        // Determine status
        let status_roll: f64 = self.rng.gen();
        let status = if status_roll < 0.60 {
            PayrollRunStatus::Posted
        } else if status_roll < 0.85 {
            PayrollRunStatus::Approved
        } else if status_roll < 0.95 {
            PayrollRunStatus::Calculated
        } else {
            PayrollRunStatus::Draft
        };

        let approved_by = if matches!(
            status,
            PayrollRunStatus::Approved | PayrollRunStatus::Posted
        ) {
            Some(format!("USR-{:04}", self.rng.gen_range(201..=400)))
        } else {
            None
        };

        let posted_by = if status == PayrollRunStatus::Posted {
            Some(format!("USR-{:04}", self.rng.gen_range(401..=500)))
        } else {
            None
        };

        let run = PayrollRun {
            company_code: company_code.to_string(),
            payroll_id: payroll_id.clone(),
            pay_period_start: period_start,
            pay_period_end: period_end,
            run_date: period_end,
            status,
            total_gross,
            total_deductions,
            total_net,
            total_employer_cost,
            employee_count: employees.len() as u32,
            currency: currency.to_string(),
            posted_by,
            approved_by,
        };

        (run, line_items)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_employees() -> Vec<(String, Decimal, Option<String>, Option<String>)> {
        vec![
            (
                "EMP-001".to_string(),
                Decimal::from(60_000),
                Some("CC-100".to_string()),
                Some("Engineering".to_string()),
            ),
            (
                "EMP-002".to_string(),
                Decimal::from(85_000),
                Some("CC-200".to_string()),
                Some("Finance".to_string()),
            ),
            (
                "EMP-003".to_string(),
                Decimal::from(120_000),
                None,
                Some("Sales".to_string()),
            ),
        ]
    }

    #[test]
    fn test_basic_payroll_generation() {
        let mut gen = PayrollGenerator::new(42);
        let employees = test_employees();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let (run, items) = gen.generate("C001", &employees, period_start, period_end, "USD");

        assert_eq!(run.company_code, "C001");
        assert_eq!(run.currency, "USD");
        assert_eq!(run.employee_count, 3);
        assert_eq!(items.len(), 3);
        assert!(run.total_gross > Decimal::ZERO);
        assert!(run.total_deductions > Decimal::ZERO);
        assert!(run.total_net > Decimal::ZERO);
        assert!(run.total_employer_cost > run.total_gross);
        // net = gross - deductions
        assert_eq!(run.total_net, run.total_gross - run.total_deductions);

        for item in &items {
            assert_eq!(item.payroll_id, run.payroll_id);
            assert!(item.gross_pay > Decimal::ZERO);
            assert!(item.net_pay > Decimal::ZERO);
            assert!(item.net_pay < item.gross_pay);
            assert!(item.base_salary > Decimal::ZERO);
            assert_eq!(item.pay_date, period_end);
        }
    }

    #[test]
    fn test_deterministic_payroll() {
        let employees = test_employees();
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let mut gen1 = PayrollGenerator::new(42);
        let (run1, items1) = gen1.generate("C001", &employees, period_start, period_end, "USD");

        let mut gen2 = PayrollGenerator::new(42);
        let (run2, items2) = gen2.generate("C001", &employees, period_start, period_end, "USD");

        assert_eq!(run1.payroll_id, run2.payroll_id);
        assert_eq!(run1.total_gross, run2.total_gross);
        assert_eq!(run1.total_net, run2.total_net);
        assert_eq!(run1.status, run2.status);
        assert_eq!(items1.len(), items2.len());
        for (a, b) in items1.iter().zip(items2.iter()) {
            assert_eq!(a.line_id, b.line_id);
            assert_eq!(a.gross_pay, b.gross_pay);
            assert_eq!(a.net_pay, b.net_pay);
        }
    }

    #[test]
    fn test_payroll_deduction_components() {
        let mut gen = PayrollGenerator::new(99);
        let employees = vec![(
            "EMP-010".to_string(),
            Decimal::from(100_000),
            Some("CC-300".to_string()),
            Some("HR".to_string()),
        )];
        let period_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let (_run, items) = gen.generate("C001", &employees, period_start, period_end, "USD");
        assert_eq!(items.len(), 1);

        let item = &items[0];
        // base_salary should be approximately 100000/12 = 8333.33
        let expected_monthly = (Decimal::from(100_000) / Decimal::from(12)).round_dp(2);
        assert_eq!(item.base_salary, expected_monthly);

        // Deductions should sum correctly
        let deduction_sum = item.tax_withholding
            + item.social_security
            + item.health_insurance
            + item.retirement_contribution
            + item.other_deductions;
        let expected_net = item.gross_pay - deduction_sum;
        assert_eq!(item.net_pay, expected_net);

        // Tax withholding should be reasonable (22% federal + 5% state = 27% of gross)
        assert!(item.tax_withholding > Decimal::ZERO);
        assert!(item.social_security > Decimal::ZERO);
    }
}
