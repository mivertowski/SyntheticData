//! Label quality analysis.
//!
//! Analyzes label distributions and quality for supervised ML tasks.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of label analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelAnalysis {
    /// Total samples.
    pub total_samples: usize,
    /// Samples with labels.
    pub labeled_samples: usize,
    /// Label coverage (labeled / total).
    pub label_coverage: f64,
    /// Anomaly rate (for binary anomaly detection).
    pub anomaly_rate: f64,
    /// Class distribution.
    pub class_distribution: Vec<LabelDistribution>,
    /// Imbalance ratio (max class / min class).
    pub imbalance_ratio: f64,
    /// Anomaly type breakdown.
    pub anomaly_types: HashMap<String, usize>,
    /// Label quality score (0.0-1.0).
    pub quality_score: f64,
    /// Issues with labels.
    pub issues: Vec<String>,
}

/// Distribution for a single label class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelDistribution {
    /// Class name/value.
    pub class_name: String,
    /// Count of samples.
    pub count: usize,
    /// Percentage of total.
    pub percentage: f64,
}

/// Input for label analysis.
#[derive(Debug, Clone, Default)]
pub struct LabelData {
    /// Binary labels (true = anomaly/positive).
    pub binary_labels: Vec<Option<bool>>,
    /// Multi-class labels.
    pub multiclass_labels: Vec<Option<String>>,
    /// Anomaly type labels (for anomalies).
    pub anomaly_types: Vec<Option<String>>,
}

/// Analyzer for label quality.
pub struct LabelAnalyzer {
    /// Minimum acceptable anomaly rate.
    min_anomaly_rate: f64,
    /// Maximum acceptable anomaly rate.
    max_anomaly_rate: f64,
    /// Maximum acceptable imbalance ratio.
    max_imbalance_ratio: f64,
}

