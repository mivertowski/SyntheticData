//! Fixed asset master data model.
//!
//! Provides fixed asset master data including depreciation schedules
//! for realistic fixed asset accounting simulation.

use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// MACRS GDS half-year convention percentages (IRS Publication 946).
/// Stored as string slices so they convert to `Decimal` without floating-point artefacts.
const MACRS_GDS_3_YEAR: &[&str] = &["33.33", "44.45", "14.81", "7.41"];
const MACRS_GDS_5_YEAR: &[&str] = &["20.00", "32.00", "19.20", "11.52", "11.52", "5.76"];
const MACRS_GDS_7_YEAR: &[&str] = &[
    "14.29", "24.49", "17.49", "12.49", "8.93", "8.92", "8.93", "4.46",
];
const MACRS_GDS_10_YEAR: &[&str] = &[
    "10.00", "18.00", "14.40", "11.52", "9.22", "7.37", "6.55", "6.55", "6.56", "6.55", "3.28",
];
const MACRS_GDS_15_YEAR: &[&str] = &[
    "5.00", "9.50", "8.55", "7.70", "6.93", "6.23", "5.90", "5.90", "5.91", "5.90", "5.91", "5.90",
    "5.91", "5.90", "5.91", "2.95",
];
const MACRS_GDS_20_YEAR: &[&str] = &[
    "3.750", "7.219", "6.677", "6.177", "5.713", "5.285", "4.888", "4.522", "4.462", "4.461",
    "4.462", "4.461", "4.462", "4.461", "4.462", "4.461", "4.462", "4.461", "4.462", "4.461",
    "2.231",
];

/// Map useful life in years to the appropriate MACRS GDS depreciation table.
fn macrs_table_for_life(useful_life_years: u32) -> Option<&'static [&'static str]> {
    match useful_life_years {
        1..=3 => Some(MACRS_GDS_3_YEAR),
        4..=5 => Some(MACRS_GDS_5_YEAR),
        6..=7 => Some(MACRS_GDS_7_YEAR),
        8..=10 => Some(MACRS_GDS_10_YEAR),
        11..=15 => Some(MACRS_GDS_15_YEAR),
        16..=20 => Some(MACRS_GDS_20_YEAR),
        _ => None,
    }
}

/// Parse a MACRS percentage string into a `Decimal`.
fn macrs_pct(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

/// Asset class for categorization and default depreciation rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    /// Buildings and structures
    Buildings,
    /// Building improvements
    BuildingImprovements,
    /// Land (typically non-depreciable)
    Land,
    /// Machinery and equipment
    #[default]
    MachineryEquipment,
    /// Machinery (alias for MachineryEquipment)
    Machinery,
    /// Computer hardware
    ComputerHardware,
    /// IT Equipment (alias for ComputerHardware)
    ItEquipment,
    /// Office furniture and fixtures
    FurnitureFixtures,
    /// Furniture (alias for FurnitureFixtures)
    Furniture,
    /// Vehicles
    Vehicles,
    /// Leasehold improvements
    LeaseholdImprovements,
    /// Intangible assets (software, patents)
    Intangibles,
    /// Software
    Software,
    /// Construction in progress (not yet depreciating)
    ConstructionInProgress,
    /// Low-value assets
    LowValueAssets,
}

impl AssetClass {
    /// Get default useful life in months for this asset class.
    pub fn default_useful_life_months(&self) -> u32 {
        match self {
            Self::Buildings | Self::BuildingImprovements => 480, // 40 years
            Self::Land => 0,                                     // Not depreciated
            Self::MachineryEquipment | Self::Machinery => 120,   // 10 years
            Self::ComputerHardware | Self::ItEquipment => 36,    // 3 years
            Self::FurnitureFixtures | Self::Furniture => 84,     // 7 years
            Self::Vehicles => 60,                                // 5 years
            Self::LeaseholdImprovements => 120,                  // 10 years (or lease term)
            Self::Intangibles | Self::Software => 60,            // 5 years
            Self::ConstructionInProgress => 0,                   // Not depreciated until complete
            Self::LowValueAssets => 12,                          // 1 year
        }
    }

    /// Check if this asset class is depreciable.
    pub fn is_depreciable(&self) -> bool {
        !matches!(self, Self::Land | Self::ConstructionInProgress)
    }

