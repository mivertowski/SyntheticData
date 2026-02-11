//! Currency Translation Adjustment (CTA) generator.
//!
//! Generates CTA entries for subsidiaries with foreign functional currencies.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use datasynth_core::models::{CTAEntry, FxRateTable, JournalEntry, JournalEntryLine};

use super::currency_translator::TranslatedTrialBalance;

/// Configuration for CTA generation.
#[derive(Debug, Clone)]
pub struct CTAGeneratorConfig {
    /// Group (reporting) currency.
    pub group_currency: String,
    /// CTA account code for posting.
    pub cta_account: String,
    /// Accumulated CTA account (OCI).
    pub accumulated_cta_account: String,
    /// Whether to generate detailed CTA breakdown.
    pub generate_detailed_breakdown: bool,
}

impl Default for CTAGeneratorConfig {
    fn default() -> Self {
        Self {
            group_currency: "USD".to_string(),
            cta_account: "3900".to_string(),
            accumulated_cta_account: "3910".to_string(),
            generate_detailed_breakdown: true,
        }
    }
}

/// Generator for Currency Translation Adjustment entries.
pub struct CTAGenerator {
    config: CTAGeneratorConfig,
    cta_counter: u64,
}

impl CTAGenerator {
    /// Creates a new CTA generator.
    pub fn new(config: CTAGeneratorConfig) -> Self {
        Self {
            config,
            cta_counter: 0,
        }
    }

    /// Generates a CTA entry for a subsidiary for a period.
    pub fn generate_cta(
        &mut self,
        company_code: &str,
        local_currency: &str,
        fiscal_year: i32,
        fiscal_period: u8,
        period_end_date: NaiveDate,
        opening_net_assets_local: Decimal,
        closing_net_assets_local: Decimal,
        net_income_local: Decimal,
        rate_table: &FxRateTable,
        opening_rate: Option<Decimal>,
    ) -> (CTAEntry, JournalEntry) {
        self.cta_counter += 1;
        let entry_id = format!("CTA-{:08}", self.cta_counter);

        // Get rates
        let closing_rate = rate_table
            .get_closing_rate(local_currency, &self.config.group_currency, period_end_date)
            .map(|r| r.rate)
            .unwrap_or(Decimal::ONE);

        let average_rate = rate_table
            .get_average_rate(local_currency, &self.config.group_currency, period_end_date)
            .map(|r| r.rate)
            .unwrap_or(closing_rate);

        let opening_rate = opening_rate.unwrap_or(closing_rate);

        let mut cta = CTAEntry::new(
            entry_id.clone(),
            company_code.to_string(),
            local_currency.to_string(),
            self.config.group_currency.clone(),
            fiscal_year,
            fiscal_period,
            period_end_date,
        );

        cta.opening_net_assets_local = opening_net_assets_local;
        cta.closing_net_assets_local = closing_net_assets_local;
        cta.net_income_local = net_income_local;
        cta.opening_rate = opening_rate;
        cta.closing_rate = closing_rate;
        cta.average_rate = average_rate;

        // Calculate CTA using current rate method
        cta.calculate_current_rate_method();

        // Generate journal entry
        let je = self.generate_cta_journal_entry(&cta);

        (cta, je)
    }

    /// Generates CTA from translated trial balance comparison.
    pub fn generate_cta_from_translation(
        &mut self,
        current_period: &TranslatedTrialBalance,
        prior_period: Option<&TranslatedTrialBalance>,
        net_income_local: Decimal,
    ) -> (CTAEntry, JournalEntry) {
        self.cta_counter += 1;
        let entry_id = format!("CTA-{:08}", self.cta_counter);

        let opening_net_assets = prior_period
            .map(|tb| tb.local_net_assets())
            .unwrap_or(Decimal::ZERO);

        let opening_rate = prior_period
            .map(|tb| tb.closing_rate)
            .unwrap_or(current_period.closing_rate);

        let mut cta = CTAEntry::new(
            entry_id.clone(),
            current_period.company_code.clone(),
            current_period.local_currency.clone(),
            current_period.group_currency.clone(),
            current_period.fiscal_year,
            current_period.fiscal_period,
            current_period.period_end_date,
        );

        cta.opening_net_assets_local = opening_net_assets;
        cta.closing_net_assets_local = current_period.local_net_assets();
        cta.net_income_local = net_income_local;
        cta.opening_rate = opening_rate;
        cta.closing_rate = current_period.closing_rate;
        cta.average_rate = current_period.average_rate;

        cta.calculate_current_rate_method();

        let je = self.generate_cta_journal_entry(&cta);

        (cta, je)
    }

