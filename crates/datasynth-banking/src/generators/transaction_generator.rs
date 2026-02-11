//! Transaction generator for banking data.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use datasynth_core::models::banking::{
    Direction, MerchantCategoryCode, TransactionCategory, TransactionChannel,
};
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::config::BankingConfig;
use crate::models::{
    BankAccount, BankTransaction, BankingCustomer, CounterpartyPool, CounterpartyRef,
    PersonaVariant,
};

/// Generator for banking transactions.
pub struct TransactionGenerator {
    config: BankingConfig,
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    counterparty_pool: CounterpartyPool,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl TransactionGenerator {
    /// Create a new transaction generator.
    pub fn new(config: BankingConfig, seed: u64) -> Self {
        let start_date = NaiveDate::parse_from_str(&config.population.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"));
        let end_date = start_date + chrono::Months::new(config.population.period_months);

        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(2000)),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::JournalEntry,
            ),
            counterparty_pool: CounterpartyPool::standard(),
            start_date,
            end_date,
        }
    }

    /// Set custom counterparty pool.
    pub fn with_counterparty_pool(mut self, pool: CounterpartyPool) -> Self {
        self.counterparty_pool = pool;
        self
    }

    /// Generate transactions for all accounts.
    pub fn generate_all(
        &mut self,
        customers: &[BankingCustomer],
        accounts: &mut [BankAccount],
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Create customer lookup
        let customer_map: std::collections::HashMap<Uuid, &BankingCustomer> =
            customers.iter().map(|c| (c.customer_id, c)).collect();

        for account in accounts.iter_mut() {
            if let Some(customer) = customer_map.get(&account.primary_owner_id) {
                let account_txns = self.generate_account_transactions(customer, account);
                transactions.extend(account_txns);
            }
        }

        // Sort by timestamp
        transactions.sort_by_key(|t| t.timestamp_initiated);

        transactions
    }

    /// Generate transactions for a single account.
    pub fn generate_account_transactions(
        &mut self,
        customer: &BankingCustomer,
        account: &mut BankAccount,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        let mut current_date = self.start_date.max(account.opening_date);
        let mut balance = account.current_balance;

        while current_date <= self.end_date {
            // Generate transactions for this day
            let daily_txns =
                self.generate_daily_transactions(customer, account, current_date, &mut balance);
            transactions.extend(daily_txns);

            current_date += Duration::days(1);
        }

        // Update account balance
        account.current_balance = balance;
        account.available_balance = balance;

        transactions
    }

    /// Generate transactions for a single day.
    fn generate_daily_transactions(
        &mut self,
        customer: &BankingCustomer,
        account: &BankAccount,
        date: NaiveDate,
        balance: &mut Decimal,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Determine expected transaction count for this day
        let expected_count = self.calculate_daily_transaction_count(customer, date);

        // Generate income transactions (if applicable)
        if self.should_generate_income(customer, date) {
            if let Some(txn) = self.generate_income_transaction(customer, account, date, balance) {
                transactions.push(txn);
            }
        }

        // Generate recurring payments (if applicable)
        if self.should_generate_recurring(customer, date) {
            transactions
                .extend(self.generate_recurring_transactions(customer, account, date, balance));
        }

        // Generate discretionary transactions
        let discretionary_count = expected_count.saturating_sub(transactions.len() as u32);
        for _ in 0..discretionary_count {
            if let Some(txn) =
                self.generate_discretionary_transaction(customer, account, date, balance)
            {
                transactions.push(txn);
            }
        }

        transactions
    }

    /// Calculate expected daily transaction count.
    fn calculate_daily_transaction_count(
        &mut self,
        customer: &BankingCustomer,
        date: NaiveDate,
    ) -> u32 {
        let (freq_min, freq_max) = match &customer.persona {
            Some(PersonaVariant::Retail(p)) => p.transaction_frequency_range(),
            Some(PersonaVariant::Business(_)) => (50, 200),
            _ => (10, 50),
        };

        let avg_daily = (freq_min + freq_max) as f64 / 2.0 / 30.0;

        // Weekend adjustment
        let day_of_week = date.weekday();
        let multiplier = match day_of_week {
            chrono::Weekday::Sat | chrono::Weekday::Sun => 0.5,
            _ => 1.0,
        };

        let expected = avg_daily * multiplier + self.rng.gen_range(-1.0..1.0);
        expected.max(0.0) as u32
    }

    /// Check if income should be generated today.
    fn should_generate_income(&mut self, customer: &BankingCustomer, date: NaiveDate) -> bool {
        match &customer.persona {
            Some(PersonaVariant::Retail(p)) => {
                use datasynth_core::models::banking::RetailPersona;
                match p {
                    RetailPersona::Retiree => date.day() == 1 || date.day() == 15, // Pension
                    RetailPersona::GigWorker => self.rng.gen::<f64>() < 0.15, // Variable income
                    _ => {
                        (date.day() == 1 || date.day() == 15)
                            && date.weekday().num_days_from_monday() < 5
                    }
                }
            }
            Some(PersonaVariant::Business(_)) => self.rng.gen::<f64>() < 0.3, // Business income
            _ => date.day() == 1,
        }
    }

    /// Check if recurring payments should be generated today.
    fn should_generate_recurring(&mut self, _customer: &BankingCustomer, date: NaiveDate) -> bool {
        // Most recurring payments on 1st, 15th, or end of month
        date.day() == 1 || date.day() == 15 || date.day() >= 28
    }

    /// Generate an income transaction.
    fn generate_income_transaction(
        &mut self,
        customer: &BankingCustomer,
        account: &BankAccount,
        date: NaiveDate,
        balance: &mut Decimal,
    ) -> Option<BankTransaction> {
        let (amount, category, counterparty) = match &customer.persona {
            Some(PersonaVariant::Retail(p)) => {
                let (min, max) = p.income_range();
                let amount = Decimal::from_f64_retain(self.rng.gen_range(min as f64..max as f64))
                    .unwrap_or(Decimal::ZERO);

                let category = match p {
                    datasynth_core::models::banking::RetailPersona::Retiree => {
                        TransactionCategory::Pension
                    }
                    datasynth_core::models::banking::RetailPersona::GigWorker => {
                        TransactionCategory::FreelanceIncome
                    }
                    _ => TransactionCategory::Salary,
                };

                let employer = self.counterparty_pool.employers.choose(&mut self.rng);
                let counterparty = employer
                    .map(|e| CounterpartyRef::employer(e.employer_id, &e.name))
                    .unwrap_or_else(|| CounterpartyRef::unknown("Employer"));

                (amount, category, counterparty)
            }
            _ => return None,
        };

        let timestamp = self.random_timestamp(date);
        *balance += amount;

        let txn = BankTransaction::new(
            self.uuid_factory.next(),
            account.account_id,
            amount,
            &account.currency,
            Direction::Inbound,
            TransactionChannel::Ach,
            category,
            counterparty,
            "Direct deposit",
            timestamp,
        )
        .with_balance(*balance - amount, *balance);

        Some(txn)
    }

    /// Generate recurring payment transactions.
    fn generate_recurring_transactions(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        date: NaiveDate,
        balance: &mut Decimal,
    ) -> Vec<BankTransaction> {
        let mut transactions = Vec::new();

        // Select random recurring payments for today
        let recurring_types = [
            (TransactionCategory::Housing, 1000.0, 3000.0, 0.3),
            (TransactionCategory::Utilities, 50.0, 200.0, 0.2),
            (TransactionCategory::Insurance, 100.0, 500.0, 0.15),
            (TransactionCategory::Subscription, 10.0, 100.0, 0.3),
        ];

        for (category, min, max, probability) in recurring_types {
            if self.rng.gen::<f64>() < probability {
                let amount =
                    Decimal::from_f64_retain(self.rng.gen_range(min..max)).unwrap_or(Decimal::ZERO);

                // Skip if insufficient balance
                if *balance < amount {
                    continue;
                }

                *balance -= amount;

                let utility = self.counterparty_pool.utilities.choose(&mut self.rng);
                let counterparty = utility
                    .map(|u| CounterpartyRef::unknown(&u.name))
                    .unwrap_or_else(|| CounterpartyRef::unknown("Service Provider"));

                let txn = BankTransaction::new(
                    self.uuid_factory.next(),
                    account.account_id,
                    amount,
                    &account.currency,
                    Direction::Outbound,
                    TransactionChannel::Ach,
                    category,
                    counterparty,
                    &format!("{:?} payment", category),
                    self.random_timestamp(date),
                )
                .with_balance(*balance + amount, *balance);

                transactions.push(txn);
            }
        }

        transactions
    }

    /// Generate a discretionary transaction.
    fn generate_discretionary_transaction(
        &mut self,
        _customer: &BankingCustomer,
        account: &BankAccount,
        date: NaiveDate,
        balance: &mut Decimal,
    ) -> Option<BankTransaction> {
        // Determine channel
        let channel = self.select_channel();

        // Determine category
        let (category, mcc) = self.select_category(channel);

        // Determine amount
        let amount = self.generate_transaction_amount(category);

        // Determine direction (mostly outbound for discretionary)
        let direction = if self.rng.gen::<f64>() < 0.1 {
            Direction::Inbound
        } else {
            Direction::Outbound
        };

        // Check balance for outbound
        if direction == Direction::Outbound && *balance < amount {
            return None;
        }

        // Select counterparty
        let counterparty = self.select_counterparty(category);

        // Update balance
        match direction {
            Direction::Inbound => *balance += amount,
            Direction::Outbound => *balance -= amount,
        }

        let balance_before = match direction {
            Direction::Inbound => *balance - amount,
            Direction::Outbound => *balance + amount,
        };

        let mut txn = BankTransaction::new(
            self.uuid_factory.next(),
            account.account_id,
            amount,
            &account.currency,
            direction,
            channel,
            category,
            counterparty,
            &self.generate_reference(category),
            self.random_timestamp(date),
        )
        .with_balance(balance_before, *balance);

        if let Some(mcc) = mcc {
            txn = txn.with_mcc(mcc);
        }

        Some(txn)
    }

    /// Select transaction channel.
    fn select_channel(&mut self) -> TransactionChannel {
        let card_ratio = self.config.products.card_vs_transfer;
        let roll: f64 = self.rng.gen();

        if roll < card_ratio * 0.6 {
            TransactionChannel::CardPresent
        } else if roll < card_ratio {
            TransactionChannel::CardNotPresent
        } else if roll < card_ratio + (1.0 - card_ratio) * 0.3 {
            TransactionChannel::Ach
        } else if roll < card_ratio + (1.0 - card_ratio) * 0.5 {
            TransactionChannel::Online
        } else if roll < card_ratio + (1.0 - card_ratio) * 0.7 {
            TransactionChannel::Mobile
        } else if roll < card_ratio + (1.0 - card_ratio) * 0.85 {
            TransactionChannel::Atm
        } else {
            TransactionChannel::PeerToPeer
        }
    }

    /// Select transaction category.
    fn select_category(
        &mut self,
        channel: TransactionChannel,
    ) -> (TransactionCategory, Option<MerchantCategoryCode>) {
        let categories: Vec<(TransactionCategory, Option<MerchantCategoryCode>, f64)> =
            match channel {
                TransactionChannel::CardPresent | TransactionChannel::CardNotPresent => vec![
                    (
                        TransactionCategory::Groceries,
                        Some(MerchantCategoryCode::GROCERY_STORES),
                        0.25,
                    ),
                    (
                        TransactionCategory::Dining,
                        Some(MerchantCategoryCode::RESTAURANTS),
                        0.20,
                    ),
                    (
                        TransactionCategory::Shopping,
                        Some(MerchantCategoryCode::DEPARTMENT_STORES),
                        0.20,
                    ),
                    (
                        TransactionCategory::Transportation,
                        Some(MerchantCategoryCode::GAS_STATIONS),
                        0.15,
                    ),
                    (TransactionCategory::Entertainment, None, 0.10),
                    (
                        TransactionCategory::Healthcare,
                        Some(MerchantCategoryCode::MEDICAL),
                        0.05,
                    ),
                    (TransactionCategory::Other, None, 0.05),
                ],
                TransactionChannel::Atm => vec![(TransactionCategory::AtmWithdrawal, None, 1.0)],
                TransactionChannel::PeerToPeer => {
                    vec![(TransactionCategory::P2PPayment, None, 1.0)]
                }
                _ => vec![
                    (TransactionCategory::TransferOut, None, 0.5),
                    (TransactionCategory::Other, None, 0.5),
                ],
            };

        let total: f64 = categories.iter().map(|(_, _, w)| w).sum();
        let roll: f64 = self.rng.gen::<f64>() * total;
        let mut cumulative = 0.0;

        for (cat, mcc, weight) in categories {
            cumulative += weight;
            if roll < cumulative {
                return (cat, mcc);
            }
        }

        (TransactionCategory::Other, None)
    }

    /// Generate transaction amount.
    fn generate_transaction_amount(&mut self, category: TransactionCategory) -> Decimal {
        let (min, max) = match category {
            TransactionCategory::Groceries => (20.0, 200.0),
            TransactionCategory::Dining => (10.0, 150.0),
            TransactionCategory::Shopping => (15.0, 500.0),
            TransactionCategory::Transportation => (20.0, 100.0),
            TransactionCategory::Entertainment => (10.0, 200.0),
            TransactionCategory::Healthcare => (20.0, 500.0),
            TransactionCategory::AtmWithdrawal => (20.0, 500.0),
            TransactionCategory::P2PPayment => (5.0, 200.0),
            _ => (10.0, 200.0),
        };

        Decimal::from_f64_retain(self.rng.gen_range(min..max)).unwrap_or(Decimal::ZERO)
    }

    /// Select counterparty.
    fn select_counterparty(&mut self, category: TransactionCategory) -> CounterpartyRef {
        match category {
            TransactionCategory::AtmWithdrawal => CounterpartyRef::atm("Branch ATM"),
            TransactionCategory::P2PPayment => CounterpartyRef::peer("Friend", None),
            _ => self
                .counterparty_pool
                .merchants
                .choose(&mut self.rng)
                .map(|m| CounterpartyRef::merchant(m.merchant_id, &m.name))
                .unwrap_or_else(|| CounterpartyRef::unknown("Merchant")),
        }
    }

    /// Generate transaction reference.
    fn generate_reference(&self, category: TransactionCategory) -> String {
        match category {
            TransactionCategory::Groceries => "Grocery purchase",
            TransactionCategory::Dining => "Restaurant",
            TransactionCategory::Shopping => "Retail purchase",
            TransactionCategory::Transportation => "Fuel purchase",
            TransactionCategory::Entertainment => "Entertainment",
            TransactionCategory::Healthcare => "Medical expense",
            TransactionCategory::AtmWithdrawal => "ATM withdrawal",
            TransactionCategory::P2PPayment => "P2P transfer",
            _ => "Transaction",
        }
        .to_string()
    }

    /// Generate random timestamp for a date.
    fn random_timestamp(&mut self, date: NaiveDate) -> DateTime<Utc> {
        let hour: u32 = self.rng.gen_range(8..22);
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

    #[test]
    fn test_transaction_generation() {
        let config = BankingConfig::small();
        let mut customer_gen = crate::generators::CustomerGenerator::new(config.clone(), 12345);
        let mut customers = customer_gen.generate_all();

        let mut account_gen = crate::generators::AccountGenerator::new(config.clone(), 12345);
        let mut accounts = account_gen.generate_for_customers(&mut customers);

        let mut txn_gen = TransactionGenerator::new(config, 12345);
        let transactions = txn_gen.generate_all(&customers, &mut accounts);

        assert!(!transactions.is_empty());
    }
}
