# Wave 1: Consolidation (v1.6.0) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the audit FSM engine production-ready with CLI tooling, comprehensive IA command dispatch, graph integration, and demo mode support.

**Architecture:** Four independent work areas modifying existing crates: CLI subcommand in `datasynth-cli`, dispatch enrichment in `datasynth-audit-fsm`, graph edges in `datasynth-graph`, and demo config update. No new crates created.

**Tech Stack:** Existing Rust workspace, clap for CLI, petgraph for graph, serde for serialization.

**Spec:** `docs/superpowers/specs/2026-03-24-wave1-consolidation-design.md`

**CRITICAL:** Use `--test-threads=1` for ALL test runs. NEVER run concurrent builds or tests.

---

### Task 1: Add `datasynth-audit-fsm` Dependency to CLI Crate

**Files:**
- Modify: `crates/datasynth-cli/Cargo.toml`

- [ ] **Step 1: Add dependency**

In `crates/datasynth-cli/Cargo.toml`, add to `[dependencies]`:
```toml
datasynth-audit-fsm = { workspace = true }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p datasynth-cli`
Expected: compiles (the dependency resolves via workspace)

- [ ] **Step 3: Commit**

```bash
git add crates/datasynth-cli/Cargo.toml Cargo.lock
git commit -m "build(cli): add datasynth-audit-fsm dependency"
```

---

### Task 2: CLI `audit validate` and `audit info` Subcommands

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

This adds the `audit` top-level command with `validate` and `info` actions.

- [ ] **Step 1: Read the current Commands enum and Fingerprint pattern**

Read `crates/datasynth-cli/src/main.rs` to understand:
- The `Commands` enum (around line 48)
- How `Fingerprint { command: FingerprintCommands }` is structured
- How the main dispatch works in the match statement

- [ ] **Step 2: Add the AuditCommands enum and Commands::Audit variant**

Add after the existing `ScenarioCommands` enum:

```rust
#[derive(Subcommand)]
enum AuditCommands {
    /// Validate a blueprint YAML file
    Validate {
        /// Blueprint source: "builtin:fsa", "builtin:ia", or path to YAML
        #[arg(long, default_value = "builtin:fsa")]
        blueprint: String,
    },
    /// Display blueprint information
    Info {
        /// Blueprint source
        #[arg(long, default_value = "builtin:fsa")]
        blueprint: String,
    },
    /// Run a standalone FSM engagement
    Run {
        /// Blueprint source
        #[arg(long, default_value = "builtin:fsa")]
        blueprint: String,
        /// Overlay source: "builtin:default", "builtin:thorough", "builtin:rushed", or path
        #[arg(long, default_value = "builtin:default")]
        overlay: String,
        /// Output directory
        #[arg(short, long, default_value = "./audit_output")]
        output: PathBuf,
        /// RNG seed
        #[arg(long, default_value = "42")]
        seed: u64,
    },
}
```

Add to `Commands` enum:
```rust
/// Audit FSM blueprint commands
Audit {
    #[command(subcommand)]
    command: AuditCommands,
},
```

- [ ] **Step 3: Implement blueprint resolution helper**

Add a helper function that parses the `--blueprint` string into a `BlueprintWithPreconditions`:

```rust
fn resolve_blueprint(blueprint: &str) -> Result<datasynth_audit_fsm::loader::BlueprintWithPreconditions> {
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
    match blueprint {
        "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa()
            .map_err(|e| anyhow::anyhow!("Failed to load FSA blueprint: {e}")),
        "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia()
            .map_err(|e| anyhow::anyhow!("Failed to load IA blueprint: {e}")),
        path => {
            // Custom path — load via loader
            use datasynth_audit_fsm::loader::{load_blueprint, BlueprintSource};
            let bp = load_blueprint(&BlueprintSource::Custom(std::path::PathBuf::from(path)))
                .map_err(|e| anyhow::anyhow!("Failed to load blueprint from {path}: {e}"))?;
            // For custom blueprints, preconditions come from the raw YAML
            // Use empty preconditions as fallback
            Ok(BlueprintWithPreconditions {
                blueprint: bp,
                preconditions: std::collections::HashMap::new(),
            })
        }
    }
}
```

Also add an overlay resolution helper:
```rust
fn resolve_overlay(overlay: &str) -> Result<datasynth_audit_fsm::schema::GenerationOverlay> {
    use datasynth_audit_fsm::loader::*;
    let source = match overlay {
        "builtin:default" => OverlaySource::Builtin(BuiltinOverlay::Default),
        "builtin:thorough" => OverlaySource::Builtin(BuiltinOverlay::Thorough),
        "builtin:rushed" => OverlaySource::Builtin(BuiltinOverlay::Rushed),
        path => OverlaySource::Custom(std::path::PathBuf::from(path)),
    };
    load_overlay(&source).map_err(|e| anyhow::anyhow!("Failed to load overlay: {e}"))
}
```

