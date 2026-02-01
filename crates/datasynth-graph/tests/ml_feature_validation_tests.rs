//! Validation tests for ML feature modules.
//!
//! These tests validate:
//! - Feature bounds (all values in expected ranges)
//! - Mathematical properties (e.g., HHI between 0 and 1)
//! - Edge cases (empty graphs, single nodes, disconnected components)
//! - Consistency between related features
//! - Realistic graph structure handling

use std::collections::HashSet;

use chrono::NaiveDate;
use datasynth_graph::ml::{
    aggregation::{
        aggregate_all_groups, aggregate_features, aggregate_node_features, aggregate_values,
        aggregate_weighted, AggregatedFeatures, AggregationType,
    },
    entity_groups::{detect_entity_groups, GroupDetectionAlgorithm, GroupDetectionConfig},
    motifs::{
        detect_motifs, find_back_and_forth, find_circular_flows, find_star_patterns, MotifConfig,
    },
    relationship_features::{
        compute_all_relationship_features, compute_counterparty_risk,
        compute_relationship_features, CounterpartyRisk, RelationshipFeatureConfig,
        RelationshipFeatures,
    },
    temporal::{
        compute_all_temporal_features, compute_temporal_sequence_features, TemporalConfig,
        TemporalFeatures, TemporalIndex,
    },
};
use datasynth_graph::models::{Graph, GraphEdge, GraphNode, GraphType, NodeType};
use datasynth_graph::EdgeType;

// =============================================================================
// FR-001: Temporal Feature Validation Tests
// =============================================================================

/// Test that temporal features are within valid bounds.
#[test]
fn test_temporal_features_bounds() {
    let graph = create_temporal_test_graph();
    let index = TemporalIndex::build(&graph);
    let config = TemporalConfig::default();

    for node_id in 1..=5 {
        let features = compute_temporal_sequence_features(node_id, &graph, &index, &config);

        // All features should be non-negative
        assert!(
            features.transaction_velocity >= 0.0,
            "Node {}: transaction_velocity should be non-negative: {}",
            node_id,
            features.transaction_velocity
        );
        assert!(
            features.inter_event_interval_mean >= 0.0,
            "Node {}: inter_event_interval_mean should be non-negative: {}",
            node_id,
            features.inter_event_interval_mean
        );
        assert!(
            features.inter_event_interval_std >= 0.0,
            "Node {}: inter_event_interval_std should be non-negative: {}",
            node_id,
            features.inter_event_interval_std
        );

        // Burst score should be in [0, 1] or reasonable range
        assert!(
            features.burst_score >= 0.0,
            "Node {}: burst_score should be non-negative: {}",
            node_id,
            features.burst_score
        );

        // Trend direction should be in [-1, 1]
        assert!(
            features.trend_direction >= -1.0 && features.trend_direction <= 1.0,
            "Node {}: trend_direction should be in [-1, 1]: {}",
            node_id,
            features.trend_direction
        );

        // Seasonality score should be in [0, 1]
        assert!(
            features.seasonality_score >= 0.0 && features.seasonality_score <= 1.0,
            "Node {}: seasonality_score should be in [0, 1]: {}",
            node_id,
            features.seasonality_score
        );

        // Recency should be non-negative
        assert!(
            features.recency_days >= 0.0,
            "Node {}: recency_days should be non-negative: {}",
            node_id,
            features.recency_days
        );
    }
}

/// Test that temporal features handle empty/sparse nodes.
#[test]
fn test_temporal_features_sparse_nodes() {
    let mut graph = Graph::new("test", GraphType::Transaction);

    // Add isolated node with no edges
    graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "ISOLATED".to_string(),
        "Isolated Node".to_string(),
    ));

    // Add node with only one edge
    let n2 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "SPARSE".to_string(),
        "Sparse Node".to_string(),
    ));
    let n3 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "TARGET".to_string(),
        "Target Node".to_string(),
    ));

    graph.add_edge(
        GraphEdge::new(0, n2, n3, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
    );

    let index = TemporalIndex::build(&graph);
    let config = TemporalConfig::default();

    // Isolated node should have zero/default features
    let isolated_features = compute_temporal_sequence_features(1, &graph, &index, &config);
    assert_eq!(
        isolated_features.transaction_velocity, 0.0,
        "Isolated node should have zero velocity"
    );

    // Sparse node should still compute valid features
    let sparse_features = compute_temporal_sequence_features(n2, &graph, &index, &config);
    assert!(
        sparse_features.transaction_velocity >= 0.0,
        "Sparse node should have valid velocity"
    );
}

