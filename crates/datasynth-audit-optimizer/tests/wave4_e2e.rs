//! End-to-end tests for Wave 4: Learned & Adaptive Generation.
//!
//! Run with: cargo test -p datasynth-audit-optimizer --test wave4_e2e -- --nocapture --test-threads=1

use datasynth_audit_fsm::content::{
    ContentGenerator, FindingContext, ResponseContext, TemplateContentGenerator, WorkpaperContext,
};
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use datasynth_audit_optimizer::calibration::{calibrate_anomaly_rates, CalibrationTarget};
use datasynth_audit_optimizer::conformance::analyze_conformance;
use datasynth_audit_optimizer::discovery::{compare_blueprints, discover_blueprint};
use datasynth_audit_optimizer::overlay_fitting::{fit_overlay, EngagementProfile};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn ctx() -> EngagementContext {
    EngagementContext::demo()
}

// =========================================================================
// 1. Overlay Fitting → Generate → Conformance Pipeline
// =========================================================================

#[test]
fn test_fit_overlay_then_generate_and_verify() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    // Fit overlay targeting longer duration
    let profile = EngagementProfile {
        target_duration_hours: 1500.0,
        target_event_count: 50,
        target_finding_count: 5,
        target_revision_rate: 0.2,
        target_anomaly_rate: 0.10,
        target_completion_rate: 1.0,
    };

    let fitted = fit_overlay(&bwp, &profile, 10, 3, 42, &EngagementContext::demo());

    // Generate with fitted overlay
    let mut engine = AuditFsmEngine::new(
        bwp.clone(),
        fitted.overlay.clone(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let result = engine.run_engagement(&ctx()).unwrap();

    // Conformance should still be high (valid transitions)
    let conf = analyze_conformance(&result.event_log, &bwp.blueprint);
    assert!(
        conf.fitness >= 0.9,
        "Fitted overlay should still produce valid transitions, fitness={:.2}",
        conf.fitness
    );

    println!(
        "  Fit→Generate→Conformance: fitted in {} iters, residual={:.3}, fitness={:.2}",
        fitted.iterations, fitted.residual, conf.fitness
    );
}

// =========================================================================
// 2. Generate → Discover → Compare Round-Trip
// =========================================================================

#[test]
fn test_generate_discover_compare_fsa() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let mut engine = AuditFsmEngine::new(
        bwp.clone(),
        default_overlay(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let result = engine.run_engagement(&ctx()).unwrap();

    // Discover blueprint from events
    let discovered = discover_blueprint(&result.event_log);

    assert!(
        !discovered.procedures.is_empty(),
        "Should discover procedures"
    );

    // Compare against reference
    let diff = compare_blueprints(&discovered, &bwp.blueprint);

    assert!(
        diff.conformance_score >= 0.6,
        "FSA conformance should be >= 0.6, got {:.2}",
        diff.conformance_score
    );

    println!(
        "  Discover FSA: {} procedures, conformance={:.2}, missing={}, extra={}",
        discovered.procedures.len(),
        diff.conformance_score,
        diff.missing_procedures.len(),
        diff.extra_procedures.len()
    );
}

#[test]
fn test_generate_discover_compare_ia() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(
        bwp.clone(),
        default_overlay(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let result = engine.run_engagement(&ctx()).unwrap();

    let discovered = discover_blueprint(&result.event_log);
    let diff = compare_blueprints(&discovered, &bwp.blueprint);

    assert!(
        discovered.procedures.len() >= 20,
        "Should discover >= 20 IA procedures, got {}",
        discovered.procedures.len()
    );

    println!(
        "  Discover IA: {} procedures, conformance={:.2}, missing={}, extra={}",
        discovered.procedures.len(),
        diff.conformance_score,
        diff.missing_procedures.len(),
        diff.extra_procedures.len()
    );
}

// =========================================================================
// 3. Anomaly Calibration → Benchmark → Conformance
// =========================================================================

#[test]
fn test_calibrate_then_benchmark() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    // Calibrate to 15% anomaly rate
    let calibrated = calibrate_anomaly_rates(
        &bwp,
        &CalibrationTarget {
            target_anomaly_rate: 0.15,
            tolerance: 0.07,
            max_iterations: 10,
        },
        42,
        &EngagementContext::demo(),
    )
    .unwrap();

    // Generate with calibrated overlay
    let mut engine = AuditFsmEngine::new(
        bwp.clone(),
        calibrated.overlay.clone(),
        ChaCha8Rng::seed_from_u64(99),
    );
    let result = engine.run_engagement(&ctx()).unwrap();

    // Check conformance
    let conf = analyze_conformance(&result.event_log, &bwp.blueprint);

    println!(
        "  Calibrated: rate={:.2} (target 0.15), converged={}, fitness={:.2}",
        calibrated.achieved_rate, calibrated.converged, conf.fitness
    );

    // Fitness should still be high even with elevated anomalies
    assert!(conf.fitness >= 0.8, "Fitness should remain high");
}

// =========================================================================
// 4. Content Generator Integration
// =========================================================================

#[test]
fn test_content_generator_produces_contextual_text() {
    let gen = TemplateContentGenerator;

    let finding = gen.generate_finding_narrative(&FindingContext {
        procedure_id: "substantive_testing".into(),
        step_id: "step_3".into(),
        standards_refs: vec!["ISA 500".into(), "ISA 530".into()],
        finding_type: "control_deficiency".into(),
        condition: "missing approval signatures on 12% of sampled invoices".into(),
        criteria: "all invoices above threshold require dual approval".into(),
    });

    assert!(finding.contains("substantive_testing"));
    assert!(finding.contains("ISA 500"));
    assert!(finding.contains("missing approval"));
    assert!(finding.len() > 50, "Narrative should be substantial");

    let workpaper = gen.generate_workpaper_narrative(&WorkpaperContext {
        procedure_id: "risk_identification".into(),
        section: "Risk Assessment".into(),
        actor: "audit_manager".into(),
        standards_refs: vec!["ISA 315".into()],
    });

    assert!(workpaper.contains("Risk Assessment"));
    assert!(workpaper.contains("audit_manager"));

    let response = gen.generate_management_response(&ResponseContext {
        finding_type: "significant_deficiency".into(),
        condition: "inadequate segregation of duties".into(),
        recommendation: "implement compensating controls".into(),
    });

    assert!(response.contains("significant_deficiency"));
    assert!(response.contains("implement compensating controls"));
    assert!(response.contains("90 days"));

    println!(
        "  Content: finding={}b, workpaper={}b, response={}b",
        finding.len(),
        workpaper.len(),
        response.len()
    );
}

// =========================================================================
// 5. Full Pipeline: Fit → Generate → Discover → Compare → Content
// =========================================================================

#[test]
fn test_full_wave4_pipeline() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    // Step 1: Fit overlay
    let profile = EngagementProfile {
        target_duration_hours: 1000.0,
        target_event_count: 50,
        target_finding_count: 3,
        target_revision_rate: 0.10,
        target_anomaly_rate: 0.05,
        target_completion_rate: 1.0,
    };
    let fitted = fit_overlay(&bwp, &profile, 8, 3, 42, &EngagementContext::demo());

    // Step 2: Generate with fitted overlay
    let mut engine = AuditFsmEngine::new(
        bwp.clone(),
        fitted.overlay.clone(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let result = engine.run_engagement(&ctx()).unwrap();

    // Step 3: Discover from generated events
    let discovered = discover_blueprint(&result.event_log);

    // Step 4: Compare against reference
    let diff = compare_blueprints(&discovered, &bwp.blueprint);

    // Step 5: Generate content for findings
    let gen = TemplateContentGenerator;
    let content_count = result.artifacts.findings.len();
    for finding in result.artifacts.findings.iter().take(3) {
        let narrative = gen.generate_finding_narrative(&FindingContext {
            procedure_id: "fsm_generated".into(),
            step_id: "auto".into(),
            standards_refs: vec!["ISA 315".into()],
            finding_type: format!("{:?}", finding.finding_type),
            condition: finding.condition.clone(),
            criteria: finding.criteria.clone(),
        });
        assert!(!narrative.is_empty());
    }

    println!("  Full pipeline:");
    println!(
        "    Fit: {} iters, residual={:.3}",
        fitted.iterations, fitted.residual
    );
    println!(
        "    Generate: {} events, {} artifacts",
        result.event_log.len(),
        result.artifacts.total_artifacts()
    );
    println!(
        "    Discover: {} procedures, conformance={:.2}",
        discovered.procedures.len(),
        diff.conformance_score
    );
    println!("    Content: {} findings narrated", content_count);
}
