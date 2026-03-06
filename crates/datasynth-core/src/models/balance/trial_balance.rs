//! Trial balance model and reporting structures.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::account_balance::{AccountBalance, AccountType};

/// A trial balance report for a company and period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalance {
    /// Trial balance identifier.
    pub trial_balance_id: String,
    /// Company code.
    pub company_code: String,
    /// Company name.
    pub company_name: Option<String>,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u32,
    /// Currency.
    pub currency: String,
    /// Trial balance type.
    pub balance_type: TrialBalanceType,
    /// Individual account lines.
    pub lines: Vec<TrialBalanceLine>,
    /// Total debits.
    pub total_debits: Decimal,
    /// Total credits.
    pub total_credits: Decimal,
    /// Is the trial balance balanced (debits = credits)?
    pub is_balanced: bool,
    /// Out of balance amount.
    pub out_of_balance: Decimal,
    /// Is the accounting equation valid (Assets = Liabilities + Equity)?
    pub is_equation_valid: bool,
    /// Difference in accounting equation (Assets - (Liabilities + Equity)).
    pub equation_difference: Decimal,
    /// Summary by account category.
    pub category_summary: HashMap<AccountCategory, CategorySummary>,
    /// Created timestamp.
    pub created_at: NaiveDateTime,
    /// Created by.
    pub created_by: String,
    /// Approved by (if applicable).
    pub approved_by: Option<String>,
    /// Approval date.
    pub approved_at: Option<NaiveDateTime>,
    /// Status.
    pub status: TrialBalanceStatus,
}

impl TrialBalance {
    /// Create a new trial balance.
    pub fn new(
        trial_balance_id: String,
        company_code: String,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
        currency: String,
        balance_type: TrialBalanceType,
    ) -> Self {
        Self {
            trial_balance_id,
            company_code,
            company_name: None,
            as_of_date,
            fiscal_year,
            fiscal_period,
            currency,
            balance_type,
            lines: Vec::new(),
            total_debits: Decimal::ZERO,
            total_credits: Decimal::ZERO,
            is_balanced: true,
            out_of_balance: Decimal::ZERO,
            is_equation_valid: true,
            equation_difference: Decimal::ZERO,
            category_summary: HashMap::new(),
            created_at: chrono::Utc::now().naive_utc(),
            created_by: "SYSTEM".to_string(),
            approved_by: None,
            approved_at: None,
            status: TrialBalanceStatus::Draft,
        }
    }

    /// Add a line to the trial balance.
    pub fn add_line(&mut self, line: TrialBalanceLine) {
        self.total_debits += line.debit_balance;
        self.total_credits += line.credit_balance;

        // Update category summary
        let summary = self
            .category_summary
            .entry(line.category)
            .or_insert_with(|| CategorySummary::new(line.category));
        summary.add_balance(line.debit_balance, line.credit_balance);

        self.lines.push(line);
        self.recalculate();
    }

    /// Add a line from an AccountBalance.
    pub fn add_from_account_balance(&mut self, balance: &AccountBalance) {
        let category = AccountCategory::from_account_type(balance.account_type);

        let (debit, credit) = if balance.is_debit_normal() {
            if balance.closing_balance >= Decimal::ZERO {
                (balance.closing_balance, Decimal::ZERO)
            } else {
                (Decimal::ZERO, balance.closing_balance.abs())
            }
        } else if balance.closing_balance >= Decimal::ZERO {
            (Decimal::ZERO, balance.closing_balance)
        } else {
            (balance.closing_balance.abs(), Decimal::ZERO)
        };

        let line = TrialBalanceLine {
            account_code: balance.account_code.clone(),
            account_description: balance.account_description.clone().unwrap_or_default(),
            category,
            account_type: balance.account_type,
            opening_balance: balance.opening_balance,
            period_debits: balance.period_debits,
            period_credits: balance.period_credits,
            closing_balance: balance.closing_balance,
            debit_balance: debit,
            credit_balance: credit,
            cost_center: balance.cost_center.clone(),
            profit_center: balance.profit_center.clone(),
        };

        self.add_line(line);
    }

