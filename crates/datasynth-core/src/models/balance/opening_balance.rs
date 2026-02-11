//! Opening balance specification models.
//!
//! Provides structures for defining and generating coherent opening balance sheets
//! with industry-specific compositions and configurable financial ratios.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::account_balance::AccountType;
use super::trial_balance::AccountCategory;

/// Specification for generating opening balances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningBalanceSpec {
    /// Company code.
    pub company_code: String,
    /// Opening balance date (typically start of fiscal year).
    pub as_of_date: NaiveDate,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Currency.
    pub currency: String,
    /// Total assets target.
    pub total_assets: Decimal,
    /// Industry sector for composition.
    pub industry: IndustryType,
    /// Asset composition specification.
    pub asset_composition: AssetComposition,
    /// Liability and equity specification.
    pub capital_structure: CapitalStructure,
    /// Target financial ratios.
    pub target_ratios: TargetRatios,
    /// Individual account specifications (overrides).
    pub account_overrides: HashMap<String, AccountSpec>,
}

impl OpeningBalanceSpec {
    /// Create a new opening balance specification.
    pub fn new(
        company_code: String,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        currency: String,
        total_assets: Decimal,
        industry: IndustryType,
    ) -> Self {
        Self {
            company_code,
            as_of_date,
            fiscal_year,
            currency,
            total_assets,
            industry,
            asset_composition: AssetComposition::for_industry(industry),
            capital_structure: CapitalStructure::default(),
            target_ratios: TargetRatios::for_industry(industry),
            account_overrides: HashMap::new(),
        }
    }

    /// Create a specification for a given industry with default parameters.
    /// This is a convenience method for creating industry-specific opening balances.
    pub fn for_industry(total_assets: Decimal, industry: IndustryType) -> Self {
        Self {
            company_code: String::new(),
            as_of_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            fiscal_year: 2024,
            currency: "USD".to_string(),
            total_assets,
            industry,
            asset_composition: AssetComposition::for_industry(industry),
            capital_structure: CapitalStructure::for_industry(industry),
            target_ratios: TargetRatios::for_industry(industry),
            account_overrides: HashMap::new(),
        }
    }

    /// Validate that the specification is coherent (A = L + E).
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check asset composition sums to 100%
        let asset_total = self.asset_composition.total_percentage();
        if (asset_total - dec!(100)).abs() > dec!(0.01) {
            errors.push(format!(
                "Asset composition should sum to 100%, got {}%",
                asset_total
            ));
        }

        // Check capital structure sums to 100%
        let capital_total =
            self.capital_structure.debt_percent + self.capital_structure.equity_percent;
        if (capital_total - dec!(100)).abs() > dec!(0.01) {
            errors.push(format!(
                "Capital structure should sum to 100%, got {}%",
                capital_total
            ));
        }

