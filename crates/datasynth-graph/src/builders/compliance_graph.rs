//! Compliance graph builder.
//!
//! Builds a graph layer with compliance-specific node and edge types:
//!
//! **Node types** (via `Custom`):
//! - `Standard`: A compliance standard (IFRS 16, SOX 404, ISA 315, etc.)
//! - `Jurisdiction`: A country jurisdiction profile
//! - `AuditProcedure`: An audit procedure instance
//! - `Finding`: A compliance finding
//! - `Filing`: A regulatory filing
//!
//! **Edge types** (via `Custom`):
//! - `MapsToStandard`: Jurisdiction → Standard (mandatory mapping)
//! - `CrossReference`: Standard ↔ Standard (convergence, related, etc.)
//! - `Supersedes`: Standard → Standard (temporal supersession)
//! - `TestsCompliance`: AuditProcedure → Standard
//! - `FindingOnStandard`: Finding → Standard
//! - `FilingForJurisdiction`: Filing → Jurisdiction

use std::collections::HashMap;

use crate::models::{EdgeType, Graph, GraphEdge, GraphNode, GraphType, NodeId, NodeType};

/// Configuration for compliance graph building.
#[derive(Debug, Clone)]
pub struct ComplianceGraphConfig {
    /// Include compliance standard nodes.
    pub include_standard_nodes: bool,
    /// Include jurisdiction nodes.
    pub include_jurisdiction_nodes: bool,
    /// Include cross-reference edges between standards.
    pub include_cross_references: bool,
    /// Include supersession edges.
    pub include_supersession_edges: bool,
}

impl Default for ComplianceGraphConfig {
    fn default() -> Self {
        Self {
            include_standard_nodes: true,
            include_jurisdiction_nodes: true,
            include_cross_references: true,
            include_supersession_edges: false,
        }
    }
}

/// Input data for a compliance standard node.
#[derive(Debug, Clone)]
pub struct StandardNodeInput {
    pub standard_id: String,
    pub title: String,
    pub category: String,
    pub domain: String,
    pub is_active: bool,
    /// ML features: [category_code, domain_code, is_active, convergence_avg]
    pub features: Vec<f64>,
}

/// Input data for a jurisdiction node.
#[derive(Debug, Clone)]
pub struct JurisdictionNodeInput {
    pub country_code: String,
    pub country_name: String,
    pub framework: String,
    pub standard_count: usize,
    pub tax_rate: f64,
}

/// Input data for a cross-reference edge.
#[derive(Debug, Clone)]
pub struct CrossReferenceEdgeInput {
    pub from_standard: String,
    pub to_standard: String,
    pub relationship: String,
    pub convergence_level: f64,
}

/// Input data for a supersession edge.
#[derive(Debug, Clone)]
pub struct SupersessionEdgeInput {
    pub old_standard: String,
    pub new_standard: String,
}

/// Input data for a jurisdiction→standard mapping edge.
#[derive(Debug, Clone)]
pub struct JurisdictionMappingInput {
    pub country_code: String,
    pub standard_id: String,
}

/// Input data for an audit procedure node.
#[derive(Debug, Clone)]
pub struct ProcedureNodeInput {
    pub procedure_id: String,
    pub standard_id: String,
    pub procedure_type: String,
    pub sample_size: u32,
    pub confidence_level: f64,
}

/// Input data for a finding node.
#[derive(Debug, Clone)]
pub struct FindingNodeInput {
    pub finding_id: String,
    pub standard_id: String,
    pub severity: String,
    pub deficiency_level: String,
    pub severity_score: f64,
}

/// Builder for compliance regulatory graphs.
pub struct ComplianceGraphBuilder {
    config: ComplianceGraphConfig,
    graph: Graph,
    /// Map from standard_id to node ID.
    standard_nodes: HashMap<String, NodeId>,
    /// Map from country_code to node ID.
    jurisdiction_nodes: HashMap<String, NodeId>,
    /// Map from procedure_id to node ID.
    procedure_nodes: HashMap<String, NodeId>,
    /// Map from finding_id to node ID.
    finding_nodes: HashMap<String, NodeId>,
}

