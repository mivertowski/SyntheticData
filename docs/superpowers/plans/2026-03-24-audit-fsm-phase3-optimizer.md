# Audit FSM Phase 3: Optimizer Crate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `datasynth-audit-optimizer` crate providing graph analysis of audit FSM blueprints — shortest path to completion, constraint-based optimization, and Monte Carlo simulation for outcome distribution analysis.

**Architecture:** Convert audit blueprints into petgraph directed graphs where nodes are `(procedure_id, state)` pairs and edges are transitions. Run graph algorithms (Dijkstra, constrained search) and stochastic simulation (Monte Carlo with the FSM engine) to find optimal audit paths and analyze outcome distributions.

**Tech Stack:** petgraph for graph algorithms, datasynth-audit-fsm for blueprint types and engine, rand/rand_chacha for Monte Carlo.

**Spec:** `docs/superpowers/specs/2026-03-24-audit-fsm-integration-design.md` (Optimizer Crate section)

---

### Task 1: Create Crate Skeleton

**Files:**
- Create: `crates/datasynth-audit-optimizer/Cargo.toml`
- Create: `crates/datasynth-audit-optimizer/src/lib.rs`
- Modify: `Cargo.toml` (workspace members + add petgraph to workspace deps)

- [ ] **Step 1: Add petgraph to workspace dependencies**

In root `Cargo.toml` `[workspace.dependencies]` section, add:
```toml
petgraph = "0.7"
```

- [ ] **Step 2: Create crate directory and Cargo.toml**

```bash
mkdir -p crates/datasynth-audit-optimizer/src
```

Create `crates/datasynth-audit-optimizer/Cargo.toml` following the workspace pattern (check `crates/datasynth-audit-fsm/Cargo.toml` for the exact format):
```toml
[package]
name = "datasynth-audit-optimizer"
# Use version.workspace = true if that's the pattern, or explicit "1.4.0"
edition = "2021"

[dependencies]
datasynth-audit-fsm = { path = "../datasynth-audit-fsm" }
petgraph = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
rand = { workspace = true }
rand_chacha = { workspace = true }
```

- [ ] **Step 3: Write lib.rs**

```rust
//! Audit FSM optimizer — graph analysis and Monte Carlo simulation.
//!
//! Converts audit methodology blueprints into directed graphs for
//! shortest-path analysis, constraint-based optimization, and
//! stochastic outcome simulation.

pub mod graph;
pub mod shortest_path;
pub mod constrained;
pub mod monte_carlo;
pub mod report;
```

- [ ] **Step 4: Create stub modules**

Create stubs for `graph.rs`, `shortest_path.rs`, `constrained.rs`, `monte_carlo.rs`, `report.rs`.

- [ ] **Step 5: Add to workspace**

Add `"crates/datasynth-audit-optimizer"` to workspace members in root `Cargo.toml`.

- [ ] **Step 6: Verify**

```bash
cargo check -p datasynth-audit-optimizer
```

- [ ] **Step 7: Commit**

```bash
git add crates/datasynth-audit-optimizer/ Cargo.toml Cargo.lock
git commit -m "feat(audit-optimizer): create datasynth-audit-optimizer crate skeleton"
```

---

### Task 2: Blueprint to Graph Conversion

**Files:**
- Modify: `crates/datasynth-audit-optimizer/src/graph.rs`

