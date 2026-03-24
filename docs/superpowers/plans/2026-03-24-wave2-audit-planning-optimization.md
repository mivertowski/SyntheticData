# Wave 2: Audit Planning Optimization (v1.7.0) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add per-procedure iteration limits, cost model, resource-constrained optimization, risk-based scoping, and multi-engagement portfolio simulation to the audit FSM engine and optimizer.

**Architecture:** Schema additions in `datasynth-audit-fsm` (iteration limits, cost fields), three new modules in `datasynth-audit-optimizer` (resource_optimizer, risk_scoping, portfolio). Blueprint YAMLs get `base_hours` annotations. Overlay YAMLs get `iteration_limits` and `resource_costs` sections.

**Tech Stack:** Existing Rust workspace, `datasynth-audit-fsm` engine, `datasynth-audit-optimizer` with petgraph.

**Spec:** `docs/superpowers/specs/2026-03-24-wave2-audit-planning-optimization-design.md`

**CRITICAL:** Use `--test-threads=1` for ALL test runs. NEVER run concurrent builds or tests. Sequential per-crate testing only.

---

### Task 1: Per-Procedure Iteration Limits

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/schema.rs`
- Modify: `crates/datasynth-audit-fsm/src/engine.rs`
- Modify: `crates/datasynth-audit-fsm/overlays/default.yaml`
- Modify: `crates/datasynth-audit-fsm/overlays/thorough.yaml`
- Modify: `crates/datasynth-audit-fsm/overlays/rushed.yaml`

- [ ] **Step 1: Add IterationLimits to schema.rs**

Read `crates/datasynth-audit-fsm/src/schema.rs`. Find `GenerationOverlay` struct (around line 354). Add `IterationLimits` struct and field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationLimits {
    #[serde(default = "default_iteration_limit")]
    pub default: usize,
    #[serde(default)]
    pub per_procedure: HashMap<String, usize>,
}

fn default_iteration_limit() -> usize {
    30
}

impl Default for IterationLimits {
    fn default() -> Self {
        Self {
            default: 30,
            per_procedure: HashMap::new(),
        }
    }
}
```

Add to `GenerationOverlay`:
```rust
#[serde(default)]
pub iteration_limits: IterationLimits,
```

- [ ] **Step 2: Replace MAX_ITERATIONS in engine.rs**

Read `crates/datasynth-audit-fsm/src/engine.rs`. Find `const MAX_ITERATIONS: usize = 20;` (line 67) and the usage in the FSM walk loop (around line 170).

Remove the constant. Replace the check with:
```rust
let max_iter = self.overlay.iteration_limits
    .per_procedure.get(proc_id)
    .copied()
    .unwrap_or(self.overlay.iteration_limits.default);
if iterations >= max_iter {
    break;
}
```

- [ ] **Step 3: Update overlay YAMLs**

Add to each overlay file:

`default.yaml`:
```yaml
iteration_limits:
  default: 30
```

`thorough.yaml`:
```yaml
iteration_limits:
  default: 40
```

`rushed.yaml`:
```yaml
iteration_limits:
  default: 20
```

- [ ] **Step 4: Test IA completion improvement**

```rust
#[test]
fn test_ia_completion_rate_with_higher_limit() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay(); // now has iteration_limits.default = 30
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx).unwrap();

    let completed = result.procedure_states.values()
        .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
        .count();
    assert!(completed >= 28, "Expected >= 28/34 completed with limit=30, got {}", completed);
}
```

- [ ] **Step 5: Verify all existing tests pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=1`

Note: Existing tests may need threshold adjustments since more procedures now complete. Check `wave1_e2e.rs` and `evaluate_output.rs` assertions.

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-audit-fsm/
git commit -m "feat(audit-fsm): replace global MAX_ITERATIONS with per-procedure configurable limits"
```

---

### Task 2: Cost Model Schema

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/schema.rs`
- Modify: `crates/datasynth-audit-fsm/blueprints/generic_fsa.yaml`
- Modify: `crates/datasynth-audit-fsm/blueprints/generic_ia.yaml`
- Modify: `crates/datasynth-audit-fsm/overlays/default.yaml`

- [ ] **Step 1: Add cost fields to schema**

In `schema.rs`, add to `BlueprintProcedure`:
```rust
#[serde(default)]
pub base_hours: Option<f64>,
#[serde(default)]
pub required_roles: Vec<String>,
```

Add `ResourceCosts` struct:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCosts {
    #[serde(default = "default_cost_multiplier")]
    pub cost_multiplier: f64,
    #[serde(default)]
    pub role_hourly_rates: HashMap<String, f64>,
    #[serde(default)]
    pub per_procedure_multipliers: HashMap<String, f64>,
}

fn default_cost_multiplier() -> f64 { 1.0 }

impl Default for ResourceCosts {
    fn default() -> Self {
        Self {
            cost_multiplier: 1.0,
            role_hourly_rates: HashMap::new(),
            per_procedure_multipliers: HashMap::new(),
        }
    }
}
```

