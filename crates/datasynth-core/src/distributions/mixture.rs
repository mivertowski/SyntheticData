//! Mixture model distributions for multi-modal data generation.
//!
//! Provides Gaussian and Log-Normal mixture models that can generate
//! realistic multi-modal distributions commonly observed in accounting data
//! (e.g., routine vs. significant vs. major transactions).

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal, Normal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configuration for a single Gaussian component in a mixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianComponent {
    /// Weight of this component (0.0-1.0, all weights should sum to 1.0)
    pub weight: f64,
    /// Mean (mu) of the Gaussian distribution
    pub mu: f64,
    /// Standard deviation (sigma) of the Gaussian distribution
    pub sigma: f64,
    /// Optional label for this component (e.g., "routine", "significant")
    #[serde(default)]
    pub label: Option<String>,
}

impl GaussianComponent {
    /// Create a new Gaussian component.
    pub fn new(weight: f64, mu: f64, sigma: f64) -> Self {
        Self {
            weight,
            mu,
            sigma,
            label: None,
        }
    }

    /// Create a labeled Gaussian component.
    pub fn with_label(weight: f64, mu: f64, sigma: f64, label: impl Into<String>) -> Self {
        Self {
            weight,
            mu,
            sigma,
            label: Some(label.into()),
        }
    }
}

/// Configuration for a Gaussian Mixture Model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianMixtureConfig {
    /// Components of the mixture
    pub components: Vec<GaussianComponent>,
    /// Whether to allow negative values (default: true)
    #[serde(default = "default_true")]
    pub allow_negative: bool,
    /// Minimum value (if not allowing negative)
    #[serde(default)]
    pub min_value: Option<f64>,
    /// Maximum value (clamps output)
    #[serde(default)]
    pub max_value: Option<f64>,
}

fn default_true() -> bool {
    true
}

impl Default for GaussianMixtureConfig {
    fn default() -> Self {
        Self {
            components: vec![GaussianComponent::new(1.0, 0.0, 1.0)],
            allow_negative: true,
            min_value: None,
            max_value: None,
        }
    }
}

impl GaussianMixtureConfig {
    /// Create a new Gaussian mixture configuration.
    pub fn new(components: Vec<GaussianComponent>) -> Self {
        Self {
            components,
            ..Default::default()
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.components.is_empty() {
            return Err("At least one component is required".to_string());
        }

        let weight_sum: f64 = self.components.iter().map(|c| c.weight).sum();
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Component weights must sum to 1.0, got {weight_sum}"
            ));
        }

        for (i, component) in self.components.iter().enumerate() {
            if component.weight < 0.0 || component.weight > 1.0 {
                return Err(format!(
                    "Component {} weight must be between 0.0 and 1.0, got {}",
                    i, component.weight
                ));
            }
            if component.sigma <= 0.0 {
                return Err(format!(
                    "Component {} sigma must be positive, got {}",
                    i, component.sigma
                ));
            }
        }

        Ok(())
    }
}

/// Configuration for a single Log-Normal component in a mixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNormalComponent {
    /// Weight of this component (0.0-1.0, all weights should sum to 1.0)
    pub weight: f64,
    /// Mu parameter (location) of the log-normal distribution
    pub mu: f64,
    /// Sigma parameter (scale) of the log-normal distribution
    pub sigma: f64,
    /// Optional label for this component
    #[serde(default)]
    pub label: Option<String>,
}

impl LogNormalComponent {
    /// Create a new Log-Normal component.
    pub fn new(weight: f64, mu: f64, sigma: f64) -> Self {
        Self {
            weight,
            mu,
            sigma,
            label: None,
        }
    }

    /// Create a labeled Log-Normal component.
    pub fn with_label(weight: f64, mu: f64, sigma: f64, label: impl Into<String>) -> Self {
        Self {
            weight,
            mu,
            sigma,
            label: Some(label.into()),
        }
    }

    /// Get the expected value (mean) of this component.
    pub fn expected_value(&self) -> f64 {
        (self.mu + self.sigma.powi(2) / 2.0).exp()
    }

    /// Get the median of this component.
    pub fn median(&self) -> f64 {
        self.mu.exp()
    }
}