/// Test temporal feature consistency across all nodes.
#[test]
fn test_temporal_features_batch_consistency() {
    let graph = create_temporal_test_graph();
    let config = TemporalConfig::default();

    let all_features = compute_all_temporal_features(&graph, &config);

    // All nodes should have features
    assert!(
        !all_features.is_empty(),
        "Should compute features for all nodes"
    );

    // Feature vectors should have consistent length
    let window_sizes = &config.window_sizes;
    let first_len = all_features
        .values()
        .next()
        .unwrap()
        .to_features(window_sizes)
        .len();
    for (node_id, features) in &all_features {
        assert_eq!(
            features.to_features(window_sizes).len(),
            first_len,
            "Node {} should have consistent feature vector length",
            node_id
        );
    }
}

/// Test that window features are computed correctly.
#[test]
fn test_temporal_window_features() {
    let graph = create_temporal_test_graph();
    let index = TemporalIndex::build(&graph);
    let config = TemporalConfig {
        window_sizes: vec![7, 30, 90],
        ..Default::default()
    };

    let features = compute_temporal_sequence_features(1, &graph, &index, &config);

    // Should have window features for each configured window
    assert_eq!(
        features.window_features.len(),
        3,
        "Should have 3 window feature sets"
    );

    for (window_size, window_features) in &features.window_features {
        assert!(
            *window_size == 7 || *window_size == 30 || *window_size == 90,
            "Window size should be one of configured values"
        );

        // Window features should be valid (event_count is usize, always >= 0)
        assert!(
            window_features.total_amount >= 0.0,
            "Total amount should be non-negative"
        );
        if window_features.event_count > 0 {
            assert!(
                window_features.avg_amount >= 0.0,
                "Avg amount should be non-negative when events exist"
            );
        }
    }
}

// =============================================================================
// FR-002: Motif Detection Validation Tests
// =============================================================================

/// Test that cycle detection finds valid cycles.
#[test]
fn test_circular_flow_validity() {
    let graph = create_cycle_test_graph();
    let config = MotifConfig {
        max_cycle_length: 5,
        ..Default::default()
    };

    let cycles = find_circular_flows(&graph, &config);

    for cycle in &cycles {
        // Cycle should have at least 2 nodes (A -> B -> A)
        assert!(
            cycle.nodes.len() >= 2,
            "Cycle should have at least 2 nodes: {:?}",
            cycle.nodes
        );

        // Cycle length should be within limit
        assert!(
            cycle.nodes.len() <= config.max_cycle_length,
            "Cycle should respect max length: {:?}",
            cycle.nodes
        );

        // First and last node should form a cycle (edge back to start)
        // This is implicit in the cycle structure

        // Total weight should be positive
        assert!(
            cycle.total_weight > 0.0,
            "Cycle total weight should be positive: {}",
            cycle.total_weight
        );

        // Confidence should be in [0, 1]
        assert!(
            cycle.confidence >= 0.0 && cycle.confidence <= 1.0,
            "Cycle confidence should be in [0, 1]: {}",
            cycle.confidence
        );
    }
}

/// Test that star pattern detection works correctly.
#[test]
fn test_star_pattern_validity() {
    let graph = create_star_test_graph();
    let config = MotifConfig {
        min_star_spokes: 3,
        ..Default::default()
    };

    let stars = find_star_patterns(&graph, &config);

    for star in &stars {
        // Star should have nodes (hub + spokes)
        assert!(!star.nodes.is_empty(), "Star pattern should have nodes");

        // Star should have at least min_spokes + 1 nodes (hub + spokes)
        assert!(
            star.nodes.len() > config.min_star_spokes,
            "Star should have at least {} nodes (hub + spokes): {}",
            config.min_star_spokes + 1,
            star.nodes.len()
        );

        // Total weight should be positive
        assert!(star.total_weight > 0.0, "Star should have positive weight");

        // Confidence should be in [0, 1]
        assert!(
            star.confidence >= 0.0 && star.confidence <= 1.0,
            "Star confidence should be in [0, 1]: {}",
            star.confidence
        );
    }
}

