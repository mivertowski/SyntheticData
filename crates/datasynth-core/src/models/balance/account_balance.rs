//! Account balance and balance snapshot models.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Account balance for a single GL account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Company code.
    pub company_code: String,
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: Option<String>,
    /// Account type (Asset, Liability, Equity, Revenue, Expense).
    pub account_type: AccountType,
    /// Currency.
    pub currency: String,
    /// Opening balance (beginning of period).
    pub opening_balance: Decimal,
    /// Period debits.
    pub period_debits: Decimal,
    /// Period credits.
    pub period_credits: Decimal,
    /// Closing balance (end of period).
    pub closing_balance: Decimal,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u32,
    /// Balance in group currency (for consolidation).
    pub group_currency_balance: Option<Decimal>,
    /// Exchange rate used.
    pub exchange_rate: Option<Decimal>,
    /// Cost center (if applicable).
    pub cost_center: Option<String>,
    /// Profit center (if applicable).
    pub profit_center: Option<String>,
    /// Last updated timestamp.
    pub last_updated: NaiveDateTime,
}

impl AccountBalance {
    /// Create a new account balance.
    pub fn new(
        company_code: String,
        account_code: String,
        account_type: AccountType,
        currency: String,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Self {
        Self {
            company_code,
            account_code,
            account_description: None,
            account_type,
            currency,
            opening_balance: Decimal::ZERO,
            period_debits: Decimal::ZERO,
            period_credits: Decimal::ZERO,
            closing_balance: Decimal::ZERO,
            fiscal_year,
            fiscal_period,
            group_currency_balance: None,
            exchange_rate: None,
            cost_center: None,
            profit_center: None,
            last_updated: chrono::Utc::now().naive_utc(),
        }
    }

    /// Apply a debit to this balance.
    pub fn apply_debit(&mut self, amount: Decimal) {
        self.period_debits += amount;
        self.recalculate_closing();
    }

    /// Apply a credit to this balance.
    pub fn apply_credit(&mut self, amount: Decimal) {
        self.period_credits += amount;
        self.recalculate_closing();
    }

    /// Recalculate closing balance based on account type.
    fn recalculate_closing(&mut self) {
        // Asset and Expense accounts: Debit increases, Credit decreases
        // Liability, Equity, Revenue accounts: Credit increases, Debit decreases
        match self.account_type {
            AccountType::Asset
            | AccountType::Expense
            | AccountType::ContraLiability
            | AccountType::ContraEquity => {
                self.closing_balance =
                    self.opening_balance + self.period_debits - self.period_credits;
            }
            AccountType::Liability
            | AccountType::Equity
            | AccountType::Revenue
            | AccountType::ContraAsset => {
                self.closing_balance =
                    self.opening_balance - self.period_debits + self.period_credits;
            }
        }
        self.last_updated = chrono::Utc::now().naive_utc();
    }

    /// Set opening balance.
    pub fn set_opening_balance(&mut self, balance: Decimal) {
        self.opening_balance = balance;
        self.recalculate_closing();
    }

    /// Get the net change for the period.
    pub fn net_change(&self) -> Decimal {
        match self.account_type {
            AccountType::Asset
            | AccountType::Expense
            | AccountType::ContraLiability
            | AccountType::ContraEquity => self.period_debits - self.period_credits,
            AccountType::Liability
            | AccountType::Equity
            | AccountType::Revenue
            | AccountType::ContraAsset => self.period_credits - self.period_debits,
        }
    }

    /// Check if this is a debit-normal account.
    pub fn is_debit_normal(&self) -> bool {
        matches!(
            self.account_type,
            AccountType::Asset
                | AccountType::Expense
                | AccountType::ContraLiability
                | AccountType::ContraEquity
        )
    }

    /// Get the normal balance (positive closing for correct sign).
    pub fn normal_balance(&self) -> Decimal {
        if self.is_debit_normal() {
            self.closing_balance
        } else {
            -self.closing_balance
        }
    }

    /// Roll forward to next period.
    pub fn roll_forward(&mut self) {
        self.opening_balance = self.closing_balance;
        self.period_debits = Decimal::ZERO;
        self.period_credits = Decimal::ZERO;

        // Increment period
        if self.fiscal_period == 12 {
            self.fiscal_period = 1;
            self.fiscal_year += 1;
        } else {
            self.fiscal_period += 1;
        }

        self.last_updated = chrono::Utc::now().naive_utc();
    }

    /// Check if this is a balance sheet account.
    pub fn is_balance_sheet(&self) -> bool {
        matches!(
            self.account_type,
            AccountType::Asset
                | AccountType::Liability
                | AccountType::Equity
                | AccountType::ContraAsset
                | AccountType::ContraLiability
                | AccountType::ContraEquity
        )
    }

    /// Check if this is an income statement account.
    pub fn is_income_statement(&self) -> bool {
        matches!(
            self.account_type,
            AccountType::Revenue | AccountType::Expense
        )
    }
}

/// Account type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Assets (debit normal).
    #[default]
    Asset,
    /// Contra-asset (credit normal, e.g., accumulated depreciation).
    ContraAsset,
    /// Liabilities (credit normal).
    Liability,
    /// Contra-liability (debit normal).
    ContraLiability,
    /// Equity (credit normal).
    Equity,
    /// Contra-equity (debit normal, e.g., treasury stock).
    ContraEquity,
    /// Revenue (credit normal).
    Revenue,
    /// Expenses (debit normal).
    Expense,
}

