//! Depreciation models and calculations.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DepreciationAreaType, DepreciationMethod, FixedAssetRecord};
use crate::models::subledger::GLReference;

/// Depreciation run (batch depreciation posting).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationRun {
    /// Run ID.
    pub run_id: String,
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u32,
    /// Depreciation area.
    pub depreciation_area: DepreciationAreaType,
    /// Run date.
    pub run_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Run status.
    pub status: DepreciationRunStatus,
    /// Individual asset depreciation.
    pub asset_entries: Vec<DepreciationEntry>,
    /// Total depreciation amount.
    pub total_depreciation: Decimal,
    /// Asset count processed.
    pub asset_count: u32,
    /// GL references.
    pub gl_references: Vec<GLReference>,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Completed at.
    pub completed_at: Option<DateTime<Utc>>,
    /// Error count.
    pub error_count: u32,
    /// Errors.
    pub errors: Vec<DepreciationError>,
}

impl DepreciationRun {
    /// Creates a new depreciation run.
    pub fn new(
        run_id: String,
        company_code: String,
        fiscal_year: i32,
        fiscal_period: u32,
        depreciation_area: DepreciationAreaType,
        run_date: NaiveDate,
        created_by: String,
    ) -> Self {
        // Calculate posting date (end of fiscal period)
        let posting_date = Self::calculate_period_end(fiscal_year, fiscal_period);

        Self {
            run_id,
            company_code,
            fiscal_year,
            fiscal_period,
            depreciation_area,
            run_date,
            posting_date,
            status: DepreciationRunStatus::Created,
            asset_entries: Vec::new(),
            total_depreciation: Decimal::ZERO,
            asset_count: 0,
            gl_references: Vec::new(),
            created_by,
            created_at: Utc::now(),
            completed_at: None,
            error_count: 0,
            errors: Vec::new(),
        }
    }

    /// Calculates period end date.
    fn calculate_period_end(year: i32, period: u32) -> NaiveDate {
        let month = period;
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .expect("valid date components")
            .pred_opt()
            .expect("valid date components")
    }

    /// Adds a depreciation entry.
    pub fn add_entry(&mut self, entry: DepreciationEntry) {
        self.total_depreciation += entry.depreciation_amount;
        self.asset_count += 1;
        self.asset_entries.push(entry);
    }

    /// Adds an error.
    pub fn add_error(&mut self, error: DepreciationError) {
        self.error_count += 1;
        self.errors.push(error);
    }

    /// Starts the run.
    pub fn start(&mut self) {
        self.status = DepreciationRunStatus::Running;
    }

    /// Completes the run.
    pub fn complete(&mut self) {
        self.status = if self.error_count > 0 {
            DepreciationRunStatus::CompletedWithErrors
        } else {
            DepreciationRunStatus::Completed
        };
        self.completed_at = Some(Utc::now());
    }

    /// Posts the depreciation.
    pub fn post(&mut self) {
        self.status = DepreciationRunStatus::Posted;
    }

    /// Gets summary by asset class.
    pub fn summary_by_class(&self) -> HashMap<String, DepreciationSummary> {
        let mut summary: HashMap<String, DepreciationSummary> = HashMap::new();

        for entry in &self.asset_entries {
            let class_summary =
                summary
                    .entry(entry.asset_class.clone())
                    .or_insert_with(|| DepreciationSummary {
                        category: entry.asset_class.clone(),
                        asset_count: 0,
                        total_depreciation: Decimal::ZERO,
                        total_net_book_value: Decimal::ZERO,
                    });

            class_summary.asset_count += 1;
            class_summary.total_depreciation += entry.depreciation_amount;
            class_summary.total_net_book_value += entry.net_book_value_after;
        }

        summary
    }
}

/// Status of depreciation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepreciationRunStatus {
    /// Created, not started.
    Created,
    /// Running.
    Running,
    /// Completed successfully.
    Completed,
    /// Completed with errors.
    CompletedWithErrors,
    /// Posted to GL.
    Posted,
    /// Cancelled.
    Cancelled,
}

/// Individual asset depreciation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationEntry {
    /// Asset number.
    pub asset_number: String,
    /// Sub-number.
    pub sub_number: String,
    /// Asset description.
    pub asset_description: String,
    /// Asset class.
    pub asset_class: String,
    /// Depreciation method.
    pub depreciation_method: DepreciationMethod,
    /// Acquisition cost.
    pub acquisition_cost: Decimal,
    /// Accumulated depreciation before.
    pub accumulated_before: Decimal,
    /// Depreciation amount.
    pub depreciation_amount: Decimal,
    /// Accumulated depreciation after.
    pub accumulated_after: Decimal,
    /// Net book value after.
    pub net_book_value_after: Decimal,
    /// Is fully depreciated after this run.
    pub fully_depreciated: bool,
    /// Depreciation accounts.
    pub expense_account: String,
    /// Accumulated depreciation account.
    pub accum_depr_account: String,
    /// Cost center.
    pub cost_center: Option<String>,
}

