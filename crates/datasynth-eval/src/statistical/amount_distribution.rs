//! Amount distribution analysis.
//!
//! Analyzes the statistical properties of generated amounts including
//! log-normal distribution fitting and round-number bias detection.

use crate::error::{EvalError, EvalResult};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Results of amount distribution analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistributionAnalysis {
    /// Number of samples analyzed.
    pub sample_size: usize,
    /// Mean amount.
    pub mean: Decimal,
    /// Median amount.
    pub median: Decimal,
    /// Standard deviation.
    pub std_dev: Decimal,
    /// Minimum amount.
    pub min: Decimal,
    /// Maximum amount.
    pub max: Decimal,
    /// 1st percentile.
    pub percentile_1: Decimal,
    /// 99th percentile.
    pub percentile_99: Decimal,
    /// Skewness (positive = right-skewed, typical for financial data).
    pub skewness: f64,
    /// Kurtosis (excess kurtosis, 0 = normal distribution).
    pub kurtosis: f64,
    /// Kolmogorov-Smirnov statistic against log-normal.
    pub lognormal_ks_stat: Option<f64>,
    /// P-value from KS test.
    pub lognormal_ks_pvalue: Option<f64>,
    /// Fitted log-normal mu parameter.
    pub fitted_mu: Option<f64>,
    /// Fitted log-normal sigma parameter.
    pub fitted_sigma: Option<f64>,
    /// Ratio of round numbers (ending in .00).
    pub round_number_ratio: f64,
    /// Ratio of nice numbers (ending in 0 or 5).
    pub nice_number_ratio: f64,
    /// Whether test passes thresholds.
    pub passes: bool,
}

/// Analyzer for amount distributions.
pub struct AmountDistributionAnalyzer {
    /// Expected log-normal mu parameter.
    expected_mu: Option<f64>,
    /// Expected log-normal sigma parameter.
    expected_sigma: Option<f64>,
    /// Significance level for statistical tests.
    significance_level: f64,
}

impl AmountDistributionAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            expected_mu: None,
            expected_sigma: None,
            significance_level: 0.05,
        }
    }

    /// Set expected log-normal parameters for comparison.
    pub fn with_expected_lognormal(mut self, mu: f64, sigma: f64) -> Self {
        self.expected_mu = Some(mu);
        self.expected_sigma = Some(sigma);
        self
    }

    /// Set significance level for statistical tests.
    pub fn with_significance_level(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }

    /// Analyze amount distribution.
    pub fn analyze(&self, amounts: &[Decimal]) -> EvalResult<AmountDistributionAnalysis> {
        let n = amounts.len();
        if n < 2 {
            return Err(EvalError::InsufficientData {
                required: 2,
                actual: n,
            });
        }

        // Filter positive amounts for log-normal analysis
        let positive_amounts: Vec<Decimal> = amounts
            .iter()
            .filter(|a| **a > Decimal::ZERO)
            .copied()
            .collect();

        // Sort for percentile calculations
        let mut sorted = amounts.to_vec();
        sorted.sort();

        // Basic statistics
        let sum: Decimal = amounts.iter().sum();
        let mean = sum / Decimal::from(n);
        let median = sorted[n / 2];
        let min = sorted[0];
        let max = sorted[n - 1];

        // Percentiles
        let percentile_1 = sorted[(n as f64 * 0.01) as usize];
        let percentile_99 = sorted[((n as f64 * 0.99) as usize).min(n - 1)];

        // Variance and standard deviation
        let variance: Decimal = amounts
            .iter()
            .map(|a| (*a - mean) * (*a - mean))
            .sum::<Decimal>()
            / Decimal::from(n - 1);
        let std_dev = decimal_sqrt(variance);

        // Convert to f64 for higher moments
        let amounts_f64: Vec<f64> = amounts.iter().filter_map(|a| a.to_f64()).collect();
        let mean_f64 = amounts_f64.iter().sum::<f64>() / amounts_f64.len() as f64;
        let std_f64 = (amounts_f64
            .iter()
            .map(|a| (a - mean_f64).powi(2))
            .sum::<f64>()
            / (amounts_f64.len() - 1) as f64)
            .sqrt();

        // Skewness
        let skewness = if std_f64 > 0.0 {
            let n_f64 = amounts_f64.len() as f64;
            let m3 = amounts_f64
                .iter()
                .map(|a| ((a - mean_f64) / std_f64).powi(3))
                .sum::<f64>()
                / n_f64;
            m3 * (n_f64 * (n_f64 - 1.0)).sqrt() / (n_f64 - 2.0)
        } else {
            0.0
        };

        // Kurtosis (excess kurtosis)
        let kurtosis = if std_f64 > 0.0 {
            let n_f64 = amounts_f64.len() as f64;
            let m4 = amounts_f64
                .iter()
                .map(|a| ((a - mean_f64) / std_f64).powi(4))
                .sum::<f64>()
                / n_f64;
            m4 - 3.0 // Excess kurtosis
        } else {
            0.0
        };

        // Log-normal fit and KS test (for positive amounts only)
        let (lognormal_ks_stat, lognormal_ks_pvalue, fitted_mu, fitted_sigma) =
            if positive_amounts.len() >= 10 {
                self.lognormal_ks_test(&positive_amounts)
            } else {
                (None, None, None, None)
            };

        // Round number analysis
        let round_count = amounts
            .iter()
            .filter(|a| {
                let frac = a.fract();
                frac.is_zero()
            })
            .count();
        let round_number_ratio = round_count as f64 / n as f64;

        // Nice number analysis (ends in 0 or 5 in cents)
        let nice_count = amounts
            .iter()
            .filter(|a| {
                let cents = (a.fract() * Decimal::ONE_HUNDRED).abs();
                let last_digit = (cents.to_i64().unwrap_or(0) % 10) as u8;
                last_digit == 0 || last_digit == 5
            })
            .count();
        let nice_number_ratio = nice_count as f64 / n as f64;

        // Determine pass/fail
        let passes = lognormal_ks_pvalue.is_none_or(|p| p >= self.significance_level);

        Ok(AmountDistributionAnalysis {
            sample_size: n,
            mean,
            median,
            std_dev,
            min,
            max,
            percentile_1,
            percentile_99,
            skewness,
            kurtosis,
            lognormal_ks_stat,
            lognormal_ks_pvalue,
            fitted_mu,
            fitted_sigma,
            round_number_ratio,
            nice_number_ratio,
            passes,
        })
    }

    /// Perform Kolmogorov-Smirnov test against log-normal distribution.
    fn lognormal_ks_test(
        &self,
        amounts: &[Decimal],
    ) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
        // Convert to log values
        let log_amounts: Vec<f64> = amounts
            .iter()
            .filter_map(|a| a.to_f64())
            .filter(|a| *a > 0.0)
            .map(|a| a.ln())
            .collect();

        if log_amounts.len() < 10 {
            return (None, None, None, None);
        }

        // Fit log-normal by MLE (mean and std of log values)
        let n = log_amounts.len() as f64;
        let mu: f64 = log_amounts.iter().sum::<f64>() / n;
        let sigma: f64 =
            (log_amounts.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

        if sigma <= 0.0 {
            return (None, None, Some(mu), None);
        }

        // KS test against fitted normal distribution of log values
        let mut sorted_log = log_amounts.clone();
        sorted_log.sort_by(|a, b| a.total_cmp(b));

        // Calculate KS statistic
        let n_usize = sorted_log.len();
        let mut d_max = 0.0f64;

        for (i, &x) in sorted_log.iter().enumerate() {
            let f_n = (i + 1) as f64 / n_usize as f64;
            let f_x = normal_cdf((x - mu) / sigma);
            let d_plus = (f_n - f_x).abs();
            let d_minus = (f_x - i as f64 / n_usize as f64).abs();
            d_max = d_max.max(d_plus).max(d_minus);
        }

        // Approximate p-value using Kolmogorov distribution
        // For large n, use asymptotic approximation
        let sqrt_n = (n_usize as f64).sqrt();
        let lambda = (sqrt_n + 0.12 + 0.11 / sqrt_n) * d_max;
        let p_value = kolmogorov_pvalue(lambda);

        (Some(d_max), Some(p_value), Some(mu), Some(sigma))
    }
}

