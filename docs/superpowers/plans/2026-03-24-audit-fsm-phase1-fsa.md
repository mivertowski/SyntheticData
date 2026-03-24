# Audit FSM Phase 1: FSA Blueprint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the `datasynth-audit-fsm` crate that loads YAML audit methodology blueprints as finite state machines and generates realistic audit artifacts + event trails, starting with the FSA (Financial Statement Audit) blueprint.

**Architecture:** Two-layer design: a methodology blueprint YAML defines "what happens" (states, transitions, procedures, steps), while a generation overlay YAML controls "how" (probabilities, timing, artifact volumes, anomaly injection). The FSM engine walks the procedure DAG, executes steps, and emits audit events. Existing audit generators in `datasynth-generators` are called as utility functions by the StepExecutor.

**Tech Stack:** Rust 1.88+, serde/serde_yaml for schema, ChaCha8Rng for deterministic generation, rand_distr for log-normal timing, chrono for timestamps, existing datasynth-core/generators/standards crates for models and artifact generation.

**Spec:** `docs/superpowers/specs/2026-03-24-audit-fsm-integration-design.md`

**Review fixes applied:** Deterministic UUIDs via RNG (not `Uuid::new_v4()`), `EngagementContext` parameter on `run_engagement()`, minimal `StepExecutor` producing `Generic` artifacts, proper log-normal timing distribution via `rand_distr`, consistent `OverlaySource` enum across tasks, `AnomalySeverity` on anomaly records, crate version aligned to workspace (1.4.0).

---

### Task 1: Create Crate Skeleton and Add to Workspace

**Files:**
- Create: `crates/datasynth-audit-fsm/Cargo.toml`
- Create: `crates/datasynth-audit-fsm/src/lib.rs`
- Modify: `Cargo.toml` (workspace members list)

- [ ] **Step 1: Create the crate directory**

```bash
mkdir -p crates/datasynth-audit-fsm/src
```

- [ ] **Step 2: Write Cargo.toml**

Create `crates/datasynth-audit-fsm/Cargo.toml`:

```toml
[package]
name = "datasynth-audit-fsm"
version = "1.4.0"
edition = "2021"
rust-version = "1.88"
description = "YAML-driven audit FSM engine for methodology-based audit artifact generation"

[dependencies]
datasynth-core = { workspace = true }
datasynth-standards = { workspace = true }
datasynth-generators = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
rust_decimal = { workspace = true }
rand = { workspace = true }
rand_chacha = { workspace = true }
rand_distr = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
```

- [ ] **Step 3: Write initial lib.rs**

Create `crates/datasynth-audit-fsm/src/lib.rs`:

```rust
//! YAML-driven audit FSM engine.
//!
//! Loads audit methodology blueprints (ISA, IIA-GIAS) as finite state machines
//! and generates realistic audit artifacts with event trail output.

pub mod context;
pub mod schema;
pub mod error;
```

- [ ] **Step 4: Create stub modules**

Create `crates/datasynth-audit-fsm/src/schema.rs`:

```rust
//! Blueprint and overlay schema types.
```

Create `crates/datasynth-audit-fsm/src/error.rs`:

```rust
//! Error types for FSM operations.
```

Create `crates/datasynth-audit-fsm/src/context.rs`:

```rust
//! Engagement context for FSM engine.
```

- [ ] **Step 5: Add to workspace members**

In root `Cargo.toml`, add `"crates/datasynth-audit-fsm"` to the `[workspace] members` list (alphabetically near the other datasynth crates).

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p datasynth-audit-fsm`
Expected: compiles with no errors (warnings OK)

- [ ] **Step 7: Commit**

```bash
git add crates/datasynth-audit-fsm/ Cargo.toml Cargo.lock
git commit -m "feat(audit-fsm): create datasynth-audit-fsm crate skeleton"
```

---

### Task 2: Blueprint Schema Types

**Files:**
- Create: `crates/datasynth-audit-fsm/src/schema.rs`

This defines all Rust types for deserializing the YAML blueprint. The types must match the structure in `docs/blueprints/generic_fsa.yaml` from the AuditMethodology repo.

- [ ] **Step 1: Write schema deserialization test**

Add to the bottom of `crates/datasynth-audit-fsm/src/schema.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_level_deserialize() {
        let yaml = "standard";
        let level: DepthLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(level, DepthLevel::Standard);
    }

    #[test]
    fn test_binding_level_deserialize() {
        let yaml = "requirement";
        let level: BindingLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(level, BindingLevel::Requirement);
    }

    #[test]
    fn test_procedure_aggregate_roundtrip() {
        let yaml = r#"
initial_state: not_started
states: [not_started, in_progress, under_review, completed]
transitions:
  - from_state: not_started
    to_state: in_progress
    command: start_materiality
    emits: MaterialityStarted
  - from_state: in_progress
    to_state: under_review
    command: submit_materiality
    emits: MaterialitySubmitted
    guards: [all_steps_complete]
  - from_state: under_review
    to_state: in_progress
    command: revise_materiality
    emits: MaterialityRevisionRequested
  - from_state: under_review
    to_state: completed
    command: approve_materiality
    emits: MaterialityApproved
    guards: [reviewer_approved]
"#;
        let agg: ProcedureAggregate = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(agg.initial_state, "not_started");
        assert_eq!(agg.states.len(), 4);
        assert_eq!(agg.transitions.len(), 4);
        assert_eq!(agg.transitions[2].emits, "MaterialityRevisionRequested");
    }

    #[test]
    fn test_step_with_evidence_deserialize() {
        let yaml = r#"
id: mat_step_1
order: 1
action: determine
actor: audit_manager
description: "Determine overall materiality"
binding: requirement
command: determine_overall_materiality
emits: OverallMaterialityDetermined
evidence:
  inputs: []
  outputs:
    - ref: materiality_workpaper
      type: workpaper
standards:
  - ref: "ISA 320"
    paragraphs: ["10"]
    binding: requirement
"#;
        let step: BlueprintStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.id, "mat_step_1");
        assert_eq!(step.action, "determine");
        assert_eq!(step.evidence.outputs.len(), 1);
        assert_eq!(step.evidence.outputs[0].ref_id, "materiality_workpaper");
        assert_eq!(step.standards.len(), 1);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL — types not defined yet

- [ ] **Step 3: Implement the schema types**

Write the full schema in `crates/datasynth-audit-fsm/src/schema.rs`:

