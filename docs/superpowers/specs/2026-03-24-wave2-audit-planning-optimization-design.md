# Wave 2: Audit Planning Optimization (v1.7.0) Design Spec

**Date**: 2026-03-24
**Status**: Approved
**Scope**: Per-procedure iteration limits, cost model (blueprint + overlay), resource-constrained optimization, risk-based audit scoping, and multi-engagement portfolio simulation.

## Problem

The audit FSM engine (v1.5.0) generates realistic audit trails and artifacts but lacks planning intelligence:

1. **Iteration limit cap**: Global `MAX_ITERATIONS=20` prevents 12/34 IA procedures from completing. Continuous-phase procedures with revision loops are disproportionately affected.
2. **No cost model**: Procedures have no associated hour estimates or role requirements, preventing budget-based planning.
3. **No resource constraints**: Cannot model staff availability, budget limits, or role capacity.
4. **No coverage analysis**: Cannot assess what percentage of ISA/IIA-GIAS standards a given scope addresses.
5. **No portfolio view**: Cannot simulate multiple engagements with shared resources and correlated risks.

## Solution

Six work areas across `datasynth-audit-fsm` (engine fixes, schema additions) and `datasynth-audit-optimizer` (planning modules).

---

## 1. Per-Procedure Iteration Limits

### Problem

`MAX_ITERATIONS=20` is a compile-time constant in `engine.rs`. IA procedures with high revision probability or self-loops hit this cap, ending in `under_review` instead of `completed`. With default overlay (revision_probability=0.15), only 22/34 IA procedures complete.

### Solution

Replace global constant with configurable per-procedure limits in the overlay:

```yaml
iteration_limits:
  default: 30
  per_procedure:
    maintain_competency: 50
    monitor_action_plans: 40
```

### Schema

Add to `GenerationOverlay`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationLimits {
    #[serde(default = "default_iteration_limit")]
    pub default: usize,
    #[serde(default)]
    pub per_procedure: HashMap<String, usize>,
}

fn default_iteration_limit() -> usize { 30 }

impl Default for IterationLimits {
    fn default() -> Self {
        Self { default: 30, per_procedure: HashMap::new() }
    }
}
```

### Engine Change

In `engine.rs`, replace:
```rust
const MAX_ITERATIONS: usize = 20;
// ...
if iterations >= MAX_ITERATIONS { break; }
```

With:
```rust
let max_iter = self.overlay.iteration_limits
    .per_procedure.get(proc_id)
    .copied()
    .unwrap_or(self.overlay.iteration_limits.default);
