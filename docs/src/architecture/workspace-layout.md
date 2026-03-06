# Workspace Layout

DataSynth is organized as a Rust workspace with 15 crates following a layered architecture.

## Crate Hierarchy

```
datasynth-cli          → Binary entry point (commands: generate, validate, init, info, fingerprint)
datasynth-server       → REST/gRPC/WebSocket server with auth, rate limiting, timeouts
datasynth-ui           → Tauri/SvelteKit desktop UI
    │
    ▼
datasynth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    │
    ├─────────────────────────────────────┐
    ▼                                     ▼
datasynth-generators   datasynth-banking  datasynth-ocpm  datasynth-fingerprint  datasynth-standards
    │                        │                  │                    │
    └────────────────────────┴──────────────────┴────────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                    ▼                                 ▼
           datasynth-graph                    datasynth-eval
                    │                                 │
                    └────────────────┬────────────────┘
                                     ▼
                            datasynth-config
                                     │
                                     ▼
                            datasynth-core         → Foundation layer
                                     │
                                     ▼
                            datasynth-output

                            datasynth-test-utils   → Testing utilities
```

## Dependency Matrix

| Crate | Depends On |
|-------|------------|
| datasynth-core | (none) |
| datasynth-config | datasynth-core |
| datasynth-output | datasynth-core |
| datasynth-generators | datasynth-core, datasynth-config |
| datasynth-graph | datasynth-core, datasynth-generators |
| datasynth-eval | datasynth-core |
| datasynth-banking | datasynth-core, datasynth-config |
| datasynth-ocpm | datasynth-core |
| datasynth-fingerprint | datasynth-core, datasynth-config |
| datasynth-standards | datasynth-core, datasynth-config |
| datasynth-runtime | datasynth-core, datasynth-config, datasynth-generators, datasynth-output, datasynth-graph, datasynth-banking, datasynth-ocpm, datasynth-fingerprint, datasynth-eval |
| datasynth-cli | datasynth-runtime, datasynth-fingerprint |
| datasynth-server | datasynth-runtime |
| datasynth-ui | datasynth-runtime (via Tauri) |
| datasynth-test-utils | datasynth-core |

## Directory Structure

```
DataSynth/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── datasynth-core/
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # Domain models (JournalEntry, Master data, etc.)
│   │       ├── distributions/  # Statistical samplers
│   │       ├── traits/         # Generator, Sink, PostProcessor traits
│   │       ├── templates/      # Template loading system
│   │       ├── accounts.rs     # GL account constants
│   │       ├── uuid_factory.rs # Deterministic UUID generation
│   │       ├── memory_guard.rs # Memory limit enforcement
│   │       ├── disk_guard.rs   # Disk space monitoring
│   │       ├── cpu_monitor.rs  # CPU load tracking
│   │       ├── resource_guard.rs # Unified resource orchestration
│   │       ├── degradation.rs  # Graceful degradation controller
│   │       ├── llm/            # LLM provider abstraction (Mock, HTTP, OpenAI, Anthropic)
│   │       ├── diffusion/      # Diffusion model backend (statistical, hybrid, training)
│   │       └── causal/         # Causal graphs, SCMs, interventions, counterfactuals
│   ├── datasynth-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs       # Configuration schema
│   │       ├── validation.rs   # Config validation rules
│   │       └── presets/        # Industry preset definitions
│   ├── datasynth-generators/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── je_generator.rs
│   │       ├── coa_generator.rs
│   │       ├── control_generator.rs
│   │       ├── master_data/    # Vendor, Customer, Material, Asset, Employee
│   │       ├── document_flow/  # P2P, O2C, three-way match
│   │       ├── intercompany/   # IC generation, matching, elimination
│   │       ├── balance/        # Opening balance, balance tracker
│   │       ├── subledger/      # AR, AP, FA, Inventory
│   │       ├── fx/             # FX rates, translation, CTA
│   │       ├── period_close/   # Close engine, accruals, depreciation
│   │       ├── anomaly/        # Anomaly injection engine
│   │       ├── data_quality/   # Missing values, typos, duplicates
│   │       ├── audit/          # Engagement, workpaper, evidence, findings
│   │       ├── llm_enrichment/ # LLM-powered vendor names, descriptions, anomaly explanations
│   │       └── relationships/  # Entity graph, cross-process links, relationship strength
│   ├── datasynth-output/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── csv_sink.rs
│   │       ├── json_sink.rs
│   │       └── control_export.rs
│   ├── datasynth-graph/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # Node, edge types
│   │       ├── builders/       # Transaction, approval, entity graphs
│   │       ├── exporters/      # PyTorch Geometric, Neo4j, DGL
│   │       └── ml/             # Feature computation, train/val/test splits
│   ├── datasynth-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs # GenerationOrchestrator
│   │       └── progress.rs     # Progress tracking
│   ├── datasynth-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs         # generate, validate, init, info, fingerprint commands
│   ├── datasynth-server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── rest/           # Axum REST API
│   │       ├── grpc/           # Tonic gRPC service
│   │       └── websocket/      # WebSocket streaming
│   ├── datasynth-ui/
│   │   ├── package.json
│   │   ├── src/                # SvelteKit frontend
│   │   │   ├── routes/         # 15+ config pages
│   │   │   └── lib/            # Components, stores
│   │   └── src-tauri/          # Rust backend
│   ├── datasynth-eval/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── statistical/    # Benford, distributions, temporal
│   │       ├── coherence/      # Balance, IC, document chains
│   │       ├── quality/        # Completeness, consistency, duplicates
│   │       ├── ml/             # Feature distributions, label quality
│   │       ├── report/         # HTML/JSON report generation
│   │       └── enhancement/    # AutoTuner, RecommendationEngine
│   ├── datasynth-banking/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # Customer, Account, Transaction, KYC
│   │       ├── generators/     # Customer, account, transaction generation
│   │       ├── typologies/     # Structuring, funnel, layering, mule, fraud
│   │       ├── personas/       # Retail, business, trust behaviors
│   │       └── labels/         # Entity, relationship, transaction labels
│   ├── datasynth-ocpm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # EventLog, Event, ObjectInstance, ObjectType
│   │       ├── generator/      # P2P, O2C event generation
│   │       └── export/         # OCEL 2.0 JSON export
│   ├── datasynth-fingerprint/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/         # Fingerprint, Manifest, Schema, Statistics
│   │       ├── privacy/        # Laplace, Gaussian, k-anonymity, PrivacyEngine
│   │       ├── extraction/     # Schema, stats, correlation, integrity extractors
│   │       ├── io/             # DSF file reader, writer, validator
│   │       ├── synthesis/      # ConfigSynthesizer, DistributionFitter, GaussianCopula
│   │       ├── evaluation/     # FidelityEvaluator, FidelityReport
│   │       ├── federated/      # Federated fingerprint protocol, secure aggregation
│   │       └── certificates/   # Synthetic data certificates, HMAC-SHA256 signing
│   ├── datasynth-standards/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── framework.rs     # AccountingFramework, FrameworkSettings
│   │       ├── accounting/      # Revenue (ASC 606/IFRS 15), Leases, Fair Value, Impairment
│   │       ├── audit/           # ISA standards, Analytical procedures, Opinions
│   │       └── regulatory/      # SOX 302/404, DeficiencyMatrix
│   └── datasynth-test-utils/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs          # Test fixtures, assertions, mocks
├── benches/                    # Criterion benchmark suite
├── docs/                       # This documentation (mdBook)
├── python/                     # Python wrapper (datasynth_py)
├── examples/                   # Example configurations and templates
└── tests/                      # Integration tests
```

