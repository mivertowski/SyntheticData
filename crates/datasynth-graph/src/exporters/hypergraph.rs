//! Multi-layer hypergraph exporter for RustGraph integration.
//!
//! Exports a built `Hypergraph` to JSONL files:
//! - `nodes.jsonl` - All nodes with layer, entity_type_code
//! - `edges.jsonl` - Cross-layer and intra-layer pairwise edges
//! - `hyperedges.jsonl` - Journal entries and OCPM events as hyperedges
//! - `metadata.json` - Schema, counts, layer stats, budget report

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::models::hypergraph::{Hypergraph, HypergraphMetadata};

/// Configuration for the hypergraph exporter.
#[derive(Debug, Clone, Default)]
pub struct HypergraphExportConfig {
    /// Pretty-print metadata.json (for debugging).
    pub pretty_print: bool,
}

/// Exports a `Hypergraph` to JSONL files for RustGraph import.
pub struct HypergraphExporter {
    config: HypergraphExportConfig,
}

impl HypergraphExporter {
    /// Create a new exporter with the given configuration.
    pub fn new(config: HypergraphExportConfig) -> Self {
        Self { config }
    }

    /// Export the hypergraph to the given output directory.
    ///
    /// Creates:
    /// - `nodes.jsonl` (one JSON object per line)
    /// - `edges.jsonl` (one JSON object per line)
    /// - `hyperedges.jsonl` (one JSON object per line)
    /// - `metadata.json` (export metadata)
    pub fn export(
        &self,
        hypergraph: &Hypergraph,
        output_dir: &Path,
    ) -> std::io::Result<HypergraphMetadata> {
        fs::create_dir_all(output_dir)?;

        // Export nodes
        let nodes_path = output_dir.join("nodes.jsonl");
        let file = File::create(&nodes_path)?;
        let mut writer = BufWriter::new(file);
        for node in &hypergraph.nodes {
            serde_json::to_writer(&mut writer, node)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Export edges
        let edges_path = output_dir.join("edges.jsonl");
        let file = File::create(&edges_path)?;
        let mut writer = BufWriter::new(file);
        for edge in &hypergraph.edges {
            serde_json::to_writer(&mut writer, edge)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Export hyperedges
        let hyperedges_path = output_dir.join("hyperedges.jsonl");
        let file = File::create(&hyperedges_path)?;
        let mut writer = BufWriter::new(file);
        for he in &hypergraph.hyperedges {
            serde_json::to_writer(&mut writer, he)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Export metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(&metadata_path)?;
        if self.config.pretty_print {
            serde_json::to_writer_pretty(file, &hypergraph.metadata)?;
        } else {
            serde_json::to_writer(file, &hypergraph.metadata)?;
        }

        Ok(hypergraph.metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
    use crate::models::hypergraph::{
        CrossLayerEdge, Hyperedge, HyperedgeParticipant, HypergraphLayer, HypergraphNode,
    };
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_export_creates_all_files() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            include_p2p: false,
            include_o2c: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();

        let hypergraph = builder.build();
        let dir = tempdir().unwrap();

        let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
        let metadata = exporter.export(&hypergraph, dir.path()).unwrap();

        assert!(dir.path().join("nodes.jsonl").exists());
        assert!(dir.path().join("edges.jsonl").exists());
        assert!(dir.path().join("hyperedges.jsonl").exists());
        assert!(dir.path().join("metadata.json").exists());

        assert_eq!(metadata.num_nodes, 22); // 5 components + 17 principles
    }

    #[test]
    fn test_nodes_jsonl_parseable() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            include_p2p: false,
            include_o2c: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();

        let hypergraph = builder.build();
        let dir = tempdir().unwrap();

        let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
        exporter.export(&hypergraph, dir.path()).unwrap();

        // Read and parse each line
        let content = std::fs::read_to_string(dir.path().join("nodes.jsonl")).unwrap();
        let mut count = 0;
        for line in content.lines() {
            let node: HypergraphNode = serde_json::from_str(line).unwrap();
            assert!(!node.id.is_empty());
            assert!(!node.entity_type.is_empty());
            count += 1;
        }
        assert_eq!(count, 22);
    }

    #[test]
    fn test_edges_jsonl_parseable() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            include_p2p: false,
            include_o2c: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();

        let hypergraph = builder.build();
        let dir = tempdir().unwrap();

        let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
        exporter.export(&hypergraph, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("edges.jsonl")).unwrap();
        for line in content.lines() {
            let _edge: CrossLayerEdge = serde_json::from_str(line).unwrap();
        }
    }

    #[test]
    fn test_hyperedges_jsonl_parseable() {
        // Build a hypergraph with a synthetic hyperedge
        let config = HypergraphConfig {
            max_nodes: 1000,
            include_coso: false,
            include_controls: false,
            include_sox: false,
            include_p2p: false,
            include_o2c: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            ..Default::default()
        };
        let builder = HypergraphBuilder::new(config);
        let mut hg = builder.build();

        // Manually inject a hyperedge for testing
        hg.hyperedges.push(Hyperedge {
            id: "test_he".to_string(),
            hyperedge_type: "JournalEntry".to_string(),
            subtype: "R2R".to_string(),
            participants: vec![
                HyperedgeParticipant {
                    node_id: "acct_1000".to_string(),
                    role: "debit".to_string(),
                    weight: Some(100.0),
                },
                HyperedgeParticipant {
                    node_id: "acct_2000".to_string(),
                    role: "credit".to_string(),
                    weight: Some(100.0),
                },
            ],
            layer: HypergraphLayer::AccountingNetwork,
            properties: HashMap::new(),
            timestamp: None,
            is_anomaly: false,
            anomaly_type: None,
            features: vec![4.6, 2.0],
        });

        let dir = tempdir().unwrap();
        let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
        exporter.export(&hg, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("hyperedges.jsonl")).unwrap();
        let mut count = 0;
        for line in content.lines() {
            let he: Hyperedge = serde_json::from_str(line).unwrap();
            assert_eq!(he.participants.len(), 2);
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_metadata_json_parseable() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            include_p2p: false,
            include_o2c: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();

        let hypergraph = builder.build();
        let dir = tempdir().unwrap();

        let exporter = HypergraphExporter::new(HypergraphExportConfig { pretty_print: true });
        exporter.export(&hypergraph, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("metadata.json")).unwrap();
        let metadata: HypergraphMetadata = serde_json::from_str(&content).unwrap();
        assert_eq!(metadata.num_nodes, 22);
        assert_eq!(metadata.source, "datasynth");
        assert!(!metadata.files.is_empty());
    }
}
