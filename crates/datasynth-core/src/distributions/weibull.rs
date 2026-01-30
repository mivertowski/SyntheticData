//! Weibull distribution for time-to-event and duration modeling.
//!
//! The Weibull distribution is commonly used for modeling:
//! - Days-to-payment (accounts receivable aging)
//! - Time-to-failure (asset depreciation studies)
//! - Processing times (document flow durations)

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Weibull};
use serde::{Deserialize, Serialize};

/// Configuration for Weibull distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeibullConfig {
    /// Shape parameter (k) - controls the shape of the distribution.
    /// k < 1: decreasing failure rate (early failures more likely)
    /// k = 1: constant failure rate (exponential distribution)
    /// k > 1: increasing failure rate (wear-out failures)
    pub shape: f64,
    /// Scale parameter (lambda) - controls the characteristic life.
    /// 63.2% of values will be below this threshold.
    pub scale: f64,
    /// Minimum value (shifts the distribution).
    #[serde(default)]
    pub min_value: f64,
    /// Maximum value (clamps output).
    #[serde(default)]
    pub max_value: Option<f64>,
    /// Whether to round to integers (useful for days).
    #[serde(default)]
    pub round_to_integer: bool,
}

impl Default for WeibullConfig {
    fn default() -> Self {
        Self {
            shape: 1.5,  // Increasing failure rate
            scale: 30.0, // 30 day characteristic time
            min_value: 0.0,
            max_value: None,
            round_to_integer: false,
        }
    }
}

impl WeibullConfig {
    /// Create a new Weibull configuration.
    pub fn new(shape: f64, scale: f64) -> Self {
        Self {
            shape,
            scale,
            ..Default::default()
        }
    }

    /// Create a configuration for days-to-payment modeling.
    pub fn days_to_payment() -> Self {
        Self {
            shape: 1.8,             // Slight increasing hazard (more likely to pay as time goes on)
            scale: 35.0,            // Characteristic payment around 35 days
            min_value: 1.0,         // At least 1 day
            max_value: Some(120.0), // Cap at 120 days
            round_to_integer: true,
        }
    }

    /// Create a configuration for early payment behavior.
    pub fn early_payment() -> Self {
        Self {
            shape: 2.5,  // Strong increasing hazard
            scale: 15.0, // Characteristic payment around 15 days
            min_value: 1.0,
            max_value: Some(30.0),
            round_to_integer: true,
        }
    }

    /// Create a configuration for late payment behavior.
    pub fn late_payment() -> Self {
        Self {
            shape: 0.8,      // Decreasing hazard (procrastination)
            scale: 60.0,     // Characteristic payment around 60 days
            min_value: 30.0, // Already past due date
            max_value: Some(180.0),
            round_to_integer: true,
        }
    }

    /// Create a configuration for processing time.
    pub fn processing_time() -> Self {
        Self {
            shape: 2.0,            // Bell-shaped, typical processing time
            scale: 3.0,            // ~3 hours characteristic time
            min_value: 0.5,        // At least 30 minutes
            max_value: Some(24.0), // Cap at 24 hours
            round_to_integer: false,
        }
    }

