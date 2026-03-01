#![allow(clippy::unwrap_used)]

//! Integration tests for graph property types, entity registry, edge constraints,
//! and category helpers (Task 12 - RustGraph Round 2).

use std::collections::HashSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use datasynth_core::models::{Cardinality, GraphEntityType, GraphPropertyValue, RelationshipType};

// ============================================================================
// 1. Entity registry tests
// ============================================================================

#[test]
fn entity_type_count_is_70() {
    // 20 original + 7 tax + 8 treasury + 13 ESG + 5 project
    // + 4 S2C + 4 H2R + 4 MFG + 5 GOV = 70
    assert_eq!(
        GraphEntityType::all_types().len(),
        70,
        "Expected 70 entity types in the registry"
    );
}

#[test]
fn all_numeric_codes_are_unique() {
    let mut seen = HashSet::new();
    for &entity_type in GraphEntityType::all_types() {
        let code = entity_type.numeric_code();
        assert!(
            seen.insert(code),
            "Duplicate numeric_code {} found for {:?}",
            code,
            entity_type
        );
    }
    assert_eq!(seen.len(), 70);
}

#[test]
fn all_node_type_names_are_unique() {
    let mut seen = HashSet::new();
    for &entity_type in GraphEntityType::all_types() {
        let name = entity_type.node_type_name();
        assert!(
            seen.insert(name),
            "Duplicate node_type_name '{}' found for {:?}",
            name,
            entity_type
        );
    }
    assert_eq!(seen.len(), 70);
}

#[test]
fn all_letter_codes_are_unique() {
    let mut seen = HashSet::new();
    for &entity_type in GraphEntityType::all_types() {
        let code = entity_type.code();
        assert!(
            seen.insert(code),
            "Duplicate letter code '{}' found for {:?}",
            code,
            entity_type
        );
    }
    assert_eq!(seen.len(), 70);
}

#[test]
fn from_numeric_code_roundtrip_for_all_types() {
    for &entity_type in GraphEntityType::all_types() {
        let code = entity_type.numeric_code();
        let recovered = GraphEntityType::from_numeric_code(code);
        assert_eq!(
            recovered,
            Some(entity_type),
            "Round-trip failed for {:?} with numeric_code {}",
            entity_type,
            code
        );
    }
}

#[test]
fn from_node_type_name_roundtrip_for_all_types() {
    for &entity_type in GraphEntityType::all_types() {
        let name = entity_type.node_type_name();
        let recovered = GraphEntityType::from_node_type_name(name);
        assert_eq!(
            recovered,
            Some(entity_type),
            "Round-trip failed for {:?} with node_type_name '{}'",
            entity_type,
            name
        );
    }
}

#[test]
fn from_numeric_code_returns_none_for_unknown() {
    assert_eq!(GraphEntityType::from_numeric_code(9999), None);
    assert_eq!(GraphEntityType::from_numeric_code(0), None);
    assert_eq!(GraphEntityType::from_numeric_code(999), None);
}

#[test]
fn from_node_type_name_returns_none_for_unknown() {
    assert_eq!(GraphEntityType::from_node_type_name("nonexistent"), None);
    assert_eq!(GraphEntityType::from_node_type_name(""), None);
    assert_eq!(
        GraphEntityType::from_node_type_name("COMPANY"),
        None,
        "Lookup should be case-sensitive (snake_case)"
    );
}

#[test]
fn specific_entity_numeric_codes() {
    // Spot-check important codes from the implementation
    assert_eq!(GraphEntityType::Company.numeric_code(), 100);
    assert_eq!(GraphEntityType::Vendor.numeric_code(), 101);
    assert_eq!(GraphEntityType::Customer.numeric_code(), 103);
    assert_eq!(GraphEntityType::PurchaseOrder.numeric_code(), 200);
    assert_eq!(GraphEntityType::SalesOrder.numeric_code(), 201);
    assert_eq!(GraphEntityType::Invoice.numeric_code(), 202);
    assert_eq!(GraphEntityType::Payment.numeric_code(), 203);
    assert_eq!(GraphEntityType::BankReconciliation.numeric_code(), 210);
    // Tax
    assert_eq!(GraphEntityType::TaxJurisdiction.numeric_code(), 410);
    assert_eq!(GraphEntityType::UncertainTaxPosition.numeric_code(), 416);
    // Treasury
    assert_eq!(GraphEntityType::CashPosition.numeric_code(), 420);
    assert_eq!(GraphEntityType::DebtCovenant.numeric_code(), 427);
    // ESG
    assert_eq!(GraphEntityType::EmissionRecord.numeric_code(), 430);
    assert_eq!(GraphEntityType::ClimateScenario.numeric_code(), 442);
    // Project
    assert_eq!(GraphEntityType::ProjectCostLine.numeric_code(), 451);
    assert_eq!(GraphEntityType::ProjectMilestone.numeric_code(), 455);
    // S2C
    assert_eq!(GraphEntityType::SupplierBid.numeric_code(), 322);
    assert_eq!(GraphEntityType::SupplierQualification.numeric_code(), 325);
    // H2R
    assert_eq!(GraphEntityType::PayrollRun.numeric_code(), 330);
    assert_eq!(GraphEntityType::BenefitEnrollment.numeric_code(), 333);
    // MFG
    assert_eq!(GraphEntityType::ProductionOrder.numeric_code(), 340);
    assert_eq!(GraphEntityType::BomComponent.numeric_code(), 343);
    // GOV
    assert_eq!(GraphEntityType::CosoComponent.numeric_code(), 500);
    assert_eq!(GraphEntityType::ProfessionalJudgment.numeric_code(), 365);
}

