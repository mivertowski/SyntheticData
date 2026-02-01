//! Beta distribution for modeling proportions and percentages.
//!
//! The Beta distribution is ideal for:
//! - Discount percentages
//! - Completion rates
//! - Proportion of revenue recognized
//! - Match rates and quality scores

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configuration for Beta distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaConfig {
    /// Alpha parameter (shape1) - controls skewness towards 1.
    /// Higher alpha = more mass towards 1.
    pub alpha: f64,
    /// Beta parameter (shape2) - controls skewness towards 0.
    /// Higher beta = more mass towards 0.
    pub beta: f64,
    /// Lower bound of the output range (default: 0.0).
    #[serde(default)]
    pub lower_bound: f64,
    /// Upper bound of the output range (default: 1.0).
    #[serde(default = "default_upper_bound")]
    pub upper_bound: f64,
    /// Number of decimal places for rounding.
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_upper_bound() -> f64 {
    1.0
}

fn default_decimal_places() -> u8 {
    4
}

impl Default for BetaConfig {
    fn default() -> Self {
        Self {
            alpha: 2.0,
            beta: 5.0,
            lower_bound: 0.0,
            upper_bound: 1.0,
            decimal_places: 4,
        }
    }
}

impl BetaConfig {
    /// Create a new Beta configuration.
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self {
            alpha,
            beta,
            ..Default::default()
        }
    }

    /// Create a configuration scaled to a percentage range.
    pub fn percentage(alpha: f64, beta: f64) -> Self {
        Self {
            alpha,
            beta,
            lower_bound: 0.0,
            upper_bound: 100.0,
            decimal_places: 2,
        }
    }

    /// Create a configuration for discount percentages (typically 2-15%).
    pub fn discount_rate() -> Self {
        Self {
            alpha: 2.0, // Skewed towards lower discounts
            beta: 8.0,
            lower_bound: 0.02, // 2% minimum
            upper_bound: 0.15, // 15% maximum
            decimal_places: 4,
        }
    }

    /// Create a configuration for cash discount rates (1-3%).
    pub fn cash_discount() -> Self {
        Self {
            alpha: 3.0,
            beta: 3.0,         // Symmetric around 2%
            lower_bound: 0.01, // 1% minimum
            upper_bound: 0.03, // 3% maximum
            decimal_places: 4,
        }
    }

    /// Create a configuration for completion rates (biased towards high).
    pub fn completion_rate() -> Self {
        Self {
            alpha: 8.0, // Strongly biased towards 1
            beta: 2.0,
            lower_bound: 0.0,
            upper_bound: 1.0,
            decimal_places: 4,
        }
    }

    /// Create a configuration for match rates (typically 85-99%).
    pub fn match_rate() -> Self {
        Self {
            alpha: 10.0,
            beta: 1.5,
            lower_bound: 0.85,
            upper_bound: 0.99,
            decimal_places: 4,
        }
    }

    /// Create a configuration for quality scores (0-100, slightly skewed high).
    pub fn quality_score() -> Self {
        Self {
            alpha: 5.0,
            beta: 2.0,
            lower_bound: 0.0,
            upper_bound: 100.0,
            decimal_places: 1,
        }
    }

    /// Create a uniform distribution on [0, 1].
    pub fn uniform() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            ..Default::default()
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.alpha <= 0.0 {
            return Err("alpha must be positive".to_string());
        }
        if self.beta <= 0.0 {
            return Err("beta must be positive".to_string());
        }
        if self.upper_bound <= self.lower_bound {
            return Err("upper_bound must be greater than lower_bound".to_string());
        }
        Ok(())
    }

    /// Get the expected value (mean) of the distribution.
    pub fn expected_value(&self) -> f64 {
        let raw_mean = self.alpha / (self.alpha + self.beta);
        self.lower_bound + raw_mean * (self.upper_bound - self.lower_bound)
    }

    /// Get the mode of the distribution.
    /// Only defined for alpha > 1 and beta > 1.
    pub fn mode(&self) -> Option<f64> {
        if self.alpha > 1.0 && self.beta > 1.0 {
            let raw_mode = (self.alpha - 1.0) / (self.alpha + self.beta - 2.0);
            Some(self.lower_bound + raw_mode * (self.upper_bound - self.lower_bound))
        } else {
            None
        }
    }

    /// Get the variance of the distribution.
    pub fn variance(&self) -> f64 {
        let ab = self.alpha + self.beta;
        let raw_variance = (self.alpha * self.beta) / (ab.powi(2) * (ab + 1.0));
        raw_variance * (self.upper_bound - self.lower_bound).powi(2)
    }
}

/// Beta distribution sampler.
pub struct BetaSampler {
    rng: ChaCha8Rng,
    config: BetaConfig,
    distribution: Beta<f64>,
    decimal_multiplier: f64,
    range: f64,
}

impl BetaSampler {
    /// Create a new Beta sampler.
    pub fn new(seed: u64, config: BetaConfig) -> Result<Self, String> {
        config.validate()?;

        let distribution = Beta::new(config.alpha, config.beta)
            .map_err(|e| format!("Invalid Beta distribution: {}", e))?;

        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);
        let range = config.upper_bound - config.lower_bound;

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            distribution,
            decimal_multiplier,
            range,
        })
    }

    /// Sample a value from the distribution.
    pub fn sample(&mut self) -> f64 {
        let raw_value = self.distribution.sample(&mut self.rng);
        let scaled_value = self.config.lower_bound + raw_value * self.range;

        // Round to decimal places
        (scaled_value * self.decimal_multiplier).round() / self.decimal_multiplier
    }

    /// Sample a value as Decimal.
    pub fn sample_decimal(&mut self) -> Decimal {
        let value = self.sample();
        Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO)
    }

    /// Sample a value as a percentage (multiplied by 100).
    pub fn sample_percentage(&mut self) -> f64 {
        let raw_value = self.distribution.sample(&mut self.rng);
        let scaled_value = self.config.lower_bound + raw_value * self.range;
        (scaled_value * 100.0 * self.decimal_multiplier).round() / self.decimal_multiplier
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
    pub fn config(&self) -> &BetaConfig {
        &self.config
    }
}

