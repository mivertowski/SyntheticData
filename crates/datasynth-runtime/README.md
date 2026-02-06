# datasynth-runtime

Runtime orchestration, parallel execution, and memory management.

## Overview

`datasynth-runtime` provides the execution layer for SyntheticData:

- **GenerationOrchestrator**: Coordinates the complete generation workflow
- **Parallel Execution**: Multi-threaded generation with Rayon
- **Memory Management**: Integration with memory guard for OOM prevention
- **Progress Tracking**: Real-time progress reporting with pause/resume

## Key Components

| Component | Description |
|-----------|-------------|
| `GenerationOrchestrator` | Main workflow coordinator |
| `EnhancedOrchestrator` | Extended orchestrator with all enterprise features |
| `ParallelExecutor` | Thread pool management |
| `ProgressTracker` | Progress bars and status reporting |

## Generation Workflow

The `generate()` method orchestrates 12 focused phase methods in sequence:

1. **Chart of Accounts**: CoA generation with industry-specific structures
2. **Master Data**: Vendors, customers, materials, fixed assets, employees
3. **Document Flows**: P2P and O2C document chain generation
4. **OCPM Events**: OCEL 2.0 event log generation
5. **Journal Entries**: JEs from document flows and standalone transactions
6. **Anomaly Injection**: Entity-aware anomaly injection with risk-adjusted rates
7. **Balance Validation**: Balance sheet equation verification
8. **Data Quality**: Typos, missing values, format variations injection
9. **Audit Data**: Engagements, workpapers, evidence, findings, judgments
10. **Banking Data**: KYC/AML banking transaction generation
11. **Graph Export**: PyTorch Geometric, Neo4j, DGL, RustGraph export
12. **Hypergraph Export**: Multi-layer hypergraph for RustGraph

## Usage

```rust
use datasynth_runtime::GenerationOrchestrator;

let orchestrator = GenerationOrchestrator::new(config)?;

// Full generation
orchestrator.run()?;

// With progress callback
orchestrator.run_with_progress(|progress| {
    println!("Generated: {}/{}", progress.completed, progress.total);
})?;
```

## Pause/Resume

On Unix systems, send `SIGUSR1` to toggle pause:

```bash
kill -USR1 $(pgrep datasynth-data)
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
