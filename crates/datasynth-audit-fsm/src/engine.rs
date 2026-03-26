//! FSM execution engine for audit engagements.
//!
//! Walks each procedure's state machine in DAG order, emitting deterministic
//! [`AuditEvent`] records and optional anomaly labels.

use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal};

use serde::Serialize;

use crate::artifact::ArtifactBag;
use crate::context::EngagementContext;
use crate::dispatch::StepDispatcher;
use crate::error::AuditFsmError;
use crate::event::{
    AnomalySeverity, AuditAnomalyRecord, AuditAnomalyType, AuditEvent, AuditEventBuilder,
};
use crate::loader::BlueprintWithPreconditions;
use crate::schema::{AuditBlueprint, BlueprintProcedure, GenerationOverlay, ProcedureTransition};

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

/// The output of a complete engagement simulation.
#[derive(Serialize)]
pub struct EngagementResult {
    /// Ordered event trail for the entire engagement.
    pub event_log: Vec<AuditEvent>,
    /// Final FSM state for each procedure (keyed by procedure id).
    pub procedure_states: HashMap<String, String>,
    /// Whether each step was completed (keyed by step id).
    pub step_completions: HashMap<String, bool>,
    /// Evidence state (keyed by evidence template id).
    pub evidence_states: HashMap<String, String>,
    /// Anomaly records injected during the engagement.
    pub anomalies: Vec<AuditAnomalyRecord>,
    /// Phases whose exit-gate conditions were satisfied.
    pub phases_completed: Vec<String>,
    /// Cumulative wall-clock hours of the engagement.
    pub total_duration_hours: f64,
    /// Typed audit artifacts accumulated by step dispatchers.
    pub artifacts: ArtifactBag,
}

// ---------------------------------------------------------------------------
// Internal accumulator (bundles mutable state to avoid many fn args)
// ---------------------------------------------------------------------------

/// Mutable accumulators passed through the engine's walk.
struct RunAccum {
    event_log: Vec<AuditEvent>,
    procedure_states: HashMap<String, String>,
    step_completions: HashMap<String, bool>,
    evidence_states: HashMap<String, String>,
    anomalies: Vec<AuditAnomalyRecord>,
    artifacts: ArtifactBag,
    current_ts: NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The FSM execution engine.
///
/// Given a validated [`AuditBlueprint`], a [`GenerationOverlay`], and an
/// [`EngagementContext`], the engine walks each procedure's state machine in
/// topological (DAG) order, emitting a deterministic event trail.
pub struct AuditFsmEngine {
    pub(crate) blueprint: AuditBlueprint,
    overlay: GenerationOverlay,
    rng: ChaCha8Rng,
    preconditions: HashMap<String, Vec<String>>,
    dispatcher: StepDispatcher,
}

impl AuditFsmEngine {
    /// Create a new engine from a validated blueprint-with-preconditions bundle
    /// and an overlay.
    ///
    /// The `seed` for the [`StepDispatcher`] is derived from the first value
    /// drawn from the RNG clone so that the dispatcher has a deterministic but
    /// independent seed, leaving the engine's own RNG sequence unchanged.
    pub fn new(
        bwp: BlueprintWithPreconditions,
        overlay: GenerationOverlay,
        rng: ChaCha8Rng,
    ) -> Self {
        // Derive a dispatcher seed without consuming the engine RNG.
        // We use a clone to draw a u64, keeping the original stream intact.
        let dispatcher_seed: u64 = {
            let mut seed_rng = rng.clone();
            seed_rng.random()
        };
        Self {
            blueprint: bwp.blueprint,
            preconditions: bwp.preconditions,
            overlay,
            rng,
            dispatcher: StepDispatcher::new(dispatcher_seed),
        }
    }

