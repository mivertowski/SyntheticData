# YAML-Driven Audit FSM Integration

**Date**: 2026-03-24
**Status**: Draft
**Scope**: Integrate event-sourced state machine blueprints from AuditMethodology into SyntheticData for realistic audit trail and artifact generation.

## Problem

Current audit generators in `datasynth-generators/src/audit/` (27 files) are stateless and independent. They produce ISA-compliant artifacts but lack:

- Workflow sequencing (no phase progression, no precondition enforcement)
- Event trail generation (no audit trail of state transitions)
- Methodology-driven generation (hardcoded patterns instead of standard-based procedures)
- Configurable complexity (no way to steer audit depth, scope, or actor behavior)

The AuditMethodology repository contains production-ready YAML blueprints defining deterministic finite state machines for Internal Audit (IIA-GIAS, 96.2% coverage) and Financial Statement Audit (ISA-based). These model the full audit lifecycle as event-sourced aggregates with states, transitions, commands, events, guards, actors, evidence catalogs, and standards mappings.

## Solution

Two new crates:

1. **`datasynth-audit-fsm`** — Blueprint schema, YAML loader, FSM execution engine, artifact generation, event trail emission
2. **`datasynth-audit-optimizer`** — Graph analysis, shortest-path, constraint-based optimization, Monte Carlo simulation

The architecture separates "what happens" (methodology blueprint YAML) from "how it generates" (overlay YAML with probabilities, timing, volumes, anomalies). Users can supply custom blueprints and overlays.

## Architecture

### Layer Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Configuration Layer                         │
│  audit_fsm:                                                   │
│    blueprint: builtin:fsa | builtin:ia | path/to/custom.yaml  │
│    overlay: builtin:default | path/to/overlay.yaml            │
│    depth: simplified | standard | full                        │
│    discriminators: { tiers: [core], risk_ratings: [high] }    │
└──────────────────────┬───────────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────────┐
│                  Blueprint Loader                              │
│  YAML → AuditBlueprint (validated Rust types)                 │
│  + Overlay YAML → GenerationOverlay                           │
│  Validates: phase refs, precondition DAG, transition          │
│  integrity, actor/evidence/standards cross-refs               │
└──────────────────────┬───────────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────────┐
│                   FSM Engine                                   │
│  Walks procedures respecting precondition DAG + phase gates   │
│  For each procedure: walks aggregate FSM (states/transitions) │
│  For each step: executes via StepExecutor                     │
│  Emits AuditEvent on every transition and step completion     │
│  Injects anomalies based on overlay config                    │
│  Deterministic: ChaCha8Rng with configurable seed             │
│  Receives EngagementContext (revenue, CoA, employees, etc.)   │
└──────────┬───────────────────────────────────┬───────────────┘
           │                                   │
┌──────────▼──────────┐           ┌────────────▼──────────────┐
│   StepExecutor      │           │    Event Trail             │
│   Maps step.action  │           │    ┌─ Flat JSON log        │
│   + evidence catalog│           │    └─ OCEL 2.0 projection  │
│   to artifact gen   │           └───────────────────────────┘
│   Uses extracted    │
│   generator utils   │
└─────────────────────┘
```

### Blueprint Schema (Rust Types)

All types derive `Debug, Clone, Serialize, Deserialize` and mirror the YAML schema from AuditMethodology.

Example YAML fragment (`generic_fsa.yaml`, planning_materiality procedure):

```yaml
- id: planning_materiality
  phase: planning
  title: "Determine Materiality"
  discriminators:
    tiers: [core]
  aggregate:
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
  steps:
    - id: mat_step_1
      order: 1
      action: determine
      actor: audit_manager
      description: "Determine overall materiality..."
      binding: requirement
      command: determine_overall_materiality
      emits: OverallMaterialityDetermined
      guards:
        - type: field_required
          fields: [materiality_amount, benchmark, percentage]
      evidence:
        inputs: []
        outputs:
          - ref: materiality_workpaper
            type: workpaper
      standards:
        - ref: "ISA 320"
          paragraphs: ["10"]
          binding: requirement
  preconditions:
    - accept_engagement
