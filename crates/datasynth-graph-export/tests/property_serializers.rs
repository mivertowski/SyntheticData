//! Integration tests for property serializers.
//!
//! Tests the `ControlPropertySerializer` and `RiskPropertySerializer` against
//! a real (minimal) `EnhancedGenerationResult` to verify field mapping correctness.

#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use datasynth_graph_export::config::ExportConfig;
use datasynth_graph_export::properties::control::ControlPropertySerializer;
use datasynth_graph_export::properties::risk::RiskPropertySerializer;
use datasynth_graph_export::traits::{PropertySerializer, SerializationContext};

use datasynth_core::models::audit::risk::{RiskAssessment, RiskCategory};
use datasynth_core::models::audit::RiskLevel as AuditRiskLevel;
use datasynth_core::models::{ChartOfAccounts, CoAComplexity, IndustrySector};
use datasynth_core::models::{
    ControlEffectiveness, ControlFrequency, ControlType, InternalControl, RiskLevel, SoxAssertion,
    TestResult,
};

use datasynth_runtime::enhanced_orchestrator::{
    AccountingStandardsSnapshot, AnomalyLabels, AuditSnapshot, BalanceValidationResult,
    BankingSnapshot, ComplianceRegulationsSnapshot, DocumentFlowSnapshot, EnhancedGenerationResult,
    EnhancedGenerationStatistics, EsgSnapshot, FinancialReportingSnapshot, GraphExportSnapshot,
    HrSnapshot, IntercompanySnapshot, ManufacturingSnapshot, MasterDataSnapshot, OcpmSnapshot,
    ProjectAccountingSnapshot, SalesKpiBudgetsSnapshot, SourcingSnapshot, SubledgerSnapshot,
    TaxSnapshot, TreasurySnapshot,
};

use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────

/// Build a minimal `EnhancedGenerationResult` with the given controls and risks.
fn build_ds_result(
    controls: Vec<InternalControl>,
    risks: Vec<RiskAssessment>,
) -> EnhancedGenerationResult {
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
        audit: AuditSnapshot {
            risk_assessments: risks,
            ..AuditSnapshot::default()
        },
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
        internal_controls: controls,
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

/// Build a `SerializationContext` from a `ds_result` and default config.
fn build_context<'a>(
    ds_result: &'a EnhancedGenerationResult,
    config: &'a ExportConfig,
    employee_by_id: &'a HashMap<String, String>,
    employee_by_role: &'a HashMap<String, String>,
    user_to_employee: &'a HashMap<String, String>,
    opening_balances: &'a HashMap<String, f64>,
    risk_names: &'a HashMap<String, String>,
) -> SerializationContext<'a> {
    SerializationContext {
        ds_result,
        config,
        employee_by_id,
        employee_by_role,
        user_to_employee,
        opening_balances,
        risk_names,
    }
}

fn make_test_control() -> InternalControl {
    InternalControl::new(
        "C001",
        "Cash Account Daily Review",
        ControlType::Detective,
        "Review all cash transactions daily",
    )
    .with_frequency(ControlFrequency::Daily)
    .with_risk_level(RiskLevel::High)
    .as_key_control()
    .with_assertion(SoxAssertion::Existence)
    .with_description("Daily reconciliation of cash accounts")
    .with_owner_employee("EMP-001", "Jane Smith")
    .with_test_history(
        3,
        Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 15).unwrap()),
        TestResult::Pass,
    )
    .with_effectiveness(ControlEffectiveness::Effective)
    .with_mitigates_risk_ids(vec!["RISK-001".into(), "RISK-002".into()])
    .with_covers_account_classes(vec!["Assets".into()])
}

fn make_test_risk() -> RiskAssessment {
    let engagement_id = Uuid::new_v4();
    RiskAssessment::new(
        engagement_id,
        RiskCategory::AssertionLevel,
        "Revenue",
        "Risk of fictitious revenue recognition",
    )
    .with_risk_levels(AuditRiskLevel::High, AuditRiskLevel::Medium)
    .mark_significant("Presumed fraud risk per ISA 240")
    .with_assessed_by(
        "EMP-002",
        chrono::NaiveDate::from_ymd_opt(2025, 11, 1).unwrap(),
    )
}

// ── Control Serializer Tests ─────────────────────────────────

