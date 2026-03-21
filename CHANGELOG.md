# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.0] - 2026-03-21

### Added

**Realism & Coherence — v1.4.0**

#### Master Data
- **Cost center hierarchy generator** — `CostCenterGenerator` produces parent/child cost center trees with department mappings and GL account assignments; output written to `master_data/cost_centers.json`
- **Employee change history** — `EmployeeGenerator` now emits `EmployeeChangeRecord` events (title changes, salary adjustments, department transfers) written to `hr/employee_change_history.json`
- **Employee salary field** — `Employee` model carries `annual_salary_usd` drawn from a log-normal distribution calibrated by seniority band; used as the DBO basis for pension calculations

#### Financial Coherence
- **Multi-period balance carry-forward** — Trial balance closing balances from period N are propagated as opening balances for period N+1; `BalanceTracker` accumulates running account balances across all generated JEs
- **Dunning generator wiring** — `DunningGenerator` is now called after AR aging; dunning runs and letters are written to `subledger/dunning_runs.json` and `subledger/dunning_letters.json`
- **AR/AP reconciliation validation** — Subledger reconciliation validates AR invoice totals against GL control account (1100) and AP invoice totals against GL control account (2000); pass/fail logged and written to `balance/subledger_reconciliation.json`
- **Contract→PO linkage** — `ProcurementContract` records carry `purchase_order_ids` referencing the POs generated under that contract; enables source-to-pay chain traversal
- **Document references output** — `DocumentReference` cross-link records (PO→GR, GR→Invoice, Invoice→Payment, SO→Delivery, Delivery→Invoice) written to `document_flows/document_references.json`

#### Manufacturing
- **Moving-average inventory cost** — `InventoryMovementGenerator` applies each goods receipt to update the moving-average unit cost on the inventory position; subsequent goods issues use the updated AVCO
- **Production order ↔ inventory movement cross-refs** — `ProductionOrder` records carry `inventory_movement_ids`; `InventoryMovement` records carry `production_order_id` for bidirectional traceability

#### Audit & Standards
- **ISA mappings output** — ISA standard reference records (number, title, series) for all 34 ISA standards written to `audit/isa_mappings.json`
- **SoD/COSO control mappings** — `ControlExporter::export_standard()` always runs when controls are generated, producing `sod_conflict_pairs.csv`, `sod_rules.csv`, and `coso_control_mapping.csv` alongside the control master data

#### Graph Export
- **Employee/vendor/customer node properties enriched** — `EmployeeGenerator`, `VendorGenerator`, and `CustomerGenerator` now implement `ToNodeProperties`; department, salary band, vendor tier, and customer segment are exposed as typed graph properties
- **JE→Employee edges** — Journal entries carry `created_by_employee_id`; the graph builder emits `JournalEntry→Employee` edges (type `CreatedBy`) for approval and segregation-of-duties analytics
- **Control→JE edges** — `InternalControl` records linked to matching journal entries via `TestedBy` edges in the transaction graph
- **Velocity features** — Graph node feature vectors include transaction velocity (entries per day over rolling 30d window) and amount velocity for anomaly detection models
- **PageRank and degree centrality** — Graph builder computes approximate PageRank and in/out-degree centrality for all entity nodes and attaches them as node features

#### Dead Code Activation
- **AccrualGenerator wired** — period-end accruals (prepaid insurance, accrued wages, accrued utilities, deferred revenue) now generated per company. Were fully implemented but never called from the orchestrator.
- **ProjectRevenueGenerator wired** — PoC/ASC 606 revenue recognition entries now generated for active projects. Were implemented but never invoked.
- **Quality issues/labels collected** — `QualityIssue` records from data quality injection now captured and written to `labels/quality_issues.json` and `labels/quality_labels.json`. Were generated but discarded.
- **ISA PCAOB mappings output** — 34 ISA standards and their PCAOB equivalents now written to `audit/isa_mappings.json` and `audit/isa_pcaob_mappings.json`

#### Config Wiring
- **VendorNetwork config** — `vendor_network` YAML config (tier counts, cluster ratios, concentration limits) now passed to `VendorGenerator.set_network_config()`. Was parsed and validated but never applied.
- **CustomerSegmentation config** — `customer_segmentation` YAML config (value segments, lifecycle rates) now passed to `CustomerGenerator`. Was parsed and validated but never applied.
- **LLM provider selection** — `HttpLlmProvider` now instantiated when `llm.provider = "openai"` and API key is available. Was always using `MockLlmProvider` regardless of config.
- **Segment depreciation threaded** — `total_depreciation` from close engine now passed to segment generator. Was `None`.

#### Audit & Standards
- **AuditScope model** — new `AuditScope` struct linking engagements to CRA via `scope_id`. Unblocks graph edges 132 (assessment→scope) and 134 (engagement→scope).
- **Equity split** — Balance sheet equity section now shows 3 components (Share Capital 10%, APIC 30%, Retained Earnings 60%) instead of a single plug line.

### Fixed

- **AuditScope node synthesizer** — `V130NodeSynthesizer` now registers `AuditScope` nodes (type code 403, external ID = `AuditScope.id`) in the graph id_map; edges 132 (`ASSESSMENT_ON_SCOPE`: CRA→scope) and 134 (`ENGAGEMENT_HAS_SCOPE`: engagement→scope) now emit instead of returning zero edges
- **Quality labels output** — `labels/quality_labels.json` now written alongside `quality_issues.json`; each `QualityIssue` is mapped to a `QualityIssueLabel` with the corresponding `LabeledIssueType` and severity level
- **Heterogeneous DGL export** — `DGLExportConfig.heterogeneous` is now read from `graph_export.dgl.heterogeneous` in the YAML config (previously hardcoded to `false`); new `DglExportConfig` struct added to `GraphExportConfig`

### Changed
- `Employee.annual_salary_usd` is now always populated (was previously `Option<Decimal>` and often absent)
- `InventoryPosition.unit_cost` reflects moving-average cost after each `GoodsReceipt` movement (was static)
- `ProcurementContract.purchase_order_ids` populated by `ContractGenerator` (was always empty `Vec`)
- `BankAccount.gl_account` populated from account type (was always `None`)
- Removed `kyc_profiles` and `bank_statement_lines` from CLAUDE.md export list (embedded in parent files, not standalone outputs)
- `FinancialStatement` balance sheet now has `BS-SC` (Share Capital), `BS-APIC` (Additional Paid-In Capital), `BS-RE` (Retained Earnings) instead of single `BS-TE` equity line

## [1.3.1] - 2026-03-20

### Fixed

**Critical Fixes**
- Fixed `expect()` panic on empty invoice slice in AP payment generator
- Fixed non-deterministic RNG in `ReferenceFormat::Random` — now uses deterministic hash instead of `rand::rng()`
- Fixed non-deterministic RNG in banking phone format generator — uses seeded `ChaCha8Rng`
- Fixed income tax provision posting to wrong GL account (`SALES_TAX_PAYABLE` 2100 → `INCOME_TAX_PAYABLE` 2130)
- Added year upper-bound validation (`start_date + period_months` must not exceed year 9999)
- Added `won_back_rate` to customer lifecycle `validate_sum_to_one` check

**Production Hardening**
- Server REST/gRPC endpoints now use `PhaseConfig::from_config()` instead of hardcoded flags
- Replaced `Mutex::lock().unwrap()` with poison-safe variant in auth cache
- Added `tracing::info!`/`debug!` logging to all 12 audit generators and FX rate service
- Large optional phases (ESG, treasury, project accounting, OCPM, S2C) now gated on degradation level
- Added IC elimination net-zero validation (warns if elimination debits ≠ credits)
- Added financial statement BS equation coherence check (assets = liabilities + equity)
- Fingerprint synthesis seed now logged for reproducibility
- Added `validate_source_to_pay` config validation function
- Regime change dates now validated as parseable `NaiveDate`
- Subledger reconciliation logs pass/fail summary
- Added `Vec::with_capacity()` hints for period-close allocations
- `generate_counterfactuals` now configurable from YAML (`scenarios.generate_counterfactuals`)
- Fingerprint `--sign` flag warning made explicit about implementation status

**Stub Resolution (26 items)**
- Wired `Evaluator::run_evaluation_with_amounts()` to `BenfordAnalyzer` (was empty placeholder)
- ECL and deferred tax rollforward generators now accept optional prior-period closing balances
- Fingerprint evaluate command reads all CSV files (was first only)
- Added 5 foreign key fields for graph edge resolution: `SampledItem.sampling_plan_id`, `RelatedPartyTransaction.journal_entry_id`, `BankAccount.gl_account`, `ProcurementContract.purchase_order_ids`
- Segment D&A now distributable from actual depreciation data (with heuristic fallback)
- Provision unwinding-of-discount now computable from prior opening balance × discount rate
- AutoTuner domain-gap placeholder replaced with `SetToTarget` strategy
- Custom quality gate metrics return `None` (skip) with warning instead of silent failure
- Documented: equity plug on BS, fiscal calendar 4-4-5 approximation, lease current/non-current split, GAAP/IFRS equity_impact for OCI, streaming orchestrator scope, Black-Scholes proxy, lunar calendar approximation

### Added

**Causal DAG Extension**
- 10 new causal DAG nodes: `materiality_threshold`, `inherent_risk`, `combined_risk`, `sample_size_factor`, `opinion_severity`, `gross_margin`, `debt_ratio`, `ecl_provision_rate`, `going_concern_risk`, `tax_rate`
- 11 new causal edges wiring the full audit chain: inherent_risk → combined_risk → sample_size → misstatement → opinion; bad_debt → ECL; interest_rate → debt_ratio → going_concern; GDP/revenue → gross_margin
- `MaterialityThresholdChange`, `AuditStandardChange`, `TaxRateChange` intervention type mappings
- `CreditCrunch` macro shock now drives `ecl_provision_rate`, `going_concern_risk`, `debt_ratio`
- 3 audit scenario packs: `audit_scope_change.yaml`, `control_failure_cascade.yaml`, `going_concern_trigger.yaml`
- `Audit` variant added to `NodeCategory` enum

**Publish Script**
- Fixed crate dependency order: `datasynth-graph` moved from Tier 3 to Tier 5 (depends on `datasynth-generators`)
- Added sparse index refresh (`cargo update --dry-run`) after each crate publish to prevent stale index failures
- Increased `PUBLISH_DELAY` from 45s to 60s

## [1.3.0] - 2026-03-19

### Added

