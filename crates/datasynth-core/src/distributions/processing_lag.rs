//! Processing lag modeling for event-to-posting time delays.
//!
//! Models the realistic time delays between business events and their
//! recording in the accounting system, including cross-day posting logic.

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of business event that triggers a posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Sales order creation
    SalesOrder,
    /// Purchase order creation
    PurchaseOrder,
    /// Goods receipt
    GoodsReceipt,
    /// Invoice receipt (vendor)
    InvoiceReceipt,
    /// Invoice issue (customer)
    InvoiceIssue,
    /// Payment (incoming or outgoing)
    Payment,
    /// Manual journal entry
    JournalEntry,
    /// Accrual entry
    Accrual,
    /// Depreciation posting
    Depreciation,
    /// Intercompany transaction
    Intercompany,
    /// Period close adjustment
    PeriodClose,
}

/// Distribution type for lag calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LagDistributionType {
    /// Fixed lag (deterministic)
    Fixed {
        /// Lag in hours
        hours: f64,
    },
    /// Normal distribution
    Normal {
        /// Mean lag in hours
        mu: f64,
        /// Standard deviation in hours
        sigma: f64,
    },
    /// Log-normal distribution (common for processing delays)
    LogNormal {
        /// Log-scale mean
        mu: f64,
        /// Log-scale standard deviation
        sigma: f64,
    },
    /// Exponential distribution
    Exponential {
        /// Rate parameter (1/mean)
        lambda: f64,
    },
}

impl Default for LagDistributionType {
    fn default() -> Self {
        Self::LogNormal {
            mu: 0.5, // ~1.6 hours median
            sigma: 0.8,
        }
    }
}

/// Configuration for a specific lag distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagDistribution {
    /// Distribution type for sampling
    #[serde(default)]
    pub distribution: LagDistributionType,
    /// Minimum lag in hours (floor)
    #[serde(default)]
    pub min_lag_hours: f64,
    /// Maximum lag in hours (ceiling)
    #[serde(default = "default_max_lag")]
    pub max_lag_hours: f64,
}

fn default_max_lag() -> f64 {
    72.0 // 3 days
}

impl Default for LagDistribution {
    fn default() -> Self {
        Self {
            distribution: LagDistributionType::default(),
            min_lag_hours: 0.0,
            max_lag_hours: 72.0,
        }
    }
}

impl LagDistribution {
    /// Create a fixed lag distribution.
    pub fn fixed(hours: f64) -> Self {
        Self {
            distribution: LagDistributionType::Fixed { hours },
            min_lag_hours: hours,
            max_lag_hours: hours,
        }
    }

    /// Create a log-normal distribution with typical accounting delays.
    pub fn log_normal(mu: f64, sigma: f64) -> Self {
        Self {
            distribution: LagDistributionType::LogNormal { mu, sigma },
            min_lag_hours: 0.0,
            max_lag_hours: 72.0,
        }
    }

    /// Create a normal distribution.
    pub fn normal(mu: f64, sigma: f64) -> Self {
        Self {
            distribution: LagDistributionType::Normal { mu, sigma },
            min_lag_hours: 0.0,
            max_lag_hours: 72.0,
        }
    }

    /// Sample a lag value in hours.
    pub fn sample(&self, rng: &mut ChaCha8Rng) -> f64 {
        let raw = match &self.distribution {
            LagDistributionType::Fixed { hours } => *hours,
            LagDistributionType::Normal { mu, sigma } => {
                // Box-Muller transform for normal distribution
                let u1: f64 = rng.gen();
                let u2: f64 = rng.gen();
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mu + sigma * z
            }
            LagDistributionType::LogNormal { mu, sigma } => {
                // Sample from log-normal
                let u1: f64 = rng.gen();
                let u2: f64 = rng.gen();
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                (mu + sigma * z).exp()
            }
            LagDistributionType::Exponential { lambda } => {
                let u: f64 = rng.gen();
                -u.ln() / lambda
            }
        };

        raw.clamp(self.min_lag_hours, self.max_lag_hours)
    }
}