    /// Generates CTA entries for multiple subsidiaries.
    pub fn generate_cta_for_subsidiaries(
        &mut self,
        subsidiaries: &[SubsidiaryCTAInput],
        rate_table: &FxRateTable,
    ) -> Vec<(CTAEntry, JournalEntry)> {
        subsidiaries
            .iter()
            .map(|sub| {
                self.generate_cta(
                    &sub.company_code,
                    &sub.local_currency,
                    sub.fiscal_year,
                    sub.fiscal_period,
                    sub.period_end_date,
                    sub.opening_net_assets_local,
                    sub.closing_net_assets_local,
                    sub.net_income_local,
                    rate_table,
                    sub.opening_rate,
                )
            })
            .collect()
    }

    /// Generates the journal entry for a CTA.
    fn generate_cta_journal_entry(&self, cta: &CTAEntry) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", cta.entry_id),
            cta.company_code.clone(),
            cta.period_end_date,
            format!(
                "CTA {} Period {}/{}",
                cta.company_code, cta.fiscal_year, cta.fiscal_period
            ),
        );

        if cta.cta_amount >= Decimal::ZERO {
            // CTA gain (credit to CTA, debit to plug)
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: self.config.accumulated_cta_account.clone(),
                debit_amount: cta.cta_amount,
                reference: Some(cta.entry_id.clone()),
                text: Some("CTA - Net Assets Translation".to_string()),
                ..Default::default()
            });

            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: self.config.cta_account.clone(),
                credit_amount: cta.cta_amount,
                reference: Some(cta.entry_id.clone()),
                text: Some(format!(
                    "CTA: {} @ {} -> {}",
                    cta.local_currency, cta.closing_rate, cta.group_currency
                )),
                ..Default::default()
            });
        } else {
            // CTA loss (debit to CTA, credit to plug)
            let abs_amount = cta.cta_amount.abs();

            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: self.config.cta_account.clone(),
                debit_amount: abs_amount,
                reference: Some(cta.entry_id.clone()),
                text: Some(format!(
                    "CTA: {} @ {} -> {}",
                    cta.local_currency, cta.closing_rate, cta.group_currency
                )),
                ..Default::default()
            });

            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: self.config.accumulated_cta_account.clone(),
                credit_amount: abs_amount,
                reference: Some(cta.entry_id.clone()),
                text: Some("CTA - Net Assets Translation".to_string()),
                ..Default::default()
            });
        }

        je
    }
}

/// Input data for subsidiary CTA calculation.
#[derive(Debug, Clone)]
pub struct SubsidiaryCTAInput {
    /// Company code.
    pub company_code: String,
    /// Local (functional) currency.
    pub local_currency: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Period end date.
    pub period_end_date: NaiveDate,
    /// Opening net assets in local currency.
    pub opening_net_assets_local: Decimal,
    /// Closing net assets in local currency.
    pub closing_net_assets_local: Decimal,
    /// Net income for the period in local currency.
    pub net_income_local: Decimal,
    /// Opening exchange rate (prior period closing rate).
    pub opening_rate: Option<Decimal>,
}

/// Summary of CTA across all subsidiaries.
#[derive(Debug, Clone)]
pub struct CTASummary {
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Period end date.
    pub period_end_date: NaiveDate,
    /// Group currency.
    pub group_currency: String,
    /// CTA entries by subsidiary.
    pub entries: Vec<CTAEntry>,
    /// Total CTA (sum across all subsidiaries).
    pub total_cta: Decimal,
    /// CTA by currency.
    pub cta_by_currency: HashMap<String, Decimal>,
}

impl CTASummary {
    /// Creates a new CTA summary from entries.
    pub fn from_entries(
        entries: Vec<CTAEntry>,
        fiscal_year: i32,
        fiscal_period: u8,
        period_end_date: NaiveDate,
        group_currency: String,
    ) -> Self {
        let total_cta: Decimal = entries.iter().map(|e| e.cta_amount).sum();

        let mut cta_by_currency: HashMap<String, Decimal> = HashMap::new();
        for entry in &entries {
            *cta_by_currency
                .entry(entry.local_currency.clone())
                .or_insert(Decimal::ZERO) += entry.cta_amount;
        }

        Self {
            fiscal_year,
            fiscal_period,
            period_end_date,
            group_currency,
            entries,
            total_cta,
            cta_by_currency,
        }
    }

    /// Returns a summary string.
    pub fn summary(&self) -> String {
        let mut summary = format!(
            "CTA Summary for Period {}/{} ending {}\n",
            self.fiscal_year, self.fiscal_period, self.period_end_date
        );
        summary.push_str(&format!(
            "Total CTA: {} {}\n",
            self.total_cta, self.group_currency
        ));
        summary.push_str("By Currency:\n");
        for (currency, amount) in &self.cta_by_currency {
            summary.push_str(&format!(
                "  {}: {} {}\n",
                currency, amount, self.group_currency
            ));
        }
        summary
    }
}

