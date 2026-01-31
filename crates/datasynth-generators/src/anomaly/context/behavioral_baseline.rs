//! Behavioral baseline tracking for anomaly detection.
//!
//! Tracks normal behavioral patterns for entities and detects
//! deviations that may indicate anomalies.

use chrono::{NaiveDate, NaiveTime, Timelike};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use datasynth_core::models::{AnomalyType, SeverityLevel, StatisticalAnomalyType};

/// Configuration for behavioral baseline tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralBaselineConfig {
    /// Enable behavioral baseline tracking.
    pub enabled: bool,
    /// Number of days to build baseline.
    pub baseline_period_days: u32,
    /// Minimum observations to establish baseline.
    pub min_observations: u32,
    /// Standard deviation threshold for amount anomalies.
    pub amount_deviation_threshold: f64,
    /// Standard deviation threshold for frequency anomalies.
    pub frequency_deviation_threshold: f64,
    /// Decay factor for recency weighting (0.0-1.0).
    pub recency_decay_factor: f64,
}

impl Default for BehavioralBaselineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            baseline_period_days: 90,
            min_observations: 10,
            amount_deviation_threshold: 3.0,
            frequency_deviation_threshold: 2.0,
            recency_decay_factor: 0.95,
        }
    }
}

/// Behavioral baseline manager.
pub struct BehavioralBaseline {
    config: BehavioralBaselineConfig,
    /// Baselines by entity ID.
    entity_baselines: HashMap<String, EntityBaseline>,
}

impl Default for BehavioralBaseline {
    fn default() -> Self {
        Self::new(BehavioralBaselineConfig::default())
    }
}

impl BehavioralBaseline {
    /// Creates a new behavioral baseline manager.
    pub fn new(config: BehavioralBaselineConfig) -> Self {
        Self {
            config,
            entity_baselines: HashMap::new(),
        }
    }

    /// Records an observation for an entity.
    pub fn record_observation(&mut self, entity_id: impl Into<String>, observation: Observation) {
        let id = entity_id.into();
        let baseline = self
            .entity_baselines
            .entry(id)
            .or_insert_with(EntityBaseline::new);
        baseline.add_observation(observation);
    }

    /// Gets the baseline for an entity.
    pub fn get_baseline(&self, entity_id: &str) -> Option<&EntityBaseline> {
        self.entity_baselines.get(entity_id)
    }

    /// Checks for behavioral deviations.
    pub fn check_deviation(
        &self,
        entity_id: &str,
        observation: &Observation,
    ) -> Vec<BehavioralDeviation> {
        if !self.config.enabled {
            return Vec::new();
        }

        let baseline = match self.get_baseline(entity_id) {
            Some(b) if b.observation_count >= self.config.min_observations => b,
            _ => return Vec::new(),
        };

        let mut deviations = Vec::new();

        // Check amount deviation
        if let Some(amount) = observation.amount {
            let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
            if baseline.amount_std_dev > 0.0 {
                let z_score =
                    (amount_f64 - baseline.avg_transaction_amount) / baseline.amount_std_dev;
                if z_score.abs() > self.config.amount_deviation_threshold {
                    deviations.push(BehavioralDeviation {
                        deviation_type: DeviationType::AmountAnomaly,
                        std_deviations: z_score.abs(),
                        expected_value: baseline.avg_transaction_amount,
                        actual_value: amount_f64,
                        label: AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
                        severity: Self::severity_from_std_dev(z_score.abs()),
                        description: format!(
                            "Amount ${:.2} is {:.1} std devs from mean ${:.2}",
                            amount_f64, z_score.abs(), baseline.avg_transaction_amount
                        ),
                    });
                }
            }
        }

        // Check timing deviation
        if let Some(time) = observation.time {
            if !baseline.is_within_typical_hours(time) {
                deviations.push(BehavioralDeviation {
                    deviation_type: DeviationType::TimingAnomaly,
                    std_deviations: 0.0,
                    expected_value: 0.0,
                    actual_value: 0.0,
                    label: AnomalyType::Statistical(StatisticalAnomalyType::UnusualTiming),
                    severity: SeverityLevel::Low,
                    description: format!(
                        "Transaction at {} outside typical hours {:02}:00-{:02}:00",
                        time, baseline.typical_posting_hours.0, baseline.typical_posting_hours.1
                    ),
                });
            }
        }

        // Check new counterparty
        if let Some(ref counterparty) = observation.counterparty {
            if !baseline.common_counterparties.contains(counterparty)
                && baseline.common_counterparties.len() >= 5
            {
                deviations.push(BehavioralDeviation {
                    deviation_type: DeviationType::NewCounterparty,
                    std_deviations: 0.0,
                    expected_value: 0.0,
                    actual_value: 0.0,
                    label: AnomalyType::Statistical(StatisticalAnomalyType::StatisticalOutlier),
                    severity: SeverityLevel::Low,
                    description: format!(
                        "New counterparty '{}' not in typical partners",
                        counterparty
                    ),
                });
            }
        }

        // Check unusual account
        if let Some(ref account) = observation.account_code {
            if !baseline.usual_account_codes.contains(account)
                && baseline.usual_account_codes.len() >= 3
            {
                deviations.push(BehavioralDeviation {
                    deviation_type: DeviationType::UnusualAccount,
                    std_deviations: 0.0,
                    expected_value: 0.0,
                    actual_value: 0.0,
                    label: AnomalyType::Statistical(StatisticalAnomalyType::StatisticalOutlier),
                    severity: SeverityLevel::Low,
                    description: format!(
                        "Account '{}' not typically used by this entity",
                        account
                    ),
                });
            }
        }

        deviations
    }

