//! Tests for the 8 new domain builder methods added to HypergraphBuilder.

use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal_macros::dec;

use datasynth_core::models::intercompany::{EliminationEntry, EliminationType, ICMatchedPair};
use datasynth_core::models::organizational_event::ReorganizationConfig;
use datasynth_core::models::process_evolution::{PolicyCategory, PolicyChangeConfig};
use datasynth_core::models::AssessmentMethod;
use datasynth_core::models::{
    AssuranceLevel, CashForecast, CashPosition, ClimateScenario, DebtInstrument, EarnedValueMetric,
    EmissionRecord, EmissionScope, EsgDisclosure, EsgFramework, EsgRiskFlag, EstimationMethod,
    HedgeRelationship, OrganizationalEvent, OrganizationalEventType, ProcessEvolutionEvent,
    ProcessEvolutionType, Project, ProjectMilestone, ScenarioType, SupplierEsgAssessment, TaxCode,
    TaxJurisdiction, TaxLine, TaxProvision, TaxReturn, TimeHorizon, WithholdingTaxRecord,
};
use datasynth_generators::disruption::{
    DisruptionEvent, DisruptionType, OutageCause, OutageConfig,
};
use datasynth_graph::builders::hypergraph::{HypergraphBuilder, HypergraphConfig};
use datasynth_graph::models::hypergraph::HypergraphLayer;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn default_config() -> HypergraphConfig {
    HypergraphConfig {
        max_nodes: 10_000,
        ..Default::default()
    }
}

// ===========================================================================
// Tax
// ===========================================================================

#[test]
fn builder_creates_tax_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let jurisdictions = vec![TaxJurisdiction::new(
        "US-FED",
        "US Federal",
        "US",
        Default::default(),
    )];
    let codes = vec![TaxCode::new(
        "TC-001",
        "VAT-STD-20",
        "Standard VAT 20%",
        Default::default(),
        dec!(0.20),
        "US-FED",
        date(2024, 1, 1),
    )];
    let tax_lines = vec![TaxLine::new(
        "TL-001",
        Default::default(),
        "INV-001",
        1,
        "TC-001",
        "US-FED",
        dec!(1000),
        dec!(200),
    )];
    let tax_returns = vec![TaxReturn::new(
        "TR-001",
        "ENTITY-1",
        "US-FED",
        date(2024, 1, 1),
        date(2024, 3, 31),
        Default::default(),
        dec!(50000),
        dec!(30000),
        date(2024, 4, 30),
    )];
    let tax_provisions = vec![TaxProvision::new(
        "TP-001",
        "ENTITY-1",
        date(2024, 12, 31),
        dec!(100000),
        dec!(20000),
        dec!(15000),
        dec!(0.21),
        dec!(0.25),
    )];
    let withholdings = vec![WithholdingTaxRecord::new(
        "WHT-001",
        "PMT-001",
        "VND-001",
        Default::default(),
        dec!(0.15),
        dec!(0.10),
        dec!(50000),
    )];

    builder.add_tax_documents(
        &jurisdictions,
        &codes,
        &tax_lines,
        &tax_returns,
        &tax_provisions,
        &withholdings,
    );

    let hg = builder.build();

    // 6 types x 1 each = 6 nodes
    assert_eq!(hg.nodes.len(), 6);
    assert!(hg.nodes.iter().any(|n| n.entity_type == "tax_jurisdiction"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "tax_code"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "tax_line"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "tax_return"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "tax_provision"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "withholding_tax_record"));

    // All tax nodes go to Layer 3 (Accounting Network)
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::AccountingNetwork));

    // Check type codes
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 410));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 411));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 412));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 413));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 414));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 415));
}

