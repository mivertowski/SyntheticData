//! Causal model evaluator.
//!
//! Validates causal model preservation including edge correlation sign accuracy,
//! topological consistency (DAG structure), and intervention effect direction.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Causal edge data for validation.
#[derive(Debug, Clone)]
pub struct CausalEdgeData {
    /// Source variable.
    pub source: String,
    /// Target variable.
    pub target: String,
    /// Expected correlation sign: +1.0 for positive, -1.0 for negative.
    pub expected_sign: f64,
    /// Observed correlation between source and target.
    pub observed_correlation: f64,
}

/// Intervention data for validation.
#[derive(Debug, Clone)]
pub struct InterventionData {
    /// Variable intervened upon.
    pub intervention_variable: String,
    /// Expected effect direction on target: +1.0 for increase, -1.0 for decrease.
    pub expected_direction: f64,
    /// Observed change in target.
    pub observed_change: f64,
    /// Target variable.
    pub target_variable: String,
    /// Expected magnitude of the intervention effect.
    pub expected_magnitude: f64,
    /// Pre-intervention sample values (for Cohen's d computation).
    pub pre_intervention_values: Vec<f64>,
    /// Post-intervention sample values (for Cohen's d computation).
    pub post_intervention_values: Vec<f64>,
}

/// Thresholds for causal model evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalThresholds {
    /// Minimum edge correlation sign accuracy.
    pub min_sign_accuracy: f64,
    /// Minimum intervention effect accuracy.
    pub min_intervention_accuracy: f64,
    /// Minimum intervention magnitude accuracy (fraction within 0.25x-4.0x bounds).
    pub min_magnitude_accuracy: f64,
}

impl Default for CausalThresholds {
    fn default() -> Self {
        Self {
            min_sign_accuracy: 0.80,
            min_intervention_accuracy: 0.70,
            min_magnitude_accuracy: 0.60,
        }
    }
}

/// Results of causal model evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalModelEvaluation {
    /// Edge correlation sign accuracy: fraction of edges with correct sign.
    pub edge_correlation_sign_accuracy: f64,
    /// Whether the graph is topologically consistent (DAG - no cycles).
    pub topological_consistency: bool,
    /// Intervention effect accuracy: fraction with correct direction.
    pub intervention_effect_accuracy: f64,
    /// Fraction of interventions with observed magnitude within 0.25x to 4.0x of expected.
    pub intervention_magnitude_accuracy: f64,
    /// Average effect size (Cohen's d) across interventions.
    pub avg_effect_size: f64,
    /// Total edges evaluated.
    pub total_edges: usize,
    /// Total interventions evaluated.
    pub total_interventions: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for causal model preservation.
pub struct CausalModelEvaluator {
    thresholds: CausalThresholds,
}

