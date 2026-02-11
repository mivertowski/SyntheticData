//! Analytical Procedures (ISA 520).
//!
//! Implements analytical procedures used throughout the audit:
//! - Risk assessment procedures
//! - Substantive analytical procedures
//! - Final analytical review
//!
//! Analytical procedures involve evaluating financial information through
//! analysis of plausible relationships among both financial and non-financial data.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Analytical procedure record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalProcedure {
    /// Unique procedure identifier.
    pub procedure_id: Uuid,

    /// Engagement ID.
    pub engagement_id: Uuid,

    /// Account or area being analyzed.
    pub account_area: String,

    /// Type of analytical procedure.
    pub procedure_type: AnalyticalProcedureType,

    /// Purpose of the procedure.
    pub purpose: AnalyticalPurpose,

    /// Expectation developed by auditor.
    pub expectation: AnalyticalExpectation,

    /// Actual recorded value.
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_value: Decimal,

    /// Variance from expectation.
    #[serde(with = "rust_decimal::serde::str")]
    pub variance: Decimal,

    /// Variance as percentage.
    #[serde(with = "rust_decimal::serde::str")]
    pub variance_percent: Decimal,

    /// Threshold for investigation.
    #[serde(with = "rust_decimal::serde::str")]
    pub investigation_threshold: Decimal,

    /// Whether variance exceeds threshold.
    pub exceeds_threshold: bool,

    /// Investigation details if variance exceeded threshold.
    pub investigation: Option<VarianceInvestigation>,

    /// Conclusion from the procedure.
    pub conclusion: AnalyticalConclusion,

    /// Procedure date.
    pub procedure_date: NaiveDate,

    /// Preparer ID.
    pub prepared_by: String,

    /// Reviewer ID.
    pub reviewed_by: Option<String>,

    /// Workpaper reference.
    pub workpaper_reference: Option<String>,
}

impl AnalyticalProcedure {
    /// Create a new analytical procedure.
    pub fn new(
        engagement_id: Uuid,
        account_area: impl Into<String>,
        procedure_type: AnalyticalProcedureType,
        purpose: AnalyticalPurpose,
    ) -> Self {
        Self {
            procedure_id: Uuid::now_v7(),
            engagement_id,
            account_area: account_area.into(),
            procedure_type,
            purpose,
            expectation: AnalyticalExpectation::default(),
            actual_value: Decimal::ZERO,
            variance: Decimal::ZERO,
            variance_percent: Decimal::ZERO,
            investigation_threshold: Decimal::ZERO,
            exceeds_threshold: false,
            investigation: None,
            conclusion: AnalyticalConclusion::NotCompleted,
            procedure_date: chrono::Utc::now().date_naive(),
            prepared_by: String::new(),
            reviewed_by: None,
            workpaper_reference: None,
        }
    }

    /// Calculate variance from expectation.
    pub fn calculate_variance(&mut self) {
        self.variance = self.actual_value - self.expectation.expected_value;

        // Calculate percentage variance
        if self.expectation.expected_value != Decimal::ZERO {
            self.variance_percent =
                (self.variance / self.expectation.expected_value) * Decimal::from(100);
        } else {
            self.variance_percent = Decimal::ZERO;
        }

        // Check if exceeds threshold
        self.exceeds_threshold = self.variance.abs() > self.investigation_threshold;
    }

    /// Determine if investigation is required.
    pub fn requires_investigation(&self) -> bool {
        self.exceeds_threshold
    }
}

/// Type of analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalProcedureType {
    /// Trend analysis (comparison over time).
    #[default]
    Trend,
    /// Ratio analysis (financial ratios).
    Ratio,
    /// Reasonableness test (expectation model).
    Reasonableness,
    /// Regression analysis.
    Regression,
    /// Comparison to budget or forecast.
    BudgetComparison,
    /// Industry comparison.
    IndustryComparison,
    /// Non-financial to financial relationship.
    NonFinancialRelationship,
}

impl std::fmt::Display for AnalyticalProcedureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trend => write!(f, "Trend Analysis"),
            Self::Ratio => write!(f, "Ratio Analysis"),
            Self::Reasonableness => write!(f, "Reasonableness Test"),
            Self::Regression => write!(f, "Regression Analysis"),
            Self::BudgetComparison => write!(f, "Budget Comparison"),
            Self::IndustryComparison => write!(f, "Industry Comparison"),
            Self::NonFinancialRelationship => write!(f, "Non-Financial Relationship"),
        }
    }
}

/// Purpose of analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalPurpose {
    /// Used during risk assessment (ISA 315).
    #[default]
    RiskAssessment,
    /// Used as substantive procedure (ISA 520).
    Substantive,
    /// Used during final review (ISA 520).
    FinalReview,
}

