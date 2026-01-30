//! Chi-squared goodness-of-fit test for distribution validation.
//!
//! Tests whether observed frequency distribution matches expected frequencies.
//! Useful for categorical data and binned continuous data.

use crate::error::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};

/// Binning strategy for continuous data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinningStrategy {
    /// Fixed number of equal-width bins
    EqualWidth { num_bins: usize },
    /// Equal-frequency (quantile) bins
    EqualFrequency { num_bins: usize },
    /// Custom bin edges
    Custom { edges: Vec<f64> },
    /// Automatic binning using Sturges' rule
    Auto,
}

impl Default for BinningStrategy {
    fn default() -> Self {
        Self::Auto
    }
}

/// Bin frequency information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinFrequency {
    /// Bin index
    pub index: usize,
    /// Bin lower edge (inclusive)
    pub lower: f64,
    /// Bin upper edge (exclusive, except for last bin)
    pub upper: f64,
    /// Observed count
    pub observed: usize,
    /// Expected count
    pub expected: f64,
    /// Contribution to chi-squared statistic
    pub contribution: f64,
}

/// Chi-squared test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSquaredAnalysis {
    /// Sample size
    pub sample_size: usize,
    /// Number of bins
    pub num_bins: usize,
    /// Degrees of freedom
    pub degrees_of_freedom: usize,
    /// Chi-squared test statistic
    pub statistic: f64,
    /// P-value
    pub p_value: f64,
    /// Significance level used for pass/fail
    pub significance_level: f64,
    /// Whether the test passes
    pub passes: bool,
    /// Critical value at significance level
    pub critical_value: f64,
    /// Bin frequencies with observed vs expected
    pub bin_frequencies: Vec<BinFrequency>,
    /// Cramér's V effect size (0 = no association, 1 = perfect association)
    pub cramers_v: f64,
    /// Issues found during analysis
    pub issues: Vec<String>,
}

/// Expected distribution type for comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectedDistribution {
    /// Uniform distribution (equal probability per bin)
    Uniform,
    /// Custom expected frequencies (must sum to sample size)
    Custom(Vec<f64>),
    /// Expected proportions (must sum to 1.0)
    Proportions(Vec<f64>),
    /// Compare against another observed distribution
    Observed(Vec<usize>),
}

impl Default for ExpectedDistribution {
    fn default() -> Self {
        Self::Uniform
    }
}

/// Analyzer for chi-squared goodness-of-fit tests.
pub struct ChiSquaredAnalyzer {
    /// Binning strategy for continuous data
    binning: BinningStrategy,
    /// Expected distribution
    expected: ExpectedDistribution,
    /// Significance level
    significance_level: f64,
    /// Minimum expected frequency per bin (for validity)
    min_expected: f64,
}

impl ChiSquaredAnalyzer {
    /// Create a new analyzer with default settings.
    pub fn new() -> Self {
        Self {
            binning: BinningStrategy::Auto,
            expected: ExpectedDistribution::Uniform,
            significance_level: 0.05,
            min_expected: 5.0,
        }
    }

    /// Set the binning strategy.
    pub fn with_binning(mut self, strategy: BinningStrategy) -> Self {
        self.binning = strategy;
        self
    }

    /// Set the expected distribution.
    pub fn with_expected(mut self, expected: ExpectedDistribution) -> Self {
        self.expected = expected;
        self
    }

    /// Set the significance level.
    pub fn with_significance_level(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }

    /// Set minimum expected frequency per bin.
    pub fn with_min_expected(mut self, min: f64) -> Self {
        self.min_expected = min;
        self
    }

