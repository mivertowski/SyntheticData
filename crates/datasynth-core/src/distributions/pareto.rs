//! Pareto distribution for heavy-tailed data generation.
//!
//! The Pareto distribution is useful for modeling phenomena that follow
//! the "80/20 rule" - e.g., 20% of vendors accounting for 80% of spend,
//! or capital expenditure patterns where most transactions are small
//! but a few are very large.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Pareto};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configuration for Pareto distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoConfig {
    /// Shape parameter (alpha) - controls tail heaviness.
    /// Lower values = heavier tail (more extreme values).
    /// Typical values: 1.5-3.0 for financial data.
    pub alpha: f64,
    /// Scale parameter (x_min) - minimum value.
    /// All samples will be >= x_min.
    pub x_min: f64,
    /// Maximum value (clamps output).
    #[serde(default)]
    pub max_value: Option<f64>,
    /// Number of decimal places for rounding.
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_decimal_places() -> u8 {
    2
}

impl Default for ParetoConfig {
    fn default() -> Self {
        Self {
            alpha: 2.0,   // Moderate tail heaviness
            x_min: 100.0, // Minimum $100
            max_value: None,
            decimal_places: 2,
        }
    }
}

impl ParetoConfig {
    /// Create a new Pareto configuration.
    pub fn new(alpha: f64, x_min: f64) -> Self {
        Self {
            alpha,
            x_min,
            ..Default::default()
        }
    }

    /// Create a configuration for capital expenditures (heavy tail).
    pub fn capital_expenditure() -> Self {
        Self {
            alpha: 1.5,      // Heavy tail
            x_min: 10_000.0, // Minimum $10,000
            max_value: Some(100_000_000.0),
            decimal_places: 2,
        }
    }

    /// Create a configuration for maintenance costs.
    pub fn maintenance_costs() -> Self {
        Self {
            alpha: 2.5,   // Moderate tail
            x_min: 500.0, // Minimum $500
            max_value: Some(500_000.0),
            decimal_places: 2,
        }
    }

    /// Create a configuration for vendor spend distribution.
    pub fn vendor_spend() -> Self {
        Self {
            alpha: 1.8, // 80/20 rule approximation
            x_min: 1_000.0,
            max_value: Some(10_000_000.0),
            decimal_places: 2,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.alpha <= 0.0 {
            return Err("alpha must be positive".to_string());
        }
        if self.x_min <= 0.0 {
            return Err("x_min must be positive".to_string());
        }
        if let Some(max) = self.max_value {
            if max <= self.x_min {
                return Err("max_value must be greater than x_min".to_string());
            }
        }
        Ok(())
    }

    /// Get the expected value (mean) of the distribution.
    /// Only defined for alpha > 1.
    pub fn expected_value(&self) -> Option<f64> {
        if self.alpha > 1.0 {
            Some(self.alpha * self.x_min / (self.alpha - 1.0))
        } else {
            None // Infinite for alpha <= 1
        }
    }

    /// Get the variance of the distribution.
    /// Only defined for alpha > 2.
    pub fn variance(&self) -> Option<f64> {
        if self.alpha > 2.0 {
            let numerator = self.x_min.powi(2) * self.alpha;
            let denominator = (self.alpha - 1.0).powi(2) * (self.alpha - 2.0);
            Some(numerator / denominator)
        } else {
            None // Infinite for alpha <= 2
        }
    }
}

/// Pareto distribution sampler.
pub struct ParetoSampler {
    rng: ChaCha8Rng,
    config: ParetoConfig,
    distribution: Pareto<f64>,
    decimal_multiplier: f64,
}

impl ParetoSampler {
    /// Create a new Pareto sampler.
    pub fn new(seed: u64, config: ParetoConfig) -> Result<Self, String> {
        config.validate()?;

        let distribution = Pareto::new(config.x_min, config.alpha)
            .map_err(|e| format!("Invalid Pareto distribution: {}", e))?;

        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            distribution,
            decimal_multiplier,
        })
    }

    /// Sample a value from the distribution.
    pub fn sample(&mut self) -> f64 {
        let mut value = self.distribution.sample(&mut self.rng);

        // Apply max constraint
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        // Round to decimal places
        (value * self.decimal_multiplier).round() / self.decimal_multiplier
    }

    /// Sample a value as Decimal.
    pub fn sample_decimal(&mut self) -> Decimal {
        let value = self.sample();
        Decimal::from_f64_retain(value).unwrap_or(Decimal::ONE)
    }

    /// Sample multiple values.
    pub fn sample_n(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Get the configuration.
    pub fn config(&self) -> &ParetoConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_pareto_validation() {
        let config = ParetoConfig::new(2.0, 100.0);
        assert!(config.validate().is_ok());

        let invalid_alpha = ParetoConfig::new(-1.0, 100.0);
        assert!(invalid_alpha.validate().is_err());

        let invalid_xmin = ParetoConfig::new(2.0, -100.0);
        assert!(invalid_xmin.validate().is_err());
    }

    #[test]
    fn test_pareto_sampling() {
        let config = ParetoConfig::new(2.0, 100.0);
        let mut sampler = ParetoSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be >= x_min
        assert!(samples.iter().all(|&x| x >= 100.0));
    }

    #[test]
    fn test_pareto_determinism() {
        let config = ParetoConfig::new(2.0, 100.0);

        let mut sampler1 = ParetoSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = ParetoSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_pareto_max_constraint() {
        let mut config = ParetoConfig::new(2.0, 100.0);
        config.max_value = Some(1000.0);

        let mut sampler = ParetoSampler::new(42, config).unwrap();
        let samples = sampler.sample_n(1000);

        assert!(samples.iter().all(|&x| x <= 1000.0));
    }

    #[test]
    fn test_pareto_expected_value() {
        let config = ParetoConfig::new(2.0, 100.0);
        // E[X] = alpha * x_min / (alpha - 1) = 2 * 100 / 1 = 200
        assert_eq!(config.expected_value(), Some(200.0));

        // No expected value for alpha <= 1
        let heavy_tail = ParetoConfig::new(1.0, 100.0);
        assert_eq!(heavy_tail.expected_value(), None);
    }

    #[test]
    fn test_pareto_presets() {
        let capex = ParetoConfig::capital_expenditure();
        assert!(capex.validate().is_ok());
        assert_eq!(capex.alpha, 1.5);

        let maintenance = ParetoConfig::maintenance_costs();
        assert!(maintenance.validate().is_ok());

        let vendor = ParetoConfig::vendor_spend();
        assert!(vendor.validate().is_ok());
    }

    #[test]
    fn test_heavy_tail_behavior() {
        // With alpha=1.5 and x_min=100:
        // P(X > 1000) = (100/1000)^1.5 = 0.0316 (~3.16%)
        // For 10000 samples, expect ~316 values > 1000
        let config = ParetoConfig::new(1.5, 100.0);
        let mut sampler = ParetoSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(10000);
        let large_values = samples.iter().filter(|&&x| x > 1000.0).count();

        // With heavy tail, we should have around 300 large values (3%)
        // Use a loose bound to account for statistical variation
        assert!(
            large_values > 200 && large_values < 500,
            "Expected ~316 values > 1000, got {}",
            large_values
        );
    }
}
