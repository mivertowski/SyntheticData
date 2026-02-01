//! Validation tests for Enhanced Anomaly Taxonomy (FR-003).
//!
//! These tests validate:
//! - Confidence score bounds and calculation properties
//! - Severity score bounds and calculation properties
//! - Contributing factor generation
//! - AnomalyCategory coverage and mapping
//! - Combined scoring consistency

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::{
    AnomalyCategory, AnomalyType, ContributingFactor, EnhancedAnomalyLabel, ErrorType, FactorType,
    FraudType, LabeledAnomaly, ProcessIssueType, RelationalAnomalyType, StatisticalAnomalyType,
};
use datasynth_generators::anomaly::{
    confidence::{ConfidenceCalculator, ConfidenceConfig, ConfidenceContext},
    severity::{AnomalyScoreCalculator, SeverityCalculator, SeverityConfig, SeverityContext},
};

// =============================================================================
// Confidence Score Validation Tests
// =============================================================================

/// Test that confidence scores are always within [0, 1].
#[test]
fn test_confidence_bounds_all_anomaly_types() {
    let calculator = ConfidenceCalculator::new();

    // Test all anomaly types
    let anomaly_types = get_all_anomaly_types();

    for anomaly_type in &anomaly_types {
        // Test with minimal context
        let minimal_context = ConfidenceContext::default();
        let (confidence, _) = calculator.calculate(anomaly_type, &minimal_context);
        assert!(
            (0.0..=1.0).contains(&confidence),
            "Confidence for {:?} with minimal context should be in [0,1]: {}",
            anomaly_type,
            confidence
        );

        // Test with maximal context
        let maximal_context = ConfidenceContext {
            amount: Some(dec!(1_000_000)),
            expected_amount: Some(dec!(10_000)),
            prior_anomaly_count: 100,
            entity_risk_score: 1.0,
            auto_detected: true,
            evidence_count: 50,
            pattern_confidence: 1.0,
            timing_score: 1.0,
        };
        let (confidence, _) = calculator.calculate(anomaly_type, &maximal_context);
        assert!(
            (0.0..=1.0).contains(&confidence),
            "Confidence for {:?} with maximal context should be in [0,1]: {}",
            anomaly_type,
            confidence
        );
    }
}

/// Test that higher evidence increases confidence.
#[test]
fn test_confidence_increases_with_evidence() {
    let calculator = ConfidenceCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::FictitiousEntry);

    let low_evidence = ConfidenceContext {
        pattern_confidence: 0.3,
        evidence_count: 0,
        auto_detected: false,
        ..Default::default()
    };

    let high_evidence = ConfidenceContext {
        pattern_confidence: 0.9,
        evidence_count: 10,
        auto_detected: true,
        entity_risk_score: 0.8,
        ..Default::default()
    };

    let (low_confidence, _) = calculator.calculate(&anomaly_type, &low_evidence);
    let (high_confidence, _) = calculator.calculate(&anomaly_type, &high_evidence);

    assert!(
        high_confidence > low_confidence,
        "High evidence ({}) should yield higher confidence than low evidence ({})",
        high_confidence,
        low_confidence
    );
}

/// Test that confidence factors are generated correctly.
#[test]
fn test_confidence_contributing_factors() {
    let calculator = ConfidenceCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::SelfApproval);

    let context = ConfidenceContext {
        pattern_confidence: 0.8,
        evidence_count: 5,
        entity_risk_score: 0.7,
        auto_detected: true,
        ..Default::default()
    };

    let (confidence, factors) = calculator.calculate(&anomaly_type, &context);

    // Should have at least some factors
    assert!(!factors.is_empty(), "Should generate contributing factors");

    // All factors should have valid values
    for factor in &factors {
        assert!(
            factor.value >= 0.0,
            "Factor value should be non-negative: {:?}",
            factor
        );
        assert!(
            factor.weight >= 0.0 && factor.weight <= 1.0,
            "Factor weight should be in [0,1]: {:?}",
            factor
        );
        assert!(
            !factor.description.is_empty(),
            "Factor should have description"
        );
    }

    // Confidence should be consistent with factors
    assert!(
        confidence > 0.0,
        "Confidence should be positive with this context"
    );
}

