//! Audit event types and builder for the FSM event trail.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A single record in the audit event trail, capturing a state transition or procedure step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub phase_id: String,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub actor_id: String,
    pub command: String,
    pub evidence_refs: Vec<String>,
    pub standards_refs: Vec<String>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AuditAnomalyType>,
}

/// Categories of audit anomalies that can be injected into the event trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAnomalyType {
    SkippedApproval,
    LatePosting,
    MissingEvidence,
    OutOfSequence,
    InsufficientDocumentation,
}

impl fmt::Display for AuditAnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AuditAnomalyType::SkippedApproval => "skipped_approval",
            AuditAnomalyType::LatePosting => "late_posting",
            AuditAnomalyType::MissingEvidence => "missing_evidence",
            AuditAnomalyType::OutOfSequence => "out_of_sequence",
            AuditAnomalyType::InsufficientDocumentation => "insufficient_documentation",
        };
        write!(f, "{s}")
    }
}

/// Severity levels for audit anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A labelled anomaly record produced alongside the event trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnomalyRecord {
    pub anomaly_id: Uuid,
    pub anomaly_type: AuditAnomalyType,
    pub severity: AnomalySeverity,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub timestamp: NaiveDateTime,
    pub description: String,
}

/// Builder for constructing [`AuditEvent`] instances with deterministic UUIDs.
pub struct AuditEventBuilder {
    event_type: String,
    procedure_id: String,
    step_id: Option<String>,
    phase_id: String,
    from_state: Option<String>,
    to_state: Option<String>,
    actor_id: String,
    command: String,
    evidence_refs: Vec<String>,
    standards_refs: Vec<String>,
    timestamp: Option<NaiveDateTime>,
    is_anomaly: bool,
    anomaly_type: Option<AuditAnomalyType>,
}

impl AuditEventBuilder {
    fn new() -> Self {
        Self {
            event_type: String::new(),
            procedure_id: String::new(),
            step_id: None,
            phase_id: String::new(),
            from_state: None,
            to_state: None,
            actor_id: String::new(),
            command: String::new(),
            evidence_refs: Vec::new(),
            standards_refs: Vec::new(),
            timestamp: None,
            is_anomaly: false,
            anomaly_type: None,
        }
    }

    /// Start building a state-transition event.
    pub fn transition() -> Self {
        Self::new()
    }

    /// Start building a procedure-step event.
    pub fn step() -> Self {
        Self::new()
    }

    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    pub fn procedure_id(mut self, procedure_id: impl Into<String>) -> Self {
        self.procedure_id = procedure_id.into();
        self
    }

    pub fn step_id(mut self, step_id: impl Into<String>) -> Self {
        self.step_id = Some(step_id.into());
        self
    }

    pub fn phase_id(mut self, phase_id: impl Into<String>) -> Self {
        self.phase_id = phase_id.into();
        self
    }

    pub fn from_state(mut self, from_state: impl Into<String>) -> Self {
        self.from_state = Some(from_state.into());
        self
    }

    pub fn to_state(mut self, to_state: impl Into<String>) -> Self {
        self.to_state = Some(to_state.into());
        self
    }

