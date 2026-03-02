#![allow(clippy::unwrap_used)]

use datasynth_config::{
    CausalModelSchemaConfig, GeneratorConfig, InterventionSchemaConfig,
    InterventionTimingSchemaConfig, ScenarioConstraintsSchemaConfig, ScenarioOutputSchemaConfig,
    ScenarioSchemaConfig, ScenariosConfig,
};
use datasynth_core::{
    Intervention, InterventionTiming, InterventionType, OnsetType, ParameterShiftIntervention,
    ScenarioConstraints,
};
use datasynth_runtime::causal_engine::PropagatedInterventions;
use datasynth_runtime::config_mutator::{ConfigMutator, MutationError};
use datasynth_runtime::intervention_manager::{InterventionError, InterventionManager};
use datasynth_runtime::scenario_engine::ScenarioEngine;
use datasynth_test_utils::fixtures::minimal_config;
use std::collections::BTreeMap;
use tempfile::TempDir;
use uuid::Uuid;

/// Helper: build a GeneratorConfig with the given list of scenarios.
fn config_with_scenarios(scenarios: Vec<ScenarioSchemaConfig>) -> GeneratorConfig {
    let mut config = minimal_config();
    // Ensure period_months is large enough for multi-month tests.
    config.global.period_months = 12;
    config.scenarios = ScenariosConfig {
        enabled: true,
        scenarios,
        causal_model: CausalModelSchemaConfig::default(),
        defaults: Default::default(),
    };
    config
}

/// Helper: build a single ScenarioSchemaConfig with a parameter_shift intervention.
fn make_scenario(
    name: &str,
    description: &str,
    tags: Vec<&str>,
    start_month: u32,
    priority: u32,
) -> ScenarioSchemaConfig {
    ScenarioSchemaConfig {
        name: name.to_string(),
        description: description.to_string(),
        tags: tags.into_iter().map(String::from).collect(),
        base: None,
        probability_weight: Some(0.5),
        interventions: vec![InterventionSchemaConfig {
            intervention_type: serde_json::json!({
                "type": "parameter_shift",
                "target": "global.period_months",
                "to": 6,
                "interpolation": "linear"
            }),
            timing: InterventionTimingSchemaConfig {
                start_month,
                duration_months: None,
                onset: "sudden".to_string(),
                ramp_months: None,
            },
            label: Some(format!("{} shift", name)),
            priority,
        }],
        constraints: ScenarioConstraintsSchemaConfig::default(),
        output: ScenarioOutputSchemaConfig::default(),
        metadata: Default::default(),
    }
}

// ---------------------------------------------------------------------------
// 1. Full scenario pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_full_scenario_pipeline() {
    let scenario = make_scenario("recession", "Economic downturn", vec!["macro"], 1, 0);
    let config = config_with_scenarios(vec![scenario]);

    let engine = ScenarioEngine::new(config).expect("should create engine");
    let tmpdir = TempDir::new().expect("should create tmpdir");

    let results = engine
        .generate_all(tmpdir.path())
        .expect("generate_all should succeed");

    // Exactly one scenario was defined.
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].scenario_name, "recession");
    assert!(results[0].interventions_applied >= 1);

    // Scenario output directory must exist.
    let scenario_data_dir = tmpdir
        .path()
        .join("scenarios")
        .join("recession")
        .join("data");
    assert!(
        scenario_data_dir.exists(),
        "scenario data directory should exist at {:?}",
        scenario_data_dir
    );

    // Manifest file must exist and be valid YAML.
    let manifest_path = tmpdir
        .path()
        .join("scenarios")
        .join("recession")
        .join("scenario_manifest.yaml");
    assert!(
        manifest_path.exists(),
        "scenario manifest should exist at {:?}",
        manifest_path
    );

    let manifest_content =
        std::fs::read_to_string(&manifest_path).expect("should read manifest file");
    let manifest: serde_yaml::Value =
        serde_yaml::from_str(&manifest_content).expect("manifest should be valid YAML");

    // Verify key fields in the manifest.
    assert_eq!(manifest["scenario_name"].as_str().unwrap(), "recession");
    assert!(manifest["interventions_count"].as_u64().unwrap() >= 1);

    // Baseline directory must also have been created.
    let baseline_dir = tmpdir.path().join("baseline");
    assert!(
        baseline_dir.exists(),
        "baseline directory should exist at {:?}",
        baseline_dir
    );
}

