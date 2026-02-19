//! Accrual entry generator.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::debug;

use datasynth_core::models::{
    AccrualCalculationMethod, AccrualDefinition, AccrualFrequency, AccrualType, FiscalPeriod,
    JournalEntry, JournalEntryLine,
};

/// Configuration for accrual generation.
#[derive(Debug, Clone)]
pub struct AccrualGeneratorConfig {
    /// Whether to generate reversal entries.
    pub generate_reversals: bool,
    /// Days after period end to post reversals.
    pub reversal_days_offset: i64,
    /// Default document type for accruals.
    pub document_type: String,
}

impl Default for AccrualGeneratorConfig {
    fn default() -> Self {
        Self {
            generate_reversals: true,
            reversal_days_offset: 1,
            document_type: "SA".to_string(), // Standard Accrual
        }
    }
}

/// Generator for accrual entries.
pub struct AccrualGenerator {
    config: AccrualGeneratorConfig,
    accrual_counter: u64,
}

impl AccrualGenerator {
    /// Creates a new accrual generator.
    pub fn new(config: AccrualGeneratorConfig) -> Self {
        Self {
            config,
            accrual_counter: 0,
        }
    }

    /// Generates accrual entries for a period.
    pub fn generate_accruals(
        &mut self,
        definitions: &[AccrualDefinition],
        fiscal_period: &FiscalPeriod,
        account_balances: &HashMap<String, Decimal>,
    ) -> AccrualGenerationResult {
        debug!(
            definition_count = definitions.len(),
            period = fiscal_period.period,
            year = fiscal_period.year,
            "Generating accruals"
        );
        let mut result = AccrualGenerationResult {
            period: fiscal_period.clone(),
            accrual_entries: Vec::new(),
            reversal_entries: Vec::new(),
            total_accrued_expenses: Decimal::ZERO,
            total_accrued_revenue: Decimal::ZERO,
            skipped_definitions: Vec::new(),
        };

        for definition in definitions {
            // Check if definition is active for this period
            if !definition.is_effective_on(fiscal_period.end_date) {
                result.skipped_definitions.push(SkippedAccrual {
                    accrual_id: definition.accrual_id.clone(),
                    reason: "Not effective for this period".to_string(),
                });
                continue;
            }

            // Check frequency
            if !self.should_accrue(definition, fiscal_period) {
                result.skipped_definitions.push(SkippedAccrual {
                    accrual_id: definition.accrual_id.clone(),
                    reason: "Frequency does not match this period".to_string(),
                });
                continue;
            }

            // Calculate accrual amount
            let amount = self.calculate_amount(definition, fiscal_period, account_balances);
            if amount == Decimal::ZERO {
                result.skipped_definitions.push(SkippedAccrual {
                    accrual_id: definition.accrual_id.clone(),
                    reason: "Calculated amount is zero".to_string(),
                });
                continue;
            }

            // Generate accrual entry
            let (accrual_je, reversal_je) =
                self.generate_accrual_entry(definition, fiscal_period, amount);

            // Track totals
            match definition.accrual_type {
                AccrualType::AccruedExpense => result.total_accrued_expenses += amount,
                AccrualType::AccruedRevenue => result.total_accrued_revenue += amount,
                _ => {}
            }

            result.accrual_entries.push(accrual_je);
            if let Some(rev) = reversal_je {
                result.reversal_entries.push(rev);
            }
        }

        result
    }