    /// Analyze continuous data (will be binned).
    pub fn analyze_continuous(&self, values: &[f64]) -> EvalResult<ChiSquaredAnalysis> {
        let n = values.len();
        if n < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: n,
            });
        }

        // Filter invalid values
        let valid_values: Vec<f64> = values.iter().filter(|&&v| v.is_finite()).copied().collect();

        if valid_values.len() < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: valid_values.len(),
            });
        }

        // Create bins
        let (edges, observed) = self.bin_data(&valid_values)?;
        let n_f = valid_values.len() as f64;

        // Calculate expected frequencies
        let expected = self.calculate_expected(&observed, n_f)?;

        self.perform_test(&edges, &observed, &expected)
    }

    /// Analyze categorical/count data directly.
    pub fn analyze_categorical(&self, observed: &[usize]) -> EvalResult<ChiSquaredAnalysis> {
        if observed.is_empty() {
            return Err(EvalError::InvalidParameter(
                "Observed counts cannot be empty".to_string(),
            ));
        }

        let total: usize = observed.iter().sum();
        if total < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: total,
            });
        }

        let n_f = total as f64;

        // Create pseudo-edges for categorical bins
        let edges: Vec<f64> = (0..=observed.len()).map(|i| i as f64).collect();

        // Calculate expected frequencies
        let expected = self.calculate_expected(observed, n_f)?;

        self.perform_test(&edges, observed, &expected)
    }

    /// Bin continuous data according to strategy.
    fn bin_data(&self, values: &[f64]) -> EvalResult<(Vec<f64>, Vec<usize>)> {
        let mut sorted: Vec<f64> = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];

        let edges = match &self.binning {
            BinningStrategy::EqualWidth { num_bins } => {
                let width = (max - min) / (*num_bins as f64);
                (0..=*num_bins).map(|i| min + (i as f64) * width).collect()
            }
            BinningStrategy::EqualFrequency { num_bins } => {
                let n = sorted.len();
                let mut edges = vec![min];
                for i in 1..*num_bins {
                    let idx = (i * n) / *num_bins;
                    edges.push(sorted[idx.min(n - 1)]);
                }
                edges.push(max);
                edges
            }
            BinningStrategy::Custom { edges } => edges.clone(),
            BinningStrategy::Auto => {
                // Sturges' rule
                let num_bins = (1.0 + (values.len() as f64).log2()).ceil() as usize;
                let width = (max - min) / (num_bins as f64);
                (0..=num_bins).map(|i| min + (i as f64) * width).collect()
            }
        };

        if edges.len() < 2 {
            return Err(EvalError::InvalidParameter(
                "Need at least 2 bin edges".to_string(),
            ));
        }

        // Count observations in each bin
        let num_bins = edges.len() - 1;
        let mut counts = vec![0usize; num_bins];

        for &v in values {
            for (i, window) in edges.windows(2).enumerate() {
                let (lower, upper) = (window[0], window[1]);
                if v >= lower && (v < upper || (i == num_bins - 1 && v <= upper)) {
                    counts[i] += 1;
                    break;
                }
            }
        }

        Ok((edges, counts))
    }

    /// Calculate expected frequencies based on distribution type.
    fn calculate_expected(&self, observed: &[usize], total: f64) -> EvalResult<Vec<f64>> {
        match &self.expected {
            ExpectedDistribution::Uniform => {
                let expected_per_bin = total / (observed.len() as f64);
                Ok(vec![expected_per_bin; observed.len()])
            }
            ExpectedDistribution::Custom(expected) => {
                if expected.len() != observed.len() {
                    return Err(EvalError::InvalidParameter(format!(
                        "Expected {} frequencies, got {}",
                        observed.len(),
                        expected.len()
                    )));
                }
                Ok(expected.clone())
            }
            ExpectedDistribution::Proportions(props) => {
                if props.len() != observed.len() {
                    return Err(EvalError::InvalidParameter(format!(
                        "Expected {} proportions, got {}",
                        observed.len(),
                        props.len()
                    )));
                }
                let sum: f64 = props.iter().sum();
                if (sum - 1.0).abs() > 0.01 {
                    return Err(EvalError::InvalidParameter(format!(
                        "Proportions must sum to 1.0, got {}",
                        sum
                    )));
                }
                Ok(props.iter().map(|&p| p * total).collect())
            }
            ExpectedDistribution::Observed(other) => {
                if other.len() != observed.len() {
                    return Err(EvalError::InvalidParameter(format!(
                        "Expected {} categories, got {}",
                        observed.len(),
                        other.len()
                    )));
                }
                let other_total: f64 = other.iter().sum::<usize>() as f64;
                Ok(other
                    .iter()
                    .map(|&c| (c as f64) / other_total * total)
                    .collect())
            }
        }
    }

    /// Perform the chi-squared test.
    fn perform_test(
        &self,
        edges: &[f64],
        observed: &[usize],
        expected: &[f64],
    ) -> EvalResult<ChiSquaredAnalysis> {
        let n = observed.len();
        let total: usize = observed.iter().sum();
        let n_f = total as f64;

        let mut issues = Vec::new();

        // Check minimum expected frequency
        let low_expected: Vec<_> = expected
            .iter()
            .enumerate()
            .filter(|(_, &e)| e < self.min_expected)
            .collect();
        if !low_expected.is_empty() {
            issues.push(format!(
                "{} bins have expected frequency < {:.1}; results may be unreliable",
                low_expected.len(),
                self.min_expected
            ));
        }

        // Calculate chi-squared statistic and bin details
        let mut chi_squared = 0.0;
        let mut bin_frequencies = Vec::new();

        for (i, ((&obs, &exp), window)) in observed
            .iter()
            .zip(expected.iter())
            .zip(edges.windows(2))
            .enumerate()
        {
            let contribution = if exp > 0.0 {
                let diff = obs as f64 - exp;
                (diff * diff) / exp
            } else {
                0.0
            };
            chi_squared += contribution;

            bin_frequencies.push(BinFrequency {
                index: i,
                lower: window[0],
                upper: window[1],
                observed: obs,
                expected: exp,
                contribution,
            });
        }

        // Degrees of freedom
        // For goodness-of-fit: df = num_bins - 1 - estimated_parameters
        // For uniform: df = num_bins - 1 (no parameters estimated from data)
        let df = n.saturating_sub(1);
        if df == 0 {
            return Err(EvalError::InvalidParameter(
                "Need at least 2 bins for chi-squared test".to_string(),
            ));
        }

        // Calculate p-value
        let p_value = chi_squared_p_value(chi_squared, df);

        // Critical value
        let critical_value = chi_squared_critical(df, self.significance_level);

        // Cramér's V effect size
        let cramers_v = (chi_squared / n_f).sqrt();

        let passes = chi_squared <= critical_value;

        if !passes {
            issues.push(format!(
                "χ² = {:.4} exceeds critical value {:.4} at α = {:.2}",
                chi_squared, critical_value, self.significance_level
            ));
        }

        Ok(ChiSquaredAnalysis {
            sample_size: total,
            num_bins: n,
            degrees_of_freedom: df,
            statistic: chi_squared,
            p_value,
            significance_level: self.significance_level,
            passes,
            critical_value,
            bin_frequencies,
            cramers_v,
            issues,
        })
    }
}

