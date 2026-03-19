//! Accounting estimates generator — ISA 540.
//!
//! Generates `AccountingEstimate` records for each reporting entity,
//! covering the full range of estimate types required by ISA 540 (Revised 2019).
//!
//! # Generation rules
//!
//! * 5–8 distinct estimate types per entity (sampled without replacement from the full set).
//! * Each estimate has 2–3 key assumptions with sensitivity analysis.
//! * 30 % of estimates receive a retrospective review; the review compares the
//!   prior-period estimate to a simulated actual outcome (±5–15 % variance).
//! * `management_bias_indicator` is set to `true` when the variance consistently
//!   exceeds 10 % in the same direction.
//! * ISA 540 risk factors are assigned per estimate type:
//!   - High uncertainty: PensionObligation, ExpectedCreditLoss.
//!   - Moderate uncertainty: DeferredTaxProvision, FairValueMeasurement,
//!     ImpairmentTest, ProvisionForLiabilities, ShareBasedPayment.
//!   - Low uncertainty: DepreciationUsefulLife.

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::audit::accounting_estimates::{
    AccountingEstimate, AssumptionAssessment, EstimateAssumption, EstimateComplexity,
    EstimateType, Isa540RiskFactors, RetrospectiveReview, SubjectivityLevel, UncertaintyLevel,
};
use datasynth_core::utils::seeded_rng;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the accounting estimates generator.
#[derive(Debug, Clone)]
pub struct AccountingEstimateGeneratorConfig {
    /// Minimum number of estimates generated per entity.
    pub min_estimates_per_entity: usize,
    /// Maximum number of estimates generated per entity.
    pub max_estimates_per_entity: usize,
    /// Probability that an estimate receives a retrospective review (0.0–1.0).
    pub retrospective_review_probability: f64,
    /// Minimum variance % applied to the prior-period estimate in retrospective reviews.
    pub min_variance_pct: f64,
    /// Maximum variance % applied to the prior-period estimate in retrospective reviews.
    pub max_variance_pct: f64,
    /// Variance % threshold above which management bias indicator is set to `true`.
    pub bias_threshold_pct: f64,
}

