//! GNN readiness evaluation.
//!
//! Evaluates graph structure suitability for Graph Neural Network training,
//! including feature completeness, homophily ratio, label leakage, and
//! neighborhood diversity.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Input graph data for GNN readiness analysis.
#[derive(Debug, Clone)]
pub struct GraphData {
    /// Node identifiers.
    pub node_ids: Vec<String>,
    /// Optional label for each node.
    pub node_labels: Vec<Option<String>>,
    /// Feature vector length for each node (0 = missing features).
    pub node_feature_counts: Vec<usize>,
    /// Edge list as index pairs into node_ids.
    pub edges: Vec<(usize, usize)>,
    /// Feature vector length for each edge.
    pub edge_feature_counts: Vec<usize>,
}

/// Thresholds for GNN readiness analysis.
#[derive(Debug, Clone)]
pub struct GnnReadinessThresholds {
    /// Minimum overall GNN readiness score.
    pub min_gnn_readiness: f64,
}

impl Default for GnnReadinessThresholds {
    fn default() -> Self {
        Self {
            min_gnn_readiness: 0.65,
        }
    }
}

/// Results of GNN readiness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GnnReadinessAnalysis {
    /// Overall GNN readiness score (0.0-1.0).
    pub gnn_readiness_score: f64,
    /// Fraction of edges connecting nodes with the same label.
    pub homophily_ratio: f64,
    /// Correlation between node degree and label (structural label leakage).
    pub structural_label_leakage: f64,
    /// Fraction of nodes with complete (non-zero) feature vectors.
    pub feature_completeness_score: f64,
    /// Average number of distinct labels in each node's 1-hop neighborhood.
    pub avg_neighborhood_diversity: f64,
    /// Total number of nodes.
    pub total_nodes: usize,
    /// Total number of edges.
    pub total_edges: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for GNN readiness.
pub struct GnnReadinessAnalyzer {
    thresholds: GnnReadinessThresholds,
}

