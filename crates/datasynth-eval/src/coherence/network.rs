//! Network evaluation module for interconnectivity analysis.
//!
//! Provides evaluation of entity relationship graphs and network metrics including:
//! - Graph connectivity analysis
//! - Degree distribution (power law fit)
//! - Clustering coefficient
//! - Vendor/customer concentration
//! - Relationship strength validation

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Results of network evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvaluation {
    /// Number of nodes in the graph.
    pub node_count: usize,
    /// Number of edges in the graph.
    pub edge_count: usize,
    /// Largest connected component size as fraction of total nodes.
    pub connectivity_ratio: f64,
    /// Power law exponent (alpha) for degree distribution.
    pub power_law_alpha: Option<f64>,
    /// Global clustering coefficient.
    pub clustering_coefficient: f64,
    /// Vendor concentration metrics.
    pub vendor_concentration: ConcentrationMetrics,
    /// Customer concentration metrics.
    pub customer_concentration: ConcentrationMetrics,
    /// Average relationship strength.
    pub avg_relationship_strength: f64,
    /// Relationship strength distribution statistics.
    pub strength_stats: StrengthStats,
    /// Cross-process link coverage (P2P↔O2C via inventory).
    pub cross_process_link_rate: f64,
    /// Whether the network passes all thresholds.
    pub passes: bool,
    /// List of threshold violations.
    pub issues: Vec<String>,
}

impl Default for NetworkEvaluation {
    fn default() -> Self {
        Self {
            node_count: 0,
            edge_count: 0,
            connectivity_ratio: 0.0,
            power_law_alpha: None,
            clustering_coefficient: 0.0,
            vendor_concentration: ConcentrationMetrics::default(),
            customer_concentration: ConcentrationMetrics::default(),
            avg_relationship_strength: 0.0,
            strength_stats: StrengthStats::default(),
            cross_process_link_rate: 0.0,
            passes: true,
            issues: Vec::new(),
        }
    }
}

/// Concentration metrics for vendor or customer analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConcentrationMetrics {
    /// Total count of entities.
    pub total_count: usize,
    /// Top vendor/customer share of volume.
    pub top_1_share: f64,
    /// Top 5 vendors/customers share of volume.
    pub top_5_share: f64,
    /// Herfindahl-Hirschman Index (HHI).
    pub hhi: f64,
    /// Whether concentration violates limits.
    pub exceeds_limits: bool,
}

/// Relationship strength distribution statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrengthStats {
    /// Minimum strength value.
    pub min: f64,
    /// Maximum strength value.
    pub max: f64,
    /// Mean strength value.
    pub mean: f64,
    /// Standard deviation of strength.
    pub std_dev: f64,
    /// Count of strong relationships (>= 0.7).
    pub strong_count: usize,
    /// Count of moderate relationships (0.4-0.7).
    pub moderate_count: usize,
    /// Count of weak relationships (0.1-0.4).
    pub weak_count: usize,
    /// Count of dormant relationships (< 0.1).
    pub dormant_count: usize,
}

/// Configuration for network evaluation thresholds.
#[derive(Debug, Clone)]
pub struct NetworkThresholds {
    /// Minimum connectivity ratio (fraction of nodes in largest component).
    pub connectivity_min: f64,
    /// Expected power law alpha range.
    pub power_law_alpha_min: f64,
    pub power_law_alpha_max: f64,
    /// Expected clustering coefficient range.
    pub clustering_min: f64,
    pub clustering_max: f64,
    /// Maximum single vendor concentration.
    pub max_single_vendor_concentration: f64,
    /// Maximum top 5 vendor concentration.
    pub max_top5_vendor_concentration: f64,
    /// Minimum cross-process link rate.
    pub min_cross_process_link_rate: f64,
}

impl Default for NetworkThresholds {
    fn default() -> Self {
        Self {
            connectivity_min: 0.95,
            power_law_alpha_min: 2.0,
            power_law_alpha_max: 3.0,
            clustering_min: 0.10,
            clustering_max: 0.50,
            max_single_vendor_concentration: 0.15,
            max_top5_vendor_concentration: 0.45,
            min_cross_process_link_rate: 0.30,
        }
    }
}

