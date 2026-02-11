//! Year-end closing entry generator.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::{
    JournalEntry, JournalEntryLine, TaxAdjustment, TaxProvisionInput, TaxProvisionResult,
    YearEndClosingSpec,
};

/// Configuration for year-end closing.
#[derive(Debug, Clone)]
pub struct YearEndCloseConfig {
    /// Income summary account.
    pub income_summary_account: String,
    /// Retained earnings account.
    pub retained_earnings_account: String,
    /// Dividend declared account.
    pub dividend_account: String,
    /// Current tax payable account.
    pub current_tax_payable_account: String,
    /// Deferred tax liability account.
    pub deferred_tax_liability_account: String,
    /// Deferred tax asset account.
    pub deferred_tax_asset_account: String,
    /// Tax expense account.
    pub tax_expense_account: String,
    /// Statutory tax rate.
    pub statutory_tax_rate: Decimal,
}

impl Default for YearEndCloseConfig {
    fn default() -> Self {
        Self {
            income_summary_account: "3500".to_string(),
            retained_earnings_account: "3300".to_string(),
            dividend_account: "3400".to_string(),
            current_tax_payable_account: "2300".to_string(),
            deferred_tax_liability_account: "2350".to_string(),
            deferred_tax_asset_account: "1600".to_string(),
            tax_expense_account: "7100".to_string(),
            statutory_tax_rate: dec!(21),
        }
    }
}

/// Generator for year-end closing entries.
pub struct YearEndCloseGenerator {
    config: YearEndCloseConfig,
    entry_counter: u64,
}

impl YearEndCloseGenerator {
    /// Creates a new year-end close generator.
    pub fn new(config: YearEndCloseConfig) -> Self {
        Self {
            config,
            entry_counter: 0,
        }
    }

    /// Generates the complete year-end closing entries.
    pub fn generate_year_end_close(
        &mut self,
        company_code: &str,
        fiscal_year: i32,
        trial_balance: &HashMap<String, Decimal>,
        spec: &YearEndClosingSpec,
    ) -> YearEndCloseResult {
        let closing_date =
            NaiveDate::from_ymd_opt(fiscal_year, 12, 31).expect("valid year-end date");

        let mut result = YearEndCloseResult {
            company_code: company_code.to_string(),
            fiscal_year,
            closing_entries: Vec::new(),
            total_revenue_closed: Decimal::ZERO,
            total_expense_closed: Decimal::ZERO,
            net_income: Decimal::ZERO,
            retained_earnings_impact: Decimal::ZERO,
        };

        // Step 1: Close revenue accounts to income summary
        let (revenue_je, revenue_total) =
            self.close_revenue_accounts(company_code, closing_date, trial_balance, spec);
        result.total_revenue_closed = revenue_total;
        result.closing_entries.push(revenue_je);

        // Step 2: Close expense accounts to income summary
        let (expense_je, expense_total) =
            self.close_expense_accounts(company_code, closing_date, trial_balance, spec);
        result.total_expense_closed = expense_total;
        result.closing_entries.push(expense_je);

        // Step 3: Close income summary to retained earnings
        let net_income = revenue_total - expense_total;
        result.net_income = net_income;

        let income_summary_je = self.close_income_summary(company_code, closing_date, net_income);
        result.closing_entries.push(income_summary_je);

        // Step 4: Close dividends to retained earnings (if applicable)
        if let Some(dividend_account) = &spec.dividend_account {
            if let Some(dividend_balance) = trial_balance.get(dividend_account) {
                if *dividend_balance != Decimal::ZERO {
                    let dividend_je = self.close_dividends(
                        company_code,
                        closing_date,
                        *dividend_balance,
                        dividend_account,
                    );
                    result.closing_entries.push(dividend_je);
                    result.retained_earnings_impact = net_income - *dividend_balance;
                } else {
                    result.retained_earnings_impact = net_income;
                }
            } else {
                result.retained_earnings_impact = net_income;
            }
        } else {
            result.retained_earnings_impact = net_income;
        }

        result
    }

