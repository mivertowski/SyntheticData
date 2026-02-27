//! Trial balance generator.
//!
//! Generates trial balances at period end from running balance snapshots,
//! with support for:
//! - Unadjusted, adjusted, and post-closing trial balances
//! - Category summaries and subtotals
//! - Comparative trial balances across periods
//! - Consolidated trial balances across companies

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::debug;

use datasynth_core::models::balance::{
    AccountBalance, AccountCategory, AccountType, BalanceSnapshot, CategorySummary,
    ComparativeTrialBalance, TrialBalance, TrialBalanceLine, TrialBalanceStatus, TrialBalanceType,
};
use datasynth_core::models::ChartOfAccounts;
use datasynth_core::FrameworkAccounts;

use super::RunningBalanceTracker;

/// Configuration for trial balance generation.
#[derive(Debug, Clone)]
pub struct TrialBalanceConfig {
    /// Include zero balance accounts.
    pub include_zero_balances: bool,
    /// Group accounts by category.
    pub group_by_category: bool,
    /// Generate category subtotals.
    pub generate_subtotals: bool,
    /// Sort accounts by code.
    pub sort_by_account_code: bool,
    /// Trial balance type to generate.
    pub trial_balance_type: TrialBalanceType,
}

impl Default for TrialBalanceConfig {
    fn default() -> Self {
        Self {
            include_zero_balances: false,
            group_by_category: true,
            generate_subtotals: true,
            sort_by_account_code: true,
            trial_balance_type: TrialBalanceType::Unadjusted,
        }
    }
}

/// Generator for trial balance reports.
pub struct TrialBalanceGenerator {
    config: TrialBalanceConfig,
    /// Account category mappings.
    category_mappings: HashMap<String, AccountCategory>,
    /// Account descriptions.
    account_descriptions: HashMap<String, String>,
    /// Framework-aware account classification.
    framework_accounts: FrameworkAccounts,
}

impl TrialBalanceGenerator {
    /// Creates a new trial balance generator for a specific accounting framework.
    pub fn new_with_framework(config: TrialBalanceConfig, framework: &str) -> Self {
        Self {
            config,
            category_mappings: HashMap::new(),
            account_descriptions: HashMap::new(),
            framework_accounts: FrameworkAccounts::for_framework(framework),
        }
    }

    /// Creates a new trial balance generator (defaults to US GAAP).
    pub fn new(config: TrialBalanceConfig) -> Self {
        Self::new_with_framework(config, "us_gaap")
    }

    /// Creates a generator with default configuration (US GAAP).
    pub fn with_defaults() -> Self {
        Self::new(TrialBalanceConfig::default())
    }

    /// Registers category mappings from chart of accounts.
    pub fn register_from_chart(&mut self, chart: &ChartOfAccounts) {
        for account in &chart.accounts {
            self.account_descriptions.insert(
                account.account_code().to_string(),
                account.description().to_string(),
            );

            // Determine category from account code prefix
            let category = self.determine_category(account.account_code());
            self.category_mappings
                .insert(account.account_code().to_string(), category);
        }
    }

    /// Registers a custom category mapping.
    pub fn register_category(&mut self, account_code: &str, category: AccountCategory) {
        self.category_mappings
            .insert(account_code.to_string(), category);
    }

