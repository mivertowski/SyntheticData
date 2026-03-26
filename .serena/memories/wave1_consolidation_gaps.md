# Wave 1 Consolidation Gaps — Comprehensive Audit FSM Analysis

## Executive Summary

The audit FSM Phase 4 implementation has produced a functional foundation with 43 mapped step commands, an ArtifactBag with 20 artifact types, and successful orchestration of FSM→AuditSnapshot→OutputWriter pipeline. However, 5 critical gaps prevent full Wave 1 consolidation:

**Gap Categories:**
- **CLI Integration (1)**: FSM output not integrated into output_writer.rs
- **Blueprint Validation (1)**: No CLI tooling for FSM blueprint inspection
- **IA Dispatch (3)**: 108 unmapped IA commands + missing artifact types + unknown edge types in graph

---

## CRITICAL FINDINGS

### 1. CLI Integration Gaps (OUTPUT WRITER)

**File**: `/crates/datasynth-cli/src/output_writer.rs` (lines 626–633)

#### Current Implementation
- **FSM event trail** IS written: `result.audit.fsm_event_trail` → `audit/fsm_event_trail.json`
- All 20 ArtifactBag fields ARE present in AuditSnapshot and ARE written:
  - `engagements`, `engagement_letters`, `materiality_calculations`, `risk_assessments`, `combined_risk_assessments`
  - `workpapers`, `evidence`, `findings`, `judgments`
  - `sampling_plans`, `sampled_items`, `analytical_results`
  - `going_concern_assessments`, `subsequent_events`
  - `audit_opinions`, `key_audit_matters`
  - `procedure_steps`, `samples`, `confirmations`, `confirmation_responses`

#### Status: ✓ COMPLETE
No output writer gaps exist for FSM-generated artifacts. All fields flow through correctly.

---

### 2. Blueprint Validation Tooling Gaps

**Files**: 
- `/crates/datasynth-cli/src/main.rs` (Commands enum)
- `/crates/datasynth-audit-fsm/src/loader.rs` (public validation functions)

#### Current State
- `Commands` enum has: `Generate`, `Validate` (for config), `Init`, `Info`, `Verify`, `Fingerprint`, `Scenario`
- **NO dedicated audit/FSM subcommand**
- Loader provides public functions:
  - `load_blueprint(source: &BlueprintSource)` (line 587)
  - `validate_blueprint(bp: &AuditBlueprint)` (line 643)
  - `validate_blueprint_with_preconditions(...)` (line 1032)

#### What's Missing
1. **No `audit` or `fsm` CLI subcommand** to:
   - Validate blueprints: `datasynth-data audit validate --blueprint generic_ia.yaml`
   - List procedures: `datasynth-data audit info --blueprint generic_ia.yaml` → outputs:
     - Total step count (199 in generic_ia.yaml)
     - Phase/cycle breakdown (governance, planning, performing, monitoring)
     - All command names and their ISA mappings
     - Materiality and risk categorizations
     - Prerequisite tracking
   - Display coverage: `datasynth-data audit coverage --blueprint generic_ia.yaml` → shows:
     - % of commands mapped to generators vs. fallback
     - Missing artifact types
     - Unmapped IA generator requirements

2. **Demo preset does NOT enable FSM** by default
   - Line 2441: `audit: AuditGenerationConfig::default()` (FSM disabled)
   - `--audit` CLI flag enables standard audit but NOT FSM mode

#### Gap Impact
Users cannot easily understand:
- Which FSM blueprints are valid/loadable
- What coverage a blueprint provides
- Suitability of blueprints for different scenarios

---

### 3. IA Dispatch Gaps (STEP DISPATCHER)

**File**: `/crates/datasynth-audit-fsm/src/dispatch.rs` (lines 96–212, fallback at 208–210)