**Enterprise Group Audit Simulation** — 32 items across 5 tiers transforming DataSynth into a comprehensive group audit simulation platform.

#### Tier 0 — Bug Fixes & Wiring
- **Audit output completeness**: All 15 `AuditSnapshot` fields now serialized (was 6/15)
- **Subledger settlement**: Payment application reduces `APInvoice`/`ARInvoice` `amount_remaining`
- **IC eliminations → GL**: `EliminationEntry` records converted to `JournalEntry` with `is_elimination` flag
- **Close engine**: Period-close phase with tax provision stub and income statement closing entries
- **Opening balances**: Converted to balanced JEs using CoA account type lookup (contra-asset aware)
- **Test guardrails**: `scripts/test-safe.sh` sequential per-crate runner, `.cargo/config.toml` with `RUST_TEST_THREADS=2`

#### Tier 1 — Group Audit Core
- **ISA 600 Component Auditor**: `ComponentAuditor`, `GroupAuditPlan`, `ComponentInstruction`, `ComponentAuditorReport` with materiality allocation and scope assignment (full/specific/analytical)
- **GroupStructure ownership model**: Parent-subsidiary relationships with `ConsolidationMethod` (full/equity/fair value) and NCI derivation
- **Consolidated financial statements**: Standalone per-entity + consolidated FS with `ConsolidationSchedule` (pre-elimination → eliminations → post-elimination)
- **Deferred tax engine (IAS 12 / ASC 740)**: 5-8 temporary differences per entity, ETR reconciliation with permanent differences, rollforward schedule, country-aware statutory rates
- **AR/AP aging reports**: Wired existing `ARAgingReport::from_invoices()` / `APAgingReport::from_invoices()` into orchestrator
- **Depreciation runs + inventory valuation**: Per-asset depreciation schedules, IAS 2 lower-of-cost-or-NRV valuation
- **Customer receipts output**: AR receipt payments separated to `document_flows/customer_receipts.json`

#### Tier 2 — Standards Expansion
- **IFRS 3 / ASC 805 — Business Combinations**: PPA with fair value step-ups, goodwill computation, Day 1 JEs, contingent consideration
- **IFRS 8 / ASC 280 — Segment Reporting**: Operating segments (geographic + product line) with reconciliation to consolidated totals
- **IFRS 9 / ASC 326 — Expected Credit Loss**: Simplified approach with provision matrix by aging bucket, forward-looking scenario weighting, ECL provision movements
- **IAS 19 / ASC 715 — Pensions**: Defined benefit plans with DBO rollforward, plan assets, pension expense components, OCI remeasurements
- **IAS 37 / ASC 450 — Provisions & Contingencies**: 3-10 provisions per entity with framework-aware recognition thresholds (IFRS >50%, US GAAP >75%)
- **ASC 718 / IFRS 2 — Stock Compensation**: Stock grants (RSUs/Options/PSUs) with graded vesting and expense recognition
- **IAS 21 — Functional Currency**: Per-entity functional currency, current-rate method translation, CTA as OCI

#### Tier 3 — Audit Documentation & Outputs
- **ISA 210 — Engagement Letters**: Template-driven with scope, fees, and framework
- **ISA 560 / IAS 10 — Subsequent Events**: Adjusting (IAS 10.8) and non-adjusting (IAS 10.21) events
- **ISA 402 — Service Organization Controls**: SOC 1 Type II reports with control objectives and exceptions
- **Notes to Financial Statements**: 8-13 template-driven notes with typed `NoteTableValue` (Amount/Percentage/Date/Text)
- **Going Concern Indicators**: Financial indicator derivation, 90% clean / 8% mild / 2% severe distribution
- **ISA 540 — Accounting Estimates**: Links all estimate-producing generators with ISA 540 risk factors, assumptions, and retrospective review
- **NCI Measurement**: Enhanced elimination generator using GroupStructure ownership percentages
- **Format Exporters**: `--export-format sap|fec|gobd` CLI flag activating existing SAP BKPF/BSEG/ACDOCA, French FEC, and German GoBD exporters

#### Tier 4 — Evaluation & Professionalization
- **Financial Ratio Evaluator (ISA 520)**: 12 ratios (liquidity, activity, profitability, leverage) with industry-specific reasonableness bounds
- **JE Risk Scoring**: 7 risk attributes (round number, unusual hour, weekend, non-standard user, below threshold, manual to automated, large round-trip) with anomaly separability check
- **Materiality-Stratified Sampling**: 4 strata with above-materiality coverage, anomaly stratum/entity/temporal coverage
- **Trend Plausibility**: Revenue stability, expense ratio consistency, BS growth, directional consistency checks
- **Audit Preset**: `audit_group_overlay()` modifier + `--preset audit-group` CLI flag enabling all audit features

#### Configuration Unification
- **PhaseConfig::from_config()**: Single source of truth — all 19 phase flags derived from GeneratorConfig sections
- Removed redundant AND-logic double-checks in banking and graph export phases
- Documented unused AuditGenerationConfig sub-fields (reserved for future fine-grained control)
- Added `audit:`, `hr:`, `treasury:`, `project_accounting:` config examples to documentation

#### Quality Fixes (post-initial implementation)
- **Audit opinion generator (ISA 700/705/706/701)**: Derives opinion from findings severity, going concern, scope limitations. Generates Key Audit Matters (ISA 701). PCAOB/ICFR opinion for US-listed entities
- **SOX 302/404 assessments**: CEO/CFO certification (302) and ICFR effectiveness assessment (404) wired into orchestrator
- **ECL + provision JEs merged into GL**: Allowance and provision balances now flow to trial balance and financial statements
- **Tax rate standardized**: Consistent 21% statutory rate across all generators (was inconsistent 21%/25%)
- **Hardcoded proxies replaced**: All $10M revenue proxies, 5x asset heuristics, and positional weights replaced with actual trial balance data
- **Cash flow from actual data**: CF statement now derives from real BS movements (AR/AP/inventory changes, FA depreciation) instead of random ranges
- **Depreciation in period close**: Per-asset depreciation JEs generated from fixed asset register
- **Pension from payroll**: DBO uses actual avg salary from payroll, prorated for sub-annual periods
- **Notes populated**: Notes to FS context wired with deferred tax, provisions, pensions, related parties
- **Going concern from financials**: Indicators derived from actual working capital, net income, operating CF, debt ratios
- **IC equity eliminations**: Investment/equity amounts computed from subsidiary net assets (was empty HashMaps)
- **Revenue sign fixed**: Income statement shows positive revenue (was negative due to credit-normal convention)
- **Ratio evaluator**: Current assets properly filtered (10xx-13xx, was total assets)
- **Known simplifications documented**: ECL rollforward, BS retained earnings plug, Black-Scholes proxy, fiscal calendar, lease current portion, copula/lunar approximations

#### Audit Methodology Enhancements
- **Combined Risk Assessment (ISA 315)**: CRA per account area and assertion (Minimal/Low/Moderate/High), 12 account areas, 24-30 assessments per entity, automatic significant risk flagging (ISA 240/315)
- **Materiality calculations (ISA 320)**: Benchmark selection (PretaxIncome/Revenue/Assets), PM at 65% of overall, clearly trivial at 5%, tolerable error, SAD nominal, normalized earnings with adjustment tracking
- **Sampling methodology (ISA 530)**: Key item identification (>TE), MUS/systematic selection, sample size driven by CRA level, misstatement rates correlated with risk, sampling interval computation
- **SCOTS classification (ISA 315)**: 8-9 standard SCOTs with significance level, processing method (Automated/Manual/Hybrid), 4-stage critical path, estimation complexity indicators
- **Unusual item detection**: Multi-dimensional flagging (Size/Timing/Relationship/Frequency/Nature) with severity scoring, anomaly correlation, 5-10% flagging rate
- **Analytical relationships (ISA 520)**: 8 standard ratios (DSO, DPO, margins, turnover) with historical trends, expected ranges, variance explanations, and supporting non-financial metrics
- **Coherence chain**: CRA → sampling approach → misstatement detection → control assessment → audit opinion — end-to-end data-driven chain

#### Graph Export Onboarding
- **28 new entity types** onboarded to graph export pipeline (was ~58% coverage, now ~95%)
- **L1 Governance** (13 types): CRA, materiality, audit opinion, KAMs, SOX 302/404, going concern, component auditor, group audit plan, component instruction/report, engagement letter, group structure
- **L2 Process** (10 types): Sampling plans, sampled items, SCOTS, unusual items, analytical relationships, accounting estimates, subsequent events, service organizations, SOC reports
- **L3 Financial** (11 types): Consolidation schedule, operating segments, ECL model, provisions, pensions, stock grants, temporary differences, business combinations, NCI measurements, FS notes, currency translation
- **27 cross-entity edge types** (codes 160-187): CRA→entity, opinion→engagement, KAM→opinion, sampling→CRA, unusual→JE, segment→entity, provision→entity, pension→entity, etc.

### New Output Files
- `audit/`: engagement_letters, subsequent_events, service_organizations, soc_reports, user_entity_controls, going_concern_assessments, accounting_estimates, component_auditors, group_audit_plan, component_instructions, component_reports, confirmations, confirmation_responses, procedure_steps, samples, analytical_results, ia_functions, ia_reports, related_parties, related_party_transactions, audit_opinions, key_audit_matters, sox_302_certifications, sox_404_assessments, materiality_calculations, combined_risk_assessments, sampling_plans, sampled_items, significant_transaction_classes, unusual_items, analytical_relationships
- `financial_reporting/`: standalone/{entity}_financial_statements, consolidated/consolidated_financial_statements, consolidated/consolidation_schedule, segment_reporting, notes_to_financial_statements
- `tax/`: temporary_differences, etr_reconciliation, deferred_tax_rollforward, deferred_tax_journal_entries
- `accounting_standards/`: business_combinations, purchase_price_allocations, ecl_models, ecl_provision_movements, ecl_journal_entries, fx/currency_translation_results, provisions, contingent_liabilities, provision_movements
- `hr/`: pension_plans, pension_obligations, plan_assets, pension_disclosures, stock_grants, stock_comp_expense
- `subledger/`: ar_aging, ap_aging, depreciation_runs, inventory_valuation
- `intercompany/`: group_structure, nci_measurements
- `document_flows/`: customer_receipts

## [1.2.0] - 2026-03-15

