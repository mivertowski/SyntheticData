//! Process variant and case trace models for OCPM.
//!
//! Process variants represent distinct execution patterns through processes.
//! Case traces link events to process instances.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::models::BusinessProcess;

/// A distinct execution sequence through the process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessVariant {
    /// Unique variant ID
    pub variant_id: String,
    /// Business process
    pub business_process: BusinessProcess,
    /// Sequence of activity IDs in this variant
    pub activity_sequence: Vec<String>,
    /// Frequency count (how many times this variant occurred)
    pub frequency: u64,
    /// Percentage of total cases
    pub frequency_percent: f64,
    /// Average duration for this variant (in hours)
    pub avg_duration_hours: f64,
    /// Minimum duration observed
    pub min_duration_hours: f64,
    /// Maximum duration observed
    pub max_duration_hours: f64,
    /// Standard deviation of duration
    pub std_duration_hours: f64,
    /// Example case IDs following this variant
    pub example_case_ids: Vec<Uuid>,
    /// Is this a happy path (expected) variant
    pub is_happy_path: bool,
    /// Deviation indicators
    pub has_rework: bool,
    /// Has skipped steps
    pub has_skipped_steps: bool,
    /// Has out-of-order steps
    pub has_out_of_order: bool,
}

impl ProcessVariant {
    /// Create a new process variant.
    pub fn new(variant_id: &str, business_process: BusinessProcess) -> Self {
        Self {
            variant_id: variant_id.into(),
            business_process,
            activity_sequence: Vec::new(),
            frequency: 0,
            frequency_percent: 0.0,
            avg_duration_hours: 0.0,
            min_duration_hours: 0.0,
            max_duration_hours: 0.0,
            std_duration_hours: 0.0,
            example_case_ids: Vec::new(),
            is_happy_path: false,
            has_rework: false,
            has_skipped_steps: false,
            has_out_of_order: false,
        }
    }

    /// Set the activity sequence.
    pub fn with_sequence(mut self, sequence: Vec<&str>) -> Self {
        self.activity_sequence = sequence.into_iter().map(String::from).collect();
        self
    }

    /// Mark as happy path.
    pub fn happy_path(mut self) -> Self {
        self.is_happy_path = true;
        self
    }

    /// Mark as having rework.
    pub fn with_rework(mut self) -> Self {
        self.has_rework = true;
        self
    }

    /// Mark as having skipped steps.
    pub fn with_skipped_steps(mut self) -> Self {
        self.has_skipped_steps = true;
        self
    }

    /// Increment frequency and add example case.
    pub fn add_case(&mut self, case_id: Uuid, duration_hours: f64) {
        // Online update of statistics
        let n = self.frequency as f64;
        self.frequency += 1;

        if self.frequency == 1 {
            self.avg_duration_hours = duration_hours;
            self.min_duration_hours = duration_hours;
            self.max_duration_hours = duration_hours;
            self.std_duration_hours = 0.0;
        } else {
            // Welford's online algorithm for mean and variance
            let delta = duration_hours - self.avg_duration_hours;
            self.avg_duration_hours += delta / (n + 1.0);
            let delta2 = duration_hours - self.avg_duration_hours;
            // M2 update (for variance calculation)
            self.std_duration_hours =
                ((self.std_duration_hours.powi(2) * n + delta * delta2) / (n + 1.0)).sqrt();

            self.min_duration_hours = self.min_duration_hours.min(duration_hours);
            self.max_duration_hours = self.max_duration_hours.max(duration_hours);
        }

        // Keep only a few example cases
        if self.example_case_ids.len() < 5 {
            self.example_case_ids.push(case_id);
        }
    }

    /// Generate a hash-based variant ID from the sequence.
    pub fn sequence_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.activity_sequence.hash(&mut hasher);
        format!("V{:016X}", hasher.finish())
    }

    /// Standard P2P happy path variant.
    pub fn p2p_happy_path() -> Self {
        Self::new("P2P_HAPPY", BusinessProcess::P2P)
            .with_sequence(vec![
                "create_po",
                "approve_po",
                "release_po",
                "create_gr",
                "post_gr",
                "receive_invoice",
                "verify_invoice",
                "post_invoice",
                "execute_payment",
            ])
            .happy_path()
    }

    /// Standard O2C happy path variant.
    pub fn o2c_happy_path() -> Self {
        Self::new("O2C_HAPPY", BusinessProcess::O2C)
            .with_sequence(vec![
                "create_so",
                "check_credit",
                "release_so",
                "create_delivery",
                "pick",
                "pack",
                "ship",
                "create_customer_invoice",
                "post_customer_invoice",
                "receive_payment",
            ])
            .happy_path()
    }
}

