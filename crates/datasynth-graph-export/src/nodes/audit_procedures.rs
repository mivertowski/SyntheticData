//! Audit procedure node synthesizer.
//!
//! Creates nodes for the 9 audit procedure entity types added by ISA 505/330/530/520/610/550:
//! - `external_confirmation` (366)
//! - `confirmation_response` (367)
//! - `audit_procedure_step` (368)
//! - `audit_sample` (369)
//! - `analytical_procedure_result` (375)
//! - `internal_audit_function` (376)
//! - `internal_audit_report` (377)
//! - `related_party` (378)
//! - `related_party_transaction` (379) — Layer 2 (Process)

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes nodes for all audit procedure entity types (ISA 505/330/530/520/610/550).
pub struct AuditProcedureNodeSynthesizer;

impl NodeSynthesizer for AuditProcedureNodeSynthesizer {
    fn name(&self) -> &'static str {
        "audit_procedures"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let audit = &ctx.ds_result.audit;

        let total = audit.confirmations.len()
            + audit.confirmation_responses.len()
            + audit.procedure_steps.len()
            + audit.samples.len()
            + audit.analytical_results.len()
            + audit.ia_functions.len()
            + audit.ia_reports.len()
            + audit.related_parties.len()
            + audit.related_party_transactions.len();

        if total == 0 {
            debug!("AuditProcedureNodeSynthesizer: no audit procedure data, skipping");
            return Ok(nodes);
        }

        debug!("AuditProcedureNodeSynthesizer: synthesizing ~{total} audit procedure nodes");

        // ExternalConfirmation (366) — Layer 1 (Governance)
        for conf in &audit.confirmations {
            let external_id = format!("CONF-{}", conf.confirmation_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "confirmationRef".into(),
                serde_json::json!(conf.confirmation_ref),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("external_confirmation"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 366,
                node_type_name: "external_confirmation".into(),
                label: format!("Confirmation {}", conf.confirmation_ref),
                layer: 1,
                properties: props,
            });
        }

        // ConfirmationResponse (367) — Layer 1 (Governance)
        for resp in &audit.confirmation_responses {
            let external_id = format!("RESP-{}", resp.response_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("responseRef".into(), serde_json::json!(resp.response_ref));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("confirmation_response"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 367,
                node_type_name: "confirmation_response".into(),
                label: format!("Response {}", resp.response_ref),
                layer: 1,
                properties: props,
            });
        }

        // AuditProcedureStep (368) — Layer 1 (Governance)
        for step in &audit.procedure_steps {
            let external_id = format!("STEP-{}", step.step_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("stepRef".into(), serde_json::json!(step.step_ref));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("audit_procedure_step"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 368,
                node_type_name: "audit_procedure_step".into(),
                label: format!("Step {}", step.step_ref),
                layer: 1,
                properties: props,
            });
        }

        // AuditSample (369) — Layer 1 (Governance)
        for sample in &audit.samples {
            let external_id = format!("SAMP-{}", sample.sample_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("sampleRef".into(), serde_json::json!(sample.sample_ref));
            props.insert("nodeTypeName".into(), serde_json::json!("audit_sample"));
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 369,
                node_type_name: "audit_sample".into(),
                label: format!("Sample {}", sample.sample_ref),
                layer: 1,
                properties: props,
            });
        }

        // AnalyticalProcedureResult (375) — Layer 1 (Governance)
        for ap in &audit.analytical_results {
            let external_id = format!("AP-{}", ap.result_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("resultRef".into(), serde_json::json!(ap.result_ref));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("analytical_procedure_result"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 375,
                node_type_name: "analytical_procedure_result".into(),
                label: format!("AP {}", ap.result_ref),
                layer: 1,
                properties: props,
            });
        }

        // InternalAuditFunction (376) — Layer 1 (Governance)
        for iaf in &audit.ia_functions {
            let external_id = format!("IAF-{}", iaf.function_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("functionRef".into(), serde_json::json!(iaf.function_ref));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("internal_audit_function"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 376,
                node_type_name: "internal_audit_function".into(),
                label: format!("IAF {}", iaf.function_ref),
                layer: 1,
                properties: props,
            });
        }

        // InternalAuditReport (377) — Layer 1 (Governance)
        for iar in &audit.ia_reports {
            let external_id = format!("IAR-{}", iar.report_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("reportRef".into(), serde_json::json!(iar.report_ref));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("internal_audit_report"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 377,
                node_type_name: "internal_audit_report".into(),
                label: format!("IAR {}", iar.report_ref),
                layer: 1,
                properties: props,
            });
        }

        // RelatedParty (378) — Layer 1 (Governance)
        for rp in &audit.related_parties {
            let external_id = format!("RP-{}", rp.party_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("partyRef".into(), serde_json::json!(rp.party_ref));
            props.insert("nodeTypeName".into(), serde_json::json!("related_party"));
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 378,
                node_type_name: "related_party".into(),
                label: format!("Related Party {}", rp.party_name),
                layer: 1,
                properties: props,
            });
        }

        // RelatedPartyTransaction (379) — Layer 2 (Process)
        for rpt in &audit.related_party_transactions {
            let external_id = format!("RPT-{}", rpt.transaction_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "transactionRef".into(),
                serde_json::json!(rpt.transaction_ref),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("related_party_transaction"),
            );
            props.insert("processFamily".into(), serde_json::json!("AUDIT"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 379,
                node_type_name: "related_party_transaction".into(),
                label: format!("RPT {}", rpt.transaction_ref),
                layer: 2, // Process — financial event
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
        let s = AuditProcedureNodeSynthesizer;
        assert_eq!(s.name(), "audit_procedures");
    }
}