Convert an audit blueprint into a petgraph DiGraph. Each node is a `(procedure_id, state)` pair. Each edge represents a transition with optional timing weight.

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    #[test]
    fn test_fsa_graph_has_nodes_and_edges() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let graph = blueprint_to_graph(&bwp.blueprint);
        assert!(graph.node_count() > 0, "Graph should have nodes");
        assert!(graph.edge_count() > 0, "Graph should have edges");
    }

    #[test]
    fn test_fsa_graph_has_initial_and_terminal_nodes() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let graph = blueprint_to_graph(&bwp.blueprint);
        let initial = find_initial_nodes(&graph);
        let terminal = find_terminal_nodes(&graph);
        assert!(!initial.is_empty(), "Should have initial nodes");
        assert!(!terminal.is_empty(), "Should have terminal nodes");
    }

    #[test]
    fn test_ia_graph_larger_than_fsa() {
        let fsa = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let ia = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let fsa_graph = blueprint_to_graph(&fsa.blueprint);
        let ia_graph = blueprint_to_graph(&ia.blueprint);
        assert!(ia_graph.node_count() > fsa_graph.node_count());
        assert!(ia_graph.edge_count() > fsa_graph.edge_count());
    }
}
```

- [ ] **Step 2: Implement graph conversion**

```rust
//! Convert audit blueprints to petgraph directed graphs.

use datasynth_audit_fsm::schema::AuditBlueprint;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// A node in the audit graph: (procedure_id, state_name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateNode {
    pub procedure_id: String,
    pub state: String,
}

/// An edge in the audit graph representing a transition.
#[derive(Debug, Clone)]
pub struct TransitionEdge {
    pub command: Option<String>,
    pub emits: Option<String>,
    pub guards: Vec<String>,
}

/// Convert a blueprint into a directed graph.
pub fn blueprint_to_graph(blueprint: &AuditBlueprint) -> DiGraph<StateNode, TransitionEdge> {
    let mut graph = DiGraph::new();
    let mut node_map: HashMap<(String, String), NodeIndex> = HashMap::new();

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            let agg = &proc.aggregate;
            if agg.states.is_empty() {
                continue;
            }

            // Create nodes for each state
            for state in &agg.states {
                let node = StateNode {
                    procedure_id: proc.id.clone(),
                    state: state.clone(),
                };
                let idx = graph.add_node(node);
                node_map.insert((proc.id.clone(), state.clone()), idx);
            }

            // Create edges for each transition
            for trans in &agg.transitions {
                let from_key = (proc.id.clone(), trans.from_state.clone());
                let to_key = (proc.id.clone(), trans.to_state.clone());
                if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(&from_key), node_map.get(&to_key)) {
                    graph.add_edge(from_idx, to_idx, TransitionEdge {
                        command: trans.command.clone(),
                        emits: trans.emits.clone(),
                        guards: trans.guards.clone(),
                    });
                }
            }
        }
    }

    graph
}

/// Find all initial state nodes (nodes whose state matches initial_state).
pub fn find_initial_nodes(graph: &DiGraph<StateNode, TransitionEdge>) -> Vec<NodeIndex> {
    // Initial nodes have no incoming edges within their procedure
    graph.node_indices()
        .filter(|&idx| {
            graph.neighbors_directed(idx, petgraph::Direction::Incoming).count() == 0
        })
        .collect()
}

/// Find all terminal state nodes (nodes with no outgoing edges).
pub fn find_terminal_nodes(graph: &DiGraph<StateNode, TransitionEdge>) -> Vec<NodeIndex> {
    graph.node_indices()
        .filter(|&idx| {
            graph.neighbors_directed(idx, petgraph::Direction::Outgoing).count() == 0
        })
        .collect()
}
```

- [ ] **Step 3: Run tests, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=4
git add crates/datasynth-audit-optimizer/src/graph.rs
git commit -m "feat(audit-optimizer): add blueprint to petgraph conversion"
```

---

### Task 3: Shortest Path Analysis

**Files:**
- Modify: `crates/datasynth-audit-optimizer/src/shortest_path.rs`

Find the minimum number of transitions from initial to terminal states for each procedure.

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    #[test]
    fn test_fsa_shortest_paths() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let result = analyze_shortest_paths(&bwp.blueprint);
        assert!(!result.procedure_paths.is_empty());
        // Each FSA procedure should have a path (3-4 transitions typically)
        for (proc_id, path) in &result.procedure_paths {
            assert!(!path.states.is_empty(), "Procedure {} should have a path", proc_id);
            assert!(path.transition_count >= 2, "Procedure {} should need >= 2 transitions", proc_id);
        }
    }

    #[test]
    fn test_shortest_path_report_serializes() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let result = analyze_shortest_paths(&bwp.blueprint);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("procedure_paths"));
    }
}
```

- [ ] **Step 2: Implement**

Use BFS (unweighted shortest path) per procedure since we want minimum transition count.

```rust
//! Shortest path analysis for audit procedures.

