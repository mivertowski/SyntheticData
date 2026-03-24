# Audit FSM Phase 2: IA Blueprint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `datasynth-audit-fsm` to load and execute the Internal Audit (IIA-GIAS) blueprint, proving the engine generalizes beyond FSA to support 8-state C2CE lifecycles, self-loops, continuous phases, and discriminator filtering.

**Architecture:** The IA blueprint has 9 phases (3 continuous with negative `order`), 34 procedures, 82 steps, and uses discriminators for scope filtering. The engine needs schema additions (phase order, discriminators), continuous phase detection, self-loop bounding, and discriminator-based procedure filtering. OCEL 2.0 projection exporter maps audit events to the existing `datasynth-ocpm` format.

**Tech Stack:** Existing `datasynth-audit-fsm` crate, `datasynth-ocpm` for OCEL types, serde_yaml, ChaCha8Rng.

**Spec:** `docs/superpowers/specs/2026-03-24-audit-fsm-integration-design.md` (Phase 2 section)

**Phase 1 baseline:** 32 tests passing, 15 files in crate.

---

### Task 1: Schema Additions for IA Support

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/schema.rs`

The IA blueprint requires three schema additions that the FSA blueprint didn't need.

- [ ] **Step 1: Write failing tests**

Add to `schema.rs` tests module:

```rust
#[test]
fn test_phase_with_order_deserialize() {
    let yaml = r#"
id: ethics
name: "Ethics & Professionalism"
order: -2
"#;
    let phase: BlueprintPhase = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(phase.order, Some(-2));
}

#[test]
fn test_phase_without_order_defaults_none() {
    let yaml = r#"
id: planning
name: Planning
"#;
    let phase: BlueprintPhase = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(phase.order, None);
}

#[test]
fn test_blueprint_discriminators_deserialize() {
    let yaml = r#"
id: test
name: Test
version: "1.0"
methodology:
  framework: ISA
discriminators:
  categories: [financial, operational]
  risk_ratings: [high, medium, low]
"#;
    let bp: AuditBlueprint = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(bp.discriminators.len(), 2);
    assert_eq!(bp.discriminators["categories"].len(), 2);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: FAIL — fields don't exist yet

- [ ] **Step 3: Add fields to schema types**

In `BlueprintPhase`, add:
```rust
/// Phase execution order. Negative values indicate continuous phases
/// that run in parallel. None means no ordering constraint.
#[serde(default)]
pub order: Option<i32>,
```

In `AuditBlueprint`, add:
```rust
/// Discriminator categories for scope filtering (e.g. categories, risk_ratings, engagement_types).
#[serde(default)]
pub discriminators: HashMap<String, Vec<String>>,
```

In `BlueprintProcedure`, add:
```rust
/// Discriminators that apply to this procedure for scope filtering.
#[serde(default)]
pub discriminators: HashMap<String, Vec<String>>,
```

In `GenerationOverlay`, add:
```rust
/// Maximum iterations for self-loop states (prevents infinite cycling).
#[serde(default = "default_max_self_loop_iterations")]
pub max_self_loop_iterations: usize,
```

Add default function:
```rust
fn default_max_self_loop_iterations() -> usize {
    5
}
```

- [ ] **Step 4: Update loader conversion**

Read `loader.rs` and find the `convert_raw_*` functions. The `RawPhase` already has `order: Option<u32>`. Update the conversion to pass `order` through to `BlueprintPhase` (cast to `Option<i32>` since IA uses negative values — also update `RawPhase.order` to `Option<i32>`).

Similarly wire through discriminators from `RawProcedure` and `RawBlueprint` to the canonical types.

- [ ] **Step 5: Run tests, verify pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS (existing + 3 new)

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-audit-fsm/src/schema.rs crates/datasynth-audit-fsm/src/loader.rs
git commit -m "feat(audit-fsm): add phase order, discriminators, and self-loop config to schema"
```

---

### Task 2: Copy IA Blueprint and Verify Loading

**Files:**
- Create: `crates/datasynth-audit-fsm/blueprints/generic_ia.yaml`
- Modify: `crates/datasynth-audit-fsm/src/loader.rs`

- [ ] **Step 1: Copy the IA blueprint**

```bash
cp /home/michael/DEV/Repos/Methodology/AuditMethodology/docs/blueprints/generic_ia.yaml \
   crates/datasynth-audit-fsm/blueprints/generic_ia.yaml
```

- [ ] **Step 2: Add builtin IA variant**

In `loader.rs`, add:
```rust
const BUILTIN_IA: &str = include_str!("../blueprints/generic_ia.yaml");
```

Add `Ia` to `BuiltinBlueprint` enum and handle in `load_blueprint`:
```rust
pub enum BuiltinBlueprint {
    Fsa,
    Ia,
}
// In load_blueprint match:
BuiltinBlueprint::Ia => BUILTIN_IA,
```

Add `BlueprintWithPreconditions::load_builtin_ia()` method following the pattern of `load_builtin_fsa()`.

- [ ] **Step 3: Write loading test**

```rust
#[test]
fn test_load_ia_blueprint_parses() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    assert_eq!(bwp.blueprint.methodology.framework, "IIA-GIAS");
    assert!(bwp.blueprint.phases.len() >= 9, "Expected >= 9 phases");
    // Count total procedures across all phases
    let total_procs: usize = bwp.blueprint.phases.iter()
        .map(|p| p.procedures.len()).sum();
    assert!(total_procs >= 30, "Expected >= 30 procedures, got {}", total_procs);
}

