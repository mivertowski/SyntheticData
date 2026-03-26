//! Overlay parameter fitting from target engagement metrics.
//!
//! Given an [`EngagementProfile`] describing desired engagement characteristics
//! (duration, event count, finding count, revision rate, anomaly rate, completion
//! rate), this module iteratively adjusts [`GenerationOverlay`] parameters until
//! Monte Carlo simulations match the targets within a configurable tolerance.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    loader::{default_overlay, BlueprintWithPreconditions},
    schema::GenerationOverlay,
};

// ---------------------------------------------------------------------------
// Target profile
// ---------------------------------------------------------------------------

/// Desired engagement characteristics that the fitting algorithm targets.
#[derive(Debug, Clone)]
pub struct EngagementProfile {
    /// Target average engagement duration in hours.
    pub target_duration_hours: f64,
    /// Target average event count per engagement.
    pub target_event_count: usize,
    /// Target average finding count per engagement.
    pub target_finding_count: usize,
    /// Target revision rate (fraction of total transitions that are revisions).
    pub target_revision_rate: f64,
    /// Target anomaly rate (fraction of events flagged as anomalies).
    pub target_anomaly_rate: f64,
    /// Target completion rate (fraction of procedures reaching completed/closed).
    pub target_completion_rate: f64,
}

// ---------------------------------------------------------------------------
// Achieved metrics
// ---------------------------------------------------------------------------

/// Mean metrics observed from Monte Carlo simulation runs.
#[derive(Debug, Clone, Serialize)]
pub struct EngagementMetrics {
    /// Average engagement duration in hours.
    pub avg_duration_hours: f64,
    /// Average event count per engagement.
    pub avg_event_count: f64,
    /// Average finding count per engagement.
    pub avg_finding_count: f64,
    /// Average revision rate (revision transitions / total events).
    pub avg_revision_rate: f64,
    /// Average anomaly rate (anomaly events / total events).
    pub avg_anomaly_rate: f64,
    /// Average completion rate (completed procedures / total procedures).
    pub avg_completion_rate: f64,
}

// ---------------------------------------------------------------------------
// Fitted result
// ---------------------------------------------------------------------------

/// The result of an overlay fitting run.
#[derive(Debug, Clone, Serialize)]
pub struct FittedOverlay {
    /// The adjusted overlay parameters.
    pub overlay: GenerationOverlay,
    /// Metrics achieved with the fitted overlay.
    pub achieved_metrics: EngagementMetrics,
    /// Number of fitting iterations executed.
    pub iterations: usize,
    /// Whether the algorithm converged (residual < threshold).
    pub converged: bool,
    /// Final normalized residual distance to target.
    pub residual: f64,
}

// ---------------------------------------------------------------------------
// Core fitting algorithm
// ---------------------------------------------------------------------------