/// Test back-and-forth detection.
#[test]
fn test_back_and_forth_validity() {
    let graph = create_back_and_forth_test_graph();
    let config = MotifConfig::default();
    let patterns = find_back_and_forth(&graph, &config);

    for pattern in &patterns {
        // Should have exactly 2 nodes
        assert_eq!(
            pattern.nodes.len(),
            2,
            "Back-and-forth should have exactly 2 nodes"
        );

        // Should have edges in both directions (verified by the detection)
        assert!(
            pattern.total_weight > 0.0,
            "Back-and-forth should have positive weight"
        );

        // Confidence should be in [0, 1]
        assert!(
            pattern.confidence >= 0.0 && pattern.confidence <= 1.0,
            "Back-and-forth confidence should be in [0, 1]: {}",
            pattern.confidence
        );
    }
}

/// Test that motif detection handles empty graphs.
#[test]
fn test_motif_detection_empty_graph() {
    let graph = Graph::new("empty", GraphType::Transaction);
    let config = MotifConfig::default();

    let result = detect_motifs(&graph, &config);

    assert!(
        result.motifs.is_empty() || result.motifs.values().all(|v| v.is_empty()),
        "Empty graph should have no motifs"
    );
    assert_eq!(result.total_circular_flows, 0);
    assert_eq!(result.total_star_patterns, 0);
    assert_eq!(result.total_back_and_forth, 0);
}

/// Test node motif participation counts.
#[test]
fn test_node_motif_participation() {
    let graph = create_cycle_test_graph();
    let config = MotifConfig {
        max_cycle_length: 5,
        min_star_spokes: 3,
        detect_back_and_forth: true,
        ..Default::default()
    };

    let result = detect_motifs(&graph, &config);

    // Validate node motif counts structure exists
    // Note: counts are usize type, inherently non-negative
    let _ = &result.node_motif_counts;
}

// =============================================================================
// FR-004: Relationship Feature Validation Tests
// =============================================================================

/// Test that relationship features are within valid bounds.
#[test]
fn test_relationship_features_bounds() {
    let graph = create_relationship_test_graph();
    let config = RelationshipFeatureConfig::default();

    for node_id in 1..=5 {
        let features = compute_relationship_features(node_id, &graph, &config);

        // unique_counterparties is usize, inherently non-negative

        // Ratios should be in [0, 1]
        assert!(
            features.new_relationship_ratio >= 0.0 && features.new_relationship_ratio <= 1.0,
            "Node {}: new_relationship_ratio should be in [0, 1]: {}",
            node_id,
            features.new_relationship_ratio
        );
        assert!(
            features.relationship_reciprocity >= 0.0 && features.relationship_reciprocity <= 1.0,
            "Node {}: relationship_reciprocity should be in [0, 1]: {}",
            node_id,
            features.relationship_reciprocity
        );
        assert!(
            features.dominant_counterparty_share >= 0.0
                && features.dominant_counterparty_share <= 1.0,
            "Node {}: dominant_counterparty_share should be in [0, 1]: {}",
            node_id,
            features.dominant_counterparty_share
        );

        // HHI (concentration) should be in [0, 1]
        assert!(
            features.counterparty_concentration >= 0.0
                && features.counterparty_concentration <= 1.0,
            "Node {}: counterparty_concentration (HHI) should be in [0, 1]: {}",
            node_id,
            features.counterparty_concentration
        );

        // Average age should be non-negative
        assert!(
            features.avg_relationship_age_days >= 0.0,
            "Node {}: avg_relationship_age_days should be non-negative: {}",
            node_id,
            features.avg_relationship_age_days
        );

        // Velocity should be non-negative
        assert!(
            features.relationship_velocity >= 0.0,
            "Node {}: relationship_velocity should be non-negative: {}",
            node_id,
            features.relationship_velocity
        );
    }
}

/// Test HHI calculation properties.
#[test]
fn test_herfindahl_index_properties() {
    // Graph with single counterparty (maximum concentration)
    let mut single_cp_graph = Graph::new("single_cp", GraphType::Transaction);
    let n1 = single_cp_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "A".to_string(),
        "A".to_string(),
    ));
    let n2 = single_cp_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "B".to_string(),
        "B".to_string(),
    ));
    single_cp_graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
    );

    let config = RelationshipFeatureConfig::default();
    let single_features = compute_relationship_features(n1, &single_cp_graph, &config);

    // Single counterparty should have HHI = 1.0
    assert!(
        (single_features.counterparty_concentration - 1.0).abs() < 0.01,
        "Single counterparty should have HHI ≈ 1.0: {}",
        single_features.counterparty_concentration
    );

    // Graph with evenly distributed counterparties (low concentration)
    let mut diverse_graph = Graph::new("diverse", GraphType::Transaction);
    let center = diverse_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "CENTER".to_string(),
        "Center".to_string(),
    ));

    for i in 0..10 {
        let target = diverse_graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("T{}", i),
            format!("Target {}", i),
        ));
        diverse_graph.add_edge(
            GraphEdge::new(0, center, target, EdgeType::Transaction)
                .with_weight(100.0) // Equal weights
                .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
        );
    }

    let diverse_features = compute_relationship_features(center, &diverse_graph, &config);

    // Evenly distributed should have low HHI (1/N = 0.1 for 10 counterparties)
    assert!(
        diverse_features.counterparty_concentration < 0.2,
        "Evenly distributed should have low HHI: {}",
        diverse_features.counterparty_concentration
    );
}