### Added
- **`datasynth-graph-export` crate** — New standalone crate for unified graph export pipeline, replacing the monolithic adapter pattern
  - **Pipeline orchestrator**: Budget-managed, topologically-sorted export pipeline with phase ordering (audit before banking)
  - **30 property serializers**: Typed property extraction for all domain models — accounting, P2P, O2C, S2C, H2R, manufacturing, audit, banking, controls, and risk
  - **13 node synthesizers**: Adapter-originated entity types including AML alerts, collusion rings, compliance, ESG, intercompany, KYC profiles, OCEL events, projects, red flags, subledger reconciliation, tax, temporal events, and treasury
  - **10 edge synthesizer domains**: Accounting, audit trail, banking, document chain, entity relationships, H2R, manufacturing, process sequence, risk-control, and S2C with topological ordering
  - **Audit procedure edge synthesizer**: 15 edge types for ISA-based audit procedure linking
  - **Audit procedure node synthesizer**: 9 node types with property serializers for procedure steps, samples, confirmations, and analytical results
  - **Post-processors**: EffectiveControlCount, AnomalyFlag, RedFlag, and DuplicateEdge post-processing passes
  - **OCEL exporter**: Process mining export from graph structure
  - **Budget manager**: Configurable node/edge budgets with rebalancing API and enforcement
  - **IdMap**: Collision-free ID mapping between domain UUIDs and graph node indices
  - **Config schema**: Full `GraphExportConfig` with per-domain enable/disable, budget limits, and output format selection
- **Audit procedure models** (`datasynth-core`)
  - `ExternalConfirmation` and `ConfirmationResponse` (ISA 505)
  - `AuditProcedureStep` and `AuditSample` (ISA 330/530)
  - `AnalyticalProcedureResult`, `InternalAuditFunction`, `InternalAuditReport` (ISA 520/610)
  - `RelatedParty` and `RelatedPartyTransaction` (ISA 550)
  - `EvidenceStatus` and `CustomerStatus` enums
- **Audit procedure generators** (`datasynth-generators`)
  - `ConfirmationGenerator` (ISA 505)
  - `ProcedureStepGenerator`, `SampleGenerator`, `AnalyticalProcedureGenerator` (ISA 330/530/520)
  - `InternalAuditGenerator` and `RelatedPartyGenerator` (ISA 610/550)
- **Runtime integration**: 9 new audit entity generators wired into `AuditSnapshot` and orchestrator
- **Hypergraph builder**: 9 audit procedure entity types and 15 edge types added
- **Graph builder**: 8 new domain builder methods (tax, treasury, ESG, project, intercompany, temporal, AML, KYC)
- **Budget rebalancing API** with enforced phase ordering (audit before banking)
- **Core model enhancements**:
  - Relationship fields on `AuditFinding`, `DocumentRef` enum, and approval tracking on `JournalEntry`
  - Test history, effectiveness metrics, and risk linkage on `InternalControl`
  - Continuous risk scores, risk names, and `RiskStatus` on `RiskAssessment`
- Integration tests: full pipeline tests, budget enforcement, property serializer coverage, edge synthesizer tests
- Document FK population in P2P/O2C chains with `created_by_employee_id`

### Changed
- Graph entity naming uses snake_case; dates serialized as RFC 3339

### Fixed
- Hardcoded status strings in graph export replaced with `serde_json::to_value()`
- Dashboard-style camelCase properties removed from HypergraphBuilder

## [1.1.0] - 2026-03-10

### Added
- **Compliance Regulations Framework** (`datasynth-core`, `datasynth-standards`, `datasynth-generators`, `datasynth-graph`, `datasynth-config`, `datasynth-runtime`, `datasynth-cli`)
  - **StandardId**: Canonical `"{BODY}-{NUMBER}"` identifier (e.g., `IFRS-16`, `SOX-404`, `ISA-315`) with `body()`, `number()`, `parse()`, and `From` impls
  - **ComplianceStandard**: Full metadata model with issuing body, category, domain, temporal versions, cross-references, mandatory/permitted jurisdictions, and **applicable account types / business processes** for cross-domain linking
  - **TemporalVersion**: Jurisdiction-aware date resolution with `is_active_at()` / `is_active_at_in()`, early adoption dates, and per-country effective date overrides
  - **JurisdictionProfile**: Country-specific compliance profiles (accounting framework, audit framework, tax rate, supranational memberships) for 10 countries (US, DE, GB, FR, JP, IN, SG, AU, BR, KR)
  - **StandardRegistry**: Central catalog with ~45 built-in standards, 9 cross-references, temporal lookup, supersession chain resolution, and jurisdiction-aware filtering
  - **ComplianceAssertion**: 15 ISA 315 assertion types across transaction, balance, disclosure, and presentation categories with ML feature codes
  - **ComplianceFinding**: Audit finding model with SOX/ISA deficiency classification (MaterialWeakness, SignificantDeficiency, ControlDeficiency), remediation tracking, and financial impact
  - **RegulatoryFiling**: Filing model with 10 filing types (10-K, 10-Q, Jahresabschluss, E-Bilanz, Liasse fiscale, UK Annual Return, CT600, 有価証券報告書) and deadline tracking
  - **CrossReference**: Typed standard relationships (Converged, Related, Complementary, DerivedFrom, AuditMapping, ControlFrameworkMapping) with convergence levels
  - **RegulationGenerator**: Produces compliance standard records, cross-reference records, and jurisdiction profile records for CSV/JSON export
  - **ProcedureGenerator**: Generates audit procedure instances from 9 ISA-based templates (substantive detail, analytical, controls test, inspection, confirmation, recalculation, observation, inquiry, cutoff test) with step definitions and sampling parameters
  - **ComplianceFindingGenerator**: Generates compliance findings from 10 templates with condition/criteria/cause/effect structure, deficiency classification, and remediation status
  - **FilingGenerator**: Generates regulatory filing records for 8 filing types across 5 jurisdictions (US, DE, FR, GB, JP) with status progression and deadline tracking
  - **ComplianceGraphBuilder**: Builds compliance graph layer with Standard, Jurisdiction, AuditProcedure, and Finding nodes; CrossReference, Supersedes, MapsToStandard, TestsCompliance, and FindingOnStandard edges
  - **ComplianceRegulationsConfig**: Full configuration schema with sub-configs for standards selection, audit procedures, findings, filings, graph, and output
  - Config validation for jurisdiction codes, reference dates, standard categories, sampling methods, and rate parameters
  - Runtime wiring in `EnhancedOrchestrator` with `phase_compliance_regulations()` producing `ComplianceRegulationsSnapshot`
  - CLI output of 7 JSON files to `compliance_regulations/` directory (standards, cross-references, jurisdictions, procedures, findings, filings, graph)
  - 67+ tests across all compliance modules
- **Cross-Domain Compliance Graph Linking** (`datasynth-core`, `datasynth-standards`, `datasynth-graph`, `datasynth-config`, `datasynth-runtime`)
  - **ToNodeProperties for compliance models**: `ComplianceStandard` (type code 510), `ComplianceFinding` (511), `RegulatoryFiling` (512), `JurisdictionProfile` (513) — all compliance models now export typed camelCase property maps for graph node conversion
  - **Standard-to-account mapping**: Every built-in standard (IFRS 9/13/15/16/17/18, IAS 36, ASC 606/326/360/740/805/810/718/820/842, ISA 240–720, SOX, PCAOB) declares which GL account types it governs (`applicable_account_types`) and which business processes it applies to (`applicable_processes`)
  - **5 new cross-domain edge types**: `GovernedByStandard` (Standard→Account), `ImplementsStandard` (Standard→Control), `FiledByCompany` (Filing→Company), `FindingAffectsControl`, `FindingAffectsAccount`
  - **ComplianceGraphBuilder cross-domain support**: `add_account_links()`, `add_control_links()`, `add_filings()` with company node linking; `AccountLinkInput`, `ControlLinkInput`, `FilingNodeInput` input types
  - **Hypergraph compliance integration**: Standards added to Layer 1 (GovernanceControls, type code 505), findings and filings added to Layer 2 (ProcessEvents, type codes 508/507); cross-layer edge resolution for Standard→Account and Standard→Control edges in `build_cross_layer_edges()`
  - **Full enterprise graph traversal**: Enabled paths like Company → Filing → Jurisdiction → Standard → Account → JournalEntry and Standard → Control → Finding
  - **3 new config fields**: `include_account_links`, `include_control_links`, `include_company_links` on `ComplianceGraphConfig` (all default `true`)
  - Orchestrator wires compliance data into hypergraph export, compliance regulations phase now runs before hypergraph export for data availability
- **Entity-aware anomaly injection**: EntityTargetingManager selects repeat-offender entities, AnomalyCoOccurrence queues correlated anomaly pairs with lag days, TemporalClusterGenerator boosts injection rates during period-end windows (`datasynth-generators`)
- `QuoteLineItem.id` field with deterministic UUIDs from `item_uuid_factory` (`datasynth-core`, `datasynth-generators`)
- `UnmatchedICItem` exported from `datasynth-eval` coherence module
- Integration tests for Parquet roundtrip (5 tests), multi-framework standards (11 tests), OCEL 2.0 roundtrip (9 tests), and coherence evaluators (8 tests)

### Changed
- `ExpenseReportGenerator` now uses stored `config` and `country_pack` fields via `generate_from_config()` convenience method (`datasynth-generators`)
- `EarnedValueGenerator` respects `config.frequency` for measurement date calculation (weekly/biweekly/monthly) (`datasynth-generators`)
- `RevenueGenerator` uses `config.method` and `config.completion_measure` instead of hardcoded values, and generates deterministic UUIDs via `uuid_factory` (`datasynth-generators`)
- `OracleExporter` wires `batch_counter` to `je_batch_id` when `include_batches` is enabled (`datasynth-output`)
- `ICMatchingEvaluator` applies tolerance to classify unmatched items as within/outside tolerance, with `discrepancy_count` reflecting only outside-tolerance items (`datasynth-eval`)

### Fixed
- `fiscal_year` in `ComplianceFinding` now uses `Datelike::year()` instead of format/parse roundtrip
- Compliance reference date validation uses `chrono::NaiveDate::parse_from_str` consistent with rest of validation module
- Supersession chain cycle guard with `HashSet<StandardId>` prevents infinite loops in both backward and forward walks
- Compliance findings now generated for all companies (was previously only first company)
- Material weakness + significant deficiency rate sum validated to be <= 1.0

### Removed
- 9 `#[allow(dead_code)]` annotations across generators, output, and eval crates (Pillar 1: 6, Pillar 2: 3)

