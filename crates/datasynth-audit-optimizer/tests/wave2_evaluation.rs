//! Wave 2 evaluation — prints detailed results for human inspection.
//!
//! Run with: cargo test -p datasynth-audit-optimizer --test wave2_evaluation -- --nocapture --test-threads=1

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use datasynth_audit_optimizer::portfolio::{
    simulate_portfolio, CorrelationConfig, EngagementSpec, PortfolioConfig, ResourcePool,
    ResourceSlot, RiskProfile,
};
use datasynth_audit_optimizer::resource_optimizer::{optimize_plan, ResourceConstraints};
use datasynth_audit_optimizer::risk_scoping::{analyze_coverage, impact_of_removing};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

#[test]
fn evaluate_wave2_features() {
    println!("\n{}", "=".repeat(70));
    println!("  Wave 2 Evaluation: Audit Planning Optimization");
    println!("{}\n", "=".repeat(70));

    // ---------------------------------------------------------------
    // 1. Iteration Limit Improvement
    // ---------------------------------------------------------------
    println!("--- 1. Iteration Limit Improvement ---\n");

    let bwp_ia = BlueprintWithPreconditions::load_builtin_ia().unwrap();
    let overlay = default_overlay();
    let mut engine = AuditFsmEngine::new(
        bwp_ia.clone(),
        overlay.clone(),
        ChaCha8Rng::seed_from_u64(42),
    );
    let ctx = EngagementContext::test_default();
    let result = engine.run_engagement(&ctx).unwrap();

    let completed = result
        .procedure_states
        .values()
        .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
        .count();
    let total = result.procedure_states.len();

    println!(
        "  IA procedures completed: {}/{} ({:.0}%)",
        completed,
        total,
        completed as f64 / total as f64 * 100.0
    );
    println!("  IA events: {}", result.event_log.len());
    println!("  IA artifacts: {}", result.artifacts.total_artifacts());
    println!("  IA duration: {:.1}h", result.total_duration_hours);

    let incomplete: Vec<_> = result
        .procedure_states
        .iter()
        .filter(|(_, s)| s.as_str() != "completed" && s.as_str() != "closed")
        .collect();
    if !incomplete.is_empty() {
        println!("  Incomplete procedures:");
        for (id, state) in &incomplete {
            println!("    {} → {}", id, state);
        }
    }
    println!();

    // ---------------------------------------------------------------
    // 2. Cost Model
    // ---------------------------------------------------------------
    println!("--- 2. Cost Model ---\n");

    let bwp_fsa = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    let mut total_fsa_hours = 0.0;
    let mut total_fsa_cost = 0.0;
    println!("  FSA procedure costs (default overlay):");
    println!(
        "  {:30} {:>8} {:>8} {:>10}",
        "Procedure", "Hours", "Role", "Cost"
    );
    for phase in &bwp_fsa.blueprint.phases {
        for proc in &phase.procedures {
            let hours = overlay.resource_costs.effective_hours(proc);
            let cost = overlay.resource_costs.procedure_cost(proc);
            let role = proc
                .required_roles
                .first()
                .map(|r| r.as_str())
                .unwrap_or("staff");
            total_fsa_hours += hours;
            total_fsa_cost += cost;
            println!("  {:30} {:>8.1} {:>8} {:>10.0}", proc.id, hours, role, cost);
        }
    }
    println!(
        "  {:30} {:>8.1} {:>8} {:>10.0}",
        "TOTAL", total_fsa_hours, "", total_fsa_cost
    );
    println!();

    // ---------------------------------------------------------------
    // 3. Resource-Constrained Optimization
    // ---------------------------------------------------------------
    println!("--- 3. Resource-Constrained Optimization ---\n");

    // FSA with generous budget
    let plan_full = optimize_plan(
        &bwp_fsa.blueprint,
        &overlay,
        &bwp_fsa.preconditions,
        &ResourceConstraints {
            total_budget_hours: 500.0,
            role_availability: HashMap::new(),
            must_include: vec![],
            must_exclude: vec![],
        },
    );
    println!(
        "  FSA (500h budget): {}/{} procedures, {:.1}h, ${:.0}, {:.0}% standards coverage",
        plan_full.included_procedures.len(),
        plan_full.included_procedures.len() + plan_full.excluded_procedures.len(),
        plan_full.total_hours,
        plan_full.total_cost,
        plan_full.standards_coverage * 100.0
    );

    // FSA with tight budget
    let plan_tight = optimize_plan(
        &bwp_fsa.blueprint,
        &overlay,
        &bwp_fsa.preconditions,
        &ResourceConstraints {
            total_budget_hours: 40.0,
            role_availability: HashMap::new(),
            must_include: vec!["form_opinion".into()],
            must_exclude: vec![],
        },
    );
    println!(
        "  FSA (40h, must: form_opinion): {}/{} procedures, {:.1}h, ${:.0}, {:.0}% standards",
        plan_tight.included_procedures.len(),
        plan_tight.included_procedures.len() + plan_tight.excluded_procedures.len(),
        plan_tight.total_hours,
        plan_tight.total_cost,
        plan_tight.standards_coverage * 100.0
    );
    println!("    Included: {:?}", plan_tight.included_procedures);
    println!("    Excluded: {:?}", plan_tight.excluded_procedures);

    // IA with moderate budget
    let plan_ia = optimize_plan(
        &bwp_ia.blueprint,
        &overlay,
        &bwp_ia.preconditions,
        &ResourceConstraints {
            total_budget_hours: 200.0,
            role_availability: HashMap::new(),
            must_include: vec![],
            must_exclude: vec![],
        },
    );
    println!(
        "  IA (200h budget): {}/{} procedures, {:.1}h, ${:.0}, {:.0}% standards",
        plan_ia.included_procedures.len(),
        plan_ia.included_procedures.len() + plan_ia.excluded_procedures.len(),
        plan_ia.total_hours,
        plan_ia.total_cost,
        plan_ia.standards_coverage * 100.0
    );
    println!();

    // ---------------------------------------------------------------
    // 4. Risk-Based Scoping
    // ---------------------------------------------------------------
    println!("--- 4. Risk-Based Scoping ---\n");

    let all_fsa_procs: Vec<String> = bwp_fsa
        .blueprint
        .phases
        .iter()
        .flat_map(|p| p.procedures.iter())
        .map(|p| p.id.clone())
        .collect();

    let full_coverage = analyze_coverage(&bwp_fsa.blueprint, &all_fsa_procs);
    println!(
        "  FSA full scope: {:.0}% standards ({}/{}), {} procedures",
        full_coverage.standards_coverage * 100.0,
        full_coverage.standards_covered.len(),
        full_coverage.standards_covered.len() + full_coverage.standards_uncovered.len(),
        full_coverage.included_procedures
    );

    // What-if: remove substantive_testing
    let impact = impact_of_removing(
        &bwp_fsa.blueprint,
        &bwp_fsa.preconditions,
        &all_fsa_procs,
        "substantive_testing",
    );
    println!("  What-if remove substantive_testing:");
    println!("    Standards lost: {:?}", impact.standards_lost);
    println!(
        "    Coverage delta: {:.1}%",
        impact.standards_coverage_delta * 100.0
    );
    println!(
        "    Dependent procedures: {:?}",
        impact.dependent_procedures_affected
    );
    println!();

    // ---------------------------------------------------------------
    // 5. Portfolio Simulation
    // ---------------------------------------------------------------
    println!("--- 5. Portfolio Simulation ---\n");

    let mut pool_roles = HashMap::new();
    pool_roles.insert(
        "engagement_partner".into(),
        ResourceSlot {
            count: 2,
            hours_per_person: 2000.0,
            unavailable_periods: vec![],
        },
    );
    pool_roles.insert(
        "audit_manager".into(),
        ResourceSlot {
            count: 4,
            hours_per_person: 1800.0,
            unavailable_periods: vec![],
        },
    );
    pool_roles.insert(
        "audit_senior".into(),
        ResourceSlot {
            count: 6,
            hours_per_person: 1600.0,
            unavailable_periods: vec![],
        },
    );
    pool_roles.insert(
        "audit_staff".into(),
        ResourceSlot {
            count: 10,
            hours_per_person: 1600.0,
            unavailable_periods: vec![],
        },
    );

    let portfolio_config = PortfolioConfig {
        engagements: vec![
            EngagementSpec {
                entity_id: "BANK_A".into(),
                blueprint: "fsa".into(),
                overlay: "thorough".into(),
                industry: "financial_services".into(),
                risk_profile: RiskProfile::High,
                seed: 100,
            },
            EngagementSpec {
                entity_id: "BANK_B".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "financial_services".into(),
                risk_profile: RiskProfile::Medium,
                seed: 200,
            },
            EngagementSpec {
                entity_id: "MFGCO".into(),
                blueprint: "fsa".into(),
                overlay: "default".into(),
                industry: "manufacturing".into(),
                risk_profile: RiskProfile::Medium,
                seed: 300,
            },
            EngagementSpec {
                entity_id: "TECHCO".into(),
                blueprint: "ia".into(),
                overlay: "default".into(),
                industry: "technology".into(),
                risk_profile: RiskProfile::High,
                seed: 400,
            },
        ],
        shared_resources: ResourcePool { roles: pool_roles },
        correlation: CorrelationConfig {
            systemic_finding_probability: 0.5,
            industry_correlation: 0.6,
        },
    };

    let report = simulate_portfolio(&portfolio_config).unwrap();

    println!(
        "  {:12} {:>8} {:>10} {:>10} {:>10} {:>8}",
        "Entity", "Events", "Artifacts", "Hours", "Cost", "Compl%"
    );
    for s in &report.engagement_summaries {
        println!(
            "  {:12} {:>8} {:>10} {:>10.1} {:>10.0} {:>7.0}%",
            s.entity_id,
            s.events,
            s.artifacts,
            s.hours,
            s.cost,
            s.completion_rate * 100.0
        );
    }
    println!(
        "  {:12} {:>8} {:>10} {:>10.1} {:>10.0}",
        "TOTAL", "", "", report.total_hours, report.total_cost
    );

    println!("\n  Resource utilization:");
    let mut util: Vec<_> = report.resource_utilization.iter().collect();
    util.sort_by_key(|(k, _)| (*k).clone());
    for (role, pct) in &util {
        println!("    {:25} {:.0}%", role, *pct * 100.0);
    }

    if !report.scheduling_conflicts.is_empty() {
        println!("\n  Scheduling conflicts:");
        for c in &report.scheduling_conflicts {
            println!(
                "    {} — need {:.0}h, have {:.0}h",
                c.role, c.required_hours, c.available_hours
            );
        }
    } else {
        println!("\n  No scheduling conflicts.");
    }

    if !report.systemic_findings.is_empty() {
        println!("\n  Systemic findings:");
        for f in &report.systemic_findings {
            println!(
                "    {} in {} — affects {:?}",
                f.finding_type, f.industry, f.affected_entities
            );
        }
    }

    println!("\n  Risk heatmap:");
    for entry in &report.risk_heatmap {
        let bar = "#".repeat((entry.score * 20.0) as usize);
        println!(
            "    {:12} {:20} {:.1} {}",
            entry.entity_id, entry.category, entry.score, bar
        );
    }

    println!("\n--- All evaluations complete ---\n");
}
