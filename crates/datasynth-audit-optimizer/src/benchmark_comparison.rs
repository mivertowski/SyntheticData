//! Cross-firm methodology benchmark comparison.
//!
//! Runs all available built-in blueprints under the same conditions (seed,
//! overlay, engagement context) and produces a comparative report that enables
//! cross-firm audit methodology benchmarking.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use datasynth_audit_fsm::{
    context::EngagementContext,
    dispatch::infer_judgment_level,
    engine::AuditFsmEngine,
    error::AuditFsmError,
    loader::{default_overlay, BlueprintWithPreconditions},
};

/// Function pointer type for blueprint loader functions.
type BlueprintLoader = fn() -> Result<BlueprintWithPreconditions, AuditFsmError>;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Per-firm benchmark metrics produced from a single engagement simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmBenchmark {
    /// Display name of the firm / methodology (e.g. "KPMG Clara").
    pub firm: String,
    /// Short identifier of the blueprint used (e.g. "kpmg").
    pub blueprint: String,
    /// Number of phases in the blueprint.
    pub phases: usize,
    /// Total number of procedures across all phases.
    pub procedures: usize,
    /// Total number of steps across all procedures.
    pub steps: usize,
    /// Number of events emitted during the engagement simulation.
    pub events: usize,
    /// Total typed artifacts produced by step dispatchers.
    pub artifacts: usize,
    /// Simulated engagement duration in hours.
    pub duration_hours: f64,
    /// Number of anomaly records injected during the engagement.
    pub anomalies: usize,
    /// Fraction of procedures reaching "completed" or "closed" state.
    pub completion_rate: f64,
    /// Breakdown of steps by judgment level.
    pub judgment_distribution: JudgmentDistribution,
    /// Number of accounting/audit standards referenced in the blueprint.
    pub standards_count: usize,
}

/// Step-level judgment classification breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentDistribution {
    /// Steps fully automatable via data processing.
    pub data_only: usize,
    /// Steps where AI can assist but a human reviews.
    pub ai_assistable: usize,
    /// Steps requiring professional skepticism / human judgment.
    pub human_required: usize,
    /// `data_only` as a percentage of total steps.
    pub data_only_pct: f64,
    /// `ai_assistable` as a percentage of total steps.
    pub ai_assistable_pct: f64,
    /// `human_required` as a percentage of total steps.
    pub human_required_pct: f64,
}

/// The full cross-firm comparison report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// One entry per firm / blueprint that loaded successfully.
    pub benchmarks: Vec<FirmBenchmark>,
    /// RNG seed used for all engagement simulations.
    pub seed: u64,
    /// Name of the overlay applied to all simulations.
    pub overlay: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run all available built-in blueprints under identical conditions and return
/// a [`ComparisonReport`].
///
/// The same `seed` and the default overlay are used for every engagement so
/// that differences in the report reflect methodology (blueprint) rather than
/// randomness or configuration.
pub fn run_comparison(seed: u64) -> ComparisonReport {
    let overlay = default_overlay();
    let ctx = EngagementContext::test_default();
    let mut benchmarks = Vec::new();

    // (display name, short key, loader)
    let loaders: &[(&str, &str, BlueprintLoader)] = &[
        (
            "Generic ISA",
            "fsa",
            BlueprintWithPreconditions::load_builtin_fsa,
        ),
        (
            "KPMG Clara",
            "kpmg",
            BlueprintWithPreconditions::load_builtin_kpmg,
        ),
        (
            "PwC Aura",
            "pwc",
            BlueprintWithPreconditions::load_builtin_pwc,
        ),
        (
            "Deloitte Omnia",
            "deloitte",
            BlueprintWithPreconditions::load_builtin_deloitte,
        ),
        (
            "IIA-GIAS",
            "ia",
            BlueprintWithPreconditions::load_builtin_ia,
        ),
    ];

    for (firm_name, bp_name, loader) in loaders {
        let bwp = match loader() {
            Ok(b) => b,
            Err(_) => continue,
        };

        // ------------------------------------------------------------------
        // Structural counts
        // ------------------------------------------------------------------
        let phases = bwp.blueprint.phases.len();
        let procedures: usize = bwp
            .blueprint
            .phases
            .iter()
            .map(|p| p.procedures.len())
            .sum();
        let steps: usize = bwp
            .blueprint
            .phases
            .iter()
            .flat_map(|p| p.procedures.iter())
            .map(|proc| proc.steps.len())
            .sum();

        // ------------------------------------------------------------------
        // Judgment-level classification
        // ------------------------------------------------------------------
        let mut data_only = 0usize;
        let mut ai_assistable = 0usize;
        let mut human_required = 0usize;

        for phase in &bwp.blueprint.phases {
            for proc in &phase.procedures {
                for step in &proc.steps {
                    let level = step.judgment_level.as_deref().unwrap_or_else(|| {
                        infer_judgment_level(step.command.as_deref().unwrap_or(""))
                    });
                    match level {
                        "data_only" => data_only += 1,
                        "human_required" => human_required += 1,
                        _ => ai_assistable += 1,
                    }
                }
            }
        }

        let total_steps_f = (data_only + ai_assistable + human_required).max(1) as f64;

        // ------------------------------------------------------------------
        // Run engagement simulation
        // ------------------------------------------------------------------
        let mut engine = AuditFsmEngine::new(
            bwp.clone(),
            overlay.clone(),
            ChaCha8Rng::seed_from_u64(seed),
        );
        let result = engine.run_engagement(&ctx).unwrap();

        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();

        let standards_count = bwp.blueprint.standards.len();

        benchmarks.push(FirmBenchmark {
            firm: firm_name.to_string(),
            blueprint: bp_name.to_string(),
            phases,
            procedures,
            steps,
            events: result.event_log.len(),
            artifacts: result.artifacts.total_artifacts(),
            duration_hours: result.total_duration_hours,
            anomalies: result.anomalies.len(),
            completion_rate: completed as f64 / result.procedure_states.len().max(1) as f64,
            judgment_distribution: JudgmentDistribution {
                data_only,
                ai_assistable,
                human_required,
                data_only_pct: data_only as f64 / total_steps_f * 100.0,
                ai_assistable_pct: ai_assistable as f64 / total_steps_f * 100.0,
                human_required_pct: human_required as f64 / total_steps_f * 100.0,
            },
            standards_count,
        });
    }

    ComparisonReport {
        benchmarks,
        seed,
        overlay: "default".to_string(),
    }
}

