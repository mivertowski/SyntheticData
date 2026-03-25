# Wave 4: Learned & Adaptive Generation (v2.0.0) Design Spec

**Date**: 2026-03-25
**Status**: Approved
**Scope**: Overlay parameter fitting from target engagement metrics, blueprint discovery from event logs, adaptive anomaly calibration, and LLM integration interface.

## Problem

The audit FSM engine uses hand-authored overlays (default/thorough/rushed). Real audit engagements have measurable characteristics (duration, finding count, revision frequency) that don't match any single preset. There's no way to:
- Fit overlay parameters to reproduce observed engagement profiles
- Discover methodology blueprints from existing event logs
- Auto-calibrate anomaly rates to target detection difficulty
- Generate contextually rich artifact content (narratives are template-based)

## Solution

Four modules across `datasynth-audit-optimizer` and `datasynth-audit-fsm`.

---

## 1. Overlay Parameter Fitting

Given target engagement metrics, find overlay parameters that produce similar output.

### Input

```rust
pub struct EngagementProfile {
    pub target_duration_hours: f64,
    pub target_event_count: usize,
    pub target_finding_count: usize,
    pub target_revision_rate: f64,    // fraction of procedures with revisions
    pub target_anomaly_rate: f64,
    pub target_completion_rate: f64,  // fraction of procedures completing
}
```

### Algorithm

Iterative parameter search:
1. Start with default overlay
2. Run N Monte Carlo simulations (N=10 for speed)
3. Compute mean metrics
4. Adjust parameters toward target:
   - If duration too short → increase `timing.mu_hours`
   - If too few revisions → increase `revision_probability`
   - If too few anomalies → increase anomaly rates
   - If too few findings → increase anomaly rates (findings correlate)
5. Repeat for K iterations (K=20) or until convergence (delta < 5%)
6. Return fitted overlay

### Output

```rust
pub struct FittedOverlay {
    pub overlay: GenerationOverlay,
    pub achieved_metrics: EngagementMetrics,
    pub target_metrics: EngagementProfile,
    pub iterations: usize,
    pub converged: bool,
    pub residual: f64,  // normalized distance from target
}

pub struct EngagementMetrics {
    pub avg_duration_hours: f64,
    pub avg_event_count: f64,
    pub avg_finding_count: f64,
    pub avg_revision_rate: f64,
    pub avg_anomaly_rate: f64,
    pub avg_completion_rate: f64,
}
```

### Files
- Create: `crates/datasynth-audit-optimizer/src/overlay_fitting.rs`

---

## 2. Blueprint Discovery from Event Logs

Given a `Vec<AuditEvent>`, infer the underlying procedure state machines.

### Algorithm (Alpha Miner variant)

1. Group events by `procedure_id`
2. For each procedure:
   a. Extract unique states from `from_state` and `to_state`
   b. Extract transitions: each `(from_state, to_state)` pair becomes a transition
   c. Identify initial state (first `from_state` seen)
   d. Identify terminal states (states with no outgoing transitions)
3. Reconstruct phases from `phase_id` grouping
4. Build a `DiscoveredBlueprint` (subset of `AuditBlueprint`)

### Output

```rust
pub struct DiscoveredBlueprint {
    pub procedures: Vec<DiscoveredProcedure>,
    pub phases: Vec<String>,
    pub total_events_analyzed: usize,
}

pub struct DiscoveredProcedure {
    pub id: String,
    pub phase: String,
    pub states: Vec<String>,
    pub transitions: Vec<(String, String)>,  // (from, to)
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub event_count: usize,
}
```

### Blueprint Comparison

Compare a discovered blueprint against a reference:
```rust
pub struct BlueprintDiff {
    pub matching_procedures: Vec<String>,
    pub missing_procedures: Vec<String>,   // in reference but not discovered
    pub extra_procedures: Vec<String>,      // discovered but not in reference
    pub transition_diffs: Vec<TransitionDiff>,
    pub conformance_score: f64,
}
```

### Files
- Create: `crates/datasynth-audit-optimizer/src/discovery.rs`

---

## 3. Adaptive Anomaly Calibration

Given a target anomaly detection difficulty (F1 score range), adjust injection parameters.

### Algorithm

1. Start with base anomaly rates from overlay
2. Generate benchmark dataset
3. Compute conformance (anomaly events / total events)
4. If rate too low → increase probabilities
5. If rate too high → decrease probabilities
6. Converge on target rate within tolerance

### Types

```rust
pub struct CalibrationTarget {
    pub target_anomaly_rate: f64,     // e.g., 0.15 (15% of events)
    pub tolerance: f64,               // e.g., 0.02 (±2%)
    pub max_iterations: usize,
}

pub struct CalibratedOverlay {
    pub overlay: GenerationOverlay,
    pub achieved_rate: f64,
    pub iterations: usize,
    pub converged: bool,
}

pub fn calibrate_anomaly_rates(
    blueprint: &BlueprintWithPreconditions,
    target: &CalibrationTarget,
    base_seed: u64,
) -> Result<CalibratedOverlay, AuditFsmError>
```

### Files
- Create: `crates/datasynth-audit-optimizer/src/calibration.rs`

---

## 4. LLM Integration Interface

Define the trait interface for LLM-augmented artifact content. Ship with a template-based default implementation. Actual LLM backends are pluggable.

### Trait

```rust
pub trait ContentGenerator: Send + Sync {
    fn generate_finding_narrative(&self, context: &FindingContext) -> String;
    fn generate_workpaper_narrative(&self, context: &WorkpaperContext) -> String;
    fn generate_management_response(&self, context: &ResponseContext) -> String;
}

pub struct FindingContext {
    pub procedure_id: String,
    pub step_id: String,
    pub standards_refs: Vec<String>,
    pub finding_type: String,
}

// Default: template-based (no LLM needed)
pub struct TemplateContentGenerator;
impl ContentGenerator for TemplateContentGenerator { ... }
```

### Files
- Create: `crates/datasynth-audit-fsm/src/content.rs`

---

## 5. Testing

- Overlay fitting: target profile → fitted overlay achieves metrics within 20% tolerance
- Blueprint discovery: generate FSA events → discover blueprint → compare against FSA → conformance > 0.8
- Anomaly calibration: target 15% rate → achieved rate within ±2%
- Content generator: template impl produces non-empty strings
- Full pipeline: fit overlay → generate → discover → compare
- All tests `--test-threads=1`
