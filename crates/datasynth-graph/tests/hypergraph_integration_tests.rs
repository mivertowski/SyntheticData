//! Integration tests for the multi-layer hypergraph builder and exporter.
//!
//! These tests build a complete hypergraph from test data and export it,
//! verifying that the JSONL output files are well-formed and internally consistent.

use std::collections::HashSet;

use datasynth_graph::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
use datasynth_graph::exporters::hypergraph::{HypergraphExportConfig, HypergraphExporter};
use datasynth_graph::models::hypergraph::{
    CrossLayerEdge, Hyperedge, HypergraphLayer, HypergraphMetadata, HypergraphNode,
};
use tempfile::tempdir;

/// Build a hypergraph with all 3 layers populated, export it,
/// then read back and validate every output file.
#[test]
fn test_full_hypergraph_roundtrip() {
    // -- Build --
    let config = HypergraphConfig {
        max_nodes: 5000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    // -- Export --
    let dir = tempdir().unwrap();
    let exporter = HypergraphExporter::new(HypergraphExportConfig { pretty_print: true });
    let metadata = exporter.export(&hg, dir.path()).unwrap();

    // -- Validate files exist --
    for filename in &[
        "nodes.jsonl",
        "edges.jsonl",
        "hyperedges.jsonl",
        "metadata.json",
    ] {
        assert!(
            dir.path().join(filename).exists(),
            "Missing output file: {}",
            filename
        );
    }

    // -- Validate nodes.jsonl --
    let nodes_content = std::fs::read_to_string(dir.path().join("nodes.jsonl")).unwrap();
    let nodes: Vec<HypergraphNode> = nodes_content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(nodes.len(), metadata.num_nodes);
    assert_eq!(nodes.len(), 22); // 5 COSO components + 17 principles

    // All nodes should have unique IDs
    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(node_ids.len(), nodes.len(), "Node IDs must be unique");

    // All nodes should be in Layer 1 (Governance)
    for node in &nodes {
        assert_eq!(node.layer, HypergraphLayer::GovernanceControls);
        assert!(!node.id.is_empty());
        assert!(!node.entity_type.is_empty());
        assert!(!node.label.is_empty());
    }

    // -- Validate edges.jsonl --
    let edges_content = std::fs::read_to_string(dir.path().join("edges.jsonl")).unwrap();
    let edges: Vec<CrossLayerEdge> = edges_content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(edges.len(), metadata.num_edges);

    // All edges should reference valid node IDs
    for edge in &edges {
        assert!(
            node_ids.contains(edge.source_id.as_str()),
            "Edge source '{}' not found in nodes",
            edge.source_id
        );
        assert!(
            node_ids.contains(edge.target_id.as_str()),
            "Edge target '{}' not found in nodes",
            edge.target_id
        );
    }

    // -- Validate hyperedges.jsonl --
    let he_content = std::fs::read_to_string(dir.path().join("hyperedges.jsonl")).unwrap();
    let hyperedges: Vec<Hyperedge> = he_content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(hyperedges.len(), metadata.num_hyperedges);

    // -- Validate metadata.json --
    let meta_content = std::fs::read_to_string(dir.path().join("metadata.json")).unwrap();
    let parsed_meta: HypergraphMetadata = serde_json::from_str(&meta_content).unwrap();

    assert_eq!(parsed_meta.num_nodes, nodes.len());
    assert_eq!(parsed_meta.num_edges, edges.len());
    assert_eq!(parsed_meta.num_hyperedges, hyperedges.len());
    assert_eq!(parsed_meta.source, "datasynth");
    assert!(!parsed_meta.generated_at.is_empty());
    assert_eq!(parsed_meta.files.len(), 4);

    // Layer counts should be consistent
    let l1_count = parsed_meta
        .layer_node_counts
        .get("Governance & Controls")
        .copied()
        .unwrap_or(0);
    assert_eq!(l1_count, 22);
}

/// Verify that the node budget is respected when building large datasets.
#[test]
fn test_budget_enforcement() {
    let config = HypergraphConfig {
        max_nodes: 100, // very small budget
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    // Total nodes should not exceed the configured max
    assert!(
        hg.nodes.len() <= 100,
        "Node count {} exceeds budget of 100",
        hg.nodes.len()
    );

    // Budget report should reflect actual usage
    assert_eq!(hg.budget_report.total_used, hg.nodes.len());
}

/// Verify metadata layer_node_counts sums to total nodes.
#[test]
fn test_metadata_consistency() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    // Sum of per-layer counts should equal total
    let sum: usize = hg.metadata.layer_node_counts.values().sum();
    assert_eq!(sum, hg.metadata.num_nodes);

    // Sum of per-type counts should equal total
    let type_sum: usize = hg.metadata.node_type_counts.values().sum();
    assert_eq!(type_sum, hg.metadata.num_nodes);
}

/// Verify that edges reference nodes that actually exist.
#[test]
fn test_edge_node_referential_integrity() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    let node_ids: HashSet<&str> = hg.nodes.iter().map(|n| n.id.as_str()).collect();

    for edge in &hg.edges {
        assert!(
            node_ids.contains(edge.source_id.as_str()),
            "Edge references non-existent source node: {}",
            edge.source_id
        );
        assert!(
            node_ids.contains(edge.target_id.as_str()),
            "Edge references non-existent target node: {}",
            edge.target_id
        );
    }
}

/// Verify that an empty hypergraph (no layers enabled) produces valid but empty output.
#[test]
fn test_empty_hypergraph() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: false,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let builder = HypergraphBuilder::new(config);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 0);
    assert_eq!(hg.edges.len(), 0);
    assert_eq!(hg.hyperedges.len(), 0);
    assert_eq!(hg.metadata.num_nodes, 0);

    // Export should still succeed
    let dir = tempdir().unwrap();
    let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
    let metadata = exporter.export(&hg, dir.path()).unwrap();
    assert_eq!(metadata.num_nodes, 0);

    // Files should exist but have no content lines
    let nodes_content = std::fs::read_to_string(dir.path().join("nodes.jsonl")).unwrap();
    assert!(nodes_content.is_empty());
}

/// Verify that COSO component-principle edges have correct layer assignments.
#[test]
fn test_coso_edge_layers() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_p2p: false,
        include_o2c: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    // All COSO edges should be intra-layer (GovernanceControls -> GovernanceControls)
    for edge in &hg.edges {
        assert_eq!(
            edge.source_layer,
            HypergraphLayer::GovernanceControls,
            "COSO edge source should be GovernanceControls"
        );
        assert_eq!(
            edge.target_layer,
            HypergraphLayer::GovernanceControls,
            "COSO edge target should be GovernanceControls"
        );
    }

    // Should have exactly 17 edges (one per principle -> component)
    assert_eq!(hg.edges.len(), 17);
}
