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

use datasynth_core::models::{
    ChartOfAccounts, CosoComponent, CosoPrinciple, Customer, Employee, InternalControl,
    JournalEntry, Vendor,
};

use crate::models::hypergraph::{
    AggregationStrategy, CrossLayerEdge, Hyperedge, HyperedgeParticipant, Hypergraph,
    HypergraphLayer, HypergraphMetadata, HypergraphNode, NodeBudget, NodeBudgetReport,
};

/// RustGraph entity type codes for Layer 1 governance nodes.
#[allow(dead_code)]
mod type_codes {
    // Existing codes used by RustGraph
    pub const ACCOUNT: u32 = 100;
    pub const VENDOR: u32 = 200;
    pub const CUSTOMER: u32 = 201;
    pub const EMPLOYEE: u32 = 202;

    // Layer 1 governance type codes (proposed CR-02)
    pub const COSO_COMPONENT: u32 = 500;
    pub const COSO_PRINCIPLE: u32 = 501;
    pub const SOX_ASSERTION: u32 = 502;
    pub const INTERNAL_CONTROL: u32 = 504;

    // Layer 2 process type codes
    pub const PURCHASE_ORDER: u32 = 300;
    pub const GOODS_RECEIPT: u32 = 301;
    pub const VENDOR_INVOICE: u32 = 302;
    pub const PAYMENT: u32 = 303;
    pub const SALES_ORDER: u32 = 310;
    pub const DELIVERY: u32 = 311;
    pub const CUSTOMER_INVOICE: u32 = 312;
    pub const POOL_NODE: u32 = 399;

