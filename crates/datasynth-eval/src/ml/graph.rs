//! Graph structure analysis for ML.
//!
//! Analyzes graph properties relevant for graph neural networks.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of graph analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Basic graph metrics.
    pub metrics: GraphMetrics,
    /// Degree distribution analysis.
    pub degree_distribution: DegreeDistribution,
    /// Node type balance.
    pub node_type_balance: HashMap<String, f64>,
    /// Edge type balance.
    pub edge_type_balance: HashMap<String, f64>,
    /// Connectivity score (0.0-1.0).
    pub connectivity_score: f64,
    /// Whether graph meets quality criteria.
    pub is_valid: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Basic graph metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Number of nodes.
    pub node_count: usize,
    /// Number of edges.
    pub edge_count: usize,
    /// Graph density (edges / max_possible_edges).
    pub density: f64,
    /// Number of connected components.
    pub connected_components: usize,
    /// Size of largest connected component.
    pub largest_component_size: usize,
    /// Percentage of nodes in largest component.
    pub largest_component_ratio: f64,
    /// Average degree.
    pub average_degree: f64,
    /// Maximum degree.
    pub max_degree: usize,
    /// Number of isolated nodes (degree 0).
    pub isolated_nodes: usize,
}

/// Degree distribution analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegreeDistribution {
    /// Degree histogram (degree -> count).
    pub histogram: HashMap<usize, usize>,
    /// Mean degree.
    pub mean: f64,
    /// Median degree.
    pub median: f64,
    /// Standard deviation.
    pub std_dev: f64,
    /// Whether distribution follows power law.
    pub is_power_law: bool,
    /// Power law exponent (if applicable).
    pub power_law_exponent: Option<f64>,
}

/// Input for graph analysis.
#[derive(Debug, Clone)]
pub struct GraphData {
    /// Node count.
    pub node_count: usize,
    /// Edge list: (source, target) pairs.
    pub edges: Vec<(usize, usize)>,
    /// Node types: node_id -> type.
    pub node_types: HashMap<usize, String>,
    /// Edge types: edge_index -> type.
    pub edge_types: HashMap<usize, String>,
    /// Whether graph is directed.
    pub is_directed: bool,
}

impl Default for GraphData {
    fn default() -> Self {
        Self {
            node_count: 0,
            edges: Vec::new(),
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            is_directed: true,
        }
    }
}

/// Analyzer for graph structure.
pub struct GraphAnalyzer {
    /// Minimum connectivity threshold.
    min_connectivity: f64,
    /// Maximum isolated node ratio.
    max_isolated_ratio: f64,
}

