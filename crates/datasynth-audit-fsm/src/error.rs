//! Error types for FSM operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditFsmError {
    #[error("Blueprint parse error ({path}): {source}")]
    BlueprintParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("Blueprint validation failed: {violations:?}")]
    BlueprintValidation {
        violations: Vec<ValidationViolation>,
    },

    #[error("Overlay parse error ({path}): {source}")]
    OverlayParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("Guard failure in procedure '{procedure_id}': guard '{guard}' — {reason}")]
    GuardFailure {
        procedure_id: String,
        guard: String,
        reason: String,
    },

    #[error(
        "Precondition not met for '{procedure_id}': requires '{required}' but was '{actual_state}'"
    )]
    PreconditionNotMet {
        procedure_id: String,
        required: String,
        actual_state: String,
    },

    #[error("Source not found: {source_id}")]
    SourceNotFound { source_id: String },

    #[error("DAG cycle detected involving procedures: {procedures:?}")]
    DagCycle { procedures: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct ValidationViolation {
    pub location: String,
    pub message: String,
}

impl std::fmt::Display for ValidationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}