impl std::fmt::Display for AnalyticalPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RiskAssessment => write!(f, "Risk Assessment"),
            Self::Substantive => write!(f, "Substantive"),
            Self::FinalReview => write!(f, "Final Review"),
        }
    }
}

/// Auditor's expectation for analytical procedure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticalExpectation {
    /// Expected value.
    #[serde(with = "rust_decimal::serde::str")]
    pub expected_value: Decimal,

    /// Basis for the expectation.
    pub expectation_basis: ExpectationBasis,

    /// Description of how expectation was developed.
    pub methodology: String,

    /// Reliability level of underlying data.
    pub data_reliability: ReliabilityLevel,

    /// Precision of the expectation.
    pub precision_level: PrecisionLevel,

    /// Key assumptions made.
    pub key_assumptions: Vec<String>,

    /// Data sources used.
    pub data_sources: Vec<String>,
}

impl AnalyticalExpectation {
    /// Create a new expectation.
    pub fn new(expected_value: Decimal, expectation_basis: ExpectationBasis) -> Self {
        Self {
            expected_value,
            expectation_basis,
            methodology: String::new(),
            data_reliability: ReliabilityLevel::default(),
            precision_level: PrecisionLevel::default(),
            key_assumptions: Vec::new(),
            data_sources: Vec::new(),
        }
    }
}

/// Basis for developing expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpectationBasis {
    /// Based on prior period amounts.
    #[default]
    PriorPeriod,
    /// Based on budget or forecast.
    Budget,
    /// Based on industry data.
    Industry,
    /// Based on non-financial data.
    NonFinancial,
    /// Based on statistical model.
    StatisticalModel,
    /// Based on auditor's independent calculation.
    IndependentCalculation,
    /// Based on interim period results.
    InterimResults,
}

impl std::fmt::Display for ExpectationBasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PriorPeriod => write!(f, "Prior Period"),
            Self::Budget => write!(f, "Budget/Forecast"),
            Self::Industry => write!(f, "Industry Data"),
            Self::NonFinancial => write!(f, "Non-Financial Data"),
            Self::StatisticalModel => write!(f, "Statistical Model"),
            Self::IndependentCalculation => write!(f, "Independent Calculation"),
            Self::InterimResults => write!(f, "Interim Results"),
        }
    }
}

/// Reliability level of underlying data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReliabilityLevel {
    /// Low reliability (e.g., management estimates).
    Low,
    /// Moderate reliability (e.g., unaudited internal data).
    #[default]
    Moderate,
    /// High reliability (e.g., audited data, external sources).
    High,
}

impl std::fmt::Display for ReliabilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Moderate => write!(f, "Moderate"),
            Self::High => write!(f, "High"),
        }
    }
}

/// Precision level of the expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrecisionLevel {
    /// Low precision (general reasonableness).
    Low,
    /// Moderate precision.
    #[default]
    Moderate,
    /// High precision (detailed calculation).
    High,
}

/// Variance investigation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceInvestigation {
    /// Investigation ID.
    pub investigation_id: Uuid,

    /// Explanation obtained for variance.
    pub explanation: String,

    /// Source of explanation (e.g., management, documentation).
    pub explanation_source: String,

    /// Whether explanation was corroborated.
    pub corroborated: bool,

    /// Corroborating evidence obtained.
    pub corroborating_evidence: Vec<String>,

    /// Additional procedures performed.
    pub additional_procedures: Vec<String>,

    /// Whether variance is explained and reasonable.
    pub variance_explained: bool,

    /// Misstatement identified, if any.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub misstatement_amount: Option<Decimal>,

    /// Investigation conclusion.
    pub conclusion: InvestigationConclusion,
}

impl VarianceInvestigation {
    /// Create a new variance investigation.
    pub fn new() -> Self {
        Self {
            investigation_id: Uuid::now_v7(),
            explanation: String::new(),
            explanation_source: String::new(),
            corroborated: false,
            corroborating_evidence: Vec::new(),
            additional_procedures: Vec::new(),
            variance_explained: false,
            misstatement_amount: None,
            conclusion: InvestigationConclusion::NotCompleted,
        }
    }
}

impl Default for VarianceInvestigation {
    fn default() -> Self {
        Self::new()
    }
}

/// Investigation conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationConclusion {
    /// Investigation not completed.
    #[default]
    NotCompleted,
    /// Variance is explained and reasonable.
    Explained,
    /// Variance is explained but may indicate misstatement.
    PotentialMisstatement,
    /// Misstatement identified.
    MisstatementIdentified,
    /// Unable to explain variance.
    UnableToExplain,
}

