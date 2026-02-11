//! Fixed Asset model.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::subledger::GLReference;

/// Fixed Asset record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedAssetRecord {
    /// Asset number (unique identifier).
    pub asset_number: String,
    /// Sub-number (for complex assets).
    pub sub_number: String,
    /// Company code.
    pub company_code: String,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Description.
    pub description: String,
    /// Serial number.
    pub serial_number: Option<String>,
    /// Inventory number.
    pub inventory_number: Option<String>,
    /// Asset status.
    pub status: AssetStatus,
    /// Acquisition date.
    pub acquisition_date: NaiveDate,
    /// Capitalization date.
    pub capitalization_date: NaiveDate,
    /// First depreciation period.
    pub first_depreciation_date: NaiveDate,
    /// Original acquisition cost.
    pub acquisition_cost: Decimal,
    /// Currency.
    pub currency: String,
    /// Accumulated depreciation.
    pub accumulated_depreciation: Decimal,
    /// Net book value.
    pub net_book_value: Decimal,
    /// Depreciation areas.
    pub depreciation_areas: Vec<DepreciationArea>,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Profit center.
    pub profit_center: Option<String>,
    /// Plant/location.
    pub plant: Option<String>,
    /// Room/location detail.
    pub location: Option<String>,
    /// Responsible person.
    pub responsible_person: Option<String>,
    /// Vendor (for acquisitions).
    pub vendor_id: Option<String>,
    /// Purchase order reference.
    pub po_reference: Option<String>,
    /// GL account mappings.
    pub account_determination: AssetAccountDetermination,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Created by user.
    pub created_by: Option<String>,
    /// Last modified.
    pub modified_at: Option<DateTime<Utc>>,
    /// Notes.
    pub notes: Option<String>,
}

impl FixedAssetRecord {
    /// Creates a new fixed asset.
    pub fn new(
        asset_number: String,
        company_code: String,
        asset_class: AssetClass,
        description: String,
        acquisition_date: NaiveDate,
        acquisition_cost: Decimal,
        currency: String,
    ) -> Self {
        Self {
            asset_number,
            sub_number: "0".to_string(),
            company_code,
            asset_class,
            description,
            serial_number: None,
            inventory_number: None,
            status: AssetStatus::Active,
            acquisition_date,
            capitalization_date: acquisition_date,
            first_depreciation_date: Self::calculate_first_depreciation_date(acquisition_date),
            acquisition_cost,
            currency,
            accumulated_depreciation: Decimal::ZERO,
            net_book_value: acquisition_cost,
            depreciation_areas: Vec::new(),
            cost_center: None,
            profit_center: None,
            plant: None,
            location: None,
            responsible_person: None,
            vendor_id: None,
            po_reference: None,
            account_determination: AssetAccountDetermination::default_for_class(asset_class),
            created_at: Utc::now(),
            created_by: None,
            modified_at: None,
            notes: None,
        }
    }