use crate::graph::{blueprint_to_graph, StateNode, TransitionEdge};
use datasynth_audit_fsm::schema::AuditBlueprint;
use petgraph::graph::DiGraph;
use petgraph::visit::Bfs;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ShortestPathReport {
    pub procedure_paths: HashMap<String, ProcedurePath>,
    pub total_minimum_transitions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcedurePath {
    pub states: Vec<String>,
    pub transition_count: usize,
    pub commands: Vec<String>,
}

pub fn analyze_shortest_paths(blueprint: &AuditBlueprint) -> ShortestPathReport {
    let mut procedure_paths = HashMap::new();
    let mut total = 0;

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            let agg = &proc.aggregate;
            if agg.states.is_empty() || agg.initial_state.is_empty() {
                continue;
            }

            // BFS from initial state to find shortest path to terminal states
            if let Some(path) = find_shortest_path_bfs(agg) {
                total += path.transition_count;
                procedure_paths.insert(proc.id.clone(), path);
            }
        }
    }

    ShortestPathReport {
        procedure_paths,
        total_minimum_transitions: total,
    }
}
```

Implement `find_shortest_path_bfs` using a simple BFS on the procedure's aggregate states/transitions to find the shortest sequence of transitions from `initial_state` to a state with no outgoing transitions (terminal).

- [ ] **Step 3: Run tests, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=4
git add crates/datasynth-audit-optimizer/src/shortest_path.rs
git commit -m "feat(audit-optimizer): add shortest path analysis"
```

---

### Task 4: Monte Carlo Simulation

**Files:**
- Modify: `crates/datasynth-audit-optimizer/src/monte_carlo.rs`

Run N stochastic walks through the FSM engine and analyze outcome distributions.

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    #[test]
    fn test_monte_carlo_fsa() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let report = run_monte_carlo(&bwp, 10, 42);
        assert_eq!(report.iterations, 10);
        assert!(report.avg_events > 0.0);
        assert!(report.avg_duration_hours > 0.0);
        assert!(!report.happy_path.is_empty());
    }

    #[test]
    fn test_monte_carlo_deterministic() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let r1 = run_monte_carlo(&bwp, 5, 99);
        let r2 = run_monte_carlo(&bwp, 5, 99);
        assert_eq!(r1.avg_events, r2.avg_events);
        assert_eq!(r1.avg_duration_hours, r2.avg_duration_hours);
    }

    #[test]
    fn test_monte_carlo_report_serializes() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let report = run_monte_carlo(&bwp, 5, 42);
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("iterations"));
        assert!(json.contains("happy_path"));
    }
}
```

- [ ] **Step 2: Implement**

```rust
//! Monte Carlo simulation over the FSM engine.

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloReport {
    pub iterations: usize,
    pub avg_events: f64,
    pub avg_duration_hours: f64,
    pub avg_procedures_completed: f64,
    pub bottleneck_procedures: Vec<(String, f64)>,  // proc_id, avg time
    pub revision_hotspots: Vec<(String, f64)>,       // proc_id, avg revision count
    pub happy_path: Vec<String>,                      // most common procedure completion order
}

