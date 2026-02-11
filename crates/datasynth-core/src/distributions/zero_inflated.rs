//! Zero-inflated distributions for data with excess zeros.
//!
//! Zero-inflated distributions model scenarios where zeros occur more
//! frequently than a standard distribution would predict, such as:
//! - Credit memos and returns (most transactions have no credits)
//! - Warranty claims (most products have no claims)
//! - Late payment penalties (most payments have no penalties)
//! - Adjustment entries (most periods have no adjustments)

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Exp, LogNormal, Poisson};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of base distribution for the non-zero values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ZeroInflatedBaseDistribution {
    /// Log-normal distribution (positive amounts)
    #[default]
    LogNormal,
    /// Exponential distribution (time-based or decay patterns)
    Exponential,
    /// Poisson distribution (count data)
    Poisson,
}

/// Configuration for zero-inflated distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroInflatedConfig {
    /// Probability of a structural zero (0.0-1.0).
    /// Higher values = more zeros.
    pub zero_probability: f64,
    /// Type of base distribution for non-zero values.
    pub base_distribution: ZeroInflatedBaseDistribution,
    /// Mu parameter for log-normal base distribution.
    #[serde(default = "default_mu")]
    pub lognormal_mu: f64,
    /// Sigma parameter for log-normal base distribution.
    #[serde(default = "default_sigma")]
    pub lognormal_sigma: f64,
    /// Lambda parameter for exponential base distribution.
    #[serde(default = "default_lambda")]
    pub exponential_lambda: f64,
    /// Lambda parameter for Poisson base distribution.
    #[serde(default = "default_poisson_lambda")]
    pub poisson_lambda: f64,
    /// Minimum non-zero value.
    #[serde(default = "default_min_value")]
    pub min_value: f64,
    /// Maximum value (clamps output).
    #[serde(default)]
    pub max_value: Option<f64>,
    /// Number of decimal places for rounding.
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_mu() -> f64 {
    6.0
}

fn default_sigma() -> f64 {
    1.5
}

fn default_lambda() -> f64 {
    0.01
}

fn default_poisson_lambda() -> f64 {
    3.0
}

fn default_min_value() -> f64 {
    0.01
}

fn default_decimal_places() -> u8 {
    2
}

impl Default for ZeroInflatedConfig {
    fn default() -> Self {
        Self {
            zero_probability: 0.7, // 70% zeros
            base_distribution: ZeroInflatedBaseDistribution::LogNormal,
            lognormal_mu: 6.0,
            lognormal_sigma: 1.5,
            exponential_lambda: 0.01,
            poisson_lambda: 3.0,
            min_value: 0.01,
            max_value: None,
            decimal_places: 2,
        }
    }
}

impl ZeroInflatedConfig {
    /// Create a new zero-inflated configuration with log-normal base.
    pub fn lognormal(zero_probability: f64, mu: f64, sigma: f64) -> Self {
        Self {
            zero_probability,
            base_distribution: ZeroInflatedBaseDistribution::LogNormal,
            lognormal_mu: mu,
            lognormal_sigma: sigma,
            ..Default::default()
        }
    }

    /// Create a new zero-inflated configuration with exponential base.
    pub fn exponential(zero_probability: f64, lambda: f64) -> Self {
        Self {
            zero_probability,
            base_distribution: ZeroInflatedBaseDistribution::Exponential,
            exponential_lambda: lambda,
            ..Default::default()
        }
    }

    /// Create a new zero-inflated configuration with Poisson base.
    pub fn poisson(zero_probability: f64, lambda: f64) -> Self {
        Self {
            zero_probability,
            base_distribution: ZeroInflatedBaseDistribution::Poisson,
            poisson_lambda: lambda,
            decimal_places: 0, // Poisson is discrete
            min_value: 0.0,
            ..Default::default()
        }
    }