```

Corresponding Rust types:

```rust
pub struct AuditBlueprint {
    pub schema_version: String,
    pub depth: DepthLevel,                    // simplified | standard | full
    pub methodology: BlueprintMethodology,
    pub discriminators: HashMap<String, Vec<String>>,
    pub actors: Vec<BlueprintActor>,
    pub standards_catalog: Vec<BlueprintStandard>,
    pub evidence_catalog: Vec<BlueprintEvidence>,
    pub phases: Vec<BlueprintPhase>,
    pub procedures: Vec<BlueprintProcedure>,
}

pub struct BlueprintMethodology {
    pub name: String,
    pub version: String,
    pub framework: String,                    // "ISA", "IIA-GIAS"
    pub description: String,
}

pub struct BlueprintActor {
    pub id: String,
    pub name: String,
    pub responsibilities: Vec<String>,
}

pub struct BlueprintStandard {
    pub ref_id: String,                       // "ISA 320", "GIAS-11.1"
    pub title: String,
    pub binding: BindingLevel,                // requirement | guidance
    pub paragraphs: Vec<String>,
    pub dependencies: Vec<String>,
}

pub struct BlueprintEvidence {
    pub id: String,
    pub name: String,
    pub evidence_type: EvidenceType,          // document | workpaper | assessment | memo | report
    pub lifecycle: Vec<String>,               // [draft, under_review, finalized]
}

pub struct BlueprintPhase {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub description: String,
    pub gate: Option<PhaseGate>,
}

pub struct PhaseGate {
    pub all_of: Vec<GateCondition>,
}

pub struct GateCondition {
    pub procedure: String,
    pub state: String,
}

pub struct BlueprintProcedure {
    pub id: String,
    pub phase: String,
    pub title: String,
    pub discriminators: HashMap<String, Vec<String>>,
    pub aggregate: ProcedureAggregate,
    pub steps: Vec<BlueprintStep>,
    pub preconditions: Vec<String>,           // procedure IDs
    pub knowledge_refs: Vec<String>,          // weak refs
}

pub struct ProcedureAggregate {
    pub initial_state: String,
    pub states: Vec<String>,
    pub transitions: Vec<ProcedureTransition>,
}

pub struct ProcedureTransition {
    pub from_state: String,
    pub to_state: String,
    pub command: String,
    pub emits: String,                        // event name
    pub guards: Vec<String>,                  // guard identifiers (e.g. "all_steps_complete")
}

pub struct BlueprintStep {
    pub id: String,
    pub order: i32,
    pub action: StepAction,                   // evaluate, determine, perform, design, issue, etc.
    pub actor: String,
    pub description: String,
    pub binding: BindingLevel,
    pub command: String,
    pub emits: String,
    pub guards: Vec<StepGuard>,
    pub evidence: StepEvidence,
    pub standards: Vec<StepStandard>,
    pub decisions: Vec<StepDecision>,
}

/// Typed guard for step-level preconditions (richer than transition string guards).
pub struct StepGuard {
    pub guard_type: StepGuardType,
    pub fields: Vec<String>,                  // context-dependent fields
}

pub enum StepGuardType {
    FieldRequired,                            // specific fields must be populated
    EvidenceAvailable,                        // referenced input evidence must exist
    PriorStepComplete,                        // a named step must be completed
}

pub struct StepEvidence {
    pub inputs: Vec<EvidenceRef>,
    pub outputs: Vec<EvidenceRef>,
}

pub struct EvidenceRef {
    pub ref_id: String,
    pub evidence_type: EvidenceType,
}

pub struct StepStandard {
    pub ref_id: String,
    pub paragraphs: Vec<String>,
    pub binding: BindingLevel,
}

