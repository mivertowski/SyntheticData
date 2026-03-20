//! Integration tests for ISA 530 sampling plans and ISA 315 SCOTS generators.
//!
//! Tests validate:
//! - Key items all have amount ≥ tolerable error
//! - Sample size correlates correctly with CRA level
//! - Sampling interval = remaining_population / sample_size
//! - All standard SCOTs present for a standard entity
//! - Estimation SCOTs have complexity; non-estimation do not
//! - Volume derives from JE data when available

#![allow(clippy::unwrap_used)]

use datasynth_core::models::audit::risk_assessment_cra::{
    AuditAssertion, CombinedRiskAssessment, CraLevel, RiskRating,
};
use datasynth_core::models::audit::sampling_plan::{KeyItemReason, SamplingMethodology, SelectionType};
use datasynth_core::models::audit::scots::{EstimationComplexity, ScotSignificance, ScotTransactionType};
use datasynth_generators::audit::sampling_plan_generator::SamplingPlanGenerator;
use datasynth_generators::audit::scots_generator::{ScotsGenerator, ScotsGeneratorConfig};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// Helpers
// ============================================================================

fn make_cra(
    area: &str,
    assertion: AuditAssertion,
    ir: RiskRating,
    cr: RiskRating,
    significant: bool,
) -> CombinedRiskAssessment {
    CombinedRiskAssessment::new("C001", area, assertion, ir, cr, significant, vec![])
}

fn make_high_cra(area: &str, assertion: AuditAssertion) -> CombinedRiskAssessment {
    make_cra(area, assertion, RiskRating::High, RiskRating::High, true)
}

fn make_moderate_cra(area: &str, assertion: AuditAssertion) -> CombinedRiskAssessment {
    make_cra(area, assertion, RiskRating::Medium, RiskRating::Medium, false)
}

fn make_low_cra(area: &str, assertion: AuditAssertion) -> CombinedRiskAssessment {
    make_cra(area, assertion, RiskRating::Low, RiskRating::Low, false)
}

const TEST_TE: Decimal = dec!(32_500);

// ============================================================================
// Sampling Plan Tests
// ============================================================================

#[test]
fn sampling_plan_minimal_cra_skipped() {
    let minimal_cra = make_low_cra("Cash", AuditAssertion::Existence);
    assert_eq!(minimal_cra.combined_risk, CraLevel::Minimal);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras(&[minimal_cra], Some(TEST_TE));

    assert!(plans.is_empty(), "Minimal CRA should produce no sampling plan");
    assert!(items.is_empty(), "Minimal CRA should produce no sampled items");
}

#[test]
fn sampling_plan_low_cra_skipped() {
    // Low + Medium = Low CRA
    let low_cra = make_cra("Cost of Sales", AuditAssertion::Occurrence, RiskRating::Low, RiskRating::Medium, false);
    assert_eq!(low_cra.combined_risk, CraLevel::Low);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _) = gen.generate_for_cras(&[low_cra], Some(TEST_TE));
    assert!(plans.is_empty(), "Low CRA should produce no sampling plan");
}

#[test]
fn sampling_plan_moderate_cra_produces_plan() {
    let cra = make_moderate_cra("Trade Receivables", AuditAssertion::Existence);
    assert_eq!(cra.combined_risk, CraLevel::Moderate);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1, "Moderate CRA should produce exactly one plan");
    let plan = &plans[0];
    assert!(
        plan.sample_size >= 20 && plan.sample_size <= 30,
        "Moderate CRA sample size should be 20–30, got {}",
        plan.sample_size
    );
    assert!(!items.is_empty(), "Should produce sampled items");
}

#[test]
fn sampling_plan_high_cra_large_sample() {
    let cra = make_high_cra("Revenue", AuditAssertion::Occurrence);
    assert_eq!(cra.combined_risk, CraLevel::High);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert!(
        plan.sample_size >= 40 && plan.sample_size <= 60,
        "High CRA sample size should be 40–60, got {}",
        plan.sample_size
    );
}

