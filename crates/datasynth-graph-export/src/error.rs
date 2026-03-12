//! Error types for the graph export pipeline.
//!
//! Two categories:
//! - [`ExportError`]: Fatal errors that abort the pipeline.
//! - [`ExportWarning`]: Non-fatal issues collected during export.

use std::fmt;

/// Fatal error that stops the export pipeline.
#[derive(Debug)]
pub enum ExportError {
    /// A required pipeline stage failed.
    StageError {
        /// Name of the stage that failed (e.g., "property_serialization", "edge_synthesis").
        stage: &'static str,
        /// Human-readable description of the failure.
        message: String,
    },
    /// The input data is fundamentally invalid.
    InvalidInput {
        /// What was wrong with the input.
        message: String,
    },
    /// Budget constraints make the export impossible.
    BudgetExhausted {
        /// Which budget was exceeded (e.g., "nodes", "edges").
        resource: &'static str,
        /// The limit that was exceeded.
        limit: usize,
        /// The actual count.
        actual: usize,
    },
    /// An ID mapping failed (forward or reverse lookup).
    IdMappingError {
        /// The identifier that could not be resolved.
        key: String,
        /// Additional context.
        message: String,
    },
    /// Serialization error (serde).
    Serialization(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::StageError { stage, message } => {
                write!(f, "export stage '{stage}' failed: {message}")
            }
            ExportError::InvalidInput { message } => {
                write!(f, "invalid input: {message}")
            }
            ExportError::BudgetExhausted {
                resource,
                limit,
                actual,
            } => {
                write!(
                    f,
                    "budget exhausted: {resource} limit={limit}, actual={actual}"
                )
            }
            ExportError::IdMappingError { key, message } => {
                write!(f, "ID mapping error for '{key}': {message}")
            }
            ExportError::Serialization(msg) => {
                write!(f, "serialization error: {msg}")
            }
        }
    }
}

impl std::error::Error for ExportError {}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self {
        ExportError::Serialization(e.to_string())
    }
}

/// Severity level for non-fatal warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningSeverity {
    /// Informational — the pipeline handled it automatically.
    Info,
    /// The output may be slightly degraded.
    Low,
    /// Something significant was skipped or approximated.
    Medium,
    /// Data quality concern — review recommended.
    High,
}

impl fmt::Display for WarningSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarningSeverity::Info => write!(f, "INFO"),
            WarningSeverity::Low => write!(f, "LOW"),
            WarningSeverity::Medium => write!(f, "MEDIUM"),
            WarningSeverity::High => write!(f, "HIGH"),
        }
    }
}

/// A single non-fatal warning produced during export.
#[derive(Debug, Clone)]
pub struct ExportWarning {
    /// Which pipeline stage produced this warning.
    pub stage: &'static str,
    /// Severity of the warning.
    pub severity: WarningSeverity,
    /// Human-readable description.
    pub message: String,
}

impl fmt::Display for ExportWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.stage, self.message)
    }
}

/// Accumulator for non-fatal warnings collected during the export pipeline.
#[derive(Debug, Clone, Default)]
pub struct ExportWarnings {
    warnings: Vec<ExportWarning>,
}

impl ExportWarnings {
    /// Create an empty warning accumulator.
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn push(&mut self, warning: ExportWarning) {
        self.warnings.push(warning);
    }

    /// Add a warning with the given stage, severity, and message.
    pub fn add(&mut self, stage: &'static str, severity: WarningSeverity, message: String) {
        self.warnings.push(ExportWarning {
            stage,
            severity,
            message,
        });
    }

    /// Add an info-level warning.
    pub fn info(&mut self, stage: &'static str, message: String) {
        self.add(stage, WarningSeverity::Info, message);
    }

    /// Add a medium-severity warning.
    pub fn warn(&mut self, stage: &'static str, message: String) {
        self.add(stage, WarningSeverity::Medium, message);
    }

    /// Returns true if there are no warnings.
    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }

    /// Number of warnings.
    pub fn len(&self) -> usize {
        self.warnings.len()
    }

    /// Iterate over warnings.
    pub fn iter(&self) -> impl Iterator<Item = &ExportWarning> {
        self.warnings.iter()
    }

    /// Consume and return the inner Vec.
    pub fn into_vec(self) -> Vec<ExportWarning> {
        self.warnings
    }

    /// Count warnings at or above the given severity.
    pub fn count_at_severity(&self, min_severity: WarningSeverity) -> usize {
        self.warnings
            .iter()
            .filter(|w| w.severity >= min_severity)
            .count()
    }

    /// Merge another ExportWarnings into this one.
    pub fn merge(&mut self, other: ExportWarnings) {
        self.warnings.extend(other.warnings);
    }
}

impl IntoIterator for ExportWarnings {
    type Item = ExportWarning;
    type IntoIter = std::vec::IntoIter<ExportWarning>;

    fn into_iter(self) -> Self::IntoIter {
        self.warnings.into_iter()
    }
}

impl<'a> IntoIterator for &'a ExportWarnings {
    type Item = &'a ExportWarning;
    type IntoIter = std::slice::Iter<'a, ExportWarning>;

    fn into_iter(self) -> Self::IntoIter {
        self.warnings.iter()
    }
}