        // Check current ratio feasibility
        if self.target_ratios.current_ratio < dec!(0.5) {
            errors.push("Current ratio below 0.5 indicates severe liquidity problems".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Calculate total liabilities based on capital structure.
    pub fn calculate_total_liabilities(&self) -> Decimal {
        self.total_assets * self.capital_structure.debt_percent / dec!(100)
    }

    /// Calculate total equity based on capital structure.
    pub fn calculate_total_equity(&self) -> Decimal {
        self.total_assets * self.capital_structure.equity_percent / dec!(100)
    }

    /// Calculate current assets based on composition.
    pub fn calculate_current_assets(&self) -> Decimal {
        self.total_assets * self.asset_composition.current_assets_percent / dec!(100)
    }

    /// Calculate non-current assets based on composition.
    pub fn calculate_non_current_assets(&self) -> Decimal {
        self.total_assets * (dec!(100) - self.asset_composition.current_assets_percent) / dec!(100)
    }

    /// Calculate current liabilities to achieve target current ratio.
    pub fn calculate_current_liabilities(&self) -> Decimal {
        let current_assets = self.calculate_current_assets();
        if self.target_ratios.current_ratio > Decimal::ZERO {
            current_assets / self.target_ratios.current_ratio
        } else {
            Decimal::ZERO
        }
    }
}

/// Industry type for composition defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndustryType {
    /// Manufacturing company.
    #[default]
    Manufacturing,
    /// Retail/wholesale trade.
    Retail,
    /// Service company.
    Services,
    /// Technology company.
    Technology,
    /// Financial services.
    Financial,
    /// Healthcare.
    Healthcare,
    /// Utilities.
    Utilities,
    /// Real estate.
    RealEstate,
}

/// Asset composition specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetComposition {
    /// Current assets as percentage of total.
    pub current_assets_percent: Decimal,
    /// Cash and equivalents as % of current assets.
    pub cash_percent: Decimal,
    /// Accounts receivable as % of current assets.
    pub ar_percent: Decimal,
    /// Inventory as % of current assets.
    pub inventory_percent: Decimal,
    /// Prepaid expenses as % of current assets.
    pub prepaid_percent: Decimal,
    /// Other current assets as % of current assets.
    pub other_current_percent: Decimal,
    /// Property, plant, equipment as % of non-current assets.
    pub ppe_percent: Decimal,
    /// Intangible assets as % of non-current assets.
    pub intangibles_percent: Decimal,
    /// Investments as % of non-current assets.
    pub investments_percent: Decimal,
    /// Other non-current assets as % of non-current assets.
    pub other_noncurrent_percent: Decimal,
}

impl AssetComposition {
    /// Get composition for a specific industry.
    pub fn for_industry(industry: IndustryType) -> Self {
        match industry {
            IndustryType::Manufacturing => Self {
                current_assets_percent: dec!(40),
                cash_percent: dec!(15),
                ar_percent: dec!(30),
                inventory_percent: dec!(45),
                prepaid_percent: dec!(5),
                other_current_percent: dec!(5),
                ppe_percent: dec!(70),
                intangibles_percent: dec!(10),
                investments_percent: dec!(10),
                other_noncurrent_percent: dec!(10),
            },
            IndustryType::Retail => Self {
                current_assets_percent: dec!(55),
                cash_percent: dec!(10),
                ar_percent: dec!(15),
                inventory_percent: dec!(65),
                prepaid_percent: dec!(5),
                other_current_percent: dec!(5),
                ppe_percent: dec!(60),
                intangibles_percent: dec!(20),
                investments_percent: dec!(10),
                other_noncurrent_percent: dec!(10),
            },
            IndustryType::Services => Self {
                current_assets_percent: dec!(50),
                cash_percent: dec!(25),
                ar_percent: dec!(50),
                inventory_percent: dec!(5),
                prepaid_percent: dec!(10),
                other_current_percent: dec!(10),
                ppe_percent: dec!(40),
                intangibles_percent: dec!(30),
                investments_percent: dec!(15),
                other_noncurrent_percent: dec!(15),
            },
            IndustryType::Technology => Self {
                current_assets_percent: dec!(60),
                cash_percent: dec!(40),
                ar_percent: dec!(35),
                inventory_percent: dec!(5),
                prepaid_percent: dec!(10),
                other_current_percent: dec!(10),
                ppe_percent: dec!(25),
                intangibles_percent: dec!(50),
                investments_percent: dec!(15),
                other_noncurrent_percent: dec!(10),
            },
            IndustryType::Financial => Self {
                current_assets_percent: dec!(70),
                cash_percent: dec!(30),
                ar_percent: dec!(40),
                inventory_percent: dec!(0),
                prepaid_percent: dec!(5),
                other_current_percent: dec!(25),
                ppe_percent: dec!(20),
                intangibles_percent: dec!(30),
                investments_percent: dec!(40),
                other_noncurrent_percent: dec!(10),
            },
            IndustryType::Healthcare => Self {
                current_assets_percent: dec!(35),
                cash_percent: dec!(20),
                ar_percent: dec!(50),
                inventory_percent: dec!(15),
                prepaid_percent: dec!(10),
                other_current_percent: dec!(5),
                ppe_percent: dec!(60),
                intangibles_percent: dec!(20),
                investments_percent: dec!(10),
                other_noncurrent_percent: dec!(10),
            },
            IndustryType::Utilities => Self {
                current_assets_percent: dec!(15),
                cash_percent: dec!(20),
                ar_percent: dec!(50),
                inventory_percent: dec!(15),
                prepaid_percent: dec!(10),
                other_current_percent: dec!(5),
                ppe_percent: dec!(85),
                intangibles_percent: dec!(5),
                investments_percent: dec!(5),
                other_noncurrent_percent: dec!(5),
            },
            IndustryType::RealEstate => Self {
                current_assets_percent: dec!(10),
                cash_percent: dec!(30),
                ar_percent: dec!(40),
                inventory_percent: dec!(10),
                prepaid_percent: dec!(10),
                other_current_percent: dec!(10),
                ppe_percent: dec!(90),
                intangibles_percent: dec!(3),
                investments_percent: dec!(5),
                other_noncurrent_percent: dec!(2),
            },
        }
    }

