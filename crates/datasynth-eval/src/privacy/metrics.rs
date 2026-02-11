//! Privacy metrics: NIST SP 800-226 alignment and SynQP quality-privacy matrix.
//!
//! Provides structured self-assessment against NIST standards for synthetic data
//! and a quality-privacy evaluation quadrant (SynQP) for high-level classification.

use serde::{Deserialize, Serialize};

/// NIST SP 800-226 alignment self-assessment report.
///
/// Maps DataSynth's privacy controls to NIST criteria for evaluating
/// de-identification and synthetic data methodologies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistAlignmentReport {
    /// Whether differential privacy is applied.
    pub differential_privacy_applied: bool,
    /// Epsilon value used (if applicable).
    pub epsilon: Option<f64>,
    /// Delta value used (if applicable).
    pub delta: Option<f64>,
    /// The composition method used.
    pub composition_method: Option<String>,
    /// Whether k-anonymity is enforced.
    pub k_anonymity_enforced: bool,
    /// The k-anonymity level achieved.
    pub k_anonymity_level: Option<usize>,
    /// Whether membership inference was tested.
    pub membership_inference_tested: bool,
    /// MIA AUC-ROC result (if tested).
    pub mia_auc_roc: Option<f64>,
    /// Whether linkage attack was tested.
    pub linkage_attack_tested: bool,
    /// Re-identification rate (if tested).
    pub re_identification_rate: Option<f64>,
    /// Overall NIST alignment score (0.0-1.0).
    /// Based on how many criteria are met.
    pub alignment_score: f64,
    /// Individual criterion assessments.
    pub criteria: Vec<NistCriterion>,
    /// Whether the overall assessment passes.
    pub passes: bool,
}

/// A single NIST criterion assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistCriterion {
    /// Criterion identifier (e.g., "DP-1", "KA-1", "MIA-1").
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this criterion is met.
    pub met: bool,
    /// Evidence or rationale.
    pub evidence: String,
}

impl NistAlignmentReport {
    /// Build a NIST alignment report from privacy evaluation results.
    pub fn build(
        dp_applied: bool,
        epsilon: Option<f64>,
        delta: Option<f64>,
        composition_method: Option<String>,
        k_anonymity_enforced: bool,
        k_anonymity_level: Option<usize>,
        mia_auc_roc: Option<f64>,
        re_identification_rate: Option<f64>,
    ) -> Self {
        let mut criteria = Vec::new();

        // DP criteria
        criteria.push(NistCriterion {
            id: "DP-1".to_string(),
            description: "Differential privacy mechanism applied".to_string(),
            met: dp_applied,
            evidence: if dp_applied {
                format!(
                    "DP applied with epsilon={}, delta={}, method={}",
                    epsilon.map_or("N/A".to_string(), |e| format!("{:.4}", e)),
                    delta.map_or("N/A".to_string(), |d| format!("{:.2e}", d)),
                    composition_method.as_deref().unwrap_or("naive"),
                )
            } else {
                "No differential privacy mechanism applied".to_string()
            },
        });

        criteria.push(NistCriterion {
            id: "DP-2".to_string(),
            description: "Epsilon within reasonable bounds (< 10.0)".to_string(),
            met: epsilon.is_some_and(|e| e < 10.0),
            evidence: epsilon.map_or("No epsilon specified".to_string(), |e| {
                format!("Epsilon = {:.4}", e)
            }),
        });

        // K-anonymity criteria
        criteria.push(NistCriterion {
            id: "KA-1".to_string(),
            description: "K-anonymity enforced with k >= 5".to_string(),
            met: k_anonymity_enforced && k_anonymity_level.is_some_and(|k| k >= 5),
            evidence: if k_anonymity_enforced {
                format!(
                    "K-anonymity enforced, k = {}",
                    k_anonymity_level.map_or("unknown".to_string(), |k| k.to_string())
                )
            } else {
                "K-anonymity not enforced".to_string()
            },
        });

        // MIA criteria
        let mia_tested = mia_auc_roc.is_some();
        criteria.push(NistCriterion {
            id: "MIA-1".to_string(),
            description: "Membership inference attack tested".to_string(),
            met: mia_tested,
            evidence: if mia_tested {
                format!("MIA AUC-ROC = {:.4}", mia_auc_roc.unwrap_or(0.0))
            } else {
                "MIA not tested".to_string()
            },
        });

        criteria.push(NistCriterion {
            id: "MIA-2".to_string(),
            description: "MIA AUC-ROC < 0.6 (near-random)".to_string(),
            met: mia_auc_roc.is_some_and(|auc| auc < 0.6),
            evidence: mia_auc_roc.map_or("MIA not tested".to_string(), |auc| {
                format!("AUC-ROC = {:.4}", auc)
            }),
        });

        // Linkage criteria
        let linkage_tested = re_identification_rate.is_some();
        criteria.push(NistCriterion {
            id: "LA-1".to_string(),
            description: "Linkage attack tested".to_string(),
            met: linkage_tested,
            evidence: if linkage_tested {
                format!(
                    "Re-identification rate = {:.4}",
                    re_identification_rate.unwrap_or(0.0)
                )
            } else {
                "Linkage attack not tested".to_string()
            },
        });

        criteria.push(NistCriterion {
            id: "LA-2".to_string(),
            description: "Re-identification rate < 5%".to_string(),
            met: re_identification_rate.is_some_and(|r| r < 0.05),
            evidence: re_identification_rate.map_or("Not tested".to_string(), |r| {
                format!("Re-identification rate = {:.2}%", r * 100.0)
            }),
        });

        let met_count = criteria.iter().filter(|c| c.met).count();
        let alignment_score = if criteria.is_empty() {
            0.0
        } else {
            met_count as f64 / criteria.len() as f64
        };

        // Pass if at least 5 out of 7 criteria are met
        let passes = met_count >= 5;

        Self {
            differential_privacy_applied: dp_applied,
            epsilon,
            delta,
            composition_method,
            k_anonymity_enforced,
            k_anonymity_level,
            membership_inference_tested: mia_tested,
            mia_auc_roc,
            linkage_attack_tested: linkage_tested,
            re_identification_rate,
            alignment_score,
            criteria,
            passes,
        }
    }
}

