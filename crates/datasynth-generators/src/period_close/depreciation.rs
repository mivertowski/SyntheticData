//! Depreciation run generator for period close.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::debug;

use datasynth_core::models::subledger::fa::{
    AssetStatus, DepreciationAreaType, DepreciationEntry, DepreciationRun, DepreciationRunStatus,
    FixedAssetRecord,
};
use datasynth_core::models::{FiscalPeriod, JournalEntry, JournalEntryLine};

/// Configuration for depreciation run.
#[derive(Debug, Clone)]
pub struct DepreciationRunConfig {
    /// Default depreciation expense account.
    pub default_expense_account: String,
    /// Default accumulated depreciation account.
    pub default_accum_depr_account: String,
    /// Whether to post zero depreciation entries.
    pub post_zero_entries: bool,
    /// Minimum depreciation amount to post.
    pub minimum_amount: Decimal,
}

impl Default for DepreciationRunConfig {
    fn default() -> Self {
        Self {
            default_expense_account: "6100".to_string(),
            default_accum_depr_account: "1510".to_string(),
            post_zero_entries: false,
            minimum_amount: dec!(0.01),
        }
    }
}

/// Generator for depreciation runs.
pub struct DepreciationRunGenerator {
    config: DepreciationRunConfig,
    run_counter: u64,
}

impl DepreciationRunGenerator {
    /// Creates a new depreciation run generator.
    pub fn new(config: DepreciationRunConfig) -> Self {
        Self {
            config,
            run_counter: 0,
        }
    }

    /// Executes a depreciation run for a company.
    pub fn execute_run(
        &mut self,
        company_code: &str,
        assets: &mut [FixedAssetRecord],
        fiscal_period: &FiscalPeriod,
    ) -> DepreciationRunResult {
        debug!(
            company_code,
            asset_count = assets.len(),
            period = fiscal_period.period,
            year = fiscal_period.year,
            "Executing depreciation run"
        );
        self.run_counter += 1;
        let run_id = format!("DEPR-{}-{:08}", company_code, self.run_counter);

        let mut run = DepreciationRun::new(
            run_id.clone(),
            company_code.to_string(),
            fiscal_period.year,
            fiscal_period.period as u32,
            DepreciationAreaType::Book,
            fiscal_period.end_date,
            "SYSTEM".to_string(),
        );

        run.start();

        let mut journal_entries = Vec::new();
        let errors = Vec::new();

        for asset in assets.iter_mut() {
            // Skip non-active assets
            if asset.status != AssetStatus::Active {
                continue;
            }

            // Skip if company doesn't match
            if asset.company_code != company_code {
                continue;
            }

            // Create entry from asset using the from_asset method
            if let Some(entry) = DepreciationEntry::from_asset(asset, DepreciationAreaType::Book) {
                if entry.depreciation_amount < self.config.minimum_amount
                    && !self.config.post_zero_entries
                {
                    continue;
                }

                // Generate journal entry
                let je = self.generate_depreciation_je(asset, &entry, fiscal_period);

                // Update asset's depreciation
                asset.record_depreciation(entry.depreciation_amount, DepreciationAreaType::Book);

                // Check if fully depreciated
                if asset.is_fully_depreciated() {
                    asset.status = AssetStatus::FullyDepreciated;
                }

                run.add_entry(entry);
                journal_entries.push(je);
            }
        }

        run.complete();

        DepreciationRunResult {
            run,
            journal_entries,
            errors,
        }
    }

