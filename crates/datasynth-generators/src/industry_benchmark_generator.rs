//! Industry benchmark generator.
//!
//! Generates synthetic industry-average metrics for comparative analysis.
//! Auditors use these benchmarks under ISA 520 (Analytical Procedures) to
//! evaluate an entity's financial ratios against sector norms.

use datasynth_core::models::IndustryBenchmark;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// A metric template with a base value and standard deviation for perturbation.
struct MetricDef {
    name: &'static str,
    base: f64,
    sigma: f64,
}

/// Retail industry benchmark definitions.
const RETAIL_METRICS: &[MetricDef] = &[
    MetricDef {
        name: "median_revenue",
        base: 50_000_000.0,
        sigma: 0.30,
    },
    MetricDef {
        name: "gross_margin_pct",
        base: 0.35,
        sigma: 0.05,
    },
    MetricDef {
        name: "net_margin_pct",
        base: 0.05,
        sigma: 0.02,
    },
    MetricDef {
        name: "current_ratio",
        base: 1.5,
        sigma: 0.30,
    },
    MetricDef {
        name: "debt_to_equity",
        base: 0.8,
        sigma: 0.20,
    },
    MetricDef {
        name: "revenue_growth_pct",
        base: 0.03,
        sigma: 0.02,
    },
    MetricDef {
        name: "inventory_turnover",
        base: 8.0,
        sigma: 2.0,
    },
    MetricDef {
        name: "interest_rate_pct",
        base: 0.045,
        sigma: 0.01,
    },
    MetricDef {
        name: "return_on_assets_pct",
        base: 0.06,
        sigma: 0.02,
    },
    MetricDef {
        name: "days_sales_outstanding",
        base: 35.0,
        sigma: 8.0,
    },
];

/// Manufacturing industry benchmark definitions.
const MANUFACTURING_METRICS: &[MetricDef] = &[
    MetricDef {
        name: "median_revenue",
        base: 100_000_000.0,
        sigma: 0.30,
    },
    MetricDef {
        name: "gross_margin_pct",
        base: 0.30,
        sigma: 0.05,
    },
    MetricDef {
        name: "net_margin_pct",
        base: 0.07,
        sigma: 0.02,
    },
    MetricDef {
        name: "current_ratio",
        base: 1.8,
        sigma: 0.30,
    },
    MetricDef {
        name: "debt_to_equity",
        base: 0.6,
        sigma: 0.20,
    },
    MetricDef {
        name: "revenue_growth_pct",
        base: 0.04,
        sigma: 0.02,
    },
    MetricDef {
        name: "inventory_turnover",
        base: 5.0,
        sigma: 1.5,
    },
    MetricDef {
        name: "interest_rate_pct",
        base: 0.04,
        sigma: 0.01,
    },
    MetricDef {
        name: "return_on_assets_pct",
        base: 0.07,
        sigma: 0.02,
    },
    MetricDef {
        name: "asset_turnover",
        base: 1.2,
        sigma: 0.3,
    },
];

/// Financial services industry benchmark definitions.
const FINANCIAL_SERVICES_METRICS: &[MetricDef] = &[
    MetricDef {
        name: "median_revenue",
        base: 200_000_000.0,
        sigma: 0.30,
    },
    MetricDef {
        name: "net_interest_margin_pct",
        base: 0.03,
        sigma: 0.005,
    },
    MetricDef {
        name: "net_margin_pct",
        base: 0.20,
        sigma: 0.05,
    },
    MetricDef {
        name: "tier1_capital_ratio",
        base: 0.12,
        sigma: 0.02,
    },
    MetricDef {
        name: "cost_to_income_ratio",
        base: 0.55,
        sigma: 0.08,
    },
    MetricDef {
        name: "loan_to_deposit_ratio",
        base: 0.80,
        sigma: 0.10,
    },
    MetricDef {
        name: "return_on_equity_pct",
        base: 0.10,
        sigma: 0.03,
    },
    MetricDef {
        name: "non_performing_loan_pct",
        base: 0.02,
        sigma: 0.01,
    },
    MetricDef {
        name: "interest_rate_pct",
        base: 0.05,
        sigma: 0.01,
    },
    MetricDef {
        name: "revenue_growth_pct",
        base: 0.05,
        sigma: 0.03,
    },
];

/// Generic fallback benchmark definitions for unrecognized industries.
const GENERIC_METRICS: &[MetricDef] = &[
    MetricDef {
        name: "median_revenue",
        base: 75_000_000.0,
        sigma: 0.30,
    },
    MetricDef {
        name: "gross_margin_pct",
        base: 0.40,
        sigma: 0.08,
    },
    MetricDef {
        name: "net_margin_pct",
        base: 0.08,
        sigma: 0.03,
    },
    MetricDef {
        name: "current_ratio",
        base: 1.6,
        sigma: 0.30,
    },
    MetricDef {
        name: "debt_to_equity",
        base: 0.7,
        sigma: 0.20,
    },
    MetricDef {
        name: "revenue_growth_pct",
        base: 0.04,
        sigma: 0.02,
    },
    MetricDef {
        name: "return_on_assets_pct",
        base: 0.06,
        sigma: 0.02,
    },
    MetricDef {
        name: "interest_rate_pct",
        base: 0.045,
        sigma: 0.01,
    },
];

/// Generates [`IndustryBenchmark`] records with industry-specific metrics
/// perturbed around realistic base values.
pub struct IndustryBenchmarkGenerator {
    rng: ChaCha8Rng,
}