#[test]
fn test_ia_validates_successfully() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let result = bwp.validate();
    assert!(result.is_ok(), "IA validation failed: {:?}", result.err());
}

#[test]
fn test_ia_topological_sort() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let order = bwp.topological_sort().unwrap();
    assert!(order.len() >= 30, "Expected >= 30 procedures in sort order");
}
```

- [ ] **Step 4: Run tests — if parsing fails, fix the loader conversion**

The IA blueprint may use YAML fields not yet handled by the raw→canonical conversion. Common issues:
- Discriminator fields at procedure level
- Phase `order` as negative integer
- Steps with `decisions` (plural) vs `decision` (singular)
- Steps with `guards` as structured objects vs strings

Read the error messages and fix the raw types / conversion as needed.

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/blueprints/generic_ia.yaml crates/datasynth-audit-fsm/src/loader.rs
git commit -m "feat(audit-fsm): add IA blueprint (IIA-GIAS) with loader support"
```

---

### Task 3: Continuous Phase Support

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/engine.rs`

Continuous phases have negative `order` values and should:
- Not block sequential phase execution
- Have their procedures executed but not require gate completion
- Be excluded from `phases_completed` gate checking

- [ ] **Step 1: Write tests**

```rust
#[test]
fn test_continuous_phases_not_required_for_completion() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx).unwrap();

    // Continuous phases (order < 0) should not block other phases
    // Sequential phases should still complete
    let sequential_completed: Vec<&String> = result.phases_completed.iter()
        .filter(|p| {
            // Check if this phase has positive order
            engine.blueprint.phases.iter()
                .find(|ph| &ph.id == *p)
                .and_then(|ph| ph.order)
                .map(|o| o > 0)
                .unwrap_or(false)
        })
        .collect();
    assert!(!sequential_completed.is_empty(), "Expected some sequential phases to complete");
}
```

- [ ] **Step 2: Implement continuous phase handling**

In `engine.rs`, modify `run_engagement`:

1. Before the procedure walk loop, partition phases into continuous vs sequential:
```rust
let continuous_phase_ids: HashSet<String> = self.blueprint.phases.iter()
    .filter(|p| p.order.map(|o| o < 0).unwrap_or(false))
    .map(|p| p.id.clone())
    .collect();
```

2. In `compute_completed_phases`, skip gate checks for continuous phases — they're always "active":
```rust
// Continuous phases are always considered active, not "completed"
if continuous_phase_ids.contains(&phase.id) {
    continue; // Don't add to completed list
}
```

- [ ] **Step 3: Run tests, verify pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-audit-fsm/src/engine.rs
git commit -m "feat(audit-fsm): add continuous phase support (negative order)"
```

---

### Task 4: Self-Loop Handling

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/engine.rs`

The IA blueprint's `monitor_action_plans` has `follow_up → follow_up` self-loops. The engine needs to:
- Detect self-loop transitions (from_state == to_state)
- Bound them with configurable max iterations from overlay
- Still execute steps on each self-loop iteration

- [ ] **Step 1: Write test**

```rust
#[test]
fn test_self_loop_bounded() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay(); // max_self_loop_iterations = 5
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx).unwrap();

    // Count self-loop transitions (from_state == to_state) for any procedure
    let self_loop_events: Vec<&AuditEvent> = result.event_log.iter()
        .filter(|e| e.from_state.as_ref() == e.to_state.as_ref() && e.from_state.is_some())
        .collect();

    // Self-loops should be bounded (not more than max_self_loop_iterations per procedure)
    // With default overlay max_self_loop_iterations=5, no single procedure should loop more than 5 times
    let mut loop_counts: HashMap<String, usize> = HashMap::new();
    for e in &self_loop_events {
        *loop_counts.entry(e.procedure_id.clone()).or_default() += 1;
    }
    for (proc_id, count) in &loop_counts {
        assert!(*count <= 5, "Procedure {} had {} self-loops, max is 5", proc_id, count);
    }
}
```

- [ ] **Step 2: Implement self-loop detection and bounding**

In the FSM walk loop in `run_engagement`, add self-loop tracking:

```rust
let mut self_loop_count: usize = 0;

