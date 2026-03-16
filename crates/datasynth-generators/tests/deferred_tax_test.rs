//! Integration tests for the deferred tax engine (IAS 12 / ASC 740).
//!
//! Verifies:
//! - DTA / DTL = temporary difference × statutory rate
//! - ETR reconciliation: expected_tax + Σ perm_diff.tax_effect ≈ actual_tax
//! - Rollforward: opening + movement = closing (both DTA and DTL sides)
//! - Generated JEs are balanced (Σ debits = Σ credits)
//! - All deferred tax JEs have `document_type` containing "TAX"
//! - Deterministic output for identical seed

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::deferred_tax::{DeferredTaxType, TemporaryDifference};
use datasynth_generators::tax::deferred_tax_generator::{
    compute_dta_dtl, DeferredTaxGenerator,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn period_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
}

// ---------------------------------------------------------------------------
// compute_dta_dtl unit tests
// ---------------------------------------------------------------------------

#[test]
fn test_compute_dta_dtl_single_dta() {
    let diffs = vec![TemporaryDifference {
        id: "TD1".into(),
        entity_code: "C001".into(),
        account: "2200".into(),
        description: "Accruals".into(),
        book_basis: dec!(100_000),
        tax_basis: Decimal::ZERO,
        difference: dec!(100_000),
        deferred_type: DeferredTaxType::Asset,
        originating_standard: None,
    }];
    let (dta, dtl) = compute_dta_dtl(&diffs, dec!(0.21));
    assert_eq!(dta, (dec!(100_000) * dec!(0.21)).round_dp(2));
    assert_eq!(dtl, Decimal::ZERO);
}

#[test]
fn test_compute_dta_dtl_single_dtl() {
    let diffs = vec![TemporaryDifference {
        id: "TD2".into(),
        entity_code: "C001".into(),
        account: "1500".into(),
        description: "Depreciation".into(),
        book_basis: dec!(800_000),
        tax_basis: dec!(600_000),
        difference: dec!(200_000),
        deferred_type: DeferredTaxType::Liability,
        originating_standard: None,
    }];
    let (dta, dtl) = compute_dta_dtl(&diffs, dec!(0.21));
    assert_eq!(dtl, (dec!(200_000) * dec!(0.21)).round_dp(2));
    assert_eq!(dta, Decimal::ZERO);
}

#[test]
fn test_compute_dta_dtl_mixed() {
    let diffs = vec![
        TemporaryDifference {
            id: "TD1".into(),
            entity_code: "C001".into(),
            account: "1500".into(),
            description: "Depreciation".into(),
            book_basis: dec!(500_000),
            tax_basis: dec!(400_000),
            difference: dec!(100_000),
            deferred_type: DeferredTaxType::Liability,
            originating_standard: None,
        },
        TemporaryDifference {
            id: "TD2".into(),
            entity_code: "C001".into(),
            account: "2200".into(),
            description: "Accruals".into(),
            book_basis: dec!(50_000),
            tax_basis: Decimal::ZERO,
            difference: dec!(50_000),
            deferred_type: DeferredTaxType::Asset,
            originating_standard: None,
        },
    ];
    let (dta, dtl) = compute_dta_dtl(&diffs, dec!(0.21));
    assert_eq!(dtl, (dec!(100_000) * dec!(0.21)).round_dp(2));
    assert_eq!(dta, (dec!(50_000) * dec!(0.21)).round_dp(2));
}

// ---------------------------------------------------------------------------
// Generator integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_generate_produces_data_for_all_companies() {
    let mut gen = DeferredTaxGenerator::new(42);
    let companies = vec![("C001", "US"), ("C002", "DE"), ("C003", "GB")];
    let snap = gen.generate(&companies, period_end(), &[]);

    // At least 5 temp diffs per company
    assert!(
        snap.temporary_differences.len() >= 3 * 5,
        "Expected ≥15 temp diffs for 3 companies, got {}",
        snap.temporary_differences.len()
    );

    // One ETR reconciliation per company
    assert_eq!(
        snap.etr_reconciliations.len(),
        3,
        "Expected 3 ETR reconciliations, got {}",
        snap.etr_reconciliations.len()
    );

    // One rollforward per company
    assert_eq!(
        snap.rollforwards.len(),
        3,
        "Expected 3 rollforwards, got {}",
        snap.rollforwards.len()
    );
}

