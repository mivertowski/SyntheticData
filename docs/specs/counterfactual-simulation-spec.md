# Counterfactual & What-If Simulation — Technical Specification

**Date**: 2026-02-19
**Status**: Draft
**Companion to**: `2026-02-19-counterfactual-simulation-roadmap.md`
**Scope**: Phase 1 (Scenario Engine) + Phase 2 (Causal Propagation) detailed design

---

## 1. Overview

This specification defines the data structures, APIs, configuration schema, generation algorithm, and output formats for the Counterfactual Simulation Engine. It covers:

- Scenario definition schema (YAML)
- Intervention type system
- Paired generation algorithm
- Causal DAG and propagation engine
- Diff computation and output
- CLI commands
- Server API extensions
- Python wrapper extensions

---

## 2. Data Models

### 2.1 Core Types — `datasynth-core/src/models/scenario.rs`

```rust
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A named, self-contained scenario definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    /// Reference to base scenario (None = default config).
    pub base: Option<String>,
    /// For IFRS 9-style probability-weighted outcomes.
    pub probability_weight: Option<f64>,
    pub interventions: Vec<Intervention>,
    pub constraints: ScenarioConstraints,
    pub output: ScenarioOutputConfig,
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
    pub onset: OnsetType,
    /// Ramp-in period in months (for gradual/oscillating onset).
    pub ramp_months: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnsetType {
    /// Full effect immediately.
    Sudden,
    /// Linear ramp over ramp_months.
    Gradual,
    /// Sinusoidal oscillation.
    Oscillating,
    /// Custom easing curve.
    Custom { easing: EasingFunction },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub preserve_accounting_identity: bool,
    /// Document chain references remain valid.
    pub preserve_document_chains: bool,
    /// Period close still executes.
    pub preserve_period_close: bool,
    /// Balance sheet still balances at each period.
    pub preserve_balance_coherence: bool,
    /// Custom constraints (config path → value range).
    pub custom: Vec<CustomConstraint>,
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
    pub paired: bool,
    /// Which diff formats to produce.
    pub diff_formats: Vec<DiffFormat>,
    /// Which output files to include in diff (empty = all).
    pub diff_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

### 2.2 Intervention Types — `datasynth-core/src/models/intervention.rs`

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The full taxonomy of supported interventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterventionType {
    // ── Entity Events ──────────────────────────────
    EntityEvent(EntityEventIntervention),

    // ── Parameter Shifts ───────────────────────────
    ParameterShift(ParameterShiftIntervention),

    // ── Control Failures ───────────────────────────
    ControlFailure(ControlFailureIntervention),

    // ── Process Changes ────────────────────────────
    ProcessChange(ProcessChangeIntervention),

    // ── Macro Shocks ───────────────────────────────
    MacroShock(MacroShockIntervention),

    // ── Regulatory Changes ─────────────────────────
    RegulatoryChange(RegulatoryChangeIntervention),

    // ── Composite ──────────────────────────────────
    Composite(CompositeIntervention),

    // ── Custom ─────────────────────────────────────
    Custom(CustomIntervention),
}

// ── Entity Events ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEventIntervention {
    pub subtype: EntityEventType,
    pub target: EntityTarget,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityEventType {
    VendorDefault,
    CustomerChurn,
    EmployeeDeparture,
    NewVendorOnboarding,
    MergerAcquisition,
    VendorCollusion,
    CustomerConsolidation,
    KeyPersonRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTarget {
    /// Target by cluster type.
    pub cluster: Option<String>,
    /// Target by specific entity IDs.
    pub entity_ids: Option<Vec<String>>,
    /// Target by attribute filter (e.g., country = "US").
    pub filter: Option<HashMap<String, String>>,
    /// Number of entities to affect (random selection from filter).
    pub count: Option<u32>,
    /// Fraction of entities to affect (alternative to count).
    pub fraction: Option<f64>,
}

// ── Parameter Shifts ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShiftIntervention {
    /// Dot-path to config parameter.
    pub target: String,
    /// Original value (for documentation; auto-filled from config).
    pub from: Option<serde_json::Value>,
    /// New value.
    pub to: serde_json::Value,
    /// Interpolation method during ramp.
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationType {
    #[default]
    Linear,
    Exponential,
    Logistic { steepness: f64 },
    Step,
}

// ── Control Failures ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFailureIntervention {
    pub subtype: ControlFailureType,
    /// Control ID (e.g., "C003") or control category.
    pub control_target: ControlTarget,
    /// Effectiveness multiplier (0.0 = complete failure, 1.0 = normal).
    pub severity: f64,
    /// Whether the failure is detectable by monitoring.
    pub detectable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFailureType {
    EffectivenessReduction,
    CompleteBypass,
    IntermittentFailure { failure_probability: f64 },
    DelayedDetection { detection_lag_months: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ControlTarget {
    ById { control_id: String },
    ByCategory { coso_component: String },
    ByScope { scope: String },
    All,
}

// ── Process Changes ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChangeIntervention {
    pub subtype: ProcessChangeType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessChangeType {
    ApprovalThresholdChange,
    NewApprovalLevel,
    SystemMigration,
    ProcessAutomation,
    OutsourcingTransition,
    PolicyChange,
    ReorganizationRestructuring,
}

// ── Macro Shocks ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroShockIntervention {
    pub subtype: MacroShockType,
    /// Severity multiplier (1.0 = standard severity for the shock type).
    pub severity: f64,
    /// Named preset (maps to pre-configured parameter bundles).
    pub preset: Option<String>,
    /// Override individual macro parameters.
    pub overrides: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroShockType {
    Recession,
    InflationSpike,
    CurrencyCrisis,
    InterestRateShock,
    CommodityShock,
    PandemicDisruption,
    SupplyChainCrisis,
    CreditCrunch,
}

// ── Regulatory Changes ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryChangeIntervention {
    pub subtype: RegulatoryChangeType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegulatoryChangeType {
    NewStandardAdoption,
    MaterialityThresholdChange,
    ReportingRequirementChange,
    ComplianceThresholdChange,
    AuditStandardChange,
    TaxRateChange,
}

// ── Composite ──────────────────────────────────────

/// Bundles multiple interventions into a named package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeIntervention {
    pub name: String,
    pub description: String,
    /// Child interventions applied together.
    pub children: Vec<InterventionType>,
    /// Conflict resolution: first_wins, last_wins, average, error.
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    #[default]
    FirstWins,
    LastWins,
    Average,
    Error,
}

// ── Custom ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomIntervention {
    pub name: String,
    /// Config path → value mappings.
    pub config_overrides: HashMap<String, serde_json::Value>,
    /// Causal downstream effects to trigger.
    pub downstream_triggers: Vec<String>,
}
```

