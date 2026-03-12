//! Built-in post-processors for the graph export pipeline.
//!
//! Post-processors run after all nodes and edges are generated (including budget
//! enforcement). They transform the [`GraphExportResult`] in-place.
//!
//! ## Included Post-Processors
//!
//! | Name                          | Purpose                                          |
//! |-------------------------------|--------------------------------------------------|
//! | `EffectiveControlCountPatcher`| Count mitigating + effective controls per risk    |
//! | `AnomalyFlagNormalizer`       | Ensure `is_anomaly` + `isAnomalous` both set      |
//! | `RedFlagAnnotator`            | Mark parent document nodes with `hasRedFlag`      |
//! | `DuplicateEdgeValidator`      | Remove duplicate (source, target, edge_type) edges|

use std::collections::{HashMap, HashSet};

use tracing::debug;

use crate::config::ExportConfig;
use crate::error::ExportError;
use crate::id_map::IdMap;
use crate::traits::PostProcessor;
use crate::types::GraphExportResult;

use datasynth_runtime::EnhancedGenerationResult;

// ──────────────────────────── Factory ────────────────────────────

/// Returns all built-in post-processors in their recommended execution order.
///
/// Order matters:
/// 1. `DuplicateEdgeValidator` — dedup edges before counting.
/// 2. `EffectiveControlCountPatcher` — needs clean edge set.
/// 3. `AnomalyFlagNormalizer` — normalize flags.
/// 4. `RedFlagAnnotator` — annotate parent documents.
pub fn all_post_processors() -> Vec<Box<dyn PostProcessor>> {
    vec![
        Box::new(DuplicateEdgeValidator),
        Box::new(EffectiveControlCountPatcher),
        Box::new(AnomalyFlagNormalizer),
        Box::new(RedFlagAnnotator),
    ]
}

// ──────────────────── EffectiveControlCountPatcher ───────────────

/// Counts RISK_MITIGATED_BY edges (code 75) per risk node and annotates
/// each risk node with `mitigatingControlCount` and `effectiveControlCount`.
///
/// For each risk → control edge (type 75), checks if the target control's
/// `effectiveness` property is `"Effective"` or `"effective"`.
pub struct EffectiveControlCountPatcher;

/// Edge type code for RISK_MITIGATED_BY.
const RISK_MITIGATED_BY: u32 = 75;

impl PostProcessor for EffectiveControlCountPatcher {
    fn name(&self) -> &'static str {
        "EffectiveControlCountPatcher"
    }

    fn process(
        &self,
        result: &mut GraphExportResult,
        _ds_result: &EnhancedGenerationResult,
        _config: &ExportConfig,
        _id_map: &IdMap,
    ) -> Result<(), ExportError> {
        // 1. Build a map from node_id → effectiveness value for control nodes.
        let control_effectiveness: HashMap<u64, bool> = result
            .nodes
            .iter()
            .filter(|n| n.node_type_name == "internal_control")
            .filter_map(|n| {
                let node_id = n.id?;
                let is_effective = n
                    .properties
                    .get("effectiveness")
                    .and_then(|v| v.as_str())
                    .map(|s| s.eq_ignore_ascii_case("effective"))
                    .unwrap_or(false);
                Some((node_id, is_effective))
            })
            .collect();

        // 2. Count RISK_MITIGATED_BY edges per risk (source) node.
        //    source = risk, target = control.
        let mut mitigating_count: HashMap<u64, usize> = HashMap::new();
        let mut effective_count: HashMap<u64, usize> = HashMap::new();

        for edge in &result.edges {
            if edge.edge_type == RISK_MITIGATED_BY {
                *mitigating_count.entry(edge.source).or_insert(0) += 1;
                if control_effectiveness
                    .get(&edge.target)
                    .copied()
                    .unwrap_or(false)
                {
                    *effective_count.entry(edge.source).or_insert(0) += 1;
                }
            }
        }

        // 3. Annotate risk nodes.
        let mut patched = 0usize;
        for node in &mut result.nodes {
            if node.node_type_name == "risk_assessment" {
                if let Some(node_id) = node.id {
                    let mc = mitigating_count.get(&node_id).copied().unwrap_or(0);
                    let ec = effective_count.get(&node_id).copied().unwrap_or(0);
                    node.properties
                        .insert("mitigatingControlCount".into(), serde_json::json!(mc));
                    node.properties
                        .insert("effectiveControlCount".into(), serde_json::json!(ec));
                    patched += 1;
                }
            }
        }

        debug!("EffectiveControlCountPatcher: patched {patched} risk nodes");
        Ok(())
    }
}