pub fn run_monte_carlo(
    bwp: &BlueprintWithPreconditions,
    iterations: usize,
    seed: u64,
) -> MonteCarloReport {
    let ctx = EngagementContext::test_default();
    let mut total_events = 0usize;
    let mut total_duration = 0.0f64;
    let mut total_procs = 0usize;
    let mut procedure_event_counts: HashMap<String, Vec<usize>> = HashMap::new();
    let mut revision_counts: HashMap<String, Vec<usize>> = HashMap::new();
    let mut completion_orders: Vec<Vec<String>> = Vec::new();

    for i in 0..iterations {
        let rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(i as u64));
        let mut engine = AuditFsmEngine::new(bwp.clone(), default_overlay(), rng);
        let result = engine.run_engagement(&ctx).unwrap();

        total_events += result.event_log.len();
        total_duration += result.total_duration_hours;
        total_procs += result.procedure_states.len();

        // Count events per procedure
        let mut proc_counts: HashMap<String, usize> = HashMap::new();
        let mut rev_counts: HashMap<String, usize> = HashMap::new();
        for event in &result.event_log {
            *proc_counts.entry(event.procedure_id.clone()).or_default() += 1;
            // Count revisions (under_review → in_progress)
            if event.from_state.as_deref() == Some("under_review")
                && event.to_state.as_deref() == Some("in_progress")
            {
                *rev_counts.entry(event.procedure_id.clone()).or_default() += 1;
            }
        }
        for (pid, count) in proc_counts {
            procedure_event_counts.entry(pid).or_default().push(count);
        }
        for (pid, count) in rev_counts {
            revision_counts.entry(pid).or_default().push(count);
        }

        // Track completion order by first-event timestamp
        let mut first_events: Vec<(String, _)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for event in &result.event_log {
            if seen.insert(event.procedure_id.clone()) {
                first_events.push((event.procedure_id.clone(), event.timestamp));
            }
        }
        first_events.sort_by_key(|(_, ts)| *ts);
        completion_orders.push(first_events.into_iter().map(|(id, _)| id).collect());
    }

    let n = iterations as f64;

    // Bottlenecks: procedures with highest avg event count
    let mut bottlenecks: Vec<(String, f64)> = procedure_event_counts.iter()
        .map(|(pid, counts)| (pid.clone(), counts.iter().sum::<usize>() as f64 / n))
        .collect();
    bottlenecks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    bottlenecks.truncate(5);

    // Revision hotspots
    let mut hotspots: Vec<(String, f64)> = revision_counts.iter()
        .map(|(pid, counts)| (pid.clone(), counts.iter().sum::<usize>() as f64 / n))
        .collect();
    hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    hotspots.truncate(5);

    // Happy path: most common first completion order
    let happy_path = if let Some(first) = completion_orders.first() {
        first.clone()
    } else {
        Vec::new()
    };

    MonteCarloReport {
        iterations,
        avg_events: total_events as f64 / n,
        avg_duration_hours: total_duration / n,
        avg_procedures_completed: total_procs as f64 / n,
        bottleneck_procedures: bottlenecks,
        revision_hotspots: hotspots,
        happy_path,
    }
}
```

- [ ] **Step 3: Run tests, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=4
git add crates/datasynth-audit-optimizer/src/monte_carlo.rs
git commit -m "feat(audit-optimizer): add Monte Carlo simulation"
```

---

### Task 5: Constrained Path and Report

**Files:**
- Modify: `crates/datasynth-audit-optimizer/src/constrained.rs`
- Modify: `crates/datasynth-audit-optimizer/src/report.rs`

- [ ] **Step 1: Implement constrained path**

Simple constraint-based analysis: given a set of must-visit procedures, find which additional procedures are required (via preconditions) and compute the minimum path.

