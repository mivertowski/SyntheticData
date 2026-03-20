//! Materiality benchmark calculation generator per ISA 320.
//!
//! Generates one `MaterialityCalculation` per entity per period.  The benchmark
//! is selected based on the entity's financial profile and the appropriate
//! percentage is drawn from ranges consistent with professional practice:
//!
//! | Benchmark     | Typical range | Rationale                              |
//! |---------------|---------------|----------------------------------------|
//! | Pre-tax income | 3–7%         | Profit-oriented entities               |
//! | Revenue        | 0.5–1%       | Thin-margin or loss-making entities    |
//! | Total assets   | 0.5–1%       | Asset-intensive industries             |
//! | Equity         | 1–2%         | Non-profit / equity-focused users      |
//!
//! Performance materiality is set at 65% of overall (within the ISA 320.11
//! range of 50–75%).  Clearly trivial is set at 5% of overall.
//!
//! Normalisation adjustments are generated when reported pre-tax earnings are
//! unusually volatile (swing > 50% from an estimated "normal").

use datasynth_core::models::audit::materiality_calculation::{
    AdjustmentType, MaterialityBenchmark, MaterialityCalculation, NormalizationAdjustment,
    NormalizedEarnings,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Financial data extracted from the trial balance / JE stream, used to
/// select the appropriate benchmark and compute materiality amounts.
#[derive(Debug, Clone)]
pub struct MaterialityInput {
    /// Entity / company code.
    pub entity_code: String,
    /// Human-readable period descriptor (e.g. "FY2024").
    pub period: String,
    /// Revenue for the period (zero or positive).
    pub revenue: Decimal,
    /// Pre-tax income for the period (may be negative for a loss).
    pub pretax_income: Decimal,
    /// Total assets at period end.
    pub total_assets: Decimal,
    /// Total equity at period end.
    pub equity: Decimal,
    /// Gross profit = revenue − cost of goods sold.
    pub gross_profit: Decimal,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the materiality generator.
#[derive(Debug, Clone)]
pub struct MaterialityGeneratorConfig {
    /// Performance materiality as a fraction of overall materiality.
    /// Must be in [0.50, 0.75] per ISA 320 guidance.
    pub pm_percentage: Decimal,
    /// Minimum benchmark amount — prevents immaterial overall materiality
    /// for micro-entities.
    pub minimum_overall_materiality: Decimal,
}

impl Default for MaterialityGeneratorConfig {
    fn default() -> Self {
        Self {
            pm_percentage: dec!(0.65),
            minimum_overall_materiality: dec!(5_000),
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 320 materiality calculations.
pub struct MaterialityGenerator {
    rng: ChaCha8Rng,
    config: MaterialityGeneratorConfig,
}

impl MaterialityGenerator {
    /// Create a new generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x320), // discriminator for ISA 320
            config: MaterialityGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: MaterialityGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x320),
            config,
        }
    }

    /// Generate a materiality calculation for a single entity.
    pub fn generate(&mut self, input: &MaterialityInput) -> MaterialityCalculation {
        let (benchmark, benchmark_amount, benchmark_pct, rationale) = self.select_benchmark(input);

        // Apply the minimum overall materiality floor after benchmark selection
        let raw_overall = benchmark_amount * benchmark_pct;
        let effective_overall = raw_overall.max(self.config.minimum_overall_materiality);

        // If the floor was applied, adjust the percentage so the formula stays consistent
        let effective_pct = if benchmark_amount > Decimal::ZERO {
            effective_overall / benchmark_amount
        } else {
            benchmark_pct
        };

        let normalized_earnings = self.maybe_generate_normalization(input);

        MaterialityCalculation::new(
            &input.entity_code,
            &input.period,
            benchmark,
            benchmark_amount,
            effective_pct,
            self.config.pm_percentage,
            normalized_earnings,
            &rationale,
        )
    }

    /// Generate materiality calculations for a batch of entities.
    pub fn generate_batch(&mut self, inputs: &[MaterialityInput]) -> Vec<MaterialityCalculation> {
        inputs.iter().map(|i| self.generate(i)).collect()
    }

    // -----------------------------------------------------------------------
    // Benchmark selection
    // -----------------------------------------------------------------------

    /// Select the appropriate benchmark and percentage for the entity.
    ///
    /// Decision rules:
    /// 1. If pre-tax income is positive and > 5% of revenue → PretaxIncome at 3–7%
    /// 2. If pre-tax income is negative or < 2% of revenue (thin margin) → Revenue at 0.5–1%
    /// 3. If total assets dominate (assets > 10× revenue, e.g. financial institutions) → TotalAssets at 0.5–1%
    /// 4. If equity is the primary concern → Equity at 1–2%
    /// 5. Default fallback → Revenue at 0.5–1%
    fn select_benchmark(
        &mut self,
        input: &MaterialityInput,
    ) -> (MaterialityBenchmark, Decimal, Decimal, String) {
        // Check if entity is asset-heavy (assets > 10× revenue)
        let asset_heavy =
            input.revenue > Decimal::ZERO && input.total_assets > input.revenue * dec!(10);

        // Check if profitable (positive pre-tax income and > 5% of revenue)
        let healthy_profit = input.pretax_income > Decimal::ZERO
            && (input.revenue == Decimal::ZERO || input.pretax_income > input.revenue * dec!(0.05));

        // Thin margin: profitable but < 2% of revenue (or small absolute)
        let thin_margin = input.pretax_income > Decimal::ZERO
            && input.revenue > Decimal::ZERO
            && input.pretax_income < input.revenue * dec!(0.02);

        if asset_heavy && input.total_assets > Decimal::ZERO {
            // Asset-intensive entities (banks, real estate, investment firms)
            let pct = self.random_pct(dec!(0.005), dec!(0.010));
            let rationale = format!(
                "Total assets selected as benchmark (asset-intensive entity; assets {:.0}× revenue). \
                 {:.2}% of total assets applied.",
                (input.total_assets / input.revenue.max(dec!(1))).round(),
                pct * dec!(100)
            );
            (
                MaterialityBenchmark::TotalAssets,
                input.total_assets,
                pct,
                rationale,
            )
        } else if healthy_profit && !thin_margin {
            // Standard profitable entity — pre-tax income benchmark
            let pct = self.random_pct(dec!(0.03), dec!(0.07));
            let rationale = format!(
                "Pre-tax income selected as benchmark (profit-making entity with healthy margins). \
                 {:.0}% applied.",
                pct * dec!(100)
            );
            (
                MaterialityBenchmark::PretaxIncome,
                input.pretax_income,
                pct,
                rationale,
            )
        } else if input.pretax_income <= Decimal::ZERO || thin_margin {
            // Loss-making or thin-margin entity — revenue benchmark
            let pct = self.random_pct(dec!(0.005), dec!(0.010));
            let rationale =
                format!(
                "Revenue selected as benchmark (entity has {} pre-tax income; revenue provides \
                 more stable benchmark). {:.2}% applied.",
                if input.pretax_income <= Decimal::ZERO { "negative" } else { "thin" },
                pct * dec!(100)
            );
            (
                MaterialityBenchmark::Revenue,
                input.revenue.max(dec!(1)),
                pct,
                rationale,
            )
        } else if input.equity > Decimal::ZERO {
            // Equity-focused (e.g. non-profit, investment entity)
            let pct = self.random_pct(dec!(0.01), dec!(0.02));
            let rationale = format!(
                "Equity selected as benchmark (equity-focused entity). {:.0}% applied.",
                pct * dec!(100)
            );
            (MaterialityBenchmark::Equity, input.equity, pct, rationale)
        } else {
            // Fallback — revenue
            let pct = self.random_pct(dec!(0.005), dec!(0.010));
            let rationale = format!(
                "Revenue selected as default benchmark. {:.2}% applied.",
                pct * dec!(100)
            );
            (
                MaterialityBenchmark::Revenue,
                input.revenue.max(dec!(1)),
                pct,
                rationale,
            )
        }
    }

    // -----------------------------------------------------------------------
    // Normalization
    // -----------------------------------------------------------------------

    /// Optionally generate 0–2 normalization adjustments.
    ///
    /// Adjustments are generated when pre-tax income is unusually volatile —
    /// defined as abs(pre-tax income) being very small relative to revenue
    /// (i.e. near break-even) or when the absolute earnings swing indicates
    /// a non-recurring item.
    fn maybe_generate_normalization(
        &mut self,
        input: &MaterialityInput,
    ) -> Option<NormalizedEarnings> {
        // Only normalize when using pre-tax income as a benchmark
        if input.pretax_income <= Decimal::ZERO {
            return None;
        }
        // Check for "unusual" earnings: earnings < 3% of revenue suggests volatility
        let is_unusual =
            input.revenue > Decimal::ZERO && input.pretax_income < input.revenue * dec!(0.03);
        if !is_unusual {
            return None;
        }
        // 60% chance of generating adjustments when earnings are unusual
        let roll: f64 = self.rng.random();
        if roll > 0.60 {
            return None;
        }

        let n_adjustments: u32 = self.rng.random_range(1u32..=2);
        let mut adjustments = Vec::new();

        for i in 0..n_adjustments {
            let (description, amount, adj_type) = self.random_adjustment(input, i);
            adjustments.push(NormalizationAdjustment {
                description,
                amount,
                adjustment_type: adj_type,
            });
        }

        let ne = NormalizedEarnings::new(input.pretax_income, adjustments);
        Some(ne)
    }

    /// Generate a single normalization adjustment.
    fn random_adjustment(
        &mut self,
        input: &MaterialityInput,
        index: u32,
    ) -> (String, Decimal, AdjustmentType) {
        let templates = [
            (
                "Restructuring charge — one-time plant closure costs",
                AdjustmentType::NonRecurring,
                0.01_f64,
            ),
            (
                "Impairment of goodwill — non-recurring write-down",
                AdjustmentType::NonRecurring,
                0.02_f64,
            ),
            (
                "Gain on disposal of subsidiary — non-recurring",
                AdjustmentType::Extraordinary,
                -0.015_f64,
            ),
            (
                "Litigation settlement — one-time charge",
                AdjustmentType::NonRecurring,
                0.008_f64,
            ),
            (
                "COVID-19 related costs — non-recurring operational impact",
                AdjustmentType::NonRecurring,
                0.005_f64,
            ),
        ];

        let idx =
            (index as usize + self.rng.random_range(0usize..templates.len())) % templates.len();
        let (desc, adj_type, revenue_frac) = &templates[idx];
        let base = input.revenue.max(dec!(100_000));
        let frac = Decimal::try_from(*revenue_frac).unwrap_or(dec!(0.01));
        let amount = (base * frac).round_dp(0);

        (desc.to_string(), amount, *adj_type)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Sample a Decimal percentage in the closed range [lo, hi].
    fn random_pct(&mut self, lo: Decimal, hi: Decimal) -> Decimal {
        use rust_decimal::prelude::ToPrimitive;
        let lo_f = lo.to_f64().unwrap_or(0.005);
        let hi_f = hi.to_f64().unwrap_or(0.010);
        let val = self.rng.random_range(lo_f..=hi_f);
        // Round to 4 decimal places
        Decimal::try_from(val).unwrap_or(lo).round_dp(4)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_input() -> MaterialityInput {
        MaterialityInput {
            entity_code: "C001".into(),
            period: "FY2024".into(),
            revenue: dec!(10_000_000),
            pretax_income: dec!(1_000_000),
            total_assets: dec!(8_000_000),
            equity: dec!(4_000_000),
            gross_profit: dec!(3_500_000),
        }
    }

    #[test]
    fn materiality_formula_holds() {
        let mut gen = MaterialityGenerator::new(42);
        let calc = gen.generate(&sample_input());
        // overall_materiality = benchmark_amount × benchmark_percentage
        let expected = (calc.benchmark_amount * calc.benchmark_percentage).round_dp(10);
        assert_eq!(
            calc.overall_materiality.round_dp(10),
            expected,
            "overall_materiality must equal benchmark_amount × benchmark_percentage"
        );
    }

    #[test]
    fn pm_is_between_50_and_75_percent_of_overall() {
        let mut gen = MaterialityGenerator::new(42);
        let calc = gen.generate(&sample_input());
        let ratio = calc.performance_materiality / calc.overall_materiality;
        assert!(
            ratio >= dec!(0.50),
            "PM ({}) < 50% of overall ({})",
            calc.performance_materiality,
            calc.overall_materiality
        );
        assert!(
            ratio <= dec!(0.75),
            "PM ({}) > 75% of overall ({})",
            calc.performance_materiality,
            calc.overall_materiality
        );
    }

    #[test]
    fn clearly_trivial_is_five_percent_of_overall() {
        let mut gen = MaterialityGenerator::new(42);
        let calc = gen.generate(&sample_input());
        let expected_ct = calc.overall_materiality * dec!(0.05);
        assert_eq!(calc.clearly_trivial, expected_ct);
    }

    #[test]
    fn sad_nominal_is_five_percent_of_overall() {
        let mut gen = MaterialityGenerator::new(42);
        let calc = gen.generate(&sample_input());
        // SAD nominal = 5% of overall materiality (ISA 450 guidance).
        let expected = calc.overall_materiality * dec!(0.05);
        assert_eq!(calc.sad_nominal, expected);
    }

    #[test]
    fn minimum_materiality_floor_applied() {
        let mut gen = MaterialityGenerator::new(42);
        let tiny_input = MaterialityInput {
            entity_code: "TINY".into(),
            period: "FY2024".into(),
            revenue: dec!(10_000),
            pretax_income: dec!(500),
            total_assets: dec!(5_000),
            equity: dec!(2_000),
            gross_profit: dec!(2_000),
        };
        let calc = gen.generate(&tiny_input);
        assert!(
            calc.overall_materiality >= dec!(5_000),
            "Minimum floor should apply; got {}",
            calc.overall_materiality
        );
    }

    #[test]
    fn asset_heavy_entity_uses_total_assets() {
        let mut gen = MaterialityGenerator::new(42);
        let asset_input = MaterialityInput {
            entity_code: "BANK".into(),
            period: "FY2024".into(),
            revenue: dec!(1_000_000),
            pretax_income: dec!(200_000),
            total_assets: dec!(50_000_000), // 50× revenue → asset-heavy
            equity: dec!(5_000_000),
            gross_profit: dec!(800_000),
        };
        let calc = gen.generate(&asset_input);
        assert_eq!(
            calc.benchmark,
            MaterialityBenchmark::TotalAssets,
            "Asset-heavy entity should use TotalAssets benchmark"
        );
    }

    #[test]
    fn loss_making_entity_uses_revenue() {
        let mut gen = MaterialityGenerator::new(42);
        let loss_input = MaterialityInput {
            entity_code: "LOSS".into(),
            period: "FY2024".into(),
            revenue: dec!(5_000_000),
            pretax_income: dec!(-200_000),
            total_assets: dec!(3_000_000),
            equity: dec!(1_000_000),
            gross_profit: dec!(500_000),
        };
        let calc = gen.generate(&loss_input);
        assert_eq!(
            calc.benchmark,
            MaterialityBenchmark::Revenue,
            "Loss-making entity should use Revenue benchmark"
        );
    }
}
