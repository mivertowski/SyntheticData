//! Anomaly injection realism evaluator.
//!
//! Validates that injected anomalies produce statistically detectable patterns,
//! cascade coherence is maintained, and multi-stage schemes share participants.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Anomaly data for realism validation.
#[derive(Debug, Clone)]
pub struct AnomalyData {
    /// Anomaly identifier.
    pub anomaly_id: String,
    /// Anomaly type/category.
    pub anomaly_type: String,
    /// The anomalous value.
    pub value: f64,
    /// Mean of the normal population.
    pub population_mean: f64,
    /// Standard deviation of the normal population.
    pub population_std: f64,
    /// Parent anomaly ID for cascaded anomalies.
    pub parent_anomaly_id: Option<String>,
    /// Scheme identifier for multi-stage schemes.
    pub scheme_id: Option<String>,
    /// Participants involved in this anomaly.
    pub participants: Vec<String>,
}

/// Thresholds for anomaly realism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRealismThresholds {
    /// Minimum average z-score for anomalies to be detectable.
    pub min_avg_z_score: f64,
    /// Minimum cascade coherence rate.
    pub min_cascade_coherence: f64,
    /// Minimum scheme participant consistency.
    pub min_scheme_consistency: f64,
}

impl Default for AnomalyRealismThresholds {
    fn default() -> Self {
        Self {
            min_avg_z_score: 2.0,
            min_cascade_coherence: 0.90,
            min_scheme_consistency: 0.85,
        }
    }
}

/// Results of anomaly realism evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRealismEvaluation {
    /// Statistical detectability: fraction of anomalies with z-score > 2.
    pub statistical_detectability: f64,
    /// Average z-score across all anomalies.
    pub avg_anomaly_z_score: f64,
    /// Cascade coherence: fraction of cascaded anomalies referencing valid parents.
    pub cascade_coherence: f64,
    /// Scheme participant consistency: fraction of schemes where participants overlap.
    pub scheme_participant_consistency: f64,
    /// Total anomalies evaluated.
    pub total_anomalies: usize,
    /// Cascaded anomalies count.
    pub cascaded_count: usize,
    /// Unique schemes count.
    pub scheme_count: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for anomaly injection realism.
pub struct AnomalyRealismEvaluator {
    thresholds: AnomalyRealismThresholds,
}

impl AnomalyRealismEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: AnomalyRealismThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: AnomalyRealismThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate anomaly data.
    pub fn evaluate(&self, anomalies: &[AnomalyData]) -> EvalResult<AnomalyRealismEvaluation> {
        let mut issues = Vec::new();

        if anomalies.is_empty() {
            return Ok(AnomalyRealismEvaluation {
                statistical_detectability: 1.0,
                avg_anomaly_z_score: 0.0,
                cascade_coherence: 1.0,
                scheme_participant_consistency: 1.0,
                total_anomalies: 0,
                cascaded_count: 0,
                scheme_count: 0,
                passes: true,
                issues: Vec::new(),
            });
        }

        // 1. Statistical detectability via z-scores
        let z_scores: Vec<f64> = anomalies
            .iter()
            .filter(|a| a.population_std > f64::EPSILON)
            .map(|a| (a.value - a.population_mean).abs() / a.population_std)
            .collect();

        let detectable = z_scores.iter().filter(|&&z| z > 2.0).count();
        let statistical_detectability = if z_scores.is_empty() {
            1.0
        } else {
            detectable as f64 / z_scores.len() as f64
        };

        let avg_anomaly_z_score = if z_scores.is_empty() {
            0.0
        } else {
            z_scores.iter().sum::<f64>() / z_scores.len() as f64
        };

        // 2. Cascade coherence: cascaded anomalies should reference valid parent IDs
        let all_ids: HashSet<&str> = anomalies.iter().map(|a| a.anomaly_id.as_str()).collect();
        let cascaded: Vec<&AnomalyData> = anomalies
            .iter()
            .filter(|a| a.parent_anomaly_id.is_some())
            .collect();
        let cascaded_count = cascaded.len();

        let cascade_valid = cascaded
            .iter()
            .filter(|a| {
                a.parent_anomaly_id
                    .as_ref()
                    .map(|pid| all_ids.contains(pid.as_str()))
                    .unwrap_or(false)
            })
            .count();
        let cascade_coherence = if cascaded_count == 0 {
            1.0
        } else {
            cascade_valid as f64 / cascaded_count as f64
        };

        // 3. Scheme participant consistency
        let mut schemes: HashMap<&str, Vec<HashSet<&str>>> = HashMap::new();
        for a in anomalies {
            if let Some(ref sid) = a.scheme_id {
                let participants: HashSet<&str> = a
                    .participants
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();
                schemes.entry(sid.as_str()).or_default().push(participants);
            }
        }
        let scheme_count = schemes.len();

        let consistent_schemes = schemes
            .values()
            .filter(|participant_sets| {
                if participant_sets.len() < 2 {
                    return true;
                }
                // Check that there's overlap between all participant sets
                let first = &participant_sets[0];
                participant_sets[1..]
                    .iter()
                    .all(|ps| !first.is_disjoint(ps))
            })
            .count();
        let scheme_participant_consistency = if scheme_count == 0 {
            1.0
        } else {
            consistent_schemes as f64 / scheme_count as f64
        };

        // Check thresholds
        if avg_anomaly_z_score < self.thresholds.min_avg_z_score && !z_scores.is_empty() {
            issues.push(format!(
                "Avg anomaly z-score {:.2} < {:.2}",
                avg_anomaly_z_score, self.thresholds.min_avg_z_score
            ));
        }
        if cascade_coherence < self.thresholds.min_cascade_coherence {
            issues.push(format!(
                "Cascade coherence {:.3} < {:.3}",
                cascade_coherence, self.thresholds.min_cascade_coherence
            ));
        }
        if scheme_participant_consistency < self.thresholds.min_scheme_consistency {
            issues.push(format!(
                "Scheme participant consistency {:.3} < {:.3}",
                scheme_participant_consistency, self.thresholds.min_scheme_consistency
            ));
        }

        let passes = issues.is_empty();

        Ok(AnomalyRealismEvaluation {
            statistical_detectability,
            avg_anomaly_z_score,
            cascade_coherence,
            scheme_participant_consistency,
            total_anomalies: anomalies.len(),
            cascaded_count,
            scheme_count,
            passes,
            issues,
        })
    }
}