/// Case execution trace linking events to a process instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseTrace {
    /// Unique case ID
    pub case_id: Uuid,
    /// Variant ID (computed after case completion)
    pub variant_id: Option<String>,
    /// Business process
    pub business_process: BusinessProcess,
    /// Case start time
    pub start_time: DateTime<Utc>,
    /// Case end time (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// Event IDs in chronological order
    pub event_ids: Vec<Uuid>,
    /// Activity sequence (for variant matching)
    pub activity_sequence: Vec<String>,
    /// Primary object ID (the main object being processed)
    pub primary_object_id: Uuid,
    /// Primary object type
    pub primary_object_type: String,
    /// Case status
    pub status: CaseStatus,
    /// Company code
    pub company_code: String,
    /// Is this case marked as anomalous
    pub is_anomaly: bool,
}

impl CaseTrace {
    /// Create a new case trace.
    pub fn new(
        business_process: BusinessProcess,
        primary_object_id: Uuid,
        primary_object_type: &str,
        company_code: &str,
    ) -> Self {
        Self {
            case_id: Uuid::new_v4(),
            variant_id: None,
            business_process,
            start_time: Utc::now(),
            end_time: None,
            event_ids: Vec::new(),
            activity_sequence: Vec::new(),
            primary_object_id,
            primary_object_type: primary_object_type.into(),
            status: CaseStatus::InProgress,
            company_code: company_code.into(),
            is_anomaly: false,
        }
    }

    /// Set a specific case ID (for deterministic generation).
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.case_id = id;
        self
    }

    /// Add an event to the trace.
    pub fn add_event(&mut self, event_id: Uuid, activity_id: &str, timestamp: DateTime<Utc>) {
        self.event_ids.push(event_id);
        self.activity_sequence.push(activity_id.into());

        // Update start time if this is the first event
        if self.event_ids.len() == 1 {
            self.start_time = timestamp;
        }
    }

    /// Complete the case.
    pub fn complete(&mut self) {
        self.status = CaseStatus::Completed;
        self.end_time = Some(Utc::now());
    }

    /// Complete the case at a specific time.
    pub fn complete_at(&mut self, end_time: DateTime<Utc>) {
        self.status = CaseStatus::Completed;
        self.end_time = Some(end_time);
    }

    /// Abort the case.
    pub fn abort(&mut self) {
        self.status = CaseStatus::Aborted;
        self.end_time = Some(Utc::now());
    }

    /// Get the duration in hours (if completed).
    pub fn duration_hours(&self) -> Option<f64> {
        self.end_time
            .map(|end| (end - self.start_time).num_seconds() as f64 / 3600.0)
    }

    /// Check if the case is completed.
    pub fn is_completed(&self) -> bool {
        matches!(self.status, CaseStatus::Completed)
    }
}

/// Case status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    /// Case is in progress
    #[default]
    InProgress,
    /// Case completed successfully
    Completed,
    /// Case was aborted
    Aborted,
    /// Case is on hold
    OnHold,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_creation() {
        let variant = ProcessVariant::p2p_happy_path();
        assert!(variant.is_happy_path);
        assert_eq!(variant.activity_sequence.len(), 9);
    }

    #[test]
    fn test_variant_statistics() {
        let mut variant = ProcessVariant::new("TEST", BusinessProcess::P2P);

        variant.add_case(Uuid::new_v4(), 10.0);
        variant.add_case(Uuid::new_v4(), 20.0);
        variant.add_case(Uuid::new_v4(), 15.0);

        assert_eq!(variant.frequency, 3);
        assert_eq!(variant.min_duration_hours, 10.0);
        assert_eq!(variant.max_duration_hours, 20.0);
        assert!((variant.avg_duration_hours - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_case_trace() {
        let po_id = Uuid::new_v4();
        let mut trace = CaseTrace::new(BusinessProcess::P2P, po_id, "purchase_order", "1000");

        trace.add_event(Uuid::new_v4(), "create_po", Utc::now());
        trace.add_event(Uuid::new_v4(), "approve_po", Utc::now());

        assert_eq!(trace.activity_sequence.len(), 2);
        assert_eq!(trace.status, CaseStatus::InProgress);

        trace.complete();
        assert!(trace.is_completed());
    }
}
