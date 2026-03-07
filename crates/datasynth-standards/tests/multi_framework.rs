//! Multi-framework integration tests for the datasynth-standards crate.
//!
//! Validates framework-specific differences and cross-module integration
//! for accounting, audit, and regulatory standards.

use datasynth_standards::{
    audit::isa_reference::{IsaRequirement, IsaRequirementType, IsaStandard},
    audit::pcaob::PcaobStandard,
    framework::{AccountingFramework, FrameworkDifference, FrameworkSettings},
};

// ============================================================================
// Multi-Framework Accounting Tests
// ============================================================================

#[test]
fn test_us_gaap_prohibits_revaluation() {
    let mut settings = FrameworkSettings::us_gaap();
    assert!(settings.validate().is_ok());

    // PPE revaluation is prohibited under US GAAP
    settings.use_ppe_revaluation = true;
    assert!(
        settings.validate().is_err(),
        "US GAAP should reject PPE revaluation"
    );
}

#[test]
fn test_ifrs_prohibits_lifo() {
    let mut settings = FrameworkSettings::ifrs();
    assert!(settings.validate().is_ok());

    // LIFO is prohibited under IFRS
    settings.use_lifo_inventory = true;
    assert!(
        settings.validate().is_err(),
        "IFRS should reject LIFO inventory"
    );
}

#[test]
fn test_ifrs_allows_impairment_reversal() {
    let ifrs = FrameworkSettings::ifrs();
    assert!(
        ifrs.allow_impairment_reversal,
        "IFRS should allow impairment reversal"
    );

    let us_gaap = FrameworkSettings::us_gaap();
    assert!(
        !us_gaap.allow_impairment_reversal,
        "US GAAP should not allow impairment reversal"
    );
}

#[test]
fn test_ifrs_capitalizes_development_costs() {
    let ifrs = FrameworkSettings::ifrs();
    assert!(
        ifrs.capitalize_development_costs,
        "IFRS should capitalize development costs"
    );

    let us_gaap = FrameworkSettings::us_gaap();
    assert!(
        !us_gaap.capitalize_development_costs,
        "US GAAP should expense development costs"
    );
}

#[test]
fn test_framework_differences_between_gaap_and_ifrs() {
    let us_gaap = FrameworkSettings::us_gaap();
    let ifrs = FrameworkSettings::ifrs();

    // At minimum, development costs and impairment reversal differ
    let key_diffs = vec![
        (
            "capitalize_development_costs",
            us_gaap.capitalize_development_costs,
            ifrs.capitalize_development_costs,
        ),
        (
            "allow_impairment_reversal",
            us_gaap.allow_impairment_reversal,
            ifrs.allow_impairment_reversal,
        ),
    ];

    for (name, gaap_val, ifrs_val) in key_diffs {
        assert_ne!(gaap_val, ifrs_val, "Expected difference for {name}");
    }
}

// ============================================================================
// Framework Serialization Roundtrip
// ============================================================================

#[test]
fn test_framework_settings_roundtrip() {
    let original = FrameworkSettings::ifrs();
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: FrameworkSettings = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        original.capitalize_development_costs,
        restored.capitalize_development_costs
    );
    assert_eq!(
        original.allow_impairment_reversal,
        restored.allow_impairment_reversal
    );
    assert_eq!(original.use_lifo_inventory, restored.use_lifo_inventory);
    assert_eq!(original.use_ppe_revaluation, restored.use_ppe_revaluation);
}

#[test]
fn test_accounting_framework_enum_roundtrip() {
    for framework in [
        AccountingFramework::UsGaap,
        AccountingFramework::Ifrs,
        AccountingFramework::DualReporting,
    ] {
        let json = serde_json::to_string(&framework).expect("serialize");
        let restored: AccountingFramework = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(framework, restored);
    }
}

// ============================================================================
// ISA / PCAOB Tests
// ============================================================================

#[test]
fn test_isa_standards_cover_key_areas() {
    let standards = vec![
        IsaStandard::Isa200, // Overall objectives
        IsaStandard::Isa315, // Risk assessment
        IsaStandard::Isa330, // Responses to assessed risks
        IsaStandard::Isa500, // Audit evidence
        IsaStandard::Isa505, // External confirmations
        IsaStandard::Isa520, // Analytical procedures
        IsaStandard::Isa700, // Forming an opinion
    ];

    for standard in &standards {
        let number = standard.number();
        assert!(!number.is_empty(), "ISA should have a number");
        let title = standard.title();
        assert!(!title.is_empty(), "ISA {number} should have a title");
    }
}

#[test]
fn test_pcaob_standards_exist() {
    let standards = vec![
        PcaobStandard::As1101,
        PcaobStandard::As2201,
        PcaobStandard::As3101,
    ];

    for standard in &standards {
        let number = standard.number();
        assert!(!number.is_empty(), "PCAOB standard should have a number");
        let title = standard.title();
        assert!(!title.is_empty(), "PCAOB standard should have a title");
    }
}

#[test]
fn test_isa_requirement_serialization() {
    let req = IsaRequirement {
        standard: IsaStandard::Isa315,
        paragraph: "12".to_string(),
        requirement_type: IsaRequirementType::Requirement,
        description: "Understand the entity and its environment".to_string(),
        is_mandatory: true,
    };

    let json = serde_json::to_string(&req).expect("serialize");
    let restored: IsaRequirement = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.standard, IsaStandard::Isa315);
    assert_eq!(restored.requirement_type, IsaRequirementType::Requirement);
    assert!(restored.is_mandatory);
}

// ============================================================================
// Framework Difference Tracking
// ============================================================================

#[test]
fn test_framework_difference_creation() {
    let diff = FrameworkDifference {
        area: "Inventory Valuation".to_string(),
        us_gaap_treatment: "LIFO permitted".to_string(),
        ifrs_treatment: "LIFO prohibited".to_string(),
        typically_material: true,
        us_gaap_reference: "ASC 330".to_string(),
        ifrs_reference: "IAS 2".to_string(),
    };

    let json = serde_json::to_string(&diff).expect("serialize");
    assert!(json.contains("LIFO"));

    let restored: FrameworkDifference = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.area, "Inventory Valuation");
    assert!(restored.typically_material);
}