    /// Run the full engagement simulation.
    pub fn run_engagement(
        &mut self,
        context: &EngagementContext,
    ) -> Result<EngagementResult, AuditFsmError> {
        // 1. Build procedure-id -> (phase_id, &BlueprintProcedure) lookup.
        let mut proc_lookup: HashMap<String, (String, BlueprintProcedure)> = HashMap::new();
        for phase in &self.blueprint.phases {
            for proc in &phase.procedures {
                proc_lookup.insert(proc.id.clone(), (phase.id.clone(), proc.clone()));
            }
        }

        // 2. Get DAG execution order.
        let bwp = BlueprintWithPreconditions {
            blueprint: self.blueprint.clone(),
            preconditions: self.preconditions.clone(),
        };
        let exec_order = bwp.topological_sort()?;

        // 3. Initialise result accumulators.
        let start_ts = context
            .engagement_start
            .and_hms_opt(9, 0, 0)
            .unwrap_or_default();

        let mut acc = RunAccum {
            event_log: Vec::new(),
            procedure_states: HashMap::new(),
            step_completions: HashMap::new(),
            evidence_states: HashMap::new(),
            anomalies: Vec::new(),
            artifacts: ArtifactBag::default(),
            current_ts: start_ts,
        };

        // 4. Walk each procedure in DAG order.
        for proc_id in &exec_order {
            let (phase_id, proc) = match proc_lookup.get(proc_id) {
                Some(entry) => entry.clone(),
                None => continue,
            };

            // Skip procedures that don't match the overlay's discriminator filter.
            if self.should_skip_procedure(&proc) {
                continue;
            }

            let agg = &proc.aggregate;
            if agg.initial_state.is_empty() || agg.states.is_empty() {
                // No FSM defined for this procedure — skip.
                continue;
            }

            let mut current_state = agg.initial_state.clone();
            acc.procedure_states
                .insert(proc_id.clone(), current_state.clone());

            let max_iter = self
                .overlay
                .iteration_limits
                .per_procedure
                .get(proc_id)
                .copied()
                .unwrap_or(self.overlay.iteration_limits.default);

            let mut iterations = 0;
            let mut self_loop_count: usize = 0;
            loop {
                if iterations >= max_iter {
                    break;
                }
                iterations += 1;

                // Find outgoing transitions from current_state.
                let outgoing: Vec<&ProcedureTransition> = agg
                    .transitions
                    .iter()
                    .filter(|t| t.from_state == current_state)
                    .collect();

                if outgoing.is_empty() {
                    // Terminal state — no outgoing transitions.
                    break;
                }

                // Select transition.
                let mut transition = self.select_transition(&outgoing, proc_id, &agg.states);

                // Self-loop detection and bounding.
                if transition.from_state == transition.to_state {
                    self_loop_count += 1;
                    if self_loop_count >= self.overlay.max_self_loop_iterations {
                        // Force forward: find a non-self-loop transition.
                        if let Some(forward) = outgoing.iter().find(|t| t.from_state != t.to_state)
                        {
                            transition = forward;
                            self_loop_count = 0;
                        } else {
                            break; // No forward transition available.
                        }
                    }
                } else {
                    self_loop_count = 0;
                }

                // Determine actor for this transition.
                let actor_id = self.select_actor_for_transition(transition, &proc);

                // Emit transition event.
                let event = AuditEventBuilder::transition()
                    .event_type("state_transition")
                    .procedure_id(proc_id.clone())
                    .phase_id(phase_id.clone())
                    .from_state(current_state.clone())
                    .to_state(transition.to_state.clone())
                    .actor_id(actor_id.clone())
                    .command(
                        transition
                            .command
                            .clone()
                            .unwrap_or_else(|| "transition".to_string()),
                    )
                    .timestamp(acc.current_ts)
                    .build_with_rng(&mut self.rng);

                acc.event_log.push(event);

                let previous_state = current_state.clone();
                current_state = transition.to_state.clone();
                acc.procedure_states
                    .insert(proc_id.clone(), current_state.clone());

                // Execute steps when entering the "work" state — the state
                // immediately after the initial state.  FSA uses "in_progress",
                // IA continuous phases use "active".  We detect it as the
                // second state in the aggregate's state list, or fall back to
                // matching "in_progress"/"active" by name.
                let work_state = agg
                    .states
                    .get(1)
                    .map(|s| s.as_str())
                    .unwrap_or("in_progress");
                let is_self_loop = previous_state == current_state;
                let entering_work = current_state == work_state && previous_state != work_state;
                if entering_work || is_self_loop {
                    self.execute_steps(&proc, &phase_id, &mut acc, context);
                }

                // Advance time between transitions.
                acc.current_ts = self.advance_time(acc.current_ts);
            }
        }

        // 5. Determine which phases are completed.
        let phases_completed = self.compute_completed_phases(&acc.procedure_states);

        // 6. Total duration.
        let total_duration_hours = (acc.current_ts - start_ts).num_minutes() as f64 / 60.0;

        Ok(EngagementResult {
            event_log: acc.event_log,
            procedure_states: acc.procedure_states,
            step_completions: acc.step_completions,
            evidence_states: acc.evidence_states,
            anomalies: acc.anomalies,
            phases_completed,
            total_duration_hours,
            artifacts: acc.artifacts,
        })
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Select a transition when multiple outgoing edges exist from the current
    /// state. If there are forward and backward transitions (revision loop),
    /// use `revision_probability` to decide.
    ///
    /// Forward vs backward is determined by state ordering in the aggregate:
    /// a transition targeting a higher-indexed state is forward, lower is backward.
    fn select_transition<'a>(
        &mut self,
        outgoing: &[&'a ProcedureTransition],
        _proc_id: &str,
        states: &[String],
    ) -> &'a ProcedureTransition {
        if outgoing.len() == 1 {
            return outgoing[0];
        }

        let revision_prob = self.overlay.transitions.defaults.revision_probability;
        let roll: f64 = self.rng.random();

        // Determine forward vs backward by state index in the aggregate.
        // The from_state is the same for all outgoing transitions.
        let from_state = &outgoing[0].from_state;
        let from_idx = states.iter().position(|s| s == from_state).unwrap_or(0);

        // Forward: targets a state with higher index than current
        // Backward (revision): targets a state with lower or equal index
        let forward = outgoing
            .iter()
            .find(|t| states.iter().position(|s| s == &t.to_state).unwrap_or(0) > from_idx);
        let backward = outgoing
            .iter()
            .find(|t| states.iter().position(|s| s == &t.to_state).unwrap_or(0) <= from_idx);

        if roll < revision_prob {
            if let Some(rev) = backward {
                return rev;
            }
        }

        forward.unwrap_or(&outgoing[0])
    }

