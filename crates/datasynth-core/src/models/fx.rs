//! Foreign exchange (FX) models.
//!
//! This module provides comprehensive FX rate management including:
//! - Exchange rate types (spot, closing, average, budget)
//! - Currency pair definitions
//! - Rate tables with temporal validity
//! - Currency translation methods for consolidation

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Currency pair representing source and target currencies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyPair {
    /// Source (from) currency code (ISO 4217).
    pub from_currency: String,
    /// Target (to) currency code (ISO 4217).
    pub to_currency: String,
}

impl CurrencyPair {
    /// Creates a new currency pair.
    #[allow(clippy::too_many_arguments)]
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from_currency: from.to_uppercase(),
            to_currency: to.to_uppercase(),
        }
    }

    /// Returns the inverse currency pair.
    pub fn inverse(&self) -> Self {
        Self {
            from_currency: self.to_currency.clone(),
            to_currency: self.from_currency.clone(),
        }
    }

    /// Returns the pair as a string (e.g., "EUR/USD").
    pub fn as_string(&self) -> String {
        format!("{}/{}", self.from_currency, self.to_currency)
    }

    /// Returns true if this is a same-currency pair.
    pub fn is_same_currency(&self) -> bool {
        self.from_currency == self.to_currency
    }
}

impl std::fmt::Display for CurrencyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.from_currency, self.to_currency)
    }
}

/// Type of exchange rate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateType {
    /// Spot rate - rate at transaction date.
    Spot,
    /// Closing rate - rate at period end (for balance sheet translation).
    Closing,
    /// Average rate - period average rate (for P&L translation).
    Average,
    /// Budget rate - rate used for budgeting/planning.
    Budget,
    /// Historical rate - rate at original transaction date (for equity items).
    Historical,
    /// Negotiated rate - contractually agreed rate.
    Negotiated,
    /// Custom rate type.
    Custom(String),
}

impl std::fmt::Display for RateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateType::Spot => write!(f, "SPOT"),
            RateType::Closing => write!(f, "CLOSING"),
            RateType::Average => write!(f, "AVERAGE"),
            RateType::Budget => write!(f, "BUDGET"),
            RateType::Historical => write!(f, "HISTORICAL"),
            RateType::Negotiated => write!(f, "NEGOTIATED"),
            RateType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// An exchange rate for a currency pair on a specific date.
#[derive(Debug, Clone)]
pub struct FxRate {
    /// Currency pair.
    pub pair: CurrencyPair,
    /// Rate type.
    pub rate_type: RateType,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Exchange rate (units of to_currency per 1 unit of from_currency).
    pub rate: Decimal,
    /// Inverse rate (for convenience).
    pub inverse_rate: Decimal,
    /// Rate source (e.g., "ECB", "FED", "INTERNAL").
    pub source: String,
    /// Validity end date (if rate has limited validity).
    pub valid_until: Option<NaiveDate>,
}

impl FxRate {
    /// Creates a new FX rate.
    pub fn new(
        from_currency: &str,
        to_currency: &str,
        rate_type: RateType,
        effective_date: NaiveDate,
        rate: Decimal,
        source: &str,
    ) -> Self {
        let inverse = if rate > Decimal::ZERO {
            (Decimal::ONE / rate).round_dp(6)
        } else {
            Decimal::ZERO
        };

        Self {
            pair: CurrencyPair::new(from_currency, to_currency),
            rate_type,
            effective_date,
            rate,
            inverse_rate: inverse,
            source: source.to_string(),
            valid_until: None,
        }
    }

    /// Creates a rate with validity period.
    pub fn with_validity(mut self, valid_until: NaiveDate) -> Self {
        self.valid_until = Some(valid_until);
        self
    }

    /// Converts an amount from source to target currency.
    pub fn convert(&self, amount: Decimal) -> Decimal {
        (amount * self.rate).round_dp(2)
    }

    /// Converts an amount from target to source currency (inverse).
    pub fn convert_inverse(&self, amount: Decimal) -> Decimal {
        (amount * self.inverse_rate).round_dp(2)
    }

