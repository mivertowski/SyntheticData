//! Blueprint testing framework.
//!
//! Provides automated validation that a given audit blueprint produces expected
//! artifact types, event counts, phase progression, and timing constraints.
//! The [`test_blueprint`] function runs a single suite; [`test_all_builtins`]
//! exercises every built-in blueprint against reasonable default expectations.

use std::path::PathBuf;

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::error::AuditFsmError;
use datasynth_audit_fsm::loader::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Suite and expectation types
// ---------------------------------------------------------------------------

/// A test suite for validating a single blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintTestSuite {
    /// Blueprint selector (e.g. `"fsa"`, `"ia"`, or a file path).
    pub blueprint: String,
    /// Overlay selector (e.g. `"default"`, `"thorough"`, or a file path).
    pub overlay: String,
    /// Expected metric thresholds.
    pub expectations: BlueprintExpectations,
}

/// Metric thresholds that the blueprint engagement must satisfy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintExpectations {
    /// Minimum total FSM events.
    pub min_events: usize,
    /// Minimum total typed artifacts.
    pub min_artifacts: usize,
    /// Minimum number of procedures that reach a terminal state.
    pub min_procedures: usize,
    /// Phase IDs that must appear in the completed-phases list.
    pub expected_phases: Vec<String>,
    /// Minimum fraction of procedures completed (0.0..=1.0).
    pub min_completion_rate: f64,
    /// Maximum engagement duration in hours.
    pub max_duration_hours: f64,
    /// Artifact category names that must be non-empty (e.g. `"engagements"`, `"workpapers"`).
    pub required_artifact_types: Vec<String>,
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Outcome of running a blueprint test suite.
#[derive(Debug, Clone, Serialize)]
pub struct BlueprintTestResult {
    /// Whether all expectations were met.
    pub passed: bool,
    /// Human-readable descriptions of each failed expectation.
    pub failures: Vec<String>,
    /// Actual metrics observed during the engagement.
    pub metrics: BlueprintMetrics,
}

/// Measured metrics from the engagement run.
#[derive(Debug, Clone, Serialize)]
pub struct BlueprintMetrics {
    /// Total FSM events.
    pub events: usize,
    /// Total typed artifacts.
    pub artifacts: usize,
    /// Number of procedures that reached a terminal state.
    pub procedures: usize,
    /// Phase IDs that were completed.
    pub phases_completed: Vec<String>,
    /// Fraction of procedures completed.
    pub completion_rate: f64,
    /// Engagement duration in hours.
    pub duration_hours: f64,
    /// Artifact category names that contained at least one item.
    pub artifact_types_present: Vec<String>,
}

// ---------------------------------------------------------------------------
// Blueprint / overlay resolution
// ---------------------------------------------------------------------------

fn resolve_blueprint(name: &str) -> Result<BlueprintWithPreconditions, AuditFsmError> {
    match name {
        "fsa" | "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa(),
        "ia" | "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia(),
        "kpmg" | "builtin:kpmg" => BlueprintWithPreconditions::load_builtin_kpmg(),
        "pwc" | "builtin:pwc" => BlueprintWithPreconditions::load_builtin_pwc(),
        "deloitte" | "builtin:deloitte" => BlueprintWithPreconditions::load_builtin_deloitte(),
        "ey_gam_lite" | "builtin:ey_gam_lite" => {
            BlueprintWithPreconditions::load_builtin_ey_gam_lite()
        }
        path => BlueprintWithPreconditions::load_from_file(PathBuf::from(path)),
    }
}

fn resolve_overlay(
    name: &str,
) -> Result<datasynth_audit_fsm::schema::GenerationOverlay, AuditFsmError> {
    match name {
        "default" | "builtin:default" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default))
        }
        "thorough" | "builtin:thorough" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Thorough))
        }
        "rushed" | "builtin:rushed" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Rushed))
        }
        "retail" | "builtin:retail" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::IndustryRetail))
        }
        "manufacturing" | "builtin:manufacturing" => load_overlay(&OverlaySource::Builtin(
            BuiltinOverlay::IndustryManufacturing,
        )),
        "financial_services" | "builtin:financial_services" => load_overlay(
            &OverlaySource::Builtin(BuiltinOverlay::IndustryFinancialServices),
        ),
        path => load_overlay(&OverlaySource::Custom(PathBuf::from(path))),
    }
}

// ---------------------------------------------------------------------------
// Artifact type introspection
// ---------------------------------------------------------------------------

