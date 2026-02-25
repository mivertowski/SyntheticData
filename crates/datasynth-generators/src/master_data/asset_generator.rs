//! Fixed asset generator for asset master data with depreciation schedules.

use chrono::NaiveDate;
use datasynth_core::models::{
    AssetAccountDetermination, AssetClass, AssetStatus, DepreciationMethod, FixedAsset,
    FixedAssetPool,
};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Configuration for asset generation.
#[derive(Debug, Clone)]
pub struct AssetGeneratorConfig {
    /// Distribution of asset classes (class, probability)
    pub asset_class_distribution: Vec<(AssetClass, f64)>,
    /// Distribution of depreciation methods (method, probability)
    pub depreciation_method_distribution: Vec<(DepreciationMethod, f64)>,
    /// Default useful life by asset class (class, months)
    pub useful_life_by_class: Vec<(AssetClass, u32)>,
    /// Acquisition cost range (min, max)
    pub acquisition_cost_range: (Decimal, Decimal),
    /// Salvage value percentage (typically 5-10%)
    pub salvage_value_percent: f64,
    /// Probability of asset being fully depreciated
    pub fully_depreciated_rate: f64,
    /// Probability of asset being disposed
    pub disposed_rate: f64,
}

impl Default for AssetGeneratorConfig {
    fn default() -> Self {
        Self {
            asset_class_distribution: vec![
                (AssetClass::Buildings, 0.05),
                (AssetClass::Machinery, 0.25),
                (AssetClass::Vehicles, 0.15),
                (AssetClass::Furniture, 0.15),
                (AssetClass::ItEquipment, 0.25),
                (AssetClass::Software, 0.10),
                (AssetClass::LeaseholdImprovements, 0.05),
            ],
            depreciation_method_distribution: vec![
                (DepreciationMethod::StraightLine, 0.70),
                (DepreciationMethod::DoubleDecliningBalance, 0.15),
                (DepreciationMethod::Macrs, 0.10),
                (DepreciationMethod::SumOfYearsDigits, 0.05),
            ],
            useful_life_by_class: vec![
                (AssetClass::Buildings, 480),             // 40 years
                (AssetClass::BuildingImprovements, 180),  // 15 years
                (AssetClass::Machinery, 84),              // 7 years
                (AssetClass::Vehicles, 60),               // 5 years
                (AssetClass::Furniture, 84),              // 7 years
                (AssetClass::ItEquipment, 36),            // 3 years
                (AssetClass::Software, 36),               // 3 years
                (AssetClass::LeaseholdImprovements, 120), // 10 years
                (AssetClass::Land, 0),                    // Not depreciated
                (AssetClass::ConstructionInProgress, 0),  // Not depreciated
            ],
            acquisition_cost_range: (Decimal::from(1_000), Decimal::from(500_000)),
            salvage_value_percent: 0.05,
            fully_depreciated_rate: 0.10,
            disposed_rate: 0.02,
        }
    }
}

/// Asset description templates by class.
const ASSET_DESCRIPTIONS: &[(AssetClass, &[&str])] = &[
    (
        AssetClass::Buildings,
        &[
            "Corporate Office Building",
            "Manufacturing Facility",
            "Warehouse Complex",
            "Distribution Center",
            "Research Laboratory",
            "Administrative Building",
        ],
    ),
    (
        AssetClass::Machinery,
        &[
            "Production Line Equipment",
            "CNC Machining Center",
            "Assembly Robot System",
            "Industrial Press Machine",
            "Packaging Equipment",
            "Testing Equipment",
            "Quality Control System",
            "Material Handling System",
        ],
    ),
    (
        AssetClass::Vehicles,
        &[
            "Delivery Truck",
            "Company Car",
            "Forklift",
            "Van Fleet Unit",
            "Executive Vehicle",
            "Service Vehicle",
            "Cargo Truck",
            "Utility Vehicle",
        ],
    ),
    (
        AssetClass::Furniture,
        &[
            "Office Workstation Set",
            "Conference Room Furniture",
            "Executive Desk Set",
            "Reception Area Furniture",
            "Cubicle System",
            "Storage Cabinet Set",
            "Meeting Room Table",
            "Ergonomic Chair Set",
        ],
    ),
    (
        AssetClass::ItEquipment,
        &[
            "Server Rack System",
            "Network Switch Array",
            "Desktop Computer Set",
            "Laptop Fleet",
            "Storage Array",
            "Backup System",
            "Security System",
            "Communication System",
        ],
    ),
    (
        AssetClass::Software,
        &[
            "ERP System License",
            "CAD Software Suite",
            "Database License",
            "Office Suite License",
            "Security Software",
            "Development Tools",
            "Analytics Platform",
            "CRM System",
        ],
    ),
    (
        AssetClass::LeaseholdImprovements,
        &[
            "Office Build-out",
            "HVAC Improvements",
            "Electrical Upgrades",
            "Floor Renovations",
            "Lighting System",
            "Security Improvements",
            "Accessibility Upgrades",
            "IT Infrastructure",
        ],
    ),
];

