//! Entity relationship feature computation for fraud detection.
//!
//! This module provides relationship-based features including:
//! - Counterparty concentration (Herfindahl index)
//! - Relationship age and velocity
//! - Reciprocity (bidirectional transaction patterns)
//! - Counterparty risk propagation

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::{Graph, NodeId};

/// Configuration for relationship feature computation.
#[derive(Debug, Clone)]
pub struct RelationshipFeatureConfig {
    /// Number of days to consider a relationship "new".
    pub new_relationship_days: i64,
    /// Reference date for age calculations.
    pub reference_date: NaiveDate,
    /// Threshold for high-risk counterparty classification.
    pub high_risk_threshold: f64,
    /// Whether to weight features by transaction amount.
    pub weight_by_amount: bool,
    /// Minimum number of transactions for meaningful features.
    pub min_transactions: usize,
}

impl Default for RelationshipFeatureConfig {
    fn default() -> Self {
        Self {
            new_relationship_days: 30,
            reference_date: NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid default date"),
            high_risk_threshold: 0.5,
            weight_by_amount: true,
            min_transactions: 1,
        }
    }
}

/// Relationship features for a node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipFeatures {
    /// Number of unique counterparties.
    pub unique_counterparties: usize,
    /// Ratio of new relationships (< new_relationship_days old).
    pub new_relationship_ratio: f64,
    /// Herfindahl-Hirschman Index for counterparty concentration.
    pub counterparty_concentration: f64,
    /// Ratio of bidirectional relationships.
    pub relationship_reciprocity: f64,
    /// Average relationship age in days.
    pub avg_relationship_age_days: f64,
    /// Rate of new relationships per month.
    pub relationship_velocity: f64,
    /// Total number of relationships (including multiple txns per counterparty).
    pub total_relationships: usize,
    /// Share of transactions with dominant counterparty.
    pub dominant_counterparty_share: f64,
}

impl RelationshipFeatures {
    /// Converts to a feature vector.
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.unique_counterparties as f64,
            self.new_relationship_ratio,
            self.counterparty_concentration,
            self.relationship_reciprocity,
            self.avg_relationship_age_days / 365.0, // Normalize to years
            self.relationship_velocity,
            self.total_relationships as f64,
            self.dominant_counterparty_share,
        ]
    }

    /// Returns the number of features.
    pub fn feature_count() -> usize {
        8
    }

    /// Returns feature names.
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "unique_counterparties",
            "new_relationship_ratio",
            "counterparty_concentration_hhi",
            "relationship_reciprocity",
            "avg_relationship_age_years",
            "relationship_velocity",
            "total_relationships",
            "dominant_counterparty_share",
        ]
    }
}

/// Counterparty risk features for a node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterpartyRisk {
    /// Ratio of high-risk counterparties.
    pub high_risk_counterparty_ratio: f64,
    /// Average risk score of counterparties.
    pub avg_counterparty_risk_score: f64,
    /// Concentration of risk in few counterparties.
    pub risk_concentration: f64,
    /// Number of anomalous counterparties.
    pub anomalous_counterparty_count: usize,
    /// Total exposure to high-risk counterparties (by amount).
    pub high_risk_exposure: f64,
}

impl CounterpartyRisk {
    /// Converts to a feature vector.
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.high_risk_counterparty_ratio,
            self.avg_counterparty_risk_score,
            self.risk_concentration,
            self.anomalous_counterparty_count as f64,
            (self.high_risk_exposure + 1.0).ln(),
        ]
    }

    /// Returns the number of features.
    pub fn feature_count() -> usize {
        5
    }

    /// Returns feature names.
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "high_risk_counterparty_ratio",
            "avg_counterparty_risk_score",
            "risk_concentration",
            "anomalous_counterparty_count",
            "high_risk_exposure_log",
        ]
    }
}

/// Internal structure for tracking counterparty relationships.
#[derive(Debug, Clone, Default)]
struct CounterpartyInfo {
    /// First transaction date with this counterparty.
    first_contact: Option<NaiveDate>,
    /// Total transaction count.
    transaction_count: usize,
    /// Total transaction volume.
    total_volume: f64,
    /// Is this counterparty anomalous.
    is_anomalous: bool,
    /// Risk score for this counterparty.
    risk_score: f64,
}