    /// Returns true if the rate is valid on the given date.
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        if date < self.effective_date {
            return false;
        }
        if let Some(valid_until) = self.valid_until {
            date <= valid_until
        } else {
            true
        }
    }
}

/// Collection of FX rates with lookup functionality.
#[derive(Debug, Clone, Default)]
pub struct FxRateTable {
    /// Rates indexed by (currency_pair, rate_type, date).
    rates: HashMap<(String, String), Vec<FxRate>>,
    /// Base currency for the rate table.
    pub base_currency: String,
}

impl FxRateTable {
    /// Creates a new FX rate table with the specified base currency.
    pub fn new(base_currency: &str) -> Self {
        Self {
            rates: HashMap::new(),
            base_currency: base_currency.to_uppercase(),
        }
    }

    /// Adds a rate to the table.
    pub fn add_rate(&mut self, rate: FxRate) {
        let key = (
            format!("{}_{}", rate.pair.from_currency, rate.pair.to_currency),
            rate.rate_type.to_string(),
        );
        self.rates.entry(key).or_default().push(rate);
    }

    /// Gets a rate for the currency pair, type, and date.
    pub fn get_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate_type: &RateType,
        date: NaiveDate,
    ) -> Option<&FxRate> {
        // Same currency - no rate needed
        if from_currency.to_uppercase() == to_currency.to_uppercase() {
            return None;
        }

        let key = (
            format!(
                "{}_{}",
                from_currency.to_uppercase(),
                to_currency.to_uppercase()
            ),
            rate_type.to_string(),
        );

        self.rates.get(&key).and_then(|rates| {
            rates
                .iter()
                .filter(|r| r.is_valid_on(date))
                .max_by_key(|r| r.effective_date)
        })
    }

    /// Gets the closing rate for a currency pair on a date.
    pub fn get_closing_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Option<&FxRate> {
        self.get_rate(from_currency, to_currency, &RateType::Closing, date)
    }

    /// Gets the average rate for a currency pair on a date.
    pub fn get_average_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Option<&FxRate> {
        self.get_rate(from_currency, to_currency, &RateType::Average, date)
    }

    /// Gets the spot rate for a currency pair on a date.
    pub fn get_spot_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Option<&FxRate> {
        self.get_rate(from_currency, to_currency, &RateType::Spot, date)
    }

    /// Converts an amount using the appropriate rate.
    pub fn convert(
        &self,
        amount: Decimal,
        from_currency: &str,
        to_currency: &str,
        rate_type: &RateType,
        date: NaiveDate,
    ) -> Option<Decimal> {
        if from_currency.to_uppercase() == to_currency.to_uppercase() {
            return Some(amount);
        }

        // Try direct rate
        if let Some(rate) = self.get_rate(from_currency, to_currency, rate_type, date) {
            return Some(rate.convert(amount));
        }

        // Try inverse rate
        if let Some(rate) = self.get_rate(to_currency, from_currency, rate_type, date) {
            return Some(rate.convert_inverse(amount));
        }

        // Try triangulation through base currency
        if from_currency.to_uppercase() != self.base_currency
            && to_currency.to_uppercase() != self.base_currency
        {
            let to_base = self.get_rate(from_currency, &self.base_currency, rate_type, date);
            let from_base = self.get_rate(&self.base_currency, to_currency, rate_type, date);

            if let (Some(r1), Some(r2)) = (to_base, from_base) {
                let base_amount = r1.convert(amount);
                return Some(r2.convert(base_amount));
            }
        }

        None
    }

    /// Returns all rates for a currency pair.
    pub fn get_all_rates(&self, from_currency: &str, to_currency: &str) -> Vec<&FxRate> {
        self.rates
            .iter()
            .filter(|((pair, _), _)| {
                *pair
                    == format!(
                        "{}_{}",
                        from_currency.to_uppercase(),
                        to_currency.to_uppercase()
                    )
            })
            .flat_map(|(_, rates)| rates.iter())
            .collect()
    }

    /// Returns the number of rates in the table.
    pub fn len(&self) -> usize {
        self.rates.values().map(|v| v.len()).sum()
    }

    /// Returns true if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.rates.is_empty()
    }
}

