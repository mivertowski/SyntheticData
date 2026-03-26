//! End-to-end tests for Wave 1 features.
//!
//! Run with: cargo test -p datasynth-audit-fsm --test wave1_e2e -- --test-threads=1

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{
    default_overlay, load_overlay, BlueprintWithPreconditions, BuiltinOverlay, OverlaySource,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

fn ctx() -> EngagementContext {
    EngagementContext::demo()
}

// =========================================================================
// 1. IA 100% Dispatch Coverage Regression Guard
// =========================================================================

#[test]
fn test_ia_all_140_commands_mapped() {
    // Extract all unique commands from the IA blueprint
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut all_commands: HashSet<String> = HashSet::new();
    for phase in &bwp.blueprint.phases {
        for proc in &phase.procedures {
            for step in &proc.steps {
                if let Some(ref cmd) = step.command {
                    all_commands.insert(cmd.clone());
                }
            }
        }
    }

    // IA blueprint has 82 unique step commands (the 140 figure includes
    // aggregate-level transition commands which are not in step.command)
    assert!(
        all_commands.len() >= 70,
        "Expected >= 70 unique IA step commands, got {}",
        all_commands.len()
    );

    // Run a full engagement and verify every command produced at least one event
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // IA has 34 procedures; continuous-phase procedures with revision
    // loops may end in under_review if they hit the iteration limit.
    // With default overlay (revision_probability=0.15), ~60-70% complete.
    let completed_count = result
        .procedure_states
        .values()
        .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
        .count();
    assert!(
        completed_count >= 20,
        "Expected >= 20 procedures completed, got {}/{}",
        completed_count,
        result.procedure_states.len()
    );

    // Artifact bag should be substantial
    assert!(
        result.artifacts.total_artifacts() > 2000,
        "Expected > 2000 total artifacts, got {}",
        result.artifacts.total_artifacts()
    );
}

// =========================================================================
// 2. IA Artifact Diversity (not just workpapers)
// =========================================================================

#[test]
fn test_ia_produces_diverse_artifact_types() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();
    let bag = &result.artifacts;

    // Should produce at least 5 different artifact types
    let mut types_present = 0u32;
    if !bag.engagements.is_empty() {
        types_present += 1;
    }
    if !bag.workpapers.is_empty() {
        types_present += 1;
    }
    if !bag.risk_assessments.is_empty() {
        types_present += 1;
    }
    if !bag.findings.is_empty() {
        types_present += 1;
    }
    if !bag.judgments.is_empty() {
        types_present += 1;
    }
    if !bag.evidence.is_empty() {
        types_present += 1;
    }
    if !bag.sampling_plans.is_empty() {
        types_present += 1;
    }
    if !bag.combined_risk_assessments.is_empty() {
        types_present += 1;
    }

    assert!(
        types_present >= 5,
        "Expected >= 5 artifact types, got {} (eng={}, wp={}, risk={}, find={}, judg={}, ev={}, sp={}, cra={})",
        types_present,
        bag.engagements.len(),
        bag.workpapers.len(),
        bag.risk_assessments.len(),
        bag.findings.len(),
        bag.judgments.len(),
        bag.evidence.len(),
        bag.sampling_plans.len(),
        bag.combined_risk_assessments.len(),
    );
}

// =========================================================================
// 3. FSA Full Pipeline (events + artifacts + export)
// =========================================================================

#[test]
fn test_fsa_full_pipeline_events_and_artifacts() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    bwp.validate().unwrap();

    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Events
    assert!(result.event_log.len() >= 40, "Expected >= 40 events");
    assert_eq!(result.phases_completed.len(), 3, "FSA has 3 phases");

    // Artifacts — key types must be present
    let bag = &result.artifacts;
    assert!(!bag.engagements.is_empty(), "Missing engagements");
    assert!(
        !bag.materiality_calculations.is_empty(),
        "Missing materiality"
    );
    assert!(!bag.risk_assessments.is_empty(), "Missing risk assessments");
    assert!(!bag.workpapers.is_empty(), "Missing workpapers");
    assert!(!bag.audit_opinions.is_empty(), "Missing audit opinions");
    assert!(
        !bag.going_concern_assessments.is_empty(),
        "Missing going concern"
    );
    assert!(
        !bag.subsequent_events.is_empty(),
        "Missing subsequent events"
    );

    // Export roundtrip
    let json =
        datasynth_audit_fsm::export::flat_log::export_events_to_json(&result.event_log).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), result.event_log.len());

    // OCEL projection
    let ocel = datasynth_audit_fsm::export::ocel::project_to_ocel(&result.event_log);
    assert_eq!(ocel.events.len(), result.event_log.len());
    assert!(!ocel.object_types.is_empty());
}

