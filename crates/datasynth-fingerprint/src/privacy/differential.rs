//! Differential privacy mechanisms.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Laplace mechanism for differential privacy.
pub struct LaplaceMechanism {
    /// Global epsilon budget.
    epsilon: f64,
    /// RNG for noise generation.
    rng: ChaCha8Rng,
}

impl LaplaceMechanism {
    /// Create a new Laplace mechanism.
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon,
            rng: ChaCha8Rng::from_entropy(),
        }
    }

    /// Create with a specific seed.
    pub fn with_seed(epsilon: f64, seed: u64) -> Self {
        Self {
            epsilon,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Add Laplace noise to a value.
    ///
    /// # Arguments
    /// * `value` - The true value
    /// * `sensitivity` - The sensitivity (maximum change from adding/removing one record)
    /// * `epsilon` - The privacy budget for this query
    pub fn add_noise(&mut self, value: f64, sensitivity: f64, epsilon: f64) -> f64 {
        let scale = sensitivity / epsilon;
        let noise = self.sample_laplace(scale);
        value + noise
    }

    /// Add noise to a count (always non-negative).
    pub fn add_noise_to_count(&mut self, count: u64, epsilon: f64) -> u64 {
        let noised = self.add_noise(count as f64, 1.0, epsilon);
        noised.max(0.0).round() as u64
    }

    /// Sample from Laplace distribution with scale b.
    fn sample_laplace(&mut self, scale: f64) -> f64 {
        // Laplace(0, b) = sign(U-0.5) * b * ln(1 - 2|U-0.5|)
        // where U ~ Uniform(0, 1)
        let u: f64 = self.rng.gen();
        let sign = if u < 0.5 { -1.0 } else { 1.0 };
        let abs_u = (u - 0.5).abs();
        sign * scale * (1.0 - 2.0 * abs_u).ln()
    }

    /// Get the global epsilon budget.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }
}

/// Gaussian mechanism for differential privacy.
pub struct GaussianMechanism {
    /// Global epsilon budget.
    #[allow(dead_code)]
    epsilon: f64,
    /// Delta parameter.
    delta: f64,
    /// RNG for noise generation.
    rng: ChaCha8Rng,
}

impl GaussianMechanism {
    /// Create a new Gaussian mechanism.
    pub fn new(epsilon: f64, delta: f64) -> Self {
        Self {
            epsilon,
            delta,
            rng: ChaCha8Rng::from_entropy(),
        }
    }

    /// Add Gaussian noise to a value.
    pub fn add_noise(&mut self, value: f64, sensitivity: f64, epsilon: f64) -> f64 {
        // σ = sensitivity * sqrt(2 * ln(1.25/δ)) / ε
        let sigma = sensitivity * (2.0 * (1.25 / self.delta).ln()).sqrt() / epsilon;
        let noise = self.sample_gaussian(sigma);
        value + noise
    }

    /// Sample from Gaussian distribution with standard deviation sigma.
    fn sample_gaussian(&mut self, sigma: f64) -> f64 {
        // Box-Muller transform
        let u1: f64 = self.rng.gen();
        let u2: f64 = self.rng.gen();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        z * sigma
    }
}

/// Compute the sensitivity of a query.
pub mod sensitivity {
    /// Sensitivity of a count query (adding/removing one record changes count by 1).
    pub const COUNT: f64 = 1.0;

    /// Sensitivity of a sum query depends on the value bounds.
    pub fn sum(min_value: f64, max_value: f64) -> f64 {
        max_value - min_value
    }

    /// Sensitivity of a mean query (bounded sensitivity).
    pub fn mean(min_value: f64, max_value: f64, n: usize) -> f64 {
        (max_value - min_value) / n as f64
    }

    /// Sensitivity of a histogram query.
    pub const HISTOGRAM: f64 = 2.0; // Removing one record affects two bins
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_laplace_noise() {
        let mut mechanism = LaplaceMechanism::with_seed(1.0, 42);

        // With same seed, should be deterministic
        let value = 100.0;
        let noised1 = mechanism.add_noise(value, 1.0, 1.0);

        let mut mechanism2 = LaplaceMechanism::with_seed(1.0, 42);
        let noised2 = mechanism2.add_noise(value, 1.0, 1.0);

        assert_eq!(noised1, noised2);
    }

    #[test]
    fn test_laplace_preserves_rough_magnitude() {
        let mut mechanism = LaplaceMechanism::with_seed(1.0, 42);

        // With reasonable epsilon, noise shouldn't be extreme
        let value = 1000.0;
        let noised = mechanism.add_noise(value, 1.0, 1.0);

        // Should be within reasonable bounds (not a guarantee, but very likely)
        assert!((noised - value).abs() < 100.0);
    }
}
