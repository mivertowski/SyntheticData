# Generic Process Blueprint Engine — Phase 1: Core Engine

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `datasynth-process-engine`, a domain-agnostic FSM process engine with artifact registry, schema-driven generation, and process mining-ready event trails.

**Architecture:** New standalone crate extracting generic concepts from `datasynth-audit-fsm` (which remains unchanged in this phase). The crate provides: blueprint schema types, YAML loader/validator, FSM execution engine, artifact registry with trait-based dispatch and schema-driven fallback, and multi-format event trail export. Domain crates register their generators via the `ArtifactGenerator` trait.

**Tech Stack:** Rust, serde/serde_yaml (YAML parsing), chrono (temporal), rust_decimal (precise decimals), rand/rand_chacha (deterministic RNG), rand_distr (distributions), uuid, thiserror, tracing, indexmap (ordered maps)

**Spec:** `docs/superpowers/specs/2026-04-01-generic-process-blueprint-engine-design.md`

**Scope:** This plan covers Phase 1 only (core engine creation). Phases 2-5 (audit-fsm refactor, P2P/O2C facades, CLI integration, schema generator enhancements) will be separate plans.

---

## File Structure

```
crates/datasynth-process-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Module declarations, crate docs, re-exports
│   ├── schema.rs                 # ProcessBlueprint, ProcessPhase, ProcessProcedure, ProcessStep,
│   │                             # ProcedureAggregate, StateTransition, PhaseGate, GateCondition,
│   │                             # ProcessActor, ProcessDomain, DepthLevel, BindingLevel
│   ├── artifact_schema.rs        # ArtifactSchema, FieldSchema, FieldType, FieldGenerator,
│   │                             # DistributionSpec, DistributionType, FieldConstraint, ExternalSchemaRef
│   ├── event.rs                  # ProcessEvent, ProcessAnomalyType, AnomalySeverity,
│   │                             # ProcessAnomalyRecord, ProcessEventBuilder
│   ├── error.rs                  # ProcessEngineError, ValidationViolation
│   ├── context.rs                # StepContext, ArtifactRef
│   ├── record.rs                 # GeneratedRecord, RecordValue, RecordMetadata, ToGeneratedRecord trait
│   ├── registry.rs               # ArtifactGenerator trait, FieldDescriptor, ArtifactRegistry
│   ├── overlay.rs                # GenerationOverlay, TransitionConfig, TimingDistribution,
│   │                             # AnomalyConfig, ActorProfile, ProcessVariant, VolumeConfig
│   ├── loader.rs                 # BlueprintSource, parse_blueprint, validate_blueprint,
│   │                             # topological_sort_procedures, BlueprintWithPreconditions
│   ├── engine.rs                 # ProcessEngine, EngineResult, run_process
│   ├── schema_generator.rs       # SchemaGenerator — fallback for custom artifact types
│   └── export/
│       ├── mod.rs                # Re-exports
│       ├── json.rs               # Flat JSON event log export
│       ├── csv.rs                # CSV event log export
│       └── ocel.rs               # OCEL 2.0 projection export
└── tests/
    ├── schema_tests.rs           # Blueprint YAML round-trip parsing
    ├── loader_tests.rs           # Validation: DAG cycles, missing refs, unreachable states
    ├── engine_tests.rs           # FSM transitions, topological walk, event emission
    ├── registry_tests.rs         # Trait dispatch, schema fallback, unknown type
    ├── schema_generator_tests.rs # Field types, distributions, constraints
    └── integration_test.rs       # End-to-end: load blueprint → run → verify events + artifacts
```

---

### Task 1: Create crate scaffold and workspace registration

**Files:**
- Create: `crates/datasynth-process-engine/Cargo.toml`
- Create: `crates/datasynth-process-engine/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "datasynth-process-engine"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
description = "Domain-agnostic YAML-driven process FSM engine with artifact registry"
keywords = ["process", "fsm", "blueprint", "synthetic-data", "process-mining"]
categories.workspace = true

[dependencies]
datasynth-core = { workspace = true }
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
indexmap = { version = "2", features = ["serde"] }

[dev-dependencies]
proptest = { workspace = true }
tempfile = { workspace = true }
```

- [ ] **Step 2: Create lib.rs with module stubs**

```rust
//! Domain-agnostic YAML-driven process FSM engine.
//!
//! Loads process blueprints as finite state machines and generates realistic
//! event trails and artifacts with configurable behavior via overlays.
//!
//! # Architecture
//!
//! - **Blueprint**: YAML-defined process (phases → procedures → steps)
//! - **Overlay**: YAML-defined generation parameters (timing, anomalies, variants)
//! - **Registry**: Maps artifact types to generators (built-in or schema-driven)
//! - **Engine**: Walks FSM, executes steps, emits events, produces artifacts

pub mod artifact_schema;
pub mod context;
pub mod engine;
pub mod error;
pub mod event;
pub mod export;
pub mod loader;
pub mod overlay;
pub mod record;
pub mod registry;
pub mod schema;
pub mod schema_generator;
```

- [ ] **Step 3: Add to workspace Cargo.toml**

In the root `Cargo.toml`, add `"crates/datasynth-process-engine"` to the `members` list and add the workspace dependency:

```toml
# In [workspace.members]:
"crates/datasynth-process-engine",

# In [workspace.dependencies]:
datasynth-process-engine = { version = "2.0.0", path = "crates/datasynth-process-engine" }
```

- [ ] **Step 4: Create empty module files so the crate compiles**

Create empty files for each module declared in `lib.rs`:
- `src/schema.rs`
- `src/artifact_schema.rs`
- `src/event.rs`
- `src/error.rs`
- `src/context.rs`
- `src/record.rs`
- `src/registry.rs`
- `src/overlay.rs`
- `src/loader.rs`
- `src/engine.rs`
- `src/schema_generator.rs`
- `src/export/mod.rs`

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p datasynth-process-engine`
Expected: Compiles with no errors (warnings OK for empty modules)

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-process-engine/ Cargo.toml
git commit -m "feat(process-engine): create crate scaffold with module stubs"
```

---

### Task 2: Define error types

**Files:**
- Modify: `crates/datasynth-process-engine/src/error.rs`
- Create: `crates/datasynth-process-engine/tests/schema_tests.rs` (stub)

- [ ] **Step 1: Write error types**

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessEngineError {
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

    #[error("Schema file not found: {path}")]
    SchemaFileNotFound { path: String },

    #[error("Schema parse error ({path}): {source}")]
    SchemaParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("DAG cycle detected involving procedures: {procedures:?}")]
    DagCycle { procedures: Vec<String> },

    #[error("Precondition not met for '{procedure_id}': requires '{required}' in state '{required_state}' but was '{actual_state}'")]
    PreconditionNotMet {
        procedure_id: String,
        required: String,
        required_state: String,
        actual_state: String,
    },

    #[error("No valid transition from state '{current_state}' in procedure '{procedure_id}'")]
    NoValidTransition {
        procedure_id: String,
        current_state: String,
    },

    #[error("Artifact generation failed for type '{artifact_type}': {reason}")]
    ArtifactGeneration {
        artifact_type: String,
        reason: String,
    },

    #[error("Unknown artifact type '{artifact_type}' with no schema defined")]
    UnknownArtifactType { artifact_type: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub location: String,
    pub message: String,
}