/// Currency translation method for financial statement consolidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationMethod {
    /// Current rate method - all items at closing rate (except equity at historical).
    CurrentRate,
    /// Temporal method - monetary items at closing, non-monetary at historical.
    Temporal,
    /// Monetary/Non-monetary method.
    MonetaryNonMonetary,
}

/// Account classification for currency translation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationAccountType {
    /// Asset - balance sheet debit.
    Asset,
    /// Liability - balance sheet credit.
    Liability,
    /// Equity - owner's equity.
    Equity,
    /// Revenue - income statement credit.
    Revenue,
    /// Expense - income statement debit.
    Expense,
    /// Retained Earnings - special equity treatment.
    RetainedEarnings,
    /// Common Stock - historical rate.
    CommonStock,
    /// APIC - historical rate.
    AdditionalPaidInCapital,
}

/// Result of translating an amount.
#[derive(Debug, Clone)]
pub struct TranslatedAmount {
    /// Original amount in local currency.
    pub local_amount: Decimal,
    /// Local currency code.
    pub local_currency: String,
    /// Translated amount in group currency.
    pub group_amount: Decimal,
    /// Group currency code.
    pub group_currency: String,
    /// Rate used for translation.
    pub rate_used: Decimal,
    /// Rate type used.
    pub rate_type: RateType,
    /// Translation date.
    pub translation_date: NaiveDate,
}

/// Currency Translation Adjustment (CTA) entry.
#[derive(Debug, Clone)]
pub struct CTAEntry {
    /// Entry ID.
    pub entry_id: String,
    /// Company code (subsidiary).
    pub company_code: String,
    /// Local currency.
    pub local_currency: String,
    /// Group currency.
    pub group_currency: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Period end date.
    pub period_end_date: NaiveDate,
    /// CTA amount (positive = gain, negative = loss).
    pub cta_amount: Decimal,
    /// Opening rate used.
    pub opening_rate: Decimal,
    /// Closing rate used.
    pub closing_rate: Decimal,
    /// Average rate used.
    pub average_rate: Decimal,
    /// Net assets at opening (local currency).
    pub opening_net_assets_local: Decimal,
    /// Net assets at closing (local currency).
    pub closing_net_assets_local: Decimal,
    /// Net income for period (local currency).
    pub net_income_local: Decimal,
    /// Breakdown by component.
    pub components: Vec<CTAComponent>,
}

/// Component of CTA calculation.
#[derive(Debug, Clone)]
pub struct CTAComponent {
    /// Component description.
    pub description: String,
    /// Local currency amount.
    pub local_amount: Decimal,
    /// Rate applied.
    pub rate: Decimal,
    /// Group currency amount.
    pub group_amount: Decimal,
}

impl CTAEntry {
    /// Creates a new CTA entry.
    pub fn new(
        entry_id: String,
        company_code: String,
        local_currency: String,
        group_currency: String,
        fiscal_year: i32,
        fiscal_period: u8,
        period_end_date: NaiveDate,
    ) -> Self {
        Self {
            entry_id,
            company_code,
            local_currency,
            group_currency,
            fiscal_year,
            fiscal_period,
            period_end_date,
            cta_amount: Decimal::ZERO,
            opening_rate: Decimal::ONE,
            closing_rate: Decimal::ONE,
            average_rate: Decimal::ONE,
            opening_net_assets_local: Decimal::ZERO,
            closing_net_assets_local: Decimal::ZERO,
            net_income_local: Decimal::ZERO,
            components: Vec::new(),
        }
    }

    /// Calculates CTA using the current rate method.
    ///
    /// CTA = Net Assets(closing) × Closing Rate
    ///     - Net Assets(opening) × Opening Rate
    ///     - Net Income × Average Rate
    pub fn calculate_current_rate_method(&mut self) {
        let closing_translated = self.closing_net_assets_local * self.closing_rate;
        let opening_translated = self.opening_net_assets_local * self.opening_rate;
        let income_translated = self.net_income_local * self.average_rate;

        self.cta_amount = closing_translated - opening_translated - income_translated;

        self.components = vec![
            CTAComponent {
                description: "Closing net assets at closing rate".to_string(),
                local_amount: self.closing_net_assets_local,
                rate: self.closing_rate,
                group_amount: closing_translated,
            },
            CTAComponent {
                description: "Opening net assets at opening rate".to_string(),
                local_amount: self.opening_net_assets_local,
                rate: self.opening_rate,
                group_amount: opening_translated,
            },
            CTAComponent {
                description: "Net income at average rate".to_string(),
                local_amount: self.net_income_local,
                rate: self.average_rate,
                group_amount: income_translated,
            },
        ];
    }
}

