# Counterfactual & What-If Simulation Engine — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Counterfactual Simulation Engine per `docs/specs/counterfactual-simulation-spec.md` — enabling paired baseline/counterfactual dataset generation with causal DAG propagation, config mutation, and diff computation.

**Architecture:** The spec defines a layered pipeline: Scenario config → InterventionManager (validate) → CausalPropagationEngine (propagate through DAG) → ConfigMutator (apply to config) → GenerationOrchestrator (generate with same seed) → DiffEngine (compare outputs). The existing `datasynth-core/src/causal/` infrastructure (CausalGraph, SCM, InterventionEngine, CounterfactualGenerator) is preserved; the new engine sits *above* it as a scenario orchestrator.

**Spec:** `docs/specs/counterfactual-simulation-spec.md`

---

## Phase 1: Core Models (datasynth-core)

### Task 1: Scenario & Intervention Data Models

**Files:**
- Create: `crates/datasynth-core/src/models/scenario.rs`
- Create: `crates/datasynth-core/src/models/intervention.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

**Step 1:** Create `scenario.rs` with types from spec §2.1:
- `Scenario` (id, name, description, tags, base, probability_weight, interventions, constraints, output, metadata)
- `Intervention` (id, intervention_type, timing, label, priority)
- `InterventionTiming` (start_month, duration_months, onset, ramp_months)
- `OnsetType` (Sudden, Gradual, Oscillating, Custom)
- `EasingFunction` (Linear, EaseIn, EaseOut, EaseInOut, Step)
- `ScenarioConstraints` (preserve_accounting_identity, preserve_document_chains, preserve_period_close, preserve_balance_coherence, custom)
- `CustomConstraint` (config_path, min, max, description)
- `ScenarioOutputConfig` (paired, diff_formats, diff_scope)
- `DiffFormat` enum (Summary, RecordLevel, Aggregate, InterventionTrace)

All types derive `Debug, Clone, Serialize, Deserialize`. `ScenarioConstraints` defaults to all-true. `ScenarioOutputConfig` defaults to paired=true, diff_formats=[Summary, Aggregate].

**Step 2:** Create `intervention.rs` with types from spec §2.2:
- `InterventionType` enum (EntityEvent, ParameterShift, ControlFailure, ProcessChange, MacroShock, RegulatoryChange, Composite, Custom) — serde tag="type", rename_all="snake_case"
- `EntityEventIntervention` with `EntityEventType` (8 variants) and `EntityTarget`
- `ParameterShiftIntervention` with `InterpolationType` (Linear, Exponential, Logistic, Step)
- `ControlFailureIntervention` with `ControlFailureType` and `ControlTarget`
- `ProcessChangeIntervention` with `ProcessChangeType` (7 variants)
- `MacroShockIntervention` with `MacroShockType` (8 variants)
- `RegulatoryChangeIntervention` with `RegulatoryChangeType` (6 variants)
- `CompositeIntervention` with `ConflictResolution`
- `CustomIntervention`

**Step 3:** Update `models/mod.rs` — add `mod scenario; mod intervention;` and `pub use scenario::*; pub use intervention::*;`

**Step 4:** Add basic unit tests:
- `test_scenario_serde_roundtrip` — serialize/deserialize a Scenario
- `test_intervention_type_tagged_serde` — verify serde tag dispatch works
- `test_scenario_constraints_default_all_true`
- `test_onset_type_variants`

**Step 5:** `cargo check -p datasynth-core && cargo test -p datasynth-core -- scenario && cargo test -p datasynth-core -- intervention`

**Step 6:** Commit: `feat(core): add Scenario and InterventionType data models (spec §2.1-2.2)`

---

### Task 2: CausalDAG Model with TransferFunction

**Files:**
- Create: `crates/datasynth-core/src/models/causal_dag.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

