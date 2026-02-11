# Roadmap: Enterprise Simulation & ML Ground Truth

This roadmap outlines completed features, planned enhancements, and the wave-based expansion strategy for enterprise process chain coverage.

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
- **Desktop UI**: Tauri/SvelteKit with 15+ configuration pages
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

- **LLM-augmented generation**: Pluggable provider abstraction (Mock, OpenAI, Anthropic) for realistic vendor names, descriptions, memo fields, and anomaly explanations
- **Natural language configuration**: Generate YAML configs from descriptions
- **Diffusion model backend**: Statistical diffusion with configurable noise schedules (linear, cosine, sigmoid) for learned distribution capture
- **Hybrid generation**: Blend rule-based and diffusion outputs
- **Causal generation**: Structural Causal Models (SCMs), do-calculus interventions, counterfactual generation
- **Built-in causal templates**: `fraud_detection` and `revenue_cycle` causal graphs
- **Federated fingerprinting**: Secure aggregation (weighted average, median, trimmed mean) for distributed data sources
- **Synthetic data certificates**: Cryptographic proof of DP guarantees with HMAC-SHA256 signing
- **Privacy-utility Pareto frontier**: Automated exploration of optimal epsilon values
- **Ecosystem integrations**: Airflow, dbt, MLflow, Spark pipeline integration

---

## Planned Enhancements

### Wave 1 — Foundation (enables everything else)

These items close the most critical gaps and unblock downstream work.

| Item | Chain | Description | Dependencies |
|------|-------|-------------|-------------|
| **S2C completion** | S2P | Source-to-Contract: spend analysis, RFx, bid evaluation, contract management, catalog items, supplier scorecards | Extends existing P2P |
| **Bank reconciliation** | BANK | Bank statement lines, auto-matching, reconciliation breaks, clearing | Validates all payment chains |
| **Financial statement generator** | R2R | Balance sheet, income statement, cash flow statement from trial balance | Consumes all JE data |

**Impact:** S2C creates a closed-loop procurement model. Bank reconciliation validates payment integrity across S2P and O2C. Financial statements provide the final reporting layer for R2R.

### Wave 2 — Core Process Chains

| Item | Chain | Description | Dependencies |
|------|-------|-------------|-------------|
| **Payroll & time management** | H2R | Payroll runs, time entries, overtime, benefits, tax withholding | Employee master data |
| **Revenue recognition generator** | O2C→R2R | Wire `CustomerContract` + `PerformanceObligation` models to SO/Invoice data | Existing ASC 606 models |
| **Impairment generator** | A2R→R2R | Wire existing `ImpairmentTest` model to FA generator with JE output | Existing ASC 360 models |

**Impact:** Payroll is the largest H2R gap and enables SoD analysis for personnel. Revenue recognition and impairment generators wire existing standards models into the generation pipeline.

### Wave 3 — Operational Depth

| Item | Chain | Description | Dependencies |
|------|-------|-------------|-------------|
| **Production orders & WIP** | MFG | Production order lifecycle, material consumption, WIP costing, variance analysis | Manufacturing industry config |
| **Cycle counting & QA** | INV | Cycle count programs, quality inspection, inspection lots, vendor quality feedback | Inventory subledger |
| **Expense management** | H2R | Expense reports, policy enforcement, receipt matching, reimbursement | Employee master data |

**Impact:** Manufacturing becomes a fully simulated chain. Inventory completeness enables ABC analysis and obsolescence. Expenses extend H2R with AP integration.

### Wave 4 — Polish

| Item | Chain | Description | Dependencies |
|------|-------|-------------|-------------|
| **Sales quotes** | O2C | Quote-to-order conversion tracking (fills orphan `quote_id` FK) | O2C generator |
| **Cash forecasting** | BANK | Projected cash flows from AP/AR schedules | AP/AR subledgers |
| **KPIs & budget variance** | R2R | Management reporting, budget vs actual analysis | Financial statements |
| **Obsolescence management** | INV | Slow-moving/excess stock identification and write-downs | Inventory aging |

**Impact:** These items round out each chain with planning and reporting capabilities.

---

## Cross-Process Integration Vision

The wave plan steadily increases cross-process coverage:

| Integration | Current | After Wave 1 | After Wave 2 | After Wave 4 |
|-------------|---------|-------------|-------------|-------------|
| S2P → Inventory | GR updates stock | Same | Same | Same |
| Inventory → O2C | Delivery reduces stock | Same | Same | Obsolescence feeds write-downs |
| S2P/O2C → BANK | Payments created | Payments reconciled | Same | Cash forecasting |
| All → R2R | JEs → Trial Balance | JEs → Financial Statements | + Revenue recog, impairment | + Budget variance |
| H2R → S2P | Employee authorizations | Same | Expense → AP | Same |
| S2P → A2R | Capital PO → FA | Same | Same | Same |
| MFG → S2P | Config only | Same | Production → PR demand | Same |
| MFG → INV | Config only | Same | WIP → FG transfers | + QA feedback |

---

## Coverage Targets

| Chain | Current | Wave 1 | Wave 2 | Wave 3 | Wave 4 |
|-------|---------|--------|--------|--------|--------|
| S2P | 85% | 95% | 95% | 95% | 95% |
| O2C | 93% | 93% | 97% | 97% | 99% |
| R2R | 78% | 88% | 92% | 92% | 97% |
| A2R | 70% | 70% | 80% | 80% | 80% |
| INV | 55% | 55% | 55% | 75% | 85% |
| BANK | 65% | 85% | 85% | 85% | 90% |
| H2R | 30% | 30% | 60% | 75% | 75% |
| MFG | 20% | 20% | 20% | 60% | 60% |

---

## Guiding Principles

- **Enterprise realism**: Simulate multi-entity, multi-region, multi-currency operations with coherent process flows
- **ML ground truth**: Capture true labels and causal factors for supervised learning, explainability, and evaluation
- **Scalability**: Handle large volumes with stable performance and reproducible results
- **Backward compatibility**: New features are additive; existing configs continue to work

---

## Dependencies & Risks

- **Schema stability**: New models must not break existing serialization formats
- **Performance**: Each wave adds generators; resource guards ensure stable memory/CPU
- **Validation complexity**: Cross-chain coherence checks multiply as integration points increase

---

## Contributing

We welcome contributions to any roadmap area. See [Contributing Guidelines](../contributing/README.md) for details.

To propose new features:
1. Open a GitHub issue with the `enhancement` label
2. Describe the use case and expected behavior
3. Reference relevant roadmap items if applicable

---

## Feedback

Roadmap priorities are influenced by user feedback. Please share your use cases and requirements:

- GitHub Issues: Feature requests and bug reports
- Email: [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com)

## See Also

- [Process Chains](../architecture/process-chains.md) — Current process chain architecture and coverage matrix
- [S2P Spec](../../specs/s2p-process-chain-spec.md) — Source-to-Contract specification
- [Process Chain Gaps](../../specs/enterprise-process-chain-gaps.md) — Detailed gap analysis
