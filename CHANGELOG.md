# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-02-18

### Added

- **Country Pack Pluggable Architecture** (`datasynth-core`): Runtime-loaded JSON country packs replacing ~7,500 lines of hardcoded country-specific data
  - **CountryPackRegistry**: Layered merge system (`_default.json` → country pack → user overrides) with `include_str!` embedding for zero-config usage
  - **Built-in packs**: US, DE, GB with comprehensive data for holidays, names, tax rates, phone formats, addresses, payroll rules, and legal entity formats
  - **Holiday resolution**: 5 holiday types — fixed dates, Easter-relative, nth-weekday (with offset), last-weekday, and lunar calendar algorithms
  - **CountryPack schema**: 16-section JSON structure covering locale, names, holidays, tax, address, phone, banking, business rules, legal entities, accounting, payroll, vendor/customer/material templates, and document texts
  - **Deep merge**: Objects merge recursively, arrays/scalars replace — enables surgical per-country overrides
  - **Easter & lunar extraction**: Extracted algorithmic holiday computation into reusable `country/easter.rs` and `country/lunar.rs` modules
  - **External packs**: `country_packs.external_dir` config for loading custom/commercial country packs from disk

- **Generator Country Pack Integration** (`datasynth-generators`, `datasynth-banking`, `datasynth-runtime`):
  - `HolidayCalendar::from_country_pack()` — resolves all 5 holiday types with weekend observation rules; parity-tested against existing `for_region()` for US, DE, GB
  - `MultiCultureNameGenerator::from_country_pack()` — culture-weighted name generation from JSON data
  - `generate_from_country_pack()` on tax code generator — reads rates, jurisdictions, states from pack
  - `generate_with_country_pack()` on payroll generator — reads statutory deduction rates from pack
  - `spend_emission_factor_from_pack()` on emission generator — country multipliers from pack
  - `generate_phone_from_pack()`, `generate_address_from_pack()`, `generate_national_id_from_pack()` on customer generator
  - `EnhancedOrchestrator` wired with `CountryPackRegistry`, passes `&CountryPack` per company

- **Country Pack Wiring Across All Modules** (`datasynth-runtime`, `datasynth-banking`, `datasynth-ocpm`, `datasynth-graph`, `datasynth-fingerprint`):
  - **Orchestrator Phase 20 — Tax Generation**: `phase_tax_generation()` calls `TaxCodeGenerator::generate_from_country_pack()` using the primary company's country pack to produce locale-specific jurisdictions and tax codes
  - **Orchestrator Phase 21 — ESG Generation**: `phase_esg_generation()` runs the full 9-generator ESG pipeline (EmissionGenerator scope 1/2/3, EnergyGenerator, WaterGenerator, WasteGenerator, WorkforceGenerator, GovernanceGenerator, SupplierEsgGenerator, DisclosureGenerator, EsgAnomalyInjector) with country pack emission factors
  - **Banking**: `BankingOrchestrator` passes country pack to `CustomerGenerator` for locale-aware phone, address, and national ID generation
  - **Document Flows (P2P/O2C)**: Country-specific document texts (PO headers, invoice terms, payment notices) from country packs
  - **Payroll**: Localized deduction labels (tax, social security, health insurance) from country packs
  - **OCPM**: `country_code` attribute on P2P and O2C process mining events for cross-country process analysis
  - **Data Quality**: Locale-aware format variation baselines (date/number/phone formats) from country packs
  - **Graph**: `country` field on `AccountNode` for ML feature enrichment
  - **Fingerprint**: `country_code` on `SourceMetadata` for provenance tracking
  - **Presets**: Country pack awareness in all industry preset configurations
  - **Orchestrator Helpers**: `primary_pack()` and `primary_country_code()` convenience methods replacing repeated boilerplate

- **Country Pack Configuration** (`datasynth-config`): New `country_packs` section in `GeneratorConfig` with `external_dir` and `overrides` fields, validation for directory existence and override key format

- **FA/Inventory Subledger Generation** (`datasynth-runtime`): `FAGenerator` and `InventoryGenerator` wired into orchestrator subledger phase, generating fixed asset acquisition records from master data assets and inventory positions from materials

- **Payroll & Manufacturing Journal Entries** (`datasynth-runtime`): JE generation from payroll runs (DR Salaries & Wages 6100 / CR Payroll Clearing 9100) and completed production orders (DR Raw Materials 5100 / CR Inventory 1200)

- **Quality Gate Evaluation** (`datasynth-runtime`): `GateEngine::evaluate()` wired into generation result when `quality_gates.enabled` is true, resolving named profiles (strict/default/lenient) and logging pass/fail counts

- **Banking Customer Coherence** (`datasynth-runtime`): Banking customers cross-referenced with core master data, overlaying names and countries for consistent identity across modules

- **Statistics Tracking** (`datasynth-runtime`): `EnhancedGenerationStatistics` extended with `ic_transaction_count`, `fa_subledger_count`, `inventory_subledger_count`

- **Master Data Country Pack Support** (`datasynth-generators`): `set_country_pack()` method added to `VendorGenerator`, `CustomerGenerator`, and `MaterialGenerator` with orchestrator wiring

- **Generator Tracing** (`datasynth-generators`): `tracing::debug!` instrumentation added to P2P, O2C, KPI, Budget, and Sourcing Project generators for structured logging of generation parameters

- **Deterministic UUID Discriminators** (`datasynth-core`): `GeneratorType::SupplierQualification` and `GeneratorType::SupplierScorecard` discriminators added for collision-free UUID generation in sourcing module

- **`with_seed()` Constructors** (`datasynth-generators`): Standardized `with_seed(config, seed)` constructor added to `ARGenerator`, `APGenerator`, `FAGenerator`, `InventoryGenerator`, `OpeningBalanceGenerator`, `FxRateService`, and `FxRateGenerator`

### Changed

- Bumped all Rust crate versions to 0.8.0
- `NamePool` fields changed from `Vec<&'static str>` to `Vec<String>` to support JSON-deserialized name data
- Holiday `NthWeekdayHoliday` schema includes `offset_days` field for holidays like "Day after Thanksgiving"
- Executive overview document updated to v0.8.0
- `EnhancedOrchestrator` now runs 21 generation phases (previously 19), adding Tax (Phase 20) and ESG (Phase 21)
- `EnhancedGenerationResult` now includes `tax: TaxSnapshot` and `esg: EsgSnapshot` fields
- `EnhancedGenerationStatistics` tracks `tax_jurisdiction_count`, `tax_code_count`, `esg_emission_count`, `esg_disclosure_count`
- **Constructor ordering standardized** to `(config, seed)` across 16 generators in ESG, project accounting, treasury, and tax modules
- **Country pack API unified** to setter pattern; removed redundant `generate_with_country_pack()` per-call usage in orchestrator
- **Shared utilities** (`datasynth-core::utils`): `seeded_rng()` replaces `ChaCha8Rng::seed_from_u64()` in 9 generators; `sample_decimal_range()` replaces manual Decimal sampling in 4 generators; `weighted_select()` migrated across 9 generator files
- **Re-export cleanup**: Explicit type lists replace glob `pub use *` re-exports in `datasynth-core`, `datasynth-generators`, and `datasynth-graph` lib.rs files; removed `#![allow(ambiguous_glob_reexports)]`
- **Dead code removed** (round 1): Unused `active_regimes` field, `VENDOR_NAME_TEMPLATES_LEGACY` constant, `generate_address()` method, `position_counter` field, and redundant `#[allow(dead_code)]` annotation
- **Dead code removed** (round 2): Legacy trial balance builder, `GenericParquetRecord`, `BaselineComparer`, unused `ComputeStrategy` variants, dead server route field; blanket `#![allow(dead_code)]` removed from `datasynth-generators` with targeted per-item fixes; "reserved-for-future" `#[allow(dead_code)]` annotations removed from eval/core fields
- **Unused dependencies removed**: `statrs`, `nalgebra`, `askama`, `ndarray`, `plotters` from `datasynth-fingerprint` and `datasynth-eval`
- **Uuid::new_v4() replaced** with `DeterministicUuidFactory` across 16 generator files (anomaly strategies, injector, schemes, fraud collusion/management override, relationships, counterfactual, data quality labels, error cascade) for full deterministic reproducibility
- **`AccountingStandardsSnapshot`** now persists actual `Vec<CustomerContract>` and `Vec<ImpairmentTest>` data (previously only stored counts)
- **Output pipeline expanded** (`datasynth-cli`, `datasynth-runtime`): Treasury (6 generators), project accounting (5 generators), tax generators, period-close generators, accounting standards, internal controls, banking AML labels, IC journal entries, period-close trial balances, process mining events, and graph export warnings all wired to output writer with manifest registration
- **Quality gate evaluation** now receives actual balance sheet evaluation, coherence pass/fail, and statistical/quality scores instead of stub data
- **KPIs derived from actual data**: Gross Margin, Operating Margin, and Current Ratio overridden with values computed from generated financial statements
- **Sourcing project cross-references**: `rfx_ids`, `contract_id`, and `spend_analysis_id` back-populated from generated data after sourcing generation completes
- **Audit team linked to real employees**: `engagement_partner_id`, `engagement_manager_id`, and `team_member_ids` replaced with actual employee IDs from master data
- **`created_by` field** rotated across employees via round-robin in P2P/O2C document flow loops (previously always used first employee)
- **OCPM UUID generation** centralized via `DeterministicUuidFactory` with sub-discriminators, replacing hand-rolled FNV-1a hash and `Uuid::new_v4()` calls
- **Banking seed offsets** replaced with named constants for deterministic sub-generator seeding

### Fixed

- **Account numbering unified** across document flow JEs (`DocumentFlowJeConfig`), CoA generator, and `accounts.rs` constants — all modules now reference the same 4-digit account codes
- **Trial balance** derived from actual JE data by aggregating debit/credit amounts per account, replacing hardcoded document-flow aggregates
- **Financial statements** derived from JE trial balances with proper cumulative balance sheet accounts and comparative prior-period amounts; cash flow statement built via indirect method from working capital changes
- **CLI output pipeline** now exports all generated data (master data, document flows, subledgers, financial statements, controls, banking, process mining, audit, standards) via `datasynth-output` sinks, replacing the truncated `sample_entries.json`
- **Intercompany module** wired into orchestrator with IC transaction generation, matching engine, and elimination entries
- **OCPM event log** persisted to output directory as `event_log.json` (OCEL 2.0 format)
- **`InjectorStats` fields** made public for external consumers
- **`CountryPack` clone** eliminated in payroll generator hot path via `as_ref()` borrowing
- **E2e tests** updated to expect `journal_entries.json` output filename
- **AP three-way match variance**: Price and quantity variances now computed as ~3% and ~1.5% of line total respectively when `ThreeWayMatchFailed`, instead of hardcoded `Decimal::ZERO`
- **Subledger vendor/customer names**: `DocumentFlowLinker` now receives vendor and customer name maps from master data, replacing placeholder `"Vendor {id}"` / `"Customer {id}"` strings with actual generated names
- **DocumentFlowJeGenerator seed**: `with_config()` constructor no longer uses hardcoded seed; accepts seed parameter for deterministic generation

## [0.7.0] - 2026-02-17

### Added

- **Tax Accounting Domain** (`datasynth-core`, `datasynth-generators`, `datasynth-config`, `datasynth-output`): Complete tax lifecycle simulation
  - **Data Models**: `TaxJurisdiction` (Federal/State/Local/Municipal/Supranational), `TaxCode` with effective date ranges, `TaxLine` (attached to VI/CI/JE/Payment/PayrollRun), `TaxReturn` (VAT/GST/Income/Withholding/Payroll), `TaxProvision` (ASC 740/IAS 12 with deferred tax tracking), `UncertainTaxPosition` (FIN 48/IFRIC 23), `WithholdingTaxRecord` (cross-border with treaty benefits), `RateReconciliationItem`
  - **Generators**: Tax code generator, tax line decorator (attaches tax lines to source documents), tax return aggregation by jurisdiction/period, ASC 740/IAS 12 provision computation, withholding tax on cross-border payments, tax anomaly injector with ground truth labels
  - **Configuration**: `TaxConfig` with sub-sections for jurisdictions, VAT/GST (standard/reduced rates, exempt categories, reverse charge), US sales tax by nexus states, withholding (treaty network & rates), provisions (statutory rates & uncertain positions), payroll tax
  - **Output**: 9 CSV files — `tax_jurisdictions`, `tax_codes`, `tax_lines`, `tax_returns`, `tax_provisions`, `rate_reconciliation`, `uncertain_tax_positions`, `withholding_records`, `tax_anomaly_labels`