## [1.0.0] - 2026-03-06

### Added
- **Process evolution & organizational event generation** (Phase 24): workflow changes, automation events, policy updates, acquisitions, divestitures, reorganizations (`datasynth-runtime`)
- **Disruption event generation** (Phase 24b): new `DisruptionGenerator` producing outage, migration, process change, recovery, and regulatory disruption events (`datasynth-generators`, `datasynth-runtime`)
- **Counterfactual pair generation** (Phase 25): generates (original, mutated) journal entry pairs for ML training when `generate_counterfactuals` is enabled (`datasynth-runtime`)
- **Fraud red-flag indicators** (Phase 26): `RedFlagGenerator` attaches risk indicators to P2P/O2C document chains when fraud labels exist (`datasynth-runtime`)
- **Collusion ring generation** (Phase 26b): new `CollusionRingGenerator` creates coordinated fraud networks from employee/vendor pools when `fraud.clustering_enabled` (`datasynth-generators`, `datasynth-runtime`)
- **Bi-temporal vendor versioning** (Phase 27): `TemporalAttributeGenerator` creates version chains with valid-time/transaction-time dimensions (`datasynth-runtime`)
- **Entity relationship graph** (Phase 28): `EntityGraphGenerator` builds relationship graphs with strength scores and cross-process links (P2P↔O2C via inventory) (`datasynth-runtime`)
- **Industry transaction factory** (Phase 29): dispatches industry-specific GL accounts for Retail, Manufacturing, Healthcare, FinancialServices (`datasynth-generators`, `datasynth-runtime`)
- 11 integration tests for all newly wired generators covering enabled/disabled toggles, deterministic output, and structural validation (`datasynth-runtime`)
- Benefit enrollment generation in HR pipeline (`datasynth-runtime`)
- BOM component and inventory movement generation in manufacturing pipeline (`datasynth-runtime`)
- Cash forecasting from AR/AP aging and cash pooling with sweep transactions (`datasynth-runtime`)
- Bank guarantee and intercompany netting run generators (`datasynth-generators`)
- CorrelationPreservation quality gate wired to statistical correlation analysis (`datasynth-eval`)
- EntityGraphBuilder wired with Company→CompanyConfig mapping and IC relationships (`datasynth-runtime`)
- Country pack support for EmployeeGenerator and ExpenseReportGenerator (`datasynth-generators`)
- Inline anomaly injection in streaming mode (`datasynth-runtime`)
- Complete crates.io metadata (keywords, categories) for all crates

### Changed
- Treasury phase now accepts subledger data for AR/AP-driven cash forecasting
- Streaming orchestrator applies anomaly injection inline during JE generation instead of skipping
- Bank reconciliation generator already wired (confirmed existing)

### Performance
- SmallVec optimization for `ExpenseReport.line_items` and `QualityInspection.characteristics` (avoids heap allocation for ≤4 items)
- Zero-copy `std::mem::take()` for master data transfer to result (eliminates clone of full vendor/customer/material/asset/employee vectors)
- `Arc::try_unwrap()` for chart of accounts when refcount is 1

### Fixed
- Temporal version chains now config-driven (uses `TemporalAttributeConfigBuilder` instead of `with_defaults()`) producing multi-version chains
- Collusion rings now simulate transactions during `advance_month()` for Forming/Active/Escalating states
- Entity graph no longer produces duplicate edges; transaction summaries extracted from P2P/O2C document flows for computed relationship strengths
- Cross-process links auto-enable when both P2P and O2C data exist; material_id now propagated from PO items to GR items in P2P generator
- Non-workspace reqwest dependency in datasynth-test-utils
- Benefit enrollment generation now gated behind payroll.enabled config flag
- IC seller withholding tax JEs now debit WHT receivable (was incorrectly credited as payable)
- Cross-process links (P2P↔O2C) no longer fail when delivery dates precede goods receipt dates
- Division-by-zero guard in `CompanySelector` when all company weights are zero (falls back to uniform)
- Division-by-zero guard in anomaly `SplitTransactionStrategy` when entry total debit is zero
- Division-by-zero guard in `SchemeAdvancer` when all fraud scheme probabilities are zero
- Customer generator now produces realistic names with proper first/last name combinations
- Material generator now uses industry-appropriate material descriptions and realistic units of measure
- Vendor generator now produces realistic vendor names, proper bank details, and industry-appropriate payment terms
- Orchestrator correctly sets vendor/customer/employee counts in generation statistics
- Orchestrator clamps `period_months` (1-120) and `statutory_rate` (0.0-1.0) before numeric casts to prevent truncation
- Tax rate constant uses infallible `Decimal::new(25, 2)` instead of fallible `Decimal::from_f64_retain`

## [0.11.1] - 2026-03-03

### Changed

- **Hypergraph type codes** (`datasynth-graph`): Aligned entity type codes with AssureTwin's canonical `entity_registry.rs` — added `JOURNAL_ENTRY` (101), `BANKING_CUSTOMER` (203), `BANK_STATEMENT_LINE` (352), `KYC_PROFILE` (504); renumbered MFG and banking codes for consistency; moved governance codes (COSO/SOX) to Layer 1 block (500–504); fixed `INTERNAL_CONTROL` code from 504 to 503
- **Journal entry nodes** (`datasynth-graph`): Added `add_journal_entry_nodes()` to `HypergraphBuilder` — creates standalone Layer 3 JE nodes with amount, date, anomaly info, and line count for dashboard counting alongside existing hyperedge representation

## [0.11.0] - 2026-03-02

### Added

- **GenerationSession**: Stateful multi-period generation with `.dss` checkpoint files (`datasynth-runtime`, `datasynth-core`)
  - `GenerationPeriod` splits total time span into fiscal-year-aligned periods
  - `SessionState` tracks RNG seeds, balance state, document IDs, entity counts across periods
  - Deterministic seed advancement via `advance_seed()` ensures reproducibility
- **Incremental generation**: `--append --months N` adds more periods to an existing session (`datasynth-cli`)
- **Fraud scenario packs**: 5 built-in YAML packs (`--fraud-scenario revenue_fraud|payroll_ghost|vendor_kickback|management_override|comprehensive`) (`datasynth-config`)
  - Deep-merge into config with `apply_fraud_packs()`, compatible with `--fraud-rate` override
- **StreamPipeline**: Phase-aware streaming via `PhaseSink` trait (`datasynth-runtime`)
  - File target (JSONL), HTTP target, and no-op sink
  - Tracks items emitted, bytes sent, phases completed
- **OCEL 2.0 enrichment** (`datasynth-ocpm`)
  - Lifecycle state machines for PurchaseOrder, SalesOrder, VendorInvoice with probabilistic transitions
  - Multi-object correlation events: ThreeWayMatch, PaymentAllocation, BankReconciliation
  - Resource pool workload modeling with RoundRobin, LeastBusy, SkillBased assignment
  - Enriched OcpmEvent: `from_state`, `to_state`, `resource_workload`, `correlation_id` fields
- **CLI flags**: `--fiscal-year-months`, `--append`, `--months`, `--fraud-scenario`, `--fraud-rate`, `--stream-file` (`datasynth-cli`)
- **SessionSchemaConfig**: Config schema for session behavior (checkpointing, per-period output) (`datasynth-config`)
- 13 integration tests across session, OCEL, and fraud pack modules
- **4 new evaluators** (`datasynth-eval`)
  - Multi-period coherence: validates opening balance = prior closing balance, sequential document IDs, consistent entity references
  - Fraud pack effectiveness: detection rate at configurable thresholds, false positive analysis, pack coverage metrics
  - OCEL enrichment quality: state transition coverage, correlation event linking accuracy, resource utilization distribution
  - Causal intervention magnitude: KPI delta vs. expected magnitude, propagation path verification, constraint preservation checks
- **DiffEngine completion** (`datasynth-runtime`): Record-level diffs (added/removed/modified) and aggregate metric comparison between baseline and counterfactual output directories
- **ConfigMutator constraint validation** (`datasynth-runtime`): `preserve_accounting_identity`, `preserve_document_chains`, `preserve_period_close`, `preserve_balance_coherence` constraints enforced during counterfactual generation
- **Minimal DAG preset** (`datasynth-core`): 6-node causal DAG for lightweight counterfactual analysis
- **ProcessChange and RegulatoryChange** causal mapping (`datasynth-core`): Two new `InterventionType` variants for modeling process and regulatory interventions
- **Desktop UI pages** (`datasynth-ui`): Fraud Scenario Packs, Causal DAG visualization, Generation Session management, Streaming monitor, OCPM enrichment configuration
- **Python v1.8.0** (`python/`): `with_fraud_packs()`, `with_scenarios()`, `with_streaming()` blueprints; `fraud_scenario`, `fraud_rate`, `stream_file` parameters on `generate()`
- **5 new documentation pages** (`docs/`): Fraud Scenario Packs, Counterfactual Scenarios, OCEL 2.0 Enrichment, Streaming Pipeline, Evaluation Framework

### Fixed

- **Automated posting time distribution** (`datasynth-core`): Batch-processing times now spread across overnight (0-6) and evening (20-23) windows instead of spiking at hour 23 due to broken clamping arithmetic
- **Weekend entry filtering** (`datasynth-generators`): Default `BusinessDayCalculator` with US holidays always created, so weekend entries are filtered even without explicit `temporal_patterns` config
- **Journal entry line item enrichment** (`datasynth-generators`): `account_description`, `cost_center`, `profit_center`, `line_text`, `value_date`, and `assignment` fields now populated via `enrich_line_items()` on both standard and document-flow JEs
- **Document type derivation** (`datasynth-generators`): `document_type` derived from business process (P2P→KR, O2C→DR, H2R→HR) and document source (WE, KZ, WL, DZ) instead of hardcoded "SA"
- **Persona naming consistency** (`datasynth-core`): `Display` impl for `UserPersona` outputs snake_case (`automated_system`, `junior_accountant`) instead of concatenated Debug format
- **Banking transaction type** (`datasynth-banking`): Added `transaction_type` field to `BankTransaction`, derived from channel and category (e.g., `CARD_PRESENT_SHOPPING`)
- **Banking sparse field population** (`datasynth-banking`): `device_id`, `ip_address`, `location_country`, `location_city`, `timestamp_settled`, `auth_code`, and `mcc` now populated based on transaction channel
- **Internal controls application** (`datasynth-runtime`): `ControlGenerator::apply_controls()` wired into orchestrator pipeline; JEs receive `control_ids` when `internal_controls.enabled=true`

