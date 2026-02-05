//! Common types shared across ML graph exporters (PyG, DGL).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Common export configuration shared by PyG and DGL exporters.
#[derive(Debug, Clone)]
pub struct CommonExportConfig {
    /// Export node features.
    pub export_node_features: bool,
    /// Export edge features.
    pub export_edge_features: bool,
    /// Export node labels (anomaly flags).
    pub export_node_labels: bool,
    /// Export edge labels (anomaly flags).
    pub export_edge_labels: bool,
    /// Export train/val/test masks.
    pub export_masks: bool,
    /// Train split ratio.
    pub train_ratio: f64,
    /// Validation split ratio.
    pub val_ratio: f64,
    /// Random seed for splits.
    pub seed: u64,
}

impl Default for CommonExportConfig {
    fn default() -> Self {
        Self {
            export_node_features: true,
            export_edge_features: true,
            export_node_labels: true,
            export_edge_labels: true,
            export_masks: true,
            train_ratio: 0.7,
            val_ratio: 0.15,
            seed: 42,
        }
    }
}

/// Common metadata shared by PyG and DGL exporters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonGraphMetadata {
    /// Graph name.
    pub name: String,
    /// Number of nodes.
    pub num_nodes: usize,
    /// Number of edges.
    pub num_edges: usize,
    /// Node feature dimension.
    pub node_feature_dim: usize,
    /// Edge feature dimension.
    pub edge_feature_dim: usize,
    /// Number of node classes (for classification).
    pub num_node_classes: usize,
    /// Number of edge classes (for classification).
    pub num_edge_classes: usize,
    /// Node type mapping.
    pub node_types: HashMap<String, usize>,
    /// Edge type mapping.
    pub edge_types: HashMap<String, usize>,
    /// Whether graph is directed.
    pub is_directed: bool,
    /// Files included in export.
    pub files: Vec<String>,
    /// Additional statistics.
    pub statistics: HashMap<String, f64>,
}
