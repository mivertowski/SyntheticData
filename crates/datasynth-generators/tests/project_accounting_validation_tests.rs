//! Integration tests for project accounting generation pipeline.
//!
//! Tests the full flow: project creation → WBS hierarchy → cost linking
//! → revenue recognition (PoC) → earned value metrics → change orders → milestones.

use chrono::NaiveDate;
use datasynth_config::schema::{
    ChangeOrderSchemaConfig, CostAllocationConfig, EarnedValueSchemaConfig, MilestoneSchemaConfig,
    ProjectAccountingConfig, ProjectRevenueRecognitionConfig,
};
use datasynth_core::models::{CostCategory, CostSourceType, MilestoneStatus, ProjectType};
use datasynth_generators::project_accounting::{
    ChangeOrderGenerator, EarnedValueGenerator, MilestoneGenerator, ProjectCostGenerator,
    ProjectGenerator, RevenueGenerator, SourceDocument,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("valid date")
}

/// Generate source documents (time entries) for testing.
fn generate_test_time_entries(count: usize) -> Vec<SourceDocument> {
    (0..count)
        .map(|i| {
            let day = (i % 28) as u32 + 1;
            let month = (i / 28) as u32 % 12 + 1;
            SourceDocument {
                id: format!("TE-{:04}", i + 1),
                entity_id: "TEST".to_string(),
                date: NaiveDate::from_ymd_opt(2024, month, day).unwrap_or(d("2024-01-15")),
                amount: dec!(750),
                source_type: CostSourceType::TimeEntry,
                hours: Some(dec!(8)),
            }
        })
        .collect()
}

/// Generate expense report source documents.
fn generate_test_expenses(count: usize) -> Vec<SourceDocument> {
    (0..count)
        .map(|i| {
            let day = (i % 28) as u32 + 1;
            let month = (i / 28) as u32 % 12 + 1;
            SourceDocument {
                id: format!("EXP-{:04}", i + 1),
                entity_id: "TEST".to_string(),
                date: NaiveDate::from_ymd_opt(2024, month, day).unwrap_or(d("2024-01-15")),
                amount: dec!(350),
                source_type: CostSourceType::ExpenseReport,
                hours: None,
            }
        })
        .collect()
}

// ===========================================================================
// Full Pipeline
// ===========================================================================

#[test]
fn test_project_accounting_full_pipeline() {
    let start_date = d("2024-01-01");
    let end_date = d("2024-06-30");

    // 1. Create projects with WBS
    let mut config = ProjectAccountingConfig::default();
    config.enabled = true;
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start_date, end_date);
    assert_eq!(pool.projects.len(), 5);

    // 2. Generate source documents
    let time_entries = generate_test_time_entries(100);
    let expenses = generate_test_expenses(30);

    // 3. Link to projects
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 0.60,
        expense_project_rate: 0.30,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);

    let mut all_docs = time_entries.clone();
    all_docs.extend(expenses.clone());
    let cost_lines = cost_gen.link_documents(&pool, &all_docs);
    assert!(!cost_lines.is_empty(), "Should produce cost lines");

    // 4. Recognize revenue for customer projects
    let customer_projects: Vec<_> = pool
        .projects
        .iter()
        .filter(|p| p.project_type == ProjectType::Customer)
        .collect();

    let contracts: Vec<_> = customer_projects
        .iter()
        .map(|p| {
            let contract_value = p.budget * dec!(1.25); // 25% margin
            (p.project_id.clone(), contract_value, p.budget)
        })
        .collect();

    let rev_config = ProjectRevenueRecognitionConfig::default();
    let mut rev_gen = RevenueGenerator::new(rev_config, 44);
    let _revenues = rev_gen.generate(
        &pool.projects,
        &cost_lines,
        &contracts,
        start_date,
        end_date,
    );

    // 5. Calculate earned value
    let evm_config = EarnedValueSchemaConfig::default();
    let mut evm_gen = EarnedValueGenerator::new(evm_config, 45);
    let evm_metrics = evm_gen.generate(&pool.projects, &cost_lines, start_date, end_date);

    // 6. Generate change orders
    let co_config = ChangeOrderSchemaConfig {
        enabled: true,
        probability: 0.80,
        max_per_project: 2,
        approval_rate: 0.75,
    };
    let mut co_gen = ChangeOrderGenerator::new(co_config, 46);
    let _change_orders = co_gen.generate(&pool.projects, start_date, end_date);

    // 7. Generate milestones
    let ms_config = MilestoneSchemaConfig {
        enabled: true,
        avg_per_project: 3,
        payment_milestone_rate: 0.50,
    };
    let mut ms_gen = MilestoneGenerator::new(ms_config, 47);
    let milestones = ms_gen.generate(&pool.projects, start_date, end_date, d("2024-03-31"));

    // VERIFY: All data was produced
    assert!(!cost_lines.is_empty(), "Cost lines should be generated");
    assert!(!evm_metrics.is_empty(), "EVM metrics should be generated");
    assert!(!milestones.is_empty(), "Milestones should be generated");
    assert_eq!(milestones.len(), 15, "5 projects * 3 milestones each");

    // VERIFY: Cost lines reference valid projects
    for cl in &cost_lines {
        assert!(
            pool.projects.iter().any(|p| p.project_id == cl.project_id),
            "Cost line {} references invalid project {}",
            cl.id,
            cl.project_id
        );
    }

    // VERIFY: EVM formulas
    for metric in &evm_metrics {
        let expected_sv = (metric.earned_value - metric.planned_value).round_dp(2);
        assert_eq!(metric.schedule_variance, expected_sv);
        let expected_cv = (metric.earned_value - metric.actual_cost).round_dp(2);
        assert_eq!(metric.cost_variance, expected_cv);
    }
}

