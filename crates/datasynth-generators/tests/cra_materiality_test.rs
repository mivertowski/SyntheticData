//! Integration tests for the ISA 315 CRA and ISA 320 materiality generators.

use datasynth_core::models::audit::materiality_calculation::{
    AdjustmentType, MaterialityBenchmark, MaterialityCalculation, NormalizationAdjustment,
    NormalizedEarnings,
};
use datasynth_core::models::audit::risk_assessment_cra::{
    AuditAssertion, CraLevel, CraPlannedResponse, ProcedureNature, ProcedureTiming, RiskRating,
    SamplingExtent,
};
use datasynth_generators::audit::cra_generator::CraGenerator;
use datasynth_generators::audit::materiality_generator::{MaterialityGenerator, MaterialityInput};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// CRA matrix tests
// ============================================================================

#[test]
fn cra_matrix_low_low_gives_minimal() {
    let level = CraLevel::from_ratings(RiskRating::Low, RiskRating::Low);
    assert_eq!(level, CraLevel::Minimal);
}

#[test]
fn cra_matrix_low_medium_gives_low() {
    let level = CraLevel::from_ratings(RiskRating::Low, RiskRating::Medium);
    assert_eq!(level, CraLevel::Low);
}

#[test]
fn cra_matrix_medium_low_gives_low() {
    let level = CraLevel::from_ratings(RiskRating::Medium, RiskRating::Low);
    assert_eq!(level, CraLevel::Low);
}

#[test]
fn cra_matrix_medium_medium_gives_moderate() {
    let level = CraLevel::from_ratings(RiskRating::Medium, RiskRating::Medium);
    assert_eq!(level, CraLevel::Moderate);
}

#[test]
fn cra_matrix_high_low_gives_moderate() {
    let level = CraLevel::from_ratings(RiskRating::High, RiskRating::Low);
    assert_eq!(level, CraLevel::Moderate);
}

#[test]
fn cra_matrix_high_medium_gives_high() {
    let level = CraLevel::from_ratings(RiskRating::High, RiskRating::Medium);
    assert_eq!(level, CraLevel::High);
}

#[test]
fn cra_matrix_high_high_gives_high() {
    let level = CraLevel::from_ratings(RiskRating::High, RiskRating::High);
    assert_eq!(level, CraLevel::High);
}

#[test]
fn cra_matrix_medium_high_gives_high() {
    let level = CraLevel::from_ratings(RiskRating::Medium, RiskRating::High);
    assert_eq!(level, CraLevel::High);
}

#[test]
fn cra_matrix_low_high_gives_moderate() {
    // Low IR + High CR → Moderate: low inherent risk caps the combined level even
    // when controls provide no assurance (catch-all else branch in matrix).
    let level = CraLevel::from_ratings(RiskRating::Low, RiskRating::High);
    assert_eq!(level, CraLevel::Moderate);
}

// ============================================================================
// Planned response tests
// ============================================================================

#[test]
fn planned_response_minimal_is_substantive_reduced_yearend() {
    let r = CraPlannedResponse::from_cra_level(CraLevel::Minimal);
    assert_eq!(r.nature, ProcedureNature::SubstantiveOnly);
    assert_eq!(r.extent, SamplingExtent::Reduced);
    assert_eq!(r.timing, ProcedureTiming::YearEnd);
}

#[test]
fn planned_response_low_is_substantive_reduced_yearend() {
    let r = CraPlannedResponse::from_cra_level(CraLevel::Low);
    assert_eq!(r.nature, ProcedureNature::SubstantiveOnly);
    assert_eq!(r.extent, SamplingExtent::Reduced);
    assert_eq!(r.timing, ProcedureTiming::YearEnd);
}

#[test]
fn planned_response_moderate_is_combined_standard_yearend() {
    let r = CraPlannedResponse::from_cra_level(CraLevel::Moderate);
    assert_eq!(r.nature, ProcedureNature::Combined);
    assert_eq!(r.extent, SamplingExtent::Standard);
    assert_eq!(r.timing, ProcedureTiming::YearEnd);
}