    /// Determines severity based on standard deviations.
    fn severity_from_std_dev(std_devs: f64) -> SeverityLevel {
        if std_devs > 5.0 {
            SeverityLevel::Critical
        } else if std_devs > 4.0 {
            SeverityLevel::High
        } else if std_devs > 3.5 {
            SeverityLevel::Medium
        } else {
            SeverityLevel::Low
        }
    }

    /// Checks if frequency has deviated from baseline.
    pub fn check_frequency_deviation(
        &self,
        entity_id: &str,
        current_frequency: f64,
    ) -> Option<BehavioralDeviation> {
        if !self.config.enabled {
            return None;
        }

        let baseline = self.get_baseline(entity_id)?;

        if baseline.observation_count < self.config.min_observations {
            return None;
        }

        if baseline.frequency_std_dev <= 0.0 {
            return None;
        }

        let z_score =
            (current_frequency - baseline.transaction_frequency) / baseline.frequency_std_dev;

        if z_score.abs() > self.config.frequency_deviation_threshold {
            Some(BehavioralDeviation {
                deviation_type: DeviationType::FrequencyAnomaly,
                std_deviations: z_score.abs(),
                expected_value: baseline.transaction_frequency,
                actual_value: current_frequency,
                label: AnomalyType::Statistical(StatisticalAnomalyType::UnusualFrequency),
                severity: Self::severity_from_std_dev(z_score.abs()),
                description: format!(
                    "Frequency {:.2}/day is {:.1} std devs from normal {:.2}/day",
                    current_frequency,
                    z_score.abs(),
                    baseline.transaction_frequency
                ),
            })
        } else {
            None
        }
    }

    /// Returns the number of tracked entities.
    pub fn entity_count(&self) -> usize {
        self.entity_baselines.len()
    }

    /// Returns the configuration.
    pub fn config(&self) -> &BehavioralBaselineConfig {
        &self.config
    }

    /// Clears all baselines.
    pub fn clear(&mut self) {
        self.entity_baselines.clear();
    }
}

/// An observation for behavioral tracking.
#[derive(Debug, Clone)]
pub struct Observation {
    /// Date of observation.
    pub date: NaiveDate,
    /// Time of observation.
    pub time: Option<NaiveTime>,
    /// Transaction amount.
    pub amount: Option<Decimal>,
    /// Counterparty ID.
    pub counterparty: Option<String>,
    /// Account code used.
    pub account_code: Option<String>,
}

impl Observation {
    /// Creates a new observation.
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            time: None,
            amount: None,
            counterparty: None,
            account_code: None,
        }
    }

    /// Sets the time.
    pub fn with_time(mut self, time: NaiveTime) -> Self {
        self.time = Some(time);
        self
    }

    /// Sets the amount.
    pub fn with_amount(mut self, amount: Decimal) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Sets the counterparty.
    pub fn with_counterparty(mut self, counterparty: impl Into<String>) -> Self {
        self.counterparty = Some(counterparty.into());
        self
    }

    /// Sets the account code.
    pub fn with_account(mut self, account: impl Into<String>) -> Self {
        self.account_code = Some(account.into());
        self
    }
}

