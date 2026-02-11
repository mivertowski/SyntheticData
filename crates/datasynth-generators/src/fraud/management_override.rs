//! Management override fraud patterns.
//!
//! Models fraud at senior management level including:
//! - Revenue override techniques
//! - Expense manipulation
//! - Asset valuation overrides
//! - Fraud triangle integration

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use datasynth_core::{
    AcfeFraudCategory, AnomalyDetectionDifficulty, FraudTriangle, OpportunityFactor, PressureType,
    Rationalization,
};

/// Level of management involved in override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManagementLevel {
    /// Senior management (VP, Director).
    SeniorManagement,
    /// C-suite executives (CEO, CFO, COO).
    CSuite,
    /// Board of directors or audit committee.
    Board,
}

impl ManagementLevel {
    /// Returns the typical detection difficulty for this level.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            ManagementLevel::SeniorManagement => AnomalyDetectionDifficulty::Hard,
            ManagementLevel::CSuite => AnomalyDetectionDifficulty::Expert,
            ManagementLevel::Board => AnomalyDetectionDifficulty::Expert,
        }
    }

    /// Returns the typical median loss for this level (based on ACFE data).
    pub fn typical_median_loss(&self) -> Decimal {
        match self {
            ManagementLevel::SeniorManagement => Decimal::new(150_000, 0),
            ManagementLevel::CSuite => Decimal::new(600_000, 0),
            ManagementLevel::Board => Decimal::new(500_000, 0),
        }
    }

    /// Returns the probability of successful concealment.
    pub fn concealment_probability(&self) -> f64 {
        match self {
            ManagementLevel::SeniorManagement => 0.70,
            ManagementLevel::CSuite => 0.85,
            ManagementLevel::Board => 0.80,
        }
    }
}

/// Type of management override.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OverrideType {
    /// Revenue recognition overrides.
    Revenue(Vec<RevenueOverrideTechnique>),
    /// Expense timing overrides.
    Expense(Vec<ExpenseOverrideTechnique>),
    /// Asset valuation overrides.
    Asset(Vec<AssetOverrideTechnique>),
    /// Reserve manipulation.
    Reserve(Vec<ReserveOverrideTechnique>),
}

impl OverrideType {
    /// Returns the ACFE category for this override type.
    pub fn acfe_category(&self) -> AcfeFraudCategory {
        AcfeFraudCategory::FinancialStatementFraud
    }

    /// Returns a description of the override type.
    pub fn description(&self) -> String {
        match self {
            OverrideType::Revenue(techniques) => {
                format!("Revenue override: {:?}", techniques)
            }
            OverrideType::Expense(techniques) => {
                format!("Expense override: {:?}", techniques)
            }
            OverrideType::Asset(techniques) => {
                format!("Asset valuation override: {:?}", techniques)
            }
            OverrideType::Reserve(techniques) => {
                format!("Reserve manipulation: {:?}", techniques)
            }
        }
    }
}

/// Revenue override techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RevenueOverrideTechnique {
    /// Overriding journal entries to accelerate revenue.
    JournalEntryOverride,
    /// Accelerating revenue recognition timing.
    RevenueRecognitionAcceleration,
    /// Reducing allowance for doubtful accounts.
    AllowanceReduction,
    /// Concealing side agreements with customers.
    SideAgreementConcealment,
    /// Channel stuffing with side letters.
    ChannelStuffingWithSideLetters,
    /// Bill-and-hold arrangements without proper criteria.
    ImproperBillAndHold,
    /// Percentage of completion overstatement.
    PercentageOfCompletionOverstatement,
}