pub struct StepDecision {
    pub condition: String,
    pub branches: Vec<DecisionBranch>,
}

pub struct DecisionBranch {
    pub label: String,
    pub description: String,
    pub next_step: Option<String>,
}

pub enum DepthLevel { Simplified, Standard, Full }
pub enum BindingLevel { Requirement, Guidance }
pub enum EvidenceType { Document, Workpaper, Assessment, Memo, Report }

/// Typed step actions to enable artifact dispatch.
pub enum StepAction {
    Evaluate,
    Determine,
    Perform,
    Design,
    Agree,
    Identify,
    Assess,
    Issue,
    Understand,
    Other(String),                            // extensibility for custom blueprints
}
```

### Generation Overlay Schema

```rust
pub struct GenerationOverlay {
    pub depth: Option<DepthLevel>,
    pub discriminators: Option<HashMap<String, Vec<String>>>,
    pub transitions: TransitionConfig,
    pub artifacts: ArtifactConfig,
    pub anomalies: AnomalyConfig,
    pub actor_profiles: HashMap<String, ActorProfile>,
}

pub struct TransitionConfig {
    pub defaults: TransitionDefaults,
    pub overrides: HashMap<String, TransitionOverride>,  // keyed by procedure_id
}

pub struct TransitionDefaults {
    pub revision_probability: f64,            // P(under_review → in_progress)
    pub timing: TimingDistribution,
}

/// Per-procedure override. All fields optional; falls back to defaults.
pub struct TransitionOverride {
    pub revision_probability: Option<f64>,
    pub timing: Option<TimingDistribution>,
}

pub struct TimingDistribution {
    pub distribution: TimingDistributionType,
    pub mu_hours: f64,
    pub sigma_hours: f64,
}

pub enum TimingDistributionType {
    LogNormal,
    Normal,
    Uniform,
}

pub struct ArtifactConfig {
    pub workpapers_per_step: VolumeDistribution,
    pub evidence_items_per_workpaper: VolumeDistribution,
}

pub struct VolumeDistribution {
    pub min: u32,
    pub max: u32,
    pub distribution: VolumeDistributionType,
    pub lambda: Option<f64>,                  // for Poisson
}

pub enum VolumeDistributionType {
    Uniform,
    Poisson,
    Normal,
}

pub struct AnomalyConfig {
    pub skipped_approval: AnomalyRule,
    pub late_posting: AnomalyRule,
    pub missing_evidence: AnomalyRule,
    pub out_of_sequence: AnomalyRule,
}

pub struct AnomalyRule {
    pub probability: f64,
    pub applicable_guards: Option<Vec<String>>,
    pub max_delay_hours: Option<f64>,
}

pub struct ActorProfile {
    pub revision_multiplier: f64,
    pub evidence_multiplier: f64,
    pub skip_guidance_steps: bool,
}
```

### Blueprint and Overlay Source Resolution

```rust
/// How to resolve the blueprint YAML.
pub enum BlueprintSource {
    /// Builtin blueprint embedded via include_str!().
    Builtin(BuiltinBlueprint),
    /// Custom YAML file path (resolved relative to config file location).
    Custom(PathBuf),
}

pub enum BuiltinBlueprint {
    Fsa,        // generic_fsa.yaml
    Ia,         // generic_ia.yaml
}

/// How to resolve the overlay YAML.
pub enum OverlaySource {
    /// Builtin overlay preset.
    Builtin(BuiltinOverlay),
    /// Custom YAML file path.
    Custom(PathBuf),
    /// No overlay — use engine defaults.
    None,
}

