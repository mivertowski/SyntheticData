# Changelog

For the full changelog, see the [CHANGELOG.md](https://github.com/ey-asu-rnd/SyntheticData/blob/main/CHANGELOG.md) in the repository root.

## Recent Releases

### [0.11.0] - 2026-03-02

**Multi-Period Sessions, Fraud Packs, Streaming Pipeline, OCEL Enrichment**

- **GenerationSession**: Stateful multi-period generation with `.dss` checkpoint files, fiscal-year-aligned period splitting, deterministic seed advancement
- **Incremental generation**: `--append --months N` adds more periods to an existing session
- **Fraud scenario packs**: 5 built-in YAML packs (`revenue_fraud`, `payroll_ghost`, `vendor_kickback`, `management_override`, `comprehensive`) with deep-merge and `--fraud-rate` override
- **StreamPipeline**: Phase-aware streaming via `PhaseSink` trait with file (JSONL), HTTP, and no-op targets
- **OCEL 2.0 enrichment**: Lifecycle state machines (PO, SO, VI), correlation events (ThreeWayMatch, PaymentAllocation, BankReconciliation), resource pool modeling (RoundRobin, LeastBusy, SkillBased)
- **4 new evaluators**: Multi-period coherence, fraud pack effectiveness, OCEL enrichment quality, causal intervention magnitude
- **DiffEngine completion**: Record-level diffs and aggregate metric comparison for counterfactual analysis
- **ConfigMutator constraints**: `preserve_accounting_identity`, `preserve_document_chains`, `preserve_period_close`, `preserve_balance_coherence`
- **Minimal DAG preset**: 6-node causal DAG for lightweight counterfactual analysis
- **ProcessChange and RegulatoryChange** intervention types for causal mapping
- **Desktop UI**: Fraud Scenario Packs, Causal DAG, Generation Session, Streaming, OCPM enrichment pages
- **Python v1.8.0**: `with_fraud_packs()`, `with_scenarios()`, `with_streaming()` blueprints
- **5 new documentation pages**: Fraud Scenario Packs, Counterfactual Scenarios, OCEL 2.0 Enrichment, Streaming Pipeline, Evaluation Framework
- CLI flags: `--fiscal-year-months`, `--append`, `--months`, `--fraud-scenario`, `--fraud-rate`, `--stream-file`
- 13 integration tests across session, OCEL, and fraud pack modules

### [0.10.0] - 2026-03-02

**Counterfactual Simulation Engine**

- Causal DAG with 17 financial process nodes, 8 transfer function types, topological sort via Kahn's algorithm
- `Scenario`, `Intervention`, `InterventionTiming` core data models with 8 `InterventionType` variants
- `CausalPropagationEngine`: onset interpolation (Sudden, Gradual, Oscillating), lag-aware propagation, bounds clamping
- `InterventionManager`: timing validation, bounds checking, conflict detection with priority-based resolution
- `ConfigMutator`: dot-path config mutation with array indexing, null-stripping, custom constraint validation
- `ScenarioEngine` orchestrator: paired baseline/counterfactual generation, scenario manifest, DAG presets
- `ScenarioDiff` types: `ImpactSummary`, `KpiImpact`, `RecordLevelDiff`, `AggregateComparison`, `InterventionTrace`
- CLI subcommand: `datasynth-data scenario {list, validate, generate, diff}`
- 59 new tests (45 unit + 14 integration) across core, config, runtime, and eval crates

### [0.9.5] - 2026-03-01

- Mutex poisoning recovery in streaming channel (11 calls replaced with graceful recovery)
- 7 new country packs: France, Japan, China, India, Italy, Spain, Canada
- 4 new holiday calendars (FR, IT, ES, CA)
- Progressive tax bracket computation, credit memo wiring in O2C flow
- 4 new generators: OrganizationalEvent, ProcessEvolution, DriftEvent, Confirmation
- 39 new integration tests across eval, output, config, and banking crates

### [0.9.4] - 2026-03-01

**RustGraph Round 2 — Graph Property Mapping & Entity/Edge Registry (DS-001 through DS-012)**

- `ToNodeProperties` trait and `GraphPropertyValue` enum for converting typed model structs to graph property maps with camelCase keys
- `GraphEntityType` expanded with 35+ new entity variants across Tax (7), Treasury (8), ESG (13), Project (5), S2C (4), H2R (4), MFG (4), GOV (5) domains
- Edge type registry: 28 new `RelationshipType` variants with `EdgeConstraint` struct (source/target entity types, cardinality)
- New model structs: `BomComponent` (multi-level BOM), `InventoryMovement` (goods movement tracking), `BenefitEnrollment` (employee benefit plans)
- `ToNodeProperties` implemented for all 51 entity types across 10 process families
- Denormalized name fields (`vendor_name`, `customer_name`, `employee_name`, `material_description`) on transaction models with generator population
- Boolean query flags: `treatyApplied`, `isApproved`, `isPassed`, `isPhantom`, `isActive`, `billable`
- New generators: BOM component, inventory movement, benefit enrollment
- `GraphNode::from_entity()` bridge wiring `ToNodeProperties` into the graph export pipeline
- Comprehensive test suite: entity registry uniqueness, edge constraint validation, category helpers, property round-trips

### [0.9.3] - 2026-02-27

**Community Features & Edge-Case Hardening**

- **VAT line splitting in O2C/P2P journal entries** ([#64](https://github.com/DataSynth/SyntheticData/issues/64)): Customer Invoice JE now correctly posts DR AR (gross), CR Revenue (net), CR VAT Payable (tax); Vendor Invoice JE posts DR GR/IR (net), DR Input VAT (tax), CR AP (payable)
- **Multipayment behavior** ([#65](https://github.com/DataSynth/SyntheticData/issues/65)): O2C/P2P partial payments now generate remainder payments with configurable timing, full JE generation, and cash flow integration
- **Account-class fingerprinting** ([#66](https://github.com/DataSynth/SyntheticData/issues/66)): Per-account-class statistics extraction with semantic column detection, per-class Benford analysis, and distribution fitting for synthesis
- Division-by-zero guards in fingerprint k-anonymity and federated aggregation
- Graph ghost edge elimination (skip missing nodes instead of remapping to node 0)
- GoBD safe document ID truncation, Prometheus/rate-limit unwrap removal
- Deterministic household UUIDs, config-driven P2P/O2C rates
- NaN guards in distribution fitter and mixture samplers
- Dead code removal (3 structs, 2 fields, 1 variable)
- Improved error logging across 12 crates (serialization, parsing, configuration)

### [0.9.2] - 2026-02-27

**Comprehensive Codebase Quality Fixes (Tiers 1-6)**

- Framework-aware account classification in all generators (balance tracker, trial balance, currency translator, IC generator, graph builder)
- Constant-time gRPC auth token comparison (`subtle::ConstantTimeEq`)
- Fixed employee generator `last_mut()` ordering, banking RNG determinism, CLI verify count mismatch, DGL node types
- Neo4j/DGL graph exports wired in orchestrator; server stream/reload/proto stubs implemented
- GoBD tax amount and contra account improvements for multi-line entries
- Config validation: start_date format, company name/country, safety limit warnings
- Production unwrap/expect calls replaced with descriptive errors across 6 crates
- Shared NPY writer extraction; proper Beta distribution; improved A-D p-value and AML detectability
- 59 files changed, 3,376 tests pass, 0 clippy warnings

### [0.9.1] - 2026-02-26

**Generalized Multi-GAAP Framework + German GAAP (HGB)**

- German GAAP (HGB) framework: SKR04 chart of accounts, Degressiv depreciation, GWG low-value asset expensing, BMF lease classification, mandatory impairment reversal
- Generalized `FrameworkAccounts` mapping ~45 semantic accounts per framework (US GAAP, French PCG, German SKR04)
- GoBD audit export (13-column journal CSV + account CSV + XML index)
- Auxiliary GL sub-accounts on vendor/customer master data (PCG `401XXXX`/`411XXXX`, SKR04 `3300XXXX`/`1200XXXX`)
- FEC auxiliary fields now use framework-specific GL accounts instead of raw partner IDs
- Expanded French PCG account modules (fixed assets, tax, suspense, equity, liabilities)

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