impl RevenueOverrideTechnique {
    /// Returns the detection difficulty for this technique.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            RevenueOverrideTechnique::JournalEntryOverride => AnomalyDetectionDifficulty::Moderate,
            RevenueOverrideTechnique::RevenueRecognitionAcceleration => {
                AnomalyDetectionDifficulty::Hard
            }
            RevenueOverrideTechnique::AllowanceReduction => AnomalyDetectionDifficulty::Moderate,
            RevenueOverrideTechnique::SideAgreementConcealment => {
                AnomalyDetectionDifficulty::Expert
            }
            RevenueOverrideTechnique::ChannelStuffingWithSideLetters => {
                AnomalyDetectionDifficulty::Expert
            }
            RevenueOverrideTechnique::ImproperBillAndHold => AnomalyDetectionDifficulty::Hard,
            RevenueOverrideTechnique::PercentageOfCompletionOverstatement => {
                AnomalyDetectionDifficulty::Hard
            }
        }
    }

    /// Returns typical indicators for this technique.
    pub fn indicators(&self) -> Vec<&'static str> {
        match self {
            RevenueOverrideTechnique::JournalEntryOverride => {
                vec!["manual_je_at_period_end", "unusual_revenue_account_entries"]
            }
            RevenueOverrideTechnique::RevenueRecognitionAcceleration => {
                vec![
                    "revenue_spike_at_period_end",
                    "reversals_in_subsequent_period",
                ]
            }
            RevenueOverrideTechnique::AllowanceReduction => {
                vec![
                    "allowance_ratio_decline",
                    "aging_profile_deterioration_without_allowance_increase",
                ]
            }
            RevenueOverrideTechnique::SideAgreementConcealment => {
                vec!["unusual_return_rates", "credit_memos_post_period"]
            }
            RevenueOverrideTechnique::ChannelStuffingWithSideLetters => {
                vec![
                    "distributor_inventory_buildup",
                    "quarter_end_shipment_spike",
                ]
            }
            RevenueOverrideTechnique::ImproperBillAndHold => {
                vec!["inventory_not_shipped", "unusual_storage_arrangements"]
            }
            RevenueOverrideTechnique::PercentageOfCompletionOverstatement => {
                vec![
                    "cost_to_complete_estimates_declining",
                    "revenue_recognized_exceeds_billing",
                ]
            }
        }
    }
}

/// Expense override techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpenseOverrideTechnique {
    /// Capitalizing expenses that should be expensed.
    CapitalizationAbuse,
    /// Deferring expense recognition.
    ExpenseDeferral,
    /// Manipulating cost allocations.
    CostAllocationManipulation,
    /// Failing to record accrued liabilities.
    AccrualOmission,
    /// Improperly extending useful lives.
    UsefulLifeExtension,
    /// Changing depreciation methods.
    DepreciationMethodChange,
}

impl ExpenseOverrideTechnique {
    /// Returns the detection difficulty for this technique.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            ExpenseOverrideTechnique::CapitalizationAbuse => AnomalyDetectionDifficulty::Hard,
            ExpenseOverrideTechnique::ExpenseDeferral => AnomalyDetectionDifficulty::Moderate,
            ExpenseOverrideTechnique::CostAllocationManipulation => {
                AnomalyDetectionDifficulty::Hard
            }
            ExpenseOverrideTechnique::AccrualOmission => AnomalyDetectionDifficulty::Moderate,
            ExpenseOverrideTechnique::UsefulLifeExtension => AnomalyDetectionDifficulty::Moderate,
            ExpenseOverrideTechnique::DepreciationMethodChange => AnomalyDetectionDifficulty::Easy,
        }
    }
}

/// Asset override techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetOverrideTechnique {
    /// Avoiding or manipulating impairment tests.
    ImpairmentAvoidance,
    /// Manipulating fair value measurements.
    FairValueManipulation,
    /// Overstating inventory values.
    InventoryOverstatement,
    /// Failing to write off obsolete assets.
    ObsolescenceConcealment,
    /// Manipulating receivables aging.
    ReceivablesAging,
}

impl AssetOverrideTechnique {
    /// Returns the detection difficulty for this technique.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            AssetOverrideTechnique::ImpairmentAvoidance => AnomalyDetectionDifficulty::Hard,
            AssetOverrideTechnique::FairValueManipulation => AnomalyDetectionDifficulty::Expert,
            AssetOverrideTechnique::InventoryOverstatement => AnomalyDetectionDifficulty::Moderate,
            AssetOverrideTechnique::ObsolescenceConcealment => AnomalyDetectionDifficulty::Hard,
            AssetOverrideTechnique::ReceivablesAging => AnomalyDetectionDifficulty::Moderate,
        }
    }
}

