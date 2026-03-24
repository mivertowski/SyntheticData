//! OCEL 2.0 projection of audit events for process mining.

use crate::event::AuditEvent;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Minimal OCEL 2.0 compatible output.
#[derive(Debug, Clone, Serialize)]
pub struct OcelProjection {
    pub ocel_version: String,
    pub object_types: Vec<String>,
    pub events: Vec<OcelEvent>,
    pub objects: Vec<OcelObject>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcelEvent {
    pub id: String,
    pub activity: String,
    pub timestamp: String,
    pub omap: Vec<String>,
    pub vmap: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcelObject {
    pub id: String,
    pub object_type: String,
    pub attributes: HashMap<String, String>,
}

/// Project audit events to OCEL 2.0 format.
pub fn project_to_ocel(events: &[AuditEvent]) -> OcelProjection {
    let mut object_types: HashSet<String> = HashSet::new();
    let mut objects: HashMap<String, OcelObject> = HashMap::new();
    let mut ocel_events = Vec::new();

    for event in events {
        // Each procedure is an object type
        object_types.insert(event.procedure_id.clone());

        // Create object for the procedure
        let obj_id = format!("proc_{}", event.procedure_id);
        objects.entry(obj_id.clone()).or_insert_with(|| OcelObject {
            id: obj_id.clone(),
            object_type: event.procedure_id.clone(),
            attributes: HashMap::new(),
        });

        // Create objects for evidence refs
        for ev_ref in &event.evidence_refs {
            let ev_obj_id = format!("evidence_{ev_ref}");
            object_types.insert("evidence".to_string());
            objects.entry(ev_obj_id.clone()).or_insert_with(|| {
                let mut attrs = HashMap::new();
                attrs.insert("ref".to_string(), ev_ref.clone());
                OcelObject {
                    id: ev_obj_id.clone(),
                    object_type: "evidence".to_string(),
                    attributes: attrs,
                }
            });
        }

        // Map event
        let mut omap = vec![format!("proc_{}", event.procedure_id)];
        for ev_ref in &event.evidence_refs {
            omap.push(format!("evidence_{ev_ref}"));
        }

        let mut vmap = HashMap::new();
        vmap.insert("phase".to_string(), event.phase_id.clone());
        vmap.insert("actor".to_string(), event.actor_id.clone());
        if let Some(ref from) = event.from_state {
            vmap.insert("from_state".to_string(), from.clone());
        }
        if let Some(ref to) = event.to_state {
            vmap.insert("to_state".to_string(), to.clone());
        }

        ocel_events.push(OcelEvent {
            id: event.event_id.to_string(),
            activity: event.command.clone(),
            timestamp: event.timestamp.to_string(),
            omap,
            vmap,
        });
    }

    OcelProjection {
        ocel_version: "2.0".to_string(),
        object_types: {
            let mut v: Vec<_> = object_types.into_iter().collect();
            v.sort();
            v
        },
        events: ocel_events,
        objects: objects.into_values().collect(),
    }
}

/// Export OCEL projection to JSON string.
pub fn export_ocel_to_json(events: &[AuditEvent]) -> Result<String, serde_json::Error> {
    let ocel = project_to_ocel(events);
    serde_json::to_string_pretty(&ocel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocel_projection() {
        use crate::context::EngagementContext;
        use crate::engine::AuditFsmEngine;
        use crate::loader::{default_overlay, BlueprintWithPreconditions};
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
        let result = engine
            .run_engagement(&EngagementContext::test_default())
            .unwrap();

        let ocel = project_to_ocel(&result.event_log);
        assert_eq!(ocel.ocel_version, "2.0");
        assert!(!ocel.events.is_empty());
        assert!(!ocel.object_types.is_empty());
        assert!(!ocel.objects.is_empty());

        // JSON roundtrip
        let json = export_ocel_to_json(&result.event_log).unwrap();
        assert!(json.contains("ocel_version"));
        assert!(json.contains("2.0"));
    }
}
