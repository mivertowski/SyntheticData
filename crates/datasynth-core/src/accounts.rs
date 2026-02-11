//! Centralized GL account constants for consistent account mapping.
//!
//! This module provides standard account numbers used across all generators
//! to ensure consistency between document flow JE generation, subledger
//! generation, and reconciliation.

/// Control accounts for subledger integration.
pub mod control_accounts {
    /// Accounts Receivable control account
    pub const AR_CONTROL: &str = "1100";

    /// Accounts Payable control account
    pub const AP_CONTROL: &str = "2000";

    /// Inventory control account
    pub const INVENTORY: &str = "1200";

    /// Fixed Assets control account
    pub const FIXED_ASSETS: &str = "1500";

    /// Accumulated Depreciation control account
    pub const ACCUMULATED_DEPRECIATION: &str = "1510";

    /// GR/IR Clearing account (Goods Receipt/Invoice Receipt)
    pub const GR_IR_CLEARING: &str = "2900";

    /// Intercompany AR clearing
    pub const IC_AR_CLEARING: &str = "1150";

    /// Intercompany AP clearing
    pub const IC_AP_CLEARING: &str = "2050";
}

/// Cash and bank accounts.
pub mod cash_accounts {
    /// Primary operating cash account
    pub const OPERATING_CASH: &str = "1000";

    /// Primary bank account
    pub const BANK_ACCOUNT: &str = "1010";

    /// Petty cash account
    pub const PETTY_CASH: &str = "1020";

    /// Wire transfer clearing account
    pub const WIRE_CLEARING: &str = "1030";
}

/// Revenue accounts.
pub mod revenue_accounts {
    /// Product revenue account
    pub const PRODUCT_REVENUE: &str = "4000";

    /// Service revenue account
    pub const SERVICE_REVENUE: &str = "4100";

    /// Intercompany revenue account
    pub const IC_REVENUE: &str = "4500";

    /// Other revenue account
    pub const OTHER_REVENUE: &str = "4900";

    /// Sales discounts account
    pub const SALES_DISCOUNTS: &str = "4010";

    /// Sales returns and allowances account
    pub const SALES_RETURNS: &str = "4020";
}

/// Expense accounts.
pub mod expense_accounts {
    /// Cost of Goods Sold account
    pub const COGS: &str = "5000";

    /// Raw materials expense account
    pub const RAW_MATERIALS: &str = "5100";

    /// Direct labor expense account
    pub const DIRECT_LABOR: &str = "5200";

    /// Manufacturing overhead account
    pub const MANUFACTURING_OVERHEAD: &str = "5300";

    /// Depreciation expense account
    pub const DEPRECIATION: &str = "6000";

    /// Salaries and wages expense account
    pub const SALARIES_WAGES: &str = "6100";

    /// Benefits expense account
    pub const BENEFITS: &str = "6200";

    /// Rent expense account
    pub const RENT: &str = "6300";

    /// Utilities expense account
    pub const UTILITIES: &str = "6400";

    /// Office supplies expense account
    pub const OFFICE_SUPPLIES: &str = "6500";

    /// Travel and entertainment expense account
    pub const TRAVEL_ENTERTAINMENT: &str = "6600";

    /// Professional fees expense account
    pub const PROFESSIONAL_FEES: &str = "6700";

    /// Insurance expense account
    pub const INSURANCE: &str = "6800";

    /// Bad debt expense account
    pub const BAD_DEBT: &str = "6900";

    /// Interest expense account
    pub const INTEREST_EXPENSE: &str = "7100";

    /// Purchase discounts account
    pub const PURCHASE_DISCOUNTS: &str = "7400";

    /// FX gain/loss account
    pub const FX_GAIN_LOSS: &str = "7500";
}

/// Tax accounts.
pub mod tax_accounts {
    /// Sales tax payable account
    pub const SALES_TAX_PAYABLE: &str = "2100";

    /// VAT payable account
    pub const VAT_PAYABLE: &str = "2110";

    /// Withholding tax payable account
    pub const WITHHOLDING_TAX_PAYABLE: &str = "2120";

    /// Input VAT (VAT receivable) account
    pub const INPUT_VAT: &str = "1160";

    /// Tax expense account
    pub const TAX_EXPENSE: &str = "8000";

    /// Deferred tax liability account
    pub const DEFERRED_TAX_LIABILITY: &str = "2500";

    /// Deferred tax asset account
    pub const DEFERRED_TAX_ASSET: &str = "1600";
}

/// Liability accounts.
pub mod liability_accounts {
    /// Accrued expenses account
    pub const ACCRUED_EXPENSES: &str = "2200";

    /// Accrued salaries account
    pub const ACCRUED_SALARIES: &str = "2210";

    /// Accrued benefits account
    pub const ACCRUED_BENEFITS: &str = "2220";

    /// Unearned revenue account
    pub const UNEARNED_REVENUE: &str = "2300";

    /// Short-term debt account
    pub const SHORT_TERM_DEBT: &str = "2400";