// ---------------------------------------------------------------------------
// 2. ConfigMutator dot-path roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_config_mutator_dot_path_roundtrip() {
    let mut json = serde_json::json!({
        "global": {
            "seed": 42,
            "period_months": 12,
            "start_date": "2024-01-01",
            "industry": "manufacturing"
        },
        "distributions": {
            "amounts": {
                "components": [
                    {"mu": 6.0, "sigma": 1.5, "label": "routine"},
                    {"mu": 8.5, "sigma": 1.0, "label": "significant"}
                ]
            }
        },
        "transactions": {
            "count": 1000,
            "batch_size": 50
        }
    });

    // Apply several mutations via dot-paths.
    ConfigMutator::apply_at_path(&mut json, "global.seed", &serde_json::json!(99)).unwrap();
    ConfigMutator::apply_at_path(&mut json, "global.period_months", &serde_json::json!(6)).unwrap();
    ConfigMutator::apply_at_path(
        &mut json,
        "distributions.amounts.components[0].mu",
        &serde_json::json!(5.0),
    )
    .unwrap();
    ConfigMutator::apply_at_path(
        &mut json,
        "distributions.amounts.components[1].sigma",
        &serde_json::json!(2.0),
    )
    .unwrap();
    ConfigMutator::apply_at_path(&mut json, "transactions.count", &serde_json::json!(2000))
        .unwrap();

    // Verify all mutations applied correctly.
    assert_eq!(json["global"]["seed"], 99);
    assert_eq!(json["global"]["period_months"], 6);
    assert_eq!(json["distributions"]["amounts"]["components"][0]["mu"], 5.0);
    assert_eq!(
        json["distributions"]["amounts"]["components"][1]["sigma"],
        2.0
    );
    assert_eq!(json["transactions"]["count"], 2000);

    // Verify other fields are preserved.
    assert_eq!(json["global"]["start_date"], "2024-01-01");
    assert_eq!(json["global"]["industry"], "manufacturing");
    assert_eq!(
        json["distributions"]["amounts"]["components"][0]["sigma"],
        1.5
    );
    assert_eq!(
        json["distributions"]["amounts"]["components"][0]["label"],
        "routine"
    );
    assert_eq!(json["distributions"]["amounts"]["components"][1]["mu"], 8.5);
    assert_eq!(
        json["distributions"]["amounts"]["components"][1]["label"],
        "significant"
    );
    assert_eq!(json["transactions"]["batch_size"], 50);
}

// ---------------------------------------------------------------------------
// 3. Intervention validation rejects out-of-range timing
// ---------------------------------------------------------------------------

#[test]
fn test_intervention_validation_rejects_out_of_range() {
    let config = minimal_config();
    // minimal_config has period_months = 1, so start_month = 5 is out of range.
    let intervention = Intervention {
        id: Uuid::new_v4(),
        intervention_type: InterventionType::ParameterShift(ParameterShiftIntervention {
            target: "global.period_months".to_string(),
            from: None,
            to: serde_json::json!(3),
            interpolation: Default::default(),
        }),
        timing: InterventionTiming {
            start_month: 5,
            duration_months: None,
            onset: OnsetType::Sudden,
            ramp_months: None,
        },
        label: Some("out-of-range".to_string()),
        priority: 0,
    };

    let result = InterventionManager::validate(&[intervention], &config);
    assert!(result.is_err(), "should reject out-of-range start_month");

    match result.unwrap_err() {
        InterventionError::TimingOutOfRange { start, period } => {
            assert_eq!(start, 5);
            assert_eq!(period, config.global.period_months);
        }
        other => panic!("expected TimingOutOfRange, got: {}", other),
    }
}

