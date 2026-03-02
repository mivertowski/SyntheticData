# Unified Generation Pipeline — Design Document

**Date:** 2026-03-02
**Version:** v0.10.0 target
**Status:** Approved

## 1. Problem Statement

DataSynth currently generates data in a single batch: one invocation, one date range, one output. Five enhancement requests share a common root cause — the generation pipeline lacks statefulness:

1. **Real-time streaming ingestion** — streaming is post-hoc (generate all, then push), not woven into generation phases
2. **Configurable fraud patterns** — fraud config is powerful but buried in YAML; no CLI presets or reusable packs
3. **Multi-period generation** — limited to a single contiguous date range; no multi-year support
4. **Incremental generation** — no way to append new transactions to an existing dataset
5. **OCEL 2.0 enrichment** — events lack lifecycle state machines, multi-object correlations, and resource workload modeling

## 2. Architectural Approach: Unified Generation Pipeline

Replace the current "generate everything in one shot" model with a **stateful GenerationSession** that generates periods incrementally, streams as it goes, and resumes from checkpoints.

The existing `EnhancedOrchestrator` becomes an internal implementation detail — the session calls it per-period with adjusted configs (date range, opening balances, seed advancement).

## 3. GenerationSession Core Abstraction

### 3.1 Session Structure

```
GenerationSession
├── config: GeneratorConfig          // immutable base config
├── state: SessionState              // mutable: seed checkpoint, registries, balances, cursor
├── output_mode: OutputMode          // Batch(path) | Stream(target) | Hybrid(path + target)
├── fraud_pack: Option<FraudPack>    // merged fraud scenario overlay
└── periods: Vec<FiscalPeriod>       // multi-year: [FY2022, FY2023, FY2024]
```

### 3.2 SessionState (serializable to `.dss` file)

```
SessionState
├── rng_state: [u8; 32]             // ChaCha8 state for deterministic resume
├── entity_registry: EntityRegistry  // vendors, customers, employees created so far
├── balance_state: BalanceState      // opening balances for next period
├── coa: ChartOfAccounts            // shared across periods
├── period_cursor: usize            // which period we're on
├── generation_log: Vec<PeriodLog>  // what was generated per period
└── last_document_ids: DocumentIdState // sequence counters for PO/SO/JE numbering
```

### 3.3 Key Behaviors

| Method | Description |
|--------|-------------|
| `Session::new(config)` | Create fresh session, compute fiscal periods |
| `Session::resume(path)` | Load `.dss` checkpoint, pick up where left off |
| `Session::generate_next_period()` | Generate one fiscal period, advance cursor, save state |
| `Session::generate_all()` | Loop through all remaining periods |
| `Session::generate_delta(months)` | Append N months of new transactions to existing output |
| `Session::save(path)` | Serialize state to `.dss` for later resume |

### 3.4 Seed Determinism

Seed advancement across periods is deterministic: `seed_n+1 = hash(seed_n, period_index)`. This ensures that the same config + seed always produces identical output regardless of whether periods are generated in one run or across multiple resume cycles.

## 4. Multi-Period Generation

### 4.1 Config Extension

```yaml
global:
  start_date: "2022-01-01"
  period_months: 36          # 3 years total
  fiscal_year_months: 12     # each FY is 12 months
  # Derived: FY2022, FY2023, FY2024
```

When `period_months > fiscal_year_months`, the session automatically splits into multiple fiscal periods.

### 4.2 Cross-Period Continuity

- **Balances carry forward**: Period N's closing trial balance becomes Period N+1's opening balances. `BalanceState` captures all GL account balances, AR/AP aging, FA net book values.
- **Entity registry persists**: Vendors, customers, employees created in FY2022 exist in FY2023. New entities added per-period based on growth/churn rates.
- **Document numbering continues**: PO-2022-000150 → PO-2023-000151 (no restart at year boundary).
- **Trend drift**: Year-over-year growth rates applied to transaction volumes, amounts, and entity counts. The existing `DriftConfig` and `EconomicCycle` parameters play out over longer horizons.

### 4.3 Year-End Transitions

Each period boundary triggers:
1. Year-end close (existing `year_end` module)
2. Opening balance generation for next period
3. Seed advancement (`seed_n+1 = hash(seed_n, period_index)`)
4. Entity lifecycle events (vendor churn, customer acquisition, employee turnover)

### 4.4 Output Structure

```
output/
├── session.dss                    # checkpoint file
├── FY2022/
│   ├── journal_entries.csv
│   ├── trial_balances/
│   └── ...
├── FY2023/
│   ├── journal_entries.csv
│   └── ...
├── FY2024/
│   └── ...
└── consolidated/                  # merged view across all periods
    ├── journal_entries.csv        # all JEs with fiscal_year column
    └── trend_summary.csv          # YoY metrics
```

## 5. Incremental Generation

### 5.1 Two Modes

**Delta append** (`session.generate_delta(3)`) — generate 3 more months of data, appending to existing output files. Uses saved `SessionState` for continuity.

**Live progression** (`session.generate_delta(1)` in a loop with delay) — for demos, generate one month at a time with pauses, creating the appearance of live data arriving.

