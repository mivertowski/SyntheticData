//! Correlation analysis for evaluating cross-field dependencies.
//!
//! Validates that generated data maintains expected correlations between
//! related fields (e.g., amount vs. line items, processing time vs. complexity).

use crate::error::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expected correlation between two fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedCorrelation {
    /// First field name
    pub field1: String,
    /// Second field name
    pub field2: String,
    /// Expected Pearson correlation coefficient
    pub expected_r: f64,
    /// Acceptable deviation tolerance
    pub tolerance: f64,
}

impl ExpectedCorrelation {
    /// Create a new expected correlation.
    pub fn new(field1: impl Into<String>, field2: impl Into<String>, expected_r: f64) -> Self {
        Self {
            field1: field1.into(),
            field2: field2.into(),
            expected_r,
            tolerance: 0.10, // 0.10 default tolerance
        }
    }

    /// Set tolerance.
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }
}

/// Result of correlation check for a pair of fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationCheckResult {
    /// First field
    pub field1: String,
    /// Second field
    pub field2: String,
    /// Observed Pearson correlation
    pub observed_r: f64,
    /// Expected correlation (if specified)
    pub expected_r: Option<f64>,
    /// Deviation from expected
    pub deviation: Option<f64>,
    /// Whether within tolerance
    pub within_tolerance: bool,
    /// P-value for correlation significance
    pub p_value: f64,
    /// Sample size
    pub sample_size: usize,
}

/// Full correlation matrix analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    /// Sample size
    pub sample_size: usize,
    /// Field names in order
    pub fields: Vec<String>,
    /// Correlation matrix (upper triangular, row-major)
    pub correlation_matrix: Vec<f64>,
    /// Individual correlation check results
    pub correlation_checks: Vec<CorrelationCheckResult>,
    /// Number of checks that passed
    pub checks_passed: usize,
    /// Number of checks that failed
    pub checks_failed: usize,
    /// Overall pass status
    pub passes: bool,
    /// Issues found
    pub issues: Vec<String>,
}

impl CorrelationAnalysis {
    /// Get correlation between two fields by name.
    pub fn get_correlation(&self, field1: &str, field2: &str) -> Option<f64> {
        let idx1 = self.fields.iter().position(|f| f == field1)?;
        let idx2 = self.fields.iter().position(|f| f == field2)?;

        if idx1 == idx2 {
            return Some(1.0);
        }

        let (i, j) = if idx1 < idx2 {
            (idx1, idx2)
        } else {
            (idx2, idx1)
        };

        // Calculate index in upper triangular matrix
        let n = self.fields.len();
        let mut matrix_idx = 0;
        for row in 0..i {
            matrix_idx += n - row - 1;
        }
        matrix_idx += j - i - 1;

        self.correlation_matrix.get(matrix_idx).copied()
    }
}

/// Analyzer for correlation analysis.
pub struct CorrelationAnalyzer {
    /// Expected correlations to validate
    expected_correlations: Vec<ExpectedCorrelation>,
    /// Significance level for p-value tests
    significance_level: f64,
}

impl CorrelationAnalyzer {
    /// Create a new correlation analyzer.
    pub fn new() -> Self {
        Self {
            expected_correlations: Vec::new(),
            significance_level: 0.05,
        }
    }

    /// Add expected correlations to validate.
    pub fn with_expected_correlations(mut self, correlations: Vec<ExpectedCorrelation>) -> Self {
        self.expected_correlations = correlations;
        self
    }

    /// Set significance level.
    pub fn with_significance_level(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }

    /// Analyze correlations in the provided data.
    ///
    /// `data` is a map from field name to values for that field.
    /// All value vectors must have the same length.
    pub fn analyze(&self, data: &HashMap<String, Vec<f64>>) -> EvalResult<CorrelationAnalysis> {
        if data.is_empty() {
            return Err(EvalError::MissingData("No data provided".to_string()));
        }

        // Verify all columns have same length
        let lengths: Vec<usize> = data.values().map(|v| v.len()).collect();
        if !lengths.iter().all(|&l| l == lengths[0]) {
            return Err(EvalError::InvalidParameter(
                "All fields must have same number of values".to_string(),
            ));
        }

        let sample_size = lengths[0];
        if sample_size < 3 {
            return Err(EvalError::InsufficientData {
                required: 3,
                actual: sample_size,
            });
        }

        // Get ordered field names
        let fields: Vec<String> = data.keys().cloned().collect();
        let n_fields = fields.len();

        // Calculate full correlation matrix
        let mut correlation_matrix = Vec::new();
        for i in 0..n_fields {
            for j in (i + 1)..n_fields {
                let field1 = &fields[i];
                let field2 = &fields[j];
                let values1 = data.get(field1).unwrap();
                let values2 = data.get(field2).unwrap();
                let r = pearson_correlation(values1, values2);
                correlation_matrix.push(r);
            }
        }

        // Check expected correlations
        let mut correlation_checks = Vec::new();
        let mut issues = Vec::new();

        for expected in &self.expected_correlations {
            let values1 = match data.get(&expected.field1) {
                Some(v) => v,
                None => {
                    issues.push(format!("Field '{}' not found in data", expected.field1));
                    continue;
                }
            };
            let values2 = match data.get(&expected.field2) {
                Some(v) => v,
                None => {
                    issues.push(format!("Field '{}' not found in data", expected.field2));
                    continue;
                }
            };

            let observed_r = pearson_correlation(values1, values2);
            let p_value = correlation_p_value(observed_r, sample_size);
            let deviation = (observed_r - expected.expected_r).abs();
            let within_tolerance = deviation <= expected.tolerance;

            if !within_tolerance {
                issues.push(format!(
                    "Correlation between '{}' and '{}': expected {:.3}, got {:.3} (deviation {:.3} > tolerance {:.3})",
                    expected.field1, expected.field2, expected.expected_r, observed_r, deviation, expected.tolerance
                ));
            }

            correlation_checks.push(CorrelationCheckResult {
                field1: expected.field1.clone(),
                field2: expected.field2.clone(),
                observed_r,
                expected_r: Some(expected.expected_r),
                deviation: Some(deviation),
                within_tolerance,
                p_value,
                sample_size,
            });
        }

        let checks_passed = correlation_checks
            .iter()
            .filter(|c| c.within_tolerance)
            .count();
        let checks_failed = correlation_checks.len() - checks_passed;
        let passes = checks_failed == 0;

        Ok(CorrelationAnalysis {
            sample_size,
            fields,
            correlation_matrix,
            correlation_checks,
            checks_passed,
            checks_failed,
            passes,
            issues,
        })
    }

    /// Analyze correlations from paired data (simpler interface for two fields).
    pub fn analyze_pair(
        &self,
        values1: &[f64],
        values2: &[f64],
    ) -> EvalResult<CorrelationCheckResult> {
        if values1.len() != values2.len() {
            return Err(EvalError::InvalidParameter(
                "Value vectors must have same length".to_string(),
            ));
        }

        let n = values1.len();
        if n < 3 {
            return Err(EvalError::InsufficientData {
                required: 3,
                actual: n,
            });
        }

        let observed_r = pearson_correlation(values1, values2);
        let p_value = correlation_p_value(observed_r, n);

        Ok(CorrelationCheckResult {
            field1: "field1".to_string(),
            field2: "field2".to_string(),
            observed_r,
            expected_r: None,
            deviation: None,
            within_tolerance: true,
            p_value,
            sample_size: n,
        })
    }
}

impl Default for CorrelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Pearson correlation coefficient.
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "Vectors must have same length");

    let n = x.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x <= 0.0 || var_y <= 0.0 {
        return 0.0;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

/// Calculate Spearman rank correlation coefficient.
pub fn spearman_correlation(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "Vectors must have same length");

    let n = x.len();
    if n < 2 {
        return 0.0;
    }

    // Calculate ranks
    let rank_x = calculate_ranks(x);
    let rank_y = calculate_ranks(y);

    // Pearson correlation of ranks
    pearson_correlation(&rank_x, &rank_y)
}

/// Calculate ranks for a vector (handles ties with average rank).
fn calculate_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().cloned().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        // Find all ties
        let mut j = i;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-10 {
            j += 1;
        }

        // Average rank for ties
        let avg_rank = (i + j) as f64 / 2.0 + 0.5;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }

        i = j;
    }

    ranks
}

/// Calculate p-value for correlation coefficient using t-distribution approximation.
fn correlation_p_value(r: f64, n: usize) -> f64 {
    if n <= 2 {
        return 1.0;
    }

    if r.abs() >= 1.0 {
        return 0.0;
    }

    // t-statistic: t = r * sqrt((n-2) / (1-r²))
    let df = n - 2;
    let t = r * ((df as f64) / (1.0 - r * r)).sqrt();

    // Two-tailed p-value using t-distribution
    let t_abs = t.abs();
    2.0 * student_t_cdf(-t_abs, df as f64)
}

