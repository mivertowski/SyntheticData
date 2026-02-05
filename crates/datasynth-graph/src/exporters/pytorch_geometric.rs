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
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::exporters::common::{CommonExportConfig, CommonGraphMetadata};
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

        // Remap edge indices
        let sources_remapped: Vec<i64> = sources
            .iter()
            .map(|id| *id_to_idx.get(id).unwrap_or(&0) as i64)
            .collect();
        let targets_remapped: Vec<i64> = targets
            .iter()
            .map(|id| *id_to_idx.get(id).unwrap_or(&0) as i64)
            .collect();

        // Write as NPY format
        let path = output_dir.join("edge_index.npy");
        self.write_npy_2d_i64(&path, &[sources_remapped, targets_remapped])?;

        Ok(())
    }

    /// Exports node features.
    fn export_node_features(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<usize> {
        let features = graph.node_features();
        let dim = features.first().map(|f| f.len()).unwrap_or(0);

        let path = output_dir.join("node_features.npy");
        self.write_npy_2d_f64(&path, &features)?;

        Ok(dim)
    }

    /// Exports edge features.
    fn export_edge_features(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<usize> {
        let features = graph.edge_features();
        let dim = features.first().map(|f| f.len()).unwrap_or(0);

        let path = output_dir.join("edge_features.npy");
        self.write_npy_2d_f64(&path, &features)?;

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
        self.write_npy_1d_i64(&path, &labels)?;

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
        self.write_npy_1d_i64(&path, &labels)?;

        Ok(())
    }

    /// Exports train/val/test masks.
    fn export_masks(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        let n = graph.node_count();
        let mut rng = SimpleRng::new(self.config.common.seed);

        let train_size = (n as f64 * self.config.common.train_ratio) as usize;
        let val_size = (n as f64 * self.config.common.val_ratio) as usize;

        // Create shuffled indices
        let mut indices: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (rng.next() % (i as u64 + 1)) as usize;
            indices.swap(i, j);
        }

        // Create masks
        let mut train_mask = vec![false; n];
        let mut val_mask = vec![false; n];
        let mut test_mask = vec![false; n];

        for (i, &idx) in indices.iter().enumerate() {
            if i < train_size {
                train_mask[idx] = true;
            } else if i < train_size + val_size {
                val_mask[idx] = true;
            } else {
                test_mask[idx] = true;
            }
        }

        // Write masks
        self.write_npy_1d_bool(&output_dir.join("train_mask.npy"), &train_mask)?;
        self.write_npy_1d_bool(&output_dir.join("val_mask.npy"), &val_mask)?;
        self.write_npy_1d_bool(&output_dir.join("test_mask.npy"), &test_mask)?;

        Ok(())
    }

    /// Writes a 1D array of i64 in NPY format.
    fn write_npy_1d_i64(&self, path: &Path, data: &[i64]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // NPY header
        let shape = format!("({},)", data.len());
        self.write_npy_header(&mut writer, "<i8", &shape)?;

        // Data
        for &val in data {
            writer.write_all(&val.to_le_bytes())?;
        }

        Ok(())
    }

    /// Writes a 1D array of bool in NPY format.
    fn write_npy_1d_bool(&self, path: &Path, data: &[bool]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // NPY header
        let shape = format!("({},)", data.len());
        self.write_npy_header(&mut writer, "|b1", &shape)?;

        // Data
        for &val in data {
            writer.write_all(&[if val { 1u8 } else { 0u8 }])?;
        }

        Ok(())
    }

    /// Writes a 2D array of i64 in NPY format.
    fn write_npy_2d_i64(&self, path: &Path, data: &[Vec<i64>]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let rows = data.len();
        let cols = data.first().map(|r| r.len()).unwrap_or(0);

        // NPY header
        let shape = format!("({}, {})", rows, cols);
        self.write_npy_header(&mut writer, "<i8", &shape)?;

        // Data (row-major)
        for row in data {
            for &val in row {
                writer.write_all(&val.to_le_bytes())?;
            }
        }

        Ok(())
    }

    /// Writes a 2D array of f64 in NPY format.
    fn write_npy_2d_f64(&self, path: &Path, data: &[Vec<f64>]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let rows = data.len();
        let cols = data.first().map(|r| r.len()).unwrap_or(0);

        // NPY header
        let shape = format!("({}, {})", rows, cols);
        self.write_npy_header(&mut writer, "<f8", &shape)?;

        // Data (row-major)
        for row in data {
            for &val in row {
                writer.write_all(&val.to_le_bytes())?;
            }
            // Pad short rows with zeros
            for _ in row.len()..cols {
                writer.write_all(&0.0_f64.to_le_bytes())?;
            }
        }

        Ok(())
    }

    /// Writes NPY header.
    fn write_npy_header<W: Write>(
        &self,
        writer: &mut W,
        dtype: &str,
        shape: &str,
    ) -> std::io::Result<()> {
        // Magic number and version
        writer.write_all(&[0x93])?; // \x93
        writer.write_all(b"NUMPY")?;
        writer.write_all(&[0x01, 0x00])?; // Version 1.0

        // Header dict
        let header = format!(
            "{{'descr': '{}', 'fortran_order': False, 'shape': {} }}",
            dtype, shape
        );

        // Pad header to multiple of 64 bytes (including magic, version, header_len)
        let header_len = header.len();
        let total_len = 10 + header_len + 1; // magic(6) + version(2) + header_len(2) + header + newline
        let padding = (64 - (total_len % 64)) % 64;
        let padded_len = header_len + 1 + padding;

        writer.write_all(&(padded_len as u16).to_le_bytes())?;
        writer.write_all(header.as_bytes())?;
        for _ in 0..padding {
            writer.write_all(b" ")?;
        }
        writer.write_all(b"\n")?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EdgeType, GraphEdge, GraphNode, GraphType, NodeType};
    use tempfile::tempdir;

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new("test", GraphType::Transaction);

        let n1 = graph.add_node(
            GraphNode::new(0, NodeType::Account, "1000".to_string(), "Cash".to_string())
                .with_feature(0.5),
        );
        let n2 = graph.add_node(
            GraphNode::new(0, NodeType::Account, "2000".to_string(), "AP".to_string())
                .with_feature(0.8),
        );

        graph.add_edge(
            GraphEdge::new(0, n1, n2, EdgeType::Transaction)
                .with_weight(1000.0)
                .with_feature(6.9),
        );

        graph.compute_statistics();
        graph
    }

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
