//! Integration tests for edge synthesizers.
//!
//! Tests the DocumentChainEdgeSynthesizer and RiskControlEdgeSynthesizer
//! using mock data constructed from the datasynth-core models.

#![allow(clippy::unwrap_used)]

use datasynth_graph_export::{
    EdgeSynthesisContext, EdgeSynthesizer, ExportConfig, ExportWarnings, IdMap,
};

use chrono::NaiveDate;
use datasynth_core::models::audit::finding::{AuditFinding, FindingType};
use datasynth_core::models::audit::risk::{RiskAssessment, RiskCategory};
use datasynth_core::models::documents::{
    CustomerInvoice, Delivery, GoodsReceipt, Payment, PurchaseOrder, SalesOrder, VendorInvoice,
};
use datasynth_core::models::internal_control::{ControlType, InternalControl, SoxAssertion};
use datasynth_core::models::{ChartOfAccounts, CoAComplexity, IndustrySector};
use datasynth_runtime::enhanced_orchestrator::{
    AccountingStandardsSnapshot, AnomalyLabels, AuditSnapshot, BalanceValidationResult,
    BankingSnapshot, ComplianceRegulationsSnapshot, DocumentFlowSnapshot, EnhancedGenerationResult,
    EnhancedGenerationStatistics, EsgSnapshot, FinancialReportingSnapshot, GraphExportSnapshot,
    HrSnapshot, IntercompanySnapshot, ManufacturingSnapshot, MasterDataSnapshot, OcpmSnapshot,
    ProjectAccountingSnapshot, SalesKpiBudgetsSnapshot, SourcingSnapshot, SubledgerSnapshot,
    TaxSnapshot, TreasurySnapshot,
};
use rust_decimal::Decimal;
use uuid::Uuid;

// ──────────────────────────── Builder ────────────────────────

/// Build a minimal `EnhancedGenerationResult` with empty collections.
fn empty_result() -> EnhancedGenerationResult {
    EnhancedGenerationResult {
        chart_of_accounts: ChartOfAccounts::new(
            "TEST-COA".into(),
            "Test CoA".into(),
            "US".into(),
            IndustrySector::Technology,
            CoAComplexity::Small,
        ),
        master_data: MasterDataSnapshot::default(),
        document_flows: DocumentFlowSnapshot::default(),
        subledger: SubledgerSnapshot::default(),
        ocpm: OcpmSnapshot::default(),
        audit: AuditSnapshot::default(),
        banking: BankingSnapshot::default(),
        graph_export: GraphExportSnapshot::default(),
        sourcing: SourcingSnapshot::default(),
        financial_reporting: FinancialReportingSnapshot::default(),
        hr: HrSnapshot::default(),
        accounting_standards: AccountingStandardsSnapshot::default(),
        manufacturing: ManufacturingSnapshot::default(),
        sales_kpi_budgets: SalesKpiBudgetsSnapshot::default(),
        tax: TaxSnapshot::default(),
        esg: EsgSnapshot::default(),
        treasury: TreasurySnapshot::default(),
        project_accounting: ProjectAccountingSnapshot::default(),
        process_evolution: Vec::new(),
        organizational_events: Vec::new(),
        disruption_events: Vec::new(),
        intercompany: IntercompanySnapshot::default(),
        journal_entries: Vec::new(),
        anomaly_labels: AnomalyLabels::default(),
        balance_validation: BalanceValidationResult::default(),
        data_quality_stats: Default::default(),
        statistics: EnhancedGenerationStatistics::default(),
        lineage: None,
        gate_result: None,
        internal_controls: Vec::new(),
        opening_balances: Vec::new(),
        subledger_reconciliation: Vec::new(),
        counterfactual_pairs: Vec::new(),
        red_flags: Vec::new(),
        collusion_rings: Vec::new(),
        temporal_vendor_chains: Vec::new(),
        entity_relationship_graph: None,
        cross_process_links: Vec::new(),
        industry_output: None,
        compliance_regulations: ComplianceRegulationsSnapshot::default(),
        sod_violations: Vec::new(),
    }
}

// ──────────────────────────── Test Data Factories ────────────────────────

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn make_po(id: &str) -> PurchaseOrder {
    PurchaseOrder::new(id, "1000", "V-000001", 2025, 1, date(2025, 1, 15), "USER01")
}