/// Detailed CTA analysis for a single subsidiary.
#[derive(Debug, Clone)]
pub struct CTAAnalysis {
    /// CTA entry.
    pub entry: CTAEntry,
    /// Translation of balance sheet impact.
    pub balance_sheet_impact: Decimal,
    /// Translation of income statement impact.
    pub income_statement_impact: Decimal,
    /// Rate change impact on opening net assets.
    pub rate_change_impact: Decimal,
    /// Breakdown by component.
    pub breakdown: Vec<CTABreakdownItem>,
}

/// CTA breakdown item for detailed analysis.
#[derive(Debug, Clone)]
pub struct CTABreakdownItem {
    /// Description of the item.
    pub description: String,
    /// Local currency amount.
    pub local_amount: Decimal,
    /// Rate used.
    pub rate: Decimal,
    /// Group currency amount.
    pub group_amount: Decimal,
    /// Impact on CTA.
    pub cta_impact: Decimal,
}

impl CTAAnalysis {
    /// Creates a detailed CTA analysis from an entry.
    pub fn from_entry(entry: CTAEntry) -> Self {
        // Calculate impacts
        let opening_at_opening = entry.opening_net_assets_local * entry.opening_rate;
        let opening_at_closing = entry.opening_net_assets_local * entry.closing_rate;
        let rate_change_impact = opening_at_closing - opening_at_opening;

        let income_at_average = entry.net_income_local * entry.average_rate;
        let income_at_closing = entry.net_income_local * entry.closing_rate;
        let income_statement_impact = income_at_closing - income_at_average;

        let balance_sheet_impact = entry.cta_amount - income_statement_impact;

        let breakdown = vec![
            CTABreakdownItem {
                description: "Opening net assets rate change".to_string(),
                local_amount: entry.opening_net_assets_local,
                rate: entry.closing_rate - entry.opening_rate,
                group_amount: rate_change_impact,
                cta_impact: rate_change_impact,
            },
            CTABreakdownItem {
                description: "Net income translation difference".to_string(),
                local_amount: entry.net_income_local,
                rate: entry.closing_rate - entry.average_rate,
                group_amount: income_statement_impact,
                cta_impact: income_statement_impact,
            },
        ];

        Self {
            entry,
            balance_sheet_impact,
            income_statement_impact,
            rate_change_impact,
            breakdown,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{FxRate, RateType};
    use rust_decimal_macros::dec;

    #[test]
    fn test_generate_cta() {
        let mut generator = CTAGenerator::new(CTAGeneratorConfig::default());

        let mut rate_table = FxRateTable::new("USD");
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Closing,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.12),
            "TEST",
        ));
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Average,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.10),
            "TEST",
        ));

        let (cta, je) = generator.generate_cta(
            "1200",
            "EUR",
            2024,
            12,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1000000), // Opening net assets
            dec!(1100000), // Closing net assets
            dec!(100000),  // Net income
            &rate_table,
            Some(dec!(1.08)), // Opening rate
        );

        assert!(je.is_balanced());
        assert_eq!(cta.closing_rate, dec!(1.12));
        assert_eq!(cta.average_rate, dec!(1.10));

        // CTA = 1,100,000 × 1.12 - 1,000,000 × 1.08 - 100,000 × 1.10
        // CTA = 1,232,000 - 1,080,000 - 110,000 = 42,000
        assert_eq!(cta.cta_amount, dec!(42000));
    }

    #[test]
    fn test_cta_summary() {
        let entries = vec![
            CTAEntry {
                entry_id: "CTA-001".to_string(),
                company_code: "1200".to_string(),
                local_currency: "EUR".to_string(),
                group_currency: "USD".to_string(),
                fiscal_year: 2024,
                fiscal_period: 12,
                period_end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                cta_amount: dec!(42000),
                opening_rate: dec!(1.08),
                closing_rate: dec!(1.12),
                average_rate: dec!(1.10),
                opening_net_assets_local: dec!(1000000),
                closing_net_assets_local: dec!(1100000),
                net_income_local: dec!(100000),
                components: Vec::new(),
            },
            CTAEntry {
                entry_id: "CTA-002".to_string(),
                company_code: "1300".to_string(),
                local_currency: "GBP".to_string(),
                group_currency: "USD".to_string(),
                fiscal_year: 2024,
                fiscal_period: 12,
                period_end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                cta_amount: dec!(-15000),
                opening_rate: dec!(1.30),
                closing_rate: dec!(1.27),
                average_rate: dec!(1.28),
                opening_net_assets_local: dec!(500000),
                closing_net_assets_local: dec!(550000),
                net_income_local: dec!(50000),
                components: Vec::new(),
            },
        ];

        let summary = CTASummary::from_entries(
            entries,
            2024,
            12,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            "USD".to_string(),
        );

        assert_eq!(summary.total_cta, dec!(27000)); // 42,000 - 15,000
        assert_eq!(summary.cta_by_currency.get("EUR"), Some(&dec!(42000)));
        assert_eq!(summary.cta_by_currency.get("GBP"), Some(&dec!(-15000)));
    }
}
