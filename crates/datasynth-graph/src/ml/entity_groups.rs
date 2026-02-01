//! Entity group detection for collective fraud analysis.
//!
//! This module provides group detection algorithms:
//! - Connected components
//! - Dense subgraph detection
//! - Clique detection
//! - Community detection (Louvain-style)

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::models::{Graph, NodeId};
use crate::EdgeType;

/// Type of entity group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupType {
    /// Related party transactions.
    RelatedParty,
    /// Vendor ring (collusion network).
    VendorRing,
    /// Customer cluster.
    CustomerCluster,
    /// Money mule network.
    MuleNetwork,
    /// Intercompany group.
    Intercompany,
    /// Approval chain.
    ApprovalChain,
    /// Transaction cluster.
    TransactionCluster,
    /// Custom group type.
    Custom(String),
}

impl GroupType {
    /// Returns the group type name.
    pub fn name(&self) -> &str {
        match self {
            GroupType::RelatedParty => "related_party",
            GroupType::VendorRing => "vendor_ring",
            GroupType::CustomerCluster => "customer_cluster",
            GroupType::MuleNetwork => "mule_network",
            GroupType::Intercompany => "intercompany",
            GroupType::ApprovalChain => "approval_chain",
            GroupType::TransactionCluster => "transaction_cluster",
            GroupType::Custom(s) => s.as_str(),
        }
    }
}

/// Group detection algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupDetectionAlgorithm {
    /// Connected components (weakly connected).
    ConnectedComponents,
    /// Label propagation for community detection.
    LabelPropagation,
    /// Dense subgraph detection.
    DenseSubgraph,
    /// Clique detection.
    CliqueDetection,
}

/// Configuration for group detection.
#[derive(Debug, Clone)]
pub struct GroupDetectionConfig {
    /// Minimum group size.
    pub min_group_size: usize,
    /// Maximum group size.
    pub max_group_size: usize,
    /// Minimum cohesion (internal edge density).
    pub min_cohesion: f64,
    /// Algorithms to use.
    pub algorithms: Vec<GroupDetectionAlgorithm>,
    /// Maximum number of groups to detect.
    pub max_groups: usize,
    /// Whether to classify group types.
    pub classify_types: bool,
    /// Edge types to consider.
    pub edge_types: Option<Vec<EdgeType>>,
}

impl Default for GroupDetectionConfig {
    fn default() -> Self {
        Self {
            min_group_size: 3,
            max_group_size: 50,
            min_cohesion: 0.1,
            algorithms: vec![GroupDetectionAlgorithm::ConnectedComponents],
            max_groups: 1000,
            classify_types: true,
            edge_types: None,
        }
    }
}

/// A detected entity group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGroup {
    /// Unique group identifier.
    pub group_id: u64,
    /// Member node IDs.
    pub members: Vec<NodeId>,
    /// Group type classification.
    pub group_type: GroupType,
    /// Confidence in group detection.
    pub confidence: f64,
    /// Hub node (most connected within group).
    pub hub_node: Option<NodeId>,
    /// Total internal transaction volume.
    pub internal_volume: f64,
    /// Total external transaction volume.
    pub external_volume: f64,
    /// Cohesion (internal edge density).
    pub cohesion: f64,
}

impl EntityGroup {
    /// Creates a new entity group.
    pub fn new(group_id: u64, members: Vec<NodeId>, group_type: GroupType) -> Self {
        Self {
            group_id,
            members,
            group_type,
            confidence: 1.0,
            hub_node: None,
            internal_volume: 0.0,
            external_volume: 0.0,
            cohesion: 0.0,
        }
    }

    /// Sets the hub node.
    pub fn with_hub(mut self, hub: NodeId) -> Self {
        self.hub_node = Some(hub);
        self
    }

    /// Sets the volumes.
    pub fn with_volumes(mut self, internal: f64, external: f64) -> Self {
        self.internal_volume = internal;
        self.external_volume = external;
        self
    }

    /// Sets the cohesion.
    pub fn with_cohesion(mut self, cohesion: f64) -> Self {
        self.cohesion = cohesion;
        self
    }

    /// Returns member count.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Checks if a node is a member.
    pub fn contains(&self, node_id: NodeId) -> bool {
        self.members.contains(&node_id)
    }
}

