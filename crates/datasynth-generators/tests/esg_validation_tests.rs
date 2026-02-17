//! ESG / Sustainability pipeline integration tests.
//!
//! Verifies: full pipeline from operational data to disclosures,
//! Scope 1+2+3 coherence, TRIR formula, material topic coverage,
//! anomaly injection, and deterministic reproducibility.

use chrono::NaiveDate;
use datasynth_config::schema::{
    ClimateScenarioConfig, EnvironmentalConfig, EsgReportingConfig, SocialConfig,
    SupplyChainEsgConfig,
};
use datasynth_generators::esg::{
    DisclosureGenerator, EmissionGenerator, EnergyGenerator, EnergyInput, EnergyInputType,
    EsgAnomalyInjector, GovernanceGenerator, SupplierEsgGenerator, VendorInput, VendorSpendInput,
    WasteGenerator, WaterGenerator, WorkforceGenerator,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

// ---------------------------------------------------------------------------
// Full pipeline test
// ---------------------------------------------------------------------------

#[test]
fn test_full_esg_pipeline() {
    let entity = "C001";
    let start = d("2025-01-01");
    let end = d("2025-12-01");

    // 1. Energy
    let mut energy_gen = EnergyGenerator::new(42, EnvironmentalConfig::default().energy);
    let energy_records = energy_gen.generate(entity, start, end);
    assert!(!energy_records.is_empty(), "Should generate energy records");

    // 2. Emissions from energy
    let mut emission_gen = EmissionGenerator::new(42, EnvironmentalConfig::default());
    let energy_inputs: Vec<EnergyInput> = energy_records
        .iter()
        .map(|e| EnergyInput {
            facility_id: e.facility_id.clone(),
            energy_type: if e.is_renewable {
                EnergyInputType::Electricity // Renewables → Scope 2
            } else {
                match format!("{:?}", e.energy_source).as_str() {
                    "NaturalGas" => EnergyInputType::NaturalGas,
                    "Diesel" => EnergyInputType::Diesel,
                    "Coal" => EnergyInputType::Coal,
                    _ => EnergyInputType::Electricity,
                }
            },
            consumption_kwh: e.consumption_kwh,
            period: e.period,
        })
        .collect();

    let scope1 = emission_gen.generate_scope1(entity, &energy_inputs);
    let scope2 = emission_gen.generate_scope2(entity, &energy_inputs);

    // 3. Scope 3 from vendor spend
    let vendor_spend = vec![
        VendorSpendInput {
            vendor_id: "V-001".into(),
            category: "manufacturing".into(),
            spend: dec!(500000),
            country: "US".into(),
        },
        VendorSpendInput {
            vendor_id: "V-002".into(),
            category: "office_supplies".into(),
            spend: dec!(50000),
            country: "US".into(),
        },
    ];
    let scope3 = emission_gen.generate_scope3_purchased_goods(entity, &vendor_spend, start, end);

    // 4. Water and Waste
    let mut water_gen = WaterGenerator::new(42, 3);
    let _water = water_gen.generate(entity, start, end);
    let mut waste_gen = WasteGenerator::new(42, 0.50, 3);
    let _waste = waste_gen.generate(entity, start, end);

    // 5. Workforce
    let mut workforce_gen = WorkforceGenerator::new(42, SocialConfig::default());
    let diversity = workforce_gen.generate_diversity(entity, 500, d("2025-06-30"));
    let _pay_equity = workforce_gen.generate_pay_equity(entity, d("2025-06-30"));
    let incidents = workforce_gen.generate_safety_incidents(entity, 3, start, d("2025-12-31"));
    let safety_metric =
        workforce_gen.compute_safety_metrics(entity, &incidents, 1_000_000, d("2025-06-30"));

    // 6. Governance
    let mut gov_gen = GovernanceGenerator::new(42, 11, 0.67);
    let _governance = gov_gen.generate(entity, d("2025-06-30"));

    // 7. Supplier ESG
    let vendors = vec![
        VendorInput {
            vendor_id: "V-001".into(),
            country: "US".into(),
            industry: "manufacturing".into(),
            quality_score: Some(80.0),
        },
        VendorInput {
            vendor_id: "V-002".into(),
            country: "CN".into(),
            industry: "technology".into(),
            quality_score: Some(70.0),
        },
    ];
    let mut supplier_gen = SupplierEsgGenerator::new(42, SupplyChainEsgConfig::default());
    let _assessments = supplier_gen.generate(entity, &vendors, d("2025-06-01"));

    // 8. Disclosures
    let mut disc_gen = DisclosureGenerator::new(
        42,
        EsgReportingConfig::default(),
        ClimateScenarioConfig {
            enabled: true,
            ..Default::default()
        },
    );
    let materiality = disc_gen.generate_materiality(entity, d("2025-01-01"));
    let disclosures = disc_gen.generate_disclosures(entity, &materiality, start, d("2025-12-31"));
    let scenarios = disc_gen.generate_climate_scenarios(entity);

    // Verify pipeline produced data across all categories
    assert!(
        !scope1.is_empty() || !scope2.is_empty(),
        "Should have Scope 1 or 2 emissions"
    );
    assert!(!scope3.is_empty(), "Should have Scope 3 emissions");
    assert!(!diversity.is_empty(), "Should have diversity metrics");
    assert!(safety_metric.total_hours_worked > 0);
    assert!(
        !materiality.is_empty(),
        "Should have materiality assessments"
    );
    assert!(!disclosures.is_empty(), "Should have disclosures");
    assert!(!scenarios.is_empty(), "Should have climate scenarios");
}

// ---------------------------------------------------------------------------
// Emission coherence
// ---------------------------------------------------------------------------

#[test]
fn test_scope1_scope2_scope3_totals_coherent() {
    let mut gen = EmissionGenerator::new(42, EnvironmentalConfig::default());

    let fuel_inputs = vec![
        EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::NaturalGas,
            consumption_kwh: dec!(200000),
            period: d("2025-01-01"),
        },
        EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::Electricity,
            consumption_kwh: dec!(500000),
            period: d("2025-01-01"),
        },
    ];

    let scope1 = gen.generate_scope1("C001", &fuel_inputs);
    let scope2 = gen.generate_scope2("C001", &fuel_inputs);

    let s1_total: Decimal = scope1.iter().map(|r| r.co2e_tonnes).sum();
    let s2_total: Decimal = scope2.iter().map(|r| r.co2e_tonnes).sum();

    // Both should be positive
    assert!(s1_total > Decimal::ZERO, "Scope 1 total should be positive");
    assert!(s2_total > Decimal::ZERO, "Scope 2 total should be positive");

    // Scope 2 (electricity) typically > Scope 1 (gas) for similar consumption
    // because electricity grid factor (0.417) > natural gas (0.181)
    // and we have 500k kWh elec vs 200k kWh gas
    assert!(
        s2_total > s1_total,
        "Scope 2 ({}) should exceed Scope 1 ({}) given higher consumption and factor",
        s2_total,
        s1_total
    );
}