    /// Recalculate totals and balance status.
    fn recalculate(&mut self) {
        // Check DR = CR
        self.out_of_balance = self.total_debits - self.total_credits;
        self.is_balanced = self.out_of_balance.abs() < dec!(0.01);

        // Check accounting equation: Assets = Liabilities + Equity
        // For balance sheet accounts only
        let assets = self.total_assets();
        let liabilities = self.total_liabilities();
        let equity = self.total_equity();

        self.equation_difference = assets - (liabilities + equity);
        self.is_equation_valid = self.equation_difference.abs() < dec!(0.01);
    }

    /// Validate the accounting equation (Assets = Liabilities + Equity).
    ///
    /// Returns true if the equation holds within tolerance, along with
    /// the calculated totals for each component.
    pub fn validate_accounting_equation(&self) -> (bool, Decimal, Decimal, Decimal, Decimal) {
        let assets = self.total_assets();
        let liabilities = self.total_liabilities();
        let equity = self.total_equity();
        let difference = assets - (liabilities + equity);
        let valid = difference.abs() < dec!(0.01);

        (valid, assets, liabilities, equity, difference)
    }

    /// Get lines for a specific category.
    pub fn get_lines_by_category(&self, category: AccountCategory) -> Vec<&TrialBalanceLine> {
        self.lines
            .iter()
            .filter(|l| l.category == category)
            .collect()
    }

    /// Get total for a category.
    pub fn get_category_total(&self, category: AccountCategory) -> Option<&CategorySummary> {
        self.category_summary.get(&category)
    }

    /// Get total assets.
    pub fn total_assets(&self) -> Decimal {
        self.category_summary
            .get(&AccountCategory::CurrentAssets)
            .map(CategorySummary::net_balance)
            .unwrap_or(Decimal::ZERO)
            + self
                .category_summary
                .get(&AccountCategory::NonCurrentAssets)
                .map(CategorySummary::net_balance)
                .unwrap_or(Decimal::ZERO)
    }

    /// Get total liabilities.
    pub fn total_liabilities(&self) -> Decimal {
        self.category_summary
            .get(&AccountCategory::CurrentLiabilities)
            .map(CategorySummary::net_balance)
            .unwrap_or(Decimal::ZERO)
            + self
                .category_summary
                .get(&AccountCategory::NonCurrentLiabilities)
                .map(CategorySummary::net_balance)
                .unwrap_or(Decimal::ZERO)
    }

    /// Get total equity.
    pub fn total_equity(&self) -> Decimal {
        self.category_summary
            .get(&AccountCategory::Equity)
            .map(CategorySummary::net_balance)
            .unwrap_or(Decimal::ZERO)
    }

    /// Get total revenue.
    pub fn total_revenue(&self) -> Decimal {
        self.category_summary
            .get(&AccountCategory::Revenue)
            .map(CategorySummary::net_balance)
            .unwrap_or(Decimal::ZERO)
    }

    /// Get total expenses.
    pub fn total_expenses(&self) -> Decimal {
        self.category_summary
            .get(&AccountCategory::CostOfGoodsSold)
            .map(CategorySummary::net_balance)
            .unwrap_or(Decimal::ZERO)
            + self
                .category_summary
                .get(&AccountCategory::OperatingExpenses)
                .map(CategorySummary::net_balance)
                .unwrap_or(Decimal::ZERO)
            + self
                .category_summary
                .get(&AccountCategory::OtherExpenses)
                .map(CategorySummary::net_balance)
                .unwrap_or(Decimal::ZERO)
    }

    /// Get net income.
    pub fn net_income(&self) -> Decimal {
        self.total_revenue() - self.total_expenses()
    }

    /// Finalize the trial balance.
    pub fn finalize(&mut self) {
        if self.is_balanced {
            self.status = TrialBalanceStatus::Final;
        }
    }