impl ComplianceGraphBuilder {
    /// Creates a new compliance graph builder.
    pub fn new(config: ComplianceGraphConfig) -> Self {
        Self {
            config,
            graph: Graph::new(
                "compliance_regulation_network",
                GraphType::Custom("ComplianceRegulation".to_string()),
            ),
            standard_nodes: HashMap::new(),
            jurisdiction_nodes: HashMap::new(),
            procedure_nodes: HashMap::new(),
            finding_nodes: HashMap::new(),
        }
    }

    /// Adds standard nodes.
    pub fn add_standards(&mut self, standards: &[StandardNodeInput]) {
        if !self.config.include_standard_nodes {
            return;
        }

        for std in standards {
            if self.standard_nodes.contains_key(&std.standard_id) {
                continue;
            }

            let node = GraphNode::new(
                0,
                NodeType::Custom("Standard".to_string()),
                std.standard_id.clone(),
                std.title.clone(),
            )
            .with_features(std.features.clone())
            .with_categorical("category", &std.category)
            .with_categorical("domain", &std.domain)
            .with_categorical("is_active", if std.is_active { "true" } else { "false" });

            let id = self.graph.add_node(node);
            self.standard_nodes.insert(std.standard_id.clone(), id);
        }
    }

    /// Adds jurisdiction nodes.
    pub fn add_jurisdictions(&mut self, jurisdictions: &[JurisdictionNodeInput]) {
        if !self.config.include_jurisdiction_nodes {
            return;
        }

        for jp in jurisdictions {
            if self.jurisdiction_nodes.contains_key(&jp.country_code) {
                continue;
            }

            let node = GraphNode::new(
                0,
                NodeType::Custom("Jurisdiction".to_string()),
                jp.country_code.clone(),
                jp.country_name.clone(),
            )
            .with_feature(jp.standard_count as f64)
            .with_feature(jp.tax_rate)
            .with_categorical("framework", &jp.framework);

            let id = self.graph.add_node(node);
            self.jurisdiction_nodes.insert(jp.country_code.clone(), id);
        }
    }

