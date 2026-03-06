//! Centralized framework-aware account mapping.
//!
//! `FrameworkAccounts` maps ~45 semantic account purposes to framework-specific
//! GL account codes. Adding a new accounting framework requires writing one
//! constructor and one constants module — all downstream consumers use the
//! same struct.

use std::sync::Arc;

use crate::accounts::AccountCategory;
use crate::models::balance::AccountCategory as TrialBalanceCategory;
use crate::models::balance::AccountType;

/// Audit export configuration flags.
#[derive(Debug, Clone, Default)]
pub struct AuditExportConfig {
    /// French FEC (Fichier des Écritures Comptables), Art. A47 A-1 LPF.
    pub fec_enabled: bool,
    /// German GoBD (Grundsätze zur ordnungsmäßigen Führung und Aufbewahrung
    /// von Büchern, Aufzeichnungen und Unterlagen in elektronischer Form).
    pub gobd_enabled: bool,
}

/// Maps semantic account purposes to framework-specific GL codes.
///
/// Downstream generators use field names (`ar_control`, `cogs`, …) instead of
/// hard-coded account numbers, making them framework-agnostic.
#[derive(Clone)]
pub struct FrameworkAccounts {
    // ── Control ──────────────────────────────────────────────────────
    pub ar_control: String,
    pub ap_control: String,
    pub inventory: String,
    pub fixed_assets: String,
    pub accumulated_depreciation: String,
    pub gr_ir_clearing: String,
    pub ic_ar_clearing: String,
    pub ic_ap_clearing: String,

    // ── Cash ─────────────────────────────────────────────────────────
    pub operating_cash: String,
    pub bank_account: String,
    pub petty_cash: String,

    // ── Revenue ──────────────────────────────────────────────────────
    pub product_revenue: String,
    pub service_revenue: String,
    pub ic_revenue: String,
    pub purchase_discount_income: String,
    pub other_revenue: String,
    pub sales_discounts: String,
    pub sales_returns: String,

    // ── Expense ──────────────────────────────────────────────────────
    pub cogs: String,
    pub raw_materials: String,
    pub depreciation_expense: String,
    pub salaries_wages: String,
    pub rent: String,
    pub interest_expense: String,
    pub purchase_discounts: String,
    pub fx_gain_loss: String,
    pub bad_debt: String,

    // ── Tax ──────────────────────────────────────────────────────────
    pub sales_tax_payable: String,
    pub vat_payable: String,
    pub input_vat: String,
    pub tax_receivable: String,
    pub tax_expense: String,
    pub deferred_tax_liability: String,
    pub deferred_tax_asset: String,

    // ── Liability ────────────────────────────────────────────────────
    pub accrued_expenses: String,
    pub accrued_salaries: String,
    pub unearned_revenue: String,
    pub short_term_debt: String,
    pub long_term_debt: String,
    pub ic_payable: String,

    // ── Equity ───────────────────────────────────────────────────────
    pub common_stock: String,
    pub retained_earnings: String,
    pub current_year_earnings: String,
    pub cta: String,
    pub income_summary: String,
    pub dividends_paid: String,

    // ── Suspense / Clearing ──────────────────────────────────────────
    pub general_suspense: String,
    pub payroll_clearing: String,
    pub bank_reconciliation_suspense: String,

    // ── HGB-specific ─────────────────────────────────────────────────
    pub provisions: String,

    // ── Audit export flags ───────────────────────────────────────────
    pub audit_export: AuditExportConfig,

    // ── Classification function ──────────────────────────────────────
    classifier: Arc<dyn Fn(&str) -> AccountCategory + Send + Sync>,
}

impl std::fmt::Debug for FrameworkAccounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameworkAccounts")
            .field("ar_control", &self.ar_control)
            .field("ap_control", &self.ap_control)
            .field("audit_export", &self.audit_export)
            .finish_non_exhaustive()
    }
}

impl FrameworkAccounts {
    /// Classify an account number using this framework's rules.
    pub fn classify(&self, account: &str) -> AccountCategory {
        (self.classifier)(account)
    }

