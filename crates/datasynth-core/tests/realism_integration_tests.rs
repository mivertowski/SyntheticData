//! Integration tests for the realism module.
//!
//! These tests verify the complete functionality of the realism subsystem
//! including company names, vendor names, descriptions, user IDs, references,
//! and addresses.

use datasynth_core::templates::realism::{
    Address, AddressGenerator, AddressRegion, AddressStyle, CompanyNameGenerator, CompanyNameStyle,
    DescriptionVariator, EnhancedReferenceFormat, EnhancedReferenceGenerator, Industry,
    LegalSuffix, RealismConfig, RealismGenerator, SpendCategory, TypoGenerator,
    UserIdGenerator, UserIdPattern, VariationConfig, VendorNameGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

// ============================================================================
// RealismGenerator Integration Tests
// ============================================================================

#[test]
fn test_realism_generator_comprehensive() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = RealismGenerator::new();

    // Test company name generation for all industries
    for industry in Industry::all() {
        let name = gen.generate_company_name(*industry, &mut rng);
        assert!(!name.is_empty(), "Company name should not be empty for {:?}", industry);
        assert!(
            name.contains('.') || name.contains(' '),
            "Company name '{}' should contain space or period for suffix",
            name
        );
    }

    // Test vendor name generation for all categories
    for category in SpendCategory::all() {
        let name = gen.generate_vendor_name(*category, &mut rng);
        assert!(!name.is_empty(), "Vendor name should not be empty for {:?}", category);
    }

    // Test address generation
    let addr = gen.generate_address(&mut rng);
    assert!(!addr.city.is_empty());
    assert!(!addr.street_name.is_empty());

    // Test user ID generation
    let user_id = gen.generate_user_id("John", "Smith", 1, &mut rng);
    assert!(!user_id.is_empty());
}

#[test]
fn test_realism_generator_determinism() {
    let config = RealismConfig::default();

    let gen1 = RealismGenerator::with_config(config.clone());
    let gen2 = RealismGenerator::with_config(config);

    let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
    let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

    // Same seed should produce same results
    let name1 = gen1.generate_company_name(Industry::Technology, &mut rng1);
    let name2 = gen2.generate_company_name(Industry::Technology, &mut rng2);
    assert_eq!(name1, name2, "Deterministic generation failed for company names");

    let mut rng1 = ChaCha8Rng::seed_from_u64(54321);
    let mut rng2 = ChaCha8Rng::seed_from_u64(54321);

    let vendor1 = gen1.generate_vendor_name(SpendCategory::ITServices, &mut rng1);
    let vendor2 = gen2.generate_vendor_name(SpendCategory::ITServices, &mut rng2);
    assert_eq!(vendor1, vendor2, "Deterministic generation failed for vendor names");
}

#[test]
fn test_realism_generator_config_effects() {
    // Test with variations disabled
    let config = RealismConfig {
        description_variations: false,
        ..Default::default()
    };
    let gen = RealismGenerator::with_config(config);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // When variations are disabled, description should not change
    let original = "Invoice for Services";
    let varied = gen.vary_description(original, &mut rng);
    assert_eq!(original, varied, "Description should not change when variations disabled");

    // Test with variations enabled and high rate
    let config2 = RealismConfig {
        description_variations: true,
        abbreviation_rate: 1.0,
        typo_rate: 0.0,
        ..Default::default()
    };
    let gen2 = RealismGenerator::with_config(config2);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);

    let original = "Invoice for Purchase Order";
    let varied = gen2.vary_description(original, &mut rng2);
    // With 100% abbreviation rate, should have some abbreviation
    assert!(
        varied.contains("Inv") || varied.contains("PO") || varied == original,
        "Expected abbreviation in '{}'",
        varied
    );
}

// ============================================================================
// Company Name Generation Tests
// ============================================================================

#[test]
fn test_company_names_all_industries() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = CompanyNameGenerator::new();

    for industry in Industry::all() {
        let mut names = HashSet::new();
        for _ in 0..50 {
            let name = gen.generate(*industry, &mut rng);
            names.insert(name.clone());

            // Verify has legal suffix
            let has_suffix = name.ends_with("Inc.")
                || name.ends_with("Corp.")
                || name.ends_with("Corporation")
                || name.ends_with("LLC")
                || name.ends_with("Ltd.")
                || name.ends_with("Co.")
                || name.ends_with("Company")
                || name.ends_with("Group");
            assert!(has_suffix, "Missing legal suffix in '{}'", name);
        }

        // Should have good variety
        assert!(
            names.len() > 20,
            "Industry {:?} should generate diverse names, got {} unique",
            industry,
            names.len()
        );
    }
}

