//! Shared test helper functions for the datasynth-graph crate.
//!
//! These helpers create various test graph configurations used across
//! exporter and ML module tests, avoiding duplication.

use chrono::NaiveDate;

use crate::models::{EdgeType, Graph, GraphEdge, GraphNode, GraphType, NodeType};

/// Creates a minimal test graph with 2 Account nodes and 1 Transaction edge.
///
/// Used by: pytorch_geometric, neo4j exporter tests.
///
/// Structure:
///   - Node 1: Account "1000" / "Cash" (feature: 0.5)
///   - Node 2: Account "2000" / "AP" (feature: 0.8)
///   - Edge: Transaction from n1 -> n2 (weight: 1000.0, feature: 6.9)
pub fn create_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

    let n1 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "1000".to_string(), "Cash".to_string())
            .with_feature(0.5),
    );
    let n2 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "2000".to_string(), "AP".to_string())
            .with_feature(0.8),
    );

    graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(1000.0)
            .with_feature(6.9),
    );

    graph.compute_statistics();
    graph
}

/// Creates a test graph with 3 nodes (2 Accounts + 1 Company) and 2 edges,
/// including an anomalous Company node.
///
/// Used by: DGL exporter tests.
///
/// Structure:
///   - Node 1: Account "1000" / "Cash" (feature: 0.5)
///   - Node 2: Account "2000" / "AP" (feature: 0.8)
///   - Node 3: Company "ACME" / "ACME Corp" (feature: 0.3, anomaly: "fraud")
///   - Edge 1: Transaction n1 -> n2 (weight: 1000.0, feature: 6.9)
///   - Edge 2: Ownership n2 -> n3 (weight: 100.0, feature: 4.6)
pub fn create_test_graph_with_company() -> Graph {
    let mut graph = Graph::new("test_dgl", GraphType::Transaction);

    let n1 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "1000".to_string(), "Cash".to_string())
            .with_feature(0.5),
    );
    let n2 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "2000".to_string(), "AP".to_string())
            .with_feature(0.8),
    );
    let n3 = graph.add_node(
        GraphNode::new(
            0,
            NodeType::Company,
            "ACME".to_string(),
            "ACME Corp".to_string(),
        )
        .with_feature(0.3)
        .as_anomaly("fraud"),
    );

    graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(1000.0)
            .with_feature(6.9),
    );

    graph.add_edge(
        GraphEdge::new(0, n2, n3, EdgeType::Ownership)
            .with_weight(100.0)
            .with_feature(4.6),
    );

    graph.compute_statistics();
    graph
}

/// Creates a test graph with 3 nodes (2 Accounts + 1 Vendor) and 2 edges,
/// including anomalous nodes/edges, categoricals, and timestamps.
///
/// Used by: RustGraph exporter tests.
///
/// Structure:
///   - Node 1: Account "1000" / "Cash" (features: [0.5, 0.3], categorical: account_type=Asset)
///   - Node 2: Account "2000" / "AP" (features: [0.8, 0.2], anomaly: "unusual_balance")
///   - Node 3: Vendor "V001" / "Acme Corp" (feature: 1.0)
///   - Edge 1: Transaction n1 -> n2 (weight: 1000.0, feature: 6.9, timestamp: 2024-01-15)
///   - Edge 2: Transaction n2 -> n3 (weight: 500.0, anomaly: "split_transaction")
pub fn create_test_graph_with_vendor() -> Graph {
    let mut graph = Graph::new("test_graph", GraphType::Transaction);

    let n1 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "1000".to_string(), "Cash".to_string())
            .with_feature(0.5)
            .with_feature(0.3)
            .with_categorical("account_type", "Asset"),
    );
    let n2 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "2000".to_string(), "AP".to_string())
            .with_feature(0.8)
            .with_feature(0.2)
            .as_anomaly("unusual_balance"),
    );
    let n3 = graph.add_node(
        GraphNode::new(
            0,
            NodeType::Vendor,
            "V001".to_string(),
            "Acme Corp".to_string(),
        )
        .with_feature(1.0),
    );

    graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(1000.0)
            .with_feature(6.9)
            .with_timestamp(chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
    );
    graph.add_edge(
        GraphEdge::new(0, n2, n3, EdgeType::Transaction)
            .with_weight(500.0)
            .as_anomaly("split_transaction"),
    );

    graph.compute_statistics();
    graph
}

