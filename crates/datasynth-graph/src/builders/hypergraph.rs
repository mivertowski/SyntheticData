//! Multi-layer hypergraph builder for RustGraph integration.
//!
//! Constructs a 3-layer hypergraph from accounting data:
//! - Layer 1: Governance & Controls (COSO, internal controls, master data)
//! - Layer 2: Process Events (P2P/O2C documents, OCPM events)
//! - Layer 3: Accounting Network (GL accounts, journal entries as hyperedges)
//!
//! Includes a node budget system that allocates capacity across layers and
//! aggregates overflow nodes into pool nodes when budget is exceeded.

use std::collections::HashMap;

use chrono::Datelike;
use serde_json::Value;

use datasynth_banking::models::{BankAccount, BankTransaction, BankingCustomer};
use datasynth_core::models::audit::{
    AuditEngagement, AuditEvidence, AuditFinding, ProfessionalJudgment, RiskAssessment, Workpaper,
};
use datasynth_core::models::compliance::{ComplianceFinding, ComplianceStandard, RegulatoryFiling};
use datasynth_core::models::sourcing::{
    BidEvaluation, ProcurementContract, RfxEvent, SourcingProject, SupplierBid,
    SupplierQualification,
};
use datasynth_core::models::intercompany::{EliminationEntry, ICMatchedPair};
use datasynth_core::models::ExpenseReport;
use datasynth_core::models::{
    BankReconciliation, CashForecast, CashPosition, ChartOfAccounts, ClimateScenario,
    CosoComponent, CosoPrinciple, Customer, CycleCount, DebtInstrument, EarnedValueMetric,
    EmissionRecord, Employee, EsgDisclosure, FixedAsset, HedgeRelationship, InternalControl,
    JournalEntry, Material, OrganizationalEvent, PayrollRun, ProcessEvolutionEvent,
    ProductionOrder, Project, ProjectMilestone, QualityInspection, SupplierEsgAssessment,
    TaxCode, TaxJurisdiction, TaxLine, TaxProvision, TaxReturn, TimeEntry, Vendor,
    WithholdingTaxRecord,
};
use datasynth_generators::disruption::DisruptionEvent;

use crate::models::hypergraph::{
    AggregationStrategy, CrossLayerEdge, Hyperedge, HyperedgeParticipant, Hypergraph,
    HypergraphLayer, HypergraphMetadata, HypergraphNode, NodeBudget, NodeBudgetReport,
};

/// Day-of-month threshold for considering a date as "month-end" in features.
const MONTH_END_DAY_THRESHOLD: u32 = 28;
/// Normalizer for weekday feature (0=Monday..6=Sunday).
const WEEKDAY_NORMALIZER: f64 = 6.0;
/// Normalizer for day-of-month feature.
const DAY_OF_MONTH_NORMALIZER: f64 = 31.0;
/// Normalizer for month feature.
const MONTH_NORMALIZER: f64 = 12.0;

/// RustGraph entity type codes — canonical codes from AssureTwin's entity_registry.rs.
/// Not all codes are consumed yet; the full set is kept for parity with the
/// upstream registry so that new layer builders can reference them immediately.
#[allow(dead_code)]
mod type_codes {
    // Layer 3 — Accounting / Master Data
    pub const ACCOUNT: u32 = 100;
    pub const JOURNAL_ENTRY: u32 = 101;
    pub const MATERIAL: u32 = 102;
    pub const FIXED_ASSET: u32 = 103;
    pub const COST_CENTER: u32 = 104;

    // People / Organizations
    pub const VENDOR: u32 = 200;
    pub const CUSTOMER: u32 = 201;
    pub const EMPLOYEE: u32 = 202;
    pub const BANKING_CUSTOMER: u32 = 203;

    // Layer 2 process type codes — P2P
    pub const PURCHASE_ORDER: u32 = 300;
    pub const GOODS_RECEIPT: u32 = 301;
    pub const VENDOR_INVOICE: u32 = 302;
    pub const PAYMENT: u32 = 303;
    // Layer 2 — O2C
    pub const SALES_ORDER: u32 = 310;
    pub const DELIVERY: u32 = 311;
    pub const CUSTOMER_INVOICE: u32 = 312;
    // Layer 2 — S2C
    pub const SOURCING_PROJECT: u32 = 320;
    pub const RFX_EVENT: u32 = 321;
    pub const SUPPLIER_BID: u32 = 322;
    pub const BID_EVALUATION: u32 = 323;
    pub const PROCUREMENT_CONTRACT: u32 = 324;
    pub const SUPPLIER_QUALIFICATION: u32 = 325;
    // Layer 2 — H2R
    pub const PAYROLL_RUN: u32 = 330;
    pub const TIME_ENTRY: u32 = 331;
    pub const EXPENSE_REPORT: u32 = 332;
    pub const PAYROLL_LINE_ITEM: u32 = 333;
    // Layer 2 — MFG
    pub const PRODUCTION_ORDER: u32 = 340;
    pub const QUALITY_INSPECTION: u32 = 341;
    pub const CYCLE_COUNT: u32 = 342;
    // Layer 2 — BANK
    pub const BANK_ACCOUNT: u32 = 350;
    pub const BANK_TRANSACTION: u32 = 351;
    pub const BANK_STATEMENT_LINE: u32 = 352;
    // Layer 2 — AUDIT
    pub const AUDIT_ENGAGEMENT: u32 = 360;
    pub const WORKPAPER: u32 = 361;
    pub const AUDIT_FINDING: u32 = 362;
    pub const AUDIT_EVIDENCE: u32 = 363;
    pub const RISK_ASSESSMENT: u32 = 364;
    pub const PROFESSIONAL_JUDGMENT: u32 = 365;
    // Layer 2 — Bank Recon (R2R subfamily)
    pub const BANK_RECONCILIATION: u32 = 370;
    pub const RECONCILING_ITEM: u32 = 372;
    // Layer 2 — OCPM events
    pub const OCPM_EVENT: u32 = 400;
    // Pool / aggregate
    pub const POOL_NODE: u32 = 399;

    // Layer 1 — Governance
    pub const COSO_COMPONENT: u32 = 500;
    pub const COSO_PRINCIPLE: u32 = 501;
    pub const SOX_ASSERTION: u32 = 502;
    pub const INTERNAL_CONTROL: u32 = 503;
    pub const KYC_PROFILE: u32 = 504;
    pub const COMPLIANCE_STANDARD: u32 = 505;
    pub const JURISDICTION: u32 = 506;
    // Layer 2 — Compliance events
    pub const REGULATORY_FILING: u32 = 507;
    pub const COMPLIANCE_FINDING: u32 = 508;

    // Layer 3 — Tax
    pub const TAX_JURISDICTION: u32 = 410;
    pub const TAX_CODE: u32 = 411;
    pub const TAX_LINE: u32 = 412;
    pub const TAX_RETURN: u32 = 413;
    pub const TAX_PROVISION: u32 = 414;
    pub const WITHHOLDING_TAX: u32 = 415;

    // Layer 3 — Treasury
    pub const CASH_POSITION: u32 = 420;
    pub const CASH_FORECAST: u32 = 421;
    pub const HEDGE_RELATIONSHIP: u32 = 422;
    pub const DEBT_INSTRUMENT: u32 = 423;

    // Layer 1 — ESG
    pub const EMISSION_RECORD: u32 = 430;
    pub const ESG_DISCLOSURE: u32 = 431;
    pub const SUPPLIER_ESG_ASSESSMENT: u32 = 432;
    pub const CLIMATE_SCENARIO: u32 = 433;

    // Layer 3 — Project Accounting
    pub const PROJECT: u32 = 451;
    pub const EARNED_VALUE: u32 = 452;
    pub const PROJECT_MILESTONE: u32 = 454;

    // Layer 3 — Intercompany
    pub const IC_MATCHED_PAIR: u32 = 460;
    pub const ELIMINATION_ENTRY: u32 = 461;

    // Layer 2 — Temporal Events
    pub const PROCESS_EVOLUTION: u32 = 470;
    pub const ORGANIZATIONAL_EVENT: u32 = 471;
    pub const DISRUPTION_EVENT: u32 = 472;

    // Layer 2 — AML/KYC (from banking)
    pub const AML_ALERT: u32 = 505;
    // KYC_PROFILE already defined above as 504

    // Edge type codes
    pub const IMPLEMENTS_CONTROL: u32 = 40;
    pub const GOVERNED_BY_STANDARD: u32 = 41;
    pub const OWNS_CONTROL: u32 = 42;
    pub const OVERSEE_PROCESS: u32 = 43;
    pub const ENFORCES_ASSERTION: u32 = 44;
    pub const STANDARD_TO_CONTROL: u32 = 45;
    pub const FINDING_ON_CONTROL: u32 = 46;
    pub const STANDARD_TO_ACCOUNT: u32 = 47;
    pub const SUPPLIES_TO: u32 = 48;
    pub const FILED_BY_COMPANY: u32 = 49;
    pub const COVERS_COSO_PRINCIPLE: u32 = 54;
    pub const CONTAINS_ACCOUNT: u32 = 55;
}

/// Configuration for the hypergraph builder.
#[derive(Debug, Clone)]
pub struct HypergraphConfig {
    /// Maximum total nodes across all layers.
    pub max_nodes: usize,
    /// Aggregation strategy when budget is exceeded.
    pub aggregation_strategy: AggregationStrategy,
    // Layer 1 toggles
    pub include_coso: bool,
    pub include_controls: bool,
    pub include_sox: bool,
    pub include_vendors: bool,
    pub include_customers: bool,
    pub include_employees: bool,
    // Layer 2 toggles
    pub include_p2p: bool,
    pub include_o2c: bool,
    pub include_s2c: bool,
    pub include_h2r: bool,
    pub include_mfg: bool,
    pub include_bank: bool,
    pub include_audit: bool,
    pub include_compliance: bool,
    pub include_r2r: bool,
    pub include_tax: bool,
    pub include_treasury: bool,
    pub include_esg: bool,
    pub include_project: bool,
    pub include_intercompany: bool,
    pub include_temporal_events: bool,
    pub events_as_hyperedges: bool,
    /// Documents per counterparty above which aggregation is triggered.
    pub docs_per_counterparty_threshold: usize,
    // Layer 3 toggles
    pub include_accounts: bool,
    pub je_as_hyperedges: bool,
    // Cross-layer
    pub include_cross_layer_edges: bool,
}

impl Default for HypergraphConfig {
    fn default() -> Self {
        Self {
            max_nodes: 50_000,
            aggregation_strategy: AggregationStrategy::PoolByCounterparty,
            include_coso: true,
            include_controls: true,
            include_sox: true,
            include_vendors: true,
            include_customers: true,
            include_employees: true,
            include_p2p: true,
            include_o2c: true,
            include_s2c: true,
            include_h2r: true,
            include_mfg: true,
            include_bank: true,
            include_audit: true,
            include_compliance: true,
            include_r2r: true,
            include_tax: true,
            include_treasury: true,
            include_esg: true,
            include_project: true,
            include_intercompany: true,
            include_temporal_events: true,
            events_as_hyperedges: true,
            docs_per_counterparty_threshold: 20,
            include_accounts: true,
            je_as_hyperedges: true,
            include_cross_layer_edges: true,
        }
    }
}

/// Builder for constructing a multi-layer hypergraph.
pub struct HypergraphBuilder {
    config: HypergraphConfig,
    budget: NodeBudget,
    nodes: Vec<HypergraphNode>,
    edges: Vec<CrossLayerEdge>,
    hyperedges: Vec<Hyperedge>,
    /// Track node IDs to avoid duplicates: external_id → index in nodes vec.
    node_index: HashMap<String, usize>,
    /// Track aggregate node count.
    aggregate_count: usize,
    /// Control ID → node ID mapping for cross-layer edges.
    control_node_ids: HashMap<String, String>,
    /// COSO component → node ID mapping.
    coso_component_ids: HashMap<String, String>,
    /// Account code → node ID mapping.
    account_node_ids: HashMap<String, String>,
    /// Vendor ID → node ID mapping.
    vendor_node_ids: HashMap<String, String>,
    /// Customer ID → node ID mapping.
    customer_node_ids: HashMap<String, String>,
    /// Employee ID → node ID mapping.
    employee_node_ids: HashMap<String, String>,
    /// Process document node IDs to their counterparty type and ID.
    /// (node_id, entity_type) → counterparty_id
    doc_counterparty_links: Vec<(String, String, String)>, // (doc_node_id, counterparty_type, counterparty_id)
    /// Compliance standard ID → node ID mapping.
    standard_node_ids: HashMap<String, String>,
    /// Compliance finding → control_id deferred edges.
    compliance_finding_control_links: Vec<(String, String)>, // (finding_node_id, control_id)
    /// Standard → account code deferred edges (resolved in `build_cross_layer_edges`).
    #[allow(dead_code)]
    standard_account_links: Vec<(String, String)>, // (standard_node_id, account_code)
}

impl HypergraphBuilder {
    /// Create a new builder with the given configuration.
    pub fn new(config: HypergraphConfig) -> Self {
        let budget = NodeBudget::new(config.max_nodes);
        Self {
            config,
            budget,
            nodes: Vec::new(),
            edges: Vec::new(),
            hyperedges: Vec::new(),
            node_index: HashMap::new(),
            aggregate_count: 0,
            control_node_ids: HashMap::new(),
            coso_component_ids: HashMap::new(),
            account_node_ids: HashMap::new(),
            vendor_node_ids: HashMap::new(),
            customer_node_ids: HashMap::new(),
            employee_node_ids: HashMap::new(),
            doc_counterparty_links: Vec::new(),
            standard_node_ids: HashMap::new(),
            compliance_finding_control_links: Vec::new(),
            standard_account_links: Vec::new(),
        }
    }

    /// Rebalance the per-layer budget based on actual demand.
    /// Unused slots from layers with fewer entities than their max are
    /// redistributed to L2 (Process), which is typically the largest consumer.
    /// Call this after adding all governance and accounting nodes, but before
    /// adding large L2 producers like OCPM events.
    pub fn rebalance_budget(&mut self, l1_demand: usize, l2_demand: usize, l3_demand: usize) {
        self.budget.rebalance(l1_demand, l2_demand, l3_demand);
    }

