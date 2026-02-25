//! Fixed Assets (FA) generator.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use tracing::debug;

use datasynth_core::models::subledger::fa::{
    AssetClass, AssetDisposal, AssetStatus, DepreciationArea, DepreciationAreaType,
    DepreciationEntry, DepreciationMethod, DepreciationRun, DisposalReason, DisposalType,
    FixedAssetRecord,
};
use datasynth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for FA generation.
#[derive(Debug, Clone)]
pub struct FAGeneratorConfig {
    /// Default depreciation method.
    pub default_depreciation_method: DepreciationMethod,
    /// Default useful life in months.
    pub default_useful_life_months: u32,
    /// Salvage value percentage.
    pub salvage_value_percent: Decimal,
    /// Average acquisition cost.
    pub avg_acquisition_cost: Decimal,
    /// Cost variation factor.
    pub cost_variation: Decimal,
    /// Disposal rate per year.
    pub annual_disposal_rate: Decimal,
}

impl Default for FAGeneratorConfig {
    fn default() -> Self {
        Self {
            default_depreciation_method: DepreciationMethod::StraightLine,
            default_useful_life_months: 60,
            salvage_value_percent: dec!(0.10),
            avg_acquisition_cost: dec!(50000),
            cost_variation: dec!(0.7),
            annual_disposal_rate: dec!(0.05),
        }
    }
}

/// Generator for Fixed Assets transactions.
pub struct FAGenerator {
    config: FAGeneratorConfig,
    rng: ChaCha8Rng,
    asset_counter: u64,
    depreciation_run_counter: u64,
    disposal_counter: u64,
}

