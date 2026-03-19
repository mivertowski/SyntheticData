//! IAS 21 functional currency translation generator.
//!
//! Produces [`CurrencyTranslationResult`] records for entities whose functional
//! currency differs from the group presentation currency.  The current-rate
//! method (most common for foreign operations) is used by default:
//!
//! | Item type                       | Rate applied      |
//! |---------------------------------|-------------------|
//! | BS monetary (cash, AR, AP …)    | closing rate      |
//! | BS non-monetary (PP&E, inventory, equity) | historical rate |
//! | P&L (revenue, expenses)         | average rate      |
//!
//! The **Currency Translation Adjustment (CTA)** is the balancing amount
//! between "all BS items at closing rate" and "mixed-rate translated BS"
//! — recognised in Other Comprehensive Income.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::currency_translation_result::{
    CurrencyTranslationResult, Ias21TranslationMethod, TranslatedLineItem, TranslationRateType,
};
use datasynth_core::models::{FxRateTable, RateType};

// ---------------------------------------------------------------------------
// Synthetic account structure used when no trial-balance data is provided.
// Each entry is (account_code, account_type_label, is_monetary_bs, is_pnl,
//               functional_amount_factor).
// ---------------------------------------------------------------------------

/// Synthetic balance-sheet / P&L structure used when no real trial balance
/// is provided.  Amounts are scaled from a `revenue_proxy` so that the
/// generated data is internally consistent.
static SYNTHETIC_ACCOUNTS: &[(&str, &str, bool, bool, f64)] = &[
    // (code, type_label, is_monetary_bs, is_pnl, amount_factor_of_revenue)
    // --- Balance sheet — monetary ---
    ("1000", "Asset", true, false, 0.20),      // Cash
    ("1100", "Asset", true, false, 0.30),      // Accounts receivable
    ("2000", "Liability", true, false, -0.25), // Accounts payable
    ("2100", "Liability", true, false, -0.10), // Accrued liabilities
    // --- Balance sheet — non-monetary ---
    ("1500", "Asset", false, false, 0.15), // Inventory (at cost)
    ("1600", "Asset", false, false, 0.40), // PP&E (net)
    ("3100", "Equity", false, false, -0.50), // Common stock (historical)
    ("3300", "Equity", false, false, -0.20), // Retained earnings
    // --- P&L ---
    ("4000", "Revenue", false, true, 1.00),  // Revenue
    ("5000", "Expense", false, true, -0.60), // COGS
    ("6000", "Expense", false, true, -0.25), // Operating expenses
];

/// Generator for IAS 21 functional-currency translations.
pub struct FunctionalCurrencyTranslator;