**Step 1:** Create `causal_dag.rs` with types from spec §2.3:
- `CausalDAG` (nodes, edges, topological_order — skip-serialized)
- `CausalNode` (id, label, category, baseline_value, bounds, interventionable, config_bindings)
- `NodeCategory` enum (Macro, Operational, Control, Financial, Behavioral, Regulatory, Outcome)
- `CausalEdge` (from, to, transfer, lag_months, strength, mechanism description)
- `TransferFunction` enum with 8 variants (Linear, Exponential, Logistic, InverseLogistic, Step, Threshold, Decay, Piecewise) — serde tag="type", rename_all="snake_case"
- `CausalDAGError` error enum (CycleDetected, UnknownNode, DuplicateNode, NonInterventionable)

**Step 2:** Implement `CausalDAG` methods:
- `validate(&mut self) -> Result<(), CausalDAGError>` — Kahn's algorithm for topological sort (same as existing `CausalGraph::topological_order` but stores result in `self.topological_order`), check for self-loops, unknown nodes, duplicate IDs
- `find_node(&self, id: &str) -> Option<&CausalNode>`
- `propagate(&self, interventions: &HashMap<String, f64>, month: u32) -> HashMap<String, f64>` — forward pass in topological order, applying TransferFunction with lag_months

**Step 3:** Implement `TransferFunction::compute(&self, input: f64) -> f64` for all 8 variants:
- Linear: `input * coefficient + intercept`
- Exponential: `base * (1.0 + rate).powf(input)`
- Logistic: `capacity / (1.0 + (-steepness * (input - midpoint)).exp())`
- InverseLogistic: `capacity / (1.0 + (steepness * (input - midpoint)).exp())`
- Step: `if input > threshold { magnitude } else { 0.0 }`
- Threshold: `if input > threshold { (magnitude * (input - threshold) / threshold).min(saturation) } else { 0.0 }`
- Decay: `initial * (-decay_rate * input).exp()`
- Piecewise: linear interpolation between sorted points

**Step 4:** Update `models/mod.rs` — add `pub mod causal_dag; pub use causal_dag::*;`

**Step 5:** Add unit tests:
- `test_transfer_function_linear` — compute(2.0) with coeff=0.5, intercept=1.0 → 2.0
- `test_transfer_function_logistic` — sigmoid midpoint returns capacity/2
- `test_transfer_function_exponential`
- `test_transfer_function_step`
- `test_transfer_function_threshold`
- `test_transfer_function_decay`
- `test_transfer_function_piecewise` — interpolation between known points
- `test_dag_validate_acyclic` — valid DAG passes
- `test_dag_validate_cycle_detected` — cycle returns error
- `test_dag_validate_unknown_node` — reference to nonexistent node returns error
- `test_dag_validate_duplicate_node` — duplicate ID returns error
- `test_dag_propagate_chain` — A→B→C, intervene on A, verify B and C change
- `test_dag_propagate_with_lag` — lag_months=2, verify month 1 has no effect

**Step 6:** `cargo check -p datasynth-core && cargo test -p datasynth-core -- causal_dag`

**Step 7:** Commit: `feat(core): add CausalDAG with 8 TransferFunction variants and propagation engine (spec §2.3)`

---

### Task 3: Default Causal DAG Template

**Files:**
- Create: `crates/datasynth-config/src/templates/causal_dag_default.yaml`

**Step 1:** Create the YAML file from spec §6.1 — 16 nodes (gdp_growth, interest_rate, inflation_rate, unemployment_rate, transaction_volume, staffing_pressure, processing_lag, vendor_default_rate, customer_churn_rate, control_effectiveness, sod_compliance, error_rate, fraud_detection_rate, bad_debt_rate, purchase_price_index, revenue_growth, misstatement_risk) and 14 edges with transfer functions, lag_months, and config_bindings.

**Step 2:** Verify the YAML parses with a test:
```rust
#[test]
fn test_default_causal_dag_parses() {
    let yaml = include_str!("../../templates/causal_dag_default.yaml");
    let dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();
    assert_eq!(dag.nodes.len(), 16); // or 17 with misstatement_risk
}
```

**Step 3:** `cargo test -p datasynth-config -- causal_dag`

**Step 4:** Commit: `feat(config): add default financial process causal DAG template (spec §6.1)`

---

## Phase 2: Config Schema Extension (datasynth-config)

