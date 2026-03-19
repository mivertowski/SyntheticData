//! Combined integration tests for audit documentation generators:
//!
//! - Task 3.1: Engagement Letters (ISA 210) — `EngagementLetterGenerator`
//! - Task 3.2: Subsequent Events (ISA 560 / IAS 10) — `SubsequentEventGenerator`
//! - Task 3.3: Service Organization Controls (ISA 402) — `ServiceOrgGenerator`

use std::collections::HashSet;

use chrono::{Duration, NaiveDate};
use datasynth_core::models::audit::engagement_letter::{EngagementLetter, EngagementScope};
use datasynth_core::models::audit::subsequent_events::EventClassification;
use datasynth_core::models::audit::service_organization::SocReportType;
use datasynth_generators::audit::engagement_letter_generator::EngagementLetterGenerator;
use datasynth_generators::audit::subsequent_event_generator::SubsequentEventGenerator;
use datasynth_generators::audit::service_org_generator::ServiceOrgGenerator;

// =============================================================================
// Helpers
// =============================================================================

fn period_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
}

fn entity_codes(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("C{i:03}")).collect()
}

fn make_letter(gen: &mut EngagementLetterGenerator, entity_count: usize) -> EngagementLetter {
    gen.generate(
        "ENG-001",
        "Test Company Ltd.",
        entity_count,
        period_end(),
        "USD",
        "IFRS",
        period_end() - Duration::days(90),
    )
}

// =============================================================================
// Task 3.1: Engagement Letters (ISA 210)
// =============================================================================

#[test]
fn letter_single_entity_scope_is_statutory() {
    let mut gen = EngagementLetterGenerator::new(42);
    let letter = make_letter(&mut gen, 1);
    assert_eq!(
        letter.scope,
        EngagementScope::StatutoryAudit,
        "single entity should produce StatutoryAudit scope"
    );
}

#[test]
fn letter_multi_entity_scope_is_group() {
    let mut gen = EngagementLetterGenerator::new(42);
    let letter = make_letter(&mut gen, 3);
    assert_eq!(
        letter.scope,
        EngagementScope::GroupAudit,
        "3 entities should produce GroupAudit scope"
    );
}

#[test]
fn letter_one_per_engagement() {
    // Simulate 5 companies producing 5 engagements and 5 letters
    let mut gen = EngagementLetterGenerator::new(42);
    let period = period_end();

    let engagements: Vec<(String, String, NaiveDate, String)> = (1..=5)
        .map(|i| {
            (
                format!("ENG-{i:03}"),
                format!("Company {i}"),
                period,
                "USD".to_string(),
            )
        })
        .collect();

    let letters = gen.generate_batch(&engagements, 5, "IFRS");
    assert_eq!(
        letters.len(),
        5,
        "should produce one letter per engagement"
    );
}

#[test]
fn letter_fee_is_positive() {
    let mut gen = EngagementLetterGenerator::new(42);
    let letter = make_letter(&mut gen, 2);
    assert!(
        letter.fee_arrangement.amount > rust_decimal::Decimal::ZERO,
        "fee must be positive"
    );
}

#[test]
fn letter_fee_scales_with_entity_count() {
    let mut gen1 = EngagementLetterGenerator::new(42);
    let mut gen2 = EngagementLetterGenerator::new(42);
    let letter1 = make_letter(&mut gen1, 1);
    let letter5 = make_letter(&mut gen2, 5);
    assert!(
        letter5.fee_arrangement.amount > letter1.fee_arrangement.amount,
        "fee for 5 entities ({}) should exceed fee for 1 entity ({})",
        letter5.fee_arrangement.amount,
        letter1.fee_arrangement.amount
    );
}

#[test]
fn letter_reporting_deadline_is_after_period_end() {
    let mut gen = EngagementLetterGenerator::new(42);
    let letter = make_letter(&mut gen, 1);
    assert!(
        letter.reporting_deadline > period_end(),
        "reporting_deadline {} must be after period_end {}",
        letter.reporting_deadline,
        period_end()
    );
}

#[test]
fn letter_responsibilities_are_populated() {
    let mut gen = EngagementLetterGenerator::new(42);
    let letter = make_letter(&mut gen, 1);
    assert!(!letter.responsibilities_auditor.is_empty(), "auditor responsibilities must not be empty");
    assert!(!letter.responsibilities_management.is_empty(), "management responsibilities must not be empty");
}

#[test]
fn letter_ids_unique_across_batch() {
    let mut gen = EngagementLetterGenerator::new(42);
    let period = period_end();
    let engagements: Vec<(String, String, NaiveDate, String)> = (1..=10)
        .map(|i| (format!("ENG-{i:03}"), format!("Company {i}"), period, "USD".to_string()))
        .collect();
    let letters = gen.generate_batch(&engagements, 10, "IFRS");
    let ids: HashSet<&str> = letters.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids.len(), letters.len(), "all letter IDs must be unique");
}

// =============================================================================
// Task 3.2: Subsequent Events (ISA 560 / IAS 10)
// =============================================================================

#[test]
fn subsequent_events_count_within_bounds() {
    let mut gen = SubsequentEventGenerator::new(42);
    let events = gen.generate_for_entity("C001", period_end());
    assert!(
        events.len() <= 5,
        "count must be 0..=5, got {}",
        events.len()
    );
}

#[test]
fn subsequent_events_dates_after_period_end() {
    let pe = period_end();
    for seed in 0..20u64 {
        let mut gen = SubsequentEventGenerator::new(seed);
        let events = gen.generate_for_entity("C001", pe);
        for ev in &events {
            assert!(
                ev.event_date > pe,
                "event_date {} must be after period_end {}",
                ev.event_date,
                pe
            );
            assert!(
                ev.discovery_date >= ev.event_date,
                "discovery_date {} must be >= event_date {}",
                ev.discovery_date,
                ev.event_date
            );
        }
    }
}