- **Treasury & Cash Management Domain** (`datasynth-core`, `datasynth-generators`, `datasynth-config`, `datasynth-output`): Full treasury operations simulation
  - **Data Models**: `CashPosition` (daily balance by entity/account/currency), `CashForecast`/`CashForecastItem` (probability-weighted), `CashPool`/`CashPoolSweep` (physical/notional/zero-balance), `HedgingInstrument` (FX forwards, IR swaps, options), `HedgeRelationship` (ASC 815/IFRS 9 effectiveness testing), `DebtInstrument` (loans, bonds, credit facilities), `AmortizationPayment`, `DebtCovenant` (Debt/Equity, Interest Coverage), `BankGuarantee`, `NettingRun`/`NettingPosition`
  - **Generators**: Cash position aggregation from payment flows, probability-weighted cash forecasts, cash pool sweep generation, hedging instruments with hedge designations, debt instruments with covenants & amortization, treasury anomaly injector
  - **Configuration**: `TreasuryConfig` with sub-sections for cash positioning, forecasting, pooling, hedging (FX/IR instruments, effectiveness methods), debt (term loans, bonds, revolving facilities), netting (multilateral settlement), bank guarantees
  - **Output**: 13 CSV files — `cash_positions`, `cash_forecasts`, `cash_forecast_items`, `cash_pool_sweeps`, `hedging_instruments`, `hedge_relationships`, `debt_instruments`, `debt_covenants`, `amortization_schedules`, `bank_guarantees`, `netting_runs`, `netting_positions`, `treasury_anomaly_labels`

- **Project Accounting Domain** (`datasynth-core`, `datasynth-generators`, `datasynth-config`, `datasynth-output`): End-to-end project cost and revenue management
  - **Data Models**: `ProjectCostLine` (Labor/Material/Subcontractor/Overhead/Equipment/Travel), `ProjectRevenue` (percentage-of-completion / ASC 606 with unbilled tracking), `ProjectMilestone` (payment milestones with status), `ChangeOrder` (scope/cost/schedule changes), `Retainage` (payment holds and releases), `EarnedValueMetrics` (BCWS/BCWP/ACWP/SPI/CPI/EAC/ETC/TCPI)
  - **Generators**: Project creation with WBS hierarchies, cost linking from time entries/expenses/POs, PoC revenue recognition, EVM metrics computation, change order generation
  - **Configuration**: `ProjectAccountingConfig` with project types distribution (Capital 25%, Internal 20%, Customer 30%, R&D 10%, Maintenance 10%, Technology 5%), WBS depth/elements, cost allocation from source documents, revenue recognition (PoC, cost-to-cost), milestones, change orders, retainage, earned value
  - **Output**: 9 CSV files — `projects`, `wbs_elements`, `project_cost_lines`, `project_revenue`, `project_milestones`, `change_orders`, `retainage`, `earned_value_metrics`, `project_accounting_anomaly_labels`

- **ESG / Sustainability Domain** (`datasynth-core`, `datasynth-generators`, `datasynth-config`, `datasynth-output`): Comprehensive environmental, social, and governance data
  - **Environmental Models**: `EmissionRecord` (GHG Protocol Scope 1/2/3, CO2e tonnes), `EnergyConsumption` (by type with renewable flag), `WaterUsage` (withdrawal/discharge/consumption), `WasteRecord` (by type and disposal method)
  - **Social Models**: `WorkforceDiversityMetric` (Gender/Ethnicity/Age/Disability/Veteran by org level), `PayEquityMetric` (pay gap analysis), `SafetyIncident` (Injury/Illness/NearMiss/Fatality), `SafetyMetric` (TRIR/LTIR/DART rates)
  - **Governance Models**: `GovernanceMetric` (board composition, independence, diversity, ethics), `EsgDisclosure` (GRI/SASB/TCFD frameworks), `SupplierEsgAssessment` (supply chain ratings), `ClimateScenarioAnalysis`
  - **Generators**: Scope 1/2/3 emission generator (activity-based & spend-based derivation), energy consumption tracker with renewable targets, workforce diversity & pay equity analysis, ESG disclosure/assurance generator, supplier ESG assessment, ESG anomaly injector
  - **Configuration**: `EsgConfig` with environmental (emissions by scope, energy, water, waste), social (diversity, pay equity, safety), governance, supply chain ESG, reporting (GRI/SASB/TCFD), climate scenario analysis
  - **Output**: 13 CSV files — `emission_records`, `energy_consumption`, `water_usage`, `waste_records`, `workforce_diversity_metrics`, `pay_equity_metrics`, `safety_incidents`, `safety_metrics`, `governance_metrics`, `esg_disclosures`, `supplier_esg_assessments`, `climate_scenarios`, `esg_anomaly_labels`

- **OCPM Domain Expansion** (`datasynth-ocpm`): Process mining extended from 8 to 12 process families
  - **Tax**: 2 object types (`tax_line`, `tax_return`), activities for determination, filing, assessment, payment
  - **Treasury**: 4 object types (`cash_position`, `cash_forecast`, `hedge_instrument`, `debt_instrument`), activities for calculation, forecasting, designation, issuance
  - **Project Accounting**: 4 object types (`project`, `project_cost_line`, `project_milestone`, `change_order`), activities for creation, cost posting, milestone completion, change approval
  - **ESG**: 3 object types (`esg_data_point`, `emission_record`, `esg_disclosure`), activities for collection, calculation, submission
  - Total: 101+ activities across 12 process families, 65+ object types

- **Industry-Specific Presets** (`datasynth-config`): New presets for Tax, Treasury, Project Accounting, and ESG domains integrated into all industry configurations

- **BusinessProcess Enum Extensions** (`datasynth-core`): Added `Tax`, `Treasury`, `ProjectAccounting`, `Esg` variants for process classification

### Changed

- Bumped all Rust crate versions to 0.7.0
- Python wrapper version bumped to 1.3.0 with new domain configuration models
- OCPM event log now covers 12 process families (P2P, O2C, S2C, H2R, MFG, R2R, BANK, AUDIT, Tax, Treasury, ProjectAccounting, ESG)
- Executive overview document updated to v0.7.0 with new domain descriptions

## [0.6.2] - 2026-02-13

### Added

- **Desktop UI Enhancements** (`datasynth-ui`): Visual polish and UX improvements across the configuration interface
  - **Info cards on 6 config pages**: relationship-strength (3 cards), cross-process-links (3 cards), vendor-network (4 cards), customer-segmentation (4 cards), data-quality (4 cards), accounting-standards (4 cards) — always visible regardless of feature toggle state
  - **Sidebar scroll indicator**: Animated bounce chevron appears when nav content overflows, hides when scrolled to bottom; collapsed Specialized section by default for better discoverability
  - **Dashboard web-mode fallback**: Detects Tauri runtime availability via `window.__TAURI__`; shows friendly "Web Preview Mode" placeholder with setup instructions instead of raw TypeError in web-only contexts
  - **56 visual regression baselines regenerated** with Playwright; 272 functional tests pass

- **Universal OCPM Generation** (`datasynth-ocpm`): Extended process mining from 2 to 8 process families with 88 total activities and 52 object types
  - **S2C Generator**: Source-to-Contract process events — sourcing projects, supplier qualification, RFx, bids, evaluations, contracts (8 activities, 6 object types)
  - **H2R Generator**: Hire-to-Retire process events — payroll runs, time entries, expense reports with approval chains (8 activities, 4 object types)
  - **MFG Generator**: Manufacturing process events — production orders, routing operations, quality inspections, cycle counts (10 activities, 4 object types)
  - **BANK Generator**: Banking operations process events — customer onboarding, KYC, account management, transaction lifecycle (8 activities, 3 object types)
  - **AUDIT Generator**: Audit engagement lifecycle events — planning, risk assessment, workpapers, evidence, findings, judgments (10 activities, 6 object types)
  - **Bank Recon Generator**: Bank reconciliation process events — statement import, auto/manual matching, exception resolution (8 activities, 3 object types)
  - All generators support three variant types: HappyPath (75%), ExceptionPath (20%), ErrorPath (5%)
  - Per-family config toggles: `generate_s2c`, `generate_h2r`, `generate_mfg`, `generate_bank`, `generate_audit`, `generate_bank_recon`

- **Expanded Hypergraph Builder** (`datasynth-graph`): Extended from P2P/O2C to all 8 process families with 24 new entity type codes
  - `add_s2c_documents()`: Sourcing project, RFx, bid, contract nodes with intra-chain edges
  - `add_h2r_documents()`: Payroll, time entry, expense report nodes linked to employees
  - `add_mfg_documents()`: Production order, quality inspection, cycle count nodes linked to materials
  - `add_bank_documents()`: Banking customer, account, transaction nodes
  - `add_audit_documents()`: Engagement, workpaper, finding, evidence, risk, judgment nodes
  - `add_bank_recon_documents()`: Reconciliation, statement line, reconciling item nodes
  - `add_ocpm_events()`: OCPM events as hyperedges connecting all participating object nodes (entity type 400)
  - Per-family config toggles: `include_s2c`, `include_h2r`, `include_mfg`, `include_bank`, `include_audit`, `include_r2r`
  - Entity type code ranges: S2C (320-325), H2R (330-333), MFG (340-343), BANK (350-352), AUDIT (360-365), Bank Recon (370-372), OCPM Events (400)

- **BusinessProcess Enum Extensions** (`datasynth-core`): Added `S2C`, `Mfg`, `Bank`, `Audit` variants for process classification

- **Orchestrator Phase Reordering** (`datasynth-runtime`): OCPM generation moved to Phase 18b (after all data generation) and hypergraph export to Phase 19b (after OCPM) for correct data flow

### Changed

- Bumped all Rust crate versions to 0.6.2
- OCPM event log now contains events from all 8 process families (P2P, O2C, S2C, H2R, MFG, R2R, BANK, AUDIT)
- Hypergraph `nodes.jsonl` includes entity types 100-400 spanning all process families
- Hypergraph `hyperedges.jsonl` includes OCPM events when `events_as_hyperedges: true`
- `ProcessLayerSettings` extended with 6 new per-family toggle fields
- `HypergraphConfig` extended with matching per-family toggle fields
- `OcpmGeneratorConfig` extended with per-family generation toggles
- Comprehensive eval test marked `#[ignore]` to exclude from normal `cargo test` runs (run with `--ignored` flag)

### Fixed

- **Magenta footer bar** (`datasynth-ui`): Added explicit `background-color` to `.app-footer`, `.main-area`, and `html`/`body` to prevent color bleed-through in full-page screenshots
- **Dashboard raw error display** (`datasynth-ui`): Server error string now hidden behind a `<details>` collapse instead of shown inline
- **Cross-process-links info cards** (`datasynth-ui`): Moved info cards outside `{#if enabled}` block so they are visible when the feature is disabled
- CI pipeline: Added separate `eval` job for comprehensive evaluation test (runs only on main pushes, 30-min timeout)

## [0.6.1] - 2026-02-13

### Added

- **Comprehensive Evaluation Framework** (`datasynth-eval`): 23 new evaluator modules providing end-to-end quality assessment
  - **Statistical Evaluators**: Anomaly realism scoring, drift detection with labeled events
  - **Coherence Evaluators**: Audit trail validation, bank reconciliation accuracy, cross-process consistency, financial reporting tie-back, HR/payroll gross-to-net verification, manufacturing yield/sequence validation, sourcing pipeline completion
  - **ML Readiness Evaluators**: Anomaly scoring analysis, cross-modal consistency, domain gap measurement, embedding readiness, feature quality assessment, GNN readiness, scheme detectability, temporal fidelity
  - **Banking Evaluators**: KYC completeness analysis, AML typology detectability
  - **Process Mining Evaluators**: OCEL 2.0 event sequence validation, process variant analysis
  - **Causal & Enrichment Evaluators**: Causal model validation, LLM enrichment quality assessment
  - **Enhancement Engine**: Auto-tuner generating config patches from evaluation gaps, recommendation engine with root cause analysis and prioritized actions

- **Quality Gate Engine Improvements** (`datasynth-eval`): Extended from 8 to 17 quality metrics
  - New gates: HR payroll accuracy, manufacturing yield, bank reconciliation balance, KYC completeness, AML detectability, process mining coverage, audit evidence coverage, sourcing completion
  - Balance coherence gate now uses rate-based scoring (balanced entries / total entries) instead of binary pass/fail

