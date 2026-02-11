//! Line item distribution analysis.
//!
//! Analyzes the distribution of line item counts in journal entries
//! against expected empirical distributions from accounting research.

use crate::error::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use std::collections::HashMap;

/// Expected line item distribution from empirical research (Table III).
pub const EXPECTED_LINE_DISTRIBUTION: [(usize, f64); 11] = [
    (2, 0.6068),    // 60.68% two-line entries
    (3, 0.0577),    // 5.77%
    (4, 0.1663),    // 16.63%
    (5, 0.0306),    // 3.06%
    (6, 0.0332),    // 3.32%
    (7, 0.0113),    // 1.13%
    (8, 0.0188),    // 1.88%
    (9, 0.0042),    // 0.42%
    (10, 0.0633),   // 10-99: 6.33% (simplified to 10+)
    (100, 0.0076),  // 100-999: 0.76%
    (1000, 0.0002), // 1000+: 0.02%
];

/// Expected even/odd distribution.
pub const EXPECTED_EVEN_RATIO: f64 = 0.88;

/// Expected equal debit/credit split ratio.
pub const EXPECTED_EQUAL_SPLIT_RATIO: f64 = 0.82;

/// Results of line item distribution analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemAnalysis {
    /// Number of entries analyzed.
    pub sample_size: usize,
    /// Distribution of line counts.
    pub line_count_distribution: HashMap<usize, usize>,
    /// Chi-squared statistic against expected distribution.
    pub chi_squared: f64,
    /// Degrees of freedom.
    pub degrees_of_freedom: u32,
    /// P-value from chi-squared test.
    pub p_value: f64,
    /// Ratio of entries with even line counts.
    pub even_ratio: f64,
    /// Deviation from expected even ratio.
    pub even_ratio_deviation: f64,
    /// Ratio of entries with equal debit/credit counts.
    pub equal_split_ratio: f64,
    /// Deviation from expected equal split ratio.
    pub equal_split_deviation: f64,
    /// Average line count.
    pub avg_line_count: f64,
    /// Minimum line count.
    pub min_line_count: usize,
    /// Maximum line count.
    pub max_line_count: usize,
    /// Whether test passes.
    pub passes: bool,
}

/// Input for line item analysis.
#[derive(Debug, Clone)]
pub struct LineItemEntry {
    /// Total number of lines in the entry.
    pub line_count: usize,
    /// Number of debit lines.
    pub debit_count: usize,
    /// Number of credit lines.
    pub credit_count: usize,
}

/// Analyzer for line item distributions.
pub struct LineItemAnalyzer {
    /// Significance level for statistical tests.
    significance_level: f64,
}

impl LineItemAnalyzer {
    /// Create a new analyzer.
    pub fn new(significance_level: f64) -> Self {
        Self { significance_level }
    }

    /// Analyze line item distribution from entries.
    pub fn analyze(&self, entries: &[LineItemEntry]) -> EvalResult<LineItemAnalysis> {
        let n = entries.len();
        if n < 10 {
            return Err(EvalError::InsufficientData {
                required: 10,
                actual: n,
            });
        }

        // Count line count occurrences
        let mut line_count_distribution: HashMap<usize, usize> = HashMap::new();
        for entry in entries {
            *line_count_distribution.entry(entry.line_count).or_insert(0) += 1;
        }

        // Group into buckets matching expected distribution
        let buckets = self.bucket_counts(&line_count_distribution, n);

        // Chi-squared test
        let (chi_squared, p_value) = self.chi_squared_test(&buckets, n);

        // Even/odd analysis
        let even_count = entries.iter().filter(|e| e.line_count % 2 == 0).count();
        let even_ratio = even_count as f64 / n as f64;
        let even_ratio_deviation = (even_ratio - EXPECTED_EVEN_RATIO).abs();

        // Equal split analysis
        let equal_split_count = entries
            .iter()
            .filter(|e| e.debit_count == e.credit_count)
            .count();
        let equal_split_ratio = equal_split_count as f64 / n as f64;
        let equal_split_deviation = (equal_split_ratio - EXPECTED_EQUAL_SPLIT_RATIO).abs();

        // Basic statistics
        let line_counts: Vec<usize> = entries.iter().map(|e| e.line_count).collect();
        let avg_line_count = line_counts.iter().sum::<usize>() as f64 / n as f64;
        let min_line_count = *line_counts.iter().min().unwrap_or(&0);
        let max_line_count = *line_counts.iter().max().unwrap_or(&0);

        // Pass if chi-squared test passes and deviations are acceptable
        let passes = p_value >= self.significance_level
            && even_ratio_deviation < 0.10
            && equal_split_deviation < 0.10;

        Ok(LineItemAnalysis {
            sample_size: n,
            line_count_distribution,
            chi_squared,
            degrees_of_freedom: (EXPECTED_LINE_DISTRIBUTION.len() - 1) as u32,
            p_value,
            even_ratio,
            even_ratio_deviation,
            equal_split_ratio,
            equal_split_deviation,
            avg_line_count,
            min_line_count,
            max_line_count,
            passes,
        })
    }

