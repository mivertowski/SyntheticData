#![allow(clippy::unwrap_used)]

//! Integration tests for CausalDAG, Scenario, and Intervention models.

use std::collections::HashMap;

use datasynth_core::causal_dag::CausalDAG;
use datasynth_core::{
    ControlFailureIntervention, ControlFailureType, ControlTarget, InterventionType,
    MacroShockIntervention, MacroShockType, ParameterShiftIntervention, Scenario,
    ScenarioConstraints, ScenarioOutputConfig,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// 1. Default DAG: load, validate, count nodes & edges
// ---------------------------------------------------------------------------

#[test]
fn test_default_causal_dag_loads_and_validates() {
    let yaml = include_str!("../../datasynth-config/src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();

    dag.validate().unwrap();

    assert!(
        dag.nodes.len() >= 17,
        "Expected at least 17 nodes, got {}",
        dag.nodes.len()
    );
    assert!(
        dag.edges.len() >= 14,
        "Expected at least 14 edges, got {}",
        dag.edges.len()
    );

    // Topological order should have been filled in by validate()
    assert_eq!(
        dag.topological_order.len(),
        dag.nodes.len(),
        "Topological order should cover all nodes"
    );
}

// ---------------------------------------------------------------------------
// 2. DAG propagation: GDP shock ripples downstream
// ---------------------------------------------------------------------------

#[test]
fn test_dag_propagate_gdp_shock() {
    let yaml = include_str!("../../datasynth-config/src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();

    // Record baseline values for comparison
    let baseline_values = dag.propagate(&HashMap::new(), 0);

    // Severe recession: GDP drops from 0.025 baseline to -0.5
    let mut interventions = HashMap::new();
    interventions.insert("gdp_growth".to_string(), -0.05);

    // Month 1: GDP → transaction_volume has lag_months=1, so it should propagate
    let month1 = dag.propagate(&interventions, 1);

    // The intervention should cause gdp_growth to differ from baseline
    assert!(
        (month1["gdp_growth"] - (-0.05)).abs() < f64::EPSILON,
        "gdp_growth should be directly set to -0.05"
    );

    // transaction_volume should be affected (edge lag_months=1, so month 1 qualifies)
    let baseline_tv = baseline_values["transaction_volume"];
    let shocked_tv = month1["transaction_volume"];
    assert!(
        (shocked_tv - baseline_tv).abs() > f64::EPSILON,
        "transaction_volume should differ from baseline after GDP shock. baseline={}, shocked={}",
        baseline_tv,
        shocked_tv
    );

    // At month 3, more downstream effects should propagate (vendor_default_rate has lag=3)
    let month3 = dag.propagate(&interventions, 3);
    let baseline_vdr = baseline_values["vendor_default_rate"];
    let shocked_vdr = month3["vendor_default_rate"];
    assert!(
        (shocked_vdr - baseline_vdr).abs() > f64::EPSILON,
        "vendor_default_rate should differ from baseline at month 3. baseline={}, shocked={}",
        baseline_vdr,
        shocked_vdr
    );
}

// ---------------------------------------------------------------------------
// 3. Scenario YAML round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_scenario_serde_yaml_roundtrip() {
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "recession_stress_test".to_string(),
        description: "Simulates a moderate recession with GDP contraction".to_string(),
        tags: vec!["stress".to_string(), "macro".to_string()],
        base: Some("baseline_2025".to_string()),
        probability_weight: Some(0.3),
        interventions: vec![],
        constraints: ScenarioConstraints::default(),
        output: ScenarioOutputConfig::default(),
        metadata: {
            let mut m = HashMap::new();
            m.insert("author".to_string(), "test_suite".to_string());
            m.insert("version".to_string(), "1".to_string());
            m
        },
    };

    let yaml = serde_yaml::to_string(&scenario).unwrap();
    let deserialized: Scenario = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(deserialized.id, scenario.id);
    assert_eq!(deserialized.name, "recession_stress_test");
    assert_eq!(
        deserialized.description,
        "Simulates a moderate recession with GDP contraction"
    );
    assert_eq!(deserialized.tags, vec!["stress", "macro"]);
    assert_eq!(deserialized.base, Some("baseline_2025".to_string()));
    assert_eq!(deserialized.probability_weight, Some(0.3));
    assert!(deserialized.interventions.is_empty());
    assert!(deserialized.constraints.preserve_accounting_identity);
    assert!(deserialized.output.paired);
    assert_eq!(
        deserialized.metadata.get("author").map(|s| s.as_str()),
        Some("test_suite")
    );
    assert_eq!(
        deserialized.metadata.get("version").map(|s| s.as_str()),
        Some("1")
    );
}

// ---------------------------------------------------------------------------
// 4. InterventionType JSON round-trips (multiple variants)
// ---------------------------------------------------------------------------

#[test]
fn test_intervention_type_serde_json_roundtrip() {
    // -- ParameterShift --
    let param_shift = InterventionType::ParameterShift(ParameterShiftIntervention {
        target: "transactions.count".to_string(),
        from: Some(serde_json::json!(1000)),
        to: serde_json::json!(500),
        interpolation: Default::default(),
    });
    let json = serde_json::to_string(&param_shift).unwrap();
    let deser: InterventionType = serde_json::from_str(&json).unwrap();
    match &deser {
        InterventionType::ParameterShift(ps) => {
            assert_eq!(ps.target, "transactions.count");
            assert_eq!(ps.from, Some(serde_json::json!(1000)));
            assert_eq!(ps.to, serde_json::json!(500));
        }
        other => panic!(
            "Expected ParameterShift, got {:?}",
            std::mem::discriminant(other)
        ),
    }

    // -- ControlFailure --
    let control_failure = InterventionType::ControlFailure(ControlFailureIntervention {
        subtype: ControlFailureType::CompleteBypass,
        control_target: ControlTarget::ById {
            control_id: "C003".to_string(),
        },
        severity: 0.0,
        detectable: false,
    });
    let json = serde_json::to_string(&control_failure).unwrap();
    let deser: InterventionType = serde_json::from_str(&json).unwrap();
    match &deser {
        InterventionType::ControlFailure(cf) => {
            assert_eq!(cf.severity, 0.0);
            assert!(!cf.detectable);
        }
        other => panic!(
            "Expected ControlFailure, got {:?}",
            std::mem::discriminant(other)
        ),
    }

    // -- MacroShock --
    let macro_shock = InterventionType::MacroShock(MacroShockIntervention {
        subtype: MacroShockType::Recession,
        severity: 2.0,
        preset: Some("2008_financial_crisis".to_string()),
        overrides: {
            let mut m = HashMap::new();
            m.insert("gdp_growth".to_string(), -0.04);
            m
        },
    });
    let json = serde_json::to_string(&macro_shock).unwrap();
    let deser: InterventionType = serde_json::from_str(&json).unwrap();
    match &deser {
        InterventionType::MacroShock(ms) => {
            assert_eq!(ms.severity, 2.0);
            assert_eq!(ms.preset, Some("2008_financial_crisis".to_string()));
            assert_eq!(ms.overrides.get("gdp_growth"), Some(&-0.04));
        }
        other => panic!(
            "Expected MacroShock, got {:?}",
            std::mem::discriminant(other)
        ),
    }
}

// ---------------------------------------------------------------------------
// 5. ScenarioConstraints default: all preserve_* fields are true
// ---------------------------------------------------------------------------

#[test]
fn test_scenario_constraints_default() {
    let constraints = ScenarioConstraints::default();

    assert!(
        constraints.preserve_accounting_identity,
        "preserve_accounting_identity should default to true"
    );
    assert!(
        constraints.preserve_document_chains,
        "preserve_document_chains should default to true"
    );
    assert!(
        constraints.preserve_period_close,
        "preserve_period_close should default to true"
    );
    assert!(
        constraints.preserve_balance_coherence,
        "preserve_balance_coherence should default to true"
    );
    assert!(
        constraints.custom.is_empty(),
        "custom constraints should default to empty"
    );
}