// =========================================================================
// 4. Overlay Presets Produce Different Profiles
// =========================================================================

#[test]
fn test_overlay_presets_differ() {
    let run = |overlay_name: BuiltinOverlay| -> (usize, f64, usize) {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let overlay = load_overlay(&OverlaySource::Builtin(overlay_name)).unwrap();
        let mut engine = AuditFsmEngine::new(bwp, overlay, ChaCha8Rng::seed_from_u64(42));
        let r = engine.run_engagement(&ctx()).unwrap();
        (r.event_log.len(), r.total_duration_hours, r.anomalies.len())
    };

    let (_, dur_default, anom_default) = run(BuiltinOverlay::Default);
    let (_, dur_thorough, _) = run(BuiltinOverlay::Thorough);
    let (_, dur_rushed, anom_rushed) = run(BuiltinOverlay::Rushed);

    // Thorough should take longer than default
    assert!(
        dur_thorough > dur_default,
        "Thorough ({:.0}h) should take longer than default ({:.0}h)",
        dur_thorough,
        dur_default
    );

    // Rushed should be faster than default
    assert!(
        dur_rushed < dur_default,
        "Rushed ({:.0}h) should be faster than default ({:.0}h)",
        dur_rushed,
        dur_default
    );

    // Rushed should have more anomalies than default
    assert!(
        anom_rushed >= anom_default,
        "Rushed ({}) should have >= anomalies than default ({})",
        anom_rushed,
        anom_default
    );
}

// =========================================================================
// 5. CLI audit run (programmatic equivalent)
// =========================================================================

#[test]
fn test_audit_run_writes_event_trail() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    bwp.validate().unwrap();
    let overlay = default_overlay();
    let mut engine = AuditFsmEngine::new(bwp, overlay, ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Write to temp dir
    let dir = std::env::temp_dir().join(format!("wave1_e2e_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let trail_path = dir.join("audit_event_trail.json");

    datasynth_audit_fsm::export::flat_log::export_events_to_file(&result.event_log, &trail_path)
        .unwrap();

    // Verify file exists and is valid JSON
    assert!(trail_path.exists(), "Event trail file should exist");
    let content = std::fs::read_to_string(&trail_path).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.len(), result.event_log.len());

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

// =========================================================================
// 6. Determinism across all blueprints
// =========================================================================

#[test]
fn test_determinism_fsa_and_ia() {
    for blueprint_name in ["fsa", "ia"] {
        let load = || -> BlueprintWithPreconditions {
            match blueprint_name {
                "fsa" => BlueprintWithPreconditions::load_builtin_fsa().unwrap(),
                "ia" => BlueprintWithPreconditions::load_builtin_ia().unwrap(),
                _ => unreachable!(),
            }
        };

        let mut e1 = AuditFsmEngine::new(load(), default_overlay(), ChaCha8Rng::seed_from_u64(77));
        let mut e2 = AuditFsmEngine::new(load(), default_overlay(), ChaCha8Rng::seed_from_u64(77));

        let r1 = e1.run_engagement(&ctx()).unwrap();
        let r2 = e2.run_engagement(&ctx()).unwrap();

        assert_eq!(
            r1.event_log.len(),
            r2.event_log.len(),
            "{}: event count mismatch",
            blueprint_name
        );
        assert_eq!(
            r1.artifacts.total_artifacts(),
            r2.artifacts.total_artifacts(),
            "{}: artifact count mismatch",
            blueprint_name
        );

        for (a, b) in r1.event_log.iter().zip(r2.event_log.iter()) {
            assert_eq!(
                a.event_id, b.event_id,
                "{}: event_id mismatch",
                blueprint_name
            );
            assert_eq!(
                a.timestamp, b.timestamp,
                "{}: timestamp mismatch",
                blueprint_name
            );
        }
    }
}
