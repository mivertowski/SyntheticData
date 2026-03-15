//! Subledger Reconciliation node synthesizer.
//!
//! Creates `subledger_reconciliation` nodes (entity code 463) from
//! GL-to-subledger reconciliation results.

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes subledger reconciliation nodes.
pub struct SubledgerReconNodeSynthesizer;

impl NodeSynthesizer for SubledgerReconNodeSynthesizer {
    fn name(&self) -> &'static str {
        "subledger_recon"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let recons = &ctx.ds_result.subledger_reconciliation;

        if recons.is_empty() {
            debug!("SubledgerReconNodeSynthesizer: no reconciliation results, skipping");
            return Ok(nodes);
        }

        debug!(
            "SubledgerReconNodeSynthesizer: creating {} reconciliation nodes",
            recons.len()
        );

        for recon in recons {
            let external_id = format!("RECON-{}", recon.reconciliation_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("reconId".into(), serde_json::json!(recon.reconciliation_id));
            props.insert(
                "subledger".into(),
                serde_json::json!(format!("{:?}", recon.subledger_type)),
            );
            props.insert("glAccount".into(), serde_json::json!(recon.gl_account));
            props.insert(
                "subledgerBalance".into(),
                serde_json::json!(recon.subledger_balance.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "glBalance".into(),
                serde_json::json!(recon.gl_balance.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "difference".into(),
                serde_json::json!(recon.difference.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "status".into(),
                serde_json::json!(format!("{:?}", recon.status)),
            );
            props.insert("companyCode".into(), serde_json::json!(recon.company_code));
            props.insert(
                "asOfDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", recon.as_of_date)),
            );
            props.insert(
                "unreconciledCount".into(),
                serde_json::json!(recon.unreconciled_items.len()),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("subledger_reconciliation"),
            );
            props.insert("processFamily".into(), serde_json::json!("R2R"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 463,
                node_type_name: "subledger_reconciliation".into(),
                label: format!(
                    "Recon: {} ({:?} → {})",
                    recon.reconciliation_id, recon.subledger_type, recon.gl_account
                ),
                layer: 3, // Accounting
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
        let s = SubledgerReconNodeSynthesizer;
        assert_eq!(s.name(), "subledger_recon");
    }
}