impl GraphAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            min_connectivity: 0.95,
            max_isolated_ratio: 0.05,
        }
    }

    /// Analyze graph structure.
    pub fn analyze(&self, data: &GraphData) -> EvalResult<GraphAnalysis> {
        let mut issues = Vec::new();

        if data.node_count == 0 {
            return Ok(GraphAnalysis {
                metrics: GraphMetrics {
                    node_count: 0,
                    edge_count: 0,
                    density: 0.0,
                    connected_components: 0,
                    largest_component_size: 0,
                    largest_component_ratio: 1.0,
                    average_degree: 0.0,
                    max_degree: 0,
                    isolated_nodes: 0,
                },
                degree_distribution: DegreeDistribution {
                    histogram: HashMap::new(),
                    mean: 0.0,
                    median: 0.0,
                    std_dev: 0.0,
                    is_power_law: false,
                    power_law_exponent: None,
                },
                node_type_balance: HashMap::new(),
                edge_type_balance: HashMap::new(),
                connectivity_score: 1.0,
                is_valid: true,
                issues: vec![],
            });
        }

        // Calculate degree for each node
        let mut degrees: Vec<usize> = vec![0; data.node_count];
        for (src, tgt) in &data.edges {
            if *src < data.node_count {
                degrees[*src] += 1;
            }
            if !data.is_directed && *tgt < data.node_count {
                degrees[*tgt] += 1;
            }
        }

        // Calculate metrics
        let edge_count = data.edges.len();
        let max_edges = if data.is_directed {
            data.node_count * (data.node_count - 1)
        } else {
            data.node_count * (data.node_count - 1) / 2
        };
        let density = if max_edges > 0 {
            edge_count as f64 / max_edges as f64
        } else {
            0.0
        };

        let average_degree = if data.node_count > 0 {
            degrees.iter().sum::<usize>() as f64 / data.node_count as f64
        } else {
            0.0
        };

        let max_degree = degrees.iter().max().copied().unwrap_or(0);
        let isolated_nodes = degrees.iter().filter(|d| **d == 0).count();

        // Find connected components using union-find
        let (connected_components, component_sizes) = self.find_components(data);
        let largest_component_size = component_sizes.iter().max().copied().unwrap_or(0);
        let largest_component_ratio = if data.node_count > 0 {
            largest_component_size as f64 / data.node_count as f64
        } else {
            1.0
        };

        let connectivity_score = largest_component_ratio;

        // Calculate degree distribution
        let degree_distribution = self.calculate_degree_distribution(&degrees);

        // Calculate node/edge type balance
        let node_type_balance = self.calculate_type_balance(&data.node_types, data.node_count);
        let edge_type_balance = self.calculate_type_balance_usize(&data.edge_types, edge_count);

        let metrics = GraphMetrics {
            node_count: data.node_count,
            edge_count,
            density,
            connected_components,
            largest_component_size,
            largest_component_ratio,
            average_degree,
            max_degree,
            isolated_nodes,
        };

        // Check for issues
        if connectivity_score < self.min_connectivity {
            issues.push(format!(
                "Low connectivity: {:.2}% of nodes in largest component",
                connectivity_score * 100.0
            ));
        }

        let isolated_ratio = if data.node_count > 0 {
            isolated_nodes as f64 / data.node_count as f64
        } else {
            0.0
        };
        if isolated_ratio > self.max_isolated_ratio {
            issues.push(format!(
                "High isolated node ratio: {:.2}%",
                isolated_ratio * 100.0
            ));
        }

        if connected_components > 1 {
            issues.push(format!(
                "Graph has {} connected components",
                connected_components
            ));
        }

        let is_valid = connectivity_score >= self.min_connectivity
            && isolated_ratio <= self.max_isolated_ratio;

        Ok(GraphAnalysis {
            metrics,
            degree_distribution,
            node_type_balance,
            edge_type_balance,
            connectivity_score,
            is_valid,
            issues,
        })
    }

    /// Find connected components using union-find.
    fn find_components(&self, data: &GraphData) -> (usize, Vec<usize>) {
        let mut parent: Vec<usize> = (0..data.node_count).collect();
        let mut rank: Vec<usize> = vec![0; data.node_count];

        fn find(parent: &mut [usize], x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }

        fn union(parent: &mut [usize], rank: &mut [usize], x: usize, y: usize) {
            let px = find(parent, x);
            let py = find(parent, y);
            if px != py {
                if rank[px] < rank[py] {
                    parent[px] = py;
                } else if rank[px] > rank[py] {
                    parent[py] = px;
                } else {
                    parent[py] = px;
                    rank[px] += 1;
                }
            }
        }

        for (src, tgt) in &data.edges {
            if *src < data.node_count && *tgt < data.node_count {
                union(&mut parent, &mut rank, *src, *tgt);
            }
        }

        // Count components and their sizes
        let mut component_sizes: HashMap<usize, usize> = HashMap::new();
        for i in 0..data.node_count {
            let root = find(&mut parent, i);
            *component_sizes.entry(root).or_insert(0) += 1;
        }

        let num_components = component_sizes.len();
        let sizes: Vec<usize> = component_sizes.values().copied().collect();

        (num_components, sizes)
    }

    /// Calculate degree distribution statistics.
    fn calculate_degree_distribution(&self, degrees: &[usize]) -> DegreeDistribution {
        if degrees.is_empty() {
            return DegreeDistribution {
                histogram: HashMap::new(),
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                is_power_law: false,
                power_law_exponent: None,
            };
        }

        // Build histogram
        let mut histogram: HashMap<usize, usize> = HashMap::new();
        for &d in degrees {
            *histogram.entry(d).or_insert(0) += 1;
        }

        // Calculate statistics
        let n = degrees.len() as f64;
        let mean = degrees.iter().sum::<usize>() as f64 / n;

        let mut sorted = degrees.to_vec();
        sorted.sort_unstable();
        let median = if sorted.len().is_multiple_of(2) {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) as f64 / 2.0
        } else {
            sorted[sorted.len() / 2] as f64
        };

        let variance: f64 = degrees
            .iter()
            .map(|&d| (d as f64 - mean).powi(2))
            .sum::<f64>()
            / n;
        let std_dev = variance.sqrt();

        // Simple power-law check: log-log linear relationship
        // This is a simplified heuristic
        let non_zero_degrees: Vec<_> = degrees.iter().filter(|&&d| d > 0).collect();
        let is_power_law = if non_zero_degrees.len() > 10 && std_dev > mean {
            // High variance relative to mean suggests heavy tail
            true
        } else {
            false
        };

        let power_law_exponent = if is_power_law {
            // Simplified estimate using Hill estimator
            let k_min = 1.0;
            let valid: Vec<f64> = non_zero_degrees
                .iter()
                .filter(|&&d| (*d as f64) >= k_min)
                .map(|&&d| d as f64)
                .collect();
            if valid.len() > 2 {
                let n = valid.len() as f64;
                let sum_log: f64 = valid.iter().map(|&x| (x / k_min).ln()).sum();
                Some(1.0 + n / sum_log)
            } else {
                None
            }
        } else {
            None
        };

        DegreeDistribution {
            histogram,
            mean,
            median,
            std_dev,
            is_power_law,
            power_law_exponent,
        }
    }

    /// Calculate type balance (usize keys).
    fn calculate_type_balance(
        &self,
        types: &HashMap<usize, String>,
        total: usize,
    ) -> HashMap<String, f64> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for t in types.values() {
            *counts.entry(t.clone()).or_insert(0) += 1;
        }

        counts
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    if total > 0 {
                        v as f64 / total as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect()
    }

    /// Calculate type balance for edge types.
    fn calculate_type_balance_usize(
        &self,
        types: &HashMap<usize, String>,
        total: usize,
    ) -> HashMap<String, f64> {
        self.calculate_type_balance(types, total)
    }
}