/// Reserve manipulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReserveOverrideTechnique {
    /// Cookie jar reserves (over-reserve in good times).
    CookieJarReserves,
    /// Releasing reserves to meet targets.
    ReserveRelease,
    /// Understating warranty reserves.
    WarrantyReserveUnderstatement,
    /// Manipulating restructuring reserves.
    RestructuringReserveManipulation,
}

impl ReserveOverrideTechnique {
    /// Returns the detection difficulty for this technique.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            ReserveOverrideTechnique::CookieJarReserves => AnomalyDetectionDifficulty::Hard,
            ReserveOverrideTechnique::ReserveRelease => AnomalyDetectionDifficulty::Moderate,
            ReserveOverrideTechnique::WarrantyReserveUnderstatement => {
                AnomalyDetectionDifficulty::Hard
            }
            ReserveOverrideTechnique::RestructuringReserveManipulation => {
                AnomalyDetectionDifficulty::Hard
            }
        }
    }
}

/// Concealment methods used by management.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagementConcealment {
    /// Creates or alters documentation to support fraud.
    pub false_documentation: bool,
    /// Uses position to intimidate subordinates into compliance.
    pub intimidation_of_subordinates: bool,
    /// Deliberately misleads auditors.
    pub auditor_deception: bool,
    /// Bypasses or undermines board oversight.
    pub board_oversight_circumvention: bool,
    /// Controls access to information.
    pub information_control: bool,
    /// Uses complex transactions to obscure fraud.
    pub transaction_complexity: bool,
    /// Uses related parties to facilitate fraud.
    pub related_party_concealment: bool,
}

impl ManagementConcealment {
    /// Returns the count of active concealment methods.
    pub fn active_count(&self) -> u32 {
        [
            self.false_documentation,
            self.intimidation_of_subordinates,
            self.auditor_deception,
            self.board_oversight_circumvention,
            self.information_control,
            self.transaction_complexity,
            self.related_party_concealment,
        ]
        .iter()
        .filter(|&&x| x)
        .count() as u32
    }

    /// Returns the detection difficulty modifier based on concealment.
    pub fn difficulty_modifier(&self) -> f64 {
        // Each active concealment method adds to difficulty
        1.0 + (self.active_count() as f64 * 0.1)
    }

    /// Returns indicators that might reveal the concealment.
    pub fn potential_indicators(&self) -> Vec<&'static str> {
        let mut indicators = Vec::new();
        if self.false_documentation {
            indicators.push("document_inconsistencies");
        }
        if self.intimidation_of_subordinates {
            indicators.push("employee_turnover_in_accounting");
            indicators.push("anonymous_hotline_tips");
        }
        if self.auditor_deception {
            indicators.push("limited_information_to_auditors");
            indicators.push("auditor_scope_limitations");
        }
        if self.board_oversight_circumvention {
            indicators.push("limited_board_information");
            indicators.push("audit_committee_turnover");
        }
        indicators
    }
}

