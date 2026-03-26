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

    // >= 5 phases
    assert!(
        bwp.blueprint.phases.len() >= 5,
        "KPMG expected >= 5 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // >= 9 procedures (9 + EQR = 10)
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 9,
        "KPMG expected >= 9 procedures, got {}",
        proc_count
    );

    // Has the engagement_quality_review procedure
    let has_eqr = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "engagement_quality_review");
    assert!(
        has_eqr,
        "KPMG must have engagement_quality_review procedure"
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

    // >= 7 phases
    assert!(
        bwp.blueprint.phases.len() >= 7,
        "PwC expected >= 7 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // >= 9 procedures
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 9,
        "PwC expected >= 9 procedures, got {}",
        proc_count
    );

    // Has halo analytics procedure
    let has_halo = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "halo_analytical_procedures");
    assert!(
        has_halo,
        "PwC must have halo_analytical_procedures procedure"
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

    // >= 8 phases
    assert!(
        bwp.blueprint.phases.len() >= 8,
        "Deloitte expected >= 8 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // >= 9 procedures
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 9,
        "Deloitte expected >= 9 procedures, got {}",
        proc_count
    );

    // Has cognitive technology procedure
    let has_cognitive = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| &p.procedures)
        .any(|proc| proc.id == "cognitive_analytical_procedures");
    assert!(
        has_cognitive,
        "Deloitte must have cognitive_analytical_procedures procedure"
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

    // Different phase counts
    let kpmg_phases = kpmg.blueprint.phases.len();
    let pwc_phases = pwc.blueprint.phases.len();
    let deloitte_phases = deloitte.blueprint.phases.len();

    // At least one pair should differ in phase count
    assert!(
        kpmg_phases != pwc_phases || pwc_phases != deloitte_phases,
        "at least two blueprints should have different phase counts: KPMG={}, PwC={}, Deloitte={}",
        kpmg_phases,
        pwc_phases,
        deloitte_phases
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