/// Behavioral baseline for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBaseline {
    /// Average transaction amount.
    pub avg_transaction_amount: f64,
    /// Standard deviation of amounts.
    pub amount_std_dev: f64,
    /// Average transaction frequency (per day).
    pub transaction_frequency: f64,
    /// Standard deviation of frequency.
    pub frequency_std_dev: f64,
    /// Typical posting hours (start, end).
    pub typical_posting_hours: (u8, u8),
    /// Common counterparties (most frequent).
    pub common_counterparties: Vec<String>,
    /// Usual account codes.
    pub usual_account_codes: Vec<String>,
    /// Number of observations.
    pub observation_count: u32,
    /// Running sum for incremental mean.
    #[serde(skip)]
    amount_sum: f64,
    /// Running sum of squares for incremental variance.
    #[serde(skip)]
    amount_sum_sq: f64,
    /// Daily counts for frequency calculation.
    #[serde(skip)]
    daily_counts: HashMap<NaiveDate, u32>,
    /// Hour distribution for typical hours.
    #[serde(skip)]
    hour_counts: [u32; 24],
    /// Counterparty frequency map.
    #[serde(skip)]
    counterparty_freq: HashMap<String, u32>,
    /// Account frequency map.
    #[serde(skip)]
    account_freq: HashMap<String, u32>,
}

impl Default for EntityBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityBaseline {
    /// Creates a new empty baseline.
    pub fn new() -> Self {
        Self {
            avg_transaction_amount: 0.0,
            amount_std_dev: 0.0,
            transaction_frequency: 0.0,
            frequency_std_dev: 0.0,
            typical_posting_hours: (8, 18),
            common_counterparties: Vec::new(),
            usual_account_codes: Vec::new(),
            observation_count: 0,
            amount_sum: 0.0,
            amount_sum_sq: 0.0,
            daily_counts: HashMap::new(),
            hour_counts: [0; 24],
            counterparty_freq: HashMap::new(),
            account_freq: HashMap::new(),
        }
    }

    /// Adds an observation to the baseline.
    pub fn add_observation(&mut self, observation: Observation) {
        self.observation_count += 1;

        // Update amount statistics
        if let Some(amount) = observation.amount {
            let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
            self.amount_sum += amount_f64;
            self.amount_sum_sq += amount_f64 * amount_f64;
            self.avg_transaction_amount = self.amount_sum / self.observation_count as f64;

            if self.observation_count > 1 {
                let variance = (self.amount_sum_sq
                    - (self.amount_sum * self.amount_sum) / self.observation_count as f64)
                    / (self.observation_count - 1) as f64;
                self.amount_std_dev = variance.max(0.0).sqrt();
            }
        }

        // Update daily counts for frequency
        *self.daily_counts.entry(observation.date).or_insert(0) += 1;
        self.update_frequency_stats();

        // Update hour distribution
        if let Some(time) = observation.time {
            self.hour_counts[time.hour() as usize] += 1;
            self.update_typical_hours();
        }

        // Update counterparty frequency
        if let Some(ref counterparty) = observation.counterparty {
            *self
                .counterparty_freq
                .entry(counterparty.clone())
                .or_insert(0) += 1;
            self.update_common_counterparties();
        }

        // Update account frequency
        if let Some(ref account) = observation.account_code {
            *self.account_freq.entry(account.clone()).or_insert(0) += 1;
            self.update_usual_accounts();
        }
    }

    /// Updates frequency statistics.
    fn update_frequency_stats(&mut self) {
        if self.daily_counts.is_empty() {
            return;
        }

        let counts: Vec<f64> = self.daily_counts.values().map(|&c| c as f64).collect();
        let n = counts.len() as f64;

        self.transaction_frequency = counts.iter().sum::<f64>() / n;

        if counts.len() > 1 {
            let variance: f64 = counts
                .iter()
                .map(|c| (c - self.transaction_frequency).powi(2))
                .sum::<f64>()
                / (n - 1.0);
            self.frequency_std_dev = variance.sqrt();
        }
    }

