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
//! 11. LLM enrichment (AI-augmented vendor names, descriptions)
//! 12. Diffusion enhancement (statistical diffusion-based sample generation)
//! 13. Causal overlay (structural causal model generation and validation)
//! 14. Source-to-Contract (S2C) sourcing data generation
//! 15. Bank reconciliation generation
//! 16. Financial statement generation

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
use datasynth_core::models::sourcing::{
    BidEvaluation, CatalogItem, ProcurementContract, RfxEvent, SourcingProject, SpendAnalysis,
    SupplierBid, SupplierQualification, SupplierScorecard,
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
    // Bank reconciliation generator
    BankReconciliationGenerator,
    // S2C sourcing generators
    BidEvaluationGenerator,
    BidGenerator,
    CatalogGenerator,
    // Core generators
    ChartOfAccountsGenerator,
    ContractGenerator,
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
    // Financial statement generator
    FinancialStatementGenerator,
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
    PaymentReference,
    QualificationGenerator,
    RfxGenerator,
    RiskAssessmentGenerator,
    // Balance validation
    RunningBalanceTracker,
    ScorecardGenerator,
    SourcingProjectGenerator,
    SpendAnalysisGenerator,
    ValidationError,
    // Master data generators
    VendorGenerator,
    WorkpaperGenerator,
};
use datasynth_graph::{
    PyGExportConfig, PyGExporter, TransactionGraphBuilder, TransactionGraphConfig,
};
use datasynth_ocpm::{
    AuditDocuments, BankDocuments, BankReconDocuments, EventLogMetadata, H2rDocuments,
    MfgDocuments, O2cDocuments, OcpmEventGenerator, OcpmEventLog, OcpmGeneratorConfig,
    P2pDocuments, S2cDocuments,
};

use datasynth_config::schema::{O2CFlowConfig, P2PFlowConfig};
use datasynth_core::causal::{CausalGraph, CausalValidator, StructuralCausalModel};
use datasynth_core::diffusion::{DiffusionBackend, DiffusionConfig, StatisticalDiffusionBackend};
use datasynth_core::llm::MockLlmProvider;
use datasynth_core::models::documents::PaymentMethod;
use datasynth_generators::llm_enrichment::VendorLlmEnricher;

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
    /// Generate S2C sourcing data (spend analysis, RFx, bids, contracts, catalogs, scorecards).
    pub generate_sourcing: bool,
    /// Generate bank reconciliations from payments.
    pub generate_bank_reconciliation: bool,
    /// Generate financial statements from trial balances.
    pub generate_financial_statements: bool,
    /// Generate accounting standards data (revenue recognition, impairment).
    pub generate_accounting_standards: bool,
    /// Generate manufacturing data (production orders, quality inspections, cycle counts).
    pub generate_manufacturing: bool,
    /// Generate sales quotes, management KPIs, and budgets.
    pub generate_sales_kpi_budgets: bool,
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
            generate_banking: false,              // Off by default
            generate_graph_export: false,         // Off by default
            generate_sourcing: false,             // Off by default
            generate_bank_reconciliation: false,  // Off by default
            generate_financial_statements: false, // Off by default
            generate_accounting_standards: false, // Off by default
            generate_manufacturing: false,        // Off by default
            generate_sales_kpi_budgets: false,    // Off by default
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

/// Info about a completed hypergraph export.
#[derive(Debug, Clone)]
pub struct HypergraphExportInfo {
    /// Number of nodes exported.
    pub node_count: usize,
    /// Number of pairwise edges exported.
    pub edge_count: usize,
    /// Number of hyperedges exported.
    pub hyperedge_count: usize,
    /// Output directory path.
    pub output_path: PathBuf,
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

/// S2C sourcing data snapshot.
#[derive(Debug, Clone, Default)]
pub struct SourcingSnapshot {
    /// Spend analyses.
    pub spend_analyses: Vec<SpendAnalysis>,
    /// Sourcing projects.
    pub sourcing_projects: Vec<SourcingProject>,
    /// Supplier qualifications.
    pub qualifications: Vec<SupplierQualification>,
    /// RFx events (RFI, RFP, RFQ).
    pub rfx_events: Vec<RfxEvent>,
    /// Supplier bids.
    pub bids: Vec<SupplierBid>,
    /// Bid evaluations.
    pub bid_evaluations: Vec<BidEvaluation>,
    /// Procurement contracts.
    pub contracts: Vec<ProcurementContract>,
    /// Catalog items.
    pub catalog_items: Vec<CatalogItem>,
    /// Supplier scorecards.
    pub scorecards: Vec<SupplierScorecard>,
}

/// Financial reporting snapshot (financial statements + bank reconciliations).
#[derive(Debug, Clone, Default)]
pub struct FinancialReportingSnapshot {
    /// Financial statements (balance sheet, income statement, cash flow).
    pub financial_statements: Vec<FinancialStatement>,
    /// Bank reconciliations.
    pub bank_reconciliations: Vec<BankReconciliation>,
}

/// HR data snapshot (payroll runs, time entries, expense reports).
#[derive(Debug, Clone, Default)]
pub struct HrSnapshot {
    /// Payroll runs (actual data).
    pub payroll_runs: Vec<PayrollRun>,
    /// Payroll line items (actual data).
    pub payroll_line_items: Vec<PayrollLineItem>,
    /// Time entries (actual data).
    pub time_entries: Vec<TimeEntry>,
    /// Expense reports (actual data).
    pub expense_reports: Vec<ExpenseReport>,
    /// Payroll runs.
    pub payroll_run_count: usize,
    /// Payroll line item count.
    pub payroll_line_item_count: usize,
    /// Time entry count.
    pub time_entry_count: usize,
    /// Expense report count.
    pub expense_report_count: usize,
}

/// Accounting standards data snapshot (revenue recognition, impairment).
#[derive(Debug, Clone, Default)]
pub struct AccountingStandardsSnapshot {
    /// Revenue recognition contract count.
    pub revenue_contract_count: usize,
    /// Impairment test count.
    pub impairment_test_count: usize,
}

/// Manufacturing data snapshot (production orders, quality inspections, cycle counts).
#[derive(Debug, Clone, Default)]
pub struct ManufacturingSnapshot {
    /// Production orders (actual data).
    pub production_orders: Vec<ProductionOrder>,
    /// Quality inspections (actual data).
    pub quality_inspections: Vec<QualityInspection>,
    /// Cycle counts (actual data).
    pub cycle_counts: Vec<CycleCount>,
    /// Production order count.
    pub production_order_count: usize,
    /// Quality inspection count.
    pub quality_inspection_count: usize,
    /// Cycle count count.
    pub cycle_count_count: usize,
}

/// Sales, KPI, and budget data snapshot.
#[derive(Debug, Clone, Default)]
pub struct SalesKpiBudgetsSnapshot {
    /// Sales quotes (actual data).
    pub sales_quotes: Vec<SalesQuote>,
    /// Management KPIs (actual data).
    pub kpis: Vec<ManagementKpi>,
    /// Budgets (actual data).
    pub budgets: Vec<Budget>,
    /// Sales quote count.
    pub sales_quote_count: usize,
    /// Management KPI count.
    pub kpi_count: usize,
    /// Budget line count.
    pub budget_line_count: usize,
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
    /// S2C sourcing data snapshot (if sourcing generation enabled).
    pub sourcing: SourcingSnapshot,
    /// Financial reporting snapshot (financial statements + bank reconciliations).
    pub financial_reporting: FinancialReportingSnapshot,
    /// HR data snapshot (payroll, time entries, expenses).
    pub hr: HrSnapshot,
    /// Accounting standards snapshot (revenue recognition, impairment).
    pub accounting_standards: AccountingStandardsSnapshot,
    /// Manufacturing snapshot (production orders, quality inspections, cycle counts).
    pub manufacturing: ManufacturingSnapshot,
    /// Sales, KPI, and budget snapshot.
    pub sales_kpi_budgets: SalesKpiBudgetsSnapshot,
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
    /// Data lineage graph (if tracking enabled).
    pub lineage: Option<super::lineage::LineageGraph>,
    /// Quality gate evaluation result.
    pub gate_result: Option<datasynth_eval::gates::GateResult>,
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
    /// LLM enrichment timing (milliseconds).
    #[serde(default)]
    pub llm_enrichment_ms: u64,
    /// Number of vendor names enriched by LLM.
    #[serde(default)]
    pub llm_vendors_enriched: usize,
    /// Diffusion enhancement timing (milliseconds).
    #[serde(default)]
    pub diffusion_enhancement_ms: u64,
    /// Number of diffusion samples generated.
    #[serde(default)]
    pub diffusion_samples_generated: usize,
    /// Causal generation timing (milliseconds).
    #[serde(default)]
    pub causal_generation_ms: u64,
    /// Number of causal samples generated.
    #[serde(default)]
    pub causal_samples_generated: usize,
    /// Whether causal validation passed.
    #[serde(default)]
    pub causal_validation_passed: Option<bool>,
    /// S2C sourcing counts.
    #[serde(default)]
    pub sourcing_project_count: usize,
    #[serde(default)]
    pub rfx_event_count: usize,
    #[serde(default)]
    pub bid_count: usize,
    #[serde(default)]
    pub contract_count: usize,
    #[serde(default)]
    pub catalog_item_count: usize,
    #[serde(default)]
    pub scorecard_count: usize,
    /// Financial reporting counts.
    #[serde(default)]
    pub financial_statement_count: usize,
    #[serde(default)]
    pub bank_reconciliation_count: usize,
    /// HR counts.
    #[serde(default)]
    pub payroll_run_count: usize,
    #[serde(default)]
    pub time_entry_count: usize,
    #[serde(default)]
    pub expense_report_count: usize,
    /// Accounting standards counts.
    #[serde(default)]
    pub revenue_contract_count: usize,
    #[serde(default)]
    pub impairment_test_count: usize,
    /// Manufacturing counts.
    #[serde(default)]
    pub production_order_count: usize,
    #[serde(default)]
    pub quality_inspection_count: usize,
    #[serde(default)]
    pub cycle_count_count: usize,
    /// Sales & reporting counts.
    #[serde(default)]
    pub sales_quote_count: usize,
    #[serde(default)]
    pub kpi_count: usize,
    #[serde(default)]
    pub budget_line_count: usize,
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

        let mut stats = EnhancedGenerationStatistics {
            companies_count: self.config.companies.len(),
            period_months: self.config.global.period_months,
            ..Default::default()
        };

        // Phase 1: Chart of Accounts
        let coa = self.phase_chart_of_accounts(&mut stats)?;

