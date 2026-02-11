//! RustGraph Unified Hypergraph exporter.
//!
//! Maps internal hypergraph types to RustGraph's expected unified format:
//! - `entity_type` → `node_type`
//! - `source_id`/`target_id` → `source`/`target`
//! - `label` → `name`
//! - `HypergraphLayer` enum → `layer` as `u8`
//!
//! Preserves backward compatibility by wrapping rather than renaming internal fields.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::hypergraph::{
    CrossLayerEdge, Hyperedge, HyperedgeParticipant, Hypergraph, HypergraphMetadata,
    HypergraphNode, NodeBudgetReport,
};

/// A node in the RustGraph unified format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawUnifiedNode {
    /// Unique node identifier.
    pub id: String,
    /// Entity type name (mapped from `entity_type`).
    pub node_type: String,
    /// RustGraph entity type code.
    pub entity_type_code: u32,
    /// Layer index (1-3) instead of enum string.
    pub layer: u8,
    /// External identifier from the source system.
    pub external_id: String,
    /// Human-readable name (mapped from `label`).
    pub name: String,
    /// Additional properties as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Value>,
    /// Numeric feature vector for ML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<f64>,
    /// Whether this node represents an anomaly.
    #[serde(default)]
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_type: Option<String>,
    /// Whether this is an aggregate (pool) node from budget compression.
    #[serde(default)]
    pub is_aggregate: bool,
    /// Number of original entities this aggregate node represents.
    #[serde(default)]
    pub aggregate_count: usize,
}

impl RawUnifiedNode {
    /// Convert from internal `HypergraphNode` to unified format.
    pub fn from_hypergraph_node(node: &HypergraphNode) -> Self {
        Self {
            id: node.id.clone(),
            node_type: node.entity_type.clone(),
            entity_type_code: node.entity_type_code,
            layer: node.layer.index(),
            external_id: node.external_id.clone(),
            name: node.label.clone(),
            properties: node.properties.clone(),
            features: node.features.clone(),
            is_anomaly: node.is_anomaly,
            anomaly_type: node.anomaly_type.clone(),
            is_aggregate: node.is_aggregate,
            aggregate_count: node.aggregate_count,
        }
    }
}

/// An edge in the RustGraph unified format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawUnifiedEdge {
    /// Source node ID (mapped from `source_id`).
    pub source: String,
    /// Target node ID (mapped from `target_id`).
    pub target: String,
    /// Source layer index (1-3).
    pub source_layer: u8,
    /// Target layer index (1-3).
    pub target_layer: u8,
    /// Edge type name.
    pub edge_type: String,
    /// RustGraph edge type code.
    pub edge_type_code: u32,
    /// Edge weight (default 1.0).
    pub weight: f32,
    /// Additional properties as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Value>,
}

impl RawUnifiedEdge {
    /// Convert from internal `CrossLayerEdge` to unified format.
    pub fn from_cross_layer_edge(edge: &CrossLayerEdge) -> Self {
        Self {
            source: edge.source_id.clone(),
            target: edge.target_id.clone(),
            source_layer: edge.source_layer.index(),
            target_layer: edge.target_layer.index(),
            edge_type: edge.edge_type.clone(),
            edge_type_code: edge.edge_type_code,
            weight: 1.0,
            properties: edge.properties.clone(),
        }
    }
}

/// A hyperedge in the RustGraph unified format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawUnifiedHyperedge {
    /// Unique hyperedge identifier.
    pub id: String,
    /// High-level type: "ProcessFamily", "MultiRelation", "JournalEntry".
    pub hyperedge_type: String,
    /// Subtype with more detail.
    pub subtype: String,
    /// IDs of all member nodes (extracted from participants).
    pub member_ids: Vec<String>,
    /// Layer index (1-3).
    pub layer: u8,
    /// Full participant details with roles and weights.
    pub participants: Vec<HyperedgeParticipant>,
    /// Additional properties as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Value>,
    /// Optional timestamp for temporal hyperedges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<NaiveDate>,
    /// Whether this hyperedge represents an anomaly.
    #[serde(default)]
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_type: Option<String>,
    /// Numeric feature vector for ML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<f64>,
}

impl RawUnifiedHyperedge {
    /// Convert from internal `Hyperedge` to unified format.
    pub fn from_hyperedge(he: &Hyperedge) -> Self {
        Self {
            id: he.id.clone(),
            hyperedge_type: he.hyperedge_type.clone(),
            subtype: he.subtype.clone(),
            member_ids: he.participants.iter().map(|p| p.node_id.clone()).collect(),
            layer: he.layer.index(),
            participants: he.participants.clone(),
            properties: he.properties.clone(),
            timestamp: he.timestamp,
            is_anomaly: he.is_anomaly,
            anomaly_type: he.anomaly_type.clone(),
            features: he.features.clone(),
        }
    }
}

