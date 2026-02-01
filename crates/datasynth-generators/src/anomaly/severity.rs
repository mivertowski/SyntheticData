//! Severity calculation for anomaly detection.
//!
//! This module provides contextual severity scoring based on:
//! - Base type severity (static from AnomalyType)
//! - Monetary impact (normalized by materiality)
//! - Frequency factor (repeated anomalies = higher severity)
//! - Scope factor (number of affected entities)
//! - Timing factor (period-end = higher severity)

use chrono::{Datelike, NaiveDate};
use datasynth_core::models::{
    AnomalyType, ContributingFactor, ErrorType, FactorType, FraudType, ProcessIssueType,
    RelationalAnomalyType,
};
use rust_decimal::Decimal;

/// Configuration for severity calculation.
#[derive(Debug, Clone)]
pub struct SeverityConfig {
    /// Weight for base type severity component.
    pub base_type_weight: f64,
    /// Weight for monetary impact component.
    pub monetary_weight: f64,
    /// Weight for frequency factor component.
    pub frequency_weight: f64,
    /// Weight for scope factor component.
    pub scope_weight: f64,
    /// Weight for timing factor component.
    pub timing_weight: f64,
    /// Materiality threshold for monetary impact normalization.
    pub materiality_threshold: Decimal,
    /// Number of anomalies considered "high frequency".
    pub high_frequency_threshold: usize,
    /// Number of entities considered "broad scope".
    pub broad_scope_threshold: usize,
}

impl Default for SeverityConfig {
    fn default() -> Self {
        Self {
            base_type_weight: 0.25,
            monetary_weight: 0.30,
            frequency_weight: 0.20,
            scope_weight: 0.15,
            timing_weight: 0.10,
            materiality_threshold: Decimal::new(10000, 0), // 10,000
            high_frequency_threshold: 5,
            broad_scope_threshold: 3,
        }
    }
}

impl SeverityConfig {
    /// Validates that weights sum to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.base_type_weight
            + self.monetary_weight
            + self.frequency_weight
            + self.scope_weight
            + self.timing_weight;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!("Severity weights must sum to 1.0, got {}", sum));
        }

        Ok(())
    }
}

/// Context for severity calculation.
#[derive(Debug, Clone)]
pub struct SeverityContext {
    /// Monetary impact of the anomaly.
    pub monetary_impact: Option<Decimal>,
    /// Number of times this anomaly type has occurred.
    pub occurrence_count: usize,
    /// Number of entities affected.
    pub affected_entity_count: usize,
    /// Date of the anomaly.
    pub anomaly_date: Option<NaiveDate>,
    /// Is this a month-end period.
    pub is_month_end: bool,
    /// Is this a quarter-end period.
    pub is_quarter_end: bool,
    /// Is this a year-end period.
    pub is_year_end: bool,
    /// Is this during an audit period.
    pub is_audit_period: bool,
    /// Custom severity modifier (multiplier).
    pub custom_modifier: f64,
}

impl Default for SeverityContext {
    fn default() -> Self {
        Self {
            monetary_impact: None,
            occurrence_count: 0,
            affected_entity_count: 0,
            anomaly_date: None,
            is_month_end: false,
            is_quarter_end: false,
            is_year_end: false,
            is_audit_period: false,
            custom_modifier: 1.0, // Multiplier should default to 1.0, not 0.0
        }
    }
}

impl SeverityContext {
    /// Creates context from a date, auto-detecting period-end flags.
    pub fn from_date(date: NaiveDate) -> Self {
        let day = date.day();
        let month = date.month();

        let is_month_end = day >= 28;
        let is_quarter_end = is_month_end && matches!(month, 3 | 6 | 9 | 12);
        let is_year_end = month == 12 && day >= 28;

        Self {
            anomaly_date: Some(date),
            is_month_end,
            is_quarter_end,
            is_year_end,
            custom_modifier: 1.0,
            ..Default::default()
        }
    }
}

/// Calculator for anomaly severity scores.
#[derive(Debug, Clone)]
pub struct SeverityCalculator {
    config: SeverityConfig,
}

impl SeverityCalculator {
    /// Creates a new severity calculator with default config.
    pub fn new() -> Self {
        Self {
            config: SeverityConfig::default(),
        }
    }

