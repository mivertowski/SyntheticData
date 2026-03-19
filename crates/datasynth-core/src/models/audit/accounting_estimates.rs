//! Accounting estimates models — ISA 540.
//!
//! Accounting estimates are approximations of monetary amounts in the
//! absence of a precise means of measurement. ISA 540 (Revised 2019)
//! requires auditors to evaluate the reasonableness of management's
//! estimates and identify risks of material misstatement.
//!
//! Key estimate types covered:
//! - Deferred tax provisions (IAS 12 / ASC 740)
//! - Expected credit losses (IFRS 9 / ASC 326)
//! - Pension obligations (IAS 19 / ASC 715)
//! - Fair value measurements (IFRS 13 / ASC 820)
//! - Impairment tests (IAS 36 / ASC 350–360)
//! - Provisions for liabilities (IAS 37 / ASC 450)
//! - Share-based payments (IFRS 2 / ASC 718)
//! - Depreciation useful-life assessments

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Category of accounting estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateType {
    /// Deferred tax asset/liability recognition (IAS 12 / ASC 740).
    DeferredTaxProvision,
    /// Expected credit loss allowance (IFRS 9 / ASC 326).
    ExpectedCreditLoss,
    /// Defined benefit pension obligation (IAS 19 / ASC 715).
    PensionObligation,
    /// Level 2/3 fair value measurement (IFRS 13 / ASC 820).
    FairValueMeasurement,
    /// Goodwill or long-lived asset impairment test (IAS 36 / ASC 350–360).
    ImpairmentTest,
    /// Provision for legal/environmental/warranty liabilities (IAS 37 / ASC 450).
    ProvisionForLiabilities,
    /// Share-based payment expense (IFRS 2 / ASC 718).
    ShareBasedPayment,
    /// Useful-life revision for property, plant & equipment (IAS 16 / ASC 360).
    DepreciationUsefulLife,
}

impl std::fmt::Display for EstimateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DeferredTaxProvision => "Deferred Tax Provision",
            Self::ExpectedCreditLoss => "Expected Credit Loss",
            Self::PensionObligation => "Pension Obligation",
            Self::FairValueMeasurement => "Fair Value Measurement",
            Self::ImpairmentTest => "Impairment Test",
            Self::ProvisionForLiabilities => "Provision for Liabilities",
            Self::ShareBasedPayment => "Share-Based Payment",
            Self::DepreciationUsefulLife => "Depreciation Useful Life",
        };
        write!(f, "{s}")
    }
}

/// Degree of uncertainty inherent in an accounting estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyLevel {
    /// Outcome is predictable with reasonable confidence.
    Low,
    /// Outcome involves some uncertainty; range of outcomes is limited.
    Medium,
    /// Outcome is highly sensitive to assumptions; wide range of outcomes possible.
    High,
}

/// Complexity of the estimation process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateComplexity {
    /// Standard actuarial or formulaic calculation; limited judgment required.
    Simple,
    /// Multiple interdependent inputs; moderate management judgment required.
    Moderate,
    /// Sophisticated models, specialist input, or significant management judgment.
    Complex,
}

/// Auditor's assessment of an individual key assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionAssessment {
    /// Assumption falls within an acceptable range given the entity's circumstances.
    Reasonable,
    /// Assumption is at the favourable end of an acceptable range.
    Optimistic,
    /// Assumption is outside or at the extreme end of an acceptable range.
    Aggressive,
}

/// Degree of subjectivity in a key assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectivityLevel {
    /// Observable market data or contractual terms determine the assumption.
    Low,
    /// Assumptions are internally derived but corroborated by external evidence.
    Medium,
    /// Assumptions rely primarily on management intent or unobservable inputs.
    High,
}

// ---------------------------------------------------------------------------
// Supporting structs
// ---------------------------------------------------------------------------

/// A key assumption underlying an accounting estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateAssumption {
    /// Plain-language description of the assumption (e.g. "discount rate 4.5 %").
    pub description: String,
    /// Sensitivity of the estimate to a 1-unit change in this assumption
    /// (expressed as absolute monetary impact).
    #[serde(with = "rust_decimal::serde::str")]
    pub sensitivity: Decimal,
    /// Auditor's assessment of the assumption's reasonableness.
    pub reasonableness: AssumptionAssessment,
}