    /// Generates a single accrued expense entry.
    pub fn generate_accrued_expense(
        &mut self,
        company_code: &str,
        description: &str,
        amount: Decimal,
        expense_account: &str,
        liability_account: &str,
        posting_date: NaiveDate,
        cost_center: Option<&str>,
    ) -> (JournalEntry, Option<JournalEntry>) {
        self.accrual_counter += 1;
        let doc_number = format!("ACCR{:08}", self.accrual_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            posting_date,
            format!("Accrued Expense: {}", description),
        );

        // Debit Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: expense_account.to_string(),
            debit_amount: amount,
            cost_center: cost_center.map(|s| s.to_string()),
            reference: Some(doc_number.clone()),
            text: Some(description.to_string()),
            ..Default::default()
        });

        // Credit Accrued Liability
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: liability_account.to_string(),
            credit_amount: amount,
            reference: Some(doc_number.clone()),
            ..Default::default()
        });

        // Generate reversal if configured
        let reversal = if self.config.generate_reversals {
            let reversal_date = posting_date
                .checked_add_signed(chrono::Duration::days(self.config.reversal_days_offset))
                .unwrap_or(posting_date);
            Some(self.generate_reversal(&je, reversal_date))
        } else {
            None
        };

        (je, reversal)
    }

    /// Generates a single accrued revenue entry.
    pub fn generate_accrued_revenue(
        &mut self,
        company_code: &str,
        description: &str,
        amount: Decimal,
        revenue_account: &str,
        asset_account: &str,
        posting_date: NaiveDate,
        cost_center: Option<&str>,
    ) -> (JournalEntry, Option<JournalEntry>) {
        self.accrual_counter += 1;
        let doc_number = format!("ACCR{:08}", self.accrual_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            posting_date,
            format!("Accrued Revenue: {}", description),
        );

        // Debit Accrued Revenue Receivable
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: asset_account.to_string(),
            debit_amount: amount,
            reference: Some(doc_number.clone()),
            text: Some(description.to_string()),
            ..Default::default()
        });

        // Credit Revenue
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: revenue_account.to_string(),
            credit_amount: amount,
            cost_center: cost_center.map(|s| s.to_string()),
            reference: Some(doc_number.clone()),
            ..Default::default()
        });

        // Generate reversal if configured
        let reversal = if self.config.generate_reversals {
            let reversal_date = posting_date
                .checked_add_signed(chrono::Duration::days(self.config.reversal_days_offset))
                .unwrap_or(posting_date);
            Some(self.generate_reversal(&je, reversal_date))
        } else {
            None
        };

        (je, reversal)
    }

    /// Generates a prepaid expense amortization entry.
    pub fn generate_prepaid_amortization(
        &mut self,
        company_code: &str,
        description: &str,
        amount: Decimal,
        expense_account: &str,
        prepaid_account: &str,
        posting_date: NaiveDate,
        cost_center: Option<&str>,
    ) -> JournalEntry {
        self.accrual_counter += 1;
        let doc_number = format!("PREP{:08}", self.accrual_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            posting_date,
            format!("Prepaid Amortization: {}", description),
        );

        // Debit Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: expense_account.to_string(),
            debit_amount: amount,
            cost_center: cost_center.map(|s| s.to_string()),
            reference: Some(doc_number.clone()),
            text: Some(description.to_string()),
            ..Default::default()
        });

        // Credit Prepaid Asset
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: prepaid_account.to_string(),
            credit_amount: amount,
            reference: Some(doc_number.clone()),
            ..Default::default()
        });

        je
    }

    /// Generates deferred revenue recognition entry.
    pub fn generate_deferred_revenue_recognition(
        &mut self,
        company_code: &str,
        description: &str,
        amount: Decimal,
        revenue_account: &str,
        deferred_account: &str,
        posting_date: NaiveDate,
        cost_center: Option<&str>,
    ) -> JournalEntry {
        self.accrual_counter += 1;
        let doc_number = format!("DEFR{:08}", self.accrual_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            posting_date,
            format!("Deferred Revenue Recognition: {}", description),
        );

        // Debit Deferred Revenue (Liability)
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: deferred_account.to_string(),
            debit_amount: amount,
            reference: Some(doc_number.clone()),
            text: Some(description.to_string()),
            ..Default::default()
        });

        // Credit Revenue
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: revenue_account.to_string(),
            credit_amount: amount,
            cost_center: cost_center.map(|s| s.to_string()),
            reference: Some(doc_number.clone()),
            ..Default::default()
        });

        je
    }

    fn should_accrue(&self, definition: &AccrualDefinition, period: &FiscalPeriod) -> bool {
        match definition.frequency {
            AccrualFrequency::Monthly => true,
            AccrualFrequency::Quarterly => period.period.is_multiple_of(3),
            AccrualFrequency::Annually => period.is_year_end,
        }
    }

    fn calculate_amount(
        &self,
        definition: &AccrualDefinition,
        period: &FiscalPeriod,
        account_balances: &HashMap<String, Decimal>,
    ) -> Decimal {
        match definition.calculation_method {
            AccrualCalculationMethod::FixedAmount => {
                definition.fixed_amount.unwrap_or(Decimal::ZERO)
            }
            AccrualCalculationMethod::PercentageOfBase => {
                if let (Some(rate), Some(base_account)) =
                    (definition.percentage_rate, &definition.base_account)
                {
                    let base = account_balances
                        .get(base_account)
                        .copied()
                        .unwrap_or(Decimal::ZERO);
                    (base * rate / dec!(100)).round_dp(2)
                } else {
                    Decimal::ZERO
                }
            }
            AccrualCalculationMethod::DaysBased => {
                // Prorate based on days in period
                if let Some(annual_amount) = definition.fixed_amount {
                    let daily = annual_amount / dec!(365);
                    (daily * Decimal::from(period.days())).round_dp(2)
                } else {
                    Decimal::ZERO
                }
            }
            AccrualCalculationMethod::Manual => Decimal::ZERO,
        }
    }

    fn generate_accrual_entry(
        &mut self,
        definition: &AccrualDefinition,
        period: &FiscalPeriod,
        amount: Decimal,
    ) -> (JournalEntry, Option<JournalEntry>) {
        match definition.accrual_type {
            AccrualType::AccruedExpense => self.generate_accrued_expense(
                &definition.company_code,
                &definition.description,
                amount,
                &definition.expense_revenue_account,
                &definition.accrual_account,
                period.end_date,
                definition.cost_center.as_deref(),
            ),
            AccrualType::AccruedRevenue => self.generate_accrued_revenue(
                &definition.company_code,
                &definition.description,
                amount,
                &definition.expense_revenue_account,
                &definition.accrual_account,
                period.end_date,
                definition.cost_center.as_deref(),
            ),
            AccrualType::PrepaidExpense => {
                let je = self.generate_prepaid_amortization(
                    &definition.company_code,
                    &definition.description,
                    amount,
                    &definition.expense_revenue_account,
                    &definition.accrual_account,
                    period.end_date,
                    definition.cost_center.as_deref(),
                );
                (je, None) // No reversal for prepaid amortization
            }
            AccrualType::DeferredRevenue => {
                let je = self.generate_deferred_revenue_recognition(
                    &definition.company_code,
                    &definition.description,
                    amount,
                    &definition.expense_revenue_account,
                    &definition.accrual_account,
                    period.end_date,
                    definition.cost_center.as_deref(),
                );
                (je, None) // No reversal for deferred revenue recognition
            }
        }
    }

    fn generate_reversal(
        &mut self,
        original: &JournalEntry,
        reversal_date: NaiveDate,
    ) -> JournalEntry {
        self.accrual_counter += 1;
        let doc_number = format!("REV{:08}", self.accrual_counter);

        let mut reversal = JournalEntry::new_simple(
            doc_number.clone(),
            original.company_code().to_string(),
            reversal_date,
            format!("Reversal of {}", original.description().unwrap_or("entry")),
        );

        // Reverse each line (swap debits and credits)
        for (idx, line) in original.lines.iter().enumerate() {
            reversal.add_line(JournalEntryLine {
                line_number: (idx + 1) as u32,
                gl_account: line.gl_account.clone(),
                debit_amount: line.credit_amount,
                credit_amount: line.debit_amount,
                cost_center: line.cost_center.clone(),
                profit_center: line.profit_center.clone(),
                reference: Some(format!("REV-{}", original.document_number())),
                assignment: line.assignment.clone(),
                text: Some(format!(
                    "Reversal: {}",
                    line.text.clone().unwrap_or_default()
                )),
                quantity: line.quantity,
                unit: line.unit.clone(),
                tax_code: line.tax_code.clone(),
                trading_partner: line.trading_partner.clone(),
                value_date: line.value_date,
                ..Default::default()
            });
        }

        reversal
    }
}