// Inside the loop, after selecting transition:
if transition.from_state == transition.to_state {
    self_loop_count += 1;
    if self_loop_count >= self.overlay.max_self_loop_iterations {
        // Force forward: find a non-self-loop transition if available
        if let Some(forward) = outgoing.iter().find(|t| t.from_state != t.to_state) {
            transition = forward;
            self_loop_count = 0;
        } else {
            break; // No forward transition available, terminate
        }
    }
} else {
    self_loop_count = 0; // Reset on non-self-loop
}
```

Also modify `execute_steps` to run on self-loop iterations: currently steps only execute when entering `in_progress` from a different state. For self-loops, steps should execute on each iteration. Add logic to detect self-loops and re-execute steps.

- [ ] **Step 3: Run tests, verify pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-audit-fsm/src/engine.rs
git commit -m "feat(audit-fsm): add self-loop detection and bounding"
```

---

### Task 5: Discriminator Filtering

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/engine.rs`

The overlay's `discriminators` field filters which procedures execute. A procedure is skipped if its discriminators don't match the overlay filter.

- [ ] **Step 1: Write test**

```rust
#[test]
fn test_discriminator_filtering_reduces_procedures() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();

    // Create overlay that only includes "financial" category
    let mut overlay = default_overlay();
    let mut disc = HashMap::new();
    disc.insert("categories".to_string(), vec!["financial".to_string()]);
    overlay.discriminators = Some(disc);

    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp.clone(), overlay, rng);
    let ctx = EngagementContext::test_default();
    let filtered_result = engine.run_engagement(&ctx).unwrap();

    // Run again without discriminator filter
    let mut engine2 = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let full_result = engine2.run_engagement(&ctx).unwrap();

    // Filtered run should have fewer or equal procedures executed
    assert!(
        filtered_result.procedure_states.len() <= full_result.procedure_states.len(),
        "Filtered ({}) should have <= procedures than full ({})",
        filtered_result.procedure_states.len(),
        full_result.procedure_states.len()
    );
}
```

- [ ] **Step 2: Implement discriminator filtering**

In `run_engagement`, before executing a procedure, check discriminator compatibility:

```rust
fn should_skip_procedure(&self, proc: &BlueprintProcedure) -> bool {
    let Some(ref overlay_discs) = self.overlay.discriminators else {
        return false; // No filter = run everything
    };

    if proc.discriminators.is_empty() {
        return false; // No discriminators on procedure = always run
    }

    // For each discriminator category in the overlay filter,
    // check if the procedure has at least one matching value
    for (category, filter_values) in overlay_discs {
        if let Some(proc_values) = proc.discriminators.get(category) {
            if !proc_values.iter().any(|v| filter_values.contains(v)) {
                return true; // No match in this category = skip
            }
        }
    }
    false
}
```

Call this at the start of the procedure walk loop and `continue` if it returns true.

- [ ] **Step 3: Run tests, verify pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-audit-fsm/src/engine.rs
git commit -m "feat(audit-fsm): add discriminator-based procedure filtering"
```

---

### Task 6: IA Integration Tests

**Files:**
- Create: `crates/datasynth-audit-fsm/tests/ia_integration.rs`

- [ ] **Step 1: Write integration tests**

