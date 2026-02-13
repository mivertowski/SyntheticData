//! Audit evaluator.
//!
//! Validates audit data coherence including evidence-to-finding mapping,
//! risk-to-procedure mapping, workpaper completeness, and materiality hierarchy.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for audit evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditThresholds {
    /// Minimum evidence-to-finding mapping rate.
    pub min_evidence_mapping: f64,
    /// Minimum risk-to-procedure mapping rate.
    pub min_risk_procedure_mapping: f64,
    /// Minimum workpaper completeness rate.
    pub min_workpaper_completeness: f64,
}

impl Default for AuditThresholds {
    fn default() -> Self {
        Self {
            min_evidence_mapping: 0.90,
            min_risk_procedure_mapping: 0.90,
            min_workpaper_completeness: 0.85,
        }
    }
}

/// Audit finding data.
#[derive(Debug, Clone)]
pub struct AuditFindingData {
    /// Finding identifier.
    pub finding_id: String,
    /// Whether this finding has supporting evidence.
    pub has_evidence: bool,
    /// Number of evidence items.
    pub evidence_count: usize,
}

/// Audit risk data.
#[derive(Debug, Clone)]
pub struct AuditRiskData {
    /// Risk identifier.
    pub risk_id: String,
    /// Whether responsive audit procedures exist.
    pub has_procedures: bool,
    /// Number of responsive procedures.
    pub procedure_count: usize,
}

/// Workpaper data.
#[derive(Debug, Clone)]
pub struct WorkpaperData {
    /// Workpaper identifier.
    pub workpaper_id: String,
    /// Whether the workpaper has a conclusion.
    pub has_conclusion: bool,
    /// Whether the workpaper has references.
    pub has_references: bool,
    /// Whether the workpaper has a preparer.
    pub has_preparer: bool,
    /// Whether the workpaper has been reviewed.
    pub has_reviewer: bool,
}

/// Materiality data.
#[derive(Debug, Clone)]
pub struct MaterialityData {
    /// Overall materiality.
    pub overall_materiality: f64,
    /// Performance materiality.
    pub performance_materiality: f64,
    /// Clearly trivial threshold.
    pub clearly_trivial: f64,
}

/// Results of audit evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvaluation {
    /// Evidence-to-finding rate: fraction of findings with evidence.
    pub evidence_to_finding_rate: f64,
    /// Risk-to-procedure rate: fraction of risks with procedures.
    pub risk_to_procedure_rate: f64,
    /// Workpaper completeness: fraction with conclusion + references.
    pub workpaper_completeness: f64,
    /// Whether materiality hierarchy is valid (overall > performance > trivial).
    pub materiality_hierarchy_valid: bool,
    /// Total findings evaluated.
    pub total_findings: usize,
    /// Total risks evaluated.
    pub total_risks: usize,
    /// Total workpapers evaluated.
    pub total_workpapers: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for audit coherence.
pub struct AuditEvaluator {
    thresholds: AuditThresholds,
}

