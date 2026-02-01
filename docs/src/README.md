<div class="hero-section">

# SyntheticData

<p class="subtitle">High-Performance Synthetic Enterprise Financial Data Generator</p>

<div class="badges">

[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](https://github.com/ey-asu-rnd/SyntheticData)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](https://github.com/ey-asu-rnd/SyntheticData/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

<p class="attribution">Developed by <a href="https://www.ey.com/ch">Ernst & Young Ltd.</a>, Zurich, Switzerland</p>

</div>

## What is SyntheticData?

SyntheticData is a configurable synthetic data generator that produces realistic, interconnected enterprise financial data. It generates General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, banking/KYC/AML transactions, OCEL 2.0 process mining data, audit workpapers, and ML-ready graph exports at scale.

The generator produces statistically accurate data based on empirical research from real-world general ledger patterns, ensuring that synthetic datasets exhibit the same characteristics as production data—including Benford's Law compliance, temporal patterns, and document flow integrity.

**New in v0.3.0:** ACFE-aligned fraud taxonomy, collusion modeling, industry-specific transactions (Manufacturing, Retail, Healthcare), and ML benchmarks.

**v0.2.x:** Privacy-preserving fingerprinting, accounting/audit standards (US GAAP, IFRS, ISA, SOX), streaming output API.

## Quick Links

| Section | Description |
|---------|-------------|
| [Getting Started](getting-started/README.md) | Installation, quick start guide, and demo mode |
| [User Guide](user-guide/README.md) | CLI reference, server API, desktop UI, Python wrapper |
| [Configuration](configuration/README.md) | Complete YAML schema and presets |
| [Architecture](architecture/README.md) | System design, data flow, resource management |
| [Crate Reference](crates/README.md) | Detailed crate documentation (15 crates) |
| [Advanced Topics](advanced/README.md) | Anomaly injection, graph export, fingerprinting, performance |
| [Use Cases](use-cases/README.md) | Fraud detection, audit, AML/KYC, compliance |

## Key Features

### Core Data Generation

| Feature | Description |
|---------|-------------|
| **Statistical Distributions** | Line item counts, amounts, and patterns based on empirical GL research |
| **Benford's Law Compliance** | First-digit distribution following Benford's Law with configurable fraud patterns |
| **Industry Presets** | Manufacturing, Retail, Financial Services, Healthcare, Technology |
| **Chart of Accounts** | Small (~100), Medium (~400), Large (~2500) account structures |
| **Temporal Patterns** | Month-end, quarter-end, year-end volume spikes with working hour modeling |
| **Regional Calendars** | Holiday calendars for US, DE, GB, CN, JP, IN with lunar calendar support |

### Enterprise Simulation

- **Master Data Management**: Vendors, customers, materials, fixed assets, employees with temporal validity
- **Document Flow Engine**: Complete P2P (Procure-to-Pay) and O2C (Order-to-Cash) processes with three-way matching
- **Intercompany Transactions**: IC matching, transfer pricing, consolidation eliminations
- **Balance Coherence**: Opening balances, running balance tracking, trial balance generation
- **Subledger Simulation**: AR, AP, Fixed Assets, Inventory with GL reconciliation
- **Currency & FX**: Realistic exchange rates (Ornstein-Uhlenbeck process), currency translation, CTA generation
- **Period Close Engine**: Monthly close, depreciation runs, accruals, year-end closing
- **Banking/KYC/AML**: Customer personas, KYC profiles, AML typologies (structuring, funnel, mule, layering, round-tripping)
- **Process Mining**: OCEL 2.0 event logs with object-centric relationships
- **Audit Simulation**: ISA-compliant engagements, workpapers, findings, risk assessments, professional judgments

### Fraud Patterns & Industry-Specific Features

- **ACFE-Aligned Fraud Taxonomy**: Asset Misappropriation, Corruption, Financial Statement Fraud calibrated to ACFE statistics
- **Collusion & Conspiracy Modeling**: Multi-party fraud networks with 9 ring types and role-based conspirators
- **Management Override**: Senior-level fraud with fraud triangle modeling (Pressure, Opportunity, Rationalization)
- **Red Flag Generation**: 40+ probabilistic fraud indicators with Bayesian probabilities
- **Industry-Specific Transactions**: Manufacturing (BOM, WIP), Retail (POS, shrinkage), Healthcare (ICD-10, claims)
- **Industry-Specific Anomalies**: Authentic fraud patterns per industry (upcoding, sweethearting, yield manipulation)

### Machine Learning & Analytics

- **Graph Export**: PyTorch Geometric, Neo4j, DGL, and RustGraph formats with train/val/test splits
- **Anomaly Injection**: 60+ fraud types, errors, process issues with full labeling
- **Data Quality Variations**: Missing values (MCAR, MAR, MNAR), format variations, duplicates, typos
- **Evaluation Framework**: Auto-tuning with configuration recommendations based on metric gaps
- **ACFE Benchmarks**: ML benchmarks calibrated to ACFE fraud statistics
- **Industry Benchmarks**: Pre-configured benchmarks for fraud detection by industry

### Privacy-Preserving Fingerprinting

- **Fingerprint Extraction**: Extract statistical properties from real data into `.dsf` files
- **Differential Privacy**: Laplace and Gaussian mechanisms with configurable epsilon budget
- **K-Anonymity**: Suppression of rare categorical values below configurable threshold
- **Privacy Audit Trail**: Complete logging of all privacy decisions and epsilon spent
- **Fidelity Evaluation**: Validate synthetic data matches original fingerprint (KS, Wasserstein, Benford MAD)
- **Gaussian Copula**: Preserve multivariate correlations during synthesis

### Production Features

- **REST & gRPC APIs**: Streaming generation with authentication and rate limiting
- **Desktop UI**: Cross-platform Tauri/SvelteKit application with 15+ configuration pages
- **Resource Guards**: Memory, disk, and CPU monitoring with graceful degradation
- **Graceful Degradation**: Progressive feature reduction under resource pressure (Normal→Reduced→Minimal→Emergency)
- **Deterministic Generation**: Seeded RNG (ChaCha8) for reproducible output
- **Python Wrapper**: Programmatic access with blueprints and config validation

## Performance

| Metric | Performance |
|--------|-------------|
| Single-threaded throughput | ~100,000+ entries/second |
| Parallel scaling | Linear with available cores |
| Memory efficiency | Streaming generation for large volumes |

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
| **Data Quality ML** | Train models to detect missing values, typos, duplicates |

## Quick Start

```bash
# Install from source
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release

# Run demo mode
./target/release/datasynth-data generate --demo --output ./output

# Or create a custom configuration
./target/release/datasynth-data init --industry manufacturing --complexity medium -o config.yaml
./target/release/datasynth-data generate --config config.yaml --output ./output
```

### Fingerprinting (New in v0.2.0)

```bash
# Extract fingerprint from real data with privacy protection
./target/release/datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --privacy-level standard

# Validate fingerprint integrity
./target/release/datasynth-data fingerprint validate ./fingerprint.dsf

# View fingerprint details
./target/release/datasynth-data fingerprint info ./fingerprint.dsf --detailed

# Evaluate synthetic data fidelity
./target/release/datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --threshold 0.8
```

### Python Wrapper

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
print(result.output_dir)
```

## Architecture

SyntheticData is organized as a Rust workspace with 15 modular crates:

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

## License

Copyright 2024-2026 Michael Ivertowski, Ernst & Young Ltd., Zurich, Switzerland

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/ey-asu-rnd/SyntheticData/blob/main/LICENSE) for details.

## Support

Commercial support, custom development, and enterprise licensing are available upon request. Please contact the author at [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com) for inquiries.

---

*SyntheticData is provided "as is" without warranty of any kind. It is intended for testing, development, and educational purposes. Generated data should not be used as a substitute for real financial records.*