fn make_vendor_invoice(id: &str, po_id: Option<&str>) -> VendorInvoice {
    let mut inv = VendorInvoice::new(
        id,
        "1000",
        "V-000001",
        "EXT-INV-001",
        2025,
        1,
        date(2025, 1, 20),
        "USER01",
    );
    inv.purchase_order_id = po_id.map(String::from);
    inv
}

fn make_goods_receipt(id: &str, po_id: Option<&str>) -> GoodsReceipt {
    let mut gr = GoodsReceipt::new(
        id,
        "1000",
        "PLANT01",
        "SLOC01",
        2025,
        1,
        date(2025, 1, 18),
        "USER01",
    );
    gr.purchase_order_id = po_id.map(String::from);
    gr
}

fn make_payment(id: &str, invoice_id: &str) -> Payment {
    use datasynth_core::models::documents::DocumentType;
    let mut pay = Payment::new_ap_payment(
        id,
        "1000",
        "V-000001",
        Decimal::from(1000),
        2025,
        1,
        date(2025, 2, 15),
        "USER01",
    );
    pay.allocate_to_invoice(
        invoice_id,
        DocumentType::VendorInvoice,
        Decimal::from(1000),
        Decimal::ZERO,
    );
    pay
}

fn make_sales_order(id: &str) -> SalesOrder {
    SalesOrder::new(id, "1000", "C-000001", 2025, 1, date(2025, 1, 10), "USER01")
}

fn make_delivery(id: &str, so_id: Option<&str>) -> Delivery {
    let mut del = Delivery::new(
        id,
        "1000",
        "C-000001",
        "SHIP01",
        2025,
        1,
        date(2025, 1, 12),
        "USER01",
    );
    del.sales_order_id = so_id.map(String::from);
    del
}

fn make_customer_invoice(
    id: &str,
    so_id: Option<&str>,
    delivery_id: Option<&str>,
) -> CustomerInvoice {
    let mut inv = CustomerInvoice::new(
        id,
        "1000",
        "C-000001",
        2025,
        1,
        date(2025, 1, 20),
        date(2025, 2, 19),
        "USER01",
    );
    inv.sales_order_id = so_id.map(String::from);
    inv.delivery_id = delivery_id.map(String::from);
    inv
}

fn make_control(id: &str, name: &str, assertion: SoxAssertion) -> InternalControl {
    InternalControl::new(id, name, ControlType::Detective, "Test objective")
        .with_assertion(assertion)
}

fn make_risk(risk_ref: &str, account_or_process: &str) -> RiskAssessment {
    let mut risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        account_or_process,
        &format!("Risk for {account_or_process}"),
    );
    risk.risk_ref = risk_ref.to_string();
    risk
}

fn make_finding(
    finding_ref: &str,
    control_ids: Vec<String>,
    workpaper_id: Option<String>,
) -> AuditFinding {
    let mut finding = AuditFinding::new(
        Uuid::new_v4(),
        FindingType::ControlDeficiency,
        "Test finding",
    );
    finding.finding_ref = finding_ref.to_string();
    finding.related_control_ids = control_ids;
    finding.workpaper_id = workpaper_id;
    finding
}

// ──────────────────────────── DocumentChain Tests ────────────────────────

fn build_doc_chain_result() -> EnhancedGenerationResult {
    let mut result = empty_result();

    // P2P documents
    result
        .document_flows
        .purchase_orders
        .push(make_po("PO-001"));
    result
        .document_flows
        .vendor_invoices
        .push(make_vendor_invoice("VI-001", Some("PO-001")));
    result
        .document_flows
        .goods_receipts
        .push(make_goods_receipt("GR-001", Some("PO-001")));
    result
        .document_flows
        .payments
        .push(make_payment("PAY-001", "VI-001"));

    // O2C documents
    result
        .document_flows
        .sales_orders
        .push(make_sales_order("SO-001"));
    result
        .document_flows
        .deliveries
        .push(make_delivery("DEL-001", Some("SO-001")));
    result
        .document_flows
        .customer_invoices
        .push(make_customer_invoice(
            "CI-001",
            Some("SO-001"),
            Some("DEL-001"),
        ));

    result
}

fn register_doc_ids(id_map: &mut IdMap) {
    for ext_id in &[
        "PO-001", "VI-001", "GR-001", "PAY-001", "SO-001", "DEL-001", "CI-001",
    ] {
        id_map.get_or_insert(ext_id);
    }
}