    // Edge type codes (proposed CR-03)
    pub const IMPLEMENTS_CONTROL: u32 = 40;
    pub const GOVERNED_BY_STANDARD: u32 = 41;
    pub const OWNS_CONTROL: u32 = 42;
    pub const OVERSEE_PROCESS: u32 = 43;
    pub const ENFORCES_ASSERTION: u32 = 44;
    pub const SUPPLIES_TO: u32 = 48;
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
        }
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
                entity_type: "CosoComponent".to_string(),
                entity_type_code: type_codes::COSO_COMPONENT,
                layer: HypergraphLayer::GovernanceControls,
                external_id: format!("{:?}", component),
                label: name.to_string(),
                properties: HashMap::new(),
                features: vec![component_to_feature(component)],
                is_anomaly: false,
                anomaly_type: None,
                is_aggregate: false,
                aggregate_count: 0,
            }) {
                self.coso_component_ids
                    .insert(format!("{:?}", component), id);
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
                entity_type: "CosoPrinciple".to_string(),
                entity_type_code: type_codes::COSO_PRINCIPLE,
                layer: HypergraphLayer::GovernanceControls,
                external_id: format!("{:?}", principle),
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
                let comp_key = format!("{:?}", parent_component);
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
                entity_type: "InternalControl".to_string(),
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
                        "risk_level".to_string(),
                        Value::String(format!("{:?}", control.risk_level)),
                    );
                    p.insert(
                        "is_key_control".to_string(),
                        Value::Bool(control.is_key_control),
                    );
                    p.insert(
                        "maturity_level".to_string(),
                        Value::String(format!("{:?}", control.maturity_level)),
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
                            entity_type: "SoxAssertion".to_string(),
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
                entity_type: "Vendor".to_string(),
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
                entity_type: "Customer".to_string(),
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
                entity_type: "Employee".to_string(),
                entity_type_code: type_codes::EMPLOYEE,
                layer: HypergraphLayer::GovernanceControls,
                external_id: employee.employee_id.clone(),
                label: employee.display_name.clone(),
                properties: {
                    let mut p = HashMap::new();
                    p.insert(
                        "persona".to_string(),
                        Value::String(format!("{:?}", employee.persona)),
                    );
                    p.insert(
                        "job_level".to_string(),
                        Value::String(format!("{:?}", employee.job_level)),
                    );
                    p.insert(
                        "company_code".to_string(),
                        Value::String(employee.company_code.clone()),
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

    /// Add GL accounts as Layer 3 nodes.
    pub fn add_accounts(&mut self, coa: &ChartOfAccounts) {
        if !self.config.include_accounts {
            return;
        }

        for account in &coa.accounts {
            let node_id = format!("acct_{}", account.account_number);
            if self.try_add_node(HypergraphNode {
                id: node_id.clone(),
                entity_type: "Account".to_string(),
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
                        entity_type: "Account".to_string(),
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
                .map(|bp| format!("{:?}", bp))
                .unwrap_or_else(|| "General".to_string());

            self.hyperedges.push(Hyperedge {
                id: format!("je_{}", doc_id),
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
                anomaly_type: entry.header.anomaly_type.clone().or_else(|| {
                    entry
                        .header
                        .fraud_type
                        .as_ref()
                        .map(|ft| format!("{:?}", ft))
                }),
                features: compute_je_features(entry),
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
            let pool_id = format!("pool_p2p_{}", vendor_id);
            self.try_add_node(HypergraphNode {
                id: pool_id,
                entity_type: "P2PPool".to_string(),
                entity_type_code: type_codes::POOL_NODE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("pool_p2p_{}", vendor_id),
                label: format!("P2P Pool ({}): {} docs", vendor_id, count),
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
            });
            self.aggregate_count += 1;
        }

        // Add individual PO nodes (if not pooled)
        for po in purchase_orders {
            if should_aggregate && vendors_needing_pools.contains(&po.vendor_id) {
                continue; // Pooled
            }

            let doc_id = &po.header.document_id;
            let node_id = format!("po_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "PurchaseOrder".to_string(),
                entity_type_code: type_codes::PURCHASE_ORDER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("PO {}", doc_id),
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
            });
        }

        // Add GR nodes
        for gr in goods_receipts {
            let vendor_id = gr.vendor_id.as_deref().unwrap_or("UNKNOWN");
            if should_aggregate && vendors_needing_pools.contains(&vendor_id.to_string()) {
                continue;
            }
            let doc_id = &gr.header.document_id;
            let node_id = format!("gr_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "GoodsReceipt".to_string(),
                entity_type_code: type_codes::GOODS_RECEIPT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("GR {}", doc_id),
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
            let node_id = format!("vinv_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "VendorInvoice".to_string(),
                entity_type_code: type_codes::VENDOR_INVOICE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("VI {}", doc_id),
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
            let node_id = format!("pmt_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "Payment".to_string(),
                entity_type_code: type_codes::PAYMENT,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("PMT {}", doc_id),
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
            let pool_id = format!("pool_o2c_{}", customer_id);
            self.try_add_node(HypergraphNode {
                id: pool_id,
                entity_type: "O2CPool".to_string(),
                entity_type_code: type_codes::POOL_NODE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: format!("pool_o2c_{}", customer_id),
                label: format!("O2C Pool ({}): {} docs", customer_id, count),
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
            });
            self.aggregate_count += 1;
        }

        for so in sales_orders {
            if should_aggregate && customers_needing_pools.contains(&so.customer_id) {
                continue;
            }
            let doc_id = &so.header.document_id;
            let node_id = format!("so_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "SalesOrder".to_string(),
                entity_type_code: type_codes::SALES_ORDER,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("SO {}", doc_id),
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
            });
        }

        for del in deliveries {
            if should_aggregate && customers_needing_pools.contains(&del.customer_id) {
                continue;
            }
            let doc_id = &del.header.document_id;
            let node_id = format!("del_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "Delivery".to_string(),
                entity_type_code: type_codes::DELIVERY,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("DEL {}", doc_id),
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
            let node_id = format!("cinv_{}", doc_id);
            self.try_add_node(HypergraphNode {
                id: node_id,
                entity_type: "CustomerInvoice".to_string(),
                entity_type_code: type_codes::CUSTOMER_INVOICE,
                layer: HypergraphLayer::ProcessEvents,
                external_id: doc_id.clone(),
                label: format!("CI {}", doc_id),
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

    /// Build cross-layer edges linking governance to accounting and process layers.
    pub fn build_cross_layer_edges(&mut self) {
        if !self.config.include_cross_layer_edges {
            return;
        }

        // Vendor → PO: SuppliesTo edges (L1 → L2)
        for node in &self.nodes {
            if node.entity_type == "PurchaseOrder" {
                if let Some(vendor_id) = node.properties.get("vendor_id").and_then(|v| v.as_str()) {
                    if let Some(vendor_node_id) = self.vendor_node_ids.get(vendor_id) {
                        self.edges.push(CrossLayerEdge {
                            source_id: vendor_node_id.clone(),
                            source_layer: HypergraphLayer::GovernanceControls,
                            target_id: node.id.clone(),
                            target_layer: HypergraphLayer::ProcessEvents,
                            edge_type: "SuppliesTo".to_string(),
                            edge_type_code: type_codes::SUPPLIES_TO,
                            properties: HashMap::new(),
                        });
                    }
                }
            }

            // Customer → SO: SuppliesTo edges (L1 → L2)
            if node.entity_type == "SalesOrder" {
                if let Some(customer_id) =
                    node.properties.get("customer_id").and_then(|v| v.as_str())
                {
                    if let Some(customer_node_id) = self.customer_node_ids.get(customer_id) {
                        self.edges.push(CrossLayerEdge {
                            source_id: customer_node_id.clone(),
                            source_layer: HypergraphLayer::GovernanceControls,
                            target_id: node.id.clone(),
                            target_layer: HypergraphLayer::ProcessEvents,
                            edge_type: "SuppliesTo".to_string(),
                            edge_type_code: type_codes::SUPPLIES_TO,
                            properties: HashMap::new(),
                        });
                    }
                }
            }

            // Pool nodes → vendor/customer too
            if node.entity_type == "P2PPool" {
                if let Some(vendor_id) = node.properties.get("vendor_id").and_then(|v| v.as_str()) {
                    if let Some(vendor_node_id) = self.vendor_node_ids.get(vendor_id) {
                        self.edges.push(CrossLayerEdge {
                            source_id: vendor_node_id.clone(),
                            source_layer: HypergraphLayer::GovernanceControls,
                            target_id: node.id.clone(),
                            target_layer: HypergraphLayer::ProcessEvents,
                            edge_type: "SuppliesTo".to_string(),
                            edge_type_code: type_codes::SUPPLIES_TO,
                            properties: HashMap::new(),
                        });
                    }
                }
            }
            if node.entity_type == "O2CPool" {
                if let Some(customer_id) =
                    node.properties.get("customer_id").and_then(|v| v.as_str())
                {
                    if let Some(customer_node_id) = self.customer_node_ids.get(customer_id) {
                        self.edges.push(CrossLayerEdge {
                            source_id: customer_node_id.clone(),
                            source_layer: HypergraphLayer::GovernanceControls,
                            target_id: node.id.clone(),
                            target_layer: HypergraphLayer::ProcessEvents,
                            edge_type: "SuppliesTo".to_string(),
                            edge_type_code: type_codes::SUPPLIES_TO,
                            properties: HashMap::new(),
                        });
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
    let weekday = posting_date.weekday().num_days_from_monday() as f64 / 6.0;
    let day = posting_date.day() as f64 / 31.0;
    let month = posting_date.month() as f64 / 12.0;
    let is_month_end = if posting_date.day() >= 28 { 1.0 } else { 0.0 };

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
        assert!(hg.nodes.iter().any(|n| n.entity_type == "InternalControl"));
        assert!(hg.nodes.iter().any(|n| n.entity_type == "SoxAssertion"));
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
