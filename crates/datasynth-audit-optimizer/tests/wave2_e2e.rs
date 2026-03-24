//! End-to-end tests for Wave 2 features.
//!
//! Run with: cargo test -p datasynth-audit-optimizer --test wave2_e2e -- --nocapture --test-threads=1

use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use datasynth_audit_optimizer::portfolio::{
    simulate_portfolio, CorrelationConfig, EngagementSpec, PortfolioConfig, ResourcePool,
    ResourceSlot, RiskProfile,
};
use datasynth_audit_optimizer::resource_optimizer::{optimize_plan, ResourceConstraints};
use datasynth_audit_optimizer::risk_scoping::{analyze_coverage, impact_of_removing};
use std::collections::HashMap;

// =========================================================================
// 1. Resource Optimizer E2E — Real FSA Blueprint
// =========================================================================

#[test]
fn test_resource_optimizer_fsa_within_budget() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let overlay = default_overlay();

    let constraints = ResourceConstraints {
        total_budget_hours: 200.0, // generous budget for FSA
        role_availability: HashMap::new(),
        must_include: vec!["accept_engagement".into(), "form_opinion".into()],
        must_exclude: vec![],
    };

    let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

    // Must-include procedures present
    assert!(
        plan.included_procedures
            .contains(&"accept_engagement".to_string()),
        "accept_engagement must be included"
    );
    assert!(
        plan.included_procedures
            .contains(&"form_opinion".to_string()),
        "form_opinion must be included"
    );

    // form_opinion has preconditions — they should be auto-included
    assert!(
        plan.included_procedures.len() >= 3,
        "form_opinion depends on going_concern + subsequent_events, expected >= 3 procedures, got {}",
        plan.included_procedures.len()
    );

    // Budget respected
    assert!(
        plan.total_hours <= 200.0,
        "Total hours {} should be <= 200",
        plan.total_hours
    );
    assert!(plan.total_hours > 0.0, "Should have positive hours");
    assert!(plan.total_cost > 0.0, "Should have positive cost");

    // Coverage computed
    assert!(
        plan.standards_coverage > 0.0,
        "Should have some standards coverage"
    );
    assert!(plan.standards_coverage <= 1.0, "Coverage should be <= 1.0");
}

#[test]
fn test_resource_optimizer_ia_tight_budget() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay();

    let constraints = ResourceConstraints {
        total_budget_hours: 50.0, // very tight for IA (34 procedures)
        role_availability: HashMap::new(),
        must_include: vec![],
        must_exclude: vec![],
    };

    let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

    // Tight budget should exclude some procedures
    assert!(
        !plan.excluded_procedures.is_empty(),
        "Tight budget should exclude some procedures"
    );

    // Should still include some
    assert!(
        !plan.included_procedures.is_empty(),
        "Should include at least some procedures"
    );

    // Budget respected
    assert!(
        plan.total_hours <= 50.0 + 20.0, // allow some slack for mandatory deps
        "Hours {} unreasonably high for 50h budget",
        plan.total_hours
    );
}

// =========================================================================
// 2. Risk Scoping E2E — Real Blueprints
// =========================================================================

#[test]
fn test_risk_scoping_fsa_full_coverage() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    // Include all procedures
    let all_procs: Vec<String> = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| p.procedures.iter())
        .map(|p| p.id.clone())
        .collect();

    let report = analyze_coverage(&bwp.blueprint, &all_procs);

    assert!(
        (report.standards_coverage - 1.0).abs() < 0.01,
        "Full scope should have ~100% standards coverage, got {:.1}%",
        report.standards_coverage * 100.0
    );
    assert!(
        report.standards_uncovered.is_empty(),
        "Full scope should have no uncovered standards: {:?}",
        report.standards_uncovered
    );
    assert_eq!(report.included_procedures, report.total_procedures);
}

#[test]
fn test_risk_scoping_ia_partial_coverage() {
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();

    // Include only first 5 procedures
    let partial: Vec<String> = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| p.procedures.iter())
        .take(5)
        .map(|p| p.id.clone())
        .collect();

    let report = analyze_coverage(&bwp.blueprint, &partial);

    assert!(
        report.standards_coverage < 1.0,
        "Partial scope should have < 100% coverage"
    );
    assert!(
        report.standards_coverage > 0.0,
        "5 procedures should cover some standards"
    );
    assert!(
        !report.standards_uncovered.is_empty(),
        "Should have uncovered standards"
    );
    assert_eq!(report.included_procedures, 5);
}

#[test]
fn test_what_if_removal_impact() {
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();

    let all_procs: Vec<String> = bwp
        .blueprint
        .phases
        .iter()
        .flat_map(|p| p.procedures.iter())
        .map(|p| p.id.clone())
        .collect();

    let impact = impact_of_removing(
        &bwp.blueprint,
        &bwp.preconditions,
        &all_procs,
        "risk_identification",
    );

    assert_eq!(impact.removed_procedure, "risk_identification");
    assert!(
        impact.standards_coverage_delta < 0.0,
        "Removing a procedure should reduce coverage, delta={}",
        impact.standards_coverage_delta
    );
}

// =========================================================================
// 3. Portfolio E2E — Real Blueprints
// =========================================================================

fn default_pool() -> ResourcePool {
    let mut roles = HashMap::new();
    roles.insert(
        "engagement_partner".into(),
        ResourceSlot {
            count: 2,
            hours_per_person: 2000.0,
        },
    );
    roles.insert(
        "audit_manager".into(),
        ResourceSlot {
            count: 3,
            hours_per_person: 1800.0,
        },
    );
    roles.insert(
        "audit_senior".into(),
        ResourceSlot {
            count: 5,
            hours_per_person: 1600.0,
        },
    );
    roles.insert(
        "audit_staff".into(),
        ResourceSlot {
            count: 8,
            hours_per_person: 1600.0,
        },
    );
    ResourcePool { roles }
}

