//! Asset Impairment Models (ASC 360 / IAS 36).
//!
//! Implements impairment testing for long-lived assets:
//!
//! - Impairment indicators (triggering events)
//! - Recoverability tests
//! - Fair value less costs to sell
//! - Value in use calculations
//!
//! Key differences between frameworks:
//! - US GAAP (ASC 360): Two-step test (recoverability then measurement)
//! - IFRS (IAS 36): One-step test (recoverable amount)
//! - IFRS allows reversal of impairment losses (except goodwill)

use chrono::NaiveDate;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::framework::AccountingFramework;

/// Impairment test record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpairmentTest {
    /// Unique test identifier.
    pub test_id: Uuid,

    /// Company code.
    pub company_code: String,

    /// Asset or cash-generating unit being tested.
    pub asset_id: String,

    /// Asset description.
    pub asset_description: String,

    /// Type of asset.
    pub asset_type: ImpairmentAssetType,

    /// Test date.
    pub test_date: NaiveDate,

    /// Carrying amount before impairment.
    #[serde(with = "rust_decimal::serde::str")]
    pub carrying_amount: Decimal,

    /// Recoverable amount (higher of fair value less costs to sell and value in use).
    #[serde(with = "rust_decimal::serde::str")]
    pub recoverable_amount: Decimal,

    /// Fair value less costs to sell.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_less_costs: Decimal,

    /// Value in use (present value of future cash flows).
    #[serde(with = "rust_decimal::serde::str")]
    pub value_in_use: Decimal,

    /// Impairment loss recognized.
    #[serde(with = "rust_decimal::serde::str")]
    pub impairment_loss: Decimal,

    /// Indicators that triggered the test.
    pub impairment_indicators: Vec<ImpairmentIndicator>,

    /// Test result/conclusion.
    pub test_result: ImpairmentTestResult,

    /// Framework applied.
    pub framework: AccountingFramework,

    /// For US GAAP: Undiscounted cash flows for Step 1.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub undiscounted_cash_flows: Option<Decimal>,

    /// Discount rate used for value in use.
    #[serde(with = "rust_decimal::serde::str")]
    pub discount_rate: Decimal,

    /// Cash flow projections used.
    pub cash_flow_projections: Vec<CashFlowProjection>,

    /// Reference to journal entry recording impairment.
    pub journal_entry_id: Option<Uuid>,
}

impl ImpairmentTest {
    /// Create a new impairment test.
    pub fn new(
        company_code: impl Into<String>,
        asset_id: impl Into<String>,
        asset_description: impl Into<String>,
        asset_type: ImpairmentAssetType,
        test_date: NaiveDate,
        carrying_amount: Decimal,
        framework: AccountingFramework,
    ) -> Self {
        Self {
            test_id: Uuid::now_v7(),
            company_code: company_code.into(),
            asset_id: asset_id.into(),
            asset_description: asset_description.into(),
            asset_type,
            test_date,
            carrying_amount,
            recoverable_amount: Decimal::ZERO,
            fair_value_less_costs: Decimal::ZERO,
            value_in_use: Decimal::ZERO,
            impairment_loss: Decimal::ZERO,
            impairment_indicators: Vec::new(),
            test_result: ImpairmentTestResult::NotImpaired,
            framework,
            undiscounted_cash_flows: None,
            discount_rate: Decimal::ZERO,
            cash_flow_projections: Vec::new(),
            journal_entry_id: None,
        }
    }

    /// Add an impairment indicator.
    pub fn add_indicator(&mut self, indicator: ImpairmentIndicator) {
        self.impairment_indicators.push(indicator);
    }

    /// Perform impairment test based on framework.
    pub fn perform_test(&mut self) {
        match self.framework {
            AccountingFramework::UsGaap => self.perform_us_gaap_test(),
            AccountingFramework::Ifrs | AccountingFramework::DualReporting => {
                self.perform_ifrs_test()
            }
        }
    }