impl AuditEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: AuditThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: AuditThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate audit data.
    pub fn evaluate(
        &self,
        findings: &[AuditFindingData],
        risks: &[AuditRiskData],
        workpapers: &[WorkpaperData],
        materiality: &Option<MaterialityData>,
    ) -> EvalResult<AuditEvaluation> {
        let mut issues = Vec::new();

        // 1. Evidence-to-finding mapping
        let findings_with_evidence = findings.iter().filter(|f| f.has_evidence).count();
        let evidence_to_finding_rate = if findings.is_empty() {
            1.0
        } else {
            findings_with_evidence as f64 / findings.len() as f64
        };

        // 2. Risk-to-procedure mapping
        let risks_with_procedures = risks.iter().filter(|r| r.has_procedures).count();
        let risk_to_procedure_rate = if risks.is_empty() {
            1.0
        } else {
            risks_with_procedures as f64 / risks.len() as f64
        };

        // 3. Workpaper completeness (has conclusion AND references)
        let complete_workpapers = workpapers
            .iter()
            .filter(|w| w.has_conclusion && w.has_references)
            .count();
        let workpaper_completeness = if workpapers.is_empty() {
            1.0
        } else {
            complete_workpapers as f64 / workpapers.len() as f64
        };

        // 4. Materiality hierarchy
        let materiality_hierarchy_valid = if let Some(ref mat) = materiality {
            mat.overall_materiality > mat.performance_materiality
                && mat.performance_materiality > mat.clearly_trivial
                && mat.clearly_trivial >= 0.0
        } else {
            true // Not provided = not checked
        };

        // Check thresholds
        if evidence_to_finding_rate < self.thresholds.min_evidence_mapping {
            issues.push(format!(
                "Evidence-to-finding rate {:.3} < {:.3}",
                evidence_to_finding_rate, self.thresholds.min_evidence_mapping
            ));
        }
        if risk_to_procedure_rate < self.thresholds.min_risk_procedure_mapping {
            issues.push(format!(
                "Risk-to-procedure rate {:.3} < {:.3}",
                risk_to_procedure_rate, self.thresholds.min_risk_procedure_mapping
            ));
        }
        if workpaper_completeness < self.thresholds.min_workpaper_completeness {
            issues.push(format!(
                "Workpaper completeness {:.3} < {:.3}",
                workpaper_completeness, self.thresholds.min_workpaper_completeness
            ));
        }
        if !materiality_hierarchy_valid {
            issues.push(
                "Materiality hierarchy invalid: expected overall > performance > trivial"
                    .to_string(),
            );
        }

        let passes = issues.is_empty();

        Ok(AuditEvaluation {
            evidence_to_finding_rate,
            risk_to_procedure_rate,
            workpaper_completeness,
            materiality_hierarchy_valid,
            total_findings: findings.len(),
            total_risks: risks.len(),
            total_workpapers: workpapers.len(),
            passes,
            issues,
        })
    }
}

impl Default for AuditEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_audit() {
        let evaluator = AuditEvaluator::new();
        let findings = vec![AuditFindingData {
            finding_id: "F001".to_string(),
            has_evidence: true,
            evidence_count: 3,
        }];
        let risks = vec![AuditRiskData {
            risk_id: "R001".to_string(),
            has_procedures: true,
            procedure_count: 2,
        }];
        let workpapers = vec![WorkpaperData {
            workpaper_id: "WP001".to_string(),
            has_conclusion: true,
            has_references: true,
            has_preparer: true,
            has_reviewer: true,
        }];
        let materiality = Some(MaterialityData {
            overall_materiality: 100_000.0,
            performance_materiality: 75_000.0,
            clearly_trivial: 5_000.0,
        });

        let result = evaluator
            .evaluate(&findings, &risks, &workpapers, &materiality)
            .unwrap();
        assert!(result.passes);
        assert!(result.materiality_hierarchy_valid);
    }

    #[test]
    fn test_missing_evidence() {
        let evaluator = AuditEvaluator::new();
        let findings = vec![
            AuditFindingData {
                finding_id: "F001".to_string(),
                has_evidence: false,
                evidence_count: 0,
            },
            AuditFindingData {
                finding_id: "F002".to_string(),
                has_evidence: false,
                evidence_count: 0,
            },
        ];

        let result = evaluator.evaluate(&findings, &[], &[], &None).unwrap();
        assert!(!result.passes);
        assert_eq!(result.evidence_to_finding_rate, 0.0);
    }

    #[test]
    fn test_invalid_materiality() {
        let evaluator = AuditEvaluator::new();
        let materiality = Some(MaterialityData {
            overall_materiality: 50_000.0,
            performance_materiality: 100_000.0, // Higher than overall!
            clearly_trivial: 5_000.0,
        });

        let result = evaluator.evaluate(&[], &[], &[], &materiality).unwrap();
        assert!(!result.materiality_hierarchy_valid);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = AuditEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[], &None).unwrap();
        assert!(result.passes);
    }
}
