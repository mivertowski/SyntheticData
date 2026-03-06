//! Train/test split validation.
//!
//! Validates split ratios, data leakage, and distribution preservation.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of split analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitAnalysis {
    /// Train set metrics.
    pub train_metrics: SplitMetrics,
    /// Validation set metrics (if present).
    pub validation_metrics: Option<SplitMetrics>,
    /// Test set metrics.
    pub test_metrics: SplitMetrics,
    /// Split ratio validation.
    pub ratio_valid: bool,
    /// Actual split ratios.
    pub actual_ratios: SplitRatios,
    /// Expected split ratios.
    pub expected_ratios: SplitRatios,
    /// Data leakage detected.
    pub leakage_detected: bool,
    /// Leakage details (if detected).
    pub leakage_details: Vec<String>,
    /// Class distribution preserved.
    pub distribution_preserved: bool,
    /// Distribution shift score (KL divergence).
    pub distribution_shift: f64,
    /// Overall validity.
    pub is_valid: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Metrics for a single split.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitMetrics {
    /// Number of samples.
    pub sample_count: usize,
    /// Class distribution.
    pub class_distribution: HashMap<String, f64>,
    /// Unique entity IDs (for leakage detection).
    pub unique_entities: usize,
    /// Date range (min, max).
    pub date_range: Option<(String, String)>,
}

/// Split ratios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitRatios {
    /// Train ratio.
    pub train: f64,
    /// Validation ratio.
    pub validation: f64,
    /// Test ratio.
    pub test: f64,
}

impl Default for SplitRatios {
    fn default() -> Self {
        Self {
            train: 0.7,
            validation: 0.15,
            test: 0.15,
        }
    }
}

/// Input for split analysis.
#[derive(Debug, Clone)]
pub struct SplitData {
    /// Train set data.
    pub train: SplitSetData,
    /// Validation set data (optional).
    pub validation: Option<SplitSetData>,
    /// Test set data.
    pub test: SplitSetData,
    /// Expected split ratios.
    pub expected_ratios: SplitRatios,
}

/// Data for a single split set.
#[derive(Debug, Clone, Default)]
pub struct SplitSetData {
    /// Number of samples.
    pub sample_count: usize,
    /// Class labels.
    pub labels: Vec<String>,
    /// Entity IDs (for leakage detection).
    pub entity_ids: Vec<String>,
    /// Dates (for temporal leakage detection).
    pub dates: Vec<String>,
}

/// Analyzer for train/test splits.
pub struct SplitAnalyzer {
    /// Tolerance for ratio validation.
    ratio_tolerance: f64,
    /// Maximum KL divergence for distribution preservation.
    max_kl_divergence: f64,
}