    /// Updates typical posting hours (80th percentile range).
    fn update_typical_hours(&mut self) {
        let total: u32 = self.hour_counts.iter().sum();
        if total == 0 {
            return;
        }

        // Find the hour range containing 80% of transactions
        let threshold = (total as f64 * 0.1) as u32; // 10% from each end

        let mut cumsum = 0u32;
        let mut start_hour = 0u8;
        for (hour, &count) in self.hour_counts.iter().enumerate() {
            cumsum += count;
            if cumsum > threshold {
                start_hour = hour as u8;
                break;
            }
        }

        cumsum = 0;
        let mut end_hour = 23u8;
        for (hour, &count) in self.hour_counts.iter().enumerate().rev() {
            cumsum += count;
            if cumsum > threshold {
                end_hour = hour as u8;
                break;
            }
        }

        self.typical_posting_hours = (start_hour, end_hour.max(start_hour + 1));
    }

    /// Updates common counterparties (top 10).
    fn update_common_counterparties(&mut self) {
        let mut sorted: Vec<_> = self.counterparty_freq.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        self.common_counterparties = sorted.into_iter().take(10).map(|(k, _)| k.clone()).collect();
    }

    /// Updates usual accounts (top 5).
    fn update_usual_accounts(&mut self) {
        let mut sorted: Vec<_> = self.account_freq.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        self.usual_account_codes = sorted.into_iter().take(5).map(|(k, _)| k.clone()).collect();
    }

    /// Checks if a time is within typical posting hours.
    pub fn is_within_typical_hours(&self, time: NaiveTime) -> bool {
        let hour = time.hour() as u8;
        hour >= self.typical_posting_hours.0 && hour <= self.typical_posting_hours.1
    }

    /// Returns whether the baseline has enough observations.
    pub fn is_established(&self, min_observations: u32) -> bool {
        self.observation_count >= min_observations
    }
}

/// Type of behavioral deviation detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviationType {
    /// Amount significantly different from baseline.
    AmountAnomaly,
    /// Transaction frequency significantly different.
    FrequencyAnomaly,
    /// New counterparty not seen before.
    NewCounterparty,
    /// Unusual posting time.
    TimingAnomaly,
    /// Account not typically used.
    UnusualAccount,
}

/// A detected behavioral deviation.
#[derive(Debug, Clone)]
pub struct BehavioralDeviation {
    /// Type of deviation.
    pub deviation_type: DeviationType,
    /// Number of standard deviations from baseline.
    pub std_deviations: f64,
    /// Expected value from baseline.
    pub expected_value: f64,
    /// Actual observed value.
    pub actual_value: f64,
    /// Suggested anomaly label.
    pub label: AnomalyType,
    /// Severity level.
    pub severity: SeverityLevel,
    /// Description of the deviation.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_baseline_creation() {
        let baseline = EntityBaseline::new();
        assert_eq!(baseline.observation_count, 0);
        assert!((baseline.avg_transaction_amount - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_observation_builder() {
        let obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
            .with_amount(dec!(1000))
            .with_counterparty("VENDOR001")
            .with_account("5000");

        assert_eq!(obs.amount, Some(dec!(1000)));
        assert_eq!(obs.counterparty, Some("VENDOR001".to_string()));
        assert_eq!(obs.account_code, Some("5000".to_string()));
    }

    #[test]
    fn test_baseline_amount_tracking() {
        let mut baseline = EntityBaseline::new();

        for amount in [1000.0, 1100.0, 900.0, 1050.0, 950.0] {
            let obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
                .with_amount(Decimal::try_from(amount).unwrap());
            baseline.add_observation(obs);
        }

        assert_eq!(baseline.observation_count, 5);
        assert!((baseline.avg_transaction_amount - 1000.0).abs() < 1.0);
        assert!(baseline.amount_std_dev > 0.0);
    }

    #[test]
    fn test_behavioral_baseline_deviation_detection() {
        let mut baseline_mgr = BehavioralBaseline::default();

        // Build baseline with consistent amounts (with some variance)
        // Using amounts from 900-1100 to establish a baseline with std dev
        let amounts = [
            900, 950, 1000, 1050, 1100, 920, 980, 1020, 1080, 950,
            960, 1000, 1040, 990, 970, 1010, 1030, 1000, 980, 1020,
        ];
        for (i, &amount) in amounts.iter().enumerate() {
            let obs = Observation::new(
                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()
                    + chrono::Duration::days(i as i64 % 10),
            )
            .with_amount(Decimal::from(amount))
            .with_counterparty("VENDOR001")
            .with_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
            baseline_mgr.record_observation("ENTITY1", obs);
        }

        // Check for deviation with a very different amount (50x normal)
        let unusual_obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 25).unwrap())
            .with_amount(dec!(50000))
            .with_counterparty("VENDOR001");

        let deviations = baseline_mgr.check_deviation("ENTITY1", &unusual_obs);

        // Should detect amount anomaly
        assert!(deviations
            .iter()
            .any(|d| d.deviation_type == DeviationType::AmountAnomaly));
    }