pub enum BuiltinOverlay {
    Default,    // balanced generation parameters
    Thorough,   // more revisions, more evidence, more time
    Rushed,     // fewer revisions, skip guidance steps, compressed timeline
}
```

Config string parsing: `"builtin:fsa"` → `BlueprintSource::Builtin(BuiltinBlueprint::Fsa)`, `"/path/to/custom.yaml"` → `BlueprintSource::Custom(path)`.

Note on embedded blueprint size: FSA blueprint is ~900 lines (~25KB), IA blueprint is ~3,700 lines (~100KB). Both are within acceptable `include_str!()` range (the codebase already embeds larger assets like SKR04 chart of accounts). If future blueprints exceed ~200KB, switch to filesystem loading with embedded fallback.

### FSM Engine

```rust
pub struct AuditFsmEngine {
    blueprint: AuditBlueprint,
    overlay: GenerationOverlay,
    rng: ChaCha8Rng,
}

/// Context from the broader generation run, passed to the FSM engine.
/// Provides the financial data needed for realistic artifact generation
/// (e.g., revenue for materiality calculation, CoA for risk assessment).
pub struct EngagementContext {
    pub company_code: String,
    pub company_name: String,
    pub fiscal_year: i32,
    pub currency: String,
    pub total_revenue: Decimal,
    pub total_assets: Decimal,
    pub chart_of_accounts: ChartOfAccounts,
    pub employees: Vec<Employee>,             // for actor assignment
    pub engagement_start: NaiveDate,
    pub report_date: NaiveDate,
}

pub struct EngagementState {
    pub procedure_states: HashMap<String, String>,       // procedure_id → current_state
    pub step_completions: HashMap<String, bool>,         // step_id → completed
    pub evidence_states: HashMap<String, String>,        // evidence_id → lifecycle_state
    pub event_log: Vec<AuditEvent>,
    pub artifacts: Vec<GeneratedArtifact>,
    pub anomalies: Vec<AuditAnomalyRecord>,
}

/// Audit-specific anomaly record. Distinct from datasynth-core's AnomalyType
/// which covers fraud/error/process anomalies in transactional data.
pub struct AuditAnomalyRecord {
    pub anomaly_id: Uuid,
    pub anomaly_type: AuditAnomalyType,
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub timestamp: NaiveDateTime,
    pub description: String,
    pub severity: AnomalySeverity,
}

/// Audit-specific anomaly types. These map to audit methodology violations,
/// not transactional anomalies (which use datasynth_core::models::AnomalyType).
pub enum AuditAnomalyType {
    SkippedApproval,          // guard bypassed (e.g., partner_approved skipped)
    LatePosting,              // step executed after expected timeline
    MissingEvidence,          // required evidence input not available
    OutOfSequence,            // precondition not met but procedure advanced
    InsufficientDocumentation, // workpaper below minimum quality threshold
}

pub enum AnomalySeverity { Low, Medium, High, Critical }

pub struct EngagementResult {
    pub state: EngagementState,
    pub phases_completed: Vec<String>,
    pub opinion: Option<OpinionType>,         // reuses datasynth_standards::audit::opinion::OpinionType
    pub total_duration_hours: f64,
}

/// Error categories for FSM operations.
pub enum AuditFsmError {
    /// Blueprint YAML parse failure.
    BlueprintParse { path: String, source: serde_yaml::Error },
    /// Blueprint validation failure (invalid refs, DAG cycles, etc.).
    BlueprintValidation { violations: Vec<ValidationViolation> },
    /// Overlay YAML parse failure.
    OverlayParse { path: String, source: serde_yaml::Error },
    /// Runtime guard failure (should not happen in normal flow).
    GuardFailure { procedure_id: String, guard: String, reason: String },
    /// Precondition not met.
    PreconditionNotMet { procedure_id: String, required: String, actual_state: String },
    /// Blueprint source not found.
    SourceNotFound { source: String },
}

pub struct ValidationViolation {
    pub location: String,     // e.g., "procedures[3].steps[2].evidence.inputs[0]"
    pub message: String,
}

impl AuditFsmEngine {
    /// Load blueprint + overlay from paths or builtins.
    pub fn load(
        blueprint: BlueprintSource,
        overlay: OverlaySource,
        seed: u64,
    ) -> Result<Self, AuditFsmError>;

