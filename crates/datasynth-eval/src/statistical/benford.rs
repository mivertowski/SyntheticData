//! Benford's Law analysis for amount distributions.
//!
//! Benford's Law states that in many naturally occurring collections of numbers,
//! the leading digit is more likely to be small. Specifically, the probability
//! of the first digit d (1-9) is: P(d) = log₁₀(1 + 1/d)

use crate::error::{EvalError, EvalResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use statrs::distribution::{ChiSquared, ContinuousCDF};

/// Expected Benford's Law probabilities for digits 1-9.
/// These are the mathematically exact probabilities: P(d) = log₁₀(1 + 1/d)
#[allow(clippy::approx_constant)] // Values are exact Benford probabilities, not approximations
pub const BENFORD_PROBABILITIES: [f64; 9] = [
    0.30103, // digit 1: log₁₀(2)
    0.17609, // digit 2: log₁₀(3/2)
    0.12494, // digit 3: log₁₀(4/3)
    0.09691, // digit 4: log₁₀(5/4)
    0.07918, // digit 5: log₁₀(6/5)
    0.06695, // digit 6: log₁₀(7/6)
    0.05799, // digit 7: log₁₀(8/7)
    0.05115, // digit 8: log₁₀(9/8)
    0.04576, // digit 9: log₁₀(10/9)
];

/// Conformity level based on Mean Absolute Deviation (MAD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenfordConformity {
    /// Close conformity (MAD < 0.006).
    Close,
    /// Acceptable conformity (MAD < 0.012).
    Acceptable,
    /// Marginal conformity (MAD < 0.015).
    Marginal,
    /// Non-conformity (MAD >= 0.015).
    NonConforming,
}

impl BenfordConformity {
    /// Determine conformity level from MAD value.
    pub fn from_mad(mad: f64) -> Self {
        if mad < 0.006 {
            Self::Close
        } else if mad < 0.012 {
            Self::Acceptable
        } else if mad < 0.015 {
            Self::Marginal
        } else {
            Self::NonConforming
        }
    }
}

/// Results of Benford's Law analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordAnalysis {
    /// Number of samples analyzed.
    pub sample_size: usize,
    /// Observed first-digit frequencies (digits 1-9).
    pub observed_frequencies: [f64; 9],
    /// Observed first-digit counts (digits 1-9).
    pub observed_counts: [u64; 9],
    /// Expected Benford frequencies.
    pub expected_frequencies: [f64; 9],
    /// Chi-squared statistic.
    pub chi_squared: f64,
    /// Degrees of freedom (8).
    pub degrees_of_freedom: u32,
    /// P-value from chi-squared test.
    pub p_value: f64,
    /// Mean Absolute Deviation from expected.
    pub mad: f64,
    /// Conformity level based on MAD.
    pub conformity: BenfordConformity,
    /// Maximum deviation (digit index, deviation value).
    pub max_deviation: (u8, f64),
    /// Whether test passes at the given significance level.
    pub passes: bool,
    /// Anti-Benford score (0.0 = perfect Benford, 1.0 = anti-Benford).
    pub anti_benford_score: f64,
}

/// Analyzer for Benford's Law compliance.
pub struct BenfordAnalyzer {
    /// Significance level for the chi-squared test.
    significance_level: f64,
}

impl BenfordAnalyzer {
    /// Create a new analyzer with the specified significance level.
    pub fn new(significance_level: f64) -> Self {
        Self { significance_level }
    }

    /// Extract the first digit from a decimal amount.
    fn get_first_digit(amount: Decimal) -> Option<u8> {
        let abs_amount = amount.abs();
        if abs_amount.is_zero() {
            return None;
        }

        // Convert to string and find first non-zero digit
        let s = abs_amount.to_string();
        for c in s.chars() {
            if c.is_ascii_digit() && c != '0' {
                return Some(c.to_digit(10).expect("char is ascii digit") as u8);
            }
        }
        None
    }