#[test]
fn test_etr_reconciliation_math() {
    let mut gen = DeferredTaxGenerator::new(17);
    let snap = gen.generate(&[("C001", "US")], period_end(), &[]);
    let etr = &snap.etr_reconciliations[0];

    // expected_tax = pre_tax_income × statutory_rate
    let computed_expected = (etr.pre_tax_income * etr.statutory_rate).round_dp(2);
    assert_eq!(
        etr.expected_tax, computed_expected,
        "expected_tax mismatch: got {}, want {}",
        etr.expected_tax, computed_expected
    );

    // actual_tax = expected_tax + Σ perm_diff.tax_effect
    let total_perm_effect: Decimal = etr.permanent_differences.iter().map(|p| p.tax_effect).sum();
    let computed_actual = (etr.expected_tax + total_perm_effect).round_dp(2);
    assert_eq!(
        etr.actual_tax, computed_actual,
        "actual_tax mismatch: got {}, want {}",
        etr.actual_tax, computed_actual
    );

    // effective_rate = actual_tax / pre_tax_income  (when PTI ≠ 0)
    if etr.pre_tax_income != Decimal::ZERO {
        let computed_etr = (etr.actual_tax / etr.pre_tax_income).round_dp(6);
        assert_eq!(
            etr.effective_rate, computed_etr,
            "effective_rate mismatch: got {}, want {}",
            etr.effective_rate, computed_etr
        );
    }
}

#[test]
fn test_permanent_differences_count() {
    let mut gen = DeferredTaxGenerator::new(7);
    let snap = gen.generate(&[("C001", "US")], period_end(), &[]);
    let etr = &snap.etr_reconciliations[0];
    assert!(
        etr.permanent_differences.len() >= 3,
        "Expected ≥3 permanent diffs, got {}",
        etr.permanent_differences.len()
    );
    assert!(
        etr.permanent_differences.len() <= 5,
        "Expected ≤5 permanent diffs, got {}",
        etr.permanent_differences.len()
    );
}

#[test]
fn test_rollforward_opening_plus_movement_equals_closing() {
    let mut gen = DeferredTaxGenerator::new(55);
    let snap = gen.generate(&[("C001", "US"), ("C002", "DE")], period_end(), &[]);

    for rf in &snap.rollforwards {
        // current_year_movement = (closing_dta - opening_dta) - (closing_dtl - opening_dtl)
        let implied_movement =
            (rf.closing_dta - rf.opening_dta) - (rf.closing_dtl - rf.opening_dtl);
        assert_eq!(
            rf.current_year_movement, implied_movement,
            "Rollforward identity violated for {}: movement={}, implied={}",
            rf.entity_code, rf.current_year_movement, implied_movement
        );
    }
}

#[test]
fn test_journal_entries_are_balanced() {
    let mut gen = DeferredTaxGenerator::new(42);
    let snap = gen.generate(&[("C001", "US"), ("C002", "DE")], period_end(), &[]);

    assert!(
        !snap.journal_entries.is_empty(),
        "Expected at least one deferred tax JE"
    );

    for je in &snap.journal_entries {
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debit, total_credit,
            "JE {} is unbalanced: debits={}, credits={}",
            je.header.document_id, total_debit, total_credit
        );
    }
}

#[test]
fn test_journal_entries_document_type_contains_tax() {
    let mut gen = DeferredTaxGenerator::new(42);
    let snap = gen.generate(&[("C001", "US"), ("C002", "GB")], period_end(), &[]);

    for je in &snap.journal_entries {
        assert!(
            je.header.document_type.contains("TAX"),
            "Expected document_type to contain 'TAX', got '{}'",
            je.header.document_type
        );
    }
}