Add to `GenerationOverlay`:
```rust
#[serde(default)]
pub resource_costs: ResourceCosts,
```

- [ ] **Step 2: Add base_hours to FSA blueprint**

In `generic_fsa.yaml`, add `base_hours` to each procedure. Example hours:

| Procedure | base_hours |
|-----------|-----------|
| accept_engagement | 4.0 |
| planning_materiality | 8.0 |
| risk_identification | 16.0 |
| test_of_controls | 24.0 |
| substantive_testing | 32.0 |
| analytical_procedures | 12.0 |
| going_concern | 8.0 |
| subsequent_events | 6.0 |
| form_opinion | 12.0 |

The field goes at the procedure level, after `title` or `discriminators`.

- [ ] **Step 3: Add base_hours to IA blueprint**

Add `base_hours` to each of the 34 IA procedures. Use reasonable estimates:
- Planning procedures: 4-8h
- Fieldwork procedures: 12-32h
- Reporting procedures: 8-16h
- Continuous/governance: 4-6h

- [ ] **Step 4: Add resource_costs to default overlay**

```yaml
resource_costs:
  cost_multiplier: 1.0
  role_hourly_rates:
    engagement_partner: 500
    audit_manager: 300
    audit_senior: 200
    audit_staff: 120
    cae: 400
    audit_director: 350
    senior_auditor: 200
    staff_auditor: 120
```

- [ ] **Step 5: Add cost computation helper**

Create a utility function (in `schema.rs` or a new `cost.rs` module in the optimizer):

```rust
pub fn effective_hours(
    proc: &BlueprintProcedure,
    overlay: &GenerationOverlay,
) -> f64 {
    let base = proc.base_hours.unwrap_or(8.0);
    let global_mult = overlay.resource_costs.cost_multiplier;
    let proc_mult = overlay.resource_costs
        .per_procedure_multipliers
        .get(&proc.id)
        .copied()
        .unwrap_or(1.0);
    base * global_mult * proc_mult
}

pub fn procedure_cost(
    proc: &BlueprintProcedure,
    overlay: &GenerationOverlay,
) -> f64 {
    let hours = effective_hours(proc, overlay);
    let role = proc.required_roles.first()
        .map(|r| r.as_str())
        .unwrap_or("audit_staff");
    let rate = overlay.resource_costs
        .role_hourly_rates
        .get(role)
        .copied()
        .unwrap_or(200.0);
    hours * rate
}
```

- [ ] **Step 6: Test cost computation**

```rust
#[test]
fn test_effective_hours_with_multipliers() {
    // base=8, global_mult=1.5, proc_mult=1.3 → 8 * 1.5 * 1.3 = 15.6
}

#[test]
fn test_procedure_cost_with_role_rate() {
    // hours=10, role=audit_manager, rate=300 → 3000
}

#[test]
fn test_fsa_blueprint_has_base_hours() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    for phase in &bwp.blueprint.phases {
        for proc in &phase.procedures {
            assert!(proc.base_hours.is_some(), "Procedure {} missing base_hours", proc.id);
        }
    }
}
```

- [ ] **Step 7: Verify and commit**

```bash
cargo test -p datasynth-audit-fsm -- --test-threads=1
cargo test -p datasynth-audit-optimizer -- --test-threads=1
git commit -m "feat(audit-fsm): add cost model with blueprint base hours and overlay multipliers"
```

---

### Task 3: Resource-Constrained Optimization

**Files:**
- Create: `crates/datasynth-audit-optimizer/src/resource_optimizer.rs`
- Modify: `crates/datasynth-audit-optimizer/src/lib.rs`

- [ ] **Step 1: Define types**

```rust
//! Resource-constrained audit plan optimization.

use datasynth_audit_fsm::schema::{AuditBlueprint, GenerationOverlay};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceConstraints {
    pub total_budget_hours: f64,
    pub role_availability: HashMap<String, f64>,
    pub must_include: Vec<String>,
    pub must_exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizedPlan {
    pub included_procedures: Vec<String>,
    pub excluded_procedures: Vec<String>,
    pub total_hours: f64,
    pub total_cost: f64,
    pub risk_coverage: f64,
    pub standards_coverage: f64,
    pub critical_path_hours: f64,
    pub role_hours: HashMap<String, f64>,
}
```

- [ ] **Step 2: Implement optimizer**

