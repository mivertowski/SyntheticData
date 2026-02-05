//! Currency translation for financial statements.
//!
//! Translates trial balances and financial statements from local currency
//! to group reporting currency using appropriate translation methods.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::balance::TrialBalance;
use datasynth_core::models::{
    FxRateTable, RateType, TranslatedAmount, TranslationAccountType, TranslationMethod,
};

/// Configuration for currency translation.
#[derive(Debug, Clone)]
pub struct CurrencyTranslatorConfig {
    /// Translation method to use.
    pub method: TranslationMethod,
    /// Group (reporting) currency.
    pub group_currency: String,
    /// Account type mappings (account code prefix -> translation account type).
    pub account_type_map: HashMap<String, TranslationAccountType>,
    /// Equity accounts that use historical rates.
    pub historical_rate_accounts: Vec<String>,
    /// Retained earnings account code.
    pub retained_earnings_account: String,
    /// CTA (Currency Translation Adjustment) account code.
    pub cta_account: String,
}

impl Default for CurrencyTranslatorConfig {
    fn default() -> Self {
        let mut account_type_map = HashMap::new();
        // Assets
        account_type_map.insert("1".to_string(), TranslationAccountType::Asset);
        // Liabilities
        account_type_map.insert("2".to_string(), TranslationAccountType::Liability);
        // Equity
        account_type_map.insert("3".to_string(), TranslationAccountType::Equity);
        // Revenue
        account_type_map.insert("4".to_string(), TranslationAccountType::Revenue);
        // Expenses
        account_type_map.insert("5".to_string(), TranslationAccountType::Expense);
        account_type_map.insert("6".to_string(), TranslationAccountType::Expense);

        Self {
            method: TranslationMethod::CurrentRate,
            group_currency: "USD".to_string(),
            account_type_map,
            historical_rate_accounts: vec![
                "3100".to_string(), // Common Stock
                "3200".to_string(), // APIC
            ],
            retained_earnings_account: "3300".to_string(),
            cta_account: "3900".to_string(),
        }
    }
}

/// Classifies whether an account is monetary based on its account code prefix.
///
/// Monetary items include cash, receivables, payables, and other items that
/// are settled in fixed currency amounts. Non-monetary items include inventory,
/// PP&E, intangibles, equity, and other items whose value fluctuates.
///
/// Classification by 2-digit prefix ranges:
/// - 10xx (Cash & Cash Equivalents): Monetary
/// - 11xx (Accounts Receivable): Monetary
/// - 12xx (Short-term Investments): Monetary
/// - 13xx (Notes Receivable): Monetary
/// - 14xx (Prepaid Expenses): Non-monetary (future economic benefit)
/// - 15xx (Inventory): Non-monetary
/// - 16xx (Property, Plant & Equipment): Non-monetary
/// - 17xx (Intangible Assets): Non-monetary
/// - 18xx (Long-term Investments): Non-monetary
/// - 19xx (Other Non-current Assets): Non-monetary
/// - 20xx-29xx (Liabilities): Monetary (obligations settled in cash)
/// - 30xx-39xx (Equity): Non-monetary
/// - 40xx-49xx (Revenue): Treated as monetary for temporal method
/// - 50xx-69xx (Expenses): Treated as monetary for temporal method
pub fn is_monetary(account_code: &str) -> bool {
    if account_code.len() < 2 {
        // Default to monetary for very short codes
        return true;
    }

    let prefix2 = &account_code[..2];

    match prefix2 {
        // Cash and cash equivalents - monetary
        "10" => true,
        // Accounts receivable - monetary
        "11" => true,
        // Short-term investments / marketable securities - monetary
        "12" => true,
        // Notes receivable - monetary
        "13" => true,
        // Prepaid expenses - non-monetary (future benefit, not cash settlement)
        "14" => false,
        // Inventory - non-monetary
        "15" => false,
        // Property, plant & equipment - non-monetary
        "16" => false,
        // Intangible assets - non-monetary
        "17" => false,
        // Long-term investments - non-monetary
        "18" => false,
        // Other non-current assets - non-monetary
        "19" => false,
        // All liabilities (20xx-29xx) - monetary (obligations to pay cash)
        "20" | "21" | "22" | "23" | "24" | "25" | "26" | "27" | "28" | "29" => true,
        // All equity accounts (30xx-39xx) - non-monetary
        "30" | "31" | "32" | "33" | "34" | "35" | "36" | "37" | "38" | "39" => false,
        // Revenue (40xx-49xx) - income statement items use average rate
        "40" | "41" | "42" | "43" | "44" | "45" | "46" | "47" | "48" | "49" => true,
        // Expenses (50xx-69xx) - income statement items use average rate
        "50" | "51" | "52" | "53" | "54" | "55" | "56" | "57" | "58" | "59" => true,
        "60" | "61" | "62" | "63" | "64" | "65" | "66" | "67" | "68" | "69" => true,
        // Default: treat as monetary (conservative approach)
        _ => true,
    }
}

