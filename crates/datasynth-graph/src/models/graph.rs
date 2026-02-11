//! Graph container model.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::edges::{EdgeId, EdgeType, GraphEdge};
use super::nodes::{GraphNode, NodeId, NodeType};

/// A graph containing nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Graph name.
    pub name: String,
    /// Graph type.
    pub graph_type: GraphType,
    /// Nodes indexed by ID.
    pub nodes: HashMap<NodeId, GraphNode>,
    /// Edges indexed by ID.
    pub edges: HashMap<EdgeId, GraphEdge>,
    /// Adjacency list (node -> outgoing edges).
    pub adjacency: HashMap<NodeId, Vec<EdgeId>>,
    /// Reverse adjacency (node -> incoming edges).
    pub reverse_adjacency: HashMap<NodeId, Vec<EdgeId>>,
    /// Node type index.
    pub nodes_by_type: HashMap<NodeType, Vec<NodeId>>,
    /// Edge type index.
    pub edges_by_type: HashMap<EdgeType, Vec<EdgeId>>,
    /// Metadata.
    pub metadata: GraphMetadata,
    /// Next node ID.
    next_node_id: NodeId,
    /// Next edge ID.
    next_edge_id: EdgeId,
}

impl Graph {
    /// Creates a new graph.
    pub fn new(name: &str, graph_type: GraphType) -> Self {
        Self {
            name: name.to_string(),
            graph_type,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            nodes_by_type: HashMap::new(),
            edges_by_type: HashMap::new(),
            metadata: GraphMetadata::default(),
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    /// Adds a node to the graph, returning its ID.
    pub fn add_node(&mut self, mut node: GraphNode) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        node.id = id;

        // Update type index
        self.nodes_by_type
            .entry(node.node_type.clone())
            .or_default()
            .push(id);

        // Initialize adjacency
        self.adjacency.insert(id, Vec::new());
        self.reverse_adjacency.insert(id, Vec::new());

        self.nodes.insert(id, node);
        id
    }

    /// Adds an edge to the graph, returning its ID.
    pub fn add_edge(&mut self, mut edge: GraphEdge) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id += 1;
        edge.id = id;

        // Update adjacency
        self.adjacency.entry(edge.source).or_default().push(id);
        self.reverse_adjacency
            .entry(edge.target)
            .or_default()
            .push(id);

        // Update type index
        self.edges_by_type
            .entry(edge.edge_type.clone())
            .or_default()
            .push(id);

        self.edges.insert(id, edge);
        id
    }

