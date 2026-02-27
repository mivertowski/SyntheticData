//! Deep Graph Library (DGL) exporter.
//!
//! Exports graph data in formats compatible with DGL:
//! - NumPy arrays (.npy) for node/edge features and labels
//! - COO format edge index [num_edges, 2] (differs from PyG's [2, num_edges])
//! - JSON metadata for graph information
//!
//! The exported data can be loaded in Python with:
//! ```python
//! import numpy as np
//! import torch
//! import dgl
//!
//! node_features = torch.from_numpy(np.load('node_features.npy'))
//! edge_index = np.load('edge_index.npy')  # [num_edges, 2] COO format
//! src, dst = edge_index[:, 0], edge_index[:, 1]
//!
//! g = dgl.graph((src, dst))
//! g.ndata['feat'] = node_features
//! ```
//!
//! For heterogeneous graphs, DGL uses separate arrays per node/edge type.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::exporters::common::{CommonExportConfig, CommonGraphMetadata};
use crate::exporters::npy_writer;
use crate::models::Graph;

/// Configuration for DGL export.
#[derive(Debug, Clone)]
pub struct DGLExportConfig {
    /// Common export settings (features, labels, masks, splits, seed).
    pub common: CommonExportConfig,
    /// Export as heterogeneous graph (separate files per type).
    pub heterogeneous: bool,
    /// Include Python pickle helper script.
    pub include_pickle_script: bool,
}

impl Default for DGLExportConfig {
    fn default() -> Self {
        Self {
            common: CommonExportConfig::default(),
            heterogeneous: false,
            include_pickle_script: true,
        }
    }
}

/// Metadata about the exported DGL data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DGLMetadata {
    /// Common graph metadata fields.
    #[serde(flatten)]
    pub common: CommonGraphMetadata,
    /// Whether export is heterogeneous.
    pub is_heterogeneous: bool,
    /// Edge index format (COO).
    pub edge_format: String,
}

/// DGL graph exporter.
pub struct DGLExporter {
    config: DGLExportConfig,
}

impl DGLExporter {
    /// Creates a new DGL exporter.
    pub fn new(config: DGLExportConfig) -> Self {
        Self { config }
    }

