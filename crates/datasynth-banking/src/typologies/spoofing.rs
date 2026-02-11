//! Spoofing engine for adversarial AML pattern camouflage.

use chrono::Timelike;
use datasynth_core::models::banking::TransactionCategory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use crate::config::SpoofingConfig;
use crate::models::{BankTransaction, BankingCustomer};

/// Spoofing engine for adversarial mode.
///
/// Spoofing makes suspicious transactions appear more legitimate by:
/// - Aligning timing to customer's normal cadence
/// - Sampling amounts from customer's historical distribution
/// - Using merchant categories consistent with persona
/// - Adjusting velocity to match baseline behavior
/// - Adding longer dwell times to avoid detection
pub struct SpoofingEngine {
    config: SpoofingConfig,
    rng: ChaCha8Rng,
}

impl SpoofingEngine {
    /// Create a new spoofing engine.
    pub fn new(config: SpoofingConfig, seed: u64) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(6400)),
        }
    }

    /// Apply spoofing to a transaction.
    pub fn apply(&mut self, txn: &mut BankTransaction, customer: &BankingCustomer) {
        if !self.config.enabled || txn.spoofing_intensity.is_none() {
            return;
        }

        let intensity = txn.spoofing_intensity.unwrap_or(0.0);

        // Adjust timing to look more natural
        if self.config.spoof_timing && self.rng.gen::<f64>() < intensity {
            self.spoof_timing(txn, customer);
        }

        // Adjust amounts to fit customer profile
        if self.config.spoof_amounts && self.rng.gen::<f64>() < intensity {
            self.spoof_amount(txn, customer);
        }

        // Use persona-appropriate merchant categories
        if self.config.spoof_merchants && self.rng.gen::<f64>() < intensity {
            self.spoof_merchant(txn, customer);
        }

        // Add delays to reduce velocity signatures
        if self.config.add_delays && self.rng.gen::<f64>() < intensity {
            self.add_timing_jitter(txn);
        }
    }

    /// Spoof transaction timing to match customer patterns.
    fn spoof_timing(&mut self, txn: &mut BankTransaction, _customer: &BankingCustomer) {
        // Adjust timestamp to business hours for this customer type
        let current_hour = txn.timestamp_initiated.hour();

        // Move suspicious late-night/early-morning transactions to business hours
        if !(7..=22).contains(&current_hour) {
            let new_hour = self.rng.gen_range(9..18);
            let new_minute = self.rng.gen_range(0..60);
            let new_second = self.rng.gen_range(0..60);

            if let Some(new_time) = txn
                .timestamp_initiated
                .date_naive()
                .and_hms_opt(new_hour, new_minute, new_second)
            {
                txn.timestamp_initiated =
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                        new_time,
                        chrono::Utc,
                    );
            }
        }
    }

    /// Spoof transaction amount to fit customer profile.
    fn spoof_amount(&mut self, txn: &mut BankTransaction, _customer: &BankingCustomer) {
        // Add noise to make amounts less conspicuous
        // Avoid round numbers that trigger rules
        let amount_f64: f64 = txn.amount.try_into().unwrap_or(0.0);

        // Add small random cents
        let cents = self.rng.gen_range(1..99) as f64 / 100.0;
        let new_amount = amount_f64 + cents;

        // Avoid threshold-adjacent amounts (like $9,999)
        let new_amount = self.avoid_thresholds(new_amount);

        txn.amount = Decimal::from_f64_retain(new_amount).unwrap_or(txn.amount);
    }

    /// Avoid amounts near reporting thresholds.
    fn avoid_thresholds(&mut self, amount: f64) -> f64 {
        let thresholds = [10_000.0, 5_000.0, 3_000.0, 1_000.0];

        for threshold in thresholds {
            let lower_bound = threshold * 0.95;
            let upper_bound = threshold * 1.05;

            if amount > lower_bound && amount < upper_bound {
                // Move amount away from threshold
                if self.rng.gen::<bool>() {
                    return threshold * 0.85 + self.rng.gen_range(0.0..100.0);
                } else {
                    return threshold * 1.15 + self.rng.gen_range(0.0..100.0);
                }
            }
        }

        amount
    }

    /// Spoof merchant to match customer persona.
    fn spoof_merchant(&mut self, txn: &mut BankTransaction, customer: &BankingCustomer) {
        // Use categories typical for this customer's persona
        let persona_categories = self.get_persona_categories(customer);

        if !persona_categories.is_empty() {
            let idx = self.rng.gen_range(0..persona_categories.len());
            txn.category = persona_categories[idx];
        }
    }

    /// Get typical merchant categories for a customer persona.
    fn get_persona_categories(&self, customer: &BankingCustomer) -> Vec<TransactionCategory> {
        use crate::models::PersonaVariant;
        use datasynth_core::models::banking::RetailPersona;

        match &customer.persona {
            Some(PersonaVariant::Retail(persona)) => match persona {
                RetailPersona::Student => vec![
                    TransactionCategory::Shopping,
                    TransactionCategory::Dining,
                    TransactionCategory::Entertainment,
                    TransactionCategory::Subscription,
                ],
                RetailPersona::EarlyCareer => vec![
                    TransactionCategory::Shopping,
                    TransactionCategory::Dining,
                    TransactionCategory::Subscription,
                    TransactionCategory::Transportation,
                ],
                RetailPersona::MidCareer => vec![
                    TransactionCategory::Groceries,
                    TransactionCategory::Shopping,
                    TransactionCategory::Utilities,
                    TransactionCategory::Insurance,
                ],
                RetailPersona::Retiree => vec![
                    TransactionCategory::Healthcare,
                    TransactionCategory::Groceries,
                    TransactionCategory::Utilities,
                ],
                RetailPersona::HighNetWorth => vec![
                    TransactionCategory::Investment,
                    TransactionCategory::Entertainment,
                    TransactionCategory::Shopping,
                ],
                RetailPersona::GigWorker => vec![
                    TransactionCategory::Shopping,
                    TransactionCategory::Transportation,
                    TransactionCategory::Dining,
                ],
                _ => vec![
                    TransactionCategory::Shopping,
                    TransactionCategory::Groceries,
                ],
            },
            Some(PersonaVariant::Business(_)) => vec![
                TransactionCategory::TransferOut,
                TransactionCategory::Utilities,
                TransactionCategory::Other,
            ],
            Some(PersonaVariant::Trust(_)) => vec![
                TransactionCategory::Investment,
                TransactionCategory::Other,
                TransactionCategory::Charity,
            ],
            None => vec![TransactionCategory::Shopping],
        }
    }

    /// Add timing jitter to reduce velocity detection.
    fn add_timing_jitter(&mut self, txn: &mut BankTransaction) {
        // Add random minutes to the timestamp
        let jitter_minutes = self.rng.gen_range(-30..30);
        txn.timestamp_initiated += chrono::Duration::minutes(jitter_minutes as i64);
    }

    /// Calculate spoofing effectiveness score.
    pub fn calculate_effectiveness(
        &self,
        txn: &BankTransaction,
        customer: &BankingCustomer,
    ) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Check timing alignment
        let hour = txn.timestamp_initiated.hour();
        if (9..=17).contains(&hour) {
            score += 1.0;
        }
        factors += 1;

        // Check amount naturalness
        let amount: f64 = txn.amount.try_into().unwrap_or(0.0);
        let has_cents = (amount * 100.0) % 100.0 != 0.0;
        if has_cents {
            score += 0.5;
        }
        // Not near thresholds
        if !(9_000.0..=11_000.0).contains(&amount) {
            score += 0.5;
        }
        factors += 1;

        // Check category alignment
        let expected = self.get_persona_categories(customer);
        if expected.contains(&txn.category) {
            score += 1.0;
        }
        factors += 1;

        score / factors as f64
    }
}

