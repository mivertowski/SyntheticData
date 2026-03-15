//! Process sequence edge synthesizer.
//!
//! Produces DirectlyFollows edges from OCEL/OCPM event logs to represent
//! process execution order.
//!
//! ## Edge Types Produced
//!
//! | Code | Name              | Direction               |
//! |------|-------------------|-------------------------|
//! | 121  | DIRECTLY_FOLLOWS  | event -> event           |

use std::collections::HashMap;

use tracing::debug;
use uuid::Uuid;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type code for process sequence.
const DIRECTLY_FOLLOWS: u32 = 121;

/// Synthesizes process sequence (directly-follows) edges from OCEL event logs.
pub struct ProcessSequenceEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for ProcessSequenceEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "process_sequence"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let Some(ref event_log) = ctx.ds_result.ocpm.event_log else {
            debug!("ProcessSequenceEdgeSynthesizer: no OCPM event log available");
            return Ok(Vec::new());
        };

        let events = &event_log.events;
        if events.is_empty() {
            debug!("ProcessSequenceEdgeSynthesizer: event log is empty");
            return Ok(Vec::new());
        }

        let mut edges = Vec::new();

        // Group events by case_id and sort by timestamp within each case.
        // Events without a case_id are skipped (can't determine sequence).
        let mut cases: HashMap<Uuid, Vec<(usize, chrono::DateTime<chrono::Utc>)>> = HashMap::new();
        for (idx, event) in events.iter().enumerate() {
            if let Some(case_id) = event.case_id {
                cases
                    .entry(case_id)
                    .or_default()
                    .push((idx, event.timestamp));
            }
        }

        for (_case_id, mut case_events) in cases {
            // Sort by timestamp
            case_events.sort_by_key(|(_, ts)| *ts);

            // Create directly-follows edges between consecutive events
            for window in case_events.windows(2) {
                let (prev_idx, _) = window[0];
                let (next_idx, _) = window[1];

                let prev_event = &events[prev_idx];
                let next_event = &events[next_idx];

                let prev_ext_id = prev_event.event_id.to_string();
                let next_ext_id = next_event.event_id.to_string();

                let Some(prev_id) = ctx.id_map.get(&prev_ext_id) else {
                    continue;
                };
                let Some(next_id) = ctx.id_map.get(&next_ext_id) else {
                    continue;
                };

                edges.push(ExportEdge {
                    source: prev_id,
                    target: next_id,
                    edge_type: DIRECTLY_FOLLOWS,
                    weight: 1.0,
                    properties: HashMap::new(),
                });
            }
        }

        debug!(
            "ProcessSequenceEdgeSynthesizer produced {} DIRECTLY_FOLLOWS edges",
            edges.len()
        );
        Ok(edges)
    }
}
