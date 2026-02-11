//! Case narrative models for SAR generation and ML training.

use chrono::NaiveDate;
use datasynth_core::models::banking::{AmlTypology, LaunderingStage, Sophistication};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// EvasionTactic is re-exported at the bottom of the file

/// An AML scenario (ground truth case).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlScenario {
    /// Unique scenario identifier
    pub scenario_id: String,
    /// Primary AML typology
    pub typology: AmlTypology,
    /// Secondary typologies (if multiple)
    pub secondary_typologies: Vec<AmlTypology>,
    /// Money laundering stages involved
    pub stages: Vec<LaunderingStage>,
    /// Scenario start date
    pub start_date: NaiveDate,
    /// Scenario end date
    pub end_date: NaiveDate,
    /// Customer IDs involved
    pub involved_customers: Vec<Uuid>,
    /// Account IDs involved
    pub involved_accounts: Vec<Uuid>,
    /// Transaction IDs involved
    pub involved_transactions: Vec<Uuid>,
    /// Total amount laundered/defrauded
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Evasion tactics employed
    pub evasion_tactics: Vec<EvasionTactic>,
    /// Sophistication level
    pub sophistication: Sophistication,
    /// Detectability score (0.0-1.0, higher = easier to detect)
    pub detectability: f64,
    /// Case narrative
    pub narrative: CaseNarrative,
    /// Alert triggers that should fire
    pub expected_alerts: Vec<ExpectedAlert>,
    /// Whether scenario was successfully completed
    pub was_successful: bool,
}

impl AmlScenario {
    /// Create a new AML scenario.
    pub fn new(
        scenario_id: &str,
        typology: AmlTypology,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            scenario_id: scenario_id.to_string(),
            typology,
            secondary_typologies: Vec::new(),
            stages: Vec::new(),
            start_date,
            end_date,
            involved_customers: Vec::new(),
            involved_accounts: Vec::new(),
            involved_transactions: Vec::new(),
            total_amount: Decimal::ZERO,
            evasion_tactics: Vec::new(),
            sophistication: Sophistication::default(),
            detectability: typology.severity() as f64 / 10.0,
            narrative: CaseNarrative::default(),
            expected_alerts: Vec::new(),
            was_successful: true,
        }
    }

    /// Add a stage.
    pub fn add_stage(&mut self, stage: LaunderingStage) {
        if !self.stages.contains(&stage) {
            self.stages.push(stage);
        }
    }

    /// Add a customer.
    pub fn add_customer(&mut self, customer_id: Uuid) {
        if !self.involved_customers.contains(&customer_id) {
            self.involved_customers.push(customer_id);
        }
    }

    /// Add an account.
    pub fn add_account(&mut self, account_id: Uuid) {
        if !self.involved_accounts.contains(&account_id) {
            self.involved_accounts.push(account_id);
        }
    }

    /// Add a transaction.
    pub fn add_transaction(&mut self, transaction_id: Uuid, amount: Decimal) {
        self.involved_transactions.push(transaction_id);
        self.total_amount += amount;
    }

    /// Add an evasion tactic.
    pub fn add_evasion_tactic(&mut self, tactic: EvasionTactic) {
        if !self.evasion_tactics.contains(&tactic) {
            self.evasion_tactics.push(tactic);
            // Adjust detectability
            self.detectability *= 1.0 / tactic.difficulty_modifier();
        }
    }

    /// Set sophistication level.
    pub fn with_sophistication(mut self, sophistication: Sophistication) -> Self {
        self.sophistication = sophistication;
        self.detectability *= sophistication.detectability_modifier();
        self
    }

    /// Calculate case complexity score.
    pub fn complexity_score(&self) -> u8 {
        let mut score = 0.0;

        // Number of entities (use max(1) to avoid ln(0))
        score += (self.involved_customers.len().max(1) as f64).ln() * 10.0;

        // Number of accounts
        score += (self.involved_accounts.len().max(1) as f64).ln() * 5.0;

        // Number of transactions
        score += (self.involved_transactions.len().max(1) as f64).ln() * 3.0;

        // Duration
        let duration = (self.end_date - self.start_date).num_days();
        score += (duration as f64 / 30.0).min(10.0) * 3.0;

        // Evasion tactics
        score += self.evasion_tactics.len() as f64 * 5.0;

        // Sophistication
        score += match self.sophistication {
            Sophistication::Basic => 0.0,
            Sophistication::Standard => 10.0,
            Sophistication::Professional => 20.0,
            Sophistication::Advanced => 30.0,
            Sophistication::StateLevel => 40.0,
        };

        // Number of stages
        score += self.stages.len() as f64 * 5.0;

        score.min(100.0) as u8
    }
}