/// Metadata for the unified hypergraph export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedHypergraphMetadata {
    /// Format identifier for RustGraph import.
    pub format: String,
    /// Name of this hypergraph export.
    pub name: String,
    /// Total number of nodes.
    pub num_nodes: usize,
    /// Total number of pairwise edges.
    pub num_edges: usize,
    /// Total number of hyperedges.
    pub num_hyperedges: usize,
    /// Node counts per layer.
    pub layer_node_counts: HashMap<String, usize>,
    /// Node counts per entity type.
    pub node_type_counts: HashMap<String, usize>,
    /// Edge counts per edge type.
    pub edge_type_counts: HashMap<String, usize>,
    /// Hyperedge counts per type.
    pub hyperedge_type_counts: HashMap<String, usize>,
    /// Number of anomalous nodes.
    pub anomalous_nodes: usize,
    /// Number of anomalous hyperedges.
    pub anomalous_hyperedges: usize,
    /// Source system identifier.
    pub source: String,
    /// Generation timestamp (ISO 8601).
    pub generated_at: String,
    /// Budget utilization report.
    pub budget_report: NodeBudgetReport,
    /// Files included in export.
    pub files: Vec<String>,
}

impl UnifiedHypergraphMetadata {
    /// Create unified metadata from internal `HypergraphMetadata`.
    pub fn from_metadata(meta: &HypergraphMetadata) -> Self {
        Self {
            format: "rustgraph_unified_v1".to_string(),
            name: meta.name.clone(),
            num_nodes: meta.num_nodes,
            num_edges: meta.num_edges,
            num_hyperedges: meta.num_hyperedges,
            layer_node_counts: meta.layer_node_counts.clone(),
            node_type_counts: meta.node_type_counts.clone(),
            edge_type_counts: meta.edge_type_counts.clone(),
            hyperedge_type_counts: meta.hyperedge_type_counts.clone(),
            anomalous_nodes: meta.anomalous_nodes,
            anomalous_hyperedges: meta.anomalous_hyperedges,
            source: meta.source.clone(),
            generated_at: meta.generated_at.clone(),
            budget_report: meta.budget_report.clone(),
            files: meta.files.clone(),
        }
    }
}

/// Configuration for the RustGraph unified exporter.
#[derive(Debug, Clone, Default)]
pub struct UnifiedExportConfig {
    /// Pretty-print metadata.json (for debugging).
    pub pretty_print: bool,
}

/// Exports a `Hypergraph` to JSONL files in RustGraph's unified format.
pub struct RustGraphUnifiedExporter {
    config: UnifiedExportConfig,
}

impl RustGraphUnifiedExporter {
    /// Create a new unified exporter with the given configuration.
    pub fn new(config: UnifiedExportConfig) -> Self {
        Self { config }
    }

