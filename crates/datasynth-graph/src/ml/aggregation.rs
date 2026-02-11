//! Feature aggregation for entity groups.
//!
//! This module provides aggregation functions for computing
//! group-level features from individual node features.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{Graph, NodeId};

use super::entity_groups::EntityGroup;

/// Aggregation method for combining features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum of all values.
    Sum,
    /// Arithmetic mean.
    Mean,
    /// Weighted mean (by transaction volume).
    WeightedMean,
    /// Maximum value.
    Max,
    /// Minimum value.
    Min,
    /// Median value.
    Median,
}

/// Aggregated features for an entity group.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedFeatures {
    /// Total transaction volume within the group.
    pub total_volume: f64,
    /// Average transaction size.
    pub avg_transaction_size: f64,
    /// Combined risk score for the group.
    pub combined_risk_score: f64,
    /// Ratio of internal flows to total.
    pub internal_flow_ratio: f64,
    /// Ratio of external flows to total.
    pub external_flow_ratio: f64,
    /// Number of unique external counterparties.
    pub external_counterparty_count: usize,
    /// Variance in member activity.
    pub activity_variance: f64,
    /// Number of members.
    pub member_count: usize,
}

impl AggregatedFeatures {
    /// Converts to a feature vector.
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            (self.total_volume + 1.0).ln(),
            (self.avg_transaction_size + 1.0).ln(),
            self.combined_risk_score,
            self.internal_flow_ratio,
            self.external_flow_ratio,
            self.external_counterparty_count as f64,
            self.activity_variance,
            self.member_count as f64,
        ]
    }

    /// Returns feature count.
    pub fn feature_count() -> usize {
        8
    }

    /// Returns feature names.
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "total_volume_log",
            "avg_transaction_size_log",
            "combined_risk_score",
            "internal_flow_ratio",
            "external_flow_ratio",
            "external_counterparty_count",
            "activity_variance",
            "member_count",
        ]
    }
}

/// Aggregates features for a group of nodes.
pub fn aggregate_features(
    group: &EntityGroup,
    graph: &Graph,
    _agg_type: AggregationType,
) -> AggregatedFeatures {
    let member_set: std::collections::HashSet<NodeId> = group.members.iter().copied().collect();

    let mut total_volume = 0.0;
    let mut internal_volume = 0.0;
    let mut external_volume = 0.0;
    let mut transaction_count = 0;
    let mut external_counterparties = std::collections::HashSet::new();
    let mut member_activities = Vec::new();

    // Calculate volumes and counterparties
    for &member in &group.members {
        let mut member_activity = 0.0;

        for edge in graph.outgoing_edges(member) {
            total_volume += edge.weight;
            member_activity += edge.weight;
            transaction_count += 1;

            if member_set.contains(&edge.target) {
                internal_volume += edge.weight;
            } else {
                external_volume += edge.weight;
                external_counterparties.insert(edge.target);
            }
        }

        for edge in graph.incoming_edges(member) {
            if !member_set.contains(&edge.source) {
                external_counterparties.insert(edge.source);
            }
        }

        member_activities.push(member_activity);
    }

    // Calculate averages and ratios
    let avg_transaction_size = if transaction_count > 0 {
        total_volume / transaction_count as f64
    } else {
        0.0
    };

    let total_flow = internal_volume + external_volume;
    let internal_flow_ratio = if total_flow > 0.0 {
        internal_volume / total_flow
    } else {
        0.0
    };
    let external_flow_ratio = if total_flow > 0.0 {
        external_volume / total_flow
    } else {
        0.0
    };

    // Calculate activity variance
    let mean_activity = if !member_activities.is_empty() {
        member_activities.iter().sum::<f64>() / member_activities.len() as f64
    } else {
        0.0
    };

    let activity_variance = if member_activities.len() > 1 {
        let variance: f64 = member_activities
            .iter()
            .map(|&a| (a - mean_activity).powi(2))
            .sum::<f64>()
            / member_activities.len() as f64;
        variance.sqrt() / (mean_activity + 1.0) // Coefficient of variation
    } else {
        0.0
    };

    // Calculate combined risk score
    let anomalous_members = group
        .members
        .iter()
        .filter(|&&n| {
            graph
                .get_node(n)
                .map(|node| node.is_anomaly)
                .unwrap_or(false)
        })
        .count();

    let anomalous_edges = group
        .members
        .iter()
        .flat_map(|&n| {
            graph
                .outgoing_edges(n)
                .into_iter()
                .chain(graph.incoming_edges(n))
        })
        .filter(|e| e.is_anomaly)
        .count();

    let total_edges = group
        .members
        .iter()
        .map(|&n| graph.degree(n))
        .sum::<usize>();

    let member_risk = anomalous_members as f64 / group.members.len().max(1) as f64;
    let edge_risk = anomalous_edges as f64 / total_edges.max(1) as f64;
    let combined_risk_score = (member_risk * 0.6 + edge_risk * 0.4).min(1.0);

    AggregatedFeatures {
        total_volume,
        avg_transaction_size,
        combined_risk_score,
        internal_flow_ratio,
        external_flow_ratio,
        external_counterparty_count: external_counterparties.len(),
        activity_variance,
        member_count: group.members.len(),
    }
}

/// Aggregates a specific feature across multiple values.
pub fn aggregate_values(values: &[f64], agg_type: AggregationType) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    match agg_type {
        AggregationType::Sum => values.iter().sum(),
        AggregationType::Mean => values.iter().sum::<f64>() / values.len() as f64,
        AggregationType::WeightedMean => {
            // Without weights, defaults to mean
            values.iter().sum::<f64>() / values.len() as f64
        }
        AggregationType::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        AggregationType::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
        AggregationType::Median => {
            let mut sorted = values.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted.len() / 2;
            if sorted.len().is_multiple_of(2) {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        }
    }
}

