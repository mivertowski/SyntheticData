//! AML Alert node synthesizer.
//!
//! Creates `aml_alert` nodes (entity code 505) from suspicious banking transactions.
//! Each suspicious transaction generates one AML alert node.

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes AML alert nodes from suspicious banking transactions.
pub struct AmlAlertNodeSynthesizer;

impl NodeSynthesizer for AmlAlertNodeSynthesizer {
    fn name(&self) -> &'static str {
        "aml_alert"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let txns = &ctx.ds_result.banking.transactions;

        let suspicious: Vec<_> = txns.iter().filter(|t| t.is_suspicious).collect();

        if suspicious.is_empty() {
            debug!("AmlAlertNodeSynthesizer: no suspicious transactions, skipping");
            return Ok(nodes);
        }

        debug!(
            "AmlAlertNodeSynthesizer: creating {} AML alert nodes",
            suspicious.len()
        );

        for txn in &suspicious {
            let external_id = format!("AML-ALERT-{}", txn.transaction_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("alertId".into(), serde_json::json!(external_id));
            props.insert(
                "alertType".into(),
                serde_json::json!(txn
                    .suspicion_reason
                    .as_ref()
                    .map(|r| format!("{r:?}"))
                    .unwrap_or_else(|| "Unknown".to_string())),
            );
            props.insert(
                "severity".into(),
                serde_json::json!(if txn.amount.to_f64().unwrap_or(0.0) > 50000.0 {
                    "high"
                } else if txn.amount.to_f64().unwrap_or(0.0) > 10000.0 {
                    "medium"
                } else {
                    "low"
                }),
            );
            props.insert(
                "amount".into(),
                serde_json::json!(txn.amount.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "accountId".into(),
                serde_json::json!(txn.account_id.to_string()),
            );
            if let Some(ref case) = txn.case_id {
                props.insert("caseId".into(), serde_json::json!(case));
            }
            props.insert("status".into(), serde_json::json!("open"));
            props.insert(
                "timestamp".into(),
                serde_json::json!(txn.timestamp_initiated.to_rfc3339()),
            );
            props.insert("nodeTypeName".into(), serde_json::json!("aml_alert"));
            props.insert("processFamily".into(), serde_json::json!("BANK"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 505,
                node_type_name: "aml_alert".into(),
                label: format!("AML Alert {}", txn.transaction_id),
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
        let s = AmlAlertNodeSynthesizer;
        assert_eq!(s.name(), "aml_alert");
    }
}
