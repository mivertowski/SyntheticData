//! Temporal sequence feature computation for graph nodes.
//!
//! This module provides temporal analysis features including:
//! - Transaction velocity (amount per time)
//! - Inter-event interval statistics
//! - Burst detection using Kleinberg-style counting
//! - Trend analysis via linear regression
//! - Seasonality scoring via weekly pattern variance
//! - Per-window aggregation features

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::models::{EdgeId, Graph, NodeId};

/// Configuration for temporal feature computation.
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    /// Window sizes in days for aggregation (e.g., [7, 30, 90]).
    pub window_sizes: Vec<i64>,
    /// Reference date for computing recency. If None, uses max date in data.
    pub reference_date: Option<NaiveDate>,
    /// Minimum number of edges for a node to have temporal features computed.
    pub min_edge_count: usize,
    /// Threshold multiplier for burst detection (events > threshold * mean = burst).
    pub burst_threshold: f64,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            window_sizes: vec![7, 30, 90],
            reference_date: None,
            min_edge_count: 2,
            burst_threshold: 3.0,
        }
    }
}

/// Aggregated features for a specific time window.
#[derive(Debug, Clone, Default)]
pub struct WindowFeatures {
    /// Number of events in the window.
    pub event_count: usize,
    /// Total amount (sum of edge weights) in the window.
    pub total_amount: f64,
    /// Average amount per event.
    pub avg_amount: f64,
    /// Maximum amount in the window.
    pub max_amount: f64,
    /// Number of unique counterparties in the window.
    pub unique_counterparties: usize,
}

impl WindowFeatures {
    /// Converts window features to a feature vector.
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.event_count as f64,
            (self.total_amount + 1.0).ln(),
            (self.avg_amount + 1.0).ln(),
            (self.max_amount + 1.0).ln(),
            self.unique_counterparties as f64,
        ]
    }
}

/// Temporal sequence features for a single node.
#[derive(Debug, Clone, Default)]
pub struct TemporalFeatures {
    /// Transaction velocity: total amount / time span in days.
    pub transaction_velocity: f64,
    /// Mean inter-event interval in days.
    pub inter_event_interval_mean: f64,
    /// Standard deviation of inter-event intervals.
    pub inter_event_interval_std: f64,
    /// Burst score: max daily event count / mean daily event count.
    pub burst_score: f64,
    /// Trend direction: +1.0 (increasing), -1.0 (decreasing), 0.0 (stable).
    pub trend_direction: f64,
    /// Seasonality score: variance of weekday activity normalized.
    pub seasonality_score: f64,
    /// Days since last event (recency).
    pub recency_days: f64,
    /// Per-window aggregated features.
    pub window_features: HashMap<i64, WindowFeatures>,
}

impl TemporalFeatures {
    /// Converts temporal features to a flat feature vector.
    /// Returns base features + window features for each configured window.
    pub fn to_features(&self, window_sizes: &[i64]) -> Vec<f64> {
        let mut features = vec![
            (self.transaction_velocity + 1.0).ln(),
            self.inter_event_interval_mean,
            self.inter_event_interval_std,
            self.burst_score,
            self.trend_direction,
            self.seasonality_score,
            self.recency_days / 365.0, // Normalize to ~[0, 1] for yearly data
        ];

        // Add window features in order
        for &window in window_sizes {
            if let Some(wf) = self.window_features.get(&window) {
                features.extend(wf.to_features());
            } else {
                // Default values if window not present
                features.extend(vec![0.0; 5]);
            }
        }

        features
    }

    /// Returns the number of features in the output vector.
    pub fn feature_count(window_count: usize) -> usize {
        7 + (5 * window_count) // 7 base features + 5 per window
    }
}

/// Index for efficient temporal queries on graph edges.
#[derive(Debug, Clone)]
pub struct TemporalIndex {
    /// For each node, sorted list of (date, edge_id) pairs.
    node_edges_by_date: HashMap<NodeId, Vec<(NaiveDate, EdgeId)>>,
    /// Minimum date in the index.
    pub min_date: Option<NaiveDate>,
    /// Maximum date in the index.
    pub max_date: Option<NaiveDate>,
}

