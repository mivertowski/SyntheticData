//! Detection difficulty calculation for anomalies.
//!
//! This module provides tools for calculating how difficult an anomaly
//! is to detect, based on various factors like concealment techniques,
//! blending with normal activity, and collusion.

use datasynth_core::models::{
    AnomalyDetectionDifficulty, AnomalyType, ConcealmentTechnique, DetectionMethod, FraudType,
    LabeledAnomaly,
};
use serde::{Deserialize, Serialize};

/// Factors that affect concealment of anomalies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConcealmentFactors {
    /// Document manipulation or forgery used.
    pub document_manipulation: bool,
    /// Approval process circumvention.
    pub approval_circumvention: bool,
    /// Timing exploitation (period-end, holidays).
    pub timing_exploitation: bool,
    /// Transaction splitting to avoid thresholds.
    pub splitting: bool,
    /// Concealment techniques used.
    pub techniques: Vec<ConcealmentTechnique>,
}

impl ConcealmentFactors {
    /// Creates new concealment factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a concealment technique.
    pub fn with_technique(mut self, technique: ConcealmentTechnique) -> Self {
        if !self.techniques.contains(&technique) {
            self.techniques.push(technique);
        }
        self
    }

    /// Sets document manipulation flag.
    pub fn with_document_manipulation(mut self) -> Self {
        self.document_manipulation = true;
        self
    }

    /// Sets approval circumvention flag.
    pub fn with_approval_circumvention(mut self) -> Self {
        self.approval_circumvention = true;
        self
    }

    /// Sets timing exploitation flag.
    pub fn with_timing_exploitation(mut self) -> Self {
        self.timing_exploitation = true;
        self
    }

    /// Sets transaction splitting flag.
    pub fn with_splitting(mut self) -> Self {
        self.splitting = true;
        self
    }

    /// Calculates the total difficulty contribution from concealment.
    pub fn difficulty_contribution(&self) -> f64 {
        let mut contribution = 0.0;

        if self.document_manipulation {
            contribution += 0.20;
        }
        if self.approval_circumvention {
            contribution += 0.15;
        }
        if self.timing_exploitation {
            contribution += 0.10;
        }
        if self.splitting {
            contribution += 0.15;
        }

        // Add technique-specific bonuses
        for technique in &self.techniques {
            contribution += technique.difficulty_bonus();
        }

        // Cap at reasonable maximum
        contribution.min(0.50)
    }
}

/// Factors related to how well anomaly blends with normal activity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlendingFactors {
    /// Amount is within normal range for this account/entity.
    pub amount_within_normal_range: bool,
    /// Timing is within normal business hours.
    pub timing_within_normal_hours: bool,
    /// Counterparty is an established relationship.
    pub counterparty_is_established: bool,
    /// Account coding is correct (just wrong activity).
    pub account_coding_correct: bool,
    /// Description matches normal patterns.
    pub description_matches_pattern: bool,
    /// Transaction frequency is normal.
    pub frequency_is_normal: bool,
}

impl BlendingFactors {
    /// Creates new blending factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets amount within normal range.
    pub fn with_normal_amount(mut self) -> Self {
        self.amount_within_normal_range = true;
        self
    }

    /// Sets timing within normal hours.
    pub fn with_normal_timing(mut self) -> Self {
        self.timing_within_normal_hours = true;
        self
    }

    /// Sets counterparty as established.
    pub fn with_established_counterparty(mut self) -> Self {
        self.counterparty_is_established = true;
        self
    }

    /// Sets account coding as correct.
    pub fn with_correct_coding(mut self) -> Self {
        self.account_coding_correct = true;
        self
    }

    /// Sets description as matching normal patterns.
    pub fn with_normal_description(mut self) -> Self {
        self.description_matches_pattern = true;
        self
    }

    /// Sets frequency as normal.
    pub fn with_normal_frequency(mut self) -> Self {
        self.frequency_is_normal = true;
        self
    }