impl Default for GraphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_graph() {
        let data = GraphData {
            node_count: 4,
            edges: vec![(0, 1), (1, 2), (2, 3)],
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            is_directed: false,
        };

        let analyzer = GraphAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.metrics.connected_components, 1);
        assert_eq!(result.metrics.largest_component_ratio, 1.0);
        assert_eq!(result.metrics.isolated_nodes, 0);
    }

    #[test]
    fn test_disconnected_graph() {
        let data = GraphData {
            node_count: 4,
            edges: vec![(0, 1)], // Only 2 nodes connected
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            is_directed: false,
        };

        let analyzer = GraphAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert!(result.metrics.connected_components > 1);
        assert!(result.metrics.isolated_nodes > 0);
    }

    #[test]
    fn test_empty_graph() {
        let data = GraphData::default();

        let analyzer = GraphAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.metrics.node_count, 0);
        assert!(result.is_valid);
    }

    #[test]
    fn test_degree_distribution() {
        let data = GraphData {
            node_count: 5,
            edges: vec![(0, 1), (0, 2), (0, 3), (0, 4), (1, 2)],
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            is_directed: true,
        };

        let analyzer = GraphAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.metrics.max_degree, 4); // Node 0 has degree 4
        assert!(result.degree_distribution.mean > 0.0);
    }
}
