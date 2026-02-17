//! Impairment test generator (ASC 360 / IAS 36).
//!
//! Generates synthetic impairment tests for long-lived assets, including:
//! - Asset type distribution (PPE, intangibles, goodwill, ROU, etc.)
//! - Impairment indicator selection
//! - 5-year cash flow projections with discounting
//! - Framework-specific test logic (US GAAP two-step vs IFRS one-step)
//! - Configurable impairment rate targeting

use chrono::NaiveDate;
use datasynth_config::schema::ImpairmentConfig;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use datasynth_standards::accounting::impairment::{
    CashFlowProjection, ImpairmentAssetType, ImpairmentIndicator, ImpairmentTest,
    ImpairmentTestResult,
};
use datasynth_standards::framework::AccountingFramework;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// All non-goodwill asset types with their approximate probability weights.
const ASSET_TYPE_WEIGHTS_NO_GOODWILL: [(ImpairmentAssetType, u32); 6] = [
    (ImpairmentAssetType::PropertyPlantEquipment, 40),
    (ImpairmentAssetType::IntangibleFinite, 20),
    (ImpairmentAssetType::IntangibleIndefinite, 15),
    (ImpairmentAssetType::RightOfUseAsset, 10),
    (ImpairmentAssetType::EquityInvestment, 10),
    (ImpairmentAssetType::CashGeneratingUnit, 5),
];

/// Asset types with goodwill included (redistributed weights).
const ASSET_TYPE_WEIGHTS_WITH_GOODWILL: [(ImpairmentAssetType, u32); 7] = [
    (ImpairmentAssetType::PropertyPlantEquipment, 30),
    (ImpairmentAssetType::IntangibleFinite, 15),
    (ImpairmentAssetType::IntangibleIndefinite, 12),
    (ImpairmentAssetType::Goodwill, 15),
    (ImpairmentAssetType::RightOfUseAsset, 10),
    (ImpairmentAssetType::EquityInvestment, 10),
    (ImpairmentAssetType::CashGeneratingUnit, 8),
];

/// Indicators that can be randomly assigned to any asset test.
const GENERAL_INDICATORS: [ImpairmentIndicator; 10] = [
    ImpairmentIndicator::MarketValueDecline,
    ImpairmentIndicator::AdverseEnvironmentChanges,
    ImpairmentIndicator::InterestRateIncrease,
    ImpairmentIndicator::MarketCapBelowBookValue,
    ImpairmentIndicator::ObsolescenceOrDamage,
    ImpairmentIndicator::AdverseUseChanges,
    ImpairmentIndicator::OperatingLosses,
    ImpairmentIndicator::DiscontinuationPlans,
    ImpairmentIndicator::EarlyDisposal,
    ImpairmentIndicator::WorsePerformance,
];

/// Projection horizon in years for value-in-use calculations.
const PROJECTION_YEARS: u32 = 5;

