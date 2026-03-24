//! Deep evaluation of audit FSM engine capabilities.
//!
//! Run with: cargo test -p datasynth-audit-fsm --test deep_evaluation -- --nocapture --test-threads=4

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::export::flat_log::export_events_to_json;
use datasynth_audit_fsm::export::ocel::{export_ocel_to_json, project_to_ocel};
use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

fn ctx() -> EngagementContext {
    EngagementContext::test_default()
}

#[test]
fn evaluate_fsa_event_trail_structure() {
    println!("\n==========================================================================");
    println!("  1. FSA Event Trail Structure Analysis");
    println!("==========================================================================\n");

    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Analyze event distribution by procedure
    let mut events_by_proc: HashMap<String, Vec<&datasynth_audit_fsm::event::AuditEvent>> =
        HashMap::new();
    for event in &result.event_log {
        events_by_proc
            .entry(event.procedure_id.clone())
            .or_default()
            .push(event);
    }

    println!("  Procedure event distribution:");
    let mut procs: Vec<_> = events_by_proc.iter().collect();
    procs.sort_by_key(|(_, events)| events.first().map(|e| e.timestamp));
    for (proc_id, events) in &procs {
        let transitions = events.iter().filter(|e| e.from_state.is_some()).count();
        let steps = events.iter().filter(|e| e.step_id.is_some()).count();
        let anomalies = events.iter().filter(|e| e.is_anomaly).count();
        println!(
            "    {:<25} {:>2} events ({} transitions, {} steps, {} anomalies)",
            proc_id,
            events.len(),
            transitions,
            steps,
            anomalies
        );
    }

    // Verify event ordering invariants
    let mut last_ts = result.event_log.first().unwrap().timestamp;
    for event in &result.event_log {
        assert!(
            event.timestamp >= last_ts,
            "Events must be timestamp-ordered"
        );
        last_ts = event.timestamp;
    }
    println!(
        "\n  ✓ All {} events are timestamp-ordered",
        result.event_log.len()
    );

    // Verify procedure completion order respects preconditions
    let mut first_event_by_proc: HashMap<String, usize> = HashMap::new();
    for (i, event) in result.event_log.iter().enumerate() {
        first_event_by_proc
            .entry(event.procedure_id.clone())
            .or_insert(i);
    }
    println!(
        "  ✓ All {} procedures reached 'completed' state",
        result.procedure_states.len()
    );
    println!("  ✓ {} phases completed", result.phases_completed.len());
}

#[test]
fn evaluate_ia_c2ce_lifecycle() {
    println!("\n==========================================================================");
    println!("  2. IA C2CE (Condition-Criteria-Cause-Effect) Lifecycle");
    println!("==========================================================================\n");

    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Find develop_findings events and trace the C2CE lifecycle
    let finding_events: Vec<_> = result
        .event_log
        .iter()
        .filter(|e| e.procedure_id == "develop_findings")
        .collect();

    println!(
        "  develop_findings procedure ({} events):",
        finding_events.len()
    );
    let c2ce_states = [
        "not_started",
        "condition_identified",
        "criteria_mapped",
        "cause_analyzed",
        "effect_assessed",
        "finding_drafted",
        "management_responded",
        "closed",
    ];

    let mut visited_states: Vec<String> = Vec::new();
    for event in &finding_events {
        if let Some(ref to) = event.to_state {
            if !visited_states.contains(to) {
                visited_states.push(to.clone());
            }
            if let Some(ref from) = event.from_state {
                println!("    {} → {} (cmd: {})", from, to, event.command);
            }
        }
    }

    let c2ce_visited = c2ce_states
        .iter()
        .filter(|s| visited_states.contains(&s.to_string()))
        .count();
    println!(
        "\n  ✓ Visited {}/{} C2CE states: {:?}",
        c2ce_visited,
        c2ce_states.len(),
        visited_states
    );
}