/// ISA 540 risk factor indicators for the estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isa540RiskFactors {
    /// Estimation uncertainty indicator per ISA 540.
    pub estimation_uncertainty: UncertaintyLevel,
    /// Complexity of the estimation process.
    pub complexity: EstimateComplexity,
    /// Degree of management subjectivity in key assumptions.
    pub subjectivity: SubjectivityLevel,
}

/// Retrospective review comparing the prior-period estimate to the actual outcome.
///
/// ISA 540 requires auditors to review management's prior-period estimates
/// to detect indications of possible management bias.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrospectiveReview {
    /// Management's estimate at the prior period-end.
    #[serde(with = "rust_decimal::serde::str")]
    pub prior_period_estimate: Decimal,
    /// Actual outcome observed in the current period.
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_outcome: Decimal,
    /// Monetary variance (actual − estimate).
    #[serde(with = "rust_decimal::serde::str")]
    pub variance: Decimal,
    /// Variance expressed as a percentage of the prior-period estimate.
    #[serde(with = "rust_decimal::serde::str")]
    pub variance_percentage: Decimal,
    /// `true` if the direction of variance suggests consistent management bias
    /// (e.g. estimate consistently overstated vs actual).
    pub management_bias_indicator: bool,
}

// ---------------------------------------------------------------------------
// Primary model
// ---------------------------------------------------------------------------

/// A single accounting estimate reviewed under ISA 540.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEstimate {
    /// Unique identifier for this estimate.
    pub id: String,
    /// Entity code of the reporting entity.
    pub entity_code: String,
    /// Type of accounting estimate.
    pub estimate_type: EstimateType,
    /// Human-readable description (e.g. "ECL allowance — trade receivables").
    pub description: String,
    /// Management's point estimate (the amount recognised in the financial statements).
    #[serde(with = "rust_decimal::serde::str")]
    pub management_point_estimate: Decimal,
    /// Auditor's independent point estimate (when developed as an audit procedure).
    #[serde(default, skip_serializing_if = "Option::is_none", with = "rust_decimal::serde::str_option")]
    pub auditor_point_estimate: Option<Decimal>,
    /// Auditor's assessment of estimation uncertainty per ISA 540.
    pub estimation_uncertainty: UncertaintyLevel,
    /// Auditor's assessment of the complexity of the estimation process.
    pub complexity: EstimateComplexity,
    /// Key assumptions identified and assessed during the audit.
    pub assumptions: Vec<EstimateAssumption>,
    /// Retrospective review of the prior-period estimate (when applicable).
    pub retrospective_review: Option<RetrospectiveReview>,
    /// Consolidated ISA 540 risk factor indicators for the estimate.
    pub isa540_risk_factors: Isa540RiskFactors,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_estimate_type_display() {
        assert_eq!(
            EstimateType::ExpectedCreditLoss.to_string(),
            "Expected Credit Loss"
        );
        assert_eq!(
            EstimateType::PensionObligation.to_string(),
            "Pension Obligation"
        );
    }

    #[test]
    fn test_accounting_estimate_roundtrip() {
        let estimate = AccountingEstimate {
            id: "EST-001".to_string(),
            entity_code: "C001".to_string(),
            estimate_type: EstimateType::ExpectedCreditLoss,
            description: "ECL allowance — trade receivables".to_string(),
            management_point_estimate: dec!(125000.00),
            auditor_point_estimate: Some(dec!(130000.00)),
            estimation_uncertainty: UncertaintyLevel::High,
            complexity: EstimateComplexity::Moderate,
            assumptions: vec![EstimateAssumption {
                description: "12-month default rate 2.5%".to_string(),
                sensitivity: dec!(50000.00),
                reasonableness: AssumptionAssessment::Reasonable,
            }],
            retrospective_review: Some(RetrospectiveReview {
                prior_period_estimate: dec!(115000.00),
                actual_outcome: dec!(118000.00),
                variance: dec!(3000.00),
                variance_percentage: dec!(2.61),
                management_bias_indicator: false,
            }),
            isa540_risk_factors: Isa540RiskFactors {
                estimation_uncertainty: UncertaintyLevel::High,
                complexity: EstimateComplexity::Moderate,
                subjectivity: SubjectivityLevel::Medium,
            },
        };

        let json = serde_json::to_string(&estimate).unwrap();
        let parsed: AccountingEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_code, "C001");
        assert_eq!(parsed.estimate_type, EstimateType::ExpectedCreditLoss);
        assert!(parsed.auditor_point_estimate.is_some());
    }
}
