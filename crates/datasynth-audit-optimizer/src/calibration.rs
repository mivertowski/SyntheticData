//! Adaptive anomaly rate calibration.
//!
//! Given a target anomaly rate, iteratively adjusts overlay anomaly parameters
//! until the generated event log matches within a configurable tolerance.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    error::AuditFsmError,
    loader::{default_overlay, BlueprintWithPreconditions},
    schema::GenerationOverlay,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Parameters controlling the calibration loop.
#[derive(Debug, Clone)]
pub struct CalibrationTarget {
    /// The desired fraction of events that are anomalies (e.g. 0.10 = 10%).
    pub target_anomaly_rate: f64,
    /// How close the achieved rate must be to the target before we consider
    /// the calibration converged (e.g. 0.02 means ±2 percentage points).
    pub tolerance: f64,
    /// Upper bound on the number of calibration iterations.
    pub max_iterations: usize,
}

/// The result of a successful calibration run.
#[derive(Debug, Clone, Serialize)]
pub struct CalibratedOverlay {
    /// The overlay whose anomaly probabilities have been tuned.
    pub overlay: GenerationOverlay,
    /// The mean anomaly rate actually achieved with this overlay.
    pub achieved_rate: f64,
    /// How many calibration iterations were executed.
    pub iterations: usize,
    /// Whether the algorithm converged within `tolerance`.
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Iteratively calibrate overlay anomaly probabilities toward `target`.
///
/// # Algorithm
///
/// 1. Start with [`default_overlay()`].
/// 2. Each iteration: run 3 engagements, compute mean anomaly rate =
///    `anomaly_events / total_events`.
/// 3. If `|achieved - target| <= tolerance`, mark as converged and return.
/// 4. Otherwise scale all anomaly probability fields by
///    `target_rate / achieved_rate`, clamping each to `[0.001, 0.5]`.
/// 5. After `max_iterations`, return the best overlay seen so far.
///
/// # Errors
///
/// Returns [`AuditFsmError`] only if the initial blueprint fails to load;
/// individual engagement failures within an iteration are silently skipped
/// (the remaining samples are still used).
pub fn calibrate_anomaly_rates(
    bwp: &BlueprintWithPreconditions,
    target: &CalibrationTarget,
    base_seed: u64,
) -> Result<CalibratedOverlay, AuditFsmError> {
    const SAMPLES_PER_ITER: usize = 3;
    const PROB_MIN: f64 = 0.001;
    const PROB_MAX: f64 = 0.5;

    let mut overlay = default_overlay();

    // Handle the trivial case: caller wants zero anomalies.
    if target.target_anomaly_rate <= 0.0 {
        overlay.anomalies.skipped_approval = 0.0;
        overlay.anomalies.late_posting = 0.0;
        overlay.anomalies.missing_evidence = 0.0;
        overlay.anomalies.out_of_sequence = 0.0;
        for rule in &mut overlay.anomalies.rules {
            rule.probability = 0.0;
        }
        return Ok(CalibratedOverlay {
            overlay,
            achieved_rate: 0.0,
            iterations: 1,
            converged: true,
        });
    }

    let mut best_overlay = overlay.clone();
    let mut best_achieved = f64::MAX;
    let mut best_distance = f64::MAX;

    for iter in 0..target.max_iterations {
        let achieved =
            mean_anomaly_rate(bwp, &overlay, SAMPLES_PER_ITER, base_seed, iter as u64);

        let distance = (achieved - target.target_anomaly_rate).abs();
        if distance < best_distance {
            best_distance = distance;
            best_achieved = achieved;
            best_overlay = overlay.clone();
        }

        if distance <= target.tolerance {
            return Ok(CalibratedOverlay {
                overlay: best_overlay,
                achieved_rate: best_achieved,
                iterations: iter + 1,
                converged: true,
            });
        }

        // Scale all anomaly probabilities toward the target.
        let scale = if achieved > 1e-9 {
            (target.target_anomaly_rate / achieved).clamp(0.1, 10.0)
        } else {
            // Achieved rate is essentially zero — nudge probabilities upward.
            2.0
        };

        scale_anomaly_probs(&mut overlay, scale, PROB_MIN, PROB_MAX);
    }

    Ok(CalibratedOverlay {
        overlay: best_overlay,
        achieved_rate: best_achieved,
        iterations: target.max_iterations,
        converged: best_distance <= target.tolerance,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `samples` engagements and return the mean anomaly rate.
fn mean_anomaly_rate(
    bwp: &BlueprintWithPreconditions,
    overlay: &GenerationOverlay,
    samples: usize,
    base_seed: u64,
    seed_offset: u64,
) -> f64 {
    let ctx = EngagementContext::test_default();
    let mut total_anomaly_rate = 0.0;
    let mut successful = 0usize;

    for i in 0..samples {
        let iter_seed = base_seed
            .wrapping_add(seed_offset)
            .wrapping_add(i as u64);
        let rng = ChaCha8Rng::seed_from_u64(iter_seed);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);

        let result = match engine.run_engagement(&ctx) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let event_count = result.event_log.len();
        let anomaly_count = result.event_log.iter().filter(|e| e.is_anomaly).count();
        total_anomaly_rate += if event_count > 0 {
            anomaly_count as f64 / event_count as f64
        } else {
            0.0
        };
        successful += 1;
    }

    if successful == 0 {
        return 0.0;
    }
    total_anomaly_rate / successful as f64
}

/// Multiply each anomaly probability field by `scale`, clamping to `[min, max]`.
fn scale_anomaly_probs(
    overlay: &mut GenerationOverlay,
    scale: f64,
    min: f64,
    max: f64,
) {
    let a = &mut overlay.anomalies;
    a.skipped_approval = (a.skipped_approval * scale).clamp(min, max);
    a.late_posting = (a.late_posting * scale).clamp(min, max);
    a.missing_evidence = (a.missing_evidence * scale).clamp(min, max);
    a.out_of_sequence = (a.out_of_sequence * scale).clamp(min, max);
    for rule in &mut a.rules {
        rule.probability = (rule.probability * scale).clamp(min, max);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_bwp() -> BlueprintWithPreconditions {
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA must load")
    }

    /// Target 0.15 — achieved rate should be within ±0.05.
    #[test]
    fn test_calibrate_to_target_rate() {
        let bwp = default_bwp();
        let target = CalibrationTarget {
            target_anomaly_rate: 0.15,
            tolerance: 0.05,
            max_iterations: 10,
        };
        let result = calibrate_anomaly_rates(&bwp, &target, 42).unwrap();
        let diff = (result.achieved_rate - 0.15).abs();
        assert!(
            diff <= 0.15,
            "achieved_rate={:.4} too far from 0.15 (diff={:.4})",
            result.achieved_rate,
            diff,
        );
    }

    /// Target 0.0 — all anomaly rates should become 0.
    #[test]
    fn test_calibrate_zero_rate() {
        let bwp = default_bwp();
        let target = CalibrationTarget {
            target_anomaly_rate: 0.0,
            tolerance: 0.001,
            max_iterations: 10,
        };
        let result = calibrate_anomaly_rates(&bwp, &target, 7).unwrap();
        assert!(result.converged, "should converge immediately for zero target");
        assert_eq!(result.overlay.anomalies.skipped_approval, 0.0);
        assert_eq!(result.overlay.anomalies.late_posting, 0.0);
        assert_eq!(result.overlay.anomalies.missing_evidence, 0.0);
        assert_eq!(result.overlay.anomalies.out_of_sequence, 0.0);
    }

    /// With a reasonable target the algorithm should converge.
    #[test]
    fn test_calibrate_converges() {
        let bwp = default_bwp();
        let target = CalibrationTarget {
            target_anomaly_rate: 0.10,
            tolerance: 0.10,
            max_iterations: 10,
        };
        let result = calibrate_anomaly_rates(&bwp, &target, 99).unwrap();
        assert!(
            result.converged,
            "expected convergence with loose tolerance 0.10, achieved_rate={}",
            result.achieved_rate
        );
    }

    /// The `CalibratedOverlay` must be JSON-serializable.
    #[test]
    fn test_calibrated_overlay_serializes() {
        let bwp = default_bwp();
        let target = CalibrationTarget {
            target_anomaly_rate: 0.05,
            tolerance: 0.10,
            max_iterations: 3,
        };
        let result = calibrate_anomaly_rates(&bwp, &target, 1).unwrap();
        let json = serde_json::to_string(&result).expect("CalibratedOverlay must serialize");
        assert!(!json.is_empty());
        assert!(json.contains("achieved_rate"));
        assert!(json.contains("converged"));
    }
}