/// Configuration for cross-day posting behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDayConfig {
    /// Enable cross-day posting logic
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Probability of cross-day posting by hour
    /// Keys are hours (0-23), values are probabilities (0.0-1.0)
    #[serde(default)]
    pub probability_by_hour: HashMap<u8, f64>,
    /// Working day start hour (events before this may post same day)
    #[serde(default = "default_work_start")]
    pub work_start_hour: u8,
    /// Working day end hour (events after this likely post next day)
    #[serde(default = "default_work_end")]
    pub work_end_hour: u8,
    /// Cutoff hour for same-day posting
    #[serde(default = "default_cutoff")]
    pub same_day_cutoff_hour: u8,
}

fn default_true() -> bool {
    true
}

fn default_work_start() -> u8 {
    8
}

fn default_work_end() -> u8 {
    18
}

fn default_cutoff() -> u8 {
    16
}

impl Default for CrossDayConfig {
    fn default() -> Self {
        let mut probability_by_hour = HashMap::new();
        // After 5pm, increasing probability of next-day posting
        probability_by_hour.insert(17, 0.3);
        probability_by_hour.insert(18, 0.6);
        probability_by_hour.insert(19, 0.8);
        probability_by_hour.insert(20, 0.9);
        probability_by_hour.insert(21, 0.95);
        probability_by_hour.insert(22, 0.99);
        probability_by_hour.insert(23, 0.99);

        Self {
            enabled: true,
            probability_by_hour,
            work_start_hour: 8,
            work_end_hour: 18,
            same_day_cutoff_hour: 16,
        }
    }
}

impl CrossDayConfig {
    /// Get the probability of next-day posting for a given hour.
    pub fn next_day_probability(&self, hour: u8) -> f64 {
        if !self.enabled {
            return 0.0;
        }

        if let Some(&prob) = self.probability_by_hour.get(&hour) {
            return prob;
        }

        // Default behavior based on work hours
        if hour < self.same_day_cutoff_hour {
            0.0 // Before cutoff, same-day
        } else if hour < self.work_end_hour {
            0.2 // Late afternoon, some spillover
        } else {
            0.8 // After hours, likely next day
        }
    }
}

/// Full configuration for processing lags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingLagConfig {
    /// Enable processing lag calculations
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default lag distribution (used when event type not specified)
    #[serde(default)]
    pub default_lag: LagDistribution,

    /// Event-specific lag distributions
    #[serde(default)]
    pub event_lags: HashMap<EventType, LagDistribution>,

    /// Cross-day posting configuration
    #[serde(default)]
    pub cross_day: CrossDayConfig,
}

impl Default for ProcessingLagConfig {
    fn default() -> Self {
        let mut event_lags = HashMap::new();

        // Typical lags for different event types
        event_lags.insert(
            EventType::SalesOrder,
            LagDistribution::log_normal(0.5, 0.8), // Quick, ~1.6 hours median
        );
        event_lags.insert(
            EventType::PurchaseOrder,
            LagDistribution::log_normal(0.7, 0.6), // Slightly longer
        );
        event_lags.insert(
            EventType::GoodsReceipt,
            LagDistribution::log_normal(1.0, 0.5), // ~2.7 hours median
        );
        event_lags.insert(
            EventType::InvoiceReceipt,
            LagDistribution::log_normal(1.5, 0.6), // ~4.5 hours median
        );
        event_lags.insert(
            EventType::InvoiceIssue,
            LagDistribution::log_normal(0.3, 0.5), // Fast, ~1.3 hours
        );
        event_lags.insert(
            EventType::Payment,
            LagDistribution::log_normal(0.8, 0.7), // ~2.2 hours median
        );
        event_lags.insert(
            EventType::JournalEntry,
            LagDistribution::log_normal(0.0, 0.3), // Near instant, ~1 hour
        );
        event_lags.insert(
            EventType::Accrual,
            LagDistribution::fixed(0.0), // Immediate (batch)
        );
        event_lags.insert(
            EventType::Depreciation,
            LagDistribution::fixed(0.0), // Immediate (batch)
        );
        event_lags.insert(
            EventType::Intercompany,
            LagDistribution::log_normal(2.0, 0.8), // Longer due to coordination
        );
        event_lags.insert(
            EventType::PeriodClose,
            LagDistribution::fixed(0.0), // Immediate (batch)
        );

        Self {
            enabled: true,
            default_lag: LagDistribution::log_normal(0.5, 0.8),
            event_lags,
            cross_day: CrossDayConfig::default(),
        }
    }
}

