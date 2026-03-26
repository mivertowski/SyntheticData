//! Report formatting for optimizer outputs.

use crate::monte_carlo::MonteCarloReport;
use crate::shortest_path::ShortestPathReport;

// ---------------------------------------------------------------------------
// Formatters
// ---------------------------------------------------------------------------

/// Render a [`ShortestPathReport`] as a human-readable multi-line string.
///
/// Procedures are listed in alphabetical order for deterministic output.
pub fn format_shortest_path_report(report: &ShortestPathReport) -> String {
    let mut out = format!(
        "Shortest Path Analysis\n  Total minimum transitions: {}\n\n",
        report.total_minimum_transitions
    );

    let mut sorted: Vec<_> = report.procedure_paths.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (proc_id, path) in sorted {
        out.push_str(&format!(
            "  {} ({} transitions): {}\n",
            proc_id,
            path.transition_count,
            path.states.join(" → ")
        ));
    }

    out
}

/// Render a [`MonteCarloReport`] as a human-readable multi-line string.
pub fn format_monte_carlo_report(report: &MonteCarloReport) -> String {
    let mut out = format!("Monte Carlo ({} iterations)\n", report.iterations);
    out.push_str(&format!(
        "  Avg events: {:.1}\n  Avg duration: {:.1}h\n  Avg procedures: {:.1}\n",
        report.avg_events, report.avg_duration_hours, report.avg_procedures_completed
    ));

    if !report.bottleneck_procedures.is_empty() {
        out.push_str("\n  Bottlenecks:\n");
        for (p, v) in &report.bottleneck_procedures {
            out.push_str(&format!("    {} — {:.1} avg events\n", p, v));
        }
    }

    if !report.revision_hotspots.is_empty() {
        out.push_str("\n  Revision hotspots:\n");
        for (p, v) in &report.revision_hotspots {
            out.push_str(&format!("    {} — {:.1} avg revisions\n", p, v));
        }
    }

    out.push_str(&format!(
        "\n  Happy path: {}\n",
        report.happy_path.join(" → ")
    ));

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::context::EngagementContext;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    use crate::monte_carlo::run_monte_carlo;
    use crate::shortest_path::analyze_shortest_paths;

    fn load_fsa() -> BlueprintWithPreconditions {
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint should load")
    }

    #[test]
    fn test_format_shortest_path_report_non_empty() {
        let bwp = load_fsa();
        let report = analyze_shortest_paths(&bwp.blueprint);
        let text = format_shortest_path_report(&report);

        assert!(!text.is_empty(), "formatted report should not be empty");
        assert!(
            text.contains("Shortest Path Analysis"),
            "output should contain header"
        );
        assert!(
            text.contains("Total minimum transitions:"),
            "output should contain transition summary"
        );
    }

    #[test]
    fn test_format_shortest_path_report_contains_procedures() {
        let bwp = load_fsa();
        let report = analyze_shortest_paths(&bwp.blueprint);
        let text = format_shortest_path_report(&report);

        // At least one procedure id should appear in the output.
        let any_present = report
            .procedure_paths
            .keys()
            .any(|id| text.contains(id.as_str()));
        assert!(
            any_present,
            "formatted report should list at least one procedure id"
        );
    }

    #[test]
    fn test_format_monte_carlo_report_non_empty() {
        let bwp = load_fsa();
        let report = run_monte_carlo(&bwp, 5, 42, &EngagementContext::demo()).unwrap();
        let text = format_monte_carlo_report(&report);

        assert!(!text.is_empty(), "formatted MC report should not be empty");
        assert!(
            text.contains("Monte Carlo"),
            "output should contain 'Monte Carlo'"
        );
        assert!(
            text.contains("Avg events:"),
            "output should contain 'Avg events:'"
        );
        assert!(
            text.contains("Happy path:"),
            "output should contain 'Happy path:'"
        );
    }

    #[test]
    fn test_format_monte_carlo_report_shows_iteration_count() {
        let bwp = load_fsa();
        let report = run_monte_carlo(&bwp, 7, 0, &EngagementContext::demo()).unwrap();
        let text = format_monte_carlo_report(&report);

        assert!(
            text.contains('7'),
            "formatted MC report should include the iteration count"
        );
    }
}