    /// Determine the actor for a transition. Approval commands get a senior
    /// actor; others fall back to the first step's actor or a default.
    fn select_actor_for_transition(
        &self,
        transition: &ProcedureTransition,
        proc: &BlueprintProcedure,
    ) -> String {
        let cmd = transition.command.as_deref().unwrap_or("");
        if cmd.contains("approve") || cmd.contains("issue") {
            // Senior actor for approval transitions.
            self.blueprint
                .actors
                .first()
                .map(|a| a.id.clone())
                .unwrap_or_else(|| "engagement_partner".to_string())
        } else {
            // Use the first step's actor or fall back.
            proc.steps
                .first()
                .and_then(|s| s.actor.clone())
                .unwrap_or_else(|| {
                    self.blueprint
                        .actors
                        .last()
                        .map(|a| a.id.clone())
                        .unwrap_or_else(|| "audit_staff".to_string())
                })
        }
    }

    /// Execute all steps within a procedure (called when entering in_progress).
    fn execute_steps(
        &mut self,
        proc: &BlueprintProcedure,
        phase_id: &str,
        acc: &mut RunAccum,
        context: &EngagementContext,
    ) {
        for step in &proc.steps {
            // Advance a small amount of time between steps.
            acc.current_ts = self.advance_step_time(acc.current_ts);

            // Check for anomaly injection.
            let (is_anomaly, anomaly_type) = self.check_anomaly_injection(
                &proc.id,
                &step.id,
                acc.current_ts,
                &mut acc.anomalies,
            );

            let actor_id = step
                .actor
                .clone()
                .unwrap_or_else(|| "audit_staff".to_string());

            let cmd = step
                .command
                .clone()
                .unwrap_or_else(|| format!("execute_{}", step.id));

            // Build evidence refs from step evidence items.
            let mut builder = AuditEventBuilder::step()
                .event_type("procedure_step")
                .procedure_id(proc.id.clone())
                .step_id(step.id.clone())
                .phase_id(phase_id.to_string())
                .actor_id(actor_id)
                .command(cmd)
                .timestamp(acc.current_ts);

            for ev in &step.evidence {
                if let Some(ref tpl_ref) = ev.template_ref {
                    builder = builder.evidence_ref(tpl_ref.ref_id.clone());
                    // Track evidence state.
                    acc.evidence_states
                        .insert(tpl_ref.ref_id.clone(), ev.direction.clone());
                }
            }

            for std_ref in &step.standards {
                builder = builder.standard_ref(std_ref.ref_id.clone());
            }

            if is_anomaly {
                if let Some(at) = anomaly_type {
                    builder = builder.anomaly(at);
                }
            }

            let event = builder.build_with_rng(&mut self.rng);
            acc.event_log.push(event);

            // Dispatch step to the appropriate generator to produce artifacts.
            self.dispatcher
                .dispatch(step, &proc.id, context, &mut acc.artifacts);

            acc.step_completions.insert(step.id.clone(), true);
        }
    }