// ---------------------------------------------------------------------------
// 4. Intervention conflict detection for same priority
// ---------------------------------------------------------------------------

#[test]
fn test_intervention_conflict_same_priority() {
    let mut config = minimal_config();
    config.global.period_months = 12;

    let intervention_a = Intervention {
        id: Uuid::new_v4(),
        intervention_type: InterventionType::ParameterShift(ParameterShiftIntervention {
            target: "transactions.count".to_string(),
            from: None,
            to: serde_json::json!(2000),
            interpolation: Default::default(),
        }),
        timing: InterventionTiming {
            start_month: 1,
            duration_months: Some(6),
            onset: OnsetType::Sudden,
            ramp_months: None,
        },
        label: Some("shift-a".to_string()),
        priority: 0,
    };

    let intervention_b = Intervention {
        id: Uuid::new_v4(),
        intervention_type: InterventionType::ParameterShift(ParameterShiftIntervention {
            target: "transactions.count".to_string(),
            from: None,
            to: serde_json::json!(3000),
            interpolation: Default::default(),
        }),
        timing: InterventionTiming {
            start_month: 3,
            duration_months: Some(6),
            onset: OnsetType::Gradual,
            ramp_months: None,
        },
        label: Some("shift-b".to_string()),
        priority: 0, // same priority as intervention_a
    };

    let result = InterventionManager::validate(&[intervention_a, intervention_b], &config);
    assert!(
        result.is_err(),
        "should detect conflict on same path + priority"
    );

    match result.unwrap_err() {
        InterventionError::ConflictDetected(priority, path) => {
            assert_eq!(priority, 0);
            assert_eq!(path, "transactions.count");
        }
        other => panic!("expected ConflictDetected, got: {}", other),
    }
}

// ---------------------------------------------------------------------------
// 5. list_scenarios and validate_all
// ---------------------------------------------------------------------------

#[test]
fn test_scenario_list_and_validate() {
    let scenario_a = make_scenario("boom", "Economic boom", vec!["macro", "growth"], 1, 0);
    let scenario_b = make_scenario("bust", "Economic bust", vec!["macro", "decline"], 3, 1);
    let config = config_with_scenarios(vec![scenario_a, scenario_b]);

    let engine = ScenarioEngine::new(config).expect("should create engine");

    // list_scenarios returns all configured scenarios.
    let summaries = engine.list_scenarios();
    assert_eq!(summaries.len(), 2);

    let names: Vec<&str> = summaries.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"boom"), "should contain 'boom'");
    assert!(names.contains(&"bust"), "should contain 'bust'");

    // Verify intervention counts.
    let boom = summaries.iter().find(|s| s.name == "boom").unwrap();
    assert_eq!(boom.intervention_count, 1);
    assert_eq!(boom.tags, vec!["macro", "growth"]);
    assert_eq!(boom.probability_weight, Some(0.5));

    let bust = summaries.iter().find(|s| s.name == "bust").unwrap();
    assert_eq!(bust.intervention_count, 1);
    assert_eq!(bust.tags, vec!["macro", "decline"]);

    // validate_all returns a validation result per scenario.
    let validations = engine.validate_all();
    assert_eq!(validations.len(), 2);

    for v in &validations {
        assert!(
            v.valid,
            "scenario '{}' should be valid, error: {:?}",
            v.name, v.error
        );
        assert!(v.error.is_none());
    }
}

// ---------------------------------------------------------------------------
// 6. Full scenario pipeline with diff (spec creation + config mutation E2E)
// ---------------------------------------------------------------------------