impl Default for ChiSquaredAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate p-value for chi-squared statistic using incomplete gamma function.
fn chi_squared_p_value(chi_sq: f64, df: usize) -> f64 {
    // P(X > chi_sq) = 1 - P(X <= chi_sq) = 1 - gamma_cdf(chi_sq, df)
    // Using upper incomplete gamma function
    1.0 - lower_incomplete_gamma(df as f64 / 2.0, chi_sq / 2.0)
}

/// Calculate chi-squared critical value for given df and significance level.
fn chi_squared_critical(df: usize, alpha: f64) -> f64 {
    // Use Wilson-Hilferty approximation for chi-squared quantiles
    // For df >= 2: chi_sq ≈ df * (1 - 2/(9*df) + z * sqrt(2/(9*df)))^3
    // where z is the standard normal quantile

    if df == 0 {
        return 0.0;
    }

    let df_f = df as f64;

    // Get z-score for 1-alpha quantile
    let z = normal_quantile(1.0 - alpha);

    // Wilson-Hilferty approximation
    let term = 2.0 / (9.0 * df_f);
    let inner = 1.0 - term + z * term.sqrt();

    df_f * inner.powi(3).max(0.0)
}

/// Lower incomplete gamma function regularized.
fn lower_incomplete_gamma(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= a + 1.0 {
        // Use continued fraction for large x
        1.0 - upper_incomplete_gamma_cf(a, x)
    } else {
        // Use series expansion for small x
        lower_incomplete_gamma_series(a, x)
    }
}