#[test]
fn planned_response_high_is_substantive_extended_both() {
    let r = CraPlannedResponse::from_cra_level(CraLevel::High);
    assert_eq!(r.nature, ProcedureNature::SubstantiveOnly);
    assert_eq!(r.extent, SamplingExtent::Extended);
    assert_eq!(r.timing, ProcedureTiming::Both);
}

// ============================================================================
// CRA generator tests
// ============================================================================

#[test]
fn cra_generator_produces_cras_for_entity() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);
    assert!(!cras.is_empty(), "Should produce CRAs");
    // 12 account areas × 2-3 assertions each → at least 24 CRAs
    assert!(cras.len() >= 20);
}

#[test]
fn revenue_occurrence_always_significant() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);

    let rev_occurrence = cras
        .iter()
        .find(|c| c.account_area == "Revenue" && c.assertion == AuditAssertion::Occurrence)
        .expect("Revenue/Occurrence CRA must exist");

    assert!(
        rev_occurrence.significant_risk,
        "Revenue Occurrence must always be flagged as significant risk per ISA 240"
    );
}

#[test]
fn related_party_occurrence_is_significant() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);

    let rp_occurrence = cras
        .iter()
        .find(|c| c.account_area == "Related Parties" && c.assertion == AuditAssertion::Occurrence)
        .expect("Related Parties/Occurrence CRA must exist");

    assert!(
        rp_occurrence.significant_risk,
        "Related party occurrence must be flagged as significant"
    );
}

#[test]
fn cra_ids_are_unique_within_entity() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);
    let ids: std::collections::HashSet<&str> = cras.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        cras.len(),
        "All CRA IDs must be unique within an entity"
    );
}

#[test]
fn combined_risk_matches_ir_cr_matrix() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);

    for cra in &cras {
        let expected = CraLevel::from_ratings(cra.inherent_risk, cra.control_risk);
        assert_eq!(
            cra.combined_risk, expected,
            "CRA level for {}/{:?} must equal matrix result of {:?} × {:?}",
            cra.account_area, cra.assertion, cra.inherent_risk, cra.control_risk
        );
    }
}

#[test]
fn planned_response_consistent_with_cra_level() {
    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", None);

    for cra in &cras {
        let expected = CraPlannedResponse::from_cra_level(cra.combined_risk);
        assert_eq!(
            cra.planned_response.nature, expected.nature,
            "Planned response nature mismatch for {}/{:?}",
            cra.account_area, cra.assertion
        );
        assert_eq!(
            cra.planned_response.extent, expected.extent,
            "Planned response extent mismatch for {}/{:?}",
            cra.account_area, cra.assertion
        );
        assert_eq!(
            cra.planned_response.timing, expected.timing,
            "Planned response timing mismatch for {}/{:?}",
            cra.account_area, cra.assertion
        );
    }
}

#[test]
fn control_override_drives_control_risk() {
    let mut overrides = std::collections::HashMap::new();
    overrides.insert("Cash".into(), RiskRating::Low);

    let mut gen = CraGenerator::new(42);
    let cras = gen.generate_for_entity("C001", Some(&overrides));

    let cash_cras: Vec<_> = cras.iter().filter(|c| c.account_area == "Cash").collect();
    assert!(!cash_cras.is_empty(), "Cash CRAs must be generated");

    for c in &cash_cras {
        assert_eq!(
            c.control_risk,
            RiskRating::Low,
            "Overridden control risk must be Low for Cash"
        );
    }
}

#[test]
fn multi_entity_cra_generation() {
    let mut gen = CraGenerator::new(42);
    let cras_c1 = gen.generate_for_entity("C001", None);
    let cras_c2 = gen.generate_for_entity("C002", None);

    // All C001 CRAs should reference C001
    for c in &cras_c1 {
        assert_eq!(c.entity_code, "C001");
    }
    // All C002 CRAs should reference C002
    for c in &cras_c2 {
        assert_eq!(c.entity_code, "C002");
    }
}

// ============================================================================
// Materiality model tests
// ============================================================================

#[test]
fn materiality_overall_equals_benchmark_times_percentage() {
    let calc = MaterialityCalculation::new(
        "C001",
        "FY2024",
        MaterialityBenchmark::PretaxIncome,
        dec!(1_000_000),
        dec!(0.05),
        dec!(0.65),
        None,
        "5% of pre-tax income",
    );
    assert_eq!(calc.overall_materiality, dec!(50_000));
}