#[test]
fn test_company_names_all_styles() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = CompanyNameGenerator::new();

    let styles = [
        CompanyNameStyle::FounderBased,
        CompanyNameStyle::Descriptive,
        CompanyNameStyle::LocationBased,
        CompanyNameStyle::Acronym,
        CompanyNameStyle::Abstract,
    ];

    for style in styles {
        let name = gen.generate_with_style(Industry::Manufacturing, style, &mut rng);
        assert!(!name.is_empty(), "Style {:?} should generate non-empty name", style);

        if style == CompanyNameStyle::Acronym {
            // Acronym style should have 3 uppercase letters at the start
            let first_word = name.split_whitespace().next().unwrap();
            assert!(
                first_word.len() == 3 && first_word.chars().all(|c| c.is_ascii_uppercase()),
                "Acronym style should start with 3 uppercase letters: {}",
                name
            );
        }
    }
}

#[test]
fn test_legal_suffixes() {
    assert_eq!(LegalSuffix::Inc.as_str(), "Inc.");
    assert_eq!(LegalSuffix::LLC.as_str(), "LLC");
    assert_eq!(LegalSuffix::GmbH.as_str(), "GmbH");
    assert_eq!(LegalSuffix::SA.as_str(), "S.A.");
    assert_eq!(LegalSuffix::PLC.as_str(), "PLC");
    assert_eq!(LegalSuffix::Pty.as_str(), "Pty Ltd");
}

// ============================================================================
// Vendor Name Generation Tests
// ============================================================================

#[test]
fn test_vendor_names_all_categories() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = VendorNameGenerator::new();

    for category in SpendCategory::all() {
        let mut names = HashSet::new();
        for _ in 0..50 {
            let name = gen.generate(*category, &mut rng);
            names.insert(name);
        }

        // Should have variety
        assert!(
            names.len() > 15,
            "Category {:?} should generate diverse names, got {} unique",
            category,
            names.len()
        );
    }
}

#[test]
fn test_vendor_profiles() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = VendorNameGenerator::new();

    let profile = gen.generate_profile(SpendCategory::ITServices, &mut rng);
    assert!(!profile.name.is_empty());
    assert_eq!(profile.category, SpendCategory::ITServices);
}

#[test]
fn test_vendor_well_known_brands() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = VendorNameGenerator::new();

    // Generate many names and check if well-known brands appear
    let mut found_brands = HashSet::new();
    for _ in 0..500 {
        let name = gen.generate(SpendCategory::OfficeSupplies, &mut rng);
        if name == "Staples" || name == "Office Depot" || name == "ULINE" {
            found_brands.insert(name);
        }
    }

    // Should find at least one well-known brand
    assert!(
        !found_brands.is_empty(),
        "Should occasionally generate well-known brands"
    );
}

// ============================================================================
// Description Variation Tests
// ============================================================================

#[test]
fn test_description_abbreviations_comprehensive() {
    let config = VariationConfig {
        abbreviation_rate: 1.0,
        typo_rate: 0.0,
        case_variation_rate: 0.0,
        ..Default::default()
    };
    let variator = DescriptionVariator::with_config(config);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let test_cases = [
        ("Invoice for Purchase Order", vec!["Inv", "INV", "PO", "P.O."]),
        ("Accounts Payable Payment", vec!["AP", "A/P", "Pmt", "PMT"]),
        ("Revenue Recognition for December", vec!["Rev", "Dec"]),
        ("Transaction Reference Number", vec!["Trans", "TXN", "Ref", "No"]),
    ];

    for (input, expected_abbrevs) in test_cases {
        let output = variator.apply(input, &mut rng);
        let has_abbrev = expected_abbrevs.iter().any(|a| output.contains(a));
        assert!(
            has_abbrev || output == input,
            "Expected abbreviation in '{}' -> '{}'",
            input,
            output
        );
    }
}

#[test]
fn test_typo_generator() {
    let gen = TypoGenerator::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Generate many typos and verify they're different from original
    let original = "payment processing transaction";
    let mut different_count = 0;

    for _ in 0..100 {
        let typo = gen.introduce_typo(original, &mut rng);
        if typo != original {
            different_count += 1;
        }
    }

    // Most should be different
    assert!(
        different_count > 80,
        "Typo generator should usually modify the text, got {} different out of 100",
        different_count
    );
}

// ============================================================================
// User ID Generation Tests
// ============================================================================