impl ValidationViolation {
    pub fn new(location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            message: message.into(),
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p datasynth-process-engine`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add crates/datasynth-process-engine/src/error.rs
git commit -m "feat(process-engine): define ProcessEngineError and ValidationViolation"
```

---

### Task 3: Define core blueprint schema types

**Files:**
- Modify: `crates/datasynth-process-engine/src/schema.rs`
- Create: `crates/datasynth-process-engine/tests/schema_tests.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/schema_tests.rs`:

```rust
use datasynth_process_engine::schema::*;

#[test]
fn test_minimal_blueprint_parses_from_yaml() {
    let yaml = r#"
id: test_process
name: "Test Process"
version: "1.0.0"
schema_version: "2.0"
domain: custom
depth: standard
phases: []
"#;
    let bp: ProcessBlueprint = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(bp.id, "test_process");
    assert_eq!(bp.name, "Test Process");
    assert_eq!(bp.schema_version, "2.0");
    assert!(matches!(bp.domain, ProcessDomain::Custom(ref s) if s == "custom"));
    assert!(matches!(bp.depth, DepthLevel::Standard));
    assert!(bp.phases.is_empty());
}

#[test]
fn test_full_blueprint_with_phases_and_procedures() {
    let yaml = r#"
id: procurement_p2p
name: "Procure to Pay"
version: "1.0.0"
schema_version: "2.0"
domain: procurement
depth: standard

actors:
  - id: buyer
    name: "Procurement Buyer"
  - id: approver
    name: "Approver"

phases:
  - id: requisition
    name: "Requisition"
    order: 1
    procedures:
      - id: create_po
        title: "Create Purchase Order"
        aggregate:
          initial_state: draft
          states: [draft, pending_approval, approved]
          transitions:
            - from_state: draft
              to_state: pending_approval
              command: submit_po
              emits: POSubmitted
            - from_state: pending_approval
              to_state: approved
              command: approve_po
              emits: POApproved
              guards: [within_budget]
        steps:
          - id: step_create
            order: 1
            action: create
            actor: buyer
            description: "Create the purchase order"
            command: create_purchase_order
            emits: PurchaseOrderCreated
            artifact_type: purchase_order
        preconditions: []
"#;
    let bp: ProcessBlueprint = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(bp.phases.len(), 1);
    let phase = &bp.phases[0];
    assert_eq!(phase.id, "requisition");
    assert_eq!(phase.order, Some(1));

    let proc = &phase.procedures[0];
    assert_eq!(proc.id, "create_po");
    assert_eq!(proc.aggregate.initial_state, "draft");
    assert_eq!(proc.aggregate.states.len(), 3);
    assert_eq!(proc.aggregate.transitions.len(), 2);
    assert_eq!(proc.aggregate.transitions[1].guards, vec!["within_budget"]);

    let step = &proc.steps[0];
    assert_eq!(step.action, "create");
    assert_eq!(step.actor, "buyer");
    assert_eq!(step.artifact_type.as_deref(), Some("purchase_order"));
}

#[test]
fn test_known_domains_parse() {
    for (yaml_val, expected) in [
        ("audit", ProcessDomain::Audit),
        ("procurement", ProcessDomain::Procurement),
        ("sales", ProcessDomain::Sales),
        ("manufacturing", ProcessDomain::Manufacturing),
        ("banking", ProcessDomain::Banking),
        ("human_resources", ProcessDomain::HumanResources),
    ] {
        let yaml = format!(
            "id: t\nname: t\nversion: '1'\nschema_version: '2.0'\ndomain: {}\ndepth: standard\nphases: []",
            yaml_val
        );
        let bp: ProcessBlueprint = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(std::mem::discriminant(&bp.domain), std::mem::discriminant(&expected));
    }
}

#[test]
fn test_phase_gate_parses() {
    let yaml = r#"
id: t
name: t
version: "1"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase2
    name: "Phase 2"
    order: 2
    gate:
      all_of:
        - procedure: proc_a
          state: completed
        - procedure: proc_b
          state: completed
    procedures: []
"#;
    let bp: ProcessBlueprint = serde_yaml::from_str(yaml).unwrap();
    let gate = bp.phases[0].gate.as_ref().unwrap();
    assert_eq!(gate.all_of.len(), 2);
    assert_eq!(gate.all_of[0].procedure, "proc_a");
    assert_eq!(gate.all_of[0].state, "completed");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-process-engine --test schema_tests -- --test-threads=4 2>&1 | head -20`
Expected: Compilation errors — `ProcessBlueprint` not defined yet

- [ ] **Step 3: Implement schema types**

Write `src/schema.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Enums =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DepthLevel {
    Simplified,
    #[default]
    Standard,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BindingLevel {
    #[default]
    Requirement,
    Guidance,
    Optional,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessDomain {
    Audit,
    Procurement,
    Sales,
    Manufacturing,
    Banking,
    HumanResources,
    #[serde(untagged)]
    Custom(String),
}

impl Default for ProcessDomain {
    fn default() -> Self {
        Self::Custom("custom".to_string())
    }
}

// ===== Top-level Blueprint =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBlueprint {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub domain: ProcessDomain,
    #[serde(default)]
    pub methodology: Option<BlueprintMethodology>,
    #[serde(default)]
    pub depth: DepthLevel,

    #[serde(default)]
    pub discriminators: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub actors: Vec<ProcessActor>,
    #[serde(default)]
    pub artifact_schemas: Vec<crate::artifact_schema::ArtifactSchema>,
    #[serde(default)]
    pub evidence_templates: Vec<EvidenceTemplate>,
    #[serde(default)]
    pub standards: Vec<StandardReference>,
    #[serde(default)]
    pub external_schemas: Vec<crate::artifact_schema::ExternalSchemaRef>,

    #[serde(default)]
    pub phases: Vec<ProcessPhase>,
}

fn default_schema_version() -> String {
    "2.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintMethodology {
    pub framework: String,
    #[serde(default)]
    pub default_depth: DepthLevel,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessActor {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTemplate {
    pub id: String,
    #[serde(rename = "type", default)]
    pub evidence_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardReference {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub binding: BindingLevel,
}

// ===== Phase =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPhase {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gate: Option<PhaseGate>,
    #[serde(default)]
    pub procedures: Vec<ProcessProcedure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseGate {
    #[serde(default)]
    pub all_of: Vec<GateCondition>,
    #[serde(default)]
    pub any_of: Vec<GateCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCondition {
    pub procedure: String,
    pub state: String,
}

// ===== Procedure =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessProcedure {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub discriminators: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub aggregate: ProcedureAggregate,
    #[serde(default)]
    pub steps: Vec<ProcessStep>,
    #[serde(default)]
    pub preconditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcedureAggregate {
    #[serde(default = "default_initial_state")]
    pub initial_state: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<StateTransition>,
}

fn default_initial_state() -> String {
    "not_started".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub emits: String,
    #[serde(default)]
    pub guards: Vec<String>,
}

// ===== Step =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    pub id: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub binding: Option<BindingLevel>,

    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub emits: String,

    #[serde(default)]
    pub artifact_type: Option<String>,
    #[serde(default)]
    pub artifact_overrides: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub evidence: Option<StepEvidenceSpec>,
    #[serde(default)]
    pub standards: Vec<StepStandardMapping>,
    #[serde(default)]
    pub decisions: Vec<StepDecision>,
    #[serde(default)]
    pub guards: Vec<StepGuard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvidenceSpec {
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<StepEvidenceOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvidenceOutput {
    pub ref_id: String,
    #[serde(rename = "type", default)]
    pub output_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStandardMapping {
    #[serde(rename = "ref")]
    pub standard_ref: String,
    #[serde(default)]
    pub paragraphs: Vec<String>,
    #[serde(default)]
    pub binding: Option<BindingLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDecision {
    pub condition: String,
    #[serde(default)]
    pub branches: Vec<DecisionBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionBranch {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGuard {
    #[serde(rename = "type")]
    pub guard_type: String,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub condition: Option<String>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p datasynth-process-engine --test schema_tests -- --test-threads=4`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/src/schema.rs crates/datasynth-process-engine/tests/schema_tests.rs
git commit -m "feat(process-engine): define core blueprint schema types with YAML serde"
```

---

### Task 4: Define artifact schema types

**Files:**
- Modify: `crates/datasynth-process-engine/src/artifact_schema.rs`
- Modify: `crates/datasynth-process-engine/tests/schema_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/schema_tests.rs`:

```rust
use datasynth_process_engine::artifact_schema::*;

#[test]
fn test_artifact_schema_parses_from_yaml() {
    let yaml = r#"
id: loan_application
description: "Consumer loan application"
fields:
  - name: application_id
    type: string
    generator: uuid
  - name: loan_amount
    type: decimal
    distribution:
      type: lognormal
      params:
        mu: 10.2
        sigma: 1.4
      benford_compliance: true
  - name: credit_score
    type: integer
    distribution:
      type: normal
      params:
        mean: 720.0
        std: 80.0
        min: 300.0
        max: 850.0
  - name: loan_type
    type:
      enum:
        values: [mortgage, auto, personal]
        weights: [0.4, 0.35, 0.25]
  - name: branch_code
    type: string
    generator:
      from_context: branch_code
    nullable: true
    null_rate: 0.05
constraints:
  - type: correlation
    fields: [loan_amount, credit_score]
    coefficient: -0.3
"#;
    let schema: ArtifactSchema = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(schema.id, "loan_application");
    assert_eq!(schema.fields.len(), 5);

    // Check field types
    assert!(matches!(schema.fields[0].field_type, FieldType::String));
    assert!(matches!(schema.fields[0].generator, Some(FieldGenerator::Uuid)));
    assert!(matches!(schema.fields[1].field_type, FieldType::Decimal));
    assert!(schema.fields[1].distribution.as_ref().unwrap().benford_compliance);
    assert!(matches!(schema.fields[3].field_type, FieldType::Enum { .. }));

    // Check nullable
    assert!(schema.fields[4].nullable);
    assert_eq!(schema.fields[4].null_rate, Some(0.05));

    // Check constraints
    assert_eq!(schema.constraints.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p datasynth-process-engine --test schema_tests test_artifact_schema -- --test-threads=4`
Expected: Compilation error — `artifact_schema` module is empty

- [ ] **Step 3: Implement artifact schema types**

Write `src/artifact_schema.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSchema {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldSchema>,
    #[serde(default)]
    pub constraints: Vec<FieldConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub generator: Option<FieldGenerator>,
    #[serde(default)]
    pub distribution: Option<DistributionSpec>,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub null_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    DateTime,
    #[serde(rename = "enum")]
    Enum {
        values: Vec<String>,
        #[serde(default)]
        weights: Vec<f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldGenerator {
    Uuid,
    PersonName,
    CompanyName,
    Address,
    Email,
    Phone,
    StepTimestamp,
    FromContext(String),
    Sequential(String),
    Pattern(String),
    Reference(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSpec {
    #[serde(rename = "type")]
    pub dist_type: DistributionType,
    #[serde(default)]
    pub params: HashMap<String, f64>,
    #[serde(default)]
    pub benford_compliance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    Normal,
    LogNormal,
    Beta,
    Uniform,
    Pareto,
    Weibull,
    ZeroInflated,
    Mixture(Vec<MixtureComponent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureComponent {
    pub weight: f64,
    pub distribution: Box<DistributionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldConstraint {
    Conditional {
        when: FieldCondition,
        then: Vec<FieldOverride>,
    },
    Correlation {
        fields: Vec<String>,
        coefficient: f64,
    },
    UniqueWithin {
        field: String,
        scope: String,
    },
    ForeignKey {
        field: String,
        references: ArtifactFieldRef,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCondition {
    pub field: String,
    pub equals: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOverride {
    pub field: String,
    #[serde(default)]
    pub distribution: Option<DistributionSpec>,
    #[serde(default)]
    pub generator: Option<FieldGenerator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFieldRef {
    pub artifact_type: String,
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSchemaRef {
    pub id: String,
    pub path: String,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p datasynth-process-engine --test schema_tests -- --test-threads=4`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/src/artifact_schema.rs crates/datasynth-process-engine/tests/schema_tests.rs
git commit -m "feat(process-engine): define artifact schema types for schema-driven generation"
```

---

### Task 5: Define ProcessEvent and builder

**Files:**
- Modify: `crates/datasynth-process-engine/src/event.rs`
- Create: `crates/datasynth-process-engine/tests/event_tests.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/event_tests.rs`:

```rust
use chrono::NaiveDateTime;
use datasynth_process_engine::event::*;

#[test]
fn test_event_builder_creates_transition_event() {
    let event = ProcessEventBuilder::transition()
        .case_id("CASE-001")
        .procedure_id("create_po")
        .phase_id("requisition")
        .from_state("draft")
        .to_state("pending_approval")
        .actor_id("buyer_01")
        .command("submit_po")
        .emits("POSubmitted")
        .timestamp(NaiveDateTime::parse_from_str("2024-06-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap())
        .build();

    assert_eq!(event.case_id, "CASE-001");
    assert_eq!(event.event_type, "transition");
    assert_eq!(event.procedure_id, "create_po");
    assert_eq!(event.from_state.as_deref(), Some("draft"));
    assert_eq!(event.to_state.as_deref(), Some("pending_approval"));
    assert_eq!(event.command, "submit_po");
    assert!(!event.is_anomaly);
}

#[test]
fn test_event_builder_creates_step_event() {
    let event = ProcessEventBuilder::step()
        .case_id("CASE-001")
        .procedure_id("create_po")
        .step_id("step_create")
        .phase_id("requisition")
        .actor_id("buyer_01")
        .command("create_purchase_order")
        .emits("PurchaseOrderCreated")
        .artifact_type("purchase_order")
        .build();

    assert_eq!(event.event_type, "step");
    assert_eq!(event.step_id.as_deref(), Some("step_create"));
    assert_eq!(event.artifact_type.as_deref(), Some("purchase_order"));
}

#[test]
fn test_anomaly_event() {
    let event = ProcessEventBuilder::step()
        .case_id("CASE-002")
        .procedure_id("approve_po")
        .phase_id("requisition")
        .actor_id("unauthorized_user")
        .command("approve_po")
        .anomaly(ProcessAnomalyType::UnauthorizedActor)
        .build();

    assert!(event.is_anomaly);
    assert_eq!(event.anomaly_type, Some(ProcessAnomalyType::UnauthorizedActor));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p datasynth-process-engine --test event_tests -- --test-threads=4`
Expected: Compilation error

- [ ] **Step 3: Implement event types**

Write `src/event.rs`:

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_id: Uuid,
    pub case_id: String,
    pub timestamp: NaiveDateTime,
    pub event_type: String,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub phase_id: String,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub actor_id: String,
    pub command: String,
    pub emits: String,
    pub artifact_type: Option<String>,
    pub evidence_refs: Vec<String>,
    pub standards_refs: Vec<String>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<ProcessAnomalyType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessAnomalyType {
    SkippedStep,
    OutOfOrder,
    DuplicateExecution,
    UnauthorizedActor,
    LateExecution,
    MissingArtifact,
    MissingEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAnomalyRecord {
    pub anomaly_id: Uuid,
    pub anomaly_type: ProcessAnomalyType,
    pub severity: AnomalySeverity,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub timestamp: NaiveDateTime,
    pub description: String,
}

// ===== Builder =====

pub struct ProcessEventBuilder {
    case_id: String,
    event_type: String,
    procedure_id: String,
    step_id: Option<String>,
    phase_id: String,
    from_state: Option<String>,
    to_state: Option<String>,
    actor_id: String,
    command: String,
    emits: String,
    artifact_type: Option<String>,
    evidence_refs: Vec<String>,
    standards_refs: Vec<String>,
    timestamp: Option<NaiveDateTime>,
    is_anomaly: bool,
    anomaly_type: Option<ProcessAnomalyType>,
}

impl ProcessEventBuilder {
    pub fn transition() -> Self {
        Self::new("transition")
    }

    pub fn step() -> Self {
        Self::new("step")
    }

    fn new(event_type: &str) -> Self {
        Self {
            case_id: String::new(),
            event_type: event_type.to_string(),
            procedure_id: String::new(),
            step_id: None,
            phase_id: String::new(),
            from_state: None,
            to_state: None,
            actor_id: String::new(),
            command: String::new(),
            emits: String::new(),
            artifact_type: None,
            evidence_refs: Vec::new(),
            standards_refs: Vec::new(),
            timestamp: None,
            is_anomaly: false,
            anomaly_type: None,
        }
    }

    pub fn case_id(mut self, val: impl Into<String>) -> Self { self.case_id = val.into(); self }
    pub fn procedure_id(mut self, val: impl Into<String>) -> Self { self.procedure_id = val.into(); self }
    pub fn step_id(mut self, val: impl Into<String>) -> Self { self.step_id = Some(val.into()); self }
    pub fn phase_id(mut self, val: impl Into<String>) -> Self { self.phase_id = val.into(); self }
    pub fn from_state(mut self, val: impl Into<String>) -> Self { self.from_state = Some(val.into()); self }
    pub fn to_state(mut self, val: impl Into<String>) -> Self { self.to_state = Some(val.into()); self }
    pub fn actor_id(mut self, val: impl Into<String>) -> Self { self.actor_id = val.into(); self }
    pub fn command(mut self, val: impl Into<String>) -> Self { self.command = val.into(); self }
    pub fn emits(mut self, val: impl Into<String>) -> Self { self.emits = val.into(); self }
    pub fn artifact_type(mut self, val: impl Into<String>) -> Self { self.artifact_type = Some(val.into()); self }
    pub fn evidence_ref(mut self, val: impl Into<String>) -> Self { self.evidence_refs.push(val.into()); self }
    pub fn standard_ref(mut self, val: impl Into<String>) -> Self { self.standards_refs.push(val.into()); self }
    pub fn timestamp(mut self, val: NaiveDateTime) -> Self { self.timestamp = Some(val); self }

    pub fn anomaly(mut self, anomaly_type: ProcessAnomalyType) -> Self {
        self.is_anomaly = true;
        self.anomaly_type = Some(anomaly_type);
        self
    }

    pub fn build(self) -> ProcessEvent {
        let timestamp = self.timestamp.unwrap_or_else(|| {
            chrono::Utc::now().naive_utc()
        });
        ProcessEvent {
            event_id: Uuid::new_v4(),
            case_id: self.case_id,
            timestamp,
            event_type: self.event_type,
            procedure_id: self.procedure_id,
            step_id: self.step_id,
            phase_id: self.phase_id,
            from_state: self.from_state,
            to_state: self.to_state,
            actor_id: self.actor_id,
            command: self.command,
            emits: self.emits,
            artifact_type: self.artifact_type,
            evidence_refs: self.evidence_refs,
            standards_refs: self.standards_refs,
            is_anomaly: self.is_anomaly,
            anomaly_type: self.anomaly_type,
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p datasynth-process-engine --test event_tests -- --test-threads=4`
Expected: All 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/src/event.rs crates/datasynth-process-engine/tests/event_tests.rs
git commit -m "feat(process-engine): define ProcessEvent with builder pattern"
```

---

### Task 6: Define GeneratedRecord and StepContext

**Files:**
- Modify: `crates/datasynth-process-engine/src/record.rs`
- Modify: `crates/datasynth-process-engine/src/context.rs`

- [ ] **Step 1: Implement record types**

Write `src/record.rs`:

```rust
use chrono::{NaiveDate, NaiveDateTime};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRecord {
    pub artifact_type: String,
    pub record_id: String,
    pub fields: IndexMap<String, RecordValue>,
    pub metadata: RecordMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordValue {
    String(String),
    Integer(i64),
    Decimal(Decimal),
    Boolean(bool),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Null,
    Array(Vec<RecordValue>),
    Object(IndexMap<String, RecordValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub case_id: String,
    pub step_id: String,
    pub procedure_id: String,
    pub phase_id: String,
    pub timestamp: NaiveDateTime,
    pub actor_id: String,
    pub blueprint_id: String,
}

impl GeneratedRecord {
    /// Create a minimal event-only record (no artifact fields, just metadata).
    pub fn event_only(case_id: &str, step_id: &str, procedure_id: &str, phase_id: &str, actor_id: &str, blueprint_id: &str, timestamp: NaiveDateTime) -> Self {
        Self {
            artifact_type: "event".to_string(),
            record_id: uuid::Uuid::new_v4().to_string(),
            fields: IndexMap::new(),
            metadata: RecordMetadata {
                case_id: case_id.to_string(),
                step_id: step_id.to_string(),
                procedure_id: procedure_id.to_string(),
                phase_id: phase_id.to_string(),
                timestamp,
                actor_id: actor_id.to_string(),
                blueprint_id: blueprint_id.to_string(),
            },
        }
    }
}

/// Trait for converting domain structs to GeneratedRecord.
pub trait ToGeneratedRecord {
    fn to_generated_record(&self, metadata: RecordMetadata) -> GeneratedRecord;
}
```

- [ ] **Step 2: Implement context types**

Write `src/context.rs`:

```rust
use chrono::NaiveDateTime;
use std::collections::HashMap;

use crate::record::GeneratedRecord;

/// Context passed to artifact generators during step execution.
#[derive(Debug, Clone)]
pub struct StepContext {
    // Process identity
    pub case_id: String,
    pub blueprint_id: String,
    pub phase_id: String,
    pub procedure_id: String,
    pub step_id: String,

    // Temporal
    pub timestamp: NaiveDateTime,

    // Actor
    pub actor_id: String,
    pub actor_role: String,

    // Organizational
    pub company_code: String,
    pub entity_code: Option<String>,
    pub department: Option<String>,
    pub currency: String,

    // Process state
    pub procedure_state: String,
    pub iteration: u32,
    pub prior_artifacts: Vec<ArtifactRef>,

    // Domain-specific context (opaque to engine, populated by domain layers)
    pub domain_context: HashMap<String, serde_json::Value>,
}

/// Reference to a previously generated artifact (for chaining).
#[derive(Debug, Clone)]
pub struct ArtifactRef {
    pub artifact_type: String,
    pub record_id: String,
    pub step_id: String,
}

impl ArtifactRef {
    pub fn from_record(record: &GeneratedRecord) -> Self {
        Self {
            artifact_type: record.artifact_type.clone(),
            record_id: record.record_id.clone(),
            step_id: record.metadata.step_id.clone(),
        }
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p datasynth-process-engine`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-process-engine/src/record.rs crates/datasynth-process-engine/src/context.rs
git commit -m "feat(process-engine): define GeneratedRecord, RecordValue, StepContext"
```

---

### Task 7: Define ArtifactGenerator trait and ArtifactRegistry

**Files:**
- Modify: `crates/datasynth-process-engine/src/registry.rs`
- Create: `crates/datasynth-process-engine/tests/registry_tests.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/registry_tests.rs`:

```rust
use datasynth_process_engine::artifact_schema::ArtifactSchema;
use datasynth_process_engine::context::StepContext;
use datasynth_process_engine::error::ProcessEngineError;
use datasynth_process_engine::record::{GeneratedRecord, RecordMetadata, RecordValue};
use datasynth_process_engine::registry::*;
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::collections::HashMap;

struct MockPOGenerator;

impl ArtifactGenerator for MockPOGenerator {
    fn artifact_type(&self) -> &str { "purchase_order" }

    fn generate(
        &self,
        context: &StepContext,
        _schema: Option<&ArtifactSchema>,
        _rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, ProcessEngineError> {
        let mut fields = IndexMap::new();
        fields.insert("po_number".to_string(), RecordValue::String("PO-001".to_string()));
        fields.insert("amount".to_string(), RecordValue::Decimal(rust_decimal::Decimal::new(10000, 2)));
        Ok(vec![GeneratedRecord {
            artifact_type: "purchase_order".to_string(),
            record_id: "test-record-id".to_string(),
            fields,
            metadata: RecordMetadata {
                case_id: context.case_id.clone(),
                step_id: context.step_id.clone(),
                procedure_id: context.procedure_id.clone(),
                phase_id: context.phase_id.clone(),
                timestamp: context.timestamp,
                actor_id: context.actor_id.clone(),
                blueprint_id: context.blueprint_id.clone(),
            },
        }])
    }

    fn output_fields(&self) -> Vec<FieldDescriptor> {
        vec![
            FieldDescriptor { name: "po_number".to_string(), field_type: "string".to_string() },
            FieldDescriptor { name: "amount".to_string(), field_type: "decimal".to_string() },
        ]
    }
}

fn test_context() -> StepContext {
    StepContext {
        case_id: "CASE-001".to_string(),
        blueprint_id: "test".to_string(),
        phase_id: "phase1".to_string(),
        procedure_id: "proc1".to_string(),
        step_id: "step1".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2024-06-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        actor_id: "actor1".to_string(),
        actor_role: "buyer".to_string(),
        company_code: "C001".to_string(),
        entity_code: None,
        department: None,
        currency: "USD".to_string(),
        procedure_state: "in_progress".to_string(),
        iteration: 0,
        prior_artifacts: vec![],
        domain_context: HashMap::new(),
    }
}

#[test]
fn test_registry_dispatches_to_registered_generator() {
    let mut registry = ArtifactRegistry::new();
    registry.register(Box::new(MockPOGenerator));

    let ctx = test_context();
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let records = registry.generate("purchase_order", &ctx, None, &mut rng).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].artifact_type, "purchase_order");
    assert!(records[0].fields.contains_key("po_number"));
}

#[test]
fn test_registry_returns_event_only_for_unknown_type_without_schema() {
    let registry = ArtifactRegistry::new();
    let ctx = test_context();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let records = registry.generate("unknown_type", &ctx, None, &mut rng).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].artifact_type, "event");
    assert!(records[0].fields.is_empty());
}

#[test]
fn test_registry_has_generator() {
    let mut registry = ArtifactRegistry::new();
    assert!(!registry.has_generator("purchase_order"));
    registry.register(Box::new(MockPOGenerator));
    assert!(registry.has_generator("purchase_order"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p datasynth-process-engine --test registry_tests -- --test-threads=4`
Expected: Compilation error

- [ ] **Step 3: Implement registry**

Write `src/registry.rs`:

```rust
use std::collections::HashMap;

use rand_chacha::ChaCha8Rng;

use crate::artifact_schema::ArtifactSchema;
use crate::context::StepContext;
use crate::error::ProcessEngineError;
use crate::record::GeneratedRecord;
use crate::schema_generator::SchemaGenerator;

/// Trait for domain-specific artifact generators.
pub trait ArtifactGenerator: Send + Sync {
    /// The artifact type name this generator handles (e.g., "purchase_order").
    fn artifact_type(&self) -> &str;

    /// Generate one or more records for a step execution.
    fn generate(
        &self,
        context: &StepContext,
        schema: Option<&ArtifactSchema>,
        rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, ProcessEngineError>;

    /// Describe the fields this generator produces (for introspection/validation).
    fn output_fields(&self) -> Vec<FieldDescriptor>;
}

/// Describes a field produced by a generator (for introspection).
#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub name: String,
    pub field_type: String,
}

/// Registry that resolves artifact_type to a generator.
pub struct ArtifactRegistry {
    generators: HashMap<String, Box<dyn ArtifactGenerator>>,
    schema_generator: SchemaGenerator,
}

impl ArtifactRegistry {
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
            schema_generator: SchemaGenerator::new(),
        }
    }

    /// Register a domain-specific generator.
    pub fn register(&mut self, generator: Box<dyn ArtifactGenerator>) {
        let key = generator.artifact_type().to_string();
        self.generators.insert(key, generator);
    }

    /// Check if a generator is registered for the given type.
    pub fn has_generator(&self, artifact_type: &str) -> bool {
        self.generators.contains_key(artifact_type)
    }

    /// List registered artifact types.
    pub fn registered_types(&self) -> Vec<&str> {
        self.generators.keys().map(|s| s.as_str()).collect()
    }

    /// Resolve and generate — falls back to SchemaGenerator if no match.
    pub fn generate(
        &self,
        artifact_type: &str,
        context: &StepContext,
        schema: Option<&ArtifactSchema>,
        rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, ProcessEngineError> {
        if let Some(gen) = self.generators.get(artifact_type) {
            gen.generate(context, schema, rng)
        } else if let Some(schema) = schema {
            self.schema_generator.generate(context, schema, rng)
        } else {
            // No generator and no schema — emit a generic event record
            Ok(vec![GeneratedRecord::event_only(
                &context.case_id,
                &context.step_id,
                &context.procedure_id,
                &context.phase_id,
                &context.actor_id,
                &context.blueprint_id,
                context.timestamp,
            )])
        }
    }
}

impl Default for ArtifactRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Add SchemaGenerator stub** (so registry compiles)

Write `src/schema_generator.rs`:

```rust
use rand_chacha::ChaCha8Rng;

use crate::artifact_schema::ArtifactSchema;
use crate::context::StepContext;
use crate::error::ProcessEngineError;
use crate::record::GeneratedRecord;

/// Fallback generator that produces records from artifact schema definitions.
pub struct SchemaGenerator;

impl SchemaGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate records from an artifact schema using field definitions and distributions.
    pub fn generate(
        &self,
        context: &StepContext,
        schema: &ArtifactSchema,
        _rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, ProcessEngineError> {
        // Stub — will be fully implemented in Task 12
        Ok(vec![GeneratedRecord::event_only(
            &context.case_id,
            &context.step_id,
            &context.procedure_id,
            &context.phase_id,
            &context.actor_id,
            &context.blueprint_id,
            context.timestamp,
        )])
    }
}

impl Default for SchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p datasynth-process-engine --test registry_tests -- --test-threads=4`
Expected: All 3 tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-process-engine/src/registry.rs crates/datasynth-process-engine/src/schema_generator.rs crates/datasynth-process-engine/tests/registry_tests.rs
git commit -m "feat(process-engine): implement ArtifactGenerator trait and ArtifactRegistry"
```

---

### Task 8: Define GenerationOverlay types

**Files:**
- Modify: `crates/datasynth-process-engine/src/overlay.rs`
- Modify: `crates/datasynth-process-engine/tests/schema_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/schema_tests.rs`:

```rust
use datasynth_process_engine::overlay::*;

#[test]
fn test_overlay_parses_from_yaml() {
    let yaml = r#"
overlay_version: "2.0"

variants:
  - id: happy_path
    weight: 0.70
    description: "Standard flow"
  - id: with_rework
    weight: 0.30
    description: "Revision loop"
    force_transitions:
      - procedure: quality_check
        transition: reject_to_rework

transitions:
  defaults:
    revision_probability: 0.15
    timing:
      mu_hours: 24.0
      sigma_hours: 8.0
  per_procedure:
    approve_po:
      revision_probability: 0.05
      timing:
        mu_hours: 4.0
        sigma_hours: 2.0

anomalies:
  skipped_step: 0.02
  out_of_order: 0.01
  unauthorized_actor: 0.01

actor_profiles:
  approver:
    availability_hours: [8, 17]
    concurrent_cases: 15

volume:
  cases_per_period: 1000
  period: month

iteration_limits:
  default: 50
"#;
    let overlay: GenerationOverlay = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(overlay.variants.len(), 2);
    assert_eq!(overlay.variants[0].weight, 0.70);
    assert_eq!(overlay.transitions.defaults.revision_probability, 0.15);
    assert_eq!(overlay.transitions.defaults.timing.mu_hours, 24.0);
    assert!(overlay.transitions.per_procedure.contains_key("approve_po"));
    assert_eq!(overlay.anomalies.skipped_step, 0.02);
    assert_eq!(overlay.volume.cases_per_period, 1000);
    assert_eq!(overlay.iteration_limits.default, 50);
}

#[test]
fn test_overlay_defaults() {
    let overlay = GenerationOverlay::default();
    assert_eq!(overlay.transitions.defaults.revision_probability, 0.15);
    assert_eq!(overlay.transitions.defaults.timing.mu_hours, 24.0);
    assert_eq!(overlay.iteration_limits.default, 50);
    assert!(overlay.variants.is_empty());
}
```

- [ ] **Step 2: Implement overlay types**

Write `src/overlay.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOverlay {
    #[serde(default = "default_overlay_version")]
    pub overlay_version: String,

    #[serde(default)]
    pub variants: Vec<ProcessVariant>,

    #[serde(default)]
    pub transitions: TransitionConfig,

    #[serde(default)]
    pub anomalies: AnomalyConfig,

    #[serde(default)]
    pub actor_profiles: HashMap<String, ActorProfile>,

    #[serde(default)]
    pub discriminators: Option<HashMap<String, Vec<String>>>,

    #[serde(default)]
    pub volume: VolumeConfig,

    #[serde(default)]
    pub iteration_limits: IterationLimits,

    #[serde(default = "default_max_self_loop")]
    pub max_self_loop_iterations: usize,
}

fn default_overlay_version() -> String { "2.0".to_string() }
fn default_max_self_loop() -> usize { 5 }

impl Default for GenerationOverlay {
    fn default() -> Self {
        Self {
            overlay_version: default_overlay_version(),
            variants: Vec::new(),
            transitions: TransitionConfig::default(),
            anomalies: AnomalyConfig::default(),
            actor_profiles: HashMap::new(),
            discriminators: None,
            volume: VolumeConfig::default(),
            iteration_limits: IterationLimits::default(),
            max_self_loop_iterations: 5,
        }
    }
}

// ===== Variants =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessVariant {
    pub id: String,
    pub weight: f64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub skip_procedures: Vec<String>,
    #[serde(default)]
    pub force_transitions: Vec<ForcedTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForcedTransition {
    pub procedure: String,
    pub transition: String,
}

// ===== Transitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionConfig {
    #[serde(default)]
    pub defaults: TransitionDefaults,
    #[serde(default)]
    pub per_procedure: HashMap<String, TransitionOverride>,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            defaults: TransitionDefaults::default(),
            per_procedure: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefaults {
    #[serde(default = "default_revision_prob")]
    pub revision_probability: f64,
    #[serde(default)]
    pub timing: TimingDistribution,
}

fn default_revision_prob() -> f64 { 0.15 }

impl Default for TransitionDefaults {
    fn default() -> Self {
        Self {
            revision_probability: 0.15,
            timing: TimingDistribution::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionOverride {
    #[serde(default)]
    pub revision_probability: Option<f64>,
    #[serde(default)]
    pub timing: Option<TimingDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingDistribution {
    #[serde(default = "default_mu_hours")]
    pub mu_hours: f64,
    #[serde(default = "default_sigma_hours")]
    pub sigma_hours: f64,
    #[serde(default = "default_timing_dist_type")]
    pub distribution: String,
}

fn default_mu_hours() -> f64 { 24.0 }
fn default_sigma_hours() -> f64 { 8.0 }
fn default_timing_dist_type() -> String { "lognormal".to_string() }

impl Default for TimingDistribution {
    fn default() -> Self {
        Self {
            mu_hours: 24.0,
            sigma_hours: 8.0,
            distribution: "lognormal".to_string(),
        }
    }
}

// ===== Anomalies =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    #[serde(default)]
    pub skipped_step: f64,
    #[serde(default)]
    pub out_of_order: f64,
    #[serde(default)]
    pub duplicate_execution: f64,
    #[serde(default)]
    pub unauthorized_actor: f64,
    #[serde(default)]
    pub late_execution: f64,
    #[serde(default)]
    pub missing_artifact: f64,
    #[serde(default)]
    pub missing_evidence: f64,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            skipped_step: 0.0,
            out_of_order: 0.0,
            duplicate_execution: 0.0,
            unauthorized_actor: 0.0,
            late_execution: 0.0,
            missing_artifact: 0.0,
            missing_evidence: 0.0,
        }
    }
}

// ===== Actor Profiles =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorProfile {
    #[serde(default)]
    pub availability_hours: Vec<u32>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub concurrent_cases: Option<u32>,
    #[serde(default)]
    pub batch_processing: bool,
    #[serde(default)]
    pub batch_size: Option<BatchSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSize {
    pub min: u32,
    pub max: u32,
}

// ===== Volume =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    #[serde(default = "default_cases")]
    pub cases_per_period: u32,
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default)]
    pub seasonality: Option<SeasonalityConfig>,
}

fn default_cases() -> u32 { 100 }
fn default_period() -> String { "month".to_string() }

impl Default for VolumeConfig {
    fn default() -> Self {
        Self {
            cases_per_period: 100,
            period: "month".to_string(),
            seasonality: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityConfig {
    pub enabled: bool,
    #[serde(default)]
    pub pattern: Vec<f64>,
}

// ===== Iteration Limits =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationLimits {
    #[serde(default = "default_iteration_limit")]
    pub default: u32,
    #[serde(default)]
    pub per_procedure: HashMap<String, u32>,
}

fn default_iteration_limit() -> u32 { 50 }

impl Default for IterationLimits {
    fn default() -> Self {
        Self {
            default: 50,
            per_procedure: HashMap::new(),
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p datasynth-process-engine --test schema_tests -- --test-threads=4`
Expected: All tests pass including the overlay tests

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-process-engine/src/overlay.rs crates/datasynth-process-engine/tests/schema_tests.rs
git commit -m "feat(process-engine): define GenerationOverlay with variants, timing, anomalies"
```

---

### Task 9: Implement blueprint loader and validator

**Files:**
- Modify: `crates/datasynth-process-engine/src/loader.rs`
- Create: `crates/datasynth-process-engine/tests/loader_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/loader_tests.rs`:

```rust
use datasynth_process_engine::loader::*;
use datasynth_process_engine::error::ProcessEngineError;

#[test]
fn test_parse_valid_blueprint() {
    let yaml = r#"
id: test
name: "Test"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "Phase 1"
    order: 1
    procedures:
      - id: proc_a
        title: "Procedure A"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start, emits: Started }
            - { from_state: in_progress, to_state: completed, command: finish, emits: Finished }
        steps:
          - { id: s1, order: 1, action: do, actor: user, command: do_thing, emits: ThingDone }
        preconditions: []
"#;
    let bp = parse_blueprint(yaml).unwrap();
    assert_eq!(bp.id, "test");
}

#[test]
fn test_validate_detects_dag_cycle() {
    let yaml = r#"
id: cycle_test
name: "Cycle"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: proc_a
        title: "A"
        aggregate:
          initial_state: not_started
          states: [not_started, completed]
          transitions:
            - { from_state: not_started, to_state: completed, command: go, emits: Done }
        steps: []
        preconditions: [proc_b]
      - id: proc_b
        title: "B"
        aggregate:
          initial_state: not_started
          states: [not_started, completed]
          transitions:
            - { from_state: not_started, to_state: completed, command: go, emits: Done }
        steps: []
        preconditions: [proc_a]
"#;
    let bp = parse_blueprint(yaml).unwrap();
    let result = validate_blueprint(&bp);
    assert!(result.is_err());
    match result.unwrap_err() {
        ProcessEngineError::DagCycle { .. } => {}
        other => panic!("Expected DagCycle, got: {:?}", other),
    }
}

#[test]
fn test_validate_detects_missing_actor_ref() {
    let yaml = r#"
id: actor_test
name: "Actor"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
actors:
  - { id: buyer, name: "Buyer" }
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: proc_a
        title: "A"
        aggregate:
          initial_state: not_started
          states: [not_started, completed]
          transitions:
            - { from_state: not_started, to_state: completed, command: go, emits: Done }
        steps:
          - { id: s1, order: 1, action: do, actor: nonexistent_actor, command: go, emits: Done }
        preconditions: []
"#;
    let bp = parse_blueprint(yaml).unwrap();
    let result = validate_blueprint(&bp);
    assert!(result.is_err());
}

#[test]
fn test_validate_detects_unreachable_state() {
    let yaml = r#"
id: state_test
name: "State"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: proc_a
        title: "A"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed, orphan_state]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start, emits: Started }
            - { from_state: in_progress, to_state: completed, command: finish, emits: Finished }
        steps: []
        preconditions: []
"#;
    let bp = parse_blueprint(yaml).unwrap();
    let result = validate_blueprint(&bp);
    assert!(result.is_err());
}

#[test]
fn test_topological_sort() {
    let yaml = r#"
id: topo_test
name: "Topo"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: proc_c
        title: "C"
        aggregate: { initial_state: ns, states: [ns, done], transitions: [{ from_state: ns, to_state: done, command: go, emits: Done }] }
        steps: []
        preconditions: [proc_a, proc_b]
      - id: proc_a
        title: "A"
        aggregate: { initial_state: ns, states: [ns, done], transitions: [{ from_state: ns, to_state: done, command: go, emits: Done }] }
        steps: []
        preconditions: []
      - id: proc_b
        title: "B"
        aggregate: { initial_state: ns, states: [ns, done], transitions: [{ from_state: ns, to_state: done, command: go, emits: Done }] }
        steps: []
        preconditions: [proc_a]
"#;
    let bp = parse_blueprint(yaml).unwrap();
    let sorted = topological_sort_procedures(&bp).unwrap();
    let pos_a = sorted.iter().position(|x| x == "proc_a").unwrap();
    let pos_b = sorted.iter().position(|x| x == "proc_b").unwrap();
    let pos_c = sorted.iter().position(|x| x == "proc_c").unwrap();
    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);
}

#[test]
fn test_blueprint_with_preconditions_load_and_validate() {
    let yaml = r#"
id: bwp_test
name: "BWP"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: proc_a
        title: "A"
        aggregate: { initial_state: ns, states: [ns, done], transitions: [{ from_state: ns, to_state: done, command: go, emits: Done }] }
        steps: []
        preconditions: []
      - id: proc_b
        title: "B"
        aggregate: { initial_state: ns, states: [ns, done], transitions: [{ from_state: ns, to_state: done, command: go, emits: Done }] }
        steps: []
        preconditions: [proc_a]
"#;
    let bwp = BlueprintWithPreconditions::from_yaml(yaml).unwrap();
    bwp.validate().unwrap();
    let sorted = bwp.topological_sort().unwrap();
    assert_eq!(sorted[0], "proc_a");
    assert_eq!(sorted[1], "proc_b");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-process-engine --test loader_tests -- --test-threads=4 2>&1 | head -10`
Expected: Compilation error

- [ ] **Step 3: Implement loader**

Write `src/loader.rs`:

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::error::{ProcessEngineError, ValidationViolation};
use crate::overlay::GenerationOverlay;
use crate::schema::ProcessBlueprint;

// ===== Source types =====

#[derive(Debug, Clone)]
pub enum BlueprintSource {
    Custom(PathBuf),
    Raw(String),
}

#[derive(Debug, Clone)]
pub enum OverlaySource {
    Custom(PathBuf),
    Raw(String),
}

// ===== Parsing =====

pub fn parse_blueprint(yaml: &str) -> Result<ProcessBlueprint, ProcessEngineError> {
    serde_yaml::from_str(yaml).map_err(|e| ProcessEngineError::BlueprintParse {
        path: "<raw>".to_string(),
        source: e,
    })
}

pub fn parse_overlay(yaml: &str) -> Result<GenerationOverlay, ProcessEngineError> {
    serde_yaml::from_str(yaml).map_err(|e| ProcessEngineError::OverlayParse {
        path: "<raw>".to_string(),
        source: e,
    })
}

pub fn load_blueprint(source: &BlueprintSource) -> Result<ProcessBlueprint, ProcessEngineError> {
    match source {
        BlueprintSource::Custom(path) => {
            let yaml = std::fs::read_to_string(path).map_err(ProcessEngineError::Io)?;
            serde_yaml::from_str(&yaml).map_err(|e| ProcessEngineError::BlueprintParse {
                path: path.display().to_string(),
                source: e,
            })
        }
        BlueprintSource::Raw(yaml) => parse_blueprint(yaml),
    }
}

pub fn load_overlay(source: &OverlaySource) -> Result<GenerationOverlay, ProcessEngineError> {
    match source {
        OverlaySource::Custom(path) => {
            let yaml = std::fs::read_to_string(path).map_err(ProcessEngineError::Io)?;
            serde_yaml::from_str(&yaml).map_err(|e| ProcessEngineError::OverlayParse {
                path: path.display().to_string(),
                source: e,
            })
        }
        OverlaySource::Raw(yaml) => parse_overlay(yaml),
    }
}

// ===== Validation =====

pub fn validate_blueprint(bp: &ProcessBlueprint) -> Result<(), ProcessEngineError> {
    let mut violations = Vec::new();

    // Collect all procedure IDs
    let mut all_procedures: HashMap<&str, &crate::schema::ProcessProcedure> = HashMap::new();
    for phase in &bp.phases {
        for proc in &phase.procedures {
            if all_procedures.contains_key(proc.id.as_str()) {
                violations.push(ValidationViolation::new(
                    format!("procedure.{}", proc.id),
                    "Duplicate procedure ID",
                ));
            }
            all_procedures.insert(&proc.id, proc);
        }
    }

    // Validate actor references
    let actor_ids: HashSet<&str> = bp.actors.iter().map(|a| a.id.as_str()).collect();
    if !bp.actors.is_empty() {
        for phase in &bp.phases {
            for proc in &phase.procedures {
                for step in &proc.steps {
                    if !step.actor.is_empty() && !actor_ids.contains(step.actor.as_str()) {
                        violations.push(ValidationViolation::new(
                            format!("procedure.{}.step.{}", proc.id, step.id),
                            format!("Actor '{}' not declared in actors list", step.actor),
                        ));
                    }
                }
            }
        }
    }

    // Validate state machine reachability
    for phase in &bp.phases {
        for proc in &phase.procedures {
            let agg = &proc.aggregate;
            if agg.states.is_empty() {
                continue;
            }

            // Check initial state is declared
            if !agg.states.contains(&agg.initial_state) {
                violations.push(ValidationViolation::new(
                    format!("procedure.{}.aggregate", proc.id),
                    format!("Initial state '{}' not in states list", agg.initial_state),
                ));
            }

            // Check transition states are declared
            for tr in &agg.transitions {
                if !agg.states.contains(&tr.from_state) {
                    violations.push(ValidationViolation::new(
                        format!("procedure.{}.transition", proc.id),
                        format!("from_state '{}' not in states list", tr.from_state),
                    ));
                }
                if !agg.states.contains(&tr.to_state) {
                    violations.push(ValidationViolation::new(
                        format!("procedure.{}.transition", proc.id),
                        format!("to_state '{}' not in states list", tr.to_state),
                    ));
                }
            }

            // Check state reachability from initial state (BFS)
            let mut reachable: HashSet<&str> = HashSet::new();
            let mut queue: VecDeque<&str> = VecDeque::new();
            queue.push_back(&agg.initial_state);
            reachable.insert(&agg.initial_state);
            while let Some(state) = queue.pop_front() {
                for tr in &agg.transitions {
                    if tr.from_state == state && !reachable.contains(tr.to_state.as_str()) {
                        reachable.insert(&tr.to_state);
                        queue.push_back(&tr.to_state);
                    }
                }
            }
            for state in &agg.states {
                if !reachable.contains(state.as_str()) {
                    violations.push(ValidationViolation::new(
                        format!("procedure.{}.aggregate", proc.id),
                        format!("State '{}' is unreachable from initial state '{}'", state, agg.initial_state),
                    ));
                }
            }
        }
    }

    // Validate preconditions reference existing procedures
    for phase in &bp.phases {
        for proc in &phase.procedures {
            for pre in &proc.preconditions {
                if !all_procedures.contains_key(pre.as_str()) {
                    violations.push(ValidationViolation::new(
                        format!("procedure.{}.preconditions", proc.id),
                        format!("Precondition '{}' references non-existent procedure", pre),
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        return Err(ProcessEngineError::BlueprintValidation { violations });
    }

    // DAG cycle detection
    topological_sort_procedures(bp)?;

    Ok(())
}

// ===== Topological Sort =====

pub fn topological_sort_procedures(bp: &ProcessBlueprint) -> Result<Vec<String>, ProcessEngineError> {
    // Collect all procedures with their preconditions
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

    for phase in &bp.phases {
        for proc in &phase.procedures {
            in_degree.entry(proc.id.clone()).or_insert(0);
            for pre in &proc.preconditions {
                *in_degree.entry(proc.id.clone()).or_insert(0) += 1;
                dependents.entry(pre.clone()).or_default().push(proc.id.clone());
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    // Stable sort: process in alphabetical order when multiple have in-degree 0
    let mut sorted_queue: Vec<String> = queue.drain(..).collect();
    sorted_queue.sort();
    queue.extend(sorted_queue);

    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        result.push(id.clone());
        if let Some(deps) = dependents.get(&id) {
            let mut ready = Vec::new();
            for dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        ready.push(dep.clone());
                    }
                }
            }
            ready.sort();
            queue.extend(ready);
        }
    }

    if result.len() != in_degree.len() {
        let remaining: Vec<String> = in_degree
            .iter()
            .filter(|(id, _)| !result.contains(id))
            .map(|(id, _)| id.clone())
            .collect();
        return Err(ProcessEngineError::DagCycle { procedures: remaining });
    }

    Ok(result)
}

// ===== BlueprintWithPreconditions =====

pub struct BlueprintWithPreconditions {
    pub blueprint: ProcessBlueprint,
    pub preconditions: HashMap<String, Vec<String>>,
}

impl BlueprintWithPreconditions {
    pub fn from_yaml(yaml: &str) -> Result<Self, ProcessEngineError> {
        let blueprint = parse_blueprint(yaml)?;
        let preconditions = Self::extract_preconditions(&blueprint);
        Ok(Self { blueprint, preconditions })
    }

    pub fn from_blueprint(blueprint: ProcessBlueprint) -> Self {
        let preconditions = Self::extract_preconditions(&blueprint);
        Self { blueprint, preconditions }
    }

    pub fn load(source: &BlueprintSource) -> Result<Self, ProcessEngineError> {
        let blueprint = load_blueprint(source)?;
        let preconditions = Self::extract_preconditions(&blueprint);
        Ok(Self { blueprint, preconditions })
    }

    pub fn validate(&self) -> Result<(), ProcessEngineError> {
        validate_blueprint(&self.blueprint)
    }

    pub fn topological_sort(&self) -> Result<Vec<String>, ProcessEngineError> {
        topological_sort_procedures(&self.blueprint)
    }

    fn extract_preconditions(bp: &ProcessBlueprint) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        for phase in &bp.phases {
            for proc in &phase.procedures {
                map.insert(proc.id.clone(), proc.preconditions.clone());
            }
        }
        map
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p datasynth-process-engine --test loader_tests -- --test-threads=4`
Expected: All 6 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/src/loader.rs crates/datasynth-process-engine/tests/loader_tests.rs
git commit -m "feat(process-engine): implement blueprint loader with DAG, reachability, cross-ref validation"
```

---

### Task 10: Implement FSM engine

**Files:**
- Modify: `crates/datasynth-process-engine/src/engine.rs`
- Create: `crates/datasynth-process-engine/tests/engine_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/engine_tests.rs`:

```rust
use datasynth_process_engine::engine::*;
use datasynth_process_engine::loader::*;
use datasynth_process_engine::overlay::GenerationOverlay;
use datasynth_process_engine::registry::ArtifactRegistry;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn simple_blueprint_yaml() -> &'static str {
    r#"
id: simple_test
name: "Simple Test"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "Phase 1"
    order: 1
    procedures:
      - id: step_one
        title: "Step One"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: begin, emits: Begun }
            - { from_state: in_progress, to_state: completed, command: finish, emits: Finished }
        steps:
          - { id: s1, order: 1, action: execute, actor: worker, command: do_work, emits: WorkDone }
        preconditions: []
      - id: step_two
        title: "Step Two"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: begin, emits: Begun }
            - { from_state: in_progress, to_state: completed, command: finish, emits: Finished }
        steps:
          - { id: s2, order: 1, action: review, actor: worker, command: review_work, emits: ReviewDone }
        preconditions: [step_one]
"#
}

#[test]
fn test_engine_produces_events_for_simple_blueprint() {
    let bwp = BlueprintWithPreconditions::from_yaml(simple_blueprint_yaml()).unwrap();
    bwp.validate().unwrap();
    let overlay = GenerationOverlay::default();
    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(42);

    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
    let config = EngineConfig {
        company_code: "C001".to_string(),
        currency: "USD".to_string(),
        case_id: "CASE-001".to_string(),
        start_time: chrono::NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        domain_context: Default::default(),
    };
    let result = engine.run_case(&config).unwrap();

    // Should have transition events + step events for both procedures
    assert!(!result.event_log.is_empty());
    // step_one should complete before step_two (precondition ordering)
    let proc_ids: Vec<&str> = result.event_log.iter()
        .filter(|e| e.event_type == "step")
        .map(|e| e.procedure_id.as_str())
        .collect();
    let first_step_one = proc_ids.iter().position(|&p| p == "step_one");
    let first_step_two = proc_ids.iter().position(|&p| p == "step_two");
    assert!(first_step_one.unwrap() < first_step_two.unwrap());

    // All procedures should reach completed
    assert_eq!(result.procedure_states.get("step_one").map(String::as_str), Some("completed"));
    assert_eq!(result.procedure_states.get("step_two").map(String::as_str), Some("completed"));
}

#[test]
fn test_engine_is_deterministic() {
    let yaml = simple_blueprint_yaml();

    let run = |seed: u64| -> Vec<String> {
        let bwp = BlueprintWithPreconditions::from_yaml(yaml).unwrap();
        let overlay = GenerationOverlay::default();
        let registry = ArtifactRegistry::new();
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
        let config = EngineConfig {
            company_code: "C001".to_string(),
            currency: "USD".to_string(),
            case_id: "CASE-001".to_string(),
            start_time: chrono::NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            domain_context: Default::default(),
        };
        let result = engine.run_case(&config).unwrap();
        result.event_log.iter().map(|e| format!("{}:{}:{}", e.procedure_id, e.command, e.event_type)).collect()
    };

    assert_eq!(run(42), run(42));
    assert_ne!(run(42), run(99));
}

#[test]
fn test_engine_with_revision_loop() {
    let yaml = r#"
id: revision_test
name: "Revision Test"
version: "1.0"
schema_version: "2.0"
domain: custom
depth: standard
phases:
  - id: phase1
    name: "P1"
    order: 1
    procedures:
      - id: review_proc
        title: "Review"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, under_review, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: begin, emits: Begun }
            - { from_state: in_progress, to_state: under_review, command: submit, emits: Submitted }
            - { from_state: under_review, to_state: in_progress, command: revise, emits: RevisionRequested }
            - { from_state: under_review, to_state: completed, command: approve, emits: Approved }
        steps:
          - { id: s1, order: 1, action: do, actor: worker, command: do_work, emits: Done }
        preconditions: []
"#;
    let bwp = BlueprintWithPreconditions::from_yaml(yaml).unwrap();
    let mut overlay = GenerationOverlay::default();
    // Force high revision probability
    overlay.transitions.defaults.revision_probability = 0.99;
    overlay.max_self_loop_iterations = 3;

    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
    let config = EngineConfig {
        company_code: "C001".to_string(),
        currency: "USD".to_string(),
        case_id: "CASE-001".to_string(),
        start_time: chrono::NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        domain_context: Default::default(),
    };
    let result = engine.run_case(&config).unwrap();

    // Should eventually complete despite high revision probability (bounded by max_self_loop)
    assert_eq!(result.procedure_states.get("review_proc").map(String::as_str), Some("completed"));

    // Should have some revision events
    let revision_events: Vec<_> = result.event_log.iter()
        .filter(|e| e.command == "revise")
        .collect();
    assert!(!revision_events.is_empty(), "Expected at least one revision with 0.99 probability");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-process-engine --test engine_tests -- --test-threads=4 2>&1 | head -10`
Expected: Compilation error

- [ ] **Step 3: Implement engine**

Write `src/engine.rs`:

```rust
use std::collections::HashMap;

use chrono::{Duration, NaiveDateTime};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal};
use tracing::{debug, warn};

use crate::context::{ArtifactRef, StepContext};
use crate::error::ProcessEngineError;
use crate::event::{ProcessAnomalyRecord, ProcessAnomalyType, ProcessEvent, ProcessEventBuilder, AnomalySeverity};
use crate::loader::BlueprintWithPreconditions;
use crate::overlay::GenerationOverlay;
use crate::record::GeneratedRecord;
use crate::registry::ArtifactRegistry;
use crate::schema::{ProcessBlueprint, ProcessProcedure, ProcedureAggregate, StateTransition};

/// Configuration for a single process case execution.
pub struct EngineConfig {
    pub company_code: String,
    pub currency: String,
    pub case_id: String,
    pub start_time: NaiveDateTime,
    pub domain_context: HashMap<String, serde_json::Value>,
}

/// Result of running a single process case.
pub struct EngineResult {
    pub event_log: Vec<ProcessEvent>,
    pub procedure_states: HashMap<String, String>,
    pub artifacts: Vec<GeneratedRecord>,
    pub anomalies: Vec<ProcessAnomalyRecord>,
    pub total_duration_hours: f64,
}

/// The generic process FSM engine.
pub struct ProcessEngine {
    blueprint: ProcessBlueprint,
    overlay: GenerationOverlay,
    registry: ArtifactRegistry,
    rng: ChaCha8Rng,
    procedure_order: Vec<String>,
}

impl ProcessEngine {
    pub fn new(
        bwp: BlueprintWithPreconditions,
        overlay: GenerationOverlay,
        registry: ArtifactRegistry,
        rng: ChaCha8Rng,
    ) -> Self {
        let procedure_order = bwp.topological_sort().unwrap_or_default();
        Self {
            blueprint: bwp.blueprint,
            overlay,
            registry,
            rng,
            procedure_order,
        }
    }

    /// Run a single process case, producing events and artifacts.
    pub fn run_case(&mut self, config: &EngineConfig) -> Result<EngineResult, ProcessEngineError> {
        let mut event_log = Vec::new();
        let mut procedure_states: HashMap<String, String> = HashMap::new();
        let mut all_artifacts: Vec<GeneratedRecord> = Vec::new();
        let mut anomalies: Vec<ProcessAnomalyRecord> = Vec::new();
        let mut current_time = config.start_time;

        // Build procedure lookup: id → (phase_id, procedure)
        let mut proc_lookup: HashMap<String, (String, ProcessProcedure)> = HashMap::new();
        for phase in &self.blueprint.phases {
            for proc in &phase.procedures {
                proc_lookup.insert(proc.id.clone(), (phase.id.clone(), proc.clone()));
            }
        }

        // Initialize all procedure states
        for phase in &self.blueprint.phases {
            for proc in &phase.procedures {
                procedure_states.insert(proc.id.clone(), proc.aggregate.initial_state.clone());
            }
        }

        // Walk procedures in topological order
        for proc_id in &self.procedure_order {
            let (phase_id, procedure) = match proc_lookup.get(proc_id) {
                Some(p) => p.clone(),
                None => continue,
            };

            // Check discriminator filter
            if let Some(ref disc_filter) = self.overlay.discriminators {
                if let Some(ref proc_disc) = procedure.discriminators {
                    let mut matches = false;
                    for (key, filter_vals) in disc_filter {
                        if let Some(proc_vals) = proc_disc.get(key) {
                            if proc_vals.iter().any(|v| filter_vals.contains(v)) {
                                matches = true;
                                break;
                            }
                        }
                    }
                    if !matches && procedure.discriminators.is_some() {
                        continue;
                    }
                }
            }

            // Run procedure FSM
            let result = self.run_procedure(
                &procedure,
                &phase_id,
                config,
                &mut current_time,
                &all_artifacts,
            )?;

            // Record final state
            procedure_states.insert(proc_id.clone(), result.final_state);

            event_log.extend(result.events);
            all_artifacts.extend(result.artifacts);
            anomalies.extend(result.anomalies);
        }

        let total_duration_hours = (current_time - config.start_time).num_minutes() as f64 / 60.0;

        Ok(EngineResult {
            event_log,
            procedure_states,
            artifacts: all_artifacts,
            anomalies,
            total_duration_hours,
        })
    }

    fn run_procedure(
        &mut self,
        procedure: &ProcessProcedure,
        phase_id: &str,
        config: &EngineConfig,
        current_time: &mut NaiveDateTime,
        prior_artifacts: &[GeneratedRecord],
    ) -> Result<ProcedureResult, ProcessEngineError> {
        let mut events = Vec::new();
        let mut artifacts = Vec::new();
        let mut anomalies = Vec::new();
        let agg = &procedure.aggregate;
        let mut current_state = agg.initial_state.clone();
        let mut iteration = 0u32;
        let max_iterations = self.overlay.iteration_limits
            .per_procedure.get(&procedure.id)
            .copied()
            .unwrap_or(self.overlay.iteration_limits.default);
        let mut loop_count: HashMap<String, usize> = HashMap::new();

        // Resolve actor for this procedure
        let actor_id = procedure.steps.first()
            .map(|s| s.actor.clone())
            .unwrap_or_else(|| "system".to_string());

        loop {
            if iteration >= max_iterations {
                debug!("Procedure {} hit iteration limit {}", procedure.id, max_iterations);
                break;
            }
            iteration += 1;

            // Find outgoing transitions from current state
            let outgoing: Vec<&StateTransition> = agg.transitions.iter()
                .filter(|t| t.from_state == current_state)
                .collect();

            if outgoing.is_empty() {
                // Terminal state — no transitions out
                break;
            }

            // Select transition
            let transition = if outgoing.len() == 1 {
                outgoing[0]
            } else {
                // Multiple transitions: use revision probability to decide
                self.select_transition(&outgoing, &procedure.id)?
            };

            // Self-loop detection
            let loop_key = format!("{}→{}", transition.from_state, transition.to_state);
            let count = loop_count.entry(loop_key.clone()).or_insert(0);
            *count += 1;
            if *count > self.overlay.max_self_loop_iterations {
                // Force to a non-looping transition if available
                let non_loop: Vec<&&StateTransition> = outgoing.iter()
                    .filter(|t| t.to_state != current_state && t.to_state != transition.from_state)
                    .collect();
                if let Some(alt) = non_loop.first() {
                    // Use the alternative transition
                    let alt = **alt;
                    self.emit_transition_event(
                        &mut events, config, phase_id, &procedure.id, &actor_id,
                        &alt.from_state, &alt.to_state, &alt.command, &alt.emits, current_time,
                    );
                    current_state = alt.to_state.clone();
                    self.advance_time(current_time, &procedure.id);
                    continue;
                } else {
                    break; // No non-looping alternatives
                }
            }

            // Emit transition event
            self.emit_transition_event(
                &mut events, config, phase_id, &procedure.id, &actor_id,
                &transition.from_state, &transition.to_state,
                &transition.command, &transition.emits, current_time,
            );

            let prev_state = current_state.clone();
            current_state = transition.to_state.clone();

            // Advance time
            self.advance_time(current_time, &procedure.id);

            // Execute steps when entering a "work" state (not initial, not terminal-looking)
            // Heuristic: execute steps on the first transition that moves past the initial state
            let is_work_entry = prev_state == agg.initial_state
                || (prev_state.contains("review") && current_state.contains("progress"));

            if is_work_entry {
                for step in &procedure.steps {
                    // Build step context
                    let step_ctx = StepContext {
                        case_id: config.case_id.clone(),
                        blueprint_id: self.blueprint.id.clone(),
                        phase_id: phase_id.to_string(),
                        procedure_id: procedure.id.clone(),
                        step_id: step.id.clone(),
                        timestamp: *current_time,
                        actor_id: if step.actor.is_empty() { actor_id.clone() } else { step.actor.clone() },
                        actor_role: step.action.clone(),
                        company_code: config.company_code.clone(),
                        entity_code: None,
                        department: None,
                        currency: config.currency.clone(),
                        procedure_state: current_state.clone(),
                        iteration,
                        prior_artifacts: prior_artifacts.iter()
                            .chain(artifacts.iter())
                            .map(ArtifactRef::from_record)
                            .collect(),
                        domain_context: config.domain_context.clone(),
                    };

                    // Generate artifact if step has artifact_type
                    if let Some(ref artifact_type) = step.artifact_type {
                        // Find schema for this artifact type
                        let schema = self.blueprint.artifact_schemas.iter()
                            .find(|s| s.id == *artifact_type);

                        match self.registry.generate(artifact_type, &step_ctx, schema, &mut self.rng) {
                            Ok(records) => artifacts.extend(records),
                            Err(e) => {
                                warn!("Artifact generation failed for {}: {}", artifact_type, e);
                            }
                        }
                    }

                    // Emit step event
                    let step_event = ProcessEventBuilder::step()
                        .case_id(&config.case_id)
                        .procedure_id(&procedure.id)
                        .step_id(&step.id)
                        .phase_id(phase_id)
                        .actor_id(if step.actor.is_empty() { &actor_id } else { &step.actor })
                        .command(&step.command)
                        .emits(&step.emits)
                        .timestamp(*current_time);

                    let step_event = if let Some(ref at) = step.artifact_type {
                        step_event.artifact_type(at)
                    } else {
                        step_event
                    };

                    events.push(step_event.build());

                    // Advance time slightly between steps
                    *current_time += Duration::minutes(self.rng.gen_range(5..30));
                }
            }
        }

        Ok(ProcedureResult {
            final_state: current_state,
            events,
            artifacts,
            anomalies,
        })
    }

    fn select_transition<'a>(
        &mut self,
        transitions: &[&'a StateTransition],
        procedure_id: &str,
    ) -> Result<&'a StateTransition, ProcessEngineError> {
        // Check for revision-like transitions (going backward)
        // Convention: transitions going to a state that appears "earlier" are revisions
        let revision_prob = self.overlay.transitions.per_procedure
            .get(procedure_id)
            .and_then(|o| o.revision_probability)
            .unwrap_or(self.overlay.transitions.defaults.revision_probability);

        if transitions.len() == 2 {
            // Common pattern: one forward, one backward (revision)
            let roll: f64 = self.rng.gen();
            if roll < revision_prob {
                // Pick the "revision" transition (heuristic: shorter to_state or going to earlier state)
                // Simple heuristic: the one whose to_state appears in a from_state of another transition
                return Ok(transitions[0]);
            } else {
                return Ok(transitions[1]);
            }
        }

        // Fallback: uniform random
        let idx = self.rng.gen_range(0..transitions.len());
        Ok(transitions[idx])
    }

    fn emit_transition_event(
        &mut self,
        events: &mut Vec<ProcessEvent>,
        config: &EngineConfig,
        phase_id: &str,
        procedure_id: &str,
        actor_id: &str,
        from_state: &str,
        to_state: &str,
        command: &str,
        emits: &str,
        current_time: &NaiveDateTime,
    ) {
        let event = ProcessEventBuilder::transition()
            .case_id(&config.case_id)
            .procedure_id(procedure_id)
            .phase_id(phase_id)
            .from_state(from_state)
            .to_state(to_state)
            .actor_id(actor_id)
            .command(command)
            .emits(emits)
            .timestamp(*current_time)
            .build();
        events.push(event);
    }

    fn advance_time(&mut self, current_time: &mut NaiveDateTime, procedure_id: &str) {
        let timing = self.overlay.transitions.per_procedure
            .get(procedure_id)
            .and_then(|o| o.timing.as_ref())
            .unwrap_or(&self.overlay.transitions.defaults.timing);

        // LogNormal time advance
        let mu = timing.mu_hours.ln();
        let sigma = timing.sigma_hours.max(0.01);
        if let Ok(dist) = LogNormal::new(mu, sigma) {
            let hours: f64 = dist.sample(&mut self.rng);
            let minutes = (hours * 60.0).min(10_000.0) as i64; // Cap at ~7 days
            *current_time += Duration::minutes(minutes);
        } else {
            *current_time += Duration::hours(timing.mu_hours as i64);
        }
    }
}

struct ProcedureResult {
    final_state: String,
    events: Vec<ProcessEvent>,
    artifacts: Vec<GeneratedRecord>,
    anomalies: Vec<ProcessAnomalyRecord>,
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p datasynth-process-engine --test engine_tests -- --test-threads=4`
Expected: All 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/src/engine.rs crates/datasynth-process-engine/tests/engine_tests.rs
git commit -m "feat(process-engine): implement FSM engine with topological walk, revision loops, deterministic RNG"
```

---

### Task 11: Implement export modules

**Files:**
- Modify: `crates/datasynth-process-engine/src/export/mod.rs`
- Create: `crates/datasynth-process-engine/src/export/json.rs`
- Create: `crates/datasynth-process-engine/src/export/csv.rs`
- Create: `crates/datasynth-process-engine/src/export/ocel.rs`

- [ ] **Step 1: Write failing test**

Add to `tests/engine_tests.rs`:

```rust
use datasynth_process_engine::export::json::export_events_to_json;
use datasynth_process_engine::export::csv::export_events_to_csv;

#[test]
fn test_json_export() {
    let bwp = BlueprintWithPreconditions::from_yaml(simple_blueprint_yaml()).unwrap();
    let overlay = GenerationOverlay::default();
    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
    let config = EngineConfig {
        company_code: "C001".to_string(),
        currency: "USD".to_string(),
        case_id: "CASE-001".to_string(),
        start_time: chrono::NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        domain_context: Default::default(),
    };
    let result = engine.run_case(&config).unwrap();

    let json = export_events_to_json(&result.event_log).unwrap();
    assert!(json.contains("CASE-001"));
    assert!(json.contains("step_one"));

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_csv_export() {
    let bwp = BlueprintWithPreconditions::from_yaml(simple_blueprint_yaml()).unwrap();
    let overlay = GenerationOverlay::default();
    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
    let config = EngineConfig {
        company_code: "C001".to_string(),
        currency: "USD".to_string(),
        case_id: "CASE-001".to_string(),
        start_time: chrono::NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        domain_context: Default::default(),
    };
    let result = engine.run_case(&config).unwrap();

    let csv = export_events_to_csv(&result.event_log, "CASE-001").unwrap();
    assert!(csv.contains("case_id"));
    assert!(csv.contains("CASE-001"));
}
```

- [ ] **Step 2: Implement export modules**

Write `src/export/mod.rs`:

```rust
pub mod csv;
pub mod json;
pub mod ocel;
```

Write `src/export/json.rs`:

```rust
use crate::event::ProcessEvent;

pub fn export_events_to_json(events: &[ProcessEvent]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(events)
}

pub fn export_events_to_file(events: &[ProcessEvent], path: &std::path::Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(events)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}
```

Write `src/export/csv.rs`:

```rust
use crate::event::ProcessEvent;

pub fn export_events_to_csv(events: &[ProcessEvent], case_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();
    output.push_str("case_id,event_id,timestamp,event_type,procedure_id,step_id,phase_id,from_state,to_state,actor_id,command,emits,is_anomaly\n");

    for event in events {
        output.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            case_id,
            event.event_id,
            event.timestamp.format("%Y-%m-%dT%H:%M:%S"),
            event.event_type,
            event.procedure_id,
            event.step_id.as_deref().unwrap_or(""),
            event.phase_id,
            event.from_state.as_deref().unwrap_or(""),
            event.to_state.as_deref().unwrap_or(""),
            event.actor_id,
            event.command,
            event.emits,
            event.is_anomaly,
        ));
    }
    Ok(output)
}

pub fn export_events_to_csv_file(events: &[ProcessEvent], case_id: &str, path: &std::path::Path) -> std::io::Result<()> {
    let csv = export_events_to_csv(events, case_id)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, csv)
}
```

Write `src/export/ocel.rs`:

```rust
use serde::Serialize;
use crate::event::ProcessEvent;

/// OCEL 2.0 compliant event log structure.
#[derive(Debug, Serialize)]
pub struct OcelLog {
    #[serde(rename = "ocel:global-event")]
    pub global_event: OcelGlobalEvent,
    #[serde(rename = "ocel:global-object")]
    pub global_object: OcelGlobalObject,
    #[serde(rename = "ocel:events")]
    pub events: Vec<OcelEvent>,
    #[serde(rename = "ocel:objects")]
    pub objects: Vec<OcelObject>,
}

#[derive(Debug, Serialize)]
pub struct OcelGlobalEvent {
    #[serde(rename = "ocel:activity")]
    pub activity: String,
}

#[derive(Debug, Serialize)]
pub struct OcelGlobalObject {
    #[serde(rename = "ocel:type")]
    pub object_type: String,
}

#[derive(Debug, Serialize)]
pub struct OcelEvent {
    #[serde(rename = "ocel:id")]
    pub id: String,
    #[serde(rename = "ocel:activity")]
    pub activity: String,
    #[serde(rename = "ocel:timestamp")]
    pub timestamp: String,
    #[serde(rename = "ocel:omap")]
    pub object_map: Vec<String>,
    #[serde(rename = "ocel:vmap")]
    pub value_map: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct OcelObject {
    #[serde(rename = "ocel:id")]
    pub id: String,
    #[serde(rename = "ocel:type")]
    pub object_type: String,
}

pub fn project_to_ocel(events: &[ProcessEvent], case_id: &str) -> OcelLog {
    let ocel_events: Vec<OcelEvent> = events.iter().map(|e| {
        OcelEvent {
            id: e.event_id.to_string(),
            activity: if e.emits.is_empty() { e.command.clone() } else { e.emits.clone() },
            timestamp: e.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
            object_map: vec![case_id.to_string()],
            value_map: serde_json::json!({
                "procedure_id": e.procedure_id,
                "phase_id": e.phase_id,
                "actor_id": e.actor_id,
                "event_type": e.event_type,
            }),
        }
    }).collect();

    OcelLog {
        global_event: OcelGlobalEvent { activity: "__INVALID__".to_string() },
        global_object: OcelGlobalObject { object_type: "case".to_string() },
        events: ocel_events,
        objects: vec![OcelObject {
            id: case_id.to_string(),
            object_type: "case".to_string(),
        }],
    }
}

pub fn export_ocel_to_json(events: &[ProcessEvent], case_id: &str) -> Result<String, serde_json::Error> {
    let ocel = project_to_ocel(events, case_id);
    serde_json::to_string_pretty(&ocel)
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p datasynth-process-engine --test engine_tests -- --test-threads=4`
Expected: All tests pass including export tests

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-process-engine/src/export/
git commit -m "feat(process-engine): implement JSON, CSV, and OCEL 2.0 event trail exporters"
```

---

### Task 12: Implement SchemaGenerator

**Files:**
- Modify: `crates/datasynth-process-engine/src/schema_generator.rs`
- Create: `crates/datasynth-process-engine/tests/schema_generator_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/schema_generator_tests.rs`:

```rust
use datasynth_process_engine::artifact_schema::*;
use datasynth_process_engine::context::StepContext;
use datasynth_process_engine::record::RecordValue;
use datasynth_process_engine::schema_generator::SchemaGenerator;
use chrono::NaiveDateTime;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

fn test_context() -> StepContext {
    StepContext {
        case_id: "CASE-001".to_string(),
        blueprint_id: "test".to_string(),
        phase_id: "p1".to_string(),
        procedure_id: "proc1".to_string(),
        step_id: "s1".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2024-06-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        actor_id: "actor1".to_string(),
        actor_role: "worker".to_string(),
        company_code: "C001".to_string(),
        entity_code: None,
        department: None,
        currency: "USD".to_string(),
        procedure_state: "in_progress".to_string(),
        iteration: 0,
        prior_artifacts: vec![],
        domain_context: HashMap::new(),
    }
}

#[test]
fn test_schema_generator_produces_fields() {
    let schema_yaml = r#"
id: test_schema
fields:
  - name: id_field
    type: string
    generator: uuid
  - name: amount
    type: decimal
    distribution:
      type: normal
      params:
        mean: 1000.0
        std: 200.0
  - name: count
    type: integer
    distribution:
      type: uniform
      params:
        min: 1.0
        max: 10.0
  - name: active
    type: boolean
  - name: category
    type:
      enum:
        values: [A, B, C]
        weights: [0.5, 0.3, 0.2]
"#;
    let schema: ArtifactSchema = serde_yaml::from_str(schema_yaml).unwrap();
    let gen = SchemaGenerator::new();
    let ctx = test_context();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let records = gen.generate(&ctx, &schema, &mut rng).unwrap();
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.artifact_type, "test_schema");
    assert_eq!(record.fields.len(), 5);

    // UUID field should be a string
    assert!(matches!(record.fields.get("id_field"), Some(RecordValue::String(_))));

    // Amount should be a decimal
    assert!(matches!(record.fields.get("amount"), Some(RecordValue::Decimal(_))));

    // Count should be an integer
    assert!(matches!(record.fields.get("count"), Some(RecordValue::Integer(_))));

    // Active should be a boolean
    assert!(matches!(record.fields.get("active"), Some(RecordValue::Boolean(_))));

    // Category should be a string (from enum)
    match record.fields.get("category") {
        Some(RecordValue::String(s)) => assert!(["A", "B", "C"].contains(&s.as_str())),
        other => panic!("Expected String enum value, got: {:?}", other),
    }
}

#[test]
fn test_schema_generator_handles_nullable_fields() {
    let schema_yaml = r#"
id: nullable_test
fields:
  - name: optional_field
    type: string
    generator: uuid
    nullable: true
    null_rate: 1.0
"#;
    let schema: ArtifactSchema = serde_yaml::from_str(schema_yaml).unwrap();
    let gen = SchemaGenerator::new();
    let ctx = test_context();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let records = gen.generate(&ctx, &schema, &mut rng).unwrap();
    // With null_rate 1.0, field should always be null
    assert!(matches!(records[0].fields.get("optional_field"), Some(RecordValue::Null)));
}

#[test]
fn test_schema_generator_deterministic() {
    let schema_yaml = r#"
id: det_test
fields:
  - name: value
    type: decimal
    distribution:
      type: normal
      params:
        mean: 100.0
        std: 10.0
"#;
    let schema: ArtifactSchema = serde_yaml::from_str(schema_yaml).unwrap();
    let gen = SchemaGenerator::new();
    let ctx = test_context();

    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let r1 = gen.generate(&ctx, &schema, &mut rng1).unwrap();

    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let r2 = gen.generate(&ctx, &schema, &mut rng2).unwrap();

    assert_eq!(
        format!("{:?}", r1[0].fields.get("value")),
        format!("{:?}", r2[0].fields.get("value")),
    );
}
```

- [ ] **Step 2: Implement SchemaGenerator**

Replace `src/schema_generator.rs`:

```rust
use indexmap::IndexMap;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal, Normal, Uniform};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;

use crate::artifact_schema::{ArtifactSchema, DistributionType, FieldGenerator, FieldSchema, FieldType};
use crate::context::StepContext;
use crate::error::ProcessEngineError;
use crate::record::{GeneratedRecord, RecordMetadata, RecordValue};

/// Fallback generator that produces records from artifact schema definitions.
pub struct SchemaGenerator;

impl SchemaGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        context: &StepContext,
        schema: &ArtifactSchema,
        rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, ProcessEngineError> {
        let mut fields = IndexMap::new();

        for field_schema in &schema.fields {
            let value = self.generate_field(field_schema, context, rng)?;
            fields.insert(field_schema.name.clone(), value);
        }

        let record = GeneratedRecord {
            artifact_type: schema.id.clone(),
            record_id: Uuid::new_v4().to_string(),
            fields,
            metadata: RecordMetadata {
                case_id: context.case_id.clone(),
                step_id: context.step_id.clone(),
                procedure_id: context.procedure_id.clone(),
                phase_id: context.phase_id.clone(),
                timestamp: context.timestamp,
                actor_id: context.actor_id.clone(),
                blueprint_id: context.blueprint_id.clone(),
            },
        };

        Ok(vec![record])
    }

    fn generate_field(
        &self,
        field: &FieldSchema,
        context: &StepContext,
        rng: &mut ChaCha8Rng,
    ) -> Result<RecordValue, ProcessEngineError> {
        // Check nullable
        if field.nullable {
            let null_rate = field.null_rate.unwrap_or(0.1);
            if rng.gen::<f64>() < null_rate {
                return Ok(RecordValue::Null);
            }
        }

        // If there's a generator, use it
        if let Some(ref gen) = field.generator {
            return self.generate_from_generator(gen, field, context, rng);
        }

        // If there's a distribution, use it
        if let Some(ref dist) = field.distribution {
            return self.generate_from_distribution(dist, field, rng);
        }

        // Default generation by type
        self.generate_default(field, rng)
    }

    fn generate_from_generator(
        &self,
        gen: &FieldGenerator,
        field: &FieldSchema,
        context: &StepContext,
        rng: &mut ChaCha8Rng,
    ) -> Result<RecordValue, ProcessEngineError> {
        match gen {
            FieldGenerator::Uuid => Ok(RecordValue::String(Uuid::new_v4().to_string())),
            FieldGenerator::PersonName => {
                let first_names = ["Alice", "Bob", "Carol", "David", "Emma", "Frank", "Grace", "Henry"];
                let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis"];
                let first = first_names[rng.gen_range(0..first_names.len())];
                let last = last_names[rng.gen_range(0..last_names.len())];
                Ok(RecordValue::String(format!("{} {}", first, last)))
            }
            FieldGenerator::CompanyName => {
                let names = ["Acme Corp", "Global Industries", "Tech Solutions", "Prime Services", "Atlas Holdings"];
                Ok(RecordValue::String(names[rng.gen_range(0..names.len())].to_string()))
            }
            FieldGenerator::StepTimestamp => {
                Ok(RecordValue::DateTime(context.timestamp))
            }
            FieldGenerator::FromContext(key) => {
                if let Some(val) = context.domain_context.get(key) {
                    match val {
                        serde_json::Value::String(s) => Ok(RecordValue::String(s.clone())),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Ok(RecordValue::Integer(i))
                            } else if let Some(f) = n.as_f64() {
                                Ok(RecordValue::Decimal(Decimal::from_f64(f).unwrap_or_default()))
                            } else {
                                Ok(RecordValue::String(n.to_string()))
                            }
                        }
                        _ => Ok(RecordValue::String(val.to_string())),
                    }
                } else {
                    Ok(RecordValue::Null)
                }
            }
            FieldGenerator::Sequential(prefix) => {
                let seq: u64 = rng.gen_range(1..999999);
                Ok(RecordValue::String(format!("{}{:06}", prefix, seq)))
            }
            FieldGenerator::Pattern(pattern) => {
                // Simple pattern: replace [A-Z] with random uppercase, [0-9] with random digit
                let mut result = String::new();
                let chars: Vec<char> = pattern.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if i + 4 < chars.len() && chars[i] == '[' {
                        if &pattern[i..i+5] == "[A-Z]" {
                            result.push((b'A' + rng.gen_range(0..26)) as char);
                            i += 5;
                            continue;
                        } else if &pattern[i..i+5] == "[0-9]" {
                            result.push((b'0' + rng.gen_range(0..10)) as char);
                            i += 5;
                            continue;
                        }
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                Ok(RecordValue::String(result))
            }
            FieldGenerator::Reference(_) | FieldGenerator::Address | FieldGenerator::Email | FieldGenerator::Phone => {
                // Simplified generation for less common types
                Ok(RecordValue::String(format!("{}_{}", field.name, rng.gen_range(1000..9999))))
            }
        }
    }

    fn generate_from_distribution(
        &self,
        dist_spec: &crate::artifact_schema::DistributionSpec,
        field: &FieldSchema,
        rng: &mut ChaCha8Rng,
    ) -> Result<RecordValue, ProcessEngineError> {
        let raw_value = match dist_spec.dist_type {
            DistributionType::Normal => {
                let mean = dist_spec.params.get("mean").copied().unwrap_or(0.0);
                let std = dist_spec.params.get("std").copied().unwrap_or(1.0);
                let dist = Normal::new(mean, std).unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
                let mut val = dist.sample(rng);
                // Apply min/max bounds
                if let Some(&min) = dist_spec.params.get("min") { val = val.max(min); }
                if let Some(&max) = dist_spec.params.get("max") { val = val.min(max); }
                val
            }
            DistributionType::LogNormal => {
                let mu = dist_spec.params.get("mu").copied().unwrap_or(0.0);
                let sigma = dist_spec.params.get("sigma").copied().unwrap_or(1.0);
                let dist = LogNormal::new(mu, sigma.max(0.01)).unwrap_or_else(|_| LogNormal::new(0.0, 1.0).unwrap());
                let mut val = dist.sample(rng);
                if let Some(&min) = dist_spec.params.get("min") { val = val.max(min); }
                if let Some(&max) = dist_spec.params.get("max") { val = val.min(max); }
                val
            }
            DistributionType::Uniform => {
                let min = dist_spec.params.get("min").copied().unwrap_or(0.0);
                let max = dist_spec.params.get("max").copied().unwrap_or(1.0);
                let dist = Uniform::new(min, max);
                dist.sample(rng)
            }
            DistributionType::Beta => {
                // Approximate beta with rejection sampling from uniform
                let alpha = dist_spec.params.get("alpha").copied().unwrap_or(2.0);
                let beta_param = dist_spec.params.get("beta").copied().unwrap_or(5.0);
                let scale = dist_spec.params.get("scale").copied().unwrap_or(1.0);
                // Simple approximation: use mean of beta distribution
                let mean = alpha / (alpha + beta_param);
                let std = (alpha * beta_param / ((alpha + beta_param).powi(2) * (alpha + beta_param + 1.0))).sqrt();
                let dist = Normal::new(mean, std).unwrap_or_else(|_| Normal::new(0.5, 0.1).unwrap());
                let val = dist.sample(rng).clamp(0.0, 1.0);
                val * scale
            }
            _ => {
                // Fallback for less common distributions
                rng.gen::<f64>() * 1000.0
            }
        };

        // Convert to appropriate type
        match field.field_type {
            FieldType::Decimal => Ok(RecordValue::Decimal(
                Decimal::from_f64(raw_value).unwrap_or_default()
            )),
            FieldType::Integer => Ok(RecordValue::Integer(raw_value.round() as i64)),
            _ => Ok(RecordValue::Decimal(
                Decimal::from_f64(raw_value).unwrap_or_default()
            )),
        }
    }

    fn generate_default(
        &self,
        field: &FieldSchema,
        rng: &mut ChaCha8Rng,
    ) -> Result<RecordValue, ProcessEngineError> {
        match &field.field_type {
            FieldType::String => Ok(RecordValue::String(format!("{}_{}", field.name, rng.gen_range(1000..9999)))),
            FieldType::Integer => Ok(RecordValue::Integer(rng.gen_range(0..1000))),
            FieldType::Decimal => Ok(RecordValue::Decimal(
                Decimal::from_f64(rng.gen::<f64>() * 1000.0).unwrap_or_default()
            )),
            FieldType::Boolean => Ok(RecordValue::Boolean(rng.gen_bool(0.5))),
            FieldType::Date => Ok(RecordValue::Date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + chrono::Duration::days(rng.gen_range(0..365))
            )),
            FieldType::DateTime => Ok(RecordValue::DateTime(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()
                + chrono::Duration::seconds(rng.gen_range(0..31_536_000))
            )),
            FieldType::Enum { values, weights } => {
                if values.is_empty() {
                    return Ok(RecordValue::Null);
                }
                let idx = if weights.is_empty() || weights.len() != values.len() {
                    rng.gen_range(0..values.len())
                } else {
                    weighted_select(weights, rng)
                };
                Ok(RecordValue::String(values[idx].clone()))
            }
        }
    }
}

impl Default for SchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn weighted_select(weights: &[f64], rng: &mut ChaCha8Rng) -> usize {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0;
    }
    let mut roll: f64 = rng.gen::<f64>() * total;
    for (i, &w) in weights.iter().enumerate() {
        roll -= w;
        if roll <= 0.0 {
            return i;
        }
    }
    weights.len() - 1
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p datasynth-process-engine --test schema_generator_tests -- --test-threads=4`
Expected: All 3 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-process-engine/src/schema_generator.rs crates/datasynth-process-engine/tests/schema_generator_tests.rs
git commit -m "feat(process-engine): implement SchemaGenerator with distributions, generators, and nullable fields"
```

---

### Task 13: End-to-end integration test

**Files:**
- Create: `crates/datasynth-process-engine/tests/integration_test.rs`

- [ ] **Step 1: Write integration test**

Create `tests/integration_test.rs`:

```rust
//! End-to-end test: load a custom blueprint with schema-driven artifacts,
//! run the engine, verify event trail and artifact output.

use datasynth_process_engine::engine::{EngineConfig, ProcessEngine};
use datasynth_process_engine::export::json::export_events_to_json;
use datasynth_process_engine::export::ocel::export_ocel_to_json;
use datasynth_process_engine::loader::BlueprintWithPreconditions;
use datasynth_process_engine::overlay::GenerationOverlay;
use datasynth_process_engine::record::RecordValue;
use datasynth_process_engine::registry::ArtifactRegistry;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const LOAN_BLUEPRINT: &str = r#"
id: loan_origination
name: "Consumer Loan Origination"
version: "1.0.0"
schema_version: "2.0"
domain: banking
depth: standard

actors:
  - { id: loan_officer, name: "Loan Officer" }
  - { id: credit_analyst, name: "Credit Analyst" }
  - { id: underwriter, name: "Underwriter" }

artifact_schemas:
  - id: loan_application
    fields:
      - { name: application_id, type: string, generator: uuid }
      - { name: applicant_name, type: string, generator: person_name }
      - name: loan_amount
        type: decimal
        distribution:
          type: lognormal
          params: { mu: 10.2, sigma: 1.4 }
      - name: loan_type
        type:
          enum:
            values: [mortgage, auto, personal]
            weights: [0.4, 0.35, 0.25]
  - id: credit_report
    fields:
      - { name: report_id, type: string, generator: uuid }
      - name: credit_score
        type: integer
        distribution:
          type: normal
          params: { mean: 720.0, std: 80.0, min: 300.0, max: 850.0 }
  - id: underwriting_decision
    fields:
      - { name: decision_id, type: string, generator: uuid }
      - name: decision
        type:
          enum:
            values: [approved, denied, conditional]
            weights: [0.55, 0.15, 0.30]

phases:
  - id: intake
    name: "Application Intake"
    order: 1
    procedures:
      - id: receive_application
        title: "Receive Application"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_intake, emits: IntakeStarted }
            - { from_state: in_progress, to_state: completed, command: complete_intake, emits: IntakeCompleted }
        steps:
          - { id: collect_info, order: 1, action: collect, actor: loan_officer, command: collect_data, emits: DataCollected, artifact_type: loan_application }
        preconditions: []

  - id: assessment
    name: "Credit Assessment"
    order: 2
    gate:
      all_of:
        - { procedure: receive_application, state: completed }
    procedures:
      - id: credit_check
        title: "Credit Check"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_check, emits: CheckStarted }
            - { from_state: in_progress, to_state: completed, command: complete_check, emits: CheckCompleted }
        steps:
          - { id: pull_credit, order: 1, action: retrieve, actor: credit_analyst, command: pull_credit_report, emits: CreditPulled, artifact_type: credit_report }
        preconditions: [receive_application]

  - id: underwriting
    name: "Underwriting"
    order: 3
    gate:
      all_of:
        - { procedure: credit_check, state: completed }
    procedures:
      - id: underwrite
        title: "Underwriting Decision"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, decided, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_uw, emits: UWStarted }
            - { from_state: in_progress, to_state: decided, command: decide, emits: DecisionMade }
            - { from_state: decided, to_state: completed, command: finalize, emits: Finalized }
        steps:
          - { id: make_decision, order: 1, action: decide, actor: underwriter, command: make_uw_decision, emits: UWDecided, artifact_type: underwriting_decision }
        preconditions: [credit_check]
"#;

#[test]
fn test_loan_origination_end_to_end() {
    // Load and validate blueprint
    let bwp = BlueprintWithPreconditions::from_yaml(LOAN_BLUEPRINT).unwrap();
    bwp.validate().unwrap();
    let sorted = bwp.topological_sort().unwrap();
    assert_eq!(sorted, vec!["receive_application", "credit_check", "underwrite"]);

    // Run engine
    let overlay = GenerationOverlay::default();
    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);

    let config = EngineConfig {
        company_code: "BANK01".to_string(),
        currency: "USD".to_string(),
        case_id: "LOAN-2024-001".to_string(),
        start_time: chrono::NaiveDateTime::parse_from_str("2024-06-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        domain_context: Default::default(),
    };

    let result = engine.run_case(&config).unwrap();

    // Verify all procedures completed
    assert_eq!(result.procedure_states.get("receive_application").map(String::as_str), Some("completed"));
    assert_eq!(result.procedure_states.get("credit_check").map(String::as_str), Some("completed"));
    assert_eq!(result.procedure_states.get("underwrite").map(String::as_str), Some("completed"));

    // Verify events were generated
    assert!(!result.event_log.is_empty());
    let transition_events: Vec<_> = result.event_log.iter().filter(|e| e.event_type == "transition").collect();
    let step_events: Vec<_> = result.event_log.iter().filter(|e| e.event_type == "step").collect();
    assert!(!transition_events.is_empty());
    assert!(!step_events.is_empty());

    // Verify artifacts were generated (3 procedures with 1 artifact each)
    assert_eq!(result.artifacts.len(), 3);

    // Verify loan_application artifact has expected fields
    let loan_app = result.artifacts.iter().find(|a| a.artifact_type == "loan_application").unwrap();
    assert!(loan_app.fields.contains_key("application_id"));
    assert!(loan_app.fields.contains_key("applicant_name"));
    assert!(loan_app.fields.contains_key("loan_amount"));
    assert!(loan_app.fields.contains_key("loan_type"));

    // loan_amount should be a decimal
    assert!(matches!(loan_app.fields.get("loan_amount"), Some(RecordValue::Decimal(_))));

    // loan_type should be one of the enum values
    match loan_app.fields.get("loan_type") {
        Some(RecordValue::String(s)) => assert!(["mortgage", "auto", "personal"].contains(&s.as_str())),
        other => panic!("Expected loan_type string, got: {:?}", other),
    }

    // Verify credit_report
    let credit = result.artifacts.iter().find(|a| a.artifact_type == "credit_report").unwrap();
    match credit.fields.get("credit_score") {
        Some(RecordValue::Integer(score)) => assert!(*score >= 300 && *score <= 850),
        other => panic!("Expected credit_score integer, got: {:?}", other),
    }

    // Verify underwriting decision
    let uw = result.artifacts.iter().find(|a| a.artifact_type == "underwriting_decision").unwrap();
    match uw.fields.get("decision") {
        Some(RecordValue::String(s)) => assert!(["approved", "denied", "conditional"].contains(&s.as_str())),
        other => panic!("Expected decision string, got: {:?}", other),
    }

    // Verify JSON export works
    let json = export_events_to_json(&result.event_log).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());

    // Verify OCEL export works
    let ocel = export_ocel_to_json(&result.event_log, "LOAN-2024-001").unwrap();
    let parsed_ocel: serde_json::Value = serde_json::from_str(&ocel).unwrap();
    assert!(parsed_ocel.get("ocel:events").unwrap().is_array());

    // Verify timestamps are monotonically increasing
    let timestamps: Vec<_> = result.event_log.iter().map(|e| e.timestamp).collect();
    for window in timestamps.windows(2) {
        assert!(window[0] <= window[1], "Events should be in chronological order");
    }

    // Verify positive duration
    assert!(result.total_duration_hours > 0.0);
}

#[test]
fn test_multiple_cases_with_different_seeds() {
    let bwp = BlueprintWithPreconditions::from_yaml(LOAN_BLUEPRINT).unwrap();
    let overlay = GenerationOverlay::default();
    let registry = ArtifactRegistry::new();
    let rng = ChaCha8Rng::seed_from_u64(1);
    let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);

    let mut all_decisions = Vec::new();
    for i in 0..20 {
        let bwp = BlueprintWithPreconditions::from_yaml(LOAN_BLUEPRINT).unwrap();
        let overlay = GenerationOverlay::default();
        let registry = ArtifactRegistry::new();
        let rng = ChaCha8Rng::seed_from_u64(i);
        let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);

        let config = EngineConfig {
            company_code: "BANK01".to_string(),
            currency: "USD".to_string(),
            case_id: format!("LOAN-{}", i),
            start_time: chrono::NaiveDateTime::parse_from_str("2024-06-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            domain_context: Default::default(),
        };
        let result = engine.run_case(&config).unwrap();
        if let Some(uw) = result.artifacts.iter().find(|a| a.artifact_type == "underwriting_decision") {
            if let Some(RecordValue::String(d)) = uw.fields.get("decision") {
                all_decisions.push(d.clone());
            }
        }
    }

    // With 20 runs and weighted enum, we should see variety
    let unique: std::collections::HashSet<_> = all_decisions.iter().collect();
    assert!(unique.len() > 1, "Expected variety in decisions across 20 runs, got: {:?}", unique);
}
```

- [ ] **Step 2: Run integration tests**

Run: `cargo test -p datasynth-process-engine --test integration_test -- --test-threads=4`
Expected: All tests pass

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p datasynth-process-engine -- --test-threads=4`
Expected: All tests across all test files pass

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p datasynth-process-engine -- -D warnings 2>&1 | tail -20`
Expected: No errors (fix any warnings)

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-process-engine/tests/integration_test.rs
git commit -m "feat(process-engine): add end-to-end integration test with loan origination blueprint"
```

---

### Task 14: Final cleanup, lib.rs re-exports, and workspace verification

**Files:**
- Modify: `crates/datasynth-process-engine/src/lib.rs`

- [ ] **Step 1: Update lib.rs with re-exports for ergonomic API**

```rust
//! Domain-agnostic YAML-driven process FSM engine.
//!
//! Loads process blueprints as finite state machines and generates realistic
//! event trails and artifacts with configurable behavior via overlays.
//!
//! # Quick Start
//!
//! ```no_run
//! use datasynth_process_engine::loader::BlueprintWithPreconditions;
//! use datasynth_process_engine::overlay::GenerationOverlay;
//! use datasynth_process_engine::registry::ArtifactRegistry;
//! use datasynth_process_engine::engine::{ProcessEngine, EngineConfig};
//! use datasynth_process_engine::export::json::export_events_to_json;
//! use rand::SeedableRng;
//! use rand_chacha::ChaCha8Rng;
//!
//! let yaml = std::fs::read_to_string("my_process.yaml").unwrap();
//! let bwp = BlueprintWithPreconditions::from_yaml(&yaml).unwrap();
//! bwp.validate().unwrap();
//! let overlay = GenerationOverlay::default();
//! let registry = ArtifactRegistry::new();
//! let rng = ChaCha8Rng::seed_from_u64(42);
//! let mut engine = ProcessEngine::new(bwp, overlay, registry, rng);
//! let config = EngineConfig {
//!     company_code: "C001".to_string(),
//!     currency: "USD".to_string(),
//!     case_id: "CASE-001".to_string(),
//!     start_time: chrono::Utc::now().naive_utc(),
//!     domain_context: Default::default(),
//! };
//! let result = engine.run_case(&config).unwrap();
//! let json = export_events_to_json(&result.event_log).unwrap();
//! ```

pub mod artifact_schema;
pub mod context;
pub mod engine;
pub mod error;
pub mod event;
pub mod export;
pub mod loader;
pub mod overlay;
pub mod record;
pub mod registry;
pub mod schema;
pub mod schema_generator;
```

- [ ] **Step 2: Verify full workspace still builds**

Run: `cargo check --workspace`
Expected: Compiles (existing crates unaffected)

- [ ] **Step 3: Run full workspace tests**

Run: `cargo test --workspace -- --test-threads=4 2>&1 | tail -20`
Expected: All existing tests still pass, new process-engine tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-process-engine/src/lib.rs
git commit -m "feat(process-engine): finalize lib.rs with docs and workspace verification"
```

---

## What's Next (Future Plans)

This plan covers **Phase 1: Core Engine Creation**. The following phases will need separate implementation plans:

**Phase 2: Validate with Reference Blueprints**
- Write P2P and O2C blueprint YAML facades
- Register existing generators (`PurchaseOrderGenerator`, etc.) as `ArtifactGenerator` impls
- Integration tests comparing blueprint-driven vs. direct generation

**Phase 3: CLI and Config Integration**
- Add `process_blueprints` config section to `datasynth-config/src/schema.rs`
- Add `--blueprint` CLI flag to `datasynth-cli`
- Wire into `EnhancedOrchestrator`
- Output routing to `processes/` directory

**Phase 4: Schema-Driven Generator Enhancements**
- Field constraint evaluation (conditional distributions, correlations)
- Benford compliance for decimal fields
- Copula-based correlation using `datasynth-core` distributions
- External schema file loading

**Phase 5: Audit Crate Refactor**
- Refactor `datasynth-audit-fsm` to depend on `datasynth-process-engine`
- Create adapter layer (audit registry, context mapping)
- Verify all 10 audit blueprints produce identical output
- Backward compatibility for existing audit config