### Task 4: ScenariosConfig Schema

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs`
- Modify: `crates/datasynth-config/src/validation.rs`

**Step 1:** Add to `schema.rs` the following types from spec §3.1:
- `ScenariosConfig` (enabled, scenarios, causal_model, defaults) — derives Default
- `ScenarioSchemaConfig` (name, description, tags, base, probability_weight, interventions, constraints, output, metadata)
- `InterventionSchemaConfig` (intervention_type flattened, timing, label, priority)
- `InterventionTimingConfig` (start_month, duration_months, onset as String, ramp_months)
- `ScenarioConstraintsConfig` (preserve_* fields defaulting to true, custom vec)
- `CustomConstraintConfig` (config_path, min, max, description)
- `ScenarioOutputSchemaConfig` (paired default true, diff_formats default ["summary","aggregate"], diff_scope)
- `CausalModelConfig` (preset default "default", nodes, edges)
- `CausalNodeConfig` (id, label, category, baseline_value, bounds, interventionable, config_bindings)
- `CausalEdgeConfig` (from, to, transfer as serde_json::Value, lag_months, strength, mechanism)
- `ScenarioDefaults` (constraints, output)
- `InterventionTypeConfig` — tagged enum mirroring core `InterventionType` but with looser serde for YAML

**Step 2:** Add `scenarios: ScenariosConfig` field to `GeneratorConfig` struct with `#[serde(default)]`.

**Step 3:** In `validation.rs`, add `validate_scenarios(config: &ScenariosConfig) -> Vec<ValidationWarning>`:
- If enabled, at least one scenario must be defined
- Each scenario must have a name
- Intervention start_month must be >= 1
- probability_weights must sum to <= 1.0 (if present)
- Onset string must be one of: "sudden", "gradual", "oscillating", "custom"

**Step 4:** Wire `validate_scenarios` into the main `validate_config()` function.

**Step 5:** Add tests:
- `test_scenarios_config_deserialize_full` — parse the example YAML from spec §3.2
- `test_scenarios_config_default_disabled` — default is disabled with empty scenarios
- `test_config_without_scenarios_backward_compatible` — existing configs parse fine
- `test_scenario_validation_rejects_invalid` — no name, start_month=0, bad onset

**Step 6:** `cargo check -p datasynth-config && cargo test -p datasynth-config -- scenario`

**Step 7:** Commit: `feat(config): add ScenariosConfig schema with validation (spec §3.1)`

---

## Phase 3: Runtime Engine (datasynth-runtime)

### Task 5: CausalPropagationEngine

**Files:**
- Create: `crates/datasynth-runtime/src/causal_engine.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `causal_engine.rs` with types from spec §4.5:
- `ValidatedIntervention` (intervention, affected_config_paths)
- `PropagatedInterventions` (changes_by_month: BTreeMap<u32, Vec<ConfigChange>>)
- `ConfigChange` (path, value as serde_json::Value, source_node, is_direct)
- `PropagationError` enum

**Step 2:** Implement `CausalPropagationEngine`:
- `new(dag: &CausalDAG) -> Self`
- `propagate(&self, interventions: &[ValidatedIntervention], period_months: u32) -> Result<PropagatedInterventions, PropagationError>`
  - For each month 1..=period_months: compute direct effects (with timing/onset interpolation), propagate through DAG in topological order, convert node values to ConfigChanges via config_bindings
- `compute_direct_effects(&self, interventions: &[ValidatedIntervention], month: u32) -> HashMap<String, f64>`
  - Check timing active, compute onset factor (Sudden=1.0, Gradual=linear ramp, Oscillating=cosine)
  - Map intervention type to affected causal nodes

**Step 3:** Register in `lib.rs`: `pub mod causal_engine;`

**Step 4:** Add tests:
- `test_propagation_no_interventions` — empty → no changes
- `test_propagation_sudden_onset` — full effect from start_month
- `test_propagation_gradual_onset` — linear ramp factor 0..1
- `test_propagation_chain_through_dag` — intervention on gdp_growth → transaction_volume → downstream
- `test_propagation_lag_respected` — effect delayed by lag_months
- `test_propagation_node_bounds_clamped` — values stay within bounds

**Step 5:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- causal_engine`

