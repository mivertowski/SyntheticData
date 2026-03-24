//! Monte Carlo simulation over the audit FSM engine.
//!
//! Runs N stochastic walks through the FSM engine and analyses outcome
//! distributions to surface bottleneck procedures and revision hotspots.

use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    loader::{default_overlay, BlueprintWithPreconditions},
};

// ---------------------------------------------------------------------------
// Report type
// ---------------------------------------------------------------------------

/// Summary statistics produced by a Monte Carlo simulation run.
#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloReport {
    /// Number of stochastic iterations executed.
    pub iterations: usize,
    /// Average total event count per iteration.
    pub avg_events: f64,
    /// Average total engagement duration in hours per iteration.
    pub avg_duration_hours: f64,
    /// Average number of procedures that reached "completed" per iteration.
    pub avg_procedures_completed: f64,
    /// Top-5 procedures ranked by average event count (procedure_id, avg_count).
    pub bottleneck_procedures: Vec<(String, f64)>,
    /// Top-5 procedures with the most under_review → in_progress revision
    /// transitions (procedure_id, avg_revision_count).
    pub revision_hotspots: Vec<(String, f64)>,
    /// Procedure completion order observed in the first iteration.
    pub happy_path: Vec<String>,
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Run a Monte Carlo simulation over the given blueprint.
///
/// For each iteration a fresh [`AuditFsmEngine`] is constructed with a
/// deterministically-derived seed (`seed.wrapping_add(i as u64)`) so that
/// results are reproducible for any given `seed`.
///
/// # Arguments
/// * `bwp` – Validated blueprint with preconditions.
/// * `iterations` – Number of stochastic walks to run (must be >= 1).
/// * `seed` – Base RNG seed; iteration `i` uses `seed.wrapping_add(i)`.
pub fn run_monte_carlo(
    bwp: &BlueprintWithPreconditions,
    iterations: usize,
    seed: u64,
) -> MonteCarloReport {
    assert!(iterations >= 1, "iterations must be >= 1");

    let ctx = EngagementContext::test_default();

    // Per-procedure accumulators.
    let mut total_events: u64 = 0;
    let mut total_duration: f64 = 0.0;
    let mut total_procedures_completed: u64 = 0;

    // procedure_id → cumulative event count across all iterations.
    let mut proc_event_counts: HashMap<String, u64> = HashMap::new();
    // procedure_id → cumulative revision count (under_review → in_progress).
    let mut proc_revision_counts: HashMap<String, u64> = HashMap::new();

    // Happy path captured from the first successful iteration.
    let mut happy_path: Vec<String> = Vec::new();

    for i in 0..iterations {
        let iter_seed = seed.wrapping_add(i as u64);
        let rng = ChaCha8Rng::seed_from_u64(iter_seed);
        let overlay = default_overlay();

        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay, rng);

        let result = match engine.run_engagement(&ctx) {
            Ok(r) => r,
            // Skip failed iterations rather than panicking.
            Err(_) => continue,
        };

        total_events += result.event_log.len() as u64;
        total_duration += result.total_duration_hours;
        total_procedures_completed += result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed")
            .count() as u64;

        // Accumulate per-procedure event counts.
        for event in &result.event_log {
            *proc_event_counts
                .entry(event.procedure_id.clone())
                .or_default() += 1;

            // Count under_review → in_progress revision transitions.
            if event.from_state.as_deref() == Some("under_review")
                && event.to_state.as_deref() == Some("in_progress")
            {
                *proc_revision_counts
                    .entry(event.procedure_id.clone())
                    .or_default() += 1;
            }
        }

        // Capture the happy path from the first iteration.
        if i == 0 && happy_path.is_empty() {
            happy_path = build_happy_path(&result.event_log);
        }
    }

    let n = iterations as f64;

    // Bottleneck procedures: top-5 by average event count.
    let mut bottleneck_procedures: Vec<(String, f64)> = proc_event_counts
        .iter()
        .map(|(id, &count)| (id.clone(), count as f64 / n))
        .collect();
    bottleneck_procedures.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    bottleneck_procedures.truncate(5);

    // Revision hotspots: top-5 by average revision count.
    let mut revision_hotspots: Vec<(String, f64)> = proc_revision_counts
        .iter()
        .map(|(id, &count)| (id.clone(), count as f64 / n))
        .collect();
    revision_hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    revision_hotspots.truncate(5);

    MonteCarloReport {
        iterations,
        avg_events: total_events as f64 / n,
        avg_duration_hours: total_duration / n,
        avg_procedures_completed: total_procedures_completed as f64 / n,
        bottleneck_procedures,
        revision_hotspots,
        happy_path,
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Extract the procedure completion order from the first iteration's event log.
///
/// Returns the procedure ids in the order that each procedure first emits a
/// transition to the `"completed"` state.
fn build_happy_path(event_log: &[datasynth_audit_fsm::event::AuditEvent]) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut path: Vec<String> = Vec::new();

    for event in event_log {
        if event.to_state.as_deref() == Some("completed")
            && seen.insert(event.procedure_id.clone())
        {
            path.push(event.procedure_id.clone());
        }
    }

    path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fsa() -> BlueprintWithPreconditions {
        BlueprintWithPreconditions::load_builtin_fsa()
            .expect("builtin FSA blueprint should load")
    }

    #[test]
    fn test_monte_carlo_fsa() {
        let bwp = load_fsa();
        let report = run_monte_carlo(&bwp, 10, 42);

        assert!(
            report.avg_events > 0.0,
            "avg_events should be > 0, got {}",
            report.avg_events
        );
        assert!(
            report.avg_duration_hours > 0.0,
            "avg_duration_hours should be > 0, got {}",
            report.avg_duration_hours
        );
        assert!(
            !report.happy_path.is_empty(),
            "happy_path should be non-empty"
        );
    }

    #[test]
    fn test_monte_carlo_deterministic() {
        let bwp = load_fsa();

        let report1 = run_monte_carlo(&bwp, 10, 42);
        let report2 = run_monte_carlo(&bwp, 10, 42);

        assert_eq!(
            report1.avg_events, report2.avg_events,
            "avg_events should be identical across runs with the same seed"
        );
        assert_eq!(
            report1.avg_duration_hours, report2.avg_duration_hours,
            "avg_duration_hours should be identical across runs with the same seed"
        );
    }

    #[test]
    fn test_monte_carlo_report_serializes() {
        let bwp = load_fsa();
        let report = run_monte_carlo(&bwp, 5, 99);

        let json = serde_json::to_string(&report).expect("report should serialize to JSON");

        assert!(
            json.contains("\"iterations\""),
            "JSON should contain 'iterations'"
        );
        assert!(
            json.contains("\"happy_path\""),
            "JSON should contain 'happy_path'"
        );
    }
}
