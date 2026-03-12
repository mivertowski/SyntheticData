//! Core export types — crate-owned, no external dependency required.
//!
//! [`ExportNode`] and [`ExportEdge`] are the pipeline's canonical intermediate representation.
//! Conversion to `BulkNodeData`/`BulkEdgeData` is behind the `rustgraph` feature flag.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::ExportWarnings;

// ──────────────────────────── Node ────────────────────────────

/// A single node ready for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNode {
    /// Assigned numeric ID (populated by the pipeline's IdMap).
    /// `None` until the ID assignment phase runs.
    pub id: Option<u64>,
    /// RustGraph entity type code (100-599 range).
    pub node_type: u32,
    /// Human-readable entity type name in snake_case (e.g., "internal_control").
    pub node_type_name: String,
    /// Human-readable label for display.
    pub label: String,
    /// Hypergraph layer (1=Governance, 2=Process, 3=Accounting).
    pub layer: u8,
    /// Serialized properties as JSON key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, serde_json::Value>,
}

// ──────────────────────────── Edge ────────────────────────────

/// A single edge ready for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEdge {
    /// Source node numeric ID (must reference an ExportNode.id).
    pub source: u64,
    /// Target node numeric ID (must reference an ExportNode.id).
    pub target: u64,
    /// RustGraph edge type code (40-120 range).
    pub edge_type: u32,
    /// Edge weight (default 1.0).
    pub weight: f32,
    /// Serialized properties as JSON key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, serde_json::Value>,
}

// ──────────────────────────── OCEL ────────────────────────────

/// OCEL 2.0 export payload.
#[derive(Debug, Clone, Default)]
pub struct OcelExport {
    /// Serialized OCEL 2.0 JSON data.
    pub data: serde_json::Value,
    /// Number of events in the OCEL log.
    pub event_count: usize,
}

// ──────────────────────────── Ground Truth ────────────────────

/// A single ground truth record for ML evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthRecord {
    /// External entity identifier (e.g., "VENDOR-001").
    pub entity_id: String,
    /// Entity type name (e.g., "vendor", "journal_entry").
    pub entity_type: String,
    /// Whether this entity is an anomaly.
    pub is_anomaly: bool,
    /// Specific anomaly type if anomalous (e.g., "duplicate_payment", "ghost_employee").
    pub anomaly_type: Option<String>,
    /// Confidence score for the anomaly label (0.0-1.0).
    pub confidence: f64,
}

// ──────────────────────────── Feature Vectors ─────────────────

/// Feature vector for a node, used for GNN training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFeatureVector {
    /// The numeric node ID this feature vector belongs to.
    pub node_id: u64,
    /// Feature values (order matches the feature schema).
    pub features: Vec<f64>,
}

// ──────────────────────────── Hyperedge ───────────────────────

/// A hyperedge connecting multiple nodes (e.g., journal entry touching N accounts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperedgeExport {
    /// IDs of member nodes.
    pub member_node_ids: Vec<u64>,
    /// IDs of member edges (pairwise decomposition of the hyperedge).
    pub member_edge_ids: Vec<u64>,
    /// Hyperedge properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, serde_json::Value>,
    /// Layer this hyperedge belongs to.
    pub layer: u8,
}

// ──────────────────────────── Metadata ────────────────────────

/// Statistics and metadata about the completed export.
#[derive(Debug, Clone, Default)]
pub struct ExportMetadata {
    /// Total number of nodes exported.
    pub total_nodes: usize,
    /// Total number of edges exported.
    pub total_edges: usize,
    /// Node count per layer: [L1_governance, L2_process, L3_accounting].
    pub nodes_per_layer: [usize; 3],
    /// Distinct edge type codes that were produced.
    pub edge_types_produced: Vec<u32>,
    /// Pipeline execution time in milliseconds.
    pub duration_ms: u64,
}

// ──────────────────────────── Result ──────────────────────────

/// The final output of the graph export pipeline.
#[derive(Debug, Clone)]
pub struct GraphExportResult {
    /// All exported nodes with assigned IDs and serialized properties.
    pub nodes: Vec<ExportNode>,
    /// All exported edges referencing node IDs.
    pub edges: Vec<ExportEdge>,
    /// Optional OCEL 2.0 event log.
    pub ocel: Option<OcelExport>,
    /// Ground truth records for ML evaluation.
    pub ground_truth: Vec<GroundTruthRecord>,
    /// Feature vectors for GNN training.
    pub feature_vectors: Vec<NodeFeatureVector>,
    /// Hyperedges (multi-node relationships).
    pub hyperedges: Vec<HyperedgeExport>,
    /// Export metadata and statistics.
    pub metadata: ExportMetadata,
    /// Non-fatal warnings collected during the pipeline.
    pub warnings: ExportWarnings,
}

impl GraphExportResult {
    /// Create an empty result (useful for testing or error recovery).
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            ocel: None,
            ground_truth: Vec::new(),
            feature_vectors: Vec::new(),
            hyperedges: Vec::new(),
            metadata: ExportMetadata::default(),
            warnings: ExportWarnings::new(),
        }
    }
}

// ──────────────────────────── RustGraph Compatibility ─────────

#[cfg(feature = "rustgraph")]
mod rustgraph_compat {
    use super::*;
    use rustgraph_api_types::{BulkEdgeData, BulkNodeData};

    impl From<ExportNode> for BulkNodeData {
        fn from(n: ExportNode) -> Self {
            let mut props = n.properties;
            // Store node_type_name in properties since BulkNodeData has no dedicated field.
            props.insert("nodeTypeName".into(), n.node_type_name.into());
            BulkNodeData {
                id: n.id,
                node_type: n.node_type,
                layer: Some(n.layer),
                labels: vec![n.label],
                properties: props,
            }
        }
    }

    impl From<ExportEdge> for BulkEdgeData {
        fn from(e: ExportEdge) -> Self {
            BulkEdgeData {
                source: e.source,
                target: e.target,
                edge_type: e.edge_type,
                weight: e.weight,
                properties: e.properties,
            }
        }
    }

    impl GraphExportResult {
        /// Convert the export result into RustGraph bulk import format.
        pub fn into_bulk(self) -> (Vec<BulkNodeData>, Vec<BulkEdgeData>) {
            (
                self.nodes.into_iter().map(Into::into).collect(),
                self.edges.into_iter().map(Into::into).collect(),
            )
        }
    }
}