impl ProcessingLagConfig {
    /// Get the lag distribution for an event type.
    pub fn get_lag_distribution(&self, event_type: EventType) -> &LagDistribution {
        self.event_lags
            .get(&event_type)
            .unwrap_or(&self.default_lag)
    }
}

/// Calculator for processing lags.
pub struct ProcessingLagCalculator {
    config: ProcessingLagConfig,
    rng: ChaCha8Rng,
}

impl ProcessingLagCalculator {
    /// Create a new calculator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            config: ProcessingLagConfig::default(),
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: ProcessingLagConfig) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Calculate the posting time for an event.
    ///
    /// Takes the event datetime and returns the datetime when it would be posted
    /// to the accounting system.
    pub fn calculate_posting_time(
        &mut self,
        event_type: EventType,
        event_datetime: NaiveDateTime,
    ) -> NaiveDateTime {
        if !self.config.enabled {
            return event_datetime;
        }

        // Get lag distribution for this event type
        let lag_dist = self.config.get_lag_distribution(event_type);
        let lag_hours = lag_dist.sample(&mut self.rng);

        // Calculate raw posting time
        let lag_seconds = (lag_hours * 3600.0) as i64;
        let mut posting_time = event_datetime + Duration::seconds(lag_seconds);

        // Check for cross-day posting
        if self.should_post_next_day(event_datetime.hour() as u8) {
            // Move to next business day morning
            let next_day = event_datetime.date() + Duration::days(1);
            let morning_hour = self.config.cross_day.work_start_hour as u32;
            let morning_minute: u32 = self.rng.gen_range(0..60);
            posting_time = NaiveDateTime::new(
                next_day,
                NaiveTime::from_hms_opt(morning_hour, morning_minute, 0).unwrap(),
            );
        }

        // Ensure posting is not before event
        if posting_time < event_datetime {
            posting_time = event_datetime;
        }

        posting_time
    }

    /// Determine if an event should be posted the next day based on its hour.
    pub fn should_post_next_day(&mut self, hour: u8) -> bool {
        let prob = self.config.cross_day.next_day_probability(hour);
        self.rng.gen::<f64>() < prob
    }

    /// Calculate posting date (ignoring time).
    pub fn calculate_posting_date(
        &mut self,
        event_type: EventType,
        event_date: NaiveDate,
        event_hour: u8,
    ) -> NaiveDate {
        let event_time = NaiveTime::from_hms_opt(event_hour as u32, 0, 0).unwrap();
        let event_datetime = NaiveDateTime::new(event_date, event_time);
        self.calculate_posting_time(event_type, event_datetime)
            .date()
    }

    /// Get the configuration.
    pub fn config(&self) -> &ProcessingLagConfig {
        &self.config
    }

    /// Reset with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Schema configuration for YAML/JSON deserialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingLagSchemaConfig {
    /// Enable processing lag calculations
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Sales order lag (log-normal mu, sigma)
    #[serde(default)]
    pub sales_order_lag: Option<LagSchemaConfig>,

    /// Purchase order lag
    #[serde(default)]
    pub purchase_order_lag: Option<LagSchemaConfig>,

    /// Goods receipt lag
    #[serde(default)]
    pub goods_receipt_lag: Option<LagSchemaConfig>,

    /// Invoice receipt lag
    #[serde(default)]
    pub invoice_receipt_lag: Option<LagSchemaConfig>,

    /// Invoice issue lag
    #[serde(default)]
    pub invoice_issue_lag: Option<LagSchemaConfig>,

    /// Payment lag
    #[serde(default)]
    pub payment_lag: Option<LagSchemaConfig>,

    /// Journal entry lag
    #[serde(default)]
    pub journal_entry_lag: Option<LagSchemaConfig>,

    /// Cross-day posting configuration
    #[serde(default)]
    pub cross_day_posting: Option<CrossDaySchemaConfig>,
}