    /// Approve the trial balance.
    pub fn approve(&mut self, approved_by: String) {
        self.approved_by = Some(approved_by);
        self.approved_at = Some(chrono::Utc::now().naive_utc());
        self.status = TrialBalanceStatus::Approved;
    }

    /// Sort lines by account code.
    pub fn sort_by_account(&mut self) {
        self.lines
            .sort_by(|a, b| a.account_code.cmp(&b.account_code));
    }

    /// Sort lines by category then account code.
    pub fn sort_by_category(&mut self) {
        self.lines
            .sort_by(|a, b| match a.category.cmp(&b.category) {
                std::cmp::Ordering::Equal => a.account_code.cmp(&b.account_code),
                other => other,
            });
    }
}

/// Type of trial balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrialBalanceType {
    /// Unadjusted trial balance (before adjusting entries).
    Unadjusted,
    /// Adjusted trial balance (after adjusting entries).
    #[default]
    Adjusted,
    /// Post-closing trial balance (after closing entries).
    PostClosing,
    /// Consolidated trial balance.
    Consolidated,
}

/// Status of trial balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrialBalanceStatus {
    /// Draft - still being prepared.
    #[default]
    Draft,
    /// Final - period closed.
    Final,
    /// Approved - reviewed and approved.
    Approved,
    /// Archived.
    Archived,
}

/// A single line in a trial balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceLine {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: String,
    /// Account category.
    pub category: AccountCategory,
    /// Account type.
    pub account_type: AccountType,
    /// Opening balance.
    pub opening_balance: Decimal,
    /// Period debits.
    pub period_debits: Decimal,
    /// Period credits.
    pub period_credits: Decimal,
    /// Closing balance.
    pub closing_balance: Decimal,
    /// Debit balance (for trial balance display).
    pub debit_balance: Decimal,
    /// Credit balance (for trial balance display).
    pub credit_balance: Decimal,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Profit center.
    pub profit_center: Option<String>,
}

impl TrialBalanceLine {
    /// Get the net balance.
    pub fn net_balance(&self) -> Decimal {
        self.debit_balance - self.credit_balance
    }
}

/// Account category for grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountCategory {
    /// Current assets (cash, AR, inventory).
    CurrentAssets,
    /// Non-current assets (fixed assets, intangibles).
    NonCurrentAssets,
    /// Current liabilities (AP, short-term debt).
    CurrentLiabilities,
    /// Non-current liabilities (long-term debt).
    NonCurrentLiabilities,
    /// Equity (capital, retained earnings).
    Equity,
    /// Revenue.
    Revenue,
    /// Cost of goods sold.
    CostOfGoodsSold,
    /// Operating expenses.
    OperatingExpenses,
    /// Other income.
    OtherIncome,
    /// Other expenses.
    OtherExpenses,
}

impl AccountCategory {
    /// Determine category from account type.
    pub fn from_account_type(account_type: AccountType) -> Self {
        match account_type {
            AccountType::Asset | AccountType::ContraAsset => Self::CurrentAssets,
            AccountType::Liability | AccountType::ContraLiability => Self::CurrentLiabilities,
            AccountType::Equity | AccountType::ContraEquity => Self::Equity,
            AccountType::Revenue => Self::Revenue,
            AccountType::Expense => Self::OperatingExpenses,
        }
    }

