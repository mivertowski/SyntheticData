//! PyTorch Geometric exporter.
//!
//! Exports graph data in formats compatible with PyTorch Geometric:
//! - NumPy arrays (.npy) for easy Python loading
//! - JSON metadata for graph information
//!
//! The exported data can be loaded in Python with:
//! ```python
//! import numpy as np
//! import torch
//! from torch_geometric.data import Data
//!
//! node_features = torch.from_numpy(np.load('node_features.npy'))
//! edge_index = torch.from_numpy(np.load('edge_index.npy'))
//! edge_attr = torch.from_numpy(np.load('edge_attr.npy'))
//! y = torch.from_numpy(np.load('labels.npy'))
//!
//! data = Data(x=node_features, edge_index=edge_index, edge_attr=edge_attr, y=y)
//! ```

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::exporters::common::{CommonExportConfig, CommonGraphMetadata};
use crate::exporters::npy_writer;
use crate::models::Graph;

/// Configuration for PyTorch Geometric export.
#[derive(Debug, Clone, Default)]
pub struct PyGExportConfig {
    /// Common export settings (features, labels, masks, splits, seed).
    pub common: CommonExportConfig,
    /// Export categorical features as one-hot.
    pub one_hot_categoricals: bool,
}

/// Metadata about the exported PyG data.
pub type PyGMetadata = CommonGraphMetadata;

/// PyTorch Geometric exporter.
pub struct PyGExporter {
    config: PyGExportConfig,
}

impl PyGExporter {
    /// Creates a new PyG exporter.
    pub fn new(config: PyGExportConfig) -> Self {
        Self { config }
    }