    /// Calculates first depreciation date (start of next month).
    fn calculate_first_depreciation_date(acquisition_date: NaiveDate) -> NaiveDate {
        let year = acquisition_date.year();
        let month = acquisition_date.month();

        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid date components")
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid date components")
        }
    }

    /// Adds a depreciation area.
    pub fn add_depreciation_area(&mut self, area: DepreciationArea) {
        self.depreciation_areas.push(area);
    }

    /// Creates with standard depreciation areas (book and tax).
    pub fn with_standard_depreciation(
        mut self,
        useful_life_years: u32,
        method: DepreciationMethod,
    ) -> Self {
        // Book depreciation
        self.depreciation_areas.push(DepreciationArea::new(
            DepreciationAreaType::Book,
            method,
            useful_life_years * 12,
            self.acquisition_cost,
        ));

        // Tax depreciation (may use different life)
        self.depreciation_areas.push(DepreciationArea::new(
            DepreciationAreaType::Tax,
            method,
            useful_life_years * 12,
            self.acquisition_cost,
        ));

        self
    }

    /// Records depreciation.
    pub fn record_depreciation(&mut self, amount: Decimal, area_type: DepreciationAreaType) {
        self.accumulated_depreciation += amount;
        self.net_book_value = self.acquisition_cost - self.accumulated_depreciation;

        if let Some(area) = self
            .depreciation_areas
            .iter_mut()
            .find(|a| a.area_type == area_type)
        {
            area.accumulated_depreciation += amount;
            area.net_book_value = area.acquisition_cost - area.accumulated_depreciation;
        }

        self.modified_at = Some(Utc::now());
    }

    /// Records an acquisition addition.
    pub fn add_acquisition(&mut self, amount: Decimal, _date: NaiveDate) {
        self.acquisition_cost += amount;
        self.net_book_value += amount;

        for area in &mut self.depreciation_areas {
            area.acquisition_cost += amount;
            area.net_book_value += amount;
        }

        self.modified_at = Some(Utc::now());
    }

    /// Checks if fully depreciated.
    pub fn is_fully_depreciated(&self) -> bool {
        self.net_book_value <= Decimal::ZERO
    }

    /// Gets remaining useful life in months.
    pub fn remaining_life_months(&self, area_type: DepreciationAreaType) -> u32 {
        self.depreciation_areas
            .iter()
            .find(|a| a.area_type == area_type)
            .map(|a| a.remaining_life_months())
            .unwrap_or(0)
    }

    /// Sets location information.
    pub fn with_location(mut self, plant: String, location: String) -> Self {
        self.plant = Some(plant);
        self.location = Some(location);
        self
    }

    /// Sets cost center.
    pub fn with_cost_center(mut self, cost_center: String) -> Self {
        self.cost_center = Some(cost_center);
        self
    }

    /// Sets vendor reference.
    pub fn with_vendor(mut self, vendor_id: String, po_reference: Option<String>) -> Self {
        self.vendor_id = Some(vendor_id);
        self.po_reference = po_reference;
        self
    }

    // === Backward compatibility accessor methods ===

    /// Gets asset_id (alias for asset_number).
    pub fn asset_id(&self) -> &str {
        &self.asset_number
    }

    /// Gets current acquisition cost (alias for acquisition_cost).
    pub fn current_acquisition_cost(&self) -> Decimal {
        self.acquisition_cost
    }

    /// Gets salvage value from the first depreciation area.
    pub fn salvage_value(&self) -> Decimal {
        self.depreciation_areas
            .first()
            .map(|a| a.salvage_value)
            .unwrap_or(Decimal::ZERO)
    }

    /// Gets useful life in months from the first depreciation area.
    pub fn useful_life_months(&self) -> u32 {
        self.depreciation_areas
            .first()
            .map(|a| a.useful_life_months)
            .unwrap_or(0)
    }

    /// Gets accumulated depreciation account from account determination.
    pub fn accumulated_depreciation_account(&self) -> &str {
        &self.account_determination.accumulated_depreciation_account
    }

    /// Marks as inactive (retired).
    pub fn retire(&mut self, retirement_date: NaiveDate) {
        self.status = AssetStatus::Retired;
        self.notes = Some(format!(
            "{}Retired on {}",
            self.notes
                .as_ref()
                .map(|n| format!("{}. ", n))
                .unwrap_or_default(),
            retirement_date
        ));
        self.modified_at = Some(Utc::now());
    }
}

