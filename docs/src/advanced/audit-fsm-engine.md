# Audit FSM Engine

Deep dive into the YAML-driven audit finite state machine engine.

## Concept

Traditional audit data generators hard-code procedure sequences and artifact types in application logic. When a new methodology standard is published or a firm wants to model its own workflow, the code itself must change. The audit FSM engine inverts this: methodology blueprints are declarative YAML documents that define **what happens** (procedures, states, transitions, evidence requirements), while generation overlays define **how it happens** (probabilities, timing, volumes, anomalies). The engine is the runtime that walks the blueprints and dispatches work to concrete generators.

All output is event-sourced. Each state transition and procedure step emits an `AuditEvent` with a deterministic UUID derived from a ChaCha8 RNG. Given the same seed and inputs, the engine always produces the same event trail, the same artifacts, and the same anomaly labels. This makes the output suitable for regression testing, ML model training, and reproducible research.

The engine also produces concrete typed artifacts (engagements, materiality calculations, risk assessments, workpapers, sampling plans, findings, opinions, and more) via the `StepDispatcher`, which maps step commands to the 14 generators in `datasynth-generators`.

In addition to batch execution, the engine supports streaming execution (event-by-event emission via callbacks or mpsc channels) and live anomaly injection into already-generated event logs. A blueprint testing framework provides automated validation that blueprints produce expected artifact types and event counts.

## Blueprint YAML Structure

A blueprint defines an audit methodology as a collection of phased procedures, each with its own embedded finite state machine.

### Top-Level Structure

```yaml
schema_version: "1.0"
methodology:
  framework: ISA
  description: "Generic financial statement audit"
  default_depth: standard        # simplified | standard | full

discriminators:
  categories: [financial, operational, compliance, strategic]
  risk_ratings: [high, medium, low]

actors:
  - id: engagement_partner
    label: "Engagement Partner"
  - id: audit_senior
    label: "Audit Senior"
  - id: audit_staff
    label: "Audit Staff"

standards_catalog:
  - id: ISA-220
    title: "Quality Management for an Audit"
    binding: requirement
  - id: ISA-315
    title: "Identifying and Assessing Risks of Material Misstatement"
    binding: requirement

evidence_catalog:
  - id: wp_client_assessment
    type: workpaper
    description: "Client acceptance/continuance assessment"
    required: true
```

### Phases and Procedures

```yaml
phases:
  - id: planning
    name: Planning
    order: 1
    entry_gate:
      all_of:
        - predicate: "procedure.accept_engagement.completed"
          rationale: "Must accept engagement before planning begins"
    procedures:
      - id: planning_materiality
        name: "Determine Materiality"
        preconditions: [accept_engagement]
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, under_review, completed]
          transitions:
            - from_state: not_started
              to_state: in_progress
              command: begin_materiality_assessment
              emits: MaterialityStarted
            - from_state: in_progress
              to_state: under_review
              command: submit_materiality
              emits: MaterialitySubmitted
            - from_state: under_review
              to_state: completed
              command: approve_materiality
              emits: MaterialityApproved
            - from_state: under_review
              to_state: in_progress
              command: revise_materiality
              emits: MaterialityRevised
        steps:
          - id: determine_materiality
            name: "Determine overall materiality"
            actor: audit_senior
            command: determine_overall_materiality
            binding: requirement
            evidence:
              - type: workpaper
                required: true
                direction: produces
            standards:
              - ref: ISA-320
                paragraph: "10-11"
```

### Key Sections

| Section | Purpose |
|---------|---------|
| `methodology` | Framework ID, depth, description |
| `discriminators` | Dimensions for scoping procedure execution |
| `actors` | Role definitions with labels |
| `standards_catalog` | Referenced standards (binding: requirement/guidance) |
| `evidence_catalog` | Reusable evidence templates |
| `phases` | Ordered groups with entry/exit gates |
| `procedures` | FSM aggregates + ordered steps |
| `steps` | Atomic work units with commands, evidence, standards |

## How the Engine Works

### 1. Loading and Validation

