# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-31

### Added

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

- `GeneratorConfig` now includes `distributions` field for advanced distribution settings
- All presets, fixtures, and config initializers updated with distributions support
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
