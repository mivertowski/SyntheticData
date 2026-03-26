//! Conformance metrics for audit event trails against blueprints.
//!
//! Computes fitness (fraction of observed transitions that are valid per the
//! blueprint), precision (fraction of defined transitions that were observed),
//! and anomaly statistics.

use std::collections::{HashMap, HashSet};

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::event::AuditEvent;
use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
use datasynth_audit_fsm::schema::{AuditBlueprint, GenerationOverlay};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Full conformance report for an event trail against a blueprint.
#[derive(Debug, Clone, Serialize)]
pub struct ConformanceReport {
    /// Fraction of observed transition events that match a defined transition.
    pub fitness: f64,
    /// Fraction of defined transitions that were observed in the event trail.
    pub precision: f64,
    /// Generalization score in `[0, 1]`. High values indicate the blueprint
    /// produces consistent fitness across different seeds (low variance).
    /// `None` if not computed (requires `compute_generalization`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generalization: Option<f64>,
    /// Anomaly statistics.
    pub anomaly_stats: AnomalyStats,
    /// Per-procedure conformance breakdown.
    pub per_procedure: Vec<ProcedureConformance>,
}

/// Metrics for evaluating an external anomaly detector against ground-truth labels.
#[derive(Debug, Clone, Serialize)]
pub struct AnomalyDetectionMetrics {
    /// Events correctly identified as anomalies.
    pub true_positives: usize,
    /// Events incorrectly identified as anomalies.
    pub false_positives: usize,
    /// Anomaly events missed by the detector.
    pub false_negatives: usize,
    /// True negatives: correctly identified normal events.
    pub true_negatives: usize,
    /// Precision = TP / (TP + FP).
    pub precision: f64,
    /// Recall = TP / (TP + FN).
    pub recall: f64,
    /// F1 = 2 * precision * recall / (precision + recall).
    pub f1: f64,
}

/// Summary statistics about anomalies in the event trail.
#[derive(Debug, Clone, Serialize)]
pub struct AnomalyStats {
    /// Total events in the trail.
    pub total_events: usize,
    /// Number of events flagged as anomalies.
    pub anomaly_events: usize,
    /// Anomaly rate (anomaly_events / total_events).
    pub anomaly_rate: f64,
    /// Anomaly counts by type.
    pub by_type: HashMap<String, usize>,
}

/// Conformance metrics for a single procedure.
#[derive(Debug, Clone, Serialize)]
pub struct ProcedureConformance {
    /// Procedure identifier.
    pub procedure_id: String,
    /// Fraction of this procedure's observed transitions that are valid.
    pub fitness: f64,
    /// Number of transition events observed for this procedure.
    pub transitions_observed: usize,
    /// Number of transitions defined for this procedure in the blueprint.
    pub transitions_defined: usize,
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

/// Analyze conformance of an event trail against a blueprint.
///
/// - **Fitness**: For each event that has both `from_state` and `to_state`,
///   checks whether `(from_state, to_state)` exists in the corresponding
///   procedure's aggregate transitions. `fitness = valid / total`.
///
/// - **Precision**: Counts the unique `(procedure_id, from_state, to_state)`
///   triples observed in the event trail, divided by the total number of
///   transitions defined across all procedures in the blueprint.
///
/// - **Anomaly stats**: Counts events with `is_anomaly == true`, grouped by
///   `anomaly_type`.
///
/// - **Per-procedure**: Computes fitness for each procedure independently.
pub fn analyze_conformance(events: &[AuditEvent], blueprint: &AuditBlueprint) -> ConformanceReport {
    // Build a lookup: procedure_id -> set of (from_state, to_state).
    let mut defined_transitions: HashMap<String, HashSet<(String, String)>> = HashMap::new();
    let mut total_defined = 0usize;

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            let pairs: HashSet<(String, String)> = proc
                .aggregate
                .transitions
                .iter()
                .map(|t| (t.from_state.clone(), t.to_state.clone()))
                .collect();
            total_defined += pairs.len();
            defined_transitions.insert(proc.id.clone(), pairs);
        }
    }

    // Traverse events, computing fitness and precision.
    let mut global_valid = 0usize;
    let mut global_total = 0usize;
    let mut observed_triples: HashSet<(String, String, String)> = HashSet::new();

    // Per-procedure accumulators: (valid, total).
    let mut proc_accum: HashMap<String, (usize, usize)> = HashMap::new();

    // Anomaly tracking.
    let mut anomaly_events = 0usize;
    let mut anomaly_by_type: HashMap<String, usize> = HashMap::new();

    for event in events {
        // Anomaly stats.
        if event.is_anomaly {
            anomaly_events += 1;
            let type_str = event
                .anomaly_type
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            *anomaly_by_type.entry(type_str).or_default() += 1;
        }

        // Fitness: only consider events with both from_state and to_state.
        if let (Some(ref from), Some(ref to)) = (&event.from_state, &event.to_state) {
            global_total += 1;
            let entry = proc_accum.entry(event.procedure_id.clone()).or_default();
            entry.1 += 1;

            let is_valid = defined_transitions
                .get(&event.procedure_id)
                .map(|pairs| pairs.contains(&(from.clone(), to.clone())))
                .unwrap_or(false);

            if is_valid {
                global_valid += 1;
                entry.0 += 1;
            }

            // Track observed triple for precision.
            observed_triples.insert((event.procedure_id.clone(), from.clone(), to.clone()));
        }
    }

    let fitness = if global_total > 0 {
        global_valid as f64 / global_total as f64
    } else {
        1.0
    };

    let precision = if total_defined > 0 {
        observed_triples.len() as f64 / total_defined as f64
    } else {
        0.0
    };

    let anomaly_rate = if events.is_empty() {
        0.0
    } else {
        anomaly_events as f64 / events.len() as f64
    };

    let anomaly_stats = AnomalyStats {
        total_events: events.len(),
        anomaly_events,
        anomaly_rate,
        by_type: anomaly_by_type,
    };

    // Build per-procedure conformance.
    let mut per_procedure: Vec<ProcedureConformance> = Vec::new();
    // Include all procedures from the blueprint, even if they had no events.
    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            let (valid, total) = proc_accum.get(&proc.id).copied().unwrap_or((0, 0));
            let proc_fitness = if total > 0 {
                valid as f64 / total as f64
            } else {
                1.0
            };
            let transitions_defined = defined_transitions
                .get(&proc.id)
                .map(|s| s.len())
                .unwrap_or(0);
            per_procedure.push(ProcedureConformance {
                procedure_id: proc.id.clone(),
                fitness: proc_fitness,
                transitions_observed: total,
                transitions_defined,
            });
        }
    }

    ConformanceReport {
        fitness,
        precision,
        generalization: None,
        anomaly_stats,
        per_procedure,
    }
}