    /// Adds cross-reference edges between standards.
    pub fn add_cross_references(&mut self, xrefs: &[CrossReferenceEdgeInput]) {
        if !self.config.include_cross_references {
            return;
        }

        for xref in xrefs {
            if let (Some(&from_id), Some(&to_id)) = (
                self.standard_nodes.get(&xref.from_standard),
                self.standard_nodes.get(&xref.to_standard),
            ) {
                let edge = GraphEdge::new(
                    0,
                    from_id,
                    to_id,
                    EdgeType::Custom(format!("CrossReference:{}", xref.relationship)),
                )
                .with_weight(xref.convergence_level)
                .with_feature(xref.convergence_level);

                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds supersession edges.
    pub fn add_supersessions(&mut self, supersessions: &[SupersessionEdgeInput]) {
        if !self.config.include_supersession_edges {
            return;
        }

        for sup in supersessions {
            if let (Some(&old_id), Some(&new_id)) = (
                self.standard_nodes.get(&sup.old_standard),
                self.standard_nodes.get(&sup.new_standard),
            ) {
                let edge = GraphEdge::new(
                    0,
                    old_id,
                    new_id,
                    EdgeType::Custom("Supersedes".to_string()),
                )
                .with_weight(1.0);

                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds jurisdiction→standard mapping edges.
    pub fn add_jurisdiction_mappings(&mut self, mappings: &[JurisdictionMappingInput]) {
        for mapping in mappings {
            if let (Some(&jp_id), Some(&std_id)) = (
                self.jurisdiction_nodes.get(&mapping.country_code),
                self.standard_nodes.get(&mapping.standard_id),
            ) {
                let edge = GraphEdge::new(
                    0,
                    jp_id,
                    std_id,
                    EdgeType::Custom("MapsToStandard".to_string()),
                )
                .with_weight(1.0);

                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds audit procedure nodes and links them to standards.
    pub fn add_procedures(&mut self, procedures: &[ProcedureNodeInput]) {
        for proc in procedures {
            if self.procedure_nodes.contains_key(&proc.procedure_id) {
                continue;
            }

            let node = GraphNode::new(
                0,
                NodeType::Custom("AuditProcedure".to_string()),
                proc.procedure_id.clone(),
                format!("{} [{}]", proc.procedure_type, proc.standard_id),
            )
            .with_feature(proc.sample_size as f64)
            .with_feature(proc.confidence_level)
            .with_categorical("procedure_type", &proc.procedure_type);

            let proc_node_id = self.graph.add_node(node);
            self.procedure_nodes
                .insert(proc.procedure_id.clone(), proc_node_id);

            // Link to standard
            if let Some(&std_id) = self.standard_nodes.get(&proc.standard_id) {
                let edge = GraphEdge::new(
                    0,
                    proc_node_id,
                    std_id,
                    EdgeType::Custom("TestsCompliance".to_string()),
                )
                .with_weight(1.0);

                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds finding nodes and links them to standards.
    pub fn add_findings(&mut self, findings: &[FindingNodeInput]) {
        for finding in findings {
            if self.finding_nodes.contains_key(&finding.finding_id) {
                continue;
            }

            let node = GraphNode::new(
                0,
                NodeType::Custom("Finding".to_string()),
                finding.finding_id.clone(),
                format!("{} [{}]", finding.deficiency_level, finding.standard_id),
            )
            .with_feature(finding.severity_score)
            .with_categorical("severity", &finding.severity)
            .with_categorical("deficiency_level", &finding.deficiency_level);

            let finding_node_id = self.graph.add_node(node);
            self.finding_nodes
                .insert(finding.finding_id.clone(), finding_node_id);

            // Link to standard
            if let Some(&std_id) = self.standard_nodes.get(&finding.standard_id) {
                let edge = GraphEdge::new(
                    0,
                    finding_node_id,
                    std_id,
                    EdgeType::Custom("FindingOnStandard".to_string()),
                )
                .with_weight(finding.severity_score);

                self.graph.add_edge(edge);
            }
        }
    }

    /// Consumes the builder and returns the built graph.
    pub fn build(mut self) -> Graph {
        self.graph.metadata.node_count = self.graph.nodes.len();
        self.graph.metadata.edge_count = self.graph.edges.len();
        self.graph
    }

    /// Returns the number of standard nodes.
    pub fn standard_count(&self) -> usize {
        self.standard_nodes.len()
    }

    /// Returns the number of jurisdiction nodes.
    pub fn jurisdiction_count(&self) -> usize {
        self.jurisdiction_nodes.len()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_graph_builder() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_standards(&[
            StandardNodeInput {
                standard_id: "IFRS-16".to_string(),
                title: "Leases".to_string(),
                category: "AccountingStandard".to_string(),
                domain: "FinancialReporting".to_string(),
                is_active: true,
                features: vec![0.0, 0.0, 1.0, 0.85],
            },
            StandardNodeInput {
                standard_id: "ASC-842".to_string(),
                title: "Leases".to_string(),
                category: "AccountingStandard".to_string(),
                domain: "FinancialReporting".to_string(),
                is_active: true,
                features: vec![0.0, 0.0, 1.0, 0.85],
            },
        ]);

        builder.add_jurisdictions(&[JurisdictionNodeInput {
            country_code: "US".to_string(),
            country_name: "United States".to_string(),
            framework: "UsGaap".to_string(),
            standard_count: 25,
            tax_rate: 0.21,
        }]);

        builder.add_cross_references(&[CrossReferenceEdgeInput {
            from_standard: "IFRS-16".to_string(),
            to_standard: "ASC-842".to_string(),
            relationship: "Related".to_string(),
            convergence_level: 0.6,
        }]);

        builder.add_jurisdiction_mappings(&[JurisdictionMappingInput {
            country_code: "US".to_string(),
            standard_id: "ASC-842".to_string(),
        }]);

        let graph = builder.build();
        assert_eq!(graph.nodes.len(), 3); // 2 standards + 1 jurisdiction
        assert_eq!(graph.edges.len(), 2); // 1 cross-ref + 1 jurisdiction mapping
    }
}