#[test]
fn subsequent_events_adjusting_have_financial_impact() {
    // Every adjusting event must carry a financial_impact
    for seed in 0..50u64 {
        let mut gen = SubsequentEventGenerator::new(seed);
        let events = gen.generate_for_entity("C001", period_end());
        for ev in events
            .iter()
            .filter(|e| matches!(e.classification, EventClassification::Adjusting))
        {
            assert!(
                ev.financial_impact.is_some(),
                "adjusting event must have a financial_impact"
            );
        }
    }
}

#[test]
fn subsequent_events_approx_40_percent_adjusting() {
    let pe = period_end();
    let mut total = 0usize;
    let mut adjusting = 0usize;

    for seed in 0..300u64 {
        let mut gen = SubsequentEventGenerator::new(seed);
        let events = gen.generate_for_entity("C001", pe);
        total += events.len();
        adjusting += events
            .iter()
            .filter(|e| matches!(e.classification, EventClassification::Adjusting))
            .count();
    }

    if total > 0 {
        let ratio = adjusting as f64 / total as f64;
        // Allow wide tolerance: 20%–65%
        assert!(
            (0.20..=0.65).contains(&ratio),
            "adjusting ratio = {:.3}, expected ≈0.40 (20%-65% tolerance)",
            ratio
        );
    }
}

#[test]
fn subsequent_event_ids_unique() {
    let mut gen = SubsequentEventGenerator::new(7);
    let codes = entity_codes(5);
    let all_events = gen.generate_for_entities(&codes, period_end());
    let ids: HashSet<&str> = all_events.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        all_events.len(),
        "all subsequent event IDs must be unique"
    );
}

#[test]
fn subsequent_events_discovery_within_window() {
    let pe = period_end();
    for seed in 0..30u64 {
        let mut gen = SubsequentEventGenerator::new(seed);
        let events = gen.generate_for_entity("C001", pe);
        let max_window = pe + Duration::days(90);
        for ev in &events {
            assert!(
                ev.discovery_date <= max_window,
                "discovery_date {} exceeds max window {}",
                ev.discovery_date,
                max_window
            );
        }
    }
}

// =============================================================================
// Task 3.3: Service Organization Controls (ISA 402)
// =============================================================================

#[test]
fn service_orgs_count_in_range_per_entity() {
    let mut gen = ServiceOrgGenerator::new(42);
    // Use 1 entity; should produce 1-3 service orgs
    let snapshot = gen.generate(&entity_codes(1), period_end());
    assert!(
        (1..=3).contains(&snapshot.service_organizations.len()),
        "expected 1-3 service orgs for 1 entity, got {}",
        snapshot.service_organizations.len()
    );
}

#[test]
fn soc_reports_have_objectives_in_range() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(2), period_end());
    for report in &snapshot.soc_reports {
        assert!(
            (3..=8).contains(&report.control_objectives.len()),
            "control_objectives count must be 3-8, got {}",
            report.control_objectives.len()
        );
    }
}

#[test]
fn soc_reports_exceptions_max_2() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());
    for report in &snapshot.soc_reports {
        assert!(
            report.exceptions_noted.len() <= 2,
            "exceptions_noted must be 0-2, got {}",
            report.exceptions_noted.len()
        );
    }
}

#[test]
fn soc_reports_are_type2() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(2), period_end());
    for report in &snapshot.soc_reports {
        assert_eq!(
            report.report_type,
            SocReportType::Soc1Type2,
            "all SOC reports should be Type II"
        );
    }
}

#[test]
fn user_entity_controls_reference_valid_soc_reports() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());

    let report_ids: HashSet<&str> = snapshot.soc_reports.iter().map(|r| r.id.as_str()).collect();
    for ctrl in &snapshot.user_entity_controls {
        assert!(
            report_ids.contains(ctrl.soc_report_id.as_str()),
            "UserEntityControl references unknown soc_report_id '{}'",
            ctrl.soc_report_id
        );
    }
}

#[test]
fn service_org_ids_unique() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());
    let ids: HashSet<&str> = snapshot
        .service_organizations
        .iter()
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(ids.len(), snapshot.service_organizations.len(), "service org IDs must be unique");
}

#[test]
fn soc_report_ids_unique() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());
    let ids: HashSet<&str> = snapshot.soc_reports.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), snapshot.soc_reports.len(), "SOC report IDs must be unique");
}

#[test]
fn user_entity_control_ids_unique() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());
    let ids: HashSet<&str> = snapshot
        .user_entity_controls
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(ids.len(), snapshot.user_entity_controls.len(), "user entity control IDs must be unique");
}

#[test]
fn empty_entities_produces_empty_service_org_snapshot() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&[], period_end());
    assert!(snapshot.service_organizations.is_empty());
    assert!(snapshot.soc_reports.is_empty());
    assert!(snapshot.user_entity_controls.is_empty());
}

#[test]
fn exceptions_reference_valid_control_objectives() {
    let mut gen = ServiceOrgGenerator::new(42);
    let snapshot = gen.generate(&entity_codes(3), period_end());
    for report in &snapshot.soc_reports {
        let obj_ids: HashSet<&str> = report
            .control_objectives
            .iter()
            .map(|o| o.id.as_str())
            .collect();
        for exc in &report.exceptions_noted {
            assert!(
                obj_ids.contains(exc.control_objective_id.as_str()),
                "exception references unknown control_objective_id '{}'",
                exc.control_objective_id
            );
        }
    }
}