impl DepreciationEntry {
    /// Creates from asset record.
    pub fn from_asset(asset: &FixedAssetRecord, area_type: DepreciationAreaType) -> Option<Self> {
        let area = asset
            .depreciation_areas
            .iter()
            .find(|a| a.area_type == area_type)?;

        let depreciation_amount = area.calculate_monthly_depreciation();
        let accumulated_after = area.accumulated_depreciation + depreciation_amount;
        let nbv_after = area.acquisition_cost - accumulated_after;
        let fully_depreciated = nbv_after <= area.salvage_value;

        Some(Self {
            asset_number: asset.asset_number.clone(),
            sub_number: asset.sub_number.clone(),
            asset_description: asset.description.clone(),
            asset_class: format!("{:?}", asset.asset_class),
            depreciation_method: area.method,
            acquisition_cost: area.acquisition_cost,
            accumulated_before: area.accumulated_depreciation,
            depreciation_amount,
            accumulated_after,
            net_book_value_after: nbv_after.max(Decimal::ZERO),
            fully_depreciated,
            expense_account: asset
                .account_determination
                .depreciation_expense_account
                .clone(),
            accum_depr_account: asset
                .account_determination
                .accumulated_depreciation_account
                .clone(),
            cost_center: asset.cost_center.clone(),
        })
    }
}

/// Error during depreciation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationError {
    /// Asset number.
    pub asset_number: String,
    /// Error code.
    pub error_code: DepreciationErrorCode,
    /// Error message.
    pub message: String,
}

/// Depreciation error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepreciationErrorCode {
    /// Asset not found.
    AssetNotFound,
    /// Asset already fully depreciated.
    FullyDepreciated,
    /// Asset not active.
    NotActive,
    /// Missing depreciation area.
    MissingDepreciationArea,
    /// Invalid depreciation method.
    InvalidMethod,
    /// Already depreciated for period.
    AlreadyDepreciated,
    /// Missing cost center.
    MissingCostCenter,
}

/// Summary of depreciation by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationSummary {
    /// Category (asset class).
    pub category: String,
    /// Asset count.
    pub asset_count: u32,
    /// Total depreciation.
    pub total_depreciation: Decimal,
    /// Total net book value.
    pub total_net_book_value: Decimal,
}

/// Depreciation forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationForecast {
    /// Company code.
    pub company_code: String,
    /// Forecast start date.
    pub start_date: NaiveDate,
    /// Forecast periods.
    pub periods: u32,
    /// Monthly forecasts.
    pub monthly_forecasts: Vec<MonthlyDepreciationForecast>,
    /// Total forecasted depreciation.
    pub total_forecast: Decimal,
    /// Generated at.
    pub generated_at: DateTime<Utc>,
}

impl DepreciationForecast {
    /// Creates a forecast from assets.
    pub fn from_assets(
        company_code: String,
        assets: &[FixedAssetRecord],
        start_date: NaiveDate,
        periods: u32,
        area_type: DepreciationAreaType,
    ) -> Self {
        let active_assets: Vec<_> = assets
            .iter()
            .filter(|a| {
                a.company_code == company_code
                    && a.status == super::AssetStatus::Active
                    && !a.is_fully_depreciated()
            })
            .collect();

        let mut monthly_forecasts = Vec::new();
        let mut total_forecast = Decimal::ZERO;
        let mut current_date = start_date;

        for period in 0..periods {
            let mut period_total = Decimal::ZERO;
            let mut asset_details = Vec::new();

            for asset in &active_assets {
                if let Some(area) = asset
                    .depreciation_areas
                    .iter()
                    .find(|a| a.area_type == area_type)
                {
                    // Simulate depreciation considering fully depreciated threshold
                    let projected_accum = area.accumulated_depreciation
                        + area.calculate_monthly_depreciation() * Decimal::from(period);
                    let remaining_nbv =
                        (area.acquisition_cost - projected_accum).max(Decimal::ZERO);

                    if remaining_nbv > area.salvage_value {
                        let monthly = area.calculate_monthly_depreciation();
                        period_total += monthly;
                        asset_details.push(AssetDepreciationForecast {
                            asset_number: asset.asset_number.clone(),
                            depreciation_amount: monthly,
                            projected_nbv: remaining_nbv - monthly,
                        });
                    }
                }
            }

            monthly_forecasts.push(MonthlyDepreciationForecast {
                period_date: current_date,
                total_depreciation: period_total,
                asset_count: asset_details.len() as u32,
                asset_details,
            });

            total_forecast += period_total;

            // Move to next month
            current_date = if current_date.month() == 12 {
                NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
                    .expect("valid date components")
            } else {
                NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1)
                    .expect("valid date components")
            };
        }

        Self {
            company_code,
            start_date,
            periods,
            monthly_forecasts,
            total_forecast,
            generated_at: Utc::now(),
        }
    }
}

/// Monthly depreciation forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyDepreciationForecast {
    /// Period date (first of month).
    pub period_date: NaiveDate,
    /// Total depreciation.
    pub total_depreciation: Decimal,
    /// Asset count.
    pub asset_count: u32,
    /// Asset details.
    pub asset_details: Vec<AssetDepreciationForecast>,
}