#[test]
fn specific_entity_node_type_names() {
    assert_eq!(GraphEntityType::Company.node_type_name(), "company");
    assert_eq!(GraphEntityType::Vendor.node_type_name(), "vendor");
    assert_eq!(GraphEntityType::CostCenter.node_type_name(), "cost_center");
    assert_eq!(
        GraphEntityType::PurchaseOrder.node_type_name(),
        "purchase_order"
    );
    assert_eq!(
        GraphEntityType::TaxJurisdiction.node_type_name(),
        "tax_jurisdiction"
    );
    assert_eq!(
        GraphEntityType::UncertainTaxPosition.node_type_name(),
        "uncertain_tax_position"
    );
    assert_eq!(
        GraphEntityType::CashPosition.node_type_name(),
        "cash_position"
    );
    assert_eq!(
        GraphEntityType::HedgeRelationship.node_type_name(),
        "hedge_relationship"
    );
    assert_eq!(
        GraphEntityType::EmissionRecord.node_type_name(),
        "emission_record"
    );
    assert_eq!(
        GraphEntityType::ClimateScenario.node_type_name(),
        "climate_scenario"
    );
    assert_eq!(
        GraphEntityType::ProjectCostLine.node_type_name(),
        "project_cost_line"
    );
    assert_eq!(
        GraphEntityType::EarnedValueMetric.node_type_name(),
        "earned_value_metric"
    );
    assert_eq!(
        GraphEntityType::SupplierBid.node_type_name(),
        "supplier_bid"
    );
    assert_eq!(GraphEntityType::PayrollRun.node_type_name(), "payroll_run");
    assert_eq!(
        GraphEntityType::BomComponent.node_type_name(),
        "bom_component"
    );
    assert_eq!(
        GraphEntityType::CosoComponent.node_type_name(),
        "coso_component"
    );
    assert_eq!(
        GraphEntityType::ProfessionalJudgment.node_type_name(),
        "professional_judgment"
    );
}

#[test]
fn all_node_type_names_are_snake_case() {
    for &entity_type in GraphEntityType::all_types() {
        let name = entity_type.node_type_name();
        assert!(
            !name.is_empty(),
            "node_type_name is empty for {:?}",
            entity_type
        );
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "node_type_name '{}' for {:?} is not valid snake_case",
            name,
            entity_type
        );
        assert!(
            !name.starts_with('_') && !name.ends_with('_'),
            "node_type_name '{}' for {:?} has leading/trailing underscore",
            name,
            entity_type
        );
    }
}

// ============================================================================
// 2. Edge constraint tests
// ============================================================================

#[test]
fn all_constraints_count_is_29() {
    let constraints = RelationshipType::all_constraints();
    assert_eq!(
        constraints.len(),
        29,
        "Expected 29 domain-specific edge constraints"
    );
}

#[test]
fn all_constraint_source_types_exist_in_registry() {
    let all_types: HashSet<GraphEntityType> =
        GraphEntityType::all_types().iter().copied().collect();
    let constraints = RelationshipType::all_constraints();
    for constraint in &constraints {
        assert!(
            all_types.contains(&constraint.source_type),
            "Constraint {:?} has source_type {:?} not in all_types()",
            constraint.relationship_type,
            constraint.source_type
        );
    }
}

#[test]
fn all_constraint_target_types_exist_in_registry() {
    let all_types: HashSet<GraphEntityType> =
        GraphEntityType::all_types().iter().copied().collect();
    let constraints = RelationshipType::all_constraints();
    for constraint in &constraints {
        assert!(
            all_types.contains(&constraint.target_type),
            "Constraint {:?} has target_type {:?} not in all_types()",
            constraint.relationship_type,
            constraint.target_type
        );
    }
}

#[test]
fn all_constraint_numeric_codes_are_positive() {
    let constraints = RelationshipType::all_constraints();
    for constraint in &constraints {
        assert!(
            constraint.source_type.numeric_code() > 0,
            "source_type {:?} has zero numeric_code",
            constraint.source_type
        );
        assert!(
            constraint.target_type.numeric_code() > 0,
            "target_type {:?} has zero numeric_code",
            constraint.target_type
        );
    }
}

