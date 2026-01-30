# CLAUDE.md

Guidance for Claude Code working with this repository.

## Build Commands

```bash
cargo build --release          # Build binary
cargo test                     # All tests
cargo test -p datasynth-core   # Specific crate
cargo test test_name           # Single test
cargo check                    # Check only
cargo fmt && cargo clippy      # Format + lint
cargo bench                    # Benchmarks
```

## CLI Usage

Binary: `datasynth-data` (at `target/release/datasynth-data`)

```bash
datasynth-data generate --demo --output ./output
datasynth-data init --industry manufacturing --complexity medium -o config.yaml
datasynth-data validate --config config.yaml
datasynth-data generate --config config.yaml --output ./output
kill -USR1 $(pgrep datasynth-data)  # Pause/resume (Unix)
```

## Server

```bash
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

## Architecture

Rust workspace with 16 crates:

```
datasynth-cli          → Binary (generate, validate, init, info, fingerprint)
datasynth-server       → REST/gRPC/WebSocket server
datasynth-ui           → Tauri/SvelteKit desktop UI
datasynth-runtime      → GenerationOrchestrator coordinates workflow
datasynth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies, Audit)
datasynth-banking      → KYC/AML banking with fraud typologies
datasynth-ocpm         → OCEL 2.0 process mining
datasynth-fingerprint  → Privacy-preserving fingerprint extraction/synthesis
datasynth-standards    → Accounting/audit standards (IFRS, US GAAP, ISA, SOX, PCAOB)
datasynth-graph        → Graph export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         → Evaluation framework with auto-tuning
datasynth-config       → Configuration schema, validation, presets
datasynth-core         → Domain models, traits, distributions, resource guards
datasynth-output       → Output sinks (CSV, JSON, Parquet)
datasynth-test-utils   → Test utilities
```

### Key Models (datasynth-core/src/models/)

| Category | Models |
|----------|--------|
| Accounting | JournalEntry, ChartOfAccounts, ACDOCA |
| Master Data | Vendor, Customer, Material, FixedAsset, Employee, EntityRegistry |
| Document Flow | PurchaseOrder, GoodsReceipt, VendorInvoice, Payment, SalesOrder, Delivery, CustomerInvoice, CustomerReceipt, DocumentReference |
| Intercompany | IntercompanyRelationship, ICTransactionType, ICMatchedPair, TransferPricingMethod |
| Subledger | AccountBalance, TrialBalance, AR*/AP*/FA*/Inventory* records |
| FX/Close | FxRate, CurrencyTranslation, FiscalPeriod, AccrualEntry |
| Anomalies | AnomalyType, LabeledAnomaly, QualityIssue |
| Controls | InternalControl, ControlMapping, SoD |
| COSO Framework | CosoComponent, CosoPrinciple, ControlScope, CosoMaturityLevel |

### Core Infrastructure (datasynth-core/src/)

- **uuid_factory.rs**: FNV-1a hash-based deterministic UUIDs with generator-type discriminators
- **memory_guard.rs**: Memory limits (Linux /proc/self/statm, macOS ps)
- **disk_guard.rs**: Disk space monitoring (statvfs/GetDiskFreeSpaceExW)
- **cpu_monitor.rs**: CPU tracking with auto-throttle at 0.95 threshold
- **resource_guard.rs**: Unified resource orchestration
- **degradation.rs**: Graceful degradation (Normal→Reduced→Minimal→Emergency)
- **accounts.rs**: GL account constants (AR_CONTROL="1100", AP_CONTROL="2000")
- **templates/**: YAML/JSON template loading with merge strategies

### Generator Modules (datasynth-generators/src/)

| Directory | Purpose |
|-----------|---------|
| (root) | je_generator, coa_generator, company_selector, user_generator, control_generator |
| master_data/ | vendor, customer, material, asset, employee generators |
| document_flow/ | p2p_generator, o2c_generator, three_way_match, document_chain_manager |
| intercompany/ | ic_generator, matching_engine, elimination_generator |
| balance/ | opening_balance, balance_tracker, trial_balance generators |
| subledger/ | ar, ap, fa, inventory generators + reconciliation |
| fx/ | fx_rate_service, currency_translator, cta_generator |
| period_close/ | close_engine, accruals, depreciation, year_end |
| anomaly/ | injector, types, strategies, patterns |
| data_quality/ | missing_values, format_variations, duplicates, typos, labels |
| audit/ | engagement, workpaper, evidence, risk, finding, judgment generators |

### Server (datasynth-server/src/)

- REST: `/api/config`, `/api/stream/{start|stop|pause|resume}`, `/api/stream/trigger/{pattern}`
- WebSocket: `/ws/events`
- Features: API key auth (`X-API-Key`), rate limiting, request timeout

### Desktop UI (datasynth-ui/)

Tauri + SvelteKit + TailwindCSS. Run: `cd crates/datasynth-ui && npm install && npm run tauri dev`

### Graph Module (datasynth-graph/src/)

Builders: transaction_graph, approval_graph, entity_graph
Exporters: pytorch_geometric (.pt), neo4j (CSV + Cypher), dgl

### Banking Module (datasynth-banking/src/)

KYC/AML generator with typologies: structuring, funnel, layering, mule, round_tripping, fraud, spoofing

### Process Mining (datasynth-ocpm/src/)

OCEL 2.0 event logs with P2P/O2C process generators

### Fingerprint Module (datasynth-fingerprint/src/)

Privacy-preserving extraction (differential privacy, k-anonymity) → .dsf files → synthesis

```bash
datasynth-data fingerprint extract --input ./data.csv --output ./fp.dsf --privacy-level standard
datasynth-data fingerprint validate ./fp.dsf
datasynth-data fingerprint evaluate --fingerprint ./fp.dsf --synthetic ./synthetic/
```

### Evaluation Module (datasynth-eval/src/)

- statistical/: Benford's Law, distributions, temporal patterns
- coherence/: Balance validation, IC matching, document chains
- quality/: Completeness, duplicates, format validation
- ml/: Feature distributions, label quality, splits
- enhancement/: AutoTuner generates config patches from evaluation gaps

### COSO Framework (datasynth-core/src/models/coso.rs)

COSO 2013 Internal Control-Integrated Framework:
- **CosoComponent**: ControlEnvironment, RiskAssessment, ControlActivities, InformationCommunication, MonitoringActivities
- **CosoPrinciple**: 17 principles (IntegrityAndEthics through DeficiencyEvaluation) with `component()` and `principle_number()` helpers
- **ControlScope**: EntityLevel, TransactionLevel, ItGeneralControl, ItApplicationControl
- **CosoMaturityLevel**: NonExistent, AdHoc, Repeatable, Defined, Managed, Optimized

Standard controls include 12 transaction-level (C001-C060) and 6 entity-level (C070-C081) controls with full COSO mappings.

### Standards Module (datasynth-standards/src/)

Accounting and audit standards framework:

| Directory | Purpose |
|-----------|---------|
| framework.rs | `AccountingFramework` (UsGaap, Ifrs, DualReporting), `FrameworkSettings` |
| accounting/ | Revenue (ASC 606/IFRS 15), Leases (ASC 842/IFRS 16), Fair Value (ASC 820/IFRS 13), Impairment (ASC 360/IAS 36) |
| audit/ | ISA references (34 standards), Analytical procedures (ISA 520), Confirmations (ISA 505), Opinions (ISA 700/705/706/701), Audit trail, PCAOB mappings |
| regulatory/ | SOX 302/404 compliance, `DeficiencyMatrix`, Material weakness classification |

Key types:
- **Accounting**: `CustomerContract`, `PerformanceObligation`, `Lease`, `ROUAsset`, `LeaseLiability`, `FairValueMeasurement`, `ImpairmentTest`
- **Audit**: `IsaStandard`, `IsaRequirement`, `AnalyticalProcedure`, `ExternalConfirmation`, `AuditOpinion`, `KeyAuditMatter`, `AuditTrail`
- **Regulatory**: `Sox302Certification`, `Sox404Assessment`, `DeficiencyMatrix`, `MaterialWeakness`

### Standards Configuration

```yaml
accounting_standards:
  enabled: true
  framework: us_gaap  # us_gaap, ifrs, dual_reporting
  revenue_recognition:
    enabled: true
    generate_contracts: true
    avg_obligations_per_contract: 2.0
  leases:
    enabled: true
    lease_count: 50
    finance_lease_percent: 0.30
  fair_value:
    enabled: true
    level1_percent: 0.60
    level2_percent: 0.30
    level3_percent: 0.10