#[test]
fn control_serializer_entity_type() {
    let s = ControlPropertySerializer;
    assert_eq!(s.entity_type(), "internal_control");
}

#[test]
fn control_serializer_produces_required_fields() {
    let control = make_test_control();
    let control_id = control.control_id.clone();
    let ds_result = build_ds_result(vec![control], vec![]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = ControlPropertySerializer;
    let props = s
        .serialize(&control_id, &ctx)
        .expect("serialize should return Some");

    // Identity
    assert!(props.contains_key("controlId"));
    assert_eq!(props["controlId"], serde_json::json!("C001"));
    assert!(props.contains_key("name"));
    assert_eq!(
        props["name"],
        serde_json::json!("Cash Account Daily Review")
    );
    assert!(props.contains_key("description"));

    // Classification
    assert!(props.contains_key("controlType"));
    assert_eq!(props["controlType"], serde_json::json!("Detective"));
    assert!(props.contains_key("isKeyControl"));
    assert_eq!(props["isKeyControl"], serde_json::json!(true));
    assert!(props.contains_key("category"));

    // SOX / COSO
    assert!(props.contains_key("soxAssertion"));
    assert_eq!(props["soxAssertion"], serde_json::json!("Existence"));
    assert!(props.contains_key("cosoComponent"));
    assert!(props.contains_key("controlScope"));
    assert!(props.contains_key("objective"));

    // Operational
    assert!(props.contains_key("frequency"));
    assert_eq!(props["frequency"], serde_json::json!("Daily"));
    assert!(props.contains_key("owner"));
    assert_eq!(props["owner"], serde_json::json!("Jane Smith"));
    assert!(props.contains_key("riskLevel"));
    assert_eq!(props["riskLevel"], serde_json::json!("High"));

    // Test history + effectiveness
    assert!(props.contains_key("effectiveness"));
    assert_eq!(props["effectiveness"], serde_json::json!("Effective"));
    assert!(props.contains_key("testCount"));
    assert_eq!(props["testCount"], serde_json::json!(3));
    assert!(props.contains_key("testResult"));
    assert_eq!(props["testResult"], serde_json::json!("Pass"));
    assert!(props.contains_key("lastTestedDate"));
    assert_eq!(props["lastTestedDate"], serde_json::json!("2025-12-15"));

    // Risk linkage
    assert!(props.contains_key("linkedRiskIds"));
    assert_eq!(
        props["linkedRiskIds"],
        serde_json::json!(["RISK-001", "RISK-002"])
    );

    // Account class coverage
    assert!(props.contains_key("coversAccountClasses"));
    assert_eq!(props["coversAccountClasses"], serde_json::json!(["Assets"]));
}

#[test]
fn control_serializer_returns_none_for_unknown_id() {
    let control = make_test_control();
    let ds_result = build_ds_result(vec![control], vec![]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = ControlPropertySerializer;
    assert!(s.serialize("NONEXISTENT", &ctx).is_none());
}

#[test]
fn control_serializer_owner_fallback_to_role() {
    // Control with empty owner_name => should fallback to role Debug format
    let mut control = InternalControl::new(
        "C999",
        "Test Control No Owner",
        ControlType::Preventive,
        "Test objective",
    );
    control.owner_name = String::new(); // empty name

    let ds_result = build_ds_result(vec![control], vec![]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = ControlPropertySerializer;
    let props = s.serialize("C999", &ctx).expect("should find control");

    // When owner_name is empty, should format from owner_role (snake_case via serde)
    let owner = props["owner"].as_str().unwrap();
    assert!(!owner.is_empty());
    assert!(owner.contains("controller")); // default owner_role, snake_case
}

#[test]
fn control_serializer_omits_empty_optional_fields() {
    // Control with no risk IDs and no account classes
    let control = InternalControl::new(
        "C888",
        "Minimal Control",
        ControlType::Monitoring,
        "Minimal",
    );

    let ds_result = build_ds_result(vec![control], vec![]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = ControlPropertySerializer;
    let props = s.serialize("C888", &ctx).unwrap();

    // Empty vecs should not produce keys
    assert!(!props.contains_key("linkedRiskIds"));
    assert!(!props.contains_key("coversAccountClasses"));
    // No last_tested_date since test_count is 0
    assert!(!props.contains_key("lastTestedDate"));
}

// ── Risk Serializer Tests ────────────────────────────────────

#[test]
fn risk_serializer_entity_type() {
    let s = RiskPropertySerializer;
    assert_eq!(s.entity_type(), "risk_assessment");
}

#[test]
fn risk_serializer_produces_required_fields() {
    let risk = make_test_risk();
    let risk_ref = risk.risk_ref.clone();
    let ds_result = build_ds_result(vec![], vec![risk]);
    let config = ExportConfig::default();
    let mut emp_id = HashMap::new();
    emp_id.insert("EMP-002".into(), "John Doe".into());
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = RiskPropertySerializer;
    let props = s
        .serialize(&risk_ref, &ctx)
        .expect("serialize should return Some");

    // Identity
    assert!(props.contains_key("riskRef"));
    assert_eq!(props["riskRef"], serde_json::json!(risk_ref));
    assert!(props.contains_key("name"));
    assert!(props.contains_key("description"));
    assert_eq!(
        props["description"],
        serde_json::json!("Risk of fictitious revenue recognition")
    );

    // Classification
    assert!(props.contains_key("category"));
    assert!(props.contains_key("accountOrProcess"));
    assert_eq!(props["accountOrProcess"], serde_json::json!("Revenue"));

    // Risk scores (continuous)
    assert!(props.contains_key("inherentImpact"));
    assert!(props.contains_key("inherentLikelihood"));
    assert!(props.contains_key("residualImpact"));
    assert!(props.contains_key("residualLikelihood"));
    assert!(props.contains_key("riskScore"));

    // Values should be positive numbers
    let ii = props["inherentImpact"].as_f64().unwrap();
    let il = props["inherentLikelihood"].as_f64().unwrap();
    let ri = props["residualImpact"].as_f64().unwrap();
    let rl = props["residualLikelihood"].as_f64().unwrap();
    let rs = props["riskScore"].as_f64().unwrap();
    assert!(ii > 0.0 && ii <= 1.0, "inherentImpact={ii} out of range");
    assert!(
        il > 0.0 && il <= 1.0,
        "inherentLikelihood={il} out of range"
    );
    assert!(ri > 0.0 && ri <= 1.0, "residualImpact={ri} out of range");
    assert!(
        rl > 0.0 && rl <= 1.0,
        "residualLikelihood={rl} out of range"
    );
    assert!(rs > 0.0, "riskScore should be positive");

    // Risk levels (snake_case via serde)
    assert!(props.contains_key("inherentRisk"));
    assert_eq!(props["inherentRisk"], serde_json::json!("high"));
    assert!(props.contains_key("controlRisk"));
    assert_eq!(props["controlRisk"], serde_json::json!("medium"));
    assert!(props.contains_key("riskOfMaterialMisstatement"));

    // Significance
    assert!(props.contains_key("isSignificant"));
    assert_eq!(props["isSignificant"], serde_json::json!(true));
    assert!(props.contains_key("significantRiskRationale"));
    assert_eq!(
        props["significantRiskRationale"],
        serde_json::json!("Presumed fraud risk per ISA 240")
    );

    // Lifecycle (snake_case via serde)
    assert!(props.contains_key("status"));
    assert_eq!(props["status"], serde_json::json!("active"));

    // Owner resolved from employee map
    assert!(props.contains_key("owner"));
    assert_eq!(props["owner"], serde_json::json!("John Doe"));

    // Control linkage
    assert!(props.contains_key("mitigatingControlCount"));
    assert!(props.contains_key("effectiveControlCount"));
}

#[test]
fn risk_serializer_returns_none_for_unknown_ref() {
    let risk = make_test_risk();
    let ds_result = build_ds_result(vec![], vec![risk]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = RiskPropertySerializer;
    assert!(s.serialize("NONEXISTENT-REF", &ctx).is_none());
}

#[test]
fn risk_serializer_owner_fallback_to_assessed_by() {
    // Risk with assessed_by that's NOT in employee_by_id => uses raw assessed_by
    let risk = make_test_risk();
    let risk_ref = risk.risk_ref.clone();
    let ds_result = build_ds_result(vec![], vec![risk]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new(); // empty => no resolution
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = RiskPropertySerializer;
    let props = s.serialize(&risk_ref, &ctx).unwrap();

    // Should fall back to "EMP-002"
    assert_eq!(props["owner"], serde_json::json!("EMP-002"));
}

#[test]
fn risk_serializer_omits_empty_optional_fields() {
    // Risk with no fraud factors, no related controls, no assertion, no significant rationale
    let risk = RiskAssessment::new(
        Uuid::new_v4(),
        RiskCategory::AssertionLevel,
        "Payroll",
        "Payroll accuracy risk",
    );
    let risk_ref = risk.risk_ref.clone();
    let ds_result = build_ds_result(vec![], vec![risk]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let s = RiskPropertySerializer;
    let props = s.serialize(&risk_ref, &ctx).unwrap();

    // Empty vecs and None optionals should not produce keys
    assert!(!props.contains_key("fraudRiskFactors"));
    assert!(!props.contains_key("relatedControls"));
    assert!(!props.contains_key("assertion"));
    assert!(!props.contains_key("significantRiskRationale"));
    // assessed_by is empty string => no owner key
    assert!(!props.contains_key("owner"));
}

// ── all_serializers Tests ────────────────────────────────────

#[test]
fn all_serializers_returns_both() {
    let serializers = datasynth_graph_export::properties::all_serializers();
    assert_eq!(serializers.len(), 41);

    let types: Vec<&str> = serializers.iter().map(|s| s.entity_type()).collect();
    // Original Task 8 serializers
    assert!(types.contains(&"internal_control"));
    assert!(types.contains(&"risk_assessment"));
    // Task 9 additions — spot-check representative types
    assert!(types.contains(&"journal_entry"));
    assert!(types.contains(&"gl_account"));
    assert!(types.contains(&"employee"));
    assert!(types.contains(&"vendor"));
    assert!(types.contains(&"customer"));
    assert!(types.contains(&"purchase_order"));
    assert!(types.contains(&"sales_order"));
    assert!(types.contains(&"banking_customer"));
    assert!(types.contains(&"audit_engagement"));
    assert!(types.contains(&"sourcing_project"));
    assert!(types.contains(&"payroll_run"));
    assert!(types.contains(&"production_order"));
    // Task 14 additions — audit procedure serializers
    assert!(types.contains(&"external_confirmation"));
    assert!(types.contains(&"confirmation_response"));
    assert!(types.contains(&"audit_procedure_step"));
    assert!(types.contains(&"audit_sample"));
    assert!(types.contains(&"analytical_procedure_result"));
    assert!(types.contains(&"internal_audit_function"));
    assert!(types.contains(&"internal_audit_report"));
    assert!(types.contains(&"related_party"));
    assert!(types.contains(&"related_party_transaction"));
}

// ── Cross-Serializer Tests ───────────────────────────────────

#[test]
fn control_and_risk_serialize_from_same_result() {
    let control = make_test_control();
    let risk = make_test_risk();
    let control_id = control.control_id.clone();
    let risk_ref = risk.risk_ref.clone();

    let ds_result = build_ds_result(vec![control], vec![risk]);
    let config = ExportConfig::default();
    let emp_id = HashMap::new();
    let emp_role = HashMap::new();
    let u2e = HashMap::new();
    let ob = HashMap::new();
    let rn = HashMap::new();
    let ctx = build_context(&ds_result, &config, &emp_id, &emp_role, &u2e, &ob, &rn);

    let ctrl_s = ControlPropertySerializer;
    let risk_s = RiskPropertySerializer;

    let ctrl_props = ctrl_s.serialize(&control_id, &ctx).unwrap();
    let risk_props = risk_s.serialize(&risk_ref, &ctx).unwrap();

    // Both should have produced non-empty property maps
    assert!(
        ctrl_props.len() >= 15,
        "control should have at least 15 properties, got {}",
        ctrl_props.len()
    );
    assert!(
        risk_props.len() >= 12,
        "risk should have at least 12 properties, got {}",
        risk_props.len()
    );

    // Both should serialize to valid JSON
    let ctrl_json = serde_json::to_string(&ctrl_props).unwrap();
    let risk_json = serde_json::to_string(&risk_props).unwrap();
    assert!(!ctrl_json.is_empty());
    assert!(!risk_json.is_empty());
}