/// Conclusion from analytical procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalConclusion {
    /// Procedure not completed.
    #[default]
    NotCompleted,
    /// Results consistent with expectations, no further work needed.
    Consistent,
    /// Results inconsistent, investigation performed, variance explained.
    InvestigatedAndExplained,
    /// Results indicate potential misstatement, requires follow-up.
    PotentialMisstatement,
    /// Misstatement identified.
    MisstatementIdentified,
    /// Unable to form conclusion, alternative procedures needed.
    Inconclusive,
}

impl std::fmt::Display for AnalyticalConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCompleted => write!(f, "Not Completed"),
            Self::Consistent => write!(f, "Consistent with Expectations"),
            Self::InvestigatedAndExplained => write!(f, "Investigated and Explained"),
            Self::PotentialMisstatement => write!(f, "Potential Misstatement"),
            Self::MisstatementIdentified => write!(f, "Misstatement Identified"),
            Self::Inconclusive => write!(f, "Inconclusive"),
        }
    }
}

/// Common financial ratio types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinancialRatio {
    // Profitability Ratios
    GrossMargin,
    OperatingMargin,
    NetProfitMargin,
    ReturnOnAssets,
    ReturnOnEquity,

    // Liquidity Ratios
    CurrentRatio,
    QuickRatio,
    CashRatio,

    // Activity/Efficiency Ratios
    InventoryTurnover,
    ReceivablesTurnover,
    PayablesTurnover,
    AssetTurnover,
    DaysSalesOutstanding,
    DaysPayablesOutstanding,
    DaysInventoryOnHand,

    // Leverage Ratios
    DebtToEquity,
    DebtToAssets,
    InterestCoverage,

    // Other
    Custom,
}

impl std::fmt::Display for FinancialRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GrossMargin => write!(f, "Gross Margin"),
            Self::OperatingMargin => write!(f, "Operating Margin"),
            Self::NetProfitMargin => write!(f, "Net Profit Margin"),
            Self::ReturnOnAssets => write!(f, "Return on Assets"),
            Self::ReturnOnEquity => write!(f, "Return on Equity"),
            Self::CurrentRatio => write!(f, "Current Ratio"),
            Self::QuickRatio => write!(f, "Quick Ratio"),
            Self::CashRatio => write!(f, "Cash Ratio"),
            Self::InventoryTurnover => write!(f, "Inventory Turnover"),
            Self::ReceivablesTurnover => write!(f, "Receivables Turnover"),
            Self::PayablesTurnover => write!(f, "Payables Turnover"),
            Self::AssetTurnover => write!(f, "Asset Turnover"),
            Self::DaysSalesOutstanding => write!(f, "Days Sales Outstanding"),
            Self::DaysPayablesOutstanding => write!(f, "Days Payables Outstanding"),
            Self::DaysInventoryOnHand => write!(f, "Days Inventory on Hand"),
            Self::DebtToEquity => write!(f, "Debt to Equity"),
            Self::DebtToAssets => write!(f, "Debt to Assets"),
            Self::InterestCoverage => write!(f, "Interest Coverage"),
            Self::Custom => write!(f, "Custom Ratio"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_analytical_procedure_creation() {
        let procedure = AnalyticalProcedure::new(
            Uuid::now_v7(),
            "Revenue",
            AnalyticalProcedureType::Trend,
            AnalyticalPurpose::Substantive,
        );

        assert_eq!(procedure.account_area, "Revenue");
        assert_eq!(procedure.procedure_type, AnalyticalProcedureType::Trend);
        assert_eq!(procedure.conclusion, AnalyticalConclusion::NotCompleted);
    }

    #[test]
    fn test_variance_calculation() {
        let mut procedure = AnalyticalProcedure::new(
            Uuid::now_v7(),
            "Revenue",
            AnalyticalProcedureType::Trend,
            AnalyticalPurpose::Substantive,
        );

        procedure.expectation =
            AnalyticalExpectation::new(dec!(100000), ExpectationBasis::PriorPeriod);
        procedure.actual_value = dec!(110000);
        procedure.investigation_threshold = dec!(5000);

        procedure.calculate_variance();

        assert_eq!(procedure.variance, dec!(10000));
        assert_eq!(procedure.variance_percent, dec!(10));
        assert!(procedure.exceeds_threshold);
    }

    #[test]
    fn test_variance_within_threshold() {
        let mut procedure = AnalyticalProcedure::new(
            Uuid::now_v7(),
            "Cost of Sales",
            AnalyticalProcedureType::Reasonableness,
            AnalyticalPurpose::FinalReview,
        );

        procedure.expectation =
            AnalyticalExpectation::new(dec!(50000), ExpectationBasis::IndependentCalculation);
        procedure.actual_value = dec!(51000);
        procedure.investigation_threshold = dec!(2500);

        procedure.calculate_variance();

        assert_eq!(procedure.variance, dec!(1000));
        assert!(!procedure.exceeds_threshold);
    }
}
