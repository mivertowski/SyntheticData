//! Management KPI generator.
//!
//! Generates realistic key performance indicators across financial, operational,
//! customer, employee, and quality categories with period-over-period trends.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::ManagementKpisConfig;
use datasynth_core::models::{KpiCategory, KpiTrend, ManagementKpi};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Definition of a standard KPI template.
struct KpiDefinition {
    name: &'static str,
    category: KpiCategory,
    unit: &'static str,
    target_min: f64,
    target_max: f64,
}

/// Standard KPIs generated per period.
const STANDARD_KPIS: &[KpiDefinition] = &[
    KpiDefinition {
        name: "Revenue Growth Rate",
        category: KpiCategory::Financial,
        unit: "%",
        target_min: 5.0,
        target_max: 15.0,
    },
    KpiDefinition {
        name: "Gross Margin",
        category: KpiCategory::Financial,
        unit: "%",
        target_min: 30.0,
        target_max: 60.0,
    },
    KpiDefinition {
        name: "Operating Margin",
        category: KpiCategory::Financial,
        unit: "%",
        target_min: 10.0,
        target_max: 25.0,
    },
    KpiDefinition {
        name: "Current Ratio",
        category: KpiCategory::Financial,
        unit: "ratio",
        target_min: 1.5,
        target_max: 2.5,
    },
    KpiDefinition {
        name: "Days Sales Outstanding",
        category: KpiCategory::Operational,
        unit: "days",
        target_min: 30.0,
        target_max: 60.0,
    },
    KpiDefinition {
        name: "Inventory Turnover",
        category: KpiCategory::Operational,
        unit: "turns",
        target_min: 4.0,
        target_max: 12.0,
    },
    KpiDefinition {
        name: "Order Fulfillment Rate",
        category: KpiCategory::Operational,
        unit: "%",
        target_min: 95.0,
        target_max: 99.0,
    },
    KpiDefinition {
        name: "Customer Satisfaction Score",
        category: KpiCategory::Customer,
        unit: "score",
        target_min: 80.0,
        target_max: 95.0,
    },
    KpiDefinition {
        name: "Employee Turnover Rate",
        category: KpiCategory::Employee,
        unit: "%",
        target_min: 5.0,
        target_max: 15.0,
    },
    KpiDefinition {
        name: "Defect Rate",
        category: KpiCategory::Quality,
        unit: "%",
        target_min: 0.5,
        target_max: 3.0,
    },
];

/// Generates [`ManagementKpi`] instances with realistic values,
/// targets, trends, and period-over-period comparisons.
pub struct KpiGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl KpiGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Kpi),
        }
    }

    /// Generate management KPIs for the given period and configuration.
    ///
    /// # Arguments
    ///
    /// * `company_code` - The company code these KPIs belong to.
    /// * `period_start` - Start of the generation period (inclusive).
    /// * `period_end` - End of the generation period (inclusive).
    /// * `config` - Management KPI configuration.
    pub fn generate(
        &mut self,
        company_code: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &ManagementKpisConfig,
    ) -> Vec<ManagementKpi> {
        tracing::debug!(company_code, %period_start, %period_end, "Generating management KPIs");
        let mut kpis = Vec::new();
        let is_quarterly = config.frequency.to_lowercase() == "quarterly";

        // Iterate over periods
        let mut current_start = period_start;
        while current_start <= period_end {
            let current_end = if is_quarterly {
                advance_quarter(current_start)
            } else {
                advance_month(current_start)
            };

            // Clamp to period_end
            let actual_end = if current_end > period_end {
                period_end
            } else {
                // End date is the last day of the period (day before next period starts)
                current_end.pred_opt().unwrap_or(current_end)
            };

            // Generate all standard KPIs for this period
            for def in STANDARD_KPIS {
                let kpi = self.generate_single_kpi(company_code, def, current_start, actual_end);
                kpis.push(kpi);
            }

            current_start = current_end;
            if current_start > period_end {
                break;
            }
        }

        kpis
    }

    /// Generate a single KPI for a given period.
    fn generate_single_kpi(
        &mut self,
        company_code: &str,
        def: &KpiDefinition,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> ManagementKpi {
        let kpi_id = self.uuid_factory.next().to_string();

        // Generate a target value within the defined range
        let target_raw: f64 = self.rng.random_range(def.target_min..=def.target_max);
        let target = Decimal::from_f64_retain(target_raw)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2);

        // Actual value = target * random(0.8 - 1.2) with noise
        let multiplier: f64 = self.rng.random_range(0.8..1.2);
        let value_raw = target_raw * multiplier;
        let value = Decimal::from_f64_retain(value_raw)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2);

        // Determine trend: if value > target -> Improving, within 5% -> Stable, else Declining
        let ratio = if target_raw > 0.0 {
            value_raw / target_raw
        } else {
            1.0
        };
        let trend = if ratio > 1.05 {
            KpiTrend::Improving
        } else if ratio >= 0.95 {
            KpiTrend::Stable
        } else {
            KpiTrend::Declining
        };

        // Year-over-year change: random -10% to +15%
        let yoy_raw: f64 = self.rng.random_range(-0.10..0.15);
        let year_over_year_change = Some((yoy_raw * 10000.0).round() / 10000.0);

        // Prior period value: value * (1 - small random change)
        let prior_change: f64 = self.rng.random_range(-0.08..0.08);
        let prior_raw = value_raw * (1.0 - prior_change);
        let prior_period_value = Some(
            Decimal::from_f64_retain(prior_raw)
                .unwrap_or(Decimal::ZERO)
                .round_dp(2),
        );

        ManagementKpi {
            kpi_id,
            company_code: company_code.to_string(),
            name: def.name.to_string(),
            category: def.category,
            period_start,
            period_end,
            value,
            target,
            unit: def.unit.to_string(),
            trend,
            year_over_year_change,
            prior_period_value,
        }
    }
}