/// Case narrative for SAR-style reporting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaseNarrative {
    /// Summary storyline
    pub storyline: String,
    /// Key evidence points
    pub evidence_points: Vec<String>,
    /// Violated expectations
    pub violated_expectations: Vec<ViolatedExpectation>,
    /// Red flags identified
    pub red_flags: Vec<RedFlag>,
    /// Recommended action
    pub recommendation: CaseRecommendation,
    /// Investigation notes
    pub investigation_notes: Vec<String>,
}

impl CaseNarrative {
    /// Create a new case narrative.
    pub fn new(storyline: &str) -> Self {
        Self {
            storyline: storyline.to_string(),
            ..Default::default()
        }
    }

    /// Add an evidence point.
    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence_points.push(evidence.to_string());
    }

    /// Add a violated expectation.
    pub fn add_violated_expectation(&mut self, expectation: ViolatedExpectation) {
        self.violated_expectations.push(expectation);
    }

    /// Add a red flag.
    pub fn add_red_flag(&mut self, flag: RedFlag) {
        self.red_flags.push(flag);
    }

    /// Set recommendation.
    pub fn with_recommendation(mut self, recommendation: CaseRecommendation) -> Self {
        self.recommendation = recommendation;
        self
    }

    /// Generate narrative text.
    pub fn generate_text(&self) -> String {
        let mut text = format!("## Case Summary\n\n{}\n\n", self.storyline);

        if !self.evidence_points.is_empty() {
            text.push_str("## Evidence Points\n\n");
            for (i, point) in self.evidence_points.iter().enumerate() {
                text.push_str(&format!("{}. {}\n", i + 1, point));
            }
            text.push('\n');
        }

        if !self.violated_expectations.is_empty() {
            text.push_str("## Violated Expectations\n\n");
            for ve in &self.violated_expectations {
                text.push_str(&format!(
                    "- **{}**: Expected {}, Actual {}\n",
                    ve.expectation_type, ve.expected_value, ve.actual_value
                ));
            }
            text.push('\n');
        }

        if !self.red_flags.is_empty() {
            text.push_str("## Red Flags\n\n");
            for flag in &self.red_flags {
                text.push_str(&format!(
                    "- {} (Severity: {})\n",
                    flag.description, flag.severity
                ));
            }
            text.push('\n');
        }

        text.push_str(&format!(
            "## Recommendation\n\n{}\n",
            self.recommendation.description()
        ));

        text
    }
}

/// A violated KYC expectation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolatedExpectation {
    /// Type of expectation
    pub expectation_type: String,
    /// Expected value
    pub expected_value: String,
    /// Actual value
    pub actual_value: String,
    /// Deviation percentage
    pub deviation_percentage: f64,
}

impl ViolatedExpectation {
    /// Create a new violated expectation.
    pub fn new(expectation_type: &str, expected: &str, actual: &str, deviation: f64) -> Self {
        Self {
            expectation_type: expectation_type.to_string(),
            expected_value: expected.to_string(),
            actual_value: actual.to_string(),
            deviation_percentage: deviation,
        }
    }
}

/// A red flag indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedFlag {
    /// Red flag category
    pub category: RedFlagCategory,
    /// Description
    pub description: String,
    /// Severity (1-10)
    pub severity: u8,
    /// Date identified
    pub date_identified: NaiveDate,
}

impl RedFlag {
    /// Create a new red flag.
    pub fn new(
        category: RedFlagCategory,
        description: &str,
        severity: u8,
        date: NaiveDate,
    ) -> Self {
        Self {
            category,
            description: description.to_string(),
            severity,
            date_identified: date,
        }
    }
}