    /// Create a configuration for credit memos/returns.
    pub fn credit_memos() -> Self {
        Self {
            zero_probability: 0.85, // 85% have no credits
            base_distribution: ZeroInflatedBaseDistribution::LogNormal,
            lognormal_mu: 5.5, // ~$245 median credit
            lognormal_sigma: 1.2,
            min_value: 10.0, // Minimum $10 credit
            max_value: Some(50_000.0),
            decimal_places: 2,
            ..Default::default()
        }
    }

    /// Create a configuration for warranty claims.
    pub fn warranty_claims() -> Self {
        Self {
            zero_probability: 0.95, // 95% have no claims
            base_distribution: ZeroInflatedBaseDistribution::LogNormal,
            lognormal_mu: 6.0, // ~$403 median claim
            lognormal_sigma: 1.5,
            min_value: 25.0,
            max_value: Some(10_000.0),
            decimal_places: 2,
            ..Default::default()
        }
    }

    /// Create a configuration for late payment penalties.
    pub fn late_penalties() -> Self {
        Self {
            zero_probability: 0.80, // 80% pay on time
            base_distribution: ZeroInflatedBaseDistribution::LogNormal,
            lognormal_mu: 4.0, // ~$55 median penalty
            lognormal_sigma: 1.0,
            min_value: 5.0,
            max_value: Some(5_000.0),
            decimal_places: 2,
            ..Default::default()
        }
    }

    /// Create a configuration for adjustment entries (count-based).
    pub fn adjustment_count() -> Self {
        Self {
            zero_probability: 0.70, // 70% have no adjustments
            base_distribution: ZeroInflatedBaseDistribution::Poisson,
            poisson_lambda: 2.0, // Average 2 adjustments when they occur
            min_value: 0.0,
            max_value: Some(10.0),
            decimal_places: 0,
            ..Default::default()
        }
    }

    /// Create a configuration for returns processing time.
    pub fn return_processing_time() -> Self {
        Self {
            zero_probability: 0.90, // 90% have no returns
            base_distribution: ZeroInflatedBaseDistribution::Exponential,
            exponential_lambda: 0.1, // Average 10 days processing
            min_value: 1.0,
            max_value: Some(60.0),
            decimal_places: 0,
            ..Default::default()
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.zero_probability) {
            return Err("zero_probability must be between 0.0 and 1.0".to_string());
        }

        match self.base_distribution {
            ZeroInflatedBaseDistribution::LogNormal => {
                if self.lognormal_sigma <= 0.0 {
                    return Err("lognormal_sigma must be positive".to_string());
                }
            }
            ZeroInflatedBaseDistribution::Exponential => {
                if self.exponential_lambda <= 0.0 {
                    return Err("exponential_lambda must be positive".to_string());
                }
            }
            ZeroInflatedBaseDistribution::Poisson => {
                if self.poisson_lambda <= 0.0 {
                    return Err("poisson_lambda must be positive".to_string());
                }
            }
        }

        if let Some(max) = self.max_value {
            if max <= self.min_value {
                return Err("max_value must be greater than min_value".to_string());
            }
        }

        Ok(())
    }

    /// Get the expected value (mean) including zeros.
    pub fn expected_value(&self) -> f64 {
        let non_zero_prob = 1.0 - self.zero_probability;

        let non_zero_mean = match self.base_distribution {
            ZeroInflatedBaseDistribution::LogNormal => {
                (self.lognormal_mu + self.lognormal_sigma.powi(2) / 2.0).exp()
            }
            ZeroInflatedBaseDistribution::Exponential => 1.0 / self.exponential_lambda,
            ZeroInflatedBaseDistribution::Poisson => self.poisson_lambda,
        };

        non_zero_prob * non_zero_mean.max(self.min_value)
    }

    /// Get the probability of non-zero value.
    pub fn non_zero_probability(&self) -> f64 {
        1.0 - self.zero_probability
    }
}

/// Internal enum for holding the base distribution sampler.
enum BaseDistributionSampler {
    LogNormal(LogNormal<f64>),
    Exponential(Exp<f64>),
    Poisson(Poisson<f64>),
}