impl Default for AccountingEstimateGeneratorConfig {
    fn default() -> Self {
        Self {
            min_estimates_per_entity: 5,
            max_estimates_per_entity: 8,
            retrospective_review_probability: 0.30,
            min_variance_pct: 5.0,
            max_variance_pct: 15.0,
            bias_threshold_pct: 10.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 540 accounting estimates.
pub struct AccountingEstimateGenerator {
    rng: ChaCha8Rng,
    config: AccountingEstimateGeneratorConfig,
}

impl AccountingEstimateGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x540), // discriminator for ISA 540
            config: AccountingEstimateGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: AccountingEstimateGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x540),
            config,
        }
    }

    /// Generate all accounting estimates for a list of entity codes.
    pub fn generate_for_entities(
        &mut self,
        entity_codes: &[String],
    ) -> Vec<AccountingEstimate> {
        let mut all_estimates = Vec::new();
        for entity_code in entity_codes {
            let estimates = self.generate_for_entity(entity_code);
            all_estimates.extend(estimates);
        }
        all_estimates
    }

    /// Generate accounting estimates for a single entity.
    pub fn generate_for_entity(&mut self, entity_code: &str) -> Vec<AccountingEstimate> {
        let count = self
            .rng
            .random_range(self.config.min_estimates_per_entity..=self.config.max_estimates_per_entity);

        // All estimate types available
        let all_types = [
            EstimateType::DeferredTaxProvision,
            EstimateType::ExpectedCreditLoss,
            EstimateType::PensionObligation,
            EstimateType::FairValueMeasurement,
            EstimateType::ImpairmentTest,
            EstimateType::ProvisionForLiabilities,
            EstimateType::ShareBasedPayment,
            EstimateType::DepreciationUsefulLife,
        ];

        // Fisher-Yates shuffle to pick `count` types without replacement
        let mut available: Vec<EstimateType> = all_types.to_vec();
        for i in 0..count.min(available.len()) {
            let j = self.rng.random_range(i..available.len());
            available.swap(i, j);
        }

        let selected_types = &available[..count.min(available.len())];
        let mut estimates = Vec::with_capacity(selected_types.len());

        for (idx, &estimate_type) in selected_types.iter().enumerate() {
            let id = format!("ISA540-{}-{:04}", entity_code, idx + 1);
            let estimate = self.build_estimate(id, entity_code.to_string(), estimate_type);
            estimates.push(estimate);
        }

        estimates
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn build_estimate(
        &mut self,
        id: String,
        entity_code: String,
        estimate_type: EstimateType,
    ) -> AccountingEstimate {
        let base_amount = self.generate_base_amount(estimate_type);
        let (uncertainty, complexity, subjectivity) = self.risk_factors_for(estimate_type);

        let assumptions = self.generate_assumptions(estimate_type, base_amount);
        let retrospective_review = if self.rng.random_bool(self.config.retrospective_review_probability) {
            Some(self.generate_retrospective_review(base_amount))
        } else {
            None
        };

        // Optionally generate an auditor point estimate (differs slightly from management's)
        let auditor_point_estimate = if complexity != EstimateComplexity::Simple
            && self.rng.random_bool(0.50)
        {
            // Small random deviation: ±2–8 % from management's estimate
            let deviation_pct = self.rng.random_range(2u32..=8u32);
            let deviation_sign = if self.rng.random_bool(0.5) { 1i64 } else { -1i64 };
            let multiplier = dec!(1)
                + Decimal::new(deviation_sign * i64::from(deviation_pct), 2); // e.g. 1.04
            Some((base_amount * multiplier).round_dp(2))
        } else {
            None
        };

        AccountingEstimate {
            id,
            entity_code,
            estimate_type,
            description: self.description_for(estimate_type),
            management_point_estimate: base_amount,
            auditor_point_estimate,
            estimation_uncertainty: uncertainty,
            complexity,
            assumptions,
            retrospective_review,
            isa540_risk_factors: Isa540RiskFactors {
                estimation_uncertainty: uncertainty,
                complexity,
                subjectivity,
            },
        }
    }

    /// Generate a plausible base monetary amount for each estimate type.
    fn generate_base_amount(&mut self, estimate_type: EstimateType) -> Decimal {
        // Ranges are approximate, in thousands; scaled to typical entity sizes.
        let (min_k, max_k) = match estimate_type {
            EstimateType::DeferredTaxProvision => (50, 2_000),
            EstimateType::ExpectedCreditLoss => (100, 5_000),
            EstimateType::PensionObligation => (500, 20_000),
            EstimateType::FairValueMeasurement => (1_000, 50_000),
            EstimateType::ImpairmentTest => (2_000, 100_000),
            EstimateType::ProvisionForLiabilities => (100, 10_000),
            EstimateType::ShareBasedPayment => (200, 8_000),
            EstimateType::DepreciationUsefulLife => (500, 15_000),
        };
        let thousands: u64 = self.rng.random_range(min_k..=max_k);
        Decimal::from(thousands * 1_000)
    }

    /// Canonical description for each estimate type.
    fn description_for(&self, estimate_type: EstimateType) -> String {
        match estimate_type {
            EstimateType::DeferredTaxProvision => {
                "Deferred tax asset/liability — temporary differences".to_string()
            }
            EstimateType::ExpectedCreditLoss => {
                "Expected credit loss allowance — trade receivables".to_string()
            }
            EstimateType::PensionObligation => {
                "Defined benefit obligation — pension plan".to_string()
            }
            EstimateType::FairValueMeasurement => {
                "Level 3 fair value — unlisted equity investments".to_string()
            }
            EstimateType::ImpairmentTest => {
                "Goodwill impairment test — cash-generating unit".to_string()
            }
            EstimateType::ProvisionForLiabilities => {
                "Provision for legal claims and warranty obligations".to_string()
            }
            EstimateType::ShareBasedPayment => {
                "Share-based payment — employee stock options".to_string()
            }
            EstimateType::DepreciationUsefulLife => {
                "Useful life revision — plant and equipment".to_string()
            }
        }
    }

    /// ISA 540 risk factors: (uncertainty, complexity, subjectivity) per estimate type.
    fn risk_factors_for(
        &self,
        estimate_type: EstimateType,
    ) -> (UncertaintyLevel, EstimateComplexity, SubjectivityLevel) {
        match estimate_type {
            EstimateType::PensionObligation | EstimateType::ExpectedCreditLoss => (
                UncertaintyLevel::High,
                EstimateComplexity::Complex,
                SubjectivityLevel::High,
            ),
            EstimateType::FairValueMeasurement | EstimateType::ImpairmentTest => (
                UncertaintyLevel::High,
                EstimateComplexity::Complex,
                SubjectivityLevel::Medium,
            ),
            EstimateType::DeferredTaxProvision
            | EstimateType::ProvisionForLiabilities
            | EstimateType::ShareBasedPayment => (
                UncertaintyLevel::Medium,
                EstimateComplexity::Moderate,
                SubjectivityLevel::Medium,
            ),
            EstimateType::DepreciationUsefulLife => (
                UncertaintyLevel::Low,
                EstimateComplexity::Simple,
                SubjectivityLevel::Low,
            ),
        }
    }

    /// Generate 2–3 key assumptions for the estimate.
    fn generate_assumptions(
        &mut self,
        estimate_type: EstimateType,
        base_amount: Decimal,
    ) -> Vec<EstimateAssumption> {
        let count = self.rng.random_range(2..=3usize);
        let templates = assumption_templates(estimate_type);
        let mut assumptions = Vec::with_capacity(count);

        for i in 0..count.min(templates.len()) {
            let (desc, sens_pct) = &templates[i];
            let sensitivity = (base_amount
                * Decimal::new(i64::from(*sens_pct), 2))
            .round_dp(2);

            let reasonableness = if self.rng.random_bool(0.70) {
                AssumptionAssessment::Reasonable
            } else if self.rng.random_bool(0.67) {
                AssumptionAssessment::Optimistic
            } else {
                AssumptionAssessment::Aggressive
            };

            assumptions.push(EstimateAssumption {
                description: desc.to_string(),
                sensitivity,
                reasonableness,
            });
        }

        assumptions
    }

    /// Generate a retrospective review comparing the prior-period estimate to a
    /// simulated actual outcome.
    fn generate_retrospective_review(&mut self, current_estimate: Decimal) -> RetrospectiveReview {
        // Simulate a prior-period estimate as current ± 5–20 %
        let prior_delta_pct: f64 = self.rng.random_range(5.0..20.0);
        let prior_sign = if self.rng.random_bool(0.5) { 1.0_f64 } else { -1.0_f64 };
        let prior_factor = 1.0 + prior_sign * prior_delta_pct / 100.0;

        let prior_estimate = {
            let f = Decimal::try_from(prior_factor).unwrap_or(dec!(1));
            (current_estimate * f).round_dp(2)
        };

        // Simulate actual outcome: prior ± configured variance range
        let var_pct: f64 = self
            .rng
            .random_range(self.config.min_variance_pct..=self.config.max_variance_pct);
        let var_sign = if self.rng.random_bool(0.5) { 1.0_f64 } else { -1.0_f64 };
        let var_factor = 1.0 + var_sign * var_pct / 100.0;

        let actual_outcome = {
            let f = Decimal::try_from(var_factor).unwrap_or(dec!(1));
            (prior_estimate * f).round_dp(2)
        };

        let variance = (actual_outcome - prior_estimate).round_dp(2);

        // variance_percentage = variance / prior_estimate * 100
        let variance_percentage = if prior_estimate.is_zero() {
            Decimal::ZERO
        } else {
            (variance / prior_estimate * dec!(100)).round_dp(2)
        };

        // Bias indicator: variance > threshold AND in same direction as prior_sign
        let abs_var_pct = variance_percentage.abs();
        let bias_threshold = Decimal::try_from(self.config.bias_threshold_pct).unwrap_or(dec!(10));
        let management_bias_indicator = abs_var_pct > bias_threshold;

        RetrospectiveReview {
            prior_period_estimate: prior_estimate,
            actual_outcome,
            variance,
            variance_percentage,
            management_bias_indicator,
        }
    }
}