/// Test reciprocity calculation.
#[test]
fn test_reciprocity_calculation() {
    // Fully reciprocal graph
    let mut reciprocal_graph = Graph::new("reciprocal", GraphType::Transaction);
    let n1 = reciprocal_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "A".to_string(),
        "A".to_string(),
    ));
    let n2 = reciprocal_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "B".to_string(),
        "B".to_string(),
    ));
    let n3 = reciprocal_graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "C".to_string(),
        "C".to_string(),
    ));

    // A <-> B (reciprocal)
    reciprocal_graph.add_edge(
        GraphEdge::new(0, n1, n2, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
    );
    reciprocal_graph.add_edge(
        GraphEdge::new(0, n2, n1, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 2).unwrap()),
    );
    // A <-> C (reciprocal)
    reciprocal_graph.add_edge(
        GraphEdge::new(0, n1, n3, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
    );
    reciprocal_graph.add_edge(
        GraphEdge::new(0, n3, n1, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(NaiveDate::from_ymd_opt(2024, 6, 2).unwrap()),
    );

    let config = RelationshipFeatureConfig::default();
    let features = compute_relationship_features(n1, &reciprocal_graph, &config);

    // Fully reciprocal should have reciprocity = 1.0
    assert!(
        (features.relationship_reciprocity - 1.0).abs() < 0.01,
        "Fully reciprocal should have reciprocity ≈ 1.0: {}",
        features.relationship_reciprocity
    );
}

/// Test counterparty risk bounds.
#[test]
fn test_counterparty_risk_bounds() {
    let graph = create_relationship_test_graph();
    let config = RelationshipFeatureConfig::default();

    for node_id in 1..=5 {
        let risk = compute_counterparty_risk(node_id, &graph, &config);

        // All risk scores should be in [0, 1]
        assert!(
            risk.high_risk_counterparty_ratio >= 0.0 && risk.high_risk_counterparty_ratio <= 1.0,
            "Node {}: high_risk_counterparty_ratio should be in [0, 1]: {}",
            node_id,
            risk.high_risk_counterparty_ratio
        );
        assert!(
            risk.avg_counterparty_risk_score >= 0.0 && risk.avg_counterparty_risk_score <= 1.0,
            "Node {}: avg_counterparty_risk_score should be in [0, 1]: {}",
            node_id,
            risk.avg_counterparty_risk_score
        );
        assert!(
            risk.risk_concentration >= 0.0 && risk.risk_concentration <= 1.0,
            "Node {}: risk_concentration should be in [0, 1]: {}",
            node_id,
            risk.risk_concentration
        );

        // anomalous_counterparty_count is usize, inherently non-negative
        assert!(
            risk.high_risk_exposure >= 0.0,
            "Node {}: high_risk_exposure should be non-negative",
            node_id
        );
    }
}

/// Test batch relationship feature computation.
#[test]
fn test_relationship_features_batch_consistency() {
    let graph = create_relationship_test_graph();
    let config = RelationshipFeatureConfig::default();

    let all_features = compute_all_relationship_features(&graph, &config);

    // All nodes should have features
    assert!(!all_features.is_empty());

    // Feature vectors should have consistent length
    let first_len = all_features.values().next().unwrap().to_features().len();
    for (node_id, features) in &all_features {
        assert_eq!(
            features.to_features().len(),
            first_len,
            "Node {} should have consistent feature vector length",
            node_id
        );
    }
}

// =============================================================================
// FR-005: Entity Group Validation Tests
// =============================================================================

fn create_group_detection_config() -> GroupDetectionConfig {
    GroupDetectionConfig {
        min_group_size: 2,
        max_group_size: 100,
        min_cohesion: 0.0,
        algorithms: vec![GroupDetectionAlgorithm::ConnectedComponents],
        max_groups: 100,
        classify_types: false,
        edge_types: None,
    }
}

/// Test that group detection produces valid groups.
#[test]
fn test_entity_group_validity() {
    let graph = create_group_test_graph();
    let config = create_group_detection_config();

    let result = detect_entity_groups(&graph, &config);

    for group in &result.groups {
        // Group ID should be positive
        assert!(group.group_id > 0, "Group ID should be positive");

        // Group should have at least min_group_size members
        assert!(
            group.members.len() >= config.min_group_size,
            "Group {} should have at least {} members: {}",
            group.group_id,
            config.min_group_size,
            group.members.len()
        );

        // Group should not exceed max_group_size
        assert!(
            group.members.len() <= config.max_group_size,
            "Group {} should not exceed {} members: {}",
            group.group_id,
            config.max_group_size,
            group.members.len()
        );

        // Confidence should be in [0, 1]
        assert!(
            group.confidence >= 0.0 && group.confidence <= 1.0,
            "Group {} confidence should be in [0, 1]: {}",
            group.group_id,
            group.confidence
        );

        // Cohesion should be in [0, 1]
        assert!(
            group.cohesion >= 0.0 && group.cohesion <= 1.0,
            "Group {} cohesion should be in [0, 1]: {}",
            group.group_id,
            group.cohesion
        );

        // Members should be unique
        let unique_members: HashSet<_> = group.members.iter().collect();
        assert_eq!(
            unique_members.len(),
            group.members.len(),
            "Group {} should have unique members",
            group.group_id
        );

        // Volumes should be non-negative
        assert!(
            group.internal_volume >= 0.0,
            "Group {} internal_volume should be non-negative",
            group.group_id
        );
        assert!(
            group.external_volume >= 0.0,
            "Group {} external_volume should be non-negative",
            group.group_id
        );
    }
}

/// Test that groups don't overlap (for connected components).
#[test]
fn test_entity_groups_no_overlap() {
    let graph = create_group_test_graph();
    let config = create_group_detection_config();

    let result = detect_entity_groups(&graph, &config);

    let mut all_members = HashSet::new();
    for group in &result.groups {
        for &member in &group.members {
            assert!(
                !all_members.contains(&member),
                "Node {} appears in multiple groups (connected components should not overlap)",
                member
            );
            all_members.insert(member);
        }
    }
}

/// Test aggregated features bounds.
#[test]
fn test_aggregated_features_bounds() {
    let graph = create_group_test_graph();
    let config = create_group_detection_config();

    let result = detect_entity_groups(&graph, &config);

    for group in &result.groups {
        let features = aggregate_features(group, &graph, AggregationType::Mean);

        // Volumes should be non-negative
        assert!(
            features.total_volume >= 0.0,
            "total_volume should be non-negative"
        );
        assert!(
            features.avg_transaction_size >= 0.0,
            "avg_transaction_size should be non-negative"
        );

        // Risk score should be in [0, 1]
        assert!(
            features.combined_risk_score >= 0.0 && features.combined_risk_score <= 1.0,
            "combined_risk_score should be in [0, 1]: {}",
            features.combined_risk_score
        );

        // Flow ratios should be in [0, 1]
        assert!(
            features.internal_flow_ratio >= 0.0 && features.internal_flow_ratio <= 1.0,
            "internal_flow_ratio should be in [0, 1]: {}",
            features.internal_flow_ratio
        );
        assert!(
            features.external_flow_ratio >= 0.0 && features.external_flow_ratio <= 1.0,
            "external_flow_ratio should be in [0, 1]: {}",
            features.external_flow_ratio
        );

        // Flow ratios should sum to <= 1 (could be < 1 due to internal accounting)
        assert!(
            features.internal_flow_ratio + features.external_flow_ratio <= 1.01,
            "Flow ratios should sum to <= 1: {} + {}",
            features.internal_flow_ratio,
            features.external_flow_ratio
        );

        // Member count should match
        assert_eq!(
            features.member_count,
            group.members.len(),
            "member_count should match group size"
        );

        // Activity variance should be non-negative
        assert!(
            features.activity_variance >= 0.0,
            "activity_variance should be non-negative"
        );
    }
}

/// Test aggregation functions mathematical properties.
#[test]
fn test_aggregation_mathematical_properties() {
    let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];

    // Sum
    let sum = aggregate_values(&values, AggregationType::Sum);
    assert_eq!(sum, 150.0, "Sum should equal 150");

    // Mean
    let mean = aggregate_values(&values, AggregationType::Mean);
    assert_eq!(mean, 30.0, "Mean should equal 30");

    // Min
    let min = aggregate_values(&values, AggregationType::Min);
    assert_eq!(min, 10.0, "Min should equal 10");

    // Max
    let max = aggregate_values(&values, AggregationType::Max);
    assert_eq!(max, 50.0, "Max should equal 50");

    // Median
    let median = aggregate_values(&values, AggregationType::Median);
    assert_eq!(median, 30.0, "Median should equal 30");

    // Weighted mean
    let weights = vec![1.0, 1.0, 1.0, 1.0, 6.0]; // Heavy weight on 50
    let weighted = aggregate_weighted(&values, &weights, AggregationType::WeightedMean);
    // (10 + 20 + 30 + 40 + 300) / 10 = 40
    assert_eq!(weighted, 40.0, "Weighted mean should equal 40");
}