/// Student-t CDF approximation.
fn student_t_cdf(t: f64, df: f64) -> f64 {
    // For large df, use normal approximation
    if df > 30.0 {
        return normal_cdf(t);
    }

    // Use beta function approximation
    let t2 = t * t;
    let prob = 0.5 * incomplete_beta(df / 2.0, 0.5, df / (df + t2));

    if t > 0.0 {
        1.0 - prob
    } else {
        prob
    }
}

/// Standard normal CDF.
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
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

/// Incomplete beta function approximation.
fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    let lbeta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let front = (x.powf(a) * (1.0 - x).powf(b)) / lbeta.exp();

    // Lentz's algorithm for continued fraction
    let mut c: f64 = 1.0;
    let mut d: f64 = 1.0 / (1.0 - (a + b) * x / (a + 1.0)).max(1e-30);
    let mut h = d;

    for m in 1..100 {
        let m = m as f64;
        let d1 = m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m));
        let d2 = -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0));

        d = 1.0 / (1.0 + d1 * d).max(1e-30);
        c = 1.0 + d1 / c.max(1e-30);
        h *= c * d;

        d = 1.0 / (1.0 + d2 * d).max(1e-30);
        c = 1.0 + d2 / c.max(1e-30);
        h *= c * d;

        if ((c * d) - 1.0).abs() < 1e-8 {
            break;
        }
    }

    front * h / a
}

/// Log gamma function approximation.
fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    0.5 * (2.0 * std::f64::consts::PI / x).ln() + x * ((x + 1.0 / (12.0 * x)).ln() - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_correlation() {
        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&x, &y);
        assert!((r - 1.0).abs() < 0.001);

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r_neg = pearson_correlation(&x, &y_neg);
        assert!((r_neg + 1.0).abs() < 0.001);

        // Low correlation (values chosen to have weak correlation)
        let x_rand = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_rand = vec![3.0, 1.0, 4.0, 5.0, 2.0];
        let r_rand = pearson_correlation(&x_rand, &y_rand);
        // Verify correlation is weak (not strongly positive or negative)
        assert!(
            r_rand.abs() < 0.7,
            "Expected weak correlation, got {}",
            r_rand
        );
    }

    #[test]
    fn test_spearman_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = spearman_correlation(&x, &y);
        assert!((r - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_analyzer() {
        let mut data = HashMap::new();
        data.insert(
            "x".to_string(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        );
        data.insert(
            "y".to_string(),
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
        );
        data.insert(
            "z".to_string(),
            vec![10.0, 8.0, 6.0, 4.0, 2.0, 1.0, 3.0, 5.0, 7.0, 9.0],
        );

        let analyzer =
            CorrelationAnalyzer::new()
                .with_expected_correlations(vec![
                    ExpectedCorrelation::new("x", "y", 1.0).with_tolerance(0.01)
                ]);

        let result = analyzer.analyze(&data).unwrap();
        assert_eq!(result.sample_size, 10);
        assert!(result.passes);

        // Check we can retrieve correlation
        let r_xy = result.get_correlation("x", "y").unwrap();
        assert!((r_xy - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_failure() {
        let mut data = HashMap::new();
        data.insert("x".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        data.insert("y".to_string(), vec![5.0, 4.0, 3.0, 2.0, 1.0]); // Negative correlation

        let analyzer = CorrelationAnalyzer::new().with_expected_correlations(vec![
            ExpectedCorrelation::new("x", "y", 0.8).with_tolerance(0.1), // Expected positive
        ]);

        let result = analyzer.analyze(&data).unwrap();
        assert!(!result.passes);
        assert_eq!(result.checks_failed, 1);
    }

    #[test]
    fn test_correlation_p_value() {
        // Strong correlation with large sample should have low p-value
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| v * 2.0 + 1.0).collect();

        let r = pearson_correlation(&x, &y);
        let p = correlation_p_value(r, x.len());

        assert!(r > 0.99);
        assert!(p < 0.001);
    }

    #[test]
    fn test_rank_calculation() {
        let values = vec![1.0, 3.0, 2.0, 3.0, 5.0]; // Note: ties at 3.0
        let ranks = calculate_ranks(&values);

        // 1.0 -> rank 1
        // 2.0 -> rank 2
        // 3.0, 3.0 -> ranks 3.5, 3.5 (average of 3 and 4)
        // 5.0 -> rank 5
        assert!((ranks[0] - 1.0).abs() < 0.001);
        assert!((ranks[2] - 2.0).abs() < 0.001);
        assert!((ranks[1] - 3.5).abs() < 0.001);
        assert!((ranks[3] - 3.5).abs() < 0.001);
        assert!((ranks[4] - 5.0).abs() < 0.001);
    }
}