## Crate Purposes

### Application Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-cli** | Command-line interface with generate, validate, init, info, fingerprint commands |
| **datasynth-server** | REST/gRPC/WebSocket API with auth, rate limiting, timeouts |
| **datasynth-ui** | Cross-platform desktop application (Tauri + SvelteKit) |

### Processing Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-runtime** | Orchestrates generation workflow with resource guards |
| **datasynth-generators** | Core data generation (JE, master data, documents, anomalies, audit) |
| **datasynth-graph** | Graph construction and export for ML |

### Domain-Specific Modules

| Crate | Purpose |
|-------|---------|
| **datasynth-banking** | KYC/AML banking transactions with fraud typologies |
| **datasynth-ocpm** | OCEL 2.0 process mining event logs |
| **datasynth-fingerprint** | Privacy-preserving fingerprint extraction and synthesis |
| **datasynth-standards** | Accounting/audit standards (US GAAP, IFRS, ISA, SOX, PCAOB) |

### Foundation Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-core** | Domain models, traits, distributions, resource guards |
| **datasynth-config** | Configuration schema and validation |
| **datasynth-output** | Output sinks (CSV, JSON, Parquet) |

### Supporting Crates

| Crate | Purpose |
|-------|---------|
| **datasynth-eval** | Quality evaluation with auto-tuning recommendations |
| **datasynth-test-utils** | Test fixtures and assertions |

## Build Commands

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p datasynth-core
cargo build -p datasynth-generators
cargo build -p datasynth-fingerprint

# Run tests
cargo test
cargo test -p datasynth-core
cargo test -p datasynth-fingerprint

# Generate documentation
cargo doc --workspace --no-deps

# Run benchmarks
cargo bench
```

## Feature Flags

Workspace-level features:

```toml
[workspace.features]
default = ["full"]
full = ["server", "ui", "graph"]
server = []
ui = []
graph = []
```

Crate-level features:

```toml
# datasynth-core
[features]
templates = ["serde_yaml"]

# datasynth-output
[features]
compression = ["flate2", "zstd"]
```

## Adding a New Crate

1. Create directory: `crates/datasynth-newcrate/`
2. Add `Cargo.toml`:
   ```toml
   [package]
   name = "datasynth-newcrate"
   version = "0.2.0"
   edition = "2021"

   [dependencies]
   datasynth-core = { path = "../datasynth-core" }
   ```
3. Add to workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       # ...
       "crates/datasynth-newcrate",
   ]
   ```
4. Create `src/lib.rs`
5. Add documentation to `docs/src/crates/`

## See Also

- [Crate Reference](../crates/README.md)
- [Domain Models](domain-models.md)
- [Data Flow](data-flow.md)
- [Memory Management](memory-management.md)