#[test]
fn late_tax_return_flagged_as_anomaly() {
    let mut builder = HypergraphBuilder::new(default_config());
    let late_return = TaxReturn::new(
        "TR-LATE",
        "ENTITY-1",
        "US-FED",
        date(2024, 1, 1),
        date(2024, 3, 31),
        Default::default(),
        dec!(50000),
        dec!(30000),
        date(2024, 4, 30),
    )
    .with_filing(date(2024, 5, 15)); // Filed after deadline

    builder.add_tax_documents(&[], &[], &[], &[late_return], &[], &[]);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 1);
    assert!(hg.nodes[0].is_anomaly);
    assert_eq!(hg.nodes[0].anomaly_type.as_deref(), Some("late_filing"));
}

// ===========================================================================
// Treasury
// ===========================================================================

#[test]
fn builder_creates_treasury_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let positions = vec![CashPosition::new(
        "CP-001",
        "ENTITY-1",
        "ACCT-001",
        "USD",
        date(2024, 6, 15),
        dec!(100000),
        dec!(50000),
        dec!(30000),
    )];
    let forecasts = vec![CashForecast {
        id: "CF-001".into(),
        entity_id: "ENTITY-1".into(),
        currency: "USD".into(),
        forecast_date: date(2024, 6, 15),
        horizon_days: 30,
        items: vec![],
        net_position: dec!(200000),
        confidence_level: dec!(0.85),
    }];
    let hedges = vec![HedgeRelationship::new(
        "HR-001",
        Default::default(),
        "FX exposure EUR/USD",
        "INST-001",
        Default::default(),
        date(2024, 1, 1),
        Default::default(),
        dec!(0.95),
    )];
    let debts = vec![DebtInstrument::new(
        "DI-001",
        "ENTITY-1",
        Default::default(),
        "Bank of Test",
        dec!(5000000),
        "USD",
        dec!(0.045),
        Default::default(),
        date(2023, 1, 1),
        date(2028, 1, 1),
    )];

    builder.add_treasury_documents(&positions, &forecasts, &hedges, &debts);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 4);
    assert!(hg.nodes.iter().any(|n| n.entity_type == "cash_position"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "cash_forecast"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "hedge_relationship"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "debt_instrument"));

    // All treasury nodes -> Layer 3
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::AccountingNetwork));

    // Check codes: 420, 421, 422, 423
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 420));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 421));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 422));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 423));
}

#[test]
fn ineffective_hedge_flagged_as_anomaly() {
    let mut builder = HypergraphBuilder::new(default_config());
    // Effectiveness ratio 0.70 is below 80% threshold -> ineffective
    let hedge = HedgeRelationship::new(
        "HR-BAD",
        Default::default(),
        "Bad hedge",
        "INST-002",
        Default::default(),
        date(2024, 1, 1),
        Default::default(),
        dec!(0.70),
    );
    builder.add_treasury_documents(&[], &[], &[hedge], &[]);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 1);
    assert!(hg.nodes[0].is_anomaly);
    assert_eq!(
        hg.nodes[0].anomaly_type.as_deref(),
        Some("ineffective_hedge")
    );
}

// ===========================================================================
// ESG
// ===========================================================================

