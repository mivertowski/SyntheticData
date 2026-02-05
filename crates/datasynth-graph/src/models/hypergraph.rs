//! Multi-layer hypergraph model types for RustGraph integration.
//!
//! Defines a 3-layer hypergraph structure:
//! - Layer 1: Governance & Controls (COSO, SOX, internal controls, organizational)
//! - Layer 2: Process Events (P2P/O2C document flows, OCPM events)
//! - Layer 3: Accounting Network (GL accounts, journal entries as hyperedges)

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which layer of the hypergraph a node or hyperedge belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypergraphLayer {
    /// Layer 1: Governance & Controls (COSO components, internal controls, SOX, organizational).
    GovernanceControls,
    /// Layer 2: Process Events (P2P/O2C document flows, OCPM process events).
    ProcessEvents,
    /// Layer 3: Accounting Network (GL accounts, journal entries as hyperedges).
    AccountingNetwork,
}

impl HypergraphLayer {
    /// Returns the numeric layer index (1-3).
    pub fn index(&self) -> u8 {
        match self {
            HypergraphLayer::GovernanceControls => 1,
            HypergraphLayer::ProcessEvents => 2,
            HypergraphLayer::AccountingNetwork => 3,
        }
    }

    /// Returns the display name for the layer.
    pub fn name(&self) -> &'static str {
        match self {
            HypergraphLayer::GovernanceControls => "Governance & Controls",
            HypergraphLayer::ProcessEvents => "Process Events",
            HypergraphLayer::AccountingNetwork => "Accounting Network",
        }
    }
}

/// Strategy for aggregating nodes when budget is exceeded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationStrategy {
    /// Truncate: simply stop adding nodes after budget is reached.
    Truncate,
    /// Pool documents by their counterparty (vendor/customer).
    #[default]
    PoolByCounterparty,
    /// Pool documents by time period (month).
    PoolByTimePeriod,
    /// Keep most important nodes based on transaction volume.
    ImportanceSample,
}

/// A participant in a hyperedge (node reference with role and optional weight).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperedgeParticipant {
    /// ID of the participating node.
    pub node_id: String,
    /// Role of this participant (e.g., "debit", "credit", "approver", "vendor").
    pub role: String,
    /// Optional weight (e.g., line amount for journal entry lines).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// A hyperedge connecting multiple nodes simultaneously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperedge {
    /// Unique hyperedge identifier.
    pub id: String,
    /// High-level type: "ProcessFamily", "MultiRelation", "JournalEntry".
    pub hyperedge_type: String,
    /// Subtype with more detail: "P2P", "O2C", "JournalEntry".
    pub subtype: String,
    /// Nodes participating in this hyperedge with their roles.
    pub participants: Vec<HyperedgeParticipant>,
    /// Which layer this hyperedge belongs to.
    pub layer: HypergraphLayer,
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

/// A node in the hypergraph with layer assignment and RustGraph type codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphNode {
    /// Unique node identifier.
    pub id: String,
    /// Entity type name (e.g., "Account", "Vendor", "CosoComponent").
    pub entity_type: String,
    /// RustGraph entity type code for import.
    pub entity_type_code: u32,
    /// Which layer this node belongs to.
    pub layer: HypergraphLayer,
    /// External identifier from the source system.
    pub external_id: String,
    /// Human-readable label.
    pub label: String,
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

/// A pairwise edge connecting nodes across or within layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLayerEdge {
    /// Source node ID.
    pub source_id: String,
    /// Source node's layer.
    pub source_layer: HypergraphLayer,
    /// Target node ID.
    pub target_id: String,
    /// Target node's layer.
    pub target_layer: HypergraphLayer,
    /// Edge type name (e.g., "ImplementsControl", "GovernedByStandard").
    pub edge_type: String,
    /// RustGraph edge type code for import.
    pub edge_type_code: u32,
    /// Additional properties as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Value>,
}

/// Per-layer node budget allocation and tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeBudget {
    /// Maximum nodes allowed for Layer 1 (Governance).
    pub layer1_max: usize,
    /// Maximum nodes allowed for Layer 2 (Process).
    pub layer2_max: usize,
    /// Maximum nodes allowed for Layer 3 (Accounting).
    pub layer3_max: usize,
    /// Current count for Layer 1.
    pub layer1_count: usize,
    /// Current count for Layer 2.
    pub layer2_count: usize,
    /// Current count for Layer 3.
    pub layer3_count: usize,
}

impl NodeBudget {
    /// Create a budget with the given total max nodes.
    /// Default allocation: L1 gets 20%, L3 gets 10%, L2 gets remainder (70%).
    pub fn new(max_nodes: usize) -> Self {
        let l1 = max_nodes / 5; // 20%
        let l3 = max_nodes / 10; // 10%
        let l2 = max_nodes - l1 - l3; // 70%
        Self {
            layer1_max: l1,
            layer2_max: l2,
            layer3_max: l3,
            layer1_count: 0,
            layer2_count: 0,
            layer3_count: 0,
        }
    }

    /// Check if a layer can accept more nodes.
    pub fn can_add(&self, layer: HypergraphLayer) -> bool {
        match layer {
            HypergraphLayer::GovernanceControls => self.layer1_count < self.layer1_max,
            HypergraphLayer::ProcessEvents => self.layer2_count < self.layer2_max,
            HypergraphLayer::AccountingNetwork => self.layer3_count < self.layer3_max,
        }
    }