/// Category of red flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedFlagCategory {
    /// Activity pattern red flag
    ActivityPattern,
    /// Geographic red flag
    Geographic,
    /// Customer behavior red flag
    CustomerBehavior,
    /// Transaction characteristic red flag
    TransactionCharacteristic,
    /// Account characteristic red flag
    AccountCharacteristic,
    /// Third party red flag
    ThirdParty,
    /// Timing red flag
    Timing,
    /// Documentation red flag
    Documentation,
}

/// Case recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaseRecommendation {
    /// Close case - no further action
    #[default]
    CloseNoAction,
    /// Continue monitoring
    ContinueMonitoring,
    /// Enhanced monitoring
    EnhancedMonitoring,
    /// Escalate to compliance
    EscalateCompliance,
    /// File SAR
    FileSar,
    /// Close account
    CloseAccount,
    /// Report to law enforcement
    ReportLawEnforcement,
}

impl CaseRecommendation {
    /// Description of the recommendation.
    pub fn description(&self) -> &'static str {
        match self {
            Self::CloseNoAction => "Close case - no suspicious activity identified",
            Self::ContinueMonitoring => "Continue standard monitoring",
            Self::EnhancedMonitoring => "Place customer under enhanced monitoring",
            Self::EscalateCompliance => "Escalate to compliance officer for review",
            Self::FileSar => "Escalate to SAR filing",
            Self::CloseAccount => "Close account and file SAR",
            Self::ReportLawEnforcement => "Report to law enforcement immediately",
        }
    }

    /// Severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            Self::CloseNoAction => 1,
            Self::ContinueMonitoring => 1,
            Self::EnhancedMonitoring => 2,
            Self::EscalateCompliance => 3,
            Self::FileSar => 4,
            Self::CloseAccount => 4,
            Self::ReportLawEnforcement => 5,
        }
    }
}

/// Expected alert that should be triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedAlert {
    /// Alert type/rule name
    pub alert_type: String,
    /// Expected trigger date
    pub expected_date: NaiveDate,
    /// Transactions that should trigger
    pub triggering_transactions: Vec<Uuid>,
    /// Severity
    pub severity: AlertSeverity,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// Low severity
    #[default]
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

// Re-export EvasionTactic from synth-core
pub use datasynth_core::models::banking::EvasionTactic;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_aml_scenario() {
        let scenario = AmlScenario::new(
            "SC-2024-001",
            AmlTypology::Structuring,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        assert_eq!(scenario.typology, AmlTypology::Structuring);
        assert!(scenario.was_successful);
    }

    #[test]
    fn test_case_narrative() {
        let mut narrative = CaseNarrative::new(
            "Subject conducted 12 cash deposits just below $10,000 threshold over 3 days.",
        );
        narrative.add_evidence("12 deposits ranging from $9,500 to $9,900");
        narrative.add_evidence("All deposits at different branch locations");
        narrative.add_violated_expectation(ViolatedExpectation::new(
            "Monthly deposits",
            "2",
            "12",
            500.0,
        ));

        let text = narrative.generate_text();
        assert!(text.contains("12 cash deposits"));
        assert!(text.contains("Evidence Points"));
    }

    #[test]
    fn test_complexity_score() {
        let mut simple = AmlScenario::new(
            "SC-001",
            AmlTypology::Structuring,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );
        simple.add_customer(Uuid::new_v4());
        simple.add_account(Uuid::new_v4());

        let mut complex = AmlScenario::new(
            "SC-002",
            AmlTypology::Layering,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        )
        .with_sophistication(Sophistication::Advanced);

        for _ in 0..10 {
            complex.add_customer(Uuid::new_v4());
            complex.add_account(Uuid::new_v4());
        }
        complex.add_stage(LaunderingStage::Placement);
        complex.add_stage(LaunderingStage::Layering);
        complex.add_evasion_tactic(EvasionTactic::TimeJitter);
        complex.add_evasion_tactic(EvasionTactic::AccountSplitting);

        assert!(complex.complexity_score() > simple.complexity_score());
    }
}