impl Default for AmountDistributionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard normal CDF approximation.
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Error function approximation.
fn erf(x: f64) -> f64 {
    // Horner form of approximation
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

/// Approximate p-value from Kolmogorov distribution.
fn kolmogorov_pvalue(lambda: f64) -> f64 {
    if lambda <= 0.0 {
        return 1.0;
    }

    // Asymptotic approximation for p-value
    // P(D_n > d) ≈ 2 * sum_{k=1}^∞ (-1)^(k-1) * exp(-2k²λ²)
    let mut sum = 0.0;
    let lambda_sq = lambda * lambda;

    for k in 1..=100 {
        let k_f64 = k as f64;
        let term = (-1.0f64).powi(k - 1) * (-2.0 * k_f64 * k_f64 * lambda_sq).exp();
        sum += term;
        if term.abs() < 1e-10 {
            break;
        }
    }

    (2.0 * sum).clamp(0.0, 1.0)
}

/// Approximate square root for Decimal.
fn decimal_sqrt(value: Decimal) -> Decimal {
    if value <= Decimal::ZERO {
        return Decimal::ZERO;
    }

    // Newton-Raphson method
    let mut guess = value / Decimal::TWO;
    for _ in 0..20 {
        let new_guess = (guess + value / guess) / Decimal::TWO;
        if (new_guess - guess).abs() < Decimal::new(1, 10) {
            return new_guess;
        }
        guess = new_guess;
    }
    guess
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_basic_statistics() {
        let amounts = vec![
            dec!(100.00),
            dec!(200.00),
            dec!(300.00),
            dec!(400.00),
            dec!(500.00),
        ];

        let analyzer = AmountDistributionAnalyzer::new();
        let result = analyzer.analyze(&amounts).unwrap();

        assert_eq!(result.sample_size, 5);
        assert_eq!(result.mean, dec!(300.00));
        assert_eq!(result.min, dec!(100.00));
        assert_eq!(result.max, dec!(500.00));
    }

    #[test]
    fn test_round_number_detection() {
        let amounts = vec![
            dec!(100.00), // round
            dec!(200.50), // not round
            dec!(300.00), // round
            dec!(400.25), // not round
            dec!(500.00), // round
        ];

        let analyzer = AmountDistributionAnalyzer::new();
        let result = analyzer.analyze(&amounts).unwrap();

        assert!((result.round_number_ratio - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_insufficient_data() {
        let amounts = vec![dec!(100.00)];
        let analyzer = AmountDistributionAnalyzer::new();
        let result = analyzer.analyze(&amounts);
        assert!(matches!(result, Err(EvalError::InsufficientData { .. })));
    }

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }
}