    /// Classify an account code into an [`AccountType`] using this framework's rules.
    ///
    /// Maps the framework-specific [`AccountCategory`] to the balance-sheet
    /// oriented [`AccountType`] (Asset, Liability, Equity, Revenue, Expense).
    pub fn classify_account_type(&self, account_code: &str) -> AccountType {
        account_type_from_category(self.classify(account_code))
    }

    /// Classify an account code into a [`TrialBalanceCategory`] using this
    /// framework's rules.
    ///
    /// Provides the finer-grained trial-balance grouping (CurrentAssets,
    /// NonCurrentAssets, etc.) derived from the framework classifier.
    pub fn classify_trial_balance_category(&self, account_code: &str) -> TrialBalanceCategory {
        trial_balance_category_from_category(self.classify(account_code))
    }

    /// US GAAP (default) — 4-digit accounts from `crate::accounts`.
    pub fn us_gaap() -> Self {
        use crate::accounts::*;
        Self {
            ar_control: control_accounts::AR_CONTROL.into(),
            ap_control: control_accounts::AP_CONTROL.into(),
            inventory: control_accounts::INVENTORY.into(),
            fixed_assets: control_accounts::FIXED_ASSETS.into(),
            accumulated_depreciation: control_accounts::ACCUMULATED_DEPRECIATION.into(),
            gr_ir_clearing: control_accounts::GR_IR_CLEARING.into(),
            ic_ar_clearing: control_accounts::IC_AR_CLEARING.into(),
            ic_ap_clearing: control_accounts::IC_AP_CLEARING.into(),

            operating_cash: cash_accounts::OPERATING_CASH.into(),
            bank_account: cash_accounts::BANK_ACCOUNT.into(),
            petty_cash: cash_accounts::PETTY_CASH.into(),

            product_revenue: revenue_accounts::PRODUCT_REVENUE.into(),
            service_revenue: revenue_accounts::SERVICE_REVENUE.into(),
            ic_revenue: revenue_accounts::IC_REVENUE.into(),
            purchase_discount_income: revenue_accounts::PURCHASE_DISCOUNT_INCOME.into(),
            other_revenue: revenue_accounts::OTHER_REVENUE.into(),
            sales_discounts: revenue_accounts::SALES_DISCOUNTS.into(),
            sales_returns: revenue_accounts::SALES_RETURNS.into(),

            cogs: expense_accounts::COGS.into(),
            raw_materials: expense_accounts::RAW_MATERIALS.into(),
            depreciation_expense: expense_accounts::DEPRECIATION.into(),
            salaries_wages: expense_accounts::SALARIES_WAGES.into(),
            rent: expense_accounts::RENT.into(),
            interest_expense: expense_accounts::INTEREST_EXPENSE.into(),
            purchase_discounts: expense_accounts::PURCHASE_DISCOUNTS.into(),
            fx_gain_loss: expense_accounts::FX_GAIN_LOSS.into(),
            bad_debt: expense_accounts::BAD_DEBT.into(),

            sales_tax_payable: tax_accounts::SALES_TAX_PAYABLE.into(),
            vat_payable: tax_accounts::VAT_PAYABLE.into(),
            input_vat: tax_accounts::INPUT_VAT.into(),
            tax_receivable: tax_accounts::TAX_RECEIVABLE.into(),
            tax_expense: tax_accounts::TAX_EXPENSE.into(),
            deferred_tax_liability: tax_accounts::DEFERRED_TAX_LIABILITY.into(),
            deferred_tax_asset: tax_accounts::DEFERRED_TAX_ASSET.into(),

            accrued_expenses: liability_accounts::ACCRUED_EXPENSES.into(),
            accrued_salaries: liability_accounts::ACCRUED_SALARIES.into(),
            unearned_revenue: liability_accounts::UNEARNED_REVENUE.into(),
            short_term_debt: liability_accounts::SHORT_TERM_DEBT.into(),
            long_term_debt: liability_accounts::LONG_TERM_DEBT.into(),
            ic_payable: liability_accounts::IC_PAYABLE.into(),

            common_stock: equity_accounts::COMMON_STOCK.into(),
            retained_earnings: equity_accounts::RETAINED_EARNINGS.into(),
            current_year_earnings: equity_accounts::CURRENT_YEAR_EARNINGS.into(),
            cta: equity_accounts::CTA.into(),
            income_summary: equity_accounts::INCOME_SUMMARY.into(),
            dividends_paid: equity_accounts::DIVIDENDS_PAID.into(),

            general_suspense: suspense_accounts::GENERAL_SUSPENSE.into(),
            payroll_clearing: suspense_accounts::PAYROLL_CLEARING.into(),
            bank_reconciliation_suspense: suspense_accounts::BANK_RECONCILIATION_SUSPENSE.into(),

            provisions: liability_accounts::ACCRUED_EXPENSES.into(), // US GAAP: no separate provisions class

            audit_export: AuditExportConfig::default(),
            classifier: Arc::new(us_gaap_classify),
        }
    }