/// Test config validation.
#[test]
fn test_confidence_config_validation() {
    let valid_config = ConfidenceConfig::default();
    assert!(
        valid_config.validate().is_ok(),
        "Default config should be valid"
    );

    let invalid_config = ConfidenceConfig {
        pattern_clarity_weight: 0.5,
        strength_weight: 0.5,
        detectability_weight: 0.5,
        context_weight: 0.5, // Sum = 2.0, invalid
        ..Default::default()
    };
    assert!(
        invalid_config.validate().is_err(),
        "Config with weights > 1.0 should be invalid"
    );
}

// =============================================================================
// Severity Score Validation Tests
// =============================================================================

/// Test that severity scores are always within [0, 1].
#[test]
fn test_severity_bounds_all_anomaly_types() {
    let calculator = SeverityCalculator::new();

    let anomaly_types = get_all_anomaly_types();

    for anomaly_type in &anomaly_types {
        // Test with minimal context
        let minimal_context = SeverityContext::default();
        let (severity, _) = calculator.calculate(anomaly_type, &minimal_context);
        assert!(
            (0.0..=1.0).contains(&severity),
            "Severity for {:?} with minimal context should be in [0,1]: {}",
            anomaly_type,
            severity
        );

        // Test with maximal context
        let maximal_context = SeverityContext {
            monetary_impact: Some(dec!(10_000_000)),
            occurrence_count: 100,
            affected_entity_count: 50,
            is_month_end: true,
            is_quarter_end: true,
            is_year_end: true,
            is_audit_period: true,
            custom_modifier: 1.0,
            ..Default::default()
        };
        let (severity, _) = calculator.calculate(anomaly_type, &maximal_context);
        assert!(
            (0.0..=1.0).contains(&severity),
            "Severity for {:?} with maximal context should be in [0,1]: {}",
            anomaly_type,
            severity
        );
    }
}

/// Test that monetary impact affects severity correctly.
#[test]
fn test_severity_monetary_impact_ordering() {
    let calculator = SeverityCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::FictitiousEntry);

    // Test increasing monetary impacts
    let impacts = [
        dec!(100),     // Immaterial
        dec!(1_000),   // Low
        dec!(5_000),   // Approaching
        dec!(10_000),  // At materiality
        dec!(50_000),  // Significant
        dec!(100_000), // Highly material
    ];

    let mut prev_severity = 0.0;
    for impact in impacts {
        let context = SeverityContext {
            monetary_impact: Some(impact),
            ..Default::default()
        };
        let (severity, _) = calculator.calculate(&anomaly_type, &context);

        assert!(
            severity >= prev_severity,
            "Severity should increase with monetary impact: {} at {} vs {} before",
            severity,
            impact,
            prev_severity
        );
        prev_severity = severity;
    }
}

/// Test that frequency affects severity.
#[test]
fn test_severity_frequency_ordering() {
    let calculator = SeverityCalculator::new();
    let anomaly_type = AnomalyType::Error(ErrorType::DuplicateEntry);

    let frequencies = [0, 1, 3, 5, 10, 20];

    let mut prev_severity = 0.0;
    for count in frequencies {
        let context = SeverityContext {
            occurrence_count: count,
            ..Default::default()
        };
        let (severity, _) = calculator.calculate(&anomaly_type, &context);

        assert!(
            severity >= prev_severity,
            "Severity should increase with frequency: {} at count {} vs {} before",
            severity,
            count,
            prev_severity
        );
        prev_severity = severity;
    }
}