/// Iteratively adjust overlay parameters until Monte Carlo simulations match
/// the target engagement profile.
///
/// # Algorithm
///
/// 1. Start with [`default_overlay()`].
/// 2. For each iteration:
///    a. Run `samples_per_iteration` engagements with deterministic seeds.
///    b. Compute mean metrics across samples.
///    c. Compute normalized residual distance to target.
///    d. If residual < 0.05 (5%), stop (converged).
///    e. Adjust overlay parameters proportionally toward targets with clamping.
/// 3. Return the fitted overlay with final metrics.
///
/// # Arguments
///
/// * `bwp` - Validated blueprint with preconditions.
/// * `profile` - Target engagement profile to fit toward.
/// * `max_iterations` - Maximum fitting iterations (recommended: 10-20).
/// * `samples_per_iteration` - Monte Carlo runs per evaluation (recommended: 3-5).
/// * `base_seed` - Base RNG seed for reproducibility.
pub fn fit_overlay(
    bwp: &BlueprintWithPreconditions,
    profile: &EngagementProfile,
    max_iterations: usize,
    samples_per_iteration: usize,
    base_seed: u64,
    context: &EngagementContext,
) -> FittedOverlay {
    assert!(max_iterations >= 1, "max_iterations must be >= 1");
    assert!(
        samples_per_iteration >= 1,
        "samples_per_iteration must be >= 1"
    );

    let mut overlay = default_overlay();
    let mut best_residual = f64::MAX;
    let mut best_overlay = overlay.clone();
    let mut best_metrics =
        compute_metrics(bwp, &overlay, samples_per_iteration, base_seed, 0, context);
    let mut iterations_used = 0;

    for iter in 0..max_iterations {
        iterations_used = iter + 1;

        let metrics = compute_metrics(
            bwp,
            &overlay,
            samples_per_iteration,
            base_seed,
            iter as u64 * samples_per_iteration as u64,
            context,
        );
        let residual = compute_residual(&metrics, profile);

        if residual < best_residual {
            best_residual = residual;
            best_overlay = overlay.clone();
            best_metrics = metrics.clone();
        }

        // Converged — within 5% of target.
        if residual < 0.05 {
            return FittedOverlay {
                overlay: best_overlay,
                achieved_metrics: best_metrics,
                iterations: iterations_used,
                converged: true,
                residual: best_residual,
            };
        }

        // Adjust overlay parameters proportionally.
        adjust_overlay(&mut overlay, &metrics, profile);
    }

    FittedOverlay {
        overlay: best_overlay,
        achieved_metrics: best_metrics,
        iterations: iterations_used,
        converged: best_residual < 0.05,
        residual: best_residual,
    }
}

// ---------------------------------------------------------------------------
// Metrics computation
// ---------------------------------------------------------------------------

/// Run `samples` engagements and compute mean metrics.
fn compute_metrics(
    bwp: &BlueprintWithPreconditions,
    overlay: &GenerationOverlay,
    samples: usize,
    base_seed: u64,
    seed_offset: u64,
    context: &EngagementContext,
) -> EngagementMetrics {
    let mut total_duration = 0.0;
    let mut total_events = 0.0;
    let mut total_findings = 0.0;
    let mut total_revision_rate = 0.0;
    let mut total_anomaly_rate = 0.0;
    let mut total_completion_rate = 0.0;
    let mut successful_runs = 0usize;

    for i in 0..samples {
        let iter_seed = base_seed.wrapping_add(seed_offset).wrapping_add(i as u64);
        let rng = ChaCha8Rng::seed_from_u64(iter_seed);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);

        let result = match engine.run_engagement(context) {
            Ok(r) => r,
            Err(_) => continue,
        };

        successful_runs += 1;

        let event_count = result.event_log.len();
        total_duration += result.total_duration_hours;
        total_events += event_count as f64;
        total_findings += result.artifacts.findings.len() as f64;

        // Revision rate: events where under_review -> in_progress divided by total events.
        let revision_count = result
            .event_log
            .iter()
            .filter(|e| {
                e.from_state.as_deref() == Some("under_review")
                    && e.to_state.as_deref() == Some("in_progress")
            })
            .count();
        total_revision_rate += if event_count > 0 {
            revision_count as f64 / event_count as f64
        } else {
            0.0
        };

        // Anomaly rate: events with is_anomaly == true divided by total events.
        let anomaly_count = result.event_log.iter().filter(|e| e.is_anomaly).count();
        total_anomaly_rate += if event_count > 0 {
            anomaly_count as f64 / event_count as f64
        } else {
            0.0
        };

        // Completion rate: procedures in completed or closed / total procedures.
        let total_procs = result.procedure_states.len();
        let completed_procs = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();
        total_completion_rate += if total_procs > 0 {
            completed_procs as f64 / total_procs as f64
        } else {
            0.0
        };
    }

    let n = successful_runs.max(1) as f64;

    EngagementMetrics {
        avg_duration_hours: total_duration / n,
        avg_event_count: total_events / n,
        avg_finding_count: total_findings / n,
        avg_revision_rate: total_revision_rate / n,
        avg_anomaly_rate: total_anomaly_rate / n,
        avg_completion_rate: total_completion_rate / n,
    }
}

