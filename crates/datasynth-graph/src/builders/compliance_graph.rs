//! Compliance graph builder.
//!
//! Builds a graph layer with compliance-specific node and edge types, including
//! cross-domain edges that link compliance standards to GL accounts, internal
//! controls, and company entities.
//!
//! **Node types** (via `Custom`):
//! - `Standard`: A compliance standard (IFRS 16, SOX 404, ISA 315, etc.)
//! - `Jurisdiction`: A country jurisdiction profile
//! - `AuditProcedure`: An audit procedure instance
//! - `Finding`: A compliance finding
//! - `Filing`: A regulatory filing
//! - `Account`: A GL account (cross-domain from accounting layer)
//! - `Control`: An internal control (cross-domain from governance layer)
//! - `Company`: A company entity (cross-domain from entity layer)
//!
//! **Edge types** (via `Custom`):
//! - `MapsToStandard`: Jurisdiction → Standard (mandatory mapping)
//! - `CrossReference`: Standard ↔ Standard (convergence, related, etc.)
//! - `Supersedes`: Standard → Standard (temporal supersession)
//! - `TestsCompliance`: AuditProcedure → Standard
//! - `FindingOnStandard`: Finding → Standard
//! - `FilingForJurisdiction`: Filing → Jurisdiction
//! - `GovernedByStandard`: Standard → Account (standard governs account treatment)
//! - `ImplementsStandard`: Control → Standard (control implements standard)
//! - `FiledByCompany`: Filing → Company
//! - `FindingAffectsControl`: Finding → Control
//! - `FindingAffectsAccount`: Finding → Account

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
    /// Include edges linking standards to GL accounts.
    pub include_account_links: bool,
    /// Include edges linking standards to internal controls.
    pub include_control_links: bool,
    /// Include edges linking filings to companies.
    pub include_company_links: bool,
}

