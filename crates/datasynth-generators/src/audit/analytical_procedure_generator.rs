//! Analytical procedure generator per ISA 520.
//!
//! Generates `AnalyticalProcedureResult` records distributed across the three
//! audit phases (Planning, Substantive, FinalReview) with realistic expectation /
//! actual value pairs and conclusion distributions.

use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    AnalyticalConclusion, AnalyticalMethod, AnalyticalPhase, AnalyticalProcedureResult,
    AuditEngagement,
};

/// Configuration for the analytical procedure generator (ISA 520).
#[derive(Debug, Clone)]
pub struct AnalyticalProcedureGeneratorConfig {
    /// Number of procedures to generate per engagement (min, max)
    pub procedures_per_engagement: (u32, u32),
    /// Fraction of procedures that conclude `Consistent`
    pub consistent_ratio: f64,
    /// Fraction of procedures that conclude `ExplainedVariance`
    pub explained_ratio: f64,
    /// Fraction of procedures that conclude `FurtherInvestigation`
    pub further_ratio: f64,
    /// Fraction of procedures that conclude `PossibleMisstatement`
    pub misstatement_ratio: f64,
}

impl Default for AnalyticalProcedureGeneratorConfig {
    fn default() -> Self {
        Self {
            procedures_per_engagement: (8, 15),
            consistent_ratio: 0.60,
            explained_ratio: 0.25,
            further_ratio: 0.10,
            misstatement_ratio: 0.05,
        }
    }
}

/// Generator for `AnalyticalProcedureResult` records per ISA 520.
pub struct AnalyticalProcedureGenerator {
    /// Seeded random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: AnalyticalProcedureGeneratorConfig,
}

