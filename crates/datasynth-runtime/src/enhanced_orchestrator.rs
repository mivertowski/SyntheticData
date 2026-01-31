//! Enhanced generation orchestrator with full feature integration.
//!
//! This orchestrator coordinates all generation phases:
//! 1. Chart of Accounts generation
//! 2. Master data generation (vendors, customers, materials, assets, employees)
//! 3. Document flow generation (P2P, O2C) + subledger linking + OCPM events
//! 4. Journal entry generation
//! 5. Anomaly injection
//! 6. Balance validation
//! 7. Data quality injection
//! 8. Audit data generation (engagements, workpapers, evidence, risks, findings, judgments)
//! 9. Banking KYC/AML data generation (customers, accounts, transactions, typologies)
//! 10. Graph export (accounting network for ML training and network reconstruction)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use datasynth_banking::{
    models::{BankAccount, BankTransaction, BankingCustomer},
    BankingOrchestratorBuilder,
};
use datasynth_config::schema::GeneratorConfig;
use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::audit::{
    AuditEngagement, AuditEvidence, AuditFinding, ProfessionalJudgment, RiskAssessment, Workpaper,
};
use datasynth_core::models::subledger::ap::APInvoice;
use datasynth_core::models::subledger::ar::ARInvoice;
use datasynth_core::models::*;
use datasynth_core::{DegradationActions, DegradationLevel, ResourceGuard, ResourceGuardBuilder};
use datasynth_fingerprint::{
    io::FingerprintReader,
    models::Fingerprint,
    synthesis::{ConfigSynthesizer, CopulaGeneratorSpec, SynthesisOptions},
};
use datasynth_generators::{
    // Anomaly injection
    AnomalyInjector,
    AnomalyInjectorConfig,
    AssetGenerator,
    // Audit generators
    AuditEngagementGenerator,
    BalanceTrackerConfig,
    // Core generators
    ChartOfAccountsGenerator,
    CustomerGenerator,
    DataQualityConfig,
    // Data quality
    DataQualityInjector,
    DataQualityStats,
    // Document flow JE generator
    DocumentFlowJeConfig,
    DocumentFlowJeGenerator,
    // Subledger linker
    DocumentFlowLinker,
    EmployeeGenerator,
    EvidenceGenerator,
    FindingGenerator,
    JournalEntryGenerator,
    JudgmentGenerator,
    LatePaymentDistribution,
    MaterialGenerator,
    O2CDocumentChain,
    O2CGenerator,
    O2CGeneratorConfig,
    O2CPaymentBehavior,
    P2PDocumentChain,
    // Document flow generators
    P2PGenerator,
    P2PGeneratorConfig,
    P2PPaymentBehavior,
    RiskAssessmentGenerator,
    // Balance validation
    RunningBalanceTracker,
    ValidationError,
    // Master data generators
    VendorGenerator,
    WorkpaperGenerator,
};
use datasynth_graph::{
    PyGExportConfig, PyGExporter, TransactionGraphBuilder, TransactionGraphConfig,
};
use datasynth_ocpm::{
    EventLogMetadata, O2cDocuments, OcpmEventGenerator, OcpmEventLog, OcpmGeneratorConfig,
    P2pDocuments,
};

use datasynth_config::schema::{O2CFlowConfig, P2PFlowConfig};
use datasynth_core::models::documents::PaymentMethod;

// ============================================================================
// Configuration Conversion Functions
// ============================================================================

/// Convert P2P flow config from schema to generator config.
fn convert_p2p_config(schema_config: &P2PFlowConfig) -> P2PGeneratorConfig {
    let payment_behavior = &schema_config.payment_behavior;
    let late_dist = &payment_behavior.late_payment_days_distribution;

    P2PGeneratorConfig {
        three_way_match_rate: schema_config.three_way_match_rate,
        partial_delivery_rate: schema_config.partial_delivery_rate,
        over_delivery_rate: 0.02, // Not in schema, use default
        price_variance_rate: schema_config.price_variance_rate,
        max_price_variance_percent: schema_config.max_price_variance_percent,
        avg_days_po_to_gr: schema_config.average_po_to_gr_days,
        avg_days_gr_to_invoice: schema_config.average_gr_to_invoice_days,
        avg_days_invoice_to_payment: schema_config.average_invoice_to_payment_days,
        payment_method_distribution: vec![
            (PaymentMethod::BankTransfer, 0.60),
            (PaymentMethod::Check, 0.25),
            (PaymentMethod::Wire, 0.10),
            (PaymentMethod::CreditCard, 0.05),
        ],
        early_payment_discount_rate: 0.30, // Not in schema, use default
        payment_behavior: P2PPaymentBehavior {
            late_payment_rate: payment_behavior.late_payment_rate,
            late_payment_distribution: LatePaymentDistribution {
                slightly_late_1_to_7: late_dist.slightly_late_1_to_7,
                late_8_to_14: late_dist.late_8_to_14,
                very_late_15_to_30: late_dist.very_late_15_to_30,
                severely_late_31_to_60: late_dist.severely_late_31_to_60,
                extremely_late_over_60: late_dist.extremely_late_over_60,
            },
            partial_payment_rate: payment_behavior.partial_payment_rate,
            payment_correction_rate: payment_behavior.payment_correction_rate,
        },
    }
}

/// Convert O2C flow config from schema to generator config.
fn convert_o2c_config(schema_config: &O2CFlowConfig) -> O2CGeneratorConfig {
    let payment_behavior = &schema_config.payment_behavior;

    O2CGeneratorConfig {
        credit_check_failure_rate: schema_config.credit_check_failure_rate,
        partial_shipment_rate: schema_config.partial_shipment_rate,
        avg_days_so_to_delivery: schema_config.average_so_to_delivery_days,
        avg_days_delivery_to_invoice: schema_config.average_delivery_to_invoice_days,
        avg_days_invoice_to_payment: schema_config.average_invoice_to_receipt_days,
        late_payment_rate: 0.15, // Managed through dunning now
        bad_debt_rate: schema_config.bad_debt_rate,
        returns_rate: schema_config.return_rate,
        cash_discount_take_rate: schema_config.cash_discount.taken_rate,
        payment_method_distribution: vec![
            (PaymentMethod::BankTransfer, 0.50),
            (PaymentMethod::Check, 0.30),
            (PaymentMethod::Wire, 0.15),
            (PaymentMethod::CreditCard, 0.05),
        ],
        payment_behavior: O2CPaymentBehavior {
            partial_payment_rate: payment_behavior.partial_payments.rate,
            short_payment_rate: payment_behavior.short_payments.rate,
            max_short_percent: payment_behavior.short_payments.max_short_percent,
            on_account_rate: payment_behavior.on_account_payments.rate,
            payment_correction_rate: payment_behavior.payment_corrections.rate,
            avg_days_until_remainder: payment_behavior.partial_payments.avg_days_until_remainder,
        },
    }
}

/// Configuration for which generation phases to run.
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    /// Generate master data (vendors, customers, materials, assets, employees).
    pub generate_master_data: bool,
    /// Generate document flows (P2P, O2C).
    pub generate_document_flows: bool,
    /// Generate OCPM events from document flows.
    pub generate_ocpm_events: bool,
    /// Generate journal entries.
    pub generate_journal_entries: bool,
    /// Inject anomalies.
    pub inject_anomalies: bool,
    /// Inject data quality variations (typos, missing values, format variations).
    pub inject_data_quality: bool,
    /// Validate balance sheet equation after generation.
    pub validate_balances: bool,
    /// Show progress bars.
    pub show_progress: bool,
    /// Number of vendors to generate per company.
    pub vendors_per_company: usize,
    /// Number of customers to generate per company.
    pub customers_per_company: usize,
    /// Number of materials to generate per company.
    pub materials_per_company: usize,
    /// Number of assets to generate per company.
    pub assets_per_company: usize,
    /// Number of employees to generate per company.
    pub employees_per_company: usize,
    /// Number of P2P chains to generate.
    pub p2p_chains: usize,
    /// Number of O2C chains to generate.
    pub o2c_chains: usize,
    /// Generate audit data (engagements, workpapers, evidence, risks, findings, judgments).
    pub generate_audit: bool,
    /// Number of audit engagements to generate.
    pub audit_engagements: usize,
    /// Number of workpapers per engagement.
    pub workpapers_per_engagement: usize,
    /// Number of evidence items per workpaper.
    pub evidence_per_workpaper: usize,
    /// Number of risk assessments per engagement.
    pub risks_per_engagement: usize,
    /// Number of findings per engagement.
    pub findings_per_engagement: usize,
    /// Number of professional judgments per engagement.
    pub judgments_per_engagement: usize,
    /// Generate banking KYC/AML data (customers, accounts, transactions, typologies).
    pub generate_banking: bool,
    /// Generate graph exports (accounting network for ML training).
    pub generate_graph_export: bool,
}

impl Default for PhaseConfig {
    fn default() -> Self {
        Self {
            generate_master_data: true,
            generate_document_flows: true,
            generate_ocpm_events: false, // Off by default
            generate_journal_entries: true,
            inject_anomalies: false,
            inject_data_quality: false, // Off by default (to preserve clean test data)
            validate_balances: true,
            show_progress: true,
            vendors_per_company: 50,
            customers_per_company: 100,
            materials_per_company: 200,
            assets_per_company: 50,
            employees_per_company: 100,
            p2p_chains: 100,
            o2c_chains: 100,
            generate_audit: false, // Off by default
            audit_engagements: 5,
            workpapers_per_engagement: 20,
            evidence_per_workpaper: 5,
            risks_per_engagement: 15,
            findings_per_engagement: 8,
            judgments_per_engagement: 10,
            generate_banking: false,      // Off by default
            generate_graph_export: false, // Off by default
        }
    }
}

/// Master data snapshot containing all generated entities.
#[derive(Debug, Clone, Default)]
pub struct MasterDataSnapshot {
    /// Generated vendors.
    pub vendors: Vec<Vendor>,
    /// Generated customers.
    pub customers: Vec<Customer>,
    /// Generated materials.
    pub materials: Vec<Material>,
    /// Generated fixed assets.
    pub assets: Vec<FixedAsset>,
    /// Generated employees.
    pub employees: Vec<Employee>,
}