/// Result of accrual generation.
#[derive(Debug, Clone)]
pub struct AccrualGenerationResult {
    /// Fiscal period.
    pub period: FiscalPeriod,
    /// Generated accrual entries.
    pub accrual_entries: Vec<JournalEntry>,
    /// Generated reversal entries.
    pub reversal_entries: Vec<JournalEntry>,
    /// Total accrued expenses.
    pub total_accrued_expenses: Decimal,
    /// Total accrued revenue.
    pub total_accrued_revenue: Decimal,
    /// Definitions that were skipped.
    pub skipped_definitions: Vec<SkippedAccrual>,
}

/// Information about a skipped accrual.
#[derive(Debug, Clone)]
pub struct SkippedAccrual {
    /// Accrual definition ID.
    pub accrual_id: String,
    /// Reason for skipping.
    pub reason: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_accrued_expense() {
        let mut generator = AccrualGenerator::new(AccrualGeneratorConfig::default());

        let (je, reversal) = generator.generate_accrued_expense(
            "1000",
            "Accrued Utilities",
            dec!(5000),
            "6200",
            "2100",
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            Some("CC100"),
        );

        assert!(je.is_balanced());
        assert!(reversal.is_some());
        assert!(reversal.unwrap().is_balanced());
    }

    #[test]
    fn test_generate_accrued_revenue() {
        let mut generator = AccrualGenerator::new(AccrualGeneratorConfig::default());

        let (je, _) = generator.generate_accrued_revenue(
            "1000",
            "Accrued Interest",
            dec!(1000),
            "4100",
            "1250",
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            None,
        );

        assert!(je.is_balanced());
    }

    #[test]
    fn test_prepaid_amortization() {
        let mut generator = AccrualGenerator::new(AccrualGeneratorConfig::default());

        let je = generator.generate_prepaid_amortization(
            "1000",
            "Insurance Premium",
            dec!(1000),
            "6300",
            "1400",
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            None,
        );

        assert!(je.is_balanced());
    }
}
