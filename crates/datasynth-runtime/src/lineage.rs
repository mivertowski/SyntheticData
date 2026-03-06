//! Data lineage graph tracking for generation provenance.
//!
//! Tracks which config sections produced which output files via a directed graph.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A lineage graph tracking data flow from config → generators → output files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageGraph {
    /// All nodes in the lineage graph.
    pub nodes: Vec<LineageNode>,
    /// Directed edges between nodes.
    pub edges: Vec<LineageEdge>,
}

/// A node in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    /// Unique node identifier.
    pub id: String,
    /// Type of node.
    pub node_type: LineageNodeType,
    /// Human-readable label.
    pub label: String,
    /// Additional attributes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

/// Types of nodes in the lineage graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageNodeType {
    /// A configuration section (input).
    ConfigSection,
    /// A generator phase (processing).
    GeneratorPhase,
    /// An output file (output).
    OutputFile,
}

/// A directed edge in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Relationship type.
    pub relationship: LineageRelationship,
}

/// Types of relationships between lineage nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageRelationship {
    /// Config section configures a generator phase.
    ConfiguredBy,
    /// Generator phase produces an output file.
    ProducedBy,
    /// One output is derived from another.
    DerivedFrom,
    /// One output serves as input to a phase.
    InputTo,
}

/// Builder for constructing lineage graphs with a fluent API.
#[derive(Debug, Default)]
pub struct LineageGraphBuilder {
    nodes: Vec<LineageNode>,
    edges: Vec<LineageEdge>,
    node_ids: std::collections::HashSet<String>,
}

impl LineageGraphBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a config section node.
    pub fn add_config_section(&mut self, id: &str, label: &str) -> &mut Self {
        self.add_node(id, LineageNodeType::ConfigSection, label, HashMap::new())
    }

    /// Adds a generator phase node.
    pub fn add_generator_phase(&mut self, id: &str, label: &str) -> &mut Self {
        self.add_node(id, LineageNodeType::GeneratorPhase, label, HashMap::new())
    }

    /// Adds an output file node.
    pub fn add_output_file(&mut self, id: &str, label: &str, path: &str) -> &mut Self {
        let mut attrs = HashMap::new();
        attrs.insert("path".to_string(), path.to_string());
        self.add_node(id, LineageNodeType::OutputFile, label, attrs)
    }

    /// Adds a node with attributes.
    pub fn add_node(
        &mut self,
        id: &str,
        node_type: LineageNodeType,
        label: &str,
        attributes: HashMap<String, String>,
    ) -> &mut Self {
        if self.node_ids.insert(id.to_string()) {
            self.nodes.push(LineageNode {
                id: id.to_string(),
                node_type,
                label: label.to_string(),
                attributes,
            });
        }
        self
    }

    /// Adds a "configured by" edge: config section → generator phase.
    pub fn configured_by(&mut self, generator_id: &str, config_id: &str) -> &mut Self {
        self.add_edge(config_id, generator_id, LineageRelationship::ConfiguredBy)
    }

    /// Adds a "produced by" edge: generator phase → output file.
    pub fn produced_by(&mut self, output_id: &str, generator_id: &str) -> &mut Self {
        self.add_edge(generator_id, output_id, LineageRelationship::ProducedBy)
    }

    /// Adds a "derived from" edge: output → output.
    pub fn derived_from(&mut self, derived_id: &str, source_id: &str) -> &mut Self {
        self.add_edge(source_id, derived_id, LineageRelationship::DerivedFrom)
    }

    /// Adds an "input to" edge: output → generator phase.
    pub fn input_to(&mut self, output_id: &str, phase_id: &str) -> &mut Self {
        self.add_edge(output_id, phase_id, LineageRelationship::InputTo)
    }

    /// Adds an edge.
    pub fn add_edge(
        &mut self,
        source: &str,
        target: &str,
        relationship: LineageRelationship,
    ) -> &mut Self {
        self.edges.push(LineageEdge {
            source: source.to_string(),
            target: target.to_string(),
            relationship,
        });
        self
    }

    /// Builds the lineage graph.
    pub fn build(self) -> LineageGraph {
        LineageGraph {
            nodes: self.nodes,
            edges: self.edges,
        }
    }
}