Nine built-in blueprints are available:

| Blueprint | Loader | Framework | Procedures | Phases |
|-----------|--------|-----------|-----------|--------|
| Financial Statement Audit (FSA) | `load_builtin_fsa()` | ISA | 9 | 3 |
| Internal Audit (IA) | `load_builtin_ia()` | IIA-GIAS | 34 | 9 |
| KPMG ISA Complete | `load_builtin_kpmg()` | ISA | 44 | 7 |
| PwC ISA Complete | `load_builtin_pwc()` | ISA | 44 | 7 |
| Deloitte ISA Complete | `load_builtin_deloitte()` | ISA | 46 | 7 |
| EY GAM Lite | `load_builtin_ey_gam_lite()` | ISA | 52 | 7 |
| SOC 2 Type II | `load_builtin_soc2()` | AICPA-TSC | 12 | 3 |
| PCAOB Integrated | `load_builtin_pcaob()` | PCAOB | 14 | 5 |
| Regulatory Exam | `load_builtin_regulatory()` | Regulatory | 15 | 6 |

All loaders are methods on `BlueprintWithPreconditions`. Custom YAML files are loaded via `BlueprintSource::Custom(path)` or `BlueprintSource::Raw(yaml_string)`. The loader:

1. Deserializes the raw YAML into intermediate types
2. Converts to the canonical `AuditBlueprint` schema
3. Extracts procedure preconditions into a separate map
4. Validates cross-references (all standard refs resolve, all evidence template refs resolve)
5. Checks for DAG cycles in the precondition graph

Overlays are loaded independently via `load_overlay()` and applied at engine construction time.

### 2. Topological Sort

The engine uses Kahn's algorithm to compute execution order from the precondition DAG. Each procedure declares its dependencies via `preconditions: [...]`, and the sort guarantees that all prerequisites complete before a dependent procedure begins.

Continuous phases (those with `order < 0`) participate in the sort but are never blocked by phase gates. They model activities like ethics compliance, quality assurance, and governance oversight that run throughout the engagement.

### 3. FSM Walk

For each procedure in topological order:

1. **Initialize** at the procedure's `initial_state`
2. **Select transition**: if multiple outgoing edges exist, the engine uses the overlay's `revision_probability` to decide between forward and revision (backward) transitions
3. **Self-loop detection**: if the same `(from_state, to_state)` repeats, a counter increments; when it reaches `max_self_loop_iterations` (default 5), the engine forces a forward transition
4. **Step execution**: when entering `in_progress` (or on a self-loop), all steps within the procedure execute in order, each emitting a `procedure_step` event
5. **Time advancement**: a log-normal distributed delay is sampled between transitions (mu/sigma from the overlay); a shorter delay is applied between steps within a procedure
6. **Repeat** until the procedure reaches a terminal state (no outgoing transitions)

The engine caps total iterations per procedure at 20 (`MAX_ITERATIONS`) as a safety bound.

### 4. Step Execution and Artifact Dispatch

When steps execute, the `StepDispatcher` maps each step's `command` string to a concrete generator. The dispatcher maintains 14 pre-initialized generators, each seeded with a deterministic offset from the base seed (7000-series discriminators).

The dispatch logic works as follows:

1. Look up the step's `command` string
2. Match against known command patterns (see table below)
3. Read prerequisite artifacts from the `ArtifactBag`
4. Build the generator-specific input from `EngagementContext`
5. Call the generator and push results into the bag
6. Unknown commands fall through to a generic workpaper generator

**14 generator types handle 40+ command mappings:**