## [0.10.0] - 2026-03-02

### Added

- **Counterfactual Simulation Engine** (`datasynth-core`, `datasynth-config`, `datasynth-runtime`, `datasynth-eval`, `datasynth-cli`)
  - **Core data models**: `Scenario`, `Intervention`, `InterventionTiming`, `OnsetType`, `ScenarioConstraints`, `ScenarioOutputConfig`, `DiffFormat`
  - **InterventionType** tagged enum with 8 variants: `ParameterShift`, `ControlFailure`, `MacroShock`, `EntityEvent`, `ProcessChange`, `RegulatoryChange`, `Composite`, `Custom`
  - **CausalDAG** with 8 `TransferFunction` types (Linear, Exponential, Logistic, InverseLogistic, Step, Threshold, Decay, Piecewise), topological sort via Kahn's algorithm, and forward propagation with lag and strength
  - **Default causal DAG template**: 17 financial process nodes (GDP growth, interest rates, transaction volume, control effectiveness, fraud detection, misstatement risk, etc.) and 16 edges with transfer functions and config bindings
  - **ScenariosConfig schema** with validation: scenario definitions, intervention timing, constraint configuration, causal model presets, probability weights
  - **CausalPropagationEngine**: onset interpolation (Sudden, Gradual, Oscillating), lag-aware propagation, node bounds clamping, config binding resolution
  - **InterventionManager**: timing validation, bounds checking, config path resolution, conflict detection with priority-based resolution
  - **ConfigMutator**: dot-path config mutation with array indexing (e.g., `distributions.amounts.components[0].mu`), null-stripping for JSON roundtrip, custom constraint validation
  - **ScenarioEngine orchestrator**: paired baseline/counterfactual generation, scenario manifest output, DAG loading from presets
  - **ScenarioDiff types**: `ImpactSummary`, `KpiImpact`, `FinancialStatementImpact`, `AnomalyImpact`, `ControlImpact`, `RecordLevelDiff`, `AggregateComparison`, `InterventionTrace`
  - **DiffEngine**: computes summary KPIs, record-level diffs (added/removed/modified), and aggregate metrics between baseline and counterfactual output directories
  - **CLI subcommand**: `datasynth-data scenario {list, validate, generate, diff}` for managing counterfactual scenarios from the command line
  - **59 new tests**: 45 unit tests + 14 integration tests across core, config, runtime, and eval crates

## [0.9.5] - 2026-03-01

### Fixed

- **Mutex poisoning recovery in streaming channel** (`datasynth-core`)
  - All 11 `.expect()` calls on `Mutex::lock()` and `Condvar` waits replaced with `.unwrap_or_else(|p| p.into_inner())` for graceful recovery after thread panics

### Added

- **7 new country packs** (`datasynth-core`)
  - France (FR): EUR, PCG accounting, SIREN/SIRET, 11 national holidays + Easter-relative
  - Japan (JP): JPY (0 decimals), April–March fiscal year, KK/GK legal forms, bonus months
  - China (CN): CNY, CNAPS banking, WFOE/JV legal forms, 5-fund social insurance
  - India (IN): INR with [3,2] number grouping, April–March fiscal, IFSC/UPI banking, GST/TDS
  - Italy (IT): EUR, IRES 24% + IRAP 3.9%, IVA 22%, 13th+14th month payroll, FatturaPA
  - Spain (ES): EUR, IS 25%, IVA 21%, 14 payments, SII reporting
  - Canada (CA): CAD, federal + provincial tax, GST/HST, CPP/EI, bilingual EN/FR

- **4 new holiday calendars with Region enum extension** (`datasynth-core`)
  - `FR`, `IT`, `ES`, `CA` variants added to `Region` enum
  - France: 11 national holidays + Easter Monday + Whit Monday
  - Italy: 12 holidays including Ferragosto, St. Stephen's Day
  - Spain: 10 national holidays including Epiphany, Hispanic Day
  - Canada: 10 holidays including Victoria Day, Canada Day, Thanksgiving (2nd Mon Oct)

- **Progressive tax bracket computation** (`datasynth-generators`)
  - `compute_progressive_tax()` helper iterates brackets in ascending order
  - Country pack income tax brackets wired into payroll generation
  - High/low earners pay different effective rates per country tax tables

- **Credit memo wiring in O2C flow** (`datasynth-generators`)
  - `returns_rate` config now generates `ARCreditMemo` documents in O2C chains
  - Credit memos reference parent invoice with random reason (Return, Damaged, QualityIssue, PriceError)
  - Credit amount bounded to 10–100% of invoice amount

- **4 new generators** (`datasynth-generators`)
  - `OrganizationalEventGenerator`: Acquisition, Divestiture, Reorganization, LeadershipChange, WorkforceReduction, Merger events with multi-phase integration
  - `ProcessEvolutionGenerator`: S-curve automation rollout, workflow type transitions, policy/control changes
  - `DriftEventGenerator`: Meta-generator producing ML ground-truth drift labels from organizational and process events (Statistical, Temporal, Behavioral, Market drift types)
  - `ConfirmationGenerator`: ISA 505 external confirmations (AR 40%, AP 30%, Bank 20%, Legal 10%) with positive/negative response modeling and reconciliation

- **39 new integration tests** (`datasynth-eval`, `datasynth-output`, `datasynth-config`, `datasynth-banking`)
  - `datasynth-eval`: 17 tests for Benford analysis, balance sheet coherence, and comprehensive evaluation
  - `datasynth-output`: 8 tests for CSV/JSON write-readback, Unicode, decimal precision
  - `datasynth-config`: 7 tests for preset loading, validation, and config inheritance
  - `datasynth-banking`: 7 tests for KYC/AML fraud typology behavior (structuring, layering, funnel, round-tripping)

### Changed

- **NetSuite CSV export heap allocation elimination** (`datasynth-output`)
  - `Option<String>` patterns replaced with `.as_deref().unwrap_or("")` to avoid temporary `String` allocations
  - `HashMap::get()` patterns replaced with `.map(|s| s.as_str()).unwrap_or("")`

- **Hot path clone elimination in orchestrator** (`datasynth-runtime`)
  - `journal.entries.clone()` replaced with `std::mem::take()` ownership transfer
  - Large struct clones replaced with `Arc` wrapping for shared ownership

## [0.9.4] - 2026-03-01

### Added

- **Graph property mapping trait and value enum** (DS-001) (`datasynth-core`)
  - `ToNodeProperties` trait for converting typed model structs to `HashMap<String, GraphPropertyValue>` with camelCase keys
  - `GraphPropertyValue` enum supporting String, Int, Float, Decimal, Bool, Date, and StringList variants
  - `node_type_name()` and `node_type_code()` methods for canonical entity identification

- **Extended entity type registry with 50+ variants** (DS-001 through DS-009) (`datasynth-core`)
  - `GraphEntityType` expanded with Tax (7), Treasury (8), ESG (13), Project (5), S2C (4), H2R (4), MFG (4), GOV (5) entity types
  - `numeric_code()`, `node_type_name()`, `from_numeric_code()`, `from_node_type_name()` methods
  - Category helpers: `is_tax()`, `is_treasury()`, `is_esg()`, `is_project()`, `is_h2r()`, `is_mfg()`, `is_governance()`
  - `all_types()` iterator for exhaustive enumeration

- **Edge type registry with 28 new relationship variants** (DS-010) (`datasynth-core`)
  - `RelationshipType` expanded with P2P, O2C, S2C, H2R, MFG, TAX, Treasury, ESG, Project, and GOV edge types
  - `EdgeConstraint` struct with source/target entity types and `Cardinality` enum
  - `constraint()` and `all_constraints()` methods for typed edge validation

- **New model structs** (DS-007, DS-008) (`datasynth-core`)
  - `BomComponent`: multi-level bill-of-materials with phantom assembly support
  - `InventoryMovement`: goods receipt/issue/transfer/return/scrap/adjustment tracking
  - `BenefitEnrollment`: employee benefit plan enrollment with contribution tracking

- **ToNodeProperties implementations for all 10 process families** (DS-001 through DS-009) (`datasynth-core`)
  - Tax domain (7 types): TaxJurisdiction, TaxCode, TaxLine, TaxReturn, TaxProvision, WithholdingTaxRecord, UncertainTaxPosition
  - Treasury domain (8 types): CashPosition, CashForecast, CashPool, CashPoolSweep, HedgingInstrument, HedgeRelationship, DebtInstrument, DebtCovenant
  - ESG domain (13 types): EmissionRecord, EnergyConsumption, WaterUsage, WasteRecord, WorkforceDiversityMetric, PayEquityMetric, SafetyIncident, SafetyMetric, GovernanceMetric, SupplierEsgAssessment, MaterialityAssessment, EsgDisclosure, ClimateScenario
  - Project domain (6 types): Project, ProjectCostLine, ProjectRevenue, EarnedValueMetric, ChangeOrder, ProjectMilestone
  - S2C domain (6 types): SourcingProject, RfxEvent, SupplierBid, BidEvaluation, ProcurementContract, SupplierQualification
  - H2R domain (4 types): PayrollRun, TimeEntry, ExpenseReport, BenefitEnrollment
  - MFG domain (6 types): ProductionOrder, QualityInspection, CycleCount, BomComponent, InventoryMovement, Material
  - GOV domain (5 types): CosoComponent, CosoPrinciple, SoxAssertion, AuditEngagement, ProfessionalJudgment

- **Denormalized name fields on transaction models** (DS-011) (`datasynth-core`)
  - `vendor_name: Option<String>` on PurchaseOrder and VendorInvoice
  - `customer_name: Option<String>` on SalesOrder and CustomerInvoice
  - `employee_name: Option<String>` on TimeEntry and ExpenseReport
  - `material_description: Option<String>` on CycleCountItem
  - Builder methods (`with_vendor_name()`, `with_customer_name()`, etc.) for ergonomic construction

- **Boolean flags for graph queries** (DS-012) (`datasynth-core`)
  - `treatyApplied` on WithholdingTaxRecord (derived from `treaty_rate.is_some()`)
  - `isApproved` on PayrollRun (derived from status)
  - `isPassed` on QualityInspection (derived from result)
  - `isPhantom` on BomComponent
  - `isActive` on BenefitEnrollment, Material, TaxCode
  - `billable` on TimeEntry (70% probability in generator)