/// Computes relationship features for a single node.
pub fn compute_relationship_features(
    node_id: NodeId,
    graph: &Graph,
    config: &RelationshipFeatureConfig,
) -> RelationshipFeatures {
    let outgoing = graph.outgoing_edges(node_id);
    let incoming = graph.incoming_edges(node_id);

    if outgoing.is_empty() && incoming.is_empty() {
        return RelationshipFeatures::default();
    }

    // Build counterparty info
    let mut counterparties: HashMap<NodeId, CounterpartyInfo> = HashMap::new();
    let mut outgoing_targets: HashSet<NodeId> = HashSet::new();
    let mut incoming_sources: HashSet<NodeId> = HashSet::new();

    // Process outgoing edges
    for edge in &outgoing {
        outgoing_targets.insert(edge.target);
        let info = counterparties.entry(edge.target).or_default();
        info.transaction_count += 1;
        info.total_volume += edge.weight;

        if let Some(date) = edge.timestamp {
            match info.first_contact {
                None => info.first_contact = Some(date),
                Some(existing) if date < existing => info.first_contact = Some(date),
                _ => {}
            }
        }
    }

    // Process incoming edges
    for edge in &incoming {
        incoming_sources.insert(edge.source);
        let info = counterparties.entry(edge.source).or_default();
        info.transaction_count += 1;
        info.total_volume += edge.weight;

        if let Some(date) = edge.timestamp {
            match info.first_contact {
                None => info.first_contact = Some(date),
                Some(existing) if date < existing => info.first_contact = Some(date),
                _ => {}
            }
        }
    }

    let unique_counterparties = counterparties.len();
    let total_relationships = outgoing.len() + incoming.len();

    if unique_counterparties == 0 {
        return RelationshipFeatures::default();
    }

    // Calculate new relationship ratio
    let new_threshold =
        config.reference_date - chrono::Duration::days(config.new_relationship_days);
    let new_count = counterparties
        .values()
        .filter(|info| {
            info.first_contact
                .map(|d| d >= new_threshold)
                .unwrap_or(false)
        })
        .count();
    let new_relationship_ratio = new_count as f64 / unique_counterparties as f64;

    // Calculate HHI for concentration
    let total_volume: f64 = counterparties.values().map(|i| i.total_volume).sum();
    let counterparty_concentration = if total_volume > 0.0 {
        counterparties
            .values()
            .map(|info| {
                let share = info.total_volume / total_volume;
                share * share
            })
            .sum()
    } else {
        1.0 / unique_counterparties as f64 // Equal distribution
    };

    // Calculate reciprocity (bidirectional relationships)
    let bidirectional_count = outgoing_targets.intersection(&incoming_sources).count();
    let relationship_reciprocity = if unique_counterparties > 0 {
        bidirectional_count as f64 / unique_counterparties as f64
    } else {
        0.0
    };

    // Calculate average relationship age
    let ages: Vec<i64> = counterparties
        .values()
        .filter_map(|info| info.first_contact)
        .map(|date| (config.reference_date - date).num_days().max(0))
        .collect();

    let avg_relationship_age_days = if !ages.is_empty() {
        ages.iter().sum::<i64>() as f64 / ages.len() as f64
    } else {
        0.0
    };

    // Calculate relationship velocity (new relationships per month)
    let date_range = counterparties
        .values()
        .filter_map(|info| info.first_contact)
        .fold((None, None), |(min, max), date| {
            let new_min = min.map_or(date, |m: NaiveDate| m.min(date));
            let new_max = max.map_or(date, |m: NaiveDate| m.max(date));
            (Some(new_min), Some(new_max))
        });

    let relationship_velocity = if let (Some(min_date), Some(max_date)) = date_range {
        let months = (max_date - min_date).num_days() as f64 / 30.0;
        if months > 0.0 {
            unique_counterparties as f64 / months
        } else {
            unique_counterparties as f64
        }
    } else {
        0.0
    };

    // Calculate dominant counterparty share
    let max_volume = counterparties
        .values()
        .map(|i| i.total_volume)
        .fold(0.0, f64::max);
    let dominant_counterparty_share = if total_volume > 0.0 {
        max_volume / total_volume
    } else {
        0.0
    };

    RelationshipFeatures {
        unique_counterparties,
        new_relationship_ratio,
        counterparty_concentration,
        relationship_reciprocity,
        avg_relationship_age_days,
        relationship_velocity,
        total_relationships,
        dominant_counterparty_share,
    }
}