impl IndustryBenchmarkGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
        }
    }

    /// Generate benchmarks for the given industry and fiscal year.
    ///
    /// Returns 8-10 metrics depending on the industry, each perturbed
    /// around a realistic base value.
    pub fn generate(&mut self, industry: &str, fiscal_year: i32) -> Vec<IndustryBenchmark> {
        let metrics = match industry.to_lowercase().as_str() {
            "retail" => RETAIL_METRICS,
            "manufacturing" => MANUFACTURING_METRICS,
            "financial_services" | "financial services" => FINANCIAL_SERVICES_METRICS,
            _ => GENERIC_METRICS,
        };

        let period = format!("FY{fiscal_year}");

        metrics
            .iter()
            .map(|def| {
                let noise: f64 = self.rng.random_range(-1.0..1.0) * def.sigma;
                let raw = def.base * (1.0 + noise);
                // Clamp to non-negative
                let raw = if raw < 0.0 { 0.0 } else { raw };
                let value = Decimal::from_f64_retain(raw)
                    .unwrap_or(Decimal::ZERO)
                    .round_dp(4);

                IndustryBenchmark {
                    industry: industry.to_string(),
                    metric: def.name.to_string(),
                    value,
                    source: "Industry Average (Synthetic)".to_string(),
                    period: period.clone(),
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_non_empty_output() {
        let mut gen = IndustryBenchmarkGenerator::new(42);
        let benchmarks = gen.generate("retail", 2025);
        assert!(!benchmarks.is_empty(), "should produce benchmarks");
        assert!(benchmarks.len() >= 8, "should produce at least 8 metrics");
    }

    #[test]
    fn test_industry_specific_content_differs() {
        let mut gen = IndustryBenchmarkGenerator::new(42);
        let retail = gen.generate("retail", 2025);

        let mut gen2 = IndustryBenchmarkGenerator::new(42);
        let manufacturing = gen2.generate("manufacturing", 2025);

        // Different industries should have at least some different metric names
        let retail_metrics: std::collections::HashSet<_> =
            retail.iter().map(|b| b.metric.as_str()).collect();
        let mfg_metrics: std::collections::HashSet<_> =
            manufacturing.iter().map(|b| b.metric.as_str()).collect();

        assert_ne!(
            retail_metrics, mfg_metrics,
            "retail and manufacturing metrics should differ"
        );
    }

    #[test]
    fn test_financial_services_has_unique_metrics() {
        let mut gen = IndustryBenchmarkGenerator::new(99);
        let fs = gen.generate("financial_services", 2025);

        let metric_names: Vec<_> = fs.iter().map(|b| b.metric.as_str()).collect();
        assert!(
            metric_names.contains(&"net_interest_margin_pct"),
            "financial services should include net interest margin"
        );
        assert!(
            metric_names.contains(&"tier1_capital_ratio"),
            "financial services should include tier-1 capital ratio"
        );
    }

    #[test]
    fn test_source_is_synthetic() {
        let mut gen = IndustryBenchmarkGenerator::new(1);
        let benchmarks = gen.generate("retail", 2025);
        for b in &benchmarks {
            assert_eq!(b.source, "Industry Average (Synthetic)");
        }
    }

    #[test]
    fn test_period_label() {
        let mut gen = IndustryBenchmarkGenerator::new(1);
        let benchmarks = gen.generate("retail", 2026);
        for b in &benchmarks {
            assert_eq!(b.period, "FY2026");
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let mut gen1 = IndustryBenchmarkGenerator::new(555);
        let b1 = gen1.generate("manufacturing", 2025);

        let mut gen2 = IndustryBenchmarkGenerator::new(555);
        let b2 = gen2.generate("manufacturing", 2025);

        assert_eq!(b1.len(), b2.len());
        for (a, b) in b1.iter().zip(b2.iter()) {
            assert_eq!(a.metric, b.metric);
            assert_eq!(a.value, b.value);
        }
    }

    #[test]
    fn test_values_are_non_negative() {
        let mut gen = IndustryBenchmarkGenerator::new(42);
        for industry in &[
            "retail",
            "manufacturing",
            "financial_services",
            "healthcare",
        ] {
            let benchmarks = gen.generate(industry, 2025);
            for b in &benchmarks {
                assert!(
                    b.value >= Decimal::ZERO,
                    "benchmark value should be non-negative: {} = {}",
                    b.metric,
                    b.value
                );
            }
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = IndustryBenchmarkGenerator::new(42);
        let benchmarks = gen.generate("retail", 2025);
        let json = serde_json::to_string(&benchmarks).expect("serialize");
        let parsed: Vec<IndustryBenchmark> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(benchmarks.len(), parsed.len());
        for (orig, rt) in benchmarks.iter().zip(parsed.iter()) {
            assert_eq!(orig.metric, rt.metric);
            assert_eq!(orig.value, rt.value);
            assert_eq!(orig.industry, rt.industry);
        }
    }

    #[test]
    fn test_unknown_industry_falls_back_to_generic() {
        let mut gen = IndustryBenchmarkGenerator::new(42);
        let benchmarks = gen.generate("space_exploration", 2025);
        assert!(
            !benchmarks.is_empty(),
            "unknown industry should still produce output"
        );
        // Generic set has 8 metrics
        assert_eq!(benchmarks.len(), 8);
    }
}