#[test]
fn test_full_scenario_pipeline_with_diff() {
    // Create a minimal config: 1 company, 3 months, small CoA.
    let mut config = minimal_config();
    config.global.period_months = 3;

    // Create a scenario with a parameter_shift targeting transaction volume.
    // Use the minimal causal DAG preset which has a transaction_volume node
    // bound to "transactions.volume_multiplier".
    let scenario = ScenarioSchemaConfig {
        name: "volume_increase".to_string(),
        description: "Increase transaction volume".to_string(),
        tags: vec!["volume".to_string(), "test".to_string()],
        base: None,
        probability_weight: Some(0.7),
        interventions: vec![InterventionSchemaConfig {
            intervention_type: serde_json::json!({
                "type": "parameter_shift",
                "target": "transactions.volume_multiplier",
                "to": 2.0,
                "interpolation": "linear"
            }),
            timing: InterventionTimingSchemaConfig {
                start_month: 1,
                duration_months: None,
                onset: "sudden".to_string(),
                ramp_months: None,
            },
            label: Some("Volume doubling".to_string()),
            priority: 0,
        }],
        constraints: ScenarioConstraintsSchemaConfig::default(),
        output: ScenarioOutputSchemaConfig::default(),
        metadata: Default::default(),
    };

    config.scenarios = ScenariosConfig {
        enabled: true,
        scenarios: vec![scenario],
        causal_model: CausalModelSchemaConfig {
            preset: "minimal".to_string(),
            ..Default::default()
        },
        defaults: Default::default(),
    };

    // Build engine and run generate_all.
    let engine = ScenarioEngine::new(config.clone()).expect("should create engine");
    let tmpdir = TempDir::new().expect("should create tmpdir");
    let results = engine
        .generate_all(tmpdir.path())
        .expect("generate_all should succeed");

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.scenario_name, "volume_increase");
    assert!(
        result.interventions_applied >= 1,
        "at least one intervention should be applied"
    );

    // Verify that months were affected by the intervention propagation.
    assert!(
        result.months_affected > 0,
        "intervention should affect at least one month"
    );

    // Verify baseline and counterfactual directories exist.
    assert!(
        result.baseline_path.exists(),
        "baseline path should exist: {:?}",
        result.baseline_path
    );
    assert!(
        result.counterfactual_path.exists(),
        "counterfactual path should exist: {:?}",
        result.counterfactual_path
    );

    // Read the manifest and verify the config_paths_changed field is non-empty
    // (this acts as a "diff" -- the propagation produced concrete config changes).
    let manifest_path = tmpdir
        .path()
        .join("scenarios")
        .join("volume_increase")
        .join("scenario_manifest.yaml");
    assert!(manifest_path.exists(), "manifest should exist");

    let manifest_content = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_content).unwrap();
    let paths_changed = manifest["config_paths_changed"]
        .as_sequence()
        .expect("config_paths_changed should be a sequence");
    assert!(
        !paths_changed.is_empty(),
        "config_paths_changed should be non-empty, indicating the intervention produced diffs"
    );

    // Verify one of the changed paths relates to the volume_multiplier binding.
    let changed_strs: Vec<&str> = paths_changed.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        changed_strs
            .iter()
            .any(|p| p.contains("volume_multiplier") || p.contains("base_rate")),
        "expected at least one path related to volume_multiplier or base_rate in changed paths: {:?}",
        changed_strs
    );
}

// ---------------------------------------------------------------------------
// 7. Scenario with constraints rejects disabling document chains
// ---------------------------------------------------------------------------