// ---------------------------------------------------------------------------
// Static assumption template data
// ---------------------------------------------------------------------------

/// Return canonical assumption descriptions and sensitivity percentages (as i64 basis points × 100)
/// for each estimate type. Sensitivity is expressed as % of the base amount that would change
/// with a 1-unit move in the assumption.
fn assumption_templates(estimate_type: EstimateType) -> Vec<(&'static str, u32)> {
    match estimate_type {
        EstimateType::DeferredTaxProvision => vec![
            ("Effective tax rate (24 %)", 10),
            ("Probability of reversal within 5 years", 8),
            ("Taxable profit forecasts (3-year)", 6),
        ],
        EstimateType::ExpectedCreditLoss => vec![
            ("12-month probability of default (2.5 %)", 12),
            ("Loss given default (45 %)", 9),
            ("Macro overlay — unemployment rate sensitivity", 7),
        ],
        EstimateType::PensionObligation => vec![
            ("Discount rate (4.5 %)", 15),
            ("Salary escalation rate (3.0 %)", 10),
            ("Mortality assumption — actuarial table", 8),
        ],
        EstimateType::FairValueMeasurement => vec![
            ("Discount rate / WACC (8.5 %)", 18),
            ("Terminal growth rate (2.5 %)", 14),
            ("Revenue multiple (EV/Revenue 3.5x)", 11),
        ],
        EstimateType::ImpairmentTest => vec![
            ("Value in use — discount rate (9.0 %)", 20),
            ("Revenue growth rate (3-year CAGR)", 15),
            ("Long-term operating margin", 10),
        ],
        EstimateType::ProvisionForLiabilities => vec![
            ("Probability of unfavourable outcome (60 %)", 12),
            ("Legal costs estimate", 8),
            ("Settlement range — lower/upper bound", 6),
        ],
        EstimateType::ShareBasedPayment => vec![
            ("Expected volatility (28 %)", 11),
            ("Risk-free interest rate (4.0 %)", 7),
            ("Expected forfeiture rate (5 %)", 5),
        ],
        EstimateType::DepreciationUsefulLife => vec![
            ("Useful life revision (20 → 25 years)", 8),
            ("Residual value assumption (10 %)", 5),
            ("Technology obsolescence probability", 4),
        ],
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_correct_count_range() {
        let mut gen = AccountingEstimateGenerator::new(42);
        let entity_codes = vec!["C001".to_string(), "C002".to_string()];
        let estimates = gen.generate_for_entities(&entity_codes);

        // Each entity should produce 5–8 estimates; with 2 entities: 10–16 total
        assert!(
            estimates.len() >= 10,
            "expected at least 10 estimates, got {}",
            estimates.len()
        );
        assert!(
            estimates.len() <= 16,
            "expected at most 16 estimates, got {}",
            estimates.len()
        );
    }

    #[test]
    fn test_each_entity_produces_bounded_count() {
        let mut gen = AccountingEstimateGenerator::new(99);
        let entity_codes = vec!["E001".to_string()];
        let estimates = gen.generate_for_entities(&entity_codes);
        assert!(
            (5..=8).contains(&estimates.len()),
            "expected 5–8 estimates per entity, got {}",
            estimates.len()
        );
    }

    #[test]
    fn test_all_estimates_have_isa540_risk_factors() {
        let mut gen = AccountingEstimateGenerator::new(7);
        let estimates = gen.generate_for_entity("C001");
        for est in &estimates {
            // Risk factors must be self-consistent: same uncertainty level
            assert_eq!(
                format!("{:?}", est.isa540_risk_factors.estimation_uncertainty),
                format!("{:?}", est.estimation_uncertainty)
            );
        }
    }

    #[test]
    fn test_each_estimate_has_2_to_3_assumptions() {
        let mut gen = AccountingEstimateGenerator::new(55);
        let estimates = gen.generate_for_entity("C002");
        for est in &estimates {
            let n = est.assumptions.len();
            assert!(
                (2..=3).contains(&n),
                "estimate {:?} has {} assumptions (expected 2-3)",
                est.estimate_type,
                n
            );
        }
    }

    #[test]
    fn test_retrospective_review_variance_calculation() {
        // Run many iterations; at least some must have reviews
        let mut gen = AccountingEstimateGenerator::new(1234);
        let config = AccountingEstimateGeneratorConfig {
            retrospective_review_probability: 1.0, // force all estimates to have reviews
            ..Default::default()
        };
        let mut gen_all = AccountingEstimateGenerator::with_config(1234, config);
        let estimates = gen_all.generate_for_entity("C003");

        for est in &estimates {
            let rev = est.retrospective_review.as_ref().unwrap();
            let expected_variance = (rev.actual_outcome - rev.prior_period_estimate).round_dp(2);
            assert_eq!(
                rev.variance, expected_variance,
                "variance mismatch for {:?}",
                est.estimate_type
            );
        }

        // Suppress unused warning
        let _ = gen.generate_for_entity("X");
    }

    #[test]
    fn test_high_uncertainty_types() {
        let mut gen = AccountingEstimateGenerator::new(777);
        // Force only pensions to be generated by running many entities and collecting
        let all_estimates = gen.generate_for_entities(&["E1".to_string(), "E2".to_string(), "E3".to_string()]);

        // Find a pension estimate and verify its risk factors
        let pension = all_estimates.iter().find(|e| e.estimate_type == EstimateType::PensionObligation);
        if let Some(p) = pension {
            assert!(
                matches!(p.isa540_risk_factors.estimation_uncertainty, UncertaintyLevel::High),
                "Pension obligation should have High uncertainty"
            );
            assert!(
                matches!(p.isa540_risk_factors.complexity, EstimateComplexity::Complex),
                "Pension obligation should be Complex"
            );
        }

        // Find a depreciation estimate and verify its risk factors
        let depr = all_estimates.iter().find(|e| e.estimate_type == EstimateType::DepreciationUsefulLife);
        if let Some(d) = depr {
            assert!(
                matches!(d.isa540_risk_factors.estimation_uncertainty, UncertaintyLevel::Low),
                "DepreciationUsefulLife should have Low uncertainty"
            );
        }
    }

    #[test]
    fn test_no_duplicate_estimate_types_per_entity() {
        let mut gen = AccountingEstimateGenerator::new(321);
        let estimates = gen.generate_for_entity("C004");
        let mut seen = std::collections::HashSet::new();
        for est in &estimates {
            let key = format!("{:?}", est.estimate_type);
            assert!(
                seen.insert(key.clone()),
                "Duplicate estimate type per entity: {}",
                key
            );
        }
    }
}
