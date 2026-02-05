//! Neo4j exporter.
//!
//! Exports graph data in formats compatible with Neo4j import:
//! - CSV files for nodes and edges (neo4j-admin import format)
//! - Cypher script for direct loading

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::models::Graph;

/// Configuration for Neo4j export.
#[derive(Debug, Clone)]
pub struct Neo4jExportConfig {
    /// Export node properties.
    pub export_node_properties: bool,
    /// Export edge properties.
    pub export_edge_properties: bool,
    /// Export features as properties.
    pub export_features: bool,
    /// Generate Cypher import script.
    pub generate_cypher: bool,
    /// Generate neo4j-admin import script.
    pub generate_admin_import: bool,
    /// Database name for Cypher.
    pub database_name: String,
    /// Batch size for Cypher imports.
    pub cypher_batch_size: usize,
}

impl Default for Neo4jExportConfig {
    fn default() -> Self {
        Self {
            export_node_properties: true,
            export_edge_properties: true,
            export_features: true,
            generate_cypher: true,
            generate_admin_import: true,
            database_name: "synth".to_string(),
            cypher_batch_size: 1000,
        }
    }
}

/// Metadata about the exported Neo4j data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jMetadata {
    /// Graph name.
    pub name: String,
    /// Number of nodes.
    pub num_nodes: usize,
    /// Number of edges.
    pub num_edges: usize,
    /// Node labels (types).
    pub node_labels: Vec<String>,
    /// Relationship types.
    pub relationship_types: Vec<String>,
    /// Files included in export.
    pub files: Vec<String>,
}

/// Neo4j exporter.
pub struct Neo4jExporter {
    config: Neo4jExportConfig,
}

impl Neo4jExporter {
    /// Creates a new Neo4j exporter.
    pub fn new(config: Neo4jExportConfig) -> Self {
        Self { config }
    }

    /// Exports a graph to Neo4j format.
    pub fn export(&self, graph: &Graph, output_dir: &Path) -> std::io::Result<Neo4jMetadata> {
        fs::create_dir_all(output_dir)?;

        let mut files = Vec::new();

        // Export nodes by type
        let node_labels = self.export_nodes(graph, output_dir, &mut files)?;

        // Export edges by type
        let relationship_types = self.export_edges(graph, output_dir, &mut files)?;

        // Generate Cypher script
        if self.config.generate_cypher {
            self.generate_cypher_script(graph, output_dir, &node_labels, &relationship_types)?;
            files.push("import.cypher".to_string());
        }

        // Generate neo4j-admin import script
        if self.config.generate_admin_import {
            self.generate_admin_import_script(output_dir, &node_labels, &relationship_types)?;
            files.push("admin_import.sh".to_string());
        }

        // Create metadata
        let metadata = Neo4jMetadata {
            name: graph.name.clone(),
            num_nodes: graph.node_count(),
            num_edges: graph.edge_count(),
            node_labels,
            relationship_types,
            files,
        };

        // Write metadata
        let metadata_path = output_dir.join("metadata.json");
        let file = File::create(metadata_path)?;
        serde_json::to_writer_pretty(file, &metadata)?;

        Ok(metadata)
    }

    /// Exports nodes grouped by type.
    fn export_nodes(
        &self,
        graph: &Graph,
        output_dir: &Path,
        files: &mut Vec<String>,
    ) -> std::io::Result<Vec<String>> {
        let mut labels = Vec::new();

        for (node_type, node_ids) in &graph.nodes_by_type {
            let label = node_type.as_str();
            labels.push(label.to_string());

            let filename = format!("nodes_{}.csv", label.to_lowercase());
            let path = output_dir.join(&filename);
            files.push(filename);

            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);

            // Determine properties from first node
            let sample_node = node_ids.first().and_then(|id| graph.nodes.get(id));

            // Write header
            let mut header = vec![
                "nodeId:ID".to_string(),
                "code".to_string(),
                "name".to_string(),
            ];

            if self.config.export_node_properties {
                if let Some(node) = sample_node {
                    for key in node.properties.keys() {
                        header.push(key.clone());
                    }
                }
            }

            if self.config.export_features {
                if let Some(node) = sample_node {
                    for i in 0..node.features.len() {
                        header.push(format!("feature_{}", i));
                    }
                }
            }

            header.push("isAnomaly:boolean".to_string());
            header.push(":LABEL".to_string());

            writeln!(writer, "{}", header.join(","))?;

            // Write nodes
            for &node_id in node_ids {
                if let Some(node) = graph.nodes.get(&node_id) {
                    let mut row = vec![
                        node_id.to_string(),
                        escape_csv(&node.external_id),
                        escape_csv(&node.label),
                    ];

                    if self.config.export_node_properties {
                        for key in &header[3..] {
                            if key.starts_with("feature_")
                                || key == "isAnomaly:boolean"
                                || key == ":LABEL"
                            {
                                break;
                            }
                            let value = node
                                .properties
                                .get(key)
                                .map(|p| p.to_string_value())
                                .unwrap_or_default();
                            row.push(escape_csv(&value));
                        }
                    }

                    if self.config.export_features {
                        for &feat in &node.features {
                            row.push(format!("{:.6}", feat));
                        }
                    }

                    row.push(node.is_anomaly.to_string());
                    row.push(label.to_string());

                    writeln!(writer, "{}", row.join(","))?;
                }
            }
        }

