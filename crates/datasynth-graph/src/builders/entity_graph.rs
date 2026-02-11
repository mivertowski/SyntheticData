//! Entity relationship graph builder.
//!
//! Builds a graph where:
//! - Nodes are legal entities (companies)
//! - Edges are ownership/intercompany relationships

use std::collections::HashMap;

use rust_decimal::Decimal;

use datasynth_core::models::intercompany::IntercompanyRelationship;
use datasynth_core::models::Company;

use crate::models::{CompanyNode, EdgeType, Graph, GraphEdge, GraphType, NodeId, OwnershipEdge};

/// Configuration for entity graph building.
#[derive(Debug, Clone)]
pub struct EntityGraphConfig {
    /// Whether to include intercompany transaction edges.
    pub include_intercompany_edges: bool,
    /// Whether to compute consolidation path weights.
    pub compute_consolidation_weights: bool,
    /// Minimum ownership percentage to include.
    pub min_ownership_percent: Decimal,
    /// Whether to include indirect ownership paths.
    pub include_indirect_ownership: bool,
}

impl Default for EntityGraphConfig {
    fn default() -> Self {
        Self {
            include_intercompany_edges: true,
            compute_consolidation_weights: true,
            min_ownership_percent: Decimal::ZERO,
            include_indirect_ownership: true,
        }
    }
}

/// Builder for entity relationship graphs.
pub struct EntityGraphBuilder {
    config: EntityGraphConfig,
    graph: Graph,
    /// Map from company code to node ID.
    company_nodes: HashMap<String, NodeId>,
    /// Ownership relationships for indirect computation.
    ownership_edges: Vec<(String, String, Decimal)>,
}

impl EntityGraphBuilder {
    /// Creates a new entity graph builder.
    pub fn new(config: EntityGraphConfig) -> Self {
        Self {
            config,
            graph: Graph::new("entity_network", GraphType::EntityRelationship),
            company_nodes: HashMap::new(),
            ownership_edges: Vec::new(),
        }
    }

    /// Adds companies to the graph.
    pub fn add_companies(&mut self, companies: &[Company]) {
        for company in companies {
            self.get_or_create_company_node(company);
        }
    }

    /// Adds ownership relationships to the graph.
    pub fn add_ownership_relationships(&mut self, relationships: &[IntercompanyRelationship]) {
        for rel in relationships {
            if rel.ownership_percentage < self.config.min_ownership_percent {
                continue;
            }

            let parent_id = self.ensure_company_node(&rel.parent_company, &rel.parent_company);
            let subsidiary_id =
                self.ensure_company_node(&rel.subsidiary_company, &rel.subsidiary_company);

            // Store for indirect computation
            self.ownership_edges.push((
                rel.parent_company.clone(),
                rel.subsidiary_company.clone(),
                rel.ownership_percentage,
            ));

            // Create ownership edge
            let mut edge = OwnershipEdge::new(
                0,
                parent_id,
                subsidiary_id,
                rel.ownership_percentage,
                rel.effective_date,
            );
            edge.parent_code = rel.parent_company.clone();
            edge.subsidiary_code = rel.subsidiary_company.clone();
            edge.consolidation_method = rel.consolidation_method.as_str().to_string();
            edge.compute_features();

            self.graph.add_edge(edge.edge);
        }
    }

    /// Adds an intercompany transaction edge.
    pub fn add_intercompany_edge(
        &mut self,
        from_company: &str,
        to_company: &str,
        amount: Decimal,
        transaction_type: &str,
    ) {
        if !self.config.include_intercompany_edges {
            return;
        }

        let from_id = self.ensure_company_node(from_company, from_company);
        let to_id = self.ensure_company_node(to_company, to_company);

        let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
        let edge = GraphEdge::new(0, from_id, to_id, EdgeType::Intercompany)
            .with_weight(amount_f64)
            .with_feature((amount_f64.abs() + 1.0).ln())
            .with_feature(Self::encode_transaction_type(transaction_type));

        self.graph.add_edge(edge);
    }

    /// Gets or creates a company node from a Company struct.
    fn get_or_create_company_node(&mut self, company: &Company) -> NodeId {
        if let Some(&id) = self.company_nodes.get(&company.company_code) {
            return id;
        }

        let mut company_node = CompanyNode::new(
            0,
            company.company_code.clone(),
            company.company_name.clone(),
        );
        company_node.country = company.country.clone();
        company_node.currency = company.local_currency.clone();
        company_node.is_parent = company.is_parent;
        company_node.parent_code = company.parent_company.clone();
        company_node.ownership_percent = company.ownership_percentage;
        company_node.compute_features();

        let id = self.graph.add_node(company_node.node);
        self.company_nodes.insert(company.company_code.clone(), id);
        id
    }

