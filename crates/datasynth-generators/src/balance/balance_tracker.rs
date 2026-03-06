//! Running balance tracker.
//!
//! Maintains real-time account balances as journal entries are processed,
//! with continuous validation of balance sheet integrity.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::debug;

use datasynth_core::models::balance::{
    AccountBalance, AccountPeriodActivity, AccountType, BalanceSnapshot,
};
use datasynth_core::models::JournalEntry;
use datasynth_core::FrameworkAccounts;

/// Configuration for the balance tracker.
#[derive(Debug, Clone)]
pub struct BalanceTrackerConfig {
    /// Whether to validate balance sheet equation after each entry.
    pub validate_on_each_entry: bool,
    /// Whether to track balance history.
    pub track_history: bool,
    /// Tolerance for balance sheet validation (for rounding).
    pub balance_tolerance: Decimal,
    /// Whether to fail on validation errors.
    pub fail_on_validation_error: bool,
}

impl Default for BalanceTrackerConfig {
    fn default() -> Self {
        Self {
            validate_on_each_entry: true,
            track_history: true,
            balance_tolerance: dec!(0.01),
            fail_on_validation_error: false,
        }
    }
}

/// Tracks running balances for all accounts across companies.
pub struct RunningBalanceTracker {
    config: BalanceTrackerConfig,
    /// Balances by company code -> account code -> balance.
    balances: HashMap<String, HashMap<String, AccountBalance>>,
    /// Account type registry for determining debit/credit behavior.
    account_types: HashMap<String, AccountType>,
    /// Framework-aware account classification.
    framework_accounts: FrameworkAccounts,
    /// Balance history by company code.
    history: HashMap<String, Vec<BalanceHistoryEntry>>,
    /// Validation errors encountered.
    validation_errors: Vec<ValidationError>,
    /// Statistics.
    stats: TrackerStatistics,
    /// Default currency for new account balances and snapshots.
    currency: String,
}

/// Entry in balance history.
#[derive(Debug, Clone)]
pub struct BalanceHistoryEntry {
    pub date: NaiveDate,
    pub entry_id: String,
    pub account_code: String,
    pub previous_balance: Decimal,
    pub change: Decimal,
    pub new_balance: Decimal,
}

/// Validation error details.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub date: NaiveDate,
    pub company_code: String,
    pub entry_id: Option<String>,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub details: HashMap<String, Decimal>,
}

/// Types of validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorType {
    /// Entry debits don't equal credits.
    UnbalancedEntry,
    /// Balance sheet equation violated.
    BalanceSheetImbalance,
    /// Account has negative balance where not allowed.
    NegativeBalance,
    /// Unknown account code.
    UnknownAccount,
    /// Entry applied out of chronological order.
    OutOfOrder,
}

/// Statistics about tracked entries.
#[derive(Debug, Clone, Default)]
pub struct TrackerStatistics {
    pub entries_processed: u64,
    pub lines_processed: u64,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub companies_tracked: usize,
    pub accounts_tracked: usize,
    pub validation_errors: usize,
}

impl RunningBalanceTracker {
    /// Creates a new balance tracker with the specified currency and accounting framework.
    pub fn new_with_currency_and_framework(
        config: BalanceTrackerConfig,
        currency: String,
        framework: &str,
    ) -> Self {
        Self {
            config,
            balances: HashMap::new(),
            account_types: HashMap::new(),
            framework_accounts: FrameworkAccounts::for_framework(framework),
            history: HashMap::new(),
            validation_errors: Vec::new(),
            stats: TrackerStatistics::default(),
            currency,
        }
    }

    /// Creates a new balance tracker with the specified currency (defaults to US GAAP).
    pub fn new_with_currency(config: BalanceTrackerConfig, currency: String) -> Self {
        Self::new_with_currency_and_framework(config, currency, "us_gaap")
    }

    /// Creates a new balance tracker (defaults to USD and US GAAP).
    pub fn new(config: BalanceTrackerConfig) -> Self {
        Self::new_with_currency(config, "USD".to_string())
    }

    /// Creates a new balance tracker for a specific accounting framework (defaults to USD).
    pub fn new_with_framework(config: BalanceTrackerConfig, framework: &str) -> Self {
        Self::new_with_currency_and_framework(config, "USD".to_string(), framework)
    }