#[test]
fn builder_creates_esg_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let emissions = vec![EmissionRecord {
        id: "EM-001".into(),
        entity_id: "ENTITY-1".into(),
        scope: EmissionScope::Scope1,
        scope3_category: None,
        facility_id: Some("FAC-001".into()),
        period: date(2024, 6, 1),
        activity_data: None,
        activity_unit: None,
        emission_factor: None,
        co2e_tonnes: dec!(150),
        estimation_method: EstimationMethod::ActivityBased,
        source: None,
    }];
    let disclosures = vec![EsgDisclosure {
        id: "DISC-001".into(),
        entity_id: "ENTITY-1".into(),
        reporting_period_start: date(2024, 1, 1),
        reporting_period_end: date(2024, 12, 31),
        framework: EsgFramework::Gri,
        assurance_level: AssuranceLevel::Limited,
        disclosure_topic: "GHG Emissions".into(),
        metric_value: "150 tCO2e".into(),
        metric_unit: "tCO2e".into(),
        is_assured: true,
    }];
    let assessments = vec![SupplierEsgAssessment {
        id: "SA-001".into(),
        entity_id: "ENTITY-1".into(),
        vendor_id: "VND-001".into(),
        assessment_date: date(2024, 3, 15),
        method: AssessmentMethod::ThirdPartyAudit,
        environmental_score: dec!(72),
        social_score: dec!(68),
        governance_score: dec!(80),
        overall_score: dec!(73),
        risk_flag: EsgRiskFlag::Low,
        corrective_actions_required: 0,
    }];
    let scenarios = vec![ClimateScenario {
        id: "CS-001".into(),
        entity_id: "ENTITY-1".into(),
        scenario_type: ScenarioType::WellBelow2C,
        time_horizon: TimeHorizon::Medium,
        description: "Paris-aligned scenario".into(),
        temperature_rise_c: dec!(1.5),
        transition_risk_impact: dec!(2000000),
        physical_risk_impact: dec!(500000),
        financial_impact: dec!(2500000),
    }];

    builder.add_esg_documents(&emissions, &disclosures, &assessments, &scenarios);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 4);
    assert!(hg.nodes.iter().any(|n| n.entity_type == "emission_record"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "esg_disclosure"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "supplier_esg_assessment"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "climate_scenario"));

    // ESG -> Layer 1 (Governance)
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::GovernanceControls));

    // Check codes: 430, 431, 432, 433
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 430));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 431));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 432));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 433));
}

// ===========================================================================
// Project Accounting
// ===========================================================================

#[test]
fn builder_creates_project_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let projects = vec![
        Project::new("P-001", "ERP Implementation", Default::default()).with_budget(dec!(5000000)),
    ];
    let evms = vec![EarnedValueMetric::compute(
        "EVM-001",
        "P-001",
        date(2024, 6, 30),
        dec!(5000000),
        dec!(2500000),
        dec!(2300000),
        dec!(2600000),
    )];
    let milestones = vec![ProjectMilestone::new(
        "MS-001",
        "P-001",
        "Go-Live",
        date(2024, 12, 1),
        1,
    )];

    builder.add_project_documents(&projects, &evms, &milestones);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 3);
    assert!(hg.nodes.iter().any(|n| n.entity_type == "project"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "earned_value_metric"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "project_milestone"));

    // All -> Layer 3
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::AccountingNetwork));

    // Codes: 451, 452, 454
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 451));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 452));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 454));
}

#[test]
fn poor_project_performance_flagged_as_anomaly() {
    let mut builder = HypergraphBuilder::new(default_config());

    // SPI = 0.6, CPI = 0.7 - both below 0.8 threshold
    let evm = EarnedValueMetric::compute(
        "EVM-BAD",
        "P-002",
        date(2024, 6, 30),
        dec!(1000000),
        dec!(500000),
        dec!(300000), // EV << PV -> low SPI
        dec!(430000), // AC > EV -> low CPI
    );

    builder.add_project_documents(&[], &[evm], &[]);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 1);
    assert!(hg.nodes[0].is_anomaly);
    assert_eq!(
        hg.nodes[0].anomaly_type.as_deref(),
        Some("poor_project_performance")
    );
}

// ===========================================================================
// Intercompany
// ===========================================================================

#[test]
fn builder_creates_intercompany_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let pairs = vec![ICMatchedPair::new(
        "IC-001".into(),
        Default::default(),
        "1000".into(),
        "2000".into(),
        dec!(500000),
        "USD".into(),
        date(2024, 6, 1),
    )];
    let eliminations = vec![EliminationEntry::new(
        "ELIM-001".into(),
        EliminationType::ICBalances,
        "GROUP".into(),
        "202406".into(),
        date(2024, 6, 30),
        "USD".into(),
    )];

    builder.add_intercompany_documents(&pairs, &eliminations);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 2);
    assert!(hg.nodes.iter().any(|n| n.entity_type == "ic_matched_pair"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "elimination_entry"));

    // All -> Layer 3
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::AccountingNetwork));

    // Codes: 460, 461
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 460));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 461));
}

// ===========================================================================
// Temporal Events
// ===========================================================================

