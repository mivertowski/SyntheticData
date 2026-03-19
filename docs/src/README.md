<div class="hero-section">

# DataSynth

<p class="subtitle">High-Performance Synthetic Enterprise Financial Data Generator</p>

<div class="badges">

[![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)](https://github.com/mivertowski/SyntheticData)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](https://github.com/mivertowski/SyntheticData/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)

</div>

<p class="attribution">Developed by Michael Ivertowski, Zurich, Switzerland</p>

</div>

## What is DataSynth?

DataSynth is a high-performance, configurable synthetic data generator that produces realistic, interconnected enterprise financial data at scale. It generates coherent General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, banking/KYC/AML transactions, OCEL 2.0 process mining data, audit workpapers, ML-ready graph exports, and complete enterprise process chains covering 20+ process families.

All generated data respects accounting identities (debits = credits, Assets = Liabilities + Equity), follows empirical distributions (Benford's Law, log-normal mixtures), and maintains referential integrity across 100+ output tables.

## Quick Links

| Section | Description |
|---------|-------------|
| [Getting Started](getting-started/README.md) | Installation, quick start guide, and demo mode |
| [User Guide](user-guide/README.md) | CLI reference, server API, desktop UI, Python SDK |
| [Configuration](configuration/README.md) | Complete YAML schema and industry presets |
| [Architecture](architecture/README.md) | System design, data flow, resource management |
| [Crate Reference](crates/README.md) | Detailed documentation for all 15 crates |
| [Advanced Topics](advanced/README.md) | Anomaly injection, graph export, fingerprinting, standards |
| [Deployment](deployment/README.md) | Docker, Kubernetes, bare metal, security hardening |
| [Use Cases](use-cases/README.md) | Fraud detection, audit, AML/KYC, compliance, ESG |
| [Changelog](changelog.md) | Release history and version details |

## Key Features

### Core Data Generation

| Feature | Description |
|---------|-------------|
| **Statistical Distributions** | Log-normal mixtures, Gaussian mixtures, Pareto, Weibull, Beta, zero-inflated with configurable components |
| **Copula Correlations** | Cross-field dependencies via Gaussian, Clayton, Gumbel, Frank, and Student-t copulas |
| **Benford's Law** | First and second-digit compliance with configurable deviation for anomaly injection |
| **Temporal Patterns** | Month-end/quarter-end/year-end volume spikes, intraday segments, business day calendars (15 regions), processing lags |
| **Regime Changes** | Economic cycles, acquisition effects, and structural breaks in time series |
| **Industry Presets** | Manufacturing, Retail, Financial Services, Healthcare, Technology |
| **Chart of Accounts** | Small (~100), Medium (~400), Large (~2500) account structures |
| **Country Packs** | Pluggable JSON packs (US, DE, GB + 7 more) with holidays, names, tax, addresses, payroll |

### Enterprise Process Simulation

DataSynth covers the full enterprise process landscape:

| Process Family | Scope |
|----------------|-------|
| **General Ledger** | Journal entries, chart of accounts, ACDOCA event logs |
| **Procure-to-Pay** | Purchase requisitions, POs, goods receipts, vendor invoices, payments, three-way match |
| **Order-to-Cash** | Sales orders, deliveries, customer invoices, receipts, dunning |
| **Source-to-Contract** | Spend analysis, sourcing projects, supplier qualification, RFx, bids, contracts, scorecards |
| **Hire-to-Retire** | Payroll, tax/deduction calculations, time & attendance, expense reports, benefit enrollment |
| **Manufacturing** | Production orders, BOM explosion, routing, WIP costing, quality inspections, cycle counts |
| **Financial Reporting** | Balance sheet, income statement, cash flow, changes in equity, KPIs, budget variance |
| **Tax Accounting** | Multi-jurisdiction, VAT/GST returns, ASC 740/IAS 12 provisions, FIN 48, withholding |
| **Treasury** | Cash positioning, forecasts, cash pooling, hedging (ASC 815/IFRS 9), debt covenants, netting |
| **Project Accounting** | WBS hierarchies, cost lines, PoC revenue, earned value (SPI/CPI/EAC), change orders |
| **ESG / Sustainability** | GHG Scope 1/2/3, energy/water/waste, diversity, safety, GRI/SASB/TCFD disclosures |
| **Intercompany** | IC matching, transfer pricing, consolidation eliminations, currency translation |
| **Subledgers** | AR, AP, Fixed Assets, Inventory with GL reconciliation |
| **Period Close** | Monthly close engine, depreciation, accruals, year-end closing entries |
| **Banking / KYC / AML** | Customer personas, KYC profiles, AML typologies (structuring, layering, mule, funnel) |
| **Audit** | ISA-compliant engagements, workpapers, evidence, risk assessments, findings |
| **Sales** | Quote-to-order pipeline with win rate modeling |
| **Bank Reconciliation** | Statement matching, outstanding checks, deposits in transit |

### Accounting & Audit Standards

- **Accounting frameworks**: US GAAP, IFRS, French GAAP (PCG), German GAAP (HGB/SKR04), dual reporting
- **Revenue recognition**: ASC 606 / IFRS 15 with performance obligations and SSP allocation
- **Leases**: ASC 842 / IFRS 16 with ROU assets and lease liabilities
- **Fair value**: ASC 820 / IFRS 13 Level 1/2/3 hierarchy
- **Impairment**: ASC 360 / IAS 36 testing with fair value estimation
- **Audit standards**: ISA (34 standards), PCAOB (19+ standards), SOX 302/404 compliance
- **COSO 2013**: 5 components, 17 principles, maturity levels
- **Localized exports**: FEC (French) and GoBD (German) audit file formats

### Fraud, Anomalies & Data Quality

- **ACFE-aligned fraud taxonomy**: Asset misappropriation, corruption, financial statement fraud
- **60+ anomaly types** with full labeling for supervised ML
- **Collusion modeling**: 9 ring types with role-based conspirators and escalation dynamics
- **Management override**: Fraud triangle modeling (pressure, opportunity, rationalization)
- **Red flag generation**: 40+ probabilistic fraud indicators with Bayesian calibration
- **Industry-specific patterns**: Manufacturing yield manipulation, retail sweethearting, healthcare upcoding
- **Data quality variations**: Missing values (MCAR/MAR/MNAR), format variations, typos, duplicates

### Machine Learning & Graph Export

- **Graph formats**: PyTorch Geometric, Neo4j, DGL, RustGraph JSON
- **Multi-layer hypergraph**: 3-layer (Governance, Process Events, Accounting Network)
- **Train/val/test splits** with configurable partitioning
- **Anomaly, fraud, quality, and drift labels** in standardized format
- **Evaluation framework**: Auto-tuning with quality gate enforcement

### Advanced Generation

| Capability | Description |
|------------|-------------|
| **LLM enrichment** | Pluggable providers (mock/OpenAI-compatible) for vendor names, descriptions, anomaly explanations |
| **Diffusion models** | Statistical diffusion with Langevin reverse process and hybrid blending |
| **Causal models** | Structural causal models with do-calculus interventions and counterfactual generation |
| **Natural language config** | Generate YAML configurations from plain English |
| **Scenario engine** | Built-in fraud packs: revenue_fraud, payroll_ghost, vendor_kickback, management_override |
| **Process mining** | OCEL 2.0 + XES 2.0 with 101+ activity types across 12 process families |

### Production Features

- **REST / gRPC / WebSocket APIs** with streaming and backpressure handling
- **Authentication**: API key (Argon2id), JWT/OIDC (RS256), RBAC (Admin/Operator/Viewer)
- **Resource guards**: Memory, disk, CPU monitoring with graceful degradation
- **Deterministic generation**: Seeded ChaCha8 RNG for reproducible output
- **Desktop UI**: Cross-platform Tauri/SvelteKit with 40+ configuration pages
- **Python SDK**: Programmatic access with blueprints and DataFrame loading
- **Docker & Kubernetes**: Distroless containers, Helm chart with HPA/PDB
- **Observability**: OpenTelemetry traces, Prometheus metrics, structured JSON logging
- **Data lineage**: Per-file checksums, lineage graph, W3C PROV-JSON export
- **Privacy-preserving fingerprinting**: Differential privacy, k-anonymity, federated extraction
- **Ecosystem integrations**: Apache Airflow, dbt, MLflow, Apache Spark

## Quick Start

```bash
# Install from source
git clone https://github.com/mivertowski/SyntheticData.git
cd SyntheticData
cargo build --release

# Demo mode
./target/release/datasynth-data generate --demo --output ./output

# Custom configuration
./target/release/datasynth-data init --industry manufacturing --complexity medium -o config.yaml
./target/release/datasynth-data generate --config config.yaml --output ./output
```

## Performance

| Metric | Value |
|--------|-------|
| Single-threaded throughput | 200,000+ journal entries/second |
| Parallel scaling | Linear with available CPU cores |
| Memory model | Streaming generation with configurable backpressure |
| Determinism | Fully reproducible via seeded ChaCha8 RNG |

## Architecture

DataSynth is organized as a Rust workspace with 15 modular crates:

```
datasynth-cli            CLI binary (generate, validate, init, info, fingerprint, scenario)
datasynth-server         REST / gRPC / WebSocket server with auth and rate limiting
datasynth-ui             Tauri + SvelteKit desktop application
                │
datasynth-runtime        Generation orchestrator (parallel execution, resource guards, streaming)
                │
datasynth-generators     50+ data generators across all process families
datasynth-banking        KYC / AML banking transaction generator
datasynth-ocpm           OCEL 2.0 / XES 2.0 process mining
datasynth-fingerprint    Privacy-preserving fingerprint extraction and synthesis
datasynth-standards      Accounting and audit standards (IFRS, US GAAP, ISA, SOX, PCAOB)
                │
datasynth-graph          Graph export (PyTorch Geometric, Neo4j, DGL, RustGraph, Hypergraph)
datasynth-eval           Statistical evaluation, quality gates, auto-tuning
                │
datasynth-config         Configuration schema, validation, industry presets
                │
datasynth-core           Domain models, traits, distributions, resource guards
                │
datasynth-output         Output sinks (CSV, JSON, NDJSON, Parquet + Zstd) with streaming
datasynth-test-utils     Test utilities, fixtures, mocks
```

## License

Copyright 2024-2026 Michael Ivertowski

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/mivertowski/SyntheticData/blob/main/LICENSE) for details.

## Support

Commercial support, custom development, and enterprise licensing are available upon request. Open an issue on [GitHub](https://github.com/mivertowski/SyntheticData/issues).

---

*DataSynth is provided "as is" without warranty of any kind. It is intended for testing, development, and research purposes. Generated data should not be used as a substitute for real financial records.*