/// Currency translator for financial statements.
pub struct CurrencyTranslator {
    config: CurrencyTranslatorConfig,
    /// Historical equity rates keyed by account code.
    historical_equity_rates: HashMap<String, Decimal>,
}

impl CurrencyTranslator {
    /// Creates a new currency translator.
    pub fn new(config: CurrencyTranslatorConfig) -> Self {
        Self {
            config,
            historical_equity_rates: HashMap::new(),
        }
    }

    /// Sets historical equity rates for specific accounts.
    ///
    /// These rates are used when translating equity accounts under the
    /// Temporal or MonetaryNonMonetary methods. The rates represent
    /// the exchange rate at the time equity transactions originally occurred.
    pub fn set_historical_equity_rates(&mut self, rates: HashMap<String, Decimal>) {
        self.historical_equity_rates = rates;
    }

    /// Translates a trial balance from local to group currency.
    pub fn translate_trial_balance(
        &self,
        trial_balance: &TrialBalance,
        rate_table: &FxRateTable,
        historical_rates: &HashMap<String, Decimal>,
    ) -> TranslatedTrialBalance {
        let local_currency = &trial_balance.currency;
        let period_end = trial_balance.as_of_date;

        // Get closing and average rates
        let closing_rate = rate_table
            .get_closing_rate(local_currency, &self.config.group_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(Decimal::ONE);

        let average_rate = rate_table
            .get_average_rate(local_currency, &self.config.group_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(closing_rate);

        let mut translated_lines = Vec::new();
        let mut total_local_debit = Decimal::ZERO;
        let mut total_local_credit = Decimal::ZERO;
        let mut total_group_debit = Decimal::ZERO;
        let mut total_group_credit = Decimal::ZERO;

        for line in &trial_balance.lines {
            let account_type = self.determine_account_type(&line.account_code);
            let rate = self.determine_rate(
                &line.account_code,
                &account_type,
                closing_rate,
                average_rate,
                historical_rates,
            );

            let group_debit = (line.debit_balance * rate).round_dp(2);
            let group_credit = (line.credit_balance * rate).round_dp(2);

            translated_lines.push(TranslatedTrialBalanceLine {
                account_code: line.account_code.clone(),
                account_description: Some(line.account_description.clone()),
                account_type: account_type.clone(),
                local_debit: line.debit_balance,
                local_credit: line.credit_balance,
                rate_used: rate,
                rate_type: self.rate_type_for_account(&account_type),
                group_debit,
                group_credit,
            });

            total_local_debit += line.debit_balance;
            total_local_credit += line.credit_balance;
            total_group_debit += group_debit;
            total_group_credit += group_credit;
        }

        // Calculate CTA to balance the translated trial balance
        let cta_amount = total_group_debit - total_group_credit;

        TranslatedTrialBalance {
            company_code: trial_balance.company_code.clone(),
            company_name: trial_balance.company_name.clone().unwrap_or_default(),
            local_currency: local_currency.clone(),
            group_currency: self.config.group_currency.clone(),
            period_end_date: period_end,
            fiscal_year: trial_balance.fiscal_year,
            fiscal_period: trial_balance.fiscal_period as u8,
            lines: translated_lines,
            closing_rate,
            average_rate,
            total_local_debit,
            total_local_credit,
            total_group_debit,
            total_group_credit,
            cta_amount,
            translation_method: self.config.method.clone(),
        }
    }

    /// Translates a single amount.
    ///
    /// For equity accounts, this method will use historical equity rates set
    /// via [`set_historical_equity_rates`] if available, falling back to
    /// `Decimal::ONE` when no historical rate is found.
    pub fn translate_amount(
        &self,
        amount: Decimal,
        local_currency: &str,
        account_code: &str,
        account_type: &TranslationAccountType,
        rate_table: &FxRateTable,
        date: NaiveDate,
    ) -> TranslatedAmount {
        let closing_rate = rate_table
            .get_closing_rate(local_currency, &self.config.group_currency, date)
            .map(|r| r.rate)
            .unwrap_or(Decimal::ONE);

        let average_rate = rate_table
            .get_average_rate(local_currency, &self.config.group_currency, date)
            .map(|r| r.rate)
            .unwrap_or(closing_rate);

        let (rate, rate_type) = match &self.config.method {
            TranslationMethod::CurrentRate => match account_type {
                TranslationAccountType::Asset | TranslationAccountType::Liability => {
                    (closing_rate, RateType::Closing)
                }
                TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                    (average_rate, RateType::Average)
                }
                TranslationAccountType::Equity
                | TranslationAccountType::CommonStock
                | TranslationAccountType::AdditionalPaidInCapital
                | TranslationAccountType::RetainedEarnings => {
                    let hist_rate = self
                        .historical_equity_rates
                        .get(account_code)
                        .copied()
                        .unwrap_or(Decimal::ONE);
                    (hist_rate, RateType::Historical)
                }
            },
            TranslationMethod::Temporal => match account_type {
                TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                    (average_rate, RateType::Average)
                }
                TranslationAccountType::CommonStock
                | TranslationAccountType::AdditionalPaidInCapital
                | TranslationAccountType::RetainedEarnings
                | TranslationAccountType::Equity => {
                    let hist_rate = self
                        .historical_equity_rates
                        .get(account_code)
                        .copied()
                        .unwrap_or(Decimal::ONE);
                    (hist_rate, RateType::Historical)
                }
                TranslationAccountType::Asset | TranslationAccountType::Liability => {
                    if is_monetary(account_code) {
                        (closing_rate, RateType::Closing)
                    } else {
                        let hist_rate = self
                            .historical_equity_rates
                            .get(account_code)
                            .copied()
                            .unwrap_or(closing_rate);
                        (hist_rate, RateType::Historical)
                    }
                }
            },
            TranslationMethod::MonetaryNonMonetary => match account_type {
                TranslationAccountType::CommonStock
                | TranslationAccountType::AdditionalPaidInCapital
                | TranslationAccountType::RetainedEarnings
                | TranslationAccountType::Equity => {
                    let hist_rate = self
                        .historical_equity_rates
                        .get(account_code)
                        .copied()
                        .unwrap_or(Decimal::ONE);
                    (hist_rate, RateType::Historical)
                }
                _ => {
                    if is_monetary(account_code) {
                        (closing_rate, RateType::Closing)
                    } else {
                        let hist_rate = self
                            .historical_equity_rates
                            .get(account_code)
                            .copied()
                            .unwrap_or(closing_rate);
                        (hist_rate, RateType::Historical)
                    }
                }
            },
        };

        TranslatedAmount {
            local_amount: amount,
            local_currency: local_currency.to_string(),
            group_amount: (amount * rate).round_dp(2),
            group_currency: self.config.group_currency.clone(),
            rate_used: rate,
            rate_type,
            translation_date: date,
        }
    }