#[test]
fn builder_creates_temporal_event_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let process_events = vec![ProcessEvolutionEvent::new(
        "PE-001",
        ProcessEvolutionType::PolicyChange(PolicyChangeConfig {
            category: PolicyCategory::ApprovalThreshold,
            description: Some("Expense threshold raised".into()),
            old_value: Some(dec!(500)),
            new_value: Some(dec!(1000)),
            ..Default::default()
        }),
        date(2024, 7, 1),
    )];

    let org_events = vec![OrganizationalEvent::new(
        "OE-001",
        OrganizationalEventType::Reorganization(ReorganizationConfig {
            effective_date: date(2024, 8, 1),
            description: Some("Division restructure".into()),
            transition_months: 3,
            ..Default::default()
        }),
    )];

    let disruption_events = vec![DisruptionEvent {
        event_id: "DE-001".into(),
        disruption_type: DisruptionType::SystemOutage(OutageConfig {
            start_date: date(2024, 9, 1),
            end_date: date(2024, 9, 3),
            affected_systems: vec!["ERP".into()],
            data_loss: false,
            recovery_mode: None,
            cause: OutageCause::PlannedMaintenance,
        }),
        description: "ERP outage during upgrade".into(),
        severity: 4,
        affected_companies: vec!["1000".into()],
        labels: HashMap::new(),
    }];

    builder.add_temporal_events(&process_events, &org_events, &disruption_events);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 3);
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "process_evolution"));
    assert!(hg
        .nodes
        .iter()
        .any(|n| n.entity_type == "organizational_event"));
    assert!(hg.nodes.iter().any(|n| n.entity_type == "disruption_event"));

    // All -> Layer 2 (Process Events)
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::ProcessEvents));

    // Codes: 470, 471, 472
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 470));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 471));
    assert!(hg.nodes.iter().any(|n| n.entity_type_code == 472));
}

#[test]
fn high_severity_disruption_flagged_as_anomaly() {
    let mut builder = HypergraphBuilder::new(default_config());

    let events = vec![DisruptionEvent {
        event_id: "DE-SEV5".into(),
        disruption_type: DisruptionType::SystemOutage(OutageConfig {
            start_date: date(2024, 10, 1),
            end_date: date(2024, 10, 8),
            affected_systems: vec!["ERP".into(), "GL".into()],
            data_loss: true,
            recovery_mode: None,
            cause: OutageCause::SystemFailure,
        }),
        description: "Critical multi-system outage".into(),
        severity: 5,
        affected_companies: vec!["1000".into(), "2000".into()],
        labels: HashMap::new(),
    }];

    builder.add_temporal_events(&[], &[], &events);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 1);
    assert!(hg.nodes[0].is_anomaly);
    assert_eq!(
        hg.nodes[0].anomaly_type.as_deref(),
        Some("high_severity_disruption")
    );
}

// ===========================================================================
// AML Alerts (from BankTransaction)
// ===========================================================================

#[test]
fn builder_creates_aml_alerts_from_suspicious_transactions() {
    let mut builder = HypergraphBuilder::new(default_config());

    // Build test transactions using the existing add_bank_documents method's
    // approach: the builder takes &[BankTransaction] for add_aml_alerts.
    // Construct minimal BankTransaction structs.
    let suspicious = make_test_bank_txn(true);
    let normal = make_test_bank_txn(false);

    builder.add_aml_alerts(&[suspicious, normal]);
    let hg = builder.build();

    // Only the suspicious one becomes an AML alert
    assert_eq!(hg.nodes.len(), 1);
    assert_eq!(hg.nodes[0].entity_type, "aml_alert");
    assert_eq!(hg.nodes[0].entity_type_code, 505);
    assert_eq!(hg.nodes[0].layer, HypergraphLayer::ProcessEvents);
    assert!(hg.nodes[0].is_anomaly);
}

// ===========================================================================
// KYC Profiles (from BankingCustomer)
// ===========================================================================