/// Aggregates weighted values.
pub fn aggregate_weighted(values: &[f64], weights: &[f64], agg_type: AggregationType) -> f64 {
    if values.is_empty() || weights.is_empty() || values.len() != weights.len() {
        return aggregate_values(values, agg_type);
    }

    match agg_type {
        AggregationType::WeightedMean => {
            let total_weight: f64 = weights.iter().sum();
            if total_weight > 0.0 {
                let weighted_sum: f64 = values.iter().zip(weights.iter()).map(|(v, w)| v * w).sum();
                weighted_sum / total_weight
            } else {
                aggregate_values(values, AggregationType::Mean)
            }
        }
        _ => aggregate_values(values, agg_type),
    }
}

/// Aggregates features for all groups.
pub fn aggregate_all_groups(
    groups: &[EntityGroup],
    graph: &Graph,
    agg_type: AggregationType,
) -> HashMap<u64, AggregatedFeatures> {
    let mut result = HashMap::new();

    for group in groups {
        let features = aggregate_features(group, graph, agg_type);
        result.insert(group.group_id, features);
    }

    result
}

/// Multi-feature aggregation result.
#[derive(Debug, Clone)]
pub struct MultiFeatureAggregation {
    /// Aggregated features per dimension.
    pub features: Vec<f64>,
    /// Feature names.
    pub names: Vec<String>,
}

impl MultiFeatureAggregation {
    /// Creates a new multi-feature aggregation.
    pub fn new(features: Vec<f64>, names: Vec<String>) -> Self {
        Self { features, names }
    }

    /// Returns the feature vector.
    pub fn to_features(&self) -> &[f64] {
        &self.features
    }
}

/// Aggregates multiple node feature vectors into a single vector.
pub fn aggregate_node_features(
    node_ids: &[NodeId],
    graph: &Graph,
    agg_type: AggregationType,
) -> MultiFeatureAggregation {
    if node_ids.is_empty() {
        return MultiFeatureAggregation::new(Vec::new(), Vec::new());
    }

    // Collect features from all nodes
    let node_features: Vec<Vec<f64>> = node_ids
        .iter()
        .filter_map(|&id| graph.get_node(id))
        .map(|n| n.features.clone())
        .filter(|f| !f.is_empty())
        .collect();

    if node_features.is_empty() {
        return MultiFeatureAggregation::new(Vec::new(), Vec::new());
    }

    // Find feature dimension
    let dim = node_features[0].len();

    // Aggregate each dimension
    let aggregated: Vec<f64> = (0..dim)
        .map(|d| {
            let values: Vec<f64> = node_features
                .iter()
                .map(|f| f.get(d).copied().unwrap_or(0.0))
                .collect();
            aggregate_values(&values, agg_type)
        })
        .collect();

    let names: Vec<String> = (0..dim).map(|d| format!("feature_{}", d)).collect();

    MultiFeatureAggregation::new(aggregated, names)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::create_aggregation_test_graph;

    #[test]
    fn test_aggregate_values_sum() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(aggregate_values(&values, AggregationType::Sum), 15.0);
    }

    #[test]
    fn test_aggregate_values_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(aggregate_values(&values, AggregationType::Mean), 3.0);
    }

    #[test]
    fn test_aggregate_values_max() {
        let values = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        assert_eq!(aggregate_values(&values, AggregationType::Max), 5.0);
    }

    #[test]
    fn test_aggregate_values_min() {
        let values = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        assert_eq!(aggregate_values(&values, AggregationType::Min), 1.0);
    }

    #[test]
    fn test_aggregate_values_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(aggregate_values(&values, AggregationType::Median), 3.0);
    }

    #[test]
    fn test_aggregate_values_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(aggregate_values(&values, AggregationType::Median), 2.5);
    }

    #[test]
    fn test_aggregate_weighted() {
        let values = vec![10.0, 20.0, 30.0];
        let weights = vec![1.0, 2.0, 1.0];

        let result = aggregate_weighted(&values, &weights, AggregationType::WeightedMean);
        // (10*1 + 20*2 + 30*1) / 4 = 80/4 = 20
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_aggregate_features() {
        let graph = create_aggregation_test_graph();
        let group = EntityGroup::new(
            1,
            vec![1, 2, 3],
            super::super::entity_groups::GroupType::TransactionCluster,
        );

        let features = aggregate_features(&group, &graph, AggregationType::Sum);

        assert!(features.total_volume > 0.0);
        assert_eq!(features.member_count, 3);
    }

    #[test]
    fn test_aggregate_node_features() {
        let graph = create_aggregation_test_graph();
        let result = aggregate_node_features(&[1, 2, 3], &graph, AggregationType::Mean);

        assert_eq!(result.features.len(), 3);
        // Mean of [1,4,7], [2,5,8], [3,6,9] = [4, 5, 6]
        assert_eq!(result.features[0], 4.0);
        assert_eq!(result.features[1], 5.0);
        assert_eq!(result.features[2], 6.0);
    }

    #[test]
    fn test_aggregated_features_to_vector() {
        let features = AggregatedFeatures {
            total_volume: 1000.0,
            avg_transaction_size: 100.0,
            combined_risk_score: 0.5,
            internal_flow_ratio: 0.6,
            external_flow_ratio: 0.4,
            external_counterparty_count: 5,
            activity_variance: 0.3,
            member_count: 3,
        };

        let vec = features.to_features();
        assert_eq!(vec.len(), AggregatedFeatures::feature_count());
    }
}
