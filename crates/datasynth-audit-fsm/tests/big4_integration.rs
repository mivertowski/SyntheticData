//! Integration tests for Big 4 firm-style ISA blueprints.
//!
//! Validates KPMG, PwC, and Deloitte blueprints load, validate, and run
//! full engagements producing events and artifacts.
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm --test big4_integration -- --test-threads=1

use datasynth_audit_fsm::{
    context::EngagementContext,
    dispatch::infer_judgment_level,
    engine::AuditFsmEngine,
    loader::{default_overlay, BlueprintWithPreconditions},
    schema::AuditBlueprint,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Count total procedures across all phases.
fn count_procedures(bp: &AuditBlueprint) -> usize {
    bp.phases.iter().map(|p| p.procedures.len()).sum()
}

/// Collect all steps across all phases and procedures.
fn all_steps(bp: &AuditBlueprint) -> Vec<&datasynth_audit_fsm::schema::BlueprintStep> {
    bp.phases
        .iter()
        .flat_map(|phase| phase.procedures.iter())
        .flat_map(|proc| proc.steps.iter())
        .collect()
}

// ---------------------------------------------------------------------------
// Schema: judgment_level deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_blueprint_step_judgment_level_deserializes() {
    let bwp = BlueprintWithPreconditions::load_builtin_kpmg().expect("KPMG blueprint must load");
    let steps = all_steps(&bwp.blueprint);
    // KPMG blueprint has judgment_level set on every step
    let with_judgment: Vec<_> = steps
        .iter()
        .filter(|s| s.judgment_level.is_some())
        .collect();
    assert!(
        !with_judgment.is_empty(),
        "at least one step should have judgment_level set"
    );
    // Check that the values are valid
    for step in &with_judgment {
        let level = step.judgment_level.as_ref().unwrap();
        assert!(
            ["data_only", "ai_assistable", "human_required"].contains(&level.as_str()),
            "step '{}' has invalid judgment_level: '{}'",
            step.id,
            level
        );
    }
}

#[test]
fn test_blueprint_step_ai_capabilities_deserializes() {
    let bwp = BlueprintWithPreconditions::load_builtin_kpmg().expect("KPMG blueprint must load");
    let steps = all_steps(&bwp.blueprint);
    let with_ai_caps: Vec<_> = steps
        .iter()
        .filter(|s| !s.ai_capabilities.is_empty())
        .collect();
    assert!(
        !with_ai_caps.is_empty(),
        "at least one step should have ai_capabilities"
    );
}

// ---------------------------------------------------------------------------
// infer_judgment_level utility
// ---------------------------------------------------------------------------

#[test]
fn test_infer_judgment_level_data_only() {
    assert_eq!(infer_judgment_level("perform_controls_tests"), "data_only");
    assert_eq!(infer_judgment_level("calculate_balance"), "data_only");
    assert_eq!(infer_judgment_level("compute_ratio"), "data_only");
    assert_eq!(infer_judgment_level("verify_completeness"), "data_only");
    assert_eq!(infer_judgment_level("reperform_control"), "data_only");
    assert_eq!(infer_judgment_level("check_accuracy"), "data_only");
    assert_eq!(infer_judgment_level("analyze_trends"), "data_only");
    assert_eq!(
        infer_judgment_level("test_operating_effectiveness"),
        "data_only"
    );
    assert_eq!(infer_judgment_level("execute_procedures"), "data_only");
}

#[test]
fn test_infer_judgment_level_human_required() {
    assert_eq!(infer_judgment_level("evaluate_evidence"), "human_required");
    assert_eq!(infer_judgment_level("assess_risk"), "human_required");
    assert_eq!(infer_judgment_level("consider_fraud"), "human_required");
    assert_eq!(
        infer_judgment_level("determine_materiality"),
        "human_required"
    );
    assert_eq!(
        infer_judgment_level("exercise_skepticism"),
        "human_required"
    );
    assert_eq!(infer_judgment_level("discuss_findings"), "human_required");
    assert_eq!(infer_judgment_level("approve_report"), "human_required");
    assert_eq!(infer_judgment_level("identify_risks"), "human_required");
    assert_eq!(infer_judgment_level("review_workpapers"), "human_required");
    assert_eq!(infer_judgment_level("sign_report"), "human_required");
    assert_eq!(infer_judgment_level("authorize_release"), "human_required");
    assert_eq!(infer_judgment_level("observe_inventory"), "human_required");
    assert_eq!(infer_judgment_level("inquire_management"), "human_required");
}

#[test]
fn test_infer_judgment_level_ai_assistable_fallback() {
    assert_eq!(infer_judgment_level("draft_report"), "ai_assistable");
    assert_eq!(infer_judgment_level("prepare_workpaper"), "ai_assistable");
    assert_eq!(infer_judgment_level("document_findings"), "ai_assistable");
    assert_eq!(infer_judgment_level("unknown_command"), "ai_assistable");
    assert_eq!(infer_judgment_level(""), "ai_assistable");
}

