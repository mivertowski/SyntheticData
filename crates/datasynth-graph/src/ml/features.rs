//! Feature engineering utilities for graph neural networks.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use crate::models::{Graph, NodeId};

/// Feature normalization method.
#[derive(Debug, Clone)]
pub enum NormalizationMethod {
    /// No normalization.
    None,
    /// Min-max normalization to [0, 1].
    MinMax,
    /// Z-score standardization.
    ZScore,
    /// Log transformation (log1p).
    Log,
    /// Robust scaling (using median and IQR).
    Robust,
}

/// Feature normalizer for graph features.
pub struct FeatureNormalizer {
    method: NormalizationMethod,
    /// Statistics per feature dimension.
    stats: Vec<FeatureStats>,
}

/// Statistics for a single feature dimension.
#[derive(Debug, Clone, Default)]
struct FeatureStats {
    min: f64,
    max: f64,
    mean: f64,
    std: f64,
    median: f64,
    q1: f64,
    q3: f64,
}

impl FeatureNormalizer {
    /// Creates a new feature normalizer.
    pub fn new(method: NormalizationMethod) -> Self {
        Self {
            method,
            stats: Vec::new(),
        }
    }

    /// Fits the normalizer to node features.
    pub fn fit_nodes(&mut self, graph: &Graph) {
        let features = graph.node_features();
        self.fit(&features);
    }

    /// Fits the normalizer to edge features.
    pub fn fit_edges(&mut self, graph: &Graph) {
        let features = graph.edge_features();
        self.fit(&features);
    }

    /// Fits the normalizer to features.
    fn fit(&mut self, features: &[Vec<f64>]) {
        if features.is_empty() {
            return;
        }

        let dim = features[0].len();
        self.stats = (0..dim)
            .map(|d| {
                let values: Vec<f64> = features
                    .iter()
                    .map(|f| f.get(d).copied().unwrap_or(0.0))
                    .collect();
                Self::compute_stats(&values)
            })
            .collect();
    }

    /// Computes statistics for a feature dimension.
    fn compute_stats(values: &[f64]) -> FeatureStats {
        if values.is_empty() {
            return FeatureStats::default();
        }

        let n = values.len() as f64;
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let mean = sum / n;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();

        // Compute quartiles
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let median = if sorted.len().is_multiple_of(2) {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let q1_idx = sorted.len() / 4;
        let q3_idx = 3 * sorted.len() / 4;
        let q1 = sorted.get(q1_idx).copied().unwrap_or(min);
        let q3 = sorted.get(q3_idx).copied().unwrap_or(max);

        FeatureStats {
            min,
            max,
            mean,
            std,
            median,
            q1,
            q3,
        }
    }

    /// Transforms features using fitted statistics.
    pub fn transform(&self, features: &[Vec<f64>]) -> Vec<Vec<f64>> {
        features.iter().map(|f| self.transform_single(f)).collect()
    }

    /// Transforms a single feature vector.
    fn transform_single(&self, features: &[f64]) -> Vec<f64> {
        features
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let stats = self.stats.get(i).cloned().unwrap_or_default();
                self.normalize_value(x, &stats)
            })
            .collect()
    }

    /// Normalizes a single value.
    fn normalize_value(&self, x: f64, stats: &FeatureStats) -> f64 {
        match self.method {
            NormalizationMethod::None => x,
            NormalizationMethod::MinMax => {
                let range = stats.max - stats.min;
                if range.abs() < 1e-10 {
                    0.0
                } else {
                    (x - stats.min) / range
                }
            }
            NormalizationMethod::ZScore => {
                if stats.std.abs() < 1e-10 {
                    0.0
                } else {
                    (x - stats.mean) / stats.std
                }
            }
            NormalizationMethod::Log => (x.abs() + 1.0).ln() * x.signum(),
            NormalizationMethod::Robust => {
                let iqr = stats.q3 - stats.q1;
                if iqr.abs() < 1e-10 {
                    0.0
                } else {
                    (x - stats.median) / iqr
                }
            }
        }
    }
}