    /// Exports a graph to PyTorch Geometric format.
    pub fn export(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<PyGMetadata> {
        fs::create_dir_all(output_dir)?;

        let mut files = Vec::new();
        let mut statistics = HashMap::new();

        // Export edge index
        self.export_edge_index(graph, output_dir)?;
        files.push("edge_index.npy".to_string());

        // Export node features
        if self.config.common.export_node_features {
            let dim = self.export_node_features(graph, output_dir)?;
            files.push("node_features.npy".to_string());
            statistics.insert("node_feature_dim".to_string(), dim as f64);
        }

        // Export edge features
        if self.config.common.export_edge_features {
            let dim = self.export_edge_features(graph, output_dir)?;
            files.push("edge_features.npy".to_string());
            statistics.insert("edge_feature_dim".to_string(), dim as f64);
        }

        // Export node labels
        if self.config.common.export_node_labels {
            self.export_node_labels(graph, output_dir)?;
            files.push("node_labels.npy".to_string());
        }

        // Export edge labels
        if self.config.common.export_edge_labels {
            self.export_edge_labels(graph, output_dir)?;
            files.push("edge_labels.npy".to_string());
        }

        // Export masks
        if self.config.common.export_masks {
            self.export_masks(graph, output_dir)?;
            files.push("train_mask.npy".to_string());
            files.push("val_mask.npy".to_string());
            files.push("test_mask.npy".to_string());
        }

        // Compute node/edge type mappings
        let node_types: HashMap<String, usize> = graph
            .nodes_by_type
            .keys()
            .enumerate()
            .map(|(i, t)| (t.as_str().to_string(), i))
            .collect();

        let edge_types: HashMap<String, usize> = graph
            .edges_by_type
            .keys()
            .enumerate()
            .map(|(i, t)| (t.as_str().to_string(), i))
            .collect();

        // Compute statistics
        statistics.insert("density".to_string(), graph.metadata.density);
        statistics.insert(
            "anomalous_node_ratio".to_string(),
            graph.metadata.anomalous_node_count as f64 / graph.node_count().max(1) as f64,
        );
        statistics.insert(
            "anomalous_edge_ratio".to_string(),
            graph.metadata.anomalous_edge_count as f64 / graph.edge_count().max(1) as f64,
        );

        // Create metadata
        let metadata = PyGMetadata {
            name: graph.name.clone(),
            num_nodes: graph.node_count(),
            num_edges: graph.edge_count(),
            node_feature_dim: graph.metadata.node_feature_dim,
            edge_feature_dim: graph.metadata.edge_feature_dim,
            num_node_classes: 2, // Normal/Anomaly
            num_edge_classes: 2,
            node_types,
            edge_types,
            is_directed: true,
            files,
            statistics,
        };

        // Write metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(metadata_path)?;
        serde_json::to_writer_pretty(file, &metadata)?;

        // Write Python loader script
        self.write_loader_script(output_dir)?;

        Ok(metadata)
    }

    /// Exports edge index as [2, num_edges] array.
    fn export_edge_index(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        let (sources, targets) = graph.edge_index();

        // Create node ID to index mapping
        let mut node_ids: Vec<_> = graph.nodes.keys().copied().collect();
        node_ids.sort();
        let id_to_idx: HashMap<_, _> = node_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        // Remap edge indices, skipping edges with missing node IDs
        let mut sources_remapped: Vec<i64> = Vec::with_capacity(sources.len());
        let mut targets_remapped: Vec<i64> = Vec::with_capacity(targets.len());
        let mut skipped_edges = 0usize;

        for (src, dst) in sources.iter().zip(targets.iter()) {
            match (id_to_idx.get(src), id_to_idx.get(dst)) {
                (Some(&s), Some(&d)) => {
                    sources_remapped.push(s as i64);
                    targets_remapped.push(d as i64);
                }
                _ => {
                    skipped_edges += 1;
                }
            }
        }
        if skipped_edges > 0 {
            tracing::warn!(
                "PyTorch Geometric export: skipped {} edges with missing node IDs",
                skipped_edges
            );
        }

        // Write as NPY format
        let path = output_dir.join("edge_index.npy");
        npy_writer::write_npy_2d_i64(&path, &[sources_remapped, targets_remapped])?;

        Ok(())
    }

    /// Exports node features.
    fn export_node_features(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<usize> {
        let features = graph.node_features();
        let dim = features.first().map(|f| f.len()).unwrap_or(0);

        let path = output_dir.join("node_features.npy");
        npy_writer::write_npy_2d_f64(&path, &features)?;

        Ok(dim)
    }

    /// Exports edge features.
    fn export_edge_features(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<usize> {
        let features = graph.edge_features();
        let dim = features.first().map(|f| f.len()).unwrap_or(0);

        let path = output_dir.join("edge_features.npy");
        npy_writer::write_npy_2d_f64(&path, &features)?;

        Ok(dim)
    }

    /// Exports node labels (anomaly flags).
    fn export_node_labels(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        let labels: Vec<i64> = graph
            .node_anomaly_mask()
            .iter()
            .map(|&b| if b { 1 } else { 0 })
            .collect();

        let path = output_dir.join("node_labels.npy");
        npy_writer::write_npy_1d_i64(&path, &labels)?;

        Ok(())
    }

    /// Exports edge labels (anomaly flags).
    fn export_edge_labels(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        let labels: Vec<i64> = graph
            .edge_anomaly_mask()
            .iter()
            .map(|&b| if b { 1 } else { 0 })
            .collect();

        let path = output_dir.join("edge_labels.npy");
        npy_writer::write_npy_1d_i64(&path, &labels)?;

        Ok(())
    }

    /// Exports train/val/test masks.
    fn export_masks(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        npy_writer::export_masks(
            output_dir,
            graph.node_count(),
            self.config.common.seed,
            self.config.common.train_ratio,
            self.config.common.val_ratio,
        )
    }

    /// Writes a Python loader script.
    fn write_loader_script(&self, output_dir: &Path) -> std::io::Result<()> {
        let script = r#"#!/usr/bin/env python3
"""
PyTorch Geometric Data Loader

Auto-generated loader for graph data exported from synth-graph.
"""

import json
import numpy as np
import torch
from pathlib import Path

try:
    from torch_geometric.data import Data
    HAS_PYG = True
except ImportError:
    HAS_PYG = False
    print("Warning: torch_geometric not installed. Install with: pip install torch-geometric")


def load_graph(data_dir: str = ".") -> "Data":
    """Load graph data into a PyTorch Geometric Data object."""
    data_dir = Path(data_dir)

    # Load metadata
    with open(data_dir / "metadata.json") as f:
        metadata = json.load(f)

    # Load edge index
    edge_index = torch.from_numpy(np.load(data_dir / "edge_index.npy")).long()

    # Load node features (if available)
    x = None
    node_features_path = data_dir / "node_features.npy"
    if node_features_path.exists():
        x = torch.from_numpy(np.load(node_features_path)).float()

    # Load edge features (if available)
    edge_attr = None
    edge_features_path = data_dir / "edge_features.npy"
    if edge_features_path.exists():
        edge_attr = torch.from_numpy(np.load(edge_features_path)).float()

    # Load node labels (if available)
    y = None
    node_labels_path = data_dir / "node_labels.npy"
    if node_labels_path.exists():
        y = torch.from_numpy(np.load(node_labels_path)).long()

    # Load masks (if available)
    train_mask = None
    val_mask = None
    test_mask = None

    if (data_dir / "train_mask.npy").exists():
        train_mask = torch.from_numpy(np.load(data_dir / "train_mask.npy")).bool()
    if (data_dir / "val_mask.npy").exists():
        val_mask = torch.from_numpy(np.load(data_dir / "val_mask.npy")).bool()
    if (data_dir / "test_mask.npy").exists():
        test_mask = torch.from_numpy(np.load(data_dir / "test_mask.npy")).bool()

    if not HAS_PYG:
        return {
            "edge_index": edge_index,
            "x": x,
            "edge_attr": edge_attr,
            "y": y,
            "train_mask": train_mask,
            "val_mask": val_mask,
            "test_mask": test_mask,
            "metadata": metadata,
        }

    # Create PyG Data object
    data = Data(
        x=x,
        edge_index=edge_index,
        edge_attr=edge_attr,
        y=y,
        train_mask=train_mask,
        val_mask=val_mask,
        test_mask=test_mask,
    )

    # Store metadata
    data.metadata = metadata

    return data


def print_summary(data_dir: str = "."):
    """Print summary of the graph data."""
    data = load_graph(data_dir)

    if isinstance(data, dict):
        metadata = data["metadata"]
        print(f"Graph: {metadata['name']}")
        print(f"Nodes: {metadata['num_nodes']}")
        print(f"Edges: {metadata['num_edges']}")
        print(f"Node features: {data['x'].shape if data['x'] is not None else 'None'}")
        print(f"Edge features: {data['edge_attr'].shape if data['edge_attr'] is not None else 'None'}")
    else:
        print(f"Graph: {data.metadata['name']}")
        print(f"Nodes: {data.num_nodes}")
        print(f"Edges: {data.num_edges}")
        print(f"Node features: {data.x.shape if data.x is not None else 'None'}")
        print(f"Edge features: {data.edge_attr.shape if data.edge_attr is not None else 'None'}")
        if data.y is not None:
            print(f"Anomalous nodes: {data.y.sum().item()}")
        if data.train_mask is not None:
            print(f"Train/Val/Test: {data.train_mask.sum()}/{data.val_mask.sum()}/{data.test_mask.sum()}")


if __name__ == "__main__":
    import sys
    data_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    print_summary(data_dir)
"#;

        let path = output_dir.join("load_graph.py");
        let mut file = File::create(path)?;
        file.write_all(script.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_graph;
    use tempfile::tempdir;

    #[test]
    fn test_pyg_export() {
        let graph = create_test_graph();
        let dir = tempdir().unwrap();

        let exporter = PyGExporter::new(PyGExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.num_nodes, 2);
        assert_eq!(metadata.num_edges, 1);
        assert!(dir.path().join("edge_index.npy").exists());
        assert!(dir.path().join("node_features.npy").exists());
        assert!(dir.path().join("metadata.json").exists());
        assert!(dir.path().join("load_graph.py").exists());
    }
}