    /// Determines the account type based on account code.
    fn determine_account_type(&self, account_code: &str) -> TranslationAccountType {
        // Check for specific accounts first
        if self
            .config
            .historical_rate_accounts
            .contains(&account_code.to_string())
        {
            if account_code.starts_with("31") {
                return TranslationAccountType::CommonStock;
            } else if account_code.starts_with("32") {
                return TranslationAccountType::AdditionalPaidInCapital;
            }
        }

        if account_code == self.config.retained_earnings_account {
            return TranslationAccountType::RetainedEarnings;
        }

        // Use prefix mapping
        for (prefix, account_type) in &self.config.account_type_map {
            if account_code.starts_with(prefix) {
                return account_type.clone();
            }
        }

        // Default to asset
        TranslationAccountType::Asset
    }

    /// Looks up a historical equity rate for a given account code.
    ///
    /// Checks both the instance-level `historical_equity_rates` and the
    /// passed-in `historical_rates` parameter. Instance-level rates take precedence.
    fn lookup_historical_equity_rate(
        &self,
        account_code: &str,
        historical_rates: &HashMap<String, Decimal>,
        fallback: Decimal,
    ) -> Decimal {
        self.historical_equity_rates
            .get(account_code)
            .or_else(|| historical_rates.get(account_code))
            .copied()
            .unwrap_or(fallback)
    }

