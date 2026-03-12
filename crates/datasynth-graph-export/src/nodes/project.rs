//! Project accounting node synthesizer.
//!
//! Creates project accounting nodes from the project accounting snapshot:
//! - `project_cost_line` (451)
//! - `project_revenue` (452)
//! - `earned_value_metric` (453)
//! - `change_order` (454)
//! - `project_milestone` (455)
//!
//! Note: `project` entities (450) come from HypergraphBuilder. This synthesizer
//! handles the sub-entities that HypergraphBuilder doesn't produce.
//!
//! Uses [`ToNodeProperties`] trait implementations for property serialization.

use datasynth_core::models::graph_properties::ToNodeProperties;
use tracing::debug;

use super::graph_props_to_json;
use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes project accounting sub-entity nodes.
pub struct ProjectNodeSynthesizer;

/// Helper: create an ExportNode from a ToNodeProperties item.
fn synth_one(
    item: &dyn ToNodeProperties,
    external_id: &str,
    label: &str,
    layer: u8,
    ctx: &mut NodeSynthesisContext<'_>,
) -> ExportNode {
    let numeric_id = ctx.id_map.get_or_insert(external_id);
    let mut props = graph_props_to_json(item.to_node_properties());
    props.insert(
        "nodeTypeName".into(),
        serde_json::json!(item.node_type_name()),
    );
    props.insert("processFamily".into(), serde_json::json!("PROJECT"));

    ExportNode {
        id: Some(numeric_id),
        node_type: item.node_type_code() as u32,
        node_type_name: item.node_type_name().into(),
        label: label.to_string(),
        layer,
        properties: props,
    }
}

impl NodeSynthesizer for ProjectNodeSynthesizer {
    fn name(&self) -> &'static str {
        "project"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let p = &ctx.ds_result.project_accounting;

        let total = p.cost_lines.len()
            + p.revenue_records.len()
            + p.earned_value_metrics.len()
            + p.change_orders.len()
            + p.milestones.len();

        if total == 0 {
            debug!("ProjectNodeSynthesizer: project accounting snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("ProjectNodeSynthesizer: synthesizing ~{total} project nodes");

        // Project Cost Lines (451)
        for item in &p.cost_lines {
            let eid = format!("PROJ-COST-{}", item.id);
            let label = format!("Project Cost: {} ({})", item.id, item.project_id);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Project Revenue (452)
        for item in &p.revenue_records {
            let eid = format!("PROJ-REV-{}", item.id);
            let label = format!("Project Revenue: {} ({})", item.id, item.project_id);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Earned Value Metrics (453)
        for item in &p.earned_value_metrics {
            let eid = format!("EVM-{}", item.id);
            let label = format!("EVM: {} ({})", item.id, item.project_id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Change Orders (454)
        for item in &p.change_orders {
            let eid = format!("CHANGE-ORDER-{}", item.id);
            let label = format!("Change Order: {} ({})", item.id, item.project_id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Project Milestones (455)
        for item in &p.milestones {
            let eid = format!("MILESTONE-{}", item.id);
            let label = format!("Milestone: {} ({})", item.name, item.project_id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        debug!("ProjectNodeSynthesizer: produced {} nodes", nodes.len());
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = ProjectNodeSynthesizer;
        assert_eq!(s.name(), "project");
    }
}