#[test]
fn test_temporary_differences_have_correct_entity_code() {
    let mut gen = DeferredTaxGenerator::new(33);
    let snap = gen.generate(&[("C001", "US")], period_end(), &[]);

    for diff in &snap.temporary_differences {
        assert_eq!(
            diff.entity_code, "C001",
            "Expected entity_code 'C001', got '{}'",
            diff.entity_code
        );
    }
}

#[test]
fn test_dta_dtl_equals_diff_times_rate() {
    let mut gen = DeferredTaxGenerator::new(42);
    let snap = gen.generate(&[("C001", "US")], period_end(), &[]);

    let diffs: Vec<&TemporaryDifference> = snap
        .temporary_differences
        .iter()
        .filter(|d| d.entity_code == "C001")
        .collect();

    let (computed_dta, computed_dtl) = compute_dta_dtl(&diffs.iter().map(|d| (*d).clone()).collect::<Vec<_>>(), dec!(0.21));

    // Verify rollforward closing balances equal computed values
    let rf = snap.rollforwards.iter().find(|r| r.entity_code == "C001").unwrap();
    assert_eq!(
        rf.closing_dta, computed_dta,
        "Rollforward closing_dta {} != computed DTA {}",
        rf.closing_dta, computed_dta
    );
    assert_eq!(
        rf.closing_dtl, computed_dtl,
        "Rollforward closing_dtl {} != computed DTL {}",
        rf.closing_dtl, computed_dtl
    );
}

#[test]
fn test_deterministic_output() {
    let companies = vec![("C001", "US")];

    let mut gen1 = DeferredTaxGenerator::new(99);
    let snap1 = gen1.generate(&companies, period_end(), &[]);

    let mut gen2 = DeferredTaxGenerator::new(99);
    let snap2 = gen2.generate(&companies, period_end(), &[]);

    assert_eq!(
        snap1.temporary_differences.len(),
        snap2.temporary_differences.len(),
        "Temp diff count not deterministic"
    );
    assert_eq!(
        snap1.etr_reconciliations[0].actual_tax,
        snap2.etr_reconciliations[0].actual_tax,
        "actual_tax not deterministic"
    );
    assert_eq!(
        snap1.rollforwards[0].closing_dta,
        snap2.rollforwards[0].closing_dta,
        "closing_dta not deterministic"
    );
    assert_eq!(
        snap1.rollforwards[0].closing_dtl,
        snap2.rollforwards[0].closing_dtl,
        "closing_dtl not deterministic"
    );
    assert_eq!(
        snap1.journal_entries.len(),
        snap2.journal_entries.len(),
        "JE count not deterministic"
    );
}

#[test]
fn test_statutory_rates_by_country() {
    // US = 21%, DE = 30%, GB = 25%
    let mut gen = DeferredTaxGenerator::new(1);
    let snap = gen.generate(&[("C001", "US"), ("C002", "DE"), ("C003", "GB")], period_end(), &[]);

    let us_etr = snap.etr_reconciliations.iter().find(|e| e.entity_code == "C001").unwrap();
    let de_etr = snap.etr_reconciliations.iter().find(|e| e.entity_code == "C002").unwrap();
    let gb_etr = snap.etr_reconciliations.iter().find(|e| e.entity_code == "C003").unwrap();

    assert_eq!(us_etr.statutory_rate, dec!(0.21), "US statutory rate should be 21%");
    assert_eq!(de_etr.statutory_rate, dec!(0.30), "DE statutory rate should be 30%");
    assert_eq!(gb_etr.statutory_rate, dec!(0.25), "GB statutory rate should be 25%");
}

#[test]
fn test_temp_diff_count_per_company_in_range() {
    let mut gen = DeferredTaxGenerator::new(77);
    let snap = gen.generate(&[("C001", "US")], period_end(), &[]);

    let c001_diffs: Vec<_> = snap
        .temporary_differences
        .iter()
        .filter(|d| d.entity_code == "C001")
        .collect();

    assert!(
        c001_diffs.len() >= 5 && c001_diffs.len() <= 8,
        "Expected 5-8 temp diffs per company, got {}",
        c001_diffs.len()
    );
}
