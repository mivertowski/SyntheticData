//! Payroll generator for the Hire-to-Retire (H2R) process.
//!
//! Generates payroll runs with individual employee line items, computing
//! gross pay (base salary + overtime + bonus), deductions (tax, social security,
//! health insurance, retirement), and net pay.

use chrono::NaiveDate;
use datasynth_config::schema::PayrollConfig;
use datasynth_core::models::{PayrollLineItem, PayrollRun, PayrollRunStatus};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use datasynth_core::CountryPack;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Resolved payroll deduction rates used during generation.
#[derive(Debug, Clone)]
struct PayrollRates {
    /// Combined income tax rate (federal + state, or equivalent).
    income_tax_rate: Decimal,
    /// Social security / FICA rate.
    fica_rate: Decimal,
    /// Employee health insurance rate.
    health_rate: Decimal,
    /// Employee retirement / pension rate.
    retirement_rate: Decimal,
    /// Employer-side social security matching rate.
    employer_fica_rate: Decimal,
}

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
    /// Uses tax rates from the [`PayrollConfig`] (defaults: 22% federal, 5% state,
    /// 7.65% FICA, 3% health, 5% retirement).
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
        let rates = self.rates_from_config();
        self.generate_with_rates(company_code, employees, period_start, period_end, currency, &rates)
    }

    /// Generate a payroll run using statutory deduction rates from a country pack.
    ///
    /// Iterates over `pack.payroll.statutory_deductions` to resolve rates by
    /// deduction code / English name.  Any rate not found in the pack falls back
    /// to the corresponding value from the generator's [`PayrollConfig`].
    ///
    /// # Deduction mapping
    ///
    /// | Pack code / `name_en` pattern              | Resolves to         |
    /// |--------------------------------------------|---------------------|
    /// | `FIT`, `LOHNST`, or `*Income Tax*` (not state) | federal income tax  |
    /// | `SIT` or `*State Income Tax*`              | state income tax    |
    /// | `FICA` or `*Social Security*`              | FICA / social security |
    /// | `*Health Insurance*`                       | health insurance    |
    /// | `*Pension*` or `*Retirement*`              | retirement / pension |
    ///
    /// For packs that have many small deductions (e.g. DE with pension, health,
    /// unemployment, long-term care, solidarity surcharge, church tax), the rates
    /// are summed into the closest category. Deductions not matching any category
    /// above are accumulated into the FICA/social-security bucket.
    pub fn generate_with_country_pack(
        &mut self,
        company_code: &str,
        employees: &[(String, Decimal, Option<String>, Option<String>)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
        pack: &CountryPack,
    ) -> (PayrollRun, Vec<PayrollLineItem>) {
        let rates = self.rates_from_country_pack(pack);
        self.generate_with_rates(company_code, employees, period_start, period_end, currency, &rates)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Build [`PayrollRates`] from the generator's config (original behaviour).
    fn rates_from_config(&self) -> PayrollRates {
        let federal_rate = Decimal::from_f64_retain(self.config.tax_rates.federal_effective)
            .unwrap_or(Decimal::ZERO);
        let state_rate = Decimal::from_f64_retain(self.config.tax_rates.state_effective)
            .unwrap_or(Decimal::ZERO);
        let fica_rate =
            Decimal::from_f64_retain(self.config.tax_rates.fica).unwrap_or(Decimal::ZERO);

        PayrollRates {
            income_tax_rate: federal_rate + state_rate,
            fica_rate,
            health_rate: Decimal::from_f64_retain(0.03).unwrap_or(Decimal::ZERO),
            retirement_rate: Decimal::from_f64_retain(0.05).unwrap_or(Decimal::ZERO),
            employer_fica_rate: fica_rate,
        }
    }

    /// Build [`PayrollRates`] from a [`CountryPack`], falling back to config
    /// values for any category not found.
    fn rates_from_country_pack(&self, pack: &CountryPack) -> PayrollRates {
        let fallback = self.rates_from_config();

        // Accumulators – start at zero; we only use the fallback when a
        // category has *no* matching deduction in the pack at all.
        let mut federal_tax = Decimal::ZERO;
        let mut state_tax = Decimal::ZERO;
        let mut fica = Decimal::ZERO;
        let mut health = Decimal::ZERO;
        let mut retirement = Decimal::ZERO;

        // Track which categories were populated from the pack.
        let mut found_federal = false;
        let mut found_state = false;
        let mut found_fica = false;
        let mut found_health = false;
        let mut found_retirement = false;

        for ded in &pack.payroll.statutory_deductions {
            let code_upper = ded.code.to_uppercase();
            let name_en_lower = ded.name_en.to_lowercase();
            let rate = Decimal::from_f64_retain(ded.rate).unwrap_or(Decimal::ZERO);

            // Skip progressive (bracket-based) income taxes that have rate 0.0
            // in the pack — these are placeholders indicating bracket lookup is
            // needed. We will fall back to the config's effective rate instead.
            if (ded.deduction_type == "progressive" || ded.type_field == "progressive")
                && ded.rate == 0.0
            {
                continue;
            }

            if code_upper == "FIT"
                || code_upper == "LOHNST"
                || (name_en_lower.contains("income tax")
                    && !name_en_lower.contains("state"))
            {
                federal_tax += rate;
                found_federal = true;
            } else if code_upper == "SIT"
                || name_en_lower.contains("state income tax")
            {
                state_tax += rate;
                found_state = true;
            } else if code_upper == "FICA"
                || name_en_lower.contains("social security")
            {
                fica += rate;
                found_fica = true;
            } else if name_en_lower.contains("health insurance") {
                health += rate;
                found_health = true;
            } else if name_en_lower.contains("pension")
                || name_en_lower.contains("retirement")
            {
                retirement += rate;
                found_retirement = true;
            } else {
                // Unrecognised statutory deductions (solidarity surcharge,
                // church tax, unemployment insurance, long-term care, etc.)
                // are accumulated into the social-security / FICA bucket so
                // that total deductions still reflect the country's burden.
                fica += rate;
                found_fica = true;
            }
        }

        PayrollRates {
            income_tax_rate: if found_federal || found_state {
                let f = if found_federal { federal_tax } else { fallback.income_tax_rate - Decimal::from_f64_retain(self.config.tax_rates.state_effective).unwrap_or(Decimal::ZERO) };
                let s = if found_state { state_tax } else { Decimal::from_f64_retain(self.config.tax_rates.state_effective).unwrap_or(Decimal::ZERO) };
                f + s
            } else {
                fallback.income_tax_rate
            },
            fica_rate: if found_fica { fica } else { fallback.fica_rate },
            health_rate: if found_health { health } else { fallback.health_rate },
            retirement_rate: if found_retirement { retirement } else { fallback.retirement_rate },
            employer_fica_rate: if found_fica { fica } else { fallback.employer_fica_rate },
        }
    }

    /// Core generation logic parameterised on resolved rates.
    fn generate_with_rates(
        &mut self,
        company_code: &str,
        employees: &[(String, Decimal, Option<String>, Option<String>)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
        rates: &PayrollRates,
    ) -> (PayrollRun, Vec<PayrollLineItem>) {
        let payroll_id = self.uuid_factory.next().to_string();

        let mut line_items = Vec::with_capacity(employees.len());
        let mut total_gross = Decimal::ZERO;
        let mut total_deductions = Decimal::ZERO;
        let mut total_net = Decimal::ZERO;
        let mut total_employer_cost = Decimal::ZERO;

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
            let tax_withholding = (gross_pay * rates.income_tax_rate).round_dp(2);
            let social_security = (gross_pay * rates.fica_rate).round_dp(2);

            let health_insurance = if self.rng.gen_bool(benefits_enrolled) {
                (gross_pay * rates.health_rate).round_dp(2)
            } else {
                Decimal::ZERO
            };

            let retirement_contribution = if self.rng.gen_bool(retirement_participating) {
                (gross_pay * rates.retirement_rate).round_dp(2)
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

            // Employer-side cost: gross + employer contribution match
            let employer_contrib = (gross_pay * rates.employer_fica_rate).round_dp(2);
            let employer_cost = gross_pay + employer_contrib;

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

    // ---------------------------------------------------------------
    // Country-pack tests
    // ---------------------------------------------------------------

    /// Helper: build a US-like country pack with explicit statutory deductions.
    fn us_country_pack() -> CountryPack {
        use datasynth_core::country::schema::{PayrollCountryConfig, PayrollDeduction};
        CountryPack {
            country_code: "US".to_string(),
            payroll: PayrollCountryConfig {
                statutory_deductions: vec![
                    PayrollDeduction {
                        code: "FICA".to_string(),
                        name_en: "Federal Insurance Contributions Act".to_string(),
                        deduction_type: "percentage".to_string(),
                        rate: 0.0765,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "FIT".to_string(),
                        name_en: "Federal Income Tax".to_string(),
                        deduction_type: "progressive".to_string(),
                        rate: 0.0, // progressive placeholder
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "SIT".to_string(),
                        name_en: "State Income Tax".to_string(),
                        deduction_type: "percentage".to_string(),
                        rate: 0.05,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Helper: build a DE-like country pack.
    fn de_country_pack() -> CountryPack {
        use datasynth_core::country::schema::{PayrollCountryConfig, PayrollDeduction};
        CountryPack {
            country_code: "DE".to_string(),
            payroll: PayrollCountryConfig {
                pay_frequency: "monthly".to_string(),
                currency: "EUR".to_string(),
                statutory_deductions: vec![
                    PayrollDeduction {
                        code: "LOHNST".to_string(),
                        name_en: "Income Tax".to_string(),
                        type_field: "progressive".to_string(),
                        rate: 0.0, // progressive placeholder
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "SOLI".to_string(),
                        name_en: "Solidarity Surcharge".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.055,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "KiSt".to_string(),
                        name_en: "Church Tax".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.08,
                        optional: true,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "RV".to_string(),
                        name_en: "Pension Insurance".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.093,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "KV".to_string(),
                        name_en: "Health Insurance".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.073,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "AV".to_string(),
                        name_en: "Unemployment Insurance".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.013,
                        ..Default::default()
                    },
                    PayrollDeduction {
                        code: "PV".to_string(),
                        name_en: "Long-Term Care Insurance".to_string(),
                        type_field: "percentage".to_string(),
                        rate: 0.017,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_with_us_country_pack() {
        let mut gen = PayrollGenerator::new(42);
        let employees = test_employees();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let pack = us_country_pack();

        let (run, items) = gen.generate_with_country_pack(
            "C001", &employees, period_start, period_end, "USD", &pack,
        );

        assert_eq!(run.company_code, "C001");
        assert_eq!(run.employee_count, 3);
        assert_eq!(items.len(), 3);
        assert_eq!(run.total_net, run.total_gross - run.total_deductions);

        for item in &items {
            assert!(item.gross_pay > Decimal::ZERO);
            assert!(item.net_pay > Decimal::ZERO);
            assert!(item.net_pay < item.gross_pay);
            // FICA deduction should be present
            assert!(item.social_security > Decimal::ZERO);
        }
    }

    #[test]
    fn test_generate_with_de_country_pack() {
        let mut gen = PayrollGenerator::new(42);
        let employees = test_employees();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let pack = de_country_pack();

        let (run, items) = gen.generate_with_country_pack(
            "DE01", &employees, period_start, period_end, "EUR", &pack,
        );

        assert_eq!(run.company_code, "DE01");
        assert_eq!(items.len(), 3);
        assert_eq!(run.total_net, run.total_gross - run.total_deductions);

        // DE pack should use pension rate 0.093 for retirement
        // and health insurance rate 0.073
        let rates = gen.rates_from_country_pack(&pack);
        assert_eq!(
            rates.retirement_rate,
            Decimal::from_f64_retain(0.093).unwrap()
        );
        assert_eq!(
            rates.health_rate,
            Decimal::from_f64_retain(0.073).unwrap()
        );
    }

    #[test]
    fn test_country_pack_falls_back_to_config_for_missing_categories() {
        // Empty pack: no statutory deductions => all rates fall back to config
        let pack = CountryPack::default();
        let gen = PayrollGenerator::new(42);
        let rates_pack = gen.rates_from_country_pack(&pack);
        let rates_cfg = gen.rates_from_config();

        assert_eq!(rates_pack.income_tax_rate, rates_cfg.income_tax_rate);
        assert_eq!(rates_pack.fica_rate, rates_cfg.fica_rate);
        assert_eq!(rates_pack.health_rate, rates_cfg.health_rate);
        assert_eq!(rates_pack.retirement_rate, rates_cfg.retirement_rate);
    }

    #[test]
    fn test_country_pack_deterministic() {
        let employees = test_employees();
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let pack = de_country_pack();

        let mut gen1 = PayrollGenerator::new(42);
        let (run1, items1) = gen1.generate_with_country_pack(
            "DE01", &employees, period_start, period_end, "EUR", &pack,
        );

        let mut gen2 = PayrollGenerator::new(42);
        let (run2, items2) = gen2.generate_with_country_pack(
            "DE01", &employees, period_start, period_end, "EUR", &pack,
        );

        assert_eq!(run1.payroll_id, run2.payroll_id);
        assert_eq!(run1.total_gross, run2.total_gross);
        assert_eq!(run1.total_net, run2.total_net);
        for (a, b) in items1.iter().zip(items2.iter()) {
            assert_eq!(a.net_pay, b.net_pay);
        }
    }

    #[test]
    fn test_de_rates_differ_from_default() {
        // With the DE pack, the resolved rates should differ from config defaults
        let gen = PayrollGenerator::new(42);
        let pack = de_country_pack();
        let rates_cfg = gen.rates_from_config();
        let rates_de = gen.rates_from_country_pack(&pack);

        // DE has no non-progressive income tax in pack → income_tax_rate falls
        // back to config default for federal part.
        // But health (0.073 vs 0.03) and retirement (0.093 vs 0.05) should differ.
        assert_ne!(rates_de.health_rate, rates_cfg.health_rate);
        assert_ne!(rates_de.retirement_rate, rates_cfg.retirement_rate);
    }
}
