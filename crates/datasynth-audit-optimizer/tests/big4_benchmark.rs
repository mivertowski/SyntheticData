//! Integration test: cross-firm Big 4 methodology benchmark.
//!
//! Run with: cargo test -p datasynth-audit-optimizer --test big4_benchmark -- --nocapture --test-threads=1

use datasynth_audit_optimizer::benchmark_comparison::{format_comparison_report, run_comparison};

#[test]
fn test_big4_benchmark_evaluation() {
    let report = run_comparison(42, None);
    let formatted = format_comparison_report(&report);
    println!("\n{}", formatted);

    // All built-in blueprints must be present and produce non-trivial output.
    assert!(
        report.benchmarks.len() >= 4,
        "Expected >= 4 benchmarks, got {}",
        report.benchmarks.len()
    );
    for b in &report.benchmarks {
        assert!(
            b.events > 0,
            "{}: expected events > 0, got {}",
            b.firm,
            b.events
        );
        assert!(
            b.artifacts > 0,
            "{}: expected artifacts > 0, got {}",
            b.firm,
            b.artifacts
        );
        assert!(
            b.completion_rate > 0.5,
            "{}: expected completion_rate > 0.5, got {:.2}",
            b.firm,
            b.completion_rate
        );
    }
}