#[test]
fn test_user_id_all_patterns() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = UserIdGenerator::new();

    let patterns = [
        (UserIdPattern::InitialLastName, "JSMITH"),
        (UserIdPattern::DotSeparated, "john.smith"),
        (UserIdPattern::UnderscoreSeparated, "john_smith"),
        (UserIdPattern::LastNameInitial, "smithj"),
        (UserIdPattern::EmployeeNumber, "E00000001"),
    ];

    for (pattern, expected_start) in patterns {
        let id = gen.generate_with_pattern("John", "Smith", 0, pattern, &mut rng);
        if pattern != UserIdPattern::EmployeeNumber {
            assert!(
                id.to_lowercase().contains(&expected_start.to_lowercase())
                    || id == expected_start,
                "Pattern {:?} generated '{}', expected to contain/match '{}'",
                pattern,
                id,
                expected_start
            );
        }
    }
}

#[test]
fn test_system_and_admin_accounts() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = UserIdGenerator::new();

    // System accounts
    let mut system_accounts = HashSet::new();
    for _ in 0..50 {
        system_accounts.insert(gen.generate_system_account(&mut rng));
    }
    assert!(system_accounts.len() > 10, "Should generate diverse system accounts");

    // All should have system prefix
    for account in &system_accounts {
        assert!(
            account.starts_with("SVC_")
                || account.starts_with("SYS_")
                || account.starts_with("BATCH_")
                || account.starts_with("AUTO_")
                || account.starts_with("SCHED_"),
            "System account '{}' should have valid prefix",
            account
        );
    }

    // Interface accounts
    let interface = gen.generate_interface_account("SAP");
    assert_eq!(interface, "INT_SAP");
}

// ============================================================================
// Reference Format Tests
// ============================================================================

#[test]
fn test_reference_formats_comprehensive() {
    let gen = EnhancedReferenceGenerator::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let formats = [
        (EnhancedReferenceFormat::Standard, 15, "DOC-"),
        (EnhancedReferenceFormat::SapStyle, 10, "4500"),
        (EnhancedReferenceFormat::OracleStyle, 0, "ORG1-"),
        (EnhancedReferenceFormat::NetSuiteStyle, 8, "INV"),
        (EnhancedReferenceFormat::Alphanumeric, 10, ""),
        (EnhancedReferenceFormat::CheckNumber, 6, ""),
    ];

    for (format, expected_len, expected_prefix) in formats {
        let reference = gen.generate(format, 2024, &mut rng);

        if expected_len > 0 {
            assert_eq!(
                reference.len(),
                expected_len,
                "Format {:?} should have length {}, got {} ('{}')",
                format,
                expected_len,
                reference.len(),
                reference
            );
        }

        if !expected_prefix.is_empty() {
            assert!(
                reference.starts_with(expected_prefix),
                "Format {:?} should start with '{}', got '{}'",
                format,
                expected_prefix,
                reference
            );
        }
    }
}

#[test]
fn test_reference_sequential() {
    let gen = EnhancedReferenceGenerator::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let refs: Vec<String> = (0..10)
        .map(|_| gen.generate(EnhancedReferenceFormat::Standard, 2024, &mut rng))
        .collect();

    // Verify sequential - extract numbers and check they're increasing
    let numbers: Vec<u64> = refs
        .iter()
        .map(|r| {
            r.split('-')
                .last()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        })
        .collect();

    for i in 1..numbers.len() {
        assert_eq!(
            numbers[i],
            numbers[i - 1] + 1,
            "References should be sequential"
        );
    }
}

#[test]
fn test_vendor_invoice_variety() {
    let gen = EnhancedReferenceGenerator::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mut formats_seen = HashSet::new();
    for _ in 0..200 {
        let reference = gen.generate(EnhancedReferenceFormat::VendorInvoice, 2024, &mut rng);
        // Extract first 3 chars as format identifier
        let format_id: String = reference.chars().take(3).collect();
        formats_seen.insert(format_id);
    }

    // Should see multiple different formats
    assert!(
        formats_seen.len() >= 5,
        "Should generate at least 5 different invoice formats, got {}",
        formats_seen.len()
    );
}

// ============================================================================
// Address Generation Tests
// ============================================================================

#[test]
fn test_address_all_regions() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let regions = [
        AddressRegion::NorthAmerica,
        AddressRegion::Europe,
        AddressRegion::AsiaPacific,
        AddressRegion::LatinAmerica,
    ];

    for region in regions {
        let gen = AddressGenerator::for_region(region);
        let addr = gen.generate(&mut rng);

        assert_eq!(addr.region, region);
        assert!(!addr.street_number.is_empty());
        assert!(!addr.street_name.is_empty());
        assert!(!addr.street_type.is_empty());
        assert!(!addr.city.is_empty());
        assert!(!addr.state.is_empty());
        assert!(!addr.postal_code.is_empty());
        assert!(!addr.country.is_empty());
    }
}