/// Schema config for a lag distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagSchemaConfig {
    /// Log-scale mean (for log-normal)
    pub mu: f64,
    /// Log-scale standard deviation (for log-normal)
    pub sigma: f64,
    /// Minimum lag in hours
    #[serde(default)]
    pub min_hours: Option<f64>,
    /// Maximum lag in hours
    #[serde(default)]
    pub max_hours: Option<f64>,
}

impl LagSchemaConfig {
    /// Convert to LagDistribution.
    pub fn to_distribution(&self) -> LagDistribution {
        LagDistribution {
            distribution: LagDistributionType::LogNormal {
                mu: self.mu,
                sigma: self.sigma,
            },
            min_lag_hours: self.min_hours.unwrap_or(0.0),
            max_lag_hours: self.max_hours.unwrap_or(72.0),
        }
    }
}

/// Schema config for cross-day posting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossDaySchemaConfig {
    /// Enable cross-day posting
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Probability by hour (map of hour -> probability)
    #[serde(default)]
    pub probability_by_hour: HashMap<u8, f64>,
}

impl ProcessingLagSchemaConfig {
    /// Convert to ProcessingLagConfig.
    pub fn to_config(&self) -> ProcessingLagConfig {
        let mut config = ProcessingLagConfig {
            enabled: self.enabled,
            ..Default::default()
        };

        // Apply event-specific lags
        if let Some(lag) = &self.sales_order_lag {
            config
                .event_lags
                .insert(EventType::SalesOrder, lag.to_distribution());
        }
        if let Some(lag) = &self.purchase_order_lag {
            config
                .event_lags
                .insert(EventType::PurchaseOrder, lag.to_distribution());
        }
        if let Some(lag) = &self.goods_receipt_lag {
            config
                .event_lags
                .insert(EventType::GoodsReceipt, lag.to_distribution());
        }
        if let Some(lag) = &self.invoice_receipt_lag {
            config
                .event_lags
                .insert(EventType::InvoiceReceipt, lag.to_distribution());
        }
        if let Some(lag) = &self.invoice_issue_lag {
            config
                .event_lags
                .insert(EventType::InvoiceIssue, lag.to_distribution());
        }
        if let Some(lag) = &self.payment_lag {
            config
                .event_lags
                .insert(EventType::Payment, lag.to_distribution());
        }
        if let Some(lag) = &self.journal_entry_lag {
            config
                .event_lags
                .insert(EventType::JournalEntry, lag.to_distribution());
        }

        // Apply cross-day config
        if let Some(cross_day) = &self.cross_day_posting {
            config.cross_day.enabled = cross_day.enabled;
            if !cross_day.probability_by_hour.is_empty() {
                config.cross_day.probability_by_hour = cross_day.probability_by_hour.clone();
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_lag() {
        let lag = LagDistribution::fixed(2.0);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Should always return 2.0
        for _ in 0..10 {
            assert!((lag.sample(&mut rng) - 2.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_log_normal_lag() {
        let lag = LagDistribution::log_normal(0.5, 0.5);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut samples: Vec<f64> = (0..1000).map(|_| lag.sample(&mut rng)).collect();
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Median should be around e^0.5 ≈ 1.65
        let median = samples[500];
        assert!(median > 1.0 && median < 3.0);

        // All values should be within bounds
        assert!(samples.iter().all(|&x| (0.0..=72.0).contains(&x)));
    }

    #[test]
    fn test_cross_day_probability() {
        let config = CrossDayConfig::default();

        // Early morning - no cross-day
        assert!(config.next_day_probability(8) < 0.1);

        // Mid-day - no cross-day
        assert!(config.next_day_probability(14) < 0.1);

        // After 5pm - increasing probability
        assert!(config.next_day_probability(17) > 0.2);
        assert!(config.next_day_probability(19) > 0.7);
        assert!(config.next_day_probability(22) > 0.9);
    }

    #[test]
    fn test_processing_lag_calculator() {
        let mut calc = ProcessingLagCalculator::new(42);

        let event_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        );

        let posting_time = calc.calculate_posting_time(EventType::SalesOrder, event_time);

        // Posting should be after event
        assert!(posting_time >= event_time);

        // For sales orders with mid-morning event, posting should be same day or close
        let hours_diff = (posting_time - event_time).num_hours();
        assert!(hours_diff < 24);
    }

    #[test]
    fn test_late_event_cross_day() {
        // Test with high probability of cross-day posting
        let mut config = ProcessingLagConfig::default();
        config.cross_day.probability_by_hour.insert(22, 1.0); // Force next-day for 10pm

        let mut calc = ProcessingLagCalculator::with_config(42, config);

        let event_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
        );

        let posting_time = calc.calculate_posting_time(EventType::SalesOrder, event_time);

        // Should post next day
        assert!(posting_time.date() > event_time.date());
    }

    #[test]
    fn test_event_specific_lags() {
        let config = ProcessingLagConfig::default();

        // Accrual and depreciation should have fixed 0 lag
        let accrual_lag = config.get_lag_distribution(EventType::Accrual);
        if let LagDistributionType::Fixed { hours } = accrual_lag.distribution {
            assert!((hours - 0.0).abs() < 0.01);
        } else {
            panic!("Accrual should have fixed lag");
        }

        // Invoice receipt should have longer lag than sales order
        let invoice_lag = config.get_lag_distribution(EventType::InvoiceReceipt);
        let sales_lag = config.get_lag_distribution(EventType::SalesOrder);

        if let (
            LagDistributionType::LogNormal { mu: inv_mu, .. },
            LagDistributionType::LogNormal { mu: sales_mu, .. },
        ) = (&invoice_lag.distribution, &sales_lag.distribution)
        {
            assert!(inv_mu > sales_mu);
        }
    }

    #[test]
    fn test_schema_config_conversion() {
        let schema = ProcessingLagSchemaConfig {
            enabled: true,
            sales_order_lag: Some(LagSchemaConfig {
                mu: 1.0,
                sigma: 0.5,
                min_hours: Some(0.5),
                max_hours: Some(24.0),
            }),
            cross_day_posting: Some(CrossDaySchemaConfig {
                enabled: true,
                probability_by_hour: {
                    let mut m = HashMap::new();
                    m.insert(18, 0.5);
                    m
                },
            }),
            ..Default::default()
        };

        let config = schema.to_config();

        // Check sales order lag was customized
        let sales_lag = config.get_lag_distribution(EventType::SalesOrder);
        assert!((sales_lag.min_lag_hours - 0.5).abs() < 0.01);
        assert!((sales_lag.max_lag_hours - 24.0).abs() < 0.01);

        // Check cross-day config
        assert_eq!(config.cross_day.probability_by_hour.get(&18), Some(&0.5));
    }

    #[test]
    fn test_calculate_posting_date() {
        let mut calc = ProcessingLagCalculator::new(42);

        let event_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let posting_date = calc.calculate_posting_date(EventType::JournalEntry, event_date, 10);

        // Journal entries are quick, should be same day or close
        let days_diff = (posting_date - event_date).num_days();
        assert!(days_diff <= 1);
    }
}
