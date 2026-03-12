//! Temporal Events node synthesizer.
//!
//! Creates event nodes from temporal data sources:
//! - `process_evolution_event` (470) — process changes (workflow, automation, policy, control)
//! - `organizational_event` (471) — acquisitions, divestitures, reorgs, leadership changes
//! - `disruption_event` (472) — system outages, migrations, regulatory events
//!
//! These events represent temporal markers that affect the graph state over time.

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes temporal event nodes from process evolution, organizational, and disruption data.
pub struct TemporalEventsNodeSynthesizer;

impl NodeSynthesizer for TemporalEventsNodeSynthesizer {
    fn name(&self) -> &'static str {
        "temporal_events"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();

        let pe_count = ctx.ds_result.process_evolution.len();
        let oe_count = ctx.ds_result.organizational_events.len();
        let de_count = ctx.ds_result.disruption_events.len();

        let total = pe_count + oe_count + de_count;

        if total == 0 {
            debug!("TemporalEventsNodeSynthesizer: no temporal events, skipping");
            return Ok(nodes);
        }

        debug!("TemporalEventsNodeSynthesizer: synthesizing ~{total} temporal event nodes");

        // Process Evolution Events (470)
        for evt in &ctx.ds_result.process_evolution {
            let external_id = format!("PROC-EVT-{}", evt.event_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("eventId".into(), serde_json::json!(evt.event_id));
            props.insert(
                "eventType".into(),
                serde_json::json!(evt.event_type.type_name()),
            );
            props.insert(
                "timestamp".into(),
                serde_json::json!(format!("{}T00:00:00Z", evt.effective_date)),
            );
            if let Some(ref desc) = evt.description {
                props.insert("description".into(), serde_json::json!(desc));
            }
            props.insert(
                "severity".into(),
                serde_json::json!("medium"), // Process events are generally medium impact
            );
            if !evt.tags.is_empty() {
                props.insert(
                    "tags".into(),
                    serde_json::Value::Array(
                        evt.tags.iter().map(|t| serde_json::json!(t)).collect(),
                    ),
                );
            }
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("process_evolution_event"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 470,
                node_type_name: "process_evolution_event".into(),
                label: format!(
                    "Process: {} ({})",
                    evt.event_type.type_name(),
                    evt.effective_date
                ),
                layer: 2, // Process
                properties: props,
            });
        }

        // Organizational Events (471)
        for evt in &ctx.ds_result.organizational_events {
            let external_id = format!("ORG-EVT-{}", evt.event_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let event_type_name = match &evt.event_type {
                datasynth_core::models::organizational_event::OrganizationalEventType::Acquisition(_) => "acquisition",
                datasynth_core::models::organizational_event::OrganizationalEventType::Divestiture(_) => "divestiture",
                datasynth_core::models::organizational_event::OrganizationalEventType::Reorganization(_) => "reorganization",
                datasynth_core::models::organizational_event::OrganizationalEventType::LeadershipChange(_) => "leadership_change",
                datasynth_core::models::organizational_event::OrganizationalEventType::WorkforceReduction(_) => "workforce_reduction",
                datasynth_core::models::organizational_event::OrganizationalEventType::Merger(_) => "merger",
            };

            let mut props = HashMap::new();
            props.insert("eventId".into(), serde_json::json!(evt.event_id));
            props.insert(
                "eventType".into(),
                serde_json::json!(event_type_name),
            );
            props.insert(
                "timestamp".into(),
                serde_json::json!(format!("{}T00:00:00Z", evt.effective_date)),
            );
            if let Some(ref desc) = evt.description {
                props.insert("description".into(), serde_json::json!(desc));
            }
            props.insert("severity".into(), serde_json::json!("high"));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("organizational_event"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 471,
                node_type_name: "organizational_event".into(),
                label: format!(
                    "Org Event: {} ({})",
                    event_type_name, evt.effective_date
                ),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Disruption Events (472)
        for evt in &ctx.ds_result.disruption_events {
            let external_id = format!("DISRUPT-EVT-{}", evt.event_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("eventId".into(), serde_json::json!(evt.event_id));
            props.insert(
                "eventType".into(),
                serde_json::json!(format!("{:?}", evt.disruption_type)),
            );
            props.insert(
                "description".into(),
                serde_json::json!(evt.description),
            );
            props.insert("severity".into(), serde_json::json!(evt.severity));
            if !evt.affected_companies.is_empty() {
                props.insert(
                    "affectedCompanies".into(),
                    serde_json::Value::Array(
                        evt.affected_companies
                            .iter()
                            .map(|c| serde_json::json!(c))
                            .collect(),
                    ),
                );
            }
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("disruption_event"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 472,
                node_type_name: "disruption_event".into(),
                label: format!(
                    "Disruption: {:?} ({})",
                    evt.disruption_type, evt.event_id
                ),
                layer: 2, // Process
                properties: props,
            });
        }

        debug!(
            "TemporalEventsNodeSynthesizer: produced {} nodes",
            nodes.len()
        );
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = TemporalEventsNodeSynthesizer;
        assert_eq!(s.name(), "temporal_events");
    }
}
