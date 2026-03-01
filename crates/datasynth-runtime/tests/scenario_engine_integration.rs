#![allow(clippy::unwrap_used)]

use datasynth_config::{
    CausalModelSchemaConfig, GeneratorConfig, InterventionSchemaConfig,
    InterventionTimingSchemaConfig, ScenarioConstraintsSchemaConfig, ScenarioOutputSchemaConfig,
    ScenarioSchemaConfig, ScenariosConfig,
};
use datasynth_core::{
    Intervention, InterventionTiming, InterventionType, OnsetType, ParameterShiftIntervention,
};
use datasynth_runtime::config_mutator::ConfigMutator;
use datasynth_runtime::intervention_manager::{InterventionError, InterventionManager};
use datasynth_runtime::scenario_engine::ScenarioEngine;
use datasynth_test_utils::fixtures::minimal_config;
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
