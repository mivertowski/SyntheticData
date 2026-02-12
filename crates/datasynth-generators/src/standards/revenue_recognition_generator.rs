//! Revenue Recognition Generator (ASC 606 / IFRS 15).
//!
//! Generates realistic customer contracts with performance obligations,
//! variable consideration components, and revenue recognition schedules
//! following the five-step model:
//!
//! 1. Identify the contract with a customer
//! 2. Identify the performance obligations
//! 3. Determine the transaction price
//! 4. Allocate the transaction price to performance obligations
//! 5. Recognize revenue when (or as) obligations are satisfied

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::LogNormal;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use datasynth_config::schema::RevenueRecognitionConfig;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use datasynth_standards::accounting::revenue::{
    ContractStatus, CustomerContract, ObligationType, PerformanceObligation, SatisfactionPattern,
    VariableConsideration, VariableConsiderationType,
};
use datasynth_standards::framework::AccountingFramework;

/// Realistic company names for generated customer contracts.
const CUSTOMER_NAMES: &[&str] = &[
    "Acme Corp",
    "TechVision Inc",
    "GlobalTrade Solutions",
    "Pinnacle Systems",
    "BlueHorizon Technologies",
    "NovaStar Industries",
    "CrestPoint Partners",
    "Meridian Analytics",
    "Apex Digital",
    "Ironclad Manufacturing",
    "Skyline Logistics",
    "Vantage Financial Group",
    "Quantum Dynamics",
    "Silverline Media",
    "ClearPath Software",
    "Frontier Biotech",
    "Harborview Enterprises",
    "Summit Healthcare",
    "CrossBridge Consulting",
    "EverGreen Energy",
    "Nexus Data Systems",
    "PrimeWave Communications",
    "RedStone Capital",
    "TrueNorth Advisors",
    "Atlas Robotics",
    "BrightEdge Networks",
    "CoreVault Security",
    "Dragonfly Aerospace",
    "Elevation Partners",
    "ForgePoint Materials",
];

/// Descriptions for performance obligations keyed by obligation type.
const GOOD_DESCRIPTIONS: &[&str] = &[
    "Hardware equipment delivery",
    "Manufactured goods shipment",
    "Raw materials supply",
    "Finished product delivery",
    "Spare parts package",
    "Custom fabricated components",
];

const SERVICE_DESCRIPTIONS: &[&str] = &[
    "Professional consulting services",
    "Implementation services",
    "Training and onboarding program",
    "Managed services agreement",
    "Technical support package",
    "System integration services",
];

const LICENSE_DESCRIPTIONS: &[&str] = &[
    "Enterprise software license",
    "Platform subscription license",
    "Intellectual property license",
    "Technology license agreement",
    "Data analytics license",
    "API access license",
];

const SERIES_DESCRIPTIONS: &[&str] = &[
    "Monthly data processing services",
    "Recurring maintenance services",
    "Continuous monitoring services",
    "Periodic compliance reviews",
];

const WARRANTY_DESCRIPTIONS: &[&str] = &[
    "Extended warranty coverage",
    "Premium support warranty",
    "Enhanced service-level warranty",
];

const MATERIAL_RIGHT_DESCRIPTIONS: &[&str] = &[
    "Customer loyalty program credits",
    "Renewal discount option",
    "Volume purchase option",
];

/// Variable consideration type descriptions.
const VC_DESCRIPTIONS: &[(&str, VariableConsiderationType)] = &[
    (
        "Volume discount based on annual purchases",
        VariableConsiderationType::Discount,
    ),
    (
        "Performance rebate for meeting targets",
        VariableConsiderationType::Rebate,
    ),
    (
        "Right of return within 90-day window",
        VariableConsiderationType::RightOfReturn,
    ),
    (
        "Milestone completion bonus",
        VariableConsiderationType::IncentiveBonus,
    ),
    (
        "Late delivery penalty clause",
        VariableConsiderationType::Penalty,
    ),
    (
        "Early payment price concession",
        VariableConsiderationType::PriceConcession,
    ),
    (
        "Sales-based royalty arrangement",
        VariableConsiderationType::Royalty,
    ),
    (
        "Contingent payment on regulatory approval",
        VariableConsiderationType::ContingentPayment,
    ),
];