    /// Add COSO framework as Layer 1 nodes (5 components + 17 principles).
    pub fn add_coso_framework(&mut self) {
        if !self.config.include_coso {
            return;
        }

        let components = [
            (CosoComponent::ControlEnvironment, "Control Environment"),
            (CosoComponent::RiskAssessment, "Risk Assessment"),
            (CosoComponent::ControlActivities, "Control Activities"),
            (
                CosoComponent::InformationCommunication,
                "Information & Communication",
            ),
            (CosoComponent::MonitoringActivities, "Monitoring Activities"),
        ];

        for (component, name) in &components {
            let id = format!("coso_comp_{}", name.replace(' ', "_").replace('&', "and"));
            if self.try_add_node(HypergraphNode {
                id: id.clone(),
                entity_type: "coso_component".to_string(),
                entity_type_code: type_codes::COSO_COMPONENT,
                layer: HypergraphLayer::GovernanceControls,
                external_id: format!("{component:?}"),
                label: name.to_string(),
                properties: HashMap::new(),
                features: vec![component_to_feature(component)],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.coso_component_ids.insert(format!("{component:?}"), id);
            }
        }

        let principles = [
            (
                CosoPrinciple::IntegrityAndEthics,
                "Integrity and Ethics",
                CosoComponent::ControlEnvironment,
            ),
            (
                CosoPrinciple::BoardOversight,
                "Board Oversight",
                CosoComponent::ControlEnvironment,
            ),
            (
                CosoPrinciple::OrganizationalStructure,
                "Organizational Structure",
                CosoComponent::ControlEnvironment,
            ),
            (
                CosoPrinciple::CommitmentToCompetence,
                "Commitment to Competence",
                CosoComponent::ControlEnvironment,
            ),
            (
                CosoPrinciple::Accountability,
                "Accountability",
                CosoComponent::ControlEnvironment,
            ),
            (
                CosoPrinciple::ClearObjectives,
                "Clear Objectives",
                CosoComponent::RiskAssessment,
            ),
            (
                CosoPrinciple::IdentifyRisks,
                "Identify Risks",
                CosoComponent::RiskAssessment,
            ),
            (
                CosoPrinciple::FraudRisk,
                "Fraud Risk",
                CosoComponent::RiskAssessment,
            ),
            (
                CosoPrinciple::ChangeIdentification,
                "Change Identification",
                CosoComponent::RiskAssessment,
            ),
            (
                CosoPrinciple::ControlActions,
                "Control Actions",
                CosoComponent::ControlActivities,
            ),
            (
                CosoPrinciple::TechnologyControls,
                "Technology Controls",
                CosoComponent::ControlActivities,
            ),
            (
                CosoPrinciple::PoliciesAndProcedures,
                "Policies and Procedures",
                CosoComponent::ControlActivities,
            ),
            (
                CosoPrinciple::QualityInformation,
                "Quality Information",
                CosoComponent::InformationCommunication,
            ),
            (
                CosoPrinciple::InternalCommunication,
                "Internal Communication",
                CosoComponent::InformationCommunication,
            ),
            (
                CosoPrinciple::ExternalCommunication,
                "External Communication",
                CosoComponent::InformationCommunication,
            ),
            (
                CosoPrinciple::OngoingMonitoring,
                "Ongoing Monitoring",
                CosoComponent::MonitoringActivities,
            ),
            (
                CosoPrinciple::DeficiencyEvaluation,
                "Deficiency Evaluation",
                CosoComponent::MonitoringActivities,
            ),
        ];

        for (principle, name, parent_component) in &principles {
            let principle_id = format!("coso_prin_{}", name.replace(' ', "_").replace('&', "and"));
            if self.try_add_node(HypergraphNode {
                id: principle_id.clone(),
                entity_type: "coso_principle".to_string(),
                entity_type_code: type_codes::COSO_PRINCIPLE,
                layer: HypergraphLayer::GovernanceControls,
                external_id: format!("{principle:?}"),
                label: name.to_string(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "principle_number".to_string(),
                        Value::Number(principle.principle_number().into()),
                    );
                    p
                },
                features: vec![principle.principle_number() as f64],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                // Link principle to its parent component
                let comp_key = format!("{parent_component:?}");
                if let Some(comp_id) = self.coso_component_ids.get(&comp_key) {
                    self.edges.push(CrossLayerEdge {
                        source_id: principle_id,
                        source_layer: HypergraphLayer::GovernanceControls,
                        target_id: comp_id.clone(),
                        target_layer: HypergraphLayer::GovernanceControls,
                        edge_type: "CoversCosoPrinciple".to_string(),
                        edge_type_code: type_codes::COVERS_COSO_PRINCIPLE,
                        properties: HashMap::new(),
                    });
                }
            }
        }
    }

    /// Add internal controls as Layer 1 nodes with edges to COSO components.
    pub fn add_controls(&mut self, controls: &[InternalControl]) {
        if !self.config.include_controls {
            return;
        }

        for control in controls {
            let node_id = format!("ctrl_{}", control.control_id);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "internal_control".to_string(),
                entity_type_code: type_codes::INTERNAL_CONTROL,
                layer: HypergraphLayer::GovernanceControls,
                external_id: control.control_id.clone(),
                label: control.control_name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "control_type".to_string(),
                        Value::String(format!("{:?}", control.control_type)),
                    );
                    p.insert(
                        "controlType".to_string(),
                        Value::String(format!("{}", control.control_type).to_lowercase()),
                    );
                    p.insert(
                        "risk_level".to_string(),
                        Value::String(format!("{:?}", control.risk_level)),
                    );
                    p.insert(
                        "is_key_control".to_string(),
                        Value::Bool(control.is_key_control),
                    );
                    p.insert(
                        "isKeyControl".to_string(),
                        Value::Bool(control.is_key_control),
                    );
                    p.insert(
                        "maturity_level".to_string(),
                        Value::String(format!("{:?}", control.maturity_level)),
                    );
                    let effectiveness = match control.maturity_level.level() {
                        4 | 5 => "effective",
                        3 => "partially-effective",
                        _ => "not-tested",
                    };
                    p.insert(
                        "effectiveness".to_string(),
                        Value::String(effectiveness.to_string()),
                    );
                    p.insert(
                        "description".to_string(),
                        Value::String(control.description.clone()),
                    );
                    p.insert(
                        "objective".to_string(),
                        Value::String(control.objective.clone()),
                    );
                    p.insert(
                        "frequency".to_string(),
                        Value::String(format!("{}", control.frequency).to_lowercase()),
                    );
                    p.insert(
                        "owner".to_string(),
                        Value::String(format!("{}", control.owner_role)),
                    );
                    p.insert(
                        "controlId".to_string(),
                        Value::String(control.control_id.clone()),
                    );
                    p.insert(
                        "name".to_string(),
                        Value::String(control.control_name.clone()),
                    );
                    p.insert(
                        "category".to_string(),
                        Value::String(format!("{}", control.control_type)),
                    );
                    p.insert(
                        "automated".to_string(),
                        Value::Bool(matches!(
                            control.control_type,
                            datasynth_core::models::ControlType::Monitoring
                        )),
                    );
                    p.insert(
                        "coso_component".to_string(),
                        Value::String(format!("{:?}", control.coso_component)),
                    );
                    p.insert(
                        "sox_assertion".to_string(),
                        Value::String(format!("{:?}", control.sox_assertion)),
                    );
                    p.insert(
                        "control_scope".to_string(),
                        Value::String(format!("{:?}", control.control_scope)),
                    );
                    p
                },
                features: vec![
                    if control.is_key_control { 1.0 } else { 0.0 },
                    control.maturity_level.level() as f64 / 5.0,
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.control_node_ids
                    .insert(control.control_id.clone(), node_id.clone());

                // Edge: Control → COSO Component
                let comp_key = format!("{:?}", control.coso_component);
                if let Some(comp_id) = self.coso_component_ids.get(&comp_key) {
                    self.edges.push(CrossLayerEdge {
                        source_id: node_id.clone(),
                        source_layer: HypergraphLayer::GovernanceControls,
                        target_id: comp_id.clone(),
                        target_layer: HypergraphLayer::GovernanceControls,
                        edge_type: "ImplementsControl".to_string(),
                        edge_type_code: type_codes::IMPLEMENTS_CONTROL,
                        properties: HashMap::new(),
                    });
                }

                // Edge: Control → SOX Assertion
                if self.config.include_sox {
                    let assertion_id = format!("sox_{:?}", control.sox_assertion).to_lowercase();
                    // Ensure SOX assertion node exists
                    if !self.node_index.contains_key(&assertion_id) {
                        self.try_add_node(HypergraphNode {
                            id: assertion_id.clone(),
                            entity_type: "sox_assertion".to_string(),
                            entity_type_code: type_codes::SOX_ASSERTION,
                            layer: HypergraphLayer::GovernanceControls,
                            external_id: format!("{:?}", control.sox_assertion),
                            label: format!("{:?}", control.sox_assertion),
                            properties: HashMap::new(),
                            features: vec![],
                            is_anomaly: false,
                            anomaly_type: None,
                            is_aggregate: false,
                            aggregate_count: 0,
                        });
                    }
                    self.edges.push(CrossLayerEdge {
                        source_id: node_id,
                        source_layer: HypergraphLayer::GovernanceControls,
                        target_id: assertion_id,
                        target_layer: HypergraphLayer::GovernanceControls,
                        edge_type: "EnforcesAssertion".to_string(),
                        edge_type_code: type_codes::ENFORCES_ASSERTION,
                        properties: HashMap::new(),
                    });
                }
            }
        }
    }

    /// Add vendor master data as Layer 1 nodes.
    pub fn add_vendors(&mut self, vendors: &[Vendor]) {
        if !self.config.include_vendors {
            return;
        }

        for vendor in vendors {
            let node_id = format!("vnd_{}", vendor.vendor_id);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "vendor".to_string(),
                entity_type_code: type_codes::VENDOR,
                layer: HypergraphLayer::GovernanceControls,
                external_id: vendor.vendor_id.clone(),
                label: vendor.name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "vendor_type".to_string(),
                        Value::String(format!("{:?}", vendor.vendor_type)),
                    );
                    p.insert("country".to_string(), Value::String(vendor.country.clone()));
                    p.insert("is_active".to_string(), Value::Bool(vendor.is_active));
                    p
                },
                features: vec![if vendor.is_active { 1.0 } else { 0.0 }],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.vendor_node_ids
                    .insert(vendor.vendor_id.clone(), node_id);
            }
        }
    }

    /// Add customer master data as Layer 1 nodes.
    pub fn add_customers(&mut self, customers: &[Customer]) {
        if !self.config.include_customers {
            return;
        }

        for customer in customers {
            let node_id = format!("cust_{}", customer.customer_id);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "customer".to_string(),
                entity_type_code: type_codes::CUSTOMER,
                layer: HypergraphLayer::GovernanceControls,
                external_id: customer.customer_id.clone(),
                label: customer.name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "customer_type".to_string(),
                        Value::String(format!("{:?}", customer.customer_type)),
                    );
                    p.insert(
                        "country".to_string(),
                        Value::String(customer.country.clone()),
                    );
                    p.insert(
                        "credit_rating".to_string(),
                        Value::String(format!("{:?}", customer.credit_rating)),
                    );
                    p
                },
                features: vec![if customer.is_active { 1.0 } else { 0.0 }],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.customer_node_ids
                    .insert(customer.customer_id.clone(), node_id);
            }
        }
    }

    /// Add employee/organizational nodes as Layer 1 nodes.
    pub fn add_employees(&mut self, employees: &[Employee]) {
        if !self.config.include_employees {
            return;
        }

        for employee in employees {
            let node_id = format!("emp_{}", employee.employee_id);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "employee".to_string(),
                entity_type_code: type_codes::EMPLOYEE,
                layer: HypergraphLayer::GovernanceControls,
                external_id: employee.employee_id.clone(),
                label: employee.display_name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "persona".to_string(),
                        Value::String(employee.persona.to_string()),
                    );
                    p.insert(
                        "job_level".to_string(),
                        Value::String(format!("{:?}", employee.job_level)),
                    );
                    p.insert(
                        "company_code".to_string(),
                        Value::String(employee.company_code.clone()),
                    );
                    p.insert(
                        "fullName".to_string(),
                        Value::String(employee.display_name.clone()),
                    );
                    p.insert("email".to_string(), Value::String(employee.email.clone()));
                    p.insert(
                        "department".to_string(),
                        Value::String(employee.department_id.clone().unwrap_or_default()),
                    );
                    p.insert(
                        "job_title".to_string(),
                        Value::String(employee.job_title.clone()),
                    );
                    p.insert(
                        "status".to_string(),
                        Value::String(format!("{:?}", employee.status)),
                    );
                    p
                },
                features: vec![employee
                    .approval_limit
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.employee_node_ids
                    .insert(employee.employee_id.clone(), node_id);
            }
        }
    }

    /// Add material master data as Layer 3 nodes.
    pub fn add_materials(&mut self, materials: &[Material]) {
        for mat in materials {
            let node_id = format!("mat_{}", mat.material_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "material".to_string(),
                entity_type_code: type_codes::MATERIAL,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: mat.material_id.clone(),
                label: format!("{} ({})", mat.description, mat.material_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "material_type".to_string(),
                        Value::String(format!("{:?}", mat.material_type)),
                    );
                    p.insert(
                        "material_group".to_string(),
                        Value::String(format!("{:?}", mat.material_group)),
                    );
                    let cost: f64 = mat.standard_cost.to_string().parse().unwrap_or(0.0);
                    p.insert("standard_cost".to_string(), serde_json::json!(cost));
                    p
                },
                features: vec![mat
                    .standard_cost
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add fixed asset master data as Layer 3 nodes.
    pub fn add_fixed_assets(&mut self, assets: &[FixedAsset]) {
        for asset in assets {
            let node_id = format!("fa_{}", asset.asset_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "fixed_asset".to_string(),
                entity_type_code: type_codes::FIXED_ASSET,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: asset.asset_id.clone(),
                label: format!("{} ({})", asset.description, asset.asset_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "asset_class".to_string(),
                        Value::String(format!("{:?}", asset.asset_class)),
                    );
                    p.insert(
                        "company_code".to_string(),
                        Value::String(asset.company_code.clone()),
                    );
                    if let Some(ref cc) = asset.cost_center {
                        p.insert("cost_center".to_string(), Value::String(cc.clone()));
                    }
                    let cost: f64 = asset.acquisition_cost.to_string().parse().unwrap_or(0.0);
                    p.insert("acquisition_cost".to_string(), serde_json::json!(cost));
                    p
                },
                features: vec![asset
                    .acquisition_cost
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add GL accounts as Layer 3 nodes.
    pub fn add_accounts(&mut self, coa: &ChartOfAccounts) {
        if !self.config.include_accounts {
            return;
        }

        for account in &coa.accounts {
            let node_id = format!("acct_{}", account.account_number);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "account".to_string(),
                entity_type_code: type_codes::ACCOUNT,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: account.account_number.clone(),
                label: account.short_description.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "account_type".to_string(),
                        Value::String(format!("{:?}", account.account_type)),
                    );
                    p.insert(
                        "is_control_account".to_string(),
                        Value::Bool(account.is_control_account),
                    );
                    p.insert("is_postable".to_string(), Value::Bool(account.is_postable));
                    p
                },
                features: vec![
                    account_type_feature(&account.account_type),
                    if account.is_control_account { 1.0 } else { 0.0 },
                    if account.normal_debit_balance {
                        1.0
                    } else {
                        0.0
                    },
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.account_node_ids
                    .insert(account.account_number.clone(), node_id);
            }
        }
    }

    /// Add journal entries as Layer 3 hyperedges.
    ///
    /// Each journal entry becomes a hyperedge connecting its debit and credit accounts.
    pub fn add_journal_entries_as_hyperedges(&mut self, entries: &[JournalEntry]) {
        if !self.config.je_as_hyperedges {
            return;
        }

        for entry in entries {
            let mut participants = Vec::new();

            for line in &entry.lines {
                let account_id = format!("acct_{}", line.gl_account);

                // Ensure account node exists (might not if CoA was incomplete)
                if !self.node_index.contains_key(&account_id) {
                    self.try_add_node(HypergraphNode {
                        id: account_id.clone(),
                        entity_type: "account".to_string(),
                        entity_type_code: type_codes::ACCOUNT,
                        layer: HypergraphLayer::AccountingNetwork,
                        external_id: line.gl_account.clone(),
                        label: line
                            .account_description
                            .clone()
                            .unwrap_or_else(|| line.gl_account.clone()),
                        properties: HashMap::new(),
                        features: vec![],
                        is_anomaly: false,
                        anomaly_type: None,
                        is_aggregate: false,
                        aggregate_count: 0,
                    });
                    self.account_node_ids
                        .insert(line.gl_account.clone(), account_id.clone());
                }

                let amount: f64 = if !line.debit_amount.is_zero() {
                    line.debit_amount.to_string().parse().unwrap_or(0.0)
                } else {
                    line.credit_amount.to_string().parse().unwrap_or(0.0)
                };

                let role = if !line.debit_amount.is_zero() {
                    "debit"
                } else {
                    "credit"
                };

                participants.push(HyperedgeParticipant {
                    node_id: account_id,
                    role: role.to_string(),
                    weight: Some(amount),
                });
            }

            if participants.is_empty() {
                continue;
            }

            let doc_id = entry.header.document_id.to_string();
            let subtype = entry
                .header
                .business_process
                .as_ref()
                .map(|bp| format!("{bp:?}"))
                .unwrap_or_else(|| "General".to_string());

            self.hyperedges.push(Hyperedge {
                id: format!("je_{doc_id}"),
                hyperedge_type: "JournalEntry".to_string(),
                subtype,
                participants,
                layer: HypergraphLayer::AccountingNetwork,
                properties: {
                    let mut p = HashMap::new();
                    p.insert("document_id".to_string(), Value::String(doc_id));
                    p.insert(
                        "company_code".to_string(),
                        Value::String(entry.header.company_code.clone()),
                    );
                    p.insert(
                        "document_type".to_string(),
                        Value::String(entry.header.document_type.clone()),
                    );
                    p.insert(
                        "created_by".to_string(),
                        Value::String(entry.header.created_by.clone()),
                    );
                    p
                },
                timestamp: Some(entry.header.posting_date),
                is_anomaly: entry.header.is_anomaly || entry.header.is_fraud,
                anomaly_type: entry
                    .header
                    .anomaly_type
                    .clone()
                    .or_else(|| entry.header.fraud_type.as_ref().map(|ft| format!("{ft:?}"))),
                features: compute_je_features(entry),
            });
        }
    }

    /// Add journal entries as standalone Layer 3 nodes.
    ///
    /// Creates a node per JE with amount, date, anomaly info, and line count.
    /// Use alongside `add_journal_entries_as_hyperedges` so the dashboard can
    /// count JE nodes while the accounting network still has proper hyperedges.
    pub fn add_journal_entry_nodes(&mut self, entries: &[JournalEntry]) {
        for entry in entries {
            let node_id = format!("je_{}", entry.header.document_id);
            let total_amount: f64 = entry
                .lines
                .iter()
                .map(|l| l.debit_amount.to_string().parse::<f64>().unwrap_or(0.0))
                .sum();

            let is_anomaly = entry.header.is_anomaly || entry.header.is_fraud;
            let anomaly_type = entry
                .header
                .anomaly_type
                .clone()
                .or_else(|| entry.header.fraud_type.as_ref().map(|ft| format!("{ft:?}")));

            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "journal_entry".to_string(),
                entity_type_code: type_codes::JOURNAL_ENTRY,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: entry.header.document_id.to_string(),
                label: format!("JE-{}", entry.header.document_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "amount".into(),
                        Value::Number(
                            serde_json::Number::from_f64(total_amount)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    );
                    p.insert(
                        "date".into(),
                        Value::String(entry.header.posting_date.to_string()),
                    );
                    p.insert(
                        "company_code".into(),
                        Value::String(entry.header.company_code.clone()),
                    );
                    p.insert(
                        "line_count".into(),
                        Value::Number((entry.lines.len() as u64).into()),
                    );
                    p.insert("is_anomaly".into(), Value::Bool(is_anomaly));
                    if let Some(ref at) = anomaly_type {
                        p.insert("anomaly_type".into(), Value::String(at.clone()));
                    }
                    p
                },
                features: vec![total_amount / 100_000.0],
                is_anomaly,
                anomaly_type,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add P2P document chains as Layer 2 nodes.
    ///
    /// If a vendor has more documents than the threshold, they're aggregated into pool nodes.
    pub fn add_p2p_documents(
        &mut self,
        purchase_orders: &[datasynth_core::models::documents::PurchaseOrder],
        goods_receipts: &[datasynth_core::models::documents::GoodsReceipt],
        vendor_invoices: &[datasynth_core::models::documents::VendorInvoice],
        payments: &[datasynth_core::models::documents::Payment],
    ) {
        if !self.config.include_p2p {
            return;
        }

        // Count documents per vendor for aggregation decisions
        let mut vendor_doc_counts: HashMap<String, usize> = HashMap::new();
        for po in purchase_orders {
            *vendor_doc_counts.entry(po.vendor_id.clone()).or_insert(0) += 1;
        }

        let threshold = self.config.docs_per_counterparty_threshold;
        let should_aggregate = matches!(
            self.config.aggregation_strategy,
            AggregationStrategy::PoolByCounterparty
        );

        // Track which vendors need pool nodes
        let vendors_needing_pools: Vec<String> = if should_aggregate {
            vendor_doc_counts
                .iter()
                .filter(|(_, count)| **count > threshold)
                .map(|(vid, _)| vid.clone())
                .collect()
        } else {
            Vec::new()
        };

        // Create pool nodes for high-volume vendors
        for vendor_id in &vendors_needing_pools {
            let count = vendor_doc_counts[vendor_id];
            let pool_id = format!("pool_p2p_{vendor_id}");
            if self.try_add_node(HypergraphNode {
                id: pool_id.clone(),
                entity_type: "p2p_pool".to_string(),
                entity_type_code: type_codes::POOL_NODE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("pool_p2p_{vendor_id}"),
                label: format!("P2P Pool ({vendor_id}): {count} docs"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("vendor_id".to_string(), Value::String(vendor_id.clone()));
                    p.insert("document_count".to_string(), Value::Number(count.into()));
                    p
                },
                features: vec![count as f64],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: true,
                aggregate_count: count,
            }) {
                self.doc_counterparty_links.push((
                    pool_id,
                    "vendor".to_string(),
                    vendor_id.clone(),
                ));
            }
            self.aggregate_count += 1;
        }

        // Add individual PO nodes (if not pooled)
        for po in purchase_orders {
            if should_aggregate && vendors_needing_pools.contains(&po.vendor_id) {
                continue; // Pooled
            }

            let doc_id = &po.header.document_id;
            let node_id = format!("po_{doc_id}");
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "purchase_order".to_string(),
                entity_type_code: type_codes::PURCHASE_ORDER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("PO {doc_id}"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("vendor_id".to_string(), Value::String(po.vendor_id.clone()));
                    p.insert(
                        "company_code".to_string(),
                        Value::String(po.header.company_code.clone()),
                    );
                    p
                },
                features: vec![po
                    .total_net_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.doc_counterparty_links.push((
                    node_id,
                    "vendor".to_string(),
                    po.vendor_id.clone(),
                ));
            }
        }

        // Add GR nodes
        for gr in goods_receipts {
            let vendor_id = gr.vendor_id.as_deref().unwrap_or("UNKNOWN");
            if should_aggregate && vendors_needing_pools.contains(&vendor_id.to_string()) {
                continue;
            }
            let doc_id = &gr.header.document_id;
            let node_id = format!("gr_{doc_id}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "goods_receipt".to_string(),
                entity_type_code: type_codes::GOODS_RECEIPT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("GR {doc_id}"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "vendor_id".to_string(),
                        Value::String(vendor_id.to_string()),
                    );
                    p
                },
                features: vec![gr
                    .total_value
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        // Add vendor invoice nodes
        for inv in vendor_invoices {
            if should_aggregate && vendors_needing_pools.contains(&inv.vendor_id) {
                continue;
            }
            let doc_id = &inv.header.document_id;
            let node_id = format!("vinv_{doc_id}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "vendor_invoice".to_string(),
                entity_type_code: type_codes::VENDOR_INVOICE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("VI {doc_id}"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "vendor_id".to_string(),
                        Value::String(inv.vendor_id.clone()),
                    );
                    p
                },
                features: vec![inv
                    .payable_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        // Add payment nodes
        for pmt in payments {
            let doc_id = &pmt.header.document_id;
            let node_id = format!("pmt_{doc_id}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "payment".to_string(),
                entity_type_code: type_codes::PAYMENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("PMT {doc_id}"),
                properties: HashMap::new(),
                features: vec![pmt.amount.to_string().parse::<f64>().unwrap_or(0.0).ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add O2C document chains as Layer 2 nodes.
    pub fn add_o2c_documents(
        &mut self,
        sales_orders: &[datasynth_core::models::documents::SalesOrder],
        deliveries: &[datasynth_core::models::documents::Delivery],
        customer_invoices: &[datasynth_core::models::documents::CustomerInvoice],
    ) {
        if !self.config.include_o2c {
            return;
        }

        // Count docs per customer for aggregation
        let mut customer_doc_counts: HashMap<String, usize> = HashMap::new();
        for so in sales_orders {
            *customer_doc_counts
                .entry(so.customer_id.clone())
                .or_insert(0) += 1;
        }

        let threshold = self.config.docs_per_counterparty_threshold;
        let should_aggregate = matches!(
            self.config.aggregation_strategy,
            AggregationStrategy::PoolByCounterparty
        );

        let customers_needing_pools: Vec<String> = if should_aggregate {
            customer_doc_counts
                .iter()
                .filter(|(_, count)| **count > threshold)
                .map(|(cid, _)| cid.clone())
                .collect()
        } else {
            Vec::new()
        };

        // Create pool nodes
        for customer_id in &customers_needing_pools {
            let count = customer_doc_counts[customer_id];
            let pool_id = format!("pool_o2c_{customer_id}");
            if self.try_add_node(HypergraphNode {
                id: pool_id.clone(),
                entity_type: "o2c_pool".to_string(),
                entity_type_code: type_codes::POOL_NODE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("pool_o2c_{customer_id}"),
                label: format!("O2C Pool ({customer_id}): {count} docs"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "customer_id".to_string(),
                        Value::String(customer_id.clone()),
                    );
                    p.insert("document_count".to_string(), Value::Number(count.into()));
                    p
                },
                features: vec![count as f64],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: true,
                aggregate_count: count,
            }) {
                self.doc_counterparty_links.push((
                    pool_id,
                    "customer".to_string(),
                    customer_id.clone(),
                ));
            }
            self.aggregate_count += 1;
        }

        for so in sales_orders {
            if should_aggregate && customers_needing_pools.contains(&so.customer_id) {
                continue;
            }
            let doc_id = &so.header.document_id;
            let node_id = format!("so_{doc_id}");
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "sales_order".to_string(),
                entity_type_code: type_codes::SALES_ORDER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("SO {doc_id}"),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "customer_id".to_string(),
                        Value::String(so.customer_id.clone()),
                    );
                    p
                },
                features: vec![so
                    .total_net_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.doc_counterparty_links.push((
                    node_id,
                    "customer".to_string(),
                    so.customer_id.clone(),
                ));
            }
        }

        for del in deliveries {
            if should_aggregate && customers_needing_pools.contains(&del.customer_id) {
                continue;
            }
            let doc_id = &del.header.document_id;
            let node_id = format!("del_{doc_id}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "delivery".to_string(),
                entity_type_code: type_codes::DELIVERY,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("DEL {doc_id}"),
                properties: HashMap::new(),
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for inv in customer_invoices {
            if should_aggregate && customers_needing_pools.contains(&inv.customer_id) {
                continue;
            }
            let doc_id = &inv.header.document_id;
            let node_id = format!("cinv_{doc_id}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "customer_invoice".to_string(),
                entity_type_code: type_codes::CUSTOMER_INVOICE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("CI {doc_id}"),
                properties: HashMap::new(),
                features: vec![inv
                    .total_gross_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add S2C (Source-to-Contract) documents as Layer 2 nodes.
    pub fn add_s2c_documents(
        &mut self,
        projects: &[SourcingProject],
        qualifications: &[SupplierQualification],
        rfx_events: &[RfxEvent],
        bids: &[SupplierBid],
        evaluations: &[BidEvaluation],
        contracts: &[ProcurementContract],
    ) {
        if !self.config.include_s2c {
            return;
        }
        for p in projects {
            let node_id = format!("s2c_proj_{}", p.project_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "sourcing_project".into(),
                entity_type_code: type_codes::SOURCING_PROJECT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: p.project_id.clone(),
                label: format!("SPRJ {}", p.project_id),
                properties: HashMap::new(),
                features: vec![p
                    .estimated_annual_spend
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for q in qualifications {
            let node_id = format!("s2c_qual_{}", q.qualification_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "supplier_qualification".into(),
                entity_type_code: type_codes::SUPPLIER_QUALIFICATION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: q.qualification_id.clone(),
                label: format!("SQUAL {}", q.qualification_id),
                properties: HashMap::new(),
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for r in rfx_events {
            let node_id = format!("s2c_rfx_{}", r.rfx_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "rfx_event".into(),
                entity_type_code: type_codes::RFX_EVENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: r.rfx_id.clone(),
                label: format!("RFX {}", r.rfx_id),
                properties: HashMap::new(),
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for b in bids {
            let node_id = format!("s2c_bid_{}", b.bid_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "supplier_bid".into(),
                entity_type_code: type_codes::SUPPLIER_BID,
                layer: HypergraphLayer::ProcessEvents,
                external_id: b.bid_id.clone(),
                label: format!("BID {}", b.bid_id),
                properties: HashMap::new(),
                features: vec![b
                    .total_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for e in evaluations {
            let node_id = format!("s2c_eval_{}", e.evaluation_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "bid_evaluation".into(),
                entity_type_code: type_codes::BID_EVALUATION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: e.evaluation_id.clone(),
                label: format!("BEVAL {}", e.evaluation_id),
                properties: HashMap::new(),
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for c in contracts {
            let node_id = format!("s2c_ctr_{}", c.contract_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "procurement_contract".into(),
                entity_type_code: type_codes::PROCUREMENT_CONTRACT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: c.contract_id.clone(),
                label: format!("CTR {}", c.contract_id),
                properties: HashMap::new(),
                features: vec![c
                    .total_value
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
            // Track vendor for cross-layer edges
            self.doc_counterparty_links.push((
                format!("s2c_ctr_{}", c.contract_id),
                "vendor".into(),
                c.vendor_id.clone(),
            ));
        }
    }

    /// Add H2R (Hire-to-Retire) documents as Layer 2 nodes.
    pub fn add_h2r_documents(
        &mut self,
        payroll_runs: &[PayrollRun],
        time_entries: &[TimeEntry],
        expense_reports: &[ExpenseReport],
    ) {
        if !self.config.include_h2r {
            return;
        }
        for pr in payroll_runs {
            let node_id = format!("h2r_pay_{}", pr.payroll_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "payroll_run".into(),
                entity_type_code: type_codes::PAYROLL_RUN,
                layer: HypergraphLayer::ProcessEvents,
                external_id: pr.payroll_id.clone(),
                label: format!("PAY {}", pr.payroll_id),
                properties: HashMap::new(),
                features: vec![pr
                    .total_gross
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for te in time_entries {
            let node_id = format!("h2r_time_{}", te.entry_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "time_entry".into(),
                entity_type_code: type_codes::TIME_ENTRY,
                layer: HypergraphLayer::ProcessEvents,
                external_id: te.entry_id.clone(),
                label: format!("TIME {}", te.entry_id),
                properties: HashMap::new(),
                features: vec![te.hours_regular + te.hours_overtime],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for er in expense_reports {
            let node_id = format!("h2r_exp_{}", er.report_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "expense_report".into(),
                entity_type_code: type_codes::EXPENSE_REPORT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: er.report_id.clone(),
                label: format!("EXP {}", er.report_id),
                properties: HashMap::new(),
                features: vec![er
                    .total_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add MFG (Manufacturing) documents as Layer 2 nodes.
    pub fn add_mfg_documents(
        &mut self,
        production_orders: &[ProductionOrder],
        quality_inspections: &[QualityInspection],
        cycle_counts: &[CycleCount],
    ) {
        if !self.config.include_mfg {
            return;
        }
        for po in production_orders {
            let node_id = format!("mfg_po_{}", po.order_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "production_order".into(),
                entity_type_code: type_codes::PRODUCTION_ORDER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: po.order_id.clone(),
                label: format!("PROD {}", po.order_id),
                properties: HashMap::new(),
                features: vec![po
                    .planned_quantity
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for qi in quality_inspections {
            let node_id = format!("mfg_qi_{}", qi.inspection_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "quality_inspection".into(),
                entity_type_code: type_codes::QUALITY_INSPECTION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: qi.inspection_id.clone(),
                label: format!("QI {}", qi.inspection_id),
                properties: HashMap::new(),
                features: vec![qi.defect_rate],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for cc in cycle_counts {
            let node_id = format!("mfg_cc_{}", cc.count_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "cycle_count".into(),
                entity_type_code: type_codes::CYCLE_COUNT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: cc.count_id.clone(),
                label: format!("CC {}", cc.count_id),
                properties: HashMap::new(),
                features: vec![cc.variance_rate],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add Banking documents as Layer 2 nodes.
    pub fn add_bank_documents(
        &mut self,
        customers: &[BankingCustomer],
        accounts: &[BankAccount],
        transactions: &[BankTransaction],
    ) {
        if !self.config.include_bank {
            return;
        }
        for cust in customers {
            let cid = cust.customer_id.to_string();
            let node_id = format!("bank_cust_{cid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "banking_customer".into(),
                entity_type_code: type_codes::BANKING_CUSTOMER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: cid,
                label: format!("BCUST {}", cust.customer_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "customer_type".into(),
                        Value::String(format!("{:?}", cust.customer_type)),
                    );
                    p.insert("name".into(), Value::String(cust.name.legal_name.clone()));
                    p.insert(
                        "residence_country".into(),
                        Value::String(cust.residence_country.clone()),
                    );
                    p.insert(
                        "risk_tier".into(),
                        Value::String(format!("{:?}", cust.risk_tier)),
                    );
                    p.insert("is_pep".into(), Value::Bool(cust.is_pep));
                    p
                },
                features: vec![],
                is_anomaly: cust.is_mule,
                anomaly_type: if cust.is_mule {
                    Some("mule_account".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for acct in accounts {
            let aid = acct.account_id.to_string();
            let node_id = format!("bank_acct_{aid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "bank_account".into(),
                entity_type_code: type_codes::BANK_ACCOUNT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: aid,
                label: format!("BACCT {}", acct.account_number),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "account_type".into(),
                        Value::String(format!("{:?}", acct.account_type)),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", acct.status)));
                    p.insert("currency".into(), Value::String(acct.currency.clone()));
                    let balance: f64 = acct.current_balance.to_string().parse().unwrap_or(0.0);
                    p.insert("balance".into(), serde_json::json!(balance));
                    p.insert(
                        "account_number".into(),
                        Value::String(acct.account_number.clone()),
                    );
                    p
                },
                features: vec![acct
                    .current_balance
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: acct.is_mule_account,
                anomaly_type: if acct.is_mule_account {
                    Some("mule_account".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for txn in transactions {
            let tid = txn.transaction_id.to_string();
            let node_id = format!("bank_txn_{tid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "bank_transaction".into(),
                entity_type_code: type_codes::BANK_TRANSACTION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: tid,
                label: format!("BTXN {}", txn.reference),
                properties: {
                    let mut p = HashMap::new();
                    let amount: f64 = txn.amount.to_string().parse().unwrap_or(0.0);
                    p.insert("amount".into(), serde_json::json!(amount));
                    p.insert("currency".into(), Value::String(txn.currency.clone()));
                    p.insert("reference".into(), Value::String(txn.reference.clone()));
                    p.insert(
                        "direction".into(),
                        Value::String(format!("{:?}", txn.direction)),
                    );
                    p.insert(
                        "channel".into(),
                        Value::String(format!("{:?}", txn.channel)),
                    );
                    p.insert(
                        "category".into(),
                        Value::String(format!("{:?}", txn.category)),
                    );
                    p.insert(
                        "transaction_type".into(),
                        Value::String(txn.transaction_type.clone()),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", txn.status)));
                    if txn.is_suspicious {
                        p.insert("isAnomalous".into(), Value::Bool(true));
                        p.insert("is_suspicious".into(), Value::Bool(true));
                        if let Some(ref reason) = txn.suspicion_reason {
                            p.insert(
                                "suspicion_reason".into(),
                                Value::String(format!("{reason:?}")),
                            );
                        }
                        if let Some(ref stage) = txn.laundering_stage {
                            p.insert(
                                "laundering_stage".into(),
                                Value::String(format!("{stage:?}")),
                            );
                        }
                    }
                    p
                },
                features: vec![txn
                    .amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: txn.is_suspicious,
                anomaly_type: txn.suspicion_reason.as_ref().map(|r| format!("{r:?}")),
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add Audit documents as Layer 2 nodes.
    #[allow(clippy::too_many_arguments)]
    pub fn add_audit_documents(
        &mut self,
        engagements: &[AuditEngagement],
        workpapers: &[Workpaper],
        findings: &[AuditFinding],
        evidence: &[AuditEvidence],
        risks: &[RiskAssessment],
        judgments: &[ProfessionalJudgment],
    ) {
        if !self.config.include_audit {
            return;
        }
        for eng in engagements {
            let eid = eng.engagement_id.to_string();
            let node_id = format!("audit_eng_{eid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "audit_engagement".into(),
                entity_type_code: type_codes::AUDIT_ENGAGEMENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: eid,
                label: format!("AENG {}", eng.engagement_ref),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "engagement_ref".into(),
                        Value::String(eng.engagement_ref.clone()),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", eng.status)));
                    p.insert(
                        "engagement_type".into(),
                        Value::String(format!("{:?}", eng.engagement_type)),
                    );
                    p.insert("client_name".into(), Value::String(eng.client_name.clone()));
                    p.insert("fiscal_year".into(), serde_json::json!(eng.fiscal_year));
                    let mat: f64 = eng.materiality.to_string().parse().unwrap_or(0.0);
                    p.insert("materiality".into(), serde_json::json!(mat));
                    p.insert(
                        "fieldwork_start".into(),
                        Value::String(eng.fieldwork_start.to_string()),
                    );
                    p.insert(
                        "fieldwork_end".into(),
                        Value::String(eng.fieldwork_end.to_string()),
                    );
                    p
                },
                features: vec![eng
                    .materiality
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for wp in workpapers {
            let wid = wp.workpaper_id.to_string();
            let node_id = format!("audit_wp_{wid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "workpaper".into(),
                entity_type_code: type_codes::WORKPAPER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: wid,
                label: format!("WP {}", wp.workpaper_ref),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "workpaper_ref".into(),
                        Value::String(wp.workpaper_ref.clone()),
                    );
                    p.insert("title".into(), Value::String(wp.title.clone()));
                    p.insert("status".into(), Value::String(format!("{:?}", wp.status)));
                    p.insert("section".into(), Value::String(format!("{:?}", wp.section)));
                    p
                },
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for f in findings {
            let fid = f.finding_id.to_string();
            let node_id = format!("audit_find_{fid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "audit_finding".into(),
                entity_type_code: type_codes::AUDIT_FINDING,
                layer: HypergraphLayer::ProcessEvents,
                external_id: fid,
                label: format!("AFIND {}", f.finding_ref),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("finding_ref".into(), Value::String(f.finding_ref.clone()));
                    p.insert("title".into(), Value::String(f.title.clone()));
                    p.insert("description".into(), Value::String(f.condition.clone()));
                    p.insert(
                        "severity".into(),
                        Value::String(format!("{:?}", f.severity)),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", f.status)));
                    p.insert(
                        "finding_type".into(),
                        Value::String(format!("{:?}", f.finding_type)),
                    );
                    p
                },
                features: vec![f.severity.score() as f64 / 5.0],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for ev in evidence {
            let evid = ev.evidence_id.to_string();
            let node_id = format!("audit_ev_{evid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "audit_evidence".into(),
                entity_type_code: type_codes::AUDIT_EVIDENCE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: evid,
                label: format!("AEV {}", ev.evidence_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "evidence_type".into(),
                        Value::String(format!("{:?}", ev.evidence_type)),
                    );
                    p.insert("description".into(), Value::String(ev.description.clone()));
                    p.insert(
                        "source_type".into(),
                        Value::String(format!("{:?}", ev.source_type)),
                    );
                    p.insert(
                        "reliability".into(),
                        Value::String(format!(
                            "{:?}",
                            ev.reliability_assessment.overall_reliability
                        )),
                    );
                    p
                },
                features: vec![ev.reliability_assessment.overall_reliability.score() as f64 / 3.0],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for r in risks {
            let rid = r.risk_id.to_string();
            let node_id = format!("audit_risk_{rid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "risk_assessment".into(),
                entity_type_code: type_codes::RISK_ASSESSMENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: rid,
                label: format!("ARISK {}", r.risk_ref),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("status".into(), Value::String("active".into()));
                    p.insert("risk_ref".into(), Value::String(r.risk_ref.clone()));
                    p.insert("name".into(), Value::String(r.risk_ref.clone()));
                    p.insert("description".into(), Value::String(r.description.clone()));
                    p.insert(
                        "category".into(),
                        Value::String(format!("{:?}", r.risk_category)),
                    );
                    p.insert(
                        "account_or_process".into(),
                        Value::String(r.account_or_process.clone()),
                    );
                    // Risk levels as lowercase strings for dashboard consumption
                    let inherent = match r.inherent_risk {
                        datasynth_core::models::audit::RiskLevel::Low => "low",
                        datasynth_core::models::audit::RiskLevel::Medium => "medium",
                        datasynth_core::models::audit::RiskLevel::High => "high",
                        datasynth_core::models::audit::RiskLevel::Significant => "critical",
                    };
                    let control = match r.control_risk {
                        datasynth_core::models::audit::RiskLevel::Low => "low",
                        datasynth_core::models::audit::RiskLevel::Medium => "medium",
                        datasynth_core::models::audit::RiskLevel::High => "high",
                        datasynth_core::models::audit::RiskLevel::Significant => "critical",
                    };
                    p.insert("inherentImpact".into(), Value::String(inherent.into()));
                    p.insert("inherentLikelihood".into(), Value::String(inherent.into()));
                    p.insert("residualImpact".into(), Value::String(control.into()));
                    p.insert("residualLikelihood".into(), Value::String(control.into()));
                    p.insert(
                        "riskScore".into(),
                        serde_json::json!(r.inherent_risk.score() as f64 * 25.0),
                    );
                    p.insert("owner".into(), Value::String(r.assessed_by.clone()));
                    p.insert("isSignificant".into(), Value::Bool(r.is_significant_risk));
                    p.insert(
                        "is_significant_risk".into(),
                        Value::Bool(r.is_significant_risk),
                    );
                    p.insert(
                        "response_nature".into(),
                        Value::String(format!("{:?}", r.response_nature)),
                    );
                    p
                },
                features: vec![
                    r.inherent_risk.score() as f64 / 4.0,
                    r.control_risk.score() as f64 / 4.0,
                    if r.is_significant_risk { 1.0 } else { 0.0 },
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
        for j in judgments {
            let jid = j.judgment_id.to_string();
            let node_id = format!("audit_judg_{jid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "professional_judgment".into(),
                entity_type_code: type_codes::PROFESSIONAL_JUDGMENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: jid,
                label: format!("AJUDG {}", j.judgment_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("judgment_ref".into(), Value::String(j.judgment_ref.clone()));
                    p.insert("subject".into(), Value::String(j.subject.clone()));
                    p.insert(
                        "description".into(),
                        Value::String(j.issue_description.clone()),
                    );
                    p.insert("conclusion".into(), Value::String(j.conclusion.clone()));
                    p.insert(
                        "judgment_type".into(),
                        Value::String(format!("{:?}", j.judgment_type)),
                    );
                    p
                },
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add Bank Reconciliation documents as Layer 2 nodes.
    pub fn add_bank_recon_documents(&mut self, reconciliations: &[BankReconciliation]) {
        if !self.config.include_r2r {
            return;
        }
        for recon in reconciliations {
            let node_id = format!("recon_{}", recon.reconciliation_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "bank_reconciliation".into(),
                entity_type_code: type_codes::BANK_RECONCILIATION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: recon.reconciliation_id.clone(),
                label: format!("RECON {}", recon.reconciliation_id),
                properties: HashMap::new(),
                features: vec![recon
                    .bank_ending_balance
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
            for line in &recon.statement_lines {
                let node_id = format!("recon_line_{}", line.line_id);
                self.try_add_node(HypergraphNode {
                    id: node_id,
                    entity_type: "bank_statement_line".into(),
                    entity_type_code: type_codes::BANK_STATEMENT_LINE,
                    layer: HypergraphLayer::ProcessEvents,
                    external_id: line.line_id.clone(),
                    label: format!("BSL {}", line.line_id),
                    properties: HashMap::new(),
                    features: vec![line
                        .amount
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p()],
                    is_anomaly: false,
                    anomaly_type: None,
                    is_aggregate: false,
                    aggregate_count: 0,
                });
            }
            for item in &recon.reconciling_items {
                let node_id = format!("recon_item_{}", item.item_id);
                self.try_add_node(HypergraphNode {
                    id: node_id,
                    entity_type: "reconciling_item".into(),
                    entity_type_code: type_codes::RECONCILING_ITEM,
                    layer: HypergraphLayer::ProcessEvents,
                    external_id: item.item_id.clone(),
                    label: format!("RITEM {}", item.item_id),
                    properties: HashMap::new(),
                    features: vec![item
                        .amount
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p()],
                    is_anomaly: false,
                    anomaly_type: None,
                    is_aggregate: false,
                    aggregate_count: 0,
                });
            }
        }
    }

    /// Add OCPM events as hyperedges connecting their participating objects.
    pub fn add_ocpm_events(&mut self, event_log: &datasynth_ocpm::OcpmEventLog) {
        if !self.config.events_as_hyperedges {
            return;
        }
        for event in &event_log.events {
            let participants: Vec<HyperedgeParticipant> = event
                .object_refs
                .iter()
                .map(|obj_ref| {
                    let node_id = format!("ocpm_obj_{}", obj_ref.object_id);
                    // Ensure the object node exists
                    self.try_add_node(HypergraphNode {
                        id: node_id.clone(),
                        entity_type: "ocpm_object".into(),
                        entity_type_code: type_codes::OCPM_EVENT,
                        layer: HypergraphLayer::ProcessEvents,
                        external_id: obj_ref.object_id.to_string(),
                        label: format!("OBJ {}", obj_ref.object_type_id),
                        properties: HashMap::new(),
                        features: vec![],
                        is_anomaly: false,
                        anomaly_type: None,
                        is_aggregate: false,
                        aggregate_count: 0,
                    });
                    HyperedgeParticipant {
                        node_id,
                        role: format!("{:?}", obj_ref.qualifier),
                        weight: None,
                    }
                })
                .collect();

            if !participants.is_empty() {
                let mut props = HashMap::new();
                props.insert(
                    "activity_id".into(),
                    Value::String(event.activity_id.clone()),
                );
                props.insert(
                    "timestamp".into(),
                    Value::String(event.timestamp.to_rfc3339()),
                );
                if !event.resource_id.is_empty() {
                    props.insert("resource".into(), Value::String(event.resource_id.clone()));
                }

                self.hyperedges.push(Hyperedge {
                    id: format!("ocpm_evt_{}", event.event_id),
                    hyperedge_type: "OcpmEvent".into(),
                    subtype: event.activity_id.clone(),
                    participants,
                    layer: HypergraphLayer::ProcessEvents,
                    properties: props,
                    timestamp: Some(event.timestamp.date_naive()),
                    is_anomaly: false,
                    anomaly_type: None,
                    features: vec![],
                });
            }
        }
    }

    /// Adds compliance regulation nodes: standards (Layer 1), findings & filings (Layer 2).
    ///
    /// Creates cross-layer edges:
    /// - Standard → Account (GovernedByStandard) via `applicable_account_types`
    /// - Standard → Control (StandardToControl) via domain/process mapping
    /// - Finding → Control (FindingOnControl) if finding has `control_id`
    pub fn add_compliance_regulations(
        &mut self,
        standards: &[ComplianceStandard],
        findings: &[ComplianceFinding],
        filings: &[RegulatoryFiling],
    ) {
        if !self.config.include_compliance {
            return;
        }

        // Standards → Layer 1 (Governance)
        for std in standards {
            if std.is_superseded() {
                continue;
            }
            let sid = std.id.as_str().to_string();
            let node_id = format!("cr_std_{sid}");
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "compliance_standard".into(),
                entity_type_code: type_codes::COMPLIANCE_STANDARD,
                layer: HypergraphLayer::GovernanceControls,
                external_id: sid.clone(),
                label: format!("{}: {}", sid, std.title),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("title".into(), Value::String(std.title.clone()));
                    p.insert("category".into(), Value::String(std.category.to_string()));
                    p.insert("domain".into(), Value::String(std.domain.to_string()));
                    p.insert(
                        "issuingBody".into(),
                        Value::String(std.issuing_body.to_string()),
                    );
                    if !std.applicable_account_types.is_empty() {
                        p.insert(
                            "applicableAccountTypes".into(),
                            Value::Array(
                                std.applicable_account_types
                                    .iter()
                                    .map(|s| Value::String(s.clone()))
                                    .collect(),
                            ),
                        );
                    }
                    if !std.applicable_processes.is_empty() {
                        p.insert(
                            "applicableProcesses".into(),
                            Value::Array(
                                std.applicable_processes
                                    .iter()
                                    .map(|s| Value::String(s.clone()))
                                    .collect(),
                            ),
                        );
                    }
                    p
                },
                features: vec![
                    std.versions.len() as f64,
                    std.requirements.len() as f64,
                    std.mandatory_jurisdictions.len() as f64,
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.standard_node_ids.insert(sid.clone(), node_id.clone());

                // Collect deferred standard→account links for cross-layer edges
                for _acct_type in &std.applicable_account_types {
                    // Deferred: resolved in build_cross_layer_edges
                    // We match account_type against account names/labels
                }
            }
        }

        // Findings → Layer 2 (ProcessEvents)
        for finding in findings {
            let fid = finding.finding_id.to_string();
            let node_id = format!("cr_find_{fid}");
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "compliance_finding".into(),
                entity_type_code: type_codes::COMPLIANCE_FINDING,
                layer: HypergraphLayer::ProcessEvents,
                external_id: fid,
                label: format!("CF {} [{}]", finding.deficiency_level, finding.company_code),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("title".into(), Value::String(finding.title.clone()));
                    p.insert(
                        "severity".into(),
                        Value::String(finding.severity.to_string()),
                    );
                    p.insert(
                        "deficiencyLevel".into(),
                        Value::String(finding.deficiency_level.to_string()),
                    );
                    p.insert(
                        "companyCode".into(),
                        Value::String(finding.company_code.clone()),
                    );
                    p.insert(
                        "remediationStatus".into(),
                        Value::String(finding.remediation_status.to_string()),
                    );
                    p.insert("isRepeat".into(), Value::Bool(finding.is_repeat));
                    p.insert(
                        "identifiedDate".into(),
                        Value::String(finding.identified_date.to_string()),
                    );
                    p
                },
                features: vec![
                    finding.severity.score(),
                    finding.deficiency_level.severity_score(),
                    if finding.is_repeat { 1.0 } else { 0.0 },
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                // Link finding → standard(s)
                for std_id in &finding.related_standards {
                    let sid = std_id.as_str().to_string();
                    if let Some(std_node) = self.standard_node_ids.get(&sid) {
                        self.edges.push(CrossLayerEdge {
                            source_id: node_id.clone(),
                            source_layer: HypergraphLayer::ProcessEvents,
                            target_id: std_node.clone(),
                            target_layer: HypergraphLayer::GovernanceControls,
                            edge_type: "FindingOnStandard".to_string(),
                            edge_type_code: type_codes::GOVERNED_BY_STANDARD,
                            properties: HashMap::new(),
                        });
                    }
                }

                // Deferred: Finding → Control
                if let Some(ref ctrl_id) = finding.control_id {
                    self.compliance_finding_control_links
                        .push((node_id, ctrl_id.clone()));
                }
            }
        }

        // Filings → Layer 2 (ProcessEvents)
        for filing in filings {
            let filing_key = format!(
                "{}_{}_{}_{}",
                filing.filing_type, filing.company_code, filing.jurisdiction, filing.period_end
            );
            let node_id = format!("cr_filing_{filing_key}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "regulatory_filing".into(),
                entity_type_code: type_codes::REGULATORY_FILING,
                layer: HypergraphLayer::ProcessEvents,
                external_id: filing_key,
                label: format!("{} [{}]", filing.filing_type, filing.company_code),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "filingType".into(),
                        Value::String(filing.filing_type.to_string()),
                    );
                    p.insert(
                        "companyCode".into(),
                        Value::String(filing.company_code.clone()),
                    );
                    p.insert(
                        "jurisdiction".into(),
                        Value::String(filing.jurisdiction.clone()),
                    );
                    p.insert(
                        "status".into(),
                        Value::String(format!("{:?}", filing.status)),
                    );
                    p.insert(
                        "periodEnd".into(),
                        Value::String(filing.period_end.to_string()),
                    );
                    p.insert(
                        "deadline".into(),
                        Value::String(filing.deadline.to_string()),
                    );
                    p
                },
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    // =========================================================================
    // New Domain Builder Methods
    // =========================================================================

    /// Add tax documents as Layer 3 (Accounting Network) nodes.
    ///
    /// Creates nodes for jurisdictions, tax codes, tax lines, tax returns,
    /// tax provisions, and withholding tax records.
    #[allow(clippy::too_many_arguments)]
    pub fn add_tax_documents(
        &mut self,
        jurisdictions: &[TaxJurisdiction],
        codes: &[TaxCode],
        tax_lines: &[TaxLine],
        tax_returns: &[TaxReturn],
        tax_provisions: &[TaxProvision],
        withholding_records: &[WithholdingTaxRecord],
    ) {
        if !self.config.include_tax {
            return;
        }

        for jur in jurisdictions {
            let node_id = format!("tax_jur_{}", jur.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "tax_jurisdiction".into(),
                entity_type_code: type_codes::TAX_JURISDICTION,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: jur.id.clone(),
                label: jur.name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("country_code".into(), Value::String(jur.country_code.clone()));
                    p.insert(
                        "jurisdiction_type".into(),
                        Value::String(format!("{:?}", jur.jurisdiction_type)),
                    );
                    p.insert("vat_registered".into(), Value::Bool(jur.vat_registered));
                    if let Some(ref region) = jur.region_code {
                        p.insert("region_code".into(), Value::String(region.clone()));
                    }
                    p
                },
                features: vec![if jur.vat_registered { 1.0 } else { 0.0 }],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for code in codes {
            let node_id = format!("tax_code_{}", code.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "tax_code".into(),
                entity_type_code: type_codes::TAX_CODE,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: code.id.clone(),
                label: format!("{} ({})", code.code, code.description),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("code".into(), Value::String(code.code.clone()));
                    p.insert(
                        "tax_type".into(),
                        Value::String(format!("{:?}", code.tax_type)),
                    );
                    let rate: f64 = code.rate.to_string().parse().unwrap_or(0.0);
                    p.insert("rate".into(), serde_json::json!(rate));
                    p.insert(
                        "jurisdiction_id".into(),
                        Value::String(code.jurisdiction_id.clone()),
                    );
                    p.insert("is_exempt".into(), Value::Bool(code.is_exempt));
                    p.insert("is_reverse_charge".into(), Value::Bool(code.is_reverse_charge));
                    p
                },
                features: vec![code.rate.to_string().parse::<f64>().unwrap_or(0.0)],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for line in tax_lines {
            let node_id = format!("tax_line_{}", line.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "tax_line".into(),
                entity_type_code: type_codes::TAX_LINE,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: line.id.clone(),
                label: format!("TAXL {} L{}", line.document_id, line.line_number),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "document_type".into(),
                        Value::String(format!("{:?}", line.document_type)),
                    );
                    p.insert("document_id".into(), Value::String(line.document_id.clone()));
                    p.insert(
                        "tax_code_id".into(),
                        Value::String(line.tax_code_id.clone()),
                    );
                    let amt: f64 = line.tax_amount.to_string().parse().unwrap_or(0.0);
                    p.insert("tax_amount".into(), serde_json::json!(amt));
                    p
                },
                features: vec![line
                    .tax_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for ret in tax_returns {
            let node_id = format!("tax_ret_{}", ret.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "tax_return".into(),
                entity_type_code: type_codes::TAX_RETURN,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: ret.id.clone(),
                label: format!("TAXR {} [{:?}]", ret.entity_id, ret.return_type),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(ret.entity_id.clone()));
                    p.insert(
                        "jurisdiction_id".into(),
                        Value::String(ret.jurisdiction_id.clone()),
                    );
                    p.insert(
                        "return_type".into(),
                        Value::String(format!("{:?}", ret.return_type)),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", ret.status)));
                    p.insert(
                        "period_start".into(),
                        Value::String(ret.period_start.to_string()),
                    );
                    p.insert(
                        "period_end".into(),
                        Value::String(ret.period_end.to_string()),
                    );
                    p.insert("is_late".into(), Value::Bool(ret.is_late));
                    let net: f64 = ret.net_payable.to_string().parse().unwrap_or(0.0);
                    p.insert("net_payable".into(), serde_json::json!(net));
                    p
                },
                features: vec![
                    ret.net_payable
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p(),
                    if ret.is_late { 1.0 } else { 0.0 },
                ],
                is_anomaly: ret.is_late,
                anomaly_type: if ret.is_late {
                    Some("late_filing".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for prov in tax_provisions {
            let node_id = format!("tax_prov_{}", prov.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "tax_provision".into(),
                entity_type_code: type_codes::TAX_PROVISION,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: prov.id.clone(),
                label: format!("TAXPROV {} {}", prov.entity_id, prov.period),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(prov.entity_id.clone()));
                    p.insert("period".into(), Value::String(prov.period.to_string()));
                    let eff: f64 = prov.effective_rate.to_string().parse().unwrap_or(0.0);
                    p.insert("effective_rate".into(), serde_json::json!(eff));
                    let stat: f64 = prov.statutory_rate.to_string().parse().unwrap_or(0.0);
                    p.insert("statutory_rate".into(), serde_json::json!(stat));
                    let expense: f64 = prov.current_tax_expense.to_string().parse().unwrap_or(0.0);
                    p.insert("current_tax_expense".into(), serde_json::json!(expense));
                    p
                },
                features: vec![
                    prov.effective_rate
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    prov.current_tax_expense
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p(),
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for wht in withholding_records {
            let node_id = format!("tax_wht_{}", wht.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "withholding_tax_record".into(),
                entity_type_code: type_codes::WITHHOLDING_TAX,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: wht.id.clone(),
                label: format!("WHT {} → {}", wht.payment_id, wht.vendor_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("payment_id".into(), Value::String(wht.payment_id.clone()));
                    p.insert("vendor_id".into(), Value::String(wht.vendor_id.clone()));
                    p.insert(
                        "withholding_type".into(),
                        Value::String(format!("{:?}", wht.withholding_type)),
                    );
                    let amt: f64 = wht.withheld_amount.to_string().parse().unwrap_or(0.0);
                    p.insert("withheld_amount".into(), serde_json::json!(amt));
                    let rate: f64 = wht.applied_rate.to_string().parse().unwrap_or(0.0);
                    p.insert("applied_rate".into(), serde_json::json!(rate));
                    p
                },
                features: vec![wht
                    .withheld_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add treasury documents as Layer 3 (Accounting Network) nodes.
    ///
    /// Creates nodes for cash positions, cash forecasts, hedge relationships,
    /// and debt instruments.
    pub fn add_treasury_documents(
        &mut self,
        cash_positions: &[CashPosition],
        cash_forecasts: &[CashForecast],
        hedge_relationships: &[HedgeRelationship],
        debt_instruments: &[DebtInstrument],
    ) {
        if !self.config.include_treasury {
            return;
        }

        for pos in cash_positions {
            let node_id = format!("treas_pos_{}", pos.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "cash_position".into(),
                entity_type_code: type_codes::CASH_POSITION,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: pos.id.clone(),
                label: format!("CPOS {} {}", pos.bank_account_id, pos.date),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(pos.entity_id.clone()));
                    p.insert(
                        "bank_account_id".into(),
                        Value::String(pos.bank_account_id.clone()),
                    );
                    p.insert("currency".into(), Value::String(pos.currency.clone()));
                    p.insert("date".into(), Value::String(pos.date.to_string()));
                    let closing: f64 = pos.closing_balance.to_string().parse().unwrap_or(0.0);
                    p.insert("closing_balance".into(), serde_json::json!(closing));
                    p
                },
                features: vec![pos
                    .closing_balance
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for fc in cash_forecasts {
            let node_id = format!("treas_fc_{}", fc.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "cash_forecast".into(),
                entity_type_code: type_codes::CASH_FORECAST,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: fc.id.clone(),
                label: format!("CFOR {} {}d", fc.entity_id, fc.horizon_days),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(fc.entity_id.clone()));
                    p.insert("currency".into(), Value::String(fc.currency.clone()));
                    p.insert(
                        "forecast_date".into(),
                        Value::String(fc.forecast_date.to_string()),
                    );
                    p.insert(
                        "horizon_days".into(),
                        Value::Number((fc.horizon_days as u64).into()),
                    );
                    let net: f64 = fc.net_position.to_string().parse().unwrap_or(0.0);
                    p.insert("net_position".into(), serde_json::json!(net));
                    let conf: f64 = fc.confidence_level.to_string().parse().unwrap_or(0.0);
                    p.insert("confidence_level".into(), serde_json::json!(conf));
                    p
                },
                features: vec![
                    fc.net_position
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p(),
                    fc.confidence_level
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for hr in hedge_relationships {
            let node_id = format!("treas_hedge_{}", hr.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "hedge_relationship".into(),
                entity_type_code: type_codes::HEDGE_RELATIONSHIP,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: hr.id.clone(),
                label: format!("HEDGE {:?} {}", hr.hedge_type, hr.hedged_item_description),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "hedged_item_type".into(),
                        Value::String(format!("{:?}", hr.hedged_item_type)),
                    );
                    p.insert(
                        "hedge_type".into(),
                        Value::String(format!("{:?}", hr.hedge_type)),
                    );
                    p.insert(
                        "designation_date".into(),
                        Value::String(hr.designation_date.to_string()),
                    );
                    p.insert("is_effective".into(), Value::Bool(hr.is_effective));
                    let ratio: f64 = hr.effectiveness_ratio.to_string().parse().unwrap_or(0.0);
                    p.insert("effectiveness_ratio".into(), serde_json::json!(ratio));
                    p
                },
                features: vec![
                    hr.effectiveness_ratio
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    if hr.is_effective { 1.0 } else { 0.0 },
                ],
                is_anomaly: !hr.is_effective,
                anomaly_type: if !hr.is_effective {
                    Some("ineffective_hedge".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for debt in debt_instruments {
            let node_id = format!("treas_debt_{}", debt.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "debt_instrument".into(),
                entity_type_code: type_codes::DEBT_INSTRUMENT,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: debt.id.clone(),
                label: format!("DEBT {:?} {}", debt.instrument_type, debt.lender),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(debt.entity_id.clone()));
                    p.insert(
                        "instrument_type".into(),
                        Value::String(format!("{:?}", debt.instrument_type)),
                    );
                    p.insert("lender".into(), Value::String(debt.lender.clone()));
                    p.insert("currency".into(), Value::String(debt.currency.clone()));
                    let principal: f64 = debt.principal.to_string().parse().unwrap_or(0.0);
                    p.insert("principal".into(), serde_json::json!(principal));
                    let rate: f64 = debt.interest_rate.to_string().parse().unwrap_or(0.0);
                    p.insert("interest_rate".into(), serde_json::json!(rate));
                    p.insert(
                        "maturity_date".into(),
                        Value::String(debt.maturity_date.to_string()),
                    );
                    p.insert(
                        "covenant_count".into(),
                        Value::Number((debt.covenants.len() as u64).into()),
                    );
                    p
                },
                features: vec![
                    debt.principal
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .ln_1p(),
                    debt.interest_rate
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add ESG documents as Layer 1 (Governance & Controls) nodes.
    ///
    /// Creates nodes for emissions, disclosures, supplier assessments,
    /// and climate scenarios.
    pub fn add_esg_documents(
        &mut self,
        emissions: &[EmissionRecord],
        disclosures: &[EsgDisclosure],
        supplier_assessments: &[SupplierEsgAssessment],
        climate_scenarios: &[ClimateScenario],
    ) {
        if !self.config.include_esg {
            return;
        }

        for em in emissions {
            let node_id = format!("esg_em_{}", em.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "emission_record".into(),
                entity_type_code: type_codes::EMISSION_RECORD,
                layer: HypergraphLayer::GovernanceControls,
                external_id: em.id.clone(),
                label: format!("EM {:?} {}", em.scope, em.period),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(em.entity_id.clone()));
                    p.insert("scope".into(), Value::String(format!("{:?}", em.scope)));
                    p.insert("period".into(), Value::String(em.period.to_string()));
                    let co2e: f64 = em.co2e_tonnes.to_string().parse().unwrap_or(0.0);
                    p.insert("co2e_tonnes".into(), serde_json::json!(co2e));
                    p.insert(
                        "estimation_method".into(),
                        Value::String(format!("{:?}", em.estimation_method)),
                    );
                    if let Some(ref fid) = em.facility_id {
                        p.insert("facility_id".into(), Value::String(String::clone(fid)));
                    }
                    p
                },
                features: vec![em
                    .co2e_tonnes
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for disc in disclosures {
            let node_id = format!("esg_disc_{}", disc.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "esg_disclosure".into(),
                entity_type_code: type_codes::ESG_DISCLOSURE,
                layer: HypergraphLayer::GovernanceControls,
                external_id: disc.id.clone(),
                label: format!("{:?}: {}", disc.framework, disc.disclosure_topic),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(disc.entity_id.clone()));
                    p.insert(
                        "framework".into(),
                        Value::String(format!("{:?}", disc.framework)),
                    );
                    p.insert(
                        "disclosure_topic".into(),
                        Value::String(disc.disclosure_topic.clone()),
                    );
                    p.insert(
                        "assurance_level".into(),
                        Value::String(format!("{:?}", disc.assurance_level)),
                    );
                    p.insert("is_assured".into(), Value::Bool(disc.is_assured));
                    p.insert(
                        "reporting_period_start".into(),
                        Value::String(disc.reporting_period_start.to_string()),
                    );
                    p.insert(
                        "reporting_period_end".into(),
                        Value::String(disc.reporting_period_end.to_string()),
                    );
                    p
                },
                features: vec![if disc.is_assured { 1.0 } else { 0.0 }],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for sa in supplier_assessments {
            let node_id = format!("esg_sa_{}", sa.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "supplier_esg_assessment".into(),
                entity_type_code: type_codes::SUPPLIER_ESG_ASSESSMENT,
                layer: HypergraphLayer::GovernanceControls,
                external_id: sa.id.clone(),
                label: format!("ESG-SA {} ({})", sa.vendor_id, sa.assessment_date),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(sa.entity_id.clone()));
                    p.insert("vendor_id".into(), Value::String(sa.vendor_id.clone()));
                    p.insert(
                        "assessment_date".into(),
                        Value::String(sa.assessment_date.to_string()),
                    );
                    let overall: f64 = sa.overall_score.to_string().parse().unwrap_or(0.0);
                    p.insert("overall_score".into(), serde_json::json!(overall));
                    p.insert(
                        "risk_flag".into(),
                        Value::String(format!("{:?}", sa.risk_flag)),
                    );
                    p
                },
                features: vec![sa
                    .overall_score
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for cs in climate_scenarios {
            let node_id = format!("esg_cs_{}", cs.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "climate_scenario".into(),
                entity_type_code: type_codes::CLIMATE_SCENARIO,
                layer: HypergraphLayer::GovernanceControls,
                external_id: cs.id.clone(),
                label: format!("{:?} {:?}", cs.scenario_type, cs.time_horizon),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("entity_id".into(), Value::String(cs.entity_id.clone()));
                    p.insert(
                        "scenario_type".into(),
                        Value::String(format!("{:?}", cs.scenario_type)),
                    );
                    p.insert(
                        "time_horizon".into(),
                        Value::String(format!("{:?}", cs.time_horizon)),
                    );
                    p.insert("description".into(), Value::String(cs.description.clone()));
                    let temp: f64 = cs.temperature_rise_c.to_string().parse().unwrap_or(0.0);
                    p.insert("temperature_rise_c".into(), serde_json::json!(temp));
                    let fin: f64 = cs.financial_impact.to_string().parse().unwrap_or(0.0);
                    p.insert("financial_impact".into(), serde_json::json!(fin));
                    p
                },
                features: vec![
                    cs.temperature_rise_c
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    cs.financial_impact
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        .abs()
                        .ln_1p(),
                ],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add project accounting documents as Layer 3 (Accounting Network) nodes.
    ///
    /// Creates nodes for projects, earned value metrics, and milestones.
    pub fn add_project_documents(
        &mut self,
        projects: &[Project],
        earned_value_metrics: &[EarnedValueMetric],
        milestones: &[ProjectMilestone],
    ) {
        if !self.config.include_project {
            return;
        }

        for proj in projects {
            let node_id = format!("proj_{}", proj.project_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "project".into(),
                entity_type_code: type_codes::PROJECT,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: proj.project_id.clone(),
                label: format!("{} ({})", proj.name, proj.project_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("name".into(), Value::String(proj.name.clone()));
                    p.insert(
                        "project_type".into(),
                        Value::String(format!("{:?}", proj.project_type)),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", proj.status)));
                    p.insert(
                        "company_code".into(),
                        Value::String(proj.company_code.clone()),
                    );
                    let budget: f64 = proj.budget.to_string().parse().unwrap_or(0.0);
                    p.insert("budget".into(), serde_json::json!(budget));
                    p
                },
                features: vec![proj
                    .budget
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for evm in earned_value_metrics {
            let node_id = format!("proj_evm_{}", evm.id);
            let spi: f64 = evm.spi.to_string().parse().unwrap_or(1.0);
            let cpi: f64 = evm.cpi.to_string().parse().unwrap_or(1.0);
            // Flag as anomaly if schedule or cost performance is significantly off
            let is_anomaly = spi < 0.8 || cpi < 0.8;
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "earned_value_metric".into(),
                entity_type_code: type_codes::EARNED_VALUE,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: evm.id.clone(),
                label: format!("EVM {} {}", evm.project_id, evm.measurement_date),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("project_id".into(), Value::String(evm.project_id.clone()));
                    p.insert(
                        "measurement_date".into(),
                        Value::String(evm.measurement_date.to_string()),
                    );
                    p.insert("spi".into(), serde_json::json!(spi));
                    p.insert("cpi".into(), serde_json::json!(cpi));
                    let eac: f64 = evm.eac.to_string().parse().unwrap_or(0.0);
                    p.insert("eac".into(), serde_json::json!(eac));
                    p
                },
                features: vec![spi, cpi],
                is_anomaly,
                anomaly_type: if is_anomaly {
                    Some("poor_project_performance".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for ms in milestones {
            let node_id = format!("proj_ms_{}", ms.id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "project_milestone".into(),
                entity_type_code: type_codes::PROJECT_MILESTONE,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: ms.id.clone(),
                label: format!("MS {} ({})", ms.name, ms.project_id),
                properties: {
                    let mut p = HashMap::new();
                    p.insert("project_id".into(), Value::String(ms.project_id.clone()));
                    p.insert("name".into(), Value::String(ms.name.clone()));
                    p.insert(
                        "planned_date".into(),
                        Value::String(ms.planned_date.to_string()),
                    );
                    p.insert("status".into(), Value::String(format!("{:?}", ms.status)));
                    p.insert("sequence".into(), Value::Number((ms.sequence as u64).into()));
                    let amt: f64 = ms.payment_amount.to_string().parse().unwrap_or(0.0);
                    p.insert("payment_amount".into(), serde_json::json!(amt));
                    if let Some(ref actual) = ms.actual_date {
                        p.insert("actual_date".into(), Value::String(actual.to_string()));
                    }
                    p
                },
                features: vec![ms
                    .payment_amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add intercompany documents as Layer 3 (Accounting Network) nodes.
    ///
    /// Creates nodes for IC matched pairs and elimination entries.
    pub fn add_intercompany_documents(
        &mut self,
        matched_pairs: &[ICMatchedPair],
        elimination_entries: &[EliminationEntry],
    ) {
        if !self.config.include_intercompany {
            return;
        }

        for pair in matched_pairs {
            let node_id = format!("ic_pair_{}", pair.ic_reference);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "ic_matched_pair".into(),
                entity_type_code: type_codes::IC_MATCHED_PAIR,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: pair.ic_reference.clone(),
                label: format!(
                    "IC {} → {}",
                    pair.seller_company, pair.buyer_company
                ),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "transaction_type".into(),
                        Value::String(format!("{:?}", pair.transaction_type)),
                    );
                    p.insert(
                        "seller_company".into(),
                        Value::String(pair.seller_company.clone()),
                    );
                    p.insert(
                        "buyer_company".into(),
                        Value::String(pair.buyer_company.clone()),
                    );
                    let amt: f64 = pair.amount.to_string().parse().unwrap_or(0.0);
                    p.insert("amount".into(), serde_json::json!(amt));
                    p.insert("currency".into(), Value::String(pair.currency.clone()));
                    p.insert(
                        "settlement_status".into(),
                        Value::String(format!("{:?}", pair.settlement_status)),
                    );
                    p.insert(
                        "transaction_date".into(),
                        Value::String(pair.transaction_date.to_string()),
                    );
                    p
                },
                features: vec![pair
                    .amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for elim in elimination_entries {
            let node_id = format!("ic_elim_{}", elim.entry_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "elimination_entry".into(),
                entity_type_code: type_codes::ELIMINATION_ENTRY,
                layer: HypergraphLayer::AccountingNetwork,
                external_id: elim.entry_id.clone(),
                label: format!(
                    "ELIM {:?} {} {}",
                    elim.elimination_type, elim.consolidation_entity, elim.fiscal_period
                ),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "elimination_type".into(),
                        Value::String(format!("{:?}", elim.elimination_type)),
                    );
                    p.insert(
                        "consolidation_entity".into(),
                        Value::String(elim.consolidation_entity.clone()),
                    );
                    p.insert(
                        "fiscal_period".into(),
                        Value::String(elim.fiscal_period.clone()),
                    );
                    p.insert("currency".into(), Value::String(elim.currency.clone()));
                    p.insert(
                        "is_permanent".into(),
                        Value::Bool(elim.is_permanent),
                    );
                    let debit: f64 = elim.total_debit.to_string().parse().unwrap_or(0.0);
                    p.insert("total_debit".into(), serde_json::json!(debit));
                    p
                },
                features: vec![elim
                    .total_debit
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add temporal events as Layer 2 (Process Events) nodes.
    ///
    /// Creates nodes for process evolution events, organizational events,
    /// and disruption events.
    pub fn add_temporal_events(
        &mut self,
        process_events: &[ProcessEvolutionEvent],
        organizational_events: &[OrganizationalEvent],
        disruption_events: &[DisruptionEvent],
    ) {
        if !self.config.include_temporal_events {
            return;
        }

        for pe in process_events {
            let node_id = format!("tevt_proc_{}", pe.event_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "process_evolution".into(),
                entity_type_code: type_codes::PROCESS_EVOLUTION,
                layer: HypergraphLayer::ProcessEvents,
                external_id: pe.event_id.clone(),
                label: format!("PEVOL {} {}", pe.event_id, pe.effective_date),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "event_type".into(),
                        Value::String(format!("{:?}", pe.event_type)),
                    );
                    p.insert(
                        "effective_date".into(),
                        Value::String(pe.effective_date.to_string()),
                    );
                    if let Some(ref desc) = pe.description {
                        p.insert("description".into(), Value::String(desc.clone()));
                    }
                    if !pe.tags.is_empty() {
                        p.insert(
                            "tags".into(),
                            Value::Array(pe.tags.iter().map(|t| Value::String(t.clone())).collect()),
                        );
                    }
                    p
                },
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for oe in organizational_events {
            let node_id = format!("tevt_org_{}", oe.event_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "organizational_event".into(),
                entity_type_code: type_codes::ORGANIZATIONAL_EVENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: oe.event_id.clone(),
                label: format!("ORGEV {} {}", oe.event_id, oe.effective_date),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "event_type".into(),
                        Value::String(format!("{:?}", oe.event_type)),
                    );
                    p.insert(
                        "effective_date".into(),
                        Value::String(oe.effective_date.to_string()),
                    );
                    if let Some(ref desc) = oe.description {
                        p.insert("description".into(), Value::String(desc.clone()));
                    }
                    if !oe.tags.is_empty() {
                        p.insert(
                            "tags".into(),
                            Value::Array(oe.tags.iter().map(|t| Value::String(t.clone())).collect()),
                        );
                    }
                    p
                },
                features: vec![],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            });
        }

        for de in disruption_events {
            let node_id = format!("tevt_dis_{}", de.event_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "disruption_event".into(),
                entity_type_code: type_codes::DISRUPTION_EVENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: de.event_id.clone(),
                label: format!("DISRUPT {} sev={}", de.event_id, de.severity),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "disruption_type".into(),
                        Value::String(format!("{:?}", de.disruption_type)),
                    );
                    p.insert("description".into(), Value::String(de.description.clone()));
                    p.insert("severity".into(), Value::Number(de.severity.into()));
                    if !de.affected_companies.is_empty() {
                        p.insert(
                            "affected_companies".into(),
                            Value::Array(
                                de.affected_companies
                                    .iter()
                                    .map(|c| Value::String(c.clone()))
                                    .collect(),
                            ),
                        );
                    }
                    p
                },
                features: vec![de.severity as f64 / 5.0],
                is_anomaly: de.severity >= 4,
                anomaly_type: if de.severity >= 4 {
                    Some("high_severity_disruption".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add AML alert nodes derived from suspicious banking transactions (Layer 2).
    ///
    /// Creates an `aml_alert` node for each suspicious transaction. These are
    /// separate from the `bank_transaction` nodes produced by `add_bank_documents`.
    pub fn add_aml_alerts(&mut self, transactions: &[BankTransaction]) {
        let suspicious: Vec<&BankTransaction> =
            transactions.iter().filter(|t| t.is_suspicious).collect();

        for txn in suspicious {
            let tid = txn.transaction_id.to_string();
            let node_id = format!("aml_alert_{tid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "aml_alert".into(),
                entity_type_code: type_codes::AML_ALERT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("AML-{tid}"),
                label: format!("AML {}", txn.reference),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "transaction_id".into(),
                        Value::String(tid.clone()),
                    );
                    let amount: f64 = txn.amount.to_string().parse().unwrap_or(0.0);
                    p.insert("amount".into(), serde_json::json!(amount));
                    p.insert("currency".into(), Value::String(txn.currency.clone()));
                    p.insert("reference".into(), Value::String(txn.reference.clone()));
                    if let Some(ref reason) = txn.suspicion_reason {
                        p.insert(
                            "suspicion_reason".into(),
                            Value::String(format!("{reason:?}")),
                        );
                    }
                    if let Some(ref stage) = txn.laundering_stage {
                        p.insert(
                            "laundering_stage".into(),
                            Value::String(format!("{stage:?}")),
                        );
                    }
                    p
                },
                features: vec![txn
                    .amount
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
                    .abs()
                    .ln_1p()],
                is_anomaly: true,
                anomaly_type: txn.suspicion_reason.as_ref().map(|r| format!("{r:?}")),
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Add KYC profile nodes derived from banking customers (Layer 2).
    ///
    /// Creates a `kyc_profile` node for each banking customer. These capture
    /// the KYC/AML risk profile rather than the transactional behavior.
    pub fn add_kyc_profiles(&mut self, customers: &[BankingCustomer]) {
        for cust in customers {
            let cid = cust.customer_id.to_string();
            let node_id = format!("kyc_{cid}");
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "kyc_profile".into(),
                entity_type_code: type_codes::KYC_PROFILE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("KYC-{cid}"),
                label: format!("KYC {}", cust.name.legal_name),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "customer_id".into(),
                        Value::String(cid.clone()),
                    );
                    p.insert("name".into(), Value::String(cust.name.legal_name.clone()));
                    p.insert(
                        "customer_type".into(),
                        Value::String(format!("{:?}", cust.customer_type)),
                    );
                    p.insert(
                        "risk_tier".into(),
                        Value::String(format!("{:?}", cust.risk_tier)),
                    );
                    p.insert(
                        "residence_country".into(),
                        Value::String(cust.residence_country.clone()),
                    );
                    p.insert("is_pep".into(), Value::Bool(cust.is_pep));
                    p.insert("is_mule".into(), Value::Bool(cust.is_mule));
                    p
                },
                features: vec![
                    if cust.is_pep { 1.0 } else { 0.0 },
                    if cust.is_mule { 1.0 } else { 0.0 },
                ],
                is_anomaly: cust.is_mule,
                anomaly_type: if cust.is_mule {
                    Some("mule_account".into())
                } else {
                    None
                },
                is_aggregate: false,
                aggregate_count: 0,
            });
        }
    }

    /// Tag all nodes with a `process_family` property based on their entity type.
    ///
    /// This replaces AssureTwin's entity_registry logic. Call after all nodes
    /// have been added and before `build()`.
    pub fn tag_process_family(&mut self) {
        for node in &mut self.nodes {
            let family = match node.entity_type.as_str() {
                // P2P (Procure-to-Pay)
                "purchase_order" | "goods_receipt" | "vendor_invoice" | "payment" | "p2p_pool" => {
                    "P2P"
                }
                // O2C (Order-to-Cash)
                "sales_order" | "delivery" | "customer_invoice" | "o2c_pool" => "O2C",
                // S2C (Source-to-Contract)
                "sourcing_project" | "supplier_qualification" | "rfx_event" | "supplier_bid"
                | "bid_evaluation" | "procurement_contract" => "S2C",
                // H2R (Hire-to-Retire)
                "payroll_run" | "time_entry" | "expense_report" | "payroll_line_item" => "H2R",
                // MFG (Manufacturing)
                "production_order" | "quality_inspection" | "cycle_count" => "MFG",
                // BANK (Banking)
                "banking_customer" | "bank_account" | "bank_transaction" | "aml_alert"
                | "kyc_profile" => "BANK",
                // AUDIT
                "audit_engagement" | "workpaper" | "audit_finding" | "audit_evidence"
                | "risk_assessment" | "professional_judgment" => "AUDIT",
                // R2R (Record-to-Report)
                "bank_reconciliation" | "bank_statement_line" | "reconciling_item" => "R2R",
                // TAX
                "tax_jurisdiction" | "tax_code" | "tax_line" | "tax_return" | "tax_provision"
                | "withholding_tax_record" => "TAX",
                // TREASURY
                "cash_position" | "cash_forecast" | "hedge_relationship" | "debt_instrument" => {
                    "TREASURY"
                }
                // ESG
                "emission_record" | "esg_disclosure" | "supplier_esg_assessment"
                | "climate_scenario" => "ESG",
                // PROJECT
                "project" | "earned_value_metric" | "project_milestone" => "PROJECT",
                // IC (Intercompany)
                "ic_matched_pair" | "elimination_entry" => "IC",
                // TEMPORAL
                "process_evolution" | "organizational_event" | "disruption_event" => "TEMPORAL",
                // COMPLIANCE
                "compliance_standard" | "compliance_finding" | "regulatory_filing" => "COMPLIANCE",
                // GOVERNANCE (COSO/Controls)
                "coso_component" | "coso_principle" | "sox_assertion" | "internal_control" => {
                    "GOVERNANCE"
                }
                // MASTER DATA
                "vendor" | "customer" | "employee" | "material" | "fixed_asset" => "MASTER_DATA",
                // ACCOUNTING
                "account" | "journal_entry" => "ACCOUNTING",
                // OCPM
                "ocpm_object" => "OCPM",
                // Unknown/other
                _ => "OTHER",
            };
            node.properties.insert(
                "process_family".into(),
                Value::String(family.to_string()),
            );
        }
    }

    /// Build cross-layer edges linking governance to accounting and process layers.
    pub fn build_cross_layer_edges(&mut self) {
        if !self.config.include_cross_layer_edges {
            return;
        }

        // Use pre-collected counterparty links instead of iterating all nodes
        let links = std::mem::take(&mut self.doc_counterparty_links);
        for (doc_node_id, counterparty_type, counterparty_id) in &links {
            let source_node_id = match counterparty_type.as_str() {
                "vendor" => self.vendor_node_ids.get(counterparty_id),
                "customer" => self.customer_node_ids.get(counterparty_id),
                _ => None,
            };
            if let Some(source_id) = source_node_id {
                self.edges.push(CrossLayerEdge {
                    source_id: source_id.clone(),
                    source_layer: HypergraphLayer::GovernanceControls,
                    target_id: doc_node_id.clone(),
                    target_layer: HypergraphLayer::ProcessEvents,
                    edge_type: "SuppliesTo".to_string(),
                    edge_type_code: type_codes::SUPPLIES_TO,
                    properties: HashMap::new(),
                });
            }
        }
        self.doc_counterparty_links = links;

        // Compliance: Finding → Control edges
        let finding_ctrl_links = std::mem::take(&mut self.compliance_finding_control_links);
        for (finding_node_id, ctrl_id) in &finding_ctrl_links {
            if let Some(ctrl_node_id) = self.control_node_ids.get(ctrl_id) {
                self.edges.push(CrossLayerEdge {
                    source_id: finding_node_id.clone(),
                    source_layer: HypergraphLayer::ProcessEvents,
                    target_id: ctrl_node_id.clone(),
                    target_layer: HypergraphLayer::GovernanceControls,
                    edge_type: "FindingOnControl".to_string(),
                    edge_type_code: type_codes::FINDING_ON_CONTROL,
                    properties: HashMap::new(),
                });
            }
        }
        self.compliance_finding_control_links = finding_ctrl_links;

        // Compliance: Standard → Account edges (match by account label/name)
        let std_ids: Vec<(String, String)> = self
            .standard_node_ids
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (std_id, std_node_id) in &std_ids {
            // Look up the standard's applicable_account_types from node properties
            if let Some(&node_idx) = self.node_index.get(std_node_id) {
                if let Some(node) = self.nodes.get(node_idx) {
                    if let Some(Value::Array(acct_types)) =
                        node.properties.get("applicableAccountTypes")
                    {
                        let type_strings: Vec<String> = acct_types
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
                            .collect();

                        // Match against account nodes by checking if name contains
                        for (acct_code, acct_node_id) in &self.account_node_ids {
                            // Get account label from node
                            if let Some(&acct_idx) = self.node_index.get(acct_node_id) {
                                if let Some(acct_node) = self.nodes.get(acct_idx) {
                                    let label_lower = acct_node.label.to_lowercase();
                                    let matches = type_strings.iter().any(|t| {
                                        label_lower.contains(t)
                                            || acct_code.to_lowercase().contains(t)
                                    });
                                    if matches {
                                        self.edges.push(CrossLayerEdge {
                                            source_id: std_node_id.clone(),
                                            source_layer: HypergraphLayer::GovernanceControls,
                                            target_id: acct_node_id.clone(),
                                            target_layer: HypergraphLayer::AccountingNetwork,
                                            edge_type: format!("GovernedByStandard:{}", std_id),
                                            edge_type_code: type_codes::STANDARD_TO_ACCOUNT,
                                            properties: HashMap::new(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Compliance: Standard → Control edges (match by control process mapping)
        for (_std_id, std_node_id) in &std_ids {
            if let Some(&node_idx) = self.node_index.get(std_node_id) {
                if let Some(node) = self.nodes.get(node_idx) {
                    if let Some(Value::Array(processes)) =
                        node.properties.get("applicableProcesses")
                    {
                        let proc_strings: Vec<String> = processes
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();

                        // For SOX/audit standards, link to all controls
                        let is_universal = proc_strings.len() >= 5;
                        if is_universal {
                            // Link to all controls (this standard governs all processes)
                            for ctrl_node_id in self.control_node_ids.values() {
                                self.edges.push(CrossLayerEdge {
                                    source_id: std_node_id.clone(),
                                    source_layer: HypergraphLayer::GovernanceControls,
                                    target_id: ctrl_node_id.clone(),
                                    target_layer: HypergraphLayer::GovernanceControls,
                                    edge_type: "StandardToControl".to_string(),
                                    edge_type_code: type_codes::STANDARD_TO_CONTROL,
                                    properties: HashMap::new(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Finalize and build the Hypergraph.
    pub fn build(mut self) -> Hypergraph {
        // Build cross-layer edges last (they reference all nodes)
        self.build_cross_layer_edges();

        // Compute metadata
        let mut layer_node_counts: HashMap<String, usize> = HashMap::new();
        let mut node_type_counts: HashMap<String, usize> = HashMap::new();
        let mut anomalous_nodes = 0;

        for node in &self.nodes {
            *layer_node_counts
                .entry(node.layer.name().to_string())
                .or_insert(0) += 1;
            *node_type_counts
                .entry(node.entity_type.clone())
                .or_insert(0) += 1;
            if node.is_anomaly {
                anomalous_nodes += 1;
            }
        }

        let mut edge_type_counts: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *edge_type_counts.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        let mut hyperedge_type_counts: HashMap<String, usize> = HashMap::new();
        let mut anomalous_hyperedges = 0;
        for he in &self.hyperedges {
            *hyperedge_type_counts
                .entry(he.hyperedge_type.clone())
                .or_insert(0) += 1;
            if he.is_anomaly {
                anomalous_hyperedges += 1;
            }
        }

        let budget_report = NodeBudgetReport {
            total_budget: self.budget.total_max(),
            total_used: self.budget.total_count(),
            layer1_budget: self.budget.layer1_max,
            layer1_used: self.budget.layer1_count,
            layer2_budget: self.budget.layer2_max,
            layer2_used: self.budget.layer2_count,
            layer3_budget: self.budget.layer3_max,
            layer3_used: self.budget.layer3_count,
            aggregate_nodes_created: self.aggregate_count,
            aggregation_triggered: self.aggregate_count > 0,
        };

        let metadata = HypergraphMetadata {
            name: "multi_layer_hypergraph".to_string(),
            num_nodes: self.nodes.len(),
            num_edges: self.edges.len(),
            num_hyperedges: self.hyperedges.len(),
            layer_node_counts,
            node_type_counts,
            edge_type_counts,
            hyperedge_type_counts,
            anomalous_nodes,
            anomalous_hyperedges,
            source: "datasynth".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            budget_report: budget_report.clone(),
            files: vec![
                "nodes.jsonl".to_string(),
                "edges.jsonl".to_string(),
                "hyperedges.jsonl".to_string(),
                "metadata.json".to_string(),
            ],
        };

        Hypergraph {
            nodes: self.nodes,
            edges: self.edges,
            hyperedges: self.hyperedges,
            metadata,
            budget_report,
        }
    }

    /// Try to add a node, respecting the budget. Returns true if added.
    fn try_add_node(&mut self, node: HypergraphNode) -> bool {
        if self.node_index.contains_key(&node.id) {
            return false; // Already exists
        }

        if !self.budget.can_add(node.layer) {
            return false; // Budget exceeded
        }

        let id = node.id.clone();
        let layer = node.layer;
        self.nodes.push(node);
        let idx = self.nodes.len() - 1;
        self.node_index.insert(id, idx);
        self.budget.record_add(layer);
        true
    }
}

/// Map COSO component to a numeric feature.
fn component_to_feature(component: &CosoComponent) -> f64 {
    match component {
        CosoComponent::ControlEnvironment => 1.0,
        CosoComponent::RiskAssessment => 2.0,
        CosoComponent::ControlActivities => 3.0,
        CosoComponent::InformationCommunication => 4.0,
        CosoComponent::MonitoringActivities => 5.0,
    }
}

/// Map account type to a numeric feature.
fn account_type_feature(account_type: &datasynth_core::models::AccountType) -> f64 {
    use datasynth_core::models::AccountType;
    match account_type {
        AccountType::Asset => 1.0,
        AccountType::Liability => 2.0,
        AccountType::Equity => 3.0,
        AccountType::Revenue => 4.0,
        AccountType::Expense => 5.0,
        AccountType::Statistical => 6.0,
    }
}

/// Compute features for a journal entry hyperedge.
fn compute_je_features(entry: &JournalEntry) -> Vec<f64> {
    let total_debit: f64 = entry
        .lines
        .iter()
        .map(|l| l.debit_amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    let line_count = entry.lines.len() as f64;
    let posting_date = entry.header.posting_date;
    let weekday = posting_date.weekday().num_days_from_monday() as f64 / WEEKDAY_NORMALIZER;
    let day = posting_date.day() as f64 / DAY_OF_MONTH_NORMALIZER;
    let month = posting_date.month() as f64 / MONTH_NORMALIZER;
    let is_month_end = if posting_date.day() >= MONTH_END_DAY_THRESHOLD {
        1.0
    } else {
        0.0
    };

    vec![
        (total_debit.abs() + 1.0).ln(), // log amount
        line_count,                     // number of lines
        weekday,                        // weekday normalized
        day,                            // day of month normalized
        month,                          // month normalized
        is_month_end,                   // month-end flag
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{
        AccountSubType, AccountType, ChartOfAccounts, CoAComplexity, ControlFrequency, ControlType,
        CosoComponent, CosoMaturityLevel, GLAccount, InternalControl, RiskLevel, SoxAssertion,
        UserPersona,
    };

    fn make_test_coa() -> ChartOfAccounts {
        let mut coa = ChartOfAccounts::new(
            "TEST_COA".to_string(),
            "Test Chart".to_string(),
            "US".to_string(),
            datasynth_core::models::IndustrySector::Manufacturing,
            CoAComplexity::Small,
        );

        coa.add_account(GLAccount::new(
            "1000".to_string(),
            "Cash".to_string(),
            AccountType::Asset,
            AccountSubType::Cash,
        ));
        coa.add_account(GLAccount::new(
            "2000".to_string(),
            "AP".to_string(),
            AccountType::Liability,
            AccountSubType::AccountsPayable,
        ));

        coa
    }

    fn make_test_control() -> InternalControl {
        InternalControl {
            control_id: "C001".to_string(),
            control_name: "Three-Way Match".to_string(),
            control_type: ControlType::Preventive,
            objective: "Ensure proper matching".to_string(),
            frequency: ControlFrequency::Transactional,
            owner_role: UserPersona::Controller,
            risk_level: RiskLevel::High,
            description: "Test control".to_string(),
            is_key_control: true,
            sox_assertion: SoxAssertion::Existence,
            coso_component: CosoComponent::ControlActivities,
            coso_principles: vec![CosoPrinciple::ControlActions],
            control_scope: datasynth_core::models::ControlScope::TransactionLevel,
            maturity_level: CosoMaturityLevel::Managed,
            owner_employee_id: None,
            owner_name: "Test Controller".to_string(),
            test_count: 0,
            last_tested_date: None,
            test_result: datasynth_core::models::internal_control::TestResult::default(),
            effectiveness: datasynth_core::models::internal_control::ControlEffectiveness::default(),
            mitigates_risk_ids: Vec::new(),
            covers_account_classes: Vec::new(),
        }
    }

    #[test]
    fn test_builder_coso_framework() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();

        let hg = builder.build();
        // 5 components + 17 principles = 22 nodes
        assert_eq!(hg.nodes.len(), 22);
        assert!(hg
            .nodes
            .iter()
            .all(|n| n.layer == HypergraphLayer::GovernanceControls));
        // 17 principle → component edges
        assert_eq!(
            hg.edges
                .iter()
                .filter(|e| e.edge_type == "CoversCosoPrinciple")
                .count(),
            17
        );
    }

    #[test]
    fn test_builder_controls() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();
        builder.add_controls(&[make_test_control()]);

        let hg = builder.build();
        // 22 COSO + 1 control + 1 SOX assertion = 24
        assert_eq!(hg.nodes.len(), 24);
        assert!(hg.nodes.iter().any(|n| n.entity_type == "internal_control"));
        assert!(hg.nodes.iter().any(|n| n.entity_type == "sox_assertion"));
    }

    #[test]
    fn test_builder_accounts() {
        let config = HypergraphConfig {
            max_nodes: 1000,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_accounts(&make_test_coa());

        let hg = builder.build();
        assert_eq!(hg.nodes.len(), 2);
        assert!(hg
            .nodes
            .iter()
            .all(|n| n.layer == HypergraphLayer::AccountingNetwork));
    }

    #[test]
    fn test_budget_enforcement() {
        let config = HypergraphConfig {
            max_nodes: 10, // Very small budget
            include_coso: false,
            include_controls: false,
            include_sox: false,
            include_vendors: false,
            include_customers: false,
            include_employees: false,
            include_p2p: false,
            include_o2c: false,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_accounts(&make_test_coa());

        let hg = builder.build();
        // Budget for L3 is 10% of 10 = 1, so only 1 of 2 accounts should be added
        assert!(hg.nodes.len() <= 1);
    }

    #[test]
    fn test_full_build() {
        let config = HypergraphConfig {
            max_nodes: 10000,
            ..Default::default()
        };
        let mut builder = HypergraphBuilder::new(config);
        builder.add_coso_framework();
        builder.add_controls(&[make_test_control()]);
        builder.add_accounts(&make_test_coa());

        let hg = builder.build();
        assert!(!hg.nodes.is_empty());
        assert!(!hg.edges.is_empty());
        assert_eq!(hg.metadata.num_nodes, hg.nodes.len());
        assert_eq!(hg.metadata.num_edges, hg.edges.len());
    }
}