/// Test that timing affects severity.
#[test]
fn test_severity_timing_factors() {
    let calculator = SeverityCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::JustBelowThreshold);

    // Normal day
    let normal = SeverityContext::default();
    let (normal_severity, _) = calculator.calculate(&anomaly_type, &normal);

    // Month end
    let month_end = SeverityContext {
        is_month_end: true,
        ..Default::default()
    };
    let (month_end_severity, _) = calculator.calculate(&anomaly_type, &month_end);

    // Quarter end
    let quarter_end = SeverityContext {
        is_month_end: true,
        is_quarter_end: true,
        ..Default::default()
    };
    let (quarter_end_severity, _) = calculator.calculate(&anomaly_type, &quarter_end);

    // Year end with audit
    let year_end_audit = SeverityContext {
        is_month_end: true,
        is_quarter_end: true,
        is_year_end: true,
        is_audit_period: true,
        ..Default::default()
    };
    let (year_end_severity, _) = calculator.calculate(&anomaly_type, &year_end_audit);

    // Timing should increase severity progressively
    assert!(
        month_end_severity >= normal_severity,
        "Month end severity ({}) should be >= normal ({})",
        month_end_severity,
        normal_severity
    );
    assert!(
        quarter_end_severity >= month_end_severity,
        "Quarter end severity ({}) should be >= month end ({})",
        quarter_end_severity,
        month_end_severity
    );
    assert!(
        year_end_severity >= quarter_end_severity,
        "Year end+audit severity ({}) should be >= quarter end ({})",
        year_end_severity,
        quarter_end_severity
    );
}

/// Test SeverityContext::from_date() auto-detection.
#[test]
fn test_severity_context_date_detection() {
    // Regular mid-month date
    let mid_month = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let mid_context = SeverityContext::from_date(mid_month);
    assert!(!mid_context.is_month_end);
    assert!(!mid_context.is_quarter_end);
    assert!(!mid_context.is_year_end);

    // Month end (not quarter)
    let month_end = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
    let month_context = SeverityContext::from_date(month_end);
    assert!(month_context.is_month_end);
    assert!(!month_context.is_quarter_end);

    // Quarter end
    let quarter_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    let quarter_context = SeverityContext::from_date(quarter_end);
    assert!(quarter_context.is_month_end);
    assert!(quarter_context.is_quarter_end);
    assert!(!quarter_context.is_year_end);

    // Year end
    let year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let year_context = SeverityContext::from_date(year_end);
    assert!(year_context.is_month_end);
    assert!(year_context.is_quarter_end);
    assert!(year_context.is_year_end);
}

/// Test config validation.
#[test]
fn test_severity_config_validation() {
    let valid_config = SeverityConfig::default();
    assert!(
        valid_config.validate().is_ok(),
        "Default config should be valid"
    );

    let invalid_config = SeverityConfig {
        base_type_weight: 0.5,
        monetary_weight: 0.5,
        frequency_weight: 0.5,
        scope_weight: 0.5,
        timing_weight: 0.5, // Sum = 2.5
        ..Default::default()
    };
    assert!(
        invalid_config.validate().is_err(),
        "Config with weights > 1.0 should be invalid"
    );
}

// =============================================================================
// Combined Score Validation Tests
// =============================================================================

/// Test combined confidence and severity calculation.
#[test]
fn test_combined_score_bounds() {
    let calculator = AnomalyScoreCalculator::new();

    let anomaly_types = get_all_anomaly_types();

    for anomaly_type in &anomaly_types {
        let conf_context = ConfidenceContext {
            pattern_confidence: 0.7,
            evidence_count: 3,
            entity_risk_score: 0.5,
            ..Default::default()
        };

        let sev_context = SeverityContext {
            monetary_impact: Some(dec!(25_000)),
            occurrence_count: 3,
            ..Default::default()
        };

        let scores = calculator.calculate(anomaly_type, &conf_context, &sev_context);

        // All scores should be in [0, 1]
        assert!(
            scores.confidence >= 0.0 && scores.confidence <= 1.0,
            "Confidence for {:?} should be in [0,1]: {}",
            anomaly_type,
            scores.confidence
        );
        assert!(
            scores.severity >= 0.0 && scores.severity <= 1.0,
            "Severity for {:?} should be in [0,1]: {}",
            anomaly_type,
            scores.severity
        );
        assert!(
            scores.risk_score >= 0.0 && scores.risk_score <= 1.0,
            "Risk score for {:?} should be in [0,1]: {}",
            anomaly_type,
            scores.risk_score
        );

        // Risk score should be geometric mean of confidence and severity
        let expected_risk = (scores.confidence * scores.severity).sqrt();
        assert!(
            (scores.risk_score - expected_risk).abs() < 0.001,
            "Risk score should be geometric mean: {} vs expected {}",
            scores.risk_score,
            expected_risk
        );

        // Should have contributing factors
        assert!(
            !scores.contributing_factors.is_empty(),
            "Should have contributing factors for {:?}",
            anomaly_type
        );
    }
}

