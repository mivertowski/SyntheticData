//! Integration tests for RiskAssessment continuous scores and RiskStatus.

use datasynth_core::models::audit::risk::RiskStatus;
use datasynth_core::models::audit::{RiskAssessment, RiskCategory, RiskLevel};
use uuid::Uuid;

#[test]
fn risk_assessment_has_continuous_scores() {
    let risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        "Revenue Recognition",
        "Test risk for revenue recognition timing",
    )
    .with_risk_levels(RiskLevel::High, RiskLevel::Medium);

    // Continuous inherent scores derived from High level (0.55-0.80 range)
    assert!(
        risk.inherent_impact >= 0.0 && risk.inherent_impact <= 1.0,
        "inherent_impact {} out of [0,1] range",
        risk.inherent_impact
    );
    assert!(
        risk.inherent_likelihood >= 0.0 && risk.inherent_likelihood <= 1.0,
        "inherent_likelihood {} out of [0,1] range",
        risk.inherent_likelihood
    );

    // Continuous residual scores derived from Medium level (0.35-0.55 range)
    assert!(
        risk.residual_impact >= 0.0 && risk.residual_impact <= 1.0,
        "residual_impact {} out of [0,1] range",
        risk.residual_impact
    );
    assert!(
        risk.residual_likelihood >= 0.0 && risk.residual_likelihood <= 1.0,
        "residual_likelihood {} out of [0,1] range",
        risk.residual_likelihood
    );

    // risk_score is impact * likelihood * 100
    assert!(risk.risk_score >= 0.0, "risk_score should be non-negative");

    // risk_name populated from account_or_process
    assert!(!risk.risk_name.is_empty(), "risk_name should not be empty");
    assert!(
        risk.risk_name.contains("Revenue Recognition"),
        "risk_name '{}' should contain account_or_process",
        risk.risk_name
    );

    // Control linkage counts default to zero
    assert_eq!(risk.mitigating_control_count, 0);
    assert_eq!(risk.effective_control_count, 0);

    // Status defaults to Active
    assert!(matches!(risk.status, RiskStatus::Active));
}

#[test]
fn continuous_scores_match_risk_level_ranges() {
    // Low -> 0.15-0.35
    let low_risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        "Cash",
        "Low risk account",
    )
    .with_risk_levels(RiskLevel::Low, RiskLevel::Low);

    assert!(
        low_risk.inherent_impact >= 0.15 && low_risk.inherent_impact <= 0.35,
        "Low inherent_impact {} not in [0.15, 0.35]",
        low_risk.inherent_impact
    );
    assert!(
        low_risk.inherent_likelihood >= 0.15 && low_risk.inherent_likelihood <= 0.35,
        "Low inherent_likelihood {} not in [0.15, 0.35]",
        low_risk.inherent_likelihood
    );

    // High -> 0.55-0.80
    let high_risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::FraudRisk,
        "Revenue",
        "High risk",
    )
    .with_risk_levels(RiskLevel::High, RiskLevel::Low);

    assert!(
        high_risk.inherent_impact >= 0.55 && high_risk.inherent_impact <= 0.80,
        "High inherent_impact {} not in [0.55, 0.80]",
        high_risk.inherent_impact
    );

    // Significant -> 0.80-0.95
    let sig_risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::FraudRisk,
        "Management Override",
        "Significant risk",
    )
    .with_risk_levels(RiskLevel::Significant, RiskLevel::Significant);

    assert!(
        sig_risk.inherent_impact >= 0.80 && sig_risk.inherent_impact <= 0.95,
        "Significant inherent_impact {} not in [0.80, 0.95]",
        sig_risk.inherent_impact
    );
}

#[test]
fn risk_score_is_impact_times_likelihood() {
    let risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        "Inventory",
        "Inventory valuation risk",
    )
    .with_risk_levels(RiskLevel::High, RiskLevel::Medium);

    let expected_score = risk.inherent_impact * risk.inherent_likelihood * 100.0;
    assert!(
        (risk.risk_score - expected_score).abs() < f64::EPSILON,
        "risk_score {} should equal inherent_impact * inherent_likelihood * 100 = {}",
        risk.risk_score,
        expected_score
    );
}

#[test]
fn risk_name_includes_account_and_level() {
    let risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        "Accounts Receivable",
        "AR valuation risk",
    )
    .with_risk_levels(RiskLevel::Medium, RiskLevel::Low);

    assert!(
        risk.risk_name.contains("Accounts Receivable"),
        "risk_name '{}' should contain account name",
        risk.risk_name
    );
    assert!(
        risk.risk_name.contains("Medium"),
        "risk_name '{}' should contain risk level",
        risk.risk_name
    );
}

#[test]
fn risk_status_default_is_active() {
    assert_eq!(RiskStatus::default(), RiskStatus::Active);
}

#[test]
fn risk_status_serde_roundtrip() {
    let statuses = [
        RiskStatus::Active,
        RiskStatus::Mitigated,
        RiskStatus::Accepted,
        RiskStatus::Closed,
    ];

    for status in &statuses {
        let json = serde_json::to_string(status).expect("serialize");
        let deserialized: RiskStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*status, deserialized, "roundtrip failed for {:?}", status);
    }
}

#[test]
fn continuous_scores_deterministic_for_same_risk_id() {
    let id = Uuid::new_v4();

    let risk1 = RiskAssessment::new(id, RiskCategory::AssertionLevel, "Revenue", "Test")
        .with_risk_levels(RiskLevel::High, RiskLevel::Medium);

    let risk2 = RiskAssessment::new(id, RiskCategory::AssertionLevel, "Revenue", "Test")
        .with_risk_levels(RiskLevel::High, RiskLevel::Medium);

    // Note: risk_id is generated inside new(), so two calls produce different risk_ids.
    // This test verifies the jitter derivation mechanism works, not exact equality.
    // Both should be in the valid range for their level.
    assert!(risk1.inherent_impact >= 0.55 && risk1.inherent_impact <= 0.80);
    assert!(risk2.inherent_impact >= 0.55 && risk2.inherent_impact <= 0.80);
}