    /// Gets a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&GraphNode> {
        self.nodes.get(&id)
    }

    /// Gets a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut GraphNode> {
        self.nodes.get_mut(&id)
    }

    /// Gets an edge by ID.
    pub fn get_edge(&self, id: EdgeId) -> Option<&GraphEdge> {
        self.edges.get(&id)
    }

    /// Gets a mutable edge by ID.
    pub fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut GraphEdge> {
        self.edges.get_mut(&id)
    }

    /// Returns all nodes of a given type.
    pub fn nodes_of_type(&self, node_type: &NodeType) -> Vec<&GraphNode> {
        self.nodes_by_type
            .get(node_type)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns all edges of a given type.
    pub fn edges_of_type(&self, edge_type: &EdgeType) -> Vec<&GraphEdge> {
        self.edges_by_type
            .get(edge_type)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns outgoing edges from a node.
    pub fn outgoing_edges(&self, node_id: NodeId) -> Vec<&GraphEdge> {
        self.adjacency
            .get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns incoming edges to a node.
    pub fn incoming_edges(&self, node_id: NodeId) -> Vec<&GraphEdge> {
        self.reverse_adjacency
            .get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Returns neighbors of a node.
    pub fn neighbors(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut neighbors = HashSet::new();

        // Outgoing
        if let Some(edges) = self.adjacency.get(&node_id) {
            for edge_id in edges {
                if let Some(edge) = self.edges.get(edge_id) {
                    neighbors.insert(edge.target);
                }
            }
        }

        // Incoming
        if let Some(edges) = self.reverse_adjacency.get(&node_id) {
            for edge_id in edges {
                if let Some(edge) = self.edges.get(edge_id) {
                    neighbors.insert(edge.source);
                }
            }
        }

        neighbors.into_iter().collect()
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the out-degree of a node.
    pub fn out_degree(&self, node_id: NodeId) -> usize {
        self.adjacency.get(&node_id).map(|e| e.len()).unwrap_or(0)
    }

    /// Returns the in-degree of a node.
    pub fn in_degree(&self, node_id: NodeId) -> usize {
        self.reverse_adjacency
            .get(&node_id)
            .map(|e| e.len())
            .unwrap_or(0)
    }

    /// Returns the total degree of a node.
    pub fn degree(&self, node_id: NodeId) -> usize {
        self.out_degree(node_id) + self.in_degree(node_id)
    }

    /// Returns anomalous nodes.
    pub fn anomalous_nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().filter(|n| n.is_anomaly).collect()
    }

    /// Returns anomalous edges.
    pub fn anomalous_edges(&self) -> Vec<&GraphEdge> {
        self.edges.values().filter(|e| e.is_anomaly).collect()
    }

    /// Computes graph statistics.
    pub fn compute_statistics(&mut self) {
        self.metadata.node_count = self.nodes.len();
        self.metadata.edge_count = self.edges.len();

        // Count by type
        self.metadata.node_type_counts = self
            .nodes_by_type
            .iter()
            .map(|(t, ids)| (t.as_str().to_string(), ids.len()))
            .collect();

        self.metadata.edge_type_counts = self
            .edges_by_type
            .iter()
            .map(|(t, ids)| (t.as_str().to_string(), ids.len()))
            .collect();

        // Anomaly counts
        self.metadata.anomalous_node_count = self.anomalous_nodes().len();
        self.metadata.anomalous_edge_count = self.anomalous_edges().len();

        // Density
        if self.metadata.node_count > 1 {
            let max_edges = self.metadata.node_count * (self.metadata.node_count - 1);
            self.metadata.density = self.metadata.edge_count as f64 / max_edges as f64;
        }

        // Feature dimensions
        if let Some(node) = self.nodes.values().next() {
            self.metadata.node_feature_dim = node.features.len();
        }
        if let Some(edge) = self.edges.values().next() {
            self.metadata.edge_feature_dim = edge.features.len();
        }
    }

    /// Returns the edge index as a pair of vectors (source_ids, target_ids).
    pub fn edge_index(&self) -> (Vec<NodeId>, Vec<NodeId>) {
        let mut sources = Vec::with_capacity(self.edges.len());
        let mut targets = Vec::with_capacity(self.edges.len());

        for edge in self.edges.values() {
            sources.push(edge.source);
            targets.push(edge.target);
        }

        (sources, targets)
    }

    /// Returns the node feature matrix (nodes x features).
    pub fn node_features(&self) -> Vec<Vec<f64>> {
        let mut node_ids: Vec<_> = self.nodes.keys().copied().collect();
        node_ids.sort();

        node_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|n| n.features.clone())
            .collect()
    }

    /// Returns the edge feature matrix (edges x features).
    pub fn edge_features(&self) -> Vec<Vec<f64>> {
        let mut edge_ids: Vec<_> = self.edges.keys().copied().collect();
        edge_ids.sort();

        edge_ids
            .iter()
            .filter_map(|id| self.edges.get(id))
            .map(|e| e.features.clone())
            .collect()
    }

    /// Returns node labels.
    pub fn node_labels(&self) -> Vec<Vec<String>> {
        let mut node_ids: Vec<_> = self.nodes.keys().copied().collect();
        node_ids.sort();

        node_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|n| n.labels.clone())
            .collect()
    }

    /// Returns edge labels.
    pub fn edge_labels(&self) -> Vec<Vec<String>> {
        let mut edge_ids: Vec<_> = self.edges.keys().copied().collect();
        edge_ids.sort();

        edge_ids
            .iter()
            .filter_map(|id| self.edges.get(id))
            .map(|e| e.labels.clone())
            .collect()
    }

    /// Returns node anomaly flags.
    pub fn node_anomaly_mask(&self) -> Vec<bool> {
        let mut node_ids: Vec<_> = self.nodes.keys().copied().collect();
        node_ids.sort();

        node_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|n| n.is_anomaly)
            .collect()
    }

    /// Returns edge anomaly flags.
    pub fn edge_anomaly_mask(&self) -> Vec<bool> {
        let mut edge_ids: Vec<_> = self.edges.keys().copied().collect();
        edge_ids.sort();

        edge_ids
            .iter()
            .filter_map(|id| self.edges.get(id))
            .map(|e| e.is_anomaly)
            .collect()
    }
}

