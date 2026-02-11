//! Privacy evaluation module.
//!
//! Provides empirical privacy assessments for synthetic data including:
//! - **Membership Inference Attack (MIA)**: Distance-based classifier to detect training data leakage
//! - **Linkage Attack**: Quasi-identifier-based re-identification risk assessment
//! - **NIST SP 800-226 Alignment**: Self-assessment against NIST criteria for de-identification
//! - **SynQP Matrix**: Quality-privacy quadrant evaluation

pub mod linkage;
pub mod membership_inference;
pub mod metrics;

pub use linkage::{LinkageAttack, LinkageConfig, LinkageResults};
pub use membership_inference::{MembershipInferenceAttack, MiaConfig, MiaResults};
pub use metrics::{NistAlignmentReport, NistCriterion, SynQPMatrix, SynQPQuadrant};

use serde::{Deserialize, Serialize};

/// Combined privacy evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyEvaluation {
    /// Membership inference attack results (if evaluated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub membership_inference: Option<MiaResults>,
    /// Linkage attack results (if evaluated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linkage: Option<LinkageResults>,
    /// NIST alignment report (if generated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nist_alignment: Option<NistAlignmentReport>,
    /// SynQP quality-privacy matrix (if computed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synqp: Option<SynQPMatrix>,
    /// Overall privacy evaluation passes.
    pub passes: bool,
    /// Failures encountered.
    pub failures: Vec<String>,
}

impl Default for PrivacyEvaluation {
    fn default() -> Self {
        Self {
            membership_inference: None,
            linkage: None,
            nist_alignment: None,
            synqp: None,
            passes: true,
            failures: Vec::new(),
        }
    }
}

impl PrivacyEvaluation {
    /// Create a new empty privacy evaluation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the overall pass/fail status based on sub-evaluations.
    pub fn update_status(&mut self) {
        self.failures.clear();

        if let Some(ref mia) = self.membership_inference {
            if !mia.passes {
                self.failures.push(format!(
                    "MIA: AUC-ROC {:.4} exceeds threshold {:.4}",
                    mia.auc_roc, mia.auc_threshold
                ));
            }
        }

        if let Some(ref linkage) = self.linkage {
            if !linkage.passes {
                self.failures.push(format!(
                    "Linkage: re-identification rate {:.2}%, k-anonymity {}",
                    linkage.re_identification_rate * 100.0,
                    linkage.k_anonymity_achieved,
                ));
            }
        }

        if let Some(ref nist) = self.nist_alignment {
            if !nist.passes {
                self.failures.push(format!(
                    "NIST alignment: score {:.0}% (requires >= 71%)",
                    nist.alignment_score * 100.0
                ));
            }
        }

        self.passes = self.failures.is_empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_evaluation_default() {
        let eval = PrivacyEvaluation::default();
        assert!(eval.passes);
        assert!(eval.failures.is_empty());
        assert!(eval.membership_inference.is_none());
        assert!(eval.linkage.is_none());
    }

    #[test]
    fn test_privacy_evaluation_with_failures() {
        let mut eval = PrivacyEvaluation::new();

        eval.membership_inference = Some(MiaResults {
            auc_roc: 0.85,
            accuracy: 0.80,
            precision: 0.78,
            recall: 0.82,
            passes: false,
            n_members: 100,
            n_non_members: 100,
            auc_threshold: 0.6,
        });

        eval.update_status();
        assert!(!eval.passes);
        assert_eq!(eval.failures.len(), 1);
        assert!(eval.failures[0].contains("MIA"));
    }

    #[test]
    fn test_privacy_evaluation_all_pass() {
        let mut eval = PrivacyEvaluation::new();

        eval.membership_inference = Some(MiaResults {
            auc_roc: 0.52,
            accuracy: 0.50,
            precision: 0.50,
            recall: 0.50,
            passes: true,
            n_members: 100,
            n_non_members: 100,
            auc_threshold: 0.6,
        });

        eval.linkage = Some(LinkageResults {
            re_identification_rate: 0.01,
            k_anonymity_achieved: 10,
            unique_qi_combos_original: 50,
            unique_qi_combos_synthetic: 48,
            overlapping_combos: 30,
            uniquely_linked: 1,
            total_synthetic: 100,
            passes: true,
        });

        eval.update_status();
        assert!(eval.passes);
        assert!(eval.failures.is_empty());
    }

    #[test]
    fn test_privacy_evaluation_serde() {
        let eval = PrivacyEvaluation::default();
        let json = serde_json::to_string(&eval).unwrap();
        let parsed: PrivacyEvaluation = serde_json::from_str(&json).unwrap();
        assert!(parsed.passes);
    }
}