audit_standards:
  enabled: true
  isa_compliance:
    enabled: true
    compliance_level: comprehensive  # basic, standard, comprehensive
    framework: dual  # isa, pcaob, dual
  analytical_procedures:
    enabled: true
    procedures_per_account: 3
  confirmations:
    enabled: true
    positive_response_rate: 0.85
  sox:
    enabled: true
    materiality_threshold: 10000.0
```

### Distributions (datasynth-core/src/distributions/)

| File | Purpose |
|------|---------|
| amount.rs | AmountSampler (log-normal + Benford compliance) |
| benford.rs | BenfordSampler, EnhancedBenfordSampler, BenfordDeviationSampler |
| mixture.rs | GaussianMixtureSampler, LogNormalMixtureSampler (weighted components) |
| copula.rs | Gaussian, Clayton, Gumbel, Frank, StudentT copulas |
| correlation.rs | CorrelationEngine (cross-field dependency modeling) |
| pareto.rs | ParetoSampler (heavy-tailed distributions) |
| weibull.rs | WeibullSampler (time-to-event modeling) |
| beta.rs | BetaSampler (proportions, percentages) |
| zero_inflated.rs | ZeroInflatedSampler (excess zeros) |
| conditional.rs | ConditionalDistribution (breakpoint-based generation) |
| drift.rs | DriftConfig, RegimeChange, EconomicCycle parameters |
| industry_profiles.rs | Pre-configured profiles for Retail, Manufacturing, Financial Services |
| temporal.rs | TemporalSampler (seasonality), HolidayCalendar |
| fraud.rs | FraudAmountGenerator |

### Distributions Configuration

```yaml
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
    fields:
      - { name: amount, distribution_type: lognormal }
      - { name: line_items, distribution_type: normal, min_value: 1, max_value: 20 }
      - { name: approval_level, distribution_type: normal, min_value: 1, max_value: 5 }
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
      recession_depth: 0.25
  validation:
    enabled: true
    tests:
      - { type: benford_first_digit, threshold_mad: 0.015 }
      - { type: distribution_fit, target: lognormal, significance: 0.05 }
      - { type: correlation_check, significance: 0.05 }
      - { type: chi_squared, significance: 0.05 }
      - { type: anderson_darling, significance: 0.05 }
    fail_on_violation: false