    /// IFRS — uses the same numbering conventions as US GAAP.
    pub fn ifrs() -> Self {
        Self::us_gaap()
    }

    /// French GAAP (PCG) — 6-digit accounts from `crate::pcg`.
    pub fn french_gaap() -> Self {
        use crate::pcg::*;
        Self {
            ar_control: control_accounts::AR_CONTROL.into(),
            ap_control: control_accounts::AP_CONTROL.into(),
            inventory: control_accounts::INVENTORY.into(),
            fixed_assets: control_accounts::FIXED_ASSETS.into(),
            accumulated_depreciation: control_accounts::ACCUMULATED_DEPRECIATION.into(),
            gr_ir_clearing: control_accounts::GR_IR_CLEARING.into(),
            ic_ar_clearing: control_accounts::IC_AR_CLEARING.into(),
            ic_ap_clearing: control_accounts::IC_AP_CLEARING.into(),

            operating_cash: cash_accounts::OPERATING_CASH.into(),
            bank_account: cash_accounts::BANK_ACCOUNT.into(),
            petty_cash: cash_accounts::PETTY_CASH.into(),

            product_revenue: revenue_accounts::PRODUCT_REVENUE.into(),
            service_revenue: revenue_accounts::SERVICE_REVENUE.into(),
            ic_revenue: additional_revenue::IC_REVENUE.into(),
            purchase_discount_income: additional_revenue::PURCHASE_DISCOUNT_INCOME.into(),
            other_revenue: revenue_accounts::OTHER_REVENUE.into(),
            sales_discounts: revenue_accounts::SALES_DISCOUNTS.into(),
            sales_returns: additional_revenue::SALES_RETURNS.into(),

            cogs: expense_accounts::COGS.into(),
            raw_materials: additional_expense::RAW_MATERIALS.into(),
            depreciation_expense: expense_accounts::DEPRECIATION.into(),
            salaries_wages: expense_accounts::SALARIES_WAGES.into(),
            rent: expense_accounts::RENT.into(),
            interest_expense: expense_accounts::INTEREST_EXPENSE.into(),
            purchase_discounts: additional_expense::PURCHASE_DISCOUNTS.into(),
            fx_gain_loss: additional_expense::FX_GAIN_LOSS.into(),
            bad_debt: additional_expense::BAD_DEBT.into(),

            sales_tax_payable: tax_accounts::OUTPUT_VAT.into(),
            vat_payable: tax_accounts::OUTPUT_VAT.into(),
            input_vat: tax_accounts::INPUT_VAT.into(),
            tax_receivable: tax_accounts::TAX_RECEIVABLE.into(),
            tax_expense: tax_accounts::TAX_EXPENSE.into(),
            deferred_tax_liability: tax_accounts::DEFERRED_TAX_LIABILITY.into(),
            deferred_tax_asset: tax_accounts::DEFERRED_TAX_ASSET.into(),

            accrued_expenses: liability_accounts::ACCRUED_EXPENSES.into(),
            accrued_salaries: liability_accounts::ACCRUED_SALARIES.into(),
            unearned_revenue: liability_accounts::UNEARNED_REVENUE.into(),
            short_term_debt: equity_liability_accounts::SHORT_TERM_DEBT.into(),
            long_term_debt: equity_liability_accounts::LONG_TERM_DEBT.into(),
            ic_payable: liability_accounts::IC_PAYABLE.into(),

            common_stock: equity_liability_accounts::COMMON_STOCK.into(),
            retained_earnings: equity_liability_accounts::RETAINED_EARNINGS.into(),
            current_year_earnings: equity_accounts::CURRENT_YEAR_EARNINGS.into(),
            cta: equity_accounts::CTA.into(),
            income_summary: equity_accounts::INCOME_SUMMARY.into(),
            dividends_paid: equity_accounts::DIVIDENDS_PAID.into(),

            general_suspense: suspense_accounts::GENERAL_SUSPENSE.into(),
            payroll_clearing: suspense_accounts::PAYROLL_CLEARING.into(),
            bank_reconciliation_suspense: suspense_accounts::GENERAL_SUSPENSE.into(), // PCG uses 471000 for general suspense

            provisions: equity_liability_accounts::PROVISIONS.into(),

            audit_export: AuditExportConfig {
                fec_enabled: true,
                gobd_enabled: false,
            },
            classifier: Arc::new(pcg_classify),
        }
    }