#[test]
fn materiality_pm_between_50_and_75_percent() {
    let overall = dec!(50_000);
    let pm_pct = dec!(0.65);
    let calc = MaterialityCalculation::new(
        "C001",
        "FY2024",
        MaterialityBenchmark::PretaxIncome,
        dec!(1_000_000),
        dec!(0.05),
        pm_pct,
        None,
        "Test",
    );
    let ratio = calc.performance_materiality / calc.overall_materiality;
    assert!(
        ratio >= dec!(0.50),
        "PM must be >= 50% of overall, got {ratio}"
    );
    assert!(
        ratio <= dec!(0.75),
        "PM must be <= 75% of overall, got {ratio}"
    );
    assert_eq!(calc.performance_materiality, overall * pm_pct);
}

#[test]
fn materiality_clearly_trivial_is_five_percent_of_overall() {
    let calc = MaterialityCalculation::new(
        "C001",
        "FY2024",
        MaterialityBenchmark::Revenue,
        dec!(10_000_000),
        dec!(0.005),
        dec!(0.65),
        None,
        "0.5% of revenue",
    );
    let expected = calc.overall_materiality * dec!(0.05);
    assert_eq!(calc.clearly_trivial, expected);
}

#[test]
fn materiality_tolerable_error_equals_pm() {
    let calc = MaterialityCalculation::new(
        "C001",
        "FY2024",
        MaterialityBenchmark::PretaxIncome,
        dec!(500_000),
        dec!(0.05),
        dec!(0.65),
        None,
        "Test",
    );
    assert_eq!(calc.tolerable_error, calc.performance_materiality);
}

#[test]
fn materiality_sad_nominal_is_five_percent_of_om() {
    let calc = MaterialityCalculation::new(
        "C001",
        "FY2024",
        MaterialityBenchmark::TotalAssets,
        dec!(8_000_000),
        dec!(0.005),
        dec!(0.65),
        None,
        "Test",
    );
    // SAD = 5% of overall materiality (ISA 450 common practice)
    let om = dec!(8_000_000) * dec!(0.005);
    let expected_sad = om * dec!(0.05);
    assert_eq!(calc.sad_nominal, expected_sad);
}

#[test]
fn normalized_earnings_adjustments_sum_correctly() {
    let adjustments = vec![
        NormalizationAdjustment {
            description: "Restructuring charge".into(),
            amount: dec!(200_000),
            adjustment_type: AdjustmentType::NonRecurring,
        },
        NormalizationAdjustment {
            description: "Disposal gain".into(),
            amount: dec!(-75_000),
            adjustment_type: AdjustmentType::Extraordinary,
        },
        NormalizationAdjustment {
            description: "Reclassification".into(),
            amount: dec!(10_000),
            adjustment_type: AdjustmentType::Reclassification,
        },
    ];

    let ne = NormalizedEarnings::new(dec!(500_000), adjustments);

    let expected_normalized = dec!(500_000) + dec!(200_000) + dec!(-75_000) + dec!(10_000);
    assert_eq!(ne.normalized_amount, expected_normalized);
    assert_eq!(ne.reported_earnings, dec!(500_000));
    assert_eq!(ne.adjustments.len(), 3);
}

// ============================================================================
// Materiality generator tests
// ============================================================================

fn make_profitable_input() -> MaterialityInput {
    MaterialityInput {
        entity_code: "C001".into(),
        period: "FY2024".into(),
        revenue: dec!(10_000_000),
        pretax_income: dec!(1_000_000), // 10% margin → healthy profit
        total_assets: dec!(8_000_000),
        equity: dec!(4_000_000),
        gross_profit: dec!(3_500_000),
    }
}

fn make_loss_input() -> MaterialityInput {
    MaterialityInput {
        entity_code: "LOSS".into(),
        period: "FY2024".into(),
        revenue: dec!(5_000_000),
        pretax_income: dec!(-200_000),
        total_assets: dec!(3_000_000),
        equity: dec!(1_000_000),
        gross_profit: dec!(500_000),
    }
}