#[test]
fn document_chain_produces_p2p_edges() {
    let ds_result = build_doc_chain_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_doc_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    // P2P: VI->PO (60), PAY->VI (62), GR->PO (64)
    let p2p_edges: Vec<_> = edges.iter().filter(|e| e.edge_type <= 64).collect();
    assert_eq!(
        p2p_edges.len(),
        3,
        "Expected 3 P2P edges, got {}",
        p2p_edges.len()
    );

    // Check ReferencesOrder (60): VI-001 -> PO-001
    let ref_order = edges.iter().find(|e| e.edge_type == 60).unwrap();
    assert_eq!(ref_order.source, id_map.get("VI-001").unwrap());
    assert_eq!(ref_order.target, id_map.get("PO-001").unwrap());

    // Check PaysInvoice (62): PAY-001 -> VI-001
    let pays = edges.iter().find(|e| e.edge_type == 62).unwrap();
    assert_eq!(pays.source, id_map.get("PAY-001").unwrap());
    assert_eq!(pays.target, id_map.get("VI-001").unwrap());

    // Check FulfillsOrder (64): GR-001 -> PO-001
    let fulfills = edges.iter().find(|e| e.edge_type == 64).unwrap();
    assert_eq!(fulfills.source, id_map.get("GR-001").unwrap());
    assert_eq!(fulfills.target, id_map.get("PO-001").unwrap());
}

#[test]
fn document_chain_produces_o2c_edges() {
    let ds_result = build_doc_chain_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_doc_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    // O2C: CI->SO (66), DEL->SO (68), CI->DEL (69)
    let o2c_edges: Vec<_> = edges.iter().filter(|e| e.edge_type >= 66).collect();
    assert_eq!(
        o2c_edges.len(),
        3,
        "Expected 3 O2C edges, got {}",
        o2c_edges.len()
    );

    // Check BillsOrder (66): CI-001 -> SO-001
    let bills = edges.iter().find(|e| e.edge_type == 66).unwrap();
    assert_eq!(bills.source, id_map.get("CI-001").unwrap());
    assert_eq!(bills.target, id_map.get("SO-001").unwrap());

    // Check DeliversOrder (68): DEL-001 -> SO-001
    let delivers = edges.iter().find(|e| e.edge_type == 68).unwrap();
    assert_eq!(delivers.source, id_map.get("DEL-001").unwrap());
    assert_eq!(delivers.target, id_map.get("SO-001").unwrap());

    // Check InvoiceReferencesDelivery (69): CI-001 -> DEL-001
    let inv_del = edges.iter().find(|e| e.edge_type == 69).unwrap();
    assert_eq!(inv_del.source, id_map.get("CI-001").unwrap());
    assert_eq!(inv_del.target, id_map.get("DEL-001").unwrap());
}

#[test]
fn document_chain_skips_missing_fk() {
    let mut ds_result = build_doc_chain_result();
    ds_result.document_flows.vendor_invoices[0].purchase_order_id = None;

    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_doc_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    assert!(
        edges.iter().all(|e| e.edge_type != 60),
        "Should not produce ReferencesOrder edge when PO FK is None"
    );
}

#[test]
fn document_chain_skips_budget_dropped_nodes() {
    let ds_result = build_doc_chain_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();

    // Only register PO-001 and VI-001
    id_map.get_or_insert("PO-001");
    id_map.get_or_insert("VI-001");

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge_type, 60);
}

#[test]
fn document_chain_total_edge_count() {
    let ds_result = build_doc_chain_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_doc_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    assert_eq!(edges.len(), 6, "3 P2P + 3 O2C = 6");

    let edge_types: std::collections::HashSet<u32> = edges.iter().map(|e| e.edge_type).collect();
    for code in &[60, 62, 64, 66, 68, 69] {
        assert!(edge_types.contains(code), "Missing edge type code {code}");
    }
}

#[test]
fn document_chain_edge_types_exclude_accounting() {
    let ds_result = build_doc_chain_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_doc_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::document_chain::DocumentChainEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    let types: std::collections::HashSet<u32> = edges.iter().map(|e| e.edge_type).collect();

    assert!(
        !types.contains(&70),
        "Code 70 (PostsToAccount) belongs in AccountingEdgeSynthesizer"
    );
    assert!(
        !types.contains(&99),
        "Code 99 (DocPostsJe) belongs in AccountingEdgeSynthesizer"
    );
}

// ──────────────────────────── RiskControl Tests ────────────────────────