- **KYC Data Generation** (`datasynth-banking`): Customer generator now populates all KYC-required fields
  - Address fields (address_line1, city, state, postal_code) for all customer types
  - Identity documents (national_id, passport_number) for retail customers
  - Beneficial owners with control type and verification status for business and trust customers

- **Bank Reconciliation Generation** (`datasynth-runtime`): Payment-to-statement matching
  - Groups payments by company code and fiscal period
  - Generates monthly reconciliations for each company (24 reconciliations for 2-company, 12-month setup)

- **Snapshot Data Expansion** (`datasynth-runtime`): HR, Manufacturing, and Sales/KPI/Budget snapshots now store actual model instances
  - `HrSnapshot`: payroll_runs, payroll_line_items, time_entries, expense_reports
  - `ManufacturingSnapshot`: production_orders, quality_inspections, cycle_counts
  - `SalesKpiBudgetsSnapshot`: sales_quotes, kpis, budgets

- **Comprehensive Integration Test** (`datasynth-runtime`): Full end-to-end evaluation test
  - Generates data with all enterprise process chains enabled
  - Wires generated data into all 17 evaluator modules
  - Runs quality gates across lenient/default/strict profiles
  - Auto-tuner recommendations and enhancement report
  - 17/17 feature coverage (100%), lenient gates 9/9 PASS

### Fixed

- **Temporal Pattern Correlation Bug** (`datasynth-eval`): Fixed month-end multiplier in expected pattern calculation
  - `MONTH_END_SPIKE / 2.5` evaluated to 1.0 (no effect) — now correctly applies `MONTH_END_SPIKE` (2.5x)
  - Pattern correlation improved from ~0.37 to ~0.78

- **Financial Reporting BS Equation** (`datasynth-eval`): Fixed balance sheet equation validation
  - Financial statements now correctly use total line codes (BS-TA, BS-TL, BS-TE) instead of summing all line items

- **Manufacturing Operation Sequence** (`datasynth-eval`): Fixed timestamp-based ordering
  - Uses operation_number offset to distinguish operations sharing the same date

- **Audit Evaluator Mapping** (`datasynth-eval`): Fixed evidence-to-finding and workpaper cross-reference detection
  - Falls back to engagement-level evidence when `evidence_refs` is empty
  - Uses account_ids and evidence_refs as proxy for workpaper cross-references

### Changed

- Bumped all Rust crate versions to 0.6.1
- Extended `PhaseConfig` with flags for all enterprise generators (audit, banking, sourcing, manufacturing, etc.)
- `ComprehensiveEvaluation` extended with optional banking, process_mining, causal, and enrichment_quality fields

## [0.6.0] - 2026-02-12

### Added

- **Enterprise Process Chain Extensions**: 8 new enterprise process chains spanning 4 implementation waves with 18+ new models and 18+ new generators

- **Source-to-Contract (S2C) Pipeline** (`datasynth-core`, `datasynth-generators`): Complete sourcing lifecycle
  - `SourcingProject`, `SupplierQualification`, `RfxEvent`, `SupplierBid`, `BidEvaluation`, `ProcurementContract`, `CatalogItem`, `SupplierScorecard` models
  - `SpendAnalysis` with vendor spend shares and HHI concentration
  - Full generation DAG: spend analysis → sourcing project → qualification → RFx → bid → evaluation → contract → catalog → scorecard
  - P2P integration: contract-based PO creation, maverick spend tracking, contract utilization
  - Three-way match extended with contract price variance checking
  - 12 new export files: `sourcing_projects.csv`, `rfx_events.csv`, `supplier_bids.csv`, `procurement_contracts.csv`, `catalog_items.csv`, `supplier_scorecards.csv`, and more

- **Bank Reconciliation** (`datasynth-generators`): Automated bank statement matching
  - `BankReconciliation`, `BankStatementLine`, `ReconcilingItem` models with match status tracking
  - Auto-match cleared payments to statement lines (configurable 90% rate)
  - Outstanding checks, deposits in transit, and bank-only lines (fees, interest)
  - Net difference validation ensuring reconciliation balances to zero
  - 3 new export files: `bank_statement_lines.csv`, `bank_reconciliations.csv`, `reconciling_items.csv`

- **Financial Statements** (`datasynth-generators`): Complete financial reporting suite
  - `FinancialStatement`, `FinancialStatementLineItem`, `CashFlowItem` models
  - Balance Sheet, Income Statement, Cash Flow Statement, Changes in Equity generation
  - GL account-to-statement line item mapping via account type classification
  - BS equation enforcement (Assets = Liabilities + Equity)
  - Indirect cash flow method (Net Income + non-cash adjustments ± working capital changes)
  - 4 new export files: `balance_sheet.csv`, `income_statement.csv`, `cash_flow_statement.csv`, `changes_in_equity.csv`

- **Hire-to-Retire (H2R) — Payroll, Time & Attendance, Expenses** (`datasynth-core`, `datasynth-generators`): Full HR lifecycle
  - `PayrollRun`, `PayrollLineItem` with earnings, deductions, and employer tax calculations
  - `TimeEntry` with regular, overtime, and leave tracking with approval workflow
  - `ExpenseReport`, `ExpenseLineItem` with category-based amounts and policy violation detection
  - Payroll journal entry generation (DR Salary Expense, CR Payroll Payable, CR Tax Withholding)
  - Overtime correlated with period-end dynamics
  - 5 new export files: `payroll_runs.csv`, `payslips.csv`, `time_entries.csv`, `expense_reports.csv`, `expense_line_items.csv`

- **Revenue Recognition** (`datasynth-generators`): ASC 606/IFRS 15 contract generation
  - `RevenueRecognitionGenerator` creating `CustomerContract` and `PerformanceObligation` records
  - Single and multi-element arrangements with standalone selling price allocation
  - Linked to O2C sales orders via `sales_order_id`

- **Impairment Testing** (`datasynth-generators`): Asset impairment workflow
  - `ImpairmentGenerator` selecting assets for testing based on risk indicators
  - Fair value estimation with random walk from carrying amount
  - Impairment loss calculation and journal entry generation
  - Technology assets at higher impairment risk (2x multiplier)

- **Manufacturing** (`datasynth-core`, `datasynth-generators`): Production order lifecycle
  - `ProductionOrder` with routing operations, component issues, and WIP tracking
  - `QualityInspection` with inspection characteristics and pass/fail results
  - `CycleCount` with book-to-physical variance tracking and ABC classification
  - BOM explosion for component requirements
  - Production variances (material, labor, overhead)
  - 4 new export files: `production_orders.csv`, `routing_operations.csv`, `quality_inspection_lots.csv`, `cycle_count_records.csv`

- **Sales Quotes** (`datasynth-core`, `datasynth-generators`): Quote-to-order pipeline
  - `SalesQuote`, `QuoteLineItem` with configurable win rate and validity periods
  - Quote-to-sales-order conversion tracking
  - Pricing 5-15% above final order price with negotiation modeling

- **Management KPIs** (`datasynth-core`, `datasynth-generators`): Financial ratio computation
  - `ManagementKpi` with category classification (liquidity, profitability, efficiency, leverage)
  - Monthly or quarterly frequency with trend tracking
  - Derived from financial statement line items

- **Budgets** (`datasynth-core`, `datasynth-generators`): Budget variance analysis
  - `Budget`, `BudgetLineItem` with GL account-level budget vs actual tracking
  - Prior year actuals × (1 + growth_rate) with configurable noise
  - Variance calculation with favorable/unfavorable classification

- **New Configuration Sections** (`datasynth-config`): All defaulting to `enabled: false`
  - `source_to_pay`: S2C sourcing pipeline configuration (spend analysis, qualification, RFx, contracts, catalogs, scorecards, P2P integration)
  - `financial_reporting`: Financial statements, management KPIs, and budgets
  - `hr`: Payroll (pay frequency, salary ranges, tax rates), time & attendance (overtime rate), expenses (submission rate, policy violations)
  - `manufacturing`: Production orders (batch size, yield rate, rework), costing (labor/overhead rates), routing (operations, setup time)
  - `sales_quotes`: Quote generation (quotes/month, win rate, validity days)

- **Industry Preset Updates** (`datasynth-config`): All 5 industry presets updated with new process chain defaults
  - Manufacturing: Quality-focused sourcing, full production order support
  - Retail: Price-focused sourcing, high-volume sales quotes
  - Financial Services: Compliance-focused sourcing, conservative budgets
  - Healthcare and Technology: Appropriate defaults for each new process chain

- **UUID Factory Extensions** (`datasynth-core`): 12 new generator type discriminators
  - `SourcingProject` (0x28), `RfxEvent` (0x29), `SupplierBid` (0x2A), `ProcurementContract` (0x2B), `CatalogItem` (0x2C)
  - `BankReconciliation` (0x2D), `FinancialStatement` (0x2E)
  - `PayrollRun` (0x2F), `TimeEntry` (0x30), `ExpenseReport` (0x31)
  - `ProductionOrder` (0x32), `CycleCount` (0x33), `QualityInspection` (0x34)
  - `SalesQuote` (0x35), `BudgetLine` (0x36), `RevenueRecognition` (0x37), `ImpairmentTest` (0x38), `Kpi` (0x39)

- **Orchestrator Phases** (`datasynth-runtime`): 4 new generation phases
  - Phase 16: HR Data (payroll runs, time entries, expense reports)
  - Phase 17: Accounting Standards (revenue recognition contracts, impairment tests)
  - Phase 18: Manufacturing (production orders, quality inspections, cycle counts)
  - Phase 19: Sales Quotes, Management KPIs, and Budgets

### Changed

- Bumped all Rust crate versions to 0.6.0
- `GeneratorConfig` extended with `source_to_pay`, `financial_reporting`, `hr`, `manufacturing`, `sales_quotes` fields (all `#[serde(default)]`)
- `GeneratorType` enum extended from 0x27 to 0x39 (18 new discriminators)
- All new model structs use `Option<>` fields and `#[serde(default)]` for backward compatibility
- Existing YAML configs parse without errors (all new sections have defaults)

## [0.5.0] - 2026-02-11

### Added

- **LLM-Augmented Generation** (`datasynth-core`, `datasynth-generators`): Opt-in LLM-powered metadata enrichment
  - `LlmProvider` trait with `MockProvider` (deterministic) and `HttpProvider` (external API) implementations
  - `LlmEnrichmentEngine`: Enriches vendor names, transaction descriptions, anomaly explanations, and memo fields
  - Natural language configuration: Generate YAML configs from plain English descriptions
  - `LlmConfig` with provider selection, model, caching, and per-field enrichment toggles
  - Response caching with deterministic fallback for reproducible generation

- **Diffusion Model Integration** (`datasynth-core`): Statistical diffusion backend for learned distribution capture
  - `DiffusionBackend` trait with `StatisticalDiffusionBackend` implementation
  - Noise schedules: cosine, linear, sigmoid with configurable step count
  - Hybrid generation mode combining rule-based and diffusion outputs with configurable weight
  - Training pipeline: `DiffusionTrainer` with epoch, batch size, and learning rate configuration
  - `DiffusionConfig` with backend selection, noise schedule, hybrid mode, and training settings

- **Advanced Privacy: Federated Fingerprinting** (`datasynth-fingerprint`): Distributed fingerprint extraction
  - `FederatedFingerprintCoordinator`: Aggregate fingerprints from distributed data sources without centralization
  - Synthetic data certificates: `SyntheticDataCertificate` with cryptographic proof of DP guarantees
  - Privacy-utility Pareto frontier: `ParetoFrontier` for automated exploration of optimal epsilon values
  - Certificate validation and verification workflow

- **Causal & Counterfactual Generation** (`datasynth-core`): Causal modeling for what-if scenarios
  - `CausalGraph`: DAG specification of causal relationships between entities
  - `StructuralCausalModel` (SCM): Functional causal model with noise terms
  - `Intervention`: Do-calculus interventions for hypothetical scenario generation
  - `CounterfactualGenerator`: Generate counterfactual versions of existing records
  - `CausalValidator`: Validate that generated data preserves specified causal structure
  - Template causal graphs: fraud_detection, revenue_impact, supply_chain

- **Ecosystem Integrations** (`python/datasynth_py/integrations`): Pipeline and workflow integrations
  - `DataSynthOperator` / `DataSynthSensor` / `DataSynthValidateOperator`: Apache Airflow operators for orchestrated generation
  - `DbtSourceGenerator`: Generate dbt source YAML and seed files from DataSynth output
  - `DataSynthMlflowTracker`: Track generation runs as MLflow experiments with quality metrics
  - `DataSynthSparkReader`: Read DataSynth output directly as Spark DataFrames
  - Lazy imports to avoid requiring dependencies at import time