```rust
pub fn optimize_plan(
    blueprint: &AuditBlueprint,
    overlay: &GenerationOverlay,
    preconditions: &HashMap<String, Vec<String>>,
    constraints: &ResourceConstraints,
) -> OptimizedPlan
```

Algorithm:
1. Collect all procedures from blueprint phases
2. Start with must_include + transitive preconditions
3. Remove must_exclude
4. Compute hours for mandatory set
5. Score remaining by `risk_tags.len() / effective_hours`
6. Greedily add until budget exhausted
7. Compute coverage and critical path

- [ ] **Step 3: Tests**

```rust
#[test]
fn test_must_include_always_present()

#[test]
fn test_budget_constrains_selection()

#[test]
fn test_must_exclude_removed()

#[test]
fn test_critical_path_computed()

#[test]
fn test_serializes_to_json()
```

- [ ] **Step 4: Update lib.rs, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=1
git commit -m "feat(audit-optimizer): add resource-constrained plan optimization"
```

---

### Task 4: Risk-Based Audit Scoping

**Files:**
- Create: `crates/datasynth-audit-optimizer/src/risk_scoping.rs`
- Modify: `crates/datasynth-audit-optimizer/src/lib.rs`

- [ ] **Step 1: Define types**

```rust
//! Risk-based audit scoping with coverage analysis and what-if.

use datasynth_audit_fsm::schema::AuditBlueprint;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    pub standards_coverage: f64,
    pub standards_covered: Vec<String>,
    pub standards_uncovered: Vec<String>,
    pub risk_coverage: HashMap<String, f64>,
    pub total_procedures: usize,
    pub included_procedures: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactReport {
    pub removed_procedure: String,
    pub standards_lost: Vec<String>,
    pub standards_coverage_delta: f64,
    pub risk_coverage_delta: HashMap<String, f64>,
    pub dependent_procedures_affected: Vec<String>,
}
```

- [ ] **Step 2: Implement coverage analysis**

```rust
pub fn analyze_coverage(
    blueprint: &AuditBlueprint,
    included_procedures: &[String],
) -> CoverageReport
```

Compute:
- standards_coverage = unique standards in included / total standards
- risk_coverage per category = included procs with tag / total procs with tag

- [ ] **Step 3: Implement what-if analysis**

```rust
pub fn impact_of_removing(
    blueprint: &AuditBlueprint,
    preconditions: &HashMap<String, Vec<String>>,
    current_plan: &[String],
    remove_procedure: &str,
) -> ImpactReport
```

- [ ] **Step 4: Tests**

```rust
#[test]
fn test_full_scope_is_100_percent_coverage()

#[test]
fn test_empty_scope_is_zero_coverage()

#[test]
fn test_removing_procedure_reduces_coverage()

#[test]
fn test_impact_reports_dependent_procedures()

#[test]
fn test_reports_serialize()
```

- [ ] **Step 5: Update lib.rs, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=1
git commit -m "feat(audit-optimizer): add risk-based scoping with coverage and what-if analysis"
```

---

### Task 5: Multi-Engagement Portfolio Simulation

**Files:**
- Create: `crates/datasynth-audit-optimizer/src/portfolio.rs`
- Modify: `crates/datasynth-audit-optimizer/src/lib.rs`
- Modify: `crates/datasynth-audit-optimizer/Cargo.toml` (may need chrono for scheduling)

- [ ] **Step 1: Define portfolio types**

```rust
//! Multi-engagement portfolio simulation.

use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PortfolioConfig {
    pub engagements: Vec<EngagementSpec>,
    pub shared_resources: ResourcePool,
    pub correlation: CorrelationConfig,
}

#[derive(Debug, Clone)]
pub struct EngagementSpec {
    pub entity_id: String,
    pub blueprint: String,      // "fsa", "ia", or file path
    pub overlay: String,        // "default", "thorough", "rushed", or file path
    pub industry: String,
    pub risk_profile: RiskProfile,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum RiskProfile { High, Medium, Low }

#[derive(Debug, Clone)]
pub struct ResourcePool {
    pub roles: HashMap<String, ResourceSlot>,
}

#[derive(Debug, Clone)]
pub struct ResourceSlot {
    pub count: usize,
    pub hours_per_person: f64,
}

#[derive(Debug, Clone)]
pub struct CorrelationConfig {
    pub systemic_finding_probability: f64,
    pub industry_correlation: f64,
}
```

- [ ] **Step 2: Define report types**

