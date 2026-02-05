# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