    /// Get default depreciation method for this asset class.
    pub fn default_depreciation_method(&self) -> DepreciationMethod {
        match self {
            Self::Buildings | Self::BuildingImprovements | Self::LeaseholdImprovements => {
                DepreciationMethod::StraightLine
            }
            Self::MachineryEquipment | Self::Machinery => DepreciationMethod::StraightLine,
            Self::ComputerHardware | Self::ItEquipment => {
                DepreciationMethod::DoubleDecliningBalance
            }
            Self::FurnitureFixtures | Self::Furniture => DepreciationMethod::StraightLine,
            Self::Vehicles => DepreciationMethod::DoubleDecliningBalance,
            Self::Intangibles | Self::Software => DepreciationMethod::StraightLine,
            Self::LowValueAssets => DepreciationMethod::ImmediateExpense,
            Self::Land | Self::ConstructionInProgress => DepreciationMethod::None,
        }
    }
}

/// Depreciation calculation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DepreciationMethod {
    /// Straight-line depreciation
    #[default]
    StraightLine,
    /// Double declining balance
    DoubleDecliningBalance,
    /// Sum of years' digits
    SumOfYearsDigits,
    /// Units of production
    UnitsOfProduction,
    /// MACRS (Modified Accelerated Cost Recovery System)
    Macrs,
    /// Immediate expense (low-value assets)
    ImmediateExpense,
    /// No depreciation (land, CIP)
    None,
}

impl DepreciationMethod {
    /// Calculate monthly depreciation amount.
    pub fn calculate_monthly_depreciation(
        &self,
        acquisition_cost: Decimal,
        salvage_value: Decimal,
        useful_life_months: u32,
        months_elapsed: u32,
        accumulated_depreciation: Decimal,
    ) -> Decimal {
        if useful_life_months == 0 {
            return Decimal::ZERO;
        }

        let depreciable_base = acquisition_cost - salvage_value;
        let net_book_value = acquisition_cost - accumulated_depreciation;

        // Don't depreciate below salvage value
        if net_book_value <= salvage_value {
            return Decimal::ZERO;
        }

        match self {
            Self::StraightLine => {
                let monthly_amount = depreciable_base / Decimal::from(useful_life_months);
                // Cap at remaining book value above salvage
                monthly_amount.min(net_book_value - salvage_value)
            }

            Self::DoubleDecliningBalance => {
                // Double the straight-line rate applied to NBV
                let annual_rate = Decimal::from(2) / Decimal::from(useful_life_months) * dec!(12);
                let monthly_rate = annual_rate / dec!(12);
                let depreciation = net_book_value * monthly_rate;
                // Cap at remaining book value above salvage
                depreciation.min(net_book_value - salvage_value)
            }

            Self::SumOfYearsDigits => {
                let years_total = useful_life_months / 12;
                let sum_of_years: u32 = (1..=years_total).sum();
                let current_year = (months_elapsed / 12) + 1;
                let remaining_years = years_total.saturating_sub(current_year) + 1;

                if sum_of_years == 0 || remaining_years == 0 {
                    return Decimal::ZERO;
                }

                let year_fraction = Decimal::from(remaining_years) / Decimal::from(sum_of_years);
                let annual_depreciation = depreciable_base * year_fraction;
                let monthly_amount = annual_depreciation / dec!(12);
                monthly_amount.min(net_book_value - salvage_value)
            }

            Self::UnitsOfProduction => {
                // For units of production, caller should use specific production-based calculation
                // This is a fallback that uses straight-line
                let monthly_amount = depreciable_base / Decimal::from(useful_life_months);
                monthly_amount.min(net_book_value - salvage_value)
            }

            Self::Macrs => {
                // MACRS GDS half-year convention using IRS Publication 946 tables.
                // MACRS ignores salvage value — the full acquisition cost is the depreciable base.
                let useful_life_years = useful_life_months / 12;
                let current_year = (months_elapsed / 12) as usize;

                if let Some(table) = macrs_table_for_life(useful_life_years) {
                    if current_year < table.len() {
                        let pct = macrs_pct(table[current_year]);
                        let annual_depreciation = acquisition_cost * pct / dec!(100);
                        let monthly_amount = annual_depreciation / dec!(12);
                        // Cap so we don't go below zero NBV
                        monthly_amount.min(net_book_value)
                    } else {
                        Decimal::ZERO
                    }
                } else {
                    // Useful life outside MACRS table range — fall back to DDB
                    let annual_rate =
                        Decimal::from(2) / Decimal::from(useful_life_months) * dec!(12);
                    let monthly_rate = annual_rate / dec!(12);
                    let depreciation = net_book_value * monthly_rate;
                    depreciation.min(net_book_value - salvage_value)
                }
            }

            Self::ImmediateExpense => {
                // Full expense in first month
                if months_elapsed == 0 {
                    depreciable_base
                } else {
                    Decimal::ZERO
                }
            }

            Self::None => Decimal::ZERO,
        }
    }
}