    /// Generates a trial balance from a balance snapshot.
    pub fn generate_from_snapshot(
        &self,
        snapshot: &BalanceSnapshot,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> TrialBalance {
        debug!(
            company_code = %snapshot.company_code,
            fiscal_year,
            fiscal_period,
            balance_count = snapshot.balances.len(),
            "Generating trial balance from snapshot"
        );

        let mut lines = Vec::new();
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;

        // Convert balances to trial balance lines
        for (account_code, balance) in &snapshot.balances {
            if !self.config.include_zero_balances && balance.closing_balance == Decimal::ZERO {
                continue;
            }

            let (debit, credit) = self.split_balance(balance);
            total_debits += debit;
            total_credits += credit;

            let category = self.determine_category(account_code);
            let description = self
                .account_descriptions
                .get(account_code)
                .cloned()
                .unwrap_or_else(|| format!("Account {}", account_code));

            lines.push(TrialBalanceLine {
                account_code: account_code.clone(),
                account_description: description,
                category,
                account_type: balance.account_type,
                debit_balance: debit,
                credit_balance: credit,
                opening_balance: balance.opening_balance,
                period_debits: balance.period_debits,
                period_credits: balance.period_credits,
                closing_balance: balance.closing_balance,
                cost_center: None,
                profit_center: None,
            });
        }

        // Sort lines
        if self.config.sort_by_account_code {
            lines.sort_by(|a, b| a.account_code.cmp(&b.account_code));
        }

        // Calculate category summaries
        let category_summary = if self.config.group_by_category {
            self.calculate_category_summary(&lines)
        } else {
            HashMap::new()
        };

        let out_of_balance = total_debits - total_credits;

        let mut tb = TrialBalance {
            trial_balance_id: format!(
                "TB-{}-{}-{:02}",
                snapshot.company_code, fiscal_year, fiscal_period
            ),
            company_code: snapshot.company_code.clone(),
            company_name: None,
            as_of_date: snapshot.as_of_date,
            fiscal_year,
            fiscal_period,
            currency: snapshot.currency.clone(),
            balance_type: self.config.trial_balance_type,
            lines,
            total_debits,
            total_credits,
            is_balanced: out_of_balance.abs() < dec!(0.01),
            out_of_balance,
            is_equation_valid: false,           // Will be calculated below
            equation_difference: Decimal::ZERO, // Will be calculated below
            category_summary,
            created_at: snapshot
                .as_of_date
                .and_hms_opt(23, 59, 59)
                .unwrap_or_default(),
            created_by: "TrialBalanceGenerator".to_string(),
            approved_by: None,
            approved_at: None,
            status: TrialBalanceStatus::Draft,
        };

        // Calculate and set accounting equation validity
        let (is_valid, _assets, _liabilities, _equity, diff) = tb.validate_accounting_equation();
        tb.is_equation_valid = is_valid;
        tb.equation_difference = diff;

        tb
    }

    /// Generates a trial balance from the balance tracker.
    pub fn generate_from_tracker(
        &self,
        tracker: &RunningBalanceTracker,
        company_code: &str,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Option<TrialBalance> {
        tracker
            .get_snapshot(company_code, as_of_date)
            .map(|snapshot| self.generate_from_snapshot(&snapshot, fiscal_year, fiscal_period))
    }

    /// Generates trial balances for all companies in the tracker.
    pub fn generate_all_from_tracker(
        &self,
        tracker: &RunningBalanceTracker,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Vec<TrialBalance> {
        tracker
            .get_all_snapshots(as_of_date)
            .iter()
            .map(|snapshot| self.generate_from_snapshot(snapshot, fiscal_year, fiscal_period))
            .collect()
    }

    /// Generates a comparative trial balance across multiple periods.
    pub fn generate_comparative(
        &self,
        snapshots: &[(NaiveDate, BalanceSnapshot)],
        fiscal_year: i32,
    ) -> ComparativeTrialBalance {
        use datasynth_core::models::balance::ComparativeTrialBalanceLine;

        // Generate trial balances for each period
        let trial_balances: Vec<TrialBalance> = snapshots
            .iter()
            .enumerate()
            .map(|(i, (date, snapshot))| {
                let mut tb = self.generate_from_snapshot(snapshot, fiscal_year, (i + 1) as u32);
                tb.as_of_date = *date;
                tb
            })
            .collect();

        // Build periods list
        let periods: Vec<(i32, u32)> = trial_balances
            .iter()
            .map(|tb| (tb.fiscal_year, tb.fiscal_period))
            .collect();

        // Build comparative lines
        let mut lines_map: HashMap<String, ComparativeTrialBalanceLine> = HashMap::new();

        for tb in &trial_balances {
            for line in &tb.lines {
                let entry = lines_map
                    .entry(line.account_code.clone())
                    .or_insert_with(|| ComparativeTrialBalanceLine {
                        account_code: line.account_code.clone(),
                        account_description: line.account_description.clone(),
                        category: line.category,
                        period_balances: HashMap::new(),
                        period_changes: HashMap::new(),
                    });

                entry
                    .period_balances
                    .insert((tb.fiscal_year, tb.fiscal_period), line.closing_balance);
            }
        }

        // Calculate period-over-period changes
        for line in lines_map.values_mut() {
            let mut sorted_periods: Vec<_> = line.period_balances.keys().cloned().collect();
            sorted_periods.sort();

            for i in 1..sorted_periods.len() {
                let prev_period = sorted_periods[i - 1];
                let curr_period = sorted_periods[i];

                if let (Some(&prev_balance), Some(&curr_balance)) = (
                    line.period_balances.get(&prev_period),
                    line.period_balances.get(&curr_period),
                ) {
                    line.period_changes
                        .insert(curr_period, curr_balance - prev_balance);
                }
            }
        }

        let lines: Vec<ComparativeTrialBalanceLine> = lines_map.into_values().collect();

        let company_code = snapshots
            .first()
            .map(|(_, s)| s.company_code.clone())
            .unwrap_or_default();

        let currency = snapshots
            .first()
            .map(|(_, s)| s.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        let created_at = snapshots
            .last()
            .map(|(date, _)| date.and_hms_opt(23, 59, 59).unwrap_or_default())
            .unwrap_or_default();

        ComparativeTrialBalance {
            company_code,
            currency,
            periods,
            lines,
            created_at,
        }
    }

    /// Generates a consolidated trial balance across companies.
    pub fn generate_consolidated(
        &self,
        trial_balances: &[TrialBalance],
        consolidated_company_code: &str,
    ) -> TrialBalance {
        let mut consolidated_balances: HashMap<String, TrialBalanceLine> = HashMap::new();

        for tb in trial_balances {
            for line in &tb.lines {
                let entry = consolidated_balances
                    .entry(line.account_code.clone())
                    .or_insert_with(|| TrialBalanceLine {
                        account_code: line.account_code.clone(),
                        account_description: line.account_description.clone(),
                        category: line.category,
                        account_type: line.account_type,
                        debit_balance: Decimal::ZERO,
                        credit_balance: Decimal::ZERO,
                        opening_balance: Decimal::ZERO,
                        period_debits: Decimal::ZERO,
                        period_credits: Decimal::ZERO,
                        closing_balance: Decimal::ZERO,
                        cost_center: None,
                        profit_center: None,
                    });

                entry.debit_balance += line.debit_balance;
                entry.credit_balance += line.credit_balance;
                entry.opening_balance += line.opening_balance;
                entry.period_debits += line.period_debits;
                entry.period_credits += line.period_credits;
                entry.closing_balance += line.closing_balance;
            }
        }

        let mut lines: Vec<TrialBalanceLine> = consolidated_balances.into_values().collect();
        if self.config.sort_by_account_code {
            lines.sort_by(|a, b| a.account_code.cmp(&b.account_code));
        }

        let total_debits: Decimal = lines.iter().map(|l| l.debit_balance).sum();
        let total_credits: Decimal = lines.iter().map(|l| l.credit_balance).sum();

        let category_summary = if self.config.group_by_category {
            self.calculate_category_summary(&lines)
        } else {
            HashMap::new()
        };

        let as_of_date = trial_balances
            .first()
            .map(|tb| tb.as_of_date)
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let fiscal_year = trial_balances.first().map(|tb| tb.fiscal_year).unwrap_or(0);
        let fiscal_period = trial_balances
            .first()
            .map(|tb| tb.fiscal_period)
            .unwrap_or(0);

        let currency = trial_balances
            .first()
            .map(|tb| tb.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        let out_of_balance = total_debits - total_credits;

        let mut tb = TrialBalance {
            trial_balance_id: format!(
                "TB-CONS-{}-{}-{:02}",
                consolidated_company_code, fiscal_year, fiscal_period
            ),
            company_code: consolidated_company_code.to_string(),
            company_name: None,
            as_of_date,
            fiscal_year,
            fiscal_period,
            currency,
            balance_type: TrialBalanceType::Consolidated,
            lines,
            total_debits,
            total_credits,
            is_balanced: out_of_balance.abs() < dec!(0.01),
            out_of_balance,
            is_equation_valid: false,           // Will be calculated below
            equation_difference: Decimal::ZERO, // Will be calculated below
            category_summary,
            created_at: as_of_date.and_hms_opt(23, 59, 59).unwrap_or_default(),
            created_by: format!(
                "TrialBalanceGenerator (Consolidated from {} companies)",
                trial_balances.len()
            ),
            approved_by: None,
            approved_at: None,
            status: TrialBalanceStatus::Draft,
        };

        // Calculate and set accounting equation validity
        let (is_valid, _assets, _liabilities, _equity, diff) = tb.validate_accounting_equation();
        tb.is_equation_valid = is_valid;
        tb.equation_difference = diff;

        tb
    }

    /// Splits a balance into debit and credit components.
    fn split_balance(&self, balance: &AccountBalance) -> (Decimal, Decimal) {
        let closing = balance.closing_balance;

        // Determine natural balance side based on account type
        match balance.account_type {
            AccountType::Asset | AccountType::Expense => {
                if closing >= Decimal::ZERO {
                    (closing, Decimal::ZERO)
                } else {
                    (Decimal::ZERO, closing.abs())
                }
            }
            AccountType::ContraAsset | AccountType::ContraLiability | AccountType::ContraEquity => {
                // Contra accounts have opposite natural balance
                if closing >= Decimal::ZERO {
                    (Decimal::ZERO, closing)
                } else {
                    (closing.abs(), Decimal::ZERO)
                }
            }
            AccountType::Liability | AccountType::Equity | AccountType::Revenue => {
                if closing >= Decimal::ZERO {
                    (Decimal::ZERO, closing)
                } else {
                    (closing.abs(), Decimal::ZERO)
                }
            }
        }
    }

    /// Determines account category from code prefix.
    ///
    /// Checks explicitly registered mappings first, then falls back to the
    /// framework-aware classifier from [`FrameworkAccounts`].
    fn determine_category(&self, account_code: &str) -> AccountCategory {
        // Check registered mappings first
        if let Some(category) = self.category_mappings.get(account_code) {
            return *category;
        }

        // Use framework-aware classification
        self.framework_accounts
            .classify_trial_balance_category(account_code)
    }

    /// Calculates category summaries from lines.
    fn calculate_category_summary(
        &self,
        lines: &[TrialBalanceLine],
    ) -> HashMap<AccountCategory, CategorySummary> {
        let mut summaries: HashMap<AccountCategory, CategorySummary> = HashMap::new();

        for line in lines {
            let summary = summaries
                .entry(line.category)
                .or_insert_with(|| CategorySummary::new(line.category));

            summary.add_balance(line.debit_balance, line.credit_balance);
        }

        summaries
    }

    /// Finalizes a trial balance (changes status to Final).
    pub fn finalize(&self, mut trial_balance: TrialBalance) -> TrialBalance {
        trial_balance.status = TrialBalanceStatus::Final;
        trial_balance
    }

    /// Approves a trial balance.
    pub fn approve(&self, mut trial_balance: TrialBalance, approver: &str) -> TrialBalance {
        trial_balance.status = TrialBalanceStatus::Approved;
        trial_balance.approved_by = Some(approver.to_string());
        trial_balance.approved_at = Some(
            trial_balance
                .as_of_date
                .succ_opt()
                .unwrap_or(trial_balance.as_of_date)
                .and_hms_opt(9, 0, 0)
                .unwrap_or_default(),
        );
        trial_balance
    }
}

/// Builder for trial balance generation with fluent API.
pub struct TrialBalanceBuilder {
    generator: TrialBalanceGenerator,
    snapshots: Vec<(String, BalanceSnapshot)>,
    fiscal_year: i32,
    fiscal_period: u32,
}

impl TrialBalanceBuilder {
    /// Creates a new builder.
    pub fn new(fiscal_year: i32, fiscal_period: u32) -> Self {
        Self {
            generator: TrialBalanceGenerator::with_defaults(),
            snapshots: Vec::new(),
            fiscal_year,
            fiscal_period,
        }
    }

    /// Adds a balance snapshot.
    pub fn add_snapshot(mut self, company_code: &str, snapshot: BalanceSnapshot) -> Self {
        self.snapshots.push((company_code.to_string(), snapshot));
        self
    }

    /// Sets configuration.
    pub fn with_config(mut self, config: TrialBalanceConfig) -> Self {
        self.generator = TrialBalanceGenerator::new(config);
        self
    }

    /// Builds individual trial balances.
    pub fn build(self) -> Vec<TrialBalance> {
        self.snapshots
            .iter()
            .map(|(_, snapshot)| {
                self.generator.generate_from_snapshot(
                    snapshot,
                    self.fiscal_year,
                    self.fiscal_period,
                )
            })
            .collect()
    }

    /// Builds a consolidated trial balance.
    pub fn build_consolidated(self, consolidated_code: &str) -> TrialBalance {
        let individual = self
            .snapshots
            .iter()
            .map(|(_, snapshot)| {
                self.generator.generate_from_snapshot(
                    snapshot,
                    self.fiscal_year,
                    self.fiscal_period,
                )
            })
            .collect::<Vec<_>>();

        self.generator
            .generate_consolidated(&individual, consolidated_code)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_balance(
        company: &str,
        account: &str,
        acct_type: AccountType,
        opening: Decimal,
    ) -> AccountBalance {
        let mut bal = AccountBalance::new(
            company.to_string(),
            account.to_string(),
            acct_type,
            "USD".to_string(),
            2024,
            1,
        );
        bal.opening_balance = opening;
        bal.closing_balance = opening;
        bal
    }

    fn create_test_snapshot() -> BalanceSnapshot {
        let mut snapshot = BalanceSnapshot::new(
            "SNAP-TEST-2024-01".to_string(),
            "TEST".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            2024,
            1,
            "USD".to_string(),
        );

        // Add assets
        snapshot.balances.insert(
            "1100".to_string(),
            create_test_balance("TEST", "1100", AccountType::Asset, dec!(10000)),
        );

        // Add liabilities
        snapshot.balances.insert(
            "2100".to_string(),
            create_test_balance("TEST", "2100", AccountType::Liability, dec!(5000)),
        );

        // Add equity
        snapshot.balances.insert(
            "3100".to_string(),
            create_test_balance("TEST", "3100", AccountType::Equity, dec!(5000)),
        );

        snapshot.recalculate_totals();
        snapshot
    }

    #[test]
    fn test_generate_trial_balance() {
        let generator = TrialBalanceGenerator::with_defaults();
        let snapshot = create_test_snapshot();

        let tb = generator.generate_from_snapshot(&snapshot, 2024, 1);

        assert!(tb.is_balanced);
        assert_eq!(tb.lines.len(), 3);
        assert_eq!(tb.total_debits, dec!(10000));
        assert_eq!(tb.total_credits, dec!(10000));
    }

    #[test]
    fn test_category_summaries() {
        let generator = TrialBalanceGenerator::with_defaults();
        let snapshot = create_test_snapshot();

        let tb = generator.generate_from_snapshot(&snapshot, 2024, 1);

        assert!(!tb.category_summary.is_empty());
    }

    #[test]
    fn test_consolidated_trial_balance() {
        let generator = TrialBalanceGenerator::with_defaults();

        let snapshot1 = create_test_snapshot();
        let mut snapshot2 = BalanceSnapshot::new(
            "SNAP-TEST2-2024-01".to_string(),
            "TEST2".to_string(),
            snapshot1.as_of_date,
            2024,
            1,
            "USD".to_string(),
        );

        // Copy and double the balances
        for (code, balance) in &snapshot1.balances {
            let mut new_bal = balance.clone();
            new_bal.company_code = "TEST2".to_string();
            new_bal.closing_balance *= dec!(2);
            new_bal.opening_balance *= dec!(2);
            snapshot2.balances.insert(code.clone(), new_bal);
        }
        snapshot2.recalculate_totals();

        let tb1 = generator.generate_from_snapshot(&snapshot1, 2024, 1);
        let tb2 = generator.generate_from_snapshot(&snapshot2, 2024, 1);

        let consolidated = generator.generate_consolidated(&[tb1, tb2], "CONSOL");

        assert_eq!(consolidated.company_code, "CONSOL");
        assert!(consolidated.is_balanced);
    }

    #[test]
    fn test_builder_pattern() {
        let snapshot = create_test_snapshot();

        let trial_balances = TrialBalanceBuilder::new(2024, 1)
            .add_snapshot("TEST", snapshot)
            .build();

        assert_eq!(trial_balances.len(), 1);
        assert!(trial_balances[0].is_balanced);
    }
}
