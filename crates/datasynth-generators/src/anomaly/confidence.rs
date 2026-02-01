//! Confidence calculation for anomaly detection.
//!
//! This module provides dynamic confidence scoring based on multiple factors:
//! - Pattern clarity (how clear is the anomalous pattern)
//! - Anomaly strength (magnitude of deviation)
//! - Detectability (automated detection likelihood)
//! - Context match (supporting evidence)

use datasynth_core::models::{
    AnomalyType, ContributingFactor, ErrorType, FactorType, FraudType, ProcessIssueType,
    RelationalAnomalyType, StatisticalAnomalyType,
};
use rust_decimal::Decimal;

/// Configuration for confidence calculation.
#[derive(Debug, Clone)]
pub struct ConfidenceConfig {
    /// Weight for pattern clarity component.
    pub pattern_clarity_weight: f64,
    /// Weight for anomaly strength component.
    pub strength_weight: f64,
    /// Weight for detectability component.
    pub detectability_weight: f64,
    /// Weight for context match component.
    pub context_weight: f64,
    /// Materiality threshold for amount-based anomalies.
    pub materiality_threshold: Decimal,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            pattern_clarity_weight: 0.30,
            strength_weight: 0.25,
            detectability_weight: 0.25,
            context_weight: 0.20,
            materiality_threshold: Decimal::new(10000, 0), // 10,000
        }
    }
}

impl ConfidenceConfig {
    /// Validates that weights sum to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.pattern_clarity_weight
            + self.strength_weight
            + self.detectability_weight
            + self.context_weight;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!("Confidence weights must sum to 1.0, got {}", sum));
        }

        Ok(())
    }
}

/// Context for confidence calculation.
#[derive(Debug, Clone, Default)]
pub struct ConfidenceContext {
    /// Amount involved in the anomaly.
    pub amount: Option<Decimal>,
    /// Normal/expected amount for comparison.
    pub expected_amount: Option<Decimal>,
    /// Number of similar anomalies previously detected.
    pub prior_anomaly_count: usize,
    /// Entity risk score (0.0 - 1.0).
    pub entity_risk_score: f64,
    /// Whether the anomaly was detected by automated rules.
    pub auto_detected: bool,
    /// Number of supporting evidence items.
    pub evidence_count: usize,
    /// Pattern match confidence (0.0 - 1.0).
    pub pattern_confidence: f64,
    /// Time-based anomaly indicators.
    pub timing_score: f64,
}

/// Calculator for anomaly confidence scores.
#[derive(Debug, Clone)]
pub struct ConfidenceCalculator {
    config: ConfidenceConfig,
}

impl ConfidenceCalculator {
    /// Creates a new confidence calculator with default config.
    pub fn new() -> Self {
        Self {
            config: ConfidenceConfig::default(),
        }
    }

    /// Creates a new confidence calculator with custom config.
    pub fn with_config(config: ConfidenceConfig) -> Self {
        Self { config }
    }

    /// Calculates confidence score for an anomaly.
    ///
    /// Returns a tuple of (confidence_score, contributing_factors).
    pub fn calculate(
        &self,
        anomaly_type: &AnomalyType,
        context: &ConfidenceContext,
    ) -> (f64, Vec<ContributingFactor>) {
        let mut factors = Vec::new();

        // Component 1: Pattern Clarity
        let pattern_clarity = self.calculate_pattern_clarity(anomaly_type, context);
        factors.push(ContributingFactor::new(
            FactorType::PatternMatch,
            pattern_clarity,
            0.5, // Threshold for "clear" pattern
            true,
            self.config.pattern_clarity_weight,
            &format!("Pattern clarity score: {:.2}", pattern_clarity),
        ));

        // Component 2: Anomaly Strength
        let strength = self.calculate_anomaly_strength(anomaly_type, context);
        factors.push(ContributingFactor::new(
            FactorType::AmountDeviation,
            strength,
            0.3, // Threshold for "strong" anomaly
            true,
            self.config.strength_weight,
            &format!("Anomaly strength: {:.2}", strength),
        ));

        // Component 3: Detectability
        let detectability = self.calculate_detectability(anomaly_type, context);
        factors.push(ContributingFactor::new(
            FactorType::PatternMatch,
            detectability,
            0.5,
            true,
            self.config.detectability_weight,
            &format!("Auto-detectability: {:.2}", detectability),
        ));

        // Component 4: Context Match
        let context_match = self.calculate_context_match(context);
        factors.push(ContributingFactor::new(
            FactorType::EntityRisk,
            context_match,
            0.3,
            true,
            self.config.context_weight,
            &format!("Context match score: {:.2}", context_match),
        ));

        // Calculate weighted sum
        let confidence = pattern_clarity * self.config.pattern_clarity_weight
            + strength * self.config.strength_weight
            + detectability * self.config.detectability_weight
            + context_match * self.config.context_weight;

        (confidence.clamp(0.0, 1.0), factors)
    }

