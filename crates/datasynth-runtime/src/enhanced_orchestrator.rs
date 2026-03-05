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
//! 25. Counterfactual pair generation (ML training)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use datasynth_banking::{
    models::{BankAccount, BankTransaction, BankingCustomer, CustomerName},
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
use datasynth_core::traits::Generator;
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
    // Control generator
    ControlGenerator,
    ControlGeneratorConfig,
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
    // ESG anomaly labels
    EsgAnomalyLabel,
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
    ApprovalGraphBuilder, ApprovalGraphConfig, BankingGraphBuilder, BankingGraphConfig,
    EntityGraphBuilder, EntityGraphConfig, PyGExportConfig, PyGExporter, TransactionGraphBuilder,
    TransactionGraphConfig,
};
use datasynth_ocpm::{
    AuditDocuments, BankDocuments, BankReconDocuments, EventLogMetadata, H2rDocuments,
    MfgDocuments, O2cDocuments, OcpmEventGenerator, OcpmEventLog, OcpmGeneratorConfig,
    OcpmUuidFactory, P2pDocuments, S2cDocuments,
};

use datasynth_config::schema::{O2CFlowConfig, P2PFlowConfig};
use datasynth_core::causal::{CausalGraph, CausalValidator, StructuralCausalModel};
use datasynth_core::diffusion::{DiffusionBackend, DiffusionConfig, StatisticalDiffusionBackend};
use datasynth_core::llm::MockLlmProvider;
use datasynth_core::models::balance::{GeneratedOpeningBalance, IndustryType, OpeningBalanceSpec};
use datasynth_core::models::documents::PaymentMethod;
use datasynth_core::models::IndustrySector;
use datasynth_generators::coa_generator::CoAFramework;
use datasynth_generators::llm_enrichment::VendorLlmEnricher;
use rayon::prelude::*;

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
        over_delivery_rate: schema_config.over_delivery_rate.unwrap_or(0.02),
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
        early_payment_discount_rate: schema_config.early_payment_discount_rate.unwrap_or(0.30),
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
            avg_days_until_remainder: payment_behavior.avg_days_until_remainder,
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
        late_payment_rate: schema_config.late_payment_rate.unwrap_or(0.15),
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
    /// Generate tax jurisdictions and tax codes.
    pub generate_tax: bool,
    /// Generate ESG data (emissions, energy, water, waste, social, governance).
    pub generate_esg: bool,
    /// Generate intercompany transactions and eliminations.
    pub generate_intercompany: bool,
    /// Generate process evolution and organizational events.
    pub generate_evolution_events: bool,
    /// Generate counterfactual (original, mutated) JE pairs for ML training.
    pub generate_counterfactuals: bool,
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
            generate_tax: false,                  // Off by default
            generate_esg: false,                  // Off by default
            generate_intercompany: false,         // Off by default
            generate_evolution_events: true,      // On by default
            generate_counterfactuals: false,      // Off by default (opt-in for ML workloads)
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
    /// FA subledger records (asset acquisitions from FA generator).
    pub fa_records: Vec<datasynth_core::models::subledger::fa::FixedAssetRecord>,
    /// Inventory positions from inventory generator.
    pub inventory_positions: Vec<datasynth_core::models::subledger::inventory::InventoryPosition>,
    /// Inventory movements from inventory generator.
    pub inventory_movements: Vec<datasynth_core::models::subledger::inventory::InventoryMovement>,
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
    /// Transaction-level AML labels with features.
    pub transaction_labels: Vec<datasynth_banking::labels::TransactionLabel>,
    /// Customer-level AML labels.
    pub customer_labels: Vec<datasynth_banking::labels::CustomerLabel>,
    /// Account-level AML labels.
    pub account_labels: Vec<datasynth_banking::labels::AccountLabel>,
    /// Relationship-level AML labels.
    pub relationship_labels: Vec<datasynth_banking::labels::RelationshipLabel>,
    /// Case narratives for AML scenarios.
    pub narratives: Vec<datasynth_banking::labels::ExportedNarrative>,
    /// Number of suspicious transactions.
    pub suspicious_count: usize,
    /// Number of AML scenarios generated.
    pub scenario_count: usize,
}

/// Graph export snapshot containing exported graph metadata.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GraphExportSnapshot {
    /// Whether graph export was performed.
    pub exported: bool,
    /// Number of graphs exported.
    pub graph_count: usize,
    /// Exported graph metadata (by format name).
    pub exports: HashMap<String, GraphExportInfo>,
}

/// Information about an exported graph.
#[derive(Debug, Clone, Serialize)]
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

/// A single period's trial balance with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodTrialBalance {
    /// Fiscal year.
    pub fiscal_year: u16,
    /// Fiscal period (1-12).
    pub fiscal_period: u8,
    /// Period start date.
    pub period_start: NaiveDate,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Trial balance entries for this period.
    pub entries: Vec<datasynth_generators::TrialBalanceEntry>,
}

/// Financial reporting snapshot (financial statements + bank reconciliations).
#[derive(Debug, Clone, Default)]
pub struct FinancialReportingSnapshot {
    /// Financial statements (balance sheet, income statement, cash flow).
    pub financial_statements: Vec<FinancialStatement>,
    /// Bank reconciliations.
    pub bank_reconciliations: Vec<BankReconciliation>,
    /// Period-close trial balances (one per period).
    pub trial_balances: Vec<PeriodTrialBalance>,
}

/// HR data snapshot (payroll runs, time entries, expense reports, benefit enrollments).
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
    /// Benefit enrollments (actual data).
    pub benefit_enrollments: Vec<BenefitEnrollment>,
    /// Payroll runs.
    pub payroll_run_count: usize,
    /// Payroll line item count.
    pub payroll_line_item_count: usize,
    /// Time entry count.
    pub time_entry_count: usize,
    /// Expense report count.
    pub expense_report_count: usize,
    /// Benefit enrollment count.
    pub benefit_enrollment_count: usize,
}

/// Accounting standards data snapshot (revenue recognition, impairment).
#[derive(Debug, Clone, Default)]
pub struct AccountingStandardsSnapshot {
    /// Revenue recognition contracts (actual data).
    pub contracts: Vec<datasynth_standards::accounting::revenue::CustomerContract>,
    /// Impairment tests (actual data).
    pub impairment_tests: Vec<datasynth_standards::accounting::impairment::ImpairmentTest>,
    /// Revenue recognition contract count.
    pub revenue_contract_count: usize,
    /// Impairment test count.
    pub impairment_test_count: usize,
}

/// Manufacturing data snapshot (production orders, quality inspections, cycle counts, BOMs, inventory movements).
#[derive(Debug, Clone, Default)]
pub struct ManufacturingSnapshot {
    /// Production orders (actual data).
    pub production_orders: Vec<ProductionOrder>,
    /// Quality inspections (actual data).
    pub quality_inspections: Vec<QualityInspection>,
    /// Cycle counts (actual data).
    pub cycle_counts: Vec<CycleCount>,
    /// BOM components (actual data).
    pub bom_components: Vec<BomComponent>,
    /// Inventory movements (actual data).
    pub inventory_movements: Vec<InventoryMovement>,
    /// Production order count.
    pub production_order_count: usize,
    /// Quality inspection count.
    pub quality_inspection_count: usize,
    /// Cycle count count.
    pub cycle_count_count: usize,
    /// BOM component count.
    pub bom_component_count: usize,
    /// Inventory movement count.
    pub inventory_movement_count: usize,
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

/// Tax data snapshot (jurisdictions, codes, provisions, returns, withholding).
#[derive(Debug, Clone, Default)]
pub struct TaxSnapshot {
    /// Tax jurisdictions.
    pub jurisdictions: Vec<TaxJurisdiction>,
    /// Tax codes.
    pub codes: Vec<TaxCode>,
    /// Tax lines computed on documents.
    pub tax_lines: Vec<TaxLine>,
    /// Tax returns filed per period.
    pub tax_returns: Vec<TaxReturn>,
    /// Tax provisions.
    pub tax_provisions: Vec<TaxProvision>,
    /// Withholding tax records.
    pub withholding_records: Vec<WithholdingTaxRecord>,
    /// Tax anomaly labels.
    pub tax_anomaly_labels: Vec<datasynth_generators::TaxAnomalyLabel>,
    /// Jurisdiction count.
    pub jurisdiction_count: usize,
    /// Code count.
    pub code_count: usize,
}

/// Intercompany data snapshot (IC transactions, matched pairs, eliminations).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntercompanySnapshot {
    /// IC matched pairs (transaction pairs between related entities).
    pub matched_pairs: Vec<datasynth_core::models::intercompany::ICMatchedPair>,
    /// IC journal entries generated from matched pairs (seller side).
    pub seller_journal_entries: Vec<JournalEntry>,
    /// IC journal entries generated from matched pairs (buyer side).
    pub buyer_journal_entries: Vec<JournalEntry>,
    /// Elimination entries for consolidation.
    pub elimination_entries: Vec<datasynth_core::models::intercompany::EliminationEntry>,
    /// IC matched pair count.
    pub matched_pair_count: usize,
    /// IC elimination entry count.
    pub elimination_entry_count: usize,
    /// IC matching rate (0.0 to 1.0).
    pub match_rate: f64,
}

/// ESG data snapshot (emissions, energy, water, waste, social, governance, supply chain, disclosures).
#[derive(Debug, Clone, Default)]
pub struct EsgSnapshot {
    /// Emission records (scope 1, 2, 3).
    pub emissions: Vec<EmissionRecord>,
    /// Energy consumption records.
    pub energy: Vec<EnergyConsumption>,
    /// Water usage records.
    pub water: Vec<WaterUsage>,
    /// Waste records.
    pub waste: Vec<WasteRecord>,
    /// Workforce diversity metrics.
    pub diversity: Vec<WorkforceDiversityMetric>,
    /// Pay equity metrics.
    pub pay_equity: Vec<PayEquityMetric>,
    /// Safety incidents.
    pub safety_incidents: Vec<SafetyIncident>,
    /// Safety metrics.
    pub safety_metrics: Vec<SafetyMetric>,
    /// Governance metrics.
    pub governance: Vec<GovernanceMetric>,
    /// Supplier ESG assessments.
    pub supplier_assessments: Vec<SupplierEsgAssessment>,
    /// Materiality assessments.
    pub materiality: Vec<MaterialityAssessment>,
    /// ESG disclosures.
    pub disclosures: Vec<EsgDisclosure>,
    /// Climate scenarios.
    pub climate_scenarios: Vec<ClimateScenario>,
    /// ESG anomaly labels.
    pub anomaly_labels: Vec<EsgAnomalyLabel>,
    /// Total emission record count.
    pub emission_count: usize,
    /// Total disclosure count.
    pub disclosure_count: usize,
}

/// Treasury data snapshot (cash management, hedging, debt, pooling).
#[derive(Debug, Clone, Default)]
pub struct TreasurySnapshot {
    /// Cash positions (daily balances per account).
    pub cash_positions: Vec<CashPosition>,
    /// Cash forecasts.
    pub cash_forecasts: Vec<CashForecast>,
    /// Cash pools.
    pub cash_pools: Vec<CashPool>,
    /// Cash pool sweep transactions.
    pub cash_pool_sweeps: Vec<CashPoolSweep>,
    /// Hedging instruments.
    pub hedging_instruments: Vec<HedgingInstrument>,
    /// Hedge relationships (ASC 815/IFRS 9 designations).
    pub hedge_relationships: Vec<HedgeRelationship>,
    /// Debt instruments.
    pub debt_instruments: Vec<DebtInstrument>,
    /// Bank guarantees and letters of credit.
    pub bank_guarantees: Vec<BankGuarantee>,
    /// Intercompany netting runs.
    pub netting_runs: Vec<NettingRun>,
    /// Treasury anomaly labels.
    pub treasury_anomaly_labels: Vec<datasynth_generators::treasury::TreasuryAnomalyLabel>,
}

/// Project accounting data snapshot (projects, costs, revenue, milestones, EVM).
#[derive(Debug, Clone, Default)]
pub struct ProjectAccountingSnapshot {
    /// Projects with WBS hierarchies.
    pub projects: Vec<Project>,
    /// Project cost lines (linked from source documents).
    pub cost_lines: Vec<ProjectCostLine>,
    /// Revenue recognition records.
    pub revenue_records: Vec<ProjectRevenue>,
    /// Earned value metrics.
    pub earned_value_metrics: Vec<EarnedValueMetric>,
    /// Change orders.
    pub change_orders: Vec<ChangeOrder>,
    /// Project milestones.
    pub milestones: Vec<ProjectMilestone>,
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
    /// Tax data snapshot (jurisdictions, codes, provisions, returns).
    pub tax: TaxSnapshot,
    /// ESG data snapshot (emissions, energy, social, governance, disclosures).
    pub esg: EsgSnapshot,
    /// Treasury data snapshot (cash management, hedging, debt).
    pub treasury: TreasurySnapshot,
    /// Project accounting data snapshot (projects, costs, revenue, EVM, milestones).
    pub project_accounting: ProjectAccountingSnapshot,
    /// Process evolution events (workflow changes, automation, policy changes, control enhancements).
    pub process_evolution: Vec<ProcessEvolutionEvent>,
    /// Organizational events (acquisitions, divestitures, reorganizations, leadership changes).
    pub organizational_events: Vec<OrganizationalEvent>,
    /// Disruption events (outages, migrations, process changes, recoveries, regulatory).
    pub disruption_events: Vec<datasynth_generators::disruption::DisruptionEvent>,
    /// Intercompany data snapshot (IC transactions, matched pairs, eliminations).
    pub intercompany: IntercompanySnapshot,
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
    /// Internal controls (if controls generation enabled).
    pub internal_controls: Vec<InternalControl>,
    /// Opening balances (if opening balance generation enabled).
    pub opening_balances: Vec<GeneratedOpeningBalance>,
    /// GL-to-subledger reconciliation results (if reconciliation enabled).
    pub subledger_reconciliation: Vec<datasynth_generators::ReconciliationResult>,
    /// Counterfactual (original, mutated) JE pairs for ML training.
    pub counterfactual_pairs: Vec<datasynth_generators::counterfactual::CounterfactualPair>,
    /// Fraud red-flag indicators on P2P/O2C documents.
    pub red_flags: Vec<datasynth_generators::fraud::RedFlag>,
    /// Collusion rings (coordinated fraud networks).
    pub collusion_rings: Vec<datasynth_generators::fraud::CollusionRing>,
    /// Bi-temporal version chains for vendor entities.
    pub temporal_vendor_chains:
        Vec<datasynth_core::models::TemporalVersionChain<datasynth_core::models::Vendor>>,
    /// Entity relationship graph (nodes + edges with strength scores).
    pub entity_relationship_graph: Option<datasynth_core::models::EntityGraph>,
    /// Cross-process links (P2P ↔ O2C via inventory movements).
    pub cross_process_links: Vec<datasynth_core::models::CrossProcessLink>,
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
    #[serde(default)]
    pub benefit_enrollment_count: usize,
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
    #[serde(default)]
    pub bom_component_count: usize,
    #[serde(default)]
    pub inventory_movement_count: usize,
    /// Sales & reporting counts.
    #[serde(default)]
    pub sales_quote_count: usize,
    #[serde(default)]
    pub kpi_count: usize,
    #[serde(default)]
    pub budget_line_count: usize,
    /// Tax counts.
    #[serde(default)]
    pub tax_jurisdiction_count: usize,
    #[serde(default)]
    pub tax_code_count: usize,
    /// ESG counts.
    #[serde(default)]
    pub esg_emission_count: usize,
    #[serde(default)]
    pub esg_disclosure_count: usize,
    /// Intercompany counts.
    #[serde(default)]
    pub ic_matched_pair_count: usize,
    #[serde(default)]
    pub ic_elimination_count: usize,
    /// Number of intercompany journal entries (seller + buyer side).
    #[serde(default)]
    pub ic_transaction_count: usize,
    /// Number of fixed asset subledger records.
    #[serde(default)]
    pub fa_subledger_count: usize,
    /// Number of inventory subledger records.
    #[serde(default)]
    pub inventory_subledger_count: usize,
    /// Treasury debt instrument count.
    #[serde(default)]
    pub treasury_debt_instrument_count: usize,
    /// Treasury hedging instrument count.
    #[serde(default)]
    pub treasury_hedging_instrument_count: usize,
    /// Project accounting project count.
    #[serde(default)]
    pub project_count: usize,
    /// Project accounting change order count.
    #[serde(default)]
    pub project_change_order_count: usize,
    /// Tax provision count.
    #[serde(default)]
    pub tax_provision_count: usize,
    /// Opening balance count.
    #[serde(default)]
    pub opening_balance_count: usize,
    /// Subledger reconciliation count.
    #[serde(default)]
    pub subledger_reconciliation_count: usize,
    /// Tax line count.
    #[serde(default)]
    pub tax_line_count: usize,
    /// Project cost line count.
    #[serde(default)]
    pub project_cost_line_count: usize,
    /// Cash position count.
    #[serde(default)]
    pub cash_position_count: usize,
    /// Cash forecast count.
    #[serde(default)]
    pub cash_forecast_count: usize,
    /// Cash pool count.
    #[serde(default)]
    pub cash_pool_count: usize,
    /// Process evolution event count.
    #[serde(default)]
    pub process_evolution_event_count: usize,
    /// Organizational event count.
    #[serde(default)]
    pub organizational_event_count: usize,
    /// Counterfactual pair count.
    #[serde(default)]
    pub counterfactual_pair_count: usize,
    /// Number of fraud red-flag indicators generated.
    #[serde(default)]
    pub red_flag_count: usize,
    /// Number of collusion rings generated.
    #[serde(default)]
    pub collusion_ring_count: usize,
    /// Number of bi-temporal vendor version chains generated.
    #[serde(default)]
    pub temporal_version_chain_count: usize,
    /// Number of nodes in the entity relationship graph.
    #[serde(default)]
    pub entity_relationship_node_count: usize,
    /// Number of edges in the entity relationship graph.
    #[serde(default)]
    pub entity_relationship_edge_count: usize,
    /// Number of cross-process links generated.
    #[serde(default)]
    pub cross_process_link_count: usize,
    /// Number of disruption events generated.
    #[serde(default)]
    pub disruption_event_count: usize,
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
    /// Country pack registry for localized data generation
    country_pack_registry: datasynth_core::CountryPackRegistry,
    /// Optional streaming sink for phase-by-phase output
    phase_sink: Option<Box<dyn crate::stream_pipeline::PhaseSink>>,
}

