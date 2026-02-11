# Changelog

## 1.0.0 (2026-02-11)

### Features

- **Async client**: `AsyncDataSynth` for non-blocking generation and WebSocket streaming
- **DataFrame integration**: `to_pandas()` and `to_polars()` for direct DataFrame loading
- **Table discovery**: `list_tables()` to enumerate available output tables
- **Streaming events**: `StreamEvent` dataclass for real-time generation progress
- **Config blueprints**: Pre-built configurations via `blueprints` module
  - `retail_small()`, `banking_medium()`, `manufacturing_large()`
  - `ml_training()`, `statistical_validation()`, `with_distributions()`
- **Fingerprint client**: `FingerprintClient` for extraction, validation, and evaluation
- **Config validation**: Client-side config validation before generation

### Configuration Models

- Full typed config models: `Config`, `GlobalSettings`, `CompanyConfig`, etc.
- Accounting standards: `AccountingStandardsConfig`, `AuditStandardsConfig`
- Revenue recognition, leases, fair value, impairment configs
- ISA/PCAOB compliance, SOX compliance settings

### Optional Dependencies

- `datasynth-py[pandas]` — pandas DataFrame support
- `datasynth-py[polars]` — polars DataFrame support
- `datasynth-py[jupyter]` — Jupyter + matplotlib
- `datasynth-py[streaming]` — WebSocket streaming
- `datasynth-py[all]` — everything

### Requirements

- Python >= 3.9
- DataSynth binary (`datasynth-data`) must be on PATH or specified via `binary_path`