    /// Creates a tracker with default configuration (US GAAP).
    pub fn with_defaults() -> Self {
        Self::new(BalanceTrackerConfig::default())
    }

    /// Registers an account type for balance tracking.
    pub fn register_account_type(&mut self, account_code: &str, account_type: AccountType) {
        self.account_types
            .insert(account_code.to_string(), account_type);
    }

    /// Registers multiple account types.
    pub fn register_account_types(&mut self, types: &[(String, AccountType)]) {
        for (code, account_type) in types {
            self.account_types.insert(code.clone(), *account_type);
        }
    }

    /// Registers account types from a chart of accounts prefix pattern.
    pub fn register_from_chart_prefixes(&mut self, prefixes: &[(&str, AccountType)]) {
        for (prefix, account_type) in prefixes {
            self.account_types.insert(prefix.to_string(), *account_type);
        }
    }

    /// Initializes balances from opening balance snapshot.
    pub fn initialize_from_snapshot(&mut self, snapshot: &BalanceSnapshot) {
        let company_balances = self
            .balances
            .entry(snapshot.company_code.clone())
            .or_default();

        for (account_code, balance) in &snapshot.balances {
            company_balances.insert(account_code.clone(), balance.clone());
        }

        self.stats.companies_tracked = self.balances.len();
        self.stats.accounts_tracked = self
            .balances
            .values()
            .map(std::collections::HashMap::len)
            .sum();
    }

    /// Applies a journal entry to the running balances.
    pub fn apply_entry(&mut self, entry: &JournalEntry) -> Result<(), ValidationError> {
        // Validate entry is balanced first
        if !entry.is_balanced() {
            let error = ValidationError {
                date: entry.posting_date(),
                company_code: entry.company_code().to_string(),
                entry_id: Some(entry.document_number().clone()),
                error_type: ValidationErrorType::UnbalancedEntry,
                message: format!(
                    "Entry {} is unbalanced: debits={}, credits={}",
                    entry.document_number(),
                    entry.total_debit(),
                    entry.total_credit()
                ),
                details: {
                    let mut d = HashMap::new();
                    d.insert("total_debit".to_string(), entry.total_debit());
                    d.insert("total_credit".to_string(), entry.total_credit());
                    d
                },
            };

            if self.config.fail_on_validation_error {
                return Err(error);
            }
            self.validation_errors.push(error);
        }

        // Extract data we need before mutably borrowing balances
        let company_code = entry.company_code().to_string();
        let document_number = entry.document_number().clone();
        let posting_date = entry.posting_date();
        let track_history = self.config.track_history;

        // Pre-compute account types for all lines
        let line_data: Vec<_> = entry
            .lines
            .iter()
            .map(|line| {
                let account_type = self.determine_account_type(&line.account_code);
                (line.clone(), account_type)
            })
            .collect();

        // Get or create company balances
        let company_balances = self.balances.entry(company_code.clone()).or_default();

        // History entries to add
        let mut history_entries = Vec::new();

        // Apply each line
        for (line, account_type) in &line_data {
            // Get or create account balance
            let balance = company_balances
                .entry(line.account_code.clone())
                .or_insert_with(|| {
                    AccountBalance::new(
                        company_code.clone(),
                        line.account_code.clone(),
                        *account_type,
                        self.currency.clone(),
                        posting_date.year(),
                        posting_date.month(),
                    )
                });

            let previous_balance = balance.closing_balance;

            // Apply debit or credit
            if line.debit_amount > Decimal::ZERO {
                balance.apply_debit(line.debit_amount);
            }
            if line.credit_amount > Decimal::ZERO {
                balance.apply_credit(line.credit_amount);
            }

            let new_balance = balance.closing_balance;

            // Record history if configured
            if track_history {
                let change = line.debit_amount - line.credit_amount;
                history_entries.push(BalanceHistoryEntry {
                    date: posting_date,
                    entry_id: document_number.clone(),
                    account_code: line.account_code.clone(),
                    previous_balance,
                    change,
                    new_balance,
                });
            }
        }

        // Add history entries after releasing the balances borrow
        if !history_entries.is_empty() {
            let hist = self.history.entry(company_code.clone()).or_default();
            hist.extend(history_entries);
        }

        // Update statistics
        self.stats.entries_processed += 1;
        self.stats.lines_processed += entry.lines.len() as u64;
        self.stats.total_debits += entry.total_debit();
        self.stats.total_credits += entry.total_credit();
        self.stats.companies_tracked = self.balances.len();
        self.stats.accounts_tracked = self
            .balances
            .values()
            .map(std::collections::HashMap::len)
            .sum();

        // Validate balance sheet if configured
        if self.config.validate_on_each_entry {
            self.validate_balance_sheet(
                entry.company_code(),
                entry.posting_date(),
                Some(&entry.document_number()),
            )?;
        }

        Ok(())
    }

