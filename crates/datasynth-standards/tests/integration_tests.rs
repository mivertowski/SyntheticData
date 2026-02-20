//! Integration tests for the datasynth-standards crate.
//!
//! These tests verify that:
//! 1. All standards models serialize/deserialize correctly
//! 2. Validation logic works as expected
//! 3. Framework-specific rules are enforced
//! 4. Cross-module integration works properly

use datasynth_standards::{
    audit::isa_reference::{IsaRequirement, IsaRequirementType, IsaStandard},
    audit::pcaob::{PcaobIsaMapping, PcaobStandard},
    framework::{AccountingFramework, FrameworkDifference, FrameworkSettings},
};

// ============================================================================
// Framework Tests
// ============================================================================

#[test]
fn test_accounting_framework_defaults() {
    let framework = AccountingFramework::default();
    assert_eq!(framework, AccountingFramework::UsGaap);
}

#[test]
fn test_accounting_framework_settings() {
    // US GAAP settings
    let us_gaap = FrameworkSettings::us_gaap();
    assert!(!us_gaap.use_lifo_inventory); // Most companies don't use LIFO
    assert!(!us_gaap.capitalize_development_costs);
    assert!(!us_gaap.use_ppe_revaluation);
    assert!(!us_gaap.allow_impairment_reversal);

    // IFRS settings
    let ifrs = FrameworkSettings::ifrs();
    assert!(!ifrs.use_lifo_inventory); // LIFO prohibited
    assert!(ifrs.capitalize_development_costs);
    assert!(!ifrs.use_ppe_revaluation); // Optional, most use cost
    assert!(ifrs.allow_impairment_reversal);
}

#[test]
fn test_framework_settings_validation() {
    // Valid US GAAP settings
    let us_gaap = FrameworkSettings::us_gaap();
    assert!(us_gaap.validate().is_ok());

    // Valid IFRS settings
    let ifrs = FrameworkSettings::ifrs();
    assert!(ifrs.validate().is_ok());

    // LIFO under IFRS should fail
    let mut invalid = FrameworkSettings::ifrs();
    invalid.use_lifo_inventory = true;
    assert!(invalid.validate().is_err());

    // PPE revaluation under US GAAP should fail
    let mut invalid_gaap = FrameworkSettings::us_gaap();
    invalid_gaap.use_ppe_revaluation = true;
    assert!(invalid_gaap.validate().is_err());
}

#[test]
fn test_framework_serialization() {
    let framework = AccountingFramework::UsGaap;
    let json = serde_json::to_string(&framework).unwrap();
    let deserialized: AccountingFramework = serde_json::from_str(&json).unwrap();
    assert_eq!(framework, deserialized);

    let framework = AccountingFramework::Ifrs;
    let json = serde_json::to_string(&framework).unwrap();
    let deserialized: AccountingFramework = serde_json::from_str(&json).unwrap();
    assert_eq!(framework, deserialized);

    let settings = FrameworkSettings::us_gaap();
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: FrameworkSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(settings.use_lifo_inventory, deserialized.use_lifo_inventory);
}

#[test]
fn test_framework_standard_names() {
    assert_eq!(AccountingFramework::UsGaap.revenue_standard(), "ASC 606");
    assert_eq!(AccountingFramework::Ifrs.revenue_standard(), "IFRS 15");
    assert!(AccountingFramework::FrenchGaap
        .revenue_standard()
        .contains("PCG"));

    assert_eq!(AccountingFramework::UsGaap.lease_standard(), "ASC 842");
    assert_eq!(AccountingFramework::Ifrs.lease_standard(), "IFRS 16");

    assert_eq!(AccountingFramework::UsGaap.fair_value_standard(), "ASC 820");
    assert_eq!(AccountingFramework::Ifrs.fair_value_standard(), "IFRS 13");

    assert_eq!(AccountingFramework::UsGaap.impairment_standard(), "ASC 360");
    assert_eq!(AccountingFramework::Ifrs.impairment_standard(), "IAS 36");
}