impl Default for AnomalyRealismEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_detectable_anomalies() {
        let evaluator = AnomalyRealismEvaluator::new();
        let anomalies = vec![
            AnomalyData {
                anomaly_id: "A001".to_string(),
                anomaly_type: "unusual_amount".to_string(),
                value: 100_000.0,
                population_mean: 10_000.0,
                population_std: 5_000.0,
                parent_anomaly_id: None,
                scheme_id: None,
                participants: vec![],
            },
            AnomalyData {
                anomaly_id: "A002".to_string(),
                anomaly_type: "unusual_amount".to_string(),
                value: 50_000.0,
                population_mean: 10_000.0,
                population_std: 5_000.0,
                parent_anomaly_id: None,
                scheme_id: None,
                participants: vec![],
            },
        ];

        let result = evaluator.evaluate(&anomalies).unwrap();
        assert!(result.passes);
        assert!(result.avg_anomaly_z_score > 2.0);
    }

    #[test]
    fn test_undetectable_anomalies() {
        let evaluator = AnomalyRealismEvaluator::new();
        let anomalies = vec![AnomalyData {
            anomaly_id: "A001".to_string(),
            anomaly_type: "subtle".to_string(),
            value: 10_100.0, // Only slightly above mean
            population_mean: 10_000.0,
            population_std: 5_000.0,
            parent_anomaly_id: None,
            scheme_id: None,
            participants: vec![],
        }];

        let result = evaluator.evaluate(&anomalies).unwrap();
        assert!(!result.passes); // z-score < 2.0
    }

    #[test]
    fn test_cascade_coherence() {
        let evaluator = AnomalyRealismEvaluator::new();
        let anomalies = vec![
            AnomalyData {
                anomaly_id: "A001".to_string(),
                anomaly_type: "root".to_string(),
                value: 50_000.0,
                population_mean: 10_000.0,
                population_std: 5_000.0,
                parent_anomaly_id: None,
                scheme_id: None,
                participants: vec![],
            },
            AnomalyData {
                anomaly_id: "A002".to_string(),
                anomaly_type: "cascade".to_string(),
                value: 50_000.0,
                population_mean: 10_000.0,
                population_std: 5_000.0,
                parent_anomaly_id: Some("A001".to_string()), // Valid parent
                scheme_id: None,
                participants: vec![],
            },
        ];

        let result = evaluator.evaluate(&anomalies).unwrap();
        assert_eq!(result.cascade_coherence, 1.0);
    }

    #[test]
    fn test_empty() {
        let evaluator = AnomalyRealismEvaluator::new();
        let result = evaluator.evaluate(&[]).unwrap();
        assert!(result.passes);
    }
}
