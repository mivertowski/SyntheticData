use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::intervention::InterventionType;

/// A named, self-contained scenario definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Reference to base scenario (None = default config).
    pub base: Option<String>,
    /// For IFRS 9-style probability-weighted outcomes.
    pub probability_weight: Option<f64>,
    pub interventions: Vec<Intervention>,
    #[serde(default)]
    pub constraints: ScenarioConstraints,
    #[serde(default)]
    pub output: ScenarioOutputConfig,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// A single intervention that modifies the generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub id: Uuid,
    pub intervention_type: InterventionType,
    pub timing: InterventionTiming,
    /// Human-readable label for UI display.
    pub label: Option<String>,
    /// Priority for conflict resolution (higher wins).
    #[serde(default)]
    pub priority: u32,
}

/// When the intervention takes effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTiming {
    /// Month offset from generation start (1-indexed).
    pub start_month: u32,
    /// Duration in months (None = permanent from start_month).
    pub duration_months: Option<u32>,
    /// How the intervention ramps in.
    #[serde(default)]
    pub onset: OnsetType,
    /// Ramp-in period in months (for gradual/oscillating onset).
    pub ramp_months: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnsetType {
    /// Full effect immediately.
    #[default]
    Sudden,
    /// Linear ramp over ramp_months.
    Gradual,
    /// Sinusoidal oscillation.
    Oscillating,
    /// Custom easing curve.
    Custom { easing: EasingFunction },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step { steps: u32 },
}

/// What invariants must hold in the counterfactual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConstraints {
    /// Debits = Credits for all journal entries.
    #[serde(default = "default_true")]
    pub preserve_accounting_identity: bool,
    /// Document chain references remain valid.
    #[serde(default = "default_true")]
    pub preserve_document_chains: bool,
    /// Period close still executes.
    #[serde(default = "default_true")]
    pub preserve_period_close: bool,
    /// Balance sheet still balances at each period.
    #[serde(default = "default_true")]
    pub preserve_balance_coherence: bool,
    /// Custom constraints (config path -> value range).
    #[serde(default)]
    pub custom: Vec<CustomConstraint>,
}

impl Default for ScenarioConstraints {
    fn default() -> Self {
        Self {
            preserve_accounting_identity: true,
            preserve_document_chains: true,
            preserve_period_close: true,
            preserve_balance_coherence: true,
            custom: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConstraint {
    pub config_path: String,
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioOutputConfig {
    /// Generate baseline alongside counterfactual.
    #[serde(default = "default_true")]
    pub paired: bool,
    /// Which diff formats to produce.
    #[serde(default = "default_diff_formats")]
    pub diff_formats: Vec<DiffFormat>,
    /// Which output files to include in diff (empty = all).
    #[serde(default)]
    pub diff_scope: Vec<String>,
}

impl Default for ScenarioOutputConfig {
    fn default() -> Self {
        Self {
            paired: true,
            diff_formats: default_diff_formats(),
            diff_scope: Vec::new(),
        }
    }
}

fn default_diff_formats() -> Vec<DiffFormat> {
    vec![DiffFormat::Summary, DiffFormat::Aggregate]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiffFormat {
    /// High-level KPI impact summary.
    Summary,
    /// Record-by-record comparison.
    RecordLevel,
    /// Aggregated metric comparison.
    Aggregate,
    /// Which interventions caused which changes.
    InterventionTrace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_constraints_default_all_true() {
        let constraints = ScenarioConstraints::default();
        assert!(constraints.preserve_accounting_identity);
        assert!(constraints.preserve_document_chains);
        assert!(constraints.preserve_period_close);
        assert!(constraints.preserve_balance_coherence);
        assert!(constraints.custom.is_empty());
    }

    #[test]
    fn test_onset_type_variants() {
        // Sudden is default
        let onset: OnsetType = serde_json::from_str(r#""sudden""#).unwrap();
        assert!(matches!(onset, OnsetType::Sudden));

        let onset: OnsetType = serde_json::from_str(r#""gradual""#).unwrap();
        assert!(matches!(onset, OnsetType::Gradual));

        let onset: OnsetType = serde_json::from_str(r#""oscillating""#).unwrap();
        assert!(matches!(onset, OnsetType::Oscillating));

        // Custom with easing
        let onset: OnsetType = serde_json::from_str(r#"{"custom":{"easing":"ease_in"}}"#).unwrap();
        assert!(matches!(onset, OnsetType::Custom { .. }));
    }

    #[test]
    fn test_scenario_serde_roundtrip() {
        let scenario = Scenario {
            id: Uuid::new_v4(),
            name: "test_scenario".to_string(),
            description: "A test scenario".to_string(),
            tags: vec!["test".to_string()],
            base: None,
            probability_weight: Some(0.5),
            interventions: vec![],
            constraints: ScenarioConstraints::default(),
            output: ScenarioOutputConfig::default(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&scenario).unwrap();
        let deserialized: Scenario = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_scenario");
        assert_eq!(deserialized.probability_weight, Some(0.5));
        assert!(deserialized.constraints.preserve_accounting_identity);
        assert!(deserialized.output.paired);
    }

    #[test]
    fn test_scenario_output_config_defaults() {
        let config = ScenarioOutputConfig::default();
        assert!(config.paired);
        assert_eq!(config.diff_formats.len(), 2);
        assert!(config.diff_formats.contains(&DiffFormat::Summary));
        assert!(config.diff_formats.contains(&DiffFormat::Aggregate));
        assert!(config.diff_scope.is_empty());
    }

    #[test]
    fn test_diff_format_serde() {
        let format: DiffFormat = serde_json::from_str(r#""summary""#).unwrap();
        assert_eq!(format, DiffFormat::Summary);

        let format: DiffFormat = serde_json::from_str(r#""record_level""#).unwrap();
        assert_eq!(format, DiffFormat::RecordLevel);
    }
}