/// Document flow snapshot containing all generated document chains.
#[derive(Debug, Clone, Default)]
pub struct DocumentFlowSnapshot {
    /// P2P document chains.
    pub p2p_chains: Vec<P2PDocumentChain>,
    /// O2C document chains.
    pub o2c_chains: Vec<O2CDocumentChain>,
    /// All purchase orders (flattened).
    pub purchase_orders: Vec<documents::PurchaseOrder>,
    /// All goods receipts (flattened).
    pub goods_receipts: Vec<documents::GoodsReceipt>,
    /// All vendor invoices (flattened).
    pub vendor_invoices: Vec<documents::VendorInvoice>,
    /// All sales orders (flattened).
    pub sales_orders: Vec<documents::SalesOrder>,
    /// All deliveries (flattened).
    pub deliveries: Vec<documents::Delivery>,
    /// All customer invoices (flattened).
    pub customer_invoices: Vec<documents::CustomerInvoice>,
    /// All payments (flattened).
    pub payments: Vec<documents::Payment>,
}

/// Subledger snapshot containing generated subledger records.
#[derive(Debug, Clone, Default)]
pub struct SubledgerSnapshot {
    /// AP invoices linked from document flow vendor invoices.
    pub ap_invoices: Vec<APInvoice>,
    /// AR invoices linked from document flow customer invoices.
    pub ar_invoices: Vec<ARInvoice>,
}

/// OCPM snapshot containing generated OCPM event log data.
#[derive(Debug, Clone, Default)]
pub struct OcpmSnapshot {
    /// OCPM event log (if generated)
    pub event_log: Option<OcpmEventLog>,
    /// Number of events generated
    pub event_count: usize,
    /// Number of objects generated
    pub object_count: usize,
    /// Number of cases generated
    pub case_count: usize,
}

/// Audit data snapshot containing all generated audit-related entities.
#[derive(Debug, Clone, Default)]
pub struct AuditSnapshot {
    /// Audit engagements per ISA 210/220.
    pub engagements: Vec<AuditEngagement>,
    /// Workpapers per ISA 230.
    pub workpapers: Vec<Workpaper>,
    /// Audit evidence per ISA 500.
    pub evidence: Vec<AuditEvidence>,
    /// Risk assessments per ISA 315/330.
    pub risk_assessments: Vec<RiskAssessment>,
    /// Audit findings per ISA 265.
    pub findings: Vec<AuditFinding>,
    /// Professional judgments per ISA 200.
    pub judgments: Vec<ProfessionalJudgment>,
}

/// Banking KYC/AML data snapshot containing all generated banking entities.
#[derive(Debug, Clone, Default)]
pub struct BankingSnapshot {
    /// Banking customers (retail, business, trust).
    pub customers: Vec<BankingCustomer>,
    /// Bank accounts.
    pub accounts: Vec<BankAccount>,
    /// Bank transactions with AML labels.
    pub transactions: Vec<BankTransaction>,
    /// Number of suspicious transactions.
    pub suspicious_count: usize,
    /// Number of AML scenarios generated.
    pub scenario_count: usize,
}

/// Graph export snapshot containing exported graph metadata.
#[derive(Debug, Clone, Default)]
pub struct GraphExportSnapshot {
    /// Whether graph export was performed.
    pub exported: bool,
    /// Number of graphs exported.
    pub graph_count: usize,
    /// Exported graph metadata (by format name).
    pub exports: HashMap<String, GraphExportInfo>,
}

/// Information about an exported graph.
#[derive(Debug, Clone)]
pub struct GraphExportInfo {
    /// Graph name.
    pub name: String,
    /// Export format (pytorch_geometric, neo4j, dgl).
    pub format: String,
    /// Output directory path.
    pub output_path: PathBuf,
    /// Number of nodes.
    pub node_count: usize,
    /// Number of edges.
    pub edge_count: usize,
}

/// Anomaly labels generated during injection.
#[derive(Debug, Clone, Default)]
pub struct AnomalyLabels {
    /// All anomaly labels.
    pub labels: Vec<LabeledAnomaly>,
    /// Summary statistics.
    pub summary: Option<AnomalySummary>,
    /// Count by anomaly type.
    pub by_type: HashMap<String, usize>,
}

/// Balance validation results from running balance tracker.
#[derive(Debug, Clone, Default)]
pub struct BalanceValidationResult {
    /// Whether validation was performed.
    pub validated: bool,
    /// Whether balance sheet equation is satisfied.
    pub is_balanced: bool,
    /// Number of entries processed.
    pub entries_processed: u64,
    /// Total debits across all entries.
    pub total_debits: rust_decimal::Decimal,
    /// Total credits across all entries.
    pub total_credits: rust_decimal::Decimal,
    /// Number of accounts tracked.
    pub accounts_tracked: usize,
    /// Number of companies tracked.
    pub companies_tracked: usize,
    /// Validation errors encountered.
    pub validation_errors: Vec<ValidationError>,
    /// Whether any unbalanced entries were found.
    pub has_unbalanced_entries: bool,
}

/// Complete result of enhanced generation run.
#[derive(Debug)]
pub struct EnhancedGenerationResult {
    /// Generated chart of accounts.
    pub chart_of_accounts: ChartOfAccounts,
    /// Master data snapshot.
    pub master_data: MasterDataSnapshot,
    /// Document flow snapshot.
    pub document_flows: DocumentFlowSnapshot,
    /// Subledger snapshot (linked from document flows).
    pub subledger: SubledgerSnapshot,
    /// OCPM event log snapshot (if OCPM generation enabled).
    pub ocpm: OcpmSnapshot,
    /// Audit data snapshot (if audit generation enabled).
    pub audit: AuditSnapshot,
    /// Banking KYC/AML data snapshot (if banking generation enabled).
    pub banking: BankingSnapshot,
    /// Graph export snapshot (if graph export enabled).
    pub graph_export: GraphExportSnapshot,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
    /// Anomaly labels (if injection enabled).
    pub anomaly_labels: AnomalyLabels,
    /// Balance validation results (if validation enabled).
    pub balance_validation: BalanceValidationResult,
    /// Data quality statistics (if injection enabled).
    pub data_quality_stats: DataQualityStats,
    /// Generation statistics.
    pub statistics: EnhancedGenerationStatistics,
}

/// Enhanced statistics about a generation run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnhancedGenerationStatistics {
    /// Total journal entries generated.
    pub total_entries: u64,
    /// Total line items generated.
    pub total_line_items: u64,
    /// Number of accounts in CoA.
    pub accounts_count: usize,
    /// Number of companies.
    pub companies_count: usize,
    /// Period in months.
    pub period_months: u32,
    /// Master data counts.
    pub vendor_count: usize,
    pub customer_count: usize,
    pub material_count: usize,
    pub asset_count: usize,
    pub employee_count: usize,
    /// Document flow counts.
    pub p2p_chain_count: usize,
    pub o2c_chain_count: usize,
    /// Subledger counts.
    pub ap_invoice_count: usize,
    pub ar_invoice_count: usize,
    /// OCPM counts.
    pub ocpm_event_count: usize,
    pub ocpm_object_count: usize,
    pub ocpm_case_count: usize,
    /// Audit counts.
    pub audit_engagement_count: usize,
    pub audit_workpaper_count: usize,
    pub audit_evidence_count: usize,
    pub audit_risk_count: usize,
    pub audit_finding_count: usize,
    pub audit_judgment_count: usize,
    /// Anomaly counts.
    pub anomalies_injected: usize,
    /// Data quality issue counts.
    pub data_quality_issues: usize,
    /// Banking counts.
    pub banking_customer_count: usize,
    pub banking_account_count: usize,
    pub banking_transaction_count: usize,
    pub banking_suspicious_count: usize,
    /// Graph export counts.
    pub graph_export_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
}

/// Enhanced orchestrator with full feature integration.
pub struct EnhancedOrchestrator {
    config: GeneratorConfig,
    phase_config: PhaseConfig,
    coa: Option<Arc<ChartOfAccounts>>,
    master_data: MasterDataSnapshot,
    seed: u64,
    multi_progress: Option<MultiProgress>,
    /// Resource guard for memory, disk, and CPU monitoring
    resource_guard: ResourceGuard,
    /// Output path for disk space monitoring
    output_path: Option<PathBuf>,
    /// Copula generators for preserving correlations (from fingerprint)
    copula_generators: Vec<CopulaGeneratorSpec>,
}

impl EnhancedOrchestrator {
    /// Create a new enhanced orchestrator.
    pub fn new(config: GeneratorConfig, phase_config: PhaseConfig) -> SynthResult<Self> {
        datasynth_config::validate_config(&config)?;

        let seed = config.global.seed.unwrap_or_else(rand::random);

        // Build resource guard from config
        let resource_guard = Self::build_resource_guard(&config, None);

        Ok(Self {
            config,
            phase_config,
            coa: None,
            master_data: MasterDataSnapshot::default(),
            seed,
            multi_progress: None,
            resource_guard,
            output_path: None,
            copula_generators: Vec::new(),
        })
    }

    /// Create with default phase config.
    pub fn with_defaults(config: GeneratorConfig) -> SynthResult<Self> {
        Self::new(config, PhaseConfig::default())
    }