    /// Create a configuration for asset useful life (years).
    pub fn asset_useful_life() -> Self {
        Self {
            shape: 3.5, // Wear-out failure pattern
            scale: 7.0, // ~7 year characteristic life
            min_value: 1.0,
            max_value: Some(20.0),
            round_to_integer: true,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.shape <= 0.0 {
            return Err("shape must be positive".to_string());
        }
        if self.scale <= 0.0 {
            return Err("scale must be positive".to_string());
        }
        if let Some(max) = self.max_value {
            if max <= self.min_value {
                return Err("max_value must be greater than min_value".to_string());
            }
        }
        Ok(())
    }

    /// Get the expected value (mean) of the distribution.
    pub fn expected_value(&self) -> f64 {
        use std::f64::consts::PI;

        // E[X] = scale * Gamma(1 + 1/shape)
        // Using Stirling approximation for Gamma function
        let arg = 1.0 + 1.0 / self.shape;
        let gamma_approx = (2.0 * PI / arg).sqrt() * (arg / std::f64::consts::E).powf(arg);
        self.min_value + self.scale * gamma_approx
    }

    /// Get the median of the distribution.
    pub fn median(&self) -> f64 {
        self.min_value + self.scale * (2.0_f64.ln()).powf(1.0 / self.shape)
    }

    /// Get the mode of the distribution.
    /// Only defined for shape > 1.
    pub fn mode(&self) -> Option<f64> {
        if self.shape > 1.0 {
            let mode = self.scale * ((self.shape - 1.0) / self.shape).powf(1.0 / self.shape);
            Some(self.min_value + mode)
        } else {
            None
        }
    }
}

/// Weibull distribution sampler.
pub struct WeibullSampler {
    rng: ChaCha8Rng,
    config: WeibullConfig,
    distribution: Weibull<f64>,
}

impl WeibullSampler {
    /// Create a new Weibull sampler.
    pub fn new(seed: u64, config: WeibullConfig) -> Result<Self, String> {
        config.validate()?;

        let distribution = Weibull::new(config.scale, config.shape)
            .map_err(|e| format!("Invalid Weibull distribution: {}", e))?;

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            distribution,
        })
    }

    /// Sample a value from the distribution.
    pub fn sample(&mut self) -> f64 {
        let mut value = self.distribution.sample(&mut self.rng) + self.config.min_value;

        // Apply max constraint
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        // Round to integer if configured
        if self.config.round_to_integer {
            value = value.round();
        }

        value
    }

    /// Sample a value as integer (for days).
    pub fn sample_days(&mut self) -> u32 {
        self.sample().max(0.0) as u32
    }

    /// Sample multiple values.
    pub fn sample_n(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Sample multiple values as days.
    pub fn sample_n_days(&mut self, n: usize) -> Vec<u32> {
        (0..n).map(|_| self.sample_days()).collect()
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Get the configuration.
    pub fn config(&self) -> &WeibullConfig {
        &self.config
    }
}

/// Result of survival analysis using Weibull.
#[derive(Debug, Clone)]
pub struct WeibullSurvivalResult {
    /// Time point
    pub time: f64,
    /// Survival probability at time t
    pub survival_probability: f64,
    /// Hazard rate at time t
    pub hazard_rate: f64,
}

impl WeibullConfig {
    /// Calculate survival probability at time t.
    pub fn survival_probability(&self, t: f64) -> f64 {
        if t <= self.min_value {
            return 1.0;
        }
        let adjusted_t = t - self.min_value;
        (-((adjusted_t / self.scale).powf(self.shape))).exp()
    }

    /// Calculate hazard rate at time t.
    pub fn hazard_rate(&self, t: f64) -> f64 {
        if t <= self.min_value {
            if self.shape < 1.0 {
                return f64::INFINITY; // Decreasing hazard starts at infinity
            }
            return 0.0;
        }
        let adjusted_t = t - self.min_value;
        (self.shape / self.scale) * (adjusted_t / self.scale).powf(self.shape - 1.0)
    }

    /// Generate survival analysis data.
    pub fn survival_analysis(&self, time_points: &[f64]) -> Vec<WeibullSurvivalResult> {
        time_points
            .iter()
            .map(|&t| WeibullSurvivalResult {
                time: t,
                survival_probability: self.survival_probability(t),
                hazard_rate: self.hazard_rate(t),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weibull_validation() {
        let config = WeibullConfig::new(1.5, 30.0);
        assert!(config.validate().is_ok());

        let invalid_shape = WeibullConfig::new(-1.0, 30.0);
        assert!(invalid_shape.validate().is_err());

        let invalid_scale = WeibullConfig::new(1.5, 0.0);
        assert!(invalid_scale.validate().is_err());
    }

    #[test]
    fn test_weibull_sampling() {
        let config = WeibullConfig::new(1.5, 30.0);
        let mut sampler = WeibullSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be non-negative
        assert!(samples.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn test_weibull_determinism() {
        let config = WeibullConfig::new(1.5, 30.0);

        let mut sampler1 = WeibullSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = WeibullSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_weibull_days_to_payment() {
        let config = WeibullConfig::days_to_payment();
        let mut sampler = WeibullSampler::new(42, config.clone()).unwrap();

        let samples = sampler.sample_n_days(1000);

        // All should be at least 1 day and at most 120 days
        assert!(samples.iter().all(|&x| x >= 1 && x <= 120));

        // Most should be around the characteristic time (35 days)
        let median_approx = samples.iter().copied().sum::<u32>() as f64 / 1000.0;
        assert!(median_approx > 20.0 && median_approx < 60.0);
    }

    #[test]
    fn test_weibull_median() {
        let config = WeibullConfig::new(2.0, 30.0);
        let median = config.median();

        // For k=2, median = scale * sqrt(ln(2)) ≈ 30 * 0.833 ≈ 24.99
        assert!((median - 24.99).abs() < 0.1);
    }

    #[test]
    fn test_weibull_mode() {
        let config = WeibullConfig::new(2.0, 30.0);
        let mode = config.mode();

        // For k=2, mode = scale * sqrt((k-1)/k) = 30 * sqrt(0.5) ≈ 21.21
        assert!(mode.is_some());
        assert!((mode.unwrap() - 21.21).abs() < 0.1);

        // No mode for k <= 1
        let no_mode_config = WeibullConfig::new(0.8, 30.0);
        assert!(no_mode_config.mode().is_none());
    }

    #[test]
    fn test_weibull_survival() {
        let config = WeibullConfig::new(2.0, 30.0);

        // At t=0, survival should be 1.0
        assert!((config.survival_probability(0.0) - 1.0).abs() < 0.001);

        // At t=median, survival should be 0.5
        let median = config.median();
        assert!((config.survival_probability(median) - 0.5).abs() < 0.01);

        // At t→∞, survival should approach 0
        assert!(config.survival_probability(1000.0) < 0.001);
    }

    #[test]
    fn test_weibull_hazard_shapes() {
        // Decreasing hazard (k < 1)
        let config_dec = WeibullConfig::new(0.5, 30.0);
        assert!(config_dec.hazard_rate(10.0) > config_dec.hazard_rate(50.0));

        // Increasing hazard (k > 1)
        let config_inc = WeibullConfig::new(2.0, 30.0);
        assert!(config_inc.hazard_rate(10.0) < config_inc.hazard_rate(50.0));
    }

    #[test]
    fn test_weibull_presets() {
        let early = WeibullConfig::early_payment();
        assert!(early.validate().is_ok());

        let late = WeibullConfig::late_payment();
        assert!(late.validate().is_ok());

        let processing = WeibullConfig::processing_time();
        assert!(processing.validate().is_ok());

        let asset = WeibullConfig::asset_useful_life();
        assert!(asset.validate().is_ok());
    }
}