    /// Calculates the total difficulty contribution from blending.
    pub fn difficulty_contribution(&self) -> f64 {
        let mut contribution: f64 = 0.0;

        if self.amount_within_normal_range {
            contribution += 0.15;
        }
        if self.timing_within_normal_hours {
            contribution += 0.10;
        }
        if self.counterparty_is_established {
            contribution += 0.10;
        }
        if self.account_coding_correct {
            contribution += 0.10;
        }
        if self.description_matches_pattern {
            contribution += 0.08;
        }
        if self.frequency_is_normal {
            contribution += 0.07;
        }

        contribution.min(0.40)
    }
}

/// Factors related to collusion in fraud.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollusionFactors {
    /// Number of people involved in collusion.
    pub participants: u32,
    /// Management level involvement.
    pub management_involved: bool,
    /// IT/system admin involvement.
    pub it_involved: bool,
    /// External party involvement.
    pub external_party_involved: bool,
}

impl CollusionFactors {
    /// Creates new collusion factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets number of participants.
    pub fn with_participants(mut self, count: u32) -> Self {
        self.participants = count;
        self
    }

    /// Sets management involvement.
    pub fn with_management(mut self) -> Self {
        self.management_involved = true;
        self
    }

    /// Sets IT involvement.
    pub fn with_it(mut self) -> Self {
        self.it_involved = true;
        self
    }

    /// Sets external party involvement.
    pub fn with_external_party(mut self) -> Self {
        self.external_party_involved = true;
        self
    }

    /// Calculates the total difficulty contribution from collusion.
    pub fn difficulty_contribution(&self) -> f64 {
        let mut contribution = 0.0;

        // Base contribution from participants (diminishing returns)
        if self.participants > 1 {
            contribution += (self.participants as f64 - 1.0).min(3.0) * 0.08;
        }

        if self.management_involved {
            contribution += 0.15;
        }
        if self.it_involved {
            contribution += 0.12;
        }
        if self.external_party_involved {
            contribution += 0.10;
        }

        contribution.min(0.35)
    }
}

/// Temporal factors affecting detection difficulty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporalFactors {
    /// Anomaly occurred during high-volume period.
    pub high_volume_period: bool,
    /// Anomaly occurred during staff transition.
    pub staff_transition_period: bool,
    /// Anomaly was spread across multiple periods.
    pub cross_period: bool,
    /// Time since anomaly (older = harder to investigate).
    pub days_since_anomaly: u32,
}

impl TemporalFactors {
    /// Creates new temporal factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets high volume period flag.
    pub fn with_high_volume(mut self) -> Self {
        self.high_volume_period = true;
        self
    }

    /// Sets staff transition period flag.
    pub fn with_staff_transition(mut self) -> Self {
        self.staff_transition_period = true;
        self
    }

    /// Sets cross period flag.
    pub fn with_cross_period(mut self) -> Self {
        self.cross_period = true;
        self
    }

    /// Sets days since anomaly.
    pub fn with_age(mut self, days: u32) -> Self {
        self.days_since_anomaly = days;
        self
    }

    /// Calculates the total difficulty contribution from temporal factors.
    pub fn difficulty_contribution(&self) -> f64 {
        let mut contribution = 0.0;

        if self.high_volume_period {
            contribution += 0.08;
        }
        if self.staff_transition_period {
            contribution += 0.10;
        }
        if self.cross_period {
            contribution += 0.12;
        }

        // Age factor (older = harder, with diminishing effect)
        if self.days_since_anomaly > 30 {
            contribution += ((self.days_since_anomaly as f64 - 30.0) / 365.0).min(0.15);
        }

        contribution.min(0.30)
    }
}

/// Amount-related factors affecting detection difficulty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmountFactors {
    /// Amount is close to a common/expected value.
    pub near_common_amount: bool,
    /// Amount is just below a threshold.
    pub just_below_threshold: bool,
    /// Amount represents small percentage of total activity.
    pub small_relative_percentage: bool,
    /// Standard deviations from mean (lower = harder to detect).
    pub std_deviations_from_mean: f64,
}