impl AccountType {
    /// Determine account type from account code (simplified, US GAAP heuristic).
    ///
    /// For framework-aware classification, use
    /// [`from_account_code_with_framework`](Self::from_account_code_with_framework).
    pub fn from_account_code(code: &str) -> Self {
        let first_char = code.chars().next().unwrap_or('0');
        match first_char {
            '1' => Self::Asset,
            '2' => Self::Liability,
            '3' => Self::Equity,
            '4' => Self::Revenue,
            '5' | '6' | '7' | '8' => Self::Expense,
            _ => Self::Asset,
        }
    }

    /// Determine account type using framework-aware classification.
    ///
    /// `framework` is the framework string (e.g. `"us_gaap"`, `"french_gaap"`,
    /// `"german_gaap"`, `"ifrs"`). Uses [`FrameworkAccounts`] internally.
    pub fn from_account_code_with_framework(code: &str, framework: &str) -> Self {
        crate::framework_accounts::FrameworkAccounts::for_framework(framework)
            .classify_account_type(code)
    }

    /// Check if contra account based on code pattern.
    pub fn is_contra_from_code(code: &str) -> bool {
        // Common patterns for contra accounts
        code.contains("ACCUM") || code.contains("ALLOW") || code.contains("CONTRA")
    }
}

/// A snapshot of all account balances at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    /// Snapshot identifier.
    pub snapshot_id: String,
    /// Company code.
    pub company_code: String,
    /// Snapshot date.
    pub as_of_date: NaiveDate,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u32,
    /// Currency.
    pub currency: String,
    /// All account balances.
    pub balances: HashMap<String, AccountBalance>,
    /// Total assets.
    pub total_assets: Decimal,
    /// Total liabilities.
    pub total_liabilities: Decimal,
    /// Total equity.
    pub total_equity: Decimal,
    /// Total revenue.
    pub total_revenue: Decimal,
    /// Total expenses.
    pub total_expenses: Decimal,
    /// Net income.
    pub net_income: Decimal,
    /// Is the balance sheet balanced (A = L + E)?
    pub is_balanced: bool,
    /// Balance sheet difference (should be zero).
    pub balance_difference: Decimal,
    /// Created timestamp.
    pub created_at: NaiveDateTime,
}

