//! Entity graph generator for interconnectivity modeling.
//!
//! Provides generation of comprehensive entity relationship graphs including:
//! - Transactional relationships from journal entries and document flows
//! - Cross-process linkages (P2P ↔ O2C via inventory)
//! - Relationship strength calculation
//! - Network analysis support

use chrono::NaiveDate;
use datasynth_core::models::{
    CrossProcessLink, CrossProcessLinkType, EntityGraph, EntityNode, GraphEntityId,
    GraphEntityType, GraphMetadata, RelationshipEdge, RelationshipStrengthCalculator,
    RelationshipType, VendorNetwork,
};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// Configuration for entity graph generation.
#[derive(Debug, Clone)]
pub struct EntityGraphConfig {
    /// Enable entity graph generation
    pub enabled: bool,
    /// Cross-process link configuration
    pub cross_process: CrossProcessConfig,
    /// Strength calculation settings
    pub strength_config: StrengthConfig,
    /// Include organizational relationships
    pub include_organizational: bool,
    /// Include document relationships
    pub include_document: bool,
}

impl Default for EntityGraphConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cross_process: CrossProcessConfig::default(),
            strength_config: StrengthConfig::default(),
            include_organizational: true,
            include_document: true,
        }
    }
}

/// Configuration for cross-process linkages.
#[derive(Debug, Clone)]
pub struct CrossProcessConfig {
    /// Enable inventory links between P2P and O2C
    pub enable_inventory_links: bool,
    /// Enable return flow generation
    pub enable_return_flows: bool,
    /// Enable payment reconciliation links
    pub enable_payment_links: bool,
    /// Enable intercompany bilateral matching
    pub enable_ic_bilateral: bool,
    /// Percentage of GR/Deliveries to link via inventory (0.0 - 1.0)
    pub inventory_link_rate: f64,
    /// Percentage of payments to link for reconciliation (0.0 - 1.0)
    pub payment_link_rate: f64,
}

impl Default for CrossProcessConfig {
    fn default() -> Self {
        Self {
            enable_inventory_links: true,
            enable_return_flows: true,
            enable_payment_links: true,
            enable_ic_bilateral: true,
            inventory_link_rate: 0.30,
            payment_link_rate: 0.80,
        }
    }
}

/// Configuration for relationship strength calculation.
#[derive(Debug, Clone)]
pub struct StrengthConfig {
    /// Transaction volume weight
    pub transaction_volume_weight: f64,
    /// Transaction count weight
    pub transaction_count_weight: f64,
    /// Duration weight
    pub duration_weight: f64,
    /// Recency weight
    pub recency_weight: f64,
    /// Mutual connections weight
    pub mutual_connections_weight: f64,
    /// Recency half-life in days
    pub recency_half_life_days: u32,
}

impl Default for StrengthConfig {
    fn default() -> Self {
        Self {
            transaction_volume_weight: 0.30,
            transaction_count_weight: 0.25,
            duration_weight: 0.20,
            recency_weight: 0.15,
            mutual_connections_weight: 0.10,
            recency_half_life_days: 90,
        }
    }
}

/// Summary of transaction history between two entities.
#[derive(Debug, Clone)]
pub struct TransactionSummary {
    /// Total transaction volume
    pub total_volume: Decimal,
    /// Number of transactions
    pub transaction_count: u32,
    /// First transaction date
    pub first_transaction_date: NaiveDate,
    /// Last transaction date
    pub last_transaction_date: NaiveDate,
    /// Related entity IDs (for mutual connection calculation)
    pub related_entities: HashSet<String>,
}

impl Default for TransactionSummary {
    fn default() -> Self {
        Self {
            total_volume: Decimal::ZERO,
            transaction_count: 0,
            first_transaction_date: NaiveDate::from_ymd_opt(2020, 1, 1)
                .expect("valid default date"),
            last_transaction_date: NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid default date"),
            related_entities: HashSet::new(),
        }
    }
}