/// Account determination rules for fixed asset transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAccountDetermination {
    /// Asset balance sheet account
    pub asset_account: String,
    /// Accumulated depreciation account
    pub accumulated_depreciation_account: String,
    /// Depreciation expense account
    pub depreciation_expense_account: String,
    /// Gain on disposal account
    pub gain_on_disposal_account: String,
    /// Loss on disposal account
    pub loss_on_disposal_account: String,
    /// Clearing account for acquisitions
    pub acquisition_clearing_account: String,
    /// Gain/loss account (combined, for backward compatibility).
    pub gain_loss_account: String,
}

impl Default for AssetAccountDetermination {
    fn default() -> Self {
        Self {
            asset_account: "160000".to_string(),
            accumulated_depreciation_account: "169000".to_string(),
            depreciation_expense_account: "640000".to_string(),
            gain_on_disposal_account: "810000".to_string(),
            loss_on_disposal_account: "840000".to_string(),
            acquisition_clearing_account: "299000".to_string(),
            gain_loss_account: "810000".to_string(),
        }
    }
}

/// Asset acquisition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionType {
    /// External purchase
    #[default]
    Purchase,
    /// Self-constructed
    SelfConstructed,
    /// Transfer from another entity
    Transfer,
    /// Acquired in business combination
    BusinessCombination,
    /// Leased asset (finance lease)
    FinanceLease,
    /// Donation received
    Donation,
}

/// Status of a fixed asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    /// Under construction (CIP)
    UnderConstruction,
    /// Active and in use
    #[default]
    Active,
    /// Temporarily not in use
    Inactive,
    /// Fully depreciated but still in use
    FullyDepreciated,
    /// Scheduled for disposal
    PendingDisposal,
    /// Disposed/retired
    Disposed,
}

/// Fixed asset master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedAsset {
    /// Asset ID (e.g., "FA-001234")
    pub asset_id: String,

    /// Asset sub-number (for component accounting)
    pub sub_number: u16,

    /// Asset description
    pub description: String,

    /// Asset class
    pub asset_class: AssetClass,

    /// Company code
    pub company_code: String,

    /// Cost center responsible for the asset
    pub cost_center: Option<String>,

    /// Location/plant
    pub location: Option<String>,

    /// Acquisition date
    pub acquisition_date: NaiveDate,

    /// Acquisition type
    pub acquisition_type: AcquisitionType,

    /// Original acquisition cost
    pub acquisition_cost: Decimal,

    /// Capitalized date (when depreciation starts)
    pub capitalized_date: Option<NaiveDate>,

    /// Depreciation method
    pub depreciation_method: DepreciationMethod,

    /// Useful life in months
    pub useful_life_months: u32,

    /// Salvage/residual value
    pub salvage_value: Decimal,

    /// Accumulated depreciation as of current period
    pub accumulated_depreciation: Decimal,

    /// Net book value (acquisition_cost - accumulated_depreciation)
    pub net_book_value: Decimal,

    /// Account determination rules
    pub account_determination: AssetAccountDetermination,

    /// Current status
    pub status: AssetStatus,

    /// Disposal date (if disposed)
    pub disposal_date: Option<NaiveDate>,

    /// Disposal proceeds (if disposed)
    pub disposal_proceeds: Option<Decimal>,

    /// Serial number (for tracking)
    pub serial_number: Option<String>,

    /// Manufacturer
    pub manufacturer: Option<String>,

    /// Model
    pub model: Option<String>,

    /// Warranty expiration date
    pub warranty_expiration: Option<NaiveDate>,

    /// Insurance policy number
    pub insurance_policy: Option<String>,

    /// Original PO number
    pub purchase_order: Option<String>,

    /// Vendor ID (who supplied the asset)
    pub vendor_id: Option<String>,

    /// Invoice reference
    pub invoice_reference: Option<String>,
}