// ---------------------------------------------------------------------------
// Residual computation
// ---------------------------------------------------------------------------

/// Compute normalized distance between achieved metrics and target profile.
///
/// Each metric contributes equally (1/6 weight) via its relative error:
/// `|achieved - target| / max(target, epsilon)`.
fn compute_residual(metrics: &EngagementMetrics, profile: &EngagementProfile) -> f64 {
    let eps = 1e-6;
    let n_metrics = 6.0;

    let dur_err = (metrics.avg_duration_hours - profile.target_duration_hours).abs()
        / profile.target_duration_hours.max(eps);
    let evt_err = (metrics.avg_event_count - profile.target_event_count as f64).abs()
        / (profile.target_event_count as f64).max(eps);
    let find_err = (metrics.avg_finding_count - profile.target_finding_count as f64).abs()
        / (profile.target_finding_count as f64).max(eps);
    let rev_err = (metrics.avg_revision_rate - profile.target_revision_rate).abs()
        / profile.target_revision_rate.max(eps);
    let anom_err = (metrics.avg_anomaly_rate - profile.target_anomaly_rate).abs()
        / profile.target_anomaly_rate.max(eps);
    let comp_err = (metrics.avg_completion_rate - profile.target_completion_rate).abs()
        / profile.target_completion_rate.max(eps);

    (dur_err + evt_err + find_err + rev_err + anom_err + comp_err) / n_metrics
}

// ---------------------------------------------------------------------------
// Overlay adjustment
// ---------------------------------------------------------------------------

/// Adjust overlay parameters proportionally toward the target profile.
///
/// Each parameter is scaled by `target / achieved` (clamped to [0.5x, 2.0x]
/// per step to prevent oscillation) and then clamped to sane absolute ranges.
fn adjust_overlay(
    overlay: &mut GenerationOverlay,
    metrics: &EngagementMetrics,
    profile: &EngagementProfile,
) {
    let eps = 1e-6;

    // --- Duration: adjust timing.mu_hours ---
    let duration_ratio =
        clamp_ratio(profile.target_duration_hours / metrics.avg_duration_hours.max(eps));
    overlay.transitions.defaults.timing.mu_hours *= duration_ratio;
    // Also adjust sigma proportionally to keep the distribution shape.
    overlay.transitions.defaults.timing.sigma_hours *= duration_ratio;
    // Clamp mu_hours to [0.5, 5000.0] hours.
    overlay.transitions.defaults.timing.mu_hours = overlay
        .transitions
        .defaults
        .timing
        .mu_hours
        .clamp(0.5, 5000.0);
    overlay.transitions.defaults.timing.sigma_hours = overlay
        .transitions
        .defaults
        .timing
        .sigma_hours
        .clamp(0.1, 2000.0);

    // --- Revision rate: adjust revision_probability ---
    let revision_ratio =
        clamp_ratio(profile.target_revision_rate / metrics.avg_revision_rate.max(eps));
    overlay.transitions.defaults.revision_probability *= revision_ratio;
    overlay.transitions.defaults.revision_probability = overlay
        .transitions
        .defaults
        .revision_probability
        .clamp(0.01, 0.5);

    // --- Anomaly rates: scale all anomaly probabilities ---
    let anomaly_ratio =
        clamp_ratio(profile.target_anomaly_rate / metrics.avg_anomaly_rate.max(eps));
    overlay.anomalies.skipped_approval =
        (overlay.anomalies.skipped_approval * anomaly_ratio).clamp(0.0, 0.5);
    overlay.anomalies.late_posting =
        (overlay.anomalies.late_posting * anomaly_ratio).clamp(0.0, 0.5);
    overlay.anomalies.missing_evidence =
        (overlay.anomalies.missing_evidence * anomaly_ratio).clamp(0.0, 0.5);
    overlay.anomalies.out_of_sequence =
        (overlay.anomalies.out_of_sequence * anomaly_ratio).clamp(0.0, 0.5);
}

