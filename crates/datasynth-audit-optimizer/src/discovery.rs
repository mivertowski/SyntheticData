//! Blueprint discovery from event logs.
//!
//! Given a `Vec<AuditEvent>`, infers the underlying procedure state machines
//! (states, transitions, initial/terminal states) and compares the result
//! against a reference [`AuditBlueprint`].

use std::collections::{HashMap, HashSet};

use datasynth_audit_fsm::event::AuditEvent;
use datasynth_audit_fsm::schema::AuditBlueprint;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A blueprint inferred from an observed event log.
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredBlueprint {
    /// One entry per unique `procedure_id` found in the event log.
    pub procedures: Vec<DiscoveredProcedure>,
    /// Unique phase identifiers observed across all events.
    pub phases: Vec<String>,
    /// Total number of events that were analysed.
    pub total_events_analyzed: usize,
}

/// The state machine inferred for a single procedure from the event log.
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredProcedure {
    /// Procedure identifier, taken directly from `AuditEvent::procedure_id`.
    pub id: String,
    /// Phase inferred from the first event belonging to this procedure.
    pub phase: String,
    /// All states encountered (union of `from_state` and `to_state` values).
    pub states: Vec<String>,
    /// Directed transitions as `(from_state, to_state)` pairs (deduplicated).
    pub transitions: Vec<(String, String)>,
    /// The `from_state` of the chronologically-first transition event.
    pub initial_state: String,
    /// States that appear as a `to_state` but never as a `from_state` in any
    /// subsequent transition — these are the natural terminal states.
    pub terminal_states: Vec<String>,
    /// Number of events (of any kind) associated with this procedure.
    pub event_count: usize,
}

/// Diff between a discovered blueprint and a reference blueprint.
#[derive(Debug, Clone, Serialize)]
pub struct BlueprintDiff {
    /// Procedure IDs that exist in both blueprints.
    pub matching_procedures: Vec<String>,
    /// Procedure IDs present in the reference but absent from the discovered
    /// blueprint (i.e. not observed in the event log).
    pub missing_procedures: Vec<String>,
    /// Procedure IDs present in the discovered blueprint but absent from the
    /// reference (i.e. unexpected procedures in the event log).
    pub extra_procedures: Vec<String>,
    /// Transition-level differences for procedures that appear in both.
    pub transition_diffs: Vec<TransitionDiff>,
    /// Overall conformance score in `[0, 1]`:
    /// `matching / (matching + missing + extra)` where *matching*,
    /// *missing*, and *extra* are **procedure-level** counts.
    pub conformance_score: f64,
}

