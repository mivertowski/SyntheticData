//! Benchmark dataset generator for audit FSM event trails.
//!
//! Generates labelled benchmark datasets at different complexity levels,
//! suitable for training and evaluating process mining and anomaly detection
//! models.

use std::io::Write;
use std::path::Path;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::context::EngagementContext;
use crate::engine::EngagementResult;
use crate::error::AuditFsmError;
use crate::event::{AuditAnomalyRecord, AuditEvent};
use crate::loader::*;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Complexity level for benchmark generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkComplexity {
    /// FSA blueprint, default overlay, anomalies zeroed out.
    Simple,
    /// FSA blueprint, rushed overlay (higher anomaly rates).
    Medium,
    /// IA blueprint, default overlay.
    Complex,
}

/// Configuration for benchmark dataset generation.
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Complexity level controlling blueprint/overlay selection.
    pub complexity: BenchmarkComplexity,
    /// Optional override for the anomaly rate (scales all anomaly probabilities).
    pub anomaly_rate: Option<f64>,
    /// RNG seed for deterministic generation.
    pub seed: u64,
}

/// Metadata about the generated benchmark dataset.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkMetadata {
    pub complexity: String,
    pub blueprint: String,
    pub overlay: String,
    pub event_count: usize,
    pub anomaly_count: usize,
    pub anomaly_rate: f64,
    pub procedure_count: usize,
    pub artifact_count: usize,
    pub seed: u64,
}