// ──────────────────── AnomalyFlagNormalizer ─────────────────────

/// Ensures both `is_anomaly` and `isAnomalous` properties are set on
/// every node that has either flag. This provides backward compatibility
/// for consumers that expect one naming convention or the other.
pub struct AnomalyFlagNormalizer;

impl PostProcessor for AnomalyFlagNormalizer {
    fn name(&self) -> &'static str {
        "AnomalyFlagNormalizer"
    }

    fn process(
        &self,
        result: &mut GraphExportResult,
        _ds_result: &EnhancedGenerationResult,
        _config: &ExportConfig,
        _id_map: &IdMap,
    ) -> Result<(), ExportError> {
        let mut normalized = 0usize;

        for node in &mut result.nodes {
            let has_snake = is_truthy(node.properties.get("is_anomaly"));
            let has_camel = is_truthy(node.properties.get("isAnomalous"));

            if has_snake || has_camel {
                if !has_snake {
                    node.properties
                        .insert("is_anomaly".into(), serde_json::json!(true));
                }
                if !has_camel {
                    node.properties
                        .insert("isAnomalous".into(), serde_json::json!(true));
                }
                normalized += 1;
            }
        }

        debug!("AnomalyFlagNormalizer: normalized {normalized} anomalous nodes");
        Ok(())
    }
}

/// Check if a JSON value is truthy (true bool, or truthy string/number).
fn is_truthy(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(serde_json::Value::String(s)) => s == "true" || s == "1",
        _ => false,
    }
}

// ──────────────────── RedFlagAnnotator ──────────────────────────

/// For each `red_flag` node (type 510), finds the parent document node
/// by matching the red flag's `documentId` property to a node in the ID
/// map, and annotates that document node with `hasRedFlag: true`.
pub struct RedFlagAnnotator;

impl PostProcessor for RedFlagAnnotator {
    fn name(&self) -> &'static str {
        "RedFlagAnnotator"
    }

    fn process(
        &self,
        result: &mut GraphExportResult,
        _ds_result: &EnhancedGenerationResult,
        _config: &ExportConfig,
        id_map: &IdMap,
    ) -> Result<(), ExportError> {
        // 1. Collect document node IDs that have a red flag pointing at them.
        let mut flagged_node_ids: HashSet<u64> = HashSet::new();

        for node in &result.nodes {
            if node.node_type == 510 || node.node_type_name == "red_flag" {
                if let Some(doc_id_value) = node.properties.get("documentId") {
                    if let Some(doc_id) = doc_id_value.as_str() {
                        if let Some(numeric_id) = id_map.get(doc_id) {
                            flagged_node_ids.insert(numeric_id);
                        }
                    }
                }
            }
        }

        // 2. Annotate the parent document nodes.
        let mut annotated = 0usize;
        for node in &mut result.nodes {
            if let Some(node_id) = node.id {
                if flagged_node_ids.contains(&node_id) {
                    node.properties
                        .insert("hasRedFlag".into(), serde_json::json!(true));
                    annotated += 1;
                }
            }
        }

        debug!(
            "RedFlagAnnotator: {} red flags pointed at {} document nodes",
            flagged_node_ids.len(),
            annotated
        );
        Ok(())
    }
}

// ──────────────────── DuplicateEdgeValidator ────────────────────

/// Removes duplicate edges (same source + target + edge_type).
///
/// Uses a `HashSet` keyed on `(source, target, edge_type)` for O(n) dedup.
/// When duplicates are found, the first occurrence is kept.
pub struct DuplicateEdgeValidator;

