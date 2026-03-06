# Roadmap: Enterprise Simulation & ML Ground Truth

This roadmap documents the completed feature waves and outlines the direction for future development.

---

## Completed Features

### v0.1.0 — Core Generation

- **Statistical distributions**: Benford's Law compliance, log-normal mixtures, copulas
- **Industry presets**: Manufacturing, Retail, Financial Services, Healthcare, Technology
- **Chart of Accounts**: Small (~100), Medium (~400), Large (~2500) complexity levels
- **Temporal patterns**: Month-end/quarter-end volume spikes, business day calendars
- **Master data**: Vendors, customers, materials, fixed assets, employees
- **Document flows**: P2P (6 PO types, three-way match) and O2C (9 SO types, 6 delivery types, 7 invoice types)
- **Intercompany**: IC matching, transfer pricing, consolidation elimination entries
- **Subledgers**: AR (aging, dunning), AP (scheduling, discounts), FA (6 depreciation methods), Inventory (22 movement types, 4 valuation methods)
- **Currency & FX**: Ornstein-Uhlenbeck exchange rates, ASC 830 translation, CTA
- **Period close**: Monthly close engine, accruals, depreciation runs, year-end closing
- **Balance coherence**: Opening balances, running balance tracking, trial balance per period
- **Anomaly injection**: 60+ fraud types, error patterns, process issues with full labeling
- **Data quality**: Missing values (MCAR/MAR/MNAR), format variations, typos, duplicates
- **Graph export**: PyTorch Geometric, Neo4j, DGL with train/val/test splits
- **Internal controls**: COSO 2013 framework, SoD rules, 12 transaction + 6 entity controls
- **Resource guards**: Memory, disk, CPU monitoring with graceful degradation
- **REST/gRPC/WebSocket server** with authentication and rate limiting
- **Desktop UI**: Tauri/SvelteKit with configuration pages
- **Python wrapper**: Programmatic access with blueprints and config validation

### v0.2.0 — Privacy & Standards

- **Fingerprint extraction**: Statistical properties from real data into `.dsf` files
- **Differential privacy**: Laplace and Gaussian mechanisms with configurable epsilon
- **K-anonymity**: Suppression of rare categorical values
- **Fidelity evaluation**: KS, Wasserstein, Benford MAD metric comparison
- **Gaussian copula synthesis**: Preserve multivariate correlations
- **Accounting standards**: Revenue recognition (ASC 606/IFRS 15), Leases (ASC 842/IFRS 16), Fair Value (ASC 820/IFRS 13), Impairment (ASC 360/IAS 36)
- **Audit standards**: ISA compliance (34 standards), analytical procedures, confirmations, opinions, PCAOB mappings
- **SOX compliance**: Section 302/404 assessments, deficiency matrix, material weakness classification
- **Streaming output**: CSV, JSON, NDJSON, Parquet streaming sinks with backpressure
- **ERP output formats**: SAP S/4HANA (BKPF, BSEG, ACDOCA, LFA1, KNA1, MARA), Oracle EBS (GL_JE_HEADERS/LINES), NetSuite

### v0.3.0 — Fraud & Industry

- **ACFE-aligned fraud taxonomy**: Asset misappropriation, corruption, financial statement fraud calibrated to ACFE statistics
- **Collusion modeling**: 8 ring types, 6 conspirator roles, defection/escalation dynamics
- **Management override**: Fraud triangle modeling (pressure, opportunity, rationalization)
- **Red flag generation**: 40+ probabilistic indicators with Bayesian probabilities
- **Industry-specific generators**: Manufacturing (BOM, WIP, production orders), Retail (POS, shrinkage, loyalty), Healthcare (ICD-10, CPT, DRG, payer mix)
- **Industry benchmarks**: Pre-configured ML benchmarks per industry
- **Banking/KYC/AML**: Customer personas, KYC profiles, fraud typologies (structuring, funnel, layering, mule, round-tripping)
- **Process mining**: OCEL 2.0 event logs with P2P and O2C processes
- **Evaluation framework**: Auto-tuning with configuration recommendations from metric gaps
- **Vendor networks**: Tiered supply chains, quality scores, clusters
- **Customer segmentation**: Value segments, lifecycle stages, network positions
- **Cross-process links**: Entity graph, relationship strength, cross-process integration

### v0.5.0 — AI & Advanced Features

- **LLM-augmented generation**: Pluggable provider abstraction for realistic vendor names, descriptions, and anomaly explanations
- **Natural language configuration**: Generate YAML configs from descriptions
- **Diffusion model backend**: Statistical diffusion with configurable noise schedules
- **Hybrid generation**: Blend rule-based and diffusion outputs
- **Causal generation**: Structural Causal Models, do-calculus interventions, counterfactual generation
- **Federated fingerprinting**: Secure aggregation for distributed data sources
- **Synthetic data certificates**: Cryptographic proof of DP guarantees with HMAC-SHA256
- **Privacy-utility Pareto frontier**: Automated exploration of optimal epsilon values
- **Ecosystem integrations**: Airflow, dbt, MLflow, Spark pipeline integration

### v0.6.0–v0.8.x — Enterprise Process Chains & Localization

- **Source-to-Contract (S2C)**: Spend analysis, sourcing projects, supplier qualification, RFx, bids, evaluation, contracts, catalogs, scorecards
- **Hire-to-Retire (H2R)**: Payroll runs, time & attendance, expense reports, benefit enrollment
- **Manufacturing**: Production orders, BOM explosion, routing operations, WIP costing, quality inspections, cycle counting
- **Universal OCPM**: 12 process families with 101+ activities and 65+ object types
- **Country packs**: Pluggable JSON architecture with 10 built-in packs (US, DE, GB, FR, JP, CN, IN, IT, ES, CA)
- **French GAAP (PCG)**: Plan Comptable General 2024 with FEC export
- **German GAAP (HGB)**: SKR04 chart of accounts, Degressiv depreciation, GWG, GoBD export
- **Generalized `FrameworkAccounts`**: ~45 semantic accounts per framework

