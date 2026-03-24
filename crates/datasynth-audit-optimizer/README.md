# datasynth-audit-optimizer

Graph analysis and Monte Carlo simulation for audit FSM blueprints.

## Overview

`datasynth-audit-optimizer` converts audit methodology blueprints from `datasynth-audit-fsm` into directed graphs for path analysis and stochastic simulation:

- **Graph conversion**: Blueprint procedures → petgraph `DiGraph` with `(procedure_id, state)` nodes and transition edges
- **Shortest path**: BFS per procedure — FSA: 27 minimum transitions, IA: 101
- **Constrained path**: Must-visit procedures with transitive precondition expansion
- **Monte Carlo**: N stochastic walks with outcome distribution analysis

## Usage

```rust
use datasynth_audit_optimizer::shortest_path::analyze_shortest_paths;
use datasynth_audit_optimizer::monte_carlo::run_monte_carlo;
use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

// Shortest path analysis
let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
let report = analyze_shortest_paths(&bwp.blueprint);
println!("Min transitions: {}", report.total_minimum_transitions); // 27

// Monte Carlo simulation (100 iterations)
let mc = run_monte_carlo(&bwp, 100, 42);
println!("Avg events: {:.1}", mc.avg_events);
println!("Happy path: {}", mc.happy_path.join(" → "));
```

## Monte Carlo Report

```rust
pub struct MonteCarloReport {
    pub iterations: usize,
    pub avg_events: f64,
    pub avg_duration_hours: f64,
    pub avg_procedures_completed: f64,
    pub bottleneck_procedures: Vec<(String, f64)>,   // top 5 by avg event count
    pub revision_hotspots: Vec<(String, f64)>,        // top 5 by avg revision count
    pub happy_path: Vec<String>,                       // most common completion order
}
```

## Modules

| Module | Purpose |
|--------|---------|
| `graph` | Blueprint → petgraph `DiGraph<StateNode, TransitionEdge>` conversion |
| `shortest_path` | BFS shortest path per procedure |
| `constrained` | Must-visit + precondition expansion path optimization |
| `monte_carlo` | Stochastic simulation with outcome distribution analysis |
| `report` | Human-readable report formatting |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