impl AmountFactors {
    /// Creates new amount factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets near common amount flag.
    pub fn with_common_amount(mut self) -> Self {
        self.near_common_amount = true;
        self
    }

    /// Sets just below threshold flag.
    pub fn just_below_threshold(mut self) -> Self {
        self.just_below_threshold = true;
        self
    }

    /// Sets small relative percentage flag.
    pub fn with_small_percentage(mut self) -> Self {
        self.small_relative_percentage = true;
        self
    }

    /// Sets standard deviations from mean.
    pub fn with_std_devs(mut self, std_devs: f64) -> Self {
        self.std_deviations_from_mean = std_devs;
        self
    }

    /// Calculates the total difficulty contribution from amount factors.
    pub fn difficulty_contribution(&self) -> f64 {
        let mut contribution = 0.0;

        if self.near_common_amount {
            contribution += 0.12;
        }
        if self.just_below_threshold {
            contribution += 0.05; // This actually makes it easier in some ways
        }
        if self.small_relative_percentage {
            contribution += 0.15;
        }

        // Closer to mean = harder to detect
        if self.std_deviations_from_mean < 2.0 {
            contribution += 0.10 * (2.0 - self.std_deviations_from_mean).max(0.0);
        }

        contribution.min(0.35)
    }
}

/// Combined difficulty factors for comprehensive calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifficultyFactors {
    /// Concealment techniques and methods.
    pub concealment: ConcealmentFactors,
    /// How well anomaly blends with normal activity.
    pub blending: BlendingFactors,
    /// Collusion involvement.
    pub collusion: CollusionFactors,
    /// Temporal characteristics.
    pub temporal: TemporalFactors,
    /// Amount characteristics.
    pub amount: AmountFactors,
}

impl DifficultyFactors {
    /// Creates new difficulty factors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets concealment factors.
    pub fn with_concealment(mut self, concealment: ConcealmentFactors) -> Self {
        self.concealment = concealment;
        self
    }

    /// Sets blending factors.
    pub fn with_blending(mut self, blending: BlendingFactors) -> Self {
        self.blending = blending;
        self
    }

    /// Sets collusion factors.
    pub fn with_collusion(mut self, collusion: CollusionFactors) -> Self {
        self.collusion = collusion;
        self
    }

    /// Sets temporal factors.
    pub fn with_temporal(mut self, temporal: TemporalFactors) -> Self {
        self.temporal = temporal;
        self
    }

    /// Sets amount factors.
    pub fn with_amount(mut self, amount: AmountFactors) -> Self {
        self.amount = amount;
        self
    }
}

/// Calculator for detection difficulty of anomalies.
#[derive(Debug, Clone)]
pub struct DifficultyCalculator {
    /// Base difficulty by anomaly type.
    type_base_difficulty: std::collections::HashMap<String, f64>,
}

