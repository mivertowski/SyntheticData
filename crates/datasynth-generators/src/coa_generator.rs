//! Chart of Accounts generator.

use datasynth_core::models::*;
use datasynth_core::pcg_loader;
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
    /// When true, generate Plan Comptable Général (French GAAP) structure.
    use_french_pcg: bool,
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
