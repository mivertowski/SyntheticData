//! Train/validation/test split utilities for graph data.

use std::collections::HashSet;

use chrono::NaiveDate;

use crate::models::{EdgeId, Graph, NodeId};

/// Configuration for data splitting.
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// Train split ratio.
    pub train_ratio: f64,
    /// Validation split ratio.
    pub val_ratio: f64,
    /// Test split ratio (computed as 1 - train - val).
    pub random_seed: u64,
    /// Split strategy.
    pub strategy: SplitStrategy,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            train_ratio: 0.7,
            val_ratio: 0.15,
            random_seed: 42,
            strategy: SplitStrategy::Random,
        }
    }
}

/// Strategy for splitting data.
#[derive(Debug, Clone)]
pub enum SplitStrategy {
    /// Random split.
    Random,
    /// Temporal split (by timestamp).
    Temporal {
        /// Date field to use for splitting.
        train_cutoff: NaiveDate,
        val_cutoff: NaiveDate,
    },
    /// Stratified split (maintain class distribution).
    Stratified,
    /// K-fold cross validation.
    KFold { k: usize, fold: usize },
    /// Transductive split (nodes appear in all splits, but different edges).
    Transductive,
}

/// Result of a data split.
#[derive(Debug, Clone)]
pub struct DataSplit {
    /// Training node IDs.
    pub train_nodes: Vec<NodeId>,
    /// Validation node IDs.
    pub val_nodes: Vec<NodeId>,
    /// Test node IDs.
    pub test_nodes: Vec<NodeId>,
    /// Training edge IDs.
    pub train_edges: Vec<EdgeId>,
    /// Validation edge IDs.
    pub val_edges: Vec<EdgeId>,
    /// Test edge IDs.
    pub test_edges: Vec<EdgeId>,
}

impl DataSplit {
    /// Creates node masks for the graph.
    pub fn node_masks(&self, graph: &Graph) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
        let n = graph.node_count();
        let mut train_mask = vec![false; n];
        let mut val_mask = vec![false; n];
        let mut test_mask = vec![false; n];

        // Create ID to index mapping
        let mut node_ids: Vec<_> = graph.nodes.keys().copied().collect();
        node_ids.sort();
        let id_to_idx: std::collections::HashMap<_, _> = node_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        for &id in &self.train_nodes {
            if let Some(&idx) = id_to_idx.get(&id) {
                train_mask[idx] = true;
            }
        }
        for &id in &self.val_nodes {
            if let Some(&idx) = id_to_idx.get(&id) {
                val_mask[idx] = true;
            }
        }
        for &id in &self.test_nodes {
            if let Some(&idx) = id_to_idx.get(&id) {
                test_mask[idx] = true;
            }
        }

        (train_mask, val_mask, test_mask)
    }

    /// Creates edge masks for the graph.
    pub fn edge_masks(&self, graph: &Graph) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
        let m = graph.edge_count();
        let mut train_mask = vec![false; m];
        let mut val_mask = vec![false; m];
        let mut test_mask = vec![false; m];

        // Create ID to index mapping
        let mut edge_ids: Vec<_> = graph.edges.keys().copied().collect();
        edge_ids.sort();
        let id_to_idx: std::collections::HashMap<_, _> = edge_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        for &id in &self.train_edges {
            if let Some(&idx) = id_to_idx.get(&id) {
                train_mask[idx] = true;
            }
        }
        for &id in &self.val_edges {
            if let Some(&idx) = id_to_idx.get(&id) {
                val_mask[idx] = true;
            }
        }
        for &id in &self.test_edges {
            if let Some(&idx) = id_to_idx.get(&id) {
                test_mask[idx] = true;
            }
        }

        (train_mask, val_mask, test_mask)
    }
}

/// Data splitter for graph data.
pub struct DataSplitter {
    config: SplitConfig,
}

impl DataSplitter {
    /// Creates a new data splitter.
    pub fn new(config: SplitConfig) -> Self {
        Self { config }
    }

    /// Splits graph data according to configuration.
    pub fn split(&self, graph: &Graph) -> DataSplit {
        match &self.config.strategy {
            SplitStrategy::Random => self.random_split(graph),
            SplitStrategy::Temporal {
                train_cutoff,
                val_cutoff,
            } => self.temporal_split(graph, *train_cutoff, *val_cutoff),
            SplitStrategy::Stratified => self.stratified_split(graph),
            SplitStrategy::KFold { k, fold } => self.kfold_split(graph, *k, *fold),
            SplitStrategy::Transductive => self.transductive_split(graph),
        }
    }