    pub fn actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = actor_id.into();
        self
    }

    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    pub fn evidence_ref(mut self, evidence_ref: impl Into<String>) -> Self {
        self.evidence_refs.push(evidence_ref.into());
        self
    }

    pub fn standard_ref(mut self, standard_ref: impl Into<String>) -> Self {
        self.standards_refs.push(standard_ref.into());
        self
    }

    pub fn timestamp(mut self, timestamp: NaiveDateTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn anomaly(mut self, anomaly_type: AuditAnomalyType) -> Self {
        self.is_anomaly = true;
        self.anomaly_type = Some(anomaly_type);
        self
    }

    /// Build the [`AuditEvent`], generating a deterministic UUID from the provided RNG.
    pub fn build_with_rng(self, rng: &mut impl rand::Rng) -> AuditEvent {
        let bytes: [u8; 16] = rng.random();
        let event_id = uuid::Builder::from_random_bytes(bytes).into_uuid();

        let timestamp = self.timestamp.unwrap_or_else(|| {
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
        });

        AuditEvent {
            event_id,
            timestamp,
            event_type: self.event_type,
            procedure_id: self.procedure_id,
            step_id: self.step_id,
            phase_id: self.phase_id,
            from_state: self.from_state,
            to_state: self.to_state,
            actor_id: self.actor_id,
            command: self.command,
            evidence_refs: self.evidence_refs,
            standards_refs: self.standards_refs,
            is_anomaly: self.is_anomaly,
            anomaly_type: self.anomaly_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_event_builder_transition() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let ts = NaiveDate::from_ymd_opt(2025, 3, 1)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();

        let event = AuditEventBuilder::transition()
            .event_type("state_transition")
            .procedure_id("PROC-001")
            .phase_id("planning")
            .from_state("not_started")
            .to_state("in_progress")
            .actor_id("auditor-1")
            .command("begin_planning")
            .timestamp(ts)
            .build_with_rng(&mut rng);

        assert_eq!(event.event_type, "state_transition");
        assert_eq!(event.procedure_id, "PROC-001");
        assert_eq!(event.phase_id, "planning");
        assert_eq!(event.from_state.as_deref(), Some("not_started"));
        assert_eq!(event.to_state.as_deref(), Some("in_progress"));
        assert_eq!(event.actor_id, "auditor-1");
        assert_eq!(event.command, "begin_planning");
        assert_eq!(event.timestamp, ts);
        assert!(!event.is_anomaly);
        assert!(event.anomaly_type.is_none());
        assert!(event.step_id.is_none());
    }

    #[test]
    fn test_event_builder_step() {
        let mut rng = ChaCha8Rng::seed_from_u64(99);
        let ts = NaiveDate::from_ymd_opt(2025, 4, 15)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap();

        let event = AuditEventBuilder::step()
            .event_type("procedure_step")
            .procedure_id("PROC-002")
            .step_id("STEP-001")
            .phase_id("fieldwork")
            .actor_id("auditor-2")
            .command("execute_step")
            .evidence_ref("EVD-001")
            .standard_ref("ISA-500")
            .timestamp(ts)
            .build_with_rng(&mut rng);

        assert_eq!(event.step_id.as_deref(), Some("STEP-001"));
        assert_eq!(event.evidence_refs, vec!["EVD-001"]);
        assert_eq!(event.standards_refs, vec!["ISA-500"]);
        assert_eq!(event.procedure_id, "PROC-002");
        assert_eq!(event.phase_id, "fieldwork");
    }

    #[test]
    fn test_deterministic_event_ids() {
        let ts = NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let mut rng1 = ChaCha8Rng::seed_from_u64(7);
        let event1 = AuditEventBuilder::transition()
            .event_type("transition")
            .procedure_id("P1")
            .phase_id("phase1")
            .actor_id("actor")
            .command("cmd")
            .timestamp(ts)
            .build_with_rng(&mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(7);
        let event2 = AuditEventBuilder::transition()
            .event_type("transition")
            .procedure_id("P1")
            .phase_id("phase1")
            .actor_id("actor")
            .command("cmd")
            .timestamp(ts)
            .build_with_rng(&mut rng2);

        assert_eq!(event1.event_id, event2.event_id);
    }

    #[test]
    fn test_anomaly_type_display() {
        assert_eq!(
            AuditAnomalyType::SkippedApproval.to_string(),
            "skipped_approval"
        );
        assert_eq!(AuditAnomalyType::LatePosting.to_string(), "late_posting");
        assert_eq!(
            AuditAnomalyType::MissingEvidence.to_string(),
            "missing_evidence"
        );
        assert_eq!(
            AuditAnomalyType::OutOfSequence.to_string(),
            "out_of_sequence"
        );
        assert_eq!(
            AuditAnomalyType::InsufficientDocumentation.to_string(),
            "insufficient_documentation"
        );
    }
}