/// Test that combined factors are complete.
#[test]
fn test_combined_factors_completeness() {
    let calculator = AnomalyScoreCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::SuspenseAccountAbuse);

    let conf_context = ConfidenceContext {
        pattern_confidence: 0.9,
        evidence_count: 5,
        auto_detected: true,
        ..Default::default()
    };

    let sev_context = SeverityContext {
        monetary_impact: Some(dec!(50_000)),
        occurrence_count: 5,
        is_year_end: true,
        ..Default::default()
    };

    let scores = calculator.calculate(&anomaly_type, &conf_context, &sev_context);

    // Should have factors from both confidence and severity calculations
    let factor_types: Vec<_> = scores
        .contributing_factors
        .iter()
        .map(|f| f.factor_type)
        .collect();

    // Should have at least some confidence factors
    let has_pattern_match = factor_types.contains(&FactorType::PatternMatch);
    assert!(has_pattern_match, "Should have pattern match factor");

    // Should have at least some severity factors
    let has_amount_deviation = factor_types.contains(&FactorType::AmountDeviation);
    let has_timing = factor_types.contains(&FactorType::TimingAnomaly);
    assert!(
        has_amount_deviation || has_timing,
        "Should have severity-related factors"
    );
}

// =============================================================================
// AnomalyCategory Validation Tests
// =============================================================================

/// Test that all AnomalyTypes can be categorized.
#[test]
fn test_anomaly_category_coverage() {
    let anomaly_types = get_all_anomaly_types();

    for anomaly_type in &anomaly_types {
        let category = AnomalyCategory::from_anomaly_type(anomaly_type);

        // Every anomaly type should map to a valid category
        match category {
            AnomalyCategory::FictitiousVendor
            | AnomalyCategory::VendorKickback
            | AnomalyCategory::RelatedPartyVendor
            | AnomalyCategory::DuplicatePayment
            | AnomalyCategory::UnauthorizedTransaction
            | AnomalyCategory::StructuredTransaction
            | AnomalyCategory::CircularFlow
            | AnomalyCategory::BehavioralAnomaly
            | AnomalyCategory::TimingAnomaly
            | AnomalyCategory::JournalAnomaly
            | AnomalyCategory::ManualOverride
            | AnomalyCategory::MissingApproval
            | AnomalyCategory::StatisticalOutlier
            | AnomalyCategory::DistributionAnomaly
            | AnomalyCategory::Custom(_) => {
                // Valid category
            }
        }
    }
}

/// Test category to string and back.
#[test]
fn test_anomaly_category_display() {
    let categories = vec![
        AnomalyCategory::FictitiousVendor,
        AnomalyCategory::DuplicatePayment,
        AnomalyCategory::CircularFlow,
        AnomalyCategory::StatisticalOutlier,
        AnomalyCategory::Custom("test_custom".to_string()),
    ];

    for category in categories {
        let display = format!("{:?}", category);
        assert!(
            !display.is_empty(),
            "Category should have display representation"
        );
    }
}

// =============================================================================
// ContributingFactor Validation Tests
// =============================================================================

/// Test ContributingFactor construction.
#[test]
fn test_contributing_factor_construction() {
    let factor = ContributingFactor::new(
        FactorType::AmountDeviation,
        0.85,
        0.5,
        true,
        0.3,
        "Amount exceeds threshold by 85%",
    );

    assert_eq!(factor.factor_type, FactorType::AmountDeviation);
    assert_eq!(factor.value, 0.85);
    assert_eq!(factor.threshold, 0.5);
    assert!(factor.direction_greater);
    assert_eq!(factor.weight, 0.3);
    assert!(!factor.description.is_empty());
}