impl BalanceSnapshot {
    /// Create a new balance snapshot.
    pub fn new(
        snapshot_id: String,
        company_code: String,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
        currency: String,
    ) -> Self {
        Self {
            snapshot_id,
            company_code,
            as_of_date,
            fiscal_year,
            fiscal_period,
            currency,
            balances: HashMap::new(),
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            total_revenue: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            net_income: Decimal::ZERO,
            is_balanced: true,
            balance_difference: Decimal::ZERO,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    /// Add an account balance to the snapshot.
    pub fn add_balance(&mut self, balance: AccountBalance) {
        let closing = balance.closing_balance;

        match balance.account_type {
            AccountType::Asset => self.total_assets += closing,
            AccountType::ContraAsset => self.total_assets -= closing,
            AccountType::Liability => self.total_liabilities += closing,
            AccountType::ContraLiability => self.total_liabilities -= closing,
            AccountType::Equity => self.total_equity += closing,
            AccountType::ContraEquity => self.total_equity -= closing,
            AccountType::Revenue => self.total_revenue += closing,
            AccountType::Expense => self.total_expenses += closing,
        }

        self.balances.insert(balance.account_code.clone(), balance);
        self.recalculate_totals();
    }

    /// Recalculate totals and validate balance sheet equation.
    pub fn recalculate_totals(&mut self) {
        self.net_income = self.total_revenue - self.total_expenses;

        // Balance sheet equation: Assets = Liabilities + Equity
        // For current period, equity includes net income
        let total_equity_with_income = self.total_equity + self.net_income;
        self.balance_difference =
            self.total_assets - self.total_liabilities - total_equity_with_income;
        self.is_balanced = self.balance_difference.abs() < dec!(0.01);
    }

    /// Get balance for a specific account.
    pub fn get_balance(&self, account_code: &str) -> Option<&AccountBalance> {
        self.balances.get(account_code)
    }

    /// Get all asset balances.
    pub fn get_asset_balances(&self) -> Vec<&AccountBalance> {
        self.balances
            .values()
            .filter(|b| {
                matches!(
                    b.account_type,
                    AccountType::Asset | AccountType::ContraAsset
                )
            })
            .collect()
    }

    /// Get all liability balances.
    pub fn get_liability_balances(&self) -> Vec<&AccountBalance> {
        self.balances
            .values()
            .filter(|b| {
                matches!(
                    b.account_type,
                    AccountType::Liability | AccountType::ContraLiability
                )
            })
            .collect()
    }

    /// Get all equity balances.
    pub fn get_equity_balances(&self) -> Vec<&AccountBalance> {
        self.balances
            .values()
            .filter(|b| {
                matches!(
                    b.account_type,
                    AccountType::Equity | AccountType::ContraEquity
                )
            })
            .collect()
    }

    /// Get all income statement balances.
    pub fn get_income_statement_balances(&self) -> Vec<&AccountBalance> {
        self.balances
            .values()
            .filter(|b| b.is_income_statement())
            .collect()
    }

    /// Get current ratio (Current Assets / Current Liabilities).
    pub fn current_ratio(
        &self,
        current_asset_accounts: &[&str],
        current_liability_accounts: &[&str],
    ) -> Option<Decimal> {
        let current_assets: Decimal = current_asset_accounts
            .iter()
            .filter_map(|code| self.balances.get(*code))
            .map(|b| b.closing_balance)
            .sum();

        let current_liabilities: Decimal = current_liability_accounts
            .iter()
            .filter_map(|code| self.balances.get(*code))
            .map(|b| b.closing_balance)
            .sum();

        if current_liabilities != Decimal::ZERO {
            Some(current_assets / current_liabilities)
        } else {
            None
        }
    }

    /// Get debt-to-equity ratio.
    pub fn debt_to_equity_ratio(&self) -> Option<Decimal> {
        if self.total_equity != Decimal::ZERO {
            Some(self.total_liabilities / self.total_equity)
        } else {
            None
        }
    }

    /// Get gross margin (Revenue - COGS) / Revenue.
    pub fn gross_margin(&self, cogs_accounts: &[&str]) -> Option<Decimal> {
        if self.total_revenue == Decimal::ZERO {
            return None;
        }

        let cogs: Decimal = cogs_accounts
            .iter()
            .filter_map(|code| self.balances.get(*code))
            .map(|b| b.closing_balance)
            .sum();

        Some((self.total_revenue - cogs) / self.total_revenue)
    }
}

/// Period-over-period balance change analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: Option<String>,
    /// Prior period balance.
    pub prior_balance: Decimal,
    /// Current period balance.
    pub current_balance: Decimal,
    /// Absolute change.
    pub change_amount: Decimal,
    /// Percentage change.
    pub change_percent: Option<Decimal>,
    /// Is this a significant change (above threshold)?
    pub is_significant: bool,
}

impl BalanceChange {
    /// Create a new balance change analysis.
    pub fn new(
        account_code: String,
        account_description: Option<String>,
        prior_balance: Decimal,
        current_balance: Decimal,
        significance_threshold: Decimal,
    ) -> Self {
        let change_amount = current_balance - prior_balance;
        let change_percent = if prior_balance != Decimal::ZERO {
            Some((change_amount / prior_balance.abs()) * dec!(100))
        } else {
            None
        };

        let is_significant = change_amount.abs() >= significance_threshold
            || change_percent.is_some_and(|p| p.abs() >= dec!(10));

        Self {
            account_code,
            account_description,
            prior_balance,
            current_balance,
            change_amount,
            change_percent,
            is_significant,
        }
    }
}

/// Account activity tracking within a period.
///
/// Tracks debits, credits, and transaction counts for an account
/// over a specific period.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountPeriodActivity {
    /// Account code.
    pub account_code: String,
    /// Period start date.
    pub period_start: NaiveDate,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Opening balance at period start.
    pub opening_balance: Decimal,
    /// Closing balance at period end.
    pub closing_balance: Decimal,
    /// Total debit amounts during period.
    pub total_debits: Decimal,
    /// Total credit amounts during period.
    pub total_credits: Decimal,
    /// Net change (total_debits - total_credits).
    pub net_change: Decimal,
    /// Number of transactions during period.
    pub transaction_count: u32,
}