        // Phase 2: Master Data
        self.phase_master_data(&mut stats)?;

        // Phase 3: Document Flows + Subledger Linking
        let (document_flows, subledger) = self.phase_document_flows(&mut stats)?;

        // Phase 4: Journal Entries
        let mut entries = self.phase_journal_entries(&coa, &document_flows, &mut stats)?;

        // Get current degradation actions for optional phases
        let actions = self.get_degradation_actions();

        // Phase 5: Anomaly Injection
        let anomaly_labels = self.phase_anomaly_injection(&mut entries, &actions, &mut stats)?;

        // Phase 6: Balance Validation
        let balance_validation = self.phase_balance_validation(&entries)?;

        // Phase 7: Data Quality Injection
        let data_quality_stats =
            self.phase_data_quality_injection(&mut entries, &actions, &mut stats)?;

        // Phase 8: Audit Data
        let audit = self.phase_audit_data(&entries, &mut stats)?;

        // Phase 9: Banking KYC/AML Data
        let banking = self.phase_banking_data(&mut stats)?;

        // Phase 10: Graph Export
        let graph_export = self.phase_graph_export(&entries, &coa, &mut stats)?;

        // Phase 11: LLM Enrichment
        self.phase_llm_enrichment(&mut stats);

        // Phase 12: Diffusion Enhancement
        self.phase_diffusion_enhancement(&mut stats);

        // Phase 13: Causal Overlay
        self.phase_causal_overlay(&mut stats);

        // Phase 14: S2C Sourcing Data
        let sourcing = self.phase_sourcing_data(&mut stats)?;

        // Phase 15: Bank Reconciliation + Financial Statements
        let financial_reporting = self.phase_financial_reporting(&document_flows, &mut stats)?;

        // Phase 16: HR Data (Payroll, Time Entries, Expenses)
        let hr = self.phase_hr_data(&mut stats)?;

        // Phase 17: Accounting Standards (Revenue Recognition, Impairment)
        let accounting_standards = self.phase_accounting_standards(&mut stats)?;

        // Phase 18: Manufacturing (Production Orders, Quality Inspections, Cycle Counts)
        let manufacturing_snap = self.phase_manufacturing(&mut stats)?;

        // Phase 18b: OCPM Events (after all process data is available)
        let ocpm = self.phase_ocpm_events(
            &document_flows,
            &sourcing,
            &hr,
            &manufacturing_snap,
            &banking,
            &audit,
            &financial_reporting,
            &mut stats,
        )?;

        // Phase 19: Sales Quotes, Management KPIs, Budgets
        let sales_kpi_budgets = self.phase_sales_kpi_budgets(&coa, &mut stats)?;

        // Phase 19b: Hypergraph Export (after all data is available)
        self.phase_hypergraph_export(
            &coa,
            &entries,
            &document_flows,
            &sourcing,
            &hr,
            &manufacturing_snap,
            &banking,
            &audit,
            &financial_reporting,
            &ocpm,
            &mut stats,
        )?;

        // Log final resource statistics
        let resource_stats = self.resource_guard.stats();
        info!(
            "Generation workflow complete. Resource stats: memory_peak={}MB, disk_written={}bytes, degradation_level={}",
            resource_stats.memory.peak_resident_bytes / (1024 * 1024),
            resource_stats.disk.estimated_bytes_written,
            resource_stats.degradation_level
        );

        // Build data lineage graph
        let lineage = self.build_lineage_graph();

        Ok(EnhancedGenerationResult {
            chart_of_accounts: (*coa).clone(),
            master_data: self.master_data.clone(),
            document_flows,
            subledger,
            ocpm,
            audit,
            banking,
            graph_export,
            sourcing,
            financial_reporting,
            hr,
            accounting_standards,
            manufacturing: manufacturing_snap,
            sales_kpi_budgets,
            journal_entries: entries,
            anomaly_labels,
            balance_validation,
            data_quality_stats,
            statistics: stats,
            lineage: Some(lineage),
            gate_result: None,
        })
    }

    // ========================================================================
    // Generation Phase Methods
    // ========================================================================

    /// Phase 1: Generate Chart of Accounts and update statistics.
    fn phase_chart_of_accounts(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Arc<ChartOfAccounts>> {
        info!("Phase 1: Generating Chart of Accounts");
        let coa = self.generate_coa()?;
        stats.accounts_count = coa.account_count();
        info!(
            "Chart of Accounts generated: {} accounts",
            stats.accounts_count
        );
        self.check_resources_with_log("post-coa")?;
        Ok(coa)
    }

    /// Phase 2: Generate master data (vendors, customers, materials, assets, employees).
    fn phase_master_data(&mut self, stats: &mut EnhancedGenerationStatistics) -> SynthResult<()> {
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
            self.check_resources_with_log("post-master-data")?;
        } else {
            debug!("Phase 2: Skipped (master data generation disabled)");
        }
        Ok(())
    }

    /// Phase 3: Generate document flows (P2P and O2C) and link to subledgers.
    fn phase_document_flows(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<(DocumentFlowSnapshot, SubledgerSnapshot)> {
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

            self.check_resources_with_log("post-document-flows")?;
        } else {
            debug!("Phase 3: Skipped (document flow generation disabled or no master data)");
        }

        Ok((document_flows, subledger))
    }

