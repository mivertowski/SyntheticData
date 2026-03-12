//! ESG node synthesizer.
//!
//! Creates ESG-domain nodes from the ESG snapshot:
//! - `emission_record` (430)
//! - `energy_consumption` (431)
//! - `water_usage` (432)
//! - `waste_record` (433)
//! - `workforce_diversity_metric` (434)
//! - `pay_equity_metric` (435)
//! - `safety_incident` (436)
//! - `safety_metric` (437)
//! - `governance_metric` (438)
//! - `supplier_esg_assessment` (439)
//! - `materiality_assessment` (440)
//! - `esg_disclosure` (441)
//! - `climate_scenario` (442)
//!
//! Uses [`ToNodeProperties`] trait implementations for property serialization.

use datasynth_core::models::graph_properties::ToNodeProperties;
use tracing::debug;

use super::graph_props_to_json;
use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes all ESG-domain nodes.
pub struct EsgNodeSynthesizer;

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
    props.insert("processFamily".into(), serde_json::json!("ESG"));

    ExportNode {
        id: Some(numeric_id),
        node_type: item.node_type_code() as u32,
        node_type_name: item.node_type_name().into(),
        label: label.to_string(),
        layer,
        properties: props,
    }
}

impl NodeSynthesizer for EsgNodeSynthesizer {
    fn name(&self) -> &'static str {
        "esg"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let e = &ctx.ds_result.esg;

        let total = e.emissions.len()
            + e.energy.len()
            + e.water.len()
            + e.waste.len()
            + e.diversity.len()
            + e.pay_equity.len()
            + e.safety_incidents.len()
            + e.safety_metrics.len()
            + e.governance.len()
            + e.supplier_assessments.len()
            + e.materiality.len()
            + e.disclosures.len()
            + e.climate_scenarios.len();

        if total == 0 {
            debug!("EsgNodeSynthesizer: ESG snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("EsgNodeSynthesizer: synthesizing ~{total} ESG nodes");

        // Emission Records (430)
        for (i, item) in e.emissions.iter().enumerate() {
            let eid = format!("ESG-EMISSION-{:06}", i + 1);
            let label = format!("Emission: {:?} ({})", item.scope, item.id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Energy Consumption (431)
        for (i, item) in e.energy.iter().enumerate() {
            let eid = format!("ESG-ENERGY-{:06}", i + 1);
            let label = format!("Energy: {:?} ({})", item.energy_source, item.id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Water Usage (432)
        for (i, item) in e.water.iter().enumerate() {
            let eid = format!("ESG-WATER-{:06}", i + 1);
            let label = format!("Water Usage: {:?} ({})", item.source, item.id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Waste Records (433)
        for (i, item) in e.waste.iter().enumerate() {
            let eid = format!("ESG-WASTE-{:06}", i + 1);
            let label = format!("Waste: {:?} ({})", item.waste_type, item.id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Workforce Diversity (434)
        for (i, item) in e.diversity.iter().enumerate() {
            let eid = format!("ESG-DIVERSITY-{:06}", i + 1);
            let label = format!("Diversity: {:?} ({})", item.dimension, item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Pay Equity (435)
        for (i, item) in e.pay_equity.iter().enumerate() {
            let eid = format!("ESG-PAYEQ-{:06}", i + 1);
            let label = format!("Pay Equity: {:?} ({})", item.dimension, item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Safety Incidents (436)
        for (i, item) in e.safety_incidents.iter().enumerate() {
            let eid = format!("ESG-SAFETY-INC-{:06}", i + 1);
            let label = format!("Safety Incident: {:?} ({})", item.incident_type, item.id);
            nodes.push(synth_one(item, &eid, &label, 2, ctx));
        }

        // Safety Metrics (437)
        for (i, item) in e.safety_metrics.iter().enumerate() {
            let eid = format!("ESG-SAFETY-MET-{:06}", i + 1);
            let label = format!("Safety Metric: {}", item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Governance Metrics (438)
        for (i, item) in e.governance.iter().enumerate() {
            let eid = format!("ESG-GOV-{:06}", i + 1);
            let label = format!("Governance: {}", item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Supplier ESG Assessments (439)
        for (i, item) in e.supplier_assessments.iter().enumerate() {
            let eid = format!("ESG-SUPPLIER-{:06}", i + 1);
            let label = format!("Supplier ESG: {}", item.vendor_id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Materiality Assessments (440)
        for (i, item) in e.materiality.iter().enumerate() {
            let eid = format!("ESG-MATERIALITY-{:06}", i + 1);
            let label = format!("Materiality: {}", item.topic);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // ESG Disclosures (441)
        for (i, item) in e.disclosures.iter().enumerate() {
            let eid = format!("ESG-DISCLOSURE-{:06}", i + 1);
            let label = format!("Disclosure: {:?} ({})", item.framework, item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Climate Scenarios (442)
        for (i, item) in e.climate_scenarios.iter().enumerate() {
            let eid = format!("ESG-CLIMATE-{:06}", i + 1);
            let label = format!("Climate: {:?} ({})", item.scenario_type, item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        debug!("EsgNodeSynthesizer: produced {} nodes", nodes.len());
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = EsgNodeSynthesizer;
        assert_eq!(s.name(), "esg");
    }
}
