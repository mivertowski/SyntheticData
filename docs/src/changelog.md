# Changelog

For the full changelog, see the [CHANGELOG.md](https://github.com/ey-asu-rnd/SyntheticData/blob/main/CHANGELOG.md) in the repository root.

## Recent Releases

### [0.9.2] - 2026-02-27

**Comprehensive Codebase Quality Fixes (Tiers 1-6)**

- Framework-aware account classification in all generators (balance tracker, trial balance, currency translator, IC generator, graph builder)
- Constant-time gRPC auth token comparison (`subtle::ConstantTimeEq`)
- Fixed employee generator `last_mut()` ordering, banking RNG determinism, CLI verify count mismatch, DGL node types
- Neo4j/DGL graph exports wired in orchestrator; server stream/reload/proto stubs implemented
- GoBD tax amount and contra account improvements for multi-line entries
- Config validation: start_date format, company name/country, safety limit warnings
- Production unwrap/expect calls replaced with descriptive errors across 6 crates
- Shared NPY writer extraction; proper Beta distribution; improved A-D p-value and AML detectability
- 59 files changed, 3,376 tests pass, 0 clippy warnings

### [0.9.1] - 2026-02-26

**Generalized Multi-GAAP Framework + German GAAP (HGB)**

- German GAAP (HGB) framework: SKR04 chart of accounts, Degressiv depreciation, GWG low-value asset expensing, BMF lease classification, mandatory impairment reversal
- Generalized `FrameworkAccounts` mapping ~45 semantic accounts per framework (US GAAP, French PCG, German SKR04)
- GoBD audit export (13-column journal CSV + account CSV + XML index)
- Auxiliary GL sub-accounts on vendor/customer master data (PCG `401XXXX`/`411XXXX`, SKR04 `3300XXXX`/`1200XXXX`)
- FEC auxiliary fields now use framework-specific GL accounts instead of raw partner IDs
- Expanded French PCG account modules (fixed assets, tax, suspense, equity, liabilities)

### [0.9.0] - 2026-02-25

**Performance & Dependencies**

- ~2x single-threaded throughput via cached temporal CDF, fast Decimal, SmallVec line items, parallel generation, and I/O optimization
- `ParallelGenerator` trait with deterministic seed splitting for multi-core generation
- `fast_csv` module, itoa/ryu formatting, zstd `CompressedWriter`
- Dependencies: rand 0.9, arrow/parquet 58, zip 8, jsonwebtoken 10, redis 1.0

### [0.8.1] - 2026-02-20

**French GAAP**

- French GAAP (PCG) accounting framework with Plan Comptable General 2024
- FEC export (Article A47 A-1 compliant)

### [0.8.0] - 2026-02-18

**Country Packs**

- Pluggable country pack architecture with runtime-loaded JSON packs
- Built-in packs: US, DE, GB with holidays, names, tax rates, addresses, payroll rules
- Generator integration across all modules

### [0.7.0] - 2026-02-17

**Enterprise Domains**

- Tax Accounting (ASC 740/IAS 12), Treasury & Cash Management, Project Accounting, ESG/Sustainability
- OCPM expanded to 12 process families with 101+ activities and 65+ object types
