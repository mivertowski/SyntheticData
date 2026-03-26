//! Integration tests for the Banking Regulatory Examination blueprint.
//!
//! Validates that the regulatory_exam blueprint loads, validates, and produces
//! a well-formed engagement run with the correct actors, phases, procedures,
//! and evidence catalog.
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm --test regulatory_exam_integration -- --test-threads=1

use datasynth_audit_fsm::{
    loader::{default_overlay, BlueprintWithPreconditions},
    schema::AuditBlueprint,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn count_procedures(bp: &AuditBlueprint) -> usize {
    bp.phases.iter().map(|p| p.procedures.len()).sum()
}

fn all_steps(bp: &AuditBlueprint) -> Vec<&datasynth_audit_fsm::schema::BlueprintStep> {
    bp.phases
        .iter()
        .flat_map(|phase| phase.procedures.iter())
        .flat_map(|proc| proc.steps.iter())
        .collect()
}

// ---------------------------------------------------------------------------
// Blueprint loading and structural validation
// ---------------------------------------------------------------------------

#[test]
fn test_load_regulatory_exam_blueprint() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    // Must validate without errors
    bwp.validate()
        .expect("regulatory exam blueprint must be structurally valid");

    // Framework must be REGULATORY (not ISA or PCAOB)
    assert_eq!(
        bwp.blueprint.methodology.framework, "REGULATORY",
        "framework must be REGULATORY"
    );

    // Must have the expected 6 phases
    assert_eq!(
        bwp.blueprint.phases.len(),
        6,
        "expected 6 phases, got {}",
        bwp.blueprint.phases.len()
    );

    // Must have at least 15 procedures
    let proc_count = count_procedures(&bwp.blueprint);
    assert!(
        proc_count >= 15,
        "expected >= 15 procedures, got {}",
        proc_count
    );
}

#[test]
fn test_regulatory_blueprint_has_expected_actors() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let actor_ids: Vec<&str> = bwp.blueprint.actors.iter().map(|a| a.id.as_str()).collect();

    let expected_actors = [
        "examiner_in_charge",
        "field_examiner",
        "specialist_examiner",
        "supervisory_analyst",
        "district_supervisor",
    ];

    for actor in &expected_actors {
        assert!(
            actor_ids.contains(actor),
            "expected actor '{}' not found in blueprint actors",
            actor
        );
    }

    assert_eq!(
        actor_ids.len(),
        5,
        "expected exactly 5 actors, got {}",
        actor_ids.len()
    );
}

#[test]
fn test_regulatory_blueprint_has_expected_phases() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let phase_ids: Vec<&str> = bwp.blueprint.phases.iter().map(|p| p.id.as_str()).collect();

    let expected_phases = [
        "pre_examination",
        "on_site_examination",
        "bsa_aml_review",
        "capital_adequacy",
        "examination_conclusion",
        "enforcement",
    ];

    for phase in &expected_phases {
        assert!(
            phase_ids.contains(phase),
            "expected phase '{}' not found",
            phase
        );
    }
}

#[test]
fn test_regulatory_blueprint_has_expected_procedures() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let proc_ids: Vec<&str> = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| p.procedures.iter())
        .map(|proc| proc.id.as_str())
        .collect();

    let expected_procedures = [
        "risk_scoping",
        "document_request",
        "management_assessment",
        "asset_quality_review",
        "earnings_analysis",
        "liquidity_assessment",
        "sensitivity_to_market_risk",
        "capital_analysis",
        "bsa_compliance_review",
        "it_examination",
        "consumer_compliance",
        "camels_rating_determination",
        "findings_communication",
        "exit_meeting",
        "enforcement_action",
    ];

    for proc in &expected_procedures {
        assert!(
            proc_ids.contains(proc),
            "expected procedure '{}' not found",
            proc
        );
    }
}

#[test]
fn test_regulatory_blueprint_has_expected_evidence() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let evidence_ids: Vec<&str> = bwp
        .blueprint
        .evidence_templates
        .iter()
        .map(|e| e.id.as_str())
        .collect();

    let expected_evidence = [
        "prior_exam_report",
        "risk_assessment_summary",
        "document_request_list",
        "loan_review_workpapers",
        "earnings_analysis_workpaper",
        "liquidity_assessment",
        "capital_analysis_workpaper",
        "bsa_review_workpaper",
        "it_exam_workpaper",
        "camels_rating_sheet",
        "report_of_examination",
        "enforcement_action_memo",
    ];

    for ev in &expected_evidence {
        assert!(
            evidence_ids.contains(ev),
            "expected evidence '{}' not found in evidence_catalog",
            ev
        );
    }

    assert!(
        evidence_ids.len() >= 12,
        "expected >= 12 evidence items, got {}",
        evidence_ids.len()
    );
}

