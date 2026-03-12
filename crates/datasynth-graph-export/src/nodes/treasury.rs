//! Treasury node synthesizer.
//!
//! Creates treasury / cash management nodes from the treasury snapshot:
//! - `cash_position` (420)
//! - `cash_forecast` (421)
//! - `cash_pool` (422)
//! - `cash_pool_sweep` (423)
//! - `hedging_instrument` (424)
//! - `hedge_relationship` (425)
//! - `debt_instrument` (426)
//!
//! Also creates minimal nodes for:
//! - `bank_guarantee` (428) — no ToNodeProperties
//! - `netting_run` (429) — no ToNodeProperties
//!
//! Uses [`ToNodeProperties`] trait implementations for property serialization.

use std::collections::HashMap;

use datasynth_core::models::graph_properties::ToNodeProperties;
use tracing::debug;

use super::graph_props_to_json;
use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes all treasury / cash management nodes.
pub struct TreasuryNodeSynthesizer;

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
    props.insert("processFamily".into(), serde_json::json!("TCM"));

    ExportNode {
        id: Some(numeric_id),
        node_type: item.node_type_code() as u32,
        node_type_name: item.node_type_name().into(),
        label: label.to_string(),
        layer,
        properties: props,
    }
}

impl NodeSynthesizer for TreasuryNodeSynthesizer {
    fn name(&self) -> &'static str {
        "treasury"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let t = &ctx.ds_result.treasury;

        let total = t.cash_positions.len()
            + t.cash_forecasts.len()
            + t.cash_pools.len()
            + t.cash_pool_sweeps.len()
            + t.hedging_instruments.len()
            + t.hedge_relationships.len()
            + t.debt_instruments.len()
            + t.bank_guarantees.len()
            + t.netting_runs.len();

        if total == 0 {
            debug!("TreasuryNodeSynthesizer: treasury snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("TreasuryNodeSynthesizer: synthesizing ~{total} treasury nodes");

        // Cash Positions (420)
        for item in &t.cash_positions {
            let eid = format!("CASH-POS-{}", item.id);
            let label = format!("Cash Position: {} ({})", item.id, item.currency);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Cash Forecasts (421)
        for item in &t.cash_forecasts {
            let eid = format!("CASH-FCST-{}", item.id);
            let label = format!("Cash Forecast: {}", item.id);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Cash Pools (422)
        for item in &t.cash_pools {
            let eid = format!("CASH-POOL-{}", item.id);
            let label = format!("Cash Pool: {}", item.name);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Cash Pool Sweeps (423)
        for item in &t.cash_pool_sweeps {
            let eid = format!("SWEEP-{}", item.id);
            let label = format!("Cash Sweep: {}", item.id);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Hedging Instruments (424)
        for item in &t.hedging_instruments {
            let eid = format!("HEDGE-{}", item.id);
            let label = format!("Hedging: {} ({:?})", item.id, item.instrument_type);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Hedge Relationships (425)
        for item in &t.hedge_relationships {
            let eid = format!("HEDGE-REL-{}", item.id);
            let label = format!("Hedge Relationship: {}", item.id);
            nodes.push(synth_one(item, &eid, &label, 1, ctx));
        }

        // Debt Instruments (426)
        for item in &t.debt_instruments {
            let eid = format!("DEBT-{}", item.id);
            let label = format!("Debt: {} ({})", item.id, item.lender);
            nodes.push(synth_one(item, &eid, &label, 3, ctx));
        }

        // Bank Guarantees (428) — no ToNodeProperties
        for item in &t.bank_guarantees {
            let eid = format!("BANK-GUAR-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&eid);
            let mut props = HashMap::new();
            props.insert("guaranteeId".into(), serde_json::json!(item.id));
            props.insert("entityId".into(), serde_json::json!(item.entity_id));
            props.insert(
                "guaranteeType".into(),
                serde_json::json!(format!("{:?}", item.guarantee_type)),
            );
            props.insert("beneficiary".into(), serde_json::json!(item.beneficiary));
            props.insert("nodeTypeName".into(), serde_json::json!("bank_guarantee"));
            props.insert("processFamily".into(), serde_json::json!("TCM"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 428,
                node_type_name: "bank_guarantee".into(),
                label: format!("Bank Guarantee: {}", item.id),
                layer: 1,
                properties: props,
            });
        }

        // Netting Runs (429) — no ToNodeProperties
        for item in &t.netting_runs {
            let eid = format!("NETTING-{}", item.id);
            let numeric_id = ctx.id_map.get_or_insert(&eid);
            let mut props = HashMap::new();
            props.insert("nettingId".into(), serde_json::json!(item.id));
            props.insert(
                "nettingDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", item.netting_date)),
            );
            props.insert(
                "participantCount".into(),
                serde_json::json!(item.participating_entities.len()),
            );
            props.insert("nodeTypeName".into(), serde_json::json!("netting_run"));
            props.insert("processFamily".into(), serde_json::json!("TCM"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 429,
                node_type_name: "netting_run".into(),
                label: format!("Netting Run: {}", item.id),
                layer: 3,
                properties: props,
            });
        }

        debug!("TreasuryNodeSynthesizer: produced {} nodes", nodes.len());
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = TreasuryNodeSynthesizer;
        assert_eq!(s.name(), "treasury");
    }
}