// ===========================================================================
// Cost Linking
// ===========================================================================

#[test]
fn test_cost_linking_rates_match_config() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 10;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let time_entries = generate_test_time_entries(200);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 0.60,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    let linked_rate = cost_lines.len() as f64 / time_entries.len() as f64;
    assert!(
        linked_rate >= 0.40 && linked_rate <= 0.80,
        "Expected linking rate near 0.60, got {:.2}",
        linked_rate
    );
}

#[test]
fn test_cost_categories_match_source_types() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let mut docs = generate_test_time_entries(50);
    docs.extend(generate_test_expenses(50));
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        expense_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &docs);

    for cl in &cost_lines {
        match cl.source_type {
            CostSourceType::TimeEntry => assert_eq!(cl.cost_category, CostCategory::Labor),
            CostSourceType::ExpenseReport => assert_eq!(cl.cost_category, CostCategory::Travel),
            _ => {}
        }
    }
}

// ===========================================================================
// Revenue Recognition
// ===========================================================================

#[test]
fn test_revenue_increases_monotonically() {
    let start = d("2024-01-01");
    let end = d("2024-06-30");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 3;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    // Create cost lines spread over months
    let time_entries = generate_test_time_entries(100);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    let contracts: Vec<_> = pool
        .projects
        .iter()
        .map(|p| (p.project_id.clone(), p.budget * dec!(1.20), p.budget))
        .collect();

    let mut rev_gen = RevenueGenerator::new(ProjectRevenueRecognitionConfig::default(), 44);
    let revenues = rev_gen.generate(&pool.projects, &cost_lines, &contracts, start, end);

    // Check monotonicity per project
    for project in &pool.projects {
        let project_revenues: Vec<_> = revenues
            .iter()
            .filter(|r| r.project_id == project.project_id)
            .collect();

        let mut prev = dec!(0);
        for rev in &project_revenues {
            assert!(
                rev.cumulative_revenue >= prev,
                "Revenue should increase monotonically for {}: {} >= {}",
                project.project_id,
                rev.cumulative_revenue,
                prev
            );
            prev = rev.cumulative_revenue;
        }
    }
}