fn build_risk_control_result() -> EnhancedGenerationResult {
    let mut result = empty_result();

    let mut c1 = make_control(
        "C020",
        "Revenue Recognition Review",
        SoxAssertion::Valuation,
    );
    c1.mitigates_risk_ids = vec!["R001".to_string()];
    c1.owner_employee_id = Some("EMP-001".to_string());

    let c2 = make_control("C010", "Three-Way Match", SoxAssertion::Completeness);
    let c3 = make_control("C001", "Cash Account Daily Review", SoxAssertion::Existence);

    result.internal_controls = vec![c1, c2, c3];

    let r1 = make_risk("R001", "Revenue");
    let r2 = make_risk("R002", "Expenditure");
    let r3 = make_risk("R003", "Cash");

    result.audit.risk_assessments = vec![r1, r2, r3];

    let f1 = make_finding(
        "FIND-001",
        vec!["C020".to_string()],
        Some("WP-001".to_string()),
    );
    result.audit.findings = vec![f1];

    result
}

fn register_risk_control_ids(id_map: &mut IdMap) {
    for ext_id in &[
        "C020", "C010", "C001", "R001", "R002", "R003", "EMP-001", "FIND-001", "WP-001",
    ] {
        id_map.get_or_insert(ext_id);
    }
}

#[test]
fn risk_control_produces_fk_edges() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    // Check RISK_MITIGATED_BY (75): R001 -> C020 (FK match, weight=1.0)
    let r001_id = id_map.get("R001").unwrap();
    let c020_id = id_map.get("C020").unwrap();
    let fk_edge = edges.iter().find(|e| {
        e.edge_type == 75 && e.source == r001_id && e.target == c020_id && e.weight == 1.0
    });
    assert!(
        fk_edge.is_some(),
        "Should have FK edge R001 -> C020 with weight 1.0"
    );
}

#[test]
fn risk_control_produces_name_matched_edges() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    // R002 (Expenditure) -> C010 (Three-Way Match = Expenditure domain)
    let r002_id = id_map.get("R002").unwrap();
    let c010_id = id_map.get("C010").unwrap();
    let name_match = edges
        .iter()
        .find(|e| e.edge_type == 75 && e.source == r002_id && e.target == c010_id);
    assert!(
        name_match.is_some(),
        "R002 (Expenditure) should match C010 (Three-Way Match)"
    );
    assert!(
        name_match.unwrap().weight < 1.0,
        "Name-matched edge should have weight < 1.0"
    );
}

#[test]
fn risk_control_direction_is_risk_to_control() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let risk_ids: std::collections::HashSet<u64> = ["R001", "R002", "R003"]
        .iter()
        .filter_map(|r| id_map.get(r))
        .collect();
    let control_ids: std::collections::HashSet<u64> = ["C020", "C010", "C001"]
        .iter()
        .filter_map(|c| id_map.get(c))
        .collect();

    for edge in edges.iter().filter(|e| e.edge_type == 75) {
        assert!(
            risk_ids.contains(&edge.source),
            "Source {} should be a risk",
            edge.source
        );
        assert!(
            control_ids.contains(&edge.target),
            "Target {} should be a control",
            edge.target
        );
    }
}

#[test]
fn risk_control_produces_owned_by_edges() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let owned_by: Vec<_> = edges.iter().filter(|e| e.edge_type == 127).collect();
    assert_eq!(owned_by.len(), 1, "Only C020 has owner_employee_id");
    assert_eq!(owned_by[0].source, id_map.get("C020").unwrap());
    assert_eq!(owned_by[0].target, id_map.get("EMP-001").unwrap());
}

#[test]
fn risk_control_produces_finding_edges() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let finding_edges: Vec<_> = edges.iter().filter(|e| e.edge_type == 128).collect();
    assert_eq!(finding_edges.len(), 1);
    assert_eq!(finding_edges[0].source, id_map.get("FIND-001").unwrap());
    assert_eq!(finding_edges[0].target, id_map.get("C020").unwrap());
}

#[test]
fn risk_control_produces_workpaper_tests_control_edges() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let wp_edges: Vec<_> = edges.iter().filter(|e| e.edge_type == 129).collect();
    assert_eq!(wp_edges.len(), 1);
    assert_eq!(wp_edges[0].source, id_map.get("WP-001").unwrap());
    assert_eq!(wp_edges[0].target, id_map.get("C020").unwrap());
}