/// Computes structural features for nodes.
pub fn compute_structural_features(graph: &Graph) -> HashMap<NodeId, Vec<f64>> {
    let mut features = HashMap::new();

    for &node_id in graph.nodes.keys() {
        let mut node_features = Vec::new();

        // Degree features
        let in_degree = graph.in_degree(node_id) as f64;
        let out_degree = graph.out_degree(node_id) as f64;
        let total_degree = in_degree + out_degree;

        node_features.push(in_degree);
        node_features.push(out_degree);
        node_features.push(total_degree);

        // Log degree (common in GNNs)
        node_features.push((in_degree + 1.0).ln());
        node_features.push((out_degree + 1.0).ln());

        // Degree ratio
        if total_degree > 0.0 {
            node_features.push(in_degree / total_degree);
            node_features.push(out_degree / total_degree);
        } else {
            node_features.push(0.5);
            node_features.push(0.5);
        }

        // Weight-based features (sum of incident edge weights)
        let in_weight: f64 = graph.incoming_edges(node_id).iter().map(|e| e.weight).sum();
        let out_weight: f64 = graph.outgoing_edges(node_id).iter().map(|e| e.weight).sum();

        node_features.push((in_weight + 1.0).ln());
        node_features.push((out_weight + 1.0).ln());

        // Average edge weight
        if in_degree > 0.0 {
            node_features.push(in_weight / in_degree);
        } else {
            node_features.push(0.0);
        }
        if out_degree > 0.0 {
            node_features.push(out_weight / out_degree);
        } else {
            node_features.push(0.0);
        }

        // Local clustering coefficient (simplified)
        let neighbors = graph.neighbors(node_id);
        let k = neighbors.len();
        if k > 1 {
            let mut triangle_count = 0;
            for i in 0..k {
                for j in i + 1..k {
                    if graph.neighbors(neighbors[i]).contains(&neighbors[j]) {
                        triangle_count += 1;
                    }
                }
            }
            let max_triangles = k * (k - 1) / 2;
            node_features.push(triangle_count as f64 / max_triangles as f64);
        } else {
            node_features.push(0.0);
        }

        features.insert(node_id, node_features);
    }

    features
}

/// Computes temporal features for edges.
pub fn compute_temporal_features(date: NaiveDate) -> Vec<f64> {
    let mut features = Vec::new();

    // Day of week (0-6)
    let weekday = date.weekday().num_days_from_monday() as f64;
    features.push(weekday / 6.0);

    // Day of month (1-31)
    let day = date.day() as f64;
    features.push(day / 31.0);

    // Month (1-12)
    let month = date.month() as f64;
    features.push(month / 12.0);

    // Quarter (1-4)
    let quarter = ((month - 1.0) / 3.0).floor() + 1.0;
    features.push(quarter / 4.0);

    // Is weekend
    features.push(if weekday >= 5.0 { 1.0 } else { 0.0 });

    // Is month end (last 3 days)
    features.push(if day >= 28.0 { 1.0 } else { 0.0 });

    // Is quarter end
    let is_quarter_end = matches!(month as u32, 3 | 6 | 9 | 12) && day >= 28.0;
    features.push(if is_quarter_end { 1.0 } else { 0.0 });

    // Is year end (December)
    features.push(if month == 12.0 { 1.0 } else { 0.0 });

    // Cyclical encoding for periodicity
    let day_of_year = date.ordinal() as f64;
    features.push((2.0 * std::f64::consts::PI * day_of_year / 365.0).sin());
    features.push((2.0 * std::f64::consts::PI * day_of_year / 365.0).cos());

    // Cyclical encoding for week
    features.push((2.0 * std::f64::consts::PI * weekday / 7.0).sin());
    features.push((2.0 * std::f64::consts::PI * weekday / 7.0).cos());

    features
}

/// Computes Benford's law features for an amount.
pub fn compute_benford_features(amount: f64) -> Vec<f64> {
    let mut features = Vec::new();

    // First digit
    let first_digit = extract_first_digit(amount);
    let benford_prob = benford_probability(first_digit);
    features.push(benford_prob);

    // Deviation from expected Benford distribution
    let expected_benford = [
        0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
    ];
    if (1..=9).contains(&first_digit) {
        let deviation = (expected_benford[first_digit as usize - 1] - benford_prob).abs();
        features.push(deviation);
    } else {
        features.push(0.0);
    }

    // First digit one-hot encoding
    for d in 1..=9 {
        features.push(if first_digit == d { 1.0 } else { 0.0 });
    }

    // Second digit (if available)
    let second_digit = extract_second_digit(amount);
    features.push(second_digit as f64 / 9.0);

    features
}

/// Extracts the first significant digit.
fn extract_first_digit(value: f64) -> u32 {
    if value == 0.0 {
        return 0;
    }
    let abs_val = value.abs();
    let log10 = abs_val.log10().floor();
    let normalized = abs_val / 10_f64.powf(log10);
    normalized.floor() as u32
}

/// Extracts the second significant digit.
fn extract_second_digit(value: f64) -> u32 {
    if value == 0.0 {
        return 0;
    }
    let abs_val = value.abs();
    let log10 = abs_val.log10().floor();
    let normalized = abs_val / 10_f64.powf(log10);
    ((normalized - normalized.floor()) * 10.0).floor() as u32
}