#[test]
fn p2p_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    // PlacedWith: PurchaseOrder -> Vendor
    let placed_with = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PlacedWith)
        .expect("P2P must have PlacedWith edge");
    assert_eq!(placed_with.source_type, GraphEntityType::PurchaseOrder);
    assert_eq!(placed_with.target_type, GraphEntityType::Vendor);
    assert_eq!(placed_with.cardinality, Cardinality::ManyToOne);

    // MatchesOrder: Invoice -> PurchaseOrder
    let matches_order = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::MatchesOrder)
        .expect("P2P must have MatchesOrder edge");
    assert_eq!(matches_order.source_type, GraphEntityType::Invoice);
    assert_eq!(matches_order.target_type, GraphEntityType::PurchaseOrder);
    assert_eq!(matches_order.cardinality, Cardinality::ManyToOne);

    // PaysInvoice: Payment -> Invoice
    let pays_invoice = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PaysInvoice)
        .expect("P2P must have PaysInvoice edge");
    assert_eq!(pays_invoice.source_type, GraphEntityType::Payment);
    assert_eq!(pays_invoice.target_type, GraphEntityType::Invoice);
    assert_eq!(pays_invoice.cardinality, Cardinality::ManyToMany);
}

#[test]
fn o2c_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    // PlacedBy: SalesOrder -> Customer
    let placed_by = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PlacedBy)
        .expect("O2C must have PlacedBy edge");
    assert_eq!(placed_by.source_type, GraphEntityType::SalesOrder);
    assert_eq!(placed_by.target_type, GraphEntityType::Customer);
    assert_eq!(placed_by.cardinality, Cardinality::ManyToOne);

    // BillsOrder: Invoice -> SalesOrder
    let bills_order = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::BillsOrder)
        .expect("O2C must have BillsOrder edge");
    assert_eq!(bills_order.source_type, GraphEntityType::Invoice);
    assert_eq!(bills_order.target_type, GraphEntityType::SalesOrder);
    assert_eq!(bills_order.cardinality, Cardinality::ManyToOne);
}

#[test]
fn s2c_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let rfx_belongs = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::RfxBelongsToProject)
        .expect("S2C must have RfxBelongsToProject edge");
    assert_eq!(rfx_belongs.source_type, GraphEntityType::RfxEvent);
    assert_eq!(rfx_belongs.target_type, GraphEntityType::SourcingProject);

    let responds_to = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::RespondsTo)
        .expect("S2C must have RespondsTo edge");
    assert_eq!(responds_to.source_type, GraphEntityType::SupplierBid);
    assert_eq!(responds_to.target_type, GraphEntityType::RfxEvent);

    let awarded_from = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::AwardedFrom)
        .expect("S2C must have AwardedFrom edge");
    assert_eq!(
        awarded_from.source_type,
        GraphEntityType::ProcurementContract
    );
    assert_eq!(awarded_from.target_type, GraphEntityType::BidEvaluation);
    assert_eq!(awarded_from.cardinality, Cardinality::OneToOne);
}

#[test]
fn h2r_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let recorded_by = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::RecordedBy)
        .expect("H2R must have RecordedBy edge");
    assert_eq!(recorded_by.source_type, GraphEntityType::TimeEntry);
    assert_eq!(recorded_by.target_type, GraphEntityType::Employee);

    let payroll_includes = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PayrollIncludes)
        .expect("H2R must have PayrollIncludes edge");
    assert_eq!(payroll_includes.source_type, GraphEntityType::PayrollRun);
    assert_eq!(payroll_includes.target_type, GraphEntityType::Employee);
    assert_eq!(payroll_includes.cardinality, Cardinality::ManyToMany);

    let submitted_by = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::SubmittedBy)
        .expect("H2R must have SubmittedBy edge");
    assert_eq!(submitted_by.source_type, GraphEntityType::ExpenseReport);
    assert_eq!(submitted_by.target_type, GraphEntityType::Employee);

    let enrolled_by = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::EnrolledBy)
        .expect("H2R must have EnrolledBy edge");
    assert_eq!(enrolled_by.source_type, GraphEntityType::BenefitEnrollment);
    assert_eq!(enrolled_by.target_type, GraphEntityType::Employee);
}

#[test]
fn mfg_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let produces = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::Produces)
        .expect("MFG must have Produces edge");
    assert_eq!(produces.source_type, GraphEntityType::ProductionOrder);
    assert_eq!(produces.target_type, GraphEntityType::Material);

    let inspects = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::Inspects)
        .expect("MFG must have Inspects edge");
    assert_eq!(inspects.source_type, GraphEntityType::QualityInspection);
    assert_eq!(inspects.target_type, GraphEntityType::ProductionOrder);

    let part_of = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PartOf)
        .expect("MFG must have PartOf edge");
    assert_eq!(part_of.source_type, GraphEntityType::BomComponent);
    assert_eq!(part_of.target_type, GraphEntityType::Material);
}

