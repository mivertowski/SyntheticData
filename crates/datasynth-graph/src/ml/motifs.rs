//! Subgraph and motif detection for fraud pattern identification.
//!
//! This module detects common fraud-related graph patterns:
//! - Circular flows (money laundering, round-tripping)
//! - Star patterns (hub entities)
//! - Back-and-forth transactions (structuring)
//! - Cliques (collusion networks)
//! - Chain patterns (layering)
//! - Funnel patterns (aggregation schemes)

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::{EdgeId, Graph, NodeId};

/// Types of graph motifs relevant for anomaly detection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphMotif {
    /// Circular transaction flow (A -> B -> C -> A).
    CircularFlow {
        /// Number of nodes in the cycle.
        length: usize,
    },
    /// Star pattern with central hub node.
    StarPattern {
        /// Minimum number of spoke edges.
        min_spokes: usize,
    },
    /// Back-and-forth transactions between two nodes.
    BackAndForth,
    /// Fully connected subgraph.
    Clique {
        /// Size of the clique.
        size: usize,
    },
    /// Linear chain of transactions.
    Chain {
        /// Length of the chain.
        length: usize,
    },
    /// Multiple sources flowing to single target (funnel/aggregation).
    FunnelPattern {
        /// Number of source nodes.
        sources: usize,
    },
}

impl GraphMotif {
    /// Returns the motif type name as a string.
    pub fn name(&self) -> &str {
        match self {
            GraphMotif::CircularFlow { .. } => "circular_flow",
            GraphMotif::StarPattern { .. } => "star_pattern",
            GraphMotif::BackAndForth => "back_and_forth",
            GraphMotif::Clique { .. } => "clique",
            GraphMotif::Chain { .. } => "chain",
            GraphMotif::FunnelPattern { .. } => "funnel_pattern",
        }
    }
}

/// Configuration for motif detection.
#[derive(Debug, Clone)]
pub struct MotifConfig {
    /// Maximum cycle length to detect.
    pub max_cycle_length: usize,
    /// Minimum number of spokes for star pattern detection.
    pub min_star_spokes: usize,
    /// Whether to detect back-and-forth patterns.
    pub detect_back_and_forth: bool,
    /// Maximum clique size to detect.
    pub max_clique_size: usize,
    /// Minimum chain length to detect.
    pub min_chain_length: usize,
    /// Optional time window for temporal filtering (days).
    pub time_window_days: Option<i64>,
    /// Minimum edge weight for consideration.
    pub min_edge_weight: f64,
    /// Maximum number of results per motif type.
    pub max_results_per_type: usize,
}

impl Default for MotifConfig {
    fn default() -> Self {
        Self {
            max_cycle_length: 5,
            min_star_spokes: 5,
            detect_back_and_forth: true,
            max_clique_size: 4,
            min_chain_length: 3,
            time_window_days: None,
            min_edge_weight: 0.0,
            max_results_per_type: 1000,
        }
    }
}

