//! Integration tests for datasynth-config: preset creation, validation, and YAML roundtrip.

use datasynth_config::presets::{create_preset, demo_preset, stress_test_preset};
use datasynth_config::validate_config;
use datasynth_config::TransactionVolume;
use datasynth_core::models::{CoAComplexity, IndustrySector};

/// The demo preset should pass validation without errors.
#[test]
fn test_demo_preset_validates() {
    let config = demo_preset();
    let result = validate_config(&config);
    assert!(
        result.is_ok(),
        "demo_preset() should pass validation but got: {:?}",
        result.err()
    );
}

/// Every named industry sector preset should pass validation.
#[test]
fn test_all_industry_presets_validate() {
    let industries = [
        IndustrySector::Manufacturing,
        IndustrySector::Retail,
        IndustrySector::FinancialServices,
        IndustrySector::Healthcare,
        IndustrySector::Technology,
    ];

    for industry in industries {
        let config = create_preset(
            industry,
            2,
            6,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        let result = validate_config(&config);
        assert!(
            result.is_ok(),
            "Preset for {:?} should pass validation but got: {:?}",
            industry,
            result.err()
        );
    }
}

/// The stress-test preset should pass validation without errors.
#[test]
fn test_stress_test_preset_validates() {
    let config = stress_test_preset();
    let result = validate_config(&config);
    assert!(
        result.is_ok(),
        "stress_test_preset() should pass validation but got: {:?}",
        result.err()
    );
}

/// Clearing the companies list from a valid preset must cause validation to fail.
#[test]
fn test_empty_companies_fails() {
    let mut config = demo_preset();
    config.companies.clear();

    let result = validate_config(&config);
    assert!(
        result.is_err(),
        "Config with no companies should fail validation"
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("company") || err_msg.to_lowercase().contains("companies"),
        "Error message should mention companies: {}",
        err_msg
    );
}

/// Setting period_months to 0 must cause validation to fail.
#[test]
fn test_invalid_period_months_fails() {
    let mut config = demo_preset();
    config.global.period_months = 0;

    let result = validate_config(&config);
    assert!(
        result.is_err(),
        "Config with period_months=0 should fail validation"
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.to_lowercase().contains("period"),
        "Error message should mention period: {}",
        err_msg
    );
}

/// Serializing a preset config to YAML and deserializing it back should preserve key fields.
#[test]
fn test_config_roundtrip_yaml() {
    let original = demo_preset();

    let yaml_str =
        serde_yaml::to_string(&original).expect("Failed to serialize GeneratorConfig to YAML");

    let deserialized: datasynth_config::GeneratorConfig =
        serde_yaml::from_str(&yaml_str).expect("Failed to deserialize GeneratorConfig from YAML");

    // Verify key fields survived the roundtrip
    assert_eq!(deserialized.global.industry, original.global.industry);
    assert_eq!(
        deserialized.global.period_months,
        original.global.period_months
    );
    assert_eq!(deserialized.global.start_date, original.global.start_date);
    assert_eq!(
        deserialized.global.group_currency,
        original.global.group_currency
    );
    assert_eq!(deserialized.companies.len(), original.companies.len());
    assert_eq!(deserialized.companies[0].code, original.companies[0].code);
    assert_eq!(deserialized.companies[0].name, original.companies[0].name);
    assert_eq!(
        deserialized.companies[0].currency,
        original.companies[0].currency
    );
    assert_eq!(
        deserialized.companies[0].country,
        original.companies[0].country
    );
    assert_eq!(
        deserialized.chart_of_accounts.complexity,
        original.chart_of_accounts.complexity
    );

    // The deserialized config should also pass validation
    let result = validate_config(&deserialized);
    assert!(
        result.is_ok(),
        "Deserialized config should still pass validation but got: {:?}",
        result.err()
    );
}

/// The demo preset should have sensible defaults for basic fields.
#[test]
fn test_preset_has_reasonable_defaults() {
    let config = demo_preset();

    // At least 1 company
    assert!(
        !config.companies.is_empty(),
        "Demo preset should have at least one company"
    );

    // Positive period_months
    assert!(
        config.global.period_months > 0,
        "Demo preset should have positive period_months, got {}",
        config.global.period_months
    );

    // Non-default complexity (Small, Medium, or Large -- any is acceptable, but must be set)
    // We verify by checking that the CoA complexity field is one of the known variants.
    let complexity = &config.chart_of_accounts.complexity;
    assert!(
        matches!(
            complexity,
            CoAComplexity::Small | CoAComplexity::Medium | CoAComplexity::Large
        ),
        "Demo preset should have a recognized CoA complexity, got {:?}",
        complexity
    );

    // Industry should be set
    let industry = &config.global.industry;
    assert!(
        matches!(
            industry,
            IndustrySector::Manufacturing
                | IndustrySector::Retail
                | IndustrySector::FinancialServices
                | IndustrySector::Healthcare
                | IndustrySector::Technology
                | IndustrySector::ProfessionalServices
                | IndustrySector::Energy
                | IndustrySector::Transportation
                | IndustrySector::RealEstate
                | IndustrySector::Telecommunications
        ),
        "Demo preset should have a recognized industry sector, got {:?}",
        industry
    );

    // start_date should be a valid date string
    assert!(
        !config.global.start_date.is_empty(),
        "Demo preset should have a non-empty start_date"
    );

    // group_currency should be a non-empty string
    assert!(
        !config.global.group_currency.is_empty(),
        "Demo preset should have a non-empty group_currency"
    );

    // Each company should have non-empty code, name, currency, country
    for company in &config.companies {
        assert!(!company.code.is_empty(), "Company code should not be empty");
        assert!(!company.name.is_empty(), "Company name should not be empty");
        assert!(
            !company.currency.is_empty(),
            "Company currency should not be empty"
        );
        assert!(
            company.country.len() == 2,
            "Company country should be 2-char ISO code, got '{}'",
            company.country
        );
    }
}