impl AccountPeriodActivity {
    /// Create a new account period activity tracker.
    pub fn new(account_code: String, period_start: NaiveDate, period_end: NaiveDate) -> Self {
        Self {
            account_code,
            period_start,
            period_end,
            opening_balance: Decimal::ZERO,
            closing_balance: Decimal::ZERO,
            total_debits: Decimal::ZERO,
            total_credits: Decimal::ZERO,
            net_change: Decimal::ZERO,
            transaction_count: 0,
        }
    }

    /// Add a debit transaction.
    pub fn add_debit(&mut self, amount: Decimal) {
        self.total_debits += amount;
        self.net_change += amount;
        self.transaction_count += 1;
    }

    /// Add a credit transaction.
    pub fn add_credit(&mut self, amount: Decimal) {
        self.total_credits += amount;
        self.net_change -= amount;
        self.transaction_count += 1;
    }
}

/// Compare two snapshots and identify changes.
pub fn compare_snapshots(
    prior: &BalanceSnapshot,
    current: &BalanceSnapshot,
    significance_threshold: Decimal,
) -> Vec<BalanceChange> {
    let mut changes = Vec::new();

    // Get all unique account codes
    let mut all_accounts: Vec<&str> = prior
        .balances
        .keys()
        .map(std::string::String::as_str)
        .collect();
    for code in current.balances.keys() {
        if !all_accounts.contains(&code.as_str()) {
            all_accounts.push(code.as_str());
        }
    }

    for account_code in all_accounts {
        let prior_balance = prior
            .balances
            .get(account_code)
            .map(|b| b.closing_balance)
            .unwrap_or(Decimal::ZERO);

        let current_balance = current
            .balances
            .get(account_code)
            .map(|b| b.closing_balance)
            .unwrap_or(Decimal::ZERO);

        let description = current
            .balances
            .get(account_code)
            .and_then(|b| b.account_description.clone())
            .or_else(|| {
                prior
                    .balances
                    .get(account_code)
                    .and_then(|b| b.account_description.clone())
            });

        if prior_balance != current_balance {
            changes.push(BalanceChange::new(
                account_code.to_string(),
                description,
                prior_balance,
                current_balance,
                significance_threshold,
            ));
        }
    }

    changes
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_account_balance_debit_normal() {
        let mut balance = AccountBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            AccountType::Asset,
            "USD".to_string(),
            2022,
            6,
        );

        balance.set_opening_balance(dec!(10000));
        balance.apply_debit(dec!(5000));
        balance.apply_credit(dec!(2000));

        assert_eq!(balance.closing_balance, dec!(13000)); // 10000 + 5000 - 2000
        assert_eq!(balance.net_change(), dec!(3000));
    }

    #[test]
    fn test_account_balance_credit_normal() {
        let mut balance = AccountBalance::new(
            "1000".to_string(),
            "2100".to_string(),
            AccountType::Liability,
            "USD".to_string(),
            2022,
            6,
        );

        balance.set_opening_balance(dec!(10000));
        balance.apply_credit(dec!(5000));
        balance.apply_debit(dec!(2000));

        assert_eq!(balance.closing_balance, dec!(13000)); // 10000 - 2000 + 5000
        assert_eq!(balance.net_change(), dec!(3000));
    }

    #[test]
    fn test_balance_snapshot_balanced() {
        let mut snapshot = BalanceSnapshot::new(
            "SNAP001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            2022,
            6,
            "USD".to_string(),
        );

        // Add asset
        let mut cash = AccountBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            AccountType::Asset,
            "USD".to_string(),
            2022,
            6,
        );
        cash.closing_balance = dec!(50000);
        snapshot.add_balance(cash);

        // Add liability
        let mut ap = AccountBalance::new(
            "1000".to_string(),
            "2100".to_string(),
            AccountType::Liability,
            "USD".to_string(),
            2022,
            6,
        );
        ap.closing_balance = dec!(20000);
        snapshot.add_balance(ap);

        // Add equity
        let mut equity = AccountBalance::new(
            "1000".to_string(),
            "3100".to_string(),
            AccountType::Equity,
            "USD".to_string(),
            2022,
            6,
        );
        equity.closing_balance = dec!(30000);
        snapshot.add_balance(equity);

        assert!(snapshot.is_balanced);
        assert_eq!(snapshot.total_assets, dec!(50000));
        assert_eq!(snapshot.total_liabilities, dec!(20000));
        assert_eq!(snapshot.total_equity, dec!(30000));
    }

    #[test]
    fn test_balance_snapshot_with_income() {
        let mut snapshot = BalanceSnapshot::new(
            "SNAP001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            2022,
            6,
            "USD".to_string(),
        );

        // Assets = 60000
        let mut cash = AccountBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            AccountType::Asset,
            "USD".to_string(),
            2022,
            6,
        );
        cash.closing_balance = dec!(60000);
        snapshot.add_balance(cash);

        // Liabilities = 20000
        let mut ap = AccountBalance::new(
            "1000".to_string(),
            "2100".to_string(),
            AccountType::Liability,
            "USD".to_string(),
            2022,
            6,
        );
        ap.closing_balance = dec!(20000);
        snapshot.add_balance(ap);

        // Equity = 30000
        let mut equity = AccountBalance::new(
            "1000".to_string(),
            "3100".to_string(),
            AccountType::Equity,
            "USD".to_string(),
            2022,
            6,
        );
        equity.closing_balance = dec!(30000);
        snapshot.add_balance(equity);

        // Revenue = 50000
        let mut revenue = AccountBalance::new(
            "1000".to_string(),
            "4100".to_string(),
            AccountType::Revenue,
            "USD".to_string(),
            2022,
            6,
        );
        revenue.closing_balance = dec!(50000);
        snapshot.add_balance(revenue);

        // Expenses = 40000
        let mut expense = AccountBalance::new(
            "1000".to_string(),
            "5100".to_string(),
            AccountType::Expense,
            "USD".to_string(),
            2022,
            6,
        );
        expense.closing_balance = dec!(40000);
        snapshot.add_balance(expense);

        // Net income = 50000 - 40000 = 10000
        // A = 60000, L = 20000, E = 30000, NI = 10000
        // A = L + E + NI -> 60000 = 20000 + 30000 + 10000 ✓
        assert!(snapshot.is_balanced);
        assert_eq!(snapshot.net_income, dec!(10000));
    }

    #[test]
    fn test_account_type_from_code() {
        assert_eq!(AccountType::from_account_code("1100"), AccountType::Asset);
        assert_eq!(
            AccountType::from_account_code("2100"),
            AccountType::Liability
        );
        assert_eq!(AccountType::from_account_code("3100"), AccountType::Equity);
        assert_eq!(AccountType::from_account_code("4100"), AccountType::Revenue);
        assert_eq!(AccountType::from_account_code("5100"), AccountType::Expense);
    }

    #[test]
    fn test_account_type_from_code_with_framework_us_gaap() {
        assert_eq!(
            AccountType::from_account_code_with_framework("1100", "us_gaap"),
            AccountType::Asset
        );
        assert_eq!(
            AccountType::from_account_code_with_framework("4000", "us_gaap"),
            AccountType::Revenue
        );
    }

    #[test]
    fn test_account_type_from_code_with_framework_french_gaap() {
        // PCG class 1 (10x) = Equity, not Asset
        assert_eq!(
            AccountType::from_account_code_with_framework("101000", "french_gaap"),
            AccountType::Equity
        );
        // PCG class 2 = Fixed Assets
        assert_eq!(
            AccountType::from_account_code_with_framework("210000", "french_gaap"),
            AccountType::Asset
        );
        // PCG class 7 = Revenue
        assert_eq!(
            AccountType::from_account_code_with_framework("701000", "french_gaap"),
            AccountType::Revenue
        );
    }

    #[test]
    fn test_account_type_from_code_with_framework_german_gaap() {
        // SKR04 class 0 = Fixed Assets
        assert_eq!(
            AccountType::from_account_code_with_framework("0200", "german_gaap"),
            AccountType::Asset
        );
        // SKR04 class 2 = Equity (not Liability as US GAAP would say)
        assert_eq!(
            AccountType::from_account_code_with_framework("2000", "german_gaap"),
            AccountType::Equity
        );
        // SKR04 class 4 = Revenue
        assert_eq!(
            AccountType::from_account_code_with_framework("4000", "german_gaap"),
            AccountType::Revenue
        );
    }

    #[test]
    fn test_balance_roll_forward() {
        let mut balance = AccountBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            AccountType::Asset,
            "USD".to_string(),
            2022,
            12,
        );

        balance.set_opening_balance(dec!(10000));
        balance.apply_debit(dec!(5000));
        balance.roll_forward();

        assert_eq!(balance.opening_balance, dec!(15000));
        assert_eq!(balance.period_debits, Decimal::ZERO);
        assert_eq!(balance.fiscal_year, 2023);
        assert_eq!(balance.fiscal_period, 1);
    }
}