#[test]
fn risk_control_skips_unregistered_nodes() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();

    // Only register some nodes
    id_map.get_or_insert("C020");
    id_map.get_or_insert("R001");

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let risk_mitigated: Vec<_> = edges.iter().filter(|e| e.edge_type == 75).collect();
    assert_eq!(risk_mitigated.len(), 1);
    assert_eq!(risk_mitigated[0].source, id_map.get("R001").unwrap());
    assert_eq!(risk_mitigated[0].target, id_map.get("C020").unwrap());

    // No CONTROL_OWNED_BY since EMP-001 is not registered
    assert!(edges.iter().all(|e| e.edge_type != 127));
}

#[test]
fn risk_control_empty_inputs_produce_no_edges() {
    let ds_result = empty_result();
    let config = ExportConfig::default();
    let id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    assert!(edges.is_empty());
}

#[test]
fn risk_control_edge_types_are_correct() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();
    let types: std::collections::HashSet<u32> = edges.iter().map(|e| e.edge_type).collect();

    assert!(types.contains(&75), "Missing RISK_MITIGATED_BY (75)");
    assert!(types.contains(&127), "Missing CONTROL_OWNED_BY (127)");
    assert!(types.contains(&128), "Missing CONTROL_HAS_FINDING (128)");
    assert!(
        types.contains(&129),
        "Missing WORKPAPER_TESTS_CONTROL (129)"
    );
}

#[test]
fn risk_control_all_edges_reference_valid_ids() {
    let ds_result = build_risk_control_result();
    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    register_risk_control_ids(&mut id_map);

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    for edge in &edges {
        assert!(
            id_map.reverse_get(edge.source).is_some(),
            "Edge source {} not in id_map (edge_type={})",
            edge.source,
            edge.edge_type
        );
        assert!(
            id_map.reverse_get(edge.target).is_some(),
            "Edge target {} not in id_map (edge_type={})",
            edge.target,
            edge.edge_type
        );
    }
}

// ──────────────────────────── Module-Level Tests ────────────────────────

#[test]
fn all_edge_synthesizers_returns_all() {
    let synthesizers = datasynth_graph_export::edges::all_synthesizers();
    assert_eq!(synthesizers.len(), 13);
    assert_eq!(synthesizers[0].name(), "document_chain");
    assert_eq!(synthesizers[1].name(), "risk_control");
    assert_eq!(synthesizers[2].name(), "audit_trail");
    assert_eq!(synthesizers[3].name(), "banking");
    assert_eq!(synthesizers[4].name(), "s2c");
    assert_eq!(synthesizers[5].name(), "h2r");
    assert_eq!(synthesizers[6].name(), "mfg");
    assert_eq!(synthesizers[7].name(), "accounting");
    assert_eq!(synthesizers[8].name(), "entity_relationships");
    assert_eq!(synthesizers[9].name(), "process_sequence");
    assert_eq!(synthesizers[10].name(), "audit_procedures");
    assert_eq!(synthesizers[11].name(), "v130_edges");
    assert_eq!(synthesizers[12].name(), "v140_edges");
}

#[test]
fn standard_pipeline_includes_edge_synthesizers() {
    use datasynth_graph_export::GraphExportPipeline;

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let debug_str = format!("{:?}", pipeline);
    assert!(debug_str.contains("edge_synthesizers: 13"));
}

#[test]
fn workpaper_tests_control_deduplicates() {
    // Two findings referencing the same (workpaper, control) pair
    // should produce only one edge.
    let mut ds_result = empty_result();

    let f1 = make_finding(
        "FIND-001",
        vec!["C001".to_string()],
        Some("WP-001".to_string()),
    );
    let f2 = make_finding(
        "FIND-002",
        vec!["C001".to_string()],
        Some("WP-001".to_string()),
    );
    ds_result.audit.findings = vec![f1, f2];
    ds_result.internal_controls =
        vec![make_control("C001", "Cash Review", SoxAssertion::Existence)];

    let config = ExportConfig::default();
    let mut id_map = IdMap::new();
    let mut warnings = ExportWarnings::new();
    id_map.get_or_insert("C001");
    id_map.get_or_insert("WP-001");
    id_map.get_or_insert("FIND-001");
    id_map.get_or_insert("FIND-002");

    let synthesizer = datasynth_graph_export::edges::risk_control::RiskControlEdgeSynthesizer;
    let mut ctx = EdgeSynthesisContext {
        ds_result: &ds_result,
        config: &config,
        id_map: &id_map,
        warnings: &mut warnings,
    };

    let edges = synthesizer.synthesize(&mut ctx).unwrap();

    let wp_edges: Vec<_> = edges.iter().filter(|e| e.edge_type == 129).collect();
    assert_eq!(wp_edges.len(), 1, "Should deduplicate WP-001 -> C001");
}