/// Return the names of artifact categories that have at least one item.
fn present_artifact_types(bag: &datasynth_audit_fsm::artifact::ArtifactBag) -> Vec<String> {
    let mut types = Vec::new();
    if !bag.engagements.is_empty() {
        types.push("engagements".into());
    }
    if !bag.engagement_letters.is_empty() {
        types.push("engagement_letters".into());
    }
    if !bag.materiality_calculations.is_empty() {
        types.push("materiality_calculations".into());
    }
    if !bag.risk_assessments.is_empty() {
        types.push("risk_assessments".into());
    }
    if !bag.combined_risk_assessments.is_empty() {
        types.push("combined_risk_assessments".into());
    }
    if !bag.workpapers.is_empty() {
        types.push("workpapers".into());
    }
    if !bag.evidence.is_empty() {
        types.push("evidence".into());
    }
    if !bag.findings.is_empty() {
        types.push("findings".into());
    }
    if !bag.judgments.is_empty() {
        types.push("judgments".into());
    }
    if !bag.sampling_plans.is_empty() {
        types.push("sampling_plans".into());
    }
    if !bag.sampled_items.is_empty() {
        types.push("sampled_items".into());
    }
    if !bag.analytical_results.is_empty() {
        types.push("analytical_results".into());
    }
    if !bag.going_concern_assessments.is_empty() {
        types.push("going_concern_assessments".into());
    }
    if !bag.subsequent_events.is_empty() {
        types.push("subsequent_events".into());
    }
    if !bag.audit_opinions.is_empty() {
        types.push("audit_opinions".into());
    }
    if !bag.key_audit_matters.is_empty() {
        types.push("key_audit_matters".into());
    }
    if !bag.procedure_steps.is_empty() {
        types.push("procedure_steps".into());
    }
    if !bag.samples.is_empty() {
        types.push("samples".into());
    }
    if !bag.confirmations.is_empty() {
        types.push("confirmations".into());
    }
    if !bag.confirmation_responses.is_empty() {
        types.push("confirmation_responses".into());
    }
    types
}

// ---------------------------------------------------------------------------
// Main entry points
// ---------------------------------------------------------------------------

/// Run a single blueprint test suite and return the result.
///
/// Loads the blueprint and overlay, executes an engagement with the given
/// `seed`, then checks each expectation against the observed metrics.
pub fn test_blueprint(suite: &BlueprintTestSuite, seed: u64) -> BlueprintTestResult {
    let run = || -> Result<BlueprintTestResult, AuditFsmError> {
        let bwp = resolve_blueprint(&suite.blueprint)?;
        let overlay = resolve_overlay(&suite.overlay)?;
        let rng = ChaCha8Rng::seed_from_u64(seed);

        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx)?;

        let total_procs = result.procedure_states.len();
        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();
        let completion_rate = if total_procs > 0 {
            completed as f64 / total_procs as f64
        } else {
            0.0
        };

        let artifact_types = present_artifact_types(&result.artifacts);

        let metrics = BlueprintMetrics {
            events: result.event_log.len(),
            artifacts: result.artifacts.total_artifacts(),
            procedures: completed,
            phases_completed: result.phases_completed.clone(),
            completion_rate,
            duration_hours: result.total_duration_hours,
            artifact_types_present: artifact_types.clone(),
        };

        // Check expectations.
        let exp = &suite.expectations;
        let mut failures = Vec::new();

        if metrics.events < exp.min_events {
            failures.push(format!(
                "events: expected >= {}, got {}",
                exp.min_events, metrics.events
            ));
        }
        if metrics.artifacts < exp.min_artifacts {
            failures.push(format!(
                "artifacts: expected >= {}, got {}",
                exp.min_artifacts, metrics.artifacts
            ));
        }
        if metrics.procedures < exp.min_procedures {
            failures.push(format!(
                "procedures completed: expected >= {}, got {}",
                exp.min_procedures, metrics.procedures
            ));
        }
        if metrics.completion_rate < exp.min_completion_rate {
            failures.push(format!(
                "completion_rate: expected >= {:.2}, got {:.2}",
                exp.min_completion_rate, metrics.completion_rate
            ));
        }
        if metrics.duration_hours > exp.max_duration_hours {
            failures.push(format!(
                "duration_hours: expected <= {:.1}, got {:.1}",
                exp.max_duration_hours, metrics.duration_hours
            ));
        }
        for phase in &exp.expected_phases {
            if !metrics.phases_completed.contains(phase) {
                failures.push(format!(
                    "expected phase '{}' to be completed, but it was not",
                    phase
                ));
            }
        }
        for art_type in &exp.required_artifact_types {
            if !artifact_types.contains(art_type) {
                failures.push(format!(
                    "required artifact type '{}' not present (present: {:?})",
                    art_type, artifact_types
                ));
            }
        }

        let passed = failures.is_empty();
        Ok(BlueprintTestResult {
            passed,
            failures,
            metrics,
        })
    };

    match run() {
        Ok(result) => result,
        Err(e) => BlueprintTestResult {
            passed: false,
            failures: vec![format!("engine error: {}", e)],
            metrics: BlueprintMetrics {
                events: 0,
                artifacts: 0,
                procedures: 0,
                phases_completed: vec![],
                completion_rate: 0.0,
                duration_hours: 0.0,
                artifact_types_present: vec![],
            },
        },
    }
}

