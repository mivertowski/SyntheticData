//! Mock implementations for testing.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use datasynth_core::models::{GLAccount, JournalEntry};
use rust_decimal::Decimal;

/// Mock balance tracker for testing.
pub struct MockBalanceTracker {
    balances: Arc<RwLock<HashMap<String, Decimal>>>,
}

impl MockBalanceTracker {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get balance for an account.
    pub fn get_balance(&self, account: &str) -> Decimal {
        self.balances
            .read()
            .expect("MockBalanceTracker RwLock read poisoned")
            .get(account)
            .copied()
            .unwrap_or(Decimal::ZERO)
    }

    /// Update balance for an account.
    pub fn update_balance(&self, account: &str, amount: Decimal) {
        let mut balances = self
            .balances
            .write()
            .expect("MockBalanceTracker RwLock write poisoned");
        *balances.entry(account.to_string()).or_insert(Decimal::ZERO) += amount;
    }

    /// Apply a journal entry to the balances.
    pub fn apply_entry(&self, entry: &JournalEntry) {
        for line in &entry.lines {
            let net_amount = line.debit_amount - line.credit_amount;
            self.update_balance(&line.gl_account, net_amount);
        }
    }

    /// Get total debits across all accounts.
    pub fn total_debits(&self) -> Decimal {
        self.balances
            .read()
            .expect("MockBalanceTracker RwLock read poisoned")
            .values()
            .filter(|v| **v > Decimal::ZERO)
            .copied()
            .sum()
    }

    /// Get total credits across all accounts (as positive number).
    pub fn total_credits(&self) -> Decimal {
        self.balances
            .read()
            .expect("MockBalanceTracker RwLock read poisoned")
            .values()
            .filter(|v| **v < Decimal::ZERO)
            .map(|v| v.abs())
            .sum()
    }

    /// Clear all balances.
    pub fn clear(&self) {
        self.balances
            .write()
            .expect("MockBalanceTracker RwLock write poisoned")
            .clear();
    }
}

impl Default for MockBalanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock chart of accounts for testing.
pub struct MockChartOfAccounts {
    accounts: Vec<GLAccount>,
}

impl MockChartOfAccounts {
    pub fn new(accounts: Vec<GLAccount>) -> Self {
        Self { accounts }
    }

    /// Get an account by number.
    pub fn get_account(&self, number: &str) -> Option<&GLAccount> {
        self.accounts.iter().find(|a| a.account_number == number)
    }

    /// Get all accounts.
    pub fn all_accounts(&self) -> &[GLAccount] {
        &self.accounts
    }

    /// Check if an account exists.
    pub fn has_account(&self, number: &str) -> bool {
        self.accounts.iter().any(|a| a.account_number == number)
    }

    /// Get accounts by type.
    pub fn get_accounts_by_type(
        &self,
        account_type: datasynth_core::models::AccountType,
    ) -> Vec<&GLAccount> {
        self.accounts
            .iter()
            .filter(|a| a.account_type == account_type)
            .collect()
    }
}

impl Default for MockChartOfAccounts {
    fn default() -> Self {
        Self::new(crate::fixtures::standard_test_accounts())
    }
}

/// Mock random number generator for deterministic testing.
pub struct MockRng {
    sequence: Vec<u64>,
    index: usize,
}

impl MockRng {
    pub fn new(sequence: Vec<u64>) -> Self {
        Self { sequence, index: 0 }
    }

    /// Get the next value in the sequence.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        let value = self.sequence[self.index % self.sequence.len()];
        self.index += 1;
        value
    }

    /// Get a value in a range.
    pub fn next_in_range(&mut self, min: u64, max: u64) -> u64 {
        let value = self.next();
        min + (value % (max - min + 1))
    }

    /// Get a float in [0, 1).
    pub fn next_float(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    /// Reset the sequence.
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl Default for MockRng {
    fn default() -> Self {
        // Predictable sequence for tests
        Self::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::fixtures::balanced_journal_entry;
    use datasynth_core::models::AccountType;

    #[test]
    fn test_mock_balance_tracker() {
        let tracker = MockBalanceTracker::new();

        assert_eq!(tracker.get_balance("100000"), Decimal::ZERO);

        tracker.update_balance("100000", Decimal::new(1000, 0));
        assert_eq!(tracker.get_balance("100000"), Decimal::new(1000, 0));

        tracker.update_balance("100000", Decimal::new(-500, 0));
        assert_eq!(tracker.get_balance("100000"), Decimal::new(500, 0));
    }

    #[test]
    fn test_mock_balance_tracker_apply_entry() {
        let tracker = MockBalanceTracker::new();
        let entry = balanced_journal_entry(Decimal::new(10000, 2));

        tracker.apply_entry(&entry);

        // Debit account should have positive balance
        assert_eq!(tracker.get_balance("100000"), Decimal::new(10000, 2));
        // Credit account should have negative balance
        assert_eq!(tracker.get_balance("200000"), Decimal::new(-10000, 2));
    }

    #[test]
    fn test_mock_chart_of_accounts() {
        let coa = MockChartOfAccounts::default();

        assert!(coa.has_account("100000"));
        assert!(!coa.has_account("999999"));

        let account = coa.get_account("100000").unwrap();
        assert_eq!(account.account_type, AccountType::Asset);

        let assets = coa.get_accounts_by_type(AccountType::Asset);
        assert!(!assets.is_empty());
    }

    #[test]
    fn test_mock_rng() {
        let mut rng = MockRng::new(vec![10, 20, 30]);

        assert_eq!(rng.next(), 10);
        assert_eq!(rng.next(), 20);
        assert_eq!(rng.next(), 30);
        assert_eq!(rng.next(), 10); // Wraps around

        rng.reset();
        assert_eq!(rng.next(), 10);
    }

    #[test]
    fn test_mock_rng_range() {
        let mut rng = MockRng::new(vec![0, 5, 10, 15, 20]);

        let value = rng.next_in_range(1, 10);
        assert!((1..=10).contains(&value));
    }
}