- **Python Phase 4 Blueprints** (`python/datasynth_py`): New blueprint variants
  - `with_llm_enrichment()`: Add LLM enrichment overlay to any base config
  - `with_diffusion()`: Add diffusion model enhancement with hybrid mode
  - `with_causal()`: Add causal generation with interventions and counterfactuals
  - `Config.llm`, `Config.diffusion`, `Config.causal` optional dict fields for Phase 4 config sections

- **JWT Validation & OIDC Support** (`datasynth-server`): Token-based authentication behind `jwt` feature flag
  - RS256 JWT validation via `jsonwebtoken` crate with issuer/audience verification
  - OIDC provider support (Keycloak, Auth0, Entra ID) via `--jwt-issuer`, `--jwt-audience`, `--jwt-public-key` CLI args
  - Bearer token flow: JWT validated first, falls back to API key if JWT feature disabled
  - `JwtConfig`, `TokenClaims`, `JwtValidator` structs with full test coverage

- **Role-Based Access Control** (`datasynth-server`): RBAC with structured audit logging
  - `Role` enum: Admin, Operator (default), Viewer with 7 permission types
  - `Permission` matrix: GenerateData, ManageJobs, ViewJobs, ManageConfig, ViewConfig, ViewMetrics, ManageApiKeys
  - `RolePermissions::has_permission()` for middleware-level authorization
  - `--rbac-enabled` CLI flag for opt-in activation

- **Structured Audit Logging** (`datasynth-server`): JSON audit event trail
  - `AuditEvent` struct with actor, action, resource, outcome, and correlation ID
  - `AuditLogger` trait with `JsonAuditLogger` (via `tracing::info`) and `NoopAuditLogger`
  - `--audit-log` CLI flag for opt-in activation

- **gRPC Authentication Interceptor** (`datasynth-server`): Token validation for gRPC endpoints
  - `GrpcAuthConfig` with `new(api_keys)` and `disabled()` constructors
  - `auth_interceptor()` function extracting Bearer tokens from `authorization` metadata
  - `X-API-Version: v1` response header injected on all REST responses

- **Quality Gate Engine** (`datasynth-eval`): Configurable pass/fail thresholds for generation quality
  - `GateEngine::evaluate()` extracts 8 metrics from `ComprehensiveEvaluation`
  - Metrics: BenfordMad, BalanceCoherence, DocumentChainCompleteness, DuplicateRate, MissingRate, DistributionFit, CorrelationAccuracy, AnomalyPrecision
  - Built-in profiles: `strict`, `default`, `lenient` with per-metric thresholds
  - `GateResult` with pass/fail, metric details, and failed gate list
  - CLI: `--quality-gate <none|lenient|default|strict>` with exit code 2 on failure
  - Config: `quality_gates` section with profile, custom gates, and `fail_on_violation`

- **Plugin SDK** (`datasynth-core`): Extensible trait-based plugin system
  - `GeneratorPlugin` trait: `generate(context) -> Vec<GeneratedRecord>`
  - `SinkPlugin` trait: `open()`, `write_batch()`, `close() -> SinkSummary`
  - `TransformPlugin` trait: `transform(records) -> Vec<GeneratedRecord>`
  - `PluginRegistry`: Thread-safe `Arc<RwLock<...>>` registry for all plugin types
  - `PluginInfo` struct with name, version, and description
  - Example plugins: `CsvEchoSink` and `TimestampEnricher`

- **Webhook Notifications** (`datasynth-runtime`): Fire-and-forget event dispatch
  - `WebhookEvent` enum: RunStarted, RunCompleted, RunFailed, GateViolation
  - `WebhookPayload` with event type, run ID, timestamp, and extensible detail map
  - `WebhookEndpoint` with URL, event filter, optional HMAC secret, retry, and timeout
  - `WebhookDispatcher` with endpoint matching and payload factory methods
  - Config: `webhooks` section with enabled flag and endpoint list

- **Async Python Client** (`python/datasynth_py`): Non-blocking generation via asyncio
  - `AsyncDataSynth` with async context manager, `generate()`, `stream_generate()`, `validate_config()`
  - `StreamEvent` dataclass for real-time generation progress
  - Uses `asyncio.create_subprocess_exec` for subprocess-based execution

- **DataFrame Integration** (`python/datasynth_py`): Direct DataFrame loading
  - `to_pandas(result)`: Load CSV tables into pandas DataFrames (comment="#" for synthetic markers)
  - `to_polars(result)`: Load CSV tables into polars DataFrames (comment_prefix="#")
  - `list_tables(result)`: Enumerate available output tables with subdirectory support

- **EU AI Act Compliance** (`datasynth-core`): Article 50 synthetic content marking
  - `SyntheticContentMarker` with `create_credential()` and config hashing
  - `ContentCredential` struct with generator, version, timestamp, config hash, and marking format
  - `MarkingFormat` enum: Embedded (default), Sidecar, Both
  - Article 10 data governance: `DataGovernanceReport` with `BiasAssessment`
  - Config: `compliance` section with `content_marking` and `article10_report` settings

- **Compliance Documentation** (`docs/src/compliance/`): Regulatory framework guides
  - EU AI Act: Article 50 content marking and Article 10 governance report usage
  - NIST AI RMF: Self-assessment across MAP, MEASURE, MANAGE, GOVERN functions
  - GDPR: Article 30 record templates, DPIA guidance, data minimization
  - SOC 2 Type II: Readiness assessment across 5 Trust Service Criteria with controls mapping
  - ISO 27001:2022: Annex A alignment with 11 implemented, 6 partial, and 8 N/A controls

- **Python 1.0.0 Release** (`python/`): Production-stable Python wrapper
  - Version bumped to 1.0.0 with "Production/Stable" classifier
  - CHANGELOG.md documenting all features, config models, and optional dependencies

- **CI/CD Hardening** (`.github/workflows/`): Expanded from single-job to 7-job CI pipeline
  - `fmt`: `cargo fmt --check`
  - `clippy`: Lint with `-D warnings`
  - `test`: Cross-platform test matrix (Ubuntu, macOS, Windows)
  - `msrv`: Minimum supported Rust version validation (1.88)
  - `security`: `cargo deny check` + `cargo audit` for CVE and license compliance
  - `coverage`: `cargo-llvm-cov` with Codecov integration
  - `benchmarks`: Criterion regression check on PRs

- **Dependency Auditing** (`deny.toml`): cargo-deny policy for license, advisory, bans, and source auditing
  - Denies known vulnerabilities, warns on unmaintained/yanked crates
  - Allows MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode, OpenSSL, BSL-1.0, MPL-2.0
  - Denies copyleft licenses, unknown registries, and git sources

- **Automated Dependency Updates** (`.github/dependabot.yml`): Dependabot for cargo, pip, and GitHub Actions dependencies

- **Release Automation** (`.github/workflows/release.yml`): Full release pipeline on `v*` tags
  - Draft GitHub Release with git-cliff changelog generation
  - Pre-built binaries for 5 platforms: x86_64-linux, aarch64-linux, x86_64-macos, aarch64-macos, x86_64-windows
  - Docker image build + push to GHCR (linux/amd64 + linux/arm64)
  - Trivy container security scanning with SARIF upload

- **Benchmark Tracking** (`.github/workflows/benchmarks.yml`): Criterion benchmark results tracked on main pushes

- **Security Headers Middleware** (`datasynth-server`): Injects security response headers on all responses
  - `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `X-XSS-Protection: 0`
  - `Referrer-Policy: strict-origin-when-cross-origin`, `Content-Security-Policy: default-src 'none'`
  - `Cache-Control: no-store` for API responses

- **Request Validation Middleware** (`datasynth-server`): Content-Type enforcement for mutation requests
  - POST/PUT/PATCH with body must include `Content-Type: application/json` (returns 415 otherwise)
  - GET/DELETE/OPTIONS bypass Content-Type check

- **Request ID Middleware** (`datasynth-server`): X-Request-Id header propagation
  - Preserves client-sent request IDs or generates UUID v4
  - Available in request extensions for downstream middleware (logging, tracing)

- **Environment Variable Interpolation** (`datasynth-config`): `${ENV_VAR}` and `${ENV_VAR:-default}` support in YAML configs
  - Regex-based preprocessing before YAML parsing
  - Errors on unset variables without defaults

- **TLS Support** (`datasynth-server`): Optional rustls TLS behind `tls` feature flag
  - `--tls-cert` and `--tls-key` CLI arguments
  - Uses `axum-server` with rustls backend

- **Observability Stack** (`datasynth-server`): Feature-gated OpenTelemetry integration
  - `otel` feature flag enables OTLP trace export and Prometheus metric bridge
  - `ServerMetrics` struct with AtomicU64 counters/gauges and `DurationTimer` utility
  - Structured JSON logging via `tracing-subscriber` registry with `EnvFilter`
  - Request logging middleware with method, path, status, latency_ms, request_id spans

- **Prometheus Alert Rules** (`deploy/prometheus-alerts.yml`): Example alerting rules
  - HighErrorRate, HighLatency, HighMemoryUsage, ServerDown, NoEntitiesGenerated

- **Docker Support**: Multi-stage container builds for server and CLI
  - `Dockerfile`: cargo-chef dependency caching + distroless runtime with both server and CLI binaries
  - `Dockerfile.cli`: Slim CLI-only variant
  - `.dockerignore`: Proper context exclusion

- **Docker Compose Stack** (`docker-compose.yml`): Local development stack
  - DataSynth server (ports 50051 gRPC + 3000 REST)
  - Prometheus (port 9090) with auto-configured scrape target
  - Grafana (port 3001) with auto-provisioned Prometheus datasource

- **SystemD Service** (`deploy/datasynth-server.service`): Production daemon configuration
  - Security hardening: NoNewPrivileges, ProtectSystem=strict, PrivateTmp, PrivateDevices
  - Resource limits: MemoryMax=4G, CPUQuota=200%, TasksMax=512, LimitNOFILE=65536

- **Deployment Guide** (`deploy/README.md`): Docker, Docker Compose, and SystemD deployment instructions

- **TLS Reverse Proxy Guide** (`docs/src/deployment/tls-reverse-proxy.md`): nginx and envoy configuration examples

- **Data Lineage & Provenance** (`datasynth-runtime`): Full generation lineage tracking
  - Per-file SHA-256 checksums in `RunManifest` with streaming verification
  - `LineageGraph` tracking config → generator phase → output file relationships
  - CLI `verify` command for manifest integrity validation (`--checksums`, `--record-counts`)
  - W3C PROV-JSON export for interoperability with lineage tools

- **Async Job Queue** (`datasynth-server`): Submit/poll/cancel pattern for long-running generation
  - `POST /api/jobs/submit`, `GET /api/jobs/:id`, `GET /api/jobs`, `POST /api/jobs/:id/cancel`
  - Configurable concurrency limit (`--max-concurrent-jobs`, default 4)
  - Status transitions: Queued → Running → Completed/Failed/Cancelled

- **Redis-Backed Distributed Rate Limiting** (`datasynth-server`): Optional `redis` feature flag
  - Lua-scripted atomic sliding window via `INCR + EXPIRE`
  - `RateLimitBackend` enum abstracting InMemory vs Redis backends
  - Shared rate limit state across server instances

- **Stateless Config Loading** (`datasynth-server`): External config sources
  - `ConfigSource` enum: File, URL, Inline, Default
  - `POST /api/config/reload` endpoint for hot config reloading

- **Formal DP Composition** (`datasynth-fingerprint`): Rényi DP and zCDP accounting
  - `PrivacyAccountant` trait with `NaiveAccountant`, `RenyiDPAccountant`, `ZeroCDPAccountant`
  - RDP curve tracking at alpha values 2-128 with conversion to (ε,δ)-DP
  - zCDP additive ρ composition with tighter bounds
  - `PrivacyBudgetManager` for global budget tracking across extraction runs
  - Composition-aware `PrivacyEngine` with accountant integration

- **Privacy Evaluation Module** (`datasynth-eval`): Post-generation privacy quality gate
  - Membership Inference Attack (MIA) testing via kNN distance-based classifier with AUC-ROC
  - Linkage attack assessment via quasi-identifier re-identification rate
  - NIST SP 800-226 alignment self-assessment
  - SynQP quality-privacy matrix classification (IEEE framework)

- **Custom Privacy Levels** (`datasynth-config`): Configurable (ε, δ) tuples
  - `FingerprintPrivacyConfig` with level, epsilon, delta, k_anonymity, composition_method
  - `PrivacyLevel::Custom` variant for user-specified parameters
  - Validation: epsilon > 0, delta ∈ [0,1), valid composition method

- **Kubernetes Helm Chart** (`deploy/helm/datasynth/`): Production-ready chart
  - HPA (2-10 replicas, CPU target 70%), PDB (minAvailable 1)
  - Rolling updates (maxUnavailable 0, maxSurge 1) with preStop hook
  - Optional Redis subchart (bitnami) for distributed rate limiting
  - Prometheus ServiceMonitor for `/metrics` scraping
  - ConfigMap and Secret templates for YAML config and API keys

- **Load Testing Framework** (`tests/load/`): k6 scripts for API stress testing
  - Health endpoint smoke test (p95 < 100ms)
  - Bulk generation ramp test (1→50→1 VUs)
  - WebSocket streaming test
  - Job queue lifecycle test
  - 30-minute soak test with memory leak monitoring

- **Fuzzing Harnesses** (`fuzz/`): cargo-fuzz targets for untrusted input boundaries
  - Config parsing fuzz target (`serde_yaml::from_slice::<GeneratorConfig>`)
  - DSF fingerprint loading fuzz target
  - YAML validation subsection fuzzing
  - Expanded proptest coverage for distributions, balance coherence, document flows, and privacy

- **Deployment & Operations Documentation** (`docs/src/deployment/`):
  - Docker, Kubernetes, and bare-metal deployment guides
  - Operational runbook with alert response procedures
  - Capacity planning guide with sizing model
  - Disaster recovery procedures
  - API reference with auth, rate limiting, and CORS documentation
  - Security hardening checklist for production deployments

### Changed

- **Unwrap Audit**: Replaced ~2,000+ `.unwrap()` calls in library crates with proper error handling
  - Added `#![deny(clippy::unwrap_used)]` to all library crates (fingerprint, core, generators, output, eval, config, runtime, graph, banking, ocpm, standards)
  - `partial_cmp().unwrap()` → `total_cmp()` for f64 sorting throughout codebase
  - Fallible operations use `?`, `.unwrap_or_default()`, or `.expect()` with descriptive messages
  - Binary crates (cli, server) and test-utils excluded from deny lint

