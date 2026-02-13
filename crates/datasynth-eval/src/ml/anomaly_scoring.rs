//! Anomaly scoring evaluation.
//!
//! Computes statistical anomaly scores and validates that injected anomalies
//! are detectable via AUC-ROC and z-score separability analysis.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// A single record with an anomaly score and ground truth label.
#[derive(Debug, Clone)]
pub struct ScoredRecord {
    /// Unique identifier for this record.
    pub record_id: String,
    /// Anomaly score (higher = more anomalous).
    pub score: f64,
    /// Ground truth: true if this record is an anomaly.
    pub is_anomaly: bool,
}

/// Thresholds for anomaly scoring analysis.
#[derive(Debug, Clone)]
pub struct AnomalyScoringThresholds {
    /// Minimum AUC-ROC for anomaly separability.
    pub min_anomaly_separability: f64,
}

impl Default for AnomalyScoringThresholds {
    fn default() -> Self {
        Self {
            min_anomaly_separability: 0.70,
        }
    }
}

/// Results of anomaly scoring analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScoringAnalysis {
    /// AUC-ROC measuring anomaly separability (0.5 = random, 1.0 = perfect).
    pub anomaly_separability: f64,
    /// Average anomaly score for anomaly records.
    pub avg_anomaly_score: f64,
    /// Average anomaly score for normal records.
    pub avg_normal_score: f64,
    /// Per-type separability metrics (reserved for future use).
    pub per_type_separability: Vec<(String, f64)>,
    /// Total number of records analyzed.
    pub total_records: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for anomaly scoring quality.
pub struct AnomalyScoringAnalyzer {
    thresholds: AnomalyScoringThresholds,
}

impl AnomalyScoringAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: AnomalyScoringThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: AnomalyScoringThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze anomaly scoring quality.
    pub fn analyze(&self, records: &[ScoredRecord]) -> EvalResult<AnomalyScoringAnalysis> {
        let mut issues = Vec::new();
        let total_records = records.len();

        if records.is_empty() {
            return Ok(AnomalyScoringAnalysis {
                anomaly_separability: 0.0,
                avg_anomaly_score: 0.0,
                avg_normal_score: 0.0,
                per_type_separability: Vec::new(),
                total_records: 0,
                passes: true,
                issues: vec!["No records provided".to_string()],
            });
        }

        // Separate anomaly and normal records
        let anomaly_scores: Vec<f64> = records
            .iter()
            .filter(|r| r.is_anomaly)
            .map(|r| r.score)
            .collect();
        let normal_scores: Vec<f64> = records
            .iter()
            .filter(|r| !r.is_anomaly)
            .map(|r| r.score)
            .collect();

        let avg_anomaly_score = if anomaly_scores.is_empty() {
            0.0
        } else {
            anomaly_scores.iter().sum::<f64>() / anomaly_scores.len() as f64
        };

        let avg_normal_score = if normal_scores.is_empty() {
            0.0
        } else {
            normal_scores.iter().sum::<f64>() / normal_scores.len() as f64
        };

        // Compute AUC-ROC via trapezoidal integration
        let anomaly_separability = if anomaly_scores.is_empty() || normal_scores.is_empty() {
            issues.push("Need both anomaly and normal records for AUC-ROC".to_string());
            0.5
        } else {
            self.compute_auc_roc(records)
        };

        // Check thresholds
        if anomaly_separability < self.thresholds.min_anomaly_separability {
            issues.push(format!(
                "Anomaly separability {:.4} < {:.4} (threshold)",
                anomaly_separability, self.thresholds.min_anomaly_separability
            ));
        }

        let passes = issues.is_empty();

        Ok(AnomalyScoringAnalysis {
            anomaly_separability,
            avg_anomaly_score,
            avg_normal_score,
            per_type_separability: Vec::new(),
            total_records,
            passes,
            issues,
        })
    }

    /// Compute AUC-ROC via trapezoidal integration.
    ///
    /// Sorts records by score descending, then sweeps thresholds to compute
    /// TPR/FPR pairs and integrates the ROC curve.
    fn compute_auc_roc(&self, records: &[ScoredRecord]) -> f64 {
        let total_positives = records.iter().filter(|r| r.is_anomaly).count();
        let total_negatives = records.len() - total_positives;

        if total_positives == 0 || total_negatives == 0 {
            return 0.5;
        }

        // Sort by score descending
        let mut sorted: Vec<&ScoredRecord> = records.iter().collect();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut tp = 0usize;
        let mut fp = 0usize;
        let mut auc = 0.0;
        let mut prev_fpr = 0.0;
        let mut prev_tpr = 0.0;

        for record in &sorted {
            if record.is_anomaly {
                tp += 1;
            } else {
                fp += 1;
            }

            let tpr = tp as f64 / total_positives as f64;
            let fpr = fp as f64 / total_negatives as f64;

            // Trapezoidal rule
            auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;

            prev_fpr = fpr;
            prev_tpr = tpr;
        }

        auc
    }
}

impl Default for AnomalyScoringAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_anomaly_scoring() {
        let records = vec![
            ScoredRecord {
                record_id: "r1".to_string(),
                score: 0.9,
                is_anomaly: true,
            },
            ScoredRecord {
                record_id: "r2".to_string(),
                score: 0.85,
                is_anomaly: true,
            },
            ScoredRecord {
                record_id: "r3".to_string(),
                score: 0.1,
                is_anomaly: false,
            },
            ScoredRecord {
                record_id: "r4".to_string(),
                score: 0.15,
                is_anomaly: false,
            },
            ScoredRecord {
                record_id: "r5".to_string(),
                score: 0.05,
                is_anomaly: false,
            },
        ];

        let analyzer = AnomalyScoringAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.total_records, 5);
        assert!(result.anomaly_separability > 0.7);
        assert!(result.avg_anomaly_score > result.avg_normal_score);
        assert!(result.passes);
    }

    #[test]
    fn test_invalid_anomaly_scoring() {
        // Scores are inverted: anomalies have lower scores than normals
        let records = vec![
            ScoredRecord {
                record_id: "r1".to_string(),
                score: 0.1,
                is_anomaly: true,
            },
            ScoredRecord {
                record_id: "r2".to_string(),
                score: 0.05,
                is_anomaly: true,
            },
            ScoredRecord {
                record_id: "r3".to_string(),
                score: 0.9,
                is_anomaly: false,
            },
            ScoredRecord {
                record_id: "r4".to_string(),
                score: 0.85,
                is_anomaly: false,
            },
        ];

        let analyzer = AnomalyScoringAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        assert!(result.anomaly_separability < 0.7);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_records() {
        let analyzer = AnomalyScoringAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_records, 0);
        assert_eq!(result.anomaly_separability, 0.0);
    }
}
