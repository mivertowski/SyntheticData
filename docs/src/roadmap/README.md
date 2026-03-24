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

### v1.4.0 — Realism & Coherence

- **Cost center hierarchy generator**: Parent/child trees with department mappings and GL assignments
- **Employee change history**: Title changes, salary adjustments, department transfers
- **Multi-period balance carry-forward**: Trial balance closing → opening balance propagation
- **Dunning generator wiring**: Dunning runs and letters after AR aging
- **AR/AP reconciliation validation**: Subledger totals vs GL control accounts
- **Contract→PO linkage**: Procurement contracts carry PO IDs for S2P chain traversal
- **Moving-average inventory cost**: AVCO updated on each goods receipt
- **Production order ↔ inventory cross-refs**: Bidirectional traceability
- **ISA mappings output**: 34 ISA standard reference records
- **SoD/COSO control mappings**: Automated export of conflict pairs and COSO mappings
- **Graph export enhancements**: JE→Employee (POSTED_BY), Control→JE (CONTROL_APPLIED) edges
- **Consolidated financial statements**: Standalone + consolidated with elimination schedules

### v1.5.0 — Audit FSM Engine & Optimizer

- **YAML-driven audit FSM engine** (`datasynth-audit-fsm`): Loads ISA and IIA-GIAS methodology blueprints as event-sourced finite state machines
  - Financial Statement Audit (FSA): 9 procedures, 3 phases, 24 steps → 51 events, 1,916 artifacts
  - Internal Audit (IA): 34 procedures, 9 phases, 82 steps → 368 events, 1,891 artifacts
  - StepDispatcher: 135 command mappings to 14 pre-initialized audit generators
  - 8-state C2CE (Condition-Criteria-Cause-Effect) lifecycle for finding development
  - Self-loop handling with configurable max iterations
  - Continuous phase support (parallel execution for ethics, governance, quality)
  - Discriminator-based procedure filtering (categories, risk ratings, engagement types)
  - Generation overlay presets: default, thorough, rushed
  - Flat JSON event trail + OCEL 2.0 projection exports
  - Custom YAML blueprint support
- **Audit FSM optimizer** (`datasynth-audit-optimizer`): Graph analysis and Monte Carlo simulation
  - Blueprint → petgraph directed graph conversion
  - Shortest path analysis (BFS per procedure): FSA 27 min transitions, IA 101
  - Constraint-based path optimization with transitive precondition expansion
  - Monte Carlo simulation: bottleneck detection, revision hotspots, happy path identification
- **Orchestrator integration**: FSM engine wired into enhanced orchestrator with full artifact pipeline
- **Blueprint repository**: [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints)

---

## Process Coverage (v1.5.0)

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
| **AUDIT** (Audit Methodology) | 90% | FSA + IA blueprints, 1,900+ artifacts, event trail, OCEL 2.0 |

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
| AUDIT → R2R | Materiality derived from financial statements |
| AUDIT → Controls | Findings linked to COSO controls and affected accounts |
| AUDIT → OCEL | Event trail projected to OCEL 2.0 for process mining |

---

## Strategic Roadmap

### Wave 1: Consolidation (v1.6.0)

Near-term work to make v1.5.0 features production-ready.

#### End-to-End CLI Integration

- **Full pipeline verification**: Run `datasynth-data generate` with `audit.fsm.enabled: true` and verify the complete output directory (all 50+ artifact types written to `audit/`, event trail to `audit/fsm_event_trail.json`)
- **Missing sink registrations**: Ensure all ArtifactBag types flow through the standard output writer (CSV, JSON, Parquet)
- **Config validation**: Validate `audit.fsm` config section during `datasynth-data validate`
- **Demo mode**: Add FSM-enabled preset to `--demo` for instant evaluation

#### Blueprint Validation Tooling

- **CLI command**: `datasynth-data audit validate-blueprint --file my_methodology.yaml` — runs loader validation, reports cross-reference errors, DAG cycles, unreachable states
- **Blueprint info**: `datasynth-data audit info --blueprint builtin:fsa` — prints procedure count, phase structure, step commands, standards coverage
- **Diff tool**: Compare two blueprints to show added/removed procedures, changed transitions

#### IA Artifact Fidelity

- **Richer IA command dispatch**: Map IA-specific commands to specialized generators:
  - `assess_universe_risks` → risk universe document with entity-level risk ratings
  - `develop_recommendations` → structured recommendation artifacts with management response fields
  - `draft_ia_charter` → formal IA charter document with mandate, scope, authority
  - `develop_annual_plan` → audit plan artifact with resource allocation and timeline
- **IA-specific workpaper sections**: Extend `WorkpaperSection` with IA variants (Universe, Planning, Fieldwork, Monitoring, QA)
- **Finding quality**: C2CE findings should carry quantified financial impact, root cause categorization, and management action plan timelines

#### Graph Integration Depth