    /// Run a complete engagement, returning all artifacts + events.
    /// The engine is Send (ChaCha8Rng is Send + Sync) so multiple
    /// engagements can be generated in parallel via rayon.
    pub fn run_engagement(
        &mut self,
        context: &EngagementContext,
    ) -> Result<EngagementResult, AuditFsmError>;
}
```

Engine execution algorithm:

1. **Topological sort** procedures by precondition DAG
2. **For each procedure** (in DAG order):
   a. Check discriminator filter — skip if not in scope
   b. Check preconditions — all referenced procedures must be in required state
   c. Check phase gate — if this is the first procedure in a new phase, verify gate conditions
   d. Initialize aggregate at `initial_state`
   e. **Walk FSM**: select transition from current state (probabilistic if multiple outgoing), execute command, emit transition event
   f. **On entering `in_progress` state**: execute all steps in order via `StepExecutor`
   g. **On entering `under_review` state**: roll for revision (overlay `revision_probability`); if revision, transition back to `in_progress` and re-execute subset of steps
   h. **Resolve decisions**: within step execution, select branch based on overlay probabilities (default: uniform across branches)
   i. **Inject anomalies**: on each transition/step, check overlay anomaly rules against RNG
   j. Repeat until `completed` (or other terminal) state reached
3. **Collect** all events, artifacts, anomalies into `EngagementResult`

**Step-to-state association**: Steps execute when the procedure enters the `in_progress` state. Steps are ordered by their `order` field and execute sequentially within that state. Transition events (`emits` on `ProcedureTransition`) fire on state changes; step events (`emits` on `BlueprintStep`) fire during step execution within `in_progress`. This maps directly to the YAML semantics where steps represent the work done within a procedure, and the aggregate tracks the procedure's review lifecycle.

### Step Executor and Artifact Generation

```rust
pub struct StepExecutor {
    evidence_catalog: HashMap<String, BlueprintEvidence>,
    actor_profiles: HashMap<String, ActorProfile>,
}

pub struct StepResult {
    pub artifacts: Vec<GeneratedArtifact>,
    pub events: Vec<AuditEvent>,
    pub evidence_state_changes: Vec<(String, String)>,   // (evidence_id, new_state)
}

pub struct GeneratedArtifact {
    pub artifact_id: Uuid,
    pub artifact_type: ArtifactType,
    pub step_id: String,
    pub procedure_id: String,
    pub phase_id: String,
    pub actor_id: String,
    pub timestamp: NaiveDateTime,
    pub standards_refs: Vec<String>,
    pub content: ArtifactContent,
}

/// Discriminates the type of generated artifact.
pub enum ArtifactType {
    Workpaper,
    Evidence,
    Assessment,
    Report,
    Letter,
    Memo,
    Plan,
}