/// Configuration for a Log-Normal Mixture Model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNormalMixtureConfig {
    /// Components of the mixture
    pub components: Vec<LogNormalComponent>,
    /// Minimum value (default: 0.01)
    #[serde(default = "default_min_value")]
    pub min_value: f64,
    /// Maximum value (clamps output)
    #[serde(default)]
    pub max_value: Option<f64>,
    /// Number of decimal places for rounding
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_min_value() -> f64 {
    0.01
}

fn default_decimal_places() -> u8 {
    2
}

impl Default for LogNormalMixtureConfig {
    fn default() -> Self {
        Self {
            components: vec![LogNormalComponent::new(1.0, 7.0, 2.0)],
            min_value: 0.01,
            max_value: None,
            decimal_places: 2,
        }
    }
}

impl LogNormalMixtureConfig {
    /// Create a new Log-Normal mixture configuration.
    pub fn new(components: Vec<LogNormalComponent>) -> Self {
        Self {
            components,
            ..Default::default()
        }
    }

    /// Create a typical transaction amount mixture (routine/significant/major).
    pub fn typical_transactions() -> Self {
        Self {
            components: vec![
                LogNormalComponent::with_label(0.60, 6.0, 1.5, "routine"),
                LogNormalComponent::with_label(0.30, 8.5, 1.0, "significant"),
                LogNormalComponent::with_label(0.10, 11.0, 0.8, "major"),
            ],
            min_value: 0.01,
            max_value: Some(100_000_000.0),
            decimal_places: 2,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.components.is_empty() {
            return Err("At least one component is required".to_string());
        }

        let weight_sum: f64 = self.components.iter().map(|c| c.weight).sum();
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Component weights must sum to 1.0, got {weight_sum}"
            ));
        }

        for (i, component) in self.components.iter().enumerate() {
            if component.weight < 0.0 || component.weight > 1.0 {
                return Err(format!(
                    "Component {} weight must be between 0.0 and 1.0, got {}",
                    i, component.weight
                ));
            }
            if component.sigma <= 0.0 {
                return Err(format!(
                    "Component {} sigma must be positive, got {}",
                    i, component.sigma
                ));
            }
        }

        if self.min_value < 0.0 {
            return Err("min_value must be non-negative".to_string());
        }

        Ok(())
    }
}

/// Result of sampling with component information.
#[derive(Debug, Clone)]
pub struct SampleWithComponent {
    /// The sampled value
    pub value: f64,
    /// Index of the component that generated this sample
    pub component_index: usize,
    /// Label of the component (if available)
    pub component_label: Option<String>,
}

/// Gaussian Mixture Model sampler.
pub struct GaussianMixtureSampler {
    rng: ChaCha8Rng,
    config: GaussianMixtureConfig,
    /// Pre-computed cumulative weights for O(log n) component selection
    cumulative_weights: Vec<f64>,
    /// Normal distributions for each component
    distributions: Vec<Normal<f64>>,
}