// ---------------------------------------------------------------------------
// Generalization
// ---------------------------------------------------------------------------

/// Compute generalization: run the blueprint with 3 different seeds, measure
/// fitness variance. Low variance = high generalization (score near 1.0).
///
/// Generalization = 1.0 - std_dev(fitness values across seeds).
/// The result is clamped to [0, 1].
pub fn compute_generalization(
    bwp: &BlueprintWithPreconditions,
    overlay: &GenerationOverlay,
    blueprint: &AuditBlueprint,
    base_seed: u64,
    context: &EngagementContext,
) -> f64 {
    let seeds = [
        base_seed,
        base_seed.wrapping_add(1000),
        base_seed.wrapping_add(2000),
    ];
    let mut fitness_values = Vec::new();

    for seed in &seeds {
        let rng = ChaCha8Rng::seed_from_u64(*seed);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);
        if let Ok(result) = engine.run_engagement(context) {
            let report = analyze_conformance(&result.event_log, blueprint);
            fitness_values.push(report.fitness);
        }
    }

    if fitness_values.len() < 2 {
        return 1.0; // Not enough data; assume perfect generalization.
    }

    let n = fitness_values.len() as f64;
    let mean = fitness_values.iter().sum::<f64>() / n;
    let variance = fitness_values
        .iter()
        .map(|f| (f - mean).powi(2))
        .sum::<f64>()
        / n;
    let std_dev = variance.sqrt();

    (1.0 - std_dev).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Anomaly Detection Evaluation
// ---------------------------------------------------------------------------

