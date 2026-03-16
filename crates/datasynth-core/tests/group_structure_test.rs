//! Integration tests for the GroupStructure ownership model.

use datasynth_core::models::intercompany::{
    AssociateRelationship, GroupConsolidationMethod, GroupStructure, SubsidiaryRelationship,
};
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_group_structure_roundtrip_json() {
    let mut group = GroupStructure::new("PARENT".to_string());

    group.add_subsidiary(SubsidiaryRelationship::new_full(
        "SUB_A".to_string(),
        "USD".to_string(),
    ));
    group.add_subsidiary(SubsidiaryRelationship::new_with_ownership(
        "SUB_B".to_string(),
        dec!(75),
        "EUR".to_string(),
        None,
    ));
    group.add_associate(AssociateRelationship::new("ASSOC_C".to_string(), dec!(30)));

    let json = serde_json::to_string(&group).expect("serialization failed");
    let deserialized: GroupStructure = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized.parent_entity, "PARENT");
    assert_eq!(deserialized.subsidiaries.len(), 2);
    assert_eq!(deserialized.associates.len(), 1);

    let sub_a = &deserialized.subsidiaries[0];
    assert_eq!(sub_a.entity_code, "SUB_A");
    assert_eq!(sub_a.ownership_percentage, dec!(100));
    assert_eq!(sub_a.nci_percentage, dec!(0));
    assert_eq!(
        sub_a.consolidation_method,
        GroupConsolidationMethod::FullConsolidation
    );

    let sub_b = &deserialized.subsidiaries[1];
    assert_eq!(sub_b.entity_code, "SUB_B");
    assert_eq!(sub_b.ownership_percentage, dec!(75));
    assert_eq!(sub_b.nci_percentage, dec!(25));
    assert_eq!(
        sub_b.consolidation_method,
        GroupConsolidationMethod::FullConsolidation
    );

    let assoc = &deserialized.associates[0];
    assert_eq!(assoc.entity_code, "ASSOC_C");
    assert_eq!(assoc.ownership_percentage, dec!(30));
}

#[test]
fn test_subsidiary_relationship_roundtrip_json() {
    let sub = SubsidiaryRelationship::new_with_ownership(
        "ENTITY_X".to_string(),
        dec!(60),
        "GBP".to_string(),
        Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()),
    );

    let json = serde_json::to_string(&sub).expect("serialization failed");
    let deserialized: SubsidiaryRelationship =
        serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(deserialized.entity_code, "ENTITY_X");
    assert_eq!(deserialized.ownership_percentage, dec!(60));
    assert_eq!(deserialized.nci_percentage, dec!(40));
    assert_eq!(deserialized.functional_currency, "GBP");
    assert_eq!(
        deserialized.acquisition_date,
        Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
    );
    assert_eq!(
        deserialized.consolidation_method,
        GroupConsolidationMethod::FullConsolidation
    );
}

// ---------------------------------------------------------------------------
// ConsolidationMethod derivation from ownership percentage
// ---------------------------------------------------------------------------

#[test]
fn test_consolidation_method_full_consolidation() {
    // > 50 % → FullConsolidation
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(100)),
        GroupConsolidationMethod::FullConsolidation,
        "100% ownership should give FullConsolidation"
    );
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(51)),
        GroupConsolidationMethod::FullConsolidation,
        "51% ownership should give FullConsolidation"
    );
    // boundary: exactly 50 % is NOT full consolidation under IFRS 10
    assert_ne!(
        GroupConsolidationMethod::from_ownership(dec!(50)),
        GroupConsolidationMethod::FullConsolidation,
        "50% ownership should NOT give FullConsolidation"
    );
}

#[test]
fn test_consolidation_method_equity_method() {
    // 20–50 % → EquityMethod
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(50)),
        GroupConsolidationMethod::EquityMethod,
        "50% ownership should give EquityMethod"
    );
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(35)),
        GroupConsolidationMethod::EquityMethod,
        "35% ownership should give EquityMethod"
    );
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(20)),
        GroupConsolidationMethod::EquityMethod,
        "20% ownership should give EquityMethod"
    );
    assert_ne!(
        GroupConsolidationMethod::from_ownership(dec!(19)),
        GroupConsolidationMethod::EquityMethod,
        "19% ownership should NOT give EquityMethod"
    );
}

#[test]
fn test_consolidation_method_fair_value() {
    // < 20 % → FairValue
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(19)),
        GroupConsolidationMethod::FairValue,
        "19% ownership should give FairValue"
    );
    assert_eq!(
        GroupConsolidationMethod::from_ownership(dec!(0)),
        GroupConsolidationMethod::FairValue,
        "0% ownership should give FairValue"
    );
}

// ---------------------------------------------------------------------------
// GroupStructure helpers
// ---------------------------------------------------------------------------

#[test]
fn test_group_structure_entity_count() {
    let mut group = GroupStructure::new("P".to_string());
    assert_eq!(group.entity_count(), 1, "only parent");

    group.add_subsidiary(SubsidiaryRelationship::new_full(
        "S1".to_string(),
        "USD".to_string(),
    ));
    group.add_subsidiary(SubsidiaryRelationship::new_full(
        "S2".to_string(),
        "EUR".to_string(),
    ));
    group.add_associate(AssociateRelationship::new("A1".to_string(), dec!(25)));

    assert_eq!(group.entity_count(), 4, "parent + 2 subs + 1 associate");
}

#[test]
fn test_nci_percentage_derived_correctly() {
    let sub = SubsidiaryRelationship::new_with_ownership(
        "E".to_string(),
        dec!(80),
        "USD".to_string(),
        None,
    );
    assert_eq!(
        sub.nci_percentage,
        dec!(20),
        "NCI should be 100 - ownership_percentage"
    );
}

#[test]
fn test_full_subsidiary_has_zero_nci() {
    let sub = SubsidiaryRelationship::new_full("E".to_string(), "USD".to_string());
    assert_eq!(sub.nci_percentage, dec!(0));
    assert_eq!(sub.ownership_percentage, dec!(100));
}

#[test]
fn test_associate_default_equity_pickup_is_zero() {
    let assoc = AssociateRelationship::new("A".to_string(), dec!(30));
    assert_eq!(assoc.equity_pickup, dec!(0));
}

// ---------------------------------------------------------------------------
// JSON field name serialisation (snake_case enum variants)
// ---------------------------------------------------------------------------

#[test]
fn test_consolidation_method_serializes_snake_case() {
    let fc = GroupConsolidationMethod::FullConsolidation;
    let em = GroupConsolidationMethod::EquityMethod;
    let fv = GroupConsolidationMethod::FairValue;

    assert_eq!(
        serde_json::to_string(&fc).unwrap(),
        "\"full_consolidation\""
    );
    assert_eq!(serde_json::to_string(&em).unwrap(), "\"equity_method\"");
    assert_eq!(serde_json::to_string(&fv).unwrap(), "\"fair_value\"");
}
