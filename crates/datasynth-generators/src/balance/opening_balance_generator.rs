//! Opening balance generator.
//!
//! Generates coherent opening balances that:
//! - Satisfy the balance sheet equation (A = L + E)
//! - Reflect industry-specific asset compositions
//! - Support configurable debt-to-equity ratios
//! - Provide realistic starting positions for simulation

use chrono::{Datelike, NaiveDate};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::balance::{
    AccountBalance, AccountCategory, AccountType, AssetComposition, CapitalStructure,
    GeneratedOpeningBalance, IndustryType, OpeningBalanceSpec, TargetRatios,
};
use datasynth_core::models::ChartOfAccounts;

/// Configuration for opening balance generation.
#[derive(Debug, Clone)]
pub struct OpeningBalanceConfig {
    /// Total assets to generate.
    pub total_assets: Decimal,
    /// Industry type for composition defaults.
    pub industry: IndustryType,
    /// Custom asset composition (overrides industry defaults).
    pub asset_composition: Option<AssetComposition>,
    /// Custom capital structure (overrides industry defaults).
    pub capital_structure: Option<CapitalStructure>,
    /// Custom target ratios (overrides industry defaults).
    pub target_ratios: Option<TargetRatios>,
    /// Whether to add random variation to amounts.
    pub add_variation: bool,
    /// Maximum variation percentage (0.0 to 1.0).
    pub variation_percent: Decimal,
}

impl Default for OpeningBalanceConfig {
    fn default() -> Self {
        Self {
            total_assets: dec!(10_000_000),
            industry: IndustryType::Manufacturing,
            asset_composition: None,
            capital_structure: None,
            target_ratios: None,
            add_variation: true,
            variation_percent: dec!(0.05),
        }
    }
}

/// Generator for opening balance sheets.
pub struct OpeningBalanceGenerator {
    config: OpeningBalanceConfig,
    rng: ChaCha8Rng,
}

impl OpeningBalanceGenerator {
    /// Creates a new opening balance generator.
    pub fn new(config: OpeningBalanceConfig, rng: ChaCha8Rng) -> Self {
        Self { config, rng }
    }

    /// Creates a new opening balance generator from a seed, constructing the RNG internally.
    pub fn with_seed(config: OpeningBalanceConfig, seed: u64) -> Self {
        Self::new(config, ChaCha8Rng::seed_from_u64(seed))
    }

    /// Creates a generator with default configuration.
    pub fn with_defaults(rng: ChaCha8Rng) -> Self {
        Self::new(OpeningBalanceConfig::default(), rng)
    }