    /// Ensures a company node exists.
    fn ensure_company_node(&mut self, company_code: &str, company_name: &str) -> NodeId {
        if let Some(&id) = self.company_nodes.get(company_code) {
            return id;
        }

        let mut company_node =
            CompanyNode::new(0, company_code.to_string(), company_name.to_string());
        company_node.compute_features();

        let id = self.graph.add_node(company_node.node);
        self.company_nodes.insert(company_code.to_string(), id);
        id
    }

    /// Encodes transaction type as numeric feature.
    fn encode_transaction_type(transaction_type: &str) -> f64 {
        match transaction_type {
            "GoodsSale" => 1.0,
            "ServiceProvided" => 2.0,
            "Loan" => 3.0,
            "Dividend" => 4.0,
            "ManagementFee" => 5.0,
            "Royalty" => 6.0,
            "CostSharing" => 7.0,
            _ => 0.0,
        }
    }

    /// Computes indirect ownership percentages.
    fn compute_indirect_ownership(&self) -> HashMap<(String, String), Decimal> {
        let mut indirect: HashMap<(String, String), Decimal> = HashMap::new();

        // Start with direct ownership
        for (parent, subsidiary, pct) in &self.ownership_edges {
            indirect.insert((parent.clone(), subsidiary.clone()), *pct);
        }

        // Iterate to find transitive ownership (simple BFS approach)
        let mut changed = true;
        let max_iterations = 10;
        let mut iteration = 0;

        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;

            let current_indirect: Vec<_> = indirect.iter().map(|(k, v)| (k.clone(), *v)).collect();

            for ((parent, subsidiary), pct) in &current_indirect {
                // Find all entities owned by subsidiary
                for (child_parent, child_sub, child_pct) in &self.ownership_edges {
                    if child_parent == subsidiary {
                        let key = (parent.clone(), child_sub.clone());
                        let indirect_pct = *pct * *child_pct / Decimal::ONE_HUNDRED;

                        if let Some(existing) = indirect.get(&key) {
                            if indirect_pct > *existing {
                                indirect.insert(key, indirect_pct);
                                changed = true;
                            }
                        } else {
                            indirect.insert(key, indirect_pct);
                            changed = true;
                        }
                    }
                }
            }
        }

        indirect
    }

    /// Adds indirect ownership edges to the graph.
    pub fn add_indirect_ownership_edges(&mut self) {
        if !self.config.include_indirect_ownership {
            return;
        }

        let indirect = self.compute_indirect_ownership();

        // Direct ownership keys
        let direct: std::collections::HashSet<_> = self
            .ownership_edges
            .iter()
            .map(|(p, s, _)| (p.clone(), s.clone()))
            .collect();

        // Add edges for indirect-only relationships
        for ((parent, subsidiary), pct) in indirect {
            if direct.contains(&(parent.clone(), subsidiary.clone())) {
                continue; // Skip direct ownership (already added)
            }

            if pct < self.config.min_ownership_percent {
                continue;
            }

            if let (Some(&parent_id), Some(&sub_id)) = (
                self.company_nodes.get(&parent),
                self.company_nodes.get(&subsidiary),
            ) {
                let pct_f64: f64 = pct.try_into().unwrap_or(0.0);
                let edge = GraphEdge::new(0, parent_id, sub_id, EdgeType::Ownership)
                    .with_weight(pct_f64)
                    .with_feature(pct_f64 / 100.0)
                    .with_feature(1.0); // Mark as indirect

                self.graph.add_edge(edge);
            }
        }
    }

    /// Builds the final graph.
    pub fn build(mut self) -> Graph {
        // Add indirect ownership edges if configured
        if self.config.include_indirect_ownership {
            self.add_indirect_ownership_edges();
        }

        self.graph.compute_statistics();
        self.graph
    }

    /// Returns the company code to node ID mapping.
    pub fn company_node_map(&self) -> &HashMap<String, NodeId> {
        &self.company_nodes
    }
}

/// Builds a consolidated ownership hierarchy.
#[derive(Debug, Clone)]
pub struct OwnershipHierarchy {
    /// Root company code.
    pub root: String,
    /// Children with ownership percentage.
    pub children: Vec<OwnershipHierarchyNode>,
}