/// A detected circular flow pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularFlow {
    /// Nodes in the cycle (ordered).
    pub nodes: Vec<NodeId>,
    /// Edges forming the cycle.
    pub edges: Vec<EdgeId>,
    /// Total weight of edges in the cycle.
    pub total_weight: f64,
    /// Time span in days (if temporal data available).
    pub time_span_days: Option<i64>,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// A generic motif instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotifInstance {
    /// Type of motif.
    pub motif_type: GraphMotif,
    /// Nodes involved in the motif.
    pub nodes: Vec<NodeId>,
    /// Edges involved in the motif.
    pub edges: Vec<EdgeId>,
    /// Total weight of edges.
    pub total_weight: f64,
    /// Time span in days (if available).
    pub time_span_days: Option<i64>,
    /// Confidence score.
    pub confidence: f64,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl MotifInstance {
    /// Creates a new motif instance.
    pub fn new(
        motif_type: GraphMotif,
        nodes: Vec<NodeId>,
        edges: Vec<EdgeId>,
        total_weight: f64,
    ) -> Self {
        Self {
            motif_type,
            nodes,
            edges,
            total_weight,
            time_span_days: None,
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Sets the time span.
    pub fn with_time_span(mut self, days: i64) -> Self {
        self.time_span_days = Some(days);
        self
    }

    /// Sets the confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Results of motif detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MotifDetectionResult {
    /// All detected motif instances grouped by type.
    pub motifs: HashMap<String, Vec<MotifInstance>>,
    /// Per-node motif participation counts.
    pub node_motif_counts: HashMap<NodeId, HashMap<String, usize>>,
    /// Total circular flows detected.
    pub total_circular_flows: usize,
    /// Total star patterns detected.
    pub total_star_patterns: usize,
    /// Total back-and-forth patterns detected.
    pub total_back_and_forth: usize,
    /// Total cliques detected.
    pub total_cliques: usize,
    /// Total chains detected.
    pub total_chains: usize,
    /// Total funnel patterns detected.
    pub total_funnel_patterns: usize,
}

impl MotifDetectionResult {
    /// Returns per-node motif feature counts.
    pub fn node_features(&self, node_id: NodeId) -> Vec<f64> {
        let counts = self.node_motif_counts.get(&node_id);

        let get_count = |name: &str| counts.and_then(|c| c.get(name)).copied().unwrap_or(0) as f64;

        vec![
            get_count("circular_flow"),
            get_count("star_pattern"),
            get_count("back_and_forth"),
            get_count("clique"),
            get_count("funnel_pattern"),
        ]
    }

    /// Returns the feature dimension for motif features.
    pub fn feature_dim() -> usize {
        5 // circular, star, back_and_forth, clique, funnel
    }
}

/// Detects all configured motif patterns in a graph.
pub fn detect_motifs(graph: &Graph, config: &MotifConfig) -> MotifDetectionResult {
    let mut result = MotifDetectionResult::default();

    // Detect circular flows
    let circular_flows = find_circular_flows(graph, config);
    result.total_circular_flows = circular_flows.len();
    add_motifs_to_result(&mut result, circular_flows);

    // Detect star patterns
    let star_patterns = find_star_patterns(graph, config);
    result.total_star_patterns = star_patterns.len();
    add_motifs_to_result(&mut result, star_patterns);

    // Detect back-and-forth patterns
    if config.detect_back_and_forth {
        let back_and_forth = find_back_and_forth(graph, config);
        result.total_back_and_forth = back_and_forth.len();
        add_motifs_to_result(&mut result, back_and_forth);
    }

    // Detect cliques
    let cliques = find_cliques(graph, config);
    result.total_cliques = cliques.len();
    add_motifs_to_result(&mut result, cliques);

    // Detect funnel patterns
    let funnels = find_funnel_patterns(graph, config);
    result.total_funnel_patterns = funnels.len();
    add_motifs_to_result(&mut result, funnels);

    result
}

/// Adds motif instances to the result and updates node counts.
fn add_motifs_to_result(result: &mut MotifDetectionResult, motifs: Vec<MotifInstance>) {
    for motif in motifs {
        let type_name = motif.motif_type.name().to_string();

        // Update node participation counts
        for &node_id in &motif.nodes {
            result
                .node_motif_counts
                .entry(node_id)
                .or_default()
                .entry(type_name.clone())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Add to motif collection
        result.motifs.entry(type_name).or_default().push(motif);
    }
}

/// Finds circular flow patterns using DFS with path tracking.
pub fn find_circular_flows(graph: &Graph, config: &MotifConfig) -> Vec<MotifInstance> {
    let mut cycles = Vec::new();
    let mut seen_cycles: HashSet<Vec<NodeId>> = HashSet::new();

    // For each node, try to find cycles starting from it
    for &start_node in graph.nodes.keys() {
        find_cycles_from_node(
            graph,
            start_node,
            config.max_cycle_length,
            config.min_edge_weight,
            &mut cycles,
            &mut seen_cycles,
        );

        // Early termination if we have enough results
        if cycles.len() >= config.max_results_per_type {
            break;
        }
    }

    cycles.truncate(config.max_results_per_type);
    cycles
}

/// DFS helper to find cycles starting from a node.
fn find_cycles_from_node(
    graph: &Graph,
    start: NodeId,
    max_length: usize,
    min_weight: f64,
    cycles: &mut Vec<MotifInstance>,
    seen_cycles: &mut HashSet<Vec<NodeId>>,
) {
    let mut path = vec![start];
    let mut path_edges = Vec::new();
    let mut visited = HashSet::new();
    visited.insert(start);

    dfs_find_cycles(
        graph,
        start,
        start,
        &mut path,
        &mut path_edges,
        &mut visited,
        max_length,
        min_weight,
        cycles,
        seen_cycles,
    );
}

/// Recursive DFS for cycle detection.
#[allow(clippy::too_many_arguments)]
fn dfs_find_cycles(
    graph: &Graph,
    current: NodeId,
    start: NodeId,
    path: &mut Vec<NodeId>,
    path_edges: &mut Vec<EdgeId>,
    visited: &mut HashSet<NodeId>,
    max_length: usize,
    min_weight: f64,
    cycles: &mut Vec<MotifInstance>,
    seen_cycles: &mut HashSet<Vec<NodeId>>,
) {
    // Check length limit
    if path.len() > max_length {
        return;
    }

    // Get outgoing edges
    for edge in graph.outgoing_edges(current) {
        // Skip edges below weight threshold
        if edge.weight < min_weight {
            continue;
        }

        let target = edge.target;

        // Found a cycle back to start
        if target == start && path.len() >= 3 {
            // Create canonical representation (smallest node first)
            let canonical = canonicalize_cycle(path);

            if !seen_cycles.contains(&canonical) {
                seen_cycles.insert(canonical);

                let total_weight: f64 = path_edges
                    .iter()
                    .filter_map(|&id| graph.get_edge(id))
                    .map(|e| e.weight)
                    .sum::<f64>()
                    + edge.weight;

                let mut edges = path_edges.clone();
                edges.push(edge.id);

                let motif = MotifInstance::new(
                    GraphMotif::CircularFlow { length: path.len() },
                    path.clone(),
                    edges,
                    total_weight,
                );

                // Calculate time span if timestamps available
                let motif = if let Some(span) = calculate_time_span(&motif.edges, graph) {
                    motif.with_time_span(span)
                } else {
                    motif
                };

                cycles.push(motif);
            }
        }
        // Continue DFS if not visited
        else if !visited.contains(&target) {
            visited.insert(target);
            path.push(target);
            path_edges.push(edge.id);

            dfs_find_cycles(
                graph,
                target,
                start,
                path,
                path_edges,
                visited,
                max_length,
                min_weight,
                cycles,
                seen_cycles,
            );

            path.pop();
            path_edges.pop();
            visited.remove(&target);
        }
    }
}

/// Creates a canonical representation of a cycle for deduplication.
fn canonicalize_cycle(path: &[NodeId]) -> Vec<NodeId> {
    if path.is_empty() {
        return Vec::new();
    }

    // Find the position of the minimum node
    let min_pos = path
        .iter()
        .enumerate()
        .min_by_key(|(_, &node)| node)
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Rotate so minimum is first
    let mut canonical: Vec<NodeId> = path[min_pos..].to_vec();
    canonical.extend(&path[..min_pos]);

    // Also consider reverse direction and pick lexicographically smaller
    let mut reversed = canonical.clone();
    reversed.reverse();

    // Rotate reversed to start with minimum
    if reversed.len() > 1 {
        let last = reversed.pop().expect("len > 1 guarantees non-empty");
        reversed.insert(0, last);
    }

    if reversed < canonical {
        reversed
    } else {
        canonical
    }
}

/// Finds star patterns (hub nodes with many connections).
pub fn find_star_patterns(graph: &Graph, config: &MotifConfig) -> Vec<MotifInstance> {
    let mut stars = Vec::new();

    for &node_id in graph.nodes.keys() {
        let out_edges = graph.outgoing_edges(node_id);
        let in_edges = graph.incoming_edges(node_id);

        // Check outgoing star (hub sends to many)
        if out_edges.len() >= config.min_star_spokes {
            let edges: Vec<EdgeId> = out_edges.iter().map(|e| e.id).collect();
            let targets: Vec<NodeId> = out_edges.iter().map(|e| e.target).collect();
            let total_weight: f64 = out_edges.iter().map(|e| e.weight).sum();

            let mut nodes = vec![node_id];
            nodes.extend(&targets);

            let motif = MotifInstance::new(
                GraphMotif::StarPattern {
                    min_spokes: out_edges.len(),
                },
                nodes,
                edges,
                total_weight,
            )
            .with_metadata("hub_node", &node_id.to_string())
            .with_metadata("direction", "outgoing");

            stars.push(motif);
        }

        // Check incoming star (hub receives from many)
        if in_edges.len() >= config.min_star_spokes {
            let edges: Vec<EdgeId> = in_edges.iter().map(|e| e.id).collect();
            let sources: Vec<NodeId> = in_edges.iter().map(|e| e.source).collect();
            let total_weight: f64 = in_edges.iter().map(|e| e.weight).sum();

            let mut nodes = vec![node_id];
            nodes.extend(&sources);

            let motif = MotifInstance::new(
                GraphMotif::StarPattern {
                    min_spokes: in_edges.len(),
                },
                nodes,
                edges,
                total_weight,
            )
            .with_metadata("hub_node", &node_id.to_string())
            .with_metadata("direction", "incoming");

            stars.push(motif);
        }

        if stars.len() >= config.max_results_per_type {
            break;
        }
    }

    stars.truncate(config.max_results_per_type);
    stars
}

/// Finds back-and-forth transaction patterns.
pub fn find_back_and_forth(graph: &Graph, config: &MotifConfig) -> Vec<MotifInstance> {
    let mut patterns = Vec::new();
    let mut seen_pairs: HashSet<(NodeId, NodeId)> = HashSet::new();

    for (&edge_id, edge) in &graph.edges {
        if edge.weight < config.min_edge_weight {
            continue;
        }

        let (a, b) = (edge.source.min(edge.target), edge.source.max(edge.target));

        if seen_pairs.contains(&(a, b)) {
            continue;
        }

        // Look for reverse edge
        let reverse_edges: Vec<&_> = graph
            .outgoing_edges(edge.target)
            .into_iter()
            .filter(|e| e.target == edge.source && e.weight >= config.min_edge_weight)
            .collect();

        if !reverse_edges.is_empty() {
            seen_pairs.insert((a, b));

            let mut edges = vec![edge_id];
            edges.extend(reverse_edges.iter().map(|e| e.id));

            let total_weight = edge.weight + reverse_edges.iter().map(|e| e.weight).sum::<f64>();

            let motif = MotifInstance::new(
                GraphMotif::BackAndForth,
                vec![edge.source, edge.target],
                edges,
                total_weight,
            )
            .with_metadata("forward_count", "1")
            .with_metadata("reverse_count", &reverse_edges.len().to_string());

            patterns.push(motif);

            if patterns.len() >= config.max_results_per_type {
                break;
            }
        }
    }

    patterns
}

/// Finds clique patterns (fully connected subgraphs).
pub fn find_cliques(graph: &Graph, config: &MotifConfig) -> Vec<MotifInstance> {
    let mut cliques = Vec::new();

    if config.max_clique_size < 3 {
        return cliques;
    }

    // Build undirected adjacency for clique detection
    let mut adjacency: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

    for edge in graph.edges.values() {
        adjacency
            .entry(edge.source)
            .or_default()
            .insert(edge.target);
        adjacency
            .entry(edge.target)
            .or_default()
            .insert(edge.source);
    }

    // Find triangles first (cliques of size 3)
    let mut seen_cliques: HashSet<Vec<NodeId>> = HashSet::new();
    let nodes: Vec<NodeId> = graph.nodes.keys().copied().collect();

    for &a in &nodes {
        if cliques.len() >= config.max_results_per_type {
            break;
        }
        let neighbors_a = match adjacency.get(&a) {
            Some(n) => n,
            None => continue,
        };

        for &b in neighbors_a {
            if b <= a {
                continue;
            }

            let neighbors_b = match adjacency.get(&b) {
                Some(n) => n,
                None => continue,
            };

            // Find common neighbors (triangle)
            for &c in neighbors_a {
                if c <= b {
                    continue;
                }

                if neighbors_b.contains(&c) {
                    let mut clique_nodes = vec![a, b, c];
                    clique_nodes.sort();

                    if !seen_cliques.contains(&clique_nodes) {
                        seen_cliques.insert(clique_nodes.clone());

                        // Find edges in the clique
                        let edges = find_edges_between_nodes(graph, &clique_nodes);
                        let total_weight: f64 = edges
                            .iter()
                            .filter_map(|&id| graph.get_edge(id))
                            .map(|e| e.weight)
                            .sum();

                        let motif = MotifInstance::new(
                            GraphMotif::Clique { size: 3 },
                            clique_nodes,
                            edges,
                            total_weight,
                        );

                        cliques.push(motif);
                    }
                }
            }
        }
    }

    cliques.truncate(config.max_results_per_type);
    cliques
}

/// Finds edges between a set of nodes.
fn find_edges_between_nodes(graph: &Graph, nodes: &[NodeId]) -> Vec<EdgeId> {
    let node_set: HashSet<NodeId> = nodes.iter().copied().collect();
    let mut edges = Vec::new();

    for &node in nodes {
        for edge in graph.outgoing_edges(node) {
            if node_set.contains(&edge.target) {
                edges.push(edge.id);
            }
        }
    }

    edges
}

/// Finds funnel patterns (multiple sources flowing to single target).
pub fn find_funnel_patterns(graph: &Graph, config: &MotifConfig) -> Vec<MotifInstance> {
    let mut funnels = Vec::new();

    for &node_id in graph.nodes.keys() {
        let in_edges = graph.incoming_edges(node_id);

        // A funnel has many sources with few or no outgoing edges
        if in_edges.len() >= config.min_star_spokes {
            let sources: Vec<NodeId> = in_edges.iter().map(|e| e.source).collect();

            // Check if sources are "leaf-like" (few outgoing edges each)
            let leaf_sources: Vec<NodeId> = sources
                .iter()
                .filter(|&&s| graph.out_degree(s) <= 2)
                .copied()
                .collect();

            // Funnel requires most sources to be leaf-like
            if leaf_sources.len() >= config.min_star_spokes / 2 {
                let edges: Vec<EdgeId> = in_edges.iter().map(|e| e.id).collect();
                let total_weight: f64 = in_edges.iter().map(|e| e.weight).sum();

                let mut nodes = vec![node_id];
                nodes.extend(&sources);

                let motif = MotifInstance::new(
                    GraphMotif::FunnelPattern {
                        sources: sources.len(),
                    },
                    nodes,
                    edges,
                    total_weight,
                )
                .with_metadata("target_node", &node_id.to_string())
                .with_metadata("leaf_source_count", &leaf_sources.len().to_string());

                funnels.push(motif);

                if funnels.len() >= config.max_results_per_type {
                    break;
                }
            }
        }
    }

    funnels
}

/// Calculates the time span of edges in days.
fn calculate_time_span(edge_ids: &[EdgeId], graph: &Graph) -> Option<i64> {
    let dates: Vec<NaiveDate> = edge_ids
        .iter()
        .filter_map(|&id| graph.get_edge(id))
        .filter_map(|e| e.timestamp)
        .collect();

    if dates.len() < 2 {
        return None;
    }

    let min_date = dates.iter().min()?;
    let max_date = dates.iter().max()?;

    Some((*max_date - *min_date).num_days())
}

/// Computes motif-based features for all nodes.
pub fn compute_motif_features(graph: &Graph, config: &MotifConfig) -> HashMap<NodeId, Vec<f64>> {
    let result = detect_motifs(graph, config);
    let mut features = HashMap::new();

    for &node_id in graph.nodes.keys() {
        features.insert(node_id, result.node_features(node_id));
    }

    features
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{GraphEdge, GraphNode, GraphType, NodeType};
    use crate::EdgeType;

    fn create_cycle_graph() -> Graph {
        let mut graph = Graph::new("test", GraphType::Transaction);

        // Create a simple cycle: 1 -> 2 -> 3 -> 1
        let n1 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "A".to_string(),
            "A".to_string(),
        ));
        let n2 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "B".to_string(),
            "B".to_string(),
        ));
        let n3 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "C".to_string(),
            "C".to_string(),
        ));

        graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction).with_weight(100.0));
        graph.add_edge(GraphEdge::new(0, n2, n3, EdgeType::Transaction).with_weight(100.0));
        graph.add_edge(GraphEdge::new(0, n3, n1, EdgeType::Transaction).with_weight(100.0));

        graph
    }

    fn create_star_graph() -> Graph {
        let mut graph = Graph::new("test", GraphType::Transaction);

        // Create a star pattern with hub node
        let hub = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "Hub".to_string(),
            "Hub".to_string(),
        ));

        for i in 1..=6 {
            let spoke = graph.add_node(GraphNode::new(
                0,
                NodeType::Account,
                format!("Spoke{}", i),
                format!("Spoke{}", i),
            ));
            graph.add_edge(GraphEdge::new(0, hub, spoke, EdgeType::Transaction).with_weight(100.0));
        }

        graph
    }

    fn create_back_and_forth_graph() -> Graph {
        let mut graph = Graph::new("test", GraphType::Transaction);

        let n1 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "A".to_string(),
            "A".to_string(),
        ));
        let n2 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "B".to_string(),
            "B".to_string(),
        ));

        // Bidirectional edges
        graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction).with_weight(100.0));
        graph.add_edge(GraphEdge::new(0, n2, n1, EdgeType::Transaction).with_weight(100.0));

        graph
    }

    #[test]
    fn test_find_circular_flows() {
        let graph = create_cycle_graph();
        let config = MotifConfig {
            max_cycle_length: 5,
            ..Default::default()
        };

        let cycles = find_circular_flows(&graph, &config);

        assert!(!cycles.is_empty());
        let cycle = &cycles[0];
        assert_eq!(cycle.nodes.len(), 3);
        assert!(cycle.total_weight > 0.0);
    }

    #[test]
    fn test_find_star_patterns() {
        let graph = create_star_graph();
        let config = MotifConfig {
            min_star_spokes: 5,
            ..Default::default()
        };

        let stars = find_star_patterns(&graph, &config);

        assert!(!stars.is_empty());
        let star = &stars[0];
        assert!(star.nodes.len() >= 6); // hub + 5+ spokes
    }

    #[test]
    fn test_find_back_and_forth() {
        let graph = create_back_and_forth_graph();
        let config = MotifConfig::default();

        let patterns = find_back_and_forth(&graph, &config);

        assert!(!patterns.is_empty());
        let pattern = &patterns[0];
        assert_eq!(pattern.nodes.len(), 2);
    }

    #[test]
    fn test_detect_motifs() {
        let graph = create_cycle_graph();
        let config = MotifConfig::default();

        let result = detect_motifs(&graph, &config);

        assert!(result.total_circular_flows > 0);
    }

    #[test]
    fn test_canonicalize_cycle() {
        let cycle1 = vec![3, 1, 2];
        let cycle2 = vec![1, 2, 3];
        let cycle3 = vec![2, 3, 1];

        let canonical1 = canonicalize_cycle(&cycle1);
        let canonical2 = canonicalize_cycle(&cycle2);
        let canonical3 = canonicalize_cycle(&cycle3);

        // All should produce the same canonical form
        assert_eq!(canonical1, canonical2);
        assert_eq!(canonical2, canonical3);
    }

    #[test]
    fn test_node_features() {
        let graph = create_cycle_graph();
        let config = MotifConfig::default();

        let result = detect_motifs(&graph, &config);
        let features = result.node_features(1);

        assert_eq!(features.len(), MotifDetectionResult::feature_dim());
    }

    #[test]
    fn test_motif_instance_builder() {
        let motif = MotifInstance::new(
            GraphMotif::CircularFlow { length: 3 },
            vec![1, 2, 3],
            vec![1, 2, 3],
            300.0,
        )
        .with_time_span(5)
        .with_confidence(0.95)
        .with_metadata("key", "value");

        assert_eq!(motif.time_span_days, Some(5));
        assert_eq!(motif.confidence, 0.95);
        assert_eq!(motif.metadata.get("key"), Some(&"value".to_string()));
    }
}