```rust
//! Integration tests for the IA (Internal Audit) blueprint.

use std::collections::HashMap;
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::export::flat_log::export_events_to_json;
use datasynth_audit_fsm::loader::{BlueprintWithPreconditions, default_overlay, parse_overlay};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn ctx() -> EngagementContext {
    EngagementContext::test_default()
}

#[test]
fn test_ia_full_engagement() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    bwp.validate().unwrap();
    let overlay = default_overlay();
    let mut engine = AuditFsmEngine::new(bwp, overlay, ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // IA has 34 procedures — expect many events
    assert!(result.event_log.len() >= 50, "Expected >= 50 events, got {}", result.event_log.len());

    // Multiple phases should complete
    assert!(!result.phases_completed.is_empty(), "Expected some phases to complete");

    // Events ordered by timestamp
    for window in result.event_log.windows(2) {
        assert!(window[0].timestamp <= window[1].timestamp);
    }

    // JSON export works
    let json = export_events_to_json(&result.event_log).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), result.event_log.len());

    // Duration positive
    assert!(result.total_duration_hours > 0.0);
}

#[test]
fn test_ia_determinism() {
    let c = ctx();
    let mut e1 = AuditFsmEngine::new(
        BlueprintWithPreconditions::load_builtin_ia().unwrap(),
        default_overlay(),
        ChaCha8Rng::seed_from_u64(99),
    );
    let mut e2 = AuditFsmEngine::new(
        BlueprintWithPreconditions::load_builtin_ia().unwrap(),
        default_overlay(),
        ChaCha8Rng::seed_from_u64(99),
    );

    let r1 = e1.run_engagement(&c).unwrap();
    let r2 = e2.run_engagement(&c).unwrap();

    assert_eq!(r1.event_log.len(), r2.event_log.len());
    for (a, b) in r1.event_log.iter().zip(r2.event_log.iter()) {
        assert_eq!(a.event_id, b.event_id);
        assert_eq!(a.event_type, b.event_type);
        assert_eq!(a.timestamp, b.timestamp);
    }
}

#[test]
fn test_ia_c2ce_lifecycle_events() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // The develop_findings procedure should produce events through the C2CE states
    let finding_events: Vec<_> = result.event_log.iter()
        .filter(|e| e.procedure_id == "develop_findings")
        .collect();
    // Should have multiple transition events (8 states = at least 7 transitions)
    assert!(finding_events.len() >= 5, "Expected >= 5 events for develop_findings, got {}", finding_events.len());
}

#[test]
fn test_ia_with_financial_only_discriminator() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let mut overlay = default_overlay();
    let mut disc = HashMap::new();
    disc.insert("categories".to_string(), vec!["financial".to_string()]);
    overlay.discriminators = Some(disc);

    let mut engine = AuditFsmEngine::new(bwp, overlay, ChaCha8Rng::seed_from_u64(42));
    let result = engine.run_engagement(&ctx()).unwrap();

    // Should have fewer procedures than a full run
    assert!(!result.procedure_states.is_empty(), "Should have some procedures even with filter");
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`

If the IA blueprint parsing or engine execution fails, fix the issues. Common problems:
- YAML fields not mapped in raw types
- Gate predicate format differences
- Procedures with empty aggregates

- [ ] **Step 3: Commit**

```bash
git add crates/datasynth-audit-fsm/tests/ia_integration.rs
git commit -m "test(audit-fsm): add IA blueprint integration tests"
```

---

### Task 7: OCEL 2.0 Projection Exporter

**Files:**
- Create: `crates/datasynth-audit-fsm/src/export/ocel.rs`
- Modify: `crates/datasynth-audit-fsm/src/export/mod.rs`

Maps audit events to OCEL 2.0 format for process mining analysis.

- [ ] **Step 1: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocel_projection_produces_valid_output() {
        use crate::context::EngagementContext;
        use crate::engine::AuditFsmEngine;
        use crate::loader::{BlueprintWithPreconditions, default_overlay};
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let mut engine = AuditFsmEngine::new(bwp, default_overlay(), ChaCha8Rng::seed_from_u64(42));
        let result = engine.run_engagement(&EngagementContext::test_default()).unwrap();

        let ocel = project_to_ocel(&result.event_log);
        assert!(!ocel.events.is_empty());
        assert!(!ocel.object_types.is_empty());
        // Procedures become object types
        assert!(ocel.object_types.contains(&"accept_engagement".to_string()));

        let json = serde_json::to_string_pretty(&ocel).unwrap();
        assert!(json.contains("ocel_version"));
    }
}
```

- [ ] **Step 2: Implement OCEL projection**

The OCEL 2.0 format is a JSON structure with `events`, `objects`, and `object_types`. We produce a standalone OCEL output (not depending on `datasynth-ocpm` types to keep the dependency light).

```rust
//! OCEL 2.0 projection of audit events for process mining.

