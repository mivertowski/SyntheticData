//! Money mule typology implementation.

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
use crate::seed_offsets::MULE_INJECTOR_SEED_OFFSET;

/// Money mule pattern injector.
///
/// Money mule accounts show:
/// - New account with limited history
/// - Inbound transfers from unknown/unrelated sources
/// - Rapid cash-out via ATM withdrawals or wire transfers
/// - Pattern of receive-and-forward behavior
/// - Little legitimate activity
pub struct MuleInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl MuleInjector {
    /// Create a new mule injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(MULE_INJECTOR_SEED_OFFSET)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate money mule transactions.
    pub fn generate(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Mule parameters based on sophistication
        let (num_cycles, amount_per_cycle, retention_pct) = match sophistication {
            Sophistication::Basic => (1..3, 2_000.0..8_000.0, 0.05..0.10),
            Sophistication::Standard => (2..5, 5_000.0..15_000.0, 0.08..0.15),
            Sophistication::Professional => (3..7, 10_000.0..30_000.0, 0.10..0.20),
            Sophistication::Advanced => (4..10, 20_000.0..75_000.0, 0.12..0.25),
            Sophistication::StateLevel => (6..15, 50_000.0..200_000.0, 0.15..0.30),
        };

        let cycles = self.rng.gen_range(num_cycles);
        let scenario_id = format!("MUL-{:06}", self.rng.gen::<u32>());

        let available_days = (end_date - start_date).num_days().max(1);
        let days_per_cycle = (available_days / cycles as i64).max(2);

        let mut seq = 0u32;

        for cycle in 0..cycles {
            let cycle_start = start_date + chrono::Duration::days(cycle as i64 * days_per_cycle);
            let amount: f64 = self.rng.gen_range(amount_per_cycle.clone());
            let mule_cut: f64 = self.rng.gen_range(retention_pct.clone());

            // Phase 1: Inbound transfer(s)
            let num_inbound = match sophistication {
                Sophistication::Basic => 1,
                Sophistication::Standard => self.rng.gen_range(1..3),
                _ => self.rng.gen_range(2..4),
            };

            let mut total_received = 0.0;
            for i in 0..num_inbound {
                let portion = if i == num_inbound - 1 {
                    amount - total_received
                } else {
                    amount / num_inbound as f64
                };
                total_received += portion;

                let in_day = self.rng.gen_range(0..2) as i64;
                let in_date = cycle_start + chrono::Duration::days(in_day);
                let in_timestamp = self.random_timestamp(in_date);

                let (channel, counterparty) = self.random_mule_source();

                let in_txn = BankTransaction::new(
                    self.uuid_factory.next(),
                    account.account_id,
                    Decimal::from_f64_retain(portion).unwrap_or(Decimal::ZERO),
                    &account.currency,
                    Direction::Inbound,
                    channel,
                    TransactionCategory::TransferIn,
                    counterparty,
                    &format!("Transfer cycle {} - {}", cycle + 1, i + 1),
                    in_timestamp,
                )
                .mark_suspicious(AmlTypology::MoneyMule, &scenario_id)
                .with_laundering_stage(LaunderingStage::Placement)
                .with_scenario(&scenario_id, seq);

                transactions.push(in_txn);
                seq += 1;
            }

            // Phase 2: Rapid cash-out (within 1-3 days)
            let cashout_delay = self.rng.gen_range(1..4) as i64;
            let cashout_date = cycle_start + chrono::Duration::days(cashout_delay);

            let amount_to_forward = total_received * (1.0 - mule_cut);

            // Cash-out method varies by sophistication
            let cashout_methods = match sophistication {
                Sophistication::Basic => vec![CashoutMethod::AtmWithdrawal],
                Sophistication::Standard => {
                    vec![CashoutMethod::AtmWithdrawal, CashoutMethod::WireTransfer]
                }
                Sophistication::Professional => vec![
                    CashoutMethod::WireTransfer,
                    CashoutMethod::CryptoExchange,
                    CashoutMethod::GiftCards,
                ],
                Sophistication::Advanced => vec![
                    CashoutMethod::WireTransfer,
                    CashoutMethod::CryptoExchange,
                    CashoutMethod::MoneyOrder,
                ],
                Sophistication::StateLevel => vec![
                    CashoutMethod::WireTransfer,
                    CashoutMethod::InternationalWire,
                    CashoutMethod::CryptoExchange,
                ],
            };

            let num_cashouts = match sophistication {
                Sophistication::Basic => 1..2,
                Sophistication::Standard => 1..3,
                _ => 2..4,
            };

            let cashout_count = self.rng.gen_range(num_cashouts);
            let mut remaining = amount_to_forward;

            for i in 0..cashout_count {
                let cashout_amount = if i == cashout_count - 1 {
                    remaining
                } else {
                    remaining / ((cashout_count - i) as f64) * self.rng.gen_range(0.8..1.2)
                };
                remaining -= cashout_amount;

                let method = cashout_methods[self.rng.gen_range(0..cashout_methods.len())];
                let (channel, category, counterparty, description) = self.cashout_details(method);

                let out_timestamp = self.random_timestamp(cashout_date);

                let out_txn = BankTransaction::new(
                    self.uuid_factory.next(),
                    account.account_id,
                    Decimal::from_f64_retain(cashout_amount).unwrap_or(Decimal::ZERO),
                    &account.currency,
                    Direction::Outbound,
                    channel,
                    category,
                    counterparty,
                    &description,
                    out_timestamp,
                )
                .mark_suspicious(AmlTypology::MoneyMule, &scenario_id)
                .with_laundering_stage(LaunderingStage::Integration)
                .with_scenario(&scenario_id, seq);

                transactions.push(out_txn);
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

    /// Generate random mule source.
    fn random_mule_source(&mut self) -> (TransactionChannel, CounterpartyRef) {
        let source_type = self.rng.gen_range(0..4);
        match source_type {
            0 => (
                TransactionChannel::Ach,
                CounterpartyRef::person(&format!("Unknown Sender {}", self.rng.gen::<u16>())),
            ),
            1 => (
                TransactionChannel::Ach,
                CounterpartyRef::business(&format!("Dubious LLC {}", self.rng.gen::<u16>())),
            ),
            2 => (
                TransactionChannel::Swift,
                CounterpartyRef::international(&format!(
                    "Foreign Account {}",
                    self.rng.gen::<u16>()
                )),
            ),
            _ => (
                TransactionChannel::Wire,
                CounterpartyRef::person(&format!("Contact {}", self.rng.gen::<u16>())),
            ),
        }
    }

    /// Get cash-out transaction details.
    fn cashout_details(
        &mut self,
        method: CashoutMethod,
    ) -> (
        TransactionChannel,
        TransactionCategory,
        CounterpartyRef,
        String,
    ) {
        match method {
            CashoutMethod::AtmWithdrawal => (
                TransactionChannel::Atm,
                TransactionCategory::AtmWithdrawal,
                CounterpartyRef::atm("ATM"),
                "Cash withdrawal".to_string(),
            ),
            CashoutMethod::WireTransfer => (
                TransactionChannel::Wire,
                TransactionCategory::TransferOut,
                CounterpartyRef::person(&format!("Recipient {}", self.rng.gen::<u16>())),
                "Wire transfer".to_string(),
            ),
            CashoutMethod::InternationalWire => (
                TransactionChannel::Swift,
                TransactionCategory::InternationalTransfer,
                CounterpartyRef::international(&format!(
                    "Overseas Account {}",
                    self.rng.gen::<u16>()
                )),
                "International wire".to_string(),
            ),
            CashoutMethod::CryptoExchange => (
                TransactionChannel::Wire,
                TransactionCategory::Investment,
                CounterpartyRef::crypto_exchange("CryptoExchange"),
                "Crypto purchase".to_string(),
            ),
            CashoutMethod::GiftCards => (
                TransactionChannel::CardPresent,
                TransactionCategory::Shopping,
                CounterpartyRef::merchant_by_name("Gift Card Retailer", "5999"),
                "Gift card purchase".to_string(),
            ),
            CashoutMethod::MoneyOrder => (
                TransactionChannel::Branch,
                TransactionCategory::Other,
                CounterpartyRef::service("Money Order Service"),
                "Money order".to_string(),
            ),
        }
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.gen_range(7..21);
        let minute: u32 = self.rng.gen_range(0..60);
        let second: u32 = self.rng.gen_range(0..60);

        date.and_hms_opt(hour, minute, second)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now)
    }
}

/// Cash-out method for mule accounts.
#[derive(Debug, Clone, Copy)]
enum CashoutMethod {
    AtmWithdrawal,
    WireTransfer,
    InternationalWire,
    CryptoExchange,
    GiftCards,
    MoneyOrder,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_mule_generation() {
        let mut injector = MuleInjector::new(12345);

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
        let end = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();

        let transactions =
            injector.generate(&customer, &account, start, end, Sophistication::Standard);

        assert!(!transactions.is_empty());

        // Should have both inbound and outbound
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

        // All should be marked as money mule
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::MoneyMule));
        }
    }
}