/// Determine the shape of the distribution based on alpha and beta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BetaShape {
    /// Uniform distribution (alpha = beta = 1)
    Uniform,
    /// U-shaped (alpha < 1 and beta < 1)
    UShaped,
    /// Unimodal symmetric (alpha = beta > 1)
    Symmetric,
    /// Unimodal skewed left (alpha > beta)
    SkewedLeft,
    /// Unimodal skewed right (alpha < beta)
    SkewedRight,
    /// J-shaped towards 1 (alpha >= 1, beta < 1)
    JShapedHigh,
    /// J-shaped towards 0 (alpha < 1, beta >= 1)
    JShapedLow,
}

impl BetaConfig {
    /// Determine the shape of this distribution.
    pub fn shape(&self) -> BetaShape {
        match (self.alpha, self.beta) {
            (a, b) if (a - 1.0).abs() < 0.001 && (b - 1.0).abs() < 0.001 => BetaShape::Uniform,
            (a, b) if a < 1.0 && b < 1.0 => BetaShape::UShaped,
            (a, b) if (a - b).abs() < 0.001 && a > 1.0 => BetaShape::Symmetric,
            (a, b) if a < 1.0 && b >= 1.0 => BetaShape::JShapedLow,
            (a, b) if a >= 1.0 && b < 1.0 => BetaShape::JShapedHigh,
            (a, b) if a > b => BetaShape::SkewedLeft,
            _ => BetaShape::SkewedRight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_validation() {
        let config = BetaConfig::new(2.0, 5.0);
        assert!(config.validate().is_ok());

        let invalid_alpha = BetaConfig::new(-1.0, 5.0);
        assert!(invalid_alpha.validate().is_err());

        let invalid_beta = BetaConfig::new(2.0, 0.0);
        assert!(invalid_beta.validate().is_err());
    }

    #[test]
    fn test_beta_sampling() {
        let config = BetaConfig::new(2.0, 5.0);
        let mut sampler = BetaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be in [0, 1]
        assert!(samples.iter().all(|&x| (0.0..=1.0).contains(&x)));
    }

    #[test]
    fn test_beta_determinism() {
        let config = BetaConfig::new(2.0, 5.0);

        let mut sampler1 = BetaSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = BetaSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_beta_scaled_range() {
        let config = BetaConfig {
            alpha: 2.0,
            beta: 2.0,
            lower_bound: 0.02,
            upper_bound: 0.15,
            decimal_places: 4,
        };
        let mut sampler = BetaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert!(samples.iter().all(|&x| (0.02..=0.15).contains(&x)));
    }

    #[test]
    fn test_beta_expected_value() {
        let config = BetaConfig::new(2.0, 5.0);
        // E[X] = alpha / (alpha + beta) = 2/7 ≈ 0.286
        let expected = config.expected_value();
        assert!((expected - 0.286).abs() < 0.01);
    }

    #[test]
    fn test_beta_mode() {
        let config = BetaConfig::new(2.0, 5.0);
        // Mode = (alpha - 1) / (alpha + beta - 2) = 1/5 = 0.2
        let mode = config.mode();
        assert!(mode.is_some());
        assert!((mode.unwrap() - 0.2).abs() < 0.001);

        // No mode for alpha <= 1
        let no_mode_config = BetaConfig::new(0.5, 5.0);
        assert!(no_mode_config.mode().is_none());
    }

    #[test]
    fn test_beta_presets() {
        let discount = BetaConfig::discount_rate();
        assert!(discount.validate().is_ok());

        let cash = BetaConfig::cash_discount();
        assert!(cash.validate().is_ok());

        let completion = BetaConfig::completion_rate();
        assert!(completion.validate().is_ok());

        let match_rate = BetaConfig::match_rate();
        assert!(match_rate.validate().is_ok());

        let quality = BetaConfig::quality_score();
        assert!(quality.validate().is_ok());
    }

    #[test]
    fn test_beta_shape_detection() {
        assert_eq!(BetaConfig::uniform().shape(), BetaShape::Uniform);
        assert_eq!(BetaConfig::new(0.5, 0.5).shape(), BetaShape::UShaped);
        assert_eq!(BetaConfig::new(5.0, 5.0).shape(), BetaShape::Symmetric);
        assert_eq!(BetaConfig::new(8.0, 2.0).shape(), BetaShape::SkewedLeft);
        assert_eq!(BetaConfig::new(2.0, 8.0).shape(), BetaShape::SkewedRight);
    }

    #[test]
    fn test_discount_rate_distribution() {
        let config = BetaConfig::discount_rate();
        let mut sampler = BetaSampler::new(42, config.clone()).unwrap();

        let samples = sampler.sample_n(1000);

        // All samples should be in [2%, 15%]
        assert!(samples.iter().all(|&x| (0.02..=0.15).contains(&x)));

        // Mean should be around the expected value
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        let expected = config.expected_value();
        assert!((mean - expected).abs() < 0.01);
    }

    #[test]
    fn test_beta_percentage_sampling() {
        let config = BetaConfig::percentage(2.0, 5.0);
        let mut sampler = BetaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert!(samples.iter().all(|&x| (0.0..=100.0).contains(&x)));
    }
}