/// Series expansion for lower incomplete gamma.
fn lower_incomplete_gamma_series(a: f64, x: f64) -> f64 {
    let ln_gamma_a = ln_gamma(a);
    let mut sum = 1.0 / a;
    let mut term = 1.0 / a;

    for n in 1..200 {
        term *= x / (a + n as f64);
        sum += term;
        if term.abs() < 1e-10 * sum.abs() {
            break;
        }
    }

    sum * x.powf(a) * (-x).exp() / ln_gamma_a.exp()
}

/// Continued fraction for upper incomplete gamma.
fn upper_incomplete_gamma_cf(a: f64, x: f64) -> f64 {
    let ln_gamma_a = ln_gamma(a);

    // Lentz's algorithm
    let mut f = 1e-30_f64;
    let mut c = 1e-30_f64;
    let mut d = 0.0_f64;

    for i in 1..200 {
        let i_f = i as f64;
        let an = if i == 1 {
            1.0
        } else if i % 2 == 0 {
            (i_f / 2.0 - 1.0) - a + 1.0
        } else {
            (i_f - 1.0) / 2.0
        };
        let bn = if i == 1 { x - a + 1.0 } else { x - a + i_f };

        d = bn + an * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = bn + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < 1e-10 {
            break;
        }
    }

    f * x.powf(a) * (-x).exp() / ln_gamma_a.exp()
}

/// Log gamma function.
fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    // Lanczos approximation
    let coeffs = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ];

    let tmp = x + 5.5;
    let tmp = tmp - (x + 0.5) * tmp.ln();

    let mut ser = 1.000000000190015;
    for (i, &c) in coeffs.iter().enumerate() {
        ser += c / (x + (i + 1) as f64);
    }

    -tmp + (2.5066282746310005 * ser / x).ln()
}