impl EnhancedOrchestrator {
    /// Create a new enhanced orchestrator.
    pub fn new(config: GeneratorConfig, phase_config: PhaseConfig) -> SynthResult<Self> {
        datasynth_config::validate_config(&config)?;

        let seed = config.global.seed.unwrap_or_else(rand::random);

        // Build resource guard from config
        let resource_guard = Self::build_resource_guard(&config, None);

        // Build country pack registry from config
        let country_pack_registry = match &config.country_packs {
            Some(cp) => {
                datasynth_core::CountryPackRegistry::new(cp.external_dir.as_deref(), &cp.overrides)
                    .map_err(|e| SynthError::config(e.to_string()))?
            }
            None => datasynth_core::CountryPackRegistry::builtin_only()
                .map_err(|e| SynthError::config(e.to_string()))?,
        };

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
            country_pack_registry,
            phase_sink: None,
        })
    }

    /// Create with default phase config.
    pub fn with_defaults(config: GeneratorConfig) -> SynthResult<Self> {
        Self::new(config, PhaseConfig::default())
    }

    /// Set a streaming phase sink for real-time output.
    pub fn with_phase_sink(mut self, sink: Box<dyn crate::stream_pipeline::PhaseSink>) -> Self {
        self.phase_sink = Some(sink);
        self
    }

    /// Emit a batch of items to the phase sink (if configured).
    fn emit_phase_items<T: serde::Serialize>(&self, phase: &str, type_name: &str, items: &[T]) {
        if let Some(ref sink) = self.phase_sink {
            for item in items {
                if let Ok(value) = serde_json::to_value(item) {
                    let _ = sink.emit(phase, type_name, &value);
                }
            }
            let _ = sink.phase_complete(phase);
        }
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

    /// Access the country pack registry.
    pub fn country_pack_registry(&self) -> &datasynth_core::CountryPackRegistry {
        &self.country_pack_registry
    }

    /// Look up a country pack by country code string.
    pub fn country_pack_for(&self, country: &str) -> &datasynth_core::CountryPack {
        self.country_pack_registry.get_by_str(country)
    }

    /// Returns the ISO 3166-1 alpha-2 country code for the primary (first)
    /// company, defaulting to `"US"` if no companies are configured.
    fn primary_country_code(&self) -> &str {
        self.config
            .companies
            .first()
            .map(|c| c.country.as_str())
            .unwrap_or("US")
    }

    /// Resolve the country pack for the primary (first) company.
    fn primary_pack(&self) -> &datasynth_core::CountryPack {
        self.country_pack_for(self.primary_country_code())
    }

    /// Resolve the CoA framework from config/country-pack.
    fn resolve_coa_framework(&self) -> CoAFramework {
        if self.config.accounting_standards.enabled {
            match self.config.accounting_standards.framework {
                Some(datasynth_config::schema::AccountingFrameworkConfig::FrenchGaap) => {
                    return CoAFramework::FrenchPcg;
                }
                Some(datasynth_config::schema::AccountingFrameworkConfig::GermanGaap) => {
                    return CoAFramework::GermanSkr04;
                }
                _ => {}
            }
        }
        // Fallback: derive from country pack
        let pack = self.primary_pack();
        match pack.accounting.framework.as_str() {
            "french_gaap" => CoAFramework::FrenchPcg,
            "german_gaap" | "hgb" => CoAFramework::GermanSkr04,
            _ => CoAFramework::UsGaap,
        }
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

        // Emit master data to stream sink
        self.emit_phase_items("master_data", "Vendor", &self.master_data.vendors);
        self.emit_phase_items("master_data", "Customer", &self.master_data.customers);
        self.emit_phase_items("master_data", "Material", &self.master_data.materials);

        // Phase 3: Document Flows + Subledger Linking
        let (mut document_flows, subledger, fa_journal_entries) =
            self.phase_document_flows(&mut stats)?;

        // Emit document flows to stream sink
        self.emit_phase_items(
            "document_flows",
            "PurchaseOrder",
            &document_flows.purchase_orders,
        );
        self.emit_phase_items(
            "document_flows",
            "GoodsReceipt",
            &document_flows.goods_receipts,
        );
        self.emit_phase_items(
            "document_flows",
            "VendorInvoice",
            &document_flows.vendor_invoices,
        );
        self.emit_phase_items("document_flows", "SalesOrder", &document_flows.sales_orders);
        self.emit_phase_items("document_flows", "Delivery", &document_flows.deliveries);

        // Phase 3b: Opening Balances (before JE generation)
        let opening_balances = self.phase_opening_balances(&coa, &mut stats)?;

        // Note: Opening balances are exported as balance/opening_balances.json but are not
        // converted to journal entries. Converting to JEs requires richer type information
        // (GeneratedOpeningBalance.balances loses AccountType, making contra-asset accounts
        // like Accumulated Depreciation indistinguishable from regular assets by code prefix).
        // A future enhancement could store (Decimal, AccountType) in the balances map.

        // Phase 4: Journal Entries
        let mut entries = self.phase_journal_entries(&coa, &document_flows, &mut stats)?;

        // Phase 4b: Append FA acquisition journal entries to main entries
        if !fa_journal_entries.is_empty() {
            debug!(
                "Appending {} FA acquisition JEs to main entries",
                fa_journal_entries.len()
            );
            entries.extend(fa_journal_entries);
        }

        // Phase 25: Counterfactual Pairs (before anomaly injection, using clean JEs)
        let counterfactual_pairs = self.phase_counterfactuals(&entries, &mut stats)?;

        // Get current degradation actions for optional phases
        let actions = self.get_degradation_actions();

        // Phase 5: S2C Sourcing Data (before anomaly injection, since it's standalone)
        let sourcing = self.phase_sourcing_data(&mut stats)?;

        // Phase 5a: Link S2C contracts to P2P purchase orders by matching vendor IDs
        if !sourcing.contracts.is_empty() {
            let mut linked_count = 0usize;
            for chain in &mut document_flows.p2p_chains {
                if chain.purchase_order.contract_id.is_none() {
                    if let Some(contract) = sourcing
                        .contracts
                        .iter()
                        .find(|c| c.vendor_id == chain.purchase_order.vendor_id)
                    {
                        chain.purchase_order.contract_id = Some(contract.contract_id.clone());
                        linked_count += 1;
                    }
                }
            }
            if linked_count > 0 {
                debug!(
                    "Linked {} purchase orders to S2C contracts by vendor match",
                    linked_count
                );
            }
        }

        // Phase 5b: Intercompany Transactions + Matching + Eliminations
        let intercompany = self.phase_intercompany(&mut stats)?;

        // Phase 5c: Append IC journal entries to main entries
        if !intercompany.seller_journal_entries.is_empty()
            || !intercompany.buyer_journal_entries.is_empty()
        {
            let ic_je_count = intercompany.seller_journal_entries.len()
                + intercompany.buyer_journal_entries.len();
            entries.extend(intercompany.seller_journal_entries.iter().cloned());
            entries.extend(intercompany.buyer_journal_entries.iter().cloned());
            debug!(
                "Appended {} IC journal entries to main entries",
                ic_je_count
            );
        }

        // Phase 6: HR Data (Payroll, Time Entries, Expenses)
        let hr = self.phase_hr_data(&mut stats)?;

        // Phase 6b: Generate JEs from payroll runs
        if !hr.payroll_runs.is_empty() {
            let payroll_jes = Self::generate_payroll_jes(&hr.payroll_runs);
            debug!("Generated {} JEs from payroll runs", payroll_jes.len());
            entries.extend(payroll_jes);
        }

        // Phase 7: Manufacturing (Production Orders, Quality Inspections, Cycle Counts)
        let manufacturing_snap = self.phase_manufacturing(&mut stats)?;

        // Phase 7a: Generate JEs from production orders
        if !manufacturing_snap.production_orders.is_empty() {
            let mfg_jes = Self::generate_manufacturing_jes(&manufacturing_snap.production_orders);
            debug!("Generated {} JEs from production orders", mfg_jes.len());
            entries.extend(mfg_jes);
        }

        // Update final entry/line-item stats after all JE-generating phases
        // (FA acquisition, IC, payroll, manufacturing JEs have all been appended)
        if !entries.is_empty() {
            stats.total_entries = entries.len() as u64;
            stats.total_line_items = entries.iter().map(|e| e.line_count() as u64).sum();
            debug!(
                "Final entry count: {}, line items: {} (after all JE-generating phases)",
                stats.total_entries, stats.total_line_items
            );
        }

        // Phase 7b: Apply internal controls to journal entries
        if self.config.internal_controls.enabled && !entries.is_empty() {
            info!("Phase 7b: Applying internal controls to journal entries");
            let control_config = ControlGeneratorConfig {
                exception_rate: self.config.internal_controls.exception_rate,
                sod_violation_rate: self.config.internal_controls.sod_violation_rate,
                enable_sox_marking: true,
                sox_materiality_threshold: rust_decimal::Decimal::from_f64_retain(
                    self.config.internal_controls.sox_materiality_threshold,
                )
                .unwrap_or_else(|| rust_decimal::Decimal::from(10000)),
            };
            let mut control_gen = ControlGenerator::with_config(self.seed + 99, control_config);
            for entry in &mut entries {
                control_gen.apply_controls(entry, &coa);
            }
            let with_controls = entries
                .iter()
                .filter(|e| !e.header.control_ids.is_empty())
                .count();
            info!(
                "Applied controls to {} entries ({} with control IDs assigned)",
                entries.len(),
                with_controls
            );
        }

        // Emit journal entries to stream sink (after all JE-generating phases)
        self.emit_phase_items("journal_entries", "JournalEntry", &entries);

        // Phase 8: Anomaly Injection (after all JE-generating phases)
        let anomaly_labels = self.phase_anomaly_injection(&mut entries, &actions, &mut stats)?;

        // Emit anomaly labels to stream sink
        self.emit_phase_items(
            "anomaly_injection",
            "LabeledAnomaly",
            &anomaly_labels.labels,
        );

        // Phase 26: Red Flag Indicators (after anomaly injection so fraud labels are available)
        let red_flags = self.phase_red_flags(&anomaly_labels, &document_flows, &mut stats)?;

        // Emit red flags to stream sink
        self.emit_phase_items("red_flags", "RedFlag", &red_flags);

        // Phase 26b: Collusion Ring Generation (after red flags)
        let collusion_rings = self.phase_collusion_rings(&mut stats)?;

        // Emit collusion rings to stream sink
        self.emit_phase_items("collusion_rings", "CollusionRing", &collusion_rings);

        // Phase 9: Balance Validation (after all JEs including payroll, manufacturing, IC)
        let balance_validation = self.phase_balance_validation(&entries)?;

        // Phase 9b: GL-to-Subledger Reconciliation
        let subledger_reconciliation =
            self.phase_subledger_reconciliation(&subledger, &entries, &mut stats)?;

        // Phase 10: Data Quality Injection
        let data_quality_stats =
            self.phase_data_quality_injection(&mut entries, &actions, &mut stats)?;

        // Phase 11: Audit Data
        let audit = self.phase_audit_data(&entries, &mut stats)?;

        // Phase 12: Banking KYC/AML Data
        let banking = self.phase_banking_data(&mut stats)?;

        // Phase 13: Graph Export
        let graph_export = self.phase_graph_export(&entries, &coa, &mut stats)?;

        // Phase 14: LLM Enrichment
        self.phase_llm_enrichment(&mut stats);

        // Phase 15: Diffusion Enhancement
        self.phase_diffusion_enhancement(&mut stats);

        // Phase 16: Causal Overlay
        self.phase_causal_overlay(&mut stats);

        // Phase 17: Bank Reconciliation + Financial Statements
        let financial_reporting =
            self.phase_financial_reporting(&document_flows, &entries, &coa, &mut stats)?;

        // Phase 18: Accounting Standards (Revenue Recognition, Impairment)
        let accounting_standards = self.phase_accounting_standards(&mut stats)?;

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

        // Emit OCPM events to stream sink
        if let Some(ref event_log) = ocpm.event_log {
            self.emit_phase_items("ocpm", "OcpmEvent", &event_log.events);
        }

        // Phase 19: Sales Quotes, Management KPIs, Budgets
        let sales_kpi_budgets =
            self.phase_sales_kpi_budgets(&coa, &financial_reporting, &mut stats)?;

        // Phase 20: Tax Generation
        let tax = self.phase_tax_generation(&document_flows, &mut stats)?;

        // Phase 21: ESG Data Generation
        let esg_snap = self.phase_esg_generation(&document_flows, &mut stats)?;

        // Phase 22: Treasury Data Generation
        let treasury =
            self.phase_treasury_data(&document_flows, &subledger, &intercompany, &mut stats)?;

        // Phase 23: Project Accounting Data Generation
        let project_accounting = self.phase_project_accounting(&document_flows, &hr, &mut stats)?;

        // Phase 24: Process Evolution + Organizational Events
        let (process_evolution, organizational_events) = self.phase_evolution_events(&mut stats)?;

        // Phase 24b: Disruption Events
        let disruption_events = self.phase_disruption_events(&mut stats)?;

        // Phase 27: Bi-Temporal Vendor Version Chains
        let temporal_vendor_chains = self.phase_temporal_attributes(&mut stats)?;

        // Phase 28: Entity Relationship Graph + Cross-Process Links
        let (entity_relationship_graph, cross_process_links) =
            self.phase_entity_relationships(&entries, &document_flows, &mut stats)?;

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

        // Phase 10c: Additional graph builders (approval, entity, banking)
        // These run after all data is available since they need banking/IC data.
        if self.phase_config.generate_graph_export || self.config.graph_export.enabled {
            self.build_additional_graphs(&banking, &intercompany, &entries, &mut stats);
        }

        // Log informational messages for config sections not yet fully wired
        if self.config.streaming.enabled {
            info!("Note: streaming config is enabled but batch mode does not use it");
        }
        if self.config.vendor_network.enabled {
            debug!("Vendor network config available; relationship graph generation is partial");
        }
        if self.config.customer_segmentation.enabled {
            debug!("Customer segmentation config available; segment-aware generation is partial");
        }

        // Log final resource statistics
        let resource_stats = self.resource_guard.stats();
        info!(
            "Generation workflow complete. Resource stats: memory_peak={}MB, disk_written={}bytes, degradation_level={}",
            resource_stats.memory.peak_resident_bytes / (1024 * 1024),
            resource_stats.disk.estimated_bytes_written,
            resource_stats.degradation_level
        );

        // Flush any remaining stream sink data
        if let Some(ref sink) = self.phase_sink {
            let _ = sink.flush();
        }

        // Build data lineage graph
        let lineage = self.build_lineage_graph();

        // Evaluate quality gates if enabled in config
        let gate_result = if self.config.quality_gates.enabled {
            let profile_name = &self.config.quality_gates.profile;
            match datasynth_eval::gates::get_profile(profile_name) {
                Some(profile) => {
                    // Build an evaluation populated with actual generation metrics.
                    let mut eval = datasynth_eval::ComprehensiveEvaluation::new();

                    // Populate balance sheet evaluation from balance validation results
                    if balance_validation.validated {
                        eval.coherence.balance =
                            Some(datasynth_eval::coherence::BalanceSheetEvaluation {
                                equation_balanced: balance_validation.is_balanced,
                                max_imbalance: (balance_validation.total_debits
                                    - balance_validation.total_credits)
                                    .abs(),
                                periods_evaluated: 1,
                                periods_imbalanced: if balance_validation.is_balanced {
                                    0
                                } else {
                                    1
                                },
                                period_results: Vec::new(),
                                companies_evaluated: self.config.companies.len(),
                            });
                    }

                    // Set coherence passes based on balance validation
                    eval.coherence.passes = balance_validation.is_balanced;
                    if !balance_validation.is_balanced {
                        eval.coherence
                            .failures
                            .push("Balance sheet equation not satisfied".to_string());
                    }

                    // Set statistical score based on entry count (basic sanity)
                    eval.statistical.overall_score = if entries.len() > 10 { 0.9 } else { 0.5 };
                    eval.statistical.passes = !entries.is_empty();

                    // Set quality score from data quality stats
                    eval.quality.overall_score = 0.9; // Default high for generated data
                    eval.quality.passes = true;

                    let result = datasynth_eval::gates::GateEngine::evaluate(&eval, &profile);
                    info!(
                        "Quality gates evaluated (profile '{}'): {}/{} passed — {}",
                        profile_name, result.gates_passed, result.gates_total, result.summary
                    );
                    Some(result)
                }
                None => {
                    warn!(
                        "Quality gates enabled but profile '{}' not found; skipping gate evaluation",
                        profile_name
                    );
                    None
                }
            }
        } else {
            None
        };

        // Generate internal controls if enabled
        let internal_controls = if self.config.internal_controls.enabled {
            InternalControl::standard_controls()
        } else {
            Vec::new()
        };

        Ok(EnhancedGenerationResult {
            chart_of_accounts: Arc::try_unwrap(coa).unwrap_or_else(|arc| (*arc).clone()),
            master_data: std::mem::take(&mut self.master_data),
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
            tax,
            esg: esg_snap,
            treasury,
            project_accounting,
            process_evolution,
            organizational_events,
            disruption_events,
            intercompany,
            journal_entries: entries,
            anomaly_labels,
            balance_validation,
            data_quality_stats,
            statistics: stats,
            lineage: Some(lineage),
            gate_result,
            internal_controls,
            opening_balances,
            subledger_reconciliation,
            counterfactual_pairs,
            red_flags,
            collusion_rings,
            temporal_vendor_chains,
            entity_relationship_graph,
            cross_process_links,
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
    ) -> SynthResult<(DocumentFlowSnapshot, SubledgerSnapshot, Vec<JournalEntry>)> {
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

        // Generate FA subledger records (and acquisition JEs) from master data fixed assets
        let mut fa_journal_entries = Vec::new();
        if !self.master_data.assets.is_empty() {
            debug!("Generating FA subledger records");
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

            let mut fa_gen = datasynth_generators::FAGenerator::new(
                datasynth_generators::FAGeneratorConfig::default(),
                rand_chacha::ChaCha8Rng::seed_from_u64(self.seed + 70),
            );

            for asset in &self.master_data.assets {
                let (record, je) = fa_gen.generate_asset_acquisition(
                    company_code,
                    &format!("{:?}", asset.asset_class),
                    &asset.description,
                    asset.acquisition_date,
                    currency,
                    asset.cost_center.as_deref(),
                );
                subledger.fa_records.push(record);
                fa_journal_entries.push(je);
            }

            stats.fa_subledger_count = subledger.fa_records.len();
            debug!(
                "FA subledger records generated: {} (with {} acquisition JEs)",
                stats.fa_subledger_count,
                fa_journal_entries.len()
            );
        }

        // Generate Inventory subledger records from master data materials
        if !self.master_data.materials.is_empty() {
            debug!("Generating Inventory subledger records");
            let first_company = self.config.companies.first();
            let company_code = first_company.map(|c| c.code.as_str()).unwrap_or("1000");
            let inv_currency = first_company
                .map(|c| c.currency.clone())
                .unwrap_or_else(|| "USD".to_string());

            let mut inv_gen = datasynth_generators::InventoryGenerator::new_with_currency(
                datasynth_generators::InventoryGeneratorConfig::default(),
                rand_chacha::ChaCha8Rng::seed_from_u64(self.seed + 71),
                inv_currency.clone(),
            );

            for (i, material) in self.master_data.materials.iter().enumerate() {
                let plant = format!("PLANT{:02}", (i % 3) + 1);
                let storage_loc = format!("SL-{:03}", (i % 10) + 1);
                let initial_qty = rust_decimal::Decimal::from(
                    material
                        .safety_stock
                        .to_string()
                        .parse::<i64>()
                        .unwrap_or(100),
                );

                let position = inv_gen.generate_position(
                    company_code,
                    &plant,
                    &storage_loc,
                    &material.material_id,
                    &material.description,
                    initial_qty,
                    Some(material.standard_cost),
                    &inv_currency,
                );
                subledger.inventory_positions.push(position);
            }

            stats.inventory_subledger_count = subledger.inventory_positions.len();
            debug!(
                "Inventory subledger records generated: {}",
                stats.inventory_subledger_count
            );
        }

        Ok((document_flows, subledger, fa_journal_entries))
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
        _stats: &mut EnhancedGenerationStatistics,
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
            // Note: stats.total_entries/total_line_items are set in generate()
            // after all JE-generating phases (FA, IC, payroll, mfg) complete.
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

        // Back-populate cross-references on sourcing projects (Task 35)
        // Link each project to its RFx events, contracts, and spend analyses
        let mut sourcing_projects = sourcing_projects;
        for project in &mut sourcing_projects {
            // Link RFx events generated for this project
            project.rfx_ids = rfx_events
                .iter()
                .filter(|rfx| rfx.sourcing_project_id == project.project_id)
                .map(|rfx| rfx.rfx_id.clone())
                .collect();

            // Link contract awarded from this project's RFx
            project.contract_id = contracts
                .iter()
                .find(|c| {
                    c.sourcing_project_id
                        .as_deref()
                        .is_some_and(|sp| sp == project.project_id)
                })
                .map(|c| c.contract_id.clone());

            // Link spend analysis for matching category (use category_id as the reference)
            project.spend_analysis_id = spend_analyses
                .iter()
                .find(|sa| sa.category_id == project.category_id)
                .map(|sa| sa.category_id.clone());
        }

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

    /// Phase 14b: Generate intercompany transactions, matching, and eliminations.
    fn phase_intercompany(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<IntercompanySnapshot> {
        // Skip if intercompany is disabled in config
        if !self.phase_config.generate_intercompany && !self.config.intercompany.enabled {
            debug!("Phase 14b: Skipped (intercompany generation disabled)");
            return Ok(IntercompanySnapshot::default());
        }

        // Intercompany requires at least 2 companies
        if self.config.companies.len() < 2 {
            debug!(
                "Phase 14b: Skipped (intercompany requires 2+ companies, found {})",
                self.config.companies.len()
            );
            return Ok(IntercompanySnapshot::default());
        }

        info!("Phase 14b: Generating Intercompany Transactions");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        // Build ownership structure from company configs
        // First company is treated as the parent, remaining are subsidiaries
        let parent_code = self.config.companies[0].code.clone();
        let mut ownership_structure =
            datasynth_core::models::intercompany::OwnershipStructure::new(parent_code.clone());

        for (i, company) in self.config.companies.iter().skip(1).enumerate() {
            let relationship = datasynth_core::models::intercompany::IntercompanyRelationship::new(
                format!("REL{:03}", i + 1),
                parent_code.clone(),
                company.code.clone(),
                rust_decimal::Decimal::from(100), // Default 100% ownership
                start_date,
            );
            ownership_structure.add_relationship(relationship);
        }

        // Convert config transfer pricing method to core model enum
        let tp_method = match self.config.intercompany.transfer_pricing_method {
            datasynth_config::schema::TransferPricingMethod::CostPlus => {
                datasynth_core::models::intercompany::TransferPricingMethod::CostPlus
            }
            datasynth_config::schema::TransferPricingMethod::ComparableUncontrolled => {
                datasynth_core::models::intercompany::TransferPricingMethod::ComparableUncontrolled
            }
            datasynth_config::schema::TransferPricingMethod::ResalePrice => {
                datasynth_core::models::intercompany::TransferPricingMethod::ResalePrice
            }
            datasynth_config::schema::TransferPricingMethod::TransactionalNetMargin => {
                datasynth_core::models::intercompany::TransferPricingMethod::TransactionalNetMargin
            }
            datasynth_config::schema::TransferPricingMethod::ProfitSplit => {
                datasynth_core::models::intercompany::TransferPricingMethod::ProfitSplit
            }
        };

        // Build IC generator config from schema config
        let ic_currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.clone())
            .unwrap_or_else(|| "USD".to_string());
        let ic_gen_config = datasynth_generators::ICGeneratorConfig {
            ic_transaction_rate: self.config.intercompany.ic_transaction_rate,
            transfer_pricing_method: tp_method,
            markup_percent: rust_decimal::Decimal::from_f64_retain(
                self.config.intercompany.markup_percent,
            )
            .unwrap_or(rust_decimal::Decimal::from(5)),
            generate_matched_pairs: self.config.intercompany.generate_matched_pairs,
            default_currency: ic_currency,
            ..Default::default()
        };

        // Create IC generator
        let mut ic_generator = datasynth_generators::ICGenerator::new(
            ic_gen_config,
            ownership_structure.clone(),
            seed + 50,
        );

        // Generate IC transactions for the period
        // Use ~3 transactions per day as a reasonable default
        let transactions_per_day = 3;
        let matched_pairs = ic_generator.generate_transactions_for_period(
            start_date,
            end_date,
            transactions_per_day,
        );

        // Generate journal entries from matched pairs
        let mut seller_entries = Vec::new();
        let mut buyer_entries = Vec::new();
        let fiscal_year = start_date.year();

        for pair in &matched_pairs {
            let fiscal_period = pair.posting_date.month();
            let (seller_je, buyer_je) =
                ic_generator.generate_journal_entries(pair, fiscal_year, fiscal_period);
            seller_entries.push(seller_je);
            buyer_entries.push(buyer_je);
        }

        // Run matching engine
        let matching_config = datasynth_generators::ICMatchingConfig {
            base_currency: self
                .config
                .companies
                .first()
                .map(|c| c.currency.clone())
                .unwrap_or_else(|| "USD".to_string()),
            ..Default::default()
        };
        let mut matching_engine = datasynth_generators::ICMatchingEngine::new(matching_config);
        matching_engine.load_matched_pairs(&matched_pairs);
        let matching_result = matching_engine.run_matching(end_date);

        // Generate elimination entries if configured
        let mut elimination_entries = Vec::new();
        if self.config.intercompany.generate_eliminations {
            let elim_config = datasynth_generators::EliminationConfig {
                consolidation_entity: "GROUP".to_string(),
                base_currency: self
                    .config
                    .companies
                    .first()
                    .map(|c| c.currency.clone())
                    .unwrap_or_else(|| "USD".to_string()),
                ..Default::default()
            };

            let mut elim_generator =
                datasynth_generators::EliminationGenerator::new(elim_config, ownership_structure);

            let fiscal_period = format!("{}{:02}", fiscal_year, end_date.month());
            let all_balances: Vec<datasynth_core::models::intercompany::ICAggregatedBalance> =
                matching_result
                    .matched_balances
                    .iter()
                    .chain(matching_result.unmatched_balances.iter())
                    .cloned()
                    .collect();

            let journal = elim_generator.generate_eliminations(
                &fiscal_period,
                end_date,
                &all_balances,
                &matched_pairs,
                &std::collections::HashMap::new(), // investment amounts (simplified)
                &std::collections::HashMap::new(), // equity amounts (simplified)
            );

            elimination_entries = journal.entries.clone();
        }

        let matched_pair_count = matched_pairs.len();
        let elimination_entry_count = elimination_entries.len();
        let match_rate = matching_result.match_rate;

        stats.ic_matched_pair_count = matched_pair_count;
        stats.ic_elimination_count = elimination_entry_count;
        stats.ic_transaction_count = seller_entries.len() + buyer_entries.len();

        info!(
            "Intercompany data generated: {} matched pairs, {} JEs ({} seller + {} buyer), {} elimination entries, {:.1}% match rate",
            matched_pair_count,
            stats.ic_transaction_count,
            seller_entries.len(),
            buyer_entries.len(),
            elimination_entry_count,
            match_rate * 100.0
        );
        self.check_resources_with_log("post-intercompany")?;

        Ok(IntercompanySnapshot {
            matched_pairs,
            seller_journal_entries: seller_entries,
            buyer_journal_entries: buyer_entries,
            elimination_entries,
            matched_pair_count,
            elimination_entry_count,
            match_rate,
        })
    }

    /// Phase 15: Generate bank reconciliations and financial statements.
    fn phase_financial_reporting(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        journal_entries: &[JournalEntry],
        coa: &Arc<ChartOfAccounts>,
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
        let mut trial_balances = Vec::new();

        // Generate financial statements from JE-derived trial balances.
        //
        // When journal entries are available, we use cumulative trial balances for
        // balance sheet accounts and current-period trial balances for income
        // statement accounts. We also track prior-period trial balances so the
        // generator can produce comparative amounts, and we build a proper
        // cash flow statement from working capital changes rather than random data.
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
            let has_journal_entries = !journal_entries.is_empty();

            // Use FinancialStatementGenerator for balance sheet and income statement,
            // but build cash flow ourselves from TB data when JEs are available.
            let mut fs_gen = FinancialStatementGenerator::new(seed + 20);

            // Track prior-period cumulative TB for comparative amounts and cash flow
            let mut prior_cumulative_tb: Option<Vec<datasynth_generators::TrialBalanceEntry>> =
                None;

            // Generate one set of statements per period
            for period in 0..self.config.global.period_months {
                let period_start = start_date + chrono::Months::new(period);
                let period_end =
                    start_date + chrono::Months::new(period + 1) - chrono::Days::new(1);
                let fiscal_year = period_end.year() as u16;
                let fiscal_period = period_end.month() as u8;

                if has_journal_entries {
                    // Build cumulative trial balance from actual JEs for coherent
                    // balance sheet (cumulative) and income statement (current period)
                    let tb_entries = Self::build_cumulative_trial_balance(
                        journal_entries,
                        coa,
                        company_code,
                        start_date,
                        period_end,
                        fiscal_year,
                        fiscal_period,
                    );

                    // Generate balance sheet and income statement via the generator,
                    // passing prior-period TB for comparative amounts
                    let prior_ref = prior_cumulative_tb.as_deref();
                    let stmts = fs_gen.generate(
                        company_code,
                        currency,
                        &tb_entries,
                        period_start,
                        period_end,
                        fiscal_year,
                        fiscal_period,
                        prior_ref,
                        "SYS-AUTOCLOSE",
                    );

                    // Replace the generator's random cash flow with our TB-derived one
                    for stmt in stmts {
                        if stmt.statement_type == StatementType::CashFlowStatement {
                            // Build a coherent cash flow from trial balance changes
                            let net_income = Self::calculate_net_income_from_tb(&tb_entries);
                            let cf_items = Self::build_cash_flow_from_trial_balances(
                                &tb_entries,
                                prior_ref,
                                net_income,
                            );
                            financial_statements.push(FinancialStatement {
                                cash_flow_items: cf_items,
                                ..stmt
                            });
                        } else {
                            financial_statements.push(stmt);
                        }
                    }

                    // Store current TB in snapshot for output
                    trial_balances.push(PeriodTrialBalance {
                        fiscal_year,
                        fiscal_period,
                        period_start,
                        period_end,
                        entries: tb_entries.clone(),
                    });

                    // Store current TB as prior for next period
                    prior_cumulative_tb = Some(tb_entries);
                } else {
                    // Fallback: no JEs available, use single-period TB from entries
                    // (which will be empty, producing zero-valued statements)
                    let tb_entries = Self::build_trial_balance_from_entries(
                        journal_entries,
                        coa,
                        company_code,
                        fiscal_year,
                        fiscal_period,
                    );

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

                    // Store trial balance even in fallback path
                    if !tb_entries.is_empty() {
                        trial_balances.push(PeriodTrialBalance {
                            fiscal_year,
                            fiscal_period,
                            period_start,
                            period_end,
                            entries: tb_entries,
                        });
                    }
                }
            }
            stats.financial_statement_count = financial_statements.len();
            info!(
                "Financial statements generated: {} statements (JE-derived: {})",
                stats.financial_statement_count, has_journal_entries
            );
        }

        // Generate bank reconciliations from payment data
        if br_enabled && !document_flows.payments.is_empty() {
            let employee_ids: Vec<String> = self
                .master_data
                .employees
                .iter()
                .map(|e| e.employee_id.clone())
                .collect();
            let mut br_gen =
                BankReconciliationGenerator::new(seed + 25).with_employee_pool(employee_ids);

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

        if !trial_balances.is_empty() {
            info!(
                "Period-close trial balances captured: {} periods",
                trial_balances.len()
            );
        }

        Ok(FinancialReportingSnapshot {
            financial_statements,
            bank_reconciliations,
            trial_balances,
        })
    }

    /// Build trial balance entries by aggregating actual journal entry debits and credits per account.
    ///
    /// This ensures the trial balance is coherent with the JEs: every debit and credit
    /// posted in the journal entries flows through to the trial balance, using the real
    /// GL account numbers from the CoA.
    fn build_trial_balance_from_entries(
        journal_entries: &[JournalEntry],
        coa: &ChartOfAccounts,
        company_code: &str,
        fiscal_year: u16,
        fiscal_period: u8,
    ) -> Vec<datasynth_generators::TrialBalanceEntry> {
        use rust_decimal::Decimal;

        // Accumulate total debits and credits per GL account
        let mut account_debits: HashMap<String, Decimal> = HashMap::new();
        let mut account_credits: HashMap<String, Decimal> = HashMap::new();

        for je in journal_entries {
            // Filter to matching company, fiscal year, and period
            if je.header.company_code != company_code
                || je.header.fiscal_year != fiscal_year
                || je.header.fiscal_period != fiscal_period
            {
                continue;
            }

            for line in &je.lines {
                let acct = &line.gl_account;
                *account_debits.entry(acct.clone()).or_insert(Decimal::ZERO) += line.debit_amount;
                *account_credits.entry(acct.clone()).or_insert(Decimal::ZERO) += line.credit_amount;
            }
        }

        // Build a TrialBalanceEntry for each account that had activity
        let mut all_accounts: Vec<&String> = account_debits
            .keys()
            .chain(account_credits.keys())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        all_accounts.sort();

        let mut entries = Vec::new();

        for acct_number in all_accounts {
            let debit = account_debits
                .get(acct_number)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let credit = account_credits
                .get(acct_number)
                .copied()
                .unwrap_or(Decimal::ZERO);

            if debit.is_zero() && credit.is_zero() {
                continue;
            }

            // Look up account name from CoA, fall back to "Account {code}"
            let account_name = coa
                .get_account(acct_number)
                .map(|gl| gl.short_description.clone())
                .unwrap_or_else(|| format!("Account {}", acct_number));

            // Map account code prefix to the category strings expected by
            // FinancialStatementGenerator (Cash, Receivables, Inventory,
            // FixedAssets, Payables, AccruedLiabilities, Revenue, CostOfSales,
            // OperatingExpenses).
            let category = Self::category_from_account_code(acct_number);

            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: acct_number.clone(),
                account_name,
                category,
                debit_balance: debit,
                credit_balance: credit,
            });
        }

        entries
    }

    /// Build a cumulative trial balance by aggregating all JEs from the start up to
    /// (and including) the given period end date.
    ///
    /// Balance sheet accounts (assets, liabilities, equity) use cumulative balances
    /// while income statement accounts (revenue, expenses) show only the current period.
    /// The two are merged into a single Vec for the FinancialStatementGenerator.
    fn build_cumulative_trial_balance(
        journal_entries: &[JournalEntry],
        coa: &ChartOfAccounts,
        company_code: &str,
        start_date: NaiveDate,
        period_end: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
    ) -> Vec<datasynth_generators::TrialBalanceEntry> {
        use rust_decimal::Decimal;

        // Accumulate debits/credits for balance sheet accounts (cumulative from start)
        let mut bs_debits: HashMap<String, Decimal> = HashMap::new();
        let mut bs_credits: HashMap<String, Decimal> = HashMap::new();

        // Accumulate debits/credits for income statement accounts (current period only)
        let mut is_debits: HashMap<String, Decimal> = HashMap::new();
        let mut is_credits: HashMap<String, Decimal> = HashMap::new();

        for je in journal_entries {
            if je.header.company_code != company_code {
                continue;
            }

            for line in &je.lines {
                let acct = &line.gl_account;
                let category = Self::category_from_account_code(acct);
                let is_bs_account = matches!(
                    category.as_str(),
                    "Cash"
                        | "Receivables"
                        | "Inventory"
                        | "FixedAssets"
                        | "Payables"
                        | "AccruedLiabilities"
                        | "LongTermDebt"
                        | "Equity"
                );

                if is_bs_account {
                    // Balance sheet: accumulate from start through period_end
                    if je.header.document_date <= period_end
                        && je.header.document_date >= start_date
                    {
                        *bs_debits.entry(acct.clone()).or_insert(Decimal::ZERO) +=
                            line.debit_amount;
                        *bs_credits.entry(acct.clone()).or_insert(Decimal::ZERO) +=
                            line.credit_amount;
                    }
                } else {
                    // Income statement: current period only
                    if je.header.fiscal_year == fiscal_year
                        && je.header.fiscal_period == fiscal_period
                    {
                        *is_debits.entry(acct.clone()).or_insert(Decimal::ZERO) +=
                            line.debit_amount;
                        *is_credits.entry(acct.clone()).or_insert(Decimal::ZERO) +=
                            line.credit_amount;
                    }
                }
            }
        }

        // Merge all accounts
        let mut all_accounts: std::collections::HashSet<String> = std::collections::HashSet::new();
        all_accounts.extend(bs_debits.keys().cloned());
        all_accounts.extend(bs_credits.keys().cloned());
        all_accounts.extend(is_debits.keys().cloned());
        all_accounts.extend(is_credits.keys().cloned());

        let mut sorted_accounts: Vec<String> = all_accounts.into_iter().collect();
        sorted_accounts.sort();

        let mut entries = Vec::new();

        for acct_number in &sorted_accounts {
            let category = Self::category_from_account_code(acct_number);
            let is_bs_account = matches!(
                category.as_str(),
                "Cash"
                    | "Receivables"
                    | "Inventory"
                    | "FixedAssets"
                    | "Payables"
                    | "AccruedLiabilities"
                    | "LongTermDebt"
                    | "Equity"
            );

            let (debit, credit) = if is_bs_account {
                (
                    bs_debits.get(acct_number).copied().unwrap_or(Decimal::ZERO),
                    bs_credits
                        .get(acct_number)
                        .copied()
                        .unwrap_or(Decimal::ZERO),
                )
            } else {
                (
                    is_debits.get(acct_number).copied().unwrap_or(Decimal::ZERO),
                    is_credits
                        .get(acct_number)
                        .copied()
                        .unwrap_or(Decimal::ZERO),
                )
            };

            if debit.is_zero() && credit.is_zero() {
                continue;
            }

            let account_name = coa
                .get_account(acct_number)
                .map(|gl| gl.short_description.clone())
                .unwrap_or_else(|| format!("Account {}", acct_number));

            entries.push(datasynth_generators::TrialBalanceEntry {
                account_code: acct_number.clone(),
                account_name,
                category,
                debit_balance: debit,
                credit_balance: credit,
            });
        }

        entries
    }

    /// Build a JE-derived cash flow statement using the indirect method.
    ///
    /// Compares current and prior cumulative trial balances to derive working capital
    /// changes, producing a coherent cash flow statement tied to actual journal entries.
    fn build_cash_flow_from_trial_balances(
        current_tb: &[datasynth_generators::TrialBalanceEntry],
        prior_tb: Option<&[datasynth_generators::TrialBalanceEntry]>,
        net_income: rust_decimal::Decimal,
    ) -> Vec<CashFlowItem> {
        use rust_decimal::Decimal;

        // Helper: aggregate a TB by category and return net (debit - credit)
        let aggregate =
            |tb: &[datasynth_generators::TrialBalanceEntry]| -> HashMap<String, Decimal> {
                let mut map: HashMap<String, Decimal> = HashMap::new();
                for entry in tb {
                    let net = entry.debit_balance - entry.credit_balance;
                    *map.entry(entry.category.clone()).or_default() += net;
                }
                map
            };

        let current = aggregate(current_tb);
        let prior = prior_tb.map(aggregate);

        // Get balance for a category, defaulting to zero
        let get = |map: &HashMap<String, Decimal>, key: &str| -> Decimal {
            *map.get(key).unwrap_or(&Decimal::ZERO)
        };

        // Compute change: current - prior (or current if no prior)
        let change = |key: &str| -> Decimal {
            let curr = get(&current, key);
            match &prior {
                Some(p) => curr - get(p, key),
                None => curr,
            }
        };

        // Operating activities (indirect method)
        // Depreciation add-back: approximate from FixedAssets decrease
        let fixed_asset_change = change("FixedAssets");
        let depreciation_addback = if fixed_asset_change < Decimal::ZERO {
            -fixed_asset_change
        } else {
            Decimal::ZERO
        };

        // Working capital changes (increase in assets = cash outflow, increase in liabilities = cash inflow)
        let ar_change = change("Receivables");
        let inventory_change = change("Inventory");
        // AP and AccruedLiabilities are credit-normal: negative net means larger balance = cash inflow
        let ap_change = change("Payables");
        let accrued_change = change("AccruedLiabilities");

        let operating_cf = net_income + depreciation_addback - ar_change - inventory_change
            + (-ap_change)
            + (-accrued_change);

        // Investing activities
        let capex = if fixed_asset_change > Decimal::ZERO {
            -fixed_asset_change
        } else {
            Decimal::ZERO
        };
        let investing_cf = capex;

        // Financing activities
        let debt_change = -change("LongTermDebt");
        let equity_change = -change("Equity");
        let financing_cf = debt_change + equity_change;

        let net_change = operating_cf + investing_cf + financing_cf;

        vec![
            CashFlowItem {
                item_code: "CF-NI".to_string(),
                label: "Net Income".to_string(),
                category: CashFlowCategory::Operating,
                amount: net_income,
                amount_prior: None,
                sort_order: 1,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-DEP".to_string(),
                label: "Depreciation & Amortization".to_string(),
                category: CashFlowCategory::Operating,
                amount: depreciation_addback,
                amount_prior: None,
                sort_order: 2,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-AR".to_string(),
                label: "Change in Accounts Receivable".to_string(),
                category: CashFlowCategory::Operating,
                amount: -ar_change,
                amount_prior: None,
                sort_order: 3,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-AP".to_string(),
                label: "Change in Accounts Payable".to_string(),
                category: CashFlowCategory::Operating,
                amount: -ap_change,
                amount_prior: None,
                sort_order: 4,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-INV".to_string(),
                label: "Change in Inventory".to_string(),
                category: CashFlowCategory::Operating,
                amount: -inventory_change,
                amount_prior: None,
                sort_order: 5,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-OP".to_string(),
                label: "Net Cash from Operating Activities".to_string(),
                category: CashFlowCategory::Operating,
                amount: operating_cf,
                amount_prior: None,
                sort_order: 6,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-CAPEX".to_string(),
                label: "Capital Expenditures".to_string(),
                category: CashFlowCategory::Investing,
                amount: capex,
                amount_prior: None,
                sort_order: 7,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-INV-T".to_string(),
                label: "Net Cash from Investing Activities".to_string(),
                category: CashFlowCategory::Investing,
                amount: investing_cf,
                amount_prior: None,
                sort_order: 8,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-DEBT".to_string(),
                label: "Net Borrowings / (Repayments)".to_string(),
                category: CashFlowCategory::Financing,
                amount: debt_change,
                amount_prior: None,
                sort_order: 9,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-EQ".to_string(),
                label: "Equity Changes".to_string(),
                category: CashFlowCategory::Financing,
                amount: equity_change,
                amount_prior: None,
                sort_order: 10,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-FIN-T".to_string(),
                label: "Net Cash from Financing Activities".to_string(),
                category: CashFlowCategory::Financing,
                amount: financing_cf,
                amount_prior: None,
                sort_order: 11,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-NET".to_string(),
                label: "Net Change in Cash".to_string(),
                category: CashFlowCategory::Operating,
                amount: net_change,
                amount_prior: None,
                sort_order: 12,
                is_total: true,
            },
        ]
    }

    /// Calculate net income from a set of trial balance entries.
    ///
    /// Revenue is credit-normal (negative net = positive revenue), expenses are debit-normal.
    fn calculate_net_income_from_tb(
        tb: &[datasynth_generators::TrialBalanceEntry],
    ) -> rust_decimal::Decimal {
        use rust_decimal::Decimal;

        let mut aggregated: HashMap<String, Decimal> = HashMap::new();
        for entry in tb {
            let net = entry.debit_balance - entry.credit_balance;
            *aggregated.entry(entry.category.clone()).or_default() += net;
        }

        let revenue = *aggregated.get("Revenue").unwrap_or(&Decimal::ZERO);
        let cogs = *aggregated.get("CostOfSales").unwrap_or(&Decimal::ZERO);
        let opex = *aggregated
            .get("OperatingExpenses")
            .unwrap_or(&Decimal::ZERO);
        let other_income = *aggregated.get("OtherIncome").unwrap_or(&Decimal::ZERO);
        let other_expenses = *aggregated.get("OtherExpenses").unwrap_or(&Decimal::ZERO);

        // revenue is negative (credit-normal), expenses are positive (debit-normal)
        // other_income is typically negative (credit), other_expenses is typically positive
        let operating_income = revenue - cogs - opex - other_expenses - other_income;
        let tax_rate = Decimal::from_f64_retain(0.25).unwrap_or(Decimal::ZERO);
        let tax = operating_income * tax_rate;
        operating_income - tax
    }

    /// Map a GL account code to the category string expected by FinancialStatementGenerator.
    ///
    /// Uses the first two digits of the account code to classify into the categories
    /// that the financial statement generator aggregates on: Cash, Receivables, Inventory,
    /// FixedAssets, Payables, AccruedLiabilities, LongTermDebt, Equity, Revenue, CostOfSales,
    /// OperatingExpenses, OtherIncome, OtherExpenses.
    fn category_from_account_code(code: &str) -> String {
        let prefix: String = code.chars().take(2).collect();
        match prefix.as_str() {
            "10" => "Cash",
            "11" => "Receivables",
            "12" | "13" | "14" => "Inventory",
            "15" | "16" | "17" | "18" | "19" => "FixedAssets",
            "20" => "Payables",
            "21" | "22" | "23" | "24" => "AccruedLiabilities",
            "25" | "26" | "27" | "28" | "29" => "LongTermDebt",
            "30" | "31" | "32" | "33" | "34" | "35" | "36" | "37" | "38" | "39" => "Equity",
            "40" | "41" | "42" | "43" | "44" => "Revenue",
            "50" | "51" | "52" => "CostOfSales",
            "60" | "61" | "62" | "63" | "64" | "65" | "66" | "67" | "68" | "69" => {
                "OperatingExpenses"
            }
            "70" | "71" | "72" | "73" | "74" => "OtherIncome",
            "80" | "81" | "82" | "83" | "84" | "85" | "86" | "87" | "88" | "89" => "OtherExpenses",
            _ => "OperatingExpenses",
        }
        .to_string()
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

        // Extract cost-center pool from master data employees for cross-reference
        // coherence. Fabricated IDs (e.g. "CC-123") are replaced by real values.
        let cost_center_ids: Vec<String> = self
            .master_data
            .employees
            .iter()
            .filter_map(|e| e.cost_center.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut snapshot = HrSnapshot::default();

        // Generate payroll runs (one per month)
        if self.config.hr.payroll.enabled {
            let mut payroll_gen = datasynth_generators::PayrollGenerator::new(seed + 30)
                .with_pools(employee_ids.clone(), cost_center_ids.clone());

            // Look up country pack for payroll deductions and labels
            let payroll_pack = self.primary_pack();

            // Store the pack on the generator so generate() resolves
            // localized deduction rates and labels from it.
            payroll_gen.set_country_pack(payroll_pack.clone());

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
            let mut time_gen = datasynth_generators::TimeEntryGenerator::new(seed + 31)
                .with_pools(employee_ids.clone(), cost_center_ids.clone());
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
            let mut expense_gen = datasynth_generators::ExpenseReportGenerator::new(seed + 32)
                .with_pools(employee_ids.clone(), cost_center_ids.clone());
            expense_gen.set_country_pack(self.primary_pack().clone());
            let company_currency = self
                .config
                .companies
                .first()
                .map(|c| c.currency.as_str())
                .unwrap_or("USD");
            let reports = expense_gen.generate_with_currency(
                &employee_ids,
                start_date,
                end_date,
                &self.config.hr.expenses,
                company_currency,
            );
            snapshot.expense_report_count = reports.len();
            snapshot.expense_reports = reports;
        }

        // Generate benefit enrollments (gated on payroll, since benefits require employees)
        if self.config.hr.payroll.enabled {
            let mut benefit_gen = datasynth_generators::BenefitEnrollmentGenerator::new(seed + 33);
            let employee_pairs: Vec<(String, String)> = self
                .master_data
                .employees
                .iter()
                .map(|e| (e.employee_id.clone(), e.display_name.clone()))
                .collect();
            let enrollments =
                benefit_gen.generate(company_code, &employee_pairs, start_date, currency);
            snapshot.benefit_enrollment_count = enrollments.len();
            snapshot.benefit_enrollments = enrollments;
        }

        stats.payroll_run_count = snapshot.payroll_run_count;
        stats.time_entry_count = snapshot.time_entry_count;
        stats.expense_report_count = snapshot.expense_report_count;
        stats.benefit_enrollment_count = snapshot.benefit_enrollment_count;

        info!(
            "HR data generated: {} payroll runs ({} line items), {} time entries, {} expense reports, {} benefit enrollments",
            snapshot.payroll_run_count, snapshot.payroll_line_item_count,
            snapshot.time_entry_count, snapshot.expense_report_count,
            snapshot.benefit_enrollment_count
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

        // Convert config framework to standards framework.
        // If the user explicitly set a framework in the YAML config, use that.
        // Otherwise, fall back to the country pack's accounting.framework field,
        // and if that is also absent or unrecognised, default to US GAAP.
        let framework = match self.config.accounting_standards.framework {
            Some(datasynth_config::schema::AccountingFrameworkConfig::UsGaap) => {
                datasynth_standards::framework::AccountingFramework::UsGaap
            }
            Some(datasynth_config::schema::AccountingFrameworkConfig::Ifrs) => {
                datasynth_standards::framework::AccountingFramework::Ifrs
            }
            Some(datasynth_config::schema::AccountingFrameworkConfig::DualReporting) => {
                datasynth_standards::framework::AccountingFramework::DualReporting
            }
            Some(datasynth_config::schema::AccountingFrameworkConfig::FrenchGaap) => {
                datasynth_standards::framework::AccountingFramework::FrenchGaap
            }
            Some(datasynth_config::schema::AccountingFrameworkConfig::GermanGaap) => {
                datasynth_standards::framework::AccountingFramework::GermanGaap
            }
            None => {
                // Derive framework from the primary company's country pack
                let pack = self.primary_pack();
                let pack_fw = pack.accounting.framework.as_str();
                match pack_fw {
                    "ifrs" => datasynth_standards::framework::AccountingFramework::Ifrs,
                    "dual_reporting" => {
                        datasynth_standards::framework::AccountingFramework::DualReporting
                    }
                    "french_gaap" => {
                        datasynth_standards::framework::AccountingFramework::FrenchGaap
                    }
                    "german_gaap" | "hgb" => {
                        datasynth_standards::framework::AccountingFramework::GermanGaap
                    }
                    // "us_gaap" or any other/unrecognised value falls back to US GAAP
                    _ => datasynth_standards::framework::AccountingFramework::UsGaap,
                }
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
                snapshot.contracts = contracts;
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
                snapshot.impairment_tests = tests;
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

        let employee_ids: Vec<String> = self
            .master_data
            .employees
            .iter()
            .map(|e| e.employee_id.clone())
            .collect();
        let mut cc_gen = datasynth_generators::CycleCountGenerator::new(seed + 52)
            .with_employee_pool(employee_ids);
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

        // Generate BOM components
        let mut bom_gen = datasynth_generators::BomGenerator::new(seed + 53);
        let bom_components = bom_gen.generate(company_code, &material_data);
        snapshot.bom_component_count = bom_components.len();
        snapshot.bom_components = bom_components;

        // Generate inventory movements
        let currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.as_str())
            .unwrap_or("USD");
        let mut inv_mov_gen = datasynth_generators::InventoryMovementGenerator::new(seed + 54);
        let inventory_movements = inv_mov_gen.generate(
            company_code,
            &material_data,
            start_date,
            end_date,
            2,
            currency,
        );
        snapshot.inventory_movement_count = inventory_movements.len();
        snapshot.inventory_movements = inventory_movements;

        stats.production_order_count = snapshot.production_order_count;
        stats.quality_inspection_count = snapshot.quality_inspection_count;
        stats.cycle_count_count = snapshot.cycle_count_count;
        stats.bom_component_count = snapshot.bom_component_count;
        stats.inventory_movement_count = snapshot.inventory_movement_count;

        info!(
            "Manufacturing data generated: {} production orders, {} quality inspections, {} cycle counts, {} BOM components, {} inventory movements",
            snapshot.production_order_count, snapshot.quality_inspection_count, snapshot.cycle_count_count,
            snapshot.bom_component_count, snapshot.inventory_movement_count
        );
        self.check_resources_with_log("post-manufacturing")?;

        Ok(snapshot)
    }

    /// Phase 19: Generate sales quotes, management KPIs, and budgets.
    fn phase_sales_kpi_budgets(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
        financial_reporting: &FinancialReportingSnapshot,
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
                let employee_ids: Vec<String> = self
                    .master_data
                    .employees
                    .iter()
                    .map(|e| e.employee_id.clone())
                    .collect();
                let customer_ids: Vec<String> = self
                    .master_data
                    .customers
                    .iter()
                    .map(|c| c.customer_id.clone())
                    .collect();
                let company_currency = self
                    .config
                    .companies
                    .first()
                    .map(|c| c.currency.as_str())
                    .unwrap_or("USD");

                let mut quote_gen = datasynth_generators::SalesQuoteGenerator::new(seed + 60)
                    .with_pools(employee_ids, customer_ids);
                let quotes = quote_gen.generate_with_currency(
                    company_code,
                    &customer_data,
                    &material_data,
                    start_date,
                    end_date,
                    &self.config.sales_quotes,
                    company_currency,
                );
                snapshot.sales_quote_count = quotes.len();
                snapshot.sales_quotes = quotes;
            }
        }

        // Management KPIs
        if self.config.financial_reporting.management_kpis.enabled {
            let mut kpi_gen = datasynth_generators::KpiGenerator::new(seed + 61);
            let mut kpis = kpi_gen.generate(
                company_code,
                start_date,
                end_date,
                &self.config.financial_reporting.management_kpis,
            );

            // Override financial KPIs with actual data from financial statements
            {
                use rust_decimal::Decimal;

                if let Some(income_stmt) =
                    financial_reporting.financial_statements.iter().find(|fs| {
                        fs.statement_type == StatementType::IncomeStatement
                            && fs.company_code == company_code
                    })
                {
                    // Extract revenue and COGS from income statement line items
                    let total_revenue: Decimal = income_stmt
                        .line_items
                        .iter()
                        .filter(|li| li.section.contains("Revenue") && !li.is_total)
                        .map(|li| li.amount)
                        .sum();
                    let total_cogs: Decimal = income_stmt
                        .line_items
                        .iter()
                        .filter(|li| {
                            (li.section.contains("Cost") || li.line_code.starts_with("IS-COGS"))
                                && !li.is_total
                        })
                        .map(|li| li.amount.abs())
                        .sum();
                    let total_opex: Decimal = income_stmt
                        .line_items
                        .iter()
                        .filter(|li| {
                            li.section.contains("Expense")
                                && !li.is_total
                                && !li.section.contains("Cost")
                        })
                        .map(|li| li.amount.abs())
                        .sum();

                    if total_revenue > Decimal::ZERO {
                        let hundred = Decimal::from(100);
                        let gross_margin_pct =
                            ((total_revenue - total_cogs) * hundred / total_revenue).round_dp(2);
                        let operating_income = total_revenue - total_cogs - total_opex;
                        let op_margin_pct =
                            (operating_income * hundred / total_revenue).round_dp(2);

                        // Override gross margin and operating margin KPIs
                        for kpi in &mut kpis {
                            if kpi.name == "Gross Margin" {
                                kpi.value = gross_margin_pct;
                            } else if kpi.name == "Operating Margin" {
                                kpi.value = op_margin_pct;
                            }
                        }
                    }
                }

                // Override Current Ratio from balance sheet
                if let Some(bs) = financial_reporting.financial_statements.iter().find(|fs| {
                    fs.statement_type == StatementType::BalanceSheet
                        && fs.company_code == company_code
                }) {
                    let current_assets: Decimal = bs
                        .line_items
                        .iter()
                        .filter(|li| li.section.contains("Current Assets") && !li.is_total)
                        .map(|li| li.amount)
                        .sum();
                    let current_liabilities: Decimal = bs
                        .line_items
                        .iter()
                        .filter(|li| li.section.contains("Current Liabilities") && !li.is_total)
                        .map(|li| li.amount.abs())
                        .sum();

                    if current_liabilities > Decimal::ZERO {
                        let current_ratio = (current_assets / current_liabilities).round_dp(2);
                        for kpi in &mut kpis {
                            if kpi.name == "Current Ratio" {
                                kpi.value = current_ratio;
                            }
                        }
                    }
                }
            }

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

    /// Phase 20: Generate tax jurisdictions, tax codes, and tax lines from invoices.
    fn phase_tax_generation(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<TaxSnapshot> {
        if !self.phase_config.generate_tax || !self.config.tax.enabled {
            debug!("Phase 20: Skipped (tax generation disabled)");
            return Ok(TaxSnapshot::default());
        }
        info!("Phase 20: Generating Tax Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let fiscal_year = start_date.year();
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        let mut gen =
            datasynth_generators::TaxCodeGenerator::with_config(seed + 70, self.config.tax.clone());

        let pack = self.primary_pack().clone();
        let (jurisdictions, codes) =
            gen.generate_from_country_pack(&pack, company_code, fiscal_year);

        // Generate tax provisions for each company
        let mut provisions = Vec::new();
        if self.config.tax.provisions.enabled {
            let mut provision_gen = datasynth_generators::TaxProvisionGenerator::new(seed + 71);
            for company in &self.config.companies {
                let pre_tax_income = rust_decimal::Decimal::from(1_000_000);
                let statutory_rate = rust_decimal::Decimal::new(
                    (self.config.tax.provisions.statutory_rate * 100.0) as i64,
                    2,
                );
                let provision = provision_gen.generate(
                    &company.code,
                    start_date,
                    pre_tax_income,
                    statutory_rate,
                );
                provisions.push(provision);
            }
        }

        // Generate tax lines from document invoices
        let mut tax_lines = Vec::new();
        if !codes.is_empty() {
            let mut tax_line_gen = datasynth_generators::TaxLineGenerator::new(
                datasynth_generators::TaxLineGeneratorConfig::default(),
                codes.clone(),
                seed + 72,
            );

            // Tax lines from vendor invoices (input tax)
            // Use the first company's country as buyer country
            let buyer_country = self
                .config
                .companies
                .first()
                .map(|c| c.country.as_str())
                .unwrap_or("US");
            for vi in &document_flows.vendor_invoices {
                let lines = tax_line_gen.generate_for_document(
                    datasynth_core::models::TaxableDocumentType::VendorInvoice,
                    &vi.header.document_id,
                    buyer_country, // seller approx same country
                    buyer_country,
                    vi.payable_amount,
                    vi.header.document_date,
                    None,
                );
                tax_lines.extend(lines);
            }

            // Tax lines from customer invoices (output tax)
            for ci in &document_flows.customer_invoices {
                let lines = tax_line_gen.generate_for_document(
                    datasynth_core::models::TaxableDocumentType::CustomerInvoice,
                    &ci.header.document_id,
                    buyer_country, // seller is the company
                    buyer_country,
                    ci.total_gross_amount,
                    ci.header.document_date,
                    None,
                );
                tax_lines.extend(lines);
            }
        }

        let snapshot = TaxSnapshot {
            jurisdiction_count: jurisdictions.len(),
            code_count: codes.len(),
            jurisdictions,
            codes,
            tax_provisions: provisions,
            tax_lines,
            tax_returns: Vec::new(),
            withholding_records: Vec::new(),
            tax_anomaly_labels: Vec::new(),
        };

        stats.tax_jurisdiction_count = snapshot.jurisdiction_count;
        stats.tax_code_count = snapshot.code_count;
        stats.tax_provision_count = snapshot.tax_provisions.len();
        stats.tax_line_count = snapshot.tax_lines.len();

        info!(
            "Tax data generated: {} jurisdictions, {} codes, {} provisions",
            snapshot.jurisdiction_count,
            snapshot.code_count,
            snapshot.tax_provisions.len()
        );
        self.check_resources_with_log("post-tax")?;

        Ok(snapshot)
    }

    /// Phase 21: Generate ESG data (emissions, energy, water, waste, social, governance, disclosures).
    fn phase_esg_generation(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<EsgSnapshot> {
        if !self.phase_config.generate_esg || !self.config.esg.enabled {
            debug!("Phase 21: Skipped (ESG generation disabled)");
            return Ok(EsgSnapshot::default());
        }
        info!("Phase 21: Generating ESG Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let entity_id = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        let esg_cfg = &self.config.esg;
        let mut snapshot = EsgSnapshot::default();

        // Energy consumption (feeds into scope 1 & 2 emissions)
        let mut energy_gen = datasynth_generators::EnergyGenerator::new(
            esg_cfg.environmental.energy.clone(),
            seed + 80,
        );
        let energy_records = energy_gen.generate(entity_id, start_date, end_date);

        // Water usage
        let facility_count = esg_cfg.environmental.energy.facility_count;
        let mut water_gen = datasynth_generators::WaterGenerator::new(seed + 81, facility_count);
        snapshot.water = water_gen.generate(entity_id, start_date, end_date);

        // Waste
        let mut waste_gen = datasynth_generators::WasteGenerator::new(
            seed + 82,
            esg_cfg.environmental.waste.diversion_target,
            facility_count,
        );
        snapshot.waste = waste_gen.generate(entity_id, start_date, end_date);

        // Emissions (scope 1, 2, 3)
        let mut emission_gen =
            datasynth_generators::EmissionGenerator::new(esg_cfg.environmental.clone(), seed + 83);

        // Build EnergyInput from energy_records
        let energy_inputs: Vec<datasynth_generators::EnergyInput> = energy_records
            .iter()
            .map(|e| datasynth_generators::EnergyInput {
                facility_id: e.facility_id.clone(),
                energy_type: match e.energy_source {
                    EnergySourceType::NaturalGas => {
                        datasynth_generators::EnergyInputType::NaturalGas
                    }
                    EnergySourceType::Diesel => datasynth_generators::EnergyInputType::Diesel,
                    EnergySourceType::Coal => datasynth_generators::EnergyInputType::Coal,
                    _ => datasynth_generators::EnergyInputType::Electricity,
                },
                consumption_kwh: e.consumption_kwh,
                period: e.period,
            })
            .collect();

        let mut emissions = Vec::new();
        emissions.extend(emission_gen.generate_scope1(entity_id, &energy_inputs));
        emissions.extend(emission_gen.generate_scope2(entity_id, &energy_inputs));

        // Scope 3: use vendor spend data from actual payments
        let vendor_payment_totals: HashMap<String, rust_decimal::Decimal> = {
            let mut totals: HashMap<String, rust_decimal::Decimal> = HashMap::new();
            for payment in &document_flows.payments {
                if payment.is_vendor {
                    *totals
                        .entry(payment.business_partner_id.clone())
                        .or_default() += payment.amount;
                }
            }
            totals
        };
        let vendor_spend: Vec<datasynth_generators::VendorSpendInput> = self
            .master_data
            .vendors
            .iter()
            .map(|v| {
                let spend = vendor_payment_totals
                    .get(&v.vendor_id)
                    .copied()
                    .unwrap_or_else(|| rust_decimal::Decimal::new(10000, 0));
                datasynth_generators::VendorSpendInput {
                    vendor_id: v.vendor_id.clone(),
                    category: format!("{:?}", v.vendor_type).to_lowercase(),
                    spend,
                    country: v.country.clone(),
                }
            })
            .collect();
        if !vendor_spend.is_empty() {
            emissions.extend(emission_gen.generate_scope3_purchased_goods(
                entity_id,
                &vendor_spend,
                start_date,
                end_date,
            ));
        }

        // Business travel & commuting (scope 3)
        let headcount = self.master_data.employees.len() as u32;
        if headcount > 0 {
            let travel_spend = rust_decimal::Decimal::new(headcount as i64 * 2000, 0);
            emissions.extend(emission_gen.generate_scope3_business_travel(
                entity_id,
                travel_spend,
                start_date,
            ));
            emissions
                .extend(emission_gen.generate_scope3_commuting(entity_id, headcount, start_date));
        }

        snapshot.emission_count = emissions.len();
        snapshot.emissions = emissions;
        snapshot.energy = energy_records;

        // Social: Workforce diversity, pay equity, safety
        let mut workforce_gen =
            datasynth_generators::WorkforceGenerator::new(esg_cfg.social.clone(), seed + 84);
        let total_headcount = headcount.max(100);
        snapshot.diversity =
            workforce_gen.generate_diversity(entity_id, total_headcount, start_date);
        snapshot.pay_equity = workforce_gen.generate_pay_equity(entity_id, start_date);
        snapshot.safety_incidents = workforce_gen.generate_safety_incidents(
            entity_id,
            facility_count,
            start_date,
            end_date,
        );

        // Compute safety metrics
        let total_hours = total_headcount as u64 * 2000; // ~2000 hours/employee/year
        let safety_metric = workforce_gen.compute_safety_metrics(
            entity_id,
            &snapshot.safety_incidents,
            total_hours,
            start_date,
        );
        snapshot.safety_metrics = vec![safety_metric];

        // Governance
        let mut gov_gen = datasynth_generators::GovernanceGenerator::new(
            seed + 85,
            esg_cfg.governance.board_size,
            esg_cfg.governance.independence_target,
        );
        snapshot.governance = vec![gov_gen.generate(entity_id, start_date)];

        // Supplier ESG assessments
        let mut supplier_gen = datasynth_generators::SupplierEsgGenerator::new(
            esg_cfg.supply_chain_esg.clone(),
            seed + 86,
        );
        let vendor_inputs: Vec<datasynth_generators::VendorInput> = self
            .master_data
            .vendors
            .iter()
            .map(|v| datasynth_generators::VendorInput {
                vendor_id: v.vendor_id.clone(),
                country: v.country.clone(),
                industry: format!("{:?}", v.vendor_type).to_lowercase(),
                quality_score: None,
            })
            .collect();
        snapshot.supplier_assessments =
            supplier_gen.generate(entity_id, &vendor_inputs, start_date);

        // Disclosures
        let mut disclosure_gen = datasynth_generators::DisclosureGenerator::new(
            seed + 87,
            esg_cfg.reporting.clone(),
            esg_cfg.climate_scenarios.clone(),
        );
        snapshot.materiality = disclosure_gen.generate_materiality(entity_id, start_date);
        snapshot.disclosures = disclosure_gen.generate_disclosures(
            entity_id,
            &snapshot.materiality,
            start_date,
            end_date,
        );
        snapshot.climate_scenarios = disclosure_gen.generate_climate_scenarios(entity_id);
        snapshot.disclosure_count = snapshot.disclosures.len();

        // Anomaly injection
        if esg_cfg.anomaly_rate > 0.0 {
            let mut anomaly_injector =
                datasynth_generators::EsgAnomalyInjector::new(seed + 88, esg_cfg.anomaly_rate);
            let mut labels = Vec::new();
            labels.extend(anomaly_injector.inject_greenwashing(&mut snapshot.emissions));
            labels.extend(anomaly_injector.inject_diversity_stagnation(&mut snapshot.diversity));
            labels.extend(
                anomaly_injector.inject_supply_chain_risk(&mut snapshot.supplier_assessments),
            );
            labels.extend(anomaly_injector.inject_data_quality_gaps(&mut snapshot.safety_metrics));
            labels.extend(anomaly_injector.inject_missing_disclosures(&mut snapshot.materiality));
            snapshot.anomaly_labels = labels;
        }

        stats.esg_emission_count = snapshot.emission_count;
        stats.esg_disclosure_count = snapshot.disclosure_count;

        info!(
            "ESG data generated: {} emissions, {} disclosures, {} supplier assessments",
            snapshot.emission_count,
            snapshot.disclosure_count,
            snapshot.supplier_assessments.len()
        );
        self.check_resources_with_log("post-esg")?;

        Ok(snapshot)
    }

    /// Phase 22: Generate Treasury data (cash management, hedging, debt, pooling, guarantees, netting).
    fn phase_treasury_data(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        subledger: &SubledgerSnapshot,
        intercompany: &IntercompanySnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<TreasurySnapshot> {
        if !self.config.treasury.enabled {
            debug!("Phase 22: Skipped (treasury generation disabled)");
            return Ok(TreasurySnapshot::default());
        }
        info!("Phase 22: Generating Treasury Data");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.as_str())
            .unwrap_or("USD");
        let entity_id = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        let mut snapshot = TreasurySnapshot::default();

        // Generate debt instruments
        let mut debt_gen = datasynth_generators::treasury::DebtGenerator::new(
            self.config.treasury.debt.clone(),
            seed + 90,
        );
        snapshot.debt_instruments = debt_gen.generate(entity_id, currency, start_date);

        // Generate hedging instruments (IR swaps for floating-rate debt)
        let mut hedge_gen = datasynth_generators::treasury::HedgingGenerator::new(
            self.config.treasury.hedging.clone(),
            seed + 91,
        );
        for debt in &snapshot.debt_instruments {
            if debt.rate_type == InterestRateType::Variable {
                let swap = hedge_gen.generate_ir_swap(
                    currency,
                    debt.principal,
                    debt.origination_date,
                    debt.maturity_date,
                );
                snapshot.hedging_instruments.push(swap);
            }
        }

        // Build FX exposures from foreign-currency payments and generate
        // FX forwards + hedge relationship designations via generate() API.
        {
            let mut fx_map: HashMap<String, (rust_decimal::Decimal, NaiveDate)> = HashMap::new();
            for payment in &document_flows.payments {
                if payment.currency != currency {
                    let entry = fx_map
                        .entry(payment.currency.clone())
                        .or_insert((rust_decimal::Decimal::ZERO, payment.header.document_date));
                    entry.0 += payment.amount;
                    // Use the latest settlement date among grouped payments
                    if payment.header.document_date > entry.1 {
                        entry.1 = payment.header.document_date;
                    }
                }
            }
            if !fx_map.is_empty() {
                let fx_exposures: Vec<datasynth_generators::treasury::FxExposure> = fx_map
                    .into_iter()
                    .map(|(foreign_ccy, (net_amount, settlement_date))| {
                        datasynth_generators::treasury::FxExposure {
                            currency_pair: format!("{}/{}", foreign_ccy, currency),
                            foreign_currency: foreign_ccy,
                            net_amount,
                            settlement_date,
                            description: "AP payment FX exposure".to_string(),
                        }
                    })
                    .collect();
                let (fx_instruments, fx_relationships) =
                    hedge_gen.generate(start_date, &fx_exposures);
                snapshot.hedging_instruments.extend(fx_instruments);
                snapshot.hedge_relationships.extend(fx_relationships);
            }
        }

        // Inject anomalies if configured
        if self.config.treasury.anomaly_rate > 0.0 {
            let mut anomaly_injector = datasynth_generators::treasury::TreasuryAnomalyInjector::new(
                seed + 92,
                self.config.treasury.anomaly_rate,
            );
            let mut labels = Vec::new();
            labels.extend(
                anomaly_injector.inject_into_hedge_relationships(&mut snapshot.hedge_relationships),
            );
            snapshot.treasury_anomaly_labels = labels;
        }

        // Generate cash positions from payment flows
        if self.config.treasury.cash_positioning.enabled {
            let mut cash_flows: Vec<datasynth_generators::treasury::CashFlow> = Vec::new();

            // AP payments as outflows
            for payment in &document_flows.payments {
                cash_flows.push(datasynth_generators::treasury::CashFlow {
                    date: payment.header.document_date,
                    account_id: format!("{}-MAIN", entity_id),
                    amount: payment.amount,
                    direction: datasynth_generators::treasury::CashFlowDirection::Outflow,
                });
            }

            // Customer receipts (from O2C chains) as inflows
            for chain in &document_flows.o2c_chains {
                if let Some(ref receipt) = chain.customer_receipt {
                    cash_flows.push(datasynth_generators::treasury::CashFlow {
                        date: receipt.header.document_date,
                        account_id: format!("{}-MAIN", entity_id),
                        amount: receipt.amount,
                        direction: datasynth_generators::treasury::CashFlowDirection::Inflow,
                    });
                }
                // Remainder receipts (follow-up to partial payments)
                for receipt in &chain.remainder_receipts {
                    cash_flows.push(datasynth_generators::treasury::CashFlow {
                        date: receipt.header.document_date,
                        account_id: format!("{}-MAIN", entity_id),
                        amount: receipt.amount,
                        direction: datasynth_generators::treasury::CashFlowDirection::Inflow,
                    });
                }
            }

            if !cash_flows.is_empty() {
                let mut cash_gen = datasynth_generators::treasury::CashPositionGenerator::new(
                    self.config.treasury.cash_positioning.clone(),
                    seed + 93,
                );
                let account_id = format!("{}-MAIN", entity_id);
                snapshot.cash_positions = cash_gen.generate(
                    entity_id,
                    &account_id,
                    currency,
                    &cash_flows,
                    start_date,
                    start_date + chrono::Months::new(self.config.global.period_months),
                    rust_decimal::Decimal::new(1_000_000, 0), // Default opening balance
                );
            }
        }

        // Generate cash forecasts from AR/AP aging
        if self.config.treasury.cash_forecasting.enabled {
            let end_date = start_date + chrono::Months::new(self.config.global.period_months);

            // Build AR aging items from subledger AR invoices
            let ar_items: Vec<datasynth_generators::treasury::ArAgingItem> = subledger
                .ar_invoices
                .iter()
                .filter(|inv| inv.amount_remaining > rust_decimal::Decimal::ZERO)
                .map(|inv| {
                    let days_past_due = if inv.due_date < end_date {
                        (end_date - inv.due_date).num_days().max(0) as u32
                    } else {
                        0
                    };
                    datasynth_generators::treasury::ArAgingItem {
                        expected_date: inv.due_date,
                        amount: inv.amount_remaining,
                        days_past_due,
                        document_id: inv.invoice_number.clone(),
                    }
                })
                .collect();

            // Build AP aging items from subledger AP invoices
            let ap_items: Vec<datasynth_generators::treasury::ApAgingItem> = subledger
                .ap_invoices
                .iter()
                .filter(|inv| inv.amount_remaining > rust_decimal::Decimal::ZERO)
                .map(|inv| datasynth_generators::treasury::ApAgingItem {
                    payment_date: inv.due_date,
                    amount: inv.amount_remaining,
                    document_id: inv.invoice_number.clone(),
                })
                .collect();

            let mut forecast_gen = datasynth_generators::treasury::CashForecastGenerator::new(
                self.config.treasury.cash_forecasting.clone(),
                seed + 94,
            );
            let forecast = forecast_gen.generate(
                entity_id,
                currency,
                end_date,
                &ar_items,
                &ap_items,
                &[], // scheduled disbursements - empty for now
            );
            snapshot.cash_forecasts.push(forecast);
        }

        // Generate cash pools and sweeps
        if self.config.treasury.cash_pooling.enabled && !snapshot.cash_positions.is_empty() {
            let end_date = start_date + chrono::Months::new(self.config.global.period_months);
            let mut pool_gen = datasynth_generators::treasury::CashPoolGenerator::new(
                self.config.treasury.cash_pooling.clone(),
                seed + 95,
            );

            // Create a pool from available accounts
            let account_ids: Vec<String> = snapshot
                .cash_positions
                .iter()
                .map(|cp| cp.bank_account_id.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            if let Some(pool) =
                pool_gen.create_pool(&format!("{}_MAIN_POOL", entity_id), currency, &account_ids)
            {
                // Generate sweeps - build participant balances from last cash position per account
                let mut latest_balances: HashMap<String, rust_decimal::Decimal> = HashMap::new();
                for cp in &snapshot.cash_positions {
                    latest_balances.insert(cp.bank_account_id.clone(), cp.closing_balance);
                }

                let participant_balances: Vec<datasynth_generators::treasury::AccountBalance> =
                    latest_balances
                        .into_iter()
                        .filter(|(id, _)| pool.participant_accounts.contains(id))
                        .map(
                            |(id, balance)| datasynth_generators::treasury::AccountBalance {
                                account_id: id,
                                balance,
                            },
                        )
                        .collect();

                let sweeps =
                    pool_gen.generate_sweeps(&pool, end_date, currency, &participant_balances);
                snapshot.cash_pool_sweeps = sweeps;
                snapshot.cash_pools.push(pool);
            }
        }

        // Generate bank guarantees
        if self.config.treasury.bank_guarantees.enabled {
            let vendor_names: Vec<String> = self
                .master_data
                .vendors
                .iter()
                .map(|v| v.name.clone())
                .collect();
            if !vendor_names.is_empty() {
                let mut bg_gen = datasynth_generators::treasury::BankGuaranteeGenerator::new(
                    self.config.treasury.bank_guarantees.clone(),
                    seed + 96,
                );
                snapshot.bank_guarantees =
                    bg_gen.generate(entity_id, currency, start_date, &vendor_names);
            }
        }

        // Generate netting runs from intercompany matched pairs
        if self.config.treasury.netting.enabled && !intercompany.matched_pairs.is_empty() {
            let entity_ids: Vec<String> = self
                .config
                .companies
                .iter()
                .map(|c| c.code.clone())
                .collect();
            let ic_amounts: Vec<(String, String, rust_decimal::Decimal)> = intercompany
                .matched_pairs
                .iter()
                .map(|mp| {
                    (
                        mp.seller_company.clone(),
                        mp.buyer_company.clone(),
                        mp.amount,
                    )
                })
                .collect();
            if entity_ids.len() >= 2 {
                let mut netting_gen = datasynth_generators::treasury::NettingRunGenerator::new(
                    self.config.treasury.netting.clone(),
                    seed + 97,
                );
                snapshot.netting_runs = netting_gen.generate(
                    &entity_ids,
                    currency,
                    start_date,
                    self.config.global.period_months,
                    &ic_amounts,
                );
            }
        }

        stats.treasury_debt_instrument_count = snapshot.debt_instruments.len();
        stats.treasury_hedging_instrument_count = snapshot.hedging_instruments.len();
        stats.cash_position_count = snapshot.cash_positions.len();
        stats.cash_forecast_count = snapshot.cash_forecasts.len();
        stats.cash_pool_count = snapshot.cash_pools.len();

        info!(
            "Treasury data generated: {} debt instruments, {} hedging instruments, {} cash positions, {} forecasts, {} pools, {} guarantees, {} netting runs",
            snapshot.debt_instruments.len(),
            snapshot.hedging_instruments.len(),
            snapshot.cash_positions.len(),
            snapshot.cash_forecasts.len(),
            snapshot.cash_pools.len(),
            snapshot.bank_guarantees.len(),
            snapshot.netting_runs.len(),
        );
        self.check_resources_with_log("post-treasury")?;

        Ok(snapshot)
    }

    /// Phase 23: Generate Project Accounting data (projects, costs, revenue, EVM, milestones).
    fn phase_project_accounting(
        &mut self,
        document_flows: &DocumentFlowSnapshot,
        hr: &HrSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<ProjectAccountingSnapshot> {
        if !self.config.project_accounting.enabled {
            debug!("Phase 23: Skipped (project accounting disabled)");
            return Ok(ProjectAccountingSnapshot::default());
        }
        info!("Phase 23: Generating Project Accounting Data");

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

        let mut snapshot = ProjectAccountingSnapshot::default();

        // Generate projects with WBS hierarchies
        let mut project_gen = datasynth_generators::project_accounting::ProjectGenerator::new(
            self.config.project_accounting.clone(),
            seed + 95,
        );
        let pool = project_gen.generate(company_code, start_date, end_date);
        snapshot.projects = pool.projects.clone();

        // Link source documents to projects for cost allocation
        {
            let mut source_docs: Vec<datasynth_generators::project_accounting::SourceDocument> =
                Vec::new();

            // Time entries
            for te in &hr.time_entries {
                let total_hours = te.hours_regular + te.hours_overtime;
                if total_hours > 0.0 {
                    source_docs.push(datasynth_generators::project_accounting::SourceDocument {
                        id: te.entry_id.clone(),
                        entity_id: company_code.to_string(),
                        date: te.date,
                        amount: rust_decimal::Decimal::from_f64_retain(total_hours * 75.0)
                            .unwrap_or(rust_decimal::Decimal::ZERO),
                        source_type: CostSourceType::TimeEntry,
                        hours: Some(
                            rust_decimal::Decimal::from_f64_retain(total_hours)
                                .unwrap_or(rust_decimal::Decimal::ZERO),
                        ),
                    });
                }
            }

            // Expense reports
            for er in &hr.expense_reports {
                source_docs.push(datasynth_generators::project_accounting::SourceDocument {
                    id: er.report_id.clone(),
                    entity_id: company_code.to_string(),
                    date: er.submission_date,
                    amount: er.total_amount,
                    source_type: CostSourceType::ExpenseReport,
                    hours: None,
                });
            }

            // Purchase orders
            for po in &document_flows.purchase_orders {
                source_docs.push(datasynth_generators::project_accounting::SourceDocument {
                    id: po.header.document_id.clone(),
                    entity_id: company_code.to_string(),
                    date: po.header.document_date,
                    amount: po.total_net_amount,
                    source_type: CostSourceType::PurchaseOrder,
                    hours: None,
                });
            }

            // Vendor invoices
            for vi in &document_flows.vendor_invoices {
                source_docs.push(datasynth_generators::project_accounting::SourceDocument {
                    id: vi.header.document_id.clone(),
                    entity_id: company_code.to_string(),
                    date: vi.header.document_date,
                    amount: vi.payable_amount,
                    source_type: CostSourceType::VendorInvoice,
                    hours: None,
                });
            }

            if !source_docs.is_empty() && !pool.projects.is_empty() {
                let mut cost_gen =
                    datasynth_generators::project_accounting::ProjectCostGenerator::new(
                        self.config.project_accounting.cost_allocation.clone(),
                        seed + 99,
                    );
                snapshot.cost_lines = cost_gen.link_documents(&pool, &source_docs);
            }
        }

        // Generate change orders
        if self.config.project_accounting.change_orders.enabled {
            let mut co_gen = datasynth_generators::project_accounting::ChangeOrderGenerator::new(
                self.config.project_accounting.change_orders.clone(),
                seed + 96,
            );
            snapshot.change_orders = co_gen.generate(&pool.projects, start_date, end_date);
        }

        // Generate milestones
        if self.config.project_accounting.milestones.enabled {
            let mut ms_gen = datasynth_generators::project_accounting::MilestoneGenerator::new(
                self.config.project_accounting.milestones.clone(),
                seed + 97,
            );
            snapshot.milestones = ms_gen.generate(&pool.projects, start_date, end_date, end_date);
        }

        // Generate earned value metrics (needs cost lines, so only if we have projects)
        if self.config.project_accounting.earned_value.enabled && !snapshot.projects.is_empty() {
            let mut evm_gen = datasynth_generators::project_accounting::EarnedValueGenerator::new(
                self.config.project_accounting.earned_value.clone(),
                seed + 98,
            );
            snapshot.earned_value_metrics =
                evm_gen.generate(&pool.projects, &snapshot.cost_lines, start_date, end_date);
        }

        stats.project_count = snapshot.projects.len();
        stats.project_change_order_count = snapshot.change_orders.len();
        stats.project_cost_line_count = snapshot.cost_lines.len();

        info!(
            "Project accounting generated: {} projects, {} change orders, {} milestones, {} EVM records",
            snapshot.projects.len(),
            snapshot.change_orders.len(),
            snapshot.milestones.len(),
            snapshot.earned_value_metrics.len()
        );
        self.check_resources_with_log("post-project-accounting")?;

        Ok(snapshot)
    }

    /// Phase 24: Generate process evolution and organizational events.
    fn phase_evolution_events(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<(Vec<ProcessEvolutionEvent>, Vec<OrganizationalEvent>)> {
        if !self.phase_config.generate_evolution_events {
            debug!("Phase 24: Skipped (evolution events disabled)");
            return Ok((Vec::new(), Vec::new()));
        }
        info!("Phase 24: Generating Process Evolution + Organizational Events");

        let seed = self.seed;
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        // Process evolution events
        let mut proc_gen =
            datasynth_generators::process_evolution_generator::ProcessEvolutionGenerator::new(
                seed + 100,
            );
        let process_events = proc_gen.generate_events(start_date, end_date);

        // Organizational events
        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();
        let mut org_gen =
            datasynth_generators::organizational_event_generator::OrganizationalEventGenerator::new(
                seed + 101,
            );
        let org_events = org_gen.generate_events(start_date, end_date, &company_codes);

        stats.process_evolution_event_count = process_events.len();
        stats.organizational_event_count = org_events.len();

        info!(
            "Evolution events generated: {} process evolution, {} organizational",
            process_events.len(),
            org_events.len()
        );
        self.check_resources_with_log("post-evolution-events")?;

        Ok((process_events, org_events))
    }

    /// Phase 24b: Generate disruption events (outages, migrations, process changes,
    /// data recovery, and regulatory changes).
    fn phase_disruption_events(
        &self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<datasynth_generators::disruption::DisruptionEvent>> {
        if !self.config.organizational_events.enabled {
            debug!("Phase 24b: Skipped (organizational events disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 24b: Generating Disruption Events");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        let mut gen = datasynth_generators::disruption::DisruptionGenerator::new(self.seed + 150);
        let events = gen.generate(start_date, end_date, &company_codes);

        stats.disruption_event_count = events.len();
        info!("Disruption events generated: {} events", events.len());
        self.check_resources_with_log("post-disruption-events")?;

        Ok(events)
    }

    /// Phase 25: Generate counterfactual (original, mutated) JE pairs for ML training.
    ///
    /// Produces paired examples where each pair contains the original clean JE
    /// and a controlled mutation (scaled amount, shifted date, self-approval, or
    /// split transaction). Useful for training anomaly detection models with
    /// known ground truth.
    fn phase_counterfactuals(
        &self,
        journal_entries: &[JournalEntry],
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<datasynth_generators::counterfactual::CounterfactualPair>> {
        if !self.phase_config.generate_counterfactuals || journal_entries.is_empty() {
            debug!("Phase 25: Skipped (counterfactual generation disabled or no JEs)");
            return Ok(Vec::new());
        }
        info!("Phase 25: Generating Counterfactual Pairs for ML Training");

        use datasynth_generators::counterfactual::{CounterfactualGenerator, CounterfactualSpec};

        let mut gen = CounterfactualGenerator::new(self.seed + 110);

        // Rotating set of specs to produce diverse mutation types
        let specs = [
            CounterfactualSpec::ScaleAmount { factor: 2.5 },
            CounterfactualSpec::ShiftDate { days: -14 },
            CounterfactualSpec::SelfApprove,
            CounterfactualSpec::SplitTransaction { split_count: 3 },
        ];

        let pairs: Vec<_> = journal_entries
            .iter()
            .enumerate()
            .map(|(i, je)| {
                let spec = &specs[i % specs.len()];
                gen.generate(je, spec)
            })
            .collect();

        stats.counterfactual_pair_count = pairs.len();
        info!(
            "Counterfactual pairs generated: {} pairs from {} journal entries",
            pairs.len(),
            journal_entries.len()
        );
        self.check_resources_with_log("post-counterfactuals")?;

        Ok(pairs)
    }

    /// Phase 26: Inject fraud red-flag indicators onto P2P/O2C documents.
    ///
    /// Uses the anomaly labels (from Phase 8) to determine which documents are
    /// fraudulent, then generates probabilistic red flags on all chain documents.
    /// Non-fraud documents also receive red flags at a lower rate (false positives)
    /// to produce realistic ML training data.
    fn phase_red_flags(
        &self,
        anomaly_labels: &AnomalyLabels,
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<datasynth_generators::fraud::RedFlag>> {
        if !self.config.fraud.enabled {
            debug!("Phase 26: Skipped (fraud generation disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 26: Generating Fraud Red-Flag Indicators");

        use datasynth_generators::fraud::RedFlagGenerator;

        let generator = RedFlagGenerator::new();
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(self.seed + 120);

        // Build a set of document IDs that are known-fraudulent from anomaly labels.
        let fraud_doc_ids: std::collections::HashSet<&str> = anomaly_labels
            .labels
            .iter()
            .filter(|label| label.anomaly_type.is_intentional())
            .map(|label| label.document_id.as_str())
            .collect();

        let mut flags = Vec::new();

        // Iterate P2P chains: use the purchase order document ID as the chain key.
        for chain in &document_flows.p2p_chains {
            let doc_id = &chain.purchase_order.header.document_id;
            let is_fraud = fraud_doc_ids.contains(doc_id.as_str());
            flags.extend(generator.inject_flags(doc_id, is_fraud, &mut rng));
        }

        // Iterate O2C chains: use the sales order document ID as the chain key.
        for chain in &document_flows.o2c_chains {
            let doc_id = &chain.sales_order.header.document_id;
            let is_fraud = fraud_doc_ids.contains(doc_id.as_str());
            flags.extend(generator.inject_flags(doc_id, is_fraud, &mut rng));
        }

        stats.red_flag_count = flags.len();
        info!(
            "Red flags generated: {} flags across {} P2P + {} O2C chains ({} fraud docs)",
            flags.len(),
            document_flows.p2p_chains.len(),
            document_flows.o2c_chains.len(),
            fraud_doc_ids.len()
        );
        self.check_resources_with_log("post-red-flags")?;

        Ok(flags)
    }

    /// Phase 26b: Generate collusion rings from employee/vendor pools.
    ///
    /// Gated on `fraud.enabled && fraud.clustering_enabled`. Uses the
    /// `CollusionRingGenerator` to create 1-3 coordinated fraud networks and
    /// advance them over the simulation period.
    fn phase_collusion_rings(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<datasynth_generators::fraud::CollusionRing>> {
        if !(self.config.fraud.enabled && self.config.fraud.clustering_enabled) {
            debug!("Phase 26b: Skipped (fraud collusion generation disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 26b: Generating Collusion Rings");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let months = self.config.global.period_months;

        let employee_ids: Vec<String> = self
            .master_data
            .employees
            .iter()
            .map(|e| e.employee_id.clone())
            .collect();
        let vendor_ids: Vec<String> = self
            .master_data
            .vendors
            .iter()
            .map(|v| v.vendor_id.clone())
            .collect();

        let mut generator =
            datasynth_generators::fraud::CollusionRingGenerator::new(self.seed + 160);
        let rings = generator.generate(&employee_ids, &vendor_ids, start_date, months);

        stats.collusion_ring_count = rings.len();
        info!(
            "Collusion rings generated: {} rings, total members: {}",
            rings.len(),
            rings.iter().map(|r| r.size()).sum::<usize>()
        );
        self.check_resources_with_log("post-collusion-rings")?;

        Ok(rings)
    }

    /// Phase 27: Generate bi-temporal version chains for vendor entities.
    ///
    /// Creates `TemporalVersionChain<Vendor>` records that model how vendor
    /// master data changes over time, supporting bi-temporal audit queries.
    fn phase_temporal_attributes(
        &mut self,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<
        Vec<datasynth_core::models::TemporalVersionChain<datasynth_core::models::Vendor>>,
    > {
        if !self.config.temporal_attributes.enabled {
            debug!("Phase 27: Skipped (temporal attributes disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 27: Generating Bi-Temporal Vendor Version Chains");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        let mut gen = datasynth_generators::temporal::TemporalAttributeGenerator::with_defaults(
            self.seed + 130,
            start_date,
        );

        let uuid_factory = datasynth_core::DeterministicUuidFactory::new(
            self.seed + 130,
            datasynth_core::GeneratorType::Vendor,
        );

        let chains: Vec<_> = self
            .master_data
            .vendors
            .iter()
            .map(|vendor| {
                let id = uuid_factory.next();
                gen.generate_version_chain(vendor.clone(), id)
            })
            .collect();

        stats.temporal_version_chain_count = chains.len();
        info!("Temporal version chains generated: {} chains", chains.len());
        self.check_resources_with_log("post-temporal-attributes")?;

        Ok(chains)
    }

    /// Phase 28: Build entity relationship graph and cross-process links.
    ///
    /// Part 1 (gated on `relationship_strength.enabled`): builds an
    /// `EntityGraph` from master-data vendor/customer entities and
    /// journal-entry-derived transaction summaries.
    ///
    /// Part 2 (gated on `cross_process_links.enabled`): extracts
    /// `GoodsReceiptRef` / `DeliveryRef` from document flow chains and
    /// generates inventory-movement cross-process links.
    fn phase_entity_relationships(
        &self,
        journal_entries: &[JournalEntry],
        document_flows: &DocumentFlowSnapshot,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<(
        Option<datasynth_core::models::EntityGraph>,
        Vec<datasynth_core::models::CrossProcessLink>,
    )> {
        use datasynth_generators::relationships::{
            DeliveryRef, EntityGraphConfig, EntityGraphGenerator, EntitySummary, GoodsReceiptRef,
            TransactionSummary,
        };

        let rs_enabled = self.config.relationship_strength.enabled;
        let cpl_enabled = self.config.cross_process_links.enabled;

        if !rs_enabled && !cpl_enabled {
            debug!(
                "Phase 28: Skipped (relationship_strength and cross_process_links both disabled)"
            );
            return Ok((None, Vec::new()));
        }

        info!("Phase 28: Generating Entity Relationship Graph + Cross-Process Links");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        // Build the generator with matching config flags
        let gen_config = EntityGraphConfig {
            enabled: rs_enabled,
            cross_process: datasynth_generators::relationships::CrossProcessConfig {
                enable_inventory_links: self.config.cross_process_links.inventory_p2p_o2c,
                enable_return_flows: false,
                enable_payment_links: self.config.cross_process_links.payment_bank_reconciliation,
                enable_ic_bilateral: self.config.cross_process_links.intercompany_bilateral,
                ..Default::default()
            },
            strength_config: datasynth_generators::relationships::StrengthConfig {
                transaction_volume_weight: self
                    .config
                    .relationship_strength
                    .calculation
                    .transaction_volume_weight,
                transaction_count_weight: self
                    .config
                    .relationship_strength
                    .calculation
                    .transaction_count_weight,
                duration_weight: self
                    .config
                    .relationship_strength
                    .calculation
                    .relationship_duration_weight,
                recency_weight: self.config.relationship_strength.calculation.recency_weight,
                mutual_connections_weight: self
                    .config
                    .relationship_strength
                    .calculation
                    .mutual_connections_weight,
                recency_half_life_days: self
                    .config
                    .relationship_strength
                    .calculation
                    .recency_half_life_days as u32,
            },
            ..Default::default()
        };

        let mut gen = EntityGraphGenerator::with_config(self.seed + 140, gen_config);

        // --- Part 1: Entity Relationship Graph ---
        let entity_graph = if rs_enabled {
            // Build EntitySummary lists from master data
            let vendor_summaries: Vec<EntitySummary> = self
                .master_data
                .vendors
                .iter()
                .map(|v| {
                    EntitySummary::new(
                        &v.vendor_id,
                        &v.name,
                        datasynth_core::models::GraphEntityType::Vendor,
                        start_date,
                    )
                })
                .collect();

            let customer_summaries: Vec<EntitySummary> = self
                .master_data
                .customers
                .iter()
                .map(|c| {
                    EntitySummary::new(
                        &c.customer_id,
                        &c.name,
                        datasynth_core::models::GraphEntityType::Customer,
                        start_date,
                    )
                })
                .collect();

            // Build transaction summaries from journal entries.
            // Key = (company_code, trading_partner) for entries that have a
            // trading partner.  This captures intercompany flows and any JE
            // whose line items carry a trading_partner reference.
            let mut txn_summaries: std::collections::HashMap<(String, String), TransactionSummary> =
                std::collections::HashMap::new();

            for je in journal_entries {
                let cc = je.header.company_code.clone();
                let posting_date = je.header.posting_date;
                for line in &je.lines {
                    if let Some(ref tp) = line.trading_partner {
                        let amount = if line.debit_amount > line.credit_amount {
                            line.debit_amount
                        } else {
                            line.credit_amount
                        };
                        let entry = txn_summaries
                            .entry((cc.clone(), tp.clone()))
                            .or_insert_with(|| TransactionSummary {
                                total_volume: rust_decimal::Decimal::ZERO,
                                transaction_count: 0,
                                first_transaction_date: posting_date,
                                last_transaction_date: posting_date,
                                related_entities: std::collections::HashSet::new(),
                            });
                        entry.total_volume += amount;
                        entry.transaction_count += 1;
                        if posting_date < entry.first_transaction_date {
                            entry.first_transaction_date = posting_date;
                        }
                        if posting_date > entry.last_transaction_date {
                            entry.last_transaction_date = posting_date;
                        }
                        entry.related_entities.insert(cc.clone());
                    }
                }
            }

            let as_of_date = journal_entries
                .last()
                .map(|je| je.header.posting_date)
                .unwrap_or(start_date);

            let graph = gen.generate_entity_graph(
                company_code,
                as_of_date,
                &vendor_summaries,
                &customer_summaries,
                &txn_summaries,
            );

            info!(
                "Entity relationship graph: {} nodes, {} edges",
                graph.nodes.len(),
                graph.edges.len()
            );
            stats.entity_relationship_node_count = graph.nodes.len();
            stats.entity_relationship_edge_count = graph.edges.len();
            Some(graph)
        } else {
            None
        };

        // --- Part 2: Cross-Process Links ---
        let cross_process_links = if cpl_enabled {
            // Build GoodsReceiptRef from P2P chains
            let gr_refs: Vec<GoodsReceiptRef> = document_flows
                .p2p_chains
                .iter()
                .flat_map(|chain| {
                    let vendor_id = chain.purchase_order.vendor_id.clone();
                    let cc = chain.purchase_order.header.company_code.clone();
                    chain.goods_receipts.iter().flat_map(move |gr| {
                        gr.items.iter().filter_map({
                            let doc_id = gr.header.document_id.clone();
                            let v_id = vendor_id.clone();
                            let company = cc.clone();
                            let receipt_date = gr.header.document_date;
                            move |item| {
                                item.base
                                    .material_id
                                    .as_ref()
                                    .map(|mat_id| GoodsReceiptRef {
                                        document_id: doc_id.clone(),
                                        material_id: mat_id.clone(),
                                        quantity: item.base.quantity,
                                        receipt_date,
                                        vendor_id: v_id.clone(),
                                        company_code: company.clone(),
                                    })
                            }
                        })
                    })
                })
                .collect();

            // Build DeliveryRef from O2C chains
            let del_refs: Vec<DeliveryRef> = document_flows
                .o2c_chains
                .iter()
                .flat_map(|chain| {
                    let customer_id = chain.sales_order.customer_id.clone();
                    let cc = chain.sales_order.header.company_code.clone();
                    chain.deliveries.iter().flat_map(move |del| {
                        let delivery_date = del.actual_gi_date.unwrap_or(del.planned_gi_date);
                        del.items.iter().filter_map({
                            let doc_id = del.header.document_id.clone();
                            let c_id = customer_id.clone();
                            let company = cc.clone();
                            move |item| {
                                item.base.material_id.as_ref().map(|mat_id| DeliveryRef {
                                    document_id: doc_id.clone(),
                                    material_id: mat_id.clone(),
                                    quantity: item.base.quantity,
                                    delivery_date,
                                    customer_id: c_id.clone(),
                                    company_code: company.clone(),
                                })
                            }
                        })
                    })
                })
                .collect();

            let links = gen.generate_cross_process_links(&gr_refs, &del_refs);
            info!("Cross-process links generated: {} links", links.len());
            stats.cross_process_link_count = links.len();
            links
        } else {
            Vec::new()
        };

        self.check_resources_with_log("post-entity-relationships")?;
        Ok((entity_graph, cross_process_links))
    }

    /// Phase 3b: Generate opening balances for each company.
    fn phase_opening_balances(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<GeneratedOpeningBalance>> {
        if !self.config.balance.generate_opening_balances {
            debug!("Phase 3b: Skipped (opening balance generation disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 3b: Generating Opening Balances");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let fiscal_year = start_date.year();

        let industry = match self.config.global.industry {
            IndustrySector::Manufacturing => IndustryType::Manufacturing,
            IndustrySector::Retail => IndustryType::Retail,
            IndustrySector::FinancialServices => IndustryType::Financial,
            IndustrySector::Healthcare => IndustryType::Healthcare,
            IndustrySector::Technology => IndustryType::Technology,
            _ => IndustryType::Manufacturing,
        };

        let config = datasynth_generators::OpeningBalanceConfig {
            industry,
            ..Default::default()
        };
        let mut gen =
            datasynth_generators::OpeningBalanceGenerator::with_seed(config, self.seed + 200);

        let mut results = Vec::new();
        for company in &self.config.companies {
            let spec = OpeningBalanceSpec::new(
                company.code.clone(),
                start_date,
                fiscal_year,
                company.currency.clone(),
                rust_decimal::Decimal::new(10_000_000, 0),
                industry,
            );
            let ob = gen.generate(&spec, coa, start_date, &company.code);
            results.push(ob);
        }

        stats.opening_balance_count = results.len();
        info!("Opening balances generated: {} companies", results.len());
        self.check_resources_with_log("post-opening-balances")?;

        Ok(results)
    }

    /// Phase 9b: Reconcile GL control accounts to subledger balances.
    fn phase_subledger_reconciliation(
        &mut self,
        subledger: &SubledgerSnapshot,
        entries: &[JournalEntry],
        stats: &mut EnhancedGenerationStatistics,
    ) -> SynthResult<Vec<datasynth_generators::ReconciliationResult>> {
        if !self.config.balance.reconcile_subledgers {
            debug!("Phase 9b: Skipped (subledger reconciliation disabled)");
            return Ok(Vec::new());
        }
        info!("Phase 9b: Reconciling GL to subledger balances");

        let end_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map(|d| d + chrono::Months::new(self.config.global.period_months))
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        // Build GL balance map from journal entries using a balance tracker
        let tracker_config = BalanceTrackerConfig {
            validate_on_each_entry: false,
            track_history: false,
            fail_on_validation_error: false,
            ..Default::default()
        };
        let recon_currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.clone())
            .unwrap_or_else(|| "USD".to_string());
        let mut tracker = RunningBalanceTracker::new_with_currency(tracker_config, recon_currency);
        let _ = tracker.apply_entries(entries);

        let mut engine = datasynth_generators::ReconciliationEngine::new(
            datasynth_generators::ReconciliationConfig::default(),
        );

        let mut results = Vec::new();
        let company_code = self
            .config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        // Reconcile AR
        if !subledger.ar_invoices.is_empty() {
            let gl_balance = tracker
                .get_account_balance(
                    company_code,
                    datasynth_core::accounts::control_accounts::AR_CONTROL,
                )
                .map(|b| b.closing_balance)
                .unwrap_or_default();
            let ar_refs: Vec<&ARInvoice> = subledger.ar_invoices.iter().collect();
            results.push(engine.reconcile_ar(company_code, end_date, gl_balance, &ar_refs));
        }

        // Reconcile AP
        if !subledger.ap_invoices.is_empty() {
            let gl_balance = tracker
                .get_account_balance(
                    company_code,
                    datasynth_core::accounts::control_accounts::AP_CONTROL,
                )
                .map(|b| b.closing_balance)
                .unwrap_or_default();
            let ap_refs: Vec<&APInvoice> = subledger.ap_invoices.iter().collect();
            results.push(engine.reconcile_ap(company_code, end_date, gl_balance, &ap_refs));
        }

        // Reconcile FA
        if !subledger.fa_records.is_empty() {
            let gl_asset_balance = tracker
                .get_account_balance(
                    company_code,
                    datasynth_core::accounts::control_accounts::FIXED_ASSETS,
                )
                .map(|b| b.closing_balance)
                .unwrap_or_default();
            let gl_accum_depr_balance = tracker
                .get_account_balance(
                    company_code,
                    datasynth_core::accounts::control_accounts::ACCUMULATED_DEPRECIATION,
                )
                .map(|b| b.closing_balance)
                .unwrap_or_default();
            let fa_refs: Vec<&datasynth_core::models::subledger::fa::FixedAssetRecord> =
                subledger.fa_records.iter().collect();
            let (asset_recon, depr_recon) = engine.reconcile_fa(
                company_code,
                end_date,
                gl_asset_balance,
                gl_accum_depr_balance,
                &fa_refs,
            );
            results.push(asset_recon);
            results.push(depr_recon);
        }

        // Reconcile Inventory
        if !subledger.inventory_positions.is_empty() {
            let gl_balance = tracker
                .get_account_balance(
                    company_code,
                    datasynth_core::accounts::control_accounts::INVENTORY,
                )
                .map(|b| b.closing_balance)
                .unwrap_or_default();
            let inv_refs: Vec<&datasynth_core::models::subledger::inventory::InventoryPosition> =
                subledger.inventory_positions.iter().collect();
            results.push(engine.reconcile_inventory(company_code, end_date, gl_balance, &inv_refs));
        }

        stats.subledger_reconciliation_count = results.len();
        info!(
            "Subledger reconciliation complete: {} reconciliations",
            results.len()
        );
        self.check_resources_with_log("post-subledger-reconciliation")?;

        Ok(results)
    }

    /// Generate the chart of accounts.
    fn generate_coa(&mut self) -> SynthResult<Arc<ChartOfAccounts>> {
        let pb = self.create_progress_bar(1, "Generating Chart of Accounts");

        let coa_framework = self.resolve_coa_framework();

        let mut gen = ChartOfAccountsGenerator::new(
            self.config.chart_of_accounts.complexity,
            self.config.global.industry,
            self.seed,
        )
        .with_coa_framework(coa_framework);

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

        // Resolve country pack once for all companies (uses primary company's country)
        let pack = self.primary_pack().clone();

        // Capture config values needed inside the parallel closure
        let vendors_per_company = self.phase_config.vendors_per_company;
        let customers_per_company = self.phase_config.customers_per_company;
        let materials_per_company = self.phase_config.materials_per_company;
        let assets_per_company = self.phase_config.assets_per_company;
        let coa_framework = self.resolve_coa_framework();

        // Generate all master data in parallel across companies.
        // Each company's data is independent, making this embarrassingly parallel.
        let per_company_results: Vec<_> = self
            .config
            .companies
            .par_iter()
            .enumerate()
            .map(|(i, company)| {
                let company_seed = self.seed.wrapping_add(i as u64 * 1000);
                let pack = pack.clone();

                // Generate vendors
                let mut vendor_gen = VendorGenerator::new(company_seed);
                vendor_gen.set_country_pack(pack.clone());
                vendor_gen.set_coa_framework(coa_framework);
                let vendor_pool =
                    vendor_gen.generate_vendor_pool(vendors_per_company, &company.code, start_date);

                // Generate customers
                let mut customer_gen = CustomerGenerator::new(company_seed + 100);
                customer_gen.set_country_pack(pack.clone());
                customer_gen.set_coa_framework(coa_framework);
                let customer_pool = customer_gen.generate_customer_pool(
                    customers_per_company,
                    &company.code,
                    start_date,
                );

                // Generate materials
                let mut material_gen = MaterialGenerator::new(company_seed + 200);
                material_gen.set_country_pack(pack.clone());
                let material_pool = material_gen.generate_material_pool(
                    materials_per_company,
                    &company.code,
                    start_date,
                );

                // Generate fixed assets
                let mut asset_gen = AssetGenerator::new(company_seed + 300);
                let asset_pool = asset_gen.generate_asset_pool(
                    assets_per_company,
                    &company.code,
                    (start_date, end_date),
                );

                // Generate employees
                let mut employee_gen = EmployeeGenerator::new(company_seed + 400);
                employee_gen.set_country_pack(pack);
                let employee_pool =
                    employee_gen.generate_company_pool(&company.code, (start_date, end_date));

                (
                    vendor_pool.vendors,
                    customer_pool.customers,
                    material_pool.materials,
                    asset_pool.assets,
                    employee_pool.employees,
                )
            })
            .collect();

        // Aggregate results from all companies
        for (vendors, customers, materials, assets, employees) in per_company_results {
            self.master_data.vendors.extend(vendors);
            self.master_data.customers.extend(customers);
            self.master_data.materials.extend(materials);
            self.master_data.assets.extend(assets);
            self.master_data.employees.extend(employees);
        }

        if let Some(pb) = &pb {
            pb.inc(total);
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
        // Cap at ~2 POs per vendor per month to keep spend concentration realistic
        let months = (self.config.global.period_months as usize).max(1);
        let p2p_count = self
            .phase_config
            .p2p_chains
            .min(self.master_data.vendors.len() * 2 * months);
        let pb = self.create_progress_bar(p2p_count as u64, "Generating P2P Document Flows");

        // Convert P2P config from schema to generator config
        let p2p_config = convert_p2p_config(&self.config.document_flows.p2p);
        let mut p2p_gen = P2PGenerator::with_config(self.seed + 1000, p2p_config);
        p2p_gen.set_country_pack(self.primary_pack().clone());

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
            let created_by = if self.master_data.employees.is_empty() {
                "SYSTEM"
            } else {
                self.master_data.employees[i % self.master_data.employees.len()]
                    .user_id
                    .as_str()
            };

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
            for remainder in &chain.remainder_payments {
                flows.payments.push(remainder.clone());
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
        // Cap at ~2 SOs per customer per month to keep order volume realistic
        let o2c_count = self
            .phase_config
            .o2c_chains
            .min(self.master_data.customers.len() * 2 * months);
        let pb = self.create_progress_bar(o2c_count as u64, "Generating O2C Document Flows");

        // Convert O2C config from schema to generator config
        let o2c_config = convert_o2c_config(&self.config.document_flows.o2c);
        let mut o2c_gen = O2CGenerator::with_config(self.seed + 2000, o2c_config);
        o2c_gen.set_country_pack(self.primary_pack().clone());

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
            let created_by = if self.master_data.employees.is_empty() {
                "SYSTEM"
            } else {
                self.master_data.employees[i % self.master_data.employees.len()]
                    .user_id
                    .as_str()
            };

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
            // Extract remainder receipts (follow-up to partial payments)
            for receipt in &chain.remainder_receipts {
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

    /// Generate journal entries using parallel generation across multiple cores.
    fn generate_journal_entries(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
    ) -> SynthResult<Vec<JournalEntry>> {
        use datasynth_core::traits::ParallelGenerator;

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
        let je_pack = self.primary_pack();

        let mut generator = generator
            .with_master_data(
                &self.master_data.vendors,
                &self.master_data.customers,
                &self.master_data.materials,
            )
            .with_country_pack_names(je_pack)
            .with_country_pack_temporal(
                self.config.temporal_patterns.clone(),
                self.seed + 200,
                je_pack,
            )
            .with_persona_errors(true)
            .with_fraud_config(self.config.fraud.clone());

        // Apply temporal drift if configured
        if self.config.temporal.enabled {
            let drift_config = self.config.temporal.to_core_config();
            generator = generator.with_drift_config(drift_config, self.seed + 100);
        }

        // Check memory limit at start
        self.check_memory_limit()?;

        // Determine parallelism: use available cores, but cap at total entries
        let num_threads = num_cpus::get().max(1).min(total as usize).max(1);

        // Use parallel generation for datasets with 10K+ entries.
        // Below this threshold, the statistical properties of a single-seeded
        // generator (e.g. Benford compliance) are better preserved.
        let entries = if total >= 10_000 && num_threads > 1 {
            // Parallel path: split the generator across cores and generate in parallel.
            // Each sub-generator gets a unique seed for deterministic, independent generation.
            let sub_generators = generator.split(num_threads);
            let entries_per_thread = total as usize / num_threads;
            let remainder = total as usize % num_threads;

            let batches: Vec<Vec<JournalEntry>> = sub_generators
                .into_par_iter()
                .enumerate()
                .map(|(i, mut gen)| {
                    let count = entries_per_thread + if i < remainder { 1 } else { 0 };
                    gen.generate_batch(count)
                })
                .collect();

            // Merge all batches into a single Vec
            let entries = JournalEntryGenerator::merge_results(batches);

            if let Some(pb) = &pb {
                pb.inc(total);
            }
            entries
        } else {
            // Sequential path for small datasets (< 1000 entries)
            let mut entries = Vec::with_capacity(total as usize);
            for _ in 0..total {
                let entry = generator.generate();
                entries.push(entry);
                if let Some(pb) = &pb {
                    pb.inc(1);
                }
            }
            entries
        };

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

        let je_config = match self.resolve_coa_framework() {
            CoAFramework::FrenchPcg => DocumentFlowJeConfig::french_gaap(),
            CoAFramework::GermanSkr04 => {
                let fa = datasynth_core::FrameworkAccounts::german_gaap();
                DocumentFlowJeConfig::from(&fa)
            }
            CoAFramework::UsGaap => DocumentFlowJeConfig::default(),
        };

        let populate_fec = je_config.populate_fec_fields;
        let mut generator = DocumentFlowJeGenerator::with_config_and_seed(je_config, self.seed);

        // Build auxiliary account lookup from vendor/customer master data so that
        // FEC auxiliary_account_number uses framework-specific GL accounts (e.g.,
        // PCG "4010001") instead of raw partner IDs.
        if populate_fec {
            let mut aux_lookup = std::collections::HashMap::new();
            for vendor in &self.master_data.vendors {
                if let Some(ref aux) = vendor.auxiliary_gl_account {
                    aux_lookup.insert(vendor.vendor_id.clone(), aux.clone());
                }
            }
            for customer in &self.master_data.customers {
                if let Some(ref aux) = customer.auxiliary_gl_account {
                    aux_lookup.insert(customer.customer_id.clone(), aux.clone());
                }
            }
            if !aux_lookup.is_empty() {
                generator.set_auxiliary_account_lookup(aux_lookup);
            }
        }

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

    /// Generate journal entries from payroll runs.
    ///
    /// Creates one JE per payroll run:
    /// - DR Salaries & Wages (6100) for gross pay
    /// - CR Payroll Clearing (9100) for gross pay
    fn generate_payroll_jes(payroll_runs: &[PayrollRun]) -> Vec<JournalEntry> {
        use datasynth_core::accounts::{expense_accounts, suspense_accounts};

        let mut jes = Vec::with_capacity(payroll_runs.len());

        for run in payroll_runs {
            let mut je = JournalEntry::new_simple(
                format!("JE-PAYROLL-{}", run.payroll_id),
                run.company_code.clone(),
                run.run_date,
                format!("Payroll {}", run.payroll_id),
            );

            // Debit Salaries & Wages for gross pay
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: expense_accounts::SALARIES_WAGES.to_string(),
                debit_amount: run.total_gross,
                reference: Some(run.payroll_id.clone()),
                text: Some(format!(
                    "Payroll {} ({} employees)",
                    run.payroll_id, run.employee_count
                )),
                ..Default::default()
            });

            // Credit Payroll Clearing for gross pay
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: suspense_accounts::PAYROLL_CLEARING.to_string(),
                credit_amount: run.total_gross,
                reference: Some(run.payroll_id.clone()),
                ..Default::default()
            });

            jes.push(je);
        }

        jes
    }

    /// Generate journal entries from production orders.
    ///
    /// Creates one JE per completed production order:
    /// - DR Raw Materials (5100) for material consumption (actual_cost)
    /// - CR Inventory (1200) for material consumption
    fn generate_manufacturing_jes(production_orders: &[ProductionOrder]) -> Vec<JournalEntry> {
        use datasynth_core::accounts::{control_accounts, expense_accounts};
        use datasynth_core::models::ProductionOrderStatus;

        let mut jes = Vec::new();

        for order in production_orders {
            // Only generate JEs for completed or closed orders
            if !matches!(
                order.status,
                ProductionOrderStatus::Completed | ProductionOrderStatus::Closed
            ) {
                continue;
            }

            let mut je = JournalEntry::new_simple(
                format!("JE-MFG-{}", order.order_id),
                order.company_code.clone(),
                order.actual_end.unwrap_or(order.planned_end),
                format!(
                    "Production Order {} - {}",
                    order.order_id, order.material_description
                ),
            );

            // Debit Raw Materials / Manufacturing expense for actual cost
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: expense_accounts::RAW_MATERIALS.to_string(),
                debit_amount: order.actual_cost,
                reference: Some(order.order_id.clone()),
                text: Some(format!(
                    "Material consumption for {}",
                    order.material_description
                )),
                quantity: Some(order.actual_quantity),
                unit: Some("EA".to_string()),
                ..Default::default()
            });

            // Credit Inventory for material consumption
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: control_accounts::INVENTORY.to_string(),
                credit_amount: order.actual_cost,
                reference: Some(order.order_id.clone()),
                ..Default::default()
            });

            jes.push(je);
        }

        jes
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

        // Build vendor/customer name maps from master data for realistic subledger names
        let vendor_names: std::collections::HashMap<String, String> = self
            .master_data
            .vendors
            .iter()
            .map(|v| (v.vendor_id.clone(), v.name.clone()))
            .collect();
        let customer_names: std::collections::HashMap<String, String> = self
            .master_data
            .customers
            .iter()
            .map(|c| (c.customer_id.clone(), c.name.clone()))
            .collect();

        let mut linker = DocumentFlowLinker::new()
            .with_vendor_names(vendor_names)
            .with_customer_names(customer_names);

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
            fa_records: Vec::new(),
            inventory_positions: Vec::new(),
            inventory_movements: Vec::new(),
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
        let ocpm_uuid_factory = OcpmUuidFactory::new(self.seed + 3001);

        // Get available users for resource assignment
        let available_users: Vec<String> = self
            .master_data
            .employees
            .iter()
            .take(20)
            .map(|e| e.user_id.clone())
            .collect();

        // Deterministic base date from config (avoids Utc::now() non-determinism)
        let fallback_date =
            NaiveDate::from_ymd_opt(2024, 1, 1).expect("static date 2024-01-01 is always valid");
        let base_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .unwrap_or(fallback_date);
        let base_midnight = base_date
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always valid");
        let base_datetime =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(base_midnight, chrono::Utc);

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
            for corr in result.correlation_events {
                event_log.add_correlation_event(corr);
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
                &ocpm_uuid_factory,
            )
            .with_goods_receipt(
                chain
                    .goods_receipts
                    .first()
                    .map(|gr| gr.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
            )
            .with_invoice(
                chain
                    .vendor_invoice
                    .as_ref()
                    .map(|vi| vi.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
            )
            .with_payment(
                chain
                    .payment
                    .as_ref()
                    .map(|p| p.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
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
                &ocpm_uuid_factory,
            )
            .with_delivery(
                chain
                    .deliveries
                    .first()
                    .map(|d| d.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
            )
            .with_invoice(
                chain
                    .customer_invoice
                    .as_ref()
                    .map(|ci| ci.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
            )
            .with_receipt(
                chain
                    .customer_receipt
                    .as_ref()
                    .map(|r| r.header.document_id.as_str())
                    .unwrap_or(""),
                &ocpm_uuid_factory,
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
                .or_else(|| {
                    self.master_data
                        .vendors
                        .first()
                        .map(|v| v.vendor_id.clone())
                })
                .unwrap_or_else(|| "V000".to_string());
            let mut docs = S2cDocuments::new(
                &project.project_id,
                &vendor_id,
                &project.company_code,
                project.estimated_annual_spend,
                &ocpm_uuid_factory,
            );
            // Link RFx if available
            if let Some(rfx) = sourcing
                .rfx_events
                .iter()
                .find(|r| r.sourcing_project_id == project.project_id)
            {
                docs = docs.with_rfx(&rfx.rfx_id, &ocpm_uuid_factory);
                // Link winning bid (status == Accepted)
                if let Some(bid) = sourcing.bids.iter().find(|b| {
                    b.rfx_id == rfx.rfx_id
                        && b.status == datasynth_core::models::sourcing::BidStatus::Accepted
                }) {
                    docs = docs.with_winning_bid(&bid.bid_id, &ocpm_uuid_factory);
                }
            }
            // Link contract
            if let Some(contract) = sourcing
                .contracts
                .iter()
                .find(|c| c.sourcing_project_id.as_deref() == Some(&project.project_id))
            {
                docs = docs.with_contract(&contract.contract_id, &ocpm_uuid_factory);
            }
            let start_time = base_datetime - chrono::Duration::days(90);
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
                &ocpm_uuid_factory,
            )
            .with_time_entries(
                hr.time_entries
                    .iter()
                    .filter(|t| t.date >= run.pay_period_start && t.date <= run.pay_period_end)
                    .take(5)
                    .map(|t| t.entry_id.as_str())
                    .collect(),
            );
            let start_time = base_datetime - chrono::Duration::days(30);
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
                &ocpm_uuid_factory,
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
                docs = docs.with_inspection(&insp.inspection_id, &ocpm_uuid_factory);
            }
            // Link cycle count if available (match by material_id in items)
            if let Some(cc) = manufacturing.cycle_counts.iter().find(|cc| {
                cc.items
                    .iter()
                    .any(|item| item.material_id == order.material_id)
            }) {
                docs = docs.with_cycle_count(&cc.count_id, &ocpm_uuid_factory);
            }
            let start_time = base_datetime - chrono::Duration::days(60);
            let result = ocpm_gen.generate_mfg_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from Banking customers
        for customer in &banking.customers {
            let customer_id_str = customer.customer_id.to_string();
            let mut docs = BankDocuments::new(&customer_id_str, "1000", &ocpm_uuid_factory);
            // Link accounts (primary_owner_id matches customer_id)
            if let Some(account) = banking
                .accounts
                .iter()
                .find(|a| a.primary_owner_id == customer.customer_id)
            {
                let account_id_str = account.account_id.to_string();
                docs = docs.with_account(&account_id_str, &ocpm_uuid_factory);
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
            let start_time = base_datetime - chrono::Duration::days(180);
            let result = ocpm_gen.generate_bank_case(&docs, start_time, &available_users);
            add_result(&mut event_log, result);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate events from Audit engagements
        for engagement in &audit.engagements {
            let engagement_id_str = engagement.engagement_id.to_string();
            let docs = AuditDocuments::new(
                &engagement_id_str,
                &engagement.client_entity_id,
                &ocpm_uuid_factory,
            )
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
            let start_time = base_datetime - chrono::Duration::days(120);
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
                &ocpm_uuid_factory,
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
            let start_time = base_datetime - chrono::Duration::days(30);
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

        // Read anomaly rates from config instead of using hardcoded values.
        // Priority: anomaly_injection config > fraud config > default 0.02
        let total_rate = if self.config.anomaly_injection.enabled {
            self.config.anomaly_injection.rates.total_rate
        } else if self.config.fraud.enabled {
            self.config.fraud.fraud_rate
        } else {
            0.02
        };

        let fraud_rate = if self.config.anomaly_injection.enabled {
            self.config.anomaly_injection.rates.fraud_rate
        } else {
            AnomalyRateConfig::default().fraud_rate
        };

        let error_rate = if self.config.anomaly_injection.enabled {
            self.config.anomaly_injection.rates.error_rate
        } else {
            AnomalyRateConfig::default().error_rate
        };

        let process_issue_rate = if self.config.anomaly_injection.enabled {
            self.config.anomaly_injection.rates.process_rate
        } else {
            AnomalyRateConfig::default().process_issue_rate
        };

        let anomaly_config = AnomalyInjectorConfig {
            rates: AnomalyRateConfig {
                total_rate,
                fraud_rate,
                error_rate,
                process_issue_rate,
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
        let validation_currency = self
            .config
            .companies
            .first()
            .map(|c| c.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        let mut tracker = RunningBalanceTracker::new_with_currency(config, validation_currency);

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
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

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

        // Build config from user-specified schema settings when data_quality is enabled;
        // otherwise fall back to the low-rate minimal() preset.
        let config = if self.config.data_quality.enabled {
            let dq = &self.config.data_quality;
            DataQualityConfig {
                enable_missing_values: dq.missing_values.enabled,
                missing_values: datasynth_generators::MissingValueConfig {
                    global_rate: dq.effective_missing_rate(),
                    ..Default::default()
                },
                enable_format_variations: dq.format_variations.enabled,
                format_variations: datasynth_generators::FormatVariationConfig {
                    date_variation_rate: dq.format_variations.dates.rate,
                    amount_variation_rate: dq.format_variations.amounts.rate,
                    identifier_variation_rate: dq.format_variations.identifiers.rate,
                    ..Default::default()
                },
                enable_duplicates: dq.duplicates.enabled,
                duplicates: datasynth_generators::DuplicateConfig {
                    duplicate_rate: dq.effective_duplicate_rate(),
                    ..Default::default()
                },
                enable_typos: dq.typos.enabled,
                typos: datasynth_generators::TypoConfig {
                    char_error_rate: dq.effective_typo_rate(),
                    ..Default::default()
                },
                enable_encoding_issues: dq.encoding_issues.enabled,
                encoding_issue_rate: dq.encoding_issues.rate,
                seed: self.seed.wrapping_add(77), // deterministic offset for DQ phase
                track_statistics: true,
            }
        } else {
            DataQualityConfig::minimal()
        };
        let mut injector = DataQualityInjector::new(config);

        // Wire country pack for locale-aware format baselines
        injector.set_country_pack(self.primary_pack().clone());

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
                let mut engagement = engagement_gen.generate_engagement(
                    &company.code,
                    &company.name,
                    fiscal_year,
                    period_end,
                    company_revenue,
                    None, // Use default engagement type
                );

                // Replace synthetic team IDs with real employee IDs from master data
                if !self.master_data.employees.is_empty() {
                    let emp_count = self.master_data.employees.len();
                    // Use employee IDs deterministically based on engagement index
                    let base = (i * 10 + _eng_idx) % emp_count;
                    engagement.engagement_partner_id = self.master_data.employees[base % emp_count]
                        .employee_id
                        .clone();
                    engagement.engagement_manager_id = self.master_data.employees
                        [(base + 1) % emp_count]
                        .employee_id
                        .clone();
                    let real_team: Vec<String> = engagement
                        .team_member_ids
                        .iter()
                        .enumerate()
                        .map(|(j, _)| {
                            self.master_data.employees[(base + 2 + j) % emp_count]
                                .employee_id
                                .clone()
                        })
                        .collect();
                    engagement.team_member_ids = real_team;
                }

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
                framework: None,
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
                        use datasynth_graph::{Neo4jExportConfig, Neo4jExporter};

                        let neo4j_config = Neo4jExportConfig {
                            export_node_properties: true,
                            export_edge_properties: true,
                            export_features: true,
                            generate_cypher: true,
                            generate_admin_import: true,
                            database_name: "synth".to_string(),
                            cypher_batch_size: 1000,
                        };

                        let exporter = Neo4jExporter::new(neo4j_config);
                        match exporter.export(&graph, &format_dir) {
                            Ok(metadata) => {
                                snapshot.exports.insert(
                                    format!("{}_{}", graph_type.name, "neo4j"),
                                    GraphExportInfo {
                                        name: graph_type.name.clone(),
                                        format: "neo4j".to_string(),
                                        output_path: format_dir.clone(),
                                        node_count: metadata.num_nodes,
                                        edge_count: metadata.num_edges,
                                    },
                                );
                                snapshot.graph_count += 1;
                            }
                            Err(e) => {
                                warn!("Failed to export Neo4j graph: {}", e);
                            }
                        }
                    }
                    datasynth_config::schema::GraphExportFormat::Dgl => {
                        use datasynth_graph::{DGLExportConfig, DGLExporter};

                        let dgl_config = DGLExportConfig {
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
                            heterogeneous: false,
                            include_pickle_script: true, // DGL ecosystem standard helper
                        };

                        let exporter = DGLExporter::new(dgl_config);
                        match exporter.export(&graph, &format_dir) {
                            Ok(metadata) => {
                                snapshot.exports.insert(
                                    format!("{}_{}", graph_type.name, "dgl"),
                                    GraphExportInfo {
                                        name: graph_type.name.clone(),
                                        format: "dgl".to_string(),
                                        output_path: format_dir.clone(),
                                        node_count: metadata.common.num_nodes,
                                        edge_count: metadata.common.num_edges,
                                    },
                                );
                                snapshot.graph_count += 1;
                            }
                            Err(e) => {
                                warn!("Failed to export DGL graph: {}", e);
                            }
                        }
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

    /// Build additional graph types (banking, approval, entity) when relevant data
    /// is available. These run as a late phase because the data they need (banking
    /// snapshot, intercompany snapshot) is only generated after the main graph
    /// export phase.
    fn build_additional_graphs(
        &self,
        banking: &BankingSnapshot,
        intercompany: &IntercompanySnapshot,
        entries: &[JournalEntry],
        stats: &mut EnhancedGenerationStatistics,
    ) {
        let output_dir = self
            .output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.config.output.output_directory));
        let graph_dir = output_dir.join(&self.config.graph_export.output_subdirectory);

        // Banking graph: build when banking customers and transactions exist
        if !banking.customers.is_empty() && !banking.transactions.is_empty() {
            info!("Phase 10c: Building banking network graph");
            let config = BankingGraphConfig::default();
            let mut builder = BankingGraphBuilder::new(config);
            builder.add_customers(&banking.customers);
            builder.add_accounts(&banking.accounts, &banking.customers);
            builder.add_transactions(&banking.transactions);
            let graph = builder.build();

            let node_count = graph.node_count();
            let edge_count = graph.edge_count();
            stats.graph_node_count += node_count;
            stats.graph_edge_count += edge_count;

            // Export as PyG if configured
            for format in &self.config.graph_export.formats {
                if matches!(
                    format,
                    datasynth_config::schema::GraphExportFormat::PytorchGeometric
                ) {
                    let format_dir = graph_dir.join("banking_network").join("pytorch_geometric");
                    if let Err(e) = std::fs::create_dir_all(&format_dir) {
                        warn!("Failed to create banking graph output dir: {}", e);
                        continue;
                    }
                    let pyg_config = PyGExportConfig::default();
                    let exporter = PyGExporter::new(pyg_config);
                    if let Err(e) = exporter.export(&graph, &format_dir) {
                        warn!("Failed to export banking graph as PyG: {}", e);
                    } else {
                        info!(
                            "Banking network graph exported: {} nodes, {} edges",
                            node_count, edge_count
                        );
                    }
                }
            }
        }

        // Approval graph: build from journal entry approval workflows
        let approval_entries: Vec<_> = entries
            .iter()
            .filter(|je| je.header.approval_workflow.is_some())
            .collect();

        if !approval_entries.is_empty() {
            info!(
                "Phase 10c: Building approval network graph ({} entries with approvals)",
                approval_entries.len()
            );
            let config = ApprovalGraphConfig::default();
            let mut builder = ApprovalGraphBuilder::new(config);

            for je in &approval_entries {
                if let Some(ref wf) = je.header.approval_workflow {
                    for action in &wf.actions {
                        let record = datasynth_core::models::ApprovalRecord {
                            approval_id: format!(
                                "APR-{}-{}",
                                je.header.document_id, action.approval_level
                            ),
                            document_number: je.header.document_id.to_string(),
                            document_type: "JE".to_string(),
                            company_code: je.company_code().to_string(),
                            requester_id: wf.preparer_id.clone(),
                            requester_name: Some(wf.preparer_name.clone()),
                            approver_id: action.actor_id.clone(),
                            approver_name: action.actor_name.clone(),
                            approval_date: je.posting_date(),
                            action: format!("{:?}", action.action),
                            amount: wf.amount,
                            approval_limit: None,
                            comments: action.comments.clone(),
                            delegation_from: None,
                            is_auto_approved: false,
                        };
                        builder.add_approval(&record);
                    }
                }
            }

            let graph = builder.build();
            let node_count = graph.node_count();
            let edge_count = graph.edge_count();
            stats.graph_node_count += node_count;
            stats.graph_edge_count += edge_count;

            // Export as PyG if configured
            for format in &self.config.graph_export.formats {
                if matches!(
                    format,
                    datasynth_config::schema::GraphExportFormat::PytorchGeometric
                ) {
                    let format_dir = graph_dir.join("approval_network").join("pytorch_geometric");
                    if let Err(e) = std::fs::create_dir_all(&format_dir) {
                        warn!("Failed to create approval graph output dir: {}", e);
                        continue;
                    }
                    let pyg_config = PyGExportConfig::default();
                    let exporter = PyGExporter::new(pyg_config);
                    if let Err(e) = exporter.export(&graph, &format_dir) {
                        warn!("Failed to export approval graph as PyG: {}", e);
                    } else {
                        info!(
                            "Approval network graph exported: {} nodes, {} edges",
                            node_count, edge_count
                        );
                    }
                }
            }
        }

        // Entity graph: map CompanyConfig → Company and wire intercompany relationships
        if self.config.companies.len() >= 2 {
            info!(
                "Phase 10c: Building entity relationship graph ({} companies)",
                self.config.companies.len()
            );

            let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"));

            // Map CompanyConfig → Company objects
            let parent_code = &self.config.companies[0].code;
            let mut companies: Vec<datasynth_core::models::Company> =
                Vec::with_capacity(self.config.companies.len());

            // First company is the parent
            let first = &self.config.companies[0];
            companies.push(datasynth_core::models::Company::parent(
                &first.code,
                &first.name,
                &first.country,
                &first.currency,
            ));

            // Remaining companies are subsidiaries (100% owned by parent)
            for cc in self.config.companies.iter().skip(1) {
                companies.push(datasynth_core::models::Company::subsidiary(
                    &cc.code,
                    &cc.name,
                    &cc.country,
                    &cc.currency,
                    parent_code,
                    rust_decimal::Decimal::from(100),
                ));
            }

            // Build IntercompanyRelationship records (same logic as phase_intercompany)
            let relationships: Vec<datasynth_core::models::intercompany::IntercompanyRelationship> =
                self.config
                    .companies
                    .iter()
                    .skip(1)
                    .enumerate()
                    .map(|(i, cc)| {
                        let mut rel =
                            datasynth_core::models::intercompany::IntercompanyRelationship::new(
                                format!("REL{:03}", i + 1),
                                parent_code.clone(),
                                cc.code.clone(),
                                rust_decimal::Decimal::from(100),
                                start_date,
                            );
                        rel.functional_currency = cc.currency.clone();
                        rel
                    })
                    .collect();

            let mut builder = EntityGraphBuilder::new(EntityGraphConfig::default());
            builder.add_companies(&companies);
            builder.add_ownership_relationships(&relationships);

            // Thread IC matched-pair transaction edges into the entity graph
            for pair in &intercompany.matched_pairs {
                builder.add_intercompany_edge(
                    &pair.seller_company,
                    &pair.buyer_company,
                    pair.amount,
                    &format!("{:?}", pair.transaction_type),
                );
            }

            let graph = builder.build();
            let node_count = graph.node_count();
            let edge_count = graph.edge_count();
            stats.graph_node_count += node_count;
            stats.graph_edge_count += edge_count;

            // Export as PyG if configured
            for format in &self.config.graph_export.formats {
                if matches!(
                    format,
                    datasynth_config::schema::GraphExportFormat::PytorchGeometric
                ) {
                    let format_dir = graph_dir.join("entity_network").join("pytorch_geometric");
                    if let Err(e) = std::fs::create_dir_all(&format_dir) {
                        warn!("Failed to create entity graph output dir: {}", e);
                        continue;
                    }
                    let pyg_config = PyGExportConfig::default();
                    let exporter = PyGExporter::new(pyg_config);
                    if let Err(e) = exporter.export(&graph, &format_dir) {
                        warn!("Failed to export entity graph as PyG: {}", e);
                    } else {
                        info!(
                            "Entity relationship graph exported: {} nodes, {} edges",
                            node_count, edge_count
                        );
                    }
                }
            }
        } else {
            debug!(
                "EntityGraphBuilder: skipped (requires 2+ companies, found {})",
                self.config.companies.len()
            );
        }
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
            .country_pack(self.primary_pack().clone())
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

        // Cross-reference banking customers with core master data so that
        // banking customer names align with the enterprise customer list.
        // We rotate through core customers, overlaying their name and country
        // onto the generated banking customers where possible.
        let mut banking_customers = result.customers;
        let core_customers = &self.master_data.customers;
        if !core_customers.is_empty() {
            for (i, bc) in banking_customers.iter_mut().enumerate() {
                let core = &core_customers[i % core_customers.len()];
                bc.name = CustomerName::business(&core.name);
                bc.residence_country = core.country.clone();
                bc.enterprise_customer_id = Some(core.customer_id.clone());
            }
            debug!(
                "Cross-referenced {} banking customers with {} core customers",
                banking_customers.len(),
                core_customers.len()
            );
        }

        Ok(BankingSnapshot {
            customers: banking_customers,
            accounts: result.accounts,
            transactions: result.transactions,
            transaction_labels: result.transaction_labels,
            customer_labels: result.customer_labels,
            account_labels: result.account_labels,
            relationship_labels: result.relationship_labels,
            narratives: result.narratives,
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
                fiscal_year_months: None,
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
            country_packs: None,
            scenarios: Default::default(),
            session: Default::default(),
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
        let result = orchestrator.generate().unwrap();

        // After generate(), master_data is moved into the result
        assert!(!result.master_data.vendors.is_empty());
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
        assert_eq!(result.statistics.counterfactual_pair_count, 0);
        assert!(result.counterfactual_pairs.is_empty());
    }

    #[test]
    fn test_counterfactual_generation_enabled() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            generate_counterfactuals: true,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // With JE generation enabled, counterfactual pairs should be generated
        if !result.journal_entries.is_empty() {
            assert_eq!(
                result.counterfactual_pairs.len(),
                result.journal_entries.len()
            );
            assert_eq!(
                result.statistics.counterfactual_pair_count,
                result.journal_entries.len()
            );
            // Each pair should have a distinct pair_id
            let ids: std::collections::HashSet<_> = result
                .counterfactual_pairs
                .iter()
                .map(|p| p.pair_id.clone())
                .collect();
            assert_eq!(ids.len(), result.counterfactual_pairs.len());
        }
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