/// Quality-Privacy evaluation quadrant (SynQP).
///
/// Classifies synthetic data output into one of four quadrants based on
/// how well it balances data quality (utility) with privacy protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynQPQuadrant {
    /// High quality, high privacy — the ideal outcome.
    HighQHighP,
    /// High quality, low privacy — useful but risky.
    HighQLowP,
    /// Low quality, high privacy — safe but less useful.
    LowQHighP,
    /// Low quality, low privacy — worst outcome.
    LowQLowP,
}

impl std::fmt::Display for SynQPQuadrant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighQHighP => write!(f, "High Quality / High Privacy (Ideal)"),
            Self::HighQLowP => write!(f, "High Quality / Low Privacy (Risky)"),
            Self::LowQHighP => write!(f, "Low Quality / High Privacy (Conservative)"),
            Self::LowQLowP => write!(f, "Low Quality / Low Privacy (Poor)"),
        }
    }
}

/// SynQP matrix evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynQPMatrix {
    /// Quality score (0.0 - 1.0). Derived from data evaluation metrics.
    pub quality_score: f64,
    /// Privacy score (0.0 - 1.0). Derived from privacy evaluation metrics.
    pub privacy_score: f64,
    /// The quadrant classification.
    pub quadrant: SynQPQuadrant,
    /// Quality threshold for high/low classification.
    pub quality_threshold: f64,
    /// Privacy threshold for high/low classification.
    pub privacy_threshold: f64,
}

impl SynQPMatrix {
    /// Compute the SynQP matrix from quality and privacy scores.
    ///
    /// # Arguments
    /// * `quality_score` - Overall data quality score (0.0-1.0, higher = better quality)
    /// * `privacy_score` - Overall privacy score (0.0-1.0, higher = better privacy)
    /// * `quality_threshold` - Threshold for high vs low quality (default: 0.7)
    /// * `privacy_threshold` - Threshold for high vs low privacy (default: 0.7)
    pub fn evaluate(
        quality_score: f64,
        privacy_score: f64,
        quality_threshold: f64,
        privacy_threshold: f64,
    ) -> Self {
        let quadrant = match (
            quality_score >= quality_threshold,
            privacy_score >= privacy_threshold,
        ) {
            (true, true) => SynQPQuadrant::HighQHighP,
            (true, false) => SynQPQuadrant::HighQLowP,
            (false, true) => SynQPQuadrant::LowQHighP,
            (false, false) => SynQPQuadrant::LowQLowP,
        };

        Self {
            quality_score,
            privacy_score,
            quadrant,
            quality_threshold,
            privacy_threshold,
        }
    }