/// Asset-level depreciation forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDepreciationForecast {
    /// Asset number.
    pub asset_number: String,
    /// Depreciation amount.
    pub depreciation_amount: Decimal,
    /// Projected net book value.
    pub projected_nbv: Decimal,
}

/// Depreciation schedule (annual view).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationSchedule {
    /// Asset number.
    pub asset_number: String,
    /// Asset description.
    pub description: String,
    /// Acquisition cost.
    pub acquisition_cost: Decimal,
    /// Salvage value.
    pub salvage_value: Decimal,
    /// Depreciation method.
    pub method: DepreciationMethod,
    /// Useful life in months.
    pub useful_life_months: u32,
    /// Start date.
    pub start_date: NaiveDate,
    /// Annual schedule.
    pub annual_entries: Vec<AnnualDepreciationEntry>,
}

impl DepreciationSchedule {
    /// Creates a schedule for an asset.
    pub fn for_asset(asset: &FixedAssetRecord, area_type: DepreciationAreaType) -> Option<Self> {
        let area = asset
            .depreciation_areas
            .iter()
            .find(|a| a.area_type == area_type)?;

        let depreciable_base = area.acquisition_cost - area.salvage_value;
        let years = (area.useful_life_months as f64 / 12.0).ceil() as u32;

        let mut annual_entries = Vec::new();
        let mut cumulative = Decimal::ZERO;
        let monthly = area.calculate_monthly_depreciation();

        for year in 1..=years {
            let annual = (monthly * dec!(12)).min(depreciable_base - cumulative);
            cumulative += annual;
            let ending_nbv = area.acquisition_cost - cumulative;

            annual_entries.push(AnnualDepreciationEntry {
                year,
                beginning_nbv: area.acquisition_cost - (cumulative - annual),
                depreciation: annual,
                ending_nbv: ending_nbv.max(area.salvage_value),
            });

            if ending_nbv <= area.salvage_value {
                break;
            }
        }

        Some(Self {
            asset_number: asset.asset_number.clone(),
            description: asset.description.clone(),
            acquisition_cost: area.acquisition_cost,
            salvage_value: area.salvage_value,
            method: area.method,
            useful_life_months: area.useful_life_months,
            start_date: asset.first_depreciation_date,
            annual_entries,
        })
    }
}

/// Annual depreciation entry in schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualDepreciationEntry {
    /// Year number.
    pub year: u32,
    /// Beginning net book value.
    pub beginning_nbv: Decimal,
    /// Depreciation amount.
    pub depreciation: Decimal,
    /// Ending net book value.
    pub ending_nbv: Decimal,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::subledger::fa::{AssetClass, DepreciationArea};

    #[test]
    fn test_depreciation_run() {
        let mut run = DepreciationRun::new(
            "RUN001".to_string(),
            "1000".to_string(),
            2024,
            1,
            DepreciationAreaType::Book,
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            "USER1".to_string(),
        );

        let entry = DepreciationEntry {
            asset_number: "ASSET001".to_string(),
            sub_number: "0".to_string(),
            asset_description: "Test Asset".to_string(),
            asset_class: "MachineryEquipment".to_string(),
            depreciation_method: DepreciationMethod::StraightLine,
            acquisition_cost: dec!(100000),
            accumulated_before: Decimal::ZERO,
            depreciation_amount: dec!(1666.67),
            accumulated_after: dec!(1666.67),
            net_book_value_after: dec!(98333.33),
            fully_depreciated: false,
            expense_account: "7100".to_string(),
            accum_depr_account: "1539".to_string(),
            cost_center: Some("CC100".to_string()),
        };

        run.add_entry(entry);
        run.complete();

        assert_eq!(run.asset_count, 1);
        assert_eq!(run.total_depreciation, dec!(1666.67));
        assert_eq!(run.status, DepreciationRunStatus::Completed);
    }

    #[test]
    fn test_depreciation_forecast() {
        let mut asset = FixedAssetRecord::new(
            "ASSET001".to_string(),
            "1000".to_string(),
            AssetClass::MachineryEquipment,
            "Machine".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            dec!(60000),
            "USD".to_string(),
        );

        asset.add_depreciation_area(DepreciationArea::new(
            DepreciationAreaType::Book,
            DepreciationMethod::StraightLine,
            60,
            dec!(60000),
        ));

        let assets = vec![asset];
        let forecast = DepreciationForecast::from_assets(
            "1000".to_string(),
            &assets,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            12,
            DepreciationAreaType::Book,
        );

        assert_eq!(forecast.monthly_forecasts.len(), 12);
        assert!(forecast.total_forecast > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_period_end() {
        let end_jan = DepreciationRun::calculate_period_end(2024, 1);
        assert_eq!(end_jan, NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());

        let end_dec = DepreciationRun::calculate_period_end(2024, 12);
        assert_eq!(end_dec, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }
}