    /// Enable/disable progress bars.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.phase_config.show_progress = show;
        if show {
            self.multi_progress = Some(MultiProgress::new());
        }
        self
    }

    /// Set the output path for disk space monitoring.
    pub fn with_output_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        let path = path.into();
        self.output_path = Some(path.clone());
        // Rebuild resource guard with the output path
        self.resource_guard = Self::build_resource_guard(&self.config, Some(path));
        self
    }

    /// Check if copula generators are available.
    ///
    /// Returns true if the orchestrator has copula generators for preserving
    /// correlations (typically from fingerprint-based generation).
    pub fn has_copulas(&self) -> bool {
        !self.copula_generators.is_empty()
    }

    /// Get the copula generators.
    ///
    /// Returns a reference to the copula generators for use during generation.
    /// These can be used to generate correlated samples that preserve the
    /// statistical relationships from the source data.
    pub fn copulas(&self) -> &[CopulaGeneratorSpec] {
        &self.copula_generators
    }

    /// Get a mutable reference to the copula generators.
    ///
    /// Allows generators to sample from copulas during data generation.
    pub fn copulas_mut(&mut self) -> &mut [CopulaGeneratorSpec] {
        &mut self.copula_generators
    }

    /// Sample correlated values from a named copula.
    ///
    /// Returns None if the copula doesn't exist.
    pub fn sample_from_copula(&mut self, copula_name: &str) -> Option<Vec<f64>> {
        self.copula_generators
            .iter_mut()
            .find(|c| c.name == copula_name)
            .map(|c| c.generator.sample())
    }

    /// Create an orchestrator from a fingerprint file.
    ///
    /// This reads the fingerprint, synthesizes a GeneratorConfig from it,
    /// and creates an orchestrator configured to generate data matching
    /// the statistical properties of the original data.
    ///
    /// # Arguments
    /// * `fingerprint_path` - Path to the .dsf fingerprint file
    /// * `phase_config` - Phase configuration for generation
    /// * `scale` - Scale factor for row counts (1.0 = same as original)
    ///
    /// # Example
    /// ```no_run
    /// use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
    /// use std::path::Path;
    ///
    /// let orchestrator = EnhancedOrchestrator::from_fingerprint(
    ///     Path::new("fingerprint.dsf"),
    ///     PhaseConfig::default(),
    ///     1.0,
    /// ).unwrap();
    /// ```
    pub fn from_fingerprint(
        fingerprint_path: &std::path::Path,
        phase_config: PhaseConfig,
        scale: f64,
    ) -> SynthResult<Self> {
        info!("Loading fingerprint from: {}", fingerprint_path.display());

        // Read the fingerprint
        let reader = FingerprintReader::new();
        let fingerprint = reader
            .read_from_file(fingerprint_path)
            .map_err(|e| SynthError::config(format!("Failed to read fingerprint: {}", e)))?;

        Self::from_fingerprint_data(fingerprint, phase_config, scale)
    }

    /// Create an orchestrator from a loaded fingerprint.
    ///
    /// # Arguments
    /// * `fingerprint` - The loaded fingerprint
    /// * `phase_config` - Phase configuration for generation
    /// * `scale` - Scale factor for row counts (1.0 = same as original)
    pub fn from_fingerprint_data(
        fingerprint: Fingerprint,
        phase_config: PhaseConfig,
        scale: f64,
    ) -> SynthResult<Self> {
        info!(
            "Synthesizing config from fingerprint (version: {}, tables: {})",
            fingerprint.manifest.version,
            fingerprint.schema.tables.len()
        );

        // Generate a seed for the synthesis
        let seed: u64 = rand::random();

        // Use ConfigSynthesizer with scale option to convert fingerprint to GeneratorConfig
        let options = SynthesisOptions {
            scale,
            seed: Some(seed),
            preserve_correlations: true,
            inject_anomalies: true,
        };
        let synthesizer = ConfigSynthesizer::with_options(options);

        // Synthesize full result including copula generators
        let synthesis_result = synthesizer
            .synthesize_full(&fingerprint, seed)
            .map_err(|e| {
                SynthError::config(format!(
                    "Failed to synthesize config from fingerprint: {}",
                    e
                ))
            })?;

        // Start with a base config from the fingerprint's industry if available
        let mut config = if let Some(ref industry) = fingerprint.manifest.source.industry {
            Self::base_config_for_industry(industry)
        } else {
            Self::base_config_for_industry("manufacturing")
        };

        // Apply the synthesized patches
        config = Self::apply_config_patch(config, &synthesis_result.config_patch);

        // Log synthesis results
        info!(
            "Config synthesized: {} tables, scale={:.2}, copula generators: {}",
            fingerprint.schema.tables.len(),
            scale,
            synthesis_result.copula_generators.len()
        );

        if !synthesis_result.copula_generators.is_empty() {
            for spec in &synthesis_result.copula_generators {
                info!(
                    "  Copula '{}' for table '{}': {} columns",
                    spec.name,
                    spec.table,
                    spec.columns.len()
                );
            }
        }

        // Create the orchestrator with the synthesized config
        let mut orchestrator = Self::new(config, phase_config)?;

        // Store copula generators for use during generation
        orchestrator.copula_generators = synthesis_result.copula_generators;

        Ok(orchestrator)
    }

    /// Create a base config for a given industry.
    fn base_config_for_industry(industry: &str) -> GeneratorConfig {
        use datasynth_config::presets::create_preset;
        use datasynth_config::TransactionVolume;
        use datasynth_core::models::{CoAComplexity, IndustrySector};

        let sector = match industry.to_lowercase().as_str() {
            "manufacturing" => IndustrySector::Manufacturing,
            "retail" => IndustrySector::Retail,
            "financial" | "financial_services" => IndustrySector::FinancialServices,
            "healthcare" => IndustrySector::Healthcare,
            "technology" | "tech" => IndustrySector::Technology,
            _ => IndustrySector::Manufacturing,
        };

        // Create a preset with reasonable defaults
        create_preset(
            sector,
            1,  // company count
            12, // period months
            CoAComplexity::Medium,
            TransactionVolume::TenK,
        )
    }

    /// Apply a config patch to a GeneratorConfig.
    fn apply_config_patch(
        mut config: GeneratorConfig,
        patch: &datasynth_fingerprint::synthesis::ConfigPatch,
    ) -> GeneratorConfig {
        use datasynth_fingerprint::synthesis::ConfigValue;

        for (key, value) in patch.values() {
            match (key.as_str(), value) {
                // Transaction count is handled via TransactionVolume enum on companies
                // Log it but cannot directly set it (would need to modify company volumes)
                ("transactions.count", ConfigValue::Integer(n)) => {
                    info!(
                        "Fingerprint suggests {} transactions (apply via company volumes)",
                        n
                    );
                }
                ("global.period_months", ConfigValue::Integer(n)) => {
                    config.global.period_months = *n as u32;
                }
                ("global.start_date", ConfigValue::String(s)) => {
                    config.global.start_date = s.clone();
                }
                ("global.seed", ConfigValue::Integer(n)) => {
                    config.global.seed = Some(*n as u64);
                }
                ("fraud.enabled", ConfigValue::Bool(b)) => {
                    config.fraud.enabled = *b;
                }
                ("fraud.fraud_rate", ConfigValue::Float(f)) => {
                    config.fraud.fraud_rate = *f;
                }
                ("data_quality.enabled", ConfigValue::Bool(b)) => {
                    config.data_quality.enabled = *b;
                }
                // Handle anomaly injection paths (mapped to fraud config)
                ("anomaly_injection.enabled", ConfigValue::Bool(b)) => {
                    config.fraud.enabled = *b;
                }
                ("anomaly_injection.overall_rate", ConfigValue::Float(f)) => {
                    config.fraud.fraud_rate = *f;
                }
                _ => {
                    debug!("Ignoring unknown config patch key: {}", key);
                }
            }
        }

        config
    }

    /// Build a resource guard from the configuration.
    fn build_resource_guard(
        config: &GeneratorConfig,
        output_path: Option<PathBuf>,
    ) -> ResourceGuard {
        let mut builder = ResourceGuardBuilder::new();

        // Configure memory limit if set
        if config.global.memory_limit_mb > 0 {
            builder = builder.memory_limit(config.global.memory_limit_mb);
        }

        // Configure disk monitoring for output path
        if let Some(path) = output_path {
            builder = builder.output_path(path).min_free_disk(100); // Require at least 100 MB free
        }

        // Use conservative degradation settings for production safety
        builder = builder.conservative();

        builder.build()
    }

    /// Check resources (memory, disk, CPU) and return degradation level.
    ///
    /// Returns an error if hard limits are exceeded.
    /// Returns Ok(DegradationLevel) indicating current resource state.
    fn check_resources(&self) -> SynthResult<DegradationLevel> {
        self.resource_guard.check()
    }

    /// Check resources with logging.
    fn check_resources_with_log(&self, phase: &str) -> SynthResult<DegradationLevel> {
        let level = self.resource_guard.check()?;

        if level != DegradationLevel::Normal {
            warn!(
                "Resource degradation at {}: level={}, memory={}MB, disk={}MB",
                phase,
                level,
                self.resource_guard.current_memory_mb(),
                self.resource_guard.available_disk_mb()
            );
        }

        Ok(level)
    }

    /// Get current degradation actions based on resource state.
    fn get_degradation_actions(&self) -> DegradationActions {
        self.resource_guard.get_actions()
    }

    /// Legacy method for backwards compatibility - now uses ResourceGuard.
    fn check_memory_limit(&self) -> SynthResult<()> {
        self.check_resources()?;
        Ok(())
    }

    /// Run the complete generation workflow.
    #[allow(clippy::field_reassign_with_default)]
    pub fn generate(&mut self) -> SynthResult<EnhancedGenerationResult> {
        info!("Starting enhanced generation workflow");
        info!(
            "Config: industry={:?}, period_months={}, companies={}",
            self.config.global.industry,
            self.config.global.period_months,
            self.config.companies.len()
        );

        // Initial resource check before starting
        let initial_level = self.check_resources_with_log("initial")?;
        if initial_level == DegradationLevel::Emergency {
            return Err(SynthError::resource(
                "Insufficient resources to start generation",
            ));
        }

        let mut stats = EnhancedGenerationStatistics::default();
        stats.companies_count = self.config.companies.len();
        stats.period_months = self.config.global.period_months;

        // Phase 1: Generate Chart of Accounts
        info!("Phase 1: Generating Chart of Accounts");
        let coa = self.generate_coa()?;
        stats.accounts_count = coa.account_count();
        info!(
            "Chart of Accounts generated: {} accounts",
            stats.accounts_count
        );

        // Check resources after CoA generation
        self.check_resources_with_log("post-coa")?;

        // Phase 2: Generate Master Data
        if self.phase_config.generate_master_data {
            info!("Phase 2: Generating Master Data");
            self.generate_master_data()?;
            stats.vendor_count = self.master_data.vendors.len();
            stats.customer_count = self.master_data.customers.len();
            stats.material_count = self.master_data.materials.len();
            stats.asset_count = self.master_data.assets.len();
            stats.employee_count = self.master_data.employees.len();
            info!(
                "Master data generated: {} vendors, {} customers, {} materials, {} assets, {} employees",
                stats.vendor_count, stats.customer_count, stats.material_count,
                stats.asset_count, stats.employee_count
            );

            // Check resources after master data generation
            self.check_resources_with_log("post-master-data")?;
        } else {
            debug!("Phase 2: Skipped (master data generation disabled)");
        }

        // Phase 3: Generate Document Flows
        let mut document_flows = DocumentFlowSnapshot::default();
        let mut subledger = SubledgerSnapshot::default();
        if self.phase_config.generate_document_flows && !self.master_data.vendors.is_empty() {
            info!("Phase 3: Generating Document Flows");
            self.generate_document_flows(&mut document_flows)?;
            stats.p2p_chain_count = document_flows.p2p_chains.len();
            stats.o2c_chain_count = document_flows.o2c_chains.len();
            info!(
                "Document flows generated: {} P2P chains, {} O2C chains",
                stats.p2p_chain_count, stats.o2c_chain_count
            );

            // Phase 3b: Link document flows to subledgers (for data coherence)
            debug!("Phase 3b: Linking document flows to subledgers");
            subledger = self.link_document_flows_to_subledgers(&document_flows)?;
            stats.ap_invoice_count = subledger.ap_invoices.len();
            stats.ar_invoice_count = subledger.ar_invoices.len();
            debug!(
                "Subledgers linked: {} AP invoices, {} AR invoices",
                stats.ap_invoice_count, stats.ar_invoice_count
            );

            // Check resources after document flow generation
            self.check_resources_with_log("post-document-flows")?;
        } else {
            debug!("Phase 3: Skipped (document flow generation disabled or no master data)");
        }

        // Phase 3c: Generate OCPM events from document flows
        let mut ocpm_snapshot = OcpmSnapshot::default();
        if self.phase_config.generate_ocpm_events && !document_flows.p2p_chains.is_empty() {
            info!("Phase 3c: Generating OCPM Events");
            ocpm_snapshot = self.generate_ocpm_events(&document_flows)?;
            stats.ocpm_event_count = ocpm_snapshot.event_count;
            stats.ocpm_object_count = ocpm_snapshot.object_count;
            stats.ocpm_case_count = ocpm_snapshot.case_count;
            info!(
                "OCPM events generated: {} events, {} objects, {} cases",
                stats.ocpm_event_count, stats.ocpm_object_count, stats.ocpm_case_count
            );

            // Check resources after OCPM generation
            self.check_resources_with_log("post-ocpm")?;
        } else {
            debug!("Phase 3c: Skipped (OCPM generation disabled or no document flows)");
        }

        // Phase 4: Generate Journal Entries
        let mut entries = Vec::new();

        // Phase 4a: Generate JEs from document flows (for data coherence)
        if self.phase_config.generate_document_flows && !document_flows.p2p_chains.is_empty() {
            debug!("Phase 4a: Generating JEs from document flows");
            let flow_entries = self.generate_jes_from_document_flows(&document_flows)?;
            debug!("Generated {} JEs from document flows", flow_entries.len());
            entries.extend(flow_entries);
        }

        // Phase 4b: Generate standalone journal entries
        if self.phase_config.generate_journal_entries {
            info!("Phase 4: Generating Journal Entries");
            let je_entries = self.generate_journal_entries(&coa)?;
            info!("Generated {} standalone journal entries", je_entries.len());
            entries.extend(je_entries);
        } else {
            debug!("Phase 4: Skipped (journal entry generation disabled)");
        }

        if !entries.is_empty() {
            stats.total_entries = entries.len() as u64;
            stats.total_line_items = entries.iter().map(|e| e.line_count() as u64).sum();
            info!(
                "Total entries: {}, total line items: {}",
                stats.total_entries, stats.total_line_items
            );

            // Check resources after JE generation (high-volume phase)
            self.check_resources_with_log("post-journal-entries")?;
        }

        // Get current degradation actions for optional phases
        let actions = self.get_degradation_actions();

        // Phase 5: Inject Anomalies (skip if degradation dictates)
        let mut anomaly_labels = AnomalyLabels::default();
        if self.phase_config.inject_anomalies
            && !entries.is_empty()
            && !actions.skip_anomaly_injection
        {
            info!("Phase 5: Injecting Anomalies");
            let result = self.inject_anomalies(&mut entries)?;
            stats.anomalies_injected = result.labels.len();
            anomaly_labels = result;
            info!("Injected {} anomalies", stats.anomalies_injected);

            // Check resources after anomaly injection
            self.check_resources_with_log("post-anomaly-injection")?;
        } else if actions.skip_anomaly_injection {
            warn!("Phase 5: Skipped due to resource degradation");
        } else {
            debug!("Phase 5: Skipped (anomaly injection disabled or no entries)");
        }

        // Phase 6: Validate Balances
        let mut balance_validation = BalanceValidationResult::default();
        if self.phase_config.validate_balances && !entries.is_empty() {
            debug!("Phase 6: Validating Balances");
            balance_validation = self.validate_journal_entries(&entries)?;
            if balance_validation.is_balanced {
                debug!("Balance validation passed");
            } else {
                warn!(
                    "Balance validation found {} errors",
                    balance_validation.validation_errors.len()
                );
            }
        }

        // Phase 7: Inject Data Quality Variations (skip if degradation dictates)
        let mut data_quality_stats = DataQualityStats::default();
        if self.phase_config.inject_data_quality
            && !entries.is_empty()
            && !actions.skip_data_quality
        {
            info!("Phase 7: Injecting Data Quality Variations");
            data_quality_stats = self.inject_data_quality(&mut entries)?;
            stats.data_quality_issues = data_quality_stats.records_with_issues;
            info!("Injected {} data quality issues", stats.data_quality_issues);

            // Check resources after data quality injection
            self.check_resources_with_log("post-data-quality")?;
        } else if actions.skip_data_quality {
            warn!("Phase 7: Skipped due to resource degradation");
        } else {
            debug!("Phase 7: Skipped (data quality injection disabled or no entries)");
        }

        // Phase 8: Generate Audit Data
        let mut audit_snapshot = AuditSnapshot::default();
        if self.phase_config.generate_audit {
            info!("Phase 8: Generating Audit Data");
            audit_snapshot = self.generate_audit_data(&entries)?;
            stats.audit_engagement_count = audit_snapshot.engagements.len();
            stats.audit_workpaper_count = audit_snapshot.workpapers.len();
            stats.audit_evidence_count = audit_snapshot.evidence.len();
            stats.audit_risk_count = audit_snapshot.risk_assessments.len();
            stats.audit_finding_count = audit_snapshot.findings.len();
            stats.audit_judgment_count = audit_snapshot.judgments.len();
            info!(
                "Audit data generated: {} engagements, {} workpapers, {} evidence, {} risks, {} findings, {} judgments",
                stats.audit_engagement_count, stats.audit_workpaper_count,
                stats.audit_evidence_count, stats.audit_risk_count,
                stats.audit_finding_count, stats.audit_judgment_count
            );

            // Check resources after audit generation
            self.check_resources_with_log("post-audit")?;
        } else {
            debug!("Phase 8: Skipped (audit generation disabled)");
        }

        // Phase 9: Generate Banking KYC/AML Data
        let mut banking_snapshot = BankingSnapshot::default();
        if self.phase_config.generate_banking && self.config.banking.enabled {
            info!("Phase 9: Generating Banking KYC/AML Data");
            banking_snapshot = self.generate_banking_data()?;
            stats.banking_customer_count = banking_snapshot.customers.len();
            stats.banking_account_count = banking_snapshot.accounts.len();
            stats.banking_transaction_count = banking_snapshot.transactions.len();
            stats.banking_suspicious_count = banking_snapshot.suspicious_count;
            info!(
                "Banking data generated: {} customers, {} accounts, {} transactions ({} suspicious)",
                stats.banking_customer_count, stats.banking_account_count,
                stats.banking_transaction_count, stats.banking_suspicious_count
            );

            // Check resources after banking generation
            self.check_resources_with_log("post-banking")?;
        } else {
            debug!("Phase 9: Skipped (banking generation disabled)");
        }

        // Phase 10: Export Graphs
        let graph_export_snapshot = if (self.phase_config.generate_graph_export
            || self.config.graph_export.enabled)
            && !entries.is_empty()
        {
            info!("Phase 10: Exporting Accounting Network Graphs");
            match self.export_graphs(&entries, &coa, &mut stats) {
                Ok(snapshot) => {
                    info!(
                        "Graph export complete: {} graphs ({} nodes, {} edges)",
                        snapshot.graph_count, stats.graph_node_count, stats.graph_edge_count
                    );
                    snapshot
                }
                Err(e) => {
                    warn!("Phase 10: Graph export failed: {}", e);
                    GraphExportSnapshot::default()
                }
            }
        } else {
            debug!("Phase 10: Skipped (graph export disabled or no entries)");
            GraphExportSnapshot::default()
        };

        // Log final resource statistics
        let resource_stats = self.resource_guard.stats();
        info!(
            "Generation workflow complete. Resource stats: memory_peak={}MB, disk_written={}bytes, degradation_level={}",
            resource_stats.memory.peak_resident_bytes / (1024 * 1024),
            resource_stats.disk.estimated_bytes_written,
            resource_stats.degradation_level
        );

        Ok(EnhancedGenerationResult {
            chart_of_accounts: (*coa).clone(),
            master_data: self.master_data.clone(),
            document_flows,
            subledger,
            ocpm: ocpm_snapshot,
            audit: audit_snapshot,
            banking: banking_snapshot,
            graph_export: graph_export_snapshot,
            journal_entries: entries,
            anomaly_labels,
            balance_validation,
            data_quality_stats,
            statistics: stats,
        })
    }

    /// Generate the chart of accounts.
    fn generate_coa(&mut self) -> SynthResult<Arc<ChartOfAccounts>> {
        let pb = self.create_progress_bar(1, "Generating Chart of Accounts");

        let mut gen = ChartOfAccountsGenerator::new(
            self.config.chart_of_accounts.complexity,
            self.config.global.industry,
            self.seed,
        );

        let coa = Arc::new(gen.generate());
        self.coa = Some(Arc::clone(&coa));

        if let Some(pb) = pb {
            pb.finish_with_message("Chart of Accounts complete");
        }

        Ok(coa)
    }

    /// Generate master data entities.
    fn generate_master_data(&mut self) -> SynthResult<()> {
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let total = self.config.companies.len() as u64 * 5; // 5 entity types
        let pb = self.create_progress_bar(total, "Generating Master Data");

        for (i, company) in self.config.companies.iter().enumerate() {
            let company_seed = self.seed.wrapping_add(i as u64 * 1000);

            // Generate vendors
            let mut vendor_gen = VendorGenerator::new(company_seed);
            let vendor_pool = vendor_gen.generate_vendor_pool(
                self.phase_config.vendors_per_company,
                &company.code,
                start_date,
            );
            self.master_data.vendors.extend(vendor_pool.vendors);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate customers
            let mut customer_gen = CustomerGenerator::new(company_seed + 100);
            let customer_pool = customer_gen.generate_customer_pool(
                self.phase_config.customers_per_company,
                &company.code,
                start_date,
            );
            self.master_data.customers.extend(customer_pool.customers);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate materials
            let mut material_gen = MaterialGenerator::new(company_seed + 200);
            let material_pool = material_gen.generate_material_pool(
                self.phase_config.materials_per_company,
                &company.code,
                start_date,
            );
            self.master_data.materials.extend(material_pool.materials);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate fixed assets
            let mut asset_gen = AssetGenerator::new(company_seed + 300);
            let asset_pool = asset_gen.generate_asset_pool(
                self.phase_config.assets_per_company,
                &company.code,
                (start_date, end_date),
            );
            self.master_data.assets.extend(asset_pool.assets);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate employees
            let mut employee_gen = EmployeeGenerator::new(company_seed + 400);
            let employee_pool =
                employee_gen.generate_company_pool(&company.code, (start_date, end_date));
            self.master_data.employees.extend(employee_pool.employees);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Master data generation complete");
        }

        Ok(())
    }

    /// Generate document flows (P2P and O2C).
    fn generate_document_flows(&mut self, flows: &mut DocumentFlowSnapshot) -> SynthResult<()> {
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        // Generate P2P chains
        let p2p_count = self
            .phase_config
            .p2p_chains
            .min(self.master_data.vendors.len() * 2);
        let pb = self.create_progress_bar(p2p_count as u64, "Generating P2P Document Flows");

        // Convert P2P config from schema to generator config
        let p2p_config = convert_p2p_config(&self.config.document_flows.p2p);
        let mut p2p_gen = P2PGenerator::with_config(self.seed + 1000, p2p_config);

        for i in 0..p2p_count {
            let vendor = &self.master_data.vendors[i % self.master_data.vendors.len()];
            let materials: Vec<&Material> = self
                .master_data
                .materials
                .iter()
                .skip(i % self.master_data.materials.len().max(1))
                .take(2.min(self.master_data.materials.len()))
                .collect();

            if materials.is_empty() {
                continue;
            }

            let company = &self.config.companies[i % self.config.companies.len()];
            let po_date = start_date + chrono::Duration::days((i * 3) as i64 % 365);
            let fiscal_period = po_date.month() as u8;
            let created_by = self
                .master_data
                .employees
                .first()
                .map(|e| e.user_id.as_str())
                .unwrap_or("SYSTEM");

            let chain = p2p_gen.generate_chain(
                &company.code,
                vendor,
                &materials,
                po_date,
                start_date.year() as u16,
                fiscal_period,
                created_by,
            );

            // Flatten documents
            flows.purchase_orders.push(chain.purchase_order.clone());
            flows.goods_receipts.extend(chain.goods_receipts.clone());
            if let Some(vi) = &chain.vendor_invoice {
                flows.vendor_invoices.push(vi.clone());
            }
            if let Some(payment) = &chain.payment {
                flows.payments.push(payment.clone());
            }
            flows.p2p_chains.push(chain);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("P2P document flows complete");
        }

        // Generate O2C chains
        let o2c_count = self
            .phase_config
            .o2c_chains
            .min(self.master_data.customers.len() * 2);
        let pb = self.create_progress_bar(o2c_count as u64, "Generating O2C Document Flows");

        // Convert O2C config from schema to generator config
        let o2c_config = convert_o2c_config(&self.config.document_flows.o2c);
        let mut o2c_gen = O2CGenerator::with_config(self.seed + 2000, o2c_config);

        for i in 0..o2c_count {
            let customer = &self.master_data.customers[i % self.master_data.customers.len()];
            let materials: Vec<&Material> = self
                .master_data
                .materials
                .iter()
                .skip(i % self.master_data.materials.len().max(1))
                .take(2.min(self.master_data.materials.len()))
                .collect();

            if materials.is_empty() {
                continue;
            }

            let company = &self.config.companies[i % self.config.companies.len()];
            let so_date = start_date + chrono::Duration::days((i * 2) as i64 % 365);
            let fiscal_period = so_date.month() as u8;
            let created_by = self
                .master_data
                .employees
                .first()
                .map(|e| e.user_id.as_str())
                .unwrap_or("SYSTEM");

            let chain = o2c_gen.generate_chain(
                &company.code,
                customer,
                &materials,
                so_date,
                start_date.year() as u16,
                fiscal_period,
                created_by,
            );

            // Flatten documents
            flows.sales_orders.push(chain.sales_order.clone());
            flows.deliveries.extend(chain.deliveries.clone());
            if let Some(ci) = &chain.customer_invoice {
                flows.customer_invoices.push(ci.clone());
            }
            if let Some(receipt) = &chain.customer_receipt {
                flows.payments.push(receipt.clone());
            }
            flows.o2c_chains.push(chain);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("O2C document flows complete");
        }

        Ok(())
    }

    /// Generate journal entries.
    fn generate_journal_entries(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
    ) -> SynthResult<Vec<JournalEntry>> {
        let total = self.calculate_total_transactions();
        let pb = self.create_progress_bar(total, "Generating Journal Entries");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        let generator = JournalEntryGenerator::new_with_params(
            self.config.transactions.clone(),
            Arc::clone(coa),
            company_codes,
            start_date,
            end_date,
            self.seed,
        );

        // Connect generated master data to ensure JEs reference real entities
        // Enable persona-based error injection for realistic human behavior
        // Pass fraud configuration for fraud injection
        let mut generator = generator
            .with_master_data(
                &self.master_data.vendors,
                &self.master_data.customers,
                &self.master_data.materials,
            )
            .with_persona_errors(true)
            .with_fraud_config(self.config.fraud.clone());

        // Apply temporal drift if configured
        if self.config.temporal.enabled {
            let drift_config = self.config.temporal.to_core_config();
            generator = generator.with_drift_config(drift_config, self.seed + 100);
        }

        let mut entries = Vec::with_capacity(total as usize);

        // Check memory limit at start
        self.check_memory_limit()?;

        // Check every 1000 entries to avoid overhead
        const MEMORY_CHECK_INTERVAL: u64 = 1000;

        for i in 0..total {
            let entry = generator.generate();
            entries.push(entry);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Periodic memory limit check
            if (i + 1) % MEMORY_CHECK_INTERVAL == 0 {
                self.check_memory_limit()?;
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Journal entries complete");
        }

        Ok(entries)
    }

    /// Generate journal entries from document flows.
    ///
    /// This creates proper GL entries for each document in the P2P and O2C flows,
    /// ensuring that document activity is reflected in the general ledger.
    fn generate_jes_from_document_flows(
        &mut self,
        flows: &DocumentFlowSnapshot,
    ) -> SynthResult<Vec<JournalEntry>> {
        let total_chains = flows.p2p_chains.len() + flows.o2c_chains.len();
        let pb = self.create_progress_bar(total_chains as u64, "Generating Document Flow JEs");

        let mut generator = DocumentFlowJeGenerator::with_config_and_seed(
            DocumentFlowJeConfig::default(),
            self.seed,
        );
        let mut entries = Vec::new();

        // Generate JEs from P2P chains
        for chain in &flows.p2p_chains {
            let chain_entries = generator.generate_from_p2p_chain(chain);
            entries.extend(chain_entries);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate JEs from O2C chains
        for chain in &flows.o2c_chains {
            let chain_entries = generator.generate_from_o2c_chain(chain);
            entries.extend(chain_entries);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Generated {} JEs from document flows",
                entries.len()
            ));
        }

        Ok(entries)
    }

    /// Link document flows to subledger records.
    ///
    /// Creates AP invoices from vendor invoices and AR invoices from customer invoices,
    /// ensuring subledger data is coherent with document flow data.
    fn link_document_flows_to_subledgers(
        &mut self,
        flows: &DocumentFlowSnapshot,
    ) -> SynthResult<SubledgerSnapshot> {
        let total = flows.vendor_invoices.len() + flows.customer_invoices.len();
        let pb = self.create_progress_bar(total as u64, "Linking Subledgers");

        let mut linker = DocumentFlowLinker::new();

        // Convert vendor invoices to AP invoices
        let ap_invoices = linker.batch_create_ap_invoices(&flows.vendor_invoices);
        if let Some(pb) = &pb {
            pb.inc(flows.vendor_invoices.len() as u64);
        }

        // Convert customer invoices to AR invoices
        let ar_invoices = linker.batch_create_ar_invoices(&flows.customer_invoices);
        if let Some(pb) = &pb {
            pb.inc(flows.customer_invoices.len() as u64);
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Linked {} AP and {} AR invoices",
                ap_invoices.len(),
                ar_invoices.len()
            ));
        }

        Ok(SubledgerSnapshot {
            ap_invoices,
            ar_invoices,
        })
    }

    /// Generate OCPM events from document flows.
    ///
    /// Creates OCEL 2.0 compliant event logs from P2P and O2C document flows,
    /// capturing the object-centric process perspective.
    fn generate_ocpm_events(&mut self, flows: &DocumentFlowSnapshot) -> SynthResult<OcpmSnapshot> {
        let total_chains = flows.p2p_chains.len() + flows.o2c_chains.len();
        let pb = self.create_progress_bar(total_chains as u64, "Generating OCPM Events");

        // Create OCPM event log with standard types
        let metadata = EventLogMetadata::new("SyntheticData OCPM Log");
        let mut event_log = OcpmEventLog::with_metadata(metadata).with_standard_types();

        // Configure the OCPM generator
        let ocpm_config = OcpmGeneratorConfig {
            generate_p2p: true,
            generate_o2c: true,
            happy_path_rate: 0.75,
            exception_path_rate: 0.20,
            error_path_rate: 0.05,
            add_duration_variability: true,
            duration_std_dev_factor: 0.3,
        };
        let mut ocpm_gen = OcpmEventGenerator::with_config(self.seed + 3000, ocpm_config);

        // Get available users for resource assignment
        let available_users: Vec<String> = self
            .master_data
            .employees
            .iter()
            .take(20)
            .map(|e| e.user_id.clone())
            .collect();

        // Generate events from P2P chains
        for chain in &flows.p2p_chains {
            let po = &chain.purchase_order;
            let documents = P2pDocuments::new(
                &po.header.document_id,
                &po.vendor_id,
                &po.header.company_code,
                po.total_net_amount,
                &po.header.currency,
            )
            .with_goods_receipt(
                chain
                    .goods_receipts
                    .first()
                    .map(|gr| gr.header.document_id.as_str())
                    .unwrap_or(""),
            )
            .with_invoice(
                chain
                    .vendor_invoice
                    .as_ref()
                    .map(|vi| vi.header.document_id.as_str())
                    .unwrap_or(""),
            )
            .with_payment(
                chain
                    .payment
                    .as_ref()
                    .map(|p| p.header.document_id.as_str())
                    .unwrap_or(""),
            );

            let start_time =
                chrono::DateTime::from_naive_utc_and_offset(po.header.entry_timestamp, chrono::Utc);
            let result = ocpm_gen.generate_p2p_case(&documents, start_time, &available_users);

            // Add events and objects to the event log
            for event in result.events {
                event_log.add_event(event);
            }
            for object in result.objects {
                event_log.add_object(object);
            }
            for relationship in result.relationships {
                event_log.add_relationship(relationship);
            }
            event_log.add_case(result.case_trace);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from O2C chains
        for chain in &flows.o2c_chains {
            let so = &chain.sales_order;
            let documents = O2cDocuments::new(
                &so.header.document_id,
                &so.customer_id,
                &so.header.company_code,
                so.total_net_amount,
                &so.header.currency,
            )
            .with_delivery(
                chain
                    .deliveries
                    .first()
                    .map(|d| d.header.document_id.as_str())
                    .unwrap_or(""),
            )
            .with_invoice(
                chain
                    .customer_invoice
                    .as_ref()
                    .map(|ci| ci.header.document_id.as_str())
                    .unwrap_or(""),
            )
            .with_receipt(
                chain
                    .customer_receipt
                    .as_ref()
                    .map(|r| r.header.document_id.as_str())
                    .unwrap_or(""),
            );

            let start_time =
                chrono::DateTime::from_naive_utc_and_offset(so.header.entry_timestamp, chrono::Utc);
            let result = ocpm_gen.generate_o2c_case(&documents, start_time, &available_users);

            // Add events and objects to the event log
            for event in result.events {
                event_log.add_event(event);
            }
            for object in result.objects {
                event_log.add_object(object);
            }
            for relationship in result.relationships {
                event_log.add_relationship(relationship);
            }
            event_log.add_case(result.case_trace);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Compute process variants
        event_log.compute_variants();

        let summary = event_log.summary();

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Generated {} OCPM events, {} objects",
                summary.event_count, summary.object_count
            ));
        }

        Ok(OcpmSnapshot {
            event_count: summary.event_count,
            object_count: summary.object_count,
            case_count: summary.case_count,
            event_log: Some(event_log),
        })
    }

    /// Inject anomalies into journal entries.
    fn inject_anomalies(&mut self, entries: &mut [JournalEntry]) -> SynthResult<AnomalyLabels> {
        let pb = self.create_progress_bar(entries.len() as u64, "Injecting Anomalies");

        let anomaly_config = AnomalyInjectorConfig {
            rates: AnomalyRateConfig {
                total_rate: 0.02,
                ..Default::default()
            },
            seed: self.seed + 5000,
            ..Default::default()
        };

        let mut injector = AnomalyInjector::new(anomaly_config);
        let result = injector.process_entries(entries);

        if let Some(pb) = &pb {
            pb.inc(entries.len() as u64);
            pb.finish_with_message("Anomaly injection complete");
        }

        let mut by_type = HashMap::new();
        for label in &result.labels {
            *by_type
                .entry(format!("{:?}", label.anomaly_type))
                .or_insert(0) += 1;
        }

        Ok(AnomalyLabels {
            labels: result.labels,
            summary: Some(result.summary),
            by_type,
        })
    }

    /// Validate journal entries using running balance tracker.
    ///
    /// Applies all entries to the balance tracker and validates:
    /// - Each entry is internally balanced (debits = credits)
    /// - Balance sheet equation holds (Assets = Liabilities + Equity + Net Income)
    ///
    /// Note: Entries with human errors (marked with [HUMAN_ERROR:*] tags) are
    /// excluded from balance validation as they may be intentionally unbalanced.
    fn validate_journal_entries(
        &mut self,
        entries: &[JournalEntry],
    ) -> SynthResult<BalanceValidationResult> {
        // Filter out entries with human errors as they may be intentionally unbalanced
        let clean_entries: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| {
                e.header
                    .header_text
                    .as_ref()
                    .map(|t| !t.contains("[HUMAN_ERROR:"))
                    .unwrap_or(true)
            })
            .collect();

        let pb = self.create_progress_bar(clean_entries.len() as u64, "Validating Balances");

        // Configure tracker to not fail on errors (collect them instead)
        let config = BalanceTrackerConfig {
            validate_on_each_entry: false,   // We'll validate at the end
            track_history: false,            // Skip history for performance
            fail_on_validation_error: false, // Collect errors, don't fail
            ..Default::default()
        };

        let mut tracker = RunningBalanceTracker::new(config);

        // Apply clean entries (without human errors)
        let clean_refs: Vec<JournalEntry> = clean_entries.into_iter().cloned().collect();
        let errors = tracker.apply_entries(&clean_refs);

        if let Some(pb) = &pb {
            pb.inc(entries.len() as u64);
        }

        // Check if any entries were unbalanced
        // Note: When fail_on_validation_error is false, errors are stored in tracker
        let has_unbalanced = tracker
            .get_validation_errors()
            .iter()
            .any(|e| e.error_type == datasynth_generators::ValidationErrorType::UnbalancedEntry);

        // Validate balance sheet for each company
        // Include both returned errors and collected validation errors
        let mut all_errors = errors;
        all_errors.extend(tracker.get_validation_errors().iter().cloned());
        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        let end_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map(|d| d + chrono::Months::new(self.config.global.period_months))
            .unwrap_or_else(|_| chrono::Local::now().date_naive());

        for company_code in &company_codes {
            if let Err(e) = tracker.validate_balance_sheet(company_code, end_date, None) {
                all_errors.push(e);
            }
        }

        // Get statistics after all mutable operations are done
        let stats = tracker.get_statistics();

        // Determine if balanced overall
        let is_balanced = all_errors.is_empty();

        if let Some(pb) = pb {
            let msg = if is_balanced {
                "Balance validation passed"
            } else {
                "Balance validation completed with errors"
            };
            pb.finish_with_message(msg);
        }

        Ok(BalanceValidationResult {
            validated: true,
            is_balanced,
            entries_processed: stats.entries_processed,
            total_debits: stats.total_debits,
            total_credits: stats.total_credits,
            accounts_tracked: stats.accounts_tracked,
            companies_tracked: stats.companies_tracked,
            validation_errors: all_errors,
            has_unbalanced_entries: has_unbalanced,
        })
    }

    /// Inject data quality variations into journal entries.
    ///
    /// Applies typos, missing values, and format variations to make
    /// the synthetic data more realistic for testing data cleaning pipelines.
    fn inject_data_quality(
        &mut self,
        entries: &mut [JournalEntry],
    ) -> SynthResult<DataQualityStats> {
        let pb = self.create_progress_bar(entries.len() as u64, "Injecting Data Quality Issues");

        // Use minimal configuration by default for realistic but not overwhelming issues
        let config = DataQualityConfig::minimal();
        let mut injector = DataQualityInjector::new(config);

        // Build context for missing value decisions
        let context = HashMap::new();

        for entry in entries.iter_mut() {
            // Process header_text field (common target for typos)
            if let Some(text) = &entry.header.header_text {
                let processed = injector.process_text_field(
                    "header_text",
                    text,
                    &entry.header.document_id.to_string(),
                    &context,
                );
                match processed {
                    Some(new_text) if new_text != *text => {
                        entry.header.header_text = Some(new_text);
                    }
                    None => {
                        entry.header.header_text = None; // Missing value
                    }
                    _ => {}
                }
            }

            // Process reference field
            if let Some(ref_text) = &entry.header.reference {
                let processed = injector.process_text_field(
                    "reference",
                    ref_text,
                    &entry.header.document_id.to_string(),
                    &context,
                );
                match processed {
                    Some(new_text) if new_text != *ref_text => {
                        entry.header.reference = Some(new_text);
                    }
                    None => {
                        entry.header.reference = None;
                    }
                    _ => {}
                }
            }

            // Process user_persona field (potential for typos in user IDs)
            let user_persona = entry.header.user_persona.clone();
            if let Some(processed) = injector.process_text_field(
                "user_persona",
                &user_persona,
                &entry.header.document_id.to_string(),
                &context,
            ) {
                if processed != user_persona {
                    entry.header.user_persona = processed;
                }
            }

            // Process line items
            for line in &mut entry.lines {
                // Process line description if present
                if let Some(ref text) = line.line_text {
                    let processed = injector.process_text_field(
                        "line_text",
                        text,
                        &entry.header.document_id.to_string(),
                        &context,
                    );
                    match processed {
                        Some(new_text) if new_text != *text => {
                            line.line_text = Some(new_text);
                        }
                        None => {
                            line.line_text = None;
                        }
                        _ => {}
                    }
                }

                // Process cost_center if present
                if let Some(cc) = &line.cost_center {
                    let processed = injector.process_text_field(
                        "cost_center",
                        cc,
                        &entry.header.document_id.to_string(),
                        &context,
                    );
                    match processed {
                        Some(new_cc) if new_cc != *cc => {
                            line.cost_center = Some(new_cc);
                        }
                        None => {
                            line.cost_center = None;
                        }
                        _ => {}
                    }
                }
            }

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Data quality injection complete");
        }

        Ok(injector.stats().clone())
    }

    /// Generate audit data (engagements, workpapers, evidence, risks, findings, judgments).
    ///
    /// Creates complete audit documentation for each company in the configuration,
    /// following ISA standards:
    /// - ISA 210/220: Engagement acceptance and terms
    /// - ISA 230: Audit documentation (workpapers)
    /// - ISA 265: Control deficiencies (findings)
    /// - ISA 315/330: Risk assessment and response
    /// - ISA 500: Audit evidence
    /// - ISA 200: Professional judgment
    fn generate_audit_data(&mut self, entries: &[JournalEntry]) -> SynthResult<AuditSnapshot> {
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let fiscal_year = start_date.year() as u16;
        let period_end = start_date + chrono::Months::new(self.config.global.period_months);

        // Calculate rough total revenue from entries for materiality
        let total_revenue: rust_decimal::Decimal = entries
            .iter()
            .flat_map(|e| e.lines.iter())
            .filter(|l| l.credit_amount > rust_decimal::Decimal::ZERO)
            .map(|l| l.credit_amount)
            .sum();

        let total_items = (self.phase_config.audit_engagements * 50) as u64; // Approximate items
        let pb = self.create_progress_bar(total_items, "Generating Audit Data");

        let mut snapshot = AuditSnapshot::default();

        // Initialize generators
        let mut engagement_gen = AuditEngagementGenerator::new(self.seed + 7000);
        let mut workpaper_gen = WorkpaperGenerator::new(self.seed + 7100);
        let mut evidence_gen = EvidenceGenerator::new(self.seed + 7200);
        let mut risk_gen = RiskAssessmentGenerator::new(self.seed + 7300);
        let mut finding_gen = FindingGenerator::new(self.seed + 7400);
        let mut judgment_gen = JudgmentGenerator::new(self.seed + 7500);

        // Get list of accounts from CoA for risk assessment
        let accounts: Vec<String> = self
            .coa
            .as_ref()
            .map(|coa| {
                coa.get_postable_accounts()
                    .iter()
                    .map(|acc| acc.account_code().to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Generate engagements for each company
        for (i, company) in self.config.companies.iter().enumerate() {
            // Calculate company-specific revenue (proportional to volume weight)
            let company_revenue = total_revenue
                * rust_decimal::Decimal::try_from(company.volume_weight).unwrap_or_default();

            // Generate engagements for this company
            let engagements_for_company =
                self.phase_config.audit_engagements / self.config.companies.len().max(1);
            let extra = if i < self.phase_config.audit_engagements % self.config.companies.len() {
                1
            } else {
                0
            };

            for _eng_idx in 0..(engagements_for_company + extra) {
                // Generate the engagement
                let engagement = engagement_gen.generate_engagement(
                    &company.code,
                    &company.name,
                    fiscal_year,
                    period_end,
                    company_revenue,
                    None, // Use default engagement type
                );

                if let Some(pb) = &pb {
                    pb.inc(1);
                }

                // Get team members from the engagement
                let team_members: Vec<String> = engagement.team_member_ids.clone();

                // Generate workpapers for the engagement
                let workpapers =
                    workpaper_gen.generate_complete_workpaper_set(&engagement, &team_members);

                for wp in &workpapers {
                    if let Some(pb) = &pb {
                        pb.inc(1);
                    }

                    // Generate evidence for each workpaper
                    let evidence = evidence_gen.generate_evidence_for_workpaper(
                        wp,
                        &team_members,
                        wp.preparer_date,
                    );

                    for _ in &evidence {
                        if let Some(pb) = &pb {
                            pb.inc(1);
                        }
                    }

                    snapshot.evidence.extend(evidence);
                }

                // Generate risk assessments for the engagement
                let risks =
                    risk_gen.generate_risks_for_engagement(&engagement, &team_members, &accounts);

                for _ in &risks {
                    if let Some(pb) = &pb {
                        pb.inc(1);
                    }
                }
                snapshot.risk_assessments.extend(risks);

                // Generate findings for the engagement
                let findings = finding_gen.generate_findings_for_engagement(
                    &engagement,
                    &workpapers,
                    &team_members,
                );

                for _ in &findings {
                    if let Some(pb) = &pb {
                        pb.inc(1);
                    }
                }
                snapshot.findings.extend(findings);

                // Generate professional judgments for the engagement
                let judgments =
                    judgment_gen.generate_judgments_for_engagement(&engagement, &team_members);

                for _ in &judgments {
                    if let Some(pb) = &pb {
                        pb.inc(1);
                    }
                }
                snapshot.judgments.extend(judgments);

                // Add workpapers after findings since findings need them
                snapshot.workpapers.extend(workpapers);
                snapshot.engagements.push(engagement);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Audit data: {} engagements, {} workpapers, {} evidence",
                snapshot.engagements.len(),
                snapshot.workpapers.len(),
                snapshot.evidence.len()
            ));
        }

        Ok(snapshot)
    }

    /// Export journal entries as graph data for ML training and network reconstruction.
    ///
    /// Builds a transaction graph where:
    /// - Nodes are GL accounts
    /// - Edges are money flows from credit to debit accounts
    /// - Edge attributes include amount, date, business process, anomaly flags
    fn export_graphs(
        &mut self,
        entries: &[JournalEntry],
        _coa: &Arc<ChartOfAccounts>,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<GraphExportSnapshot> {
        let pb = self.create_progress_bar(100, "Exporting Graphs");

        let mut snapshot = GraphExportSnapshot::default();

        // Get output directory
        let output_dir = self
            .output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.config.output.output_directory));
        let graph_dir = output_dir.join(&self.config.graph_export.output_subdirectory);

        // Process each graph type configuration
        for graph_type in &self.config.graph_export.graph_types {
            if let Some(pb) = &pb {
                pb.inc(10);
            }

            // Build transaction graph
            let graph_config = TransactionGraphConfig {
                include_vendors: false,
                include_customers: false,
                create_debit_credit_edges: true,
                include_document_nodes: graph_type.include_document_nodes,
                min_edge_weight: graph_type.min_edge_weight,
                aggregate_parallel_edges: graph_type.aggregate_edges,
            };

            let mut builder = TransactionGraphBuilder::new(graph_config);
            builder.add_journal_entries(entries);
            let graph = builder.build();

            // Update stats
            stats.graph_node_count += graph.node_count();
            stats.graph_edge_count += graph.edge_count();

            if let Some(pb) = &pb {
                pb.inc(40);
            }

            // Export to each configured format
            for format in &self.config.graph_export.formats {
                let format_dir = graph_dir.join(&graph_type.name).join(format_name(*format));

                // Create output directory
                if let Err(e) = std::fs::create_dir_all(&format_dir) {
                    warn!("Failed to create graph output directory: {}", e);
                    continue;
                }

                match format {
                    datasynth_config::schema::GraphExportFormat::PytorchGeometric => {
                        let pyg_config = PyGExportConfig {
                            export_node_features: true,
                            export_edge_features: true,
                            export_node_labels: true,
                            export_edge_labels: true,
                            one_hot_categoricals: false,
                            export_masks: true,
                            train_ratio: self.config.graph_export.train_ratio,
                            val_ratio: self.config.graph_export.validation_ratio,
                            seed: self.config.graph_export.split_seed.unwrap_or(self.seed),
                        };

                        let exporter = PyGExporter::new(pyg_config);
                        match exporter.export(&graph, &format_dir) {
                            Ok(metadata) => {
                                snapshot.exports.insert(
                                    format!("{}_{}", graph_type.name, "pytorch_geometric"),
                                    GraphExportInfo {
                                        name: graph_type.name.clone(),
                                        format: "pytorch_geometric".to_string(),
                                        output_path: format_dir.clone(),
                                        node_count: metadata.num_nodes,
                                        edge_count: metadata.num_edges,
                                    },
                                );
                                snapshot.graph_count += 1;
                            }
                            Err(e) => {
                                warn!("Failed to export PyTorch Geometric graph: {}", e);
                            }
                        }
                    }
                    datasynth_config::schema::GraphExportFormat::Neo4j => {
                        // Neo4j export will be added in a future update
                        debug!("Neo4j export not yet implemented for accounting networks");
                    }
                    datasynth_config::schema::GraphExportFormat::Dgl => {
                        // DGL export will be added in a future update
                        debug!("DGL export not yet implemented for accounting networks");
                    }
                    datasynth_config::schema::GraphExportFormat::RustGraph => {
                        use datasynth_graph::{
                            RustGraphExportConfig, RustGraphExporter, RustGraphOutputFormat,
                        };

                        let rustgraph_config = RustGraphExportConfig {
                            include_features: true,
                            include_temporal: true,
                            include_labels: true,
                            source_name: "datasynth".to_string(),
                            batch_id: None,
                            output_format: RustGraphOutputFormat::JsonLines,
                            export_node_properties: true,
                            export_edge_properties: true,
                            pretty_print: false,
                        };

                        let exporter = RustGraphExporter::new(rustgraph_config);
                        match exporter.export(&graph, &format_dir) {
                            Ok(metadata) => {
                                snapshot.exports.insert(
                                    format!("{}_{}", graph_type.name, "rustgraph"),
                                    GraphExportInfo {
                                        name: graph_type.name.clone(),
                                        format: "rustgraph".to_string(),
                                        output_path: format_dir.clone(),
                                        node_count: metadata.num_nodes,
                                        edge_count: metadata.num_edges,
                                    },
                                );
                                snapshot.graph_count += 1;
                            }
                            Err(e) => {
                                warn!("Failed to export RustGraph: {}", e);
                            }
                        }
                    }
                }
            }

            if let Some(pb) = &pb {
                pb.inc(40);
            }
        }

        stats.graph_export_count = snapshot.graph_count;
        snapshot.exported = snapshot.graph_count > 0;

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Graphs exported: {} graphs ({} nodes, {} edges)",
                snapshot.graph_count, stats.graph_node_count, stats.graph_edge_count
            ));
        }

        Ok(snapshot)
    }

    /// Generate banking KYC/AML data.
    ///
    /// Creates banking customers, accounts, and transactions with AML typology injection.
    /// Uses the BankingOrchestrator from synth-banking crate.
    fn generate_banking_data(&mut self) -> SynthResult<BankingSnapshot> {
        let pb = self.create_progress_bar(100, "Generating Banking Data");

        // Build the banking orchestrator from config
        let orchestrator = BankingOrchestratorBuilder::new()
            .config(self.config.banking.clone())
            .seed(self.seed + 9000)
            .build();

        if let Some(pb) = &pb {
            pb.inc(10);
        }

        // Generate the banking data
        let result = orchestrator.generate();

        if let Some(pb) = &pb {
            pb.inc(90);
            pb.finish_with_message(format!(
                "Banking: {} customers, {} transactions",
                result.customers.len(),
                result.transactions.len()
            ));
        }

        Ok(BankingSnapshot {
            customers: result.customers,
            accounts: result.accounts,
            transactions: result.transactions,
            suspicious_count: result.stats.suspicious_count,
            scenario_count: result.scenarios.len(),
        })
    }

    /// Calculate total transactions to generate.
    fn calculate_total_transactions(&self) -> u64 {
        let months = self.config.global.period_months as f64;
        self.config
            .companies
            .iter()
            .map(|c| {
                let annual = c.annual_transaction_volume.count() as f64;
                let weighted = annual * c.volume_weight;
                (weighted * months / 12.0) as u64
            })
            .sum()
    }

    /// Create a progress bar if progress display is enabled.
    fn create_progress_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.phase_config.show_progress {
            return None;
        }

        let pb = if let Some(mp) = &self.multi_progress {
            mp.add(ProgressBar::new(total))
        } else {
            ProgressBar::new(total)
        };

        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} {} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{pos}}/{{len}} ({{per_sec}})",
                    message
                ))
                .expect("Progress bar template should be valid - uses only standard indicatif placeholders")
                .progress_chars("#>-"),
        );

        Some(pb)
    }

    /// Get the generated chart of accounts.
    pub fn get_coa(&self) -> Option<Arc<ChartOfAccounts>> {
        self.coa.clone()
    }

    /// Get the generated master data.
    pub fn get_master_data(&self) -> &MasterDataSnapshot {
        &self.master_data
    }
}