/// Management override fraud scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementOverrideScheme {
    /// Unique scheme identifier.
    pub scheme_id: Uuid,
    /// Level of management perpetrating the fraud.
    pub perpetrator_level: ManagementLevel,
    /// Perpetrator entity ID.
    pub perpetrator_id: String,
    /// Type of override being used.
    pub override_type: OverrideType,
    /// Fraud triangle components.
    pub fraud_triangle: FraudTriangle,
    /// Concealment methods used.
    pub concealment: ManagementConcealment,
    /// Start date of the scheme.
    pub start_date: NaiveDate,
    /// End date (if detected or stopped).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
    /// Total financial impact.
    pub financial_impact: Decimal,
    /// Number of periods affected.
    pub periods_affected: u32,
    /// Whether the scheme has been detected.
    pub is_detected: bool,
    /// Detection method if detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detection_method: Option<String>,
    /// Related transaction IDs.
    pub transaction_ids: Vec<String>,
    /// Metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ManagementOverrideScheme {
    /// Creates a new management override scheme.
    pub fn new(
        perpetrator_level: ManagementLevel,
        perpetrator_id: impl Into<String>,
        override_type: OverrideType,
        start_date: NaiveDate,
    ) -> Self {
        // Create default fraud triangle
        let fraud_triangle = FraudTriangle::new(
            PressureType::FinancialTargets,
            vec![OpportunityFactor::ManagementOverride],
            Rationalization::ForTheCompanyGood,
        );

        Self {
            scheme_id: Uuid::new_v4(),
            perpetrator_level,
            perpetrator_id: perpetrator_id.into(),
            override_type,
            fraud_triangle,
            concealment: ManagementConcealment::default(),
            start_date,
            end_date: None,
            financial_impact: Decimal::ZERO,
            periods_affected: 0,
            is_detected: false,
            detection_method: None,
            transaction_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Sets the fraud triangle.
    pub fn with_fraud_triangle(mut self, triangle: FraudTriangle) -> Self {
        self.fraud_triangle = triangle;
        self
    }

    /// Sets the concealment methods.
    pub fn with_concealment(mut self, concealment: ManagementConcealment) -> Self {
        self.concealment = concealment;
        self
    }

    /// Records a fraudulent transaction.
    pub fn record_transaction(&mut self, amount: Decimal, transaction_id: impl Into<String>) {
        self.financial_impact += amount;
        self.transaction_ids.push(transaction_id.into());
    }

    /// Records period end activity.
    pub fn record_period(&mut self) {
        self.periods_affected += 1;
    }

    /// Marks the scheme as detected.
    pub fn mark_detected(&mut self, end_date: NaiveDate, method: impl Into<String>) {
        self.is_detected = true;
        self.end_date = Some(end_date);
        self.detection_method = Some(method.into());
    }

    /// Returns the overall detection difficulty.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        let base = self.perpetrator_level.detection_difficulty();
        let concealment_modifier = self.concealment.difficulty_modifier();

        let score = base.difficulty_score() * concealment_modifier;
        AnomalyDetectionDifficulty::from_score(score.min(1.0))
    }

    /// Returns key risk indicators for this scheme.
    pub fn risk_indicators(&self) -> Vec<String> {
        let mut indicators = Vec::new();

        // Add technique-specific indicators
        if let OverrideType::Revenue(techniques) = &self.override_type {
            for tech in techniques {
                indicators.extend(tech.indicators().into_iter().map(String::from));
            }
        }

        // Add concealment indicators
        indicators.extend(
            self.concealment
                .potential_indicators()
                .into_iter()
                .map(String::from),
        );

        // General override indicators
        indicators.push("manual_period_end_entries".to_string());
        indicators.push("entries_without_supporting_documentation".to_string());
        indicators.push("overridden_system_controls".to_string());

        indicators
    }

    /// Returns a summary description of the scheme.
    pub fn description(&self) -> String {
        format!(
            "{:?} level override: {}, impact: {}, {} periods affected",
            self.perpetrator_level,
            self.override_type.description(),
            self.financial_impact,
            self.periods_affected
        )
    }
}

/// Generator for management override schemes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementOverrideGenerator {
    /// Probability of management override in financial statement fraud.
    pub override_rate: f64,
    /// Distribution of override types.
    pub type_weights: HashMap<String, f64>,
    /// Distribution of management levels.
    pub level_weights: HashMap<String, f64>,
}