    /// Generates opening balances based on specification.
    pub fn generate(
        &mut self,
        spec: &OpeningBalanceSpec,
        chart_of_accounts: &ChartOfAccounts,
        as_of_date: NaiveDate,
        company_code: &str,
    ) -> GeneratedOpeningBalance {
        let mut balances = HashMap::new();

        // Get effective compositions and structures
        let asset_comp = spec.asset_composition.clone();
        let capital_struct = spec.capital_structure.clone();

        // Calculate major balance sheet categories
        let total_assets = spec.total_assets;

        // Assets breakdown (percentages are already in decimal form, e.g., 40 means 40%)
        let current_assets =
            self.apply_variation(total_assets * asset_comp.current_assets_percent / dec!(100));
        let non_current_assets = total_assets - current_assets;
        let fixed_assets =
            self.apply_variation(non_current_assets * asset_comp.ppe_percent / dec!(100));
        let intangible_assets =
            self.apply_variation(non_current_assets * asset_comp.intangibles_percent / dec!(100));
        let other_assets = non_current_assets - fixed_assets - intangible_assets;

        // Current assets detail
        let cash = self.apply_variation(current_assets * asset_comp.cash_percent / dec!(100));
        let accounts_receivable =
            self.calculate_ar_from_dso(&spec.target_ratios, current_assets - cash, as_of_date);
        let inventory =
            self.apply_variation((current_assets - cash - accounts_receivable) * dec!(0.6));
        let prepaid_expenses = current_assets - cash - accounts_receivable - inventory;

        // Fixed assets detail
        let ppe_gross = self.apply_variation(fixed_assets * dec!(1.4)); // Gross amount before depreciation
        let accumulated_depreciation = ppe_gross - fixed_assets;

        // Liabilities and equity
        let total_liabilities = total_assets * capital_struct.debt_percent / dec!(100);
        let total_equity = total_assets - total_liabilities;

        // Current liabilities
        let current_liabilities = self.apply_variation(total_liabilities * dec!(0.35));
        let accounts_payable =
            self.calculate_ap_from_dpo(&spec.target_ratios, current_liabilities, as_of_date);
        let accrued_expenses = self.apply_variation(current_liabilities * dec!(0.25));
        let short_term_debt = self.apply_variation(current_liabilities * dec!(0.15));
        let other_current_liabilities =
            current_liabilities - accounts_payable - accrued_expenses - short_term_debt;

        // Long-term liabilities
        let long_term_liabilities = total_liabilities - current_liabilities;
        let long_term_debt = self.apply_variation(long_term_liabilities * dec!(0.85));
        let other_long_term_liabilities = long_term_liabilities - long_term_debt;

        // Equity breakdown
        let common_stock =
            self.apply_variation(total_equity * capital_struct.common_stock_percent / dec!(100));
        let retained_earnings = total_equity - common_stock;

        // Create account balances using chart of accounts
        // Assets (debit balances)
        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1000", "Cash"),
            AccountType::Asset,
            cash,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1100", "Accounts Receivable"),
            AccountType::Asset,
            accounts_receivable,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1200", "Inventory"),
            AccountType::Asset,
            inventory,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1300", "Prepaid Expenses"),
            AccountType::Asset,
            prepaid_expenses,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1500", "Property Plant Equipment"),
            AccountType::Asset,
            ppe_gross,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1590", "Accumulated Depreciation"),
            AccountType::ContraAsset,
            accumulated_depreciation,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1600", "Intangible Assets"),
            AccountType::Asset,
            intangible_assets,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "1900", "Other Assets"),
            AccountType::Asset,
            other_assets,
            as_of_date,
            company_code,
        );

        // Liabilities (credit balances)
        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2000", "Accounts Payable"),
            AccountType::Liability,
            accounts_payable,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2100", "Accrued Expenses"),
            AccountType::Liability,
            accrued_expenses,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2200", "Short-term Debt"),
            AccountType::Liability,
            short_term_debt,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2300", "Other Current Liabilities"),
            AccountType::Liability,
            other_current_liabilities,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2500", "Long-term Debt"),
            AccountType::Liability,
            long_term_debt,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "2900", "Other Long-term Liabilities"),
            AccountType::Liability,
            other_long_term_liabilities,
            as_of_date,
            company_code,
        );

        // Equity (credit balances)
        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "3000", "Common Stock"),
            AccountType::Equity,
            common_stock,
            as_of_date,
            company_code,
        );

        self.add_balance(
            &mut balances,
            &self.find_account(chart_of_accounts, "3200", "Retained Earnings"),
            AccountType::Equity,
            retained_earnings,
            as_of_date,
            company_code,
        );

        // Calculate totals from balances
        // Assets = gross assets - contra assets (accumulated depreciation)
        let gross_assets = self.calculate_total_type(&balances, AccountType::Asset);
        let contra_assets = self.calculate_total_type(&balances, AccountType::ContraAsset);
        let total_assets = gross_assets - contra_assets;
        let total_liabilities = self.calculate_total_type(&balances, AccountType::Liability);
        let total_equity = self.calculate_total_type(&balances, AccountType::Equity);
        let is_balanced = (total_assets - total_liabilities - total_equity).abs() < dec!(1.00);

        // Convert AccountBalance map to simple Decimal map
        let simple_balances: HashMap<String, Decimal> = balances
            .iter()
            .map(|(k, v)| (k.clone(), v.closing_balance))
            .collect();

        // Calculate actual ratios
        let calculated_ratios = self.calculate_ratios_simple(
            &simple_balances,
            total_assets,
            total_liabilities,
            total_equity,
        );

        GeneratedOpeningBalance {
            company_code: company_code.to_string(),
            as_of_date,
            balances: simple_balances,
            total_assets,
            total_liabilities,
            total_equity,
            is_balanced,
            calculated_ratios,
        }
    }

    /// Calculate total for an account type.
    fn calculate_total_type(
        &self,
        balances: &HashMap<String, AccountBalance>,
        account_type: AccountType,
    ) -> Decimal {
        balances
            .values()
            .filter(|b| b.account_type == account_type)
            .map(|b| b.closing_balance)
            .sum()
    }

    /// Generates opening balances from configuration defaults.
    pub fn generate_from_config(
        &mut self,
        chart_of_accounts: &ChartOfAccounts,
        as_of_date: NaiveDate,
        company_code: &str,
    ) -> GeneratedOpeningBalance {
        let spec = OpeningBalanceSpec::for_industry(self.config.total_assets, self.config.industry);
        self.generate(&spec, chart_of_accounts, as_of_date, company_code)
    }

    /// Generates opening balances for multiple companies.
    pub fn generate_for_companies(
        &mut self,
        specs: &[(String, OpeningBalanceSpec)],
        chart_of_accounts: &ChartOfAccounts,
        as_of_date: NaiveDate,
    ) -> Vec<GeneratedOpeningBalance> {
        specs
            .iter()
            .map(|(company_code, spec)| {
                self.generate(spec, chart_of_accounts, as_of_date, company_code)
            })
            .collect()
    }

    /// Applies random variation to an amount if configured.
    fn apply_variation(&mut self, amount: Decimal) -> Decimal {
        if !self.config.add_variation || self.config.variation_percent == Decimal::ZERO {
            return amount;
        }

        let variation_range = amount * self.config.variation_percent;
        let random_factor: f64 = self.rng.gen_range(-1.0..1.0);
        let variation = variation_range * Decimal::try_from(random_factor).unwrap_or_default();

        (amount + variation).max(Decimal::ZERO)
    }

    /// Calculates AR balance from target DSO.
    fn calculate_ar_from_dso(
        &self,
        target_ratios: &TargetRatios,
        max_ar: Decimal,
        _as_of_date: NaiveDate,
    ) -> Decimal {
        // DSO = (AR / Annual Revenue) * 365
        // AR = (DSO * Annual Revenue) / 365
        // Estimate annual revenue as 4x current assets (rough approximation)
        let estimated_annual_revenue = max_ar * dec!(10);
        let target_ar =
            (Decimal::from(target_ratios.target_dso_days) * estimated_annual_revenue) / dec!(365);

        // Cap at reasonable percentage of current assets
        target_ar.min(max_ar * dec!(0.7))
    }

    /// Calculates AP balance from target DPO.
    fn calculate_ap_from_dpo(
        &self,
        target_ratios: &TargetRatios,
        current_liabilities: Decimal,
        _as_of_date: NaiveDate,
    ) -> Decimal {
        // DPO = (AP / COGS) * 365
        // AP = (DPO * COGS) / 365
        // Estimate COGS as related to current liabilities
        let estimated_cogs = current_liabilities * dec!(8);
        let target_ap = (Decimal::from(target_ratios.target_dpo_days) * estimated_cogs) / dec!(365);

        // Cap at reasonable percentage of current liabilities
        target_ap.min(current_liabilities * dec!(0.5))
    }

    /// Finds an account code from the chart of accounts or uses a default.
    fn find_account(
        &self,
        chart_of_accounts: &ChartOfAccounts,
        default_code: &str,
        description: &str,
    ) -> String {
        // Try to find account by description pattern
        for account in &chart_of_accounts.accounts {
            if account
                .description()
                .to_lowercase()
                .contains(&description.to_lowercase())
            {
                return account.account_code().to_string();
            }
        }

        // Fall back to default code
        default_code.to_string()
    }

    /// Adds a balance to the map.
    fn add_balance(
        &self,
        balances: &mut HashMap<String, AccountBalance>,
        account_code: &str,
        account_type: AccountType,
        amount: Decimal,
        as_of_date: NaiveDate,
        company_code: &str,
    ) {
        use chrono::Datelike;

        if amount == Decimal::ZERO {
            return;
        }

        let mut balance = AccountBalance::new(
            company_code.to_string(),
            account_code.to_string(),
            account_type,
            "USD".to_string(),
            as_of_date.year(),
            as_of_date.month(),
        );
        balance.opening_balance = amount;
        balance.closing_balance = amount;

        balances.insert(account_code.to_string(), balance);
    }

    /// Calculates financial ratios from the generated balances.
    fn calculate_ratios_simple(
        &self,
        balances: &HashMap<String, Decimal>,
        _total_assets: Decimal,
        _total_liabilities: Decimal,
        total_equity: Decimal,
    ) -> datasynth_core::models::balance::CalculatedRatios {
        // Calculate current ratio
        let current_assets = self.sum_balances(balances, &["1000", "1100", "1200", "1300"]);
        let current_liabilities = self.sum_balances(balances, &["2000", "2100", "2200", "2300"]);
        let current_ratio = if current_liabilities > Decimal::ZERO {
            Some(current_assets / current_liabilities)
        } else {
            None
        };

        // Calculate quick ratio (current assets - inventory) / current liabilities
        let inventory = self.get_balance(balances, "1200");
        let quick_ratio = if current_liabilities > Decimal::ZERO {
            Some((current_assets - inventory) / current_liabilities)
        } else {
            None
        };

        // Calculate debt to equity
        let total_debt = self.sum_balances(balances, &["2200", "2500"]);
        let debt_to_equity = if total_equity > Decimal::ZERO {
            Some(total_debt / total_equity)
        } else {
            None
        };

        // Working capital = current assets - current liabilities
        let working_capital = current_assets - current_liabilities;

        datasynth_core::models::balance::CalculatedRatios {
            current_ratio,
            quick_ratio,
            debt_to_equity,
            working_capital,
        }
    }

    /// Sums balances for a set of account codes.
    fn sum_balances(
        &self,
        balances: &HashMap<String, Decimal>,
        account_prefixes: &[&str],
    ) -> Decimal {
        balances
            .iter()
            .filter(|(code, _)| {
                account_prefixes
                    .iter()
                    .any(|prefix| code.starts_with(prefix))
            })
            .map(|(_, amount)| amount.abs())
            .sum()
    }

    /// Gets a single account balance.
    fn get_balance(&self, balances: &HashMap<String, Decimal>, account_prefix: &str) -> Decimal {
        balances
            .iter()
            .filter(|(code, _)| code.starts_with(account_prefix))
            .map(|(_, amount)| amount.abs())
            .sum()
    }
}