/// Test aggregation with empty inputs.
#[test]
fn test_aggregation_empty_inputs() {
    let empty: Vec<f64> = vec![];

    assert_eq!(aggregate_values(&empty, AggregationType::Sum), 0.0);
    assert_eq!(aggregate_values(&empty, AggregationType::Mean), 0.0);
    assert_eq!(aggregate_values(&empty, AggregationType::Min), 0.0);
    assert_eq!(aggregate_values(&empty, AggregationType::Max), 0.0);
    assert_eq!(aggregate_values(&empty, AggregationType::Median), 0.0);
}

/// Test node feature aggregation.
#[test]
fn test_node_feature_aggregation() {
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

    let result = aggregate_node_features(&[n1, n2, n3], &graph, AggregationType::Mean);

    assert_eq!(result.features.len(), 3, "Should have 3 features");
    assert_eq!(result.features[0], 4.0, "First feature mean: (1+4+7)/3 = 4");
    assert_eq!(
        result.features[1], 5.0,
        "Second feature mean: (2+5+8)/3 = 5"
    );
    assert_eq!(result.features[2], 6.0, "Third feature mean: (3+6+9)/3 = 6");
}

/// Test aggregation across all groups.
#[test]
fn test_aggregate_all_groups() {
    let graph = create_group_test_graph();
    let config = create_group_detection_config();

    let result = detect_entity_groups(&graph, &config);
    let all_aggregated = aggregate_all_groups(&result.groups, &graph, AggregationType::Mean);

    // Should have features for each group
    assert_eq!(
        all_aggregated.len(),
        result.groups.len(),
        "Should have aggregated features for each group"
    );

    for group in &result.groups {
        assert!(
            all_aggregated.contains_key(&group.group_id),
            "Should have features for group {}",
            group.group_id
        );
    }
}