/// Type of graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphType {
    /// Transaction network (accounts as nodes, transactions as edges).
    Transaction,
    /// Approval network (users as nodes, approvals as edges).
    Approval,
    /// Entity relationship (companies as nodes, ownership as edges).
    EntityRelationship,
    /// Heterogeneous graph (multiple node and edge types).
    Heterogeneous,
    /// Custom graph type.
    Custom(String),
}

/// Metadata about a graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Number of nodes.
    pub node_count: usize,
    /// Number of edges.
    pub edge_count: usize,
    /// Node counts by type.
    pub node_type_counts: HashMap<String, usize>,
    /// Edge counts by type.
    pub edge_type_counts: HashMap<String, usize>,
    /// Number of anomalous nodes.
    pub anomalous_node_count: usize,
    /// Number of anomalous edges.
    pub anomalous_edge_count: usize,
    /// Graph density.
    pub density: f64,
    /// Node feature dimension.
    pub node_feature_dim: usize,
    /// Edge feature dimension.
    pub edge_feature_dim: usize,
    /// Additional properties.
    pub properties: HashMap<String, String>,
}

/// A heterogeneous graph with multiple node and edge types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeterogeneousGraph {
    /// Graph name.
    pub name: String,
    /// Subgraphs by relation type (source_type, edge_type, target_type).
    pub relations: HashMap<(String, String, String), Graph>,
    /// All node IDs by type.
    pub all_nodes: HashMap<String, Vec<NodeId>>,
    /// Metadata.
    pub metadata: GraphMetadata,
}

impl HeterogeneousGraph {
    /// Creates a new heterogeneous graph.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            relations: HashMap::new(),
            all_nodes: HashMap::new(),
            metadata: GraphMetadata::default(),
        }
    }

    /// Adds a relation (edge type between node types).
    pub fn add_relation(
        &mut self,
        source_type: &str,
        edge_type: &str,
        target_type: &str,
        graph: Graph,
    ) {
        let key = (
            source_type.to_string(),
            edge_type.to_string(),
            target_type.to_string(),
        );
        self.relations.insert(key, graph);
    }

    /// Gets a relation graph.
    pub fn get_relation(
        &self,
        source_type: &str,
        edge_type: &str,
        target_type: &str,
    ) -> Option<&Graph> {
        let key = (
            source_type.to_string(),
            edge_type.to_string(),
            target_type.to_string(),
        );
        self.relations.get(&key)
    }

    /// Returns all relation keys.
    pub fn relation_types(&self) -> Vec<(String, String, String)> {
        self.relations.keys().cloned().collect()
    }

    /// Computes statistics for the heterogeneous graph.
    pub fn compute_statistics(&mut self) {
        let mut total_nodes = 0;
        let mut total_edges = 0;

        for graph in self.relations.values() {
            total_nodes += graph.node_count();
            total_edges += graph.edge_count();
        }

        self.metadata.node_count = total_nodes;
        self.metadata.edge_count = total_edges;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = Graph::new("test", GraphType::Transaction);

        let node1 = GraphNode::new(0, NodeType::Account, "1000".to_string(), "Cash".to_string());
        let node2 = GraphNode::new(0, NodeType::Account, "2000".to_string(), "AP".to_string());

        let id1 = graph.add_node(node1);
        let id2 = graph.add_node(node2);

        let edge = GraphEdge::new(0, id1, id2, EdgeType::Transaction);
        graph.add_edge(edge);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_adjacency() {
        let mut graph = Graph::new("test", GraphType::Transaction);

        let n1 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "1".to_string(),
            "A".to_string(),
        ));
        let n2 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "2".to_string(),
            "B".to_string(),
        ));
        let n3 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "3".to_string(),
            "C".to_string(),
        ));

        graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction));
        graph.add_edge(GraphEdge::new(0, n1, n3, EdgeType::Transaction));
        graph.add_edge(GraphEdge::new(0, n2, n3, EdgeType::Transaction));

        assert_eq!(graph.out_degree(n1), 2);
        assert_eq!(graph.in_degree(n3), 2);
        assert_eq!(graph.neighbors(n1).len(), 2);
    }

    #[test]
    fn test_edge_index() {
        let mut graph = Graph::new("test", GraphType::Transaction);

        let n1 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "1".to_string(),
            "A".to_string(),
        ));
        let n2 = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            "2".to_string(),
            "B".to_string(),
        ));

        graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction));

        let (sources, targets) = graph.edge_index();
        assert_eq!(sources.len(), 1);
        assert_eq!(targets.len(), 1);
        assert_eq!(sources[0], n1);
        assert_eq!(targets[0], n2);
    }
}