#### Command Coverage Summary
- **Total IA commands** in generic_ia.yaml: 140 unique commands across 199 steps
- **Mapped to generators** (explicit match): **43 commands**
  - Engagement (4): evaluate_client_acceptance, conduct_opening_meeting, conduct_meeting, define_engagement_scope
  - Letter/charter (3): agree_engagement_terms, draft_ia_charter, finalize_mandate
  - Materiality (1): determine_overall_materiality
  - Risk (4): identify_engagement_risks, assess_engagement_risks, assess_universe_risks, identify_risks
  - CRA (3): assess_risks, assess_control_design, evaluate_control_effectiveness
  - Workpaper design (5): design_test_procedures, design_work_program, design_controls_tests, design_substantive_procedures, design_analytical_procedures
  - Sampling (3): perform_test_procedures, perform_controls_tests, perform_tests_of_details
  - Analytical (1): perform_analytical_procedures
  - Confirmations (1): send_confirmations
  - Going concern (2): evaluate_management_assessment, determine_going_concern_doubt
  - Subsequent events (1): perform_subsequent_events_review
  - Findings (11): identify_condition, map_criteria, analyze_cause, assess_effect, draft_finding, identify_finding_condition, map_finding_criteria, analyze_cause_and_effect, evaluate_finding_significance, evaluate_findings, evaluate_misstatements
  - Opinion (4): form_audit_opinion, develop_engagement_conclusions, finalize_audit_report, issue_report

- **Fallback-only** (no dispatch, generic workpaper): **8 commands**
  - "" (empty command), start_universe_update, submit_universe, approve_universe, activate, submit_for_review, cycle_back, complete_cycle

- **Fall through to generic workpaper** (unknown): **89 commands** (63% of IA blueprint)
  - Administrative/workflow: approve_*, submit_*, start_*, finalize_*, revise_*, cycle_back, complete_*
  - Planning: draft_annual_plan, present_plan_to_board, communicate_approved_plan, identify_auditable_entities, prioritize_audit_entities
  - Risk/competency: assess_competencies, determine_sme_needs, assess_objectivity_threats, assess_technology_capabilities, confirm_resource_competencies, assign_team_members, develop_staffing_plan, confirm_cae_qualifications
  - Quality/ethics: conduct_ethics_training, establish_ethics_code, establish_independence, implement_objectivity_safeguards, exercise_skepticism, apply_due_care
  - Governance: establish_board_interaction, obtain_board_support, conduct_periodic_assessment, report_performance_to_board, oversee_qaip, commission_external_qa
  - Reporting: draft_audit_report, review_draft_report, revise_draft_report, distribute_final_report, discuss_findings, discuss_preliminary_findings, obtain_response, request_response, receive_response, approve_draft_report
  - Follow-up: initiate_follow_up, start_monitoring, perform_ongoing_monitoring, report_follow_up_status, track_action_plan_status, escalate_overdue_actions, close_finding, close_monitoring, re_verify_action, complete_verification, verify_remediation_implementation, conclude_on_remediation, agree_action_plans, evaluate_management_responses, document_risk_acceptance
  - Documentation: document_engagement_work, archive_engagement_documentation
  - Quality: review_engagement_quality, supervise_engagement_quality, track_performance, evaluate_staff_performance
  - Standards/technology: verify_standards_conformance, implement_technology, assess_technology_capabilities
  - Resource/budget: develop_ia_budget, monitor_budget_utilization, present_resource_requirements, recruit_and_develop_staff, develop_cpd_plans, develop_recommendations
  - Information protection: protect_information, establish_confidentiality_policies, disclose_impairments, disclose_errors_omissions
  - Scope/procedures: identify_stakeholders, identify_auditable_entities, prioritize_audit_entities, start_scoping, submit_scope, approve_scope, define_ia_mandate, finalize_mandate, start_procedure, approve_procedure, determine_engagement_timeline, determine_sme_needs, review_test_results, approve_testing, start_testing, request_additional_testing, submit_testing, approve_control_evaluation, start_control_evaluation, submit_control_evaluation, approve_risk_assessment, start_risk_assessment, submit_risk_assessment, start_allocation, finalize_allocation, start_plan_development, submit_plan, approve_plan, revise_plan, finalize_plan, approve_universe, submit_universe, approve_universe, start_universe_update, approve_charter, submit_charter, revise_charter, start_mandate_development, approve_draft_report, submit_draft_report, start_draft_report, review_draft_report, revise_draft_report, start_final_report, submit_final_report, finalize_audit_report, approve_work_program, design_work_program, approve_procedure, start_procedure, start_verification, complete_verification, start_monitoring, initiate_follow_up, track_action_plan_status