/// Evaluate an external anomaly detector's predictions against ground-truth
/// labels from the audit event trail.
///
/// `events` — the audit event trail with `is_anomaly` ground-truth labels.
/// `predictions` — one boolean per event: `true` = "detector thinks anomaly".
///
/// # Errors
///
/// Returns an error if `events.len() != predictions.len()`.
pub fn evaluate_detector(
    events: &[AuditEvent],
    predictions: &[bool],
) -> Result<AnomalyDetectionMetrics, String> {
    if events.len() != predictions.len() {
        return Err(format!(
            "events and predictions must have the same length ({} vs {})",
            events.len(),
            predictions.len()
        ));
    }

    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    let mut tn = 0usize;

    for (event, &predicted) in events.iter().zip(predictions.iter()) {
        match (event.is_anomaly, predicted) {
            (true, true) => tp += 1,
            (false, true) => fp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
        }
    }

    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    Ok(AnomalyDetectionMetrics {
        true_positives: tp,
        false_positives: fp,
        false_negatives: fn_,
        true_negatives: tn,
        precision,
        recall,
        f1,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::context::EngagementContext;
    use datasynth_audit_fsm::engine::AuditFsmEngine;
    use datasynth_audit_fsm::loader::{
        default_overlay, load_overlay, BlueprintWithPreconditions, BuiltinOverlay, OverlaySource,
    };
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn run_fsa_engagement(
        overlay_type: BuiltinOverlay,
        seed: u64,
    ) -> (Vec<AuditEvent>, AuditBlueprint) {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let overlay = load_overlay(&OverlaySource::Builtin(overlay_type)).unwrap();
        let bp = bwp.blueprint.clone();
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();
        (result.event_log, bp)
    }

    #[test]
    fn test_conformance_perfect_log() {
        // FSA with zeroed anomalies: all transitions should be valid.
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let bp = bwp.blueprint.clone();
        let mut overlay = default_overlay();
        overlay.anomalies.skipped_approval = 0.0;
        overlay.anomalies.late_posting = 0.0;
        overlay.anomalies.missing_evidence = 0.0;
        overlay.anomalies.out_of_sequence = 0.0;
        overlay.anomalies.rules.clear();
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        let report = analyze_conformance(&result.event_log, &bp);
        assert!(
            (report.fitness - 1.0).abs() < f64::EPSILON,
            "Fitness should be 1.0 for a perfect log, got {}",
            report.fitness
        );
        assert_eq!(report.anomaly_stats.anomaly_events, 0);
    }

    #[test]
    fn test_conformance_with_anomalies() {
        // Rushed overlay has elevated anomaly rates.
        let (events, bp) = run_fsa_engagement(BuiltinOverlay::Rushed, 42);
        let report = analyze_conformance(&events, &bp);

        // Fitness should still be high (anomalies don't create invalid transitions).
        assert!(
            report.fitness > 0.0,
            "Fitness should be > 0, got {}",
            report.fitness
        );
        // With rushed overlay, the anomaly_rate should be captured.
        // (We check the stats are computed, not the exact value.)
        assert!(report.anomaly_stats.total_events > 0, "Should have events");
    }

    #[test]
    fn test_precision_computed() {
        let (events, bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        let report = analyze_conformance(&events, &bp);

        assert!(
            report.precision > 0.0,
            "Precision should be > 0, got {}",
            report.precision
        );
        assert!(
            report.precision <= 1.0,
            "Precision should be <= 1.0, got {}",
            report.precision
        );
    }

    #[test]
    fn test_per_procedure_conformance() {
        let (events, bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        let report = analyze_conformance(&events, &bp);

        // Should have a conformance entry for each procedure in the blueprint.
        let total_procedures: usize = bp.phases.iter().map(|p| p.procedures.len()).sum();
        assert_eq!(
            report.per_procedure.len(),
            total_procedures,
            "Expected {} per-procedure entries, got {}",
            total_procedures,
            report.per_procedure.len()
        );

        // Each entry should have reasonable values.
        for pc in &report.per_procedure {
            assert!(
                pc.fitness >= 0.0 && pc.fitness <= 1.0,
                "Procedure '{}' fitness out of range: {}",
                pc.procedure_id,
                pc.fitness
            );
        }
    }

    #[test]
    fn test_conformance_report_serializes() {
        let (events, bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        let report = analyze_conformance(&events, &bp);

        // JSON roundtrip.
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(!json.is_empty());
        let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(deserialized.get("fitness").is_some());
        assert!(deserialized.get("precision").is_some());
        assert!(deserialized.get("anomaly_stats").is_some());
        assert!(deserialized.get("per_procedure").is_some());
    }

    #[test]
    fn test_generalization_score() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let bp = bwp.blueprint.clone();
        let overlay = default_overlay();
        let ctx = EngagementContext::demo();
        let gen = compute_generalization(&bwp, &overlay, &bp, 42, &ctx);

        assert!(
            gen >= 0.0 && gen <= 1.0,
            "Generalization should be in [0, 1], got {}",
            gen
        );
        // With deterministic FSM, fitness should be very consistent across seeds.
        assert!(
            gen > 0.8,
            "Generalization should be > 0.8 for consistent FSM, got {}",
            gen
        );
    }

    #[test]
    fn test_evaluate_detector_perfect() {
        let (events, _bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        // Perfect detector: predictions match ground truth exactly.
        let predictions: Vec<bool> = events.iter().map(|e| e.is_anomaly).collect();
        let metrics = evaluate_detector(&events, &predictions).unwrap();

        assert!(
            (metrics.f1 - 1.0).abs() < f64::EPSILON || metrics.true_positives == 0,
            "Perfect detector should have F1=1.0 or no anomalies to detect"
        );
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 0);
    }

    #[test]
    fn test_evaluate_detector_all_positive() {
        let (events, _bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        // Naive detector: predicts everything as anomaly.
        let predictions = vec![true; events.len()];
        let metrics = evaluate_detector(&events, &predictions).unwrap();

        // All actual anomalies found (FN=0) but many false positives.
        assert_eq!(metrics.false_negatives, 0);
        assert!(metrics.recall == 1.0 || metrics.true_positives == 0);
    }

    #[test]
    fn test_evaluate_detector_serializes() {
        let (events, _bp) = run_fsa_engagement(BuiltinOverlay::Default, 42);
        let predictions: Vec<bool> = events.iter().map(|e| e.is_anomaly).collect();
        let metrics = evaluate_detector(&events, &predictions).unwrap();

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("f1"));
        assert!(json.contains("precision"));
        assert!(json.contains("recall"));
    }
}