/// A single transition-level difference between discovered and reference.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionDiff {
    /// The procedure this difference belongs to.
    pub procedure_id: String,
    /// `"missing"` — the transition is in the reference but not discovered;
    /// `"extra"` — the transition is discovered but not in the reference.
    pub diff_type: String,
    /// Source state of the differing transition.
    pub from_state: String,
    /// Destination state of the differing transition.
    pub to_state: String,
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Infer a [`DiscoveredBlueprint`] from a slice of [`AuditEvent`] records.
///
/// Only events that carry both a `from_state` **and** a `to_state` are used
/// for state-machine reconstruction (i.e. transition events).  Pure step
/// events — where `step_id.is_some()` and the state fields are `None` — are
/// counted towards `event_count` but ignored for FSM inference.
///
/// Events are expected to be in the order they were emitted by the engine; the
/// function preserves that ordering when determining the initial state.
pub fn discover_blueprint(events: &[AuditEvent]) -> DiscoveredBlueprint {
    // -----------------------------------------------------------------------
    // 1. Group events by procedure_id, preserving arrival order.
    // -----------------------------------------------------------------------
    let mut proc_events: HashMap<String, Vec<&AuditEvent>> = HashMap::new();
    for event in events {
        proc_events
            .entry(event.procedure_id.clone())
            .or_default()
            .push(event);
    }

    // -----------------------------------------------------------------------
    // 2. For stable output order, sort procedures by the timestamp of their
    //    first event.
    // -----------------------------------------------------------------------
    let mut proc_ids: Vec<String> = proc_events.keys().cloned().collect();
    proc_ids.sort_by_key(|id| {
        proc_events[id]
            .first()
            .map(|e| e.timestamp)
            .unwrap_or_default()
    });

    // -----------------------------------------------------------------------
    // 3. Reconstruct a state machine for each procedure.
    // -----------------------------------------------------------------------
    let mut procedures: Vec<DiscoveredProcedure> = Vec::new();

    for id in &proc_ids {
        let evts = &proc_events[id];

        // Phase: from the first event in the group.
        let phase = evts
            .first()
            .map(|e| e.phase_id.as_str())
            .unwrap_or("")
            .to_string();
        let event_count = evts.len();

        // Only consider transition events (both from_state and to_state present).
        let transition_evts: Vec<&&AuditEvent> = evts
            .iter()
            .filter(|e| e.from_state.is_some() && e.to_state.is_some())
            .collect();

        // Collect states and transitions (preserving first-seen order while
        // deduplicating).
        let mut states_ordered: Vec<String> = Vec::new();
        let mut states_seen: HashSet<String> = HashSet::new();
        let mut transitions_ordered: Vec<(String, String)> = Vec::new();
        let mut transitions_seen: HashSet<(String, String)> = HashSet::new();

        // Track which states appear as from_state of any transition.
        let mut from_states_set: HashSet<String> = HashSet::new();

        for evt in &transition_evts {
            let from = evt.from_state.as_ref().unwrap().clone();
            let to = evt.to_state.as_ref().unwrap().clone();

            if states_seen.insert(from.clone()) {
                states_ordered.push(from.clone());
            }
            if states_seen.insert(to.clone()) {
                states_ordered.push(to.clone());
            }

            if transitions_seen.insert((from.clone(), to.clone())) {
                transitions_ordered.push((from.clone(), to.clone()));
            }

            from_states_set.insert(from);
        }

        // Initial state: from_state of the chronologically-first transition event.
        let initial_state = transition_evts
            .first()
            .and_then(|e| e.from_state.as_ref())
            .cloned()
            .unwrap_or_default();

        // Terminal states: appear as to_state but never as from_state of a
        // subsequent transition.
        let to_states_set: HashSet<String> = transition_evts
            .iter()
            .filter_map(|e| e.to_state.as_ref())
            .cloned()
            .collect();

        let mut terminal_states: Vec<String> = to_states_set
            .iter()
            .filter(|s| !from_states_set.contains(*s))
            .cloned()
            .collect();
        terminal_states.sort();

        procedures.push(DiscoveredProcedure {
            id: id.clone(),
            phase,
            states: states_ordered,
            transitions: transitions_ordered,
            initial_state,
            terminal_states,
            event_count,
        });
    }

    // -----------------------------------------------------------------------
    // 4. Collect unique phases (in the order they are first seen).
    // -----------------------------------------------------------------------
    let mut phases_ordered: Vec<String> = Vec::new();
    let mut phases_seen: HashSet<String> = HashSet::new();
    for proc in &procedures {
        if phases_seen.insert(proc.phase.clone()) {
            phases_ordered.push(proc.phase.clone());
        }
    }

    DiscoveredBlueprint {
        procedures,
        phases: phases_ordered,
        total_events_analyzed: events.len(),
    }
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// Compare a [`DiscoveredBlueprint`] against a reference [`AuditBlueprint`].
///
/// Returns a [`BlueprintDiff`] that describes:
/// - which procedures match, are missing, or are extra;
/// - which transitions within matching procedures are missing or extra;
/// - an overall conformance score.
///
/// The conformance score is computed at the **procedure level**:
/// ```text
/// score = |matching| / (|matching| + |missing| + |extra|)
/// ```
/// where *matching* = procedures in both, *missing* = in reference only,
/// *extra* = in discovered only.  A score of 1.0 means perfect structural
/// alignment.
pub fn compare_blueprints(
    discovered: &DiscoveredBlueprint,
    reference: &AuditBlueprint,
) -> BlueprintDiff {
    // -----------------------------------------------------------------------
    // 1. Build sets of procedure IDs from each blueprint.
    // -----------------------------------------------------------------------
    let discovered_ids: HashSet<String> =
        discovered.procedures.iter().map(|p| p.id.clone()).collect();

    let reference_ids: HashSet<String> = reference
        .phases
        .iter()
        .flat_map(|phase| phase.procedures.iter())
        .map(|p| p.id.clone())
        .collect();

    // -----------------------------------------------------------------------
    // 2. Compute set differences.
    // -----------------------------------------------------------------------
    let mut matching_procedures: Vec<String> = discovered_ids
        .intersection(&reference_ids)
        .cloned()
        .collect();
    matching_procedures.sort();

    let mut missing_procedures: Vec<String> =
        reference_ids.difference(&discovered_ids).cloned().collect();
    missing_procedures.sort();

    let mut extra_procedures: Vec<String> =
        discovered_ids.difference(&reference_ids).cloned().collect();
    extra_procedures.sort();

    // -----------------------------------------------------------------------
    // 3. Compare transitions for matching procedures.
    // -----------------------------------------------------------------------

    // Build a lookup from the discovered blueprint.
    let discovered_map: HashMap<&str, &DiscoveredProcedure> = discovered
        .procedures
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();

    // Build a lookup from the reference blueprint.
    let reference_transitions: HashMap<&str, HashSet<(String, String)>> = reference
        .phases
        .iter()
        .flat_map(|phase| phase.procedures.iter())
        .map(|p| {
            let set: HashSet<(String, String)> = p
                .aggregate
                .transitions
                .iter()
                .map(|t| (t.from_state.clone(), t.to_state.clone()))
                .collect();
            (p.id.as_str(), set)
        })
        .collect();

    let mut transition_diffs: Vec<TransitionDiff> = Vec::new();

    for proc_id in &matching_procedures {
        let disc_proc = match discovered_map.get(proc_id.as_str()) {
            Some(p) => p,
            None => continue,
        };
        let ref_transitions = match reference_transitions.get(proc_id.as_str()) {
            Some(t) => t,
            None => continue,
        };

        let disc_set: HashSet<(String, String)> = disc_proc.transitions.iter().cloned().collect();

        // Transitions in reference but not discovered → "missing"
        let mut missing_trans: Vec<&(String, String)> =
            ref_transitions.difference(&disc_set).collect();
        missing_trans.sort();
        for (from, to) in missing_trans {
            transition_diffs.push(TransitionDiff {
                procedure_id: proc_id.clone(),
                diff_type: "missing".to_string(),
                from_state: from.clone(),
                to_state: to.clone(),
            });
        }

        // Transitions in discovered but not reference → "extra"
        let mut extra_trans: Vec<&(String, String)> =
            disc_set.difference(ref_transitions).collect();
        extra_trans.sort();
        for (from, to) in extra_trans {
            transition_diffs.push(TransitionDiff {
                procedure_id: proc_id.clone(),
                diff_type: "extra".to_string(),
                from_state: from.clone(),
                to_state: to.clone(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // 4. Conformance score.
    // -----------------------------------------------------------------------
    let m = matching_procedures.len() as f64;
    let mi = missing_procedures.len() as f64;
    let ex = extra_procedures.len() as f64;
    let denominator = m + mi + ex;
    let conformance_score = if denominator > 0.0 {
        m / denominator
    } else {
        1.0
    };

    BlueprintDiff {
        matching_procedures,
        missing_procedures,
        extra_procedures,
        transition_diffs,
        conformance_score,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::benchmark::{
        generate_benchmark, BenchmarkComplexity, BenchmarkConfig,
    };
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    // -----------------------------------------------------------------------
    // Helper: generate FSA events
    // -----------------------------------------------------------------------
    fn fsa_events() -> Vec<AuditEvent> {
        generate_benchmark(&BenchmarkConfig {
            complexity: BenchmarkComplexity::Simple,
            anomaly_rate: None,
            seed: 42,
        })
        .unwrap()
        .events
    }

    // -----------------------------------------------------------------------
    // Helper: generate IA events
    // -----------------------------------------------------------------------
    fn ia_events() -> Vec<AuditEvent> {
        generate_benchmark(&BenchmarkConfig {
            complexity: BenchmarkComplexity::Complex,
            anomaly_rate: None,
            seed: 42,
        })
        .unwrap()
        .events
    }

    // -----------------------------------------------------------------------
    // Test 1: FSA events → 9 discovered procedures with states & transitions
    // -----------------------------------------------------------------------
    #[test]
    fn test_discover_from_fsa_events() {
        let events = fsa_events();
        let discovered = discover_blueprint(&events);

        assert_eq!(
            discovered.procedures.len(),
            9,
            "FSA blueprint has 9 procedures, got {}",
            discovered.procedures.len()
        );
        assert_eq!(discovered.total_events_analyzed, events.len());

        for proc in &discovered.procedures {
            assert!(
                !proc.states.is_empty(),
                "Procedure {} should have states",
                proc.id
            );
            assert!(
                !proc.transitions.is_empty(),
                "Procedure {} should have transitions",
                proc.id
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 2: IA events → >= 30 discovered procedures
    // -----------------------------------------------------------------------
    #[test]
    fn test_discover_from_ia_events() {
        let events = ia_events();
        let discovered = discover_blueprint(&events);

        assert!(
            discovered.procedures.len() >= 30,
            "IA blueprint should yield >= 30 discovered procedures, got {}",
            discovered.procedures.len()
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: States for accept_engagement match expected set
    // -----------------------------------------------------------------------
    #[test]
    fn test_discovered_states_match_aggregate() {
        let events = fsa_events();
        let discovered = discover_blueprint(&events);

        let proc = discovered
            .procedures
            .iter()
            .find(|p| p.id == "accept_engagement")
            .expect("accept_engagement should be discovered");

        let expected: HashSet<&str> = ["not_started", "in_progress", "under_review", "completed"]
            .iter()
            .copied()
            .collect();

        let found: HashSet<&str> = proc.states.iter().map(|s| s.as_str()).collect();

        assert_eq!(
            found, expected,
            "accept_engagement states should be {:?}, got {:?}",
            expected, found
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: Conformance score > 0.7 when compared against FSA reference
    // -----------------------------------------------------------------------
    #[test]
    fn test_compare_discovered_vs_reference() {
        let events = fsa_events();
        let discovered = discover_blueprint(&events);
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

        let diff = compare_blueprints(&discovered, &bwp.blueprint);

        assert!(
            diff.conformance_score > 0.7,
            "Conformance score should be > 0.7, got {}",
            diff.conformance_score
        );
        assert!(
            !diff.matching_procedures.is_empty(),
            "Should have matching procedures"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: Partial event log → missing_procedures is non-empty
    // -----------------------------------------------------------------------
    #[test]
    fn test_compare_reports_missing_procedures() {
        let all_events = fsa_events();

        // Keep only events from the first 3 unique procedure_ids.
        let mut seen: Vec<String> = Vec::new();
        let mut partial: Vec<AuditEvent> = Vec::new();
        for evt in &all_events {
            if !seen.contains(&evt.procedure_id) {
                if seen.len() >= 3 {
                    break;
                }
                seen.push(evt.procedure_id.clone());
            }
            if seen.contains(&evt.procedure_id) {
                partial.push(evt.clone());
            }
        }

        // Ensure we got exactly 3 procedures.
        let discovered = discover_blueprint(&partial);
        assert_eq!(discovered.procedures.len(), 3);

        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let diff = compare_blueprints(&discovered, &bwp.blueprint);

        assert!(
            !diff.missing_procedures.is_empty(),
            "Should report missing procedures when only 3 / 9 procedures are in the log"
        );
        assert_eq!(
            diff.missing_procedures.len(),
            6,
            "Expected 6 missing procedures (9 - 3), got {}",
            diff.missing_procedures.len()
        );
    }

    // -----------------------------------------------------------------------
    // Test 6: BlueprintDiff serialises to JSON without error
    // -----------------------------------------------------------------------
    #[test]
    fn test_blueprint_diff_serializes() {
        let events = fsa_events();
        let discovered = discover_blueprint(&events);
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

        let diff = compare_blueprints(&discovered, &bwp.blueprint);
        let json = serde_json::to_string(&diff).expect("BlueprintDiff should serialise to JSON");

        assert!(json.contains("conformance_score"));
        assert!(json.contains("matching_procedures"));
    }
}
