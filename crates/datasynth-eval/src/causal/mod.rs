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
}

/// Thresholds for causal model evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalThresholds {
    /// Minimum edge correlation sign accuracy.
    pub min_sign_accuracy: f64,
    /// Minimum intervention effect accuracy.
    pub min_intervention_accuracy: f64,
}

impl Default for CausalThresholds {
    fn default() -> Self {
        Self {
            min_sign_accuracy: 0.80,
            min_intervention_accuracy: 0.70,
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

        let passes = issues.is_empty();

        Ok(CausalModelEvaluation {
            edge_correlation_sign_accuracy,
            topological_consistency,
            intervention_effect_accuracy,
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
}