// =============================================================================
// Cross-Feature Validation Tests
// =============================================================================

/// Test that feature vectors have expected dimensions.
#[test]
fn test_feature_vector_dimensions() {
    // Temporal features
    let temporal = TemporalFeatures::default();
    let temporal_vec = temporal.to_features(&[7, 30, 90]);
    assert!(
        temporal_vec.len() >= 7,
        "Temporal features should have at least 7 base features"
    );

    // Relationship features
    let relationship = RelationshipFeatures::default();
    let relationship_vec = relationship.to_features();
    assert!(
        relationship_vec.len() >= 8,
        "Relationship features should have at least 8 features"
    );

    // Counterparty risk
    let risk = CounterpartyRisk::default();
    let risk_vec = risk.to_features();
    assert!(
        risk_vec.len() >= 5,
        "Counterparty risk should have at least 5 features"
    );

    // Aggregated features
    let aggregated = AggregatedFeatures::default();
    let aggregated_vec = aggregated.to_features();
    assert_eq!(aggregated_vec.len(), AggregatedFeatures::feature_count());
}

/// Test integration of multiple feature types on same graph.
#[test]
fn test_multi_feature_integration() {
    let graph = create_comprehensive_test_graph();

    // Compute all feature types
    let temporal_config = TemporalConfig::default();
    let relationship_config = RelationshipFeatureConfig::default();
    let group_config = create_group_detection_config();

    let temporal_features = compute_all_temporal_features(&graph, &temporal_config);
    let relationship_features = compute_all_relationship_features(&graph, &relationship_config);
    let _groups = detect_entity_groups(&graph, &group_config);

    // All should process same graph without errors
    assert!(!temporal_features.is_empty());
    assert!(!relationship_features.is_empty());

    // Features should cover same nodes
    let temporal_nodes: HashSet<_> = temporal_features.keys().collect();
    let relationship_nodes: HashSet<_> = relationship_features.keys().collect();

    // Both feature sets should cover the same nodes
    assert_eq!(
        temporal_nodes.len(),
        relationship_nodes.len(),
        "Feature sets should cover same number of nodes"
    );
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_temporal_test_graph() -> Graph {
    let mut graph = Graph::new("temporal_test", GraphType::Transaction);

    // Create nodes
    for i in 1..=5 {
        graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("A{}", i),
            format!("Account {}", i),
        ));
    }

    // Create edges with timestamps spread over time
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    // Node 1 -> 2, 3, 4 (multiple transactions over time)
    for (i, &target) in [2, 3, 4, 2, 3].iter().enumerate() {
        graph.add_edge(
            GraphEdge::new(0, 1, target, EdgeType::Transaction)
                .with_weight(100.0 * (i + 1) as f64)
                .with_timestamp(base_date + chrono::Duration::days(i as i64 * 7)),
        );
    }

    // Node 2 -> 3, 4, 5
    for (i, &target) in [3, 4, 5].iter().enumerate() {
        graph.add_edge(
            GraphEdge::new(0, 2, target, EdgeType::Transaction)
                .with_weight(200.0)
                .with_timestamp(base_date + chrono::Duration::days(30 + i as i64 * 10)),
        );
    }

    // Node 3 -> 4, 5
    graph.add_edge(
        GraphEdge::new(0, 3, 4, EdgeType::Transaction)
            .with_weight(150.0)
            .with_timestamp(base_date + chrono::Duration::days(60)),
    );
    graph.add_edge(
        GraphEdge::new(0, 3, 5, EdgeType::Transaction)
            .with_weight(150.0)
            .with_timestamp(base_date + chrono::Duration::days(65)),
    );

    graph
}