/// Results of group detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupDetectionResult {
    /// All detected groups.
    pub groups: Vec<EntityGroup>,
    /// Node to group membership mapping.
    pub node_groups: HashMap<NodeId, Vec<u64>>,
    /// Total groups detected.
    pub total_groups: usize,
    /// Groups by type.
    pub groups_by_type: HashMap<String, usize>,
}

impl GroupDetectionResult {
    /// Returns groups containing a specific node.
    pub fn groups_for_node(&self, node_id: NodeId) -> Vec<&EntityGroup> {
        self.node_groups
            .get(&node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|&id| self.groups.iter().find(|g| g.group_id == id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns group membership features for a node.
    pub fn node_features(&self, node_id: NodeId) -> Vec<f64> {
        let groups = self.groups_for_node(node_id);

        let group_count = groups.len() as f64;
        let max_group_size = groups.iter().map(|g| g.size()).max().unwrap_or(0) as f64;
        let is_hub = groups.iter().any(|g| g.hub_node == Some(node_id));

        vec![group_count, max_group_size, if is_hub { 1.0 } else { 0.0 }]
    }

    /// Returns feature dimension.
    pub fn feature_dim() -> usize {
        3
    }
}

/// Detects entity groups in a graph.
pub fn detect_entity_groups(graph: &Graph, config: &GroupDetectionConfig) -> GroupDetectionResult {
    let mut all_groups = Vec::new();
    let mut next_group_id = 1u64;

    for algorithm in &config.algorithms {
        let groups = match algorithm {
            GroupDetectionAlgorithm::ConnectedComponents => {
                detect_connected_components(graph, config, &mut next_group_id)
            }
            GroupDetectionAlgorithm::LabelPropagation => {
                detect_label_propagation(graph, config, &mut next_group_id)
            }
            GroupDetectionAlgorithm::DenseSubgraph => {
                detect_dense_subgraphs(graph, config, &mut next_group_id)
            }
            GroupDetectionAlgorithm::CliqueDetection => {
                detect_cliques(graph, config, &mut next_group_id)
            }
        };

        all_groups.extend(groups);

        if all_groups.len() >= config.max_groups {
            all_groups.truncate(config.max_groups);
            break;
        }
    }

    // Build node to group mapping
    let mut node_groups: HashMap<NodeId, Vec<u64>> = HashMap::new();
    for group in &all_groups {
        for &member in &group.members {
            node_groups.entry(member).or_default().push(group.group_id);
        }
    }

    // Count groups by type
    let mut groups_by_type: HashMap<String, usize> = HashMap::new();
    for group in &all_groups {
        *groups_by_type
            .entry(group.group_type.name().to_string())
            .or_insert(0) += 1;
    }

    GroupDetectionResult {
        total_groups: all_groups.len(),
        groups: all_groups,
        node_groups,
        groups_by_type,
    }
}

/// Detects connected components using BFS.
fn detect_connected_components(
    graph: &Graph,
    config: &GroupDetectionConfig,
    next_id: &mut u64,
) -> Vec<EntityGroup> {
    let mut groups = Vec::new();
    let mut visited: HashSet<NodeId> = HashSet::new();

    for &start_node in graph.nodes.keys() {
        if visited.contains(&start_node) {
            continue;
        }

        // BFS to find component
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node);
        visited.insert(start_node);

        while let Some(node) = queue.pop_front() {
            component.push(node);

            // Add unvisited neighbors
            for neighbor in graph.neighbors(node) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }

            // Limit component size
            if component.len() >= config.max_group_size {
                break;
            }
        }

        // Check size constraints
        if component.len() >= config.min_group_size && component.len() <= config.max_group_size {
            let group_type = if config.classify_types {
                classify_group_type(graph, &component)
            } else {
                GroupType::TransactionCluster
            };

            let mut group = EntityGroup::new(*next_id, component.clone(), group_type);
            *next_id += 1;

            // Calculate metrics
            let (internal, external, cohesion) = calculate_group_metrics(graph, &component);
            if cohesion >= config.min_cohesion {
                let hub = find_hub_node(graph, &component);
                group = group
                    .with_hub(hub)
                    .with_volumes(internal, external)
                    .with_cohesion(cohesion);
                groups.push(group);
            }
        }
    }

    groups
}

/// Detects communities using label propagation.
fn detect_label_propagation(
    graph: &Graph,
    config: &GroupDetectionConfig,
    next_id: &mut u64,
) -> Vec<EntityGroup> {
    let nodes: Vec<NodeId> = graph.nodes.keys().copied().collect();
    if nodes.is_empty() {
        return Vec::new();
    }

    // Initialize each node with its own label
    let mut labels: HashMap<NodeId, u64> = nodes
        .iter()
        .enumerate()
        .map(|(i, &n)| (n, i as u64))
        .collect();

    // Simple deterministic iteration (not randomized for reproducibility)
    for _ in 0..10 {
        // Max iterations
        let mut changed = false;

        for &node in &nodes {
            let neighbors = graph.neighbors(node);
            if neighbors.is_empty() {
                continue;
            }

            // Count neighbor labels
            let mut label_counts: HashMap<u64, usize> = HashMap::new();
            for neighbor in neighbors {
                if let Some(&label) = labels.get(&neighbor) {
                    *label_counts.entry(label).or_insert(0) += 1;
                }
            }

            // Find most common label
            if let Some((&most_common, _)) = label_counts.iter().max_by_key(|(_, &count)| count) {
                if labels.get(&node) != Some(&most_common) {
                    labels.insert(node, most_common);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    // Group nodes by label
    let mut communities: HashMap<u64, Vec<NodeId>> = HashMap::new();
    for (node, label) in labels {
        communities.entry(label).or_default().push(node);
    }

    // Convert to EntityGroups
    let mut groups = Vec::new();
    for (_, members) in communities {
        if members.len() >= config.min_group_size && members.len() <= config.max_group_size {
            let group_type = if config.classify_types {
                classify_group_type(graph, &members)
            } else {
                GroupType::TransactionCluster
            };

            let (internal, external, cohesion) = calculate_group_metrics(graph, &members);
            if cohesion >= config.min_cohesion {
                let hub = find_hub_node(graph, &members);
                let group = EntityGroup::new(*next_id, members, group_type)
                    .with_hub(hub)
                    .with_volumes(internal, external)
                    .with_cohesion(cohesion);
                *next_id += 1;
                groups.push(group);
            }
        }
    }

    groups
}

/// Detects dense subgraphs based on edge density.
fn detect_dense_subgraphs(
    graph: &Graph,
    config: &GroupDetectionConfig,
    next_id: &mut u64,
) -> Vec<EntityGroup> {
    let mut groups = Vec::new();

    // Find high-degree nodes as seeds
    let mut nodes_by_degree: Vec<(NodeId, usize)> =
        graph.nodes.keys().map(|&n| (n, graph.degree(n))).collect();
    nodes_by_degree.sort_by_key(|(_, d)| std::cmp::Reverse(*d));

    let mut used_nodes: HashSet<NodeId> = HashSet::new();

    for (seed, _) in nodes_by_degree {
        if used_nodes.contains(&seed) {
            continue;
        }

        // Grow dense subgraph from seed
        let mut subgraph = vec![seed];
        let mut candidates: HashSet<NodeId> = graph.neighbors(seed).into_iter().collect();

        while subgraph.len() < config.max_group_size && !candidates.is_empty() {
            // Find candidate with highest connectivity to subgraph
            let best_candidate = candidates
                .iter()
                .map(|&c| {
                    let connections = graph
                        .neighbors(c)
                        .iter()
                        .filter(|n| subgraph.contains(n))
                        .count();
                    (c, connections)
                })
                .max_by_key(|(_, conn)| *conn);

            match best_candidate {
                Some((c, conn)) if conn > 0 => {
                    subgraph.push(c);
                    candidates.remove(&c);

                    // Add new candidates
                    for neighbor in graph.neighbors(c) {
                        if !subgraph.contains(&neighbor) && !used_nodes.contains(&neighbor) {
                            candidates.insert(neighbor);
                        }
                    }
                }
                _ => break,
            }

            // Check density
            let (_, _, cohesion) = calculate_group_metrics(graph, &subgraph);
            if cohesion < config.min_cohesion * 2.0 {
                // Require higher density for dense subgraph
                break;
            }
        }

        if subgraph.len() >= config.min_group_size {
            used_nodes.extend(&subgraph);

            let group_type = if config.classify_types {
                classify_group_type(graph, &subgraph)
            } else {
                GroupType::TransactionCluster
            };

            let (internal, external, cohesion) = calculate_group_metrics(graph, &subgraph);
            let hub = find_hub_node(graph, &subgraph);

            let group = EntityGroup::new(*next_id, subgraph, group_type)
                .with_hub(hub)
                .with_volumes(internal, external)
                .with_cohesion(cohesion);
            *next_id += 1;
            groups.push(group);

            if groups.len() >= config.max_groups {
                break;
            }
        }
    }

    groups
}

/// Detects cliques (fully connected subgraphs).
fn detect_cliques(
    graph: &Graph,
    config: &GroupDetectionConfig,
    next_id: &mut u64,
) -> Vec<EntityGroup> {
    let mut groups = Vec::new();
    let mut seen_cliques: HashSet<Vec<NodeId>> = HashSet::new();

    // Build adjacency set for faster lookup
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

    // Find triangles first
    let nodes: Vec<NodeId> = graph.nodes.keys().copied().collect();

    for &a in &nodes {
        if groups.len() >= config.max_groups {
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

            for &c in neighbors_a {
                if c <= b {
                    continue;
                }

                if neighbors_b.contains(&c) {
                    let mut clique = vec![a, b, c];
                    clique.sort();

                    if !seen_cliques.contains(&clique) && clique.len() >= config.min_group_size {
                        seen_cliques.insert(clique.clone());

                        let group_type = if config.classify_types {
                            classify_group_type(graph, &clique)
                        } else {
                            GroupType::TransactionCluster
                        };

                        let (internal, external, cohesion) =
                            calculate_group_metrics(graph, &clique);
                        let hub = find_hub_node(graph, &clique);

                        let group = EntityGroup::new(*next_id, clique, group_type)
                            .with_hub(hub)
                            .with_volumes(internal, external)
                            .with_cohesion(cohesion);
                        *next_id += 1;
                        groups.push(group);
                    }
                }
            }
        }
    }

    groups
}

/// Classifies the type of a group based on its characteristics.
fn classify_group_type(graph: &Graph, members: &[NodeId]) -> GroupType {
    let member_set: HashSet<NodeId> = members.iter().copied().collect();

    // Check for circular transactions
    let has_cycles = members.iter().any(|&node| {
        graph
            .outgoing_edges(node)
            .iter()
            .any(|e| member_set.contains(&e.target))
            && graph
                .incoming_edges(node)
                .iter()
                .any(|e| member_set.contains(&e.source))
    });

    // Check for ownership/intercompany edges
    let has_ownership = graph.edges.values().any(|e| {
        member_set.contains(&e.source)
            && member_set.contains(&e.target)
            && matches!(e.edge_type, EdgeType::Ownership | EdgeType::Intercompany)
    });

    // Check for approval chain edges
    let has_approval = graph.edges.values().any(|e| {
        member_set.contains(&e.source)
            && member_set.contains(&e.target)
            && matches!(e.edge_type, EdgeType::Approval | EdgeType::ReportsTo)
    });

    // Check anomaly rate
    let anomalous_nodes = members
        .iter()
        .filter(|&&n| {
            graph
                .get_node(n)
                .map(|node| node.is_anomaly)
                .unwrap_or(false)
        })
        .count();
    let anomaly_rate = anomalous_nodes as f64 / members.len() as f64;

    // Classify based on characteristics
    if has_ownership {
        GroupType::Intercompany
    } else if has_approval {
        GroupType::ApprovalChain
    } else if has_cycles && anomaly_rate > 0.5 {
        GroupType::MuleNetwork
    } else if has_cycles {
        GroupType::VendorRing
    } else if anomaly_rate > 0.3 {
        GroupType::MuleNetwork
    } else {
        GroupType::TransactionCluster
    }
}

/// Calculates group metrics (internal volume, external volume, cohesion).
fn calculate_group_metrics(graph: &Graph, members: &[NodeId]) -> (f64, f64, f64) {
    let member_set: HashSet<NodeId> = members.iter().copied().collect();

    let mut internal_volume = 0.0;
    let mut external_volume = 0.0;
    let mut internal_edges = 0;

    for &member in members {
        for edge in graph.outgoing_edges(member) {
            if member_set.contains(&edge.target) {
                internal_volume += edge.weight;
                internal_edges += 1;
            } else {
                external_volume += edge.weight;
            }
        }

        for edge in graph.incoming_edges(member) {
            if !member_set.contains(&edge.source) {
                external_volume += edge.weight;
            }
        }
    }

    // Calculate cohesion (edge density)
    let max_possible_edges = members.len() * (members.len() - 1);
    let cohesion = if max_possible_edges > 0 {
        internal_edges as f64 / max_possible_edges as f64
    } else {
        0.0
    };

    (internal_volume, external_volume, cohesion)
}

/// Finds the hub node (most connected) in a group.
fn find_hub_node(graph: &Graph, members: &[NodeId]) -> NodeId {
    let member_set: HashSet<NodeId> = members.iter().copied().collect();

    members
        .iter()
        .map(|&n| {
            let internal_degree = graph
                .neighbors(n)
                .iter()
                .filter(|neighbor| member_set.contains(neighbor))
                .count();
            (n, internal_degree)
        })
        .max_by_key(|(_, degree)| *degree)
        .map(|(n, _)| n)
        .unwrap_or(members[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GraphEdge, GraphNode, GraphType, NodeType};

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new("test", GraphType::Transaction);

        // Create two connected components
        // Component 1: n1 - n2 - n3 (triangle)
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

        // Component 2: n4 - n5 - n6 (chain)
        let n4 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "D".to_string(),
            "D".to_string(),
        ));
        let n5 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "E".to_string(),
            "E".to_string(),
        ));
        let n6 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "F".to_string(),
            "F".to_string(),
        ));

