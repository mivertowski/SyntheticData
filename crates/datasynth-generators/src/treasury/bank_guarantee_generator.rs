//! Bank Guarantee Generator.
//!
//! Creates bank guarantees and letters of credit for an entity, selecting
//! beneficiaries from the vendor pool and issuing banks from a fixed pool.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_config::schema::BankGuaranteeSchemaConfig;
use datasynth_core::models::{BankGuarantee, GuaranteeStatus, GuaranteeType};

// ---------------------------------------------------------------------------
// Issuing bank pool
// ---------------------------------------------------------------------------

const ISSUING_BANKS: &[&str] = &[
    "Deutsche Bank",
    "HSBC",
    "Citibank",
    "BNP Paribas",
    "Standard Chartered",
    "JPMorgan Chase",
    "Barclays",
    "Commerzbank",
    "Societe Generale",
    "ING Bank",
];

const GUARANTEE_TYPES: &[GuaranteeType] = &[
    GuaranteeType::PerformanceBond,
    GuaranteeType::BankGuarantee,
    GuaranteeType::StandbyLc,
    GuaranteeType::CommercialLc,
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates bank guarantees and letters of credit.
pub struct BankGuaranteeGenerator {
    rng: ChaCha8Rng,
    config: BankGuaranteeSchemaConfig,
    counter: u64,
}

impl BankGuaranteeGenerator {
    /// Creates a new bank guarantee generator.
    pub fn new(config: BankGuaranteeSchemaConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generates bank guarantees for an entity.
    ///
    /// Uses vendors as beneficiaries. Generates the configured number of
    /// guarantees with random types, amounts (5K-500K), and durations
    /// (90-365 days). Most guarantees are `Active`; roughly 20% are `Expired`.
    pub fn generate(
        &mut self,
        entity_id: &str,
        currency: &str,
        start_date: NaiveDate,
        vendors: &[String],
    ) -> Vec<BankGuarantee> {
        if vendors.is_empty() {
            return Vec::new();
        }

        let count = self.config.count as usize;
        let mut guarantees = Vec::with_capacity(count);

        for _ in 0..count {
            self.counter += 1;
            let id = format!("BG-{:06}", self.counter);

            // Pick guarantee type
            let gt_idx = self.rng.random_range(0..GUARANTEE_TYPES.len());
            let guarantee_type = GUARANTEE_TYPES[gt_idx];

            // Random amount between 5,000 and 500,000
            let amount_f = self.rng.random_range(5_000.0f64..500_000.0);
            let amount = Decimal::try_from(amount_f)
                .unwrap_or(Decimal::new(50_000, 0))
                .round_dp(2);

            // Pick beneficiary from vendors
            let vendor_idx = self.rng.random_range(0..vendors.len());
            let beneficiary = &vendors[vendor_idx];

            // Pick issuing bank
            let bank_idx = self.rng.random_range(0..ISSUING_BANKS.len());
            let issuing_bank = ISSUING_BANKS[bank_idx];

            // Issue date: random offset within the period
            let offset_days = self.rng.random_range(0i64..180);
            let issue_date = start_date + chrono::Duration::days(offset_days);

            // Duration: 90-365 days
            let duration_days = self.rng.random_range(90i64..365);
            let expiry_date = issue_date + chrono::Duration::days(duration_days);

            let mut guarantee = BankGuarantee::new(
                id,
                entity_id,
                guarantee_type,
                amount,
                currency,
                beneficiary.as_str(),
                issuing_bank,
                issue_date,
                expiry_date,
            );

            // ~20% are expired
            if self.rng.random_bool(0.20) {
                guarantee = guarantee.with_status(GuaranteeStatus::Expired);
            }

            guarantees.push(guarantee);
        }

        guarantees
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn sample_vendors() -> Vec<String> {
        vec![
            "Acme Corp".to_string(),
            "Widget Co".to_string(),
            "BuildRight Ltd".to_string(),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let config = BankGuaranteeSchemaConfig {
            enabled: true,
            count: 5,
        };
        let mut gen = BankGuaranteeGenerator::new(config, 42);
        let guarantees = gen.generate("C001", "USD", d("2025-01-01"), &sample_vendors());

        assert_eq!(guarantees.len(), 5);
        for g in &guarantees {
            assert!(g.id.starts_with("BG-"));
            assert_eq!(g.entity_id, "C001");
            assert_eq!(g.currency, "USD");
            assert!(g.amount > Decimal::ZERO);
            assert!(g.expiry_date > g.issue_date);
            assert!(!g.beneficiary.is_empty());
            assert!(!g.issuing_bank.is_empty());
        }
    }

    #[test]
    fn test_deterministic() {
        let config = BankGuaranteeSchemaConfig {
            enabled: true,
            count: 3,
        };
        let vendors = sample_vendors();

        let mut gen1 = BankGuaranteeGenerator::new(config.clone(), 42);
        let r1 = gen1.generate("C001", "USD", d("2025-01-01"), &vendors);

        let mut gen2 = BankGuaranteeGenerator::new(config, 42);
        let r2 = gen2.generate("C001", "USD", d("2025-01-01"), &vendors);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.amount, b.amount);
            assert_eq!(a.guarantee_type, b.guarantee_type);
            assert_eq!(a.beneficiary, b.beneficiary);
            assert_eq!(a.status, b.status);
        }
    }

    #[test]
    fn test_empty_vendors() {
        let config = BankGuaranteeSchemaConfig {
            enabled: true,
            count: 5,
        };
        let mut gen = BankGuaranteeGenerator::new(config, 42);
        let guarantees = gen.generate("C001", "USD", d("2025-01-01"), &[]);

        assert!(guarantees.is_empty());
    }
}