    /// Determines the appropriate rate to use for an account.
    fn determine_rate(
        &self,
        account_code: &str,
        account_type: &TranslationAccountType,
        closing_rate: Decimal,
        average_rate: Decimal,
        historical_rates: &HashMap<String, Decimal>,
    ) -> Decimal {
        match self.config.method {
            TranslationMethod::CurrentRate => {
                match account_type {
                    TranslationAccountType::Asset | TranslationAccountType::Liability => {
                        closing_rate
                    }
                    TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                        average_rate
                    }
                    TranslationAccountType::CommonStock
                    | TranslationAccountType::AdditionalPaidInCapital => {
                        // Use historical rate if available
                        self.lookup_historical_equity_rate(
                            account_code,
                            historical_rates,
                            closing_rate,
                        )
                    }
                    TranslationAccountType::Equity | TranslationAccountType::RetainedEarnings => {
                        // These are typically calculated separately
                        closing_rate
                    }
                }
            }
            TranslationMethod::Temporal => {
                // Temporal method: monetary items at closing rate, non-monetary at historical rate.
                // Equity accounts always use historical rates.
                match account_type {
                    TranslationAccountType::CommonStock
                    | TranslationAccountType::AdditionalPaidInCapital
                    | TranslationAccountType::RetainedEarnings => self
                        .lookup_historical_equity_rate(
                            account_code,
                            historical_rates,
                            closing_rate,
                        ),
                    TranslationAccountType::Equity => self.lookup_historical_equity_rate(
                        account_code,
                        historical_rates,
                        closing_rate,
                    ),
                    TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                        // Income statement items use average rate under temporal method
                        average_rate
                    }
                    TranslationAccountType::Asset | TranslationAccountType::Liability => {
                        // Distinguish monetary vs non-monetary for balance sheet items
                        if is_monetary(account_code) {
                            closing_rate
                        } else {
                            // Non-monetary items use historical rate if available,
                            // otherwise fall back to closing rate
                            historical_rates
                                .get(account_code)
                                .copied()
                                .unwrap_or(closing_rate)
                        }
                    }
                }
            }
            TranslationMethod::MonetaryNonMonetary => {
                // Monetary/Non-monetary method: similar to temporal but focused
                // specifically on the monetary vs non-monetary distinction.
                // Equity accounts use historical rates.
                match account_type {
                    TranslationAccountType::CommonStock
                    | TranslationAccountType::AdditionalPaidInCapital
                    | TranslationAccountType::RetainedEarnings => self
                        .lookup_historical_equity_rate(
                            account_code,
                            historical_rates,
                            closing_rate,
                        ),
                    TranslationAccountType::Equity => self.lookup_historical_equity_rate(
                        account_code,
                        historical_rates,
                        closing_rate,
                    ),
                    _ => {
                        // For all other accounts, classify based on monetary nature
                        if is_monetary(account_code) {
                            closing_rate
                        } else {
                            // Non-monetary items use historical rate if available
                            historical_rates
                                .get(account_code)
                                .copied()
                                .unwrap_or(closing_rate)
                        }
                    }
                }
            }
        }
    }

    /// Returns the rate type for a given account type.
    fn rate_type_for_account(&self, account_type: &TranslationAccountType) -> RateType {
        match account_type {
            TranslationAccountType::Asset | TranslationAccountType::Liability => RateType::Closing,
            TranslationAccountType::Revenue | TranslationAccountType::Expense => RateType::Average,
            TranslationAccountType::Equity
            | TranslationAccountType::CommonStock
            | TranslationAccountType::AdditionalPaidInCapital
            | TranslationAccountType::RetainedEarnings => RateType::Historical,
        }
    }
}