/// Zero-inflated distribution sampler.
pub struct ZeroInflatedSampler {
    rng: ChaCha8Rng,
    config: ZeroInflatedConfig,
    base_sampler: BaseDistributionSampler,
    decimal_multiplier: f64,
}

impl ZeroInflatedSampler {
    /// Create a new zero-inflated sampler.
    pub fn new(seed: u64, config: ZeroInflatedConfig) -> Result<Self, String> {
        config.validate()?;

        let base_sampler = match config.base_distribution {
            ZeroInflatedBaseDistribution::LogNormal => {
                let dist = LogNormal::new(config.lognormal_mu, config.lognormal_sigma)
                    .map_err(|e| format!("Invalid LogNormal distribution: {}", e))?;
                BaseDistributionSampler::LogNormal(dist)
            }
            ZeroInflatedBaseDistribution::Exponential => {
                let dist = Exp::new(config.exponential_lambda)
                    .map_err(|e| format!("Invalid Exponential distribution: {}", e))?;
                BaseDistributionSampler::Exponential(dist)
            }
            ZeroInflatedBaseDistribution::Poisson => {
                let dist = Poisson::new(config.poisson_lambda)
                    .map_err(|e| format!("Invalid Poisson distribution: {}", e))?;
                BaseDistributionSampler::Poisson(dist)
            }
        };

        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            base_sampler,
            decimal_multiplier,
        })
    }

    /// Sample a value from the distribution.
    pub fn sample(&mut self) -> f64 {
        // First, determine if this is a structural zero
        let p: f64 = self.rng.gen();
        if p < self.config.zero_probability {
            return 0.0;
        }

        // Sample from base distribution
        let mut value = match &self.base_sampler {
            BaseDistributionSampler::LogNormal(dist) => dist.sample(&mut self.rng),
            BaseDistributionSampler::Exponential(dist) => dist.sample(&mut self.rng),
            BaseDistributionSampler::Poisson(dist) => dist.sample(&mut self.rng),
        };

        // Apply constraints
        value = value.max(self.config.min_value);
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        // Round to decimal places
        (value * self.decimal_multiplier).round() / self.decimal_multiplier
    }

    /// Sample a value as Decimal.
    pub fn sample_decimal(&mut self) -> Decimal {
        let value = self.sample();
        Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO)
    }

    /// Sample with information about whether it's a structural zero.
    pub fn sample_with_info(&mut self) -> ZeroInflatedSample {
        let p: f64 = self.rng.gen();
        if p < self.config.zero_probability {
            return ZeroInflatedSample {
                value: 0.0,
                is_structural_zero: true,
            };
        }

        let mut value = match &self.base_sampler {
            BaseDistributionSampler::LogNormal(dist) => dist.sample(&mut self.rng),
            BaseDistributionSampler::Exponential(dist) => dist.sample(&mut self.rng),
            BaseDistributionSampler::Poisson(dist) => dist.sample(&mut self.rng),
        };

        value = value.max(self.config.min_value);
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }
        value = (value * self.decimal_multiplier).round() / self.decimal_multiplier;

        ZeroInflatedSample {
            value,
            is_structural_zero: false,
        }
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
    pub fn config(&self) -> &ZeroInflatedConfig {
        &self.config
    }
}

/// Result of sampling with structural zero information.
#[derive(Debug, Clone)]
pub struct ZeroInflatedSample {
    /// The sampled value
    pub value: f64,
    /// Whether this is a structural zero (vs. a sampling zero)
    pub is_structural_zero: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_inflated_validation() {
        let config = ZeroInflatedConfig::lognormal(0.7, 6.0, 1.5);
        assert!(config.validate().is_ok());

        let invalid_prob = ZeroInflatedConfig::lognormal(1.5, 6.0, 1.5);
        assert!(invalid_prob.validate().is_err());

        let invalid_sigma = ZeroInflatedConfig::lognormal(0.7, 6.0, -1.0);
        assert!(invalid_sigma.validate().is_err());
    }