- **API Key Authentication** (`datasynth-server`): Hardened with Argon2id hashing
  - Keys hashed with Argon2id at construction time, stored as PHC-format hashes
  - `with_prehashed_keys()` for loading pre-hashed keys from config/env
  - Timing-safe verification iterating ALL hashes (no short-circuit) via `subtle::ConstantTimeEq`
  - FNV-1a LRU cache with 5-second TTL to avoid Argon2id cost on every request

- **Enhanced `/ready` Endpoint** (`datasynth-server`): Now returns structured health checks
  - Config, memory, and disk health checks with individual status
  - Returns 503 if any check reports "fail"

- **Server Startup** (`datasynth-server`): Now runs both gRPC and REST servers concurrently
  - `--rest-port` (default 3000) and `--grpc-port` (default 50051) CLI arguments
  - `--api-keys` CLI argument for comma-separated API keys
  - Shared `ServerState` between both servers

- **Middleware Stack** (`datasynth-server`): Full production middleware ordering
  - Timeout (5 min) → Rate Limiting → Request Validation → Auth → Request ID → CORS → Security Headers → Router

- **Release Profile** (`Cargo.toml`): Added `strip = true` for smaller release binaries

- Bumped all Rust crate versions to 0.5.0
- Python wrapper version bumped to 0.5.0

## [0.4.1] - 2026-02-06

### Added

- **RustGraph Unified Hypergraph Exporter** (`datasynth-graph`): New `RustGraphUnifiedExporter` producing JSONL with RustGraph-native field names
  - `RawUnifiedNode`: maps `entity_type`→`node_type`, `label`→`name`, `HypergraphLayer`→`layer` as `u8`
  - `RawUnifiedEdge`: maps `source_id`→`source`, `target_id`→`target`, layers→`u8`, adds `weight: f32`
  - `RawUnifiedHyperedge`: extracts `member_ids` from participants, `layer`→`u8`
  - `UnifiedHypergraphMetadata` with `format: "rustgraph_unified_v1"` identifier
  - `export()` for file-based output, `export_to_writer()` for streaming with `_type` tag per line
  - 8 unit tests covering field mapping, file creation, JSONL parseability, and metadata format

- **Streaming to RustGraph Ingest Endpoint** (`datasynth-runtime`): HTTP streaming client behind `streaming` feature flag
  - `StreamClient` implements `std::io::Write` for direct use with `export_to_writer()`
  - Buffers JSONL lines and auto-flushes batches via `reqwest::blocking::Client` POST
  - Configurable batch size (default 1000), timeout (30s), API key auth (`RUSTGRAPH_API_KEY` env), retry with backoff (max 3)
  - `reqwest` added as workspace dependency, gated behind `streaming` feature in runtime and CLI crates

- **CLI Streaming Flags** (`datasynth-cli`): `--stream-target <URL>`, `--stream-api-key`, `--stream-batch-size`
  - Auto-enables hypergraph export in unified format when `--stream-target` is set

- **Hypergraph Output Format Config** (`datasynth-config`): `output_format`, `stream_target`, `stream_batch_size` fields on `HypergraphExportSettings`
  - `output_format: "native"` (default) preserves existing behavior; `"unified"` uses new exporter

### Changed

- **AssureTwin Comprehensive Template**: `graph_export.hypergraph` section now enabled with `output_format: unified`, governance/process/accounting layers, and cross-layer edges
- Orchestrator branches on `output_format` to select unified vs native hypergraph exporter
- Bumped all Rust crate versions to 0.4.1
- Python wrapper version bumped to 0.4.1

## [0.4.0] - 2026-02-05

### Added

- **Parquet Output Sink** (`datasynth-output`): Full Apache Parquet output replacing previous stub
  - 15-column Arrow schema for denormalized journal entry line items
  - Zstd compression (level 3) for efficient storage
  - Configurable batch size (default 10,000 rows) for memory-efficient writes
  - Decimal amounts and UUIDs stored as UTF-8 strings (IEEE 754 precision-safe)
  - `ParquetSink` implements the `Sink` trait with `write()`, `flush()`, `close()`