impl Default for DifficultyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl DifficultyCalculator {
    /// Creates a new difficulty calculator with default base difficulties.
    pub fn new() -> Self {
        let mut type_base_difficulty = std::collections::HashMap::new();

        // Fraud types - generally harder to detect
        type_base_difficulty.insert("FictitiousEntry".to_string(), 0.30);
        type_base_difficulty.insert("FictitiousTransaction".to_string(), 0.30);
        type_base_difficulty.insert("FictitiousVendor".to_string(), 0.40);
        type_base_difficulty.insert("SelfApproval".to_string(), 0.15);
        type_base_difficulty.insert("SegregationOfDutiesViolation".to_string(), 0.20);
        type_base_difficulty.insert("DuplicatePayment".to_string(), 0.10);
        type_base_difficulty.insert("Kickback".to_string(), 0.50);
        type_base_difficulty.insert("KickbackScheme".to_string(), 0.50);
        type_base_difficulty.insert("RevenueManipulation".to_string(), 0.45);
        type_base_difficulty.insert("CollusiveApproval".to_string(), 0.55);

        // Error types - generally easier to detect
        type_base_difficulty.insert("DuplicateEntry".to_string(), 0.05);
        type_base_difficulty.insert("ReversedAmount".to_string(), 0.10);
        type_base_difficulty.insert("WrongPeriod".to_string(), 0.20);
        type_base_difficulty.insert("MissingField".to_string(), 0.05);
        type_base_difficulty.insert("UnbalancedEntry".to_string(), 0.03);

        // Process issues - moderate difficulty
        type_base_difficulty.insert("SkippedApproval".to_string(), 0.15);
        type_base_difficulty.insert("LatePosting".to_string(), 0.12);
        type_base_difficulty.insert("ManualOverride".to_string(), 0.25);

        // Statistical anomalies - depends on type
        type_base_difficulty.insert("UnusuallyHighAmount".to_string(), 0.15);
        type_base_difficulty.insert("BenfordViolation".to_string(), 0.25);
        type_base_difficulty.insert("TrendBreak".to_string(), 0.30);

        // Relational anomalies - often hard
        type_base_difficulty.insert("CircularTransaction".to_string(), 0.40);
        type_base_difficulty.insert("CircularIntercompany".to_string(), 0.45);

        Self {
            type_base_difficulty,
        }
    }

    /// Calculates the detection difficulty for an anomaly.
    pub fn calculate(&self, anomaly: &LabeledAnomaly) -> AnomalyDetectionDifficulty {
        let score = self.compute_difficulty_score(anomaly, &DifficultyFactors::default());
        AnomalyDetectionDifficulty::from_score(score)
    }

    /// Calculates difficulty with additional context factors.
    pub fn calculate_with_factors(
        &self,
        anomaly: &LabeledAnomaly,
        factors: &DifficultyFactors,
    ) -> AnomalyDetectionDifficulty {
        let score = self.compute_difficulty_score(anomaly, factors);
        AnomalyDetectionDifficulty::from_score(score)
    }

    /// Computes the raw difficulty score (0.0-1.0).
    pub fn compute_difficulty_score(
        &self,
        anomaly: &LabeledAnomaly,
        factors: &DifficultyFactors,
    ) -> f64 {
        // Get base difficulty from anomaly type
        let type_name = anomaly.anomaly_type.type_name();
        let base_difficulty = *self.type_base_difficulty.get(&type_name).unwrap_or(&0.25);

        // Add factor contributions
        let concealment_contribution = factors.concealment.difficulty_contribution();
        let blending_contribution = factors.blending.difficulty_contribution();
        let collusion_contribution = factors.collusion.difficulty_contribution();
        let temporal_contribution = factors.temporal.difficulty_contribution();
        let amount_contribution = factors.amount.difficulty_contribution();

        // Combine with weighted average (base has most weight)
        let total_contribution = concealment_contribution
            + blending_contribution
            + collusion_contribution
            + temporal_contribution
            + amount_contribution;

        // Base difficulty contributes 40%, factors contribute 60%
        let score = base_difficulty * 0.4 + total_contribution * 0.6;

        // Ensure score is in valid range
        score.clamp(0.0, 1.0)
    }

    /// Returns recommended detection methods for a difficulty level.
    pub fn recommended_methods(
        &self,
        difficulty: AnomalyDetectionDifficulty,
    ) -> Vec<DetectionMethod> {
        match difficulty {
            AnomalyDetectionDifficulty::Trivial => vec![DetectionMethod::RuleBased],
            AnomalyDetectionDifficulty::Easy => {
                vec![DetectionMethod::RuleBased, DetectionMethod::Statistical]
            }
            AnomalyDetectionDifficulty::Moderate => vec![
                DetectionMethod::Statistical,
                DetectionMethod::MachineLearning,
            ],
            AnomalyDetectionDifficulty::Hard => vec![
                DetectionMethod::MachineLearning,
                DetectionMethod::GraphBased,
            ],
            AnomalyDetectionDifficulty::Expert => vec![
                DetectionMethod::GraphBased,
                DetectionMethod::ForensicAudit,
                DetectionMethod::Hybrid,
            ],
        }
    }