    /// Get total percentage (should be 100%).
    pub fn total_percentage(&self) -> Decimal {
        // Current assets composition should sum to 100%
        let current = self.cash_percent
            + self.ar_percent
            + self.inventory_percent
            + self.prepaid_percent
            + self.other_current_percent;

        // Non-current assets composition should sum to 100%
        let noncurrent = self.ppe_percent
            + self.intangibles_percent
            + self.investments_percent
            + self.other_noncurrent_percent;

        // Both should be approximately 100%
        if (current - dec!(100)).abs() > dec!(1) || (noncurrent - dec!(100)).abs() > dec!(1) {
            // Return a value that will fail validation
            current
        } else {
            dec!(100)
        }
    }
}

impl Default for AssetComposition {
    fn default() -> Self {
        Self::for_industry(IndustryType::Manufacturing)
    }
}

/// Capital structure specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalStructure {
    /// Total debt as percentage of total assets.
    pub debt_percent: Decimal,
    /// Total equity as percentage of total assets.
    pub equity_percent: Decimal,
    /// Current liabilities as % of total liabilities.
    pub current_liabilities_percent: Decimal,
    /// Long-term debt as % of total liabilities.
    pub long_term_debt_percent: Decimal,
    /// Other liabilities as % of total liabilities.
    pub other_liabilities_percent: Decimal,
    /// Common stock as % of equity.
    pub common_stock_percent: Decimal,
    /// Additional paid-in capital as % of equity.
    pub apic_percent: Decimal,
    /// Retained earnings as % of equity.
    pub retained_earnings_percent: Decimal,
    /// Other equity as % of equity.
    pub other_equity_percent: Decimal,
}

impl Default for CapitalStructure {
    fn default() -> Self {
        Self {
            debt_percent: dec!(40),
            equity_percent: dec!(60),
            current_liabilities_percent: dec!(50),
            long_term_debt_percent: dec!(40),
            other_liabilities_percent: dec!(10),
            common_stock_percent: dec!(15),
            apic_percent: dec!(25),
            retained_earnings_percent: dec!(55),
            other_equity_percent: dec!(5),
        }
    }
}