    /// Long-term debt account
    pub const LONG_TERM_DEBT: &str = "2600";

    /// Intercompany payable account
    pub const IC_PAYABLE: &str = "2700";
}

/// Equity accounts.
pub mod equity_accounts {
    /// Common stock account
    pub const COMMON_STOCK: &str = "3000";

    /// Additional paid-in capital account
    pub const APIC: &str = "3100";

    /// Retained earnings account
    pub const RETAINED_EARNINGS: &str = "3200";

    /// Current year earnings account
    pub const CURRENT_YEAR_EARNINGS: &str = "3300";

    /// Treasury stock account
    pub const TREASURY_STOCK: &str = "3400";

    /// Currency translation adjustment account
    pub const CTA: &str = "3500";
}

/// Suspense and clearing accounts.
pub mod suspense_accounts {
    /// General suspense account
    pub const GENERAL_SUSPENSE: &str = "9000";

    /// Payroll clearing account
    pub const PAYROLL_CLEARING: &str = "9100";

    /// Bank reconciliation suspense account
    pub const BANK_RECONCILIATION_SUSPENSE: &str = "9200";

    /// IC elimination suspense account
    pub const IC_ELIMINATION_SUSPENSE: &str = "9300";
}

/// Account type by prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountCategory {
    /// Assets (1xxx)
    Asset,
    /// Liabilities (2xxx)
    Liability,
    /// Equity (3xxx)
    Equity,
    /// Revenue (4xxx)
    Revenue,
    /// Cost of Goods Sold (5xxx)
    Cogs,
    /// Operating Expenses (6xxx)
    OperatingExpense,
    /// Other Income/Expense (7xxx)
    OtherIncomeExpense,
    /// Taxes (8xxx)
    Tax,
    /// Suspense/Clearing (9xxx)
    Suspense,
    /// Unknown
    Unknown,
}

impl AccountCategory {
    /// Determine account category from account number.
    pub fn from_account(account: &str) -> Self {
        if account.is_empty() {
            return Self::Unknown;
        }

        match account.chars().next() {
            Some('1') => Self::Asset,
            Some('2') => Self::Liability,
            Some('3') => Self::Equity,
            Some('4') => Self::Revenue,
            Some('5') => Self::Cogs,
            Some('6') => Self::OperatingExpense,
            Some('7') => Self::OtherIncomeExpense,
            Some('8') => Self::Tax,
            Some('9') => Self::Suspense,
            _ => Self::Unknown,
        }
    }

    /// Check if this category is a debit-normal account.
    pub fn is_debit_normal(&self) -> bool {
        matches!(
            self,
            Self::Asset
                | Self::Cogs
                | Self::OperatingExpense
                | Self::OtherIncomeExpense
                | Self::Tax
        )
    }

    /// Check if this category is a credit-normal account.
    pub fn is_credit_normal(&self) -> bool {
        matches!(self, Self::Liability | Self::Equity | Self::Revenue)
    }

    /// Check if this category is a balance sheet account.
    pub fn is_balance_sheet(&self) -> bool {
        matches!(self, Self::Asset | Self::Liability | Self::Equity)
    }

    /// Check if this category is an income statement account.
    pub fn is_income_statement(&self) -> bool {
        matches!(
            self,
            Self::Revenue
                | Self::Cogs
                | Self::OperatingExpense
                | Self::OtherIncomeExpense
                | Self::Tax
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_account_category_from_account() {
        assert_eq!(
            AccountCategory::from_account(control_accounts::AR_CONTROL),
            AccountCategory::Asset
        );
        assert_eq!(
            AccountCategory::from_account(control_accounts::AP_CONTROL),
            AccountCategory::Liability
        );
        assert_eq!(
            AccountCategory::from_account(equity_accounts::RETAINED_EARNINGS),
            AccountCategory::Equity
        );
        assert_eq!(
            AccountCategory::from_account(revenue_accounts::PRODUCT_REVENUE),
            AccountCategory::Revenue
        );
        assert_eq!(
            AccountCategory::from_account(expense_accounts::COGS),
            AccountCategory::Cogs
        );
    }

    #[test]
    fn test_debit_credit_normal() {
        assert!(AccountCategory::Asset.is_debit_normal());
        assert!(AccountCategory::Revenue.is_credit_normal());
        assert!(!AccountCategory::Asset.is_credit_normal());
        assert!(!AccountCategory::Revenue.is_debit_normal());
    }

    #[test]
    fn test_balance_sheet_vs_income_statement() {
        assert!(AccountCategory::Asset.is_balance_sheet());
        assert!(AccountCategory::Liability.is_balance_sheet());
        assert!(AccountCategory::Equity.is_balance_sheet());
        assert!(!AccountCategory::Revenue.is_balance_sheet());

        assert!(AccountCategory::Revenue.is_income_statement());
        assert!(AccountCategory::Cogs.is_income_statement());
        assert!(!AccountCategory::Asset.is_income_statement());
    }
}
