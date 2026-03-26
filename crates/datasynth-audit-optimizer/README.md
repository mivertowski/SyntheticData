# datasynth-audit-optimizer

Graph analysis, Monte Carlo simulation, and optimization for audit FSM blueprints.

## Overview

`datasynth-audit-optimizer` converts audit methodology blueprints from `datasynth-audit-fsm` into directed graphs for path analysis, stochastic simulation, and engagement optimization:

- **Graph conversion**: Blueprint procedures to petgraph `DiGraph` with `(procedure_id, state)` nodes and transition edges
- **Shortest path**: BFS per procedure -- FSA: 27 minimum transitions, IA: 101
- **Constrained path**: Must-visit procedures with transitive precondition expansion
- **Monte Carlo**: N stochastic walks with outcome distribution analysis
- **Cross-firm benchmark comparison**: Compare methodology coverage and efficiency across firms
- **ISA 600 group audit simulation**: Component auditor assignment, materiality allocation, scope
- **Year-over-year engagement chains**: Multi-period engagement simulation with carry-forward

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
println!("Happy path: {}", mc.happy_path.join(" -> "));
```

## Modules

| Module | Purpose |
|--------|---------|
| `graph` | Blueprint to petgraph `DiGraph<StateNode, TransitionEdge>` conversion |
| `shortest_path` | BFS shortest path per procedure |
| `constrained` | Must-visit + precondition expansion path optimization |
| `monte_carlo` | Stochastic simulation with outcome distribution analysis |
| `report` | Human-readable report formatting |
| `resource_optimizer` | Budget/role-aware audit plan selection with coverage reporting |
| `risk_scoping` | Standards/risk coverage analysis, what-if procedure removal impact |
| `portfolio` | Multi-engagement simulation with shared resources and scheduling |
| `conformance` | Fitness, precision, and anomaly detection statistics |
| `overlay_fitting` | Iterative parameter search from target engagement profiles |
| `discovery` | Blueprint inference from event logs (alpha miner variant) |
| `calibration` | Anomaly injection rate auto-tuning to target detection difficulty |
| `benchmark_comparison` | Cross-firm methodology comparison (coverage, efficiency, cost) |
| `yoy_chain` | Year-over-year engagement chains with carry-forward findings |
| `group_audit` | ISA 600 group audit simulation (components, materiality, scope) |
| `blueprint_testing` | Automated blueprint validation and regression testing |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