/// Realized FX gain/loss from settling a transaction in foreign currency.
#[derive(Debug, Clone)]
pub struct RealizedFxGainLoss {
    /// Document reference.
    pub document_number: String,
    /// Company code.
    pub company_code: String,
    /// Transaction date (original).
    pub transaction_date: NaiveDate,
    /// Settlement date.
    pub settlement_date: NaiveDate,
    /// Transaction currency.
    pub transaction_currency: String,
    /// Local currency.
    pub local_currency: String,
    /// Original amount (transaction currency).
    pub original_amount: Decimal,
    /// Original local amount (at transaction date rate).
    pub original_local_amount: Decimal,
    /// Settlement local amount (at settlement date rate).
    pub settlement_local_amount: Decimal,
    /// Realized gain/loss (positive = gain).
    pub gain_loss: Decimal,
    /// Transaction date rate.
    pub transaction_rate: Decimal,
    /// Settlement date rate.
    pub settlement_rate: Decimal,
}

impl RealizedFxGainLoss {
    /// Creates a new realized FX gain/loss entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_number: String,
        company_code: String,
        transaction_date: NaiveDate,
        settlement_date: NaiveDate,
        transaction_currency: String,
        local_currency: String,
        original_amount: Decimal,
        transaction_rate: Decimal,
        settlement_rate: Decimal,
    ) -> Self {
        let original_local = (original_amount * transaction_rate).round_dp(2);
        let settlement_local = (original_amount * settlement_rate).round_dp(2);
        let gain_loss = settlement_local - original_local;

        Self {
            document_number,
            company_code,
            transaction_date,
            settlement_date,
            transaction_currency,
            local_currency,
            original_amount,
            original_local_amount: original_local,
            settlement_local_amount: settlement_local,
            gain_loss,
            transaction_rate,
            settlement_rate,
        }
    }

    /// Returns true if this is a gain.
    pub fn is_gain(&self) -> bool {
        self.gain_loss > Decimal::ZERO
    }
}

/// Unrealized FX gain/loss from revaluing open items.
#[derive(Debug, Clone)]
pub struct UnrealizedFxGainLoss {
    /// Revaluation run ID.
    pub revaluation_id: String,
    /// Company code.
    pub company_code: String,
    /// Revaluation date.
    pub revaluation_date: NaiveDate,
    /// Account code.
    pub account_code: String,
    /// Document reference.
    pub document_number: String,
    /// Transaction currency.
    pub transaction_currency: String,
    /// Local currency.
    pub local_currency: String,
    /// Open amount (transaction currency).
    pub open_amount: Decimal,
    /// Book value (local currency, at original rate).
    pub book_value_local: Decimal,
    /// Revalued amount (local currency, at revaluation rate).
    pub revalued_local: Decimal,
    /// Unrealized gain/loss.
    pub gain_loss: Decimal,
    /// Original rate.
    pub original_rate: Decimal,
    /// Revaluation rate.
    pub revaluation_rate: Decimal,
}

/// Common currency codes.
pub mod currencies {
    pub const USD: &str = "USD";
    pub const EUR: &str = "EUR";
    pub const GBP: &str = "GBP";
    pub const JPY: &str = "JPY";
    pub const CHF: &str = "CHF";
    pub const CAD: &str = "CAD";
    pub const AUD: &str = "AUD";
    pub const CNY: &str = "CNY";
    pub const INR: &str = "INR";
    pub const BRL: &str = "BRL";
    pub const MXN: &str = "MXN";
    pub const KRW: &str = "KRW";
    pub const SGD: &str = "SGD";
    pub const HKD: &str = "HKD";
    pub const SEK: &str = "SEK";
    pub const NOK: &str = "NOK";
    pub const DKK: &str = "DKK";
    pub const PLN: &str = "PLN";
    pub const ZAR: &str = "ZAR";
    pub const THB: &str = "THB";
}