#[test]
fn test_unbilled_revenue_equals_recognized_minus_billed() {
    let start = d("2024-01-01");
    let end = d("2024-03-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 2;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let time_entries = generate_test_time_entries(50);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    let contracts: Vec<_> = pool
        .projects
        .iter()
        .map(|p| (p.project_id.clone(), p.budget * dec!(1.20), p.budget))
        .collect();

    let mut rev_gen = RevenueGenerator::new(ProjectRevenueRecognitionConfig::default(), 44);
    let revenues = rev_gen.generate(&pool.projects, &cost_lines, &contracts, start, end);

    for rev in &revenues {
        let expected_unbilled = (rev.cumulative_revenue - rev.billed_to_date).round_dp(2);
        assert_eq!(
            rev.unbilled_revenue, expected_unbilled,
            "Unbilled = recognized - billed"
        );
    }
}

// ===========================================================================
// Earned Value Management
// ===========================================================================

#[test]
fn test_evm_formulas_correct() {
    let start = d("2024-01-01");
    let end = d("2024-06-30");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 3;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let time_entries = generate_test_time_entries(100);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    let mut evm_gen = EarnedValueGenerator::new(EarnedValueSchemaConfig::default(), 45);
    let metrics = evm_gen.generate(&pool.projects, &cost_lines, start, end);

    for metric in &metrics {
        // SV = EV - PV
        let expected_sv = (metric.earned_value - metric.planned_value).round_dp(2);
        assert_eq!(metric.schedule_variance, expected_sv, "SV = EV - PV");

        // CV = EV - AC
        let expected_cv = (metric.earned_value - metric.actual_cost).round_dp(2);
        assert_eq!(metric.cost_variance, expected_cv, "CV = EV - AC");

        // SPI = EV / PV
        if metric.planned_value > Decimal::ZERO {
            let expected_spi = (metric.earned_value / metric.planned_value).round_dp(4);
            assert_eq!(metric.spi, expected_spi, "SPI = EV / PV");
        }

        // CPI = EV / AC
        if metric.actual_cost > Decimal::ZERO {
            let expected_cpi = (metric.earned_value / metric.actual_cost).round_dp(4);
            assert_eq!(metric.cpi, expected_cpi, "CPI = EV / AC");
        }
    }
}

// ===========================================================================
// Change Orders
// ===========================================================================

#[test]
fn test_change_order_impacts_positive() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 10;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let co_config = ChangeOrderSchemaConfig {
        enabled: true,
        probability: 1.0,
        max_per_project: 3,
        approval_rate: 0.75,
    };
    let mut co_gen = ChangeOrderGenerator::new(co_config, 46);
    let change_orders = co_gen.generate(&pool.projects, start, end);

    assert!(!change_orders.is_empty());

    for co in &change_orders {
        assert!(
            co.cost_impact > Decimal::ZERO,
            "Cost impact should be positive"
        );
        assert!(co.estimated_cost_impact > Decimal::ZERO);
        assert!(co.schedule_impact_days >= 0);
    }
}

// ===========================================================================
// Milestones
// ===========================================================================

#[test]
fn test_milestone_sequence_and_count() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let ms_config = MilestoneSchemaConfig {
        enabled: true,
        avg_per_project: 4,
        payment_milestone_rate: 0.50,
    };
    let mut ms_gen = MilestoneGenerator::new(ms_config, 47);
    let milestones = ms_gen.generate(&pool.projects, start, end, d("2024-06-30"));

    assert_eq!(milestones.len(), 20, "5 projects * 4 milestones");

    for project in &pool.projects {
        let project_ms: Vec<_> = milestones
            .iter()
            .filter(|m| m.project_id == project.project_id)
            .collect();
        assert_eq!(project_ms.len(), 4);

        // Sequences should be 1, 2, 3, 4
        for (i, ms) in project_ms.iter().enumerate() {
            assert_eq!(ms.sequence, (i + 1) as u32);
        }
    }
}

#[test]
fn test_past_milestones_have_final_status() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");
    let reference = d("2024-09-30");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let ms_config = MilestoneSchemaConfig::default();
    let mut ms_gen = MilestoneGenerator::new(ms_config, 47);
    let milestones = ms_gen.generate(&pool.projects, start, end, reference);

    let past_ms: Vec<_> = milestones
        .iter()
        .filter(|m| m.planned_date <= reference)
        .collect();

    for ms in &past_ms {
        assert!(
            ms.status == MilestoneStatus::Completed || ms.status == MilestoneStatus::Overdue,
            "Past milestone {:?} should be completed or overdue, got {:?}",
            ms.id,
            ms.status
        );
    }
}