    /// US GAAP two-step impairment test (ASC 360).
    fn perform_us_gaap_test(&mut self) {
        // Step 1: Recoverability test
        // Compare carrying amount to undiscounted cash flows
        if let Some(undiscounted) = self.undiscounted_cash_flows {
            if self.carrying_amount <= undiscounted {
                // Asset is recoverable, no impairment
                self.test_result = ImpairmentTestResult::NotImpaired;
                self.impairment_loss = Decimal::ZERO;
                return;
            }
        }

        // Step 2: Measurement (if Step 1 failed)
        // Impairment = Carrying Amount - Fair Value
        self.recoverable_amount = self.fair_value_less_costs;
        self.impairment_loss = (self.carrying_amount - self.recoverable_amount).max(Decimal::ZERO);

        self.test_result = if self.impairment_loss > Decimal::ZERO {
            ImpairmentTestResult::Impaired
        } else {
            ImpairmentTestResult::NotImpaired
        };
    }

    /// IFRS one-step impairment test (IAS 36).
    fn perform_ifrs_test(&mut self) {
        // Recoverable amount = higher of fair value less costs to sell and value in use
        self.recoverable_amount = self.fair_value_less_costs.max(self.value_in_use);

        // Impairment loss = Carrying Amount - Recoverable Amount
        self.impairment_loss = (self.carrying_amount - self.recoverable_amount).max(Decimal::ZERO);

        self.test_result = if self.impairment_loss > Decimal::ZERO {
            ImpairmentTestResult::Impaired
        } else {
            ImpairmentTestResult::NotImpaired
        };
    }

    /// Calculate value in use from cash flow projections.
    pub fn calculate_value_in_use(&mut self) {
        let mut viu = Decimal::ZERO;

        for projection in &self.cash_flow_projections {
            let discount_factor = Decimal::ONE
                / (Decimal::ONE + self.discount_rate).powd(Decimal::from(projection.year as i64));
            viu += projection.net_cash_flow * discount_factor;
        }

        self.value_in_use = viu;
    }

    /// Calculate undiscounted cash flows (for US GAAP Step 1).
    pub fn calculate_undiscounted_cash_flows(&mut self) {
        let undiscounted: Decimal = self
            .cash_flow_projections
            .iter()
            .map(|p| p.net_cash_flow)
            .sum();

        self.undiscounted_cash_flows = Some(undiscounted);
    }
}

/// Type of asset subject to impairment testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImpairmentAssetType {
    /// Property, plant, and equipment.
    #[default]
    PropertyPlantEquipment,
    /// Intangible assets with finite lives.
    IntangibleFinite,
    /// Intangible assets with indefinite lives.
    IntangibleIndefinite,
    /// Goodwill.
    Goodwill,
    /// Right-of-use assets (leases).
    RightOfUseAsset,
    /// Equity method investments.
    EquityInvestment,
    /// Cash-generating unit (group of assets).
    CashGeneratingUnit,
}

impl std::fmt::Display for ImpairmentAssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PropertyPlantEquipment => write!(f, "Property, Plant & Equipment"),
            Self::IntangibleFinite => write!(f, "Intangible Asset (Finite Life)"),
            Self::IntangibleIndefinite => write!(f, "Intangible Asset (Indefinite Life)"),
            Self::Goodwill => write!(f, "Goodwill"),
            Self::RightOfUseAsset => write!(f, "Right-of-Use Asset"),
            Self::EquityInvestment => write!(f, "Equity Method Investment"),
            Self::CashGeneratingUnit => write!(f, "Cash-Generating Unit"),
        }
    }
}

/// Impairment indicator (triggering event).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpairmentIndicator {
    // External indicators
    /// Significant decline in market value.
    MarketValueDecline,
    /// Adverse changes in technology, markets, economy, or legal environment.
    AdverseEnvironmentChanges,
    /// Increase in market interest rates.
    InterestRateIncrease,
    /// Market capitalization below book value.
    MarketCapBelowBookValue,

    // Internal indicators
    /// Evidence of obsolescence or physical damage.
    ObsolescenceOrDamage,
    /// Significant adverse changes in use or expected use.
    AdverseUseChanges,
    /// Operating losses or negative cash flows.
    OperatingLosses,
    /// Plans to discontinue or restructure operations.
    DiscontinuationPlans,
    /// Asset expected to be disposed of before end of useful life.
    EarlyDisposal,
    /// Worse than expected performance.
    WorsePerformance,

    // Goodwill-specific
    /// Loss of key personnel.
    KeyPersonnelLoss,
    /// Loss of major customer.
    MajorCustomerLoss,
    /// Significant competition increase.
    CompetitionIncrease,
    /// Regulatory changes.
    RegulatoryChanges,

    // Annual testing (no trigger needed)
    /// Mandatory annual test (goodwill, indefinite-life intangibles).
    AnnualTest,
}