    /// German GAAP (HGB) — 4-digit SKR04 accounts from `crate::skr`.
    pub fn german_gaap() -> Self {
        use crate::skr::*;
        Self {
            ar_control: control_accounts::AR_CONTROL.into(),
            ap_control: control_accounts::AP_CONTROL.into(),
            inventory: control_accounts::INVENTORY.into(),
            fixed_assets: control_accounts::FIXED_ASSETS.into(),
            accumulated_depreciation: control_accounts::ACCUMULATED_DEPRECIATION.into(),
            gr_ir_clearing: control_accounts::GR_IR_CLEARING.into(),
            ic_ar_clearing: control_accounts::IC_AR_CLEARING.into(),
            ic_ap_clearing: control_accounts::IC_AP_CLEARING.into(),

            operating_cash: cash_accounts::OPERATING_CASH.into(),
            bank_account: cash_accounts::BANK_ACCOUNT.into(),
            petty_cash: cash_accounts::PETTY_CASH.into(),

            product_revenue: revenue_accounts::PRODUCT_REVENUE.into(),
            service_revenue: revenue_accounts::SERVICE_REVENUE.into(),
            ic_revenue: revenue_accounts::IC_REVENUE.into(),
            purchase_discount_income: revenue_accounts::PURCHASE_DISCOUNT_INCOME.into(),
            other_revenue: revenue_accounts::OTHER_REVENUE.into(),
            sales_discounts: revenue_accounts::SALES_DISCOUNTS.into(),
            sales_returns: revenue_accounts::SALES_RETURNS.into(),

            cogs: expense_accounts::COGS.into(),
            raw_materials: expense_accounts::RAW_MATERIALS.into(),
            depreciation_expense: expense_accounts::DEPRECIATION.into(),
            salaries_wages: expense_accounts::SALARIES_WAGES.into(),
            rent: expense_accounts::RENT.into(),
            interest_expense: expense_accounts::INTEREST_EXPENSE.into(),
            purchase_discounts: expense_accounts::PURCHASE_DISCOUNTS.into(),
            fx_gain_loss: expense_accounts::FX_GAIN_LOSS.into(),
            bad_debt: expense_accounts::BAD_DEBT.into(),

            sales_tax_payable: tax_accounts::OUTPUT_VAT.into(),
            vat_payable: tax_accounts::OUTPUT_VAT.into(),
            input_vat: tax_accounts::INPUT_VAT.into(),
            tax_receivable: tax_accounts::TAX_RECEIVABLE.into(),
            tax_expense: tax_accounts::TAX_EXPENSE.into(),
            deferred_tax_liability: tax_accounts::DEFERRED_TAX_LIABILITY.into(),
            deferred_tax_asset: tax_accounts::DEFERRED_TAX_ASSET.into(),

            accrued_expenses: liability_accounts::ACCRUED_EXPENSES.into(),
            accrued_salaries: liability_accounts::ACCRUED_SALARIES.into(),
            unearned_revenue: liability_accounts::UNEARNED_REVENUE.into(),
            short_term_debt: liability_accounts::SHORT_TERM_DEBT.into(),
            long_term_debt: liability_accounts::LONG_TERM_DEBT.into(),
            ic_payable: liability_accounts::IC_PAYABLE.into(),

            common_stock: equity_accounts::COMMON_STOCK.into(),
            retained_earnings: equity_accounts::RETAINED_EARNINGS.into(),
            current_year_earnings: equity_accounts::CURRENT_YEAR_EARNINGS.into(),
            cta: equity_accounts::CTA.into(),
            income_summary: equity_accounts::INCOME_SUMMARY.into(),
            dividends_paid: equity_accounts::DIVIDENDS_PAID.into(),

            general_suspense: suspense_accounts::GENERAL_SUSPENSE.into(),
            payroll_clearing: suspense_accounts::PAYROLL_CLEARING.into(),
            bank_reconciliation_suspense: suspense_accounts::BANK_RECONCILIATION_SUSPENSE.into(),

            provisions: equity_accounts::PROVISIONS.into(),

            audit_export: AuditExportConfig {
                fec_enabled: false,
                gobd_enabled: true,
            },
            classifier: Arc::new(skr04_classify),
        }
    }

