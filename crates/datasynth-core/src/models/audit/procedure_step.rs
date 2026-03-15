//! Audit procedure step models per ISA 330.
//!
//! Represents individual steps within a workpaper, each addressing a specific
//! assertion using a defined procedure type.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Assertion;

/// Type of substantive or control testing procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepProcedureType {
    /// Physical or documentary inspection
    #[default]
    Inspection,
    /// Direct observation of a process or activity
    Observation,
    /// Inquiry of management or personnel
    Inquiry,
    /// External or internal confirmation
    Confirmation,
    /// Independent recalculation
    Recalculation,
    /// Independent re-execution of a procedure
    Reperformance,
    /// Analytical procedure (ratio, trend, expectation)
    AnalyticalProcedure,
    /// Tracing from document to ledger (vouching direction)
    Vouching,
    /// High-level review scan for unusual items
    Scanning,
}

/// Status of an audit procedure step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Planned but not yet started
    #[default]
    Planned,
    /// Currently being performed
    InProgress,
    /// Completed
    Complete,
    /// Deferred to a later date
    Deferred,
    /// Not applicable to this engagement
    NotApplicable,
}

/// Result of a completed audit procedure step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepResult {
    /// No exception noted
    #[default]
    Pass,
    /// A failure / deviation found
    Fail,
    /// An exception noted (may be less than a full failure)
    Exception,
    /// Result is inconclusive; additional work required
    Inconclusive,
}

/// A single documented step within an audit workpaper (ISA 330).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditProcedureStep {
    /// Unique step ID
    pub step_id: Uuid,
    /// Step reference code, e.g. "STEP-a1b2c3d4-01"
    pub step_ref: String,
    /// Workpaper this step belongs to
    pub workpaper_id: Uuid,
    /// Engagement this step belongs to
    pub engagement_id: Uuid,
    /// Sequential step number within the workpaper
    pub step_number: u32,
    /// Description of the procedure
    pub description: String,
    /// Type of audit procedure
    pub procedure_type: StepProcedureType,
    /// Assertion addressed by this step
    pub assertion: Assertion,
    /// Planned performance date
    pub planned_date: Option<NaiveDate>,
    /// Actual performance date
    pub performed_date: Option<NaiveDate>,
    /// User ID of the performer
    pub performed_by: Option<String>,
    /// Display name of the performer
    pub performed_by_name: Option<String>,
    /// Current status of the step
    pub status: StepStatus,
    /// Result after completion
    pub result: Option<StepResult>,
    /// Whether an exception was noted
    pub exception_noted: bool,
    /// Description of any exception
    pub exception_description: Option<String>,
    /// Linked audit sample, if sampling was used
    pub sample_id: Option<Uuid>,
    /// Evidence items supporting this step
    pub evidence_ids: Vec<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last-modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl AuditProcedureStep {
    /// Create a new planned audit procedure step.
    pub fn new(
        workpaper_id: Uuid,
        engagement_id: Uuid,
        step_number: u32,
        description: impl Into<String>,
        procedure_type: StepProcedureType,
        assertion: Assertion,
    ) -> Self {
        let now = Utc::now();
        let step_ref = format!("STEP-{}-{:02}", &workpaper_id.to_string()[..8], step_number,);
        Self {
            step_id: Uuid::new_v4(),
            step_ref,
            workpaper_id,
            engagement_id,
            step_number,
            description: description.into(),
            procedure_type,
            assertion,
            planned_date: None,
            performed_date: None,
            performed_by: None,
            performed_by_name: None,
            status: StepStatus::Planned,
            result: None,
            exception_noted: false,
            exception_description: None,
            sample_id: None,
            evidence_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Link this step to an audit sample.
    pub fn with_sample(mut self, sample_id: Uuid) -> Self {
        self.sample_id = Some(sample_id);
        self
    }

    /// Attach a set of evidence items to this step.
    pub fn with_evidence(mut self, evidence_ids: Vec<Uuid>) -> Self {
        self.evidence_ids = evidence_ids;
        self
    }

    /// Record performance of the step.
    ///
    /// Sets the performer, date, result, status (`Complete`), and
    /// `exception_noted` (true when `result` is `Exception`).
    pub fn perform(&mut self, by: String, by_name: String, date: NaiveDate, result: StepResult) {
        self.performed_by = Some(by);
        self.performed_by_name = Some(by_name);
        self.performed_date = Some(date);
        self.result = Some(result);
        self.exception_noted = matches!(result, StepResult::Exception);
        self.status = StepStatus::Complete;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_step() -> AuditProcedureStep {
        AuditProcedureStep::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            "Inspect invoices for proper authorisation",
            StepProcedureType::Inspection,
            Assertion::Occurrence,
        )
    }

    #[test]
    fn test_new_step() {
        let step = make_step();
        assert_eq!(step.step_number, 1);
        assert_eq!(step.status, StepStatus::Planned);
        assert!(step.result.is_none());
        assert!(!step.exception_noted);
        assert!(step.step_ref.starts_with("STEP-"));
        assert!(step.step_ref.ends_with("-01"));
    }

    #[test]
    fn test_perform_sets_fields() {
        let mut step = make_step();
        let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        step.perform("u123".into(), "Alice Audit".into(), date, StepResult::Pass);

        assert_eq!(step.status, StepStatus::Complete);
        assert_eq!(step.result, Some(StepResult::Pass));
        assert_eq!(step.performed_by.as_deref(), Some("u123"));
        assert_eq!(step.performed_by_name.as_deref(), Some("Alice Audit"));
        assert_eq!(step.performed_date, Some(date));
        assert!(!step.exception_noted);
    }

    #[test]
    fn test_perform_exception_noted() {
        let mut step = make_step();
        let date = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap();
        step.perform(
            "u456".into(),
            "Bob Check".into(),
            date,
            StepResult::Exception,
        );

        assert!(step.exception_noted);
        assert_eq!(step.result, Some(StepResult::Exception));
    }

    #[test]
    fn test_with_sample() {
        let sample_id = Uuid::new_v4();
        let step = make_step().with_sample(sample_id);
        assert_eq!(step.sample_id, Some(sample_id));
    }

    #[test]
    fn test_with_evidence() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let step = make_step().with_evidence(ids.clone());
        assert_eq!(step.evidence_ids, ids);
    }

    #[test]
    fn test_step_status_serde() {
        let statuses = [
            StepStatus::Planned,
            StepStatus::InProgress,
            StepStatus::Complete,
            StepStatus::Deferred,
            StepStatus::NotApplicable,
        ];
        for s in &statuses {
            let json = serde_json::to_string(s).unwrap();
            let back: StepStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *s);
        }
    }

    #[test]
    fn test_step_result_serde() {
        let results = [
            StepResult::Pass,
            StepResult::Fail,
            StepResult::Exception,
            StepResult::Inconclusive,
        ];
        for r in &results {
            let json = serde_json::to_string(r).unwrap();
            let back: StepResult = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *r);
        }
    }

    #[test]
    fn test_procedure_type_serde() {
        let types = [
            StepProcedureType::Inspection,
            StepProcedureType::Observation,
            StepProcedureType::Inquiry,
            StepProcedureType::Confirmation,
            StepProcedureType::Recalculation,
            StepProcedureType::Reperformance,
            StepProcedureType::AnalyticalProcedure,
            StepProcedureType::Vouching,
            StepProcedureType::Scanning,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let back: StepProcedureType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *t);
        }
    }
}