- **New generators** (DS-007, DS-008) (`datasynth-generators`)
  - `BomGenerator`: multi-level BOM component generation with phantom assembly support
  - `InventoryMovementGenerator`: goods movement generation tied to production orders and materials
  - `BenefitEnrollmentGenerator`: employee benefit enrollment with plan type distribution

- **Generator denormalization support** (DS-011) (`datasynth-generators`)
  - P2P generator populates `vendor_name` on PurchaseOrder and VendorInvoice
  - O2C generator populates `customer_name` on SalesOrder and CustomerInvoice
  - HR generators support `with_employee_names()` for name pool injection
  - Cycle count generator supports `with_material_descriptions()` for description pool injection

- **Graph export bridge** (DS-001) (`datasynth-graph`)
  - `From<GraphPropertyValue> for NodeProperty` conversion
  - `GraphNode::from_entity()` for creating graph nodes from any `ToNodeProperties` implementor

- **Comprehensive test suite** (`datasynth-core`, `datasynth-graph`)
  - Entity registry uniqueness tests (numeric codes, node type names)
  - Edge constraint validation tests (source/target validity, per-family coverage)
  - Category helper exhaustive tests
  - GraphPropertyValue round-trip tests
  - Graph export bridge integration tests

## [0.9.3] - 2026-02-27

### Added