impl Default for ComplianceGraphConfig {
    fn default() -> Self {
        Self {
            include_standard_nodes: true,
            include_jurisdiction_nodes: true,
            include_cross_references: true,
            include_supersession_edges: false,
            include_account_links: true,
            include_control_links: true,
            include_company_links: true,
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
    /// GL account types this standard applies to.
    pub applicable_account_types: Vec<String>,
    /// Business processes this standard governs.
    pub applicable_processes: Vec<String>,
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
    /// Control ID where finding was identified (for cross-domain linking).
    pub control_id: Option<String>,
    /// Account codes affected by this finding.
    pub affected_accounts: Vec<String>,
}

/// Input data for linking a standard to a GL account.
#[derive(Debug, Clone)]
pub struct AccountLinkInput {
    pub standard_id: String,
    pub account_code: String,
    pub account_name: String,
}

/// Input data for linking a standard to an internal control.
#[derive(Debug, Clone)]
pub struct ControlLinkInput {
    pub standard_id: String,
    pub control_id: String,
    pub control_name: String,
}

/// Input data for a filing node.
#[derive(Debug, Clone)]
pub struct FilingNodeInput {
    pub filing_id: String,
    pub filing_type: String,
    pub company_code: String,
    pub jurisdiction: String,
    pub status: String,
}

/// Builder for compliance regulatory graphs with cross-domain edges.
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
    /// Map from account_code to node ID (cross-domain).
    account_nodes: HashMap<String, NodeId>,
    /// Map from control_id to node ID (cross-domain).
    control_nodes: HashMap<String, NodeId>,
    /// Map from filing_id to node ID.
    filing_nodes: HashMap<String, NodeId>,
    /// Map from company_code to node ID (cross-domain).
    company_nodes: HashMap<String, NodeId>,
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
            account_nodes: HashMap::new(),
            control_nodes: HashMap::new(),
            filing_nodes: HashMap::new(),
            company_nodes: HashMap::new(),
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

            let mut node = GraphNode::new(
                0,
                NodeType::Custom("Standard".to_string()),
                std.standard_id.clone(),
                std.title.clone(),
            )
            .with_features(std.features.clone())
            .with_categorical("category", &std.category)
            .with_categorical("domain", &std.domain)
            .with_categorical("is_active", if std.is_active { "true" } else { "false" });

            if !std.applicable_processes.is_empty() {
                node = node
                    .with_categorical("applicable_processes", &std.applicable_processes.join(";"));
            }
            if !std.applicable_account_types.is_empty() {
                node = node.with_categorical(
                    "applicable_account_types",
                    &std.applicable_account_types.join(";"),
                );
            }

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

    /// Adds finding nodes and links them to standards, controls, and accounts.
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

            // Cross-domain: Finding → Control
            if let Some(ref ctrl_id) = finding.control_id {
                if let Some(&ctrl_node) = self.control_nodes.get(ctrl_id) {
                    let edge = GraphEdge::new(
                        0,
                        finding_node_id,
                        ctrl_node,
                        EdgeType::Custom("FindingAffectsControl".to_string()),
                    )
                    .with_weight(finding.severity_score);
                    self.graph.add_edge(edge);
                }
            }

            // Cross-domain: Finding → affected Accounts
            if self.config.include_account_links {
                for acct_code in &finding.affected_accounts {
                    if let Some(&acct_node) = self.account_nodes.get(acct_code) {
                        let edge = GraphEdge::new(
                            0,
                            finding_node_id,
                            acct_node,
                            EdgeType::Custom("FindingAffectsAccount".to_string()),
                        )
                        .with_weight(finding.severity_score);
                        self.graph.add_edge(edge);
                    }
                }
            }
        }
    }

    /// Adds GL account nodes and creates `GovernedByStandard` edges from standards.
    pub fn add_account_links(&mut self, links: &[AccountLinkInput]) {
        if !self.config.include_account_links {
            return;
        }

        for link in links {
            // Ensure account node exists
            let acct_id = *self
                .account_nodes
                .entry(link.account_code.clone())
                .or_insert_with(|| {
                    let node = GraphNode::new(
                        0,
                        NodeType::Account,
                        link.account_code.clone(),
                        link.account_name.clone(),
                    );
                    self.graph.add_node(node)
                });

            // Standard → Account edge
            if let Some(&std_id) = self.standard_nodes.get(&link.standard_id) {
                let edge = GraphEdge::new(
                    0,
                    std_id,
                    acct_id,
                    EdgeType::Custom("GovernedByStandard".to_string()),
                )
                .with_weight(1.0);
                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds internal control nodes and creates `ImplementsStandard` edges.
    pub fn add_control_links(&mut self, links: &[ControlLinkInput]) {
        if !self.config.include_control_links {
            return;
        }

        for link in links {
            // Ensure control node exists
            let ctrl_id = *self
                .control_nodes
                .entry(link.control_id.clone())
                .or_insert_with(|| {
                    let node = GraphNode::new(
                        0,
                        NodeType::Custom("Control".to_string()),
                        link.control_id.clone(),
                        link.control_name.clone(),
                    );
                    self.graph.add_node(node)
                });

            // Control → Standard edge
            if let Some(&std_id) = self.standard_nodes.get(&link.standard_id) {
                let edge = GraphEdge::new(
                    0,
                    ctrl_id,
                    std_id,
                    EdgeType::Custom("ImplementsStandard".to_string()),
                )
                .with_weight(1.0);
                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds filing nodes with edges to jurisdictions and companies.
    pub fn add_filings(&mut self, filings: &[FilingNodeInput]) {
        for filing in filings {
            if self.filing_nodes.contains_key(&filing.filing_id) {
                continue;
            }

            let node = GraphNode::new(
                0,
                NodeType::Custom("Filing".to_string()),
                filing.filing_id.clone(),
                format!("{} [{}]", filing.filing_type, filing.company_code),
            )
            .with_categorical("filing_type", &filing.filing_type)
            .with_categorical("status", &filing.status);

            let filing_id = self.graph.add_node(node);
            self.filing_nodes
                .insert(filing.filing_id.clone(), filing_id);

            // Filing → Jurisdiction
            if let Some(&jp_id) = self.jurisdiction_nodes.get(&filing.jurisdiction) {
                let edge = GraphEdge::new(
                    0,
                    filing_id,
                    jp_id,
                    EdgeType::Custom("FilingForJurisdiction".to_string()),
                )
                .with_weight(1.0);
                self.graph.add_edge(edge);
            }

            // Filing → Company
            if self.config.include_company_links {
                let company_id = *self
                    .company_nodes
                    .entry(filing.company_code.clone())
                    .or_insert_with(|| {
                        let node = GraphNode::new(
                            0,
                            NodeType::Company,
                            filing.company_code.clone(),
                            filing.company_code.clone(),
                        );
                        self.graph.add_node(node)
                    });

                let edge = GraphEdge::new(
                    0,
                    filing_id,
                    company_id,
                    EdgeType::Custom("FiledByCompany".to_string()),
                )
                .with_weight(1.0);
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

    /// Returns the number of account nodes (cross-domain).
    pub fn account_count(&self) -> usize {
        self.account_nodes.len()
    }

    /// Returns the number of control nodes (cross-domain).
    pub fn control_count(&self) -> usize {
        self.control_nodes.len()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_standard(id: &str, title: &str) -> StandardNodeInput {
        StandardNodeInput {
            standard_id: id.to_string(),
            title: title.to_string(),
            category: "AccountingStandard".to_string(),
            domain: "FinancialReporting".to_string(),
            is_active: true,
            features: vec![0.0, 0.0, 1.0, 0.85],
            applicable_account_types: vec![],
            applicable_processes: vec![],
        }
    }

    #[test]
    fn test_compliance_graph_builder() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_standards(&[
            make_standard("IFRS-16", "Leases"),
            make_standard("ASC-842", "Leases"),
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

    #[test]
    fn test_cross_domain_account_links() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_standards(&[StandardNodeInput {
            standard_id: "IFRS-16".to_string(),
            title: "Leases".to_string(),
            category: "AccountingStandard".to_string(),
            domain: "FinancialReporting".to_string(),
            is_active: true,
            features: vec![1.0],
            applicable_account_types: vec!["Leases".to_string(), "ROUAsset".to_string()],
            applicable_processes: vec!["R2R".to_string()],
        }]);

        builder.add_account_links(&[
            AccountLinkInput {
                standard_id: "IFRS-16".to_string(),
                account_code: "1800".to_string(),
                account_name: "ROU Assets".to_string(),
            },
            AccountLinkInput {
                standard_id: "IFRS-16".to_string(),
                account_code: "2800".to_string(),
                account_name: "Lease Liabilities".to_string(),
            },
        ]);

        let graph = builder.build();
        // 1 standard + 2 accounts
        assert_eq!(graph.nodes.len(), 3);
        // 2 GovernedByStandard edges
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_cross_domain_control_links() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_standards(&[make_standard("SOX-404", "ICFR Assessment")]);

        builder.add_control_links(&[
            ControlLinkInput {
                standard_id: "SOX-404".to_string(),
                control_id: "C010".to_string(),
                control_name: "PO Approval Control".to_string(),
            },
            ControlLinkInput {
                standard_id: "SOX-404".to_string(),
                control_id: "C020".to_string(),
                control_name: "Revenue Recognition Control".to_string(),
            },
        ]);

        let graph = builder.build();
        // 1 standard + 2 controls
        assert_eq!(graph.nodes.len(), 3);
        // 2 ImplementsStandard edges
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_filing_with_company_links() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_jurisdictions(&[JurisdictionNodeInput {
            country_code: "US".to_string(),
            country_name: "United States".to_string(),
            framework: "UsGaap".to_string(),
            standard_count: 25,
            tax_rate: 0.21,
        }]);

        builder.add_filings(&[FilingNodeInput {
            filing_id: "F001".to_string(),
            filing_type: "10-K".to_string(),
            company_code: "C001".to_string(),
            jurisdiction: "US".to_string(),
            status: "Filed".to_string(),
        }]);

        let graph = builder.build();
        // 1 jurisdiction + 1 filing + 1 company
        assert_eq!(graph.nodes.len(), 3);
        // 1 FilingForJurisdiction + 1 FiledByCompany
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_finding_cross_domain_edges() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        builder.add_standards(&[make_standard("SOX-404", "ICFR Assessment")]);

        // Add control and account nodes first
        builder.add_control_links(&[ControlLinkInput {
            standard_id: "SOX-404".to_string(),
            control_id: "C010".to_string(),
            control_name: "PO Approval".to_string(),
        }]);
        builder.add_account_links(&[AccountLinkInput {
            standard_id: "SOX-404".to_string(),
            account_code: "2000".to_string(),
            account_name: "Accounts Payable".to_string(),
        }]);

        // Now add finding that references the control and account
        builder.add_findings(&[FindingNodeInput {
            finding_id: "FIND-001".to_string(),
            standard_id: "SOX-404".to_string(),
            severity: "High".to_string(),
            deficiency_level: "MaterialWeakness".to_string(),
            severity_score: 1.0,
            control_id: Some("C010".to_string()),
            affected_accounts: vec!["2000".to_string()],
        }]);

        let graph = builder.build();
        // 1 standard + 1 control + 1 account + 1 finding = 4
        assert_eq!(graph.nodes.len(), 4);
        // ImplementsStandard + GovernedByStandard + FindingOnStandard
        //   + FindingAffectsControl + FindingAffectsAccount = 5
        assert_eq!(graph.edges.len(), 5);
    }

    #[test]
    fn test_full_traversal_path() {
        let mut builder = ComplianceGraphBuilder::new(ComplianceGraphConfig::default());

        // Build: Company → Filing → Jurisdiction → Standard → Account + Control
        builder.add_standards(&[make_standard("IFRS-15", "Revenue")]);
        builder.add_jurisdictions(&[JurisdictionNodeInput {
            country_code: "DE".to_string(),
            country_name: "Germany".to_string(),
            framework: "LocalGaapWithIfrs".to_string(),
            standard_count: 10,
            tax_rate: 0.30,
        }]);
        builder.add_jurisdiction_mappings(&[JurisdictionMappingInput {
            country_code: "DE".to_string(),
            standard_id: "IFRS-15".to_string(),
        }]);
        builder.add_account_links(&[AccountLinkInput {
            standard_id: "IFRS-15".to_string(),
            account_code: "4000".to_string(),
            account_name: "Revenue".to_string(),
        }]);
        builder.add_control_links(&[ControlLinkInput {
            standard_id: "IFRS-15".to_string(),
            control_id: "C020".to_string(),
            control_name: "Revenue Recognition".to_string(),
        }]);
        builder.add_filings(&[FilingNodeInput {
            filing_id: "F001".to_string(),
            filing_type: "Jahresabschluss".to_string(),
            company_code: "DE01".to_string(),
            jurisdiction: "DE".to_string(),
            status: "Filed".to_string(),
        }]);

        let graph = builder.build();
        // Company(DE01) + Filing(F001) + Jurisdiction(DE) + Standard(IFRS-15)
        //   + Account(4000) + Control(C020) = 6 nodes
        assert_eq!(graph.nodes.len(), 6);
        // MapsToStandard + GovernedByStandard + ImplementsStandard
        //   + FilingForJurisdiction + FiledByCompany = 5 edges
        assert_eq!(graph.edges.len(), 5);
    }
}