- [ ] **Step 4: Implement `audit validate`**

```rust
fn handle_audit_validate(blueprint: &str) -> Result<()> {
    let bwp = resolve_blueprint(blueprint)?;
    match bwp.validate() {
        Ok(()) => {
            let total_procs: usize = bwp.blueprint.phases.iter()
                .map(|p| p.procedures.len()).sum();
            let total_steps: usize = bwp.blueprint.phases.iter()
                .flat_map(|p| p.procedures.iter())
                .map(|proc| proc.steps.len()).sum();
            println!("✓ Blueprint valid");
            println!("  Framework:   {}", bwp.blueprint.methodology.framework);
            println!("  Phases:      {}", bwp.blueprint.phases.len());
            println!("  Procedures:  {}", total_procs);
            println!("  Steps:       {}", total_steps);
            println!("  Standards:   {}", bwp.blueprint.standards.len());
            println!("  Actors:      {}", bwp.blueprint.actors.len());
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Blueprint validation failed:");
            eprintln!("  {e}");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 5: Implement `audit info`**

```rust
fn handle_audit_info(blueprint: &str) -> Result<()> {
    let bwp = resolve_blueprint(blueprint)?;
    let bp = &bwp.blueprint;

    let total_procs: usize = bp.phases.iter().map(|p| p.procedures.len()).sum();
    let total_steps: usize = bp.phases.iter()
        .flat_map(|p| p.procedures.iter())
        .map(|proc| proc.steps.len()).sum();
    let continuous = bp.phases.iter()
        .filter(|p| p.order.map(|o| o < 0).unwrap_or(false)).count();

    println!("Blueprint: {} ({})", bp.methodology.framework,
        bp.methodology.description.as_deref().unwrap_or(""));
    println!("Phases:      {} ({} continuous, {} sequential)",
        bp.phases.len(), continuous, bp.phases.len() - continuous);
    println!("Procedures:  {}", total_procs);
    println!("Steps:       {}", total_steps);
    println!("Standards:   {}", bp.standards.len());
    println!("Actors:      {}", bp.actors.len());
    println!("Evidence:    {}", bp.evidence_templates.len());
    println!();

    // Command coverage
    let all_commands: Vec<&str> = bp.phases.iter()
        .flat_map(|p| p.procedures.iter())
        .flat_map(|proc| proc.steps.iter())
        .filter_map(|s| s.command.as_deref())
        .collect();
    let unique_commands: std::collections::HashSet<&str> = all_commands.iter().copied().collect();
    println!("Unique commands: {}", unique_commands.len());
    println!();

    // Phase breakdown
    println!("Phase breakdown:");
    for phase in &bp.phases {
        let ptype = if phase.order.map(|o| o < 0).unwrap_or(false) {
            "continuous"
        } else {
            "sequential"
        };
        let steps: usize = phase.procedures.iter().map(|p| p.steps.len()).sum();
        println!("  {:40} ({})  {} procedures, {} steps",
            phase.name, ptype, phase.procedures.len(), steps);
    }

    Ok(())
}
```

- [ ] **Step 6: Implement `audit run`**

```rust
fn handle_audit_run(blueprint: &str, overlay: &str, output: &Path, seed: u64) -> Result<()> {
    use datasynth_audit_fsm::engine::AuditFsmEngine;
    use datasynth_audit_fsm::context::EngagementContext;
    use datasynth_audit_fsm::export::flat_log::export_events_to_file;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let bwp = resolve_blueprint(blueprint)?;
    bwp.validate().map_err(|e| anyhow::anyhow!("Validation failed: {e}"))?;
    let ov = resolve_overlay(overlay)?;

    let rng = ChaCha8Rng::seed_from_u64(seed);
    let mut engine = AuditFsmEngine::new(bwp, ov, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx)
        .map_err(|e| anyhow::anyhow!("Engagement failed: {e}"))?;

    std::fs::create_dir_all(output)?;
    let trail_path = output.join("audit_event_trail.json");
    export_events_to_file(&result.event_log, &trail_path)?;

    println!("Engagement complete:");
    println!("  Events:     {}", result.event_log.len());
    println!("  Artifacts:  {}", result.artifacts.total_artifacts());
    println!("  Phases:     {}", result.phases_completed.len());
    println!("  Anomalies:  {}", result.anomalies.len());
    println!("  Duration:   {:.1}h", result.total_duration_hours);
    println!("  Output:     {}", trail_path.display());

    Ok(())
}
```

- [ ] **Step 7: Wire into main dispatch**

In the main `match cli.command { ... }` block, add:
```rust
Commands::Audit { command } => match command {
    AuditCommands::Validate { blueprint } => handle_audit_validate(&blueprint)?,
    AuditCommands::Info { blueprint } => handle_audit_info(&blueprint)?,
    AuditCommands::Run { blueprint, overlay, output, seed } => {
        handle_audit_run(&blueprint, &overlay, &output, seed)?
    }
},
```

- [ ] **Step 8: Verify compilation and test**

Run: `cargo check -p datasynth-cli`
Run: `cargo run -p datasynth-cli -- audit info --blueprint builtin:fsa`
Run: `cargo run -p datasynth-cli -- audit validate --blueprint builtin:ia`
Expected: both produce output without errors

- [ ] **Step 9: Commit**

```bash
git add crates/datasynth-cli/src/main.rs
git commit -m "feat(cli): add audit subcommand with validate, info, and run actions"
```

---

### Task 3: Enable FSM in Demo Mode

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

- [ ] **Step 1: Find `create_safe_demo_preset` and update audit config**

Find the function (around line 2391) and change the audit section from `AuditGenerationConfig::default()` to enable FSM:

```rust
audit: {
    let mut a = AuditGenerationConfig::default();
    a.enabled = true;
    a.fsm = Some(AuditFsmConfig {
        enabled: true,
        blueprint: "builtin:fsa".into(),
        overlay: "builtin:default".into(),
        ..Default::default()
    });
    a
},
```

You'll need to import `AuditFsmConfig` from `datasynth_config::schema`.

- [ ] **Step 2: Test demo mode**

Run: `cargo run -p datasynth-cli -- generate --demo --output /tmp/demo_test`
Expected: produces `audit/fsm_event_trail.json` in output directory

- [ ] **Step 3: Commit**

```bash
git add crates/datasynth-cli/src/main.rs
git commit -m "feat(cli): enable FSM audit generation in demo mode"
```

---

### Task 4: IA Dispatch Enrichment

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/dispatch.rs`

