//! Helper functions for node conversion, property injection, and entity classification.
//!
//! These are pure functions used by property serializers, node synthesizers, and the
//! pipeline orchestrator. They handle:
//! - Converting `HypergraphNode` to `ExportNode`
//! - Injecting standard properties (process family, anomaly flags, type name)
//! - Mapping entity types to process family codes

use std::collections::HashMap;

use datasynth_graph::models::hypergraph::HypergraphNode;

use crate::config::PropertyCase;
use crate::types::ExportNode;

/// Convert a `HypergraphNode` into an `ExportNode` with the given property map.
///
/// The returned node has `id: None` — the pipeline assigns IDs via the `IdMap`.
pub fn to_export_node(
    hg_node: &HypergraphNode,
    properties: HashMap<String, serde_json::Value>,
) -> ExportNode {
    ExportNode {
        id: None,
        node_type: hg_node.entity_type_code,
        node_type_name: hg_node.entity_type.clone(),
        label: hg_node.label.clone(),
        layer: hg_node.layer.index(),
        properties,
    }
}

/// Inject standard properties that every node should have.
///
/// This adds:
/// - `processFamily` — derived from the entity type name (P2P, O2C, R2R, etc.)
/// - `is_anomaly` / `isAnomalous` — if the node is flagged as anomalous
/// - `nodeTypeName` — the entity type name for downstream consumers
pub fn inject_standard_properties(
    props: &mut HashMap<String, serde_json::Value>,
    hg_node: &HypergraphNode,
    property_case: &PropertyCase,
) {
    if let Some(pf) = entity_type_process_family(&hg_node.entity_type) {
        props.insert(key("processFamily", property_case), pf.into());
    }
    if hg_node.is_anomaly {
        props.insert(key("is_anomaly", property_case), true.into());
        props.insert(key("isAnomalous", property_case), true.into());
    }
    props.insert(
        key("nodeTypeName", property_case),
        hg_node.entity_type.clone().into(),
    );
}

/// Convert a property key to the correct casing convention.
pub fn key(name: &str, case: &PropertyCase) -> String {
    match case {
        PropertyCase::CamelCase => name.to_string(),
        PropertyCase::SnakeCase => camel_to_snake(name),
    }
}

/// Convert a camelCase string to snake_case.
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