impl Default for ManagementOverrideGenerator {
    fn default() -> Self {
        let mut type_weights = HashMap::new();
        type_weights.insert("revenue".to_string(), 0.40);
        type_weights.insert("expense".to_string(), 0.25);
        type_weights.insert("asset".to_string(), 0.20);
        type_weights.insert("reserve".to_string(), 0.15);

        let mut level_weights = HashMap::new();
        level_weights.insert("senior_management".to_string(), 0.50);
        level_weights.insert("c_suite".to_string(), 0.35);
        level_weights.insert("board".to_string(), 0.15);

        Self {
            override_rate: 0.70, // 70% of FS fraud involves management override
            type_weights,
            level_weights,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_management_level() {
        let cfo = ManagementLevel::CSuite;
        assert_eq!(
            cfo.detection_difficulty(),
            AnomalyDetectionDifficulty::Expert
        );
        assert_eq!(cfo.typical_median_loss(), Decimal::new(600_000, 0));
    }

    #[test]
    fn test_revenue_override_technique() {
        let tech = RevenueOverrideTechnique::SideAgreementConcealment;
        assert_eq!(
            tech.detection_difficulty(),
            AnomalyDetectionDifficulty::Expert
        );
        assert!(!tech.indicators().is_empty());
    }

    #[test]
    fn test_management_concealment() {
        let mut concealment = ManagementConcealment::default();
        assert_eq!(concealment.active_count(), 0);
        assert_eq!(concealment.difficulty_modifier(), 1.0);

        concealment.false_documentation = true;
        concealment.auditor_deception = true;
        concealment.intimidation_of_subordinates = true;

        assert_eq!(concealment.active_count(), 3);
        assert!((concealment.difficulty_modifier() - 1.3).abs() < 0.01);
        assert!(!concealment.potential_indicators().is_empty());
    }

    #[test]
    fn test_management_override_scheme() {
        let scheme = ManagementOverrideScheme::new(
            ManagementLevel::CSuite,
            "CFO001",
            OverrideType::Revenue(vec![
                RevenueOverrideTechnique::RevenueRecognitionAcceleration,
                RevenueOverrideTechnique::ChannelStuffingWithSideLetters,
            ]),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(scheme.perpetrator_level, ManagementLevel::CSuite);
        assert!(!scheme.is_detected);
        assert!(!scheme.risk_indicators().is_empty());
    }

    #[test]
    fn test_scheme_transaction_recording() {
        let mut scheme = ManagementOverrideScheme::new(
            ManagementLevel::SeniorManagement,
            "VP001",
            OverrideType::Expense(vec![ExpenseOverrideTechnique::CapitalizationAbuse]),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        scheme.record_transaction(Decimal::new(100_000, 0), "JE001");
        scheme.record_transaction(Decimal::new(50_000, 0), "JE002");
        scheme.record_period();

        assert_eq!(scheme.financial_impact, Decimal::new(150_000, 0));
        assert_eq!(scheme.transaction_ids.len(), 2);
        assert_eq!(scheme.periods_affected, 1);
    }

    #[test]
    fn test_scheme_detection() {
        let mut scheme = ManagementOverrideScheme::new(
            ManagementLevel::SeniorManagement,
            "VP001",
            OverrideType::Asset(vec![AssetOverrideTechnique::ImpairmentAvoidance]),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        scheme.mark_detected(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "internal_audit",
        );

        assert!(scheme.is_detected);
        assert_eq!(
            scheme.end_date,
            Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
        );
        assert_eq!(scheme.detection_method, Some("internal_audit".to_string()));
    }

    #[test]
    fn test_fraud_triangle_integration() {
        let triangle = FraudTriangle::new(
            PressureType::MarketExpectations,
            vec![
                OpportunityFactor::ManagementOverride,
                OpportunityFactor::WeakInternalControls,
            ],
            Rationalization::ForTheCompanyGood,
        );

        let scheme = ManagementOverrideScheme::new(
            ManagementLevel::CSuite,
            "CEO001",
            OverrideType::Revenue(vec![RevenueOverrideTechnique::JournalEntryOverride]),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_fraud_triangle(triangle);

        assert_eq!(
            scheme.fraud_triangle.pressure,
            PressureType::MarketExpectations
        );
        assert!(scheme.fraud_triangle.risk_score() > 0.5);
    }
}