#[test]
fn sampling_plan_key_items_all_above_tolerable_error() {
    let cras = vec![
        make_high_cra("Revenue", AuditAssertion::Occurrence),
        make_moderate_cra("Inventory", AuditAssertion::Existence),
        make_high_cra("Provisions", AuditAssertion::ValuationAndAllocation),
    ];

    let mut gen = SamplingPlanGenerator::new(99);
    let (plans, _) = gen.generate_for_cras(&cras, Some(TEST_TE));

    assert!(!plans.is_empty());
    for plan in &plans {
        for ki in &plan.key_items {
            assert!(
                ki.amount >= TEST_TE,
                "Key item {} amount {} must be >= TE {}",
                ki.item_id,
                ki.amount,
                TEST_TE
            );
        }
    }
}

#[test]
fn sampling_plan_first_key_item_always_above_te() {
    let cra = make_high_cra("Trade Receivables", AuditAssertion::ValuationAndAllocation);

    for seed in [1u64, 7, 42, 99, 123] {
        let mut gen = SamplingPlanGenerator::new(seed);
        let (plans, _) = gen.generate_for_cras(&[cra.clone()], Some(TEST_TE));
        assert!(!plans.is_empty());
        let ki0 = &plans[0].key_items[0];
        assert_eq!(ki0.reason, KeyItemReason::AboveTolerableError, "First key item must be AboveTolerableError");
        assert!(ki0.amount >= TEST_TE, "First key item amount {} >= TE {}", ki0.amount, TEST_TE);
    }
}

#[test]
fn sampling_plan_interval_equals_remaining_over_sample_size() {
    let cras = vec![
        make_high_cra("Revenue", AuditAssertion::Occurrence),
        make_moderate_cra("Inventory", AuditAssertion::Existence),
    ];

    let mut gen = SamplingPlanGenerator::new(7);
    let (plans, _) = gen.generate_for_cras(&cras, Some(TEST_TE));

    for plan in &plans {
        if plan.sample_size > 0 && plan.remaining_population_value > Decimal::ZERO {
            let expected = plan.remaining_population_value / Decimal::from(plan.sample_size as i64);
            let diff = (plan.sampling_interval - expected).abs();
            assert!(
                diff < dec!(0.01),
                "Plan '{}': interval {} ≠ remaining/n {}",
                plan.id,
                plan.sampling_interval,
                expected
            );
        }
    }
}

#[test]
fn sampling_plan_key_items_value_sums_correctly() {
    let cra = make_high_cra("Revenue", AuditAssertion::Occurrence);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    let computed_key_value: Decimal = plan.key_items.iter().map(|k| k.amount).sum();
    let diff = (computed_key_value - plan.key_items_value).abs();
    assert!(diff < dec!(0.01), "key_items_value mismatch: computed={} stored={}", computed_key_value, plan.key_items_value);
}

#[test]
fn sampling_plan_remaining_value_is_population_minus_key_items() {
    let cra = make_moderate_cra("Inventory", AuditAssertion::Existence);

    let mut gen = SamplingPlanGenerator::new(55);
    let (plans, _) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    let expected_remaining = plan.population_value - plan.key_items_value;
    let diff = (plan.remaining_population_value - expected_remaining).abs();
    assert!(diff < dec!(0.01), "remaining_population_value mismatch");
}

#[test]
fn sampling_plan_balance_assertion_uses_mus() {
    let cra = make_moderate_cra("Trade Receivables", AuditAssertion::Existence);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].methodology,
        SamplingMethodology::MonetaryUnitSampling,
        "Balance assertion should use MUS"
    );
}

#[test]
fn sampling_plan_transaction_assertion_uses_systematic() {
    let cra = make_moderate_cra("Revenue", AuditAssertion::Occurrence);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].methodology,
        SamplingMethodology::SystematicSelection,
        "Transaction assertion should use Systematic"
    );
}

#[test]
fn sampled_items_are_all_tested() {
    let cras = vec![
        make_high_cra("Revenue", AuditAssertion::Occurrence),
        make_moderate_cra("Provisions", AuditAssertion::ValuationAndAllocation),
    ];

    let mut gen = SamplingPlanGenerator::new(33);
    let (_, items) = gen.generate_for_cras(&cras, Some(TEST_TE));

    assert!(!items.is_empty());
    assert!(items.iter().all(|i| i.tested), "All sampled items should be marked as tested");
}