/// Generator for revenue recognition contracts following ASC 606 / IFRS 15.
///
/// Produces realistic [`CustomerContract`] instances with performance obligations,
/// variable consideration, and progress tracking suitable for financial data
/// generation and audit testing scenarios.
pub struct RevenueRecognitionGenerator {
    /// Seeded random number generator for reproducibility.
    rng: ChaCha8Rng,
    /// UUID factory for contract identifiers.
    uuid_factory: DeterministicUuidFactory,
    /// UUID factory for performance obligation identifiers (sub-discriminator 1).
    obligation_uuid_factory: DeterministicUuidFactory,
}

impl RevenueRecognitionGenerator {
    /// Create a new generator with default configuration.
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed for deterministic generation.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::RevenueRecognition),
            obligation_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::RevenueRecognition,
                1,
            ),
        }
    }

    /// Create a new generator with a custom configuration seed.
    ///
    /// This constructor exists for API symmetry with other generators;
    /// the actual per-generation configuration is passed to [`Self::generate`].
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed for deterministic generation.
    /// * `_config` - Revenue recognition configuration (used at generate time).
    pub fn with_config(seed: u64, _config: &RevenueRecognitionConfig) -> Self {
        Self::new(seed)
    }

    /// Generate a set of customer contracts for the given period.
    ///
    /// Produces `config.contract_count` contracts, each containing one or more
    /// performance obligations with allocated transaction prices and progress
    /// tracking appropriate for the specified accounting framework.
    ///
    /// # Arguments
    ///
    /// * `company_code` - The company code to associate contracts with.
    /// * `customer_ids` - Pool of customer identifiers to draw from.
    /// * `period_start` - Start of the generation period.
    /// * `period_end` - End of the generation period.
    /// * `currency` - ISO currency code (e.g., "USD").
    /// * `config` - Revenue recognition configuration parameters.
    /// * `framework` - Accounting framework (US GAAP, IFRS, or Dual).
    ///
    /// # Returns
    ///
    /// A vector of [`CustomerContract`] instances with fully allocated
    /// performance obligations.
    pub fn generate(
        &mut self,
        company_code: &str,
        customer_ids: &[String],
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
        config: &RevenueRecognitionConfig,
        framework: AccountingFramework,
    ) -> Vec<CustomerContract> {
        if customer_ids.is_empty() {
            return Vec::new();
        }

        let count = config.contract_count;
        let period_days = (period_end - period_start).num_days().max(1);

        let mut contracts = Vec::with_capacity(count);

        for _ in 0..count {
            let contract = self.generate_single_contract(
                company_code,
                customer_ids,
                period_start,
                period_days,
                period_end,
                currency,
                config,
                framework,
            );
            contracts.push(contract);
        }

        contracts
    }

    /// Generate a single customer contract with obligations and variable consideration.
    #[allow(clippy::too_many_arguments)]
    fn generate_single_contract(
        &mut self,
        company_code: &str,
        customer_ids: &[String],
        period_start: NaiveDate,
        period_days: i64,
        period_end: NaiveDate,
        currency: &str,
        config: &RevenueRecognitionConfig,
        framework: AccountingFramework,
    ) -> CustomerContract {
        // Pick a random customer
        let customer_idx = self.rng.gen_range(0..customer_ids.len());
        let customer_id = &customer_ids[customer_idx];

        // Pick a random customer name from the pool
        let name_idx = self.rng.gen_range(0..CUSTOMER_NAMES.len());
        let customer_name = CUSTOMER_NAMES[name_idx];

        // Random inception date within the period
        let offset_days = self.rng.gen_range(0..period_days);
        let inception_date = period_start + chrono::Duration::days(offset_days);

        // Transaction price: log-normal in $5K-$5M range
        let transaction_price = self.generate_transaction_price();

        // Create the contract with a deterministic UUID
        let contract_id = self.uuid_factory.next();
        let mut contract = CustomerContract::new(
            customer_id.as_str(),
            customer_name,
            company_code,
            inception_date,
            transaction_price,
            currency,
            framework,
        );
        contract.contract_id = contract_id;

        // Generate performance obligations
        let num_obligations = self.sample_obligation_count(config.avg_obligations_per_contract);
        let obligations = self.generate_obligations(
            contract.contract_id,
            num_obligations,
            transaction_price,
            config.over_time_recognition_rate,
            inception_date,
            period_end,
        );
        for obligation in obligations {
            contract.add_performance_obligation(obligation);
        }

        // Allocate transaction price proportionally
        self.allocate_transaction_price(&mut contract);

        // Update progress for each obligation
        self.update_obligation_progress(&mut contract, inception_date, period_end);

        // Optionally add variable consideration
        if self
            .rng
            .gen_bool(config.variable_consideration_rate.clamp(0.0, 1.0))
        {
            let vc = self.generate_variable_consideration(contract.contract_id, transaction_price);
            contract.add_variable_consideration(vc);
        }

        // Set contract status
        contract.status = self.pick_contract_status();

        // Set end date for completed/terminated contracts
        match contract.status {
            ContractStatus::Complete | ContractStatus::Terminated => {
                let days_after = self.rng.gen_range(30..365);
                contract.end_date = Some(inception_date + chrono::Duration::days(days_after));
            }
            _ => {}
        }

        contract
    }

    /// Generate a log-normal transaction price clamped to [$5,000, $5,000,000].
    fn generate_transaction_price(&mut self) -> Decimal {
        // Log-normal with mu=10.0, sigma=1.5 gives a wide range centered
        // around ~$22K with a heavy right tail.
        let ln_dist = LogNormal::new(10.0, 1.5).unwrap_or_else(|_| {
            // Fallback to a safe default - this should never fail with valid params
            LogNormal::new(10.0, 1.0).expect("fallback log-normal must succeed")
        });
        let raw: f64 = self.rng.sample(ln_dist);
        let clamped = raw.clamp(5_000.0, 5_000_000.0);

        // Round to 2 decimal places
        let price = Decimal::from_f64_retain(clamped).unwrap_or(Decimal::from(50_000));
        price.round_dp(2)
    }

    /// Sample the number of performance obligations using a Poisson-like distribution
    /// centered on the configured average, clamped to [1, 4].
    fn sample_obligation_count(&mut self, avg: f64) -> u32 {
        // Simple approach: generate from a shifted geometric-like distribution
        // by using the average as a bias factor.
        let base: f64 = self.rng.gen();
        let count = if base < 0.3 {
            1
        } else if base < 0.3 + 0.4 * (avg / 2.0).min(1.0) {
            2
        } else if base < 0.85 {
            3
        } else {
            4
        };
        count.clamp(1, 4)
    }

    /// Generate performance obligations for a contract.
    fn generate_obligations(
        &mut self,
        contract_id: uuid::Uuid,
        count: u32,
        total_price: Decimal,
        over_time_rate: f64,
        inception_date: NaiveDate,
        period_end: NaiveDate,
    ) -> Vec<PerformanceObligation> {
        let mut obligations = Vec::with_capacity(count as usize);

        // Generate standalone selling prices that sum to roughly the total
        // (they won't match exactly -- allocation step handles that)
        let ssp_values = self.generate_standalone_prices(count, total_price);

        for seq in 0..count {
            let ob_type = self.pick_obligation_type();
            let satisfaction = if self.rng.gen_bool(over_time_rate.clamp(0.0, 1.0)) {
                SatisfactionPattern::OverTime
            } else {
                SatisfactionPattern::PointInTime
            };

            let description = self.pick_obligation_description(ob_type);
            let ssp = ssp_values[seq as usize];

            let ob_id = self.obligation_uuid_factory.next();
            let mut obligation = PerformanceObligation::new(
                contract_id,
                seq + 1,
                description,
                ob_type,
                satisfaction,
                ssp,
            );
            obligation.obligation_id = ob_id;

            // Set expected satisfaction date
            let days_to_satisfy = self.rng.gen_range(30..365);
            let expected_date = inception_date + chrono::Duration::days(days_to_satisfy);
            obligation.expected_satisfaction_date =
                Some(expected_date.min(period_end + chrono::Duration::days(365)));

            obligations.push(obligation);
        }

        obligations
    }

    /// Generate standalone selling prices that distribute around the total price.
    fn generate_standalone_prices(&mut self, count: u32, total_price: Decimal) -> Vec<Decimal> {
        if count == 0 {
            return Vec::new();
        }
        if count == 1 {
            return vec![total_price];
        }

        // Generate random weights and normalize
        let mut weights: Vec<f64> = (0..count)
            .map(|_| self.rng.gen_range(0.2_f64..1.0))
            .collect();
        let weight_sum: f64 = weights.iter().sum();

        // Normalize weights
        for w in &mut weights {
            *w /= weight_sum;
        }

        // Apply small markup (5-25%) to each SSP to simulate market pricing
        let mut prices: Vec<Decimal> = weights
            .iter()
            .map(|w| {
                let markup = 1.0 + self.rng.gen_range(0.05..0.25);
                let ssp_f64 = w * total_price.to_f64().unwrap_or(50_000.0) * markup;
                Decimal::from_f64_retain(ssp_f64)
                    .unwrap_or(Decimal::ONE)
                    .round_dp(2)
            })
            .collect();

        // Ensure no zero prices
        for price in &mut prices {
            if *price <= Decimal::ZERO {
                *price = Decimal::from(1_000);
            }
        }

        prices
    }

    /// Allocate the transaction price to obligations proportionally based on SSP.
    fn allocate_transaction_price(&mut self, contract: &mut CustomerContract) {
        let total_ssp: Decimal = contract
            .performance_obligations
            .iter()
            .map(|po| po.standalone_selling_price)
            .sum();

        if total_ssp <= Decimal::ZERO {
            // Fallback: equal allocation
            let per_ob = if contract.performance_obligations.is_empty() {
                Decimal::ZERO
            } else {
                let count_dec = Decimal::from(contract.performance_obligations.len() as u32);
                (contract.transaction_price / count_dec).round_dp(2)
            };
            for po in &mut contract.performance_obligations {
                po.allocated_price = per_ob;
            }
            return;
        }

        let tx_price = contract.transaction_price;
        let mut allocated_total = Decimal::ZERO;

        let ob_count = contract.performance_obligations.len();
        for (i, po) in contract.performance_obligations.iter_mut().enumerate() {
            if i == ob_count - 1 {
                // Last obligation gets the remainder to ensure exact allocation
                po.allocated_price = (tx_price - allocated_total).max(Decimal::ZERO);
            } else {
                let ratio = po.standalone_selling_price / total_ssp;
                po.allocated_price = (tx_price * ratio).round_dp(2);
                allocated_total += po.allocated_price;
            }
            // Initialize deferred revenue to the full allocated price
            po.deferred_revenue = po.allocated_price;
        }
    }

    /// Update progress for each obligation based on how far into the period we are.
    fn update_obligation_progress(
        &mut self,
        contract: &mut CustomerContract,
        inception_date: NaiveDate,
        period_end: NaiveDate,
    ) {
        let total_days = (period_end - inception_date).num_days().max(1) as f64;

        for po in &mut contract.performance_obligations {
            match po.satisfaction_pattern {
                SatisfactionPattern::OverTime => {
                    // Progress proportional to time elapsed, with some randomness
                    let elapsed = (period_end - inception_date).num_days().max(0) as f64;
                    let base_progress = (elapsed / total_days) * 100.0;
                    // Add noise: +/- 15%
                    let noise = self.rng.gen_range(-15.0_f64..15.0);
                    let progress = (base_progress + noise).clamp(5.0, 95.0);
                    let progress_dec =
                        Decimal::from_f64_retain(progress).unwrap_or(Decimal::from(50));
                    po.update_progress(progress_dec, period_end);
                }
                SatisfactionPattern::PointInTime => {
                    // 70% chance the obligation is already satisfied
                    if self.rng.gen_bool(0.70) {
                        // Satisfaction date is some time between inception and period end
                        let max_offset = (period_end - inception_date).num_days().max(1);
                        let sat_offset = self.rng.gen_range(0..max_offset);
                        let sat_date = inception_date + chrono::Duration::days(sat_offset);
                        po.update_progress(Decimal::from(100), sat_date);
                    }
                    // Otherwise remains unsatisfied (0% progress)
                }
            }
        }
    }

    /// Generate a variable consideration component for a contract.
    fn generate_variable_consideration(
        &mut self,
        contract_id: uuid::Uuid,
        transaction_price: Decimal,
    ) -> VariableConsideration {
        let idx = self.rng.gen_range(0..VC_DESCRIPTIONS.len());
        let (description, vc_type) = VC_DESCRIPTIONS[idx];

        // Estimated amount is 5-20% of the transaction price
        let pct = self.rng.gen_range(0.05..0.20);
        let estimated_f64 = transaction_price.to_f64().unwrap_or(50_000.0) * pct;
        let estimated_amount = Decimal::from_f64_retain(estimated_f64)
            .unwrap_or(Decimal::from(5_000))
            .round_dp(2);

        let mut vc =
            VariableConsideration::new(contract_id, vc_type, estimated_amount, description);

        // Apply constraint (80-95% of estimated amount is "highly probable")
        let constraint_pct = self.rng.gen_range(0.80..0.95);
        let constraint_dec = Decimal::from_f64_retain(constraint_pct)
            .unwrap_or(Decimal::from_str("0.85").unwrap_or(Decimal::ONE));
        vc.apply_constraint(constraint_dec);

        vc
    }

    /// Pick a random contract status with the specified distribution:
    /// 60% Active, 15% Complete, 10% Pending, 10% Modified, 5% Terminated.
    fn pick_contract_status(&mut self) -> ContractStatus {
        let roll: f64 = self.rng.gen();
        if roll < 0.60 {
            ContractStatus::Active
        } else if roll < 0.75 {
            ContractStatus::Complete
        } else if roll < 0.85 {
            ContractStatus::Pending
        } else if roll < 0.95 {
            ContractStatus::Modified
        } else {
            ContractStatus::Terminated
        }
    }

    /// Pick a random obligation type with realistic weighting.
    fn pick_obligation_type(&mut self) -> ObligationType {
        let roll: f64 = self.rng.gen();
        if roll < 0.25 {
            ObligationType::Good
        } else if roll < 0.50 {
            ObligationType::Service
        } else if roll < 0.70 {
            ObligationType::License
        } else if roll < 0.82 {
            ObligationType::Series
        } else if roll < 0.92 {
            ObligationType::ServiceTypeWarranty
        } else {
            ObligationType::MaterialRight
        }
    }

    /// Pick a description string appropriate for the obligation type.
    fn pick_obligation_description(&mut self, ob_type: ObligationType) -> &'static str {
        let pool = match ob_type {
            ObligationType::Good => GOOD_DESCRIPTIONS,
            ObligationType::Service => SERVICE_DESCRIPTIONS,
            ObligationType::License => LICENSE_DESCRIPTIONS,
            ObligationType::Series => SERIES_DESCRIPTIONS,
            ObligationType::ServiceTypeWarranty => WARRANTY_DESCRIPTIONS,
            ObligationType::MaterialRight => MATERIAL_RIGHT_DESCRIPTIONS,
        };
        let idx = self.rng.gen_range(0..pool.len());
        pool[idx]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_config() -> RevenueRecognitionConfig {
        RevenueRecognitionConfig {
            enabled: true,
            generate_contracts: true,
            avg_obligations_per_contract: 2.0,
            variable_consideration_rate: 0.15,
            over_time_recognition_rate: 0.30,
            contract_count: 10,
        }
    }

    fn sample_customer_ids() -> Vec<String> {
        (1..=20).map(|i| format!("CUST{:04}", i)).collect()
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = RevenueRecognitionGenerator::new(42);
        let config = default_config();
        let customers = sample_customer_ids();

        let contracts = gen.generate(
            "1000",
            &customers,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            "USD",
            &config,
            AccountingFramework::UsGaap,
        );

        assert_eq!(contracts.len(), 10);

        for contract in &contracts {
            // Every contract should have at least one obligation
            assert!(
                !contract.performance_obligations.is_empty(),
                "Contract {} has no obligations",
                contract.contract_id
            );

            // Every obligation should have a positive allocated price
            for po in &contract.performance_obligations {
                assert!(
                    po.allocated_price > Decimal::ZERO,
                    "Obligation {} has non-positive allocated price: {}",
                    po.obligation_id,
                    po.allocated_price
                );
            }

            // Transaction price should be within expected range
            assert!(
                contract.transaction_price >= Decimal::from(5_000),
                "Transaction price too low: {}",
                contract.transaction_price
            );
            assert!(
                contract.transaction_price <= Decimal::from(5_000_000),
                "Transaction price too high: {}",
                contract.transaction_price
            );

            // Currency and company code should match input
            assert_eq!(contract.currency, "USD");
            assert_eq!(contract.company_code, "1000");
        }
    }

    #[test]
    fn test_deterministic_output() {
        let config = default_config();
        let customers = sample_customer_ids();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let mut gen1 = RevenueRecognitionGenerator::new(12345);
        let contracts1 = gen1.generate(
            "1000",
            &customers,
            start,
            end,
            "USD",
            &config,
            AccountingFramework::UsGaap,
        );

        let mut gen2 = RevenueRecognitionGenerator::new(12345);
        let contracts2 = gen2.generate(
            "1000",
            &customers,
            start,
            end,
            "USD",
            &config,
            AccountingFramework::UsGaap,
        );

        assert_eq!(contracts1.len(), contracts2.len());

        for (c1, c2) in contracts1.iter().zip(contracts2.iter()) {
            assert_eq!(c1.contract_id, c2.contract_id);
            assert_eq!(c1.customer_id, c2.customer_id);
            assert_eq!(c1.transaction_price, c2.transaction_price);
            assert_eq!(c1.inception_date, c2.inception_date);
            assert_eq!(
                c1.performance_obligations.len(),
                c2.performance_obligations.len()
            );

            for (po1, po2) in c1
                .performance_obligations
                .iter()
                .zip(c2.performance_obligations.iter())
            {
                assert_eq!(po1.obligation_id, po2.obligation_id);
                assert_eq!(po1.allocated_price, po2.allocated_price);
                assert_eq!(po1.standalone_selling_price, po2.standalone_selling_price);
            }
        }
    }

    #[test]
    fn test_obligation_allocation_sums_to_transaction_price() {
        let mut gen = RevenueRecognitionGenerator::new(99);
        let config = RevenueRecognitionConfig {
            contract_count: 50,
            variable_consideration_rate: 0.0, // disable VC for cleaner test
            ..default_config()
        };
        let customers = sample_customer_ids();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let contracts = gen.generate(
            "2000",
            &customers,
            start,
            end,
            "EUR",
            &config,
            AccountingFramework::Ifrs,
        );

        for contract in &contracts {
            let total_allocated: Decimal = contract
                .performance_obligations
                .iter()
                .map(|po| po.allocated_price)
                .sum();

            assert_eq!(
                total_allocated, contract.transaction_price,
                "Allocation mismatch for contract {}: allocated={} vs transaction_price={}",
                contract.contract_id, total_allocated, contract.transaction_price
            );
        }
    }

    #[test]
    fn test_empty_customer_ids_returns_empty() {
        let mut gen = RevenueRecognitionGenerator::new(1);
        let config = default_config();
        let empty: Vec<String> = vec![];

        let contracts = gen.generate(
            "1000",
            &empty,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            "USD",
            &config,
            AccountingFramework::UsGaap,
        );

        assert!(contracts.is_empty());
    }
}
