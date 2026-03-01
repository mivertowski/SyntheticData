//! Benefit enrollment generator for the Hire-to-Retire (H2R) process.
//!
//! Generates benefit enrollment records for employees across plan types
//! (health, dental, vision, retirement, life insurance) with realistic
//! contribution amounts and enrollment distributions.

use chrono::NaiveDate;
use datasynth_core::models::{BenefitEnrollment, BenefitPlanType, BenefitStatus};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Plan names by type for realistic output.
const HEALTH_PLANS: &[&str] = &["Blue Cross PPO", "Aetna HMO", "UnitedHealth Choice Plus"];
const DENTAL_PLANS: &[&str] = &["Delta Dental Basic", "MetLife Dental PPO"];
const VISION_PLANS: &[&str] = &["VSP Standard", "EyeMed Vision Care"];
const RETIREMENT_PLANS: &[&str] = &["401(k) Traditional", "401(k) Roth"];
const LIFE_PLANS: &[&str] = &["Basic Life 1x Salary", "Supplemental Life 2x"];

/// Generates [`BenefitEnrollment`] records for employees.
pub struct BenefitEnrollmentGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BenefitEnrollmentGenerator {
    /// Create a new benefit enrollment generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BenefitEnrollment),
        }
    }

    /// Generate benefit enrollments for employees.
    ///
    /// Each employee receives 1-3 benefit enrollments based on plan type
    /// enrollment probabilities: Health (90%), Dental (70%), Vision (50%),
    /// Retirement (60%), Life Insurance (40%).
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company / entity code.
    /// * `employees` - Employee data as `(employee_id, employee_name)` tuples.
    /// * `enrollment_date` - Base enrollment date (e.g., open enrollment period).
    /// * `currency` - Currency code for contribution amounts.
    pub fn generate(
        &mut self,
        company_code: &str,
        employees: &[(String, String)],
        enrollment_date: NaiveDate,
        currency: &str,
    ) -> Vec<BenefitEnrollment> {
        debug!(
            company_code,
            employee_count = employees.len(),
            %enrollment_date,
            "Generating benefit enrollments"
        );

        let mut enrollments = Vec::new();
        let effective_date = enrollment_date;
        let period = format!(
            "{}-{:02}",
            enrollment_date.format("%Y"),
            enrollment_date.format("%m")
        );

        for (employee_id, employee_name) in employees {
            // Health: 90% enrollment rate
            if self.rng.random_bool(0.90) {
                let enrollment = self.make_enrollment(
                    company_code,
                    employee_id,
                    employee_name,
                    BenefitPlanType::Health,
                    HEALTH_PLANS,
                    enrollment_date,
                    effective_date,
                    &period,
                    currency,
                );
                enrollments.push(enrollment);
            }

            // Dental: 70%
            if self.rng.random_bool(0.70) {
                let enrollment = self.make_enrollment(
                    company_code,
                    employee_id,
                    employee_name,
                    BenefitPlanType::Dental,
                    DENTAL_PLANS,
                    enrollment_date,
                    effective_date,
                    &period,
                    currency,
                );
                enrollments.push(enrollment);
            }

            // Vision: 50%
            if self.rng.random_bool(0.50) {
                let enrollment = self.make_enrollment(
                    company_code,
                    employee_id,
                    employee_name,
                    BenefitPlanType::Vision,
                    VISION_PLANS,
                    enrollment_date,
                    effective_date,
                    &period,
                    currency,
                );
                enrollments.push(enrollment);
            }

            // Retirement: 60%
            if self.rng.random_bool(0.60) {
                let enrollment = self.make_enrollment(
                    company_code,
                    employee_id,
                    employee_name,
                    BenefitPlanType::Retirement401k,
                    RETIREMENT_PLANS,
                    enrollment_date,
                    effective_date,
                    &period,
                    currency,
                );
                enrollments.push(enrollment);
            }

            // Life insurance: 40%
            if self.rng.random_bool(0.40) {
                let enrollment = self.make_enrollment(
                    company_code,
                    employee_id,
                    employee_name,
                    BenefitPlanType::LifeInsurance,
                    LIFE_PLANS,
                    enrollment_date,
                    effective_date,
                    &period,
                    currency,
                );
                enrollments.push(enrollment);
            }
        }

        enrollments
    }

    /// Create a single enrollment record.
    #[allow(clippy::too_many_arguments)]
    fn make_enrollment(
        &mut self,
        company_code: &str,
        employee_id: &str,
        employee_name: &str,
        plan_type: BenefitPlanType,
        plan_names: &[&str],
        enrollment_date: NaiveDate,
        effective_date: NaiveDate,
        period: &str,
        currency: &str,
    ) -> BenefitEnrollment {
        let id = self.uuid_factory.next().to_string();
        let plan_name = plan_names[self.rng.random_range(0..plan_names.len())].to_string();

        let (employee_contrib, employer_contrib) = self.contribution_amounts(plan_type);

        // 95% active, 3% pending, 2% terminated
        let status_roll: f64 = self.rng.random();
        let (status, is_active) = if status_roll < 0.95 {
            (BenefitStatus::Active, true)
        } else if status_roll < 0.98 {
            (BenefitStatus::Pending, false)
        } else {
            (BenefitStatus::Terminated, false)
        };

        BenefitEnrollment::new(
            id,
            company_code,
            employee_id,
            employee_name,
            plan_type,
            plan_name,
            enrollment_date,
            effective_date,
            period,
            employee_contrib,
            employer_contrib,
            currency,
            status,
            is_active,
        )
    }

    /// Generate contribution amounts based on plan type.
    fn contribution_amounts(&mut self, plan_type: BenefitPlanType) -> (Decimal, Decimal) {
        let (emp_min, emp_max, er_min, er_max) = match plan_type {
            BenefitPlanType::Health => (200.0, 800.0, 400.0, 1200.0),
            BenefitPlanType::Dental => (25.0, 75.0, 30.0, 90.0),
            BenefitPlanType::Vision => (10.0, 30.0, 10.0, 30.0),
            BenefitPlanType::Retirement401k => (200.0, 2000.0, 100.0, 1000.0),
            BenefitPlanType::LifeInsurance => (15.0, 50.0, 15.0, 50.0),
            BenefitPlanType::StockPurchase => (100.0, 500.0, 0.0, 0.0),
            BenefitPlanType::Disability => (20.0, 60.0, 20.0, 60.0),
        };

        let emp: f64 = self.rng.random_range(emp_min..=emp_max);
        let er: f64 = self.rng.random_range(er_min..=er_max);

        let employee_contribution = Decimal::from_f64_retain(emp)
            .unwrap_or(Decimal::from(100))
            .round_dp(2);
        let employer_contribution = Decimal::from_f64_retain(er)
            .unwrap_or(Decimal::from(100))
            .round_dp(2);

        (employee_contribution, employer_contribution)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_employees() -> Vec<(String, String)> {
        vec![
            ("EMP-001".to_string(), "Alice Smith".to_string()),
            ("EMP-002".to_string(), "Bob Jones".to_string()),
            ("EMP-003".to_string(), "Carol White".to_string()),
            ("EMP-004".to_string(), "David Brown".to_string()),
            ("EMP-005".to_string(), "Eve Johnson".to_string()),
        ]
    }

    #[test]
    fn test_enrollment_generation() {
        let mut gen = BenefitEnrollmentGenerator::new(42);
        let employees = test_employees();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let enrollments = gen.generate("C001", &employees, date, "USD");

        assert!(!enrollments.is_empty());
        for e in &enrollments {
            assert_eq!(e.entity_code, "C001");
            assert_eq!(e.currency, "USD");
            assert!(e.employee_contribution > Decimal::ZERO);
            assert!(!e.employee_name.is_empty());
            assert!(!e.plan_name.is_empty());
        }
    }

    #[test]
    fn test_enrollment_plan_types() {
        let mut gen = BenefitEnrollmentGenerator::new(77);
        let employees: Vec<(String, String)> = (0..50)
            .map(|i| (format!("EMP-{:03}", i), format!("Employee {}", i)))
            .collect();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let enrollments = gen.generate("C001", &employees, date, "USD");

        let health = enrollments
            .iter()
            .filter(|e| matches!(e.plan_type, BenefitPlanType::Health))
            .count();
        let dental = enrollments
            .iter()
            .filter(|e| matches!(e.plan_type, BenefitPlanType::Dental))
            .count();

        // With 50 employees and 90% health rate, expect ~40-50 health enrollments
        assert!(
            health >= 30,
            "Expected ~90% health enrollment, got {}/50",
            health
        );
        assert!(dental > 0, "Expected some dental enrollments");
    }

    #[test]
    fn test_enrollment_deterministic() {
        let employees = test_employees();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let mut gen1 = BenefitEnrollmentGenerator::new(12345);
        let e1 = gen1.generate("C001", &employees, date, "USD");
        let mut gen2 = BenefitEnrollmentGenerator::new(12345);
        let e2 = gen2.generate("C001", &employees, date, "USD");

        assert_eq!(e1.len(), e2.len());
        for (a, b) in e1.iter().zip(e2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.employee_id, b.employee_id);
            assert_eq!(a.plan_type, b.plan_type);
            assert_eq!(a.employee_contribution, b.employee_contribution);
        }
    }

    #[test]
    fn test_enrollment_active_rate() {
        let mut gen = BenefitEnrollmentGenerator::new(55);
        let employees: Vec<(String, String)> = (0..100)
            .map(|i| (format!("EMP-{:03}", i), format!("Employee {}", i)))
            .collect();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let enrollments = gen.generate("C001", &employees, date, "USD");
        let active_count = enrollments.iter().filter(|e| e.is_active).count();
        let active_pct = active_count as f64 / enrollments.len() as f64;

        assert!(
            active_pct > 0.85,
            "Expected ~95% active rate, got {:.1}%",
            active_pct * 100.0
        );
    }
}