    /// Evaluate with default thresholds (0.7 for both).
    pub fn evaluate_default(quality_score: f64, privacy_score: f64) -> Self {
        Self::evaluate(quality_score, privacy_score, 0.7, 0.7)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_report_all_criteria_met() {
        let report = NistAlignmentReport::build(
            true,
            Some(1.0),
            Some(1e-5),
            Some("renyi_dp".to_string()),
            true,
            Some(10),
            Some(0.52),
            Some(0.01),
        );

        assert!(report.passes);
        assert!(report.alignment_score > 0.9);
        assert_eq!(report.criteria.len(), 7);
        assert!(report.criteria.iter().all(|c| c.met));
    }

    #[test]
    fn test_nist_report_no_privacy() {
        let report = NistAlignmentReport::build(
            false, // no DP
            None, None, None, false, // no k-anonymity
            None, None, // no MIA
            None, // no linkage
        );

        assert!(!report.passes);
        assert_eq!(report.alignment_score, 0.0);
        assert!(report.criteria.iter().all(|c| !c.met));
    }

    #[test]
    fn test_nist_report_partial() {
        let report = NistAlignmentReport::build(
            true,
            Some(5.0),
            Some(1e-5),
            Some("naive".to_string()),
            true,
            Some(3),    // k=3, which is < 5 threshold
            Some(0.55), // passes MIA
            Some(0.03), // passes linkage
        );

        // DP-1: met, DP-2: met (5<10), KA-1: NOT met (3<5),
        // MIA-1: met, MIA-2: met (0.55<0.6), LA-1: met, LA-2: met (0.03<0.05)
        let met = report.criteria.iter().filter(|c| c.met).count();
        assert_eq!(met, 6); // 6 out of 7
        assert!(report.passes);
    }

    #[test]
    fn test_nist_report_serde() {
        let report = NistAlignmentReport::build(
            true,
            Some(1.0),
            Some(1e-5),
            None,
            true,
            Some(10),
            Some(0.5),
            Some(0.01),
        );
        let json = serde_json::to_string(&report).unwrap();
        let parsed: NistAlignmentReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.criteria.len(), 7);
        assert!(parsed.passes);
    }

    #[test]
    fn test_synqp_high_quality_high_privacy() {
        let matrix = SynQPMatrix::evaluate_default(0.85, 0.90);
        assert_eq!(matrix.quadrant, SynQPQuadrant::HighQHighP);
    }

    #[test]
    fn test_synqp_high_quality_low_privacy() {
        let matrix = SynQPMatrix::evaluate_default(0.85, 0.40);
        assert_eq!(matrix.quadrant, SynQPQuadrant::HighQLowP);
    }

    #[test]
    fn test_synqp_low_quality_high_privacy() {
        let matrix = SynQPMatrix::evaluate_default(0.30, 0.90);
        assert_eq!(matrix.quadrant, SynQPQuadrant::LowQHighP);
    }

    #[test]
    fn test_synqp_low_quality_low_privacy() {
        let matrix = SynQPMatrix::evaluate_default(0.30, 0.40);
        assert_eq!(matrix.quadrant, SynQPQuadrant::LowQLowP);
    }

    #[test]
    fn test_synqp_custom_thresholds() {
        // With low thresholds, everything is "high"
        let matrix = SynQPMatrix::evaluate(0.5, 0.5, 0.3, 0.3);
        assert_eq!(matrix.quadrant, SynQPQuadrant::HighQHighP);
    }

    #[test]
    fn test_synqp_display() {
        assert_eq!(
            format!("{}", SynQPQuadrant::HighQHighP),
            "High Quality / High Privacy (Ideal)"
        );
        assert_eq!(
            format!("{}", SynQPQuadrant::LowQLowP),
            "Low Quality / Low Privacy (Poor)"
        );
    }

    #[test]
    fn test_synqp_serde() {
        let matrix = SynQPMatrix::evaluate_default(0.8, 0.9);
        let json = serde_json::to_string(&matrix).unwrap();
        let parsed: SynQPMatrix = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.quadrant, SynQPQuadrant::HighQHighP);
        assert!((parsed.quality_score - 0.8).abs() < 1e-10);
    }
}
