//! Integration tests for the full IA (Internal Audit) engagement pipeline.
//!
//! These tests exercise the complete round-trip using the generic_ia blueprint:
//!   blueprint load → validation → engine run → export
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm -- --test-threads=4

use std::collections::HashMap;

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    export::flat_log::export_events_to_json,
    loader::{default_overlay, BlueprintWithPreconditions},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_ia_engine(seed: u64) -> AuditFsmEngine {
    let bwp =
        BlueprintWithPreconditions::load_builtin_ia().expect("builtin IA blueprint must load");
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(seed);
    AuditFsmEngine::new(bwp, overlay, rng)
}

// ---------------------------------------------------------------------------
// Test 1: Full engagement — structural correctness
// ---------------------------------------------------------------------------

#[test]
fn test_ia_full_engagement() {
    let bwp =
        BlueprintWithPreconditions::load_builtin_ia().expect("builtin IA blueprint must load");

    // Blueprint must pass validation.
    bwp.validate().expect("IA blueprint must be valid");

    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("IA engagement run must succeed");

    // (a) At least 50 events: the IA blueprint has 34 procedures, each walking
    //     several FSM transitions plus step events.
    assert!(
        result.event_log.len() >= 50,
        "expected >= 50 events for IA blueprint, got {}",
        result.event_log.len()
    );

    // (b) phases_completed must be non-empty.
    assert!(
        !result.phases_completed.is_empty(),
        "at least one phase should be completed; got none"
    );

    // (c) Events must be ordered by timestamp.
    for window in result.event_log.windows(2) {
        assert!(
            window[0].timestamp <= window[1].timestamp,
            "events out of order: {} > {}",
            window[0].timestamp,
            window[1].timestamp
        );
    }

    // (d) JSON export round-trips correctly.
    let json = export_events_to_json(&result.event_log).expect("JSON serialisation must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&json).expect("JSON deserialisation must succeed");
    assert_eq!(
        parsed.len(),
        result.event_log.len(),
        "round-tripped JSON array length mismatch"
    );

    // (e) Engagement duration must be positive.
    assert!(
        result.total_duration_hours > 0.0,
        "total_duration_hours must be > 0, got {}",
        result.total_duration_hours
    );
}

// ---------------------------------------------------------------------------
// Test 2: Determinism — same seed produces identical event sequence
// ---------------------------------------------------------------------------

#[test]
fn test_ia_determinism() {
    let ctx = EngagementContext::test_default();

    let mut engine1 = build_ia_engine(99);
    let result1 = engine1
        .run_engagement(&ctx)
        .expect("first IA run must succeed");

    let mut engine2 = build_ia_engine(99);
    let result2 = engine2
        .run_engagement(&ctx)
        .expect("second IA run must succeed");

    // Same number of events.
    assert_eq!(
        result1.event_log.len(),
        result2.event_log.len(),
        "event counts differ between deterministic IA runs"
    );

    // Pairwise identity on key fields.
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

    // Procedure final states must also match.
    assert_eq!(
        result1.procedure_states, result2.procedure_states,
        "procedure_states differ between deterministic IA runs"
    );
}

// ---------------------------------------------------------------------------
// Test 3: C2CE lifecycle — develop_findings produces events through all states
// ---------------------------------------------------------------------------

#[test]
fn test_ia_c2ce_lifecycle_events() {
    // The `develop_findings` procedure uses the C2CE model:
    //   not_started → condition_identified → criteria_mapped → cause_analyzed
    //   → effect_assessed → finding_drafted → management_responded → closed
    // That is 7 transitions, so we expect >= 5 events for this procedure
    // (allowing for the engine's MAX_ITERATIONS guard and any skips).

    let mut engine = build_ia_engine(42);
    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("IA engagement must succeed");

    let develop_findings_events: Vec<_> = result
        .event_log
        .iter()
        .filter(|e| e.procedure_id == "develop_findings")
        .collect();

    assert!(
        develop_findings_events.len() >= 5,
        "expected >= 5 events for develop_findings (C2CE lifecycle), got {}",
        develop_findings_events.len()
    );

    // Verify we see at least some of the C2CE state transitions in the log.
    let c2ce_states = [
        "condition_identified",
        "criteria_mapped",
        "cause_analyzed",
        "effect_assessed",
        "finding_drafted",
        "management_responded",
        "closed",
    ];

    // Collect all to_state values seen for this procedure.
    let observed_states: Vec<&str> = develop_findings_events
        .iter()
        .filter_map(|e| e.to_state.as_deref())
        .collect();

    // At least 4 of the 7 C2CE target states should appear (conservative bound
    // to tolerate MAX_ITERATIONS stopping early if anomalies redirect flow).
    let matched = c2ce_states
        .iter()
        .filter(|&&s| observed_states.contains(&s))
        .count();

    assert!(
        matched >= 4,
        "expected >= 4 C2CE states to appear in develop_findings events; \
         matched {matched} of 7. Observed to_states: {:?}",
        observed_states
    );
}

// ---------------------------------------------------------------------------
// Test 4: Discriminator filtering — financial subset executes fewer procedures
// ---------------------------------------------------------------------------

#[test]
fn test_ia_with_financial_discriminator() {
    let ctx = EngagementContext::test_default();

    // Build a filtered engine: only procedures whose "categories" discriminator
    // includes "financial" will be allowed to execute.
    let bwp_filtered =
        BlueprintWithPreconditions::load_builtin_ia().expect("IA blueprint must load");
    let mut overlay_filtered = default_overlay();
    let mut disc: HashMap<String, Vec<String>> = HashMap::new();
    disc.insert("categories".to_string(), vec!["financial".to_string()]);
    overlay_filtered.discriminators = Some(disc);

    let rng_filtered = ChaCha8Rng::seed_from_u64(42);
    let mut engine_filtered = AuditFsmEngine::new(bwp_filtered, overlay_filtered, rng_filtered);
    let filtered = engine_filtered
        .run_engagement(&ctx)
        .expect("filtered IA engagement must succeed");

    // Build an unfiltered engine for comparison.
    let bwp_full = BlueprintWithPreconditions::load_builtin_ia().expect("IA blueprint must load");
    let rng_full = ChaCha8Rng::seed_from_u64(42);
    let mut engine_full = AuditFsmEngine::new(bwp_full, default_overlay(), rng_full);
    let full = engine_full
        .run_engagement(&ctx)
        .expect("unfiltered IA engagement must succeed");

    // The filtered run must execute no more procedures than the unfiltered run.
    assert!(
        filtered.procedure_states.len() <= full.procedure_states.len(),
        "filtered run ({} procedures) should execute <= unfiltered run ({} procedures)",
        filtered.procedure_states.len(),
        full.procedure_states.len()
    );

    // The filtered run must also produce fewer events (since fewer procedures
    // with FSMs execute — procedures with no discriminators still run, but those
    // with non-matching categories are skipped).
    assert!(
        filtered.event_log.len() <= full.event_log.len(),
        "filtered run ({} events) should produce <= events than unfiltered run ({} events)",
        filtered.event_log.len(),
        full.event_log.len()
    );
}