### 5.2 CLI

```bash
# Initial generation
datasynth-data generate --config config.yaml --output ./output

# Append 6 more months
datasynth-data generate --config config.yaml --output ./output --append --months 6

# Live progression (1 month every 5 seconds)
datasynth-data generate --config config.yaml --output ./output --live --interval 5
```

### 5.3 Constraints

- Config must match the original session (validated via hash stored in `.dss`)
- Seed determinism preserved: same `--append --months 6` always produces identical output
- Cannot change entity counts retroactively — only new entities going forward

## 6. Streaming Pipeline

### 6.1 Architecture

Streaming becomes an output mode woven into each generation phase, not a post-hoc export.

```
GenerationSession
  └── OutputMode::Stream(StreamPipeline)
        ├── StreamTarget::Http { url, api_key, batch_size }
        ├── StreamTarget::Embedded { port }     # built-in ingest endpoint
        └── StreamTarget::File { path }         # JSONL file sink

StreamPipeline
  ├── phase_sink: Box<dyn PhaseSink>     # receives typed items as generated
  ├── format: StreamFormat               # Unified | OCEL | Raw
  ├── backpressure: BackpressureStrategy # DropOldest | Block | Buffer(max)
  └── stats: StreamStats                 # items/sec, bytes, errors
```

### 6.2 Phase-Aware Streaming

Each generation phase (CoA, Master Data, Document Flows, JEs, OCPM, Anomalies) emits items to the `StreamPipeline` as they're created:

- OCPM events stream as document flows are generated (not deferred to post-processing)
- Anomaly labels stream alongside injected entries
- Hypergraph nodes/edges stream per-entity rather than as bulk export

### 6.3 Deployment Modes

**External target** — stream to any JSONL-accepting HTTP endpoint:
```bash
datasynth-data generate --config config.yaml --stream-target http://localhost:8080/ingest
```

**Embedded mode** — built-in lightweight ingest endpoint (no external dependency):
```bash
datasynth-data generate --config config.yaml --stream-embedded --port 9090
```

**Docker Compose** — pre-wired networking between DataSynth and RustGraph:
```bash
docker compose up  # starts RustGraph + DataSynth streaming into it
```

### 6.4 Refactoring

The existing `StreamClient` and `StreamingOrchestrator` get refactored into the `StreamPipeline`. Same HTTP mechanics, but invoked per-phase instead of post-hoc.

## 7. Fraud Scenario Packs

### 7.1 Three Layers

**Built-in presets** (shipped in binary via `include_str!`):

| Pack | Fraud Types | Use Case |
|------|-------------|----------|
| `revenue_fraud` | Revenue manipulation, fictitious entries, improper capitalization | Audit analytics demos |
| `payroll_ghost` | Ghost employees, SOD violations, self-approval | HR fraud detection |
| `vendor_kickback` | Fictitious vendors, round-dollar, split transactions | Procurement fraud |
| `management_override` | Self-approval, threshold manipulation, journal override | SOX compliance |
| `comprehensive` | All fraud types at moderate rates | General demos |

Each pack is a YAML fragment that merges into the base config:
```yaml
# fraud_packs/revenue_fraud.yaml
fraud:
  enabled: true
  fraud_rate: 0.02
  fraud_type_distribution:
    revenue_manipulation: 0.40
    fictitious_transaction: 0.30
    improper_capitalization: 0.20
    timing_anomaly: 0.10
anomaly_injection:
  rates:
    fraud_rate: 0.02
  multi_stage_schemes:
    enabled: true
    scheme_probability: 0.002
```

**CLI integration:**
```bash
# Named preset
datasynth-data generate --fraud-scenario revenue_fraud

# Multiple presets (merged left-to-right)
datasynth-data generate --fraud-scenario vendor_kickback --fraud-scenario payroll_ghost

# Toggle specific types
datasynth-data generate --enable-fraud ghost_employee,duplicate_payment

# Override rate
datasynth-data generate --fraud-scenario revenue_fraud --fraud-rate 0.05
```

**Counterfactual bridge** — fraud packs can be used as interventions:
```yaml
scenarios:
  scenarios:
    - name: "revenue_fraud_impact"
      interventions:
        - intervention_type:
            type: custom
            config_overrides:
              fraud.enabled: true
              fraud.fraud_rate: 0.03
              fraud.fraud_type_distribution.revenue_manipulation: 0.60
          timing:
            start_month: 7
            onset: "gradual"
            ramp_months: 3
```

This generates baseline (clean) vs counterfactual (fraud injected at month 7) datasets for training fraud detection models.

## 8. OCEL 2.0 Enrichment

### 8.1 Explicit Lifecycle State Machines

Each object type gets a formal state machine with transition probabilities and durations:

```
PurchaseOrder state machine:
  Draft → Submitted      (p=0.95, lag=2..8h)
  Draft → Cancelled      (p=0.05, lag=1..24h)
  Submitted → Approved   (p=0.90, lag=4..48h)
  Submitted → Rejected   (p=0.10, lag=2..24h)
  Approved → Released     (p=0.95, lag=1..4h)
  Released → PartiallyReceived (p=0.30, lag=3..14d)
  Released → FullyReceived     (p=0.70, lag=5..21d)
  FullyReceived → Closed       (p=1.0, lag=1..5d)
```