- [ ] **Step 1: Read current dispatch.rs fully**

Understand the match statement (lines ~96-212) and `section_for_command` (lines ~404-430).

- [ ] **Step 2: Add judgment/quality dispatch**

Add a new dispatch method and match arms for judgment-producing commands:

```rust
// In the match statement, add before the fallback:
"review_engagement_quality" | "exercise_skepticism" | "apply_due_care"
| "supervise_engagement_quality" | "conduct_periodic_assessment"
| "oversee_qaip" | "evaluate_staff_performance" => {
    self.dispatch_judgment(context, bag);
}
```

Add the dispatch method (reuses JudgmentGenerator which is NOT currently in StepDispatcher):

```rust
fn dispatch_judgment(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
    let engagement = match bag.engagements.last() {
        Some(e) => e,
        None => return,
    };
    let judgment = self.judgment_gen.generate_judgment(engagement);
    bag.judgments.push(judgment);
}
```

You'll need to add `JudgmentGenerator` to the StepDispatcher struct and its `new()` constructor:
```rust
judgment_gen: JudgmentGenerator,
// In new():
judgment_gen: JudgmentGenerator::new(base_seed + 8400),
```

Import: `use datasynth_generators::audit::JudgmentGenerator;`

- [ ] **Step 3: Add evidence/documentation dispatch**

```rust
"document_engagement_work" | "archive_engagement_documentation"
| "protect_information" | "establish_confidentiality_policies" => {
    self.dispatch_evidence(context, bag);
}
```

Add method:
```rust
fn dispatch_evidence(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
    let engagement = match bag.engagements.last() {
        Some(e) => e,
        None => return,
    };
    let workpapers = &bag.workpapers;
    if let Some(wp) = workpapers.last() {
        let evidence = self.evidence_gen.generate_evidence_for_workpaper(
            wp, &ctx.team_member_ids, ctx.engagement_start,
        );
        bag.evidence.extend(evidence);
    }
}
```

- [ ] **Step 4: Add planning/scoping dispatch**

These produce workpapers with Planning section:

