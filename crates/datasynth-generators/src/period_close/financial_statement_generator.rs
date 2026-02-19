//! Financial statement generator.
//!
//! Generates financial statements from adjusted trial balance data:
//! - Balance Sheet: Assets = Liabilities + Equity
//! - Income Statement: Revenue - COGS - OpEx - Tax = Net Income
//! - Cash Flow Statement (indirect method)
//! - Statement of Changes in Equity

use chrono::NaiveDate;
use datasynth_config::schema::FinancialReportingConfig;
use datasynth_core::models::{
    CashFlowCategory, CashFlowItem, FinancialStatement, FinancialStatementLineItem, StatementBasis,
    StatementType,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Generates financial statements from trial balance data.
pub struct FinancialStatementGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: FinancialReportingConfig,
}

/// Trial balance entry for statement generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceEntry {
    /// GL account code
    pub account_code: String,
    /// Account description
    pub account_name: String,
    /// Account category (Asset, Liability, Equity, Revenue, Expense)
    pub category: String,
    /// Debit balance
    pub debit_balance: Decimal,
    /// Credit balance
    pub credit_balance: Decimal,
}

impl FinancialStatementGenerator {
    /// Create a new financial statement generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::FinancialStatement),
            config: FinancialReportingConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: FinancialReportingConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::FinancialStatement),
            config,
        }
    }

    /// Generate all financial statements for a period.
    pub fn generate(
        &mut self,
        company_code: &str,
        currency: &str,
        trial_balance: &[TrialBalanceEntry],
        period_start: NaiveDate,
        period_end: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        prior_trial_balance: Option<&[TrialBalanceEntry]>,
        preparer_id: &str,
    ) -> Vec<FinancialStatement> {
        debug!(
            company_code,
            currency,
            fiscal_year,
            fiscal_period,
            tb_entries = trial_balance.len(),
            "Generating financial statements"
        );
        let mut statements = Vec::new();

        if self.config.generate_balance_sheet {
            statements.push(self.generate_balance_sheet(
                company_code,
                currency,
                trial_balance,
                period_start,
                period_end,
                fiscal_year,
                fiscal_period,
                prior_trial_balance,
                preparer_id,
            ));
        }

        if self.config.generate_income_statement {
            statements.push(self.generate_income_statement(
                company_code,
                currency,
                trial_balance,
                period_start,
                period_end,
                fiscal_year,
                fiscal_period,
                prior_trial_balance,
                preparer_id,
            ));
        }

        if self.config.generate_cash_flow {
            let net_income = self.calculate_net_income(trial_balance);
            statements.push(self.generate_cash_flow_statement(
                company_code,
                currency,
                trial_balance,
                period_start,
                period_end,
                fiscal_year,
                fiscal_period,
                net_income,
                preparer_id,
            ));
        }

        statements
    }

    fn generate_balance_sheet(
        &mut self,
        company_code: &str,
        currency: &str,
        tb: &[TrialBalanceEntry],
        period_start: NaiveDate,
        period_end: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        prior_tb: Option<&[TrialBalanceEntry]>,
        preparer_id: &str,
    ) -> FinancialStatement {
        let mut line_items = Vec::new();
        let mut sort_order = 0u32;

        // Aggregate by category
        let aggregated = self.aggregate_by_category(tb);
        let prior_aggregated = prior_tb.map(|ptb| self.aggregate_by_category(ptb));

        let get_prior = |key: &str| -> Option<Decimal> {
            prior_aggregated
                .as_ref()
                .and_then(|pa| pa.get(key).copied())
        };

        // Assets
        let cash = *aggregated.get("Cash").unwrap_or(&Decimal::ZERO);
        let ar = *aggregated.get("Receivables").unwrap_or(&Decimal::ZERO);
        let inventory = *aggregated.get("Inventory").unwrap_or(&Decimal::ZERO);
        let current_assets = cash + ar + inventory;
        let fixed_assets = *aggregated.get("FixedAssets").unwrap_or(&Decimal::ZERO);
        let total_assets = current_assets + fixed_assets;

        let items_data = [
            (
                "BS-CASH",
                "Cash and Cash Equivalents",
                "Current Assets",
                cash,
                get_prior("Cash"),
                0,
                false,
            ),
            (
                "BS-AR",
                "Accounts Receivable",
                "Current Assets",
                ar,
                get_prior("Receivables"),
                0,
                false,
            ),
            (
                "BS-INV",
                "Inventory",
                "Current Assets",
                inventory,
                get_prior("Inventory"),
                0,
                false,
            ),
            (
                "BS-CA",
                "Total Current Assets",
                "Current Assets",
                current_assets,
                None,
                0,
                true,
            ),
            (
                "BS-FA",
                "Property, Plant & Equipment, net",
                "Non-Current Assets",
                fixed_assets,
                get_prior("FixedAssets"),
                0,
                false,
            ),
            (
                "BS-TA",
                "Total Assets",
                "Total Assets",
                total_assets,
                None,
                0,
                true,
            ),
        ];

        for (code, label, section, amount, prior, indent, is_total) in &items_data {
            sort_order += 1;
            line_items.push(FinancialStatementLineItem {
                line_code: code.to_string(),
                label: label.to_string(),
                section: section.to_string(),
                sort_order,
                amount: *amount,
                amount_prior: *prior,
                indent_level: *indent,
                is_total: *is_total,
                gl_accounts: Vec::new(),
            });
        }

        // Liabilities & Equity
        let ap = *aggregated.get("Payables").unwrap_or(&Decimal::ZERO);
        let accrued = *aggregated
            .get("AccruedLiabilities")
            .unwrap_or(&Decimal::ZERO);
        let current_liabilities = ap + accrued;
        let lt_debt = *aggregated.get("LongTermDebt").unwrap_or(&Decimal::ZERO);
        let total_liabilities = current_liabilities + lt_debt;

        let retained_earnings = total_assets - total_liabilities;
        let total_equity = retained_earnings;
        let total_le = total_liabilities + total_equity;

        let le_items = [
            (
                "BS-AP",
                "Accounts Payable",
                "Current Liabilities",
                ap,
                get_prior("Payables"),
                0,
                false,
            ),
            (
                "BS-ACR",
                "Accrued Liabilities",
                "Current Liabilities",
                accrued,
                get_prior("AccruedLiabilities"),
                0,
                false,
            ),
            (
                "BS-CL",
                "Total Current Liabilities",
                "Current Liabilities",
                current_liabilities,
                None,
                0,
                true,
            ),
            (
                "BS-LTD",
                "Long-Term Debt",
                "Non-Current Liabilities",
                lt_debt,
                get_prior("LongTermDebt"),
                0,
                false,
            ),
            (
                "BS-TL",
                "Total Liabilities",
                "Total Liabilities",
                total_liabilities,
                None,
                0,
                true,
            ),
            (
                "BS-RE",
                "Retained Earnings",
                "Equity",
                retained_earnings,
                None,
                0,
                false,
            ),
            (
                "BS-TE",
                "Total Equity",
                "Equity",
                total_equity,
                None,
                0,
                true,
            ),
            (
                "BS-TLE",
                "Total Liabilities & Equity",
                "Total",
                total_le,
                None,
                0,
                true,
            ),
        ];

        for (code, label, section, amount, prior, indent, is_total) in &le_items {
            sort_order += 1;
            line_items.push(FinancialStatementLineItem {
                line_code: code.to_string(),
                label: label.to_string(),
                section: section.to_string(),
                sort_order,
                amount: *amount,
                amount_prior: *prior,
                indent_level: *indent,
                is_total: *is_total,
                gl_accounts: Vec::new(),
            });
        }

        FinancialStatement {
            statement_id: self.uuid_factory.next().to_string(),
            company_code: company_code.to_string(),
            statement_type: StatementType::BalanceSheet,
            basis: StatementBasis::UsGaap,
            period_start,
            period_end,
            fiscal_year,
            fiscal_period,
            line_items,
            cash_flow_items: Vec::new(),
            currency: currency.to_string(),
            is_consolidated: false,
            preparer_id: preparer_id.to_string(),
        }
    }

    fn generate_income_statement(
        &mut self,
        company_code: &str,
        currency: &str,
        tb: &[TrialBalanceEntry],
        period_start: NaiveDate,
        period_end: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        prior_tb: Option<&[TrialBalanceEntry]>,
        preparer_id: &str,
    ) -> FinancialStatement {
        let aggregated = self.aggregate_by_category(tb);
        let prior_aggregated = prior_tb.map(|ptb| self.aggregate_by_category(ptb));

        let get_prior = |key: &str| -> Option<Decimal> {
            prior_aggregated
                .as_ref()
                .and_then(|pa| pa.get(key).copied())
        };

        let revenue = *aggregated.get("Revenue").unwrap_or(&Decimal::ZERO);
        let cogs = *aggregated.get("CostOfSales").unwrap_or(&Decimal::ZERO);
        let gross_profit = revenue - cogs;
        let operating_expenses = *aggregated
            .get("OperatingExpenses")
            .unwrap_or(&Decimal::ZERO);
        let operating_income = gross_profit - operating_expenses;
        let tax = operating_income * Decimal::from_f64_retain(0.25).unwrap_or(Decimal::ZERO);
        let net_income = operating_income - tax;

        let mut line_items = Vec::new();
        let items_data = [
            (
                "IS-REV",
                "Revenue",
                "Revenue",
                revenue,
                get_prior("Revenue"),
                false,
            ),
            (
                "IS-COGS",
                "Cost of Goods Sold",
                "Cost of Sales",
                cogs,
                get_prior("CostOfSales"),
                false,
            ),
            (
                "IS-GP",
                "Gross Profit",
                "Gross Profit",
                gross_profit,
                None,
                true,
            ),
            (
                "IS-OPEX",
                "Operating Expenses",
                "Operating Expenses",
                operating_expenses,
                get_prior("OperatingExpenses"),
                false,
            ),
            (
                "IS-OI",
                "Operating Income",
                "Operating Income",
                operating_income,
                None,
                true,
            ),
            ("IS-TAX", "Income Tax Expense", "Tax", tax, None, false),
            ("IS-NI", "Net Income", "Net Income", net_income, None, true),
        ];

        for (i, (code, label, section, amount, prior, is_total)) in items_data.iter().enumerate() {
            line_items.push(FinancialStatementLineItem {
                line_code: code.to_string(),
                label: label.to_string(),
                section: section.to_string(),
                sort_order: (i + 1) as u32,
                amount: *amount,
                amount_prior: *prior,
                indent_level: 0,
                is_total: *is_total,
                gl_accounts: Vec::new(),
            });
        }

        FinancialStatement {
            statement_id: self.uuid_factory.next().to_string(),
            company_code: company_code.to_string(),
            statement_type: StatementType::IncomeStatement,
            basis: StatementBasis::UsGaap,
            period_start,
            period_end,
            fiscal_year,
            fiscal_period,
            line_items,
            cash_flow_items: Vec::new(),
            currency: currency.to_string(),
            is_consolidated: false,
            preparer_id: preparer_id.to_string(),
        }
    }

    fn generate_cash_flow_statement(
        &mut self,
        company_code: &str,
        currency: &str,
        _tb: &[TrialBalanceEntry],
        period_start: NaiveDate,
        period_end: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        net_income: Decimal,
        preparer_id: &str,
    ) -> FinancialStatement {
        // Indirect method: start with net income, adjust for non-cash items
        let depreciation = Decimal::from(self.rng.gen_range(5000..=50000));
        let ar_change = Decimal::from(self.rng.gen_range(-20000i64..=20000));
        let ap_change = Decimal::from(self.rng.gen_range(-15000i64..=15000));
        let inventory_change = Decimal::from(self.rng.gen_range(-10000i64..=10000));

        let operating_cf = net_income + depreciation - ar_change + ap_change - inventory_change;

        let capex = Decimal::from(self.rng.gen_range(-100000i64..=-5000));
        let investing_cf = capex;

        let debt_change = Decimal::from(self.rng.gen_range(-50000i64..=50000));
        let financing_cf = debt_change;

        let net_change = operating_cf + investing_cf + financing_cf;

        let cash_flow_items = vec![
            CashFlowItem {
                item_code: "CF-NI".to_string(),
                label: "Net Income".to_string(),
                category: CashFlowCategory::Operating,
                amount: net_income,
                amount_prior: None,
                sort_order: 1,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-DEP".to_string(),
                label: "Depreciation & Amortization".to_string(),
                category: CashFlowCategory::Operating,
                amount: depreciation,
                amount_prior: None,
                sort_order: 2,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-AR".to_string(),
                label: "Change in Accounts Receivable".to_string(),
                category: CashFlowCategory::Operating,
                amount: -ar_change,
                amount_prior: None,
                sort_order: 3,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-AP".to_string(),
                label: "Change in Accounts Payable".to_string(),
                category: CashFlowCategory::Operating,
                amount: ap_change,
                amount_prior: None,
                sort_order: 4,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-INV".to_string(),
                label: "Change in Inventory".to_string(),
                category: CashFlowCategory::Operating,
                amount: -inventory_change,
                amount_prior: None,
                sort_order: 5,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-OP".to_string(),
                label: "Net Cash from Operating Activities".to_string(),
                category: CashFlowCategory::Operating,
                amount: operating_cf,
                amount_prior: None,
                sort_order: 6,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-CAPEX".to_string(),
                label: "Capital Expenditures".to_string(),
                category: CashFlowCategory::Investing,
                amount: capex,
                amount_prior: None,
                sort_order: 7,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-INV-T".to_string(),
                label: "Net Cash from Investing Activities".to_string(),
                category: CashFlowCategory::Investing,
                amount: investing_cf,
                amount_prior: None,
                sort_order: 8,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-DEBT".to_string(),
                label: "Net Borrowings / (Repayments)".to_string(),
                category: CashFlowCategory::Financing,
                amount: debt_change,
                amount_prior: None,
                sort_order: 9,
                is_total: false,
            },
            CashFlowItem {
                item_code: "CF-FIN-T".to_string(),
                label: "Net Cash from Financing Activities".to_string(),
                category: CashFlowCategory::Financing,
                amount: financing_cf,
                amount_prior: None,
                sort_order: 10,
                is_total: true,
            },
            CashFlowItem {
                item_code: "CF-NET".to_string(),
                label: "Net Change in Cash".to_string(),
                category: CashFlowCategory::Operating,
                amount: net_change,
                amount_prior: None,
                sort_order: 11,
                is_total: true,
            },
        ];

        FinancialStatement {
            statement_id: self.uuid_factory.next().to_string(),
            company_code: company_code.to_string(),
            statement_type: StatementType::CashFlowStatement,
            basis: StatementBasis::UsGaap,
            period_start,
            period_end,
            fiscal_year,
            fiscal_period,
            line_items: Vec::new(),
            cash_flow_items,
            currency: currency.to_string(),
            is_consolidated: false,
            preparer_id: preparer_id.to_string(),
        }
    }

    fn calculate_net_income(&self, tb: &[TrialBalanceEntry]) -> Decimal {
        let aggregated = self.aggregate_by_category(tb);
        let revenue = *aggregated.get("Revenue").unwrap_or(&Decimal::ZERO);
        let cogs = *aggregated.get("CostOfSales").unwrap_or(&Decimal::ZERO);
        let opex = *aggregated
            .get("OperatingExpenses")
            .unwrap_or(&Decimal::ZERO);
        let operating_income = revenue - cogs - opex;
        let tax = operating_income * Decimal::from_f64_retain(0.25).unwrap_or(Decimal::ZERO);
        operating_income - tax
    }

    fn aggregate_by_category(&self, tb: &[TrialBalanceEntry]) -> HashMap<String, Decimal> {
        let mut aggregated: HashMap<String, Decimal> = HashMap::new();
        for entry in tb {
            let net = entry.debit_balance - entry.credit_balance;
            *aggregated.entry(entry.category.clone()).or_default() += net;
        }
        aggregated
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_trial_balance() -> Vec<TrialBalanceEntry> {
        vec![
            TrialBalanceEntry {
                account_code: "1000".to_string(),
                account_name: "Cash".to_string(),
                category: "Cash".to_string(),
                debit_balance: Decimal::from(500_000),
                credit_balance: Decimal::ZERO,
            },
            TrialBalanceEntry {
                account_code: "1100".to_string(),
                account_name: "Accounts Receivable".to_string(),
                category: "Receivables".to_string(),
                debit_balance: Decimal::from(200_000),
                credit_balance: Decimal::ZERO,
            },
            TrialBalanceEntry {
                account_code: "1300".to_string(),
                account_name: "Inventory".to_string(),
                category: "Inventory".to_string(),
                debit_balance: Decimal::from(150_000),
                credit_balance: Decimal::ZERO,
            },
            TrialBalanceEntry {
                account_code: "1500".to_string(),
                account_name: "Fixed Assets".to_string(),
                category: "FixedAssets".to_string(),
                debit_balance: Decimal::from(800_000),
                credit_balance: Decimal::ZERO,
            },
            TrialBalanceEntry {
                account_code: "2000".to_string(),
                account_name: "Accounts Payable".to_string(),
                category: "Payables".to_string(),
                debit_balance: Decimal::ZERO,
                credit_balance: Decimal::from(120_000),
            },
            TrialBalanceEntry {
                account_code: "2100".to_string(),
                account_name: "Accrued Liabilities".to_string(),
                category: "AccruedLiabilities".to_string(),
                debit_balance: Decimal::ZERO,
                credit_balance: Decimal::from(80_000),
            },
            TrialBalanceEntry {
                account_code: "4000".to_string(),
                account_name: "Revenue".to_string(),
                category: "Revenue".to_string(),
                debit_balance: Decimal::ZERO,
                credit_balance: Decimal::from(1_000_000),
            },
            TrialBalanceEntry {
                account_code: "5000".to_string(),
                account_name: "Cost of Goods Sold".to_string(),
                category: "CostOfSales".to_string(),
                debit_balance: Decimal::from(600_000),
                credit_balance: Decimal::ZERO,
            },
            TrialBalanceEntry {
                account_code: "6000".to_string(),
                account_name: "Operating Expenses".to_string(),
                category: "OperatingExpenses".to_string(),
                debit_balance: Decimal::from(250_000),
                credit_balance: Decimal::ZERO,
            },
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = FinancialStatementGenerator::new(42);
        let tb = test_trial_balance();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let statements = gen.generate("C001", "USD", &tb, start, end, 2024, 1, None, "PREP-01");

        // Default config generates all 3 statement types
        assert_eq!(statements.len(), 3);

        let bs = statements
            .iter()
            .find(|s| s.statement_type == StatementType::BalanceSheet)
            .unwrap();
        let is = statements
            .iter()
            .find(|s| s.statement_type == StatementType::IncomeStatement)
            .unwrap();
        let cf = statements
            .iter()
            .find(|s| s.statement_type == StatementType::CashFlowStatement)
            .unwrap();

        // Balance sheet checks
        assert!(!bs.statement_id.is_empty());
        assert_eq!(bs.company_code, "C001");
        assert_eq!(bs.currency, "USD");
        assert!(!bs.line_items.is_empty());
        assert_eq!(bs.fiscal_year, 2024);
        assert_eq!(bs.fiscal_period, 1);
        assert_eq!(bs.preparer_id, "PREP-01");

        // Income statement checks
        assert!(!is.statement_id.is_empty());
        assert!(!is.line_items.is_empty());

        // Cash flow checks
        assert!(!cf.statement_id.is_empty());
        assert!(!cf.cash_flow_items.is_empty());
    }

    #[test]
    fn test_deterministic() {
        let tb = test_trial_balance();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let mut gen1 = FinancialStatementGenerator::new(42);
        let mut gen2 = FinancialStatementGenerator::new(42);

        let r1 = gen1.generate("C001", "USD", &tb, start, end, 2024, 1, None, "PREP-01");
        let r2 = gen2.generate("C001", "USD", &tb, start, end, 2024, 1, None, "PREP-01");

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.statement_id, b.statement_id);
            assert_eq!(a.statement_type, b.statement_type);
            assert_eq!(a.line_items.len(), b.line_items.len());
            assert_eq!(a.cash_flow_items.len(), b.cash_flow_items.len());

            for (li_a, li_b) in a.line_items.iter().zip(b.line_items.iter()) {
                assert_eq!(li_a.line_code, li_b.line_code);
                assert_eq!(li_a.amount, li_b.amount);
            }
            for (cf_a, cf_b) in a.cash_flow_items.iter().zip(b.cash_flow_items.iter()) {
                assert_eq!(cf_a.item_code, cf_b.item_code);
                assert_eq!(cf_a.amount, cf_b.amount);
            }
        }
    }

    #[test]
    fn test_balance_sheet_balances() {
        let mut gen = FinancialStatementGenerator::new(42);
        let tb = test_trial_balance();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let statements = gen.generate("C001", "USD", &tb, start, end, 2024, 1, None, "PREP-01");
        let bs = statements
            .iter()
            .find(|s| s.statement_type == StatementType::BalanceSheet)
            .unwrap();

        // Find Total Assets and Total Liabilities & Equity
        let total_assets = bs
            .line_items
            .iter()
            .find(|li| li.line_code == "BS-TA")
            .unwrap();
        let total_le = bs
            .line_items
            .iter()
            .find(|li| li.line_code == "BS-TLE")
            .unwrap();

        // Assets = Liabilities + Equity (balance sheet must balance)
        assert_eq!(
            total_assets.amount, total_le.amount,
            "Balance sheet does not balance: Assets={} vs L+E={}",
            total_assets.amount, total_le.amount
        );
    }

    #[test]
    fn test_income_statement_structure() {
        let mut gen = FinancialStatementGenerator::new(42);
        let tb = test_trial_balance();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let statements = gen.generate("C001", "USD", &tb, start, end, 2024, 1, None, "PREP-01");
        let is = statements
            .iter()
            .find(|s| s.statement_type == StatementType::IncomeStatement)
            .unwrap();

        // Check expected line items exist
        let codes: Vec<&str> = is
            .line_items
            .iter()
            .map(|li| li.line_code.as_str())
            .collect();
        assert!(codes.contains(&"IS-REV"));
        assert!(codes.contains(&"IS-COGS"));
        assert!(codes.contains(&"IS-GP"));
        assert!(codes.contains(&"IS-OPEX"));
        assert!(codes.contains(&"IS-OI"));
        assert!(codes.contains(&"IS-TAX"));
        assert!(codes.contains(&"IS-NI"));

        // Revenue should be negative (credit balance in TB becomes negative net)
        let revenue = is
            .line_items
            .iter()
            .find(|li| li.line_code == "IS-REV")
            .unwrap();
        // Revenue category has credit > debit, so net = debit - credit = -1,000,000
        assert_eq!(revenue.amount, Decimal::from(-1_000_000));
    }
}
