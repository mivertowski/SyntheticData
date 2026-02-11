//! RustGraph JSON exporter.
//!
//! Exports graph data in JSON format compatible with RustGraph/RustAssureTwin:
//! - JSONL files for nodes and edges (streaming-friendly)
//! - JSON array format for batch import
//! - Full temporal and feature metadata

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{EdgeDirection, EdgeProperty, Graph, GraphEdge, GraphNode, NodeProperty};

/// Configuration for RustGraph export.
#[derive(Debug, Clone)]
pub struct RustGraphExportConfig {
    /// Include numeric features in output.
    pub include_features: bool,
    /// Include temporal metadata (valid_from, valid_to, transaction_time).
    pub include_temporal: bool,
    /// Include ML labels in output.
    pub include_labels: bool,
    /// Source name for provenance tracking.
    pub source_name: String,
    /// Optional batch ID for grouping exports.
    pub batch_id: Option<String>,
    /// Output format (JsonLines or JsonArray).
    pub output_format: RustGraphOutputFormat,
    /// Export node properties.
    pub export_node_properties: bool,
    /// Export edge properties.
    pub export_edge_properties: bool,
    /// Pretty print JSON (for debugging).
    pub pretty_print: bool,
}

impl Default for RustGraphExportConfig {
    fn default() -> Self {
        Self {
            include_features: true,
            include_temporal: true,
            include_labels: true,
            source_name: "datasynth".to_string(),
            batch_id: None,
            output_format: RustGraphOutputFormat::JsonLines,
            export_node_properties: true,
            export_edge_properties: true,
            pretty_print: false,
        }
    }
}

/// Output format for RustGraph export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustGraphOutputFormat {
    /// JSON Lines format (one JSON object per line).
    JsonLines,
    /// JSON array format (single array containing all objects).
    JsonArray,
}

/// Node output compatible with RustGraph CreateNodeRequest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustGraphNodeOutput {
    /// Node type (e.g., "Account", "Vendor", "Customer").
    pub node_type: String,
    /// Unique identifier.
    pub id: String,
    /// Node properties as key-value pairs.
    pub properties: HashMap<String, Value>,
    /// Metadata for the node.
    pub metadata: RustGraphNodeMetadata,
}

/// Metadata for a RustGraph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustGraphNodeMetadata {
    /// Source system identifier.
    pub source: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Valid time start (business time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<NaiveDateTime>,
    /// Valid time end (business time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<NaiveDateTime>,
    /// Transaction time (system time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_time: Option<DateTime<Utc>>,
    /// Custom labels for classification.
    pub labels: HashMap<String, String>,
    /// Numeric feature vector for ML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<f64>>,
    /// Batch identifier for grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Whether this node is an anomaly.
    #[serde(default)]
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_type: Option<String>,
}

/// Edge output compatible with RustGraph CreateEdgeRequest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustGraphEdgeOutput {
    /// Edge type (e.g., "Transaction", "Approval", "Ownership").
    pub edge_type: String,
    /// Unique identifier.
    pub id: String,
    /// Source node ID.
    pub source_id: String,
    /// Target node ID.
    pub target_id: String,
    /// Edge properties as key-value pairs.
    pub properties: HashMap<String, Value>,
    /// Metadata for the edge.
    pub metadata: RustGraphEdgeMetadata,
}

/// Metadata for a RustGraph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustGraphEdgeMetadata {
    /// Source system identifier.
    pub source: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Edge weight (e.g., transaction amount).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Valid time start (business time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<NaiveDateTime>,
    /// Valid time end (business time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<NaiveDateTime>,
    /// Custom labels for classification.
    pub labels: HashMap<String, String>,
    /// Numeric feature vector for ML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<f64>>,
    /// Whether edge is directed.
    pub is_directed: bool,
    /// Whether this edge is an anomaly.
    #[serde(default)]
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_type: Option<String>,
    /// Batch identifier for grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
}

/// Metadata about the exported RustGraph data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustGraphMetadata {
    /// Graph name.
    pub name: String,
    /// Number of nodes exported.
    pub num_nodes: usize,
    /// Number of edges exported.
    pub num_edges: usize,
    /// Node type counts.
    pub node_type_counts: HashMap<String, usize>,
    /// Edge type counts.
    pub edge_type_counts: HashMap<String, usize>,
    /// Node feature dimension (0 if no features).
    pub node_feature_dim: usize,
    /// Edge feature dimension (0 if no features).
    pub edge_feature_dim: usize,
    /// Graph density (edges / possible edges).
    pub graph_density: f64,
    /// Number of anomalous nodes.
    pub anomalous_nodes: usize,
    /// Number of anomalous edges.
    pub anomalous_edges: usize,
    /// Source system identifier.
    pub source: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Output format used.
    pub output_format: String,
    /// Files included in export.
    pub files: Vec<String>,
}

