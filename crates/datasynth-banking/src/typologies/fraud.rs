//! Fraud typology implementations (account takeover, fake vendors, BEC).

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use datasynth_core::models::banking::{
    AmlTypology, Direction, LaunderingStage, Sophistication, TransactionCategory,
    TransactionChannel,
};
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use crate::models::{BankAccount, BankTransaction, BankingCustomer, CounterpartyRef};
use crate::seed_offsets::FRAUD_INJECTOR_SEED_OFFSET;

/// Fraud pattern injector.
///
/// Covers multiple fraud typologies:
/// - Account takeover (unauthorized access)
/// - Fake vendor schemes
/// - Business email compromise (BEC)
/// - Authorized push payment (APP) fraud
pub struct FraudInjector {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl FraudInjector {
    /// Create a new fraud injector.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(FRAUD_INJECTOR_SEED_OFFSET)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Anomaly,
            ),
        }
    }

    /// Generate account takeover fraud transactions.
    ///
    /// Pattern: Unauthorized access followed by rapid fund extraction
    pub fn generate_account_takeover(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        _end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // ATO parameters based on sophistication
        let (num_extractions, max_amount, time_window_hours) = match sophistication {
            Sophistication::Basic => (1..3, 5_000.0, 1..4),
            Sophistication::Standard => (2..4, 15_000.0, 1..8),
            Sophistication::Professional => (3..6, 50_000.0, 2..12),
            Sophistication::Advanced => (4..8, 100_000.0, 4..24),
            Sophistication::StateLevel => (5..10, 250_000.0, 8..48),
        };

        let extractions = self.rng.random_range(num_extractions);
        let scenario_id = format!("ATO-{:06}", self.rng.random::<u32>());

        // Account takeover typically happens in a short window
        let takeover_date = start_date;
        let mut current_hour = self.rng.random_range(0..12);

        for i in 0..extractions {
            // Time progresses within the window
            let hour_offset = self.rng.random_range(time_window_hours.clone());
            current_hour = (current_hour + hour_offset) % 24;

            let timestamp = takeover_date
                .and_hms_opt(
                    current_hour as u32,
                    self.rng.random_range(0..60),
                    self.rng.random_range(0..60),
                )
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or_else(Utc::now);

            // Each extraction varies in amount
            let amount = self.rng.random_range(500.0..max_amount);

            // Varying channels for extraction
            let (channel, category, counterparty, reference) = self.random_ato_extraction(i);

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                channel,
                category,
                counterparty,
                &reference,
                timestamp,
            )
            .mark_suspicious(AmlTypology::AccountTakeover, &scenario_id)
            .with_laundering_stage(LaunderingStage::NotApplicable)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);
        }

        // For sophisticated cases, add reconnaissance-like activity
        if matches!(
            sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            // Small test transactions before main extraction
            let test_timestamp = takeover_date
                .and_hms_opt(
                    self.rng.random_range(6..12),
                    self.rng.random_range(0..60),
                    0,
                )
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or_else(Utc::now);

            let test_txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(self.rng.random_range(1.0..10.0)).unwrap_or(Decimal::ONE),
                &account.currency,
                Direction::Outbound,
                TransactionChannel::CardNotPresent,
                TransactionCategory::Shopping,
                CounterpartyRef::merchant_by_name("Test Merchant", "5999"),
                "Small test purchase",
                test_timestamp,
            )
            .mark_suspicious(AmlTypology::AccountTakeover, &scenario_id)
            .with_laundering_stage(LaunderingStage::NotApplicable)
            .with_scenario(&scenario_id, extractions as u32);

            transactions.insert(0, test_txn);
        }

        transactions
    }

    /// Generate fake vendor fraud transactions.
    ///
    /// Pattern: Fictitious vendor receiving payments for non-existent goods/services
    pub fn generate_fake_vendor(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Fake vendor parameters based on sophistication
        let (num_payments, payment_range, interval_days) = match sophistication {
            Sophistication::Basic => (2..4, 1_000.0..10_000.0, 7..14),
            Sophistication::Standard => (3..6, 5_000.0..30_000.0, 14..30),
            Sophistication::Professional => (4..8, 10_000.0..75_000.0, 21..45),
            Sophistication::Advanced => (6..12, 25_000.0..150_000.0, 30..60),
            Sophistication::StateLevel => (8..20, 50_000.0..500_000.0, 45..90),
        };

        let payments = self.rng.random_range(num_payments);
        let scenario_id = format!("FKV-{:06}", self.rng.random::<u32>());

        // Create the fake vendor
        let fake_vendor = self.random_fake_vendor();
        let available_days = (end_date - start_date).num_days().max(1);
        let mut current_date = start_date;

        for i in 0..payments {
            let timestamp = self.random_timestamp(current_date);
            let amount = self.rng.random_range(payment_range.clone());

            // Create invoice-like reference
            let invoice_ref = format!(
                "INV-{:04}-{:06}",
                current_date.year() % 100,
                self.rng.random::<u32>() % 1_000_000
            );

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                TransactionChannel::Ach,
                TransactionCategory::Other,
                CounterpartyRef::business(&fake_vendor.0),
                &format!("{} - {}", fake_vendor.1, invoice_ref),
                timestamp,
            )
            .mark_suspicious(AmlTypology::FakeVendor, &scenario_id)
            .with_laundering_stage(LaunderingStage::Placement)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);

            // Move to next payment date
            let interval = self.rng.random_range(interval_days.clone()) as i64;
            current_date += chrono::Duration::days(interval);

            if current_date > end_date || (current_date - start_date).num_days() > available_days {
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

    /// Generate business email compromise (BEC) fraud transactions.
    ///
    /// Pattern: Large urgent payment to "new" bank details, often international
    pub fn generate_bec(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        _end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // BEC typically involves 1-3 large payments
        let (num_payments, amount_range) = match sophistication {
            Sophistication::Basic => (1..2, 25_000.0..75_000.0),
            Sophistication::Standard => (1..2, 50_000.0..150_000.0),
            Sophistication::Professional => (1..3, 100_000.0..500_000.0),
            Sophistication::Advanced => (2..3, 250_000.0..1_000_000.0),
            Sophistication::StateLevel => (2..4, 500_000.0..5_000_000.0),
        };

        let payments = self.rng.random_range(num_payments);
        let scenario_id = format!("BEC-{:06}", self.rng.random::<u32>());

        // BEC typically happens quickly after the initial compromise
        let mut current_date = start_date;

        for i in 0..payments {
            let timestamp = self.random_timestamp(current_date);
            let amount = self.rng.random_range(amount_range.clone());

            let (recipient, reference) = self.random_bec_recipient();

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                TransactionChannel::Swift, // International wire
                TransactionCategory::Other,
                CounterpartyRef::business(&recipient),
                &reference,
                timestamp,
            )
            .mark_suspicious(AmlTypology::BusinessEmailCompromise, &scenario_id)
            .with_laundering_stage(LaunderingStage::Placement)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);

            // Short interval between BEC payments
            current_date += chrono::Duration::days(self.rng.random_range(1..3));
        }

        transactions
    }

    /// Generate authorized push payment (APP) fraud transactions.
    ///
    /// Pattern: Victim manipulated into authorizing payments to fraudster
    pub fn generate_app_fraud(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sophistication: Sophistication,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // APP fraud parameters
        let (num_payments, amount_range, urgency_factor) = match sophistication {
            Sophistication::Basic => (1..2, 500.0..5_000.0, 0.8),
            Sophistication::Standard => (2..4, 1_000.0..15_000.0, 0.7),
            Sophistication::Professional => (3..6, 5_000.0..50_000.0, 0.6),
            Sophistication::Advanced => (4..8, 10_000.0..100_000.0, 0.5),
            Sophistication::StateLevel => (5..10, 25_000.0..250_000.0, 0.4),
        };

        let payments = self.rng.random_range(num_payments);
        let scenario_id = format!("APP-{:06}", self.rng.random::<u32>());

        let scam_type = self.random_app_scam_type();
        let available_days = (end_date - start_date).num_days().max(1);
        let mut current_date = start_date;

        for i in 0..payments {
            let timestamp = self.random_timestamp(current_date);
            let amount = self.rng.random_range(amount_range.clone());

            let txn = BankTransaction::new(
                self.uuid_factory.next(),
                account.account_id,
                Decimal::from_f64_retain(amount).unwrap_or(Decimal::ZERO),
                &account.currency,
                Direction::Outbound,
                TransactionChannel::RealTimePayment,
                TransactionCategory::TransferOut,
                CounterpartyRef::person(&scam_type.0),
                &scam_type.1,
                timestamp,
            )
            .mark_suspicious(AmlTypology::AuthorizedPushPayment, &scenario_id)
            .with_laundering_stage(LaunderingStage::NotApplicable)
            .with_scenario(&scenario_id, i as u32);

            transactions.push(txn);

            // Interval between payments (urgency = shorter intervals)
            let base_interval = self.rng.random_range(1..7) as f64;
            let interval = (base_interval * urgency_factor).max(1.0) as i64;
            current_date += chrono::Duration::days(interval);

            if current_date > end_date || (current_date - start_date).num_days() > available_days {
                break;
            }
        }

        transactions
    }

    /// Generate random ATO extraction method.
    fn random_ato_extraction(
        &mut self,
        index: usize,
    ) -> (
        TransactionChannel,
        TransactionCategory,
        CounterpartyRef,
        String,
    ) {
        let extractions = [
            (
                TransactionChannel::Wire,
                TransactionCategory::TransferOut,
                CounterpartyRef::person("External Account"),
                "External transfer".to_string(),
            ),
            (
                TransactionChannel::Ach,
                TransactionCategory::TransferOut,
                CounterpartyRef::person("Linked Account"),
                "ACH transfer out".to_string(),
            ),
            (
                TransactionChannel::CardNotPresent,
                TransactionCategory::Shopping,
                CounterpartyRef::merchant_by_name("Online Store", "5999"),
                "Online purchase".to_string(),
            ),
            (
                TransactionChannel::Atm,
                TransactionCategory::AtmWithdrawal,
                CounterpartyRef::atm("ATM"),
                "ATM withdrawal".to_string(),
            ),
            (
                TransactionChannel::CardNotPresent,
                TransactionCategory::Shopping,
                CounterpartyRef::merchant_by_name("Gift Card Vendor", "5815"),
                "Gift card purchase".to_string(),
            ),
        ];

        let idx = (index + self.rng.random_range(0..extractions.len())) % extractions.len();
        extractions[idx].clone()
    }

    /// Generate random fake vendor details.
    fn random_fake_vendor(&mut self) -> (String, String) {
        let vendors = [
            ("ABC Consulting Services LLC", "Consulting services"),
            ("Generic Supplies Inc", "Office supplies"),
            ("Tech Solutions Partners", "IT services"),
            ("Professional Services Group", "Professional fees"),
            ("Strategic Advisory LLC", "Advisory services"),
            ("Business Support Services", "Business support"),
            ("Enterprise Solutions Corp", "Enterprise solutions"),
            ("Market Research Associates", "Research services"),
            ("Quality Assurance Partners", "QA services"),
            ("Operational Excellence LLC", "Operations consulting"),
        ];

        let idx = self.rng.random_range(0..vendors.len());
        (vendors[idx].0.to_string(), vendors[idx].1.to_string())
    }

    /// Generate random BEC recipient details.
    fn random_bec_recipient(&mut self) -> (String, String) {
        let recipients = [
            (
                "International Trade Co Ltd",
                "URGENT: Updated payment details - Invoice payment",
            ),
            (
                "Overseas Partner Holdings",
                "Wire transfer - NEW BANK DETAILS",
            ),
            (
                "Foreign Supplier Pte Ltd",
                "Payment for goods - UPDATED ACCOUNT",
            ),
            (
                "Global Trading Services",
                "URGENT: Supplier payment - new instructions",
            ),
            (
                "Asian Manufacturing Ltd",
                "Invoice settlement - REVISED BANK",
            ),
        ];

        let idx = self.rng.random_range(0..recipients.len());
        (recipients[idx].0.to_string(), recipients[idx].1.to_string())
    }

    /// Generate random APP scam type.
    fn random_app_scam_type(&mut self) -> (String, String) {
        let scam_types = [
            ("HMRC Tax Department", "Tax refund processing fee"),
            ("Investment Advisor", "Investment opportunity"),
            ("Tech Support Services", "Computer repair services"),
            ("Romantic Partner", "Emergency funds needed"),
            ("Police Officer", "Safe account transfer"),
            ("Bank Security", "Account protection transfer"),
            ("Lottery Commission", "Prize claim fee"),
            ("Crypto Investment", "Cryptocurrency investment"),
        ];

        let idx = self.rng.random_range(0..scam_types.len());
        (scam_types[idx].0.to_string(), scam_types[idx].1.to_string())
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.random_range(6..23);
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

    fn create_test_customer() -> BankingCustomer {
        BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
    }

    fn create_test_account(customer: &BankingCustomer) -> BankAccount {
        BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            datasynth_core::models::banking::BankAccountType::Checking,
            customer.customer_id,
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
    }

    #[test]
    fn test_account_takeover_generation() {
        let mut injector = FraudInjector::new(12345);
        let customer = create_test_customer();
        let account = create_test_account(&customer);

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();

        let transactions = injector.generate_account_takeover(
            &customer,
            &account,
            start,
            end,
            Sophistication::Standard,
        );

        assert!(!transactions.is_empty());

        // All should be outbound (extraction)
        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::AccountTakeover));
            assert_eq!(txn.direction, Direction::Outbound);
        }
    }

    #[test]
    fn test_fake_vendor_generation() {
        let mut injector = FraudInjector::new(54321);
        let customer = create_test_customer();
        let account = create_test_account(&customer);

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let transactions = injector.generate_fake_vendor(
            &customer,
            &account,
            start,
            end,
            Sophistication::Professional,
        );

        assert!(!transactions.is_empty());
        assert!(transactions.len() >= 4); // Professional has 4-8 payments

        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(txn.suspicion_reason, Some(AmlTypology::FakeVendor));
        }
    }

    #[test]
    fn test_bec_generation() {
        let mut injector = FraudInjector::new(11111);
        let customer = create_test_customer();
        let account = create_test_account(&customer);

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();

        let transactions =
            injector.generate_bec(&customer, &account, start, end, Sophistication::Advanced);

        assert!(!transactions.is_empty());

        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(
                txn.suspicion_reason,
                Some(AmlTypology::BusinessEmailCompromise)
            );
            // BEC typically uses SWIFT for international wires
            assert_eq!(txn.channel, TransactionChannel::Swift);
        }
    }

    #[test]
    fn test_app_fraud_generation() {
        let mut injector = FraudInjector::new(99999);
        let customer = create_test_customer();
        let account = create_test_account(&customer);

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let transactions =
            injector.generate_app_fraud(&customer, &account, start, end, Sophistication::Standard);

        assert!(!transactions.is_empty());

        for txn in &transactions {
            assert!(txn.is_suspicious);
            assert_eq!(
                txn.suspicion_reason,
                Some(AmlTypology::AuthorizedPushPayment)
            );
        }
    }
}
