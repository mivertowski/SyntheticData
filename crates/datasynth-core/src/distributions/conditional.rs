//! Conditional distributions for dependent value generation.
//!
//! This module provides tools for generating values that depend on
//! other values through breakpoint-based conditional logic, such as:
//! - Discount percentage depends on order amount
//! - Processing time depends on transaction complexity
//! - Approval level depends on amount thresholds

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution, LogNormal, Normal, Uniform};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A breakpoint defining where distribution parameters change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// The threshold value where this breakpoint applies
    pub threshold: f64,
    /// Distribution parameters for values at or above this threshold
    pub distribution: ConditionalDistributionParams,
}

/// Parameters for the conditional distribution at a given breakpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ConditionalDistributionParams {
    /// Fixed value
    Fixed { value: f64 },
    /// Normal distribution
    Normal { mu: f64, sigma: f64 },
    /// Log-normal distribution
    LogNormal { mu: f64, sigma: f64 },
    /// Uniform distribution
    Uniform { min: f64, max: f64 },
    /// Beta distribution (scaled to min-max range)
    Beta {
        alpha: f64,
        beta: f64,
        min: f64,
        max: f64,
    },
    /// Discrete choice from a set of values
    Discrete { values: Vec<f64>, weights: Vec<f64> },
}

impl Default for ConditionalDistributionParams {
    fn default() -> Self {
        Self::Fixed { value: 0.0 }
    }
}

impl ConditionalDistributionParams {
    /// Sample from this distribution.
    pub fn sample(&self, rng: &mut ChaCha8Rng) -> f64 {
        match self {
            Self::Fixed { value } => *value,
            Self::Normal { mu, sigma } => {
                let dist =
                    Normal::new(*mu, *sigma).unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
                dist.sample(rng)
            }
            Self::LogNormal { mu, sigma } => {
                let dist = LogNormal::new(*mu, *sigma)
                    .unwrap_or_else(|_| LogNormal::new(0.0, 1.0).unwrap());
                dist.sample(rng)
            }
            Self::Uniform { min, max } => {
                let dist = Uniform::new(*min, *max);
                dist.sample(rng)
            }
            Self::Beta {
                alpha,
                beta,
                min,
                max,
            } => {
                let dist =
                    Beta::new(*alpha, *beta).unwrap_or_else(|_| Beta::new(2.0, 2.0).unwrap());
                let u = dist.sample(rng);
                min + u * (max - min)
            }
            Self::Discrete { values, weights } => {
                if values.is_empty() {
                    return 0.0;
                }
                if weights.is_empty() || weights.len() != values.len() {
                    // Equal weights
                    return *values.choose(rng).unwrap_or(&0.0);
                }
                // Weighted selection
                let total: f64 = weights.iter().sum();
                let mut p: f64 = rng.gen::<f64>() * total;
                for (i, w) in weights.iter().enumerate() {
                    p -= w;
                    if p <= 0.0 {
                        return values[i];
                    }
                }
                *values.last().unwrap_or(&0.0)
            }
        }
    }
}

/// Configuration for a conditional distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalDistributionConfig {
    /// Name of the dependent field (the output)
    pub output_field: String,
    /// Name of the conditioning field (the input)
    pub input_field: String,
    /// Breakpoints defining the conditional distribution
    /// Must be sorted by threshold in ascending order
    pub breakpoints: Vec<Breakpoint>,
    /// Distribution for values below the first breakpoint
    pub default_distribution: ConditionalDistributionParams,
    /// Minimum output value (clamps)
    #[serde(default)]
    pub min_value: Option<f64>,
    /// Maximum output value (clamps)
    #[serde(default)]
    pub max_value: Option<f64>,
    /// Number of decimal places for rounding
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_decimal_places() -> u8 {
    2
}

impl Default for ConditionalDistributionConfig {
    fn default() -> Self {
        Self {
            output_field: "output".to_string(),
            input_field: "input".to_string(),
            breakpoints: vec![],
            default_distribution: ConditionalDistributionParams::Fixed { value: 0.0 },
            min_value: None,
            max_value: None,
            decimal_places: 2,
        }
    }
}