impl FixedAsset {
    /// Create a new fixed asset.
    pub fn new(
        asset_id: impl Into<String>,
        description: impl Into<String>,
        asset_class: AssetClass,
        company_code: impl Into<String>,
        acquisition_date: NaiveDate,
        acquisition_cost: Decimal,
    ) -> Self {
        let useful_life_months = asset_class.default_useful_life_months();
        let depreciation_method = asset_class.default_depreciation_method();

        Self {
            asset_id: asset_id.into(),
            sub_number: 0,
            description: description.into(),
            asset_class,
            company_code: company_code.into(),
            cost_center: None,
            location: None,
            acquisition_date,
            acquisition_type: AcquisitionType::Purchase,
            acquisition_cost,
            capitalized_date: Some(acquisition_date),
            depreciation_method,
            useful_life_months,
            salvage_value: Decimal::ZERO,
            accumulated_depreciation: Decimal::ZERO,
            net_book_value: acquisition_cost,
            account_determination: AssetAccountDetermination::default(),
            status: AssetStatus::Active,
            disposal_date: None,
            disposal_proceeds: None,
            serial_number: None,
            manufacturer: None,
            model: None,
            warranty_expiration: None,
            insurance_policy: None,
            purchase_order: None,
            vendor_id: None,
            invoice_reference: None,
        }
    }