impl CapitalStructure {
    /// Get capital structure for a specific industry.
    pub fn for_industry(industry: IndustryType) -> Self {
        match industry {
            IndustryType::Manufacturing => Self {
                debt_percent: dec!(40),
                equity_percent: dec!(60),
                current_liabilities_percent: dec!(50),
                long_term_debt_percent: dec!(40),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(15),
                apic_percent: dec!(25),
                retained_earnings_percent: dec!(55),
                other_equity_percent: dec!(5),
            },
            IndustryType::Retail => Self {
                debt_percent: dec!(45),
                equity_percent: dec!(55),
                current_liabilities_percent: dec!(60),
                long_term_debt_percent: dec!(30),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(20),
                apic_percent: dec!(20),
                retained_earnings_percent: dec!(55),
                other_equity_percent: dec!(5),
            },
            IndustryType::Services => Self {
                debt_percent: dec!(30),
                equity_percent: dec!(70),
                current_liabilities_percent: dec!(55),
                long_term_debt_percent: dec!(35),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(15),
                apic_percent: dec!(30),
                retained_earnings_percent: dec!(50),
                other_equity_percent: dec!(5),
            },
            IndustryType::Technology => Self {
                debt_percent: dec!(25),
                equity_percent: dec!(75),
                current_liabilities_percent: dec!(60),
                long_term_debt_percent: dec!(30),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(10),
                apic_percent: dec!(40),
                retained_earnings_percent: dec!(45),
                other_equity_percent: dec!(5),
            },
            IndustryType::Financial => Self {
                debt_percent: dec!(70),
                equity_percent: dec!(30),
                current_liabilities_percent: dec!(70),
                long_term_debt_percent: dec!(20),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(25),
                apic_percent: dec!(35),
                retained_earnings_percent: dec!(35),
                other_equity_percent: dec!(5),
            },
            IndustryType::Healthcare => Self {
                debt_percent: dec!(35),
                equity_percent: dec!(65),
                current_liabilities_percent: dec!(50),
                long_term_debt_percent: dec!(40),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(15),
                apic_percent: dec!(30),
                retained_earnings_percent: dec!(50),
                other_equity_percent: dec!(5),
            },
            IndustryType::Utilities => Self {
                debt_percent: dec!(55),
                equity_percent: dec!(45),
                current_liabilities_percent: dec!(35),
                long_term_debt_percent: dec!(55),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(20),
                apic_percent: dec!(25),
                retained_earnings_percent: dec!(50),
                other_equity_percent: dec!(5),
            },
            IndustryType::RealEstate => Self {
                debt_percent: dec!(60),
                equity_percent: dec!(40),
                current_liabilities_percent: dec!(30),
                long_term_debt_percent: dec!(60),
                other_liabilities_percent: dec!(10),
                common_stock_percent: dec!(25),
                apic_percent: dec!(30),
                retained_earnings_percent: dec!(40),
                other_equity_percent: dec!(5),
            },
        }
    }

    /// Create capital structure with specific debt-to-equity ratio.
    pub fn with_debt_equity_ratio(ratio: Decimal) -> Self {
        // D/E = debt_percent / equity_percent
        // debt_percent + equity_percent = 100
        // debt_percent = ratio * equity_percent
        // ratio * equity_percent + equity_percent = 100
        // equity_percent = 100 / (1 + ratio)
        let equity_percent = dec!(100) / (Decimal::ONE + ratio);
        let debt_percent = dec!(100) - equity_percent;

        Self {
            debt_percent,
            equity_percent,
            ..Default::default()
        }
    }

    /// Get debt-to-equity ratio.
    pub fn debt_equity_ratio(&self) -> Decimal {
        if self.equity_percent > Decimal::ZERO {
            self.debt_percent / self.equity_percent
        } else {
            Decimal::MAX
        }
    }
}

/// Target financial ratios for opening balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetRatios {
    /// Current ratio (Current Assets / Current Liabilities).
    pub current_ratio: Decimal,
    /// Quick ratio ((Current Assets - Inventory) / Current Liabilities).
    pub quick_ratio: Decimal,
    /// Debt-to-equity ratio (Total Liabilities / Total Equity).
    pub debt_to_equity: Decimal,
    /// Asset turnover (Revenue / Total Assets) - for planning.
    pub asset_turnover: Decimal,
    /// Days Sales Outstanding (AR / Revenue * 365).
    pub target_dso_days: u32,
    /// Days Payable Outstanding (AP / COGS * 365).
    pub target_dpo_days: u32,
    /// Days Inventory Outstanding (Inventory / COGS * 365).
    pub target_dio_days: u32,
    /// Gross margin ((Revenue - COGS) / Revenue).
    pub gross_margin: Decimal,
    /// Operating margin (Operating Income / Revenue).
    pub operating_margin: Decimal,
}

