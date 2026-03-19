//! Integration tests for `ConsolidationGenerator`.

#![allow(clippy::unwrap_used)]

use datasynth_generators::ConsolidationGenerator;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Build a simple two-entity trial balance map.
fn two_entity_tbs() -> HashMap<String, HashMap<String, Decimal>> {
    let mut tbs = HashMap::new();

    let mut c001 = HashMap::new();
    c001.insert("Cash".to_string(), Decimal::from(100_000));
    c001.insert("Revenue".to_string(), Decimal::from(-500_000));
    c001.insert("Receivables".to_string(), Decimal::from(200_000));
    c001.insert("Payables".to_string(), Decimal::from(-80_000));
    tbs.insert("C001".to_string(), c001);

    let mut c002 = HashMap::new();
    c002.insert("Cash".to_string(), Decimal::from(50_000));
    c002.insert("Revenue".to_string(), Decimal::from(-300_000));
    c002.insert("Payables".to_string(), Decimal::from(-60_000));
    tbs.insert("C002".to_string(), c002);

    tbs
}

// ============================================================
// Test: pre_elimination_total equals sum of entity amounts
// ============================================================

#[test]
fn test_pre_elimination_equals_sum_of_entity_amounts() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    for li in &schedule.line_items {
        let entity_sum: Decimal = li.entity_amounts.values().copied().sum();
        assert_eq!(
            li.pre_elimination_total, entity_sum,
            "pre_elimination_total should equal sum of entity amounts for category '{}'",
            li.account_category
        );
    }
}

// ============================================================
// Test: post_elimination = pre_elimination + elimination_adjustments
// ============================================================

#[test]
fn test_post_elimination_invariant() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    for li in &schedule.line_items {
        assert_eq!(
            li.post_elimination_total,
            li.pre_elimination_total + li.elimination_adjustments,
            "post = pre + adj invariant violated for '{}'",
            li.account_category
        );
    }
}

// ============================================================
// Test: no eliminations → elimination_adjustments = 0 everywhere
// ============================================================

#[test]
fn test_no_eliminations_zero_adjustment() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    for li in &schedule.line_items {
        assert_eq!(
            li.elimination_adjustments,
            Decimal::ZERO,
            "expected zero elimination adjustments for '{}' when no elimination JEs supplied",
            li.account_category
        );
    }
}

// ============================================================
// Test: single-entity → consolidated equals standalone
// ============================================================

#[test]
fn test_single_entity_consolidated_equals_standalone() {
    let mut tbs = HashMap::new();
    let mut c001 = HashMap::new();
    c001.insert("Cash".to_string(), Decimal::from(100_000));
    c001.insert("Revenue".to_string(), Decimal::from(-500_000));
    c001.insert("Receivables".to_string(), Decimal::from(200_000));
    tbs.insert("C001".to_string(), c001.clone());

    let (items, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-01");

    assert_eq!(schedule.period, "2024-01");

    // Each consolidated line should equal the single entity amount
    for li in &schedule.line_items {
        let entity_amount = *li.entity_amounts.get("C001").unwrap();
        assert_eq!(
            li.post_elimination_total, entity_amount,
            "single-entity: consolidated '{}' should equal standalone",
            li.account_category
        );
    }

    // Consolidated FS line items should also be non-empty
    assert!(
        !items.is_empty(),
        "consolidated FS items should not be empty"
    );
}

// ============================================================
// Test: consolidated FS line items count matches schedule lines
// ============================================================

#[test]
fn test_consolidated_items_match_schedule_lines() {
    let tbs = two_entity_tbs();
    let (items, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    assert_eq!(
        items.len(),
        schedule.line_items.len(),
        "consolidated FS item count should match schedule line count"
    );
}

// ============================================================
// Test: Cash total is correct (100_000 + 50_000 = 150_000)
// ============================================================

#[test]
fn test_cash_pre_elimination_total() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    let cash_line = schedule
        .line_items
        .iter()
        .find(|li| li.account_category == "Cash")
        .expect("Cash category should be present");

    assert_eq!(cash_line.pre_elimination_total, Decimal::from(150_000));
    assert_eq!(
        cash_line.entity_amounts.get("C001").copied().unwrap(),
        Decimal::from(100_000)
    );
    assert_eq!(
        cash_line.entity_amounts.get("C002").copied().unwrap(),
        Decimal::from(50_000)
    );
}

// ============================================================
// Test: Revenue total is correct (–500_000 + –300_000 = –800_000)
// ============================================================

#[test]
fn test_revenue_pre_elimination_total() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

    let rev_line = schedule
        .line_items
        .iter()
        .find(|li| li.account_category == "Revenue")
        .expect("Revenue category should be present");

    assert_eq!(
        rev_line.pre_elimination_total,
        Decimal::from(-800_000),
        "Revenue pre-elimination should be -800_000"
    );
}

// ============================================================
// Test: period label stored in schedule
// ============================================================

#[test]
fn test_period_label_stored() {
    let tbs = two_entity_tbs();
    let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-Q1");
    assert_eq!(schedule.period, "2024-Q1");
}

// ============================================================
// Test: build_entity_trial_balances groups correctly
// ============================================================

#[test]
fn test_build_entity_trial_balances_empty() {
    let result = ConsolidationGenerator::build_entity_trial_balances(&[], false);
    assert!(result.is_empty(), "empty journal entries → empty result");
}