    /// Check whether an anomaly should be injected for this step.
    /// Returns (is_anomaly, anomaly_type). If injected, also records
    /// the anomaly in the anomalies vec.
    fn check_anomaly_injection(
        &mut self,
        procedure_id: &str,
        step_id: &str,
        ts: NaiveDateTime,
        anomalies: &mut Vec<AuditAnomalyRecord>,
    ) -> (bool, Option<AuditAnomalyType>) {
        let cfg = &self.overlay.anomalies;

        // Check each anomaly type against its probability.
        let anomaly_checks: [(f64, AuditAnomalyType, AnomalySeverity, &str); 4] = [
            (
                cfg.skipped_approval,
                AuditAnomalyType::SkippedApproval,
                AnomalySeverity::High,
                "Approval step was skipped",
            ),
            (
                cfg.late_posting,
                AuditAnomalyType::LatePosting,
                AnomalySeverity::Medium,
                "Step was posted late",
            ),
            (
                cfg.missing_evidence,
                AuditAnomalyType::MissingEvidence,
                AnomalySeverity::High,
                "Required evidence was not attached",
            ),
            (
                cfg.out_of_sequence,
                AuditAnomalyType::OutOfSequence,
                AnomalySeverity::Medium,
                "Step executed out of defined sequence",
            ),
        ];

        for (prob, anomaly_type, severity, desc) in &anomaly_checks {
            let roll: f64 = self.rng.random();
            if roll < *prob {
                let bytes: [u8; 16] = self.rng.random();
                let anomaly_id = uuid::Builder::from_random_bytes(bytes).into_uuid();

                anomalies.push(AuditAnomalyRecord {
                    anomaly_id,
                    anomaly_type: *anomaly_type,
                    severity: *severity,
                    procedure_id: procedure_id.to_string(),
                    step_id: Some(step_id.to_string()),
                    timestamp: ts,
                    description: desc.to_string(),
                });

                return (true, Some(*anomaly_type));
            }
        }

        (false, None)
    }