```rust
"define_engagement_scope" | "determine_engagement_timeline"
| "draft_annual_plan" | "develop_work_program" | "scope_engagement"
| "develop_staffing_plan" | "develop_ia_budget" | "assign_team_members"
| "determine_sme_needs" | "confirm_resource_competencies"
| "identify_auditable_entities" | "prioritize_audit_entities" => {
    self.dispatch_workpaper_section(step, procedure_id, context, bag, WorkpaperSection::Planning);
}
```

Add helper:
```rust
fn dispatch_workpaper_section(
    &mut self,
    step: &BlueprintStep,
    _procedure_id: &str,
    ctx: &EngagementContext,
    bag: &mut ArtifactBag,
    section: WorkpaperSection,
) {
    let engagement = match bag.engagements.last() {
        Some(e) => e,
        None => return,
    };
    let wp = self.workpaper_gen.generate_workpaper(
        engagement, section, ctx.engagement_start, &ctx.team_member_ids,
    );
    bag.workpapers.push(wp);
}
```

- [ ] **Step 5: Add reporting dispatch**

```rust
"draft_audit_report" | "review_draft_report" | "prepare_draft_report"
| "evaluate_management_responses" | "send_report_for_response"
| "receive_response" | "distribute_final_report" | "communicate_approved_plan"
| "present_plan_to_board" | "communicate_plan_and_results" => {
    self.dispatch_workpaper_section(step, procedure_id, context, bag, WorkpaperSection::Reporting);
}
```

- [ ] **Step 6: Add follow-up/monitoring dispatch (produces findings)**

```rust
"track_action_plan_status" | "escalate_overdue_actions"
| "report_follow_up_status" | "verify_remediation_implementation"
| "conclude_on_remediation" => {
    self.dispatch_findings(context, bag);
}
```

- [ ] **Step 7: Add ethics/governance dispatch**

```rust
"establish_ethics_code" | "conduct_ethics_training" | "monitor_ethics_compliance"
| "assess_objectivity_threats" | "implement_objectivity_safeguards"
| "disclose_impairments" | "assess_competencies" | "develop_cpd_plans"
| "verify_standards_conformance" | "maintain_ethical_standards"
| "safeguard_objectivity" | "establish_independence" | "define_ia_mandate"
| "draft_ia_charter" | "establish_board_interaction" | "obtain_board_support"
| "assess_technology_capabilities" | "implement_technology"
| "manage_technological_resources" => {
    self.dispatch_workpaper_section(step, procedure_id, context, bag, WorkpaperSection::Planning);
}
```

- [ ] **Step 8: Add performance/monitoring dispatch**

```rust
"define_performance_metrics" | "track_performance"
| "report_performance_to_board" | "measure_ia_performance"
| "perform_ongoing_monitoring" | "monitor_budget_utilization" => {
    self.dispatch_workpaper_section(step, procedure_id, context, bag, WorkpaperSection::Completion);
}
```

- [ ] **Step 9: Update section_for_command for IA keywords**

Expand the keyword matching to cover IA-specific terms. Read the current function and extend it with additional keywords for each section.

- [ ] **Step 10: Add tests**

```rust
#[test]
fn test_ia_judgment_dispatch() {
    let mut dispatcher = StepDispatcher::new(42);
    let ctx = EngagementContext::test_default();
    let mut bag = ArtifactBag::default();
    // Bootstrap engagement
    dispatcher.dispatch(&step_with_command("e1", "evaluate_client_acceptance"), "proc", &ctx, &mut bag);
    // Judgment command
    dispatcher.dispatch(&step_with_command("j1", "review_engagement_quality"), "proc", &ctx, &mut bag);
    assert!(!bag.judgments.is_empty(), "Should produce judgments");
}

#[test]
fn test_ia_evidence_dispatch() {
    let mut dispatcher = StepDispatcher::new(42);
    let ctx = EngagementContext::test_default();
    let mut bag = ArtifactBag::default();
    dispatcher.dispatch(&step_with_command("e1", "evaluate_client_acceptance"), "proc", &ctx, &mut bag);
    // Create a workpaper first
    dispatcher.dispatch(&step_with_command("w1", "design_work_program"), "proc", &ctx, &mut bag);
    // Evidence command
    dispatcher.dispatch(&step_with_command("d1", "document_engagement_work"), "proc", &ctx, &mut bag);
    assert!(!bag.evidence.is_empty(), "Should produce evidence");
}
```