#[test]
fn sampled_items_include_key_and_representative() {
    let cra = make_high_cra("Provisions", AuditAssertion::ValuationAndAllocation);

    let mut gen = SamplingPlanGenerator::new(42);
    let (_, items) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    let key_count = items.iter().filter(|i| i.selection_type == SelectionType::KeyItem).count();
    let rep_count = items.iter().filter(|i| i.selection_type == SelectionType::Representative).count();

    assert!(key_count > 0, "Should have key items");
    assert!(rep_count > 0, "Should have representative items");
}

#[test]
fn misstatements_have_nonzero_amount() {
    let cra = make_high_cra("Revenue", AuditAssertion::Occurrence);

    let mut gen = SamplingPlanGenerator::new(42);
    let (_, items) = gen.generate_for_cras(&[cra], Some(TEST_TE));

    for item in items.iter().filter(|i| i.misstatement_found) {
        let amt = item.misstatement_amount.unwrap_or(Decimal::ZERO);
        assert!(
            amt > Decimal::ZERO,
            "Misstatement amount must be > 0 when misstatement_found=true"
        );
    }
}

#[test]
fn sampling_plan_ids_are_unique_across_entities() {
    let cras_c001 = vec![
        make_moderate_cra("Revenue", AuditAssertion::Occurrence),
        make_high_cra("Inventory", AuditAssertion::Existence),
    ];
    let cras_c002 = vec![
        CombinedRiskAssessment::new("C002", "Revenue", AuditAssertion::Occurrence, RiskRating::Medium, RiskRating::Medium, false, vec![]),
    ];

    let mut gen = SamplingPlanGenerator::new(42);
    let te = Some(TEST_TE);
    let (mut plans, _) = gen.generate_for_cras(&cras_c001, te);
    let (plans2, _) = gen.generate_for_cras(&cras_c002, te);
    plans.extend(plans2);

    let ids: std::collections::HashSet<&str> = plans.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids.len(), plans.len(), "Plan IDs should be unique across entities");
}

// ============================================================================
// SCOTS Tests
// ============================================================================

#[test]
fn scots_generates_8_standard_scots_without_ic() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    assert_eq!(
        scots.len(),
        8,
        "Should generate 8 non-IC SCOTs, got {}",
        scots.len()
    );
}

#[test]
fn scots_generates_9_scots_with_ic() {
    let config = ScotsGeneratorConfig {
        intercompany_enabled: true,
        ..ScotsGeneratorConfig::default()
    };
    let mut gen = ScotsGenerator::with_config(42, config);
    let scots = gen.generate_for_entity("C001", &[]);

    assert_eq!(scots.len(), 9, "Should generate 9 SCOTs with IC enabled");
    assert!(
        scots.iter().any(|s| s.business_process == "IC"),
        "IC SCOT should be present"
    );
}

#[test]
fn scots_estimation_types_have_complexity() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in scots.iter().filter(|s| s.transaction_type == ScotTransactionType::Estimation) {
        assert!(
            s.estimation_complexity.is_some(),
            "Estimation SCOT '{}' must have estimation_complexity set",
            s.scot_name
        );
    }
}

#[test]
fn scots_non_estimation_types_have_no_complexity() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in scots.iter().filter(|s| s.transaction_type != ScotTransactionType::Estimation) {
        assert!(
            s.estimation_complexity.is_none(),
            "Non-estimation SCOT '{}' must not have estimation_complexity",
            s.scot_name
        );
    }
}

#[test]
fn scots_all_have_four_critical_path_stages() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in &scots {
        assert_eq!(
            s.critical_path.len(),
            4,
            "SCOT '{}' should have 4 critical path stages, got {}",
            s.scot_name,
            s.critical_path.len()
        );
    }
}

#[test]
fn scots_ids_are_unique() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let ids: std::collections::HashSet<&str> = scots.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), scots.len(), "SCOT IDs should be unique");
}

