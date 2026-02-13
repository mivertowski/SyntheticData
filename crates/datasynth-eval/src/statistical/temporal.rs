//! Temporal pattern analysis.
//!
//! Analyzes the temporal distribution of transactions including
//! seasonality patterns, day-of-week effects, and periodic spikes.

use crate::error::{EvalError, EvalResult};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expected seasonality spike multipliers.
pub const MONTH_END_SPIKE: f64 = 2.5;
pub const QUARTER_END_SPIKE: f64 = 4.0;
pub const YEAR_END_SPIKE: f64 = 6.0;
pub const WEEKEND_RATIO: f64 = 0.10;

/// Results of temporal pattern analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    /// Number of entries analyzed.
    pub sample_size: usize,
    /// Start date of data.
    pub start_date: NaiveDate,
    /// End date of data.
    pub end_date: NaiveDate,
    /// Number of days spanned.
    pub days_spanned: i64,
    /// Correlation with expected temporal pattern.
    pub pattern_correlation: f64,
    /// Actual month-end spike ratio (vs average).
    pub month_end_spike: f64,
    /// Actual quarter-end spike ratio.
    pub quarter_end_spike: f64,
    /// Actual year-end spike ratio.
    pub year_end_spike: f64,
    /// Weekend activity ratio.
    pub weekend_ratio: f64,
    /// Day-of-week distribution.
    pub day_of_week_distribution: HashMap<String, f64>,
    /// Day-of-week correlation with expected pattern.
    pub day_of_week_correlation: f64,
    /// Monthly volume distribution.
    pub monthly_distribution: HashMap<u32, usize>,
    /// Whether patterns match expectations.
    pub passes: bool,
}

/// Input for temporal analysis.
#[derive(Debug, Clone)]
pub struct TemporalEntry {
    /// Posting date of the entry.
    pub posting_date: NaiveDate,
}

/// Expected day-of-week weights.
const DAY_WEIGHTS: [f64; 7] = [
    1.3,  // Monday
    1.1,  // Tuesday
    1.0,  // Wednesday
    1.0,  // Thursday
    0.85, // Friday
    0.05, // Saturday
    0.05, // Sunday
];

/// Analyzer for temporal patterns.
pub struct TemporalAnalyzer {
    /// Whether to analyze industry seasonality.
    analyze_industry_seasonality: bool,
}

