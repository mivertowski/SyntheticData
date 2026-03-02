use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input data describing configured vs actual fraud generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudPackData {
    pub configured_fraud_rate: f64,
    pub actual_fraud_count: usize,
    pub total_records: usize,
    pub configured_scheme_types: Vec<String>,
    pub actual_scheme_types: Vec<String>,
    pub scheme_type_counts: HashMap<String, usize>,
}

/// Thresholds for fraud pack effectiveness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudPackThresholds {
    /// Minimum acceptable rate accuracy (1.0 - |configured - actual| / configured).
    /// Default: 0.70.
    pub min_rate_accuracy: f64,
    /// Minimum fraction of configured scheme types that appear in output.
    /// Default: 0.80.
    pub min_scheme_coverage: f64,
    /// Minimum Shannon entropy of scheme distribution (higher = more uniform).
    /// Default: 0.5.
    pub min_distribution_entropy: f64,
}

impl Default for FraudPackThresholds {
    fn default() -> Self {
        Self {
            min_rate_accuracy: 0.70,
            min_scheme_coverage: 0.80,
            min_distribution_entropy: 0.5,
        }
    }
}

/// Result of fraud pack effectiveness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudPackAnalysis {
    pub configured_rate: f64,
    pub actual_rate: f64,
    /// Rate accuracy: 1.0 - |configured - actual| / configured.
    pub rate_accuracy: f64,
    /// Fraction of configured scheme types that appear in output.
    pub scheme_coverage: f64,
    /// Shannon entropy of scheme distribution (normalized by log2(n_types)).
    pub scheme_distribution_entropy: f64,
    pub passes: bool,
    pub issues: Vec<String>,
}

pub struct FraudPackAnalyzer {
    thresholds: FraudPackThresholds,
}

impl FraudPackAnalyzer {
    pub fn new(thresholds: FraudPackThresholds) -> Self {
        Self { thresholds }
    }

    pub fn with_defaults() -> Self {
        Self::new(FraudPackThresholds::default())
    }

    pub fn analyze(&self, data: &FraudPackData) -> FraudPackAnalysis {
        let mut issues = Vec::new();

        // Actual fraud rate
        let actual_rate = if data.total_records > 0 {
            data.actual_fraud_count as f64 / data.total_records as f64
        } else {
            0.0
        };

        // Rate accuracy
        let rate_accuracy = if data.configured_fraud_rate > 0.0 {
            1.0 - ((data.configured_fraud_rate - actual_rate).abs() / data.configured_fraud_rate)
        } else if actual_rate == 0.0 {
            1.0 // Both zero = perfect accuracy
        } else {
            0.0 // Configured 0 but got fraud = bad
        };

        // Scheme coverage: fraction of configured types that appear in actual
        let scheme_coverage = if data.configured_scheme_types.is_empty() {
            1.0
        } else {
            let covered = data
                .configured_scheme_types
                .iter()
                .filter(|t| data.actual_scheme_types.contains(t))
                .count();
            covered as f64 / data.configured_scheme_types.len() as f64
        };

        // Shannon entropy of scheme distribution (normalized)
        let scheme_distribution_entropy = {
            let total: usize = data.scheme_type_counts.values().sum();
            if total == 0 || data.scheme_type_counts.len() <= 1 {
                0.0
            } else {
                let mut entropy = 0.0f64;
                for &count in data.scheme_type_counts.values() {
                    if count > 0 {
                        let p = count as f64 / total as f64;
                        entropy -= p * p.log2();
                    }
                }
                // Normalize by max possible entropy (uniform distribution)
                let max_entropy = (data.scheme_type_counts.len() as f64).log2();
                if max_entropy > 0.0 {
                    entropy / max_entropy
                } else {
                    0.0
                }
            }
        };

        // Check thresholds
        if rate_accuracy < self.thresholds.min_rate_accuracy {
            issues.push(format!(
                "Rate accuracy {:.3} < threshold {:.3} (configured={:.4}, actual={:.4})",
                rate_accuracy, self.thresholds.min_rate_accuracy, data.configured_fraud_rate, actual_rate
            ));
        }
        if scheme_coverage < self.thresholds.min_scheme_coverage {
            issues.push(format!(
                "Scheme coverage {:.2} < threshold {:.2}",
                scheme_coverage, self.thresholds.min_scheme_coverage
            ));
        }
        if scheme_distribution_entropy < self.thresholds.min_distribution_entropy {
            issues.push(format!(
                "Distribution entropy {:.3} < threshold {:.3}",
                scheme_distribution_entropy, self.thresholds.min_distribution_entropy
            ));
        }

        let passes = rate_accuracy >= self.thresholds.min_rate_accuracy
            && scheme_coverage >= self.thresholds.min_scheme_coverage
            && scheme_distribution_entropy >= self.thresholds.min_distribution_entropy;

        FraudPackAnalysis {
            configured_rate: data.configured_fraud_rate,
            actual_rate,
            rate_accuracy,
            scheme_coverage,
            scheme_distribution_entropy,
            passes,
            issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_fraud_pack() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.05,
            actual_fraud_count: 50,
            total_records: 1000,
            configured_scheme_types: vec!["DuplicatePayment".into(), "SplitTransaction".into()],
            actual_scheme_types: vec!["DuplicatePayment".into(), "SplitTransaction".into()],
            scheme_type_counts: HashMap::from([
                ("DuplicatePayment".into(), 25),
                ("SplitTransaction".into(), 25),
            ]),
        };
        let result = analyzer.analyze(&data);
        assert!(result.passes, "issues: {:?}", result.issues);
        assert_eq!(result.rate_accuracy, 1.0);
        assert_eq!(result.scheme_coverage, 1.0);
        assert!(result.scheme_distribution_entropy > 0.9); // Uniform = max entropy
    }

