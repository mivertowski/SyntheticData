//! Integration tests for the GAM (Global Audit Methodology) blueprint.
//!
//! These tests exercise the full engagement pipeline with the large GAM
//! blueprint (~1,182 procedures, ~2,102 step commands).  They depend on an
//! external file and skip gracefully when it is not present.
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm -- --test-threads=1

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    loader::{default_overlay, BlueprintWithPreconditions},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const GAM_PATH: &str = "/home/michael/DEV/Repos/Methodology/AuditMethodology/data/export/blueprints/gam_blueprint_enriched.yaml";

// ---------------------------------------------------------------------------
// Test 1: Full GAM engagement execution
// ---------------------------------------------------------------------------

#[test]
fn test_gam_full_engagement() {
    let path = std::path::Path::new(GAM_PATH);
    if !path.exists() {
        eprintln!("GAM blueprint not found at {}, skipping", GAM_PATH);
        return;
    }

    let bwp = BlueprintWithPreconditions::load_from_file(path.to_path_buf()).unwrap();
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx).unwrap();

    println!("GAM engagement results:");
    println!("  Events: {}", result.event_log.len());
    println!("  Artifacts: {}", result.artifacts.total_artifacts());
    println!(
        "  Procedures: {}",
        result.procedure_states.len()
    );
    println!(
        "  Phases completed: {}",
        result.phases_completed.len()
    );
    println!("  Duration: {:.1}h", result.total_duration_hours);
    println!("  Anomalies: {}", result.anomalies.len());

    // Basic assertions
    assert!(
        result.event_log.len() >= 1000,
        "GAM should produce >= 1000 events, got {}",
        result.event_log.len()
    );
    assert!(
        result.artifacts.total_artifacts() >= 5000,
        "GAM should produce >= 5000 artifacts, got {}",
        result.artifacts.total_artifacts()
    );
    assert!(
        result.procedure_states.len() >= 1000,
        "GAM should execute >= 1000 procedures, got {}",
        result.procedure_states.len()
    );
}

// ---------------------------------------------------------------------------
// Test 2: GAM command prefix coverage
// ---------------------------------------------------------------------------

#[test]
fn test_gam_command_prefix_coverage() {
    let path = std::path::Path::new(GAM_PATH);
    if !path.exists() {
        eprintln!("GAM blueprint not found at {}, skipping", GAM_PATH);
        return;
    }

    let bwp = BlueprintWithPreconditions::load_from_file(path.to_path_buf()).unwrap();

    let mut total = 0usize;
    let mut prefix_matched = 0usize;
    let prefixes = [
        "provide_",
        "document_",
        "prepare_",
        "evaluate_",
        "assess_",
        "consider_",
        "determine_",
        "calculate_",
        "compute_",
        "perform_",
        "test_",
        "execute_",
        "obtain_",
        "confirm_",
        "request_",
        "send_",
        "identify_",
        "detect_",
        "find_",
        "review_",
        "inspect_",
        "examine_",
        "check_",
        "report_",
        "communicate_",
        "present_",
        "summarize_",
        "conclude_",
        "approve_",
        "sign_",
        "authorize_",
        "also_",
    ];

    for phase in &bwp.blueprint.phases {
        for proc in &phase.procedures {
            for step in &proc.steps {
                if let Some(cmd) = &step.command {
                    total += 1;
                    if prefixes.iter().any(|p| cmd.starts_with(p)) {
                        prefix_matched += 1;
                    }
                }
            }
        }
    }

    let pct = prefix_matched as f64 / total as f64 * 100.0;
    println!(
        "GAM command prefix coverage: {}/{} ({:.1}%)",
        prefix_matched, total, pct
    );
    assert!(
        pct >= 60.0,
        "Expected >= 60% prefix coverage, got {:.1}%",
        pct
    );
}