```rust
//! Constraint-based path optimization.

use datasynth_audit_fsm::schema::AuditBlueprint;
use crate::shortest_path::{analyze_shortest_paths, ShortestPathReport};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct ConstrainedPathResult {
    pub required_procedures: Vec<String>,
    pub total_transitions: usize,
    pub paths: ShortestPathReport,
}

/// Given must-visit procedures, determine the full set needed (including
/// precondition dependencies) and compute shortest paths.
pub fn constrained_path(
    blueprint: &AuditBlueprint,
    must_visit: &[String],
    preconditions: &std::collections::HashMap<String, Vec<String>>,
) -> ConstrainedPathResult {
    // Expand must-visit with transitive preconditions
    let mut required: HashSet<String> = must_visit.iter().cloned().collect();
    let mut queue: Vec<String> = must_visit.to_vec();

    while let Some(proc_id) = queue.pop() {
        if let Some(deps) = preconditions.get(&proc_id) {
            for dep in deps {
                if required.insert(dep.clone()) {
                    queue.push(dep.clone());
                }
            }
        }
    }

    let full_paths = analyze_shortest_paths(blueprint);
    let filtered_paths: std::collections::HashMap<_, _> = full_paths.procedure_paths.iter()
        .filter(|(k, _)| required.contains(k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let total = filtered_paths.values().map(|p| p.transition_count).sum();

    ConstrainedPathResult {
        required_procedures: required.into_iter().collect(),
        total_transitions: total,
        paths: ShortestPathReport {
            procedure_paths: filtered_paths,
            total_minimum_transitions: total,
        },
    }
}
```

- [ ] **Step 2: Implement report module**

```rust
//! Report formatting for optimizer outputs.

use crate::monte_carlo::MonteCarloReport;
use crate::shortest_path::ShortestPathReport;

/// Format a shortest path report as a human-readable string.
pub fn format_shortest_path_report(report: &ShortestPathReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Total minimum transitions: {}\n\n", report.total_minimum_transitions));
    for (proc_id, path) in &report.procedure_paths {
        out.push_str(&format!("  {} ({} transitions): {}\n",
            proc_id, path.transition_count, path.states.join(" → ")));
    }
    out
}

/// Format a Monte Carlo report as a human-readable string.
pub fn format_monte_carlo_report(report: &MonteCarloReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Monte Carlo Simulation ({} iterations)\n", report.iterations));
    out.push_str(&format!("  Avg events: {:.1}\n", report.avg_events));
    out.push_str(&format!("  Avg duration: {:.1} hours\n", report.avg_duration_hours));
    out.push_str(&format!("  Avg procedures: {:.1}\n\n", report.avg_procedures_completed));

    if !report.bottleneck_procedures.is_empty() {
        out.push_str("  Bottleneck procedures:\n");
        for (proc_id, avg) in &report.bottleneck_procedures {
            out.push_str(&format!("    {} — {:.1} avg events\n", proc_id, avg));
        }
    }

    if !report.revision_hotspots.is_empty() {
        out.push_str("\n  Revision hotspots:\n");
        for (proc_id, avg) in &report.revision_hotspots {
            out.push_str(&format!("    {} — {:.1} avg revisions\n", proc_id, avg));
        }
    }

    out.push_str(&format!("\n  Happy path: {}\n", report.happy_path.join(" → ")));
    out
}
```

- [ ] **Step 3: Add tests for both**

Tests for constrained path and report formatting.

- [ ] **Step 4: Run tests, verify, commit**

```bash
cargo test -p datasynth-audit-optimizer -- --test-threads=4
git add crates/datasynth-audit-optimizer/src/constrained.rs crates/datasynth-audit-optimizer/src/report.rs
git commit -m "feat(audit-optimizer): add constrained path analysis and report formatting"
```

---

### Task 6: Final Validation and Cleanup

- [ ] **Step 1: Run fmt, clippy, tests**

```bash
cargo fmt -p datasynth-audit-optimizer
cargo clippy -p datasynth-audit-optimizer --all-targets
cargo test -p datasynth-audit-optimizer -- --test-threads=4
cargo check -p datasynth-audit-fsm -p datasynth-audit-optimizer
```

- [ ] **Step 2: Commit if needed**

```bash
git commit -m "feat(audit-optimizer): finalize Phase 3 — graph analysis, shortest path, Monte Carlo"
```

---

## Summary

| Task | What it delivers |
|------|-----------------|
| 1 | Crate skeleton with petgraph dependency |
| 2 | Blueprint → petgraph DiGraph conversion |
| 3 | Shortest path (BFS) per procedure |
| 4 | Monte Carlo simulation with outcome analysis |
| 5 | Constrained path + report formatting |
| 6 | Final cleanup |