impl TemporalIndex {
    /// Builds a temporal index from a graph.
    /// Complexity: O(E log E) for sorting edges by date.
    pub fn build(graph: &Graph) -> Self {
        let mut node_edges: HashMap<NodeId, Vec<(NaiveDate, EdgeId)>> = HashMap::new();
        let mut min_date: Option<NaiveDate> = None;
        let mut max_date: Option<NaiveDate> = None;

        // Collect edges with timestamps
        for (&edge_id, edge) in &graph.edges {
            if let Some(date) = edge.timestamp {
                // Update global date range
                min_date = Some(min_date.map_or(date, |d| d.min(date)));
                max_date = Some(max_date.map_or(date, |d| d.max(date)));

                // Add to source and target node indices
                node_edges
                    .entry(edge.source)
                    .or_default()
                    .push((date, edge_id));
                node_edges
                    .entry(edge.target)
                    .or_default()
                    .push((date, edge_id));
            }
        }

        // Sort edges by date for each node
        for edges in node_edges.values_mut() {
            edges.sort_by_key(|(date, _)| *date);
        }

        Self {
            node_edges_by_date: node_edges,
            min_date,
            max_date,
        }
    }

    /// Returns edges for a node within a date range (inclusive).
    pub fn edges_in_range(
        &self,
        node_id: NodeId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<(NaiveDate, EdgeId)> {
        if let Some(edges) = self.node_edges_by_date.get(&node_id) {
            // Binary search for start position
            let start_idx = edges.partition_point(|(d, _)| *d < start);
            // Binary search for end position
            let end_idx = edges.partition_point(|(d, _)| *d <= end);

            edges[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Returns all edges for a node, sorted by date.
    pub fn edges_for_node(&self, node_id: NodeId) -> &[(NaiveDate, EdgeId)] {
        self.node_edges_by_date
            .get(&node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the number of nodes with temporal data.
    pub fn node_count(&self) -> usize {
        self.node_edges_by_date.len()
    }
}

/// Computes temporal sequence features for a single node.
pub fn compute_temporal_sequence_features(
    node_id: NodeId,
    graph: &Graph,
    index: &TemporalIndex,
    config: &TemporalConfig,
) -> TemporalFeatures {
    let edges = index.edges_for_node(node_id);

    // Return default if insufficient data
    if edges.len() < config.min_edge_count {
        return TemporalFeatures::default();
    }

    let reference_date = config
        .reference_date
        .or(index.max_date)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    // Compute inter-event intervals
    let (interval_mean, interval_std) = compute_inter_event_intervals(edges);

    // Compute transaction velocity
    let transaction_velocity = compute_transaction_velocity(edges, graph);

    // Compute burst score
    let burst_score = compute_burst_score(edges, config.burst_threshold);

    // Compute trend direction
    let trend_direction = compute_trend_direction(edges, graph);

    // Compute seasonality score
    let seasonality_score = compute_seasonality_score(edges);

    // Compute recency
    let recency_days = if let Some((last_date, _)) = edges.last() {
        (reference_date - *last_date).num_days().max(0) as f64
    } else {
        f64::MAX
    };

    // Compute window features
    let mut window_features = HashMap::new();
    for &window in &config.window_sizes {
        let wf = compute_window_features(node_id, graph, index, reference_date, window);
        window_features.insert(window, wf);
    }

    TemporalFeatures {
        transaction_velocity,
        inter_event_interval_mean: interval_mean,
        inter_event_interval_std: interval_std,
        burst_score,
        trend_direction,
        seasonality_score,
        recency_days,
        window_features,
    }
}

/// Computes temporal features for all nodes in the graph.
pub fn compute_all_temporal_features(
    graph: &Graph,
    config: &TemporalConfig,
) -> HashMap<NodeId, TemporalFeatures> {
    let index = TemporalIndex::build(graph);
    let mut features = HashMap::new();

    for &node_id in graph.nodes.keys() {
        let node_features = compute_temporal_sequence_features(node_id, graph, &index, config);
        features.insert(node_id, node_features);
    }

    features
}

/// Computes mean and standard deviation of inter-event intervals.
fn compute_inter_event_intervals(edges: &[(NaiveDate, EdgeId)]) -> (f64, f64) {
    if edges.len() < 2 {
        return (0.0, 0.0);
    }

    let intervals: Vec<f64> = edges
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).num_days() as f64)
        .collect();

    let n = intervals.len() as f64;
    let mean = intervals.iter().sum::<f64>() / n;
    let variance = intervals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    (mean, std)
}

/// Computes transaction velocity: total amount / time span.
fn compute_transaction_velocity(edges: &[(NaiveDate, EdgeId)], graph: &Graph) -> f64 {
    if edges.len() < 2 {
        return 0.0;
    }

    let first_date = edges.first().map(|(d, _)| *d);
    let last_date = edges.last().map(|(d, _)| *d);

    match (first_date, last_date) {
        (Some(first), Some(last)) => {
            let span_days = (last - first).num_days().max(1) as f64;
            let total_amount: f64 = edges
                .iter()
                .filter_map(|(_, edge_id)| graph.get_edge(*edge_id))
                .map(|e| e.weight)
                .sum();
            total_amount / span_days
        }
        _ => 0.0,
    }
}

/// Computes burst score using Kleinberg-style daily event counting.
fn compute_burst_score(edges: &[(NaiveDate, EdgeId)], threshold: f64) -> f64 {
    if edges.is_empty() {
        return 0.0;
    }

    // Count events per day
    let mut daily_counts: HashMap<NaiveDate, usize> = HashMap::new();
    for (date, _) in edges {
        *daily_counts.entry(*date).or_insert(0) += 1;
    }

    let counts: Vec<f64> = daily_counts.values().map(|&c| c as f64).collect();
    if counts.is_empty() {
        return 0.0;
    }

    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;
    let max_count = counts.iter().cloned().fold(0.0, f64::max);

    if mean_count < 0.001 {
        0.0
    } else {
        let ratio = max_count / mean_count;
        // Score is how much max exceeds threshold
        if ratio > threshold {
            (ratio - threshold).min(10.0) // Cap at 10 for stability
        } else {
            0.0
        }
    }
}

/// Computes trend direction using linear regression on amounts over time.
fn compute_trend_direction(edges: &[(NaiveDate, EdgeId)], graph: &Graph) -> f64 {
    if edges.len() < 3 {
        return 0.0;
    }

    let first_date = edges.first().map(|(d, _)| *d).unwrap();

    // Collect (days_since_start, amount) pairs
    let points: Vec<(f64, f64)> = edges
        .iter()
        .filter_map(|(date, edge_id)| {
            let edge = graph.get_edge(*edge_id)?;
            let x = (*date - first_date).num_days() as f64;
            Some((x, edge.weight))
        })
        .collect();

    if points.len() < 3 {
        return 0.0;
    }

    // Simple linear regression to find slope
    let n = points.len() as f64;
    let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
    let sum_xx: f64 = points.iter().map(|(x, _)| x * x).sum();

    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator.abs() < 1e-10 {
        return 0.0;
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;

    // Normalize slope direction
    if slope > 0.001 {
        1.0
    } else if slope < -0.001 {
        -1.0
    } else {
        0.0
    }
}

/// Computes seasonality score based on weekday activity variance.
fn compute_seasonality_score(edges: &[(NaiveDate, EdgeId)]) -> f64 {
    if edges.len() < 7 {
        return 0.0;
    }

    // Count events per weekday
    let mut weekday_counts = [0.0; 7];
    for (date, _) in edges {
        let weekday = date.weekday().num_days_from_monday() as usize;
        weekday_counts[weekday] += 1.0;
    }

    // Compute variance of weekday distribution
    let mean_count = weekday_counts.iter().sum::<f64>() / 7.0;
    let variance = weekday_counts
        .iter()
        .map(|&c| (c - mean_count).powi(2))
        .sum::<f64>()
        / 7.0;

    // Normalize by total count to get relative variance
    let total = edges.len() as f64;
    if total < 1.0 {
        0.0
    } else {
        // Coefficient of variation for weekday distribution
        (variance.sqrt() / mean_count.max(1.0)).min(1.0)
    }
}

/// Computes window-based aggregate features.
fn compute_window_features(
    node_id: NodeId,
    graph: &Graph,
    index: &TemporalIndex,
    reference_date: NaiveDate,
    window_days: i64,
) -> WindowFeatures {
    let start_date = reference_date - chrono::Duration::days(window_days);
    let edges = index.edges_in_range(node_id, start_date, reference_date);

    if edges.is_empty() {
        return WindowFeatures::default();
    }

    let mut total_amount = 0.0;
    let mut max_amount = 0.0;
    let mut counterparties = std::collections::HashSet::new();

    for (_, edge_id) in &edges {
        if let Some(edge) = graph.get_edge(*edge_id) {
            total_amount += edge.weight;
            if edge.weight > max_amount {
                max_amount = edge.weight;
            }
            // Track counterparty (the other end of the edge)
            let node = graph.get_node(node_id);
            if node.is_some() {
                if edge.source == node_id {
                    counterparties.insert(edge.target);
                } else {
                    counterparties.insert(edge.source);
                }
            }
        }
    }

    let event_count = edges.len();
    let avg_amount = if event_count > 0 {
        total_amount / event_count as f64
    } else {
        0.0
    };

    WindowFeatures {
        event_count,
        total_amount,
        avg_amount,
        max_amount,
        unique_counterparties: counterparties.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_temporal_test_graph;

    #[test]
    fn test_temporal_index_build() {
        let graph = create_temporal_test_graph();
        let index = TemporalIndex::build(&graph);

        assert!(index.min_date.is_some());
        assert!(index.max_date.is_some());
        assert_eq!(
            index.min_date.unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        assert_eq!(
            index.max_date.unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()
        );
    }

    #[test]
    fn test_edges_in_range() {
        let graph = create_temporal_test_graph();
        let index = TemporalIndex::build(&graph);

        // Node 1 should have many edges
        let start = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();
        let edges = index.edges_in_range(1, start, end);

        // Should have edges on days 3, 4, 5, 6, 7 (5 days)
        // Node 1 has 2 edges on even days (to n2 and n3) and 1 on odd days (to n2)
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_compute_temporal_features() {
        let graph = create_temporal_test_graph();
        let index = TemporalIndex::build(&graph);
        let config = TemporalConfig::default();

        let features = compute_temporal_sequence_features(1, &graph, &index, &config);

        // Node 1 should have positive velocity
        assert!(features.transaction_velocity > 0.0);

        // Should have positive trend (amounts increase over time)
        assert!(features.trend_direction >= 0.0);

        // Should have window features
        assert!(!features.window_features.is_empty());
    }

    #[test]
    fn test_inter_event_intervals() {
        let edges = vec![
            (NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 1),
            (NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 2),
            (NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(), 3),
        ];

        let (mean, std) = compute_inter_event_intervals(&edges);

        // Intervals are 2 and 3 days, mean = 2.5
        assert!((mean - 2.5).abs() < 0.01);
        assert!(std > 0.0);
    }

    #[test]
    fn test_burst_score() {
        // Create edges with a burst on one day
        let mut edges = Vec::new();
        for i in 0..3 {
            // Normal days with 1 event each
            edges.push((NaiveDate::from_ymd_opt(2024, 1, 1 + i).unwrap(), i as u64));
        }
        // Burst day with 10 events
        for i in 0..10 {
            edges.push((NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(), 100 + i));
        }

        let score = compute_burst_score(&edges, 3.0);

        // Should detect burst
        assert!(score > 0.0);
    }

    #[test]
    fn test_feature_vector_length() {
        let windows = vec![7, 30, 90];
        let expected_len = TemporalFeatures::feature_count(windows.len());

        let features = TemporalFeatures::default();
        let vec = features.to_features(&windows);

        assert_eq!(vec.len(), expected_len);
    }

    #[test]
    fn test_compute_all_temporal_features() {
        let graph = create_temporal_test_graph();
        let config = TemporalConfig::default();

        let all_features = compute_all_temporal_features(&graph, &config);

        // Should have features for all nodes
        assert_eq!(all_features.len(), 3);
    }
}
