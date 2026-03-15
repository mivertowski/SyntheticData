//! Integration tests for InternalControl test history, effectiveness, and risk linkage.

use datasynth_core::models::internal_control::{ControlEffectiveness, TestResult};
use datasynth_core::models::{ControlType, InternalControl};

#[test]
fn internal_control_has_test_history_and_effectiveness() {
    let ctrl = InternalControl::new(
        "C001",
        "Revenue Recognition Control",
        ControlType::Preventive,
        "Ensure revenue accuracy",
    );

    // Owner resolution fields default to None / empty
    assert!(ctrl.owner_employee_id.is_none());
    assert!(ctrl.owner_name.is_empty());

    // Test history defaults
    assert_eq!(ctrl.test_count, 0);
    assert!(ctrl.last_tested_date.is_none());
    assert!(matches!(ctrl.test_result, TestResult::NotTested));

    // Derived effectiveness defaults
    assert!(matches!(
        ctrl.effectiveness,
        ControlEffectiveness::NotTested
    ));

    // Risk linkage defaults
    assert!(ctrl.mitigates_risk_ids.is_empty());
    assert!(ctrl.covers_account_classes.is_empty());
}

#[test]
fn test_result_default_is_not_tested() {
    assert_eq!(TestResult::default(), TestResult::NotTested);
}

#[test]
fn control_effectiveness_default_is_not_tested() {
    assert_eq!(
        ControlEffectiveness::default(),
        ControlEffectiveness::NotTested
    );
}

#[test]
fn test_result_serde_roundtrip() {
    let variants = [
        TestResult::Pass,
        TestResult::Partial,
        TestResult::Fail,
        TestResult::NotTested,
    ];

    for variant in &variants {
        let json = serde_json::to_string(variant).expect("serialize");
        let deserialized: TestResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*variant, deserialized, "roundtrip failed for {:?}", variant);
    }
}

#[test]
fn control_effectiveness_serde_roundtrip() {
    let variants = [
        ControlEffectiveness::Effective,
        ControlEffectiveness::PartiallyEffective,
        ControlEffectiveness::NotTested,
        ControlEffectiveness::Ineffective,
    ];

    for variant in &variants {
        let json = serde_json::to_string(variant).expect("serialize");
        let deserialized: ControlEffectiveness = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*variant, deserialized, "roundtrip failed for {:?}", variant);
    }
}

#[test]
fn builder_methods_for_new_fields() {
    let ctrl = InternalControl::new(
        "C002",
        "Test Control",
        ControlType::Detective,
        "Test objective",
    )
    .with_owner_employee("EMP-001", "Jane Smith")
    .with_test_history(
        3,
        Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()),
        TestResult::Pass,
    )
    .with_effectiveness(ControlEffectiveness::Effective)
    .with_mitigates_risk_ids(vec!["R001".into(), "R002".into()])
    .with_covers_account_classes(vec!["Revenue".into(), "Receivables".into()]);

    assert_eq!(ctrl.owner_employee_id, Some("EMP-001".into()));
    assert_eq!(ctrl.owner_name, "Jane Smith");
    assert_eq!(ctrl.test_count, 3);
    assert_eq!(
        ctrl.last_tested_date,
        Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap())
    );
    assert!(matches!(ctrl.test_result, TestResult::Pass));
    assert!(matches!(
        ctrl.effectiveness,
        ControlEffectiveness::Effective
    ));
    assert_eq!(ctrl.mitigates_risk_ids, vec!["R001", "R002"]);
    assert_eq!(ctrl.covers_account_classes, vec!["Revenue", "Receivables"]);
}

#[test]
fn standard_controls_have_default_new_fields() {
    let controls = InternalControl::standard_controls();
    for ctrl in &controls {
        // All standard controls should have default new field values
        assert!(ctrl.owner_employee_id.is_none());
        assert!(ctrl.owner_name.is_empty());
        assert_eq!(ctrl.test_count, 0);
        assert!(ctrl.last_tested_date.is_none());
        assert!(matches!(ctrl.test_result, TestResult::NotTested));
        assert!(matches!(
            ctrl.effectiveness,
            ControlEffectiveness::NotTested
        ));
        assert!(ctrl.mitigates_risk_ids.is_empty());
        assert!(ctrl.covers_account_classes.is_empty());
    }
}
