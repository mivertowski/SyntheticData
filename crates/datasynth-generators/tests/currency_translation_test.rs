//! Integration tests for IAS 21 functional currency translation.
//!
//! Run with:
//!   cargo test -p datasynth-generators --test currency_translation_test -- --test-threads=1

use chrono::NaiveDate;
use datasynth_core::models::currency_translation_result::{
    Ias21TranslationMethod, TranslationRateType,
};
use datasynth_core::models::{FxRate, FxRateTable, RateType};
use datasynth_generators::fx::FunctionalCurrencyTranslator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Build a minimal rate table with EUR → USD closing and average rates.
fn eur_usd_rate_table(closing: Decimal, average: Decimal) -> FxRateTable {
    let mut table = FxRateTable::new("USD");
    let period_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    table.add_rate(FxRate::new(
        "EUR",
        "USD",
        RateType::Closing,
        period_end,
        closing,
        "TEST",
    ));
    table.add_rate(FxRate::new(
        "EUR",
        "USD",
        RateType::Average,
        period_end,
        average,
        "TEST",
    ));
    table
}

// ---------------------------------------------------------------------------
// Test 1: Same functional and presentation currency → CTA = 0
// ---------------------------------------------------------------------------

#[test]
fn test_same_currency_no_cta() {
    let table = eur_usd_rate_table(dec!(1.12), dec!(1.08));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_USD",
        "USD",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(1_000_000),
        &table,
    );

    assert_eq!(
        result.cta_amount,
        Decimal::ZERO,
        "CTA must be zero when functional == presentation"
    );
    assert_eq!(result.closing_rate, Decimal::ONE);
    assert_eq!(result.average_rate, Decimal::ONE);

    // All items should have NoTranslation rate type
    for item in &result.translated_items {
        assert_eq!(
            item.rate_type,
            TranslationRateType::NoTranslation,
            "Account {} should have NoTranslation rate type",
            item.account
        );
        assert_eq!(
            item.functional_amount, item.presentation_amount,
            "Functional and presentation amounts must match for same-currency"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 2: Different currencies → CTA must be non-zero when closing ≠ historical
// ---------------------------------------------------------------------------

#[test]
fn test_different_currency_produces_nonzero_cta() {
    // closing 1.12, average 1.08 → historical fallback = closing × 0.95 = 1.064
    let table = eur_usd_rate_table(dec!(1.12), dec!(1.08));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_EUR",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(5_000_000),
        &table,
    );

    // CTA should be non-zero because non-monetary BS items use historical rate
    // (≈ 0.95 × closing) while the all-closing method would use closing for all.
    assert_ne!(
        result.cta_amount,
        Decimal::ZERO,
        "CTA must be non-zero when functional != presentation and rates differ"
    );

    assert_eq!(result.closing_rate, dec!(1.12));
    assert_eq!(result.average_rate, dec!(1.08));
    assert_eq!(result.functional_currency, "EUR");
    assert_eq!(result.presentation_currency, "USD");
    assert_eq!(
        result.translation_method,
        Ias21TranslationMethod::CurrentRate
    );
}

// ---------------------------------------------------------------------------
// Test 3: Rate types are assigned correctly
// ---------------------------------------------------------------------------

#[test]
fn test_correct_rate_types_per_account_category() {
    let table = eur_usd_rate_table(dec!(1.10), dec!(1.06));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_EUR",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(2_000_000),
        &table,
    );

    for item in &result.translated_items {
        match (item.account.as_str(), item.account_type.as_str()) {
            // Cash and AR are monetary → closing rate
            ("1000", _) | ("1100", _) => {
                assert_eq!(
                    item.rate_type,
                    TranslationRateType::ClosingRate,
                    "Account {} (monetary) should use ClosingRate",
                    item.account
                );
                assert_eq!(item.rate_used, dec!(1.10));
            }
            // Liabilities (2000, 2100) are monetary → closing rate
            ("2000", _) | ("2100", _) => {
                assert_eq!(
                    item.rate_type,
                    TranslationRateType::ClosingRate,
                    "Account {} (liability) should use ClosingRate",
                    item.account
                );
            }
            // Inventory (1500) and PP&E (1600) are non-monetary → historical rate
            ("1500", _) | ("1600", _) => {
                assert_eq!(
                    item.rate_type,
                    TranslationRateType::HistoricalRate,
                    "Account {} (non-monetary asset) should use HistoricalRate",
                    item.account
                );
            }
            // Equity → historical rate
            (_, "Equity") => {
                assert_eq!(
                    item.rate_type,
                    TranslationRateType::HistoricalRate,
                    "Equity account {} should use HistoricalRate",
                    item.account
                );
            }
            // P&L → average rate
            (_, "Revenue") | (_, "Expense") => {
                assert_eq!(
                    item.rate_type,
                    TranslationRateType::AverageRate,
                    "P&L account {} should use AverageRate",
                    item.account
                );
                assert_eq!(item.rate_used, dec!(1.06));
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Test 4: CTA formula correctness
// ---------------------------------------------------------------------------

#[test]
fn test_cta_formula() {
    // CTA = (all_BS_functional × closing_rate) - BS_translated_at_mixed_rates
    // Verify this holds for EUR→USD with known rates.
    let table = eur_usd_rate_table(dec!(1.20), dec!(1.15));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_EUR",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(1_000_000),
        &table,
    );

    let recomputed_all_closing =
        (result.total_balance_sheet_functional * result.closing_rate).round_dp(2);
    let expected_cta =
        (recomputed_all_closing - result.total_balance_sheet_presentation).round_dp(2);

    assert_eq!(
        result.cta_amount, expected_cta,
        "CTA must equal (total_bs_functional × closing_rate) - total_bs_presentation"
    );
}

// ---------------------------------------------------------------------------
// Test 5: P&L items use the average rate, BS items do not
// ---------------------------------------------------------------------------

#[test]
fn test_pnl_uses_average_bs_uses_closing_or_historical() {
    let table = eur_usd_rate_table(dec!(1.10), dec!(1.04));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_EUR",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(3_000_000),
        &table,
    );

    // All translated P&L items must use average rate (1.04)
    for item in &result.translated_items {
        if matches!(item.account_type.as_str(), "Revenue" | "Expense") {
            assert_eq!(
                item.rate_used,
                dec!(1.04),
                "P&L account {} should use average rate 1.04, got {}",
                item.account,
                item.rate_used
            );
        }
    }

    // All monetary BS items must use closing rate (1.10)
    for item in &result.translated_items {
        if item.rate_type == TranslationRateType::ClosingRate {
            assert_eq!(
                item.rate_used,
                dec!(1.10),
                "Monetary BS account {} should use closing rate 1.10, got {}",
                item.account,
                item.rate_used
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 6: Verify presentation_amount = functional_amount × rate_used
// ---------------------------------------------------------------------------

#[test]
fn test_presentation_amount_equals_functional_times_rate() {
    let table = eur_usd_rate_table(dec!(1.15), dec!(1.10));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_GBP",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(500_000),
        &table,
    );

    for item in &result.translated_items {
        let expected = (item.functional_amount * item.rate_used).round_dp(2);
        assert_eq!(
            item.presentation_amount,
            expected,
            "Presentation amount for account {} should be {} × {} = {}, got {}",
            item.account,
            item.functional_amount,
            item.rate_used,
            expected,
            item.presentation_amount
        );
    }
}

// ---------------------------------------------------------------------------
// Test 7: Totals are consistent
// ---------------------------------------------------------------------------

#[test]
fn test_total_consistency() {
    let table = eur_usd_rate_table(dec!(1.12), dec!(1.08));
    let result = FunctionalCurrencyTranslator::translate(
        "ENTITY_EUR",
        "EUR",
        "USD",
        "2024-12",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(1_000_000),
        &table,
    );

    // Recompute BS totals from line items
    let computed_bs_func: Decimal = result
        .translated_items
        .iter()
        .filter(|i| !matches!(i.account_type.as_str(), "Revenue" | "Expense"))
        .map(|i| i.functional_amount)
        .sum();

    let computed_bs_pres: Decimal = result
        .translated_items
        .iter()
        .filter(|i| !matches!(i.account_type.as_str(), "Revenue" | "Expense"))
        .map(|i| i.presentation_amount)
        .sum();

    assert_eq!(result.total_balance_sheet_functional, computed_bs_func);
    assert_eq!(result.total_balance_sheet_presentation, computed_bs_pres);

    // Recompute P&L totals
    let computed_pnl_func: Decimal = result
        .translated_items
        .iter()
        .filter(|i| matches!(i.account_type.as_str(), "Revenue" | "Expense"))
        .map(|i| i.functional_amount)
        .sum();

    let computed_pnl_pres: Decimal = result
        .translated_items
        .iter()
        .filter(|i| matches!(i.account_type.as_str(), "Revenue" | "Expense"))
        .map(|i| i.presentation_amount)
        .sum();

    assert_eq!(result.total_pnl_functional, computed_pnl_func);
    assert_eq!(result.total_pnl_presentation, computed_pnl_pres);
}