    #[test]
    fn test_new_counterparty_detection() {
        let mut baseline_mgr = BehavioralBaseline::default();

        // Build baseline with consistent counterparties
        for i in 0..15 {
            let cp = format!("VENDOR{:03}", i % 5);
            let obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap())
                .with_amount(dec!(1000))
                .with_counterparty(&cp);
            baseline_mgr.record_observation("ENTITY1", obs);
        }

        // Check for deviation with new counterparty
        let new_cp_obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 25).unwrap())
            .with_amount(dec!(1000))
            .with_counterparty("NEW_VENDOR");

        let deviations = baseline_mgr.check_deviation("ENTITY1", &new_cp_obs);

        // Should detect new counterparty
        assert!(deviations
            .iter()
            .any(|d| d.deviation_type == DeviationType::NewCounterparty));
    }

    #[test]
    fn test_timing_anomaly_detection() {
        let mut baseline_mgr = BehavioralBaseline::default();

        // Build baseline with consistent timing (9 AM - 5 PM)
        for i in 0..15 {
            let hour = 9 + (i % 8);
            let obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap())
                .with_amount(dec!(1000))
                .with_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap());
            baseline_mgr.record_observation("ENTITY1", obs);
        }

        // Check for deviation with unusual time (3 AM)
        let unusual_time_obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 25).unwrap())
            .with_amount(dec!(1000))
            .with_time(NaiveTime::from_hms_opt(3, 0, 0).unwrap());

        let deviations = baseline_mgr.check_deviation("ENTITY1", &unusual_time_obs);

        // Should detect timing anomaly
        assert!(deviations
            .iter()
            .any(|d| d.deviation_type == DeviationType::TimingAnomaly));
    }

    #[test]
    fn test_frequency_deviation() {
        let mut baseline_mgr = BehavioralBaseline::default();

        // Build baseline with variable transactions per day (1-3) for variance
        let daily_counts = [2, 1, 3, 2, 2, 1, 3, 2, 1, 2, 3, 2, 1, 2, 2, 3, 1, 2, 2, 3,
                           2, 1, 2, 3, 2, 1, 2, 2, 3, 2];
        for (day, &count) in daily_counts.iter().enumerate() {
            for _ in 0..count {
                let obs = Observation::new(
                    NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()
                        + chrono::Duration::days(day as i64),
                )
                .with_amount(dec!(1000));
                baseline_mgr.record_observation("ENTITY1", obs);
            }
        }

        // Check for frequency deviation (10 transactions in a day, way above ~2 avg)
        let deviation = baseline_mgr.check_frequency_deviation("ENTITY1", 10.0);

        // Should detect frequency anomaly
        assert!(deviation.is_some());
        assert_eq!(
            deviation.unwrap().deviation_type,
            DeviationType::FrequencyAnomaly
        );
    }

    #[test]
    fn test_insufficient_baseline() {
        let mut baseline_mgr = BehavioralBaseline::default();

        // Only add 5 observations (less than min_observations = 10)
        for i in 0..5 {
            let obs = Observation::new(
                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap() + chrono::Duration::days(i),
            )
            .with_amount(dec!(1000));
            baseline_mgr.record_observation("ENTITY1", obs);
        }

        // Check for deviation - should return empty since baseline not established
        let unusual_obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 25).unwrap())
            .with_amount(dec!(50000));

        let deviations = baseline_mgr.check_deviation("ENTITY1", &unusual_obs);

        // Should not detect anything due to insufficient baseline
        assert!(deviations.is_empty());
    }

    #[test]
    fn test_typical_hours_calculation() {
        let mut baseline = EntityBaseline::new();

        // Add observations mostly between 9 AM and 5 PM
        for _ in 0..10 {
            for hour in 9..17 {
                let obs = Observation::new(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
                    .with_time(NaiveTime::from_hms_opt(hour, 30, 0).unwrap());
                baseline.add_observation(obs);
            }
        }

        assert!(baseline.is_within_typical_hours(NaiveTime::from_hms_opt(10, 0, 0).unwrap()));
        assert!(baseline.is_within_typical_hours(NaiveTime::from_hms_opt(14, 0, 0).unwrap()));
    }
}