impl AnalyticalProcedureGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: AnalyticalProcedureGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: AnalyticalProcedureGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generate analytical procedures for an engagement.
    ///
    /// # Arguments
    /// * `engagement`    — The audit engagement these procedures belong to.
    /// * `account_codes` — GL account codes (or area names) to associate procedures with.
    ///   When empty, generic area names are used.
    pub fn generate_procedures(
        &mut self,
        engagement: &AuditEngagement,
        account_codes: &[String],
    ) -> Vec<AnalyticalProcedureResult> {
        let count = self.rng.random_range(
            self.config.procedures_per_engagement.0..=self.config.procedures_per_engagement.1,
        ) as usize;

        // Phase distribution: Planning 20%, Substantive 60%, FinalReview 20%.
        let planning_count = (count as f64 * 0.20).round() as usize;
        let final_count = (count as f64 * 0.20).round() as usize;
        let substantive_count = count.saturating_sub(planning_count + final_count).max(1);

        // Build the phase sequence.
        let mut phases: Vec<AnalyticalPhase> = Vec::with_capacity(count);
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::Planning,
            planning_count,
        ));
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::Substantive,
            substantive_count,
        ));
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::FinalReview,
            final_count,
        ));

        // Fallback area names when no account codes are provided.
        let default_areas = [
            "Revenue",
            "Cost of Sales",
            "Operating Expenses",
            "Accounts Receivable",
            "Inventory",
            "Payroll Expense",
            "Interest Expense",
            "Depreciation",
            "Accounts Payable",
            "Income Tax Expense",
        ];

        let all_methods = [
            AnalyticalMethod::TrendAnalysis,
            AnalyticalMethod::RatioAnalysis,
            AnalyticalMethod::ReasonablenessTest,
            AnalyticalMethod::Regression,
            AnalyticalMethod::Comparison,
        ];

        let mut results = Vec::with_capacity(phases.len());

        for (i, &phase) in phases.iter().enumerate() {
            // Choose account or area.
            let account_or_area: String = if !account_codes.is_empty() {
                let idx = self.rng.random_range(0..account_codes.len());
                account_codes[idx].clone()
            } else {
                let idx = i % default_areas.len();
                default_areas[idx].to_string()
            };

            // Analytical method — cycle through available methods.
            let method = all_methods[i % all_methods.len()];

            // Expectation: $100k – $10M
            let expect_units: i64 = self.rng.random_range(100_000_i64..=10_000_000_i64);
            let expectation = Decimal::new(expect_units, 0);

            // Threshold: 5–15% of expectation.
            let threshold_pct: f64 = self.rng.random_range(0.05..0.15);
            let threshold_units = (expect_units as f64 * threshold_pct).round() as i64;
            let threshold = Decimal::new(threshold_units.max(1), 0);

            // Actual value: expectation + normal noise centred at 0, σ = threshold × 0.6.
            let sigma = (expect_units as f64 * threshold_pct * 0.6).max(1.0);
            let normal = Normal::new(0.0_f64, sigma)
                .unwrap_or_else(|_| Normal::new(0.0, 1.0).expect("fallback Normal"));
            let noise = normal.sample(&mut self.rng);
            let actual_units = (expect_units as f64 + noise).round() as i64;
            let actual_units = actual_units.max(0);
            let actual_value = Decimal::new(actual_units, 0);

            let expectation_basis =
                format!("Prior year adjusted for growth — {method:?} applied to {account_or_area}");
            let threshold_basis = format!("{:.0}% of expectation", threshold_pct * 100.0);

            let mut result = AnalyticalProcedureResult::new(
                engagement.engagement_id,
                account_or_area.clone(),
                method,
                expectation,
                expectation_basis,
                threshold,
                threshold_basis,
                actual_value,
            );

            // Override the default phase (constructor sets Substantive).
            result.procedure_phase = phase;

            // Assign a conclusion according to the configured ratios.
            let conclusion = self.choose_conclusion(result.requires_investigation);
            result.conclusion = Some(conclusion);
            result.status = datasynth_core::models::audit::AnalyticalStatus::Concluded;

            // Add an explanation for non-Consistent conclusions.
            if !matches!(conclusion, AnalyticalConclusion::Consistent) {
                result.explanation = Some(self.explanation_text(conclusion, &account_or_area));
                if matches!(conclusion, AnalyticalConclusion::ExplainedVariance) {
                    result.explanation_corroborated = Some(true);
                    result.corroboration_evidence = Some(
                        "Management provided supporting schedule; figures agreed to source data."
                            .to_string(),
                    );
                }
            }

            results.push(result);
        }

        results
    }

    /// Generate analytical procedures anchored to real account balances.
    ///
    /// Behaves identically to [`generate_procedures`] except that, for each
    /// procedure, `actual_value` is set to the account's real balance (looked
    /// up in `account_balances`) and `expectation` is derived as
    /// `actual_value * (1 + noise)` so the variance is small and realistic.
    ///
    /// Accounts that do not appear in `account_balances` fall back to a
    /// default balance of 100,000.
    pub fn generate_procedures_with_balances(
        &mut self,
        engagement: &AuditEngagement,
        account_codes: &[String],
        account_balances: &std::collections::HashMap<String, f64>,
    ) -> Vec<AnalyticalProcedureResult> {
        let count = self.rng.random_range(
            self.config.procedures_per_engagement.0..=self.config.procedures_per_engagement.1,
        ) as usize;

        // Phase distribution: Planning 20%, Substantive 60%, FinalReview 20%.
        let planning_count = (count as f64 * 0.20).round() as usize;
        let final_count = (count as f64 * 0.20).round() as usize;
        let substantive_count = count.saturating_sub(planning_count + final_count).max(1);

        let mut phases: Vec<AnalyticalPhase> = Vec::with_capacity(count);
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::Planning,
            planning_count,
        ));
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::Substantive,
            substantive_count,
        ));
        phases.extend(std::iter::repeat_n(
            AnalyticalPhase::FinalReview,
            final_count,
        ));

        let default_areas = [
            "Revenue",
            "Cost of Sales",
            "Operating Expenses",
            "Accounts Receivable",
            "Inventory",
            "Payroll Expense",
            "Interest Expense",
            "Depreciation",
            "Accounts Payable",
            "Income Tax Expense",
        ];

        let all_methods = [
            AnalyticalMethod::TrendAnalysis,
            AnalyticalMethod::RatioAnalysis,
            AnalyticalMethod::ReasonablenessTest,
            AnalyticalMethod::Regression,
            AnalyticalMethod::Comparison,
        ];

        let mut results = Vec::with_capacity(phases.len());

        for (i, &phase) in phases.iter().enumerate() {
            let account_or_area: String = if !account_codes.is_empty() {
                let idx = self.rng.random_range(0..account_codes.len());
                account_codes[idx].clone()
            } else {
                let idx = i % default_areas.len();
                default_areas[idx].to_string()
            };

            let method = all_methods[i % all_methods.len()];

            // Look up the real balance; fall back to a sensible default.
            let real_balance = account_balances
                .get(&account_or_area)
                .copied()
                .unwrap_or(100_000.0);
            let actual_units = real_balance.round() as i64;
            let actual_value = Decimal::new(actual_units.max(0), 0);

            // Threshold: 5-15% of actual value.
            let threshold_pct: f64 = self.rng.random_range(0.05..0.15);
            let threshold_units = (actual_units as f64 * threshold_pct).round().abs() as i64;
            let threshold = Decimal::new(threshold_units.max(1), 0);

            // Expectation = actual_value + small normal noise (σ = threshold * 0.6).
            let sigma = (actual_units as f64 * threshold_pct * 0.6).abs().max(1.0);
            let normal = Normal::new(0.0_f64, sigma)
                .unwrap_or_else(|_| Normal::new(0.0, 1.0).expect("fallback Normal"));
            let noise = normal.sample(&mut self.rng);
            let expect_units = (actual_units as f64 + noise).round() as i64;
            let expect_units = expect_units.max(0);
            let expectation = Decimal::new(expect_units, 0);

            let expectation_basis =
                format!("Prior year adjusted for growth — {method:?} applied to {account_or_area}");
            let threshold_basis = format!("{:.0}% of expectation", threshold_pct * 100.0);

            let mut result = AnalyticalProcedureResult::new(
                engagement.engagement_id,
                account_or_area.clone(),
                method,
                expectation,
                expectation_basis,
                threshold,
                threshold_basis,
                actual_value,
            );

            result.procedure_phase = phase;

            let conclusion = self.choose_conclusion(result.requires_investigation);
            result.conclusion = Some(conclusion);
            result.status = datasynth_core::models::audit::AnalyticalStatus::Concluded;

            if !matches!(conclusion, AnalyticalConclusion::Consistent) {
                result.explanation = Some(self.explanation_text(conclusion, &account_or_area));
                if matches!(conclusion, AnalyticalConclusion::ExplainedVariance) {
                    result.explanation_corroborated = Some(true);
                    result.corroboration_evidence = Some(
                        "Management provided supporting schedule; figures agreed to source data."
                            .to_string(),
                    );
                }
            }

            results.push(result);
        }

        results
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /// Choose a conclusion according to the configured ratios.
    ///
    /// When the variance requires investigation we bias towards the three non-Consistent
    /// outcomes; otherwise we strongly favour Consistent.
    fn choose_conclusion(&mut self, requires_investigation: bool) -> AnalyticalConclusion {
        let roll: f64 = self.rng.random();

        // If investigation is NOT required, use unmodified ratios (consistent should dominate).
        // If investigation IS required, shift weight away from Consistent.
        let consistent_ratio = if requires_investigation {
            self.config.consistent_ratio * 0.3 // much less likely
        } else {
            self.config.consistent_ratio
        };

        let consistent_cutoff = consistent_ratio;
        let explained_cutoff = consistent_cutoff + self.config.explained_ratio;
        let further_cutoff = explained_cutoff + self.config.further_ratio;

        if roll < consistent_cutoff {
            AnalyticalConclusion::Consistent
        } else if roll < explained_cutoff {
            AnalyticalConclusion::ExplainedVariance
        } else if roll < further_cutoff {
            AnalyticalConclusion::FurtherInvestigation
        } else {
            AnalyticalConclusion::PossibleMisstatement
        }
    }

    fn explanation_text(&self, conclusion: AnalyticalConclusion, area: &str) -> String {
        match conclusion {
            AnalyticalConclusion::ExplainedVariance => {
                format!(
                    "Variance in {area} explained by timing of year-end transactions \
					 and one-off items — management provided reconciliation."
                )
            }
            AnalyticalConclusion::FurtherInvestigation => {
                format!(
                    "Variance in {area} exceeds threshold; additional procedures \
					 required to determine whether a misstatement exists."
                )
            }
            AnalyticalConclusion::PossibleMisstatement => {
                format!(
                    "Variance in {area} is unexplained and may indicate a misstatement; \
					 extend substantive testing to corroborate."
                )
            }
            AnalyticalConclusion::Consistent => String::new(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    fn make_gen(seed: u64) -> AnalyticalProcedureGenerator {
        AnalyticalProcedureGenerator::new(seed)
    }

    fn empty_accounts() -> Vec<String> {
        Vec::new()
    }

    // -------------------------------------------------------------------------

    /// Count is within the configured (min, max) range.
    #[test]
    fn test_generates_procedures() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(42);
        let results = gen.generate_procedures(&engagement, &empty_accounts());

        let cfg = AnalyticalProcedureGeneratorConfig::default();
        let min = cfg.procedures_per_engagement.0 as usize;
        let max = cfg.procedures_per_engagement.1 as usize;
        assert!(
            results.len() >= min && results.len() <= max,
            "expected {min}..={max}, got {}",
            results.len()
        );
    }

    /// Phase distribution should include all three phases.
    #[test]
    fn test_phase_distribution() {
        let engagement = create_test_engagement();
        let config = AnalyticalProcedureGeneratorConfig {
            procedures_per_engagement: (20, 20),
            ..Default::default()
        };
        let mut gen = AnalyticalProcedureGenerator::with_config(10, config);
        let results = gen.generate_procedures(&engagement, &empty_accounts());

        let has_planning = results
            .iter()
            .any(|r| r.procedure_phase == AnalyticalPhase::Planning);
        let has_substantive = results
            .iter()
            .any(|r| r.procedure_phase == AnalyticalPhase::Substantive);
        let has_final = results
            .iter()
            .any(|r| r.procedure_phase == AnalyticalPhase::FinalReview);

        assert!(has_planning, "expected at least one Planning procedure");
        assert!(
            has_substantive,
            "expected at least one Substantive procedure"
        );
        assert!(has_final, "expected at least one FinalReview procedure");
    }

    /// With a large count, conclusion distribution should roughly match config.
    #[test]
    fn test_conclusion_distribution() {
        let engagement = create_test_engagement();
        let config = AnalyticalProcedureGeneratorConfig {
            procedures_per_engagement: (200, 200),
            consistent_ratio: 0.60,
            explained_ratio: 0.25,
            further_ratio: 0.10,
            misstatement_ratio: 0.05,
        };
        let mut gen = AnalyticalProcedureGenerator::with_config(99, config);
        let results = gen.generate_procedures(&engagement, &empty_accounts());

        // All results should have a conclusion.
        let no_conclusion = results.iter().filter(|r| r.conclusion.is_none()).count();
        assert_eq!(no_conclusion, 0, "all results must have a conclusion");

        // There should be at least some Consistent results (dominant outcome).
        let consistent_count = results
            .iter()
            .filter(|r| r.conclusion == Some(AnalyticalConclusion::Consistent))
            .count();
        assert!(
            consistent_count > 0,
            "expected at least some Consistent conclusions, got 0"
        );
    }

    /// Same seed produces identical output.
    #[test]
    fn test_deterministic() {
        let engagement = create_test_engagement();
        let accounts = vec!["1000".to_string(), "2000".to_string(), "3000".to_string()];

        let results_a =
            AnalyticalProcedureGenerator::new(1234).generate_procedures(&engagement, &accounts);
        let results_b =
            AnalyticalProcedureGenerator::new(1234).generate_procedures(&engagement, &accounts);

        assert_eq!(
            results_a.len(),
            results_b.len(),
            "lengths differ across identical seeds"
        );
        for (a, b) in results_a.iter().zip(results_b.iter()) {
            assert_eq!(a.account_or_area, b.account_or_area);
            assert_eq!(a.expectation, b.expectation);
            assert_eq!(a.actual_value, b.actual_value);
            assert_eq!(a.conclusion, b.conclusion);
            assert_eq!(a.procedure_phase, b.procedure_phase);
        }
    }

    /// When account_codes is non-empty, results should reference those codes.
    #[test]
    fn test_account_codes_used() {
        let engagement = create_test_engagement();
        let accounts = vec![
            "REV-1000".to_string(),
            "EXP-2000".to_string(),
            "ASS-3000".to_string(),
        ];

        let mut gen = make_gen(55);
        let results = gen.generate_procedures(&engagement, &accounts);

        for result in &results {
            assert!(
                accounts.contains(&result.account_or_area),
                "account_or_area '{}' not in provided list",
                result.account_or_area
            );
        }
    }

    /// Variance, variance_percentage, and requires_investigation should be consistent.
    #[test]
    fn test_variance_fields_consistent() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(88);
        let results = gen.generate_procedures(&engagement, &empty_accounts());

        for r in &results {
            let expected_variance = r.actual_value - r.expectation;
            assert_eq!(
                r.variance, expected_variance,
                "variance mismatch for result_ref {}",
                r.result_ref
            );
            // requires_investigation must be consistent with |variance| > threshold.
            let expected_flag = r.variance.abs() > r.threshold;
            assert_eq!(
                r.requires_investigation, expected_flag,
                "requires_investigation flag mismatch for {}",
                r.result_ref
            );
        }
    }
}
