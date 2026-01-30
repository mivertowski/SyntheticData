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
        }
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
}

impl Default for OcpmOutputConfig {
    fn default() -> Self {
        Self {
            ocel_json: true,
            ocel_xml: false,
            flattened_csv: true,
            event_object_csv: true,
            object_relationship_csv: true,
            variants_csv: true,
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

#[cfg(test)]
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
}
