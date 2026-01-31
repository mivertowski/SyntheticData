//! Anomaly co-occurrence patterns.
//!
//! Defines patterns where certain anomalies tend to appear together,
//! such as fraud concealment patterns where a fictitious vendor
//! is typically accompanied by document manipulation and approval bypass.

use rand::Rng;
use serde::{Deserialize, Serialize};

use datasynth_core::models::AnomalyType;

/// A correlated anomaly that tends to occur with a primary anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedAnomaly {
    /// The correlated anomaly type.
    pub anomaly_type: AnomalyType,
    /// Probability of this anomaly occurring given the primary (0.0-1.0).
    pub probability: f64,
    /// Minimum lag in days from the primary anomaly.
    pub lag_days_min: i32,
    /// Maximum lag in days from the primary anomaly.
    pub lag_days_max: i32,
    /// Whether this anomaly targets the same entity.
    pub same_entity: bool,
    /// Description of the correlation.
    pub description: String,
}

impl CorrelatedAnomaly {
    /// Creates a new correlated anomaly.
    pub fn new(
        anomaly_type: AnomalyType,
        probability: f64,
        lag_range: (i32, i32),
    ) -> Self {
        Self {
            anomaly_type,
            probability: probability.clamp(0.0, 1.0),
            lag_days_min: lag_range.0,
            lag_days_max: lag_range.1,
            same_entity: true,
            description: String::new(),
        }
    }

    /// Sets whether the correlated anomaly targets the same entity.
    pub fn with_same_entity(mut self, same: bool) -> Self {
        self.same_entity = same;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Generates a random lag within the range.
    pub fn random_lag<R: Rng>(&self, rng: &mut R) -> i32 {
        if self.lag_days_min == self.lag_days_max {
            return self.lag_days_min;
        }
        rng.gen_range(self.lag_days_min..=self.lag_days_max)
    }

    /// Returns whether this anomaly should be triggered.
    pub fn should_trigger<R: Rng>(&self, rng: &mut R) -> bool {
        rng.gen::<f64>() < self.probability
    }
}

/// A co-occurrence pattern defining which anomalies tend to appear together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoOccurrencePattern {
    /// Name of the pattern.
    pub name: String,
    /// Description of when this pattern applies.
    pub description: String,
    /// The primary/triggering anomaly type.
    pub primary: AnomalyType,
    /// Correlated anomalies that may occur with the primary.
    pub correlated: Vec<CorrelatedAnomaly>,
    /// Whether this pattern is currently active.
    pub enabled: bool,
}

impl CoOccurrencePattern {
    /// Creates a new co-occurrence pattern.
    pub fn new(name: impl Into<String>, primary: AnomalyType) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            primary,
            correlated: Vec::new(),
            enabled: true,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a correlated anomaly.
    pub fn with_correlated(mut self, correlated: CorrelatedAnomaly) -> Self {
        self.correlated.push(correlated);
        self
    }

    /// Sets whether the pattern is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Checks if this pattern matches a given anomaly type.
    pub fn matches(&self, anomaly_type: &AnomalyType) -> bool {
        self.enabled && self.primary == *anomaly_type
    }

    /// Gets correlated anomalies that should be triggered.
    pub fn get_triggered_correlations<R: Rng>(&self, rng: &mut R) -> Vec<&CorrelatedAnomaly> {
        self.correlated
            .iter()
            .filter(|c| c.should_trigger(rng))
            .collect()
    }
}

/// Manages co-occurrence patterns for anomaly injection.
#[derive(Debug, Clone)]
pub struct AnomalyCoOccurrence {
    /// All registered patterns.
    patterns: Vec<CoOccurrencePattern>,
}

impl Default for AnomalyCoOccurrence {
    fn default() -> Self {
        Self::new()
    }
}

