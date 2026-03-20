//! Going concern assessment generator — ISA 570 / ASC 205-40.
//!
//! Generates one `GoingConcernAssessment` per entity per period.
//!
//! # Distribution of outcomes
//!
//! | Scenario              | Probability | Indicators | Conclusion                    |
//! |-----------------------|-------------|------------|-------------------------------|
//! | Clean (no issues)     | 90–95%      | 0          | `NoMaterialUncertainty`       |
//! | Mild concerns         | 4–8%        | 1–2        | `MaterialUncertaintyExists`   |
//! | Significant concerns  | 1–2%        | 3+         | `GoingConcernDoubt`           |

use chrono::NaiveDate;
use datasynth_core::models::audit::going_concern::{
    GoingConcernAssessment, GoingConcernIndicator, GoingConcernIndicatorType, GoingConcernSeverity,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the going concern generator.
#[derive(Debug, Clone)]
pub struct GoingConcernGeneratorConfig {
    /// Probability that an entity receives zero indicators (clean assessment).
    pub clean_probability: f64,
    /// Probability that an entity receives 1–2 indicators (mild concerns).
    pub mild_probability: f64,
    // Note: severe probability = 1 - clean_probability - mild_probability.
}

impl Default for GoingConcernGeneratorConfig {
    fn default() -> Self {
        Self {
            clean_probability: 0.90,
            mild_probability: 0.08,
        }
    }
}

// ---------------------------------------------------------------------------
// Financial input for data-driven assessments
// ---------------------------------------------------------------------------

/// Financial metrics derived from actual generated data, used to derive
/// going concern indicators from real financials rather than random draws.
///
/// All amounts are in the entity's reporting currency.
#[derive(Debug, Clone)]
pub struct GoingConcernInput {
    /// Entity code being assessed.
    pub entity_code: String,
    /// Net income / (loss) for the period (negative = loss).
    pub net_income: Decimal,
    /// Working capital = current assets − current liabilities (negative = deficiency).
    pub working_capital: Decimal,
    /// Net cash from operating activities (negative = outflow).
    pub operating_cash_flow: Decimal,
    /// Total financial debt outstanding.
    pub total_debt: Decimal,
    /// Date the assessment is finalised.
    pub assessment_date: NaiveDate,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 570 / ASC 205-40 going concern assessments.
pub struct GoingConcernGenerator {
    rng: ChaCha8Rng,
    config: GoingConcernGeneratorConfig,
}

impl GoingConcernGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x570), // discriminator for ISA 570
            config: GoingConcernGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: GoingConcernGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x570),
            config,
        }
    }

    /// Generate a going concern assessment for a single entity.
    ///
    /// # Arguments
    /// * `entity_code` — Entity code being assessed.
    /// * `assessment_date` — Date the assessment was finalised (typically the
    ///   financial statement approval date).
    /// * `period` — Human-readable period descriptor (e.g. "FY2024").
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        assessment_date: NaiveDate,
        period: &str,
    ) -> GoingConcernAssessment {
        info!(
            "Generating going concern assessment for entity {} period {}",
            entity_code, period
        );
        let roll: f64 = self.rng.random();
        let indicator_count = if roll < self.config.clean_probability {
            0
        } else if roll < self.config.clean_probability + self.config.mild_probability {
            self.rng.random_range(1u32..=2)
        } else {
            self.rng.random_range(3u32..=5)
        };

        let indicators = (0..indicator_count)
            .map(|_| self.random_indicator(entity_code))
            .collect::<Vec<_>>();

        let management_plans = if indicators.is_empty() {
            Vec::new()
        } else {
            self.management_plans(indicators.len())
        };

        let assessment = GoingConcernAssessment {
            entity_code: entity_code.to_string(),
            assessment_date,
            assessment_period: period.to_string(),
            indicators,
            management_plans,
            auditor_conclusion: Default::default(), // will be overwritten below
            material_uncertainty_exists: false,
        }
        .conclude_from_indicators();
        info!(
            "Going concern for {}: {} indicators, conclusion={:?}",
            entity_code,
            assessment.indicators.len(),
            assessment.auditor_conclusion
        );
        assessment
    }

    /// Generate assessments for multiple entities in a single batch.
    pub fn generate_for_entities(
        &mut self,
        entity_codes: &[String],
        assessment_date: NaiveDate,
        period: &str,
    ) -> Vec<GoingConcernAssessment> {
        entity_codes
            .iter()
            .map(|code| self.generate_for_entity(code, assessment_date, period))
            .collect()
    }

    /// Generate a going concern assessment driven by actual financial data.
    ///
    /// Financial indicators (recurring losses, negative working capital, negative
    /// operating cash flow) are determined from the supplied [`GoingConcernInput`].
    /// Non-financial indicators (litigation, regulatory action, etc.) retain the
    /// random element since they cannot be inferred from journal entries alone.
    ///
    /// # Indicator mapping
    /// - `net_income < 0`          → [`GoingConcernIndicatorType::RecurringOperatingLosses`]
    /// - `working_capital < 0`     → [`GoingConcernIndicatorType::WorkingCapitalDeficiency`]
    /// - `operating_cash_flow < 0` → [`GoingConcernIndicatorType::NegativeOperatingCashFlow`]
    ///
    /// # Conclusion
    /// 0 indicators → `NoMaterialUncertainty`, 1–2 → `MaterialUncertaintyExists`,
    /// 3+ → `GoingConcernDoubt` (same rule as [`generate_for_entity`]).
    pub fn generate_for_entity_with_input(
        &mut self,
        input: &GoingConcernInput,
        period: &str,
    ) -> GoingConcernAssessment {
        let entity_code = input.entity_code.as_str();
        let mut indicators: Vec<GoingConcernIndicator> = Vec::new();

        // ---- Financial indicators derived from actual data --------------------

        if input.net_income < Decimal::ZERO {
            let loss = input.net_income.abs();
            let threshold = loss * dec!(1.50);
            indicators.push(GoingConcernIndicator {
                indicator_type: GoingConcernIndicatorType::RecurringOperatingLosses,
                severity: if loss > Decimal::from(1_000_000i64) {
                    GoingConcernSeverity::High
                } else if loss > Decimal::from(100_000i64) {
                    GoingConcernSeverity::Medium
                } else {
                    GoingConcernSeverity::Low
                },
                description: self.describe_indicator(
                    GoingConcernIndicatorType::RecurringOperatingLosses,
                    entity_code,
                ),
                quantitative_measure: Some(loss),
                threshold: Some(threshold),
            });
        }

        if input.working_capital < Decimal::ZERO {
            let deficit = input.working_capital.abs();
            indicators.push(GoingConcernIndicator {
                indicator_type: GoingConcernIndicatorType::WorkingCapitalDeficiency,
                severity: if deficit > Decimal::from(5_000_000i64) {
                    GoingConcernSeverity::High
                } else if deficit > Decimal::from(500_000i64) {
                    GoingConcernSeverity::Medium
                } else {
                    GoingConcernSeverity::Low
                },
                description: self.describe_indicator(
                    GoingConcernIndicatorType::WorkingCapitalDeficiency,
                    entity_code,
                ),
                quantitative_measure: Some(deficit),
                threshold: Some(Decimal::ZERO),
            });
        }

        if input.operating_cash_flow < Decimal::ZERO {
            let outflow = input.operating_cash_flow.abs();
            indicators.push(GoingConcernIndicator {
                indicator_type: GoingConcernIndicatorType::NegativeOperatingCashFlow,
                severity: if outflow > Decimal::from(2_000_000i64) {
                    GoingConcernSeverity::High
                } else if outflow > Decimal::from(200_000i64) {
                    GoingConcernSeverity::Medium
                } else {
                    GoingConcernSeverity::Low
                },
                description: self.describe_indicator(
                    GoingConcernIndicatorType::NegativeOperatingCashFlow,
                    entity_code,
                ),
                quantitative_measure: Some(outflow),
                threshold: Some(Decimal::ZERO),
            });
        }

        // ---- Random non-financial indicators (litigation, regulatory, etc.) --
        // Only add if the financial indicators haven't already pushed us into
        // going-concern doubt territory, to keep the realistic distribution.
        if indicators.len() < 3 {
            let roll: f64 = self.rng.random();
            // ~5% chance of a random non-financial indicator when finances are OK
            if roll < 0.05 {
                let extra = self.random_non_financial_indicator(entity_code);
                indicators.push(extra);
            }
        }

        let management_plans = if indicators.is_empty() {
            Vec::new()
        } else {
            self.management_plans(indicators.len())
        };

        GoingConcernAssessment {
            entity_code: entity_code.to_string(),
            assessment_date: input.assessment_date,
            assessment_period: period.to_string(),
            indicators,
            management_plans,
            auditor_conclusion: Default::default(),
            material_uncertainty_exists: false,
        }
        .conclude_from_indicators()
    }

    /// Generate assessments for multiple entities using financial data inputs.
    ///
    /// Entities without a corresponding input fall back to random behaviour.
    pub fn generate_for_entities_with_inputs(
        &mut self,
        entity_codes: &[String],
        inputs: &[GoingConcernInput],
        assessment_date: NaiveDate,
        period: &str,
    ) -> Vec<GoingConcernAssessment> {
        entity_codes
            .iter()
            .map(|code| {
                if let Some(input) = inputs.iter().find(|i| &i.entity_code == code) {
                    self.generate_for_entity_with_input(input, period)
                } else {
                    self.generate_for_entity(code, assessment_date, period)
                }
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Generate a random non-financial indicator (litigation, regulatory, etc.).
    fn random_non_financial_indicator(&mut self, entity_code: &str) -> GoingConcernIndicator {
        // Only pick from non-financial types
        let indicator_type = match self.rng.random_range(0u8..5) {
            0 => GoingConcernIndicatorType::DebtCovenantBreach,
            1 => GoingConcernIndicatorType::LossOfKeyCustomer,
            2 => GoingConcernIndicatorType::RegulatoryAction,
            3 => GoingConcernIndicatorType::LitigationExposure,
            _ => GoingConcernIndicatorType::InabilityToObtainFinancing,
        };
        let severity = self.random_severity();
        let description = self.describe_indicator(indicator_type, entity_code);
        let (measure, threshold) = self.quantitative_measures(indicator_type);
        GoingConcernIndicator {
            indicator_type,
            severity,
            description,
            quantitative_measure: Some(measure),
            threshold: Some(threshold),
        }
    }

    fn random_indicator(&mut self, entity_code: &str) -> GoingConcernIndicator {
        let indicator_type = self.random_indicator_type();
        let severity = self.random_severity();

        let description = self.describe_indicator(indicator_type, entity_code);
        let (measure, threshold) = self.quantitative_measures(indicator_type);

        GoingConcernIndicator {
            indicator_type,
            severity,
            description,
            quantitative_measure: Some(measure),
            threshold: Some(threshold),
        }
    }

    fn random_indicator_type(&mut self) -> GoingConcernIndicatorType {
        match self.rng.random_range(0u8..8) {
            0 => GoingConcernIndicatorType::RecurringOperatingLosses,
            1 => GoingConcernIndicatorType::NegativeOperatingCashFlow,
            2 => GoingConcernIndicatorType::WorkingCapitalDeficiency,
            3 => GoingConcernIndicatorType::DebtCovenantBreach,
            4 => GoingConcernIndicatorType::LossOfKeyCustomer,
            5 => GoingConcernIndicatorType::RegulatoryAction,
            6 => GoingConcernIndicatorType::LitigationExposure,
            _ => GoingConcernIndicatorType::InabilityToObtainFinancing,
        }
    }

    fn random_severity(&mut self) -> GoingConcernSeverity {
        match self.rng.random_range(0u8..3) {
            0 => GoingConcernSeverity::Low,
            1 => GoingConcernSeverity::Medium,
            _ => GoingConcernSeverity::High,
        }
    }

    fn describe_indicator(
        &self,
        indicator_type: GoingConcernIndicatorType,
        entity_code: &str,
    ) -> String {
        match indicator_type {
            GoingConcernIndicatorType::RecurringOperatingLosses => format!(
                "{} has reported operating losses in each of the past three financial years, \
                 indicating structural challenges in its core business model.",
                entity_code
            ),
            GoingConcernIndicatorType::NegativeOperatingCashFlow => format!(
                "{} generated negative operating cash flows during the current period, \
                 requiring reliance on financing activities to fund operations.",
                entity_code
            ),
            GoingConcernIndicatorType::WorkingCapitalDeficiency => format!(
                "{} has a working capital deficiency, with current liabilities exceeding \
                 current assets, potentially impairing its ability to meet short-term obligations.",
                entity_code
            ),
            GoingConcernIndicatorType::DebtCovenantBreach => format!(
                "{} has breached one or more financial covenants in its debt agreements, \
                 which may result in lenders demanding immediate repayment.",
                entity_code
            ),
            GoingConcernIndicatorType::LossOfKeyCustomer => format!(
                "{} lost a major customer during the period, representing a material decline \
                 in projected revenue and profitability.",
                entity_code
            ),
            GoingConcernIndicatorType::RegulatoryAction => format!(
                "{} is subject to regulatory action or investigation that may threaten \
                 its licence to operate or result in material financial penalties.",
                entity_code
            ),
            GoingConcernIndicatorType::LitigationExposure => format!(
                "{} faces pending legal proceedings with a potential financial exposure \
                 that could be material relative to its net assets.",
                entity_code
            ),
            GoingConcernIndicatorType::InabilityToObtainFinancing => format!(
                "{} has been unable to secure new credit facilities or roll over existing \
                 financing arrangements, creating a liquidity risk.",
                entity_code
            ),
        }
    }

    /// Return (quantitative_measure, threshold) for the given indicator type.
    fn quantitative_measures(
        &mut self,
        indicator_type: GoingConcernIndicatorType,
    ) -> (Decimal, Decimal) {
        match indicator_type {
            GoingConcernIndicatorType::RecurringOperatingLosses => {
                // Loss amount and a materiality threshold
                let loss = Decimal::new(self.rng.random_range(100_000i64..=5_000_000), 0);
                let threshold = loss * Decimal::new(150, 2); // 1.5x — significant if > threshold
                (loss, threshold)
            }
            GoingConcernIndicatorType::NegativeOperatingCashFlow => {
                let outflow = Decimal::new(self.rng.random_range(50_000i64..=2_000_000), 0);
                let threshold = Decimal::ZERO;
                (outflow, threshold)
            }
            GoingConcernIndicatorType::WorkingCapitalDeficiency => {
                let deficit = Decimal::new(self.rng.random_range(100_000i64..=10_000_000), 0);
                let threshold = Decimal::ZERO;
                (deficit, threshold)
            }
            GoingConcernIndicatorType::DebtCovenantBreach => {
                // Actual leverage ratio vs covenant limit
                let actual = Decimal::new(self.rng.random_range(350i64..=600), 2); // 3.50–6.00x
                let covenant = Decimal::new(300, 2); // 3.00x limit
                (actual, covenant)
            }
            GoingConcernIndicatorType::LossOfKeyCustomer => {
                // Revenue lost as a percentage of total revenue
                let pct = Decimal::new(self.rng.random_range(15i64..=40), 2); // 15–40%
                let threshold = Decimal::new(10, 2); // 10% materiality
                (pct, threshold)
            }
            GoingConcernIndicatorType::RegulatoryAction
            | GoingConcernIndicatorType::LitigationExposure
            | GoingConcernIndicatorType::InabilityToObtainFinancing => {
                let exposure = Decimal::new(self.rng.random_range(500_000i64..=20_000_000), 0);
                let threshold = Decimal::new(self.rng.random_range(1_000_000i64..=5_000_000), 0);
                (exposure, threshold)
            }
        }
    }

    fn management_plans(&mut self, indicator_count: usize) -> Vec<String> {
        let all_plans = [
            "Management has engaged external financial advisors to explore refinancing options \
             and extend the maturity of existing credit facilities.",
            "A detailed cash flow management plan has been approved by the board, including \
             targeted working capital improvements and deferral of non-essential capital expenditure.",
            "Management is actively pursuing new customer acquisition initiatives and has \
             secured letters of intent from prospective strategic customers.",
            "The board has committed to a capital injection of additional equity through \
             a rights issue to be completed within 90 days of the balance sheet date.",
            "Management is in advanced negotiations with existing lenders to obtain covenant \
             waivers and to restructure the terms of outstanding debt facilities.",
            "A formal cost reduction programme has been announced, targeting annualised \
             savings sufficient to return the entity to operating profitability within 12 months.",
            "The entity has received a legally binding letter of support from its parent \
             company confirming financial support for a minimum of 12 months.",
        ];

        let n_plans = indicator_count.clamp(1, 3);
        let start = self
            .rng
            .random_range(0..all_plans.len().saturating_sub(n_plans));
        all_plans[start..start + n_plans]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::audit::going_concern::GoingConcernConclusion;

    fn assessment_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()
    }

    #[test]
    fn test_generates_one_assessment_per_entity() {
        let entities = vec!["C001".to_string(), "C002".to_string(), "C003".to_string()];
        let mut gen = GoingConcernGenerator::new(42);
        let assessments = gen.generate_for_entities(&entities, assessment_date(), "FY2024");
        assert_eq!(assessments.len(), entities.len());
    }

    #[test]
    fn test_approximately_90_percent_clean() {
        let mut total = 0usize;
        let mut clean = 0usize;
        for seed in 0..200u64 {
            let mut gen = GoingConcernGenerator::new(seed);
            let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
            total += 1;
            if matches!(
                a.auditor_conclusion,
                GoingConcernConclusion::NoMaterialUncertainty
            ) {
                clean += 1;
            }
        }
        let ratio = clean as f64 / total as f64;
        assert!(
            ratio >= 0.80 && ratio <= 0.98,
            "Clean ratio = {:.2}, expected ~0.90",
            ratio
        );
    }

    #[test]
    fn test_conclusion_matches_indicator_count() {
        let mut gen = GoingConcernGenerator::new(42);
        for seed in 0..100u64 {
            let mut g = GoingConcernGenerator::new(seed);
            let a = g.generate_for_entity("C001", assessment_date(), "FY2024");
            let n = a.indicators.len();
            match a.auditor_conclusion {
                GoingConcernConclusion::NoMaterialUncertainty => {
                    assert_eq!(n, 0, "seed={}: clean but has {} indicators", seed, n);
                }
                GoingConcernConclusion::MaterialUncertaintyExists => {
                    assert!(
                        n >= 1 && n <= 2,
                        "seed={}: MaterialUncertainty but {} indicators",
                        seed,
                        n
                    );
                }
                GoingConcernConclusion::GoingConcernDoubt => {
                    assert!(n >= 3, "seed={}: Doubt but only {} indicators", seed, n);
                }
            }
        }
        drop(gen);
    }

    #[test]
    fn test_indicators_have_severity() {
        for seed in 0..50u64 {
            let mut gen = GoingConcernGenerator::new(seed);
            let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
            for indicator in &a.indicators {
                // Severity must be one of the valid variants (always true for an enum,
                // but this also exercises the serialisation round-trip)
                let json = serde_json::to_string(&indicator.severity).unwrap();
                assert!(!json.is_empty());
            }
        }
    }

    #[test]
    fn test_material_uncertainty_flag_consistent() {
        for seed in 0..100u64 {
            let mut gen = GoingConcernGenerator::new(seed);
            let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
            if a.indicators.is_empty() {
                assert!(
                    !a.material_uncertainty_exists,
                    "seed={}: no indicators but material_uncertainty_exists=true",
                    seed
                );
            } else {
                assert!(
                    a.material_uncertainty_exists,
                    "seed={}: has {} indicators but material_uncertainty_exists=false",
                    seed,
                    a.indicators.len()
                );
            }
        }
    }

    #[test]
    fn test_management_plans_when_indicators_present() {
        for seed in 0..200u64 {
            let mut gen = GoingConcernGenerator::new(seed);
            let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
            if !a.indicators.is_empty() {
                assert!(
                    !a.management_plans.is_empty(),
                    "seed={}: indicators present but no management plans",
                    seed
                );
            } else {
                assert!(
                    a.management_plans.is_empty(),
                    "seed={}: no indicators but management plans present",
                    seed
                );
            }
        }
    }
}