/// Goods receipt summary for cross-process linking.
#[derive(Debug, Clone)]
pub struct GoodsReceiptRef {
    /// GR document ID
    pub document_id: String,
    /// Material ID
    pub material_id: String,
    /// Quantity received
    pub quantity: Decimal,
    /// Receipt date
    pub receipt_date: NaiveDate,
    /// Vendor ID
    pub vendor_id: String,
    /// Company code
    pub company_code: String,
}

/// Delivery summary for cross-process linking.
#[derive(Debug, Clone)]
pub struct DeliveryRef {
    /// Delivery document ID
    pub document_id: String,
    /// Material ID
    pub material_id: String,
    /// Quantity delivered
    pub quantity: Decimal,
    /// Delivery date
    pub delivery_date: NaiveDate,
    /// Customer ID
    pub customer_id: String,
    /// Company code
    pub company_code: String,
}

/// Generator for entity relationship graphs.
pub struct EntityGraphGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: EntityGraphConfig,
    strength_calculator: RelationshipStrengthCalculator,
}

impl EntityGraphGenerator {
    /// Create a new entity graph generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, EntityGraphConfig::default())
    }

    /// Create a new entity graph generator with configuration.
    pub fn with_config(seed: u64, config: EntityGraphConfig) -> Self {
        let strength_calculator = RelationshipStrengthCalculator {
            weights: datasynth_core::models::StrengthWeights {
                transaction_volume_weight: config.strength_config.transaction_volume_weight,
                transaction_count_weight: config.strength_config.transaction_count_weight,
                duration_weight: config.strength_config.duration_weight,
                recency_weight: config.strength_config.recency_weight,
                mutual_connections_weight: config.strength_config.mutual_connections_weight,
            },
            recency_half_life_days: config.strength_config.recency_half_life_days,
            ..Default::default()
        };

        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            strength_calculator,
        }
    }

    /// Generate an entity graph from transaction data.
    pub fn generate_entity_graph(
        &mut self,
        company_code: &str,
        as_of_date: NaiveDate,
        vendors: &[EntitySummary],
        customers: &[EntitySummary],
        transaction_summaries: &HashMap<(String, String), TransactionSummary>,
    ) -> EntityGraph {
        let mut graph = EntityGraph::new();
        graph.metadata = GraphMetadata {
            company_code: Some(company_code.to_string()),
            created_date: Some(as_of_date),
            total_transaction_volume: Decimal::ZERO,
            date_range: None,
        };

        if !self.config.enabled {
            return graph;
        }

        // Add company node
        let company_id = GraphEntityId::new(GraphEntityType::Company, company_code);
        graph.add_node(EntityNode::new(
            company_id.clone(),
            format!("Company {}", company_code),
            as_of_date,
        ));

        // Add vendor nodes (edges added below after transaction summary check)
        for vendor in vendors {
            let vendor_id = GraphEntityId::new(GraphEntityType::Vendor, &vendor.entity_id);
            let node = EntityNode::new(vendor_id.clone(), &vendor.name, as_of_date)
                .with_company(company_code);
            graph.add_node(node);

            // Only add a default-strength edge if no transaction summary will
            // supply a computed-strength edge for this vendor.
            let has_txn = transaction_summaries
                .keys()
                .any(|(_, to)| to == &vendor.entity_id);
            if !has_txn {
                let edge = RelationshipEdge::new(
                    company_id.clone(),
                    vendor_id,
                    RelationshipType::BuysFrom,
                    vendor.first_activity_date,
                );
                graph.add_edge(edge);
            }
        }

        // Add customer nodes (edges added below after transaction summary check)
        for customer in customers {
            let customer_id = GraphEntityId::new(GraphEntityType::Customer, &customer.entity_id);
            let node = EntityNode::new(customer_id.clone(), &customer.name, as_of_date)
                .with_company(company_code);
            graph.add_node(node);

            // Only add a default-strength edge if no transaction summary will
            // supply a computed-strength edge for this customer.
            let has_txn = transaction_summaries
                .keys()
                .any(|(_, to)| to == &customer.entity_id);
            if !has_txn {
                let edge = RelationshipEdge::new(
                    company_id.clone(),
                    customer_id,
                    RelationshipType::SellsTo,
                    customer.first_activity_date,
                );
                graph.add_edge(edge);
            }
        }

        // Add transactional relationships with strength
        let total_connections = transaction_summaries.len().max(1);
        for ((from_id, to_id), summary) in transaction_summaries {
            let from_entity_id = self.infer_entity_id(from_id);
            let to_entity_id = self.infer_entity_id(to_id);

            // Calculate relationship strength
            let days_since_last = (as_of_date - summary.last_transaction_date)
                .num_days()
                .max(0) as u32;
            let relationship_days = (as_of_date - summary.first_transaction_date)
                .num_days()
                .max(1) as u32;

            let components = self.strength_calculator.calculate(
                summary.total_volume,
                summary.transaction_count,
                relationship_days,
                days_since_last,
                summary.related_entities.len(),
                total_connections,
            );

            let rel_type = self.infer_relationship_type(&from_entity_id, &to_entity_id);

            let edge = RelationshipEdge::new(
                from_entity_id,
                to_entity_id,
                rel_type,
                summary.first_transaction_date,
            )
            .with_strength_components(components);

            graph.add_edge(edge);
        }

        // Calculate total transaction volume
        graph.metadata.total_transaction_volume =
            transaction_summaries.values().map(|s| s.total_volume).sum();

        graph
    }

    /// Generate cross-process links between P2P and O2C.
    pub fn generate_cross_process_links(
        &mut self,
        goods_receipts: &[GoodsReceiptRef],
        deliveries: &[DeliveryRef],
    ) -> Vec<CrossProcessLink> {
        let mut links = Vec::new();

        if !self.config.cross_process.enable_inventory_links {
            return links;
        }

        // Group deliveries by material for matching
        let deliveries_by_material: HashMap<String, Vec<&DeliveryRef>> =
            deliveries.iter().fold(HashMap::new(), |mut acc, del| {
                acc.entry(del.material_id.clone()).or_default().push(del);
                acc
            });

        // Link GRs to Deliveries via shared material
        for gr in goods_receipts {
            if self.rng.random::<f64>() > self.config.cross_process.inventory_link_rate {
                continue;
            }

            if let Some(matching_deliveries) = deliveries_by_material.get(&gr.material_id) {
                // Find a delivery in the same company that shares this material.
                // P2P and O2C chains are generated independently, so we match
                // on material + company without requiring a specific date order.
                let valid_deliveries: Vec<_> = matching_deliveries
                    .iter()
                    .filter(|d| d.company_code == gr.company_code)
                    .collect();

                if !valid_deliveries.is_empty() {
                    let delivery =
                        valid_deliveries[self.rng.random_range(0..valid_deliveries.len())];

                    // Calculate linked quantity (minimum of available)
                    let linked_qty = gr.quantity.min(delivery.quantity);

                    let link_date = gr.receipt_date.max(delivery.delivery_date);
                    links.push(CrossProcessLink::new(
                        &gr.material_id,
                        "P2P",
                        &gr.document_id,
                        "O2C",
                        &delivery.document_id,
                        CrossProcessLinkType::InventoryMovement,
                        linked_qty,
                        link_date,
                    ));
                }
            }
        }

        links
    }

    /// Generate graph from vendor network.
    pub fn generate_from_vendor_network(
        &mut self,
        vendor_network: &VendorNetwork,
        as_of_date: NaiveDate,
    ) -> EntityGraph {
        let mut graph = EntityGraph::new();
        graph.metadata = GraphMetadata {
            company_code: Some(vendor_network.company_code.clone()),
            created_date: Some(as_of_date),
            total_transaction_volume: vendor_network.statistics.total_annual_spend,
            date_range: None,
        };

        if !self.config.enabled {
            return graph;
        }

        // Add company node
        let company_id = GraphEntityId::new(GraphEntityType::Company, &vendor_network.company_code);
        graph.add_node(EntityNode::new(
            company_id.clone(),
            format!("Company {}", vendor_network.company_code),
            as_of_date,
        ));

        // Add all vendors from the network
        for (vendor_id, relationship) in &vendor_network.relationships {
            let entity_id = GraphEntityId::new(GraphEntityType::Vendor, vendor_id);
            let node = EntityNode::new(entity_id.clone(), vendor_id, as_of_date)
                .with_company(&vendor_network.company_code)
                .with_attribute("tier", format!("{:?}", relationship.tier))
                .with_attribute("cluster", format!("{:?}", relationship.cluster))
                .with_attribute(
                    "strategic_level",
                    format!("{:?}", relationship.strategic_importance),
                );
            graph.add_node(node);

            // Add relationship to company (for Tier 1) or parent vendor (for Tier 2/3)
            if let Some(parent_id) = &relationship.parent_vendor {
                let parent_entity_id = GraphEntityId::new(GraphEntityType::Vendor, parent_id);
                let edge = RelationshipEdge::new(
                    entity_id.clone(),
                    parent_entity_id,
                    RelationshipType::SuppliesTo,
                    relationship.start_date,
                )
                .with_strength(relationship.relationship_score());
                graph.add_edge(edge);
            } else {
                // Tier 1 supplies directly to company
                let edge = RelationshipEdge::new(
                    entity_id,
                    company_id.clone(),
                    RelationshipType::SuppliesTo,
                    relationship.start_date,
                )
                .with_strength(relationship.relationship_score());
                graph.add_edge(edge);
            }
        }

        graph
    }

    /// Infer entity ID from string (simple heuristic).
    fn infer_entity_id(&self, id: &str) -> GraphEntityId {
        if id.starts_with("V-") || id.starts_with("VN-") {
            GraphEntityId::new(GraphEntityType::Vendor, id)
        } else if id.starts_with("C-") || id.starts_with("CU-") {
            GraphEntityId::new(GraphEntityType::Customer, id)
        } else if id.starts_with("E-") || id.starts_with("EM-") {
            GraphEntityId::new(GraphEntityType::Employee, id)
        } else if id.starts_with("MAT-") || id.starts_with("M-") {
            GraphEntityId::new(GraphEntityType::Material, id)
        } else if id.starts_with("PO-") {
            GraphEntityId::new(GraphEntityType::PurchaseOrder, id)
        } else if id.starts_with("SO-") {
            GraphEntityId::new(GraphEntityType::SalesOrder, id)
        } else if id.starts_with("INV-") || id.starts_with("IV-") {
            GraphEntityId::new(GraphEntityType::Invoice, id)
        } else if id.starts_with("PAY-") || id.starts_with("PM-") {
            GraphEntityId::new(GraphEntityType::Payment, id)
        } else {
            GraphEntityId::new(GraphEntityType::Company, id)
        }
    }

    /// Infer relationship type between two entities.
    fn infer_relationship_type(
        &self,
        from: &GraphEntityId,
        to: &GraphEntityId,
    ) -> RelationshipType {
        match (&from.entity_type, &to.entity_type) {
            (GraphEntityType::Company, GraphEntityType::Vendor) => RelationshipType::BuysFrom,
            (GraphEntityType::Company, GraphEntityType::Customer) => RelationshipType::SellsTo,
            (GraphEntityType::Vendor, GraphEntityType::Company) => RelationshipType::SuppliesTo,
            (GraphEntityType::Customer, GraphEntityType::Company) => RelationshipType::SourcesFrom,
            (GraphEntityType::PurchaseOrder, GraphEntityType::Invoice) => {
                RelationshipType::References
            }
            (GraphEntityType::Invoice, GraphEntityType::Payment) => RelationshipType::FulfilledBy,
            (GraphEntityType::Payment, GraphEntityType::Invoice) => RelationshipType::AppliesTo,
            (GraphEntityType::Employee, GraphEntityType::Employee) => RelationshipType::ReportsTo,
            (GraphEntityType::Employee, GraphEntityType::Department) => RelationshipType::WorksIn,
            _ => RelationshipType::References,
        }
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
    }
}