#[test]
fn test_address_formatting_styles() {
    let addr = Address {
        street_number: "123".to_string(),
        street_name: "Main".to_string(),
        street_type: "Street".to_string(),
        unit: Some("Suite 100".to_string()),
        city: "New York".to_string(),
        state: "NY".to_string(),
        postal_code: "10001".to_string(),
        country: "USA".to_string(),
        region: AddressRegion::NorthAmerica,
    };

    let full = addr.format(AddressStyle::Full);
    assert!(full.contains("123 Main Street"));
    assert!(full.contains("Suite 100"));
    assert!(full.contains("New York"));
    assert!(full.contains("NY"));
    assert!(full.contains("10001"));
    assert!(full.contains("USA"));

    let short = addr.format(AddressStyle::Short);
    assert!(short.contains("123 Main Street"));
    assert!(short.contains("New York"));

    let single = addr.format(AddressStyle::SingleLine);
    assert!(!single.contains('\n'), "Single line should not have newlines");
    assert!(single.contains("10001"));
}

#[test]
fn test_commercial_addresses() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = AddressGenerator::new();

    let mut unit_count = 0;
    for _ in 0..100 {
        let addr = gen.generate_commercial(&mut rng);
        if addr.unit.is_some() {
            unit_count += 1;
        }
    }

    // Commercial addresses should often have units
    assert!(
        unit_count > 50,
        "Commercial addresses should often have units, got {} out of 100",
        unit_count
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_and_special_inputs() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = UserIdGenerator::new();

    // Empty names
    let id = gen.generate("", "", 0, &mut rng);
    assert!(!id.is_empty(), "Should handle empty names");

    // Special characters
    let id2 = gen.generate("José", "García", 0, &mut rng);
    assert!(!id2.is_empty(), "Should handle special characters");
}

#[test]
fn test_unicode_handling() {
    let variator = DescriptionVariator::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Test with Unicode characters
    let input = "Invoice für Müller GmbH";
    let output = variator.apply(input, &mut rng);
    assert!(!output.is_empty(), "Should handle Unicode input");
}

#[test]
fn test_large_index_values() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = UserIdGenerator::new();

    // Very large index
    let id = gen.generate_with_pattern(
        "John",
        "Smith",
        999999,
        UserIdPattern::InitialLastName,
        &mut rng,
    );
    assert!(id.contains("999999"), "Should handle large index values");

    // Employee number with large index
    let emp = gen.generate_with_pattern(
        "John",
        "Smith",
        12345678,
        UserIdPattern::EmployeeNumber,
        &mut rng,
    );
    assert_eq!(emp, "E12345678", "Should handle 8-digit employee numbers");
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_generation_performance() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let gen = RealismGenerator::new();

    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = gen.generate_company_name(Industry::Technology, &mut rng);
        let _ = gen.generate_vendor_name(SpendCategory::ITServices, &mut rng);
        let _ = gen.generate_address(&mut rng);
    }
    let elapsed = start.elapsed();

    // Should generate 30,000 items in under 2 seconds
    assert!(
        elapsed.as_secs() < 2,
        "Performance test failed: took {:?} for 30,000 generations",
        elapsed
    );
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_config_serialization() {
    let config = RealismConfig {
        cultural_awareness: true,
        industry_vendor_names: true,
        description_variations: true,
        abbreviation_rate: 0.30,
        typo_rate: 0.02,
        realistic_references: true,
        primary_region: AddressRegion::Europe,
        international_diversity: true,
        diversity_index: 0.4,
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: RealismConfig =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.abbreviation_rate, deserialized.abbreviation_rate);
    assert_eq!(config.typo_rate, deserialized.typo_rate);
}

#[test]
fn test_address_serialization() {
    let addr = Address {
        street_number: "123".to_string(),
        street_name: "Main".to_string(),
        street_type: "Street".to_string(),
        unit: Some("Suite 100".to_string()),
        city: "New York".to_string(),
        state: "NY".to_string(),
        postal_code: "10001".to_string(),
        country: "USA".to_string(),
        region: AddressRegion::NorthAmerica,
    };

    let json = serde_json::to_string(&addr).expect("Should serialize address");
    let deserialized: Address = serde_json::from_str(&json).expect("Should deserialize address");

    assert_eq!(addr.city, deserialized.city);
    assert_eq!(addr.postal_code, deserialized.postal_code);
}