if iterations >= max_iter { break; }
```

### Expected Impact

IA completion rate: 22/34 (65%) → 30+/34 (88%+) with default=30.

### Files Modified
- `crates/datasynth-audit-fsm/src/schema.rs` — add `IterationLimits`, add field to `GenerationOverlay`
- `crates/datasynth-audit-fsm/src/engine.rs` — replace `MAX_ITERATIONS` with overlay lookup
- `crates/datasynth-audit-fsm/overlays/*.yaml` — add `iteration_limits` section

---

## 2. Cost Model (Blueprint + Overlay)

### Blueprint Base Hours

Add optional cost metadata to `BlueprintProcedure`:

```rust
// In schema.rs
pub struct BlueprintProcedure {
    // ... existing fields ...
    #[serde(default)]
    pub base_hours: Option<f64>,
    #[serde(default)]
    pub required_roles: Vec<String>,
}
```

The FSA and IA blueprint YAMLs get `base_hours` annotations on each procedure. If absent, a default of 8.0 hours is assumed.

### Overlay Cost Multipliers

```yaml
resource_costs:
  cost_multiplier: 1.0
  role_hourly_rates:
    engagement_partner: 500
    audit_manager: 300
    audit_senior: 200
    audit_staff: 120
  per_procedure_multipliers:
    substantive_testing: 1.3
```

### Schema

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
```

### Cost Calculation

```
effective_hours(proc) = proc.base_hours.unwrap_or(8.0)
                       * overlay.resource_costs.cost_multiplier
                       * overlay.resource_costs.per_procedure_multipliers.get(proc.id).unwrap_or(&1.0)

procedure_cost(proc, role) = effective_hours(proc) * overlay.resource_costs.role_hourly_rates.get(role).unwrap_or(&200.0)
```

### Files Modified
- `crates/datasynth-audit-fsm/src/schema.rs` — add `ResourceCosts`, fields on `BlueprintProcedure` and `GenerationOverlay`
- `crates/datasynth-audit-fsm/blueprints/generic_fsa.yaml` — add `base_hours` to procedures
- `crates/datasynth-audit-fsm/blueprints/generic_ia.yaml` — add `base_hours` to procedures
- `crates/datasynth-audit-fsm/overlays/*.yaml` — add `resource_costs` section

---

## 3. Resource-Constrained Optimization

New module in `datasynth-audit-optimizer`: `resource_optimizer.rs`

### Input

```rust
pub struct ResourceConstraints {
    pub total_budget_hours: f64,
    pub role_availability: HashMap<String, f64>,
    pub must_include: Vec<String>,
    pub must_exclude: Vec<String>,
}
```

### Output

```rust
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

### Algorithm

1. Start with must-include procedures
2. Expand with transitive precondition dependencies (reuses `constrained_path`)
3. Compute remaining budget after mandatory procedures
4. Score remaining procedures by `risk_coverage_per_hour = risk_tags.len() / effective_hours`
5. Greedily add procedures in descending score order until budget exhausted
6. Compute critical path: longest chain of dependent procedures by cumulative hours

### Files Created
- `crates/datasynth-audit-optimizer/src/resource_optimizer.rs`

---

## 4. Risk-Based Audit Scoping

New module: `risk_scoping.rs`

### Coverage Analysis

```rust
pub struct CoverageReport {
    pub standards_coverage: f64,
    pub standards_covered: Vec<String>,
    pub standards_uncovered: Vec<String>,
    pub risk_coverage: HashMap<String, f64>,  // category → % covered
    pub total_procedures: usize,
    pub included_procedures: usize,
}

pub fn analyze_coverage(
    blueprint: &AuditBlueprint,
    included_procedures: &[String],
) -> CoverageReport;
```

Standards coverage = (unique standards referenced by included procedures) / (total standards in blueprint).

Risk coverage per category = (included procedures tagged with category) / (total procedures tagged with category).

### What-If Analysis

```rust
pub struct ImpactReport {
    pub removed_procedure: String,
    pub standards_lost: Vec<String>,
    pub standards_coverage_delta: f64,
    pub risk_coverage_delta: HashMap<String, f64>,
    pub dependent_procedures_affected: Vec<String>,
}

pub fn impact_of_removing(
    blueprint: &AuditBlueprint,
    preconditions: &HashMap<String, Vec<String>>,
    current_plan: &[String],
    remove_procedure: &str,
) -> ImpactReport;
```

### Files Created
- `crates/datasynth-audit-optimizer/src/risk_scoping.rs`

---

## 5. Multi-Engagement Portfolio Simulation

New module: `portfolio.rs`

### Portfolio Configuration

```rust
pub struct PortfolioConfig {
    pub engagements: Vec<EngagementSpec>,
    pub shared_resources: ResourcePool,
    pub correlation: CorrelationConfig,
}

pub struct EngagementSpec {
    pub entity_id: String,
    pub blueprint: String,
    pub overlay: String,
    pub industry: String,
    pub risk_profile: RiskProfile,
    pub seed: u64,
}

pub enum RiskProfile { High, Medium, Low }

pub struct ResourcePool {
    pub roles: HashMap<String, ResourceSlot>,
}

pub struct ResourceSlot {
    pub count: usize,
    pub hours_per_person: f64,
    pub max_concurrent_engagements: usize,
}

pub struct CorrelationConfig {
    pub systemic_finding_probability: f64,
    pub industry_correlation: f64,
}
```

### Simulation

```rust
pub fn simulate_portfolio(
    config: &PortfolioConfig,
) -> Result<PortfolioReport, AuditFsmError>;
```

For each engagement:
1. Load blueprint + overlay from spec
2. Create FSM engine with engagement-specific seed
3. Run engagement, collect artifacts and events
4. Track resource consumption against shared pool
5. Apply systemic findings: if another engagement in the same industry produced a finding, probabilistically inject it

### Portfolio Report

```rust
pub struct PortfolioReport {
    pub engagement_summaries: Vec<EngagementSummary>,
    pub total_hours: f64,
    pub total_cost: f64,
    pub resource_utilization: HashMap<String, f64>,
    pub scheduling_conflicts: Vec<SchedulingConflict>,
    pub systemic_findings: Vec<SystemicFinding>,
    pub risk_heatmap: Vec<RiskHeatmapEntry>,
}

pub struct EngagementSummary {
    pub entity_id: String,
    pub events: usize,
    pub artifacts: usize,
    pub hours: f64,
    pub cost: f64,
    pub findings_count: usize,
    pub completion_rate: f64,
}

pub struct SchedulingConflict {
    pub role: String,
    pub period: String,
    pub overcommitted_hours: f64,
    pub engagements_affected: Vec<String>,
}

pub struct SystemicFinding {
    pub finding_type: String,
    pub industry: String,
    pub affected_entities: Vec<String>,
}

pub struct RiskHeatmapEntry {
    pub entity_id: String,
    pub category: String,
    pub score: f64,
}
```

### Files Created
- `crates/datasynth-audit-optimizer/src/portfolio.rs`

---

## 6. Testing

### Iteration Limits
- IA completion rate >= 88% with default=30
- Custom per-procedure limit respected
- Existing FSA tests unaffected

### Cost Model
- `effective_hours` calculation with multipliers
- Default when `base_hours` absent
- Blueprint YAML roundtrip with `base_hours`

### Resource Optimizer
- Must-include procedures + preconditions within budget
- Budget exhaustion stops adding procedures
- Critical path computation

### Risk Scoping
- Coverage % matches manual count
- What-if removal reports correct standards lost
- Empty plan = 0% coverage

### Portfolio
- N engagements run independently
- Resource utilization calculated correctly
- Systemic findings propagate between same-industry engagements
- Scheduling conflicts detected when role overcommitted

### All tests: `--test-threads=1`, sequential crate testing only

---

## Dependencies

- `datasynth-audit-fsm`: schema + engine changes (iteration limits, cost fields)
- `datasynth-audit-optimizer`: 3 new modules (resource_optimizer, risk_scoping, portfolio)
- No new crate dependencies (portfolio uses existing FSM engine)

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Iteration limit increase slows generation | Longer IA runs | Default 30 is moderate; profiled at ~0.5s for full IA |
| Blueprint YAML changes break loading | Parse failures | `base_hours` and `required_roles` are optional with defaults |
| Portfolio simulation is slow for large N | User frustration | Sequential execution, progress reporting |
| Greedy optimizer misses global optimum | Suboptimal plans | Acceptable for planning tool; exact optimization is NP-hard |