/// Asset class enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetClass {
    /// Land.
    Land,
    /// Buildings.
    Buildings,
    /// Building improvements.
    BuildingImprovements,
    /// Machinery and equipment.
    MachineryEquipment,
    /// Machinery (alias for MachineryEquipment).
    Machinery,
    /// Vehicles.
    Vehicles,
    /// Office equipment.
    OfficeEquipment,
    /// Computer equipment.
    ComputerEquipment,
    /// IT Equipment (alias for ComputerEquipment).
    ItEquipment,
    /// Computer hardware (alias for ComputerEquipment).
    ComputerHardware,
    /// Software.
    Software,
    /// Intangibles (alias for Software).
    Intangibles,
    /// Furniture and fixtures.
    FurnitureFixtures,
    /// Furniture (alias for FurnitureFixtures).
    Furniture,
    /// Leasehold improvements.
    LeaseholdImprovements,
    /// Construction in progress.
    ConstructionInProgress,
    /// Low value assets.
    LowValueAssets,
    /// Other.
    Other,
}

impl AssetClass {
    /// Gets default useful life in years.
    pub fn default_useful_life_years(&self) -> u32 {
        match self {
            AssetClass::Land => 0, // Land doesn't depreciate
            AssetClass::Buildings => 39,
            AssetClass::BuildingImprovements => 15,
            AssetClass::MachineryEquipment | AssetClass::Machinery => 7,
            AssetClass::Vehicles => 5,
            AssetClass::OfficeEquipment => 7,
            AssetClass::ComputerEquipment
            | AssetClass::ItEquipment
            | AssetClass::ComputerHardware => 5,
            AssetClass::Software | AssetClass::Intangibles => 3,
            AssetClass::FurnitureFixtures | AssetClass::Furniture => 7,
            AssetClass::LeaseholdImprovements => 10,
            AssetClass::ConstructionInProgress => 0, // CIP doesn't depreciate
            AssetClass::LowValueAssets => 1,         // Typically expensed immediately
            AssetClass::Other => 7,
        }
    }

    /// Gets default depreciation method.
    pub fn default_depreciation_method(&self) -> DepreciationMethod {
        match self {
            AssetClass::Land | AssetClass::ConstructionInProgress => DepreciationMethod::None,
            AssetClass::ComputerEquipment
            | AssetClass::ItEquipment
            | AssetClass::ComputerHardware
            | AssetClass::Software
            | AssetClass::Intangibles => DepreciationMethod::StraightLine,
            AssetClass::Vehicles | AssetClass::MachineryEquipment | AssetClass::Machinery => {
                DepreciationMethod::DecliningBalance { rate: dec!(0.40) }
            }
            AssetClass::LowValueAssets => DepreciationMethod::StraightLine,
            _ => DepreciationMethod::StraightLine,
        }
    }
}

/// Asset status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssetStatus {
    /// Under construction.
    UnderConstruction,
    /// Active and depreciating.
    #[default]
    Active,
    /// Held for sale.
    HeldForSale,
    /// Fully depreciated but still in use.
    FullyDepreciated,
    /// Retired/disposed.
    Retired,
    /// Transferred to another entity.
    Transferred,
}

/// Depreciation area (book, tax, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationArea {
    /// Area type.
    pub area_type: DepreciationAreaType,
    /// Depreciation method.
    pub method: DepreciationMethod,
    /// Useful life in months.
    pub useful_life_months: u32,
    /// Depreciation start date.
    pub depreciation_start_date: Option<NaiveDate>,
    /// Acquisition cost (can differ from main asset).
    pub acquisition_cost: Decimal,
    /// Accumulated depreciation.
    pub accumulated_depreciation: Decimal,
    /// Net book value.
    pub net_book_value: Decimal,
    /// Salvage value.
    pub salvage_value: Decimal,
    /// Depreciation periods completed.
    pub periods_completed: u32,
    /// Last depreciation date.
    pub last_depreciation_date: Option<NaiveDate>,
}

impl DepreciationArea {
    /// Creates a new depreciation area.
    pub fn new(
        area_type: DepreciationAreaType,
        method: DepreciationMethod,
        useful_life_months: u32,
        acquisition_cost: Decimal,
    ) -> Self {
        Self {
            area_type,
            method,
            useful_life_months,
            depreciation_start_date: None,
            acquisition_cost,
            accumulated_depreciation: Decimal::ZERO,
            net_book_value: acquisition_cost,
            salvage_value: Decimal::ZERO,
            periods_completed: 0,
            last_depreciation_date: None,
        }
    }