#[test]
fn tax_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let tax_line_belongs = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::TaxLineBelongsTo)
        .expect("Tax must have TaxLineBelongsTo edge");
    assert_eq!(tax_line_belongs.source_type, GraphEntityType::TaxLine);
    assert_eq!(tax_line_belongs.target_type, GraphEntityType::TaxReturn);
    assert_eq!(tax_line_belongs.cardinality, Cardinality::ManyToOne);

    let provision_applies = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::ProvisionAppliesTo)
        .expect("Tax must have ProvisionAppliesTo edge");
    assert_eq!(provision_applies.source_type, GraphEntityType::TaxProvision);
    assert_eq!(
        provision_applies.target_type,
        GraphEntityType::TaxJurisdiction
    );

    let withheld_from = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::WithheldFrom)
        .expect("Tax must have WithheldFrom edge");
    assert_eq!(
        withheld_from.source_type,
        GraphEntityType::WithholdingTaxRecord
    );
    assert_eq!(withheld_from.target_type, GraphEntityType::Vendor);
}

#[test]
fn treasury_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let sweeps_to = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::SweepsTo)
        .expect("Treasury must have SweepsTo edge");
    assert_eq!(sweeps_to.source_type, GraphEntityType::CashPoolSweep);
    assert_eq!(sweeps_to.target_type, GraphEntityType::CashPool);

    let hedges = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::HedgesInstrument)
        .expect("Treasury must have HedgesInstrument edge");
    assert_eq!(hedges.source_type, GraphEntityType::HedgeRelationship);
    assert_eq!(hedges.target_type, GraphEntityType::HedgingInstrument);

    let governs = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::GovernsInstrument)
        .expect("Treasury must have GovernsInstrument edge");
    assert_eq!(governs.source_type, GraphEntityType::DebtCovenant);
    assert_eq!(governs.target_type, GraphEntityType::DebtInstrument);
}

#[test]
fn esg_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let emission = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::EmissionReportedBy)
        .expect("ESG must have EmissionReportedBy edge");
    assert_eq!(emission.source_type, GraphEntityType::EmissionRecord);
    assert_eq!(emission.target_type, GraphEntityType::Company);

    let assesses = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::AssessesSupplier)
        .expect("ESG must have AssessesSupplier edge");
    assert_eq!(assesses.source_type, GraphEntityType::SupplierEsgAssessment);
    assert_eq!(assesses.target_type, GraphEntityType::Vendor);
}

#[test]
fn project_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let cost = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::CostChargedTo)
        .expect("Project must have CostChargedTo edge");
    assert_eq!(cost.source_type, GraphEntityType::ProjectCostLine);
    assert_eq!(cost.target_type, GraphEntityType::Project);

    let milestone = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::MilestoneOf)
        .expect("Project must have MilestoneOf edge");
    assert_eq!(milestone.source_type, GraphEntityType::ProjectMilestone);
    assert_eq!(milestone.target_type, GraphEntityType::Project);

    let modifies = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::ModifiesProject)
        .expect("Project must have ModifiesProject edge");
    assert_eq!(modifies.source_type, GraphEntityType::ChangeOrder);
    assert_eq!(modifies.target_type, GraphEntityType::Project);
}

#[test]
fn gov_process_family_has_expected_edges() {
    let constraints = RelationshipType::all_constraints();

    let principle = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::PrincipleUnder)
        .expect("GOV must have PrincipleUnder edge");
    assert_eq!(principle.source_type, GraphEntityType::CosoPrinciple);
    assert_eq!(principle.target_type, GraphEntityType::CosoComponent);
    assert_eq!(principle.cardinality, Cardinality::ManyToOne);

    let assertion = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::AssertionCovers)
        .expect("GOV must have AssertionCovers edge");
    assert_eq!(assertion.source_type, GraphEntityType::SoxAssertion);
    assert_eq!(assertion.target_type, GraphEntityType::GlAccount);
    assert_eq!(assertion.cardinality, Cardinality::ManyToMany);

    let judgment = constraints
        .iter()
        .find(|c| c.relationship_type == RelationshipType::JudgmentWithin)
        .expect("GOV must have JudgmentWithin edge");
    assert_eq!(judgment.source_type, GraphEntityType::ProfessionalJudgment);
    assert_eq!(judgment.target_type, GraphEntityType::AuditEngagement);
    assert_eq!(judgment.cardinality, Cardinality::ManyToOne);
}

#[test]
fn pre_existing_relationship_types_have_no_constraint() {
    assert!(
        RelationshipType::BuysFrom.constraint().is_none(),
        "BuysFrom should have no formal constraint"
    );
    assert!(
        RelationshipType::SellsTo.constraint().is_none(),
        "SellsTo should have no formal constraint"
    );
    assert!(
        RelationshipType::ReportsTo.constraint().is_none(),
        "ReportsTo should have no formal constraint"
    );
    assert!(
        RelationshipType::Manages.constraint().is_none(),
        "Manages should have no formal constraint"
    );
    assert!(
        RelationshipType::References.constraint().is_none(),
        "References should have no formal constraint"
    );
    assert!(
        RelationshipType::Intercompany.constraint().is_none(),
        "Intercompany should have no formal constraint"
    );
}

