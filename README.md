# SyntheticData

[![Crates.io](https://img.shields.io/crates/v/datasynth-core.svg)](https://crates.io/crates/datasynth-core)
[![Documentation](https://docs.rs/datasynth-core/badge.svg)](https://docs.rs/datasynth-core)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml/badge.svg)](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml)

A high-performance, configurable synthetic data generator for enterprise financial simulation. SyntheticData produces realistic, interconnected General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, banking/KYC/AML transactions, OCEL 2.0 process mining data, and ML-ready graph exports at scale.

**Developed by [Ernst & Young Ltd.](https://www.ey.com/ch), Zurich, Switzerland**

---

## Table of Contents

- [SyntheticData](#syntheticdata)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Key Features](#key-features)
    - [Core Data Generation](#core-data-generation)
    - [Enterprise Simulation](#enterprise-simulation)
    - [Machine Learning \& Analytics](#machine-learning--analytics)
    - [Production Features](#production-features)
  - [Architecture](#architecture)
  - [Installation](#installation)
    - [From Source](#from-source)
    - [Requirements](#requirements)
  - [Quick Start](#quick-start)
    - [Demo Mode](#demo-mode)
  - [Configuration](#configuration)
  - [Output Structure](#output-structure)
  - [Use Cases](#use-cases)
  - [Performance](#performance)
  - [Server Usage](#server-usage)
  - [Desktop UI](#desktop-ui)
  - [Documentation](#documentation)
  - [License](#license)
  - [Support](#support)
  - [Acknowledgments](#acknowledgments)

---

## Overview

SyntheticData generates coherent enterprise financial data that mirrors the characteristics of real corporate accounting systems. The generated data is suitable for:

- **Machine Learning Model Development**: Training fraud detection, anomaly detection, and graph neural network models
- **Audit Analytics Testing**: Validating audit procedures and analytical tools with realistic data patterns
- **SOX Compliance Testing**: Testing internal controls and segregation of duties monitoring systems
- **System Integration Testing**: Load and stress testing for ERP and accounting platforms
- **Process Mining**: Generating realistic event logs for process discovery and conformance checking
- **Training and Education**: Providing realistic accounting data for professional development

The generator produces statistically accurate data based on empirical research from real-world general ledger patterns, ensuring that synthetic datasets exhibit the same characteristics as production data—including Benford's Law compliance, temporal patterns, and document flow integrity.

---

## Key Features

### Core Data Generation

| Feature | Description |
|---------|-------------|
| **Statistical Distributions** | Line item counts, amounts, and patterns based on empirical GL research |
| **Mixture Models** | Gaussian and Log-Normal mixture distributions with weighted components |
| **Copula Correlations** | Cross-field dependencies via Gaussian, Clayton, Gumbel, Frank, Student-t copulas |
| **Benford's Law Compliance** | First and second-digit distribution following Benford's Law with anomaly injection |
| **Regime Changes** | Economic cycles, acquisition effects, and structural breaks in time series |
| **Industry Presets** | Manufacturing, Retail, Financial Services, Healthcare, Technology, and more |
| **Chart of Accounts** | Small (~100), Medium (~400), Large (~2500) account structures |
| **Temporal Patterns** | Month-end, quarter-end, year-end volume spikes with working hour modeling |
| **Regional Calendars** | Holiday calendars for US, DE, GB, CN, JP, IN with lunar calendar support |

### Enterprise Simulation

- **Master Data Management**: Vendors, customers, materials, fixed assets, employees with temporal validity
- **Document Flow Engine**: Complete P2P (Procure-to-Pay) and O2C (Order-to-Cash) processes
- **Intercompany Transactions**: IC matching, transfer pricing, consolidation eliminations
- **Balance Coherence**: Opening balances, running balance tracking, trial balance generation
- **Subledger Simulation**: AR, AP, Fixed Assets, Inventory with GL reconciliation
- **Currency & FX**: Realistic exchange rates, currency translation, CTA generation
- **Period Close Engine**: Monthly close, depreciation runs, accruals, year-end closing
- **Banking/KYC/AML**: Customer personas, KYC profiles, AML typologies (structuring, funnel, mule, layering)
- **Process Mining**: OCEL 2.0 event logs with object-centric relationships
- **Audit Simulation**: ISA-compliant engagements, workpapers, findings, risk assessments
- **COSO 2013 Framework**: Full internal control framework with 5 components, 17 principles, and maturity levels
- **Accounting Standards**: US GAAP and IFRS support with ASC 606/IFRS 15 (revenue), ASC 842/IFRS 16 (leases), ASC 820/IFRS 13 (fair value), ASC 360/IAS 36 (impairment)
- **Audit Standards**: ISA (34 standards), PCAOB (19+ standards), SOX 302/404 compliance with deficiency classification

### Interconnectivity & Relationships

- **Multi-Tier Vendor Networks**: Tier1/Tier2/Tier3 supply chain modeling with parent-child hierarchies
- **Vendor Clusters**: ReliableStrategic, StandardOperational, Transactional, Problematic behavioral segmentation
- **Customer Value Segmentation**: Enterprise/MidMarket/SMB/Consumer with Pareto-like revenue distribution
- **Customer Lifecycle**: Prospect, New, Growth, Mature, AtRisk, Churned, WonBack stages
- **Relationship Strength**: Composite scoring from volume, count, duration, recency, and mutual connections
- **Cross-Process Links**: P2P↔O2C linkage via inventory (GoodsReceipt connects to Delivery)
- **Entity Graphs**: 16 entity types, 26 relationship types with graph metrics (connectivity, clustering, power law)

### Machine Learning & Analytics

- **Graph Export**: PyTorch Geometric, Neo4j, DGL, and RustGraph formats with train/val/test splits
- **Anomaly Injection**: 20+ fraud types, errors, process issues with full labeling
- **Data Quality Variations**: Missing values, format variations, duplicates, typos
- **Relationship Generation**: Configurable entity relationships with cardinality rules

### Privacy-Preserving Fingerprinting

- **Fingerprint Extraction**: Extract statistical properties from real data into `.dsf` files
- **Differential Privacy**: Laplace mechanism with configurable epsilon budget
- **K-Anonymity**: Suppression of rare categorical values
- **Privacy Audit Trail**: Complete logging of all privacy decisions
- **Fidelity Evaluation**: Validate synthetic data matches original fingerprint

### Production Features

- **REST & gRPC APIs**: Streaming generation with authentication and rate limiting
- **Streaming Output API**: Async generation with backpressure handling (Block, DropOldest, DropNewest, Buffer)
- **Rate Limiting**: Token bucket rate limiter for controlled generation throughput
- **Temporal Attributes**: Bi-temporal data support (valid time + transaction time) with version chains
- **Desktop UI**: Cross-platform Tauri/SvelteKit application
- **Resource Guards**: Memory, disk, and CPU monitoring with graceful degradation
- **Evaluation Framework**: Auto-tuning with configuration recommendations
- **Deterministic Generation**: Seeded RNG for reproducible output

---

## Architecture

SyntheticData is organized as a Rust workspace with 16 modular crates:

```
datasynth-cli          Command-line interface (binary: datasynth-data)
datasynth-server       REST/gRPC/WebSocket server with auth and rate limiting
datasynth-ui           Tauri/SvelteKit desktop application
    │
datasynth-runtime      Orchestration layer (parallel execution, resource guards)
    │
datasynth-generators   Data generators (JE, documents, subledgers, anomalies, audit)
datasynth-banking      KYC/AML banking transaction generator
datasynth-ocpm         Object-Centric Process Mining (OCEL 2.0)
datasynth-fingerprint  Privacy-preserving fingerprint extraction and synthesis
datasynth-standards    Accounting/audit standards (IFRS, US GAAP, ISA, SOX, PCAOB)
    │
datasynth-graph        Graph/network export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         Evaluation framework with auto-tuning
    │
datasynth-config       Configuration schema, validation, industry presets
    │
datasynth-core         Domain models, traits, distributions, resource guards
    │
datasynth-output       Output sinks (CSV, JSON, Parquet, streaming)
datasynth-test-utils   Test utilities, fixtures, mocks
```

See individual crate READMEs for detailed documentation.

---

## Installation

### From crates.io

```bash
# Install the CLI tool
cargo install datasynth-cli

# Or add individual crates to your project
cargo add datasynth-core datasynth-generators datasynth-config
```

### From Source

```bash
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release
```

The binary is available at `target/release/datasynth-data`.

### Available Crates

| Crate | Description |
|-------|-------------|
| [`datasynth-core`](https://crates.io/crates/datasynth-core) | Domain models, traits, distributions |
| [`datasynth-config`](https://crates.io/crates/datasynth-config) | Configuration schema and validation |
| [`datasynth-generators`](https://crates.io/crates/datasynth-generators) | Data generators |
| [`datasynth-banking`](https://crates.io/crates/datasynth-banking) | KYC/AML banking transactions |
| [`datasynth-fingerprint`](https://crates.io/crates/datasynth-fingerprint) | Privacy-preserving fingerprint extraction |
| [`datasynth-standards`](https://crates.io/crates/datasynth-standards) | Accounting/audit standards (IFRS, US GAAP, ISA, SOX, PCAOB) |
| [`datasynth-graph`](https://crates.io/crates/datasynth-graph) | Graph/network export |
| [`datasynth-eval`](https://crates.io/crates/datasynth-eval) | Evaluation framework |
| [`datasynth-runtime`](https://crates.io/crates/datasynth-runtime) | Orchestration layer |
| [`datasynth-cli`](https://crates.io/crates/datasynth-cli) | Command-line interface |
| [`datasynth-server`](https://crates.io/crates/datasynth-server) | REST/gRPC server |

### Requirements

- Rust 1.75 or later
- For the desktop UI: Node.js 18+ and platform-specific Tauri dependencies

---

## Quick Start

```bash
# Generate a configuration file for a manufacturing company
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate the configuration
datasynth-data validate --config config.yaml

# Generate synthetic data
datasynth-data generate --config config.yaml --output ./output

# View available presets and options
datasynth-data info
```

### Demo Mode

```bash
# Quick demo with default settings
datasynth-data generate --demo --output ./demo-output

# Generate with graph export for ML training
datasynth-data generate --demo --output ./demo-output --graph-export
```

---

## Configuration

SyntheticData uses YAML configuration files with comprehensive options:

```yaml
global:
  seed: 42                        # For reproducible generation
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD

companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    country: US
    volume_weight: 1.0            # Transaction volume weight

transactions:
  target_count: 100000
  benford:
    enabled: true

fraud:
  enabled: true
  fraud_rate: 0.005               # 0.5% fraud rate

anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true           # For supervised learning

graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j
    - rustgraph               # RustGraph/RustAssureTwin compatible JSON

streaming:
  enabled: true
  buffer_size: 1000
  backpressure: block         # block, drop_oldest, drop_newest, buffer

rate_limit:
  enabled: true
  entities_per_second: 10000
  burst_size: 100

distributions:
  enabled: true
  industry_profile: retail        # retail, manufacturing, financial_services
  amounts:
    enabled: true
    distribution_type: lognormal
    components:
      - { weight: 0.60, mu: 6.0, sigma: 1.5, label: "routine" }
      - { weight: 0.30, mu: 8.5, sigma: 1.0, label: "significant" }
      - { weight: 0.10, mu: 11.0, sigma: 0.8, label: "major" }
    benford_compliance: true
  correlations:
    enabled: true
    copula_type: gaussian         # gaussian, clayton, gumbel, frank, student_t
    fields: [amount, line_items, approval_level]
    matrix:
      - [1.00, 0.65, 0.72]
      - [0.65, 1.00, 0.55]
      - [0.72, 0.55, 1.00]
  regime_changes:
    enabled: true
    economic_cycle:
      enabled: true
      cycle_period_months: 48
      amplitude: 0.15
      recession_probability: 0.1
  validation:
    enabled: true
    tests:
      - { type: benford_first_digit, threshold_mad: 0.015 }
      - { type: distribution_fit, target: lognormal, significance: 0.05 }
      - { type: correlation_check, significance: 0.05 }

accounting_standards:
  enabled: true
  framework: us_gaap              # us_gaap, ifrs, dual_reporting
  revenue_recognition:
    enabled: true
    generate_contracts: true
  leases:
    enabled: true
    finance_lease_percent: 0.30

audit_standards:
  enabled: true
  isa_compliance:
    enabled: true
    compliance_level: comprehensive
    framework: dual               # isa, pcaob, dual
  sox:
    enabled: true
    materiality_threshold: 10000.0

vendor_network:
  enabled: true
  depth: 3                          # Tier1/Tier2/Tier3
  clusters:
    reliable_strategic: 0.20
    standard_operational: 0.50
    transactional: 0.25
    problematic: 0.05
  dependencies:
    max_single_vendor_concentration: 0.15
    top_5_concentration: 0.45

customer_segmentation:
  enabled: true
  value_segments:
    enterprise: { revenue_share: 0.40, customer_share: 0.05 }
    mid_market: { revenue_share: 0.35, customer_share: 0.20 }
    smb: { revenue_share: 0.20, customer_share: 0.50 }
    consumer: { revenue_share: 0.05, customer_share: 0.25 }

relationship_strength:
  enabled: true
  calculation:
    transaction_volume_weight: 0.30
    transaction_count_weight: 0.25
    relationship_duration_weight: 0.20
    recency_weight: 0.15
    mutual_connections_weight: 0.10

output:
  format: csv
  compression: none
```

See the [Configuration Guide](docs/configuration.md) for complete documentation.

---

## Output Structure

```
output/
├── master_data/          Vendors, customers, materials, assets, employees
├── transactions/         Journal entries, purchase orders, invoices, payments
├── subledgers/           AR, AP, FA, inventory detail records
├── period_close/         Trial balances, accruals, closing entries
├── consolidation/        Eliminations, currency translation
├── fx/                   Exchange rates, CTA adjustments
├── banking/              KYC profiles, bank transactions, AML typology labels
├── process_mining/       OCEL 2.0 event logs, process variants
├── audit/                Engagements, workpapers, findings, risk assessments
├── graphs/               PyTorch Geometric, Neo4j, DGL, RustGraph exports
├── labels/               Anomaly, fraud, and data quality labels for ML
├── controls/             Internal controls, COSO mappings, SoD rules
└── standards/            Accounting & audit standards outputs
    ├── accounting/       Contracts, leases, fair value, impairment tests
    └── audit/            ISA mappings, confirmations, opinions, SOX assessments
```

---

## Use Cases

| Use Case | Description |
|----------|-------------|
| **Fraud Detection ML** | Train supervised models with labeled fraud patterns |
| **Graph Neural Networks** | Entity relationship graphs for anomaly detection |
| **AML/KYC Testing** | Banking transaction data with structuring, layering, mule patterns |
| **Audit Analytics** | Test audit procedures with known control exceptions |
| **Process Mining** | OCEL 2.0 event logs for process discovery |
| **ERP Testing** | Load testing with realistic transaction volumes |
| **SOX Compliance** | Test internal control monitoring systems |
| **COSO Framework** | COSO 2013 control mapping with 5 components, 17 principles, maturity levels |
| **Standards Compliance** | IFRS/US GAAP revenue recognition, lease accounting, fair value, impairment testing |
| **Audit Standards** | ISA/PCAOB procedure mapping, analytical procedures, confirmations, audit opinions |
| **Data Quality ML** | Train models to detect missing values, typos, duplicates |
| **RustGraph Integration** | Stream data directly to RustAssureTwin knowledge graphs |

---

## Performance

| Metric | Performance |
|--------|-------------|
| Single-threaded throughput | ~100,000+ entries/second |
| Parallel scaling | Linear with available cores |
| Memory efficiency | Streaming generation for large volumes |

---

## Server Usage

```bash
# Start REST/gRPC server
cargo run -p datasynth-server -- --port 3000 --worker-threads 4

# API endpoints
curl http://localhost:3000/api/config
curl -X POST http://localhost:3000/api/stream/start
```

WebSocket streaming available at `ws://localhost:3000/ws/events`.

---

## Desktop UI

```bash
cd crates/datasynth-ui
npm install
npm run tauri dev
```

The desktop application provides visual configuration, real-time streaming, and preset management.

---

## Fingerprinting

Extract privacy-preserving fingerprints from real data and generate matching synthetic data:

```bash
# Extract fingerprint from CSV data
datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --privacy-level standard

# Validate fingerprint
datasynth-data fingerprint validate ./fingerprint.dsf

# Show fingerprint info
datasynth-data fingerprint info ./fingerprint.dsf --detailed

# Compare fingerprints
datasynth-data fingerprint diff ./fp1.dsf ./fp2.dsf

# Evaluate synthetic data fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --threshold 0.8
```

**Privacy Levels:**

| Level | Epsilon | k | Use Case |
|-------|---------|---|----------|
| minimal | 5.0 | 3 | Low privacy, high utility |
| standard | 1.0 | 5 | Balanced (default) |
| high | 0.5 | 10 | Higher privacy |
| maximum | 0.1 | 20 | Maximum privacy |

See the [Fingerprinting Guide](docs/fingerprint/) for complete documentation.

---

## Python Wrapper

A Python wrapper is available for programmatic access:

```bash
cd python
pip install -e ".[all]"
```

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

# Basic generation
config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
print(result.output_dir)

# Fingerprint operations
synth.fingerprint.extract("./real_data/", "./fingerprint.dsf", privacy_level="standard")
report = synth.fingerprint.evaluate("./fingerprint.dsf", "./synthetic/")
print(f"Fidelity score: {report.overall_score}")

# Streaming with pattern triggers
session = synth.stream(config=config)
session.trigger_month_end()  # Trigger month-end volume spike
async for event in session.events():
    process(event)
```

See the [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md) for complete documentation.

---

## Documentation

- [Configuration Guide](docs/configuration.md)
- [API Reference](docs/api.md)
- [Architecture Overview](docs/architecture.md)
- [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md)
- [Contributing Guidelines](CONTRIBUTING.md)

---

## License

Copyright 2024-2026 Michael Ivertowski, Ernst & Young Ltd., Zurich, Switzerland

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Support

Commercial support, custom development, and enterprise licensing are available upon request. Please contact the author at [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com) for inquiries.

---

## Acknowledgments

This project incorporates research on statistical distributions in accounting data and implements industry-standard patterns for enterprise financial systems.

---

*SyntheticData is provided "as is" without warranty of any kind. It is intended for testing, development, and educational purposes. Generated data should not be used as a substitute for real financial records.*
