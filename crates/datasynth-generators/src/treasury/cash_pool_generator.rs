//! Cash Pool Sweep Generator.
//!
//! Groups entity bank accounts into pools and generates daily sweep
//! transactions. Supports zero-balancing (all participant balances → 0,
//! header gets net), physical pooling, and notional pooling.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::CashPoolingConfig;
use datasynth_core::models::{CashPool, CashPoolSweep, PoolType};

use chrono::NaiveTime;

// ---------------------------------------------------------------------------
// Input abstraction
// ---------------------------------------------------------------------------

/// End-of-day balance for a bank account, used as input for sweep generation.
#[derive(Debug, Clone)]
pub struct AccountBalance {
    /// Bank account identifier
    pub account_id: String,
    /// End-of-day balance before sweep
    pub balance: Decimal,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates cash pool structures and daily sweep transactions.
pub struct CashPoolGenerator {
    rng: ChaCha8Rng,
    config: CashPoolingConfig,
    pool_counter: u64,
    sweep_counter: u64,
}

impl CashPoolGenerator {
    /// Creates a new cash pool generator.
    pub fn new(config: CashPoolingConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            pool_counter: 0,
            sweep_counter: 0,
        }
    }

    /// Creates a cash pool from a list of participant accounts.
    ///
    /// The first account is designated as the header account.
    pub fn create_pool(
        &mut self,
        name: &str,
        _currency: &str,
        account_ids: &[String],
    ) -> Option<CashPool> {
        if account_ids.len() < 2 {
            return None; // Need at least header + 1 participant
        }

        self.pool_counter += 1;
        let pool_type = self.parse_pool_type();
        let sweep_time = self.parse_sweep_time();
        let interest_benefit = dec!(0.0025)
            + Decimal::try_from(self.rng.random_range(0.0f64..0.003)).unwrap_or(Decimal::ZERO);

        let mut pool = CashPool::new(
            format!("POOL-{:06}", self.pool_counter),
            name,
            pool_type,
            &account_ids[0],
            sweep_time,
        )
        .with_interest_rate_benefit(interest_benefit.round_dp(4));

        for account_id in &account_ids[1..] {
            pool = pool.with_participant(account_id);
        }

        Some(pool)
    }

    /// Generates sweep transactions for a pool on a given date.
    ///
    /// For zero-balancing: each participant's balance is swept to/from the
    /// header account, leaving the participant at zero.
    /// For physical pooling: only positive balances above a threshold are swept.
    pub fn generate_sweeps(
        &mut self,
        pool: &CashPool,
        date: NaiveDate,
        currency: &str,
        participant_balances: &[AccountBalance],
    ) -> Vec<CashPoolSweep> {
        match pool.pool_type {
            PoolType::ZeroBalancing => {
                self.generate_zero_balance_sweeps(pool, date, currency, participant_balances)
            }
            PoolType::PhysicalPooling => {
                self.generate_physical_sweeps(pool, date, currency, participant_balances)
            }
            PoolType::NotionalPooling => {
                // Notional pooling doesn't move physical cash — no sweeps generated
                Vec::new()
            }
        }
    }

    /// Zero-balancing: sweep each participant's entire balance to/from header.
    fn generate_zero_balance_sweeps(
        &mut self,
        pool: &CashPool,
        date: NaiveDate,
        currency: &str,
        balances: &[AccountBalance],
    ) -> Vec<CashPoolSweep> {
        let mut sweeps = Vec::new();

        for bal in balances {
            if bal.account_id == pool.header_account_id || bal.balance.is_zero() {
                continue;
            }

            self.sweep_counter += 1;
            let (from, to, amount) = if bal.balance > Decimal::ZERO {
                // Positive balance: sweep from participant to header
                (&bal.account_id, &pool.header_account_id, bal.balance)
            } else {
                // Negative balance: fund from header to participant
                (&pool.header_account_id, &bal.account_id, bal.balance.abs())
            };

            sweeps.push(CashPoolSweep {
                id: format!("SWP-{:06}", self.sweep_counter),
                pool_id: pool.id.clone(),
                date,
                from_account_id: from.clone(),
                to_account_id: to.clone(),
                amount,
                currency: currency.to_string(),
            });
        }

        sweeps
    }

    /// Physical pooling: sweep balances above minimum to header.
    fn generate_physical_sweeps(
        &mut self,
        pool: &CashPool,
        date: NaiveDate,
        currency: &str,
        balances: &[AccountBalance],
    ) -> Vec<CashPoolSweep> {
        let min_balance = dec!(10000); // keep minimum in sub-accounts
        let mut sweeps = Vec::new();

        for bal in balances {
            if bal.account_id == pool.header_account_id {
                continue;
            }

            let excess = bal.balance - min_balance;
            if excess > Decimal::ZERO {
                self.sweep_counter += 1;
                sweeps.push(CashPoolSweep {
                    id: format!("SWP-{:06}", self.sweep_counter),
                    pool_id: pool.id.clone(),
                    date,
                    from_account_id: bal.account_id.clone(),
                    to_account_id: pool.header_account_id.clone(),
                    amount: excess,
                    currency: currency.to_string(),
                });
            }
        }

        sweeps
    }

    fn parse_pool_type(&self) -> PoolType {
        match self.config.pool_type.as_str() {
            "physical_pooling" => PoolType::PhysicalPooling,
            "notional_pooling" => PoolType::NotionalPooling,
            _ => PoolType::ZeroBalancing,
        }
    }

