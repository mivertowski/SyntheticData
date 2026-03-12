//! Budget enforcement for the graph export pipeline.
//!
//! The [`BudgetManager`] enforces node and edge limits by trimming excess data
//! after synthesis. Node trimming is layer-aware (respects `layer_split`), and
//! edge trimming supports multiple strategies (truncation, proportional sampling).

use std::collections::{HashMap, HashSet};

use tracing::{debug, info};

use crate::config::{BudgetConfig, EdgeSamplingStrategy};
use crate::error::{ExportWarnings, WarningSeverity};
use crate::id_map::IdMap;
use crate::types::{ExportEdge, ExportNode};

/// Enforces node and edge budgets on the pipeline output.
pub struct BudgetManager<'a> {
    config: &'a BudgetConfig,
}

impl<'a> BudgetManager<'a> {
    /// Create a new budget manager with the given configuration.
    pub fn new(config: &'a BudgetConfig) -> Self {
        Self { config }
    }

    /// Enforce the node budget using layer-aware truncation.
    ///
    /// Nodes are grouped by layer, and each layer is truncated to its budget
    /// (derived from `config.layer_split`). The `id_map` is cleaned up to
    /// remove entries for dropped nodes.
    ///
    /// Returns the trimmed node vec and updates `id_map` in place.
    pub fn enforce_node_budget(
        &self,
        mut nodes: Vec<ExportNode>,
        id_map: &mut IdMap,
        warnings: &mut ExportWarnings,
    ) -> Vec<ExportNode> {
        let total = nodes.len();
        if total <= self.config.max_nodes {
            debug!(
                "Node budget OK: {total} nodes within limit of {}",
                self.config.max_nodes
            );
            return nodes;
        }

        info!(
            "Enforcing node budget: {total} nodes exceeds limit of {}, trimming by layer",
            self.config.max_nodes
        );

        // Partition nodes by layer (1-indexed: 1=L1, 2=L2, 3=L3)
        let mut layer_buckets: [Vec<ExportNode>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for node in nodes.drain(..) {
            let idx = (node.layer as usize).saturating_sub(1).min(2);
            layer_buckets[idx].push(node);
        }

        // Truncate each layer to its budget
        let mut kept = Vec::with_capacity(self.config.max_nodes);
        for (i, bucket) in layer_buckets.iter_mut().enumerate() {
            let budget = self.config.layer_budget(i);
            let original = bucket.len();
            if original > budget {
                bucket.truncate(budget);
                warnings.add(
                    "budget",
                    WarningSeverity::Medium,
                    format!(
                        "Layer {} trimmed from {original} to {budget} nodes",
                        i + 1
                    ),
                );
            }
            kept.append(bucket);
        }

        // Clean up id_map: retain only IDs present in kept nodes
        let kept_ids: HashSet<u64> = kept
            .iter()
            .filter_map(|n| n.id)
            .collect();
        id_map.retain_nodes(&kept_ids);

        let trimmed = total - kept.len();
        info!(
            "Node budget enforced: kept {} nodes, trimmed {trimmed}",
            kept.len()
        );

        kept
    }

    /// Enforce the edge budget using the configured sampling strategy.
    ///
    /// Edges whose source or target node was removed during node trimming are
    /// always dropped first. Then the remaining edges are trimmed to the budget.
    pub fn enforce_edge_budget(
        &self,
        mut edges: Vec<ExportEdge>,
        id_map: &IdMap,
        strategy: EdgeSamplingStrategy,
        warnings: &mut ExportWarnings,
    ) -> Vec<ExportEdge> {
        let max_edges = self.config.effective_max_edges();

        // Phase 1: Drop edges with dangling references (source/target not in id_map)
        let before_dangling = edges.len();
        let valid_ids: HashSet<u64> = id_map.iter_reverse().map(|(id, _)| id).collect();
        edges.retain(|e| valid_ids.contains(&e.source) && valid_ids.contains(&e.target));
        let dangling_removed = before_dangling - edges.len();
        if dangling_removed > 0 {
            warnings.info(
                "budget",
                format!("Removed {dangling_removed} edges with dangling node references"),
            );
        }

        // Phase 2: Check budget
        if edges.len() <= max_edges {
            debug!(
                "Edge budget OK: {} edges within limit of {max_edges}",
                edges.len()
            );
            return edges;
        }

        info!(
            "Enforcing edge budget: {} edges exceeds limit of {max_edges}, strategy={strategy:?}",
            edges.len()
        );

        match strategy {
            EdgeSamplingStrategy::Truncate => {
                let original = edges.len();
                edges.truncate(max_edges);
                warnings.add(
                    "budget",
                    WarningSeverity::Medium,
                    format!("Edge budget: truncated from {original} to {max_edges} edges"),
                );
            }
            EdgeSamplingStrategy::Proportional => {
                edges = self.proportional_sample(edges, max_edges, warnings);
            }
            EdgeSamplingStrategy::GovernancePriority => {
                edges = self.governance_priority_sample(edges, max_edges, warnings);
            }
        }

        edges
    }

    /// Sample edges proportionally across all edge types.
    ///
    /// Each edge type retains at least 1 edge (if present), and the remaining
    /// budget is distributed proportionally to each type's original count.
    fn proportional_sample(
        &self,
        edges: Vec<ExportEdge>,
        max_edges: usize,
        warnings: &mut ExportWarnings,
    ) -> Vec<ExportEdge> {
        let original_count = edges.len();

        // Group by edge_type
        let mut by_type: HashMap<u32, Vec<ExportEdge>> = HashMap::new();
        for edge in edges {
            by_type.entry(edge.edge_type).or_default().push(edge);
        }

        let num_types = by_type.len();
        if num_types == 0 {
            return Vec::new();
        }

        // Guarantee at least 1 edge per type, then distribute remaining proportionally
        let min_per_type = 1usize;
        let guaranteed = num_types * min_per_type;
        let remaining_budget = max_edges.saturating_sub(guaranteed);

        let mut result = Vec::with_capacity(max_edges);
        for (_edge_type, mut type_edges) in by_type {
            let proportion =
                (type_edges.len() as f64 / original_count as f64 * remaining_budget as f64).round()
                    as usize;
            let keep = min_per_type + proportion;
            let keep = keep.min(type_edges.len());
            type_edges.truncate(keep);
            result.extend(type_edges);
        }

        // Final truncation in case rounding overshot
        if result.len() > max_edges {
            result.truncate(max_edges);
        }

        warnings.add(
            "budget",
            WarningSeverity::Medium,
            format!(
                "Edge budget: proportionally sampled from {original_count} to {} edges across {num_types} types",
                result.len()
            ),
        );

        result
    }

    /// Sample edges with governance priority: keep all governance/cross-layer edges,
    /// then fill remaining budget with process-layer edges proportionally.
    ///
    /// Governance edges are edge_type codes in the 40-59 and 96-120 ranges.
    /// Process-layer edges (60-95) are sampled proportionally.
    fn governance_priority_sample(
        &self,
        edges: Vec<ExportEdge>,
        max_edges: usize,
        warnings: &mut ExportWarnings,
    ) -> Vec<ExportEdge> {
        let original_count = edges.len();

        let (governance, process): (Vec<ExportEdge>, Vec<ExportEdge>) =
            edges.into_iter().partition(|e| {
                // Governance: 40-59 + 96-120 (governance linkage + cross-layer people)
                // + 138-152 (audit procedure linkage — ISA 505/330/530/520/610/550)
                (40..60).contains(&e.edge_type)
                    || (96..=120).contains(&e.edge_type)
                    || (138..=152).contains(&e.edge_type)
            });

        let mut result = Vec::with_capacity(max_edges);

        // Keep all governance edges (up to budget)
        let gov_keep = governance.len().min(max_edges);
        result.extend(governance.into_iter().take(gov_keep));

        // Fill remaining budget with process edges
        let remaining = max_edges.saturating_sub(result.len());
        if remaining > 0 && !process.is_empty() {
            let proc_keep = remaining.min(process.len());
            result.extend(process.into_iter().take(proc_keep));
        }

        warnings.add(
            "budget",
            WarningSeverity::Medium,
            format!(
                "Edge budget: governance-priority sampled from {original_count} to {} edges ({gov_keep} governance)",
                result.len()
            ),
        );

        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::BudgetConfig;

    fn make_node(id: u64, layer: u8) -> ExportNode {
        ExportNode {
            id: Some(id),
            node_type: 100,
            node_type_name: "test".to_string(),
            label: format!("Node {id}"),
            layer,
            properties: HashMap::new(),
        }
    }

    fn make_edge(source: u64, target: u64, edge_type: u32) -> ExportEdge {
        ExportEdge {
            source,
            target,
            edge_type,
            weight: 1.0,
            properties: HashMap::new(),
        }
    }

    fn budget_config(max_nodes: usize, max_edges: usize) -> BudgetConfig {
        BudgetConfig {
            max_nodes,
            max_edges,
            layer_split: [0.20, 0.60, 0.20],
        }
    }

    #[test]
    fn node_budget_no_trimming_needed() {
        let config = budget_config(100, 0);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        let mut warnings = ExportWarnings::new();

        let nodes = vec![make_node(1, 1), make_node(2, 2), make_node(3, 3)];
        for n in &nodes {
            id_map.get_or_insert(&format!("n{}", n.id.unwrap_or(0)));
        }

        let result = mgr.enforce_node_budget(nodes, &mut id_map, &mut warnings);
        assert_eq!(result.len(), 3);
        assert!(warnings.is_empty());
    }

    #[test]
    fn node_budget_trims_by_layer() {
        let config = budget_config(5, 0);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        let mut warnings = ExportWarnings::new();

        // Create 10 nodes: 3 in L1, 4 in L2, 3 in L3
        let mut nodes = Vec::new();
        for i in 1..=3 {
            let id = id_map.get_or_insert(&format!("l1-{i}"));
            nodes.push(make_node(id, 1));
        }
        for i in 1..=4 {
            let id = id_map.get_or_insert(&format!("l2-{i}"));
            nodes.push(make_node(id, 2));
        }
        for i in 1..=3 {
            let id = id_map.get_or_insert(&format!("l3-{i}"));
            nodes.push(make_node(id, 3));
        }

        let result = mgr.enforce_node_budget(nodes, &mut id_map, &mut warnings);
        // Budget: L1=1 (20% of 5), L2=3 (60%), L3=1 (20%)
        assert_eq!(result.len(), 5);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn edge_budget_removes_dangling() {
        let config = budget_config(100, 1000);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        id_map.get_or_insert("a"); // id=1
        id_map.get_or_insert("b"); // id=2
        let mut warnings = ExportWarnings::new();

        let edges = vec![
            make_edge(1, 2, 60), // valid
            make_edge(1, 99, 60), // dangling target
            make_edge(99, 2, 60), // dangling source
        ];

        let result = mgr.enforce_edge_budget(
            edges,
            &id_map,
            EdgeSamplingStrategy::Truncate,
            &mut warnings,
        );
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn edge_budget_truncate_strategy() {
        let config = budget_config(100, 2);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        id_map.get_or_insert("a"); // id=1
        id_map.get_or_insert("b"); // id=2
        let mut warnings = ExportWarnings::new();

        let edges = vec![
            make_edge(1, 2, 60),
            make_edge(2, 1, 61),
            make_edge(1, 2, 62),
            make_edge(2, 1, 63),
        ];

        let result = mgr.enforce_edge_budget(
            edges,
            &id_map,
            EdgeSamplingStrategy::Truncate,
            &mut warnings,
        );
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn edge_budget_proportional_keeps_all_types() {
        let config = budget_config(100, 4);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        id_map.get_or_insert("a"); // id=1
        id_map.get_or_insert("b"); // id=2
        let mut warnings = ExportWarnings::new();

        // 3 edge types, 2 edges each = 6 total, budget = 4
        let edges = vec![
            make_edge(1, 2, 60),
            make_edge(2, 1, 60),
            make_edge(1, 2, 70),
            make_edge(2, 1, 70),
            make_edge(1, 2, 80),
            make_edge(2, 1, 80),
        ];

        let result = mgr.enforce_edge_budget(
            edges,
            &id_map,
            EdgeSamplingStrategy::Proportional,
            &mut warnings,
        );
        // Should keep at least 1 per type (3 types), then distribute remaining 1
        assert!(result.len() <= 4);
        // All 3 edge types should be represented
        let types: HashSet<u32> = result.iter().map(|e| e.edge_type).collect();
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn edge_budget_governance_priority() {
        let config = budget_config(100, 3);
        let mgr = BudgetManager::new(&config);
        let mut id_map = IdMap::new();
        id_map.get_or_insert("a"); // id=1
        id_map.get_or_insert("b"); // id=2
        let mut warnings = ExportWarnings::new();

        // 3 governance edges (type 40, 100, 140) + 3 process edges (type 60, 70, 80)
        let edges = vec![
            make_edge(1, 2, 40),  // governance (40-59 range)
            make_edge(2, 1, 100), // governance (cross-layer people, 96-120 range)
            make_edge(1, 2, 140), // governance (audit procedure linkage, 138-152 range)
            make_edge(1, 2, 60),  // process
            make_edge(2, 1, 70),  // process
            make_edge(1, 2, 80),  // process
        ];

        let result = mgr.enforce_edge_budget(
            edges,
            &id_map,
            EdgeSamplingStrategy::GovernancePriority,
            &mut warnings,
        );
        assert_eq!(result.len(), 3);
        // All 3 governance edges should be kept (budget=3, all governance fit)
        let gov_count = result
            .iter()
            .filter(|e| {
                (40..60).contains(&e.edge_type)
                    || (96..=120).contains(&e.edge_type)
                    || (138..=152).contains(&e.edge_type)
            })
            .count();
        assert_eq!(gov_count, 3);
    }
}