    /// Bucket observed counts into expected distribution categories.
    fn bucket_counts(
        &self,
        distribution: &HashMap<usize, usize>,
        _total: usize,
    ) -> Vec<(usize, usize)> {
        let mut buckets = vec![
            (2, 0usize),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
            (10, 0),   // 10-99
            (100, 0),  // 100-999
            (1000, 0), // 1000+
        ];

        for (&count, &freq) in distribution {
            let bucket_idx = match count {
                2 => 0,
                3 => 1,
                4 => 2,
                5 => 3,
                6 => 4,
                7 => 5,
                8 => 6,
                9 => 7,
                10..=99 => 8,
                100..=999 => 9,
                _ if count >= 1000 => 10,
                _ => continue, // Skip 0, 1
            };
            buckets[bucket_idx].1 += freq;
        }

        buckets
    }

    /// Perform chi-squared test against expected distribution.
    fn chi_squared_test(&self, observed: &[(usize, usize)], n: usize) -> (f64, f64) {
        let n_f64 = n as f64;

        let chi_squared: f64 = observed
            .iter()
            .zip(EXPECTED_LINE_DISTRIBUTION.iter())
            .map(|((_, obs), (_, exp_prob))| {
                let expected = exp_prob * n_f64;
                if expected > 0.0 {
                    let obs_f64 = *obs as f64;
                    (obs_f64 - expected).powi(2) / expected
                } else {
                    0.0
                }
            })
            .sum();

        let df = (EXPECTED_LINE_DISTRIBUTION.len() - 1) as f64;
        let chi_sq_dist = ChiSquared::new(df).expect("df > 0 for chi-squared distribution");
        let p_value = 1.0 - chi_sq_dist.cdf(chi_squared);

        (chi_squared, p_value)
    }
}

impl Default for LineItemAnalyzer {
    fn default() -> Self {
        Self::new(0.05)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_entries(distribution: &[(usize, usize)]) -> Vec<LineItemEntry> {
        let mut entries = Vec::new();
        for &(line_count, count) in distribution {
            for _ in 0..count {
                entries.push(LineItemEntry {
                    line_count,
                    debit_count: line_count / 2,
                    credit_count: line_count - line_count / 2,
                });
            }
        }
        entries
    }

    #[test]
    fn test_line_item_analysis() {
        // Create distribution roughly matching expected
        let distribution = vec![
            (2, 607),
            (3, 58),
            (4, 166),
            (5, 31),
            (6, 33),
            (7, 11),
            (8, 19),
            (9, 4),
            (10, 63),
            (100, 8),
        ];

        let entries = create_test_entries(&distribution);
        let analyzer = LineItemAnalyzer::default();
        let result = analyzer.analyze(&entries).unwrap();

        assert_eq!(result.sample_size, 1000);
        assert!(result.avg_line_count > 2.0);
    }

    #[test]
    fn test_even_ratio() {
        let entries = vec![
            LineItemEntry {
                line_count: 2,
                debit_count: 1,
                credit_count: 1,
            },
            LineItemEntry {
                line_count: 4,
                debit_count: 2,
                credit_count: 2,
            },
            LineItemEntry {
                line_count: 6,
                debit_count: 3,
                credit_count: 3,
            },
            LineItemEntry {
                line_count: 8,
                debit_count: 4,
                credit_count: 4,
            },
            LineItemEntry {
                line_count: 10,
                debit_count: 5,
                credit_count: 5,
            },
            LineItemEntry {
                line_count: 2,
                debit_count: 1,
                credit_count: 1,
            },
            LineItemEntry {
                line_count: 4,
                debit_count: 2,
                credit_count: 2,
            },
            LineItemEntry {
                line_count: 6,
                debit_count: 3,
                credit_count: 3,
            },
            LineItemEntry {
                line_count: 3,
                debit_count: 2,
                credit_count: 1,
            },
            LineItemEntry {
                line_count: 5,
                debit_count: 3,
                credit_count: 2,
            },
        ];

        let analyzer = LineItemAnalyzer::default();
        let result = analyzer.analyze(&entries).unwrap();

        // 8 even out of 10
        assert!((result.even_ratio - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_insufficient_data() {
        let entries = vec![LineItemEntry {
            line_count: 2,
            debit_count: 1,
            credit_count: 1,
        }];
        let analyzer = LineItemAnalyzer::default();
        let result = analyzer.analyze(&entries);
        assert!(matches!(result, Err(EvalError::InsufficientData { .. })));
    }
}