    /// Select accounts for a given `AccountingFramework` from `datasynth-standards`.
    ///
    /// This is the primary entry point — other code should not hard-code
    /// framework detection logic.
    pub fn for_framework(framework: &str) -> Self {
        match framework {
            "us_gaap" | "UsGaap" => Self::us_gaap(),
            "ifrs" | "Ifrs" => Self::ifrs(),
            "dual_reporting" | "DualReporting" => Self::us_gaap(),
            "french_gaap" | "FrenchGaap" => Self::french_gaap(),
            "german_gaap" | "GermanGaap" | "hgb" => Self::german_gaap(),
            other => {
                eprintln!(
                    "FrameworkAccounts::for_framework: unknown framework {other:?}, defaulting to US GAAP"
                );
                Self::us_gaap()
            }
        }
    }
}

// ── Conversion helpers ──────────────────────────────────────────────────

/// Convert a framework [`AccountCategory`] to an [`AccountType`].
fn account_type_from_category(cat: AccountCategory) -> AccountType {
    match cat {
        AccountCategory::Asset => AccountType::Asset,
        AccountCategory::Liability => AccountType::Liability,
        AccountCategory::Equity => AccountType::Equity,
        AccountCategory::Revenue => AccountType::Revenue,
        AccountCategory::Cogs
        | AccountCategory::OperatingExpense
        | AccountCategory::OtherIncomeExpense
        | AccountCategory::Tax => AccountType::Expense,
        AccountCategory::Suspense | AccountCategory::Unknown => AccountType::Asset,
    }
}

/// Convert a framework [`AccountCategory`] to a [`TrialBalanceCategory`].
fn trial_balance_category_from_category(cat: AccountCategory) -> TrialBalanceCategory {
    match cat {
        AccountCategory::Asset => TrialBalanceCategory::CurrentAssets,
        AccountCategory::Liability => TrialBalanceCategory::CurrentLiabilities,
        AccountCategory::Equity => TrialBalanceCategory::Equity,
        AccountCategory::Revenue => TrialBalanceCategory::Revenue,
        AccountCategory::Cogs => TrialBalanceCategory::CostOfGoodsSold,
        AccountCategory::OperatingExpense => TrialBalanceCategory::OperatingExpenses,
        AccountCategory::OtherIncomeExpense => TrialBalanceCategory::OtherExpenses,
        AccountCategory::Tax => TrialBalanceCategory::OtherExpenses,
        AccountCategory::Suspense | AccountCategory::Unknown => TrialBalanceCategory::OtherExpenses,
    }
}

// ── Classification functions ─────────────────────────────────────────────

/// US GAAP: first digit maps directly to category.
///   1=Asset, 2=Liability, 3=Equity, 4=Revenue, 5=COGS,
///   6=OpEx, 7=Other, 8=Tax, 9=Suspense
fn us_gaap_classify(account: &str) -> AccountCategory {
    AccountCategory::from_account(account)
}

