//! Funnel account typology implementation.

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
use crate::seed_offsets::FUNNEL_INJECTOR_SEED_OFFSET;

/// Funnel account pattern injector.
///
/// Funnel accounts show:
/// - Many unrelated inbound transfers from different sources
/// - Rapid consolidation and outward movement
/// - Short holding periods
/// - High velocity relative to account age
pub struct FunnelInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl FunnelInjector {
    /// Create a new funnel injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(FUNNEL_INJECTOR_SEED_OFFSET)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate funnel account transactions.
    pub fn generate(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Funnel parameters based on sophistication
        let (num_sources, total_amount, holding_days) = match sophistication {
            Sophistication::Basic => (5..10, 20_000.0..50_000.0, 1..3),
            Sophistication::Standard => (8..15, 50_000.0..150_000.0, 2..5),
            Sophistication::Professional => (12..25, 100_000.0..500_000.0, 3..7),
            Sophistication::Advanced => (20..40, 250_000.0..1_000_000.0, 5..14),
            Sophistication::StateLevel => (30..60, 500_000.0..5_000_000.0, 7..30),
        };

        let num_inbound = self.rng.random_range(num_sources);
        let total: f64 = self.rng.random_range(total_amount);
        let hold_period = self.rng.random_range(holding_days) as i64;

        let available_days = (end_date - start_date).num_days().max(1) as u32;
        let scenario_id = format!("FUN-{:06}", self.rng.random::<u32>());

        // Phase 1: Inbound transfers from multiple sources
        let mut accumulated = 0.0;
        let inbound_window = (available_days as i64 / 3).max(1);

        for i in 0..num_inbound {
            let portion = if i == num_inbound - 1 {
                total - accumulated
            } else {
                let min_portion = total / (num_inbound as f64 * 2.0);
                let max_portion = total / (num_inbound as f64) * 1.5;
                self.rng.random_range(min_portion..max_portion)
            };
            accumulated += portion;

            let day_offset = self.rng.random_range(0..inbound_window);
            let date = start_date + chrono::Duration::days(day_offset);
            let timestamp = self.random_timestamp(date);

            // Vary the source types
            let (channel, category, counterparty) = self.random_inbound_source(i);

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(portion).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Inbound,
                channel,
                category,
                counterparty,
                &format!("Transfer from source {}", i + 1),
                timestamp,
            )
            .mark_suspicious(AmlTypology::FunnelAccount, &scenario_id)
            .with_laundering_stage(LaunderingStage::Layering)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);
        }

        // Phase 2: Consolidation outflow after holding period
        let outflow_start = start_date + chrono::Duration::days(inbound_window + hold_period);
        let num_outbound = match sophistication {
            Sophistication::Basic => 1..3,
            Sophistication::Standard => 2..4,
            Sophistication::Professional => 2..5,
            Sophistication::Advanced => 3..6,
            Sophistication::StateLevel => 4..8,
        };

        let num_out = self.rng.random_range(num_outbound);
        let mut remaining = total * 0.97; // Account for fees

        for i in 0..num_out {
            let amount = if i == num_out - 1 {
                remaining
            } else {
                let portion = remaining / ((num_out - i) as f64);
                let variance = portion * 0.3;
                self.rng
                    .random_range((portion - variance)..(portion + variance))
            };
            remaining -= amount;

            let day_offset = self.rng.random_range(0..3) as i64;
            let date = outflow_start + chrono::Duration::days(day_offset);
            let timestamp = self.random_timestamp(date);

            let (channel, category, counterparty) = self.random_outbound_destination(i);

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                channel,
                category,
                counterparty,
                &format!("Outward transfer {}", i + 1),
                timestamp,
            )
            .mark_suspicious(AmlTypology::FunnelAccount, &scenario_id)
            .with_laundering_stage(LaunderingStage::Layering)
            .with_scenario(&scenario_id, (num_inbound + i) as u32);

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

    /// Generate random inbound source.
    fn random_inbound_source(
        &mut self,
        _index: usize,
    ) -> (TransactionChannel, TransactionCategory, CounterpartyRef) {
        let source_type = self.rng.random_range(0..4);
        match source_type {
            0 => (
                TransactionChannel::Ach,
                TransactionCategory::TransferIn,
                CounterpartyRef::person(&format!("Individual {}", self.rng.random::<u16>())),
            ),
            1 => (
                TransactionChannel::Wire,
                TransactionCategory::TransferIn,
                CounterpartyRef::business(&format!("Company {}", self.rng.random::<u16>())),
            ),
            2 => (
                TransactionChannel::Swift,
                TransactionCategory::InternationalTransfer,
                CounterpartyRef::international(&format!(
                    "Foreign Entity {}",
                    self.rng.random::<u16>()
                )),
            ),
            _ => (
                TransactionChannel::Ach,
                TransactionCategory::TransferIn,
                CounterpartyRef::person(&format!("Sender {}", self.rng.random::<u16>())),
            ),
        }
    }

    /// Generate random outbound destination.
    fn random_outbound_destination(
        &mut self,
        _index: usize,
    ) -> (TransactionChannel, TransactionCategory, CounterpartyRef) {
        let dest_type = self.rng.random_range(0..3);
        match dest_type {
            0 => (
                TransactionChannel::Swift,
                TransactionCategory::InternationalTransfer,
                CounterpartyRef::international(&format!(
                    "Offshore Account {}",
                    self.rng.random::<u16>()
                )),
            ),
            1 => (
                TransactionChannel::Wire,
                TransactionCategory::TransferOut,
                CounterpartyRef::business(&format!("Shell Corp {}", self.rng.random::<u16>())),
            ),
            _ => (
                TransactionChannel::Atm,
                TransactionCategory::AtmWithdrawal,
                CounterpartyRef::atm("ATM"),
            ),
        }
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.random_range(8..20);
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
    fn test_funnel_generation() {
        let mut injector = FunnelInjector::new(12345);

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

        let transactions =
            injector.generate(&customer, &account, start, end, Sophistication::Standard);

        assert!(!transactions.is_empty());

        // Should have mix of inbound and outbound
        let inbound: Vec<_> = transactions
            .iter()
            .filter(|t| t.direction == Direction::Inbound)
            .collect();
        let outbound: Vec<_> = transactions
            .iter()
            .filter(|t| t.direction == Direction::Outbound)
            .collect();

        assert!(!inbound.is_empty());
        assert!(!outbound.is_empty());

        // All should be marked as funnel account
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::FunnelAccount));
        }
    }
}