impl TemporalAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            analyze_industry_seasonality: false,
        }
    }

    /// Enable industry seasonality analysis.
    pub fn with_industry_seasonality(mut self) -> Self {
        self.analyze_industry_seasonality = true;
        self
    }

    /// Analyze temporal patterns from entries.
    pub fn analyze(&self, entries: &[TemporalEntry]) -> EvalResult<TemporalAnalysis> {
        let n = entries.len();
        if n < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: n,
            });
        }

        // Get date range
        let dates: Vec<NaiveDate> = entries.iter().map(|e| e.posting_date).collect();
        let start_date = *dates.iter().min().expect("non-empty after length check");
        let end_date = *dates.iter().max().expect("non-empty after length check");
        let days_spanned = (end_date - start_date).num_days() + 1;

        // Count by date
        let mut daily_counts: HashMap<NaiveDate, usize> = HashMap::new();
        for entry in entries {
            *daily_counts.entry(entry.posting_date).or_insert(0) += 1;
        }

        // Calculate average daily volume
        let avg_daily = n as f64 / days_spanned as f64;

        // Month-end spike analysis
        let month_end_spike = self.calculate_month_end_spike(&daily_counts, avg_daily);

        // Quarter-end spike analysis
        let quarter_end_spike = self.calculate_quarter_end_spike(&daily_counts, avg_daily);

        // Year-end spike analysis
        let year_end_spike = self.calculate_year_end_spike(&daily_counts, avg_daily);

        // Weekend ratio
        let weekend_count = entries
            .iter()
            .filter(|e| {
                let weekday = e.posting_date.weekday();
                weekday == Weekday::Sat || weekday == Weekday::Sun
            })
            .count();
        let weekend_ratio = weekend_count as f64 / n as f64;

        // Day-of-week distribution
        let mut dow_counts = [0usize; 7];
        for entry in entries {
            let idx = entry.posting_date.weekday().num_days_from_monday() as usize;
            dow_counts[idx] += 1;
        }
        let total_dow: usize = dow_counts.iter().sum();
        let mut day_of_week_distribution = HashMap::new();
        let weekdays = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        for (i, name) in weekdays.iter().enumerate() {
            day_of_week_distribution
                .insert(name.to_string(), dow_counts[i] as f64 / total_dow as f64);
        }

        // Day-of-week correlation
        let day_of_week_correlation = self.calculate_dow_correlation(&dow_counts);

        // Monthly distribution
        let mut monthly_distribution: HashMap<u32, usize> = HashMap::new();
        for entry in entries {
            *monthly_distribution
                .entry(entry.posting_date.month())
                .or_insert(0) += 1;
        }

        // Overall pattern correlation
        let pattern_correlation =
            self.calculate_pattern_correlation(&daily_counts, start_date, end_date, avg_daily);

        // Pass/fail check
        let passes = pattern_correlation >= 0.5 && (weekend_ratio - WEEKEND_RATIO).abs() < 0.15;

        Ok(TemporalAnalysis {
            sample_size: n,
            start_date,
            end_date,
            days_spanned,
            pattern_correlation,
            month_end_spike,
            quarter_end_spike,
            year_end_spike,
            weekend_ratio,
            day_of_week_distribution,
            day_of_week_correlation,
            monthly_distribution,
            passes,
        })
    }

    /// Calculate month-end spike ratio.
    fn calculate_month_end_spike(
        &self,
        daily_counts: &HashMap<NaiveDate, usize>,
        avg_daily: f64,
    ) -> f64 {
        if avg_daily <= 0.0 {
            return 1.0;
        }

        let month_end_dates: Vec<&NaiveDate> = daily_counts
            .keys()
            .filter(|d| self.is_month_end(**d))
            .collect();

        if month_end_dates.is_empty() {
            return 1.0;
        }

        let month_end_total: usize = month_end_dates
            .iter()
            .filter_map(|d| daily_counts.get(*d))
            .sum();
        let month_end_avg = month_end_total as f64 / month_end_dates.len() as f64;

        month_end_avg / avg_daily
    }

    /// Calculate quarter-end spike ratio.
    fn calculate_quarter_end_spike(
        &self,
        daily_counts: &HashMap<NaiveDate, usize>,
        avg_daily: f64,
    ) -> f64 {
        if avg_daily <= 0.0 {
            return 1.0;
        }

        let quarter_end_dates: Vec<&NaiveDate> = daily_counts
            .keys()
            .filter(|d| self.is_quarter_end(**d))
            .collect();

        if quarter_end_dates.is_empty() {
            return 1.0;
        }

        let quarter_end_total: usize = quarter_end_dates
            .iter()
            .filter_map(|d| daily_counts.get(*d))
            .sum();
        let quarter_end_avg = quarter_end_total as f64 / quarter_end_dates.len() as f64;

        quarter_end_avg / avg_daily
    }

    /// Calculate year-end spike ratio.
    fn calculate_year_end_spike(
        &self,
        daily_counts: &HashMap<NaiveDate, usize>,
        avg_daily: f64,
    ) -> f64 {
        if avg_daily <= 0.0 {
            return 1.0;
        }

        let year_end_dates: Vec<&NaiveDate> = daily_counts
            .keys()
            .filter(|d| self.is_year_end(**d))
            .collect();

        if year_end_dates.is_empty() {
            return 1.0;
        }

        let year_end_total: usize = year_end_dates
            .iter()
            .filter_map(|d| daily_counts.get(*d))
            .sum();
        let year_end_avg = year_end_total as f64 / year_end_dates.len() as f64;

        year_end_avg / avg_daily
    }

    /// Check if date is in month-end period (last 5 days).
    fn is_month_end(&self, date: NaiveDate) -> bool {
        let next_month = if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
        };
        if let Some(next) = next_month {
            let days_to_end = (next - date).num_days();
            days_to_end <= 5
        } else {
            false
        }
    }

    /// Check if date is in quarter-end period.
    fn is_quarter_end(&self, date: NaiveDate) -> bool {
        let quarter_end_months = [3, 6, 9, 12];
        quarter_end_months.contains(&date.month()) && self.is_month_end(date)
    }

    /// Check if date is in year-end period.
    fn is_year_end(&self, date: NaiveDate) -> bool {
        date.month() == 12 && self.is_month_end(date)
    }

    /// Calculate day-of-week correlation with expected pattern.
    fn calculate_dow_correlation(&self, observed: &[usize; 7]) -> f64 {
        let total: usize = observed.iter().sum();
        if total == 0 {
            return 0.0;
        }

        // Normalize observed to proportions
        let observed_norm: Vec<f64> = observed.iter().map(|&c| c as f64 / total as f64).collect();

        // Normalize expected weights
        let total_weight: f64 = DAY_WEIGHTS.iter().sum();
        let expected_norm: Vec<f64> = DAY_WEIGHTS.iter().map(|&w| w / total_weight).collect();

        // Pearson correlation
        let mean_obs = observed_norm.iter().sum::<f64>() / 7.0;
        let mean_exp = expected_norm.iter().sum::<f64>() / 7.0;

        let numerator: f64 = (0..7)
            .map(|i| (observed_norm[i] - mean_obs) * (expected_norm[i] - mean_exp))
            .sum();

        let var_obs: f64 = observed_norm.iter().map(|o| (o - mean_obs).powi(2)).sum();
        let var_exp: f64 = expected_norm.iter().map(|e| (e - mean_exp).powi(2)).sum();

        let denominator = (var_obs * var_exp).sqrt();

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }

    /// Calculate overall pattern correlation.
    fn calculate_pattern_correlation(
        &self,
        daily_counts: &HashMap<NaiveDate, usize>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        avg_daily: f64,
    ) -> f64 {
        // Generate expected pattern for each day
        let mut expected: Vec<f64> = Vec::new();
        let mut observed: Vec<f64> = Vec::new();

        let mut current = start_date;
        while current <= end_date {
            let mut multiplier = 1.0;

            // Weekend effect
            let weekday = current.weekday();
            if weekday == Weekday::Sat || weekday == Weekday::Sun {
                multiplier *= 0.1;
            } else {
                // Day-of-week effect
                let dow_idx = weekday.num_days_from_monday() as usize;
                multiplier *= DAY_WEIGHTS[dow_idx] / 1.0;
            }

            // Month-end effect
            if self.is_month_end(current) {
                multiplier *= MONTH_END_SPIKE;
            }

            // Year-end effect (stronger)
            if self.is_year_end(current) {
                multiplier *= YEAR_END_SPIKE / MONTH_END_SPIKE;
            } else if self.is_quarter_end(current) {
                multiplier *= QUARTER_END_SPIKE / MONTH_END_SPIKE;
            }

            expected.push(avg_daily * multiplier);
            observed.push(*daily_counts.get(&current).unwrap_or(&0) as f64);

            current = current.succ_opt().unwrap_or(current);
        }

        // Calculate Pearson correlation
        if expected.is_empty() {
            return 0.0;
        }

        let n = expected.len() as f64;
        let mean_exp = expected.iter().sum::<f64>() / n;
        let mean_obs = observed.iter().sum::<f64>() / n;

        let numerator: f64 = expected
            .iter()
            .zip(observed.iter())
            .map(|(e, o)| (e - mean_exp) * (o - mean_obs))
            .sum();

        let var_exp: f64 = expected.iter().map(|e| (e - mean_exp).powi(2)).sum();
        let var_obs: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();

        let denominator = (var_exp * var_obs).sqrt();

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
}