    #[test]
    fn test_rate_deviation_detected() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.10,
            actual_fraud_count: 20,
            total_records: 1000,
            configured_scheme_types: vec!["DuplicatePayment".into()],
            actual_scheme_types: vec!["DuplicatePayment".into()],
            scheme_type_counts: HashMap::from([("DuplicatePayment".into(), 20)]),
        };
        let result = analyzer.analyze(&data);
        // actual=0.02, configured=0.10, accuracy=1-0.08/0.10=0.2
        assert!(!result.passes);
        assert!(result.rate_accuracy < 0.7);
    }

    #[test]
    fn test_missing_scheme_types() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.05,
            actual_fraud_count: 50,
            total_records: 1000,
            configured_scheme_types: vec![
                "DuplicatePayment".into(),
                "SplitTransaction".into(),
                "GhostEmployee".into(),
                "RoundTripping".into(),
                "FictitiousTransaction".into(),
            ],
            actual_scheme_types: vec!["DuplicatePayment".into()],
            scheme_type_counts: HashMap::from([("DuplicatePayment".into(), 50)]),
        };
        let result = analyzer.analyze(&data);
        assert!(!result.passes);
        assert_eq!(result.scheme_coverage, 0.2); // Only 1 of 5
    }

    #[test]
    fn test_zero_records_handles_gracefully() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.05,
            actual_fraud_count: 0,
            total_records: 0,
            configured_scheme_types: vec!["DuplicatePayment".into()],
            actual_scheme_types: vec![],
            scheme_type_counts: HashMap::new(),
        };
        let result = analyzer.analyze(&data);
        // Should not panic
        assert!(!result.passes);
    }

    #[test]
    fn test_uniform_distribution_high_entropy() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.05,
            actual_fraud_count: 100,
            total_records: 2000,
            configured_scheme_types: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            actual_scheme_types: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            scheme_type_counts: HashMap::from([
                ("A".into(), 25),
                ("B".into(), 25),
                ("C".into(), 25),
                ("D".into(), 25),
            ]),
        };
        let result = analyzer.analyze(&data);
        assert!(result.scheme_distribution_entropy > 0.99);
        assert!(result.passes, "issues: {:?}", result.issues);
    }

    #[test]
    fn test_skewed_distribution_low_entropy() {
        let analyzer = FraudPackAnalyzer::with_defaults();
        let data = FraudPackData {
            configured_fraud_rate: 0.05,
            actual_fraud_count: 100,
            total_records: 2000,
            configured_scheme_types: vec!["A".into(), "B".into(), "C".into()],
            actual_scheme_types: vec!["A".into(), "B".into(), "C".into()],
            scheme_type_counts: HashMap::from([
                ("A".into(), 98),
                ("B".into(), 1),
                ("C".into(), 1),
            ]),
        };
        let result = analyzer.analyze(&data);
        assert!(result.scheme_distribution_entropy < 0.5);
    }
}
