//! Fair Value Measurement Models (ASC 820 / IFRS 13).
//!
//! Implements fair value measurement concepts for financial reporting:
//!
//! - Fair value hierarchy (Level 1, 2, 3 inputs)
//! - Valuation techniques
//! - Fair value disclosures
//!
//! Both ASC 820 and IFRS 13 are substantially converged, with
//! largely consistent requirements for fair value measurement.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::framework::AccountingFramework;

/// Fair value measurement record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueMeasurement {
    /// Unique measurement identifier.
    pub measurement_id: Uuid,

    /// Asset or liability being measured.
    pub item_id: String,

    /// Description of the item.
    pub item_description: String,

    /// Category of measurement.
    pub item_category: FairValueCategory,

    /// Fair value hierarchy level.
    pub hierarchy_level: FairValueHierarchyLevel,

    /// Valuation technique used.
    pub valuation_technique: ValuationTechnique,

    /// Measured fair value.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value: Decimal,

    /// Carrying amount (if different from fair value).
    #[serde(with = "rust_decimal::serde::str")]
    pub carrying_amount: Decimal,

    /// Measurement date.
    pub measurement_date: chrono::NaiveDate,

    /// Currency.
    pub currency: String,

    /// Key inputs used in valuation.
    pub valuation_inputs: Vec<ValuationInput>,

    /// Whether this is a recurring or non-recurring measurement.
    pub measurement_type: MeasurementType,

    /// Framework applied.
    pub framework: AccountingFramework,

    /// Sensitivity analysis (for Level 3 measurements).
    pub sensitivity_analysis: Option<SensitivityAnalysis>,
}

impl FairValueMeasurement {
    /// Create a new fair value measurement.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        item_id: impl Into<String>,
        item_description: impl Into<String>,
        item_category: FairValueCategory,
        hierarchy_level: FairValueHierarchyLevel,
        fair_value: Decimal,
        measurement_date: chrono::NaiveDate,
        currency: impl Into<String>,
        framework: AccountingFramework,
    ) -> Self {
        Self {
            measurement_id: Uuid::now_v7(),
            item_id: item_id.into(),
            item_description: item_description.into(),
            item_category,
            hierarchy_level,
            valuation_technique: ValuationTechnique::default(),
            fair_value,
            carrying_amount: fair_value,
            measurement_date,
            currency: currency.into(),
            valuation_inputs: Vec::new(),
            measurement_type: MeasurementType::Recurring,
            framework,
            sensitivity_analysis: None,
        }
    }

    /// Add a valuation input.
    pub fn add_input(&mut self, input: ValuationInput) {
        self.valuation_inputs.push(input);
    }

    /// Calculate unrealized gain/loss.
    pub fn unrealized_gain_loss(&self) -> Decimal {
        self.fair_value - self.carrying_amount
    }
}

/// Fair value hierarchy level.
///
/// The fair value hierarchy prioritizes the inputs to valuation techniques:
/// - Level 1: Quoted prices in active markets (most reliable)
/// - Level 2: Observable inputs other than Level 1 prices
/// - Level 3: Unobservable inputs (requires more judgment)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FairValueHierarchyLevel {
    /// Quoted prices in active markets for identical assets/liabilities.
    ///
    /// Examples: Exchange-traded securities, commodities with active markets.
    #[default]
    Level1,

    /// Observable inputs other than Level 1 prices.
    ///
    /// Examples: Quoted prices for similar items, interest rates, yield curves.
    Level2,

    /// Unobservable inputs based on entity's assumptions.
    ///
    /// Examples: Discounted cash flow using internal projections,
    /// privately held investments, complex derivatives.
    Level3,
}

impl std::fmt::Display for FairValueHierarchyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Level1 => write!(f, "Level 1"),
            Self::Level2 => write!(f, "Level 2"),
            Self::Level3 => write!(f, "Level 3"),
        }
    }
}

/// Category of item being measured at fair value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FairValueCategory {
    /// Trading securities.
    #[default]
    TradingSecurities,
    /// Available-for-sale securities (US GAAP) / FVOCI (IFRS).
    AvailableForSale,
    /// Derivative financial instruments.
    Derivatives,
    /// Investment property (IFRS fair value model).
    InvestmentProperty,
    /// Biological assets.
    BiologicalAssets,
    /// Pension plan assets.
    PensionAssets,
    /// Contingent consideration (business combinations).
    ContingentConsideration,
    /// Impaired assets.
    ImpairedAssets,
    /// Other fair value items.
    Other,
}

impl std::fmt::Display for FairValueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TradingSecurities => write!(f, "Trading Securities"),
            Self::AvailableForSale => write!(f, "Available-for-Sale Securities"),
            Self::Derivatives => write!(f, "Derivatives"),
            Self::InvestmentProperty => write!(f, "Investment Property"),
            Self::BiologicalAssets => write!(f, "Biological Assets"),
            Self::PensionAssets => write!(f, "Pension Plan Assets"),
            Self::ContingentConsideration => write!(f, "Contingent Consideration"),
            Self::ImpairedAssets => write!(f, "Impaired Assets"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Valuation technique used in fair value measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValuationTechnique {
    /// Market approach - uses prices from market transactions.
    #[default]
    MarketApproach,
    /// Income approach - converts future amounts to present value.
    IncomeApproach,
    /// Cost approach - current replacement cost.
    CostApproach,
    /// Combination of multiple approaches.
    MultipleApproaches,
}

impl std::fmt::Display for ValuationTechnique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MarketApproach => write!(f, "Market Approach"),
            Self::IncomeApproach => write!(f, "Income Approach"),
            Self::CostApproach => write!(f, "Cost Approach"),
            Self::MultipleApproaches => write!(f, "Multiple Approaches"),
        }
    }
}