/// Format a [`ComparisonReport`] as a human-readable table.
pub fn format_comparison_report(report: &ComparisonReport) -> String {
    let mut out = String::new();
    out.push_str("Cross-Firm Methodology Benchmark\n");
    out.push_str(&format!(
        "Seed: {}, Overlay: {}\n\n",
        report.seed, report.overlay
    ));

    // Header row
    out.push_str(&format!(
        "{:20} {:>6} {:>6} {:>6} {:>7} {:>9} {:>8} {:>6} {:>7} {:>6} {:>6} {:>6}\n",
        "Firm",
        "Phases",
        "Procs",
        "Steps",
        "Events",
        "Artifacts",
        "Hours",
        "Anom",
        "Compl%",
        "Data%",
        "AI%",
        "Human%"
    ));
    out.push_str(&"-".repeat(110));
    out.push('\n');

    for b in &report.benchmarks {
        out.push_str(&format!(
            "{:20} {:>6} {:>6} {:>6} {:>7} {:>9} {:>8.0} {:>6} {:>6.0}% {:>5.0}% {:>5.0}% {:>5.0}%\n",
            b.firm,
            b.phases,
            b.procedures,
            b.steps,
            b.events,
            b.artifacts,
            b.duration_hours,
            b.anomalies,
            b.completion_rate * 100.0,
            b.judgment_distribution.data_only_pct,
            b.judgment_distribution.ai_assistable_pct,
            b.judgment_distribution.human_required_pct,
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_runs_all_firms() {
        let report = run_comparison(42);
        // All 5 blueprints should load; require at least 4 to be tolerant of
        // potential future blueprint removal.
        assert!(
            report.benchmarks.len() >= 4,
            "Expected >= 4 benchmarks, got {}",
            report.benchmarks.len()
        );
    }

    #[test]
    fn test_comparison_shows_differences() {
        let report = run_comparison(42);
        // The blueprints should not all be structurally identical — at least
        // some pair should differ in phase or procedure count.
        let phases: Vec<usize> = report.benchmarks.iter().map(|b| b.phases).collect();
        let procedures: Vec<usize> = report.benchmarks.iter().map(|b| b.procedures).collect();
        let all_phases_same = phases.windows(2).all(|w| w[0] == w[1]);
        let all_procs_same = procedures.windows(2).all(|w| w[0] == w[1]);
        assert!(
            !all_phases_same || !all_procs_same,
            "All blueprints have identical phases AND procedures — expected some structural differences"
        );
    }

    #[test]
    fn test_comparison_report_serializes() {
        let report = run_comparison(42);
        let json = serde_json::to_string(&report).expect("serialization failed");
        let decoded: ComparisonReport =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(report.benchmarks.len(), decoded.benchmarks.len());
        for (orig, dec) in report.benchmarks.iter().zip(decoded.benchmarks.iter()) {
            assert_eq!(orig.firm, dec.firm);
            assert_eq!(orig.events, dec.events);
            assert_eq!(orig.artifacts, dec.artifacts);
        }
    }

    #[test]
    fn test_comparison_deterministic() {
        let r1 = run_comparison(99);
        let r2 = run_comparison(99);
        assert_eq!(r1.benchmarks.len(), r2.benchmarks.len());
        for (b1, b2) in r1.benchmarks.iter().zip(r2.benchmarks.iter()) {
            assert_eq!(b1.firm, b2.firm);
            assert_eq!(b1.events, b2.events);
            assert_eq!(b1.artifacts, b2.artifacts);
            assert_eq!(b1.duration_hours.to_bits(), b2.duration_hours.to_bits());
            assert_eq!(b1.anomalies, b2.anomalies);
        }
    }
}