    /// Exports a graph to DGL format.
    pub fn export(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<DGLMetadata> {
        fs::create_dir_all(output_dir)?;

        let mut files = Vec::new();
        let mut statistics = HashMap::new();

        // Export edge index in COO format [num_edges, 2]
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

        // Export node type indices (for heterogeneous support)
        if self.config.heterogeneous {
            self.export_node_types(graph, output_dir)?;
            files.push("node_type_indices.npy".to_string());
            self.export_edge_types(graph, output_dir)?;
            files.push("edge_type_indices.npy".to_string());
        }

        // Compute node/edge type mappings with counts
        let node_types: HashMap<String, usize> = graph
            .nodes_by_type
            .iter()
            .map(|(t, ids)| (t.as_str().to_string(), ids.len()))
            .collect();

        let edge_types: HashMap<String, usize> = graph
            .edges_by_type
            .iter()
            .map(|(t, ids)| (t.as_str().to_string(), ids.len()))
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
        let metadata = DGLMetadata {
            common: CommonGraphMetadata {
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
            },
            is_heterogeneous: self.config.heterogeneous,
            edge_format: "COO".to_string(),
        };

        // Write metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(metadata_path)?;
        serde_json::to_writer_pretty(file, &metadata)?;

        // Write Python loader script
        self.write_loader_script(output_dir)?;

        // Write pickle helper script if configured
        if self.config.include_pickle_script {
            self.write_pickle_script(output_dir)?;
        }

        Ok(metadata)
    }

    /// Exports edge index as COO format [num_edges, 2] array.
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

        // Create COO format: [num_edges, 2] where each row is (src, dst)
        let num_edges = sources.len();
        let mut coo_data: Vec<Vec<i64>> = Vec::with_capacity(num_edges);
        let mut skipped_edges = 0usize;

        for i in 0..num_edges {
            match (id_to_idx.get(&sources[i]), id_to_idx.get(&targets[i])) {
                (Some(&s), Some(&d)) => {
                    coo_data.push(vec![s as i64, d as i64]);
                }
                _ => {
                    skipped_edges += 1;
                }
            }
        }
        if skipped_edges > 0 {
            tracing::warn!(
                "DGL export: skipped {} edges with missing node IDs",
                skipped_edges
            );
        }

        // Write as NPY format [num_edges, 2]
        let path = output_dir.join("edge_index.npy");
        npy_writer::write_npy_2d_i64(&path, &coo_data)?;

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

    /// Exports node type indices for heterogeneous graphs.
    fn export_node_types(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        // Create type mapping
        let type_to_idx: HashMap<_, _> = graph
            .nodes_by_type
            .keys()
            .enumerate()
            .map(|(i, t)| (t.clone(), i as i64))
            .collect();

        // Get node IDs in sorted order for consistent indexing
        let mut node_ids: Vec<_> = graph.nodes.keys().copied().collect();
        node_ids.sort();

        // Map each node to its type index
        let type_indices: Vec<i64> = node_ids
            .iter()
            .map(|id| {
                let node = graph.nodes.get(id).expect("node ID from keys()");
                *type_to_idx.get(&node.node_type).unwrap_or_else(|| {
                    tracing::warn!(
                        "Unknown node type '{:?}', defaulting to index 0",
                        node.node_type
                    );
                    &0
                })
            })
            .collect();

        let path = output_dir.join("node_type_indices.npy");
        npy_writer::write_npy_1d_i64(&path, &type_indices)?;

        Ok(())
    }

    /// Exports edge type indices for heterogeneous graphs.
    fn export_edge_types(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<()> {
        // Create type mapping
        let type_to_idx: HashMap<_, _> = graph
            .edges_by_type
            .keys()
            .enumerate()
            .map(|(i, t)| (t.clone(), i as i64))
            .collect();

        // Get edge IDs in sorted order for consistent indexing
        let mut edge_ids: Vec<_> = graph.edges.keys().copied().collect();
        edge_ids.sort();

        // Map each edge to its type index
        let type_indices: Vec<i64> = edge_ids
            .iter()
            .map(|id| {
                let edge = graph.edges.get(id).expect("edge ID from keys()");
                *type_to_idx.get(&edge.edge_type).unwrap_or_else(|| {
                    tracing::warn!(
                        "Unknown edge type '{:?}', defaulting to index 0",
                        edge.edge_type
                    );
                    &0
                })
            })
            .collect();

        let path = output_dir.join("edge_type_indices.npy");
        npy_writer::write_npy_1d_i64(&path, &type_indices)?;

        Ok(())
    }

    /// Writes a Python loader script for DGL.
    fn write_loader_script(&self, output_dir: &Path) -> std::io::Result<()> {
        let script = r#"#!/usr/bin/env python3
"""
DGL (Deep Graph Library) Data Loader

Auto-generated loader for graph data exported from synth-graph.
Supports both homogeneous and heterogeneous graph loading.
"""

import json
import numpy as np
from pathlib import Path

try:
    import torch
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    print("Warning: torch not installed. Install with: pip install torch")

try:
    import dgl
    HAS_DGL = True
except ImportError:
    HAS_DGL = False
    print("Warning: dgl not installed. Install with: pip install dgl")


def load_graph(data_dir: str = ".") -> "dgl.DGLGraph":
    """Load graph data into a DGL graph object.

    Args:
        data_dir: Directory containing the exported graph data.

    Returns:
        DGL graph with node features, edge features, and labels attached.
    """
    data_dir = Path(data_dir)

    # Load metadata
    with open(data_dir / "metadata.json") as f:
        metadata = json.load(f)

    # Load edge index (COO format: [num_edges, 2])
    edge_index = np.load(data_dir / "edge_index.npy")
    src = edge_index[:, 0]
    dst = edge_index[:, 1]

    num_nodes = metadata["num_nodes"]

    if not HAS_DGL:
        # Return dict if DGL not available
        result = {
            "src": src,
            "dst": dst,
            "num_nodes": num_nodes,
            "metadata": metadata,
        }

        # Load optional arrays
        if (data_dir / "node_features.npy").exists():
            result["node_features"] = np.load(data_dir / "node_features.npy")
        if (data_dir / "edge_features.npy").exists():
            result["edge_features"] = np.load(data_dir / "edge_features.npy")
        if (data_dir / "node_labels.npy").exists():
            result["node_labels"] = np.load(data_dir / "node_labels.npy")
        if (data_dir / "edge_labels.npy").exists():
            result["edge_labels"] = np.load(data_dir / "edge_labels.npy")
        if (data_dir / "train_mask.npy").exists():
            result["train_mask"] = np.load(data_dir / "train_mask.npy")
            result["val_mask"] = np.load(data_dir / "val_mask.npy")
            result["test_mask"] = np.load(data_dir / "test_mask.npy")

        return result

    # Create DGL graph
    g = dgl.graph((src, dst), num_nodes=num_nodes)

    # Load and attach node features
    node_features_path = data_dir / "node_features.npy"
    if node_features_path.exists():
        node_features = np.load(node_features_path)
        if HAS_TORCH:
            g.ndata['feat'] = torch.from_numpy(node_features).float()
        else:
            g.ndata['feat'] = node_features

    # Load and attach edge features
    edge_features_path = data_dir / "edge_features.npy"
    if edge_features_path.exists():
        edge_features = np.load(edge_features_path)
        if HAS_TORCH:
            g.edata['feat'] = torch.from_numpy(edge_features).float()
        else:
            g.edata['feat'] = edge_features

    # Load and attach node labels
    node_labels_path = data_dir / "node_labels.npy"
    if node_labels_path.exists():
        node_labels = np.load(node_labels_path)
        if HAS_TORCH:
            g.ndata['label'] = torch.from_numpy(node_labels).long()
        else:
            g.ndata['label'] = node_labels

    # Load and attach edge labels
    edge_labels_path = data_dir / "edge_labels.npy"
    if edge_labels_path.exists():
        edge_labels = np.load(edge_labels_path)
        if HAS_TORCH:
            g.edata['label'] = torch.from_numpy(edge_labels).long()
        else:
            g.edata['label'] = edge_labels

    # Load and attach masks
    if (data_dir / "train_mask.npy").exists():
        train_mask = np.load(data_dir / "train_mask.npy")
        val_mask = np.load(data_dir / "val_mask.npy")
        test_mask = np.load(data_dir / "test_mask.npy")

        if HAS_TORCH:
            g.ndata['train_mask'] = torch.from_numpy(train_mask).bool()
            g.ndata['val_mask'] = torch.from_numpy(val_mask).bool()
            g.ndata['test_mask'] = torch.from_numpy(test_mask).bool()
        else:
            g.ndata['train_mask'] = train_mask
            g.ndata['val_mask'] = val_mask
            g.ndata['test_mask'] = test_mask

    # Store metadata as graph attribute
    g.metadata = metadata

    return g


def load_heterogeneous_graph(data_dir: str = ".") -> "dgl.DGLHeteroGraph":
    """Load graph data into a DGL heterogeneous graph.

    This function handles graphs with multiple node and edge types.

    Args:
        data_dir: Directory containing the exported graph data.

    Returns:
        DGL heterogeneous graph.
    """
    data_dir = Path(data_dir)

    # Load metadata
    with open(data_dir / "metadata.json") as f:
        metadata = json.load(f)

    if not metadata.get("is_heterogeneous", False):
        print("Warning: Graph was not exported as heterogeneous. Using homogeneous loader.")
        return load_graph(data_dir)

    if not HAS_DGL:
        raise ImportError("DGL is required for heterogeneous graph loading")

    # Load edge index and type indices
    edge_index = np.load(data_dir / "edge_index.npy")
    edge_types = np.load(data_dir / "edge_type_indices.npy")
    node_types = np.load(data_dir / "node_type_indices.npy")

    # Get type names from metadata
    node_type_names = list(metadata["node_types"].keys())
    edge_type_names = list(metadata["edge_types"].keys())

    # Build edge dict for heterogeneous graph
    edge_dict = {}
    for etype_idx, etype_name in enumerate(edge_type_names):
        mask = edge_types == etype_idx
        if mask.any():
            src = edge_index[mask, 0]
            dst = edge_index[mask, 1]
            # For heterogeneous, we need to specify (src_type, edge_type, dst_type)
            # Using simplified convention: (node_type, edge_type, node_type)
            edge_dict[(node_type_names[0] if node_type_names else 'node',
                      etype_name,
                      node_type_names[0] if node_type_names else 'node')] = (src, dst)

    # Create heterogeneous graph
    g = dgl.heterograph(edge_dict) if edge_dict else dgl.graph(([], []))
    g.metadata = metadata

    return g


def print_summary(data_dir: str = "."):
    """Print summary of the graph data."""
    data_dir = Path(data_dir)

    with open(data_dir / "metadata.json") as f:
        metadata = json.load(f)

    print(f"Graph: {metadata['name']}")
    print(f"Format: DGL ({metadata['edge_format']} edge format)")
    print(f"Nodes: {metadata['num_nodes']}")
    print(f"Edges: {metadata['num_edges']}")
    print(f"Node feature dim: {metadata['node_feature_dim']}")
    print(f"Edge feature dim: {metadata['edge_feature_dim']}")
    print(f"Directed: {metadata['is_directed']}")
    print(f"Heterogeneous: {metadata['is_heterogeneous']}")

    if metadata['node_types']:
        print(f"Node types: {metadata['node_types']}")
    if metadata['edge_types']:
        print(f"Edge types: {metadata['edge_types']}")

    if metadata['statistics']:
        print("\nStatistics:")
        for key, value in metadata['statistics'].items():
            print(f"  {key}: {value:.4f}")

    if HAS_DGL:
        print("\nLoading graph...")
        g = load_graph(data_dir)
        if hasattr(g, 'num_nodes'):
            print(f"DGL graph loaded: {g.num_nodes()} nodes, {g.num_edges()} edges")
            if 'label' in g.ndata:
                print(f"Anomalous nodes: {g.ndata['label'].sum().item()}")


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

    /// Writes a helper script for saving/loading DGL graphs as pickle.
    fn write_pickle_script(&self, output_dir: &Path) -> std::io::Result<()> {
        let script = r#"#!/usr/bin/env python3
"""
DGL Graph Pickle Helper

Utility to save and load DGL graphs as pickle files for faster subsequent loading.
"""

import pickle
from pathlib import Path

try:
    import dgl
    HAS_DGL = True
except ImportError:
    HAS_DGL = False


def save_dgl_graph(graph, output_path: str):
    """Save a DGL graph to a pickle file.

    Args:
        graph: DGL graph to save.
        output_path: Path to save the pickle file.
    """
    output_path = Path(output_path)

    # Save graph data
    graph_data = {
        'num_nodes': graph.num_nodes(),
        'edges': graph.edges(),
        'ndata': {k: v.numpy() if hasattr(v, 'numpy') else v
                  for k, v in graph.ndata.items()},
        'edata': {k: v.numpy() if hasattr(v, 'numpy') else v
                  for k, v in graph.edata.items()},
        'metadata': getattr(graph, 'metadata', {}),
    }

    with open(output_path, 'wb') as f:
        pickle.dump(graph_data, f, protocol=pickle.HIGHEST_PROTOCOL)

    print(f"Saved graph to {output_path}")


def load_dgl_graph(input_path: str) -> "dgl.DGLGraph":
    """Load a DGL graph from a pickle file.

    Args:
        input_path: Path to the pickle file.

    Returns:
        DGL graph.
    """
    if not HAS_DGL:
        raise ImportError("DGL is required to load graphs")

    import torch

    input_path = Path(input_path)

    with open(input_path, 'rb') as f:
        graph_data = pickle.load(f)

    # Recreate graph
    src, dst = graph_data['edges']
    g = dgl.graph((src, dst), num_nodes=graph_data['num_nodes'])

    # Restore node data
    for k, v in graph_data['ndata'].items():
        g.ndata[k] = torch.from_numpy(v) if hasattr(v, 'dtype') else v

    # Restore edge data
    for k, v in graph_data['edata'].items():
        g.edata[k] = torch.from_numpy(v) if hasattr(v, 'dtype') else v

    # Restore metadata
    g.metadata = graph_data.get('metadata', {})

    return g


def convert_to_pickle(data_dir: str, output_path: str = None):
    """Convert exported graph data to pickle format for faster loading.

    Args:
        data_dir: Directory containing the exported graph data.
        output_path: Path for output pickle file. Defaults to data_dir/graph.pkl.
    """
    from load_graph import load_graph

    data_dir = Path(data_dir)
    output_path = Path(output_path) if output_path else data_dir / "graph.pkl"

    print(f"Loading graph from {data_dir}...")
    g = load_graph(str(data_dir))

    if isinstance(g, dict):
        print("Error: DGL not available, cannot convert to pickle")
        return

    save_dgl_graph(g, str(output_path))
    print(f"Graph saved to {output_path}")


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage:")
        print("  python pickle_helper.py convert <data_dir> [output_path]")
        print("  python pickle_helper.py load <pickle_path>")
        sys.exit(1)

    command = sys.argv[1]

    if command == "convert":
        data_dir = sys.argv[2] if len(sys.argv) > 2 else "."
        output_path = sys.argv[3] if len(sys.argv) > 3 else None
        convert_to_pickle(data_dir, output_path)
    elif command == "load":
        pickle_path = sys.argv[2]
        g = load_dgl_graph(pickle_path)
        print(f"Loaded graph: {g.num_nodes()} nodes, {g.num_edges()} edges")
    else:
        print(f"Unknown command: {command}")
"#;

        let path = output_dir.join("pickle_helper.py");
        let mut file = File::create(path)?;
        file.write_all(script.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_graph_with_company;
    use tempfile::tempdir;

    #[test]
    fn test_dgl_export_basic() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let exporter = DGLExporter::new(DGLExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.common.num_nodes, 3);
        assert_eq!(metadata.common.num_edges, 2);
        assert_eq!(metadata.edge_format, "COO");
        assert!(dir.path().join("edge_index.npy").exists());
        assert!(dir.path().join("node_features.npy").exists());
        assert!(dir.path().join("node_labels.npy").exists());
        assert!(dir.path().join("metadata.json").exists());
        assert!(dir.path().join("load_graph.py").exists());
        assert!(dir.path().join("pickle_helper.py").exists());
    }

    #[test]
    fn test_dgl_export_heterogeneous() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let config = DGLExportConfig {
            heterogeneous: true,
            ..Default::default()
        };
        let exporter = DGLExporter::new(config);
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert!(metadata.is_heterogeneous);
        assert!(dir.path().join("node_type_indices.npy").exists());
        assert!(dir.path().join("edge_type_indices.npy").exists());
    }

    #[test]
    fn test_dgl_export_masks() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let exporter = DGLExporter::new(DGLExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert!(metadata
            .common
            .files
            .contains(&"train_mask.npy".to_string()));
        assert!(metadata.common.files.contains(&"val_mask.npy".to_string()));
        assert!(metadata.common.files.contains(&"test_mask.npy".to_string()));
        assert!(dir.path().join("train_mask.npy").exists());
        assert!(dir.path().join("val_mask.npy").exists());
        assert!(dir.path().join("test_mask.npy").exists());
    }

    #[test]
    fn test_dgl_coo_format() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let exporter = DGLExporter::new(DGLExportConfig::default());
        exporter.export(&graph, dir.path()).unwrap();

        // Verify edge_index file exists and has correct format
        // COO format should be [num_edges, 2]
        let edge_path = dir.path().join("edge_index.npy");
        assert!(edge_path.exists());

        // The metadata confirms format
        let metadata_path = dir.path().join("metadata.json");
        let metadata: DGLMetadata =
            serde_json::from_reader(File::open(metadata_path).unwrap()).unwrap();
        assert_eq!(metadata.edge_format, "COO");
    }

    #[test]
    fn test_dgl_export_no_masks() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let config = DGLExportConfig {
            common: CommonExportConfig {
                export_masks: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let exporter = DGLExporter::new(config);
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert!(!metadata
            .common
            .files
            .contains(&"train_mask.npy".to_string()));
        assert!(!dir.path().join("train_mask.npy").exists());
    }

    #[test]
    fn test_dgl_export_minimal() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let config = DGLExportConfig {
            common: CommonExportConfig {
                export_node_features: false,
                export_edge_features: false,
                export_node_labels: false,
                export_edge_labels: false,
                export_masks: false,
                ..Default::default()
            },
            include_pickle_script: false,
            ..Default::default()
        };
        let exporter = DGLExporter::new(config);
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        // Only edge_index and loader script should exist
        assert_eq!(metadata.common.files.len(), 1); // Only edge_index.npy
        assert!(dir.path().join("edge_index.npy").exists());
        assert!(dir.path().join("load_graph.py").exists()); // Loader always generated
        assert!(dir.path().join("metadata.json").exists());
        assert!(!dir.path().join("pickle_helper.py").exists());
    }

    #[test]
    fn test_dgl_statistics() {
        let graph = create_test_graph_with_company();
        let dir = tempdir().unwrap();

        let exporter = DGLExporter::new(DGLExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        // Should have density and anomaly ratios
        assert!(metadata.common.statistics.contains_key("density"));
        assert!(metadata
            .common
            .statistics
            .contains_key("anomalous_node_ratio"));
        assert!(metadata
            .common
            .statistics
            .contains_key("anomalous_edge_ratio"));

        // One of three nodes is anomalous
        let node_ratio = metadata
            .common
            .statistics
            .get("anomalous_node_ratio")
            .unwrap();
        assert!((*node_ratio - 1.0 / 3.0).abs() < 0.01);
    }
}
