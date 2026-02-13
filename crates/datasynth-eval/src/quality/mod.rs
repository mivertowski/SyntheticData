//! Data quality evaluation module.
//!
//! Validates data quality metrics including uniqueness, completeness,
//! format consistency, and cross-field consistency.

mod completeness;
mod consistency;
mod format;
mod uniqueness;

pub use completeness::{
    CompletenessAnalysis, CompletenessAnalyzer, FieldCompleteness, FieldDefinition, FieldValue,
};
pub use consistency::{ConsistencyAnalysis, ConsistencyAnalyzer, ConsistencyRule};
pub use format::{FormatAnalysis, FormatAnalyzer, FormatVariation};
pub use uniqueness::{DuplicateInfo, UniqueRecord, UniquenessAnalysis, UniquenessAnalyzer};

use serde::{Deserialize, Serialize};

/// Combined data quality evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEvaluation {
    /// Uniqueness analysis results.
    pub uniqueness: Option<UniquenessAnalysis>,
    /// Completeness analysis results.
    pub completeness: Option<CompletenessAnalysis>,
    /// Format consistency results.
    pub format: Option<FormatAnalysis>,
    /// Cross-field consistency results.
    pub consistency: Option<ConsistencyAnalysis>,
    /// Overall quality score (0.0-1.0).
    pub overall_score: f64,
    /// Whether quality meets thresholds.
    pub passes: bool,
    /// Quality issues found.
    pub issues: Vec<String>,
    /// Quality failures (alias for issues, used by report module).
    pub failures: Vec<String>,
}

impl QualityEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            uniqueness: None,
            completeness: None,
            format: None,
            consistency: None,
            overall_score: 1.0,
            passes: true,
            issues: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// Check all results against thresholds.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.issues.clear();
        self.failures.clear();
        let mut scores = Vec::new();

        if let Some(ref uniqueness) = self.uniqueness {
            if uniqueness.duplicate_rate > thresholds.duplicate_rate_max {
                self.issues.push(format!(
                    "Duplicate rate {} > {} (threshold)",
                    uniqueness.duplicate_rate, thresholds.duplicate_rate_max
                ));
            }
            scores.push(1.0 - uniqueness.duplicate_rate);
        }

        if let Some(ref completeness) = self.completeness {
            if completeness.overall_completeness < thresholds.completeness_rate_min {
                self.issues.push(format!(
                    "Completeness {} < {} (threshold)",
                    completeness.overall_completeness, thresholds.completeness_rate_min
                ));
            }
            scores.push(completeness.overall_completeness);
        }

        if let Some(ref format) = self.format {
            if format.consistency_score < thresholds.format_consistency_min {
                self.issues.push(format!(
                    "Format consistency {} < {} (threshold)",
                    format.consistency_score, thresholds.format_consistency_min
                ));
            }
            scores.push(format.consistency_score);
        }

        if let Some(ref consistency) = self.consistency {
            // Use format consistency threshold for cross-field as they're related
            if consistency.pass_rate < thresholds.format_consistency_min {
                self.issues.push(format!(
                    "Cross-field consistency {} < {} (threshold)",
                    consistency.pass_rate, thresholds.format_consistency_min
                ));
            }
            scores.push(consistency.pass_rate);
        }

        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        // Sync failures with issues
        self.failures = self.issues.clone();
        self.passes = self.issues.is_empty();
    }
}

impl Default for QualityEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