#[test]
fn test_scenario_with_constraints() {
    // Start with minimal_config where document_flows.generate_document_references=true
    // and balance.validate_balance_equation=true (both defaults).
    let mut config = minimal_config();
    config.global.period_months = 3;

    // Disable document references in the config -- simulating what a mutation might do.
    config.document_flows.generate_document_references = false;

    // Set up constraints that require document chains to be preserved.
    let constraints = ScenarioConstraints {
        preserve_document_chains: true,
        preserve_accounting_identity: true,
        preserve_period_close: false,
        preserve_balance_coherence: false,
        custom: vec![],
    };

    // Apply with empty propagated interventions -- the constraint check happens
    // post-mutation on the (already-modified) config.
    let propagated = PropagatedInterventions {
        changes_by_month: BTreeMap::new(),
    };

    let result = ConfigMutator::apply(&config, &propagated, &constraints);
    assert!(
        result.is_err(),
        "should fail constraint validation when document chains are required but disabled"
    );

    match result {
        Err(MutationError::ConstraintViolation(msg)) => {
            assert!(
                msg.contains("document_flows"),
                "error message should mention document_flows, got: {}",
                msg
            );
            assert!(
                msg.contains("preserve_document_chains")
                    || msg.contains("generate_document_references"),
                "error message should reference the violated constraint, got: {}",
                msg
            );
        }
        other => panic!(
            "expected MutationError::ConstraintViolation, got: {:?}",
            other
        ),
    }

    // Also verify balance coherence constraint violation.
    let mut config2 = minimal_config();
    config2.global.period_months = 3;
    config2.balance.validate_balance_equation = false;

    let constraints2 = ScenarioConstraints {
        preserve_balance_coherence: true,
        preserve_document_chains: false,
        preserve_accounting_identity: false,
        preserve_period_close: false,
        custom: vec![],
    };

    let result2 = ConfigMutator::apply(&config2, &propagated, &constraints2);
    assert!(
        result2.is_err(),
        "should fail when balance coherence is required but validate_balance_equation is false"
    );

    match result2 {
        Err(MutationError::ConstraintViolation(msg)) => {
            assert!(
                msg.contains("balance"),
                "error message should mention balance, got: {}",
                msg
            );
        }
        other => panic!(
            "expected MutationError::ConstraintViolation for balance, got: {:?}",
            other
        ),
    }

    // Verify that when constraints are NOT enabled, the same configs pass.
    let no_constraints = ScenarioConstraints {
        preserve_document_chains: false,
        preserve_accounting_identity: false,
        preserve_period_close: false,
        preserve_balance_coherence: false,
        custom: vec![],
    };

    let result3 = ConfigMutator::apply(&config, &propagated, &no_constraints);
    assert!(
        result3.is_ok(),
        "should succeed when preserve flags are off, even with doc refs disabled: {:?}",
        result3.err()
    );
}

