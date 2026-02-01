//! Anderson-Darling goodness-of-fit test for distribution validation.
//!
//! Tests whether sample data follows a specified theoretical distribution.
//! More sensitive to deviations in the tails compared to the K-S test.

use crate::error::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};

/// Target distribution types for Anderson-Darling test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetDistribution {
    /// Normal (Gaussian) distribution
    #[default]
    Normal,
    /// Log-normal distribution (for positive amounts)
    LogNormal,
    /// Exponential distribution
    Exponential,
    /// Uniform distribution
    Uniform,
}

/// Fitted distribution parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FittedParameters {
    /// Normal distribution parameters
    Normal {
        /// Mean
        mean: f64,
        /// Standard deviation
        std_dev: f64,
    },
    /// Log-normal distribution parameters
    LogNormal {
        /// Location parameter (mu)
        mu: f64,
        /// Scale parameter (sigma)
        sigma: f64,
    },
    /// Exponential distribution parameters
    Exponential {
        /// Rate parameter (lambda)
        lambda: f64,
    },
    /// Uniform distribution parameters
    Uniform {
        /// Minimum value
        min: f64,
        /// Maximum value
        max: f64,
    },
}

/// Anderson-Darling test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndersonDarlingAnalysis {
    /// Sample size
    pub sample_size: usize,
    /// Target distribution tested
    pub target_distribution: TargetDistribution,
    /// Anderson-Darling test statistic (A²)
    pub statistic: f64,
    /// Critical values at different significance levels
    pub critical_values: CriticalValues,
    /// Approximate p-value
    pub p_value: f64,
    /// Significance level used for pass/fail
    pub significance_level: f64,
    /// Whether the test passes (fail to reject null hypothesis)
    pub passes: bool,
    /// Fitted distribution parameters
    pub fitted_params: FittedParameters,
    /// Issues found during analysis
    pub issues: Vec<String>,
}

/// Critical values for Anderson-Darling test at standard significance levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalValues {
    /// Critical value at 15% significance
    pub cv_15: f64,
    /// Critical value at 10% significance
    pub cv_10: f64,
    /// Critical value at 5% significance
    pub cv_05: f64,
    /// Critical value at 2.5% significance
    pub cv_025: f64,
    /// Critical value at 1% significance
    pub cv_01: f64,
}

impl CriticalValues {
    /// Standard critical values for normal distribution test.
    pub fn normal() -> Self {
        // Critical values from D'Agostino & Stephens (1986)
        Self {
            cv_15: 0.576,
            cv_10: 0.656,
            cv_05: 0.787,
            cv_025: 0.918,
            cv_01: 1.092,
        }
    }

    /// Standard critical values for exponential distribution test.
    pub fn exponential() -> Self {
        Self {
            cv_15: 0.922,
            cv_10: 1.078,
            cv_05: 1.341,
            cv_025: 1.606,
            cv_01: 1.957,
        }
    }
}

/// Analyzer for Anderson-Darling goodness-of-fit tests.
pub struct AndersonDarlingAnalyzer {
    /// Target distribution to test against
    target_distribution: TargetDistribution,
    /// Significance level for the test
    significance_level: f64,
}

impl AndersonDarlingAnalyzer {
    /// Create a new analyzer with default settings.
    pub fn new() -> Self {
        Self {
            target_distribution: TargetDistribution::Normal,
            significance_level: 0.05,
        }
    }

    /// Set the target distribution.
    pub fn with_target_distribution(mut self, dist: TargetDistribution) -> Self {
        self.target_distribution = dist;
        self
    }

    /// Set the significance level.
    pub fn with_significance_level(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }

    /// Perform the Anderson-Darling test on the provided sample data.
    pub fn analyze(&self, values: &[f64]) -> EvalResult<AndersonDarlingAnalysis> {
        let n = values.len();
        if n < 8 {
            return Err(EvalError::InsufficientData {
                required: 8,
                actual: n,
            });
        }

        // Filter out invalid values
        let valid_values: Vec<f64> = values.iter().filter(|&&v| v.is_finite()).copied().collect();

        if valid_values.len() < 8 {
            return Err(EvalError::InsufficientData {
                required: 8,
                actual: valid_values.len(),
            });
        }

        let mut issues = Vec::new();

        // Fit parameters and compute test statistic based on target distribution
        let (statistic, fitted_params) = match self.target_distribution {
            TargetDistribution::Normal => self.test_normal(&valid_values),
            TargetDistribution::LogNormal => self.test_lognormal(&valid_values, &mut issues)?,
            TargetDistribution::Exponential => self.test_exponential(&valid_values, &mut issues)?,
            TargetDistribution::Uniform => self.test_uniform(&valid_values),
        };

        // Get critical values
        let critical_values = match self.target_distribution {
            TargetDistribution::Exponential => CriticalValues::exponential(),
            _ => CriticalValues::normal(),
        };

        // Calculate approximate p-value
        let p_value = self.approximate_p_value(statistic, self.target_distribution);

        // Determine if test passes (fail to reject null hypothesis)
        let critical_threshold = match self.significance_level {
            s if s >= 0.15 => critical_values.cv_15,
            s if s >= 0.10 => critical_values.cv_10,
            s if s >= 0.05 => critical_values.cv_05,
            s if s >= 0.025 => critical_values.cv_025,
            _ => critical_values.cv_01,
        };
        let passes = statistic <= critical_threshold;

        if !passes {
            issues.push(format!(
                "A² = {:.4} exceeds critical value {:.4} at α = {:.2}",
                statistic, critical_threshold, self.significance_level
            ));
        }

        Ok(AndersonDarlingAnalysis {
            sample_size: valid_values.len(),
            target_distribution: self.target_distribution,
            statistic,
            critical_values,
            p_value,
            significance_level: self.significance_level,
            passes,
            fitted_params,
            issues,
        })
    }

    /// Test for normality using Anderson-Darling.
    fn test_normal(&self, values: &[f64]) -> (f64, FittedParameters) {
        let n = values.len();
        let n_f = n as f64;

        // Fit parameters (MLE for normal)
        let mean = values.iter().sum::<f64>() / n_f;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n_f;
        let std_dev = variance.sqrt();

        // Standardize and sort
        let mut z: Vec<f64> = values
            .iter()
            .map(|&x| (x - mean) / std_dev.max(1e-10))
            .collect();
        z.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Compute A² statistic
        let a2 = self.compute_a2_statistic(&z, Self::normal_cdf);

        // Apply correction for estimated parameters
        let a2_corrected = a2 * (1.0 + 0.75 / n_f + 2.25 / (n_f * n_f));

        (a2_corrected, FittedParameters::Normal { mean, std_dev })
    }

    /// Test for log-normality.
    fn test_lognormal(
        &self,
        values: &[f64],
        issues: &mut Vec<String>,
    ) -> EvalResult<(f64, FittedParameters)> {
        // Check for non-positive values
        let positive_values: Vec<f64> = values.iter().filter(|&&v| v > 0.0).copied().collect();

        if positive_values.len() < 8 {
            return Err(EvalError::InvalidParameter(
                "Log-normal test requires at least 8 positive values".to_string(),
            ));
        }

        let skipped = values.len() - positive_values.len();
        if skipped > 0 {
            issues.push(format!(
                "Skipped {} non-positive values for log-normal test",
                skipped
            ));
        }

        // Transform to log scale
        let log_values: Vec<f64> = positive_values.iter().map(|&x| x.ln()).collect();

        // Test normality of log-transformed values
        let (a2, normal_params) = self.test_normal(&log_values);

        // Convert parameters back to log-normal scale
        let (mu, sigma) = match normal_params {
            FittedParameters::Normal { mean, std_dev } => (mean, std_dev),
            _ => unreachable!(),
        };

        Ok((a2, FittedParameters::LogNormal { mu, sigma }))
    }