/// Summary of an entity for graph generation.
#[derive(Debug, Clone)]
pub struct EntitySummary {
    /// Entity ID
    pub entity_id: String,
    /// Entity name
    pub name: String,
    /// First activity date
    pub first_activity_date: NaiveDate,
    /// Entity type (for categorization)
    pub entity_type: GraphEntityType,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl EntitySummary {
    /// Create a new entity summary.
    pub fn new(
        entity_id: impl Into<String>,
        name: impl Into<String>,
        entity_type: GraphEntityType,
        first_activity_date: NaiveDate,
    ) -> Self {
        Self {
            entity_id: entity_id.into(),
            name: name.into(),
            first_activity_date,
            entity_type,
            attributes: HashMap::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_graph_generation() {
        let config = EntityGraphConfig {
            enabled: true,
            ..Default::default()
        };

        let mut gen = EntityGraphGenerator::with_config(42, config);

        let vendors = vec![
            EntitySummary::new(
                "V-001",
                "Acme Supplies",
                GraphEntityType::Vendor,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            ),
            EntitySummary::new(
                "V-002",
                "Global Parts",
                GraphEntityType::Vendor,
                NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
            ),
        ];

        let customers = vec![EntitySummary::new(
            "C-001",
            "Contoso Corp",
            GraphEntityType::Customer,
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
        )];

        let graph = gen.generate_entity_graph(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            &vendors,
            &customers,
            &HashMap::new(),
        );

        // Should have company + 2 vendors + 1 customer = 4 nodes
        assert_eq!(graph.nodes.len(), 4);
        // Should have 3 edges (company buys from 2 vendors, sells to 1 customer)
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn test_cross_process_link_generation() {
        let config = EntityGraphConfig {
            enabled: true,
            cross_process: CrossProcessConfig {
                enable_inventory_links: true,
                inventory_link_rate: 1.0, // Always link for testing
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = EntityGraphGenerator::with_config(42, config);

        let goods_receipts = vec![GoodsReceiptRef {
            document_id: "GR-001".to_string(),
            material_id: "MAT-100".to_string(),
            quantity: Decimal::from(100),
            receipt_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vendor_id: "V-001".to_string(),
            company_code: "1000".to_string(),
        }];

        let deliveries = vec![DeliveryRef {
            document_id: "DEL-001".to_string(),
            material_id: "MAT-100".to_string(),
            quantity: Decimal::from(50),
            delivery_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            customer_id: "C-001".to_string(),
            company_code: "1000".to_string(),
        }];

        let links = gen.generate_cross_process_links(&goods_receipts, &deliveries);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].material_id, "MAT-100");
        assert_eq!(links[0].source_document_id, "GR-001");
        assert_eq!(links[0].target_document_id, "DEL-001");
        assert_eq!(links[0].link_type, CrossProcessLinkType::InventoryMovement);
    }

    #[test]
    fn test_disabled_graph_generation() {
        let config = EntityGraphConfig {
            enabled: false,
            ..Default::default()
        };

        let mut gen = EntityGraphGenerator::with_config(42, config);

        let graph = gen.generate_entity_graph(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            &[],
            &[],
            &HashMap::new(),
        );

        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_entity_id_inference() {
        let gen = EntityGraphGenerator::new(42);

        let vendor_id = gen.infer_entity_id("V-001");
        assert_eq!(vendor_id.entity_type, GraphEntityType::Vendor);

        let customer_id = gen.infer_entity_id("C-001");
        assert_eq!(customer_id.entity_type, GraphEntityType::Customer);

        let po_id = gen.infer_entity_id("PO-12345");
        assert_eq!(po_id.entity_type, GraphEntityType::PurchaseOrder);
    }

    #[test]
    fn test_relationship_type_inference() {
        let gen = EntityGraphGenerator::new(42);

        let company_id = GraphEntityId::new(GraphEntityType::Company, "1000");
        let vendor_id = GraphEntityId::new(GraphEntityType::Vendor, "V-001");

        let rel_type = gen.infer_relationship_type(&company_id, &vendor_id);
        assert_eq!(rel_type, RelationshipType::BuysFrom);

        let rel_type = gen.infer_relationship_type(&vendor_id, &company_id);
        assert_eq!(rel_type, RelationshipType::SuppliesTo);
    }
}