| Generator | ISA Reference | Sample Commands |
|-----------|---------------|-----------------|
| `AuditEngagementGenerator` | ISA 210/220 | `evaluate_client_acceptance`, `conduct_opening_meeting` |
| `EngagementLetterGenerator` | ISA 210 | `agree_engagement_terms`, `draft_ia_charter` |
| `MaterialityGenerator` | ISA 320 | `determine_overall_materiality` |
| `RiskAssessmentGenerator` | ISA 315/330 | `identify_risks`, `assess_engagement_risks` |
| `CraGenerator` | ISA 315 | `assess_risks`, `evaluate_control_effectiveness` |
| `WorkpaperGenerator` | ISA 230 | `design_test_procedures`, `design_work_program` |
| `EvidenceGenerator` | ISA 500 | (produced alongside workpapers) |
| `SamplingPlanGenerator` | ISA 530 | `perform_tests_of_details`, `perform_controls_tests` |
| `AnalyticalProcedureGenerator` | ISA 520 | `perform_analytical_procedures` |
| `ConfirmationGenerator` | ISA 505 | `send_confirmations` |
| `GoingConcernGenerator` | ISA 570 | `evaluate_management_assessment` |
| `SubsequentEventGenerator` | ISA 560 | `perform_subsequent_events_review` |
| `FindingGenerator` | ISA 265 | `identify_condition`, `draft_finding`, `evaluate_misstatements` |
| `AuditOpinionGenerator` | ISA 700/701 | `form_audit_opinion`, `finalize_audit_report` |

For IA blueprints, the dispatcher auto-bootstraps an `AuditEngagement` when the bag is empty and a substantive command runs. FSM lifecycle commands (`activate`, `submit_for_review`, `cycle_back`, etc.) do not trigger this bootstrap.

### 5. Event Emission

Each transition and step emits an `AuditEvent` with:

- **Deterministic UUID** from ChaCha8 RNG bytes
- **Timestamp** advanced by log-normal delays
- **Evidence and standards references** from the step definition
- **Anomaly flag** if injection was triggered

Anomaly injection checks four types per step: `SkippedApproval`, `LatePosting`, `MissingEvidence`, and `OutOfSequence`. Each is independently rolled against its overlay probability.

## Generation Overlays

### Overlay YAML Structure

```yaml
# overlays/default.yaml
transitions:
  defaults:
    revision_probability: 0.15
    timing:
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
  skipped_approval: 0.02
  late_posting: 0.05
  max_delay_hours: 72.0
  missing_evidence: 0.03
  out_of_sequence: 0.01
```

The `rushed` overlay adds actor profiles that reduce evidence and skip guidance steps:

```yaml
actor_profiles:
  audit_staff:
    revision_multiplier: 0.5
    evidence_multiplier: 0.7
    skip_guidance_steps: true
```

### Built-in Presets Comparison

| Parameter | `default` | `thorough` | `rushed` |
|-----------|-----------|------------|----------|
| **Revision probability** | 0.15 | 0.30 | 0.05 |
| **Timing mu (hours)** | 24.0 | 40.0 | 8.0 |
| **Timing sigma (hours)** | 8.0 | 12.0 | 4.0 |
| **Workpapers per step** | 1-3 | 2-5 | 1-2 |
| **Evidence per workpaper** | 2-5 | 4-8 | 1-3 |
| **Skipped approval rate** | 0.02 | 0.005 | 0.08 |
| **Late posting rate** | 0.05 | 0.02 | 0.15 |
| **Missing evidence rate** | 0.03 | 0.01 | 0.10 |
| **Out of sequence rate** | 0.01 | 0.002 | 0.05 |
| **Max delay hours** | 72 | 24 | 120 |

**Impact on FSA output** (seed 42):

| Metric | `default` | `thorough` | `rushed` |
|--------|-----------|------------|----------|
| Events | 51 | 51 | 51 |
| Artifacts | 1,916 | ~3,200 | ~900 |
| Duration (hours) | ~776 | ~1,195 | ~284 |
| Anomalies | ~2 | ~0 | ~7 |

Event counts remain constant because they depend on the blueprint's procedure/transition structure (which is fixed). Duration, artifact volume, and anomaly counts vary based on overlay parameters.

## Custom Blueprints

To create a custom blueprint:

1. Start from an existing YAML (e.g., `generic_fsa.yaml` from [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints))
2. Define phases, procedures with FSM aggregates, and steps with commands
3. Ensure all standard and evidence template refs resolve
4. Avoid cycles in the precondition DAG
5. Load via `BlueprintSource::Custom(path)` or `BlueprintSource::Raw(yaml_string)`

