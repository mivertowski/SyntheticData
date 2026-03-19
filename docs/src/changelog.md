# Changelog

For the full changelog, see the [CHANGELOG.md](https://github.com/mivertowski/SyntheticData/blob/main/CHANGELOG.md) in the repository root.

## Recent Releases

### [1.3.0] - 2026-03-19

**Enterprise Group Audit Simulation**

- **32 new features** across 5 implementation tiers for complete ISA / IFRS / US GAAP group audit simulation
- **ISA 600 Component Auditor**: Group audit planning with materiality allocation and scope assignment
- **Consolidated Financial Statements**: Standalone + consolidated FS with elimination schedules
- **10 accounting standards**: Deferred tax (IAS 12), business combinations (IFRS 3), segment reporting (IFRS 8), ECL (IFRS 9), pensions (IAS 19), provisions (IAS 37), stock comp (ASC 718), functional currency (IAS 21)
- **Audit documentation**: Engagement letters (ISA 210), subsequent events (ISA 560), service org controls (ISA 402), accounting estimates (ISA 540), going concern, notes to FS
- **4 new evaluators**: Financial ratio analysis (ISA 520), JE risk scoring, materiality sampling, trend plausibility
- **Audit preset**: `--preset audit-group` CLI flag enabling all audit features
- **Format exporters**: `--export-format sap|fec|gobd` for SAP BKPF/BSEG/ACDOCA, French FEC, German GoBD
- **Bug fixes**: Subledger settlement, IC eliminations → GL, close engine, opening balances, audit output completeness
- **40+ new output files** across audit, financial reporting, tax, accounting standards, HR, subledger, and intercompany

### [1.2.0] - 2026-03-15

**Unified Graph Export Pipeline & Audit Procedures**

- **`datasynth-graph-export` crate**: New standalone crate with budget-managed, topologically-sorted export pipeline replacing the monolithic adapter pattern
- **30 property serializers**: Typed extraction for all domain models (accounting, P2P, O2C, S2C, H2R, MFG, audit, banking, controls, risk)
- **13 node synthesizers**: AML alerts, collusion rings, compliance, ESG, intercompany, KYC, OCEL events, projects, red flags, subledger recon, tax, temporal events, treasury
- **10 edge synthesizer domains**: Accounting, audit trail, banking, document chain, entity relationships, H2R, MFG, process sequence, risk-control, S2C
- **Audit procedure models & generators**: ISA 505 confirmations, ISA 330/530 procedure steps & samples, ISA 520/610 analytical procedures & internal audit, ISA 550 related parties
- **Post-processors**: EffectiveControlCount, AnomalyFlag, RedFlag, DuplicateEdge passes
- **OCEL exporter**: Process mining export from graph structure
- **Budget rebalancing API**: Enforced phase ordering (audit before banking) with configurable node/edge budgets
- **8 new graph builder domains**: Tax, treasury, ESG, project, intercompany, temporal, AML, KYC
- **Core model enhancements**: Risk linkage on InternalControl, continuous risk scores on RiskAssessment, DocumentRef enum on JournalEntry
- Integration tests for full pipeline, budget enforcement, property serializers, and edge synthesizers

### [1.1.0] - 2026-03-09

**Compliance & Regulations Framework**

- **StandardId & StandardRegistry**: Canonical standard identifiers (`IFRS-16`, `SOX-404`, `ISA-315`) with ~45 built-in standards, temporal version resolution, supersession chains, and cross-reference traversal
- **Jurisdiction profiles**: 10 country profiles (US, DE, GB, FR, JP, IN, SG, AU, BR, KR) with accounting/audit frameworks, supranational memberships, and tax rates
- **Temporal versioning**: Jurisdiction-aware date resolution with per-country effective date overrides and early adoption dates
- **Audit procedure generation**: 9 ISA-based templates (substantive detail, analytical, controls test, inspection, confirmation, recalculation, observation, inquiry, cutoff test) with sampling parameters
- **Compliance finding generation**: 10 finding templates with SOX/ISA deficiency classification (MaterialWeakness, SignificantDeficiency, ControlDeficiency) and remediation tracking
- **Regulatory filing generation**: 8 filing types across 5 jurisdictions (US, DE, FR, GB, JP) with deadline tracking and status progression
- **Compliance graph layer**: Standard, Jurisdiction, AuditProcedure, and Finding nodes with CrossReference, Supersedes, MapsToStandard, TestsCompliance, and FindingOnStandard edges
- **Configuration & validation**: Full `compliance_regulations` config section with standards selection, audit procedures, findings, filings, graph, and output sub-configs
- 67+ tests across all compliance modules
- 7 JSON output files in `compliance_regulations/` directory

**Cross-Domain Compliance Graph Linking**

- **ToNodeProperties for compliance models**: `ComplianceStandard` (510), `ComplianceFinding` (511), `RegulatoryFiling` (512), `JurisdictionProfile` (513) — typed camelCase property maps for graph nodes
- **Standard-to-account/process mapping**: All 45+ built-in standards mapped to applicable GL account types and business processes (O2C, P2P, R2R, H2R, A2R, Intercompany)
- **5 new cross-domain edge types**: `GovernedByStandard`, `ImplementsStandard`, `FiledByCompany`, `FindingAffectsControl`, `FindingAffectsAccount`
- **Hypergraph integration**: Standards in Layer 1 (GovernanceControls), findings/filings in Layer 2 (ProcessEvents), with cross-layer edge resolution
- **Full enterprise graph traversal**: Company → Filing → Jurisdiction → Standard → Account → JournalEntry
- **3 new config fields**: `include_account_links`, `include_control_links`, `include_company_links` (all default `true`)

### [1.0.0] - 2026-03-06

**v1.0.0 Release — Enterprise Simulation & ML Ground Truth**

- **Process evolution & organizational events**: Workflow changes, automation events, policy updates, acquisitions, divestitures, reorganizations
- **Disruption event generation**: Outage, migration, process change, recovery, and regulatory disruption events
- **Counterfactual pair generation**: (original, mutated) journal entry pairs for ML training
- **Fraud red-flag indicators**: Risk indicators attached to P2P/O2C document chains
- **Collusion ring generation**: Coordinated fraud networks from employee/vendor pools with role-based conspirators
- **Bi-temporal vendor versioning**: Version chains with valid-time/transaction-time dimensions
- **Entity relationship graph**: Relationship graphs with strength scores and cross-process links (P2P/O2C via inventory)
- **Industry transaction factory**: Industry-specific GL accounts for Retail, Manufacturing, Healthcare, Financial Services
- **SmallVec optimization**: Avoids heap allocation for expense report line items and quality inspection characteristics
- **Zero-copy transfers**: `std::mem::take()` for master data, `Arc::try_unwrap()` for chart of accounts
- **Division-by-zero guards**: CompanySelector, SplitTransactionStrategy, SchemeAdvancer hardened
- **Numeric cast safety**: Clamped period_months and statutory_rate conversions in orchestrator
- **Silent error recovery**: `let _ =` patterns replaced with `tracing::warn!` logging
- **Clippy cleanup**: `uninlined_format_args` fixed across 239 files
- 11 integration tests for all newly wired generators
- Complete crates.io metadata (keywords, categories) for all crates

### [0.11.0] - 2026-03-02

**Multi-Period Sessions, Fraud Packs, Streaming Pipeline, OCEL Enrichment**

- **GenerationSession**: Stateful multi-period generation with `.dss` checkpoint files, fiscal-year-aligned period splitting, deterministic seed advancement
- **Incremental generation**: `--append --months N` adds more periods to an existing session
- **Fraud scenario packs**: 5 built-in YAML packs (`revenue_fraud`, `payroll_ghost`, `vendor_kickback`, `management_override`, `comprehensive`) with deep-merge and `--fraud-rate` override
- **StreamPipeline**: Phase-aware streaming via `PhaseSink` trait with file (JSONL), HTTP, and no-op targets
- **OCEL 2.0 enrichment**: Lifecycle state machines (PO, SO, VI), correlation events (ThreeWayMatch, PaymentAllocation, BankReconciliation), resource pool modeling (RoundRobin, LeastBusy, SkillBased)
- **4 new evaluators**: Multi-period coherence, fraud pack effectiveness, OCEL enrichment quality, causal intervention magnitude
- **DiffEngine completion**: Record-level diffs and aggregate metric comparison for counterfactual analysis
- Desktop UI: Fraud Scenario Packs, Causal DAG, Generation Session, Streaming, OCPM enrichment pages
- 13 integration tests across session, OCEL, and fraud pack modules

### [0.10.0] - 2026-03-02

**Counterfactual Simulation Engine**

- Causal DAG with 17 financial process nodes, 8 transfer function types, topological sort via Kahn's algorithm
- `ScenarioEngine` orchestrator: paired baseline/counterfactual generation, scenario manifest, DAG presets
- CLI subcommand: `datasynth-data scenario {list, validate, generate, diff}`
- 59 new tests (45 unit + 14 integration)

### [0.9.x] - 2026-02-25 through 2026-03-01

- **v0.9.5**: Mutex poisoning recovery, 7 new country packs, 4 new generators (OrganizationalEvent, ProcessEvolution, DriftEvent, Confirmation)
- **v0.9.4**: RustGraph Round 2 — `ToNodeProperties` for 51 entity types, 28 new `RelationshipType` variants, edge constraints
- **v0.9.3**: Edge-case hardening — division-by-zero guards, ghost edge elimination, VAT line splitting, multipayment behavior
- **v0.9.2**: Comprehensive quality fixes (Tiers 1-6) — constant-time auth, framework-aware accounts, Neo4j/DGL wiring
- **v0.9.1**: German GAAP (HGB) with SKR04, `FrameworkAccounts` generalization, GoBD audit export
- **v0.9.0**: ~2x performance — cached temporal CDF, fast Decimal, SmallVec, parallel generation, zstd compression

### [0.8.x] - 2026-02-18 through 2026-02-20

- French GAAP (PCG) with Plan Comptable General 2024 and FEC export
- Pluggable country pack architecture with runtime-loaded JSON packs (US, DE, GB)

### [0.7.0] - 2026-02-17

- Tax Accounting, Treasury & Cash Management, Project Accounting, ESG/Sustainability
- OCPM expanded to 12 process families with 101+ activities and 65+ object types

### Earlier Releases

- **v0.6.x**: Enterprise process chains (S2C, H2R, MFG), universal OCPM, evaluation framework
- **v0.5.0**: LLM-augmented generation, diffusion models, causal generation, federated fingerprinting, ecosystem integrations
- **v0.3.0**: ACFE fraud taxonomy, collusion modeling, industry-specific transactions, ML benchmarks
- **v0.2.x**: Privacy-preserving fingerprinting, accounting/audit standards, streaming output
- **v0.1.0**: Core generation engine, master data, document flows, subledgers, graph export