impl TargetRatios {
    /// Get target ratios for a specific industry.
    pub fn for_industry(industry: IndustryType) -> Self {
        match industry {
            IndustryType::Manufacturing => Self {
                current_ratio: dec!(1.5),
                quick_ratio: dec!(0.8),
                debt_to_equity: dec!(0.6),
                asset_turnover: dec!(1.2),
                target_dso_days: 45,
                target_dpo_days: 35,
                target_dio_days: 60,
                gross_margin: dec!(0.35),
                operating_margin: dec!(0.12),
            },
            IndustryType::Retail => Self {
                current_ratio: dec!(1.2),
                quick_ratio: dec!(0.4),
                debt_to_equity: dec!(0.8),
                asset_turnover: dec!(2.5),
                target_dso_days: 15,
                target_dpo_days: 30,
                target_dio_days: 45,
                gross_margin: dec!(0.30),
                operating_margin: dec!(0.08),
            },
            IndustryType::Services => Self {
                current_ratio: dec!(1.8),
                quick_ratio: dec!(1.6),
                debt_to_equity: dec!(0.4),
                asset_turnover: dec!(1.5),
                target_dso_days: 60,
                target_dpo_days: 25,
                target_dio_days: 0,
                gross_margin: dec!(0.45),
                operating_margin: dec!(0.18),
            },
            IndustryType::Technology => Self {
                current_ratio: dec!(2.5),
                quick_ratio: dec!(2.3),
                debt_to_equity: dec!(0.3),
                asset_turnover: dec!(0.8),
                target_dso_days: 55,
                target_dpo_days: 40,
                target_dio_days: 15,
                gross_margin: dec!(0.65),
                operating_margin: dec!(0.25),
            },
            IndustryType::Financial => Self {
                current_ratio: dec!(1.1),
                quick_ratio: dec!(1.1),
                debt_to_equity: dec!(2.0),
                asset_turnover: dec!(0.3),
                target_dso_days: 30,
                target_dpo_days: 20,
                target_dio_days: 0,
                gross_margin: dec!(0.80),
                operating_margin: dec!(0.30),
            },
            IndustryType::Healthcare => Self {
                current_ratio: dec!(1.4),
                quick_ratio: dec!(1.1),
                debt_to_equity: dec!(0.5),
                asset_turnover: dec!(1.0),
                target_dso_days: 50,
                target_dpo_days: 30,
                target_dio_days: 30,
                gross_margin: dec!(0.40),
                operating_margin: dec!(0.15),
            },
            IndustryType::Utilities => Self {
                current_ratio: dec!(0.9),
                quick_ratio: dec!(0.7),
                debt_to_equity: dec!(1.2),
                asset_turnover: dec!(0.4),
                target_dso_days: 40,
                target_dpo_days: 45,
                target_dio_days: 20,
                gross_margin: dec!(0.35),
                operating_margin: dec!(0.20),
            },
            IndustryType::RealEstate => Self {
                current_ratio: dec!(1.0),
                quick_ratio: dec!(0.8),
                debt_to_equity: dec!(1.5),
                asset_turnover: dec!(0.2),
                target_dso_days: 30,
                target_dpo_days: 25,
                target_dio_days: 0,
                gross_margin: dec!(0.50),
                operating_margin: dec!(0.35),
            },
        }
    }

    /// Calculate target AR balance from revenue.
    pub fn calculate_target_ar(&self, annual_revenue: Decimal) -> Decimal {
        annual_revenue * Decimal::from(self.target_dso_days) / dec!(365)
    }

    /// Calculate target AP balance from COGS.
    pub fn calculate_target_ap(&self, annual_cogs: Decimal) -> Decimal {
        annual_cogs * Decimal::from(self.target_dpo_days) / dec!(365)
    }

    /// Calculate target inventory balance from COGS.
    pub fn calculate_target_inventory(&self, annual_cogs: Decimal) -> Decimal {
        annual_cogs * Decimal::from(self.target_dio_days) / dec!(365)
    }
}

impl Default for TargetRatios {
    fn default() -> Self {
        Self::for_industry(IndustryType::Manufacturing)
    }
}