    /// Applies a batch of entries.
    pub fn apply_entries(&mut self, entries: &[JournalEntry]) -> Vec<ValidationError> {
        debug!(
            entry_count = entries.len(),
            companies_tracked = self.stats.companies_tracked,
            accounts_tracked = self.stats.accounts_tracked,
            "Applying entries to balance tracker"
        );

        let mut errors = Vec::new();

        for entry in entries {
            if let Err(error) = self.apply_entry(entry) {
                errors.push(error);
            }
        }

        errors
    }

    /// Determines account type from code prefix.
    ///
    /// Checks explicitly registered types first, then falls back to the
    /// framework-aware classifier from [`FrameworkAccounts`].
    fn determine_account_type(&self, account_code: &str) -> AccountType {
        // Check registered types first (exact match or prefix)
        for (registered_code, account_type) in &self.account_types {
            if account_code.starts_with(registered_code) {
                return *account_type;
            }
        }

        // Use framework-aware classification
        self.framework_accounts.classify_account_type(account_code)
    }

    /// Validates the balance sheet equation for a company.
    pub fn validate_balance_sheet(
        &mut self,
        company_code: &str,
        date: NaiveDate,
        entry_id: Option<&str>,
    ) -> Result<(), ValidationError> {
        let Some(company_balances) = self.balances.get(company_code) else {
            return Ok(()); // No balances to validate
        };

        let mut total_assets = Decimal::ZERO;
        let mut total_liabilities = Decimal::ZERO;
        let mut total_equity = Decimal::ZERO;
        let mut total_revenue = Decimal::ZERO;
        let mut total_expenses = Decimal::ZERO;

        for (account_code, balance) in company_balances {
            let account_type = self.determine_account_type(account_code);
            match account_type {
                AccountType::Asset => total_assets += balance.closing_balance,
                AccountType::ContraAsset => total_assets -= balance.closing_balance.abs(),
                AccountType::Liability => total_liabilities += balance.closing_balance.abs(),
                AccountType::ContraLiability => total_liabilities -= balance.closing_balance.abs(),
                AccountType::Equity => total_equity += balance.closing_balance.abs(),
                AccountType::ContraEquity => total_equity -= balance.closing_balance.abs(),
                AccountType::Revenue => total_revenue += balance.closing_balance.abs(),
                AccountType::Expense => total_expenses += balance.closing_balance.abs(),
            }
        }

        // Net income = Revenue - Expenses
        let net_income = total_revenue - total_expenses;

        // Balance sheet equation: Assets = Liabilities + Equity + Net Income
        let left_side = total_assets;
        let right_side = total_liabilities + total_equity + net_income;
        let difference = (left_side - right_side).abs();

        if difference > self.config.balance_tolerance {
            let error = ValidationError {
                date,
                company_code: company_code.to_string(),
                entry_id: entry_id.map(String::from),
                error_type: ValidationErrorType::BalanceSheetImbalance,
                message: format!(
                    "Balance sheet imbalance: Assets ({left_side}) != L + E + NI ({right_side}), diff = {difference}"
                ),
                details: {
                    let mut d = HashMap::new();
                    d.insert("total_assets".to_string(), total_assets);
                    d.insert("total_liabilities".to_string(), total_liabilities);
                    d.insert("total_equity".to_string(), total_equity);
                    d.insert("net_income".to_string(), net_income);
                    d.insert("difference".to_string(), difference);
                    d
                },
            };

            self.stats.validation_errors += 1;

            if self.config.fail_on_validation_error {
                return Err(error);
            }
            self.validation_errors.push(error);
        }

        Ok(())
    }