Events carry `from_state` and `to_state` fields. ObjectLanes can render state progression timelines.

State machines defined for all major object types: PurchaseOrder, GoodsReceipt, VendorInvoice, Payment, SalesOrder, Delivery, CustomerInvoice, CustomerReceipt, ProductionOrder, QualityInspection.

### 8.2 Multi-Object Correlation Events

New event types that explicitly reference 2+ objects:

| Event | Objects | Description |
|-------|---------|-------------|
| ThreeWayMatch | PO + GR + Invoice | Verification event at convergence point |
| PaymentAllocation | Payment + Invoice(s) | Payment applied to one or more invoices |
| IntercompanyElimination | IC Txn + Counter-IC Txn | Consolidation elimination |
| BankReconciliation | Bank Statement Line + JE | Statement-to-ledger matching |
| GoodsIssue | Production Order + Material + Inventory | Material consumption in manufacturing |

These events have multiple `EventObjectRef` entries with appropriate qualifiers (Consumed, Updated, Context). ObjectFlowView can render convergence/divergence points.

### 8.3 Resource Workload Modeling

```
ResourcePool
├── resources: Vec<Resource>         // AP Clerk 1, AP Clerk 2, etc.
├── capacity: WorkloadCapacity       // max_concurrent, hours_per_day
└── assignment: AssignmentStrategy   // RoundRobin | LeastBusy | SkillBased
```

Events assigned to specific resources based on workload balancing. Resource IDs are consistent across events (same clerk processes related documents). This enables resource-centric views and bottleneck analysis.

### 8.4 Enriched Event Output

```json
{
  "event_id": "...",
  "activity_name": "Approve Purchase Order",
  "from_state": "Submitted",
  "to_state": "Approved",
  "resource_id": "AP-CLERK-003",
  "resource_workload": 0.72,
  "object_refs": [
    {"object_id": "PO-1000", "qualifier": "Updated"},
    {"object_id": "BUDGET-FY2024-OPEX", "qualifier": "Context"}
  ],
  "correlation_id": "3WAY-MATCH-0042"
}
```

## 9. Cross-Feature Integration Points

The 5 features reinforce each other through the GenerationSession:

| Feature A | Feature B | Integration |
|-----------|-----------|-------------|
| Multi-period | Incremental | Same checkpoint mechanism; multi-period = planned periods, incremental = ad-hoc extensions |
| Multi-period | Streaming | Stream each period as generated; live progression streams one period at a time |
| Fraud packs | Counterfactual | Fraud packs are config overlays usable as counterfactual interventions |
| Fraud packs | Streaming | Injected anomalies stream with their labels in real time |
| OCEL enrichment | Streaming | Enriched events stream per-phase instead of batch post-processing |
| OCEL enrichment | Multi-period | Lifecycle state machines carry across periods (PO opened in FY2022, closed in FY2023) |
| Incremental | Streaming | Delta generation naturally streams new items as they're created |
| Fraud packs | Multi-period | Fraud injection timing can vary by period (Q4 fraud spike) |

## 10. Crates Affected

| Crate | Changes |
|-------|---------|
| `datasynth-core` | `SessionState`, `FiscalPeriod`, `BalanceState`, `DocumentIdState` models; OCEL state machine types |
| `datasynth-config` | `FraudPackConfig`, `SessionConfig`, `fiscal_year_months` field; fraud pack YAML templates |
| `datasynth-runtime` | `GenerationSession`, `StreamPipeline`, `PhaseSink` trait; refactor `EnhancedOrchestrator` into session-callable; `.dss` serialization |
| `datasynth-generators` | Fraud pack merge logic; OCEL lifecycle state machines; resource pool assignment |
| `datasynth-ocpm` | Multi-object correlation events; enriched event attributes; state machine transitions |
| `datasynth-eval` | Multi-period trend analysis; cross-period balance validation |
| `datasynth-cli` | `--append`, `--live`, `--fraud-scenario`, `--enable-fraud`, `--fraud-rate`, `--stream-embedded`, `--fiscal-year-months` flags |
| `datasynth-output` | Per-period directory output; consolidated output generation; CSV append mode |

## 11. Phasing

**Phase 1 — Foundation**: GenerationSession, SessionState, `.dss` serialization, multi-period loop, per-period output directories
**Phase 2 — Incremental**: Delta append, live progression mode, CLI flags
**Phase 3 — Streaming**: StreamPipeline, PhaseSink trait, phase-aware streaming, embedded mode
**Phase 4 — Fraud Packs**: Built-in presets, CLI integration, config merge, counterfactual bridge
**Phase 5 — OCEL Enrichment**: Lifecycle state machines, multi-object correlation events, resource workload modeling
**Phase 6 — Integration**: Docker Compose, consolidated output, cross-period OCEL, end-to-end tests
