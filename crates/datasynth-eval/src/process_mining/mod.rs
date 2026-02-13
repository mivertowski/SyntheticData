//! OCEL 2.0 process mining evaluation module.
//!
//! Validates event sequence validity, object lifecycle completeness,
//! and process variant distribution.

pub mod event_sequence;
pub mod variant_analysis;

pub use event_sequence::{EventSequenceAnalysis, EventSequenceAnalyzer, ProcessEventData};
pub use variant_analysis::{VariantAnalysis, VariantAnalyzer, VariantData};

use serde::{Deserialize, Serialize};

/// Combined process mining evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMiningEvaluation {
    /// Event sequence analysis.
    pub event_sequence: Option<EventSequenceAnalysis>,
    /// Variant analysis.
    pub variants: Option<VariantAnalysis>,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

impl ProcessMiningEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            event_sequence: None,
            variants: None,
            passes: true,
            issues: Vec::new(),
        }
    }

    /// Check thresholds and update pass status.
    pub fn check_thresholds(&mut self) {
        self.issues.clear();
        if let Some(ref es) = self.event_sequence {
            if !es.passes {
                self.issues.extend(es.issues.clone());
            }
        }
        if let Some(ref va) = self.variants {
            if !va.passes {
                self.issues.extend(va.issues.clone());
            }
        }
        self.passes = self.issues.is_empty();
    }
}

impl Default for ProcessMiningEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