impl LineageGraph {
    /// Serializes the lineage graph to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Exports the lineage graph in DOT (Graphviz) format.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph lineage {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Define node styles by type
        for node in &self.nodes {
            let (shape, color) = match node.node_type {
                LineageNodeType::ConfigSection => ("note", "lightblue"),
                LineageNodeType::GeneratorPhase => ("component", "lightyellow"),
                LineageNodeType::OutputFile => ("folder", "lightgreen"),
            };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\" shape={} style=filled fillcolor={}];\n",
                node.id, node.label, shape, color
            ));
        }

        dot.push('\n');

        // Define edges
        for edge in &self.edges {
            let label = match edge.relationship {
                LineageRelationship::ConfiguredBy => "configures",
                LineageRelationship::ProducedBy => "produces",
                LineageRelationship::DerivedFrom => "derives",
                LineageRelationship::InputTo => "input_to",
            };
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.source, edge.target, label
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Builds a standard lineage graph for an enhanced generation run.
pub fn build_generation_lineage(
    config_sections: &[&str],
    phases: &[(&str, &str)],
    output_files: &[(&str, &str, &str)],
    phase_config_map: &[(&str, &str)],
    phase_output_map: &[(&str, &str)],
) -> LineageGraph {
    let mut builder = LineageGraphBuilder::new();

    for section in config_sections {
        builder.add_config_section(&format!("config:{section}"), &format!("Config: {section}"));
    }

    for (id, label) in phases {
        builder.add_generator_phase(&format!("phase:{id}"), label);
    }

    for (id, label, path) in output_files {
        builder.add_output_file(&format!("output:{id}"), label, path);
    }

    for (phase, config) in phase_config_map {
        builder.configured_by(&format!("phase:{phase}"), &format!("config:{config}"));
    }

    for (phase, output) in phase_output_map {
        builder.produced_by(&format!("output:{output}"), &format!("phase:{phase}"));
    }

    builder.build()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let mut builder = LineageGraphBuilder::new();
        builder
            .add_config_section("cfg:global", "Global Config")
            .add_generator_phase("gen:coa", "CoA Generator")
            .add_output_file("out:coa", "Chart of Accounts", "chart_of_accounts.csv")
            .configured_by("gen:coa", "cfg:global")
            .produced_by("out:coa", "gen:coa");

        let graph = builder.build();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_no_duplicate_nodes() {
        let mut builder = LineageGraphBuilder::new();
        builder
            .add_config_section("cfg:global", "Global Config")
            .add_config_section("cfg:global", "Global Config Again");

        let graph = builder.build();
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut builder = LineageGraphBuilder::new();
        builder
            .add_config_section("cfg:global", "Global Config")
            .add_generator_phase("gen:coa", "CoA Generator")
            .add_output_file("out:coa", "Chart of Accounts", "chart_of_accounts.csv")
            .configured_by("gen:coa", "cfg:global")
            .produced_by("out:coa", "gen:coa");

        let graph = builder.build();
        let json = graph.to_json().expect("serialize");
        let deserialized: LineageGraph = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.node_count(), graph.node_count());
        assert_eq!(deserialized.edge_count(), graph.edge_count());
    }

    #[test]
    fn test_dot_output() {
        let mut builder = LineageGraphBuilder::new();
        builder
            .add_config_section("cfg:global", "Global Config")
            .add_generator_phase("gen:coa", "CoA Generator")
            .configured_by("gen:coa", "cfg:global");

        let graph = builder.build();
        let dot = graph.to_dot();

        assert!(dot.starts_with("digraph lineage {"));
        assert!(dot.contains("cfg:global"));
        assert!(dot.contains("gen:coa"));
        assert!(dot.contains("configures"));
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_build_generation_lineage() {
        let graph = build_generation_lineage(
            &["global", "transactions"],
            &[("coa", "CoA Generation"), ("je", "Journal Entries")],
            &[
                ("coa_csv", "CoA CSV", "chart_of_accounts.csv"),
                ("je_csv", "JE CSV", "journal_entries.csv"),
            ],
            &[("coa", "global"), ("je", "transactions")],
            &[("coa", "coa_csv"), ("je", "je_csv")],
        );

        assert_eq!(graph.node_count(), 6); // 2 config + 2 phase + 2 output
        assert_eq!(graph.edge_count(), 4); // 2 configured_by + 2 produced_by
    }

    #[test]
    fn test_derived_from_edge() {
        let mut builder = LineageGraphBuilder::new();
        builder
            .add_output_file("out:raw", "Raw Data", "raw.csv")
            .add_output_file("out:agg", "Aggregated", "aggregated.csv")
            .derived_from("out:agg", "out:raw");

        let graph = builder.build();
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            graph.edges[0].relationship,
            LineageRelationship::DerivedFrom
        );
    }
}