- **Wasserstein-1 and Jensen-Shannon Divergence** (`datasynth-fingerprint`): Real statistical distance metrics replacing placeholders
  - **Wasserstein-1 (Earth Mover's Distance)**: Piecewise-linear inverse CDF integration via trapezoidal rule across 9 percentile knots (p1-p99)
  - **Jensen-Shannon Divergence**: PMF construction from percentile bins with proper KL divergence computation
  - **Gamma CDF**: Regularized incomplete gamma function via Lanczos approximation (g=7, 9 coefficients) with series expansion and modified Lentz continued fraction
  - **Pareto CDF**: `1 - (x_m/x)^alpha` for heavy-tailed distribution fitting
  - **PointMass and Mixture CDFs**: Step function and weighted sum of component CDFs
  - Per-column Wasserstein distances and JS divergences populated in fidelity evaluation reports

- **IRS MACRS GDS Depreciation Tables** (`datasynth-core`): Proper tax depreciation replacing simplified DDB
  - 6 recovery period tables (3, 5, 7, 10, 15, 20-year) from IRS Publication 946
  - Half-year convention percentages stored as string slices (no f64 precision loss)
  - `macrs_table_for_life()` maps useful life to nearest recovery period
  - `macrs_depreciation(year)` and `ddb_depreciation()` public methods on `FixedAsset`
  - Existing double-declining balance retained as fallback for non-standard useful lives

- **ASC 842 Lease Classification Tests** (`datasynth-standards`): Complete bright-line test implementation
  - 6 new `Lease` fields: `transfers_ownership`, `has_bargain_purchase_option`, `is_specialized_asset`, `initial_direct_costs`, `prepaid_payments`, `lease_incentives`
  - Tests 1 (ownership transfer), 2 (bargain purchase option), and 5 (specialized asset) for both US GAAP and IFRS
  - Enhanced ROU asset measurement: PV + direct costs + prepaid payments - lease incentives (floored at zero)
  - All fields use `#[serde(default)]` for backward compatibility

- **FX Monetary/Non-Monetary Classification** (`datasynth-generators`): Proper translation method support
  - `is_monetary(account_code)` classifies accounts by 2-digit prefix (cash, AR, liabilities = monetary; inventory, PP&E, equity = non-monetary)
  - `historical_equity_rates` field on `CurrencyTranslator` for equity account translation
  - **Temporal method**: Monetary assets → closing rate, non-monetary → historical rate, income/expense → average rate
  - **MonetaryNonMonetary method**: Full rate selection based on monetary classification

- **Entity-Aware Anomaly Injection** (`datasynth-generators`): Risk-adjusted injection rates
  - `VendorContext`, `EmployeeContext`, `AccountContext` structs with risk attributes
  - `set_entity_contexts()` for orchestrator to provide context after master data generation
  - Rate multipliers: new vendor 2.0x, dormant vendor 1.5x, new employee 1.5x, volume-fatigued 1.3x, high-risk account 2.0x
  - Multiplied factors cap at 1.0; entity contexts persist across `reset()` calls
  - Anomaly labels annotated with `entity_context_multiplier` and `effective_rate`

### Changed

- **Orchestrator Decomposition** (`datasynth-runtime`): `generate()` method refactored from ~300-line monolith into 12 focused phase methods
  - `phase_chart_of_accounts`, `phase_master_data`, `phase_document_flows`, `phase_ocpm_events`, `phase_journal_entries`, `phase_anomaly_injection`, `phase_balance_validation`, `phase_data_quality_injection`, `phase_audit_data`, `phase_banking_data`, `phase_graph_export`, `phase_hypergraph_export`
  - Main `generate()` is now a ~90-line pipeline calling phase methods in sequence

- **Graph Exporter Config Consolidation** (`datasynth-graph`): DRY refactoring of export configuration
  - New `CommonExportConfig` struct shared by PyG, DGL, and Neo4j exporters (8 fields: features, labels, masks, train/val ratio, seed)
  - New `CommonGraphMetadata` struct shared by PyG and DGL exporters (11 fields)
  - ~200 lines of duplicated field definitions and defaults eliminated

- **Validation Helper Extraction** (`datasynth-config`): DRY refactoring of config validation
  - 5 shared helper functions: `validate_sum_to_one`, `validate_range_f64`, `validate_ascending`, `validate_positive`, `validate_rate`
  - Replaced 16+ sum-to-one, 15+ rate, 3 ascending, and 6 positive inline validation checks

- **Test Fixture Centralization**: Shared test helper modules replacing duplicated fixtures
  - `datasynth-graph`: 9 copies of `create_test_graph()` → `test_helpers.rs` with 3 variants
  - `datasynth-generators`: 4 copies of `create_test_engagement()` → `audit/test_helpers.rs`
  - `datasynth-output`: 3 copies of `create_test_je()` → `test_helpers.rs`

- Bumped all Rust crate versions to 0.4.0
- Python wrapper version bumped to 0.4.0

## [0.3.1] - 2026-02-05

### Added

- **Multi-Layer Hypergraph Export** (`datasynth-graph`): New 3-layer hypergraph builder and exporter for RustGraph integration
  - **HypergraphBuilder**: Constructs a 3-layer hypergraph from enterprise data
    - Layer 1 (Governance & Controls): COSO 2013 framework (5 components, 17 principles), internal controls with SOX assertions, vendors, customers, employees
    - Layer 2 (Process Events): P2P document chains (POs, goods receipts, invoices, payments) and O2C document chains (sales orders, deliveries, customer invoices) with counterparty-based pool aggregation when budget exceeded
    - Layer 3 (Accounting Network): GL accounts as nodes, journal entries as hyperedges connecting multiple debit/credit accounts simultaneously
  - **Node Budget System**: Per-layer allocation (L1: 20%, L2: 70%, L3: 10%) with automatic rebalancing and pool aggregation for overflow
    - `AggregationStrategy`: Truncate, PoolByCounterparty (default), PoolByTimePeriod, ImportanceSample
    - Pool nodes carry summary features: count, total amount, avg amount, date range, anomaly rate
  - **Cross-Layer Edges**: Automatic edge generation linking governance controls to accounts, vendors to POs, customers to sales orders, employees to approvals
  - **RustGraph Entity Type Codes**: Pre-assigned codes (100-510) for 20+ entity types and edge types (40-55) for seamless RustGraph import
  - **Hyperedge Support**: Journal entries modeled as hyperedges with debit/credit participants, weights, timestamps, and anomaly flags
  - **JSONL Export**: `HypergraphExporter` writes `nodes.jsonl`, `edges.jsonl`, `hyperedges.jsonl`, and `metadata.json` for RustGraph file import

- **Hypergraph Configuration** (`datasynth-config`): New `RustGraphHypergraph` export format and configuration
  - `HypergraphExportSettings`: max_nodes (default 50,000), aggregation strategy, per-layer toggles
  - `GovernanceLayerSettings`: Toggle COSO, controls, SOX, vendors, customers, employees
  - `ProcessLayerSettings`: Toggle P2P/O2C, events-as-hyperedges, counterparty threshold
  - `AccountingLayerSettings`: Toggle accounts, journal-entries-as-hyperedges
  - `CrossLayerSettings`: Toggle cross-layer edge generation
  - Validation: max_nodes 1-150,000, aggregation strategy validation, threshold bounds

- **Orchestrator Phase 10b** (`datasynth-runtime`): Hypergraph export integrated into the generation pipeline
  - Automatic hypergraph generation after Phase 10 graph export when enabled
  - Feeds all available data: CoA, journal entries, master data, document flows, COSO controls
  - Output to `graphs/hypergraph/` subdirectory

### Changed

- `GraphExportFormat` enum extended with `RustGraphHypergraph` variant
- Bumped all Rust crate versions to 0.3.1
- Python wrapper version bumped to 0.3.1

## [0.3.0] - 2026-02-01

### Added

- **OCPM Integration Enhancement** (`datasynth-ocpm`): Enhanced Object-Centric Process Mining support
  - **Deterministic UUID Generation**: `OcpmUuidFactory` using FNV-1a hashing with type discriminators
    - `OcpmUuidType` enum: Case (0xC0), Event (0xE0), Object (0xB0) discriminators
    - Reproducible event logs with seeded UUID generation
    - Counter-based sequencing for collision-free IDs
  - **XES 2.0 Export**: `XesExporter` for IEEE standard event log format
    - Compatible with ProM, Celonis, Disco, and pm4py
    - Configurable lifecycle transitions and resource attributes
    - Custom attribute export support
    - Pretty-print XML output
  - **Extended Activity Types**: 17 new R2R and A2R activities
    - GL Activities: `post_journal_entry()`, `review_journal_entry()`, `approve_journal_entry()`, `reverse_journal_entry()`
    - FX Activities: `fx_revaluation()`, `currency_translation()`
    - Period Close: `post_accruals()`, `reverse_accruals()`, `run_ic_elimination()`, `close_period()`, `reopen_period()`
    - Trial Balance: `generate_trial_balance()`, `review_trial_balance()`, `approve_trial_balance()`, `run_consolidation()`
    - Fixed Assets: `run_depreciation()`, `asset_impairment_test()`
    - Helper methods: `r2r_activities()`, `a2r_activities()`, `all_activities()`
  - **Reference Process Models**: Canonical process definitions for conformance checking
    - `ReferenceProcessModel` with activities, transitions, and variants
    - `ReferenceActivity`: Required/optional flags, start/end markers, duration estimates
    - `ReferenceTransition`: Standard path indicators, probabilities, conditions
    - `ReferenceVariant`: Activity sequences with expected frequencies
    - Standard models: `p2p_standard()` (9 activities, 3 variants), `o2c_standard()` (10 activities, 2 variants), `r2r_standard()` (11 activities, 4 variants)
    - `ReferenceModelExporter` for JSON export

- **Streaming Output Sinks** (`datasynth-output`): Complete streaming sink implementations
  - `CsvStreamingSink<T>`: CSV output with header auto-generation and field mapping
  - `JsonStreamingSink<T>`: JSON array format with pretty-print option
  - `NdjsonStreamingSink<T>`: Newline-delimited JSON for streaming consumption
  - `ParquetStreamingSink<T>`: Apache Parquet output with configurable row groups
    - `ToParquetBatch` trait for custom type serialization
    - `GenericParquetRecord` for dynamic schemas
    - Lazy writer initialization for schema inference
    - SNAPPY compression support

- **Complete Streaming Orchestrator** (`datasynth-runtime`): Full document flow generation
  - New `GeneratedItem` variants: `PurchaseOrder`, `GoodsReceipt`, `VendorInvoice`, `Payment`, `SalesOrder`, `Delivery`, `CustomerInvoice`
  - `GenerationPhase::OcpmEvents` for event log generation
  - `generate_document_flows_phase()` implementation
  - `StreamingOrchestratorConfig::with_all_phases()` helper

- **Process Family Edge Metadata** (`datasynth-graph`): Transaction graph enhancement
  - `TransactionEdge.business_process` field now populated from journal entry headers
  - Business process tracking for P2P, O2C, R2R, and other process families
  - Enables process-aware graph analytics and filtering

- **OCPM Configuration Updates** (`datasynth-config`): Extended output options
  - `OcpmOutputConfig.xes`: Enable XES 2.0 export
  - `OcpmOutputConfig.xes_include_lifecycle`: Include lifecycle transitions
  - `OcpmOutputConfig.xes_include_resources`: Include resource attributes
  - `OcpmOutputConfig.export_reference_models`: Export canonical process models

- **ACFE-Aligned Fraud Taxonomy** (`datasynth-core`, `datasynth-generators`): Comprehensive fraud classification based on ACFE Report to the Nations
  - `AcfeFraudCategory`: Asset Misappropriation (86% of cases), Corruption (33%), Financial Statement Fraud (10%)
  - `CashFraudScheme`: 20 cash-based fraud schemes (skimming, larceny, shell company, ghost employee, etc.)
  - `CorruptionScheme`: Conflicts of interest, bribery, kickbacks, bid rigging, economic extortion
  - `FinancialStatementScheme`: Revenue manipulation, expense timing, concealed liabilities, improper disclosures
  - `AcfeCalibration`: Statistics calibration ($117k median loss, 12-month median duration)
  - Detection method distribution aligned with ACFE findings (42% tips, 16% internal audit, 12% management review)

- **Collusion & Conspiracy Modeling** (`datasynth-generators`): Multi-party fraud network simulation
  - `CollusionRing`: Network of conspirators executing coordinated fraud schemes
  - `CollusionRingType`: 9 ring types (EmployeePair, DepartmentRing, EmployeeVendor, VendorRing, etc.)
  - `Conspirator`: Individual participant with role, loyalty, risk tolerance, and share of proceeds
  - `ConspiratorRole`: 6 roles (Initiator, Executor, Approver, Concealer, Lookout, Beneficiary)
  - `RingStatus`: Lifecycle tracking (Forming, Active, Escalating, Dormant, Dissolving, Detected)
  - Defection modeling based on detection risk, pressure, and loyalty
  - Coordinated transaction generation requiring multiple conspirators

- **Management Override Patterns** (`datasynth-generators`): Senior-level fraud modeling
  - `ManagementOverrideScheme`: Executive-level fraud with override techniques
  - `ManagementLevel`: SeniorManager, CFO, CEO, COO, ControllerCAO, BoardMember
  - `OverrideType`: Revenue, Expense, Asset, Liability, Disclosure overrides
  - `PressureSource`: Financial targets, market expectations, covenant compliance, personal issues
  - `FraudTriangle`: Pressure, Opportunity, Rationalization modeling
  - `ManagementConcealment`: False documentation, subordinate intimidation, auditor deception

- **Red Flag Generation** (`datasynth-generators`): Probabilistic fraud indicator injection
  - `RedFlagPattern`: Configurable red flag patterns with Bayesian probabilities
  - `RedFlagStrength`: Strong (P(fraud|flag) > 0.5), Moderate (0.2-0.5), Weak (< 0.2)
  - `RedFlagCategory`: Vendor, Transaction, Timing, Approval, Document, Behavioral categories
  - P(flag|fraud) and P(flag|not fraud) calibration for realistic false positive rates
  - 40+ pre-configured red flag patterns based on audit literature
  - `RedFlagStatistics`: Statistics tracking for generated flags

- **Industry-Specific Transactions** (`datasynth-generators`): Authentic industry transaction modeling
  - **Manufacturing**:
    - `ManufacturingTransaction`: 14 transaction types (WorkOrderIssuance, MaterialRequisition, LaborBooking, etc.)
    - `BillOfMaterials`: Multi-level BOM with components, yield rates, scrap factors
    - `Routing`: Production routings with operations, work centers, labor/machine rates
    - `WorkCenter`: Capacity, efficiency, cost center allocation
    - `ManufacturingSettings`: BOM depth, JIT, quality framework, supplier tiers
  - **Retail**:
    - `RetailTransaction`: 12 transaction types (PosSale, ReturnRefund, InventoryReceipt, etc.)
    - `StoreType`: Flagship, Standard, Express, Outlet, Warehouse, PopUp, Digital
    - `RetailSettings`: Shrinkage rate, return rate, markdown patterns
    - Loss prevention configuration with camera coverage and EAS
  - **Healthcare**:
    - `HealthcareTransaction`: 15 transaction types (PatientRegistration, ChargeCapture, ClaimSubmission, etc.)
    - `PayerType`: Medicare, Medicaid, Commercial, SelfPay with configurable payer mix
    - `CodingSystem`: ICD-10, CPT, DRG, HCPCS support
    - `FacilityType`: Hospital, PhysicianPractice, AmbulatorySurgery, SkilledNursing, HomeHealth
    - HIPAA, Stark Law, Anti-Kickback compliance configuration

- **Industry-Specific Anomalies** (`datasynth-generators`): Authentic industry fraud patterns
  - **Manufacturing**: Yield manipulation, labor misallocation, phantom production, obsolete inventory concealment
  - **Retail**: Sweethearting, skimming, refund fraud, receiving fraud, coupon fraud, employee discount abuse
  - **Healthcare**: Upcoding, unbundling, phantom billing, duplicate billing, physician kickbacks, HIPAA violations

- **Industry-Specific Configuration** (`datasynth-config`): New configuration schema
  - `IndustrySpecificConfig`: Root configuration for industry-specific generation
  - `ManufacturingConfig`: BOM depth, JIT, supplier tiers, quality framework, anomaly rates
  - `RetailConfig`: Store types, shrinkage rate, loss prevention, markdown patterns
  - `HealthcareConfig`: Facility type, payer mix, coding systems, compliance frameworks
  - `TechnologyConfig`: Revenue model, R&D capitalization, deferred revenue
  - `FinancialServicesConfig`: Institution type, regulatory framework, loan loss provisions
  - `ProfessionalServicesConfig`: Billing model, trust accounting, engagement types
  - Industry-specific anomaly rate configuration for each sector

- **ACFE-Calibrated Benchmarks** (`datasynth-eval`): ML evaluation benchmarks aligned with ACFE statistics
  - `acfe_calibrated_1k()`: General fraud detection benchmark with ACFE category distribution
  - `acfe_collusion_5k()`: Collusion-focused benchmark emphasizing network analysis
  - `acfe_management_override_2k()`: Management override detection with journal entry features
  - `AcfeAlignment`: Metrics for ACFE alignment (category distribution MAD, median loss ratio, duration KS)
  - Cost-sensitive evaluation with asymmetric cost matrices

- **Industry-Specific Benchmarks** (`datasynth-eval`): Fraud detection benchmarks by industry
  - `manufacturing_fraud_5k()`: Inventory, production order, and cost allocation fraud
  - `retail_fraud_10k()`: POS, shrinkage, and return fraud detection
  - `healthcare_fraud_5k()`: Revenue cycle fraud (upcoding, unbundling, phantom billing)
  - `technology_fraud_3k()`: Revenue recognition and capitalization fraud
  - `financial_services_fraud_5k()`: Loan, trading, and account fraud
  - `IndustryBenchmarkAnalysis`: Industry-specific performance metrics
  - `get_industry_benchmark()`: Factory function for benchmark retrieval

- **Interconnectivity Enhancements** (`datasynth-core`, `datasynth-generators`): Comprehensive relationship modeling for realistic enterprise networks
  - **Multi-Tier Vendor Networks**:
    - `VendorNetwork` with supply chain tiers (Tier1/Tier2/Tier3)
    - `VendorCluster` types: ReliableStrategic (20%), StandardOperational (50%), Transactional (25%), Problematic (5%)
    - `VendorLifecycleStage`: Onboarding, RampUp, SteadyState, Decline, Terminated
    - `VendorQualityScore`: Delivery, quality, invoice accuracy, responsiveness metrics
    - `VendorDependency`: Concentration analysis, single-source tracking, substitutability
    - `PaymentHistory`: On-time, early, late payment tracking with averages
  - **Customer Value Segmentation**:
    - `CustomerValueSegment`: Enterprise (40% rev/5% cust), MidMarket (35%/20%), SMB (20%/50%), Consumer (5%/25%)
    - `CustomerLifecycleStage`: Prospect, New, Growth, Mature, AtRisk, Churned, WonBack
    - `CustomerNetworkPosition`: Referral networks, parent/child hierarchies, industry clusters
    - `CustomerEngagement`: Order frequency, recency, NPS scores, engagement scoring
    - `SegmentedCustomerPool`: Index by segment and lifecycle stage
  - **Entity Relationship Graph**:
    - `GraphEntityType`: 16 entity types (Company, Vendor, Customer, Employee, etc.)
    - `RelationshipType`: 26 relationship types (BuysFrom, SellsTo, ReportsTo, etc.)
    - `RelationshipStrengthCalculator`: Composite strength from volume, count, duration, recency, connections
    - `CrossProcessLink`: P2P↔O2C linkage via inventory (GoodsReceipt→Delivery)
    - `EntityGraph` with node/edge management and graph metrics
  - **Generator Extensions**:
    - `VendorGenerator.generate_vendor_network()`: Multi-tier hierarchy with cluster assignment
    - `CustomerGenerator.generate_segmented_pool()`: Segment distribution, referral networks, corporate hierarchies
    - `EntityGraphGenerator`: Entity graph construction with cross-process links and strength calculation

- **Interconnectivity Configuration** (`datasynth-config`): New configuration sections for network modeling
  - `VendorNetworkSchemaConfig`: Tier depth, count ranges, cluster distribution, concentration limits
  - `CustomerSegmentationSchemaConfig`: Value segments, lifecycle distribution, referral/hierarchy config
  - `RelationshipStrengthSchemaConfig`: Weight configuration (volume 30%, count 25%, duration 20%, recency 15%, connections 10%)
  - `CrossProcessLinksSchemaConfig`: Enable inventory P2P-O2C links, IC bilateral links
  - Comprehensive validation rules for all interconnectivity settings

- **Pattern and Process Drift** (`datasynth-core`): Comprehensive drift modeling for realistic temporal evolution
  - **Organizational Events**:
    - `OrganizationalEventType`: Acquisition, Divestiture, Reorganization, LeadershipChange, WorkforceReduction, Merger
    - `AcquisitionConfig`: Volume multiplier (1.35x), integration error rate (5%), parallel posting periods
    - `IntegrationPhaseConfig`: Parallel run, cutover, stabilization, and hypercare phases
    - Effect blending modes: Multiplicative, Additive, Maximum, Minimum
  - **Process Evolution**:
    - `ProcessEvolutionType`: ApprovalWorkflowChange, ProcessAutomation, PolicyChange, ControlEnhancement
    - `ProcessAutomationConfig`: S-curve automation rollout with configurable steepness and midpoint
    - `WorkflowType`: Manual, SemiAutomated, FullyAutomated with transition modeling
  - **Technology Transitions**:
    - `TechnologyTransitionType`: ErpMigration, ModuleImplementation, IntegrationUpgrade
    - `ErpMigrationConfig`: Migration phases with error rate and processing time multipliers
    - `MigrationIssueConfig`: Duplicate rate, missing data rate, format mismatch rate
  - **Behavioral Drift**:
    - `VendorBehavioralDrift`: Payment terms extension, quality drift, pricing behavior
    - `CustomerBehavioralDrift`: Payment delays during downturns, order pattern shifts
    - `EmployeeBehavioralDrift`: Approval pattern changes, learning curve, fatigue effects
    - `CollectiveBehavioralDrift`: Year-end intensity, automation adoption (S-curve), remote work impact
  - **Market Drift**:
    - `MarketDriftModel`: Economic cycles, industry-specific cycles, commodity drift
    - `EconomicCycleModel`: Sinusoidal, Asymmetric, MeanReverting cycle types
    - `RecessionConfig`: Probability, onset type (Gradual/Sudden), duration, severity
    - `PriceShockEvent`: Supply disruption, demand surge modeling
  - **Regulatory Events**:
    - `RegulatoryDriftEvent`: Accounting standard adoption, tax rate changes, compliance requirements
    - `AuditFocusEvent`: Risk-based shifts, industry trend responses, prior year finding follow-ups
    - `RegulatoryCalendar`: Preset calendars for US GAAP 2024, IFRS 2024
  - **Event Timeline Controller**:
    - `EventTimeline`: Orchestrates organizational, process, and technology events
    - `TimelineEffects`: Volume/amount multipliers, error rate deltas, entity changes, account remapping
  - **Drift Detection Ground Truth**:
    - `DriftEventType`: StatisticalShift, CategoricalShift, TemporalShift, RegulatoryChange
    - `LabeledDriftEvent`: Event metadata with magnitude and detection difficulty
    - `DriftLabelRecorder`: Ground truth label recording with CSV/JSON export
    - `DetectionDifficulty`: Easy, Medium, Hard classification for ML training

- **Drift Detection Evaluation** (`datasynth-eval`): Evaluation framework for drift detection
  - `DriftDetectionAnalyzer`: Statistical drift detection with rolling window analysis
  - `DriftDetectionMetrics`: Precision, recall, F1 score, mean detection delay
  - Hellinger distance calculation for distribution comparison
  - Population Stability Index (PSI) for drift magnitude measurement
  - `LabeledEventAnalysis`: Ground truth event quality assessment
  - Configurable thresholds for drift detection quality

- **Drift Configuration** (`datasynth-config`): New configuration sections for drift modeling
  - `OrganizationalEventsSchemaConfig`: Event types, dates, integration phases
  - `BehavioralDriftSchemaConfig`: Vendor, customer, employee, collective behavior settings
  - `MarketDriftSchemaConfig`: Economic cycles, industry cycles, commodities, price shocks
  - `DriftLabelingSchemaConfig`: Ground truth labeling configuration

- **Network Evaluation** (`datasynth-eval`): New network metrics evaluation module
  - `NetworkEvaluator`: Graph analysis with connectivity, degree distribution, clustering
  - `ConcentrationMetrics`: Top-1, Top-5 concentration, HHI calculation
  - `StrengthStats`: Relationship strength distribution analysis
  - Power law alpha estimation for degree distribution
  - Clustering coefficient calculation
  - Cross-process link coverage validation

- **Statistical Distribution Enhancement** (`datasynth-core`): Advanced statistical distribution framework for realistic data generation
  - **Mixture Models**: Gaussian and Log-Normal mixture distributions with weighted components
    - `GaussianMixtureSampler` and `LogNormalMixtureSampler` for multi-modal distributions
    - Component labeling (e.g., "routine", "significant", "major" transactions)
    - Pre-computed cumulative weights for O(log n) component selection
    - Configurable weight validation ensuring sum to 1.0
  - **Copula-Based Correlation Engine**: Cross-field dependency modeling
    - Gaussian, Clayton, Gumbel, Frank, and Student-t copula support
    - Cholesky decomposition for correlation matrix sampling
    - `CorrelationEngine` for generating correlated field values
    - Configurable correlation matrices with symmetric validation
  - **New Distribution Types**:
    - Pareto distribution for heavy-tailed data (capital expenses)
    - Weibull distribution for time-to-event modeling (days-to-payment)
    - Beta distribution for proportions (discount percentages)
    - Zero-inflated distributions for excess zeros (credits/returns)
  - **Enhanced Benford's Law**: Second-digit compliance and anomaly injection
    - `BenfordDeviationSampler` for round number bias and threshold clustering
  - **Regime Changes**: Structural breaks in time series
    - Economic cycle modeling with configurable period and amplitude
    - Acquisition/divestiture effects on transaction volumes
    - Recession probability and depth parameters
  - **Industry Profiles**: Pre-configured distribution profiles
    - Retail, Manufacturing, Financial Services profiles
    - Industry-specific transaction amount mixtures

- **Statistical Validation Framework** (`datasynth-eval`): Comprehensive validation tests
  - Benford's Law first-digit test with MAD threshold
  - Anderson-Darling goodness-of-fit test
  - Chi-squared distribution test
  - Correlation matrix verification
  - Configurable significance levels and fail-on-violation option

- **Advanced Distribution Configuration** (`datasynth-config`): New configuration schema
  - `AdvancedDistributionConfig` with mixture, correlation, regime change settings
  - `MixtureDistributionConfig` for component weights, mu, sigma, labels
  - `CorrelationConfig` for copula type, fields, and correlation matrix
  - `RegimeChangeConfig` for economic cycles and structural breaks
  - `StatisticalValidationConfig` for test selection and thresholds
  - Validation rules for matrix symmetry, weight sums, and parameter bounds

- **Realistic Name Generation** (`datasynth-core`): Enhanced name/metadata module
  - Culture-aware name generation with distribution controls
  - `NameTemplateConfig` for email domain and name generation settings
  - `CultureDistributionConfig` for cultural name patterns

- **Python Distribution Configuration** (`python/datasynth_py`): Full Python API
  - `MixtureComponentConfig`, `MixtureDistributionConfig` dataclasses
  - `CorrelationConfig`, `CorrelationFieldConfig` for dependency modeling
  - `RegimeChangeConfig`, `EconomicCycleConfig` for time series breaks
  - `StatisticalValidationConfig`, `StatisticalTestConfig` for validation
  - New blueprints: `statistical_validation()`, `with_distributions()`, `with_regime_changes()`
  - Updated `ml_training()` and `retail_small()` with distribution support

- **Desktop UI Distribution Page** (`datasynth-ui`): Visual configuration
  - Distribution settings panel with industry profile selection
  - Mixture model editor with component weight normalization
  - Correlation matrix editor with copula type selector
  - Regime change configuration with economic cycle parameters
  - Statistical validation test selection interface

### Changed

- `GeneratorConfig` now includes `industry_specific` field for industry-specific settings
- `GeneratorConfig` now includes `distributions` field for advanced distribution settings
- All presets, fixtures, and config initializers updated with industry-specific and distributions support
- `FraudType` enum extended with ACFE-aligned fraud categories and industry-specific schemes
- `datasynth-generators/src/lib.rs` now exports `fraud` and `industry` modules
- `datasynth-eval` benchmarks module extended with ACFE and industry benchmarks
- Python wrapper version bumped to 0.3.0 with distribution dataclasses

## [0.2.3] - 2026-01-28

### Added

- **Accounting & Audit Standards Framework** (`datasynth-standards`): New crate providing comprehensive accounting and auditing standards support
  - **Accounting Standards**:
    - `AccountingFramework` enum: US GAAP, IFRS, and Dual Reporting modes
    - `FrameworkSettings`: Framework-specific accounting policies with validation
    - Revenue Recognition (ASC 606/IFRS 15): `CustomerContract`, `PerformanceObligation`, `VariableConsideration`
    - Lease Accounting (ASC 842/IFRS 16): `Lease`, `ROUAsset`, `LeaseLiability`, amortization schedules
    - Fair Value Measurement (ASC 820/IFRS 13): `FairValueMeasurement`, hierarchy levels
    - Impairment Testing (ASC 360/IAS 36): `ImpairmentTest`, US GAAP two-step and IFRS one-step tests
    - Framework differences tracking for dual reporting reconciliation
  - **Audit Standards**:
    - ISA References: 34 ISA standards (ISA 200-720) with `IsaRequirement` and `IsaProcedureMapping`
    - Analytical Procedures (ISA 520): `AnalyticalProcedure`, variance investigation, threshold checking
    - External Confirmations (ISA 505): `ExternalConfirmation`, response tracking, exception handling
    - Audit Opinion (ISA 700/705/706/701): `AuditOpinion`, `KeyAuditMatter`, modifications
    - Audit Trail: Complete traceability with gap analysis
    - PCAOB Standards: 19+ PCAOB standards with ISA mapping
  - **Regulatory Frameworks**:
    - SOX Section 302: CEO/CFO certifications with material weakness tracking
    - SOX Section 404: ICFR assessment with deficiency classification matrix
    - `DeficiencyMatrix`: Likelihood × Magnitude classification for MW/SD determination

- **Standards Compliance Evaluation** (`datasynth-eval`): New evaluators for standards compliance
  - `StandardsComplianceEvaluation`: Comprehensive standards validation
  - `RevenueRecognitionEvaluator`: ASC 606/IFRS 15 compliance checking
  - `LeaseAccountingEvaluator`: Classification accuracy, ROU asset validation
  - `FairValueEvaluation`, `ImpairmentEvaluation`, `IsaComplianceEvaluation`
  - `SoxComplianceEvaluation`, `PcaobComplianceEvaluation`, `AuditTrailEvaluation`
  - `StandardsThresholds`: Configurable compliance thresholds

- **Standards Configuration** (`datasynth-config`): Configuration sections for standards generation
  - `AccountingStandardsConfig`: Framework selection, revenue recognition, leases, fair value, impairment
  - `AuditStandardsConfig`: ISA compliance, analytical procedures, confirmations, opinions, SOX, PCAOB
  - Configuration validation for framework-specific rules
  - Integration with existing presets and templates

- **COSO 2013 Framework Integration** (`datasynth-core`): Full COSO Internal Control-Integrated Framework support
  - `CosoComponent` enum: 5 COSO components (Control Environment, Risk Assessment, Control Activities, Information & Communication, Monitoring Activities)
  - `CosoPrinciple` enum: 17 COSO principles with `component()` and `principle_number()` helper methods
  - `ControlScope` enum: Entity-level, Transaction-level, IT General Control, IT Application Control
  - `CosoMaturityLevel` enum: 6-level maturity model (Non-Existent through Optimized)
  - Extended `InternalControl` struct with COSO fields: `coso_component`, `coso_principles`, `control_scope`, `maturity_level`
  - Builder methods: `with_coso_component()`, `with_coso_principles()`, `with_control_scope()`, `with_maturity_level()`

- **Entity-Level Controls** (`datasynth-core`): 6 new organization-wide controls
  - C070: Code of Conduct and Ethics (Control Environment)
  - C071: Audit Committee Oversight (Control Environment)
  - C075: Enterprise Risk Assessment (Risk Assessment)
  - C077: IT General Controls Program (Control Activities)
  - C078: Financial Information Quality (Information & Communication)
  - C081: Internal Control Monitoring Program (Monitoring Activities)

- **COSO Control Mapping Export** (`datasynth-output`): New export file `coso_control_mapping.csv`
  - Maps each control to COSO component, principle number, principle name, and control scope
  - One row per control-principle pair for granular analysis
  - Extended `internal_controls.csv` with COSO columns

- **COSO Configuration Options** (`datasynth-config`): New `InternalControlsConfig` fields
  - `coso_enabled`: Enable/disable COSO framework integration (default: true)
  - `include_entity_level_controls`: Include entity-level controls in generation (default: false)
  - `target_maturity_level`: Target maturity level ("ad_hoc", "repeatable", "defined", "managed", "optimized", "mixed")

### Changed

- `CoherenceEvaluation` now includes `StandardsComplianceEvaluation` field
- All industry presets include default `AccountingStandardsConfig` and `AuditStandardsConfig`
- Added 73 new tests (55 unit + 18 integration) for standards crate
- All 12 existing transaction-level controls (C001-C060) now include COSO component and principle mappings
- `ExportSummary` includes `coso_mappings_count` field
- `ControlExporter::export_all()` and `export_standard()` now export COSO mapping file

## [0.2.2] - 2026-01-26

### Added

- **RustGraph JSON Export** (`datasynth-graph`): New export format for RustAssureTwin integration
  - `RustGraphNodeOutput` and `RustGraphEdgeOutput` structures compatible with RustGraph CreateNodeRequest/CreateEdgeRequest
  - Rich metadata including temporal validity (valid_from/valid_to), transaction time, labels, and ML features
  - JSONL and JSON array output formats for streaming and batch consumption
  - `RustGraphExporter` with configurable options (include_features, include_temporal, include_labels)
  - Automatic metadata generation with source tracking, batch IDs, and generation timestamps

- **Streaming Output API** (`datasynth-core`, `datasynth-runtime`): Async streaming generation with backpressure
  - `StreamingGenerator` trait with async `stream()` and `stream_with_progress()` methods
  - `StreamingSink` trait for processing stream events
  - `StreamEvent` enum: Data, Progress, BatchComplete, Error, Complete variants
  - Backpressure strategies: Block, DropOldest, DropNewest, Buffer with overflow
  - `BoundedChannel` with adaptive backpressure and statistics tracking
  - `StreamingOrchestrator` wrapping EnhancedOrchestrator for streaming generation
  - Progress reporting with items_generated, items_per_second, elapsed_ms, memory_usage
  - Stream control: pause, resume, cancel via `StreamHandle`

- **Temporal Attribute Generation** (`datasynth-generators`): Bi-temporal data support
  - `TemporalAttributeGenerator` for adding temporal dimensions to entities
  - Valid time generation with configurable closed probability and validity duration
  - Transaction time generation with optional backdating support
  - Version chain generation for entity history tracking
  - Integration with existing `BiTemporal<T>` and `TemporalVersionChain<T>` models

- **Relationship Generation** (`datasynth-generators`): Configurable entity relationships
  - `RelationshipGenerator` for creating edges between generated entities
  - Cardinality rules: OneToOne, OneToMany, ManyToOne, ManyToMany with configurable min/max
  - Property generation: Constant, RandomChoice, Range, FromSourceProperty, FromTargetProperty
  - Circular reference detection with configurable max depth
  - Orphan entity support with configurable probability

- **Rate Limiting** (`datasynth-core`): Token bucket rate limiter for controlled generation
  - `RateLimiter` with configurable entities_per_second and burst_size
  - Backpressure modes: Block, Drop, Buffer with max_buffered
  - `RateLimitedStream<G>` wrapper for rate-limiting any StreamingGenerator
  - Statistics tracking: total_acquired, total_dropped, total_waited, avg_wait_time

- **New Configuration Sections** (`datasynth-config`):
  - `streaming`: buffer_size, enable_progress, progress_interval, backpressure strategy
  - `rate_limit`: enabled, entities_per_second, burst_size, backpressure mode
  - `temporal_attributes`: valid_time config, transaction_time config, version chain options
  - `relationships`: relationship types with cardinality rules, orphan settings, circular detection

### Changed

- `GraphExportFormat` enum extended with `RustGraph` variant
- `GeneratorConfig` now includes streaming, rate_limit, temporal_attributes, and relationships sections
- All presets, fixtures, and config validation updated for new configuration fields

## [0.2.1] - 2026-01-24

### Added

- **Accounting Network Graph Export**: Integrated graph export directly into the generation pipeline
  - Automatic export of journal entries as directed transaction graphs
  - Nodes represent GL accounts, edges represent money flows (debit→credit)
  - 8-dimensional edge features: log_amount, benford_prob, weekday, period, is_month_end, is_year_end, is_anomaly, business_process
  - Train/validation/test masks for ML training (70/15/15 split)
  - CLI flag `--graph-export` to enable during generation
  - PyTorch Geometric format with `.npy` files and auto-generated loader script

- **Python Wrapper Enhancements** (`python/datasynth_py`):
  - `FingerprintClient` class for fingerprint operations (extract, validate, info, evaluate)
  - Streaming pattern triggers: `trigger_month_end()`, `trigger_year_end()`, `trigger_fraud_cluster()`
  - Complete config coverage: `BankingSettings`, `ScenarioSettings`, `TemporalDriftSettings`, `DataQualitySettings`, `GraphExportSettings`
  - New blueprints: `banking_aml()`, `ml_training()`, `with_graph_export()`
  - Synchronous event consumption with `sync_events()` callback

- **Desktop UI Improvements**:
  - Mobile responsive design with hamburger menu for sidebar navigation
  - Improved config loading UX with proper loading states
  - Fixed config store initialization with default values

### Fixed

- **Graph Edge Labels**: Fixed bug where `edge_labels.npy` contained all zeros even when anomalies existed
  - `TransactionGraphBuilder` now propagates `is_anomaly` flag from journal entries to graph edges
  - Anomaly type is also captured in edge metadata

- **E2E Test Stability**: Added explicit waits for config loading before form interactions

### Changed

- Graph export phase integrated into `EnhancedOrchestrator` workflow (Phase 10)
- Run manifest now includes graph export statistics (nodes, edges, formats)

## [0.2.0] - 2026-01-23

### Added

- **Synthetic Data Fingerprinting** (`datasynth-fingerprint`): New crate for privacy-preserving fingerprint extraction and generation
  - Extract statistical fingerprints from real data into `.dsf` files (ZIP archives with YAML/JSON components)
  - **Privacy Engine**: Differential privacy with Laplace mechanism, k-anonymity suppression, winsorization, full audit trail
  - **Privacy Levels**: Configurable presets (minimal ε=5.0/k=3, standard ε=1.0/k=5, high ε=0.5/k=10, maximum ε=0.1/k=20)
  - **Extraction Engine**: 6 extractors (schema, statistics, correlation, integrity, rules, anomaly)
  - **I/O System**: DSF file format with SHA-256 checksums and signature support
  - **Config Synthesis**: Generate `GeneratorConfig` from fingerprints with distribution fitting
  - **Gaussian Copula**: Preserve multivariate correlations during synthesis
  - **Fidelity Evaluation**: Compare synthetic data against fingerprints with KS statistics, Wasserstein distance, correlation RMSE, Benford MAD

- **CLI Fingerprint Commands**: New `fingerprint` subcommand with operations:
  - `extract`: Extract fingerprint from CSV data with privacy controls
  - `validate`: Validate DSF file integrity and checksums
  - `info`: Display fingerprint metadata and statistics
  - `diff`: Compare two fingerprints
  - `evaluate`: Evaluate fidelity of synthetic data against fingerprint

### Changed

- Bumped all Rust crate versions to 0.2.0

## [0.1.1] - 2026-01-21

### Changed

- Bumped all Rust crate versions to 0.1.1 for consistency

### Added

- **Python Wrapper** (`python/datasynth_py`): New Python package for programmatic access to DataSynth
  - `DataSynth` client class for CLI-based batch generation
  - `Config`, `GlobalSettings`, `CompanyConfig`, `ChartOfAccountsSettings`, `FraudSettings` dataclasses matching CLI schema
  - Blueprint system with `retail_small`, `banking_medium`, `manufacturing_large` presets
  - Configuration validation with structured error reporting
  - `OutputSpec` for controlling output format (csv, parquet, jsonl) and sink (path, temp_dir, memory)
  - In-memory table loading via pandas (optional dependency)
  - Streaming support via WebSocket connection to datasynth-server (optional dependency)
  - `pyproject.toml` with optional dependency groups: `cli`, `memory`, `streaming`, `all`, `dev`

### Fixed

- Python wrapper config model now correctly matches CLI schema structure
- `importlib.util` import fixed for optional dependency detection

### Documentation

- Added Python Wrapper Guide (`docs/src/user-guide/python-wrapper.md`)
- Added Python package README (`python/README.md`)

## [0.1.0] - 2026-01-20

### Added

- Initial release of SyntheticData
- Core data generation with statistical distributions based on empirical GL research
- Benford's Law compliance for amount generation
- Industry presets: Manufacturing, Retail, Financial Services, Healthcare, Technology
- Chart of Accounts complexity levels: Small (~100), Medium (~400), Large (~2500)
- Master data generation: Vendors, Customers, Materials, Fixed Assets, Employees
- Document flow engine: P2P (Procure-to-Pay) and O2C (Order-to-Cash) processes
- Intercompany transactions with IC matching and transfer pricing
- Balance coherence: Opening balances, running balance tracking, trial balance generation
- Subledger simulation: AR, AP, Fixed Assets, Inventory with GL reconciliation
- Currency & FX: Exchange rates, currency translation, CTA generation
- Period close engine: Monthly close, depreciation, accruals, year-end closing
- Banking/KYC/AML module with customer personas and AML typologies
- OCEL 2.0 process mining event logs
- Audit simulation: ISA-compliant engagements, workpapers, findings
- Graph export: PyTorch Geometric, Neo4j, DGL formats
- Anomaly injection: 20+ fraud types with full labeling
- Data quality variations: Missing values, format variations, duplicates, typos
- REST/gRPC/WebSocket server with authentication and rate limiting
- Desktop UI with Tauri/SvelteKit
- Resource guards: Memory, disk, CPU monitoring with graceful degradation
- Evaluation framework with auto-tuning recommendations
- CLI tool (`datasynth-data`) with generate, validate, init, info commands
