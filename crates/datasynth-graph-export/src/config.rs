//! Configuration types for the graph export pipeline.
//!
//! [`ExportConfig`] is the top-level configuration that controls every aspect of the pipeline:
//! node/edge budgets, property casing, edge synthesis strategy, OCEL export, and ground truth.

use serde::{Deserialize, Serialize};

// ──────────────────────────── Top-Level Config ────────────────

/// Top-level configuration for the graph export pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Node and edge budget constraints.
    pub budget: BudgetConfig,
    /// Edge synthesis configuration.
    pub edge_synthesis: EdgeSynthesisConfig,
    /// OCEL 2.0 export configuration.
    pub ocel: OcelExportConfig,
    /// Ground truth / ML evaluation configuration.
    pub ground_truth: GroundTruthConfig,
    /// Property key casing convention.
    pub property_case: PropertyCase,
    /// Which properties to include in the export.
    pub property_inclusion: PropertyInclusionPolicy,
    /// Whether to skip banking data entirely (avoids OOM for large datasets).
    pub skip_banking: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            budget: BudgetConfig::default(),
            edge_synthesis: EdgeSynthesisConfig::default(),
            ocel: OcelExportConfig::default(),
            ground_truth: GroundTruthConfig::default(),
            property_case: PropertyCase::CamelCase,
            property_inclusion: PropertyInclusionPolicy::DashboardRequired,
            skip_banking: false,
        }
    }
}

// ──────────────────────────── Budget ──────────────────────────

/// Controls the maximum number of nodes and edges in the export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum total nodes across all layers.
    pub max_nodes: usize,
    /// Maximum total edges. 0 means auto-calculate from max_nodes (typically 20x).
    pub max_edges: usize,
    /// Layer budget split as fractions: [L1_governance, L2_process, L3_accounting].
    /// Must sum to ~1.0.
    pub layer_split: [f64; 3],
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_nodes: 50_000,
            max_edges: 0, // auto: max(max_edges, max_nodes * 20, 100_000)
            layer_split: [0.04, 0.92, 0.04],
        }
    }
}

impl BudgetConfig {
    /// Returns the effective max_edges, applying the auto-calculation if max_edges == 0.
    pub fn effective_max_edges(&self) -> usize {
        if self.max_edges > 0 {
            self.max_edges
        } else {
            (self.max_nodes * 20).max(100_000)
        }
    }

    /// Returns the node budget for a specific layer (0-indexed: 0=L1, 1=L2, 2=L3).
    pub fn layer_budget(&self, layer_index: usize) -> usize {
        if layer_index >= 3 {
            return 0;
        }
        (self.max_nodes as f64 * self.layer_split[layer_index]).round() as usize
    }
}

// ──────────────────────────── Edge Synthesis ──────────────────

/// Configuration for edge synthesis (how edges are created from source data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSynthesisConfig {
    /// Strategy for sampling edges when the budget is exceeded.
    pub sampling_strategy: EdgeSamplingStrategy,
    /// Strategy for generating risk-control linkage edges.
    pub risk_control_strategy: RiskControlStrategy,
    /// Whether to generate cross-layer edges (e.g., control → account).
    pub cross_layer_edges: bool,
    /// Whether to generate accounting network edges (account hierarchy, JE posting).
    pub accounting_network_edges: bool,
    /// Whether to generate people/role edges (document creator, approver, etc.).
    pub people_edges: bool,
}

impl Default for EdgeSynthesisConfig {
    fn default() -> Self {
        Self {
            sampling_strategy: EdgeSamplingStrategy::Proportional,
            risk_control_strategy: RiskControlStrategy::NameMatching,
            cross_layer_edges: true,
            accounting_network_edges: true,
            people_edges: true,
        }
    }
}

/// Strategy for sampling edges when the edge budget is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeSamplingStrategy {
    /// Keep edges proportionally across all edge types.
    Proportional,
    /// Prioritize governance and cross-layer edges; truncate high-volume L2 edges.
    GovernancePriority,
    /// Simple truncation — first N edges in insertion order.
    Truncate,
}

/// Strategy for generating risk → control linkage edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskControlStrategy {
    /// Match risks to controls by name/description keywords.
    NameMatching,
    /// Use foreign key references from the source data models.
    ForeignKey,
    /// Combine FK lookup with name-based fallback.
    Hybrid,
}

// ──────────────────────────── OCEL ────────────────────────────

/// Configuration for OCEL 2.0 event log export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelExportConfig {
    /// Whether to generate the OCEL export at all.
    pub enabled: bool,
    /// Maximum number of events to include (0 = unlimited).
    pub max_events: usize,
}

impl Default for OcelExportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: 50_000,
        }
    }
}

// ──────────────────────────── Ground Truth ────────────────────

/// Configuration for ground truth / ML evaluation data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruthConfig {
    /// Whether to emit ground truth records.
    pub enabled: bool,
    /// Whether to generate feature vectors for GNN training.
    pub feature_vectors: bool,
}

impl Default for GroundTruthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            feature_vectors: false,
        }
    }
}

// ──────────────────────────── Property Options ────────────────

/// Casing convention for property keys in the export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyCase {
    /// camelCase (e.g., "controlType", "isKeyControl") — matches AssureTwin dashboard expectations.
    CamelCase,
    /// snake_case (e.g., "control_type", "is_key_control") — matches RustGraph wire format.
    SnakeCase,
}

/// Policy for which properties to include in the export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyInclusionPolicy {
    /// Include only the properties required by the AssureTwin dashboard.
    DashboardRequired,
    /// Include all available properties from the source data.
    All,
    /// Include no properties (bare nodes/edges only).
    None,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let config = ExportConfig::default();
        assert_eq!(config.budget.max_nodes, 50_000);
        assert_eq!(config.budget.max_edges, 0);
        assert_eq!(config.budget.effective_max_edges(), 1_000_000);
        assert_eq!(config.property_case, PropertyCase::CamelCase);
        assert!(!config.skip_banking);
    }

    #[test]
    fn layer_budget_calculation() {
        let budget = BudgetConfig {
            max_nodes: 10_000,
            max_edges: 0,
            layer_split: [0.04, 0.92, 0.04],
        };
        assert_eq!(budget.layer_budget(0), 400); // L1: 4%
        assert_eq!(budget.layer_budget(1), 9200); // L2: 92%
        assert_eq!(budget.layer_budget(2), 400); // L3: 4%
        assert_eq!(budget.layer_budget(3), 0); // out of range
    }

    #[test]
    fn effective_max_edges_auto_calculation() {
        let budget = BudgetConfig {
            max_nodes: 1_000,
            max_edges: 0,
            layer_split: [0.04, 0.92, 0.04],
        };
        // max(1000*20, 100_000) = 100_000
        assert_eq!(budget.effective_max_edges(), 100_000);

        let budget2 = BudgetConfig {
            max_nodes: 50_000,
            max_edges: 0,
            layer_split: [0.04, 0.92, 0.04],
        };
        // max(50000*20, 100_000) = 1_000_000
        assert_eq!(budget2.effective_max_edges(), 1_000_000);
    }

    #[test]
    fn explicit_max_edges_overrides_auto() {
        let budget = BudgetConfig {
            max_nodes: 50_000,
            max_edges: 200_000,
            layer_split: [0.04, 0.92, 0.04],
        };
        assert_eq!(budget.effective_max_edges(), 200_000);
    }
}