#[test]
fn all_constraints_have_unique_relationship_types() {
    let constraints = RelationshipType::all_constraints();
    let mut seen = HashSet::new();
    for constraint in &constraints {
        assert!(
            seen.insert(constraint.relationship_type),
            "Duplicate constraint for {:?}",
            constraint.relationship_type
        );
    }
}

// ============================================================================
// 3. Category helper tests
// ============================================================================

#[test]
fn is_tax_returns_true_for_tax_types() {
    let tax_types = [
        GraphEntityType::TaxJurisdiction,
        GraphEntityType::TaxCode,
        GraphEntityType::TaxLine,
        GraphEntityType::TaxReturn,
        GraphEntityType::TaxProvision,
        GraphEntityType::WithholdingTaxRecord,
        GraphEntityType::UncertainTaxPosition,
    ];
    for &t in &tax_types {
        assert!(t.is_tax(), "{:?} should be tax", t);
    }
    assert_eq!(tax_types.len(), 7, "Expected 7 tax entity types");
}

#[test]
fn is_tax_returns_false_for_non_tax_types() {
    let non_tax = [
        GraphEntityType::Company,
        GraphEntityType::Vendor,
        GraphEntityType::CashPosition,
        GraphEntityType::EmissionRecord,
        GraphEntityType::ProjectCostLine,
        GraphEntityType::PayrollRun,
        GraphEntityType::BomComponent,
        GraphEntityType::CosoComponent,
    ];
    for &t in &non_tax {
        assert!(!t.is_tax(), "{:?} should NOT be tax", t);
    }
}

#[test]
fn is_treasury_returns_true_for_treasury_types() {
    let treasury_types = [
        GraphEntityType::CashPosition,
        GraphEntityType::CashForecast,
        GraphEntityType::CashPool,
        GraphEntityType::CashPoolSweep,
        GraphEntityType::HedgingInstrument,
        GraphEntityType::HedgeRelationship,
        GraphEntityType::DebtInstrument,
        GraphEntityType::DebtCovenant,
    ];
    for &t in &treasury_types {
        assert!(t.is_treasury(), "{:?} should be treasury", t);
    }
    assert_eq!(treasury_types.len(), 8, "Expected 8 treasury entity types");
}

#[test]
fn is_treasury_returns_false_for_non_treasury_types() {
    let non_treasury = [
        GraphEntityType::Company,
        GraphEntityType::TaxJurisdiction,
        GraphEntityType::EmissionRecord,
        GraphEntityType::ProjectCostLine,
        GraphEntityType::PayrollRun,
    ];
    for &t in &non_treasury {
        assert!(!t.is_treasury(), "{:?} should NOT be treasury", t);
    }
}

#[test]
fn is_esg_returns_true_for_esg_types() {
    let esg_types = [
        GraphEntityType::EmissionRecord,
        GraphEntityType::EnergyConsumption,
        GraphEntityType::WaterUsage,
        GraphEntityType::WasteRecord,
        GraphEntityType::WorkforceDiversityMetric,
        GraphEntityType::PayEquityMetric,
        GraphEntityType::SafetyIncident,
        GraphEntityType::SafetyMetric,
        GraphEntityType::GovernanceMetric,
        GraphEntityType::SupplierEsgAssessment,
        GraphEntityType::MaterialityAssessment,
        GraphEntityType::EsgDisclosure,
        GraphEntityType::ClimateScenario,
    ];
    for &t in &esg_types {
        assert!(t.is_esg(), "{:?} should be ESG", t);
    }
    assert_eq!(esg_types.len(), 13, "Expected 13 ESG entity types");
}

#[test]
fn is_esg_returns_false_for_non_esg_types() {
    let non_esg = [
        GraphEntityType::Company,
        GraphEntityType::TaxCode,
        GraphEntityType::CashPool,
        GraphEntityType::ProjectMilestone,
        GraphEntityType::CosoComponent,
    ];
    for &t in &non_esg {
        assert!(!t.is_esg(), "{:?} should NOT be ESG", t);
    }
}

#[test]
fn is_project_returns_true_for_project_types() {
    let project_types = [
        GraphEntityType::Project,
        GraphEntityType::ProjectCostLine,
        GraphEntityType::ProjectRevenue,
        GraphEntityType::EarnedValueMetric,
        GraphEntityType::ChangeOrder,
        GraphEntityType::ProjectMilestone,
    ];
    for &t in &project_types {
        assert!(t.is_project(), "{:?} should be project", t);
    }
    // Note: Project is an original type but also in the project category (6 total)
    assert_eq!(
        project_types.len(),
        6,
        "Expected 6 project entity types (including base Project)"
    );
}