impl Default for TemporalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_entries(dates: Vec<NaiveDate>) -> Vec<TemporalEntry> {
        dates
            .into_iter()
            .map(|d| TemporalEntry { posting_date: d })
            .collect()
    }

    #[test]
    fn test_temporal_analysis_basic() {
        let entries = create_entries(vec![
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 22).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 23).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 24).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 26).unwrap(),
        ]);

        let analyzer = TemporalAnalyzer::new();
        let result = analyzer.analyze(&entries).unwrap();

        assert_eq!(result.sample_size, 10);
        assert!(!result.day_of_week_distribution.is_empty());
    }

    #[test]
    fn test_weekend_ratio() {
        let mut entries = Vec::new();
        // 10 weekday entries
        for i in 1..=10 {
            entries.push(TemporalEntry {
                posting_date: NaiveDate::from_ymd_opt(2024, 1, i).unwrap(),
            });
        }
        // 2 weekend entries (6th and 7th are Sat and Sun)
        // Note: Jan 6, 2024 is Saturday, Jan 7 is Sunday

        let analyzer = TemporalAnalyzer::new();
        let result = analyzer.analyze(&entries).unwrap();

        // Check weekend ratio is calculated
        assert!(result.weekend_ratio >= 0.0);
        assert!(result.weekend_ratio <= 1.0);
    }

    #[test]
    fn test_insufficient_data() {
        let entries = create_entries(vec![NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()]);
        let analyzer = TemporalAnalyzer::new();
        let result = analyzer.analyze(&entries);
        assert!(matches!(result, Err(EvalError::InsufficientData { .. })));
    }
}