```rust
//! Blueprint and overlay schema types.
//!
//! These types mirror the YAML schema from the AuditMethodology repository.
//! All types derive Serialize + Deserialize for YAML roundtrip.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Blueprint types ──────────────────────────────────────────

/// Root type for an audit methodology blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBlueprint {
    pub schema_version: String,
    #[serde(default)]
    pub depth: DepthLevel,
    pub methodology: BlueprintMethodology,
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub actors: Vec<BlueprintActor>,
    #[serde(default)]
    pub standards_catalog: Vec<BlueprintStandard>,
    #[serde(default)]
    pub evidence_catalog: Vec<BlueprintEvidence>,
    #[serde(default)]
    pub phases: Vec<BlueprintPhase>,
    #[serde(default)]
    pub procedures: Vec<BlueprintProcedure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintMethodology {
    pub name: String,
    pub version: String,
    pub framework: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintActor {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStandard {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub title: String,
    pub binding: BindingLevel,
    #[serde(default)]
    pub paragraphs: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintEvidence {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub evidence_type: String,
    #[serde(default)]
    pub lifecycle: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintPhase {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub description: String,
    #[serde(default)]
    pub gate: Option<PhaseGate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseGate {
    pub all_of: Vec<GateCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCondition {
    pub procedure: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintProcedure {
    pub id: String,
    pub phase: String,
    pub title: String,
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<String>>,
    pub aggregate: ProcedureAggregate,
    #[serde(default)]
    pub steps: Vec<BlueprintStep>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub knowledge_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureAggregate {
    pub initial_state: String,
    pub states: Vec<String>,
    pub transitions: Vec<ProcedureTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTransition {
    pub from_state: String,
    pub to_state: String,
    pub command: String,
    pub emits: String,
    #[serde(default)]
    pub guards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStep {
    pub id: String,
    pub order: i32,
    pub action: String,
    pub actor: String,
    pub description: String,
    #[serde(default = "default_binding")]
    pub binding: BindingLevel,
    pub command: String,
    pub emits: String,
    #[serde(default)]
    pub guards: Vec<StepGuard>,
    #[serde(default)]
    pub evidence: StepEvidence,
    #[serde(default)]
    pub standards: Vec<StepStandard>,
    #[serde(default)]
    pub decisions: Vec<StepDecision>,
}

fn default_binding() -> BindingLevel {
    BindingLevel::Requirement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGuard {
    #[serde(rename = "type")]
    pub guard_type: String,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepEvidence {
    #[serde(default)]
    pub inputs: Vec<EvidenceRef>,
    #[serde(default)]
    pub outputs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(rename = "type")]
    pub evidence_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStandard {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default)]
    pub paragraphs: Vec<String>,
    pub binding: BindingLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDecision {
    pub condition: String,
    pub branches: Vec<DecisionBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionBranch {
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub next_step: Option<String>,
}

// ── Enums ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepthLevel {
    Simplified,
    #[default]
    Standard,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingLevel {
    #[default]
    Requirement,
    Guidance,
}

// ── Overlay types ────────────────────────────────────────────

/// Generation overlay controlling probabilities, timing, volumes, anomalies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOverlay {
    #[serde(default)]
    pub depth: Option<DepthLevel>,
    #[serde(default)]
    pub discriminators: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub transitions: TransitionConfig,
    #[serde(default)]
    pub artifacts: ArtifactConfig,
    #[serde(default)]
    pub anomalies: AnomalyConfig,
    #[serde(default)]
    pub actor_profiles: HashMap<String, ActorProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    #[serde(default = "TransitionDefaults::default")]
    pub defaults: TransitionDefaults,
    #[serde(default)]
    pub overrides: HashMap<String, TransitionOverride>,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            defaults: TransitionDefaults::default(),
            overrides: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefaults {
    #[serde(default = "default_revision_probability")]
    pub revision_probability: f64,
    #[serde(default)]
    pub timing: TimingDistribution,
}

fn default_revision_probability() -> f64 {
    0.15
}

impl Default for TransitionDefaults {
    fn default() -> Self {
        Self {
            revision_probability: 0.15,
            timing: TimingDistribution::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransitionOverride {
    pub revision_probability: Option<f64>,
    pub timing: Option<TimingDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingDistribution {
    #[serde(default = "default_timing_type")]
    pub distribution: String,
    #[serde(default = "default_mu_hours")]
    pub mu_hours: f64,
    #[serde(default = "default_sigma_hours")]
    pub sigma_hours: f64,
}

fn default_timing_type() -> String {
    "log_normal".to_string()
}
fn default_mu_hours() -> f64 {
    24.0
}
fn default_sigma_hours() -> f64 {
    8.0
}

impl Default for TimingDistribution {
    fn default() -> Self {
        Self {
            distribution: "log_normal".to_string(),
            mu_hours: 24.0,
            sigma_hours: 8.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactConfig {
    #[serde(default = "default_workpapers_per_step")]
    pub workpapers_per_step: VolumeRange,
    #[serde(default = "default_evidence_per_workpaper")]
    pub evidence_items_per_workpaper: VolumeRange,
}

fn default_workpapers_per_step() -> VolumeRange {
    VolumeRange { min: 1, max: 3 }
}
fn default_evidence_per_workpaper() -> VolumeRange {
    VolumeRange { min: 2, max: 5 }
}

impl Default for ArtifactConfig {
    fn default() -> Self {
        Self {
            workpapers_per_step: VolumeRange { min: 1, max: 3 },
            evidence_items_per_workpaper: VolumeRange { min: 2, max: 5 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    #[serde(default)]
    pub skipped_approval: AnomalyRule,
    #[serde(default)]
    pub late_posting: AnomalyRule,
    #[serde(default)]
    pub missing_evidence: AnomalyRule,
    #[serde(default)]
    pub out_of_sequence: AnomalyRule,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            skipped_approval: AnomalyRule { probability: 0.02, ..Default::default() },
            late_posting: AnomalyRule { probability: 0.05, max_delay_hours: Some(72.0), ..Default::default() },
            missing_evidence: AnomalyRule { probability: 0.03, ..Default::default() },
            out_of_sequence: AnomalyRule { probability: 0.01, ..Default::default() },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalyRule {
    #[serde(default)]
    pub probability: f64,
    #[serde(default)]
    pub applicable_guards: Option<Vec<String>>,
    #[serde(default)]
    pub max_delay_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorProfile {
    #[serde(default = "default_one")]
    pub revision_multiplier: f64,
    #[serde(default = "default_one")]
    pub evidence_multiplier: f64,
    #[serde(default)]
    pub skip_guidance_steps: bool,
}

fn default_one() -> f64 {
    1.0
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/schema.rs
git commit -m "feat(audit-fsm): add blueprint and overlay schema types"
```

---

### Task 3: Error Types

**Files:**
- Create: `crates/datasynth-audit-fsm/src/error.rs`

- [ ] **Step 1: Implement error types**

```rust
//! Error types for FSM operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditFsmError {
    #[error("Blueprint parse error ({path}): {source}")]
    BlueprintParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("Blueprint validation failed: {violations:?}")]
    BlueprintValidation {
        violations: Vec<ValidationViolation>,
    },

    #[error("Overlay parse error ({path}): {source}")]
    OverlayParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("Guard failure in procedure '{procedure_id}': guard '{guard}' — {reason}")]
    GuardFailure {
        procedure_id: String,
        guard: String,
        reason: String,
    },

    #[error("Precondition not met for '{procedure_id}': requires '{required}' but was '{actual_state}'")]
    PreconditionNotMet {
        procedure_id: String,
        required: String,
        actual_state: String,
    },

    #[error("Source not found: {source}")]
    SourceNotFound { source: String },

    #[error("DAG cycle detected involving procedures: {procedures:?}")]
    DagCycle { procedures: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct ValidationViolation {
    pub location: String,
    pub message: String,
}

impl std::fmt::Display for ValidationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p datasynth-audit-fsm`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add crates/datasynth-audit-fsm/src/error.rs
git commit -m "feat(audit-fsm): add error types"
```

---

### Task 4: Blueprint Loader with Validation

**Files:**
- Create: `crates/datasynth-audit-fsm/src/loader.rs`
- Copy: `docs/blueprints/generic_fsa.yaml` from AuditMethodology repo → `crates/datasynth-audit-fsm/blueprints/generic_fsa.yaml`
- Modify: `crates/datasynth-audit-fsm/src/lib.rs` (add `pub mod loader;`)

The loader parses YAML, validates cross-references, and detects DAG cycles.

- [ ] **Step 1: Copy FSA blueprint to the crate**

```bash
mkdir -p crates/datasynth-audit-fsm/blueprints
cp /home/michael/DEV/Repos/Methodology/AuditMethodology/docs/blueprints/generic_fsa.yaml \
   crates/datasynth-audit-fsm/blueprints/generic_fsa.yaml
```

- [ ] **Step 2: Write loader tests**

Create `crates/datasynth-audit-fsm/src/loader.rs` with tests at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const FSA_YAML: &str = include_str!("../blueprints/generic_fsa.yaml");

    #[test]
    fn test_load_fsa_blueprint_parses() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        assert_eq!(bp.schema_version, "1.0");
        assert_eq!(bp.methodology.framework, "ISA");
    }

    #[test]
    fn test_fsa_has_expected_structure() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        assert!(bp.phases.len() >= 3, "Expected >= 3 phases");
        assert!(bp.procedures.len() >= 7, "Expected >= 7 procedures");
        assert!(bp.actors.len() >= 4, "Expected >= 4 actors");
        assert!(bp.evidence_catalog.len() >= 10, "Expected >= 10 evidence types");
        assert!(bp.standards_catalog.len() >= 13, "Expected >= 13 standards");
    }

    #[test]
    fn test_fsa_validates_successfully() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let result = validate_blueprint(&bp);
        assert!(result.is_ok(), "Validation errors: {:?}", result.err());
    }

    #[test]
    fn test_rejects_cycle_in_preconditions() {
        let mut bp = parse_blueprint(FSA_YAML).unwrap();
        // Create a cycle: accept_engagement depends on form_opinion,
        // which already depends on going_concern → substantive_testing → risk_identification → accept_engagement
        bp.procedures[0].preconditions.push("form_opinion".to_string());
        let result = validate_blueprint(&bp);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_invalid_phase_ref() {
        let mut bp = parse_blueprint(FSA_YAML).unwrap();
        bp.procedures[0].phase = "nonexistent_phase".to_string();
        let result = validate_blueprint(&bp);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_invalid_precondition_ref() {
        let mut bp = parse_blueprint(FSA_YAML).unwrap();
        bp.procedures[1].preconditions.push("nonexistent_procedure".to_string());
        let result = validate_blueprint(&bp);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_invalid_actor_ref() {
        let mut bp = parse_blueprint(FSA_YAML).unwrap();
        bp.procedures[0].steps[0].actor = "nonexistent_actor".to_string();
        let result = validate_blueprint(&bp);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort_fsa() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let order = topological_sort_procedures(&bp).unwrap();
        // accept_engagement must come before planning_materiality and risk_identification
        let accept_idx = order.iter().position(|id| id == "accept_engagement").unwrap();
        let mat_idx = order.iter().position(|id| id == "planning_materiality").unwrap();
        let risk_idx = order.iter().position(|id| id == "risk_identification").unwrap();
        assert!(accept_idx < mat_idx);
        assert!(accept_idx < risk_idx);
    }

    #[test]
    fn test_load_default_overlay() {
        let overlay = default_overlay();
        assert!((overlay.transitions.defaults.revision_probability - 0.15).abs() < 0.001);
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL — `parse_blueprint`, `validate_blueprint`, etc. not defined

- [ ] **Step 4: Implement the loader**

Write the implementation above the tests in `crates/datasynth-audit-fsm/src/loader.rs`:

```rust
//! Blueprint and overlay loading, parsing, and validation.