/// Spoofing statistics for reporting.
#[derive(Debug, Clone, Default)]
pub struct SpoofingStats {
    /// Number of transactions spoofed
    pub transactions_spoofed: usize,
    /// Number of timing adjustments
    pub timing_adjustments: usize,
    /// Number of amount adjustments
    pub amount_adjustments: usize,
    /// Number of merchant category changes
    pub merchant_changes: usize,
    /// Average spoofing effectiveness
    pub avg_effectiveness: f64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_spoofing_engine() {
        let config = SpoofingConfig {
            enabled: true,
            intensity: 0.5,
            spoof_timing: true,
            spoof_amounts: true,
            spoof_merchants: true,
            spoof_geography: false,
            add_delays: true,
        };

        let mut engine = SpoofingEngine::new(config, 12345);

        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let account_id = Uuid::new_v4();
        let mut txn = BankTransaction::new(
            Uuid::new_v4(),
            account_id,
            Decimal::from(9999),
            "USD",
            datasynth_core::models::banking::Direction::Outbound,
            datasynth_core::models::banking::TransactionChannel::Wire,
            TransactionCategory::TransferOut,
            crate::models::CounterpartyRef::person("Test"),
            "Test transaction",
            chrono::Utc::now(),
        );

        txn.spoofing_intensity = Some(0.8);

        engine.apply(&mut txn, &customer);

        // Transaction should have been modified
        // Amount should no longer be exactly 9999
        let amount: f64 = txn.amount.try_into().unwrap();
        assert!(amount != 9999.0); // Either changed or has cents
    }

    #[test]
    fn test_threshold_avoidance() {
        let config = SpoofingConfig::default();
        let mut engine = SpoofingEngine::new(config, 12345);

        // Test amounts near $10k threshold
        let amount = 9_950.0;
        let adjusted = engine.avoid_thresholds(amount);

        // Should be moved away from threshold
        assert!(!(9_500.0..=10_500.0).contains(&adjusted));
    }
}
