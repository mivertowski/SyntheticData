//! Integration tests for the ASC 718 / IFRS 2 stock-based compensation generator.

use chrono::NaiveDate;
use datasynth_generators::stock_comp_generator::{StockCompGenerator, StockCompSnapshot};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

fn make_employee_ids(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("EMP{i:04}")).collect()
}

/// Run the generator with a fixed seed and 20 employees.
fn make_snapshot() -> StockCompSnapshot {
    let employees = make_employee_ids(20);
    let mut gen = StockCompGenerator::new(42);
    gen.generate(
        "1000",
        &employees,
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"),
        "USD",
    )
}

// ---------------------------------------------------------------------------
// Grant count invariants
// ---------------------------------------------------------------------------

#[test]
fn grants_generated_for_executive_subset() {
    let snap = make_snapshot();
    // 10% of 20 employees = 2 grantees (ceiling)
    assert_eq!(
        snap.grants.len(),
        2,
        "expected 2 grants (10% of 20 employees), got {}",
        snap.grants.len()
    );
}

#[test]
fn grant_count_scales_with_employee_count() {
    let employees_small = make_employee_ids(10);
    let employees_large = make_employee_ids(100);

    let mut gen = StockCompGenerator::new(7);
    let snap_small = gen.generate(
        "1000",
        &employees_small,
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid"),
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid"),
        "USD",
    );

    let mut gen2 = StockCompGenerator::new(7);
    let snap_large = gen2.generate(
        "1000",
        &employees_large,
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid"),
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid"),
        "USD",
    );

    assert!(
        snap_large.grants.len() > snap_small.grants.len(),
        "larger employee pool should yield more grants: small={}, large={}",
        snap_small.grants.len(),
        snap_large.grants.len()
    );
}

#[test]
fn empty_employee_list_produces_empty_snapshot() {
    let mut gen = StockCompGenerator::new(99);
    let snap = gen.generate(
        "1000",
        &[],
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid"),
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid"),
        "USD",
    );
    assert!(snap.grants.is_empty(), "no grants for empty employee list");
    assert!(
        snap.expenses.is_empty(),
        "no expenses for empty employee list"
    );
    assert!(
        snap.journal_entries.is_empty(),
        "no JEs for empty employee list"
    );
}

// ---------------------------------------------------------------------------
// Vesting schedule invariants
// ---------------------------------------------------------------------------

#[test]
fn vesting_percentages_sum_to_one() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        let total: Decimal = grant
            .vesting_schedule
            .vesting_entries
            .iter()
            .map(|e| e.percentage)
            .sum();
        assert_eq!(
            total,
            Decimal::ONE,
            "vesting percentages for grant '{}' sum to {} (expected 1.0000)",
            grant.id,
            total
        );
    }
}

#[test]
fn vesting_cumulative_matches_running_sum() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        let mut running = Decimal::ZERO;
        for entry in &grant.vesting_schedule.vesting_entries {
            running = (running + entry.percentage).round_dp(4);
            assert_eq!(
                entry.cumulative_percentage, running,
                "cumulative mismatch at period {} for grant '{}'",
                entry.period, grant.id
            );
        }
    }
}

#[test]
fn final_vesting_entry_cumulative_is_one() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        let last = grant
            .vesting_schedule
            .vesting_entries
            .last()
            .expect("non-empty vesting entries");
        assert_eq!(
            last.cumulative_percentage,
            Decimal::ONE,
            "final cumulative_percentage for grant '{}' should be 1.0000, got {}",
            grant.id,
            last.cumulative_percentage
        );
    }
}

#[test]
fn vesting_period_count_matches_total_periods() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        assert_eq!(
            grant.vesting_schedule.vesting_entries.len() as u32,
            grant.vesting_schedule.total_periods,
            "entry count does not match total_periods for grant '{}'",
            grant.id
        );
    }
}

// ---------------------------------------------------------------------------
// Expense record invariants
// ---------------------------------------------------------------------------

#[test]
fn expense_cumulative_plus_remaining_equals_total_after_forfeiture() {
    let snap = make_snapshot();
    // Collect the last (most recent) expense record per grant
    for grant in &snap.grants {
        let grant_expenses: Vec<_> = snap
            .expenses
            .iter()
            .filter(|e| e.grant_id == grant.id)
            .collect();

        if grant_expenses.is_empty() {
            // Grant may have no vested tranches in the reporting period — ok.
            continue;
        }

        // The final expense record should have remaining_unrecognized ≥ 0
        let last_expense = grant_expenses.last().unwrap();
        assert!(
            last_expense.remaining_unrecognized >= Decimal::ZERO,
            "remaining_unrecognized must be non-negative for grant '{}', got {}",
            grant.id,
            last_expense.remaining_unrecognized
        );

        // cumulative + remaining ≈ total_grant_value * (1 - forfeiture_rate)
        let expected_total =
            (grant.total_grant_value * (Decimal::ONE - grant.forfeiture_rate)).round_dp(2);
        let actual_sum =
            (last_expense.cumulative_recognized + last_expense.remaining_unrecognized).round_dp(2);
        assert_eq!(
            actual_sum,
            expected_total,
            "cumulative + remaining ({actual_sum}) ≠ total_after_forfeiture ({expected_total}) for grant '{}'",
            grant.id
        );
    }
}

