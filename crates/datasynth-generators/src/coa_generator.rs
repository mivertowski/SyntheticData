//! Chart of Accounts generator.

use tracing::debug;

use datasynth_core::accounts::{
    cash_accounts, control_accounts, equity_accounts, expense_accounts, liability_accounts,
    revenue_accounts, suspense_accounts, tax_accounts,
};
use datasynth_core::models::*;
use datasynth_core::pcg_loader;
use datasynth_core::traits::Generator;
use datasynth_core::utils::seeded_rng;
use rand_chacha::ChaCha8Rng;

/// Generator for Chart of Accounts.
pub struct ChartOfAccountsGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    complexity: CoAComplexity,
    industry: IndustrySector,
    count: u64,
    /// When true, generate Plan Comptable Général (French GAAP) structure.
    use_french_pcg: bool,
}

impl ChartOfAccountsGenerator {
    /// Create a new CoA generator.
    pub fn new(complexity: CoAComplexity, industry: IndustrySector, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            complexity,
            industry,
            count: 0,
            use_french_pcg: false,
        }
    }

    /// Use French GAAP (Plan Comptable Général) account structure.
    pub fn with_french_pcg(mut self, use_pcg: bool) -> Self {
        self.use_french_pcg = use_pcg;
        self
    }

    /// Generate a complete chart of accounts.
    pub fn generate(&mut self) -> ChartOfAccounts {
        debug!(
            complexity = ?self.complexity,
            industry = ?self.industry,
            seed = self.seed,
            "Generating chart of accounts"
        );

        self.count += 1;
        if self.use_french_pcg {
            self.generate_pcg()
        } else {
            self.generate_default()
        }
    }

    /// Generate default (US-style) chart of accounts.
    fn generate_default(&mut self) -> ChartOfAccounts {
        let target_count = self.complexity.target_count();
        let mut coa = ChartOfAccounts::new(
            format!("COA_{:?}_{}", self.industry, self.complexity.target_count()),
            format!("{:?} Chart of Accounts", self.industry),
            "US".to_string(),
            self.industry,
            self.complexity,
        );

        // Seed canonical accounts first so other generators can find them
        Self::seed_canonical_accounts(&mut coa);

        // Generate additional accounts by type
        self.generate_asset_accounts(&mut coa, target_count / 5);
        self.generate_liability_accounts(&mut coa, target_count / 6);
        self.generate_equity_accounts(&mut coa, target_count / 10);
        self.generate_revenue_accounts(&mut coa, target_count / 5);
        self.generate_expense_accounts(&mut coa, target_count / 4);
        self.generate_suspense_accounts(&mut coa);
        coa
    }

    /// Generate Plan Comptable Général (French GAAP) chart of accounts.
    /// Uses the comprehensive PCG 2024 structure from [arrhes/PCG](https://github.com/arrhes/PCG) when available.
    fn generate_pcg(&mut self) -> ChartOfAccounts {
        match pcg_loader::build_chart_of_accounts_from_pcg_2024(self.complexity, self.industry) {
            Ok(coa) => coa,
            Err(_) => self.generate_pcg_fallback(),
        }
    }

    /// Fallback simplified PCG when the embedded 2024 JSON cannot be loaded.
    fn generate_pcg_fallback(&mut self) -> ChartOfAccounts {
        let target_count = self.complexity.target_count();
        let mut coa = ChartOfAccounts::new(
            format!("COA_PCG_{:?}_{}", self.industry, target_count),
            format!("Plan Comptable Général – {:?}", self.industry),
            "FR".to_string(),
            self.industry,
            self.complexity,
        );
        coa.account_format = "######".to_string();

        self.generate_pcg_class_1(&mut coa, target_count / 10);
        self.generate_pcg_class_2(&mut coa, target_count / 6);
        self.generate_pcg_class_3(&mut coa, target_count / 8);
        self.generate_pcg_class_4(&mut coa, target_count / 5);
        self.generate_pcg_class_5(&mut coa, target_count / 12);
        self.generate_pcg_class_6(&mut coa, target_count / 4);
        self.generate_pcg_class_7(&mut coa, target_count / 5);
        self.generate_pcg_class_8(&mut coa);

        coa
    }

    fn generate_pcg_class_1(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let items = [
            (101, "Capital", AccountType::Equity, AccountSubType::CommonStock),
            (129, "Résultat", AccountType::Equity, AccountSubType::RetainedEarnings),
            (164, "Emprunts", AccountType::Liability, AccountSubType::LongTermDebt),
            (421, "Fournisseurs", AccountType::Liability, AccountSubType::AccountsPayable),
        ];
        for (base, name, acc_type, sub_type) in items {
            for i in 0..count.max(1) {
                let num = base * 1000 + (i as u32 % 100);
                coa.add_account(GLAccount::new(
                    format!("{:06}", num),
                    format!("{} {}", name, i + 1),
                    acc_type,
                    sub_type,
                ));
            }
        }
    }

    fn generate_pcg_class_2(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        for i in 0..count.max(1) {
            let num = 215000 + (i as u32 % 100);
            coa.add_account(GLAccount::new(
                format!("{:06}", num),
                format!("Immobilisations {}", i + 1),
                AccountType::Asset,
                AccountSubType::FixedAssets,
            ));
        }
        for i in 0..(count / 2).max(1) {
            let num = 281000 + (i as u32 % 100);
            coa.add_account(GLAccount::new(
                format!("{:06}", num),
                format!("Amortissements {}", i + 1),
                AccountType::Asset,
                AccountSubType::AccumulatedDepreciation,
            ));
        }
    }

    fn generate_pcg_class_3(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        for i in 0..count.max(1) {
            let num = 310000 + (i as u32 % 1000);
            coa.add_account(GLAccount::new(
                format!("{:06}", num),
                format!("Stocks {}", i + 1),
                AccountType::Asset,
                AccountSubType::Inventory,
            ));
        }
    }

    fn generate_pcg_class_4(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        for i in 0..count.max(1) {
            let num = 411000 + (i as u32 % 1000);
            coa.add_account(GLAccount::new(
                format!("{:06}", num),
                format!("Clients {}", i + 1),
                AccountType::Asset,
                AccountSubType::AccountsReceivable,
            ));
        }
        for i in 0..count.max(1) {
            let num = 401000 + (i as u32 % 1000);
            coa.add_account(GLAccount::new(
                format!("{:06}", num),
                format!("Fournisseurs {}", i + 1),
                AccountType::Liability,
                AccountSubType::AccountsPayable,
            ));
        }
        let clearing = GLAccount::new(
            "408000".to_string(),
            "Fournisseurs – non encore reçus".to_string(),
            AccountType::Liability,
            AccountSubType::GoodsReceivedClearing,
        );
        coa.add_account(clearing);
    }

    fn generate_pcg_class_5(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let bases = [(512, "Banque"), (530, "Caisse"), (516, "Chèques")];
        for (base, name) in bases {
            for i in 0..(count / 3).max(1) {
                let num = base * 1000 + (i as u32 % 100);
                coa.add_account(GLAccount::new(
                    format!("{:06}", num),
                    format!("{} {}", name, i + 1),
                    AccountType::Asset,
                    AccountSubType::Cash,
                ));
            }
        }
    }

    fn generate_pcg_class_6(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let bases = [
            (603, "Achats"),
            (641, "Rémunérations"),
            (681, "DAP"),
            (613, "Loyers"),
            (661, "Charges financières"),
        ];
        for (base, name) in bases {
            for i in 0..(count / 5).max(1) {
                let num = base * 1000 + (i as u32 % 100);
                let mut account = GLAccount::new(
                    format!("{:06}", num),
                    format!("{} {}", name, i + 1),
                    AccountType::Expense,
                    AccountSubType::OperatingExpenses,
                );
                account.requires_cost_center = true;
                coa.add_account(account);
            }
        }
    }

    fn generate_pcg_class_7(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let bases = [(701, "Ventes"), (706, "Prestations"), (758, "Produits divers")];
        for (base, name) in bases {
            for i in 0..(count / 3).max(1) {
                let num = base * 1000 + (i as u32 % 100);
                coa.add_account(GLAccount::new(
                    format!("{:06}", num),
                    format!("{} {}", name, i + 1),
                    AccountType::Revenue,
                    AccountSubType::ProductRevenue,
                ));
            }
        }
    }

    fn generate_pcg_class_8(&mut self, coa: &mut ChartOfAccounts) {
        coa.add_account(GLAccount::new(
            "808000".to_string(),
            "Comptes spéciaux".to_string(),
            AccountType::Asset,
            AccountSubType::SuspenseClearing,
        ));

    /// Insert all canonical accounts from `datasynth_core::accounts` into the CoA.
    ///
    /// These are the well-known account numbers (4-digit, 1000-9300 range) that
    /// other generators reference. They are added before auto-generated accounts
    /// (which start at 100000+) so there are no collisions.
    fn seed_canonical_accounts(coa: &mut ChartOfAccounts) {
        // --- Cash accounts (1000-series, Asset / Cash) ---
        coa.add_account(GLAccount::new(
            cash_accounts::OPERATING_CASH.to_string(),
            "Operating Cash".to_string(),
            AccountType::Asset,
            AccountSubType::Cash,
        ));
        coa.add_account(GLAccount::new(
            cash_accounts::BANK_ACCOUNT.to_string(),
            "Bank Account".to_string(),
            AccountType::Asset,
            AccountSubType::Cash,
        ));
        coa.add_account(GLAccount::new(
            cash_accounts::PETTY_CASH.to_string(),
            "Petty Cash".to_string(),
            AccountType::Asset,
            AccountSubType::Cash,
        ));
        coa.add_account(GLAccount::new(
            cash_accounts::WIRE_CLEARING.to_string(),
            "Wire Transfer Clearing".to_string(),
            AccountType::Asset,
            AccountSubType::BankClearing,
        ));

        // --- Control accounts (Asset side) ---
        {
            let mut acct = GLAccount::new(
                control_accounts::AR_CONTROL.to_string(),
                "Accounts Receivable Control".to_string(),
                AccountType::Asset,
                AccountSubType::AccountsReceivable,
            );
            acct.is_control_account = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                control_accounts::IC_AR_CLEARING.to_string(),
                "Intercompany AR Clearing".to_string(),
                AccountType::Asset,
                AccountSubType::AccountsReceivable,
            );
            acct.is_control_account = true;
            coa.add_account(acct);
        }
        coa.add_account(GLAccount::new(
            control_accounts::INVENTORY.to_string(),
            "Inventory".to_string(),
            AccountType::Asset,
            AccountSubType::Inventory,
        ));
        coa.add_account(GLAccount::new(
            control_accounts::FIXED_ASSETS.to_string(),
            "Fixed Assets".to_string(),
            AccountType::Asset,
            AccountSubType::FixedAssets,
        ));
        coa.add_account(GLAccount::new(
            control_accounts::ACCUMULATED_DEPRECIATION.to_string(),
            "Accumulated Depreciation".to_string(),
            AccountType::Asset,
            AccountSubType::AccumulatedDepreciation,
        ));

        // --- Tax asset accounts ---
        coa.add_account(GLAccount::new(
            tax_accounts::INPUT_VAT.to_string(),
            "Input VAT".to_string(),
            AccountType::Asset,
            AccountSubType::OtherReceivables,
        ));
        coa.add_account(GLAccount::new(
            tax_accounts::DEFERRED_TAX_ASSET.to_string(),
            "Deferred Tax Asset".to_string(),
            AccountType::Asset,
            AccountSubType::OtherAssets,
        ));

        // --- Liability / Control accounts (2000-series) ---
        {
            let mut acct = GLAccount::new(
                control_accounts::AP_CONTROL.to_string(),
                "Accounts Payable Control".to_string(),
                AccountType::Liability,
                AccountSubType::AccountsPayable,
            );
            acct.is_control_account = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                control_accounts::IC_AP_CLEARING.to_string(),
                "Intercompany AP Clearing".to_string(),
                AccountType::Liability,
                AccountSubType::AccountsPayable,
            );
            acct.is_control_account = true;
            coa.add_account(acct);
        }
        coa.add_account(GLAccount::new(
            tax_accounts::SALES_TAX_PAYABLE.to_string(),
            "Sales Tax Payable".to_string(),
            AccountType::Liability,
            AccountSubType::TaxLiabilities,
        ));
        coa.add_account(GLAccount::new(
            tax_accounts::VAT_PAYABLE.to_string(),
            "VAT Payable".to_string(),
            AccountType::Liability,
            AccountSubType::TaxLiabilities,
        ));
        coa.add_account(GLAccount::new(
            tax_accounts::WITHHOLDING_TAX_PAYABLE.to_string(),
            "Withholding Tax Payable".to_string(),
            AccountType::Liability,
            AccountSubType::TaxLiabilities,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::ACCRUED_EXPENSES.to_string(),
            "Accrued Expenses".to_string(),
            AccountType::Liability,
            AccountSubType::AccruedLiabilities,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::ACCRUED_SALARIES.to_string(),
            "Accrued Salaries".to_string(),
            AccountType::Liability,
            AccountSubType::AccruedLiabilities,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::ACCRUED_BENEFITS.to_string(),
            "Accrued Benefits".to_string(),
            AccountType::Liability,
            AccountSubType::AccruedLiabilities,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::UNEARNED_REVENUE.to_string(),
            "Unearned Revenue".to_string(),
            AccountType::Liability,
            AccountSubType::DeferredRevenue,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::SHORT_TERM_DEBT.to_string(),
            "Short-Term Debt".to_string(),
            AccountType::Liability,
            AccountSubType::ShortTermDebt,
        ));
        coa.add_account(GLAccount::new(
            tax_accounts::DEFERRED_TAX_LIABILITY.to_string(),
            "Deferred Tax Liability".to_string(),
            AccountType::Liability,
            AccountSubType::TaxLiabilities,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::LONG_TERM_DEBT.to_string(),
            "Long-Term Debt".to_string(),
            AccountType::Liability,
            AccountSubType::LongTermDebt,
        ));
        coa.add_account(GLAccount::new(
            liability_accounts::IC_PAYABLE.to_string(),
            "Intercompany Payable".to_string(),
            AccountType::Liability,
            AccountSubType::OtherLiabilities,
        ));
        {
            let mut acct = GLAccount::new(
                control_accounts::GR_IR_CLEARING.to_string(),
                "GR/IR Clearing".to_string(),
                AccountType::Liability,
                AccountSubType::GoodsReceivedClearing,
            );
            acct.is_suspense_account = true;
            coa.add_account(acct);
        }

        // --- Equity accounts (3000-series) ---
        coa.add_account(GLAccount::new(
            equity_accounts::COMMON_STOCK.to_string(),
            "Common Stock".to_string(),
            AccountType::Equity,
            AccountSubType::CommonStock,
        ));
        coa.add_account(GLAccount::new(
            equity_accounts::APIC.to_string(),
            "Additional Paid-In Capital".to_string(),
            AccountType::Equity,
            AccountSubType::AdditionalPaidInCapital,
        ));
        coa.add_account(GLAccount::new(
            equity_accounts::RETAINED_EARNINGS.to_string(),
            "Retained Earnings".to_string(),
            AccountType::Equity,
            AccountSubType::RetainedEarnings,
        ));
        coa.add_account(GLAccount::new(
            equity_accounts::CURRENT_YEAR_EARNINGS.to_string(),
            "Current Year Earnings".to_string(),
            AccountType::Equity,
            AccountSubType::NetIncome,
        ));
        coa.add_account(GLAccount::new(
            equity_accounts::TREASURY_STOCK.to_string(),
            "Treasury Stock".to_string(),
            AccountType::Equity,
            AccountSubType::TreasuryStock,
        ));
        coa.add_account(GLAccount::new(
            equity_accounts::CTA.to_string(),
            "Currency Translation Adjustment".to_string(),
            AccountType::Equity,
            AccountSubType::OtherComprehensiveIncome,
        ));

        // --- Revenue accounts (4000-series) ---
        coa.add_account(GLAccount::new(
            revenue_accounts::PRODUCT_REVENUE.to_string(),
            "Product Revenue".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));
        coa.add_account(GLAccount::new(
            revenue_accounts::SALES_DISCOUNTS.to_string(),
            "Sales Discounts".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));
        coa.add_account(GLAccount::new(
            revenue_accounts::SALES_RETURNS.to_string(),
            "Sales Returns and Allowances".to_string(),
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ));
        coa.add_account(GLAccount::new(
            revenue_accounts::SERVICE_REVENUE.to_string(),
            "Service Revenue".to_string(),
            AccountType::Revenue,
            AccountSubType::ServiceRevenue,
        ));
        coa.add_account(GLAccount::new(
            revenue_accounts::IC_REVENUE.to_string(),
            "Intercompany Revenue".to_string(),
            AccountType::Revenue,
            AccountSubType::OtherIncome,
        ));
        coa.add_account(GLAccount::new(
            revenue_accounts::OTHER_REVENUE.to_string(),
            "Other Revenue".to_string(),
            AccountType::Revenue,
            AccountSubType::OtherIncome,
        ));

        // --- Expense accounts (5000-7xxx series) ---
        {
            let mut acct = GLAccount::new(
                expense_accounts::COGS.to_string(),
                "Cost of Goods Sold".to_string(),
                AccountType::Expense,
                AccountSubType::CostOfGoodsSold,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::RAW_MATERIALS.to_string(),
                "Raw Materials".to_string(),
                AccountType::Expense,
                AccountSubType::CostOfGoodsSold,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::DIRECT_LABOR.to_string(),
                "Direct Labor".to_string(),
                AccountType::Expense,
                AccountSubType::CostOfGoodsSold,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::MANUFACTURING_OVERHEAD.to_string(),
                "Manufacturing Overhead".to_string(),
                AccountType::Expense,
                AccountSubType::CostOfGoodsSold,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::DEPRECIATION.to_string(),
                "Depreciation Expense".to_string(),
                AccountType::Expense,
                AccountSubType::DepreciationExpense,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::SALARIES_WAGES.to_string(),
                "Salaries and Wages".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::BENEFITS.to_string(),
                "Benefits Expense".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::RENT.to_string(),
                "Rent Expense".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::UTILITIES.to_string(),
                "Utilities Expense".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::OFFICE_SUPPLIES.to_string(),
                "Office Supplies".to_string(),
                AccountType::Expense,
                AccountSubType::AdministrativeExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::TRAVEL_ENTERTAINMENT.to_string(),
                "Travel and Entertainment".to_string(),
                AccountType::Expense,
                AccountSubType::SellingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::PROFESSIONAL_FEES.to_string(),
                "Professional Fees".to_string(),
                AccountType::Expense,
                AccountSubType::AdministrativeExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::INSURANCE.to_string(),
                "Insurance Expense".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::BAD_DEBT.to_string(),
                "Bad Debt Expense".to_string(),
                AccountType::Expense,
                AccountSubType::OperatingExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::INTEREST_EXPENSE.to_string(),
                "Interest Expense".to_string(),
                AccountType::Expense,
                AccountSubType::InterestExpense,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::PURCHASE_DISCOUNTS.to_string(),
                "Purchase Discounts".to_string(),
                AccountType::Expense,
                AccountSubType::OtherExpenses,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                expense_accounts::FX_GAIN_LOSS.to_string(),
                "FX Gain/Loss".to_string(),
                AccountType::Expense,
                AccountSubType::ForeignExchangeLoss,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }

        // --- Tax expense (8000-series) ---
        {
            let mut acct = GLAccount::new(
                tax_accounts::TAX_EXPENSE.to_string(),
                "Tax Expense".to_string(),
                AccountType::Expense,
                AccountSubType::TaxExpense,
            );
            acct.requires_cost_center = true;
            coa.add_account(acct);
        }

        // --- Suspense / Clearing accounts (9000-series) ---
        {
            let mut acct = GLAccount::new(
                suspense_accounts::GENERAL_SUSPENSE.to_string(),
                "General Suspense".to_string(),
                AccountType::Asset,
                AccountSubType::SuspenseClearing,
            );
            acct.is_suspense_account = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                suspense_accounts::PAYROLL_CLEARING.to_string(),
                "Payroll Clearing".to_string(),
                AccountType::Asset,
                AccountSubType::SuspenseClearing,
            );
            acct.is_suspense_account = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                suspense_accounts::BANK_RECONCILIATION_SUSPENSE.to_string(),
                "Bank Reconciliation Suspense".to_string(),
                AccountType::Asset,
                AccountSubType::BankClearing,
            );
            acct.is_suspense_account = true;
            coa.add_account(acct);
        }
        {
            let mut acct = GLAccount::new(
                suspense_accounts::IC_ELIMINATION_SUSPENSE.to_string(),
                "IC Elimination Suspense".to_string(),
                AccountType::Asset,
                AccountSubType::IntercompanyClearing,
            );
            acct.is_suspense_account = true;
            coa.add_account(acct);
        }
    }

    fn generate_asset_accounts(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let sub_types = vec![
            (AccountSubType::Cash, "Cash", 0.15),
            (
                AccountSubType::AccountsReceivable,
                "Accounts Receivable",
                0.20,
            ),
            (AccountSubType::Inventory, "Inventory", 0.15),
            (AccountSubType::PrepaidExpenses, "Prepaid Expenses", 0.10),
            (AccountSubType::FixedAssets, "Fixed Assets", 0.25),
            (
                AccountSubType::AccumulatedDepreciation,
                "Accumulated Depreciation",
                0.10,
            ),
            (AccountSubType::OtherAssets, "Other Assets", 0.05),
        ];

        let mut account_num = 100000u32;
        for (sub_type, name_prefix, weight) in sub_types {
            let sub_count = ((count as f64) * weight).round() as usize;
            for i in 0..sub_count.max(1) {
                let account = GLAccount::new(
                    format!("{}", account_num),
                    format!("{} {}", name_prefix, i + 1),
                    AccountType::Asset,
                    sub_type,
                );
                coa.add_account(account);
                account_num += 10;
            }
        }
    }

    fn generate_liability_accounts(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let sub_types = vec![
            (AccountSubType::AccountsPayable, "Accounts Payable", 0.25),
            (
                AccountSubType::AccruedLiabilities,
                "Accrued Liabilities",
                0.20,
            ),
            (AccountSubType::ShortTermDebt, "Short-Term Debt", 0.15),
            (AccountSubType::LongTermDebt, "Long-Term Debt", 0.15),
            (AccountSubType::DeferredRevenue, "Deferred Revenue", 0.15),
            (AccountSubType::TaxLiabilities, "Tax Liabilities", 0.10),
        ];

        let mut account_num = 200000u32;
        for (sub_type, name_prefix, weight) in sub_types {
            let sub_count = ((count as f64) * weight).round() as usize;
            for i in 0..sub_count.max(1) {
                let account = GLAccount::new(
                    format!("{}", account_num),
                    format!("{} {}", name_prefix, i + 1),
                    AccountType::Liability,
                    sub_type,
                );
                coa.add_account(account);
                account_num += 10;
            }
        }
    }

    fn generate_equity_accounts(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let sub_types = vec![
            (AccountSubType::CommonStock, "Common Stock", 0.20),
            (AccountSubType::RetainedEarnings, "Retained Earnings", 0.30),
            (AccountSubType::AdditionalPaidInCapital, "APIC", 0.20),
            (AccountSubType::OtherComprehensiveIncome, "OCI", 0.30),
        ];

        let mut account_num = 300000u32;
        for (sub_type, name_prefix, weight) in sub_types {
            let sub_count = ((count as f64) * weight).round() as usize;
            for i in 0..sub_count.max(1) {
                let account = GLAccount::new(
                    format!("{}", account_num),
                    format!("{} {}", name_prefix, i + 1),
                    AccountType::Equity,
                    sub_type,
                );
                coa.add_account(account);
                account_num += 10;
            }
        }
    }

    fn generate_revenue_accounts(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let sub_types = vec![
            (AccountSubType::ProductRevenue, "Product Revenue", 0.40),
            (AccountSubType::ServiceRevenue, "Service Revenue", 0.30),
            (AccountSubType::InterestIncome, "Interest Income", 0.10),
            (AccountSubType::OtherIncome, "Other Income", 0.20),
        ];

        let mut account_num = 400000u32;
        for (sub_type, name_prefix, weight) in sub_types {
            let sub_count = ((count as f64) * weight).round() as usize;
            for i in 0..sub_count.max(1) {
                let account = GLAccount::new(
                    format!("{}", account_num),
                    format!("{} {}", name_prefix, i + 1),
                    AccountType::Revenue,
                    sub_type,
                );
                coa.add_account(account);
                account_num += 10;
            }
        }
    }

    fn generate_expense_accounts(&mut self, coa: &mut ChartOfAccounts, count: usize) {
        let sub_types = vec![
            (AccountSubType::CostOfGoodsSold, "COGS", 0.20),
            (
                AccountSubType::OperatingExpenses,
                "Operating Expenses",
                0.25,
            ),
            (AccountSubType::SellingExpenses, "Selling Expenses", 0.15),
            (
                AccountSubType::AdministrativeExpenses,
                "Admin Expenses",
                0.15,
            ),
            (AccountSubType::DepreciationExpense, "Depreciation", 0.10),
            (AccountSubType::InterestExpense, "Interest Expense", 0.05),
            (AccountSubType::TaxExpense, "Tax Expense", 0.05),
            (AccountSubType::OtherExpenses, "Other Expenses", 0.05),
        ];

        let mut account_num = 500000u32;
        for (sub_type, name_prefix, weight) in sub_types {
            let sub_count = ((count as f64) * weight).round() as usize;
            for i in 0..sub_count.max(1) {
                let mut account = GLAccount::new(
                    format!("{}", account_num),
                    format!("{} {}", name_prefix, i + 1),
                    AccountType::Expense,
                    sub_type,
                );
                account.requires_cost_center = true;
                coa.add_account(account);
                account_num += 10;
            }
        }
    }

    fn generate_suspense_accounts(&mut self, coa: &mut ChartOfAccounts) {
        let suspense_types = vec![
            (AccountSubType::SuspenseClearing, "Suspense Clearing"),
            (AccountSubType::GoodsReceivedClearing, "GR/IR Clearing"),
            (AccountSubType::BankClearing, "Bank Clearing"),
            (
                AccountSubType::IntercompanyClearing,
                "Intercompany Clearing",
            ),
        ];

        let mut account_num = 199000u32;
        for (sub_type, name) in suspense_types {
            let mut account = GLAccount::new(
                format!("{}", account_num),
                name.to_string(),
                AccountType::Asset,
                sub_type,
            );
            account.is_suspense_account = true;
            coa.add_account(account);
            account_num += 100;
        }
    }
}

impl Generator for ChartOfAccountsGenerator {
    type Item = ChartOfAccounts;
    type Config = (CoAComplexity, IndustrySector);

    fn new(config: Self::Config, seed: u64) -> Self {
        Self::new(config.0, config.1, seed)
    }

    fn generate_one(&mut self) -> Self::Item {
        self.generate()
    }

    fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.count = 0;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_small_coa() {
        let mut gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = gen.generate();

        assert!(coa.account_count() >= 50);
        assert!(!coa.get_suspense_accounts().is_empty());
    }

    #[test]
    fn test_generate_pcg_coa() {
        let mut gen = ChartOfAccountsGenerator::new(
            CoAComplexity::Small,
            IndustrySector::Manufacturing,
            42,
        )
        .with_french_pcg(true);
        let coa = gen.generate();

        assert_eq!(coa.country, "FR");
        assert!(coa.name.contains("Plan Comptable") || coa.name.contains("PCG"));
        assert!(coa.account_count() >= 20);
        // PCG accounts are 6-digit (e.g. 411000, 601000)
        let first = coa.accounts.first().expect("has accounts");
        assert_eq!(first.account_number.len(), 6);
    }
}