/// Translated trial balance in group currency.
#[derive(Debug, Clone)]
pub struct TranslatedTrialBalance {
    /// Company code.
    pub company_code: String,
    /// Company name.
    pub company_name: String,
    /// Local (functional) currency.
    pub local_currency: String,
    /// Group (reporting) currency.
    pub group_currency: String,
    /// Period end date.
    pub period_end_date: NaiveDate,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Translated line items.
    pub lines: Vec<TranslatedTrialBalanceLine>,
    /// Closing rate used.
    pub closing_rate: Decimal,
    /// Average rate used.
    pub average_rate: Decimal,
    /// Total local currency debits.
    pub total_local_debit: Decimal,
    /// Total local currency credits.
    pub total_local_credit: Decimal,
    /// Total group currency debits.
    pub total_group_debit: Decimal,
    /// Total group currency credits.
    pub total_group_credit: Decimal,
    /// Currency Translation Adjustment amount.
    pub cta_amount: Decimal,
    /// Translation method used.
    pub translation_method: TranslationMethod,
}

impl TranslatedTrialBalance {
    /// Returns true if the local currency trial balance is balanced.
    pub fn is_local_balanced(&self) -> bool {
        (self.total_local_debit - self.total_local_credit).abs() < dec!(0.01)
    }

    /// Returns true if the group currency trial balance is balanced (including CTA).
    pub fn is_group_balanced(&self) -> bool {
        let balance = self.total_group_debit - self.total_group_credit - self.cta_amount;
        balance.abs() < dec!(0.01)
    }

    /// Gets the net assets in local currency.
    pub fn local_net_assets(&self) -> Decimal {
        let assets: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Asset))
            .map(|l| l.local_debit - l.local_credit)
            .sum();

        let liabilities: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Liability))
            .map(|l| l.local_credit - l.local_debit)
            .sum();

        assets - liabilities
    }

    /// Gets the net assets in group currency.
    pub fn group_net_assets(&self) -> Decimal {
        let assets: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Asset))
            .map(|l| l.group_debit - l.group_credit)
            .sum();

        let liabilities: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Liability))
            .map(|l| l.group_credit - l.group_debit)
            .sum();

        assets - liabilities
    }
}

/// A line in a translated trial balance.
#[derive(Debug, Clone)]
pub struct TranslatedTrialBalanceLine {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: Option<String>,
    /// Account type for translation.
    pub account_type: TranslationAccountType,
    /// Debit balance in local currency.
    pub local_debit: Decimal,
    /// Credit balance in local currency.
    pub local_credit: Decimal,
    /// Exchange rate used.
    pub rate_used: Decimal,
    /// Rate type used.
    pub rate_type: RateType,
    /// Debit balance in group currency.
    pub group_debit: Decimal,
    /// Credit balance in group currency.
    pub group_credit: Decimal,
}

impl TranslatedTrialBalanceLine {
    /// Gets the net balance in local currency.
    pub fn local_net(&self) -> Decimal {
        self.local_debit - self.local_credit
    }

