//! Earned Value Management (EVM) metrics generator.
//!
//! Computes EVM metrics (SPI, CPI, EAC, ETC, TCPI) for projects based on
//! WBS budgets, actual costs, and schedule progress.
use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::EarnedValueSchemaConfig;
use datasynth_core::models::{EarnedValueMetric, Project, ProjectCostLine};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Generates [`EarnedValueMetric`] records for projects.
pub struct EarnedValueGenerator {
    rng: ChaCha8Rng,
    /// Controls measurement frequency (weekly, biweekly, monthly).
    config: EarnedValueSchemaConfig,
    counter: u64,
}

impl EarnedValueGenerator {
    /// Create a new earned value generator.
    pub fn new(config: EarnedValueSchemaConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generate EVM metrics for a set of projects.
    ///
    /// Produces one metric per project per measurement period (monthly by default).
    /// Planned Value (PV) is computed as a linear schedule baseline from the project budget.
    /// Earned Value (EV) reflects the budget value of work actually performed (with a
    /// small random efficiency factor to create realistic SPI/CPI variations).
    /// Actual Cost (AC) comes directly from the cost lines.
    pub fn generate(
        &mut self,
        projects: &[Project],
        cost_lines: &[ProjectCostLine],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<EarnedValueMetric> {
        let mut metrics = Vec::new();

        for project in projects {
            if !project.allows_postings() && project.budget.is_zero() {
                continue;
            }

            let bac = project.budget;
            let project_costs: Vec<&ProjectCostLine> = cost_lines
                .iter()
                .filter(|cl| cl.project_id == project.project_id)
                .collect();

            // Parse project dates or use provided range
            let proj_start = project
                .start_date
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or(start_date);
            let proj_end = project
                .end_date
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or(end_date);

            let total_days = (proj_end - proj_start).num_days().max(1) as f64;

            // Generate metrics at measurement frequency
            let mut current = start_date;
            while current <= end_date {
                let measurement_date = self.measurement_date(current);
                if measurement_date > end_date {
                    break;
                }

                // Actual Cost: sum of cost lines up to measurement date
                let ac: Decimal = project_costs
                    .iter()
                    .filter(|cl| cl.posting_date <= measurement_date)
                    .map(|cl| cl.amount)
                    .sum();

                // Skip periods with no cost activity
                if ac.is_zero() {
                    current = self.advance_date(current);
                    continue;
                }

                // Planned Value: linear schedule baseline
                let elapsed_days = (measurement_date - proj_start).num_days().max(0) as f64;
                let schedule_pct = (elapsed_days / total_days).min(1.0);
                let pv =
                    (bac * Decimal::from_f64_retain(schedule_pct).unwrap_or(dec!(0))).round_dp(2);

                // Earned Value: actual cost adjusted by efficiency factor
                // Creates realistic SPI/CPI variations
                let efficiency: f64 = self.rng.random_range(0.75..1.25);
                let ev = (ac * Decimal::from_f64_retain(efficiency).unwrap_or(dec!(1)))
                    .min(bac)
                    .round_dp(2);

                self.counter += 1;
                let metric = EarnedValueMetric::compute(
                    format!("EVM-{:06}", self.counter),
                    &project.project_id,
                    measurement_date,
                    bac,
                    pv,
                    ev,
                    ac,
                );
                metrics.push(metric);

                current = self.advance_date(current);
            }
        }

        metrics
    }

    /// Advance to the next measurement period based on configured frequency.
    fn advance_date(&self, current: NaiveDate) -> NaiveDate {
        match self.config.frequency.as_str() {
            "weekly" => current + chrono::Duration::weeks(1),
            "biweekly" => current + chrono::Duration::weeks(2),
            _ => next_month_start(current),
        }
    }

    /// Get the measurement date for the current period.
    fn measurement_date(&self, current: NaiveDate) -> NaiveDate {
        match self.config.frequency.as_str() {
            "weekly" | "biweekly" => current,
            _ => end_of_month(current),
        }
    }
}

/// Get the last day of a month.
fn end_of_month(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .expect("valid date")
        .pred_opt()
        .expect("valid date")
}

/// Get the first day of the next month.
fn next_month_start(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).expect("valid date")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{CostCategory, CostSourceType, ProjectType, WbsElement};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn test_project() -> Project {
        let mut project = Project::new("PRJ-001", "Test Project", ProjectType::Capital)
            .with_budget(dec!(1000000))
            .with_company("TEST");
        project.start_date = Some("2024-01-01".to_string());
        project.end_date = Some("2024-12-31".to_string());
        project.add_wbs_element(
            WbsElement::new("PRJ-001.01", "PRJ-001", "Phase 1").with_budget(dec!(500000)),
        );
        project.add_wbs_element(
            WbsElement::new("PRJ-001.02", "PRJ-001", "Phase 2").with_budget(dec!(500000)),
        );
        project
    }

    fn test_cost_lines() -> Vec<ProjectCostLine> {
        vec![
            ProjectCostLine::new(
                "PCL-001",
                "PRJ-001",
                "PRJ-001.01",
                "TEST",
                d("2024-01-15"),
                CostCategory::Labor,
                CostSourceType::TimeEntry,
                "TE-001",
                dec!(80000),
                "USD",
            ),
            ProjectCostLine::new(
                "PCL-002",
                "PRJ-001",
                "PRJ-001.01",
                "TEST",
                d("2024-02-15"),
                CostCategory::Labor,
                CostSourceType::TimeEntry,
                "TE-002",
                dec!(90000),
                "USD",
            ),
            ProjectCostLine::new(
                "PCL-003",
                "PRJ-001",
                "PRJ-001.02",
                "TEST",
                d("2024-03-15"),
                CostCategory::Material,
                CostSourceType::PurchaseOrder,
                "PO-001",
                dec!(120000),
                "USD",
            ),
        ]
    }

    #[test]
    fn test_evm_generation() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let config = EarnedValueSchemaConfig::default();

        let mut gen = EarnedValueGenerator::new(config, 42);
        let metrics = gen.generate(&[project], &cost_lines, d("2024-01-01"), d("2024-03-31"));

        assert_eq!(
            metrics.len(),
            3,
            "Should have one metric per month with costs"
        );

        for metric in &metrics {
            assert_eq!(metric.project_id, "PRJ-001");
            assert_eq!(metric.bac, dec!(1000000));
            assert!(metric.actual_cost > Decimal::ZERO);
            // SV = EV - PV should be computed
            assert_eq!(
                metric.schedule_variance,
                metric.earned_value - metric.planned_value
            );
            // CV = EV - AC should be computed
            assert_eq!(
                metric.cost_variance,
                metric.earned_value - metric.actual_cost
            );
        }
    }

