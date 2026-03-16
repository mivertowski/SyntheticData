//! Integration tests for the ISA 600 component audit generator.
//!
//! Validates:
//! - Single-entity: 1 auditor, 1 instruction, 1 report
//! - Multi-entity with 2 jurisdictions: 2 auditors
//! - Scope assignment follows share thresholds
//! - Sum of component materialities ≤ group materiality
//! - Every entity covered by exactly one instruction
//! - All reports reference valid instruction IDs

use std::collections::HashSet;

use chrono::NaiveDate;
use datasynth_config::schema::{CompanyConfig, TransactionVolume};
use datasynth_core::models::audit::component_audit::ComponentScope;
use datasynth_generators::audit::component_audit_generator::ComponentAuditGenerator;
use rust_decimal::Decimal;

fn make_company(code: &str, name: &str, country: &str) -> CompanyConfig {
    CompanyConfig {
        code: code.to_string(),
        name: name.to_string(),
        currency: "USD".to_string(),
        country: country.to_string(),
        fiscal_year_variant: "K4".to_string(),
        annual_transaction_volume: TransactionVolume::TenK,
        volume_weight: 1.0,
    }
}

fn period_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
}

// =============================================================================
// Single entity
// =============================================================================

#[test]
fn test_single_entity_produces_correct_counts() {
    let companies = vec![make_company("C001", "Solo Corp", "US")];
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(1_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-001", period_end());

    assert_eq!(
        snapshot.component_auditors.len(),
        1,
        "1 auditor for 1 jurisdiction"
    );
    assert_eq!(
        snapshot.component_instructions.len(),
        1,
        "1 instruction for 1 entity"
    );
    assert_eq!(snapshot.component_reports.len(), 1, "1 report for 1 entity");
    assert!(
        snapshot.group_audit_plan.is_some(),
        "group plan must be present"
    );
}

// =============================================================================
// Multi-entity, two jurisdictions
// =============================================================================

#[test]
fn test_two_jurisdictions_produce_two_auditors() {
    let companies = vec![
        make_company("C001", "Alpha Inc", "US"),
        make_company("C002", "Beta GmbH", "DE"),
        make_company("C003", "Gamma LLC", "US"), // same jurisdiction as C001
    ];
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(5_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-002", period_end());

    assert_eq!(
        snapshot.component_auditors.len(),
        2,
        "US and DE should produce exactly 2 component auditors"
    );
    assert_eq!(snapshot.component_instructions.len(), 3);
    assert_eq!(snapshot.component_reports.len(), 3);
}

// =============================================================================
// Scope assignment thresholds
// =============================================================================

#[test]
fn test_large_entity_gets_full_scope() {
    // 2 companies: weights [2, 1], total 3
    // C001 share = 2/3 ≈ 66.7% → FullScope
    let companies = vec![
        make_company("C001", "BigCo", "US"),
        make_company("C002", "TinyCo", "US"),
    ];
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(10_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-003", period_end());

    let c001_inst = snapshot
        .component_instructions
        .iter()
        .find(|i| i.entity_code == "C001")
        .expect("C001 instruction missing");

    assert_eq!(
        c001_inst.scope,
        ComponentScope::FullScope,
        "C001 at ~66.7% should be FullScope"
    );
}

#[test]
fn test_medium_entity_gets_specific_scope() {
    // 5 companies: weights [5,4,3,2,1], total 15
    // C004 weight=2, share=2/15≈13.3% → between 5% and 15% → SpecificScope
    let companies: Vec<CompanyConfig> = (1..=5)
        .map(|i| make_company(&format!("C{i:03}"), &format!("Company {i}"), "US"))
        .collect();
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(10_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-004", period_end());

    let c004_inst = snapshot
        .component_instructions
        .iter()
        .find(|i| i.entity_code == "C004")
        .expect("C004 instruction missing");

    // C004 share ≈ 2/15 ≈ 13.3% → SpecificScope
    assert!(
        matches!(c004_inst.scope, ComponentScope::SpecificScope { .. }),
        "C004 at ~13.3% should be SpecificScope, got {:?}",
        c004_inst.scope
    );
}

#[test]
fn test_small_entity_gets_analytical_only() {
    // 10 companies: weights [10,9,...,1], total 55
    // C010 weight=1, share=1/55≈1.8% → AnalyticalOnly
    let companies: Vec<CompanyConfig> = (1..=10)
        .map(|i| make_company(&format!("C{i:03}"), &format!("Company {i}"), "US"))
        .collect();
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(10_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-005", period_end());

    let c010_inst = snapshot
        .component_instructions
        .iter()
        .find(|i| i.entity_code == "C010")
        .expect("C010 instruction missing");

    assert_eq!(
        c010_inst.scope,
        ComponentScope::AnalyticalOnly,
        "C010 at ~1.8% should be AnalyticalOnly"
    );
}

// =============================================================================
// Materiality constraints
// =============================================================================

#[test]
fn test_sum_of_component_materialities_le_group_materiality() {
    let companies: Vec<CompanyConfig> = (1..=6)
        .map(|i| make_company(&format!("C{i:03}"), &format!("Firm {i}"), "US"))
        .collect();
    let mut gen = ComponentAuditGenerator::new(99);
    let group_mat = Decimal::new(3_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-006", period_end());

    let plan = snapshot.group_audit_plan.as_ref().unwrap();
    let sum: Decimal = plan
        .component_allocations
        .iter()
        .map(|a| a.component_materiality)
        .sum();

    assert!(
        sum <= group_mat,
        "Sum of component materialities ({sum}) must not exceed group materiality ({group_mat})"
    );
}

#[test]
fn test_clearly_trivial_is_fraction_of_component_materiality() {
    let companies = vec![make_company("C001", "Alpha", "US")];
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(2_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-007", period_end());

    let plan = snapshot.group_audit_plan.as_ref().unwrap();
    let alloc = &plan.component_allocations[0];

    assert!(
        alloc.clearly_trivial < alloc.component_materiality,
        "Clearly trivial ({}) must be less than component materiality ({})",
        alloc.clearly_trivial,
        alloc.component_materiality
    );
}

// =============================================================================
// Instruction and report linkage
// =============================================================================

#[test]
fn test_all_entities_covered_by_exactly_one_instruction() {
    let companies = vec![
        make_company("C001", "Alpha", "US"),
        make_company("C002", "Beta", "DE"),
        make_company("C003", "Gamma", "FR"),
        make_company("C004", "Delta", "US"),
    ];
    let mut gen = ComponentAuditGenerator::new(7);
    let group_mat = Decimal::new(4_000_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-008", period_end());

    for company in &companies {
        let count = snapshot
            .component_instructions
            .iter()
            .filter(|i| i.entity_code == company.code)
            .count();
        assert_eq!(
            count, 1,
            "Entity {} should have exactly 1 instruction, found {count}",
            company.code
        );
    }
}

#[test]
fn test_all_reports_reference_valid_instruction_ids() {
    let companies = vec![
        make_company("C001", "Alpha", "US"),
        make_company("C002", "Beta", "GB"),
        make_company("C003", "Gamma", "AU"),
    ];
    let mut gen = ComponentAuditGenerator::new(123);
    let group_mat = Decimal::new(1_500_000, 0);

    let snapshot = gen.generate(&companies, group_mat, "ENG-009", period_end());

    let instruction_ids: HashSet<String> = snapshot
        .component_instructions
        .iter()
        .map(|i| i.id.clone())
        .collect();

    for report in &snapshot.component_reports {
        assert!(
            instruction_ids.contains(&report.instruction_id),
            "Report {} references unknown instruction ID '{}'",
            report.id,
            report.instruction_id
        );
    }
}

#[test]
fn test_reporting_deadline_is_after_period_end() {
    let companies = vec![make_company("C001", "Alpha", "US")];
    let mut gen = ComponentAuditGenerator::new(42);
    let group_mat = Decimal::new(1_000_000, 0);
    let pe = period_end();

    let snapshot = gen.generate(&companies, group_mat, "ENG-010", pe);

    for inst in &snapshot.component_instructions {
        assert!(
            inst.reporting_deadline > pe,
            "Reporting deadline {} should be after period end {}",
            inst.reporting_deadline,
            pe
        );
    }
}

// =============================================================================
// Empty input guard
// =============================================================================

#[test]
fn test_empty_companies_returns_empty_snapshot() {
    let mut gen = ComponentAuditGenerator::new(42);
    let snapshot = gen.generate(&[], Decimal::new(1_000_000, 0), "ENG-000", period_end());

    assert!(snapshot.component_auditors.is_empty());
    assert!(snapshot.component_instructions.is_empty());
    assert!(snapshot.component_reports.is_empty());
    assert!(snapshot.group_audit_plan.is_none());
}

// =============================================================================
// Determinism
// =============================================================================

#[test]
fn test_deterministic_with_same_seed() {
    let companies = vec![
        make_company("C001", "Alpha", "US"),
        make_company("C002", "Beta", "DE"),
    ];
    let group_mat = Decimal::new(2_000_000, 0);
    let pe = period_end();

    let snapshot_a = ComponentAuditGenerator::new(42).generate(&companies, group_mat, "ENG", pe);
    let snapshot_b = ComponentAuditGenerator::new(42).generate(&companies, group_mat, "ENG", pe);

    assert_eq!(
        snapshot_a.component_auditors.len(),
        snapshot_b.component_auditors.len()
    );
    assert_eq!(
        snapshot_a.component_instructions.len(),
        snapshot_b.component_instructions.len()
    );
    assert_eq!(
        snapshot_a.component_reports.len(),
        snapshot_b.component_reports.len()
    );

    // Verify first instruction IDs match
    if !snapshot_a.component_instructions.is_empty() {
        assert_eq!(
            snapshot_a.component_instructions[0].id,
            snapshot_b.component_instructions[0].id
        );
    }
}
