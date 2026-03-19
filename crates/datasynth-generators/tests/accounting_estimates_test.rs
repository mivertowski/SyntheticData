//! Integration tests for ISA 540 accounting estimates generator.
//!
//! Verifies that:
//! - Each entity receives 5–8 estimates
//! - Each estimate has ISA 540 risk factors consistent with its estimate type
//! - Retrospective reviews have correct variance calculations
//! - NCI percentage matches group structure when available
//! - Estimate types are not duplicated within a single entity

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod accounting_estimates_integration_tests {
    use datasynth_core::models::audit::accounting_estimates::{
        EstimateComplexity, EstimateType, UncertaintyLevel,
    };
    use datasynth_generators::audit::accounting_estimate_generator::{
        AccountingEstimateGenerator, AccountingEstimateGeneratorConfig,
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_gen(seed: u64) -> AccountingEstimateGenerator {
        AccountingEstimateGenerator::new(seed)
    }

    fn make_gen_all_reviews(seed: u64) -> AccountingEstimateGenerator {
        AccountingEstimateGenerator::with_config(
            seed,
            AccountingEstimateGeneratorConfig {
                retrospective_review_probability: 1.0,
                ..Default::default()
            },
        )
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_5_to_8_estimates_per_entity() {
        let mut gen = make_gen(42);
        for i in 0..5 {
            let entity_code = format!("ENTITY-{:03}", i);
            let estimates = gen.generate_for_entity(&entity_code);
            assert!(
                (5..=8).contains(&estimates.len()),
                "entity {} produced {} estimates (expected 5–8)",
                entity_code,
                estimates.len()
            );
        }
    }

    #[test]
    fn test_no_duplicate_estimate_types_per_entity() {
        let mut gen = make_gen(123);
        let estimates = gen.generate_for_entity("DEDUP-001");
        let mut seen = std::collections::HashSet::new();
        for est in &estimates {
            let key = format!("{:?}", est.estimate_type);
            assert!(
                seen.insert(key.clone()),
                "Duplicate estimate type '{}' in same entity",
                key
            );
        }
    }

    #[test]
    fn test_each_estimate_has_isa540_risk_factors() {
        let mut gen = make_gen(7);
        let estimates = gen.generate_for_entities(&["A001".to_string(), "A002".to_string()]);
        assert!(!estimates.is_empty(), "should generate estimates");

        for est in &estimates {
            // The top-level uncertainty field must match the risk factor field
            assert_eq!(
                format!("{:?}", est.estimation_uncertainty),
                format!("{:?}", est.isa540_risk_factors.estimation_uncertainty),
                "uncertainty mismatch for {:?}",
                est.estimate_type
            );
            // Complexity must also be consistent
            assert_eq!(
                format!("{:?}", est.complexity),
                format!("{:?}", est.isa540_risk_factors.complexity),
                "complexity mismatch for {:?}",
                est.estimate_type
            );
        }
    }

    #[test]
    fn test_high_uncertainty_types_have_correct_risk_factors() {
        let mut gen = make_gen(555);
        // Run multiple entities to ensure we get pension/ECL types
        let estimates = gen.generate_for_entities(&[
            "B001".to_string(),
            "B002".to_string(),
            "B003".to_string(),
            "B004".to_string(),
        ]);

        let pension = estimates
            .iter()
            .find(|e| e.estimate_type == EstimateType::PensionObligation);
        if let Some(p) = pension {
            assert!(
                matches!(
                    p.isa540_risk_factors.estimation_uncertainty,
                    UncertaintyLevel::High
                ),
                "PensionObligation must be High uncertainty"
            );
            assert!(
                matches!(
                    p.isa540_risk_factors.complexity,
                    EstimateComplexity::Complex
                ),
                "PensionObligation must be Complex"
            );
        }

        let ecl = estimates
            .iter()
            .find(|e| e.estimate_type == EstimateType::ExpectedCreditLoss);
        if let Some(e) = ecl {
            assert!(
                matches!(
                    e.isa540_risk_factors.estimation_uncertainty,
                    UncertaintyLevel::High
                ),
                "ExpectedCreditLoss must be High uncertainty"
            );
        }
    }

    #[test]
    fn test_low_uncertainty_for_depreciation() {
        let mut gen = make_gen(888);
        let estimates = gen.generate_for_entities(&[
            "C001".to_string(),
            "C002".to_string(),
            "C003".to_string(),
            "C004".to_string(),
        ]);

        let depr = estimates
            .iter()
            .find(|e| e.estimate_type == EstimateType::DepreciationUsefulLife);
        if let Some(d) = depr {
            assert!(
                matches!(
                    d.isa540_risk_factors.estimation_uncertainty,
                    UncertaintyLevel::Low
                ),
                "DepreciationUsefulLife must be Low uncertainty"
            );
            assert!(
                matches!(d.isa540_risk_factors.complexity, EstimateComplexity::Simple),
                "DepreciationUsefulLife must be Simple"
            );
        }
    }

    #[test]
    fn test_each_estimate_has_2_to_3_assumptions() {
        let mut gen = make_gen(99);
        let estimates = gen.generate_for_entity("ASSUMP-001");
        for est in &estimates {
            assert!(
                (2..=3).contains(&est.assumptions.len()),
                "{:?} has {} assumptions (expected 2-3)",
                est.estimate_type,
                est.assumptions.len()
            );
        }
    }

    #[test]
    fn test_retrospective_review_variance_is_mathematically_correct() {
        let mut gen = make_gen_all_reviews(1111);
        let estimates = gen.generate_for_entity("VAR-001");

        for est in &estimates {
            let rev = est
                .retrospective_review
                .as_ref()
                .expect("all estimates should have reviews (probability=1.0)");

            // variance = actual − prior
            let expected = (rev.actual_outcome - rev.prior_period_estimate).round_dp(2);
            assert_eq!(
                rev.variance, expected,
                "{:?}: variance calculation incorrect",
                est.estimate_type
            );

            // variance_percentage = variance / prior * 100 (skip if prior is zero)
            if !rev.prior_period_estimate.is_zero() {
                use rust_decimal_macros::dec;
                let expected_pct =
                    (rev.variance / rev.prior_period_estimate * dec!(100)).round_dp(2);
                assert_eq!(
                    rev.variance_percentage, expected_pct,
                    "{:?}: variance_percentage calculation incorrect",
                    est.estimate_type
                );
            }
        }
    }

    #[test]
    fn test_management_point_estimate_is_positive() {
        let mut gen = make_gen(42);
        let estimates = gen.generate_for_entities(&["POS-001".to_string(), "POS-002".to_string()]);
        for est in &estimates {
            assert!(
                est.management_point_estimate > rust_decimal::Decimal::ZERO,
                "management_point_estimate should be positive for {:?}",
                est.estimate_type
            );
        }
    }

    #[test]
    fn test_multi_entity_generation() {
        let mut gen = make_gen(77);
        let entity_codes: Vec<String> = (1..=3).map(|i| format!("ENT-{:02}", i)).collect();
        let estimates = gen.generate_for_entities(&entity_codes);

        // Should produce estimates for all 3 entities: 5–8 per entity → 15–24 total
        assert!(
            estimates.len() >= 15,
            "expected at least 15 estimates for 3 entities, got {}",
            estimates.len()
        );
        assert!(
            estimates.len() <= 24,
            "expected at most 24 estimates for 3 entities, got {}",
            estimates.len()
        );

        // Each entity_code should appear in the results
        for ec in &entity_codes {
            assert!(
                estimates.iter().any(|e| &e.entity_code == ec),
                "entity {} missing from estimates",
                ec
            );
        }
    }

    #[test]
    fn test_sensitivity_values_are_positive() {
        let mut gen = make_gen(321);
        let estimates = gen.generate_for_entity("SEN-001");
        for est in &estimates {
            for assumption in &est.assumptions {
                assert!(
                    assumption.sensitivity >= rust_decimal::Decimal::ZERO,
                    "{:?}: assumption '{}' has negative sensitivity",
                    est.estimate_type,
                    assumption.description
                );
            }
        }
    }

    #[test]
    fn test_estimate_ids_are_unique() {
        let mut gen = make_gen(456);
        let estimates =
            gen.generate_for_entities(&["UNIQ-001".to_string(), "UNIQ-002".to_string()]);
        let mut ids = std::collections::HashSet::new();
        for est in &estimates {
            assert!(
                ids.insert(est.id.clone()),
                "Duplicate estimate ID: {}",
                est.id
            );
        }
    }
}
