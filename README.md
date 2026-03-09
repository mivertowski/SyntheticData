# DataSynth

[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml/badge.svg)](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml)

**High-performance synthetic enterprise data generation for ML, audit analytics, and system testing.**

DataSynth generates statistically realistic, fully interconnected enterprise financial data at scale. It produces coherent General Ledger journal entries, document flows, subledger records, banking transactions, process mining event logs, and graph exports — covering 20+ enterprise process families from Procure-to-Pay through ESG reporting.

All generated data respects accounting identities (debits = credits, Assets = Liabilities + Equity), follows empirical distributions (Benford's Law, log-normal mixtures), and maintains referential integrity across 100+ output tables.

**Developed by [Ernst & Young Ltd.](https://www.ey.com/ch), Zurich, Switzerland**

---

## Table of Contents

- [Quick Start](#quick-start)
- [Key Capabilities](#key-capabilities)
- [Architecture](#architecture)
- [Installation](#installation)
- [Configuration](#configuration)
- [Output Structure](#output-structure)
- [Python SDK](#python-sdk)
- [Server & Deployment](#server--deployment)
- [Desktop UI](#desktop-ui)
- [Privacy-Preserving Fingerprinting](#privacy-preserving-fingerprinting)
- [Use Cases](#use-cases)
- [Performance](#performance)
- [Documentation](#documentation)
- [License](#license)

---

## Quick Start

```bash
# Build from source
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release

# Demo mode — generates a complete dataset with defaults
./target/release/datasynth-data generate --demo --output ./demo-output

# Or configure for your use case
./target/release/datasynth-data init --industry manufacturing --complexity medium -o config.yaml
./target/release/datasynth-data validate --config config.yaml
./target/release/datasynth-data generate --config config.yaml --output ./output
```

---

## Key Capabilities

### Statistical Foundations

DataSynth models real-world financial data characteristics from the ground up:

- **Distribution engine** — Log-normal mixtures, Gaussian mixtures, Pareto, Weibull, Beta, and zero-inflated distributions with configurable components
- **Copula correlations** — Cross-field dependency modeling via Gaussian, Clayton, Gumbel, Frank, and Student-t copulas
- **Benford's Law** — First and second-digit compliance with configurable deviation for anomaly injection
- **Temporal patterns** — Month-end/quarter-end/year-end volume spikes, intraday segments, business day calendars (15 regions), processing lags, and fiscal calendar support
- **Regime changes** — Economic cycles, acquisition effects, and structural breaks in time series
- **Industry profiles** — Pre-configured distributions for Retail, Manufacturing, Financial Services, Healthcare, and Technology

### Enterprise Process Simulation

Every process chain generates its own master data, documents, and journal entries — all cross-referenced:

| Process Family | Scope |
|----------------|-------|
| **General Ledger** | Journal entries, chart of accounts (small/medium/large), ACDOCA event logs |
| **Procure-to-Pay** | Purchase requisitions, POs, goods receipts, vendor invoices, payments, three-way match |
| **Order-to-Cash** | Sales orders, deliveries, customer invoices, receipts, dunning |
| **Source-to-Contract** | Spend analysis, sourcing projects, supplier qualification, RFx, bids, contracts, scorecards |
| **Hire-to-Retire** | Payroll runs, tax/deduction calculations, time & attendance, expense reports, benefit enrollment |
| **Manufacturing** | Production orders, BOM explosion, routing operations, WIP costing, quality inspections, cycle counts |
| **Financial Reporting** | Balance sheet, income statement, cash flow, changes in equity, KPIs, budget variance |
| **Tax Accounting** | Multi-jurisdiction tax (Federal/State/Local), VAT/GST returns, ASC 740/IAS 12 provisions, FIN 48 uncertain positions, withholding |
| **Treasury** | Cash positioning, probability-weighted forecasts, cash pooling, hedging (ASC 815/IFRS 9), debt covenants, netting |
| **Project Accounting** | WBS hierarchies, cost lines, percentage-of-completion revenue, earned value (SPI/CPI/EAC), change orders |
| **ESG / Sustainability** | GHG Scope 1/2/3 emissions, energy/water/waste, workforce diversity, safety metrics, GRI/SASB/TCFD disclosures |
| **Intercompany** | IC matching, transfer pricing, consolidation eliminations, currency translation |
| **Subledgers** | AR, AP, Fixed Assets, Inventory — each with GL reconciliation |
| **Period Close** | Monthly close engine, depreciation runs, accruals, year-end closing entries |
| **Banking / KYC / AML** | Customer personas, KYC profiles, AML typologies (structuring, layering, mule, funnel) |
| **Sales** | Quote-to-order pipeline with win rate modeling and pricing negotiation |
| **Bank Reconciliation** | Statement matching, outstanding checks, deposits in transit |
| **Audit** | ISA-compliant engagements, workpapers, evidence, risk assessments, findings |

### Accounting, Audit & Compliance Standards

- **Accounting frameworks** — US GAAP, IFRS, French GAAP (PCG), German GAAP (HGB/SKR04), and dual reporting
- **Revenue recognition** — ASC 606 / IFRS 15 with contract generation, performance obligations, and SSP allocation
- **Leases** — ASC 842 / IFRS 16 with ROU assets, lease liabilities, and classification
- **Fair value** — ASC 820 / IFRS 13 Level 1/2/3 hierarchy
- **Impairment** — ASC 360 / IAS 36 testing with fair value estimation
- **Audit standards** — ISA (34 standards), PCAOB (19+ standards) with procedure mapping
- **SOX compliance** — Section 302/404 assessments with deficiency classification and material weakness detection
- **COSO 2013** — 5 components, 17 principles, maturity levels, entity-level and transaction-level controls
- **Compliance regulations** — 45+ built-in standards registry, jurisdiction profiles (10 countries), regulatory filings, audit procedures, and compliance findings with full deficiency classification
- **Cross-domain compliance graph** — Standards linked to GL account types and business processes; full traversal paths (Company → Jurisdiction → Standard → Account → JournalEntry)
- **Localized exports** — FEC (French) and GoBD (German) audit file formats

### Interconnectivity & Relationships

- **Multi-tier vendor networks** — Tier 1/2/3 supply chain with behavioral clusters (Strategic, Operational, Transactional, Problematic)
- **Customer segmentation** — Enterprise/MidMarket/SMB/Consumer with Pareto-like revenue distribution and lifecycle stages
- **Relationship strength** — Composite scoring from volume, count, duration, recency, and mutual connections
- **Cross-process links** — P2P and O2C linked via inventory; payments linked to bank reconciliation
- **Entity graphs** — 16 entity types, 26 relationship types with connectivity and clustering metrics
- **Compliance-to-accounting links** — Standards mapped to GL account types and processes; findings linked to controls and affected accounts; filings linked to companies and jurisdictions

### Fraud, Anomalies & Data Quality

- **ACFE-aligned fraud taxonomy** — Asset misappropriation, corruption, and financial statement fraud with calibrated rates
- **60+ anomaly types** — Fraud, errors, process issues, statistical outliers, and relational anomalies
- **Collusion modeling** — 9 ring types with role-based conspirators, defection, and escalation dynamics
- **Management override** — Senior-level fraud patterns with fraud triangle modeling
- **Red flag generation** — 40+ probabilistic fraud indicators with Bayesian calibration
- **Industry-specific patterns** — Manufacturing yield manipulation, retail sweethearting, healthcare upcoding
- **Data quality variations** — Missing values (MCAR/MAR/MNAR), format variations, typos (keyboard-aware, OCR), duplicates, encoding issues
- **Full labeling** — Every injected anomaly and quality issue is labeled for supervised ML training

### Process & Behavioral Drift

- **Organizational events** — Acquisitions, divestitures, mergers, reorganizations with volume multipliers
- **Process evolution** — S-curve automation rollout, workflow changes, policy updates
- **Technology transitions** — ERP migrations with phased rollout (parallel run, cutover, stabilization)
- **Market drift** — Economic cycles, commodity price shocks, recession modeling
- **Labeled drift events** — Ground truth labels with magnitude and detection difficulty for ML training

### Machine Learning & Graph Export

- **Graph formats** — PyTorch Geometric (.pt), Neo4j (CSV + Cypher), DGL, RustGraph JSON
- **Multi-layer hypergraph** — 3-layer (Governance, Process Events, Accounting Network) with OCPM events as hyperedges and compliance regulation nodes
- **Compliance graph layer** — Standards, findings, filings, and jurisdictions as graph nodes with cross-domain edges to accounts, controls, and companies
- **Train/val/test splits** — Configurable data partitioning for ML pipelines
- **Anomaly labels** — Fraud labels, quality issue labels, and drift labels in standardized format
- **Counterfactual pairs** — (original, mutated) journal entry pairs for causal ML training

### Process Mining

- **OCEL 2.0** — Object-centric event logs in JSON/XML format
- **XES 2.0** — XML export compatible with ProM, Celonis, Disco, and pm4py
- **101+ activity types** across 12 process families with 65+ object types
- **10 OCPM generators** — S2C, H2R, MFG, BANK, AUDIT, Bank Recon, Tax, Treasury, Project Accounting, ESG
- **Process variants** — Happy path (75%), exception path (20%), error path (5%)

### Advanced Generation

| Capability | Description |
|------------|-------------|
| **LLM enrichment** | Pluggable `LlmProvider` trait (mock/OpenAI-compatible) for vendor names, descriptions, and anomaly explanations |
| **Diffusion models** | Statistical diffusion with Langevin reverse process; linear/cosine/sigmoid schedules; hybrid blending |
| **Causal models** | Structural causal models with do-calculus interventions and counterfactual abduction-action-prediction |
| **Natural language config** | Generate YAML configurations from plain English descriptions |
| **Scenario engine** | Built-in fraud packs: revenue_fraud, payroll_ghost, vendor_kickback, management_override, comprehensive |
| **Counterfactual simulation** | 8 intervention types with causal DAG propagation and diff analysis |

### Production Features

- **REST / gRPC / WebSocket APIs** with streaming generation and backpressure handling
- **Authentication** — API key (Argon2id), JWT/OIDC (RS256), role-based access control (Admin/Operator/Viewer)
- **Quality gates** — Configurable pass/fail thresholds (strict/default/lenient) with 8 metrics
- **Plugin SDK** — `GeneratorPlugin`, `SinkPlugin`, `TransformPlugin` traits with thread-safe registry
- **Resource guards** — Memory, disk, and CPU monitoring with graceful degradation (Normal → Reduced → Minimal → Emergency)
- **Deterministic generation** — Seeded ChaCha8 RNG for fully reproducible output
- **Streaming output** — Async generation with configurable backpressure (block/drop_oldest/drop_newest/buffer)
- **Data lineage** — Per-file checksums, lineage graph, W3C PROV-JSON export
- **Country packs** — Pluggable JSON country configuration (US/DE/GB built-in) with holidays, names, tax, addresses
- **Observability** — OpenTelemetry traces, Prometheus metrics, structured JSON logging
- **Docker & Kubernetes** — Multi-stage distroless containers, Helm chart with HPA/PDB, Prometheus ServiceMonitor
- **CI/CD** — 7-job GitHub Actions pipeline (fmt, clippy, cross-platform test, MSRV, security, coverage, benchmarks)
- **EU AI Act** — Article 50 synthetic content marking and Article 10 data governance reports
- **Fuzzing** — cargo-fuzz targets for config parsing, fingerprint loading, and validation
- **Panic-free** — `#![deny(clippy::unwrap_used)]` enforced across all library crates

### Ecosystem Integrations

| Integration | Capability |
|-------------|------------|
| **Apache Airflow** | `DataSynthOperator`, `DataSynthSensor`, `DataSynthValidateOperator` for DAG orchestration |
| **dbt** | Source YAML generation, seed export, project scaffolding |
| **MLflow** | Generation runs as experiments with parameter, metric, and artifact logging |
| **Apache Spark** | DataFrames with schema inference and temp view registration |

---

## Architecture

DataSynth is a Rust workspace organized into 15 modular crates:

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

---

## Installation

### From Source

```bash
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release
```

The binary is available at `target/release/datasynth-data`.

### Requirements

- **Rust 1.88+**
- **Desktop UI**: Node.js 18+ and platform-specific [Tauri prerequisites](https://tauri.app/start/prerequisites/)

---

## Configuration

DataSynth uses YAML configuration with 30+ top-level sections. Generate a starter config with `init`:

```bash
datasynth-data init --industry retail --complexity medium -o config.yaml
```

**Minimal configuration:**

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD

companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    country: US

transactions:
  target_count: 100000

output:
  format: csv               # csv, json, parquet
```

**Enable specific modules by adding their sections:**

```yaml
# Fraud detection training data
fraud:
  enabled: true
  fraud_rate: 0.005
anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

# Graph export for GNN training
graph_export:
  enabled: true
  formats: [pytorch_geometric, neo4j]

# Statistical realism
distributions:
  enabled: true
  industry_profile: retail
  amounts:
    distribution_type: lognormal
    benford_compliance: true
  correlations:
    enabled: true
    copula_type: gaussian

# Enterprise process chains
document_flows:
  enabled: true
source_to_pay:
  enabled: true
hr:
  enabled: true
manufacturing:
  enabled: true
financial_reporting:
  enabled: true
esg:
  enabled: true

# Accounting standards
accounting_standards:
  enabled: true
  framework: us_gaap         # us_gaap, ifrs, french_gaap, german_gaap, dual_reporting

# Process mining
ocpm:
  enabled: true
  output:
    ocel_json: true
    xes: true
```

**Industry presets** (manufacturing, retail, financial_services, healthcare, technology) and **complexity levels** (small ~100 accounts, medium ~400, large ~2500) provide sensible defaults.

See the [Configuration Guide](docs/configuration.md) for the complete reference.

---

## Output Structure

DataSynth generates 100+ interconnected output tables organized by domain:

```
output/
├── master_data/            Vendors, customers, materials, fixed assets, employees
├── transactions/           Journal entries, ACDOCA, purchase orders, invoices, payments
├── sourcing/               S2C pipeline (projects, RFx, bids, contracts, scorecards)
├── subledgers/             AR, AP, Fixed Assets, Inventory detail records
├── hr/                     Payroll runs, payslips, time entries, expense reports
├── manufacturing/          Production orders, routing, quality inspections, cycle counts
├── period_close/           Trial balances, accruals, depreciation, closing entries
├── financial_reporting/    Balance sheet, income statement, cash flow, KPIs, budgets
├── sales/                  Sales quotes and line items
├── consolidation/          IC eliminations, currency translation
├── fx/                     Exchange rates, CTA adjustments
├── banking/                KYC profiles, bank transactions, reconciliation, AML labels
├── process_mining/         OCEL 2.0 JSON, XES 2.0, process variants, reference models
├── audit/                  Engagements, workpapers, evidence, risks, findings
├── graphs/                 PyTorch Geometric, Neo4j, DGL, RustGraph, hypergraph
├── labels/                 Anomaly, fraud, quality, and drift labels for ML
├── tax/                    Jurisdictions, codes, returns, provisions, withholding
├── treasury/               Cash positions, forecasts, hedging, debt, netting
├── project_accounting/     Projects, WBS, costs, revenue, earned value, change orders
├── esg/                    Emissions, energy, diversity, safety, disclosures
├── controls/               Internal controls, COSO mappings, SoD rules
└── standards/              Accounting contracts/leases/impairment, audit ISA/SOX
```

---

## Python SDK

```bash
cd python && pip install -e ".[all]"
```

```python
from datasynth_py import DataSynth
from datasynth_py import to_pandas, to_polars, list_tables
from datasynth_py.config import blueprints

# Generate with a preset blueprint
config = blueprints.retail_small(companies=4, transactions=10000)
result = DataSynth().generate(config=config, output={"format": "csv", "sink": "temp_dir"})

# Load as DataFrames
tables = list_tables(result)                  # ['journal_entries', 'vendors', ...]
df = to_pandas(result, "journal_entries")
pl_df = to_polars(result, "vendors")

# Async generation
from datasynth_py import AsyncDataSynth
async with AsyncDataSynth() as synth:
    result = await synth.generate(config=config)

# Fingerprint operations
synth = DataSynth()
synth.fingerprint.extract("./real_data/", "./fingerprint.dsf", privacy_level="standard")
report = synth.fingerprint.evaluate("./fingerprint.dsf", "./synthetic/")
```

**Available blueprints:** `retail_small()`, `banking_medium()`, `manufacturing_large()`, `ml_training()`, `statistical_validation()`, `with_distributions()`, `with_llm_enrichment()`, `with_diffusion()`, `with_causal()`

**Optional dependencies:** `[pandas]`, `[polars]`, `[jupyter]`, `[streaming]`, `[airflow]`, `[dbt]`, `[mlflow]`, `[spark]`, `[all]`

---

## Server & Deployment

```bash
# Start REST + gRPC server
cargo run -p datasynth-server -- --rest-port 3000 --grpc-port 50051

# With authentication
cargo run -p datasynth-server -- --api-keys "key1,key2"

# With JWT/OIDC (Keycloak, Auth0, Entra ID)
cargo run -p datasynth-server --features jwt -- \
  --jwt-issuer "https://auth.example.com" \
  --jwt-audience "datasynth-api"
```

**API endpoints:**

```bash
curl http://localhost:3000/health
curl http://localhost:3000/ready
curl http://localhost:3000/metrics
curl -H "Authorization: Bearer <key>" -X POST http://localhost:3000/api/stream/start
```

WebSocket streaming: `ws://localhost:3000/ws/events`

**Docker:**

```bash
docker build -t datasynth:latest .
docker run -p 3000:3000 -p 50051:50051 datasynth:latest

# Full stack with Prometheus + Grafana
docker compose up -d
```

See the [Deployment Guide](deploy/README.md) for Docker, Kubernetes Helm chart, systemd, and reverse proxy configuration.

---

## Desktop UI

```bash
cd crates/datasynth-ui
npm install
npm run tauri dev
```

Cross-platform Tauri + SvelteKit application with 40+ configuration pages, real-time streaming visualization, and preset management.

---

## Privacy-Preserving Fingerprinting

Extract statistical fingerprints from real data with formal privacy guarantees, then generate matching synthetic data:

```bash
# Extract with differential privacy
datasynth-data fingerprint extract --input ./real_data.csv --output ./fp.dsf --privacy-level standard

# Validate and evaluate
datasynth-data fingerprint validate ./fp.dsf
datasynth-data fingerprint evaluate --fingerprint ./fp.dsf --synthetic ./synthetic/
```

| Privacy Level | Epsilon (ε) | k-Anonymity | Description |
|---------------|-------------|-------------|-------------|
| minimal       | 5.0         | 3           | Higher utility, lower privacy |
| standard      | 1.0         | 5           | Balanced (default) |
| high          | 0.5         | 10          | Higher privacy |
| maximum       | 0.1         | 20          | Maximum privacy |

Features include Rényi DP and zCDP composition accounting, privacy budget management, federated fingerprinting for distributed data, membership inference attack testing, and cryptographic synthetic data certificates (HMAC-SHA256).

---

## Use Cases

| Domain | Application |
|--------|-------------|
| **Fraud Detection** | Train supervised models with ACFE-aligned labeled fraud patterns and collusion networks |
| **Graph Neural Networks** | Entity relationship graphs with typed edges for anomaly detection |
| **AML / KYC Testing** | Banking transactions with structuring, layering, and mule typologies |
| **Audit Analytics** | Validate audit procedures with known control exceptions and ISA/PCAOB mappings |
| **Process Mining** | OCEL 2.0 and XES 2.0 event logs for process discovery and conformance checking |
| **ERP Load Testing** | Realistic transaction volumes with proper document chains |
| **SOX Compliance** | Internal control monitoring with COSO 2013 mappings and deficiency classification |
| **Causal ML Research** | Interventional and counterfactual datasets with causal DAG propagation |
| **Data Quality ML** | Train models to detect missing values, format variations, typos, and duplicates |
| **ESG Reporting** | GHG emissions, diversity metrics, and GRI/SASB/TCFD disclosure data |
| **Tax Compliance** | Multi-jurisdiction tax returns, provisions, and withholding records |
| **Treasury Operations** | Cash positioning, hedging effectiveness, and debt covenant monitoring |

---

## Performance

| Metric | Value |
|--------|-------|
| Single-threaded throughput | 200,000+ journal entries/second |
| Parallel scaling | Linear with available CPU cores |
| Memory model | Streaming generation with configurable backpressure |
| Determinism | Fully reproducible via seeded ChaCha8 RNG |

---

## Documentation

- [Configuration Guide](docs/configuration.md)
- [API Reference](docs/api.md)
- [Architecture Overview](docs/architecture.md)
- [Python SDK Guide](docs/src/user-guide/python-wrapper.md)
- [Deployment Guide](deploy/README.md)
- [Fingerprinting Guide](docs/fingerprint/)
- [Compliance & Regulatory](docs/src/compliance/README.md)
- [Contributing](CONTRIBUTING.md)

---

## License

Copyright 2024–2026 Michael Ivertowski, Ernst & Young Ltd., Zurich, Switzerland

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Support

Commercial support, custom development, and enterprise licensing are available. Contact [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com).

---

*DataSynth is provided "as is" without warranty of any kind. It is intended for testing, development, and research purposes. Generated data should not be used as a substitute for real financial records.*