    /// Determine category from account code (US GAAP heuristic).
    ///
    /// For framework-aware classification, use
    /// [`from_account_code_with_framework`](Self::from_account_code_with_framework).
    pub fn from_account_code(code: &str) -> Self {
        let prefix = code.chars().take(2).collect::<String>();
        match prefix.as_str() {
            "10" | "11" | "12" | "13" | "14" => Self::CurrentAssets,
            "15" | "16" | "17" | "18" | "19" => Self::NonCurrentAssets,
            "20" | "21" | "22" | "23" | "24" => Self::CurrentLiabilities,
            "25" | "26" | "27" | "28" | "29" => Self::NonCurrentLiabilities,
            "30" | "31" | "32" | "33" | "34" | "35" | "36" | "37" | "38" | "39" => Self::Equity,
            "40" | "41" | "42" | "43" | "44" => Self::Revenue,
            "50" | "51" | "52" => Self::CostOfGoodsSold,
            "60" | "61" | "62" | "63" | "64" | "65" | "66" | "67" | "68" | "69" => {
                Self::OperatingExpenses
            }
            "70" | "71" | "72" | "73" | "74" => Self::OtherIncome,
            "80" | "81" | "82" | "83" | "84" | "85" | "86" | "87" | "88" | "89" => {
                Self::OtherExpenses
            }
            _ => Self::OperatingExpenses,
        }
    }

    /// Determine category using framework-aware classification.
    ///
    /// `framework` is the framework string (e.g. `"us_gaap"`, `"french_gaap"`,
    /// `"german_gaap"`, `"ifrs"`). Uses [`FrameworkAccounts`] internally.
    pub fn from_account_code_with_framework(code: &str, framework: &str) -> Self {
        crate::framework_accounts::FrameworkAccounts::for_framework(framework)
            .classify_trial_balance_category(code)
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CurrentAssets => "Current Assets",
            Self::NonCurrentAssets => "Non-Current Assets",
            Self::CurrentLiabilities => "Current Liabilities",
            Self::NonCurrentLiabilities => "Non-Current Liabilities",
            Self::Equity => "Equity",
            Self::Revenue => "Revenue",
            Self::CostOfGoodsSold => "Cost of Goods Sold",
            Self::OperatingExpenses => "Operating Expenses",
            Self::OtherIncome => "Other Income",
            Self::OtherExpenses => "Other Expenses",
        }
    }

    /// Is this a balance sheet category?
    pub fn is_balance_sheet(&self) -> bool {
        matches!(
            self,
            Self::CurrentAssets
                | Self::NonCurrentAssets
                | Self::CurrentLiabilities
                | Self::NonCurrentLiabilities
                | Self::Equity
        )
    }

    /// Is this an income statement category?
    pub fn is_income_statement(&self) -> bool {
        !self.is_balance_sheet()
    }
}

/// Summary for an account category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category.
    pub category: AccountCategory,
    /// Number of accounts.
    pub account_count: usize,
    /// Total debits.
    pub total_debits: Decimal,
    /// Total credits.
    pub total_credits: Decimal,
}

impl CategorySummary {
    /// Create a new category summary.
    pub fn new(category: AccountCategory) -> Self {
        Self {
            category,
            account_count: 0,
            total_debits: Decimal::ZERO,
            total_credits: Decimal::ZERO,
        }
    }

    /// Add a balance to the summary.
    pub fn add_balance(&mut self, debit: Decimal, credit: Decimal) {
        self.account_count += 1;
        self.total_debits += debit;
        self.total_credits += credit;
    }

    /// Get net balance.
    pub fn net_balance(&self) -> Decimal {
        self.total_debits - self.total_credits
    }
}

/// Comparative trial balance (multiple periods).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeTrialBalance {
    /// Company code.
    pub company_code: String,
    /// Currency.
    pub currency: String,
    /// Periods included.
    pub periods: Vec<(i32, u32)>, // (fiscal_year, fiscal_period)
    /// Lines with balances for each period.
    pub lines: Vec<ComparativeTrialBalanceLine>,
    /// Created timestamp.
    pub created_at: NaiveDateTime,
}

/// A line in a comparative trial balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeTrialBalanceLine {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: String,
    /// Category.
    pub category: AccountCategory,
    /// Balances by period.
    pub period_balances: HashMap<(i32, u32), Decimal>,
    /// Period-over-period changes.
    pub period_changes: HashMap<(i32, u32), Decimal>,
}