    /// Gets the current snapshot for a company.
    pub fn get_snapshot(
        &self,
        company_code: &str,
        as_of_date: NaiveDate,
    ) -> Option<BalanceSnapshot> {
        use chrono::Datelike;
        let currency = self.currency.clone();
        self.balances.get(company_code).map(|balances| {
            let mut snapshot = BalanceSnapshot::new(
                format!("SNAP-{company_code}-{as_of_date}"),
                company_code.to_string(),
                as_of_date,
                as_of_date.year(),
                as_of_date.month(),
                currency,
            );
            for (account, balance) in balances {
                snapshot.balances.insert(account.clone(), balance.clone());
            }
            snapshot.recalculate_totals();
            snapshot
        })
    }

    /// Gets snapshots for all companies.
    pub fn get_all_snapshots(&self, as_of_date: NaiveDate) -> Vec<BalanceSnapshot> {
        use chrono::Datelike;
        self.balances
            .iter()
            .map(|(company_code, balances)| {
                let mut snapshot = BalanceSnapshot::new(
                    format!("SNAP-{company_code}-{as_of_date}"),
                    company_code.clone(),
                    as_of_date,
                    as_of_date.year(),
                    as_of_date.month(),
                    self.currency.clone(),
                );
                for (account, balance) in balances {
                    snapshot.balances.insert(account.clone(), balance.clone());
                }
                snapshot.recalculate_totals();
                snapshot
            })
            .collect()
    }