impl ConditionalDistributionConfig {
    /// Create a new conditional distribution configuration.
    pub fn new(
        output_field: impl Into<String>,
        input_field: impl Into<String>,
        breakpoints: Vec<Breakpoint>,
        default: ConditionalDistributionParams,
    ) -> Self {
        Self {
            output_field: output_field.into(),
            input_field: input_field.into(),
            breakpoints,
            default_distribution: default,
            min_value: None,
            max_value: None,
            decimal_places: 2,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        // Check breakpoints are in ascending order
        for i in 1..self.breakpoints.len() {
            if self.breakpoints[i].threshold <= self.breakpoints[i - 1].threshold {
                return Err(format!(
                    "Breakpoints must be in ascending order: {} is not > {}",
                    self.breakpoints[i].threshold,
                    self.breakpoints[i - 1].threshold
                ));
            }
        }

        if let (Some(min), Some(max)) = (self.min_value, self.max_value) {
            if max <= min {
                return Err("max_value must be greater than min_value".to_string());
            }
        }

        Ok(())
    }

    /// Get the distribution parameters for a given input value.
    pub fn get_distribution(&self, input_value: f64) -> &ConditionalDistributionParams {
        // Find the highest breakpoint that the input exceeds
        for breakpoint in self.breakpoints.iter().rev() {
            if input_value >= breakpoint.threshold {
                return &breakpoint.distribution;
            }
        }
        &self.default_distribution
    }
}

/// Sampler for conditional distributions.
pub struct ConditionalSampler {
    rng: ChaCha8Rng,
    config: ConditionalDistributionConfig,
    decimal_multiplier: f64,
}

impl ConditionalSampler {
    /// Create a new conditional sampler.
    pub fn new(seed: u64, config: ConditionalDistributionConfig) -> Result<Self, String> {
        config.validate()?;
        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);
        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            decimal_multiplier,
        })
    }

    /// Sample a value given the conditioning input.
    pub fn sample(&mut self, input_value: f64) -> f64 {
        let dist = self.config.get_distribution(input_value);
        let mut value = dist.sample(&mut self.rng);

        // Apply constraints
        if let Some(min) = self.config.min_value {
            value = value.max(min);
        }
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        // Round to decimal places
        (value * self.decimal_multiplier).round() / self.decimal_multiplier
    }

    /// Sample a value as Decimal.
    pub fn sample_decimal(&mut self, input_value: f64) -> Decimal {
        let value = self.sample(input_value);
        Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Get the configuration.
    pub fn config(&self) -> &ConditionalDistributionConfig {
        &self.config
    }
}

/// Preset conditional distribution configurations.
pub mod conditional_presets {
    use super::*;

    /// Discount percentage based on order amount.
    /// Higher amounts get higher discount percentages.
    pub fn discount_by_amount() -> ConditionalDistributionConfig {
        ConditionalDistributionConfig {
            output_field: "discount_percent".to_string(),
            input_field: "order_amount".to_string(),
            breakpoints: vec![
                Breakpoint {
                    threshold: 1000.0,
                    distribution: ConditionalDistributionParams::Beta {
                        alpha: 2.0,
                        beta: 8.0,
                        min: 0.01,
                        max: 0.05, // 1-5%
                    },
                },
                Breakpoint {
                    threshold: 5000.0,
                    distribution: ConditionalDistributionParams::Beta {
                        alpha: 2.0,
                        beta: 5.0,
                        min: 0.02,
                        max: 0.08, // 2-8%
                    },
                },
                Breakpoint {
                    threshold: 25000.0,
                    distribution: ConditionalDistributionParams::Beta {
                        alpha: 3.0,
                        beta: 3.0,
                        min: 0.05,
                        max: 0.12, // 5-12%
                    },
                },
                Breakpoint {
                    threshold: 100000.0,
                    distribution: ConditionalDistributionParams::Beta {
                        alpha: 5.0,
                        beta: 2.0,
                        min: 0.08,
                        max: 0.15, // 8-15%
                    },
                },
            ],
            default_distribution: ConditionalDistributionParams::Fixed { value: 0.0 },
            min_value: Some(0.0),
            max_value: Some(0.20),
            decimal_places: 4,
        }
    }