/// RustGraph JSON exporter.
pub struct RustGraphExporter {
    config: RustGraphExportConfig,
}

impl RustGraphExporter {
    /// Creates a new RustGraph exporter with the given configuration.
    pub fn new(config: RustGraphExportConfig) -> Self {
        Self { config }
    }

    /// Exports a graph to RustGraph JSON format.
    pub fn export(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<RustGraphMetadata> {
        fs::create_dir_all(output_dir)?;

        let mut files = Vec::new();
        let generated_at = Utc::now();

        // Export nodes
        let (node_type_counts, anomalous_nodes, node_feature_dim) =
            self.export_nodes(graph, output_dir, &mut files, generated_at)?;

        // Export edges
        let (edge_type_counts, anomalous_edges, edge_feature_dim) =
            self.export_edges(graph, output_dir, &mut files, generated_at)?;

        // Calculate graph density
        let n = graph.node_count();
        let possible_edges = if n > 1 { n * (n - 1) } else { 1 };
        let graph_density = graph.edge_count() as f64 / possible_edges as f64;

        // Create metadata
        let metadata = RustGraphMetadata {
            name: graph.name.clone(),
            num_nodes: graph.node_count(),
            num_edges: graph.edge_count(),
            node_type_counts,
            edge_type_counts,
            node_feature_dim,
            edge_feature_dim,
            graph_density,
            anomalous_nodes,
            anomalous_edges,
            source: self.config.source_name.clone(),
            generated_at,
            output_format: match self.config.output_format {
                RustGraphOutputFormat::JsonLines => "jsonl".to_string(),
                RustGraphOutputFormat::JsonArray => "json".to_string(),
            },
            files: files.clone(),
        };

        // Write metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(&metadata_path)?;
        if self.config.pretty_print {
            serde_json::to_writer_pretty(file, &metadata)?;
        } else {
            serde_json::to_writer(file, &metadata)?;
        }
        files.push("metadata.json".to_string());

        Ok(metadata)
    }

    /// Exports nodes to file(s).
    fn export_nodes(
        &self,
        graph: &Graph,
        output_dir: &Path,
        files: &mut Vec<String>,
        generated_at: DateTime<Utc>,
    ) -> std::io::Result<(HashMap<String, usize>, usize, usize)> {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut anomalous_count = 0;
        let mut max_feature_dim = 0;

        // Collect all nodes
        let nodes: Vec<RustGraphNodeOutput> = graph
            .nodes
            .values()
            .map(|node| {
                let output = self.convert_node(node, generated_at);
                *type_counts.entry(output.node_type.clone()).or_insert(0) += 1;
                if output.metadata.is_anomaly {
                    anomalous_count += 1;
                }
                if let Some(ref features) = output.metadata.features {
                    max_feature_dim = max_feature_dim.max(features.len());
                }
                output
            })
            .collect();

        // Write nodes
        let filename = match self.config.output_format {
            RustGraphOutputFormat::JsonLines => "nodes.jsonl",
            RustGraphOutputFormat::JsonArray => "nodes.json",
        };
        let path = output_dir.join(filename);
        files.push(filename.to_string());

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        match self.config.output_format {
            RustGraphOutputFormat::JsonLines => {
                for node in &nodes {
                    serde_json::to_writer(&mut writer, node)?;
                    writeln!(writer)?;
                }
            }
            RustGraphOutputFormat::JsonArray => {
                if self.config.pretty_print {
                    serde_json::to_writer_pretty(&mut writer, &nodes)?;
                } else {
                    serde_json::to_writer(&mut writer, &nodes)?;
                }
            }
        }

        writer.flush()?;

        Ok((type_counts, anomalous_count, max_feature_dim))
    }

    /// Exports edges to file(s).
    fn export_edges(
        &self,
        graph: &Graph,
        output_dir: &Path,
        files: &mut Vec<String>,
        generated_at: DateTime<Utc>,
    ) -> std::io::Result<(HashMap<String, usize>, usize, usize)> {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut anomalous_count = 0;
        let mut max_feature_dim = 0;

        // Collect all edges
        let edges: Vec<RustGraphEdgeOutput> = graph
            .edges
            .values()
            .map(|edge| {
                let output = self.convert_edge(edge, generated_at);
                *type_counts.entry(output.edge_type.clone()).or_insert(0) += 1;
                if output.metadata.is_anomaly {
                    anomalous_count += 1;
                }
                if let Some(ref features) = output.metadata.features {
                    max_feature_dim = max_feature_dim.max(features.len());
                }
                output
            })
            .collect();

        // Write edges
        let filename = match self.config.output_format {
            RustGraphOutputFormat::JsonLines => "edges.jsonl",
            RustGraphOutputFormat::JsonArray => "edges.json",
        };
        let path = output_dir.join(filename);
        files.push(filename.to_string());

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        match self.config.output_format {
            RustGraphOutputFormat::JsonLines => {
                for edge in &edges {
                    serde_json::to_writer(&mut writer, edge)?;
                    writeln!(writer)?;
                }
            }
            RustGraphOutputFormat::JsonArray => {
                if self.config.pretty_print {
                    serde_json::to_writer_pretty(&mut writer, &edges)?;
                } else {
                    serde_json::to_writer(&mut writer, &edges)?;
                }
            }
        }

        writer.flush()?;

        Ok((type_counts, anomalous_count, max_feature_dim))
    }

    /// Converts a GraphNode to RustGraphNodeOutput.
    pub fn convert_node(
        &self,
        node: &GraphNode,
        generated_at: DateTime<Utc>,
    ) -> RustGraphNodeOutput {
        let mut properties = HashMap::new();

        // Add external ID and label as properties
        properties.insert(
            "external_id".to_string(),
            Value::String(node.external_id.clone()),
        );
        properties.insert("label".to_string(), Value::String(node.label.clone()));

        // Add node properties
        if self.config.export_node_properties {
            for (key, value) in &node.properties {
                properties.insert(key.clone(), node_property_to_json(value));
            }
        }

        // Add categorical features as properties
        for (key, value) in &node.categorical_features {
            properties.insert(key.clone(), Value::String(value.clone()));
        }

        // Build labels map
        let mut labels = HashMap::new();
        for (i, label) in node.labels.iter().enumerate() {
            labels.insert(format!("label_{}", i), label.clone());
        }

        RustGraphNodeOutput {
            node_type: node.node_type.as_str().to_string(),
            id: node.id.to_string(),
            properties,
            metadata: RustGraphNodeMetadata {
                source: self.config.source_name.clone(),
                generated_at,
                valid_from: if self.config.include_temporal {
                    Some(generated_at.naive_utc())
                } else {
                    None
                },
                valid_to: None,
                transaction_time: if self.config.include_temporal {
                    Some(generated_at)
                } else {
                    None
                },
                labels: if self.config.include_labels {
                    labels
                } else {
                    HashMap::new()
                },
                features: if self.config.include_features && !node.features.is_empty() {
                    Some(node.features.clone())
                } else {
                    None
                },
                batch_id: self.config.batch_id.clone(),
                is_anomaly: node.is_anomaly,
                anomaly_type: node.anomaly_type.clone(),
            },
        }
    }

    /// Converts a GraphEdge to RustGraphEdgeOutput.
    pub fn convert_edge(
        &self,
        edge: &GraphEdge,
        generated_at: DateTime<Utc>,
    ) -> RustGraphEdgeOutput {
        let mut properties = HashMap::new();

        // Add edge properties
        if self.config.export_edge_properties {
            for (key, value) in &edge.properties {
                properties.insert(key.clone(), edge_property_to_json(value));
            }
        }

        // Add timestamp if present
        if let Some(ts) = edge.timestamp {
            properties.insert("timestamp".to_string(), Value::String(ts.to_string()));
        }

        // Build labels map
        let mut labels = HashMap::new();
        for (i, label) in edge.labels.iter().enumerate() {
            labels.insert(format!("label_{}", i), label.clone());
        }

        // Determine valid_from from timestamp
        let valid_from = if self.config.include_temporal {
            edge.timestamp
                .map(|d| d.and_hms_opt(0, 0, 0).expect("midnight is always valid"))
                .or_else(|| Some(generated_at.naive_utc()))
        } else {
            None
        };

        RustGraphEdgeOutput {
            edge_type: edge.edge_type.as_str().to_string(),
            id: edge.id.to_string(),
            source_id: edge.source.to_string(),
            target_id: edge.target.to_string(),
            properties,
            metadata: RustGraphEdgeMetadata {
                source: self.config.source_name.clone(),
                generated_at,
                weight: Some(edge.weight),
                valid_from,
                valid_to: None,
                labels: if self.config.include_labels {
                    labels
                } else {
                    HashMap::new()
                },
                features: if self.config.include_features && !edge.features.is_empty() {
                    Some(edge.features.clone())
                } else {
                    None
                },
                is_directed: edge.direction == EdgeDirection::Directed,
                is_anomaly: edge.is_anomaly,
                anomaly_type: edge.anomaly_type.clone(),
                batch_id: self.config.batch_id.clone(),
            },
        }
    }

    /// Exports a graph to a writer (for streaming export).
    pub fn export_to_writer<W: Write>(
        &self,
        graph: &Graph,
        writer: &mut W,
    ) -> std::io::Result<RustGraphMetadata> {
        let generated_at = Utc::now();
        let mut type_counts_nodes: HashMap<String, usize> = HashMap::new();
        let mut type_counts_edges: HashMap<String, usize> = HashMap::new();
        let mut anomalous_nodes = 0;
        let mut anomalous_edges = 0;
        let mut node_feature_dim = 0;
        let mut edge_feature_dim = 0;

        // Convert all nodes
        let nodes: Vec<RustGraphNodeOutput> = graph
            .nodes
            .values()
            .map(|node| {
                let output = self.convert_node(node, generated_at);
                *type_counts_nodes
                    .entry(output.node_type.clone())
                    .or_insert(0) += 1;
                if output.metadata.is_anomaly {
                    anomalous_nodes += 1;
                }
                if let Some(ref features) = output.metadata.features {
                    node_feature_dim = node_feature_dim.max(features.len());
                }
                output
            })
            .collect();

        // Convert all edges
        let edges: Vec<RustGraphEdgeOutput> = graph
            .edges
            .values()
            .map(|edge| {
                let output = self.convert_edge(edge, generated_at);
                *type_counts_edges
                    .entry(output.edge_type.clone())
                    .or_insert(0) += 1;
                if output.metadata.is_anomaly {
                    anomalous_edges += 1;
                }
                if let Some(ref features) = output.metadata.features {
                    edge_feature_dim = edge_feature_dim.max(features.len());
                }
                output
            })
            .collect();

        // Calculate density
        let n = graph.node_count();
        let possible_edges = if n > 1 { n * (n - 1) } else { 1 };
        let graph_density = graph.edge_count() as f64 / possible_edges as f64;

        // Build combined output
        #[derive(Serialize)]
        struct CombinedOutput<'a> {
            nodes: &'a [RustGraphNodeOutput],
            edges: &'a [RustGraphEdgeOutput],
        }

        let combined = CombinedOutput {
            nodes: &nodes,
            edges: &edges,
        };

        if self.config.pretty_print {
            serde_json::to_writer_pretty(writer, &combined)?;
        } else {
            serde_json::to_writer(writer, &combined)?;
        }

        Ok(RustGraphMetadata {
            name: graph.name.clone(),
            num_nodes: graph.node_count(),
            num_edges: graph.edge_count(),
            node_type_counts: type_counts_nodes,
            edge_type_counts: type_counts_edges,
            node_feature_dim,
            edge_feature_dim,
            graph_density,
            anomalous_nodes,
            anomalous_edges,
            source: self.config.source_name.clone(),
            generated_at,
            output_format: "json".to_string(),
            files: vec![],
        })
    }
}