impl SplitAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            ratio_tolerance: 0.05,
            max_kl_divergence: 0.1,
        }
    }

    /// Analyze split quality.
    pub fn analyze(&self, data: &SplitData) -> EvalResult<SplitAnalysis> {
        let mut issues = Vec::new();

        // Calculate actual ratios
        let total = data.train.sample_count
            + data
                .validation
                .as_ref()
                .map(|v| v.sample_count)
                .unwrap_or(0)
            + data.test.sample_count;

        let actual_ratios = if total > 0 {
            SplitRatios {
                train: data.train.sample_count as f64 / total as f64,
                validation: data
                    .validation
                    .as_ref()
                    .map(|v| v.sample_count as f64 / total as f64)
                    .unwrap_or(0.0),
                test: data.test.sample_count as f64 / total as f64,
            }
        } else {
            SplitRatios::default()
        };

        // Validate ratios
        let ratio_valid = self.validate_ratios(&actual_ratios, &data.expected_ratios);
        if !ratio_valid {
            issues.push(format!(
                "Split ratios deviate from expected: actual {:.2}/{:.2}/{:.2}, expected {:.2}/{:.2}/{:.2}",
                actual_ratios.train,
                actual_ratios.validation,
                actual_ratios.test,
                data.expected_ratios.train,
                data.expected_ratios.validation,
                data.expected_ratios.test
            ));
        }

        // Check for data leakage (entity overlap)
        let (leakage_detected, leakage_details) = self.check_leakage(data);
        if leakage_detected {
            issues.extend(leakage_details.clone());
        }

        // Calculate distribution metrics
        let train_metrics = self.calculate_metrics(&data.train);
        let validation_metrics = data.validation.as_ref().map(|v| self.calculate_metrics(v));
        let test_metrics = self.calculate_metrics(&data.test);

        // Check distribution preservation
        let (distribution_preserved, distribution_shift) =
            self.check_distribution(&train_metrics, &test_metrics);
        if !distribution_preserved {
            issues.push(format!(
                "Class distribution shift detected: KL divergence = {distribution_shift:.4}"
            ));
        }

        let is_valid = ratio_valid && !leakage_detected && distribution_preserved;

        Ok(SplitAnalysis {
            train_metrics,
            validation_metrics,
            test_metrics,
            ratio_valid,
            actual_ratios,
            expected_ratios: data.expected_ratios.clone(),
            leakage_detected,
            leakage_details,
            distribution_preserved,
            distribution_shift,
            is_valid,
            issues,
        })
    }

    /// Validate split ratios against expected.
    fn validate_ratios(&self, actual: &SplitRatios, expected: &SplitRatios) -> bool {
        (actual.train - expected.train).abs() <= self.ratio_tolerance
            && (actual.validation - expected.validation).abs() <= self.ratio_tolerance
            && (actual.test - expected.test).abs() <= self.ratio_tolerance
    }

    /// Check for data leakage between splits.
    fn check_leakage(&self, data: &SplitData) -> (bool, Vec<String>) {
        let mut leakage = false;
        let mut details = Vec::new();

        let train_entities: std::collections::HashSet<_> = data.train.entity_ids.iter().collect();
        let test_entities: std::collections::HashSet<_> = data.test.entity_ids.iter().collect();

        let overlap: Vec<_> = train_entities.intersection(&test_entities).collect();
        if !overlap.is_empty() {
            leakage = true;
            details.push(format!(
                "Entity leakage: {} entities appear in both train and test",
                overlap.len()
            ));
        }

        // Check temporal leakage (test dates before train dates)
        if !data.train.dates.is_empty() && !data.test.dates.is_empty() {
            let train_max = data.train.dates.iter().max();
            let test_min = data.test.dates.iter().min();

            if let (Some(train_max), Some(test_min)) = (train_max, test_min) {
                if test_min < train_max {
                    leakage = true;
                    details.push(format!(
                        "Temporal leakage: test min date {test_min} < train max date {train_max}"
                    ));
                }
            }
        }

        if let Some(ref val) = data.validation {
            let val_entities: std::collections::HashSet<_> = val.entity_ids.iter().collect();

            let train_val_overlap: Vec<_> = train_entities.intersection(&val_entities).collect();
            if !train_val_overlap.is_empty() {
                leakage = true;
                details.push(format!(
                    "Entity leakage: {} entities appear in both train and validation",
                    train_val_overlap.len()
                ));
            }

            let val_test_overlap: Vec<_> = val_entities.intersection(&test_entities).collect();
            if !val_test_overlap.is_empty() {
                leakage = true;
                details.push(format!(
                    "Entity leakage: {} entities appear in both validation and test",
                    val_test_overlap.len()
                ));
            }
        }

        (leakage, details)
    }

    /// Calculate metrics for a split set.
    fn calculate_metrics(&self, data: &SplitSetData) -> SplitMetrics {
        let mut class_counts: HashMap<String, usize> = HashMap::new();
        for label in &data.labels {
            *class_counts.entry(label.clone()).or_insert(0) += 1;
        }

        let total = data.labels.len();
        let class_distribution: HashMap<String, f64> = class_counts
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    if total > 0 {
                        *v as f64 / total as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect();

        let unique_entities = data
            .entity_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();

        let date_range = if !data.dates.is_empty() {
            let min = data.dates.iter().min().cloned();
            let max = data.dates.iter().max().cloned();
            match (min, max) {
                (Some(min), Some(max)) => Some((min, max)),
                _ => None,
            }
        } else {
            None
        };

        SplitMetrics {
            sample_count: data.sample_count,
            class_distribution,
            unique_entities,
            date_range,
        }
    }

    /// Check distribution preservation between train and test.
    fn check_distribution(&self, train: &SplitMetrics, test: &SplitMetrics) -> (bool, f64) {
        if train.class_distribution.is_empty() || test.class_distribution.is_empty() {
            return (true, 0.0);
        }

        // Calculate KL divergence: KL(P||Q) = sum(P(x) * log(P(x)/Q(x)))
        let mut kl_divergence = 0.0;
        let epsilon = 1e-10;

        for (class, train_prob) in &train.class_distribution {
            let test_prob = test.class_distribution.get(class).unwrap_or(&epsilon);
            let p = *train_prob + epsilon;
            let q = *test_prob + epsilon;
            kl_divergence += p * (p / q).ln();
        }

        // Also account for classes in test not in train
        for (class, test_prob) in &test.class_distribution {
            if !train.class_distribution.contains_key(class) {
                let p = epsilon;
                let q = *test_prob + epsilon;
                kl_divergence += p * (p / q).ln();
            }
        }

        let preserved = kl_divergence <= self.max_kl_divergence;
        (preserved, kl_divergence)
    }
}

impl Default for SplitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_split() {
        let data = SplitData {
            train: SplitSetData {
                sample_count: 70,
                labels: vec!["A".to_string(); 50]
                    .into_iter()
                    .chain(vec!["B".to_string(); 20])
                    .collect(),
                entity_ids: (0..70).map(|i| format!("E{}", i)).collect(),
                dates: vec![],
            },
            validation: Some(SplitSetData {
                sample_count: 15,
                labels: vec!["A".to_string(); 11]
                    .into_iter()
                    .chain(vec!["B".to_string(); 4])
                    .collect(),
                entity_ids: (70..85).map(|i| format!("E{}", i)).collect(),
                dates: vec![],
            }),
            test: SplitSetData {
                sample_count: 15,
                labels: vec!["A".to_string(); 11]
                    .into_iter()
                    .chain(vec!["B".to_string(); 4])
                    .collect(),
                entity_ids: (85..100).map(|i| format!("E{}", i)).collect(),
                dates: vec![],
            },
            expected_ratios: SplitRatios::default(),
        };

        let analyzer = SplitAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert!(result.ratio_valid);
        assert!(!result.leakage_detected);
        assert!(result.is_valid);
    }

    #[test]
    fn test_entity_leakage() {
        let data = SplitData {
            train: SplitSetData {
                sample_count: 70,
                labels: vec![],
                entity_ids: vec!["E1".to_string(), "E2".to_string(), "E3".to_string()],
                dates: vec![],
            },
            validation: None,
            test: SplitSetData {
                sample_count: 30,
                labels: vec![],
                entity_ids: vec!["E1".to_string(), "E4".to_string()], // E1 is in both
                dates: vec![],
            },
            expected_ratios: SplitRatios {
                train: 0.7,
                validation: 0.0,
                test: 0.3,
            },
        };

        let analyzer = SplitAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert!(result.leakage_detected);
        assert!(!result.is_valid);
    }
}