    /// Infers difficulty factors from an anomaly's metadata.
    pub fn infer_factors(&self, anomaly: &LabeledAnomaly) -> DifficultyFactors {
        let mut factors = DifficultyFactors::default();

        // Infer from anomaly type
        match &anomaly.anomaly_type {
            AnomalyType::Fraud(fraud_type) => {
                // Collusion-related fraud types
                if matches!(
                    fraud_type,
                    FraudType::CollusiveApproval | FraudType::KickbackScheme | FraudType::Kickback
                ) {
                    factors.collusion = factors.collusion.with_participants(2);
                }

                // Document manipulation types
                if matches!(
                    fraud_type,
                    FraudType::FictitiousEntry
                        | FraudType::FictitiousVendor
                        | FraudType::InvoiceManipulation
                ) {
                    factors.concealment = factors.concealment.with_document_manipulation();
                }

                // Threshold-related types
                if matches!(
                    fraud_type,
                    FraudType::JustBelowThreshold | FraudType::SplitTransaction
                ) {
                    factors.concealment = factors.concealment.with_splitting();
                    factors.amount = factors.amount.just_below_threshold();
                }

                // Timing-related types
                if matches!(fraud_type, FraudType::TimingAnomaly) {
                    factors.concealment = factors.concealment.with_timing_exploitation();
                }
            }
            AnomalyType::Error(_) => {
                // Errors are generally not concealed
            }
            AnomalyType::ProcessIssue(process_type) => {
                use datasynth_core::models::ProcessIssueType;
                if matches!(process_type, ProcessIssueType::SkippedApproval) {
                    factors.concealment = factors.concealment.with_approval_circumvention();
                }
                if matches!(
                    process_type,
                    ProcessIssueType::AfterHoursPosting | ProcessIssueType::WeekendPosting
                ) {
                    factors.concealment = factors.concealment.with_timing_exploitation();
                }
            }
            _ => {}
        }

        // Check metadata for additional hints
        if anomaly.metadata.contains_key("collusion") {
            factors.collusion = factors.collusion.with_participants(2);
        }
        if anomaly.metadata.contains_key("management_override") {
            factors.collusion = factors.collusion.with_management();
        }

        factors
    }
}

/// Result of difficulty calculation with full breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAssessment {
    /// Overall difficulty level.
    pub difficulty: AnomalyDetectionDifficulty,
    /// Raw difficulty score (0.0-1.0).
    pub score: f64,
    /// Factors used in calculation.
    pub factors: DifficultyFactors,
    /// Recommended detection methods.
    pub recommended_methods: Vec<DetectionMethod>,
    /// Expected detection rate.
    pub expected_detection_rate: f64,
    /// Key indicators that make this detectable.
    pub key_indicators: Vec<String>,
}

impl DifficultyAssessment {
    /// Creates a new difficulty assessment.
    pub fn new(
        difficulty: AnomalyDetectionDifficulty,
        score: f64,
        factors: DifficultyFactors,
        methods: Vec<DetectionMethod>,
    ) -> Self {
        Self {
            expected_detection_rate: difficulty.expected_detection_rate(),
            difficulty,
            score,
            factors,
            recommended_methods: methods,
            key_indicators: Vec::new(),
        }
    }