/// Converts a NodeProperty to a JSON Value.
fn node_property_to_json(prop: &NodeProperty) -> Value {
    match prop {
        NodeProperty::String(s) => Value::String(s.clone()),
        NodeProperty::Int(i) => Value::Number((*i).into()),
        NodeProperty::Float(f) => {
            serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number)
        }
        NodeProperty::Decimal(d) => Value::String(d.to_string()),
        NodeProperty::Bool(b) => Value::Bool(*b),
        NodeProperty::Date(d) => Value::String(d.to_string()),
        NodeProperty::StringList(v) => {
            Value::Array(v.iter().map(|s| Value::String(s.clone())).collect())
        }
    }
}

/// Converts an EdgeProperty to a JSON Value.
fn edge_property_to_json(prop: &EdgeProperty) -> Value {
    match prop {
        EdgeProperty::String(s) => Value::String(s.clone()),
        EdgeProperty::Int(i) => Value::Number((*i).into()),
        EdgeProperty::Float(f) => {
            serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number)
        }
        EdgeProperty::Decimal(d) => Value::String(d.to_string()),
        EdgeProperty::Bool(b) => Value::Bool(*b),
        EdgeProperty::Date(d) => Value::String(d.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EdgeType, NodeType};
    use crate::test_helpers::create_test_graph_with_vendor;
    use tempfile::tempdir;

    #[test]
    fn test_rustgraph_export_jsonl() {
        let graph = create_test_graph_with_vendor();
        let dir = tempdir().unwrap();

        let config = RustGraphExportConfig {
            output_format: RustGraphOutputFormat::JsonLines,
            ..Default::default()
        };
        let exporter = RustGraphExporter::new(config);
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.num_nodes, 3);
        assert_eq!(metadata.num_edges, 2);
        assert_eq!(metadata.anomalous_nodes, 1);
        assert_eq!(metadata.anomalous_edges, 1);
        assert!(dir.path().join("nodes.jsonl").exists());
        assert!(dir.path().join("edges.jsonl").exists());
        assert!(dir.path().join("metadata.json").exists());
    }

    #[test]
    fn test_rustgraph_export_json_array() {
        let graph = create_test_graph_with_vendor();
        let dir = tempdir().unwrap();

        let config = RustGraphExportConfig {
            output_format: RustGraphOutputFormat::JsonArray,
            pretty_print: true,
            ..Default::default()
        };
        let exporter = RustGraphExporter::new(config);
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.num_nodes, 3);
        assert!(dir.path().join("nodes.json").exists());
        assert!(dir.path().join("edges.json").exists());
    }

    #[test]
    fn test_convert_node() {
        let node = GraphNode::new(
            42,
            NodeType::Vendor,
            "V001".to_string(),
            "Test Vendor".to_string(),
        )
        .with_feature(1.0)
        .with_feature(2.0)
        .with_categorical("region", "US")
        .with_label("high_risk");

        let config = RustGraphExportConfig::default();
        let exporter = RustGraphExporter::new(config);
        let output = exporter.convert_node(&node, Utc::now());

        assert_eq!(output.node_type, "Vendor");
        assert_eq!(output.id, "42");
        assert_eq!(
            output.properties.get("external_id"),
            Some(&Value::String("V001".to_string()))
        );
        assert_eq!(
            output.properties.get("region"),
            Some(&Value::String("US".to_string()))
        );
        assert_eq!(output.metadata.features, Some(vec![1.0, 2.0]));
        assert!(!output.metadata.is_anomaly);
    }

    #[test]
    fn test_convert_edge() {
        let edge = GraphEdge::new(99, 1, 2, EdgeType::Approval)
            .with_weight(5000.0)
            .with_feature(0.5)
            .with_timestamp(chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
            .as_anomaly("threshold_breach");

        let config = RustGraphExportConfig::default();
        let exporter = RustGraphExporter::new(config);
        let output = exporter.convert_edge(&edge, Utc::now());

        assert_eq!(output.edge_type, "Approval");
        assert_eq!(output.id, "99");
        assert_eq!(output.source_id, "1");
        assert_eq!(output.target_id, "2");
        assert_eq!(output.metadata.weight, Some(5000.0));
        assert!(output.metadata.is_directed);
        assert!(output.metadata.is_anomaly);
        assert_eq!(
            output.metadata.anomaly_type,
            Some("threshold_breach".to_string())
        );
    }

    #[test]
    fn test_export_to_writer() {
        let graph = create_test_graph_with_vendor();
        let mut buffer = Vec::new();

        let config = RustGraphExportConfig {
            include_features: true,
            include_temporal: true,
            ..Default::default()
        };
        let exporter = RustGraphExporter::new(config);
        let metadata = exporter.export_to_writer(&graph, &mut buffer).unwrap();

        assert_eq!(metadata.num_nodes, 3);
        assert_eq!(metadata.num_edges, 2);

        let output: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        assert!(output.get("nodes").is_some());
        assert!(output.get("edges").is_some());
    }

    #[test]
    fn test_node_type_counts() {
        let graph = create_test_graph_with_vendor();
        let dir = tempdir().unwrap();

        let exporter = RustGraphExporter::new(RustGraphExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.node_type_counts.get("Account"), Some(&2));
        assert_eq!(metadata.node_type_counts.get("Vendor"), Some(&1));
    }

    #[test]
    fn test_without_features() {
        let node = GraphNode::new(1, NodeType::Account, "1000".to_string(), "Cash".to_string())
            .with_feature(1.0);

        let config = RustGraphExportConfig {
            include_features: false,
            ..Default::default()
        };
        let exporter = RustGraphExporter::new(config);
        let output = exporter.convert_node(&node, Utc::now());

        assert!(output.metadata.features.is_none());
    }

    #[test]
    fn test_with_batch_id() {
        let node = GraphNode::new(1, NodeType::Account, "1000".to_string(), "Cash".to_string());

        let config = RustGraphExportConfig {
            batch_id: Some("batch_001".to_string()),
            ..Default::default()
        };
        let exporter = RustGraphExporter::new(config);
        let output = exporter.convert_node(&node, Utc::now());

        assert_eq!(output.metadata.batch_id, Some("batch_001".to_string()));
    }
}