    /// Set cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.cost_center = Some(cost_center.into());
        self
    }

    /// Set location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set salvage value.
    pub fn with_salvage_value(mut self, salvage_value: Decimal) -> Self {
        self.salvage_value = salvage_value;
        self
    }

    /// Set depreciation method.
    pub fn with_depreciation_method(mut self, method: DepreciationMethod) -> Self {
        self.depreciation_method = method;
        self
    }

    /// Set useful life.
    pub fn with_useful_life_months(mut self, months: u32) -> Self {
        self.useful_life_months = months;
        self
    }

    /// Set vendor ID.
    pub fn with_vendor(mut self, vendor_id: impl Into<String>) -> Self {
        self.vendor_id = Some(vendor_id.into());
        self
    }

    /// Calculate months since capitalization.
    pub fn months_since_capitalization(&self, as_of_date: NaiveDate) -> u32 {
        let cap_date = self.capitalized_date.unwrap_or(self.acquisition_date);
        if as_of_date < cap_date {
            return 0;
        }

        let years = as_of_date.year() - cap_date.year();
        let months = as_of_date.month() as i32 - cap_date.month() as i32;
        ((years * 12) + months).max(0) as u32
    }

    /// Calculate depreciation for a specific month.
    pub fn calculate_monthly_depreciation(&self, as_of_date: NaiveDate) -> Decimal {
        if !self.asset_class.is_depreciable() {
            return Decimal::ZERO;
        }

        if self.status == AssetStatus::Disposed {
            return Decimal::ZERO;
        }

        let months_elapsed = self.months_since_capitalization(as_of_date);

        self.depreciation_method.calculate_monthly_depreciation(
            self.acquisition_cost,
            self.salvage_value,
            self.useful_life_months,
            months_elapsed,
            self.accumulated_depreciation,
        )
    }

    /// Apply depreciation and update balances.
    pub fn apply_depreciation(&mut self, depreciation_amount: Decimal) {
        self.accumulated_depreciation += depreciation_amount;
        self.net_book_value = self.acquisition_cost - self.accumulated_depreciation;

        // Update status if fully depreciated
        if self.net_book_value <= self.salvage_value && self.status == AssetStatus::Active {
            self.status = AssetStatus::FullyDepreciated;
        }
    }

    /// Calculate gain/loss on disposal.
    pub fn calculate_disposal_gain_loss(&self, proceeds: Decimal) -> Decimal {
        proceeds - self.net_book_value
    }

    /// Record disposal.
    pub fn dispose(&mut self, disposal_date: NaiveDate, proceeds: Decimal) {
        self.disposal_date = Some(disposal_date);
        self.disposal_proceeds = Some(proceeds);
        self.status = AssetStatus::Disposed;
    }

    /// Check if asset is fully depreciated.
    pub fn is_fully_depreciated(&self) -> bool {
        self.net_book_value <= self.salvage_value
    }

    /// Calculate remaining useful life in months.
    pub fn remaining_useful_life_months(&self, as_of_date: NaiveDate) -> u32 {
        let months_elapsed = self.months_since_capitalization(as_of_date);
        self.useful_life_months.saturating_sub(months_elapsed)
    }

    /// Calculate depreciation rate (annual percentage).
    pub fn annual_depreciation_rate(&self) -> Decimal {
        if self.useful_life_months == 0 {
            return Decimal::ZERO;
        }

        match self.depreciation_method {
            DepreciationMethod::StraightLine => {
                Decimal::from(12) / Decimal::from(self.useful_life_months) * dec!(100)
            }
            DepreciationMethod::DoubleDecliningBalance => {
                Decimal::from(24) / Decimal::from(self.useful_life_months) * dec!(100)
            }
            _ => Decimal::from(12) / Decimal::from(self.useful_life_months) * dec!(100),
        }
    }

    /// Return the annual MACRS depreciation for a given recovery year (1-indexed).
    ///
    /// Uses the IRS Publication 946 GDS half-year convention tables.
    /// MACRS depreciation is based on the full acquisition cost (salvage value is ignored).
    /// Returns `Decimal::ZERO` if the year is out of range or no table matches the useful life.
    pub fn macrs_depreciation(&self, year: u32) -> Decimal {
        if year == 0 {
            return Decimal::ZERO;
        }

        let useful_life_years = self.useful_life_months / 12;
        let table_index = (year - 1) as usize;

        match macrs_table_for_life(useful_life_years) {
            Some(table) if table_index < table.len() => {
                let pct = macrs_pct(table[table_index]);
                self.acquisition_cost * pct / dec!(100)
            }
            _ => Decimal::ZERO,
        }
    }

    /// Return the monthly double-declining balance depreciation amount.
    ///
    /// The DDB rate is `2 / useful_life_months * 12` applied monthly against the current
    /// net book value. The result is rounded to 2 decimal places and capped so the asset
    /// does not depreciate below the salvage value.
    pub fn ddb_depreciation(&self) -> Decimal {
        if self.useful_life_months == 0 {
            return Decimal::ZERO;
        }

        let net_book_value = self.acquisition_cost - self.accumulated_depreciation;
        if net_book_value <= self.salvage_value {
            return Decimal::ZERO;
        }

        let annual_rate = Decimal::from(2) / Decimal::from(self.useful_life_months) * dec!(12);
        let monthly_rate = annual_rate / dec!(12);
        let depreciation = (net_book_value * monthly_rate).round_dp(2);
        depreciation.min(net_book_value - self.salvage_value)
    }
}

/// Pool of fixed assets for transaction generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixedAssetPool {
    /// All fixed assets
    pub assets: Vec<FixedAsset>,
    /// Index by asset class
    #[serde(skip)]
    class_index: std::collections::HashMap<AssetClass, Vec<usize>>,
    /// Index by company code
    #[serde(skip)]
    company_index: std::collections::HashMap<String, Vec<usize>>,
}

impl FixedAssetPool {
    /// Create a new empty asset pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an asset to the pool.
    pub fn add_asset(&mut self, asset: FixedAsset) {
        let idx = self.assets.len();
        let asset_class = asset.asset_class;
        let company_code = asset.company_code.clone();

        self.assets.push(asset);

        self.class_index.entry(asset_class).or_default().push(idx);
        self.company_index
            .entry(company_code)
            .or_default()
            .push(idx);
    }

    /// Get all assets requiring depreciation for a given month.
    pub fn get_depreciable_assets(&self) -> Vec<&FixedAsset> {
        self.assets
            .iter()
            .filter(|a| {
                a.asset_class.is_depreciable()
                    && a.status == AssetStatus::Active
                    && !a.is_fully_depreciated()
            })
            .collect()
    }

    /// Get mutable references to depreciable assets.
    pub fn get_depreciable_assets_mut(&mut self) -> Vec<&mut FixedAsset> {
        self.assets
            .iter_mut()
            .filter(|a| {
                a.asset_class.is_depreciable()
                    && a.status == AssetStatus::Active
                    && !a.is_fully_depreciated()
            })
            .collect()
    }