- **VAT line splitting in document flow journal entries** ([#64](https://github.com/DataSynth/SyntheticData/issues/64)) (`datasynth-generators`)
  - Customer Invoice JE now posts: DR AR (gross), CR Revenue (net), CR VAT Payable (tax) when tax > 0
  - Vendor Invoice JE now posts: DR GR/IR Clearing (net), DR Input VAT (tax), CR AP (payable) when tax > 0
  - New `vat_output_account` and `vat_input_account` fields on `DocumentFlowJeConfig`
  - Framework-aware VAT accounts for French PCG, German SKR04, and default chart

- **Multipayment behavior with remainder payments** ([#65](https://github.com/DataSynth/SyntheticData/issues/65)) (`datasynth-generators`, `datasynth-runtime`)
  - O2C chains: partial payments now generate `RemainderPayment` events with follow-up `Payment` documents
  - P2P chains: new `remainder_payments` field with partial payment logic (50-75% initial, remainder after configurable days)
  - `avg_days_until_remainder` config field for both O2C and P2P payment behaviors
  - Orchestrator extracts all remainder receipts/payments into payments output and cash flows
  - JE generator produces proper GL entries for remainder payments (DR Cash/CR AR for O2C, DR AP/CR Cash for P2P)
  - Document chain manager tracks remainder references and statistics

- **Account-class-based fingerprinting** ([#66](https://github.com/DataSynth/SyntheticData/issues/66)) (`datasynth-fingerprint`)
  - New `AccountClassStats` model with per-class numeric statistics, row counts, and Benford analysis
  - Semantic column detection: heuristic GL account column recognition (supports `gl_account`, `konto`, `compte`, etc.)
  - Automatic debit/credit amount column detection
  - Account classification by first digit (0XXX-9XXX) with human-readable labels
  - Per-class `NumericStats` extraction reusing existing DP-protected computation
  - `fit_account_class_distributions()` for per-class distribution parameter fitting in synthesis

- **Config-driven P2P/O2C rates** (`datasynth-config`, `datasynth-runtime`)
  - `over_delivery_rate` and `early_payment_discount_rate` on `P2PFlowConfig`
  - `late_payment_rate` on `O2CFlowConfig`
  - Orchestrator reads from config with backward-compatible defaults (0.02, 0.30, 0.15)

- **DEFAULT_SEED constant** (`datasynth-runtime`): Extracted repeated `.unwrap_or(42)` into named constant

### Fixed

- **K-anonymity division by zero** (`datasynth-fingerprint`): `filter_frequencies()` now returns empty on `total=0` instead of producing NaN
- **Federated aggregation division by zero** (`datasynth-fingerprint`): `aggregate_weighted()` returns error on `total_record_count=0` instead of Infinity/NaN
- **Graph ghost edges** (`datasynth-graph`): PyTorch Geometric and DGL exporters now skip edges with missing node IDs instead of silently remapping to node 0 via `.unwrap_or(&0)`
- **GoBD document ID panic** (`datasynth-output`): Safe truncation `[..len.min(8)]` instead of `[..8]` substring that panics on short IDs
- **Prometheus metrics crash** (`datasynth-server`): Encoder and UTF-8 conversion now return error strings instead of panicking via `.unwrap()`
- **Rate limit header unwraps** (`datasynth-server`): Replaced `.parse().unwrap()` with infallible `HeaderValue::from()`
- **Non-deterministic household UUIDs** (`datasynth-banking`): Replaced `Uuid::new_v4()` with deterministic RNG-seeded UUID for reproducibility
- **Streaming orchestrator silent date fallbacks** (`datasynth-runtime`): 3 instances now log `tracing::warn!` before defaulting to 2024-01-01
- **Streaming orchestrator company fallback** (`datasynth-runtime`): 2 instances now log warning before defaulting to "1000"
- **Gate engine stub metrics** (`datasynth-eval`): CorrelationPreservation and Custom metrics now log `error!` level instead of `warn!` to surface unimplemented gates
- **Fingerprint extraction error drops** (`datasynth-fingerprint`): Optional component extraction failures now log warnings instead of silently returning None
- **Anomaly rounding magnitude** (`datasynth-generators`): Fixed string-length-based calculation that broke for negatives and small decimals; now uses proper digit counting on absolute value
- **Drift detection underflow** (`datasynth-eval`): Added guard for `values.len() < window_size` preventing subtraction overflow panic
- **Rayon thread pool error silenced** (`datasynth-cli`): Replaced `.ok()` with `eprintln!` warning on initialization failure
- **Label export serialization** (`datasynth-runtime`): 5 instances of `.unwrap_or_default()` replaced with `serialize_or_warn()` helper that logs failures
- **Distribution fitter NaN** (`datasynth-fingerprint`): Added `mean > 0.0` guard before calling `.ln()` in log-normal fitting
- **PCAOB series parse** (`datasynth-standards`): `parse().unwrap_or(0)` now logs warning on failure
- **JE generator COA fallback** (`datasynth-generators`): Added warning before falling back to first account
- **Mixture sampler NaN** (`datasynth-core`): Binary search uses `Ordering::Less` for NaN instead of `Ordering::Equal`
- **LLM config parse fallback** (`datasynth-core`): Now logs warning before falling back to keyword parsing
- **Test utility RwLock** (`datasynth-test-utils`): Replaced bare `.unwrap()` with descriptive `.expect()` messages

### Removed

- Dead `AggregatedEdge`, `ApprovalAggregation`, `AggregatedBankingEdge` structs (`datasynth-graph`)
- Dead `account_groups` field from `BalanceCoherenceValidator` (`datasynth-core`)
- Dead `line_config` field from `LineItemSampler` (`datasynth-core`)
- Unused `_credit_amount` variable in AR generator (`datasynth-generators`)

## [0.9.2] - 2026-02-27

### Added

- **Framework-aware account classification** (`datasynth-core`, `datasynth-generators`, `datasynth-graph`)
  - `FrameworkAccounts::classify_account_type()` and `classify_trial_balance_category()` methods for centralized, framework-aware account type inference
  - `ifrs()` constructor on `FrameworkAccounts` with explicit dispatch for all framework string variants
  - `AccountBalanceType::from_account_code_with_framework()` and `TrialBalanceCategory::from_account_code_with_framework()` for framework-aware balance classification
  - `balance_tracker`, `trial_balance_generator`, `currency_translator`, and `ic_generator` now accept framework parameter and use centralized classification instead of first-digit heuristics
  - `TransactionGraphBuilder` uses `FrameworkAccounts` classifier; account code ML feature normalized to [0,1] range using up to 4 digits

- **Shared NPY writer module** (`datasynth-graph`)
  - Extracted duplicated `write_npy_header`, `write_npy_1d_f32`, `write_npy_1d_i64`, `write_npy_2d_i64`, `write_npy_1d_bool`, `export_masks` from PyTorch Geometric and DGL exporters into `npy_writer.rs`

- **Plugin trait defaults** (`datasynth-core`)
  - `SinkPlugin` and `TransformPlugin` now have default `version()` ("0.1.0") and `description()` methods

- **Config validation** (`datasynth-config`)
  - `start_date` format validation (YYYY-MM-DD) in `validate_global_settings`
  - Company `name` non-empty and `country` 2-letter ISO code validation

### Changed

- **IC generator** uses `FrameworkAccounts` fields for revenue, expense, receivable, and payable account codes instead of hardcoded US GAAP accounts (`datasynth-generators`)
- **Currency translator** `is_monetary()` now framework-aware — classifies by `AccountCategory` before sub-classifying assets, with warning on unknown categories (`datasynth-generators`)
- **Streaming orchestrator** uses `chrono::Months` for proper end-date calculation instead of 30-day month approximation; reads departments from config with fallback warning (`datasynth-runtime`)
- **Server stubs wired** (`datasynth-server`):
  - `start_stream` reads `StreamRequest` fields (events_per_second, batch_size, patterns)
  - `reload_config` uses `config_loader` with proper error handling
  - Proto field mappings populated from domain models (vendor_id, customer_id, material_id, text, generate_master_data, generate_document_flows)
  - `ConfigSource::Url` implemented with `reqwest`
- **Streaming orchestrator phases** now log explicit `warn!` for skipped phases (AnomalyInjection, DataQuality, OCPM) instead of silent no-ops (`datasynth-runtime`)
- **CLI warnings**: unrecognized industry/complexity in `init` command, safety limit capping in demo mode now log `eprintln!` warnings (`datasynth-cli`)
- **Anderson-Darling p-value** uses conservative step-wise thresholds for unknown distributions instead of generic 0.05 (`datasynth-eval`)
- **AML detectability** for unknown typologies uses suspicious transaction ratio instead of trivially returning true (`datasynth-eval`)
- **Subledger reconciliation** returns unreconciled with warning for unknown account types instead of silently marking as reconciled (`datasynth-eval`)
- **Quality gate evaluation** logs clear warning that integration is pending (`datasynth-cli`)
- **Fingerprint `--sign`** logs `warn!` instead of silent `info!` (`datasynth-cli`)

### Fixed

- **Employee generator `last_mut()` ordering bug** (`datasynth-generators`): CFO/COO manager_id was being set on the wrong employee because `add_employee()` was called after `last_mut()` — now called before
- **Banking RNG determinism** (`datasynth-banking`): `generate_phone_from_pack` replaced `rand::rng()` / `rand::random()` with `self.rng` for deterministic output
- **Timing attack on gRPC auth** (`datasynth-server`): Token validation now uses `subtle::ConstantTimeEq` instead of `==`
- **CLI verify count mismatch** (`datasynth-cli`): Count mismatches were incrementing `passed` instead of `failed` and not setting `all_pass = false`
- **DGL heterogeneous graph node types** (`datasynth-graph`): Hardcoded `node_type_names[0]` replaced with per-node type lookup via `np.bincount(node_types).argmax()`
- **GoBD tax amount** (`datasynth-output`): `Steuerbetrag` now computed from line `tax_amount` field when `tax_code` is present; `Gegenkontonummer` finds correct contra account for multi-line entries
- **Banking silent date fallbacks** (`datasynth-banking`): 6 occurrences of `unwrap_or(2024-01-01)` replaced with shared `parse_start_date()` helper that logs warnings
- **Evidence generator hardcoded date** (`datasynth-generators`): Uses config period end date instead of hardcoded `2025-12-31`
- **gRPC unknown enum handling** (`datasynth-server`): Returns `Status::invalid_argument` instead of silently defaulting
- **Neo4j and DGL graph exports** (`datasynth-runtime`): Wired in orchestrator — previously logged warning and skipped
- **Production unwrap/expect calls** (`datasynth-core`, `datasynth-generators`, `datasynth-runtime`, `datasynth-fingerprint`): Replaced bare `.unwrap()` / `.expect()` with descriptive messages, `unwrap_or_else` with warnings, or `let Some(x) = ... else { warn; continue }` patterns across name pools, streaming channels, document chain manager, JE generator, run manifest, and certificates
- **Debug-format enum matching** (`datasynth-graph`): `banking_graph.rs` now matches on proper enum variants (`TurnoverBand::Low`, `CashIntensity::High`, etc.) instead of `format!("{:?}", ...)` string comparison
- **Header value unwraps** (`datasynth-server`): `security_headers.rs` uses `HeaderValue::from_static()`; `request_id.rs` and `rate_limit.rs` use `HeaderValue::try_from()` with error handling
- **Dead `BehavioralDrift` trait** removed (`datasynth-core`); dead `GenericParquetRecord` struct removed (`datasynth-output`); stale comment cleaned from `standards/mod.rs`
- **Causal validation `_correct_signs`** now used in success message reporting (`datasynth-core`)
- **Beta distribution** in causal SCM uses proper `rand_distr::Beta` instead of uniform fallback (`datasynth-core`)
- **Counterfactual `AddLineItem` and `RemoveLineItem`** implemented (`datasynth-generators`)
- **Approval graph `include_hierarchy`** logs warning when enabled (`datasynth-graph`)
- **Orchestrator hardcoded rates** annotated with `// TODO: wire to config schema` and `tracing::debug!` (`datasynth-runtime`)
- **NL config unknown features** now log `warn!` instead of silently ignoring (`datasynth-core`)
- **Template provider unknown industry** now logs `debug!` instead of silently falling back (`datasynth-core`)
- **WebSocket streaming** sends all entries per generation cycle instead of `take(1)` (`datasynth-server`)
- **gRPC streaming** creates orchestrator once outside loop instead of per iteration (`datasynth-server`)

## [0.9.1] - 2026-02-26

### Added

- **German GAAP (HGB) Accounting Framework** (`datasynth-standards`, `datasynth-core`, `datasynth-generators`)
  - `AccountingFramework::GermanGaap` variant with HGB §238-263 rules: mandatory impairment reversal (§253(5)), no bright-line lease tests (BMF-Leasingerlasse 40-90%), no PPE revaluation, optional development capitalization (§248(2)), LIFO prohibited, pending loss provisions required
  - `FrameworkSettings::german_gaap()` constructor with validation
  - HGB-specific query methods: `requires_pending_loss_provisions()`, `allows_low_value_asset_expensing()`, `operating_leases_off_balance()`

- **Generalized Multi-GAAP Framework** (`datasynth-core`)
  - `FrameworkAccounts` struct mapping ~45 semantic account purposes to framework-specific GL codes with `for_framework(AccountingFramework)` constructor
  - `AuditExportConfig` controlling FEC (French) and GoBD (German) audit export flags
  - Framework-aware `classify(account) -> AccountCategory` via pluggable classifiers for US GAAP (1-9 digit), PCG (1-7 digit), and SKR04 (0-9 digit) account numbering
  - `From<&FrameworkAccounts>` impls for `DocumentFlowJeConfig`, `DepreciationRunConfig`, `YearEndCloseConfig`

- **SKR04 Chart of Accounts** (`datasynth-core`)
  - `skr` module with German Standardkontenrahmen 04 constants: control, cash, revenue, expense, equity, tax, personnel, and provision accounts
  - `skr_loader` module loading embedded `skr04_2024.json` (~400 accounts across classes 0-9) with complexity filtering and industry mapping
  - `CoAFramework::GermanSkr04` variant in `coa_generator` with `generate_skr()` dispatch

- **GoBD Audit Export** (`datasynth-output`)
  - `gobd::write_gobd_journal_csv()` — semicolon-separated, UTF-8, 13-column GoBD journal export (Belegdatum, Buchungsdatum, Belegnummer, Buchungstext, Kontonummer, Gegenkontonummer, Sollbetrag, Habenbetrag, Steuerschlüssel, Steuerbetrag, Währung, Kostenstelle, Belegnummernkreis)
  - `gobd::write_gobd_accounts_csv()` — account master data export
  - `gobd::write_gobd_index_xml()` — GoBD-compliant XML index with table schema descriptions

- **German Depreciation Methods** (`datasynth-core`, `datasynth-generators`)
  - `DepreciationMethod::Degressiv` — declining balance: min(3x straight-line rate, 30%) on NBV with automatic switch to straight-line (per EStG §7(2) / Wachstumschancengesetz Jul 2025-Dec 2027)
  - GWG (geringwertige Wirtschaftsgüter) support: assets ≤ 800 EUR → immediate full expense (EStG §6(2)), `is_gwg: Option<bool>` field on `FixedAsset`
  - AfA-Tabellen default useful lives for German GAAP (buildings 33yr, vehicles 6yr, IT equipment 3yr, etc.)

- **Auxiliary GL Accounts on Master Data** (`datasynth-core`, `datasynth-generators`)
  - `auxiliary_gl_account: Option<String>` field on `Vendor` and `Customer` models
  - French PCG format: `401XXXX` (vendors), `411XXXX` (customers)
  - German SKR04 format: `3300XXXX` (vendors), `1200XXXX` (customers)
  - Vendor/customer name deduplication via `HashSet` tracking with suffix fallback

- **Expanded PCG Accounts** (`datasynth-core`)
  - New modules: `fixed_asset_accounts`, `tax_accounts`, `suspense_accounts`, `additional_revenue`, `additional_expense`, `liability_accounts`, `equity_accounts`

- **DE.json Country Pack**: Updated framework to `german_gaap`, added `GoBD` to local regulations (`datasynth-core`)
- **UI**: German GAAP (HGB) framework option in accounting standards selector (`datasynth-ui`)
- **Python**: `german_gaap` added to framework enum options comment (`datasynth-py`)

### Fixed

- FEC auxiliary account fields now use framework-specific GL accounts (e.g., PCG `4010001`) instead of raw partner IDs — lookup map built from vendor/customer `auxiliary_gl_account` master data (`datasynth-generators`, `datasynth-runtime`)
- FEC export now populates columns 7-8 (auxiliary account number/label) and 14-15 (lettrage/lettrage date) on AP/AR lines for French GAAP (`datasynth-output`, `datasynth-core`)
- Document flow JE generator populates FEC auxiliary fields with business partner ID on AP/AR lines and applies lettrage codes on completed P2P/O2C chains (`datasynth-generators`)
- Orchestrator auto-detects French GAAP framework and uses PCG account mapping with FEC field population (`datasynth-runtime`)
- PCG constant `FIXED_ASSETS` corrected from 215000 (installations techniques, a specific sub-class) to 210000 (immobilisations corporelles, generic class 2) (`datasynth-core`)
- Journal entries CSV export now includes auxiliary_account_number, auxiliary_account_label, lettrage, lettrage_date columns (`datasynth-cli`)
- P2P/O2C document flow chain caps now scale with `period_months` — previously capped at `partners × 2` regardless of period length, now `partners × 2 × period_months` (`datasynth-runtime`)

## [0.9.0] - 2026-02-25

### Added

- Performance Phase 1: cached temporal CDF, fast Decimal, SmallVec line items, binary search company selector, `#[inline]` hot paths, 256KB BufWriter buffers (~2x throughput)
- Performance Phase 2: `ParallelGenerator` trait, deterministic seed splitting, multi-core master data + JE generation, per-partition UUID factories
- Performance Phase 3: itoa/ryu formatting, `fast_csv` module, zstd `CompressedWriter`, `CsvSink`/`JsonLinesSink` write optimization
- 14 parallel coherence verification tests
- Performance analysis documentation (`docs/performance-improvements.md`)

### Changed

- BufWriter default 8KB to 256KB across 86+ output sinks
- Single-threaded throughput ~100K to ~200K entries/sec

### Fixed

- Banking spoofing test resilience to RNG sequence changes
- Zero-amount line in `sample_summing_to` with sum-preserving transfer

### Dependencies

- arrow/parquet 54 to 58, zip 2 to 8
- rand 0.8 to 0.9, rand_chacha 0.3 to 0.9, rand_distr 0.4 to 0.5
- jsonwebtoken 9 to 10, redis 0.27 to 1.0, axum-server 0.7 to 0.8, indicatif 0.17 to 0.18

## [0.8.1] - 2026-02-20

### Added

- French GAAP (PCG) accounting framework support with Plan Comptable Général 2024 chart of accounts (`datasynth-core`, `datasynth-generators`)
- FEC (Fichier des Écritures Comptables) export — Article A47 A-1 compliant 18-column format (`datasynth-output`)
- French GAAP lease classification aligned with IFRS 16 / ANC 2019 (`datasynth-standards`)

### Fixed

- PCG account type classification now handles multi-digit account numbers correctly — previously accounts like 1011 (Capital souscrit) were misclassified as liabilities (`datasynth-core`)
- Account 421 (Personnel) removed from Class 1 fallback generator — was incorrectly placed with Class 1 equity accounts and mislabeled as "Fournisseurs" (`datasynth-generators`)
- FEC `format_decimal` now uses native `rust_decimal` formatting instead of lossy f64 conversion (`datasynth-output`)
- French GAAP lease classification delegates to IFRS 16 logic, fixing inconsistency with `uses_brightline_lease_tests()` returning false (`datasynth-standards`)
- PCG constant `PETTY_CASH` corrected from 516000 (internal transfers) to 531000 (caisse) (`datasynth-core`)
- PCG constant `ACCRUED_EXPENSES` moved out of `equity_liability_accounts` — was a Class 4 account in a Class 1 module (`datasynth-core`)
- FEC field escaping: removed dead code branch that checked for semicolons after already replacing them (`datasynth-output`)

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

- **Orchestrator Pipeline Wiring — Round 3** (`datasynth-runtime`):
  - **Opening balance generation** (phase 3b): `OpeningBalanceGenerator` wired per company with industry-typed specs, opening balances exported to `balance/opening_balances.json`
  - **GL-to-subledger reconciliation** (phase 9b): `ReconciliationEngine` reconciles AR, AP, FA, and Inventory control accounts against subledger totals, exported to `balance/subledger_reconciliation.json`
  - **Tax line generation**: `TaxLineGenerator` produces tax lines from vendor invoices (input VAT) and customer invoices (output VAT) using actual document flow data
  - **Project cost allocation**: `ProjectCostGenerator` links time entries, expense reports, POs, and vendor invoices as `SourceDocument` records for cost allocation
  - **ESG vendor spend**: ESG spend calculations now use actual payment data (filtered by `payment.is_vendor`) instead of stub values
  - **Treasury cash positions**: `CashPositionGenerator` aggregates P2P payment outflows and O2C customer receipt inflows into daily cash positions
  - **Graph export summary**: `graph_export_summary.json` exported when graph export is enabled

- **Determinism Fix** (`datasynth-generators`): `Uuid::new_v4()` in `SchemeAction::new` replaced with FNV-1a hash-based deterministic UUID construction for reproducible anomaly scheme generation

- **Generator Tracing — Round 3** (`datasynth-generators`): `tracing::debug!` instrumentation added to ~25 generator entry points across core (JE, CoA, control, injector), master data (vendor, customer, material, employee, asset), subledger (AR, AP, FA, inventory), period close (close engine, accruals, depreciation, financial statements), HR (payroll, time entry, expense report), manufacturing (production order, quality inspection), and intercompany modules

- **Dead Code Cleanup — Round 3** (`datasynth-generators`): Removed `#![allow(dead_code)]` from 8 ESG and project accounting generator files; deleted unused `base_vendor`/`base_customer` from JE generator, `MATERIAL_GROUPS` from material generator, `calculate_monthly_depreciation` from FA generator, and 3 unused helpers from customer generator; added targeted `#[allow(dead_code)]` with explanatory comments for legitimately pre-wired items

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
- **FA acquisition JEs collected** (`datasynth-runtime`): Fixed variable shadowing (`_je` → `je`) that silently discarded fixed asset acquisition journal entries
- **Statistics recalculated** (`datasynth-runtime`): `total_entries` and `total_line_items` now recomputed after all JE-generating phases (FA, IC, payroll, manufacturing) instead of freezing before them
- **Anomaly injection rates from config** (`datasynth-runtime`): Wired `config.anomaly_injection.rates` into anomaly injector instead of hardcoding `0.02`; falls back to `config.fraud.fraud_rate` then default
- **CLI PhaseConfig wiring** (`datasynth-cli`): 11 config-enabled sections (manufacturing, sourcing, tax, ESG, intercompany, accounting_standards, financial_statements, sales_kpi_budgets, bank_reconciliation, OCPM, audit/graph) now wired from YAML into `PhaseConfig`
- **Manufacturing preset fix** (`datasynth-config`): Manufacturing preset now enables manufacturing generation via `get_manufacturing_config()` helper
- **Treasury hedge relationships** (`datasynth-runtime`): `HedgingGenerator` wired with FX exposure data from actual FX rate service, replacing stub generation
- **Non-deterministic `Local::now()` fallbacks** (`datasynth-generators`): Replaced remaining `Local::now()` calls in balance tracker and opening balance generator with config-derived dates
- **`TransactionSource` Display impl** (`datasynth-core`): Added `Display` trait for `TransactionSource` enum, ensuring consistent CSV serialization
- **DataQualityConfig wiring** (`datasynth-runtime`): `data_quality` YAML section now parsed and passed to data quality injector instead of using defaults
- **Hardcoded USD replaced** (`datasynth-generators`): Inventory generator, opening balance generator, and balance tracker now accept company currency parameter instead of hardcoding `"USD"`
- **HR pool-based ID selection** (`datasynth-generators`): Payroll, time entry, and expense report generators draw employee IDs and cost center codes from master data pools instead of fabricating `EMP-{n}` / `CC-{n}` strings
- **`exchange_rate` serde annotation** (`datasynth-core`): Added `#[serde(serialize_with = "str")]` to `FxRate.exchange_rate` for consistent decimal string serialization
- **Manifest completeness** (`datasynth-cli`): ~50 missing output files registered in run manifest across treasury, project accounting, tax, ESG, audit, standards, quality labels, and graph export modules
- **`seeded_rng()` standardized** (`datasynth-generators`): Replaced `ChaCha8Rng::seed_from_u64()` with canonical `seeded_rng(seed, 0)` utility across ~50 production generator files for consistent RNG construction
- **Clippy fixes** (`datasynth-generators`, `datasynth-eval`, `datasynth-test-utils`): Replaced `#[allow(clippy::derivable_impls)]` with `#[derive(Default)]` on `DocumentFlowLinker`; fixed `field_reassign_with_default` in healthcare settings, red flag statistics, and test server config; removed duplicate `rust_decimal_macros` dependency

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
- **OCPM S2C vendor fallback** (`datasynth-runtime`): Replaced hardcoded `"V000"` vendor ID with actual vendor from master data when no contract found for sourcing project
- **OCPM cycle count matching** (`datasynth-runtime`): Manufacturing OCPM events now match cycle counts by `material_id` instead of always linking to the first cycle count
- **OCPM determinism** (`datasynth-runtime`): Replaced 6 `Utc::now()` calls in OCPM event generation with config-derived deterministic base date for reproducible timestamps across S2C, H2R, MFG, Banking, Audit, and Bank Recon process families
- **Trial balance determinism** (`datasynth-generators`): Replaced 4 `Utc::now()` calls in `TrialBalanceGenerator` with period-derived dates — `created_at` uses `as_of_date` end-of-day, `approved_at` uses next business day morning
- **IC hardcoded currency** (`datasynth-generators`): Added `default_currency` to `ICGeneratorConfig`, replacing hardcoded `"USD"` in IC matched pairs and IC loans; orchestrator wires first company's currency
- **Expense report hardcoded currency** (`datasynth-generators`): Added `generate_with_currency()` method to `ExpenseReportGenerator`, replacing hardcoded `"USD"` in reports and line items; orchestrator passes company currency
- **IC account code panic risk** (`datasynth-generators`): Replaced byte-index slicing (`&company[..len.min(2)]`) with safe `chars().take(2).collect()` in IC receivable/payable account code generation to prevent UTF-8 boundary panics
- **Banking customer cross-reference** (`datasynth-banking`, `datasynth-runtime`): Added `enterprise_customer_id: Option<String>` field to `BankingCustomer`, populated during cross-referencing to link banking customers to core enterprise customer IDs
- **S2C→P2P contract linkage** (`datasynth-runtime`): Purchase orders now linked to S2C procurement contracts by vendor ID match after sourcing data generation, populating `PurchaseOrder.contract_id`
- **Dead code cleanup — Round 4** (`datasynth-generators`, `datasynth-core`, `datasynth-eval`, `datasynth-fingerprint`): Removed dead `spend_emission_factor_from_pack()` function, dead `line_patterns` field and `LineTextPattern` type from description generator, orphaned doc comment; wired `LastDotFirst` and `FirstOnly` email patterns into pattern pool; added `GaussianMechanism::epsilon()` getter matching `LaplaceMechanism` API
- **Banking manifest filename** (`datasynth-cli`): Fixed mismatch `bank_transactions` → `banking_transactions` in output manifest
- **GL account collisions — year_end.rs** (`datasynth-generators`): Fixed 4 GL code collisions in `YearEndCloseConfig::default()` — income_summary `"3500"` → `"3600"` (was CTA), current_tax_payable `"2300"` → `"2100"` (was UNEARNED_REVENUE), deferred_tax_liability `"2350"` → `"2500"`, tax_expense `"7100"` → `"8000"` (was INTEREST_EXPENSE)
- **GL account — depreciation.rs** (`datasynth-generators`): Fixed depreciation expense posted to `"6100"` (SALARIES_WAGES) instead of `"6000"` (DEPRECIATION constant)
- **GL account — AR credit memo** (`datasynth-generators`): Fixed credit memo tax JE using `"2300"` (UNEARNED_REVENUE) instead of `SALES_TAX_PAYABLE` (`"2100"`); replaced all hardcoded GL strings in AR/AP generators with `accounts.rs` constants
- **New GL constants** (`datasynth-core`): Added `INCOME_SUMMARY`, `DIVIDENDS_PAID`, `TAX_RECEIVABLE`, `PURCHASE_DISCOUNT_INCOME` to `accounts.rs`
- **SalesQuoteGenerator pools** (`datasynth-generators`, `datasynth-runtime`): Wired company currency, employee pool (for sales_rep_id), and customer pool (for customer_id) replacing hardcoded `"USD"` and fabricated `SR-{n}` / `CUST-{n}` IDs
- **BankReconciliationGenerator pool** (`datasynth-generators`, `datasynth-runtime`): Wired employee pool for preparer/reviewer IDs, replacing fabricated `USR-{n}` strings
- **CycleCountGenerator pool** (`datasynth-generators`, `datasynth-runtime`): Wired employee pool for counter/supervisor IDs, replacing fabricated `WH-{n}` / `SUP-{n}` strings
- **Dead code cleanup — Rounds 5-6** (`datasynth-eval`, `datasynth-generators`): Removed `BaselineComparer` (~220 lines), dead `uuid_factory` fields from 10 structs across 9 ESG/project-accounting generators, `calculate_period_variances` function, `apply_line` method, `MultiplyByGapFactor` variant, unused vendor `_spend_category` binding, and unused imports

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