// ===========================================================================
// Determinism
// ===========================================================================

#[test]
fn test_full_pipeline_determinism() {
    let start = d("2024-01-01");
    let end = d("2024-06-30");
    let time_entries = generate_test_time_entries(100);

    let run = |seed: u64| {
        let mut config = ProjectAccountingConfig::default();
        config.project_count = 5;
        let mut proj_gen = ProjectGenerator::new(config, seed);
        let pool = proj_gen.generate("TEST", start, end);

        let cost_config = CostAllocationConfig::default();
        let mut cost_gen = ProjectCostGenerator::new(cost_config, seed + 1);
        let cost_lines = cost_gen.link_documents(&pool, &time_entries);

        let mut evm_gen = EarnedValueGenerator::new(EarnedValueSchemaConfig::default(), seed + 2);
        let metrics = evm_gen.generate(&pool.projects, &cost_lines, start, end);

        (pool.projects.len(), cost_lines.len(), metrics.len())
    };

    let (p1, c1, m1) = run(42);
    let (p2, c2, m2) = run(42);

    assert_eq!(p1, p2);
    assert_eq!(c1, c2);
    assert_eq!(m1, m2);
}

#[test]
fn test_different_seeds_produce_different_results() {
    let start = d("2024-01-01");
    let end = d("2024-06-30");
    let time_entries = generate_test_time_entries(100);

    let run = |seed: u64| {
        let mut config = ProjectAccountingConfig::default();
        config.project_count = 10;
        let mut proj_gen = ProjectGenerator::new(config, seed);
        let pool = proj_gen.generate("TEST", start, end);

        let cost_config = CostAllocationConfig::default();
        let mut cost_gen = ProjectCostGenerator::new(cost_config, seed + 1);
        let cost_lines = cost_gen.link_documents(&pool, &time_entries);
        cost_lines.len()
    };

    let count1 = run(42);
    let count2 = run(99);

    // Different seeds should (very likely) produce different counts
    // This is probabilistic but with 100 documents the odds of identical counts are low
    assert_ne!(
        count1, count2,
        "Different seeds should produce different results"
    );
}

// ===========================================================================
// Cross-generator Consistency
// ===========================================================================

#[test]
fn test_cost_lines_reference_valid_wbs() {
    let start = d("2024-01-01");
    let end = d("2024-12-31");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let time_entries = generate_test_time_entries(100);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    // Every cost line's WBS ID should exist in its project
    for cl in &cost_lines {
        let project = pool.projects.iter().find(|p| p.project_id == cl.project_id);
        assert!(
            project.is_some(),
            "Cost line references invalid project {}",
            cl.project_id
        );

        let has_wbs = project
            .unwrap()
            .wbs_elements
            .iter()
            .any(|w| w.wbs_id == cl.wbs_id);
        assert!(
            has_wbs,
            "Cost line references invalid WBS {} in project {}",
            cl.wbs_id, cl.project_id
        );
    }
}

#[test]
fn test_evm_bac_equals_project_budget() {
    let start = d("2024-01-01");
    let end = d("2024-06-30");

    let mut config = ProjectAccountingConfig::default();
    config.project_count = 5;
    let mut proj_gen = ProjectGenerator::new(config, 42);
    let pool = proj_gen.generate("TEST", start, end);

    let time_entries = generate_test_time_entries(100);
    let cost_config = CostAllocationConfig {
        time_entry_project_rate: 1.0,
        ..Default::default()
    };
    let mut cost_gen = ProjectCostGenerator::new(cost_config, 43);
    let cost_lines = cost_gen.link_documents(&pool, &time_entries);

    let mut evm_gen = EarnedValueGenerator::new(EarnedValueSchemaConfig::default(), 45);
    let metrics = evm_gen.generate(&pool.projects, &cost_lines, start, end);

    for metric in &metrics {
        let project = pool
            .projects
            .iter()
            .find(|p| p.project_id == metric.project_id)
            .unwrap();
        assert_eq!(
            metric.bac, project.budget,
            "EVM BAC should equal project budget for {}",
            project.project_id
        );
    }
}
