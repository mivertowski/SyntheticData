# SyntheticData

[![Crates.io](https://img.shields.io/crates/v/datasynth-core.svg)](https://crates.io/crates/datasynth-core)
[![Documentation](https://docs.rs/datasynth-core/badge.svg)](https://docs.rs/datasynth-core)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml/badge.svg)](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml)

A high-performance, configurable synthetic data generator for enterprise financial simulation. SyntheticData produces realistic, interconnected General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, banking/KYC/AML transactions, OCEL 2.0 process mining data, ML-ready graph exports, and complete enterprise process chains (S2C sourcing, HR/payroll, manufacturing, financial reporting, tax accounting, treasury & cash management, project accounting, ESG/sustainability) at scale.

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
    - [LLM-Augmented Generation](#llm-augmented-generation)
    - [Diffusion Model Integration](#diffusion-model-integration)
    - [Causal \& Counterfactual Generation](#causal--counterfactual-generation)
    - [Ecosystem Integrations](#ecosystem-integrations)
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
  - [Output Viewer](#output-viewer)
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
| **Country Packs** | Pluggable JSON country packs (US, DE, GB built-in) with holidays, names, tax, address, phone, payroll — extensible via external directory |

### Enterprise Simulation

- **Master Data Management**: Vendors, customers, materials, fixed assets, employees with temporal validity
- **Document Flow Engine**: Complete P2P (Procure-to-Pay) and O2C (Order-to-Cash) processes
- **Source-to-Contract (S2C)**: Spend analysis → sourcing projects → supplier qualification → RFx → bids → evaluation → contracts → catalogs → scorecards
- **Hire-to-Retire (H2R)**: Payroll runs with tax/deduction calculations, time & attendance tracking, expense report management
- **Manufacturing**: Production orders with BOM explosion, routing operations, WIP costing, quality inspections, cycle counting
- **Financial Reporting**: Balance sheet, income statement, cash flow statement, changes in equity with BS equation enforcement
- **Sales Quotes**: Quote-to-order pipeline with win rate modeling and pricing negotiation
- **Management KPIs & Budgets**: Financial ratio computation (liquidity, profitability, efficiency, leverage) and budget variance analysis
- **Revenue Recognition**: ASC 606/IFRS 15 contract generation with performance obligations and standalone selling price allocation
- **Impairment Testing**: Asset impairment workflow with fair value estimation and journal entry generation
- **Intercompany Transactions**: IC matching, transfer pricing, consolidation eliminations
- **Balance Coherence**: Opening balances, running balance tracking, trial balance generation
- **Subledger Simulation**: AR, AP, Fixed Assets, Inventory with GL reconciliation
- **Currency & FX**: Realistic exchange rates, currency translation, CTA generation
- **Period Close Engine**: Monthly close, depreciation runs, accruals, year-end closing
- **Bank Reconciliation**: Automated statement matching, outstanding checks, deposits in transit, net difference validation
- **Country Pack Architecture**: Pluggable JSON-based country configuration with `_default.json` → country → override layering; built-in US/DE/GB packs; external pack directory for custom/commercial locales
- **Banking/KYC/AML**: Customer personas, KYC profiles, AML typologies (structuring, funnel, mule, layering)
- **Process Mining**: OCEL 2.0 and XES 2.0 event logs with object-centric relationships across 12 process families
  - OCEL 2.0 JSON/XML export for object-centric process mining
  - XES 2.0 XML export for ProM, Celonis, Disco, pm4py compatibility
  - 101+ activity types across 12 process families: P2P, O2C, R2R/A2R, S2C, H2R, MFG, BANK, AUDIT, Tax, Treasury, Project Accounting, ESG
  - 65+ object types with lifecycle states and relationships
  - 10 OCPM generators: S2C (sourcing), H2R (payroll/time/expense), MFG (production/quality), BANK (customer/transactions), AUDIT (engagement lifecycle), Bank Recon (statement matching), Tax (returns/lines), Treasury (positions/forecasts/hedges), Project Accounting (projects/costs/milestones), ESG (emissions/disclosures)
  - Three variant types per generator: HappyPath (75%), ExceptionPath (20%), ErrorPath (5%)
- **Audit Simulation**: ISA-compliant engagements, workpapers, findings, risk assessments
- **COSO 2013 Framework**: Full internal control framework with 5 components, 17 principles, and maturity levels
- **Accounting Standards**: US GAAP, IFRS, and French GAAP (PCG) support with ASC 606/IFRS 15 (revenue), ASC 842/IFRS 16 (leases with 5 bright-line tests), ASC 820/IFRS 13 (fair value), ASC 360/IAS 36 (impairment)
- **Audit Standards**: ISA (34 standards), PCAOB (19+ standards), SOX 302/404 compliance with deficiency classification
- **Tax Accounting**: Tax jurisdictions (Federal/State/Local), tax codes with effective dates, tax line decoration on documents, VAT/GST/sales tax returns, ASC 740/IAS 12 provisions with deferred tax, FIN 48/IFRIC 23 uncertain positions, cross-border withholding with treaty benefits
- **Treasury & Cash Management**: Daily cash positions, probability-weighted cash forecasts, cash pooling (physical/notional/zero-balance), hedging instruments (FX forwards, IR swaps, options) with ASC 815/IFRS 9 effectiveness, debt instruments with covenants & amortization, bank guarantees, intercompany netting
- **Project Accounting**: Project WBS hierarchies, cost lines (labor/material/subcontractor/overhead), percentage-of-completion revenue recognition, earned value management (BCWS/BCWP/ACWP/SPI/CPI/EAC), milestones, change orders, retainage
- **ESG / Sustainability**: GHG Protocol Scope 1/2/3 emissions, energy consumption with renewable tracking, water/waste management, workforce diversity & pay equity, safety incidents (TRIR/LTIR/DART), governance metrics, GRI/SASB/TCFD disclosures, supplier ESG assessments, climate scenario analysis

### Interconnectivity & Relationships

- **Multi-Tier Vendor Networks**: Tier1/Tier2/Tier3 supply chain modeling with parent-child hierarchies
- **Vendor Clusters**: ReliableStrategic, StandardOperational, Transactional, Problematic behavioral segmentation
- **Customer Value Segmentation**: Enterprise/MidMarket/SMB/Consumer with Pareto-like revenue distribution
- **Customer Lifecycle**: Prospect, New, Growth, Mature, AtRisk, Churned, WonBack stages
- **Relationship Strength**: Composite scoring from volume, count, duration, recency, and mutual connections
- **Cross-Process Links**: P2P↔O2C linkage via inventory (GoodsReceipt connects to Delivery)
- **Entity Graphs**: 16 entity types, 26 relationship types with graph metrics (connectivity, clustering, power law)

### Pattern & Process Drift

- **Organizational Events**: Acquisitions (volume multipliers, integration errors), divestitures, mergers, reorganizations
- **Process Evolution**: S-curve automation rollout, workflow changes, policy updates, control enhancements
- **Technology Transitions**: ERP migrations with phased rollout (parallel run, cutover, stabilization, hypercare)
- **Behavioral Drift**: Vendor payment term extensions, customer payment delays, employee learning curves
- **Market Drift**: Economic cycles (sinusoidal, asymmetric, mean-reverting), commodity price shocks, recession modeling
- **Regulatory Events**: Accounting standard adoptions, tax rate changes, compliance requirement impacts
- **Drift Detection Ground Truth**: Labeled drift events with magnitude and detection difficulty for ML training

### Fraud Patterns & Industry-Specific Features

- **ACFE-Aligned Fraud Taxonomy**: Fraud classification based on ACFE Report to the Nations statistics
  - Asset Misappropriation (86% of cases): Cash fraud, billing schemes, expense reimbursement, payroll fraud
  - Corruption (33% of cases): Conflicts of interest, bribery, kickbacks, bid rigging
  - Financial Statement Fraud (10% of cases): Revenue manipulation, expense timing, improper disclosures
- **Collusion & Conspiracy Modeling**: Multi-party fraud networks with coordinated schemes
  - 9 ring types (EmployeePair, DepartmentRing, EmployeeVendor, VendorRing, etc.)
  - Role-based conspirators (Initiator, Executor, Approver, Concealer, Lookout, Beneficiary)
  - Defection and escalation modeling based on detection risk
- **Management Override Patterns**: Senior-level fraud with override techniques and fraud triangle modeling
- **Red Flag Generation**: 40+ probabilistic fraud indicators with calibrated Bayesian probabilities
- **Industry-Specific Transactions**: Authentic transaction modeling per industry
  - Manufacturing: Work orders, BOM, routings, production variances, WIP tracking
  - Retail: POS sales, returns, inventory, promotions, shrinkage tracking
  - Healthcare: Revenue cycle, charge capture, claims, ICD-10/CPT/DRG coding
  - Technology: License revenue, subscription billing, R&D capitalization
  - Financial Services: Loan origination, trading, customer deposits
  - Professional Services: Time & billing, engagement management, trust accounts
- **Industry-Specific Anomalies**: Authentic fraud patterns per industry
  - Manufacturing: Yield manipulation, phantom production, obsolete inventory concealment
  - Retail: Sweethearting, skimming, refund fraud, receiving fraud
  - Healthcare: Upcoding, unbundling, phantom billing, physician kickbacks
- **ACFE-Calibrated Benchmarks**: ML evaluation benchmarks aligned with ACFE statistics

### Machine Learning & Analytics

- **Graph Export**: PyTorch Geometric, Neo4j, DGL, RustGraph, and RustGraph Hypergraph formats with train/val/test splits
- **Multi-Layer Hypergraph**: 3-layer hypergraph (Governance, Process Events, Accounting Network) spanning all 8 process families with OCPM events as hyperedges, 24 entity type codes (100-400), and cross-process edge linking
- **Anomaly Injection**: 60+ fraud types, errors, process issues with full labeling
- **Data Quality Variations**: Missing values, format variations, duplicates, typos
- **Relationship Generation**: Configurable entity relationships with cardinality rules
- **Industry Benchmarks**: Pre-configured benchmarks for fraud detection by industry

### Privacy-Preserving Fingerprinting

- **Fingerprint Extraction**: Extract statistical properties from real data into `.dsf` files
- **Differential Privacy**: Laplace mechanism with configurable epsilon budget
- **Formal DP Composition**: Rényi DP and zCDP accounting with tighter composition bounds
- **K-Anonymity**: Suppression of rare categorical values
- **Custom Privacy Levels**: Configurable (ε, δ) tuples with preset levels (minimal, standard, high, maximum)
- **Privacy Budget Management**: Global budget tracking across multiple extraction runs
- **Privacy Audit Trail**: Complete logging of all privacy decisions with composition metadata
- **Fidelity Evaluation**: Wasserstein-1, Jensen-Shannon divergence, and KS statistics per column
- **Privacy Evaluation**: Membership inference attack (MIA) testing, linkage attack assessment, NIST SP 800-226 alignment, SynQP matrix
- **Federated Fingerprinting**: Extract partial fingerprints from distributed data sources and aggregate without centralizing raw data (weighted average, median, trimmed mean)
- **Synthetic Data Certificates**: Cryptographic attestation of DP guarantees and quality metrics with HMAC-SHA256 signing and verification
- **Pareto Privacy-Utility Frontier**: Explore and navigate the optimal tradeoff between privacy (epsilon) and data utility

### LLM-Augmented Generation

- **Provider Abstraction**: Pluggable `LlmProvider` trait with mock (deterministic) and HTTP (OpenAI-compatible) backends
- **Metadata Enrichment**: LLM-generated vendor names, transaction descriptions, memo fields, and anomaly explanations
- **Natural Language Configuration**: Generate YAML configs from plain English (e.g., "1 year of retail data for a German company")
- **Response Caching**: In-memory LRU cache keyed by prompt hash for deduplication
- **Graceful Fallback**: All enrichment falls back to template-based generation when LLM is disabled or unavailable

### Diffusion Model Integration

- **Backend Trait**: Extensible `DiffusionBackend` with forward (noise) and reverse (denoise) processes
- **Noise Schedules**: Linear, cosine, and sigmoid schedules with precomputed alpha/beta values
- **Statistical Diffusion**: Pure-Rust Langevin-inspired reverse process guided by fingerprint statistics (no ML framework dependency)
- **Hybrid Generation**: Blend rule-based and diffusion outputs via interpolation, selection, or per-column ensemble strategies
- **Training Pipeline**: Fit diffusion models from column statistics, persist as JSON, evaluate with mean/std/correlation error metrics

### Causal & Counterfactual Generation

- **Causal Graphs**: Directed acyclic graphs with linear, threshold, polynomial, and logistic mechanisms
- **Structural Causal Models**: Generate samples respecting causal structure via topological traversal
- **do-Calculus Interventions**: Fix variables to specific values and measure average treatment effects with confidence intervals
- **Counterfactual Generation**: Abduction-action-prediction framework for "what-if" scenario analysis
- **Causal Validation**: Verify edge correlations, non-edge weakness, and topological consistency
- **Built-in Templates**: Pre-configured fraud detection and revenue cycle causal models

### Ecosystem Integrations

- **Apache Airflow**: `DataSynthOperator`, `DataSynthSensor`, and `DataSynthValidateOperator` for DAG-based orchestration
- **dbt**: Source YAML generation, seed export, and project scaffolding from DataSynth output
- **MLflow**: Track generation runs as experiments with parameters, metrics, and artifact logging
- **Apache Spark**: Read DataSynth output as Spark DataFrames with schema inference and temp view registration

### Production Features

- **REST & gRPC APIs**: Streaming generation with Argon2id authentication and rate limiting
- **JWT/OIDC Authentication**: RS256 JWT validation with Keycloak, Auth0, and Entra ID support (feature-gated)
- **Role-Based Access Control**: Admin/Operator/Viewer roles with 7 permission types and structured JSON audit logging
- **gRPC Auth Interceptor**: Bearer token validation for gRPC endpoints with API versioning headers
- **Quality Gates**: Configurable pass/fail thresholds (strict/default/lenient) with 8 metrics and CLI enforcement
- **Plugin SDK**: Extensible `GeneratorPlugin`, `SinkPlugin`, `TransformPlugin` traits with thread-safe registry
- **Webhook Notifications**: Fire-and-forget event dispatch for RunStarted, RunCompleted, RunFailed, GateViolation
- **EU AI Act Compliance**: Article 50 synthetic content marking and Article 10 data governance reports
- **Compliance Documentation**: SOC 2 Type II readiness, ISO 27001 Annex A alignment, NIST AI RMF, GDPR templates
- **Async Job Queue**: Submit/poll/cancel pattern for long-running generation jobs
- **Security Hardening**: Security headers, request validation, request ID propagation, timing-safe auth
- **TLS Support**: Native rustls TLS or reverse proxy (nginx/envoy) with documented configuration
- **OpenTelemetry**: Feature-gated OTEL integration with OTLP traces and Prometheus metrics
- **Structured Logging**: JSON-formatted logs with request IDs, method, path, status, and latency
- **Docker & Compose**: Multi-stage distroless containers, local dev stack with Prometheus + Grafana
- **Kubernetes Helm Chart**: Production-ready chart with HPA, PDB, optional Redis subchart, and Prometheus ServiceMonitor
- **CI/CD Pipeline**: 7-job GitHub Actions (fmt, clippy, cross-platform test, MSRV, security, coverage, benchmarks)
- **Release Automation**: Binary builds for 5 platforms, GHCR container publishing, Trivy scanning
- **Data Lineage & Provenance**: Per-file checksums, lineage graph, W3C PROV-JSON export, CLI `verify` command
- **Distributed Rate Limiting**: Redis-backed sliding window rate limiting for multi-instance deployments
- **Streaming Output API**: Async generation with backpressure handling (Block, DropOldest, DropNewest, Buffer)
- **Rate Limiting**: Token bucket rate limiter for controlled generation throughput
- **Load Testing**: k6 scripts for health, bulk generation, WebSocket, job queue, and soak testing
- **Temporal Attributes**: Bi-temporal data support (valid time + transaction time) with version chains
- **Desktop UI**: Cross-platform Tauri/SvelteKit application
- **Resource Guards**: Memory, disk, and CPU monitoring with graceful degradation
- **Panic-Free Library Crates**: `#![deny(clippy::unwrap_used)]` enforced across all library crates
- **Fuzzing**: cargo-fuzz targets for config parsing, fingerprint loading, and validation
- **Evaluation Framework**: Auto-tuning with quality gate enforcement and configuration recommendations
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
datasynth-ocpm         Object-Centric Process Mining (OCEL 2.0, XES 2.0, 8 process families)
datasynth-fingerprint  Privacy-preserving fingerprint extraction and synthesis
datasynth-standards    Accounting/audit standards (IFRS, US GAAP, French GAAP, ISA, SOX, PCAOB)
    │
datasynth-graph        Graph/network export (PyTorch Geometric, Neo4j, DGL, RustGraph Multi-Layer Hypergraph)
datasynth-eval         Evaluation framework with auto-tuning
    │
datasynth-config       Configuration schema, validation, industry presets
    │
datasynth-core         Domain models, traits, distributions, resource guards
    │
datasynth-output       Output sinks (CSV, JSON, NDJSON, Parquet/Zstd) with streaming support
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
| [`datasynth-standards`](https://crates.io/crates/datasynth-standards) | Accounting/audit standards (IFRS, US GAAP, French GAAP, ISA, SOX, PCAOB) |
| [`datasynth-graph`](https://crates.io/crates/datasynth-graph) | Graph/network export |
| [`datasynth-eval`](https://crates.io/crates/datasynth-eval) | Evaluation framework |
| [`datasynth-runtime`](https://crates.io/crates/datasynth-runtime) | Orchestration layer |
| [`datasynth-cli`](https://crates.io/crates/datasynth-cli) | Command-line interface |
| [`datasynth-server`](https://crates.io/crates/datasynth-server) | REST/gRPC server |

### Requirements

- Rust 1.88 or later
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
    - rustgraph_hypergraph    # 3-layer hypergraph JSONL for RustGraph
  hypergraph:
    enabled: true
    max_nodes: 50000
    aggregation_strategy: pool_by_counterparty

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

# Enterprise Process Chain Extensions (v0.6.0)
source_to_pay:
  enabled: true
  sourcing:
    projects_per_year: 20
  qualification:
    pass_rate: 0.80
  rfx:
    invited_vendors_min: 3
    invited_vendors_max: 8
  contracts:
    duration_months_min: 12
    duration_months_max: 36
  scorecards:
    frequency: quarterly

financial_reporting:
  enabled: true
  generate_balance_sheet: true
  generate_income_statement: true
  generate_cash_flow: true
  management_kpis:
    enabled: true
    frequency: monthly
  budgets:
    enabled: true
    revenue_growth_rate: 0.05

hr:
  enabled: true
  payroll:
    enabled: true
    pay_frequency: monthly
  time_attendance:
    enabled: true
    overtime_rate: 0.10
  expenses:
    enabled: true
    submission_rate: 0.30

manufacturing:
  enabled: true
  production_orders:
    orders_per_month: 50
    yield_rate: 0.97
  costing:
    labor_rate_per_hour: 35.00
    overhead_rate: 1.50

sales_quotes:
  enabled: true
  quotes_per_month: 30
  win_rate: 0.35
  validity_days: 30

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

ocpm:
  enabled: true
  generate_lifecycle_events: true
  compute_variants: true
  output:
    ocel_json: true               # OCEL 2.0 JSON format
    ocel_xml: false               # OCEL 2.0 XML format
    xes: true                     # XES 2.0 for ProM/Celonis/Disco
    xes_include_lifecycle: true   # Include start/complete transitions
    xes_include_resources: true   # Include resource attributes
    export_reference_models: true # Export P2P/O2C/R2R reference models

llm:
  enabled: true
  provider: mock                    # mock, openai, anthropic, custom
  enrichment:
    vendor_names: true
    transaction_descriptions: true
    anomaly_explanations: true

diffusion:
  enabled: true
  n_steps: 100
  schedule: cosine                  # linear, cosine, sigmoid
  sample_size: 100

causal:
  enabled: true
  template: fraud_detection         # fraud_detection, revenue_cycle
  sample_size: 500
  validate: true

# Domain Extensions (v0.7.0)
tax:
  enabled: true
  jurisdictions:
    countries: [US, DE, GB]
  vat_gst:
    standard_rate: 0.19
  provisions:
    statutory_rate: 0.21

treasury:
  enabled: true
  cash_positioning:
    enabled: true
  hedging:
    enabled: true
  debt:
    enabled: true

project_accounting:
  enabled: true
  project_count: 10
  revenue_recognition:
    method: percentage_of_completion
  earned_value:
    enabled: true

esg:
  enabled: true
  environmental:
    scope1:
      enabled: true
    scope2:
      enabled: true
    scope3:
      enabled: true
  social:
    diversity:
      enabled: true
    safety:
      enabled: true
  reporting:
    frameworks: [gri, sasb, tcfd]

output:
  format: csv                       # csv, json, parquet
  compression: none                 # none, gzip, zstd (parquet uses zstd by default)
```

See the [Configuration Guide](docs/configuration.md) for complete documentation.

---

## Output Structure

```
output/
├── master_data/          Vendors, customers, materials, assets, employees
├── transactions/         Journal entries, purchase orders, invoices, payments
├── sourcing/             S2C sourcing pipeline outputs
│   ├── sourcing_projects.csv
│   ├── supplier_qualifications.csv
│   ├── rfx_events.csv
│   ├── supplier_bids.csv
│   ├── bid_evaluations.csv
│   ├── procurement_contracts.csv
│   ├── catalog_items.csv
│   └── supplier_scorecards.csv
├── subledgers/           AR, AP, FA, inventory detail records
├── hr/                   HR & payroll outputs
│   ├── payroll_runs.csv
│   ├── payslips.csv
│   ├── time_entries.csv
│   └── expense_reports.csv
├── manufacturing/        Production & quality outputs
│   ├── production_orders.csv
│   ├── routing_operations.csv
│   ├── quality_inspection_lots.csv
│   └── cycle_count_records.csv
├── period_close/         Trial balances, accruals, closing entries
├── financial_reporting/  Financial statements & management reporting
│   ├── balance_sheet.csv
│   ├── income_statement.csv
│   ├── cash_flow_statement.csv
│   ├── changes_in_equity.csv
│   ├── financial_kpis.csv
│   └── budget_variance.csv
├── sales/                Sales pipeline outputs
│   ├── sales_quotes.csv
│   └── sales_quote_items.csv
├── consolidation/        Eliminations, currency translation
├── fx/                   Exchange rates, CTA adjustments
├── banking/              KYC profiles, bank transactions, AML typology labels
│   ├── bank_statement_lines.csv
│   ├── bank_reconciliations.csv
│   └── reconciling_items.csv
├── process_mining/       Event logs and process models
│   ├── event_log.json    OCEL 2.0 JSON format
│   ├── event_log.xes     XES 2.0 XML format (for ProM, Celonis, Disco)
│   ├── process_variants/ Discovered process variants
│   └── reference_models/ Canonical P2P, O2C, R2R process models
├── audit/                Engagements, workpapers, findings, risk assessments
├── graphs/               PyTorch Geometric, Neo4j, DGL, RustGraph exports
│   └── hypergraph/       Multi-layer hypergraph (nodes.jsonl, edges.jsonl, hyperedges.jsonl)
├── labels/               Anomaly, fraud, and data quality labels for ML
├── tax/                 Tax accounting outputs
│   ├── tax_jurisdictions.csv
│   ├── tax_codes.csv
│   ├── tax_lines.csv
│   ├── tax_returns.csv
│   ├── tax_provisions.csv
│   ├── uncertain_tax_positions.csv
│   └── withholding_records.csv
├── treasury/            Treasury & cash management outputs
│   ├── cash_positions.csv
│   ├── cash_forecasts.csv
│   ├── hedging_instruments.csv
│   ├── debt_instruments.csv
│   ├── netting_runs.csv
│   └── bank_guarantees.csv
├── project_accounting/  Project accounting outputs
│   ├── projects.csv
│   ├── wbs_elements.csv
│   ├── project_cost_lines.csv
│   ├── project_revenue.csv
│   ├── earned_value_metrics.csv
│   └── change_orders.csv
├── esg/                 ESG / sustainability outputs
│   ├── emission_records.csv
│   ├── energy_consumption.csv
│   ├── workforce_diversity_metrics.csv
│   ├── safety_metrics.csv
│   ├── esg_disclosures.csv
│   └── supplier_esg_assessments.csv
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
| **Process Mining** | OCEL 2.0 and XES 2.0 event logs for process discovery and conformance checking |
| **Conformance Checking** | Reference process models (P2P, O2C, R2R) for process validation |
| **ERP Testing** | Load testing with realistic transaction volumes |
| **Procurement Analytics** | Source-to-contract pipeline with spend analysis, RFx, bids, and supplier scorecards |
| **HR & Payroll Testing** | Payroll runs, time tracking, expense management with policy compliance |
| **Manufacturing Simulation** | Production orders, BOM explosion, WIP costing, quality inspections |
| **Financial Reporting** | Balance sheet, income statement, cash flow, KPIs, and budget variance |
| **Bank Reconciliation** | Statement matching, outstanding items, net difference validation |
| **SOX Compliance** | Test internal control monitoring systems |
| **COSO Framework** | COSO 2013 control mapping with 5 components, 17 principles, maturity levels |
| **Standards Compliance** | IFRS/US GAAP revenue recognition, lease accounting, fair value, impairment testing |
| **Audit Standards** | ISA/PCAOB procedure mapping, analytical procedures, confirmations, audit opinions |
| **Data Quality ML** | Train models to detect missing values, typos, duplicates |
| **RustGraph Integration** | Stream data directly to RustAssureTwin knowledge graphs |
| **Hypergraph Analytics** | 3-layer hypergraph export (Governance, Process, Accounting) for multi-relational GNN models |
| **Causal Analysis** | Generate interventional and counterfactual datasets for causal ML research |
| **Tax Compliance Testing** | Tax return filing, ASC 740/IAS 12 provisions, withholding tax, uncertain positions |
| **Treasury Operations** | Cash positioning, forecasting, hedging effectiveness, debt covenant monitoring |
| **Project Cost Control** | WBS-based costing, earned value management, change order tracking, PoC revenue |
| **ESG Reporting** | GHG Scope 1/2/3 emissions, diversity metrics, GRI/SASB/TCFD disclosures |
| **LLM Training Data** | LLM-enriched metadata with realistic vendor names, descriptions, and explanations |
| **Pipeline Orchestration** | Airflow operators, dbt sources, MLflow tracking, Spark DataFrames |

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
# Start REST + gRPC server
cargo run -p datasynth-server -- --rest-port 3000 --grpc-port 50051 --worker-threads 4

# With API key authentication
cargo run -p datasynth-server -- --api-keys "key1,key2"

# With JWT/OIDC authentication (requires jwt feature)
cargo run -p datasynth-server --features jwt -- \
  --jwt-issuer "https://auth.example.com" \
  --jwt-audience "datasynth-api" \
  --jwt-public-key /path/to/public.pem

# With RBAC and audit logging
cargo run -p datasynth-server -- --api-keys "key1" --rbac-enabled --audit-log

# With TLS (requires tls feature)
cargo run -p datasynth-server --features tls -- --tls-cert cert.pem --tls-key key.pem
```

```bash
# API endpoints
curl http://localhost:3000/health              # Health check
curl http://localhost:3000/ready               # Readiness probe (config + memory + disk)
curl http://localhost:3000/metrics             # Prometheus metrics
curl -H "Authorization: Bearer <key>" http://localhost:3000/api/config
curl -H "Authorization: Bearer <key>" -X POST http://localhost:3000/api/stream/start
```

WebSocket streaming available at `ws://localhost:3000/ws/events`.

### Docker

```bash
# Build and run the server
docker build -t datasynth:latest .
docker run -p 50051:50051 -p 3000:3000 datasynth:latest

# Or use Docker Compose for full stack (server + Prometheus + Grafana)
docker compose up -d
# REST API: http://localhost:3000 | gRPC: localhost:50051
# Prometheus: http://localhost:9090 | Grafana: http://localhost:3001
```

See the [Deployment Guide](deploy/README.md) for Docker, SystemD, and reverse proxy setup.

---

## Desktop UI

```bash
cd crates/datasynth-ui
npm install
npm run tauri dev
```

The desktop application provides visual configuration, real-time streaming, and preset management.

**Features:**
- **40+ config pages** with form controls for every generation parameter
- **Info cards** on feature pages explaining capabilities before enabling
- **Sidebar navigation** with collapsible sections and scroll indicator for 10 section groups
- **Web preview mode** — run `npm run dev` for config editing without Tauri; dashboard requires `npm run tauri dev`
- **Visual regression testing** — 56 Playwright screenshot baselines for UI consistency

---

## Output Viewer

A separate **web-based Output Viewer** lets you explore generated data in the browser: journal entries (including French FEC), master data, fraud & anomaly labels, trial balance, general/auxiliary ledgers, subledgers, and an interactive graph (in-memory from JEs or Neo4j).

```bash
cd datasynth-output-viewer
npm install
npm run dev
```

Use **Load data** in the app to point at your output directory (path or URL). To bundle a local output folder into the app for dev, run `OUTPUT_DIR=../output npm run load-data` then `npm run dev`.

See **[datasynth-output-viewer/README.md](datasynth-output-viewer/README.md)** for full documentation (data source, scripts, graph view, production build). Screenshots are in [docs/src/datasynth-output-viewer](docs/src/datasynth-output-viewer).

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

A Python wrapper (v1.3.0) is available for programmatic access:

```bash
cd python
pip install -e ".[all]"    # Includes pandas, polars, jupyter, streaming
```

```python
from datasynth_py import DataSynth, AsyncDataSynth
from datasynth_py import to_pandas, to_polars, list_tables
from datasynth_py.config import blueprints

# Basic generation
config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
print(result.output_dir)

# DataFrame loading
tables = list_tables(result)          # ['journal_entries', 'vendors', ...]
df = to_pandas(result, "journal_entries")
pl_df = to_polars(result, "vendors")

# Async generation
async with AsyncDataSynth() as synth:
    result = await synth.generate(config=config)

# Fingerprint operations
synth.fingerprint.extract("./real_data/", "./fingerprint.dsf", privacy_level="standard")
report = synth.fingerprint.evaluate("./fingerprint.dsf", "./synthetic/")
print(f"Fidelity score: {report.overall_score}")
```

Optional dependencies: `[pandas]`, `[polars]`, `[jupyter]`, `[streaming]`, `[airflow]`, `[dbt]`, `[mlflow]`, `[spark]`, `[all]`.

**Ecosystem Integrations:**

```python
from datasynth_py.config import blueprints

# LLM-enriched generation
config = blueprints.with_llm_enrichment(provider="mock")

# Diffusion-enhanced generation
config = blueprints.with_diffusion(schedule="cosine", hybrid_weight=0.3)

# Causal data generation
config = blueprints.with_causal(template="fraud_detection")

# Airflow operator
from datasynth_py.integrations.airflow import DataSynthOperator

# dbt integration
from datasynth_py.integrations.dbt import DbtSourceGenerator

# MLflow tracking
from datasynth_py.integrations.mlflow_tracker import DataSynthMlflowTracker

# Spark connector
from datasynth_py.integrations.spark import DataSynthSparkReader
```

See the [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md) for complete documentation.

---

## Documentation

- [Configuration Guide](docs/configuration.md)
- [API Reference](docs/api.md)
- [Architecture Overview](docs/architecture.md)
- [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md)
- [Compliance & Regulatory](docs/src/compliance/README.md)
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