    /// Performs random split.
    fn random_split(&self, graph: &Graph) -> DataSplit {
        let mut rng = SimpleRng::new(self.config.random_seed);

        // Split nodes
        let mut node_ids: Vec<_> = graph.nodes.keys().copied().collect();
        shuffle(&mut node_ids, &mut rng);

        let n = node_ids.len();
        let train_size = (n as f64 * self.config.train_ratio) as usize;
        let val_size = (n as f64 * self.config.val_ratio) as usize;

        let train_nodes: Vec<_> = node_ids[..train_size].to_vec();
        let val_nodes: Vec<_> = node_ids[train_size..train_size + val_size].to_vec();
        let test_nodes: Vec<_> = node_ids[train_size + val_size..].to_vec();

        // Split edges
        let mut edge_ids: Vec<_> = graph.edges.keys().copied().collect();
        shuffle(&mut edge_ids, &mut rng);

        let m = edge_ids.len();
        let train_edge_size = (m as f64 * self.config.train_ratio) as usize;
        let val_edge_size = (m as f64 * self.config.val_ratio) as usize;

        let train_edges: Vec<_> = edge_ids[..train_edge_size].to_vec();
        let val_edges: Vec<_> = edge_ids[train_edge_size..train_edge_size + val_edge_size].to_vec();
        let test_edges: Vec<_> = edge_ids[train_edge_size + val_edge_size..].to_vec();

        DataSplit {
            train_nodes,
            val_nodes,
            test_nodes,
            train_edges,
            val_edges,
            test_edges,
        }
    }

    /// Performs temporal split based on edge timestamps.
    fn temporal_split(
        &self,
        graph: &Graph,
        train_cutoff: NaiveDate,
        val_cutoff: NaiveDate,
    ) -> DataSplit {
        let mut train_edges = Vec::new();
        let mut val_edges = Vec::new();
        let mut test_edges = Vec::new();

        // Split edges by timestamp
        for (&edge_id, edge) in &graph.edges {
            if let Some(timestamp) = edge.timestamp {
                if timestamp < train_cutoff {
                    train_edges.push(edge_id);
                } else if timestamp < val_cutoff {
                    val_edges.push(edge_id);
                } else {
                    test_edges.push(edge_id);
                }
            } else {
                // No timestamp - assign randomly
                let r = edge_id % 100;
                if (r as f64) < self.config.train_ratio * 100.0 {
                    train_edges.push(edge_id);
                } else if (r as f64) < (self.config.train_ratio + self.config.val_ratio) * 100.0 {
                    val_edges.push(edge_id);
                } else {
                    test_edges.push(edge_id);
                }
            }
        }

        // Determine node splits based on when they first appear
        let _train_edge_set: HashSet<_> = train_edges.iter().copied().collect();
        let _val_edge_set: HashSet<_> = val_edges.iter().copied().collect();

        let mut train_nodes = HashSet::new();
        let mut val_nodes = HashSet::new();
        let mut test_nodes = HashSet::new();

        // Nodes that appear in training edges
        for &edge_id in &train_edges {
            if let Some(edge) = graph.edges.get(&edge_id) {
                train_nodes.insert(edge.source);
                train_nodes.insert(edge.target);
            }
        }

        // Nodes that first appear in validation edges
        for &edge_id in &val_edges {
            if let Some(edge) = graph.edges.get(&edge_id) {
                if !train_nodes.contains(&edge.source) {
                    val_nodes.insert(edge.source);
                }
                if !train_nodes.contains(&edge.target) {
                    val_nodes.insert(edge.target);
                }
            }
        }

        // Nodes that first appear in test edges
        for &edge_id in &test_edges {
            if let Some(edge) = graph.edges.get(&edge_id) {
                if !train_nodes.contains(&edge.source) && !val_nodes.contains(&edge.source) {
                    test_nodes.insert(edge.source);
                }
                if !train_nodes.contains(&edge.target) && !val_nodes.contains(&edge.target) {
                    test_nodes.insert(edge.target);
                }
            }
        }

        DataSplit {
            train_nodes: train_nodes.into_iter().collect(),
            val_nodes: val_nodes.into_iter().collect(),
            test_nodes: test_nodes.into_iter().collect(),
            train_edges,
            val_edges,
            test_edges,
        }
    }