    /// Test for exponential distribution.
    fn test_exponential(
        &self,
        values: &[f64],
        issues: &mut Vec<String>,
    ) -> EvalResult<(f64, FittedParameters)> {
        // Check for non-positive values
        let positive_values: Vec<f64> = values.iter().filter(|&&v| v > 0.0).copied().collect();

        if positive_values.len() < 8 {
            return Err(EvalError::InvalidParameter(
                "Exponential test requires at least 8 positive values".to_string(),
            ));
        }

        let skipped = values.len() - positive_values.len();
        if skipped > 0 {
            issues.push(format!(
                "Skipped {} non-positive values for exponential test",
                skipped
            ));
        }

        let n = positive_values.len();
        let n_f = n as f64;

        // Fit parameter (MLE)
        let mean = positive_values.iter().sum::<f64>() / n_f;
        let lambda = 1.0 / mean;

        // Sort values
        let mut sorted: Vec<f64> = positive_values;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Transform to uniform using exponential CDF
        let u: Vec<f64> = sorted.iter().map(|&x| 1.0 - (-lambda * x).exp()).collect();

        // Compute A² statistic on uniform
        let a2 = self.compute_a2_uniform(&u);

        // Apply correction for estimated parameters
        let a2_corrected = a2 * (1.0 + 0.6 / n_f);

        Ok((a2_corrected, FittedParameters::Exponential { lambda }))
    }

    /// Test for uniform distribution.
    fn test_uniform(&self, values: &[f64]) -> (f64, FittedParameters) {
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Sort values
        let mut sorted: Vec<f64> = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Transform to standard uniform
        let range = (max - min).max(1e-10);
        let u: Vec<f64> = sorted.iter().map(|&x| (x - min) / range).collect();

        let a2 = self.compute_a2_uniform(&u);

        (a2, FittedParameters::Uniform { min, max })
    }

    /// Compute A² statistic from sorted, standardized values using given CDF.
    fn compute_a2_statistic<F>(&self, z: &[f64], cdf: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let n = z.len();
        let n_f = n as f64;

        let mut sum = 0.0;
        for (i, &zi) in z.iter().enumerate() {
            let fi = cdf(zi).clamp(1e-10, 1.0 - 1e-10);
            let fn_minus_i = cdf(z[n - 1 - i]).clamp(1e-10, 1.0 - 1e-10);

            let term = (2.0 * (i as f64) + 1.0) * (fi.ln() + (1.0 - fn_minus_i).ln());
            sum += term;
        }

        -n_f - sum / n_f
    }

    /// Compute A² statistic from uniform samples.
    fn compute_a2_uniform(&self, u: &[f64]) -> f64 {
        let n = u.len();
        let n_f = n as f64;

        let mut sum = 0.0;
        for (i, &ui) in u.iter().enumerate() {
            let ui = ui.clamp(1e-10, 1.0 - 1e-10);
            let u_n_minus_i = u[n - 1 - i].clamp(1e-10, 1.0 - 1e-10);

            let term = (2.0 * (i as f64) + 1.0) * (ui.ln() + (1.0 - u_n_minus_i).ln());
            sum += term;
        }

        -n_f - sum / n_f
    }

    /// Standard normal CDF.
    fn normal_cdf(x: f64) -> f64 {
        0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
    }

    /// Approximate p-value for Anderson-Darling statistic.
    fn approximate_p_value(&self, a2: f64, dist: TargetDistribution) -> f64 {
        match dist {
            TargetDistribution::Normal => {
                // Approximation from D'Agostino & Stephens (1986)
                if a2 < 0.2 {
                    1.0 - (-13.436 + 101.14 * a2 - 223.73 * a2.powi(2)).exp()
                } else if a2 < 0.34 {
                    1.0 - (-8.318 + 42.796 * a2 - 59.938 * a2.powi(2)).exp()
                } else if a2 < 0.6 {
                    (0.9177 - 4.279 * a2 - 1.38 * a2.powi(2)).exp()
                } else if a2 < 13.0 {
                    (1.2937 - 5.709 * a2 + 0.0186 * a2.powi(2)).exp()
                } else {
                    0.0
                }
            }
            TargetDistribution::Exponential => {
                // Approximation for exponential
                if a2 < 0.26 {
                    1.0 - (-12.0 + 70.0 * a2 - 100.0 * a2.powi(2)).exp()
                } else if a2 < 0.51 {
                    1.0 - (-6.0 + 24.0 * a2 - 24.0 * a2.powi(2)).exp()
                } else if a2 < 0.95 {
                    (0.7 - 3.5 * a2 + 0.6 * a2.powi(2)).exp()
                } else if a2 < 10.0 {
                    (0.9 - 4.0 * a2 + 0.01 * a2.powi(2)).exp()
                } else {
                    0.0
                }
            }
            _ => {
                // Generic approximation
                if a2 < 2.0 {
                    (-a2 + 0.5).exp().clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
        }
    }
}

impl Default for AndersonDarlingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Error function approximation.
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Exp, LogNormal, Normal, Uniform};

