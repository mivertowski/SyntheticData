//! Structuring (smurfing) typology implementation.

use chrono::{DateTime, NaiveDate, Utc};
use datasynth_core::models::banking::{
    AmlTypology, Direction, LaunderingStage, Sophistication, TransactionCategory,
    TransactionChannel,
};
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use crate::models::{BankAccount, BankTransaction, BankingCustomer, CounterpartyRef};

/// Structuring pattern injector.
pub struct StructuringInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl StructuringInjector {
    /// Create a new structuring injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(6000)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate structuring transactions.
    pub fn generate(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Structuring: multiple deposits just below $10,000 threshold
        let threshold = 10_000.0;
        let total_amount: f64 = self.rng.gen_range(30_000.0..100_000.0);

        // Number of deposits based on sophistication
        let num_deposits = match sophistication {
            Sophistication::Basic => self.rng.gen_range(3..6),
            Sophistication::Standard => self.rng.gen_range(5..10),
            Sophistication::Professional => self.rng.gen_range(8..15),
            Sophistication::Advanced => self.rng.gen_range(12..25),
            Sophistication::StateLevel => self.rng.gen_range(20..40),
        };

        // Time spread based on sophistication
        let days_spread = match sophistication {
            Sophistication::Basic => 3,
            Sophistication::Standard => 7,
            Sophistication::Professional => 14,
            Sophistication::Advanced => 30,
            Sophistication::StateLevel => 60,
        };

        let available_days = (end_date - start_date).num_days().max(1) as u32;
        let actual_spread = days_spread.min(available_days);

        let mut remaining = total_amount;
        let scenario_id = format!("STR-{:06}", self.rng.gen::<u32>());

        for i in 0..num_deposits {
            if remaining <= 0.0 {
                break;
            }

            // Amount just below threshold with some variation
            let max_deposit = threshold * 0.99;
            let min_deposit = threshold * 0.80;
            let deposit_amount = if remaining > max_deposit {
                self.rng.gen_range(min_deposit..max_deposit)
            } else {
                remaining.min(max_deposit)
            };

            remaining -= deposit_amount;

            // Time distribution
            let day_offset = if actual_spread > 0 {
                self.rng.gen_range(0..actual_spread) as i64
            } else {
                0
            };
            let date = start_date + chrono::Duration::days(day_offset);
            let timestamp = self.random_timestamp(date);

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(deposit_amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Inbound,
                TransactionChannel::Cash,
                TransactionCategory::CashDeposit,
                CounterpartyRef::atm("Branch"),
                &format!("Cash deposit #{}", i + 1),
                timestamp,
            )
            .mark_suspicious(AmlTypology::Structuring, &scenario_id)
            .with_laundering_stage(LaunderingStage::Placement)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);
        }

        // Apply spoofing for sophisticated patterns
        if matches!(
            sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            for txn in &mut transactions {
                txn.is_spoofed = true;
                txn.spoofing_intensity = Some(sophistication.spoofing_intensity());
            }
        }

        transactions
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.gen_range(9..17); // Business hours
        let minute: u32 = self.rng.gen_range(0..60);
        let second: u32 = self.rng.gen_range(0..60);

        date.and_hms_opt(hour, minute, second)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_structuring_generation() {
        let mut injector = StructuringInjector::new(12345);

        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            datasynth_core::models::banking::BankAccountType::Checking,
            customer.customer_id,
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let transactions =
            injector.generate(&customer, &account, start, end, Sophistication::Basic);

        assert!(!transactions.is_empty());

        // All should be suspicious structuring
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::Structuring));
            // Amount should be below $10k
            let amount_f64: f64 = txn.amount.try_into().unwrap();
            assert!(amount_f64 < 10_000.0);
        }
    }
}