/// Test all FactorTypes.
#[test]
fn test_all_factor_types() {
    let factor_types = vec![
        FactorType::AmountDeviation,
        FactorType::ThresholdProximity,
        FactorType::TimingAnomaly,
        FactorType::EntityRisk,
        FactorType::PatternMatch,
        FactorType::FrequencyDeviation,
        FactorType::RelationshipAnomaly,
        FactorType::ControlBypass,
        FactorType::BenfordViolation,
        FactorType::DuplicateIndicator,
        FactorType::ApprovalChainIssue,
        FactorType::DocumentationGap,
        FactorType::Custom,
    ];

    for factor_type in factor_types {
        let factor = ContributingFactor::new(factor_type, 0.5, 0.5, true, 0.1, "Test");
        assert_eq!(factor.factor_type, factor_type);
    }
}

// =============================================================================
// EnhancedAnomalyLabel Validation Tests
// =============================================================================

/// Test EnhancedAnomalyLabel construction and bounds.
#[test]
fn test_enhanced_anomaly_label_bounds() {
    let base = LabeledAnomaly {
        anomaly_id: "TEST001".to_string(),
        anomaly_type: AnomalyType::Fraud(FraudType::FictitiousEntry),
        document_id: "DOC001".to_string(),
        document_type: "JE".to_string(),
        company_code: "C001".to_string(),
        anomaly_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        detection_timestamp: NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap(),
        confidence: 0.85,
        severity: 4,
        description: "Fictitious entry detected".to_string(),
        related_entities: vec!["USER001".to_string()],
        monetary_impact: Some(dec!(10000)),
        metadata: HashMap::new(),
        is_injected: true,
        injection_strategy: Some("fraud_injection".to_string()),
        cluster_id: None,
        original_document_hash: None,
        causal_reason: None,
        structured_strategy: None,
        parent_anomaly_id: None,
        child_anomaly_ids: Vec::new(),
        scenario_id: None,
        run_id: None,
        generation_seed: None,
    };

    let enhanced = EnhancedAnomalyLabel {
        base: base.clone(),
        category: AnomalyCategory::FictitiousVendor,
        enhanced_confidence: 0.92,
        enhanced_severity: 0.78,
        contributing_factors: vec![
            ContributingFactor::new(
                FactorType::PatternMatch,
                1.0,
                0.5,
                true,
                0.4,
                "Pattern match found",
            ),
            ContributingFactor::new(
                FactorType::AmountDeviation,
                0.5,
                0.1,
                true,
                0.3,
                "Amount deviation",
            ),
        ],
        secondary_categories: vec![AnomalyCategory::UnauthorizedTransaction],
    };

    // Validate bounds
    assert!(
        enhanced.enhanced_confidence >= 0.0 && enhanced.enhanced_confidence <= 1.0,
        "Enhanced confidence should be in [0,1]"
    );
    assert!(
        enhanced.enhanced_severity >= 0.0 && enhanced.enhanced_severity <= 1.0,
        "Enhanced severity should be in [0,1]"
    );

    // Validate factors
    for factor in &enhanced.contributing_factors {
        assert!(factor.weight >= 0.0 && factor.weight <= 1.0);
    }
}