    /// Sets salvage value.
    pub fn with_salvage_value(mut self, value: Decimal) -> Self {
        self.salvage_value = value;
        self
    }

    /// Gets remaining useful life in months.
    pub fn remaining_life_months(&self) -> u32 {
        self.useful_life_months
            .saturating_sub(self.periods_completed)
    }

    /// Checks if fully depreciated.
    pub fn is_fully_depreciated(&self) -> bool {
        self.net_book_value <= self.salvage_value
    }

    /// Calculates monthly depreciation based on method.
    pub fn calculate_monthly_depreciation(&self) -> Decimal {
        if self.is_fully_depreciated() {
            return Decimal::ZERO;
        }

        let depreciable_base = self.acquisition_cost - self.salvage_value;

        match self.method {
            DepreciationMethod::None => Decimal::ZERO,
            DepreciationMethod::StraightLine => {
                if self.useful_life_months > 0 {
                    (depreciable_base / Decimal::from(self.useful_life_months)).round_dp(2)
                } else {
                    Decimal::ZERO
                }
            }
            DepreciationMethod::DecliningBalance { rate } => {
                let annual_depreciation = self.net_book_value * rate;
                (annual_depreciation / dec!(12)).round_dp(2)
            }
            DepreciationMethod::SumOfYearsDigits => {
                let remaining_years = self.remaining_life_months() / 12;
                let total_years = self.useful_life_months / 12;
                let sum_of_years = (total_years * (total_years + 1)) / 2;
                if sum_of_years > 0 {
                    let annual = depreciable_base * Decimal::from(remaining_years)
                        / Decimal::from(sum_of_years);
                    (annual / dec!(12)).round_dp(2)
                } else {
                    Decimal::ZERO
                }
            }
            DepreciationMethod::UnitsOfProduction { total_units, .. } => {
                // This needs production data; return estimate
                if total_units > 0 {
                    depreciable_base / Decimal::from(total_units) / dec!(12)
                } else {
                    Decimal::ZERO
                }
            }
        }
    }
}

/// Type of depreciation area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepreciationAreaType {
    /// Book (GAAP/IFRS) depreciation.
    Book,
    /// Tax depreciation.
    Tax,
    /// Group/consolidated reporting.
    Group,
    /// Management reporting.
    Management,
    /// Insurance valuation.
    Insurance,
}

/// Depreciation method.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DepreciationMethod {
    /// No depreciation (e.g., land).
    None,
    /// Straight-line depreciation.
    StraightLine,
    /// Declining balance (accelerated).
    DecliningBalance {
        /// Annual rate (e.g., 0.40 for 40%).
        rate: Decimal,
    },
    /// Sum of years digits.
    SumOfYearsDigits,
    /// Units of production.
    UnitsOfProduction {
        /// Total expected units.
        total_units: u32,
        /// Units this period.
        period_units: u32,
    },
}

/// Account determination for asset class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAccountDetermination {
    /// Asset acquisition account.
    pub acquisition_account: String,
    /// Accumulated depreciation account.
    pub accumulated_depreciation_account: String,
    /// Depreciation expense account.
    pub depreciation_expense_account: String,
    /// Depreciation account (alias for accumulated_depreciation_account).
    pub depreciation_account: String,
    /// Gain on disposal account.
    pub gain_on_disposal_account: String,
    /// Loss on disposal account.
    pub loss_on_disposal_account: String,
    /// Gain/loss account (combined, for backward compatibility).
    pub gain_loss_account: String,
    /// Clearing account (for acquisitions).
    pub clearing_account: String,
}