impl AnomalyCoOccurrence {
    /// Creates a new co-occurrence manager with default patterns.
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Creates default fraud-related co-occurrence patterns.
    fn default_patterns() -> Vec<CoOccurrencePattern> {
        use datasynth_core::models::{ErrorType, FraudType, ProcessIssueType};

        vec![
            // Fraud concealment pattern
            CoOccurrencePattern::new(
                "fraud_concealment",
                AnomalyType::Fraud(FraudType::FictitiousVendor),
            )
            .with_description("Fictitious vendor fraud typically involves document manipulation and approval bypass")
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::InvoiceManipulation),
                    0.80,
                    (0, 30),
                )
                .with_description("Document manipulation to support fictitious vendor"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval),
                    0.60,
                    (0, 15),
                )
                .with_description("Approval bypass to expedite fraudulent payments"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::DuplicatePayment),
                    0.30,
                    (15, 60),
                )
                .with_same_entity(true)
                .with_description("Multiple payments to the fictitious vendor"),
            ),

            // Error cascade pattern
            CoOccurrencePattern::new(
                "error_cascade",
                AnomalyType::Error(ErrorType::MisclassifiedAccount),
            )
            .with_description("Account misclassification leads to reconciliation issues and corrections")
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Error(ErrorType::DuplicateEntry),
                    0.40,
                    (1, 10),
                )
                .with_description("Attempt to correct misclassification creates duplicate"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Error(ErrorType::WrongPeriod),
                    0.30,
                    (5, 30),
                )
                .with_description("Correction posted to wrong period"),
            ),

            // Process breakdown pattern
            CoOccurrencePattern::new(
                "process_breakdown",
                AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval),
            )
            .with_description("Skipped approvals often accompanied by other control bypasses")
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::SplitTransaction),
                    0.50,
                    (0, 7),
                )
                .with_description("Transaction splitting to avoid threshold"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
                    0.40,
                    (0, 5),
                )
                .with_description("Late posting to avoid immediate detection"),
            ),

            // Kickback concealment pattern
            CoOccurrencePattern::new(
                "kickback_concealment",
                AnomalyType::Fraud(FraudType::Kickback),
            )
            .with_description("Kickback schemes involve price inflation and approval manipulation")
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::InvoiceManipulation),
                    0.85,
                    (0, 14),
                )
                .with_description("Invoice price inflation"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::SegregationOfDutiesViolation),
                    0.45,
                    (0, 30),
                )
                .with_description("SoD violation to approve own vendor"),
            ),

            // Revenue manipulation concealment
            CoOccurrencePattern::new(
                "revenue_manipulation_concealment",
                AnomalyType::Fraud(FraudType::RevenueManipulation),
            )
            .with_description("Revenue manipulation often involves expense deferral and reserve manipulation")
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::ImproperCapitalization),
                    0.60,
                    (0, 30),
                )
                .with_description("Capitalize expenses to boost current period income"),
            )
            .with_correlated(
                CorrelatedAnomaly::new(
                    AnomalyType::Fraud(FraudType::ReserveManipulation),
                    0.50,
                    (30, 90),
                )
                .with_description("Release reserves to meet targets"),
            ),
        ]
    }

    /// Adds a custom pattern.
    pub fn add_pattern(&mut self, pattern: CoOccurrencePattern) {
        self.patterns.push(pattern);
    }

    /// Gets patterns that match a given anomaly type.
    pub fn get_matching_patterns(&self, anomaly_type: &AnomalyType) -> Vec<&CoOccurrencePattern> {
        self.patterns
            .iter()
            .filter(|p| p.matches(anomaly_type))
            .collect()
    }

    /// Gets correlated anomalies for a given primary anomaly.
    pub fn get_correlated_anomalies<R: Rng>(
        &self,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> Vec<CorrelatedAnomalyResult> {
        let mut results = Vec::new();

        for pattern in self.get_matching_patterns(anomaly_type) {
            for correlated in pattern.get_triggered_correlations(rng) {
                let lag = correlated.random_lag(rng);
                results.push(CorrelatedAnomalyResult {
                    pattern_name: pattern.name.clone(),
                    anomaly_type: correlated.anomaly_type.clone(),
                    lag_days: lag,
                    same_entity: correlated.same_entity,
                    description: correlated.description.clone(),
                });
            }
        }

        results
    }

    /// Returns all registered patterns.
    pub fn patterns(&self) -> &[CoOccurrencePattern] {
        &self.patterns
    }

    /// Enables or disables a pattern by name.
    pub fn set_pattern_enabled(&mut self, name: &str, enabled: bool) {
        for pattern in &mut self.patterns {
            if pattern.name == name {
                pattern.enabled = enabled;
                break;
            }
        }
    }
}