#### Gap Details: Missing Artifact Types

The fallback `dispatch_generic_workpaper` (line 448) creates only **Workpaper** records. It does NOT generate:

**Missing from ArtifactBag** (9 artifact types needed for IA):
1. **ProfessionalJudgment** — needed for:
   - apply_due_care, exercise_skepticism, conduct_ethics_training, evaluate_staff_performance
   - Currently generated only in opinion/findings context

2. **AuditSample** — needed for:
   - perform_ongoing_monitoring, re_verify_action, close_finding, close_monitoring
   - Currently generated only by sampling_gen in perform_tests_of_details path

3. **AnalyticalProcedureResult** — needed for:
   - review_engagement_quality, review_test_results, track_performance, verify_standards_conformance
   - Currently generated only by analytical_gen for perform_analytical_procedures

4. **AuditFinding** — needed for:
   - conclude_on_remediation, evaluate_management_responses, approve_draft_report
   - Currently generated only by findings gen (C2CE path)

5. **AuditEvidence** — needed for:
   - document_engagement_work, document_risk_acceptance, review_test_results
   - Currently generated only alongside workpapers in dispatch_workpaper

6. **InternalAuditFunction** (from AuditSnapshot, NOT in ArtifactBag)
   - Needed for: define_ia_mandate, draft_ia_charter, establish_board_interaction, obtain_board_support
   - NOT generated by StepDispatcher at all

7. **InternalAuditReport** (from AuditSnapshot, NOT in ArtifactBag)
   - Needed for: distribute_final_report, discuss_findings, issue_report, finalize_audit_report
   - NOT generated by StepDispatcher at all

8. **RelatedParty** / **RelatedPartyTransaction** (from AuditSnapshot, NOT in ArtifactBag)
   - Needed for: identify_stakeholders, identify_auditable_entities, establish_board_interaction
   - NOT generated by StepDispatcher at all

9. **AuditScope** (from AuditSnapshot, NOT in ArtifactBag)
   - Needed for: define_engagement_scope, define_ia_mandate, finalize_mandate
   - NOT generated by StepDispatcher at all

**Additional missing types (not in ArtifactBag OR dispatch output)**:
- sox_302_certifications, sox_404_assessments (already in AuditSnapshot, separate path)
- accounting_estimates (already in AuditSnapshot, separate path)
- unusual_items, analytical_relationships (already in AuditSnapshot, separate path)
- significant_transaction_classes (already in AuditSnapshot, separate path)
- component_auditors, group_audit_plan, component_instructions, component_reports (ISA 600, separate path)
- service_organizations, soc_reports, user_entity_controls (ISA 402, separate path)
- isa_mappings, isa_pcaob_mappings (reference data, separate path)

---

### 4. Hypergraph Integration Gaps

**File**: `/crates/datasynth-graph/src/builders/hypergraph.rs` (lines 2318–2552)

#### Current Implementation
`add_audit_documents()` (line 2318) accepts 6 artifact types:
- `engagements: &[AuditEngagement]`
- `workpapers: &[Workpaper]`
- `findings: &[AuditFinding]`
- `evidence: &[AuditEvidence]`
- `risks: &[RiskAssessment]`
- `judgments: &[ProfessionalJudgment]`

Creates nodes for: AUDIT_ENGAGEMENT (360), WORKPAPER (361), AUDIT_FINDING (362), AUDIT_EVIDENCE (363), RISK_ASSESSMENT (364), PROFESSIONAL_JUDGMENT (365)