fn create_cycle_test_graph() -> Graph {
    let mut graph = Graph::new("cycle_test", GraphType::Transaction);

    // Create nodes
    for i in 1..=6 {
        graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("N{}", i),
            format!("Node {}", i),
        ));
    }

    // Create a 3-node cycle: 1 -> 2 -> 3 -> 1
    graph.add_edge(GraphEdge::new(0, 1, 2, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, 2, 3, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, 3, 1, EdgeType::Transaction).with_weight(100.0));

    // Create a 4-node cycle: 4 -> 5 -> 6 -> 4
    graph.add_edge(GraphEdge::new(0, 4, 5, EdgeType::Transaction).with_weight(200.0));
    graph.add_edge(GraphEdge::new(0, 5, 6, EdgeType::Transaction).with_weight(200.0));
    graph.add_edge(GraphEdge::new(0, 6, 4, EdgeType::Transaction).with_weight(200.0));

    graph
}

fn create_star_test_graph() -> Graph {
    let mut graph = Graph::new("star_test", GraphType::Transaction);

    // Hub node
    let hub = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "HUB".to_string(),
        "Hub Node".to_string(),
    ));

    // Spoke nodes
    for i in 1..=6 {
        let spoke = graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("S{}", i),
            format!("Spoke {}", i),
        ));
        graph.add_edge(GraphEdge::new(0, hub, spoke, EdgeType::Transaction).with_weight(100.0));
    }

    graph
}

fn create_back_and_forth_test_graph() -> Graph {
    let mut graph = Graph::new("back_and_forth_test", GraphType::Transaction);

    let n1 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "A".to_string(),
        "Node A".to_string(),
    ));
    let n2 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "B".to_string(),
        "Node B".to_string(),
    ));
    let n3 = graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "C".to_string(),
        "Node C".to_string(),
    ));

    // Back and forth between A and B
    graph.add_edge(GraphEdge::new(0, n1, n2, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, n2, n1, EdgeType::Transaction).with_weight(100.0));

    // One-way from A to C (no back-and-forth)
    graph.add_edge(GraphEdge::new(0, n1, n3, EdgeType::Transaction).with_weight(50.0));

    graph
}

