//! Analytical procedure models per ISA 520.
//!
//! Analytical procedures are evaluations of financial information through
//! analysis of plausible relationships among both financial and non-financial data.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Phase of the audit in which the analytical procedure is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalPhase {
    /// Planning phase — used to understand the entity and identify risk areas
    Planning,
    /// Substantive phase — used as a substantive procedure to detect material misstatement
    #[default]
    Substantive,
    /// Final review phase — used as an overall review at completion
    FinalReview,
}

/// Method used to perform the analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalMethod {
    /// Analysis of changes over time
    #[default]
    TrendAnalysis,
    /// Computation of key financial ratios
    RatioAnalysis,
    /// Assessment of whether recorded amounts are reasonable
    ReasonablenessTest,
    /// Statistical regression to develop an expectation
    Regression,
    /// Comparison against industry data or prior periods
    Comparison,
}

/// Conclusion reached after performing the analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalConclusion {
    /// Result is consistent with auditor's expectation — no further work required
    #[default]
    Consistent,
    /// Variance exists but has been satisfactorily explained
    ExplainedVariance,
    /// Variance requires further investigation before a conclusion can be drawn
    FurtherInvestigation,
    /// Variance may indicate a possible misstatement
    PossibleMisstatement,
}

/// Lifecycle status of the analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalStatus {
    /// Procedure has been planned but not yet performed
    Planned,
    /// Procedure has been performed and variance computed
    #[default]
    Performed,
    /// Procedure has been completed and a conclusion recorded
    Concluded,
}