    /// Closes revenue accounts to income summary.
    fn close_revenue_accounts(
        &mut self,
        company_code: &str,
        closing_date: NaiveDate,
        trial_balance: &HashMap<String, Decimal>,
        spec: &YearEndClosingSpec,
    ) -> (JournalEntry, Decimal) {
        self.entry_counter += 1;
        let doc_number = format!("YECL-REV-{:08}", self.entry_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            closing_date,
            "Year-End Close: Revenue to Income Summary".to_string(),
        );

        let mut line_num = 1u32;
        let mut total_revenue = Decimal::ZERO;

        // Find all revenue accounts (accounts starting with prefixes in spec)
        for (account, balance) in trial_balance {
            let is_revenue = spec
                .revenue_accounts
                .iter()
                .any(|prefix| account.starts_with(prefix));

            if is_revenue && *balance != Decimal::ZERO {
                // Revenue accounts have credit balances, so debit to close
                je.add_line(JournalEntryLine {
                    line_number: line_num,
                    gl_account: account.clone(),
                    debit_amount: *balance,
                    reference: Some(doc_number.clone()),
                    text: Some("Year-end close".to_string()),
                    ..Default::default()
                });
                line_num += 1;
                total_revenue += *balance;
            }
        }

        // Credit Income Summary
        if total_revenue != Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: spec.income_summary_account.clone(),
                credit_amount: total_revenue,
                reference: Some(doc_number.clone()),
                text: Some("Revenue closed".to_string()),
                ..Default::default()
            });
        }

        (je, total_revenue)
    }

    /// Closes expense accounts to income summary.
    fn close_expense_accounts(
        &mut self,
        company_code: &str,
        closing_date: NaiveDate,
        trial_balance: &HashMap<String, Decimal>,
        spec: &YearEndClosingSpec,
    ) -> (JournalEntry, Decimal) {
        self.entry_counter += 1;
        let doc_number = format!("YECL-EXP-{:08}", self.entry_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            closing_date,
            "Year-End Close: Expenses to Income Summary".to_string(),
        );

        let mut line_num = 1u32;
        let mut total_expenses = Decimal::ZERO;

        // Debit Income Summary first
        // We'll update this amount after calculating total expenses

        // Find all expense accounts
        let mut expense_lines = Vec::new();
        for (account, balance) in trial_balance {
            let is_expense = spec
                .expense_accounts
                .iter()
                .any(|prefix| account.starts_with(prefix));

            if is_expense && *balance != Decimal::ZERO {
                expense_lines.push((account.clone(), *balance));
                total_expenses += *balance;
            }
        }

        // Debit Income Summary
        if total_expenses != Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: spec.income_summary_account.clone(),
                debit_amount: total_expenses,
                reference: Some(doc_number.clone()),
                text: Some("Expenses closed".to_string()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Credit each expense account
        for (account, balance) in expense_lines {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: account,
                credit_amount: balance,
                reference: Some(doc_number.clone()),
                text: Some("Year-end close".to_string()),
                ..Default::default()
            });
            line_num += 1;
        }

        (je, total_expenses)
    }

    /// Closes income summary to retained earnings.
    fn close_income_summary(
        &mut self,
        company_code: &str,
        closing_date: NaiveDate,
        net_income: Decimal,
    ) -> JournalEntry {
        self.entry_counter += 1;
        let doc_number = format!("YECL-IS-{:08}", self.entry_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            closing_date,
            "Year-End Close: Income Summary to Retained Earnings".to_string(),
        );

        if net_income > Decimal::ZERO {
            // Profit: Debit Income Summary, Credit Retained Earnings
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: self.config.income_summary_account.clone(),
                debit_amount: net_income,
                reference: Some(doc_number.clone()),
                text: Some("Net income transfer".to_string()),
                ..Default::default()
            });

            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: self.config.retained_earnings_account.clone(),
                credit_amount: net_income,
                reference: Some(doc_number.clone()),
                text: Some("Net income for year".to_string()),
                ..Default::default()
            });
        } else if net_income < Decimal::ZERO {
            // Loss: Debit Retained Earnings, Credit Income Summary
            let loss = net_income.abs();
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: self.config.retained_earnings_account.clone(),
                debit_amount: loss,
                reference: Some(doc_number.clone()),
                text: Some("Net loss for year".to_string()),
                ..Default::default()
            });

            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: self.config.income_summary_account.clone(),
                credit_amount: loss,
                reference: Some(doc_number.clone()),
                text: Some("Net loss transfer".to_string()),
                ..Default::default()
            });
        }

        je
    }

    /// Closes dividends to retained earnings.
    fn close_dividends(
        &mut self,
        company_code: &str,
        closing_date: NaiveDate,
        dividend_amount: Decimal,
        dividend_account: &str,
    ) -> JournalEntry {
        self.entry_counter += 1;
        let doc_number = format!("YECL-DIV-{:08}", self.entry_counter);

        let mut je = JournalEntry::new_simple(
            doc_number.clone(),
            company_code.to_string(),
            closing_date,
            "Year-End Close: Dividends to Retained Earnings".to_string(),
        );

        // Debit Retained Earnings
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: self.config.retained_earnings_account.clone(),
            debit_amount: dividend_amount,
            reference: Some(doc_number.clone()),
            text: Some("Dividends declared".to_string()),
            ..Default::default()
        });

        // Credit Dividends
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: dividend_account.to_string(),
            credit_amount: dividend_amount,
            reference: Some(doc_number.clone()),
            text: Some("Dividends closed".to_string()),
            ..Default::default()
        });

        je
    }

    /// Generates tax provision entries.
    pub fn generate_tax_provision(
        &mut self,
        company_code: &str,
        fiscal_year: i32,
        pretax_income: Decimal,
        permanent_differences: Vec<TaxAdjustment>,
        temporary_differences: Vec<TaxAdjustment>,
    ) -> TaxProvisionGenerationResult {
        let closing_date =
            NaiveDate::from_ymd_opt(fiscal_year, 12, 31).expect("valid year-end date");

        let input = TaxProvisionInput {
            company_code: company_code.to_string(),
            fiscal_year,
            pretax_income,
            permanent_differences,
            temporary_differences,
            statutory_rate: self.config.statutory_tax_rate,
            tax_credits: Decimal::ZERO,
            prior_year_adjustment: Decimal::ZERO,
        };

        let provision = TaxProvisionResult::calculate(&input);

        // Generate journal entries
        let mut entries = Vec::new();

        // Entry 1: Current tax expense
        if provision.current_tax_expense != Decimal::ZERO {
            self.entry_counter += 1;
            let mut je = JournalEntry::new_simple(
                format!("TAX-CUR-{:08}", self.entry_counter),
                company_code.to_string(),
                closing_date,
                "Current Income Tax Expense".to_string(),
            );

            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: self.config.tax_expense_account.clone(),
                debit_amount: provision.current_tax_expense,
                text: Some("Current tax provision".to_string()),
                ..Default::default()
            });

            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: self.config.current_tax_payable_account.clone(),
                credit_amount: provision.current_tax_expense,
                ..Default::default()
            });

            entries.push(je);
        }

        // Entry 2: Deferred tax expense/benefit
        if provision.deferred_tax_expense != Decimal::ZERO {
            self.entry_counter += 1;
            let mut je = JournalEntry::new_simple(
                format!("TAX-DEF-{:08}", self.entry_counter),
                company_code.to_string(),
                closing_date,
                "Deferred Income Tax".to_string(),
            );

            if provision.deferred_tax_expense > Decimal::ZERO {
                // Deferred tax expense (increase in DTL or decrease in DTA)
                je.add_line(JournalEntryLine {
                    line_number: 1,
                    gl_account: self.config.tax_expense_account.clone(),
                    debit_amount: provision.deferred_tax_expense,
                    text: Some("Deferred tax expense".to_string()),
                    ..Default::default()
                });

                je.add_line(JournalEntryLine {
                    line_number: 2,
                    gl_account: self.config.deferred_tax_liability_account.clone(),
                    credit_amount: provision.deferred_tax_expense,
                    ..Default::default()
                });
            } else {
                // Deferred tax benefit (increase in DTA or decrease in DTL)
                let benefit = provision.deferred_tax_expense.abs();
                je.add_line(JournalEntryLine {
                    line_number: 1,
                    gl_account: self.config.deferred_tax_asset_account.clone(),
                    debit_amount: benefit,
                    text: Some("Deferred tax benefit".to_string()),
                    ..Default::default()
                });

                je.add_line(JournalEntryLine {
                    line_number: 2,
                    gl_account: self.config.tax_expense_account.clone(),
                    credit_amount: benefit,
                    ..Default::default()
                });
            }

            entries.push(je);
        }

        TaxProvisionGenerationResult {
            provision,
            journal_entries: entries,
        }
    }
}

