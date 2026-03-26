# datasynth-audit-optimizer

Graph analysis and Monte Carlo simulation for audit FSM blueprints.

## Overview

`datasynth-audit-optimizer` converts audit methodology blueprints into petgraph directed graphs and provides analysis, simulation, and planning capabilities across 16 modules: graph conversion, shortest-path analysis (BFS per procedure), constrained-path optimization (must-visit procedures with transitive precondition expansion), Monte Carlo simulation (N stochastic walks for bottleneck detection, revision hotspots, and happy path identification), year-over-year engagement chains, ISA 600 group audit simulation, blueprint testing, benchmark comparison, overlay fitting, anomaly calibration, resource optimization, risk-based scoping, portfolio simulation, conformance checking, and process discovery.

The crate operates on `AuditBlueprint` types from `datasynth-audit-fsm` and produces serializable report structures suitable for JSON export. Functions that execute the FSM engine (Monte Carlo, calibration, overlay fitting, benchmark comparison) accept an `&EngagementContext` parameter; pure graph-analysis functions (shortest path, constrained path) do not.

## Graph Conversion

`blueprint_to_graph()` transforms an `AuditBlueprint` into a `DiGraph<StateNode, TransitionEdge>`. Each node represents a `(procedure_id, state)` pair; each edge represents a transition within a procedure's FSM aggregate.

```rust
pub struct StateNode {
    pub procedure_id: String,
    pub state: String,
}

pub struct TransitionEdge {
    pub command: Option<String>,
    pub emits: Option<String>,
    pub guards: Vec<String>,
}
```

Helper functions `find_initial_nodes()` and `find_terminal_nodes()` identify entry points (no incoming edges) and completion states (no outgoing edges).

## Shortest Path Analysis

`analyze_shortest_paths()` runs BFS on each procedure's FSM aggregate, finding the minimum number of transitions from `initial_state` to any terminal state. BFS guarantees the first path found is the shortest.

Results are collected into a `ShortestPathReport`:

```rust
pub struct ShortestPathReport {
    pub procedure_paths: HashMap<String, ProcedurePath>,
    pub total_minimum_transitions: usize,
}

pub struct ProcedurePath {
    pub states: Vec<String>,
    pub transition_count: usize,
    pub commands: Vec<String>,
}
```

For the FSA blueprint, the total minimum transitions across all 9 procedures is 27. For the IA blueprint with 34 procedures, it is 101.

## Constrained Path Optimization

`constrained_path()` answers the question: "What is the minimum work needed to complete a specific set of procedures?" Given a list of must-visit procedures, it expands the required set by transitively following preconditions, then returns filtered shortest paths for only those procedures.

```rust
pub struct ConstrainedPathResult {
    pub required_procedures: Vec<String>,
    pub total_transitions: usize,
    pub paths: ShortestPathReport,
}
```

For example, constraining to `["form_opinion"]` in the FSA blueprint expands to include `going_concern`, `subsequent_events`, and their transitive preconditions.

## Monte Carlo Simulation

`run_monte_carlo()` executes N stochastic walks through the `AuditFsmEngine`, each with a deterministically-derived seed (`seed.wrapping_add(i)`). It accepts an `&EngagementContext` to provide financial data and team information to the engine, and returns `Result<MonteCarloReport, String>`. The simulation collects:

- **Bottleneck procedures** -- top 5 by average event count
- **Revision hotspots** -- top 5 by average `under_review -> in_progress` transition count
- **Happy path** -- procedure completion order from the first iteration
- **Aggregate statistics** -- average events, duration, and completed procedure count

```rust
pub struct MonteCarloReport {
    pub iterations: usize,
    pub avg_events: f64,
    pub avg_duration_hours: f64,
    pub avg_procedures_completed: f64,
    pub bottleneck_procedures: Vec<(String, f64)>,
    pub revision_hotspots: Vec<(String, f64)>,
    pub happy_path: Vec<String>,
}
```

Both `ShortestPathReport` and `MonteCarloReport` have human-readable formatters in the `report` module (`format_shortest_path_report()` and `format_monte_carlo_report()`).

## Usage

```rust
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::loader::BlueprintWithPreconditions;
use datasynth_audit_optimizer::{
    graph::blueprint_to_graph,
    shortest_path::analyze_shortest_paths,
    constrained::constrained_path,
    monte_carlo::run_monte_carlo,
    report::{format_shortest_path_report, format_monte_carlo_report},
};

let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
let ctx = EngagementContext::demo();

// Shortest paths (pure graph analysis — no EngagementContext needed)
let sp = analyze_shortest_paths(&bwp.blueprint);
println!("{}", format_shortest_path_report(&sp));

// Constrained paths
let must_visit = vec!["form_opinion".to_string()];
let cp = constrained_path(&bwp.blueprint, &must_visit, &bwp.preconditions);

// Monte Carlo (100 iterations — requires &EngagementContext, returns Result)
let mc = run_monte_carlo(&bwp, 100, 42, &ctx).unwrap();
println!("{}", format_monte_carlo_report(&mc));
```

## Key Types

| Type | Module | Description |
|------|--------|-------------|
| `StateNode` | `graph` | Graph node: `(procedure_id, state)` pair |
| `TransitionEdge` | `graph` | Graph edge: command, emits, guards |
| `ShortestPathReport` | `shortest_path` | BFS results across all procedures |
| `ProcedurePath` | `shortest_path` | Single procedure's minimum-transition path |
| `ConstrainedPathResult` | `constrained` | Must-visit expansion + filtered paths |
| `MonteCarloReport` | `monte_carlo` | N-iteration simulation statistics |
| `YoyChainConfig` / `YoyChainReport` | `yoy_chain` | Year-over-year engagement chains with finding carry-forward |
| `GroupAuditConfig` / `GroupAuditReport` | `group_audit` | ISA 600 group audit with component-level FSM execution |
| `BlueprintTestSuite` / `BlueprintTestResult` | `blueprint_testing` | Automated blueprint validation against expected metrics |
| `ComparisonReport` | `benchmark_comparison` | Side-by-side blueprint comparison statistics |
| `PortfolioConfig` / `PortfolioReport` | `portfolio` | Multi-engagement portfolio simulation |
| `ConformanceReport` | `conformance` | Fitness, precision, and generalization metrics |
| `DiscoveredBlueprint` | `discovery` | Blueprint inferred from event logs |

## See Also

- [datasynth-audit-fsm](datasynth-audit-fsm.md)
- [Audit FSM Engine Deep Dive](../advanced/audit-fsm-engine.md)
- [Audit Analytics](../use-cases/audit-analytics.md)
