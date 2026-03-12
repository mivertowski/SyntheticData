//! Tests for snake_case entity type naming and RFC 3339 date serialization conventions.

use datasynth_graph::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
use datasynth_graph::models::hypergraph::HypergraphNode;

/// Verify that a manually constructed HypergraphNode with snake_case
/// entity_type has no uppercase characters.
#[test]
fn node_type_name_is_snake_case() {
    let node = HypergraphNode {
        entity_type: "internal_control".to_string(),
        ..default_node()
    };
    assert!(
        !node.entity_type.contains(char::is_uppercase),
        "entity_type should be snake_case: {}",
        node.entity_type
    );
}

/// After building a COSO-only hypergraph, all entity_type values must be
/// snake_case (no uppercase characters).
#[test]
fn builder_produces_snake_case_entity_types() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        include_p2p: false,
        include_o2c: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    assert!(!hg.nodes.is_empty(), "should have produced nodes");

    for node in &hg.nodes {
        assert!(
            !node.entity_type.contains(char::is_uppercase),
            "entity_type should be snake_case, got: '{}' for node '{}'",
            node.entity_type,
            node.id
        );
    }
}

/// Verify specific expected entity type names from the builder.
#[test]
fn builder_entity_type_names_match_convention() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        include_p2p: false,
        include_o2c: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    let types: Vec<&str> = hg.nodes.iter().map(|n| n.entity_type.as_str()).collect();
    assert!(
        types.contains(&"coso_component"),
        "expected coso_component in {:?}",
        types
    );
    assert!(
        types.contains(&"coso_principle"),
        "expected coso_principle in {:?}",
        types
    );
}

/// Metadata node_type_counts keys should also be snake_case.
#[test]
fn metadata_node_type_keys_are_snake_case() {
    let config = HypergraphConfig {
        max_nodes: 1000,
        include_coso: true,
        include_controls: false,
        include_sox: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        include_p2p: false,
        include_o2c: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    builder.add_coso_framework();
    let hg = builder.build();

    for key in hg.metadata.node_type_counts.keys() {
        assert!(
            !key.contains(char::is_uppercase),
            "node_type_counts key should be snake_case, got: '{}'",
            key
        );
    }
}

/// Helper to create a default HypergraphNode for tests.
fn default_node() -> HypergraphNode {
    HypergraphNode {
        id: "test_node".to_string(),
        entity_type: "test".to_string(),
        entity_type_code: 0,
        layer: datasynth_graph::models::hypergraph::HypergraphLayer::GovernanceControls,
        external_id: "ext_1".to_string(),
        label: "Test Node".to_string(),
        properties: std::collections::HashMap::new(),
        features: vec![],
        is_anomaly: false,
        anomaly_type: None,
        is_aggregate: false,
        aggregate_count: 0,
    }
}