impl CausalModelEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: CausalThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: CausalThresholds) -> Self {
        Self { thresholds }
    }

    /// Check if the edge set forms a DAG (no cycles) using Kahn's algorithm.
    fn is_dag(edges: &[CausalEdgeData]) -> bool {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize all nodes
        for edge in edges {
            in_degree.entry(edge.source.as_str()).or_insert(0);
            in_degree.entry(edge.target.as_str()).or_insert(0);
            adj.entry(edge.source.as_str()).or_default();
        }

        // Build adjacency and in-degree
        for edge in edges {
            adj.entry(edge.source.as_str())
                .or_default()
                .push(edge.target.as_str());
            *in_degree.entry(edge.target.as_str()).or_insert(0) += 1;
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&n, _)| n)
            .collect();
        let mut visited = 0usize;

        while let Some(node) = queue.pop_front() {
            visited += 1;
            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if let Some(d) = in_degree.get_mut(neighbor) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        visited == in_degree.len()
    }

    /// Compute Cohen's d for a single intervention from pre/post samples.
    ///
    /// Cohen's d = |mean_diff| / pooled_std
    /// where pooled_std = sqrt(((n1-1)*s1^2 + (n2-1)*s2^2) / (n1+n2-2))
    fn cohens_d(pre: &[f64], post: &[f64]) -> Option<f64> {
        let n1 = pre.len();
        let n2 = post.len();
        if n1 < 2 || n2 < 2 {
            return None;
        }

        let mean1 = pre.iter().sum::<f64>() / n1 as f64;
        let mean2 = post.iter().sum::<f64>() / n2 as f64;

        let var1 = pre.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1) as f64;
        let var2 = post.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1) as f64;

        let pooled_var = ((n1 - 1) as f64 * var1 + (n2 - 1) as f64 * var2) / (n1 + n2 - 2) as f64;
        let pooled_std = pooled_var.sqrt();

        if pooled_std < f64::EPSILON {
            return None;
        }

        Some((mean2 - mean1).abs() / pooled_std)
    }

    /// Compute average Cohen's d across all interventions with sample data.
    fn compute_avg_effect_size(interventions: &[InterventionData]) -> f64 {
        let effect_sizes: Vec<f64> = interventions
            .iter()
            .filter_map(|i| Self::cohens_d(&i.pre_intervention_values, &i.post_intervention_values))
            .collect();

        if effect_sizes.is_empty() {
            0.0
        } else {
            effect_sizes.iter().sum::<f64>() / effect_sizes.len() as f64
        }
    }

    /// Evaluate causal model data.
    pub fn evaluate(
        &self,
        edges: &[CausalEdgeData],
        interventions: &[InterventionData],
    ) -> EvalResult<CausalModelEvaluation> {
        let mut issues = Vec::new();

        // 1. Edge correlation sign accuracy
        let sign_correct = edges
            .iter()
            .filter(|e| {
                // Signs match: both positive or both negative
                e.expected_sign * e.observed_correlation > 0.0
                    || (e.expected_sign.abs() < f64::EPSILON && e.observed_correlation.abs() < 0.05)
            })
            .count();
        let edge_correlation_sign_accuracy = if edges.is_empty() {
            1.0
        } else {
            sign_correct as f64 / edges.len() as f64
        };

        // 2. Topological consistency (DAG check)
        let topological_consistency = if edges.is_empty() {
            true
        } else {
            Self::is_dag(edges)
        };

        // 3. Intervention effect direction
        let intervention_correct = interventions
            .iter()
            .filter(|i| i.expected_direction * i.observed_change > 0.0)
            .count();
        let intervention_effect_accuracy = if interventions.is_empty() {
            1.0
        } else {
            intervention_correct as f64 / interventions.len() as f64
        };

        // 4. Intervention magnitude accuracy
        let magnitude_within_bounds = interventions
            .iter()
            .filter(|i| {
                if i.expected_magnitude.abs() < f64::EPSILON {
                    // Cannot compute ratio when expected magnitude is zero
                    false
                } else {
                    let ratio = i.observed_change.abs() / i.expected_magnitude.abs();
                    (0.25..=4.0).contains(&ratio)
                }
            })
            .count();
        let intervention_magnitude_accuracy = if interventions.is_empty() {
            1.0
        } else {
            magnitude_within_bounds as f64 / interventions.len() as f64
        };

        // 5. Average effect size (Cohen's d)
        let avg_effect_size = Self::compute_avg_effect_size(interventions);

        // Check thresholds
        if edge_correlation_sign_accuracy < self.thresholds.min_sign_accuracy {
            issues.push(format!(
                "Edge sign accuracy {:.3} < {:.3}",
                edge_correlation_sign_accuracy, self.thresholds.min_sign_accuracy
            ));
        }
        if !topological_consistency {
            issues.push("Causal graph contains cycles (not a DAG)".to_string());
        }
        if intervention_effect_accuracy < self.thresholds.min_intervention_accuracy {
            issues.push(format!(
                "Intervention accuracy {:.3} < {:.3}",
                intervention_effect_accuracy, self.thresholds.min_intervention_accuracy
            ));
        }
        if intervention_magnitude_accuracy < self.thresholds.min_magnitude_accuracy {
            issues.push(format!(
                "Intervention magnitude accuracy {:.3} < {:.3}",
                intervention_magnitude_accuracy, self.thresholds.min_magnitude_accuracy
            ));
        }

        let passes = issues.is_empty();

        Ok(CausalModelEvaluation {
            edge_correlation_sign_accuracy,
            topological_consistency,
            intervention_effect_accuracy,
            intervention_magnitude_accuracy,
            avg_effect_size,
            total_edges: edges.len(),
            total_interventions: interventions.len(),
            passes,
            issues,
        })
    }
}

impl Default for CausalModelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_causal_model() {
        let evaluator = CausalModelEvaluator::new();
        let edges = vec![
            CausalEdgeData {
                source: "revenue".to_string(),
                target: "profit".to_string(),
                expected_sign: 1.0,
                observed_correlation: 0.85,
            },
            CausalEdgeData {
                source: "cost".to_string(),
                target: "profit".to_string(),
                expected_sign: -1.0,
                observed_correlation: -0.70,
            },
        ];
        let interventions = vec![InterventionData {
            intervention_variable: "revenue".to_string(),
            expected_direction: 1.0,
            observed_change: 5000.0,
            target_variable: "profit".to_string(),
            expected_magnitude: 5000.0,
            pre_intervention_values: vec![100.0, 110.0, 105.0, 95.0, 108.0],
            post_intervention_values: vec![200.0, 210.0, 205.0, 195.0, 208.0],
        }];