impl std::fmt::Display for ImpairmentIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MarketValueDecline => write!(f, "Market Value Decline"),
            Self::AdverseEnvironmentChanges => write!(f, "Adverse Environment Changes"),
            Self::InterestRateIncrease => write!(f, "Interest Rate Increase"),
            Self::MarketCapBelowBookValue => write!(f, "Market Cap Below Book Value"),
            Self::ObsolescenceOrDamage => write!(f, "Obsolescence or Damage"),
            Self::AdverseUseChanges => write!(f, "Adverse Use Changes"),
            Self::OperatingLosses => write!(f, "Operating Losses"),
            Self::DiscontinuationPlans => write!(f, "Discontinuation Plans"),
            Self::EarlyDisposal => write!(f, "Early Disposal"),
            Self::WorsePerformance => write!(f, "Worse Performance"),
            Self::KeyPersonnelLoss => write!(f, "Key Personnel Loss"),
            Self::MajorCustomerLoss => write!(f, "Major Customer Loss"),
            Self::CompetitionIncrease => write!(f, "Competition Increase"),
            Self::RegulatoryChanges => write!(f, "Regulatory Changes"),
            Self::AnnualTest => write!(f, "Annual Test"),
        }
    }
}

/// Impairment test result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImpairmentTestResult {
    /// Asset is not impaired.
    #[default]
    NotImpaired,
    /// Asset is impaired.
    Impaired,
    /// Impairment loss reversed (IFRS only, except goodwill).
    ReversalRecognized,
}

impl std::fmt::Display for ImpairmentTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImpaired => write!(f, "Not Impaired"),
            Self::Impaired => write!(f, "Impaired"),
            Self::ReversalRecognized => write!(f, "Reversal Recognized"),
        }
    }
}

/// Cash flow projection for value in use calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowProjection {
    /// Projection year (1, 2, 3, etc.).
    pub year: u32,

    /// Projected revenue.
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue: Decimal,

    /// Projected operating expenses.
    #[serde(with = "rust_decimal::serde::str")]
    pub operating_expenses: Decimal,

    /// Projected capital expenditures.
    #[serde(with = "rust_decimal::serde::str")]
    pub capital_expenditures: Decimal,

    /// Net cash flow.
    #[serde(with = "rust_decimal::serde::str")]
    pub net_cash_flow: Decimal,

    /// Growth rate assumption.
    #[serde(with = "rust_decimal::serde::str")]
    pub growth_rate: Decimal,

    /// Is this a terminal value projection.
    pub is_terminal_value: bool,
}

impl CashFlowProjection {
    /// Create a new cash flow projection.
    pub fn new(year: u32, revenue: Decimal, operating_expenses: Decimal) -> Self {
        let net_cash_flow = revenue - operating_expenses;
        Self {
            year,
            revenue,
            operating_expenses,
            capital_expenditures: Decimal::ZERO,
            net_cash_flow,
            growth_rate: Decimal::ZERO,
            is_terminal_value: false,
        }
    }

    /// Calculate net cash flow.
    pub fn calculate_net_cash_flow(&mut self) {
        self.net_cash_flow = self.revenue - self.operating_expenses - self.capital_expenditures;
    }
}

/// Impairment loss reversal (IFRS only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpairmentReversal {
    /// Unique identifier.
    pub reversal_id: Uuid,

    /// Original impairment test ID.
    pub original_test_id: Uuid,

    /// Asset ID.
    pub asset_id: String,

    /// Reversal date.
    pub reversal_date: NaiveDate,

    /// Carrying amount before reversal.
    #[serde(with = "rust_decimal::serde::str")]
    pub carrying_amount_before: Decimal,

    /// Reversal amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub reversal_amount: Decimal,

    /// Carrying amount after reversal.
    #[serde(with = "rust_decimal::serde::str")]
    pub carrying_amount_after: Decimal,

    /// Maximum carrying amount (what it would have been without impairment).
    #[serde(with = "rust_decimal::serde::str")]
    pub maximum_carrying_amount: Decimal,

    /// Reason for reversal.
    pub reversal_reason: String,

    /// Reference to journal entry.
    pub journal_entry_id: Option<Uuid>,
}