    /// Creates a new severity calculator with custom config.
    pub fn with_config(config: SeverityConfig) -> Self {
        Self { config }
    }

    /// Calculates severity score for an anomaly.
    ///
    /// Returns a tuple of (severity_score, contributing_factors).
    pub fn calculate(
        &self,
        anomaly_type: &AnomalyType,
        context: &SeverityContext,
    ) -> (f64, Vec<ContributingFactor>) {
        let mut factors = Vec::new();

        // Component 1: Base Type Severity
        let base_severity = self.calculate_base_severity(anomaly_type);
        factors.push(ContributingFactor::new(
            FactorType::PatternMatch,
            base_severity,
            0.5,
            true,
            self.config.base_type_weight,
            &format!("Base type severity: {:.2}", base_severity),
        ));

        // Component 2: Monetary Impact
        let monetary_severity = self.calculate_monetary_severity(context);
        if monetary_severity > 0.0 {
            factors.push(ContributingFactor::new(
                FactorType::AmountDeviation,
                monetary_severity,
                0.3,
                true,
                self.config.monetary_weight,
                &format!("Monetary impact severity: {:.2}", monetary_severity),
            ));
        }

        // Component 3: Frequency Factor
        let frequency_severity = self.calculate_frequency_severity(context);
        if frequency_severity > 0.0 {
            factors.push(ContributingFactor::new(
                FactorType::FrequencyDeviation,
                frequency_severity,
                0.3,
                true,
                self.config.frequency_weight,
                &format!(
                    "Frequency factor (count={}): {:.2}",
                    context.occurrence_count, frequency_severity
                ),
            ));
        }

        // Component 4: Scope Factor
        let scope_severity = self.calculate_scope_severity(context);
        if scope_severity > 0.0 {
            factors.push(ContributingFactor::new(
                FactorType::RelationshipAnomaly,
                scope_severity,
                0.3,
                true,
                self.config.scope_weight,
                &format!(
                    "Scope factor (entities={}): {:.2}",
                    context.affected_entity_count, scope_severity
                ),
            ));
        }

        // Component 5: Timing Factor
        let timing_severity = self.calculate_timing_severity(context);
        factors.push(ContributingFactor::new(
            FactorType::TimingAnomaly,
            timing_severity,
            0.3,
            true,
            self.config.timing_weight,
            &format!("Timing factor: {:.2}", timing_severity),
        ));

        // Calculate weighted sum
        let severity = base_severity * self.config.base_type_weight
            + monetary_severity * self.config.monetary_weight
            + frequency_severity * self.config.frequency_weight
            + scope_severity * self.config.scope_weight
            + timing_severity * self.config.timing_weight;

        // Apply custom modifier
        let final_severity = (severity * context.custom_modifier).clamp(0.0, 1.0);

        (final_severity, factors)
    }

    /// Calculates base severity from anomaly type.
    fn calculate_base_severity(&self, anomaly_type: &AnomalyType) -> f64 {
        // Convert 1-5 severity scale to 0.0-1.0
        let base_score = anomaly_type.severity() as f64 / 5.0;

        // Apply type-specific modifiers
        let modifier = match anomaly_type {
            AnomalyType::Fraud(fraud_type) => match fraud_type {
                FraudType::CollusiveApproval => 1.2,
                FraudType::RevenueManipulation => 1.2,
                FraudType::FictitiousVendor => 1.15,
                FraudType::AssetMisappropriation => 1.1,
                _ => 1.0,
            },
            AnomalyType::Error(error_type) => match error_type {
                ErrorType::UnbalancedEntry => 1.1, // Material misstatement
                ErrorType::CurrencyError => 1.05,
                _ => 1.0,
            },
            AnomalyType::ProcessIssue(process_type) => match process_type {
                ProcessIssueType::SystemBypass => 1.1,
                ProcessIssueType::IncompleteAuditTrail => 1.05,
                _ => 1.0,
            },
            AnomalyType::Statistical(_) => 0.9, // Generally less severe
            AnomalyType::Relational(rel_type) => match rel_type {
                RelationalAnomalyType::CircularTransaction => 1.1,
                RelationalAnomalyType::TransferPricingAnomaly => 1.1,
                _ => 1.0,
            },
            AnomalyType::Custom(_) => 1.0,
        };

        (base_score * modifier).clamp(0.0, 1.0)
    }