/// Result of a correlated anomaly check.
#[derive(Debug, Clone)]
pub struct CorrelatedAnomalyResult {
    /// Pattern that triggered this.
    pub pattern_name: String,
    /// Anomaly type to inject.
    pub anomaly_type: AnomalyType,
    /// Days after the primary anomaly.
    pub lag_days: i32,
    /// Whether to target the same entity.
    pub same_entity: bool,
    /// Description of the correlation.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::models::FraudType;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_correlated_anomaly() {
        let correlated = CorrelatedAnomaly::new(
            AnomalyType::Fraud(FraudType::InvoiceManipulation),
            0.80,
            (0, 30),
        )
        .with_description("Test correlation");

        assert!((correlated.probability - 0.80).abs() < 0.01);
        assert_eq!(correlated.lag_days_min, 0);
        assert_eq!(correlated.lag_days_max, 30);
    }

    #[test]
    fn test_correlated_anomaly_trigger() {
        let correlated = CorrelatedAnomaly::new(
            AnomalyType::Fraud(FraudType::InvoiceManipulation),
            1.0, // Always triggers
            (0, 0),
        );

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        assert!(correlated.should_trigger(&mut rng));
    }

    #[test]
    fn test_co_occurrence_pattern() {
        let pattern = CoOccurrencePattern::new(
            "test_pattern",
            AnomalyType::Fraud(FraudType::FictitiousVendor),
        )
        .with_correlated(CorrelatedAnomaly::new(
            AnomalyType::Fraud(FraudType::InvoiceManipulation),
            0.80,
            (0, 30),
        ));

        assert!(pattern.matches(&AnomalyType::Fraud(FraudType::FictitiousVendor)));
        assert!(!pattern.matches(&AnomalyType::Fraud(FraudType::DuplicatePayment)));
    }

    #[test]
    fn test_anomaly_co_occurrence() {
        let co_occurrence = AnomalyCoOccurrence::new();
        assert!(!co_occurrence.patterns().is_empty());

        // Check that fraud_concealment pattern exists
        let patterns =
            co_occurrence.get_matching_patterns(&AnomalyType::Fraud(FraudType::FictitiousVendor));
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_get_correlated_anomalies() {
        let co_occurrence = AnomalyCoOccurrence::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // FictitiousVendor should trigger correlated anomalies
        let results = co_occurrence
            .get_correlated_anomalies(&AnomalyType::Fraud(FraudType::FictitiousVendor), &mut rng);

        // With high probabilities, should get some results
        // (depends on RNG, but fraud_concealment has 0.80 probability correlations)
        // Note: This is probabilistic, so we just check it doesn't panic
        assert!(results.len() <= 4); // Max 4 correlations in default pattern
    }

    #[test]
    fn test_pattern_enable_disable() {
        let mut co_occurrence = AnomalyCoOccurrence::new();

        co_occurrence.set_pattern_enabled("fraud_concealment", false);

        let patterns =
            co_occurrence.get_matching_patterns(&AnomalyType::Fraud(FraudType::FictitiousVendor));
        assert!(patterns.is_empty());

        co_occurrence.set_pattern_enabled("fraud_concealment", true);

        let patterns =
            co_occurrence.get_matching_patterns(&AnomalyType::Fraud(FraudType::FictitiousVendor));
        assert!(!patterns.is_empty());
    }
}