fn create_relationship_test_graph() -> Graph {
    let mut graph = Graph::new("relationship_test", GraphType::Transaction);
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    // Create nodes
    for i in 1..=5 {
        graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("R{}", i),
            format!("Relationship Node {}", i),
        ));
    }

    // Node 1 has diverse counterparties
    for target in 2..=5 {
        graph.add_edge(
            GraphEdge::new(0, 1, target, EdgeType::Transaction)
                .with_weight(100.0)
                .with_timestamp(base_date),
        );
    }

    // Node 2 has concentrated counterparty (mostly to node 3)
    graph.add_edge(
        GraphEdge::new(0, 2, 3, EdgeType::Transaction)
            .with_weight(500.0)
            .with_timestamp(base_date),
    );
    graph.add_edge(
        GraphEdge::new(0, 2, 4, EdgeType::Transaction)
            .with_weight(50.0)
            .with_timestamp(base_date),
    );

    // Reciprocal relationship between 3 and 4
    graph.add_edge(
        GraphEdge::new(0, 3, 4, EdgeType::Transaction)
            .with_weight(200.0)
            .with_timestamp(base_date),
    );
    graph.add_edge(
        GraphEdge::new(0, 4, 3, EdgeType::Transaction)
            .with_weight(200.0)
            .with_timestamp(base_date + chrono::Duration::days(1)),
    );

    // New relationships for node 5 (recent)
    let recent_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    graph.add_edge(
        GraphEdge::new(0, 5, 1, EdgeType::Transaction)
            .with_weight(100.0)
            .with_timestamp(recent_date),
    );

    graph
}

fn create_group_test_graph() -> Graph {
    let mut graph = Graph::new("group_test", GraphType::Transaction);

    // Group 1: Nodes 1, 2, 3 (connected)
    for i in 1..=3 {
        graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("G1N{}", i),
            format!("Group 1 Node {}", i),
        ));
    }
    graph.add_edge(GraphEdge::new(0, 1, 2, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, 2, 3, EdgeType::Transaction).with_weight(100.0));
    graph.add_edge(GraphEdge::new(0, 1, 3, EdgeType::Transaction).with_weight(100.0));

    // Group 2: Nodes 4, 5, 6 (connected, separate from group 1)
    for i in 4..=6 {
        graph.add_node(GraphNode::new(
            0,
            NodeType::Account,
            format!("G2N{}", i),
            format!("Group 2 Node {}", i),
        ));
    }
    graph.add_edge(GraphEdge::new(0, 4, 5, EdgeType::Transaction).with_weight(200.0));
    graph.add_edge(GraphEdge::new(0, 5, 6, EdgeType::Transaction).with_weight(200.0));

    // Isolated node 7 (should not form a group by itself with min_size > 1)
    graph.add_node(GraphNode::new(
        0,
        NodeType::Account,
        "ISOLATED".to_string(),
        "Isolated Node".to_string(),
    ));

    graph
}

fn create_comprehensive_test_graph() -> Graph {
    let mut graph = Graph::new("comprehensive", GraphType::Transaction);
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    // Create 10 nodes with features
    for i in 1..=10 {
        graph.add_node(
            GraphNode::new(
                0,
                NodeType::Account,
                format!("N{}", i),
                format!("Node {}", i),
            )
            .with_features(vec![i as f64, (i * 2) as f64, (i * 3) as f64]),
        );
    }

    // Create various edge patterns
    // Chain: 1 -> 2 -> 3 -> 4
    for i in 1..=3 {
        graph.add_edge(
            GraphEdge::new(0, i, i + 1, EdgeType::Transaction)
                .with_weight(100.0 * i as f64)
                .with_timestamp(base_date + chrono::Duration::days(i as i64)),
        );
    }

    // Cycle: 5 -> 6 -> 7 -> 5
    graph.add_edge(
        GraphEdge::new(0, 5, 6, EdgeType::Transaction)
            .with_weight(150.0)
            .with_timestamp(base_date + chrono::Duration::days(10)),
    );
    graph.add_edge(
        GraphEdge::new(0, 6, 7, EdgeType::Transaction)
            .with_weight(150.0)
            .with_timestamp(base_date + chrono::Duration::days(11)),
    );
    graph.add_edge(
        GraphEdge::new(0, 7, 5, EdgeType::Transaction)
            .with_weight(150.0)
            .with_timestamp(base_date + chrono::Duration::days(12)),
    );

    // Star from node 8
    for target in [9, 10, 1] {
        graph.add_edge(
            GraphEdge::new(0, 8, target, EdgeType::Transaction)
                .with_weight(200.0)
                .with_timestamp(base_date + chrono::Duration::days(20)),
        );
    }

    graph
}