**Step 6:** Commit: `feat(runtime): add CausalPropagationEngine with timing and onset interpolation (spec §4.5)`

---

### Task 6: InterventionManager

**Files:**
- Create: `crates/datasynth-runtime/src/intervention_manager.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `intervention_manager.rs` with types from spec §4.3:
- `InterventionError` enum (InvalidTarget, TimingOutOfRange, ConflictDetected, BoundsViolation)

**Step 2:** Implement `InterventionManager`:
- `validate(interventions: &[Intervention], config: &GeneratorConfig) -> Result<Vec<ValidatedIntervention>, InterventionError>`
  - `validate_timing` — start_month >= 1, start_month <= period_months
  - `validate_bounds` — severity in [0.0, 1.0] for control failures, etc.
  - `resolve_config_paths` — maps InterventionType to affected config dot-paths
- `check_conflicts(validated: &[ValidatedIntervention]) -> Result<(), InterventionError>`
  - Check overlapping time windows on same config paths; resolve by priority

**Step 3:** Register in `lib.rs`: `pub mod intervention_manager;`

**Step 4:** Add tests:
- `test_validate_timing_out_of_range` — start_month > period_months → error
- `test_validate_empty_interventions` — empty vec is OK (returns empty validated)
- `test_validate_parameter_shift` — valid shift passes
- `test_conflict_detection` — two interventions on same path, same time, same priority → error
- `test_conflict_resolution_by_priority` — different priorities → higher wins

**Step 5:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- intervention_manager`

**Step 6:** Commit: `feat(runtime): add InterventionManager with validation and conflict resolution (spec §4.3)`

---

### Task 7: ConfigMutator

**Files:**
- Create: `crates/datasynth-runtime/src/config_mutator.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `config_mutator.rs` with types from spec §4.4:
- `MutationError` enum (PathNotFound, TypeMismatch, ConstraintViolation, ValidationFailed)

**Step 2:** Implement `ConfigMutator`:
- `apply(base: &GeneratorConfig, propagated: &PropagatedInterventions, constraints: &ScenarioConstraints) -> Result<GeneratorConfig, MutationError>`
  - Deep-clone the base config
  - For changes that target known config paths, apply them to the cloned config
  - Strategy: serialize config to serde_json::Value, navigate dot-path, set value, deserialize back
- `apply_at_path(value: &mut serde_json::Value, path: &str, new_value: &serde_json::Value) -> Result<(), MutationError>` — dot-path navigation with array index support (e.g., `distributions.amounts.components[0].mu`)
- `validate_constraints(config: &GeneratorConfig, constraints: &ScenarioConstraints) -> Result<(), MutationError>` — check custom constraint bounds

**Step 3:** Register in `lib.rs`: `pub mod config_mutator;`

**Step 4:** Add tests:
- `test_apply_simple_dot_path` — "global.seed" → changes seed
- `test_apply_nested_dot_path` — "distributions.amounts.components[0].mu" → changes component
- `test_apply_preserves_other_fields` — mutation only touches target path
- `test_apply_invalid_path_returns_error` — nonexistent path
- `test_constraint_validation_passes` — valid config with constraints
- `test_roundtrip_config_mutation` — serialize→mutate→deserialize preserves types

**Step 5:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- config_mutator`

**Step 6:** Commit: `feat(runtime): add ConfigMutator with dot-path config mutation (spec §4.4)`

---

### Task 8: ScenarioEngine Orchestrator