/// Node in ownership hierarchy.
#[derive(Debug, Clone)]
pub struct OwnershipHierarchyNode {
    /// Company code.
    pub company_code: String,
    /// Direct ownership percentage from parent.
    pub direct_ownership: Decimal,
    /// Effective ownership from root.
    pub effective_ownership: Decimal,
    /// Children.
    pub children: Vec<OwnershipHierarchyNode>,
}

impl OwnershipHierarchy {
    /// Builds hierarchy from relationships.
    pub fn from_relationships(root: &str, relationships: &[IntercompanyRelationship]) -> Self {
        let children = Self::build_children(root, Decimal::ONE_HUNDRED, relationships);
        Self {
            root: root.to_string(),
            children,
        }
    }

    fn build_children(
        parent: &str,
        parent_effective: Decimal,
        relationships: &[IntercompanyRelationship],
    ) -> Vec<OwnershipHierarchyNode> {
        let mut children = Vec::new();

        for rel in relationships {
            if rel.parent_company == parent {
                let effective = parent_effective * rel.ownership_percentage / Decimal::ONE_HUNDRED;
                let grandchildren =
                    Self::build_children(&rel.subsidiary_company, effective, relationships);

                children.push(OwnershipHierarchyNode {
                    company_code: rel.subsidiary_company.clone(),
                    direct_ownership: rel.ownership_percentage,
                    effective_ownership: effective,
                    children: grandchildren,
                });
            }
        }

        children
    }

    /// Returns all companies with effective ownership.
    pub fn all_companies(&self) -> Vec<(String, Decimal)> {
        let mut result = vec![(self.root.clone(), Decimal::ONE_HUNDRED)];
        Self::collect_companies(&self.children, &mut result);
        result
    }

    fn collect_companies(nodes: &[OwnershipHierarchyNode], result: &mut Vec<(String, Decimal)>) {
        for node in nodes {
            result.push((node.company_code.clone(), node.effective_ownership));
            Self::collect_companies(&node.children, result);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::intercompany::ConsolidationMethod;
    use rust_decimal_macros::dec;

    fn create_test_relationship(
        parent: &str,
        subsidiary: &str,
        pct: Decimal,
    ) -> IntercompanyRelationship {
        IntercompanyRelationship {
            relationship_id: format!("REL-{}-{}", parent, subsidiary),
            parent_company: parent.to_string(),
            subsidiary_company: subsidiary.to_string(),
            ownership_percentage: pct,
            consolidation_method: ConsolidationMethod::Full,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            transfer_pricing_policy: None,
            holding_type: datasynth_core::models::intercompany::HoldingType::Direct,
            functional_currency: "USD".to_string(),
            requires_elimination: true,
            reporting_segment: None,
        }
    }

    #[test]
    fn test_entity_graph() {
        let mut builder = EntityGraphBuilder::new(EntityGraphConfig::default());

        let relationships = vec![
            create_test_relationship("1000", "1100", dec!(100)),
            create_test_relationship("1000", "1200", dec!(100)),
            create_test_relationship("1100", "1110", dec!(80)),
        ];

        builder.add_ownership_relationships(&relationships);

        let graph = builder.build();

        assert_eq!(graph.node_count(), 4); // 1000, 1100, 1200, 1110
        assert!(graph.edge_count() >= 3); // Direct + indirect edges
    }

    #[test]
    fn test_ownership_hierarchy() {
        let relationships = vec![
            create_test_relationship("HQ", "US", dec!(100)),
            create_test_relationship("HQ", "EU", dec!(100)),
            create_test_relationship("US", "US-WEST", dec!(100)),
            create_test_relationship("EU", "DE", dec!(80)),
        ];

        let hierarchy = OwnershipHierarchy::from_relationships("HQ", &relationships);

        assert_eq!(hierarchy.root, "HQ");
        assert_eq!(hierarchy.children.len(), 2);

        let all = hierarchy.all_companies();
        assert_eq!(all.len(), 5);

        // Check effective ownership of DE (100% * 80% = 80%)
        let de = all.iter().find(|(c, _)| c == "DE").unwrap();
        assert_eq!(de.1, dec!(80));
    }

    #[test]
    fn test_indirect_ownership() {
        let config = EntityGraphConfig {
            include_indirect_ownership: true,
            ..Default::default()
        };
        let mut builder = EntityGraphBuilder::new(config);

        let relationships = vec![
            create_test_relationship("A", "B", dec!(100)),
            create_test_relationship("B", "C", dec!(50)),
        ];

        builder.add_ownership_relationships(&relationships);
        let graph = builder.build();

        // Should have A->B (direct), B->C (direct), and A->C (indirect)
        assert_eq!(graph.node_count(), 3);
    }
}