Required structure:
- Every procedure must have a non-empty `initial_state`
- At least one terminal state (no outgoing transitions) must be reachable
- Standard 4-state lifecycle (`not_started`, `in_progress`, `under_review`, `completed`) is recommended but not required

## Discriminator Filtering

Discriminators scope procedure execution to specific engagement dimensions. The blueprint declares available dimensions at the top level and individual procedures declare which values they apply to.

IA blueprint example:

```yaml
# Blueprint-level
discriminators:
  categories: [financial, operational, compliance, strategic, IT]
  risk_ratings: [high, medium, low]
  engagement_types: [assurance, advisory, consulting]

# Procedure-level
procedures:
  - id: technology_assessment
    discriminators:
      categories: [IT]
      engagement_types: [assurance, advisory]
```

When the overlay specifies a discriminator filter (e.g., `categories: [financial]`), the engine skips procedures that do not match. Procedures with no discriminators are always executed.

## Output Formats

### Flat JSON Event Trail

The primary export format. Each event is a JSON object in an array:

```json
[
  {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2025-01-15T09:00:00",
    "event_type": "state_transition",
    "procedure_id": "accept_engagement",
    "step_id": null,
    "phase_id": "planning",
    "from_state": "not_started",
    "to_state": "in_progress",
    "actor_id": "engagement_partner",
    "command": "evaluate_client_acceptance",
    "evidence_refs": [],
    "standards_refs": [],
    "is_anomaly": false,
    "anomaly_type": null
  }
]
```

### OCEL 2.0 Projection

`project_to_ocel()` maps audit events to OCEL 2.0 format:

- **Object types**: each procedure ID becomes an object type; evidence references become `"evidence"` objects
- **Events**: mapped from audit events with `activity = command`, `omap` linking to procedure and evidence objects
- **Attributes**: phase, actor, from_state, to_state stored in `vmap`

```json
{
  "ocel_version": "2.0",
  "object_types": ["accept_engagement", "evidence", "planning_materiality", ...],
  "events": [
    {
      "id": "550e8400-...",
      "activity": "evaluate_client_acceptance",
      "timestamp": "2025-01-15 09:00:00",
      "omap": ["proc_accept_engagement"],
      "vmap": {
        "phase": "planning",
        "actor": "engagement_partner",
        "from_state": "not_started",
        "to_state": "in_progress"
      }
    }
  ],
  "objects": [
    {"id": "proc_accept_engagement", "object_type": "accept_engagement", "attributes": {}}
  ]
}
```

This output is compatible with PM4Py, Celonis, OCPA, and other OCEL 2.0 process mining tools.

## Orchestrator Integration

When `audit.fsm.enabled: true` in the DataSynth configuration:

1. The **enhanced orchestrator** builds an `EngagementContext` from the current generation state, populating financial data (revenue, assets, equity), team members, GL accounts, vendor/customer names, and configuration flags
2. The `AuditFsmEngine` runs the engagement
3. The `ArtifactBag` is mapped to the orchestrator's `AuditSnapshot`, which feeds the standard output pipeline (CSV, JSON, Parquet)
4. The graph builder receives the same typed artifacts for node/edge creation (28 entity types, 27 edge types)

The FSM engine operates alongside the existing audit generators. When FSM mode is enabled, it replaces the procedural audit generation path with the blueprint-driven approach.

## Streaming Execution

The `streaming` module provides two modes for event-by-event emission:

**Callback mode** — `run_engagement_streaming()` accepts a closure invoked for each `AuditEvent`:

```rust
use datasynth_audit_fsm::streaming::run_engagement_streaming;
use datasynth_audit_fsm::loader::{BlueprintWithPreconditions, default_overlay};
use datasynth_audit_fsm::context::EngagementContext;

let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
let overlay = default_overlay();
let ctx = EngagementContext::demo();

let result = run_engagement_streaming(
    &bwp, &overlay, &ctx, 42,
    Box::new(|event| {
        // Forward to WebSocket, Kafka, Grafana, etc.
    }),
).unwrap();
```