#[test]
fn test_higher_spend_produces_higher_scope3() {
    let mut gen = EmissionGenerator::new(42, EnvironmentalConfig::default());

    let low_spend = vec![VendorSpendInput {
        vendor_id: "V-001".into(),
        category: "office_supplies".into(),
        spend: dec!(10000),
        country: "US".into(),
    }];
    let high_spend = vec![VendorSpendInput {
        vendor_id: "V-002".into(),
        category: "office_supplies".into(),
        spend: dec!(100000),
        country: "US".into(),
    }];

    let low_result =
        gen.generate_scope3_purchased_goods("C001", &low_spend, d("2025-01-01"), d("2025-12-31"));
    let high_result =
        gen.generate_scope3_purchased_goods("C001", &high_spend, d("2025-01-01"), d("2025-12-31"));

    assert!(
        high_result[0].co2e_tonnes > low_result[0].co2e_tonnes,
        "Higher spend should produce more emissions"
    );
}

// ---------------------------------------------------------------------------
// TRIR formula
// ---------------------------------------------------------------------------

#[test]
fn test_trir_formula_matches_computed() {
    let mut gen = WorkforceGenerator::new(42, SocialConfig::default());
    let incidents = gen.generate_safety_incidents("C001", 5, d("2025-01-01"), d("2025-12-31"));
    let metric = gen.compute_safety_metrics("C001", &incidents, 2_000_000, d("2025-06-30"));

    assert_eq!(
        metric.trir,
        metric.computed_trir(),
        "TRIR should match computed value"
    );
    assert_eq!(
        metric.ltir,
        metric.computed_ltir(),
        "LTIR should match computed value"
    );
    assert_eq!(
        metric.dart_rate,
        metric.computed_dart_rate(),
        "DART should match computed value"
    );
}

// ---------------------------------------------------------------------------
// Material topics have disclosures
// ---------------------------------------------------------------------------

#[test]
fn test_all_material_topics_covered_by_disclosures() {
    let mut gen = DisclosureGenerator::new(
        42,
        EsgReportingConfig::default(),
        ClimateScenarioConfig::default(),
    );

    let materiality = gen.generate_materiality("C001", d("2025-01-01"));
    let disclosures =
        gen.generate_disclosures("C001", &materiality, d("2025-01-01"), d("2025-12-31"));

    let material_topics: Vec<&str> = materiality
        .iter()
        .filter(|m| m.is_material)
        .map(|m| m.topic.as_str())
        .collect();

    for topic in &material_topics {
        let found = disclosures
            .iter()
            .any(|d| d.disclosure_topic.contains(topic));
        assert!(
            found,
            "Material topic '{}' must have at least one disclosure",
            topic
        );
    }
}

// ---------------------------------------------------------------------------
// Anomaly injection
// ---------------------------------------------------------------------------