/// Map an entity type name (snake_case) to its process family code.
///
/// Returns `None` for entity types that don't belong to a specific process family
/// (e.g., generic governance nodes like COSO components).
pub fn entity_type_process_family(entity_type: &str) -> Option<&'static str> {
    match entity_type {
        s if s.contains("purchase_order")
            || s.contains("goods_receipt")
            || s.contains("vendor_invoice")
            || s.contains("payment") =>
        {
            Some("P2P")
        }
        s if s.contains("sales_order")
            || s.contains("delivery")
            || s.contains("customer_invoice") =>
        {
            Some("O2C")
        }
        s if s.contains("journal_entry") || s.contains("account") => Some("R2R"),
        s if s.contains("sourcing")
            || s.contains("rfx")
            || s.contains("bid")
            || s.contains("contract") =>
        {
            Some("S2C")
        }
        s if s.contains("payroll") || s.contains("time_entry") || s.contains("expense_report") => {
            Some("H2R")
        }
        s if s.contains("production")
            || s.contains("quality_inspection")
            || s.contains("cycle_count") =>
        {
            Some("MFG")
        }
        s if s.contains("bank") => Some("BANK"),
        s if s.contains("tax") => Some("TAX"),
        s if s.contains("cash")
            || s.contains("hedge")
            || s.contains("debt")
            || s.contains("treasury") =>
        {
            Some("TCM")
        }
        s if s.contains("project") || s.contains("earned_value") || s.contains("milestone") => {
            Some("PROJECT")
        }
        s if s.contains("emission")
            || s.contains("esg")
            || s.contains("disclosure")
            || s.contains("climate")
            || s.contains("supplier_esg") =>
        {
            Some("ESG")
        }
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_graph::models::hypergraph::HypergraphLayer;

    fn test_node(entity_type: &str, is_anomaly: bool) -> HypergraphNode {
        HypergraphNode {
            id: "test-1".to_string(),
            entity_type: entity_type.to_string(),
            entity_type_code: 100,
            layer: HypergraphLayer::ProcessEvents,
            external_id: "EXT-001".to_string(),
            label: "Test Node".to_string(),
            properties: HashMap::new(),
            features: vec![],
            is_anomaly,
            anomaly_type: None,
            is_aggregate: false,
            aggregate_count: 0,
        }
    }

    #[test]
    fn to_export_node_maps_fields() {
        let hg = test_node("vendor_invoice", false);
        let mut props = HashMap::new();
        props.insert("amount".to_string(), serde_json::json!(1000.0));

        let export = to_export_node(&hg, props);
        assert!(export.id.is_none());
        assert_eq!(export.node_type, 100);
        assert_eq!(export.node_type_name, "vendor_invoice");
        assert_eq!(export.label, "Test Node");
        assert_eq!(export.layer, 2); // ProcessEvents
        assert_eq!(export.properties.len(), 1);
    }

    #[test]
    fn inject_standard_properties_adds_process_family() {
        let hg = test_node("purchase_order", false);
        let mut props = HashMap::new();

        inject_standard_properties(&mut props, &hg, &PropertyCase::CamelCase);

        assert_eq!(props.get("processFamily"), Some(&serde_json::json!("P2P")));
        assert_eq!(
            props.get("nodeTypeName"),
            Some(&serde_json::json!("purchase_order"))
        );
        assert!(!props.contains_key("is_anomaly"));
    }

    #[test]
    fn inject_standard_properties_adds_anomaly_flags() {
        let hg = test_node("journal_entry", true);
        let mut props = HashMap::new();

        inject_standard_properties(&mut props, &hg, &PropertyCase::CamelCase);

        assert_eq!(props.get("is_anomaly"), Some(&serde_json::json!(true)));
        assert_eq!(props.get("isAnomalous"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn inject_standard_properties_snake_case() {
        let hg = test_node("purchase_order", true);
        let mut props = HashMap::new();

        inject_standard_properties(&mut props, &hg, &PropertyCase::SnakeCase);

        assert_eq!(props.get("process_family"), Some(&serde_json::json!("P2P")));
        assert_eq!(
            props.get("node_type_name"),
            Some(&serde_json::json!("purchase_order"))
        );
        assert_eq!(props.get("is_anomaly"), Some(&serde_json::json!(true)));
        assert_eq!(props.get("is_anomalous"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn camel_to_snake_conversions() {
        assert_eq!(camel_to_snake("processFamily"), "process_family");
        assert_eq!(camel_to_snake("isAnomalous"), "is_anomalous");
        assert_eq!(camel_to_snake("nodeTypeName"), "node_type_name");
        assert_eq!(camel_to_snake("already_snake"), "already_snake");
        assert_eq!(camel_to_snake("ABC"), "a_b_c");
    }

    #[test]
    fn process_family_mapping() {
        assert_eq!(entity_type_process_family("purchase_order"), Some("P2P"));
        assert_eq!(entity_type_process_family("goods_receipt"), Some("P2P"));
        assert_eq!(entity_type_process_family("vendor_invoice"), Some("P2P"));
        assert_eq!(entity_type_process_family("sales_order"), Some("O2C"));
        assert_eq!(entity_type_process_family("delivery"), Some("O2C"));
        assert_eq!(entity_type_process_family("journal_entry"), Some("R2R"));
        assert_eq!(entity_type_process_family("account"), Some("R2R"));
        assert_eq!(entity_type_process_family("sourcing"), Some("S2C"));
        assert_eq!(entity_type_process_family("payroll"), Some("H2R"));
        assert_eq!(entity_type_process_family("production"), Some("MFG"));
        assert_eq!(entity_type_process_family("bank_transaction"), Some("BANK"));
        assert_eq!(entity_type_process_family("tax_provision"), Some("TAX"));
        assert_eq!(entity_type_process_family("cash_position"), Some("TCM"));
        assert_eq!(entity_type_process_family("project_cost"), Some("PROJECT"));
        assert_eq!(entity_type_process_family("emission"), Some("ESG"));
        assert_eq!(entity_type_process_family("coso_component"), None);
        assert_eq!(entity_type_process_family("internal_control"), None);
    }
}