**Channel mode** — `run_engagement_to_channel()` spawns the engine on a background thread and returns an `mpsc::Receiver<AuditEvent>` for consumption by a separate reader thread, plus a `JoinHandle` for the full `EngagementResult`.

Both modes produce the same deterministic output as the batch API. The streaming API enables integration with continuous audit dashboards and real-time monitoring systems.

## Live Anomaly Injection

The `live_injection` module injects anomalies into an already-generated event log, simulating emerging risks after the initial generation. Each `LiveInjectionConfig` specifies:

- **anomaly_type** — the type of anomaly (`SkippedApproval`, `LatePosting`, `MissingEvidence`, `OutOfSequence`)
- **target_procedure** — optional filter to restrict injection to a specific procedure
- **injection_probability** — per-event probability (0.0–1.0)
- **severity** — `Low`, `Medium`, or `High`

Events already marked anomalous by the engine's build-time injection are skipped. The function returns a `Vec<AuditAnomalyRecord>` of the newly injected anomalies.

This enables scenarios such as: generate a clean engagement, then progressively inject anomalies at different rates to produce training data with calibrated difficulty levels.

## Blueprint Testing Framework

The `blueprint_testing` module in `datasynth-audit-optimizer` provides automated validation that a blueprint produces expected output. A `BlueprintTestSuite` specifies:

- Blueprint and overlay selectors
- `BlueprintExpectations` — minimum events, minimum artifacts, minimum completed procedures, and optional timing bounds

`test_blueprint()` runs a single suite and returns a `BlueprintTestResult` with pass/fail status and observed metrics. `test_all_builtins()` exercises every built-in blueprint against reasonable default expectations and returns results for each.

## Optimizer Integration

The companion `datasynth-audit-optimizer` crate provides analysis, simulation, and planning capabilities across 16 modules:

**Shortest path analysis**: BFS per procedure finds the minimum transitions from initial to terminal state. FSA requires 27 minimum transitions across 9 procedures; IA requires 101 across 34 procedures.

**Constrained path optimization**: given a set of must-visit procedures, expands the required set via transitive preconditions and returns filtered shortest paths. Useful for planning minimum-effort audit scopes.

**Monte Carlo simulation**: N stochastic walks through the FSM engine (requires `&EngagementContext`, returns `Result`), collecting:
- Bottleneck procedures (highest average event counts)
- Revision hotspots (most `under_review -> in_progress` loops)
- Happy path (procedure completion order)
- Duration distributions

```rust
use datasynth_audit_optimizer::monte_carlo::run_monte_carlo;
use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
use datasynth_audit_fsm::context::EngagementContext;

let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
let ctx = EngagementContext::demo();
let report = run_monte_carlo(&bwp, 1000, 42, &ctx).unwrap();
println!("Avg duration: {:.0}h", report.avg_duration_hours);
println!("Bottlenecks: {:?}", report.bottleneck_procedures);
```

**Year-over-year chains** (`yoy_chain`): simulate sequential engagements for the same entity across multiple fiscal years, with configurable finding carry-forward rates and trend tracking.

**Group audit** (`group_audit`): ISA 600 group audit simulation where each component entity runs its own FSM engagement, with findings and misstatement amounts consolidated at the group level. Components can be scoped as full, specific, analytical, or not-in-scope.

**Blueprint testing** (`blueprint_testing`): automated validation of blueprints against expected artifact counts, event thresholds, and phase progression. `test_all_builtins()` exercises every built-in blueprint.

## See Also

- [datasynth-audit-fsm Crate Reference](../crates/datasynth-audit-fsm.md)
- [datasynth-audit-optimizer Crate Reference](../crates/datasynth-audit-optimizer.md)
- [Audit Analytics Use Case](../use-cases/audit-analytics.md)
- [Accounting & Audit Standards](accounting-standards.md)