impl GnnReadinessAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: GnnReadinessThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: GnnReadinessThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze GNN readiness.
    pub fn analyze(&self, data: &GraphData) -> EvalResult<GnnReadinessAnalysis> {
        let mut issues = Vec::new();
        let total_nodes = data.node_ids.len();
        let total_edges = data.edges.len();

        if total_nodes == 0 {
            return Ok(GnnReadinessAnalysis {
                gnn_readiness_score: 0.0,
                homophily_ratio: 0.0,
                structural_label_leakage: 0.0,
                feature_completeness_score: 0.0,
                avg_neighborhood_diversity: 0.0,
                total_nodes: 0,
                total_edges: 0,
                passes: true,
                issues: vec!["No nodes provided".to_string()],
            });
        }

        // Feature completeness: fraction of nodes with non-zero feature count
        let complete_nodes = data.node_feature_counts.iter().filter(|&&c| c > 0).count();
        let feature_completeness_score = complete_nodes as f64 / total_nodes as f64;

        // Build adjacency list
        let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(src, tgt) in &data.edges {
            if src < total_nodes && tgt < total_nodes {
                adjacency.entry(src).or_default().push(tgt);
                adjacency.entry(tgt).or_default().push(src);
            }
        }

        // Homophily ratio: fraction of edges where both endpoints share a label
        let homophily_ratio = self.compute_homophily(data, total_nodes);

        // Structural label leakage: correlation between degree and label
        let structural_label_leakage = self.compute_label_leakage(data, &adjacency, total_nodes);

        // Neighborhood diversity: average distinct labels in 1-hop neighborhood
        let avg_neighborhood_diversity =
            self.compute_neighborhood_diversity(data, &adjacency, total_nodes);

        // Composite readiness score
        let gnn_readiness_score = (feature_completeness_score * 0.3
            + homophily_ratio.clamp(0.0, 1.0) * 0.3
            + (1.0 - structural_label_leakage.abs()).clamp(0.0, 1.0) * 0.2
            + avg_neighborhood_diversity.clamp(0.0, 1.0) * 0.2)
            .clamp(0.0, 1.0);

        if gnn_readiness_score < self.thresholds.min_gnn_readiness {
            issues.push(format!(
                "GNN readiness score {:.4} < {:.4} (threshold)",
                gnn_readiness_score, self.thresholds.min_gnn_readiness
            ));
        }

        if feature_completeness_score < 0.5 {
            issues.push(format!(
                "Low feature completeness: {:.2}%",
                feature_completeness_score * 100.0
            ));
        }

        let passes = issues.is_empty();

        Ok(GnnReadinessAnalysis {
            gnn_readiness_score,
            homophily_ratio,
            structural_label_leakage,
            feature_completeness_score,
            avg_neighborhood_diversity,
            total_nodes,
            total_edges,
            passes,
            issues,
        })
    }

    /// Compute homophily ratio: fraction of edges connecting same-label nodes.
    fn compute_homophily(&self, data: &GraphData, total_nodes: usize) -> f64 {
        if data.edges.is_empty() {
            return 0.0;
        }

        let mut same_label = 0usize;
        let mut labeled_edges = 0usize;

        for &(src, tgt) in &data.edges {
            if src >= total_nodes || tgt >= total_nodes {
                continue;
            }
            let src_label = data.node_labels.get(src).and_then(|l| l.as_ref());
            let tgt_label = data.node_labels.get(tgt).and_then(|l| l.as_ref());

            if let (Some(sl), Some(tl)) = (src_label, tgt_label) {
                labeled_edges += 1;
                if sl == tl {
                    same_label += 1;
                }
            }
        }

        if labeled_edges == 0 {
            return 0.0;
        }

        same_label as f64 / labeled_edges as f64
    }

    /// Compute structural label leakage as correlation between degree and label.
    ///
    /// Encodes labels as ordinal indices and computes Pearson correlation
    /// with node degree.
    fn compute_label_leakage(
        &self,
        data: &GraphData,
        adjacency: &HashMap<usize, Vec<usize>>,
        total_nodes: usize,
    ) -> f64 {
        // Build label-to-index mapping
        let mut label_map: HashMap<&str, f64> = HashMap::new();
        let mut next_idx = 0.0;
        for label in data.node_labels.iter().flatten() {
            if !label_map.contains_key(label.as_str()) {
                label_map.insert(label.as_str(), next_idx);
                next_idx += 1.0;
            }
        }

        let mut degrees = Vec::new();
        let mut label_indices = Vec::new();

        for i in 0..total_nodes {
            if let Some(Some(ref label)) = data.node_labels.get(i) {
                if let Some(&idx) = label_map.get(label.as_str()) {
                    let degree = adjacency.get(&i).map_or(0, |v| v.len());
                    degrees.push(degree as f64);
                    label_indices.push(idx);
                }
            }
        }

        if degrees.len() < 3 {
            return 0.0;
        }

        pearson_correlation_slices(&degrees, &label_indices).unwrap_or(0.0)
    }

    /// Compute average neighborhood diversity.
    ///
    /// For each node with a label, count distinct labels among its 1-hop neighbors,
    /// normalized by the total number of distinct labels.
    fn compute_neighborhood_diversity(
        &self,
        data: &GraphData,
        adjacency: &HashMap<usize, Vec<usize>>,
        total_nodes: usize,
    ) -> f64 {
        let all_labels: HashSet<&str> = data
            .node_labels
            .iter()
            .filter_map(|l| l.as_deref())
            .collect();

        if all_labels.is_empty() || all_labels.len() == 1 {
            return if all_labels.len() == 1 { 1.0 } else { 0.0 };
        }

        let label_count = all_labels.len() as f64;
        let mut total_diversity = 0.0;
        let mut counted_nodes = 0usize;

        for i in 0..total_nodes {
            if let Some(neighbors) = adjacency.get(&i) {
                if neighbors.is_empty() {
                    continue;
                }
                let neighbor_labels: HashSet<&str> = neighbors
                    .iter()
                    .filter_map(|&n| data.node_labels.get(n).and_then(|l| l.as_deref()))
                    .collect();

                if !neighbor_labels.is_empty() {
                    total_diversity += neighbor_labels.len() as f64 / label_count;
                    counted_nodes += 1;
                }
            }
        }

        if counted_nodes == 0 {
            return 0.0;
        }

        total_diversity / counted_nodes as f64
    }
}

/// Compute Pearson correlation between two slices.
fn pearson_correlation_slices(x: &[f64], y: &[f64]) -> Option<f64> {
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

impl Default for GnnReadinessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_graph() {
        let data = GraphData {
            node_ids: vec!["n0".into(), "n1".into(), "n2".into(), "n3".into()],
            node_labels: vec![
                Some("A".into()),
                Some("A".into()),
                Some("B".into()),
                Some("B".into()),
            ],
            node_feature_counts: vec![10, 10, 10, 10],
            edges: vec![(0, 1), (1, 2), (2, 3), (0, 3)],
            edge_feature_counts: vec![5, 5, 5, 5],
        };

        let analyzer = GnnReadinessAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.total_nodes, 4);
        assert_eq!(result.total_edges, 4);
        assert!(result.feature_completeness_score > 0.99);
        assert!(result.gnn_readiness_score > 0.0);
    }

    #[test]
    fn test_invalid_graph_missing_features() {
        let data = GraphData {
            node_ids: vec!["n0".into(), "n1".into(), "n2".into(), "n3".into()],
            node_labels: vec![
                Some("A".into()),
                Some("A".into()),
                Some("B".into()),
                Some("B".into()),
            ],
            node_feature_counts: vec![0, 0, 0, 0], // no features
            edges: vec![(0, 1)],
            edge_feature_counts: vec![0],
        };

        let analyzer = GnnReadinessAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert!(result.feature_completeness_score < 0.01);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_graph() {
        let data = GraphData {
            node_ids: Vec::new(),
            node_labels: Vec::new(),
            node_feature_counts: Vec::new(),
            edges: Vec::new(),
            edge_feature_counts: Vec::new(),
        };

        let analyzer = GnnReadinessAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.total_nodes, 0);
        assert_eq!(result.gnn_readiness_score, 0.0);
    }
}