    /// Generates the journal entry for a depreciation entry.
    fn generate_depreciation_je(
        &self,
        asset: &FixedAssetRecord,
        entry: &DepreciationEntry,
        period: &FiscalPeriod,
    ) -> JournalEntry {
        let expense_account = if entry.expense_account.is_empty() {
            &self.config.default_expense_account
        } else {
            &entry.expense_account
        };

        let accum_account = if entry.accum_depr_account.is_empty() {
            &self.config.default_accum_depr_account
        } else {
            &entry.accum_depr_account
        };

        let mut je = JournalEntry::new_simple(
            format!("DEPR-{}", asset.asset_number),
            asset.company_code.clone(),
            period.end_date,
            format!(
                "Depreciation {} P{}/{}",
                asset.asset_number, period.year, period.period
            ),
        );

        // Debit Depreciation Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: expense_account.to_string(),
            debit_amount: entry.depreciation_amount,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_number.clone()),
            assignment: Some(format!("{:?}", asset.asset_class)),
            text: Some(asset.description.clone()),
            ..Default::default()
        });

        // Credit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: accum_account.to_string(),
            credit_amount: entry.depreciation_amount,
            reference: Some(asset.asset_number.clone()),
            assignment: Some(format!("{:?}", asset.asset_class)),
            ..Default::default()
        });

        je
    }

    /// Generates a depreciation forecast for planning purposes.
    pub fn forecast_depreciation(
        &self,
        assets: &[FixedAssetRecord],
        start_period: &FiscalPeriod,
        months: u32,
    ) -> Vec<DepreciationForecastEntry> {
        let mut forecast = Vec::new();

        // Create simulated asset states
        let mut simulated_assets: Vec<SimulatedAsset> = assets
            .iter()
            .filter(|a| a.status == AssetStatus::Active)
            .map(|a| {
                let monthly_depr = a
                    .depreciation_areas
                    .first()
                    .map(|area| area.calculate_monthly_depreciation())
                    .unwrap_or(Decimal::ZERO);
                SimulatedAsset {
                    asset_number: a.asset_number.clone(),
                    net_book_value: a.net_book_value,
                    salvage_value: a.salvage_value(),
                    monthly_depreciation: monthly_depr,
                }
            })
            .collect();

        let mut current_year = start_period.year;
        let mut current_month = start_period.period;

        for _ in 0..months {
            let period_key = format!("{}-{:02}", current_year, current_month);
            let mut period_total = Decimal::ZERO;

            for sim_asset in &mut simulated_assets {
                let remaining = sim_asset.net_book_value - sim_asset.salvage_value;
                if remaining > Decimal::ZERO {
                    let depr = sim_asset.monthly_depreciation.min(remaining);
                    sim_asset.net_book_value -= depr;
                    period_total += depr;
                }
            }

            forecast.push(DepreciationForecastEntry {
                period_key,
                fiscal_year: current_year,
                fiscal_period: current_month,
                forecasted_depreciation: period_total,
            });

            // Advance to next month
            if current_month == 12 {
                current_month = 1;
                current_year += 1;
            } else {
                current_month += 1;
            }
        }

        forecast
    }
}

/// Simulated asset state for forecasting.
struct SimulatedAsset {
    // Retained for debugging/tracing individual asset forecast lines.
    #[allow(dead_code)]
    asset_number: String,
    net_book_value: Decimal,
    salvage_value: Decimal,
    monthly_depreciation: Decimal,
}

/// Result of a depreciation run.
#[derive(Debug, Clone)]
pub struct DepreciationRunResult {
    /// The depreciation run record.
    pub run: DepreciationRun,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
    /// Errors encountered.
    pub errors: Vec<DepreciationError>,
}

impl DepreciationRunResult {
    /// Returns true if the run completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(
            self.run.status,
            DepreciationRunStatus::Completed | DepreciationRunStatus::CompletedWithErrors
        )
    }

    /// Returns the total depreciation amount.
    pub fn total_depreciation(&self) -> Decimal {
        self.run.total_depreciation
    }
}

/// Error during depreciation processing.
#[derive(Debug, Clone)]
pub struct DepreciationError {
    /// Asset ID.
    pub asset_number: String,
    /// Error message.
    pub error: String,
}

/// Depreciation forecast entry.
#[derive(Debug, Clone)]
pub struct DepreciationForecastEntry {
    /// Period key (YYYY-MM).
    pub period_key: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Forecasted depreciation amount.
    pub forecasted_depreciation: Decimal,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::subledger::fa::{AssetClass, DepreciationArea, DepreciationMethod};
    use rust_decimal_macros::dec;

    fn create_test_asset() -> FixedAssetRecord {
        let mut asset = FixedAssetRecord::new(
            "FA00001".to_string(),
            "1000".to_string(),
            AssetClass::MachineryEquipment,
            "Test Machine".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            dec!(120000),
            "USD".to_string(),
        );

        // Add depreciation area with salvage value
        let area = DepreciationArea::new(
            DepreciationAreaType::Book,
            DepreciationMethod::StraightLine,
            60, // 5 years
            dec!(120000),
        )
        .with_salvage_value(dec!(12000));

        asset.add_depreciation_area(area);
        asset.cost_center = Some("CC100".to_string());

        asset
    }

    #[test]
    fn test_depreciation_run() {
        let mut generator = DepreciationRunGenerator::new(DepreciationRunConfig::default());
        let mut assets = vec![create_test_asset()];
        let period = FiscalPeriod::monthly(2024, 1);

        let result = generator.execute_run("1000", &mut assets, &period);

        assert!(result.is_success());
        assert!(result.journal_entries.iter().all(|je| je.is_balanced()));

        // Monthly depreciation should be (120000 - 12000) / 60 = 1800
        assert_eq!(result.total_depreciation(), dec!(1800));
    }

    #[test]
    fn test_depreciation_forecast() {
        let generator = DepreciationRunGenerator::new(DepreciationRunConfig::default());
        let assets = vec![create_test_asset()];
        let period = FiscalPeriod::monthly(2024, 1);

        let forecast = generator.forecast_depreciation(&assets, &period, 12);

        assert_eq!(forecast.len(), 12);
        assert!(forecast
            .iter()
            .all(|f| f.forecasted_depreciation == dec!(1800)));
    }
}