    /// Approval level based on transaction amount.
    pub fn approval_level_by_amount() -> ConditionalDistributionConfig {
        ConditionalDistributionConfig {
            output_field: "approval_level".to_string(),
            input_field: "amount".to_string(),
            breakpoints: vec![
                Breakpoint {
                    threshold: 1000.0,
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![1.0, 2.0],
                        weights: vec![0.9, 0.1],
                    },
                },
                Breakpoint {
                    threshold: 10000.0,
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![2.0, 3.0],
                        weights: vec![0.7, 0.3],
                    },
                },
                Breakpoint {
                    threshold: 50000.0,
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![3.0, 4.0],
                        weights: vec![0.6, 0.4],
                    },
                },
                Breakpoint {
                    threshold: 100000.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 4.0 },
                },
            ],
            default_distribution: ConditionalDistributionParams::Fixed { value: 1.0 },
            min_value: Some(1.0),
            max_value: Some(4.0),
            decimal_places: 0,
        }
    }

    /// Processing days based on order complexity (number of line items).
    pub fn processing_time_by_complexity() -> ConditionalDistributionConfig {
        ConditionalDistributionConfig {
            output_field: "processing_days".to_string(),
            input_field: "line_item_count".to_string(),
            breakpoints: vec![
                Breakpoint {
                    threshold: 5.0,
                    distribution: ConditionalDistributionParams::LogNormal {
                        mu: 0.5, // ~1.6 days median
                        sigma: 0.5,
                    },
                },
                Breakpoint {
                    threshold: 15.0,
                    distribution: ConditionalDistributionParams::LogNormal {
                        mu: 1.0, // ~2.7 days median
                        sigma: 0.5,
                    },
                },
                Breakpoint {
                    threshold: 30.0,
                    distribution: ConditionalDistributionParams::LogNormal {
                        mu: 1.5, // ~4.5 days median
                        sigma: 0.6,
                    },
                },
            ],
            default_distribution: ConditionalDistributionParams::LogNormal {
                mu: 0.0, // ~1 day median
                sigma: 0.4,
            },
            min_value: Some(0.5),
            max_value: Some(30.0),
            decimal_places: 1,
        }
    }

    /// Payment terms (days) based on customer credit rating.
    pub fn payment_terms_by_credit_rating() -> ConditionalDistributionConfig {
        ConditionalDistributionConfig {
            output_field: "payment_terms_days".to_string(),
            input_field: "credit_score".to_string(),
            breakpoints: vec![
                Breakpoint {
                    threshold: 300.0, // Poor credit
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![0.0, 15.0], // Due on receipt or Net 15
                        weights: vec![0.7, 0.3],
                    },
                },
                Breakpoint {
                    threshold: 500.0, // Fair credit
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![15.0, 30.0],
                        weights: vec![0.5, 0.5],
                    },
                },
                Breakpoint {
                    threshold: 650.0, // Good credit
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![30.0, 45.0, 60.0],
                        weights: vec![0.5, 0.3, 0.2],
                    },
                },
                Breakpoint {
                    threshold: 750.0, // Excellent credit
                    distribution: ConditionalDistributionParams::Discrete {
                        values: vec![30.0, 60.0, 90.0],
                        weights: vec![0.3, 0.4, 0.3],
                    },
                },
            ],
            default_distribution: ConditionalDistributionParams::Fixed { value: 0.0 }, // Due on receipt
            min_value: Some(0.0),
            max_value: Some(90.0),
            decimal_places: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conditional_config_validation() {
        let valid = ConditionalDistributionConfig::new(
            "output",
            "input",
            vec![
                Breakpoint {
                    threshold: 100.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 1.0 },
                },
                Breakpoint {
                    threshold: 200.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 2.0 },
                },
            ],
            ConditionalDistributionParams::Fixed { value: 0.0 },
        );
        assert!(valid.validate().is_ok());

        // Invalid: breakpoints not in order
        let invalid = ConditionalDistributionConfig::new(
            "output",
            "input",
            vec![
                Breakpoint {
                    threshold: 200.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 2.0 },
                },
                Breakpoint {
                    threshold: 100.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 1.0 },
                },
            ],
            ConditionalDistributionParams::Fixed { value: 0.0 },
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_conditional_sampling() {
        let config = ConditionalDistributionConfig::new(
            "output",
            "input",
            vec![
                Breakpoint {
                    threshold: 100.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 10.0 },
                },
                Breakpoint {
                    threshold: 200.0,
                    distribution: ConditionalDistributionParams::Fixed { value: 20.0 },
                },
            ],
            ConditionalDistributionParams::Fixed { value: 0.0 },
        );
        let mut sampler = ConditionalSampler::new(42, config).unwrap();

        // Below first threshold
        assert_eq!(sampler.sample(50.0), 0.0);

        // Between first and second threshold
        assert_eq!(sampler.sample(150.0), 10.0);

        // Above second threshold
        assert_eq!(sampler.sample(250.0), 20.0);
    }

    #[test]
    fn test_discount_by_amount_preset() {
        let config = conditional_presets::discount_by_amount();
        assert!(config.validate().is_ok());

        let mut sampler = ConditionalSampler::new(42, config).unwrap();

        // Small orders: no discount or very small
        let small_discounts: Vec<f64> = (0..100).map(|_| sampler.sample(500.0)).collect();
        let avg_small: f64 = small_discounts.iter().sum::<f64>() / 100.0;
        assert!(avg_small < 0.01); // Should be 0 or very small

        // Medium orders: small discount
        sampler.reset(42);
        let medium_discounts: Vec<f64> = (0..100).map(|_| sampler.sample(3000.0)).collect();
        let avg_medium: f64 = medium_discounts.iter().sum::<f64>() / 100.0;
        assert!(avg_medium > 0.01 && avg_medium < 0.06);

        // Large orders: higher discount
        sampler.reset(42);
        let large_discounts: Vec<f64> = (0..100).map(|_| sampler.sample(150000.0)).collect();
        let avg_large: f64 = large_discounts.iter().sum::<f64>() / 100.0;
        assert!(avg_large > 0.08);
    }

    #[test]
    fn test_approval_level_preset() {
        let config = conditional_presets::approval_level_by_amount();
        assert!(config.validate().is_ok());

        let mut sampler = ConditionalSampler::new(42, config).unwrap();

        // Small amounts: level 1
        let level = sampler.sample(500.0);
        assert_eq!(level, 1.0);

        // Large amounts: level 3-4
        sampler.reset(42);
        let levels: Vec<f64> = (0..100).map(|_| sampler.sample(75000.0)).collect();
        let avg_level: f64 = levels.iter().sum::<f64>() / 100.0;
        assert!(avg_level >= 3.0);
    }

    #[test]
    fn test_distribution_params_sampling() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Test Normal
        let normal = ConditionalDistributionParams::Normal {
            mu: 10.0,
            sigma: 1.0,
        };
        let samples: Vec<f64> = (0..1000).map(|_| normal.sample(&mut rng)).collect();
        let mean: f64 = samples.iter().sum::<f64>() / 1000.0;
        assert!((mean - 10.0).abs() < 0.5);

        // Test Beta
        let beta = ConditionalDistributionParams::Beta {
            alpha: 2.0,
            beta: 5.0,
            min: 0.0,
            max: 1.0,
        };
        let samples: Vec<f64> = (0..1000).map(|_| beta.sample(&mut rng)).collect();
        assert!(samples.iter().all(|&x| (0.0..=1.0).contains(&x)));

        // Test Discrete
        let discrete = ConditionalDistributionParams::Discrete {
            values: vec![1.0, 2.0, 3.0],
            weights: vec![0.5, 0.3, 0.2],
        };
        let samples: Vec<f64> = (0..1000).map(|_| discrete.sample(&mut rng)).collect();
        let count_1 = samples.iter().filter(|&&x| x == 1.0).count();
        assert!(count_1 > 400 && count_1 < 600); // ~50%
    }

    #[test]
    fn test_conditional_determinism() {
        let config = conditional_presets::discount_by_amount();

        let mut sampler1 = ConditionalSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = ConditionalSampler::new(42, config).unwrap();

        for amount in [100.0, 1000.0, 10000.0, 100000.0] {
            assert_eq!(sampler1.sample(amount), sampler2.sample(amount));
        }
    }
}