/// Content payload for generated artifacts. Each variant wraps the
/// concrete type from existing datasynth models.
pub enum ArtifactContent {
    MaterialityCalculation(MaterialityCalculation),
    RiskAssessment(RiskAssessment),
    GoingConcernAssessment(GoingConcernAssessment),
    SamplingPlan(SamplingPlan),
    SampledItems(Vec<SampledItem>),
    AnalyticalProcedure(AnalyticalProcedure),
    EngagementLetter(EngagementLetter),
    AuditOpinion(AuditOpinion),
    Workpaper(Workpaper),
    AuditEvidence(AuditEvidence),
    AuditFinding(AuditFinding),
    ControlEvaluation(CombinedRiskAssessment),
    SubsequentEvent(SubsequentEvent),
    /// Generic content for step actions not mapped to a specific model.
    Generic { title: String, description: String, fields: HashMap<String, String> },
}
```

The `StepExecutor` maps `step.action` to artifact content generation:

| Action | Artifact Type | Generated From |
|--------|--------------|----------------|
| `evaluate` | RiskAssessment, GoingConcernAssessment, ControlEvaluation | Existing risk/going_concern generator utils |
| `determine` | MaterialityCalculation, TrivialThreshold | Existing materiality generator utils |
| `perform` | TestResults, SamplingResults, AnalyticalProcedure | Existing sampling/analytical generator utils |
| `design` | AuditPlan, SamplingPlan, ProcedureDesign | Existing plan generator utils |
| `agree` | EngagementLetter | Existing engagement_letter generator utils |
| `identify` | RiskRegister, ControlDeficiency | Existing risk/finding generator utils |
| `assess` | RiskMatrix, SignificantRiskAssessment | Existing CRA generator utils |
| `issue` | AuditOpinion, AuditReport | Existing opinion generator utils |
| `understand` | EntityUnderstandingWorkpaper | New: generates entity understanding documentation |

### Event Trail

```rust
pub struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub event_type: String,                   // from transition.emits or step.emits
    pub procedure_id: String,
    pub step_id: Option<String>,
    pub phase_id: String,
    pub from_state: Option<String>,           // for transition events
    pub to_state: Option<String>,
    pub actor_id: String,
    pub command: String,
    pub evidence_refs: Vec<String>,
    pub standards_refs: Vec<String>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AuditAnomalyType>,
}
```

**Flat JSON exporter**: Writes `audit/audit_event_trail.json` — array of `AuditEvent` objects, ordered by timestamp. This is a new output file added to the audit output directory.

**OCEL 2.0 projection**: Maps to existing `datasynth-ocpm` format:
- Object types = procedures (`accept_engagement`, `planning_materiality`, etc.)
- Activities = transition commands + step commands
- Objects = evidence artifacts with lifecycle
- Events = `AuditEvent` mapped to OCEL event format

### Configuration Integration

New section in main config YAML, nested under existing `audit`:

```yaml
audit:
  enabled: true
  fsm:
    enabled: true
    blueprint: builtin:fsa              # builtin:fsa | builtin:ia | /path/to/custom.yaml
    overlay: builtin:default            # builtin:default | builtin:thorough | builtin:rushed | /path
    depth: standard                     # overrides blueprint default
    discriminators:
      tiers: [core]
    event_trail:
      flat_log: true
      ocel_projection: true
    seed: null                          # null = use global seed
```

**Coexistence semantics**: When `audit.enabled: true` and `audit.fsm.enabled: true`, the FSM engine is the sole generator for audit artifacts. The old standalone generators do NOT also run. When `audit.enabled: true` and `audit.fsm.enabled: false` (or omitted), the old generators run as before. This ensures backward compatibility during the migration period (Phases 1-3). In Phase 4, `audit.fsm.enabled` becomes the default and the old code paths are deprecated.

Builtin blueprints embedded via `include_str!()` in `datasynth-audit-fsm`. Custom paths resolved relative to config file location.

### Optimizer Crate

```rust
// datasynth-audit-optimizer

/// Convert blueprint to directed graph for analysis.
pub fn blueprint_to_graph(blueprint: &AuditBlueprint) -> DiGraph<StateNode, TransitionEdge>;

/// Find minimum-transition path from all initial states to all terminal states.
pub fn shortest_path(graph: &DiGraph<StateNode, TransitionEdge>) -> PathResult;

/// Find optimal path satisfying constraints (must-visit procedures, coverage %).
pub fn constrained_path(
    graph: &DiGraph<StateNode, TransitionEdge>,
    constraints: &PathConstraints,
) -> Result<PathResult, InfeasibleError>;

/// Run N stochastic walks, return outcome distribution analysis.
pub fn monte_carlo(
    blueprint: &AuditBlueprint,
    overlay: &GenerationOverlay,
    iterations: usize,
    seed: u64,
) -> MonteCarloReport;