```

## Key Design Decisions

1. **Deterministic RNG**: ChaCha8 with configurable seed
2. **Precise Decimals**: rust_decimal serialized as strings (no IEEE 754)
3. **Balanced Entries**: JournalEntry enforces debits = credits at construction
4. **Benford's Law**: Amount distribution follows first-digit law
5. **Document Chain Integrity**: Proper payment→invoice reference chains
6. **Balance Coherence**: Assets = Liabilities + Equity validation
7. **Collision-Free UUIDs**: Generator-type discriminators prevent ID collisions
8. **Graceful Degradation**: Progressive feature reduction under resource pressure
9. **Three-Way Match**: PO/GR/Invoice matching with configurable tolerances

## Configuration

YAML sections: `global`, `companies`, `chart_of_accounts`, `transactions`, `output`, `fraud`, `internal_controls`, `enterprise`, `master_data`, `document_flows`, `intercompany`, `balance`, `subledger`, `fx`, `period_close`, `graph_export`, `anomaly_injection`, `data_quality`, `business_processes`, `templates`, `approval`, `departments`, `distributions`, `accounting_standards`, `audit_standards`

Presets: manufacturing, retail, financial_services, healthcare, technology
Complexity: small (~100 accounts), medium (~400), large (~2500)

### Internal Controls Config

```yaml
internal_controls:
  enabled: true
  coso_enabled: true                    # Enable COSO 2013 framework
  include_entity_level_controls: true   # Include C070-C081 entity-level controls
  target_maturity_level: "managed"      # ad_hoc|repeatable|defined|managed|optimized|mixed
  exception_rate: 0.02
  sod_violation_rate: 0.01