    /// Calculates monetary severity based on impact and materiality.
    fn calculate_monetary_severity(&self, context: &SeverityContext) -> f64 {
        match context.monetary_impact {
            Some(impact) => {
                let impact_f64: f64 = impact.abs().try_into().unwrap_or(0.0);
                let materiality_f64: f64 = self
                    .config
                    .materiality_threshold
                    .try_into()
                    .unwrap_or(10000.0);

                if materiality_f64 > 0.0 {
                    // Use log scale for impact relative to materiality
                    let ratio = impact_f64 / materiality_f64;

                    if ratio < 0.1 {
                        0.1 // Immaterial
                    } else if ratio < 0.5 {
                        0.3 // Low materiality
                    } else if ratio < 1.0 {
                        0.5 // Approaching materiality
                    } else if ratio < 2.0 {
                        0.7 // At materiality
                    } else if ratio < 5.0 {
                        0.85 // Significant
                    } else {
                        1.0 // Highly material
                    }
                } else {
                    0.5
                }
            }
            None => 0.3, // Default when no monetary impact
        }
    }

    /// Calculates frequency severity based on occurrence count.
    fn calculate_frequency_severity(&self, context: &SeverityContext) -> f64 {
        let count = context.occurrence_count;
        let threshold = self.config.high_frequency_threshold;

        if count == 0 {
            0.1 // First occurrence
        } else if count < threshold / 2 {
            0.3 // Low frequency
        } else if count < threshold {
            0.5 // Moderate frequency
        } else if count < threshold * 2 {
            0.7 // High frequency
        } else {
            0.9 // Very high frequency (repeat offender)
        }
    }

    /// Calculates scope severity based on affected entities.
    fn calculate_scope_severity(&self, context: &SeverityContext) -> f64 {
        let count = context.affected_entity_count;
        let threshold = self.config.broad_scope_threshold;

        if count <= 1 {
            0.2 // Single entity
        } else if count < threshold {
            0.4 // Limited scope
        } else if count < threshold * 2 {
            0.6 // Moderate scope
        } else if count < threshold * 3 {
            0.8 // Broad scope
        } else {
            1.0 // Pervasive
        }
    }

    /// Calculates timing severity based on period-end flags.
    fn calculate_timing_severity(&self, context: &SeverityContext) -> f64 {
        let mut severity: f64 = 0.2; // Base timing severity

        if context.is_audit_period {
            severity += 0.3;
        }

        if context.is_year_end {
            severity += 0.3;
        } else if context.is_quarter_end {
            severity += 0.2;
        } else if context.is_month_end {
            severity += 0.1;
        }

        severity.clamp(0.0, 1.0)
    }
}

impl Default for SeverityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined confidence and severity calculator.
#[derive(Debug, Clone, Default)]
pub struct AnomalyScoreCalculator {
    confidence_calculator: super::confidence::ConfidenceCalculator,
    severity_calculator: SeverityCalculator,
}

impl AnomalyScoreCalculator {
    /// Creates a new combined calculator with default configs.
    pub fn new() -> Self {
        Self {
            confidence_calculator: super::confidence::ConfidenceCalculator::new(),
            severity_calculator: SeverityCalculator::new(),
        }
    }

    /// Calculates both confidence and severity for an anomaly.
    pub fn calculate(
        &self,
        anomaly_type: &AnomalyType,
        confidence_context: &super::confidence::ConfidenceContext,
        severity_context: &SeverityContext,
    ) -> AnomalyScores {
        let (confidence, confidence_factors) = self
            .confidence_calculator
            .calculate(anomaly_type, confidence_context);
        let (severity, severity_factors) = self
            .severity_calculator
            .calculate(anomaly_type, severity_context);

        // Combine factors
        let mut all_factors = confidence_factors;
        all_factors.extend(severity_factors);

        // Calculate risk score (geometric mean of confidence and severity)
        let risk_score = (confidence * severity).sqrt();

        AnomalyScores {
            confidence,
            severity,
            risk_score,
            contributing_factors: all_factors,
        }
    }
}

