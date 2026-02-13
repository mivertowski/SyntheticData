//! Temporal fidelity evaluation.
//!
//! Validates temporal patterns including autocorrelation at weekly and monthly
//! lags, period-end spikes, and weekday coefficient of variation.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single temporal record with a timestamp and associated value.
#[derive(Debug, Clone)]
pub struct TemporalRecord {
    /// Unix epoch timestamp in seconds.
    pub timestamp_epoch: i64,
    /// Observed value at this timestamp.
    pub value: f64,
}

/// Thresholds for temporal fidelity analysis.
#[derive(Debug, Clone)]
pub struct TemporalFidelityThresholds {
    /// Minimum temporal fidelity score.
    pub min_temporal_fidelity: f64,
}

impl Default for TemporalFidelityThresholds {
    fn default() -> Self {
        Self {
            min_temporal_fidelity: 0.70,
        }
    }
}

/// Results of temporal fidelity analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFidelityAnalysis {
    /// Overall temporal fidelity score (0.0-1.0).
    pub temporal_fidelity_score: f64,
    /// Maximum of weekly and monthly autocorrelation.
    pub seasonality_strength: f64,
    /// Autocorrelation at lag 7 (weekly pattern).
    pub weekly_autocorrelation: f64,
    /// Autocorrelation at lag 30 (monthly pattern).
    pub monthly_autocorrelation: f64,
    /// Ratio of mean(last 5 days of month) to mean(rest of month).
    pub period_end_spike_ratio: f64,
    /// Coefficient of variation of counts across weekday bins.
    pub weekday_cv: f64,
    /// Total number of records analyzed.
    pub total_records: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for temporal fidelity.
pub struct TemporalFidelityAnalyzer {
    thresholds: TemporalFidelityThresholds,
}