**Files:**
- Create: `crates/datasynth-runtime/src/scenario_engine.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `scenario_engine.rs` with types from spec §4.2:
- `ScenarioEngine` (base_config, causal_dag)
- `ScenarioResult` (scenario_name, baseline_path, counterfactual_path, diff, interventions_applied)
- `ScenarioError` enum (wrapping InterventionError, PropagationError, MutationError, DiffError, GenerationError)

**Step 2:** Implement `ScenarioEngine`:
- `new(config: GeneratorConfig) -> Result<Self, ScenarioError>` — loads CausalDAG from config's causal_model preset (default → parse built-in YAML, custom → user-provided nodes/edges)
- `generate_all(&self, output_root: &Path) -> Result<Vec<ScenarioResult>, ScenarioError>` — generates baseline once, then iterates scenarios
- `generate_scenario(&self, scenario: &ScenarioSchemaConfig, baseline_seed: u64, baseline_path: &Path, output_root: &Path) -> Result<ScenarioResult, ScenarioError>`
  1. Convert `ScenarioSchemaConfig` → `Vec<Intervention>` (build `Intervention` structs from schema config)
  2. Call `InterventionManager::validate()`
  3. Call `CausalPropagationEngine::propagate()`
  4. Call `ConfigMutator::apply()` to get mutated config
  5. Run `EnhancedOrchestrator::generate()` with mutated config + same seed
  6. (Optional) Compute diff if DiffEngine available
  7. Write scenario manifest YAML
- `load_causal_dag(config: &ScenariosConfig) -> Result<CausalDAG, ScenarioError>` — preset routing + custom merge

**Step 3:** For the internal generation call, use `EnhancedOrchestrator::new()` with mutated config and the same seed. The orchestrator writes to `output_root/scenarios/<name>/data/`.

**Step 4:** Register in `lib.rs`: `pub mod scenario_engine;`

**Step 5:** Add tests:
- `test_scenario_engine_new_default_dag` — loads default DAG
- `test_scenario_engine_converts_schema_to_interventions` — schema config → Intervention structs
- `test_scenario_engine_generates_baseline` — single scenario with no interventions produces baseline
- Integration test (in `tests/`): `test_scenario_engine_parameter_shift` — a parameter shift scenario produces different output than baseline

**Step 6:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- scenario_engine`

**Step 7:** Commit: `feat(runtime): add ScenarioEngine orchestrator for paired generation (spec §4.2)`

---

## Phase 4: Diff Engine (datasynth-eval)

### Task 9: Scenario Diff Types

**Files:**
- Create: `crates/datasynth-eval/src/scenario_diff.rs`
- Modify: `crates/datasynth-eval/src/lib.rs`

**Step 1:** Create `scenario_diff.rs` with types from spec §5.1:
- `ScenarioDiff` (summary, record_level, aggregate, intervention_trace — all Optional)
- `ImpactSummary` (scenario_name, generation_timestamp, interventions_applied, kpi_impacts, financial_statement_impacts, anomaly_impact, control_impact)
- `KpiImpact` (kpi_name, baseline_value, counterfactual_value, absolute_change, percent_change, direction)
- `ChangeDirection` enum (Increase, Decrease, Unchanged)
- `FinancialStatementImpact` (revenue/cogs/margin/net_income/assets/liabilities/cash_flow change_pct, top_changed_line_items)
- `LineItemImpact` (line_item, baseline, counterfactual, change_pct)
- `AnomalyImpact` (baseline/counterfactual counts, new/removed types, rate change)
- `ControlImpact` (controls_affected, new_deficiencies, material_weakness_risk)
- `ControlDeficiency` (control_id, name, baseline/counterfactual effectiveness, classification)
- `DeficiencyClassification` enum
- `RecordLevelDiff` (file_name, records added/removed/modified/unchanged, sample_changes)
- `RecordChange`, `RecordChangeType`, `FieldChange`
- `AggregateComparison` (metrics, period_comparisons)
- `MetricComparison`, `PeriodComparison`
- `InterventionTrace` (traces)
- `InterventionEffect`, `CausalPathStep`

All types derive `Debug, Clone, Serialize, Deserialize`.

**Step 2:** Register in `lib.rs`: `pub mod scenario_diff;`

**Step 3:** Add serde roundtrip tests for the core types.

**Step 4:** `cargo check -p datasynth-eval && cargo test -p datasynth-eval -- scenario_diff`

**Step 5:** Commit: `feat(eval): add scenario diff types for impact analysis (spec §5.1)`

---

### Task 10: DiffEngine Implementation

**Files:**
- Create: `crates/datasynth-eval/src/diff_engine.rs`
- Modify: `crates/datasynth-eval/src/lib.rs`