#### Gaps

1. **Missing artifact types not passed**:
   - ExternalConfirmation, ConfirmationResponse (type codes 505-equiv, NO type codes defined)
   - AuditSample (NO type code; would need 366)
   - AnalyticalProcedureResult (NO type code; would need 367)
   - AuditProcedureStep (NO type code; would need 368)
   - SamplingPlan, SampledItem (NO type codes)
   - EngagementLetter (NO type code)
   - MaterialityCalculation, CombinedRiskAssessment, GoingConcernAssessment, SubsequentEvent, AuditOpinion, KeyAuditMatter (NO type codes)

2. **No cross-layer edges defined**:
   - No "engagement_scope" edges linking Engagement→Workpaper→Risk→Finding
   - No "evidence_chain" edges linking Finding→Evidence→Sample→Confirmation
   - No "temporal_ordering" edges for procedure sequencing
   - No "audit_assertion" edges linking Risk→Account→JournalEntry

3. **No audit-specific temporal graph**:
   - No "fieldwork_timeline" nodes/edges (engagement_start → procedures → conclusions → opinion)
   - No "review_hierarchy" edges (manager→senior→director→CAE)

4. **Method `add_audit_procedure_entities()` exists** (line 2552) but:
   - Accepts only: confirmations, responses, steps, samples, analytical_results
   - Does NOT accept judgments, findings, evidence
   - Creates nodes but NO cross-layer relationships

#### Type Codes Missing (Needed for Wave 1)
```
CONFIRMATION: u32 = 368;          // ExternalConfirmation
CONFIRMATION_RESPONSE: u32 = 369; // ConfirmationResponse
AUDIT_SAMPLE: u32 = 370;          // AuditSample
ANALYTICAL_RESULT: u32 = 371;     // AnalyticalProcedureResult
AUDIT_PROCEDURE_STEP: u32 = 372;  // AuditProcedureStep
SAMPLING_PLAN: u32 = 373;         // SamplingPlan
ENGAGEMENT_LETTER: u32 = 374;     // EngagementLetter
MATERIALITY_CALC: u32 = 375;      // MaterialityCalculation
COMBINED_RISK_ASSESSMENT: u32 = 376; // CombinedRiskAssessment
GOING_CONCERN: u32 = 377;         // GoingConcernAssessment
SUBSEQUENT_EVENT: u32 = 378;      // SubsequentEvent
AUDIT_OPINION: u32 = 379;         // AuditOpinion
KEY_AUDIT_MATTER: u32 = 380;      // KeyAuditMatter
```

#### Edge Type Suggestions (For Temporal Graph)
```rust
// Audit procedure lifecycle
"fsm_state_transition"   // FSM phase change (Governance→Planning→Performing→Reporting→Monitoring)
"engagement_scope"       // Engagement→Account/Assertion
"procedure_sequence"     // Step N→Step N+1 (based on FSM ordering)
"fieldwork_assignment"   // Engagement→TeamMember
"evidence_chain"         // Finding→Evidence→Sample→Confirmation→Response
"review_relationship"    // Workpaper→Reviewer→ReviewDate
"materiality_linkage"    // Engagement→MaterialityCalculation→Risk→Sample size
"opinion_basis"          // Opinion→KeyAuditMatter→Finding/Evidence
"temporal_phase"         // JournalEntry.posting_date vs Engagement.fieldwork_date (pre/during/post)
```

---

### 5. Demo Preset Gap

**File**: `/crates/datasynth-cli/src/main.rs` (line 2441)

#### Current State
```rust
audit: AuditGenerationConfig::default()  // FSM DISABLED
```

#### Impact
- `datasynth-data generate --demo` produces standard audit artifacts
- Does NOT test FSM engine even in demo mode
- No easy way to validate FSM with `--demo --audit` unless user manually enables `audit.fsm.enabled: true`

#### Gap
Demo should enable FSM when `--audit` is passed to ensure:
1. FSM blueprint loading is tested
2. FSM event trail is written
3. Full ArtifactBag→AuditSnapshot→Output pipeline is exercised

