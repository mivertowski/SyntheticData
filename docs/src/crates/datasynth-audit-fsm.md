# datasynth-audit-fsm

YAML-driven audit FSM engine for methodology-based audit trail and artifact generation.

## Overview

`datasynth-audit-fsm` separates audit methodology definition from data generation. Audit workflows are declared as YAML **blueprints** that describe procedures, state machines, phases, and standards references. A separate **generation overlay** controls runtime behaviour: revision probabilities, timing distributions, artifact volumes, and anomaly injection rates. This two-layer architecture means the same blueprint can produce a thorough engagement, a rushed engagement, or anything in between by swapping a single overlay file.

The engine walks each procedure's finite state machine in topological (DAG) order, emitting a deterministic, event-sourced audit trail. Every event carries a UUID generated from a ChaCha8 RNG seed, so identical inputs always produce identical outputs. Alongside the event trail, the engine dispatches step commands to 14 concrete audit generators via the `StepDispatcher`, producing typed artifacts (engagements, materiality calculations, risk assessments, workpapers, findings, opinions, and more).

Two built-in blueprints ship with the crate: a Financial Statement Audit (FSA) aligned to ISA standards and an Internal Audit (IA) aligned to IIA-GIAS. Additional methodology blueprints are available at [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints).

## Architecture

```
Blueprint YAML ──► Loader ──► Validation ──► Topological Sort
                                                    │
                              EngagementContext ─────┤
                                                    ▼
                                             AuditFsmEngine
                                                    │
                                     ┌──────────────┼──────────────┐
                                     ▼              ▼              ▼
                              FSM Walk       StepDispatcher    Anomaly Injection
                            (per procedure)   (14 generators)
                                     │              │              │
                                     ▼              ▼              ▼
                              AuditEvent[]    ArtifactBag    AnomalyRecord[]
                                     │              │
                            ┌────────┴────────┐     │
                            ▼                 ▼     ▼
                       Flat JSON         OCEL 2.0   Orchestrator
                       Event Trail      Projection  AuditSnapshot
```

## Blueprints

Two built-in blueprints are included:

| Blueprint | Procedures | Phases | Steps | Standards | Events (default) | Artifacts (default) |
|-----------|-----------|--------|-------|-----------|------------------|---------------------|
| Financial Statement Audit (FSA) | 9 | 3 | 24 | 14 ISA | 51 | 1,916 |
| Internal Audit (IA) | 34 | 9 | 82 | 52 IIA-GIAS | 368 | 1,891 |

A blueprint YAML defines:

- **methodology** -- framework identifier, default depth, and description
- **discriminators** -- top-level dimensions (categories, risk_ratings, engagement_types) used to scope procedure execution
- **actors** -- role definitions (engagement_partner, senior_auditor, audit_staff)
- **standards** -- referenced standards with binding level (requirement vs guidance)
- **evidence_templates** -- reusable evidence types that steps can reference
- **phases** -- ordered audit phases containing procedures
- **procedures** -- each with its own FSM aggregate and ordered steps

Additional blueprints are maintained in the [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints) repository.

### Procedure State Machines

Each procedure defines a `ProcedureAggregate` with an initial state, valid states, and directed transitions. Transitions carry optional commands, emitted events, and guard predicates.

The standard 4-state lifecycle used by most procedures:

```
not_started ──begin──► in_progress ──submit──► under_review ──approve──► completed
                            ▲                        │
                            └────────revise───────────┘
```

The `under_review -> in_progress` revision loop is governed by the overlay's `revision_probability`. The engine bounds self-loops via `max_self_loop_iterations` (default 5).

The `develop_findings` procedure in the IA blueprint uses an expanded 8-state C2CE (Condition-Criteria-Cause-Effect) lifecycle:

```
not_started → in_progress → condition_identified → criteria_mapped →
cause_analyzed → effect_assessed → under_review → completed
```

### Phase Gates and Preconditions

Procedures are executed in topological order determined by a precondition DAG. Kahn's algorithm produces the execution sequence. Phase gates use `all_of` conditions (e.g., `procedure.risk_assessment.completed`) to control phase entry and exit.

Continuous phases have `order < 0` and run in parallel with sequential phases. They are never marked as "completed" in the output. The IA blueprint uses this for ethics, governance, and quality assurance phases.

## Generation Overlays

An overlay customises how a blueprint is instantiated without modifying the canonical YAML. It controls:

- **Transition timing** -- log-normal delay distribution (mu/sigma in hours)
- **Revision probability** -- chance that a completed step returns to in_progress
- **Artifact volumes** -- workpapers per step, evidence items per workpaper
- **Anomaly injection** -- per-type probabilities (skipped approval, late posting, missing evidence, out of sequence)
- **Actor profiles** -- per-role multipliers for revision rate, evidence volume, and guidance step skipping
- **Discriminator filters** -- restrict which procedures execute based on category dimensions

Three built-in presets:

| Parameter | `default` | `thorough` | `rushed` |
|-----------|-----------|------------|----------|
| Revision probability | 0.15 | 0.30 | 0.05 |
| Timing mu (hours) | 24.0 | 40.0 | 8.0 |
| Timing sigma (hours) | 8.0 | 12.0 | 4.0 |
| Workpapers per step | 1-3 | 2-5 | 1-2 |
| Evidence per workpaper | 2-5 | 4-8 | 1-3 |
| Skipped approval | 0.02 | 0.005 | 0.08 |
| Late posting | 0.05 | 0.02 | 0.15 |
| Missing evidence | 0.03 | 0.01 | 0.10 |
| Out of sequence | 0.01 | 0.002 | 0.05 |