impl FAGenerator {
    /// Creates a new FA generator.
    pub fn new(config: FAGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            asset_counter: 0,
            depreciation_run_counter: 0,
            disposal_counter: 0,
        }
    }

    /// Creates a new FA generator from a seed, constructing the RNG internally.
    pub fn with_seed(config: FAGeneratorConfig, seed: u64) -> Self {
        Self::new(config, seeded_rng(seed, 0))
    }

    /// Maps a string asset class to the enum.
    fn parse_asset_class(class_str: &str) -> AssetClass {
        match class_str.to_uppercase().as_str() {
            "LAND" => AssetClass::Land,
            "BUILDINGS" | "BUILDING" => AssetClass::Buildings,
            "MACHINERY" | "EQUIPMENT" | "MACHINERY_EQUIPMENT" => AssetClass::MachineryEquipment,
            "VEHICLES" | "VEHICLE" => AssetClass::Vehicles,
            "FURNITURE" | "FIXTURES" => AssetClass::FurnitureFixtures,
            "COMPUTER" | "IT" | "IT_EQUIPMENT" => AssetClass::ComputerEquipment,
            "SOFTWARE" => AssetClass::Software,
            "LEASEHOLD" | "LEASEHOLD_IMPROVEMENTS" => AssetClass::LeaseholdImprovements,
            _ => AssetClass::Other,
        }
    }

    /// Generates a new fixed asset acquisition.
    pub fn generate_asset_acquisition(
        &mut self,
        company_code: &str,
        asset_class_str: &str,
        description: &str,
        acquisition_date: NaiveDate,
        currency: &str,
        cost_center: Option<&str>,
    ) -> (FixedAssetRecord, JournalEntry) {
        debug!(company_code, asset_class_str, %acquisition_date, "Generating FA asset acquisition");
        self.asset_counter += 1;
        let asset_number = format!("FA{:08}", self.asset_counter);
        let asset_class = Self::parse_asset_class(asset_class_str);

        let acquisition_cost = self.generate_acquisition_cost();
        let salvage_value = (acquisition_cost * self.config.salvage_value_percent).round_dp(2);

        // Create the asset using the constructor
        let mut asset = FixedAssetRecord::new(
            asset_number,
            company_code.to_string(),
            asset_class,
            description.to_string(),
            acquisition_date,
            acquisition_cost,
            currency.to_string(),
        );

        // Add serial number and inventory number
        asset.serial_number = Some(format!("SN-{:010}", self.rng.random::<u32>()));
        asset.inventory_number = Some(format!("INV-{:08}", self.asset_counter));
        asset.cost_center = cost_center.map(|s| s.to_string());

        // Add a depreciation area
        let mut depreciation_area = DepreciationArea::new(
            DepreciationAreaType::Book,
            self.config.default_depreciation_method,
            self.config.default_useful_life_months,
            acquisition_cost,
        );
        depreciation_area.salvage_value = salvage_value;
        asset.add_depreciation_area(depreciation_area);

        let je = self.generate_acquisition_je(&asset);
        (asset, je)
    }

    /// Runs depreciation for a period.
    pub fn run_depreciation(
        &mut self,
        company_code: &str,
        assets: &[&FixedAssetRecord],
        period_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> (DepreciationRun, Vec<JournalEntry>) {
        self.depreciation_run_counter += 1;
        let run_id = format!("DEPR{:08}", self.depreciation_run_counter);

        let mut run = DepreciationRun::new(
            run_id,
            company_code.to_string(),
            fiscal_year,
            fiscal_period,
            DepreciationAreaType::Book,
            period_date,
            "FAGenerator".to_string(),
        );

        run.start();
        let mut journal_entries = Vec::new();

        for asset in assets {
            if asset.status != AssetStatus::Active {
                continue;
            }

            // Create entry from asset using the from_asset method
            if let Some(entry) = DepreciationEntry::from_asset(asset, DepreciationAreaType::Book) {
                if entry.depreciation_amount <= Decimal::ZERO {
                    continue;
                }

                let je = self.generate_depreciation_je(asset, &entry, period_date);
                run.add_entry(entry);
                journal_entries.push(je);
            }
        }

        run.complete();
        (run, journal_entries)
    }

    /// Generates an asset disposal.
    pub fn generate_disposal(
        &mut self,
        asset: &FixedAssetRecord,
        disposal_date: NaiveDate,
        disposal_type: DisposalType,
        proceeds: Decimal,
    ) -> (AssetDisposal, JournalEntry) {
        self.disposal_counter += 1;
        let disposal_id = format!("DISP{:08}", self.disposal_counter);

        let disposal_reason = self.random_disposal_reason();

        // Use the appropriate constructor based on disposal type
        let mut disposal = if disposal_type == DisposalType::Sale && proceeds > Decimal::ZERO {
            AssetDisposal::sale(
                disposal_id,
                asset,
                disposal_date,
                proceeds,
                format!("CUST-{}", self.disposal_counter),
                "FAGenerator".to_string(),
            )
        } else {
            let mut d = AssetDisposal::new(
                disposal_id,
                asset,
                disposal_date,
                disposal_type,
                disposal_reason,
                "FAGenerator".to_string(),
            );
            if proceeds > Decimal::ZERO {
                d = d.with_sale_proceeds(proceeds);
            } else {
                d.calculate_gain_loss();
            }
            d
        };

        // Approve the disposal
        disposal.approve("SYSTEM".to_string(), disposal_date);

        let je = self.generate_disposal_je(asset, &disposal);
        (disposal, je)
    }

    fn generate_acquisition_cost(&mut self) -> Decimal {
        let base = self.config.avg_acquisition_cost;
        let variation = base * self.config.cost_variation;
        let random: f64 = self.rng.random_range(-1.0..1.0);
        (base + variation * Decimal::try_from(random).unwrap_or_default())
            .max(dec!(1000))
            .round_dp(2)
    }

    fn random_disposal_reason(&mut self) -> DisposalReason {
        match self.rng.random_range(0..5) {
            0 => DisposalReason::Sale,
            1 => DisposalReason::EndOfLife,
            2 => DisposalReason::Obsolescence,
            3 => DisposalReason::Donated,
            _ => DisposalReason::Replacement,
        }
    }

    fn generate_acquisition_je(&self, asset: &FixedAssetRecord) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-ACQ-{}", asset.asset_number),
            asset.company_code.clone(),
            asset.acquisition_date,
            format!("Asset Acquisition {}", asset.asset_number),
        );

        // Debit Fixed Asset
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: asset.account_determination.acquisition_account.clone(),
            debit_amount: asset.acquisition_cost,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_number.clone()),
            text: Some(asset.description.clone()),
            quantity: Some(dec!(1)),
            unit: Some("EA".to_string()),
            ..Default::default()
        });

        // Credit Cash/AP (assuming cash purchase)
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: asset.account_determination.clearing_account.clone(),
            credit_amount: asset.acquisition_cost,
            reference: Some(asset.asset_number.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_depreciation_je(
        &self,
        asset: &FixedAssetRecord,
        entry: &DepreciationEntry,
        posting_date: NaiveDate,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-DEP-{}", asset.asset_number),
            asset.company_code.clone(),
            posting_date,
            format!("Depreciation {}", asset.asset_number),
        );

        // Debit Depreciation Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: entry.expense_account.clone(),
            debit_amount: entry.depreciation_amount,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_number.clone()),
            ..Default::default()
        });

        // Credit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: entry.accum_depr_account.clone(),
            credit_amount: entry.depreciation_amount,
            reference: Some(asset.asset_number.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_disposal_je(
        &self,
        asset: &FixedAssetRecord,
        disposal: &AssetDisposal,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", disposal.disposal_id),
            asset.company_code.clone(),
            disposal.disposal_date,
            format!("Asset Disposal {}", asset.asset_number),
        );

        let mut line_num = 1;

        // Debit Cash (if proceeds > 0)
        if disposal.sale_proceeds > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: "1000".to_string(),
                debit_amount: disposal.sale_proceeds,
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Debit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: line_num,
            gl_account: asset
                .account_determination
                .accumulated_depreciation_account
                .clone(),
            debit_amount: disposal.accumulated_depreciation,
            reference: Some(disposal.disposal_id.clone()),
            ..Default::default()
        });
        line_num += 1;

        // Debit Loss on Disposal (if loss)
        if !disposal.is_gain {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: asset.account_determination.loss_on_disposal_account.clone(),
                debit_amount: disposal.loss(),
                cost_center: asset.cost_center.clone(),
                profit_center: asset.profit_center.clone(),
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Credit Fixed Asset
        je.add_line(JournalEntryLine {
            line_number: line_num,
            gl_account: asset.account_determination.acquisition_account.clone(),
            credit_amount: asset.acquisition_cost,
            reference: Some(disposal.disposal_id.clone()),
            ..Default::default()
        });
        line_num += 1;

        // Credit Gain on Disposal (if gain)
        if disposal.is_gain && disposal.gain() > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: asset.account_determination.gain_on_disposal_account.clone(),
                credit_amount: disposal.gain(),
                cost_center: asset.cost_center.clone(),
                profit_center: asset.profit_center.clone(),
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
        }

        je
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::subledger::fa::DepreciationRunStatus;
    use rand::SeedableRng;

    #[test]
    fn test_generate_asset_acquisition() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = FAGenerator::new(FAGeneratorConfig::default(), rng);

        let (asset, je) = generator.generate_asset_acquisition(
            "1000",
            "MACHINERY",
            "CNC Machine",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            Some("CC100"),
        );

        assert_eq!(asset.status, AssetStatus::Active);
        assert!(asset.acquisition_cost > Decimal::ZERO);
        assert!(je.is_balanced());
    }

    #[test]
    fn test_run_depreciation() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = FAGenerator::new(FAGeneratorConfig::default(), rng);

        let (asset, _) = generator.generate_asset_acquisition(
            "1000",
            "MACHINERY",
            "CNC Machine",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            None,
        );

        let (run, jes) = generator.run_depreciation(
            "1000",
            &[&asset],
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            2024,
            1,
        );

        assert_eq!(run.status, DepreciationRunStatus::Completed);
        assert!(run.asset_count > 0);
        assert!(jes.iter().all(|je| je.is_balanced()));
    }
}