/// Generator for fixed asset master data.
pub struct AssetGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: AssetGeneratorConfig,
    asset_counter: usize,
}

impl AssetGenerator {
    /// Create a new asset generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, AssetGeneratorConfig::default())
    }

    /// Create a new asset generator with custom configuration.
    pub fn with_config(seed: u64, config: AssetGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            asset_counter: 0,
        }
    }

    /// Generate a single fixed asset.
    pub fn generate_asset(
        &mut self,
        company_code: &str,
        acquisition_date: NaiveDate,
    ) -> FixedAsset {
        self.asset_counter += 1;

        let asset_id = format!("FA-{}-{:06}", company_code, self.asset_counter);
        let asset_class = self.select_asset_class();
        let description = self.select_description(&asset_class);

        let mut asset = FixedAsset::new(
            asset_id,
            description.to_string(),
            asset_class,
            company_code,
            acquisition_date,
            self.generate_acquisition_cost(),
        );

        // Set depreciation parameters
        asset.depreciation_method = self.select_depreciation_method(&asset_class);
        asset.useful_life_months = self.get_useful_life(&asset_class);
        asset.salvage_value = (asset.acquisition_cost
            * Decimal::from_f64_retain(self.config.salvage_value_percent)
                .unwrap_or(Decimal::from_f64_retain(0.05).expect("valid decimal literal")))
        .round_dp(2);

        // Set account determination
        asset.account_determination = self.generate_account_determination(&asset_class);

        // Set location info
        asset.location = Some(format!("P{}", company_code));
        asset.cost_center = Some(format!("CC-{}-ADMIN", company_code));

        // Generate serial number for equipment
        if matches!(
            asset_class,
            AssetClass::Machinery | AssetClass::Vehicles | AssetClass::ItEquipment
        ) {
            asset.serial_number = Some(self.generate_serial_number());
        }

        // Handle fully depreciated or disposed assets
        if self.rng.random::<f64>() < self.config.disposed_rate {
            let disposal_date =
                acquisition_date + chrono::Duration::days(self.rng.random_range(365..1825) as i64);
            let (proceeds, _gain_loss) = self.generate_disposal_values(&asset);
            asset.dispose(disposal_date, proceeds);
        } else if self.rng.random::<f64>() < self.config.fully_depreciated_rate {
            asset.accumulated_depreciation = asset.acquisition_cost - asset.salvage_value;
            asset.net_book_value = asset.salvage_value;
        }

        asset
    }

    /// Generate an asset with specific class.
    pub fn generate_asset_of_class(
        &mut self,
        asset_class: AssetClass,
        company_code: &str,
        acquisition_date: NaiveDate,
    ) -> FixedAsset {
        self.asset_counter += 1;

        let asset_id = format!("FA-{}-{:06}", company_code, self.asset_counter);
        let description = self.select_description(&asset_class);

        let mut asset = FixedAsset::new(
            asset_id,
            description.to_string(),
            asset_class,
            company_code,
            acquisition_date,
            self.generate_acquisition_cost_for_class(&asset_class),
        );

        asset.depreciation_method = self.select_depreciation_method(&asset_class);
        asset.useful_life_months = self.get_useful_life(&asset_class);
        asset.salvage_value = (asset.acquisition_cost
            * Decimal::from_f64_retain(self.config.salvage_value_percent)
                .unwrap_or(Decimal::from_f64_retain(0.05).expect("valid decimal literal")))
        .round_dp(2);

        asset.account_determination = self.generate_account_determination(&asset_class);
        asset.location = Some(format!("P{}", company_code));
        asset.cost_center = Some(format!("CC-{}-ADMIN", company_code));

        if matches!(
            asset_class,
            AssetClass::Machinery | AssetClass::Vehicles | AssetClass::ItEquipment
        ) {
            asset.serial_number = Some(self.generate_serial_number());
        }

        asset
    }

    /// Generate an asset with depreciation already applied.
    pub fn generate_aged_asset(
        &mut self,
        company_code: &str,
        acquisition_date: NaiveDate,
        as_of_date: NaiveDate,
    ) -> FixedAsset {
        let mut asset = self.generate_asset(company_code, acquisition_date);

        // Calculate months elapsed
        let months_elapsed = ((as_of_date - acquisition_date).num_days() / 30) as u32;

        // Apply depreciation for each month
        for month_offset in 0..months_elapsed {
            if asset.status == AssetStatus::Active {
                // Calculate the depreciation date for this month
                let dep_date =
                    acquisition_date + chrono::Duration::days((month_offset as i64 + 1) * 30);
                let depreciation = asset.calculate_monthly_depreciation(dep_date);
                asset.apply_depreciation(depreciation);
            }
        }

        asset
    }

    /// Generate an asset pool with specified count.
    pub fn generate_asset_pool(
        &mut self,
        count: usize,
        company_code: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> FixedAssetPool {
        debug!(count, company_code, "Generating fixed asset pool");
        let mut pool = FixedAssetPool::new();

        let (start_date, end_date) = date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        for _ in 0..count {
            let acquisition_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let asset = self.generate_asset(company_code, acquisition_date);
            pool.add_asset(asset);
        }

        pool
    }

    /// Generate an asset pool with aged assets (depreciation applied).
    pub fn generate_aged_asset_pool(
        &mut self,
        count: usize,
        company_code: &str,
        acquisition_date_range: (NaiveDate, NaiveDate),
        as_of_date: NaiveDate,
    ) -> FixedAssetPool {
        let mut pool = FixedAssetPool::new();

        let (start_date, end_date) = acquisition_date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        for _ in 0..count {
            let acquisition_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let asset = self.generate_aged_asset(company_code, acquisition_date, as_of_date);
            pool.add_asset(asset);
        }

        pool
    }

    /// Generate a diverse asset pool with various classes.
    pub fn generate_diverse_pool(
        &mut self,
        count: usize,
        company_code: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> FixedAssetPool {
        let mut pool = FixedAssetPool::new();

        let (start_date, end_date) = date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        // Define class distribution
        let class_counts = [
            (AssetClass::Buildings, (count as f64 * 0.05) as usize),
            (AssetClass::Machinery, (count as f64 * 0.25) as usize),
            (AssetClass::Vehicles, (count as f64 * 0.15) as usize),
            (AssetClass::Furniture, (count as f64 * 0.15) as usize),
            (AssetClass::ItEquipment, (count as f64 * 0.25) as usize),
            (AssetClass::Software, (count as f64 * 0.10) as usize),
            (
                AssetClass::LeaseholdImprovements,
                (count as f64 * 0.05) as usize,
            ),
        ];

        for (class, class_count) in class_counts {
            for _ in 0..class_count {
                let acquisition_date = start_date
                    + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
                let asset = self.generate_asset_of_class(class, company_code, acquisition_date);
                pool.add_asset(asset);
            }
        }

        // Fill remaining slots
        while pool.assets.len() < count {
            let acquisition_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let asset = self.generate_asset(company_code, acquisition_date);
            pool.add_asset(asset);
        }

        pool
    }

    /// Select asset class based on distribution.
    fn select_asset_class(&mut self) -> AssetClass {
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (class, prob) in &self.config.asset_class_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *class;
            }
        }

        AssetClass::ItEquipment
    }

    /// Select depreciation method based on distribution and asset class.
    fn select_depreciation_method(&mut self, asset_class: &AssetClass) -> DepreciationMethod {
        // Land and CIP are not depreciated
        if matches!(
            asset_class,
            AssetClass::Land | AssetClass::ConstructionInProgress
        ) {
            return DepreciationMethod::StraightLine; // Won't be used but needs a value
        }

        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (method, prob) in &self.config.depreciation_method_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *method;
            }
        }

        DepreciationMethod::StraightLine
    }

    /// Get useful life for asset class.
    fn get_useful_life(&self, asset_class: &AssetClass) -> u32 {
        for (class, months) in &self.config.useful_life_by_class {
            if class == asset_class {
                return *months;
            }
        }
        60 // Default 5 years
    }

    /// Select description for asset class.
    fn select_description(&mut self, asset_class: &AssetClass) -> &'static str {
        for (class, descriptions) in ASSET_DESCRIPTIONS {
            if class == asset_class {
                let idx = self.rng.random_range(0..descriptions.len());
                return descriptions[idx];
            }
        }
        "Fixed Asset"
    }

    /// Generate acquisition cost.
    fn generate_acquisition_cost(&mut self) -> Decimal {
        let min = self.config.acquisition_cost_range.0;
        let max = self.config.acquisition_cost_range.1;
        let range = (max - min).to_string().parse::<f64>().unwrap_or(0.0);
        let offset =
            Decimal::from_f64_retain(self.rng.random::<f64>() * range).unwrap_or(Decimal::ZERO);
        (min + offset).round_dp(2)
    }

    /// Generate acquisition cost for specific asset class.
    fn generate_acquisition_cost_for_class(&mut self, asset_class: &AssetClass) -> Decimal {
        let (min, max) = match asset_class {
            AssetClass::Buildings => (Decimal::from(500_000), Decimal::from(10_000_000)),
            AssetClass::BuildingImprovements => (Decimal::from(50_000), Decimal::from(500_000)),
            AssetClass::Machinery | AssetClass::MachineryEquipment => {
                (Decimal::from(50_000), Decimal::from(1_000_000))
            }
            AssetClass::Vehicles => (Decimal::from(20_000), Decimal::from(100_000)),
            AssetClass::Furniture | AssetClass::FurnitureFixtures => {
                (Decimal::from(1_000), Decimal::from(50_000))
            }
            AssetClass::ItEquipment | AssetClass::ComputerHardware => {
                (Decimal::from(2_000), Decimal::from(200_000))
            }
            AssetClass::Software | AssetClass::Intangibles => {
                (Decimal::from(5_000), Decimal::from(500_000))
            }
            AssetClass::LeaseholdImprovements => (Decimal::from(10_000), Decimal::from(300_000)),
            AssetClass::Land => (Decimal::from(100_000), Decimal::from(5_000_000)),
            AssetClass::ConstructionInProgress => {
                (Decimal::from(100_000), Decimal::from(2_000_000))
            }
            AssetClass::LowValueAssets => (Decimal::from(100), Decimal::from(5_000)),
        };

        let range = (max - min).to_string().parse::<f64>().unwrap_or(0.0);
        let offset =
            Decimal::from_f64_retain(self.rng.random::<f64>() * range).unwrap_or(Decimal::ZERO);
        (min + offset).round_dp(2)
    }

    /// Generate serial number.
    fn generate_serial_number(&mut self) -> String {
        format!(
            "SN-{:04}-{:08}",
            self.rng.random_range(1000..9999),
            self.rng.random_range(10000000..99999999)
        )
    }

    /// Generate disposal values.
    fn generate_disposal_values(&mut self, asset: &FixedAsset) -> (Decimal, Decimal) {
        // Disposal proceeds typically 0-50% of acquisition cost
        let proceeds_rate = self.rng.random::<f64>() * 0.5;
        let proceeds = (asset.acquisition_cost
            * Decimal::from_f64_retain(proceeds_rate).unwrap_or(Decimal::ZERO))
        .round_dp(2);

        // Gain/loss = proceeds - NBV (can be negative)
        let nbv = asset.net_book_value;
        let gain_loss = proceeds - nbv;

        (proceeds, gain_loss)
    }

    /// Generate account determination for asset class.
    fn generate_account_determination(
        &self,
        asset_class: &AssetClass,
    ) -> AssetAccountDetermination {
        match asset_class {
            AssetClass::Buildings | AssetClass::BuildingImprovements => AssetAccountDetermination {
                asset_account: "160000".to_string(),
                accumulated_depreciation_account: "165000".to_string(),
                depreciation_expense_account: "680000".to_string(),
                gain_loss_account: "790000".to_string(),
                gain_on_disposal_account: "790010".to_string(),
                loss_on_disposal_account: "790020".to_string(),
                acquisition_clearing_account: "199100".to_string(),
            },
            AssetClass::Machinery | AssetClass::MachineryEquipment => AssetAccountDetermination {
                asset_account: "161000".to_string(),
                accumulated_depreciation_account: "166000".to_string(),
                depreciation_expense_account: "681000".to_string(),
                gain_loss_account: "791000".to_string(),
                gain_on_disposal_account: "791010".to_string(),
                loss_on_disposal_account: "791020".to_string(),
                acquisition_clearing_account: "199110".to_string(),
            },
            AssetClass::Vehicles => AssetAccountDetermination {
                asset_account: "162000".to_string(),
                accumulated_depreciation_account: "167000".to_string(),
                depreciation_expense_account: "682000".to_string(),
                gain_loss_account: "792000".to_string(),
                gain_on_disposal_account: "792010".to_string(),
                loss_on_disposal_account: "792020".to_string(),
                acquisition_clearing_account: "199120".to_string(),
            },
            AssetClass::Furniture | AssetClass::FurnitureFixtures => AssetAccountDetermination {
                asset_account: "163000".to_string(),
                accumulated_depreciation_account: "168000".to_string(),
                depreciation_expense_account: "683000".to_string(),
                gain_loss_account: "793000".to_string(),
                gain_on_disposal_account: "793010".to_string(),
                loss_on_disposal_account: "793020".to_string(),
                acquisition_clearing_account: "199130".to_string(),
            },
            AssetClass::ItEquipment | AssetClass::ComputerHardware => AssetAccountDetermination {
                asset_account: "164000".to_string(),
                accumulated_depreciation_account: "169000".to_string(),
                depreciation_expense_account: "684000".to_string(),
                gain_loss_account: "794000".to_string(),
                gain_on_disposal_account: "794010".to_string(),
                loss_on_disposal_account: "794020".to_string(),
                acquisition_clearing_account: "199140".to_string(),
            },
            AssetClass::Software | AssetClass::Intangibles => AssetAccountDetermination {
                asset_account: "170000".to_string(),
                accumulated_depreciation_account: "175000".to_string(),
                depreciation_expense_account: "685000".to_string(),
                gain_loss_account: "795000".to_string(),
                gain_on_disposal_account: "795010".to_string(),
                loss_on_disposal_account: "795020".to_string(),
                acquisition_clearing_account: "199150".to_string(),
            },
            AssetClass::LeaseholdImprovements => AssetAccountDetermination {
                asset_account: "171000".to_string(),
                accumulated_depreciation_account: "176000".to_string(),
                depreciation_expense_account: "686000".to_string(),
                gain_loss_account: "796000".to_string(),
                gain_on_disposal_account: "796010".to_string(),
                loss_on_disposal_account: "796020".to_string(),
                acquisition_clearing_account: "199160".to_string(),
            },
            AssetClass::Land => {
                AssetAccountDetermination {
                    asset_account: "150000".to_string(),
                    accumulated_depreciation_account: "".to_string(), // Land not depreciated
                    depreciation_expense_account: "".to_string(),
                    gain_loss_account: "790000".to_string(),
                    gain_on_disposal_account: "790010".to_string(),
                    loss_on_disposal_account: "790020".to_string(),
                    acquisition_clearing_account: "199000".to_string(),
                }
            }
            AssetClass::ConstructionInProgress => AssetAccountDetermination {
                asset_account: "159000".to_string(),
                accumulated_depreciation_account: "".to_string(),
                depreciation_expense_account: "".to_string(),
                gain_loss_account: "".to_string(),
                gain_on_disposal_account: "".to_string(),
                loss_on_disposal_account: "".to_string(),
                acquisition_clearing_account: "199090".to_string(),
            },
            AssetClass::LowValueAssets => AssetAccountDetermination {
                asset_account: "172000".to_string(),
                accumulated_depreciation_account: "177000".to_string(),
                depreciation_expense_account: "687000".to_string(),
                gain_loss_account: "797000".to_string(),
                gain_on_disposal_account: "797010".to_string(),
                loss_on_disposal_account: "797020".to_string(),
                acquisition_clearing_account: "199170".to_string(),
            },
        }
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.asset_counter = 0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_generation() {
        let mut gen = AssetGenerator::new(42);
        let asset = gen.generate_asset("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!asset.asset_id.is_empty());
        assert!(!asset.description.is_empty());
        assert!(asset.acquisition_cost > Decimal::ZERO);
        assert!(
            asset.useful_life_months > 0
                || matches!(
                    asset.asset_class,
                    AssetClass::Land | AssetClass::ConstructionInProgress
                )
        );
    }

    #[test]
    fn test_asset_pool_generation() {
        let mut gen = AssetGenerator::new(42);
        let pool = gen.generate_asset_pool(
            50,
            "1000",
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        assert_eq!(pool.assets.len(), 50);
    }

    #[test]
    fn test_aged_asset() {
        let mut gen = AssetGenerator::new(42);
        let asset = gen.generate_aged_asset(
            "1000",
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Should have accumulated depreciation
        if asset.status == AssetStatus::Active && asset.useful_life_months > 0 {
            assert!(asset.accumulated_depreciation > Decimal::ZERO);
            assert!(asset.net_book_value < asset.acquisition_cost);
        }
    }

    #[test]
    fn test_diverse_pool() {
        let mut gen = AssetGenerator::new(42);
        let pool = gen.generate_diverse_pool(
            100,
            "1000",
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        // Should have various asset classes
        let machinery_count = pool
            .assets
            .iter()
            .filter(|a| a.asset_class == AssetClass::Machinery)
            .count();
        let it_count = pool
            .assets
            .iter()
            .filter(|a| a.asset_class == AssetClass::ItEquipment)
            .count();

        assert!(machinery_count > 0);
        assert!(it_count > 0);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = AssetGenerator::new(42);
        let mut gen2 = AssetGenerator::new(42);

        let asset1 = gen1.generate_asset("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let asset2 = gen2.generate_asset("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(asset1.asset_id, asset2.asset_id);
        assert_eq!(asset1.description, asset2.description);
        assert_eq!(asset1.acquisition_cost, asset2.acquisition_cost);
    }

    #[test]
    fn test_depreciation_calculation() {
        let mut gen = AssetGenerator::new(42);
        let mut asset = gen.generate_asset_of_class(
            AssetClass::ItEquipment,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let initial_nbv = asset.net_book_value;

        // Apply one month of depreciation
        let depreciation =
            asset.calculate_monthly_depreciation(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        asset.apply_depreciation(depreciation);

        assert!(asset.accumulated_depreciation > Decimal::ZERO);
        assert!(asset.net_book_value < initial_nbv);
    }

    #[test]
    fn test_asset_class_cost_ranges() {
        let mut gen = AssetGenerator::new(42);

        // Buildings should be more expensive than furniture
        let building = gen.generate_asset_of_class(
            AssetClass::Buildings,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        let furniture = gen.generate_asset_of_class(
            AssetClass::Furniture,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Buildings min is 500k, furniture max is 50k
        assert!(building.acquisition_cost >= Decimal::from(500_000));
        assert!(furniture.acquisition_cost <= Decimal::from(50_000));
    }
}
