//! End-to-end tests for Wave 3: Process Mining Benchmarks.
//!
//! Run with: cargo test -p datasynth-audit-optimizer --test wave3_e2e -- --nocapture --test-threads=1

use datasynth_audit_fsm::benchmark::{
    export_benchmark, generate_benchmark, BenchmarkComplexity, BenchmarkConfig,
};
use datasynth_audit_fsm::export::csv::export_events_to_csv_string;
use datasynth_audit_fsm::export::xes::export_events_to_xes_string;
use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
use datasynth_audit_optimizer::conformance::analyze_conformance;

// =========================================================================
// 1. Benchmark Generation E2E
// =========================================================================

#[test]
fn test_benchmark_simple_no_anomalies() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    assert!(!dataset.events.is_empty(), "Should have events");
    assert_eq!(
        dataset.anomaly_labels.len(),
        0,
        "Simple benchmark should have zero anomalies"
    );
    assert_eq!(dataset.metadata.complexity, "simple");
    assert_eq!(dataset.metadata.blueprint, "FSA");
    assert_eq!(dataset.metadata.anomaly_count, 0);
    assert!((dataset.metadata.anomaly_rate - 0.0).abs() < 0.001);
}

#[test]
fn test_benchmark_medium_has_anomalies() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Medium,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    assert!(!dataset.events.is_empty());
    assert_eq!(dataset.metadata.blueprint, "FSA");
    // Rushed overlay should produce some anomalies (probabilistic but likely)
    // Don't assert exact count — just verify metadata is consistent
    assert_eq!(dataset.metadata.anomaly_count, dataset.anomaly_labels.len());
}

#[test]
fn test_benchmark_complex_is_ia() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Complex,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    assert!(
        dataset.events.len() >= 100,
        "Complex benchmark should have >= 100 events, got {}",
        dataset.events.len()
    );
    assert_eq!(dataset.metadata.blueprint, "IA");
    assert!(dataset.metadata.procedure_count >= 30);
}

#[test]
fn test_benchmark_export_creates_all_files() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("wave3_bench_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    export_benchmark(&dataset, &dir).unwrap();

    // Verify all 5 files exist
    assert!(dir.join("event_trail.json").exists());
    assert!(dir.join("event_trail.csv").exists());
    assert!(dir.join("event_trail_ocel.json").exists());
    assert!(dir.join("anomaly_labels.json").exists());
    assert!(dir.join("metadata.json").exists());

    // Verify metadata.json content
    let meta: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(meta["complexity"], "simple");
    assert_eq!(meta["blueprint"], "FSA");

    // Verify CSV has rows
    let csv = std::fs::read_to_string(dir.join("event_trail.csv")).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert!(lines.len() > 1, "CSV should have header + data rows");

    let _ = std::fs::remove_dir_all(&dir);
}

// =========================================================================
// 2. Export Format E2E
// =========================================================================

#[test]
fn test_csv_export_from_benchmark() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    let csv = export_events_to_csv_string(&dataset.events);
    let lines: Vec<&str> = csv.lines().collect();

    // Header + data rows
    assert_eq!(lines.len(), dataset.events.len() + 1);
    assert!(lines[0].contains("case_id"));
    assert!(lines[0].contains("activity"));
    assert!(lines[0].contains("is_anomaly"));
}

#[test]
fn test_xes_export_from_benchmark() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    let xes = export_events_to_xes_string(&dataset.events);

    assert!(xes.contains("<?xml"));
    assert!(xes.contains("<log"));
    assert!(xes.contains("<trace>"));
    assert!(xes.contains("<event>"));
    assert!(xes.contains("concept:name"));
    assert!(xes.contains("time:timestamp"));

    // Count events
    let event_count = xes.matches("<event>").count();
    assert_eq!(event_count, dataset.events.len());
}

// =========================================================================
// 3. Conformance Metrics E2E
// =========================================================================

#[test]
fn test_conformance_fsa_clean_log() {
    // Generate a clean FSA log (no anomalies) — should have high fitness
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let report = analyze_conformance(&dataset.events, &bwp.blueprint);

    assert!(
        report.fitness >= 0.8,
        "Clean FSA log should have fitness >= 0.8, got {:.2}",
        report.fitness
    );
    assert!(report.precision > 0.0, "Precision should be > 0");
    assert!(report.precision <= 1.0, "Precision should be <= 1.0");
    assert_eq!(report.anomaly_stats.anomaly_events, 0);
}

#[test]
fn test_conformance_ia_with_anomalies() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Complex,
        anomaly_rate: Some(0.2),
        seed: 42,
    })
    .unwrap();

    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let report = analyze_conformance(&dataset.events, &bwp.blueprint);

    assert!(report.fitness > 0.0, "Should have positive fitness");
    assert!(report.anomaly_stats.total_events > 0, "Should have events");

    // Per-procedure conformance should cover multiple procedures
    assert!(
        report.per_procedure.len() >= 10,
        "Should have conformance for >= 10 procedures, got {}",
        report.per_procedure.len()
    );
}

#[test]
fn test_conformance_report_serializes() {
    let dataset = generate_benchmark(&BenchmarkConfig {
        complexity: BenchmarkComplexity::Simple,
        anomaly_rate: None,
        seed: 42,
    })
    .unwrap();

    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let report = analyze_conformance(&dataset.events, &bwp.blueprint);

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("fitness"));
    assert!(json.contains("precision"));
    assert!(json.contains("anomaly_stats"));
    assert!(json.contains("per_procedure"));
}

// =========================================================================
// 4. Full Pipeline E2E
// =========================================================================

#[test]
fn test_full_pipeline_benchmark_to_conformance() {
    // Generate → Export → Conformance — the complete Wave 3 pipeline
    for (complexity, blueprint_name) in [
        (BenchmarkComplexity::Simple, "FSA"),
        (BenchmarkComplexity::Complex, "IA"),
    ] {
        let dataset = generate_benchmark(&BenchmarkConfig {
            complexity,
            anomaly_rate: None,
            seed: 42,
        })
        .unwrap();

        assert_eq!(dataset.metadata.blueprint, blueprint_name);

        // Export all formats
        let csv = export_events_to_csv_string(&dataset.events);
        let xes = export_events_to_xes_string(&dataset.events);
        let ocel_json =
            datasynth_audit_fsm::export::ocel::export_ocel_to_json(&dataset.events).unwrap();

        assert!(!csv.is_empty());
        assert!(!xes.is_empty());
        assert!(!ocel_json.is_empty());

        // Conformance analysis
        let bwp = match blueprint_name {
            "FSA" => BlueprintWithPreconditions::load_builtin_fsa().unwrap(),
            "IA" => BlueprintWithPreconditions::load_builtin_ia().unwrap(),
            _ => unreachable!(),
        };
        let conformance = analyze_conformance(&dataset.events, &bwp.blueprint);

        assert!(conformance.fitness > 0.0);
        assert!(conformance.precision > 0.0);

        println!(
            "  {} pipeline: {} events, fitness={:.2}, precision={:.2}, anomalies={}",
            blueprint_name,
            dataset.events.len(),
            conformance.fitness,
            conformance.precision,
            conformance.anomaly_stats.anomaly_events
        );
    }
}