impl TemporalFidelityAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: TemporalFidelityThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: TemporalFidelityThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze temporal fidelity.
    pub fn analyze(&self, records: &[TemporalRecord]) -> EvalResult<TemporalFidelityAnalysis> {
        let mut issues = Vec::new();
        let total_records = records.len();

        if records.is_empty() {
            return Ok(TemporalFidelityAnalysis {
                temporal_fidelity_score: 0.0,
                seasonality_strength: 0.0,
                weekly_autocorrelation: 0.0,
                monthly_autocorrelation: 0.0,
                period_end_spike_ratio: 1.0,
                weekday_cv: 0.0,
                total_records: 0,
                passes: true,
                issues: vec!["No records provided".to_string()],
            });
        }

        // Sort by timestamp
        let mut sorted: Vec<&TemporalRecord> = records.iter().collect();
        sorted.sort_by_key(|r| r.timestamp_epoch);

        // Build daily aggregated series
        let daily_values = self.aggregate_daily(&sorted);

        // Compute autocorrelations
        let weekly_autocorrelation = self.autocorrelation(&daily_values, 7);
        let monthly_autocorrelation = self.autocorrelation(&daily_values, 30);
        let seasonality_strength = weekly_autocorrelation
            .abs()
            .max(monthly_autocorrelation.abs());

        // Compute period-end spike ratio
        let period_end_spike_ratio = self.compute_period_end_spike(&sorted);

        // Compute weekday CV
        let weekday_cv = self.compute_weekday_cv(&sorted);

        // Composite score
        // Reward: strong seasonality, clear period-end spikes, moderate weekday variation
        let seasonality_factor = seasonality_strength.clamp(0.0, 1.0);
        let spike_factor = if period_end_spike_ratio > 1.0 {
            (1.0 - 1.0 / period_end_spike_ratio).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let weekday_factor = weekday_cv.clamp(0.0, 1.0);

        let temporal_fidelity_score =
            (seasonality_factor * 0.4 + spike_factor * 0.3 + weekday_factor * 0.3).clamp(0.0, 1.0);

        if temporal_fidelity_score < self.thresholds.min_temporal_fidelity {
            issues.push(format!(
                "Temporal fidelity score {:.4} < {:.4} (threshold)",
                temporal_fidelity_score, self.thresholds.min_temporal_fidelity
            ));
        }

        let passes = issues.is_empty();

        Ok(TemporalFidelityAnalysis {
            temporal_fidelity_score,
            seasonality_strength,
            weekly_autocorrelation,
            monthly_autocorrelation,
            period_end_spike_ratio,
            weekday_cv,
            total_records,
            passes,
            issues,
        })
    }

    /// Aggregate records into daily value sums.
    fn aggregate_daily(&self, sorted_records: &[&TemporalRecord]) -> Vec<f64> {
        if sorted_records.is_empty() {
            return Vec::new();
        }

        let seconds_per_day = 86400i64;
        let mut daily: HashMap<i64, f64> = HashMap::new();

        for record in sorted_records {
            let day = record.timestamp_epoch / seconds_per_day;
            *daily.entry(day).or_insert(0.0) += record.value;
        }

        // Convert to ordered series
        let mut days: Vec<i64> = daily.keys().copied().collect();
        days.sort_unstable();

        if days.is_empty() {
            return Vec::new();
        }

        let first_day = days[0];
        let last_day = *days.last().unwrap_or(&first_day);
        let range = (last_day - first_day + 1) as usize;

        let mut series = vec![0.0; range];
        for (&day, &val) in &daily {
            let idx = (day - first_day) as usize;
            if idx < series.len() {
                series[idx] = val;
            }
        }

        series
    }

    /// Compute autocorrelation at the given lag.
    fn autocorrelation(&self, series: &[f64], lag: usize) -> f64 {
        if series.len() <= lag {
            return 0.0;
        }

        let n = series.len();
        let mean = series.iter().sum::<f64>() / n as f64;
        let variance: f64 = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;

        if variance < 1e-12 {
            return 0.0;
        }

        let mut cov = 0.0;
        for i in 0..(n - lag) {
            cov += (series[i] - mean) * (series[i + lag] - mean);
        }
        cov /= n as f64;

        cov / variance
    }

    /// Compute period-end spike: ratio of mean(last 5 days of month) to mean(rest).
    fn compute_period_end_spike(&self, sorted_records: &[&TemporalRecord]) -> f64 {
        let mut end_values = Vec::new();
        let mut rest_values = Vec::new();

        for record in sorted_records {
            let day_of_month = self.day_of_month(record.timestamp_epoch);
            let days_in_month = self.days_in_month(record.timestamp_epoch);

            if day_of_month > days_in_month.saturating_sub(5) {
                end_values.push(record.value);
            } else {
                rest_values.push(record.value);
            }
        }

        let mean_end = if end_values.is_empty() {
            0.0
        } else {
            end_values.iter().sum::<f64>() / end_values.len() as f64
        };

        let mean_rest = if rest_values.is_empty() {
            0.0
        } else {
            rest_values.iter().sum::<f64>() / rest_values.len() as f64
        };

        if mean_rest.abs() < 1e-12 {
            return 1.0;
        }

        mean_end / mean_rest
    }

    /// Compute coefficient of variation of record counts across weekday bins.
    fn compute_weekday_cv(&self, sorted_records: &[&TemporalRecord]) -> f64 {
        let mut weekday_counts = [0usize; 7];

        for record in sorted_records {
            let weekday = self.weekday(record.timestamp_epoch);
            weekday_counts[weekday] += 1;
        }

        let counts: Vec<f64> = weekday_counts.iter().map(|&c| c as f64).collect();
        let mean = counts.iter().sum::<f64>() / 7.0;

        if mean < 1e-12 {
            return 0.0;
        }

        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / 7.0;
        variance.sqrt() / mean
    }

    /// Get day-of-month (1-based) from epoch seconds.
    fn day_of_month(&self, epoch: i64) -> u32 {
        // Simple calculation: approximate using 86400 seconds per day
        // Using a simplified Gregorian calendar conversion
        let days_since_epoch = epoch / 86400;
        let (_, _, day) = days_to_ymd(days_since_epoch);
        day
    }

    /// Get approximate days in the month for the given epoch.
    fn days_in_month(&self, epoch: i64) -> u32 {
        let days_since_epoch = epoch / 86400;
        let (year, month, _) = days_to_ymd(days_since_epoch);
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Get weekday (0=Monday, 6=Sunday) from epoch seconds.
    fn weekday(&self, epoch: i64) -> usize {
        // January 1, 1970 was a Thursday (index 3 for Mon=0)
        let days = epoch / 86400;
        ((days % 7 + 3) % 7) as usize
    }
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
    // Shift to March-based year to simplify leap-day handling
    days += 719468; // days from 0000-03-01 to 1970-01-01
    let era = if days >= 0 {
        days / 146097
    } else {
        (days - 146096) / 146097
    };
    let doe = (days - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

impl Default for TemporalFidelityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_daily_records(values: &[f64], start_epoch: i64) -> Vec<TemporalRecord> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| TemporalRecord {
                timestamp_epoch: start_epoch + (i as i64) * 86400,
                value: v,
            })
            .collect()
    }

    #[test]
    fn test_valid_temporal_patterns() {
        // Create a weekly pattern: higher on weekdays, lower on weekends
        let mut values = Vec::new();
        for week in 0..12 {
            for day in 0..7 {
                let base = 100.0;
                let val = if day < 5 {
                    base + (week as f64) * 2.0
                } else {
                    base * 0.3
                };
                values.push(val);
            }
        }

        // Start on 2024-01-01 (Monday) epoch = 1704067200
        let records = make_daily_records(&values, 1_704_067_200);

        let analyzer = TemporalFidelityAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.total_records, 84);
        assert!(result.weekly_autocorrelation > 0.0);
    }

    #[test]
    fn test_invalid_temporal_flat() {
        // Completely flat series: no temporal patterns
        let values = vec![100.0; 90];
        let records = make_daily_records(&values, 1_704_067_200);

        let analyzer = TemporalFidelityAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        // Flat series should have low fidelity
        assert!(result.temporal_fidelity_score < 0.7);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_records() {
        let analyzer = TemporalFidelityAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_records, 0);
        assert_eq!(result.temporal_fidelity_score, 0.0);
    }
}