impl ImpairmentReversal {
    /// Check if reversal is valid (IFRS rules).
    pub fn is_valid(&self, asset_type: ImpairmentAssetType) -> bool {
        // Goodwill impairment cannot be reversed
        if asset_type == ImpairmentAssetType::Goodwill {
            return false;
        }

        // Reversal cannot exceed what carrying amount would have been
        self.carrying_amount_after <= self.maximum_carrying_amount
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_impairment_test_creation() {
        let test = ImpairmentTest::new(
            "1000",
            "FA001",
            "Manufacturing Equipment",
            ImpairmentAssetType::PropertyPlantEquipment,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(500000),
            AccountingFramework::UsGaap,
        );

        assert_eq!(test.carrying_amount, dec!(500000));
        assert_eq!(test.test_result, ImpairmentTestResult::NotImpaired);
    }

    #[test]
    fn test_us_gaap_impairment_no_impairment() {
        let mut test = ImpairmentTest::new(
            "1000",
            "FA001",
            "Manufacturing Equipment",
            ImpairmentAssetType::PropertyPlantEquipment,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(500000),
            AccountingFramework::UsGaap,
        );

        // Undiscounted cash flows exceed carrying amount - no impairment
        test.undiscounted_cash_flows = Some(dec!(600000));
        test.fair_value_less_costs = dec!(450000);

        test.perform_test();

        assert_eq!(test.test_result, ImpairmentTestResult::NotImpaired);
        assert_eq!(test.impairment_loss, dec!(0));
    }

    #[test]
    fn test_us_gaap_impairment_recognized() {
        let mut test = ImpairmentTest::new(
            "1000",
            "FA001",
            "Manufacturing Equipment",
            ImpairmentAssetType::PropertyPlantEquipment,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(500000),
            AccountingFramework::UsGaap,
        );

        // Undiscounted cash flows less than carrying amount - proceed to Step 2
        test.undiscounted_cash_flows = Some(dec!(400000));
        test.fair_value_less_costs = dec!(350000);

        test.perform_test();

        assert_eq!(test.test_result, ImpairmentTestResult::Impaired);
        assert_eq!(test.impairment_loss, dec!(150000)); // 500000 - 350000
    }

    #[test]
    fn test_ifrs_impairment() {
        let mut test = ImpairmentTest::new(
            "1000",
            "FA001",
            "Manufacturing Equipment",
            ImpairmentAssetType::PropertyPlantEquipment,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(500000),
            AccountingFramework::Ifrs,
        );

        test.fair_value_less_costs = dec!(350000);
        test.value_in_use = dec!(380000);

        test.perform_test();

        // Recoverable amount = max(350000, 380000) = 380000
        assert_eq!(test.recoverable_amount, dec!(380000));
        assert_eq!(test.impairment_loss, dec!(120000)); // 500000 - 380000
        assert_eq!(test.test_result, ImpairmentTestResult::Impaired);
    }

    #[test]
    fn test_value_in_use_calculation() {
        let mut test = ImpairmentTest::new(
            "1000",
            "FA001",
            "Manufacturing Equipment",
            ImpairmentAssetType::PropertyPlantEquipment,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(500000),
            AccountingFramework::Ifrs,
        );

        test.discount_rate = dec!(0.10);
        test.cash_flow_projections = vec![
            CashFlowProjection::new(1, dec!(200000), dec!(100000)),
            CashFlowProjection::new(2, dec!(200000), dec!(100000)),
            CashFlowProjection::new(3, dec!(200000), dec!(100000)),
        ];

        test.calculate_value_in_use();

        // PV of 100,000 for 3 years at 10%
        // Year 1: 100000 / 1.10 = 90909
        // Year 2: 100000 / 1.21 = 82645
        // Year 3: 100000 / 1.331 = 75131
        // Total ≈ 248,685
        assert!(test.value_in_use > dec!(240000));
        assert!(test.value_in_use < dec!(260000));
    }
}