impl LabelAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            min_anomaly_rate: 0.01,
            max_anomaly_rate: 0.20,
            max_imbalance_ratio: 100.0,
        }
    }

    /// Analyze label quality.
    pub fn analyze(&self, data: &LabelData) -> EvalResult<LabelAnalysis> {
        let mut issues = Vec::new();

        // Determine total samples from the larger of binary or multiclass
        let total_samples = data.binary_labels.len().max(data.multiclass_labels.len());

        // Analyze binary labels
        let (anomaly_rate, labeled_binary) = if !data.binary_labels.is_empty() {
            let present: Vec<bool> = data.binary_labels.iter().filter_map(|v| *v).collect();
            let anomalies = present.iter().filter(|v| **v).count();
            let rate = if !present.is_empty() {
                anomalies as f64 / present.len() as f64
            } else {
                0.0
            };
            (rate, present.len())
        } else {
            (0.0, 0)
        };

        // Analyze multiclass labels
        let (class_distribution, labeled_multi) = if !data.multiclass_labels.is_empty() {
            let present: Vec<&String> = data
                .multiclass_labels
                .iter()
                .filter_map(|v| v.as_ref())
                .collect();

            let mut counts: HashMap<&str, usize> = HashMap::new();
            for label in &present {
                *counts.entry(label.as_str()).or_insert(0) += 1;
            }

            let total = present.len();
            let distribution: Vec<LabelDistribution> = counts
                .iter()
                .map(|(name, count)| LabelDistribution {
                    class_name: name.to_string(),
                    count: *count,
                    percentage: if total > 0 {
                        *count as f64 / total as f64
                    } else {
                        0.0
                    },
                })
                .collect();

            (distribution, present.len())
        } else {
            (Vec::new(), 0)
        };

        // Calculate label coverage
        let labeled_samples = labeled_binary.max(labeled_multi);
        let label_coverage = if total_samples > 0 {
            labeled_samples as f64 / total_samples as f64
        } else {
            1.0
        };

        // Calculate imbalance ratio
        let imbalance_ratio = if !class_distribution.is_empty() {
            let max_count = class_distribution
                .iter()
                .map(|d| d.count)
                .max()
                .unwrap_or(1);
            let min_count = class_distribution
                .iter()
                .map(|d| d.count)
                .filter(|c| *c > 0)
                .min()
                .unwrap_or(1);
            max_count as f64 / min_count as f64
        } else if labeled_binary > 0 {
            let anomalies = (anomaly_rate * labeled_binary as f64) as usize;
            let normals = labeled_binary - anomalies;
            if anomalies > 0 && normals > 0 {
                (anomalies.max(normals) as f64) / (anomalies.min(normals) as f64)
            } else {
                f64::INFINITY
            }
        } else {
            1.0
        };

        // Analyze anomaly types
        let mut anomaly_types: HashMap<String, usize> = HashMap::new();
        for atype in data.anomaly_types.iter().filter_map(|v| v.as_ref()) {
            *anomaly_types.entry(atype.clone()).or_insert(0) += 1;
        }

        // Check for issues
        if label_coverage < 0.99 {
            issues.push(format!(
                "Low label coverage: {:.2}%",
                label_coverage * 100.0
            ));
        }

        if anomaly_rate < self.min_anomaly_rate && labeled_binary > 0 {
            issues.push(format!(
                "Anomaly rate too low: {:.2}% (min: {:.2}%)",
                anomaly_rate * 100.0,
                self.min_anomaly_rate * 100.0
            ));
        }

        if anomaly_rate > self.max_anomaly_rate {
            issues.push(format!(
                "Anomaly rate too high: {:.2}% (max: {:.2}%)",
                anomaly_rate * 100.0,
                self.max_anomaly_rate * 100.0
            ));
        }

        if imbalance_ratio > self.max_imbalance_ratio {
            issues.push(format!("High class imbalance: {imbalance_ratio:.1}:1"));
        }

        // Calculate quality score
        let mut quality_factors = Vec::new();

        // Coverage factor
        quality_factors.push(label_coverage);

        // Anomaly rate factor (penalize if outside ideal range)
        if labeled_binary > 0 {
            let rate_score =
                if anomaly_rate >= self.min_anomaly_rate && anomaly_rate <= self.max_anomaly_rate {
                    1.0
                } else if anomaly_rate < self.min_anomaly_rate {
                    anomaly_rate / self.min_anomaly_rate
                } else {
                    self.max_anomaly_rate / anomaly_rate
                };
            quality_factors.push(rate_score.min(1.0));
        }

        // Imbalance factor
        if imbalance_ratio > 1.0 {
            let balance_score = (1.0 / imbalance_ratio.sqrt()).min(1.0);
            quality_factors.push(balance_score);
        }

        let quality_score = if quality_factors.is_empty() {
            1.0
        } else {
            quality_factors.iter().sum::<f64>() / quality_factors.len() as f64
        };

        Ok(LabelAnalysis {
            total_samples,
            labeled_samples,
            label_coverage,
            anomaly_rate,
            class_distribution,
            imbalance_ratio,
            anomaly_types,
            quality_score,
            issues,
        })
    }
}

impl Default for LabelAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_labels() {
        let data = LabelData {
            binary_labels: vec![
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(true),
                Some(true),
            ],
            multiclass_labels: vec![],
            anomaly_types: vec![],
        };

        let analyzer = LabelAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.total_samples, 10);
        assert_eq!(result.label_coverage, 1.0);
        assert!((result.anomaly_rate - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_multiclass_labels() {
        let data = LabelData {
            binary_labels: vec![],
            multiclass_labels: vec![
                Some("A".to_string()),
                Some("A".to_string()),
                Some("B".to_string()),
                Some("C".to_string()),
            ],
            anomaly_types: vec![],
        };

        let analyzer = LabelAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.class_distribution.len(), 3);
        assert!(result.imbalance_ratio >= 1.0);
    }

    #[test]
    fn test_missing_labels() {
        let data = LabelData {
            binary_labels: vec![Some(true), None, Some(false), None],
            multiclass_labels: vec![],
            anomaly_types: vec![],
        };

        let analyzer = LabelAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.labeled_samples, 2);
        assert!(result.label_coverage < 1.0);
    }
}