/// Result of a single analytical procedure per ISA 520.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalProcedureResult {
    /// Unique result ID
    pub result_id: Uuid,
    /// Human-readable reference (format: "AP-{first 8 hex chars of result_id}")
    pub result_ref: String,
    /// Engagement this result belongs to
    pub engagement_id: Uuid,
    /// Workpaper that documents this procedure (optional)
    pub workpaper_id: Option<Uuid>,

    // === Procedure Design ===
    /// Phase in which this procedure is applied
    pub procedure_phase: AnalyticalPhase,
    /// Account or area under analysis (e.g., "Revenue", "Accounts Receivable")
    pub account_or_area: String,
    /// Specific account ID if applicable
    pub account_id: Option<String>,
    /// Analytical method used
    pub analytical_method: AnalyticalMethod,

    // === Expectation ===
    /// Auditor's expectation of the recorded amount
    pub expectation: Decimal,
    /// Basis on which the expectation was developed
    pub expectation_basis: String,

    // === Threshold ===
    /// Threshold of acceptable variance before investigation is required
    pub threshold: Decimal,
    /// Basis for setting the threshold (e.g., "5% of expectation" or "materiality")
    pub threshold_basis: String,

    // === Actual & Computed Fields ===
    /// Actual recorded amount
    pub actual_value: Decimal,
    /// Variance (actual − expectation), auto-computed by constructor
    pub variance: Decimal,
    /// Variance as a percentage of expectation, auto-computed
    pub variance_percentage: f64,
    /// Whether the variance exceeds the threshold, auto-computed
    pub requires_investigation: bool,

    // === Investigation & Explanation ===
    /// Explanation provided by management or the auditor for the variance
    pub explanation: Option<String>,
    /// Whether the explanation has been corroborated by additional evidence
    pub explanation_corroborated: Option<bool>,
    /// Description of corroboration evidence if applicable
    pub corroboration_evidence: Option<String>,

    // === Conclusion ===
    /// Conclusion reached after evaluation
    pub conclusion: Option<AnalyticalConclusion>,
    /// Current lifecycle status
    pub status: AnalyticalStatus,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AnalyticalProcedureResult {
    /// Create a new analytical procedure result.
    ///
    /// Automatically computes `variance`, `variance_percentage`, and
    /// `requires_investigation` from the provided inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engagement_id: Uuid,
        account_or_area: impl Into<String>,
        analytical_method: AnalyticalMethod,
        expectation: Decimal,
        expectation_basis: impl Into<String>,
        threshold: Decimal,
        threshold_basis: impl Into<String>,
        actual_value: Decimal,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let result_ref = format!("AP-{}", &id.simple().to_string()[..8]);

        let variance = actual_value - expectation;
        let variance_percentage = if expectation.is_zero() {
            0.0
        } else {
            (variance / expectation * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        };
        let requires_investigation = variance.abs() > threshold;

        Self {
            result_id: id,
            result_ref,
            engagement_id,
            workpaper_id: None,
            procedure_phase: AnalyticalPhase::Substantive,
            account_or_area: account_or_area.into(),
            account_id: None,
            analytical_method,
            expectation,
            expectation_basis: expectation_basis.into(),
            threshold,
            threshold_basis: threshold_basis.into(),
            actual_value,
            variance,
            variance_percentage,
            requires_investigation,
            explanation: None,
            explanation_corroborated: None,
            corroboration_evidence: None,
            conclusion: None,
            status: AnalyticalStatus::Performed,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_result(expectation: Decimal, actual: Decimal, threshold: Decimal) -> AnalyticalProcedureResult {
        AnalyticalProcedureResult::new(
            Uuid::new_v4(),
            "Revenue",
            AnalyticalMethod::TrendAnalysis,
            expectation,
            "Prior year adjusted for growth",
            threshold,
            "5% of expectation",
            actual,
        )
    }

    #[test]
    fn test_new_analytical_procedure() {
        let result = make_result(dec!(1_000_000), dec!(1_050_000), dec!(50_000));
        assert_eq!(result.account_or_area, "Revenue");
        assert_eq!(result.analytical_method, AnalyticalMethod::TrendAnalysis);
        assert_eq!(result.procedure_phase, AnalyticalPhase::Substantive);
        assert_eq!(result.status, AnalyticalStatus::Performed);
        assert!(result.result_ref.starts_with("AP-"));
        assert_eq!(result.result_ref.len(), 11); // "AP-" + 8 hex chars
    }

    #[test]
    fn test_variance_computation() {
        let result = make_result(dec!(1_000_000), dec!(1_050_000), dec!(100_000));
        assert_eq!(result.variance, dec!(50_000));
        // 50,000 / 1,000,000 * 100 = 5.0
        let pct = result.variance_percentage;
        assert!((pct - 5.0).abs() < 0.0001, "expected ~5.0, got {pct}");
    }

    #[test]
    fn test_variance_zero_expectation() {
        let result = make_result(dec!(0), dec!(500), dec!(100));
        // Should not panic; variance_percentage defaults to 0.0
        assert_eq!(result.variance_percentage, 0.0);
        assert_eq!(result.variance, dec!(500));
    }

    #[test]
    fn test_requires_investigation_true() {
        // variance = 50,000; threshold = 30,000 → requires investigation
        let result = make_result(dec!(1_000_000), dec!(1_050_000), dec!(30_000));
        assert!(result.requires_investigation);
    }

    #[test]
    fn test_requires_investigation_false() {
        // variance = 10,000; threshold = 50,000 → does NOT require investigation
        let result = make_result(dec!(1_000_000), dec!(1_010_000), dec!(50_000));
        assert!(!result.requires_investigation);
    }

    #[test]
    fn test_analytical_phase_serde() {
        let phases = [
            AnalyticalPhase::Planning,
            AnalyticalPhase::Substantive,
            AnalyticalPhase::FinalReview,
        ];
        for phase in phases {
            let json = serde_json::to_string(&phase).unwrap();
            let roundtripped: AnalyticalPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, roundtripped);
        }
        assert_eq!(serde_json::to_string(&AnalyticalPhase::Planning).unwrap(), "\"planning\"");
        assert_eq!(serde_json::to_string(&AnalyticalPhase::FinalReview).unwrap(), "\"final_review\"");
    }

    #[test]
    fn test_analytical_method_serde() {
        let methods = [
            AnalyticalMethod::TrendAnalysis,
            AnalyticalMethod::RatioAnalysis,
            AnalyticalMethod::ReasonablenessTest,
            AnalyticalMethod::Regression,
            AnalyticalMethod::Comparison,
        ];
        for method in methods {
            let json = serde_json::to_string(&method).unwrap();
            let roundtripped: AnalyticalMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(method, roundtripped);
        }
        assert_eq!(serde_json::to_string(&AnalyticalMethod::TrendAnalysis).unwrap(), "\"trend_analysis\"");
        assert_eq!(serde_json::to_string(&AnalyticalMethod::ReasonablenessTest).unwrap(), "\"reasonableness_test\"");
    }

    #[test]
    fn test_analytical_conclusion_serde() {
        let conclusions = [
            AnalyticalConclusion::Consistent,
            AnalyticalConclusion::ExplainedVariance,
            AnalyticalConclusion::FurtherInvestigation,
            AnalyticalConclusion::PossibleMisstatement,
        ];
        for conclusion in conclusions {
            let json = serde_json::to_string(&conclusion).unwrap();
            let roundtripped: AnalyticalConclusion = serde_json::from_str(&json).unwrap();
            assert_eq!(conclusion, roundtripped);
        }
        assert_eq!(
            serde_json::to_string(&AnalyticalConclusion::ExplainedVariance).unwrap(),
            "\"explained_variance\""
        );
        assert_eq!(
            serde_json::to_string(&AnalyticalConclusion::PossibleMisstatement).unwrap(),
            "\"possible_misstatement\""
        );
    }
}