#[test]
fn test_regulatory_blueprint_steps_have_judgment_level() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let steps = all_steps(&bwp.blueprint);
    assert!(!steps.is_empty(), "blueprint must have steps");

    // Every step must have a judgment_level set
    for step in &steps {
        assert!(
            step.judgment_level.is_some(),
            "step '{}' is missing judgment_level",
            step.id
        );
    }

    // All judgment_level values must be valid
    let valid_levels = ["data_only", "ai_assistable", "human_required"];
    for step in &steps {
        let level = step.judgment_level.as_ref().unwrap();
        assert!(
            valid_levels.contains(&level.as_str()),
            "step '{}' has invalid judgment_level: '{}'",
            step.id,
            level
        );
    }
}

#[test]
fn test_regulatory_blueprint_camels_procedure_in_conclusion_phase() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let conclusion_phase = bwp
        .blueprint
        .phases
        .iter()
        .find(|p| p.id == "examination_conclusion")
        .expect("examination_conclusion phase must exist");

    let has_camels = conclusion_phase
        .procedures
        .iter()
        .any(|p| p.id == "camels_rating_determination");

    assert!(
        has_camels,
        "camels_rating_determination must be in examination_conclusion phase"
    );
}

#[test]
fn test_regulatory_blueprint_bsa_procedure_in_bsa_phase() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let bsa_phase = bwp
        .blueprint
        .phases
        .iter()
        .find(|p| p.id == "bsa_aml_review")
        .expect("bsa_aml_review phase must exist");

    let has_bsa = bsa_phase
        .procedures
        .iter()
        .any(|p| p.id == "bsa_compliance_review");

    assert!(
        has_bsa,
        "bsa_compliance_review must be in bsa_aml_review phase"
    );
}

#[test]
fn test_regulatory_blueprint_enforcement_in_correct_phase() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let enforcement_phase = bwp
        .blueprint
        .phases
        .iter()
        .find(|p| p.id == "enforcement")
        .expect("enforcement phase must exist");

    let has_enforcement = enforcement_phase
        .procedures
        .iter()
        .any(|p| p.id == "enforcement_action");

    assert!(
        has_enforcement,
        "enforcement_action must be in enforcement phase"
    );

    // Enforcement phase must be last (order 6)
    let enforcement_order = bwp
        .blueprint
        .phases
        .iter()
        .position(|p| p.id == "enforcement")
        .unwrap();

    assert_eq!(
        enforcement_order, 5,
        "enforcement phase must be last (index 5), was at index {}",
        enforcement_order
    );
}

#[test]
fn test_regulatory_blueprint_standards_catalog_populated() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    assert!(
        bwp.blueprint.standards.len() >= 8,
        "expected >= 8 standards, got {}",
        bwp.blueprint.standards.len()
    );

    // Verify key regulatory standards are present by checking id and title fields
    let has_occ = bwp.blueprint.standards.iter().any(|s| {
        s.id.contains("OCC") || s.title.contains("OCC") || s.title.contains("Comptroller")
    });

    assert!(has_occ, "standards catalog must reference OCC guidance");
}

#[test]
fn test_regulatory_blueprint_topological_sort_acyclic() {
    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let sorted = bwp
        .topological_sort()
        .expect("precondition DAG must be acyclic");

    let proc_count = count_procedures(&bwp.blueprint);
    assert_eq!(
        sorted.len(),
        proc_count,
        "topological sort must include all {} procedures",
        proc_count
    );

    // risk_scoping has no preconditions — should appear before document_request
    let risk_pos = sorted
        .iter()
        .position(|id| id == "risk_scoping")
        .expect("risk_scoping must appear in sorted order");
    let docreq_pos = sorted
        .iter()
        .position(|id| id == "document_request")
        .expect("document_request must appear in sorted order");

    assert!(
        risk_pos < docreq_pos,
        "risk_scoping (pos {}) must precede document_request (pos {})",
        risk_pos,
        docreq_pos
    );
}

#[test]
fn test_regulatory_blueprint_engine_run() {
    use datasynth_audit_fsm::{context::EngagementContext, engine::AuditFsmEngine};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let bwp = BlueprintWithPreconditions::load_builtin_regulatory()
        .expect("regulatory exam blueprint must load");

    let overlay = default_overlay();
    let context = EngagementContext::demo();
    let rng = ChaCha8Rng::seed_from_u64(42);

    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let result = engine.run_engagement(&context);

    assert!(
        result.is_ok(),
        "engine must complete full engagement without error: {:?}",
        result.err()
    );

    let engagement = result.unwrap();

    // Must have emitted events
    assert!(
        !engagement.event_log.is_empty(),
        "engagement must produce at least one event"
    );

    // Must have produced artifacts (check at least one artifact type is non-empty)
    let has_artifacts = !engagement.artifacts.audit_opinions.is_empty()
        || !engagement.artifacts.combined_risk_assessments.is_empty()
        || !engagement.artifacts.workpapers.is_empty()
        || !engagement.artifacts.findings.is_empty()
        || !engagement.artifacts.evidence.is_empty();

    assert!(
        has_artifacts,
        "engagement must produce at least one artifact"
    );
}
