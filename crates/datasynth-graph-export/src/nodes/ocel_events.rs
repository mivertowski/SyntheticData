//! OCEL Events node synthesizer.
//!
//! Creates `ocel_event` nodes (entity code 400) from the OCPM event log.
//! Each OCPM event becomes a node in the process layer, enabling graph-based
//! process mining queries.

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes OCEL event nodes from the OCPM event log.
pub struct OcelEventsNodeSynthesizer;

impl NodeSynthesizer for OcelEventsNodeSynthesizer {
    fn name(&self) -> &'static str {
        "ocel_events"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();

        let event_log = match &ctx.ds_result.ocpm.event_log {
            Some(log) => log,
            None => {
                debug!("OcelEventsNodeSynthesizer: no OCPM event log, skipping");
                return Ok(nodes);
            }
        };

        let events = &event_log.events;

        if events.is_empty() {
            debug!("OcelEventsNodeSynthesizer: OCPM event log is empty, skipping");
            return Ok(nodes);
        }

        // Cap at a reasonable number to avoid blowing up the graph budget.
        // Process mining usually works on event logs with 10K-100K events,
        // but graph visualization is limited to ~50K nodes.
        let max_events = ctx.config.budget.max_nodes.min(50_000);
        let event_count = events.len().min(max_events);

        debug!(
            "OcelEventsNodeSynthesizer: creating {} OCEL event nodes (of {} total)",
            event_count,
            events.len()
        );

        for event in events.iter().take(event_count) {
            let external_id = format!("OCEL-EVT-{}", event.event_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "eventId".into(),
                serde_json::json!(event.event_id.to_string()),
            );
            props.insert(
                "eventType".into(),
                serde_json::json!(event.activity_id),
            );
            props.insert(
                "timestamp".into(),
                serde_json::json!(event.timestamp.to_rfc3339()),
            );
            props.insert(
                "activityName".into(),
                serde_json::json!(event.activity_name),
            );
            props.insert(
                "lifecycle".into(),
                serde_json::json!(format!("{:?}", event.lifecycle)),
            );
            props.insert(
                "resourceId".into(),
                serde_json::json!(event.resource_id),
            );
            if let Some(ref res_name) = event.resource_name {
                props.insert("resourceName".into(), serde_json::json!(res_name));
            }
            props.insert(
                "companyCode".into(),
                serde_json::json!(event.company_code),
            );
            if let Some(ref doc_ref) = event.document_ref {
                props.insert("documentRef".into(), serde_json::json!(doc_ref));
            }
            if event.is_anomaly {
                props.insert("isAnomalous".into(), serde_json::json!(true));
                props.insert("is_anomaly".into(), serde_json::json!(true));
                if let Some(ref anomaly_type) = event.anomaly_type {
                    props.insert(
                        "anomalyType".into(),
                        serde_json::json!(anomaly_type),
                    );
                }
            }
            if let Some(ref case_id) = event.case_id {
                props.insert(
                    "caseId".into(),
                    serde_json::json!(case_id.to_string()),
                );
            }
            // Derive process family from activity name or document ref
            let process_family = derive_process_family(&event.activity_name, event.document_ref.as_deref());
            props.insert("processFamily".into(), serde_json::json!(process_family));
            props.insert("nodeTypeName".into(), serde_json::json!("ocel_event"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 400,
                node_type_name: "ocel_event".into(),
                label: format!("{} ({})", event.activity_name, event.event_id),
                layer: 2, // Process
                properties: props,
            });
        }

        if events.len() > max_events {
            ctx.warnings.info(
                "ocel_events",
                format!(
                    "Capped OCEL events at {} of {} to stay within node budget",
                    max_events,
                    events.len()
                ),
            );
        }

        debug!("OcelEventsNodeSynthesizer: produced {} nodes", nodes.len());
        Ok(nodes)
    }
}

/// Derive a process family code from the activity name or document reference.
fn derive_process_family(activity_name: &str, doc_ref: Option<&str>) -> &'static str {
    let haystack = activity_name.to_lowercase();
    let doc = doc_ref.unwrap_or("");

    if haystack.contains("purchase")
        || haystack.contains("goods_receipt")
        || haystack.contains("vendor")
        || doc.starts_with("PO-")
    {
        "P2P"
    } else if haystack.contains("sales")
        || haystack.contains("delivery")
        || haystack.contains("customer_invoice")
        || doc.starts_with("SO-")
    {
        "O2C"
    } else if haystack.contains("journal") || haystack.contains("posting") {
        "R2R"
    } else if haystack.contains("bank") || haystack.contains("payment") {
        "BANK"
    } else {
        "GENERAL"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = OcelEventsNodeSynthesizer;
        assert_eq!(s.name(), "ocel_events");
    }

    #[test]
    fn derive_process_family_maps_correctly() {
        assert_eq!(derive_process_family("Create Purchase Order", Some("PO-001")), "P2P");
        assert_eq!(derive_process_family("Create Sales Order", None), "O2C");
        assert_eq!(derive_process_family("Post Journal Entry", None), "R2R");
        assert_eq!(derive_process_family("Process Bank Payment", None), "BANK");
        assert_eq!(derive_process_family("Unknown Activity", None), "GENERAL");
    }
}