/// Creates a test graph with 3 Account nodes and timestamped edges for
/// temporal feature analysis.
///
/// Used by: temporal ML tests.
///
/// Structure:
///   - 3 Account nodes: "1000"/"Cash", "2000"/"AP", "3000"/"Revenue"
///   - 10 Transaction edges from n1 -> n2 (days 1-10, increasing amounts)
///   - 5 Transaction edges from n1 -> n3 on even days (double amounts)
pub fn create_temporal_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

    // Add nodes
    let n1 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "1000".to_string(),
        "Cash".to_string(),
    ));
    let n2 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "2000".to_string(),
        "AP".to_string(),
    ));
    let n3 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "3000".to_string(),
        "Revenue".to_string(),
    ));

    // Add edges with timestamps spanning several days
    for i in 0..10 {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1 + i).unwrap();
        let amount = 100.0 + (i as f64 * 10.0); // Increasing trend

        let edge = GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(amount)
            .with_timestamp(date);
        graph.add_edge(edge);

        // Add some edges to n3 for variety
        if i % 2 == 0 {
            let edge = GraphEdge::new(0, n1, n3, EdgeType::Transaction)
                .with_weight(amount * 2.0)
                .with_timestamp(date);
            graph.add_edge(edge);
        }
    }

    graph
}

/// Creates a test graph with 6 Account nodes forming two connected components:
/// a triangle (n1-n2-n3) and a chain (n4-n5-n6).
///
/// Used by: entity_groups ML tests.
///
/// Structure:
///   - Component 1: Triangle n1->n2->n3->n1 (weight: 100.0 each)
///   - Component 2: Chain n4->n5->n6 (weight: 200.0 each)
pub fn create_entity_group_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

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

/// Creates a test graph with 3 Account nodes having feature vectors
/// and 3 edges forming a triangle.
///
/// Used by: aggregation ML tests.
///
/// Structure:
///   - Node 1: Account "A" (features: [1.0, 2.0, 3.0])
///   - Node 2: Account "B" (features: [4.0, 5.0, 6.0])
///   - Node 3: Account "C" (features: [7.0, 8.0, 9.0])
///   - Edges: n1->n2 (100.0), n2->n3 (200.0), n1->n3 (150.0)
pub fn create_aggregation_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

    let n1 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "A".to_string(), "A".to_string())
            .with_features(vec![1.0, 2.0, 3.0]),
    );
    let n2 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "B".to_string(), "B".to_string())
            .with_features(vec![4.0, 5.0, 6.0]),
    );
    let n3 = graph.add_node(
        GraphNode::new(0, NodeType::Account, "C".to_string(), "C".to_string())
            .with_features(vec![7.0, 8.0, 9.0]),
    );

    graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, n2, n3, EdgeType::Transaction).with_weight(200.0));
    graph.add_edge(GraphEdge::new(0, n1, n3, EdgeType::Transaction).with_weight(150.0));

    graph
}

/// Creates a test graph with 4 Account nodes and timestamped edges
/// for relationship feature computation.
///
/// Used by: relationship_features ML tests.
///
/// Structure:
///   - 4 Account nodes: A, B, C, D
///   - N1 -> N2: 2 transactions (2024-01-01 $1000, 2024-06-01 $2000)
///   - N1 -> N3: 1 transaction (2024-03-01 $500)
///   - N2 -> N1: 1 transaction (2024-04-01 $1500, bidirectional)
///   - N1 -> N4: 1 transaction (2024-12-15 $300, recent/new)
pub fn create_relationship_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

    // Create nodes
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
    let n4 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "D".to_string(),
        "D".to_string(),
    ));

    // Create edges with timestamps
    // N1 -> N2 (2 transactions)
    graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(1000.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    );
    graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(2000.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
    );

    // N1 -> N3 (1 transaction)
    graph.add_edge(
        GraphEdge::new(0, n1, n3, EdgeType::Transaction)
            .with_weight(500.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
    );

    // N2 -> N1 (bidirectional relationship)
    graph.add_edge(
        GraphEdge::new(0, n2, n1, EdgeType::Transaction)
            .with_weight(1500.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
    );

    // N1 -> N4 (recent new relationship)
    graph.add_edge(
        GraphEdge::new(0, n1, n4, EdgeType::Transaction)
            .with_weight(300.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
    );

    graph
}

/// Creates a test graph with 10 Account nodes in a chain, with timestamps
/// and one anomalous node for split testing.
///
/// Used by: splits ML tests.
///
/// Structure:
///   - 10 Account nodes (node 5 is anomalous)
///   - 9 edges in a chain: 1->2->3->...->10
///   - Timestamps: 2024-01-01 through 2024-01-09
pub fn create_splits_test_graph() -> Graph {
    let mut graph = Graph::new("test", GraphType::Transaction);

    for i in 0..10 {
        let mut node = GraphNode::new(
            0,
            NodeType::Account,
            format!("{}", i),
            format!("Account {}", i),
        );
        if i == 5 {
            node.is_anomaly = true;
        }
        graph.add_node(node);
    }

    for i in 0..9 {
        let edge = GraphEdge::new(0, i + 1, i + 2, EdgeType::Transaction)
            .with_timestamp(chrono::NaiveDate::from_ymd_opt(2024, 1, i as u32 + 1).unwrap());
        graph.add_edge(edge);
    }

    graph.compute_statistics();
    graph
}