    /// Phase 3c: Generate OCPM events from document flows.
    #[allow(clippy::too_many_arguments)]
    fn phase_ocpm_events(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        sourcing: &SourcingSnapshot,
        hr: &HrSnapshot,
        manufacturing: &ManufacturingSnapshot,
        banking: &BankingSnapshot,
        audit: &AuditSnapshot,
        financial_reporting: &FinancialReportingSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<OcpmSnapshot> {
        if self.phase_config.generate_ocpm_events {
            info!("Phase 3c: Generating OCPM Events");
            let ocpm_snapshot = self.generate_ocpm_events(
                document_flows,
                sourcing,
                hr,
                manufacturing,
                banking,
                audit,
                financial_reporting,
            )?;
            stats.ocpm_event_count = ocpm_snapshot.event_count;
            stats.ocpm_object_count = ocpm_snapshot.object_count;
            stats.ocpm_case_count = ocpm_snapshot.case_count;
            info!(
                "OCPM events generated: {} events, {} objects, {} cases",
                stats.ocpm_event_count, stats.ocpm_object_count, stats.ocpm_case_count
            );
            self.check_resources_with_log("post-ocpm")?;
            Ok(ocpm_snapshot)
        } else {
            debug!("Phase 3c: Skipped (OCPM generation disabled or no document flows)");
            Ok(OcpmSnapshot::default())
        }
    }

    /// Phase 4: Generate journal entries from document flows and standalone generation.
    fn phase_journal_entries(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<JournalEntry>> {
        let mut entries = Vec::new();

        // Phase 4a: Generate JEs from document flows (for data coherence)
        if self.phase_config.generate_document_flows && !document_flows.p2p_chains.is_empty() {
            debug!("Phase 4a: Generating JEs from document flows");
            let flow_entries = self.generate_jes_from_document_flows(document_flows)?;
            debug!("Generated {} JEs from document flows", flow_entries.len());
            entries.extend(flow_entries);
        }

        // Phase 4b: Generate standalone journal entries
        if self.phase_config.generate_journal_entries {
            info!("Phase 4: Generating Journal Entries");
            let je_entries = self.generate_journal_entries(coa)?;
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
            self.check_resources_with_log("post-journal-entries")?;
        }

        Ok(entries)
    }

    /// Phase 5: Inject anomalies into journal entries.
    fn phase_anomaly_injection(
        &mut self,
        entries: &mut [JournalEntry],
        actions: &DegradationActions,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<AnomalyLabels> {
        if self.phase_config.inject_anomalies
            && !entries.is_empty()
            && !actions.skip_anomaly_injection
        {
            info!("Phase 5: Injecting Anomalies");
            let result = self.inject_anomalies(entries)?;
            stats.anomalies_injected = result.labels.len();
            info!("Injected {} anomalies", stats.anomalies_injected);
            self.check_resources_with_log("post-anomaly-injection")?;
            Ok(result)
        } else if actions.skip_anomaly_injection {
            warn!("Phase 5: Skipped due to resource degradation");
            Ok(AnomalyLabels::default())
        } else {
            debug!("Phase 5: Skipped (anomaly injection disabled or no entries)");
            Ok(AnomalyLabels::default())
        }
    }

    /// Phase 6: Validate balance sheet equation on journal entries.
    fn phase_balance_validation(
        &mut self,
        entries: &[JournalEntry],
    ) -> SynthResult<BalanceValidationResult> {
        if self.phase_config.validate_balances && !entries.is_empty() {
            debug!("Phase 6: Validating Balances");
            let balance_validation = self.validate_journal_entries(entries)?;
            if balance_validation.is_balanced {
                debug!("Balance validation passed");
            } else {
                warn!(
                    "Balance validation found {} errors",
                    balance_validation.validation_errors.len()
                );
            }
            Ok(balance_validation)
        } else {
            Ok(BalanceValidationResult::default())
        }
    }

    /// Phase 7: Inject data quality variations (typos, missing values, format issues).
    fn phase_data_quality_injection(
        &mut self,
        entries: &mut [JournalEntry],
        actions: &DegradationActions,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<DataQualityStats> {
        if self.phase_config.inject_data_quality
            && !entries.is_empty()
            && !actions.skip_data_quality
        {
            info!("Phase 7: Injecting Data Quality Variations");
            let dq_stats = self.inject_data_quality(entries)?;
            stats.data_quality_issues = dq_stats.records_with_issues;
            info!("Injected {} data quality issues", stats.data_quality_issues);
            self.check_resources_with_log("post-data-quality")?;
            Ok(dq_stats)
        } else if actions.skip_data_quality {
            warn!("Phase 7: Skipped due to resource degradation");
            Ok(DataQualityStats::default())
        } else {
            debug!("Phase 7: Skipped (data quality injection disabled or no entries)");
            Ok(DataQualityStats::default())
        }
    }

    /// Phase 8: Generate audit data (engagements, workpapers, evidence, risks, findings).
    fn phase_audit_data(
        &mut self,
        entries: &[JournalEntry],
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<AuditSnapshot> {
        if self.phase_config.generate_audit {
            info!("Phase 8: Generating Audit Data");
            let audit_snapshot = self.generate_audit_data(entries)?;
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
            self.check_resources_with_log("post-audit")?;
            Ok(audit_snapshot)
        } else {
            debug!("Phase 8: Skipped (audit generation disabled)");
            Ok(AuditSnapshot::default())
        }
    }

    /// Phase 9: Generate banking KYC/AML data.
    fn phase_banking_data(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<BankingSnapshot> {
        if self.phase_config.generate_banking && self.config.banking.enabled {
            info!("Phase 9: Generating Banking KYC/AML Data");
            let banking_snapshot = self.generate_banking_data()?;
            stats.banking_customer_count = banking_snapshot.customers.len();
            stats.banking_account_count = banking_snapshot.accounts.len();
            stats.banking_transaction_count = banking_snapshot.transactions.len();
            stats.banking_suspicious_count = banking_snapshot.suspicious_count;
            info!(
                "Banking data generated: {} customers, {} accounts, {} transactions ({} suspicious)",
                stats.banking_customer_count, stats.banking_account_count,
                stats.banking_transaction_count, stats.banking_suspicious_count
            );
            self.check_resources_with_log("post-banking")?;
            Ok(banking_snapshot)
        } else {
            debug!("Phase 9: Skipped (banking generation disabled)");
            Ok(BankingSnapshot::default())
        }
    }

    /// Phase 10: Export accounting network graphs for ML training.
    fn phase_graph_export(
        &mut self,
        entries: &[JournalEntry],
        coa: &Arc<ChartOfAccounts>,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<GraphExportSnapshot> {
        if (self.phase_config.generate_graph_export || self.config.graph_export.enabled)
            && !entries.is_empty()
        {
            info!("Phase 10: Exporting Accounting Network Graphs");
            match self.export_graphs(entries, coa, stats) {
                Ok(snapshot) => {
                    info!(
                        "Graph export complete: {} graphs ({} nodes, {} edges)",
                        snapshot.graph_count, stats.graph_node_count, stats.graph_edge_count
                    );
                    Ok(snapshot)
                }
                Err(e) => {
                    warn!("Phase 10: Graph export failed: {}", e);
                    Ok(GraphExportSnapshot::default())
                }
            }
        } else {
            debug!("Phase 10: Skipped (graph export disabled or no entries)");
            Ok(GraphExportSnapshot::default())
        }
    }

    /// Phase 19b: Export multi-layer hypergraph for RustGraph integration.
    #[allow(clippy::too_many_arguments)]
    fn phase_hypergraph_export(
        &self,
        coa: &Arc<ChartOfAccounts>,
        entries: &[JournalEntry],
        document_flows: &DocumentFlowSnapshot,
        sourcing: &SourcingSnapshot,
        hr: &HrSnapshot,
        manufacturing: &ManufacturingSnapshot,
        banking: &BankingSnapshot,
        audit: &AuditSnapshot,
        financial_reporting: &FinancialReportingSnapshot,
        ocpm: &OcpmSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<()> {
        if self.config.graph_export.hypergraph.enabled && !entries.is_empty() {
            info!("Phase 19b: Exporting Multi-Layer Hypergraph");
            match self.export_hypergraph(
                coa,
                entries,
                document_flows,
                sourcing,
                hr,
                manufacturing,
                banking,
                audit,
                financial_reporting,
                ocpm,
                stats,
            ) {
                Ok(info) => {
                    info!(
                        "Hypergraph export complete: {} nodes, {} edges, {} hyperedges",
                        info.node_count, info.edge_count, info.hyperedge_count
                    );
                }
                Err(e) => {
                    warn!("Phase 10b: Hypergraph export failed: {}", e);
                }
            }
        } else {
            debug!("Phase 10b: Skipped (hypergraph export disabled or no entries)");
        }
        Ok(())
    }

    /// Phase 11: LLM Enrichment.
    ///
    /// Uses an LLM provider (mock by default) to enrich vendor names with
    /// realistic, context-aware names. This phase is non-blocking: failures
    /// log a warning but do not stop the generation pipeline.
    fn phase_llm_enrichment(&mut self, stats: &mut EnhancedGenerationStatistics) {
        if !self.config.llm.enabled {
            debug!("Phase 11: Skipped (LLM enrichment disabled)");
            return;
        }

        info!("Phase 11: Starting LLM Enrichment");
        let start = std::time::Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let provider = Arc::new(MockLlmProvider::new(self.seed));
            let enricher = VendorLlmEnricher::new(provider);

            let industry = format!("{:?}", self.config.global.industry);
            let max_enrichments = self
                .config
                .llm
                .max_vendor_enrichments
                .min(self.master_data.vendors.len());

            let mut enriched_count = 0usize;
            for vendor in self.master_data.vendors.iter_mut().take(max_enrichments) {
                match enricher.enrich_vendor_name(&industry, "general", &vendor.country) {
                    Ok(name) => {
                        vendor.name = name;
                        enriched_count += 1;
                    }
                    Err(e) => {
                        warn!(
                            "LLM vendor enrichment failed for {}: {}",
                            vendor.vendor_id, e
                        );
                    }
                }
            }

            enriched_count
        }));

        match result {
            Ok(enriched_count) => {
                stats.llm_vendors_enriched = enriched_count;
                let elapsed = start.elapsed();
                stats.llm_enrichment_ms = elapsed.as_millis() as u64;
                info!(
                    "Phase 11 complete: {} vendors enriched in {}ms",
                    enriched_count, stats.llm_enrichment_ms
                );
            }
            Err(_) => {
                let elapsed = start.elapsed();
                stats.llm_enrichment_ms = elapsed.as_millis() as u64;
                warn!("Phase 11: LLM enrichment failed (panic caught), continuing");
            }
        }
    }

    /// Phase 12: Diffusion Enhancement.
    ///
    /// Generates a sample set using the statistical diffusion backend to
    /// demonstrate distribution-matching data generation. This phase is
    /// non-blocking: failures log a warning but do not stop the pipeline.
    fn phase_diffusion_enhancement(&self, stats: &mut EnhancedGenerationStatistics) {
        if !self.config.diffusion.enabled {
            debug!("Phase 12: Skipped (diffusion enhancement disabled)");
            return;
        }

        info!("Phase 12: Starting Diffusion Enhancement");
        let start = std::time::Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Target distribution: transaction amounts (log-normal-like)
            let means = vec![5000.0, 3.0, 2.0]; // amount, line_items, approval_level
            let stds = vec![2000.0, 1.5, 1.0];

            let diffusion_config = DiffusionConfig {
                n_steps: self.config.diffusion.n_steps,
                seed: self.seed,
                ..Default::default()
            };

            let backend = StatisticalDiffusionBackend::new(means, stds, diffusion_config);

            let n_samples = self.config.diffusion.sample_size;
            let n_features = 3; // amount, line_items, approval_level
            let samples = backend.generate(n_samples, n_features, self.seed);

            samples.len()
        }));

        match result {
            Ok(sample_count) => {
                stats.diffusion_samples_generated = sample_count;
                let elapsed = start.elapsed();
                stats.diffusion_enhancement_ms = elapsed.as_millis() as u64;
                info!(
                    "Phase 12 complete: {} diffusion samples generated in {}ms",
                    sample_count, stats.diffusion_enhancement_ms
                );
            }
            Err(_) => {
                let elapsed = start.elapsed();
                stats.diffusion_enhancement_ms = elapsed.as_millis() as u64;
                warn!("Phase 12: Diffusion enhancement failed (panic caught), continuing");
            }
        }
    }

    /// Phase 13: Causal Overlay.
    ///
    /// Builds a structural causal model from a built-in template (e.g.,
    /// fraud_detection) and generates causal samples. Optionally validates
    /// that the output respects the causal structure. This phase is
    /// non-blocking: failures log a warning but do not stop the pipeline.
    fn phase_causal_overlay(&self, stats: &mut EnhancedGenerationStatistics) {
        if !self.config.causal.enabled {
            debug!("Phase 13: Skipped (causal generation disabled)");
            return;
        }

        info!("Phase 13: Starting Causal Overlay");
        let start = std::time::Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Select template based on config
            let graph = match self.config.causal.template.as_str() {
                "revenue_cycle" => CausalGraph::revenue_cycle_template(),
                _ => CausalGraph::fraud_detection_template(),
            };

            let scm = StructuralCausalModel::new(graph.clone())
                .map_err(|e| SynthError::generation(format!("Failed to build SCM: {}", e)))?;

            let n_samples = self.config.causal.sample_size;
            let samples = scm
                .generate(n_samples, self.seed)
                .map_err(|e| SynthError::generation(format!("SCM generation failed: {}", e)))?;

            // Optionally validate causal structure
            let validation_passed = if self.config.causal.validate {
                let report = CausalValidator::validate_causal_structure(&samples, &graph);
                if report.valid {
                    info!(
                        "Causal validation passed: all {} checks OK",
                        report.checks.len()
                    );
                } else {
                    warn!(
                        "Causal validation: {} violations detected: {:?}",
                        report.violations.len(),
                        report.violations
                    );
                }
                Some(report.valid)
            } else {
                None
            };

            Ok::<(usize, Option<bool>), SynthError>((samples.len(), validation_passed))
        }));

