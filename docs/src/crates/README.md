# Crate Reference

DataSynth is organized as a Rust workspace with 15 modular crates. This section provides detailed documentation for each crate.

## Workspace Structure

```
datasynth-cli          → Binary entry point (commands: generate, validate, init, info, fingerprint)
datasynth-server       → REST/gRPC/WebSocket server with auth, rate limiting, timeouts
datasynth-ui           → Tauri/SvelteKit desktop UI
    ↓
datasynth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
datasynth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies, Audit)
datasynth-banking      → KYC/AML banking transaction generator with fraud typologies
datasynth-ocpm         → Object-Centric Process Mining (OCEL 2.0 event logs)
datasynth-fingerprint  → Privacy-preserving fingerprint extraction and synthesis
datasynth-standards    → Accounting/audit standards (US GAAP, IFRS, French GAAP, German GAAP, ISA, SOX)
    ↓
datasynth-graph        → Graph/network export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         → Evaluation framework with auto-tuning and recommendations
    ↓
datasynth-config       → Configuration schema, validation, industry presets
    ↓
datasynth-core         → Domain models, traits, distributions, templates, resource guards
    ↓
datasynth-output       → Output sinks (CSV, JSON, Parquet, ControlExport)

datasynth-test-utils   → Testing utilities and fixtures
```

## Crate Categories

### Application Layer

| Crate | Description |
|-------|-------------|
| [datasynth-cli](datasynth-cli.md) | Command-line interface binary with generate, validate, init, info, fingerprint commands |
| [datasynth-server](datasynth-server.md) | REST/gRPC/WebSocket server with authentication, rate limiting, and timeouts |
| [datasynth-ui](datasynth-ui.md) | Cross-platform desktop GUI application (Tauri + SvelteKit) |

### Core Processing

| Crate | Description |
|-------|-------------|
| [datasynth-runtime](datasynth-runtime.md) | Generation orchestration with resource guards and graceful degradation |
| [datasynth-generators](datasynth-generators.md) | All data generators (JE, master data, documents, subledgers, anomalies, audit) |
| [datasynth-graph](datasynth-graph.md) | ML graph export (PyTorch Geometric, Neo4j, DGL) |

### Domain-Specific Modules

| Crate | Description |
|-------|-------------|
| [datasynth-banking](datasynth-banking.md) | KYC/AML banking transactions with fraud typologies |
| [datasynth-ocpm](datasynth-ocpm.md) | Object-Centric Process Mining (OCEL 2.0) |
| [datasynth-fingerprint](datasynth-fingerprint.md) | Privacy-preserving fingerprint extraction and synthesis |
| [datasynth-standards](datasynth-standards.md) | Accounting/audit standards (US GAAP, IFRS, French GAAP, German GAAP, ISA, SOX, PCAOB) |

### Foundation

| Crate | Description |
|-------|-------------|
| [datasynth-core](datasynth-core.md) | Domain models, distributions, traits, resource guards |
| [datasynth-config](datasynth-config.md) | Configuration schema and validation |
| [datasynth-output](datasynth-output.md) | Output sinks (CSV, JSON, Parquet) |

### Supporting

| Crate | Description |
|-------|-------------|
| [datasynth-eval](datasynth-eval.md) | Quality evaluation with auto-tuning recommendations |
| [datasynth-test-utils](datasynth-test-utils.md) | Test utilities and fixtures |

## Dependencies

The crates follow a strict dependency hierarchy:

1. **datasynth-core**: No internal dependencies (foundation)
2. **datasynth-config**: Depends on datasynth-core
3. **datasynth-output**: Depends on datasynth-core
4. **datasynth-generators**: Depends on datasynth-core, datasynth-config
5. **datasynth-graph**: Depends on datasynth-core, datasynth-generators
6. **datasynth-eval**: Depends on datasynth-core
7. **datasynth-banking**: Depends on datasynth-core, datasynth-config
8. **datasynth-ocpm**: Depends on datasynth-core
9. **datasynth-fingerprint**: Depends on datasynth-core, datasynth-config
10. **datasynth-runtime**: Depends on datasynth-core, datasynth-config, datasynth-generators, datasynth-output, datasynth-graph, datasynth-banking, datasynth-ocpm, datasynth-fingerprint, datasynth-eval
11. **datasynth-cli**: Depends on datasynth-runtime, datasynth-fingerprint
12. **datasynth-server**: Depends on datasynth-runtime
13. **datasynth-ui**: Depends on datasynth-runtime (via Tauri)
14. **datasynth-standards**: Depends on datasynth-core, datasynth-config
15. **datasynth-test-utils**: Depends on datasynth-core

## Building Individual Crates

```bash
# Build specific crate
cargo build -p datasynth-core
cargo build -p datasynth-generators
cargo build -p datasynth-fingerprint

# Run tests for specific crate
cargo test -p datasynth-core
cargo test -p datasynth-generators
cargo test -p datasynth-fingerprint

# Generate docs for specific crate
cargo doc -p datasynth-core --open
cargo doc -p datasynth-fingerprint --open
```

## API Documentation

For detailed Rust API documentation, generate and view rustdoc:

```bash
cargo doc --workspace --no-deps --open
```

After deployment, API documentation is available at `/api/` in the documentation site.

## See Also

- [Architecture Overview](../architecture/README.md)
- [Workspace Layout](../architecture/workspace-layout.md)
- [Domain Models](../architecture/domain-models.md)
