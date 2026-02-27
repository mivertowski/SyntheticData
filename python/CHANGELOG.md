# Changelog

## 1.5.3 (2026-02-27)

### Changed

- Matches Rust v0.9.3 edge-case hardening and defensive programming fixes
- Division-by-zero guards, ghost edge elimination, NaN guards, dead code removal

## 1.5.2 (2026-02-27)

### Changed

- Matches Rust v0.9.2 comprehensive codebase quality fixes (Tiers 1-6)
- Framework-aware account classification, constant-time auth, deterministic banking RNG
- Graph export wiring, server stub implementations, production unwrap elimination

## 1.5.1 (2026-02-26)

### Added

- `german_gaap` option for `framework` field in accounting standards configuration
- Matches Rust v0.9.1 generalized multi-GAAP framework with German HGB/SKR04 support

## 1.5.0 (2026-02-25)

### Changed

- Bumped version to 1.5.0 to match Rust v0.9.0 (performance optimizations + dependency upgrades)
- Rust engine now ~2x faster single-threaded throughput with parallel generation support
- Updated dependencies: rand 0.9, arrow/parquet 58, zip 8

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