    /// Advance time by a log-normal distributed delay (inter-transition).
    fn advance_time(&mut self, current: NaiveDateTime) -> NaiveDateTime {
        let timing = &self.overlay.transitions.defaults.timing;
        let delay_hours = self.sample_lognormal_hours(timing.mu_hours, timing.sigma_hours);
        current + Duration::minutes((delay_hours * 60.0) as i64)
    }

    /// Advance time by a shorter delay between steps within a procedure.
    fn advance_step_time(&mut self, current: NaiveDateTime) -> NaiveDateTime {
        // Steps are much faster than transitions: ~1-4 hours.
        let delay_hours = self.sample_lognormal_hours(1.0, 0.5);
        current + Duration::minutes((delay_hours * 60.0) as i64)
    }

    /// Sample from a log-normal distribution parameterized by the desired
    /// mean (`mu`) and standard deviation (`sigma`) in hours.
    ///
    /// Converts from the natural-scale `(mu, sigma)` to the log-space
    /// parameters `(mu_ln, sigma_ln)` that `LogNormal::new` expects:
    /// ```text
    /// sigma_ln = sqrt(ln(1 + variance / mu^2))
    /// mu_ln    = ln(mu / sqrt(1 + variance / mu^2))
    /// ```
    /// Falls back to `mu` if the parameters would produce an invalid
    /// distribution.
    fn sample_lognormal_hours(&mut self, mu: f64, sigma: f64) -> f64 {
        let mu = mu.max(0.01);
        let sigma = sigma.max(0.01);
        let variance = sigma * sigma;
        let sigma_ln = (1.0 + variance / (mu * mu)).ln().sqrt();
        let mu_ln = (mu * mu / (mu * mu + variance).sqrt()).ln();
        // Ensure log-space parameters are valid (positive sigma, finite mu).
        let sigma_ln = sigma_ln.max(0.01);
        let mu_ln = if mu_ln.is_finite() { mu_ln } else { 0.01 };
        match LogNormal::new(mu_ln, sigma_ln) {
            Ok(dist) => {
                let sample: f64 = dist.sample(&mut self.rng);
                // Clamp to reasonable range: 0.1 .. 200 hours.
                sample.clamp(0.1, 200.0)
            }
            Err(_) => mu.max(0.1),
        }
    }

    /// Check whether a procedure should be skipped based on the overlay's
    /// discriminator filter. Returns `true` if the procedure does NOT match
    /// the overlay filter and should be excluded.
    fn should_skip_procedure(&self, proc: &BlueprintProcedure) -> bool {
        let Some(ref overlay_discs) = self.overlay.discriminators else {
            return false; // No filter = run everything.
        };
        if proc.discriminators.is_empty() {
            return false; // No discriminators on procedure = always run.
        }
        for (category, filter_values) in overlay_discs {
            if let Some(proc_values) = proc.discriminators.get(category) {
                if !proc_values.iter().any(|v| filter_values.contains(v)) {
                    return true; // No match in this category = skip.
                }
            }
        }
        false
    }

