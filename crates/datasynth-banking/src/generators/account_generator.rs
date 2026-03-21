//! Account generator for banking data.

use chrono::NaiveDate;
use datasynth_core::models::banking::{AccountFeatures, BankAccountType, BankingCustomerType};
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use crate::config::BankingConfig;
use crate::models::{BankAccount, BankingCustomer, PersonaVariant};
use crate::seed_offsets::ACCOUNT_GENERATOR_SEED_OFFSET;

/// Generator for bank accounts.
pub struct AccountGenerator {
    config: BankingConfig,
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    account_counter: u64,
}

impl AccountGenerator {
    /// Create a new account generator.
    pub fn new(config: BankingConfig, seed: u64) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(ACCOUNT_GENERATOR_SEED_OFFSET)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::ARSubledger,
            ), // Reuse for banking accounts
            account_counter: 0,
        }
    }

    /// Generate accounts for all customers.
    pub fn generate_for_customers(
        &mut self,
        customers: &mut [BankingCustomer],
    ) -> Vec<BankAccount> {
        let mut accounts = Vec::new();

        for customer in customers.iter_mut() {
            let customer_accounts = self.generate_customer_accounts(customer);
            for account in &customer_accounts {
                customer.add_account(account.account_id);
            }
            accounts.extend(customer_accounts);
        }

        accounts
    }

    /// Generate accounts for a single customer.
    pub fn generate_customer_accounts(&mut self, customer: &BankingCustomer) -> Vec<BankAccount> {
        let mut accounts = Vec::new();

        let account_count = self.determine_account_count(customer);

        // Primary checking account
        accounts.push(self.generate_primary_account(customer));

        // Additional accounts based on persona
        for i in 1..account_count {
            accounts.push(self.generate_secondary_account(customer, i));
        }

        accounts
    }

    /// Determine number of accounts for customer.
    fn determine_account_count(&mut self, customer: &BankingCustomer) -> u32 {
        let base_count = match customer.customer_type {
            BankingCustomerType::Retail => self.config.products.avg_accounts_retail,
            BankingCustomerType::Business => self.config.products.avg_accounts_business,
            BankingCustomerType::Trust => 2.0,
            _ => 1.5,
        };

        // Adjust for persona
        let multiplier = match &customer.persona {
            Some(PersonaVariant::Retail(p)) => {
                use datasynth_core::models::banking::RetailPersona;
                match p {
                    RetailPersona::HighNetWorth => 2.0,
                    RetailPersona::MidCareer => 1.5,
                    RetailPersona::Student => 1.0,
                    _ => 1.2,
                }
            }
            Some(PersonaVariant::Business(p)) => {
                use datasynth_core::models::banking::BusinessPersona;
                match p {
                    BusinessPersona::Enterprise => 3.0,
                    BusinessPersona::MidMarket => 2.0,
                    _ => 1.5,
                }
            }
            _ => 1.0,
        };

        let target = base_count * multiplier;
        let variation: f64 = self.rng.random_range(-0.5..0.5);
        ((target + variation).round() as u32).max(1)
    }

    /// Generate primary account for customer.
    fn generate_primary_account(&mut self, customer: &BankingCustomer) -> BankAccount {
        let account_id = self.uuid_factory.next();
        let account_number = self.generate_account_number();

        let account_type = match customer.customer_type {
            BankingCustomerType::Retail => BankAccountType::Checking,
            BankingCustomerType::Business => BankAccountType::BusinessOperating,
            BankingCustomerType::Trust => BankAccountType::TrustAccount,
            _ => BankAccountType::Checking,
        };

        let mut account = BankAccount::new(
            account_id,
            account_number,
            account_type,
            customer.customer_id,
            &self.get_customer_currency(customer),
            customer.onboarding_date,
        );

        // Set appropriate features
        account.features = self.generate_features(customer, true);

        // Set initial balance
        account.current_balance = self.generate_initial_balance(customer);
        account.available_balance = account.current_balance;

        // Set routing info for US accounts
        if customer.residence_country == "US" {
            account.routing_number = Some(self.generate_routing_number());
        }

        // Wire GL account: all bank accounts map to the Cash GL account (1000)
        account.gl_account = Some("1000".to_string());

        account
    }

    /// Generate secondary account for customer.
    fn generate_secondary_account(
        &mut self,
        customer: &BankingCustomer,
        index: u32,
    ) -> BankAccount {
        let account_id = self.uuid_factory.next();
        let account_number = self.generate_account_number();

        let account_type = self.select_secondary_account_type(customer, index);

        let mut account = BankAccount::new(
            account_id,
            account_number,
            account_type,
            customer.customer_id,
            &self.get_customer_currency(customer),
            self.random_opening_date(customer.onboarding_date),
        );

        // Secondary accounts have reduced features
        account.features = self.generate_features(customer, false);

        // Set initial balance
        account.current_balance = self.generate_initial_balance(customer)
            * Decimal::from_f64_retain(0.3).unwrap_or(Decimal::ZERO);
        account.available_balance = account.current_balance;

        // Wire GL account: all bank accounts map to the Cash GL account (1000)
        account.gl_account = Some("1000".to_string());

        account
    }

    /// Select account type for secondary account.
    fn select_secondary_account_type(
        &mut self,
        customer: &BankingCustomer,
        _index: u32,
    ) -> BankAccountType {
        match customer.customer_type {
            BankingCustomerType::Retail => {
                let types = [
                    (BankAccountType::Savings, 0.5),
                    (BankAccountType::MoneyMarket, 0.2),
                    (BankAccountType::CertificateOfDeposit, 0.1),
                    (BankAccountType::Investment, 0.2),
                ];
                self.weighted_select(&types)
            }
            BankingCustomerType::Business => {
                let types = [
                    (BankAccountType::BusinessSavings, 0.4),
                    (BankAccountType::Payroll, 0.3),
                    (BankAccountType::ForeignCurrency, 0.2),
                    (BankAccountType::Escrow, 0.1),
                ];
                self.weighted_select(&types)
            }
            _ => BankAccountType::Savings,
        }
    }

    /// Generate account features.
    fn generate_features(
        &mut self,
        customer: &BankingCustomer,
        is_primary: bool,
    ) -> AccountFeatures {
        let mut features = match customer.customer_type {
            BankingCustomerType::Retail if is_primary => {
                if matches!(
                    customer.persona,
                    Some(PersonaVariant::Retail(
                        datasynth_core::models::banking::RetailPersona::HighNetWorth
                    ))
                ) {
                    AccountFeatures::retail_premium()
                } else {
                    AccountFeatures::retail_standard()
                }
            }
            BankingCustomerType::Business => AccountFeatures::business_standard(),
            _ => AccountFeatures::retail_standard(),
        };

        // Adjust based on config
        if self.rng.random::<f64>() > self.config.products.debit_card_rate {
            features.debit_card = false;
        }
        if self.rng.random::<f64>() > self.config.products.international_rate {
            features.international_transfers = false;
            features.wire_transfers = false;
        }

        // Non-primary accounts have fewer features
        if !is_primary {
            features.debit_card = false;
            features.check_writing = false;
        }

        features
    }

    /// Generate initial balance.
    fn generate_initial_balance(&mut self, customer: &BankingCustomer) -> Decimal {
        let base_balance = match &customer.persona {
            Some(PersonaVariant::Retail(p)) => {
                use datasynth_core::models::banking::RetailPersona;
                match p {
                    RetailPersona::Student => self.rng.random_range(100.0..2_000.0),
                    RetailPersona::EarlyCareer => self.rng.random_range(500.0..10_000.0),
                    RetailPersona::MidCareer => self.rng.random_range(2_000.0..50_000.0),
                    RetailPersona::Retiree => self.rng.random_range(5_000.0..100_000.0),
                    RetailPersona::HighNetWorth => self.rng.random_range(50_000.0..1_000_000.0),
                    RetailPersona::GigWorker => self.rng.random_range(200.0..5_000.0),
                    _ => self.rng.random_range(500.0..5_000.0),
                }
            }
            Some(PersonaVariant::Business(p)) => {
                use datasynth_core::models::banking::BusinessPersona;
                match p {
                    BusinessPersona::SmallBusiness => self.rng.random_range(5_000.0..100_000.0),
                    BusinessPersona::MidMarket => self.rng.random_range(50_000.0..1_000_000.0),
                    BusinessPersona::Enterprise => self.rng.random_range(500_000.0..10_000_000.0),
                    BusinessPersona::CashIntensive => self.rng.random_range(10_000.0..200_000.0),
                    _ => self.rng.random_range(10_000.0..200_000.0),
                }
            }
            _ => self.rng.random_range(1_000.0..10_000.0),
        };

        Decimal::from_f64_retain(base_balance).unwrap_or(Decimal::ZERO)
    }

    /// Generate account number.
    fn generate_account_number(&mut self) -> String {
        self.account_counter += 1;
        format!("****{:04}", self.account_counter % 10000)
    }

    /// Generate routing number.
    fn generate_routing_number(&mut self) -> String {
        let routing_prefixes = [
            "021", "026", "031", "041", "051", "061", "071", "081", "091",
        ];
        let prefix = routing_prefixes
            .choose(&mut self.rng)
            .expect("non-empty array");
        format!("{}{:06}", prefix, self.rng.random_range(0..1_000_000))
    }

    /// Get customer's currency.
    fn get_customer_currency(&self, customer: &BankingCustomer) -> String {
        match customer.residence_country.as_str() {
            "US" => "USD",
            "GB" => "GBP",
            "CA" => "CAD",
            "DE" | "FR" | "NL" => "EUR",
            "JP" => "JPY",
            "AU" => "AUD",
            "CH" => "CHF",
            "SG" => "SGD",
            _ => "USD",
        }
        .to_string()
    }

    /// Generate random opening date after onboarding.
    fn random_opening_date(&mut self, onboarding: NaiveDate) -> NaiveDate {
        let days_after: i64 = self.rng.random_range(30..365);
        onboarding + chrono::Duration::days(days_after)
    }

    /// Weighted random selection.
    fn weighted_select<T: Copy>(&mut self, options: &[(T, f64)]) -> T {
        let total: f64 = options.iter().map(|(_, w)| w).sum();
        let roll: f64 = self.rng.random::<f64>() * total;
        let mut cumulative = 0.0;

        for (item, weight) in options {
            cumulative += weight;
            if roll < cumulative {
                return *item;
            }
        }

        options.last().expect("options must not be empty").0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_account_generation() {
        let config = BankingConfig::small();
        let mut customer_gen = crate::generators::CustomerGenerator::new(config.clone(), 12345);
        let mut customers = customer_gen.generate_all();

        let mut account_gen = AccountGenerator::new(config, 12345);
        let accounts = account_gen.generate_for_customers(&mut customers);

        assert!(!accounts.is_empty());

        // Every customer should have at least one account
        for customer in &customers {
            assert!(!customer.account_ids.is_empty());
        }
    }

    #[test]
    fn test_account_features() {
        let config = BankingConfig::default();
        let mut gen = AccountGenerator::new(config, 12345);

        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let features = gen.generate_features(&customer, true);
        assert!(features.online_banking);
    }
}