#[test]
fn is_project_returns_false_for_non_project_types() {
    let non_project = [
        GraphEntityType::Company,
        GraphEntityType::TaxCode,
        GraphEntityType::CashPool,
        GraphEntityType::EmissionRecord,
        GraphEntityType::PayrollRun,
    ];
    for &t in &non_project {
        assert!(!t.is_project(), "{:?} should NOT be project", t);
    }
}

#[test]
fn is_h2r_returns_true_for_h2r_types() {
    let h2r_types = [
        GraphEntityType::PayrollRun,
        GraphEntityType::TimeEntry,
        GraphEntityType::ExpenseReport,
        GraphEntityType::BenefitEnrollment,
    ];
    for &t in &h2r_types {
        assert!(t.is_h2r(), "{:?} should be H2R", t);
    }
    assert_eq!(h2r_types.len(), 4, "Expected 4 H2R entity types");
}

#[test]
fn is_h2r_returns_false_for_non_h2r_types() {
    let non_h2r = [
        GraphEntityType::Company,
        GraphEntityType::Employee,
        GraphEntityType::TaxCode,
        GraphEntityType::CashPool,
        GraphEntityType::BomComponent,
    ];
    for &t in &non_h2r {
        assert!(!t.is_h2r(), "{:?} should NOT be H2R", t);
    }
}

#[test]
fn is_mfg_returns_true_for_mfg_types() {
    let mfg_types = [
        GraphEntityType::ProductionOrder,
        GraphEntityType::QualityInspection,
        GraphEntityType::CycleCount,
        GraphEntityType::BomComponent,
        GraphEntityType::Material,
        GraphEntityType::InventoryMovement,
    ];
    for &t in &mfg_types {
        assert!(t.is_mfg(), "{:?} should be MFG", t);
    }
    // Note: Material and ProductionOrder are original types but also in MFG (6 total)
    assert_eq!(
        mfg_types.len(),
        6,
        "Expected 6 MFG entity types (including Material, ProductionOrder)"
    );
}

#[test]
fn is_mfg_returns_false_for_non_mfg_types() {
    let non_mfg = [
        GraphEntityType::Company,
        GraphEntityType::Vendor,
        GraphEntityType::TaxCode,
        GraphEntityType::CashPool,
        GraphEntityType::PayrollRun,
    ];
    for &t in &non_mfg {
        assert!(!t.is_mfg(), "{:?} should NOT be MFG", t);
    }
}

#[test]
fn is_governance_returns_true_for_governance_types() {
    let gov_types = [
        GraphEntityType::CosoComponent,
        GraphEntityType::CosoPrinciple,
        GraphEntityType::SoxAssertion,
        GraphEntityType::AuditEngagement,
        GraphEntityType::ProfessionalJudgment,
    ];
    for &t in &gov_types {
        assert!(t.is_governance(), "{:?} should be governance", t);
    }
    assert_eq!(gov_types.len(), 5, "Expected 5 GOV entity types");
}

#[test]
fn is_governance_returns_false_for_non_governance_types() {
    let non_gov = [
        GraphEntityType::Company,
        GraphEntityType::TaxCode,
        GraphEntityType::CashPool,
        GraphEntityType::EmissionRecord,
        GraphEntityType::PayrollRun,
    ];
    for &t in &non_gov {
        assert!(!t.is_governance(), "{:?} should NOT be governance", t);
    }
}

#[test]
fn category_helpers_cover_new_families_except_s2c() {
    // Every new (non-original, non-S2C) entity type should belong to at least
    // one domain-specific category or the broader master_data/transactional
    // categories.
    // S2C types (SupplierBid, BidEvaluation, ProcurementContract,
    // SupplierQualification) do not yet have a dedicated `is_s2c()` helper.
    // Original types: first 20 in all_types().
    let s2c_types: HashSet<GraphEntityType> = [
        GraphEntityType::SupplierBid,
        GraphEntityType::BidEvaluation,
        GraphEntityType::ProcurementContract,
        GraphEntityType::SupplierQualification,
    ]
    .into_iter()
    .collect();

    let all = GraphEntityType::all_types();
    for &entity_type in &all[20..] {
        if s2c_types.contains(&entity_type) {
            continue; // S2C types are tested separately
        }
        let any_category = entity_type.is_tax()
            || entity_type.is_treasury()
            || entity_type.is_esg()
            || entity_type.is_project()
            || entity_type.is_h2r()
            || entity_type.is_mfg()
            || entity_type.is_governance()
            || entity_type.is_master_data()
            || entity_type.is_transactional();
        assert!(
            any_category,
            "{:?} (code={}) does not belong to any category helper",
            entity_type,
            entity_type.numeric_code()
        );
    }
}