### 2.3 Causal DAG — `datasynth-core/src/models/causal_dag.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A directed acyclic graph defining causal relationships
/// between parameters in the generation model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalDAG {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
    /// Pre-computed topological order (filled at validation time).
    #[serde(skip)]
    pub topological_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    /// Unique identifier (matches config parameter path or abstract name).
    pub id: String,
    pub label: String,
    pub category: NodeCategory,
    /// Default/baseline value.
    pub baseline_value: f64,
    /// Valid range for this parameter.
    pub bounds: Option<(f64, f64)>,
    /// Whether this node can be directly intervened upon.
    pub interventionable: bool,
    /// Maps to config path(s) for actual generation parameters.
    pub config_bindings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    Macro,
    Operational,
    Control,
    Financial,
    Behavioral,
    Regulatory,
    Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub transfer: TransferFunction,
    /// Delay in months before the effect propagates.
    pub lag_months: u32,
    /// Strength multiplier (0.0 = no effect, 1.0 = full transfer).
    pub strength: f64,
    /// Human-readable description of the causal mechanism.
    pub mechanism: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferFunction {
    /// output = input * coefficient + intercept
    Linear {
        coefficient: f64,
        #[serde(default)]
        intercept: f64,
    },
    /// output = base * (1 + rate)^input
    Exponential {
        base: f64,
        rate: f64,
    },
    /// output = capacity / (1 + e^(-steepness * (input - midpoint)))
    Logistic {
        capacity: f64,
        midpoint: f64,
        steepness: f64,
    },
    /// output = capacity / (1 + e^(steepness * (input - midpoint)))
    InverseLogistic {
        capacity: f64,
        midpoint: f64,
        steepness: f64,
    },
    /// output = magnitude when input crosses threshold, else 0
    Step {
        threshold: f64,
        magnitude: f64,
    },
    /// output = magnitude when input > threshold, scaling linearly above
    Threshold {
        threshold: f64,
        magnitude: f64,
        #[serde(default = "default_saturation")]
        saturation: f64,
    },
    /// output = initial * e^(-decay_rate * time_since_trigger)
    Decay {
        initial: f64,
        decay_rate: f64,
    },
    /// Lookup table with linear interpolation between points.
    Piecewise {
        points: Vec<(f64, f64)>,
    },
}

fn default_saturation() -> f64 {
    f64::INFINITY
}

impl CausalDAG {
    /// Validate the graph is a DAG (no cycles) and compute topological order.
    pub fn validate(&mut self) -> Result<(), CausalDAGError> {
        // Kahn's algorithm for topological sort
        // ...
        todo!()
    }

    /// Given a set of interventions (node_id → new_value), propagate
    /// effects through the DAG in topological order.
    pub fn propagate(
        &self,
        interventions: &HashMap<String, f64>,
        month: u32,
    ) -> HashMap<String, f64> {
        // Forward pass in topological order
        // ...
        todo!()
    }