    /// Export the hypergraph to the given output directory in unified format.
    ///
    /// Creates:
    /// - `nodes.jsonl` (one JSON object per line, unified field names)
    /// - `edges.jsonl` (one JSON object per line, unified field names)
    /// - `hyperedges.jsonl` (one JSON object per line, unified field names)
    /// - `metadata.json` (export metadata with `format: "rustgraph_unified_v1"`)
    pub fn export(
        &self,
        hypergraph: &Hypergraph,
        output_dir: &Path,
    ) -> std::io::Result<UnifiedHypergraphMetadata> {
        fs::create_dir_all(output_dir)?;

        // Export nodes
        let nodes_path = output_dir.join("nodes.jsonl");
        let file = File::create(nodes_path)?;
        let mut writer = BufWriter::new(file);
        for node in &hypergraph.nodes {
            let unified = RawUnifiedNode::from_hypergraph_node(node);
            serde_json::to_writer(&mut writer, &unified)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Export edges
        let edges_path = output_dir.join("edges.jsonl");
        let file = File::create(edges_path)?;
        let mut writer = BufWriter::new(file);
        for edge in &hypergraph.edges {
            let unified = RawUnifiedEdge::from_cross_layer_edge(edge);
            serde_json::to_writer(&mut writer, &unified)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Export hyperedges
        let hyperedges_path = output_dir.join("hyperedges.jsonl");
        let file = File::create(hyperedges_path)?;
        let mut writer = BufWriter::new(file);
        for he in &hypergraph.hyperedges {
            let unified = RawUnifiedHyperedge::from_hyperedge(he);
            serde_json::to_writer(&mut writer, &unified)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        // Build unified metadata
        let mut metadata = UnifiedHypergraphMetadata::from_metadata(&hypergraph.metadata);
        metadata.files = vec![
            "nodes.jsonl".to_string(),
            "edges.jsonl".to_string(),
            "hyperedges.jsonl".to_string(),
            "metadata.json".to_string(),
        ];

        // Export metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(metadata_path)?;
        if self.config.pretty_print {
            serde_json::to_writer_pretty(file, &metadata)?;
        } else {
            serde_json::to_writer(file, &metadata)?;
        }

        Ok(metadata)
    }

    /// Export the hypergraph to a writer in unified JSONL format (for streaming).
    ///
    /// Writes all nodes, edges, and hyperedges as JSONL to the writer.
    /// Each line is prefixed with a type tag for demultiplexing:
    /// `{"_type":"node",...}`, `{"_type":"edge",...}`, `{"_type":"hyperedge",...}`
    pub fn export_to_writer<W: Write>(
        &self,
        hypergraph: &Hypergraph,
        writer: &mut W,
    ) -> std::io::Result<UnifiedHypergraphMetadata> {
        // Write nodes
        for node in &hypergraph.nodes {
            let unified = RawUnifiedNode::from_hypergraph_node(node);
            let mut obj = serde_json::to_value(&unified)?;
            obj.as_object_mut()
                .expect("serialized struct is always a JSON object")
                .insert("_type".to_string(), Value::String("node".to_string()));
            serde_json::to_writer(&mut *writer, &obj)?;
            writeln!(writer)?;
        }

        // Write edges
        for edge in &hypergraph.edges {
            let unified = RawUnifiedEdge::from_cross_layer_edge(edge);
            let mut obj = serde_json::to_value(&unified)?;
            obj.as_object_mut()
                .expect("serialized struct is always a JSON object")
                .insert("_type".to_string(), Value::String("edge".to_string()));
            serde_json::to_writer(&mut *writer, &obj)?;
            writeln!(writer)?;
        }

        // Write hyperedges
        for he in &hypergraph.hyperedges {
            let unified = RawUnifiedHyperedge::from_hyperedge(he);
            let mut obj = serde_json::to_value(&unified)?;
            obj.as_object_mut()
                .expect("serialized struct is always a JSON object")
                .insert("_type".to_string(), Value::String("hyperedge".to_string()));
            serde_json::to_writer(&mut *writer, &obj)?;
            writeln!(writer)?;
        }

        let mut metadata = UnifiedHypergraphMetadata::from_metadata(&hypergraph.metadata);
        metadata.files = vec![];

        Ok(metadata)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
    use crate::models::hypergraph::HypergraphLayer;
    use tempfile::tempdir;

    fn build_test_hypergraph() -> Hypergraph {
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
        builder.build()
    }

    #[test]
    fn test_node_conversion() {
        let node = HypergraphNode {
            id: "node_1".to_string(),
            entity_type: "Account".to_string(),
            entity_type_code: 100,
            layer: HypergraphLayer::AccountingNetwork,
            external_id: "1000".to_string(),
            label: "Cash".to_string(),
            properties: HashMap::new(),
            features: vec![1.0, 2.0],
            is_anomaly: false,
            anomaly_type: None,
            is_aggregate: false,
            aggregate_count: 0,
        };

        let unified = RawUnifiedNode::from_hypergraph_node(&node);
        assert_eq!(unified.id, "node_1");
        assert_eq!(unified.node_type, "Account");
        assert_eq!(unified.name, "Cash");
        assert_eq!(unified.layer, 3); // AccountingNetwork = 3
        assert_eq!(unified.entity_type_code, 100);
        assert_eq!(unified.external_id, "1000");
        assert_eq!(unified.features, vec![1.0, 2.0]);
    }

    #[test]
    fn test_edge_conversion() {
        let edge = CrossLayerEdge {
            source_id: "ctrl_C001".to_string(),
            source_layer: HypergraphLayer::GovernanceControls,
            target_id: "acct_1000".to_string(),
            target_layer: HypergraphLayer::AccountingNetwork,
            edge_type: "ImplementsControl".to_string(),
            edge_type_code: 40,
            properties: HashMap::new(),
        };

        let unified = RawUnifiedEdge::from_cross_layer_edge(&edge);
        assert_eq!(unified.source, "ctrl_C001");
        assert_eq!(unified.target, "acct_1000");
        assert_eq!(unified.source_layer, 1); // GovernanceControls = 1
        assert_eq!(unified.target_layer, 3); // AccountingNetwork = 3
        assert_eq!(unified.edge_type, "ImplementsControl");
        assert_eq!(unified.edge_type_code, 40);
        assert_eq!(unified.weight, 1.0);
    }

    #[test]
    fn test_hyperedge_conversion() {
        let he = Hyperedge {
            id: "he_1".to_string(),
            hyperedge_type: "JournalEntry".to_string(),
            subtype: "R2R".to_string(),
            participants: vec![
                HyperedgeParticipant {
                    node_id: "acct_1000".to_string(),
                    role: "debit".to_string(),
                    weight: Some(500.0),
                },
                HyperedgeParticipant {
                    node_id: "acct_2000".to_string(),
                    role: "credit".to_string(),
                    weight: Some(500.0),
                },
            ],
            layer: HypergraphLayer::AccountingNetwork,
            properties: HashMap::new(),
            timestamp: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            is_anomaly: true,
            anomaly_type: Some("split_transaction".to_string()),
            features: vec![6.2, 1.0],
        };

        let unified = RawUnifiedHyperedge::from_hyperedge(&he);
        assert_eq!(unified.id, "he_1");
        assert_eq!(unified.hyperedge_type, "JournalEntry");
        assert_eq!(unified.layer, 3); // AccountingNetwork = 3
        assert_eq!(unified.member_ids, vec!["acct_1000", "acct_2000"]);
        assert_eq!(unified.participants.len(), 2);
        assert!(unified.is_anomaly);
        assert_eq!(unified.anomaly_type, Some("split_transaction".to_string()));
    }

    #[test]
    fn test_unified_export_creates_all_files() {
        let hypergraph = build_test_hypergraph();
        let dir = tempdir().unwrap();

        let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
        let metadata = exporter.export(&hypergraph, dir.path()).unwrap();

        assert!(dir.path().join("nodes.jsonl").exists());
        assert!(dir.path().join("edges.jsonl").exists());
        assert!(dir.path().join("hyperedges.jsonl").exists());
        assert!(dir.path().join("metadata.json").exists());

        assert_eq!(metadata.num_nodes, 22); // 5 components + 17 principles
        assert_eq!(metadata.format, "rustgraph_unified_v1");
    }

    #[test]
    fn test_unified_nodes_jsonl_parseable() {
        let hypergraph = build_test_hypergraph();
        let dir = tempdir().unwrap();

        let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
        exporter.export(&hypergraph, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("nodes.jsonl")).unwrap();
        let mut count = 0;
        for line in content.lines() {
            let node: RawUnifiedNode = serde_json::from_str(line).unwrap();
            assert!(!node.id.is_empty());
            assert!(!node.node_type.is_empty());
            assert!(!node.name.is_empty());
            // Layer should be u8, not string
            assert!(node.layer >= 1 && node.layer <= 3);
            count += 1;
        }
        assert_eq!(count, 22);
    }

    #[test]
    fn test_unified_edges_jsonl_uses_source_target() {
        let hypergraph = build_test_hypergraph();
        let dir = tempdir().unwrap();

        let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
        exporter.export(&hypergraph, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("edges.jsonl")).unwrap();
        for line in content.lines() {
            let edge: RawUnifiedEdge = serde_json::from_str(line).unwrap();
            // Verify unified field names work
            assert!(!edge.source.is_empty());
            assert!(!edge.target.is_empty());
            assert!(edge.source_layer >= 1 && edge.source_layer <= 3);
            assert!(edge.target_layer >= 1 && edge.target_layer <= 3);
            assert_eq!(edge.weight, 1.0);
        }
    }

    #[test]
    fn test_unified_metadata_has_format_field() {
        let hypergraph = build_test_hypergraph();
        let dir = tempdir().unwrap();

        let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig { pretty_print: true });
        exporter.export(&hypergraph, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("metadata.json")).unwrap();
        let metadata: UnifiedHypergraphMetadata = serde_json::from_str(&content).unwrap();
        assert_eq!(metadata.format, "rustgraph_unified_v1");
        assert_eq!(metadata.source, "datasynth");
    }

    #[test]
    fn test_export_to_writer() {
        let hypergraph = build_test_hypergraph();
        let mut buffer = Vec::new();

        let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
        let metadata = exporter.export_to_writer(&hypergraph, &mut buffer).unwrap();

        assert_eq!(metadata.num_nodes, 22);

        // Verify each line is valid JSONL with a _type field
        let content = String::from_utf8(buffer).unwrap();
        let mut node_count = 0;
        let mut edge_count = 0;
        for line in content.lines() {
            let obj: serde_json::Value = serde_json::from_str(line).unwrap();
            let record_type = obj.get("_type").unwrap().as_str().unwrap();
            match record_type {
                "node" => node_count += 1,
                "edge" => edge_count += 1,
                "hyperedge" => {}
                _ => panic!("Unexpected _type: {}", record_type),
            }
        }
        assert_eq!(node_count, 22);
        assert!(edge_count > 0); // COSO edges
    }
}