/// Individual account specification override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSpec {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub description: String,
    /// Account type.
    pub account_type: AccountType,
    /// Category.
    pub category: AccountCategory,
    /// Fixed balance amount (overrides calculated).
    pub fixed_balance: Option<Decimal>,
    /// Percentage of category total.
    pub category_percent: Option<Decimal>,
    /// Percentage of total assets.
    pub total_assets_percent: Option<Decimal>,
}

/// Generated opening balance result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedOpeningBalance {
    /// Company code.
    pub company_code: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Individual account balances.
    pub balances: HashMap<String, Decimal>,
    /// Total assets.
    pub total_assets: Decimal,
    /// Total liabilities.
    pub total_liabilities: Decimal,
    /// Total equity.
    pub total_equity: Decimal,
    /// Is balanced (A = L + E)?
    pub is_balanced: bool,
    /// Calculated ratios.
    pub calculated_ratios: CalculatedRatios,
}

/// Calculated ratios from generated balances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedRatios {
    /// Current ratio.
    pub current_ratio: Option<Decimal>,
    /// Quick ratio.
    pub quick_ratio: Option<Decimal>,
    /// Debt-to-equity ratio.
    pub debt_to_equity: Option<Decimal>,
    /// Working capital.
    pub working_capital: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_balance_spec_creation() {
        let spec = OpeningBalanceSpec::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            2022,
            "USD".to_string(),
            dec!(1000000),
            IndustryType::Manufacturing,
        );

        assert!(spec.validate().is_ok());
        assert_eq!(spec.calculate_total_liabilities(), dec!(400000)); // 40%
        assert_eq!(spec.calculate_total_equity(), dec!(600000)); // 60%
    }

    #[test]
    fn test_capital_structure_debt_equity() {
        let structure = CapitalStructure::with_debt_equity_ratio(dec!(0.5));

        // D/E = 0.5, so D = 0.5E, D + E = 100
        // 0.5E + E = 100 -> 1.5E = 100 -> E = 66.67
        assert!((structure.equity_percent - dec!(66.67)).abs() < dec!(0.01));
        assert!((structure.debt_percent - dec!(33.33)).abs() < dec!(0.01));
        assert!((structure.debt_equity_ratio() - dec!(0.5)).abs() < dec!(0.01));
    }

    #[test]
    fn test_asset_composition_for_industries() {
        let manufacturing = AssetComposition::for_industry(IndustryType::Manufacturing);
        assert_eq!(manufacturing.current_assets_percent, dec!(40));

        let retail = AssetComposition::for_industry(IndustryType::Retail);
        assert_eq!(retail.current_assets_percent, dec!(55));
        assert!(retail.inventory_percent > manufacturing.inventory_percent);

        let technology = AssetComposition::for_industry(IndustryType::Technology);
        assert!(technology.intangibles_percent > manufacturing.intangibles_percent);
    }

    #[test]
    fn test_target_ratios_calculations() {
        let ratios = TargetRatios::for_industry(IndustryType::Manufacturing);

        let annual_revenue = dec!(1000000);
        let annual_cogs = dec!(650000); // 35% gross margin

        let target_ar = ratios.calculate_target_ar(annual_revenue);
        // 1,000,000 * 45 / 365 ≈ 123,288
        assert!(target_ar > dec!(120000) && target_ar < dec!(130000));

        let target_inventory = ratios.calculate_target_inventory(annual_cogs);
        // 650,000 * 60 / 365 ≈ 106,849
        assert!(target_inventory > dec!(100000) && target_inventory < dec!(115000));
    }

    #[test]
    fn test_opening_balance_validation() {
        let mut spec = OpeningBalanceSpec::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            2022,
            "USD".to_string(),
            dec!(1000000),
            IndustryType::Manufacturing,
        );

        // Valid spec
        assert!(spec.validate().is_ok());

        // Invalid capital structure
        spec.capital_structure.debt_percent = dec!(80);
        spec.capital_structure.equity_percent = dec!(30); // Sums to 110%
        assert!(spec.validate().is_err());
    }
}