    /// Determine which phases have all exit-gate conditions satisfied.
    ///
    /// Continuous phases (those with `order < 0`) are excluded — they run in
    /// parallel with sequential phases and are never "completed".
    fn compute_completed_phases(&self, procedure_states: &HashMap<String, String>) -> Vec<String> {
        let mut completed = Vec::new();

        for phase in &self.blueprint.phases {
            // Continuous phases (order < 0) are always active, not "completed".
            let is_continuous = phase.order.map(|o| o < 0).unwrap_or(false);
            if is_continuous {
                continue;
            }

            if let Some(ref gate) = phase.exit_gate {
                let all_met = gate.all_of.iter().all(|cond| {
                    // Gate predicate format: "procedure.<id>.<state>"
                    let parts: Vec<&str> = cond.predicate.splitn(3, '.').collect();
                    match (parts.first(), parts.get(1), parts.get(2)) {
                        (Some(&"procedure"), Some(proc_id), Some(required_state)) => {
                            procedure_states
                                .get(*proc_id)
                                .is_some_and(|s| s == *required_state)
                        }
                        _ => false,
                    }
                });
                if all_met {
                    completed.push(phase.id.clone());
                }
            } else {
                // No exit gate — phase is always considered completed.
                completed.push(phase.id.clone());
            }
        }

        completed
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EngagementContext;
    use crate::loader::{default_overlay, BlueprintWithPreconditions};
    use rand::SeedableRng;

    fn build_engine(seed: u64) -> AuditFsmEngine {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let overlay = default_overlay();
        let rng = ChaCha8Rng::seed_from_u64(seed);
        AuditFsmEngine::new(bwp, overlay, rng)
    }

    #[test]
    fn test_engine_loads() {
        let engine = build_engine(42);
        assert!(!engine.blueprint.phases.is_empty());
        assert!(!engine.preconditions.is_empty());
    }

    #[test]
    fn test_run_engagement_produces_events() {
        let mut engine = build_engine(42);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // With 9 procedures, each having >= 3 transition events, we expect
        // at least 14 total events (a very conservative lower bound).
        assert!(
            result.event_log.len() >= 14,
            "expected >= 14 events, got {}",
            result.event_log.len()
        );
    }

    #[test]
    fn test_deterministic_output() {
        let ctx = EngagementContext::demo();

        let mut engine1 = build_engine(42);
        let result1 = engine1.run_engagement(&ctx).unwrap();

        let mut engine2 = build_engine(42);
        let result2 = engine2.run_engagement(&ctx).unwrap();

        assert_eq!(
            result1.event_log.len(),
            result2.event_log.len(),
            "event counts differ between runs"
        );

        for (e1, e2) in result1.event_log.iter().zip(result2.event_log.iter()) {
            assert_eq!(e1.event_id, e2.event_id, "event_id mismatch");
            assert_eq!(e1.event_type, e2.event_type, "event_type mismatch");
            assert_eq!(e1.timestamp, e2.timestamp, "timestamp mismatch");
        }
    }

    #[test]
    fn test_all_procedures_reach_completed() {
        let mut engine = build_engine(42);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // All 9 procedures should reach "completed".
        for (proc_id, state) in &result.procedure_states {
            assert_eq!(
                state, "completed",
                "procedure '{}' ended in state '{}', expected 'completed'",
                proc_id, state
            );
        }
    }

    #[test]
    fn test_precondition_ordering_respected() {
        let mut engine = build_engine(42);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // accept_engagement must appear before planning_materiality in the
        // event log (by first occurrence of each procedure).
        let first_occurrence = |proc_id: &str| -> usize {
            result
                .event_log
                .iter()
                .position(|e| e.procedure_id == proc_id)
                .unwrap_or_else(|| panic!("no event for procedure '{}'", proc_id))
        };

        let accept_pos = first_occurrence("accept_engagement");
        let planning_pos = first_occurrence("planning_materiality");

        assert!(
            accept_pos < planning_pos,
            "accept_engagement (pos {}) should appear before planning_materiality (pos {})",
            accept_pos,
            planning_pos
        );
    }

    #[test]
    fn test_step_events_emitted() {
        let mut engine = build_engine(42);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        let step_events: Vec<&AuditEvent> = result
            .event_log
            .iter()
            .filter(|e| e.step_id.is_some())
            .collect();

        // 24 total steps across all 9 procedures; expect >= 15 step events.
        assert!(
            step_events.len() >= 15,
            "expected >= 15 step events, got {}",
            step_events.len()
        );
    }

    #[test]
    fn test_continuous_phases_excluded_from_completion() {
        // Use IA blueprint which has continuous phases (order < 0).
        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let overlay = default_overlay();
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // Continuous phase IDs (order < 0) should NOT appear in phases_completed.
        let continuous_ids: Vec<String> = engine
            .blueprint
            .phases
            .iter()
            .filter(|p| p.order.map(|o| o < 0).unwrap_or(false))
            .map(|p| p.id.clone())
            .collect();

        assert!(
            !continuous_ids.is_empty(),
            "IA blueprint should have at least one continuous phase"
        );

        for cid in &continuous_ids {
            assert!(
                !result.phases_completed.contains(cid),
                "Continuous phase '{}' should not be in phases_completed",
                cid
            );
        }
    }

    #[test]
    fn test_self_loop_bounded_by_overlay() {
        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let overlay = default_overlay(); // max_self_loop_iterations = 5
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // Count self-loop transitions per procedure.
        let mut loop_counts: HashMap<String, usize> = HashMap::new();
        for e in &result.event_log {
            if e.from_state.as_ref() == e.to_state.as_ref() && e.from_state.is_some() {
                *loop_counts.entry(e.procedure_id.clone()).or_default() += 1;
            }
        }

        for (proc_id, count) in &loop_counts {
            assert!(
                *count <= 5,
                "Procedure '{}' had {} self-loops, max is 5",
                proc_id,
                count
            );
        }
    }

    #[test]
    fn test_discriminator_filtering() {
        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let mut overlay = default_overlay();
        let mut disc = HashMap::new();
        disc.insert("categories".to_string(), vec!["financial".to_string()]);
        overlay.discriminators = Some(disc);

        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay, rng);
        let ctx = EngagementContext::demo();
        let filtered = engine.run_engagement(&ctx).unwrap();

        // Run unfiltered for comparison.
        let mut engine2 =
            AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
        let full = engine2.run_engagement(&ctx).unwrap();

        assert!(
            filtered.procedure_states.len() <= full.procedure_states.len(),
            "Filtered ({}) should have <= procedures than full ({})",
            filtered.procedure_states.len(),
            full.procedure_states.len()
        );
    }

