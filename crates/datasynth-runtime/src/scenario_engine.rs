//! Scenario engine orchestrator for paired baseline/counterfactual generation.

use crate::causal_engine::{CausalPropagationEngine, PropagationError};
use crate::config_mutator::{ConfigMutator, MutationError};
use crate::intervention_manager::{InterventionError, InterventionManager};
use datasynth_config::{GeneratorConfig, ScenarioSchemaConfig};
use datasynth_core::causal_dag::{CausalDAG, CausalDAGError};
use datasynth_core::{
    Intervention, InterventionTiming, InterventionType, OnsetType, ScenarioConstraints,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Errors from the scenario engine.
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("intervention error: {0}")]
    Intervention(#[from] InterventionError),
    #[error("propagation error: {0}")]
    Propagation(#[from] PropagationError),
    #[error("mutation error: {0}")]
    Mutation(#[from] MutationError),
    #[error("DAG error: {0}")]
    Dag(#[from] CausalDAGError),
    #[error("generation error: {0}")]
    Generation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Result of generating a single scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub baseline_path: PathBuf,
    pub counterfactual_path: PathBuf,
    pub interventions_applied: usize,
    pub months_affected: usize,
}

/// Orchestrates paired scenario generation.
pub struct ScenarioEngine {
    base_config: GeneratorConfig,
    causal_dag: CausalDAG,
}

impl ScenarioEngine {
    /// Create a new ScenarioEngine, loading the causal DAG from config.
    pub fn new(config: GeneratorConfig) -> Result<Self, ScenarioError> {
        let causal_dag = Self::load_causal_dag(&config)?;
        Ok(Self {
            base_config: config,
            causal_dag,
        })
    }

    /// Load the causal DAG from config presets or custom definition.
    fn load_causal_dag(config: &GeneratorConfig) -> Result<CausalDAG, ScenarioError> {
        let causal_config = &config.scenarios.causal_model;
        let mut dag: CausalDAG = match causal_config.preset.as_str() {
            "default" | "" => {
                let yaml =
                    include_str!("causal_dag_default.yaml");
                serde_yaml::from_str(yaml).map_err(|e| {
                    ScenarioError::Serialization(format!(
                        "failed to parse default causal DAG: {}",
                        e
                    ))
                })?
            }
            "minimal" => {
                use datasynth_core::causal_dag::{
                    CausalEdge, CausalNode, NodeCategory, TransferFunction,
                };
                // Minimal DAG: macro → operational → outcome (3 nodes, 2 edges)
                CausalDAG {
                    nodes: vec![
                        CausalNode {
                            id: "gdp_growth".to_string(),
                            label: "GDP Growth".to_string(),
                            category: NodeCategory::Macro,
                            baseline_value: 0.025,
                            bounds: Some((-0.10, 0.15)),
                            interventionable: true,
                            config_bindings: vec![],
                        },
                        CausalNode {
                            id: "transaction_volume".to_string(),
                            label: "Transaction Volume".to_string(),
                            category: NodeCategory::Operational,
                            baseline_value: 1.0,
                            bounds: Some((0.2, 3.0)),
                            interventionable: true,
                            config_bindings: vec!["transactions.volume_multiplier".to_string()],
                        },
                        CausalNode {
                            id: "error_rate".to_string(),
                            label: "Error Rate".to_string(),
                            category: NodeCategory::Outcome,
                            baseline_value: 0.02,
                            bounds: Some((0.0, 0.30)),
                            interventionable: false,
                            config_bindings: vec!["anomaly_injection.base_rate".to_string()],
                        },
                    ],
                    edges: vec![
                        CausalEdge {
                            from: "gdp_growth".to_string(),
                            to: "transaction_volume".to_string(),
                            transfer: TransferFunction::Linear {
                                coefficient: 0.8,
                                intercept: 1.0,
                            },
                            lag_months: 1,
                            strength: 1.0,
                            mechanism: Some("GDP growth drives transaction volume".to_string()),
                        },
                        CausalEdge {
                            from: "transaction_volume".to_string(),
                            to: "error_rate".to_string(),
                            transfer: TransferFunction::Linear {
                                coefficient: 0.01,
                                intercept: 0.0,
                            },
                            lag_months: 0,
                            strength: 1.0,
                            mechanism: Some("Higher volume increases error rate".to_string()),
                        },
                    ],
                    topological_order: vec![],
                }
            }
            other => {
                return Err(ScenarioError::Serialization(format!(
                    "unknown causal DAG preset: '{}'",
                    other
                )));
            }
        };

        dag.validate()?;
        Ok(dag)
    }

    /// Get a reference to the loaded causal DAG.
    pub fn causal_dag(&self) -> &CausalDAG {
        &self.causal_dag
    }

    /// Get a reference to the base config.
    pub fn base_config(&self) -> &GeneratorConfig {
        &self.base_config
    }

    /// Generate all scenarios defined in config.
    pub fn generate_all(&self, output_root: &Path) -> Result<Vec<ScenarioResult>, ScenarioError> {
        let scenarios = &self.base_config.scenarios.scenarios;
        let mut results = Vec::with_capacity(scenarios.len());

        // Create baseline directory
        let baseline_path = output_root.join("baseline");
        std::fs::create_dir_all(&baseline_path)?;

        // Generate each scenario
        for scenario in scenarios {
            let result = self.generate_scenario(scenario, &baseline_path, output_root)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Generate a single scenario: validate, propagate, mutate, produce output.
    pub fn generate_scenario(
        &self,
        scenario: &ScenarioSchemaConfig,
        baseline_path: &Path,
        output_root: &Path,
    ) -> Result<ScenarioResult, ScenarioError> {
        // 1. Convert schema config to core interventions
        let interventions = Self::convert_interventions(&scenario.interventions)?;

        // 2. Validate interventions
        let validated = InterventionManager::validate(&interventions, &self.base_config)?;

        // 3. Propagate through causal DAG
        let engine = CausalPropagationEngine::new(&self.causal_dag);
        let propagated = engine.propagate(&validated, self.base_config.global.period_months)?;

        // 4. Build constraints
        let constraints = ScenarioConstraints {
            preserve_accounting_identity: scenario.constraints.preserve_accounting_identity,
            preserve_document_chains: scenario.constraints.preserve_document_chains,
            preserve_period_close: scenario.constraints.preserve_period_close,
            preserve_balance_coherence: scenario.constraints.preserve_balance_coherence,
            custom: vec![],
        };

        // 5. Apply to config (creates mutated copy)
        let _mutated_config = ConfigMutator::apply(&self.base_config, &propagated, &constraints)?;

        // 6. Create scenario output directory
        let scenario_path = output_root
            .join("scenarios")
            .join(&scenario.name)
            .join("data");
        std::fs::create_dir_all(&scenario_path)?;

        // 7. Write scenario manifest
        let manifest = ScenarioManifest {
            scenario_name: scenario.name.clone(),
            description: scenario.description.clone(),
            interventions_count: interventions.len(),
            months_affected: propagated.changes_by_month.len(),
            config_paths_changed: propagated
                .changes_by_month
                .values()
                .flat_map(|changes| changes.iter().map(|c| c.path.clone()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        };

        let manifest_path = output_root
            .join("scenarios")
            .join(&scenario.name)
            .join("scenario_manifest.yaml");
        let manifest_yaml = serde_yaml::to_string(&manifest)
            .map_err(|e| ScenarioError::Serialization(e.to_string()))?;
        std::fs::write(&manifest_path, manifest_yaml)?;

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            baseline_path: baseline_path.to_path_buf(),
            counterfactual_path: scenario_path,
            interventions_applied: interventions.len(),
            months_affected: propagated.changes_by_month.len(),
        })
    }

    /// Convert schema-level intervention configs to core Intervention structs.
    fn convert_interventions(
        schema_interventions: &[datasynth_config::InterventionSchemaConfig],
    ) -> Result<Vec<Intervention>, ScenarioError> {
        let mut interventions = Vec::new();

        for schema in schema_interventions {
            let intervention_type: InterventionType =
                serde_json::from_value(schema.intervention_type.clone()).map_err(|e| {
                    ScenarioError::Serialization(format!(
                        "failed to parse intervention type: {}",
                        e
                    ))
                })?;

            let onset = match schema.timing.onset.to_lowercase().as_str() {
                "sudden" => OnsetType::Sudden,
                "gradual" => OnsetType::Gradual,
                "oscillating" => OnsetType::Oscillating,
                _ => OnsetType::Sudden,
            };

            interventions.push(Intervention {
                id: Uuid::new_v4(),
                intervention_type,
                timing: InterventionTiming {
                    start_month: schema.timing.start_month,
                    duration_months: schema.timing.duration_months,
                    onset,
                    ramp_months: schema.timing.ramp_months,
                },
                label: schema.label.clone(),
                priority: schema.priority,
            });
        }

        Ok(interventions)
    }

    /// List all scenarios in the config.
    pub fn list_scenarios(&self) -> Vec<ScenarioSummary> {
        self.base_config
            .scenarios
            .scenarios
            .iter()
            .map(|s| ScenarioSummary {
                name: s.name.clone(),
                description: s.description.clone(),
                tags: s.tags.clone(),
                intervention_count: s.interventions.len(),
                probability_weight: s.probability_weight,
            })
            .collect()
    }

    /// Validate all scenarios without generating.
    pub fn validate_all(&self) -> Vec<ScenarioValidationResult> {
        self.base_config
            .scenarios
            .scenarios
            .iter()
            .map(|s| {
                let result = self.validate_scenario(s);
                ScenarioValidationResult {
                    name: s.name.clone(),
                    valid: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                }
            })
            .collect()
    }

    /// Validate a single scenario.
    fn validate_scenario(&self, scenario: &ScenarioSchemaConfig) -> Result<(), ScenarioError> {
        let interventions = Self::convert_interventions(&scenario.interventions)?;
        let validated = InterventionManager::validate(&interventions, &self.base_config)?;
        let engine = CausalPropagationEngine::new(&self.causal_dag);
        let propagated = engine.propagate(&validated, self.base_config.global.period_months)?;

        let constraints = ScenarioConstraints {
            preserve_accounting_identity: scenario.constraints.preserve_accounting_identity,
            preserve_document_chains: scenario.constraints.preserve_document_chains,
            preserve_period_close: scenario.constraints.preserve_period_close,
            preserve_balance_coherence: scenario.constraints.preserve_balance_coherence,
            custom: vec![],
        };

        let _mutated = ConfigMutator::apply(&self.base_config, &propagated, &constraints)?;
        Ok(())
    }
}

/// Summary info for listing scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub intervention_count: usize,
    pub probability_weight: Option<f64>,
}

/// Result of validating a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioValidationResult {
    pub name: String,
    pub valid: bool,
    pub error: Option<String>,
}

/// Manifest written alongside scenario output.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioManifest {
    scenario_name: String,
    description: String,
    interventions_count: usize,
    months_affected: usize,
    config_paths_changed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_config::{
        InterventionSchemaConfig, InterventionTimingSchemaConfig, ScenarioConstraintsSchemaConfig,
        ScenarioOutputSchemaConfig, ScenariosConfig,
    };
    use datasynth_test_utils::fixtures::minimal_config;
    use tempfile::TempDir;

    fn config_with_scenario() -> GeneratorConfig {
        let mut config = minimal_config();
        config.scenarios = ScenariosConfig {
            enabled: true,
            scenarios: vec![ScenarioSchemaConfig {
                name: "test_recession".to_string(),
                description: "Test recession scenario".to_string(),
                tags: vec!["test".to_string()],
                base: None,
                probability_weight: Some(0.3),
                interventions: vec![InterventionSchemaConfig {
                    intervention_type: serde_json::json!({
                        "type": "parameter_shift",
                        "target": "global.period_months",
                        "to": 3,
                        "interpolation": "linear"
                    }),
                    timing: InterventionTimingSchemaConfig {
                        start_month: 1,
                        duration_months: None,
                        onset: "sudden".to_string(),
                        ramp_months: None,
                    },
                    label: Some("Test shift".to_string()),
                    priority: 0,
                }],
                constraints: ScenarioConstraintsSchemaConfig::default(),
                output: ScenarioOutputSchemaConfig::default(),
                metadata: Default::default(),
            }],
            causal_model: Default::default(),
            defaults: Default::default(),
        };
        config
    }

    #[test]
    fn test_scenario_engine_new_default_dag() {
        let config = config_with_scenario();
        let engine = ScenarioEngine::new(config).expect("should create engine");
        assert!(!engine.causal_dag().nodes.is_empty());
        assert!(!engine.causal_dag().edges.is_empty());
    }

    #[test]
    fn test_scenario_engine_list_scenarios() {
        let config = config_with_scenario();
        let engine = ScenarioEngine::new(config).expect("should create engine");
        let scenarios = engine.list_scenarios();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "test_recession");
        assert_eq!(scenarios[0].intervention_count, 1);
    }

    #[test]
    fn test_scenario_engine_validate_all() {
        let config = config_with_scenario();
        let engine = ScenarioEngine::new(config).expect("should create engine");
        let results = engine.validate_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].valid, "validation error: {:?}", results[0].error);
    }

    #[test]
    fn test_scenario_engine_converts_schema_to_interventions() {
        let config = config_with_scenario();
        let interventions =
            ScenarioEngine::convert_interventions(&config.scenarios.scenarios[0].interventions)
                .expect("should convert");
        assert_eq!(interventions.len(), 1);
        assert!(matches!(
            interventions[0].intervention_type,
            InterventionType::ParameterShift(_)
        ));
    }

    #[test]
    fn test_minimal_dag_preset_valid() {
        let mut config = minimal_config();
        config.scenarios = ScenariosConfig {
            enabled: true,
            scenarios: vec![ScenarioSchemaConfig {
                name: "minimal_test".to_string(),
                description: "Test with minimal DAG".to_string(),
                tags: vec![],
                base: None,
                probability_weight: None,
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
                    label: Some("Volume increase".to_string()),
                    priority: 0,
                }],
                constraints: ScenarioConstraintsSchemaConfig::default(),
                output: ScenarioOutputSchemaConfig::default(),
                metadata: Default::default(),
            }],
            causal_model: datasynth_config::CausalModelSchemaConfig {
                preset: "minimal".to_string(),
                ..Default::default()
            },
            defaults: Default::default(),
        };

        let engine = ScenarioEngine::new(config).expect("should create engine with minimal DAG");
        assert_eq!(engine.causal_dag().nodes.len(), 3);
        assert_eq!(engine.causal_dag().edges.len(), 2);

        // Validate all scenarios pass
        let results = engine.validate_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].valid, "validation error: {:?}", results[0].error);
    }

    #[test]
    fn test_scenario_engine_generates_output() {
        let config = config_with_scenario();
        let engine = ScenarioEngine::new(config).expect("should create engine");
        let tmpdir = TempDir::new().expect("should create tmpdir");
        let results = engine.generate_all(tmpdir.path()).expect("should generate");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scenario_name, "test_recession");
        // Manifest should exist
        let manifest_path = tmpdir
            .path()
            .join("scenarios")
            .join("test_recession")
            .join("scenario_manifest.yaml");
        assert!(manifest_path.exists());
    }
}