#[test]
fn scots_all_have_positive_volume_and_value() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in &scots {
        assert!(s.volume > 0, "SCOT '{}' volume must be > 0", s.scot_name);
        assert!(
            s.monetary_value > Decimal::ZERO,
            "SCOT '{}' monetary_value must be > 0",
            s.scot_name
        );
    }
}

#[test]
fn scots_expected_scot_names_present() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let names: Vec<&str> = scots.iter().map(|s| s.scot_name.as_str()).collect();

    let expected_names = [
        "Revenue — Product Sales",
        "Purchases — Procurement",
        "Payroll",
        "Fixed Asset Additions",
        "Depreciation",
        "Tax Provision",
        "ECL / Bad Debt Provision",
        "Period-End Adjustments",
    ];

    for expected in &expected_names {
        assert!(
            names.contains(expected),
            "Expected SCOT '{}' not found. Present: {:?}",
            expected,
            names
        );
    }
}

#[test]
fn scots_tax_provision_is_complex_estimation() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let tax = scots.iter().find(|s| s.scot_name == "Tax Provision")
        .expect("Tax Provision SCOT should exist");

    assert_eq!(tax.significance_level, ScotSignificance::High);
    assert_eq!(tax.transaction_type, ScotTransactionType::Estimation);
    assert_eq!(tax.estimation_complexity, Some(EstimationComplexity::Complex));
    assert_eq!(tax.business_process, "R2R");
}

#[test]
fn scots_depreciation_is_simple_estimation() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let dep = scots.iter().find(|s| s.scot_name == "Depreciation")
        .expect("Depreciation SCOT should exist");

    assert_eq!(dep.transaction_type, ScotTransactionType::Estimation);
    assert_eq!(dep.estimation_complexity, Some(EstimationComplexity::Simple));
}

#[test]
fn scots_ecl_is_moderate_estimation() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let ecl = scots.iter().find(|s| s.scot_name == "ECL / Bad Debt Provision")
        .expect("ECL SCOT should exist");

    assert_eq!(ecl.transaction_type, ScotTransactionType::Estimation);
    assert_eq!(ecl.estimation_complexity, Some(EstimationComplexity::Moderate));
    assert_eq!(ecl.significance_level, ScotSignificance::High);
}

#[test]
fn scots_revenue_is_o2c_high_routine() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    let rev = scots.iter().find(|s| s.scot_name == "Revenue — Product Sales")
        .expect("Revenue SCOT should exist");

    assert_eq!(rev.business_process, "O2C");
    assert_eq!(rev.significance_level, ScotSignificance::High);
    assert_eq!(rev.transaction_type, ScotTransactionType::Routine);
}

#[test]
fn scots_multiple_entities_get_separate_sets() {
    let mut gen = ScotsGenerator::new(42);
    let scots_c001 = gen.generate_for_entity("C001", &[]);
    let scots_c002 = gen.generate_for_entity("C002", &[]);

    assert_eq!(scots_c001.len(), scots_c002.len(), "Both entities should get same number of SCOTs");

    // IDs should differ between entities
    let ids_c001: std::collections::HashSet<&str> = scots_c001.iter().map(|s| s.id.as_str()).collect();
    let ids_c002: std::collections::HashSet<&str> = scots_c002.iter().map(|s| s.id.as_str()).collect();
    let overlap: Vec<_> = ids_c001.intersection(&ids_c002).collect();
    assert!(overlap.is_empty(), "SCOT IDs should be unique across entities: {:?}", overlap);
}

#[test]
fn scots_relevant_assertions_are_non_empty() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in &scots {
        assert!(
            !s.relevant_assertions.is_empty(),
            "SCOT '{}' must have at least one relevant assertion",
            s.scot_name
        );
    }
}

#[test]
fn scots_related_account_areas_are_non_empty() {
    let mut gen = ScotsGenerator::new(42);
    let scots = gen.generate_for_entity("C001", &[]);

    for s in &scots {
        assert!(
            !s.related_account_areas.is_empty(),
            "SCOT '{}' must have at least one related account area",
            s.scot_name
        );
    }
}