// ---------------------------------------------------------------------------
// 8. Multiple scenarios produce distinct configs sequentially
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_scenarios_sequential() {
    // Create two ScenarioSpec objects with different interventions.
    let scenario_growth = ScenarioSchemaConfig {
        name: "growth".to_string(),
        description: "Economic growth scenario".to_string(),
        tags: vec!["macro".to_string(), "growth".to_string()],
        base: None,
        probability_weight: Some(0.6),
        interventions: vec![InterventionSchemaConfig {
            intervention_type: serde_json::json!({
                "type": "parameter_shift",
                "target": "transactions.volume_multiplier",
                "to": 2.5,
                "interpolation": "linear"
            }),
            timing: InterventionTimingSchemaConfig {
                start_month: 1,
                duration_months: None,
                onset: "sudden".to_string(),
                ramp_months: None,
            },
            label: Some("Growth shift".to_string()),
            priority: 0,
        }],
        constraints: ScenarioConstraintsSchemaConfig::default(),
        output: ScenarioOutputSchemaConfig::default(),
        metadata: Default::default(),
    };

    let scenario_contraction = ScenarioSchemaConfig {
        name: "contraction".to_string(),
        description: "Economic contraction scenario".to_string(),
        tags: vec!["macro".to_string(), "decline".to_string()],
        base: None,
        probability_weight: Some(0.4),
        interventions: vec![InterventionSchemaConfig {
            intervention_type: serde_json::json!({
                "type": "parameter_shift",
                "target": "transactions.volume_multiplier",
                "to": 0.5,
                "interpolation": "linear"
            }),
            timing: InterventionTimingSchemaConfig {
                start_month: 2,
                duration_months: Some(4),
                onset: "gradual".to_string(),
                ramp_months: Some(2),
            },
            label: Some("Contraction shift".to_string()),
            priority: 0,
        }],
        constraints: ScenarioConstraintsSchemaConfig::default(),
        output: ScenarioOutputSchemaConfig::default(),
        metadata: Default::default(),
    };

    // Verify they have unique names.
    assert_ne!(scenario_growth.name, scenario_contraction.name);

    // Verify they have different probability weights.
    assert_ne!(
        scenario_growth.probability_weight,
        scenario_contraction.probability_weight
    );

    // Build a config with both scenarios using the minimal DAG
    // (which has a transaction_volume node bound to transactions.volume_multiplier).
    let mut config = minimal_config();
    config.global.period_months = 6;
    config.scenarios = ScenariosConfig {
        enabled: true,
        scenarios: vec![scenario_growth.clone(), scenario_contraction.clone()],
        causal_model: CausalModelSchemaConfig {
            preset: "minimal".to_string(),
            ..Default::default()
        },
        defaults: Default::default(),
    };

    let engine = ScenarioEngine::new(config.clone()).expect("should create engine");

    // Verify both scenarios are listed.
    let summaries = engine.list_scenarios();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].name, "growth");
    assert_eq!(summaries[1].name, "contraction");

    // Both should validate successfully.
    let validations = engine.validate_all();
    assert_eq!(validations.len(), 2);
    for v in &validations {
        assert!(
            v.valid,
            "scenario '{}' should be valid, error: {:?}",
            v.name, v.error
        );
    }

    // Run generate_all to verify both scenarios produce output.
    let tmpdir = TempDir::new().expect("should create tmpdir");
    let results = engine
        .generate_all(tmpdir.path())
        .expect("generate_all should succeed for both scenarios");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].scenario_name, "growth");
    assert_eq!(results[1].scenario_name, "contraction");

    // Each scenario should have its own output directory.
    let growth_dir = tmpdir.path().join("scenarios").join("growth").join("data");
    let contraction_dir = tmpdir
        .path()
        .join("scenarios")
        .join("contraction")
        .join("data");
    assert!(growth_dir.exists(), "growth data dir should exist");
    assert!(
        contraction_dir.exists(),
        "contraction data dir should exist"
    );

    // Read both manifests and verify they contain different config changes.
    let growth_manifest_path = tmpdir
        .path()
        .join("scenarios")
        .join("growth")
        .join("scenario_manifest.yaml");
    let contraction_manifest_path = tmpdir
        .path()
        .join("scenarios")
        .join("contraction")
        .join("scenario_manifest.yaml");

    let growth_manifest: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(&growth_manifest_path).unwrap()).unwrap();
    let contraction_manifest: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(&contraction_manifest_path).unwrap())
            .unwrap();

    // Verify names differ in manifests.
    assert_eq!(growth_manifest["scenario_name"].as_str().unwrap(), "growth");
    assert_eq!(
        contraction_manifest["scenario_name"].as_str().unwrap(),
        "contraction"
    );

    // Both should have interventions applied.
    assert!(growth_manifest["interventions_count"].as_u64().unwrap() >= 1);
    assert!(
        contraction_manifest["interventions_count"]
            .as_u64()
            .unwrap()
            >= 1
    );

    // The contraction scenario has start_month=2 and gradual onset with ramp_months=2,
    // so it should affect fewer or different months than growth (which starts at month 1, sudden).
    let growth_months = growth_manifest["months_affected"].as_u64().unwrap();
    let contraction_months = contraction_manifest["months_affected"].as_u64().unwrap();
    assert!(growth_months > 0, "growth should affect at least one month");
    assert!(
        contraction_months > 0,
        "contraction should affect at least one month"
    );
}