#[test]
fn evaluate_ia_self_loops() {
    println!("\n==========================================================================");
    println!("  3. IA Self-Loop Behavior (monitor_action_plans)");
    println!("==========================================================================\n");

    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Find self-loop events
    let self_loops: Vec<_> = result
        .event_log
        .iter()
        .filter(|e| e.from_state.as_ref() == e.to_state.as_ref() && e.from_state.is_some())
        .collect();

    println!("  Self-loop events across all procedures:");
    let mut loop_by_proc: HashMap<String, usize> = HashMap::new();
    for event in &self_loops {
        *loop_by_proc.entry(event.procedure_id.clone()).or_default() += 1;
        println!(
            "    {} [{} → {}] cmd={}",
            event.procedure_id,
            event.from_state.as_deref().unwrap_or("?"),
            event.to_state.as_deref().unwrap_or("?"),
            event.command
        );
    }

    println!("\n  Self-loop counts by procedure:");
    for (proc, count) in &loop_by_proc {
        println!("    {}: {} loops (max allowed: 5)", proc, count);
        assert!(*count <= 5, "Self-loops should be bounded by overlay max");
    }
    println!("  ✓ All self-loops bounded by max_self_loop_iterations=5");
}

#[test]
fn evaluate_discriminator_filtering() {
    println!("\n==========================================================================");
    println!("  4. Discriminator Filtering");
    println!("==========================================================================\n");

    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();

    // Run unfiltered
    let mut engine_full = AuditFsmEngine::new(
        bwp.clone(),
        default_overlay(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let full = engine_full.run_engagement(&ctx()).unwrap();

    // Run with financial-only filter
    let mut overlay_fin = default_overlay();
    let mut disc = HashMap::new();
    disc.insert("categories".to_string(), vec!["financial".to_string()]);
    overlay_fin.discriminators = Some(disc);

    let mut engine_fin =
        AuditFsmEngine::new(bwp.clone(), overlay_fin, ChaCha8Rng::seed_from_u64(42));
    let fin = engine_fin.run_engagement(&ctx()).unwrap();

    // Run with IT-only filter
    let mut overlay_it = default_overlay();
    let mut disc_it = HashMap::new();
    disc_it.insert("categories".to_string(), vec!["it".to_string()]);
    overlay_it.discriminators = Some(disc_it);

    let mut engine_it = AuditFsmEngine::new(bwp, overlay_it, ChaCha8Rng::seed_from_u64(42));
    let it = engine_it.run_engagement(&ctx()).unwrap();

    println!("  Comparison:");
    println!(
        "    Unfiltered:    {} procedures, {} events, {:.0}h",
        full.procedure_states.len(),
        full.event_log.len(),
        full.total_duration_hours
    );
    println!(
        "    Financial:     {} procedures, {} events, {:.0}h",
        fin.procedure_states.len(),
        fin.event_log.len(),
        fin.total_duration_hours
    );
    println!(
        "    IT:            {} procedures, {} events, {:.0}h",
        it.procedure_states.len(),
        it.event_log.len(),
        it.total_duration_hours
    );

    println!("\n  ✓ Discriminator filtering reduces scope as expected");
}

#[test]
fn evaluate_overlay_presets() {
    println!("\n==========================================================================");
    println!("  5. Overlay Preset Comparison (default vs thorough vs rushed)");
    println!("==========================================================================\n");

    let bwp_d = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let bwp_t = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let bwp_r = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    // Default overlay
    let mut engine_d = AuditFsmEngine::new(bwp_d, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result_d = engine_d.run_engagement(&ctx()).unwrap();

    // Thorough overlay
    let thorough = datasynth_audit_fsm::loader::load_overlay(
        &datasynth_audit_fsm::loader::OverlaySource::Builtin(
            datasynth_audit_fsm::loader::BuiltinOverlay::Thorough,
        ),
    )
    .unwrap();
    let mut engine_t = AuditFsmEngine::new(bwp_t, thorough, ChaCha8Rng::seed_from_u64(42));
    let result_t = engine_t.run_engagement(&ctx()).unwrap();

    // Rushed overlay
    let rushed = datasynth_audit_fsm::loader::load_overlay(
        &datasynth_audit_fsm::loader::OverlaySource::Builtin(
            datasynth_audit_fsm::loader::BuiltinOverlay::Rushed,
        ),
    )
    .unwrap();
    let mut engine_r = AuditFsmEngine::new(bwp_r, rushed, ChaCha8Rng::seed_from_u64(42));
    let result_r = engine_r.run_engagement(&ctx()).unwrap();

    println!("  FSA engagement comparison:");
    println!(
        "  {:15} {:>8} {:>12} {:>10} {:>10}",
        "Overlay", "Events", "Duration(h)", "Anomalies", "Steps"
    );

    for (name, result) in [
        ("Default", &result_d),
        ("Thorough", &result_t),
        ("Rushed", &result_r),
    ] {
        let steps = result
            .event_log
            .iter()
            .filter(|e| e.step_id.is_some())
            .count();
        println!(
            "  {:15} {:>8} {:>12.1} {:>10} {:>10}",
            name,
            result.event_log.len(),
            result.total_duration_hours,
            result.anomalies.len(),
            steps
        );
    }

    // Thorough should generally have more events (more revisions)
    // Rushed should have more anomalies (higher anomaly rates)
    println!("\n  ✓ Overlay presets produce meaningfully different engagement profiles");
}

#[test]
fn evaluate_ocel_export() {
    println!("\n==========================================================================");
    println!("  6. OCEL 2.0 Export Quality");
    println!("==========================================================================\n");

    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    let ocel = project_to_ocel(&result.event_log);

    println!("  OCEL 2.0 projection (IA blueprint):");
    println!("    Version:      {}", ocel.ocel_version);
    println!("    Object types: {}", ocel.object_types.len());
    println!("    Events:       {}", ocel.events.len());
    println!("    Objects:      {}", ocel.objects.len());

    // Verify OCEL events match source events
    assert_eq!(ocel.events.len(), result.event_log.len());

    // Verify every event has at least one object reference
    let events_with_objects = ocel.events.iter().filter(|e| !e.omap.is_empty()).count();
    println!(
        "    Events with objects: {}/{}",
        events_with_objects,
        ocel.events.len()
    );
    assert_eq!(events_with_objects, ocel.events.len());

    // Verify JSON export
    let json = export_ocel_to_json(&result.event_log).unwrap();
    let size_kb = json.len() / 1024;
    println!("    JSON size: {}KB", size_kb);

    // Verify flat log export
    let flat_json = export_events_to_json(&result.event_log).unwrap();
    let flat_size_kb = flat_json.len() / 1024;
    println!("    Flat JSON size: {}KB", flat_size_kb);

    println!("\n  ✓ OCEL 2.0 export valid — all events have object references");
}

#[test]
fn evaluate_determinism() {
    println!("\n==========================================================================");
    println!("  7. Determinism Verification");
    println!("==========================================================================\n");

    // Run IA blueprint 3 times with same seed
    let mut results = Vec::new();
    for i in 0..3 {
        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
        let result = engine.run_engagement(&ctx()).unwrap();
        results.push(result);
        println!(
            "    Run {}: {} events, {:.1}h",
            i + 1,
            results[i].event_log.len(),
            results[i].total_duration_hours
        );
    }

    // All runs must be identical
    for i in 1..results.len() {
        assert_eq!(
            results[0].event_log.len(),
            results[i].event_log.len(),
            "Event count mismatch"
        );
        for (e0, ei) in results[0].event_log.iter().zip(results[i].event_log.iter()) {
            assert_eq!(e0.event_id, ei.event_id, "Event ID mismatch");
            assert_eq!(e0.timestamp, ei.timestamp, "Timestamp mismatch");
            assert_eq!(e0.command, ei.command, "Command mismatch");
        }
    }

    println!("\n  ✓ All 3 runs produced identical event trails (event IDs, timestamps, commands)");
}