#[test]
fn test_greenwashing_reduces_emissions() {
    let mut gen = EmissionGenerator::new(42, EnvironmentalConfig::default());
    let inputs = vec![EnergyInput {
        facility_id: "F-001".into(),
        energy_type: EnergyInputType::NaturalGas,
        consumption_kwh: dec!(100000),
        period: d("2025-01-01"),
    }];
    let mut emissions = gen.generate_scope1("C001", &inputs);
    let original: Vec<Decimal> = emissions.iter().map(|e| e.co2e_tonnes).collect();

    let mut injector = EsgAnomalyInjector::new(42, 1.0);
    let labels = injector.inject_greenwashing(&mut emissions);

    assert_eq!(labels.len(), emissions.len());
    for (i, em) in emissions.iter().enumerate() {
        assert!(
            em.co2e_tonnes < original[i],
            "Emission should be reduced after greenwashing injection"
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn test_deterministic_esg_pipeline() {
    let config = EnvironmentalConfig::default();

    let mut gen1 = EmissionGenerator::new(42, config.clone());
    let mut gen2 = EmissionGenerator::new(42, config);

    let inputs = vec![
        EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::NaturalGas,
            consumption_kwh: dec!(100000),
            period: d("2025-01-01"),
        },
        EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::Electricity,
            consumption_kwh: dec!(200000),
            period: d("2025-01-01"),
        },
    ];

    let r1_s1 = gen1.generate_scope1("C001", &inputs);
    let r1_s2 = gen1.generate_scope2("C001", &inputs);

    let r2_s1 = gen2.generate_scope1("C001", &inputs);
    let r2_s2 = gen2.generate_scope2("C001", &inputs);

    assert_eq!(r1_s1.len(), r2_s1.len());
    assert_eq!(r1_s2.len(), r2_s2.len());
    for (a, b) in r1_s1.iter().zip(r2_s1.iter()) {
        assert_eq!(
            a.co2e_tonnes, b.co2e_tonnes,
            "Scope 1 should be deterministic"
        );
    }
    for (a, b) in r1_s2.iter().zip(r2_s2.iter()) {
        assert_eq!(
            a.co2e_tonnes, b.co2e_tonnes,
            "Scope 2 should be deterministic"
        );
    }
}

// ---------------------------------------------------------------------------
// Supplier ESG scores in range
// ---------------------------------------------------------------------------

#[test]
fn test_supplier_scores_in_valid_range() {
    let vendors = (0..20)
        .map(|i| VendorInput {
            vendor_id: format!("V-{:03}", i),
            country: if i % 3 == 0 { "CN" } else { "US" }.into(),
            industry: "manufacturing".into(),
            quality_score: Some(50.0 + i as f64 * 2.0),
        })
        .collect::<Vec<_>>();

    let config = SupplyChainEsgConfig {
        enabled: true,
        assessment_coverage: 1.0,
        high_risk_countries: vec!["CN".into()],
    };

    let mut gen = SupplierEsgGenerator::new(42, config);
    let assessments = gen.generate("C001", &vendors, d("2025-06-01"));

    assert_eq!(assessments.len(), 20);
    for a in &assessments {
        assert!(a.environmental_score >= Decimal::ZERO && a.environmental_score <= dec!(100));
        assert!(a.social_score >= Decimal::ZERO && a.social_score <= dec!(100));
        assert!(a.governance_score >= Decimal::ZERO && a.governance_score <= dec!(100));
        assert!(a.overall_score >= Decimal::ZERO && a.overall_score <= dec!(100));
    }
}

// ---------------------------------------------------------------------------
// Water consumption coherence
// ---------------------------------------------------------------------------

#[test]
fn test_water_consumption_equals_withdrawal_minus_discharge() {
    let mut gen = WaterGenerator::new(42, 3);
    let records = gen.generate("C001", d("2025-01-01"), d("2025-06-01"));

    for r in &records {
        let expected = (r.withdrawal_m3 - r.discharge_m3).round_dp(2);
        assert_eq!(
            r.consumption_m3, expected,
            "Consumption should equal withdrawal - discharge"
        );
    }
}

// ---------------------------------------------------------------------------
// Waste diversion coherence
// ---------------------------------------------------------------------------

#[test]
fn test_waste_diversion_flag_matches_method() {
    let mut gen = WasteGenerator::new(42, 0.60, 3);
    let records = gen.generate("C001", d("2025-01-01"), d("2025-06-01"));

    for r in &records {
        assert_eq!(
            r.is_diverted_from_landfill,
            r.computed_diversion(),
            "Diversion flag should match disposal method for {}",
            r.id
        );
    }
}

// ---------------------------------------------------------------------------
// Climate scenario structure
// ---------------------------------------------------------------------------

#[test]
fn test_climate_scenario_structure() {
    let mut gen = DisclosureGenerator::new(
        42,
        EsgReportingConfig::default(),
        ClimateScenarioConfig {
            enabled: true,
            ..Default::default()
        },
    );
    let scenarios = gen.generate_climate_scenarios("C001");

    // 4 types × 3 horizons = 12
    assert_eq!(scenarios.len(), 12);

    // All financial impacts should be positive
    for s in &scenarios {
        assert!(s.financial_impact >= Decimal::ZERO);
        assert!(s.transition_risk_impact >= Decimal::ZERO);
        assert!(s.physical_risk_impact >= Decimal::ZERO);
    }
}
