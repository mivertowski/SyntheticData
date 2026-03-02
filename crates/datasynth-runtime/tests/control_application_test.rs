//! Integration test verifying that internal controls are applied to journal entries.
//!
//! When `internal_controls.enabled = true`, the orchestrator should populate
//! `control_ids` on journal entry headers for the majority of entries.

use datasynth_config::schema::InternalControlsConfig;
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
use datasynth_test_utils::fixtures::minimal_config;

/// Test that >50% of journal entries have non-empty `control_ids` when
/// internal_controls.enabled = true.
#[test]
fn test_controls_applied_to_journal_entries() {
    let mut config = minimal_config();
    config.global.seed = Some(42);
    config.internal_controls = InternalControlsConfig {
        enabled: true,
        exception_rate: 0.02,
        sod_violation_rate: 0.01,
        sox_materiality_threshold: 10000.0,
        ..Default::default()
    };

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    assert!(
        !result.journal_entries.is_empty(),
        "Should generate at least one journal entry"
    );

    let total = result.journal_entries.len();
    let with_controls = result
        .journal_entries
        .iter()
        .filter(|e| !e.header.control_ids.is_empty())
        .count();

    let ratio = with_controls as f64 / total as f64;
    assert!(
        ratio > 0.50,
        "Expected >50% of JEs to have control_ids when internal_controls.enabled=true, \
         but only {with_controls}/{total} ({:.1}%) had them",
        ratio * 100.0
    );
}

/// Test that controls are NOT applied when internal_controls.enabled = false (the default).
#[test]
fn test_controls_not_applied_when_disabled() {
    let mut config = minimal_config();
    config.global.seed = Some(42);
    // internal_controls.enabled defaults to false

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    assert!(
        !result.journal_entries.is_empty(),
        "Should generate at least one journal entry"
    );

    let with_controls = result
        .journal_entries
        .iter()
        .filter(|e| !e.header.control_ids.is_empty())
        .count();

    assert_eq!(
        with_controls, 0,
        "No JEs should have control_ids when internal_controls.enabled=false, but {with_controls} had them"
    );
}