fn make_asset_heavy_input() -> MaterialityInput {
    MaterialityInput {
        entity_code: "BANK".into(),
        period: "FY2024".into(),
        revenue: dec!(1_000_000),
        pretax_income: dec!(150_000),
        total_assets: dec!(50_000_000), // 50× revenue
        equity: dec!(5_000_000),
        gross_profit: dec!(800_000),
    }
}

#[test]
fn generator_profitable_entity_uses_pretax_income() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_profitable_input());
    assert_eq!(
        calc.benchmark,
        MaterialityBenchmark::PretaxIncome,
        "Healthy profitable entity should use pre-tax income"
    );
}

#[test]
fn generator_loss_entity_uses_revenue() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_loss_input());
    assert_eq!(
        calc.benchmark,
        MaterialityBenchmark::Revenue,
        "Loss-making entity should use revenue benchmark"
    );
}

#[test]
fn generator_asset_heavy_entity_uses_total_assets() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_asset_heavy_input());
    assert_eq!(
        calc.benchmark,
        MaterialityBenchmark::TotalAssets,
        "Asset-heavy entity should use total assets benchmark"
    );
}

#[test]
fn generator_pm_always_between_50_and_75_percent() {
    let mut gen = MaterialityGenerator::new(42);
    let inputs = vec![
        make_profitable_input(),
        make_loss_input(),
        make_asset_heavy_input(),
    ];
    for input in &inputs {
        let calc = gen.generate(input);
        if calc.overall_materiality > Decimal::ZERO {
            let ratio = calc.performance_materiality / calc.overall_materiality;
            assert!(
                ratio >= dec!(0.50),
                "PM/overall ratio {} < 0.50 for entity {}",
                ratio,
                input.entity_code
            );
            assert!(
                ratio <= dec!(0.75),
                "PM/overall ratio {} > 0.75 for entity {}",
                ratio,
                input.entity_code
            );
        }
    }
}

#[test]
fn generator_clearly_trivial_is_five_percent_of_overall() {
    let mut gen = MaterialityGenerator::new(42);
    for input in &[make_profitable_input(), make_loss_input()] {
        let calc = gen.generate(input);
        let expected_ct = calc.overall_materiality * dec!(0.05);
        assert_eq!(
            calc.clearly_trivial, expected_ct,
            "Clearly trivial should be 5% of overall for entity {}",
            input.entity_code
        );
    }
}

#[test]
fn generator_minimum_floor_applied() {
    let mut gen = MaterialityGenerator::new(42);
    let tiny = MaterialityInput {
        entity_code: "MICRO".into(),
        period: "FY2024".into(),
        revenue: dec!(5_000),
        pretax_income: dec!(100),
        total_assets: dec!(2_000),
        equity: dec!(500),
        gross_profit: dec!(800),
    };
    let calc = gen.generate(&tiny);
    assert!(
        calc.overall_materiality >= dec!(5_000),
        "Minimum materiality floor of 5,000 must apply; got {}",
        calc.overall_materiality
    );
}

#[test]
fn generator_batch_generates_one_per_entity() {
    let mut gen = MaterialityGenerator::new(42);
    let inputs = vec![
        make_profitable_input(),
        make_loss_input(),
        make_asset_heavy_input(),
    ];
    let calcs = gen.generate_batch(&inputs);
    assert_eq!(calcs.len(), 3, "One calculation per input entity");
    assert_eq!(calcs[0].entity_code, "C001");
    assert_eq!(calcs[1].entity_code, "LOSS");
    assert_eq!(calcs[2].entity_code, "BANK");
}

#[test]
fn generator_rationale_is_non_empty() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_profitable_input());
    assert!(!calc.rationale.is_empty(), "Rationale must be populated");
}

#[test]
fn generator_tolerable_error_equals_pm() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_profitable_input());
    assert_eq!(
        calc.tolerable_error, calc.performance_materiality,
        "Tolerable error must equal performance materiality"
    );
}

#[test]
fn generator_sad_is_five_percent_of_om() {
    let mut gen = MaterialityGenerator::new(42);
    let calc = gen.generate(&make_profitable_input());
    // SAD nominal = 5% of overall materiality
    let expected_sad = calc.overall_materiality * dec!(0.05);
    assert_eq!(calc.sad_nominal, expected_sad);
}
