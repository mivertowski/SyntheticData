# Wave 3: Process Mining Benchmarks (v1.8.0) Design Spec

**Date**: 2026-03-24
**Status**: Approved
**Scope**: Benchmark audit event log generation with multiple export formats and conformance metrics.

## Problem

The audit FSM engine produces event trails and OCEL projections, but:
- No bridge to the existing XES exporter in `datasynth-ocpm`
- No flat CSV export for commercial tools (Disco, Celonis, Minit)
- No benchmark dataset generator with configurable complexity/anomaly rates
- No conformance metrics (fitness, precision) comparing generated logs to blueprint FSMs

## Solution

Four work areas, all within existing crates (no new crates).

---

## 1. Audit Event → Process Mining Format Bridge

Bridge `Vec<AuditEvent>` from the FSM engine to the existing `datasynth-ocpm` export formats.

### XES Export

Convert `AuditEvent` → `OcpmEvent` objects, then use the existing `XesExporter`. Each engagement becomes a case, procedures become activities.

```rust
// In datasynth-audit-fsm/src/export/xes.rs
pub fn export_events_to_xes(events: &[AuditEvent], path: &Path) -> std::io::Result<()>
```

Mapping:
- `case:concept:name` = engagement (single case per engagement run)
- `concept:name` = event.command
- `time:timestamp` = event.timestamp
- `org:resource` = event.actor_id
- `lifecycle:transition` = "complete"
- Custom attributes: `procedure_id`, `phase_id`, `from_state`, `to_state`, `is_anomaly`

### CSV Export (Disco/Celonis/Minit)

Simple flat CSV with columns: `case_id,activity,timestamp,resource,procedure,phase,from_state,to_state,is_anomaly,anomaly_type`

```rust
// In datasynth-audit-fsm/src/export/csv.rs
pub fn export_events_to_csv(events: &[AuditEvent], path: &Path) -> std::io::Result<()>
```

No external dependencies — just write CSV directly.

### Files
- Create: `crates/datasynth-audit-fsm/src/export/xes.rs`
- Create: `crates/datasynth-audit-fsm/src/export/csv.rs`
- Modify: `crates/datasynth-audit-fsm/src/export/mod.rs`

---

## 2. Benchmark Dataset Generator

A CLI action and programmatic API that generates reference audit event logs at three complexity levels with known anomaly labels.

### Complexity Levels

| Level | Blueprint | Overlay | Anomaly Rate | Expected Events | Purpose |
|-------|-----------|---------|-------------|-----------------|---------|
| simple | FSA | default | 0% (anomalies disabled) | ~50 | Baseline conformance testing |
| medium | FSA | rushed | ~10% | ~50 | Moderate anomaly detection |
| complex | IA | default | ~20% | ~200+ | Full complexity benchmark |

### API

```rust
// In datasynth-audit-fsm/src/benchmark.rs
pub struct BenchmarkConfig {
    pub complexity: BenchmarkComplexity,
    pub anomaly_rate: Option<f64>,  // override default for complexity level
    pub seed: u64,
}

pub enum BenchmarkComplexity { Simple, Medium, Complex }

pub struct BenchmarkDataset {
    pub events: Vec<AuditEvent>,
    pub anomaly_labels: Vec<AuditAnomalyRecord>,
    pub metadata: BenchmarkMetadata,
}

pub struct BenchmarkMetadata {
    pub complexity: String,
    pub blueprint: String,
    pub overlay: String,
    pub event_count: usize,
    pub anomaly_count: usize,
    pub anomaly_rate: f64,
    pub procedure_count: usize,
    pub seed: u64,
}

pub fn generate_benchmark(config: &BenchmarkConfig) -> Result<BenchmarkDataset, AuditFsmError>
```

### CLI Integration

```bash
datasynth-data audit benchmark --complexity simple --output ./benchmarks/simple/
datasynth-data audit benchmark --complexity complex --anomaly-rate 0.3 --output ./benchmarks/complex/
```

Output files per benchmark:
- `event_trail.json` — flat event log
- `event_trail.csv` — CSV for Disco/Celonis
- `event_trail_ocel.json` — OCEL 2.0
- `anomaly_labels.json` — ground truth labels
- `metadata.json` — dataset metadata

### Files
- Create: `crates/datasynth-audit-fsm/src/benchmark.rs`
- Modify: `crates/datasynth-audit-fsm/src/lib.rs`
- Modify: `crates/datasynth-cli/src/main.rs` (add Benchmark action to AuditCommands)

---

## 3. Conformance Metrics

Compute conformance scores comparing a generated event log against its source blueprint FSM.

### Fitness

Fraction of events that follow valid transitions in the blueprint FSM.

```
fitness = valid_transitions / total_transitions
```

A transition is valid if `(from_state, to_state)` exists in the procedure's aggregate transitions.

### Precision

How much of the blueprint's allowed behavior was actually observed.

```
precision = observed_transitions / enabled_transitions
```

Where `enabled_transitions` = total unique transitions defined in the blueprint, `observed_transitions` = transitions that actually fired.

### Anomaly Detection Accuracy

Compare injected anomalies (ground truth) against the `is_anomaly` flags in the event log.

```
true_positives = events marked anomaly that ARE anomalies
false_positives = events marked anomaly that are NOT anomalies
false_negatives = actual anomalies not marked
```

(In our system, TP = anomaly count since we inject and label simultaneously. This metric becomes useful when an external detector is compared against our labels.)

### API

```rust
// In datasynth-audit-optimizer/src/conformance.rs
pub struct ConformanceReport {
    pub fitness: f64,
    pub precision: f64,
    pub anomaly_stats: AnomalyStats,
    pub per_procedure: HashMap<String, ProcedureConformance>,
}

pub struct AnomalyStats {
    pub total_events: usize,
    pub anomaly_events: usize,
    pub anomaly_rate: f64,
    pub by_type: HashMap<String, usize>,
}

pub struct ProcedureConformance {
    pub procedure_id: String,
    pub fitness: f64,
    pub transitions_observed: usize,
    pub transitions_valid: usize,
}

pub fn analyze_conformance(
    events: &[AuditEvent],
    blueprint: &AuditBlueprint,
) -> ConformanceReport
```

### Files
- Create: `crates/datasynth-audit-optimizer/src/conformance.rs`
- Modify: `crates/datasynth-audit-optimizer/src/lib.rs`

---

## 4. Testing

- XES export: write + read back, verify event count and attributes
- CSV export: write + verify column headers and row count
- Benchmark: generate all 3 complexity levels, verify metadata
- Conformance: perfect log = fitness 1.0, log with anomalies < 1.0
- E2E: generate benchmark → export all formats → compute conformance
- All tests `--test-threads=1`

---

## Dependencies

- `datasynth-audit-fsm`: new export formats, benchmark generator
- `datasynth-audit-optimizer`: conformance metrics
- `datasynth-cli`: benchmark CLI action
- No new crate dependencies (CSV is hand-written, XES uses string building)