/// Get the directory name for a graph export format.
fn format_name(format: datasynth_config::schema::GraphExportFormat) -> &'static str {
    match format {
        datasynth_config::schema::GraphExportFormat::PytorchGeometric => "pytorch_geometric",
        datasynth_config::schema::GraphExportFormat::Neo4j => "neo4j",
        datasynth_config::schema::GraphExportFormat::Dgl => "dgl",
        datasynth_config::schema::GraphExportFormat::RustGraph => "rustgraph",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_config::schema::*;

    fn create_test_config() -> GeneratorConfig {
        GeneratorConfig {
            global: GlobalConfig {
                industry: IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 1,
                seed: Some(42),
                parallel: false,
                group_currency: "USD".to_string(),
                worker_threads: 0,
                memory_limit_mb: 0,
            },
            companies: vec![CompanyConfig {
                code: "1000".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
                fiscal_year_variant: "K4".to_string(),
            }],
            chart_of_accounts: ChartOfAccountsConfig {
                complexity: CoAComplexity::Small,
                industry_specific: true,
                custom_accounts: None,
                min_hierarchy_depth: 2,
                max_hierarchy_depth: 4,
            },
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
            internal_controls: InternalControlsConfig::default(),
            business_processes: BusinessProcessConfig::default(),
            user_personas: UserPersonaConfig::default(),
            templates: TemplateConfig::default(),
            approval: ApprovalConfig::default(),
            departments: DepartmentConfig::default(),
            master_data: MasterDataConfig::default(),
            document_flows: DocumentFlowConfig::default(),
            intercompany: IntercompanyConfig::default(),
            balance: BalanceConfig::default(),
            ocpm: OcpmConfig::default(),
            audit: AuditGenerationConfig::default(),
            banking: datasynth_banking::BankingConfig::default(),
            data_quality: DataQualitySchemaConfig::default(),
            scenario: ScenarioConfig::default(),
            temporal: TemporalDriftConfig::default(),
            graph_export: GraphExportConfig::default(),
            streaming: StreamingSchemaConfig::default(),
            rate_limit: RateLimitSchemaConfig::default(),
            temporal_attributes: TemporalAttributeSchemaConfig::default(),
            relationships: RelationshipSchemaConfig::default(),
            accounting_standards: AccountingStandardsConfig::default(),
            audit_standards: AuditStandardsConfig::default(),
            distributions: Default::default(),
            temporal_patterns: Default::default(),
            vendor_network: VendorNetworkSchemaConfig::default(),
            customer_segmentation: CustomerSegmentationSchemaConfig::default(),
            relationship_strength: RelationshipStrengthSchemaConfig::default(),
            cross_process_links: CrossProcessLinksSchemaConfig::default(),
            organizational_events: OrganizationalEventsSchemaConfig::default(),
            behavioral_drift: BehavioralDriftSchemaConfig::default(),
            market_drift: MarketDriftSchemaConfig::default(),
            drift_labeling: DriftLabelingSchemaConfig::default(),
            anomaly_injection: Default::default(),
        }
    }

    #[test]
    fn test_enhanced_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config);
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_minimal_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.journal_entries.is_empty());
    }

    #[test]
    fn test_master_data_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 5,
            employees_per_company: 10,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        assert!(!result.master_data.vendors.is_empty());
        assert!(!result.master_data.customers.is_empty());
        assert!(!result.master_data.materials.is_empty());
    }

    #[test]
    fn test_document_flow_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: false,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: false,
            generate_ocpm_events: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 5,
            employees_per_company: 10,
            p2p_chains: 5,
            o2c_chains: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have generated P2P and O2C chains
        assert!(!result.document_flows.p2p_chains.is_empty());
        assert!(!result.document_flows.o2c_chains.is_empty());

        // Flattened documents should be populated
        assert!(!result.document_flows.purchase_orders.is_empty());
        assert!(!result.document_flows.sales_orders.is_empty());
    }

    #[test]
    fn test_anomaly_injection() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: true,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have journal entries
        assert!(!result.journal_entries.is_empty());

        // With ~833 entries and 2% rate, expect some anomalies
        // Note: This is probabilistic, so we just verify the structure exists
        assert!(result.anomaly_labels.summary.is_some());
    }

    #[test]
    fn test_full_generation_pipeline() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: true,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: true,
            generate_ocpm_events: false,
            show_progress: false,
            vendors_per_company: 3,
            customers_per_company: 3,
            materials_per_company: 5,
            assets_per_company: 3,
            employees_per_company: 5,
            p2p_chains: 3,
            o2c_chains: 3,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // All phases should have results
        assert!(!result.master_data.vendors.is_empty());
        assert!(!result.master_data.customers.is_empty());
        assert!(!result.document_flows.p2p_chains.is_empty());
        assert!(!result.document_flows.o2c_chains.is_empty());
        assert!(!result.journal_entries.is_empty());
        assert!(result.statistics.accounts_count > 0);

        // Subledger linking should have run
        assert!(!result.subledger.ap_invoices.is_empty());
        assert!(!result.subledger.ar_invoices.is_empty());

        // Balance validation should have run
        assert!(result.balance_validation.validated);
        assert!(result.balance_validation.entries_processed > 0);
    }

    #[test]
    fn test_subledger_linking() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: false,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: false,
            generate_ocpm_events: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 3,
            employees_per_company: 5,
            p2p_chains: 5,
            o2c_chains: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have document flows
        assert!(!result.document_flows.vendor_invoices.is_empty());
        assert!(!result.document_flows.customer_invoices.is_empty());

        // Subledger should be linked from document flows
        assert!(!result.subledger.ap_invoices.is_empty());
        assert!(!result.subledger.ar_invoices.is_empty());

        // AP invoices count should match vendor invoices count
        assert_eq!(
            result.subledger.ap_invoices.len(),
            result.document_flows.vendor_invoices.len()
        );

        // AR invoices count should match customer invoices count
        assert_eq!(
            result.subledger.ar_invoices.len(),
            result.document_flows.customer_invoices.len()
        );

        // Statistics should reflect subledger counts
        assert_eq!(
            result.statistics.ap_invoice_count,
            result.subledger.ap_invoices.len()
        );
        assert_eq!(
            result.statistics.ar_invoice_count,
            result.subledger.ar_invoices.len()
        );
    }

    #[test]
    fn test_balance_validation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            validate_balances: true,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Balance validation should run
        assert!(result.balance_validation.validated);
        assert!(result.balance_validation.entries_processed > 0);

        // Generated JEs should be balanced (no unbalanced entries)
        assert!(!result.balance_validation.has_unbalanced_entries);

        // Total debits should equal total credits
        assert_eq!(
            result.balance_validation.total_debits,
            result.balance_validation.total_credits
        );
    }

    #[test]
    fn test_statistics_accuracy() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 10,
            customers_per_company: 20,
            materials_per_company: 15,
            assets_per_company: 5,
            employees_per_company: 8,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Statistics should match actual data
        assert_eq!(
            result.statistics.vendor_count,
            result.master_data.vendors.len()
        );
        assert_eq!(
            result.statistics.customer_count,
            result.master_data.customers.len()
        );
        assert_eq!(
            result.statistics.material_count,
            result.master_data.materials.len()
        );
        assert_eq!(
            result.statistics.total_entries as usize,
            result.journal_entries.len()
        );
    }

    #[test]
    fn test_phase_config_defaults() {
        let config = PhaseConfig::default();
        assert!(config.generate_master_data);
        assert!(config.generate_document_flows);
        assert!(config.generate_journal_entries);
        assert!(!config.inject_anomalies);
        assert!(config.validate_balances);
        assert!(config.show_progress);
        assert!(config.vendors_per_company > 0);
        assert!(config.customers_per_company > 0);
    }

    #[test]
    fn test_get_coa_before_generation() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config).unwrap();

        // Before generation, CoA should be None
        assert!(orchestrator.get_coa().is_none());
    }

    #[test]
    fn test_get_coa_after_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let _ = orchestrator.generate().unwrap();

        // After generation, CoA should be available
        assert!(orchestrator.get_coa().is_some());
    }

    #[test]
    fn test_get_master_data() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 5,
            assets_per_company: 5,
            employees_per_company: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let _ = orchestrator.generate().unwrap();

        let master_data = orchestrator.get_master_data();
        assert!(!master_data.vendors.is_empty());
    }

    #[test]
    fn test_with_progress_builder() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config)
            .unwrap()
            .with_progress(false);

        // Should still work without progress
        assert!(!orchestrator.phase_config.show_progress);
    }

    #[test]
    fn test_multi_company_generation() {
        let mut config = create_test_config();
        config.companies.push(CompanyConfig {
            code: "2000".to_string(),
            name: "Subsidiary".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
            fiscal_year_variant: "K4".to_string(),
        });

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 5,
            assets_per_company: 5,
            employees_per_company: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have master data for both companies
        assert!(result.statistics.vendor_count >= 10); // 5 per company
        assert!(result.statistics.customer_count >= 10);
        assert!(result.statistics.companies_count == 2);
    }

    #[test]
    fn test_empty_master_data_skips_document_flows() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,   // Skip master data
            generate_document_flows: true, // Try to generate flows
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Without master data, document flows should be empty
        assert!(result.document_flows.p2p_chains.is_empty());
        assert!(result.document_flows.o2c_chains.is_empty());
    }

    #[test]
    fn test_journal_entry_line_item_count() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Total line items should match sum of all entry line counts
        let calculated_line_items: u64 = result
            .journal_entries
            .iter()
            .map(|e| e.line_count() as u64)
            .sum();
        assert_eq!(result.statistics.total_line_items, calculated_line_items);
    }

    #[test]
    fn test_audit_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            generate_audit: true,
            audit_engagements: 2,
            workpapers_per_engagement: 5,
            evidence_per_workpaper: 2,
            risks_per_engagement: 3,
            findings_per_engagement: 2,
            judgments_per_engagement: 2,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have generated audit data
        assert_eq!(result.audit.engagements.len(), 2);
        assert!(!result.audit.workpapers.is_empty());
        assert!(!result.audit.evidence.is_empty());
        assert!(!result.audit.risk_assessments.is_empty());
        assert!(!result.audit.findings.is_empty());
        assert!(!result.audit.judgments.is_empty());

        // Statistics should match
        assert_eq!(
            result.statistics.audit_engagement_count,
            result.audit.engagements.len()
        );
        assert_eq!(
            result.statistics.audit_workpaper_count,
            result.audit.workpapers.len()
        );
        assert_eq!(
            result.statistics.audit_evidence_count,
            result.audit.evidence.len()
        );
        assert_eq!(
            result.statistics.audit_risk_count,
            result.audit.risk_assessments.len()
        );
        assert_eq!(
            result.statistics.audit_finding_count,
            result.audit.findings.len()
        );
        assert_eq!(
            result.statistics.audit_judgment_count,
            result.audit.judgments.len()
        );
    }
}
