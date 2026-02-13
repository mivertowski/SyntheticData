//! Cross-modal consistency evaluation.
//!
//! Measures consistency between graph and tabular feature representations
//! for the same entities, using Pearson correlation for corresponding
//! feature dimensions.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Modal data for a single entity with tabular and graph feature vectors.
#[derive(Debug, Clone)]
pub struct EntityModalData {
    /// Entity identifier.
    pub entity_id: String,
    /// Tabular feature vector.
    pub tabular_features: Vec<f64>,
    /// Graph-derived feature vector.
    pub graph_features: Vec<f64>,
}

/// Thresholds for cross-modal consistency analysis.
#[derive(Debug, Clone)]
pub struct CrossModalThresholds {
    /// Minimum consistency score.
    pub min_consistency: f64,
}

impl Default for CrossModalThresholds {
    fn default() -> Self {
        Self {
            min_consistency: 0.60,
        }
    }
}

/// Results of cross-modal consistency analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalAnalysis {
    /// Average Pearson correlation between tabular and graph features.
    pub tabular_graph_correlation: f64,
    /// Overall consistency score (0.0-1.0).
    pub consistency_score: f64,
    /// Total number of entities analyzed.
    pub total_entities: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for cross-modal consistency.
pub struct CrossModalAnalyzer {
    thresholds: CrossModalThresholds,
}

impl CrossModalAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: CrossModalThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: CrossModalThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze cross-modal consistency.
    pub fn analyze(&self, entities: &[EntityModalData]) -> EvalResult<CrossModalAnalysis> {
        let mut issues = Vec::new();
        let total_entities = entities.len();

        if entities.is_empty() {
            return Ok(CrossModalAnalysis {
                tabular_graph_correlation: 0.0,
                consistency_score: 0.0,
                total_entities: 0,
                passes: true,
                issues: vec!["No entities provided".to_string()],
            });
        }

        // Determine the common feature dimension
        let min_dim = entities
            .iter()
            .map(|e| e.tabular_features.len().min(e.graph_features.len()))
            .min()
            .unwrap_or(0);

        if min_dim == 0 {
            return Ok(CrossModalAnalysis {
                tabular_graph_correlation: 0.0,
                consistency_score: 0.0,
                total_entities,
                passes: false,
                issues: vec!["No common feature dimensions".to_string()],
            });
        }

        // Compute per-dimension Pearson correlation across entities
        let mut correlations = Vec::new();

        for dim in 0..min_dim {
            let tabular_vals: Vec<f64> = entities.iter().map(|e| e.tabular_features[dim]).collect();
            let graph_vals: Vec<f64> = entities.iter().map(|e| e.graph_features[dim]).collect();

            if let Some(corr) = pearson_correlation(&tabular_vals, &graph_vals) {
                correlations.push(corr);
            }
        }

        let tabular_graph_correlation = if correlations.is_empty() {
            0.0
        } else {
            correlations.iter().sum::<f64>() / correlations.len() as f64
        };

        // Consistency score: map correlation from [-1, 1] to [0, 1]
        let consistency_score = ((tabular_graph_correlation + 1.0) / 2.0).clamp(0.0, 1.0);

        if consistency_score < self.thresholds.min_consistency {
            issues.push(format!(
                "Cross-modal consistency {:.4} < {:.4} (threshold)",
                consistency_score, self.thresholds.min_consistency
            ));
        }

        let passes = issues.is_empty();

        Ok(CrossModalAnalysis {
            tabular_graph_correlation,
            consistency_score,
            total_entities,
            passes,
            issues,
        })
    }
}

/// Compute Pearson correlation between two vectors.
fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 3 {
        return None;
    }

    let mean_x = x[..n].iter().sum::<f64>() / n as f64;
    let mean_y = y[..n].iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-12 {
        return None;
    }

    Some(cov / denom)
}

impl Default for CrossModalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_modalities() {
        let entities = vec![
            EntityModalData {
                entity_id: "e1".into(),
                tabular_features: vec![1.0, 2.0, 3.0],
                graph_features: vec![1.1, 2.1, 3.1],
            },
            EntityModalData {
                entity_id: "e2".into(),
                tabular_features: vec![4.0, 5.0, 6.0],
                graph_features: vec![4.2, 5.1, 6.3],
            },
            EntityModalData {
                entity_id: "e3".into(),
                tabular_features: vec![7.0, 8.0, 9.0],
                graph_features: vec![7.1, 8.2, 9.1],
            },
            EntityModalData {
                entity_id: "e4".into(),
                tabular_features: vec![10.0, 11.0, 12.0],
                graph_features: vec![10.0, 11.1, 12.2],
            },
        ];

        let analyzer = CrossModalAnalyzer::new();
        let result = analyzer.analyze(&entities).unwrap();

        assert_eq!(result.total_entities, 4);
        assert!(result.tabular_graph_correlation > 0.9);
        assert!(result.consistency_score > 0.9);
        assert!(result.passes);
    }

    #[test]
    fn test_inconsistent_modalities() {
        let entities = vec![
            EntityModalData {
                entity_id: "e1".into(),
                tabular_features: vec![1.0, 2.0],
                graph_features: vec![10.0, 1.0],
            },
            EntityModalData {
                entity_id: "e2".into(),
                tabular_features: vec![2.0, 1.0],
                graph_features: vec![9.0, 2.0],
            },
            EntityModalData {
                entity_id: "e3".into(),
                tabular_features: vec![3.0, 0.5],
                graph_features: vec![8.0, 3.5],
            },
            EntityModalData {
                entity_id: "e4".into(),
                tabular_features: vec![4.0, 0.1],
                graph_features: vec![7.0, 4.0],
            },
        ];

        let analyzer = CrossModalAnalyzer::new();
        let result = analyzer.analyze(&entities).unwrap();

        // Anti-correlated on first dim, some correlation on second
        // Overall consistency should be lower
        assert!(result.consistency_score < 0.6);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_entities() {
        let analyzer = CrossModalAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_entities, 0);
        assert_eq!(result.consistency_score, 0.0);
    }
}