#[test]
fn expense_amounts_are_positive() {
    let snap = make_snapshot();
    for exp in &snap.expenses {
        assert!(
            exp.expense_amount > Decimal::ZERO,
            "expense_amount must be positive for grant '{}', got {}",
            exp.grant_id,
            exp.expense_amount
        );
    }
}

// ---------------------------------------------------------------------------
// Journal entry invariants
// ---------------------------------------------------------------------------

#[test]
fn journal_entries_are_balanced() {
    let snap = make_snapshot();
    assert!(
        !snap.journal_entries.is_empty(),
        "should generate at least one journal entry"
    );
    for je in &snap.journal_entries {
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debit, total_credit,
            "Journal entry '{}' is not balanced: DR={} CR={}",
            je.header.document_id, total_debit, total_credit
        );
    }
}

#[test]
fn journal_entries_have_two_lines() {
    let snap = make_snapshot();
    for je in &snap.journal_entries {
        assert_eq!(
            je.lines.len(),
            2,
            "Each stock comp JE should have exactly 2 lines, got {} for '{}'",
            je.lines.len(),
            je.header.document_id
        );
    }
}

#[test]
fn journal_entry_gl_accounts_are_correct() {
    let snap = make_snapshot();
    for je in &snap.journal_entries {
        // Line 1: DR Compensation Expense (7200)
        // Line 2: CR APIC-Stock Comp (3150)
        let dr_line = je.lines.iter().find(|l| l.debit_amount > Decimal::ZERO);
        let cr_line = je.lines.iter().find(|l| l.credit_amount > Decimal::ZERO);

        assert!(
            dr_line.is_some(),
            "JE '{}' has no debit line",
            je.header.document_id
        );
        assert!(
            cr_line.is_some(),
            "JE '{}' has no credit line",
            je.header.document_id
        );

        let dr = dr_line.unwrap();
        let cr = cr_line.unwrap();

        assert_eq!(
            dr.gl_account, "7200",
            "Debit should be to account 7200 (comp expense), got '{}' for JE '{}'",
            dr.gl_account, je.header.document_id
        );
        assert_eq!(
            cr.gl_account, "3150",
            "Credit should be to account 3150 (APIC-stock comp), got '{}' for JE '{}'",
            cr.gl_account, je.header.document_id
        );
    }
}

// ---------------------------------------------------------------------------
// Grant value invariants
// ---------------------------------------------------------------------------

#[test]
fn total_grant_value_equals_quantity_times_fair_value() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        let expected = (Decimal::from(grant.quantity) * grant.fair_value_at_grant).round_dp(2);
        assert_eq!(
            grant.total_grant_value, expected,
            "total_grant_value mismatch for grant '{}': {} ≠ {}",
            grant.id, grant.total_grant_value, expected
        );
    }
}

#[test]
fn fair_value_is_positive() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        assert!(
            grant.fair_value_at_grant > Decimal::ZERO,
            "fair_value_at_grant must be positive for grant '{}', got {}",
            grant.id,
            grant.fair_value_at_grant
        );
    }
}

#[test]
fn forfeiture_rate_within_expected_range() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        assert!(
            grant.forfeiture_rate >= dec!(0.05) && grant.forfeiture_rate <= dec!(0.15),
            "forfeiture_rate out of [0.05, 0.15] for grant '{}': {}",
            grant.id,
            grant.forfeiture_rate
        );
    }
}

#[test]
fn options_have_exercise_price_rsus_do_not() {
    let snap = make_snapshot();
    for grant in &snap.grants {
        match grant.instrument_type {
            datasynth_core::models::stock_compensation::InstrumentType::Options => {
                assert!(
                    grant.exercise_price.is_some(),
                    "Options grant '{}' must have an exercise price",
                    grant.id
                );
            }
            _ => {
                assert!(
                    grant.exercise_price.is_none(),
                    "Non-option grant '{}' should not have an exercise price",
                    grant.id
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn generator_is_deterministic() {
    let snap1 = make_snapshot();
    let snap2 = make_snapshot();

    assert_eq!(
        snap1.grants.len(),
        snap2.grants.len(),
        "grant count should be deterministic"
    );
    if let (Some(g1), Some(g2)) = (snap1.grants.first(), snap2.grants.first()) {
        assert_eq!(
            g1.total_grant_value, g2.total_grant_value,
            "total_grant_value should be deterministic"
        );
        assert_eq!(
            g1.forfeiture_rate, g2.forfeiture_rate,
            "forfeiture_rate should be deterministic"
        );
    }
}