/// Computes counterparty risk features for a node.
pub fn compute_counterparty_risk(
    node_id: NodeId,
    graph: &Graph,
    config: &RelationshipFeatureConfig,
) -> CounterpartyRisk {
    let outgoing = graph.outgoing_edges(node_id);
    let incoming = graph.incoming_edges(node_id);

    if outgoing.is_empty() && incoming.is_empty() {
        return CounterpartyRisk::default();
    }

    // Build counterparty info with risk scores
    let mut counterparties: HashMap<NodeId, CounterpartyInfo> = HashMap::new();

    // Process all edges
    for edge in outgoing.iter().chain(incoming.iter()) {
        let counterparty_id = if edge.source == node_id {
            edge.target
        } else {
            edge.source
        };

        let info = counterparties.entry(counterparty_id).or_default();
        info.transaction_count += 1;
        info.total_volume += edge.weight;

        // Inherit anomaly status from edge
        if edge.is_anomaly {
            info.is_anomalous = true;
        }
    }

    // Calculate risk scores for counterparties
    for (&cp_id, info) in counterparties.iter_mut() {
        let cp_node = graph.get_node(cp_id);

        // Base risk from counterparty's anomaly status
        let mut risk = 0.0;

        if let Some(node) = cp_node {
            if node.is_anomaly {
                risk += 0.5;
                info.is_anomalous = true;
            }
        }

        // Risk from edge anomalies with this counterparty
        let cp_edges: Vec<_> = outgoing
            .iter()
            .chain(incoming.iter())
            .filter(|e| e.source == cp_id || e.target == cp_id)
            .collect();

        let anomalous_edge_ratio =
            cp_edges.iter().filter(|e| e.is_anomaly).count() as f64 / cp_edges.len().max(1) as f64;
        risk += anomalous_edge_ratio * 0.3;

        // Risk from having suspicious labels
        if let Some(node) = cp_node {
            let suspicious_labels = ["fraud", "suspicious", "high_risk", "flagged"];
            for label in &node.labels {
                if suspicious_labels
                    .iter()
                    .any(|s| label.to_lowercase().contains(s))
                {
                    risk += 0.2;
                    break;
                }
            }
        }

        info.risk_score = risk.min(1.0);
    }

    let unique_counterparties = counterparties.len();
    if unique_counterparties == 0 {
        return CounterpartyRisk::default();
    }

    // Calculate high-risk counterparty ratio
    let high_risk_count = counterparties
        .values()
        .filter(|info| info.risk_score >= config.high_risk_threshold)
        .count();
    let high_risk_counterparty_ratio = high_risk_count as f64 / unique_counterparties as f64;

    // Calculate average risk score
    let total_risk: f64 = counterparties.values().map(|i| i.risk_score).sum();
    let avg_counterparty_risk_score = total_risk / unique_counterparties as f64;

    // Calculate risk concentration (HHI of risk-weighted volume)
    let total_risk_weighted: f64 = counterparties
        .values()
        .map(|i| i.total_volume * i.risk_score)
        .sum();

    let risk_concentration = if total_risk_weighted > 0.0 {
        counterparties
            .values()
            .map(|info| {
                let weighted = info.total_volume * info.risk_score;
                let share = weighted / total_risk_weighted;
                share * share
            })
            .sum()
    } else {
        0.0
    };

    // Count anomalous counterparties
    let anomalous_counterparty_count = counterparties.values().filter(|i| i.is_anomalous).count();

    // Calculate high-risk exposure
    let high_risk_exposure: f64 = counterparties
        .values()
        .filter(|info| info.risk_score >= config.high_risk_threshold)
        .map(|info| info.total_volume)
        .sum();

    CounterpartyRisk {
        high_risk_counterparty_ratio,
        avg_counterparty_risk_score,
        risk_concentration,
        anomalous_counterparty_count,
        high_risk_exposure,
    }
}

/// Computes relationship features for all nodes in a graph.
pub fn compute_all_relationship_features(
    graph: &Graph,
    config: &RelationshipFeatureConfig,
) -> HashMap<NodeId, RelationshipFeatures> {
    let mut features = HashMap::new();

    for &node_id in graph.nodes.keys() {
        features.insert(
            node_id,
            compute_relationship_features(node_id, graph, config),
        );
    }

    features
}

/// Computes counterparty risk for all nodes in a graph.
pub fn compute_all_counterparty_risk(
    graph: &Graph,
    config: &RelationshipFeatureConfig,
) -> HashMap<NodeId, CounterpartyRisk> {
    let mut risks = HashMap::new();

    for &node_id in graph.nodes.keys() {
        risks.insert(node_id, compute_counterparty_risk(node_id, graph, config));
    }

    risks
}