- **Audit-specific graph edges**: Step→Evidence (PRODUCED_BY), Finding→Risk (IDENTIFIED_FROM), Opinion→Finding (BASED_ON), Workpaper→Standard (COMPLIES_WITH)
- **Temporal audit graph**: Time-ordered engagement events as a temporal knowledge graph with TGN-compatible export
- **Hypergraph audit nodes**: Register all FSM artifact types (engagement, materiality, finding, opinion) as first-class hypergraph nodes

---

### Wave 2: Audit Planning Optimization (v1.7.0)

The optimizer crate evolves from an analytical tool into a planning tool.

#### Resource-Constrained Optimization

- **Cost model**: Assign hour costs per procedure (partner hours, manager hours, staff hours) from overlay or blueprint metadata
- **Staff availability constraints**: Model team capacity (e.g., partner available 20h/week, 3 seniors available) and find feasible audit plans
- **Budget optimization**: Given a total hour budget, find the audit plan that maximizes risk coverage
- **Critical path analysis**: Identify the longest dependency chain that determines minimum engagement duration

#### Risk-Based Audit Scoping

- **Risk-weighted procedure selection**: Given a risk profile (industry, entity size, prior findings), use discriminators and Monte Carlo to recommend which procedures to include
- **Coverage analysis**: For a given scope, compute ISA/IIA-GIAS standards coverage percentage and identify uncovered requirements
- **What-if analysis**: "What happens to coverage if we drop procedure X?" — instant impact assessment via graph analysis

#### Multi-Engagement Portfolio

- **Portfolio simulation**: Generate N engagements with correlated parameters (same industry → correlated risk profiles, shared systemic findings)
- **Resource pooling**: Model shared audit team across engagements with scheduling constraints
- **Portfolio risk heatmap**: Aggregate risk across engagements to identify systemic exposure

---

### Wave 3: Process Mining & Benchmarks (v1.8.0)

Leverage the OCEL 2.0 projection for process mining research.

#### Reference Audit Event Logs

- **Benchmark dataset generation**: Produce standardized OCEL 2.0 audit event logs with known anomalies (skipped approvals, late postings, out-of-sequence steps) at configurable injection rates
- **Conformance checking ground truth**: Given a blueprint (normative model) and generated event log (with deviations), provide labeled conformance violations for evaluating process mining tools
- **Multiple complexity levels**: Simple (FSA, default overlay, no anomalies), Medium (FSA, rushed overlay, moderate anomalies), Complex (IA, mixed overlays, high anomaly rate)

#### Process Mining Tool Integration

- **PM4Py native export**: Direct export to PM4Py DataFrame format with object-centric support
- **Celonis IBC format**: Export compatible with Celonis Intelligent Business Cloud import
- **ProM/XES export**: Traditional single-object event log in XES format for backward compatibility
- **Disco/Minit CSV**: Flat case-activity-timestamp CSV for commercial process mining tools

#### Conformance Metrics

- **Fitness score**: Percentage of event traces that conform to the blueprint FSM
- **Precision score**: How much behavior the model allows beyond what was observed
- **Generalization**: Model behavior on unseen engagement configurations
- **Anomaly detection benchmark**: F1/precision/recall for process mining anomaly detectors against known injected anomalies

---

### Wave 4: Learned & Adaptive Generation (v2.0.0)

Combine the deterministic FSM framework with learned components.

#### Learned Overlay Parameters

- **Engagement profile fitting**: Given real audit engagement metadata (duration, findings count, revision frequency, team size), fit overlay parameters to reproduce those characteristics
- **Industry-calibrated overlays**: Pre-fitted overlays for financial services audits, manufacturing audits, technology audits based on aggregate engagement statistics
- **Temporal drift**: Overlay parameters that evolve over time (e.g., increasing regulatory scrutiny → more revision loops, longer durations)

#### LLM-Augmented Artifact Content

- **Contextual narrative generation**: Plug an LLM into the StepDispatcher to generate finding descriptions, management responses, workpaper narratives, and engagement letter prose
- **ISA-grounded prompts**: Each step's standards references become prompt context, ensuring generated text cites the correct ISA paragraphs
- **Deterministic fallback**: LLM output is optional; the system produces valid artifacts with template text when LLM is unavailable
- **Quality control**: Generated narratives validated against step evidence requirements (finding must reference the evidence it was derived from)

#### Bidirectional Blueprint Discovery

- **Process discovery from event logs**: Given real audit event logs (anonymized), infer the underlying methodology blueprint (states, transitions, procedures)
- **Blueprint comparison**: Diff a discovered blueprint against a reference (ISA, IIA-GIAS) to identify deviations from standard methodology
- **Methodology conformance scoring**: Quantify how closely an organization's actual audit practice matches the declared methodology

#### Adaptive Anomaly Calibration