impl ComparativeTrialBalance {
    /// Create from multiple trial balances.
    pub fn from_trial_balances(trial_balances: Vec<&TrialBalance>) -> Self {
        let first = trial_balances
            .first()
            .expect("At least one trial balance required");

        let periods: Vec<(i32, u32)> = trial_balances
            .iter()
            .map(|tb| (tb.fiscal_year, tb.fiscal_period))
            .collect();

        // Collect all unique accounts
        let mut account_map: HashMap<String, ComparativeTrialBalanceLine> = HashMap::new();

        for tb in &trial_balances {
            let period = (tb.fiscal_year, tb.fiscal_period);
            for line in &tb.lines {
                let entry = account_map
                    .entry(line.account_code.clone())
                    .or_insert_with(|| ComparativeTrialBalanceLine {
                        account_code: line.account_code.clone(),
                        account_description: line.account_description.clone(),
                        category: line.category,
                        period_balances: HashMap::new(),
                        period_changes: HashMap::new(),
                    });
                entry.period_balances.insert(period, line.closing_balance);
            }
        }

        // Calculate period-over-period changes
        let sorted_periods: Vec<(i32, u32)> = {
            let mut p = periods.clone();
            p.sort();
            p
        };

        for line in account_map.values_mut() {
            for i in 1..sorted_periods.len() {
                let prior = sorted_periods[i - 1];
                let current = sorted_periods[i];
                let prior_balance = line
                    .period_balances
                    .get(&prior)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let current_balance = line
                    .period_balances
                    .get(&current)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                line.period_changes
                    .insert(current, current_balance - prior_balance);
            }
        }

        Self {
            company_code: first.company_code.clone(),
            currency: first.currency.clone(),
            periods,
            lines: account_map.into_values().collect(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_balance_creation() {
        let mut tb = TrialBalance::new(
            "TB202206".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            2022,
            6,
            "USD".to_string(),
            TrialBalanceType::Adjusted,
        );

        // Add cash (asset - debit normal)
        tb.add_line(TrialBalanceLine {
            account_code: "1100".to_string(),
            account_description: "Cash".to_string(),
            category: AccountCategory::CurrentAssets,
            account_type: AccountType::Asset,
            opening_balance: dec!(10000),
            period_debits: dec!(50000),
            period_credits: dec!(30000),
            closing_balance: dec!(30000),
            debit_balance: dec!(30000),
            credit_balance: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
        });

        // Add AP (liability - credit normal)
        tb.add_line(TrialBalanceLine {
            account_code: "2100".to_string(),
            account_description: "Accounts Payable".to_string(),
            category: AccountCategory::CurrentLiabilities,
            account_type: AccountType::Liability,
            opening_balance: dec!(5000),
            period_debits: dec!(10000),
            period_credits: dec!(25000),
            closing_balance: dec!(20000),
            debit_balance: Decimal::ZERO,
            credit_balance: dec!(20000),
            cost_center: None,
            profit_center: None,
        });

        // Add equity
        tb.add_line(TrialBalanceLine {
            account_code: "3100".to_string(),
            account_description: "Common Stock".to_string(),
            category: AccountCategory::Equity,
            account_type: AccountType::Equity,
            opening_balance: dec!(10000),
            period_debits: Decimal::ZERO,
            period_credits: Decimal::ZERO,
            closing_balance: dec!(10000),
            debit_balance: Decimal::ZERO,
            credit_balance: dec!(10000),
            cost_center: None,
            profit_center: None,
        });

        assert_eq!(tb.total_debits, dec!(30000));
        assert_eq!(tb.total_credits, dec!(30000));
        assert!(tb.is_balanced);
    }

    #[test]
    fn test_trial_balance_from_account_balance() {
        let mut tb = TrialBalance::new(
            "TB202206".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            2022,
            6,
            "USD".to_string(),
            TrialBalanceType::Adjusted,
        );

        let mut cash = AccountBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            AccountType::Asset,
            "USD".to_string(),
            2022,
            6,
        );
        cash.account_description = Some("Cash".to_string());
        cash.set_opening_balance(dec!(10000));
        cash.apply_debit(dec!(5000));

        tb.add_from_account_balance(&cash);

        assert_eq!(tb.lines.len(), 1);
        assert_eq!(tb.lines[0].debit_balance, dec!(15000));
        assert_eq!(tb.lines[0].credit_balance, Decimal::ZERO);
    }

    #[test]
    fn test_account_category_from_code() {
        assert_eq!(
            AccountCategory::from_account_code("1100"),
            AccountCategory::CurrentAssets
        );
        assert_eq!(
            AccountCategory::from_account_code("1500"),
            AccountCategory::NonCurrentAssets
        );
        assert_eq!(
            AccountCategory::from_account_code("2100"),
            AccountCategory::CurrentLiabilities
        );
        assert_eq!(
            AccountCategory::from_account_code("2700"),
            AccountCategory::NonCurrentLiabilities
        );
        assert_eq!(
            AccountCategory::from_account_code("3100"),
            AccountCategory::Equity
        );
        assert_eq!(
            AccountCategory::from_account_code("4100"),
            AccountCategory::Revenue
        );
        assert_eq!(
            AccountCategory::from_account_code("5100"),
            AccountCategory::CostOfGoodsSold
        );
        assert_eq!(
            AccountCategory::from_account_code("6100"),
            AccountCategory::OperatingExpenses
        );
    }

    #[test]
    fn test_account_category_from_code_with_framework_us_gaap() {
        // Should produce results consistent with the original from_account_code
        assert_eq!(
            AccountCategory::from_account_code_with_framework("1100", "us_gaap"),
            AccountCategory::CurrentAssets
        );
        assert_eq!(
            AccountCategory::from_account_code_with_framework("4000", "us_gaap"),
            AccountCategory::Revenue
        );
        assert_eq!(
            AccountCategory::from_account_code_with_framework("5000", "us_gaap"),
            AccountCategory::CostOfGoodsSold
        );
    }

    #[test]
    fn test_account_category_from_code_with_framework_french_gaap() {
        // PCG: class 1 = Equity (not CurrentAssets)
        assert_eq!(
            AccountCategory::from_account_code_with_framework("101000", "french_gaap"),
            AccountCategory::Equity
        );
        // PCG: class 2 = Asset
        assert_eq!(
            AccountCategory::from_account_code_with_framework("210000", "french_gaap"),
            AccountCategory::CurrentAssets
        );
        // PCG: class 6 = OperatingExpenses
        assert_eq!(
            AccountCategory::from_account_code_with_framework("603000", "french_gaap"),
            AccountCategory::OperatingExpenses
        );
        // PCG: class 7 = Revenue
        assert_eq!(
            AccountCategory::from_account_code_with_framework("701000", "french_gaap"),
            AccountCategory::Revenue
        );
    }

    #[test]
    fn test_account_category_from_code_with_framework_german_gaap() {
        // SKR04: class 0 = Asset
        assert_eq!(
            AccountCategory::from_account_code_with_framework("0200", "german_gaap"),
            AccountCategory::CurrentAssets
        );
        // SKR04: class 2 = Equity
        assert_eq!(
            AccountCategory::from_account_code_with_framework("2000", "german_gaap"),
            AccountCategory::Equity
        );
        // SKR04: class 3 = Liability
        assert_eq!(
            AccountCategory::from_account_code_with_framework("3300", "german_gaap"),
            AccountCategory::CurrentLiabilities
        );
        // SKR04: class 5 = COGS
        assert_eq!(
            AccountCategory::from_account_code_with_framework("5000", "german_gaap"),
            AccountCategory::CostOfGoodsSold
        );
    }

    #[test]
    fn test_category_summary() {
        let mut summary = CategorySummary::new(AccountCategory::CurrentAssets);

        summary.add_balance(dec!(10000), Decimal::ZERO);
        summary.add_balance(dec!(5000), Decimal::ZERO);

        assert_eq!(summary.account_count, 2);
        assert_eq!(summary.total_debits, dec!(15000));
        assert_eq!(summary.net_balance(), dec!(15000));
    }
}