    #[test]
    fn test_engine_produces_artifacts() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let overlay = default_overlay();
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // Verify artifacts were generated.
        assert!(
            result.artifacts.total_artifacts() > 0,
            "Expected artifacts, got 0"
        );
        assert!(
            !result.artifacts.engagements.is_empty(),
            "Expected at least one engagement"
        );
        assert!(
            !result.artifacts.materiality_calculations.is_empty(),
            "Expected materiality calculations"
        );
    }

    #[test]
    fn test_ia_improved_completion_with_higher_limit() {
        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let overlay = default_overlay(); // iteration_limits.default = 30
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();
        let total = result.procedure_states.len();
        // With limit=30 (up from 20), IA procedures with revision loops
        // get more headroom. Expect at least 20 of 34 to complete.
        assert!(
            completed >= 20,
            "Expected >= 20/{} completed with limit=30, got {}",
            total,
            completed
        );
    }

    #[test]
    fn test_per_procedure_iteration_limit_override() {
        use crate::schema::IterationLimits;

        let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let mut overlay = default_overlay();
        // Set a very low default but high limit for one specific procedure
        overlay.iteration_limits = IterationLimits {
            default: 3,
            per_procedure: {
                let mut m = HashMap::new();
                // develop_findings has a long C2CE lifecycle
                m.insert("develop_findings".to_string(), 50);
                m
            },
        };
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::demo();
        let result = engine.run_engagement(&ctx).unwrap();

        // develop_findings should reach "closed" with its generous per-procedure limit
        let df_state = result
            .procedure_states
            .get("develop_findings")
            .expect("develop_findings should be in procedure_states");
        assert_eq!(
            df_state, "closed",
            "develop_findings should reach 'closed' with per-procedure limit=50, got '{}'",
            df_state
        );

        // Most other procedures should be stuck (low default=3 iterations)
        let non_completed = result
            .procedure_states
            .iter()
            .filter(|(id, state)| {
                id.as_str() != "develop_findings"
                    && state.as_str() != "completed"
                    && state.as_str() != "closed"
            })
            .count();
        assert!(
            non_completed > 0,
            "With default limit=3, some procedures should not complete"
        );
    }
}