pub struct MonteCarloReport {
    pub iterations: usize,
    pub opinion_distribution: HashMap<String, f64>,  // opinion type → frequency
    pub avg_duration_hours: f64,
    pub avg_events: f64,
    pub bottleneck_procedures: Vec<(String, f64)>,    // procedure_id, avg_time_spent
    pub revision_hotspots: Vec<(String, f64)>,        // procedure_id, avg_revision_count
    pub happy_path: Vec<String>,                       // most common transition sequence
}
```

Dependencies: `datasynth-audit-fsm` (schema types only), `petgraph`, `rand`.

### Crate Layout

```
crates/
├── datasynth-audit-fsm/
│   ├── Cargo.toml
│   ├── blueprints/
│   │   ├── generic_fsa.yaml
│   │   └── generic_ia.yaml
│   ├── overlays/
│   │   ├── default.yaml
│   │   ├── thorough.yaml
│   │   └── rushed.yaml
│   └── src/
│       ├── lib.rs
│       ├── schema.rs                 # All Rust types for blueprint + overlay
│       ├── loader.rs                 # YAML parsing, validation, builtin resolution
│       ├── engine.rs                 # AuditFsmEngine, EngagementState, execution loop
│       ├── context.rs                # EngagementContext definition
│       ├── step_executor.rs          # StepExecutor, artifact generation dispatch
│       ├── event.rs                  # AuditEvent, builders
│       ├── anomaly.rs                # AuditAnomalyType, AuditAnomalyRecord, injection logic
│       ├── error.rs                  # AuditFsmError, ValidationViolation
│       └── export/
│           ├── mod.rs
│           ├── flat_log.rs           # JSON event trail
│           └── ocel.rs               # OCEL 2.0 projection
├── datasynth-audit-optimizer/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── graph.rs                  # Blueprint → petgraph conversion
│       ├── shortest_path.rs          # Dijkstra shortest path
│       ├── constrained.rs            # Constraint-based path optimization
│       ├── monte_carlo.rs            # Stochastic simulation
│       └── report.rs                 # Analysis output formatting
```

### Dependency Graph

```
datasynth-audit-fsm
├── depends on: datasynth-core (models, uuid_factory, distributions)
├── depends on: datasynth-standards (OpinionType, ISA references)
├── depends on: datasynth-generators (extracted utility functions for artifact generation)
├── depends on: serde, serde_yaml, chrono, rand_chacha

datasynth-audit-optimizer
├── depends on: datasynth-audit-fsm (schema types only)
├── depends on: petgraph, rand

datasynth-runtime (existing, modified)
├── depends on: datasynth-audit-fsm (to invoke engine from orchestrator)