/// Returns the expected Benford's law probability for a digit.
fn benford_probability(digit: u32) -> f64 {
    if digit == 0 || digit > 9 {
        return 0.0;
    }
    (1.0 + 1.0 / digit as f64).log10()
}

/// Computes amount-based features.
pub fn compute_amount_features(amount: Decimal) -> Vec<f64> {
    let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
    let mut features = Vec::new();

    // Log amount
    features.push((amount_f64.abs() + 1.0).ln());

    // Sign
    features.push(if amount_f64 >= 0.0 { 1.0 } else { 0.0 });

    // Is round number
    let is_round = (amount_f64 % 100.0).abs() < 0.01;
    features.push(if is_round { 1.0 } else { 0.0 });

    // Magnitude bucket
    let magnitude = if amount_f64.abs() < 1.0 {
        0
    } else {
        (amount_f64.abs().log10().floor() as i32).clamp(0, 9)
    };
    for m in 0..10 {
        features.push(if magnitude == m { 1.0 } else { 0.0 });
    }

    // Benford features
    features.extend(compute_benford_features(amount_f64));

    features
}

/// One-hot encodes a categorical value.
pub fn one_hot_encode(value: &str, categories: &[&str]) -> Vec<f64> {
    let mut encoding = vec![0.0; categories.len()];
    if let Some(idx) = categories.iter().position(|&c| c == value) {
        encoding[idx] = 1.0;
    }
    encoding
}

/// Label encodes a categorical value.
pub fn label_encode(value: &str, categories: &[&str]) -> f64 {
    categories
        .iter()
        .position(|&c| c == value)
        .map(|i| i as f64)
        .unwrap_or(-1.0)
}

/// Creates positional encoding for graph nodes (similar to transformer positional encoding).
pub fn positional_encoding(position: usize, d_model: usize) -> Vec<f64> {
    let mut encoding = Vec::with_capacity(d_model);

    for i in 0..d_model {
        let angle = position as f64 / 10000_f64.powf(2.0 * (i / 2) as f64 / d_model as f64);
        if i % 2 == 0 {
            encoding.push(angle.sin());
        } else {
            encoding.push(angle.cos());
        }
    }

    encoding
}

/// Computes edge direction features for directed graphs.
pub fn compute_edge_direction_features(
    source_features: &[f64],
    target_features: &[f64],
) -> Vec<f64> {
    let mut features = Vec::new();

    // Feature differences
    for (s, t) in source_features.iter().zip(target_features.iter()) {
        features.push(t - s); // Direction: source -> target
    }

    // Absolute differences
    for (s, t) in source_features.iter().zip(target_features.iter()) {
        features.push((t - s).abs());
    }

    // Hadamard product
    for (s, t) in source_features.iter().zip(target_features.iter()) {
        features.push(s * t);
    }

    // Concatenation indicator (which node is "larger")
    let source_sum: f64 = source_features.iter().sum();
    let target_sum: f64 = target_features.iter().sum();
    features.push(if source_sum > target_sum { 1.0 } else { 0.0 });

    features
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_benford_probability() {
        let prob1 = benford_probability(1);
        assert!((prob1 - 0.301).abs() < 0.001);

        let prob9 = benford_probability(9);
        assert!((prob9 - 0.046).abs() < 0.001);
    }

    #[test]
    fn test_extract_first_digit() {
        assert_eq!(extract_first_digit(1234.56), 1);
        assert_eq!(extract_first_digit(9876.54), 9);
        assert_eq!(extract_first_digit(0.0123), 1);
    }

    #[test]
    fn test_temporal_features() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let features = compute_temporal_features(date);

        assert!(!features.is_empty());
        // Should indicate year end
        assert!(features[7] > 0.5); // is_year_end
    }

    #[test]
    fn test_normalization() {
        let features = vec![vec![1.0, 100.0], vec![2.0, 200.0], vec![3.0, 300.0]];

        let mut normalizer = FeatureNormalizer::new(NormalizationMethod::MinMax);
        normalizer.fit(&features);

        let normalized = normalizer.transform(&features);
        assert_eq!(normalized[0][0], 0.0); // Min
        assert_eq!(normalized[2][0], 1.0); // Max
    }

    #[test]
    fn test_one_hot_encode() {
        let categories = ["A", "B", "C"];
        let encoded = one_hot_encode("B", &categories);
        assert_eq!(encoded, vec![0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_positional_encoding() {
        let encoding = positional_encoding(0, 8);
        assert_eq!(encoding.len(), 8);
        assert_eq!(encoding[0], 0.0); // sin(0) = 0
    }
}