/// French PCG: classes 1-9 have different semantics.
///   1=Equity/Liabilities, 2=Fixed Assets, 3=Inventory,
///   4=Mixed (AP/AR/Personnel), 5=Cash, 6=Expense, 7=Revenue,
///   8=Special, 9=Analytical
fn pcg_classify(account: &str) -> AccountCategory {
    match account.chars().next().and_then(|c| c.to_digit(10)) {
        Some(1) => {
            // Class 1: Equity & liabilities — check subclass
            let sub = account.get(1..2).and_then(|s| s.parse::<u8>().ok());
            match sub {
                Some(0..=4) => AccountCategory::Equity, // 10x-14x: capital & reserves
                Some(5) => AccountCategory::Liability,  // 15x: provisions
                Some(6..=7) => AccountCategory::Liability, // 16x-17x: debts
                _ => AccountCategory::Liability,
            }
        }
        Some(2) => AccountCategory::Asset, // Fixed assets
        Some(3) => AccountCategory::Asset, // Inventory
        Some(4) => {
            // Class 4: third parties — check subclass
            let sub = account.get(1..2).and_then(|s| s.parse::<u8>().ok());
            match sub {
                Some(0) => AccountCategory::Liability,     // 40x: suppliers
                Some(1) => AccountCategory::Asset,         // 41x: customers
                Some(2) => AccountCategory::Liability,     // 42x: personnel
                Some(3..=4) => AccountCategory::Liability, // 43-44x: social/tax
                Some(5) => AccountCategory::Liability,     // 45x: group companies
                _ => AccountCategory::Liability,
            }
        }
        Some(5) => AccountCategory::Asset, // Cash / financial
        Some(6) => AccountCategory::OperatingExpense, // Expenses
        Some(7) => AccountCategory::Revenue, // Revenue
        Some(8) => AccountCategory::Suspense, // Special
        Some(9) => AccountCategory::Suspense, // Analytical
        _ => AccountCategory::Unknown,
    }
}