        let result = evaluator.evaluate(&edges, &interventions).unwrap();
        assert!(result.passes);
        assert!(result.topological_consistency);
        assert_eq!(result.edge_correlation_sign_accuracy, 1.0);
    }

    #[test]
    fn test_cyclic_graph() {
        let evaluator = CausalModelEvaluator::new();
        let edges = vec![
            CausalEdgeData {
                source: "A".to_string(),
                target: "B".to_string(),
                expected_sign: 1.0,
                observed_correlation: 0.5,
            },
            CausalEdgeData {
                source: "B".to_string(),
                target: "C".to_string(),
                expected_sign: 1.0,
                observed_correlation: 0.5,
            },
            CausalEdgeData {
                source: "C".to_string(),
                target: "A".to_string(), // Cycle!
                expected_sign: 1.0,
                observed_correlation: 0.5,
            },
        ];

        let result = evaluator.evaluate(&edges, &[]).unwrap();
        assert!(!result.topological_consistency);
        assert!(!result.passes);
    }

    #[test]
    fn test_wrong_signs() {
        let evaluator = CausalModelEvaluator::new();
        let edges = vec![CausalEdgeData {
            source: "revenue".to_string(),
            target: "profit".to_string(),
            expected_sign: 1.0,
            observed_correlation: -0.5, // Wrong sign
        }];

        let result = evaluator.evaluate(&edges, &[]).unwrap();
        assert!(!result.passes);
        assert_eq!(result.edge_correlation_sign_accuracy, 0.0);
    }

    #[test]
    fn test_empty() {
        let evaluator = CausalModelEvaluator::new();
        let result = evaluator.evaluate(&[], &[]).unwrap();
        assert!(result.passes);
    }

    #[test]
    fn test_intervention_magnitude_within_bounds() {
        let evaluator = CausalModelEvaluator::new();
        let edges = vec![CausalEdgeData {
            source: "price".to_string(),
            target: "demand".to_string(),
            expected_sign: -1.0,
            observed_correlation: -0.6,
        }];
        // All interventions have observed magnitude within 0.25x to 4.0x of expected
        let interventions = vec![
            InterventionData {
                intervention_variable: "price".to_string(),
                expected_direction: -1.0,
                observed_change: -120.0,
                target_variable: "demand".to_string(),
                expected_magnitude: 100.0, // ratio = 1.2, within [0.25, 4.0]
                pre_intervention_values: vec![500.0, 510.0, 490.0, 505.0, 495.0],
                post_intervention_values: vec![380.0, 390.0, 370.0, 385.0, 375.0],
            },
            InterventionData {
                intervention_variable: "price".to_string(),
                expected_direction: -1.0,
                observed_change: -200.0,
                target_variable: "demand".to_string(),
                expected_magnitude: 150.0, // ratio = 1.33, within [0.25, 4.0]
                pre_intervention_values: vec![600.0, 610.0, 590.0, 605.0, 595.0],
                post_intervention_values: vec![400.0, 410.0, 390.0, 405.0, 395.0],
            },
            InterventionData {
                intervention_variable: "price".to_string(),
                expected_direction: -1.0,
                observed_change: -50.0,
                target_variable: "demand".to_string(),
                expected_magnitude: 60.0, // ratio = 0.83, within [0.25, 4.0]
                pre_intervention_values: vec![300.0, 310.0, 290.0, 305.0, 295.0],
                post_intervention_values: vec![250.0, 260.0, 240.0, 255.0, 245.0],
            },
        ];

        let result = evaluator.evaluate(&edges, &interventions).unwrap();
        assert_eq!(result.intervention_magnitude_accuracy, 1.0);
        assert!(result.avg_effect_size > 0.0);
        assert!(result.passes);
    }

    #[test]
    fn test_intervention_magnitude_out_of_bounds() {
        let evaluator = CausalModelEvaluator::new();
        let edges = vec![CausalEdgeData {
            source: "marketing".to_string(),
            target: "sales".to_string(),
            expected_sign: 1.0,
            observed_correlation: 0.7,
        }];
        // Most interventions have extreme magnitudes (outside 0.25x to 4.0x)
        let interventions = vec![
            InterventionData {
                intervention_variable: "marketing".to_string(),
                expected_direction: 1.0,
                observed_change: 10.0,
                target_variable: "sales".to_string(),
                expected_magnitude: 1000.0, // ratio = 0.01, below 0.25
                pre_intervention_values: vec![100.0, 105.0, 95.0],
                post_intervention_values: vec![110.0, 115.0, 105.0],
            },
            InterventionData {
                intervention_variable: "marketing".to_string(),
                expected_direction: 1.0,
                observed_change: 50000.0,
                target_variable: "sales".to_string(),
                expected_magnitude: 100.0, // ratio = 500.0, above 4.0
                pre_intervention_values: vec![200.0, 210.0, 190.0],
                post_intervention_values: vec![50200.0, 50210.0, 50190.0],
            },
            InterventionData {
                intervention_variable: "marketing".to_string(),
                expected_direction: 1.0,
                observed_change: 5.0,
                target_variable: "sales".to_string(),
                expected_magnitude: 500.0, // ratio = 0.01, below 0.25
                pre_intervention_values: vec![100.0, 105.0, 95.0],
                post_intervention_values: vec![105.0, 110.0, 100.0],
            },
            InterventionData {
                intervention_variable: "marketing".to_string(),
                expected_direction: 1.0,
                observed_change: 150.0,
                target_variable: "sales".to_string(),
                expected_magnitude: 100.0, // ratio = 1.5, within bounds (the one pass)
                pre_intervention_values: vec![100.0, 105.0, 95.0],
                post_intervention_values: vec![250.0, 255.0, 245.0],
            },
        ];

        let result = evaluator.evaluate(&edges, &interventions).unwrap();
        // Only 1 out of 4 is within bounds => 0.25 < 0.60 (default threshold)
        assert_eq!(result.intervention_magnitude_accuracy, 0.25);
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("magnitude accuracy")));
    }

    #[test]
    fn test_effect_size_computation() {
        let evaluator = CausalModelEvaluator::new();
        // Create intervention with known pre/post values for Cohen's d verification.
        // Pre: mean=100, Post: mean=120, pooled_std should be ~5.0
        // Cohen's d = |120 - 100| / 5.0 = 4.0
        let interventions = vec![InterventionData {
            intervention_variable: "treatment".to_string(),
            expected_direction: 1.0,
            observed_change: 20.0,
            target_variable: "outcome".to_string(),
            expected_magnitude: 20.0,
            pre_intervention_values: vec![95.0, 100.0, 105.0, 100.0, 100.0],
            post_intervention_values: vec![115.0, 120.0, 125.0, 120.0, 120.0],
        }];

        // Manually compute expected Cohen's d:
        // pre: mean=100, var = ((25+0+25+0+0)/4) = 12.5, std = 3.536
        // post: mean=120, var = ((25+0+25+0+0)/4) = 12.5, std = 3.536
        // pooled_var = ((4*12.5 + 4*12.5) / 8) = 12.5
        // pooled_std = sqrt(12.5) = 3.536
        // Cohen's d = |120-100| / 3.536 = 5.657
        let edges = vec![CausalEdgeData {
            source: "treatment".to_string(),
            target: "outcome".to_string(),
            expected_sign: 1.0,
            observed_correlation: 0.9,
        }];

        let result = evaluator.evaluate(&edges, &interventions).unwrap();
        assert!(result.avg_effect_size > 5.0);
        assert!((result.avg_effect_size - 5.657).abs() < 0.1);

        // Also test with multiple interventions
        let interventions_multi = vec![
            InterventionData {
                intervention_variable: "a".to_string(),
                expected_direction: 1.0,
                observed_change: 10.0,
                target_variable: "b".to_string(),
                expected_magnitude: 10.0,
                // pre mean=50, post mean=60, same variance => d = 10/std
                pre_intervention_values: vec![48.0, 50.0, 52.0],
                post_intervention_values: vec![58.0, 60.0, 62.0],
            },
            InterventionData {
                intervention_variable: "c".to_string(),
                expected_direction: 1.0,
                observed_change: 0.1,
                target_variable: "d".to_string(),
                expected_magnitude: 0.1,
                // pre mean=0, post mean=0 with same std => d ≈ 0
                pre_intervention_values: vec![0.0, 0.0, 0.0],
                post_intervention_values: vec![0.0, 0.0, 0.0],
            },
        ];

        let result2 = evaluator.evaluate(&edges, &interventions_multi).unwrap();
        // Second intervention has zero pooled_std, so only first contributes
        // For first: pre var = 4.0, post var = 4.0, pooled_std = 2.0, d = 10/2 = 5.0
        assert!((result2.avg_effect_size - 5.0).abs() < 0.01);
    }
}
