//! Chart of Accounts generator.

use datasynth_core::models::*;
use datasynth_core::traits::Generator;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generator for Chart of Accounts.
pub struct ChartOfAccountsGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    complexity: CoAComplexity,
    industry: IndustrySector,
    count: u64,
}

impl ChartOfAccountsGenerator {
    /// Create a new CoA generator.
    pub fn new(complexity: CoAComplexity, industry: IndustrySector, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            complexity,
            industry,
            count: 0,
        }
    }

    /// Generate a complete chart of accounts.
    pub fn generate(&mut self) -> ChartOfAccounts {
        self.count += 1;
        let target_count = self.complexity.target_count();

        let mut coa = ChartOfAccounts::new(
            format!("COA_{:?}_{}", self.industry, self.complexity.target_count()),
            format!("{:?} Chart of Accounts", self.industry),
            "US".to_string(),
            self.industry,
            self.complexity,
        );

        // Generate accounts by type
        self.generate_asset_accounts(&mut coa, target_count / 5);
        self.generate_liability_accounts(&mut coa, target_count / 6);
        self.generate_equity_accounts(&mut coa, target_count / 10);
        self.generate_revenue_accounts(&mut coa, target_count / 5);
        self.generate_expense_accounts(&mut coa, target_count / 4);
        self.generate_suspense_accounts(&mut coa);

        coa
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
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
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
}