impl AssetAccountDetermination {
    /// Creates default account determination for asset class.
    pub fn default_for_class(class: AssetClass) -> Self {
        let prefix = match class {
            AssetClass::Land => "1510",
            AssetClass::Buildings => "1520",
            AssetClass::BuildingImprovements => "1525",
            AssetClass::MachineryEquipment | AssetClass::Machinery => "1530",
            AssetClass::Vehicles => "1540",
            AssetClass::OfficeEquipment => "1550",
            AssetClass::ComputerEquipment
            | AssetClass::ItEquipment
            | AssetClass::ComputerHardware => "1555",
            AssetClass::Software | AssetClass::Intangibles => "1560",
            AssetClass::FurnitureFixtures | AssetClass::Furniture => "1570",
            AssetClass::LeaseholdImprovements => "1580",
            AssetClass::ConstructionInProgress => "1600",
            AssetClass::LowValueAssets => "1595",
            AssetClass::Other => "1590",
        };

        let depreciation_account = format!("{}9", &prefix[..3]);

        Self {
            acquisition_account: prefix.to_string(),
            accumulated_depreciation_account: depreciation_account.clone(),
            depreciation_expense_account: "7100".to_string(),
            depreciation_account,
            gain_on_disposal_account: "4900".to_string(),
            loss_on_disposal_account: "7900".to_string(),
            gain_loss_account: "4900".to_string(),
            clearing_account: "1599".to_string(),
        }
    }
}

/// Asset acquisition record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAcquisition {
    /// Transaction ID.
    pub transaction_id: String,
    /// Asset number.
    pub asset_number: String,
    /// Sub-number.
    pub sub_number: String,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Acquisition amount.
    pub amount: Decimal,
    /// Acquisition type.
    pub acquisition_type: AcquisitionType,
    /// Vendor ID.
    pub vendor_id: Option<String>,
    /// Invoice reference.
    pub invoice_reference: Option<String>,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Notes.
    pub notes: Option<String>,
}

/// Type of acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionType {
    /// External purchase.
    ExternalPurchase,
    /// Internal production.
    InternalProduction,
    /// Transfer from CIP.
    TransferFromCIP,
    /// Intercompany transfer.
    IntercompanyTransfer,
    /// Post-capitalization.
    PostCapitalization,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let asset = FixedAssetRecord::new(
            "ASSET001".to_string(),
            "1000".to_string(),
            AssetClass::MachineryEquipment,
            "Production Machine".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(100000),
            "USD".to_string(),
        );

        assert_eq!(asset.acquisition_cost, dec!(100000));
        assert_eq!(asset.net_book_value, dec!(100000));
        assert_eq!(asset.status, AssetStatus::Active);
    }

    #[test]
    fn test_depreciation_area() {
        let area = DepreciationArea::new(
            DepreciationAreaType::Book,
            DepreciationMethod::StraightLine,
            60, // 5 years
            dec!(100000),
        )
        .with_salvage_value(dec!(10000));

        let monthly = area.calculate_monthly_depreciation();
        // (100000 - 10000) / 60 = 1500
        assert_eq!(monthly, dec!(1500));
    }

    #[test]
    fn test_record_depreciation() {
        let mut asset = FixedAssetRecord::new(
            "ASSET001".to_string(),
            "1000".to_string(),
            AssetClass::ComputerEquipment,
            "Server".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(50000),
            "USD".to_string(),
        )
        .with_standard_depreciation(5, DepreciationMethod::StraightLine);

        asset.record_depreciation(dec!(833.33), DepreciationAreaType::Book);

        assert_eq!(asset.accumulated_depreciation, dec!(833.33));
        assert_eq!(asset.net_book_value, dec!(49166.67));
    }

    #[test]
    fn test_asset_class_defaults() {
        assert_eq!(AssetClass::Buildings.default_useful_life_years(), 39);
        assert_eq!(AssetClass::ComputerEquipment.default_useful_life_years(), 5);
        assert_eq!(AssetClass::Land.default_useful_life_years(), 0);
    }
}