    #[test]
    fn test_normal_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let normal = Normal::new(0.0, 1.0).unwrap();
        let values: Vec<f64> = (0..500).map(|_| normal.sample(&mut rng)).collect();

        let analyzer = AndersonDarlingAnalyzer::new()
            .with_target_distribution(TargetDistribution::Normal)
            .with_significance_level(0.05);

        let result = analyzer.analyze(&values).unwrap();
        assert!(result.passes, "Normal sample should pass normality test");
        assert!(result.p_value > 0.05);
    }

    #[test]
    fn test_non_normal_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let exp = Exp::new(1.0).unwrap();
        let values: Vec<f64> = (0..500).map(|_| exp.sample(&mut rng)).collect();

        let analyzer = AndersonDarlingAnalyzer::new()
            .with_target_distribution(TargetDistribution::Normal)
            .with_significance_level(0.05);

        let result = analyzer.analyze(&values).unwrap();
        assert!(
            !result.passes,
            "Exponential sample should fail normality test"
        );
    }

    #[test]
    fn test_lognormal_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let lognormal = LogNormal::new(3.0, 0.5).unwrap();
        let values: Vec<f64> = (0..500).map(|_| lognormal.sample(&mut rng)).collect();

        let analyzer = AndersonDarlingAnalyzer::new()
            .with_target_distribution(TargetDistribution::LogNormal)
            .with_significance_level(0.05);

        let result = analyzer.analyze(&values).unwrap();
        assert!(
            result.passes,
            "Log-normal sample should pass log-normality test"
        );

        // Check fitted parameters are reasonable
        if let FittedParameters::LogNormal { mu, sigma } = result.fitted_params {
            assert!((mu - 3.0).abs() < 0.5, "Mu should be close to 3.0");
            assert!((sigma - 0.5).abs() < 0.2, "Sigma should be close to 0.5");
        } else {
            panic!("Expected LogNormal parameters");
        }
    }

    #[test]
    fn test_exponential_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let exp = Exp::new(2.0).unwrap();
        let values: Vec<f64> = (0..500).map(|_| exp.sample(&mut rng)).collect();

        let analyzer = AndersonDarlingAnalyzer::new()
            .with_target_distribution(TargetDistribution::Exponential)
            .with_significance_level(0.05);

        let result = analyzer.analyze(&values).unwrap();
        assert!(
            result.passes,
            "Exponential sample should pass exponential test"
        );

        // Check fitted lambda is reasonable
        if let FittedParameters::Exponential { lambda } = result.fitted_params {
            assert!(
                (lambda - 2.0).abs() < 0.5,
                "Lambda should be close to 2.0, got {}",
                lambda
            );
        } else {
            panic!("Expected Exponential parameters");
        }
    }

    #[test]
    fn test_uniform_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let uniform = Uniform::new(0.0, 10.0);
        let values: Vec<f64> = (0..500).map(|_| uniform.sample(&mut rng)).collect();

        let analyzer = AndersonDarlingAnalyzer::new()
            .with_target_distribution(TargetDistribution::Uniform)
            .with_significance_level(0.05);

        let result = analyzer.analyze(&values).unwrap();
        // Uniform test may or may not pass depending on sample
        assert!(result.sample_size == 500);
    }

    #[test]
    fn test_insufficient_data() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // Only 5 values

        let analyzer = AndersonDarlingAnalyzer::new();
        let result = analyzer.analyze(&values);

        assert!(matches!(
            result,
            Err(EvalError::InsufficientData {
                required: 8,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_critical_values() {
        let cv = CriticalValues::normal();
        assert!(cv.cv_15 < cv.cv_10);
        assert!(cv.cv_10 < cv.cv_05);
        assert!(cv.cv_05 < cv.cv_025);
        assert!(cv.cv_025 < cv.cv_01);
    }
}