/// Type of fair value measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementType {
    /// Recurring measurement each reporting period.
    #[default]
    Recurring,
    /// Non-recurring measurement (e.g., impairment).
    NonRecurring,
}

/// Valuation input used in fair value measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationInput {
    /// Input name/description.
    pub name: String,

    /// Input value.
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,

    /// Unit of measurement.
    pub unit: String,

    /// Whether this is an observable or unobservable input.
    pub observable: bool,

    /// Source of the input.
    pub source: String,
}

impl ValuationInput {
    /// Create a new valuation input.
    pub fn new(
        name: impl Into<String>,
        value: Decimal,
        unit: impl Into<String>,
        observable: bool,
        source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            unit: unit.into(),
            observable,
            source: source.into(),
        }
    }

    /// Create a discount rate input.
    pub fn discount_rate(rate: Decimal, source: impl Into<String>) -> Self {
        Self::new("Discount Rate", rate, "%", true, source)
    }

    /// Create an expected growth rate input.
    pub fn growth_rate(rate: Decimal, source: impl Into<String>) -> Self {
        Self::new("Expected Growth Rate", rate, "%", false, source)
    }
}

/// Sensitivity analysis for Level 3 measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityAnalysis {
    /// Primary unobservable input.
    pub input_name: String,

    /// Range tested (low, high).
    #[serde(with = "decimal_tuple")]
    pub input_range: (Decimal, Decimal),

    /// Fair value at low end of range.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_low: Decimal,

    /// Fair value at high end of range.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_high: Decimal,

    /// Correlation with other inputs.
    pub correlated_inputs: Vec<String>,
}

mod decimal_tuple {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &(Decimal, Decimal), serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let tuple = (value.0.to_string(), value.1.to_string());
        tuple.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(Decimal, Decimal), D::Error>
    where
        D: Deserializer<'de>,
    {
        let tuple: (String, String) = Deserialize::deserialize(deserializer)?;
        let low = tuple
            .0
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid decimal"))?;
        let high = tuple
            .1
            .parse()
            .map_err(|_| serde::de::Error::custom("invalid decimal"))?;
        Ok((low, high))
    }
}

/// Fair value hierarchy summary for disclosure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueHierarchySummary {
    /// Reporting period end date.
    pub period_date: chrono::NaiveDate,

    /// Company code.
    pub company_code: String,

    /// Total Level 1 assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub level1_assets: Decimal,

    /// Total Level 2 assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub level2_assets: Decimal,

    /// Total Level 3 assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub level3_assets: Decimal,

    /// Total Level 1 liabilities.
    #[serde(with = "rust_decimal::serde::str")]
    pub level1_liabilities: Decimal,

    /// Total Level 2 liabilities.
    #[serde(with = "rust_decimal::serde::str")]
    pub level2_liabilities: Decimal,

    /// Total Level 3 liabilities.
    #[serde(with = "rust_decimal::serde::str")]
    pub level3_liabilities: Decimal,

    /// Framework applied.
    pub framework: AccountingFramework,
}

impl FairValueHierarchySummary {
    /// Create a new summary.
    pub fn new(
        period_date: chrono::NaiveDate,
        company_code: impl Into<String>,
        framework: AccountingFramework,
    ) -> Self {
        Self {
            period_date,
            company_code: company_code.into(),
            level1_assets: Decimal::ZERO,
            level2_assets: Decimal::ZERO,
            level3_assets: Decimal::ZERO,
            level1_liabilities: Decimal::ZERO,
            level2_liabilities: Decimal::ZERO,
            level3_liabilities: Decimal::ZERO,
            framework,
        }
    }

    /// Total fair value assets.
    pub fn total_assets(&self) -> Decimal {
        self.level1_assets + self.level2_assets + self.level3_assets
    }

    /// Total fair value liabilities.
    pub fn total_liabilities(&self) -> Decimal {
        self.level1_liabilities + self.level2_liabilities + self.level3_liabilities
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fair_value_measurement() {
        let measurement = FairValueMeasurement::new(
            "SEC001",
            "ABC Corp Common Stock",
            FairValueCategory::TradingSecurities,
            FairValueHierarchyLevel::Level1,
            dec!(50000),
            chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            "USD",
            AccountingFramework::UsGaap,
        );

        assert_eq!(measurement.fair_value, dec!(50000));
        assert_eq!(measurement.hierarchy_level, FairValueHierarchyLevel::Level1);
    }

    #[test]
    fn test_unrealized_gain_loss() {
        let mut measurement = FairValueMeasurement::new(
            "SEC001",
            "XYZ Corp Stock",
            FairValueCategory::TradingSecurities,
            FairValueHierarchyLevel::Level1,
            dec!(55000), // Current fair value
            chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            "USD",
            AccountingFramework::UsGaap,
        );
        measurement.carrying_amount = dec!(50000); // Original cost

        assert_eq!(measurement.unrealized_gain_loss(), dec!(5000));
    }

    #[test]
    fn test_hierarchy_summary() {
        let mut summary = FairValueHierarchySummary::new(
            chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            "1000",
            AccountingFramework::UsGaap,
        );

        summary.level1_assets = dec!(100000);
        summary.level2_assets = dec!(50000);
        summary.level3_assets = dec!(25000);

        assert_eq!(summary.total_assets(), dec!(175000));
    }
}