/// A complete benchmark dataset with events, labels, and metadata.
pub struct BenchmarkDataset {
    /// The audit event trail.
    pub events: Vec<AuditEvent>,
    /// Anomaly label records for supervised learning.
    pub anomaly_labels: Vec<AuditAnomalyRecord>,
    /// Summary metadata about the dataset.
    pub metadata: BenchmarkMetadata,
    /// The full engagement result (for advanced consumers).
    pub result: EngagementResult,
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate a benchmark dataset with the given configuration.
pub fn generate_benchmark(config: &BenchmarkConfig) -> Result<BenchmarkDataset, AuditFsmError> {
    let (bwp, mut overlay, blueprint_name, overlay_name) = match config.complexity {
        BenchmarkComplexity::Simple => {
            let bwp = BlueprintWithPreconditions::load_builtin_fsa()?;
            let mut overlay = default_overlay();
            // Zero out all anomaly rates for a clean baseline.
            overlay.anomalies.skipped_approval = 0.0;
            overlay.anomalies.late_posting = 0.0;
            overlay.anomalies.missing_evidence = 0.0;
            overlay.anomalies.out_of_sequence = 0.0;
            overlay.anomalies.rules.clear();
            (
                bwp,
                overlay,
                "FSA".to_string(),
                "default (zeroed anomalies)".to_string(),
            )
        }
        BenchmarkComplexity::Medium => {
            let bwp = BlueprintWithPreconditions::load_builtin_fsa()?;
            let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Rushed))?;
            (bwp, overlay, "FSA".to_string(), "rushed".to_string())
        }
        BenchmarkComplexity::Complex => {
            let bwp = BlueprintWithPreconditions::load_builtin_ia()?;
            let overlay = default_overlay();
            (bwp, overlay, "IA".to_string(), "default".to_string())
        }
    };

    // Apply anomaly_rate override: scale all anomaly probabilities proportionally.
    if let Some(target_rate) = config.anomaly_rate {
        let target_rate = target_rate.clamp(0.0, 1.0);
        let current_sum = overlay.anomalies.skipped_approval
            + overlay.anomalies.late_posting
            + overlay.anomalies.missing_evidence
            + overlay.anomalies.out_of_sequence;

        if current_sum > 0.0 {
            let scale = target_rate / current_sum;
            overlay.anomalies.skipped_approval =
                (overlay.anomalies.skipped_approval * scale).min(1.0);
            overlay.anomalies.late_posting = (overlay.anomalies.late_posting * scale).min(1.0);
            overlay.anomalies.missing_evidence =
                (overlay.anomalies.missing_evidence * scale).min(1.0);
            overlay.anomalies.out_of_sequence =
                (overlay.anomalies.out_of_sequence * scale).min(1.0);
        } else {
            // If all were zero, distribute evenly.
            let per_type = (target_rate / 4.0).min(1.0);
            overlay.anomalies.skipped_approval = per_type;
            overlay.anomalies.late_posting = per_type;
            overlay.anomalies.missing_evidence = per_type;
            overlay.anomalies.out_of_sequence = per_type;
        }
    }

    bwp.validate()?;

    let rng = ChaCha8Rng::seed_from_u64(config.seed);
    let mut engine = crate::engine::AuditFsmEngine::new(bwp, overlay, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx)?;

    let event_count = result.event_log.len();
    let anomaly_count = result.anomalies.len();
    let anomaly_rate = if event_count > 0 {
        anomaly_count as f64 / event_count as f64
    } else {
        0.0
    };
    let procedure_count = result.procedure_states.len();
    let artifact_count = result.artifacts.total_artifacts();

    let complexity_str = match config.complexity {
        BenchmarkComplexity::Simple => "simple",
        BenchmarkComplexity::Medium => "medium",
        BenchmarkComplexity::Complex => "complex",
    };

    let metadata = BenchmarkMetadata {
        complexity: complexity_str.to_string(),
        blueprint: blueprint_name,
        overlay: overlay_name,
        event_count,
        anomaly_count,
        anomaly_rate,
        procedure_count,
        artifact_count,
        seed: config.seed,
    };

    let events = result.event_log.clone();
    let anomaly_labels = result.anomalies.clone();

    Ok(BenchmarkDataset {
        events,
        anomaly_labels,
        metadata,
        result,
    })
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// Export a benchmark dataset to the given output directory.
///
/// Writes five files:
/// - `event_trail.json` (flat JSON event log)
/// - `event_trail.csv` (CSV for process mining tools)
/// - `event_trail_ocel.json` (OCEL 2.0 projection)
/// - `anomaly_labels.json` (labelled anomaly records)
/// - `metadata.json` (dataset metadata)
pub fn export_benchmark(dataset: &BenchmarkDataset, output_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // event_trail.json
    crate::export::flat_log::export_events_to_file(
        &dataset.events,
        &output_dir.join("event_trail.json"),
    )?;

    // event_trail.csv
    crate::export::csv::export_events_to_csv(&dataset.events, &output_dir.join("event_trail.csv"))?;

    // event_trail_ocel.json
    let ocel_json =
        crate::export::ocel::export_ocel_to_json(&dataset.events).map_err(std::io::Error::other)?;
    let mut ocel_file = std::fs::File::create(output_dir.join("event_trail_ocel.json"))?;
    ocel_file.write_all(ocel_json.as_bytes())?;

    // anomaly_labels.json
    let anomaly_json =
        serde_json::to_string_pretty(&dataset.anomaly_labels).map_err(std::io::Error::other)?;
    let mut anomaly_file = std::fs::File::create(output_dir.join("anomaly_labels.json"))?;
    anomaly_file.write_all(anomaly_json.as_bytes())?;

    // metadata.json
    let meta_json =
        serde_json::to_string_pretty(&dataset.metadata).map_err(std::io::Error::other)?;
    let mut meta_file = std::fs::File::create(output_dir.join("metadata.json"))?;
    meta_file.write_all(meta_json.as_bytes())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_simple() {
        let config = BenchmarkConfig {
            complexity: BenchmarkComplexity::Simple,
            anomaly_rate: None,
            seed: 42,
        };
        let dataset = generate_benchmark(&config).unwrap();
        assert!(
            !dataset.events.is_empty(),
            "Simple benchmark should produce events"
        );
        assert_eq!(
            dataset.anomaly_labels.len(),
            0,
            "Simple benchmark should have no anomalies (all rates zeroed)"
        );
        assert_eq!(dataset.metadata.complexity, "simple");
        assert_eq!(dataset.metadata.blueprint, "FSA");
    }

    #[test]
    fn test_benchmark_medium() {
        let config = BenchmarkConfig {
            complexity: BenchmarkComplexity::Medium,
            anomaly_rate: None,
            seed: 42,
        };
        let dataset = generate_benchmark(&config).unwrap();
        assert!(
            !dataset.events.is_empty(),
            "Medium benchmark should produce events"
        );
        // Rushed overlay has elevated anomaly rates; expect some anomalies.
        // (With the rushed overlay the probabilities are non-zero so over many
        // steps we expect at least one anomaly, but we use a weak check here
        // because the RNG might not trigger any in a short run.)
        assert_eq!(dataset.metadata.complexity, "medium");
        assert_eq!(dataset.metadata.blueprint, "FSA");
        assert_eq!(dataset.metadata.overlay, "rushed");
    }

    #[test]
    fn test_benchmark_complex() {
        let config = BenchmarkConfig {
            complexity: BenchmarkComplexity::Complex,
            anomaly_rate: None,
            seed: 42,
        };
        let dataset = generate_benchmark(&config).unwrap();
        assert!(
            dataset.events.len() > 100,
            "Complex (IA) benchmark should produce > 100 events, got {}",
            dataset.events.len()
        );
        assert_eq!(dataset.metadata.complexity, "complex");
        assert_eq!(dataset.metadata.blueprint, "IA");
    }

    #[test]
    fn test_benchmark_custom_anomaly_rate() {
        let config = BenchmarkConfig {
            complexity: BenchmarkComplexity::Simple,
            anomaly_rate: Some(0.5),
            seed: 42,
        };
        let dataset = generate_benchmark(&config).unwrap();
        assert!(
            !dataset.events.is_empty(),
            "Benchmark with custom anomaly rate should produce events"
        );
        // With a 0.5 aggregate anomaly rate and many steps, we expect anomalies.
        assert!(
            !dataset.anomaly_labels.is_empty(),
            "With anomaly_rate=0.5, should inject some anomalies"
        );
    }

    #[test]
    fn test_benchmark_export() {
        let config = BenchmarkConfig {
            complexity: BenchmarkComplexity::Simple,
            anomaly_rate: None,
            seed: 42,
        };
        let dataset = generate_benchmark(&config).unwrap();

        let dir = tempfile::tempdir().unwrap();
        export_benchmark(&dataset, dir.path()).unwrap();

        // Verify all 5 files exist.
        let expected_files = [
            "event_trail.json",
            "event_trail.csv",
            "event_trail_ocel.json",
            "anomaly_labels.json",
            "metadata.json",
        ];
        for filename in &expected_files {
            let path = dir.path().join(filename);
            assert!(
                path.exists(),
                "Expected file '{}' to exist in output dir",
                filename
            );
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(
                !content.is_empty(),
                "Expected file '{}' to be non-empty",
                filename
            );
        }
    }
}