/// Standard normal quantile (inverse CDF) using rational approximation.
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p == 0.5 {
        return 0.0;
    }

    // Rational approximation from Abramowitz & Stegun
    let t = if p < 0.5 {
        (-2.0 * p.ln()).sqrt()
    } else {
        (-2.0 * (1.0 - p).ln()).sqrt()
    };

    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let z = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);

    if p < 0.5 {
        -z
    } else {
        z
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, Uniform};

    #[test]
    fn test_uniform_distribution() {
        // Generate uniform data and test against uniform expectation
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let uniform = Uniform::new(0.0, 100.0);
        let values: Vec<f64> = (0..1000).map(|_| uniform.sample(&mut rng)).collect();

        let analyzer = ChiSquaredAnalyzer::new()
            .with_binning(BinningStrategy::EqualWidth { num_bins: 10 })
            .with_expected(ExpectedDistribution::Uniform)
            .with_significance_level(0.05);

        let result = analyzer.analyze_continuous(&values).unwrap();
        assert!(
            result.passes,
            "Uniform data should pass uniform chi-squared test"
        );
        assert!(result.p_value > 0.05);
    }

    #[test]
    fn test_categorical_uniform() {
        // Equal counts across categories
        let observed = vec![100, 98, 102, 100, 100]; // Nearly equal

        let analyzer = ChiSquaredAnalyzer::new()
            .with_expected(ExpectedDistribution::Uniform)
            .with_significance_level(0.05);

        let result = analyzer.analyze_categorical(&observed).unwrap();
        assert!(result.passes, "Nearly uniform counts should pass");
    }

    #[test]
    fn test_categorical_deviation() {
        // Clearly non-uniform distribution
        let observed = vec![400, 50, 25, 15, 10]; // Very skewed

        let analyzer = ChiSquaredAnalyzer::new()
            .with_expected(ExpectedDistribution::Uniform)
            .with_significance_level(0.05);

        let result = analyzer.analyze_categorical(&observed).unwrap();
        assert!(
            !result.passes,
            "Highly skewed counts should fail uniform test"
        );
    }

    #[test]
    fn test_custom_proportions() {
        // Test against known proportions
        let observed = vec![300, 200, 100]; // 50%, 33%, 17%
        let expected_props = vec![0.50, 0.33, 0.17];

        let analyzer = ChiSquaredAnalyzer::new()
            .with_expected(ExpectedDistribution::Proportions(expected_props))
            .with_significance_level(0.05);

        let result = analyzer.analyze_categorical(&observed).unwrap();
        // Should pass or be close to passing given the proportions match
        assert!(result.sample_size == 600);
    }

    #[test]
    fn test_binning_strategies() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let uniform = Uniform::new(0.0, 100.0);
        let values: Vec<f64> = (0..500).map(|_| uniform.sample(&mut rng)).collect();

        // Test equal-width
        let analyzer1 =
            ChiSquaredAnalyzer::new().with_binning(BinningStrategy::EqualWidth { num_bins: 10 });
        let result1 = analyzer1.analyze_continuous(&values).unwrap();
        assert_eq!(result1.num_bins, 10);

        // Test equal-frequency
        let analyzer2 =
            ChiSquaredAnalyzer::new().with_binning(BinningStrategy::EqualFrequency { num_bins: 5 });
        let result2 = analyzer2.analyze_continuous(&values).unwrap();
        assert_eq!(result2.num_bins, 5);

        // Test auto
        let analyzer3 = ChiSquaredAnalyzer::new().with_binning(BinningStrategy::Auto);
        let result3 = analyzer3.analyze_continuous(&values).unwrap();
        assert!(result3.num_bins > 0);
    }

    #[test]
    fn test_insufficient_data() {
        let values = vec![1.0, 2.0, 3.0]; // Too few

        let analyzer = ChiSquaredAnalyzer::new();
        let result = analyzer.analyze_continuous(&values);

        assert!(matches!(
            result,
            Err(EvalError::InsufficientData {
                required: 10,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_cramers_v() {
        // Perfect deviation should have high Cramér's V
        let observed = vec![500, 0, 0, 0, 0]; // All in first bin

        let analyzer = ChiSquaredAnalyzer::new()
            .with_expected(ExpectedDistribution::Uniform)
            .with_significance_level(0.05);

        let result = analyzer.analyze_categorical(&observed).unwrap();
        assert!(
            result.cramers_v > 0.5,
            "Strong deviation should have high V"
        );
    }

    #[test]
    fn test_bin_frequencies() {
        let observed = vec![50, 100, 50];

        let analyzer = ChiSquaredAnalyzer::new().with_expected(ExpectedDistribution::Uniform);

        let result = analyzer.analyze_categorical(&observed).unwrap();

        assert_eq!(result.bin_frequencies.len(), 3);

        // First bin: observed=50, expected=66.67, contribution = (50-66.67)^2/66.67
        let first_bin = &result.bin_frequencies[0];
        assert_eq!(first_bin.observed, 50);
        assert!((first_bin.expected - 66.666).abs() < 0.01);
    }

    #[test]
    fn test_critical_value_ordering() {
        // Critical values should increase as alpha decreases
        let cv_10 = chi_squared_critical(10, 0.10);
        let cv_05 = chi_squared_critical(10, 0.05);
        let cv_01 = chi_squared_critical(10, 0.01);

        assert!(cv_10 < cv_05);
        assert!(cv_05 < cv_01);
    }

    #[test]
    fn test_p_value_range() {
        // P-value should be in [0, 1]
        let p1 = chi_squared_p_value(0.0, 5);
        let p2 = chi_squared_p_value(5.0, 5);
        let p3 = chi_squared_p_value(50.0, 5);

        assert!(p1 >= 0.0 && p1 <= 1.0);
        assert!(p2 >= 0.0 && p2 <= 1.0);
        assert!(p3 >= 0.0 && p3 <= 1.0);

        // Higher chi-squared should have lower p-value
        assert!(p1 > p2);
        assert!(p2 > p3);
    }
}