#[test]
fn builder_creates_kyc_profile_nodes() {
    let mut builder = HypergraphBuilder::new(default_config());

    let good = make_test_banking_customer(false);
    let mule = make_test_banking_customer(true);

    builder.add_kyc_profiles(&[good, mule]);
    let hg = builder.build();

    assert_eq!(hg.nodes.len(), 2);
    assert!(hg.nodes.iter().all(|n| n.entity_type == "kyc_profile"));
    assert!(hg.nodes.iter().all(|n| n.entity_type_code == 504));
    assert!(hg
        .nodes
        .iter()
        .all(|n| n.layer == HypergraphLayer::ProcessEvents));

    // The mule customer should be flagged as anomaly
    let mule_node = hg.nodes.iter().find(|n| n.is_anomaly);
    assert!(mule_node.is_some());
    assert_eq!(
        mule_node.unwrap().anomaly_type.as_deref(),
        Some("mule_account")
    );
}

// ===========================================================================
// Process Family Tagging
// ===========================================================================

#[test]
fn tag_process_family_sets_correct_families() {
    let mut builder = HypergraphBuilder::new(default_config());

    // Add a mix of nodes from different domains
    let jurisdictions = vec![TaxJurisdiction::new(
        "US-FED",
        "US Federal",
        "US",
        Default::default(),
    )];
    let positions = vec![CashPosition::new(
        "CP-001",
        "ENTITY-1",
        "ACCT-001",
        "USD",
        date(2024, 6, 15),
        dec!(100000),
        dec!(50000),
        dec!(30000),
    )];
    let emissions = vec![EmissionRecord {
        id: "EM-001".into(),
        entity_id: "ENTITY-1".into(),
        scope: EmissionScope::Scope1,
        scope3_category: None,
        facility_id: None,
        period: date(2024, 6, 1),
        activity_data: None,
        activity_unit: None,
        emission_factor: None,
        co2e_tonnes: dec!(100),
        estimation_method: EstimationMethod::ActivityBased,
        source: None,
    }];

    builder.add_tax_documents(&jurisdictions, &[], &[], &[], &[], &[]);
    builder.add_treasury_documents(&positions, &[], &[], &[]);
    builder.add_esg_documents(&emissions, &[], &[], &[]);

    // Tag process families
    builder.tag_process_family();

    let hg = builder.build();

    let tax_node = hg
        .nodes
        .iter()
        .find(|n| n.entity_type == "tax_jurisdiction")
        .unwrap();
    assert_eq!(
        tax_node
            .properties
            .get("process_family")
            .and_then(|v| v.as_str()),
        Some("TAX")
    );

    let treas_node = hg
        .nodes
        .iter()
        .find(|n| n.entity_type == "cash_position")
        .unwrap();
    assert_eq!(
        treas_node
            .properties
            .get("process_family")
            .and_then(|v| v.as_str()),
        Some("TREASURY")
    );

    let esg_node = hg
        .nodes
        .iter()
        .find(|n| n.entity_type == "emission_record")
        .unwrap();
    assert_eq!(
        esg_node
            .properties
            .get("process_family")
            .and_then(|v| v.as_str()),
        Some("ESG")
    );
}

#[test]
fn tag_process_family_covers_existing_domains() {
    let mut builder = HypergraphBuilder::new(default_config());

    // Add COSO framework nodes (existing method)
    builder.add_coso_framework();
    builder.tag_process_family();

    let hg = builder.build();

    // All COSO nodes should be tagged as GOVERNANCE
    for node in &hg.nodes {
        let family = node
            .properties
            .get("process_family")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        assert_eq!(
            family, "GOVERNANCE",
            "Node {} ({}) should be GOVERNANCE",
            node.id, node.entity_type
        );
    }
}

// ===========================================================================
// Config Toggle Tests
// ===========================================================================