        Ok(labels)
    }

    /// Exports edges grouped by type.
    fn export_edges(
        &self,
        graph: &Graph,
        output_dir: &Path,
        files: &mut Vec<String>,
    ) -> std::io::Result<Vec<String>> {
        let mut rel_types = Vec::new();

        for (edge_type, edge_ids) in &graph.edges_by_type {
            let rel_type = edge_type.as_str();
            rel_types.push(rel_type.to_string());

            let filename = format!("edges_{}.csv", rel_type.to_lowercase());
            let path = output_dir.join(&filename);
            files.push(filename);

            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);

            // Determine properties from first edge
            let sample_edge = edge_ids.first().and_then(|id| graph.edges.get(id));

            // Write header
            let mut header = vec![
                ":START_ID".to_string(),
                ":END_ID".to_string(),
                "weight:double".to_string(),
            ];

            if self.config.export_edge_properties {
                if let Some(edge) = sample_edge {
                    for key in edge.properties.keys() {
                        header.push(format!("{}:string", key));
                    }
                }
            }

            if self.config.export_features {
                if let Some(edge) = sample_edge {
                    for i in 0..edge.features.len() {
                        header.push(format!("feature_{}:double", i));
                    }
                }
            }

            header.push("isAnomaly:boolean".to_string());
            header.push(":TYPE".to_string());

            writeln!(writer, "{}", header.join(","))?;

            // Write edges
            for &edge_id in edge_ids {
                if let Some(edge) = graph.edges.get(&edge_id) {
                    let mut row = vec![
                        edge.source.to_string(),
                        edge.target.to_string(),
                        format!("{:.6}", edge.weight),
                    ];

                    if self.config.export_edge_properties {
                        for (key, value) in &edge.properties {
                            if !header.iter().any(|h| h.starts_with(key)) {
                                continue;
                            }
                            row.push(escape_csv(&value.to_string_value()));
                        }
                    }

                    if self.config.export_features {
                        for &feat in &edge.features {
                            row.push(format!("{:.6}", feat));
                        }
                    }

                    row.push(edge.is_anomaly.to_string());
                    row.push(rel_type.to_string());

                    writeln!(writer, "{}", row.join(","))?;
                }
            }
        }

        Ok(rel_types)
    }

    /// Generates Cypher import script.
    fn generate_cypher_script(
        &self,
        graph: &Graph,
        output_dir: &Path,
        node_labels: &[String],
        relationship_types: &[String],
    ) -> std::io::Result<()> {
        let path = output_dir.join("import.cypher");
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "// Neo4j Import Script for {}", graph.name)?;
        writeln!(writer, "// Generated by synth-graph")?;
        writeln!(writer)?;

        // Create constraints
        writeln!(writer, "// Create constraints and indexes")?;
        for label in node_labels {
            writeln!(
                writer,
                "CREATE CONSTRAINT IF NOT EXISTS FOR (n:{}) REQUIRE n.nodeId IS UNIQUE;",
                label
            )?;
        }
        writeln!(writer)?;

        // Import nodes using LOAD CSV
        writeln!(writer, "// Import nodes")?;
        for label in node_labels {
            let filename = format!("nodes_{}.csv", label.to_lowercase());
            writeln!(writer, "LOAD CSV WITH HEADERS FROM 'file:///{}'", filename)?;
            writeln!(writer, "AS row")?;
            writeln!(
                writer,
                "CREATE (n:{} {{nodeId: toInteger(row.`nodeId:ID`), code: row.code, name: row.name, isAnomaly: toBoolean(row.`isAnomaly:boolean`)}});",
                label
            )?;
            writeln!(writer)?;
        }

        // Import edges using LOAD CSV
        writeln!(writer, "// Import relationships")?;
        for rel_type in relationship_types {
            let filename = format!("edges_{}.csv", rel_type.to_lowercase());
            writeln!(writer, "LOAD CSV WITH HEADERS FROM 'file:///{}'", filename)?;
            writeln!(writer, "AS row")?;
            writeln!(
                writer,
                "MATCH (source) WHERE source.nodeId = toInteger(row.`:START_ID`)"
            )?;
            writeln!(
                writer,
                "MATCH (target) WHERE target.nodeId = toInteger(row.`:END_ID`)"
            )?;
            writeln!(
                writer,
                "CREATE (source)-[:{}{{weight: toFloat(row.`weight:double`), isAnomaly: toBoolean(row.`isAnomaly:boolean`)}}]->(target);",
                rel_type.to_uppercase().replace("-", "_")
            )?;
            writeln!(writer)?;
        }

        // Summary query
        writeln!(writer, "// Verification query")?;
        writeln!(writer, "CALL db.labels() YIELD label")?;
        writeln!(
            writer,
            "CALL apoc.cypher.run('MATCH (n:`' + label + '`) RETURN count(n) as count', {{}})"
        )?;
        writeln!(writer, "YIELD value")?;
        writeln!(writer, "RETURN label, value.count as nodeCount;")?;

        Ok(())
    }

    /// Generates neo4j-admin import script.
    fn generate_admin_import_script(
        &self,
        output_dir: &Path,
        node_labels: &[String],
        relationship_types: &[String],
    ) -> std::io::Result<()> {
        let path = output_dir.join("admin_import.sh");
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "#!/bin/bash")?;
        writeln!(writer, "# Neo4j Admin Import Script")?;
        writeln!(writer, "# Generated by synth-graph")?;
        writeln!(writer)?;
        writeln!(writer, "# Set Neo4j home directory")?;
        writeln!(writer, "NEO4J_HOME=${{NEO4J_HOME:-/var/lib/neo4j}}")?;
        writeln!(writer, "DATA_DIR=${{DATA_DIR:-$(dirname $0)}}")?;
        writeln!(writer)?;
        writeln!(writer, "# Stop Neo4j if running")?;
        writeln!(writer, "# systemctl stop neo4j")?;
        writeln!(writer)?;
        writeln!(writer, "# Run import")?;
        writeln!(writer, "neo4j-admin database import full \\")?;
        writeln!(writer, "  --overwrite-destination=true \\")?;
        writeln!(writer, "  --database={} \\", self.config.database_name)?;

        // Add node files
        for label in node_labels {
            let filename = format!("nodes_{}.csv", label.to_lowercase());
            writeln!(writer, "  --nodes={}=$DATA_DIR/{} \\", label, filename)?;
        }

        // Add relationship files
        for rel_type in relationship_types {
            let filename = format!("edges_{}.csv", rel_type.to_lowercase());
            writeln!(
                writer,
                "  --relationships={}=$DATA_DIR/{} \\",
                rel_type.to_uppercase().replace("-", "_"),
                filename
            )?;
        }

        writeln!(writer, "  --skip-bad-relationships=true")?;
        writeln!(writer)?;
        writeln!(writer, "echo \"Import complete\"")?;
        writeln!(writer)?;
        writeln!(writer, "# Start Neo4j")?;
        writeln!(writer, "# systemctl start neo4j")?;

        Ok(())
    }
}