/// Input edge for network analysis.
#[derive(Debug, Clone)]
pub struct NetworkEdge {
    /// Source node ID.
    pub from_id: String,
    /// Target node ID.
    pub to_id: String,
    /// Relationship strength (0.0 to 1.0).
    pub strength: f64,
    /// Transaction volume for this edge.
    pub volume: f64,
}

/// Input node with type information.
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// Node ID.
    pub id: String,
    /// Node type (vendor, customer, company, etc.).
    pub node_type: String,
    /// Associated transaction volume.
    pub volume: f64,
}

/// Network evaluator for graph analysis.
pub struct NetworkEvaluator {
    thresholds: NetworkThresholds,
}

impl NetworkEvaluator {
    /// Create a new network evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: NetworkThresholds::default(),
        }
    }

    /// Create a network evaluator with custom thresholds.
    pub fn with_thresholds(thresholds: NetworkThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate a network graph.
    pub fn evaluate(
        &self,
        nodes: &[NetworkNode],
        edges: &[NetworkEdge],
        cross_process_links: usize,
        potential_links: usize,
    ) -> NetworkEvaluation {
        let mut eval = NetworkEvaluation {
            node_count: nodes.len(),
            edge_count: edges.len(),
            ..Default::default()
        };

        if nodes.is_empty() {
            eval.issues.push("Empty graph".to_string());
            eval.passes = false;
            return eval;
        }

        // Calculate connectivity
        eval.connectivity_ratio = self.calculate_connectivity(nodes, edges);

        // Calculate degree distribution and power law fit
        eval.power_law_alpha = self.estimate_power_law_alpha(nodes, edges);

        // Calculate clustering coefficient
        eval.clustering_coefficient = self.calculate_clustering_coefficient(nodes, edges);

        // Calculate concentration metrics
        eval.vendor_concentration = self.calculate_concentration(nodes, "vendor");
        eval.customer_concentration = self.calculate_concentration(nodes, "customer");

        // Calculate relationship strength statistics
        eval.strength_stats = self.calculate_strength_stats(edges);
        eval.avg_relationship_strength = eval.strength_stats.mean;

        // Calculate cross-process link rate
        eval.cross_process_link_rate = if potential_links > 0 {
            cross_process_links as f64 / potential_links as f64
        } else {
            0.0
        };

        // Check thresholds
        self.check_thresholds(&mut eval);

        eval
    }

    /// Calculate graph connectivity (largest component ratio).
    fn calculate_connectivity(&self, nodes: &[NetworkNode], edges: &[NetworkEdge]) -> f64 {
        if nodes.is_empty() {
            return 0.0;
        }

        // Build adjacency list
        let mut adjacency: HashMap<&str, HashSet<&str>> = HashMap::new();
        for node in nodes {
            adjacency.insert(&node.id, HashSet::new());
        }
        for edge in edges {
            if let Some(neighbors) = adjacency.get_mut(edge.from_id.as_str()) {
                neighbors.insert(&edge.to_id);
            }
            if let Some(neighbors) = adjacency.get_mut(edge.to_id.as_str()) {
                neighbors.insert(&edge.from_id);
            }
        }

        // Find connected components using BFS
        let mut visited: HashSet<&str> = HashSet::new();
        let mut largest_component = 0usize;

        for node in nodes {
            if visited.contains(node.id.as_str()) {
                continue;
            }

            let mut component_size = 0;
            let mut queue = vec![node.id.as_str()];

            while let Some(current) = queue.pop() {
                if visited.contains(current) {
                    continue;
                }
                visited.insert(current);
                component_size += 1;

                if let Some(neighbors) = adjacency.get(current) {
                    for neighbor in neighbors {
                        if !visited.contains(*neighbor) {
                            queue.push(neighbor);
                        }
                    }
                }
            }

            largest_component = largest_component.max(component_size);
        }

        largest_component as f64 / nodes.len() as f64
    }

    /// Estimate power law exponent for degree distribution.
    fn estimate_power_law_alpha(
        &self,
        nodes: &[NetworkNode],
        edges: &[NetworkEdge],
    ) -> Option<f64> {
        // Calculate degree for each node
        let mut degrees: HashMap<&str, usize> = HashMap::new();
        for node in nodes {
            degrees.insert(&node.id, 0);
        }
        for edge in edges {
            *degrees.entry(&edge.from_id).or_insert(0) += 1;
            *degrees.entry(&edge.to_id).or_insert(0) += 1;
        }

        let degree_values: Vec<f64> = degrees
            .values()
            .filter(|&&d| d > 0)
            .map(|&d| d as f64)
            .collect();

        if degree_values.len() < 10 {
            return None;
        }

        // Simple MLE estimation of power law alpha
        // alpha = 1 + n / sum(ln(x_i / x_min))
        let x_min = degree_values.iter().cloned().fold(f64::INFINITY, f64::min);
        if x_min <= 0.0 {
            return None;
        }

        let sum_log: f64 = degree_values.iter().map(|x| (x / x_min).ln()).sum();

        if sum_log <= 0.0 {
            return None;
        }

        let alpha = 1.0 + degree_values.len() as f64 / sum_log;
        Some(alpha)
    }

    /// Calculate global clustering coefficient.
    fn calculate_clustering_coefficient(
        &self,
        nodes: &[NetworkNode],
        edges: &[NetworkEdge],
    ) -> f64 {
        if nodes.len() < 3 {
            return 0.0;
        }

        // Build adjacency set
        let mut neighbors: HashMap<&str, HashSet<&str>> = HashMap::new();
        for node in nodes {
            neighbors.insert(&node.id, HashSet::new());
        }
        for edge in edges {
            if let Some(set) = neighbors.get_mut(edge.from_id.as_str()) {
                set.insert(&edge.to_id);
            }
            if let Some(set) = neighbors.get_mut(edge.to_id.as_str()) {
                set.insert(&edge.from_id);
            }
        }

        // Calculate local clustering for each node
        let mut total_clustering = 0.0;
        let mut valid_nodes = 0;

        for node in nodes {
            let node_neighbors = match neighbors.get(node.id.as_str()) {
                Some(n) => n,
                None => continue,
            };

            let k = node_neighbors.len();
            if k < 2 {
                continue;
            }

            // Count edges between neighbors
            let mut neighbor_edges = 0;
            let neighbor_list: Vec<_> = node_neighbors.iter().collect();
            for i in 0..neighbor_list.len() {
                for j in (i + 1)..neighbor_list.len() {
                    if let Some(n_neighbors) = neighbors.get(*neighbor_list[i]) {
                        if n_neighbors.contains(*neighbor_list[j]) {
                            neighbor_edges += 1;
                        }
                    }
                }
            }

            let max_edges = k * (k - 1) / 2;
            if max_edges > 0 {
                total_clustering += neighbor_edges as f64 / max_edges as f64;
                valid_nodes += 1;
            }
        }

        if valid_nodes > 0 {
            total_clustering / valid_nodes as f64
        } else {
            0.0
        }
    }

    /// Calculate concentration metrics for a node type.
    fn calculate_concentration(
        &self,
        nodes: &[NetworkNode],
        node_type: &str,
    ) -> ConcentrationMetrics {
        let type_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.node_type.to_lowercase() == node_type.to_lowercase())
            .collect();

        if type_nodes.is_empty() {
            return ConcentrationMetrics::default();
        }

        let total_volume: f64 = type_nodes.iter().map(|n| n.volume).sum();
        if total_volume <= 0.0 {
            return ConcentrationMetrics {
                total_count: type_nodes.len(),
                ..Default::default()
            };
        }

        // Sort by volume descending
        let mut volumes: Vec<f64> = type_nodes.iter().map(|n| n.volume).collect();
        volumes.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let top_1_share = volumes.first().map(|v| v / total_volume).unwrap_or(0.0);
        let top_5_share: f64 = volumes.iter().take(5).sum::<f64>() / total_volume;

        // Calculate HHI (sum of squared market shares)
        let hhi: f64 = volumes.iter().map(|v| (v / total_volume).powi(2)).sum();

        let exceeds_limits = top_1_share > self.thresholds.max_single_vendor_concentration
            || top_5_share > self.thresholds.max_top5_vendor_concentration;

        ConcentrationMetrics {
            total_count: type_nodes.len(),
            top_1_share,
            top_5_share,
            hhi,
            exceeds_limits,
        }
    }

    /// Calculate relationship strength statistics.
    fn calculate_strength_stats(&self, edges: &[NetworkEdge]) -> StrengthStats {
        if edges.is_empty() {
            return StrengthStats::default();
        }

        let strengths: Vec<f64> = edges.iter().map(|e| e.strength).collect();
        let n = strengths.len() as f64;

        let min = strengths.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = strengths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = strengths.iter().sum::<f64>() / n;
        let variance = strengths.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let strong_count = strengths.iter().filter(|&&s| s >= 0.7).count();
        let moderate_count = strengths
            .iter()
            .filter(|&&s| (0.4..0.7).contains(&s))
            .count();
        let weak_count = strengths
            .iter()
            .filter(|&&s| (0.1..0.4).contains(&s))
            .count();
        let dormant_count = strengths.iter().filter(|&&s| s < 0.1).count();

        StrengthStats {
            min,
            max,
            mean,
            std_dev,
            strong_count,
            moderate_count,
            weak_count,
            dormant_count,
        }
    }

    /// Check evaluation against thresholds.
    fn check_thresholds(&self, eval: &mut NetworkEvaluation) {
        eval.issues.clear();

        // Check connectivity
        if eval.connectivity_ratio < self.thresholds.connectivity_min {
            eval.issues.push(format!(
                "Connectivity ratio {:.2} < {:.2} (threshold)",
                eval.connectivity_ratio, self.thresholds.connectivity_min
            ));
        }

        // Check power law alpha
        if let Some(alpha) = eval.power_law_alpha {
            if alpha < self.thresholds.power_law_alpha_min
                || alpha > self.thresholds.power_law_alpha_max
            {
                eval.issues.push(format!(
                    "Power law alpha {:.2} not in range [{:.1}, {:.1}]",
                    alpha, self.thresholds.power_law_alpha_min, self.thresholds.power_law_alpha_max
                ));
            }
        }

        // Check clustering coefficient
        if eval.clustering_coefficient < self.thresholds.clustering_min
            || eval.clustering_coefficient > self.thresholds.clustering_max
        {
            eval.issues.push(format!(
                "Clustering coefficient {:.3} not in range [{:.2}, {:.2}]",
                eval.clustering_coefficient,
                self.thresholds.clustering_min,
                self.thresholds.clustering_max
            ));
        }

        // Check vendor concentration
        if eval.vendor_concentration.exceeds_limits {
            if eval.vendor_concentration.top_1_share
                > self.thresholds.max_single_vendor_concentration
            {
                eval.issues.push(format!(
                    "Single vendor concentration {:.2}% > {:.0}% (limit)",
                    eval.vendor_concentration.top_1_share * 100.0,
                    self.thresholds.max_single_vendor_concentration * 100.0
                ));
            }
            if eval.vendor_concentration.top_5_share > self.thresholds.max_top5_vendor_concentration
            {
                eval.issues.push(format!(
                    "Top 5 vendor concentration {:.2}% > {:.0}% (limit)",
                    eval.vendor_concentration.top_5_share * 100.0,
                    self.thresholds.max_top5_vendor_concentration * 100.0
                ));
            }
        }

        // Check cross-process link rate
        if eval.cross_process_link_rate < self.thresholds.min_cross_process_link_rate {
            eval.issues.push(format!(
                "Cross-process link rate {:.2}% < {:.0}% (threshold)",
                eval.cross_process_link_rate * 100.0,
                self.thresholds.min_cross_process_link_rate * 100.0
            ));
        }

        eval.passes = eval.issues.is_empty();
    }
}