    /// Calculates pattern clarity based on anomaly type.
    fn calculate_pattern_clarity(
        &self,
        anomaly_type: &AnomalyType,
        context: &ConfidenceContext,
    ) -> f64 {
        // Base clarity from anomaly type
        let base_clarity = match anomaly_type {
            AnomalyType::Fraud(fraud_type) => match fraud_type {
                FraudType::DuplicatePayment => 0.95, // Very clear pattern
                FraudType::SelfApproval => 0.90,
                FraudType::SegregationOfDutiesViolation => 0.85,
                FraudType::JustBelowThreshold => 0.80,
                FraudType::RoundDollarManipulation => 0.70,
                FraudType::FictitiousVendor => 0.60, // Requires investigation
                FraudType::CollusiveApproval => 0.50, // Hard to detect
                _ => 0.65,
            },
            AnomalyType::Error(error_type) => match error_type {
                ErrorType::DuplicateEntry => 0.95,
                ErrorType::ReversedAmount => 0.90,
                ErrorType::UnbalancedEntry => 0.95,
                ErrorType::MissingField => 0.85,
                _ => 0.75,
            },
            AnomalyType::ProcessIssue(process_type) => match process_type {
                ProcessIssueType::SkippedApproval => 0.90,
                ProcessIssueType::MissingDocumentation => 0.85,
                ProcessIssueType::ManualOverride => 0.80,
                _ => 0.70,
            },
            AnomalyType::Statistical(stat_type) => match stat_type {
                StatisticalAnomalyType::BenfordViolation => 0.75,
                StatisticalAnomalyType::StatisticalOutlier => 0.70,
                StatisticalAnomalyType::UnusuallyHighAmount => 0.65,
                _ => 0.60,
            },
            AnomalyType::Relational(rel_type) => match rel_type {
                RelationalAnomalyType::CircularTransaction => 0.85,
                RelationalAnomalyType::DormantAccountActivity => 0.80,
                _ => 0.65,
            },
            AnomalyType::Custom(_) => 0.50,
        };

        // Adjust based on pattern confidence from context
        let adjusted = base_clarity * 0.7 + context.pattern_confidence * 0.3;

        adjusted.clamp(0.0, 1.0)
    }

    /// Calculates anomaly strength based on deviation magnitude.
    fn calculate_anomaly_strength(
        &self,
        anomaly_type: &AnomalyType,
        context: &ConfidenceContext,
    ) -> f64 {
        // Amount-based strength
        let amount_strength =
            if let (Some(amount), Some(expected)) = (context.amount, context.expected_amount) {
                let deviation = (amount - expected).abs();
                let expected_f64: f64 = expected.try_into().unwrap_or(1.0);
                let deviation_f64: f64 = deviation.try_into().unwrap_or(0.0);

                if expected_f64.abs() > 0.01 {
                    (deviation_f64 / expected_f64.abs()).min(2.0) / 2.0 // Normalize to [0, 1]
                } else {
                    0.5
                }
            } else {
                0.5 // Default when no amount context
            };

        // Type-based strength modifier
        let type_modifier = match anomaly_type {
            AnomalyType::Fraud(_) => 1.2, // Fraud is inherently severe
            AnomalyType::Statistical(_) => 1.0,
            AnomalyType::Relational(_) => 1.1,
            AnomalyType::Error(_) => 0.9,
            AnomalyType::ProcessIssue(_) => 0.85,
            AnomalyType::Custom(_) => 1.0,
        };

        (amount_strength * type_modifier).clamp(0.0, 1.0)
    }