```

### Validation Rules

- period_months: 1-120
- compression level: 1-9
- rates/percentages: 0.0-1.0
- approval thresholds: ascending order
- distribution sums: 1.0 (±0.01)

## Anomaly Categories

- **Fraud**: FictitiousTransaction, RevenueManipulation, SplitTransaction, RoundTripping, GhostEmployee, DuplicatePayment
- **Error**: DuplicateEntry, ReversedAmount, WrongPeriod, WrongAccount, MissingReference
- **Process**: LatePosting, SkippedApproval, ThresholdManipulation
- **Statistical**: UnusualAmount, TrendBreak, BenfordViolation
- **Relational**: CircularTransaction, DormantAccountActivity

## Data Quality Variations

- **Missing**: MCAR, MAR, MNAR, Systematic
- **Formats**: Date (ISO/US/EU), Amount (comma/period), Identifier (case/padding)
- **Typos**: Keyboard-aware, transposition, OCR errors, homophones
- **Encoding**: Mojibake, BOM issues, HTML entities

## Export Files

| Category | Files |
|----------|-------|
| Transactions | journal_entries.csv/.json, acdoca.csv |
| Master Data | vendors, customers, materials, fixed_assets, employees, cost_centers |
| Document Flow | purchase_orders, goods_receipts, vendor_invoices, payments, sales_orders, deliveries, customer_invoices, customer_receipts, document_references |
| Subledgers | ar_*, ap_*, fa_*, inventory_* |
| Period Close | trial_balances/, accruals, depreciation, closing_entries |
| Consolidation | eliminations, currency_translation, consolidated_trial_balance |
| Labels | anomaly_labels, fraud_labels, quality_issues, quality_labels |
| Controls | internal_controls, control_*_mappings, sod_*, coso_control_mapping |
| Banking | banking_customers, bank_accounts, bank_transactions, kyc_profiles, aml_typology_labels |
| Process Mining | event_log.json (OCEL 2.0), objects.json, events.json, process_variants |
| Audit | audit_engagements, audit_workpapers, audit_evidence, audit_risks, audit_findings, audit_judgments |
| Standards | customer_contracts, performance_obligations, leases, rou_assets, lease_liabilities, fair_value_measurements, impairment_tests, isa_mappings, confirmations, audit_opinions, sox_assessments |

## Performance

~100K+ entries/second single-threaded, scales with cores, memory-efficient streaming

## Python Wrapper

```bash
cd python && pip install -e ".[all]"
```

```python
from datasynth_py import DataSynth, Config, GlobalSettings, CompanyConfig, ChartOfAccountsSettings

config = Config(
    global_settings=GlobalSettings(industry="retail", start_date="2024-01-01", period_months=12),
    companies=[CompanyConfig(code="C001", name="Retail Corp", currency="USD", country="US")],
    chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
)
result = DataSynth().generate(config=config, output={"format": "csv", "sink": "temp_dir"})
```

Blueprints: `blueprints.retail_small()`, `blueprints.banking_medium()`, `blueprints.manufacturing_large()`, `blueprints.ml_training()`, `blueprints.statistical_validation()`, `blueprints.with_distributions()`
