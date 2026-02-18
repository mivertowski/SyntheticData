# Changelog

## 1.4.0 (2026-02-18)

### Added

- **Country Pack Config**: `country_packs` field on `Config` with `external_dir` and `overrides` support
- **Accounting Framework Config**: `framework` field (`us_gaap`, `ifrs`, `dual_reporting`) on accounting standards settings

### Changed

- Bumped version to 1.4.0 to match Rust v0.8.0 country pack wiring release

## 1.3.0 (2026-02-17)

### Added

- **Tax Accounting Config**: `TaxConfig` dataclass for tax jurisdictions, VAT/GST, withholding, provisions
- **Treasury Config**: `TreasuryConfig` dataclass for cash positioning, hedging, debt, netting
- **Project Accounting Config**: `ProjectAccountingConfig` dataclass for WBS, cost allocation, EVM, revenue recognition
- **ESG Config**: `EsgConfig` dataclass for environmental, social, governance, and reporting settings
- All four new domain configs added to `Config` class with `to_dict()` / `from_dict()` support

### Changed

- Bumped version to 1.3.0 to match Rust v0.7.0 release

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