    /// Record a node addition.
    pub fn record_add(&mut self, layer: HypergraphLayer) {
        match layer {
            HypergraphLayer::GovernanceControls => self.layer1_count += 1,
            HypergraphLayer::ProcessEvents => self.layer2_count += 1,
            HypergraphLayer::AccountingNetwork => self.layer3_count += 1,
        }
    }

    /// Total nodes across all layers.
    pub fn total_count(&self) -> usize {
        self.layer1_count + self.layer2_count + self.layer3_count
    }

    /// Total budget across all layers.
    pub fn total_max(&self) -> usize {
        self.layer1_max + self.layer2_max + self.layer3_max
    }

    /// Rebalance the budget based on actual demand per layer.
    /// Unused budget from layers with fewer entities than max is redistributed.
    pub fn rebalance(&mut self, l1_demand: usize, l2_demand: usize, l3_demand: usize) {
        let total = self.total_max();

        // Clamp each layer to its demand
        let l1_actual = l1_demand.min(self.layer1_max);
        let l3_actual = l3_demand.min(self.layer3_max);

        // Give surplus to L2
        let surplus = (self.layer1_max - l1_actual) + (self.layer3_max - l3_actual);
        let l2_actual = (self.layer2_max + surplus)
            .min(l2_demand)
            .min(total - l1_actual - l3_actual.min(total.saturating_sub(l1_actual)));

        self.layer1_max = l1_actual;
        self.layer3_max = total.saturating_sub(l1_actual).saturating_sub(l2_actual);
        self.layer2_max = l2_actual;
    }
}

/// Report on node budget utilization after building.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeBudgetReport {
    /// Total budget configured.
    pub total_budget: usize,
    /// Total nodes actually created.
    pub total_used: usize,
    /// Layer 1 budget and usage.
    pub layer1_budget: usize,
    pub layer1_used: usize,
    /// Layer 2 budget and usage.
    pub layer2_budget: usize,
    pub layer2_used: usize,
    /// Layer 3 budget and usage.
    pub layer3_budget: usize,
    pub layer3_used: usize,
    /// Number of aggregate (pool) nodes created.
    pub aggregate_nodes_created: usize,
    /// Whether aggregation was triggered.
    pub aggregation_triggered: bool,
}

/// Metadata about the exported hypergraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphMetadata {
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

/// The complete built hypergraph with all components.
#[derive(Debug, Clone)]
pub struct Hypergraph {
    /// All nodes across all layers.
    pub nodes: Vec<HypergraphNode>,
    /// All pairwise edges (cross-layer and intra-layer).
    pub edges: Vec<CrossLayerEdge>,
    /// All hyperedges (journal entries, OCPM events).
    pub hyperedges: Vec<Hyperedge>,
    /// Export metadata.
    pub metadata: HypergraphMetadata,
    /// Budget utilization report.
    pub budget_report: NodeBudgetReport,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_index() {
        assert_eq!(HypergraphLayer::GovernanceControls.index(), 1);
        assert_eq!(HypergraphLayer::ProcessEvents.index(), 2);
        assert_eq!(HypergraphLayer::AccountingNetwork.index(), 3);
    }

    #[test]
    fn test_node_budget_new() {
        let budget = NodeBudget::new(50_000);
        assert_eq!(budget.layer1_max, 10_000); // 20%
        assert_eq!(budget.layer2_max, 35_000); // 70%
        assert_eq!(budget.layer3_max, 5_000); // 10%
        assert_eq!(budget.total_max(), 50_000);
    }

    #[test]
    fn test_node_budget_can_add() {
        let mut budget = NodeBudget::new(100);
        assert!(budget.can_add(HypergraphLayer::GovernanceControls));

        // Fill L1 to max (20)
        for _ in 0..20 {
            budget.record_add(HypergraphLayer::GovernanceControls);
        }
        assert!(!budget.can_add(HypergraphLayer::GovernanceControls));
        assert!(budget.can_add(HypergraphLayer::ProcessEvents));
    }

    #[test]
    fn test_node_budget_total() {
        let mut budget = NodeBudget::new(1000);
        budget.record_add(HypergraphLayer::GovernanceControls);
        budget.record_add(HypergraphLayer::ProcessEvents);
        budget.record_add(HypergraphLayer::AccountingNetwork);
        assert_eq!(budget.total_count(), 3);
    }

    #[test]
    fn test_hypergraph_node_serialization() {
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

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: HypergraphNode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "node_1");
        assert_eq!(deserialized.entity_type_code, 100);
        assert_eq!(deserialized.layer, HypergraphLayer::AccountingNetwork);
    }

    #[test]
    fn test_hyperedge_serialization() {
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

        let json = serde_json::to_string(&he).unwrap();
        let deserialized: Hyperedge = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.participants.len(), 2);
        assert!(deserialized.is_anomaly);
    }

    #[test]
    fn test_cross_layer_edge_serialization() {
        let edge = CrossLayerEdge {
            source_id: "ctrl_C001".to_string(),
            source_layer: HypergraphLayer::GovernanceControls,
            target_id: "acct_1000".to_string(),
            target_layer: HypergraphLayer::AccountingNetwork,
            edge_type: "ImplementsControl".to_string(),
            edge_type_code: 40,
            properties: HashMap::new(),
        };

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: CrossLayerEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.edge_type, "ImplementsControl");
        assert_eq!(
            deserialized.source_layer,
            HypergraphLayer::GovernanceControls
        );
        assert_eq!(
            deserialized.target_layer,
            HypergraphLayer::AccountingNetwork
        );
    }

    #[test]
    fn test_aggregation_strategy_default() {
        assert_eq!(
            AggregationStrategy::default(),
            AggregationStrategy::PoolByCounterparty
        );
    }
}