- [ ] **Step 11: Verify all tests pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=1`
Expected: all tests pass

- [ ] **Step 12: Run evaluation to verify IA artifact improvement**

Run: `cargo test -p datasynth-audit-fsm --test evaluate_output -- --nocapture --test-threads=1`
Expected: IA artifacts > 1,891 (previous baseline)

- [ ] **Step 13: Commit**

```bash
git add crates/datasynth-audit-fsm/src/dispatch.rs
git commit -m "feat(audit-fsm): enrich IA dispatch with judgment, evidence, planning, reporting, and governance commands"
```

---

### Task 5: Graph Integration — Audit Evidence Chain Edges

**Files:**
- Modify: `crates/datasynth-graph/src/builders/hypergraph.rs`

- [ ] **Step 1: Read the current `add_audit_documents` method**

Read `crates/datasynth-graph/src/builders/hypergraph.rs` around line 2318. Understand how nodes and edges are created, what type codes are used.

- [ ] **Step 2: Extend `add_audit_documents` to accept additional artifact types**

Update the method signature to accept materiality, opinions, and going concern:

```rust
pub fn add_audit_documents(
    &mut self,
    engagements: &[AuditEngagement],
    workpapers: &[Workpaper],
    findings: &[AuditFinding],
    evidence: &[AuditEvidence],
    risks: &[RiskAssessment],
    judgments: &[ProfessionalJudgment],
    // NEW parameters:
    materiality: &[MaterialityCalculation],
    opinions: &[AuditOpinion],
    going_concern: &[GoingConcernAssessment],
)
```

- [ ] **Step 3: Add node creation for new types**

Add node creation for MaterialityCalculation, AuditOpinion, GoingConcernAssessment using existing allocated type codes (check the type_codes module — they may already be allocated or you may need to add them).

- [ ] **Step 4: Add evidence-chain edges**

Create edges between audit nodes:

```rust
// Engagement → Workpaper (DOCUMENTED_BY)
// For each workpaper that references the engagement
for wp in workpapers {
    if let Some(eng) = engagements.first() {
        self.edges.push(CrossLayerEdge {
            source_id: format!("audit_eng_{}", eng.engagement_id),
            target_id: format!("audit_wp_{}", wp.workpaper_id),
            edge_type: "documented_by".into(),
            // ...
        });
    }
}

// Finding → RiskAssessment (IDENTIFIED_FROM)
// Finding → Evidence (EVIDENCED_BY)
// AuditOpinion → Finding (BASED_ON)
```

- [ ] **Step 5: Update the caller in enhanced_orchestrator.rs**

Find where `add_audit_documents` is called and pass the new parameters from AuditSnapshot.

- [ ] **Step 6: Verify compilation**

Run: `cargo check -p datasynth-graph`
Run: `cargo check -p datasynth-runtime`

- [ ] **Step 7: Commit**

```bash
git add crates/datasynth-graph/src/builders/hypergraph.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(graph): add audit evidence-chain edges and extended artifact nodes"
```

---

### Task 6: Final Validation and Cleanup

**Files:**
- All modified files

- [ ] **Step 1: Run fmt**

```bash
cargo fmt --all
```

- [ ] **Step 2: Run clippy on modified crates**

```bash
cargo clippy -p datasynth-cli -p datasynth-audit-fsm -p datasynth-graph -p datasynth-runtime
```

- [ ] **Step 3: Run all FSM tests**

```bash
cargo test -p datasynth-audit-fsm -- --test-threads=1
cargo test -p datasynth-audit-optimizer -- --test-threads=1
```

- [ ] **Step 4: Run full evaluation with output**

```bash
cargo test -p datasynth-audit-fsm --test evaluate_output -- --nocapture --test-threads=1
```

- [ ] **Step 5: Test CLI commands**

```bash
cargo run -p datasynth-cli -- audit validate --blueprint builtin:fsa
cargo run -p datasynth-cli -- audit validate --blueprint builtin:ia
cargo run -p datasynth-cli -- audit info --blueprint builtin:ia
cargo run -p datasynth-cli -- audit run --blueprint builtin:fsa --output /tmp/audit_test
```

- [ ] **Step 6: Commit if needed**

```bash
git commit -m "feat(wave1): finalize Wave 1 consolidation — CLI tooling, IA dispatch, graph edges"
```

---

## Summary

| Task | What it delivers | Key files |
|------|-----------------|-----------|
| 1 | CLI dependency setup | `datasynth-cli/Cargo.toml` |
| 2 | `audit validate/info/run` commands | `datasynth-cli/src/main.rs` |
| 3 | FSM-enabled demo mode | `datasynth-cli/src/main.rs` |
| 4 | IA dispatch enrichment (43→80+ commands) | `datasynth-audit-fsm/src/dispatch.rs` |
| 5 | Graph evidence-chain edges | `datasynth-graph/src/builders/hypergraph.rs` |
| 6 | Final validation | All files |