    /// Get assets by company code.
    pub fn get_by_company(&self, company_code: &str) -> Vec<&FixedAsset> {
        self.company_index
            .get(company_code)
            .map(|indices| indices.iter().map(|&i| &self.assets[i]).collect())
            .unwrap_or_default()
    }

    /// Get assets by class.
    pub fn get_by_class(&self, asset_class: AssetClass) -> Vec<&FixedAsset> {
        self.class_index
            .get(&asset_class)
            .map(|indices| indices.iter().map(|&i| &self.assets[i]).collect())
            .unwrap_or_default()
    }

    /// Get asset by ID.
    pub fn get_by_id(&self, asset_id: &str) -> Option<&FixedAsset> {
        self.assets.iter().find(|a| a.asset_id == asset_id)
    }

    /// Get mutable asset by ID.
    pub fn get_by_id_mut(&mut self, asset_id: &str) -> Option<&mut FixedAsset> {
        self.assets.iter_mut().find(|a| a.asset_id == asset_id)
    }

    /// Calculate total depreciation for all assets in a period.
    pub fn calculate_period_depreciation(&self, as_of_date: NaiveDate) -> Decimal {
        self.get_depreciable_assets()
            .iter()
            .map(|a| a.calculate_monthly_depreciation(as_of_date))
            .sum()
    }

    /// Get total net book value.
    pub fn total_net_book_value(&self) -> Decimal {
        self.assets
            .iter()
            .filter(|a| a.status != AssetStatus::Disposed)
            .map(|a| a.net_book_value)
            .sum()
    }