use crate::error::{AuditFsmError, ValidationViolation};
use crate::schema::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

// ── Builtins ─────────────────────────────────────────────────

const BUILTIN_FSA: &str = include_str!("../blueprints/generic_fsa.yaml");

/// Blueprint source specification.
pub enum BlueprintSource {
    Builtin(BuiltinBlueprint),
    Custom(std::path::PathBuf),
    Raw(String),
}

pub enum BuiltinBlueprint {
    Fsa,
}

/// Overlay source specification.
pub enum OverlaySource {
    Builtin(BuiltinOverlay),
    Custom(std::path::PathBuf),
    Raw(String),
}

pub enum BuiltinOverlay {
    Default,
    Thorough,
    Rushed,
}

// ── Parsing ──────────────────────────────────────────────────

/// Parse a blueprint from a YAML string.
pub fn parse_blueprint(yaml: &str) -> Result<AuditBlueprint, AuditFsmError> {
    serde_yaml::from_str(yaml).map_err(|e| AuditFsmError::BlueprintParse {
        path: "<string>".into(),
        source: e,
    })
}

/// Parse an overlay from a YAML string.
pub fn parse_overlay(yaml: &str) -> Result<GenerationOverlay, AuditFsmError> {
    serde_yaml::from_str(yaml).map_err(|e| AuditFsmError::OverlayParse {
        path: "<string>".into(),
        source: e,
    })
}

/// Load a blueprint from a source.
pub fn load_blueprint(source: &BlueprintSource) -> Result<AuditBlueprint, AuditFsmError> {
    match source {
        BlueprintSource::Builtin(b) => {
            let yaml = match b {
                BuiltinBlueprint::Fsa => BUILTIN_FSA,
            };
            parse_blueprint(yaml)
        }
        BlueprintSource::Custom(path) => {
            let yaml = std::fs::read_to_string(path).map_err(|_| AuditFsmError::SourceNotFound {
                source: path.display().to_string(),
            })?;
            let mut bp = parse_blueprint(&yaml)?;
            // Update path info for error messages
            bp.schema_version.clone(); // no-op, just uses bp
            Ok(bp)
        }
        BlueprintSource::Raw(yaml) => parse_blueprint(yaml),
    }
}

/// Return the default generation overlay with sensible defaults.
pub fn default_overlay() -> GenerationOverlay {
    GenerationOverlay {
        depth: None,
        discriminators: None,
        transitions: TransitionConfig::default(),
        artifacts: ArtifactConfig::default(),
        anomalies: AnomalyConfig::default(),
        actor_profiles: HashMap::new(),
    }
}

/// Load an overlay from a source, or return the default.
/// In Phase 1, only BuiltinOverlay::Default is implemented.
/// Thorough and Rushed are added in Task 10.
pub fn load_overlay(source: &OverlaySource) -> Result<GenerationOverlay, AuditFsmError> {
    match source {
        OverlaySource::Builtin(b) => match b {
            BuiltinOverlay::Default => Ok(default_overlay()),
            _ => Ok(default_overlay()), // Thorough/Rushed implemented in Task 10
        },
        OverlaySource::Custom(path) => {
            let yaml = std::fs::read_to_string(path).map_err(|_| AuditFsmError::SourceNotFound {
                source: path.display().to_string(),
            })?;
            parse_overlay(&yaml)
        }
        OverlaySource::Raw(yaml) => parse_overlay(yaml),
    }
}

// ── Validation ───────────────────────────────────────────────

/// Validate a blueprint for internal consistency.
pub fn validate_blueprint(bp: &AuditBlueprint) -> Result<(), AuditFsmError> {
    let mut violations = Vec::new();

    let phase_ids: HashSet<&str> = bp.phases.iter().map(|p| p.id.as_str()).collect();
    let procedure_ids: HashSet<&str> = bp.procedures.iter().map(|p| p.id.as_str()).collect();
    let actor_ids: HashSet<&str> = bp.actors.iter().map(|a| a.id.as_str()).collect();
    let evidence_ids: HashSet<&str> = bp.evidence_catalog.iter().map(|e| e.id.as_str()).collect();
    let standard_refs: HashSet<&str> = bp.standards_catalog.iter().map(|s| s.ref_id.as_str()).collect();

    // Validate procedures
    for (i, proc) in bp.procedures.iter().enumerate() {
        let loc = format!("procedures[{i}] ({id})", id = proc.id);

        // Phase reference
        if !phase_ids.contains(proc.phase.as_str()) {
            violations.push(ValidationViolation {
                location: loc.clone(),
                message: format!("references unknown phase '{}'", proc.phase),
            });
        }

        // Preconditions
        for pre in &proc.preconditions {
            if !procedure_ids.contains(pre.as_str()) {
                violations.push(ValidationViolation {
                    location: loc.clone(),
                    message: format!("precondition references unknown procedure '{pre}'"),
                });
            }
        }

        // Steps
        for (j, step) in proc.steps.iter().enumerate() {
            let step_loc = format!("{loc}.steps[{j}] ({id})", id = step.id);

            // Actor reference
            if !actor_ids.contains(step.actor.as_str()) {
                violations.push(ValidationViolation {
                    location: step_loc.clone(),
                    message: format!("references unknown actor '{}'", step.actor),
                });
            }

            // Evidence input references
            for input in &step.evidence.inputs {
                if !evidence_ids.contains(input.ref_id.as_str()) {
                    violations.push(ValidationViolation {
                        location: step_loc.clone(),
                        message: format!("evidence input references unknown '{}'", input.ref_id),
                    });
                }
            }

            // Evidence output references
            for output in &step.evidence.outputs {
                if !evidence_ids.contains(output.ref_id.as_str()) {
                    violations.push(ValidationViolation {
                        location: step_loc.clone(),
                        message: format!("evidence output references unknown '{}'", output.ref_id),
                    });
                }
            }

            // Standards references
            for std_ref in &step.standards {
                if !standard_refs.contains(std_ref.ref_id.as_str()) {
                    violations.push(ValidationViolation {
                        location: step_loc.clone(),
                        message: format!("references unknown standard '{}'", std_ref.ref_id),
                    });
                }
            }
        }

        // Validate aggregate: all transition states must be in states list
        let state_set: HashSet<&str> = proc.aggregate.states.iter().map(|s| s.as_str()).collect();
        for t in &proc.aggregate.transitions {
            if !state_set.contains(t.from_state.as_str()) {
                violations.push(ValidationViolation {
                    location: format!("{loc}.aggregate"),
                    message: format!("transition from unknown state '{}'", t.from_state),
                });
            }
            if !state_set.contains(t.to_state.as_str()) {
                violations.push(ValidationViolation {
                    location: format!("{loc}.aggregate"),
                    message: format!("transition to unknown state '{}'", t.to_state),
                });
            }
        }
    }

    // Validate phase gates
    for (i, phase) in bp.phases.iter().enumerate() {
        if let Some(gate) = &phase.gate {
            let loc = format!("phases[{i}] ({id})", id = phase.id);
            for cond in &gate.all_of {
                if !procedure_ids.contains(cond.procedure.as_str()) {
                    violations.push(ValidationViolation {
                        location: loc.clone(),
                        message: format!("gate references unknown procedure '{}'", cond.procedure),
                    });
                }
            }
        }
    }

    // Check for cycles in precondition DAG
    if let Err(cycle_err) = topological_sort_procedures(bp) {
        return Err(cycle_err);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(AuditFsmError::BlueprintValidation { violations })
    }
}

// ── DAG Utilities ────────────────────────────────────────────

