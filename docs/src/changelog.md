# Changelog

For the full changelog, see the [CHANGELOG.md](https://github.com/ey-asu-rnd/SyntheticData/blob/main/CHANGELOG.md) in the repository root.

## Recent Releases

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
