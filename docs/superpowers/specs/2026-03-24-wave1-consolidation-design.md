# Wave 1: Consolidation (v1.6.0) Design Spec

**Date**: 2026-03-24
**Status**: Approved
**Scope**: Make the audit FSM engine production-ready — CLI tooling, IA dispatch enrichment, graph integration depth, demo mode.

## Problem

The audit FSM engine (v1.5.0) produces events and artifacts but lacks:
- CLI commands to validate, inspect, and run blueprints standalone
- Comprehensive IA command dispatch (43/140 commands mapped, 30.7%)
- Audit-specific graph nodes and evidence-chain edges in the hypergraph
- FSM-enabled demo mode for easy evaluation

## Solution

Four work areas in a single release, all modifying existing crates (no new crates).

---

## 1. CLI Audit Subcommand

### New subcommand: `datasynth-data audit`

Three actions:

```
datasynth-data audit validate --blueprint builtin:fsa
datasynth-data audit validate --blueprint /path/to/custom.yaml
datasynth-data audit info --blueprint builtin:ia
datasynth-data audit run --blueprint builtin:fsa --overlay builtin:rushed --output ./audit_output
```

### `audit validate`

Loads a blueprint (builtin or custom path), runs `validate_blueprint_with_preconditions()`, and reports:
- On success: "Valid" with summary (procedure count, phase count, step count, standards count)
- On failure: List of `ValidationViolation` with location and message

Exit code 0 on success, 1 on validation failure.

### `audit info`

Loads a blueprint and prints a structured summary:

```
Blueprint: Generic Internal Audit Methodology (IIA-GIAS)
Version:   2025.1
Framework: IIA-GIAS

Phases:          9 (3 continuous, 6 sequential)
Procedures:      34
Steps:           82
Standards:       52
Actors:          8
Evidence types:  28

Command coverage:
  Mapped to generators:  80/140 (57.1%)
  Generic workpaper:     52/140 (37.1%)
  FSM lifecycle only:     8/140 (5.7%)

Phase breakdown:
  ethics_and_professionalism    (continuous)  4 procedures, 16 steps
  governance_establishment      (continuous)  3 procedures, 12 steps
  ...
```

### `audit run`

Standalone FSM execution without the full orchestrator:
1. Load blueprint + overlay from args
2. Create `EngagementContext::test_default()` (or from optional `--config` file)
3. Run `AuditFsmEngine::run_engagement()`
4. Export event trail to `<output>/audit_event_trail.json`
5. Export artifacts to `<output>/audit_artifacts.json` (summary counts)
6. Print summary to stdout

### Demo mode FSM integration

In `--demo` mode, enable FSM when audit is configured:
```rust
// In demo config construction
audit: AuditGenerationConfig {
    enabled: true,
    fsm: Some(AuditFsmConfig {
        enabled: true,
        blueprint: "builtin:fsa".into(),
        overlay: "builtin:default".into(),
        ..Default::default()
    }),
    ..Default::default()
}
```

### Files modified
- `crates/datasynth-cli/src/main.rs` — add `Audit` variant to `Commands` enum with `AuditAction` sub-enum
- `crates/datasynth-cli/src/main.rs` — add `handle_audit_command()` function
- `crates/datasynth-cli/src/main.rs` — update demo config to enable FSM

---

## 2. IA Dispatch Enrichment

### Goal

Extend StepDispatcher from 43 to ~80 explicitly mapped commands. The remaining ~60 are FSM lifecycle transitions (`activate`, `submit_for_review`, `cycle_back`, `complete_cycle`, `start_*`, `approve_*`, `submit_*`, `finalize_*`) that correctly produce generic workpapers.

### New dispatch categories

**Judgment/Quality → JudgmentGenerator → ProfessionalJudgment**
```
review_engagement_quality, exercise_skepticism, apply_due_care,
supervise_engagement_quality, develop_engagement_conclusions,
conduct_periodic_assessment, oversee_qaip
```

**Documentation → EvidenceGenerator → AuditEvidence**
```
document_engagement_work, archive_engagement_documentation,
protect_information, establish_confidentiality_policies
```

**Planning/Scoping → WorkpaperGenerator (Planning) → Workpaper**
```
define_engagement_scope, determine_engagement_timeline,
draft_annual_plan, develop_work_program, scope_engagement,
develop_staffing_plan, develop_ia_budget, assign_team_members,
determine_sme_needs, confirm_resource_competencies,
identify_auditable_entities, prioritize_audit_entities
```

**Reporting → WorkpaperGenerator (Reporting) → Workpaper**
```
draft_audit_report, review_draft_report, prepare_draft_report,
evaluate_management_responses, send_report_for_response,
receive_response, distribute_final_report, communicate_approved_plan,
present_plan_to_board, communicate_plan_and_results
```

**Follow-up/Monitoring → FindingGenerator → AuditFinding**
```
track_action_plan_status, escalate_overdue_actions,
report_follow_up_status, verify_remediation_implementation,
conclude_on_remediation
```

**Ethics/Governance → WorkpaperGenerator (Planning) → Workpaper**
```
establish_ethics_code, conduct_ethics_training, monitor_ethics_compliance,
assess_objectivity_threats, implement_objectivity_safeguards,
disclose_impairments, assess_competencies, develop_cpd_plans,
verify_standards_conformance, maintain_ethical_standards,
safeguard_objectivity, establish_independence, define_ia_mandate,
draft_ia_charter, establish_board_interaction, obtain_board_support
```