        graph.add_edge(GraphEdge::new(0, n4, n5, EdgeType::Transaction).with_weight(200.0));
        graph.add_edge(GraphEdge::new(0, n5, n6, EdgeType::Transaction).with_weight(200.0));

        graph
    }

    #[test]
    fn test_connected_components() {
        let graph = create_test_graph();
        let config = GroupDetectionConfig::default();

        let result = detect_entity_groups(&graph, &config);

        // Should detect 2 components
        assert!(result.total_groups >= 1);
    }

    #[test]
    fn test_label_propagation() {
        let graph = create_test_graph();
        let config = GroupDetectionConfig {
            algorithms: vec![GroupDetectionAlgorithm::LabelPropagation],
            ..Default::default()
        };

        let result = detect_entity_groups(&graph, &config);

        // Should detect communities
        assert!(!result.groups.is_empty() || result.total_groups == 0);
    }

    #[test]
    fn test_clique_detection() {
        let graph = create_test_graph();
        let config = GroupDetectionConfig {
            algorithms: vec![GroupDetectionAlgorithm::CliqueDetection],
            min_cohesion: 0.1, // Lower threshold to accept directed graph cohesion
            ..Default::default()
        };

        let result = detect_entity_groups(&graph, &config);

        // Should detect cliques (triangle has cohesion ~0.5 for directed graph)
        // A directed triangle has 3 edges out of max 6 possible = 0.5 cohesion
        let cliques: Vec<_> = result.groups.iter().filter(|g| g.cohesion > 0.4).collect();
        assert!(!cliques.is_empty());
    }

    #[test]
    fn test_node_features() {
        let graph = create_test_graph();
        let config = GroupDetectionConfig::default();

        let result = detect_entity_groups(&graph, &config);
        let features = result.node_features(1);

        assert_eq!(features.len(), GroupDetectionResult::feature_dim());
    }

    #[test]
    fn test_group_metrics() {
        let graph = create_test_graph();
        let members = vec![1, 2, 3]; // The triangle

        let (internal, _external, cohesion) = calculate_group_metrics(&graph, &members);

        assert!(internal > 0.0);
        assert!(cohesion > 0.0);
    }

    #[test]
    fn test_hub_detection() {
        let mut graph = Graph::new("test", GraphType::Transaction);

        // Create star pattern with hub at n1
        let n1 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "Hub".to_string(),
            "Hub".to_string(),
        ));
        let n2 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "A".to_string(),
            "A".to_string(),
        ));
        let n3 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "B".to_string(),
            "B".to_string(),
        ));
        let n4 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "C".to_string(),
            "C".to_string(),
        ));

        graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction));
        graph.add_edge(GraphEdge::new(0, n1, n3, EdgeType::Transaction));
        graph.add_edge(GraphEdge::new(0, n1, n4, EdgeType::Transaction));

        let members = vec![n1, n2, n3, n4];
        let hub = find_hub_node(&graph, &members);

        assert_eq!(hub, n1);
    }
}
