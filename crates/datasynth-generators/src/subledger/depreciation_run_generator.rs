//! Subledger Depreciation Schedule Generator.
//!
//! Produces periodic `DepreciationRun` records by driving the existing
//! `FAGenerator::run_depreciation()` over the set of `FixedAssetRecord`
//! entries in the subledger snapshot.  One run is emitted per fiscal period
//! for each company that has active assets.
//!
//! # Distinction from `period_close::DepreciationRunGenerator`
//!
//! The period-close variant (`DepreciationRunGenerator` in `period_close/depreciation.rs`)
//! is driven by `FiscalPeriod` objects and mutates the asset list.  This
//! subledger variant takes an immutable slice of `FixedAssetRecord` and
//! produces a complete multi-period schedule in a single call, for use when
//! populating the `SubledgerSnapshot`.

use chrono::{Datelike, NaiveDate};
use rand::SeedableRng;

use datasynth_core::models::subledger::fa::{DepreciationRun, FixedAssetRecord};

use crate::FAGenerator;
use crate::FAGeneratorConfig;

/// Configuration for the subledger depreciation schedule generator.
#[derive(Debug, Clone)]
pub struct FaDepreciationScheduleConfig {
    /// Fiscal year to generate runs for.
    pub fiscal_year: i32,
    /// First fiscal period (1 = January if calendar year, or first month of FY).
    pub start_period: u32,
    /// Last fiscal period (inclusive).
    pub end_period: u32,
    /// Seed offset added to the base seed so runs are deterministic but
    /// independent of other FA generator uses.
    pub seed_offset: u64,
}

impl Default for FaDepreciationScheduleConfig {
    fn default() -> Self {
        let year = chrono::Utc::now().date_naive().year();
        Self {
            fiscal_year: year,
            start_period: 1,
            end_period: 12,
            seed_offset: 800,
        }
    }
}

/// Generator that creates one `DepreciationRun` per fiscal period from a
/// subledger FA snapshot.
pub struct FaDepreciationScheduleGenerator {
    config: FaDepreciationScheduleConfig,
    seed: u64,
}

impl FaDepreciationScheduleGenerator {
    /// Creates a new generator with the given base seed.
    pub fn new(config: FaDepreciationScheduleConfig, seed: u64) -> Self {
        Self { config, seed }
    }

    /// Generates depreciation runs for all periods between `start_period` and
    /// `end_period` (inclusive) for the given company and asset set.
    ///
    /// Returns a `Vec<DepreciationRun>` — one entry per period that contains at
    /// least one active, non-fully-depreciated asset.
    pub fn generate(
        &self,
        company_code: &str,
        fa_records: &[FixedAssetRecord],
    ) -> Vec<DepreciationRun> {
        if fa_records.is_empty() {
            return Vec::new();
        }

        let mut fa_gen = FAGenerator::new(
            FAGeneratorConfig::default(),
            rand_chacha::ChaCha8Rng::seed_from_u64(self.seed + self.config.seed_offset),
        );

        let asset_refs: Vec<&FixedAssetRecord> = fa_records.iter().collect();

        let mut runs = Vec::new();

        for period in self.config.start_period..=self.config.end_period {
            // Build a period date: last day of the month for this period in the FY.
            // Periods are assumed to be calendar months (period 1 = Jan, 12 = Dec).
            let (year, month) = if period > 12 {
                // Handle non-standard period numbering gracefully.
                (self.config.fiscal_year, 12u32)
            } else {
                (self.config.fiscal_year, period)
            };

            let period_date = last_day_of_month(year, month);

            let (run, _jes) = fa_gen.run_depreciation(
                company_code,
                &asset_refs,
                period_date,
                self.config.fiscal_year,
                period,
            );

            if run.asset_count > 0 {
                runs.push(run);
            }
        }

        runs
    }
}

/// Returns the last day of the given year/month.
fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid next-month date")
        .pred_opt()
        .expect("valid last-day date")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::subledger::fa::{
        AssetClass, DepreciationArea, DepreciationAreaType, DepreciationMethod,
    };
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn make_asset(id: &str, company: &str, acquisition_cost: Decimal) -> FixedAssetRecord {
        let mut asset = FixedAssetRecord::new(
            id.to_string(),
            company.to_string(),
            AssetClass::MachineryEquipment,
            format!("Machine {id}"),
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            acquisition_cost,
            "USD".to_string(),
        );
        let area = DepreciationArea::new(
            DepreciationAreaType::Book,
            DepreciationMethod::StraightLine,
            60, // 5 years = 60 months
            acquisition_cost,
        );
        asset.add_depreciation_area(area);
        asset
    }

    #[test]
    fn test_straight_line_monthly_amount() {
        // acquisition_cost = 60_000, useful_life = 60 months, salvage = 0
        // monthly depreciation = 60_000 / 60 = 1_000
        let asset = make_asset("A001", "1000", dec!(60_000));
        let cfg = FaDepreciationScheduleConfig {
            fiscal_year: 2024,
            start_period: 1,
            end_period: 1,
            seed_offset: 0,
        };
        let gen = FaDepreciationScheduleGenerator::new(cfg, 42);
        let runs = gen.generate("1000", &[asset]);

        assert_eq!(runs.len(), 1, "Expected exactly one run for period 1");
        let run = &runs[0];
        assert_eq!(run.fiscal_period, 1);
        assert_eq!(run.asset_count, 1);
        assert_eq!(
            run.total_depreciation,
            dec!(1_000),
            "Straight-line monthly depreciation should be 1_000"
        );
    }

    #[test]
    fn test_accumulated_increases_each_period() {
        // Two periods for the same asset — verify both runs are generated.
        let asset = make_asset("A002", "1000", dec!(120_000));
        let cfg = FaDepreciationScheduleConfig {
            fiscal_year: 2024,
            start_period: 1,
            end_period: 2,
            seed_offset: 1,
        };
        let gen = FaDepreciationScheduleGenerator::new(cfg, 99);
        let runs = gen.generate("1000", &[asset]);

        assert_eq!(runs.len(), 2, "Expected two runs (one per period)");
        // Both runs should have positive depreciation.
        for run in &runs {
            assert!(
                run.total_depreciation > Decimal::ZERO,
                "Depreciation should be positive"
            );
        }
        // Straight-line gives equal per-period amounts.
        assert_eq!(
            runs[0].total_depreciation,
            runs[1].total_depreciation,
            "Straight-line gives equal depreciation each period"
        );
    }

    #[test]
    fn test_empty_fa_records_returns_empty() {
        let cfg = FaDepreciationScheduleConfig::default();
        let gen = FaDepreciationScheduleGenerator::new(cfg, 0);
        let runs = gen.generate("1000", &[]);
        assert!(runs.is_empty());
    }

    #[test]
    fn test_twelve_periods_generated() {
        let asset = make_asset("A003", "1000", dec!(36_000));
        let cfg = FaDepreciationScheduleConfig {
            fiscal_year: 2024,
            start_period: 1,
            end_period: 12,
            seed_offset: 2,
        };
        let gen = FaDepreciationScheduleGenerator::new(cfg, 7);
        let runs = gen.generate("1000", &[asset]);
        assert_eq!(runs.len(), 12, "Should produce 12 monthly runs");
    }
}