/// Advance a date to the first day of the next month.
fn advance_month(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

/// Advance a date to the first day of the next quarter.
fn advance_quarter(date: NaiveDate) -> NaiveDate {
    let current_quarter_start_month = ((date.month() - 1) / 3) * 3 + 1;
    let next_quarter_month = current_quarter_start_month + 3;
    let (year, month) = if next_quarter_month > 12 {
        (date.year() + 1, next_quarter_month - 12)
    } else {
        (date.year(), next_quarter_month)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_monthly_config() -> ManagementKpisConfig {
        ManagementKpisConfig {
            enabled: true,
            frequency: "monthly".to_string(),
        }
    }

    fn default_quarterly_config() -> ManagementKpisConfig {
        ManagementKpisConfig {
            enabled: true,
            frequency: "quarterly".to_string(),
        }
    }

    #[test]
    fn test_monthly_generation_produces_correct_count() {
        let mut gen = KpiGenerator::new(42);
        let config = default_monthly_config();

        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let kpis = gen.generate("C001", period_start, period_end, &config);

        // 6 months * 10 standard KPIs = 60
        assert_eq!(kpis.len(), 60);

        // All KPIs should have valid fields
        for kpi in &kpis {
            assert!(!kpi.kpi_id.is_empty());
            assert_eq!(kpi.company_code, "C001");
            assert!(!kpi.name.is_empty());
            assert!(!kpi.unit.is_empty());
            assert!(kpi.value > Decimal::ZERO);
            assert!(kpi.target > Decimal::ZERO);
            assert!(kpi.year_over_year_change.is_some());
            assert!(kpi.prior_period_value.is_some());
        }

        // Check that all categories are represented
        let categories: std::collections::HashSet<_> = kpis.iter().map(|k| k.category).collect();
        assert!(categories.contains(&KpiCategory::Financial));
        assert!(categories.contains(&KpiCategory::Operational));
        assert!(categories.contains(&KpiCategory::Customer));
        assert!(categories.contains(&KpiCategory::Employee));
        assert!(categories.contains(&KpiCategory::Quality));
    }

    #[test]
    fn test_quarterly_generation() {
        let mut gen = KpiGenerator::new(99);
        let config = default_quarterly_config();

        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let kpis = gen.generate("C002", period_start, period_end, &config);

        // 4 quarters * 10 standard KPIs = 40
        assert_eq!(kpis.len(), 40);

        // Verify all trends are valid
        for kpi in &kpis {
            assert!(
                kpi.trend == KpiTrend::Improving
                    || kpi.trend == KpiTrend::Stable
                    || kpi.trend == KpiTrend::Declining
            );
        }
    }

    #[test]
    fn test_deterministic_output_with_same_seed() {
        let config = default_monthly_config();
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();

        let mut gen1 = KpiGenerator::new(12345);
        let kpis1 = gen1.generate("C001", period_start, period_end, &config);

        let mut gen2 = KpiGenerator::new(12345);
        let kpis2 = gen2.generate("C001", period_start, period_end, &config);

        assert_eq!(kpis1.len(), kpis2.len());
        for (k1, k2) in kpis1.iter().zip(kpis2.iter()) {
            assert_eq!(k1.kpi_id, k2.kpi_id);
            assert_eq!(k1.name, k2.name);
            assert_eq!(k1.value, k2.value);
            assert_eq!(k1.target, k2.target);
            assert_eq!(k1.trend, k2.trend);
        }
    }
}