- **Reinforcement learning**: Tune anomaly injection parameters such that downstream detector performance matches a target difficulty curve
- **Curriculum generation**: Progressive difficulty datasets — start with obvious anomalies, gradually increase subtlety
- **Adversarial generation**: Generate anomalies specifically designed to evade a given detector, for robustness testing

---

### Wave 5: Enterprise Platform (v2.x)

Platform-level capabilities for production deployment.

#### Continuous Audit Simulation

- **Streaming engagement execution**: FSM engine emits events in real-time via WebSocket/Kafka, simulating a continuous audit monitoring environment
- **Live anomaly injection**: Anomalies injected at runtime, not just at generation time — simulates emerging risks
- **Dashboard integration**: Event stream compatible with Grafana, Splunk, or custom audit dashboards
- **Alert correlation**: Cross-reference audit events with transactional anomalies (fraud labels) for holistic monitoring simulation

#### Multi-Engagement Correlation

- **Systemic issue propagation**: A new IFRS standard causes clustered findings across multiple clients in the same industry
- **Peer comparison**: Generate engagement pairs (similar industry, different entity) for benchmarking analytics
- **Group audit coordination**: ISA 600 group audit with component auditors, each running their own FSM engagement, consolidated at group level
- **Year-over-year engagement chains**: Sequential engagements for the same entity with carry-forward of prior-year findings, risk assessments, and control evaluations

#### Custom Blueprint Ecosystem

- **Blueprint marketplace**: Community-contributed methodology blueprints (ISA, PCAOB, IIA-GIAS, firm-specific shells) in the [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints) repository
- **Blueprint versioning**: Semantic versioning for blueprints with backward compatibility guarantees
- **Visual blueprint editor**: Browser-based tool for creating and editing YAML blueprints with state machine visualization
- **Blueprint testing framework**: Automated validation that a blueprint produces expected artifact types, event counts, and phase progression

#### Additional Audit Frameworks

- **PCAOB AS blueprint**: US public company audit methodology with PCAOB-specific procedures
- **EY GAM shell**: Ernst & Young Global Audit Methodology structure (non-proprietary)
- **KPMG AAER shell**: KPMG audit methodology structure
- **Regulatory exam blueprints**: Banking supervision exam workflows (OCC, Fed, FDIC)
- **SOC 2 Type II blueprint**: Service organization audit with trust services criteria procedures

---

### Wave 6: Research Frontier (v3.x)

Long-term research directions.

#### Knowledge Graph Completion Benchmarks

- **Reference audit knowledge graphs**: Fully provenanced graphs where every node, edge, and property traces to a known generative process
- **Standardized KGC benchmarks**: Evaluate knowledge graph construction and completion algorithms against known ground truth
- **Multi-layer evaluation**: Separate accuracy metrics for structural (entity/relationship), statistical (amount distributions), and normative (standards compliance) layers

#### Temporal Graph Networks for Audit

- **TGN-compatible export**: Audit event streams formatted for temporal graph network models
- **Dynamic audit risk prediction**: Train TGN models to predict engagement outcomes from partial event trails
- **Early warning detection**: Identify engagements heading toward adverse opinions from early-phase events

#### Federated Audit Simulation

- **Cross-organization generation**: Multiple organizations with shared auditor (simulates Big 4 portfolio)
- **Privacy-preserving aggregation**: Federated learning on audit engagement statistics without exposing individual client data
- **Industry-level reference graphs**: Aggregate knowledge graphs spanning multiple generated enterprises for sector-level analysis

#### Causal Audit Analytics

- **Causal DAG for audit outcomes**: Structural causal model linking engagement parameters (scope, team, timeline) to outcomes (findings, opinion type, duration)
- **do-calculus interventions**: "What would happen to the opinion if we doubled the substantive testing scope?"
- **Counterfactual engagement pairs**: Generate paired engagements (baseline vs intervention) for causal inference research

---

### Implementation Priority Matrix

| Wave | Horizon | Key Deliverable | Strategic Value |
|------|---------|-----------------|-----------------|
| 1 — Consolidation | 1-2 months | Production-ready FSM pipeline | Usability |
| 2 — Audit Planning | 3-4 months | Resource-constrained audit optimization | Commercial |
| 3 — Process Mining | 4-6 months | Reference OCEL benchmark datasets | Research/Citations |
| 4 — Learned Generation | 6-12 months | LLM-augmented artifacts, learned overlays | Differentiation |
| 5 — Enterprise Platform | 12-18 months | Continuous audit, multi-engagement, marketplace | Platform |
| 6 — Research Frontier | 18-36 months | TGN, causal inference, federated simulation | Academic impact |

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
- GitHub: [Issues](https://github.com/mivertowski/SyntheticData/issues)

## See Also

- [Process Chains](../architecture/process-chains.md) — Process chain architecture and coverage matrix
- [S2P Spec](../../specs/s2p-process-chain-spec.md) — Source-to-Contract specification
- [Process Chain Gaps](../../specs/enterprise-process-chain-gaps.md) — Detailed gap analysis