    /// Adds a key indicator.
    pub fn with_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.key_indicators.push(indicator.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::ErrorType;

    fn create_test_anomaly(anomaly_type: AnomalyType) -> LabeledAnomaly {
        LabeledAnomaly::new(
            "ANO001".to_string(),
            anomaly_type,
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        )
    }

    #[test]
    fn test_concealment_factors() {
        let factors = ConcealmentFactors::new()
            .with_document_manipulation()
            .with_splitting();

        let contribution = factors.difficulty_contribution();
        assert!(contribution > 0.3);
        assert!(contribution <= 0.5);
    }

    #[test]
    fn test_blending_factors() {
        let factors = BlendingFactors::new()
            .with_normal_amount()
            .with_normal_timing()
            .with_established_counterparty();

        let contribution = factors.difficulty_contribution();
        assert!(contribution > 0.3);
        assert!(contribution <= 0.4);
    }

    #[test]
    fn test_collusion_factors() {
        let factors = CollusionFactors::new()
            .with_participants(3)
            .with_management();

        let contribution = factors.difficulty_contribution();
        assert!(contribution > 0.3);
    }

    #[test]
    fn test_difficulty_calculator_basic() {
        let calculator = DifficultyCalculator::new();

        // Easy to detect error
        let error_anomaly = create_test_anomaly(AnomalyType::Error(ErrorType::DuplicateEntry));
        let difficulty = calculator.calculate(&error_anomaly);
        assert!(matches!(
            difficulty,
            AnomalyDetectionDifficulty::Trivial | AnomalyDetectionDifficulty::Easy
        ));

        // Fraud without concealment factors - base difficulty gets scaled down
        // Base difficulty of 0.50 * 0.4 = 0.20, which maps to Easy
        let fraud_anomaly = create_test_anomaly(AnomalyType::Fraud(FraudType::KickbackScheme));
        let difficulty = calculator.calculate(&fraud_anomaly);
        assert!(matches!(
            difficulty,
            AnomalyDetectionDifficulty::Easy | AnomalyDetectionDifficulty::Moderate
        ));
    }

    #[test]
    fn test_difficulty_with_factors() {
        let calculator = DifficultyCalculator::new();
        let anomaly = create_test_anomaly(AnomalyType::Fraud(FraudType::FictitiousVendor));

        // Without factors
        let base_difficulty = calculator.calculate(&anomaly);

        // With concealment and collusion factors
        let factors = DifficultyFactors::new()
            .with_concealment(
                ConcealmentFactors::new()
                    .with_document_manipulation()
                    .with_technique(ConcealmentTechnique::Collusion),
            )
            .with_collusion(
                CollusionFactors::new()
                    .with_participants(2)
                    .with_management(),
            );

        let enhanced_difficulty = calculator.calculate_with_factors(&anomaly, &factors);

        // Enhanced difficulty should be higher
        assert!(enhanced_difficulty.difficulty_score() >= base_difficulty.difficulty_score());
    }

    #[test]
    fn test_recommended_methods() {
        let calculator = DifficultyCalculator::new();

        let trivial_methods = calculator.recommended_methods(AnomalyDetectionDifficulty::Trivial);
        assert!(trivial_methods.contains(&DetectionMethod::RuleBased));

        let expert_methods = calculator.recommended_methods(AnomalyDetectionDifficulty::Expert);
        assert!(expert_methods.contains(&DetectionMethod::ForensicAudit));
    }

    #[test]
    fn test_infer_factors() {
        let calculator = DifficultyCalculator::new();

        let kickback = create_test_anomaly(AnomalyType::Fraud(FraudType::KickbackScheme));
        let factors = calculator.infer_factors(&kickback);
        assert!(factors.collusion.participants >= 2);

        let fictitious = create_test_anomaly(AnomalyType::Fraud(FraudType::FictitiousEntry));
        let factors = calculator.infer_factors(&fictitious);
        assert!(factors.concealment.document_manipulation);
    }

    #[test]
    fn test_difficulty_assessment() {
        let assessment = DifficultyAssessment::new(
            AnomalyDetectionDifficulty::Hard,
            0.72,
            DifficultyFactors::default(),
            vec![
                DetectionMethod::GraphBased,
                DetectionMethod::MachineLearning,
            ],
        )
        .with_indicator("Complex vendor network")
        .with_indicator("Cross-entity payments");

        assert_eq!(assessment.difficulty, AnomalyDetectionDifficulty::Hard);
        assert_eq!(assessment.key_indicators.len(), 2);
        assert!((assessment.expected_detection_rate - 0.40).abs() < 0.01);
    }
}
