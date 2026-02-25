//! Round-tripping typology implementation.

#![allow(clippy::too_many_arguments)]

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
use crate::seed_offsets::ROUND_TRIPPING_INJECTOR_SEED_OFFSET;

/// Round-tripping pattern injector.
///
/// Round-tripping involves:
/// - Funds leaving the country and returning via affiliates/shells
/// - Complex ownership structures to obscure beneficial ownership
/// - Transfer pricing manipulation
/// - Trade-based laundering variants
pub struct RoundTrippingInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl RoundTrippingInjector {
    /// Create a new round-tripping injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(ROUND_TRIPPING_INJECTOR_SEED_OFFSET)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate round-tripping transactions.
    pub fn generate(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Round-tripping parameters based on sophistication
        let (num_trips, total_amount, trip_delay_days) = match sophistication {
            Sophistication::Basic => (1..2, 25_000.0..75_000.0, 7..14),
            Sophistication::Standard => (2..4, 50_000.0..200_000.0, 10..21),
            Sophistication::Professional => (3..6, 100_000.0..500_000.0, 14..30),
            Sophistication::Advanced => (4..8, 250_000.0..1_000_000.0, 21..45),
            Sophistication::StateLevel => (6..12, 750_000.0..5_000_000.0, 30..60),
        };

        let trips = self.rng.random_range(num_trips);
        let base_amount: f64 = self.rng.random_range(total_amount);
        let scenario_id = format!("RND-{:06}", self.rng.random::<u32>());

        let _available_days = (end_date - start_date).num_days().max(1);
        let mut current_date = start_date;
        let mut seq = 0u32;

        for trip in 0..trips {
            // Outbound leg - money leaves to offshore entity
            let outbound_date = current_date;
            let outbound_timestamp = self.random_timestamp(outbound_date);

            // Amount varies slightly each trip (with "fees")
            let trip_amount = base_amount * (0.95 + self.rng.random::<f64>() * 0.1);
            let offshore_entity = self.random_offshore_entity(trip);

            let outbound_txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(trip_amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                TransactionChannel::Swift,
                TransactionCategory::TransferOut,
                CounterpartyRef::business(&offshore_entity.0),
                &format!("Investment in {} - {}", offshore_entity.1, trip + 1),
                outbound_timestamp,
            )
            .mark_suspicious(AmlTypology::RoundTripping, &scenario_id)
            .with_laundering_stage(LaunderingStage::Layering)
            .with_scenario(&scenario_id, seq);

            transactions.push(outbound_txn);
            seq += 1;

            // Delay before return
            let delay = self.rng.random_range(trip_delay_days.clone()) as i64;
            current_date = outbound_date + chrono::Duration::days(delay);

            if current_date > end_date {
                current_date = end_date;
            }

            // Inbound leg - money returns from different offshore entity
            let inbound_date = current_date;
            let inbound_timestamp = self.random_timestamp(inbound_date);

            // Return amount varies (profits, fees, etc.)
            let return_multiplier = match sophistication {
                Sophistication::Basic => 0.98..1.02,
                Sophistication::Standard => 0.95..1.10,
                Sophistication::Professional => 0.90..1.20,
                Sophistication::Advanced => 0.85..1.30,
                Sophistication::StateLevel => 0.80..1.50,
            };
            let return_amount = trip_amount * self.rng.random_range(return_multiplier);

            let return_entity = self.random_return_entity(trip);
            let return_reference = self.random_return_reference();

            let inbound_txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(return_amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Inbound,
                TransactionChannel::Swift,
                TransactionCategory::TransferIn,
                CounterpartyRef::business(&return_entity),
                &return_reference,
                inbound_timestamp,
            )
            .mark_suspicious(AmlTypology::RoundTripping, &scenario_id)
            .with_laundering_stage(LaunderingStage::Integration)
            .with_scenario(&scenario_id, seq);

            transactions.push(inbound_txn);
            seq += 1;

            // For sophisticated cases, add intermediate transactions
            if matches!(
                sophistication,
                Sophistication::Professional
                    | Sophistication::Advanced
                    | Sophistication::StateLevel
            ) {
                self.add_intermediate_transactions(
                    &mut transactions,
                    account,
                    outbound_date,
                    inbound_date,
                    &scenario_id,
                    &mut seq,
                    sophistication,
                );
            }

            // Move to next trip
            let gap = self.rng.random_range(3..10) as i64;
            current_date += chrono::Duration::days(gap);

            if current_date > end_date - chrono::Duration::days(trip_delay_days.start as i64) {
                break;
            }
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

    /// Add intermediate transactions for more sophisticated schemes.
    fn add_intermediate_transactions(
        &mut self,
        transactions: &mut Vec<BankTransaction>,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        scenario_id: &str,
        seq: &mut u32,
        sophistication: Sophistication,
    ) {
        let num_intermediate = match sophistication {
            Sophistication::Professional => self.rng.random_range(1..3),
            Sophistication::Advanced => self.rng.random_range(2..5),
            Sophistication::StateLevel => self.rng.random_range(3..8),
            _ => 0,
        };

        let available_days = (end_date - start_date).num_days().max(1);

        for i in 0..num_intermediate {
            let day_offset = self.rng.random_range(1..available_days);
            let txn_date = start_date + chrono::Duration::days(day_offset);
            let timestamp = self.random_timestamp(txn_date);

            // Small intermediate transfers to add complexity
            let amount = self.rng.random_range(1_000.0..25_000.0);
            let direction = if self.rng.random::<bool>() {
                Direction::Outbound
            } else {
                Direction::Inbound
            };

            let intermediary = format!("Intermediary {} Ltd", i + 1);
            let reference = format!("Advisory fee payment {}", i + 1);

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                direction,
                TransactionChannel::Wire,
                TransactionCategory::Other,
                CounterpartyRef::business(&intermediary),
                &reference,
                timestamp,
            )
            .mark_suspicious(AmlTypology::RoundTripping, scenario_id)
            .with_laundering_stage(LaunderingStage::Layering)
            .with_scenario(scenario_id, *seq);

            transactions.push(txn);
            *seq += 1;
        }
    }

    /// Generate random offshore entity destination.
    fn random_offshore_entity(&mut self, index: usize) -> (String, String) {
        let entities = [
            ("Cayman Holding Co Ltd", "Cayman Islands"),
            ("BVI Investment Corp", "British Virgin Islands"),
            ("Singapore Ventures Pte Ltd", "Singapore"),
            ("Luxembourg Capital SA", "Luxembourg"),
            ("Cyprus Trading Ltd", "Cyprus"),
            ("Malta Holdings Ltd", "Malta"),
            ("Jersey Finance Ltd", "Jersey"),
            ("Guernsey Trust Ltd", "Guernsey"),
            ("Panama Investments SA", "Panama"),
            ("Delaware Holdings LLC", "Delaware"),
        ];

        let idx = (index + self.rng.random_range(0..entities.len())) % entities.len();
        (entities[idx].0.to_string(), entities[idx].1.to_string())
    }

    /// Generate random return entity name.
    fn random_return_entity(&mut self, _index: usize) -> String {
        let entities = [
            "Global Trade Finance Ltd",
            "International Consulting Services",
            "Worldwide Investment Partners",
            "Pacific Rim Holdings",
            "Atlantic Capital Management",
            "European Trading Company",
            "Asian Growth Fund",
            "Mediterranean Investments",
            "Nordic Ventures AB",
            "Swiss Financial Services AG",
        ];

        entities[self.rng.random_range(0..entities.len())].to_string()
    }

    /// Generate random return reference.
    fn random_return_reference(&mut self) -> String {
        let references = [
            "Dividend distribution",
            "Investment return",
            "Consulting fees",
            "Management fee rebate",
            "Performance bonus",
            "Profit share",
            "Loan repayment",
            "Capital return",
            "Advisory fee",
            "Commission payment",
        ];

        references[self.rng.random_range(0..references.len())].to_string()
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.random_range(8..18);
        let minute: u32 = self.rng.random_range(0..60);
        let second: u32 = self.rng.random_range(0..60);

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
    fn test_round_tripping_generation() {
        let mut injector = RoundTrippingInjector::new(12345);

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
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let transactions = injector.generate(
            &customer,
            &account,
            start,
            end,
            Sophistication::Professional,
        );

        assert!(!transactions.is_empty());

        // Should have pairs of outbound/inbound transactions
        assert!(transactions.len() >= 2);

        // All should be marked as round-tripping
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::RoundTripping));
        }
    }

    #[test]
    fn test_round_tripping_has_both_directions() {
        let mut injector = RoundTrippingInjector::new(54321);

        let customer = BankingCustomer::new_business(
            Uuid::new_v4(),
            "Test Corp",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let account = BankAccount::new(
            Uuid::new_v4(),
            "****5678".to_string(),
            datasynth_core::models::banking::BankAccountType::BusinessOperating,
            customer.customer_id,
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let transactions =
            injector.generate(&customer, &account, start, end, Sophistication::Standard);

        let has_outbound = transactions
            .iter()
            .any(|t| t.direction == Direction::Outbound);
        let has_inbound = transactions
            .iter()
            .any(|t| t.direction == Direction::Inbound);

        assert!(has_outbound, "Should have outbound transactions");
        assert!(has_inbound, "Should have inbound transactions (return leg)");
    }
}