#[test]
fn s2c_types_are_not_in_domain_specific_categories() {
    // S2C entity types (SupplierBid, BidEvaluation, ProcurementContract,
    // SupplierQualification) exist in the registry but currently have no
    // dedicated is_s2c() helper. Verify they are not falsely claimed by other families.
    let s2c_types = [
        GraphEntityType::SupplierBid,
        GraphEntityType::BidEvaluation,
        GraphEntityType::ProcurementContract,
        GraphEntityType::SupplierQualification,
    ];
    for &t in &s2c_types {
        assert!(!t.is_tax(), "{:?} should not be tax", t);
        assert!(!t.is_treasury(), "{:?} should not be treasury", t);
        assert!(!t.is_esg(), "{:?} should not be esg", t);
        assert!(!t.is_project(), "{:?} should not be project", t);
        assert!(!t.is_h2r(), "{:?} should not be h2r", t);
        assert!(!t.is_mfg(), "{:?} should not be mfg", t);
        assert!(!t.is_governance(), "{:?} should not be governance", t);
    }
}

#[test]
fn is_master_data_and_is_transactional_work() {
    // Smoke-test the other category helpers
    assert!(GraphEntityType::Company.is_master_data());
    assert!(GraphEntityType::Vendor.is_master_data());
    assert!(GraphEntityType::GlAccount.is_master_data());
    assert!(GraphEntityType::TaxJurisdiction.is_master_data());
    assert!(GraphEntityType::TaxCode.is_master_data());
    assert!(!GraphEntityType::PurchaseOrder.is_master_data());

    assert!(GraphEntityType::PurchaseOrder.is_transactional());
    assert!(GraphEntityType::Invoice.is_transactional());
    assert!(GraphEntityType::Payment.is_transactional());
    assert!(GraphEntityType::TaxLine.is_transactional());
    assert!(!GraphEntityType::Vendor.is_transactional());
}

// ============================================================================
// 4. GraphPropertyValue tests
// ============================================================================

#[test]
fn to_string_value_for_string() {
    let v = GraphPropertyValue::String("hello world".into());
    assert_eq!(v.to_string_value(), "hello world");
}

#[test]
fn to_string_value_for_int() {
    assert_eq!(GraphPropertyValue::Int(42).to_string_value(), "42");
    assert_eq!(GraphPropertyValue::Int(-7).to_string_value(), "-7");
    assert_eq!(GraphPropertyValue::Int(0).to_string_value(), "0");
}

#[test]
fn to_string_value_for_float() {
    assert_eq!(
        GraphPropertyValue::Float(3.14).to_string_value(),
        "3.140000"
    );
    assert_eq!(GraphPropertyValue::Float(0.0).to_string_value(), "0.000000");
}

#[test]
fn to_string_value_for_decimal() {
    assert_eq!(
        GraphPropertyValue::Decimal(Decimal::new(1234, 2)).to_string_value(),
        "12.34"
    );
    assert_eq!(
        GraphPropertyValue::Decimal(Decimal::ZERO).to_string_value(),
        "0"
    );
}

#[test]
fn to_string_value_for_bool() {
    assert_eq!(GraphPropertyValue::Bool(true).to_string_value(), "true");
    assert_eq!(GraphPropertyValue::Bool(false).to_string_value(), "false");
}

#[test]
fn to_string_value_for_date() {
    let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    assert_eq!(GraphPropertyValue::Date(d).to_string_value(), "2024-01-15");
}

#[test]
fn to_string_value_for_string_list() {
    assert_eq!(
        GraphPropertyValue::StringList(vec!["a".into(), "b".into(), "c".into()]).to_string_value(),
        "a;b;c"
    );
}

#[test]
fn to_string_value_for_empty_string_list() {
    assert_eq!(GraphPropertyValue::StringList(vec![]).to_string_value(), "");
}

#[test]
fn as_str_returns_some_for_string() {
    let v = GraphPropertyValue::String("test".into());
    assert_eq!(v.as_str(), Some("test"));
}

#[test]
fn as_str_returns_none_for_non_string() {
    assert_eq!(GraphPropertyValue::Int(42).as_str(), None);
    assert_eq!(GraphPropertyValue::Bool(true).as_str(), None);
    assert_eq!(GraphPropertyValue::Float(1.0).as_str(), None);
    assert_eq!(GraphPropertyValue::Decimal(Decimal::ONE).as_str(), None);
}

#[test]
fn as_bool_returns_some_for_bool() {
    assert_eq!(GraphPropertyValue::Bool(true).as_bool(), Some(true));
    assert_eq!(GraphPropertyValue::Bool(false).as_bool(), Some(false));
}

#[test]
fn as_bool_returns_none_for_non_bool() {
    assert_eq!(GraphPropertyValue::String("true".into()).as_bool(), None);
    assert_eq!(GraphPropertyValue::Int(1).as_bool(), None);
}

#[test]
fn as_decimal_returns_some_for_decimal() {
    let d = Decimal::new(100, 0);
    assert_eq!(GraphPropertyValue::Decimal(d).as_decimal(), Some(d));
}

#[test]
fn as_decimal_returns_none_for_non_decimal() {
    assert_eq!(GraphPropertyValue::Bool(true).as_decimal(), None);
    assert_eq!(GraphPropertyValue::Float(1.0).as_decimal(), None);
    assert_eq!(GraphPropertyValue::Int(100).as_decimal(), None);
}