/// Topological sort of procedures by precondition dependencies.
/// Returns procedure IDs in execution order. Errors on cycles.
pub fn topological_sort_procedures(bp: &AuditBlueprint) -> Result<Vec<String>, AuditFsmError> {
    let procedure_ids: HashSet<&str> = bp.procedures.iter().map(|p| p.id.as_str()).collect();

    // Build adjacency and in-degree maps
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for proc in &bp.procedures {
        in_degree.entry(proc.id.as_str()).or_insert(0);
        adj.entry(proc.id.as_str()).or_default();
        for pre in &proc.preconditions {
            if procedure_ids.contains(pre.as_str()) {
                adj.entry(pre.as_str()).or_default().push(proc.id.as_str());
                *in_degree.entry(proc.id.as_str()).or_insert(0) += 1;
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.to_string());
        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if result.len() != bp.procedures.len() {
        let sorted_set: HashSet<&str> = result.iter().map(|s| s.as_str()).collect();
        let cycle_procs: Vec<String> = bp
            .procedures
            .iter()
            .filter(|p| !sorted_set.contains(p.id.as_str()))
            .map(|p| p.id.clone())
            .collect();
        return Err(AuditFsmError::DagCycle {
            procedures: cycle_procs,
        });
    }

    Ok(result)
}
```

- [ ] **Step 5: Update lib.rs**

Add `pub mod loader;` to `crates/datasynth-audit-fsm/src/lib.rs`.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS (8 schema tests + 8 loader tests)

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p datasynth-audit-fsm`
Expected: no warnings (except possibly the protoc warning from workspace)

- [ ] **Step 8: Commit**

```bash
git add crates/datasynth-audit-fsm/blueprints/ crates/datasynth-audit-fsm/src/loader.rs crates/datasynth-audit-fsm/src/lib.rs
git commit -m "feat(audit-fsm): add blueprint loader with validation and topological sort"
```

---

### Task 5: EngagementContext, Event Types, and Anomaly Types

**Files:**
- Create: `crates/datasynth-audit-fsm/src/context.rs`
- Create: `crates/datasynth-audit-fsm/src/event.rs`
- Modify: `crates/datasynth-audit-fsm/src/lib.rs`

First, implement `context.rs` — the context struct the engine receives:

```rust
//! Engagement context for FSM engine.

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Context from the broader generation run, passed to the FSM engine.
pub struct EngagementContext {
    pub company_code: String,
    pub company_name: String,
    pub fiscal_year: i32,
    pub currency: String,
    pub total_revenue: Decimal,
    pub total_assets: Decimal,
    pub engagement_start: NaiveDate,
    pub report_date: NaiveDate,
}

impl EngagementContext {
    /// Create a minimal test context.
    pub fn test_default() -> Self {
        Self {
            company_code: "TEST01".into(),
            company_name: "Test Corp".into(),
            fiscal_year: 2025,
            currency: "USD".into(),
            total_revenue: Decimal::new(10_000_000, 0),
            total_assets: Decimal::new(50_000_000, 0),
            engagement_start: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            report_date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        }
    }
}
```

Then implement event types (note: `build_with_rng` takes `&mut impl rand::Rng` for deterministic UUIDs):

- [ ] **Step 1: Write event test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder_transition() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let event = AuditEventBuilder::transition()
            .procedure_id("planning_materiality")
            .phase_id("planning")
            .from_state("not_started")
            .to_state("in_progress")
            .command("start_materiality")
            .event_type("MaterialityStarted")
            .actor_id("audit_manager")
            .timestamp(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap().and_hms_opt(9, 0, 0).unwrap())
            .build_with_rng(&mut rng);
        assert_eq!(event.event_type, "MaterialityStarted");
        assert_eq!(event.from_state, Some("not_started".to_string()));
        assert!(event.step_id.is_none());
        assert!(!event.is_anomaly);
        assert!(!event.event_id.is_nil(), "Event ID should be non-nil");
    }

    #[test]
    fn test_event_builder_step() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let event = AuditEventBuilder::step()
            .procedure_id("planning_materiality")
            .step_id("mat_step_1")
            .phase_id("planning")
            .command("determine_overall_materiality")
            .event_type("OverallMaterialityDetermined")
            .actor_id("audit_manager")
            .evidence_ref("materiality_workpaper")
            .standard_ref("ISA 320")
            .timestamp(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap().and_hms_opt(10, 0, 0).unwrap())
            .build_with_rng(&mut rng);
        assert_eq!(event.step_id, Some("mat_step_1".to_string()));
        assert_eq!(event.evidence_refs, vec!["materiality_workpaper"]);
        assert_eq!(event.standards_refs, vec!["ISA 320"]);
    }

    #[test]
    fn test_deterministic_event_ids() {
        use rand::SeedableRng;
        let mut rng1 = rand_chacha::ChaCha8Rng::seed_from_u64(99);
        let mut rng2 = rand_chacha::ChaCha8Rng::seed_from_u64(99);
        let e1 = AuditEventBuilder::transition()
            .event_type("Test")
            .build_with_rng(&mut rng1);
        let e2 = AuditEventBuilder::transition()
            .event_type("Test")
            .build_with_rng(&mut rng2);
        assert_eq!(e1.event_id, e2.event_id);
    }

    #[test]
    fn test_anomaly_type_display() {
        let t = AuditAnomalyType::SkippedApproval;
        assert_eq!(format!("{t}"), "skipped_approval");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL

- [ ] **Step 3: Implement event types**

```rust
//! Audit event types and builder.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A single audit trail event emitted by the FSM engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub phase_id: String,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub actor_id: String,
    pub command: String,
    pub evidence_refs: Vec<String>,
    pub standards_refs: Vec<String>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AuditAnomalyType>,
}

/// Audit-specific anomaly types (distinct from core AnomalyType).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAnomalyType {
    SkippedApproval,
    LatePosting,
    MissingEvidence,
    OutOfSequence,
    InsufficientDocumentation,
}

impl fmt::Display for AuditAnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkippedApproval => write!(f, "skipped_approval"),
            Self::LatePosting => write!(f, "late_posting"),
            Self::MissingEvidence => write!(f, "missing_evidence"),
            Self::OutOfSequence => write!(f, "out_of_sequence"),
            Self::InsufficientDocumentation => write!(f, "insufficient_documentation"),
        }
    }
}

/// Anomaly severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity { Low, Medium, High, Critical }

/// Audit anomaly record for tracking injected anomalies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnomalyRecord {
    pub anomaly_id: Uuid,
    pub anomaly_type: AuditAnomalyType,
    pub severity: AnomalySeverity,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub timestamp: NaiveDateTime,
    pub description: String,
}

/// Builder for constructing AuditEvent instances.
pub struct AuditEventBuilder {
    event_type_str: String,
    procedure_id: String,
    step_id: Option<String>,
    phase_id: String,
    from_state: Option<String>,
    to_state: Option<String>,
    actor_id: String,
    command: String,
    evidence_refs: Vec<String>,
    standards_refs: Vec<String>,
    timestamp: NaiveDateTime,
    is_anomaly: bool,
    anomaly_type: Option<AuditAnomalyType>,
}

impl AuditEventBuilder {
    /// Create a builder for a transition event.
    pub fn transition() -> Self {
        Self::new()
    }

    /// Create a builder for a step event.
    pub fn step() -> Self {
        Self::new()
    }

    fn new() -> Self {
        Self {
            event_type_str: String::new(),
            procedure_id: String::new(),
            step_id: None,
            phase_id: String::new(),
            from_state: None,
            to_state: None,
            actor_id: String::new(),
            command: String::new(),
            evidence_refs: Vec::new(),
            standards_refs: Vec::new(),
            timestamp: chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            is_anomaly: false,
            anomaly_type: None,
        }
    }

    pub fn event_type(mut self, t: &str) -> Self { self.event_type_str = t.to_string(); self }
    pub fn procedure_id(mut self, id: &str) -> Self { self.procedure_id = id.to_string(); self }
    pub fn step_id(mut self, id: &str) -> Self { self.step_id = Some(id.to_string()); self }
    pub fn phase_id(mut self, id: &str) -> Self { self.phase_id = id.to_string(); self }
    pub fn from_state(mut self, s: &str) -> Self { self.from_state = Some(s.to_string()); self }
    pub fn to_state(mut self, s: &str) -> Self { self.to_state = Some(s.to_string()); self }
    pub fn actor_id(mut self, id: &str) -> Self { self.actor_id = id.to_string(); self }
    pub fn command(mut self, cmd: &str) -> Self { self.command = cmd.to_string(); self }
    pub fn evidence_ref(mut self, r: &str) -> Self { self.evidence_refs.push(r.to_string()); self }
    pub fn standard_ref(mut self, r: &str) -> Self { self.standards_refs.push(r.to_string()); self }
    pub fn timestamp(mut self, ts: NaiveDateTime) -> Self { self.timestamp = ts; self }
    pub fn anomaly(mut self, t: AuditAnomalyType) -> Self {
        self.is_anomaly = true;
        self.anomaly_type = Some(t);
        self
    }

    /// Build the event. Uses the provided RNG bytes to produce a deterministic UUID.
    pub fn build_with_rng(self, rng: &mut impl rand::Rng) -> AuditEvent {
        let bytes: [u8; 16] = rng.random();
        let event_id = uuid::Builder::from_random_bytes(bytes).into_uuid();
        AuditEvent {
            event_id,
            timestamp: self.timestamp,
            event_type: self.event_type_str,
            procedure_id: self.procedure_id,
            step_id: self.step_id,
            phase_id: self.phase_id,
            from_state: self.from_state,
            to_state: self.to_state,
            actor_id: self.actor_id,
            command: self.command,
            evidence_refs: self.evidence_refs,
            standards_refs: self.standards_refs,
            is_anomaly: self.is_anomaly,
            anomaly_type: self.anomaly_type,
        }
    }
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Add `pub mod event;` to lib.rs.

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/event.rs crates/datasynth-audit-fsm/src/lib.rs
git commit -m "feat(audit-fsm): add audit event types and builder"
```

---

### Task 6: FSM Engine Core

**Files:**
- Create: `crates/datasynth-audit-fsm/src/engine.rs`
- Modify: `crates/datasynth-audit-fsm/src/lib.rs`

This is the core execution engine. It walks procedures in DAG order, advances aggregate FSMs, and emits events. Artifact generation (StepExecutor) is deferred to Task 7 — the engine in this task produces events only.

- [ ] **Step 1: Write engine tests**

Add to `crates/datasynth-audit-fsm/src/engine.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EngagementContext;
    use crate::loader::{parse_blueprint, default_overlay};

    const FSA_YAML: &str = include_str!("../blueprints/generic_fsa.yaml");

    fn test_context() -> EngagementContext {
        EngagementContext::test_default()
    }

    #[test]
    fn test_engine_loads() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let overlay = default_overlay();
        let engine = AuditFsmEngine::new(bp, overlay, 42);
        assert!(engine.blueprint.procedures.len() >= 7);
    }

    #[test]
    fn test_run_engagement_produces_events() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let overlay = default_overlay();
        let mut engine = AuditFsmEngine::new(bp, overlay, 42);
        let result = engine.run_engagement(&test_context());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.event_log.len() >= 14, "Expected >= 14 events, got {}", result.event_log.len());
    }

    #[test]
    fn test_deterministic_output() {
        let bp1 = parse_blueprint(FSA_YAML).unwrap();
        let bp2 = parse_blueprint(FSA_YAML).unwrap();

        let mut engine1 = AuditFsmEngine::new(bp1, default_overlay(), 42);
        let mut engine2 = AuditFsmEngine::new(bp2, default_overlay(), 42);

        let ctx = test_context();
        let r1 = engine1.run_engagement(&ctx).unwrap();
        let r2 = engine2.run_engagement(&ctx).unwrap();

        assert_eq!(r1.event_log.len(), r2.event_log.len());
        for (e1, e2) in r1.event_log.iter().zip(r2.event_log.iter()) {
            assert_eq!(e1.event_id, e2.event_id, "Deterministic UUIDs must match");
            assert_eq!(e1.event_type, e2.event_type);
            assert_eq!(e1.procedure_id, e2.procedure_id);
            assert_eq!(e1.command, e2.command);
            assert_eq!(e1.timestamp, e2.timestamp);
        }
    }

    #[test]
    fn test_all_procedures_reach_completed() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let mut engine = AuditFsmEngine::new(bp, default_overlay(), 42);
        let result = engine.run_engagement(&test_context()).unwrap();
        for (proc_id, state) in &result.procedure_states {
            assert_eq!(state, "completed", "Procedure {proc_id} ended in state {state}");
        }
    }

    #[test]
    fn test_precondition_ordering_respected() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let mut engine = AuditFsmEngine::new(bp, default_overlay(), 42);
        let result = engine.run_engagement(&test_context()).unwrap();

        let mut first_events: HashMap<String, usize> = HashMap::new();
        for (i, event) in result.event_log.iter().enumerate() {
            first_events.entry(event.procedure_id.clone()).or_insert(i);
        }

        assert!(first_events["accept_engagement"] < first_events["planning_materiality"]);
        assert!(first_events["going_concern"] < first_events["form_opinion"]);
        assert!(first_events["subsequent_events"] < first_events["form_opinion"]);
    }

    #[test]
    fn test_step_events_emitted() {
        let bp = parse_blueprint(FSA_YAML).unwrap();
        let mut engine = AuditFsmEngine::new(bp, default_overlay(), 42);
        let result = engine.run_engagement(&test_context()).unwrap();
        let step_events: Vec<_> = result.event_log.iter().filter(|e| e.step_id.is_some()).collect();
        assert!(step_events.len() >= 15, "Expected >= 15 step events, got {}", step_events.len());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL

- [ ] **Step 3: Implement the engine**

Write above the tests in `crates/datasynth-audit-fsm/src/engine.rs`:

```rust
//! FSM execution engine.
//!
//! Walks procedures in precondition DAG order, advances aggregate FSMs,
//! executes steps, and emits audit events.

use crate::context::EngagementContext;
use crate::error::AuditFsmError;
use crate::event::{AuditAnomalyRecord, AuditAnomalyType, AuditEvent, AuditEventBuilder, AnomalySeverity};
use crate::loader::topological_sort_procedures;
use crate::schema::*;
use chrono::{NaiveDateTime, TimeDelta};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use rand_distr::{Distribution, LogNormal};
use std::collections::HashMap;

/// Result of running a complete engagement.
#[derive(Debug, Clone)]
pub struct EngagementResult {
    pub event_log: Vec<AuditEvent>,
    pub procedure_states: HashMap<String, String>,
    pub step_completions: HashMap<String, bool>,
    pub evidence_states: HashMap<String, String>,
    pub anomalies: Vec<AuditAnomalyRecord>,
    pub phases_completed: Vec<String>,
    pub total_duration_hours: f64,
}

/// The FSM execution engine.
pub struct AuditFsmEngine {
    pub(crate) blueprint: AuditBlueprint,
    pub(crate) overlay: GenerationOverlay,
    pub(crate) rng: ChaCha8Rng,
}

impl AuditFsmEngine {
    /// Create a new engine with a parsed blueprint, overlay, and seed.
    pub fn new(blueprint: AuditBlueprint, overlay: GenerationOverlay, seed: u64) -> Self {
        Self {
            blueprint,
            overlay,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Run a complete engagement, returning all events and final state.
    /// The EngagementContext provides financial data for artifact generation.
    pub fn run_engagement(&mut self, context: &EngagementContext) -> Result<EngagementResult, AuditFsmError> {
        let procedure_order = topological_sort_procedures(&self.blueprint)?;

        // Build lookup maps
        let proc_map: HashMap<&str, &BlueprintProcedure> = self
            .blueprint
            .procedures
            .iter()
            .map(|p| (p.id.as_str(), p))
            .collect();
        let phase_map: HashMap<&str, &BlueprintPhase> = self
            .blueprint
            .phases
            .iter()
            .map(|p| (p.id.as_str(), p))
            .collect();

        let mut event_log = Vec::new();
        let mut procedure_states: HashMap<String, String> = HashMap::new();
        let mut step_completions: HashMap<String, bool> = HashMap::new();
        let mut evidence_states: HashMap<String, String> = HashMap::new();
        let mut anomalies = Vec::new();

        // Initialize evidence states from catalog
        for ev in &self.blueprint.evidence_catalog {
            if let Some(first) = ev.lifecycle.first() {
                evidence_states.insert(ev.id.clone(), first.clone());
            }
        }

        // Base timestamp from context
        let mut current_time = context.engagement_start.and_hms_opt(9, 0, 0).unwrap();

        for proc_id in &procedure_order {
            let proc = proc_map[proc_id.as_str()];
            let phase_id = &proc.phase;

            // Check discriminator filter
            if !self.passes_discriminator_filter(proc) {
                procedure_states.insert(proc_id.clone(), "skipped".to_string());
                continue;
            }

            // Initialize aggregate
            let mut current_state = proc.aggregate.initial_state.clone();

            // Walk the FSM until terminal (completed or similar)
            let max_iterations = 20; // bound revision loops
            let mut iterations = 0;

            while iterations < max_iterations {
                iterations += 1;

                // Find valid transitions from current state
                let valid_transitions: Vec<&ProcedureTransition> = proc
                    .aggregate
                    .transitions
                    .iter()
                    .filter(|t| t.from_state == current_state)
                    .collect();

                if valid_transitions.is_empty() {
                    // Terminal state — no outgoing transitions
                    break;
                }

                // Select transition (handle revision probability)
                let selected = self.select_transition(&valid_transitions, proc_id);

                // Advance time
                let lag = self.sample_timing(proc_id);
                current_time += TimeDelta::seconds((lag * 3600.0) as i64);

                // Emit transition event
                let actor = self.select_actor_for_transition(&selected.command, proc);
                event_log.push(
                    AuditEventBuilder::transition()
                        .procedure_id(proc_id)
                        .phase_id(phase_id)
                        .from_state(&current_state)
                        .to_state(&selected.to_state)
                        .command(&selected.command)
                        .event_type(&selected.emits)
                        .actor_id(&actor)
                        .timestamp(current_time)
                        .build_with_rng(&mut self.rng),
                );

                let prev_state = current_state.clone();
                current_state = selected.to_state.clone();

                // Execute steps when entering "in_progress" state
                if current_state == "in_progress" && prev_state != "in_progress" {
                    let mut sorted_steps: Vec<&BlueprintStep> = proc.steps.iter().collect();
                    sorted_steps.sort_by_key(|s| s.order);

                    for step in sorted_steps {
                        // Skip guidance steps if actor profile says so
                        if step.binding == BindingLevel::Guidance
                            && self.should_skip_guidance(&step.actor)
                        {
                            continue;
                        }

                        current_time += TimeDelta::seconds(
                            (self.rng.random_range(0.5_f64..4.0) * 3600.0) as i64,
                        );

                        // Collect evidence refs
                        let evidence_refs: Vec<String> = step
                            .evidence
                            .outputs
                            .iter()
                            .map(|e| e.ref_id.clone())
                            .collect();

                        // Update evidence states
                        for ev_ref in &step.evidence.outputs {
                            // Advance to next lifecycle state
                            if let Some(catalog_entry) = self
                                .blueprint
                                .evidence_catalog
                                .iter()
                                .find(|e| e.id == ev_ref.ref_id)
                            {
                                if let Some(current_ev_state) = evidence_states.get(&ev_ref.ref_id) {
                                    if let Some(pos) = catalog_entry
                                        .lifecycle
                                        .iter()
                                        .position(|s| s == current_ev_state)
                                    {
                                        if pos + 1 < catalog_entry.lifecycle.len() {
                                            evidence_states.insert(
                                                ev_ref.ref_id.clone(),
                                                catalog_entry.lifecycle[pos + 1].clone(),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        let standards_refs: Vec<String> =
                            step.standards.iter().map(|s| s.ref_id.clone()).collect();

                        // Check for anomaly injection
                        let mut builder = AuditEventBuilder::step()
                            .procedure_id(proc_id)
                            .step_id(&step.id)
                            .phase_id(phase_id)
                            .command(&step.command)
                            .event_type(&step.emits)
                            .actor_id(&step.actor)
                            .timestamp(current_time);

                        for r in &evidence_refs {
                            builder = builder.evidence_ref(r);
                        }
                        for r in &standards_refs {
                            builder = builder.standard_ref(r);
                        }

                        // Maybe inject anomaly
                        if let Some(anomaly_type) = self.maybe_inject_anomaly(step) {
                            builder = builder.anomaly(anomaly_type);
                            let bytes: [u8; 16] = self.rng.random();
                            anomalies.push(AuditAnomalyRecord {
                                anomaly_id: uuid::Builder::from_random_bytes(bytes).into_uuid(),
                                anomaly_type,
                                severity: match anomaly_type {
                                    AuditAnomalyType::SkippedApproval => AnomalySeverity::High,
                                    AuditAnomalyType::OutOfSequence => AnomalySeverity::Critical,
                                    AuditAnomalyType::MissingEvidence => AnomalySeverity::Medium,
                                    AuditAnomalyType::LatePosting => AnomalySeverity::Low,
                                    AuditAnomalyType::InsufficientDocumentation => AnomalySeverity::Medium,
                                },
                                procedure_id: proc_id.clone(),
                                step_id: Some(step.id.clone()),
                                timestamp: current_time,
                                description: format!(
                                    "{anomaly_type} in step {} of procedure {proc_id}",
                                    step.id
                                ),
                            });
                        }

                        event_log.push(builder.build_with_rng(&mut self.rng));
                        step_completions.insert(step.id.clone(), true);
                    }
                }
            }

            procedure_states.insert(proc_id.clone(), current_state);
        }

        let total_duration_hours = if let Some(first) = event_log.first() {
            if let Some(last) = event_log.last() {
                (last.timestamp - first.timestamp).num_seconds() as f64 / 3600.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let phases_completed: Vec<String> = self
            .blueprint
            .phases
            .iter()
            .map(|p| p.id.clone())
            .filter(|phase_id| {
                if let Some(phase) = phase_map.get(phase_id.as_str()) {
                    if let Some(gate) = &phase.gate {
                        gate.all_of.iter().all(|cond| {
                            procedure_states
                                .get(&cond.procedure)
                                .map(|s| s == &cond.state)
                                .unwrap_or(false)
                        })
                    } else {
                        true
                    }
                } else {
                    false
                }
            })
            .collect();

        Ok(EngagementResult {
            event_log,
            procedure_states,
            step_completions,
            evidence_states,
            anomalies,
            phases_completed,
            total_duration_hours,
        })
    }

    fn passes_discriminator_filter(&self, proc: &BlueprintProcedure) -> bool {
        // If overlay has no discriminator filter, all procedures pass
        let Some(overlay_discs) = &self.overlay.discriminators else {
            return true;
        };

        // If procedure has no discriminators, it always passes
        if proc.discriminators.is_empty() {
            return true;
        }

        // For each discriminator category in the procedure, check if overlay includes
        // at least one matching value
        for (category, proc_values) in &proc.discriminators {
            if let Some(overlay_values) = overlay_discs.get(category) {
                if !proc_values.iter().any(|v| overlay_values.contains(v)) {
                    return false;
                }
            }
        }
        true
    }

    fn select_transition<'a>(
        &mut self,
        transitions: &[&'a ProcedureTransition],
        proc_id: &str,
    ) -> &'a ProcedureTransition {
        if transitions.len() == 1 {
            return transitions[0];
        }

        // Check if this is a revision choice (under_review → in_progress vs under_review → completed)
        let has_revision = transitions.iter().any(|t| {
            t.from_state.contains("review") && t.to_state.contains("progress")
        });

        if has_revision {
            let revision_prob = self.get_revision_probability(proc_id);
            if self.rng.random::<f64>() < revision_prob {
                // Select the revision transition
                return transitions
                    .iter()
                    .find(|t| t.to_state.contains("progress"))
                    .unwrap_or(&transitions[0]);
            } else {
                // Select the forward transition
                return transitions
                    .iter()
                    .find(|t| !t.to_state.contains("progress"))
                    .unwrap_or(&transitions[0]);
            }
        }

        // Default: uniform random selection
        let idx = self.rng.random_range(0..transitions.len());
        transitions[idx]
    }

    fn get_revision_probability(&self, proc_id: &str) -> f64 {
        if let Some(ov) = self.overlay.transitions.overrides.get(proc_id) {
            if let Some(p) = ov.revision_probability {
                return p;
            }
        }
        self.overlay.transitions.defaults.revision_probability
    }

    fn sample_timing(&mut self, proc_id: &str) -> f64 {
        let timing = if let Some(ov) = self.overlay.transitions.overrides.get(proc_id) {
            ov.timing.as_ref().unwrap_or(&self.overlay.transitions.defaults.timing)
        } else {
            &self.overlay.transitions.defaults.timing
        };

        // Sample from log-normal distribution (mu/sigma are in hours)
        let mu = timing.mu_hours.ln();
        let sigma = timing.sigma_hours / timing.mu_hours; // CV as sigma for log-normal
        let log_normal = LogNormal::new(mu, sigma.max(0.01)).unwrap_or_else(|_| LogNormal::new(1.0, 0.5).unwrap());
        let sample: f64 = log_normal.sample(&mut self.rng);
        sample.max(0.5) // minimum 30 minutes
    }

    fn select_actor_for_transition(&mut self, command: &str, proc: &BlueprintProcedure) -> String {
        // Approval commands → senior actor; start/submit → assigned step actor
        if command.contains("approve") || command.contains("issue") {
            // Use the most senior actor from the procedure's steps
            proc.steps
                .iter()
                .find(|s| s.actor.contains("partner") || s.actor.contains("engagement_partner"))
                .or_else(|| proc.steps.iter().find(|s| s.actor.contains("manager")))
                .map(|s| s.actor.clone())
                .unwrap_or_else(|| {
                    proc.steps.first().map(|s| s.actor.clone()).unwrap_or_default()
                })
        } else if let Some(first_step) = proc.steps.first() {
            first_step.actor.clone()
        } else {
            "audit_manager".to_string()
        }
    }

    fn should_skip_guidance(&self, actor: &str) -> bool {
        self.overlay
            .actor_profiles
            .get(actor)
            .map(|p| p.skip_guidance_steps)
            .unwrap_or(false)
    }

    fn maybe_inject_anomaly(&mut self, step: &BlueprintStep) -> Option<AuditAnomalyType> {
        let anomalies = &self.overlay.anomalies;

        // Check skipped approval
        if !step.guards.is_empty() && self.rng.random::<f64>() < anomalies.skipped_approval.probability {
            return Some(AuditAnomalyType::SkippedApproval);
        }

        // Check missing evidence
        if !step.evidence.inputs.is_empty()
            && self.rng.random::<f64>() < anomalies.missing_evidence.probability
        {
            return Some(AuditAnomalyType::MissingEvidence);
        }

        // Check late posting (generic)
        if self.rng.random::<f64>() < anomalies.late_posting.probability {
            return Some(AuditAnomalyType::LatePosting);
        }

        None
    }
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Add `pub mod engine;` to lib.rs.

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p datasynth-audit-fsm`
Expected: no errors

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-audit-fsm/src/engine.rs crates/datasynth-audit-fsm/src/lib.rs
git commit -m "feat(audit-fsm): add FSM execution engine with DAG ordering and event emission"
```

---

### Task 7: Flat JSON Event Trail Exporter

**Files:**
- Create: `crates/datasynth-audit-fsm/src/export/mod.rs`
- Create: `crates/datasynth-audit-fsm/src/export/flat_log.rs`
- Modify: `crates/datasynth-audit-fsm/src/lib.rs`

- [ ] **Step 1: Write exporter test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AuditEvent;

    #[test]
    fn test_export_to_json_string() {
        let events = vec![
            crate::event::AuditEventBuilder::transition()
                .procedure_id("test_proc")
                .phase_id("planning")
                .from_state("not_started")
                .to_state("in_progress")
                .command("start")
                .event_type("TestStarted")
                .actor_id("manager")
                .timestamp(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap().and_hms_opt(9, 0, 0).unwrap())
                .build(),
        ];
        let json = export_events_to_json(&events).unwrap();
        assert!(json.contains("TestStarted"));
        assert!(json.contains("test_proc"));
        // Verify it's valid JSON array
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_full_engagement() {
        let bp = crate::loader::parse_blueprint(include_str!("../../blueprints/generic_fsa.yaml")).unwrap();
        let overlay = crate::loader::default_overlay();
        let mut engine = crate::engine::AuditFsmEngine::new(bp, overlay, 42);
        let result = engine.run_engagement().unwrap();
        let json = export_events_to_json(&result.event_log).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert!(parsed.len() >= 14);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL

- [ ] **Step 3: Implement the exporter**

`crates/datasynth-audit-fsm/src/export/mod.rs`:
```rust
pub mod flat_log;
pub use flat_log::*;
```

`crates/datasynth-audit-fsm/src/export/flat_log.rs`:
```rust
//! Flat JSON event trail exporter.

use crate::event::AuditEvent;
use std::io::Write;
use std::path::Path;

/// Export audit events to a JSON string (pretty-printed).
pub fn export_events_to_json(events: &[AuditEvent]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(events)
}

/// Export audit events to a JSON file.
pub fn export_events_to_file(events: &[AuditEvent], path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(events)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Add `pub mod export;` to lib.rs.

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/export/ crates/datasynth-audit-fsm/src/lib.rs
git commit -m "feat(audit-fsm): add flat JSON event trail exporter"
```

---

### Task 8: Configuration Schema Integration

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs` (add `AuditFsmConfig` under `AuditGenerationConfig`)

- [ ] **Step 1: Read the current AuditGenerationConfig location**

Read `crates/datasynth-config/src/schema.rs` around the `AuditGenerationConfig` struct to find the exact insertion point. It's around lines 3814-3856.

- [ ] **Step 2: Add the FSM config struct**

Add near the existing `AuditGenerationConfig`:

```rust
/// FSM-driven audit generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFsmConfig {
    /// Enable FSM-driven audit generation (replaces standalone generators when true).
    #[serde(default)]
    pub enabled: bool,

    /// Blueprint source: "builtin:fsa", "builtin:ia", or a file path.
    #[serde(default = "default_blueprint")]
    pub blueprint: String,

    /// Overlay source: "builtin:default", "builtin:thorough", "builtin:rushed", or a file path.
    #[serde(default = "default_overlay_source")]
    pub overlay: String,

    /// Depth level override (overrides blueprint default).
    #[serde(default)]
    pub depth: Option<String>,

    /// Discriminator filter for scoping procedures.
    #[serde(default)]
    pub discriminators: std::collections::HashMap<String, Vec<String>>,

    /// Event trail output configuration.
    #[serde(default)]
    pub event_trail: AuditEventTrailConfig,

    /// RNG seed override (null = use global seed).
    #[serde(default)]
    pub seed: Option<u64>,
}

fn default_blueprint() -> String {
    "builtin:fsa".to_string()
}

fn default_overlay_source() -> String {
    "builtin:default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventTrailConfig {
    #[serde(default = "default_true")]
    pub flat_log: bool,
    #[serde(default)]
    pub ocel_projection: bool,
}

impl Default for AuditEventTrailConfig {
    fn default() -> Self {
        Self { flat_log: true, ocel_projection: false }
    }
}
```

Then add the `fsm` field to `AuditGenerationConfig`:

```rust
// Add to existing AuditGenerationConfig:
    #[serde(default)]
    pub fsm: Option<AuditFsmConfig>,
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p datasynth-config`
Expected: compiles. There may be name collisions with existing `default_true` — if so, reuse the existing one.

- [ ] **Step 4: Verify existing tests still pass**

Run: `cargo test -p datasynth-config -- --test-threads=4`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-config/src/schema.rs
git commit -m "feat(config): add audit.fsm configuration section"
```

---

### Task 9: Integration Test — Full FSA Engagement

**Files:**
- Create: `crates/datasynth-audit-fsm/tests/fsa_integration.rs`

This is the end-to-end integration test that validates the full pipeline: load blueprint → run engine → export events.

- [ ] **Step 1: Write the integration test**

```rust
//! Integration test: full FSA engagement run.

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::export::flat_log::{export_events_to_json, export_events_to_file};
use datasynth_audit_fsm::loader::{parse_blueprint, default_overlay, validate_blueprint};

const FSA_YAML: &str = include_str!("../blueprints/generic_fsa.yaml");

fn ctx() -> EngagementContext {
    EngagementContext::test_default()
}

#[test]
fn test_fsa_full_engagement() {
    // Load and validate
    let bp = parse_blueprint(FSA_YAML).unwrap();
    validate_blueprint(&bp).unwrap();

    // Run with default overlay
    let overlay = default_overlay();
    let mut engine = AuditFsmEngine::new(bp, overlay, 12345);
    let result = engine.run_engagement(&ctx()).unwrap();

    // All 7 procedures completed
    assert_eq!(result.procedure_states.len(), 7);
    for (proc_id, state) in &result.procedure_states {
        assert_eq!(state, "completed", "Procedure {proc_id} not completed");
    }

    // All 3 phases completed
    assert_eq!(result.phases_completed.len(), 3);

    // Events are ordered by timestamp
    for window in result.event_log.windows(2) {
        assert!(
            window[0].timestamp <= window[1].timestamp,
            "Events out of order: {} > {}",
            window[0].timestamp,
            window[1].timestamp
        );
    }

    // Every event references a valid procedure
    let valid_procs: std::collections::HashSet<&str> = result.procedure_states.keys().map(|s| s.as_str()).collect();
    for event in &result.event_log {
        assert!(valid_procs.contains(event.procedure_id.as_str()),
            "Event references unknown procedure: {}", event.procedure_id);
    }

    // Export to JSON roundtrips
    let json = export_events_to_json(&result.event_log).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), result.event_log.len());

    // Evidence states advanced
    assert!(result.evidence_states.values().any(|s| s != "draft"),
        "No evidence advanced beyond draft state");

    // Duration is positive
    assert!(result.total_duration_hours > 0.0);
}

#[test]
fn test_fsa_determinism_across_runs() {
    let bp1 = parse_blueprint(FSA_YAML).unwrap();
    let bp2 = parse_blueprint(FSA_YAML).unwrap();

    let mut e1 = AuditFsmEngine::new(bp1, default_overlay(), 99);
    let mut e2 = AuditFsmEngine::new(bp2, default_overlay(), 99);
    let c = ctx();

    let r1 = e1.run_engagement(&c).unwrap();
    let r2 = e2.run_engagement(&c).unwrap();

    // Same number of events
    assert_eq!(r1.event_log.len(), r2.event_log.len());

    // Same event sequence
    for (a, b) in r1.event_log.iter().zip(r2.event_log.iter()) {
        assert_eq!(a.event_type, b.event_type);
        assert_eq!(a.procedure_id, b.procedure_id);
        assert_eq!(a.step_id, b.step_id);
        assert_eq!(a.command, b.command);
        assert_eq!(a.timestamp, b.timestamp);
    }
}

#[test]
fn test_fsa_with_custom_overlay() {
    let bp = parse_blueprint(FSA_YAML).unwrap();

    // High revision probability to test revision loops
    let overlay_yaml = r#"
transitions:
  defaults:
    revision_probability: 0.9
    timing:
      distribution: log_normal
      mu_hours: 8.0
      sigma_hours: 2.0
"#;
    let overlay: datasynth_audit_fsm::schema::GenerationOverlay =
        serde_yaml::from_str(overlay_yaml).unwrap();

    let mut engine = AuditFsmEngine::new(bp, overlay, 42);
    let result = engine.run_engagement(&ctx()).unwrap();

    // Should have more events due to revision loops
    // With 0.9 revision probability, most procedures will have at least one revision
    let revision_events: Vec<_> = result.event_log.iter()
        .filter(|e| e.event_type.contains("Revision"))
        .collect();
    assert!(!revision_events.is_empty(), "Expected revision events with 0.9 revision probability");
}

#[test]
fn test_fsa_export_to_temp_file() {
    let bp = parse_blueprint(FSA_YAML).unwrap();
    let mut engine = AuditFsmEngine::new(bp, default_overlay(), 42);
    let result = engine.run_engagement(&ctx()).unwrap();

    let dir = std::env::temp_dir().join("datasynth_test_audit_fsm");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("audit_event_trail.json");

    export_events_to_file(&result.event_log, &path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.len(), result.event_log.len());

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: Run integration tests**

Run: `cargo test -p datasynth-audit-fsm --test fsa_integration -- --test-threads=4`
Expected: all 4 tests PASS

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS (unit + integration)

- [ ] **Step 4: Run clippy on the crate**

Run: `cargo clippy -p datasynth-audit-fsm`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/tests/
git commit -m "test(audit-fsm): add FSA integration tests for full engagement pipeline"
```

---

### Task 10: Builtin Overlay Presets

**Files:**
- Create: `crates/datasynth-audit-fsm/overlays/default.yaml`
- Create: `crates/datasynth-audit-fsm/overlays/thorough.yaml`
- Create: `crates/datasynth-audit-fsm/overlays/rushed.yaml`
- Modify: `crates/datasynth-audit-fsm/src/loader.rs` (add builtin overlay loading)

- [ ] **Step 1: Create overlay YAML files**

`overlays/default.yaml`:
```yaml
transitions:
  defaults:
    revision_probability: 0.15
    timing:
      distribution: log_normal
      mu_hours: 24.0
      sigma_hours: 8.0
artifacts:
  workpapers_per_step:
    min: 1
    max: 3
  evidence_items_per_workpaper:
    min: 2
    max: 5
anomalies:
  skipped_approval:
    probability: 0.02
  late_posting:
    probability: 0.05
    max_delay_hours: 72.0
  missing_evidence:
    probability: 0.03
  out_of_sequence:
    probability: 0.01
```

`overlays/thorough.yaml`:
```yaml
transitions:
  defaults:
    revision_probability: 0.30
    timing:
      distribution: log_normal
      mu_hours: 40.0
      sigma_hours: 12.0
artifacts:
  workpapers_per_step:
    min: 2
    max: 5
  evidence_items_per_workpaper:
    min: 4
    max: 8
anomalies:
  skipped_approval:
    probability: 0.005
  late_posting:
    probability: 0.02
    max_delay_hours: 24.0
  missing_evidence:
    probability: 0.01
  out_of_sequence:
    probability: 0.002
```

`overlays/rushed.yaml`:
```yaml
transitions:
  defaults:
    revision_probability: 0.05
    timing:
      distribution: log_normal
      mu_hours: 8.0
      sigma_hours: 4.0
artifacts:
  workpapers_per_step:
    min: 1
    max: 2
  evidence_items_per_workpaper:
    min: 1
    max: 3
anomalies:
  skipped_approval:
    probability: 0.08
  late_posting:
    probability: 0.15
    max_delay_hours: 120.0
  missing_evidence:
    probability: 0.10
  out_of_sequence:
    probability: 0.05
actor_profiles:
  audit_staff:
    revision_multiplier: 0.5
    evidence_multiplier: 0.7
    skip_guidance_steps: true
  audit_senior:
    revision_multiplier: 0.7
    evidence_multiplier: 0.8
    skip_guidance_steps: true
```

- [ ] **Step 2: Wire builtin overlay constants into loader**

In `crates/datasynth-audit-fsm/src/loader.rs`, add the include constants and update the `load_overlay` match to resolve `Thorough` and `Rushed` (previously they fell through to `default_overlay()`):

```rust
const BUILTIN_OVERLAY_DEFAULT: &str = include_str!("../overlays/default.yaml");
const BUILTIN_OVERLAY_THOROUGH: &str = include_str!("../overlays/thorough.yaml");
const BUILTIN_OVERLAY_RUSHED: &str = include_str!("../overlays/rushed.yaml");
```

Update the `load_overlay` `Builtin` arm (the enum was already defined in Task 4):

```rust
OverlaySource::Builtin(b) => {
    let yaml = match b {
        BuiltinOverlay::Default => BUILTIN_OVERLAY_DEFAULT,
        BuiltinOverlay::Thorough => BUILTIN_OVERLAY_THOROUGH,
        BuiltinOverlay::Rushed => BUILTIN_OVERLAY_RUSHED,
    };
    parse_overlay(yaml)
}
```

- [ ] **Step 3: Add tests for builtin overlays**

```rust
#[test]
fn test_load_builtin_default_overlay() {
    let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default)).unwrap();
    assert!((overlay.transitions.defaults.revision_probability - 0.15).abs() < 0.001);
}

#[test]
fn test_load_builtin_thorough_overlay() {
    let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Thorough)).unwrap();
    assert!((overlay.transitions.defaults.revision_probability - 0.30).abs() < 0.001);
}

#[test]
fn test_load_builtin_rushed_overlay() {
    let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Rushed)).unwrap();
    assert!(overlay.transitions.defaults.revision_probability < 0.10);
    assert!(overlay.actor_profiles.contains_key("audit_staff"));
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/overlays/ crates/datasynth-audit-fsm/src/loader.rs
git commit -m "feat(audit-fsm): add builtin overlay presets (default, thorough, rushed)"
```

---

### Task 11: Final Validation and Cleanup

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/lib.rs` (clean up re-exports)

- [ ] **Step 1: Update lib.rs with clean public API**

```rust
//! YAML-driven audit FSM engine.
//!
//! Loads audit methodology blueprints (ISA, IIA-GIAS) as finite state machines
//! and generates realistic audit artifacts with event trail output.
//!
//! # Quick Start
//!
//! ```no_run
//! use datasynth_audit_fsm::loader::{load_blueprint, load_overlay, BlueprintSource, BuiltinBlueprint, OverlaySource, BuiltinOverlay, validate_blueprint};
//! use datasynth_audit_fsm::engine::AuditFsmEngine;
//! use datasynth_audit_fsm::export::flat_log::export_events_to_json;
//!
//! let bp = load_blueprint(&BlueprintSource::Builtin(BuiltinBlueprint::Fsa)).unwrap();
//! validate_blueprint(&bp).unwrap();
//! let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default)).unwrap();
//! let mut engine = AuditFsmEngine::new(bp, overlay, 42);
//! let result = engine.run_engagement().unwrap();
//! let json = export_events_to_json(&result.event_log).unwrap();
//! ```

pub mod engine;
pub mod error;
pub mod event;
pub mod export;
pub mod loader;
pub mod schema;
```

- [ ] **Step 2: Run full workspace check**

Run: `cargo check --workspace`
Expected: compiles (only expected warning: protoc not found)

- [ ] **Step 3: Run all tests in the crate**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p datasynth-audit-fsm`
Expected: clean

- [ ] **Step 5: Run fmt**

Run: `cargo fmt -p datasynth-audit-fsm`

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-audit-fsm/
git commit -m "feat(audit-fsm): finalize Phase 1 — FSA blueprint engine with event trail export"
```

---

## Summary

| Task | What it delivers | Key files |
|------|-----------------|-----------|
| 1 | Crate skeleton in workspace | `Cargo.toml`, `src/lib.rs` |
| 2 | Blueprint + overlay schema types | `src/schema.rs` |
| 3 | Error types | `src/error.rs` |
| 4 | Blueprint loader with validation + DAG sort | `src/loader.rs`, `blueprints/generic_fsa.yaml` |
| 5 | Event types and builder | `src/event.rs` |
| 6 | FSM execution engine | `src/engine.rs` |
| 7 | Flat JSON event trail exporter | `src/export/flat_log.rs` |
| 8 | Config schema integration | `datasynth-config/src/schema.rs` |
| 9 | Integration tests | `tests/fsa_integration.rs` |
| 10 | Builtin overlay presets | `overlays/*.yaml` |
| 11 | Final validation and cleanup | `src/lib.rs` |

After completion: the crate can load the FSA blueprint, walk all 7 procedures in DAG order, execute 20+ steps, emit 30+ audit events, handle revision loops, inject anomalies, and export a flat JSON audit trail. This establishes the architecture for Phase 2 (IA blueprint) and Phase 3 (optimizer).