/// Combined anomaly scores.
#[derive(Debug, Clone)]
pub struct AnomalyScores {
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// Severity score (0.0 - 1.0).
    pub severity: f64,
    /// Combined risk score (0.0 - 1.0).
    pub risk_score: f64,
    /// All contributing factors.
    pub contributing_factors: Vec<ContributingFactor>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_severity_calculator_basic() {
        let calculator = SeverityCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::DuplicatePayment);
        let context = SeverityContext::default();

        let (severity, factors) = calculator.calculate(&anomaly_type, &context);

        assert!((0.0..=1.0).contains(&severity));
        assert!(!factors.is_empty());
    }

    #[test]
    fn test_severity_with_monetary_impact() {
        let calculator = SeverityCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::DuplicatePayment);

        let low_impact_context = SeverityContext {
            monetary_impact: Some(dec!(100)),
            ..Default::default()
        };

        let high_impact_context = SeverityContext {
            monetary_impact: Some(dec!(100000)),
            ..Default::default()
        };

        let (low_severity, _) = calculator.calculate(&anomaly_type, &low_impact_context);
        let (high_severity, _) = calculator.calculate(&anomaly_type, &high_impact_context);

        // Higher impact should have higher severity
        assert!(high_severity > low_severity);
    }

    #[test]
    fn test_severity_with_frequency() {
        let calculator = SeverityCalculator::new();
        let anomaly_type = AnomalyType::Error(ErrorType::DuplicateEntry);

        let first_time = SeverityContext {
            occurrence_count: 0,
            ..Default::default()
        };

        let repeat_offender = SeverityContext {
            occurrence_count: 10,
            ..Default::default()
        };

        let (first_severity, _) = calculator.calculate(&anomaly_type, &first_time);
        let (repeat_severity, _) = calculator.calculate(&anomaly_type, &repeat_offender);

        // Repeat occurrence should have higher severity
        assert!(repeat_severity > first_severity);
    }

    #[test]
    fn test_severity_with_timing() {
        let calculator = SeverityCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::JustBelowThreshold);

        let normal_day = SeverityContext {
            is_month_end: false,
            is_quarter_end: false,
            is_year_end: false,
            ..Default::default()
        };

        let year_end = SeverityContext {
            is_year_end: true,
            is_audit_period: true,
            ..Default::default()
        };

        let (normal_severity, _) = calculator.calculate(&anomaly_type, &normal_day);
        let (year_end_severity, _) = calculator.calculate(&anomaly_type, &year_end);

        // Year-end during audit should have higher severity
        assert!(year_end_severity > normal_severity);
    }

    #[test]
    fn test_context_from_date() {
        let year_end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let context = SeverityContext::from_date(year_end_date);

        assert!(context.is_month_end);
        assert!(context.is_quarter_end);
        assert!(context.is_year_end);

        let mid_month = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mid_context = SeverityContext::from_date(mid_month);

        assert!(!mid_context.is_month_end);
        assert!(!mid_context.is_quarter_end);
        assert!(!mid_context.is_year_end);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = SeverityConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = SeverityConfig {
            base_type_weight: 0.5,
            monetary_weight: 0.5,
            frequency_weight: 0.5,
            scope_weight: 0.5,
            timing_weight: 0.5, // Sum = 2.5
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_combined_calculator() {
        let calculator = AnomalyScoreCalculator::new();
        let anomaly_type = AnomalyType::Fraud(FraudType::CollusiveApproval);

        let conf_context = super::super::confidence::ConfidenceContext {
            entity_risk_score: 0.8,
            prior_anomaly_count: 3,
            ..Default::default()
        };

        let sev_context = SeverityContext {
            monetary_impact: Some(dec!(50000)),
            occurrence_count: 2,
            is_year_end: true,
            ..Default::default()
        };

        let scores = calculator.calculate(&anomaly_type, &conf_context, &sev_context);

        assert!(scores.confidence >= 0.0 && scores.confidence <= 1.0);
        assert!(scores.severity >= 0.0 && scores.severity <= 1.0);
        assert!(scores.risk_score >= 0.0 && scores.risk_score <= 1.0);
        assert!(!scores.contributing_factors.is_empty());
    }
}
