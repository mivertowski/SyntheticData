//! Configuration schema for synthetic data generation.

use datasynth_core::distributions::{
    AmountDistributionConfig, DebitCreditDistributionConfig, EvenOddDistributionConfig,
    LineItemDistributionConfig, SeasonalityConfig,
};
use datasynth_core::models::{CoAComplexity, IndustrySector};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root configuration for the synthetic data generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Company configuration
    pub companies: Vec<CompanyConfig>,
    /// Chart of Accounts configuration
    pub chart_of_accounts: ChartOfAccountsConfig,
    /// Transaction generation settings
    #[serde(default)]
    pub transactions: TransactionConfig,
    /// Output configuration
    pub output: OutputConfig,
    /// Fraud simulation settings
    #[serde(default)]
    pub fraud: FraudConfig,
    /// Data quality variation settings
    #[serde(default)]
    pub data_quality: DataQualitySchemaConfig,
    /// Internal Controls System settings
    #[serde(default)]
    pub internal_controls: InternalControlsConfig,
    /// Business process mix
    #[serde(default)]
    pub business_processes: BusinessProcessConfig,
    /// User persona distribution
    #[serde(default)]
    pub user_personas: UserPersonaConfig,
    /// Template configuration for realistic data
    #[serde(default)]
    pub templates: TemplateConfig,
    /// Approval workflow configuration
    #[serde(default)]
    pub approval: ApprovalConfig,
    /// Department structure configuration
    #[serde(default)]
    pub departments: DepartmentConfig,
    /// Master data generation settings
    #[serde(default)]
    pub master_data: MasterDataConfig,
    /// Document flow generation settings
    #[serde(default)]
    pub document_flows: DocumentFlowConfig,
    /// Intercompany transaction settings
    #[serde(default)]
    pub intercompany: IntercompanyConfig,
    /// Balance and trial balance settings
    #[serde(default)]
    pub balance: BalanceConfig,
    /// OCPM (Object-Centric Process Mining) settings
    #[serde(default)]
    pub ocpm: OcpmConfig,
    /// Audit engagement and workpaper generation settings
    #[serde(default)]
    pub audit: AuditGenerationConfig,
    /// Banking KYC/AML transaction generation settings
    #[serde(default)]
    pub banking: datasynth_banking::BankingConfig,
    /// Scenario configuration for metadata and tagging (Phase 1.3)
    #[serde(default)]
    pub scenario: ScenarioConfig,
    /// Temporal drift configuration for simulating distribution changes over time (Phase 2.2)
    #[serde(default)]
    pub temporal: TemporalDriftConfig,
    /// Graph export configuration for accounting network export
    #[serde(default)]
    pub graph_export: GraphExportConfig,
    /// Streaming output API configuration
    #[serde(default)]
    pub streaming: StreamingSchemaConfig,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitSchemaConfig,
    /// Temporal attribute generation configuration
    #[serde(default)]
    pub temporal_attributes: TemporalAttributeSchemaConfig,
    /// Relationship generation configuration
    #[serde(default)]
    pub relationships: RelationshipSchemaConfig,
    /// Accounting standards framework configuration (IFRS, US GAAP)
    #[serde(default)]
    pub accounting_standards: AccountingStandardsConfig,
    /// Audit standards framework configuration (ISA, PCAOB)
    #[serde(default)]
    pub audit_standards: AuditStandardsConfig,
    /// Advanced distribution configuration (mixture models, correlations, regime changes)
    #[serde(default)]
    pub distributions: AdvancedDistributionConfig,
    /// Temporal patterns configuration (business days, period-end dynamics, processing lags)
    #[serde(default)]
    pub temporal_patterns: TemporalPatternsConfig,
    /// Vendor network configuration (multi-tier supply chain modeling)
    #[serde(default)]
    pub vendor_network: VendorNetworkSchemaConfig,
    /// Customer segmentation configuration (value segments, lifecycle stages)
    #[serde(default)]
    pub customer_segmentation: CustomerSegmentationSchemaConfig,
    /// Relationship strength calculation configuration
    #[serde(default)]
    pub relationship_strength: RelationshipStrengthSchemaConfig,
    /// Cross-process link configuration (P2P ↔ O2C via inventory)
    #[serde(default)]
    pub cross_process_links: CrossProcessLinksSchemaConfig,
    /// Organizational events configuration (acquisitions, divestitures, etc.)
    #[serde(default)]
    pub organizational_events: OrganizationalEventsSchemaConfig,
    /// Behavioral drift configuration (vendor, customer, employee behavior)
    #[serde(default)]
    pub behavioral_drift: BehavioralDriftSchemaConfig,
    /// Market drift configuration (economic cycles, commodities, price shocks)
    #[serde(default)]
    pub market_drift: MarketDriftSchemaConfig,
    /// Drift labeling configuration for ground truth generation
    #[serde(default)]
    pub drift_labeling: DriftLabelingSchemaConfig,
    /// Enhanced anomaly injection configuration (multi-stage schemes, correlated injection, near-miss)
    #[serde(default)]
    pub anomaly_injection: EnhancedAnomalyConfig,
    /// Industry-specific transaction and anomaly generation configuration
    #[serde(default)]
    pub industry_specific: IndustrySpecificConfig,
    /// Fingerprint privacy configuration for extraction/synthesis
    #[serde(default)]
    pub fingerprint_privacy: FingerprintPrivacyConfig,
    /// Quality gate configuration for pass/fail thresholds
    #[serde(default)]
    pub quality_gates: QualityGatesSchemaConfig,
    /// Compliance configuration (EU AI Act, content marking)
    #[serde(default)]
    pub compliance: ComplianceSchemaConfig,
    /// Webhook notification configuration
    #[serde(default)]
    pub webhooks: WebhookSchemaConfig,
    /// LLM enrichment configuration (AI-augmented vendor names, descriptions, explanations)
    #[serde(default)]
    pub llm: LlmSchemaConfig,
    /// Diffusion model configuration (statistical diffusion-based data enhancement)
    #[serde(default)]
    pub diffusion: DiffusionSchemaConfig,
    /// Causal generation configuration (structural causal models, interventions)
    #[serde(default)]
    pub causal: CausalSchemaConfig,

    // ===== Enterprise Process Chain Extensions =====
    /// Source-to-Pay (S2C/S2P) configuration (sourcing, contracts, catalogs, scorecards)
    #[serde(default)]
    pub source_to_pay: SourceToPayConfig,
    /// Financial reporting configuration (financial statements, KPIs, budgets)
    #[serde(default)]
    pub financial_reporting: FinancialReportingConfig,
    /// HR process configuration (payroll, time & attendance, expenses)
    #[serde(default)]
    pub hr: HrConfig,
    /// Manufacturing configuration (production orders, WIP, routing)
    #[serde(default)]
    pub manufacturing: ManufacturingProcessConfig,
    /// Sales quote configuration (quote-to-order pipeline)
    #[serde(default)]
    pub sales_quotes: SalesQuoteConfig,
    /// Tax accounting configuration (VAT/GST, sales tax, withholding, provisions, payroll tax)
    #[serde(default)]
    pub tax: TaxConfig,
    /// Treasury and cash management configuration
    #[serde(default)]
    pub treasury: TreasuryConfig,
    /// Project accounting configuration
    #[serde(default)]
    pub project_accounting: ProjectAccountingConfig,
    /// ESG / Sustainability reporting configuration
    #[serde(default)]
    pub esg: EsgConfig,
}

/// LLM enrichment configuration.
///
/// Controls AI-augmented metadata enrichment using LLM providers.
/// When enabled, vendor names, transaction descriptions, and anomaly explanations
/// are enriched using the configured provider (mock by default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSchemaConfig {
    /// Whether LLM enrichment is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Provider type: "mock", "openai", "anthropic", "custom".
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    /// Model name/ID for the provider.
    #[serde(default = "default_llm_model_name")]
    pub model: String,
    /// Maximum number of vendor names to enrich per run.
    #[serde(default = "default_llm_batch_size")]
    pub max_vendor_enrichments: usize,
}

fn default_llm_provider() -> String {
    "mock".to_string()
}

fn default_llm_model_name() -> String {
    "gpt-4o-mini".to_string()
}

fn default_llm_batch_size() -> usize {
    50
}

impl Default for LlmSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_llm_provider(),
            model: default_llm_model_name(),
            max_vendor_enrichments: default_llm_batch_size(),
        }
    }
}

/// Diffusion model configuration.
///
/// Controls statistical diffusion-based data enhancement that generates samples
/// matching target distribution properties (means, standard deviations, correlations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionSchemaConfig {
    /// Whether diffusion enhancement is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Number of diffusion steps (higher = better quality, slower).
    #[serde(default = "default_diffusion_steps")]
    pub n_steps: usize,
    /// Noise schedule type: "linear", "cosine", "sigmoid".
    #[serde(default = "default_diffusion_schedule")]
    pub schedule: String,
    /// Number of sample rows to generate for demonstration.
    #[serde(default = "default_diffusion_sample_size")]
    pub sample_size: usize,
}

fn default_diffusion_steps() -> usize {
    100
}

fn default_diffusion_schedule() -> String {
    "linear".to_string()
}

fn default_diffusion_sample_size() -> usize {
    100
}

impl Default for DiffusionSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            n_steps: default_diffusion_steps(),
            schedule: default_diffusion_schedule(),
            sample_size: default_diffusion_sample_size(),
        }
    }
}

/// Causal generation configuration.
///
/// Controls structural causal model (SCM) based data generation that respects
/// causal relationships between variables, supports do-calculus interventions,
/// and enables counterfactual scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalSchemaConfig {
    /// Whether causal generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Built-in template to use: "fraud_detection", "revenue_cycle", or "custom".
    #[serde(default = "default_causal_template")]
    pub template: String,
    /// Number of causal samples to generate.
    #[serde(default = "default_causal_sample_size")]
    pub sample_size: usize,
    /// Whether to run causal validation on the output.
    #[serde(default = "default_true")]
    pub validate: bool,
}

fn default_causal_template() -> String {
    "fraud_detection".to_string()
}

fn default_causal_sample_size() -> usize {
    500
}

impl Default for CausalSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            template: default_causal_template(),
            sample_size: default_causal_sample_size(),
            validate: true,
        }
    }
}

/// Graph export configuration for accounting network and ML training exports.
///
/// This section enables exporting generated data as graphs for:
/// - Network reconstruction algorithms
/// - Graph neural network training
/// - Neo4j graph database import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportConfig {
    /// Enable graph export.
    #[serde(default)]
    pub enabled: bool,

    /// Graph types to generate.
    #[serde(default = "default_graph_types")]
    pub graph_types: Vec<GraphTypeConfig>,

    /// Export formats to generate.
    #[serde(default = "default_graph_formats")]
    pub formats: Vec<GraphExportFormat>,

    /// Train split ratio for ML datasets.
    #[serde(default = "default_train_ratio")]
    pub train_ratio: f64,

    /// Validation split ratio for ML datasets.
    #[serde(default = "default_val_ratio")]
    pub validation_ratio: f64,

    /// Random seed for train/val/test splits.
    #[serde(default)]
    pub split_seed: Option<u64>,

    /// Output subdirectory for graph exports (relative to output directory).
    #[serde(default = "default_graph_subdir")]
    pub output_subdirectory: String,

    /// Multi-layer hypergraph export settings for RustGraph integration.
    #[serde(default)]
    pub hypergraph: HypergraphExportSettings,
}

fn default_graph_types() -> Vec<GraphTypeConfig> {
    vec![GraphTypeConfig::default()]
}

fn default_graph_formats() -> Vec<GraphExportFormat> {
    vec![GraphExportFormat::PytorchGeometric]
}

fn default_train_ratio() -> f64 {
    0.7
}

fn default_val_ratio() -> f64 {
    0.15
}

fn default_graph_subdir() -> String {
    "graphs".to_string()
}

impl Default for GraphExportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            graph_types: default_graph_types(),
            formats: default_graph_formats(),
            train_ratio: 0.7,
            validation_ratio: 0.15,
            split_seed: None,
            output_subdirectory: "graphs".to_string(),
            hypergraph: HypergraphExportSettings::default(),
        }
    }
}

/// Settings for the multi-layer hypergraph export (RustGraph integration).
///
/// Produces a 3-layer hypergraph:
/// - Layer 1: Governance & Controls (COSO, SOX, internal controls, organizational)
/// - Layer 2: Process Events (P2P/O2C document flows, OCPM events)
/// - Layer 3: Accounting Network (GL accounts, journal entries as hyperedges)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphExportSettings {
    /// Enable hypergraph export.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum total nodes across all layers (default 50000).
    #[serde(default = "default_hypergraph_max_nodes")]
    pub max_nodes: usize,

    /// Aggregation strategy when node budget is exceeded.
    #[serde(default = "default_aggregation_strategy")]
    pub aggregation_strategy: String,

    /// Layer 1 (Governance & Controls) settings.
    #[serde(default)]
    pub governance_layer: GovernanceLayerSettings,

    /// Layer 2 (Process Events) settings.
    #[serde(default)]
    pub process_layer: ProcessLayerSettings,

    /// Layer 3 (Accounting Network) settings.
    #[serde(default)]
    pub accounting_layer: AccountingLayerSettings,

    /// Cross-layer edge generation settings.
    #[serde(default)]
    pub cross_layer: CrossLayerSettings,

    /// Output subdirectory for hypergraph files (relative to graph output directory).
    #[serde(default = "default_hypergraph_subdir")]
    pub output_subdirectory: String,

    /// Output format: "native" (default) for internal field names, "unified" for RustGraph format.
    #[serde(default = "default_hypergraph_format")]
    pub output_format: String,

    /// Optional URL for streaming unified JSONL to a RustGraph ingest endpoint.
    #[serde(default)]
    pub stream_target: Option<String>,

    /// Batch size for streaming (number of JSONL lines per HTTP POST). Default: 1000.
    #[serde(default = "default_stream_batch_size")]
    pub stream_batch_size: usize,
}

fn default_hypergraph_max_nodes() -> usize {
    50_000
}

fn default_aggregation_strategy() -> String {
    "pool_by_counterparty".to_string()
}

fn default_hypergraph_subdir() -> String {
    "hypergraph".to_string()
}

fn default_hypergraph_format() -> String {
    "native".to_string()
}

fn default_stream_batch_size() -> usize {
    1000
}

impl Default for HypergraphExportSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_nodes: 50_000,
            aggregation_strategy: "pool_by_counterparty".to_string(),
            governance_layer: GovernanceLayerSettings::default(),
            process_layer: ProcessLayerSettings::default(),
            accounting_layer: AccountingLayerSettings::default(),
            cross_layer: CrossLayerSettings::default(),
            output_subdirectory: "hypergraph".to_string(),
            output_format: "native".to_string(),
            stream_target: None,
            stream_batch_size: 1000,
        }
    }
}

/// Layer 1: Governance & Controls layer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLayerSettings {
    /// Include COSO framework nodes (5 components + 17 principles).
    #[serde(default = "default_true")]
    pub include_coso: bool,
    /// Include internal control nodes.
    #[serde(default = "default_true")]
    pub include_controls: bool,
    /// Include SOX assertion nodes.
    #[serde(default = "default_true")]
    pub include_sox: bool,
    /// Include vendor master data nodes.
    #[serde(default = "default_true")]
    pub include_vendors: bool,
    /// Include customer master data nodes.
    #[serde(default = "default_true")]
    pub include_customers: bool,
    /// Include employee/organizational nodes.
    #[serde(default = "default_true")]
    pub include_employees: bool,
}

impl Default for GovernanceLayerSettings {
    fn default() -> Self {
        Self {
            include_coso: true,
            include_controls: true,
            include_sox: true,
            include_vendors: true,
            include_customers: true,
            include_employees: true,
        }
    }
}

/// Layer 2: Process Events layer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLayerSettings {
    /// Include P2P (Procure-to-Pay) document flow nodes.
    #[serde(default = "default_true")]
    pub include_p2p: bool,
    /// Include O2C (Order-to-Cash) document flow nodes.
    #[serde(default = "default_true")]
    pub include_o2c: bool,
    /// Include S2C (Source-to-Contract) document flow nodes.
    #[serde(default = "default_true")]
    pub include_s2c: bool,
    /// Include H2R (Hire-to-Retire) document flow nodes.
    #[serde(default = "default_true")]
    pub include_h2r: bool,
    /// Include MFG (Manufacturing) document flow nodes.
    #[serde(default = "default_true")]
    pub include_mfg: bool,
    /// Include BANK (Banking) document flow nodes.
    #[serde(default = "default_true")]
    pub include_bank: bool,
    /// Include AUDIT document flow nodes.
    #[serde(default = "default_true")]
    pub include_audit: bool,
    /// Include R2R (Record-to-Report) document flow nodes (bank recon + period close).
    #[serde(default = "default_true")]
    pub include_r2r: bool,
    /// Export OCPM events as hyperedges.
    #[serde(default = "default_true")]
    pub events_as_hyperedges: bool,
    /// Threshold: if a counterparty has more documents than this, aggregate into pool nodes.
    #[serde(default = "default_docs_per_counterparty_threshold")]
    pub docs_per_counterparty_threshold: usize,
}

fn default_docs_per_counterparty_threshold() -> usize {
    20
}

impl Default for ProcessLayerSettings {
    fn default() -> Self {
        Self {
            include_p2p: true,
            include_o2c: true,
            include_s2c: true,
            include_h2r: true,
            include_mfg: true,
            include_bank: true,
            include_audit: true,
            include_r2r: true,
            events_as_hyperedges: true,
            docs_per_counterparty_threshold: 20,
        }
    }
}

/// Layer 3: Accounting Network layer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingLayerSettings {
    /// Include GL account nodes.
    #[serde(default = "default_true")]
    pub include_accounts: bool,
    /// Export journal entries as hyperedges (debit+credit accounts as participants).
    #[serde(default = "default_true")]
    pub je_as_hyperedges: bool,
}

impl Default for AccountingLayerSettings {
    fn default() -> Self {
        Self {
            include_accounts: true,
            je_as_hyperedges: true,
        }
    }
}

/// Cross-layer edge generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLayerSettings {
    /// Generate cross-layer edges (Control→Account, Vendor→PO, etc.).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for CrossLayerSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Configuration for a specific graph type to export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTypeConfig {
    /// Name identifier for this graph configuration.
    #[serde(default = "default_graph_name")]
    pub name: String,

    /// Whether to aggregate parallel edges between the same nodes.
    #[serde(default)]
    pub aggregate_edges: bool,

    /// Minimum edge weight to include (filters out small transactions).
    #[serde(default)]
    pub min_edge_weight: f64,

    /// Whether to include document nodes (creates hub-and-spoke structure).
    #[serde(default)]
    pub include_document_nodes: bool,
}

fn default_graph_name() -> String {
    "accounting_network".to_string()
}

impl Default for GraphTypeConfig {
    fn default() -> Self {
        Self {
            name: "accounting_network".to_string(),
            aggregate_edges: false,
            min_edge_weight: 0.0,
            include_document_nodes: false,
        }
    }
}

/// Export format for graph data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphExportFormat {
    /// PyTorch Geometric format (.npy files + metadata.json).
    PytorchGeometric,
    /// Neo4j format (CSV files + Cypher import scripts).
    Neo4j,
    /// Deep Graph Library format.
    Dgl,
    /// RustGraph/RustAssureTwin JSON format.
    RustGraph,
    /// RustGraph multi-layer hypergraph format (nodes.jsonl + edges.jsonl + hyperedges.jsonl).
    RustGraphHypergraph,
}

/// Scenario configuration for metadata, tagging, and ML training setup.
///
/// This section enables tracking the purpose and characteristics of a generation run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioConfig {
    /// Tags for categorizing and filtering datasets.
    /// Examples: "fraud_detection", "retail", "month_end_stress", "ml_training"
    #[serde(default)]
    pub tags: Vec<String>,

    /// Data quality profile preset.
    /// - "clean": Minimal data quality issues (0.1% missing, 0.05% typos)
    /// - "noisy": Moderate issues (5% missing, 2% typos, 1% duplicates)
    /// - "legacy": Heavy issues simulating legacy system data (10% missing, 5% typos)
    #[serde(default)]
    pub profile: Option<String>,

    /// Human-readable description of the scenario purpose.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether this run is for ML training (enables balanced labeling).
    #[serde(default)]
    pub ml_training: bool,

    /// Target anomaly class balance for ML training.
    /// If set, anomalies will be injected to achieve this ratio.
    #[serde(default)]
    pub target_anomaly_ratio: Option<f64>,

    /// Custom metadata key-value pairs.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Temporal drift configuration for simulating distribution changes over time.
///
/// This enables generation of data that shows realistic temporal evolution,
/// useful for training drift detection models and testing temporal robustness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDriftConfig {
    /// Enable temporal drift simulation.
    #[serde(default)]
    pub enabled: bool,

    /// Amount mean drift per period (e.g., 0.02 = 2% mean shift per month).
    /// Simulates gradual inflation or business growth.
    #[serde(default = "default_amount_drift")]
    pub amount_mean_drift: f64,

    /// Amount variance drift per period (e.g., 0.01 = 1% variance increase per month).
    /// Simulates increasing volatility over time.
    #[serde(default)]
    pub amount_variance_drift: f64,

    /// Anomaly rate drift per period (e.g., 0.001 = 0.1% increase per month).
    /// Simulates increasing fraud attempts or degrading controls.
    #[serde(default)]
    pub anomaly_rate_drift: f64,

    /// Concept drift rate - how quickly feature distributions change (0.0-1.0).
    /// Higher values cause more rapid distribution shifts.
    #[serde(default = "default_concept_drift")]
    pub concept_drift_rate: f64,

    /// Sudden drift events - probability of a sudden distribution shift in any period.
    #[serde(default)]
    pub sudden_drift_probability: f64,

    /// Magnitude of sudden drift events when they occur (multiplier).
    #[serde(default = "default_sudden_drift_magnitude")]
    pub sudden_drift_magnitude: f64,

    /// Seasonal drift - enable cyclic patterns that repeat annually.
    #[serde(default)]
    pub seasonal_drift: bool,

    /// Drift start period (0 = from beginning). Use to simulate stable baseline before drift.
    #[serde(default)]
    pub drift_start_period: u32,

    /// Drift type: "gradual", "sudden", "recurring", "mixed"
    #[serde(default = "default_drift_type")]
    pub drift_type: DriftType,
}

fn default_amount_drift() -> f64 {
    0.02
}

fn default_concept_drift() -> f64 {
    0.01
}

fn default_sudden_drift_magnitude() -> f64 {
    2.0
}

fn default_drift_type() -> DriftType {
    DriftType::Gradual
}

impl Default for TemporalDriftConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            amount_mean_drift: 0.02,
            amount_variance_drift: 0.0,
            anomaly_rate_drift: 0.0,
            concept_drift_rate: 0.01,
            sudden_drift_probability: 0.0,
            sudden_drift_magnitude: 2.0,
            seasonal_drift: false,
            drift_start_period: 0,
            drift_type: DriftType::Gradual,
        }
    }
}

impl TemporalDriftConfig {
    /// Convert to core DriftConfig for use in generators.
    pub fn to_core_config(&self) -> datasynth_core::distributions::DriftConfig {
        datasynth_core::distributions::DriftConfig {
            enabled: self.enabled,
            amount_mean_drift: self.amount_mean_drift,
            amount_variance_drift: self.amount_variance_drift,
            anomaly_rate_drift: self.anomaly_rate_drift,
            concept_drift_rate: self.concept_drift_rate,
            sudden_drift_probability: self.sudden_drift_probability,
            sudden_drift_magnitude: self.sudden_drift_magnitude,
            seasonal_drift: self.seasonal_drift,
            drift_start_period: self.drift_start_period,
            drift_type: match self.drift_type {
                DriftType::Gradual => datasynth_core::distributions::DriftType::Gradual,
                DriftType::Sudden => datasynth_core::distributions::DriftType::Sudden,
                DriftType::Recurring => datasynth_core::distributions::DriftType::Recurring,
                DriftType::Mixed => datasynth_core::distributions::DriftType::Mixed,
            },
            regime_changes: Vec::new(),
            economic_cycle: Default::default(),
            parameter_drifts: Vec::new(),
        }
    }
}

/// Types of temporal drift patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    /// Gradual, continuous drift over time (like inflation).
    #[default]
    Gradual,
    /// Sudden, point-in-time shifts (like policy changes).
    Sudden,
    /// Recurring patterns that cycle (like seasonal variations).
    Recurring,
    /// Combination of gradual background drift with occasional sudden shifts.
    Mixed,
}

// ============================================================================
// Streaming Output API Configuration (Phase 2)
// ============================================================================

/// Configuration for streaming output API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingSchemaConfig {
    /// Enable streaming output.
    #[serde(default)]
    pub enabled: bool,
    /// Buffer size for streaming (number of items).
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Enable progress reporting.
    #[serde(default = "default_true")]
    pub enable_progress: bool,
    /// Progress reporting interval (number of items).
    #[serde(default = "default_progress_interval")]
    pub progress_interval: u64,
    /// Backpressure strategy.
    #[serde(default)]
    pub backpressure: BackpressureSchemaStrategy,
}

fn default_buffer_size() -> usize {
    1000
}

fn default_progress_interval() -> u64 {
    100
}

impl Default for StreamingSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            buffer_size: 1000,
            enable_progress: true,
            progress_interval: 100,
            backpressure: BackpressureSchemaStrategy::Block,
        }
    }
}

/// Backpressure strategy for streaming output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackpressureSchemaStrategy {
    /// Block until space is available in the buffer.
    #[default]
    Block,
    /// Drop oldest items when buffer is full.
    DropOldest,
    /// Drop newest items when buffer is full.
    DropNewest,
    /// Buffer overflow items up to a limit, then block.
    Buffer,
}

// ============================================================================
// Rate Limiting Configuration (Phase 5)
// ============================================================================

/// Configuration for rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSchemaConfig {
    /// Enable rate limiting.
    #[serde(default)]
    pub enabled: bool,
    /// Entities per second limit.
    #[serde(default = "default_entities_per_second")]
    pub entities_per_second: f64,
    /// Burst size (number of tokens in bucket).
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
    /// Backpressure strategy for rate limiting.
    #[serde(default)]
    pub backpressure: RateLimitBackpressureSchema,
}

fn default_entities_per_second() -> f64 {
    1000.0
}

fn default_burst_size() -> u32 {
    100
}

impl Default for RateLimitSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            entities_per_second: 1000.0,
            burst_size: 100,
            backpressure: RateLimitBackpressureSchema::Block,
        }
    }
}

/// Backpressure strategy for rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitBackpressureSchema {
    /// Block until rate allows.
    #[default]
    Block,
    /// Drop items that exceed rate.
    Drop,
    /// Buffer items and process when rate allows.
    Buffer,
}

// ============================================================================
// Temporal Attribute Generation Configuration (Phase 3)
// ============================================================================

/// Configuration for temporal attribute generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAttributeSchemaConfig {
    /// Enable temporal attribute generation.
    #[serde(default)]
    pub enabled: bool,
    /// Valid time configuration.
    #[serde(default)]
    pub valid_time: ValidTimeSchemaConfig,
    /// Transaction time configuration.
    #[serde(default)]
    pub transaction_time: TransactionTimeSchemaConfig,
    /// Generate version chains for entities.
    #[serde(default)]
    pub generate_version_chains: bool,
    /// Average number of versions per entity.
    #[serde(default = "default_avg_versions")]
    pub avg_versions_per_entity: f64,
}

fn default_avg_versions() -> f64 {
    1.5
}

impl Default for TemporalAttributeSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            valid_time: ValidTimeSchemaConfig::default(),
            transaction_time: TransactionTimeSchemaConfig::default(),
            generate_version_chains: false,
            avg_versions_per_entity: 1.5,
        }
    }
}

/// Configuration for valid time (business time) generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidTimeSchemaConfig {
    /// Probability that valid_to is set (entity has ended validity).
    #[serde(default = "default_closed_probability")]
    pub closed_probability: f64,
    /// Average validity duration in days.
    #[serde(default = "default_avg_validity_days")]
    pub avg_validity_days: u32,
    /// Standard deviation of validity duration in days.
    #[serde(default = "default_validity_stddev")]
    pub validity_stddev_days: u32,
}

fn default_closed_probability() -> f64 {
    0.1
}

fn default_avg_validity_days() -> u32 {
    365
}

fn default_validity_stddev() -> u32 {
    90
}

impl Default for ValidTimeSchemaConfig {
    fn default() -> Self {
        Self {
            closed_probability: 0.1,
            avg_validity_days: 365,
            validity_stddev_days: 90,
        }
    }
}

/// Configuration for transaction time (system time) generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTimeSchemaConfig {
    /// Average recording delay in seconds (0 = immediate).
    #[serde(default)]
    pub avg_recording_delay_seconds: u32,
    /// Allow backdating (recording time before valid time).
    #[serde(default)]
    pub allow_backdating: bool,
    /// Probability of backdating if allowed.
    #[serde(default = "default_backdating_probability")]
    pub backdating_probability: f64,
    /// Maximum backdate days.
    #[serde(default = "default_max_backdate_days")]
    pub max_backdate_days: u32,
}

fn default_backdating_probability() -> f64 {
    0.01
}

fn default_max_backdate_days() -> u32 {
    30
}

impl Default for TransactionTimeSchemaConfig {
    fn default() -> Self {
        Self {
            avg_recording_delay_seconds: 0,
            allow_backdating: false,
            backdating_probability: 0.01,
            max_backdate_days: 30,
        }
    }
}

// ============================================================================
// Relationship Generation Configuration (Phase 4)
// ============================================================================

/// Configuration for relationship generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSchemaConfig {
    /// Relationship type definitions.
    #[serde(default)]
    pub relationship_types: Vec<RelationshipTypeSchemaConfig>,
    /// Allow orphan entities (entities with no relationships).
    #[serde(default = "default_true")]
    pub allow_orphans: bool,
    /// Probability of creating an orphan entity.
    #[serde(default = "default_orphan_probability")]
    pub orphan_probability: f64,
    /// Allow circular relationships.
    #[serde(default)]
    pub allow_circular: bool,
    /// Maximum depth for circular relationship detection.
    #[serde(default = "default_max_circular_depth")]
    pub max_circular_depth: u32,
}

fn default_orphan_probability() -> f64 {
    0.01
}

fn default_max_circular_depth() -> u32 {
    3
}

impl Default for RelationshipSchemaConfig {
    fn default() -> Self {
        Self {
            relationship_types: Vec::new(),
            allow_orphans: true,
            orphan_probability: 0.01,
            allow_circular: false,
            max_circular_depth: 3,
        }
    }
}

/// Configuration for a specific relationship type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTypeSchemaConfig {
    /// Name of the relationship type (e.g., "debits", "credits", "created").
    pub name: String,
    /// Source entity type (e.g., "journal_entry").
    pub source_type: String,
    /// Target entity type (e.g., "account").
    pub target_type: String,
    /// Cardinality rule for this relationship.
    #[serde(default)]
    pub cardinality: CardinalitySchemaRule,
    /// Weight for this relationship in random selection.
    #[serde(default = "default_relationship_weight")]
    pub weight: f64,
    /// Whether this relationship is required.
    #[serde(default)]
    pub required: bool,
    /// Whether this relationship is directed.
    #[serde(default = "default_true")]
    pub directed: bool,
}

fn default_relationship_weight() -> f64 {
    1.0
}

impl Default for RelationshipTypeSchemaConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_type: String::new(),
            target_type: String::new(),
            cardinality: CardinalitySchemaRule::default(),
            weight: 1.0,
            required: false,
            directed: true,
        }
    }
}

/// Cardinality rule for relationships in schema config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardinalitySchemaRule {
    /// One source to one target.
    OneToOne,
    /// One source to many targets.
    OneToMany {
        /// Minimum number of targets.
        min: u32,
        /// Maximum number of targets.
        max: u32,
    },
    /// Many sources to one target.
    ManyToOne {
        /// Minimum number of sources.
        min: u32,
        /// Maximum number of sources.
        max: u32,
    },
    /// Many sources to many targets.
    ManyToMany {
        /// Minimum targets per source.
        min_per_source: u32,
        /// Maximum targets per source.
        max_per_source: u32,
    },
}

impl Default for CardinalitySchemaRule {
    fn default() -> Self {
        Self::OneToMany { min: 1, max: 5 }
    }
}

/// Global configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Industry sector
    pub industry: IndustrySector,
    /// Simulation start date (YYYY-MM-DD)
    pub start_date: String,
    /// Simulation period in months
    pub period_months: u32,
    /// Base currency for group reporting
    #[serde(default = "default_currency")]
    pub group_currency: String,
    /// Enable parallel generation
    #[serde(default = "default_true")]
    pub parallel: bool,
    /// Number of worker threads (0 = auto-detect)
    #[serde(default)]
    pub worker_threads: usize,
    /// Memory limit in MB (0 = unlimited)
    #[serde(default)]
    pub memory_limit_mb: usize,
}

fn default_currency() -> String {
    "USD".to_string()
}
fn default_true() -> bool {
    true
}

/// Company code configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfig {
    /// Company code identifier
    pub code: String,
    /// Company name
    pub name: String,
    /// Local currency (ISO 4217)
    pub currency: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
    /// Fiscal year variant
    #[serde(default = "default_fiscal_variant")]
    pub fiscal_year_variant: String,
    /// Transaction volume per year
    pub annual_transaction_volume: TransactionVolume,
    /// Company-specific transaction weight
    #[serde(default = "default_weight")]
    pub volume_weight: f64,
}

fn default_fiscal_variant() -> String {
    "K4".to_string()
}
fn default_weight() -> f64 {
    1.0
}

/// Transaction volume presets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionVolume {
    /// 10,000 transactions per year
    TenK,
    /// 100,000 transactions per year
    HundredK,
    /// 1,000,000 transactions per year
    OneM,
    /// 10,000,000 transactions per year
    TenM,
    /// 100,000,000 transactions per year
    HundredM,
    /// Custom count
    Custom(u64),
}

impl TransactionVolume {
    /// Get the transaction count.
    pub fn count(&self) -> u64 {
        match self {
            Self::TenK => 10_000,
            Self::HundredK => 100_000,
            Self::OneM => 1_000_000,
            Self::TenM => 10_000_000,
            Self::HundredM => 100_000_000,
            Self::Custom(n) => *n,
        }
    }
}

/// Chart of Accounts configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccountsConfig {
    /// CoA complexity level
    pub complexity: CoAComplexity,
    /// Use industry-specific accounts
    #[serde(default = "default_true")]
    pub industry_specific: bool,
    /// Custom account definitions file
    pub custom_accounts: Option<PathBuf>,
    /// Minimum hierarchy depth
    #[serde(default = "default_min_depth")]
    pub min_hierarchy_depth: u8,
    /// Maximum hierarchy depth
    #[serde(default = "default_max_depth")]
    pub max_hierarchy_depth: u8,
}

fn default_min_depth() -> u8 {
    2
}
fn default_max_depth() -> u8 {
    5
}

impl Default for ChartOfAccountsConfig {
    fn default() -> Self {
        Self {
            complexity: CoAComplexity::Small,
            industry_specific: true,
            custom_accounts: None,
            min_hierarchy_depth: default_min_depth(),
            max_hierarchy_depth: default_max_depth(),
        }
    }
}

/// Transaction generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionConfig {
    /// Line item distribution
    #[serde(default)]
    pub line_item_distribution: LineItemDistributionConfig,
    /// Debit/credit balance distribution
    #[serde(default)]
    pub debit_credit_distribution: DebitCreditDistributionConfig,
    /// Even/odd line count distribution
    #[serde(default)]
    pub even_odd_distribution: EvenOddDistributionConfig,
    /// Transaction source distribution
    #[serde(default)]
    pub source_distribution: SourceDistribution,
    /// Seasonality configuration
    #[serde(default)]
    pub seasonality: SeasonalityConfig,
    /// Amount distribution
    #[serde(default)]
    pub amounts: AmountDistributionConfig,
    /// Benford's Law compliance configuration
    #[serde(default)]
    pub benford: BenfordConfig,
}

/// Benford's Law compliance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordConfig {
    /// Enable Benford's Law compliance for amount generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Tolerance for deviation from ideal Benford distribution (0.0-1.0)
    #[serde(default = "default_benford_tolerance")]
    pub tolerance: f64,
    /// Transaction sources exempt from Benford's Law (fixed amounts)
    #[serde(default)]
    pub exempt_sources: Vec<BenfordExemption>,
}

fn default_benford_tolerance() -> f64 {
    0.05
}

impl Default for BenfordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance: default_benford_tolerance(),
            exempt_sources: vec![BenfordExemption::Recurring, BenfordExemption::Payroll],
        }
    }
}

/// Types of transactions exempt from Benford's Law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenfordExemption {
    /// Recurring fixed amounts (rent, subscriptions)
    Recurring,
    /// Payroll (standardized salaries)
    Payroll,
    /// Fixed fees and charges
    FixedFees,
    /// Round number purchases (often legitimate)
    RoundAmounts,
}

/// Distribution of transaction sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDistribution {
    /// Manual entries percentage
    pub manual: f64,
    /// Automated system entries
    pub automated: f64,
    /// Recurring entries
    pub recurring: f64,
    /// Adjustment entries
    pub adjustment: f64,
}

impl Default for SourceDistribution {
    fn default() -> Self {
        Self {
            manual: 0.20,
            automated: 0.70,
            recurring: 0.07,
            adjustment: 0.03,
        }
    }
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output mode
    #[serde(default)]
    pub mode: OutputMode,
    /// Output directory
    pub output_directory: PathBuf,
    /// File formats to generate
    #[serde(default = "default_formats")]
    pub formats: Vec<FileFormat>,
    /// Compression settings
    #[serde(default)]
    pub compression: CompressionConfig,
    /// Batch size for writes
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Include ACDOCA format
    #[serde(default = "default_true")]
    pub include_acdoca: bool,
    /// Include BSEG format
    #[serde(default)]
    pub include_bseg: bool,
    /// Partition by fiscal period
    #[serde(default = "default_true")]
    pub partition_by_period: bool,
    /// Partition by company code
    #[serde(default)]
    pub partition_by_company: bool,
}

fn default_formats() -> Vec<FileFormat> {
    vec![FileFormat::Parquet]
}
fn default_batch_size() -> usize {
    100_000
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            mode: OutputMode::FlatFile,
            output_directory: PathBuf::from("./output"),
            formats: default_formats(),
            compression: CompressionConfig::default(),
            batch_size: default_batch_size(),
            include_acdoca: true,
            include_bseg: false,
            partition_by_period: true,
            partition_by_company: false,
        }
    }
}

/// Output mode.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    /// Stream records as generated
    Streaming,
    /// Write to flat files
    #[default]
    FlatFile,
    /// Both streaming and flat file
    Both,
}

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Csv,
    Parquet,
    Json,
    JsonLines,
}

/// Compression configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Compression algorithm
    #[serde(default)]
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
    pub level: u8,
}

fn default_compression_level() -> u8 {
    3
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::default(),
            level: default_compression_level(),
        }
    }
}

/// Compression algorithms.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionAlgorithm {
    Gzip,
    #[default]
    Zstd,
    Lz4,
    Snappy,
}

/// Fraud simulation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudConfig {
    /// Enable fraud scenario generation
    #[serde(default)]
    pub enabled: bool,
    /// Overall fraud rate (0.0 to 1.0)
    #[serde(default = "default_fraud_rate")]
    pub fraud_rate: f64,
    /// Fraud type distribution
    #[serde(default)]
    pub fraud_type_distribution: FraudTypeDistribution,
    /// Enable fraud clustering
    #[serde(default)]
    pub clustering_enabled: bool,
    /// Clustering factor
    #[serde(default = "default_clustering_factor")]
    pub clustering_factor: f64,
    /// Approval thresholds for threshold-adjacent fraud pattern
    #[serde(default = "default_approval_thresholds")]
    pub approval_thresholds: Vec<f64>,
}

fn default_approval_thresholds() -> Vec<f64> {
    vec![1000.0, 5000.0, 10000.0, 25000.0, 50000.0, 100000.0]
}

fn default_fraud_rate() -> f64 {
    0.005
}
fn default_clustering_factor() -> f64 {
    3.0
}

impl Default for FraudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fraud_rate: default_fraud_rate(),
            fraud_type_distribution: FraudTypeDistribution::default(),
            clustering_enabled: false,
            clustering_factor: default_clustering_factor(),
            approval_thresholds: default_approval_thresholds(),
        }
    }
}

/// Distribution of fraud types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudTypeDistribution {
    pub suspense_account_abuse: f64,
    pub fictitious_transaction: f64,
    pub revenue_manipulation: f64,
    pub expense_capitalization: f64,
    pub split_transaction: f64,
    pub timing_anomaly: f64,
    pub unauthorized_access: f64,
    pub duplicate_payment: f64,
}

impl Default for FraudTypeDistribution {
    fn default() -> Self {
        Self {
            suspense_account_abuse: 0.25,
            fictitious_transaction: 0.15,
            revenue_manipulation: 0.10,
            expense_capitalization: 0.10,
            split_transaction: 0.15,
            timing_anomaly: 0.10,
            unauthorized_access: 0.10,
            duplicate_payment: 0.05,
        }
    }
}

/// Internal Controls System (ICS) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalControlsConfig {
    /// Enable internal controls system
    #[serde(default)]
    pub enabled: bool,
    /// Rate at which controls result in exceptions (0.0 - 1.0)
    #[serde(default = "default_exception_rate")]
    pub exception_rate: f64,
    /// Rate at which SoD violations occur (0.0 - 1.0)
    #[serde(default = "default_sod_violation_rate")]
    pub sod_violation_rate: f64,
    /// Export control master data to separate files
    #[serde(default = "default_true")]
    pub export_control_master_data: bool,
    /// SOX materiality threshold for marking transactions as SOX-relevant
    #[serde(default = "default_sox_materiality_threshold")]
    pub sox_materiality_threshold: f64,
    /// Enable COSO 2013 framework integration
    #[serde(default = "default_true")]
    pub coso_enabled: bool,
    /// Include entity-level controls in generation
    #[serde(default)]
    pub include_entity_level_controls: bool,
    /// Target maturity level for controls
    /// Valid values: "ad_hoc", "repeatable", "defined", "managed", "optimized", "mixed"
    #[serde(default = "default_target_maturity_level")]
    pub target_maturity_level: String,
}

fn default_exception_rate() -> f64 {
    0.02
}

fn default_sod_violation_rate() -> f64 {
    0.01
}

fn default_sox_materiality_threshold() -> f64 {
    10000.0
}

fn default_target_maturity_level() -> String {
    "mixed".to_string()
}

impl Default for InternalControlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            exception_rate: default_exception_rate(),
            sod_violation_rate: default_sod_violation_rate(),
            export_control_master_data: true,
            sox_materiality_threshold: default_sox_materiality_threshold(),
            coso_enabled: true,
            include_entity_level_controls: false,
            target_maturity_level: default_target_maturity_level(),
        }
    }
}

/// Business process configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessProcessConfig {
    /// Order-to-Cash weight
    #[serde(default = "default_o2c")]
    pub o2c_weight: f64,
    /// Procure-to-Pay weight
    #[serde(default = "default_p2p")]
    pub p2p_weight: f64,
    /// Record-to-Report weight
    #[serde(default = "default_r2r")]
    pub r2r_weight: f64,
    /// Hire-to-Retire weight
    #[serde(default = "default_h2r")]
    pub h2r_weight: f64,
    /// Acquire-to-Retire weight
    #[serde(default = "default_a2r")]
    pub a2r_weight: f64,
}

fn default_o2c() -> f64 {
    0.35
}
fn default_p2p() -> f64 {
    0.30
}
fn default_r2r() -> f64 {
    0.20
}
fn default_h2r() -> f64 {
    0.10
}
fn default_a2r() -> f64 {
    0.05
}

impl Default for BusinessProcessConfig {
    fn default() -> Self {
        Self {
            o2c_weight: default_o2c(),
            p2p_weight: default_p2p(),
            r2r_weight: default_r2r(),
            h2r_weight: default_h2r(),
            a2r_weight: default_a2r(),
        }
    }
}

/// User persona configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPersonaConfig {
    /// Distribution of user personas
    #[serde(default)]
    pub persona_distribution: PersonaDistribution,
    /// Users per persona type
    #[serde(default)]
    pub users_per_persona: UsersPerPersona,
}

/// Distribution of user personas for transaction generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaDistribution {
    pub junior_accountant: f64,
    pub senior_accountant: f64,
    pub controller: f64,
    pub manager: f64,
    pub automated_system: f64,
}

impl Default for PersonaDistribution {
    fn default() -> Self {
        Self {
            junior_accountant: 0.15,
            senior_accountant: 0.15,
            controller: 0.05,
            manager: 0.05,
            automated_system: 0.60,
        }
    }
}

/// Number of users per persona type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersPerPersona {
    pub junior_accountant: usize,
    pub senior_accountant: usize,
    pub controller: usize,
    pub manager: usize,
    pub automated_system: usize,
}

impl Default for UsersPerPersona {
    fn default() -> Self {
        Self {
            junior_accountant: 10,
            senior_accountant: 5,
            controller: 2,
            manager: 3,
            automated_system: 20,
        }
    }
}

/// Template configuration for realistic data generation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    /// Name generation settings
    #[serde(default)]
    pub names: NameTemplateConfig,
    /// Description generation settings
    #[serde(default)]
    pub descriptions: DescriptionTemplateConfig,
    /// Reference number settings
    #[serde(default)]
    pub references: ReferenceTemplateConfig,
}

/// Name template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameTemplateConfig {
    /// Distribution of name cultures
    #[serde(default)]
    pub culture_distribution: CultureDistribution,
    /// Email domain for generated users
    #[serde(default = "default_email_domain")]
    pub email_domain: String,
    /// Generate realistic display names
    #[serde(default = "default_true")]
    pub generate_realistic_names: bool,
}

fn default_email_domain() -> String {
    "company.com".to_string()
}

impl Default for NameTemplateConfig {
    fn default() -> Self {
        Self {
            culture_distribution: CultureDistribution::default(),
            email_domain: default_email_domain(),
            generate_realistic_names: true,
        }
    }
}

/// Distribution of name cultures for generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureDistribution {
    pub western_us: f64,
    pub hispanic: f64,
    pub german: f64,
    pub french: f64,
    pub chinese: f64,
    pub japanese: f64,
    pub indian: f64,
}

impl Default for CultureDistribution {
    fn default() -> Self {
        Self {
            western_us: 0.40,
            hispanic: 0.20,
            german: 0.10,
            french: 0.05,
            chinese: 0.10,
            japanese: 0.05,
            indian: 0.10,
        }
    }
}

/// Description template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionTemplateConfig {
    /// Generate header text for journal entries
    #[serde(default = "default_true")]
    pub generate_header_text: bool,
    /// Generate line text for journal entry lines
    #[serde(default = "default_true")]
    pub generate_line_text: bool,
}

impl Default for DescriptionTemplateConfig {
    fn default() -> Self {
        Self {
            generate_header_text: true,
            generate_line_text: true,
        }
    }
}

/// Reference number template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceTemplateConfig {
    /// Generate reference numbers
    #[serde(default = "default_true")]
    pub generate_references: bool,
    /// Invoice prefix
    #[serde(default = "default_invoice_prefix")]
    pub invoice_prefix: String,
    /// Purchase order prefix
    #[serde(default = "default_po_prefix")]
    pub po_prefix: String,
    /// Sales order prefix
    #[serde(default = "default_so_prefix")]
    pub so_prefix: String,
}

fn default_invoice_prefix() -> String {
    "INV".to_string()
}
fn default_po_prefix() -> String {
    "PO".to_string()
}
fn default_so_prefix() -> String {
    "SO".to_string()
}

impl Default for ReferenceTemplateConfig {
    fn default() -> Self {
        Self {
            generate_references: true,
            invoice_prefix: default_invoice_prefix(),
            po_prefix: default_po_prefix(),
            so_prefix: default_so_prefix(),
        }
    }
}

/// Approval workflow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalConfig {
    /// Enable approval workflow generation
    #[serde(default)]
    pub enabled: bool,
    /// Threshold below which transactions are auto-approved
    #[serde(default = "default_auto_approve_threshold")]
    pub auto_approve_threshold: f64,
    /// Rate at which approvals are rejected (0.0 to 1.0)
    #[serde(default = "default_rejection_rate")]
    pub rejection_rate: f64,
    /// Rate at which approvals require revision (0.0 to 1.0)
    #[serde(default = "default_revision_rate")]
    pub revision_rate: f64,
    /// Average delay in hours for approval processing
    #[serde(default = "default_approval_delay_hours")]
    pub average_approval_delay_hours: f64,
    /// Approval chain thresholds
    #[serde(default)]
    pub thresholds: Vec<ApprovalThresholdConfig>,
}

fn default_auto_approve_threshold() -> f64 {
    1000.0
}
fn default_rejection_rate() -> f64 {
    0.02
}
fn default_revision_rate() -> f64 {
    0.05
}
fn default_approval_delay_hours() -> f64 {
    4.0
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_approve_threshold: default_auto_approve_threshold(),
            rejection_rate: default_rejection_rate(),
            revision_rate: default_revision_rate(),
            average_approval_delay_hours: default_approval_delay_hours(),
            thresholds: vec![
                ApprovalThresholdConfig {
                    amount: 1000.0,
                    level: 1,
                    roles: vec!["senior_accountant".to_string()],
                },
                ApprovalThresholdConfig {
                    amount: 10000.0,
                    level: 2,
                    roles: vec!["senior_accountant".to_string(), "controller".to_string()],
                },
                ApprovalThresholdConfig {
                    amount: 100000.0,
                    level: 3,
                    roles: vec![
                        "senior_accountant".to_string(),
                        "controller".to_string(),
                        "manager".to_string(),
                    ],
                },
                ApprovalThresholdConfig {
                    amount: 500000.0,
                    level: 4,
                    roles: vec![
                        "senior_accountant".to_string(),
                        "controller".to_string(),
                        "manager".to_string(),
                        "executive".to_string(),
                    ],
                },
            ],
        }
    }
}

/// Configuration for a single approval threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalThresholdConfig {
    /// Amount threshold
    pub amount: f64,
    /// Approval level required
    pub level: u8,
    /// Roles that can approve at this level
    pub roles: Vec<String>,
}

/// Department configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentConfig {
    /// Enable department assignment
    #[serde(default)]
    pub enabled: bool,
    /// Multiplier for department headcounts
    #[serde(default = "default_headcount_multiplier")]
    pub headcount_multiplier: f64,
    /// Custom department definitions (optional)
    #[serde(default)]
    pub custom_departments: Vec<CustomDepartmentConfig>,
}

fn default_headcount_multiplier() -> f64 {
    1.0
}

impl Default for DepartmentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            headcount_multiplier: default_headcount_multiplier(),
            custom_departments: Vec::new(),
        }
    }
}

/// Custom department definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDepartmentConfig {
    /// Department code
    pub code: String,
    /// Department name
    pub name: String,
    /// Associated cost center
    #[serde(default)]
    pub cost_center: Option<String>,
    /// Primary business processes
    #[serde(default)]
    pub primary_processes: Vec<String>,
    /// Parent department code
    #[serde(default)]
    pub parent_code: Option<String>,
}

// ============================================================================
// Master Data Configuration
// ============================================================================

/// Master data generation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MasterDataConfig {
    /// Vendor master data settings
    #[serde(default)]
    pub vendors: VendorMasterConfig,
    /// Customer master data settings
    #[serde(default)]
    pub customers: CustomerMasterConfig,
    /// Material master data settings
    #[serde(default)]
    pub materials: MaterialMasterConfig,
    /// Fixed asset master data settings
    #[serde(default)]
    pub fixed_assets: FixedAssetMasterConfig,
    /// Employee master data settings
    #[serde(default)]
    pub employees: EmployeeMasterConfig,
    /// Cost center master data settings
    #[serde(default)]
    pub cost_centers: CostCenterMasterConfig,
}

/// Vendor master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorMasterConfig {
    /// Number of vendors to generate
    #[serde(default = "default_vendor_count")]
    pub count: usize,
    /// Percentage of vendors that are intercompany (0.0 to 1.0)
    #[serde(default = "default_intercompany_percent")]
    pub intercompany_percent: f64,
    /// Payment terms distribution
    #[serde(default)]
    pub payment_terms_distribution: PaymentTermsDistribution,
    /// Vendor behavior distribution
    #[serde(default)]
    pub behavior_distribution: VendorBehaviorDistribution,
    /// Generate bank account details
    #[serde(default = "default_true")]
    pub generate_bank_accounts: bool,
    /// Generate tax IDs
    #[serde(default = "default_true")]
    pub generate_tax_ids: bool,
}

fn default_vendor_count() -> usize {
    500
}

fn default_intercompany_percent() -> f64 {
    0.05
}

impl Default for VendorMasterConfig {
    fn default() -> Self {
        Self {
            count: default_vendor_count(),
            intercompany_percent: default_intercompany_percent(),
            payment_terms_distribution: PaymentTermsDistribution::default(),
            behavior_distribution: VendorBehaviorDistribution::default(),
            generate_bank_accounts: true,
            generate_tax_ids: true,
        }
    }
}

/// Payment terms distribution for vendors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermsDistribution {
    /// Net 30 days
    pub net_30: f64,
    /// Net 60 days
    pub net_60: f64,
    /// Net 90 days
    pub net_90: f64,
    /// 2% 10 Net 30 (early payment discount)
    pub two_ten_net_30: f64,
    /// Due on receipt
    pub due_on_receipt: f64,
    /// End of month
    pub end_of_month: f64,
}

impl Default for PaymentTermsDistribution {
    fn default() -> Self {
        Self {
            net_30: 0.40,
            net_60: 0.20,
            net_90: 0.10,
            two_ten_net_30: 0.15,
            due_on_receipt: 0.05,
            end_of_month: 0.10,
        }
    }
}

/// Vendor behavior distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorBehaviorDistribution {
    /// Reliable vendors (consistent delivery, quality)
    pub reliable: f64,
    /// Sometimes late vendors
    pub sometimes_late: f64,
    /// Inconsistent quality vendors
    pub inconsistent_quality: f64,
    /// Premium vendors (high quality, premium pricing)
    pub premium: f64,
    /// Budget vendors (lower quality, lower pricing)
    pub budget: f64,
}

impl Default for VendorBehaviorDistribution {
    fn default() -> Self {
        Self {
            reliable: 0.50,
            sometimes_late: 0.20,
            inconsistent_quality: 0.10,
            premium: 0.10,
            budget: 0.10,
        }
    }
}

/// Customer master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerMasterConfig {
    /// Number of customers to generate
    #[serde(default = "default_customer_count")]
    pub count: usize,
    /// Percentage of customers that are intercompany (0.0 to 1.0)
    #[serde(default = "default_intercompany_percent")]
    pub intercompany_percent: f64,
    /// Credit rating distribution
    #[serde(default)]
    pub credit_rating_distribution: CreditRatingDistribution,
    /// Payment behavior distribution
    #[serde(default)]
    pub payment_behavior_distribution: PaymentBehaviorDistribution,
    /// Generate credit limits based on rating
    #[serde(default = "default_true")]
    pub generate_credit_limits: bool,
}

fn default_customer_count() -> usize {
    2000
}

impl Default for CustomerMasterConfig {
    fn default() -> Self {
        Self {
            count: default_customer_count(),
            intercompany_percent: default_intercompany_percent(),
            credit_rating_distribution: CreditRatingDistribution::default(),
            payment_behavior_distribution: PaymentBehaviorDistribution::default(),
            generate_credit_limits: true,
        }
    }
}

/// Credit rating distribution for customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditRatingDistribution {
    /// AAA rating
    pub aaa: f64,
    /// AA rating
    pub aa: f64,
    /// A rating
    pub a: f64,
    /// BBB rating
    pub bbb: f64,
    /// BB rating
    pub bb: f64,
    /// B rating
    pub b: f64,
    /// Below B rating
    pub below_b: f64,
}

impl Default for CreditRatingDistribution {
    fn default() -> Self {
        Self {
            aaa: 0.05,
            aa: 0.10,
            a: 0.20,
            bbb: 0.30,
            bb: 0.20,
            b: 0.10,
            below_b: 0.05,
        }
    }
}

/// Payment behavior distribution for customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentBehaviorDistribution {
    /// Always pays early
    pub early_payer: f64,
    /// Pays on time
    pub on_time: f64,
    /// Occasionally late
    pub occasional_late: f64,
    /// Frequently late
    pub frequent_late: f64,
    /// Takes early payment discounts
    pub discount_taker: f64,
}

impl Default for PaymentBehaviorDistribution {
    fn default() -> Self {
        Self {
            early_payer: 0.10,
            on_time: 0.50,
            occasional_late: 0.25,
            frequent_late: 0.10,
            discount_taker: 0.05,
        }
    }
}

/// Material master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMasterConfig {
    /// Number of materials to generate
    #[serde(default = "default_material_count")]
    pub count: usize,
    /// Material type distribution
    #[serde(default)]
    pub type_distribution: MaterialTypeDistribution,
    /// Valuation method distribution
    #[serde(default)]
    pub valuation_distribution: ValuationMethodDistribution,
    /// Percentage of materials with BOM (bill of materials)
    #[serde(default = "default_bom_percent")]
    pub bom_percent: f64,
    /// Maximum BOM depth
    #[serde(default = "default_max_bom_depth")]
    pub max_bom_depth: u8,
}

fn default_material_count() -> usize {
    5000
}

fn default_bom_percent() -> f64 {
    0.20
}

fn default_max_bom_depth() -> u8 {
    3
}

impl Default for MaterialMasterConfig {
    fn default() -> Self {
        Self {
            count: default_material_count(),
            type_distribution: MaterialTypeDistribution::default(),
            valuation_distribution: ValuationMethodDistribution::default(),
            bom_percent: default_bom_percent(),
            max_bom_depth: default_max_bom_depth(),
        }
    }
}

/// Material type distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTypeDistribution {
    /// Raw materials
    pub raw_material: f64,
    /// Semi-finished goods
    pub semi_finished: f64,
    /// Finished goods
    pub finished_good: f64,
    /// Trading goods (purchased for resale)
    pub trading_good: f64,
    /// Operating supplies
    pub operating_supply: f64,
    /// Services
    pub service: f64,
}

impl Default for MaterialTypeDistribution {
    fn default() -> Self {
        Self {
            raw_material: 0.30,
            semi_finished: 0.15,
            finished_good: 0.25,
            trading_good: 0.15,
            operating_supply: 0.10,
            service: 0.05,
        }
    }
}

/// Valuation method distribution for materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationMethodDistribution {
    /// Standard cost
    pub standard_cost: f64,
    /// Moving average
    pub moving_average: f64,
    /// FIFO (First In, First Out)
    pub fifo: f64,
    /// LIFO (Last In, First Out)
    pub lifo: f64,
}

impl Default for ValuationMethodDistribution {
    fn default() -> Self {
        Self {
            standard_cost: 0.50,
            moving_average: 0.30,
            fifo: 0.15,
            lifo: 0.05,
        }
    }
}

/// Fixed asset master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedAssetMasterConfig {
    /// Number of fixed assets to generate
    #[serde(default = "default_asset_count")]
    pub count: usize,
    /// Asset class distribution
    #[serde(default)]
    pub class_distribution: AssetClassDistribution,
    /// Depreciation method distribution
    #[serde(default)]
    pub depreciation_distribution: DepreciationMethodDistribution,
    /// Percentage of assets that are fully depreciated
    #[serde(default = "default_fully_depreciated_percent")]
    pub fully_depreciated_percent: f64,
    /// Generate acquisition history
    #[serde(default = "default_true")]
    pub generate_acquisition_history: bool,
}

fn default_asset_count() -> usize {
    800
}

fn default_fully_depreciated_percent() -> f64 {
    0.15
}

impl Default for FixedAssetMasterConfig {
    fn default() -> Self {
        Self {
            count: default_asset_count(),
            class_distribution: AssetClassDistribution::default(),
            depreciation_distribution: DepreciationMethodDistribution::default(),
            fully_depreciated_percent: default_fully_depreciated_percent(),
            generate_acquisition_history: true,
        }
    }
}

/// Asset class distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetClassDistribution {
    /// Buildings and structures
    pub buildings: f64,
    /// Machinery and equipment
    pub machinery: f64,
    /// Vehicles
    pub vehicles: f64,
    /// IT equipment
    pub it_equipment: f64,
    /// Furniture and fixtures
    pub furniture: f64,
    /// Land (non-depreciable)
    pub land: f64,
    /// Leasehold improvements
    pub leasehold: f64,
}

impl Default for AssetClassDistribution {
    fn default() -> Self {
        Self {
            buildings: 0.15,
            machinery: 0.30,
            vehicles: 0.15,
            it_equipment: 0.20,
            furniture: 0.10,
            land: 0.05,
            leasehold: 0.05,
        }
    }
}

/// Depreciation method distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationMethodDistribution {
    /// Straight line
    pub straight_line: f64,
    /// Declining balance
    pub declining_balance: f64,
    /// Double declining balance
    pub double_declining: f64,
    /// Sum of years' digits
    pub sum_of_years: f64,
    /// Units of production
    pub units_of_production: f64,
}

impl Default for DepreciationMethodDistribution {
    fn default() -> Self {
        Self {
            straight_line: 0.60,
            declining_balance: 0.20,
            double_declining: 0.10,
            sum_of_years: 0.05,
            units_of_production: 0.05,
        }
    }
}

/// Employee master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeMasterConfig {
    /// Number of employees to generate
    #[serde(default = "default_employee_count")]
    pub count: usize,
    /// Generate organizational hierarchy
    #[serde(default = "default_true")]
    pub generate_hierarchy: bool,
    /// Maximum hierarchy depth
    #[serde(default = "default_hierarchy_depth")]
    pub max_hierarchy_depth: u8,
    /// Average span of control (direct reports per manager)
    #[serde(default = "default_span_of_control")]
    pub average_span_of_control: f64,
    /// Approval limit distribution by job level
    #[serde(default)]
    pub approval_limits: ApprovalLimitDistribution,
    /// Department distribution
    #[serde(default)]
    pub department_distribution: EmployeeDepartmentDistribution,
}

fn default_employee_count() -> usize {
    1500
}

fn default_hierarchy_depth() -> u8 {
    6
}

fn default_span_of_control() -> f64 {
    5.0
}

impl Default for EmployeeMasterConfig {
    fn default() -> Self {
        Self {
            count: default_employee_count(),
            generate_hierarchy: true,
            max_hierarchy_depth: default_hierarchy_depth(),
            average_span_of_control: default_span_of_control(),
            approval_limits: ApprovalLimitDistribution::default(),
            department_distribution: EmployeeDepartmentDistribution::default(),
        }
    }
}

/// Approval limit distribution by job level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLimitDistribution {
    /// Staff level approval limit
    #[serde(default = "default_staff_limit")]
    pub staff: f64,
    /// Senior staff approval limit
    #[serde(default = "default_senior_limit")]
    pub senior: f64,
    /// Manager approval limit
    #[serde(default = "default_manager_limit")]
    pub manager: f64,
    /// Director approval limit
    #[serde(default = "default_director_limit")]
    pub director: f64,
    /// VP approval limit
    #[serde(default = "default_vp_limit")]
    pub vp: f64,
    /// Executive approval limit
    #[serde(default = "default_executive_limit")]
    pub executive: f64,
}

fn default_staff_limit() -> f64 {
    1000.0
}
fn default_senior_limit() -> f64 {
    5000.0
}
fn default_manager_limit() -> f64 {
    25000.0
}
fn default_director_limit() -> f64 {
    100000.0
}
fn default_vp_limit() -> f64 {
    500000.0
}
fn default_executive_limit() -> f64 {
    f64::INFINITY
}

impl Default for ApprovalLimitDistribution {
    fn default() -> Self {
        Self {
            staff: default_staff_limit(),
            senior: default_senior_limit(),
            manager: default_manager_limit(),
            director: default_director_limit(),
            vp: default_vp_limit(),
            executive: default_executive_limit(),
        }
    }
}

/// Employee distribution across departments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeDepartmentDistribution {
    /// Finance and Accounting
    pub finance: f64,
    /// Procurement
    pub procurement: f64,
    /// Sales
    pub sales: f64,
    /// Warehouse and Logistics
    pub warehouse: f64,
    /// IT
    pub it: f64,
    /// Human Resources
    pub hr: f64,
    /// Operations
    pub operations: f64,
    /// Executive
    pub executive: f64,
}

impl Default for EmployeeDepartmentDistribution {
    fn default() -> Self {
        Self {
            finance: 0.12,
            procurement: 0.10,
            sales: 0.25,
            warehouse: 0.15,
            it: 0.10,
            hr: 0.05,
            operations: 0.20,
            executive: 0.03,
        }
    }
}

/// Cost center master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCenterMasterConfig {
    /// Number of cost centers to generate
    #[serde(default = "default_cost_center_count")]
    pub count: usize,
    /// Generate cost center hierarchy
    #[serde(default = "default_true")]
    pub generate_hierarchy: bool,
    /// Maximum hierarchy depth
    #[serde(default = "default_cc_hierarchy_depth")]
    pub max_hierarchy_depth: u8,
}

fn default_cost_center_count() -> usize {
    50
}

fn default_cc_hierarchy_depth() -> u8 {
    3
}

impl Default for CostCenterMasterConfig {
    fn default() -> Self {
        Self {
            count: default_cost_center_count(),
            generate_hierarchy: true,
            max_hierarchy_depth: default_cc_hierarchy_depth(),
        }
    }
}

// ============================================================================
// Document Flow Configuration
// ============================================================================

/// Document flow generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFlowConfig {
    /// P2P (Procure-to-Pay) flow configuration
    #[serde(default)]
    pub p2p: P2PFlowConfig,
    /// O2C (Order-to-Cash) flow configuration
    #[serde(default)]
    pub o2c: O2CFlowConfig,
    /// Generate document reference chains
    #[serde(default = "default_true")]
    pub generate_document_references: bool,
    /// Export document flow graph
    #[serde(default)]
    pub export_flow_graph: bool,
}

impl Default for DocumentFlowConfig {
    fn default() -> Self {
        Self {
            p2p: P2PFlowConfig::default(),
            o2c: O2CFlowConfig::default(),
            generate_document_references: true,
            export_flow_graph: false,
        }
    }
}

/// P2P (Procure-to-Pay) flow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PFlowConfig {
    /// Enable P2P document flow generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Three-way match success rate (PO-GR-Invoice)
    #[serde(default = "default_three_way_match_rate")]
    pub three_way_match_rate: f64,
    /// Rate of partial deliveries
    #[serde(default = "default_partial_delivery_rate")]
    pub partial_delivery_rate: f64,
    /// Rate of price variances between PO and Invoice
    #[serde(default = "default_price_variance_rate")]
    pub price_variance_rate: f64,
    /// Maximum price variance percentage
    #[serde(default = "default_max_price_variance")]
    pub max_price_variance_percent: f64,
    /// Rate of quantity variances between PO/GR and Invoice
    #[serde(default = "default_quantity_variance_rate")]
    pub quantity_variance_rate: f64,
    /// Average days from PO to goods receipt
    #[serde(default = "default_po_to_gr_days")]
    pub average_po_to_gr_days: u32,
    /// Average days from GR to invoice
    #[serde(default = "default_gr_to_invoice_days")]
    pub average_gr_to_invoice_days: u32,
    /// Average days from invoice to payment
    #[serde(default = "default_invoice_to_payment_days")]
    pub average_invoice_to_payment_days: u32,
    /// PO line count distribution
    #[serde(default)]
    pub line_count_distribution: DocumentLineCountDistribution,
    /// Payment behavior configuration
    #[serde(default)]
    pub payment_behavior: P2PPaymentBehaviorConfig,
}

fn default_three_way_match_rate() -> f64 {
    0.95
}

fn default_partial_delivery_rate() -> f64 {
    0.15
}

fn default_price_variance_rate() -> f64 {
    0.08
}

fn default_max_price_variance() -> f64 {
    0.05
}

fn default_quantity_variance_rate() -> f64 {
    0.05
}

fn default_po_to_gr_days() -> u32 {
    14
}

fn default_gr_to_invoice_days() -> u32 {
    5
}

fn default_invoice_to_payment_days() -> u32 {
    30
}

impl Default for P2PFlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            three_way_match_rate: default_three_way_match_rate(),
            partial_delivery_rate: default_partial_delivery_rate(),
            price_variance_rate: default_price_variance_rate(),
            max_price_variance_percent: default_max_price_variance(),
            quantity_variance_rate: default_quantity_variance_rate(),
            average_po_to_gr_days: default_po_to_gr_days(),
            average_gr_to_invoice_days: default_gr_to_invoice_days(),
            average_invoice_to_payment_days: default_invoice_to_payment_days(),
            line_count_distribution: DocumentLineCountDistribution::default(),
            payment_behavior: P2PPaymentBehaviorConfig::default(),
        }
    }
}

// ============================================================================
// P2P Payment Behavior Configuration
// ============================================================================

/// P2P payment behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PPaymentBehaviorConfig {
    /// Rate of late payments (beyond due date)
    #[serde(default = "default_p2p_late_payment_rate")]
    pub late_payment_rate: f64,
    /// Distribution of late payment days
    #[serde(default)]
    pub late_payment_days_distribution: LatePaymentDaysDistribution,
    /// Rate of partial payments
    #[serde(default = "default_p2p_partial_payment_rate")]
    pub partial_payment_rate: f64,
    /// Rate of payment corrections (NSF, chargebacks, reversals)
    #[serde(default = "default_p2p_payment_correction_rate")]
    pub payment_correction_rate: f64,
}

fn default_p2p_late_payment_rate() -> f64 {
    0.15
}

fn default_p2p_partial_payment_rate() -> f64 {
    0.05
}

fn default_p2p_payment_correction_rate() -> f64 {
    0.02
}

impl Default for P2PPaymentBehaviorConfig {
    fn default() -> Self {
        Self {
            late_payment_rate: default_p2p_late_payment_rate(),
            late_payment_days_distribution: LatePaymentDaysDistribution::default(),
            partial_payment_rate: default_p2p_partial_payment_rate(),
            payment_correction_rate: default_p2p_payment_correction_rate(),
        }
    }
}

/// Distribution of late payment days for P2P.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatePaymentDaysDistribution {
    /// 1-7 days late (slightly late)
    #[serde(default = "default_slightly_late")]
    pub slightly_late_1_to_7: f64,
    /// 8-14 days late
    #[serde(default = "default_late_8_14")]
    pub late_8_to_14: f64,
    /// 15-30 days late (very late)
    #[serde(default = "default_very_late")]
    pub very_late_15_to_30: f64,
    /// 31-60 days late (severely late)
    #[serde(default = "default_severely_late")]
    pub severely_late_31_to_60: f64,
    /// Over 60 days late (extremely late)
    #[serde(default = "default_extremely_late")]
    pub extremely_late_over_60: f64,
}

fn default_slightly_late() -> f64 {
    0.50
}

fn default_late_8_14() -> f64 {
    0.25
}

fn default_very_late() -> f64 {
    0.15
}

fn default_severely_late() -> f64 {
    0.07
}

fn default_extremely_late() -> f64 {
    0.03
}

impl Default for LatePaymentDaysDistribution {
    fn default() -> Self {
        Self {
            slightly_late_1_to_7: default_slightly_late(),
            late_8_to_14: default_late_8_14(),
            very_late_15_to_30: default_very_late(),
            severely_late_31_to_60: default_severely_late(),
            extremely_late_over_60: default_extremely_late(),
        }
    }
}

/// O2C (Order-to-Cash) flow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct O2CFlowConfig {
    /// Enable O2C document flow generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Credit check failure rate
    #[serde(default = "default_credit_check_failure_rate")]
    pub credit_check_failure_rate: f64,
    /// Rate of partial shipments
    #[serde(default = "default_partial_shipment_rate")]
    pub partial_shipment_rate: f64,
    /// Rate of returns
    #[serde(default = "default_return_rate")]
    pub return_rate: f64,
    /// Bad debt write-off rate
    #[serde(default = "default_bad_debt_rate")]
    pub bad_debt_rate: f64,
    /// Average days from SO to delivery
    #[serde(default = "default_so_to_delivery_days")]
    pub average_so_to_delivery_days: u32,
    /// Average days from delivery to invoice
    #[serde(default = "default_delivery_to_invoice_days")]
    pub average_delivery_to_invoice_days: u32,
    /// Average days from invoice to receipt
    #[serde(default = "default_invoice_to_receipt_days")]
    pub average_invoice_to_receipt_days: u32,
    /// SO line count distribution
    #[serde(default)]
    pub line_count_distribution: DocumentLineCountDistribution,
    /// Cash discount configuration
    #[serde(default)]
    pub cash_discount: CashDiscountConfig,
    /// Payment behavior configuration
    #[serde(default)]
    pub payment_behavior: O2CPaymentBehaviorConfig,
}

fn default_credit_check_failure_rate() -> f64 {
    0.02
}

fn default_partial_shipment_rate() -> f64 {
    0.10
}

fn default_return_rate() -> f64 {
    0.03
}

fn default_bad_debt_rate() -> f64 {
    0.01
}

fn default_so_to_delivery_days() -> u32 {
    7
}

fn default_delivery_to_invoice_days() -> u32 {
    1
}

fn default_invoice_to_receipt_days() -> u32 {
    45
}

impl Default for O2CFlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            credit_check_failure_rate: default_credit_check_failure_rate(),
            partial_shipment_rate: default_partial_shipment_rate(),
            return_rate: default_return_rate(),
            bad_debt_rate: default_bad_debt_rate(),
            average_so_to_delivery_days: default_so_to_delivery_days(),
            average_delivery_to_invoice_days: default_delivery_to_invoice_days(),
            average_invoice_to_receipt_days: default_invoice_to_receipt_days(),
            line_count_distribution: DocumentLineCountDistribution::default(),
            cash_discount: CashDiscountConfig::default(),
            payment_behavior: O2CPaymentBehaviorConfig::default(),
        }
    }
}

// ============================================================================
// O2C Payment Behavior Configuration
// ============================================================================

/// O2C payment behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct O2CPaymentBehaviorConfig {
    /// Dunning (Mahnung) configuration
    #[serde(default)]
    pub dunning: DunningConfig,
    /// Partial payment configuration
    #[serde(default)]
    pub partial_payments: PartialPaymentConfig,
    /// Short payment configuration (unauthorized deductions)
    #[serde(default)]
    pub short_payments: ShortPaymentConfig,
    /// On-account payment configuration (unapplied payments)
    #[serde(default)]
    pub on_account_payments: OnAccountPaymentConfig,
    /// Payment correction configuration (NSF, chargebacks)
    #[serde(default)]
    pub payment_corrections: PaymentCorrectionConfig,
}

/// Dunning (Mahnungen) configuration for AR collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningConfig {
    /// Enable dunning process
    #[serde(default)]
    pub enabled: bool,
    /// Days overdue for level 1 dunning (1st reminder)
    #[serde(default = "default_dunning_level_1_days")]
    pub level_1_days_overdue: u32,
    /// Days overdue for level 2 dunning (2nd reminder)
    #[serde(default = "default_dunning_level_2_days")]
    pub level_2_days_overdue: u32,
    /// Days overdue for level 3 dunning (final notice)
    #[serde(default = "default_dunning_level_3_days")]
    pub level_3_days_overdue: u32,
    /// Days overdue for collection handover
    #[serde(default = "default_collection_days")]
    pub collection_days_overdue: u32,
    /// Payment rates after each dunning level
    #[serde(default)]
    pub payment_after_dunning_rates: DunningPaymentRates,
    /// Rate of invoices blocked from dunning (disputes)
    #[serde(default = "default_dunning_block_rate")]
    pub dunning_block_rate: f64,
    /// Interest rate per year for overdue amounts
    #[serde(default = "default_dunning_interest_rate")]
    pub interest_rate_per_year: f64,
    /// Fixed dunning charge per letter
    #[serde(default = "default_dunning_charge")]
    pub dunning_charge: f64,
}

fn default_dunning_level_1_days() -> u32 {
    14
}

fn default_dunning_level_2_days() -> u32 {
    28
}

fn default_dunning_level_3_days() -> u32 {
    42
}

fn default_collection_days() -> u32 {
    60
}

fn default_dunning_block_rate() -> f64 {
    0.05
}

fn default_dunning_interest_rate() -> f64 {
    0.09
}

fn default_dunning_charge() -> f64 {
    25.0
}

impl Default for DunningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level_1_days_overdue: default_dunning_level_1_days(),
            level_2_days_overdue: default_dunning_level_2_days(),
            level_3_days_overdue: default_dunning_level_3_days(),
            collection_days_overdue: default_collection_days(),
            payment_after_dunning_rates: DunningPaymentRates::default(),
            dunning_block_rate: default_dunning_block_rate(),
            interest_rate_per_year: default_dunning_interest_rate(),
            dunning_charge: default_dunning_charge(),
        }
    }
}

/// Payment rates after each dunning level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningPaymentRates {
    /// Rate that pays after level 1 reminder
    #[serde(default = "default_after_level_1")]
    pub after_level_1: f64,
    /// Rate that pays after level 2 reminder
    #[serde(default = "default_after_level_2")]
    pub after_level_2: f64,
    /// Rate that pays after level 3 final notice
    #[serde(default = "default_after_level_3")]
    pub after_level_3: f64,
    /// Rate that pays during collection
    #[serde(default = "default_during_collection")]
    pub during_collection: f64,
    /// Rate that never pays (becomes bad debt)
    #[serde(default = "default_never_pay")]
    pub never_pay: f64,
}

fn default_after_level_1() -> f64 {
    0.40
}

fn default_after_level_2() -> f64 {
    0.30
}

fn default_after_level_3() -> f64 {
    0.15
}

fn default_during_collection() -> f64 {
    0.05
}

fn default_never_pay() -> f64 {
    0.10
}

impl Default for DunningPaymentRates {
    fn default() -> Self {
        Self {
            after_level_1: default_after_level_1(),
            after_level_2: default_after_level_2(),
            after_level_3: default_after_level_3(),
            during_collection: default_during_collection(),
            never_pay: default_never_pay(),
        }
    }
}

/// Partial payment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentConfig {
    /// Rate of invoices paid partially
    #[serde(default = "default_partial_payment_rate")]
    pub rate: f64,
    /// Distribution of partial payment percentages
    #[serde(default)]
    pub percentage_distribution: PartialPaymentPercentageDistribution,
    /// Average days until remainder is paid
    #[serde(default = "default_avg_days_until_remainder")]
    pub avg_days_until_remainder: u32,
}

fn default_partial_payment_rate() -> f64 {
    0.08
}

fn default_avg_days_until_remainder() -> u32 {
    30
}

impl Default for PartialPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_partial_payment_rate(),
            percentage_distribution: PartialPaymentPercentageDistribution::default(),
            avg_days_until_remainder: default_avg_days_until_remainder(),
        }
    }
}

/// Distribution of partial payment percentages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentPercentageDistribution {
    /// Pay 25% of invoice
    #[serde(default = "default_partial_25")]
    pub pay_25_percent: f64,
    /// Pay 50% of invoice
    #[serde(default = "default_partial_50")]
    pub pay_50_percent: f64,
    /// Pay 75% of invoice
    #[serde(default = "default_partial_75")]
    pub pay_75_percent: f64,
    /// Pay random percentage
    #[serde(default = "default_partial_random")]
    pub pay_random_percent: f64,
}

fn default_partial_25() -> f64 {
    0.15
}

fn default_partial_50() -> f64 {
    0.50
}

fn default_partial_75() -> f64 {
    0.25
}

fn default_partial_random() -> f64 {
    0.10
}

impl Default for PartialPaymentPercentageDistribution {
    fn default() -> Self {
        Self {
            pay_25_percent: default_partial_25(),
            pay_50_percent: default_partial_50(),
            pay_75_percent: default_partial_75(),
            pay_random_percent: default_partial_random(),
        }
    }
}

/// Short payment configuration (unauthorized deductions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPaymentConfig {
    /// Rate of payments that are short
    #[serde(default = "default_short_payment_rate")]
    pub rate: f64,
    /// Distribution of short payment reasons
    #[serde(default)]
    pub reason_distribution: ShortPaymentReasonDistribution,
    /// Maximum percentage that can be short
    #[serde(default = "default_max_short_percent")]
    pub max_short_percent: f64,
}

fn default_short_payment_rate() -> f64 {
    0.03
}

fn default_max_short_percent() -> f64 {
    0.10
}

impl Default for ShortPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_short_payment_rate(),
            reason_distribution: ShortPaymentReasonDistribution::default(),
            max_short_percent: default_max_short_percent(),
        }
    }
}

/// Distribution of short payment reasons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPaymentReasonDistribution {
    /// Pricing dispute
    #[serde(default = "default_pricing_dispute")]
    pub pricing_dispute: f64,
    /// Quality issue
    #[serde(default = "default_quality_issue")]
    pub quality_issue: f64,
    /// Quantity discrepancy
    #[serde(default = "default_quantity_discrepancy")]
    pub quantity_discrepancy: f64,
    /// Unauthorized deduction
    #[serde(default = "default_unauthorized_deduction")]
    pub unauthorized_deduction: f64,
    /// Early payment discount taken incorrectly
    #[serde(default = "default_incorrect_discount")]
    pub incorrect_discount: f64,
}

fn default_pricing_dispute() -> f64 {
    0.30
}

fn default_quality_issue() -> f64 {
    0.20
}

fn default_quantity_discrepancy() -> f64 {
    0.20
}

fn default_unauthorized_deduction() -> f64 {
    0.15
}

fn default_incorrect_discount() -> f64 {
    0.15
}

impl Default for ShortPaymentReasonDistribution {
    fn default() -> Self {
        Self {
            pricing_dispute: default_pricing_dispute(),
            quality_issue: default_quality_issue(),
            quantity_discrepancy: default_quantity_discrepancy(),
            unauthorized_deduction: default_unauthorized_deduction(),
            incorrect_discount: default_incorrect_discount(),
        }
    }
}

/// On-account payment configuration (unapplied payments).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnAccountPaymentConfig {
    /// Rate of payments that are on-account (unapplied)
    #[serde(default = "default_on_account_rate")]
    pub rate: f64,
    /// Average days until on-account payments are applied
    #[serde(default = "default_avg_days_until_applied")]
    pub avg_days_until_applied: u32,
}

fn default_on_account_rate() -> f64 {
    0.02
}

fn default_avg_days_until_applied() -> u32 {
    14
}

impl Default for OnAccountPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_on_account_rate(),
            avg_days_until_applied: default_avg_days_until_applied(),
        }
    }
}

/// Payment correction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCorrectionConfig {
    /// Rate of payments requiring correction
    #[serde(default = "default_payment_correction_rate")]
    pub rate: f64,
    /// Distribution of correction types
    #[serde(default)]
    pub type_distribution: PaymentCorrectionTypeDistribution,
}

fn default_payment_correction_rate() -> f64 {
    0.02
}

impl Default for PaymentCorrectionConfig {
    fn default() -> Self {
        Self {
            rate: default_payment_correction_rate(),
            type_distribution: PaymentCorrectionTypeDistribution::default(),
        }
    }
}

/// Distribution of payment correction types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCorrectionTypeDistribution {
    /// NSF (Non-sufficient funds) / bounced check
    #[serde(default = "default_nsf_rate")]
    pub nsf: f64,
    /// Chargeback
    #[serde(default = "default_chargeback_rate")]
    pub chargeback: f64,
    /// Wrong amount applied
    #[serde(default = "default_wrong_amount_rate")]
    pub wrong_amount: f64,
    /// Wrong customer applied
    #[serde(default = "default_wrong_customer_rate")]
    pub wrong_customer: f64,
    /// Duplicate payment
    #[serde(default = "default_duplicate_payment_rate")]
    pub duplicate_payment: f64,
}

fn default_nsf_rate() -> f64 {
    0.30
}

fn default_chargeback_rate() -> f64 {
    0.20
}

fn default_wrong_amount_rate() -> f64 {
    0.20
}

fn default_wrong_customer_rate() -> f64 {
    0.15
}

fn default_duplicate_payment_rate() -> f64 {
    0.15
}

impl Default for PaymentCorrectionTypeDistribution {
    fn default() -> Self {
        Self {
            nsf: default_nsf_rate(),
            chargeback: default_chargeback_rate(),
            wrong_amount: default_wrong_amount_rate(),
            wrong_customer: default_wrong_customer_rate(),
            duplicate_payment: default_duplicate_payment_rate(),
        }
    }
}

/// Document line count distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLineCountDistribution {
    /// Minimum number of lines
    #[serde(default = "default_min_lines")]
    pub min_lines: u32,
    /// Maximum number of lines
    #[serde(default = "default_max_lines")]
    pub max_lines: u32,
    /// Most common line count (mode)
    #[serde(default = "default_mode_lines")]
    pub mode_lines: u32,
}

fn default_min_lines() -> u32 {
    1
}

fn default_max_lines() -> u32 {
    20
}

fn default_mode_lines() -> u32 {
    3
}

impl Default for DocumentLineCountDistribution {
    fn default() -> Self {
        Self {
            min_lines: default_min_lines(),
            max_lines: default_max_lines(),
            mode_lines: default_mode_lines(),
        }
    }
}

/// Cash discount configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashDiscountConfig {
    /// Percentage of invoices eligible for cash discount
    #[serde(default = "default_discount_eligible_rate")]
    pub eligible_rate: f64,
    /// Rate at which customers take the discount
    #[serde(default = "default_discount_taken_rate")]
    pub taken_rate: f64,
    /// Standard discount percentage
    #[serde(default = "default_discount_percent")]
    pub discount_percent: f64,
    /// Days within which discount must be taken
    #[serde(default = "default_discount_days")]
    pub discount_days: u32,
}

fn default_discount_eligible_rate() -> f64 {
    0.30
}

fn default_discount_taken_rate() -> f64 {
    0.60
}

fn default_discount_percent() -> f64 {
    0.02
}

fn default_discount_days() -> u32 {
    10
}

impl Default for CashDiscountConfig {
    fn default() -> Self {
        Self {
            eligible_rate: default_discount_eligible_rate(),
            taken_rate: default_discount_taken_rate(),
            discount_percent: default_discount_percent(),
            discount_days: default_discount_days(),
        }
    }
}

// ============================================================================
// Intercompany Configuration
// ============================================================================

/// Intercompany transaction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntercompanyConfig {
    /// Enable intercompany transaction generation
    #[serde(default)]
    pub enabled: bool,
    /// Rate of transactions that are intercompany
    #[serde(default = "default_ic_transaction_rate")]
    pub ic_transaction_rate: f64,
    /// Transfer pricing method
    #[serde(default)]
    pub transfer_pricing_method: TransferPricingMethod,
    /// Transfer pricing markup percentage (for cost-plus)
    #[serde(default = "default_markup_percent")]
    pub markup_percent: f64,
    /// Generate matched IC pairs (offsetting entries)
    #[serde(default = "default_true")]
    pub generate_matched_pairs: bool,
    /// IC transaction type distribution
    #[serde(default)]
    pub transaction_type_distribution: ICTransactionTypeDistribution,
    /// Generate elimination entries for consolidation
    #[serde(default)]
    pub generate_eliminations: bool,
}

fn default_ic_transaction_rate() -> f64 {
    0.15
}

fn default_markup_percent() -> f64 {
    0.05
}

impl Default for IntercompanyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ic_transaction_rate: default_ic_transaction_rate(),
            transfer_pricing_method: TransferPricingMethod::default(),
            markup_percent: default_markup_percent(),
            generate_matched_pairs: true,
            transaction_type_distribution: ICTransactionTypeDistribution::default(),
            generate_eliminations: false,
        }
    }
}

/// Transfer pricing method.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferPricingMethod {
    /// Cost plus a markup
    #[default]
    CostPlus,
    /// Comparable uncontrolled price
    ComparableUncontrolled,
    /// Resale price method
    ResalePrice,
    /// Transactional net margin method
    TransactionalNetMargin,
    /// Profit split method
    ProfitSplit,
}

/// IC transaction type distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICTransactionTypeDistribution {
    /// Goods sales between entities
    pub goods_sale: f64,
    /// Services provided
    pub service_provided: f64,
    /// Intercompany loans
    pub loan: f64,
    /// Dividends
    pub dividend: f64,
    /// Management fees
    pub management_fee: f64,
    /// Royalties
    pub royalty: f64,
    /// Cost sharing
    pub cost_sharing: f64,
}

impl Default for ICTransactionTypeDistribution {
    fn default() -> Self {
        Self {
            goods_sale: 0.35,
            service_provided: 0.20,
            loan: 0.10,
            dividend: 0.05,
            management_fee: 0.15,
            royalty: 0.10,
            cost_sharing: 0.05,
        }
    }
}

// ============================================================================
// Balance Configuration
// ============================================================================

/// Balance and trial balance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceConfig {
    /// Generate opening balances
    #[serde(default)]
    pub generate_opening_balances: bool,
    /// Generate trial balances
    #[serde(default = "default_true")]
    pub generate_trial_balances: bool,
    /// Target gross margin (for revenue/COGS coherence)
    #[serde(default = "default_gross_margin")]
    pub target_gross_margin: f64,
    /// Target DSO (Days Sales Outstanding)
    #[serde(default = "default_dso")]
    pub target_dso_days: u32,
    /// Target DPO (Days Payable Outstanding)
    #[serde(default = "default_dpo")]
    pub target_dpo_days: u32,
    /// Target current ratio
    #[serde(default = "default_current_ratio")]
    pub target_current_ratio: f64,
    /// Target debt-to-equity ratio
    #[serde(default = "default_debt_equity")]
    pub target_debt_to_equity: f64,
    /// Validate balance sheet equation (A = L + E)
    #[serde(default = "default_true")]
    pub validate_balance_equation: bool,
    /// Reconcile subledgers to GL control accounts
    #[serde(default = "default_true")]
    pub reconcile_subledgers: bool,
}

fn default_gross_margin() -> f64 {
    0.35
}

fn default_dso() -> u32 {
    45
}

fn default_dpo() -> u32 {
    30
}

fn default_current_ratio() -> f64 {
    1.5
}

fn default_debt_equity() -> f64 {
    0.5
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            generate_opening_balances: false,
            generate_trial_balances: true,
            target_gross_margin: default_gross_margin(),
            target_dso_days: default_dso(),
            target_dpo_days: default_dpo(),
            target_current_ratio: default_current_ratio(),
            target_debt_to_equity: default_debt_equity(),
            validate_balance_equation: true,
            reconcile_subledgers: true,
        }
    }
}

// ==========================================================================
// OCPM (Object-Centric Process Mining) Configuration
// ==========================================================================

/// OCPM (Object-Centric Process Mining) configuration.
///
/// Controls generation of OCEL 2.0 compatible event logs with
/// many-to-many event-to-object relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmConfig {
    /// Enable OCPM event log generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate lifecycle events (Start/Complete pairs vs atomic events)
    #[serde(default = "default_true")]
    pub generate_lifecycle_events: bool,

    /// Include object-to-object relationships in output
    #[serde(default = "default_true")]
    pub include_object_relationships: bool,

    /// Compute and export process variants
    #[serde(default = "default_true")]
    pub compute_variants: bool,

    /// Maximum variants to track (0 = unlimited)
    #[serde(default)]
    pub max_variants: usize,

    /// P2P process configuration
    #[serde(default)]
    pub p2p_process: OcpmProcessConfig,

    /// O2C process configuration
    #[serde(default)]
    pub o2c_process: OcpmProcessConfig,

    /// Output format configuration
    #[serde(default)]
    pub output: OcpmOutputConfig,
}

impl Default for OcpmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_lifecycle_events: true,
            include_object_relationships: true,
            compute_variants: true,
            max_variants: 0,
            p2p_process: OcpmProcessConfig::default(),
            o2c_process: OcpmProcessConfig::default(),
            output: OcpmOutputConfig::default(),
        }
    }
}

/// Process-specific OCPM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmProcessConfig {
    /// Rework probability (0.0-1.0)
    #[serde(default = "default_rework_probability")]
    pub rework_probability: f64,

    /// Skip step probability (0.0-1.0)
    #[serde(default = "default_skip_probability")]
    pub skip_step_probability: f64,

    /// Out-of-order step probability (0.0-1.0)
    #[serde(default = "default_out_of_order_probability")]
    pub out_of_order_probability: f64,
}

fn default_rework_probability() -> f64 {
    0.05
}

fn default_skip_probability() -> f64 {
    0.02
}

fn default_out_of_order_probability() -> f64 {
    0.03
}

impl Default for OcpmProcessConfig {
    fn default() -> Self {
        Self {
            rework_probability: default_rework_probability(),
            skip_step_probability: default_skip_probability(),
            out_of_order_probability: default_out_of_order_probability(),
        }
    }
}

/// OCPM output format configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmOutputConfig {
    /// Export OCEL 2.0 JSON format
    #[serde(default = "default_true")]
    pub ocel_json: bool,

    /// Export OCEL 2.0 XML format
    #[serde(default)]
    pub ocel_xml: bool,

    /// Export XES 2.0 XML format (IEEE standard for process mining tools)
    #[serde(default)]
    pub xes: bool,

    /// Include lifecycle transitions in XES output (start/complete pairs)
    #[serde(default = "default_true")]
    pub xes_include_lifecycle: bool,

    /// Include resource attributes in XES output
    #[serde(default = "default_true")]
    pub xes_include_resources: bool,

    /// Export flattened CSV for each object type
    #[serde(default = "default_true")]
    pub flattened_csv: bool,

    /// Export event-object relationship table
    #[serde(default = "default_true")]
    pub event_object_csv: bool,

    /// Export object-object relationship table
    #[serde(default = "default_true")]
    pub object_relationship_csv: bool,

    /// Export process variants summary
    #[serde(default = "default_true")]
    pub variants_csv: bool,

    /// Export reference process models (canonical P2P, O2C, R2R)
    #[serde(default)]
    pub export_reference_models: bool,
}

impl Default for OcpmOutputConfig {
    fn default() -> Self {
        Self {
            ocel_json: true,
            ocel_xml: false,
            xes: false,
            xes_include_lifecycle: true,
            xes_include_resources: true,
            flattened_csv: true,
            event_object_csv: true,
            object_relationship_csv: true,
            variants_csv: true,
            export_reference_models: false,
        }
    }
}

/// Audit engagement and workpaper generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditGenerationConfig {
    /// Enable audit engagement generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate engagement documents and workpapers
    #[serde(default = "default_true")]
    pub generate_workpapers: bool,

    /// Default engagement type distribution
    #[serde(default)]
    pub engagement_types: AuditEngagementTypesConfig,

    /// Workpaper configuration
    #[serde(default)]
    pub workpapers: WorkpaperConfig,

    /// Team configuration
    #[serde(default)]
    pub team: AuditTeamConfig,

    /// Review workflow configuration
    #[serde(default)]
    pub review: ReviewWorkflowConfig,
}

impl Default for AuditGenerationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_workpapers: true,
            engagement_types: AuditEngagementTypesConfig::default(),
            workpapers: WorkpaperConfig::default(),
            team: AuditTeamConfig::default(),
            review: ReviewWorkflowConfig::default(),
        }
    }
}

/// Engagement type distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEngagementTypesConfig {
    /// Financial statement audit probability
    #[serde(default = "default_financial_audit_prob")]
    pub financial_statement: f64,
    /// SOX/ICFR audit probability
    #[serde(default = "default_sox_audit_prob")]
    pub sox_icfr: f64,
    /// Integrated audit probability
    #[serde(default = "default_integrated_audit_prob")]
    pub integrated: f64,
    /// Review engagement probability
    #[serde(default = "default_review_prob")]
    pub review: f64,
    /// Agreed-upon procedures probability
    #[serde(default = "default_aup_prob")]
    pub agreed_upon_procedures: f64,
}

fn default_financial_audit_prob() -> f64 {
    0.40
}
fn default_sox_audit_prob() -> f64 {
    0.20
}
fn default_integrated_audit_prob() -> f64 {
    0.25
}
fn default_review_prob() -> f64 {
    0.10
}
fn default_aup_prob() -> f64 {
    0.05
}

impl Default for AuditEngagementTypesConfig {
    fn default() -> Self {
        Self {
            financial_statement: default_financial_audit_prob(),
            sox_icfr: default_sox_audit_prob(),
            integrated: default_integrated_audit_prob(),
            review: default_review_prob(),
            agreed_upon_procedures: default_aup_prob(),
        }
    }
}

/// Workpaper generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpaperConfig {
    /// Average workpapers per engagement phase
    #[serde(default = "default_workpapers_per_phase")]
    pub average_per_phase: usize,

    /// Include ISA compliance references
    #[serde(default = "default_true")]
    pub include_isa_references: bool,

    /// Generate sample details
    #[serde(default = "default_true")]
    pub include_sample_details: bool,

    /// Include cross-references between workpapers
    #[serde(default = "default_true")]
    pub include_cross_references: bool,

    /// Sampling configuration
    #[serde(default)]
    pub sampling: SamplingConfig,
}

fn default_workpapers_per_phase() -> usize {
    5
}

impl Default for WorkpaperConfig {
    fn default() -> Self {
        Self {
            average_per_phase: default_workpapers_per_phase(),
            include_isa_references: true,
            include_sample_details: true,
            include_cross_references: true,
            sampling: SamplingConfig::default(),
        }
    }
}

/// Sampling method configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Statistical sampling rate (0.0-1.0)
    #[serde(default = "default_statistical_rate")]
    pub statistical_rate: f64,
    /// Judgmental sampling rate (0.0-1.0)
    #[serde(default = "default_judgmental_rate")]
    pub judgmental_rate: f64,
    /// Haphazard sampling rate (0.0-1.0)
    #[serde(default = "default_haphazard_rate")]
    pub haphazard_rate: f64,
    /// 100% examination rate (0.0-1.0)
    #[serde(default = "default_complete_examination_rate")]
    pub complete_examination_rate: f64,
}

fn default_statistical_rate() -> f64 {
    0.40
}
fn default_judgmental_rate() -> f64 {
    0.30
}
fn default_haphazard_rate() -> f64 {
    0.20
}
fn default_complete_examination_rate() -> f64 {
    0.10
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            statistical_rate: default_statistical_rate(),
            judgmental_rate: default_judgmental_rate(),
            haphazard_rate: default_haphazard_rate(),
            complete_examination_rate: default_complete_examination_rate(),
        }
    }
}

/// Audit team configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTeamConfig {
    /// Minimum team size
    #[serde(default = "default_min_team_size")]
    pub min_team_size: usize,
    /// Maximum team size
    #[serde(default = "default_max_team_size")]
    pub max_team_size: usize,
    /// Probability of having a specialist on the team
    #[serde(default = "default_specialist_probability")]
    pub specialist_probability: f64,
}

fn default_min_team_size() -> usize {
    3
}
fn default_max_team_size() -> usize {
    8
}
fn default_specialist_probability() -> f64 {
    0.30
}

impl Default for AuditTeamConfig {
    fn default() -> Self {
        Self {
            min_team_size: default_min_team_size(),
            max_team_size: default_max_team_size(),
            specialist_probability: default_specialist_probability(),
        }
    }
}

/// Review workflow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewWorkflowConfig {
    /// Average days between preparer completion and first review
    #[serde(default = "default_review_delay_days")]
    pub average_review_delay_days: u32,
    /// Probability of review notes requiring rework
    #[serde(default = "default_rework_probability_review")]
    pub rework_probability: f64,
    /// Require partner sign-off for all workpapers
    #[serde(default = "default_true")]
    pub require_partner_signoff: bool,
}

fn default_review_delay_days() -> u32 {
    2
}
fn default_rework_probability_review() -> f64 {
    0.15
}

impl Default for ReviewWorkflowConfig {
    fn default() -> Self {
        Self {
            average_review_delay_days: default_review_delay_days(),
            rework_probability: default_rework_probability_review(),
            require_partner_signoff: true,
        }
    }
}

// =============================================================================
// Data Quality Configuration
// =============================================================================

/// Data quality variation settings for realistic flakiness injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualitySchemaConfig {
    /// Enable data quality variations
    #[serde(default)]
    pub enabled: bool,
    /// Preset to use (overrides individual settings if set)
    #[serde(default)]
    pub preset: DataQualityPreset,
    /// Missing value injection settings
    #[serde(default)]
    pub missing_values: MissingValuesSchemaConfig,
    /// Typo injection settings
    #[serde(default)]
    pub typos: TypoSchemaConfig,
    /// Format variation settings
    #[serde(default)]
    pub format_variations: FormatVariationSchemaConfig,
    /// Duplicate injection settings
    #[serde(default)]
    pub duplicates: DuplicateSchemaConfig,
    /// Encoding issue settings
    #[serde(default)]
    pub encoding_issues: EncodingIssueSchemaConfig,
    /// Generate quality issue labels for ML training
    #[serde(default)]
    pub generate_labels: bool,
    /// Per-sink quality profiles (different settings for CSV vs JSON etc.)
    #[serde(default)]
    pub sink_profiles: SinkQualityProfiles,
}

impl Default for DataQualitySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: DataQualityPreset::None,
            missing_values: MissingValuesSchemaConfig::default(),
            typos: TypoSchemaConfig::default(),
            format_variations: FormatVariationSchemaConfig::default(),
            duplicates: DuplicateSchemaConfig::default(),
            encoding_issues: EncodingIssueSchemaConfig::default(),
            generate_labels: true,
            sink_profiles: SinkQualityProfiles::default(),
        }
    }
}

impl DataQualitySchemaConfig {
    /// Creates a config for a specific preset profile.
    pub fn with_preset(preset: DataQualityPreset) -> Self {
        let mut config = Self {
            preset,
            ..Default::default()
        };
        config.apply_preset();
        config
    }

    /// Applies the preset settings to the individual configuration fields.
    /// Call this after deserializing if preset is not Custom or None.
    pub fn apply_preset(&mut self) {
        if !self.preset.overrides_settings() {
            return;
        }

        self.enabled = true;

        // Missing values
        self.missing_values.enabled = self.preset.missing_rate() > 0.0;
        self.missing_values.rate = self.preset.missing_rate();

        // Typos
        self.typos.enabled = self.preset.typo_rate() > 0.0;
        self.typos.char_error_rate = self.preset.typo_rate();

        // Duplicates
        self.duplicates.enabled = self.preset.duplicate_rate() > 0.0;
        self.duplicates.exact_duplicate_ratio = self.preset.duplicate_rate() * 0.4;
        self.duplicates.near_duplicate_ratio = self.preset.duplicate_rate() * 0.4;
        self.duplicates.fuzzy_duplicate_ratio = self.preset.duplicate_rate() * 0.2;

        // Format variations
        self.format_variations.enabled = self.preset.format_variations_enabled();

        // Encoding issues
        self.encoding_issues.enabled = self.preset.encoding_issues_enabled();
        self.encoding_issues.rate = self.preset.encoding_issue_rate();

        // OCR errors for typos in legacy preset
        if self.preset.ocr_errors_enabled() {
            self.typos.type_weights.ocr_errors = 0.3;
        }
    }

    /// Returns the effective missing value rate (considering preset).
    pub fn effective_missing_rate(&self) -> f64 {
        if self.preset.overrides_settings() {
            self.preset.missing_rate()
        } else {
            self.missing_values.rate
        }
    }

    /// Returns the effective typo rate (considering preset).
    pub fn effective_typo_rate(&self) -> f64 {
        if self.preset.overrides_settings() {
            self.preset.typo_rate()
        } else {
            self.typos.char_error_rate
        }
    }

    /// Returns the effective duplicate rate (considering preset).
    pub fn effective_duplicate_rate(&self) -> f64 {
        if self.preset.overrides_settings() {
            self.preset.duplicate_rate()
        } else {
            self.duplicates.exact_duplicate_ratio
                + self.duplicates.near_duplicate_ratio
                + self.duplicates.fuzzy_duplicate_ratio
        }
    }

    /// Creates a clean profile config.
    pub fn clean() -> Self {
        Self::with_preset(DataQualityPreset::Clean)
    }

    /// Creates a noisy profile config.
    pub fn noisy() -> Self {
        Self::with_preset(DataQualityPreset::Noisy)
    }

    /// Creates a legacy profile config.
    pub fn legacy() -> Self {
        Self::with_preset(DataQualityPreset::Legacy)
    }
}

/// Preset configurations for common data quality scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityPreset {
    /// No data quality variations (clean data)
    #[default]
    None,
    /// Minimal variations (very clean data with rare issues)
    Minimal,
    /// Normal variations (realistic enterprise data quality)
    Normal,
    /// High variations (messy data for stress testing)
    High,
    /// Custom (use individual settings)
    Custom,

    // ========================================
    // ML-Oriented Profiles (Phase 2.1)
    // ========================================
    /// Clean profile for ML training - minimal data quality issues
    /// Missing: 0.1%, Typos: 0.05%, Duplicates: 0%, Format: None
    Clean,
    /// Noisy profile simulating typical production data issues
    /// Missing: 5%, Typos: 2%, Duplicates: 1%, Format: Medium
    Noisy,
    /// Legacy profile simulating migrated/OCR'd historical data
    /// Missing: 10%, Typos: 5%, Duplicates: 3%, Format: Heavy + OCR
    Legacy,
}

impl DataQualityPreset {
    /// Returns the missing value rate for this preset.
    pub fn missing_rate(&self) -> f64 {
        match self {
            DataQualityPreset::None => 0.0,
            DataQualityPreset::Minimal => 0.005,
            DataQualityPreset::Normal => 0.02,
            DataQualityPreset::High => 0.08,
            DataQualityPreset::Custom => 0.01, // Use config value
            DataQualityPreset::Clean => 0.001,
            DataQualityPreset::Noisy => 0.05,
            DataQualityPreset::Legacy => 0.10,
        }
    }

    /// Returns the typo rate for this preset.
    pub fn typo_rate(&self) -> f64 {
        match self {
            DataQualityPreset::None => 0.0,
            DataQualityPreset::Minimal => 0.0005,
            DataQualityPreset::Normal => 0.002,
            DataQualityPreset::High => 0.01,
            DataQualityPreset::Custom => 0.001, // Use config value
            DataQualityPreset::Clean => 0.0005,
            DataQualityPreset::Noisy => 0.02,
            DataQualityPreset::Legacy => 0.05,
        }
    }

    /// Returns the duplicate rate for this preset.
    pub fn duplicate_rate(&self) -> f64 {
        match self {
            DataQualityPreset::None => 0.0,
            DataQualityPreset::Minimal => 0.001,
            DataQualityPreset::Normal => 0.005,
            DataQualityPreset::High => 0.02,
            DataQualityPreset::Custom => 0.0, // Use config value
            DataQualityPreset::Clean => 0.0,
            DataQualityPreset::Noisy => 0.01,
            DataQualityPreset::Legacy => 0.03,
        }
    }

    /// Returns whether format variations are enabled for this preset.
    pub fn format_variations_enabled(&self) -> bool {
        match self {
            DataQualityPreset::None | DataQualityPreset::Clean => false,
            DataQualityPreset::Minimal => true,
            DataQualityPreset::Normal => true,
            DataQualityPreset::High => true,
            DataQualityPreset::Custom => true,
            DataQualityPreset::Noisy => true,
            DataQualityPreset::Legacy => true,
        }
    }

    /// Returns whether OCR-style errors are enabled for this preset.
    pub fn ocr_errors_enabled(&self) -> bool {
        matches!(self, DataQualityPreset::Legacy | DataQualityPreset::High)
    }

    /// Returns whether encoding issues are enabled for this preset.
    pub fn encoding_issues_enabled(&self) -> bool {
        matches!(
            self,
            DataQualityPreset::Legacy | DataQualityPreset::High | DataQualityPreset::Noisy
        )
    }

    /// Returns the encoding issue rate for this preset.
    pub fn encoding_issue_rate(&self) -> f64 {
        match self {
            DataQualityPreset::None | DataQualityPreset::Clean | DataQualityPreset::Minimal => 0.0,
            DataQualityPreset::Normal => 0.002,
            DataQualityPreset::High => 0.01,
            DataQualityPreset::Custom => 0.0,
            DataQualityPreset::Noisy => 0.005,
            DataQualityPreset::Legacy => 0.02,
        }
    }

    /// Returns true if this preset overrides individual settings.
    pub fn overrides_settings(&self) -> bool {
        !matches!(self, DataQualityPreset::Custom | DataQualityPreset::None)
    }

    /// Returns a human-readable description of this preset.
    pub fn description(&self) -> &'static str {
        match self {
            DataQualityPreset::None => "No data quality issues (pristine data)",
            DataQualityPreset::Minimal => "Very rare data quality issues",
            DataQualityPreset::Normal => "Realistic enterprise data quality",
            DataQualityPreset::High => "Messy data for stress testing",
            DataQualityPreset::Custom => "Custom settings from configuration",
            DataQualityPreset::Clean => "ML-ready clean data with minimal issues",
            DataQualityPreset::Noisy => "Typical production data with moderate issues",
            DataQualityPreset::Legacy => "Legacy/migrated data with heavy issues and OCR errors",
        }
    }
}

/// Missing value injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingValuesSchemaConfig {
    /// Enable missing value injection
    #[serde(default)]
    pub enabled: bool,
    /// Global missing rate (0.0 to 1.0)
    #[serde(default = "default_missing_rate")]
    pub rate: f64,
    /// Missing value strategy
    #[serde(default)]
    pub strategy: MissingValueStrategy,
    /// Field-specific rates (field name -> rate)
    #[serde(default)]
    pub field_rates: std::collections::HashMap<String, f64>,
    /// Fields that should never have missing values
    #[serde(default)]
    pub protected_fields: Vec<String>,
}

fn default_missing_rate() -> f64 {
    0.01
}

impl Default for MissingValuesSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_missing_rate(),
            strategy: MissingValueStrategy::Mcar,
            field_rates: std::collections::HashMap::new(),
            protected_fields: vec![
                "document_id".to_string(),
                "company_code".to_string(),
                "posting_date".to_string(),
            ],
        }
    }
}

/// Missing value strategy types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MissingValueStrategy {
    /// Missing Completely At Random - equal probability for all values
    #[default]
    Mcar,
    /// Missing At Random - depends on other observed values
    Mar,
    /// Missing Not At Random - depends on the value itself
    Mnar,
    /// Systematic - entire field groups missing together
    Systematic,
}

/// Typo injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoSchemaConfig {
    /// Enable typo injection
    #[serde(default)]
    pub enabled: bool,
    /// Character error rate (per character, not per field)
    #[serde(default = "default_typo_rate")]
    pub char_error_rate: f64,
    /// Typo type weights
    #[serde(default)]
    pub type_weights: TypoTypeWeights,
    /// Fields that should never have typos
    #[serde(default)]
    pub protected_fields: Vec<String>,
}

fn default_typo_rate() -> f64 {
    0.001
}

impl Default for TypoSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            char_error_rate: default_typo_rate(),
            type_weights: TypoTypeWeights::default(),
            protected_fields: vec![
                "document_id".to_string(),
                "gl_account".to_string(),
                "company_code".to_string(),
            ],
        }
    }
}

/// Weights for different typo types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoTypeWeights {
    /// Keyboard-adjacent substitution (e.g., 'a' -> 's')
    #[serde(default = "default_substitution_weight")]
    pub substitution: f64,
    /// Adjacent character transposition (e.g., 'ab' -> 'ba')
    #[serde(default = "default_transposition_weight")]
    pub transposition: f64,
    /// Character insertion
    #[serde(default = "default_insertion_weight")]
    pub insertion: f64,
    /// Character deletion
    #[serde(default = "default_deletion_weight")]
    pub deletion: f64,
    /// OCR-style errors (e.g., '0' -> 'O')
    #[serde(default = "default_ocr_weight")]
    pub ocr_errors: f64,
    /// Homophone substitution (e.g., 'their' -> 'there')
    #[serde(default = "default_homophone_weight")]
    pub homophones: f64,
}

fn default_substitution_weight() -> f64 {
    0.35
}
fn default_transposition_weight() -> f64 {
    0.25
}
fn default_insertion_weight() -> f64 {
    0.10
}
fn default_deletion_weight() -> f64 {
    0.15
}
fn default_ocr_weight() -> f64 {
    0.10
}
fn default_homophone_weight() -> f64 {
    0.05
}

impl Default for TypoTypeWeights {
    fn default() -> Self {
        Self {
            substitution: default_substitution_weight(),
            transposition: default_transposition_weight(),
            insertion: default_insertion_weight(),
            deletion: default_deletion_weight(),
            ocr_errors: default_ocr_weight(),
            homophones: default_homophone_weight(),
        }
    }
}

/// Format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatVariationSchemaConfig {
    /// Enable format variations
    #[serde(default)]
    pub enabled: bool,
    /// Date format variation settings
    #[serde(default)]
    pub dates: DateFormatVariationConfig,
    /// Amount format variation settings
    #[serde(default)]
    pub amounts: AmountFormatVariationConfig,
    /// Identifier format variation settings
    #[serde(default)]
    pub identifiers: IdentifierFormatVariationConfig,
}

/// Date format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormatVariationConfig {
    /// Enable date format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_date_variation_rate")]
    pub rate: f64,
    /// Include ISO format (2024-01-15)
    #[serde(default = "default_true")]
    pub iso_format: bool,
    /// Include US format (01/15/2024)
    #[serde(default)]
    pub us_format: bool,
    /// Include EU format (15.01.2024)
    #[serde(default)]
    pub eu_format: bool,
    /// Include long format (January 15, 2024)
    #[serde(default)]
    pub long_format: bool,
}

fn default_date_variation_rate() -> f64 {
    0.05
}

impl Default for DateFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_date_variation_rate(),
            iso_format: true,
            us_format: false,
            eu_format: false,
            long_format: false,
        }
    }
}

/// Amount format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountFormatVariationConfig {
    /// Enable amount format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_amount_variation_rate")]
    pub rate: f64,
    /// Include US comma format (1,234.56)
    #[serde(default)]
    pub us_comma_format: bool,
    /// Include EU format (1.234,56)
    #[serde(default)]
    pub eu_format: bool,
    /// Include currency prefix ($1,234.56)
    #[serde(default)]
    pub currency_prefix: bool,
    /// Include accounting format with parentheses for negatives
    #[serde(default)]
    pub accounting_format: bool,
}

fn default_amount_variation_rate() -> f64 {
    0.02
}

impl Default for AmountFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_amount_variation_rate(),
            us_comma_format: false,
            eu_format: false,
            currency_prefix: false,
            accounting_format: false,
        }
    }
}

/// Identifier format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifierFormatVariationConfig {
    /// Enable identifier format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_identifier_variation_rate")]
    pub rate: f64,
    /// Case variations (uppercase, lowercase, mixed)
    #[serde(default)]
    pub case_variations: bool,
    /// Padding variations (leading zeros)
    #[serde(default)]
    pub padding_variations: bool,
    /// Separator variations (dash vs underscore)
    #[serde(default)]
    pub separator_variations: bool,
}

fn default_identifier_variation_rate() -> f64 {
    0.02
}

impl Default for IdentifierFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_identifier_variation_rate(),
            case_variations: false,
            padding_variations: false,
            separator_variations: false,
        }
    }
}

/// Duplicate injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSchemaConfig {
    /// Enable duplicate injection
    #[serde(default)]
    pub enabled: bool,
    /// Overall duplicate rate
    #[serde(default = "default_duplicate_rate")]
    pub rate: f64,
    /// Exact duplicate proportion (out of duplicates)
    #[serde(default = "default_exact_duplicate_ratio")]
    pub exact_duplicate_ratio: f64,
    /// Near duplicate proportion (slight variations)
    #[serde(default = "default_near_duplicate_ratio")]
    pub near_duplicate_ratio: f64,
    /// Fuzzy duplicate proportion (typos in key fields)
    #[serde(default = "default_fuzzy_duplicate_ratio")]
    pub fuzzy_duplicate_ratio: f64,
    /// Maximum date offset for near/fuzzy duplicates (days)
    #[serde(default = "default_max_date_offset")]
    pub max_date_offset_days: u32,
    /// Maximum amount variance for near duplicates (fraction)
    #[serde(default = "default_max_amount_variance")]
    pub max_amount_variance: f64,
}

fn default_duplicate_rate() -> f64 {
    0.005
}
fn default_exact_duplicate_ratio() -> f64 {
    0.4
}
fn default_near_duplicate_ratio() -> f64 {
    0.35
}
fn default_fuzzy_duplicate_ratio() -> f64 {
    0.25
}
fn default_max_date_offset() -> u32 {
    3
}
fn default_max_amount_variance() -> f64 {
    0.01
}

impl Default for DuplicateSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_duplicate_rate(),
            exact_duplicate_ratio: default_exact_duplicate_ratio(),
            near_duplicate_ratio: default_near_duplicate_ratio(),
            fuzzy_duplicate_ratio: default_fuzzy_duplicate_ratio(),
            max_date_offset_days: default_max_date_offset(),
            max_amount_variance: default_max_amount_variance(),
        }
    }
}

/// Encoding issue configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingIssueSchemaConfig {
    /// Enable encoding issue injection
    #[serde(default)]
    pub enabled: bool,
    /// Overall encoding issue rate
    #[serde(default = "default_encoding_rate")]
    pub rate: f64,
    /// Include mojibake (UTF-8/Latin-1 confusion)
    #[serde(default)]
    pub mojibake: bool,
    /// Include HTML entity corruption
    #[serde(default)]
    pub html_entities: bool,
    /// Include BOM issues
    #[serde(default)]
    pub bom_issues: bool,
}

fn default_encoding_rate() -> f64 {
    0.001
}

impl Default for EncodingIssueSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_encoding_rate(),
            mojibake: false,
            html_entities: false,
            bom_issues: false,
        }
    }
}

/// Per-sink quality profiles for different output formats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SinkQualityProfiles {
    /// CSV-specific quality settings
    #[serde(default)]
    pub csv: Option<SinkQualityOverride>,
    /// JSON-specific quality settings
    #[serde(default)]
    pub json: Option<SinkQualityOverride>,
    /// Parquet-specific quality settings
    #[serde(default)]
    pub parquet: Option<SinkQualityOverride>,
}

/// Quality setting overrides for a specific sink type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkQualityOverride {
    /// Override enabled state
    pub enabled: Option<bool>,
    /// Override missing value rate
    pub missing_rate: Option<f64>,
    /// Override typo rate
    pub typo_rate: Option<f64>,
    /// Override format variation rate
    pub format_variation_rate: Option<f64>,
    /// Override duplicate rate
    pub duplicate_rate: Option<f64>,
}

// =============================================================================
// Accounting Standards Configuration
// =============================================================================

/// Accounting standards framework configuration for generating standards-compliant data.
///
/// Supports US GAAP and IFRS frameworks with specific standards:
/// - ASC 606/IFRS 15: Revenue Recognition
/// - ASC 842/IFRS 16: Leases
/// - ASC 820/IFRS 13: Fair Value Measurement
/// - ASC 360/IAS 36: Impairment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountingStandardsConfig {
    /// Enable accounting standards generation
    #[serde(default)]
    pub enabled: bool,

    /// Accounting framework to use
    #[serde(default)]
    pub framework: AccountingFrameworkConfig,

    /// Revenue recognition configuration (ASC 606/IFRS 15)
    #[serde(default)]
    pub revenue_recognition: RevenueRecognitionConfig,

    /// Lease accounting configuration (ASC 842/IFRS 16)
    #[serde(default)]
    pub leases: LeaseAccountingConfig,

    /// Fair value measurement configuration (ASC 820/IFRS 13)
    #[serde(default)]
    pub fair_value: FairValueConfig,

    /// Impairment testing configuration (ASC 360/IAS 36)
    #[serde(default)]
    pub impairment: ImpairmentConfig,

    /// Generate framework differences for dual reporting
    #[serde(default)]
    pub generate_differences: bool,
}

/// Accounting framework selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountingFrameworkConfig {
    /// US Generally Accepted Accounting Principles
    #[default]
    UsGaap,
    /// International Financial Reporting Standards
    Ifrs,
    /// Generate data for both frameworks with reconciliation
    DualReporting,
}

/// Revenue recognition configuration (ASC 606/IFRS 15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueRecognitionConfig {
    /// Enable revenue recognition generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate customer contracts
    #[serde(default = "default_true")]
    pub generate_contracts: bool,

    /// Average number of performance obligations per contract
    #[serde(default = "default_avg_obligations")]
    pub avg_obligations_per_contract: f64,

    /// Rate of contracts with variable consideration
    #[serde(default = "default_variable_consideration_rate")]
    pub variable_consideration_rate: f64,

    /// Rate of over-time revenue recognition (vs point-in-time)
    #[serde(default = "default_over_time_rate")]
    pub over_time_recognition_rate: f64,

    /// Number of contracts to generate
    #[serde(default = "default_contract_count")]
    pub contract_count: usize,
}

fn default_avg_obligations() -> f64 {
    2.0
}

fn default_variable_consideration_rate() -> f64 {
    0.15
}

fn default_over_time_rate() -> f64 {
    0.30
}

fn default_contract_count() -> usize {
    100
}

impl Default for RevenueRecognitionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_contracts: true,
            avg_obligations_per_contract: default_avg_obligations(),
            variable_consideration_rate: default_variable_consideration_rate(),
            over_time_recognition_rate: default_over_time_rate(),
            contract_count: default_contract_count(),
        }
    }
}

/// Lease accounting configuration (ASC 842/IFRS 16).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseAccountingConfig {
    /// Enable lease accounting generation
    #[serde(default)]
    pub enabled: bool,

    /// Number of leases to generate
    #[serde(default = "default_lease_count")]
    pub lease_count: usize,

    /// Percentage of finance leases (vs operating)
    #[serde(default = "default_finance_lease_pct")]
    pub finance_lease_percent: f64,

    /// Average lease term in months
    #[serde(default = "default_avg_lease_term")]
    pub avg_lease_term_months: u32,

    /// Generate amortization schedules
    #[serde(default = "default_true")]
    pub generate_amortization: bool,

    /// Real estate lease percentage
    #[serde(default = "default_real_estate_pct")]
    pub real_estate_percent: f64,
}

fn default_lease_count() -> usize {
    50
}

fn default_finance_lease_pct() -> f64 {
    0.30
}

fn default_avg_lease_term() -> u32 {
    60
}

fn default_real_estate_pct() -> f64 {
    0.40
}

impl Default for LeaseAccountingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            lease_count: default_lease_count(),
            finance_lease_percent: default_finance_lease_pct(),
            avg_lease_term_months: default_avg_lease_term(),
            generate_amortization: true,
            real_estate_percent: default_real_estate_pct(),
        }
    }
}

/// Fair value measurement configuration (ASC 820/IFRS 13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueConfig {
    /// Enable fair value measurement generation
    #[serde(default)]
    pub enabled: bool,

    /// Number of fair value measurements to generate
    #[serde(default = "default_fv_count")]
    pub measurement_count: usize,

    /// Level 1 (quoted prices) percentage
    #[serde(default = "default_level1_pct")]
    pub level1_percent: f64,

    /// Level 2 (observable inputs) percentage
    #[serde(default = "default_level2_pct")]
    pub level2_percent: f64,

    /// Level 3 (unobservable inputs) percentage
    #[serde(default = "default_level3_pct")]
    pub level3_percent: f64,

    /// Include sensitivity analysis for Level 3
    #[serde(default)]
    pub include_sensitivity_analysis: bool,
}

fn default_fv_count() -> usize {
    25
}

fn default_level1_pct() -> f64 {
    0.40
}

fn default_level2_pct() -> f64 {
    0.35
}

fn default_level3_pct() -> f64 {
    0.25
}

impl Default for FairValueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            measurement_count: default_fv_count(),
            level1_percent: default_level1_pct(),
            level2_percent: default_level2_pct(),
            level3_percent: default_level3_pct(),
            include_sensitivity_analysis: false,
        }
    }
}

/// Impairment testing configuration (ASC 360/IAS 36).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpairmentConfig {
    /// Enable impairment testing generation
    #[serde(default)]
    pub enabled: bool,

    /// Number of impairment tests to generate
    #[serde(default = "default_impairment_count")]
    pub test_count: usize,

    /// Rate of tests resulting in impairment
    #[serde(default = "default_impairment_rate")]
    pub impairment_rate: f64,

    /// Generate cash flow projections
    #[serde(default = "default_true")]
    pub generate_projections: bool,

    /// Include goodwill impairment tests
    #[serde(default)]
    pub include_goodwill: bool,
}

fn default_impairment_count() -> usize {
    15
}

fn default_impairment_rate() -> f64 {
    0.10
}

impl Default for ImpairmentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            test_count: default_impairment_count(),
            impairment_rate: default_impairment_rate(),
            generate_projections: true,
            include_goodwill: false,
        }
    }
}

// =============================================================================
// Audit Standards Configuration
// =============================================================================

/// Audit standards framework configuration for generating standards-compliant audit data.
///
/// Supports ISA (International Standards on Auditing) and PCAOB standards:
/// - ISA 200-720: Complete coverage of audit standards
/// - ISA 520: Analytical Procedures
/// - ISA 505: External Confirmations
/// - ISA 700/705/706/701: Audit Reports
/// - PCAOB AS 2201: ICFR Auditing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditStandardsConfig {
    /// Enable audit standards generation
    #[serde(default)]
    pub enabled: bool,

    /// ISA compliance configuration
    #[serde(default)]
    pub isa_compliance: IsaComplianceConfig,

    /// Analytical procedures configuration (ISA 520)
    #[serde(default)]
    pub analytical_procedures: AnalyticalProceduresConfig,

    /// External confirmations configuration (ISA 505)
    #[serde(default)]
    pub confirmations: ConfirmationsConfig,

    /// Audit opinion configuration (ISA 700/705/706/701)
    #[serde(default)]
    pub opinion: AuditOpinionConfig,

    /// Generate complete audit trail with traceability
    #[serde(default)]
    pub generate_audit_trail: bool,

    /// SOX 302/404 compliance configuration
    #[serde(default)]
    pub sox: SoxComplianceConfig,

    /// PCAOB-specific configuration
    #[serde(default)]
    pub pcaob: PcaobConfig,
}

/// ISA compliance level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaComplianceConfig {
    /// Enable ISA compliance tracking
    #[serde(default)]
    pub enabled: bool,

    /// Compliance level: "basic", "standard", "comprehensive"
    #[serde(default = "default_compliance_level")]
    pub compliance_level: String,

    /// Generate ISA requirement mappings
    #[serde(default = "default_true")]
    pub generate_isa_mappings: bool,

    /// Generate ISA coverage summary
    #[serde(default = "default_true")]
    pub generate_coverage_summary: bool,

    /// Include PCAOB standard mappings (for dual framework)
    #[serde(default)]
    pub include_pcaob: bool,

    /// Framework to use: "isa", "pcaob", "dual"
    #[serde(default = "default_audit_framework")]
    pub framework: String,
}

fn default_compliance_level() -> String {
    "standard".to_string()
}

fn default_audit_framework() -> String {
    "isa".to_string()
}

impl Default for IsaComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            compliance_level: default_compliance_level(),
            generate_isa_mappings: true,
            generate_coverage_summary: true,
            include_pcaob: false,
            framework: default_audit_framework(),
        }
    }
}

/// Analytical procedures configuration (ISA 520).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalProceduresConfig {
    /// Enable analytical procedures generation
    #[serde(default)]
    pub enabled: bool,

    /// Number of procedures per account/area
    #[serde(default = "default_procedures_per_account")]
    pub procedures_per_account: usize,

    /// Probability of variance exceeding threshold
    #[serde(default = "default_variance_probability")]
    pub variance_probability: f64,

    /// Include variance investigations
    #[serde(default = "default_true")]
    pub generate_investigations: bool,

    /// Include financial ratio analysis
    #[serde(default = "default_true")]
    pub include_ratio_analysis: bool,
}

fn default_procedures_per_account() -> usize {
    3
}

fn default_variance_probability() -> f64 {
    0.20
}

impl Default for AnalyticalProceduresConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            procedures_per_account: default_procedures_per_account(),
            variance_probability: default_variance_probability(),
            generate_investigations: true,
            include_ratio_analysis: true,
        }
    }
}

/// External confirmations configuration (ISA 505).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationsConfig {
    /// Enable confirmation generation
    #[serde(default)]
    pub enabled: bool,

    /// Number of confirmations to generate
    #[serde(default = "default_confirmation_count")]
    pub confirmation_count: usize,

    /// Positive response rate
    #[serde(default = "default_positive_response_rate")]
    pub positive_response_rate: f64,

    /// Exception rate (responses with differences)
    #[serde(default = "default_exception_rate_confirm")]
    pub exception_rate: f64,

    /// Non-response rate
    #[serde(default = "default_non_response_rate")]
    pub non_response_rate: f64,

    /// Generate alternative procedures for non-responses
    #[serde(default = "default_true")]
    pub generate_alternative_procedures: bool,
}

fn default_confirmation_count() -> usize {
    50
}

fn default_positive_response_rate() -> f64 {
    0.85
}

fn default_exception_rate_confirm() -> f64 {
    0.10
}

fn default_non_response_rate() -> f64 {
    0.05
}

impl Default for ConfirmationsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            confirmation_count: default_confirmation_count(),
            positive_response_rate: default_positive_response_rate(),
            exception_rate: default_exception_rate_confirm(),
            non_response_rate: default_non_response_rate(),
            generate_alternative_procedures: true,
        }
    }
}

/// Audit opinion configuration (ISA 700/705/706/701).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOpinionConfig {
    /// Enable audit opinion generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate Key Audit Matters (KAM) / Critical Audit Matters (CAM)
    #[serde(default = "default_true")]
    pub generate_kam: bool,

    /// Average number of KAMs/CAMs per opinion
    #[serde(default = "default_kam_count")]
    pub average_kam_count: usize,

    /// Rate of modified opinions
    #[serde(default = "default_modified_opinion_rate")]
    pub modified_opinion_rate: f64,

    /// Include emphasis of matter paragraphs
    #[serde(default)]
    pub include_emphasis_of_matter: bool,

    /// Include going concern conclusions
    #[serde(default = "default_true")]
    pub include_going_concern: bool,
}

fn default_kam_count() -> usize {
    3
}

fn default_modified_opinion_rate() -> f64 {
    0.05
}

impl Default for AuditOpinionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_kam: true,
            average_kam_count: default_kam_count(),
            modified_opinion_rate: default_modified_opinion_rate(),
            include_emphasis_of_matter: false,
            include_going_concern: true,
        }
    }
}

/// SOX compliance configuration (Sections 302/404).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoxComplianceConfig {
    /// Enable SOX compliance generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate Section 302 CEO/CFO certifications
    #[serde(default = "default_true")]
    pub generate_302_certifications: bool,

    /// Generate Section 404 ICFR assessments
    #[serde(default = "default_true")]
    pub generate_404_assessments: bool,

    /// Materiality threshold for SOX testing
    #[serde(default = "default_sox_materiality_threshold")]
    pub materiality_threshold: f64,

    /// Rate of material weaknesses
    #[serde(default = "default_material_weakness_rate")]
    pub material_weakness_rate: f64,

    /// Rate of significant deficiencies
    #[serde(default = "default_significant_deficiency_rate")]
    pub significant_deficiency_rate: f64,
}

fn default_material_weakness_rate() -> f64 {
    0.02
}

fn default_significant_deficiency_rate() -> f64 {
    0.08
}

impl Default for SoxComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_302_certifications: true,
            generate_404_assessments: true,
            materiality_threshold: default_sox_materiality_threshold(),
            material_weakness_rate: default_material_weakness_rate(),
            significant_deficiency_rate: default_significant_deficiency_rate(),
        }
    }
}

/// PCAOB-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcaobConfig {
    /// Enable PCAOB-specific elements
    #[serde(default)]
    pub enabled: bool,

    /// Treat as PCAOB audit (vs ISA-only)
    #[serde(default)]
    pub is_pcaob_audit: bool,

    /// Generate Critical Audit Matters (CAM)
    #[serde(default = "default_true")]
    pub generate_cam: bool,

    /// Include ICFR opinion (for integrated audits)
    #[serde(default)]
    pub include_icfr_opinion: bool,

    /// Generate PCAOB-ISA standard mappings
    #[serde(default)]
    pub generate_standard_mappings: bool,
}

impl Default for PcaobConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            is_pcaob_audit: false,
            generate_cam: true,
            include_icfr_opinion: false,
            generate_standard_mappings: false,
        }
    }
}

// =============================================================================
// Advanced Distribution Configuration
// =============================================================================

/// Advanced distribution configuration for realistic data generation.
///
/// This section enables sophisticated distribution models including:
/// - Mixture models (multi-modal distributions)
/// - Cross-field correlations
/// - Conditional distributions
/// - Regime changes and economic cycles
/// - Statistical validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvancedDistributionConfig {
    /// Enable advanced distribution features.
    #[serde(default)]
    pub enabled: bool,

    /// Mixture model configuration for amounts.
    #[serde(default)]
    pub amounts: MixtureDistributionSchemaConfig,

    /// Cross-field correlation configuration.
    #[serde(default)]
    pub correlations: CorrelationSchemaConfig,

    /// Conditional distribution configurations.
    #[serde(default)]
    pub conditional: Vec<ConditionalDistributionSchemaConfig>,

    /// Regime change configuration.
    #[serde(default)]
    pub regime_changes: RegimeChangeSchemaConfig,

    /// Industry-specific distribution profile.
    #[serde(default)]
    pub industry_profile: Option<IndustryProfileType>,

    /// Statistical validation configuration.
    #[serde(default)]
    pub validation: StatisticalValidationSchemaConfig,
}

/// Industry profile types for pre-configured distribution settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndustryProfileType {
    /// Retail industry profile (POS sales, inventory, seasonal)
    Retail,
    /// Manufacturing industry profile (raw materials, maintenance, capital)
    Manufacturing,
    /// Financial services profile (wire transfers, ACH, fee income)
    FinancialServices,
    /// Healthcare profile (claims, procedures, supplies)
    Healthcare,
    /// Technology profile (subscriptions, services, R&D)
    Technology,
}

/// Mixture model distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureDistributionSchemaConfig {
    /// Enable mixture model for amount generation.
    #[serde(default)]
    pub enabled: bool,

    /// Distribution type: "gaussian" or "lognormal".
    #[serde(default = "default_mixture_type")]
    pub distribution_type: MixtureDistributionType,

    /// Mixture components with weights.
    #[serde(default)]
    pub components: Vec<MixtureComponentConfig>,

    /// Minimum value constraint.
    #[serde(default = "default_min_amount")]
    pub min_value: f64,

    /// Maximum value constraint (optional).
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Decimal places for rounding.
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

fn default_mixture_type() -> MixtureDistributionType {
    MixtureDistributionType::LogNormal
}

fn default_min_amount() -> f64 {
    0.01
}

fn default_decimal_places() -> u8 {
    2
}

impl Default for MixtureDistributionSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            distribution_type: MixtureDistributionType::LogNormal,
            components: Vec::new(),
            min_value: 0.01,
            max_value: None,
            decimal_places: 2,
        }
    }
}

/// Mixture distribution type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MixtureDistributionType {
    /// Gaussian (normal) mixture
    Gaussian,
    /// Log-normal mixture (for positive amounts)
    #[default]
    LogNormal,
}

/// Configuration for a single mixture component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureComponentConfig {
    /// Weight of this component (must sum to 1.0 across all components).
    pub weight: f64,

    /// Location parameter (mean for Gaussian, mu for log-normal).
    pub mu: f64,

    /// Scale parameter (std dev for Gaussian, sigma for log-normal).
    pub sigma: f64,

    /// Optional label for this component (e.g., "routine", "significant", "major").
    #[serde(default)]
    pub label: Option<String>,
}

/// Cross-field correlation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationSchemaConfig {
    /// Enable correlation modeling.
    #[serde(default)]
    pub enabled: bool,

    /// Copula type for dependency modeling.
    #[serde(default)]
    pub copula_type: CopulaSchemaType,

    /// Field definitions for correlation.
    #[serde(default)]
    pub fields: Vec<CorrelatedFieldConfig>,

    /// Correlation matrix (upper triangular, row-major).
    /// For n fields, this should have n*(n-1)/2 values.
    #[serde(default)]
    pub matrix: Vec<f64>,

    /// Expected correlations for validation.
    #[serde(default)]
    pub expected_correlations: Vec<ExpectedCorrelationConfig>,
}

impl Default for CorrelationSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            copula_type: CopulaSchemaType::Gaussian,
            fields: Vec::new(),
            matrix: Vec::new(),
            expected_correlations: Vec::new(),
        }
    }
}

/// Copula type for dependency modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CopulaSchemaType {
    /// Gaussian copula (symmetric, no tail dependence)
    #[default]
    Gaussian,
    /// Clayton copula (lower tail dependence)
    Clayton,
    /// Gumbel copula (upper tail dependence)
    Gumbel,
    /// Frank copula (symmetric, no tail dependence)
    Frank,
    /// Student-t copula (both tail dependencies)
    StudentT,
}

/// Configuration for a correlated field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedFieldConfig {
    /// Field name.
    pub name: String,

    /// Marginal distribution type.
    #[serde(default)]
    pub distribution: MarginalDistributionConfig,
}

/// Marginal distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarginalDistributionConfig {
    /// Normal distribution.
    Normal {
        /// Mean
        mu: f64,
        /// Standard deviation
        sigma: f64,
    },
    /// Log-normal distribution.
    LogNormal {
        /// Location parameter
        mu: f64,
        /// Scale parameter
        sigma: f64,
    },
    /// Uniform distribution.
    Uniform {
        /// Minimum value
        min: f64,
        /// Maximum value
        max: f64,
    },
    /// Discrete uniform distribution.
    DiscreteUniform {
        /// Minimum integer value
        min: i32,
        /// Maximum integer value
        max: i32,
    },
}

impl Default for MarginalDistributionConfig {
    fn default() -> Self {
        Self::Normal {
            mu: 0.0,
            sigma: 1.0,
        }
    }
}

/// Expected correlation for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedCorrelationConfig {
    /// First field name.
    pub field1: String,
    /// Second field name.
    pub field2: String,
    /// Expected correlation coefficient.
    pub expected_r: f64,
    /// Acceptable tolerance.
    #[serde(default = "default_correlation_tolerance")]
    pub tolerance: f64,
}

fn default_correlation_tolerance() -> f64 {
    0.10
}

/// Conditional distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalDistributionSchemaConfig {
    /// Output field name to generate.
    pub output_field: String,

    /// Input field name that conditions the distribution.
    pub input_field: String,

    /// Breakpoints defining distribution changes.
    #[serde(default)]
    pub breakpoints: Vec<ConditionalBreakpointConfig>,

    /// Default distribution when below all breakpoints.
    #[serde(default)]
    pub default_distribution: ConditionalDistributionParamsConfig,

    /// Minimum output value constraint.
    #[serde(default)]
    pub min_value: Option<f64>,

    /// Maximum output value constraint.
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Decimal places for output rounding.
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u8,
}

/// Breakpoint for conditional distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBreakpointConfig {
    /// Input value threshold.
    pub threshold: f64,

    /// Distribution to use when input >= threshold.
    pub distribution: ConditionalDistributionParamsConfig,
}

/// Distribution parameters for conditional distributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionalDistributionParamsConfig {
    /// Fixed value.
    Fixed {
        /// The fixed value
        value: f64,
    },
    /// Normal distribution.
    Normal {
        /// Mean
        mu: f64,
        /// Standard deviation
        sigma: f64,
    },
    /// Log-normal distribution.
    LogNormal {
        /// Location parameter
        mu: f64,
        /// Scale parameter
        sigma: f64,
    },
    /// Uniform distribution.
    Uniform {
        /// Minimum
        min: f64,
        /// Maximum
        max: f64,
    },
    /// Beta distribution (scaled).
    Beta {
        /// Alpha parameter
        alpha: f64,
        /// Beta parameter
        beta: f64,
        /// Minimum output value
        min: f64,
        /// Maximum output value
        max: f64,
    },
    /// Discrete values with weights.
    Discrete {
        /// Possible values
        values: Vec<f64>,
        /// Weights (should sum to 1.0)
        weights: Vec<f64>,
    },
}

impl Default for ConditionalDistributionParamsConfig {
    fn default() -> Self {
        Self::Normal {
            mu: 0.0,
            sigma: 1.0,
        }
    }
}

/// Regime change configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegimeChangeSchemaConfig {
    /// Enable regime change modeling.
    #[serde(default)]
    pub enabled: bool,

    /// List of regime changes.
    #[serde(default)]
    pub changes: Vec<RegimeChangeEventConfig>,

    /// Economic cycle configuration.
    #[serde(default)]
    pub economic_cycle: Option<EconomicCycleSchemaConfig>,

    /// Parameter drift configurations.
    #[serde(default)]
    pub parameter_drifts: Vec<ParameterDriftSchemaConfig>,
}

/// A single regime change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeChangeEventConfig {
    /// Date when the change occurs (ISO 8601 format).
    pub date: String,

    /// Type of regime change.
    pub change_type: RegimeChangeTypeConfig,

    /// Description of the change.
    #[serde(default)]
    pub description: Option<String>,

    /// Effects of this regime change.
    #[serde(default)]
    pub effects: Vec<RegimeEffectConfig>,
}

/// Type of regime change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegimeChangeTypeConfig {
    /// Acquisition - sudden volume and amount increase
    Acquisition,
    /// Divestiture - sudden volume and amount decrease
    Divestiture,
    /// Price increase - amounts increase
    PriceIncrease,
    /// Price decrease - amounts decrease
    PriceDecrease,
    /// New product launch - volume ramp-up
    ProductLaunch,
    /// Product discontinuation - volume ramp-down
    ProductDiscontinuation,
    /// Policy change - affects patterns
    PolicyChange,
    /// Competitor entry - market disruption
    CompetitorEntry,
    /// Custom effect
    Custom,
}

/// Effect of a regime change on a specific field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeEffectConfig {
    /// Field being affected.
    pub field: String,

    /// Multiplier to apply (1.0 = no change, 1.5 = 50% increase).
    pub multiplier: f64,
}

/// Economic cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicCycleSchemaConfig {
    /// Enable economic cycle modeling.
    #[serde(default)]
    pub enabled: bool,

    /// Cycle period in months (e.g., 48 for 4-year business cycle).
    #[serde(default = "default_cycle_period")]
    pub period_months: u32,

    /// Amplitude of cycle effect (0.0-1.0).
    #[serde(default = "default_cycle_amplitude")]
    pub amplitude: f64,

    /// Phase offset in months.
    #[serde(default)]
    pub phase_offset: u32,

    /// Recession periods (start_month, duration_months).
    #[serde(default)]
    pub recessions: Vec<RecessionPeriodConfig>,
}

fn default_cycle_period() -> u32 {
    48
}

fn default_cycle_amplitude() -> f64 {
    0.15
}

impl Default for EconomicCycleSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            period_months: 48,
            amplitude: 0.15,
            phase_offset: 0,
            recessions: Vec::new(),
        }
    }
}

/// Recession period configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecessionPeriodConfig {
    /// Start month (0-indexed from generation start).
    pub start_month: u32,

    /// Duration in months.
    pub duration_months: u32,

    /// Severity (0.0-1.0, affects volume reduction).
    #[serde(default = "default_recession_severity")]
    pub severity: f64,
}

fn default_recession_severity() -> f64 {
    0.20
}

/// Parameter drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDriftSchemaConfig {
    /// Parameter being drifted.
    pub parameter: String,

    /// Drift type.
    pub drift_type: ParameterDriftTypeConfig,

    /// Start value.
    pub start_value: f64,

    /// End value.
    pub end_value: f64,

    /// Start period (month, 0-indexed).
    #[serde(default)]
    pub start_period: u32,

    /// End period (month, optional - defaults to end of generation).
    #[serde(default)]
    pub end_period: Option<u32>,
}

/// Parameter drift type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDriftTypeConfig {
    /// Linear interpolation
    #[default]
    Linear,
    /// Exponential growth/decay
    Exponential,
    /// S-curve (logistic)
    Logistic,
    /// Step function
    Step,
}

/// Statistical validation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatisticalValidationSchemaConfig {
    /// Enable statistical validation.
    #[serde(default)]
    pub enabled: bool,

    /// Statistical tests to run.
    #[serde(default)]
    pub tests: Vec<StatisticalTestConfig>,

    /// Validation reporting configuration.
    #[serde(default)]
    pub reporting: ValidationReportingConfig,
}

/// Statistical test configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StatisticalTestConfig {
    /// Benford's Law first digit test.
    BenfordFirstDigit {
        /// Threshold MAD for failure.
        #[serde(default = "default_benford_threshold")]
        threshold_mad: f64,
        /// Warning MAD threshold.
        #[serde(default = "default_benford_warning")]
        warning_mad: f64,
    },
    /// Distribution fit test.
    DistributionFit {
        /// Target distribution to test.
        target: TargetDistributionConfig,
        /// K-S test significance level.
        #[serde(default = "default_ks_significance")]
        ks_significance: f64,
        /// Test method (ks, anderson_darling, chi_squared).
        #[serde(default)]
        method: DistributionFitMethod,
    },
    /// Correlation check.
    CorrelationCheck {
        /// Expected correlations to validate.
        expected_correlations: Vec<ExpectedCorrelationConfig>,
    },
    /// Chi-squared test.
    ChiSquared {
        /// Number of bins.
        #[serde(default = "default_chi_squared_bins")]
        bins: usize,
        /// Significance level.
        #[serde(default = "default_chi_squared_significance")]
        significance: f64,
    },
    /// Anderson-Darling test.
    AndersonDarling {
        /// Target distribution.
        target: TargetDistributionConfig,
        /// Significance level.
        #[serde(default = "default_ad_significance")]
        significance: f64,
    },
}

fn default_benford_threshold() -> f64 {
    0.015
}

fn default_benford_warning() -> f64 {
    0.010
}

fn default_ks_significance() -> f64 {
    0.05
}

fn default_chi_squared_bins() -> usize {
    10
}

fn default_chi_squared_significance() -> f64 {
    0.05
}

fn default_ad_significance() -> f64 {
    0.05
}

/// Target distribution for fit tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetDistributionConfig {
    /// Normal distribution
    Normal,
    /// Log-normal distribution
    #[default]
    LogNormal,
    /// Exponential distribution
    Exponential,
    /// Uniform distribution
    Uniform,
}

/// Distribution fit test method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DistributionFitMethod {
    /// Kolmogorov-Smirnov test
    #[default]
    KolmogorovSmirnov,
    /// Anderson-Darling test
    AndersonDarling,
    /// Chi-squared test
    ChiSquared,
}

/// Validation reporting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReportingConfig {
    /// Output validation report to file.
    #[serde(default)]
    pub output_report: bool,

    /// Report format.
    #[serde(default)]
    pub format: ValidationReportFormat,

    /// Fail generation if validation fails.
    #[serde(default)]
    pub fail_on_error: bool,

    /// Include detailed statistics in report.
    #[serde(default = "default_true")]
    pub include_details: bool,
}

impl Default for ValidationReportingConfig {
    fn default() -> Self {
        Self {
            output_report: false,
            format: ValidationReportFormat::Json,
            fail_on_error: false,
            include_details: true,
        }
    }
}

/// Validation report format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValidationReportFormat {
    /// JSON format
    #[default]
    Json,
    /// YAML format
    Yaml,
    /// HTML report
    Html,
}

// =============================================================================
// Temporal Patterns Configuration
// =============================================================================

/// Temporal patterns configuration for business days, period-end dynamics, and processing lags.
///
/// This section enables sophisticated temporal modeling including:
/// - Business day calculations and settlement dates
/// - Regional holiday calendars
/// - Period-end decay curves (non-flat volume spikes)
/// - Processing lag modeling (event-to-posting delays)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalPatternsConfig {
    /// Enable temporal patterns features.
    #[serde(default)]
    pub enabled: bool,

    /// Business day calculation configuration.
    #[serde(default)]
    pub business_days: BusinessDaySchemaConfig,

    /// Regional calendar configuration.
    #[serde(default)]
    pub calendars: CalendarSchemaConfig,

    /// Period-end dynamics configuration.
    #[serde(default)]
    pub period_end: PeriodEndSchemaConfig,

    /// Processing lag configuration.
    #[serde(default)]
    pub processing_lags: ProcessingLagSchemaConfig,

    /// Fiscal calendar configuration (custom year start, 4-4-5, 13-period).
    #[serde(default)]
    pub fiscal_calendar: FiscalCalendarSchemaConfig,

    /// Intra-day patterns configuration (morning spike, lunch dip, EOD rush).
    #[serde(default)]
    pub intraday: IntraDaySchemaConfig,

    /// Timezone handling configuration.
    #[serde(default)]
    pub timezones: TimezoneSchemaConfig,
}

/// Business day calculation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDaySchemaConfig {
    /// Enable business day calculations.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Half-day policy: "full_day", "half_day", "non_business_day".
    #[serde(default = "default_half_day_policy")]
    pub half_day_policy: String,

    /// Settlement rules configuration.
    #[serde(default)]
    pub settlement_rules: SettlementRulesSchemaConfig,

    /// Month-end convention: "modified_following", "preceding", "following", "end_of_month".
    #[serde(default = "default_month_end_convention")]
    pub month_end_convention: String,

    /// Weekend days (e.g., ["saturday", "sunday"] or ["friday", "saturday"] for Middle East).
    #[serde(default)]
    pub weekend_days: Option<Vec<String>>,
}

fn default_half_day_policy() -> String {
    "half_day".to_string()
}

fn default_month_end_convention() -> String {
    "modified_following".to_string()
}

impl Default for BusinessDaySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            half_day_policy: "half_day".to_string(),
            settlement_rules: SettlementRulesSchemaConfig::default(),
            month_end_convention: "modified_following".to_string(),
            weekend_days: None,
        }
    }
}

/// Settlement rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRulesSchemaConfig {
    /// Equity settlement days (T+N).
    #[serde(default = "default_settlement_2")]
    pub equity_days: i32,

    /// Government bonds settlement days.
    #[serde(default = "default_settlement_1")]
    pub government_bonds_days: i32,

    /// FX spot settlement days.
    #[serde(default = "default_settlement_2")]
    pub fx_spot_days: i32,

    /// Corporate bonds settlement days.
    #[serde(default = "default_settlement_2")]
    pub corporate_bonds_days: i32,

    /// Wire transfer cutoff time (HH:MM format).
    #[serde(default = "default_wire_cutoff")]
    pub wire_cutoff_time: String,

    /// International wire settlement days.
    #[serde(default = "default_settlement_1")]
    pub wire_international_days: i32,

    /// ACH settlement days.
    #[serde(default = "default_settlement_1")]
    pub ach_days: i32,
}

fn default_settlement_1() -> i32 {
    1
}

fn default_settlement_2() -> i32 {
    2
}

fn default_wire_cutoff() -> String {
    "14:00".to_string()
}

impl Default for SettlementRulesSchemaConfig {
    fn default() -> Self {
        Self {
            equity_days: 2,
            government_bonds_days: 1,
            fx_spot_days: 2,
            corporate_bonds_days: 2,
            wire_cutoff_time: "14:00".to_string(),
            wire_international_days: 1,
            ach_days: 1,
        }
    }
}

/// Regional calendar configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarSchemaConfig {
    /// List of regions to include (e.g., ["US", "DE", "BR", "SG", "KR"]).
    #[serde(default)]
    pub regions: Vec<String>,

    /// Custom holidays (in addition to regional calendars).
    #[serde(default)]
    pub custom_holidays: Vec<CustomHolidaySchemaConfig>,
}

/// Custom holiday configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHolidaySchemaConfig {
    /// Holiday name.
    pub name: String,
    /// Month (1-12).
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Activity multiplier (0.0-1.0, default 0.05).
    #[serde(default = "default_holiday_multiplier")]
    pub activity_multiplier: f64,
}

fn default_holiday_multiplier() -> f64 {
    0.05
}

/// Period-end dynamics configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeriodEndSchemaConfig {
    /// Model type: "flat", "exponential", "extended_crunch", "daily_profile".
    #[serde(default)]
    pub model: Option<String>,

    /// Month-end configuration.
    #[serde(default)]
    pub month_end: Option<PeriodEndModelSchemaConfig>,

    /// Quarter-end configuration.
    #[serde(default)]
    pub quarter_end: Option<PeriodEndModelSchemaConfig>,

    /// Year-end configuration.
    #[serde(default)]
    pub year_end: Option<PeriodEndModelSchemaConfig>,
}

/// Period-end model configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeriodEndModelSchemaConfig {
    /// Inherit configuration from another period (e.g., "month_end").
    #[serde(default)]
    pub inherit_from: Option<String>,

    /// Additional multiplier on top of inherited/base model.
    #[serde(default)]
    pub additional_multiplier: Option<f64>,

    /// Days before period end to start acceleration (negative, e.g., -10).
    #[serde(default)]
    pub start_day: Option<i32>,

    /// Base multiplier at start of acceleration.
    #[serde(default)]
    pub base_multiplier: Option<f64>,

    /// Peak multiplier on last day.
    #[serde(default)]
    pub peak_multiplier: Option<f64>,

    /// Decay rate for exponential model (0.1-0.5 typical).
    #[serde(default)]
    pub decay_rate: Option<f64>,

    /// Sustained high days for crunch model.
    #[serde(default)]
    pub sustained_high_days: Option<i32>,
}

/// Processing lag configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingLagSchemaConfig {
    /// Enable processing lag calculations.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Sales order lag configuration (log-normal mu, sigma).
    #[serde(default)]
    pub sales_order_lag: Option<LagDistributionSchemaConfig>,

    /// Purchase order lag configuration.
    #[serde(default)]
    pub purchase_order_lag: Option<LagDistributionSchemaConfig>,

    /// Goods receipt lag configuration.
    #[serde(default)]
    pub goods_receipt_lag: Option<LagDistributionSchemaConfig>,

    /// Invoice receipt lag configuration.
    #[serde(default)]
    pub invoice_receipt_lag: Option<LagDistributionSchemaConfig>,

    /// Invoice issue lag configuration.
    #[serde(default)]
    pub invoice_issue_lag: Option<LagDistributionSchemaConfig>,

    /// Payment lag configuration.
    #[serde(default)]
    pub payment_lag: Option<LagDistributionSchemaConfig>,

    /// Journal entry lag configuration.
    #[serde(default)]
    pub journal_entry_lag: Option<LagDistributionSchemaConfig>,

    /// Cross-day posting configuration.
    #[serde(default)]
    pub cross_day_posting: Option<CrossDayPostingSchemaConfig>,
}

impl Default for ProcessingLagSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sales_order_lag: None,
            purchase_order_lag: None,
            goods_receipt_lag: None,
            invoice_receipt_lag: None,
            invoice_issue_lag: None,
            payment_lag: None,
            journal_entry_lag: None,
            cross_day_posting: None,
        }
    }
}

/// Lag distribution configuration (log-normal parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagDistributionSchemaConfig {
    /// Log-scale mean (mu for log-normal).
    pub mu: f64,
    /// Log-scale standard deviation (sigma for log-normal).
    pub sigma: f64,
    /// Minimum lag in hours.
    #[serde(default)]
    pub min_hours: Option<f64>,
    /// Maximum lag in hours.
    #[serde(default)]
    pub max_hours: Option<f64>,
}

/// Cross-day posting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDayPostingSchemaConfig {
    /// Enable cross-day posting logic.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Probability of next-day posting by hour (map of hour -> probability).
    /// E.g., { 17: 0.7, 19: 0.9, 21: 0.99 }
    #[serde(default)]
    pub probability_by_hour: std::collections::HashMap<u8, f64>,
}

impl Default for CrossDayPostingSchemaConfig {
    fn default() -> Self {
        let mut probability_by_hour = std::collections::HashMap::new();
        probability_by_hour.insert(17, 0.3);
        probability_by_hour.insert(18, 0.6);
        probability_by_hour.insert(19, 0.8);
        probability_by_hour.insert(20, 0.9);
        probability_by_hour.insert(21, 0.95);
        probability_by_hour.insert(22, 0.99);

        Self {
            enabled: true,
            probability_by_hour,
        }
    }
}

// =============================================================================
// Fiscal Calendar Configuration (P2)
// =============================================================================

/// Fiscal calendar configuration.
///
/// Supports calendar year, custom year start, 4-4-5 retail calendar,
/// and 13-period calendars.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiscalCalendarSchemaConfig {
    /// Enable non-standard fiscal calendar.
    #[serde(default)]
    pub enabled: bool,

    /// Fiscal calendar type: "calendar_year", "custom", "four_four_five", "thirteen_period".
    #[serde(default = "default_fiscal_calendar_type")]
    pub calendar_type: String,

    /// Month the fiscal year starts (1-12). Used for custom year start.
    #[serde(default)]
    pub year_start_month: Option<u8>,

    /// Day the fiscal year starts (1-31). Used for custom year start.
    #[serde(default)]
    pub year_start_day: Option<u8>,

    /// 4-4-5 calendar configuration (if calendar_type is "four_four_five").
    #[serde(default)]
    pub four_four_five: Option<FourFourFiveSchemaConfig>,
}

fn default_fiscal_calendar_type() -> String {
    "calendar_year".to_string()
}

/// 4-4-5 retail calendar configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FourFourFiveSchemaConfig {
    /// Week pattern: "four_four_five", "four_five_four", "five_four_four".
    #[serde(default = "default_week_pattern")]
    pub pattern: String,

    /// Anchor type: "first_sunday", "last_saturday", "nearest_saturday".
    #[serde(default = "default_anchor_type")]
    pub anchor_type: String,

    /// Anchor month (1-12).
    #[serde(default = "default_anchor_month")]
    pub anchor_month: u8,

    /// Where to place leap week: "q4_period3" or "q1_period1".
    #[serde(default = "default_leap_week_placement")]
    pub leap_week_placement: String,
}

fn default_week_pattern() -> String {
    "four_four_five".to_string()
}

fn default_anchor_type() -> String {
    "last_saturday".to_string()
}

fn default_anchor_month() -> u8 {
    1 // January
}

fn default_leap_week_placement() -> String {
    "q4_period3".to_string()
}

impl Default for FourFourFiveSchemaConfig {
    fn default() -> Self {
        Self {
            pattern: "four_four_five".to_string(),
            anchor_type: "last_saturday".to_string(),
            anchor_month: 1,
            leap_week_placement: "q4_period3".to_string(),
        }
    }
}

// =============================================================================
// Intra-Day Patterns Configuration (P2)
// =============================================================================

/// Intra-day patterns configuration.
///
/// Defines time-of-day segments with different activity multipliers
/// for realistic modeling of morning spikes, lunch dips, and end-of-day rushes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntraDaySchemaConfig {
    /// Enable intra-day patterns.
    #[serde(default)]
    pub enabled: bool,

    /// Custom intra-day segments.
    #[serde(default)]
    pub segments: Vec<IntraDaySegmentSchemaConfig>,
}

/// Intra-day segment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntraDaySegmentSchemaConfig {
    /// Name of the segment (e.g., "morning_spike", "lunch_dip").
    pub name: String,

    /// Start time (HH:MM format).
    pub start: String,

    /// End time (HH:MM format).
    pub end: String,

    /// Activity multiplier (1.0 = normal).
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,

    /// Posting type: "human", "system", "both".
    #[serde(default = "default_posting_type")]
    pub posting_type: String,
}

fn default_multiplier() -> f64 {
    1.0
}

fn default_posting_type() -> String {
    "both".to_string()
}

// =============================================================================
// Timezone Configuration
// =============================================================================

/// Timezone handling configuration for multi-region entities.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimezoneSchemaConfig {
    /// Enable timezone handling.
    #[serde(default)]
    pub enabled: bool,

    /// Default timezone (IANA format, e.g., "America/New_York").
    #[serde(default = "default_timezone")]
    pub default_timezone: String,

    /// Consolidation timezone for group reporting (IANA format).
    #[serde(default = "default_consolidation_timezone")]
    pub consolidation_timezone: String,

    /// Entity-to-timezone mappings.
    /// Supports patterns like "EU_*" -> "Europe/London".
    #[serde(default)]
    pub entity_mappings: Vec<EntityTimezoneMapping>,
}

fn default_timezone() -> String {
    "America/New_York".to_string()
}

fn default_consolidation_timezone() -> String {
    "UTC".to_string()
}

/// Mapping from entity pattern to timezone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTimezoneMapping {
    /// Entity code pattern (e.g., "EU_*", "*_APAC", "1000").
    pub pattern: String,

    /// Timezone (IANA format, e.g., "Europe/London").
    pub timezone: String,
}

// =============================================================================
// Vendor Network Configuration
// =============================================================================

/// Configuration for multi-tier vendor network generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorNetworkSchemaConfig {
    /// Enable vendor network generation.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum depth of supply chain tiers (1-3).
    #[serde(default = "default_vendor_tier_depth")]
    pub depth: u8,

    /// Tier 1 vendor count configuration.
    #[serde(default)]
    pub tier1: TierCountSchemaConfig,

    /// Tier 2 vendors per Tier 1 parent.
    #[serde(default)]
    pub tier2_per_parent: TierCountSchemaConfig,

    /// Tier 3 vendors per Tier 2 parent.
    #[serde(default)]
    pub tier3_per_parent: TierCountSchemaConfig,

    /// Vendor cluster distribution.
    #[serde(default)]
    pub clusters: VendorClusterSchemaConfig,

    /// Concentration limits.
    #[serde(default)]
    pub dependencies: DependencySchemaConfig,
}

fn default_vendor_tier_depth() -> u8 {
    3
}

impl Default for VendorNetworkSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 3,
            tier1: TierCountSchemaConfig { min: 50, max: 100 },
            tier2_per_parent: TierCountSchemaConfig { min: 4, max: 10 },
            tier3_per_parent: TierCountSchemaConfig { min: 2, max: 5 },
            clusters: VendorClusterSchemaConfig::default(),
            dependencies: DependencySchemaConfig::default(),
        }
    }
}

/// Tier count configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCountSchemaConfig {
    /// Minimum count.
    #[serde(default = "default_tier_min")]
    pub min: usize,

    /// Maximum count.
    #[serde(default = "default_tier_max")]
    pub max: usize,
}

fn default_tier_min() -> usize {
    5
}

fn default_tier_max() -> usize {
    20
}

impl Default for TierCountSchemaConfig {
    fn default() -> Self {
        Self {
            min: default_tier_min(),
            max: default_tier_max(),
        }
    }
}

/// Vendor cluster distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorClusterSchemaConfig {
    /// Reliable strategic vendors percentage (default: 0.20).
    #[serde(default = "default_reliable_strategic")]
    pub reliable_strategic: f64,

    /// Standard operational vendors percentage (default: 0.50).
    #[serde(default = "default_standard_operational")]
    pub standard_operational: f64,

    /// Transactional vendors percentage (default: 0.25).
    #[serde(default = "default_transactional")]
    pub transactional: f64,

    /// Problematic vendors percentage (default: 0.05).
    #[serde(default = "default_problematic")]
    pub problematic: f64,
}

fn default_reliable_strategic() -> f64 {
    0.20
}

fn default_standard_operational() -> f64 {
    0.50
}

fn default_transactional() -> f64 {
    0.25
}

fn default_problematic() -> f64 {
    0.05
}

impl Default for VendorClusterSchemaConfig {
    fn default() -> Self {
        Self {
            reliable_strategic: 0.20,
            standard_operational: 0.50,
            transactional: 0.25,
            problematic: 0.05,
        }
    }
}

/// Dependency and concentration limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySchemaConfig {
    /// Maximum concentration for a single vendor (default: 0.15).
    #[serde(default = "default_max_single_vendor")]
    pub max_single_vendor_concentration: f64,

    /// Maximum concentration for top 5 vendors (default: 0.45).
    #[serde(default = "default_max_top5")]
    pub top_5_concentration: f64,

    /// Percentage of single-source vendors (default: 0.05).
    #[serde(default = "default_single_source_percent")]
    pub single_source_percent: f64,
}

fn default_max_single_vendor() -> f64 {
    0.15
}

fn default_max_top5() -> f64 {
    0.45
}

fn default_single_source_percent() -> f64 {
    0.05
}

impl Default for DependencySchemaConfig {
    fn default() -> Self {
        Self {
            max_single_vendor_concentration: 0.15,
            top_5_concentration: 0.45,
            single_source_percent: 0.05,
        }
    }
}

// =============================================================================
// Customer Segmentation Configuration
// =============================================================================

/// Configuration for customer segmentation generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerSegmentationSchemaConfig {
    /// Enable customer segmentation generation.
    #[serde(default)]
    pub enabled: bool,

    /// Value segment distribution.
    #[serde(default)]
    pub value_segments: ValueSegmentsSchemaConfig,

    /// Lifecycle stage configuration.
    #[serde(default)]
    pub lifecycle: LifecycleSchemaConfig,

    /// Network (referrals, hierarchies) configuration.
    #[serde(default)]
    pub networks: CustomerNetworksSchemaConfig,
}

/// Customer value segments distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSegmentsSchemaConfig {
    /// Enterprise segment configuration.
    #[serde(default)]
    pub enterprise: SegmentDetailSchemaConfig,

    /// Mid-market segment configuration.
    #[serde(default)]
    pub mid_market: SegmentDetailSchemaConfig,

    /// SMB segment configuration.
    #[serde(default)]
    pub smb: SegmentDetailSchemaConfig,

    /// Consumer segment configuration.
    #[serde(default)]
    pub consumer: SegmentDetailSchemaConfig,
}

impl Default for ValueSegmentsSchemaConfig {
    fn default() -> Self {
        Self {
            enterprise: SegmentDetailSchemaConfig {
                revenue_share: 0.40,
                customer_share: 0.05,
                avg_order_value_range: "50000+".to_string(),
            },
            mid_market: SegmentDetailSchemaConfig {
                revenue_share: 0.35,
                customer_share: 0.20,
                avg_order_value_range: "5000-50000".to_string(),
            },
            smb: SegmentDetailSchemaConfig {
                revenue_share: 0.20,
                customer_share: 0.50,
                avg_order_value_range: "500-5000".to_string(),
            },
            consumer: SegmentDetailSchemaConfig {
                revenue_share: 0.05,
                customer_share: 0.25,
                avg_order_value_range: "50-500".to_string(),
            },
        }
    }
}

/// Individual segment detail configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDetailSchemaConfig {
    /// Revenue share for this segment.
    #[serde(default)]
    pub revenue_share: f64,

    /// Customer share for this segment.
    #[serde(default)]
    pub customer_share: f64,

    /// Average order value range (e.g., "5000-50000" or "50000+").
    #[serde(default)]
    pub avg_order_value_range: String,
}

impl Default for SegmentDetailSchemaConfig {
    fn default() -> Self {
        Self {
            revenue_share: 0.25,
            customer_share: 0.25,
            avg_order_value_range: "1000-10000".to_string(),
        }
    }
}

/// Customer lifecycle stage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSchemaConfig {
    /// Prospect stage rate.
    #[serde(default)]
    pub prospect_rate: f64,

    /// New customer stage rate.
    #[serde(default = "default_new_rate")]
    pub new_rate: f64,

    /// Growth stage rate.
    #[serde(default = "default_growth_rate")]
    pub growth_rate: f64,

    /// Mature stage rate.
    #[serde(default = "default_mature_rate")]
    pub mature_rate: f64,

    /// At-risk stage rate.
    #[serde(default = "default_at_risk_rate")]
    pub at_risk_rate: f64,

    /// Churned stage rate.
    #[serde(default = "default_churned_rate")]
    pub churned_rate: f64,
}

fn default_new_rate() -> f64 {
    0.10
}

fn default_growth_rate() -> f64 {
    0.15
}

fn default_mature_rate() -> f64 {
    0.60
}

fn default_at_risk_rate() -> f64 {
    0.10
}

fn default_churned_rate() -> f64 {
    0.05
}

impl Default for LifecycleSchemaConfig {
    fn default() -> Self {
        Self {
            prospect_rate: 0.0,
            new_rate: 0.10,
            growth_rate: 0.15,
            mature_rate: 0.60,
            at_risk_rate: 0.10,
            churned_rate: 0.05,
        }
    }
}

/// Customer networks configuration (referrals, hierarchies).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerNetworksSchemaConfig {
    /// Referral network configuration.
    #[serde(default)]
    pub referrals: ReferralSchemaConfig,

    /// Corporate hierarchy configuration.
    #[serde(default)]
    pub corporate_hierarchies: HierarchySchemaConfig,
}

/// Referral network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralSchemaConfig {
    /// Enable referral generation.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rate of customers acquired via referral.
    #[serde(default = "default_referral_rate")]
    pub referral_rate: f64,
}

fn default_referral_rate() -> f64 {
    0.15
}

impl Default for ReferralSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            referral_rate: 0.15,
        }
    }
}

/// Corporate hierarchy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySchemaConfig {
    /// Enable corporate hierarchy generation.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rate of customers in hierarchies.
    #[serde(default = "default_hierarchy_rate")]
    pub probability: f64,
}

fn default_hierarchy_rate() -> f64 {
    0.30
}

impl Default for HierarchySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            probability: 0.30,
        }
    }
}

// =============================================================================
// Relationship Strength Configuration
// =============================================================================

/// Configuration for relationship strength calculation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipStrengthSchemaConfig {
    /// Enable relationship strength calculation.
    #[serde(default)]
    pub enabled: bool,

    /// Calculation weights.
    #[serde(default)]
    pub calculation: StrengthCalculationSchemaConfig,

    /// Strength thresholds for classification.
    #[serde(default)]
    pub thresholds: StrengthThresholdsSchemaConfig,
}

/// Strength calculation weights configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthCalculationSchemaConfig {
    /// Weight for transaction volume (default: 0.30).
    #[serde(default = "default_volume_weight")]
    pub transaction_volume_weight: f64,

    /// Weight for transaction count (default: 0.25).
    #[serde(default = "default_count_weight")]
    pub transaction_count_weight: f64,

    /// Weight for relationship duration (default: 0.20).
    #[serde(default = "default_duration_weight")]
    pub relationship_duration_weight: f64,

    /// Weight for recency (default: 0.15).
    #[serde(default = "default_recency_weight")]
    pub recency_weight: f64,

    /// Weight for mutual connections (default: 0.10).
    #[serde(default = "default_mutual_weight")]
    pub mutual_connections_weight: f64,

    /// Recency half-life in days (default: 90).
    #[serde(default = "default_recency_half_life")]
    pub recency_half_life_days: u32,
}

fn default_volume_weight() -> f64 {
    0.30
}

fn default_count_weight() -> f64 {
    0.25
}

fn default_duration_weight() -> f64 {
    0.20
}

fn default_recency_weight() -> f64 {
    0.15
}

fn default_mutual_weight() -> f64 {
    0.10
}

fn default_recency_half_life() -> u32 {
    90
}

impl Default for StrengthCalculationSchemaConfig {
    fn default() -> Self {
        Self {
            transaction_volume_weight: 0.30,
            transaction_count_weight: 0.25,
            relationship_duration_weight: 0.20,
            recency_weight: 0.15,
            mutual_connections_weight: 0.10,
            recency_half_life_days: 90,
        }
    }
}

/// Strength thresholds for relationship classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthThresholdsSchemaConfig {
    /// Threshold for strong relationships (default: 0.7).
    #[serde(default = "default_strong_threshold")]
    pub strong: f64,

    /// Threshold for moderate relationships (default: 0.4).
    #[serde(default = "default_moderate_threshold")]
    pub moderate: f64,

    /// Threshold for weak relationships (default: 0.1).
    #[serde(default = "default_weak_threshold")]
    pub weak: f64,
}

fn default_strong_threshold() -> f64 {
    0.7
}

fn default_moderate_threshold() -> f64 {
    0.4
}

fn default_weak_threshold() -> f64 {
    0.1
}

impl Default for StrengthThresholdsSchemaConfig {
    fn default() -> Self {
        Self {
            strong: 0.7,
            moderate: 0.4,
            weak: 0.1,
        }
    }
}

// =============================================================================
// Cross-Process Links Configuration
// =============================================================================

/// Configuration for cross-process linkages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProcessLinksSchemaConfig {
    /// Enable cross-process link generation.
    #[serde(default)]
    pub enabled: bool,

    /// Enable inventory links between P2P and O2C.
    #[serde(default = "default_true")]
    pub inventory_p2p_o2c: bool,

    /// Enable payment to bank reconciliation links.
    #[serde(default = "default_true")]
    pub payment_bank_reconciliation: bool,

    /// Enable intercompany bilateral matching.
    #[serde(default = "default_true")]
    pub intercompany_bilateral: bool,

    /// Percentage of GR/Deliveries to link via inventory (0.0 - 1.0).
    #[serde(default = "default_inventory_link_rate")]
    pub inventory_link_rate: f64,
}

fn default_inventory_link_rate() -> f64 {
    0.30
}

impl Default for CrossProcessLinksSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            inventory_p2p_o2c: true,
            payment_bank_reconciliation: true,
            intercompany_bilateral: true,
            inventory_link_rate: 0.30,
        }
    }
}

// =============================================================================
// Organizational Events Configuration
// =============================================================================

/// Configuration for organizational events (acquisitions, divestitures, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationalEventsSchemaConfig {
    /// Enable organizational events.
    #[serde(default)]
    pub enabled: bool,

    /// Effect blending mode (multiplicative, additive, maximum, minimum).
    #[serde(default)]
    pub effect_blending: EffectBlendingModeConfig,

    /// Organizational events (acquisitions, divestitures, reorganizations, etc.).
    #[serde(default)]
    pub events: Vec<OrganizationalEventSchemaConfig>,

    /// Process evolution events.
    #[serde(default)]
    pub process_evolution: Vec<ProcessEvolutionSchemaConfig>,

    /// Technology transition events.
    #[serde(default)]
    pub technology_transitions: Vec<TechnologyTransitionSchemaConfig>,
}

/// Effect blending mode for combining multiple event effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectBlendingModeConfig {
    /// Multiply effects together.
    #[default]
    Multiplicative,
    /// Add effects together.
    Additive,
    /// Take the maximum effect.
    Maximum,
    /// Take the minimum effect.
    Minimum,
}

/// Configuration for a single organizational event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalEventSchemaConfig {
    /// Event ID.
    pub id: String,

    /// Event type and configuration.
    pub event_type: OrganizationalEventTypeSchemaConfig,

    /// Effective date.
    pub effective_date: String,

    /// Transition duration in months.
    #[serde(default = "default_org_transition_months")]
    pub transition_months: u32,

    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_org_transition_months() -> u32 {
    6
}

/// Organizational event type configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrganizationalEventTypeSchemaConfig {
    /// Acquisition event.
    Acquisition {
        /// Acquired entity code.
        acquired_entity: String,
        /// Volume increase multiplier.
        #[serde(default = "default_acquisition_volume")]
        volume_increase: f64,
        /// Integration error rate.
        #[serde(default = "default_acquisition_error")]
        integration_error_rate: f64,
        /// Parallel posting days.
        #[serde(default = "default_parallel_days")]
        parallel_posting_days: u32,
    },
    /// Divestiture event.
    Divestiture {
        /// Divested entity code.
        divested_entity: String,
        /// Volume reduction factor.
        #[serde(default = "default_divestiture_volume")]
        volume_reduction: f64,
        /// Remove entity from generation.
        #[serde(default = "default_true_val")]
        remove_entity: bool,
    },
    /// Reorganization event.
    Reorganization {
        /// Cost center remapping.
        #[serde(default)]
        cost_center_remapping: std::collections::HashMap<String, String>,
        /// Transition error rate.
        #[serde(default = "default_reorg_error")]
        transition_error_rate: f64,
    },
    /// Leadership change event.
    LeadershipChange {
        /// Role that changed.
        role: String,
        /// Policy changes.
        #[serde(default)]
        policy_changes: Vec<String>,
    },
    /// Workforce reduction event.
    WorkforceReduction {
        /// Reduction percentage.
        #[serde(default = "default_workforce_reduction")]
        reduction_percent: f64,
        /// Error rate increase.
        #[serde(default = "default_workforce_error")]
        error_rate_increase: f64,
    },
    /// Merger event.
    Merger {
        /// Merged entity code.
        merged_entity: String,
        /// Volume increase multiplier.
        #[serde(default = "default_merger_volume")]
        volume_increase: f64,
    },
}

fn default_acquisition_volume() -> f64 {
    1.35
}

fn default_acquisition_error() -> f64 {
    0.05
}

fn default_parallel_days() -> u32 {
    30
}

fn default_divestiture_volume() -> f64 {
    0.70
}

fn default_true_val() -> bool {
    true
}

fn default_reorg_error() -> f64 {
    0.04
}

fn default_workforce_reduction() -> f64 {
    0.10
}

fn default_workforce_error() -> f64 {
    0.05
}

fn default_merger_volume() -> f64 {
    1.80
}

/// Configuration for a process evolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvolutionSchemaConfig {
    /// Event ID.
    pub id: String,

    /// Event type.
    pub event_type: ProcessEvolutionTypeSchemaConfig,

    /// Effective date.
    pub effective_date: String,

    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Process evolution type configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessEvolutionTypeSchemaConfig {
    /// Process automation.
    ProcessAutomation {
        /// Process name.
        process_name: String,
        /// Manual rate before.
        #[serde(default = "default_manual_before")]
        manual_rate_before: f64,
        /// Manual rate after.
        #[serde(default = "default_manual_after")]
        manual_rate_after: f64,
    },
    /// Approval workflow change.
    ApprovalWorkflowChange {
        /// Description.
        description: String,
    },
    /// Control enhancement.
    ControlEnhancement {
        /// Control ID.
        control_id: String,
        /// Error reduction.
        #[serde(default = "default_error_reduction")]
        error_reduction: f64,
    },
}

fn default_manual_before() -> f64 {
    0.80
}

fn default_manual_after() -> f64 {
    0.15
}

fn default_error_reduction() -> f64 {
    0.02
}

/// Configuration for a technology transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyTransitionSchemaConfig {
    /// Event ID.
    pub id: String,

    /// Event type.
    pub event_type: TechnologyTransitionTypeSchemaConfig,

    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Technology transition type configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TechnologyTransitionTypeSchemaConfig {
    /// ERP migration.
    ErpMigration {
        /// Source system.
        source_system: String,
        /// Target system.
        target_system: String,
        /// Cutover date.
        cutover_date: String,
        /// Stabilization end date.
        stabilization_end: String,
        /// Duplicate rate during migration.
        #[serde(default = "default_erp_duplicate_rate")]
        duplicate_rate: f64,
        /// Format mismatch rate.
        #[serde(default = "default_format_mismatch")]
        format_mismatch_rate: f64,
    },
    /// Module implementation.
    ModuleImplementation {
        /// Module name.
        module_name: String,
        /// Go-live date.
        go_live_date: String,
    },
}

fn default_erp_duplicate_rate() -> f64 {
    0.02
}

fn default_format_mismatch() -> f64 {
    0.03
}

// =============================================================================
// Behavioral Drift Configuration
// =============================================================================

/// Configuration for behavioral drift (vendor, customer, employee behavior).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehavioralDriftSchemaConfig {
    /// Enable behavioral drift.
    #[serde(default)]
    pub enabled: bool,

    /// Vendor behavior drift.
    #[serde(default)]
    pub vendor_behavior: VendorBehaviorSchemaConfig,

    /// Customer behavior drift.
    #[serde(default)]
    pub customer_behavior: CustomerBehaviorSchemaConfig,

    /// Employee behavior drift.
    #[serde(default)]
    pub employee_behavior: EmployeeBehaviorSchemaConfig,

    /// Collective behavior drift.
    #[serde(default)]
    pub collective: CollectiveBehaviorSchemaConfig,
}

/// Vendor behavior drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorBehaviorSchemaConfig {
    /// Payment terms drift.
    #[serde(default)]
    pub payment_terms_drift: PaymentTermsDriftSchemaConfig,

    /// Quality drift.
    #[serde(default)]
    pub quality_drift: QualityDriftSchemaConfig,
}

/// Payment terms drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermsDriftSchemaConfig {
    /// Extension rate per year (days).
    #[serde(default = "default_extension_rate")]
    pub extension_rate_per_year: f64,

    /// Economic sensitivity.
    #[serde(default = "default_economic_sensitivity")]
    pub economic_sensitivity: f64,
}

fn default_extension_rate() -> f64 {
    2.5
}

fn default_economic_sensitivity() -> f64 {
    1.0
}

impl Default for PaymentTermsDriftSchemaConfig {
    fn default() -> Self {
        Self {
            extension_rate_per_year: 2.5,
            economic_sensitivity: 1.0,
        }
    }
}

/// Quality drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDriftSchemaConfig {
    /// New vendor improvement rate (per year).
    #[serde(default = "default_improvement_rate")]
    pub new_vendor_improvement_rate: f64,

    /// Complacency decline rate (per year after first year).
    #[serde(default = "default_decline_rate")]
    pub complacency_decline_rate: f64,
}

fn default_improvement_rate() -> f64 {
    0.02
}

fn default_decline_rate() -> f64 {
    0.01
}

impl Default for QualityDriftSchemaConfig {
    fn default() -> Self {
        Self {
            new_vendor_improvement_rate: 0.02,
            complacency_decline_rate: 0.01,
        }
    }
}

/// Customer behavior drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerBehaviorSchemaConfig {
    /// Payment drift.
    #[serde(default)]
    pub payment_drift: CustomerPaymentDriftSchemaConfig,

    /// Order drift.
    #[serde(default)]
    pub order_drift: OrderDriftSchemaConfig,
}

/// Customer payment drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerPaymentDriftSchemaConfig {
    /// Days extension during downturn (min, max).
    #[serde(default = "default_downturn_extension")]
    pub downturn_days_extension: (u32, u32),

    /// Bad debt increase during downturn.
    #[serde(default = "default_bad_debt_increase")]
    pub downturn_bad_debt_increase: f64,
}

fn default_downturn_extension() -> (u32, u32) {
    (5, 15)
}

fn default_bad_debt_increase() -> f64 {
    0.02
}

impl Default for CustomerPaymentDriftSchemaConfig {
    fn default() -> Self {
        Self {
            downturn_days_extension: (5, 15),
            downturn_bad_debt_increase: 0.02,
        }
    }
}

/// Order drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDriftSchemaConfig {
    /// Digital shift rate (per year).
    #[serde(default = "default_digital_shift")]
    pub digital_shift_rate: f64,
}

fn default_digital_shift() -> f64 {
    0.05
}

impl Default for OrderDriftSchemaConfig {
    fn default() -> Self {
        Self {
            digital_shift_rate: 0.05,
        }
    }
}

/// Employee behavior drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeeBehaviorSchemaConfig {
    /// Approval drift.
    #[serde(default)]
    pub approval_drift: ApprovalDriftSchemaConfig,

    /// Error drift.
    #[serde(default)]
    pub error_drift: ErrorDriftSchemaConfig,
}

/// Approval drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDriftSchemaConfig {
    /// EOM intensity increase per year.
    #[serde(default = "default_eom_intensity")]
    pub eom_intensity_increase_per_year: f64,

    /// Rubber stamp volume threshold.
    #[serde(default = "default_rubber_stamp")]
    pub rubber_stamp_volume_threshold: u32,
}

fn default_eom_intensity() -> f64 {
    0.05
}

fn default_rubber_stamp() -> u32 {
    50
}

impl Default for ApprovalDriftSchemaConfig {
    fn default() -> Self {
        Self {
            eom_intensity_increase_per_year: 0.05,
            rubber_stamp_volume_threshold: 50,
        }
    }
}

/// Error drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDriftSchemaConfig {
    /// New employee error rate.
    #[serde(default = "default_new_error")]
    pub new_employee_error_rate: f64,

    /// Learning curve months.
    #[serde(default = "default_learning_months")]
    pub learning_curve_months: u32,
}

fn default_new_error() -> f64 {
    0.08
}

fn default_learning_months() -> u32 {
    6
}

impl Default for ErrorDriftSchemaConfig {
    fn default() -> Self {
        Self {
            new_employee_error_rate: 0.08,
            learning_curve_months: 6,
        }
    }
}

/// Collective behavior drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectiveBehaviorSchemaConfig {
    /// Automation adoption configuration.
    #[serde(default)]
    pub automation_adoption: AutomationAdoptionSchemaConfig,
}

/// Automation adoption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationAdoptionSchemaConfig {
    /// Enable S-curve adoption model.
    #[serde(default)]
    pub s_curve_enabled: bool,

    /// Adoption midpoint in months.
    #[serde(default = "default_midpoint")]
    pub adoption_midpoint_months: u32,

    /// Steepness of adoption curve.
    #[serde(default = "default_steepness")]
    pub steepness: f64,
}

fn default_midpoint() -> u32 {
    24
}

fn default_steepness() -> f64 {
    0.15
}

impl Default for AutomationAdoptionSchemaConfig {
    fn default() -> Self {
        Self {
            s_curve_enabled: false,
            adoption_midpoint_months: 24,
            steepness: 0.15,
        }
    }
}

// =============================================================================
// Market Drift Configuration
// =============================================================================

/// Configuration for market drift (economic cycles, commodities, price shocks).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketDriftSchemaConfig {
    /// Enable market drift.
    #[serde(default)]
    pub enabled: bool,

    /// Economic cycle configuration.
    #[serde(default)]
    pub economic_cycle: MarketEconomicCycleSchemaConfig,

    /// Industry-specific cycles.
    #[serde(default)]
    pub industry_cycles: std::collections::HashMap<String, IndustryCycleSchemaConfig>,

    /// Commodity drift configuration.
    #[serde(default)]
    pub commodities: CommoditiesSchemaConfig,
}

/// Market economic cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEconomicCycleSchemaConfig {
    /// Enable economic cycle.
    #[serde(default)]
    pub enabled: bool,

    /// Cycle type.
    #[serde(default)]
    pub cycle_type: CycleTypeSchemaConfig,

    /// Cycle period in months.
    #[serde(default = "default_market_cycle_period")]
    pub period_months: u32,

    /// Amplitude.
    #[serde(default = "default_market_amplitude")]
    pub amplitude: f64,

    /// Recession configuration.
    #[serde(default)]
    pub recession: RecessionSchemaConfig,
}

fn default_market_cycle_period() -> u32 {
    48
}

fn default_market_amplitude() -> f64 {
    0.15
}

impl Default for MarketEconomicCycleSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cycle_type: CycleTypeSchemaConfig::Sinusoidal,
            period_months: 48,
            amplitude: 0.15,
            recession: RecessionSchemaConfig::default(),
        }
    }
}

/// Cycle type configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleTypeSchemaConfig {
    /// Sinusoidal cycle.
    #[default]
    Sinusoidal,
    /// Asymmetric cycle.
    Asymmetric,
    /// Mean-reverting cycle.
    MeanReverting,
}

/// Recession configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecessionSchemaConfig {
    /// Enable recession simulation.
    #[serde(default)]
    pub enabled: bool,

    /// Probability per year.
    #[serde(default = "default_recession_prob")]
    pub probability_per_year: f64,

    /// Severity.
    #[serde(default)]
    pub severity: RecessionSeveritySchemaConfig,

    /// Specific recession periods.
    #[serde(default)]
    pub recession_periods: Vec<RecessionPeriodSchemaConfig>,
}

fn default_recession_prob() -> f64 {
    0.10
}

impl Default for RecessionSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            probability_per_year: 0.10,
            severity: RecessionSeveritySchemaConfig::Moderate,
            recession_periods: Vec::new(),
        }
    }
}

/// Recession severity configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecessionSeveritySchemaConfig {
    /// Mild recession.
    Mild,
    /// Moderate recession.
    #[default]
    Moderate,
    /// Severe recession.
    Severe,
}

/// Recession period configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecessionPeriodSchemaConfig {
    /// Start month.
    pub start_month: u32,
    /// Duration in months.
    pub duration_months: u32,
}

/// Industry cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryCycleSchemaConfig {
    /// Period in months.
    #[serde(default = "default_industry_period")]
    pub period_months: u32,

    /// Amplitude.
    #[serde(default = "default_industry_amp")]
    pub amplitude: f64,
}

fn default_industry_period() -> u32 {
    36
}

fn default_industry_amp() -> f64 {
    0.20
}

/// Commodities drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommoditiesSchemaConfig {
    /// Enable commodity drift.
    #[serde(default)]
    pub enabled: bool,

    /// Commodity items.
    #[serde(default)]
    pub items: Vec<CommodityItemSchemaConfig>,
}

/// Commodity item configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityItemSchemaConfig {
    /// Commodity name.
    pub name: String,

    /// Volatility.
    #[serde(default = "default_volatility")]
    pub volatility: f64,

    /// COGS pass-through.
    #[serde(default)]
    pub cogs_pass_through: f64,

    /// Overhead pass-through.
    #[serde(default)]
    pub overhead_pass_through: f64,
}

fn default_volatility() -> f64 {
    0.20
}

// =============================================================================
// Drift Labeling Configuration
// =============================================================================

/// Configuration for drift ground truth labeling.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriftLabelingSchemaConfig {
    /// Enable drift labeling.
    #[serde(default)]
    pub enabled: bool,

    /// Statistical drift labeling.
    #[serde(default)]
    pub statistical: StatisticalDriftLabelingSchemaConfig,

    /// Categorical drift labeling.
    #[serde(default)]
    pub categorical: CategoricalDriftLabelingSchemaConfig,

    /// Temporal drift labeling.
    #[serde(default)]
    pub temporal: TemporalDriftLabelingSchemaConfig,

    /// Regulatory calendar preset.
    #[serde(default)]
    pub regulatory_calendar_preset: Option<String>,
}

/// Statistical drift labeling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalDriftLabelingSchemaConfig {
    /// Enable statistical drift labeling.
    #[serde(default = "default_true_val")]
    pub enabled: bool,

    /// Minimum magnitude threshold.
    #[serde(default = "default_min_magnitude")]
    pub min_magnitude_threshold: f64,
}

fn default_min_magnitude() -> f64 {
    0.05
}

impl Default for StatisticalDriftLabelingSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_magnitude_threshold: 0.05,
        }
    }
}

/// Categorical drift labeling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalDriftLabelingSchemaConfig {
    /// Enable categorical drift labeling.
    #[serde(default = "default_true_val")]
    pub enabled: bool,
}

impl Default for CategoricalDriftLabelingSchemaConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Temporal drift labeling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDriftLabelingSchemaConfig {
    /// Enable temporal drift labeling.
    #[serde(default = "default_true_val")]
    pub enabled: bool,
}

impl Default for TemporalDriftLabelingSchemaConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// =============================================================================
// Enhanced Anomaly Injection Configuration
// =============================================================================

/// Enhanced anomaly injection configuration.
///
/// Provides comprehensive anomaly injection capabilities including:
/// - Multi-stage fraud schemes (embezzlement, revenue manipulation, kickbacks)
/// - Correlated anomaly injection (co-occurrence patterns, error cascades)
/// - Near-miss generation for false positive reduction
/// - Detection difficulty classification
/// - Context-aware injection based on entity behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnhancedAnomalyConfig {
    /// Enable enhanced anomaly injection.
    #[serde(default)]
    pub enabled: bool,

    /// Base anomaly rates.
    #[serde(default)]
    pub rates: AnomalyRateConfig,

    /// Multi-stage fraud scheme configuration.
    #[serde(default)]
    pub multi_stage_schemes: MultiStageSchemeConfig,

    /// Correlated anomaly injection configuration.
    #[serde(default)]
    pub correlated_injection: CorrelatedInjectionConfig,

    /// Near-miss generation configuration.
    #[serde(default)]
    pub near_miss: NearMissConfig,

    /// Detection difficulty classification configuration.
    #[serde(default)]
    pub difficulty_classification: DifficultyClassificationConfig,

    /// Context-aware injection configuration.
    #[serde(default)]
    pub context_aware: ContextAwareConfig,

    /// Enhanced labeling configuration.
    #[serde(default)]
    pub labeling: EnhancedLabelingConfig,
}

/// Base anomaly rate configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRateConfig {
    /// Total anomaly rate (0.0 to 1.0).
    #[serde(default = "default_total_anomaly_rate")]
    pub total_rate: f64,

    /// Fraud anomaly rate.
    #[serde(default = "default_fraud_anomaly_rate")]
    pub fraud_rate: f64,

    /// Error anomaly rate.
    #[serde(default = "default_error_anomaly_rate")]
    pub error_rate: f64,

    /// Process issue rate.
    #[serde(default = "default_process_anomaly_rate")]
    pub process_rate: f64,
}

fn default_total_anomaly_rate() -> f64 {
    0.03
}
fn default_fraud_anomaly_rate() -> f64 {
    0.01
}
fn default_error_anomaly_rate() -> f64 {
    0.015
}
fn default_process_anomaly_rate() -> f64 {
    0.005
}

impl Default for AnomalyRateConfig {
    fn default() -> Self {
        Self {
            total_rate: default_total_anomaly_rate(),
            fraud_rate: default_fraud_anomaly_rate(),
            error_rate: default_error_anomaly_rate(),
            process_rate: default_process_anomaly_rate(),
        }
    }
}

/// Multi-stage fraud scheme configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiStageSchemeConfig {
    /// Enable multi-stage fraud schemes.
    #[serde(default)]
    pub enabled: bool,

    /// Embezzlement scheme configuration.
    #[serde(default)]
    pub embezzlement: EmbezzlementSchemeConfig,

    /// Revenue manipulation scheme configuration.
    #[serde(default)]
    pub revenue_manipulation: RevenueManipulationSchemeConfig,

    /// Vendor kickback scheme configuration.
    #[serde(default)]
    pub kickback: KickbackSchemeConfig,
}

/// Embezzlement scheme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbezzlementSchemeConfig {
    /// Probability of starting an embezzlement scheme per perpetrator per year.
    #[serde(default = "default_embezzlement_probability")]
    pub probability: f64,

    /// Testing stage configuration.
    #[serde(default)]
    pub testing_stage: SchemeStageConfig,

    /// Escalation stage configuration.
    #[serde(default)]
    pub escalation_stage: SchemeStageConfig,

    /// Acceleration stage configuration.
    #[serde(default)]
    pub acceleration_stage: SchemeStageConfig,

    /// Desperation stage configuration.
    #[serde(default)]
    pub desperation_stage: SchemeStageConfig,
}

fn default_embezzlement_probability() -> f64 {
    0.02
}

impl Default for EmbezzlementSchemeConfig {
    fn default() -> Self {
        Self {
            probability: default_embezzlement_probability(),
            testing_stage: SchemeStageConfig {
                duration_months: 2,
                amount_min: 100.0,
                amount_max: 500.0,
                transaction_count_min: 2,
                transaction_count_max: 5,
                difficulty: "hard".to_string(),
            },
            escalation_stage: SchemeStageConfig {
                duration_months: 6,
                amount_min: 500.0,
                amount_max: 2000.0,
                transaction_count_min: 3,
                transaction_count_max: 8,
                difficulty: "moderate".to_string(),
            },
            acceleration_stage: SchemeStageConfig {
                duration_months: 3,
                amount_min: 2000.0,
                amount_max: 10000.0,
                transaction_count_min: 5,
                transaction_count_max: 12,
                difficulty: "easy".to_string(),
            },
            desperation_stage: SchemeStageConfig {
                duration_months: 1,
                amount_min: 10000.0,
                amount_max: 50000.0,
                transaction_count_min: 3,
                transaction_count_max: 6,
                difficulty: "trivial".to_string(),
            },
        }
    }
}

/// Revenue manipulation scheme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueManipulationSchemeConfig {
    /// Probability of starting a revenue manipulation scheme per period.
    #[serde(default = "default_revenue_manipulation_probability")]
    pub probability: f64,

    /// Early revenue recognition inflation target (Q4).
    #[serde(default = "default_early_recognition_target")]
    pub early_recognition_target: f64,

    /// Expense deferral inflation target (Q1).
    #[serde(default = "default_expense_deferral_target")]
    pub expense_deferral_target: f64,

    /// Reserve release inflation target (Q2).
    #[serde(default = "default_reserve_release_target")]
    pub reserve_release_target: f64,

    /// Channel stuffing inflation target (Q4).
    #[serde(default = "default_channel_stuffing_target")]
    pub channel_stuffing_target: f64,
}

fn default_revenue_manipulation_probability() -> f64 {
    0.01
}
fn default_early_recognition_target() -> f64 {
    0.02
}
fn default_expense_deferral_target() -> f64 {
    0.03
}
fn default_reserve_release_target() -> f64 {
    0.02
}
fn default_channel_stuffing_target() -> f64 {
    0.05
}

impl Default for RevenueManipulationSchemeConfig {
    fn default() -> Self {
        Self {
            probability: default_revenue_manipulation_probability(),
            early_recognition_target: default_early_recognition_target(),
            expense_deferral_target: default_expense_deferral_target(),
            reserve_release_target: default_reserve_release_target(),
            channel_stuffing_target: default_channel_stuffing_target(),
        }
    }
}

/// Vendor kickback scheme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KickbackSchemeConfig {
    /// Probability of starting a kickback scheme.
    #[serde(default = "default_kickback_probability")]
    pub probability: f64,

    /// Minimum price inflation percentage.
    #[serde(default = "default_kickback_inflation_min")]
    pub inflation_min: f64,

    /// Maximum price inflation percentage.
    #[serde(default = "default_kickback_inflation_max")]
    pub inflation_max: f64,

    /// Kickback percentage (of inflation).
    #[serde(default = "default_kickback_percent")]
    pub kickback_percent: f64,

    /// Setup duration in months.
    #[serde(default = "default_kickback_setup_months")]
    pub setup_months: u32,

    /// Main operation duration in months.
    #[serde(default = "default_kickback_operation_months")]
    pub operation_months: u32,
}

fn default_kickback_probability() -> f64 {
    0.01
}
fn default_kickback_inflation_min() -> f64 {
    0.10
}
fn default_kickback_inflation_max() -> f64 {
    0.25
}
fn default_kickback_percent() -> f64 {
    0.50
}
fn default_kickback_setup_months() -> u32 {
    3
}
fn default_kickback_operation_months() -> u32 {
    12
}

impl Default for KickbackSchemeConfig {
    fn default() -> Self {
        Self {
            probability: default_kickback_probability(),
            inflation_min: default_kickback_inflation_min(),
            inflation_max: default_kickback_inflation_max(),
            kickback_percent: default_kickback_percent(),
            setup_months: default_kickback_setup_months(),
            operation_months: default_kickback_operation_months(),
        }
    }
}

/// Individual scheme stage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeStageConfig {
    /// Duration in months.
    pub duration_months: u32,

    /// Minimum transaction amount.
    pub amount_min: f64,

    /// Maximum transaction amount.
    pub amount_max: f64,

    /// Minimum number of transactions.
    pub transaction_count_min: u32,

    /// Maximum number of transactions.
    pub transaction_count_max: u32,

    /// Detection difficulty level (trivial, easy, moderate, hard, expert).
    pub difficulty: String,
}

impl Default for SchemeStageConfig {
    fn default() -> Self {
        Self {
            duration_months: 3,
            amount_min: 100.0,
            amount_max: 1000.0,
            transaction_count_min: 2,
            transaction_count_max: 10,
            difficulty: "moderate".to_string(),
        }
    }
}

/// Correlated anomaly injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedInjectionConfig {
    /// Enable correlated anomaly injection.
    #[serde(default)]
    pub enabled: bool,

    /// Enable fraud concealment co-occurrence patterns.
    #[serde(default = "default_true_val")]
    pub fraud_concealment: bool,

    /// Enable error cascade patterns.
    #[serde(default = "default_true_val")]
    pub error_cascade: bool,

    /// Enable temporal clustering (period-end spikes).
    #[serde(default = "default_true_val")]
    pub temporal_clustering: bool,

    /// Temporal clustering configuration.
    #[serde(default)]
    pub temporal_clustering_config: TemporalClusteringConfig,

    /// Co-occurrence patterns.
    #[serde(default)]
    pub co_occurrence_patterns: Vec<CoOccurrencePatternConfig>,
}

impl Default for CorrelatedInjectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fraud_concealment: true,
            error_cascade: true,
            temporal_clustering: true,
            temporal_clustering_config: TemporalClusteringConfig::default(),
            co_occurrence_patterns: Vec::new(),
        }
    }
}

/// Temporal clustering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalClusteringConfig {
    /// Period-end error multiplier.
    #[serde(default = "default_period_end_multiplier")]
    pub period_end_multiplier: f64,

    /// Number of business days before period end to apply multiplier.
    #[serde(default = "default_period_end_days")]
    pub period_end_days: u32,

    /// Quarter-end additional multiplier.
    #[serde(default = "default_quarter_end_multiplier")]
    pub quarter_end_multiplier: f64,

    /// Year-end additional multiplier.
    #[serde(default = "default_year_end_multiplier")]
    pub year_end_multiplier: f64,
}

fn default_period_end_multiplier() -> f64 {
    2.5
}
fn default_period_end_days() -> u32 {
    5
}
fn default_quarter_end_multiplier() -> f64 {
    1.5
}
fn default_year_end_multiplier() -> f64 {
    2.0
}

impl Default for TemporalClusteringConfig {
    fn default() -> Self {
        Self {
            period_end_multiplier: default_period_end_multiplier(),
            period_end_days: default_period_end_days(),
            quarter_end_multiplier: default_quarter_end_multiplier(),
            year_end_multiplier: default_year_end_multiplier(),
        }
    }
}

/// Co-occurrence pattern configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoOccurrencePatternConfig {
    /// Pattern name.
    pub name: String,

    /// Primary anomaly type that triggers the pattern.
    pub primary_type: String,

    /// Correlated anomalies.
    pub correlated: Vec<CorrelatedAnomalyConfig>,
}

/// Correlated anomaly configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedAnomalyConfig {
    /// Anomaly type.
    pub anomaly_type: String,

    /// Probability of occurrence (0.0 to 1.0).
    pub probability: f64,

    /// Minimum lag in days.
    pub lag_days_min: i32,

    /// Maximum lag in days.
    pub lag_days_max: i32,
}

/// Near-miss generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMissConfig {
    /// Enable near-miss generation.
    #[serde(default)]
    pub enabled: bool,

    /// Proportion of "anomalies" that are actually near-misses (0.0 to 1.0).
    #[serde(default = "default_near_miss_proportion")]
    pub proportion: f64,

    /// Enable near-duplicate pattern.
    #[serde(default = "default_true_val")]
    pub near_duplicate: bool,

    /// Near-duplicate date difference range in days.
    #[serde(default)]
    pub near_duplicate_days: NearDuplicateDaysConfig,

    /// Enable threshold proximity pattern.
    #[serde(default = "default_true_val")]
    pub threshold_proximity: bool,

    /// Threshold proximity range (e.g., 0.90-0.99 of threshold).
    #[serde(default)]
    pub threshold_proximity_range: ThresholdProximityRangeConfig,

    /// Enable unusual but legitimate patterns.
    #[serde(default = "default_true_val")]
    pub unusual_legitimate: bool,

    /// Types of unusual legitimate patterns to generate.
    #[serde(default = "default_unusual_legitimate_types")]
    pub unusual_legitimate_types: Vec<String>,

    /// Enable corrected error patterns.
    #[serde(default = "default_true_val")]
    pub corrected_errors: bool,

    /// Corrected error correction lag range in days.
    #[serde(default)]
    pub corrected_error_lag: CorrectedErrorLagConfig,
}

fn default_near_miss_proportion() -> f64 {
    0.30
}

fn default_unusual_legitimate_types() -> Vec<String> {
    vec![
        "year_end_bonus".to_string(),
        "contract_prepayment".to_string(),
        "insurance_claim".to_string(),
        "settlement_payment".to_string(),
    ]
}

impl Default for NearMissConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            proportion: default_near_miss_proportion(),
            near_duplicate: true,
            near_duplicate_days: NearDuplicateDaysConfig::default(),
            threshold_proximity: true,
            threshold_proximity_range: ThresholdProximityRangeConfig::default(),
            unusual_legitimate: true,
            unusual_legitimate_types: default_unusual_legitimate_types(),
            corrected_errors: true,
            corrected_error_lag: CorrectedErrorLagConfig::default(),
        }
    }
}

/// Near-duplicate days configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDuplicateDaysConfig {
    /// Minimum days apart.
    #[serde(default = "default_near_duplicate_min")]
    pub min: u32,

    /// Maximum days apart.
    #[serde(default = "default_near_duplicate_max")]
    pub max: u32,
}

fn default_near_duplicate_min() -> u32 {
    1
}
fn default_near_duplicate_max() -> u32 {
    3
}

impl Default for NearDuplicateDaysConfig {
    fn default() -> Self {
        Self {
            min: default_near_duplicate_min(),
            max: default_near_duplicate_max(),
        }
    }
}

/// Threshold proximity range configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdProximityRangeConfig {
    /// Minimum proximity (e.g., 0.90 = 90% of threshold).
    #[serde(default = "default_threshold_proximity_min")]
    pub min: f64,

    /// Maximum proximity (e.g., 0.99 = 99% of threshold).
    #[serde(default = "default_threshold_proximity_max")]
    pub max: f64,
}

fn default_threshold_proximity_min() -> f64 {
    0.90
}
fn default_threshold_proximity_max() -> f64 {
    0.99
}

impl Default for ThresholdProximityRangeConfig {
    fn default() -> Self {
        Self {
            min: default_threshold_proximity_min(),
            max: default_threshold_proximity_max(),
        }
    }
}

/// Corrected error lag configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectedErrorLagConfig {
    /// Minimum correction lag in days.
    #[serde(default = "default_corrected_error_lag_min")]
    pub min: u32,

    /// Maximum correction lag in days.
    #[serde(default = "default_corrected_error_lag_max")]
    pub max: u32,
}

fn default_corrected_error_lag_min() -> u32 {
    1
}
fn default_corrected_error_lag_max() -> u32 {
    5
}

impl Default for CorrectedErrorLagConfig {
    fn default() -> Self {
        Self {
            min: default_corrected_error_lag_min(),
            max: default_corrected_error_lag_max(),
        }
    }
}

/// Detection difficulty classification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyClassificationConfig {
    /// Enable detection difficulty classification.
    #[serde(default)]
    pub enabled: bool,

    /// Target distribution of difficulty levels.
    #[serde(default)]
    pub target_distribution: DifficultyDistributionConfig,
}

impl Default for DifficultyClassificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_distribution: DifficultyDistributionConfig::default(),
        }
    }
}

/// Target distribution of detection difficulty levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyDistributionConfig {
    /// Proportion of trivial anomalies (expected 99% detection).
    #[serde(default = "default_difficulty_trivial")]
    pub trivial: f64,

    /// Proportion of easy anomalies (expected 90% detection).
    #[serde(default = "default_difficulty_easy")]
    pub easy: f64,

    /// Proportion of moderate anomalies (expected 70% detection).
    #[serde(default = "default_difficulty_moderate")]
    pub moderate: f64,

    /// Proportion of hard anomalies (expected 40% detection).
    #[serde(default = "default_difficulty_hard")]
    pub hard: f64,

    /// Proportion of expert anomalies (expected 15% detection).
    #[serde(default = "default_difficulty_expert")]
    pub expert: f64,
}

fn default_difficulty_trivial() -> f64 {
    0.15
}
fn default_difficulty_easy() -> f64 {
    0.25
}
fn default_difficulty_moderate() -> f64 {
    0.30
}
fn default_difficulty_hard() -> f64 {
    0.20
}
fn default_difficulty_expert() -> f64 {
    0.10
}

impl Default for DifficultyDistributionConfig {
    fn default() -> Self {
        Self {
            trivial: default_difficulty_trivial(),
            easy: default_difficulty_easy(),
            moderate: default_difficulty_moderate(),
            hard: default_difficulty_hard(),
            expert: default_difficulty_expert(),
        }
    }
}

/// Context-aware injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextAwareConfig {
    /// Enable context-aware injection.
    #[serde(default)]
    pub enabled: bool,

    /// Vendor-specific anomaly rules.
    #[serde(default)]
    pub vendor_rules: VendorAnomalyRulesConfig,

    /// Employee-specific anomaly rules.
    #[serde(default)]
    pub employee_rules: EmployeeAnomalyRulesConfig,

    /// Account-specific anomaly rules.
    #[serde(default)]
    pub account_rules: AccountAnomalyRulesConfig,

    /// Behavioral baseline configuration.
    #[serde(default)]
    pub behavioral_baseline: BehavioralBaselineConfig,
}

/// Vendor-specific anomaly rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAnomalyRulesConfig {
    /// Error rate multiplier for new vendors (< threshold days).
    #[serde(default = "default_new_vendor_multiplier")]
    pub new_vendor_error_multiplier: f64,

    /// Days threshold for "new" vendor classification.
    #[serde(default = "default_new_vendor_threshold")]
    pub new_vendor_threshold_days: u32,

    /// Error rate multiplier for international vendors.
    #[serde(default = "default_international_multiplier")]
    pub international_error_multiplier: f64,

    /// Strategic vendor anomaly types (may differ from general vendors).
    #[serde(default = "default_strategic_vendor_types")]
    pub strategic_vendor_anomaly_types: Vec<String>,
}

fn default_new_vendor_multiplier() -> f64 {
    2.5
}
fn default_new_vendor_threshold() -> u32 {
    90
}
fn default_international_multiplier() -> f64 {
    1.5
}
fn default_strategic_vendor_types() -> Vec<String> {
    vec![
        "pricing_dispute".to_string(),
        "contract_violation".to_string(),
    ]
}

impl Default for VendorAnomalyRulesConfig {
    fn default() -> Self {
        Self {
            new_vendor_error_multiplier: default_new_vendor_multiplier(),
            new_vendor_threshold_days: default_new_vendor_threshold(),
            international_error_multiplier: default_international_multiplier(),
            strategic_vendor_anomaly_types: default_strategic_vendor_types(),
        }
    }
}

/// Employee-specific anomaly rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeAnomalyRulesConfig {
    /// Error rate for new employees (< threshold days).
    #[serde(default = "default_new_employee_rate")]
    pub new_employee_error_rate: f64,

    /// Days threshold for "new" employee classification.
    #[serde(default = "default_new_employee_threshold")]
    pub new_employee_threshold_days: u32,

    /// Transaction volume threshold for fatigue errors.
    #[serde(default = "default_volume_fatigue_threshold")]
    pub volume_fatigue_threshold: u32,

    /// Error rate multiplier when primary approver is absent.
    #[serde(default = "default_coverage_multiplier")]
    pub coverage_error_multiplier: f64,
}

fn default_new_employee_rate() -> f64 {
    0.05
}
fn default_new_employee_threshold() -> u32 {
    180
}
fn default_volume_fatigue_threshold() -> u32 {
    50
}
fn default_coverage_multiplier() -> f64 {
    1.8
}

impl Default for EmployeeAnomalyRulesConfig {
    fn default() -> Self {
        Self {
            new_employee_error_rate: default_new_employee_rate(),
            new_employee_threshold_days: default_new_employee_threshold(),
            volume_fatigue_threshold: default_volume_fatigue_threshold(),
            coverage_error_multiplier: default_coverage_multiplier(),
        }
    }
}

/// Account-specific anomaly rules configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAnomalyRulesConfig {
    /// Error rate multiplier for high-risk accounts.
    #[serde(default = "default_high_risk_multiplier")]
    pub high_risk_account_multiplier: f64,

    /// Account codes considered high-risk.
    #[serde(default = "default_high_risk_accounts")]
    pub high_risk_accounts: Vec<String>,

    /// Error rate multiplier for suspense accounts.
    #[serde(default = "default_suspense_multiplier")]
    pub suspense_account_multiplier: f64,

    /// Account codes considered suspense accounts.
    #[serde(default = "default_suspense_accounts")]
    pub suspense_accounts: Vec<String>,

    /// Error rate multiplier for intercompany accounts.
    #[serde(default = "default_intercompany_multiplier")]
    pub intercompany_account_multiplier: f64,
}

fn default_high_risk_multiplier() -> f64 {
    2.0
}
fn default_high_risk_accounts() -> Vec<String> {
    vec![
        "1100".to_string(), // AR Control
        "2000".to_string(), // AP Control
        "3000".to_string(), // Cash
    ]
}
fn default_suspense_multiplier() -> f64 {
    3.0
}
fn default_suspense_accounts() -> Vec<String> {
    vec!["9999".to_string(), "9998".to_string()]
}
fn default_intercompany_multiplier() -> f64 {
    1.5
}

impl Default for AccountAnomalyRulesConfig {
    fn default() -> Self {
        Self {
            high_risk_account_multiplier: default_high_risk_multiplier(),
            high_risk_accounts: default_high_risk_accounts(),
            suspense_account_multiplier: default_suspense_multiplier(),
            suspense_accounts: default_suspense_accounts(),
            intercompany_account_multiplier: default_intercompany_multiplier(),
        }
    }
}

/// Behavioral baseline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralBaselineConfig {
    /// Enable behavioral baseline tracking.
    #[serde(default)]
    pub enabled: bool,

    /// Number of days to build baseline from.
    #[serde(default = "default_baseline_period")]
    pub baseline_period_days: u32,

    /// Standard deviation threshold for amount anomalies.
    #[serde(default = "default_deviation_threshold")]
    pub deviation_threshold_std: f64,

    /// Standard deviation threshold for frequency anomalies.
    #[serde(default = "default_frequency_deviation")]
    pub frequency_deviation_threshold: f64,
}

fn default_baseline_period() -> u32 {
    90
}
fn default_deviation_threshold() -> f64 {
    3.0
}
fn default_frequency_deviation() -> f64 {
    2.0
}

impl Default for BehavioralBaselineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            baseline_period_days: default_baseline_period(),
            deviation_threshold_std: default_deviation_threshold(),
            frequency_deviation_threshold: default_frequency_deviation(),
        }
    }
}

/// Enhanced labeling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLabelingConfig {
    /// Enable severity scoring.
    #[serde(default = "default_true_val")]
    pub severity_scoring: bool,

    /// Enable difficulty classification.
    #[serde(default = "default_true_val")]
    pub difficulty_classification: bool,

    /// Materiality thresholds for severity classification.
    #[serde(default)]
    pub materiality_thresholds: MaterialityThresholdsConfig,
}

impl Default for EnhancedLabelingConfig {
    fn default() -> Self {
        Self {
            severity_scoring: true,
            difficulty_classification: true,
            materiality_thresholds: MaterialityThresholdsConfig::default(),
        }
    }
}

/// Materiality thresholds configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialityThresholdsConfig {
    /// Threshold for trivial impact (as percentage of total).
    #[serde(default = "default_materiality_trivial")]
    pub trivial: f64,

    /// Threshold for immaterial impact.
    #[serde(default = "default_materiality_immaterial")]
    pub immaterial: f64,

    /// Threshold for material impact.
    #[serde(default = "default_materiality_material")]
    pub material: f64,

    /// Threshold for highly material impact.
    #[serde(default = "default_materiality_highly_material")]
    pub highly_material: f64,
}

fn default_materiality_trivial() -> f64 {
    0.001
}
fn default_materiality_immaterial() -> f64 {
    0.01
}
fn default_materiality_material() -> f64 {
    0.05
}
fn default_materiality_highly_material() -> f64 {
    0.10
}

impl Default for MaterialityThresholdsConfig {
    fn default() -> Self {
        Self {
            trivial: default_materiality_trivial(),
            immaterial: default_materiality_immaterial(),
            material: default_materiality_material(),
            highly_material: default_materiality_highly_material(),
        }
    }
}

// =============================================================================
// Industry-Specific Configuration
// =============================================================================

/// Industry-specific transaction and anomaly generation configuration.
///
/// This configuration enables generation of industry-authentic:
/// - Transaction types with appropriate terminology
/// - Master data (BOM, routings, clinical codes, etc.)
/// - Industry-specific anomaly patterns
/// - Regulatory framework compliance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndustrySpecificConfig {
    /// Enable industry-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Manufacturing industry settings.
    #[serde(default)]
    pub manufacturing: ManufacturingConfig,

    /// Retail industry settings.
    #[serde(default)]
    pub retail: RetailConfig,

    /// Healthcare industry settings.
    #[serde(default)]
    pub healthcare: HealthcareConfig,

    /// Technology industry settings.
    #[serde(default)]
    pub technology: TechnologyConfig,

    /// Financial services industry settings.
    #[serde(default)]
    pub financial_services: FinancialServicesConfig,

    /// Professional services industry settings.
    #[serde(default)]
    pub professional_services: ProfessionalServicesConfig,
}

/// Manufacturing industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingConfig {
    /// Enable manufacturing-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Bill of Materials depth (typical: 3-7).
    #[serde(default = "default_bom_depth")]
    pub bom_depth: u32,

    /// Whether to use just-in-time inventory.
    #[serde(default)]
    pub just_in_time: bool,

    /// Production order types to generate.
    #[serde(default = "default_production_order_types")]
    pub production_order_types: Vec<String>,

    /// Quality framework (ISO_9001, Six_Sigma, etc.).
    #[serde(default)]
    pub quality_framework: Option<String>,

    /// Number of supplier tiers to model (1-3).
    #[serde(default = "default_supplier_tiers")]
    pub supplier_tiers: u32,

    /// Standard cost update frequency.
    #[serde(default = "default_cost_frequency")]
    pub standard_cost_frequency: String,

    /// Target yield rate (0.95-0.99 typical).
    #[serde(default = "default_yield_rate")]
    pub target_yield_rate: f64,

    /// Scrap percentage threshold for alerts.
    #[serde(default = "default_scrap_threshold")]
    pub scrap_alert_threshold: f64,

    /// Manufacturing anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: ManufacturingAnomalyRates,
}

fn default_bom_depth() -> u32 {
    4
}

fn default_production_order_types() -> Vec<String> {
    vec![
        "standard".to_string(),
        "rework".to_string(),
        "prototype".to_string(),
    ]
}

fn default_supplier_tiers() -> u32 {
    2
}

fn default_cost_frequency() -> String {
    "quarterly".to_string()
}

fn default_yield_rate() -> f64 {
    0.97
}

fn default_scrap_threshold() -> f64 {
    0.03
}

impl Default for ManufacturingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bom_depth: default_bom_depth(),
            just_in_time: false,
            production_order_types: default_production_order_types(),
            quality_framework: Some("ISO_9001".to_string()),
            supplier_tiers: default_supplier_tiers(),
            standard_cost_frequency: default_cost_frequency(),
            target_yield_rate: default_yield_rate(),
            scrap_alert_threshold: default_scrap_threshold(),
            anomaly_rates: ManufacturingAnomalyRates::default(),
        }
    }
}

/// Manufacturing anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingAnomalyRates {
    /// Yield manipulation rate.
    #[serde(default = "default_mfg_yield_rate")]
    pub yield_manipulation: f64,

    /// Labor misallocation rate.
    #[serde(default = "default_mfg_labor_rate")]
    pub labor_misallocation: f64,

    /// Phantom production rate.
    #[serde(default = "default_mfg_phantom_rate")]
    pub phantom_production: f64,

    /// Standard cost manipulation rate.
    #[serde(default = "default_mfg_cost_rate")]
    pub standard_cost_manipulation: f64,

    /// Inventory fraud rate.
    #[serde(default = "default_mfg_inventory_rate")]
    pub inventory_fraud: f64,
}

fn default_mfg_yield_rate() -> f64 {
    0.015
}

fn default_mfg_labor_rate() -> f64 {
    0.02
}

fn default_mfg_phantom_rate() -> f64 {
    0.005
}

fn default_mfg_cost_rate() -> f64 {
    0.01
}

fn default_mfg_inventory_rate() -> f64 {
    0.008
}

impl Default for ManufacturingAnomalyRates {
    fn default() -> Self {
        Self {
            yield_manipulation: default_mfg_yield_rate(),
            labor_misallocation: default_mfg_labor_rate(),
            phantom_production: default_mfg_phantom_rate(),
            standard_cost_manipulation: default_mfg_cost_rate(),
            inventory_fraud: default_mfg_inventory_rate(),
        }
    }
}

/// Retail industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetailConfig {
    /// Enable retail-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Store type distribution.
    #[serde(default)]
    pub store_types: RetailStoreTypeConfig,

    /// Average daily transactions per store.
    #[serde(default = "default_retail_daily_txns")]
    pub avg_daily_transactions: u32,

    /// Enable loss prevention tracking.
    #[serde(default = "default_true")]
    pub loss_prevention: bool,

    /// Shrinkage rate (0.01-0.03 typical).
    #[serde(default = "default_shrinkage_rate")]
    pub shrinkage_rate: f64,

    /// Retail anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: RetailAnomalyRates,
}

fn default_retail_daily_txns() -> u32 {
    500
}

fn default_shrinkage_rate() -> f64 {
    0.015
}

impl Default for RetailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            store_types: RetailStoreTypeConfig::default(),
            avg_daily_transactions: default_retail_daily_txns(),
            loss_prevention: true,
            shrinkage_rate: default_shrinkage_rate(),
            anomaly_rates: RetailAnomalyRates::default(),
        }
    }
}

/// Retail store type distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetailStoreTypeConfig {
    /// Percentage of flagship stores.
    #[serde(default = "default_flagship_pct")]
    pub flagship: f64,

    /// Percentage of regional stores.
    #[serde(default = "default_regional_pct")]
    pub regional: f64,

    /// Percentage of outlet stores.
    #[serde(default = "default_outlet_pct")]
    pub outlet: f64,

    /// Percentage of e-commerce.
    #[serde(default = "default_ecommerce_pct")]
    pub ecommerce: f64,
}

fn default_flagship_pct() -> f64 {
    0.10
}

fn default_regional_pct() -> f64 {
    0.50
}

fn default_outlet_pct() -> f64 {
    0.25
}

fn default_ecommerce_pct() -> f64 {
    0.15
}

impl Default for RetailStoreTypeConfig {
    fn default() -> Self {
        Self {
            flagship: default_flagship_pct(),
            regional: default_regional_pct(),
            outlet: default_outlet_pct(),
            ecommerce: default_ecommerce_pct(),
        }
    }
}

/// Retail anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetailAnomalyRates {
    /// Sweethearting rate.
    #[serde(default = "default_sweethearting_rate")]
    pub sweethearting: f64,

    /// Skimming rate.
    #[serde(default = "default_skimming_rate")]
    pub skimming: f64,

    /// Refund fraud rate.
    #[serde(default = "default_refund_fraud_rate")]
    pub refund_fraud: f64,

    /// Void abuse rate.
    #[serde(default = "default_void_abuse_rate")]
    pub void_abuse: f64,

    /// Gift card fraud rate.
    #[serde(default = "default_gift_card_rate")]
    pub gift_card_fraud: f64,

    /// Vendor kickback rate.
    #[serde(default = "default_retail_kickback_rate")]
    pub vendor_kickback: f64,
}

fn default_sweethearting_rate() -> f64 {
    0.02
}

fn default_skimming_rate() -> f64 {
    0.005
}

fn default_refund_fraud_rate() -> f64 {
    0.015
}

fn default_void_abuse_rate() -> f64 {
    0.01
}

fn default_gift_card_rate() -> f64 {
    0.008
}

fn default_retail_kickback_rate() -> f64 {
    0.003
}

impl Default for RetailAnomalyRates {
    fn default() -> Self {
        Self {
            sweethearting: default_sweethearting_rate(),
            skimming: default_skimming_rate(),
            refund_fraud: default_refund_fraud_rate(),
            void_abuse: default_void_abuse_rate(),
            gift_card_fraud: default_gift_card_rate(),
            vendor_kickback: default_retail_kickback_rate(),
        }
    }
}

/// Healthcare industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareConfig {
    /// Enable healthcare-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Healthcare facility type.
    #[serde(default = "default_facility_type")]
    pub facility_type: String,

    /// Payer mix distribution.
    #[serde(default)]
    pub payer_mix: HealthcarePayerMix,

    /// Coding systems enabled.
    #[serde(default)]
    pub coding_systems: HealthcareCodingSystems,

    /// Healthcare compliance settings.
    #[serde(default)]
    pub compliance: HealthcareComplianceConfig,

    /// Average daily encounters.
    #[serde(default = "default_daily_encounters")]
    pub avg_daily_encounters: u32,

    /// Average charges per encounter.
    #[serde(default = "default_charges_per_encounter")]
    pub avg_charges_per_encounter: u32,

    /// Denial rate (0.0-1.0).
    #[serde(default = "default_hc_denial_rate")]
    pub denial_rate: f64,

    /// Bad debt rate (0.0-1.0).
    #[serde(default = "default_hc_bad_debt_rate")]
    pub bad_debt_rate: f64,

    /// Charity care rate (0.0-1.0).
    #[serde(default = "default_hc_charity_care_rate")]
    pub charity_care_rate: f64,

    /// Healthcare anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: HealthcareAnomalyRates,
}

fn default_facility_type() -> String {
    "hospital".to_string()
}

fn default_daily_encounters() -> u32 {
    150
}

fn default_charges_per_encounter() -> u32 {
    8
}

fn default_hc_denial_rate() -> f64 {
    0.05
}

fn default_hc_bad_debt_rate() -> f64 {
    0.03
}

fn default_hc_charity_care_rate() -> f64 {
    0.02
}

impl Default for HealthcareConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            facility_type: default_facility_type(),
            payer_mix: HealthcarePayerMix::default(),
            coding_systems: HealthcareCodingSystems::default(),
            compliance: HealthcareComplianceConfig::default(),
            avg_daily_encounters: default_daily_encounters(),
            avg_charges_per_encounter: default_charges_per_encounter(),
            denial_rate: default_hc_denial_rate(),
            bad_debt_rate: default_hc_bad_debt_rate(),
            charity_care_rate: default_hc_charity_care_rate(),
            anomaly_rates: HealthcareAnomalyRates::default(),
        }
    }
}

/// Healthcare payer mix distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcarePayerMix {
    /// Medicare percentage.
    #[serde(default = "default_medicare_pct")]
    pub medicare: f64,

    /// Medicaid percentage.
    #[serde(default = "default_medicaid_pct")]
    pub medicaid: f64,

    /// Commercial insurance percentage.
    #[serde(default = "default_commercial_pct")]
    pub commercial: f64,

    /// Self-pay percentage.
    #[serde(default = "default_self_pay_pct")]
    pub self_pay: f64,
}

fn default_medicare_pct() -> f64 {
    0.40
}

fn default_medicaid_pct() -> f64 {
    0.20
}

fn default_commercial_pct() -> f64 {
    0.30
}

fn default_self_pay_pct() -> f64 {
    0.10
}

impl Default for HealthcarePayerMix {
    fn default() -> Self {
        Self {
            medicare: default_medicare_pct(),
            medicaid: default_medicaid_pct(),
            commercial: default_commercial_pct(),
            self_pay: default_self_pay_pct(),
        }
    }
}

/// Healthcare coding systems configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareCodingSystems {
    /// Enable ICD-10 diagnosis coding.
    #[serde(default = "default_true")]
    pub icd10: bool,

    /// Enable CPT procedure coding.
    #[serde(default = "default_true")]
    pub cpt: bool,

    /// Enable DRG grouping.
    #[serde(default = "default_true")]
    pub drg: bool,

    /// Enable HCPCS Level II coding.
    #[serde(default = "default_true")]
    pub hcpcs: bool,

    /// Enable revenue codes.
    #[serde(default = "default_true")]
    pub revenue_codes: bool,
}

impl Default for HealthcareCodingSystems {
    fn default() -> Self {
        Self {
            icd10: true,
            cpt: true,
            drg: true,
            hcpcs: true,
            revenue_codes: true,
        }
    }
}

/// Healthcare compliance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareComplianceConfig {
    /// Enable HIPAA compliance.
    #[serde(default = "default_true")]
    pub hipaa: bool,

    /// Enable Stark Law compliance.
    #[serde(default = "default_true")]
    pub stark_law: bool,

    /// Enable Anti-Kickback Statute compliance.
    #[serde(default = "default_true")]
    pub anti_kickback: bool,

    /// Enable False Claims Act compliance.
    #[serde(default = "default_true")]
    pub false_claims_act: bool,

    /// Enable EMTALA compliance (for hospitals).
    #[serde(default = "default_true")]
    pub emtala: bool,
}

impl Default for HealthcareComplianceConfig {
    fn default() -> Self {
        Self {
            hipaa: true,
            stark_law: true,
            anti_kickback: true,
            false_claims_act: true,
            emtala: true,
        }
    }
}

/// Healthcare anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareAnomalyRates {
    /// Upcoding rate.
    #[serde(default = "default_upcoding_rate")]
    pub upcoding: f64,

    /// Unbundling rate.
    #[serde(default = "default_unbundling_rate")]
    pub unbundling: f64,

    /// Phantom billing rate.
    #[serde(default = "default_phantom_billing_rate")]
    pub phantom_billing: f64,

    /// Kickback rate.
    #[serde(default = "default_healthcare_kickback_rate")]
    pub kickbacks: f64,

    /// Duplicate billing rate.
    #[serde(default = "default_duplicate_billing_rate")]
    pub duplicate_billing: f64,

    /// Medical necessity abuse rate.
    #[serde(default = "default_med_necessity_rate")]
    pub medical_necessity_abuse: f64,
}

fn default_upcoding_rate() -> f64 {
    0.02
}

fn default_unbundling_rate() -> f64 {
    0.015
}

fn default_phantom_billing_rate() -> f64 {
    0.005
}

fn default_healthcare_kickback_rate() -> f64 {
    0.003
}

fn default_duplicate_billing_rate() -> f64 {
    0.008
}

fn default_med_necessity_rate() -> f64 {
    0.01
}

impl Default for HealthcareAnomalyRates {
    fn default() -> Self {
        Self {
            upcoding: default_upcoding_rate(),
            unbundling: default_unbundling_rate(),
            phantom_billing: default_phantom_billing_rate(),
            kickbacks: default_healthcare_kickback_rate(),
            duplicate_billing: default_duplicate_billing_rate(),
            medical_necessity_abuse: default_med_necessity_rate(),
        }
    }
}

/// Technology industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyConfig {
    /// Enable technology-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Revenue model type.
    #[serde(default = "default_revenue_model")]
    pub revenue_model: String,

    /// Subscription revenue percentage (for SaaS).
    #[serde(default = "default_subscription_pct")]
    pub subscription_revenue_pct: f64,

    /// License revenue percentage.
    #[serde(default = "default_license_pct")]
    pub license_revenue_pct: f64,

    /// Services revenue percentage.
    #[serde(default = "default_services_pct")]
    pub services_revenue_pct: f64,

    /// R&D capitalization settings.
    #[serde(default)]
    pub rd_capitalization: RdCapitalizationConfig,

    /// Technology anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: TechnologyAnomalyRates,
}

fn default_revenue_model() -> String {
    "saas".to_string()
}

fn default_subscription_pct() -> f64 {
    0.60
}

fn default_license_pct() -> f64 {
    0.25
}

fn default_services_pct() -> f64 {
    0.15
}

impl Default for TechnologyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            revenue_model: default_revenue_model(),
            subscription_revenue_pct: default_subscription_pct(),
            license_revenue_pct: default_license_pct(),
            services_revenue_pct: default_services_pct(),
            rd_capitalization: RdCapitalizationConfig::default(),
            anomaly_rates: TechnologyAnomalyRates::default(),
        }
    }
}

/// R&D capitalization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdCapitalizationConfig {
    /// Enable R&D capitalization.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Capitalization rate (0.0-1.0).
    #[serde(default = "default_cap_rate")]
    pub capitalization_rate: f64,

    /// Useful life in years.
    #[serde(default = "default_useful_life")]
    pub useful_life_years: u32,
}

fn default_cap_rate() -> f64 {
    0.30
}

fn default_useful_life() -> u32 {
    3
}

impl Default for RdCapitalizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capitalization_rate: default_cap_rate(),
            useful_life_years: default_useful_life(),
        }
    }
}

/// Technology anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyAnomalyRates {
    /// Premature revenue recognition rate.
    #[serde(default = "default_premature_rev_rate")]
    pub premature_revenue: f64,

    /// Side letter abuse rate.
    #[serde(default = "default_side_letter_rate")]
    pub side_letter_abuse: f64,

    /// Channel stuffing rate.
    #[serde(default = "default_channel_stuffing_rate")]
    pub channel_stuffing: f64,

    /// Improper capitalization rate.
    #[serde(default = "default_improper_cap_rate")]
    pub improper_capitalization: f64,
}

fn default_premature_rev_rate() -> f64 {
    0.015
}

fn default_side_letter_rate() -> f64 {
    0.008
}

fn default_channel_stuffing_rate() -> f64 {
    0.01
}

fn default_improper_cap_rate() -> f64 {
    0.012
}

impl Default for TechnologyAnomalyRates {
    fn default() -> Self {
        Self {
            premature_revenue: default_premature_rev_rate(),
            side_letter_abuse: default_side_letter_rate(),
            channel_stuffing: default_channel_stuffing_rate(),
            improper_capitalization: default_improper_cap_rate(),
        }
    }
}

/// Financial services industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialServicesConfig {
    /// Enable financial services-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Financial institution type.
    #[serde(default = "default_fi_type")]
    pub institution_type: String,

    /// Regulatory framework.
    #[serde(default = "default_fi_regulatory")]
    pub regulatory_framework: String,

    /// Financial services anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: FinancialServicesAnomalyRates,
}

fn default_fi_type() -> String {
    "commercial_bank".to_string()
}

fn default_fi_regulatory() -> String {
    "us_banking".to_string()
}

impl Default for FinancialServicesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            institution_type: default_fi_type(),
            regulatory_framework: default_fi_regulatory(),
            anomaly_rates: FinancialServicesAnomalyRates::default(),
        }
    }
}

/// Financial services anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialServicesAnomalyRates {
    /// Loan fraud rate.
    #[serde(default = "default_loan_fraud_rate")]
    pub loan_fraud: f64,

    /// Trading fraud rate.
    #[serde(default = "default_trading_fraud_rate")]
    pub trading_fraud: f64,

    /// Insurance fraud rate.
    #[serde(default = "default_insurance_fraud_rate")]
    pub insurance_fraud: f64,

    /// Account manipulation rate.
    #[serde(default = "default_account_manip_rate")]
    pub account_manipulation: f64,
}

fn default_loan_fraud_rate() -> f64 {
    0.01
}

fn default_trading_fraud_rate() -> f64 {
    0.008
}

fn default_insurance_fraud_rate() -> f64 {
    0.012
}

fn default_account_manip_rate() -> f64 {
    0.005
}

impl Default for FinancialServicesAnomalyRates {
    fn default() -> Self {
        Self {
            loan_fraud: default_loan_fraud_rate(),
            trading_fraud: default_trading_fraud_rate(),
            insurance_fraud: default_insurance_fraud_rate(),
            account_manipulation: default_account_manip_rate(),
        }
    }
}

/// Professional services industry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalServicesConfig {
    /// Enable professional services-specific generation.
    #[serde(default)]
    pub enabled: bool,

    /// Firm type.
    #[serde(default = "default_firm_type")]
    pub firm_type: String,

    /// Billing model.
    #[serde(default = "default_billing_model")]
    pub billing_model: String,

    /// Average hourly rate.
    #[serde(default = "default_hourly_rate")]
    pub avg_hourly_rate: f64,

    /// Trust account settings (for law firms).
    #[serde(default)]
    pub trust_accounting: TrustAccountingConfig,

    /// Professional services anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: ProfessionalServicesAnomalyRates,
}

fn default_firm_type() -> String {
    "consulting".to_string()
}

fn default_billing_model() -> String {
    "time_and_materials".to_string()
}

fn default_hourly_rate() -> f64 {
    250.0
}

impl Default for ProfessionalServicesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            firm_type: default_firm_type(),
            billing_model: default_billing_model(),
            avg_hourly_rate: default_hourly_rate(),
            trust_accounting: TrustAccountingConfig::default(),
            anomaly_rates: ProfessionalServicesAnomalyRates::default(),
        }
    }
}

/// Trust accounting configuration for law firms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAccountingConfig {
    /// Enable trust accounting.
    #[serde(default)]
    pub enabled: bool,

    /// Require three-way reconciliation.
    #[serde(default = "default_true")]
    pub require_three_way_reconciliation: bool,
}

impl Default for TrustAccountingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_three_way_reconciliation: true,
        }
    }
}

/// Professional services anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalServicesAnomalyRates {
    /// Time billing fraud rate.
    #[serde(default = "default_time_fraud_rate")]
    pub time_billing_fraud: f64,

    /// Expense report fraud rate.
    #[serde(default = "default_expense_fraud_rate")]
    pub expense_fraud: f64,

    /// Trust misappropriation rate.
    #[serde(default = "default_trust_misappropriation_rate")]
    pub trust_misappropriation: f64,
}

fn default_time_fraud_rate() -> f64 {
    0.02
}

fn default_expense_fraud_rate() -> f64 {
    0.015
}

fn default_trust_misappropriation_rate() -> f64 {
    0.003
}

impl Default for ProfessionalServicesAnomalyRates {
    fn default() -> Self {
        Self {
            time_billing_fraud: default_time_fraud_rate(),
            expense_fraud: default_expense_fraud_rate(),
            trust_misappropriation: default_trust_misappropriation_rate(),
        }
    }
}

/// Fingerprint privacy configuration for extraction and synthesis.
///
/// Controls the privacy parameters used when extracting fingerprints
/// from sensitive data. Supports predefined levels or custom (epsilon, delta) tuples.
///
/// ```yaml
/// fingerprint_privacy:
///   level: custom
///   epsilon: 0.5
///   delta: 1.0e-5
///   k_anonymity: 10
///   composition_method: renyi_dp
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintPrivacyConfig {
    /// Privacy level preset. Use "custom" for user-specified epsilon/delta.
    #[serde(default)]
    pub level: String,
    /// Custom epsilon value (only used when level = "custom").
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
    /// Custom delta value for (epsilon, delta)-DP (only used with RDP/zCDP).
    #[serde(default = "default_delta")]
    pub delta: f64,
    /// K-anonymity threshold.
    #[serde(default = "default_k_anonymity")]
    pub k_anonymity: u32,
    /// Composition method: "naive", "advanced", "renyi_dp", "zcdp".
    #[serde(default)]
    pub composition_method: String,
}

fn default_epsilon() -> f64 {
    1.0
}

fn default_delta() -> f64 {
    1e-5
}

fn default_k_anonymity() -> u32 {
    5
}

impl Default for FingerprintPrivacyConfig {
    fn default() -> Self {
        Self {
            level: "standard".to_string(),
            epsilon: default_epsilon(),
            delta: default_delta(),
            k_anonymity: default_k_anonymity(),
            composition_method: "naive".to_string(),
        }
    }
}

/// Quality gates configuration for pass/fail thresholds on generation runs.
///
/// ```yaml
/// quality_gates:
///   enabled: true
///   profile: strict  # strict, default, lenient, custom
///   fail_on_violation: true
///   custom_gates:
///     - name: benford_compliance
///       metric: benford_mad
///       threshold: 0.015
///       comparison: lte
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesSchemaConfig {
    /// Enable quality gate evaluation.
    #[serde(default)]
    pub enabled: bool,
    /// Gate profile: "strict", "default", "lenient", or "custom".
    #[serde(default = "default_gate_profile_name")]
    pub profile: String,
    /// Whether to fail the generation on gate violations.
    #[serde(default)]
    pub fail_on_violation: bool,
    /// Custom gate definitions (used when profile = "custom").
    #[serde(default)]
    pub custom_gates: Vec<QualityGateEntry>,
}

fn default_gate_profile_name() -> String {
    "default".to_string()
}

impl Default for QualityGatesSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            profile: default_gate_profile_name(),
            fail_on_violation: false,
            custom_gates: Vec::new(),
        }
    }
}

/// A single quality gate entry in configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateEntry {
    /// Gate name.
    pub name: String,
    /// Metric to check: benford_mad, balance_coherence, document_chain_integrity,
    /// correlation_preservation, temporal_consistency, privacy_mia_auc,
    /// completion_rate, duplicate_rate, referential_integrity, ic_match_rate.
    pub metric: String,
    /// Threshold value.
    pub threshold: f64,
    /// Upper threshold for "between" comparison.
    #[serde(default)]
    pub upper_threshold: Option<f64>,
    /// Comparison operator: "gte", "lte", "eq", "between".
    #[serde(default = "default_gate_comparison")]
    pub comparison: String,
}

fn default_gate_comparison() -> String {
    "gte".to_string()
}

/// Compliance configuration for regulatory requirements.
///
/// ```yaml
/// compliance:
///   content_marking:
///     enabled: true
///     format: embedded  # embedded, sidecar, both
///   article10_report: true
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceSchemaConfig {
    /// Synthetic content marking configuration (EU AI Act Article 50).
    #[serde(default)]
    pub content_marking: ContentMarkingSchemaConfig,
    /// Generate Article 10 data governance report.
    #[serde(default)]
    pub article10_report: bool,
    /// Certificate configuration for proving DP guarantees.
    #[serde(default)]
    pub certificates: CertificateSchemaConfig,
}

/// Configuration for synthetic data certificates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CertificateSchemaConfig {
    /// Whether certificate generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name for the signing key.
    #[serde(default)]
    pub signing_key_env: Option<String>,
    /// Whether to include quality metrics in the certificate.
    #[serde(default)]
    pub include_quality_metrics: bool,
}

/// Content marking configuration for synthetic data output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMarkingSchemaConfig {
    /// Whether content marking is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Marking format: "embedded", "sidecar", or "both".
    #[serde(default = "default_marking_format")]
    pub format: String,
}

fn default_marking_format() -> String {
    "embedded".to_string()
}

impl Default for ContentMarkingSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: default_marking_format(),
        }
    }
}

/// Webhook notification configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookSchemaConfig {
    /// Whether webhooks are enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Webhook endpoint configurations.
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpointConfig>,
}

/// Configuration for a single webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpointConfig {
    /// Target URL for the webhook.
    pub url: String,
    /// Event types this endpoint subscribes to.
    #[serde(default)]
    pub events: Vec<String>,
    /// Optional secret for HMAC-SHA256 signature.
    #[serde(default)]
    pub secret: Option<String>,
    /// Maximum retry attempts (default: 3).
    #[serde(default = "default_webhook_retries")]
    pub max_retries: u32,
    /// Timeout in seconds (default: 10).
    #[serde(default = "default_webhook_timeout")]
    pub timeout_secs: u64,
}

fn default_webhook_retries() -> u32 {
    3
}
fn default_webhook_timeout() -> u64 {
    10
}

// ===== Enterprise Process Chain Config Structs =====

// ----- Source-to-Pay (S2C/S2P) -----

/// Source-to-Pay configuration covering the entire sourcing lifecycle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceToPayConfig {
    /// Enable source-to-pay generation
    #[serde(default)]
    pub enabled: bool,
    /// Spend analysis configuration
    #[serde(default)]
    pub spend_analysis: SpendAnalysisConfig,
    /// Sourcing project configuration
    #[serde(default)]
    pub sourcing: SourcingConfig,
    /// Supplier qualification configuration
    #[serde(default)]
    pub qualification: QualificationConfig,
    /// RFx event configuration
    #[serde(default)]
    pub rfx: RfxConfig,
    /// Contract configuration
    #[serde(default)]
    pub contracts: ContractConfig,
    /// Catalog configuration
    #[serde(default)]
    pub catalog: CatalogConfig,
    /// Scorecard configuration
    #[serde(default)]
    pub scorecards: ScorecardConfig,
    /// P2P integration settings
    #[serde(default)]
    pub p2p_integration: P2PIntegrationConfig,
}

/// Spend analysis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendAnalysisConfig {
    /// HHI threshold for triggering sourcing project
    #[serde(default = "default_hhi_threshold")]
    pub hhi_threshold: f64,
    /// Target spend coverage under contracts
    #[serde(default = "default_contract_coverage_target")]
    pub contract_coverage_target: f64,
}

impl Default for SpendAnalysisConfig {
    fn default() -> Self {
        Self {
            hhi_threshold: default_hhi_threshold(),
            contract_coverage_target: default_contract_coverage_target(),
        }
    }
}

fn default_hhi_threshold() -> f64 {
    2500.0
}
fn default_contract_coverage_target() -> f64 {
    0.80
}

/// Sourcing project configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcingConfig {
    /// Number of sourcing projects per year
    #[serde(default = "default_sourcing_projects_per_year")]
    pub projects_per_year: u32,
    /// Months before expiry to trigger renewal project
    #[serde(default = "default_renewal_horizon_months")]
    pub renewal_horizon_months: u32,
    /// Average project duration in months
    #[serde(default = "default_project_duration_months")]
    pub project_duration_months: u32,
}

impl Default for SourcingConfig {
    fn default() -> Self {
        Self {
            projects_per_year: default_sourcing_projects_per_year(),
            renewal_horizon_months: default_renewal_horizon_months(),
            project_duration_months: default_project_duration_months(),
        }
    }
}

fn default_sourcing_projects_per_year() -> u32 {
    10
}
fn default_renewal_horizon_months() -> u32 {
    3
}
fn default_project_duration_months() -> u32 {
    4
}

/// Supplier qualification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationConfig {
    /// Pass rate for qualification
    #[serde(default = "default_qualification_pass_rate")]
    pub pass_rate: f64,
    /// Qualification validity in days
    #[serde(default = "default_qualification_validity_days")]
    pub validity_days: u32,
    /// Financial stability weight
    #[serde(default = "default_financial_weight")]
    pub financial_weight: f64,
    /// Quality management weight
    #[serde(default = "default_quality_weight")]
    pub quality_weight: f64,
    /// Delivery performance weight
    #[serde(default = "default_delivery_weight")]
    pub delivery_weight: f64,
    /// Compliance weight
    #[serde(default = "default_compliance_weight")]
    pub compliance_weight: f64,
}

impl Default for QualificationConfig {
    fn default() -> Self {
        Self {
            pass_rate: default_qualification_pass_rate(),
            validity_days: default_qualification_validity_days(),
            financial_weight: default_financial_weight(),
            quality_weight: default_quality_weight(),
            delivery_weight: default_delivery_weight(),
            compliance_weight: default_compliance_weight(),
        }
    }
}

fn default_qualification_pass_rate() -> f64 {
    0.75
}
fn default_qualification_validity_days() -> u32 {
    365
}
fn default_financial_weight() -> f64 {
    0.25
}
fn default_quality_weight() -> f64 {
    0.30
}
fn default_delivery_weight() -> f64 {
    0.25
}
fn default_compliance_weight() -> f64 {
    0.20
}

/// RFx event configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxConfig {
    /// Spend threshold above which RFI is required before RFP
    #[serde(default = "default_rfi_threshold")]
    pub rfi_threshold: f64,
    /// Minimum vendors invited per RFx
    #[serde(default = "default_min_invited_vendors")]
    pub min_invited_vendors: u32,
    /// Maximum vendors invited per RFx
    #[serde(default = "default_max_invited_vendors")]
    pub max_invited_vendors: u32,
    /// Response rate (% of invited vendors that submit bids)
    #[serde(default = "default_response_rate")]
    pub response_rate: f64,
    /// Default price weight in evaluation
    #[serde(default = "default_price_weight")]
    pub default_price_weight: f64,
    /// Default quality weight in evaluation
    #[serde(default = "default_rfx_quality_weight")]
    pub default_quality_weight: f64,
    /// Default delivery weight in evaluation
    #[serde(default = "default_rfx_delivery_weight")]
    pub default_delivery_weight: f64,
}

impl Default for RfxConfig {
    fn default() -> Self {
        Self {
            rfi_threshold: default_rfi_threshold(),
            min_invited_vendors: default_min_invited_vendors(),
            max_invited_vendors: default_max_invited_vendors(),
            response_rate: default_response_rate(),
            default_price_weight: default_price_weight(),
            default_quality_weight: default_rfx_quality_weight(),
            default_delivery_weight: default_rfx_delivery_weight(),
        }
    }
}

fn default_rfi_threshold() -> f64 {
    100_000.0
}
fn default_min_invited_vendors() -> u32 {
    3
}
fn default_max_invited_vendors() -> u32 {
    8
}
fn default_response_rate() -> f64 {
    0.70
}
fn default_price_weight() -> f64 {
    0.40
}
fn default_rfx_quality_weight() -> f64 {
    0.35
}
fn default_rfx_delivery_weight() -> f64 {
    0.25
}

/// Contract configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    /// Minimum contract duration in months
    #[serde(default = "default_min_contract_months")]
    pub min_duration_months: u32,
    /// Maximum contract duration in months
    #[serde(default = "default_max_contract_months")]
    pub max_duration_months: u32,
    /// Auto-renewal rate
    #[serde(default = "default_auto_renewal_rate")]
    pub auto_renewal_rate: f64,
    /// Amendment rate (% of contracts with at least one amendment)
    #[serde(default = "default_amendment_rate")]
    pub amendment_rate: f64,
    /// Distribution of contract types
    #[serde(default)]
    pub type_distribution: ContractTypeDistribution,
}

impl Default for ContractConfig {
    fn default() -> Self {
        Self {
            min_duration_months: default_min_contract_months(),
            max_duration_months: default_max_contract_months(),
            auto_renewal_rate: default_auto_renewal_rate(),
            amendment_rate: default_amendment_rate(),
            type_distribution: ContractTypeDistribution::default(),
        }
    }
}

fn default_min_contract_months() -> u32 {
    12
}
fn default_max_contract_months() -> u32 {
    36
}
fn default_auto_renewal_rate() -> f64 {
    0.40
}
fn default_amendment_rate() -> f64 {
    0.20
}

/// Distribution of contract types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTypeDistribution {
    /// Fixed price percentage
    #[serde(default = "default_fixed_price_pct")]
    pub fixed_price: f64,
    /// Blanket/framework percentage
    #[serde(default = "default_blanket_pct")]
    pub blanket: f64,
    /// Time and materials percentage
    #[serde(default = "default_time_materials_pct")]
    pub time_and_materials: f64,
    /// Service agreement percentage
    #[serde(default = "default_service_agreement_pct")]
    pub service_agreement: f64,
}

impl Default for ContractTypeDistribution {
    fn default() -> Self {
        Self {
            fixed_price: default_fixed_price_pct(),
            blanket: default_blanket_pct(),
            time_and_materials: default_time_materials_pct(),
            service_agreement: default_service_agreement_pct(),
        }
    }
}

fn default_fixed_price_pct() -> f64 {
    0.40
}
fn default_blanket_pct() -> f64 {
    0.30
}
fn default_time_materials_pct() -> f64 {
    0.15
}
fn default_service_agreement_pct() -> f64 {
    0.15
}

/// Catalog configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogConfig {
    /// Percentage of catalog items marked as preferred
    #[serde(default = "default_preferred_vendor_flag_rate")]
    pub preferred_vendor_flag_rate: f64,
    /// Rate of materials with multiple sources in catalog
    #[serde(default = "default_multi_source_rate")]
    pub multi_source_rate: f64,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            preferred_vendor_flag_rate: default_preferred_vendor_flag_rate(),
            multi_source_rate: default_multi_source_rate(),
        }
    }
}

fn default_preferred_vendor_flag_rate() -> f64 {
    0.70
}
fn default_multi_source_rate() -> f64 {
    0.25
}

/// Scorecard configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardConfig {
    /// Scorecard review frequency (quarterly, monthly)
    #[serde(default = "default_scorecard_frequency")]
    pub frequency: String,
    /// On-time delivery weight in overall score
    #[serde(default = "default_otd_weight")]
    pub on_time_delivery_weight: f64,
    /// Quality weight in overall score
    #[serde(default = "default_quality_score_weight")]
    pub quality_weight: f64,
    /// Price competitiveness weight
    #[serde(default = "default_price_score_weight")]
    pub price_weight: f64,
    /// Responsiveness weight
    #[serde(default = "default_responsiveness_weight")]
    pub responsiveness_weight: f64,
    /// Grade A threshold (score >= this)
    #[serde(default = "default_grade_a_threshold")]
    pub grade_a_threshold: f64,
    /// Grade B threshold
    #[serde(default = "default_grade_b_threshold")]
    pub grade_b_threshold: f64,
    /// Grade C threshold
    #[serde(default = "default_grade_c_threshold")]
    pub grade_c_threshold: f64,
}

impl Default for ScorecardConfig {
    fn default() -> Self {
        Self {
            frequency: default_scorecard_frequency(),
            on_time_delivery_weight: default_otd_weight(),
            quality_weight: default_quality_score_weight(),
            price_weight: default_price_score_weight(),
            responsiveness_weight: default_responsiveness_weight(),
            grade_a_threshold: default_grade_a_threshold(),
            grade_b_threshold: default_grade_b_threshold(),
            grade_c_threshold: default_grade_c_threshold(),
        }
    }
}

fn default_scorecard_frequency() -> String {
    "quarterly".to_string()
}
fn default_otd_weight() -> f64 {
    0.30
}
fn default_quality_score_weight() -> f64 {
    0.30
}
fn default_price_score_weight() -> f64 {
    0.25
}
fn default_responsiveness_weight() -> f64 {
    0.15
}
fn default_grade_a_threshold() -> f64 {
    90.0
}
fn default_grade_b_threshold() -> f64 {
    75.0
}
fn default_grade_c_threshold() -> f64 {
    60.0
}

/// P2P integration settings for contract enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PIntegrationConfig {
    /// Rate of off-contract (maverick) purchases
    #[serde(default = "default_off_contract_rate")]
    pub off_contract_rate: f64,
    /// Price tolerance for contract price validation
    #[serde(default = "default_price_tolerance")]
    pub price_tolerance: f64,
    /// Whether to enforce catalog ordering
    #[serde(default)]
    pub catalog_enforcement: bool,
}

impl Default for P2PIntegrationConfig {
    fn default() -> Self {
        Self {
            off_contract_rate: default_off_contract_rate(),
            price_tolerance: default_price_tolerance(),
            catalog_enforcement: false,
        }
    }
}

fn default_off_contract_rate() -> f64 {
    0.15
}
fn default_price_tolerance() -> f64 {
    0.02
}

// ----- Financial Reporting -----

/// Financial reporting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReportingConfig {
    /// Enable financial reporting generation
    #[serde(default)]
    pub enabled: bool,
    /// Generate balance sheet
    #[serde(default = "default_true")]
    pub generate_balance_sheet: bool,
    /// Generate income statement
    #[serde(default = "default_true")]
    pub generate_income_statement: bool,
    /// Generate cash flow statement
    #[serde(default = "default_true")]
    pub generate_cash_flow: bool,
    /// Generate changes in equity statement
    #[serde(default = "default_true")]
    pub generate_changes_in_equity: bool,
    /// Number of comparative periods
    #[serde(default = "default_comparative_periods")]
    pub comparative_periods: u32,
    /// Management KPIs configuration
    #[serde(default)]
    pub management_kpis: ManagementKpisConfig,
    /// Budget configuration
    #[serde(default)]
    pub budgets: BudgetConfig,
}

impl Default for FinancialReportingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_balance_sheet: true,
            generate_income_statement: true,
            generate_cash_flow: true,
            generate_changes_in_equity: true,
            comparative_periods: default_comparative_periods(),
            management_kpis: ManagementKpisConfig::default(),
            budgets: BudgetConfig::default(),
        }
    }
}

fn default_comparative_periods() -> u32 {
    1
}

/// Management KPIs configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagementKpisConfig {
    /// Enable KPI generation
    #[serde(default)]
    pub enabled: bool,
    /// KPI calculation frequency (monthly, quarterly)
    #[serde(default = "default_kpi_frequency")]
    pub frequency: String,
}

fn default_kpi_frequency() -> String {
    "monthly".to_string()
}

/// Budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Enable budget generation
    #[serde(default)]
    pub enabled: bool,
    /// Expected revenue growth rate for budgeting
    #[serde(default = "default_revenue_growth_rate")]
    pub revenue_growth_rate: f64,
    /// Expected expense inflation rate
    #[serde(default = "default_expense_inflation_rate")]
    pub expense_inflation_rate: f64,
    /// Random noise to add to budget vs actual
    #[serde(default = "default_variance_noise")]
    pub variance_noise: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            revenue_growth_rate: default_revenue_growth_rate(),
            expense_inflation_rate: default_expense_inflation_rate(),
            variance_noise: default_variance_noise(),
        }
    }
}

fn default_revenue_growth_rate() -> f64 {
    0.05
}
fn default_expense_inflation_rate() -> f64 {
    0.03
}
fn default_variance_noise() -> f64 {
    0.10
}

// ----- HR Configuration -----

/// HR (Hire-to-Retire) process configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HrConfig {
    /// Enable HR generation
    #[serde(default)]
    pub enabled: bool,
    /// Payroll configuration
    #[serde(default)]
    pub payroll: PayrollConfig,
    /// Time and attendance configuration
    #[serde(default)]
    pub time_attendance: TimeAttendanceConfig,
    /// Expense management configuration
    #[serde(default)]
    pub expenses: ExpenseConfig,
}

/// Payroll configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollConfig {
    /// Enable payroll generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Pay frequency (monthly, biweekly, weekly)
    #[serde(default = "default_pay_frequency")]
    pub pay_frequency: String,
    /// Salary ranges by job level
    #[serde(default)]
    pub salary_ranges: PayrollSalaryRanges,
    /// Effective tax rates
    #[serde(default)]
    pub tax_rates: PayrollTaxRates,
    /// Benefits enrollment rate
    #[serde(default = "default_benefits_enrollment_rate")]
    pub benefits_enrollment_rate: f64,
    /// Retirement plan participation rate
    #[serde(default = "default_retirement_participation_rate")]
    pub retirement_participation_rate: f64,
}

impl Default for PayrollConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pay_frequency: default_pay_frequency(),
            salary_ranges: PayrollSalaryRanges::default(),
            tax_rates: PayrollTaxRates::default(),
            benefits_enrollment_rate: default_benefits_enrollment_rate(),
            retirement_participation_rate: default_retirement_participation_rate(),
        }
    }
}

fn default_pay_frequency() -> String {
    "monthly".to_string()
}
fn default_benefits_enrollment_rate() -> f64 {
    0.60
}
fn default_retirement_participation_rate() -> f64 {
    0.45
}

/// Salary ranges by job level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollSalaryRanges {
    /// Staff level min/max
    #[serde(default = "default_staff_min")]
    pub staff_min: f64,
    #[serde(default = "default_staff_max")]
    pub staff_max: f64,
    /// Manager level min/max
    #[serde(default = "default_manager_min")]
    pub manager_min: f64,
    #[serde(default = "default_manager_max")]
    pub manager_max: f64,
    /// Director level min/max
    #[serde(default = "default_director_min")]
    pub director_min: f64,
    #[serde(default = "default_director_max")]
    pub director_max: f64,
    /// Executive level min/max
    #[serde(default = "default_executive_min")]
    pub executive_min: f64,
    #[serde(default = "default_executive_max")]
    pub executive_max: f64,
}

impl Default for PayrollSalaryRanges {
    fn default() -> Self {
        Self {
            staff_min: default_staff_min(),
            staff_max: default_staff_max(),
            manager_min: default_manager_min(),
            manager_max: default_manager_max(),
            director_min: default_director_min(),
            director_max: default_director_max(),
            executive_min: default_executive_min(),
            executive_max: default_executive_max(),
        }
    }
}

fn default_staff_min() -> f64 {
    50_000.0
}
fn default_staff_max() -> f64 {
    70_000.0
}
fn default_manager_min() -> f64 {
    80_000.0
}
fn default_manager_max() -> f64 {
    120_000.0
}
fn default_director_min() -> f64 {
    120_000.0
}
fn default_director_max() -> f64 {
    180_000.0
}
fn default_executive_min() -> f64 {
    180_000.0
}
fn default_executive_max() -> f64 {
    350_000.0
}

/// Effective tax rates for payroll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollTaxRates {
    /// Federal effective tax rate
    #[serde(default = "default_federal_rate")]
    pub federal_effective: f64,
    /// State effective tax rate
    #[serde(default = "default_state_rate")]
    pub state_effective: f64,
    /// FICA/social security rate
    #[serde(default = "default_fica_rate")]
    pub fica: f64,
}

impl Default for PayrollTaxRates {
    fn default() -> Self {
        Self {
            federal_effective: default_federal_rate(),
            state_effective: default_state_rate(),
            fica: default_fica_rate(),
        }
    }
}

fn default_federal_rate() -> f64 {
    0.22
}
fn default_state_rate() -> f64 {
    0.05
}
fn default_fica_rate() -> f64 {
    0.0765
}

/// Time and attendance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAttendanceConfig {
    /// Enable time tracking
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Overtime rate (% of employees with overtime in a period)
    #[serde(default = "default_overtime_rate")]
    pub overtime_rate: f64,
}

impl Default for TimeAttendanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            overtime_rate: default_overtime_rate(),
        }
    }
}

fn default_overtime_rate() -> f64 {
    0.10
}

/// Expense management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseConfig {
    /// Enable expense report generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Rate of employees submitting expenses per month
    #[serde(default = "default_expense_submission_rate")]
    pub submission_rate: f64,
    /// Rate of policy violations
    #[serde(default = "default_policy_violation_rate")]
    pub policy_violation_rate: f64,
}

impl Default for ExpenseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            submission_rate: default_expense_submission_rate(),
            policy_violation_rate: default_policy_violation_rate(),
        }
    }
}

fn default_expense_submission_rate() -> f64 {
    0.30
}
fn default_policy_violation_rate() -> f64 {
    0.08
}

// ----- Manufacturing Configuration -----

/// Manufacturing process configuration (production orders, WIP, routing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManufacturingProcessConfig {
    /// Enable manufacturing generation
    #[serde(default)]
    pub enabled: bool,
    /// Production order configuration
    #[serde(default)]
    pub production_orders: ProductionOrderConfig,
    /// Costing configuration
    #[serde(default)]
    pub costing: ManufacturingCostingConfig,
    /// Routing configuration
    #[serde(default)]
    pub routing: RoutingConfig,
}

/// Production order configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrderConfig {
    /// Orders per month
    #[serde(default = "default_prod_orders_per_month")]
    pub orders_per_month: u32,
    /// Average batch size
    #[serde(default = "default_prod_avg_batch_size")]
    pub avg_batch_size: u32,
    /// Yield rate
    #[serde(default = "default_prod_yield_rate")]
    pub yield_rate: f64,
    /// Make-to-order rate (vs make-to-stock)
    #[serde(default = "default_prod_make_to_order_rate")]
    pub make_to_order_rate: f64,
    /// Rework rate
    #[serde(default = "default_prod_rework_rate")]
    pub rework_rate: f64,
}

impl Default for ProductionOrderConfig {
    fn default() -> Self {
        Self {
            orders_per_month: default_prod_orders_per_month(),
            avg_batch_size: default_prod_avg_batch_size(),
            yield_rate: default_prod_yield_rate(),
            make_to_order_rate: default_prod_make_to_order_rate(),
            rework_rate: default_prod_rework_rate(),
        }
    }
}

fn default_prod_orders_per_month() -> u32 {
    50
}
fn default_prod_avg_batch_size() -> u32 {
    100
}
fn default_prod_yield_rate() -> f64 {
    0.97
}
fn default_prod_make_to_order_rate() -> f64 {
    0.20
}
fn default_prod_rework_rate() -> f64 {
    0.03
}

/// Manufacturing costing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingCostingConfig {
    /// Labor rate per hour
    #[serde(default = "default_labor_rate")]
    pub labor_rate_per_hour: f64,
    /// Overhead application rate (multiplier on direct labor)
    #[serde(default = "default_overhead_rate")]
    pub overhead_rate: f64,
    /// Standard cost update frequency
    #[serde(default = "default_cost_update_frequency")]
    pub standard_cost_update_frequency: String,
}

impl Default for ManufacturingCostingConfig {
    fn default() -> Self {
        Self {
            labor_rate_per_hour: default_labor_rate(),
            overhead_rate: default_overhead_rate(),
            standard_cost_update_frequency: default_cost_update_frequency(),
        }
    }
}

fn default_labor_rate() -> f64 {
    35.0
}
fn default_overhead_rate() -> f64 {
    1.50
}
fn default_cost_update_frequency() -> String {
    "quarterly".to_string()
}

/// Routing configuration for production operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Average number of operations per routing
    #[serde(default = "default_avg_operations")]
    pub avg_operations: u32,
    /// Average setup time in hours
    #[serde(default = "default_setup_time")]
    pub setup_time_hours: f64,
    /// Run time variation coefficient
    #[serde(default = "default_run_time_variation")]
    pub run_time_variation: f64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            avg_operations: default_avg_operations(),
            setup_time_hours: default_setup_time(),
            run_time_variation: default_run_time_variation(),
        }
    }
}

fn default_avg_operations() -> u32 {
    4
}
fn default_setup_time() -> f64 {
    1.5
}
fn default_run_time_variation() -> f64 {
    0.15
}

// ----- Sales Quote Configuration -----

/// Sales quote (quote-to-order) pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuoteConfig {
    /// Enable sales quote generation
    #[serde(default)]
    pub enabled: bool,
    /// Quotes per month
    #[serde(default = "default_quotes_per_month")]
    pub quotes_per_month: u32,
    /// Win rate (fraction of quotes that convert to orders)
    #[serde(default = "default_quote_win_rate")]
    pub win_rate: f64,
    /// Average quote validity in days
    #[serde(default = "default_quote_validity_days")]
    pub validity_days: u32,
}

impl Default for SalesQuoteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            quotes_per_month: default_quotes_per_month(),
            win_rate: default_quote_win_rate(),
            validity_days: default_quote_validity_days(),
        }
    }
}

fn default_quotes_per_month() -> u32 {
    30
}
fn default_quote_win_rate() -> f64 {
    0.35
}
fn default_quote_validity_days() -> u32 {
    30
}

// =============================================================================
// Tax Accounting Configuration
// =============================================================================

/// Tax accounting configuration.
///
/// Controls generation of tax-related data including VAT/GST, sales tax,
/// withholding tax, tax provisions, and payroll tax across multiple jurisdictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxConfig {
    /// Whether tax generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Tax jurisdiction configuration.
    #[serde(default)]
    pub jurisdictions: TaxJurisdictionConfig,
    /// VAT/GST configuration.
    #[serde(default)]
    pub vat_gst: VatGstConfig,
    /// Sales tax configuration.
    #[serde(default)]
    pub sales_tax: SalesTaxConfig,
    /// Withholding tax configuration.
    #[serde(default)]
    pub withholding: WithholdingTaxSchemaConfig,
    /// Tax provision configuration.
    #[serde(default)]
    pub provisions: TaxProvisionSchemaConfig,
    /// Payroll tax configuration.
    #[serde(default)]
    pub payroll_tax: PayrollTaxSchemaConfig,
    /// Anomaly injection rate for tax data (0.0 to 1.0).
    #[serde(default = "default_tax_anomaly_rate")]
    pub anomaly_rate: f64,
}

fn default_tax_anomaly_rate() -> f64 {
    0.03
}

impl Default for TaxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jurisdictions: TaxJurisdictionConfig::default(),
            vat_gst: VatGstConfig::default(),
            sales_tax: SalesTaxConfig::default(),
            withholding: WithholdingTaxSchemaConfig::default(),
            provisions: TaxProvisionSchemaConfig::default(),
            payroll_tax: PayrollTaxSchemaConfig::default(),
            anomaly_rate: default_tax_anomaly_rate(),
        }
    }
}

/// Tax jurisdiction configuration.
///
/// Specifies which countries and subnational jurisdictions to include
/// when generating tax data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxJurisdictionConfig {
    /// List of country codes to include (e.g., ["US", "DE", "GB"]).
    #[serde(default)]
    pub countries: Vec<String>,
    /// Whether to include subnational jurisdictions (e.g., US states, Canadian provinces).
    #[serde(default)]
    pub include_subnational: bool,
}

/// VAT/GST configuration.
///
/// Controls generation of Value Added Tax / Goods and Services Tax data,
/// including standard and reduced rates, exempt categories, and reverse charge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatGstConfig {
    /// Whether VAT/GST generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Standard VAT/GST rates by country code (e.g., {"DE": 0.19, "GB": 0.20}).
    #[serde(default)]
    pub standard_rates: std::collections::HashMap<String, f64>,
    /// Reduced VAT/GST rates by country code (e.g., {"DE": 0.07, "GB": 0.05}).
    #[serde(default)]
    pub reduced_rates: std::collections::HashMap<String, f64>,
    /// Categories exempt from VAT/GST (e.g., ["financial_services", "healthcare"]).
    #[serde(default)]
    pub exempt_categories: Vec<String>,
    /// Whether to apply reverse charge mechanism for cross-border B2B transactions.
    #[serde(default = "default_true")]
    pub reverse_charge: bool,
}

impl Default for VatGstConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            standard_rates: std::collections::HashMap::new(),
            reduced_rates: std::collections::HashMap::new(),
            exempt_categories: Vec::new(),
            reverse_charge: true,
        }
    }
}

/// Sales tax configuration.
///
/// Controls generation of US-style sales tax data including nexus determination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SalesTaxConfig {
    /// Whether sales tax generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// US states where the company has nexus (e.g., ["CA", "NY", "TX"]).
    #[serde(default)]
    pub nexus_states: Vec<String>,
}

/// Withholding tax configuration.
///
/// Controls generation of withholding tax data for cross-border payments,
/// including treaty network and rate overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithholdingTaxSchemaConfig {
    /// Whether withholding tax generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Whether to simulate a treaty network with reduced rates.
    #[serde(default = "default_true")]
    pub treaty_network: bool,
    /// Default withholding tax rate for non-treaty countries (0.0 to 1.0).
    #[serde(default = "default_withholding_rate")]
    pub default_rate: f64,
    /// Reduced withholding tax rate for treaty countries (0.0 to 1.0).
    #[serde(default = "default_treaty_reduced_rate")]
    pub treaty_reduced_rate: f64,
}

fn default_withholding_rate() -> f64 {
    0.30
}

fn default_treaty_reduced_rate() -> f64 {
    0.15
}

impl Default for WithholdingTaxSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            treaty_network: true,
            default_rate: default_withholding_rate(),
            treaty_reduced_rate: default_treaty_reduced_rate(),
        }
    }
}

/// Tax provision configuration.
///
/// Controls generation of tax provision data including statutory rates
/// and uncertain tax positions (ASC 740 / IAS 12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxProvisionSchemaConfig {
    /// Whether tax provision generation is enabled.
    /// Defaults to true when tax is enabled, as provisions are typically required.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Statutory corporate tax rate (0.0 to 1.0).
    #[serde(default = "default_statutory_rate")]
    pub statutory_rate: f64,
    /// Whether to generate uncertain tax positions (FIN 48 / IFRIC 23).
    #[serde(default = "default_true")]
    pub uncertain_positions: bool,
}

fn default_statutory_rate() -> f64 {
    0.21
}

impl Default for TaxProvisionSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            statutory_rate: default_statutory_rate(),
            uncertain_positions: true,
        }
    }
}

/// Payroll tax configuration.
///
/// Controls generation of payroll tax data (employer/employee contributions,
/// social security, Medicare, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PayrollTaxSchemaConfig {
    /// Whether payroll tax generation is enabled.
    #[serde(default)]
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Treasury & Cash Management Configuration
// ---------------------------------------------------------------------------

/// Treasury and cash management configuration.
///
/// Controls generation of cash positions, forecasts, pooling, hedging
/// instruments (ASC 815 / IFRS 9), debt instruments with covenants,
/// bank guarantees, and intercompany netting runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryConfig {
    /// Whether treasury generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Cash positioning configuration.
    #[serde(default)]
    pub cash_positioning: CashPositioningConfig,
    /// Cash forecasting configuration.
    #[serde(default)]
    pub cash_forecasting: CashForecastingConfig,
    /// Cash pooling configuration.
    #[serde(default)]
    pub cash_pooling: CashPoolingConfig,
    /// Hedging configuration (FX forwards, IR swaps, etc.).
    #[serde(default)]
    pub hedging: HedgingSchemaConfig,
    /// Debt instrument and covenant configuration.
    #[serde(default)]
    pub debt: DebtSchemaConfig,
    /// Intercompany netting configuration.
    #[serde(default)]
    pub netting: NettingSchemaConfig,
    /// Bank guarantee / letter of credit configuration.
    #[serde(default)]
    pub bank_guarantees: BankGuaranteeSchemaConfig,
    /// Anomaly injection rate for treasury data (0.0 to 1.0).
    #[serde(default = "default_treasury_anomaly_rate")]
    pub anomaly_rate: f64,
}

fn default_treasury_anomaly_rate() -> f64 {
    0.02
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cash_positioning: CashPositioningConfig::default(),
            cash_forecasting: CashForecastingConfig::default(),
            cash_pooling: CashPoolingConfig::default(),
            hedging: HedgingSchemaConfig::default(),
            debt: DebtSchemaConfig::default(),
            netting: NettingSchemaConfig::default(),
            bank_guarantees: BankGuaranteeSchemaConfig::default(),
            anomaly_rate: default_treasury_anomaly_rate(),
        }
    }
}

/// Cash positioning configuration.
///
/// Controls daily cash position generation per entity/bank account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPositioningConfig {
    /// Whether cash positioning is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Position generation frequency.
    #[serde(default = "default_cash_frequency")]
    pub frequency: String,
    /// Minimum cash balance policy threshold.
    #[serde(default = "default_minimum_balance_policy")]
    pub minimum_balance_policy: f64,
}

fn default_cash_frequency() -> String {
    "daily".to_string()
}

fn default_minimum_balance_policy() -> f64 {
    100_000.0
}

impl Default for CashPositioningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: default_cash_frequency(),
            minimum_balance_policy: default_minimum_balance_policy(),
        }
    }
}

/// Cash forecasting configuration.
///
/// Controls forward-looking cash forecast generation with probability-weighted items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastingConfig {
    /// Whether cash forecasting is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Number of days to forecast into the future.
    #[serde(default = "default_horizon_days")]
    pub horizon_days: u32,
    /// AR collection probability curve type ("aging" or "flat").
    #[serde(default = "default_ar_probability_curve")]
    pub ar_collection_probability_curve: String,
    /// Confidence interval for the forecast (0.0 to 1.0).
    #[serde(default = "default_confidence_interval")]
    pub confidence_interval: f64,
}

fn default_horizon_days() -> u32 {
    90
}

fn default_ar_probability_curve() -> String {
    "aging".to_string()
}

fn default_confidence_interval() -> f64 {
    0.90
}

impl Default for CashForecastingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            horizon_days: default_horizon_days(),
            ar_collection_probability_curve: default_ar_probability_curve(),
            confidence_interval: default_confidence_interval(),
        }
    }
}

/// Cash pooling configuration.
///
/// Controls cash pool structure generation (physical, notional, zero-balancing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPoolingConfig {
    /// Whether cash pooling is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Pool type: "physical_pooling", "notional_pooling", or "zero_balancing".
    #[serde(default = "default_pool_type")]
    pub pool_type: String,
    /// Time of day when sweeps occur (HH:MM format).
    #[serde(default = "default_sweep_time")]
    pub sweep_time: String,
}

fn default_pool_type() -> String {
    "zero_balancing".to_string()
}

fn default_sweep_time() -> String {
    "16:00".to_string()
}

impl Default for CashPoolingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pool_type: default_pool_type(),
            sweep_time: default_sweep_time(),
        }
    }
}

/// Hedging configuration.
///
/// Controls generation of hedging instruments and hedge relationship designations
/// under ASC 815 / IFRS 9.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingSchemaConfig {
    /// Whether hedging generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Target hedge ratio (0.0 to 1.0). Proportion of FX exposure to hedge.
    #[serde(default = "default_hedge_ratio")]
    pub hedge_ratio: f64,
    /// Types of instruments to generate (e.g., ["fx_forward", "interest_rate_swap"]).
    #[serde(default = "default_hedge_instruments")]
    pub instruments: Vec<String>,
    /// Whether to designate formal hedge accounting relationships.
    #[serde(default = "default_true")]
    pub hedge_accounting: bool,
    /// Effectiveness testing method: "dollar_offset", "regression", or "critical_terms".
    #[serde(default = "default_effectiveness_method")]
    pub effectiveness_method: String,
}

fn default_hedge_ratio() -> f64 {
    0.75
}

fn default_hedge_instruments() -> Vec<String> {
    vec!["fx_forward".to_string(), "interest_rate_swap".to_string()]
}

fn default_effectiveness_method() -> String {
    "regression".to_string()
}

impl Default for HedgingSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            hedge_ratio: default_hedge_ratio(),
            instruments: default_hedge_instruments(),
            hedge_accounting: true,
            effectiveness_method: default_effectiveness_method(),
        }
    }
}

/// Debt instrument configuration.
///
/// Controls generation of debt instruments (term loans, revolving credit, bonds)
/// with amortization schedules and financial covenants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtSchemaConfig {
    /// Whether debt instrument generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Debt instrument definitions.
    #[serde(default)]
    pub instruments: Vec<DebtInstrumentDef>,
    /// Covenant definitions.
    #[serde(default)]
    pub covenants: Vec<CovenantDef>,
}

impl Default for DebtSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            instruments: Vec::new(),
            covenants: Vec::new(),
        }
    }
}

/// Definition of a debt instrument in configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtInstrumentDef {
    /// Instrument type: "term_loan", "revolving_credit", "bond", "commercial_paper", "bridge_loan".
    #[serde(rename = "type")]
    pub instrument_type: String,
    /// Principal amount (for term loans, bonds).
    #[serde(default)]
    pub principal: Option<f64>,
    /// Interest rate (annual, as decimal fraction).
    #[serde(default)]
    pub rate: Option<f64>,
    /// Maturity in months.
    #[serde(default)]
    pub maturity_months: Option<u32>,
    /// Facility limit (for revolving credit).
    #[serde(default)]
    pub facility: Option<f64>,
}

/// Definition of a debt covenant in configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantDef {
    /// Covenant type: "debt_to_equity", "interest_coverage", "current_ratio",
    /// "net_worth", "debt_to_ebitda", "fixed_charge_coverage".
    #[serde(rename = "type")]
    pub covenant_type: String,
    /// Covenant threshold value.
    pub threshold: f64,
}

/// Intercompany netting configuration.
///
/// Controls generation of multilateral netting runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NettingSchemaConfig {
    /// Whether netting generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Netting cycle: "daily", "weekly", or "monthly".
    #[serde(default = "default_netting_cycle")]
    pub cycle: String,
}

fn default_netting_cycle() -> String {
    "monthly".to_string()
}

impl Default for NettingSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cycle: default_netting_cycle(),
        }
    }
}

/// Bank guarantee and letter of credit configuration.
///
/// Controls generation of bank guarantees, standby LCs, and performance bonds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankGuaranteeSchemaConfig {
    /// Whether bank guarantee generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Number of guarantees to generate.
    #[serde(default = "default_guarantee_count")]
    pub count: u32,
}

fn default_guarantee_count() -> u32 {
    5
}

impl Default for BankGuaranteeSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            count: default_guarantee_count(),
        }
    }
}

// ===========================================================================
// Project Accounting Configuration
// ===========================================================================

/// Project accounting configuration.
///
/// Controls generation of project cost lines, revenue recognition,
/// milestones, change orders, retainage, and earned value metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAccountingConfig {
    /// Whether project accounting is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Number of projects to generate.
    #[serde(default = "default_project_count")]
    pub project_count: u32,
    /// Distribution of project types (capital, internal, customer, r_and_d, maintenance, technology).
    #[serde(default)]
    pub project_types: ProjectTypeDistribution,
    /// WBS structure configuration.
    #[serde(default)]
    pub wbs: WbsSchemaConfig,
    /// Cost allocation rates (what % of source documents get project-tagged).
    #[serde(default)]
    pub cost_allocation: CostAllocationConfig,
    /// Revenue recognition configuration for project accounting.
    #[serde(default)]
    pub revenue_recognition: ProjectRevenueRecognitionConfig,
    /// Milestone configuration.
    #[serde(default)]
    pub milestones: MilestoneSchemaConfig,
    /// Change order configuration.
    #[serde(default)]
    pub change_orders: ChangeOrderSchemaConfig,
    /// Retainage configuration.
    #[serde(default)]
    pub retainage: RetainageSchemaConfig,
    /// Earned value management configuration.
    #[serde(default)]
    pub earned_value: EarnedValueSchemaConfig,
    /// Anomaly injection rate for project accounting data (0.0 to 1.0).
    #[serde(default = "default_project_anomaly_rate")]
    pub anomaly_rate: f64,
}

fn default_project_count() -> u32 {
    10
}

fn default_project_anomaly_rate() -> f64 {
    0.03
}

impl Default for ProjectAccountingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_count: default_project_count(),
            project_types: ProjectTypeDistribution::default(),
            wbs: WbsSchemaConfig::default(),
            cost_allocation: CostAllocationConfig::default(),
            revenue_recognition: ProjectRevenueRecognitionConfig::default(),
            milestones: MilestoneSchemaConfig::default(),
            change_orders: ChangeOrderSchemaConfig::default(),
            retainage: RetainageSchemaConfig::default(),
            earned_value: EarnedValueSchemaConfig::default(),
            anomaly_rate: default_project_anomaly_rate(),
        }
    }
}

/// Distribution of project types by weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeDistribution {
    /// Weight for capital projects (default 0.25).
    #[serde(default = "default_capital_weight")]
    pub capital: f64,
    /// Weight for internal projects (default 0.20).
    #[serde(default = "default_internal_weight")]
    pub internal: f64,
    /// Weight for customer projects (default 0.30).
    #[serde(default = "default_customer_weight")]
    pub customer: f64,
    /// Weight for R&D projects (default 0.10).
    #[serde(default = "default_rnd_weight")]
    pub r_and_d: f64,
    /// Weight for maintenance projects (default 0.10).
    #[serde(default = "default_maintenance_weight")]
    pub maintenance: f64,
    /// Weight for technology projects (default 0.05).
    #[serde(default = "default_technology_weight")]
    pub technology: f64,
}

fn default_capital_weight() -> f64 { 0.25 }
fn default_internal_weight() -> f64 { 0.20 }
fn default_customer_weight() -> f64 { 0.30 }
fn default_rnd_weight() -> f64 { 0.10 }
fn default_maintenance_weight() -> f64 { 0.10 }
fn default_technology_weight() -> f64 { 0.05 }

impl Default for ProjectTypeDistribution {
    fn default() -> Self {
        Self {
            capital: default_capital_weight(),
            internal: default_internal_weight(),
            customer: default_customer_weight(),
            r_and_d: default_rnd_weight(),
            maintenance: default_maintenance_weight(),
            technology: default_technology_weight(),
        }
    }
}

/// WBS structure configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbsSchemaConfig {
    /// Maximum depth of WBS hierarchy (default 3).
    #[serde(default = "default_wbs_max_depth")]
    pub max_depth: u32,
    /// Minimum elements per level-1 WBS (default 2).
    #[serde(default = "default_wbs_min_elements")]
    pub min_elements_per_level: u32,
    /// Maximum elements per level-1 WBS (default 6).
    #[serde(default = "default_wbs_max_elements")]
    pub max_elements_per_level: u32,
}

fn default_wbs_max_depth() -> u32 { 3 }
fn default_wbs_min_elements() -> u32 { 2 }
fn default_wbs_max_elements() -> u32 { 6 }

impl Default for WbsSchemaConfig {
    fn default() -> Self {
        Self {
            max_depth: default_wbs_max_depth(),
            min_elements_per_level: default_wbs_min_elements(),
            max_elements_per_level: default_wbs_max_elements(),
        }
    }
}

/// Cost allocation rates — what fraction of each document type gets linked to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAllocationConfig {
    /// Fraction of time entries assigned to projects (0.0 to 1.0).
    #[serde(default = "default_time_entry_rate")]
    pub time_entry_project_rate: f64,
    /// Fraction of expense reports assigned to projects (0.0 to 1.0).
    #[serde(default = "default_expense_rate")]
    pub expense_project_rate: f64,
    /// Fraction of purchase orders assigned to projects (0.0 to 1.0).
    #[serde(default = "default_po_rate")]
    pub purchase_order_project_rate: f64,
    /// Fraction of vendor invoices assigned to projects (0.0 to 1.0).
    #[serde(default = "default_vi_rate")]
    pub vendor_invoice_project_rate: f64,
}

fn default_time_entry_rate() -> f64 { 0.60 }
fn default_expense_rate() -> f64 { 0.30 }
fn default_po_rate() -> f64 { 0.40 }
fn default_vi_rate() -> f64 { 0.35 }

impl Default for CostAllocationConfig {
    fn default() -> Self {
        Self {
            time_entry_project_rate: default_time_entry_rate(),
            expense_project_rate: default_expense_rate(),
            purchase_order_project_rate: default_po_rate(),
            vendor_invoice_project_rate: default_vi_rate(),
        }
    }
}

/// Revenue recognition configuration for project accounting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRevenueRecognitionConfig {
    /// Whether revenue recognition is enabled for customer projects.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Default method: "percentage_of_completion", "completed_contract", "milestone_based".
    #[serde(default = "default_revenue_method")]
    pub method: String,
    /// Default completion measure: "cost_to_cost", "labor_hours", "physical_completion".
    #[serde(default = "default_completion_measure")]
    pub completion_measure: String,
    /// Average contract value for customer projects.
    #[serde(default = "default_avg_contract_value")]
    pub avg_contract_value: f64,
}

fn default_revenue_method() -> String { "percentage_of_completion".to_string() }
fn default_completion_measure() -> String { "cost_to_cost".to_string() }
fn default_avg_contract_value() -> f64 { 500_000.0 }

impl Default for ProjectRevenueRecognitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: default_revenue_method(),
            completion_measure: default_completion_measure(),
            avg_contract_value: default_avg_contract_value(),
        }
    }
}

/// Milestone configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSchemaConfig {
    /// Whether milestone generation is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Average number of milestones per project.
    #[serde(default = "default_milestones_per_project")]
    pub avg_per_project: u32,
    /// Fraction of milestones that are payment milestones (0.0 to 1.0).
    #[serde(default = "default_payment_milestone_rate")]
    pub payment_milestone_rate: f64,
}

fn default_milestones_per_project() -> u32 { 4 }
fn default_payment_milestone_rate() -> f64 { 0.50 }

impl Default for MilestoneSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            avg_per_project: default_milestones_per_project(),
            payment_milestone_rate: default_payment_milestone_rate(),
        }
    }
}

/// Change order configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeOrderSchemaConfig {
    /// Whether change order generation is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Probability that a project will have at least one change order (0.0 to 1.0).
    #[serde(default = "default_change_order_probability")]
    pub probability: f64,
    /// Maximum change orders per project.
    #[serde(default = "default_max_change_orders")]
    pub max_per_project: u32,
    /// Approval rate for change orders (0.0 to 1.0).
    #[serde(default = "default_change_order_approval_rate")]
    pub approval_rate: f64,
}

fn default_change_order_probability() -> f64 { 0.40 }
fn default_max_change_orders() -> u32 { 3 }
fn default_change_order_approval_rate() -> f64 { 0.75 }

impl Default for ChangeOrderSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            probability: default_change_order_probability(),
            max_per_project: default_max_change_orders(),
            approval_rate: default_change_order_approval_rate(),
        }
    }
}

/// Retainage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainageSchemaConfig {
    /// Whether retainage is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Default retainage percentage (0.0 to 1.0, e.g., 0.10 for 10%).
    #[serde(default = "default_retainage_pct")]
    pub default_percentage: f64,
}

fn default_retainage_pct() -> f64 { 0.10 }

impl Default for RetainageSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_percentage: default_retainage_pct(),
        }
    }
}

/// Earned value management (EVM) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedValueSchemaConfig {
    /// Whether EVM metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Measurement frequency: "weekly", "biweekly", "monthly".
    #[serde(default = "default_evm_frequency")]
    pub frequency: String,
}

fn default_evm_frequency() -> String { "monthly".to_string() }

impl Default for EarnedValueSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: default_evm_frequency(),
        }
    }
}

// =============================================================================
// ESG / Sustainability Configuration
// =============================================================================

/// Top-level ESG / sustainability reporting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgConfig {
    /// Whether ESG generation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Environmental metrics (emissions, energy, water, waste).
    #[serde(default)]
    pub environmental: EnvironmentalConfig,
    /// Social metrics (diversity, pay equity, safety).
    #[serde(default)]
    pub social: SocialConfig,
    /// Governance metrics (board composition, ethics, compliance).
    #[serde(default)]
    pub governance: GovernanceSchemaConfig,
    /// Supply-chain ESG assessment settings.
    #[serde(default)]
    pub supply_chain_esg: SupplyChainEsgConfig,
    /// ESG reporting / disclosure framework settings.
    #[serde(default)]
    pub reporting: EsgReportingConfig,
    /// Climate scenario analysis settings.
    #[serde(default)]
    pub climate_scenarios: ClimateScenarioConfig,
    /// Anomaly injection rate for ESG data (0.0 to 1.0).
    #[serde(default = "default_esg_anomaly_rate")]
    pub anomaly_rate: f64,
}

fn default_esg_anomaly_rate() -> f64 {
    0.02
}

impl Default for EsgConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            environmental: EnvironmentalConfig::default(),
            social: SocialConfig::default(),
            governance: GovernanceSchemaConfig::default(),
            supply_chain_esg: SupplyChainEsgConfig::default(),
            reporting: EsgReportingConfig::default(),
            climate_scenarios: ClimateScenarioConfig::default(),
            anomaly_rate: default_esg_anomaly_rate(),
        }
    }
}

/// Environmental metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConfig {
    /// Whether environmental metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Scope 1 (direct) emission generation settings.
    #[serde(default)]
    pub scope1: EmissionScopeConfig,
    /// Scope 2 (purchased energy) emission generation settings.
    #[serde(default)]
    pub scope2: EmissionScopeConfig,
    /// Scope 3 (value chain) emission generation settings.
    #[serde(default)]
    pub scope3: Scope3Config,
    /// Energy consumption tracking settings.
    #[serde(default)]
    pub energy: EnergySchemaConfig,
    /// Water usage tracking settings.
    #[serde(default)]
    pub water: WaterSchemaConfig,
    /// Waste management tracking settings.
    #[serde(default)]
    pub waste: WasteSchemaConfig,
}

impl Default for EnvironmentalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scope1: EmissionScopeConfig::default(),
            scope2: EmissionScopeConfig::default(),
            scope3: Scope3Config::default(),
            energy: EnergySchemaConfig::default(),
            water: WaterSchemaConfig::default(),
            waste: WasteSchemaConfig::default(),
        }
    }
}

/// Configuration for a single emission scope (Scope 1 or 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionScopeConfig {
    /// Whether this scope is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Emission factor region (e.g., "US", "EU", "global").
    #[serde(default = "default_emission_region")]
    pub factor_region: String,
}

fn default_emission_region() -> String {
    "US".to_string()
}

impl Default for EmissionScopeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            factor_region: default_emission_region(),
        }
    }
}

/// Scope 3 (value chain) emission configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope3Config {
    /// Whether Scope 3 emissions are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Categories to include (e.g., "purchased_goods", "business_travel", "commuting").
    #[serde(default = "default_scope3_categories")]
    pub categories: Vec<String>,
    /// Spend-based emission intensity (kg CO2e per USD).
    #[serde(default = "default_spend_intensity")]
    pub default_spend_intensity_kg_per_usd: f64,
}

fn default_scope3_categories() -> Vec<String> {
    vec![
        "purchased_goods".to_string(),
        "business_travel".to_string(),
        "employee_commuting".to_string(),
    ]
}

fn default_spend_intensity() -> f64 {
    0.5
}

impl Default for Scope3Config {
    fn default() -> Self {
        Self {
            enabled: true,
            categories: default_scope3_categories(),
            default_spend_intensity_kg_per_usd: default_spend_intensity(),
        }
    }
}

/// Energy consumption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergySchemaConfig {
    /// Whether energy consumption tracking is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Number of facilities to generate.
    #[serde(default = "default_facility_count")]
    pub facility_count: u32,
    /// Target percentage of energy from renewable sources (0.0 to 1.0).
    #[serde(default = "default_renewable_target")]
    pub renewable_target: f64,
}

fn default_facility_count() -> u32 {
    5
}

fn default_renewable_target() -> f64 {
    0.30
}

impl Default for EnergySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            facility_count: default_facility_count(),
            renewable_target: default_renewable_target(),
        }
    }
}

/// Water usage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterSchemaConfig {
    /// Whether water usage tracking is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Number of facilities with water tracking.
    #[serde(default = "default_water_facility_count")]
    pub facility_count: u32,
}

fn default_water_facility_count() -> u32 {
    3
}

impl Default for WaterSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            facility_count: default_water_facility_count(),
        }
    }
}

/// Waste management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteSchemaConfig {
    /// Whether waste tracking is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Target diversion rate (0.0 to 1.0).
    #[serde(default = "default_diversion_target")]
    pub diversion_target: f64,
}

fn default_diversion_target() -> f64 {
    0.50
}

impl Default for WasteSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            diversion_target: default_diversion_target(),
        }
    }
}

/// Social metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialConfig {
    /// Whether social metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Workforce diversity tracking settings.
    #[serde(default)]
    pub diversity: DiversitySchemaConfig,
    /// Pay equity analysis settings.
    #[serde(default)]
    pub pay_equity: PayEquitySchemaConfig,
    /// Safety incident and metrics settings.
    #[serde(default)]
    pub safety: SafetySchemaConfig,
}

impl Default for SocialConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            diversity: DiversitySchemaConfig::default(),
            pay_equity: PayEquitySchemaConfig::default(),
            safety: SafetySchemaConfig::default(),
        }
    }
}

/// Workforce diversity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversitySchemaConfig {
    /// Whether diversity metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Dimensions to track (e.g., "gender", "ethnicity", "age_group").
    #[serde(default = "default_diversity_dimensions")]
    pub dimensions: Vec<String>,
}

fn default_diversity_dimensions() -> Vec<String> {
    vec![
        "gender".to_string(),
        "ethnicity".to_string(),
        "age_group".to_string(),
    ]
}

impl Default for DiversitySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dimensions: default_diversity_dimensions(),
        }
    }
}

/// Pay equity analysis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayEquitySchemaConfig {
    /// Whether pay equity analysis is generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Target pay gap threshold for flagging (e.g., 0.05 = 5% gap).
    #[serde(default = "default_pay_gap_threshold")]
    pub gap_threshold: f64,
}

fn default_pay_gap_threshold() -> f64 {
    0.05
}

impl Default for PayEquitySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gap_threshold: default_pay_gap_threshold(),
        }
    }
}

/// Safety metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySchemaConfig {
    /// Whether safety metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Average annual recordable incidents per 200,000 hours.
    #[serde(default = "default_trir_target")]
    pub target_trir: f64,
    /// Number of safety incidents to generate.
    #[serde(default = "default_incident_count")]
    pub incident_count: u32,
}

fn default_trir_target() -> f64 {
    2.5
}

fn default_incident_count() -> u32 {
    20
}

impl Default for SafetySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_trir: default_trir_target(),
            incident_count: default_incident_count(),
        }
    }
}

/// Governance metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSchemaConfig {
    /// Whether governance metrics are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Number of board members.
    #[serde(default = "default_board_size")]
    pub board_size: u32,
    /// Target independent director ratio (0.0 to 1.0).
    #[serde(default = "default_independence_target")]
    pub independence_target: f64,
}

fn default_board_size() -> u32 {
    11
}

fn default_independence_target() -> f64 {
    0.67
}

impl Default for GovernanceSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            board_size: default_board_size(),
            independence_target: default_independence_target(),
        }
    }
}

/// Supply-chain ESG assessment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainEsgConfig {
    /// Whether supply chain ESG assessments are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Proportion of vendors to assess (0.0 to 1.0).
    #[serde(default = "default_assessment_coverage")]
    pub assessment_coverage: f64,
    /// High-risk country codes for automatic flagging.
    #[serde(default = "default_high_risk_countries")]
    pub high_risk_countries: Vec<String>,
}

fn default_assessment_coverage() -> f64 {
    0.80
}

fn default_high_risk_countries() -> Vec<String> {
    vec!["CN".to_string(), "BD".to_string(), "MM".to_string()]
}

impl Default for SupplyChainEsgConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            assessment_coverage: default_assessment_coverage(),
            high_risk_countries: default_high_risk_countries(),
        }
    }
}

/// ESG reporting / disclosure framework configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgReportingConfig {
    /// Whether ESG disclosures are generated.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Frameworks to generate disclosures for.
    #[serde(default = "default_esg_frameworks")]
    pub frameworks: Vec<String>,
    /// Whether materiality assessment is performed.
    #[serde(default = "default_true")]
    pub materiality_assessment: bool,
    /// Materiality threshold for impact dimension (0.0 to 1.0).
    #[serde(default = "default_materiality_threshold")]
    pub impact_threshold: f64,
    /// Materiality threshold for financial dimension (0.0 to 1.0).
    #[serde(default = "default_materiality_threshold")]
    pub financial_threshold: f64,
}

fn default_esg_frameworks() -> Vec<String> {
    vec!["GRI".to_string(), "ESRS".to_string()]
}

fn default_materiality_threshold() -> f64 {
    0.6
}

impl Default for EsgReportingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frameworks: default_esg_frameworks(),
            materiality_assessment: true,
            impact_threshold: default_materiality_threshold(),
            financial_threshold: default_materiality_threshold(),
        }
    }
}

/// Climate scenario analysis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateScenarioConfig {
    /// Whether climate scenario analysis is generated.
    #[serde(default)]
    pub enabled: bool,
    /// Scenarios to model (e.g., "net_zero_2050", "stated_policies", "current_trajectory").
    #[serde(default = "default_climate_scenarios")]
    pub scenarios: Vec<String>,
    /// Time horizons in years to project.
    #[serde(default = "default_time_horizons")]
    pub time_horizons: Vec<u32>,
}

fn default_climate_scenarios() -> Vec<String> {
    vec![
        "net_zero_2050".to_string(),
        "stated_policies".to_string(),
        "current_trajectory".to_string(),
    ]
}

fn default_time_horizons() -> Vec<u32> {
    vec![5, 10, 30]
}

impl Default for ClimateScenarioConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scenarios: default_climate_scenarios(),
            time_horizons: default_time_horizons(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::presets::demo_preset;

    // ==========================================================================
    // Serialization/Deserialization Tests
    // ==========================================================================

    #[test]
    fn test_config_yaml_roundtrip() {
        let config = demo_preset();
        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize to YAML");
        let deserialized: GeneratorConfig =
            serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

        assert_eq!(
            config.global.period_months,
            deserialized.global.period_months
        );
        assert_eq!(config.global.industry, deserialized.global.industry);
        assert_eq!(config.companies.len(), deserialized.companies.len());
        assert_eq!(config.companies[0].code, deserialized.companies[0].code);
    }

    #[test]
    fn test_config_json_roundtrip() {
        // Create a config without infinity values (JSON can't serialize f64::INFINITY)
        let mut config = demo_preset();
        // Replace infinity with a large but finite value for JSON compatibility
        config.master_data.employees.approval_limits.executive = 1e12;

        let json = serde_json::to_string(&config).expect("Failed to serialize to JSON");
        let deserialized: GeneratorConfig =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");

        assert_eq!(
            config.global.period_months,
            deserialized.global.period_months
        );
        assert_eq!(config.global.industry, deserialized.global.industry);
        assert_eq!(config.companies.len(), deserialized.companies.len());
    }

    #[test]
    fn test_transaction_volume_serialization() {
        // Test various transaction volumes serialize correctly
        let volumes = vec![
            (TransactionVolume::TenK, "ten_k"),
            (TransactionVolume::HundredK, "hundred_k"),
            (TransactionVolume::OneM, "one_m"),
            (TransactionVolume::TenM, "ten_m"),
            (TransactionVolume::HundredM, "hundred_m"),
        ];

        for (volume, expected_key) in volumes {
            let json = serde_json::to_string(&volume).expect("Failed to serialize");
            assert!(
                json.contains(expected_key),
                "Expected {} in JSON: {}",
                expected_key,
                json
            );
        }
    }

    #[test]
    fn test_transaction_volume_custom_serialization() {
        let volume = TransactionVolume::Custom(12345);
        let json = serde_json::to_string(&volume).expect("Failed to serialize");
        let deserialized: TransactionVolume =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.count(), 12345);
    }

    #[test]
    fn test_output_mode_serialization() {
        let modes = vec![
            OutputMode::Streaming,
            OutputMode::FlatFile,
            OutputMode::Both,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).expect("Failed to serialize");
            let deserialized: OutputMode =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", mode) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_file_format_serialization() {
        let formats = vec![
            FileFormat::Csv,
            FileFormat::Parquet,
            FileFormat::Json,
            FileFormat::JsonLines,
        ];

        for format in formats {
            let json = serde_json::to_string(&format).expect("Failed to serialize");
            let deserialized: FileFormat =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", format) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_compression_algorithm_serialization() {
        let algos = vec![
            CompressionAlgorithm::Gzip,
            CompressionAlgorithm::Zstd,
            CompressionAlgorithm::Lz4,
            CompressionAlgorithm::Snappy,
        ];

        for algo in algos {
            let json = serde_json::to_string(&algo).expect("Failed to serialize");
            let deserialized: CompressionAlgorithm =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", algo) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_transfer_pricing_method_serialization() {
        let methods = vec![
            TransferPricingMethod::CostPlus,
            TransferPricingMethod::ComparableUncontrolled,
            TransferPricingMethod::ResalePrice,
            TransferPricingMethod::TransactionalNetMargin,
            TransferPricingMethod::ProfitSplit,
        ];

        for method in methods {
            let json = serde_json::to_string(&method).expect("Failed to serialize");
            let deserialized: TransferPricingMethod =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", method) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_benford_exemption_serialization() {
        let exemptions = vec![
            BenfordExemption::Recurring,
            BenfordExemption::Payroll,
            BenfordExemption::FixedFees,
            BenfordExemption::RoundAmounts,
        ];

        for exemption in exemptions {
            let json = serde_json::to_string(&exemption).expect("Failed to serialize");
            let deserialized: BenfordExemption =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", exemption) == format!("{:?}", deserialized));
        }
    }

    // ==========================================================================
    // Default Value Tests
    // ==========================================================================

    #[test]
    fn test_global_config_defaults() {
        let yaml = r#"
            industry: manufacturing
            start_date: "2024-01-01"
            period_months: 6
        "#;
        let config: GlobalConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.group_currency, "USD");
        assert!(config.parallel);
        assert_eq!(config.worker_threads, 0);
        assert_eq!(config.memory_limit_mb, 0);
    }

    #[test]
    fn test_fraud_config_defaults() {
        let config = FraudConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.fraud_rate, 0.005);
        assert!(!config.clustering_enabled);
    }

    #[test]
    fn test_internal_controls_config_defaults() {
        let config = InternalControlsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.exception_rate, 0.02);
        assert_eq!(config.sod_violation_rate, 0.01);
        assert!(config.export_control_master_data);
        assert_eq!(config.sox_materiality_threshold, 10000.0);
        // COSO fields
        assert!(config.coso_enabled);
        assert!(!config.include_entity_level_controls);
        assert_eq!(config.target_maturity_level, "mixed");
    }

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();
        assert!(matches!(config.mode, OutputMode::FlatFile));
        assert_eq!(config.formats, vec![FileFormat::Parquet]);
        assert!(config.compression.enabled);
        assert!(matches!(
            config.compression.algorithm,
            CompressionAlgorithm::Zstd
        ));
        assert!(config.include_acdoca);
        assert!(!config.include_bseg);
        assert!(config.partition_by_period);
        assert!(!config.partition_by_company);
    }

    #[test]
    fn test_approval_config_defaults() {
        let config = ApprovalConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.auto_approve_threshold, 1000.0);
        assert_eq!(config.rejection_rate, 0.02);
        assert_eq!(config.revision_rate, 0.05);
        assert_eq!(config.average_approval_delay_hours, 4.0);
        assert_eq!(config.thresholds.len(), 4);
    }

    #[test]
    fn test_p2p_flow_config_defaults() {
        let config = P2PFlowConfig::default();
        assert!(config.enabled);
        assert_eq!(config.three_way_match_rate, 0.95);
        assert_eq!(config.partial_delivery_rate, 0.15);
        assert_eq!(config.average_po_to_gr_days, 14);
    }

    #[test]
    fn test_o2c_flow_config_defaults() {
        let config = O2CFlowConfig::default();
        assert!(config.enabled);
        assert_eq!(config.credit_check_failure_rate, 0.02);
        assert_eq!(config.return_rate, 0.03);
        assert_eq!(config.bad_debt_rate, 0.01);
    }

    #[test]
    fn test_balance_config_defaults() {
        let config = BalanceConfig::default();
        assert!(!config.generate_opening_balances);
        assert!(config.generate_trial_balances);
        assert_eq!(config.target_gross_margin, 0.35);
        assert!(config.validate_balance_equation);
        assert!(config.reconcile_subledgers);
    }

    // ==========================================================================
    // Partial Config Deserialization Tests
    // ==========================================================================

    #[test]
    fn test_partial_config_with_defaults() {
        // Minimal config that should use all defaults
        let yaml = r#"
            global:
              industry: manufacturing
              start_date: "2024-01-01"
              period_months: 3
            companies:
              - code: "TEST"
                name: "Test Company"
                currency: "USD"
                country: "US"
                annual_transaction_volume: ten_k
            chart_of_accounts:
              complexity: small
            output:
              output_directory: "./output"
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.global.period_months, 3);
        assert_eq!(config.companies.len(), 1);
        assert!(!config.fraud.enabled); // Default
        assert!(!config.internal_controls.enabled); // Default
    }

    #[test]
    fn test_config_with_fraud_enabled() {
        let yaml = r#"
            global:
              industry: retail
              start_date: "2024-01-01"
              period_months: 12
            companies:
              - code: "RETAIL"
                name: "Retail Co"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
            chart_of_accounts:
              complexity: medium
            output:
              output_directory: "./output"
            fraud:
              enabled: true
              fraud_rate: 0.05
              clustering_enabled: true
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.fraud.enabled);
        assert_eq!(config.fraud.fraud_rate, 0.05);
        assert!(config.fraud.clustering_enabled);
    }

    #[test]
    fn test_config_with_multiple_companies() {
        let yaml = r#"
            global:
              industry: manufacturing
              start_date: "2024-01-01"
              period_months: 6
            companies:
              - code: "HQ"
                name: "Headquarters"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
                volume_weight: 1.0
              - code: "EU"
                name: "European Subsidiary"
                currency: "EUR"
                country: "DE"
                annual_transaction_volume: hundred_k
                volume_weight: 0.5
              - code: "APAC"
                name: "Asia Pacific"
                currency: "JPY"
                country: "JP"
                annual_transaction_volume: ten_k
                volume_weight: 0.3
            chart_of_accounts:
              complexity: large
            output:
              output_directory: "./output"
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.companies.len(), 3);
        assert_eq!(config.companies[0].code, "HQ");
        assert_eq!(config.companies[1].currency, "EUR");
        assert_eq!(config.companies[2].volume_weight, 0.3);
    }

    #[test]
    fn test_intercompany_config() {
        let yaml = r#"
            enabled: true
            ic_transaction_rate: 0.20
            transfer_pricing_method: cost_plus
            markup_percent: 0.08
            generate_matched_pairs: true
            generate_eliminations: true
        "#;

        let config: IntercompanyConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.enabled);
        assert_eq!(config.ic_transaction_rate, 0.20);
        assert!(matches!(
            config.transfer_pricing_method,
            TransferPricingMethod::CostPlus
        ));
        assert_eq!(config.markup_percent, 0.08);
        assert!(config.generate_eliminations);
    }

    // ==========================================================================
    // Company Config Tests
    // ==========================================================================

    #[test]
    fn test_company_config_defaults() {
        let yaml = r#"
            code: "TEST"
            name: "Test Company"
            currency: "USD"
            country: "US"
            annual_transaction_volume: ten_k
        "#;

        let config: CompanyConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.fiscal_year_variant, "K4"); // Default
        assert_eq!(config.volume_weight, 1.0); // Default
    }

    // ==========================================================================
    // Chart of Accounts Config Tests
    // ==========================================================================

    #[test]
    fn test_coa_config_defaults() {
        let yaml = r#"
            complexity: medium
        "#;

        let config: ChartOfAccountsConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.industry_specific); // Default true
        assert!(config.custom_accounts.is_none());
        assert_eq!(config.min_hierarchy_depth, 2); // Default
        assert_eq!(config.max_hierarchy_depth, 5); // Default
    }

    // ==========================================================================
    // Accounting Standards Config Tests
    // ==========================================================================

    #[test]
    fn test_accounting_standards_config_defaults() {
        let config = AccountingStandardsConfig::default();
        assert!(!config.enabled);
        assert!(matches!(
            config.framework,
            AccountingFrameworkConfig::UsGaap
        ));
        assert!(!config.revenue_recognition.enabled);
        assert!(!config.leases.enabled);
        assert!(!config.fair_value.enabled);
        assert!(!config.impairment.enabled);
        assert!(!config.generate_differences);
    }

    #[test]
    fn test_accounting_standards_config_yaml() {
        let yaml = r#"
            enabled: true
            framework: ifrs
            revenue_recognition:
              enabled: true
              generate_contracts: true
              avg_obligations_per_contract: 2.5
              variable_consideration_rate: 0.20
              over_time_recognition_rate: 0.35
              contract_count: 150
            leases:
              enabled: true
              lease_count: 75
              finance_lease_percent: 0.25
              avg_lease_term_months: 48
            generate_differences: true
        "#;

        let config: AccountingStandardsConfig =
            serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.enabled);
        assert!(matches!(config.framework, AccountingFrameworkConfig::Ifrs));
        assert!(config.revenue_recognition.enabled);
        assert_eq!(config.revenue_recognition.contract_count, 150);
        assert_eq!(config.revenue_recognition.avg_obligations_per_contract, 2.5);
        assert!(config.leases.enabled);
        assert_eq!(config.leases.lease_count, 75);
        assert_eq!(config.leases.finance_lease_percent, 0.25);
        assert!(config.generate_differences);
    }

    #[test]
    fn test_accounting_framework_serialization() {
        let frameworks = [
            AccountingFrameworkConfig::UsGaap,
            AccountingFrameworkConfig::Ifrs,
            AccountingFrameworkConfig::DualReporting,
        ];

        for framework in frameworks {
            let json = serde_json::to_string(&framework).expect("Failed to serialize");
            let deserialized: AccountingFrameworkConfig =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", framework) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_revenue_recognition_config_defaults() {
        let config = RevenueRecognitionConfig::default();
        assert!(!config.enabled);
        assert!(config.generate_contracts);
        assert_eq!(config.avg_obligations_per_contract, 2.0);
        assert_eq!(config.variable_consideration_rate, 0.15);
        assert_eq!(config.over_time_recognition_rate, 0.30);
        assert_eq!(config.contract_count, 100);
    }

    #[test]
    fn test_lease_accounting_config_defaults() {
        let config = LeaseAccountingConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.lease_count, 50);
        assert_eq!(config.finance_lease_percent, 0.30);
        assert_eq!(config.avg_lease_term_months, 60);
        assert!(config.generate_amortization);
        assert_eq!(config.real_estate_percent, 0.40);
    }

    #[test]
    fn test_fair_value_config_defaults() {
        let config = FairValueConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.measurement_count, 25);
        assert_eq!(config.level1_percent, 0.40);
        assert_eq!(config.level2_percent, 0.35);
        assert_eq!(config.level3_percent, 0.25);
        assert!(!config.include_sensitivity_analysis);
    }

    #[test]
    fn test_impairment_config_defaults() {
        let config = ImpairmentConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.test_count, 15);
        assert_eq!(config.impairment_rate, 0.10);
        assert!(config.generate_projections);
        assert!(!config.include_goodwill);
    }

    // ==========================================================================
    // Audit Standards Config Tests
    // ==========================================================================

    #[test]
    fn test_audit_standards_config_defaults() {
        let config = AuditStandardsConfig::default();
        assert!(!config.enabled);
        assert!(!config.isa_compliance.enabled);
        assert!(!config.analytical_procedures.enabled);
        assert!(!config.confirmations.enabled);
        assert!(!config.opinion.enabled);
        assert!(!config.generate_audit_trail);
        assert!(!config.sox.enabled);
        assert!(!config.pcaob.enabled);
    }

    #[test]
    fn test_audit_standards_config_yaml() {
        let yaml = r#"
            enabled: true
            isa_compliance:
              enabled: true
              compliance_level: comprehensive
              generate_isa_mappings: true
              include_pcaob: true
              framework: dual
            analytical_procedures:
              enabled: true
              procedures_per_account: 5
              variance_probability: 0.25
            confirmations:
              enabled: true
              confirmation_count: 75
              positive_response_rate: 0.90
              exception_rate: 0.08
            opinion:
              enabled: true
              generate_kam: true
              average_kam_count: 4
            sox:
              enabled: true
              generate_302_certifications: true
              generate_404_assessments: true
              material_weakness_rate: 0.03
            pcaob:
              enabled: true
              is_pcaob_audit: true
              include_icfr_opinion: true
            generate_audit_trail: true
        "#;

        let config: AuditStandardsConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.enabled);
        assert!(config.isa_compliance.enabled);
        assert_eq!(config.isa_compliance.compliance_level, "comprehensive");
        assert!(config.isa_compliance.include_pcaob);
        assert_eq!(config.isa_compliance.framework, "dual");
        assert!(config.analytical_procedures.enabled);
        assert_eq!(config.analytical_procedures.procedures_per_account, 5);
        assert!(config.confirmations.enabled);
        assert_eq!(config.confirmations.confirmation_count, 75);
        assert!(config.opinion.enabled);
        assert_eq!(config.opinion.average_kam_count, 4);
        assert!(config.sox.enabled);
        assert!(config.sox.generate_302_certifications);
        assert_eq!(config.sox.material_weakness_rate, 0.03);
        assert!(config.pcaob.enabled);
        assert!(config.pcaob.is_pcaob_audit);
        assert!(config.pcaob.include_icfr_opinion);
        assert!(config.generate_audit_trail);
    }

    #[test]
    fn test_isa_compliance_config_defaults() {
        let config = IsaComplianceConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.compliance_level, "standard");
        assert!(config.generate_isa_mappings);
        assert!(config.generate_coverage_summary);
        assert!(!config.include_pcaob);
        assert_eq!(config.framework, "isa");
    }

    #[test]
    fn test_sox_compliance_config_defaults() {
        let config = SoxComplianceConfig::default();
        assert!(!config.enabled);
        assert!(config.generate_302_certifications);
        assert!(config.generate_404_assessments);
        assert_eq!(config.materiality_threshold, 10000.0);
        assert_eq!(config.material_weakness_rate, 0.02);
        assert_eq!(config.significant_deficiency_rate, 0.08);
    }

    #[test]
    fn test_pcaob_config_defaults() {
        let config = PcaobConfig::default();
        assert!(!config.enabled);
        assert!(!config.is_pcaob_audit);
        assert!(config.generate_cam);
        assert!(!config.include_icfr_opinion);
        assert!(!config.generate_standard_mappings);
    }

    #[test]
    fn test_config_with_standards_enabled() {
        let yaml = r#"
            global:
              industry: financial_services
              start_date: "2024-01-01"
              period_months: 12
            companies:
              - code: "BANK"
                name: "Test Bank"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
            chart_of_accounts:
              complexity: large
            output:
              output_directory: "./output"
            accounting_standards:
              enabled: true
              framework: us_gaap
              revenue_recognition:
                enabled: true
              leases:
                enabled: true
            audit_standards:
              enabled: true
              isa_compliance:
                enabled: true
              sox:
                enabled: true
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.accounting_standards.enabled);
        assert!(matches!(
            config.accounting_standards.framework,
            AccountingFrameworkConfig::UsGaap
        ));
        assert!(config.accounting_standards.revenue_recognition.enabled);
        assert!(config.accounting_standards.leases.enabled);
        assert!(config.audit_standards.enabled);
        assert!(config.audit_standards.isa_compliance.enabled);
        assert!(config.audit_standards.sox.enabled);
    }

    // ==========================================================================
    // Industry-Specific Config Tests
    // ==========================================================================

    #[test]
    fn test_industry_specific_config_defaults() {
        let config = IndustrySpecificConfig::default();
        assert!(!config.enabled);
        assert!(!config.manufacturing.enabled);
        assert!(!config.retail.enabled);
        assert!(!config.healthcare.enabled);
        assert!(!config.technology.enabled);
        assert!(!config.financial_services.enabled);
        assert!(!config.professional_services.enabled);
    }

    #[test]
    fn test_manufacturing_config_defaults() {
        let config = ManufacturingConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.bom_depth, 4);
        assert!(!config.just_in_time);
        assert_eq!(config.supplier_tiers, 2);
        assert_eq!(config.target_yield_rate, 0.97);
        assert_eq!(config.scrap_alert_threshold, 0.03);
    }

    #[test]
    fn test_retail_config_defaults() {
        let config = RetailConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.avg_daily_transactions, 500);
        assert!(config.loss_prevention);
        assert_eq!(config.shrinkage_rate, 0.015);
    }

    #[test]
    fn test_healthcare_config_defaults() {
        let config = HealthcareConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.facility_type, "hospital");
        assert_eq!(config.avg_daily_encounters, 150);
        assert!(config.compliance.hipaa);
        assert!(config.compliance.stark_law);
        assert!(config.coding_systems.icd10);
        assert!(config.coding_systems.cpt);
    }

    #[test]
    fn test_technology_config_defaults() {
        let config = TechnologyConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.revenue_model, "saas");
        assert_eq!(config.subscription_revenue_pct, 0.60);
        assert!(config.rd_capitalization.enabled);
    }

    #[test]
    fn test_config_with_industry_specific() {
        let yaml = r#"
            global:
              industry: healthcare
              start_date: "2024-01-01"
              period_months: 12
            companies:
              - code: "HOSP"
                name: "Test Hospital"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
            chart_of_accounts:
              complexity: medium
            output:
              output_directory: "./output"
            industry_specific:
              enabled: true
              healthcare:
                enabled: true
                facility_type: hospital
                payer_mix:
                  medicare: 0.45
                  medicaid: 0.15
                  commercial: 0.35
                  self_pay: 0.05
                coding_systems:
                  icd10: true
                  cpt: true
                  drg: true
                compliance:
                  hipaa: true
                  stark_law: true
                anomaly_rates:
                  upcoding: 0.03
                  unbundling: 0.02
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.industry_specific.enabled);
        assert!(config.industry_specific.healthcare.enabled);
        assert_eq!(
            config.industry_specific.healthcare.facility_type,
            "hospital"
        );
        assert_eq!(config.industry_specific.healthcare.payer_mix.medicare, 0.45);
        assert_eq!(config.industry_specific.healthcare.payer_mix.self_pay, 0.05);
        assert!(config.industry_specific.healthcare.coding_systems.icd10);
        assert!(config.industry_specific.healthcare.compliance.hipaa);
        assert_eq!(
            config.industry_specific.healthcare.anomaly_rates.upcoding,
            0.03
        );
    }

    #[test]
    fn test_config_with_manufacturing_specific() {
        let yaml = r#"
            global:
              industry: manufacturing
              start_date: "2024-01-01"
              period_months: 12
            companies:
              - code: "MFG"
                name: "Test Manufacturing"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
            chart_of_accounts:
              complexity: medium
            output:
              output_directory: "./output"
            industry_specific:
              enabled: true
              manufacturing:
                enabled: true
                bom_depth: 5
                just_in_time: true
                supplier_tiers: 3
                target_yield_rate: 0.98
                anomaly_rates:
                  yield_manipulation: 0.02
                  phantom_production: 0.01
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.industry_specific.enabled);
        assert!(config.industry_specific.manufacturing.enabled);
        assert_eq!(config.industry_specific.manufacturing.bom_depth, 5);
        assert!(config.industry_specific.manufacturing.just_in_time);
        assert_eq!(config.industry_specific.manufacturing.supplier_tiers, 3);
        assert_eq!(
            config.industry_specific.manufacturing.target_yield_rate,
            0.98
        );
        assert_eq!(
            config
                .industry_specific
                .manufacturing
                .anomaly_rates
                .yield_manipulation,
            0.02
        );
    }

    // ==========================================================================
    // Tax Configuration Tests
    // ==========================================================================

    #[test]
    fn test_tax_config_defaults() {
        let tax = TaxConfig::default();
        assert!(!tax.enabled);
        assert!(tax.jurisdictions.countries.is_empty());
        assert!(!tax.jurisdictions.include_subnational);
        assert!(!tax.vat_gst.enabled);
        assert!(tax.vat_gst.standard_rates.is_empty());
        assert!(tax.vat_gst.reduced_rates.is_empty());
        assert!(tax.vat_gst.exempt_categories.is_empty());
        assert!(tax.vat_gst.reverse_charge);
        assert!(!tax.sales_tax.enabled);
        assert!(tax.sales_tax.nexus_states.is_empty());
        assert!(!tax.withholding.enabled);
        assert!(tax.withholding.treaty_network);
        assert_eq!(tax.withholding.default_rate, 0.30);
        assert_eq!(tax.withholding.treaty_reduced_rate, 0.15);
        assert!(tax.provisions.enabled);
        assert_eq!(tax.provisions.statutory_rate, 0.21);
        assert!(tax.provisions.uncertain_positions);
        assert!(!tax.payroll_tax.enabled);
        assert_eq!(tax.anomaly_rate, 0.03);
    }

    #[test]
    fn test_tax_config_from_yaml() {
        let yaml = r#"
            global:
              seed: 42
              start_date: "2024-01-01"
              period_months: 12
              industry: retail
            companies:
              - code: C001
                name: Test Corp
                currency: USD
                country: US
                annual_transaction_volume: ten_k
            chart_of_accounts:
              complexity: small
            output:
              output_directory: ./output
            tax:
              enabled: true
              anomaly_rate: 0.05
              jurisdictions:
                countries: ["US", "DE", "GB"]
                include_subnational: true
              vat_gst:
                enabled: true
                standard_rates:
                  DE: 0.19
                  GB: 0.20
                reduced_rates:
                  DE: 0.07
                  GB: 0.05
                exempt_categories:
                  - financial_services
                  - healthcare
                reverse_charge: false
              sales_tax:
                enabled: true
                nexus_states: ["CA", "NY", "TX"]
              withholding:
                enabled: true
                treaty_network: false
                default_rate: 0.25
                treaty_reduced_rate: 0.10
              provisions:
                enabled: false
                statutory_rate: 0.28
                uncertain_positions: false
              payroll_tax:
                enabled: true
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.tax.enabled);
        assert_eq!(config.tax.anomaly_rate, 0.05);

        // Jurisdictions
        assert_eq!(config.tax.jurisdictions.countries.len(), 3);
        assert!(config
            .tax
            .jurisdictions
            .countries
            .contains(&"DE".to_string()));
        assert!(config.tax.jurisdictions.include_subnational);

        // VAT/GST
        assert!(config.tax.vat_gst.enabled);
        assert_eq!(config.tax.vat_gst.standard_rates.get("DE"), Some(&0.19));
        assert_eq!(config.tax.vat_gst.standard_rates.get("GB"), Some(&0.20));
        assert_eq!(config.tax.vat_gst.reduced_rates.get("DE"), Some(&0.07));
        assert_eq!(config.tax.vat_gst.exempt_categories.len(), 2);
        assert!(!config.tax.vat_gst.reverse_charge);

        // Sales tax
        assert!(config.tax.sales_tax.enabled);
        assert_eq!(config.tax.sales_tax.nexus_states.len(), 3);
        assert!(config
            .tax
            .sales_tax
            .nexus_states
            .contains(&"CA".to_string()));

        // Withholding
        assert!(config.tax.withholding.enabled);
        assert!(!config.tax.withholding.treaty_network);
        assert_eq!(config.tax.withholding.default_rate, 0.25);
        assert_eq!(config.tax.withholding.treaty_reduced_rate, 0.10);

        // Provisions
        assert!(!config.tax.provisions.enabled);
        assert_eq!(config.tax.provisions.statutory_rate, 0.28);
        assert!(!config.tax.provisions.uncertain_positions);

        // Payroll tax
        assert!(config.tax.payroll_tax.enabled);
    }

    #[test]
    fn test_generator_config_with_tax_default() {
        let yaml = r#"
            global:
              seed: 42
              start_date: "2024-01-01"
              period_months: 12
              industry: retail
            companies:
              - code: C001
                name: Test Corp
                currency: USD
                country: US
                annual_transaction_volume: ten_k
            chart_of_accounts:
              complexity: small
            output:
              output_directory: ./output
        "#;

        let config: GeneratorConfig =
            serde_yaml::from_str(yaml).expect("Failed to parse config without tax section");
        // Tax should be present with defaults when not specified in YAML
        assert!(!config.tax.enabled);
        assert!(config.tax.jurisdictions.countries.is_empty());
        assert_eq!(config.tax.anomaly_rate, 0.03);
        assert!(config.tax.provisions.enabled); // provisions default to enabled=true
        assert_eq!(config.tax.provisions.statutory_rate, 0.21);
    }
}