    /// Get count of assets.
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Check if pool is empty.
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Rebuild indices after deserialization.
    pub fn rebuild_indices(&mut self) {
        self.class_index.clear();
        self.company_index.clear();

        for (idx, asset) in self.assets.iter().enumerate() {
            self.class_index
                .entry(asset.asset_class)
                .or_default()
                .push(idx);
            self.company_index
                .entry(asset.company_code.clone())
                .or_default()
                .push(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_asset_creation() {
        let asset = FixedAsset::new(
            "FA-001",
            "Office Computer",
            AssetClass::ComputerHardware,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(2000),
        );

        assert_eq!(asset.asset_id, "FA-001");
        assert_eq!(asset.acquisition_cost, Decimal::from(2000));
        assert_eq!(asset.useful_life_months, 36); // 3 years for computers
    }

    #[test]
    fn test_straight_line_depreciation() {
        let asset = FixedAsset::new(
            "FA-001",
            "Office Equipment",
            AssetClass::FurnitureFixtures,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(8400),
        )
        .with_useful_life_months(84) // 7 years
        .with_depreciation_method(DepreciationMethod::StraightLine);

        let monthly_dep = asset.calculate_monthly_depreciation(test_date(2024, 2, 1));
        assert_eq!(monthly_dep, Decimal::from(100)); // 8400 / 84 months
    }

    #[test]
    fn test_salvage_value_limit() {
        let mut asset = FixedAsset::new(
            "FA-001",
            "Test Asset",
            AssetClass::MachineryEquipment,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(1200),
        )
        .with_useful_life_months(12)
        .with_salvage_value(Decimal::from(200));

        // Apply 11 months of depreciation (1000/12 ~= 83.33 each)
        for _ in 0..11 {
            let dep = Decimal::from(83);
            asset.apply_depreciation(dep);
        }

        // At this point, NBV should be around 287, which is above salvage (200)
        // Next depreciation should be limited to not go below salvage
        let final_dep = asset.calculate_monthly_depreciation(test_date(2024, 12, 1));

        // Verify we don't depreciate below salvage
        asset.apply_depreciation(final_dep);
        assert!(asset.net_book_value >= asset.salvage_value);
    }

    #[test]
    fn test_disposal() {
        let mut asset = FixedAsset::new(
            "FA-001",
            "Old Equipment",
            AssetClass::MachineryEquipment,
            "1000",
            test_date(2020, 1, 1),
            Decimal::from(10000),
        );

        // Simulate some depreciation
        asset.apply_depreciation(Decimal::from(5000));

        // Calculate gain/loss
        let gain_loss = asset.calculate_disposal_gain_loss(Decimal::from(6000));
        assert_eq!(gain_loss, Decimal::from(1000)); // Gain of 1000

        // Record disposal
        asset.dispose(test_date(2024, 1, 1), Decimal::from(6000));
        assert_eq!(asset.status, AssetStatus::Disposed);
    }

    #[test]
    fn test_land_not_depreciable() {
        let asset = FixedAsset::new(
            "FA-001",
            "Land Parcel",
            AssetClass::Land,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(500000),
        );

        let dep = asset.calculate_monthly_depreciation(test_date(2024, 6, 1));
        assert_eq!(dep, Decimal::ZERO);
    }

    #[test]
    fn test_asset_pool() {
        let mut pool = FixedAssetPool::new();

        pool.add_asset(FixedAsset::new(
            "FA-001",
            "Computer 1",
            AssetClass::ComputerHardware,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(2000),
        ));

        pool.add_asset(FixedAsset::new(
            "FA-002",
            "Desk",
            AssetClass::FurnitureFixtures,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(500),
        ));

        assert_eq!(pool.len(), 2);
        assert_eq!(pool.get_by_class(AssetClass::ComputerHardware).len(), 1);
        assert_eq!(pool.get_by_company("1000").len(), 2);
    }

    #[test]
    fn test_months_since_capitalization() {
        let asset = FixedAsset::new(
            "FA-001",
            "Test",
            AssetClass::MachineryEquipment,
            "1000",
            test_date(2024, 3, 15),
            Decimal::from(10000),
        );

        assert_eq!(asset.months_since_capitalization(test_date(2024, 3, 1)), 0);
        assert_eq!(asset.months_since_capitalization(test_date(2024, 6, 1)), 3);
        assert_eq!(asset.months_since_capitalization(test_date(2025, 3, 1)), 12);
    }

    // ---- MACRS GDS table tests ----

    #[test]
    fn test_macrs_tables_sum_to_100() {
        let tables: &[(&str, &[&str])] = &[
            ("3-year", MACRS_GDS_3_YEAR),
            ("5-year", MACRS_GDS_5_YEAR),
            ("7-year", MACRS_GDS_7_YEAR),
            ("10-year", MACRS_GDS_10_YEAR),
            ("15-year", MACRS_GDS_15_YEAR),
            ("20-year", MACRS_GDS_20_YEAR),
        ];

        let tolerance = dec!(0.02);
        let hundred = dec!(100);

        for (label, table) in tables {
            let sum: Decimal = table.iter().map(|s| macrs_pct(s)).sum();
            let diff = (sum - hundred).abs();
            assert!(
                diff < tolerance,
                "MACRS GDS {label} table sums to {sum}, expected ~100.0"
            );
        }
    }

    #[test]
    fn test_macrs_table_for_life_mapping() {
        // 1-3 years -> 3-year table (4 entries)
        assert_eq!(macrs_table_for_life(1).unwrap().len(), 4);
        assert_eq!(macrs_table_for_life(3).unwrap().len(), 4);

        // 4-5 years -> 5-year table (6 entries)
        assert_eq!(macrs_table_for_life(4).unwrap().len(), 6);
        assert_eq!(macrs_table_for_life(5).unwrap().len(), 6);

        // 6-7 years -> 7-year table (8 entries)
        assert_eq!(macrs_table_for_life(6).unwrap().len(), 8);
        assert_eq!(macrs_table_for_life(7).unwrap().len(), 8);

        // 8-10 years -> 10-year table (11 entries)
        assert_eq!(macrs_table_for_life(8).unwrap().len(), 11);
        assert_eq!(macrs_table_for_life(10).unwrap().len(), 11);

        // 11-15 years -> 15-year table (16 entries)
        assert_eq!(macrs_table_for_life(11).unwrap().len(), 16);
        assert_eq!(macrs_table_for_life(15).unwrap().len(), 16);

        // 16-20 years -> 20-year table (21 entries)
        assert_eq!(macrs_table_for_life(16).unwrap().len(), 21);
        assert_eq!(macrs_table_for_life(20).unwrap().len(), 21);

        // Out of range -> None
        assert!(macrs_table_for_life(0).is_none());
        assert!(macrs_table_for_life(21).is_none());
        assert!(macrs_table_for_life(100).is_none());
    }

    #[test]
    fn test_macrs_depreciation_5_year_asset() {
        let asset = FixedAsset::new(
            "FA-MACRS",
            "Vehicle",
            AssetClass::Vehicles,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(10000),
        )
        .with_useful_life_months(60) // 5 years
        .with_depreciation_method(DepreciationMethod::Macrs);

        // Year 1: 20.00% of 10,000 = 2,000
        assert_eq!(asset.macrs_depreciation(1), Decimal::from(2000));
        // Year 2: 32.00% of 10,000 = 3,200
        assert_eq!(asset.macrs_depreciation(2), Decimal::from(3200));
        // Year 3: 19.20% of 10,000 = 1,920
        assert_eq!(asset.macrs_depreciation(3), Decimal::from(1920));
        // Year 4: 11.52% of 10,000 = 1,152
        assert_eq!(asset.macrs_depreciation(4), Decimal::from(1152));
        // Year 5: 11.52% of 10,000 = 1,152
        assert_eq!(asset.macrs_depreciation(5), Decimal::from(1152));
        // Year 6: 5.76% of 10,000 = 576
        assert_eq!(asset.macrs_depreciation(6), Decimal::from(576));
        // Year 7: beyond table -> 0
        assert_eq!(asset.macrs_depreciation(7), Decimal::ZERO);
        // Year 0: invalid -> 0
        assert_eq!(asset.macrs_depreciation(0), Decimal::ZERO);
    }

    #[test]
    fn test_macrs_calculate_monthly_depreciation_uses_tables() {
        let asset = FixedAsset::new(
            "FA-MACRS-M",
            "Vehicle",
            AssetClass::Vehicles,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(12000),
        )
        .with_useful_life_months(60) // 5 years
        .with_depreciation_method(DepreciationMethod::Macrs);

        // Year 1 (months_elapsed 0..11): 20.00% of 12,000 = 2,400 annual -> 200/month
        let monthly_year1 = asset.calculate_monthly_depreciation(test_date(2024, 2, 1));
        assert_eq!(monthly_year1, Decimal::from(200));

        // Year 2 (months_elapsed 12..23): 32.00% of 12,000 = 3,840 annual -> 320/month
        let monthly_year2 = asset.calculate_monthly_depreciation(test_date(2025, 2, 1));
        assert_eq!(monthly_year2, Decimal::from(320));
    }

    #[test]
    fn test_ddb_depreciation() {
        let asset = FixedAsset::new(
            "FA-DDB",
            "Server",
            AssetClass::ComputerHardware,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(3600),
        )
        .with_useful_life_months(36) // 3 years
        .with_depreciation_method(DepreciationMethod::DoubleDecliningBalance);

        // DDB annual rate = 2 / 36 * 12 = 2/3
        // Monthly rate = (2/3) / 12 = 1/18
        // First month: 3600 * (1/18) = 200
        let monthly = asset.ddb_depreciation();
        assert_eq!(monthly, Decimal::from(200));
    }

    #[test]
    fn test_ddb_depreciation_with_accumulated() {
        let mut asset = FixedAsset::new(
            "FA-DDB2",
            "Laptop",
            AssetClass::ComputerHardware,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(1800),
        )
        .with_useful_life_months(36);

        // After accumulating 900 of depreciation, NBV = 900
        asset.apply_depreciation(Decimal::from(900));

        // Monthly rate = 1/18, on NBV 900 -> 50
        let monthly = asset.ddb_depreciation();
        assert_eq!(monthly, Decimal::from(50));
    }

    #[test]
    fn test_ddb_depreciation_respects_salvage() {
        let mut asset = FixedAsset::new(
            "FA-DDB3",
            "Printer",
            AssetClass::ComputerHardware,
            "1000",
            test_date(2024, 1, 1),
            Decimal::from(1800),
        )
        .with_useful_life_months(36)
        .with_salvage_value(Decimal::from(200));

        // Accumulate until NBV is barely above salvage
        // NBV = 1800 - 1590 = 210, salvage = 200
        asset.apply_depreciation(Decimal::from(1590));

        // DDB would compute 210 * (1/18) = 11.666..., but cap at NBV - salvage = 10
        let monthly = asset.ddb_depreciation();
        assert_eq!(monthly, Decimal::from(10));
    }
}