datasynth-config (existing, modified)
├── new audit.fsm config section added to schema
```

Note: `datasynth-audit-fsm` depends on `datasynth-generators` (not the reverse). The generators crate exposes public utility functions that the FSM's StepExecutor calls. This avoids circular dependencies. If the dependency on the full generators crate becomes heavy, we can extract a thin `datasynth-audit-utils` crate containing just the artifact generation utilities.

### Migration Strategy

**Phase 1 — FSA Blueprint (milestone 1)**:
- Create `datasynth-audit-fsm` crate with schema, loader, engine
- Implement FSM engine with FSA blueprint (`generic_fsa.yaml`)
- Build StepExecutor with action→artifact mapping
- Implement flat JSON event trail exporter
- Add `audit.fsm` config section
- Integration test: load FSA blueprint, run engagement, verify events + artifacts
- Old generators remain functional when `audit.fsm.enabled: false`

**Phase 2 — IA Blueprint (milestone 2)**:
- Load `generic_ia.yaml`, validate engine generalizes
- Add support for IA-specific patterns:
  - **C2CE lifecycle**: Condition-Criteria-Cause-Effect — an 8-state aggregate used in the `develop_findings` procedure. States: `not_started → condition_identified → criteria_mapped → cause_analyzed → effect_assessed → finding_drafted → management_responded → closed`. The engine must handle aggregates with more than the standard 4 states.
  - **Self-loops**: The `monitor_action_plans` procedure has `follow_up → follow_up` transitions (re-verify actions when remediation is incomplete). The engine must detect and bound self-loops via overlay max-iterations config.
  - **Discriminator filtering**: IA blueprints tag procedures with `categories` (financial, operational, compliance, it, fraud, esg), `risk_ratings`, and `engagement_types`. The engine filters procedures based on overlay-selected discriminators.
  - **Gated vs. continuous phases**: IA has 9 phases; some are gated (require specific procedures completed before entry), while continuous phases like `ethics_and_professionalism` and `quality_and_improvement` run in parallel with other phases (their gate `order` is negative, indicating they are always active). The engine must support parallel phase execution for continuous phases.
- Add actor behavior profiles from overlay
- Add OCEL 2.0 projection exporter
- Integration test: IA blueprint with various discriminator configurations

**Phase 3 — Optimizer (milestone 3)**:
- Create `datasynth-audit-optimizer` crate
- Implement graph conversion, shortest path, constrained path
- Implement Monte Carlo simulation
- CLI subcommand: `datasynth-data audit optimize --blueprint builtin:fsa`

**Phase 4 — Cleanup & Integration (milestone 4)**:
- Refactor existing audit generators into utility functions:
  - Core generators (materiality, risk, sampling, analytical, going_concern, subsequent_event, engagement_letter, opinion, finding, workpaper, evidence, judgment, CRA, confirmation) become `pub fn` utilities in their existing modules, callable by StepExecutor
  - Support generators (component_audit, sox, service_org, internal_audit, related_party) remain as-is initially, wired in as StepExecutor extensions for ISA 600/402/SOX overlays
  - Generator utility interface uses trait `ArtifactGenerator` for mockability in tests:
    ```rust
    pub trait ArtifactGenerator {
        fn generate(&self, step: &BlueprintStep, ctx: &EngagementContext, rng: &mut ChaCha8Rng) -> Vec<GeneratedArtifact>;
    }
    ```
- Wire FSM engine into main `GenerationOrchestrator`
- Set `audit.fsm.enabled: true` as default in presets
- Deprecate standalone audit generator entry points (mark `#[deprecated]`)
- Update CLI `generate` command
- Update documentation and config presets
- Regression test: compare FSM-generated artifacts against baseline from old generators to verify functional parity

### Testing Strategy

- **Unit tests**: Schema deserialization roundtrip, loader validation rules, FSM transition logic, event emission, anomaly injection
- **Property tests**: Determinism (same seed = same output), precondition ordering never violated, phase gate enforcement, all generated events reference valid procedure/step IDs
- **Integration tests**: Full engagement run with FSA blueprint, verify artifact counts, event ordering, standards coverage
- **Snapshot tests**: Known blueprint + overlay + seed → expected event trail (golden file comparison)
- **Regression tests** (Phase 4): Compare FSM-generated artifacts against old generator output for structural parity (same artifact types, similar counts, valid cross-references)
- **Optimizer tests**: Known FSM graph → expected shortest path, Monte Carlo convergence
- **Loader validation tests**: Reject blueprints with cycles in precondition DAG, reject blueprints where step references nonexistent evidence catalog entry, reject blueprints with unreachable states, reject blueprints with invalid actor references

### Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Blueprint schema evolves in AuditMethodology | Schema mismatch | Version field in schema; loader validates `schema_version` compatibility |
| Existing generators have implicit dependencies | Refactor breaks generation | Phase 4 is last; generators work standalone until then |
| IA blueprint complexity (82 steps) causes performance issues | Slow generation | Topological sort + lazy step execution; profile after Phase 2 |
| Custom YAML blueprints with invalid structure | Runtime errors | Comprehensive loader validation with clear error messages |
| Dependency on full generators crate is heavy | Slow compile times | Extract `datasynth-audit-utils` if needed; monitor compile times |
| `include_str!()` blueprint size | Binary bloat | Current blueprints are <200KB total; acceptable. Monitor on additions |