---

## QUANTIFIED GAPS

| Category | Metric | Count | Notes |
|----------|--------|-------|-------|
| **Step Dispatch** | Total IA commands | 140 | From 199 steps in generic_ia.yaml |
| | Mapped (explicit dispatch) | 43 | 30.7% coverage |
| | Fallback-only (no generator) | 8 | governance/workflow commands |
| | Unmapped (generic workpaper) | 89 | 63.6% fallback rate |
| **Artifacts** | ArtifactBag fields | 20 | FSM generates only 20 of 41 snapshot types |
| **Output Writer** | FSM fields written | 20/20 | ✓ 100% coverage |
| | fsm_event_trail written | 1/1 | ✓ Included |
| **Hypergraph** | Audit types supported | 6 | engagements, workpapers, findings, evidence, risks, judgments |
| | Missing type codes | 13 | confirmations, samples, analytical results, etc. |
| | Cross-layer edges defined | 0 | No temporal or evidence-chain relationships |
| **CLI** | Audit/FSM subcommands | 0 | No validate/info/coverage for blueprints |
| | Blueprint validation functions | 3 | Public but inaccessible via CLI |

---

## ROOT CAUSES

1. **StepDispatcher design assumes all commands are either mapped or generate workpapers**
   - ✓ By design (Phase 4 committed to 43 mapped generators + fallback)
   - Gap emerges only when IA blueprint expands beyond governance/risk/findings cycle

2. **ArtifactBag was optimized for "core audit path"** (engagement→materiality→risk→test→findings→opinion)
   - Administrative/workflow artifact types (IA functions, scope records) omitted
   - Would require new generators or model extensions

3. **Hypergraph builder didn't anticipate FSM-generated artifacts**
   - Designed for orchestrator-wide snapshot, not per-artifact-type generation
   - Type codes 360–365 reserved for Phase 4 MVP; remaining IDs not allocated

4. **CLI never extended for FSM introspection**
   - Loader and validation functions exist but are internal to FSM crate
   - Demo preset hardcoded to disable FSM

---

## RECOMMENDATIONS FOR WAVE 1 CLOSURE

**Priority 1 (Blocking)**:
1. Create `audit validate` / `audit info` CLI subcommands exposing `loader.rs` public functions
2. Enable FSM in demo preset when `--audit` is passed (or add `--fsm` flag)

**Priority 2 (Enhancement)**:
1. Extend StepDispatcher to handle 3–5 high-frequency IA commands (assign_team_members, draft_annual_plan, document_engagement_work, etc.) with targeted generators
2. Extend ArtifactBag with IA-specific types (InternalAuditFunction, AuditScope, RelatedParty)

**Priority 3 (Completeness)**:
1. Allocate type codes 366–380 for audit procedure artifacts in hypergraph
2. Define cross-layer edge types for temporal audit workflows
3. Update `add_audit_documents()` to accept all ArtifactBag + IA-extended types

---

## REFERENCE DATA

**Generic IA Blueprint** (`crates/datasynth-audit-fsm/blueprints/generic_ia.yaml`):
- Schema version: 1.0
- Total steps: 199
- Total unique commands: 140
- Framework: IIA-GIAS 2024
- Cycles: governance, planning, resource allocation, testing, control evaluation, reporting, monitoring, follow-up

**Type Codes Already Allocated** (from hypergraph.rs):
- 360 = AUDIT_ENGAGEMENT
- 361 = WORKPAPER
- 362 = AUDIT_FINDING
- 363 = AUDIT_EVIDENCE
- 364 = RISK_ASSESSMENT
- 365 = PROFESSIONAL_JUDGMENT
- Next available: 366+

**Output Writer Coverage** (lines 626–633 in output_writer.rs):
- ✓ All 20 ArtifactBag types written to `audit/*.json`
- ✓ fsm_event_trail written to `audit/fsm_event_trail.json`
- ✓ Sinks exist for all mapped types