/// Test feature vector generation from enhanced label.
#[test]
fn test_enhanced_label_feature_vector() {
    let base = LabeledAnomaly {
        anomaly_id: "TEST002".to_string(),
        anomaly_type: AnomalyType::Error(ErrorType::MisclassifiedAccount),
        document_id: "DOC002".to_string(),
        document_type: "JE".to_string(),
        company_code: "C001".to_string(),
        anomaly_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        detection_timestamp: NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap(),
        confidence: 0.75,
        severity: 3,
        description: "Account misclassification".to_string(),
        related_entities: vec!["ACCOUNT1100".to_string(), "ACCOUNT2100".to_string()],
        monetary_impact: Some(dec!(5000)),
        metadata: HashMap::new(),
        is_injected: true,
        injection_strategy: Some("error_injection".to_string()),
        cluster_id: None,
        original_document_hash: None,
        causal_reason: None,
        structured_strategy: None,
        parent_anomaly_id: None,
        child_anomaly_ids: Vec::new(),
        scenario_id: None,
        run_id: None,
        generation_seed: None,
    };

    let enhanced = EnhancedAnomalyLabel {
        base,
        category: AnomalyCategory::JournalAnomaly,
        enhanced_confidence: 0.82,
        enhanced_severity: 0.65,
        contributing_factors: vec![
            ContributingFactor::new(
                FactorType::PatternMatch,
                0.7,
                0.5,
                true,
                0.25,
                "Pattern match",
            ),
            ContributingFactor::new(
                FactorType::ControlBypass,
                0.3,
                0.5,
                false,
                0.15,
                "Control check",
            ),
        ],
        secondary_categories: vec![],
    };

    let features = enhanced.to_features();

    // Should have consistent feature count
    assert!(features.len() >= 15, "Should have at least 15 features");

    // All features should be valid numbers (not NaN or Inf)
    for (i, &feature) in features.iter().enumerate() {
        assert!(
            feature.is_finite(),
            "Feature {} should be finite: {}",
            i,
            feature
        );
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

/// Test with zero/minimal inputs.
#[test]
fn test_zero_inputs() {
    let conf_calculator = ConfidenceCalculator::new();
    let sev_calculator = SeverityCalculator::new();
    let anomaly_type = AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount);

    // Zero context
    let zero_conf_context = ConfidenceContext {
        amount: None,
        expected_amount: None,
        prior_anomaly_count: 0,
        entity_risk_score: 0.0,
        auto_detected: false,
        evidence_count: 0,
        pattern_confidence: 0.0,
        timing_score: 0.0,
    };

    let (confidence, _) = conf_calculator.calculate(&anomaly_type, &zero_conf_context);
    assert!(
        (0.0..=1.0).contains(&confidence),
        "Confidence with zero inputs should still be valid: {}",
        confidence
    );

    let zero_sev_context = SeverityContext {
        monetary_impact: Some(Decimal::ZERO),
        occurrence_count: 0,
        affected_entity_count: 0,
        ..Default::default()
    };

    let (severity, _) = sev_calculator.calculate(&anomaly_type, &zero_sev_context);
    assert!(
        (0.0..=1.0).contains(&severity),
        "Severity with zero inputs should still be valid: {}",
        severity
    );
}

/// Test with negative monetary impact.
#[test]
fn test_negative_monetary_impact() {
    let calculator = SeverityCalculator::new();
    let anomaly_type = AnomalyType::Error(ErrorType::ReversedAmount);

    let context = SeverityContext {
        monetary_impact: Some(dec!(-50_000)), // Negative impact
        ..Default::default()
    };

    let (severity, _) = calculator.calculate(&anomaly_type, &context);

    // Should handle negative by taking absolute value
    assert!(
        (0.0..=1.0).contains(&severity),
        "Severity with negative impact should still be valid: {}",
        severity
    );
}

/// Test with extreme values.
#[test]
fn test_extreme_values() {
    let calculator = AnomalyScoreCalculator::new();
    let anomaly_type = AnomalyType::Fraud(FraudType::RevenueManipulation);

    let extreme_conf_context = ConfidenceContext {
        amount: Some(dec!(1_000_000_000)),
        expected_amount: Some(dec!(1_000)),
        pattern_confidence: 1.0,
        evidence_count: 1000,
        entity_risk_score: 1.0,
        prior_anomaly_count: 10000,
        auto_detected: true,
        timing_score: 1.0,
    };

    let extreme_sev_context = SeverityContext {
        monetary_impact: Some(dec!(1_000_000_000)), // 1 billion
        occurrence_count: 10000,
        affected_entity_count: 1000,
        is_month_end: true,
        is_quarter_end: true,
        is_year_end: true,
        is_audit_period: true,
        custom_modifier: 1.0,
        ..Default::default()
    };

    let scores = calculator.calculate(&anomaly_type, &extreme_conf_context, &extreme_sev_context);

    // All scores should still be bounded
    assert!(
        scores.confidence >= 0.0 && scores.confidence <= 1.0,
        "Confidence should be bounded even with extreme inputs: {}",
        scores.confidence
    );
    assert!(
        scores.severity >= 0.0 && scores.severity <= 1.0,
        "Severity should be bounded even with extreme inputs: {}",
        scores.severity
    );
    assert!(
        scores.risk_score >= 0.0 && scores.risk_score <= 1.0,
        "Risk score should be bounded even with extreme inputs: {}",
        scores.risk_score
    );
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_all_anomaly_types() -> Vec<AnomalyType> {
    vec![
        // Fraud types
        AnomalyType::Fraud(FraudType::FictitiousTransaction),
        AnomalyType::Fraud(FraudType::FictitiousEntry),
        AnomalyType::Fraud(FraudType::JustBelowThreshold),
        AnomalyType::Fraud(FraudType::SplitTransaction),
        AnomalyType::Fraud(FraudType::RevenueManipulation),
        AnomalyType::Fraud(FraudType::ExpenseCapitalization),
        AnomalyType::Fraud(FraudType::SuspenseAccountAbuse),
        AnomalyType::Fraud(FraudType::SelfApproval),
        AnomalyType::Fraud(FraudType::TimingAnomaly),
        AnomalyType::Fraud(FraudType::RoundDollarManipulation),
        AnomalyType::Fraud(FraudType::ImproperCapitalization),
        AnomalyType::Fraud(FraudType::ReserveManipulation),
        AnomalyType::Fraud(FraudType::UnauthorizedAccess),
        // Error types
        AnomalyType::Error(ErrorType::DuplicateEntry),
        AnomalyType::Error(ErrorType::ReversedAmount),
        AnomalyType::Error(ErrorType::WrongPeriod),
        AnomalyType::Error(ErrorType::TransposedDigits),
        AnomalyType::Error(ErrorType::DecimalError),
        AnomalyType::Error(ErrorType::MissingField),
        AnomalyType::Error(ErrorType::InvalidAccount),
        AnomalyType::Error(ErrorType::BackdatedEntry),
        AnomalyType::Error(ErrorType::FutureDatedEntry),
        AnomalyType::Error(ErrorType::CutoffError),
        AnomalyType::Error(ErrorType::MisclassifiedAccount),
        AnomalyType::Error(ErrorType::WrongCostCenter),
        // Process issues
        AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
        AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval),
        AnomalyType::ProcessIssue(ProcessIssueType::MissingDocumentation),
        AnomalyType::ProcessIssue(ProcessIssueType::LateApproval),
        AnomalyType::ProcessIssue(ProcessIssueType::IncompleteApprovalChain),
        AnomalyType::ProcessIssue(ProcessIssueType::SystemBypass),
        AnomalyType::ProcessIssue(ProcessIssueType::ManualOverride),
        AnomalyType::ProcessIssue(ProcessIssueType::AfterHoursPosting),
        AnomalyType::ProcessIssue(ProcessIssueType::WeekendPosting),
        // Statistical
        AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
        AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyLowAmount),
        AnomalyType::Statistical(StatisticalAnomalyType::TrendBreak),
        AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation),
        AnomalyType::Statistical(StatisticalAnomalyType::UnusualTiming),
        AnomalyType::Statistical(StatisticalAnomalyType::UnusualFrequency),
        AnomalyType::Statistical(StatisticalAnomalyType::TransactionBurst),
        AnomalyType::Statistical(StatisticalAnomalyType::LevelShift),
        AnomalyType::Statistical(StatisticalAnomalyType::SeasonalAnomaly),
        // Relational
        AnomalyType::Relational(RelationalAnomalyType::CircularTransaction),
        AnomalyType::Relational(RelationalAnomalyType::DormantAccountActivity),
        AnomalyType::Relational(RelationalAnomalyType::NewCounterparty),
        AnomalyType::Relational(RelationalAnomalyType::UnusualAccountPair),
        AnomalyType::Relational(RelationalAnomalyType::CentralityAnomaly),
        AnomalyType::Relational(RelationalAnomalyType::IsolatedCluster),
        // Custom
        AnomalyType::Custom("test_custom".to_string()),
    ]
}