    /// Performs stratified split maintaining anomaly distribution.
    fn stratified_split(&self, graph: &Graph) -> DataSplit {
        let mut rng = SimpleRng::new(self.config.random_seed);

        // Separate normal and anomalous nodes
        let mut normal_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|(_, n)| !n.is_anomaly)
            .map(|(&id, _)| id)
            .collect();
        let mut anomalous_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|(_, n)| n.is_anomaly)
            .map(|(&id, _)| id)
            .collect();

        shuffle(&mut normal_nodes, &mut rng);
        shuffle(&mut anomalous_nodes, &mut rng);

        // Split each class
        let (normal_train, normal_val, normal_test) = split_by_ratio(
            &normal_nodes,
            self.config.train_ratio,
            self.config.val_ratio,
        );
        let (anomaly_train, anomaly_val, anomaly_test) = split_by_ratio(
            &anomalous_nodes,
            self.config.train_ratio,
            self.config.val_ratio,
        );

        // Combine
        let mut train_nodes = normal_train;
        train_nodes.extend(anomaly_train);

        let mut val_nodes = normal_val;
        val_nodes.extend(anomaly_val);

        let mut test_nodes = normal_test;
        test_nodes.extend(anomaly_test);

        // Split edges similarly
        let mut normal_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|(_, e)| !e.is_anomaly)
            .map(|(&id, _)| id)
            .collect();
        let mut anomalous_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|(_, e)| e.is_anomaly)
            .map(|(&id, _)| id)
            .collect();

        shuffle(&mut normal_edges, &mut rng);
        shuffle(&mut anomalous_edges, &mut rng);

        let (normal_train_e, normal_val_e, normal_test_e) = split_by_ratio(
            &normal_edges,
            self.config.train_ratio,
            self.config.val_ratio,
        );
        let (anomaly_train_e, anomaly_val_e, anomaly_test_e) = split_by_ratio(
            &anomalous_edges,
            self.config.train_ratio,
            self.config.val_ratio,
        );

        let mut train_edges = normal_train_e;
        train_edges.extend(anomaly_train_e);

        let mut val_edges = normal_val_e;
        val_edges.extend(anomaly_val_e);

        let mut test_edges = normal_test_e;
        test_edges.extend(anomaly_test_e);

        DataSplit {
            train_nodes,
            val_nodes,
            test_nodes,
            train_edges,
            val_edges,
            test_edges,
        }
    }

    /// Performs k-fold cross validation split.
    fn kfold_split(&self, graph: &Graph, k: usize, fold: usize) -> DataSplit {
        let mut rng = SimpleRng::new(self.config.random_seed);

        let mut node_ids: Vec<_> = graph.nodes.keys().copied().collect();
        shuffle(&mut node_ids, &mut rng);

        let fold_size = node_ids.len() / k;
        let val_start = fold * fold_size;
        let val_end = if fold == k - 1 {
            node_ids.len()
        } else {
            (fold + 1) * fold_size
        };

        let val_nodes: Vec<_> = node_ids[val_start..val_end].to_vec();
        let train_nodes: Vec<_> = node_ids
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < val_start || *i >= val_end)
            .map(|(_, &id)| id)
            .collect();

        // Similarly for edges
        let mut edge_ids: Vec<_> = graph.edges.keys().copied().collect();
        shuffle(&mut edge_ids, &mut rng);

        let edge_fold_size = edge_ids.len() / k;
        let edge_val_start = fold * edge_fold_size;
        let edge_val_end = if fold == k - 1 {
            edge_ids.len()
        } else {
            (fold + 1) * edge_fold_size
        };

        let val_edges: Vec<_> = edge_ids[edge_val_start..edge_val_end].to_vec();
        let train_edges: Vec<_> = edge_ids
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < edge_val_start || *i >= edge_val_end)
            .map(|(_, &id)| id)
            .collect();

        DataSplit {
            train_nodes,
            val_nodes: val_nodes.clone(),
            test_nodes: val_nodes, // In k-fold, val and test are the same
            train_edges,
            val_edges: val_edges.clone(),
            test_edges: val_edges,
        }
    }

    /// Performs transductive split (all nodes available, edges split).
    fn transductive_split(&self, graph: &Graph) -> DataSplit {
        let mut rng = SimpleRng::new(self.config.random_seed);

        // All nodes in all splits
        let all_nodes: Vec<_> = graph.nodes.keys().copied().collect();

        // Split edges only
        let mut edge_ids: Vec<_> = graph.edges.keys().copied().collect();
        shuffle(&mut edge_ids, &mut rng);

        let m = edge_ids.len();
        let train_size = (m as f64 * self.config.train_ratio) as usize;
        let val_size = (m as f64 * self.config.val_ratio) as usize;

        let train_edges: Vec<_> = edge_ids[..train_size].to_vec();
        let val_edges: Vec<_> = edge_ids[train_size..train_size + val_size].to_vec();
        let test_edges: Vec<_> = edge_ids[train_size + val_size..].to_vec();

        DataSplit {
            train_nodes: all_nodes.clone(),
            val_nodes: all_nodes.clone(),
            test_nodes: all_nodes,
            train_edges,
            val_edges,
            test_edges,
        }
    }
}