    #[test]
    fn test_zero_inflated_sampling() {
        let config = ZeroInflatedConfig::lognormal(0.7, 6.0, 1.5);
        let mut sampler = ZeroInflatedSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be non-negative
        assert!(samples.iter().all(|&x| x >= 0.0));

        // Count zeros - should be approximately 70%
        let zero_count = samples.iter().filter(|&&x| x == 0.0).count();
        assert!(zero_count > 600 && zero_count < 800);
    }

    #[test]
    fn test_zero_inflated_determinism() {
        let config = ZeroInflatedConfig::lognormal(0.7, 6.0, 1.5);

        let mut sampler1 = ZeroInflatedSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = ZeroInflatedSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_zero_inflated_exponential() {
        let config = ZeroInflatedConfig::exponential(0.5, 0.1);
        let mut sampler = ZeroInflatedSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);

        // Count zeros - should be approximately 50%
        let zero_count = samples.iter().filter(|&&x| x == 0.0).count();
        assert!(zero_count > 400 && zero_count < 600);

        // Non-zero values should be positive
        assert!(samples.iter().filter(|&&x| x > 0.0).all(|&x| x >= 0.01));
    }

    #[test]
    fn test_zero_inflated_poisson() {
        let config = ZeroInflatedConfig::poisson(0.6, 3.0);
        let mut sampler = ZeroInflatedSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);

        // Count zeros - should be approximately 60%
        let zero_count = samples.iter().filter(|&&x| x == 0.0).count();
        assert!(zero_count > 500 && zero_count < 700);

        // Non-zero values should be integers (rounded)
        for s in samples.iter().filter(|&&x| x > 0.0) {
            assert!((s - s.round()).abs() < 0.001);
        }
    }

    #[test]
    fn test_sample_with_info() {
        let config = ZeroInflatedConfig::lognormal(0.5, 6.0, 1.5);
        let mut sampler = ZeroInflatedSampler::new(42, config).unwrap();

        let mut structural_zeros = 0;
        let mut non_zeros = 0;

        for _ in 0..1000 {
            let result = sampler.sample_with_info();
            if result.is_structural_zero {
                assert_eq!(result.value, 0.0);
                structural_zeros += 1;
            } else {
                non_zeros += 1;
            }
        }

        // Should be approximately 50/50
        assert!(structural_zeros > 400 && structural_zeros < 600);
        assert!(non_zeros > 400 && non_zeros < 600);
    }

    #[test]
    fn test_credit_memos_preset() {
        let config = ZeroInflatedConfig::credit_memos();
        assert!(config.validate().is_ok());

        let mut sampler = ZeroInflatedSampler::new(42, config.clone()).unwrap();
        let samples = sampler.sample_n(1000);

        // High zero rate (~85%)
        let zero_count = samples.iter().filter(|&&x| x == 0.0).count();
        assert!(zero_count > 750);

        // Non-zero values should be >= min_value
        assert!(samples
            .iter()
            .filter(|&&x| x > 0.0)
            .all(|&x| x >= config.min_value));
    }

    #[test]
    fn test_expected_value() {
        let config = ZeroInflatedConfig::lognormal(0.5, 6.0, 1.5);
        let expected = config.expected_value();

        // E[X] = (1 - p) * exp(mu + sigma^2/2)
        // = 0.5 * exp(6 + 1.125) = 0.5 * exp(7.125) ≈ 620
        assert!(expected > 500.0 && expected < 800.0);
    }

    #[test]
    fn test_max_value_constraint() {
        let mut config = ZeroInflatedConfig::lognormal(0.3, 8.0, 2.0);
        config.max_value = Some(1000.0);

        let mut sampler = ZeroInflatedSampler::new(42, config).unwrap();
        let samples = sampler.sample_n(1000);

        // All samples should be <= max_value
        assert!(samples.iter().all(|&x| x <= 1000.0));
    }
}