    #[test]
    fn test_evm_formulas_correct() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let config = EarnedValueSchemaConfig::default();

        let mut gen = EarnedValueGenerator::new(config, 42);
        let metrics = gen.generate(&[project], &cost_lines, d("2024-01-01"), d("2024-03-31"));

        for metric in &metrics {
            // Verify SPI = EV / PV
            if metric.planned_value > Decimal::ZERO {
                let expected_spi = (metric.earned_value / metric.planned_value).round_dp(4);
                assert_eq!(metric.spi, expected_spi, "SPI formula incorrect");
            }

            // Verify CPI = EV / AC
            if metric.actual_cost > Decimal::ZERO {
                let expected_cpi = (metric.earned_value / metric.actual_cost).round_dp(4);
                assert_eq!(metric.cpi, expected_cpi, "CPI formula incorrect");
            }

            // Verify SV = EV - PV
            let expected_sv = (metric.earned_value - metric.planned_value).round_dp(2);
            assert_eq!(
                metric.schedule_variance, expected_sv,
                "SV formula incorrect"
            );

            // Verify CV = EV - AC
            let expected_cv = (metric.earned_value - metric.actual_cost).round_dp(2);
            assert_eq!(metric.cost_variance, expected_cv, "CV formula incorrect");
        }
    }

    #[test]
    fn test_evm_no_costs_no_metrics() {
        let project = test_project();
        let config = EarnedValueSchemaConfig::default();

        let mut gen = EarnedValueGenerator::new(config, 42);
        let metrics = gen.generate(&[project], &[], d("2024-01-01"), d("2024-03-31"));

        assert!(metrics.is_empty(), "No costs should produce no EVM metrics");
    }

    #[test]
    fn test_evm_deterministic() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let config = EarnedValueSchemaConfig::default();

        let mut gen1 = EarnedValueGenerator::new(config.clone(), 42);
        let m1 = gen1.generate(
            std::slice::from_ref(&project),
            &cost_lines,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        let mut gen2 = EarnedValueGenerator::new(config, 42);
        let m2 = gen2.generate(
            std::slice::from_ref(&project),
            &cost_lines,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        assert_eq!(m1.len(), m2.len());
        for (a, b) in m1.iter().zip(m2.iter()) {
            assert_eq!(a.spi, b.spi);
            assert_eq!(a.cpi, b.cpi);
            assert_eq!(a.eac, b.eac);
        }
    }
}
