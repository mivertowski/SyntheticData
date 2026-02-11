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

The orchestrator executes phases in order:

1. **Initialize**: Load configuration, validate settings
2. **Master Data**: Generate vendors, customers, materials, assets
3. **Opening Balances**: Create coherent opening balance sheet
4. **Transactions**: Generate journal entries with document flows
5. **Period Close**: Run monthly/quarterly/annual close processes
6. **Anomalies**: Inject configured anomalies and data quality issues
7. **Export**: Write outputs and generate ML labels
8. **Banking**: Generate KYC/AML data (if enabled)
9. **Audit**: Generate ISA-compliant audit data (if enabled)
10. **Graphs**: Build and export ML graphs (if enabled)
11. **LLM Enrichment**: Enrich data with LLM-generated metadata (v0.5.0, if enabled)
12. **Diffusion Enhancement**: Blend diffusion model outputs (v0.5.0, if enabled)
13. **Causal Overlay**: Apply causal structure (v0.5.0, if enabled)

## Key Types

### GenerationOrchestrator

```rust
pub struct GenerationOrchestrator {
    config: Config,
    state: GenerationState,
    progress: Arc<ProgressTracker>,
    memory_guard: MemoryGuard,
}

pub struct GenerationState {
    pub master_data: MasterDataState,
    pub entries: Vec<JournalEntry>,
    pub documents: DocumentState,
    pub balances: BalanceState,
    pub anomaly_labels: Vec<LabeledAnomaly>,
}
```

### ProgressTracker

```rust
pub struct ProgressTracker {
    pub current: AtomicU64,
    pub total: u64,
    pub phase: String,
    pub paused: AtomicBool,
    pub start_time: Instant,
}

pub struct Progress {
    pub current: u64,
    pub total: u64,
    pub percent: f64,
    pub phase: String,
    pub entries_per_second: f64,
    pub elapsed: Duration,
    pub estimated_remaining: Duration,
}
```

## Usage Examples

### Basic Generation

```rust
use synth_runtime::GenerationOrchestrator;

let config = Config::from_yaml_file("config.yaml")?;
let orchestrator = GenerationOrchestrator::new(config)?;

// Run full generation
orchestrator.run()?;
```

### With Progress Callback

```rust
orchestrator.run_with_progress(|progress| {
    println!(
        "[{:.1}%] {} - {}/{} ({:.0} entries/sec)",
        progress.percent,
        progress.phase,
        progress.current,
        progress.total,
        progress.entries_per_second,
    );
})?;
```

### Parallel Execution

```rust
use synth_runtime::ParallelExecutor;

let executor = ParallelExecutor::new(4); // 4 threads

let results: Vec<JournalEntry> = executor.run(|thread_id| {
    let mut generator = JournalEntryGenerator::new(config.clone(), seed + thread_id);
    generator.generate_batch(batch_size)
})?;
```

### Memory-Aware Generation

```rust
use synth_runtime::GenerationOrchestrator;
use synth_core::memory_guard::MemoryGuardConfig;

let memory_config = MemoryGuardConfig {
    soft_limit: 1024 * 1024 * 1024,  // 1GB
    hard_limit: 2 * 1024 * 1024 * 1024,  // 2GB
    check_interval_ms: 1000,
    ..Default::default()
};

let orchestrator = GenerationOrchestrator::with_memory_config(config, memory_config)?;
orchestrator.run()?;
```

## Pause/Resume

On Unix systems, generation can be paused and resumed:

```bash
# Start generation in background
datasynth-data generate --config config.yaml --output ./output &

# Send SIGUSR1 to toggle pause
kill -USR1 $(pgrep datasynth-data)

# Progress bar shows pause state
# [████████░░░░░░░░░░░░] 40% (PAUSED)
```

### Programmatic Pause/Resume

```rust
// Pause
orchestrator.pause();

// Check state
if orchestrator.is_paused() {
    println!("Generation paused");
}

// Resume
orchestrator.resume();
```

## Enhanced Orchestrator

The `EnhancedOrchestrator` includes additional enterprise features:

```rust
use synth_runtime::EnhancedOrchestrator;

let orchestrator = EnhancedOrchestrator::new(config)?;

// All features enabled
orchestrator
    .with_document_flows()
    .with_intercompany()
    .with_subledgers()
    .with_fx()
    .with_period_close()
    .with_anomaly_injection()
    .with_graph_export()
    .run()?;
```

## Output Coordination

The orchestrator coordinates output to multiple sinks:

```rust
// Orchestrator automatically:
// 1. Creates output directories
// 2. Writes master data files
// 3. Writes transaction files
// 4. Writes subledger files
// 5. Writes labels for ML
// 6. Generates graphs if enabled
```

## Error Handling

```rust
pub enum RuntimeError {
    ConfigurationError(ConfigError),
    GenerationError(String),
    MemoryExceeded { limit: u64, current: u64 },
    OutputError(OutputError),
    Interrupted,
}
```

## Performance Considerations

### Thread Count

```rust
// Auto-detect (uses all cores)
let orchestrator = GenerationOrchestrator::new(config)?;

// Manual thread count
let orchestrator = GenerationOrchestrator::with_threads(config, 4)?;
```

### Memory Management

The orchestrator monitors memory and can:
- Slow down generation when soft limit approached
- Pause generation at hard limit
- Stream output to reduce memory pressure

### Batch Sizes

Batch sizes are automatically tuned based on:
- Available memory
- Number of threads
- Target throughput

## See Also

- [Performance Tuning](../advanced/performance.md)
- [Memory Management](../architecture/memory-management.md)
- [datasynth-generators](datasynth-generators.md)