// ---------------------------------------------------------------------------
// Blueprint loading: KPMG
// ---------------------------------------------------------------------------

#[test]
fn test_load_kpmg_blueprint() {
    let bwp = BlueprintWithPreconditions::load_builtin_kpmg().expect("KPMG blueprint must load");

    // Validates without error
    bwp.validate().expect("KPMG blueprint must be valid");

    // 7 phases (ISA-complete blueprint)
    assert!(
        bwp.blueprint.phases.len() >= 7,
        "KPMG expected >= 7 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // 37 procedures (one per ISA standard)
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 37,
        "KPMG expected >= 37 procedures, got {}",
        proc_count
    );

    // Has the ISA 220 Quality Management procedure
    let has_quality = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "proc_isa_220");
    assert!(
        has_quality,
        "KPMG must have proc_isa_220 (ISA 220 Quality Management) procedure"
    );

    // Framework is ISA
    assert_eq!(bwp.blueprint.methodology.framework, "ISA");
}

// ---------------------------------------------------------------------------
// Blueprint loading: PwC
// ---------------------------------------------------------------------------

#[test]
fn test_load_pwc_blueprint() {
    let bwp = BlueprintWithPreconditions::load_builtin_pwc().expect("PwC blueprint must load");

    // Validates without error
    bwp.validate().expect("PwC blueprint must be valid");

    // 7 phases (ISA-complete blueprint with halo_analytics as 7th phase)
    assert!(
        bwp.blueprint.phases.len() >= 7,
        "PwC expected >= 7 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // 37 procedures (one per ISA standard)
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 37,
        "PwC expected >= 37 procedures, got {}",
        proc_count
    );

    // Has ISA 520 Analytical Procedures procedure
    let has_analytical = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "proc_isa_520");
    assert!(
        has_analytical,
        "PwC must have proc_isa_520 (ISA 520 Analytical Procedures) procedure"
    );

    // Framework is ISA
    assert_eq!(bwp.blueprint.methodology.framework, "ISA");
}

// ---------------------------------------------------------------------------
// Blueprint loading: Deloitte
// ---------------------------------------------------------------------------

#[test]
fn test_load_deloitte_blueprint() {
    let bwp =
        BlueprintWithPreconditions::load_builtin_deloitte().expect("Deloitte blueprint must load");

    // Validates without error
    bwp.validate().expect("Deloitte blueprint must be valid");

    // 7 phases (ISA-complete blueprint with cognitive_review as 7th phase)
    assert!(
        bwp.blueprint.phases.len() >= 7,
        "Deloitte expected >= 7 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // 37 procedures (one per ISA standard)
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 37,
        "Deloitte expected >= 37 procedures, got {}",
        proc_count
    );

    // Has ISA 315 Risk Assessment procedure
    let has_risk = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "proc_isa_315");
    assert!(
        has_risk,
        "Deloitte must have proc_isa_315 (ISA 315 Risk Assessment) procedure"
    );

    // Framework is ISA
    assert_eq!(bwp.blueprint.methodology.framework, "ISA");
}

// ---------------------------------------------------------------------------
// Judgment level coverage
// ---------------------------------------------------------------------------