    /// Gets the net balance in group currency.
    pub fn group_net(&self) -> Decimal {
        self.group_debit - self.group_credit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::models::balance::{
        AccountCategory, AccountType, TrialBalanceLine, TrialBalanceType,
    };
    use datasynth_core::models::FxRate;

    fn create_test_trial_balance() -> TrialBalance {
        let mut tb = TrialBalance::new(
            "TB-TEST-2024-12".to_string(),
            "1200".to_string(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            2024,
            12,
            "EUR".to_string(),
            TrialBalanceType::PostClosing,
        );
        tb.company_name = Some("Test Subsidiary".to_string());

        tb.add_line(TrialBalanceLine {
            account_code: "1000".to_string(),
            account_description: "Cash".to_string(),
            category: AccountCategory::CurrentAssets,
            account_type: AccountType::Asset,
            opening_balance: Decimal::ZERO,
            period_debits: dec!(100000),
            period_credits: Decimal::ZERO,
            closing_balance: dec!(100000),
            debit_balance: dec!(100000),
            credit_balance: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
        });

        tb.add_line(TrialBalanceLine {
            account_code: "2000".to_string(),
            account_description: "Accounts Payable".to_string(),
            category: AccountCategory::CurrentLiabilities,
            account_type: AccountType::Liability,
            opening_balance: Decimal::ZERO,
            period_debits: Decimal::ZERO,
            period_credits: dec!(50000),
            closing_balance: dec!(50000),
            debit_balance: Decimal::ZERO,
            credit_balance: dec!(50000),
            cost_center: None,
            profit_center: None,
        });

        tb.add_line(TrialBalanceLine {
            account_code: "4000".to_string(),
            account_description: "Revenue".to_string(),
            category: AccountCategory::Revenue,
            account_type: AccountType::Revenue,
            opening_balance: Decimal::ZERO,
            period_debits: Decimal::ZERO,
            period_credits: dec!(150000),
            closing_balance: dec!(150000),
            debit_balance: Decimal::ZERO,
            credit_balance: dec!(150000),
            cost_center: None,
            profit_center: None,
        });

        tb.add_line(TrialBalanceLine {
            account_code: "5000".to_string(),
            account_description: "Expenses".to_string(),
            category: AccountCategory::OperatingExpenses,
            account_type: AccountType::Expense,
            opening_balance: Decimal::ZERO,
            period_debits: dec!(100000),
            period_credits: Decimal::ZERO,
            closing_balance: dec!(100000),
            debit_balance: dec!(100000),
            credit_balance: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
        });

        tb
    }

    #[test]
    fn test_translate_trial_balance() {
        let translator = CurrencyTranslator::new(CurrencyTranslatorConfig::default());
        let trial_balance = create_test_trial_balance();

        let mut rate_table = FxRateTable::new("USD");
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Closing,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.10),
            "TEST",
        ));
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Average,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.08),
            "TEST",
        ));

        let historical_rates = HashMap::new();
        let translated =
            translator.translate_trial_balance(&trial_balance, &rate_table, &historical_rates);

        assert!(translated.is_local_balanced());
        assert_eq!(translated.closing_rate, dec!(1.10));
        assert_eq!(translated.average_rate, dec!(1.08));
    }

    #[test]
    fn test_is_monetary() {
        // Cash and cash equivalents - monetary
        assert!(is_monetary("1000"));
        assert!(is_monetary("1001"));
        assert!(is_monetary("1099"));

        // Accounts receivable - monetary
        assert!(is_monetary("1100"));
        assert!(is_monetary("1150"));

        // Short-term investments - monetary
        assert!(is_monetary("1200"));

        // Notes receivable - monetary
        assert!(is_monetary("1300"));

        // Prepaid expenses - non-monetary
        assert!(!is_monetary("1400"));
        assert!(!is_monetary("1450"));

        // Inventory - non-monetary
        assert!(!is_monetary("1500"));
        assert!(!is_monetary("1550"));

        // PP&E - non-monetary
        assert!(!is_monetary("1600"));
        assert!(!is_monetary("1650"));

        // Intangible assets - non-monetary
        assert!(!is_monetary("1700"));

        // Long-term investments - non-monetary
        assert!(!is_monetary("1800"));

        // Other non-current assets - non-monetary
        assert!(!is_monetary("1900"));

        // Liabilities - all monetary
        assert!(is_monetary("2000"));
        assert!(is_monetary("2100"));
        assert!(is_monetary("2500"));
        assert!(is_monetary("2900"));

        // Equity - non-monetary
        assert!(!is_monetary("3000"));
        assert!(!is_monetary("3100"));
        assert!(!is_monetary("3200"));
        assert!(!is_monetary("3900"));

        // Revenue - treated as monetary (average rate in temporal)
        assert!(is_monetary("4000"));
        assert!(is_monetary("4500"));

        // Expenses - treated as monetary (average rate in temporal)
        assert!(is_monetary("5000"));
        assert!(is_monetary("6000"));

        // Short codes default to monetary
        assert!(is_monetary("1"));
        assert!(is_monetary(""));
    }

    #[test]
    fn test_historical_equity_rates() {
        let mut translator = CurrencyTranslator::new(CurrencyTranslatorConfig::default());

        // Initially empty
        assert!(translator.historical_equity_rates.is_empty());

        // Set historical equity rates
        let mut rates = HashMap::new();
        rates.insert("3100".to_string(), dec!(1.05));
        rates.insert("3200".to_string(), dec!(0.98));
        translator.set_historical_equity_rates(rates);

        assert_eq!(translator.historical_equity_rates.len(), 2);
        assert_eq!(
            translator.historical_equity_rates.get("3100"),
            Some(&dec!(1.05))
        );
        assert_eq!(
            translator.historical_equity_rates.get("3200"),
            Some(&dec!(0.98))
        );

        // Verify these rates are used in determine_rate for CurrentRate method
        let rate = translator.determine_rate(
            "3100",
            &TranslationAccountType::CommonStock,
            dec!(1.10),
            dec!(1.08),
            &HashMap::new(),
        );
        assert_eq!(rate, dec!(1.05));
    }

    #[test]
    fn test_temporal_method_monetary_vs_non_monetary() {
        let config = CurrencyTranslatorConfig {
            method: TranslationMethod::Temporal,
            ..CurrencyTranslatorConfig::default()
        };
        let translator = CurrencyTranslator::new(config);

        let closing_rate = dec!(1.10);
        let average_rate = dec!(1.08);
        let mut historical_rates = HashMap::new();
        historical_rates.insert("1500".to_string(), dec!(1.02)); // historical rate for inventory

        // Cash (1000) is monetary -> closing rate
        let rate = translator.determine_rate(
            "1000",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Accounts receivable (1100) is monetary -> closing rate
        let rate = translator.determine_rate(
            "1100",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Inventory (1500) is non-monetary -> historical rate
        let rate = translator.determine_rate(
            "1500",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(1.02));

        // PP&E (1600) is non-monetary -> falls back to closing rate (no historical rate set)
        let rate = translator.determine_rate(
            "1600",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Liabilities (2000) are monetary -> closing rate
        let rate = translator.determine_rate(
            "2000",
            &TranslationAccountType::Liability,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Revenue (4000) -> average rate under temporal method
        let rate = translator.determine_rate(
            "4000",
            &TranslationAccountType::Revenue,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, average_rate);

        // Expenses (5000) -> average rate under temporal method
        let rate = translator.determine_rate(
            "5000",
            &TranslationAccountType::Expense,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, average_rate);

        // Equity accounts -> historical equity rate (via lookup)
        let mut translator_with_equity = CurrencyTranslator::new(CurrencyTranslatorConfig {
            method: TranslationMethod::Temporal,
            ..CurrencyTranslatorConfig::default()
        });
        let mut equity_rates = HashMap::new();
        equity_rates.insert("3100".to_string(), dec!(0.95));
        translator_with_equity.set_historical_equity_rates(equity_rates);

        let rate = translator_with_equity.determine_rate(
            "3100",
            &TranslationAccountType::CommonStock,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(0.95));
    }

    #[test]
    fn test_monetary_non_monetary_method() {
        let config = CurrencyTranslatorConfig {
            method: TranslationMethod::MonetaryNonMonetary,
            ..CurrencyTranslatorConfig::default()
        };
        let mut translator = CurrencyTranslator::new(config);

        let closing_rate = dec!(1.10);
        let average_rate = dec!(1.08);
        let mut historical_rates = HashMap::new();
        historical_rates.insert("1500".to_string(), dec!(1.02));
        historical_rates.insert("1600".to_string(), dec!(0.99));

        // Set historical equity rates
        let mut equity_rates = HashMap::new();
        equity_rates.insert("3100".to_string(), dec!(1.05));
        equity_rates.insert("3300".to_string(), dec!(1.03));
        translator.set_historical_equity_rates(equity_rates);

        // Cash (1000) is monetary -> closing rate
        let rate = translator.determine_rate(
            "1000",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Accounts receivable (1100) is monetary -> closing rate
        let rate = translator.determine_rate(
            "1100",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Inventory (1500) is non-monetary -> historical rate
        let rate = translator.determine_rate(
            "1500",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(1.02));

        // PP&E (1600) is non-monetary -> historical rate
        let rate = translator.determine_rate(
            "1600",
            &TranslationAccountType::Asset,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(0.99));

        // Liabilities (2000) are monetary -> closing rate
        let rate = translator.determine_rate(
            "2000",
            &TranslationAccountType::Liability,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Equity: Common Stock (3100) -> uses historical equity rate from instance
        let rate = translator.determine_rate(
            "3100",
            &TranslationAccountType::CommonStock,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(1.05));

        // Equity: Retained Earnings (3300) -> uses historical equity rate from instance
        let rate = translator.determine_rate(
            "3300",
            &TranslationAccountType::RetainedEarnings,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, dec!(1.03));

        // Revenue (4000) is monetary -> closing rate (not average like temporal)
        let rate = translator.determine_rate(
            "4000",
            &TranslationAccountType::Revenue,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);

        // Expenses (5000) is monetary -> closing rate
        let rate = translator.determine_rate(
            "5000",
            &TranslationAccountType::Expense,
            closing_rate,
            average_rate,
            &historical_rates,
        );
        assert_eq!(rate, closing_rate);
    }
}