## StepDispatcher and Artifact Generation

The `StepDispatcher` bridges FSM step commands to concrete generators. Each step command is routed to the appropriate generator, and the resulting artifacts are accumulated in an `ArtifactBag`. Unknown commands fall through to a generic workpaper generator so every step produces at least one artifact.

Key command-to-generator mappings:

| Commands | Generator | Artifact Types |
|----------|-----------|----------------|
| `evaluate_client_acceptance`, `conduct_opening_meeting` | `AuditEngagementGenerator` | `AuditEngagement` |
| `agree_engagement_terms`, `draft_ia_charter` | `EngagementLetterGenerator` | `EngagementLetter` |
| `determine_overall_materiality` | `MaterialityGenerator` | `MaterialityCalculation` |
| `identify_risks`, `assess_engagement_risks` | `RiskAssessmentGenerator` | `RiskAssessment` |
| `assess_risks`, `evaluate_control_effectiveness` | `CraGenerator` | `CombinedRiskAssessment` |
| `design_test_procedures`, `design_work_program` | `WorkpaperGenerator` + `EvidenceGenerator` | `Workpaper`, `AuditEvidence` |
| `perform_tests_of_details`, `perform_controls_tests` | `SamplingPlanGenerator` | `SamplingPlan`, `SampledItem` |
| `perform_analytical_procedures` | `AnalyticalProcedureGenerator` | `AnalyticalProcedureResult` |
| `send_confirmations` | `ConfirmationGenerator` | `ExternalConfirmation`, `ConfirmationResponse` |
| `evaluate_management_assessment` | `GoingConcernGenerator` | `GoingConcernAssessment` |
| `perform_subsequent_events_review` | `SubsequentEventGenerator` | `SubsequentEvent` |
| `identify_condition`, `draft_finding` | `FindingGenerator` | `AuditFinding` |
| `form_audit_opinion`, `finalize_audit_report` | `AuditOpinionGenerator` | `AuditOpinion`, `KeyAuditMatter` |

For IA blueprints (which lack `evaluate_client_acceptance`), the dispatcher auto-bootstraps an engagement the first time a substantive command runs.

## Event Trail

Each event in the trail captures a state transition or procedure step:

```rust
pub struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub event_type: String,        // "state_transition" or "procedure_step"
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
```

Events are exported as flat JSON via `export_events_to_json()` and can be projected to OCEL 2.0 format via `project_to_ocel()` for use with process mining tools (PM4Py, Celonis, OCPA).

## Configuration

```yaml
audit:
  enabled: true
  fsm:
    enabled: true
    blueprint: builtin:fsa    # builtin:fsa, builtin:ia, or path to custom YAML
    overlay: builtin:default   # builtin:default, builtin:thorough, builtin:rushed
```

## Example Output

Sample events from an FSA engagement event trail:

```json
[
  {
    "event_id": "a1b2c3d4-...",
    "timestamp": "2025-01-15T09:00:00",
    "event_type": "state_transition",
    "procedure_id": "accept_engagement",
    "phase_id": "planning",
    "from_state": "not_started",
    "to_state": "in_progress",
    "actor_id": "engagement_partner",
    "command": "evaluate_client_acceptance",
    "is_anomaly": false
  },
  {
    "event_id": "e5f6g7h8-...",
    "timestamp": "2025-01-15T10:23:00",
    "event_type": "procedure_step",
    "procedure_id": "accept_engagement",
    "step_id": "evaluate_acceptance",
    "phase_id": "planning",
    "actor_id": "engagement_partner",
    "command": "evaluate_client_acceptance",
    "evidence_refs": ["wp_client_assessment"],
    "standards_refs": ["ISA-220"],
    "is_anomaly": false
  }
]
```

With the default overlay, the FSA blueprint produces 51 events and 1,916 artifacts. The IA blueprint produces 368 events and 1,891 artifacts.

## Key Types

```rust
// Blueprint root
pub struct AuditBlueprint {
    pub id: String,
    pub methodology: BlueprintMethodology,
    pub discriminators: HashMap<String, Vec<String>>,
    pub actors: Vec<BlueprintActor>,
    pub phases: Vec<BlueprintPhase>,
}

// FSM engine
pub struct AuditFsmEngine { /* blueprint, overlay, rng, preconditions, dispatcher */ }

// Engagement output
pub struct EngagementResult {
    pub event_log: Vec<AuditEvent>,
    pub procedure_states: HashMap<String, String>,
    pub anomalies: Vec<AuditAnomalyRecord>,
    pub artifacts: ArtifactBag,
    pub total_duration_hours: f64,
}

// Artifact accumulator (20 typed collections)
pub struct ArtifactBag {
    pub engagements: Vec<AuditEngagement>,
    pub materiality_calculations: Vec<MaterialityCalculation>,
    pub risk_assessments: Vec<RiskAssessment>,
    pub workpapers: Vec<Workpaper>,
    pub findings: Vec<AuditFinding>,
    pub audit_opinions: Vec<AuditOpinion>,
    // ... 14 more artifact types
}
```

## See Also

- [Audit FSM Engine Deep Dive](../advanced/audit-fsm-engine.md)
- [datasynth-audit-optimizer](datasynth-audit-optimizer.md)
- [Audit Analytics](../use-cases/audit-analytics.md)
- [Accounting & Audit Standards](../advanced/accounting-standards.md)