#[test]
fn test_all_blueprints_have_judgment_levels() {
    let blueprints: Vec<(&str, BlueprintWithPreconditions)> = vec![
        (
            "KPMG",
            BlueprintWithPreconditions::load_builtin_kpmg().unwrap(),
        ),
        (
            "PwC",
            BlueprintWithPreconditions::load_builtin_pwc().unwrap(),
        ),
        (
            "Deloitte",
            BlueprintWithPreconditions::load_builtin_deloitte().unwrap(),
        ),
    ];

    for (name, bwp) in &blueprints {
        let steps = all_steps(&bwp.blueprint);
        assert!(!steps.is_empty(), "{} blueprint should have steps", name);
        for step in &steps {
            assert!(
                step.judgment_level.is_some(),
                "{}: step '{}' is missing judgment_level",
                name,
                step.id
            );
            let level = step.judgment_level.as_ref().unwrap();
            assert!(
                ["data_only", "ai_assistable", "human_required"].contains(&level.as_str()),
                "{}: step '{}' has invalid judgment_level '{}'",
                name,
                step.id,
                level
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Run engagements: KPMG
// ---------------------------------------------------------------------------

#[test]
fn test_kpmg_engagement() {
    let bwp = BlueprintWithPreconditions::load_builtin_kpmg().expect("KPMG blueprint must load");
    bwp.validate().expect("KPMG blueprint must be valid");

    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("KPMG engagement must succeed");

    // Events were produced
    assert!(
        !result.event_log.is_empty(),
        "KPMG engagement should produce events"
    );

    // Artifacts were produced
    assert!(
        !result.artifacts.engagements.is_empty()
            || !result.artifacts.workpapers.is_empty()
            || !result.artifacts.evidence.is_empty(),
        "KPMG engagement should produce artifacts"
    );

    // All procedures completed
    for (proc_id, state) in &result.procedure_states {
        assert_eq!(
            state, "completed",
            "KPMG procedure '{}' ended in state '{}', expected 'completed'",
            proc_id, state
        );
    }
}

// ---------------------------------------------------------------------------
// Run engagements: PwC
// ---------------------------------------------------------------------------

#[test]
fn test_pwc_engagement() {
    let bwp = BlueprintWithPreconditions::load_builtin_pwc().expect("PwC blueprint must load");
    bwp.validate().expect("PwC blueprint must be valid");

    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("PwC engagement must succeed");

    // Events were produced
    assert!(
        !result.event_log.is_empty(),
        "PwC engagement should produce events"
    );

    // Artifacts were produced
    assert!(
        !result.artifacts.engagements.is_empty()
            || !result.artifacts.workpapers.is_empty()
            || !result.artifacts.evidence.is_empty(),
        "PwC engagement should produce artifacts"
    );

    // All procedures completed
    for (proc_id, state) in &result.procedure_states {
        assert_eq!(
            state, "completed",
            "PwC procedure '{}' ended in state '{}', expected 'completed'",
            proc_id, state
        );
    }
}

// ---------------------------------------------------------------------------
// Run engagements: Deloitte
// ---------------------------------------------------------------------------

#[test]
fn test_deloitte_engagement() {
    let bwp =
        BlueprintWithPreconditions::load_builtin_deloitte().expect("Deloitte blueprint must load");
    bwp.validate().expect("Deloitte blueprint must be valid");

    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

    let ctx = EngagementContext::test_default();
    let result = engine
        .run_engagement(&ctx)
        .expect("Deloitte engagement must succeed");

    // Events were produced
    assert!(
        !result.event_log.is_empty(),
        "Deloitte engagement should produce events"
    );

    // Artifacts were produced
    assert!(
        !result.artifacts.engagements.is_empty()
            || !result.artifacts.workpapers.is_empty()
            || !result.artifacts.evidence.is_empty(),
        "Deloitte engagement should produce artifacts"
    );

    // All procedures completed
    for (proc_id, state) in &result.procedure_states {
        assert_eq!(
            state, "completed",
            "Deloitte procedure '{}' ended in state '{}', expected 'completed'",
            proc_id, state
        );
    }
}

// ---------------------------------------------------------------------------
// Cross-blueprint comparison
// ---------------------------------------------------------------------------

#[test]
fn test_big4_blueprints_have_distinct_structures() {
    let kpmg = BlueprintWithPreconditions::load_builtin_kpmg().unwrap();
    let pwc = BlueprintWithPreconditions::load_builtin_pwc().unwrap();
    let deloitte = BlueprintWithPreconditions::load_builtin_deloitte().unwrap();

    // All ISA-complete blueprints have 7 phases but the 7th phase differs per firm
    let kpmg_last_phase = kpmg
        .blueprint
        .phases
        .last()
        .map(|p| p.id.as_str())
        .unwrap_or("");
    let pwc_last_phase = pwc
        .blueprint
        .phases
        .last()
        .map(|p| p.id.as_str())
        .unwrap_or("");
    let deloitte_last_phase = deloitte
        .blueprint
        .phases
        .last()
        .map(|p| p.id.as_str())
        .unwrap_or("");

    // Each firm's 7th phase has a distinct firm-specific identifier
    assert_ne!(
        kpmg_last_phase, pwc_last_phase,
        "KPMG and PwC should have distinct 7th-phase IDs (got '{}' vs '{}')",
        kpmg_last_phase, pwc_last_phase
    );
    assert_ne!(
        pwc_last_phase, deloitte_last_phase,
        "PwC and Deloitte should have distinct 7th-phase IDs (got '{}' vs '{}')",
        pwc_last_phase, deloitte_last_phase
    );

    // Different methodology names
    assert_ne!(kpmg.blueprint.name, pwc.blueprint.name);
    assert_ne!(pwc.blueprint.name, deloitte.blueprint.name);
    assert_ne!(kpmg.blueprint.name, deloitte.blueprint.name);

    // All share the same ISA framework
    assert_eq!(kpmg.blueprint.methodology.framework, "ISA");
    assert_eq!(pwc.blueprint.methodology.framework, "ISA");
    assert_eq!(deloitte.blueprint.methodology.framework, "ISA");
}