impl FunctionalCurrencyTranslator {
    /// Translate a single entity for one reporting period.
    ///
    /// # Parameters
    /// - `entity_code`            — company / entity identifier
    /// - `functional_currency`    — entity's functional currency (ISO 4217)
    /// - `presentation_currency`  — group presentation currency (ISO 4217)
    /// - `period_label`           — human-readable period, e.g. "2024-12"
    /// - `period_end`             — last day of the period
    /// - `revenue_proxy`          — scale for synthetic amounts (functional currency)
    /// - `rate_table`             — pre-populated [`FxRateTable`]
    ///
    /// Returns `None` when functional == presentation (no translation needed;
    /// a zero-CTA result is still constructed for completeness).
    pub fn translate(
        entity_code: &str,
        functional_currency: &str,
        presentation_currency: &str,
        period_label: &str,
        period_end: NaiveDate,
        revenue_proxy: Decimal,
        rate_table: &FxRateTable,
    ) -> CurrencyTranslationResult {
        // Same-currency: no translation required.
        if functional_currency.to_uppercase() == presentation_currency.to_uppercase() {
            return Self::identity_result(
                entity_code,
                functional_currency,
                presentation_currency,
                period_label,
                revenue_proxy,
            );
        }

        // Retrieve rates.
        let closing_rate = rate_table
            .get_closing_rate(functional_currency, presentation_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(Decimal::ONE);

        let average_rate = rate_table
            .get_average_rate(functional_currency, presentation_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(closing_rate);

        // Use a representative "historical" rate slightly different from
        // closing to model equity accounts that were recorded at inception.
        // In practice this would come from the original transaction dates;
        // here we approximate as 95% of the closing rate.
        let historical_rate = rate_table
            .get_rate(
                functional_currency,
                presentation_currency,
                &RateType::Historical,
                period_end,
            )
            .map(|r| r.rate)
            .unwrap_or_else(|| {
                // Fallback: use 95% of closing as a reasonable historical proxy.
                (closing_rate * dec!(0.95)).round_dp(6)
            });

        let mut translated_items: Vec<TranslatedLineItem> = Vec::new();
        let mut total_bs_functional = Decimal::ZERO;
        let mut total_bs_presentation = Decimal::ZERO;
        let mut total_pnl_functional = Decimal::ZERO;
        let mut total_pnl_presentation = Decimal::ZERO;

        for &(account, type_label, is_monetary_bs, is_pnl, factor) in SYNTHETIC_ACCOUNTS {
            let func_amount =
                (revenue_proxy * Decimal::try_from(factor).unwrap_or(Decimal::ZERO)).round_dp(2);

            let (rate_used, rate_type) = if is_pnl {
                (average_rate, TranslationRateType::AverageRate)
            } else if is_monetary_bs {
                (closing_rate, TranslationRateType::ClosingRate)
            } else {
                // Non-monetary BS (inventory at cost, PP&E, equity)
                (historical_rate, TranslationRateType::HistoricalRate)
            };

            let pres_amount = (func_amount * rate_used).round_dp(2);

            if is_pnl {
                total_pnl_functional += func_amount;
                total_pnl_presentation += pres_amount;
            } else {
                total_bs_functional += func_amount;
                total_bs_presentation += pres_amount;
            }

            translated_items.push(TranslatedLineItem {
                account: account.to_string(),
                account_type: type_label.to_string(),
                functional_amount: func_amount,
                rate_used,
                rate_type,
                presentation_amount: pres_amount,
            });
        }

        // CTA = BS if all translated at closing − BS translated at mixed rates
        //
        // The "all-at-closing" method would give:
        //   total_bs_functional × closing_rate
        // The actual mixed-rate translation gives:
        //   total_bs_presentation
        //
        // CTA = all_closing − mixed
        let all_closing_bs = (total_bs_functional * closing_rate).round_dp(2);
        let cta_amount = (all_closing_bs - total_bs_presentation).round_dp(2);

        CurrencyTranslationResult {
            entity_code: entity_code.to_string(),
            functional_currency: functional_currency.to_uppercase(),
            presentation_currency: presentation_currency.to_uppercase(),
            period: period_label.to_string(),
            translation_method: Ias21TranslationMethod::CurrentRate,
            translated_items,
            cta_amount,
            closing_rate,
            average_rate,
            total_balance_sheet_functional: total_bs_functional,
            total_balance_sheet_presentation: total_bs_presentation,
            total_pnl_functional,
            total_pnl_presentation,
        }
    }

    /// Build a no-op result when functional currency equals presentation currency.
    fn identity_result(
        entity_code: &str,
        functional_currency: &str,
        presentation_currency: &str,
        period_label: &str,
        revenue_proxy: Decimal,
    ) -> CurrencyTranslationResult {
        let mut translated_items: Vec<TranslatedLineItem> = Vec::new();
        let mut total_bs_functional = Decimal::ZERO;
        let mut total_pnl_functional = Decimal::ZERO;

        for &(account, type_label, _, is_pnl, factor) in SYNTHETIC_ACCOUNTS {
            let func_amount =
                (revenue_proxy * Decimal::try_from(factor).unwrap_or(Decimal::ZERO)).round_dp(2);

            if is_pnl {
                total_pnl_functional += func_amount;
            } else {
                total_bs_functional += func_amount;
            }

            translated_items.push(TranslatedLineItem {
                account: account.to_string(),
                account_type: type_label.to_string(),
                functional_amount: func_amount,
                rate_used: Decimal::ONE,
                rate_type: TranslationRateType::NoTranslation,
                presentation_amount: func_amount,
            });
        }

        CurrencyTranslationResult {
            entity_code: entity_code.to_string(),
            functional_currency: functional_currency.to_uppercase(),
            presentation_currency: presentation_currency.to_uppercase(),
            period: period_label.to_string(),
            translation_method: Ias21TranslationMethod::CurrentRate,
            translated_items,
            cta_amount: Decimal::ZERO,
            closing_rate: Decimal::ONE,
            average_rate: Decimal::ONE,
            total_balance_sheet_functional: total_bs_functional,
            total_balance_sheet_presentation: total_bs_functional, // same
            total_pnl_functional,
            total_pnl_presentation: total_pnl_functional,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{FxRate, FxRateTable, RateType};
    use rust_decimal_macros::dec;

    fn make_rate_table() -> FxRateTable {
        let mut table = FxRateTable::new("USD");
        let period_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Closing,
            period_end,
            dec!(1.12),
            "TEST",
        ));
        table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Average,
            period_end,
            dec!(1.08),
            "TEST",
        ));
        table
    }

    #[test]
    fn test_same_currency_no_translation() {
        let table = make_rate_table();
        let result = FunctionalCurrencyTranslator::translate(
            "1000",
            "USD",
            "USD",
            "2024-12",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1_000_000),
            &table,
        );

        assert_eq!(result.cta_amount, Decimal::ZERO);
        assert_eq!(result.closing_rate, Decimal::ONE);
        assert!(result
            .translated_items
            .iter()
            .all(|i| i.rate_type == TranslationRateType::NoTranslation));
    }

    #[test]
    fn test_different_currency_cta_non_zero() {
        let table = make_rate_table();
        let result = FunctionalCurrencyTranslator::translate(
            "1200",
            "EUR",
            "USD",
            "2024-12",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1_000_000),
            &table,
        );

        // CTA should be non-zero because closing != historical rate
        assert_ne!(result.cta_amount, Decimal::ZERO);
        assert_eq!(result.closing_rate, dec!(1.12));
        assert_eq!(result.average_rate, dec!(1.08));
    }

    #[test]
    fn test_rate_types_assigned_correctly() {
        let table = make_rate_table();
        let result = FunctionalCurrencyTranslator::translate(
            "1200",
            "EUR",
            "USD",
            "2024-12",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1_000_000),
            &table,
        );

        for item in &result.translated_items {
            match item.account_type.as_str() {
                "Revenue" | "Expense" => {
                    assert_eq!(
                        item.rate_type,
                        TranslationRateType::AverageRate,
                        "P&L account {} should use average rate",
                        item.account
                    );
                }
                "Asset" | "Liability"
                    if item.account.starts_with('1')
                        && ["1000", "1100"].contains(&item.account.as_str()) =>
                {
                    assert_eq!(
                        item.rate_type,
                        TranslationRateType::ClosingRate,
                        "Monetary BS account {} should use closing rate",
                        item.account
                    );
                }
                "Equity" => {
                    assert_eq!(
                        item.rate_type,
                        TranslationRateType::HistoricalRate,
                        "Equity account {} should use historical rate",
                        item.account
                    );
                }
                _ => {} // other accounts: skip
            }
        }
    }
}