### v0.9.0 — Performance & Quality

- ~2x single-threaded throughput via cached temporal CDF, fast Decimal, SmallVec, parallel generation
- `ParallelGenerator` trait with deterministic seed splitting
- RustGraph property mapping for 51 entity types, 28 relationship types with edge constraints
- Comprehensive edge-case hardening across all crates
- VAT line splitting, multipayment behavior, account-class fingerprinting

### v0.10.0–v0.11.0 — Scenarios & Streaming

- **Counterfactual simulation engine**: Causal DAG with 17 financial nodes, 8 transfer functions, paired baseline/counterfactual generation
- **Scenario CLI**: `datasynth-data scenario {list, validate, generate, diff}`
- **GenerationSession**: Multi-period generation with checkpoint files and incremental append
- **Fraud scenario packs**: 5 built-in packs with deep-merge configuration
- **StreamPipeline**: Phase-aware streaming via `PhaseSink` trait
- **OCEL 2.0 enrichment**: Lifecycle state machines, correlation events, resource pool modeling

### v1.0.0 — Release

- **Process evolution & organizational events**: Acquisitions, divestitures, mergers, reorganizations
- **Disruption events**: Outage, migration, process change, recovery, regulatory disruption
- **Collusion ring generation**: Coordinated fraud networks with escalation dynamics
- **Bi-temporal vendor versioning**: Valid-time/transaction-time dimension version chains
- **Entity relationship graph**: Strength scores and cross-process links
- **Industry transaction factory**: Industry-specific GL accounts per vertical
- **Red flag indicators**: Risk indicators on P2P/O2C document chains
- **Counterfactual pairs**: (original, mutated) journal entry pairs for ML training
- Performance optimizations, numeric safety hardening, code quality improvements
- Complete crate metadata and documentation overhaul

---

## Process Coverage (v1.0.0)

| Process Chain | Coverage | Key Capabilities |
|---------------|----------|------------------|
| **S2P** (Source-to-Pay) | 95% | Full S2C + P2P + three-way match + bank reconciliation |
| **O2C** (Order-to-Cash) | 99% | Quote → Order → Delivery → Invoice → Receipt → Dunning |
| **R2R** (Record-to-Report) | 97% | GL → Trial Balance → Financial Statements → KPIs → Budgets |
| **A2R** (Acquire-to-Retire) | 80% | FA lifecycle with 6 depreciation methods + impairment |
| **INV** (Inventory) | 85% | 22 movement types, cycle counting, QA, obsolescence |
| **BANK** | 90% | KYC/AML + reconciliation + cash positioning + forecasting |
| **H2R** (Hire-to-Retire) | 75% | Payroll + time + expenses + benefits |
| **MFG** (Manufacturing) | 60% | Production orders + BOM + routing + WIP + quality |

---

## Cross-Process Integration

| Integration | Status |
|-------------|--------|
| S2P → Inventory | GR updates stock levels |
| Inventory → O2C | Delivery reduces stock |
| S2P/O2C → BANK | Payments reconciled against bank statements |
| All → R2R | JEs → Trial Balance → Financial Statements → Budget variance |
| H2R → S2P | Employee authorizations, expense → AP |
| S2P → A2R | Capital PO → Fixed Asset creation |
| MFG → S2P | Production → purchase requisition demand |
| MFG → INV | WIP → finished goods transfers, QA feedback |
| P2P ↔ O2C | Cross-process links via inventory (GR → Delivery) |

---

## Future Directions

With v1.0.0 delivering comprehensive enterprise coverage, future development focuses on:

- **Deeper manufacturing simulation**: Full MES integration, shop floor scheduling, predictive maintenance data
- **Advanced ESG**: Physical climate risk modeling, biodiversity metrics, Scope 3 Category 15 (investments)
- **Real-time streaming**: Event-driven generation with Kafka/Pulsar sink support
- **Multi-language NLP**: Multilingual LLM enrichment for non-English enterprise data
- **Federated generation**: Distributed generation across nodes with privacy-preserving coordination
- **Additional country packs**: LATAM, MENA, SEA region packs with local tax and regulatory compliance

---

## Guiding Principles

- **Enterprise realism**: Simulate multi-entity, multi-region, multi-currency operations with coherent process flows
- **ML ground truth**: Capture true labels and causal factors for supervised learning, explainability, and evaluation
- **Scalability**: Handle large volumes with stable performance and reproducible results
- **Backward compatibility**: New features are additive; existing configs continue to work

---

## Contributing

We welcome contributions to any area. See [Contributing Guidelines](../contributing/README.md) for details.

To propose new features:
1. Open a GitHub issue with the `enhancement` label
2. Describe the use case and expected behavior
3. Reference relevant roadmap items if applicable

---

## Feedback

Priorities are influenced by user feedback. Please share your use cases and requirements:

- GitHub Issues: Feature requests and bug reports
- Email: [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com)

## See Also

- [Process Chains](../architecture/process-chains.md) — Process chain architecture and coverage matrix
- [S2P Spec](../../specs/s2p-process-chain-spec.md) — Source-to-Contract specification
- [Process Chain Gaps](../../specs/enterprise-process-chain-gaps.md) — Detailed gap analysis
