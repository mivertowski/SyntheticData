//! Smoke tests for the graph export pipeline.
//!
//! These tests verify the pipeline constructs correctly and the public API
//! is usable. Full integration tests (with real EnhancedGenerationResult data)
//! are in Task 16.

use datasynth_graph_export::{
    BudgetConfig, BudgetManager, EdgeSamplingStrategy, ExportConfig, ExportEdge, ExportNode,
    ExportWarnings, GraphExportPipeline, IdMap,
};
use std::collections::HashMap;

#[test]
fn empty_pipeline_builds_correctly() {
    let pipeline = GraphExportPipeline::new(ExportConfig::default());
    assert_eq!(pipeline.property_serializers().len(), 0);
}

#[test]
fn standard_pipeline_builds_correctly() {
    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    // Task 9: 30 property serializers across all domains.
    assert_eq!(pipeline.property_serializers().len(), 30);
}

#[test]
fn pipeline_config_access() {
    let mut pipeline = GraphExportPipeline::new(ExportConfig::default());
    assert_eq!(pipeline.config().budget.max_nodes, 50_000);
    pipeline.config_mut().budget.max_nodes = 10_000;
    assert_eq!(pipeline.config().budget.max_nodes, 10_000);
}

#[test]
fn budget_manager_no_op_within_limits() {
    let config = BudgetConfig {
        max_nodes: 100,
        max_edges: 1000,
        layer_split: [0.20, 0.60, 0.20],
    };
    let mgr = BudgetManager::new(&config);
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();

    let nodes = vec![
        ExportNode {
            id: Some(1),
            node_type: 100,
            node_type_name: "test".into(),
            label: "A".into(),
            layer: 1,
            properties: HashMap::new(),
        },
        ExportNode {
            id: Some(2),
            node_type: 200,
            node_type_name: "test".into(),
            label: "B".into(),
            layer: 2,
            properties: HashMap::new(),
        },
    ];

    id_map.get_or_insert("a");
    id_map.get_or_insert("b");

    let result = mgr.enforce_node_budget(nodes, &mut id_map, &mut warnings);
    assert_eq!(result.len(), 2);
    assert!(warnings.is_empty());
}

#[test]
fn budget_manager_edge_budget_with_dangling() {
    let config = BudgetConfig {
        max_nodes: 100,
        max_edges: 100,
        layer_split: [0.20, 0.60, 0.20],
    };
    let mgr = BudgetManager::new(&config);
    let mut id_map = IdMap::new();
    id_map.get_or_insert("a"); // => 1
    id_map.get_or_insert("b"); // => 2
    let mut warnings = ExportWarnings::new();

    let edges = vec![
        ExportEdge {
            source: 1,
            target: 2,
            edge_type: 40,
            weight: 1.0,
            properties: HashMap::new(),
        },
        ExportEdge {
            source: 1,
            target: 999, // dangling
            edge_type: 40,
            weight: 1.0,
            properties: HashMap::new(),
        },
    ];

    let result =
        mgr.enforce_edge_budget(edges, &id_map, EdgeSamplingStrategy::Truncate, &mut warnings);
    assert_eq!(result.len(), 1);
}

#[test]
fn helpers_camel_to_snake() {
    assert_eq!(
        datasynth_graph_export::helpers::camel_to_snake("processFamily"),
        "process_family"
    );
    assert_eq!(
        datasynth_graph_export::helpers::camel_to_snake("isAnomalous"),
        "is_anomalous"
    );
}

#[test]
fn helpers_entity_type_process_family() {
    use datasynth_graph_export::helpers::entity_type_process_family;

    assert_eq!(entity_type_process_family("purchase_order"), Some("P2P"));
    assert_eq!(entity_type_process_family("journal_entry"), Some("R2R"));
    assert_eq!(entity_type_process_family("internal_control"), None);
}

#[test]
fn id_map_round_trip() {
    let mut map = IdMap::new();
    let id1 = map.get_or_insert("VENDOR-001");
    let id2 = map.get_or_insert("VENDOR-002");
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(map.reverse_get(1), Some("VENDOR-001"));
    assert_eq!(map.get("VENDOR-001"), Some(1));
}
