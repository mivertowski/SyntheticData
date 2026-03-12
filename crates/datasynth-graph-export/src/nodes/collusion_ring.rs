//! Collusion Ring node synthesizer.
//!
//! Creates `collusion_ring` nodes (entity code 511) from coordinated fraud
//! network rings detected during the fraud injection pass.

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes collusion ring nodes from fraud networks.
pub struct CollusionRingNodeSynthesizer;

impl NodeSynthesizer for CollusionRingNodeSynthesizer {
    fn name(&self) -> &'static str {
        "collusion_ring"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let rings = &ctx.ds_result.collusion_rings;

        if rings.is_empty() {
            debug!("CollusionRingNodeSynthesizer: no collusion rings, skipping");
            return Ok(nodes);
        }

        debug!(
            "CollusionRingNodeSynthesizer: creating {} collusion ring nodes",
            rings.len()
        );

        for ring in rings {
            let external_id = format!("COLLUSION-{}", ring.ring_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "ringId".into(),
                serde_json::json!(ring.ring_id.to_string()),
            );
            props.insert(
                "ringType".into(),
                serde_json::json!(format!("{:?}", ring.ring_type)),
            );
            props.insert(
                "fraudCategory".into(),
                serde_json::json!(format!("{:?}", ring.fraud_category)),
            );
            props.insert(
                "memberCount".into(),
                serde_json::json!(ring.members.len()),
            );
            props.insert(
                "totalAmount".into(),
                serde_json::json!(ring.total_stolen.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "transactionCount".into(),
                serde_json::json!(ring.transaction_count),
            );
            props.insert(
                "status".into(),
                serde_json::json!(format!("{:?}", ring.status)),
            );
            props.insert(
                "formationDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", ring.formation_date)),
            );
            props.insert(
                "detectionRisk".into(),
                serde_json::json!(ring.detection_risk),
            );
            props.insert("activeMonths".into(), serde_json::json!(ring.active_months));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("collusion_ring"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 511,
                node_type_name: "collusion_ring".into(),
                label: format!(
                    "Collusion Ring: {:?} ({} members)",
                    ring.ring_type,
                    ring.members.len()
                ),
                layer: 1, // Governance
                properties: props,
            });
        }

        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = CollusionRingNodeSynthesizer;
        assert_eq!(s.name(), "collusion_ring");
    }
}