/// Clamp a ratio to [0.5, 2.0] to prevent wild oscillation.
fn clamp_ratio(ratio: f64) -> f64 {
    ratio.clamp(0.5, 2.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fsa() -> BlueprintWithPreconditions {
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint should load")
    }

    #[test]
    fn test_fit_overlay_converges_to_target_duration() {
        // Target long duration (2000h) — should increase mu_hours.
        let bwp = load_fsa();
        let profile = EngagementProfile {
            target_duration_hours: 2000.0,
            target_event_count: 50,
            target_finding_count: 5,
            target_revision_rate: 0.15,
            target_anomaly_rate: 0.05,
            target_completion_rate: 1.0,
        };
        let fitted = fit_overlay(&bwp, &profile, 15, 3, 42, &EngagementContext::demo());
        // Achieved duration should be closer to 2000 than default (~800).
        assert!(
            fitted.achieved_metrics.avg_duration_hours > 1000.0,
            "Fitted duration {:.0} should approach target 2000",
            fitted.achieved_metrics.avg_duration_hours
        );
    }

    #[test]
    fn test_fit_overlay_adjusts_anomaly_rate() {
        // Target high anomaly rate.
        let bwp = load_fsa();
        let profile = EngagementProfile {
            target_duration_hours: 800.0,
            target_event_count: 50,
            target_finding_count: 5,
            target_revision_rate: 0.15,
            target_anomaly_rate: 0.20,
            target_completion_rate: 1.0,
        };
        let fitted = fit_overlay(&bwp, &profile, 15, 3, 42, &EngagementContext::demo());
        assert!(
            fitted.achieved_metrics.avg_anomaly_rate > 0.05,
            "Anomaly rate {:.3} should increase toward target 0.20",
            fitted.achieved_metrics.avg_anomaly_rate
        );
    }

    #[test]
    fn test_fit_overlay_returns_valid_overlay() {
        let bwp = load_fsa();
        let profile = EngagementProfile {
            target_duration_hours: 800.0,
            target_event_count: 50,
            target_finding_count: 3,
            target_revision_rate: 0.10,
            target_anomaly_rate: 0.05,
            target_completion_rate: 1.0,
        };
        let fitted = fit_overlay(&bwp, &profile, 10, 3, 42, &EngagementContext::demo());
        // Overlay should have valid parameter ranges.
        assert!(fitted.overlay.transitions.defaults.revision_probability >= 0.0);
        assert!(fitted.overlay.transitions.defaults.revision_probability <= 0.5);
        assert!(fitted.overlay.transitions.defaults.timing.mu_hours > 0.0);
    }

    #[test]
    fn test_fit_overlay_serializes() {
        let bwp = load_fsa();
        let profile = EngagementProfile {
            target_duration_hours: 800.0,
            target_event_count: 50,
            target_finding_count: 3,
            target_revision_rate: 0.10,
            target_anomaly_rate: 0.05,
            target_completion_rate: 1.0,
        };
        let fitted = fit_overlay(&bwp, &profile, 5, 2, 42, &EngagementContext::demo());
        let json = serde_json::to_string(&fitted).unwrap();
        assert!(json.contains("converged"));
        assert!(json.contains("residual"));
    }

    #[test]
    fn test_fit_overlay_deterministic() {
        let bwp = load_fsa();
        let profile = EngagementProfile {
            target_duration_hours: 1200.0,
            target_event_count: 50,
            target_finding_count: 5,
            target_revision_rate: 0.15,
            target_anomaly_rate: 0.05,
            target_completion_rate: 1.0,
        };
        let f1 = fit_overlay(&bwp, &profile, 5, 2, 42, &EngagementContext::demo());
        let f2 = fit_overlay(&bwp, &profile, 5, 2, 42, &EngagementContext::demo());
        assert_eq!(f1.iterations, f2.iterations);
        assert!(
            (f1.residual - f2.residual).abs() < 0.001,
            "Residuals should match: {} vs {}",
            f1.residual,
            f2.residual
        );
    }
}