use crate::event::AuditEvent;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Minimal OCEL 2.0 compatible output.
#[derive(Debug, Clone, Serialize)]
pub struct OcelProjection {
    pub ocel_version: String,
    pub object_types: Vec<String>,
    pub events: Vec<OcelEvent>,
    pub objects: Vec<OcelObject>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcelEvent {
    pub id: String,
    pub activity: String,
    pub timestamp: String,
    pub omap: Vec<String>,      // object IDs this event relates to
    pub vmap: HashMap<String, String>, // additional attributes
}

#[derive(Debug, Clone, Serialize)]
pub struct OcelObject {
    pub id: String,
    pub object_type: String,
    pub attributes: HashMap<String, String>,
}

/// Project audit events to OCEL 2.0 format.
pub fn project_to_ocel(events: &[AuditEvent]) -> OcelProjection {
    let mut object_types: HashSet<String> = HashSet::new();
    let mut objects: HashMap<String, OcelObject> = HashMap::new();
    let mut ocel_events = Vec::new();

    for event in events {
        // Each procedure is an object type
        object_types.insert(event.procedure_id.clone());

        // Create object for the procedure instance
        let obj_id = format!("proc_{}", event.procedure_id);
        objects.entry(obj_id.clone()).or_insert_with(|| OcelObject {
            id: obj_id.clone(),
            object_type: event.procedure_id.clone(),
            attributes: HashMap::new(),
        });

        // Create objects for evidence
        for ev_ref in &event.evidence_refs {
            let ev_obj_id = format!("evidence_{}", ev_ref);
            object_types.insert("evidence".to_string());
            objects.entry(ev_obj_id.clone()).or_insert_with(|| OcelObject {
                id: ev_obj_id.clone(),
                object_type: "evidence".to_string(),
                attributes: {
                    let mut m = HashMap::new();
                    m.insert("ref".to_string(), ev_ref.clone());
                    m
                },
            });
        }

        // Map event
        let activity = event.command.clone();
        let mut omap = vec![format!("proc_{}", event.procedure_id)];
        for ev_ref in &event.evidence_refs {
            omap.push(format!("evidence_{}", ev_ref));
        }

        let mut vmap = HashMap::new();
        vmap.insert("phase".to_string(), event.phase_id.clone());
        vmap.insert("actor".to_string(), event.actor_id.clone());
        if let Some(ref from) = event.from_state {
            vmap.insert("from_state".to_string(), from.clone());
        }
        if let Some(ref to) = event.to_state {
            vmap.insert("to_state".to_string(), to.clone());
        }

        ocel_events.push(OcelEvent {
            id: event.event_id.to_string(),
            activity,
            timestamp: event.timestamp.to_string(),
            omap,
            vmap,
        });
    }

    OcelProjection {
        ocel_version: "2.0".to_string(),
        object_types: object_types.into_iter().collect(),
        events: ocel_events,
        objects: objects.into_values().collect(),
    }
}

/// Export OCEL projection to JSON string.
pub fn export_ocel_to_json(events: &[AuditEvent]) -> Result<String, serde_json::Error> {
    let ocel = project_to_ocel(events);
    serde_json::to_string_pretty(&ocel)
}
```

- [ ] **Step 3: Update export/mod.rs**

```rust
pub mod flat_log;
pub mod ocel;
pub use flat_log::*;
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/export/ocel.rs crates/datasynth-audit-fsm/src/export/mod.rs
git commit -m "feat(audit-fsm): add OCEL 2.0 projection exporter"
```

---

### Task 8: Final Validation and Cleanup

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/lib.rs`

- [ ] **Step 1: Run fmt**

```bash
cargo fmt -p datasynth-audit-fsm
```

- [ ] **Step 2: Run clippy**

```bash
cargo clippy -p datasynth-audit-fsm --all-targets
```

Fix any warnings.

- [ ] **Step 3: Run full test suite**

```bash
cargo test -p datasynth-audit-fsm -- --test-threads=4
```

All tests must pass.

- [ ] **Step 4: Verify workspace compiles**

```bash
cargo check -p datasynth-core -p datasynth-config -p datasynth-audit-fsm
```

- [ ] **Step 5: Commit if needed**

```bash
git add -A crates/datasynth-audit-fsm/
git commit -m "feat(audit-fsm): finalize Phase 2 — IA blueprint with C2CE, self-loops, discriminators, OCEL"
```

---

## Summary

| Task | What it delivers | Key files |
|------|-----------------|-----------|
| 1 | Schema additions (order, discriminators, self-loop config) | `schema.rs`, `loader.rs` |
| 2 | IA blueprint loading + validation | `blueprints/generic_ia.yaml`, `loader.rs` |
| 3 | Continuous phase support | `engine.rs` |
| 4 | Self-loop detection and bounding | `engine.rs` |
| 5 | Discriminator filtering | `engine.rs` |
| 6 | IA integration tests | `tests/ia_integration.rs` |
| 7 | OCEL 2.0 projection exporter | `export/ocel.rs` |
| 8 | Final cleanup | All files |

After completion: the engine loads both FSA (7 procedures) and IA (34 procedures) blueprints, handles 8-state C2CE lifecycles, self-loop states, continuous phases, discriminator filtering, and exports to both flat JSON and OCEL 2.0.