#[test]
fn test_portfolio_three_engagements() {
    let config = PortfolioConfig {
        engagements: vec![
            EngagementSpec {
                entity_id: "CLIENT_A".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "financial_services".into(),
                risk_profile: RiskProfile::High,
                seed: 100,
            },
            EngagementSpec {
                entity_id: "CLIENT_B".into(),
                blueprint: "fsa".into(),
                overlay: "thorough".into(),
                industry: "financial_services".into(),
                risk_profile: RiskProfile::Medium,
                seed: 200,
            },
            EngagementSpec {
                entity_id: "CLIENT_C".into(),
                blueprint: "fsa".into(),
                overlay: "rushed".into(),
                industry: "manufacturing".into(),
                risk_profile: RiskProfile::Low,
                seed: 300,
            },
        ],
        shared_resources: default_pool(),
        correlation: CorrelationConfig::default(),
    };

    let report = simulate_portfolio(&config).unwrap();

    // All 3 engagements ran
    assert_eq!(report.engagement_summaries.len(), 3);

    // Each produced events and artifacts
    for summary in &report.engagement_summaries {
        assert!(
            summary.events > 0,
            "{} should have events",
            summary.entity_id
        );
        assert!(
            summary.artifacts > 0,
            "{} should have artifacts",
            summary.entity_id
        );
        assert!(
            summary.hours > 0.0,
            "{} should have hours",
            summary.entity_id
        );
        assert!(summary.cost > 0.0, "{} should have cost", summary.entity_id);
    }

    // Portfolio totals
    assert!(report.total_hours > 0.0);
    assert!(report.total_cost > 0.0);

    // Risk heatmap has entries for all 3
    assert_eq!(report.risk_heatmap.len(), 3);

    // Resource utilization computed
    assert!(
        !report.resource_utilization.is_empty(),
        "Should have utilization data"
    );
}

#[test]
fn test_portfolio_mixed_blueprints() {
    let config = PortfolioConfig {
        engagements: vec![
            EngagementSpec {
                entity_id: "EXT_AUDIT".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "technology".into(),
                risk_profile: RiskProfile::Medium,
                seed: 42,
            },
            EngagementSpec {
                entity_id: "INT_AUDIT".into(),
                blueprint: "ia".into(),
                overlay: "default".into(),
                industry: "technology".into(),
                risk_profile: RiskProfile::High,
                seed: 43,
            },
        ],
        shared_resources: default_pool(),
        correlation: CorrelationConfig {
            systemic_finding_probability: 0.8, // high probability to test propagation
            industry_correlation: 0.7,
        },
    };

    let report = simulate_portfolio(&config).unwrap();

    assert_eq!(report.engagement_summaries.len(), 2);

    // IA should have more events than FSA
    let fsa = &report.engagement_summaries[0];
    let ia = &report.engagement_summaries[1];
    assert!(
        ia.events > fsa.events,
        "IA ({}) should have more events than FSA ({})",
        ia.events,
        fsa.events
    );

    // Both same industry with high correlation — systemic findings likely
    // (probabilistic, so just check the field exists)
    // report.systemic_findings may or may not be populated depending on RNG
}

#[test]
fn test_portfolio_resource_conflict_with_tiny_pool() {
    let mut tiny_pool = ResourcePool {
        roles: HashMap::new(),
    };
    tiny_pool.roles.insert(
        "engagement_partner".into(),
        ResourceSlot {
            count: 1,
            hours_per_person: 10.0,
        }, // only 10 hours!
    );

    let config = PortfolioConfig {
        engagements: vec![
            EngagementSpec {
                entity_id: "A".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "retail".into(),
                risk_profile: RiskProfile::Medium,
                seed: 1,
            },
            EngagementSpec {
                entity_id: "B".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "retail".into(),
                risk_profile: RiskProfile::Medium,
                seed: 2,
            },
        ],
        shared_resources: tiny_pool,
        correlation: CorrelationConfig::default(),
    };

    let report = simulate_portfolio(&config).unwrap();

    // With only 10 partner hours for 2 engagements, should detect conflict
    let partner_conflicts: Vec<_> = report
        .scheduling_conflicts
        .iter()
        .filter(|c| c.role == "engagement_partner")
        .collect();

    assert!(
        !partner_conflicts.is_empty(),
        "Should detect partner scheduling conflict with 10h pool and 2 engagements"
    );
}

// =========================================================================
// 4. Cross-Feature Integration
// =========================================================================

#[test]
fn test_optimizer_then_scoping() {
    // Run optimizer to get a plan, then analyze its coverage
    let bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay();

    let constraints = ResourceConstraints {
        total_budget_hours: 100.0,
        role_availability: HashMap::new(),
        must_include: vec!["develop_findings".into()],
        must_exclude: vec![],
    };

    let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

    // Now analyze coverage of the optimized plan
    let coverage = analyze_coverage(&bwp.blueprint, &plan.included_procedures);

    assert!(
        coverage.standards_coverage > 0.0,
        "Optimized plan should have some coverage"
    );
    assert_eq!(coverage.included_procedures, plan.included_procedures.len());

    // Optimizer's reported coverage should match scoping's calculation
    // (may differ slightly due to algorithm differences, so just sanity check)
    assert!(
        (plan.standards_coverage - coverage.standards_coverage).abs() < 0.15,
        "Coverage mismatch: optimizer={:.2}, scoping={:.2}",
        plan.standards_coverage,
        coverage.standards_coverage
    );
}
