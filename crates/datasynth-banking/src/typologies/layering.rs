//! Layering chain typology implementation.

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

/// Layering chain pattern injector.
///
/// Layering involves:
/// - Multi-hop transfers to obscure fund trail
/// - Amount slicing to create complexity
/// - Time jitter between hops
/// - Cover traffic insertion for camouflage
pub struct LayeringInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl LayeringInjector {
    /// Create a new layering injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(6200)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate layering chain transactions.
    pub fn generate(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Layering parameters based on sophistication
        let (num_layers, total_amount, jitter_range) = match sophistication {
            Sophistication::Basic => (2..4, 15_000.0..50_000.0, 1..3),
            Sophistication::Standard => (3..5, 30_000.0..100_000.0, 2..5),
            Sophistication::Professional => (4..7, 75_000.0..300_000.0, 3..10),
            Sophistication::Advanced => (5..10, 150_000.0..750_000.0, 5..20),
            Sophistication::StateLevel => (8..15, 500_000.0..3_000_000.0, 10..45),
        };

        let layers = self.rng.gen_range(num_layers);
        let total: f64 = self.rng.gen_range(total_amount);
        let scenario_id = format!("LAY-{:06}", self.rng.gen::<u32>());

        let available_days = (end_date - start_date).num_days().max(1);

        // Initial placement
        let placement_date = start_date;
        let placement_timestamp = self.random_timestamp(placement_date);

        let placement_txn = BankTransaction::new(
            self.uuid_factory.next(),
            account.account_id,
            Decimal::from_f64_retain(total).unwrap_or(Decimal::ZERO),
            &account.currency,
            Direction::Inbound,
            TransactionChannel::Wire,
            TransactionCategory::TransferIn,
            CounterpartyRef::business("Initial Source LLC"),
            "Initial transfer",
            placement_timestamp,
        )
        .mark_suspicious(AmlTypology::Layering, &scenario_id)
        .with_laundering_stage(LaunderingStage::Placement)
        .with_scenario(&scenario_id, 0);

        transactions.push(placement_txn);

        // Generate layering hops
        let mut current_amount = total;
        let mut current_date = placement_date;
        let mut seq = 1u32;

        for layer in 0..layers {
            // Time jitter between layers
            let jitter = self.rng.gen_range(jitter_range.clone()) as i64;
            current_date += chrono::Duration::days(jitter);

            if current_date > end_date {
                current_date = end_date;
            }

            // Slice amount for complexity (for professional+ sophistication)
            let num_slices = if matches!(
                sophistication,
                Sophistication::Professional
                    | Sophistication::Advanced
                    | Sophistication::StateLevel
            ) {
                self.rng.gen_range(2..4)
            } else {
                1
            };

            let mut remaining = current_amount;

            for slice in 0..num_slices {
                let slice_amount = if slice == num_slices - 1 {
                    remaining * 0.98 // Small "fee" deduction
                } else {
                    let portion = remaining / ((num_slices - slice) as f64);
                    let variance = portion * 0.2;
                    self.rng
                        .gen_range((portion - variance)..(portion + variance))
                };
                remaining -= slice_amount;

                // Outbound transfer
                let out_timestamp = self.random_timestamp(current_date);
                let (out_channel, counterparty_name) = self.random_layer_destination(layer);

                let out_txn = BankTransaction::new(
                    self.uuid_factory.next(),
                    account.account_id,
                    Decimal::from_f64_retain(slice_amount).unwrap_or(Decimal::ZERO),
                    &account.currency,
                    Direction::Outbound,
                    out_channel,
                    TransactionCategory::TransferOut,
                    CounterpartyRef::business(&counterparty_name),
                    &format!("Layer {} transfer {}", layer + 1, slice + 1),
                    out_timestamp,
                )
                .mark_suspicious(AmlTypology::Layering, &scenario_id)
                .with_laundering_stage(LaunderingStage::Layering)
                .with_scenario(&scenario_id, seq);

                transactions.push(out_txn);
                seq += 1;

                // Corresponding inbound (simulating round-trip or return)
                if layer < layers - 1 && self.rng.gen::<f64>() < 0.6 {
                    let return_jitter = self.rng.gen_range(1..3) as i64;
                    let return_date = current_date + chrono::Duration::days(return_jitter);
                    let return_timestamp = self.random_timestamp(return_date);

                    let return_amount = slice_amount * 0.97; // More fees

                    let in_txn = BankTransaction::new(
                        self.uuid_factory.next(),
                        account.account_id,
                        Decimal::from_f64_retain(return_amount).unwrap_or(Decimal::ZERO),
                        &account.currency,
                        Direction::Inbound,
                        TransactionChannel::Wire,
                        TransactionCategory::TransferIn,
                        CounterpartyRef::business(&format!("Intermediary {} Holdings", layer + 1)),
                        &format!("Return transfer layer {}", layer + 1),
                        return_timestamp,
                    )
                    .mark_suspicious(AmlTypology::Layering, &scenario_id)
                    .with_laundering_stage(LaunderingStage::Layering)
                    .with_scenario(&scenario_id, seq);

                    transactions.push(in_txn);
                    seq += 1;
                    current_amount = return_amount;
                }
            }
        }

        // Insert cover traffic for professional+ sophistication
        if matches!(
            sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            let cover_count = match sophistication {
                Sophistication::Professional => 2..5,
                Sophistication::Advanced => 4..8,
                Sophistication::StateLevel => 6..12,
                _ => 1..2,
            };

            for _ in 0..self.rng.gen_range(cover_count) {
                let cover_day = self.rng.gen_range(0..available_days);
                let cover_date = start_date + chrono::Duration::days(cover_day);
                let cover_timestamp = self.random_timestamp(cover_date);

                // Cover traffic - legitimate-looking transactions
                let cover_amount = self.rng.gen_range(100.0..5000.0);
                let direction = if self.rng.gen::<bool>() {
                    Direction::Inbound
                } else {
                    Direction::Outbound
                };

                let cover_txn = BankTransaction::new(
                    self.uuid_factory.next(),
                    account.account_id,
                    Decimal::from_f64_retain(cover_amount).unwrap_or(Decimal::ZERO),
                    &account.currency,
                    direction,
                    TransactionChannel::CardPresent,
                    TransactionCategory::Shopping,
                    CounterpartyRef::merchant_by_name("Regular Merchant", "5411"),
                    "Regular purchase",
                    cover_timestamp,
                )
                .mark_suspicious(AmlTypology::Layering, &scenario_id)
                .with_laundering_stage(LaunderingStage::Layering)
                .with_scenario(&scenario_id, seq);

                transactions.push(cover_txn);
                seq += 1;
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

    /// Generate random layer destination.
    fn random_layer_destination(&mut self, layer: usize) -> (TransactionChannel, String) {
        let destinations = [
            (
                TransactionChannel::Wire,
                format!("Offshore Holdings {}", layer + 1),
            ),
            (
                TransactionChannel::Ach,
                format!("Investment Co {}", layer + 1),
            ),
            (
                TransactionChannel::Swift,
                format!("Trade Finance {} Ltd", layer + 1),
            ),
            (
                TransactionChannel::Wire,
                format!("Consulting {} LLC", layer + 1),
            ),
        ];

        let idx = self.rng.gen_range(0..destinations.len());
        destinations[idx].clone()
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.gen_range(6..22);
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
    fn test_layering_generation() {
        let mut injector = LayeringInjector::new(12345);

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
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let transactions = injector.generate(
            &customer,
            &account,
            start,
            end,
            Sophistication::Professional,
        );

        assert!(!transactions.is_empty());

        // Should have multiple layering transactions
        assert!(transactions.len() >= 3);

        // All should be marked as layering
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::Layering));
        }
    }
}
