//! Live anomaly injection into an already-generated event log.
//!
//! Simulates emerging risks by marking existing events with anomaly flags
//! after the engagement has been generated, producing labeled anomaly
//! records alongside the modified event trail.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

use crate::event::{AnomalySeverity, AuditAnomalyRecord, AuditAnomalyType, AuditEvent};

/// Configuration for a single class of live anomaly injection.
#[derive(Debug, Clone)]
pub struct LiveInjectionConfig {
    /// The type of anomaly to inject.
    pub anomaly_type: AuditAnomalyType,
    /// If set, only inject into events matching this procedure id.
    /// If `None`, any event may be targeted.
    pub target_procedure: Option<String>,
    /// Probability (0.0–1.0) that a matching event will be injected.
    pub injection_probability: f64,
    /// Severity level for injected anomalies.
    pub severity: AnomalySeverity,
}

/// Inject anomalies into an already-generated event log at runtime.
///
/// Iterates `events`, and for each event that matches a config's target
/// (or any event when `target_procedure` is `None`), probabilistically
/// sets `is_anomaly = true` and records an [`AuditAnomalyRecord`].
///
/// Returns the list of injected anomaly records.
pub fn inject_live_anomalies(
    events: &mut [AuditEvent],
    configs: &[LiveInjectionConfig],
    seed: u64,
) -> Vec<AuditAnomalyRecord> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut records = Vec::new();

    for event in events.iter_mut() {
        // Skip events that are already anomalous (from the engine's own injection).
        if event.is_anomaly {
            continue;
        }

        for config in configs {
            // Check procedure filter.
            if let Some(ref target) = config.target_procedure {
                if event.procedure_id != *target {
                    continue;
                }
            }

            // Probabilistic injection.
            let roll: f64 = rand::Rng::random(&mut rng);
            if roll >= config.injection_probability {
                continue;
            }

            // Mark the event as anomalous.
            event.is_anomaly = true;
            event.anomaly_type = Some(config.anomaly_type);

            // Build a deterministic anomaly id.
            let id_bytes: [u8; 16] = rand::Rng::random(&mut rng);
            let anomaly_id = Uuid::from_bytes(id_bytes);

            let description = format!(
                "Live-injected {} on procedure '{}' (step {:?})",
                config.anomaly_type,
                event.procedure_id,
                event.step_id.as_deref().unwrap_or("N/A"),
            );

            records.push(AuditAnomalyRecord {
                anomaly_id,
                anomaly_type: config.anomaly_type,
                severity: config.severity,
                procedure_id: event.procedure_id.clone(),
                step_id: event.step_id.clone(),
                timestamp: event.timestamp,
                description,
            });

            // Only inject one anomaly per event (first matching config wins).
            break;
        }
    }

    records
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EngagementContext;
    use crate::engine::AuditFsmEngine;
    use crate::loader::{default_overlay, BlueprintWithPreconditions};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    /// Generate a baseline event log for testing.
    fn generate_events() -> Vec<AuditEvent> {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        bwp.validate().unwrap();
        let overlay = default_overlay();
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let ctx = EngagementContext::test_default();
        engine.run_engagement(&ctx).unwrap().event_log
    }

    #[test]
    fn test_injection_adds_anomalies() {
        let mut events = generate_events();
        assert!(!events.is_empty());

        // Count pre-existing anomalies from the engine.
        let pre_existing = events.iter().filter(|e| e.is_anomaly).count();

        let configs = vec![LiveInjectionConfig {
            anomaly_type: AuditAnomalyType::MissingEvidence,
            target_procedure: None,
            injection_probability: 0.5,
            severity: AnomalySeverity::Medium,
        }];

        let records = inject_live_anomalies(&mut events, &configs, 99);

        // With 50% probability on a non-trivial log, we expect at least one injection.
        assert!(
            !records.is_empty(),
            "should inject at least one anomaly at 50% probability"
        );

        // Total anomalous count should be pre-existing + live-injected.
        let total_anomalous = events.iter().filter(|e| e.is_anomaly).count();
        assert_eq!(
            total_anomalous,
            pre_existing + records.len(),
            "total anomalous = pre-existing + live-injected"
        );

        // Verify record fields.
        for rec in &records {
            assert_eq!(rec.anomaly_type, AuditAnomalyType::MissingEvidence);
            assert_eq!(rec.severity, AnomalySeverity::Medium);
        }
    }

    #[test]
    fn test_targeted_injection_only_affects_specified_procedure() {
        let mut events = generate_events();

        // Pick a procedure id that exists in the log and is not already anomalous.
        let target_proc = events
            .iter()
            .find(|e| !e.procedure_id.is_empty() && !e.is_anomaly)
            .map(|e| e.procedure_id.clone())
            .expect("should have at least one non-anomalous event with a procedure id");

        // Record which events were already anomalous before injection.
        let pre_anomalous: std::collections::HashSet<uuid::Uuid> = events
            .iter()
            .filter(|e| e.is_anomaly)
            .map(|e| e.event_id)
            .collect();

        let configs = vec![LiveInjectionConfig {
            anomaly_type: AuditAnomalyType::SkippedApproval,
            target_procedure: Some(target_proc.clone()),
            injection_probability: 1.0, // 100% — inject every matching event
            severity: AnomalySeverity::High,
        }];

        let records = inject_live_anomalies(&mut events, &configs, 55);

        // All anomaly records must target the specified procedure.
        for rec in &records {
            assert_eq!(
                rec.procedure_id, target_proc,
                "injected anomaly should only affect target procedure"
            );
        }

        // Events for other procedures that were NOT already anomalous should remain clean.
        for event in &events {
            if event.procedure_id != target_proc && !pre_anomalous.contains(&event.event_id) {
                assert!(
                    !event.is_anomaly,
                    "non-pre-existing anomalies for other procedures should be unaffected"
                );
            }
        }
    }

    #[test]
    fn test_injection_is_deterministic() {
        let events_a = generate_events();
        let events_b = generate_events();

        let configs = vec![LiveInjectionConfig {
            anomaly_type: AuditAnomalyType::LatePosting,
            target_procedure: None,
            injection_probability: 0.3,
            severity: AnomalySeverity::Low,
        }];

        let mut events_a_mut = events_a;
        let mut events_b_mut = events_b;

        let records_a = inject_live_anomalies(&mut events_a_mut, &configs, 123);
        let records_b = inject_live_anomalies(&mut events_b_mut, &configs, 123);

        assert_eq!(
            records_a.len(),
            records_b.len(),
            "deterministic injection must produce same count"
        );

        for (a, b) in records_a.iter().zip(records_b.iter()) {
            assert_eq!(a.anomaly_id, b.anomaly_id, "anomaly IDs must match");
            assert_eq!(a.procedure_id, b.procedure_id, "procedure IDs must match");
            assert_eq!(a.step_id, b.step_id, "step IDs must match");
        }
    }
}