impl PostProcessor for DuplicateEdgeValidator {
    fn name(&self) -> &'static str {
        "DuplicateEdgeValidator"
    }

    fn process(
        &self,
        result: &mut GraphExportResult,
        _ds_result: &EnhancedGenerationResult,
        _config: &ExportConfig,
        _id_map: &IdMap,
    ) -> Result<(), ExportError> {
        let original_count = result.edges.len();
        let mut seen: HashSet<(u64, u64, u32)> = HashSet::with_capacity(original_count);

        result
            .edges
            .retain(|e| seen.insert((e.source, e.target, e.edge_type)));

        let removed = original_count - result.edges.len();
        if removed > 0 {
            debug!("DuplicateEdgeValidator: removed {removed} duplicate edges");
            // Update metadata to reflect the new edge count.
            result.metadata.total_edges = result.edges.len();
        }

        Ok(())
    }
}

// ──────────────────────────── Tests ─────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::error::ExportWarnings;
    use crate::types::{ExportEdge, ExportMetadata, ExportNode};

    /// Helper: create a minimal GraphExportResult with given nodes and edges.
    fn make_result(nodes: Vec<ExportNode>, edges: Vec<ExportEdge>) -> GraphExportResult {
        let edge_count = edges.len();
        let node_count = nodes.len();
        GraphExportResult {
            nodes,
            edges,
            ocel: None,
            ground_truth: Vec::new(),
            feature_vectors: Vec::new(),
            hyperedges: Vec::new(),
            metadata: ExportMetadata {
                total_nodes: node_count,
                total_edges: edge_count,
                nodes_per_layer: [0; 3],
                edge_types_produced: Vec::new(),
                duration_ms: 0,
            },
            warnings: ExportWarnings::new(),
        }
    }

    /// Helper: create a risk_assessment node.
    fn risk_node(id: u64, _status: &str) -> ExportNode {
        let mut props = HashMap::new();
        props.insert("status".into(), serde_json::json!("active"));
        ExportNode {
            id: Some(id),
            node_type: 102,
            node_type_name: "risk_assessment".into(),
            label: format!("Risk-{id}"),
            layer: 1,
            properties: props,
        }
    }

    /// Helper: create an internal_control node with given effectiveness.
    fn control_node(id: u64, effectiveness: &str) -> ExportNode {
        let mut props = HashMap::new();
        props.insert("effectiveness".into(), serde_json::json!(effectiveness));
        ExportNode {
            id: Some(id),
            node_type: 100,
            node_type_name: "internal_control".into(),
            label: format!("Control-{id}"),
            layer: 1,
            properties: props,
        }
    }

    /// Helper: create a document node (e.g., purchase_order).
    fn document_node(id: u64, type_name: &str, external_id: &str) -> ExportNode {
        let mut props = HashMap::new();
        props.insert("externalId".into(), serde_json::json!(external_id));
        ExportNode {
            id: Some(id),
            node_type: 200,
            node_type_name: type_name.into(),
            label: external_id.to_string(),
            layer: 2,
            properties: props,
        }
    }

    /// Helper: create a red_flag node pointing at a document.
    fn red_flag_node(id: u64, document_id: &str) -> ExportNode {
        let mut props = HashMap::new();
        props.insert("documentId".into(), serde_json::json!(document_id));
        props.insert("nodeTypeName".into(), serde_json::json!("red_flag"));
        ExportNode {
            id: Some(id),
            node_type: 510,
            node_type_name: "red_flag".into(),
            label: format!("Red Flag on {document_id}"),
            layer: 1,
            properties: props,
        }
    }

    /// Helper: create a stub EnhancedGenerationResult (tests don't use it).
    fn stub_ds_result() -> EnhancedGenerationResult {
        EnhancedGenerationResult::default()
    }

    // ─── EffectiveControlCountPatcher ───────────────────────────

    #[test]
    fn effective_control_count_patcher_sets_correct_counts() {
        let nodes = vec![
            risk_node(1, "active"),
            control_node(2, "Effective"),
            control_node(3, "Not Tested"),
        ];
        let edges = vec![
            ExportEdge {
                source: 1,
                target: 2,
                edge_type: 75,
                weight: 1.0,
                properties: HashMap::new(),
            },
            ExportEdge {
                source: 1,
                target: 3,
                edge_type: 75,
                weight: 1.0,
                properties: HashMap::new(),
            },
        ];

        let mut result = make_result(nodes, edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        EffectiveControlCountPatcher
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        let risk = &result.nodes[0];
        assert_eq!(risk.properties["mitigatingControlCount"], serde_json::json!(2));
        assert_eq!(risk.properties["effectiveControlCount"], serde_json::json!(1));
    }

    #[test]
    fn effective_control_count_patcher_handles_no_edges() {
        let nodes = vec![risk_node(1, "active")];
        let edges = vec![];

        let mut result = make_result(nodes, edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        EffectiveControlCountPatcher
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        let risk = &result.nodes[0];
        assert_eq!(risk.properties["mitigatingControlCount"], serde_json::json!(0));
        assert_eq!(risk.properties["effectiveControlCount"], serde_json::json!(0));
    }

    #[test]
    fn effective_control_count_case_insensitive() {
        let nodes = vec![
            risk_node(1, "active"),
            control_node(2, "effective"), // lowercase
            control_node(3, "EFFECTIVE"), // uppercase
        ];
        let edges = vec![
            ExportEdge {
                source: 1,
                target: 2,
                edge_type: 75,
                weight: 1.0,
                properties: HashMap::new(),
            },
            ExportEdge {
                source: 1,
                target: 3,
                edge_type: 75,
                weight: 1.0,
                properties: HashMap::new(),
            },
        ];

        let mut result = make_result(nodes, edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        EffectiveControlCountPatcher
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        let risk = &result.nodes[0];
        assert_eq!(risk.properties["effectiveControlCount"], serde_json::json!(2));
    }

    #[test]
    fn effective_control_count_ignores_non_risk_mitigated_edges() {
        let nodes = vec![
            risk_node(1, "active"),
            control_node(2, "Effective"),
        ];
        // Edge type 60 (document chain), not 75
        let edges = vec![ExportEdge {
            source: 1,
            target: 2,
            edge_type: 60,
            weight: 1.0,
            properties: HashMap::new(),
        }];

        let mut result = make_result(nodes, edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        EffectiveControlCountPatcher
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        let risk = &result.nodes[0];
        assert_eq!(risk.properties["mitigatingControlCount"], serde_json::json!(0));
    }

    // ─── AnomalyFlagNormalizer ──────────────────────────────────

    #[test]
    fn anomaly_normalizer_adds_missing_camel_case() {
        let mut props = HashMap::new();
        props.insert("is_anomaly".into(), serde_json::json!(true));
        let node = ExportNode {
            id: Some(1),
            node_type: 200,
            node_type_name: "purchase_order".into(),
            label: "PO-001".into(),
            layer: 2,
            properties: props,
        };

        let mut result = make_result(vec![node], vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        AnomalyFlagNormalizer
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert_eq!(result.nodes[0].properties["isAnomalous"], serde_json::json!(true));
        assert_eq!(result.nodes[0].properties["is_anomaly"], serde_json::json!(true));
    }

    #[test]
    fn anomaly_normalizer_adds_missing_snake_case() {
        let mut props = HashMap::new();
        props.insert("isAnomalous".into(), serde_json::json!(true));
        let node = ExportNode {
            id: Some(1),
            node_type: 200,
            node_type_name: "vendor".into(),
            label: "V-001".into(),
            layer: 2,
            properties: props,
        };

        let mut result = make_result(vec![node], vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        AnomalyFlagNormalizer
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert_eq!(result.nodes[0].properties["is_anomaly"], serde_json::json!(true));
    }

    #[test]
    fn anomaly_normalizer_skips_non_anomalous() {
        let node = ExportNode {
            id: Some(1),
            node_type: 200,
            node_type_name: "vendor".into(),
            label: "V-001".into(),
            layer: 2,
            properties: HashMap::new(),
        };

        let mut result = make_result(vec![node], vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        AnomalyFlagNormalizer
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert!(!result.nodes[0].properties.contains_key("is_anomaly"));
        assert!(!result.nodes[0].properties.contains_key("isAnomalous"));
    }

    #[test]
    fn anomaly_normalizer_handles_false_flags() {
        let mut props = HashMap::new();
        props.insert("is_anomaly".into(), serde_json::json!(false));
        let node = ExportNode {
            id: Some(1),
            node_type: 200,
            node_type_name: "vendor".into(),
            label: "V-001".into(),
            layer: 2,
            properties: props,
        };

        let mut result = make_result(vec![node], vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        AnomalyFlagNormalizer
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        // false is not truthy, so no normalization
        assert!(!result.nodes[0].properties.contains_key("isAnomalous"));
    }

    // ─── RedFlagAnnotator ───────────────────────────────────────

    #[test]
    fn red_flag_annotator_marks_parent_document() {
        let nodes = vec![
            document_node(1, "purchase_order", "PO-001"),
            document_node(2, "vendor_invoice", "VI-001"),
            red_flag_node(3, "PO-001"),
        ];

        let mut result = make_result(nodes, vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let mut id_map = IdMap::new();
        // Register the document IDs in the id_map
        let po_id = id_map.get_or_insert("PO-001");
        assert_eq!(po_id, 1);
        id_map.get_or_insert("VI-001"); // id=2
        // We need node IDs to match id_map, so manually set id_map to start from 1
        // The id_map was created fresh, so PO-001=1, VI-001=2

        RedFlagAnnotator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        // PO-001 should have hasRedFlag
        assert_eq!(result.nodes[0].properties["hasRedFlag"], serde_json::json!(true));
        // VI-001 should not
        assert!(!result.nodes[1].properties.contains_key("hasRedFlag"));
    }

    #[test]
    fn red_flag_annotator_no_flags_is_noop() {
        let nodes = vec![document_node(1, "purchase_order", "PO-001")];

        let mut result = make_result(nodes, vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        RedFlagAnnotator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert!(!result.nodes[0].properties.contains_key("hasRedFlag"));
    }

    #[test]
    fn red_flag_annotator_missing_document_is_tolerated() {
        // Red flag points at a document that doesn't exist in the id_map
        let nodes = vec![red_flag_node(1, "NONEXISTENT-DOC")];

        let mut result = make_result(nodes, vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        // Should not panic
        RedFlagAnnotator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();
    }

    // ─── DuplicateEdgeValidator ─────────────────────────────────

    #[test]
    fn duplicate_edge_validator_removes_duplicates() {
        let edge = ExportEdge {
            source: 1,
            target: 2,
            edge_type: 60,
            weight: 1.0,
            properties: HashMap::new(),
        };
        let edges = vec![edge.clone(), edge.clone(), edge];

        let mut result = make_result(vec![], edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        DuplicateEdgeValidator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.metadata.total_edges, 1);
    }

    #[test]
    fn duplicate_edge_validator_keeps_different_types() {
        let edges = vec![
            ExportEdge {
                source: 1,
                target: 2,
                edge_type: 60,
                weight: 1.0,
                properties: HashMap::new(),
            },
            ExportEdge {
                source: 1,
                target: 2,
                edge_type: 75,
                weight: 1.0,
                properties: HashMap::new(),
            },
        ];

        let mut result = make_result(vec![], edges);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        DuplicateEdgeValidator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert_eq!(result.edges.len(), 2);
    }

    #[test]
    fn duplicate_edge_validator_empty_is_noop() {
        let mut result = make_result(vec![], vec![]);
        let ds = stub_ds_result();
        let config = ExportConfig::default();
        let id_map = IdMap::new();

        DuplicateEdgeValidator
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert_eq!(result.edges.len(), 0);
    }

    // ─── is_truthy ──────────────────────────────────────────────

    #[test]
    fn is_truthy_various_values() {
        assert!(is_truthy(Some(&serde_json::json!(true))));
        assert!(!is_truthy(Some(&serde_json::json!(false))));
        assert!(is_truthy(Some(&serde_json::json!(1))));
        assert!(!is_truthy(Some(&serde_json::json!(0))));
        assert!(is_truthy(Some(&serde_json::json!("true"))));
        assert!(is_truthy(Some(&serde_json::json!("1"))));
        assert!(!is_truthy(Some(&serde_json::json!("false"))));
        assert!(!is_truthy(None));
        assert!(!is_truthy(Some(&serde_json::json!(null))));
    }

    // ─── all_post_processors ────────────────────────────────────

    #[test]
    fn all_post_processors_returns_four() {
        let procs = all_post_processors();
        assert_eq!(procs.len(), 4);
        assert_eq!(procs[0].name(), "DuplicateEdgeValidator");
        assert_eq!(procs[1].name(), "EffectiveControlCountPatcher");
        assert_eq!(procs[2].name(), "AnomalyFlagNormalizer");
        assert_eq!(procs[3].name(), "RedFlagAnnotator");
    }
}