#[test]
fn config_toggle_disables_tax() {
    let config = HypergraphConfig {
        max_nodes: 10_000,
        include_tax: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    let jurisdictions = vec![TaxJurisdiction::new(
        "US-FED",
        "US Federal",
        "US",
        Default::default(),
    )];
    builder.add_tax_documents(&jurisdictions, &[], &[], &[], &[], &[]);
    let hg = builder.build();
    assert_eq!(hg.nodes.len(), 0);
}

#[test]
fn config_toggle_disables_esg() {
    let config = HypergraphConfig {
        max_nodes: 10_000,
        include_esg: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    let emissions = vec![EmissionRecord {
        id: "EM-001".into(),
        entity_id: "E1".into(),
        scope: EmissionScope::Scope1,
        scope3_category: None,
        facility_id: None,
        period: date(2024, 1, 1),
        activity_data: None,
        activity_unit: None,
        emission_factor: None,
        co2e_tonnes: dec!(100),
        estimation_method: EstimationMethod::ActivityBased,
        source: None,
    }];
    builder.add_esg_documents(&emissions, &[], &[], &[]);
    let hg = builder.build();
    assert_eq!(hg.nodes.len(), 0);
}

#[test]
fn config_toggle_disables_temporal_events() {
    let config = HypergraphConfig {
        max_nodes: 10_000,
        include_temporal_events: false,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);
    let disruption = DisruptionEvent {
        event_id: "DE-001".into(),
        disruption_type: DisruptionType::SystemOutage(OutageConfig {
            start_date: date(2024, 11, 1),
            end_date: date(2024, 11, 3),
            affected_systems: vec!["ERP".into()],
            data_loss: false,
            recovery_mode: None,
            cause: OutageCause::PlannedMaintenance,
        }),
        description: "Test".into(),
        severity: 3,
        affected_companies: vec![],
        labels: HashMap::new(),
    };
    builder.add_temporal_events(&[], &[], &[disruption]);
    let hg = builder.build();
    assert_eq!(hg.nodes.len(), 0);
}

// ===========================================================================
// Helper functions
// ===========================================================================

fn make_test_bank_txn(suspicious: bool) -> datasynth_banking::models::BankTransaction {
    use datasynth_banking::models::*;
    use datasynth_core::models::banking::{Direction, TransactionCategory, TransactionChannel};

    BankTransaction {
        transaction_id: uuid::Uuid::new_v4(),
        account_id: uuid::Uuid::new_v4(),
        timestamp_initiated: chrono::Utc::now(),
        timestamp_booked: chrono::Utc::now(),
        timestamp_settled: None,
        amount: if suspicious { dec!(99999) } else { dec!(500) },
        currency: "USD".into(),
        direction: Direction::default(),
        channel: TransactionChannel::default(),
        category: TransactionCategory::TransferIn,
        counterparty: CounterpartyRef {
            counterparty_type: CounterpartyType::Peer,
            counterparty_id: None,
            name: "Test Party".into(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        },
        mcc: None,
        reference: if suspicious {
            "SUSP-TXN".into()
        } else {
            "NORMAL-TXN".into()
        },
        balance_before: None,
        balance_after: None,
        original_currency: None,
        original_amount: None,
        fx_rate: None,
        location_country: None,
        location_city: None,
        device_id: None,
        ip_address: None,
        is_authorized: true,
        auth_code: None,
        status: TransactionStatus::default(),
        parent_transaction_id: None,
        is_suspicious: suspicious,
        suspicion_reason: if suspicious {
            Some(datasynth_core::models::banking::AmlTypology::Structuring)
        } else {
            None
        },
        laundering_stage: if suspicious {
            Some(datasynth_core::models::banking::LaunderingStage::Placement)
        } else {
            None
        },
        case_id: None,
        is_spoofed: false,
        spoofing_intensity: None,
        scenario_id: None,
        scenario_sequence: None,
        transaction_type: "WIRE".into(),
    }
}

fn make_test_banking_customer(is_mule: bool) -> datasynth_banking::models::BankingCustomer {
    use datasynth_banking::models::*;

    let mut cust = BankingCustomer::new_retail(
        uuid::Uuid::new_v4(),
        if is_mule { "Mule" } else { "Good" },
        if is_mule { "Account" } else { "Customer" },
        "US",
        date(2020, 1, 1),
    );
    cust.is_mule = is_mule;
    cust
}