#[test]
fn test_framework_features() {
    // US GAAP features
    assert!(AccountingFramework::UsGaap.allows_lifo());
    assert!(!AccountingFramework::UsGaap.allows_ppe_revaluation());
    assert!(!AccountingFramework::UsGaap.allows_impairment_reversal());
    assert!(AccountingFramework::UsGaap.uses_brightline_lease_tests());

    // IFRS features
    assert!(!AccountingFramework::Ifrs.allows_lifo());
    assert!(AccountingFramework::Ifrs.allows_ppe_revaluation());
    assert!(AccountingFramework::Ifrs.allows_impairment_reversal());
    assert!(!AccountingFramework::Ifrs.uses_brightline_lease_tests());

    // French GAAP (PCG) features
    assert!(!AccountingFramework::FrenchGaap.allows_lifo());
    assert!(AccountingFramework::FrenchGaap.allows_impairment_reversal());
}

#[test]
fn test_french_gaap_settings() {
    let settings = FrameworkSettings::french_gaap();
    assert!(settings.validate().is_ok());
    assert_eq!(settings.framework, AccountingFramework::FrenchGaap);
}

#[test]
fn test_common_framework_differences() {
    let differences = FrameworkDifference::common_differences();
    assert!(!differences.is_empty());

    // Check inventory costing difference exists
    assert!(differences.iter().any(|d| d.area == "Inventory Costing"));

    // Check lease classification difference exists
    assert!(differences.iter().any(|d| d.area == "Lease Classification"));

    // Check impairment reversal difference exists
    assert!(differences.iter().any(|d| d.area == "Impairment Reversal"));
}

// ============================================================================
// ISA Standards Tests
// ============================================================================

#[test]
fn test_isa_standard_references() {
    let standard = IsaStandard::Isa315;
    assert_eq!(standard.number(), "315");
    assert!(standard.title().contains("Risk"));

    let standard = IsaStandard::Isa500;
    assert_eq!(standard.number(), "500");
    assert_eq!(standard.title(), "Audit Evidence");

    let standard = IsaStandard::Isa700;
    assert_eq!(standard.number(), "700");
    assert!(standard.title().contains("Opinion"));
}

#[test]
fn test_isa_requirement() {
    let requirement = IsaRequirement::new(
        IsaStandard::Isa500,
        "12".to_string(),
        IsaRequirementType::Requirement,
        "Design and perform audit procedures".to_string(),
    );

    assert_eq!(requirement.standard, IsaStandard::Isa500);
    assert_eq!(requirement.paragraph, "12");
    assert_eq!(
        requirement.requirement_type,
        IsaRequirementType::Requirement
    );
}

#[test]
fn test_isa_requirement_types() {
    // Verify all requirement types exist
    let objective = IsaRequirementType::Objective;
    let requirement = IsaRequirementType::Requirement;
    let application = IsaRequirementType::ApplicationGuidance;
    let definition = IsaRequirementType::Definition;

    // Verify they can be formatted
    assert_eq!(format!("{}", objective), "Objective");
    assert_eq!(format!("{}", requirement), "Requirement");
    assert_eq!(format!("{}", application), "Application Guidance");
    assert_eq!(format!("{}", definition), "Definition");
}

#[test]
fn test_isa_standards_serialization() {
    let standards = vec![
        IsaStandard::Isa200,
        IsaStandard::Isa315,
        IsaStandard::Isa500,
        IsaStandard::Isa700,
    ];

    for standard in standards {
        let json = serde_json::to_string(&standard).unwrap();
        let deserialized: IsaStandard = serde_json::from_str(&json).unwrap();
        assert_eq!(standard, deserialized);
    }
}

// ============================================================================
// PCAOB Tests
// ============================================================================

#[test]
fn test_pcaob_standard_references() {
    let standard = PcaobStandard::As2201;
    assert!(standard.title().contains("Internal Control"));

    let standard = PcaobStandard::As2110;
    assert!(standard.title().contains("Risk"));

    let standard = PcaobStandard::As3101;
    assert!(standard.title().contains("Report"));
}

#[test]
fn test_pcaob_isa_mapping() {
    let mut mapping = PcaobIsaMapping::new(PcaobStandard::As2110);
    mapping.isa_standards.push(IsaStandard::Isa315);

    assert_eq!(mapping.pcaob_standard, PcaobStandard::As2110);
    assert_eq!(mapping.isa_standards.len(), 1);
    assert_eq!(mapping.isa_standards[0], IsaStandard::Isa315);
}