        match result {
            Ok(Ok((sample_count, validation_passed))) => {
                stats.causal_samples_generated = sample_count;
                stats.causal_validation_passed = validation_passed;
                let elapsed = start.elapsed();
                stats.causal_generation_ms = elapsed.as_millis() as u64;
                info!(
                    "Phase 13 complete: {} causal samples generated in {}ms (validation: {:?})",
                    sample_count, stats.causal_generation_ms, validation_passed,
                );
            }
            Ok(Err(e)) => {
                let elapsed = start.elapsed();
                stats.causal_generation_ms = elapsed.as_millis() as u64;
                warn!("Phase 13: Causal generation failed: {}", e);
            }
            Err(_) => {
                let elapsed = start.elapsed();
                stats.causal_generation_ms = elapsed.as_millis() as u64;
                warn!("Phase 13: Causal generation failed (panic caught), continuing");
            }
        }
    }

    /// Phase 14: Generate S2C sourcing data.
    fn phase_sourcing_data(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<SourcingSnapshot> {
        if !self.phase_config.generate_sourcing && !self.config.source_to_pay.enabled {
            debug!("Phase 14: Skipped (sourcing generation disabled)");
            return Ok(SourcingSnapshot::default());
        }

        info!("Phase 14: Generating S2C Sourcing Data");
        let seed = self.seed;

        // Gather vendor data from master data
        let vendor_ids: Vec<String> = self
            .master_data
            .vendors
            .iter()
            .map(|v| v.vendor_id.clone())
            .collect();
        if vendor_ids.is_empty() {
            debug!("Phase 14: Skipped (no vendors available)");
            return Ok(SourcingSnapshot::default());
        }

        let categories: Vec<(String, String)> = vec![
            ("CAT-RAW".to_string(), "Raw Materials".to_string()),
            ("CAT-OFF".to_string(), "Office Supplies".to_string()),
            ("CAT-IT".to_string(), "IT Equipment".to_string()),
            ("CAT-SVC".to_string(), "Professional Services".to_string()),
            ("CAT-LOG".to_string(), "Logistics".to_string()),
        ];
        let categories_with_spend: Vec<(String, String, rust_decimal::Decimal)> = categories
            .iter()
            .map(|(id, name)| {
                (
                    id.clone(),
                    name.clone(),
                    rust_decimal::Decimal::from(100_000),
                )
            })
            .collect();

        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let fiscal_year = start_date.year() as u16;
        let owner_ids: Vec<String> = self
            .master_data
            .employees
            .iter()
            .take(5)
            .map(|e| e.employee_id.clone())
            .collect();
        let owner_id = owner_ids.first().map(|s| s.as_str()).unwrap_or("BUYER-001");

        // Step 1: Spend Analysis
        let mut spend_gen = SpendAnalysisGenerator::new(seed);
        let spend_analyses =
            spend_gen.generate(company_code, &vendor_ids, &categories, fiscal_year);

        // Step 2: Sourcing Projects
        let mut project_gen = SourcingProjectGenerator::new(seed + 1);
        let sourcing_projects = if owner_ids.is_empty() {
            Vec::new()
        } else {
            project_gen.generate(
                company_code,
                &categories_with_spend,
                &owner_ids,
                start_date,
                self.config.global.period_months,
            )
        };
        stats.sourcing_project_count = sourcing_projects.len();

        // Step 3: Qualifications
        let qual_vendor_ids: Vec<String> = vendor_ids.iter().take(20).cloned().collect();
        let mut qual_gen = QualificationGenerator::new(seed + 2);
        let qualifications = qual_gen.generate(
            company_code,
            &qual_vendor_ids,
            sourcing_projects.first().map(|p| p.project_id.as_str()),
            owner_id,
            start_date,
        );

        // Step 4: RFx Events
        let mut rfx_gen = RfxGenerator::new(seed + 3);
        let rfx_events: Vec<RfxEvent> = sourcing_projects
            .iter()
            .map(|proj| {
                let qualified_vids: Vec<String> = vendor_ids.iter().take(5).cloned().collect();
                rfx_gen.generate(
                    company_code,
                    &proj.project_id,
                    &proj.category_id,
                    &qualified_vids,
                    owner_id,
                    start_date,
                    50000.0,
                )
            })
            .collect();
        stats.rfx_event_count = rfx_events.len();

        // Step 5: Bids
        let mut bid_gen = BidGenerator::new(seed + 4);
        let mut all_bids = Vec::new();
        for rfx in &rfx_events {
            let bidder_count = vendor_ids.len().clamp(2, 5);
            let responding: Vec<String> = vendor_ids.iter().take(bidder_count).cloned().collect();
            let bids = bid_gen.generate(rfx, &responding, start_date);
            all_bids.extend(bids);
        }
        stats.bid_count = all_bids.len();

        // Step 6: Bid Evaluations
        let mut eval_gen = BidEvaluationGenerator::new(seed + 5);
        let bid_evaluations: Vec<BidEvaluation> = rfx_events
            .iter()
            .map(|rfx| {
                let rfx_bids: Vec<SupplierBid> = all_bids
                    .iter()
                    .filter(|b| b.rfx_id == rfx.rfx_id)
                    .cloned()
                    .collect();
                eval_gen.evaluate(rfx, &rfx_bids, owner_id)
            })
            .collect();

        // Step 7: Contracts from winning bids
        let mut contract_gen = ContractGenerator::new(seed + 6);
        let contracts: Vec<ProcurementContract> = bid_evaluations
            .iter()
            .zip(rfx_events.iter())
            .filter_map(|(eval, rfx)| {
                eval.ranked_bids.first().and_then(|winner| {
                    all_bids
                        .iter()
                        .find(|b| b.bid_id == winner.bid_id)
                        .map(|winning_bid| {
                            contract_gen.generate_from_bid(
                                winning_bid,
                                Some(&rfx.sourcing_project_id),
                                &rfx.category_id,
                                owner_id,
                                start_date,
                            )
                        })
                })
            })
            .collect();
        stats.contract_count = contracts.len();

        // Step 8: Catalog Items
        let mut catalog_gen = CatalogGenerator::new(seed + 7);
        let catalog_items = catalog_gen.generate(&contracts);
        stats.catalog_item_count = catalog_items.len();

        // Step 9: Scorecards
        let mut scorecard_gen = ScorecardGenerator::new(seed + 8);
        let vendor_contracts: Vec<(String, Vec<&ProcurementContract>)> = contracts
            .iter()
            .fold(
                std::collections::HashMap::<String, Vec<&ProcurementContract>>::new(),
                |mut acc, c| {
                    acc.entry(c.vendor_id.clone()).or_default().push(c);
                    acc
                },
            )
            .into_iter()
            .collect();
        let scorecards = scorecard_gen.generate(
            company_code,
            &vendor_contracts,
            start_date,
            end_date,
            owner_id,
        );
        stats.scorecard_count = scorecards.len();

        info!(
            "S2C sourcing generated: {} projects, {} RFx, {} bids, {} contracts, {} catalog items, {} scorecards",
            stats.sourcing_project_count, stats.rfx_event_count, stats.bid_count,
            stats.contract_count, stats.catalog_item_count, stats.scorecard_count
        );
        self.check_resources_with_log("post-sourcing")?;

        Ok(SourcingSnapshot {
            spend_analyses,
            sourcing_projects,
            qualifications,
            rfx_events,
            bids: all_bids,
            bid_evaluations,
            contracts,
            catalog_items,
            scorecards,
        })
    }

    /// Phase 15: Generate bank reconciliations and financial statements.
    fn phase_financial_reporting(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<FinancialReportingSnapshot> {
        let fs_enabled = self.phase_config.generate_financial_statements
            || self.config.financial_reporting.enabled;
        let br_enabled = self.phase_config.generate_bank_reconciliation;

        if !fs_enabled && !br_enabled {
            debug!("Phase 15: Skipped (financial reporting disabled)");
            return Ok(FinancialReportingSnapshot::default());
        }

        info!("Phase 15: Generating Financial Reporting Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        let mut financial_statements = Vec::new();
        let mut bank_reconciliations = Vec::new();

        // Generate financial statements from document flow data
        if fs_enabled {
            let company_code = self
                .config
                .companies
                .first()
                .map(|c| c.code.as_str())
                .unwrap_or("1000");
            let currency = self
                .config
                .companies
                .first()
                .map(|c| c.currency.as_str())
                .unwrap_or("USD");
            let mut fs_gen = FinancialStatementGenerator::new(seed + 20);

            // Generate one set of statements per period
            for period in 0..self.config.global.period_months {
                let period_start = start_date + chrono::Months::new(period);
                let period_end =
                    start_date + chrono::Months::new(period + 1) - chrono::Days::new(1);
                let fiscal_year = period_end.year() as u16;
                let fiscal_period = period_end.month() as u8;

                // Build simplified trial balance entries from document flow aggregates
                let tb_entries = self.build_trial_balance_from_flows(document_flows, &period_end);

                let stmts = fs_gen.generate(
                    company_code,
                    currency,
                    &tb_entries,
                    period_start,
                    period_end,
                    fiscal_year,
                    fiscal_period,
                    None,
                    "SYS-AUTOCLOSE",
                );
                financial_statements.extend(stmts);
            }
            stats.financial_statement_count = financial_statements.len();
            info!(
                "Financial statements generated: {} statements",
                stats.financial_statement_count
            );
        }

        // Generate bank reconciliations from payment data
        if br_enabled && !document_flows.payments.is_empty() {
            let mut br_gen = BankReconciliationGenerator::new(seed + 25);

            // Group payments by company code and period
            for company in &self.config.companies {
                let company_payments: Vec<PaymentReference> = document_flows
                    .payments
                    .iter()
                    .filter(|p| p.header.company_code == company.code)
                    .map(|p| PaymentReference {
                        id: p.header.document_id.clone(),
                        amount: if p.is_vendor { p.amount } else { -p.amount },
                        date: p.header.document_date,
                        reference: p
                            .check_number
                            .clone()
                            .or_else(|| p.wire_reference.clone())
                            .unwrap_or_else(|| p.header.document_id.clone()),
                    })
                    .collect();

                if company_payments.is_empty() {
                    continue;
                }

                let bank_account_id = format!("{}-MAIN", company.code);

                // Generate one reconciliation per period
                for period in 0..self.config.global.period_months {
                    let period_start = start_date + chrono::Months::new(period);
                    let period_end =
                        start_date + chrono::Months::new(period + 1) - chrono::Days::new(1);

                    let period_payments: Vec<PaymentReference> = company_payments
                        .iter()
                        .filter(|p| p.date >= period_start && p.date <= period_end)
                        .cloned()
                        .collect();

                    let recon = br_gen.generate(
                        &company.code,
                        &bank_account_id,
                        period_start,
                        period_end,
                        &company.currency,
                        &period_payments,
                    );
                    bank_reconciliations.push(recon);
                }
            }
            info!(
                "Bank reconciliations generated: {} reconciliations",
                bank_reconciliations.len()
            );
        }

        stats.bank_reconciliation_count = bank_reconciliations.len();
        self.check_resources_with_log("post-financial-reporting")?;

        Ok(FinancialReportingSnapshot {
            financial_statements,
            bank_reconciliations,
        })
    }

    /// Build simplified trial balance entries from document flow data for financial statement generation.
    fn build_trial_balance_from_flows(
        &self,
        flows: &DocumentFlowSnapshot,
        _period_end: &NaiveDate,
    ) -> Vec<datasynth_generators::TrialBalanceEntry> {
        use rust_decimal::Decimal;

        let mut entries = Vec::new();

        // Aggregate AR from customer invoices
        let ar_total: Decimal = flows
            .customer_invoices
            .iter()
            .map(|ci| ci.total_gross_amount)
            .sum();
        if !ar_total.is_zero() {
            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: "1100".to_string(),
                account_name: "Accounts Receivable".to_string(),
                category: "Receivables".to_string(),
                debit_balance: ar_total,
                credit_balance: Decimal::ZERO,
            });
        }

        // Aggregate AP from vendor invoices
        let ap_total: Decimal = flows
            .vendor_invoices
            .iter()
            .map(|vi| vi.payable_amount)
            .sum();
        if !ap_total.is_zero() {
            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: "2000".to_string(),
                account_name: "Accounts Payable".to_string(),
                category: "Payables".to_string(),
                debit_balance: Decimal::ZERO,
                credit_balance: ap_total,
            });
        }

        // Revenue from sales
        let revenue: Decimal = flows
            .customer_invoices
            .iter()
            .map(|ci| ci.total_gross_amount)
            .sum();
        if !revenue.is_zero() {
            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: "4000".to_string(),
                account_name: "Revenue".to_string(),
                category: "Revenue".to_string(),
                debit_balance: Decimal::ZERO,
                credit_balance: revenue,
            });
        }

        // COGS from purchase orders
        let cogs: Decimal = flows
            .purchase_orders
            .iter()
            .map(|po| po.total_net_amount)
            .sum();
        if !cogs.is_zero() {
            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: "5000".to_string(),
                account_name: "Cost of Goods Sold".to_string(),
                category: "CostOfSales".to_string(),
                debit_balance: cogs,
                credit_balance: Decimal::ZERO,
            });
        }

        // Cash from payments
        let payments_out: Decimal = flows.payments.iter().map(|p| p.amount).sum();
        if !payments_out.is_zero() {
            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: "1000".to_string(),
                account_name: "Cash".to_string(),
                category: "Cash".to_string(),
                debit_balance: payments_out,
                credit_balance: Decimal::ZERO,
            });
        }

        entries
    }

    /// Phase 16: Generate HR data (payroll runs, time entries, expense reports).
    fn phase_hr_data(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<HrSnapshot> {
        if !self.config.hr.enabled {
            debug!("Phase 16: Skipped (HR generation disabled)");
            return Ok(HrSnapshot::default());
        }

        info!("Phase 16: Generating HR Data (Payroll, Time Entries, Expenses)");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");
        let currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.as_str())
            .unwrap_or("USD");

        let employee_ids: Vec<String> = self
            .master_data
            .employees
            .iter()
            .map(|e| e.employee_id.clone())
            .collect();

        if employee_ids.is_empty() {
            debug!("Phase 16: Skipped (no employees available)");
            return Ok(HrSnapshot::default());
        }

        let mut snapshot = HrSnapshot::default();

        // Generate payroll runs (one per month)
        if self.config.hr.payroll.enabled {
            let mut payroll_gen = datasynth_generators::PayrollGenerator::new(seed + 30);
            let employees_with_salary: Vec<(
                String,
                rust_decimal::Decimal,
                Option<String>,
                Option<String>,
            )> = self
                .master_data
                .employees
                .iter()
                .map(|e| {
                    (
                        e.employee_id.clone(),
                        rust_decimal::Decimal::from(5000), // Default monthly salary
                        e.cost_center.clone(),
                        e.department_id.clone(),
                    )
                })
                .collect();

            for month in 0..self.config.global.period_months {
                let period_start = start_date + chrono::Months::new(month);
                let period_end = start_date + chrono::Months::new(month + 1) - chrono::Days::new(1);
                let (run, items) = payroll_gen.generate(
                    company_code,
                    &employees_with_salary,
                    period_start,
                    period_end,
                    currency,
                );
                snapshot.payroll_runs.push(run);
                snapshot.payroll_run_count += 1;
                snapshot.payroll_line_item_count += items.len();
                snapshot.payroll_line_items.extend(items);
            }
        }

        // Generate time entries
        if self.config.hr.time_attendance.enabled {
            let mut time_gen = datasynth_generators::TimeEntryGenerator::new(seed + 31);
            let entries = time_gen.generate(
                &employee_ids,
                start_date,
                end_date,
                &self.config.hr.time_attendance,
            );
            snapshot.time_entry_count = entries.len();
            snapshot.time_entries = entries;
        }

        // Generate expense reports
        if self.config.hr.expenses.enabled {
            let mut expense_gen = datasynth_generators::ExpenseReportGenerator::new(seed + 32);
            let reports = expense_gen.generate(
                &employee_ids,
                start_date,
                end_date,
                &self.config.hr.expenses,
            );
            snapshot.expense_report_count = reports.len();
            snapshot.expense_reports = reports;
        }

        stats.payroll_run_count = snapshot.payroll_run_count;
        stats.time_entry_count = snapshot.time_entry_count;
        stats.expense_report_count = snapshot.expense_report_count;

        info!(
            "HR data generated: {} payroll runs ({} line items), {} time entries, {} expense reports",
            snapshot.payroll_run_count, snapshot.payroll_line_item_count,
            snapshot.time_entry_count, snapshot.expense_report_count
        );
        self.check_resources_with_log("post-hr")?;

        Ok(snapshot)
    }

    /// Phase 17: Generate accounting standards data (revenue recognition, impairment).
    fn phase_accounting_standards(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<AccountingStandardsSnapshot> {
        if !self.phase_config.generate_accounting_standards
            || !self.config.accounting_standards.enabled
        {
            debug!("Phase 17: Skipped (accounting standards generation disabled)");
            return Ok(AccountingStandardsSnapshot::default());
        }
        info!("Phase 17: Generating Accounting Standards Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");
        let currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.as_str())
            .unwrap_or("USD");

        // Convert config framework to standards framework
        let framework = match self.config.accounting_standards.framework {
            datasynth_config::schema::AccountingFrameworkConfig::UsGaap => {
                datasynth_standards::framework::AccountingFramework::UsGaap
            }
            datasynth_config::schema::AccountingFrameworkConfig::Ifrs => {
                datasynth_standards::framework::AccountingFramework::Ifrs
            }
            datasynth_config::schema::AccountingFrameworkConfig::DualReporting => {
                datasynth_standards::framework::AccountingFramework::DualReporting
            }
        };

        let mut snapshot = AccountingStandardsSnapshot::default();

        // Revenue recognition
        if self.config.accounting_standards.revenue_recognition.enabled {
            let customer_ids: Vec<String> = self
                .master_data
                .customers
                .iter()
                .map(|c| c.customer_id.clone())
                .collect();

            if !customer_ids.is_empty() {
                let mut rev_gen = datasynth_generators::RevenueRecognitionGenerator::new(seed + 40);
                let contracts = rev_gen.generate(
                    company_code,
                    &customer_ids,
                    start_date,
                    end_date,
                    currency,
                    &self.config.accounting_standards.revenue_recognition,
                    framework,
                );
                snapshot.revenue_contract_count = contracts.len();
            }
        }

        // Impairment testing
        if self.config.accounting_standards.impairment.enabled {
            let asset_data: Vec<(String, String, rust_decimal::Decimal)> = self
                .master_data
                .assets
                .iter()
                .map(|a| {
                    (
                        a.asset_id.clone(),
                        a.description.clone(),
                        a.acquisition_cost,
                    )
                })
                .collect();

            if !asset_data.is_empty() {
                let mut imp_gen = datasynth_generators::ImpairmentGenerator::new(seed + 41);
                let tests = imp_gen.generate(
                    company_code,
                    &asset_data,
                    end_date,
                    &self.config.accounting_standards.impairment,
                    framework,
                );
                snapshot.impairment_test_count = tests.len();
            }
        }

        stats.revenue_contract_count = snapshot.revenue_contract_count;
        stats.impairment_test_count = snapshot.impairment_test_count;

        info!(
            "Accounting standards data generated: {} revenue contracts, {} impairment tests",
            snapshot.revenue_contract_count, snapshot.impairment_test_count
        );
        self.check_resources_with_log("post-accounting-standards")?;

        Ok(snapshot)
    }

    /// Phase 18: Generate manufacturing data (production orders, quality inspections, cycle counts).
    fn phase_manufacturing(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<ManufacturingSnapshot> {
        if !self.phase_config.generate_manufacturing || !self.config.manufacturing.enabled {
            debug!("Phase 18: Skipped (manufacturing generation disabled)");
            return Ok(ManufacturingSnapshot::default());
        }
        info!("Phase 18: Generating Manufacturing Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        let material_data: Vec<(String, String)> = self
            .master_data
            .materials
            .iter()
            .map(|m| (m.material_id.clone(), m.description.clone()))
            .collect();

        if material_data.is_empty() {
            debug!("Phase 18: Skipped (no materials available)");
            return Ok(ManufacturingSnapshot::default());
        }

        let mut snapshot = ManufacturingSnapshot::default();

        // Generate production orders
        let mut prod_gen = datasynth_generators::ProductionOrderGenerator::new(seed + 50);
        let production_orders = prod_gen.generate(
            company_code,
            &material_data,
            start_date,
            end_date,
            &self.config.manufacturing.production_orders,
            &self.config.manufacturing.costing,
            &self.config.manufacturing.routing,
        );
        snapshot.production_order_count = production_orders.len();

        // Generate quality inspections from production orders
        let inspection_data: Vec<(String, String, String)> = production_orders
            .iter()
            .map(|po| {
                (
                    po.order_id.clone(),
                    po.material_id.clone(),
                    po.material_description.clone(),
                )
            })
            .collect();

        snapshot.production_orders = production_orders;

        if !inspection_data.is_empty() {
            let mut qi_gen = datasynth_generators::QualityInspectionGenerator::new(seed + 51);
            let inspections = qi_gen.generate(company_code, &inspection_data, end_date);
            snapshot.quality_inspection_count = inspections.len();
            snapshot.quality_inspections = inspections;
        }

        // Generate cycle counts (one per month)
        let storage_locations: Vec<(String, String)> = material_data
            .iter()
            .enumerate()
            .map(|(i, (mid, _))| (mid.clone(), format!("SL-{:03}", (i % 10) + 1)))
            .collect();

        let mut cc_gen = datasynth_generators::CycleCountGenerator::new(seed + 52);
        let mut cycle_count_total = 0usize;
        for month in 0..self.config.global.period_months {
            let count_date = start_date + chrono::Months::new(month);
            let items_per_count = storage_locations.len().clamp(10, 50);
            let cc = cc_gen.generate(
                company_code,
                &storage_locations,
                count_date,
                items_per_count,
            );
            snapshot.cycle_counts.push(cc);
            cycle_count_total += 1;
        }
        snapshot.cycle_count_count = cycle_count_total;

        stats.production_order_count = snapshot.production_order_count;
        stats.quality_inspection_count = snapshot.quality_inspection_count;
        stats.cycle_count_count = snapshot.cycle_count_count;

        info!(
            "Manufacturing data generated: {} production orders, {} quality inspections, {} cycle counts",
            snapshot.production_order_count, snapshot.quality_inspection_count, snapshot.cycle_count_count
        );
        self.check_resources_with_log("post-manufacturing")?;

        Ok(snapshot)
    }

    /// Phase 19: Generate sales quotes, management KPIs, and budgets.
    fn phase_sales_kpi_budgets(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<SalesKpiBudgetsSnapshot> {
        if !self.phase_config.generate_sales_kpi_budgets {
            debug!("Phase 19: Skipped (sales/KPI/budget generation disabled)");
            return Ok(SalesKpiBudgetsSnapshot::default());
        }
        info!("Phase 19: Generating Sales Quotes, KPIs, and Budgets");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        let mut snapshot = SalesKpiBudgetsSnapshot::default();

        // Sales Quotes
        if self.config.sales_quotes.enabled {
            let customer_data: Vec<(String, String)> = self
                .master_data
                .customers
                .iter()
                .map(|c| (c.customer_id.clone(), c.name.clone()))
                .collect();
            let material_data: Vec<(String, String)> = self
                .master_data
                .materials
                .iter()
                .map(|m| (m.material_id.clone(), m.description.clone()))
                .collect();

            if !customer_data.is_empty() && !material_data.is_empty() {
                let mut quote_gen = datasynth_generators::SalesQuoteGenerator::new(seed + 60);
                let quotes = quote_gen.generate(
                    company_code,
                    &customer_data,
                    &material_data,
                    start_date,
                    end_date,
                    &self.config.sales_quotes,
                );
                snapshot.sales_quote_count = quotes.len();
                snapshot.sales_quotes = quotes;
            }
        }

        // Management KPIs
        if self.config.financial_reporting.management_kpis.enabled {
            let mut kpi_gen = datasynth_generators::KpiGenerator::new(seed + 61);
            let kpis = kpi_gen.generate(
                company_code,
                start_date,
                end_date,
                &self.config.financial_reporting.management_kpis,
            );
            snapshot.kpi_count = kpis.len();
            snapshot.kpis = kpis;
        }

        // Budgets
        if self.config.financial_reporting.budgets.enabled {
            let account_data: Vec<(String, String)> = coa
                .accounts
                .iter()
                .map(|a| (a.account_number.clone(), a.short_description.clone()))
                .collect();

            if !account_data.is_empty() {
                let fiscal_year = start_date.year() as u32;
                let mut budget_gen = datasynth_generators::BudgetGenerator::new(seed + 62);
                let budget = budget_gen.generate(
                    company_code,
                    fiscal_year,
                    &account_data,
                    &self.config.financial_reporting.budgets,
                );
                snapshot.budget_line_count = budget.line_items.len();
                snapshot.budgets.push(budget);
            }
        }

        stats.sales_quote_count = snapshot.sales_quote_count;
        stats.kpi_count = snapshot.kpi_count;
        stats.budget_line_count = snapshot.budget_line_count;

        info!(
            "Sales/KPI/Budget data generated: {} quotes, {} KPIs, {} budget lines",
            snapshot.sales_quote_count, snapshot.kpi_count, snapshot.budget_line_count
        );
        self.check_resources_with_log("post-sales-kpi-budgets")?;

        Ok(snapshot)
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
    #[allow(clippy::too_many_arguments)]
    fn generate_ocpm_events(
        &mut self,
        flows: &DocumentFlowSnapshot,
        sourcing: &SourcingSnapshot,
        hr: &HrSnapshot,
        manufacturing: &ManufacturingSnapshot,
        banking: &BankingSnapshot,
        audit: &AuditSnapshot,
        financial_reporting: &FinancialReportingSnapshot,
    ) -> SynthResult<OcpmSnapshot> {
        let total_chains = flows.p2p_chains.len()
            + flows.o2c_chains.len()
            + sourcing.sourcing_projects.len()
            + hr.payroll_runs.len()
            + manufacturing.production_orders.len()
            + banking.customers.len()
            + audit.engagements.len()
            + financial_reporting.bank_reconciliations.len();
        let pb = self.create_progress_bar(total_chains as u64, "Generating OCPM Events");

        // Create OCPM event log with standard types
        let metadata = EventLogMetadata::new("SyntheticData OCPM Log");
        let mut event_log = OcpmEventLog::with_metadata(metadata).with_standard_types();

        // Configure the OCPM generator
        let ocpm_config = OcpmGeneratorConfig {
            generate_p2p: true,
            generate_o2c: true,
            generate_s2c: !sourcing.sourcing_projects.is_empty(),
            generate_h2r: !hr.payroll_runs.is_empty(),
            generate_mfg: !manufacturing.production_orders.is_empty(),
            generate_bank_recon: !financial_reporting.bank_reconciliations.is_empty(),
            generate_bank: !banking.customers.is_empty(),
            generate_audit: !audit.engagements.is_empty(),
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

        // Helper closure to add case results to event log
        let add_result = |event_log: &mut OcpmEventLog,
                          result: datasynth_ocpm::CaseGenerationResult| {
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
        };

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
            add_result(&mut event_log, result);

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
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from S2C sourcing projects
        for project in &sourcing.sourcing_projects {
            // Find vendor from contracts or qualifications
            let vendor_id = sourcing
                .contracts
                .iter()
                .find(|c| c.sourcing_project_id.as_deref() == Some(&project.project_id))
                .map(|c| c.vendor_id.clone())
                .or_else(|| sourcing.qualifications.first().map(|q| q.vendor_id.clone()))
                .unwrap_or_else(|| "V000".to_string());
            let mut docs = S2cDocuments::new(
                &project.project_id,
                &vendor_id,
                &project.company_code,
                project.estimated_annual_spend,
            );
            // Link RFx if available
            if let Some(rfx) = sourcing
                .rfx_events
                .iter()
                .find(|r| r.sourcing_project_id == project.project_id)
            {
                docs = docs.with_rfx(&rfx.rfx_id);
                // Link winning bid (status == Accepted)
                if let Some(bid) = sourcing.bids.iter().find(|b| {
                    b.rfx_id == rfx.rfx_id
                        && b.status == datasynth_core::models::sourcing::BidStatus::Accepted
                }) {
                    docs = docs.with_winning_bid(&bid.bid_id);
                }
            }
            // Link contract
            if let Some(contract) = sourcing
                .contracts
                .iter()
                .find(|c| c.sourcing_project_id.as_deref() == Some(&project.project_id))
            {
                docs = docs.with_contract(&contract.contract_id);
            }
            let start_time = chrono::Utc::now() - chrono::Duration::days(90);
            let result = ocpm_gen.generate_s2c_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from H2R payroll runs
        for run in &hr.payroll_runs {
            // Use first matching payroll line item's employee, or fallback
            let employee_id = hr
                .payroll_line_items
                .iter()
                .find(|li| li.payroll_id == run.payroll_id)
                .map(|li| li.employee_id.as_str())
                .unwrap_or("EMP000");
            let docs = H2rDocuments::new(
                &run.payroll_id,
                employee_id,
                &run.company_code,
                run.total_gross,
            )
            .with_time_entries(
                hr.time_entries
                    .iter()
                    .filter(|t| t.date >= run.pay_period_start && t.date <= run.pay_period_end)
                    .take(5)
                    .map(|t| t.entry_id.as_str())
                    .collect(),
            );
            let start_time = chrono::Utc::now() - chrono::Duration::days(30);
            let result = ocpm_gen.generate_h2r_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from MFG production orders
        for order in &manufacturing.production_orders {
            let mut docs = MfgDocuments::new(
                &order.order_id,
                &order.material_id,
                &order.company_code,
                order.planned_quantity,
            )
            .with_operations(
                order
                    .operations
                    .iter()
                    .map(|o| format!("OP-{:04}", o.operation_number))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|s| s.as_str())
                    .collect(),
            );
            // Link quality inspection if available (via reference_id matching order_id)
            if let Some(insp) = manufacturing
                .quality_inspections
                .iter()
                .find(|i| i.reference_id == order.order_id)
            {
                docs = docs.with_inspection(&insp.inspection_id);
            }
            // Link cycle count if available (via items matching the material)
            if let Some(cc) = manufacturing.cycle_counts.first() {
                docs = docs.with_cycle_count(&cc.count_id);
            }
            let start_time = chrono::Utc::now() - chrono::Duration::days(60);
            let result = ocpm_gen.generate_mfg_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from Banking customers
        for customer in &banking.customers {
            let customer_id_str = customer.customer_id.to_string();
            let mut docs = BankDocuments::new(&customer_id_str, "1000");
            // Link accounts (primary_owner_id matches customer_id)
            if let Some(account) = banking
                .accounts
                .iter()
                .find(|a| a.primary_owner_id == customer.customer_id)
            {
                let account_id_str = account.account_id.to_string();
                docs = docs.with_account(&account_id_str);
                // Link transactions for this account
                let txn_strs: Vec<String> = banking
                    .transactions
                    .iter()
                    .filter(|t| t.account_id == account.account_id)
                    .take(10)
                    .map(|t| t.transaction_id.to_string())
                    .collect();
                let txn_ids: Vec<&str> = txn_strs.iter().map(|s| s.as_str()).collect();
                let txn_amounts: Vec<rust_decimal::Decimal> = banking
                    .transactions
                    .iter()
                    .filter(|t| t.account_id == account.account_id)
                    .take(10)
                    .map(|t| t.amount)
                    .collect();
                if !txn_ids.is_empty() {
                    docs = docs.with_transactions(txn_ids, txn_amounts);
                }
            }
            let start_time = chrono::Utc::now() - chrono::Duration::days(180);
            let result = ocpm_gen.generate_bank_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from Audit engagements
        for engagement in &audit.engagements {
            let engagement_id_str = engagement.engagement_id.to_string();
            let docs = AuditDocuments::new(&engagement_id_str, &engagement.client_entity_id)
                .with_workpapers(
                    audit
                        .workpapers
                        .iter()
                        .filter(|w| w.engagement_id == engagement.engagement_id)
                        .take(10)
                        .map(|w| w.workpaper_id.to_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                )
                .with_evidence(
                    audit
                        .evidence
                        .iter()
                        .filter(|e| e.engagement_id == engagement.engagement_id)
                        .take(10)
                        .map(|e| e.evidence_id.to_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                )
                .with_risks(
                    audit
                        .risk_assessments
                        .iter()
                        .filter(|r| r.engagement_id == engagement.engagement_id)
                        .take(5)
                        .map(|r| r.risk_id.to_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                )
                .with_findings(
                    audit
                        .findings
                        .iter()
                        .filter(|f| f.engagement_id == engagement.engagement_id)
                        .take(5)
                        .map(|f| f.finding_id.to_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                )
                .with_judgments(
                    audit
                        .judgments
                        .iter()
                        .filter(|j| j.engagement_id == engagement.engagement_id)
                        .take(5)
                        .map(|j| j.judgment_id.to_string())
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                );
            let start_time = chrono::Utc::now() - chrono::Duration::days(120);
            let result = ocpm_gen.generate_audit_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from Bank Reconciliations
        for recon in &financial_reporting.bank_reconciliations {
            let docs = BankReconDocuments::new(
                &recon.reconciliation_id,
                &recon.bank_account_id,
                &recon.company_code,
                recon.bank_ending_balance,
            )
            .with_statement_lines(
                recon
                    .statement_lines
                    .iter()
                    .take(20)
                    .map(|l| l.line_id.as_str())
                    .collect(),
            )
            .with_reconciling_items(
                recon
                    .reconciling_items
                    .iter()
                    .take(10)
                    .map(|i| i.item_id.as_str())
                    .collect(),
            );
            let start_time = chrono::Utc::now() - chrono::Duration::days(30);
            let result = ocpm_gen.generate_bank_recon_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

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
                            common: datasynth_graph::CommonExportConfig {
                                export_node_features: true,
                                export_edge_features: true,
                                export_node_labels: true,
                                export_edge_labels: true,
                                export_masks: true,
                                train_ratio: self.config.graph_export.train_ratio,
                                val_ratio: self.config.graph_export.validation_ratio,
                                seed: self.config.graph_export.split_seed.unwrap_or(self.seed),
                            },
                            one_hot_categoricals: false,
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
                    datasynth_config::schema::GraphExportFormat::RustGraphHypergraph => {
                        // Hypergraph export is handled separately in Phase 10b
                        debug!("RustGraphHypergraph format is handled in Phase 10b (hypergraph export)");
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

    /// Export a multi-layer hypergraph for RustGraph integration.
    ///
    /// Builds a 3-layer hypergraph:
    /// - Layer 1: Governance & Controls (COSO, internal controls, master data)
    /// - Layer 2: Process Events (all process family document flows + OCPM events)
    /// - Layer 3: Accounting Network (GL accounts, journal entries as hyperedges)
    #[allow(clippy::too_many_arguments)]
    fn export_hypergraph(
        &self,
        coa: &Arc<ChartOfAccounts>,
        entries: &[JournalEntry],
        document_flows: &DocumentFlowSnapshot,
        sourcing: &SourcingSnapshot,
        hr: &HrSnapshot,
        manufacturing: &ManufacturingSnapshot,
        banking: &BankingSnapshot,
        audit: &AuditSnapshot,
        financial_reporting: &FinancialReportingSnapshot,
        ocpm: &OcpmSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<HypergraphExportInfo> {
        use datasynth_graph::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
        use datasynth_graph::exporters::hypergraph::{HypergraphExportConfig, HypergraphExporter};
        use datasynth_graph::exporters::unified::{RustGraphUnifiedExporter, UnifiedExportConfig};
        use datasynth_graph::models::hypergraph::AggregationStrategy;

        let hg_settings = &self.config.graph_export.hypergraph;

        // Parse aggregation strategy from config string
        let aggregation_strategy = match hg_settings.aggregation_strategy.as_str() {
            "truncate" => AggregationStrategy::Truncate,
            "pool_by_counterparty" => AggregationStrategy::PoolByCounterparty,
            "pool_by_time_period" => AggregationStrategy::PoolByTimePeriod,
            "importance_sample" => AggregationStrategy::ImportanceSample,
            _ => AggregationStrategy::PoolByCounterparty,
        };

        let builder_config = HypergraphConfig {
            max_nodes: hg_settings.max_nodes,
            aggregation_strategy,
            include_coso: hg_settings.governance_layer.include_coso,
            include_controls: hg_settings.governance_layer.include_controls,
            include_sox: hg_settings.governance_layer.include_sox,
            include_vendors: hg_settings.governance_layer.include_vendors,
            include_customers: hg_settings.governance_layer.include_customers,
            include_employees: hg_settings.governance_layer.include_employees,
            include_p2p: hg_settings.process_layer.include_p2p,
            include_o2c: hg_settings.process_layer.include_o2c,
            include_s2c: hg_settings.process_layer.include_s2c,
            include_h2r: hg_settings.process_layer.include_h2r,
            include_mfg: hg_settings.process_layer.include_mfg,
            include_bank: hg_settings.process_layer.include_bank,
            include_audit: hg_settings.process_layer.include_audit,
            include_r2r: hg_settings.process_layer.include_r2r,
            events_as_hyperedges: hg_settings.process_layer.events_as_hyperedges,
            docs_per_counterparty_threshold: hg_settings
                .process_layer
                .docs_per_counterparty_threshold,
            include_accounts: hg_settings.accounting_layer.include_accounts,
            je_as_hyperedges: hg_settings.accounting_layer.je_as_hyperedges,
            include_cross_layer_edges: hg_settings.cross_layer.enabled,
        };

        let mut builder = HypergraphBuilder::new(builder_config);

        // Layer 1: Governance & Controls
        builder.add_coso_framework();

        // Add controls if available (generated during JE generation)
        // Controls are generated per-company; we use the standard set
        if hg_settings.governance_layer.include_controls && self.config.internal_controls.enabled {
            let controls = InternalControl::standard_controls();
            builder.add_controls(&controls);
        }

        // Add master data
        builder.add_vendors(&self.master_data.vendors);
        builder.add_customers(&self.master_data.customers);
        builder.add_employees(&self.master_data.employees);

        // Layer 2: Process Events (all process families)
        builder.add_p2p_documents(
            &document_flows.purchase_orders,
            &document_flows.goods_receipts,
            &document_flows.vendor_invoices,
            &document_flows.payments,
        );
        builder.add_o2c_documents(
            &document_flows.sales_orders,
            &document_flows.deliveries,
            &document_flows.customer_invoices,
        );
        builder.add_s2c_documents(
            &sourcing.sourcing_projects,
            &sourcing.qualifications,
            &sourcing.rfx_events,
            &sourcing.bids,
            &sourcing.bid_evaluations,
            &sourcing.contracts,
        );
        builder.add_h2r_documents(&hr.payroll_runs, &hr.time_entries, &hr.expense_reports);
        builder.add_mfg_documents(
            &manufacturing.production_orders,
            &manufacturing.quality_inspections,
            &manufacturing.cycle_counts,
        );
        builder.add_bank_documents(&banking.customers, &banking.accounts, &banking.transactions);
        builder.add_audit_documents(
            &audit.engagements,
            &audit.workpapers,
            &audit.findings,
            &audit.evidence,
            &audit.risk_assessments,
            &audit.judgments,
        );
        builder.add_bank_recon_documents(&financial_reporting.bank_reconciliations);

        // OCPM events as hyperedges
        if let Some(ref event_log) = ocpm.event_log {
            builder.add_ocpm_events(event_log);
        }

        // Layer 3: Accounting Network
        builder.add_accounts(coa);
        builder.add_journal_entries_as_hyperedges(entries);

        // Build the hypergraph
        let hypergraph = builder.build();

        // Export
        let output_dir = self
            .output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.config.output.output_directory));
        let hg_dir = output_dir
            .join(&self.config.graph_export.output_subdirectory)
            .join(&hg_settings.output_subdirectory);

        // Branch on output format
        let (num_nodes, num_edges, num_hyperedges) = match hg_settings.output_format.as_str() {
            "unified" => {
                let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
                let metadata = exporter.export(&hypergraph, &hg_dir).map_err(|e| {
                    SynthError::generation(format!("Unified hypergraph export failed: {}", e))
                })?;
                (
                    metadata.num_nodes,
                    metadata.num_edges,
                    metadata.num_hyperedges,
                )
            }
            _ => {
                // "native" or any unrecognized format → use existing exporter
                let exporter = HypergraphExporter::new(HypergraphExportConfig::default());
                let metadata = exporter.export(&hypergraph, &hg_dir).map_err(|e| {
                    SynthError::generation(format!("Hypergraph export failed: {}", e))
                })?;
                (
                    metadata.num_nodes,
                    metadata.num_edges,
                    metadata.num_hyperedges,
                )
            }
        };

        // Stream to RustGraph ingest endpoint if configured
        #[cfg(feature = "streaming")]
        if let Some(ref target_url) = hg_settings.stream_target {
            use crate::stream_client::{StreamClient, StreamConfig};
            use std::io::Write as _;

            let api_key = std::env::var("RUSTGRAPH_API_KEY").ok();
            let stream_config = StreamConfig {
                target_url: target_url.clone(),
                batch_size: hg_settings.stream_batch_size,
                api_key,
                ..StreamConfig::default()
            };

            match StreamClient::new(stream_config) {
                Ok(mut client) => {
                    let exporter = RustGraphUnifiedExporter::new(UnifiedExportConfig::default());
                    match exporter.export_to_writer(&hypergraph, &mut client) {
                        Ok(_) => {
                            if let Err(e) = client.flush() {
                                warn!("Failed to flush stream client: {}", e);
                            } else {
                                info!("Streamed {} records to {}", client.total_sent(), target_url);
                            }
                        }
                        Err(e) => {
                            warn!("Streaming export failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create stream client: {}", e);
                }
            }
        }

        // Update stats
        stats.graph_node_count += num_nodes;
        stats.graph_edge_count += num_edges;
        stats.graph_export_count += 1;

        Ok(HypergraphExportInfo {
            node_count: num_nodes,
            edge_count: num_edges,
            hyperedge_count: num_hyperedges,
            output_path: hg_dir,
        })
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

    /// Build a lineage graph describing config → phase → output relationships.
    fn build_lineage_graph(&self) -> super::lineage::LineageGraph {
        use super::lineage::LineageGraphBuilder;

        let mut builder = LineageGraphBuilder::new();

        // Config sections
        builder.add_config_section("config:global", "Global Config");
        builder.add_config_section("config:chart_of_accounts", "Chart of Accounts Config");
        builder.add_config_section("config:transactions", "Transaction Config");

        // Generator phases
        builder.add_generator_phase("phase:coa", "Chart of Accounts Generation");
        builder.add_generator_phase("phase:je", "Journal Entry Generation");

        // Config → phase edges
        builder.configured_by("phase:coa", "config:chart_of_accounts");
        builder.configured_by("phase:je", "config:transactions");

        // Output files
        builder.add_output_file("output:je", "Journal Entries", "sample_entries.json");
        builder.produced_by("output:je", "phase:je");

        // Optional phases based on config
        if self.phase_config.generate_master_data {
            builder.add_config_section("config:master_data", "Master Data Config");
            builder.add_generator_phase("phase:master_data", "Master Data Generation");
            builder.configured_by("phase:master_data", "config:master_data");
            builder.input_to("phase:master_data", "phase:je");
        }

        if self.phase_config.generate_document_flows {
            builder.add_config_section("config:document_flows", "Document Flow Config");
            builder.add_generator_phase("phase:p2p", "P2P Document Flow");
            builder.add_generator_phase("phase:o2c", "O2C Document Flow");
            builder.configured_by("phase:p2p", "config:document_flows");
            builder.configured_by("phase:o2c", "config:document_flows");

            builder.add_output_file("output:po", "Purchase Orders", "purchase_orders.csv");
            builder.add_output_file("output:gr", "Goods Receipts", "goods_receipts.csv");
            builder.add_output_file("output:vi", "Vendor Invoices", "vendor_invoices.csv");
            builder.add_output_file("output:so", "Sales Orders", "sales_orders.csv");
            builder.add_output_file("output:ci", "Customer Invoices", "customer_invoices.csv");

            builder.produced_by("output:po", "phase:p2p");
            builder.produced_by("output:gr", "phase:p2p");
            builder.produced_by("output:vi", "phase:p2p");
            builder.produced_by("output:so", "phase:o2c");
            builder.produced_by("output:ci", "phase:o2c");
        }

        if self.phase_config.inject_anomalies {
            builder.add_config_section("config:fraud", "Fraud/Anomaly Config");
            builder.add_generator_phase("phase:anomaly", "Anomaly Injection");
            builder.configured_by("phase:anomaly", "config:fraud");
            builder.add_output_file(
                "output:labels",
                "Anomaly Labels",
                "labels/anomaly_labels.csv",
            );
            builder.produced_by("output:labels", "phase:anomaly");
        }

        if self.phase_config.generate_audit {
            builder.add_config_section("config:audit", "Audit Config");
            builder.add_generator_phase("phase:audit", "Audit Data Generation");
            builder.configured_by("phase:audit", "config:audit");
        }

        if self.phase_config.generate_banking {
            builder.add_config_section("config:banking", "Banking Config");
            builder.add_generator_phase("phase:banking", "Banking KYC/AML Generation");
            builder.configured_by("phase:banking", "config:banking");
        }

        if self.config.llm.enabled {
            builder.add_config_section("config:llm", "LLM Enrichment Config");
            builder.add_generator_phase("phase:llm_enrichment", "LLM Enrichment");
            builder.configured_by("phase:llm_enrichment", "config:llm");
        }

        if self.config.diffusion.enabled {
            builder.add_config_section("config:diffusion", "Diffusion Enhancement Config");
            builder.add_generator_phase("phase:diffusion", "Diffusion Enhancement");
            builder.configured_by("phase:diffusion", "config:diffusion");
        }

        if self.config.causal.enabled {
            builder.add_config_section("config:causal", "Causal Generation Config");
            builder.add_generator_phase("phase:causal", "Causal Overlay");
            builder.configured_by("phase:causal", "config:causal");
        }

        builder.build()
    }
}

/// Get the directory name for a graph export format.
fn format_name(format: datasynth_config::schema::GraphExportFormat) -> &'static str {
    match format {
        datasynth_config::schema::GraphExportFormat::PytorchGeometric => "pytorch_geometric",
        datasynth_config::schema::GraphExportFormat::Neo4j => "neo4j",
        datasynth_config::schema::GraphExportFormat::Dgl => "dgl",
        datasynth_config::schema::GraphExportFormat::RustGraph => "rustgraph",
        datasynth_config::schema::GraphExportFormat::RustGraphHypergraph => "rustgraph_hypergraph",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
            industry_specific: Default::default(),
            fingerprint_privacy: Default::default(),
            quality_gates: Default::default(),
            compliance: Default::default(),
            webhooks: Default::default(),
            llm: Default::default(),
            diffusion: Default::default(),
            causal: Default::default(),
            source_to_pay: Default::default(),
            financial_reporting: Default::default(),
            hr: Default::default(),
            manufacturing: Default::default(),
            sales_quotes: Default::default(),
            tax: Default::default(),
            treasury: Default::default(),
            project_accounting: Default::default(),
            esg: Default::default(),
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

    #[test]
    fn test_new_phases_disabled_by_default() {
        let config = create_test_config();
        // Verify new config fields default to disabled
        assert!(!config.llm.enabled);
        assert!(!config.diffusion.enabled);
        assert!(!config.causal.enabled);

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

        // All new phase statistics should be zero when disabled
        assert_eq!(result.statistics.llm_enrichment_ms, 0);
        assert_eq!(result.statistics.llm_vendors_enriched, 0);
        assert_eq!(result.statistics.diffusion_enhancement_ms, 0);
        assert_eq!(result.statistics.diffusion_samples_generated, 0);
        assert_eq!(result.statistics.causal_generation_ms, 0);
        assert_eq!(result.statistics.causal_samples_generated, 0);
        assert!(result.statistics.causal_validation_passed.is_none());
    }

    #[test]
    fn test_llm_enrichment_enabled() {
        let mut config = create_test_config();
        config.llm.enabled = true;
        config.llm.max_vendor_enrichments = 3;

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 3,
            materials_per_company: 3,
            assets_per_company: 3,
            employees_per_company: 3,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // LLM enrichment should have run
        assert!(result.statistics.llm_vendors_enriched > 0);
        assert!(result.statistics.llm_vendors_enriched <= 3);
    }

    #[test]
    fn test_diffusion_enhancement_enabled() {
        let mut config = create_test_config();
        config.diffusion.enabled = true;
        config.diffusion.n_steps = 50;
        config.diffusion.sample_size = 20;

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

        // Diffusion phase should have generated samples
        assert_eq!(result.statistics.diffusion_samples_generated, 20);
    }

    #[test]
    fn test_causal_overlay_enabled() {
        let mut config = create_test_config();
        config.causal.enabled = true;
        config.causal.template = "fraud_detection".to_string();
        config.causal.sample_size = 100;
        config.causal.validate = true;

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

        // Causal phase should have generated samples
        assert_eq!(result.statistics.causal_samples_generated, 100);
        // Validation should have run
        assert!(result.statistics.causal_validation_passed.is_some());
    }

    #[test]
    fn test_causal_overlay_revenue_cycle_template() {
        let mut config = create_test_config();
        config.causal.enabled = true;
        config.causal.template = "revenue_cycle".to_string();
        config.causal.sample_size = 50;
        config.causal.validate = false;

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

        // Causal phase should have generated samples
        assert_eq!(result.statistics.causal_samples_generated, 50);
        // Validation was disabled
        assert!(result.statistics.causal_validation_passed.is_none());
    }

    #[test]
    fn test_all_new_phases_enabled_together() {
        let mut config = create_test_config();
        config.llm.enabled = true;
        config.llm.max_vendor_enrichments = 2;
        config.diffusion.enabled = true;
        config.diffusion.n_steps = 20;
        config.diffusion.sample_size = 10;
        config.causal.enabled = true;
        config.causal.sample_size = 50;
        config.causal.validate = true;

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 3,
            materials_per_company: 3,
            assets_per_company: 3,
            employees_per_company: 3,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // All three phases should have run
        assert!(result.statistics.llm_vendors_enriched > 0);
        assert_eq!(result.statistics.diffusion_samples_generated, 10);
        assert_eq!(result.statistics.causal_samples_generated, 50);
        assert!(result.statistics.causal_validation_passed.is_some());
    }

    #[test]
    fn test_statistics_serialization_with_new_fields() {
        let stats = EnhancedGenerationStatistics {
            total_entries: 100,
            total_line_items: 500,
            llm_enrichment_ms: 42,
            llm_vendors_enriched: 10,
            diffusion_enhancement_ms: 100,
            diffusion_samples_generated: 50,
            causal_generation_ms: 200,
            causal_samples_generated: 100,
            causal_validation_passed: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: EnhancedGenerationStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.llm_enrichment_ms, 42);
        assert_eq!(deserialized.llm_vendors_enriched, 10);
        assert_eq!(deserialized.diffusion_enhancement_ms, 100);
        assert_eq!(deserialized.diffusion_samples_generated, 50);
        assert_eq!(deserialized.causal_generation_ms, 200);
        assert_eq!(deserialized.causal_samples_generated, 100);
        assert_eq!(deserialized.causal_validation_passed, Some(true));
    }

    #[test]
    fn test_statistics_backward_compat_deserialization() {
        // Old JSON without the new fields should still deserialize
        let old_json = r#"{
            "total_entries": 100,
            "total_line_items": 500,
            "accounts_count": 50,
            "companies_count": 1,
            "period_months": 12,
            "vendor_count": 10,
            "customer_count": 20,
            "material_count": 15,
            "asset_count": 5,
            "employee_count": 8,
            "p2p_chain_count": 5,
            "o2c_chain_count": 5,
            "ap_invoice_count": 5,
            "ar_invoice_count": 5,
            "ocpm_event_count": 0,
            "ocpm_object_count": 0,
            "ocpm_case_count": 0,
            "audit_engagement_count": 0,
            "audit_workpaper_count": 0,
            "audit_evidence_count": 0,
            "audit_risk_count": 0,
            "audit_finding_count": 0,
            "audit_judgment_count": 0,
            "anomalies_injected": 0,
            "data_quality_issues": 0,
            "banking_customer_count": 0,
            "banking_account_count": 0,
            "banking_transaction_count": 0,
            "banking_suspicious_count": 0,
            "graph_export_count": 0,
            "graph_node_count": 0,
            "graph_edge_count": 0
        }"#;

        let stats: EnhancedGenerationStatistics = serde_json::from_str(old_json).unwrap();

        // New fields should default to 0 / None
        assert_eq!(stats.llm_enrichment_ms, 0);
        assert_eq!(stats.llm_vendors_enriched, 0);
        assert_eq!(stats.diffusion_enhancement_ms, 0);
        assert_eq!(stats.diffusion_samples_generated, 0);
        assert_eq!(stats.causal_generation_ms, 0);
        assert_eq!(stats.causal_samples_generated, 0);
        assert!(stats.causal_validation_passed.is_none());
    }
}