/// Test all built-in blueprints with reasonable default expectations.
///
/// Returns a vec of `(blueprint_name, BlueprintTestResult)`.  Each blueprint
/// is tested with the default overlay and lenient expectations suitable for
/// regression testing.
pub fn test_all_builtins() -> Vec<(String, BlueprintTestResult)> {
    // (name, min_events, min_artifacts, min_procedures, max_duration_hours)
    let builtins: Vec<(&str, usize, usize, usize, f64)> = vec![
        ("fsa", 10, 5, 3, 50_000.0),
        ("ia", 10, 1, 3, 50_000.0),
        ("kpmg", 10, 5, 3, 50_000.0),
        ("pwc", 10, 5, 3, 50_000.0),
        ("deloitte", 10, 5, 3, 50_000.0),
        ("ey_gam_lite", 10, 5, 3, 50_000.0),
    ];

    builtins
        .into_iter()
        .map(|(name, min_events, min_artifacts, min_procs, max_hours)| {
            let suite = BlueprintTestSuite {
                blueprint: name.to_string(),
                overlay: "default".to_string(),
                expectations: BlueprintExpectations {
                    min_events,
                    min_artifacts,
                    min_procedures: min_procs,
                    expected_phases: vec![], // lenient: don't require specific phases
                    min_completion_rate: 0.3,
                    max_duration_hours: max_hours,
                    required_artifact_types: vec!["engagements".into()],
                },
            };
            let result = test_blueprint(&suite, 42);
            (name.to_string(), result)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passing_suite() {
        let suite = BlueprintTestSuite {
            blueprint: "fsa".into(),
            overlay: "default".into(),
            expectations: BlueprintExpectations {
                min_events: 1,
                min_artifacts: 1,
                min_procedures: 1,
                expected_phases: vec![],
                min_completion_rate: 0.5,
                max_duration_hours: 100_000.0,
                required_artifact_types: vec!["engagements".into()],
            },
        };

        let result = test_blueprint(&suite, 42);
        assert!(
            result.passed,
            "expected suite to pass, failures: {:?}",
            result.failures
        );
        assert!(result.failures.is_empty());
        assert!(result.metrics.events > 0);
        assert!(result.metrics.artifacts > 0);
    }

    #[test]
    fn test_failing_suite_impossible_expectations() {
        let suite = BlueprintTestSuite {
            blueprint: "fsa".into(),
            overlay: "default".into(),
            expectations: BlueprintExpectations {
                min_events: 999_999,
                min_artifacts: 999_999,
                min_procedures: 999,
                expected_phases: vec!["nonexistent_phase".into()],
                min_completion_rate: 1.0,
                max_duration_hours: 0.001,
                required_artifact_types: vec!["nonexistent_artifact_type".into()],
            },
        };

        let result = test_blueprint(&suite, 42);
        assert!(!result.passed, "expected suite to fail");
        assert!(
            !result.failures.is_empty(),
            "expected at least one failure message"
        );
        // Should report multiple distinct failures.
        assert!(
            result.failures.len() >= 3,
            "expected >= 3 failures, got {}: {:?}",
            result.failures.len(),
            result.failures
        );
    }

    #[test]
    fn test_all_builtins_pass() {
        let results = test_all_builtins();

        assert!(
            !results.is_empty(),
            "should have at least one builtin blueprint"
        );

        for (name, result) in &results {
            assert!(
                result.passed,
                "builtin '{}' failed: {:?}",
                name, result.failures
            );
            assert!(
                result.metrics.events > 0,
                "builtin '{}' produced 0 events",
                name
            );
        }
    }
}