**Step 1:** Create `diff_engine.rs` with `DiffEngine` from spec §5.2:
- `DiffError` enum (IoError, ParseError, MismatchedSchemas)

**Step 2:** Implement `DiffEngine`:
- `compute(baseline_path: &Path, counterfactual_path: &Path, output_config: &ScenarioOutputConfig) -> Result<ScenarioDiff, DiffError>`
  - Dispatch to compute_summary, compute_record_level, compute_aggregate based on diff_formats
- `compute_summary(baseline: &Path, counterfactual: &Path) -> Result<ImpactSummary, DiffError>`
  - Load `journal_entries.csv` from both paths
  - Count total records, compute total debits/credits
  - Load `anomaly_labels.csv` if present — count anomalies
  - Build KpiImpact for key metrics (total_transactions, total_amount, anomaly_count)
  - Build minimal FinancialStatementImpact from trial balance if available
- `compute_record_level(baseline: &Path, counterfactual: &Path, scope: &[String]) -> Result<Vec<RecordLevelDiff>, DiffError>`
  - For each CSV file in scope (or all if empty), load both versions
  - Match records by first column (ID), classify as added/removed/modified/unchanged
  - For modified records, compute field-level changes
  - Cap sample_changes at 1000
- `compute_aggregate(baseline: &Path, counterfactual: &Path) -> Result<AggregateComparison, DiffError>`
  - Compute aggregate metrics (record counts, total amounts) per file
  - Compare period-by-period if period column exists

**Step 3:** Register in `lib.rs`: `pub mod diff_engine;`

**Step 4:** Add tests:
- `test_diff_engine_identical_dirs` — same data → no changes
- `test_diff_engine_record_added` — counterfactual has extra records
- `test_diff_engine_field_changed` — modified amounts detected
- `test_diff_engine_summary_computes_kpis` — KPI impacts computed

**Step 5:** `cargo check -p datasynth-eval && cargo test -p datasynth-eval -- diff_engine`

**Step 6:** Commit: `feat(eval): add DiffEngine for baseline vs counterfactual comparison (spec §5.2)`

---

## Phase 5: CLI Integration (datasynth-cli)

### Task 11: Scenario CLI Subcommand

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

**Step 1:** Add `Scenario(ScenarioArgs)` variant to `Commands` enum.

**Step 2:** Add `ScenarioArgs` and `ScenarioCommand` from spec §7.1:
```rust
#[derive(Args)]
struct ScenarioArgs {
    #[command(subcommand)]
    command: ScenarioCommand,
}

#[derive(Subcommand)]
enum ScenarioCommand {
    List { config: PathBuf },
    Validate { config: PathBuf, scenario: Option<String> },
    Generate { config: PathBuf, output: PathBuf, scenario: Option<String>, reuse_baseline: bool },
    Diff { baseline: PathBuf, counterfactual: PathBuf, format: String, output: Option<PathBuf> },
}
```

**Step 3:** Implement handlers:
- `scenario list` — load config, print scenarios with tags, intervention counts, timing
- `scenario validate` — load config, run InterventionManager::validate + DAG propagation, print results
- `scenario generate` — instantiate ScenarioEngine, call generate_all or generate single, print summary
- `scenario diff` — instantiate DiffEngine, compute diff, print or write to file

**Step 4:** Add import/export for the necessary types from runtime and eval crates (add dependencies if needed).

**Step 5:** `cargo check -p datasynth-cli && cargo build -p datasynth-cli`

**Step 6:** Manual test: `cargo run -p datasynth-cli -- scenario list --config test_config.yaml`

**Step 7:** Commit: `feat(cli): add scenario subcommand with list, validate, generate, diff (spec §7.1)`

---

## Phase 6: Integration Tests

### Task 12: Core Model Integration Tests

**Files:**
- Create: `crates/datasynth-core/tests/scenario_models_integration.rs`

**Step 1:** Test CausalDAG with default template:
- Load `causal_dag_default.yaml`, validate, verify 16+ nodes and 14+ edges
- Propagate a GDP shock and verify downstream effects chain

