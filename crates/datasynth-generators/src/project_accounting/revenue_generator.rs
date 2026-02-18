//! Project revenue recognition generator (Percentage of Completion).
//!
//! Takes project cost lines and project contract values to compute revenue
//! recognition using the cost-to-cost PoC method (ASC 606 input method).
#![allow(dead_code)]

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::ProjectRevenueRecognitionConfig;
use datasynth_core::models::{
    CompletionMeasure, Project, ProjectCostLine, ProjectRevenue, RevenueMethod,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Generates [`ProjectRevenue`] records using Percentage of Completion.
pub struct RevenueGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: ProjectRevenueRecognitionConfig,
    counter: u64,
}

impl RevenueGenerator {
    /// Create a new revenue generator.
    pub fn new(config: ProjectRevenueRecognitionConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProjectAccounting),
            config,
            counter: 0,
        }
    }

    /// Generate revenue recognition records for customer projects.
    ///
    /// Only generates revenue for projects that have contract values (customer projects).
    /// Revenue is computed per month using the cost-to-cost PoC method.
    pub fn generate(
        &mut self,
        projects: &[Project],
        cost_lines: &[ProjectCostLine],
        contract_values: &[(String, Decimal, Decimal)], // (project_id, contract_value, estimated_total_cost)
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<ProjectRevenue> {
        let mut revenues = Vec::new();

        for (project_id, contract_value, estimated_total_cost) in contract_values {
            let project = match projects.iter().find(|p| &p.project_id == project_id) {
                Some(p) => p,
                None => continue,
            };

            // Collect cost lines for this project, sorted by date
            let mut project_costs: Vec<&ProjectCostLine> = cost_lines
                .iter()
                .filter(|cl| &cl.project_id == project_id)
                .collect();
            project_costs.sort_by_key(|cl| cl.posting_date);

            // Generate monthly revenue records
            let mut current = start_date;
            let mut prev_cumulative_revenue = dec!(0);
            let mut billed_to_date = dec!(0);

            while current <= end_date {
                let period_end = end_of_month(current);
                let costs_to_date: Decimal = project_costs
                    .iter()
                    .filter(|cl| cl.posting_date <= period_end)
                    .map(|cl| cl.amount)
                    .sum();

                if costs_to_date.is_zero() {
                    current = next_month_start(current);
                    continue;
                }

                let completion_pct = if estimated_total_cost.is_zero() {
                    dec!(0)
                } else {
                    (costs_to_date / estimated_total_cost)
                        .min(dec!(1.0))
                        .round_dp(4)
                };

                let cumulative_revenue = (*contract_value * completion_pct).round_dp(2);
                let period_revenue = (cumulative_revenue - prev_cumulative_revenue).max(dec!(0));

                // Billing lags behind recognition by a random factor
                let billing_pct: f64 = self.rng.gen_range(0.70..0.95);
                let target_billed = cumulative_revenue
                    * Decimal::from_f64_retain(billing_pct).unwrap_or(dec!(0.85));
                if target_billed > billed_to_date {
                    billed_to_date = target_billed.round_dp(2);
                }

                let unbilled_revenue = (cumulative_revenue - billed_to_date).round_dp(2);
                let gross_margin_pct = if contract_value.is_zero() {
                    dec!(0)
                } else {
                    ((*contract_value - *estimated_total_cost) / *contract_value).round_dp(4)
                };

                self.counter += 1;
                let rev = ProjectRevenue {
                    id: format!("PREV-{:06}", self.counter),
                    project_id: project_id.clone(),
                    entity_id: project.company_code.clone(),
                    period_start: current,
                    period_end,
                    contract_value: *contract_value,
                    estimated_total_cost: *estimated_total_cost,
                    costs_to_date,
                    completion_pct,
                    method: RevenueMethod::PercentageOfCompletion,
                    measure: CompletionMeasure::CostToCost,
                    cumulative_revenue,
                    period_revenue,
                    billed_to_date,
                    unbilled_revenue,
                    gross_margin_pct,
                };

                prev_cumulative_revenue = cumulative_revenue;
                revenues.push(rev);
                current = next_month_start(current);
            }
        }

        revenues
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
    use datasynth_core::models::{CostCategory, CostSourceType, ProjectType};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn test_project() -> Project {
        Project::new("PRJ-001", "Customer Build", ProjectType::Customer)
            .with_budget(dec!(800000))
            .with_company("TEST")
    }

    fn test_cost_lines() -> Vec<ProjectCostLine> {
        // Create cost lines spread across 3 months
        let months = [
            (d("2024-01-15"), dec!(100000)),
            (d("2024-02-15"), dec!(150000)),
            (d("2024-03-15"), dec!(200000)),
        ];
        let mut lines = Vec::new();
        for (i, (date, amount)) in months.iter().enumerate() {
            lines.push(ProjectCostLine::new(
                format!("PCL-{:03}", i + 1),
                "PRJ-001",
                "PRJ-001.01",
                "TEST",
                *date,
                CostCategory::Labor,
                CostSourceType::TimeEntry,
                format!("TE-{:03}", i + 1),
                *amount,
                "USD",
            ));
        }
        lines
    }

    #[test]
    fn test_revenue_increases_monotonically() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let contracts = vec![("PRJ-001".to_string(), dec!(1000000), dec!(800000))];

        let config = ProjectRevenueRecognitionConfig::default();
        let mut gen = RevenueGenerator::new(config, 42);
        let revenues = gen.generate(
            &[project],
            &cost_lines,
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        assert!(!revenues.is_empty(), "Should generate revenue records");

        let mut prev_cumulative = dec!(0);
        for rev in &revenues {
            assert!(
                rev.cumulative_revenue >= prev_cumulative,
                "Revenue should increase monotonically: {} >= {}",
                rev.cumulative_revenue,
                prev_cumulative
            );
            prev_cumulative = rev.cumulative_revenue;
        }
    }

    #[test]
    fn test_unbilled_revenue_calculation() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let contracts = vec![("PRJ-001".to_string(), dec!(1000000), dec!(800000))];

        let config = ProjectRevenueRecognitionConfig::default();
        let mut gen = RevenueGenerator::new(config, 42);
        let revenues = gen.generate(
            &[project],
            &cost_lines,
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        for rev in &revenues {
            let expected_unbilled = (rev.cumulative_revenue - rev.billed_to_date).round_dp(2);
            assert_eq!(
                rev.unbilled_revenue, expected_unbilled,
                "Unbilled revenue = recognized - billed"
            );
        }
    }

    #[test]
    fn test_poc_completion_calculation() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let contracts = vec![("PRJ-001".to_string(), dec!(1000000), dec!(800000))];

        let config = ProjectRevenueRecognitionConfig::default();
        let mut gen = RevenueGenerator::new(config, 42);
        let revenues = gen.generate(
            &[project],
            &cost_lines,
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        // After month 1: costs 100000 / 800000 = 0.125
        assert_eq!(revenues[0].completion_pct, dec!(0.1250));
        // After month 2: costs 250000 / 800000 = 0.3125
        assert_eq!(revenues[1].completion_pct, dec!(0.3125));
        // After month 3: costs 450000 / 800000 = 0.5625
        assert_eq!(revenues[2].completion_pct, dec!(0.5625));
    }

    #[test]
    fn test_no_revenue_without_costs() {
        let project = test_project();
        let contracts = vec![("PRJ-001".to_string(), dec!(1000000), dec!(800000))];

        let config = ProjectRevenueRecognitionConfig::default();
        let mut gen = RevenueGenerator::new(config, 42);
        let revenues = gen.generate(
            &[project],
            &[], // No cost lines
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        assert!(revenues.is_empty(), "No costs should produce no revenue");
    }

    #[test]
    fn test_deterministic_revenue() {
        let project = test_project();
        let cost_lines = test_cost_lines();
        let contracts = vec![("PRJ-001".to_string(), dec!(1000000), dec!(800000))];

        let config = ProjectRevenueRecognitionConfig::default();
        let mut gen1 = RevenueGenerator::new(config.clone(), 42);
        let rev1 = gen1.generate(
            &[project.clone()],
            &cost_lines,
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        let mut gen2 = RevenueGenerator::new(config, 42);
        let rev2 = gen2.generate(
            &[project],
            &cost_lines,
            &contracts,
            d("2024-01-01"),
            d("2024-03-31"),
        );

        assert_eq!(rev1.len(), rev2.len());
        for (r1, r2) in rev1.iter().zip(rev2.iter()) {
            assert_eq!(r1.cumulative_revenue, r2.cumulative_revenue);
            assert_eq!(r1.billed_to_date, r2.billed_to_date);
        }
    }
}
