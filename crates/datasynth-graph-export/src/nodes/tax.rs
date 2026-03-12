//! Tax node synthesizer.
//!
//! Creates tax-domain nodes from the tax snapshot:
//! - `tax_jurisdiction` (410)
//! - `tax_code` (411)
//! - `tax_line` (412)
//! - `tax_return` (413)
//! - `tax_provision` (414)
//! - `withholding_tax_record` (415)
//! - `uncertain_tax_position` (416)
//!
//! Uses [`ToNodeProperties`] trait implementations for property serialization.

use datasynth_core::models::graph_properties::ToNodeProperties;
use tracing::debug;

use super::graph_props_to_json;
use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes all tax-domain nodes.
pub struct TaxNodeSynthesizer;

impl NodeSynthesizer for TaxNodeSynthesizer {
    fn name(&self) -> &'static str {
        "tax"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let tax = &ctx.ds_result.tax;

        let total = tax.jurisdictions.len()
            + tax.codes.len()
            + tax.tax_lines.len()
            + tax.tax_returns.len()
            + tax.tax_provisions.len()
            + tax.withholding_records.len();

        if total == 0 {
            debug!("TaxNodeSynthesizer: tax snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("TaxNodeSynthesizer: synthesizing ~{total} tax nodes");

        // Tax Jurisdictions (410)
        for item in &tax.jurisdictions {
            let external_id = format!("TAX-JURIS-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("Tax Jurisdiction: {}", item.name),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Tax Codes (411)
        for item in &tax.codes {
            let external_id = format!("TAX-CODE-{}", item.code);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("Tax Code: {}", item.code),
                layer: 3, // Accounting
                properties: props,
            });
        }

        // Tax Lines (412)
        for (i, item) in tax.tax_lines.iter().enumerate() {
            let external_id = format!("TAX-LINE-{}-{}", item.document_id, item.line_number);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("Tax Line #{} on {}", i + 1, item.document_id),
                layer: 3, // Accounting
                properties: props,
            });
        }

        // Tax Returns (413)
        for item in &tax.tax_returns {
            let external_id = format!("TAX-RETURN-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("Tax Return: {} ({})", item.id, item.entity_id),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Tax Provisions (414)
        for item in &tax.tax_provisions {
            let external_id = format!("TAX-PROV-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("Tax Provision: {} ({})", item.id, item.entity_id),
                layer: 3, // Accounting
                properties: props,
            });
        }

        // Withholding Tax Records (415)
        for item in &tax.withholding_records {
            let external_id = format!("WHT-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);
            let mut props = graph_props_to_json(item.to_node_properties());
            props.insert("nodeTypeName".into(), serde_json::json!(item.node_type_name()));
            props.insert("processFamily".into(), serde_json::json!("TAX"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: item.node_type_code() as u32,
                node_type_name: item.node_type_name().into(),
                label: format!("WHT: {} (vendor {})", item.id, item.vendor_id),
                layer: 3, // Accounting
                properties: props,
            });
        }

        debug!("TaxNodeSynthesizer: produced {} nodes", nodes.len());
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = TaxNodeSynthesizer;
        assert_eq!(s.name(), "tax");
    }
}