#[test]
fn test_pcaob_standards_serialization() {
    let standards = vec![
        PcaobStandard::As2201,
        PcaobStandard::As2110,
        PcaobStandard::As3101,
        PcaobStandard::As2305,
    ];

    for standard in standards {
        let json = serde_json::to_string(&standard).unwrap();
        let deserialized: PcaobStandard = serde_json::from_str(&json).unwrap();
        assert_eq!(standard, deserialized);
    }
}

// ============================================================================
// Cross-Module Integration Tests
// ============================================================================

#[test]
fn test_dual_framework_standards() {
    // Verify dual reporting framework includes both sets of standards
    let dual = AccountingFramework::DualReporting;

    assert_eq!(dual.revenue_standard(), "ASC 606 / IFRS 15");
    assert_eq!(dual.lease_standard(), "ASC 842 / IFRS 16");
    assert_eq!(dual.fair_value_standard(), "ASC 820 / IFRS 13");
    assert_eq!(dual.impairment_standard(), "ASC 360 / IAS 36");
}

#[test]
fn test_pcaob_isa_integration() {
    // Create common mappings
    let mut mapping1 = PcaobIsaMapping::new(PcaobStandard::As2110);
    mapping1.isa_standards.push(IsaStandard::Isa315);

    let mut mapping2 = PcaobIsaMapping::new(PcaobStandard::As1105);
    mapping2.isa_standards.push(IsaStandard::Isa500);

    let mut mapping3 = PcaobIsaMapping::new(PcaobStandard::As2310);
    mapping3.isa_standards.push(IsaStandard::Isa505);

    let mut mapping4 = PcaobIsaMapping::new(PcaobStandard::As3101);
    mapping4.isa_standards.push(IsaStandard::Isa700);

    let mappings = vec![mapping1, mapping2, mapping3, mapping4];

    assert_eq!(mappings.len(), 4);

    // Verify each mapping has both standards
    for mapping in &mappings {
        assert!(!mapping.isa_standards.is_empty());
        assert!(!mapping.pcaob_standard.title().is_empty());
    }
}

#[test]
fn test_isa_standards_count() {
    // Count standards
    let all_standards = vec![
        IsaStandard::Isa200,
        IsaStandard::Isa210,
        IsaStandard::Isa220,
        IsaStandard::Isa230,
        IsaStandard::Isa240,
        IsaStandard::Isa250,
        IsaStandard::Isa260,
        IsaStandard::Isa265,
        IsaStandard::Isa300,
        IsaStandard::Isa315,
        IsaStandard::Isa320,
        IsaStandard::Isa330,
        IsaStandard::Isa402,
        IsaStandard::Isa450,
        IsaStandard::Isa500,
        IsaStandard::Isa501,
        IsaStandard::Isa505,
        IsaStandard::Isa510,
        IsaStandard::Isa520,
        IsaStandard::Isa530,
        IsaStandard::Isa540,
        IsaStandard::Isa550,
        IsaStandard::Isa560,
        IsaStandard::Isa570,
        IsaStandard::Isa580,
        IsaStandard::Isa600,
        IsaStandard::Isa610,
        IsaStandard::Isa620,
        IsaStandard::Isa700,
        IsaStandard::Isa701,
        IsaStandard::Isa705,
        IsaStandard::Isa706,
        IsaStandard::Isa710,
        IsaStandard::Isa720,
    ];

    // Verify we have 34 standards
    assert_eq!(all_standards.len(), 34);

    // Verify each standard has a number and title
    for standard in &all_standards {
        assert!(!standard.number().is_empty());
        assert!(!standard.title().is_empty());
    }
}

#[test]
fn test_framework_settings_defaults() {
    let default_settings = FrameworkSettings::default();

    // Check default thresholds
    assert_eq!(default_settings.lease_term_threshold, 0.75);
    assert_eq!(default_settings.lease_pv_threshold, 0.90);
    assert_eq!(default_settings.default_incremental_borrowing_rate, 0.05);
    assert_eq!(default_settings.variable_consideration_constraint, 0.80);
}