    /// Analyze a collection of amounts for Benford's Law compliance.
    pub fn analyze(&self, amounts: &[Decimal]) -> EvalResult<BenfordAnalysis> {
        // Filter out zero amounts and extract first digits
        let first_digits: Vec<u8> = amounts
            .iter()
            .filter_map(|&a| Self::get_first_digit(a))
            .collect();

        let n = first_digits.len();
        if n < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: n,
            });
        }

        // Count occurrences of each first digit
        let mut counts = [0u64; 9];
        for digit in first_digits {
            if (1..=9).contains(&digit) {
                counts[(digit - 1) as usize] += 1;
            }
        }

        // Calculate observed frequencies
        let n_f64 = n as f64;
        let observed_frequencies: [f64; 9] = std::array::from_fn(|i| counts[i] as f64 / n_f64);

        // Calculate chi-squared statistic
        let chi_squared: f64 = (0..9)
            .map(|i| {
                let observed = counts[i] as f64;
                let expected = BENFORD_PROBABILITIES[i] * n_f64;
                if expected > 0.0 {
                    (observed - expected).powi(2) / expected
                } else {
                    0.0
                }
            })
            .sum();

        // Calculate p-value from chi-squared distribution (df = 8)
        let chi_sq_dist = ChiSquared::new(8.0).map_err(|e| {
            EvalError::StatisticalError(format!("Failed to create chi-squared distribution: {}", e))
        })?;
        let p_value = 1.0 - chi_sq_dist.cdf(chi_squared);

        // Calculate Mean Absolute Deviation
        let mad: f64 = (0..9)
            .map(|i| (observed_frequencies[i] - BENFORD_PROBABILITIES[i]).abs())
            .sum::<f64>()
            / 9.0;

        // Find maximum deviation
        let max_deviation = (0..9)
            .map(|i| {
                (
                    (i + 1) as u8,
                    (observed_frequencies[i] - BENFORD_PROBABILITIES[i]).abs(),
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("9-element range is non-empty");

        // Calculate anti-Benford score (how much it deviates toward uniform distribution)
        let uniform_prob = 1.0 / 9.0;
        let anti_benford_score: f64 = (0..9)
            .map(|i| {
                let benford_distance = (observed_frequencies[i] - BENFORD_PROBABILITIES[i]).abs();
                let uniform_distance = (observed_frequencies[i] - uniform_prob).abs();
                // Score increases when closer to uniform than to Benford
                if benford_distance > uniform_distance {
                    benford_distance - uniform_distance
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / 9.0;

        let conformity = BenfordConformity::from_mad(mad);
        let passes = p_value >= self.significance_level;

        Ok(BenfordAnalysis {
            sample_size: n,
            observed_frequencies,
            observed_counts: counts,
            expected_frequencies: BENFORD_PROBABILITIES,
            chi_squared,
            degrees_of_freedom: 8,
            p_value,
            mad,
            conformity,
            max_deviation,
            passes,
            anti_benford_score,
        })
    }

    /// Analyze second-digit distribution (more sensitive for fraud detection).
    pub fn analyze_second_digit(&self, amounts: &[Decimal]) -> EvalResult<SecondDigitAnalysis> {
        // Extract second digits
        let second_digits: Vec<u8> = amounts
            .iter()
            .filter_map(|&a| Self::get_second_digit(a))
            .collect();

        let n = second_digits.len();
        if n < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: n,
            });
        }

        // Count occurrences of each second digit (0-9)
        let mut counts = [0u64; 10];
        for digit in second_digits {
            counts[digit as usize] += 1;
        }

        // Calculate observed frequencies
        let n_f64 = n as f64;
        let observed_frequencies: [f64; 10] = std::array::from_fn(|i| counts[i] as f64 / n_f64);

        // Expected second-digit Benford probabilities
        let expected: [f64; 10] = [
            0.11968, 0.11389, 0.10882, 0.10433, 0.10031, 0.09668, 0.09337, 0.09035, 0.08757,
            0.08500,
        ];

        // Chi-squared test
        let chi_squared: f64 = (0..10)
            .map(|i| {
                let observed = counts[i] as f64;
                let exp = expected[i] * n_f64;
                if exp > 0.0 {
                    (observed - exp).powi(2) / exp
                } else {
                    0.0
                }
            })
            .sum();

        let chi_sq_dist = ChiSquared::new(9.0).map_err(|e| {
            EvalError::StatisticalError(format!("Failed to create chi-squared distribution: {}", e))
        })?;
        let p_value = 1.0 - chi_sq_dist.cdf(chi_squared);

        Ok(SecondDigitAnalysis {
            sample_size: n,
            observed_frequencies,
            expected_frequencies: expected,
            chi_squared,
            p_value,
            passes: p_value >= self.significance_level,
        })
    }

    /// Extract the second digit from a decimal amount.
    fn get_second_digit(amount: Decimal) -> Option<u8> {
        let abs_amount = amount.abs();
        if abs_amount.is_zero() {
            return None;
        }

        let s = abs_amount.to_string();
        let mut found_first = false;
        for c in s.chars() {
            if c.is_ascii_digit() {
                if !found_first && c != '0' {
                    found_first = true;
                } else if found_first && c != '.' {
                    return Some(c.to_digit(10).expect("char is ascii digit") as u8);
                }
            }
        }
        None
    }
}

impl Default for BenfordAnalyzer {
    fn default() -> Self {
        Self::new(0.05)
    }
}

/// Results of second-digit Benford's Law analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondDigitAnalysis {
    /// Number of samples analyzed.
    pub sample_size: usize,
    /// Observed second-digit frequencies (digits 0-9).
    pub observed_frequencies: [f64; 10],
    /// Expected second-digit frequencies.
    pub expected_frequencies: [f64; 10],
    /// Chi-squared statistic.
    pub chi_squared: f64,
    /// P-value from chi-squared test.
    pub p_value: f64,
    /// Whether test passes.
    pub passes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_benford_probabilities_sum_to_one() {
        let sum: f64 = BENFORD_PROBABILITIES.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_get_first_digit() {
        assert_eq!(BenfordAnalyzer::get_first_digit(dec!(123.45)), Some(1));
        assert_eq!(BenfordAnalyzer::get_first_digit(dec!(0.0456)), Some(4));
        assert_eq!(BenfordAnalyzer::get_first_digit(dec!(9999)), Some(9));
        assert_eq!(BenfordAnalyzer::get_first_digit(dec!(-567.89)), Some(5));
        assert_eq!(BenfordAnalyzer::get_first_digit(dec!(0)), None);
    }

    #[test]
    fn test_benford_analysis_with_compliant_data() {
        // Generate Benford-compliant data
        let amounts: Vec<Decimal> = (1..=1000)
            .map(|i| {
                // Simple approximation of Benford distribution
                let digit = match i % 100 {
                    0..=29 => 1,
                    30..=46 => 2,
                    47..=59 => 3,
                    60..=69 => 4,
                    70..=77 => 5,
                    78..=84 => 6,
                    85..=90 => 7,
                    91..=95 => 8,
                    _ => 9,
                };
                Decimal::new(digit * 100 + (i % 100) as i64, 2)
            })
            .collect();

        let analyzer = BenfordAnalyzer::default();
        let result = analyzer.analyze(&amounts).unwrap();

        assert_eq!(result.sample_size, 1000);
        assert_eq!(result.degrees_of_freedom, 8);
        // With approximate Benford data, should have reasonable conformity
        assert!(result.mad < 0.05);
    }

    #[test]
    fn test_benford_conformity_levels() {
        assert_eq!(BenfordConformity::from_mad(0.004), BenfordConformity::Close);
        assert_eq!(
            BenfordConformity::from_mad(0.010),
            BenfordConformity::Acceptable
        );
        assert_eq!(
            BenfordConformity::from_mad(0.014),
            BenfordConformity::Marginal
        );
        assert_eq!(
            BenfordConformity::from_mad(0.020),
            BenfordConformity::NonConforming
        );
    }

    #[test]
    fn test_insufficient_data() {
        let amounts = vec![dec!(100), dec!(200), dec!(300)];
        let analyzer = BenfordAnalyzer::default();
        let result = analyzer.analyze(&amounts);
        assert!(matches!(result, Err(EvalError::InsufficientData { .. })));
    }
}