/// Builder for opening balance specifications.
pub struct OpeningBalanceSpecBuilder {
    company_code: String,
    as_of_date: NaiveDate,
    fiscal_year: i32,
    currency: String,
    total_assets: Decimal,
    industry: IndustryType,
    asset_composition: Option<AssetComposition>,
    capital_structure: Option<CapitalStructure>,
    target_ratios: Option<TargetRatios>,
    account_overrides: HashMap<String, datasynth_core::models::balance::AccountSpec>,
}

impl OpeningBalanceSpecBuilder {
    /// Creates a new builder with required parameters.
    pub fn new(
        company_code: impl Into<String>,
        as_of_date: NaiveDate,
        total_assets: Decimal,
        industry: IndustryType,
    ) -> Self {
        let year = as_of_date.year();
        Self {
            company_code: company_code.into(),
            as_of_date,
            fiscal_year: year,
            currency: "USD".to_string(),
            total_assets,
            industry,
            asset_composition: None,
            capital_structure: None,
            target_ratios: None,
            account_overrides: HashMap::new(),
        }
    }

    /// Sets the currency.
    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }

    /// Sets the fiscal year.
    pub fn with_fiscal_year(mut self, fiscal_year: i32) -> Self {
        self.fiscal_year = fiscal_year;
        self
    }

    /// Sets custom asset composition.
    pub fn with_asset_composition(mut self, composition: AssetComposition) -> Self {
        self.asset_composition = Some(composition);
        self
    }

    /// Sets custom capital structure.
    pub fn with_capital_structure(mut self, structure: CapitalStructure) -> Self {
        self.capital_structure = Some(structure);
        self
    }

    /// Sets custom target ratios.
    pub fn with_target_ratios(mut self, ratios: TargetRatios) -> Self {
        self.target_ratios = Some(ratios);
        self
    }

    /// Adds an account override with a fixed balance.
    pub fn with_account_override(
        mut self,
        account_code: impl Into<String>,
        description: impl Into<String>,
        account_type: AccountType,
        fixed_balance: Decimal,
    ) -> Self {
        let code = account_code.into();
        self.account_overrides.insert(
            code.clone(),
            datasynth_core::models::balance::AccountSpec {
                account_code: code,
                description: description.into(),
                account_type,
                category: AccountCategory::CurrentAssets,
                fixed_balance: Some(fixed_balance),
                category_percent: None,
                total_assets_percent: None,
            },
        );
        self
    }

    /// Builds the opening balance specification.
    pub fn build(self) -> OpeningBalanceSpec {
        let industry_defaults = OpeningBalanceSpec::for_industry(self.total_assets, self.industry);

        OpeningBalanceSpec {
            company_code: self.company_code,
            as_of_date: self.as_of_date,
            fiscal_year: self.fiscal_year,
            currency: self.currency,
            total_assets: self.total_assets,
            industry: self.industry,
            asset_composition: self
                .asset_composition
                .unwrap_or(industry_defaults.asset_composition),
            capital_structure: self
                .capital_structure
                .unwrap_or(industry_defaults.capital_structure),
            target_ratios: self
                .target_ratios
                .unwrap_or(industry_defaults.target_ratios),
            account_overrides: self.account_overrides,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{CoAComplexity, IndustrySector};
    use rand::SeedableRng;

    fn create_test_chart() -> ChartOfAccounts {
        ChartOfAccounts::new(
            "TEST-COA".to_string(),
            "Test Chart of Accounts".to_string(),
            "US".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Medium,
        )
    }

    #[test]
    fn test_generate_opening_balances() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let config = OpeningBalanceConfig {
            add_variation: false,
            ..Default::default()
        };
        let mut generator = OpeningBalanceGenerator::new(config, rng);

        let spec = OpeningBalanceSpec::for_industry(dec!(1_000_000), IndustryType::Manufacturing);
        let chart = create_test_chart();
        let result = generator.generate(
            &spec,
            &chart,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            "1000",
        );

        // Verify balance sheet is balanced
        assert!(result.is_balanced);

        // Verify total assets match
        assert!(
            (result.total_assets - dec!(1_000_000)).abs() < dec!(1000),
            "Total assets should be close to spec"
        );
    }

    #[test]
    fn test_industry_specific_composition() {
        let rng = ChaCha8Rng::seed_from_u64(54321);
        let _generator = OpeningBalanceGenerator::with_defaults(rng);

        let tech_spec = OpeningBalanceSpec::for_industry(dec!(1_000_000), IndustryType::Technology);
        let mfg_spec =
            OpeningBalanceSpec::for_industry(dec!(1_000_000), IndustryType::Manufacturing);

        // Tech should have higher intangible assets
        assert!(
            tech_spec.asset_composition.intangibles_percent
                > mfg_spec.asset_composition.intangibles_percent
        );

        // Manufacturing should have higher fixed assets (PPE)
        assert!(mfg_spec.asset_composition.ppe_percent > tech_spec.asset_composition.ppe_percent);
    }

    #[test]
    fn test_builder_pattern() {
        let as_of = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let spec =
            OpeningBalanceSpecBuilder::new("TEST", as_of, dec!(5_000_000), IndustryType::Retail)
                .with_target_ratios(TargetRatios {
                    target_dso_days: 30,
                    target_dpo_days: 45,
                    ..TargetRatios::for_industry(IndustryType::Retail)
                })
                .with_account_override("1000", "Cash", AccountType::Asset, dec!(500_000))
                .build();

        assert_eq!(spec.total_assets, dec!(5_000_000));
        assert_eq!(spec.target_ratios.target_dso_days, 30);
        assert_eq!(spec.account_overrides.len(), 1);
    }
}