/// Splits a slice by ratio.
fn split_by_ratio<T: Clone>(
    items: &[T],
    train_ratio: f64,
    val_ratio: f64,
) -> (Vec<T>, Vec<T>, Vec<T>) {
    let n = items.len();
    let train_size = (n as f64 * train_ratio) as usize;
    let val_size = (n as f64 * val_ratio) as usize;

    let train = items[..train_size].to_vec();
    let val = items[train_size..train_size + val_size].to_vec();
    let test = items[train_size + val_size..].to_vec();

    (train, val, test)
}

/// Simple random number generator (xorshift64).
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// Fisher-Yates shuffle.
fn shuffle<T>(items: &mut [T], rng: &mut SimpleRng) {
    for i in (1..items.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// Creates negative edge samples for link prediction.
pub fn sample_negative_edges(
    graph: &Graph,
    num_samples: usize,
    seed: u64,
) -> Vec<(NodeId, NodeId)> {
    let mut rng = SimpleRng::new(seed);
    let node_ids: Vec<_> = graph.nodes.keys().copied().collect();
    let n = node_ids.len();

    if n < 2 {
        return Vec::new();
    }

    // Build existing edge set
    let existing_edges: HashSet<_> = graph
        .edges
        .values()
        .map(|e| (e.source.min(e.target), e.source.max(e.target)))
        .collect();

    let mut negative_edges = Vec::with_capacity(num_samples);
    let max_attempts = num_samples * 10;
    let mut attempts = 0;

    while negative_edges.len() < num_samples && attempts < max_attempts {
        let i = (rng.next() % n as u64) as usize;
        let j = (rng.next() % n as u64) as usize;

        if i == j {
            attempts += 1;
            continue;
        }

        let src = node_ids[i];
        let tgt = node_ids[j];
        let key = (src.min(tgt), src.max(tgt));

        if !existing_edges.contains(&key) {
            negative_edges.push((src, tgt));
        }

        attempts += 1;
    }

    negative_edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_splits_test_graph;

    #[test]
    fn test_random_split() {
        let graph = create_splits_test_graph();
        let splitter = DataSplitter::new(SplitConfig::default());
        let split = splitter.split(&graph);

        assert_eq!(
            split.train_nodes.len() + split.val_nodes.len() + split.test_nodes.len(),
            graph.node_count()
        );
    }

    #[test]
    fn test_temporal_split() {
        let graph = create_splits_test_graph();
        let config = SplitConfig {
            strategy: SplitStrategy::Temporal {
                train_cutoff: chrono::NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
                val_cutoff: chrono::NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            },
            ..Default::default()
        };
        let splitter = DataSplitter::new(config);
        let split = splitter.split(&graph);

        // Train edges should be before cutoff
        assert!(!split.train_edges.is_empty());
    }

    #[test]
    fn test_stratified_split() {
        let graph = create_splits_test_graph();
        let config = SplitConfig {
            strategy: SplitStrategy::Stratified,
            ..Default::default()
        };
        let splitter = DataSplitter::new(config);
        let split = splitter.split(&graph);

        assert!(!split.train_nodes.is_empty());
    }

    #[test]
    fn test_negative_sampling() {
        let graph = create_splits_test_graph();
        let negatives = sample_negative_edges(&graph, 5, 42);

        assert!(negatives.len() <= 5);
        for (src, tgt) in &negatives {
            assert_ne!(src, tgt);
        }
    }
}