    /// Calculates detectability based on anomaly type and context.
    fn calculate_detectability(
        &self,
        anomaly_type: &AnomalyType,
        context: &ConfidenceContext,
    ) -> f64 {
        // Base detectability from anomaly type
        let base_detectability = match anomaly_type {
            AnomalyType::Error(error_type) => match error_type {
                ErrorType::UnbalancedEntry => 1.0, // Always detected
                ErrorType::DuplicateEntry => 0.95,
                ErrorType::MissingField => 0.90,
                _ => 0.80,
            },
            AnomalyType::Fraud(fraud_type) => match fraud_type {
                FraudType::DuplicatePayment => 0.90,
                FraudType::SelfApproval => 0.85,
                FraudType::JustBelowThreshold => 0.75,
                FraudType::CollusiveApproval => 0.40, // Hard to auto-detect
                FraudType::FictitiousVendor => 0.45,
                _ => 0.60,
            },
            AnomalyType::ProcessIssue(_) => 0.70,
            AnomalyType::Statistical(_) => 0.65,
            AnomalyType::Relational(_) => 0.55,
            AnomalyType::Custom(_) => 0.50,
        };

        // Boost if already auto-detected
        let auto_detect_boost: f64 = if context.auto_detected { 0.2 } else { 0.0 };

        (base_detectability + auto_detect_boost).clamp(0.0, 1.0)
    }

    /// Calculates context match score.
    fn calculate_context_match(&self, context: &ConfidenceContext) -> f64 {
        let mut score = 0.0;

        // Entity risk contribution
        score += context.entity_risk_score * 0.4;

        // Prior anomaly count contribution (repeat offenders)
        let prior_contribution = (context.prior_anomaly_count as f64 / 5.0).min(1.0) * 0.3;
        score += prior_contribution;

        // Evidence count contribution
        let evidence_contribution = (context.evidence_count as f64 / 3.0).min(1.0) * 0.2;
        score += evidence_contribution;

        // Timing score contribution
        score += context.timing_score * 0.1;

        score.clamp(0.0, 1.0)
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_confidence_calculator_basic() {
        let calculator = ConfidenceCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::DuplicatePayment);
        let context = ConfidenceContext::default();

        let (confidence, factors) = calculator.calculate(&anomaly_type, &context);

        assert!((0.0..=1.0).contains(&confidence));
        assert!(!factors.is_empty());
    }

    #[test]
    fn test_confidence_with_amount_context() {
        let calculator = ConfidenceCalculator::new();
        let anomaly_type = AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount);

        let context = ConfidenceContext {
            amount: Some(dec!(100000)),
            expected_amount: Some(dec!(10000)),
            ..Default::default()
        };

        let (confidence, _) = calculator.calculate(&anomaly_type, &context);

        // High deviation should increase confidence
        assert!(confidence > 0.3);
    }

    #[test]
    fn test_confidence_with_entity_risk() {
        let calculator = ConfidenceCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::FictitiousVendor);

        let low_risk_context = ConfidenceContext {
            entity_risk_score: 0.1,
            ..Default::default()
        };

        let high_risk_context = ConfidenceContext {
            entity_risk_score: 0.9,
            prior_anomaly_count: 5,
            ..Default::default()
        };

        let (low_confidence, _) = calculator.calculate(&anomaly_type, &low_risk_context);
        let (high_confidence, _) = calculator.calculate(&anomaly_type, &high_risk_context);

        // High risk entity should have higher confidence
        assert!(high_confidence > low_confidence);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = ConfidenceConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = ConfidenceConfig {
            pattern_clarity_weight: 0.5,
            strength_weight: 0.5,
            detectability_weight: 0.5,
            context_weight: 0.5, // Sum = 2.0
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_auto_detected_boost() {
        let calculator = ConfidenceCalculator::new();
        let anomaly_type = AnomalyType::Error(ErrorType::DuplicateEntry);

        let not_detected = ConfidenceContext {
            auto_detected: false,
            ..Default::default()
        };

        let detected = ConfidenceContext {
            auto_detected: true,
            ..Default::default()
        };

        let (conf_not, _) = calculator.calculate(&anomaly_type, &not_detected);
        let (conf_detected, _) = calculator.calculate(&anomaly_type, &detected);

        // Auto-detected should have higher confidence
        assert!(conf_detected > conf_not);
    }
}
