//! Integration tests for the full FSA engagement pipeline.
//!
//! These tests exercise the complete round-trip:
//!   blueprint load → validation → engine run → export
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm -- --test-threads=4

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    export::flat_log::{export_events_to_file, export_events_to_json},
    loader::{default_overlay, parse_overlay, BlueprintWithPreconditions},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_engine(seed: u64) -> AuditFsmEngine {
    let bwp =
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint must load");
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(seed);
    AuditFsmEngine::new(bwp, overlay, rng)
}

// ---------------------------------------------------------------------------
// Test 1: Full engagement — structural correctness
// ---------------------------------------------------------------------------

#[test]
fn test_fsa_full_engagement() {
    let bwp =
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint must load");

    // Blueprint must pass validation.
    bwp.validate().expect("FSA blueprint must be valid");

    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("engagement run must succeed");

    // (a) All phases completed (or at least non-empty).
    assert!(
        !result.phases_completed.is_empty(),
        "at least one phase should be completed; got none"
    );

    // (b) All procedures must reach the "completed" state.
    assert!(
        !result.procedure_states.is_empty(),
        "procedure_states must be non-empty"
    );
    for (proc_id, state) in &result.procedure_states {
        assert_eq!(
            state, "completed",
            "procedure '{}' ended in state '{}', expected 'completed'",
            proc_id, state
        );
    }

    // (c) Events must be non-empty and ordered by timestamp.
    assert!(!result.event_log.is_empty(), "event_log must be non-empty");
    for window in result.event_log.windows(2) {
        assert!(
            window[0].timestamp <= window[1].timestamp,
            "events out of order: {} > {}",
            window[0].timestamp,
            window[1].timestamp
        );
    }

    // (d) Every event must reference a procedure that appears in procedure_states.
    for event in &result.event_log {
        assert!(
            result.procedure_states.contains_key(&event.procedure_id),
            "event references unknown procedure '{}'",
            event.procedure_id
        );
    }

    // (e) JSON export round-trips correctly.
    let json = export_events_to_json(&result.event_log).expect("JSON serialisation must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&json).expect("JSON deserialisation must succeed");
    assert_eq!(
        parsed.len(),
        result.event_log.len(),
        "round-tripped JSON array length mismatch"
    );

    // (f) Some evidence must have advanced beyond the initial state (i.e., at
    //     least one evidence_state entry must exist).
    assert!(
        !result.evidence_states.is_empty(),
        "expected at least one evidence state to be recorded"
    );

    // (g) Engagement duration must be positive.
    assert!(
        result.total_duration_hours > 0.0,
        "total_duration_hours must be > 0, got {}",
        result.total_duration_hours
    );
}

// ---------------------------------------------------------------------------
// Test 2: Determinism across runs
// ---------------------------------------------------------------------------

#[test]
fn test_fsa_determinism_across_runs() {
    let ctx = EngagementContext::test_default();

    let mut engine1 = build_engine(99);
    let result1 = engine1
        .run_engagement(&ctx)
        .expect("first run must succeed");

    let mut engine2 = build_engine(99);
    let result2 = engine2
        .run_engagement(&ctx)
        .expect("second run must succeed");

    // Same number of events.
    assert_eq!(
        result1.event_log.len(),
        result2.event_log.len(),
        "event counts differ between deterministic runs"
    );

    // Pairwise equality on the fields that define the event's identity.
    for (i, (e1, e2)) in result1
        .event_log
        .iter()
        .zip(result2.event_log.iter())
        .enumerate()
    {
        assert_eq!(e1.event_id, e2.event_id, "event_id mismatch at index {}", i);
        assert_eq!(
            e1.event_type, e2.event_type,
            "event_type mismatch at index {}",
            i
        );
        assert_eq!(
            e1.timestamp, e2.timestamp,
            "timestamp mismatch at index {}",
            i
        );
        assert_eq!(e1.command, e2.command, "command mismatch at index {}", i);
    }

    // Procedure final states must match.
    assert_eq!(
        result1.procedure_states, result2.procedure_states,
        "procedure_states differ between deterministic runs"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Custom overlay — high revision probability produces revision events
// ---------------------------------------------------------------------------

#[test]
fn test_fsa_with_custom_overlay() {
    // Set revision_probability to 0.9 so that almost every procedure that
    // enters "under_review" will loop back to "in_progress".
    let overlay_yaml = r#"
transitions:
  defaults:
    revision_probability: 0.9
    timing:
      mu_hours: 4.0
      sigma_hours: 1.0
"#;

    let overlay = parse_overlay(overlay_yaml).expect("custom overlay YAML must parse");

    let bwp =
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint must load");
    let rng = ChaCha8Rng::seed_from_u64(7);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("engagement with custom overlay must succeed");

    // With revision_probability = 0.9 across ~9 procedures, it is statistically
    // near-certain that at least one "in_progress" event was a revision loop.
    // We detect a revision by looking for a state_transition event where
    // from_state is "under_review" and to_state is "in_progress".
    let revision_count = result
        .event_log
        .iter()
        .filter(|e| {
            e.from_state.as_deref() == Some("under_review")
                && e.to_state.as_deref() == Some("in_progress")
        })
        .count();

    assert!(
        revision_count > 0,
        "expected at least one revision event (under_review -> in_progress) \
         with revision_probability=0.9, but found none"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Export to temp file and read back
// ---------------------------------------------------------------------------

#[test]
fn test_fsa_export_to_temp_file() {
    let mut engine = build_engine(123);
    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("engagement must succeed");

    let event_count = result.event_log.len();
    assert!(event_count > 0, "event_log must be non-empty before export");

    // Write to a temporary file.
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("fsa_integration_test_{}.json", std::process::id()));

    export_events_to_file(&result.event_log, &tmp_path)
        .expect("export_events_to_file must succeed");

    // Read the file back and verify the JSON array length.
    let contents = std::fs::read_to_string(&tmp_path).expect("written file must be readable");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&contents).expect("file contents must be valid JSON");

    assert_eq!(
        parsed.len(),
        event_count,
        "read-back JSON array length ({}) must match original event count ({})",
        parsed.len(),
        event_count
    );

    // Clean up.
    let _ = std::fs::remove_file(&tmp_path);
}