```rust
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioReport {
    pub engagement_summaries: Vec<EngagementSummary>,
    pub total_hours: f64,
    pub total_cost: f64,
    pub resource_utilization: HashMap<String, f64>,
    pub scheduling_conflicts: Vec<SchedulingConflict>,
    pub systemic_findings: Vec<SystemicFinding>,
    pub risk_heatmap: Vec<RiskHeatmapEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngagementSummary {
    pub entity_id: String,
    pub blueprint: String,
    pub events: usize,
    pub artifacts: usize,
    pub hours: f64,
    pub findings_count: usize,
    pub completion_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchedulingConflict {
    pub role: String,
    pub overcommitted_hours: f64,
    pub engagements_affected: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemicFinding {
    pub finding_type: String,
    pub industry: String,
    pub affected_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskHeatmapEntry {
    pub entity_id: String,
    pub category: String,
    pub score: f64,
}
```

- [ ] **Step 3: Implement blueprint loader helper**

Add a helper that resolves blueprint/overlay strings (reuse the pattern from CLI):

```rust
fn load_engagement_blueprint(spec: &EngagementSpec) -> Result<BlueprintWithPreconditions, ...> {
    match spec.blueprint.as_str() {
        "fsa" | "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa(),
        "ia" | "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia(),
        path => BlueprintWithPreconditions::load_from_file(PathBuf::from(path)),
    }
}
```

- [ ] **Step 4: Implement portfolio simulation**

```rust
pub fn simulate_portfolio(config: &PortfolioConfig) -> Result<PortfolioReport, AuditFsmError>
```

For each engagement:
1. Load blueprint + overlay
2. Create engine with spec.seed
3. Run engagement
4. Collect summary (events, artifacts, hours, findings)
5. Track resource consumption

After all engagements:
1. Detect scheduling conflicts (total role hours > pool capacity)
2. Propagate systemic findings (same industry + random < probability)
3. Build risk heatmap from procedure discriminator tags
4. Compute resource utilization %

- [ ] **Step 5: Tests**

```rust
#[test]
fn test_single_engagement_portfolio()

#[test]
fn test_two_fsa_engagements()

#[test]
fn test_mixed_fsa_ia_portfolio()

#[test]
fn test_resource_conflict_detected()

#[test]
fn test_systemic_findings_propagate()

#[test]
fn test_portfolio_report_serializes()

#[test]
fn test_portfolio_deterministic()
```

- [ ] **Step 6: Update lib.rs, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=1
git commit -m "feat(audit-optimizer): add multi-engagement portfolio simulation"
```

---

### Task 6: Update Builtin Overlay Presets

**Files:**
- Modify: `crates/datasynth-audit-fsm/overlays/default.yaml`
- Modify: `crates/datasynth-audit-fsm/overlays/thorough.yaml`
- Modify: `crates/datasynth-audit-fsm/overlays/rushed.yaml`

- [ ] **Step 1: Add resource_costs to all overlays**

`default.yaml` additions:
```yaml
resource_costs:
  cost_multiplier: 1.0
  role_hourly_rates:
    engagement_partner: 500
    audit_manager: 300
    audit_senior: 200
    audit_staff: 120
    cae: 400
    audit_director: 350
    senior_auditor: 200
    staff_auditor: 120
```

`thorough.yaml`: `cost_multiplier: 1.5`
`rushed.yaml`: `cost_multiplier: 0.6`

- [ ] **Step 2: Verify overlays parse**

```bash
cargo test -p datasynth-audit-fsm -- --test-threads=1 test_load_builtin
```

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(audit-fsm): add resource costs and iteration limits to overlay presets"
```

---

### Task 7: Final Validation and Cleanup

- [ ] **Step 1: Run fmt and clippy**

```bash
cargo fmt --all
cargo clippy -p datasynth-audit-fsm -p datasynth-audit-optimizer
```

- [ ] **Step 2: Run all tests sequentially**

```bash
cargo test -p datasynth-audit-fsm -- --test-threads=1
cargo test -p datasynth-audit-optimizer -- --test-threads=1
```

- [ ] **Step 3: Run evaluation**

```bash
cargo test -p datasynth-audit-fsm --test evaluate_output -- --nocapture --test-threads=1
```

Verify IA completion rate improved from 22/34 to 28+/34.

- [ ] **Step 4: Commit if needed**

```bash
git commit -m "feat(wave2): finalize Wave 2 — audit planning optimization"
```

---

## Summary

| Task | What it delivers | Key files |
|------|-----------------|-----------|
| 1 | Per-procedure iteration limits (20→30 default) | `schema.rs`, `engine.rs`, overlays |
| 2 | Cost model (base hours + multipliers + rates) | `schema.rs`, blueprints, overlays |
| 3 | Resource-constrained optimization | `resource_optimizer.rs` |
| 4 | Risk-based scoping with coverage/what-if | `risk_scoping.rs` |
| 5 | Multi-engagement portfolio simulation | `portfolio.rs` |
| 6 | Overlay preset updates | `overlays/*.yaml` |
| 7 | Final validation | All files |