    /// Reverse stress test: given target conditions on outcome nodes,
    /// find minimal interventions on interventionable nodes.
    pub fn reverse_stress_test(
        &self,
        targets: &HashMap<String, (f64, Ordering)>,
        max_interventions: usize,
    ) -> Vec<HashMap<String, f64>> {
        // Constraint satisfaction / optimization
        // ...
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CausalDAGError {
    #[error("Cycle detected involving node: {0}")]
    CycleDetected(String),
    #[error("Unknown node referenced in edge: {0}")]
    UnknownNode(String),
    #[error("Duplicate node ID: {0}")]
    DuplicateNode(String),
    #[error("Intervention on non-interventionable node: {0}")]
    NonInterventionable(String),
}
```

---

## 3. Configuration Schema

### 3.1 YAML Schema Extension — `datasynth-config/src/schema.rs`

Add to the existing config schema:

```rust
/// Top-level scenario configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenariosConfig {
    /// Enable scenario generation.
    #[serde(default)]
    pub enabled: bool,
    /// Named scenarios to generate.
    #[serde(default)]
    pub scenarios: Vec<ScenarioSchemaConfig>,
    /// Causal DAG definition (or "default" for built-in).
    #[serde(default)]
    pub causal_model: CausalModelConfig,
    /// Global scenario defaults.
    #[serde(default)]
    pub defaults: ScenarioDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSchemaConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub base: Option<String>,
    pub probability_weight: Option<f64>,
    pub interventions: Vec<InterventionSchemaConfig>,
    #[serde(default)]
    pub constraints: ScenarioConstraintsConfig,
    #[serde(default)]
    pub output: ScenarioOutputSchemaConfig,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionSchemaConfig {
    #[serde(flatten)]
    pub intervention_type: InterventionTypeConfig,
    pub timing: InterventionTimingConfig,
    pub label: Option<String>,
    #[serde(default)]
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTimingConfig {
    pub start_month: u32,
    pub duration_months: Option<u32>,
    #[serde(default = "default_onset")]
    pub onset: String,        // "sudden", "gradual", "oscillating"
    pub ramp_months: Option<u32>,
}

fn default_onset() -> String {
    "sudden".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioConstraintsConfig {
    #[serde(default = "default_true")]
    pub preserve_accounting_identity: bool,
    #[serde(default = "default_true")]
    pub preserve_document_chains: bool,
    #[serde(default = "default_true")]
    pub preserve_period_close: bool,
    #[serde(default = "default_true")]
    pub preserve_balance_coherence: bool,
    #[serde(default)]
    pub custom: Vec<CustomConstraintConfig>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioOutputSchemaConfig {
    #[serde(default = "default_true")]
    pub paired: bool,
    #[serde(default = "default_diff_formats")]
    pub diff_formats: Vec<String>,
    #[serde(default)]
    pub diff_scope: Vec<String>,
}

fn default_diff_formats() -> Vec<String> {
    vec!["summary".into(), "aggregate".into()]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CausalModelConfig {
    /// "default", "minimal", "comprehensive", or "custom"
    #[serde(default = "default_causal_model")]
    pub preset: String,
    /// Custom nodes (merged with preset).
    #[serde(default)]
    pub nodes: Vec<CausalNodeConfig>,
    /// Custom edges (merged with preset).
    #[serde(default)]
    pub edges: Vec<CausalEdgeConfig>,
}

fn default_causal_model() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDefaults {
    #[serde(default)]
    pub constraints: ScenarioConstraintsConfig,
    #[serde(default)]
    pub output: ScenarioOutputSchemaConfig,
}
```

### 3.2 Example YAML

```yaml
# Full example configuration with scenarios
global:
  industry: manufacturing
  start_date: "2025-01-01"
  period_months: 12
  seed: 42

companies:
  - code: MFG001
    name: "Precision Manufacturing Inc."
    currency: USD
    country: US

# ... standard config sections ...

scenarios:
  enabled: true

  causal_model:
    preset: default          # use built-in manufacturing DAG
    edges:                   # add custom edges
      - from: raw_material_price
        to: cogs
        transfer: { type: linear, coefficient: 0.6 }
        lag_months: 1

  defaults:
    constraints:
      preserve_accounting_identity: true
      preserve_document_chains: true
    output:
      paired: true
      diff_formats: [summary, aggregate]

  scenarios:
    # ── Scenario 1: Supply Chain Disruption ────────────
    - name: supply_chain_disruption
      description: |
        Three strategic vendors default in Q3 due to financial
        distress. Company shifts to spot purchasing at higher
        prices while qualifying replacement vendors.
      tags: [supply_chain, risk, stress_test, manufacturing]
      probability_weight: 0.10

      interventions:
        - type: entity_event
          subtype: vendor_default
          target:
            cluster: reliable_strategic
            count: 3
          timing:
            start_month: 7
            duration_months: 4
            onset: sudden
          label: "Strategic vendor defaults"

        - type: parameter_shift
          target: distributions.amounts.components[0].mu
          to: 5.5
          interpolation: linear
          timing:
            start_month: 7
            ramp_months: 2
          label: "Reduced average transaction size"

        - type: macro_shock
          subtype: commodity_shock
          severity: 1.5
          overrides:
            raw_material_price_increase: 0.25
          timing:
            start_month: 7
            duration_months: 6
            onset: gradual
            ramp_months: 2
          label: "Raw material price spike"

      output:
        diff_formats: [summary, aggregate, record_level]

    # ── Scenario 2: Control Failure ────────────────────
    - name: sox_material_weakness
      description: |
        IT general control failure leads to unauthorized access
        to the financial reporting system. Three-way match control
        degrades. SOD violations go undetected for two months.
      tags: [controls, sox, audit, compliance]
      probability_weight: 0.05

      interventions:
        - type: control_failure
          subtype: effectiveness_reduction
          control_target:
            control_id: "C003"
          severity: 0.40
          detectable: false
          timing:
            start_month: 4
            duration_months: 3
            onset: gradual
            ramp_months: 1

        - type: control_failure
          subtype: complete_bypass
          control_target:
            scope: it_general_control
          severity: 0.0
          detectable: false
          timing:
            start_month: 4
            duration_months: 2
            onset: sudden

        - type: process_change
          subtype: policy_change
          parameters:
            sod_enforcement: disabled
            approval_override_rate: 0.15
          timing:
            start_month: 4
            duration_months: 2

    # ── Scenario 3: Recession ──────────────────────────
    - name: recession_moderate
      description: |
        Moderate recession beginning Q2. GDP declines 2%,
        unemployment rises, customer defaults increase.
      tags: [macro, recession, stress_test]
      probability_weight: 0.15

      interventions:
        - type: macro_shock
          subtype: recession
          severity: 1.0
          preset: moderate_recession
          timing:
            start_month: 4
            duration_months: 8
            onset: gradual
            ramp_months: 3
```

---

## 4. Scenario Engine Architecture

### 4.1 Component Diagram

```
                    ┌──────────────────────┐
                    │   CLI / Server / UI   │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │    ScenarioEngine     │
                    │                      │
                    │  ┌────────────────┐  │
                    │  │ InterventionMgr│  │ ← validates + resolves conflicts
                    │  └───────┬────────┘  │
                    │          │            │
                    │  ┌───────▼────────┐  │
                    │  │  CausalEngine  │  │ ← propagates through DAG
                    │  └───────┬────────┘  │
                    │          │            │
                    │  ┌───────▼────────┐  │
                    │  │  ConfigMutator │  │ ← applies changes to config
                    │  └───────┬────────┘  │
                    │          │            │
                    └──────────┼───────────┘
                               │
                ┌──────────────▼──────────────┐
                │   GenerationOrchestrator     │
                │   (existing, unmodified)     │
                │                              │
                │  baseline_config ──→ baseline │
                │  mutated_config ──→ counter.  │
                └──────────────┬──────────────┘
                               │
                    ┌──────────▼───────────┐
                    │     DiffEngine       │
                    │                      │
                    │  ┌────────────────┐  │
                    │  │ SummaryDiff    │  │
                    │  │ AggregateDiff  │  │
                    │  │ RecordDiff     │  │
                    │  │ InterventionTr │  │
                    │  └────────────────┘  │
                    └──────────────────────┘
```

### 4.2 `ScenarioEngine` — `datasynth-runtime/src/scenario_engine.rs`

```rust
/// Orchestrates paired scenario generation.
pub struct ScenarioEngine {
    base_config: ValidatedConfig,
    causal_dag: CausalDAG,
}

impl ScenarioEngine {
    pub fn new(config: ValidatedConfig) -> Result<Self, ScenarioError> {
        let causal_dag = Self::load_causal_dag(&config)?;
        Ok(Self {
            base_config: config,
            causal_dag,
        })
    }

    /// Generate all scenarios defined in config.
    pub fn generate_all(
        &self,
        output_root: &Path,
        progress: &ProgressTracker,
    ) -> Result<Vec<ScenarioResult>, ScenarioError> {
        let scenarios = &self.base_config.scenarios.scenarios;
        let mut results = Vec::with_capacity(scenarios.len());

        // 1. Generate baseline (shared across all scenarios)
        let baseline_path = output_root.join("baseline");
        let baseline_seed = self.base_config.global.seed;
        progress.set_phase("Generating baseline");
        self.generate_single(&self.base_config, baseline_seed, &baseline_path)?;

        // 2. Generate each scenario
        for scenario in scenarios {
            progress.set_phase(&format!("Generating scenario: {}", scenario.name));
            let result = self.generate_scenario(
                scenario,
                baseline_seed,
                &baseline_path,
                output_root,
            )?;
            results.push(result);
        }

        Ok(results)
    }

    /// Generate a single scenario.
    fn generate_scenario(
        &self,
        scenario: &Scenario,
        baseline_seed: u64,
        baseline_path: &Path,
        output_root: &Path,
    ) -> Result<ScenarioResult, ScenarioError> {
        // 1. Validate interventions
        let validated = InterventionManager::validate(
            &scenario.interventions,
            &self.base_config,
        )?;

        // 2. Propagate through causal DAG
        let propagated = self.causal_dag.propagate_interventions(
            &validated,
            self.base_config.global.period_months,
        )?;

        // 3. Apply to config (creates mutated copy)
        let mutated_config = ConfigMutator::apply(
            &self.base_config,
            &propagated,
            &scenario.constraints,
        )?;

        // 4. Generate counterfactual with SAME seed
        let scenario_path = output_root
            .join("scenarios")
            .join(&scenario.name)
            .join("data");
        self.generate_single(&mutated_config, baseline_seed, &scenario_path)?;

        // 5. Compute diff
        let diff = DiffEngine::compute(
            baseline_path,
            &scenario_path,
            &scenario.output,
        )?;

        // 6. Write scenario manifest
        let manifest_path = output_root
            .join("scenarios")
            .join(&scenario.name)
            .join("scenario_manifest.yaml");
        Self::write_manifest(scenario, &propagated, &diff, &manifest_path)?;

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            baseline_path: baseline_path.to_path_buf(),
            counterfactual_path: scenario_path,
            diff,
            interventions_applied: propagated,
        })
    }
}
```

### 4.3 `InterventionManager` — `datasynth-runtime/src/intervention_manager.rs`

```rust
/// Validates, resolves conflicts, and normalizes interventions.
pub struct InterventionManager;

impl InterventionManager {
    /// Validate a set of interventions against the config.
    pub fn validate(
        interventions: &[Intervention],
        config: &ValidatedConfig,
    ) -> Result<Vec<ValidatedIntervention>, InterventionError> {
        let mut validated = Vec::new();

        for intervention in interventions {
            // 1. Check that target exists
            Self::validate_target(&intervention.intervention_type, config)?;

            // 2. Check timing is within generation period
            Self::validate_timing(&intervention.timing, config)?;

            // 3. Check parameter bounds
            Self::validate_bounds(&intervention.intervention_type, config)?;

            validated.push(ValidatedIntervention {
                intervention: intervention.clone(),
                affected_config_paths: Self::resolve_config_paths(
                    &intervention.intervention_type,
                    config,
                ),
            });
        }

        // 4. Check for conflicts between interventions
        Self::check_conflicts(&validated)?;

        Ok(validated)
    }

    /// Resolve conflicts between overlapping interventions.
    fn check_conflicts(
        interventions: &[ValidatedIntervention],
    ) -> Result<(), InterventionError> {
        // For each pair of interventions, check if they modify the
        // same config paths during overlapping time periods.
        // If so, use priority to resolve (higher priority wins).
        // If same priority, return error.
        todo!()
    }
}
```

### 4.4 `ConfigMutator` — `datasynth-runtime/src/config_mutator.rs`

```rust
/// Applies interventions to a config, producing a new config.
pub struct ConfigMutator;

impl ConfigMutator {
    /// Create a mutated config by applying propagated interventions.
    pub fn apply(
        base: &ValidatedConfig,
        propagated: &PropagatedInterventions,
        constraints: &ScenarioConstraints,
    ) -> Result<ValidatedConfig, MutationError> {
        let mut mutated = base.deep_clone();

        // Apply each config path change
        for (month, changes) in &propagated.changes_by_month {
            for change in changes {
                Self::apply_change(&mut mutated, change, *month)?;
            }
        }

        // Validate constraints still hold
        Self::validate_constraints(&mutated, constraints)?;

        // Re-run config validation
        mutated.revalidate()?;

        Ok(mutated)
    }

    /// Apply a single config change by dot-path.
    fn apply_change(
        config: &mut ValidatedConfig,
        change: &ConfigChange,
        month: u32,
    ) -> Result<(), MutationError> {
        // Parse dot-path and navigate config tree
        // Apply value with interpolation based on onset type
        todo!()
    }
}
```

### 4.5 `CausalPropagationEngine` — `datasynth-runtime/src/causal_engine.rs`

```rust
/// Forward-propagates interventions through the causal DAG.
pub struct CausalPropagationEngine<'a> {
    dag: &'a CausalDAG,
}

impl<'a> CausalPropagationEngine<'a> {
    /// Propagate interventions for each month of the generation period.
    pub fn propagate(
        &self,
        interventions: &[ValidatedIntervention],
        period_months: u32,
    ) -> Result<PropagatedInterventions, PropagationError> {
        let mut result = PropagatedInterventions::new();

        for month in 1..=period_months {
            // 1. Compute direct intervention effects for this month
            let direct = self.compute_direct_effects(interventions, month);

            // 2. Forward-propagate through DAG in topological order
            let mut node_values: HashMap<String, f64> = HashMap::new();

            // Set intervened nodes
            for (node_id, value) in &direct {
                node_values.insert(node_id.clone(), *value);
            }

            // Propagate to non-intervened nodes
            for node_id in &self.dag.topological_order {
                if node_values.contains_key(node_id) {
                    continue; // Already set by direct intervention
                }

                // Compute from parents
                let parent_edges: Vec<_> = self.dag.edges.iter()
                    .filter(|e| e.to == *node_id)
                    .collect();

                if parent_edges.is_empty() {
                    continue; // Root node, use baseline
                }

                let mut accumulated = 0.0;
                let mut has_effect = false;

                for edge in parent_edges {
                    if let Some(&parent_val) = node_values.get(&edge.from) {
                        // Check lag
                        if month >= edge.lag_months {
                            let effect = edge.transfer.compute(parent_val)
                                * edge.strength;
                            accumulated += effect;
                            has_effect = true;
                        }
                    }
                }

                if has_effect {
                    // Clamp to node bounds
                    let node = self.dag.find_node(node_id).unwrap();
                    let clamped = if let Some((min, max)) = node.bounds {
                        accumulated.clamp(min, max)
                    } else {
                        accumulated
                    };
                    node_values.insert(node_id.clone(), clamped);
                }
            }

            // 3. Convert node values to config changes
            for (node_id, value) in &node_values {
                let node = self.dag.find_node(node_id).unwrap();
                for config_path in &node.config_bindings {
                    result.add_change(month, ConfigChange {
                        path: config_path.clone(),
                        value: serde_json::Value::from(*value),
                        source_node: node_id.clone(),
                        is_direct: direct.contains_key(node_id),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Compute the direct effect of interventions at a given month,
    /// accounting for timing, onset, and interpolation.
    fn compute_direct_effects(
        &self,
        interventions: &[ValidatedIntervention],
        month: u32,
    ) -> HashMap<String, f64> {
        let mut effects = HashMap::new();

        for vi in interventions {
            let timing = &vi.intervention.timing;

            // Check if intervention is active at this month
            if month < timing.start_month {
                continue;
            }
            if let Some(duration) = timing.duration_months {
                if month >= timing.start_month + duration {
                    continue;
                }
            }

            // Compute interpolation factor
            let factor = match timing.onset {
                OnsetType::Sudden => 1.0,
                OnsetType::Gradual => {
                    let ramp = timing.ramp_months.unwrap_or(1) as f64;
                    let elapsed = (month - timing.start_month) as f64;
                    (elapsed / ramp).min(1.0)
                }
                OnsetType::Oscillating => {
                    let elapsed = (month - timing.start_month) as f64;
                    let ramp = timing.ramp_months.unwrap_or(6) as f64;
                    let progress = elapsed / ramp;
                    0.5 * (1.0 - (std::f64::consts::PI * progress).cos())
                }
                OnsetType::Custom { ref easing } => {
                    Self::compute_easing(easing, month, timing)
                }
            };

            // Map intervention type to affected causal nodes
            let node_effects = self.intervention_to_node_effects(
                &vi.intervention.intervention_type,
                factor,
            );

            for (node_id, value) in node_effects {
                effects.insert(node_id, value);
            }
        }

        effects
    }
}
```

---

## 5. Diff Engine

### 5.1 Diff Types — `datasynth-eval/src/scenario_diff.rs`

```rust
/// High-level impact summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub scenario_name: String,
    pub generation_timestamp: DateTime<Utc>,
    pub interventions_applied: usize,
    pub kpi_impacts: Vec<KpiImpact>,
    pub financial_statement_impacts: FinancialStatementImpact,
    pub anomaly_impact: AnomalyImpact,
    pub control_impact: ControlImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiImpact {
    pub kpi_name: String,
    pub baseline_value: Decimal,
    pub counterfactual_value: Decimal,
    pub absolute_change: Decimal,
    pub percent_change: f64,
    pub direction: ChangeDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDirection {
    Increase,
    Decrease,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementImpact {
    pub revenue_change_pct: f64,
    pub cogs_change_pct: f64,
    pub gross_margin_change_pct: f64,
    pub net_income_change_pct: f64,
    pub total_assets_change_pct: f64,
    pub total_liabilities_change_pct: f64,
    pub cash_flow_change_pct: f64,
    pub top_changed_line_items: Vec<LineItemImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemImpact {
    pub line_item: String,
    pub baseline: Decimal,
    pub counterfactual: Decimal,
    pub change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyImpact {
    pub baseline_anomaly_count: usize,
    pub counterfactual_anomaly_count: usize,
    pub new_anomaly_types: Vec<String>,
    pub removed_anomaly_types: Vec<String>,
    pub anomaly_rate_change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImpact {
    pub controls_affected: usize,
    pub new_deficiencies: Vec<ControlDeficiency>,
    pub material_weakness_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDeficiency {
    pub control_id: String,
    pub control_name: String,
    pub baseline_effectiveness: f64,
    pub counterfactual_effectiveness: f64,
    pub classification: DeficiencyClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeficiencyClassification {
    Deficiency,
    SignificantDeficiency,
    MaterialWeakness,
}

/// Record-level diff for a single output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordLevelDiff {
    pub file_name: String,
    pub records_added: usize,
    pub records_removed: usize,
    pub records_modified: usize,
    pub records_unchanged: usize,
    /// Sample of changed records (capped at 1000).
    pub sample_changes: Vec<RecordChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordChange {
    pub record_id: String,
    pub change_type: RecordChangeType,
    pub field_changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub baseline_value: String,
    pub counterfactual_value: String,
}

/// Aggregate metric comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateComparison {
    pub metrics: Vec<MetricComparison>,
    pub period_comparisons: Vec<PeriodComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub category: String,
    pub baseline: f64,
    pub counterfactual: f64,
    pub delta: f64,
    pub delta_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub period: String,
    pub metrics: Vec<MetricComparison>,
}

/// Intervention trace — maps interventions to their effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTrace {
    pub traces: Vec<InterventionEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionEffect {
    pub intervention_label: String,
    pub intervention_type: String,
    pub timing: String,
    /// Causal chain from intervention to outcome.
    pub causal_path: Vec<CausalPathStep>,
    /// Direct metrics affected.
    pub metrics_affected: Vec<MetricComparison>,
    /// Records affected (count by file).
    pub records_affected: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalPathStep {
    pub node: String,
    pub value_change: f64,
    pub mechanism: String,
}
```

### 5.2 `DiffEngine` — `datasynth-eval/src/diff_engine.rs`

```rust
pub struct DiffEngine;

impl DiffEngine {
    /// Compute diffs between baseline and counterfactual outputs.
    pub fn compute(
        baseline_path: &Path,
        counterfactual_path: &Path,
        output_config: &ScenarioOutputConfig,
    ) -> Result<ScenarioDiff, DiffError> {
        let mut diff = ScenarioDiff::default();

        for format in &output_config.diff_formats {
            match format {
                DiffFormat::Summary => {
                    diff.summary = Some(Self::compute_summary(
                        baseline_path,
                        counterfactual_path,
                    )?);
                }
                DiffFormat::RecordLevel => {
                    diff.record_level = Some(Self::compute_record_level(
                        baseline_path,
                        counterfactual_path,
                        &output_config.diff_scope,
                    )?);
                }
                DiffFormat::Aggregate => {
                    diff.aggregate = Some(Self::compute_aggregate(
                        baseline_path,
                        counterfactual_path,
                    )?);
                }
                DiffFormat::InterventionTrace => {
                    diff.intervention_trace = Some(Self::compute_trace(
                        baseline_path,
                        counterfactual_path,
                    )?);
                }
            }
        }

        Ok(diff)
    }

    fn compute_summary(
        baseline: &Path,
        counterfactual: &Path,
    ) -> Result<ImpactSummary, DiffError> {
        // Load financial statements from both
        // Compare KPIs, line items, anomaly counts, control status
        // Produce impact summary
        todo!()
    }

    fn compute_record_level(
        baseline: &Path,
        counterfactual: &Path,
        scope: &[String],
    ) -> Result<Vec<RecordLevelDiff>, DiffError> {
        // For each output file in scope:
        //   Load both versions
        //   Join on primary key (usually UUID)
        //   Classify: added, removed, modified, unchanged
        //   For modified: compute field-level diffs
        todo!()
    }

    fn compute_aggregate(
        baseline: &Path,
        counterfactual: &Path,
    ) -> Result<AggregateComparison, DiffError> {
        // Compute aggregate metrics for both:
        //   - Total volume by period
        //   - Amount distributions
        //   - Entity counts
        //   - Balance metrics
        //   - Anomaly rates
        // Compare period-by-period
        todo!()
    }
}
```

---

## 6. Default Causal DAG

### 6.1 Built-in Financial Process DAG — `datasynth-config/src/templates/causal_dag_default.yaml`

```yaml
# Default Causal DAG for Financial Process Simulation
# Covers the primary causal relationships between macro conditions,
# operational parameters, and financial outcomes.

nodes:
  # ── Macro / External ───────────────────────
  - id: gdp_growth
    label: "GDP Growth Rate"
    category: macro
    baseline_value: 0.025
    bounds: [-0.10, 0.15]
    interventionable: true
    config_bindings: []

  - id: interest_rate
    label: "Interest Rate"
    category: macro
    baseline_value: 0.05
    bounds: [0.0, 0.20]
    interventionable: true
    config_bindings: []

  - id: inflation_rate
    label: "Inflation Rate"
    category: macro
    baseline_value: 0.02
    bounds: [-0.02, 0.25]
    interventionable: true
    config_bindings:
      - distributions.drift.economic_cycle.amplitude

  - id: unemployment_rate
    label: "Unemployment Rate"
    category: macro
    baseline_value: 0.04
    bounds: [0.02, 0.15]
    interventionable: true
    config_bindings: []

  # ── Operational ────────────────────────────
  - id: transaction_volume
    label: "Transaction Volume Multiplier"
    category: operational
    baseline_value: 1.0
    bounds: [0.2, 3.0]
    interventionable: true
    config_bindings:
      - transactions.volume_multiplier

  - id: staffing_pressure
    label: "Staffing Pressure"
    category: operational
    baseline_value: 1.0
    bounds: [0.5, 3.0]
    interventionable: false
    config_bindings: []

  - id: processing_lag
    label: "Processing Lag Days"
    category: operational
    baseline_value: 2.0
    bounds: [0.5, 30.0]
    interventionable: true
    config_bindings:
      - temporal_patterns.processing_lags.invoice_receipt_lag.mu

  - id: vendor_default_rate
    label: "Vendor Default Rate"
    category: operational
    baseline_value: 0.02
    bounds: [0.0, 0.30]
    interventionable: true
    config_bindings:
      - vendor_network.dependencies.max_single_vendor_concentration

  - id: customer_churn_rate
    label: "Customer Churn Rate"
    category: operational
    baseline_value: 0.08
    bounds: [0.0, 0.40]
    interventionable: true
    config_bindings:
      - customer_segmentation.lifecycle.churned_rate

  # ── Controls ───────────────────────────────
  - id: control_effectiveness
    label: "Control Effectiveness"
    category: control
    baseline_value: 0.95
    bounds: [0.0, 1.0]
    interventionable: true
    config_bindings:
      - internal_controls.exception_rate

  - id: sod_compliance
    label: "SOD Compliance Rate"
    category: control
    baseline_value: 0.99
    bounds: [0.5, 1.0]
    interventionable: true
    config_bindings:
      - internal_controls.sod_violation_rate

  # ── Financial Outcomes ─────────────────────
  - id: error_rate
    label: "Transaction Error Rate"
    category: outcome
    baseline_value: 0.02
    bounds: [0.0, 0.30]
    interventionable: false
    config_bindings:
      - anomaly_injection.base_rate

  - id: fraud_detection_rate
    label: "Fraud Detection Rate"
    category: outcome
    baseline_value: 0.85
    bounds: [0.0, 1.0]
    interventionable: false
    config_bindings: []

  - id: bad_debt_rate
    label: "Bad Debt Rate"
    category: outcome
    baseline_value: 0.01
    bounds: [0.0, 0.20]
    interventionable: false
    config_bindings: []

  - id: purchase_price_index
    label: "Purchase Price Index"
    category: financial
    baseline_value: 1.0
    bounds: [0.5, 3.0]
    interventionable: true
    config_bindings:
      - distributions.amounts.components[0].mu

  - id: revenue_growth
    label: "Revenue Growth"
    category: financial
    baseline_value: 0.05
    bounds: [-0.40, 0.50]
    interventionable: false
    config_bindings:
      - distributions.drift.amount_mean_drift

  - id: misstatement_risk
    label: "Material Misstatement Risk"
    category: outcome
    baseline_value: 0.02
    bounds: [0.0, 1.0]
    interventionable: false
    config_bindings: []

edges:
  # ── Macro → Operational ────────────────────
  - from: gdp_growth
    to: transaction_volume
    transfer: { type: linear, coefficient: 0.8, intercept: 1.0 }
    lag_months: 1
    strength: 0.7
    mechanism: "Economic growth drives transaction volume"

  - from: gdp_growth
    to: customer_churn_rate
    transfer: { type: linear, coefficient: -0.5, intercept: 0.08 }
    lag_months: 2
    strength: 0.6
    mechanism: "Growth reduces churn; contraction increases it"

  - from: gdp_growth
    to: vendor_default_rate
    transfer: { type: inverse_logistic, capacity: 0.20, midpoint: -0.02, steepness: 15.0 }
    lag_months: 3
    strength: 0.7
    mechanism: "Recession drives vendor defaults via logistic curve"

  - from: unemployment_rate
    to: staffing_pressure
    transfer: { type: inverse_logistic, capacity: 2.5, midpoint: 0.06, steepness: 20.0 }
    lag_months: 1
    strength: 0.5
    mechanism: "Low unemployment = hard to hire = staffing pressure"

  - from: inflation_rate
    to: purchase_price_index
    transfer: { type: linear, coefficient: 1.0, intercept: 1.0 }
    lag_months: 1
    strength: 0.8
    mechanism: "Inflation directly impacts purchase prices"

  - from: interest_rate
    to: bad_debt_rate
    transfer: { type: logistic, capacity: 0.15, midpoint: 0.08, steepness: 30.0 }
    lag_months: 3
    strength: 0.6
    mechanism: "Higher rates increase default probability"

  # ── Operational → Controls ─────────────────
  - from: staffing_pressure
    to: control_effectiveness
    transfer: { type: inverse_logistic, capacity: 0.95, midpoint: 1.8, steepness: 5.0 }
    lag_months: 0
    strength: 0.8
    mechanism: "Understaffing degrades control execution"

  - from: transaction_volume
    to: staffing_pressure
    transfer: { type: threshold, threshold: 1.3, magnitude: 0.5, saturation: 2.5 }
    lag_months: 0
    strength: 0.6
    mechanism: "Volume above 130% of capacity creates pressure"

  - from: staffing_pressure
    to: processing_lag
    transfer: { type: linear, coefficient: 3.0, intercept: 0.0 }
    lag_months: 0
    strength: 0.5
    mechanism: "Pressure increases processing delays"

  # ── Controls → Outcomes ────────────────────
  - from: control_effectiveness
    to: error_rate
    transfer: { type: inverse_logistic, capacity: 0.20, midpoint: 0.70, steepness: 8.0 }
    lag_months: 0
    strength: 0.9
    mechanism: "Weaker controls → more errors slip through"

  - from: control_effectiveness
    to: fraud_detection_rate
    transfer: { type: logistic, capacity: 0.95, midpoint: 0.60, steepness: 6.0 }
    lag_months: 0
    strength: 0.8
    mechanism: "Effective controls detect more fraud"

  - from: sod_compliance
    to: fraud_detection_rate
    transfer: { type: linear, coefficient: 0.3, intercept: 0.55 }
    lag_months: 0
    strength: 0.4
    mechanism: "SOD compliance is a secondary fraud deterrent"

  # ── Outcomes → Financial ───────────────────
  - from: customer_churn_rate
    to: revenue_growth
    transfer: { type: linear, coefficient: -2.0, intercept: 0.05 }
    lag_months: 1
    strength: 0.7
    mechanism: "Churn directly reduces revenue"

  - from: bad_debt_rate
    to: revenue_growth
    transfer: { type: linear, coefficient: -0.5, intercept: 0.0 }
    lag_months: 1
    strength: 0.3
    mechanism: "Bad debt reduces effective revenue"

  - from: error_rate
    to: misstatement_risk
    transfer: { type: logistic, capacity: 0.80, midpoint: 0.08, steepness: 25.0 }
    lag_months: 0
    strength: 0.9
    mechanism: "Errors accumulate into misstatement risk"

  - from: fraud_detection_rate
    to: misstatement_risk
    transfer: { type: linear, coefficient: -0.3, intercept: 0.30 }
    lag_months: 0
    strength: 0.5
    mechanism: "Better detection reduces undetected fraud → lower misstatement"
```

---

## 7. CLI Commands

### 7.1 Additions to `datasynth-cli/src/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Scenario management and generation
    Scenario(ScenarioArgs),
}

#[derive(Args)]
struct ScenarioArgs {
    #[command(subcommand)]
    command: ScenarioCommand,
}

#[derive(Subcommand)]
enum ScenarioCommand {
    /// List scenarios defined in config
    List {
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Validate scenario definitions
    Validate {
        #[arg(short, long)]
        config: PathBuf,
        /// Specific scenario to validate (default: all)
        #[arg(short, long)]
        scenario: Option<String>,
    },

    /// Generate scenarios (baseline + counterfactuals)
    Generate {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Specific scenario (default: all)
        #[arg(short, long)]
        scenario: Option<String>,
        /// Skip baseline if it already exists
        #[arg(long)]
        reuse_baseline: bool,
    },

    /// Compare two generated datasets
    Diff {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        counterfactual: PathBuf,
        /// Output format: summary, aggregate, record_level, all
        #[arg(short, long, default_value = "summary")]
        format: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run sensitivity analysis
    Sensitivity {
        #[arg(short, long)]
        config: PathBuf,
        /// Parameters to vary (comma-separated node IDs)
        #[arg(short, long)]
        parameters: String,
        /// Target metrics (comma-separated)
        #[arg(short, long)]
        targets: String,
        /// Number of steps per parameter
        #[arg(long, default_value = "5")]
        steps: u32,
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Reverse stress test: find scenarios producing target condition
    ReverseStress {
        #[arg(short, long)]
        config: PathBuf,
        /// Target condition (e.g., "misstatement_risk >= 0.5")
        #[arg(short, long)]
        target: String,
        /// Parameters to vary (comma-separated)
        #[arg(long)]
        free_variables: String,
        /// Max scenarios to find
        #[arg(long, default_value = "5")]
        max_scenarios: usize,
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import a .dss scenario file
    Import {
        #[arg(short, long)]
        file: PathBuf,
        /// Target config to merge into
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Export a scenario to .dss format
    Export {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short, long)]
        scenario: String,
        #[arg(short, long)]
        output: PathBuf,
    },
}
```

### 7.2 CLI Output Examples

```
$ datasynth-data scenario list --config config.yaml

Scenarios defined in config.yaml:
  1. supply_chain_disruption    [supply_chain, risk, stress_test]
     3 interventions, months 7-10, weight: 0.10
  2. sox_material_weakness      [controls, sox, audit]
     3 interventions, months 4-6, weight: 0.05
  3. recession_moderate         [macro, recession, stress_test]
     1 intervention, months 4-11, weight: 0.15

$ datasynth-data scenario validate --config config.yaml

Validating scenarios...
  supply_chain_disruption:
    ✓ Interventions valid (3/3)
    ✓ Timing within period (months 7-10 of 12)
    ✓ No conflicts detected
    ✓ Causal propagation: 8 downstream nodes affected
    ✓ Constraints satisfiable

  sox_material_weakness:
    ✓ Interventions valid (3/3)
    ✓ Timing within period (months 4-6 of 12)
    ⚠ Potential conflict: control_failure + process_change both affect
      control_effectiveness at month 4 (resolved by priority)
    ✓ Causal propagation: 5 downstream nodes affected
    ✓ Constraints satisfiable

  recession_moderate:
    ✓ Intervention valid (1/1)
    ✓ Timing within period (months 4-11 of 12)
    ✓ No conflicts detected
    ✓ Causal propagation: 12 downstream nodes affected
    ✓ Constraints satisfiable

All 3 scenarios validated successfully.
```

---

## 8. Server API Extensions

### 8.1 REST Endpoints — `datasynth-server/src/routes/`

```
POST /api/scenarios/validate
  Body: { config: <yaml_string>, scenario?: <name> }
  Response: { valid: bool, warnings: [...], errors: [...] }

POST /api/scenarios/generate
  Body: { config: <yaml_string>, scenario?: <name>, output_format: "csv"|"json"|"parquet" }
  Response: { job_id: <uuid>, status: "started" }

GET /api/scenarios/jobs/{job_id}
  Response: { status: "running"|"completed"|"failed", progress: 0.0-1.0, result?: ... }

GET /api/scenarios/jobs/{job_id}/diff
  Query: format=summary|aggregate|record_level|trace
  Response: <diff_data>

POST /api/scenarios/sensitivity
  Body: { config: <yaml_string>, parameters: [...], targets: [...], steps: 5 }
  Response: { job_id: <uuid> }

POST /api/scenarios/reverse-stress
  Body: { config: <yaml_string>, target: "...", free_variables: [...] }
  Response: { job_id: <uuid> }

GET /api/scenarios/causal-dag
  Query: config=<yaml_string>
  Response: { nodes: [...], edges: [...] }

WebSocket: /ws/scenarios/{job_id}
  Streams: progress updates, intervention trace events, completion
```

---

## 9. Python Wrapper Extensions

### 9.1 API Additions — `python/datasynth_py/`

```python
from datasynth_py import DataSynth, Config, Scenario, Intervention

ds = DataSynth()

# Define a scenario programmatically
scenario = Scenario(
    name="vendor_shock",
    description="Key vendor defaults",
    interventions=[
        Intervention.entity_event(
            subtype="vendor_default",
            target={"cluster": "strategic", "count": 2},
            timing={"start_month": 6, "duration_months": 3, "onset": "sudden"},
        ),
        Intervention.macro_shock(
            subtype="commodity_shock",
            severity=1.5,
            timing={"start_month": 6, "duration_months": 4, "onset": "gradual"},
        ),
    ],
)

# Generate paired datasets
result = ds.generate_scenario(
    config="config.yaml",
    scenario=scenario,
    output={"format": "csv", "sink": "temp_dir"},
)

# Access results
baseline_df = result.baseline.journal_entries()      # pandas DataFrame
counterfactual_df = result.counterfactual.journal_entries()
diff = result.diff

print(f"Revenue impact: {diff.summary.financial_statement_impacts.revenue_change_pct:.1%}")
print(f"New anomalies: {diff.summary.anomaly_impact.counterfactual_anomaly_count}")
print(f"Misstatement risk: {diff.summary.kpi_impacts['misstatement_risk'].counterfactual_value:.2%}")

# ML-ready output
pairs = result.to_ml_pairs()
# Returns list of (factual_features, counterfactual_features, treatment_label, ground_truth_effect)

# Sensitivity analysis
sensitivity = ds.sensitivity_analysis(
    config="config.yaml",
    parameters=["gdp_growth", "interest_rate", "vendor_default_rate"],
    targets=["net_income", "anomaly_count", "misstatement_risk"],
    steps=10,
)
sensitivity.tornado_plot("net_income")       # matplotlib figure
sensitivity.heatmap("gdp_growth", "interest_rate", "net_income")  # 2D heatmap

# Reverse stress test
stress_results = ds.reverse_stress_test(
    config="config.yaml",
    target="misstatement_risk >= 0.5",
    free_variables=["gdp_growth", "control_effectiveness", "staffing_pressure"],
    max_scenarios=5,
)
for s in stress_results:
    print(f"Scenario: {s.interventions} → misstatement_risk = {s.outcome:.2f}")
```

### 9.2 Blueprint Extensions

```python
from datasynth_py import blueprints

# Pre-built scenario bundles
config = blueprints.stress_test_suite()
# Includes: recession_mild, recession_severe, supply_chain_shock,
#           control_failure, regulatory_change

config = blueprints.fraud_training_scenarios()
# Includes: 10 fraud typology scenarios with ML-ready output

config = blueprints.audit_simulation()
# Includes: varying materiality, sample sizes, control effectiveness
```

---

## 10. Shared-Prefix Optimization

### 10.1 Performance Strategy

When an intervention starts at month N, months 1 through N-1 produce identical output for both baseline and counterfactual (given the same seed). We exploit this:

```rust
/// Shared-prefix optimization for paired generation.
pub struct SharedPrefixGenerator {
    /// Month at which the first intervention takes effect.
    divergence_month: u32,
}

impl SharedPrefixGenerator {
    pub fn generate(
        &self,
        base_config: &ValidatedConfig,
        mutated_config: &ValidatedConfig,
        seed: u64,
        baseline_path: &Path,
        counterfactual_path: &Path,
    ) -> Result<(), GenerationError> {
        // Phase 1: Generate shared prefix (months 1..divergence_month)
        let prefix_state = self.generate_prefix(base_config, seed)?;

        // Phase 2: Write shared prefix to BOTH output dirs
        self.write_prefix(&prefix_state, baseline_path)?;
        self.write_prefix(&prefix_state, counterfactual_path)?;

        // Phase 3: Fork from prefix state
        let baseline_state = prefix_state.clone();
        let counterfactual_state = prefix_state;

        // Phase 4: Generate divergent portions in parallel
        rayon::join(
            || self.generate_suffix(base_config, baseline_state, baseline_path),
            || self.generate_suffix(mutated_config, counterfactual_state, counterfactual_path),
        );

        Ok(())
    }
}
```

**Performance characteristics**:

| Scenario | Intervention Start | Overhead vs. Single Generation |
|----------|-------------------|-------------------------------|
| Month 1 (immediate) | ~2.0x (no sharing) |
| Month 4 of 12 | ~1.67x (25% shared) |
| Month 7 of 12 | ~1.50x (50% shared) |
| Month 10 of 12 | ~1.25x (75% shared) |

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Causal DAG
    #[test]
    fn test_dag_cycle_detection() { ... }
    #[test]
    fn test_dag_topological_sort() { ... }
    #[test]
    fn test_transfer_function_linear() { ... }
    #[test]
    fn test_transfer_function_logistic() { ... }
    #[test]
    fn test_propagation_single_node() { ... }
    #[test]
    fn test_propagation_chain() { ... }
    #[test]
    fn test_propagation_with_lag() { ... }

    // Interventions
    #[test]
    fn test_intervention_timing_bounds() { ... }
    #[test]
    fn test_intervention_conflict_detection() { ... }
    #[test]
    fn test_intervention_sudden_onset() { ... }
    #[test]
    fn test_intervention_gradual_onset() { ... }

    // Config mutation
    #[test]
    fn test_config_dot_path_resolution() { ... }
    #[test]
    fn test_config_mutation_preserves_other_fields() { ... }
    #[test]
    fn test_constraint_validation_accounting_identity() { ... }
}
```

### 11.2 Integration Tests

```rust
#[test]
fn test_paired_generation_shared_prefix_identical() {
    // Generate baseline + counterfactual with intervention at month 7
    // Verify that months 1-6 output is byte-identical
}

#[test]
fn test_paired_generation_diverges_at_intervention() {
    // Generate with parameter_shift intervention
    // Verify that counterfactual differs from baseline at/after intervention month
}

#[test]
fn test_accounting_identity_preserved_in_counterfactual() {
    // Generate counterfactual with various interventions
    // Verify debits == credits for every journal entry
}

#[test]
fn test_document_chain_integrity_in_counterfactual() {
    // Generate counterfactual with entity_event interventions
    // Verify all document references are valid
}

#[test]
fn test_scenario_determinism() {
    // Generate same scenario twice with same seed
    // Verify byte-identical output
}

#[test]
fn test_multiple_scenarios_share_baseline() {
    // Generate 3 scenarios
    // Verify all reference the same baseline
}
```

### 11.3 Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_causal_dag_propagation_monotonic(
        intervention_value in -0.1f64..0.1,
    ) {
        // Larger GDP decline → larger vendor default rate
        // (monotonicity of causal chain)
    }

    #[test]
    fn prop_transfer_function_in_bounds(
        input in -10.0f64..10.0,
    ) {
        // All transfer functions produce output within node bounds
    }

    #[test]
    fn prop_intervention_timing_within_period(
        start in 1u32..12,
        duration in 1u32..6,
    ) {
        // Interventions are clamped to generation period
    }
}
```

---

## 12. File Placement Summary

```
datasynth-core/src/models/
  ├── scenario.rs          (NEW) Scenario, Intervention, Constraints
  ├── intervention.rs      (NEW) InterventionType taxonomy
  ├── causal_dag.rs        (NEW) CausalDAG, CausalNode, CausalEdge, TransferFunction
  └── mod.rs               (EDIT) add pub mod scenario, intervention, causal_dag

datasynth-config/src/
  ├── schema.rs            (EDIT) add ScenariosConfig section
  ├── validation.rs        (EDIT) add scenario validation rules
  └── templates/
      └── causal_dag_default.yaml  (NEW) default financial process DAG

datasynth-runtime/src/
  ├── scenario_engine.rs       (NEW) ScenarioEngine orchestrator
  ├── intervention_manager.rs  (NEW) validation + conflict resolution
  ├── config_mutator.rs        (NEW) config path mutation
  ├── causal_engine.rs         (NEW) DAG propagation
  └── mod.rs                   (EDIT) add pub mod scenario_engine, etc.

datasynth-eval/src/
  ├── scenario_diff.rs     (NEW) diff types
  ├── diff_engine.rs       (NEW) diff computation
  └── mod.rs               (EDIT) add pub mod scenario_diff, diff_engine

datasynth-cli/src/
  └── main.rs              (EDIT) add Scenario subcommand

datasynth-server/src/routes/
  └── scenarios.rs         (NEW) REST + WebSocket endpoints

python/datasynth_py/
  ├── scenario.py          (NEW) Python Scenario/Intervention classes
  └── __init__.py          (EDIT) export new classes

docs/specs/
  └── counterfactual-simulation-spec.md  (THIS FILE)

docs/plans/
  └── 2026-02-19-counterfactual-simulation-roadmap.md  (COMPANION)
```

---

## 13. Migration & Backward Compatibility

### 13.1 Existing `CounterfactualGenerator` Integration

The existing `CounterfactualGenerator` in `datasynth-generators/src/counterfactual/mod.rs` is preserved but repositioned:

- **Before**: Standalone generator creating single-JE counterfactual pairs
- **After**: Used internally by the Scenario Engine for record-level counterfactual generation within a broader scenario context

```rust
// The existing CounterfactualSpec maps cleanly to intervention types:
// ScaleAmount       → ParameterShift (amount distribution)
// ShiftDate         → ParameterShift (temporal)
// ReclassifyAccount → ProcessChange (account mapping)
// SplitTransaction  → EntityEvent (fraud: structuring)
// CreateRoundTrip   → EntityEvent (fraud: round_tripping)
// SelfApprove       → ControlFailure (approval override)
// InjectFraud       → Composite (maps to specific fraud scenario)
```

### 13.2 Config Backward Compatibility

Configs without `scenarios:` section work identically to today. The `scenarios` section is fully optional:

```yaml
# This config continues to work unchanged:
global:
  industry: manufacturing
  period_months: 12
companies:
  - code: MFG001
    # ...
# No scenarios section = no counterfactual generation
```

---

## 14. Open Questions

| # | Question | Options | Recommendation |
|---|----------|---------|----------------|
| 1 | Should scenarios support **inheritance** (scenario B extends scenario A)? | Yes / No / Phase 2 | Yes — enables composable scenario trees |
| 2 | Should the causal DAG be **user-editable in YAML** or **code-only** initially? | YAML / Code / Both | Both — YAML for config, code for custom transfer functions |
| 3 | Should diff computation be **streaming** (during generation) or **post-hoc** (after both datasets complete)? | Streaming / Post-hoc | Post-hoc for Phase 1; streaming for Phase 3 UI |
| 4 | Should we support **probability-weighted expected values** across scenarios (IFRS 9-style)? | Phase 1 / Phase 2 | Phase 1 — the `probability_weight` field is already in the schema |
| 5 | Maximum number of concurrent scenarios? | Unlimited / Configurable | Configurable, default 10 (memory guard integration) |
| 6 | Should record-level diffs use **UUIDs** or **semantic keys** for matching? | UUID / Semantic / Both | UUID primary, semantic fallback (some records may have different UUIDs) |