/// Base rates against USD for common currencies (approximate).
pub fn base_rates_usd() -> HashMap<String, Decimal> {
    let mut rates = HashMap::new();
    rates.insert("EUR".to_string(), dec!(1.10)); // EUR/USD
    rates.insert("GBP".to_string(), dec!(1.27)); // GBP/USD
    rates.insert("JPY".to_string(), dec!(0.0067)); // JPY/USD (1/150)
    rates.insert("CHF".to_string(), dec!(1.13)); // CHF/USD
    rates.insert("CAD".to_string(), dec!(0.74)); // CAD/USD
    rates.insert("AUD".to_string(), dec!(0.65)); // AUD/USD
    rates.insert("CNY".to_string(), dec!(0.14)); // CNY/USD
    rates.insert("INR".to_string(), dec!(0.012)); // INR/USD
    rates.insert("BRL".to_string(), dec!(0.20)); // BRL/USD
    rates.insert("MXN".to_string(), dec!(0.058)); // MXN/USD
    rates.insert("KRW".to_string(), dec!(0.00075)); // KRW/USD
    rates.insert("SGD".to_string(), dec!(0.75)); // SGD/USD
    rates.insert("HKD".to_string(), dec!(0.128)); // HKD/USD
    rates.insert("SEK".to_string(), dec!(0.095)); // SEK/USD
    rates.insert("NOK".to_string(), dec!(0.093)); // NOK/USD
    rates.insert("DKK".to_string(), dec!(0.147)); // DKK/USD
    rates.insert("PLN".to_string(), dec!(0.25)); // PLN/USD
    rates.insert("ZAR".to_string(), dec!(0.053)); // ZAR/USD
    rates.insert("THB".to_string(), dec!(0.028)); // THB/USD
    rates
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_pair() {
        let pair = CurrencyPair::new("EUR", "USD");
        assert_eq!(pair.from_currency, "EUR");
        assert_eq!(pair.to_currency, "USD");
        assert_eq!(pair.as_string(), "EUR/USD");

        let inverse = pair.inverse();
        assert_eq!(inverse.from_currency, "USD");
        assert_eq!(inverse.to_currency, "EUR");
    }

    #[test]
    fn test_fx_rate_conversion() {
        let rate = FxRate::new(
            "EUR",
            "USD",
            RateType::Spot,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(1.10),
            "ECB",
        );

        let converted = rate.convert(dec!(100));
        assert_eq!(converted, dec!(110.00));

        let inverse = rate.convert_inverse(dec!(110));
        assert_eq!(inverse, dec!(100.00));
    }

    #[test]
    fn test_fx_rate_table() {
        let mut table = FxRateTable::new("USD");

        table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Spot,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(1.10),
            "ECB",
        ));

        let converted = table.convert(
            dec!(100),
            "EUR",
            "USD",
            &RateType::Spot,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        assert_eq!(converted, Some(dec!(110.00)));
    }

    #[test]
    fn test_cta_calculation() {
        let mut cta = CTAEntry::new(
            "CTA-001".to_string(),
            "1200".to_string(),
            "EUR".to_string(),
            "USD".to_string(),
            2024,
            12,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        cta.opening_net_assets_local = dec!(1000000);
        cta.closing_net_assets_local = dec!(1100000);
        cta.net_income_local = dec!(100000);
        cta.opening_rate = dec!(1.08);
        cta.closing_rate = dec!(1.12);
        cta.average_rate = dec!(1.10);

        cta.calculate_current_rate_method();

        // Closing: 1,100,000 × 1.12 = 1,232,000
        // Opening: 1,000,000 × 1.08 = 1,080,000
        // Income:    100,000 × 1.10 =   110,000
        // CTA: 1,232,000 - 1,080,000 - 110,000 = 42,000
        assert_eq!(cta.cta_amount, dec!(42000));
    }
}