/// Escapes a value for CSV format.
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Builder for Cypher queries.
pub struct CypherQueryBuilder {
    queries: Vec<String>,
}

impl CypherQueryBuilder {
    /// Creates a new query builder.
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    /// Adds a node creation query.
    pub fn create_node(&mut self, label: &str, properties: &HashMap<String, String>) -> &mut Self {
        let props: Vec<String> = properties
            .iter()
            .map(|(k, v)| format!("{}: '{}'", k, v.replace('\'', "\\'")))
            .collect();

        self.queries
            .push(format!("CREATE (:{} {{{}}})", label, props.join(", ")));
        self
    }

    /// Adds a relationship creation query.
    pub fn create_relationship(
        &mut self,
        from_label: &str,
        from_id: &str,
        to_label: &str,
        to_id: &str,
        rel_type: &str,
        properties: &HashMap<String, String>,
    ) -> &mut Self {
        let props: Vec<String> = properties
            .iter()
            .map(|(k, v)| format!("{}: '{}'", k, v.replace('\'', "\\'")))
            .collect();

        let props_str = if props.is_empty() {
            String::new()
        } else {
            format!(" {{{}}}", props.join(", "))
        };

        self.queries.push(format!(
            "MATCH (a:{} {{nodeId: '{}'}}), (b:{} {{nodeId: '{}'}}) CREATE (a)-[:{}{}]->(b)",
            from_label, from_id, to_label, to_id, rel_type, props_str
        ));
        self
    }

    /// Builds the final Cypher script.
    pub fn build(&self) -> String {
        self.queries.join(";\n") + ";"
    }
}

impl Default for CypherQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_graph;
    use tempfile::tempdir;

    #[test]
    fn test_neo4j_export() {
        let graph = create_test_graph();
        let dir = tempdir().unwrap();

        let exporter = Neo4jExporter::new(Neo4jExportConfig::default());
        let metadata = exporter.export(&graph, dir.path()).unwrap();

        assert_eq!(metadata.num_nodes, 2);
        assert_eq!(metadata.num_edges, 1);
        assert!(dir.path().join("nodes_account.csv").exists());
        assert!(dir.path().join("edges_transaction.csv").exists());
        assert!(dir.path().join("import.cypher").exists());
        assert!(dir.path().join("admin_import.sh").exists());
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_cypher_builder() {
        let mut builder = CypherQueryBuilder::new();
        let mut props = HashMap::new();
        props.insert("name".to_string(), "Test".to_string());

        builder.create_node("Account", &props);
        let cypher = builder.build();

        assert!(cypher.contains("CREATE (:Account"));
        assert!(cypher.contains("name: 'Test'"));
    }
}