**Performance → WorkpaperGenerator (Completion) → Workpaper**
```
define_performance_metrics, track_performance,
report_performance_to_board, measure_ia_performance,
perform_ongoing_monitoring, evaluate_staff_performance,
monitor_budget_utilization
```

**Technology → WorkpaperGenerator (Planning) → Workpaper**
```
assess_technology_capabilities, implement_technology,
manage_technological_resources
```

### Updated `section_for_command`

Improve keyword matching to assign appropriate `WorkpaperSection` for IA commands:

| Keywords | Section |
|----------|---------|
| `universe`, `plan`, `scope`, `timeline`, `allocation`, `budget`, `staffing`, `technology`, `charter`, `mandate`, `ethics`, `independence`, `objectivity`, `competency`, `confidentiality` | Planning |
| `test`, `execute`, `perform`, `sample` | SubstantiveTesting |
| `control`, `evaluate_control` | ControlTesting |
| `finding`, `condition`, `criteria`, `cause`, `effect`, `recommendation`, `monitoring`, `follow_up`, `remediation`, `verification`, `conclusion`, `performance`, `quality` | Completion |
| `report`, `draft`, `issue`, `distribute`, `communicate`, `present` | Reporting |

### Files modified
- `crates/datasynth-audit-fsm/src/dispatch.rs` — expand match arms, update `section_for_command`

---

## 3. Graph Integration Depth

### New audit node types

Allocate hypergraph type codes 366-375:

| Code | Type | Source |
|------|------|--------|
| 366 | MaterialityCalculation | ArtifactBag |
| 367 | CombinedRiskAssessment | ArtifactBag |
| 368 | SamplingPlan | ArtifactBag |
| 369 | AnalyticalProcedureResult | ArtifactBag |
| 370 | EngagementLetter | ArtifactBag |
| 371 | GoingConcernAssessment | ArtifactBag |
| 372 | SubsequentEvent | ArtifactBag |
| 373 | AuditOpinion | ArtifactBag |
| 374 | KeyAuditMatter | ArtifactBag |
| 375 | ExternalConfirmation | ArtifactBag |

### New audit edge types

Evidence-chain and structural edges:

| Edge | From → To | Semantics |
|------|-----------|-----------|
| DOCUMENTED_BY | Engagement → Workpaper | Workpaper documents engagement |
| SUPPORTED_BY | Workpaper → Evidence | Evidence supports workpaper conclusions |
| IDENTIFIED_FROM | Finding → RiskAssessment | Finding identified from risk assessment |
| EVIDENCED_BY | Finding → Evidence | Finding evidenced by audit evidence |
| BASED_ON | AuditOpinion → Finding | Opinion based on aggregated findings |
| HIGHLIGHTS | AuditOpinion → KeyAuditMatter | Opinion highlights key audit matter |
| SCOPES | MaterialityCalculation → Engagement | Materiality scopes the engagement |
| RESPONDS_TO | SamplingPlan → CombinedRiskAssessment | Sampling responds to assessed risk |

### Extended `add_audit_documents`

Update the method signature to accept the additional artifact types from `AuditSnapshot`. Create nodes for each new type and add edges based on cross-references (engagement_id, workpaper_id, etc.).

### Files modified
- `crates/datasynth-graph/src/builders/hypergraph.rs` — extend `add_audit_documents`, add node creation and edge creation for new types

---

## 4. Testing & Verification

### CLI tests
- `datasynth-data audit validate --blueprint builtin:fsa` exits 0
- `datasynth-data audit validate --blueprint builtin:ia` exits 0
- `datasynth-data audit info --blueprint builtin:ia` output contains "34" procedures and "82" steps
- `datasynth-data audit run --blueprint builtin:fsa --output /tmp/test` produces event trail file

### IA dispatch tests
- IA engagement produces `total_artifacts() > 500` (up from current 1,891 which includes bulk sampling)
- IA engagement produces `judgments.len() > 0`
- IA engagement produces workpapers in at least 4 different sections

### Graph tests
- Hypergraph with audit artifacts has > 6 node types (currently 6, target 10+)
- Hypergraph has audit edge types (DOCUMENTED_BY, BASED_ON, etc.)

### Demo test
- `datasynth-data generate --demo` with FSM enabled produces `audit/fsm_event_trail.json`

### All tests
- `--test-threads=1` for all test runs (system resource constraint)
- Sequential crate testing only

---

## Migration / Compatibility

- `audit.fsm.enabled: false` (default) — no behavior change, old generators run as before
- `audit.fsm.enabled: true` — FSM engine runs, artifacts flow through standard pipeline
- New CLI `audit` subcommand is additive — no existing commands change
- Graph type codes 366-375 are additive — existing codes unchanged
- Demo mode change is backward-compatible (adds FSM alongside existing audit generation)

---

## Dependencies

No new crate dependencies. All work modifies existing crates:
- `datasynth-cli` (CLI subcommand, demo mode)
- `datasynth-audit-fsm` (dispatch enrichment)
- `datasynth-graph` (hypergraph extension)

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| IA dispatch changes alter deterministic output | Existing tests break | Run evaluation before/after, update golden values |
| Graph type code conflicts | Hypergraph corruption | Verify codes 366-375 are unallocated |
| Demo mode with FSM is slow | Poor first impression | FSA blueprint only (51 events, fast) |
| CLI subcommand adds binary size | Minimal | FSM crate already linked |