    fn parse_sweep_time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.config.sweep_time, "%H:%M")
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(16, 0, 0).expect("valid constant time"))
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

    #[test]
    fn test_zero_balancing_sweeps() {
        let mut gen = CashPoolGenerator::new(CashPoolingConfig::default(), 42);
        let pool = gen
            .create_pool(
                "EUR Pool",
                "EUR",
                &[
                    "BA-HEADER".to_string(),
                    "BA-001".to_string(),
                    "BA-002".to_string(),
                    "BA-003".to_string(),
                ],
            )
            .unwrap();

        let balances = vec![
            AccountBalance {
                account_id: "BA-001".to_string(),
                balance: dec!(50000),
            },
            AccountBalance {
                account_id: "BA-002".to_string(),
                balance: dec!(-20000),
            },
            AccountBalance {
                account_id: "BA-003".to_string(),
                balance: dec!(0), // zero = no sweep
            },
        ];

        let sweeps = gen.generate_sweeps(&pool, d("2025-01-15"), "EUR", &balances);

        assert_eq!(sweeps.len(), 2); // BA-003 is zero, no sweep

        // BA-001 positive → swept to header
        let s1 = sweeps
            .iter()
            .find(|s| s.from_account_id == "BA-001")
            .unwrap();
        assert_eq!(s1.to_account_id, "BA-HEADER");
        assert_eq!(s1.amount, dec!(50000));

        // BA-002 negative → funded from header
        let s2 = sweeps.iter().find(|s| s.to_account_id == "BA-002").unwrap();
        assert_eq!(s2.from_account_id, "BA-HEADER");
        assert_eq!(s2.amount, dec!(20000));
    }

    #[test]
    fn test_physical_pooling_sweeps() {
        let config = CashPoolingConfig {
            pool_type: "physical_pooling".to_string(),
            ..CashPoolingConfig::default()
        };
        let mut gen = CashPoolGenerator::new(config, 42);
        let pool = gen
            .create_pool(
                "USD Pool",
                "USD",
                &[
                    "BA-HEADER".to_string(),
                    "BA-001".to_string(),
                    "BA-002".to_string(),
                ],
            )
            .unwrap();

        let balances = vec![
            AccountBalance {
                account_id: "BA-001".to_string(),
                balance: dec!(50000), // excess = 40000 (50000 - 10000 min)
            },
            AccountBalance {
                account_id: "BA-002".to_string(),
                balance: dec!(5000), // below minimum, no sweep
            },
        ];

        let sweeps = gen.generate_sweeps(&pool, d("2025-01-15"), "USD", &balances);
        assert_eq!(sweeps.len(), 1);
        assert_eq!(sweeps[0].amount, dec!(40000)); // 50000 - 10000 minimum
    }

    #[test]
    fn test_notional_pooling_no_sweeps() {
        let config = CashPoolingConfig {
            pool_type: "notional_pooling".to_string(),
            ..CashPoolingConfig::default()
        };
        let mut gen = CashPoolGenerator::new(config, 42);
        let pool = gen
            .create_pool(
                "EUR Pool",
                "EUR",
                &["BA-HEADER".to_string(), "BA-001".to_string()],
            )
            .unwrap();

        let balances = vec![AccountBalance {
            account_id: "BA-001".to_string(),
            balance: dec!(100000),
        }];

        let sweeps = gen.generate_sweeps(&pool, d("2025-01-15"), "EUR", &balances);
        assert!(sweeps.is_empty());
    }

    #[test]
    fn test_pool_creation() {
        let mut gen = CashPoolGenerator::new(CashPoolingConfig::default(), 42);
        let pool = gen
            .create_pool(
                "Test Pool",
                "USD",
                &[
                    "BA-HDR".to_string(),
                    "BA-001".to_string(),
                    "BA-002".to_string(),
                ],
            )
            .unwrap();

        assert_eq!(pool.header_account_id, "BA-HDR");
        assert_eq!(pool.participant_accounts.len(), 2);
        assert_eq!(pool.total_accounts(), 3);
        assert_eq!(pool.pool_type, PoolType::ZeroBalancing);
    }

    #[test]
    fn test_pool_requires_minimum_accounts() {
        let mut gen = CashPoolGenerator::new(CashPoolingConfig::default(), 42);
        // Single account should return None
        let pool = gen.create_pool("Bad Pool", "USD", &["BA-001".to_string()]);
        assert!(pool.is_none());

        // Empty should return None
        let pool = gen.create_pool("Empty Pool", "USD", &[]);
        assert!(pool.is_none());
    }

    #[test]
    fn test_header_account_excluded_from_sweeps() {
        let mut gen = CashPoolGenerator::new(CashPoolingConfig::default(), 42);
        let pool = gen
            .create_pool(
                "Pool",
                "USD",
                &["BA-HEADER".to_string(), "BA-001".to_string()],
            )
            .unwrap();

        let balances = vec![
            AccountBalance {
                account_id: "BA-HEADER".to_string(),
                balance: dec!(500000), // header balance should not be swept
            },
            AccountBalance {
                account_id: "BA-001".to_string(),
                balance: dec!(30000),
            },
        ];

        let sweeps = gen.generate_sweeps(&pool, d("2025-01-15"), "USD", &balances);
        assert_eq!(sweeps.len(), 1);
        // Only BA-001 should be swept, not header
        assert_eq!(sweeps[0].from_account_id, "BA-001");
    }
}