/// Generates impairment tests for long-lived assets.
pub struct ImpairmentGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl ImpairmentGenerator {
    /// Create a new impairment generator with a deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ImpairmentTest),
        }
    }

    /// Create a new impairment generator with custom configuration (seed only;
    /// the per-run [`ImpairmentConfig`] is passed to [`generate`]).
    pub fn with_config(seed: u64, _config: &ImpairmentConfig) -> Self {
        // Config is used at generation time, not construction time.
        // The constructor signature is kept for consistency with other generators.
        Self::new(seed)
    }

    /// Generate impairment tests for the given assets.
    ///
    /// # Arguments
    ///
    /// * `company_code` - Company identifier
    /// * `asset_ids` - Slice of `(asset_id, description, carrying_amount)` tuples
    /// * `test_date` - Date of the impairment assessment
    /// * `config` - Impairment-specific configuration
    /// * `framework` - Accounting framework (US GAAP, IFRS, or Dual Reporting)
    ///
    /// # Returns
    ///
    /// A vector of [`ImpairmentTest`] records, sized according to `config.test_count`.
    pub fn generate(
        &mut self,
        company_code: &str,
        asset_ids: &[(String, String, Decimal)],
        test_date: NaiveDate,
        config: &ImpairmentConfig,
        framework: AccountingFramework,
    ) -> Vec<ImpairmentTest> {
        if asset_ids.is_empty() || config.test_count == 0 {
            return Vec::new();
        }

        let mut tests: Vec<ImpairmentTest> = Vec::with_capacity(config.test_count);

        for i in 0..config.test_count {
            let (asset_id, description, carrying_amount) = &asset_ids[i % asset_ids.len()];

            let asset_type = self.pick_asset_type(config.include_goodwill);

            let mut test = ImpairmentTest::new(
                company_code,
                asset_id.clone(),
                description.clone(),
                asset_type,
                test_date,
                *carrying_amount,
                framework,
            );

            // Overwrite the v7 test_id with a deterministic UUID from our factory.
            test.test_id = self.uuid_factory.next();

            // --- Indicators ---
            self.add_indicators(&mut test, asset_type);

            // --- Discount rate (8-15%) ---
            let discount_rate_f64 = self.rng.gen_range(0.08..=0.15);
            test.discount_rate =
                Decimal::from_f64_retain(discount_rate_f64).unwrap_or(Decimal::ONE);

            // --- Cash flow projections ---
            if config.generate_projections {
                let projections = self.generate_projections(*carrying_amount, discount_rate_f64);
                test.cash_flow_projections = projections;
                test.calculate_value_in_use();
            }

            // --- Fair value less costs to sell ---
            let fv_factor = self.rng.gen_range(0.5..=1.1);
            let fv_decimal = Decimal::from_f64_retain(fv_factor).unwrap_or(Decimal::ONE);
            test.fair_value_less_costs = *carrying_amount * fv_decimal;

            // --- US GAAP: undiscounted cash flows for Step 1 ---
            if matches!(framework, AccountingFramework::UsGaap) {
                test.calculate_undiscounted_cash_flows();
            }

            // --- Perform the framework-specific test ---
            test.perform_test();

            tests.push(test);
        }

        // --- Enforce impairment rate target ---
        self.enforce_impairment_rate(&mut tests, config.impairment_rate, framework);

        tests
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Pick an asset type using the configured weight tables.
    fn pick_asset_type(&mut self, include_goodwill: bool) -> ImpairmentAssetType {
        if include_goodwill {
            self.weighted_pick(&ASSET_TYPE_WEIGHTS_WITH_GOODWILL)
        } else {
            self.weighted_pick(&ASSET_TYPE_WEIGHTS_NO_GOODWILL)
        }
    }

    /// Generic weighted random selection from a `(T, weight)` slice.
    fn weighted_pick<T: Copy>(&mut self, items: &[(T, u32)]) -> T {
        let total_weight: u32 = items.iter().map(|(_, w)| w).sum();
        let mut roll = self.rng.gen_range(0..total_weight);
        for &(item, weight) in items {
            if roll < weight {
                return item;
            }
            roll -= weight;
        }
        // Fallback (should be unreachable with valid weights).
        items[0].0
    }

    /// Add 1-3 impairment indicators to a test.
    ///
    /// Goodwill and indefinite-life intangibles always receive [`AnnualTest`].
    fn add_indicators(&mut self, test: &mut ImpairmentTest, asset_type: ImpairmentAssetType) {
        let requires_annual = matches!(
            asset_type,
            ImpairmentAssetType::Goodwill | ImpairmentAssetType::IntangibleIndefinite
        );

        if requires_annual {
            test.add_indicator(ImpairmentIndicator::AnnualTest);
        }

        let extra_count = self.rng.gen_range(1..=3_usize);
        let count_needed = if requires_annual {
            // Already added AnnualTest; add 0-2 more for up to 3 total.
            extra_count.saturating_sub(1)
        } else {
            extra_count
        };

        for _ in 0..count_needed {
            let idx = self.rng.gen_range(0..GENERAL_INDICATORS.len());
            let indicator = GENERAL_INDICATORS[idx];
            // Avoid duplicates.
            if !test.impairment_indicators.contains(&indicator) {
                test.add_indicator(indicator);
            }
        }
    }

    /// Build 5-year cash flow projections with an optional terminal value.
    fn generate_projections(
        &mut self,
        carrying_amount: Decimal,
        _discount_rate: f64,
    ) -> Vec<CashFlowProjection> {
        let mut projections = Vec::with_capacity(PROJECTION_YEARS as usize);

        // Base revenue: carrying_amount * random(0.3 .. 0.6)
        let base_factor = self.rng.gen_range(0.3..=0.6);
        let base_revenue_f64 = carrying_amount.to_f64().unwrap_or(100_000.0) * base_factor;

        // Operating expense ratio: 60-80% of revenue.
        let opex_ratio = self.rng.gen_range(0.60..=0.80);

        // Annual growth rate: -5% to +5%.
        let growth_rate_f64 = self.rng.gen_range(-0.05..=0.05);
        let growth_decimal = Decimal::from_f64_retain(growth_rate_f64).unwrap_or(Decimal::ZERO);

        let mut current_revenue_f64 = base_revenue_f64;

        for year in 1..=PROJECTION_YEARS {
            let revenue = Decimal::from_f64_retain(current_revenue_f64).unwrap_or(Decimal::ZERO);
            let opex =
                Decimal::from_f64_retain(current_revenue_f64 * opex_ratio).unwrap_or(Decimal::ZERO);

            let mut proj = CashFlowProjection::new(year, revenue, opex);
            proj.growth_rate = growth_decimal;

            // Capital expenditures: 5-15% of revenue.
            let capex_ratio = self.rng.gen_range(0.05..=0.15);
            proj.capital_expenditures = Decimal::from_f64_retain(current_revenue_f64 * capex_ratio)
                .unwrap_or(Decimal::ZERO);

            // Terminal value bump in year 5.
            if year == PROJECTION_YEARS {
                proj.is_terminal_value = true;
                // Terminal value approximation: year 5 net CF / discount rate,
                // but since we already add it to the projection stream we simply
                // boost revenue by a terminal multiplier (3-5x) to approximate
                // a perpetuity.
                let terminal_multiplier = self.rng.gen_range(3.0..=5.0);
                let terminal_revenue =
                    Decimal::from_f64_retain(current_revenue_f64 * terminal_multiplier)
                        .unwrap_or(Decimal::ZERO);
                let terminal_opex = Decimal::from_f64_retain(
                    current_revenue_f64 * terminal_multiplier * opex_ratio,
                )
                .unwrap_or(Decimal::ZERO);
                proj.revenue = terminal_revenue;
                proj.operating_expenses = terminal_opex;
                // Recalculate capex for terminal year as well.
                proj.capital_expenditures = Decimal::from_f64_retain(
                    current_revenue_f64 * terminal_multiplier * capex_ratio,
                )
                .unwrap_or(Decimal::ZERO);
            }

            proj.calculate_net_cash_flow();
            projections.push(proj);

            // Apply growth for next year.
            current_revenue_f64 *= 1.0 + growth_rate_f64;
        }

        projections
    }

    /// Adjust tests so that the observed impairment rate meets the target.
    ///
    /// If natural generation produced fewer impairments than desired, force
    /// some not-impaired tests to be impaired by lowering their fair value.
    /// If too many are impaired, convert some back to not-impaired by raising
    /// fair value above carrying amount.
    fn enforce_impairment_rate(
        &mut self,
        tests: &mut [ImpairmentTest],
        target_rate: f64,
        framework: AccountingFramework,
    ) {
        if tests.is_empty() {
            return;
        }

        let target_impaired = ((tests.len() as f64) * target_rate).round() as usize;
        let current_impaired = tests
            .iter()
            .filter(|t| t.test_result == ImpairmentTestResult::Impaired)
            .count();

        if current_impaired < target_impaired {
            // Need more impairments -- lower fair value on not-impaired tests.
            let deficit = target_impaired - current_impaired;
            let mut converted = 0usize;
            for test in tests.iter_mut() {
                if converted >= deficit {
                    break;
                }
                if test.test_result == ImpairmentTestResult::NotImpaired {
                    // Set fair value well below carrying amount.
                    let reduction_factor = self.rng.gen_range(0.3..=0.6);
                    let factor_dec =
                        Decimal::from_f64_retain(reduction_factor).unwrap_or(Decimal::ONE);
                    test.fair_value_less_costs = test.carrying_amount * factor_dec;

                    // Also lower value_in_use for IFRS and French GAAP (one-step test).
                    if matches!(
                        framework,
                        AccountingFramework::Ifrs
                            | AccountingFramework::DualReporting
                            | AccountingFramework::FrenchGaap
                    ) {
                        test.value_in_use = test.fair_value_less_costs
                            - Decimal::from_f64_retain(self.rng.gen_range(1000.0..=10000.0))
                                .unwrap_or(Decimal::ZERO);
                    }

                    // For US GAAP, ensure undiscounted CFs fail Step 1.
                    if matches!(framework, AccountingFramework::UsGaap) {
                        let low_factor = self.rng.gen_range(0.5..=0.8);
                        test.undiscounted_cash_flows = Some(
                            test.carrying_amount
                                * Decimal::from_f64_retain(low_factor).unwrap_or(Decimal::ONE),
                        );
                    }

                    test.perform_test();
                    converted += 1;
                }
            }
        } else if current_impaired > target_impaired {
            // Too many impairments -- raise fair value on some impaired tests.
            let surplus = current_impaired - target_impaired;
            let mut converted = 0usize;
            for test in tests.iter_mut() {
                if converted >= surplus {
                    break;
                }
                if test.test_result == ImpairmentTestResult::Impaired {
                    // Set fair value above carrying amount.
                    let boost_factor = self.rng.gen_range(1.05..=1.30);
                    let factor_dec = Decimal::from_f64_retain(boost_factor).unwrap_or(Decimal::ONE);
                    test.fair_value_less_costs = test.carrying_amount * factor_dec;
                    test.value_in_use = test.fair_value_less_costs;

                    if matches!(framework, AccountingFramework::UsGaap) {
                        test.undiscounted_cash_flows = Some(test.carrying_amount * factor_dec);
                    }

                    test.perform_test();
                    converted += 1;
                }
            }
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_assets() -> Vec<(String, String, Decimal)> {
        vec![
            (
                "FA-001".to_string(),
                "Manufacturing Equipment".to_string(),
                dec!(500_000),
            ),
            (
                "FA-002".to_string(),
                "Office Building".to_string(),
                dec!(2_000_000),
            ),
            (
                "FA-003".to_string(),
                "Software License".to_string(),
                dec!(150_000),
            ),
            (
                "FA-004".to_string(),
                "Patent Portfolio".to_string(),
                dec!(800_000),
            ),
        ]
    }

    fn default_config() -> ImpairmentConfig {
        ImpairmentConfig {
            enabled: true,
            test_count: 15,
            impairment_rate: 0.10,
            generate_projections: true,
            include_goodwill: false,
        }
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = ImpairmentGenerator::new(42);
        let assets = sample_assets();
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let results = gen.generate(
            "C001",
            &assets,
            date,
            &default_config(),
            AccountingFramework::UsGaap,
        );

        assert_eq!(results.len(), 15);
        for test in &results {
            assert_eq!(test.company_code, "C001");
            assert_eq!(test.test_date, date);
            assert!(!test.impairment_indicators.is_empty());
            assert!(test.carrying_amount > Decimal::ZERO);
            // Each test should have cash flow projections.
            assert!(!test.cash_flow_projections.is_empty());
            // US GAAP tests should have undiscounted cash flows.
            assert!(test.undiscounted_cash_flows.is_some());
        }
    }

    #[test]
    fn test_deterministic() {
        let assets = sample_assets();
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let config = default_config();

        let mut gen1 = ImpairmentGenerator::new(99);
        let mut gen2 = ImpairmentGenerator::new(99);

        let r1 = gen1.generate("C001", &assets, date, &config, AccountingFramework::Ifrs);
        let r2 = gen2.generate("C001", &assets, date, &config, AccountingFramework::Ifrs);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.test_id, b.test_id);
            assert_eq!(a.asset_id, b.asset_id);
            assert_eq!(a.asset_type, b.asset_type);
            assert_eq!(a.carrying_amount, b.carrying_amount);
            assert_eq!(a.fair_value_less_costs, b.fair_value_less_costs);
            assert_eq!(a.value_in_use, b.value_in_use);
            assert_eq!(a.impairment_loss, b.impairment_loss);
            assert_eq!(a.test_result, b.test_result);
            assert_eq!(a.discount_rate, b.discount_rate);
            assert_eq!(a.cash_flow_projections.len(), b.cash_flow_projections.len());
        }
    }

    #[test]
    fn test_impairment_rate_respected() {
        // Use a higher rate so we can verify the enforcement logic.
        let config = ImpairmentConfig {
            enabled: true,
            test_count: 50,
            impairment_rate: 0.40,
            generate_projections: true,
            include_goodwill: true,
        };

        let assets = sample_assets();
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let mut gen = ImpairmentGenerator::new(123);

        let results = gen.generate("C001", &assets, date, &config, AccountingFramework::Ifrs);

        let impaired_count = results
            .iter()
            .filter(|t| t.test_result == ImpairmentTestResult::Impaired)
            .count();

        let target = (50.0_f64 * 0.40).round() as usize; // 20
                                                         // Allow +/- 1 tolerance since enforcement works iteratively.
        assert!(
            impaired_count >= target.saturating_sub(1) && impaired_count <= target + 1,
            "Expected ~{target} impaired, got {impaired_count}"
        );
    }

    #[test]
    fn test_us_gaap_vs_ifrs() {
        let assets = sample_assets();
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let config = ImpairmentConfig {
            enabled: true,
            test_count: 10,
            impairment_rate: 0.20,
            generate_projections: true,
            include_goodwill: false,
        };

        let mut gen_gaap = ImpairmentGenerator::new(77);
        let mut gen_ifrs = ImpairmentGenerator::new(77);

        let gaap_results =
            gen_gaap.generate("C001", &assets, date, &config, AccountingFramework::UsGaap);
        let ifrs_results =
            gen_ifrs.generate("C001", &assets, date, &config, AccountingFramework::Ifrs);

        // Both should produce the same number of tests.
        assert_eq!(gaap_results.len(), ifrs_results.len());

        // US GAAP tests must all have undiscounted_cash_flows set.
        for test in &gaap_results {
            assert!(
                test.undiscounted_cash_flows.is_some(),
                "US GAAP test should have undiscounted cash flows"
            );
            assert_eq!(test.framework, AccountingFramework::UsGaap);
        }

        // IFRS tests should NOT have undiscounted_cash_flows.
        for test in &ifrs_results {
            assert!(
                test.undiscounted_cash_flows.is_none(),
                "IFRS test should not have undiscounted cash flows"
            );
            assert_eq!(test.framework, AccountingFramework::Ifrs);
        }

        // Due to different framework logic, impairment losses may differ even
        // with the same seed -- the RNG sequence is identical, but US GAAP
        // uses the two-step model while IFRS uses recoverable amount directly.
        // We just verify structural correctness here.
        for test in gaap_results.iter().chain(ifrs_results.iter()) {
            if test.test_result == ImpairmentTestResult::Impaired {
                assert!(
                    test.impairment_loss > Decimal::ZERO,
                    "Impaired test should have positive loss"
                );
            } else {
                assert_eq!(
                    test.impairment_loss,
                    Decimal::ZERO,
                    "Not-impaired test should have zero loss"
                );
            }
        }
    }
}