impl GaussianMixtureSampler {
    /// Create a new Gaussian mixture sampler.
    pub fn new(seed: u64, config: GaussianMixtureConfig) -> Result<Self, String> {
        config.validate()?;

        // Pre-compute cumulative weights
        let mut cumulative_weights = Vec::with_capacity(config.components.len());
        let mut cumulative = 0.0;
        for component in &config.components {
            cumulative += component.weight;
            cumulative_weights.push(cumulative);
        }

        // Create distributions
        let distributions: Result<Vec<_>, _> = config
            .components
            .iter()
            .map(|c| {
                Normal::new(c.mu, c.sigma).map_err(|e| format!("Invalid normal distribution: {e}"))
            })
            .collect();

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            cumulative_weights,
            distributions: distributions?,
        })
    }

    /// Select a component using binary search on cumulative weights.
    fn select_component(&mut self) -> usize {
        let p: f64 = self.rng.random();
        match self.cumulative_weights.binary_search_by(|w| {
            w.partial_cmp(&p).unwrap_or_else(|| {
                tracing::debug!("NaN detected in mixture weight comparison");
                std::cmp::Ordering::Less
            })
        }) {
            Ok(i) => i,
            Err(i) => i.min(self.distributions.len() - 1),
        }
    }

    /// Sample a value from the mixture.
    pub fn sample(&mut self) -> f64 {
        let component_idx = self.select_component();
        let mut value = self.distributions[component_idx].sample(&mut self.rng);

        // Apply constraints
        if !self.config.allow_negative {
            value = value.abs();
        }
        if let Some(min) = self.config.min_value {
            value = value.max(min);
        }
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        value
    }

    /// Sample a value with component information.
    pub fn sample_with_component(&mut self) -> SampleWithComponent {
        let component_idx = self.select_component();
        let mut value = self.distributions[component_idx].sample(&mut self.rng);

        // Apply constraints
        if !self.config.allow_negative {
            value = value.abs();
        }
        if let Some(min) = self.config.min_value {
            value = value.max(min);
        }
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        SampleWithComponent {
            value,
            component_index: component_idx,
            component_label: self.config.components[component_idx].label.clone(),
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
    pub fn config(&self) -> &GaussianMixtureConfig {
        &self.config
    }
}

/// Log-Normal Mixture Model sampler for positive-only distributions.
pub struct LogNormalMixtureSampler {
    rng: ChaCha8Rng,
    config: LogNormalMixtureConfig,
    /// Pre-computed cumulative weights for O(log n) component selection
    cumulative_weights: Vec<f64>,
    /// Log-normal distributions for each component
    distributions: Vec<LogNormal<f64>>,
    /// Decimal multiplier for rounding
    decimal_multiplier: f64,
}

impl LogNormalMixtureSampler {
    /// Create a new Log-Normal mixture sampler.
    pub fn new(seed: u64, config: LogNormalMixtureConfig) -> Result<Self, String> {
        config.validate()?;

        // Pre-compute cumulative weights
        let mut cumulative_weights = Vec::with_capacity(config.components.len());
        let mut cumulative = 0.0;
        for component in &config.components {
            cumulative += component.weight;
            cumulative_weights.push(cumulative);
        }

        // Create distributions
        let distributions: Result<Vec<_>, _> = config
            .components
            .iter()
            .map(|c| {
                LogNormal::new(c.mu, c.sigma)
                    .map_err(|e| format!("Invalid log-normal distribution: {e}"))
            })
            .collect();

        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            cumulative_weights,
            distributions: distributions?,
            decimal_multiplier,
        })
    }

    /// Select a component using binary search on cumulative weights.
    fn select_component(&mut self) -> usize {
        let p: f64 = self.rng.random();
        match self.cumulative_weights.binary_search_by(|w| {
            w.partial_cmp(&p).unwrap_or_else(|| {
                tracing::debug!("NaN detected in mixture weight comparison");
                std::cmp::Ordering::Less
            })
        }) {
            Ok(i) => i,
            Err(i) => i.min(self.distributions.len() - 1),
        }
    }

    /// Sample a value from the mixture.
    pub fn sample(&mut self) -> f64 {
        let component_idx = self.select_component();
        let mut value = self.distributions[component_idx].sample(&mut self.rng);

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
        Decimal::from_f64_retain(value).unwrap_or(Decimal::ONE)
    }

    /// Sample a value with component information.
    pub fn sample_with_component(&mut self) -> SampleWithComponent {
        let component_idx = self.select_component();
        let mut value = self.distributions[component_idx].sample(&mut self.rng);

        // Apply constraints
        value = value.max(self.config.min_value);
        if let Some(max) = self.config.max_value {
            value = value.min(max);
        }

        // Round to decimal places
        value = (value * self.decimal_multiplier).round() / self.decimal_multiplier;

        SampleWithComponent {
            value,
            component_index: component_idx,
            component_label: self.config.components[component_idx].label.clone(),
        }
    }

    /// Sample multiple values.
    pub fn sample_n(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Sample multiple values as Decimals.
    pub fn sample_n_decimal(&mut self, n: usize) -> Vec<Decimal> {
        (0..n).map(|_| self.sample_decimal()).collect()
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Get the configuration.
    pub fn config(&self) -> &LogNormalMixtureConfig {
        &self.config
    }

    /// Get the expected value of the mixture.
    pub fn expected_value(&self) -> f64 {
        self.config
            .components
            .iter()
            .map(|c| c.weight * c.expected_value())
            .sum()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_mixture_validation() {
        // Valid config
        let config = GaussianMixtureConfig::new(vec![
            GaussianComponent::new(0.5, 0.0, 1.0),
            GaussianComponent::new(0.5, 5.0, 2.0),
        ]);
        assert!(config.validate().is_ok());

        // Invalid: weights don't sum to 1.0
        let invalid_config = GaussianMixtureConfig::new(vec![
            GaussianComponent::new(0.3, 0.0, 1.0),
            GaussianComponent::new(0.3, 5.0, 2.0),
        ]);
        assert!(invalid_config.validate().is_err());

        // Invalid: negative sigma
        let invalid_config =
            GaussianMixtureConfig::new(vec![GaussianComponent::new(1.0, 0.0, -1.0)]);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_gaussian_mixture_sampling() {
        let config = GaussianMixtureConfig::new(vec![
            GaussianComponent::new(0.5, 0.0, 1.0),
            GaussianComponent::new(0.5, 10.0, 1.0),
        ]);
        let mut sampler = GaussianMixtureSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // Check that samples are distributed around both means
        let low_count = samples.iter().filter(|&&x| x < 5.0).count();
        let high_count = samples.iter().filter(|&&x| x >= 5.0).count();

        // Both should be roughly 50% (with some tolerance)
        assert!(low_count > 350 && low_count < 650);
        assert!(high_count > 350 && high_count < 650);
    }

    #[test]
    fn test_gaussian_mixture_determinism() {
        let config = GaussianMixtureConfig::new(vec![
            GaussianComponent::new(0.5, 0.0, 1.0),
            GaussianComponent::new(0.5, 10.0, 1.0),
        ]);

        let mut sampler1 = GaussianMixtureSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = GaussianMixtureSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_lognormal_mixture_validation() {
        // Valid config
        let config = LogNormalMixtureConfig::new(vec![
            LogNormalComponent::new(0.6, 6.0, 1.5),
            LogNormalComponent::new(0.4, 8.5, 1.0),
        ]);
        assert!(config.validate().is_ok());

        // Invalid: weights don't sum to 1.0
        let invalid_config = LogNormalMixtureConfig::new(vec![
            LogNormalComponent::new(0.2, 6.0, 1.5),
            LogNormalComponent::new(0.2, 8.5, 1.0),
        ]);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_lognormal_mixture_sampling() {
        let config = LogNormalMixtureConfig::typical_transactions();
        let mut sampler = LogNormalMixtureSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be positive
        assert!(samples.iter().all(|&x| x > 0.0));

        // Check minimum value constraint
        assert!(samples.iter().all(|&x| x >= 0.01));
    }

    #[test]
    fn test_sample_with_component() {
        let config = LogNormalMixtureConfig::new(vec![
            LogNormalComponent::with_label(0.5, 6.0, 1.0, "small"),
            LogNormalComponent::with_label(0.5, 10.0, 0.5, "large"),
        ]);
        let mut sampler = LogNormalMixtureSampler::new(42, config).unwrap();

        let mut small_count = 0;
        let mut large_count = 0;

        for _ in 0..1000 {
            let result = sampler.sample_with_component();
            match result.component_label.as_deref() {
                Some("small") => small_count += 1,
                Some("large") => large_count += 1,
                _ => panic!("Unexpected label"),
            }
        }

        // Both components should be selected roughly equally
        assert!(small_count > 400 && small_count < 600);
        assert!(large_count > 400 && large_count < 600);
    }

    #[test]
    fn test_lognormal_mixture_determinism() {
        let config = LogNormalMixtureConfig::typical_transactions();

        let mut sampler1 = LogNormalMixtureSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = LogNormalMixtureSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_lognormal_expected_value() {
        let config = LogNormalMixtureConfig::new(vec![LogNormalComponent::new(1.0, 7.0, 1.0)]);
        let sampler = LogNormalMixtureSampler::new(42, config).unwrap();

        // E[X] = exp(mu + sigma^2/2) = exp(7 + 0.5) = exp(7.5) ≈ 1808
        let expected = sampler.expected_value();
        assert!((expected - 1808.04).abs() < 1.0);
    }

    #[test]
    fn test_component_label() {
        let component = LogNormalComponent::with_label(0.5, 7.0, 1.0, "test_label");
        assert_eq!(component.label, Some("test_label".to_string()));

        let component_no_label = LogNormalComponent::new(0.5, 7.0, 1.0);
        assert_eq!(component_no_label.label, None);
    }

    #[test]
    fn test_max_value_constraint() {
        let mut config = LogNormalMixtureConfig::new(vec![LogNormalComponent::new(1.0, 10.0, 1.0)]);
        config.max_value = Some(1000.0);

        let mut sampler = LogNormalMixtureSampler::new(42, config).unwrap();
        let samples = sampler.sample_n(1000);

        // All samples should be at most 1000
        assert!(samples.iter().all(|&x| x <= 1000.0));
    }
}