/// Result of year-end closing.
#[derive(Debug, Clone)]
pub struct YearEndCloseResult {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Generated closing entries.
    pub closing_entries: Vec<JournalEntry>,
    /// Total revenue closed.
    pub total_revenue_closed: Decimal,
    /// Total expenses closed.
    pub total_expense_closed: Decimal,
    /// Net income (revenue - expenses).
    pub net_income: Decimal,
    /// Impact on retained earnings.
    pub retained_earnings_impact: Decimal,
}

impl YearEndCloseResult {
    /// Returns true if all entries are balanced.
    pub fn all_entries_balanced(&self) -> bool {
        self.closing_entries.iter().all(|je| je.is_balanced())
    }
}

/// Result of tax provision generation.
#[derive(Debug, Clone)]
pub struct TaxProvisionGenerationResult {
    /// Tax provision calculation result.
    pub provision: TaxProvisionResult,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_end_close() {
        let mut generator = YearEndCloseGenerator::new(YearEndCloseConfig::default());

        let mut trial_balance = HashMap::new();
        trial_balance.insert("4000".to_string(), dec!(500000)); // Revenue
        trial_balance.insert("4100".to_string(), dec!(50000)); // Other Revenue
        trial_balance.insert("5000".to_string(), dec!(300000)); // COGS
        trial_balance.insert("6000".to_string(), dec!(100000)); // Operating Expenses

        let spec = YearEndClosingSpec {
            company_code: "1000".to_string(),
            fiscal_year: 2024,
            revenue_accounts: vec!["4".to_string()],
            expense_accounts: vec!["5".to_string(), "6".to_string()],
            income_summary_account: "3500".to_string(),
            retained_earnings_account: "3300".to_string(),
            dividend_account: None,
        };

        let result = generator.generate_year_end_close("1000", 2024, &trial_balance, &spec);

        assert_eq!(result.total_revenue_closed, dec!(550000));
        assert_eq!(result.total_expense_closed, dec!(400000));
        assert_eq!(result.net_income, dec!(150000));
        assert!(result.all_entries_balanced());
    }

    #[test]
    fn test_tax_provision() {
        let mut generator = YearEndCloseGenerator::new(YearEndCloseConfig::default());

        let result = generator.generate_tax_provision(
            "1000",
            2024,
            dec!(1000000),
            vec![TaxAdjustment {
                description: "Non-deductible expenses".to_string(),
                amount: dec!(10000),
                is_addition: true,
            }],
            vec![],
        );

        assert!(result.provision.current_tax_expense > Decimal::ZERO);
        assert!(result.journal_entries.iter().all(|je| je.is_balanced()));
    }
}