/// German SKR04: classes 0-9 follow the Abschlussgliederungsprinzip.
///   0=Fixed Assets, 1=Current Assets, 2=Equity, 3=Liabilities,
///   4=Revenue, 5=COGS/Material, 6=OpEx, 7=Financial, 8=Tax/Extra,
///   9=Statistical
fn skr04_classify(account: &str) -> AccountCategory {
    match account.chars().next().and_then(|c| c.to_digit(10)) {
        Some(0) => AccountCategory::Asset,              // Fixed assets
        Some(1) => AccountCategory::Asset,              // Current assets
        Some(2) => AccountCategory::Equity,             // Equity
        Some(3) => AccountCategory::Liability,          // Liabilities
        Some(4) => AccountCategory::Revenue,            // Revenue
        Some(5) => AccountCategory::Cogs,               // Material / COGS
        Some(6) => AccountCategory::OperatingExpense,   // Personnel & other OpEx
        Some(7) => AccountCategory::OtherIncomeExpense, // Financial income/expense
        Some(8) => AccountCategory::Tax,                // Extraordinary / Tax
        Some(9) => AccountCategory::Suspense,           // Statistical
        _ => AccountCategory::Unknown,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_accounts_us_gaap_defaults() {
        use crate::accounts::*;
        let fa = FrameworkAccounts::us_gaap();
        assert_eq!(fa.ar_control, control_accounts::AR_CONTROL);
        assert_eq!(fa.ap_control, control_accounts::AP_CONTROL);
        assert_eq!(fa.inventory, control_accounts::INVENTORY);
        assert_eq!(fa.cogs, expense_accounts::COGS);
        assert_eq!(fa.product_revenue, revenue_accounts::PRODUCT_REVENUE);
        assert_eq!(fa.retained_earnings, equity_accounts::RETAINED_EARNINGS);
        assert_eq!(fa.general_suspense, suspense_accounts::GENERAL_SUSPENSE);
        assert!(!fa.audit_export.fec_enabled);
        assert!(!fa.audit_export.gobd_enabled);
    }

    #[test]
    fn test_framework_accounts_french_gaap() {
        use crate::pcg;
        let fa = FrameworkAccounts::french_gaap();
        assert_eq!(fa.ar_control, pcg::control_accounts::AR_CONTROL);
        assert_eq!(fa.ap_control, pcg::control_accounts::AP_CONTROL);
        assert_eq!(fa.inventory, pcg::control_accounts::INVENTORY);
        assert_eq!(fa.cogs, pcg::expense_accounts::COGS);
        assert_eq!(fa.product_revenue, pcg::revenue_accounts::PRODUCT_REVENUE);
        assert!(fa.audit_export.fec_enabled);
        assert!(!fa.audit_export.gobd_enabled);
    }

    #[test]
    fn test_framework_accounts_german_gaap() {
        use crate::skr;
        let fa = FrameworkAccounts::german_gaap();
        assert_eq!(fa.ar_control, skr::control_accounts::AR_CONTROL);
        assert_eq!(fa.ap_control, skr::control_accounts::AP_CONTROL);
        assert_eq!(fa.cogs, skr::expense_accounts::COGS);
        assert!(!fa.audit_export.fec_enabled);
        assert!(fa.audit_export.gobd_enabled);
    }

    #[test]
    fn test_classify_us_gaap() {
        let fa = FrameworkAccounts::us_gaap();
        assert_eq!(fa.classify("1100"), AccountCategory::Asset);
        assert_eq!(fa.classify("2000"), AccountCategory::Liability);
        assert_eq!(fa.classify("3200"), AccountCategory::Equity);
        assert_eq!(fa.classify("4000"), AccountCategory::Revenue);
        assert_eq!(fa.classify("5000"), AccountCategory::Cogs);
        assert_eq!(fa.classify("6100"), AccountCategory::OperatingExpense);
        assert_eq!(fa.classify("9000"), AccountCategory::Suspense);
    }

    #[test]
    fn test_classify_pcg() {
        let fa = FrameworkAccounts::french_gaap();
        assert_eq!(fa.classify("101000"), AccountCategory::Equity);
        assert_eq!(fa.classify("151000"), AccountCategory::Liability);
        assert_eq!(fa.classify("210000"), AccountCategory::Asset);
        assert_eq!(fa.classify("310000"), AccountCategory::Asset);
        assert_eq!(fa.classify("401000"), AccountCategory::Liability);
        assert_eq!(fa.classify("411000"), AccountCategory::Asset);
        assert_eq!(fa.classify("512000"), AccountCategory::Asset);
        assert_eq!(fa.classify("603000"), AccountCategory::OperatingExpense);
        assert_eq!(fa.classify("701000"), AccountCategory::Revenue);
    }

    #[test]
    fn test_classify_skr04() {
        let fa = FrameworkAccounts::german_gaap();
        assert_eq!(fa.classify("0200"), AccountCategory::Asset);
        assert_eq!(fa.classify("1200"), AccountCategory::Asset);
        assert_eq!(fa.classify("2000"), AccountCategory::Equity);
        assert_eq!(fa.classify("3300"), AccountCategory::Liability);
        assert_eq!(fa.classify("4000"), AccountCategory::Revenue);
        assert_eq!(fa.classify("5000"), AccountCategory::Cogs);
        assert_eq!(fa.classify("6000"), AccountCategory::OperatingExpense);
        assert_eq!(fa.classify("7300"), AccountCategory::OtherIncomeExpense);
    }

    #[test]
    fn test_for_framework_dispatch() {
        let us = FrameworkAccounts::for_framework("us_gaap");
        assert_eq!(us.ar_control, "1100");

        let fr = FrameworkAccounts::for_framework("french_gaap");
        assert_eq!(fr.ar_control, "411000");

        let de = FrameworkAccounts::for_framework("german_gaap");
        assert_eq!(de.ar_control, "1200");

        let hgb = FrameworkAccounts::for_framework("hgb");
        assert_eq!(hgb.ar_control, "1200");

        // IFRS and dual_reporting dispatch
        let ifrs = FrameworkAccounts::for_framework("ifrs");
        assert_eq!(ifrs.ar_control, "1100");

        let dual = FrameworkAccounts::for_framework("dual_reporting");
        assert_eq!(dual.ar_control, "1100");

        // Unknown framework falls back to US GAAP (with eprintln warning)
        let unknown = FrameworkAccounts::for_framework("martian_gaap");
        assert_eq!(unknown.ar_control, "1100");
    }

    #[test]
    fn test_ifrs_constructor() {
        let ifrs = FrameworkAccounts::ifrs();
        let us = FrameworkAccounts::us_gaap();
        assert_eq!(ifrs.ar_control, us.ar_control);
        assert_eq!(ifrs.ap_control, us.ap_control);
        assert_eq!(ifrs.cogs, us.cogs);
        assert_eq!(ifrs.product_revenue, us.product_revenue);
        assert_eq!(ifrs.classify("1100"), us.classify("1100"));
        assert_eq!(ifrs.classify("4000"), us.classify("4000"));
    }

    #[test]
    fn test_classify_account_type_us_gaap() {
        let fa = FrameworkAccounts::us_gaap();
        assert_eq!(fa.classify_account_type("1100"), AccountType::Asset);
        assert_eq!(fa.classify_account_type("2000"), AccountType::Liability);
        assert_eq!(fa.classify_account_type("3200"), AccountType::Equity);
        assert_eq!(fa.classify_account_type("4000"), AccountType::Revenue);
        assert_eq!(fa.classify_account_type("5000"), AccountType::Expense);
        assert_eq!(fa.classify_account_type("6100"), AccountType::Expense);
    }

    #[test]
    fn test_classify_account_type_french_gaap() {
        let fa = FrameworkAccounts::french_gaap();
        assert_eq!(fa.classify_account_type("101000"), AccountType::Equity);
        assert_eq!(fa.classify_account_type("210000"), AccountType::Asset);
        assert_eq!(fa.classify_account_type("401000"), AccountType::Liability);
        assert_eq!(fa.classify_account_type("603000"), AccountType::Expense);
        assert_eq!(fa.classify_account_type("701000"), AccountType::Revenue);
    }

    #[test]
    fn test_classify_account_type_german_gaap() {
        let fa = FrameworkAccounts::german_gaap();
        assert_eq!(fa.classify_account_type("0200"), AccountType::Asset);
        assert_eq!(fa.classify_account_type("2000"), AccountType::Equity);
        assert_eq!(fa.classify_account_type("3300"), AccountType::Liability);
        assert_eq!(fa.classify_account_type("4000"), AccountType::Revenue);
        assert_eq!(fa.classify_account_type("5000"), AccountType::Expense);
        assert_eq!(fa.classify_account_type("6000"), AccountType::Expense);
    }

    #[test]
    fn test_classify_trial_balance_category_us_gaap() {
        let fa = FrameworkAccounts::us_gaap();
        assert_eq!(
            fa.classify_trial_balance_category("1100"),
            TrialBalanceCategory::CurrentAssets
        );
        assert_eq!(
            fa.classify_trial_balance_category("2000"),
            TrialBalanceCategory::CurrentLiabilities
        );
        assert_eq!(
            fa.classify_trial_balance_category("3200"),
            TrialBalanceCategory::Equity
        );
        assert_eq!(
            fa.classify_trial_balance_category("4000"),
            TrialBalanceCategory::Revenue
        );
        assert_eq!(
            fa.classify_trial_balance_category("5000"),
            TrialBalanceCategory::CostOfGoodsSold
        );
        assert_eq!(
            fa.classify_trial_balance_category("6100"),
            TrialBalanceCategory::OperatingExpenses
        );
    }

    #[test]
    fn test_classify_trial_balance_category_french_gaap() {
        let fa = FrameworkAccounts::french_gaap();
        assert_eq!(
            fa.classify_trial_balance_category("101000"),
            TrialBalanceCategory::Equity
        );
        assert_eq!(
            fa.classify_trial_balance_category("210000"),
            TrialBalanceCategory::CurrentAssets
        );
        assert_eq!(
            fa.classify_trial_balance_category("603000"),
            TrialBalanceCategory::OperatingExpenses
        );
        assert_eq!(
            fa.classify_trial_balance_category("701000"),
            TrialBalanceCategory::Revenue
        );
    }
}