impl Default for NetworkEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_nodes() -> Vec<NetworkNode> {
        vec![
            NetworkNode {
                id: "company".to_string(),
                node_type: "company".to_string(),
                volume: 1000000.0,
            },
            NetworkNode {
                id: "vendor1".to_string(),
                node_type: "vendor".to_string(),
                volume: 100000.0,
            },
            NetworkNode {
                id: "vendor2".to_string(),
                node_type: "vendor".to_string(),
                volume: 80000.0,
            },
            NetworkNode {
                id: "vendor3".to_string(),
                node_type: "vendor".to_string(),
                volume: 60000.0,
            },
            NetworkNode {
                id: "customer1".to_string(),
                node_type: "customer".to_string(),
                volume: 150000.0,
            },
            NetworkNode {
                id: "customer2".to_string(),
                node_type: "customer".to_string(),
                volume: 120000.0,
            },
        ]
    }

    fn create_test_edges() -> Vec<NetworkEdge> {
        vec![
            NetworkEdge {
                from_id: "company".to_string(),
                to_id: "vendor1".to_string(),
                strength: 0.8,
                volume: 100000.0,
            },
            NetworkEdge {
                from_id: "company".to_string(),
                to_id: "vendor2".to_string(),
                strength: 0.6,
                volume: 80000.0,
            },
            NetworkEdge {
                from_id: "company".to_string(),
                to_id: "vendor3".to_string(),
                strength: 0.4,
                volume: 60000.0,
            },
            NetworkEdge {
                from_id: "company".to_string(),
                to_id: "customer1".to_string(),
                strength: 0.9,
                volume: 150000.0,
            },
            NetworkEdge {
                from_id: "company".to_string(),
                to_id: "customer2".to_string(),
                strength: 0.7,
                volume: 120000.0,
            },
            // Some vendor-vendor relationships
            NetworkEdge {
                from_id: "vendor1".to_string(),
                to_id: "vendor2".to_string(),
                strength: 0.3,
                volume: 20000.0,
            },
        ]
    }

    #[test]
    fn test_network_evaluation_basic() {
        let nodes = create_test_nodes();
        let edges = create_test_edges();

        let evaluator = NetworkEvaluator::new();
        let eval = evaluator.evaluate(&nodes, &edges, 10, 30);

        assert_eq!(eval.node_count, 6);
        assert_eq!(eval.edge_count, 6);
        assert!(eval.connectivity_ratio > 0.0);
    }

    #[test]
    fn test_connectivity_calculation() {
        let nodes = create_test_nodes();
        let edges = create_test_edges();

        let evaluator = NetworkEvaluator::new();
        let connectivity = evaluator.calculate_connectivity(&nodes, &edges);

        // All nodes are connected through company
        assert_eq!(connectivity, 1.0);
    }

    #[test]
    fn test_concentration_metrics() {
        let nodes = create_test_nodes();

        let evaluator = NetworkEvaluator::new();
        let vendor_conc = evaluator.calculate_concentration(&nodes, "vendor");

        assert_eq!(vendor_conc.total_count, 3);
        assert!(vendor_conc.top_1_share > 0.0);
        assert!(vendor_conc.top_5_share > 0.0);
        assert!(vendor_conc.hhi > 0.0);
    }

    #[test]
    fn test_strength_stats() {
        let edges = create_test_edges();

        let evaluator = NetworkEvaluator::new();
        let stats = evaluator.calculate_strength_stats(&edges);

        assert!(stats.min > 0.0);
        assert!(stats.max <= 1.0);
        assert!(stats.mean > 0.0);
        assert!(stats.strong_count > 0); // We have some strong relationships
    }

    #[test]
    fn test_empty_graph() {
        let evaluator = NetworkEvaluator::new();
        let eval = evaluator.evaluate(&[], &[], 0, 0);

        assert!(!eval.passes);
        assert!(!eval.issues.is_empty());
    }
}