/// Combined relationship and risk features.
#[derive(Debug, Clone, Default)]
pub struct CombinedRelationshipFeatures {
    /// Base relationship features.
    pub relationship: RelationshipFeatures,
    /// Counterparty risk features.
    pub risk: CounterpartyRisk,
}

impl CombinedRelationshipFeatures {
    /// Converts to a combined feature vector.
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = self.relationship.to_features();
        features.extend(self.risk.to_features());
        features
    }

    /// Returns total feature count.
    pub fn feature_count() -> usize {
        RelationshipFeatures::feature_count() + CounterpartyRisk::feature_count()
    }

    /// Returns all feature names.
    pub fn feature_names() -> Vec<&'static str> {
        let mut names = RelationshipFeatures::feature_names();
        names.extend(CounterpartyRisk::feature_names());
        names
    }
}

/// Computes combined features for all nodes.
pub fn compute_all_combined_features(
    graph: &Graph,
    config: &RelationshipFeatureConfig,
) -> HashMap<NodeId, CombinedRelationshipFeatures> {
    let mut features = HashMap::new();

    for &node_id in graph.nodes.keys() {
        features.insert(
            node_id,
            CombinedRelationshipFeatures {
                relationship: compute_relationship_features(node_id, graph, config),
                risk: compute_counterparty_risk(node_id, graph, config),
            },
        );
    }

    features
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::create_relationship_test_graph;

    #[test]
    fn test_relationship_features() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let features = compute_relationship_features(1, &graph, &config);

        assert_eq!(features.unique_counterparties, 3); // N2, N3, N4
        assert!(features.new_relationship_ratio > 0.0); // N4 is new
        assert!(features.counterparty_concentration > 0.0);
        assert!(features.relationship_reciprocity > 0.0); // N2 is bidirectional
    }

    #[test]
    fn test_herfindahl_index() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let features = compute_relationship_features(1, &graph, &config);

        // HHI should be between 0 and 1
        assert!(features.counterparty_concentration > 0.0);
        assert!(features.counterparty_concentration <= 1.0);

        // With 3 counterparties and unequal volumes, should be > 1/3
        assert!(features.counterparty_concentration > 0.33);
    }

    #[test]
    fn test_reciprocity() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let features = compute_relationship_features(1, &graph, &config);

        // N1 has 3 unique counterparties, 1 bidirectional (N2)
        // Reciprocity = 1/3 = 0.333...
        assert!((features.relationship_reciprocity - 0.333).abs() < 0.1);
    }

    #[test]
    fn test_counterparty_risk_basic() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let risk = compute_counterparty_risk(1, &graph, &config);

        // No anomalies in test graph
        assert_eq!(risk.anomalous_counterparty_count, 0);
        assert_eq!(risk.avg_counterparty_risk_score, 0.0);
    }

    #[test]
    fn test_counterparty_risk_with_anomalies() {
        let mut graph = create_relationship_test_graph();

        // Mark an edge as anomalous
        if let Some(edge) = graph.get_edge_mut(1) {
            edge.is_anomaly = true;
        }

        let config = RelationshipFeatureConfig::default();
        let risk = compute_counterparty_risk(1, &graph, &config);

        // Should detect the anomalous relationship
        assert!(risk.avg_counterparty_risk_score > 0.0);
    }

    #[test]
    fn test_feature_vector_length() {
        assert_eq!(RelationshipFeatures::feature_count(), 8);
        assert_eq!(CounterpartyRisk::feature_count(), 5);
        assert_eq!(CombinedRelationshipFeatures::feature_count(), 13);

        let features = RelationshipFeatures::default();
        assert_eq!(
            features.to_features().len(),
            RelationshipFeatures::feature_count()
        );

        let risk = CounterpartyRisk::default();
        assert_eq!(risk.to_features().len(), CounterpartyRisk::feature_count());
    }

    #[test]
    fn test_all_relationship_features() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let all_features = compute_all_relationship_features(&graph, &config);

        assert_eq!(all_features.len(), 4); // 4 nodes
    }

    #[test]
    fn test_combined_features() {
        let graph = create_relationship_test_graph();
        let config = RelationshipFeatureConfig::default();

        let combined = compute_all_combined_features(&graph, &config);

        for (_node_id, features) in combined {
            assert_eq!(
                features.to_features().len(),
                CombinedRelationshipFeatures::feature_count()
            );
        }
    }
}