#[test]
fn as_int_returns_some_for_int() {
    assert_eq!(GraphPropertyValue::Int(99).as_int(), Some(99));
    assert_eq!(GraphPropertyValue::Int(-1).as_int(), Some(-1));
}

#[test]
fn as_int_returns_none_for_non_int() {
    assert_eq!(GraphPropertyValue::Float(99.0).as_int(), None);
    assert_eq!(GraphPropertyValue::String("99".into()).as_int(), None);
}

#[test]
fn as_float_returns_some_for_float() {
    assert_eq!(GraphPropertyValue::Float(1.5).as_float(), Some(1.5));
}

#[test]
fn as_float_returns_none_for_non_float() {
    assert_eq!(GraphPropertyValue::Int(1).as_float(), None);
    assert_eq!(
        GraphPropertyValue::Decimal(Decimal::new(15, 1)).as_float(),
        None
    );
}

#[test]
fn as_date_returns_some_for_date() {
    let d = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    assert_eq!(GraphPropertyValue::Date(d).as_date(), Some(d));
}

#[test]
fn as_date_returns_none_for_non_date() {
    assert_eq!(
        GraphPropertyValue::String("2024-06-01".into()).as_date(),
        None
    );
    assert_eq!(GraphPropertyValue::Int(20240601).as_date(), None);
}

#[test]
fn graph_property_value_equality() {
    let a = GraphPropertyValue::String("hello".into());
    let b = GraphPropertyValue::String("hello".into());
    let c = GraphPropertyValue::String("world".into());
    assert_eq!(a, b);
    assert_ne!(a, c);

    assert_eq!(GraphPropertyValue::Int(1), GraphPropertyValue::Int(1));
    assert_ne!(GraphPropertyValue::Int(1), GraphPropertyValue::Int(2));
    assert_ne!(GraphPropertyValue::Int(1), GraphPropertyValue::Float(1.0));
}

#[test]
fn graph_property_value_clone() {
    let original = GraphPropertyValue::StringList(vec!["x".into(), "y".into()]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ============================================================================
// 5. Relationship type inverse tests
// ============================================================================

#[test]
fn relationship_type_inverse_pairs() {
    // Verify bidirectional inverse pairs
    assert_eq!(
        RelationshipType::BuysFrom.inverse(),
        RelationshipType::SellsTo
    );
    assert_eq!(
        RelationshipType::SellsTo.inverse(),
        RelationshipType::BuysFrom
    );
    assert_eq!(
        RelationshipType::PaysTo.inverse(),
        RelationshipType::ReceivesFrom
    );
    assert_eq!(
        RelationshipType::ReceivesFrom.inverse(),
        RelationshipType::PaysTo
    );
    assert_eq!(
        RelationshipType::ReportsTo.inverse(),
        RelationshipType::Manages
    );
    assert_eq!(
        RelationshipType::Manages.inverse(),
        RelationshipType::ReportsTo
    );
    assert_eq!(
        RelationshipType::BelongsTo.inverse(),
        RelationshipType::OwnedBy
    );
    assert_eq!(
        RelationshipType::References.inverse(),
        RelationshipType::ReferencedBy
    );
    assert_eq!(
        RelationshipType::Fulfills.inverse(),
        RelationshipType::FulfilledBy
    );
    assert_eq!(
        RelationshipType::AppliesTo.inverse(),
        RelationshipType::AppliedBy
    );
}

#[test]
fn relationship_type_category_checks() {
    // Transactional
    assert!(RelationshipType::BuysFrom.is_transactional());
    assert!(RelationshipType::SellsTo.is_transactional());
    assert!(RelationshipType::PaysTo.is_transactional());
    assert!(!RelationshipType::ReportsTo.is_transactional());
    assert!(!RelationshipType::PlacedWith.is_transactional());

    // Organizational
    assert!(RelationshipType::ReportsTo.is_organizational());
    assert!(RelationshipType::Manages.is_organizational());
    assert!(RelationshipType::WorksIn.is_organizational());
    assert!(!RelationshipType::BuysFrom.is_organizational());

    // Document
    assert!(RelationshipType::References.is_document());
    assert!(RelationshipType::Fulfills.is_document());
    assert!(RelationshipType::AppliesTo.is_document());
    assert!(!RelationshipType::BuysFrom.is_document());
}

// ============================================================================
// 6. Relationship type code uniqueness
// ============================================================================

#[test]
fn relationship_type_codes_are_unique() {
    // Collect all relationship type codes via the constraint() method
    // and the known relationship types to ensure letter codes are unique
    let all_constrained: Vec<RelationshipType> = RelationshipType::all_constraints()
        .iter()
        .map(|c| c.relationship_type)
        .collect();

    let mut codes = HashSet::new();
    for rt in &all_constrained {
        let code = rt.code();
        assert!(
            codes.insert(code),
            "Duplicate relationship code '{}' for {:?}",
            code,
            rt
        );
    }
}