**Step 2:** Test scenario serde from YAML:
- Parse the example config from spec §3.2 as ScenariosConfig
- Verify 3 scenarios with correct intervention counts

**Step 3:** `cargo test -p datasynth-core --test scenario_models_integration`

**Step 4:** Commit: `test(core): add CausalDAG and scenario model integration tests`

---

### Task 13: Runtime Integration Tests

**Files:**
- Create: `crates/datasynth-runtime/tests/scenario_engine_integration.rs`

**Step 1:** Test full pipeline with a minimal config:
- Create a small config (1 company, 3 months, small CoA)
- Add a single ParameterShift scenario (shift transaction volume)
- Run ScenarioEngine::generate_all
- Verify baseline and counterfactual directories exist
- Verify output files differ after intervention month

**Step 2:** Test ConfigMutator roundtrip:
- Create config, serialize to JSON, mutate a dot-path, deserialize back
- Verify only the targeted field changed

**Step 3:** Test intervention validation:
- Test timing out of range is rejected
- Test conflicting interventions detected

**Step 4:** `cargo test -p datasynth-runtime --test scenario_engine_integration`

**Step 5:** Commit: `test(runtime): add ScenarioEngine integration tests for paired generation`

---

### Task 14: Diff Engine Integration Tests

**Files:**
- Create: `crates/datasynth-eval/tests/diff_engine_integration.rs`

**Step 1:** Create temporary directories with sample CSV data (baseline and modified).

**Step 2:** Test:
- Summary diff computes correct KPI changes
- Record-level diff identifies added/modified/removed records
- Aggregate diff computes period-level metrics

**Step 3:** `cargo test -p datasynth-eval --test diff_engine_integration`

**Step 4:** Commit: `test(eval): add DiffEngine integration tests`

---

## Phase 7: Final Verification

### Task 15: Full Workspace Verification

**Step 1:** `cargo fmt --all`
**Step 2:** `cargo clippy --workspace` — fix any warnings
**Step 3:** `cargo test --workspace` — all tests pass
**Step 4:** `cargo build --release`
**Step 5:** Create a test config with a scenario section and run:
```bash
./target/release/datasynth-data scenario validate --config test_scenario_config.yaml
./target/release/datasynth-data scenario list --config test_scenario_config.yaml
```

**Step 6:** Commit any final fixes.

---

## Task Dependency Graph

```
Phase 1 (Core Models):
  Task 1 (Scenario/Intervention) ─┐
  Task 2 (CausalDAG) ─────────────┼─→ Task 3 (Default DAG YAML)
                                   │
Phase 2 (Config):                  │
  Task 4 (ScenariosConfig) ────────┤ (depends on Task 1, 2)
                                   │
Phase 3 (Runtime):                 │
  Task 5 (CausalPropEngine) ───────┤ (depends on Task 2, 3)
  Task 6 (InterventionMgr) ────────┤ (depends on Task 1, 4)
  Task 7 (ConfigMutator) ──────────┤ (depends on Task 4, 5)
  Task 8 (ScenarioEngine) ─────────┤ (depends on Task 5, 6, 7)

Phase 4 (Eval):
  Task 9 (Diff Types) ─────────────┤ (depends on Task 1)
  Task 10 (DiffEngine) ────────────┤ (depends on Task 9)

Phase 5 (CLI):
  Task 11 (CLI) ───────────────────┤ (depends on Task 8, 10)

Phase 6 (Tests):
  Tasks 12-14 ─────────────────────┤ (depends on respective phases)

Phase 7 (Final):
  Task 15 ──────────────────────── depends on all above
```

**Recommended batching:**
- Batch 1: Tasks 1-3 (Core models, all independent once started sequentially)
- Batch 2: Tasks 4-5 (Config + CausalPropEngine, can run in parallel)
- Batch 3: Tasks 6-8 (InterventionMgr → ConfigMutator → ScenarioEngine, sequential)
- Batch 4: Tasks 9-10 (Diff types + engine, sequential)
- Batch 5: Task 11 (CLI wiring)
- Batch 6: Tasks 12-14 (Integration tests, parallel)
- Batch 7: Task 15 (Final verification)