    /// Gets balance changes for a period.
    pub fn get_balance_changes(
        &self,
        company_code: &str,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Vec<AccountPeriodActivity> {
        let Some(history) = self.history.get(company_code) else {
            return Vec::new();
        };

        let mut changes_by_account: HashMap<String, AccountPeriodActivity> = HashMap::new();

        for entry in history
            .iter()
            .filter(|e| e.date >= from_date && e.date <= to_date)
        {
            let change = changes_by_account
                .entry(entry.account_code.clone())
                .or_insert_with(|| AccountPeriodActivity {
                    account_code: entry.account_code.clone(),
                    period_start: from_date,
                    period_end: to_date,
                    opening_balance: Decimal::ZERO,
                    closing_balance: Decimal::ZERO,
                    total_debits: Decimal::ZERO,
                    total_credits: Decimal::ZERO,
                    net_change: Decimal::ZERO,
                    transaction_count: 0,
                });

            if entry.change > Decimal::ZERO {
                change.total_debits += entry.change;
            } else {
                change.total_credits += entry.change.abs();
            }
            change.net_change += entry.change;
            change.transaction_count += 1;
        }

        // Update opening/closing balances
        if let Some(company_balances) = self.balances.get(company_code) {
            for change in changes_by_account.values_mut() {
                if let Some(balance) = company_balances.get(&change.account_code) {
                    change.closing_balance = balance.closing_balance;
                    change.opening_balance = change.closing_balance - change.net_change;
                }
            }
        }

        changes_by_account.into_values().collect()
    }

    /// Gets balance for a specific account.
    pub fn get_account_balance(
        &self,
        company_code: &str,
        account_code: &str,
    ) -> Option<&AccountBalance> {
        self.balances
            .get(company_code)
            .and_then(|b| b.get(account_code))
    }

    /// Gets all validation errors.
    pub fn get_validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// Clears validation errors.
    pub fn clear_validation_errors(&mut self) {
        self.validation_errors.clear();
        self.stats.validation_errors = 0;
    }

    /// Gets tracker statistics.
    pub fn get_statistics(&self) -> &TrackerStatistics {
        &self.stats
    }

    /// Rolls forward balances to a new period.
    pub fn roll_forward(&mut self, _new_period_start: NaiveDate) {
        for company_balances in self.balances.values_mut() {
            for balance in company_balances.values_mut() {
                balance.roll_forward();
            }
        }
    }

    /// Exports balances to a simple format.
    pub fn export_balances(&self, company_code: &str) -> Vec<(String, Decimal)> {
        self.balances
            .get(company_code)
            .map(|balances| {
                balances
                    .iter()
                    .map(|(code, balance)| (code.clone(), balance.closing_balance))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{JournalEntry, JournalEntryLine};

    fn create_test_entry(
        company: &str,
        account1: &str,
        account2: &str,
        amount: Decimal,
    ) -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            "TEST001".to_string(),
            company.to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Test entry".to_string(),
        );

        entry.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: account1.to_string(),
            account_code: account1.to_string(),
            debit_amount: amount,
            ..Default::default()
        });

        entry.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: account2.to_string(),
            account_code: account2.to_string(),
            credit_amount: amount,
            ..Default::default()
        });

        entry
    }

    #[test]
    fn test_apply_balanced_entry() {
        let mut tracker = RunningBalanceTracker::with_defaults();
        tracker.register_account_type("1100", AccountType::Asset);
        tracker.register_account_type("4000", AccountType::Revenue);

        let entry = create_test_entry("1000", "1100", "4000", dec!(1000));
        let result = tracker.apply_entry(&entry);

        assert!(result.is_ok());
        assert_eq!(tracker.stats.entries_processed, 1);
        assert_eq!(tracker.stats.lines_processed, 2);
    }

    #[test]
    fn test_balance_accumulation() {
        let mut tracker = RunningBalanceTracker::with_defaults();
        tracker.config.validate_on_each_entry = false;

        let entry1 = create_test_entry("1000", "1100", "4000", dec!(1000));
        let entry2 = create_test_entry("1000", "1100", "4000", dec!(500));

        tracker.apply_entry(&entry1).unwrap();
        tracker.apply_entry(&entry2).unwrap();

        let balance = tracker.get_account_balance("1000", "1100").unwrap();
        assert_eq!(balance.closing_balance, dec!(1500));
    }

    #[test]
    fn test_get_snapshot() {
        let mut tracker = RunningBalanceTracker::with_defaults();
        tracker.config.validate_on_each_entry = false;

        let entry = create_test_entry("1000", "1100", "2000", dec!(1000));
        tracker.apply_entry(&entry).unwrap();

        let snapshot = tracker
            .get_snapshot("1000", NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
            .unwrap();

        assert_eq!(snapshot.balances.len(), 2);
    }

    #[test]
    fn test_determine_account_type_from_prefix() {
        let tracker = RunningBalanceTracker::with_defaults();

        assert_eq!(tracker.determine_account_type("1000"), AccountType::Asset);
        assert_eq!(
            tracker.determine_account_type("2000"),
            AccountType::Liability
        );
        assert_eq!(tracker.determine_account_type("3000"), AccountType::Equity);
        assert_eq!(tracker.determine_account_type("4000"), AccountType::Revenue);
        assert_eq!(tracker.determine_account_type("5000"), AccountType::Expense);
    }

    #[test]
    fn test_determine_account_type_french_gaap() {
        let tracker = RunningBalanceTracker::new_with_framework(
            BalanceTrackerConfig::default(),
            "french_gaap",
        );

        // PCG class 2 = Fixed Assets (Asset)
        assert_eq!(tracker.determine_account_type("210000"), AccountType::Asset);
        // PCG class 1 subclass 0-4 = Equity
        assert_eq!(
            tracker.determine_account_type("101000"),
            AccountType::Equity
        );
        // PCG class 4 subclass 0 = Suppliers (Liability)
        assert_eq!(
            tracker.determine_account_type("401000"),
            AccountType::Liability
        );
        // PCG class 6 = Expenses
        assert_eq!(
            tracker.determine_account_type("603000"),
            AccountType::Expense
        );
        // PCG class 7 = Revenue
        assert_eq!(
            tracker.determine_account_type("701000"),
            AccountType::Revenue
        );
    }

    #[test]
    fn test_determine_account_type_german_gaap() {
        let tracker = RunningBalanceTracker::new_with_framework(
            BalanceTrackerConfig::default(),
            "german_gaap",
        );

        // SKR04 class 0 = Fixed Assets (Asset)
        assert_eq!(tracker.determine_account_type("0200"), AccountType::Asset);
        // SKR04 class 2 = Equity
        assert_eq!(tracker.determine_account_type("2000"), AccountType::Equity);
        // SKR04 class 3 = Liabilities
        assert_eq!(
            tracker.determine_account_type("3300"),
            AccountType::Liability
        );
        // SKR04 class 4 = Revenue
        assert_eq!(tracker.determine_account_type("4000"), AccountType::Revenue);
        // SKR04 class 5 = COGS (Expense)
        assert_eq!(tracker.determine_account_type("5000"), AccountType::Expense);
    }
}
