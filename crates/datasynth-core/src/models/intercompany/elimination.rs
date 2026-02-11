//! Consolidation elimination models for intercompany transactions.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of consolidation eliminations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EliminationType {
    /// Eliminate intercompany receivables and payables.
    ICBalances,
    /// Eliminate intercompany revenue and expense.
    ICRevenueExpense,
    /// Eliminate intercompany profit in inventory.
    ICProfitInInventory,
    /// Eliminate intercompany profit in fixed assets.
    ICProfitInFixedAssets,
    /// Eliminate investment in subsidiary against equity.
    InvestmentEquity,
    /// Eliminate intercompany dividends.
    ICDividends,
    /// Eliminate intercompany loan balances.
    ICLoans,
    /// Eliminate intercompany interest income/expense.
    ICInterest,
    /// Minority interest (non-controlling interest) recognition.
    MinorityInterest,
    /// Goodwill recognition from acquisition.
    Goodwill,
    /// Currency translation adjustment.
    CurrencyTranslation,
}

impl EliminationType {
    /// Get the description for this elimination type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ICBalances => "Eliminate intercompany receivables and payables",
            Self::ICRevenueExpense => "Eliminate intercompany revenue and expense",
            Self::ICProfitInInventory => "Eliminate unrealized profit in inventory",
            Self::ICProfitInFixedAssets => "Eliminate unrealized profit in fixed assets",
            Self::InvestmentEquity => "Eliminate investment against subsidiary equity",
            Self::ICDividends => "Eliminate intercompany dividends",
            Self::ICLoans => "Eliminate intercompany loan balances",
            Self::ICInterest => "Eliminate intercompany interest income/expense",
            Self::MinorityInterest => "Recognize non-controlling interest",
            Self::Goodwill => "Recognize goodwill from acquisition",
            Self::CurrencyTranslation => "Currency translation adjustment",
        }
    }

    /// Check if this elimination affects profit/loss.
    pub fn affects_pnl(&self) -> bool {
        matches!(
            self,
            Self::ICRevenueExpense
                | Self::ICProfitInInventory
                | Self::ICProfitInFixedAssets
                | Self::ICDividends
                | Self::ICInterest
        )
    }

    /// Check if this elimination is recurring every period.
    pub fn is_recurring(&self) -> bool {
        matches!(
            self,
            Self::ICBalances
                | Self::ICRevenueExpense
                | Self::ICLoans
                | Self::ICInterest
                | Self::MinorityInterest
        )
    }
}

/// A consolidation elimination entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminationEntry {
    /// Unique elimination entry ID.
    pub entry_id: String,
    /// Elimination type.
    pub elimination_type: EliminationType,
    /// Consolidation entity (group company).
    pub consolidation_entity: String,
    /// Fiscal period (YYYYMM format).
    pub fiscal_period: String,
    /// Entry date.
    pub entry_date: NaiveDate,
    /// Related companies (for IC eliminations).
    pub related_companies: Vec<String>,
    /// Elimination journal lines.
    pub lines: Vec<EliminationLine>,
    /// Total debit amount.
    pub total_debit: Decimal,
    /// Total credit amount.
    pub total_credit: Decimal,
    /// Currency.
    pub currency: String,
    /// Is this a permanent or temporary elimination?
    pub is_permanent: bool,
    /// Related IC references (for traceability).
    pub ic_references: Vec<String>,
    /// Elimination description.
    pub description: String,
    /// Created by (user/system).
    pub created_by: String,
    /// Creation timestamp.
    pub created_at: chrono::NaiveDateTime,
}

impl EliminationEntry {
    /// Create a new elimination entry.
    pub fn new(
        entry_id: String,
        elimination_type: EliminationType,
        consolidation_entity: String,
        fiscal_period: String,
        entry_date: NaiveDate,
        currency: String,
    ) -> Self {
        Self {
            entry_id,
            elimination_type,
            consolidation_entity,
            fiscal_period,
            entry_date,
            related_companies: Vec::new(),
            lines: Vec::new(),
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            currency,
            is_permanent: !elimination_type.is_recurring(),
            ic_references: Vec::new(),
            description: elimination_type.description().to_string(),
            created_by: "SYSTEM".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    /// Add an elimination line.
    pub fn add_line(&mut self, line: EliminationLine) {
        if line.is_debit {
            self.total_debit += line.amount;
        } else {
            self.total_credit += line.amount;
        }
        self.lines.push(line);
    }

    /// Check if the entry is balanced.
    pub fn is_balanced(&self) -> bool {
        self.total_debit == self.total_credit
    }

    /// Create an IC balance elimination (receivable/payable).
    #[allow(clippy::too_many_arguments)]
    pub fn create_ic_balance_elimination(
        entry_id: String,
        consolidation_entity: String,
        fiscal_period: String,
        entry_date: NaiveDate,
        company1: &str,
        company2: &str,
        receivable_account: &str,
        payable_account: &str,
        amount: Decimal,
        currency: String,
    ) -> Self {
        let mut entry = Self::new(
            entry_id,
            EliminationType::ICBalances,
            consolidation_entity,
            fiscal_period,
            entry_date,
            currency.clone(),
        );

        entry.related_companies = vec![company1.to_string(), company2.to_string()];
        entry.description = format!("Eliminate IC balance between {} and {}", company1, company2);

        // Debit the payable (reduce liability)
        entry.add_line(EliminationLine {
            line_number: 1,
            company: company2.to_string(),
            account: payable_account.to_string(),
            is_debit: true,
            amount,
            currency: currency.clone(),
            description: format!("Eliminate IC payable to {}", company1),
        });

        // Credit the receivable (reduce asset)
        entry.add_line(EliminationLine {
            line_number: 2,
            company: company1.to_string(),
            account: receivable_account.to_string(),
            is_debit: false,
            amount,
            currency,
            description: format!("Eliminate IC receivable from {}", company2),
        });

        entry
    }

    /// Create an IC revenue/expense elimination.
    #[allow(clippy::too_many_arguments)]
    pub fn create_ic_revenue_expense_elimination(
        entry_id: String,
        consolidation_entity: String,
        fiscal_period: String,
        entry_date: NaiveDate,
        seller: &str,
        buyer: &str,
        revenue_account: &str,
        expense_account: &str,
        amount: Decimal,
        currency: String,
    ) -> Self {
        let mut entry = Self::new(
            entry_id,
            EliminationType::ICRevenueExpense,
            consolidation_entity,
            fiscal_period,
            entry_date,
            currency.clone(),
        );

        entry.related_companies = vec![seller.to_string(), buyer.to_string()];
        entry.description = format!(
            "Eliminate IC revenue/expense between {} and {}",
            seller, buyer
        );

        // Debit revenue (reduce income)
        entry.add_line(EliminationLine {
            line_number: 1,
            company: seller.to_string(),
            account: revenue_account.to_string(),
            is_debit: true,
            amount,
            currency: currency.clone(),
            description: format!("Eliminate IC revenue from {}", buyer),
        });

        // Credit expense (reduce expense)
        entry.add_line(EliminationLine {
            line_number: 2,
            company: buyer.to_string(),
            account: expense_account.to_string(),
            is_debit: false,
            amount,
            currency,
            description: format!("Eliminate IC expense to {}", seller),
        });

        entry
    }

    /// Create an unrealized profit in inventory elimination.
    #[allow(clippy::too_many_arguments)]
    pub fn create_unrealized_profit_elimination(
        entry_id: String,
        consolidation_entity: String,
        fiscal_period: String,
        entry_date: NaiveDate,
        seller: &str,
        buyer: &str,
        unrealized_profit: Decimal,
        currency: String,
    ) -> Self {
        let mut entry = Self::new(
            entry_id,
            EliminationType::ICProfitInInventory,
            consolidation_entity,
            fiscal_period,
            entry_date,
            currency.clone(),
        );

        entry.related_companies = vec![seller.to_string(), buyer.to_string()];
        entry.description = format!(
            "Eliminate unrealized profit in inventory from {} to {}",
            seller, buyer
        );

        // Debit retained earnings/COGS (reduce profit)
        entry.add_line(EliminationLine {
            line_number: 1,
            company: seller.to_string(),
            account: "5000".to_string(), // COGS or adjustment account
            is_debit: true,
            amount: unrealized_profit,
            currency: currency.clone(),
            description: "Eliminate unrealized profit".to_string(),
        });

        // Credit inventory (reduce asset value)
        entry.add_line(EliminationLine {
            line_number: 2,
            company: buyer.to_string(),
            account: "1400".to_string(), // Inventory account
            is_debit: false,
            amount: unrealized_profit,
            currency,
            description: "Reduce inventory to cost".to_string(),
        });

        entry
    }

    /// Create investment/equity elimination.
    #[allow(clippy::too_many_arguments)]
    pub fn create_investment_equity_elimination(
        entry_id: String,
        consolidation_entity: String,
        fiscal_period: String,
        entry_date: NaiveDate,
        parent: &str,
        subsidiary: &str,
        investment_amount: Decimal,
        equity_components: Vec<(String, Decimal)>, // (account, amount)
        goodwill: Option<Decimal>,
        minority_interest: Option<Decimal>,
        currency: String,
    ) -> Self {
        let consol_entity = consolidation_entity.clone();
        let mut entry = Self::new(
            entry_id,
            EliminationType::InvestmentEquity,
            consolidation_entity,
            fiscal_period,
            entry_date,
            currency.clone(),
        );

        entry.related_companies = vec![parent.to_string(), subsidiary.to_string()];
        entry.is_permanent = true;
        entry.description = format!("Eliminate investment in {} against equity", subsidiary);

        let mut line_number = 1;

        // Debit equity components of subsidiary
        for (account, amount) in equity_components {
            entry.add_line(EliminationLine {
                line_number,
                company: subsidiary.to_string(),
                account,
                is_debit: true,
                amount,
                currency: currency.clone(),
                description: "Eliminate subsidiary equity".to_string(),
            });
            line_number += 1;
        }

        // Debit goodwill if applicable
        if let Some(goodwill_amount) = goodwill {
            entry.add_line(EliminationLine {
                line_number,
                company: consol_entity.clone(),
                account: "1800".to_string(), // Goodwill account
                is_debit: true,
                amount: goodwill_amount,
                currency: currency.clone(),
                description: "Recognize goodwill".to_string(),
            });
            line_number += 1;
        }

        // Credit investment account
        entry.add_line(EliminationLine {
            line_number,
            company: parent.to_string(),
            account: "1510".to_string(), // Investment in subsidiary
            is_debit: false,
            amount: investment_amount,
            currency: currency.clone(),
            description: "Eliminate investment in subsidiary".to_string(),
        });
        line_number += 1;

        // Credit minority interest if applicable
        if let Some(mi_amount) = minority_interest {
            entry.add_line(EliminationLine {
                line_number,
                company: consol_entity.clone(),
                account: "3500".to_string(), // Non-controlling interest
                is_debit: false,
                amount: mi_amount,
                currency,
                description: "Recognize non-controlling interest".to_string(),
            });
        }

        entry
    }
}

/// A single line in an elimination entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminationLine {
    /// Line number.
    pub line_number: u32,
    /// Company code this line affects.
    pub company: String,
    /// Account code.
    pub account: String,
    /// Is this a debit (true) or credit (false)?
    pub is_debit: bool,
    /// Amount.
    pub amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Line description.
    pub description: String,
}

/// Consolidation elimination rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminationRule {
    /// Rule identifier.
    pub rule_id: String,
    /// Rule name.
    pub name: String,
    /// Elimination type this rule handles.
    pub elimination_type: EliminationType,
    /// Source account pattern (regex or exact).
    pub source_account_pattern: String,
    /// Target account pattern (regex or exact).
    pub target_account_pattern: String,
    /// Applies to specific company pairs (empty = all).
    pub company_pairs: Vec<(String, String)>,
    /// Priority (lower = higher priority).
    pub priority: u32,
    /// Is this rule active?
    pub is_active: bool,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// End date (if rule expires).
    pub end_date: Option<NaiveDate>,
    /// Auto-generate eliminations?
    pub auto_generate: bool,
}

impl EliminationRule {
    /// Create a new IC balance elimination rule.
    pub fn new_ic_balance_rule(
        rule_id: String,
        name: String,
        receivable_pattern: String,
        payable_pattern: String,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            rule_id,
            name,
            elimination_type: EliminationType::ICBalances,
            source_account_pattern: receivable_pattern,
            target_account_pattern: payable_pattern,
            company_pairs: Vec::new(),
            priority: 10,
            is_active: true,
            effective_date,
            end_date: None,
            auto_generate: true,
        }
    }

    /// Check if rule is active on a given date.
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        self.is_active && date >= self.effective_date && self.end_date.is_none_or(|end| date <= end)
    }
}

/// Aggregated IC balances for elimination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICAggregatedBalance {
    /// Seller/creditor company.
    pub creditor_company: String,
    /// Buyer/debtor company.
    pub debtor_company: String,
    /// IC receivable account.
    pub receivable_account: String,
    /// IC payable account.
    pub payable_account: String,
    /// Receivable balance (per creditor's books).
    pub receivable_balance: Decimal,
    /// Payable balance (per debtor's books).
    pub payable_balance: Decimal,
    /// Difference (should be zero if matched).
    pub difference: Decimal,
    /// Currency.
    pub currency: String,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Is fully matched?
    pub is_matched: bool,
}

impl ICAggregatedBalance {
    /// Create a new aggregated balance.
    pub fn new(
        creditor_company: String,
        debtor_company: String,
        receivable_account: String,
        payable_account: String,
        currency: String,
        as_of_date: NaiveDate,
    ) -> Self {
        Self {
            creditor_company,
            debtor_company,
            receivable_account,
            payable_account,
            receivable_balance: Decimal::ZERO,
            payable_balance: Decimal::ZERO,
            difference: Decimal::ZERO,
            currency,
            as_of_date,
            is_matched: true,
        }
    }

    /// Set balances and calculate difference.
    pub fn set_balances(&mut self, receivable: Decimal, payable: Decimal) {
        self.receivable_balance = receivable;
        self.payable_balance = payable;
        self.difference = receivable - payable;
        self.is_matched = self.difference == Decimal::ZERO;
    }

    /// Get the elimination amount (minimum of both sides).
    pub fn elimination_amount(&self) -> Decimal {
        self.receivable_balance.min(self.payable_balance)
    }
}

/// Consolidation journal for a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationJournal {
    /// Consolidation entity.
    pub consolidation_entity: String,
    /// Fiscal period.
    pub fiscal_period: String,
    /// Journal status.
    pub status: ConsolidationStatus,
    /// All elimination entries for this period.
    pub entries: Vec<EliminationEntry>,
    /// Summary by elimination type.
    pub summary: HashMap<EliminationType, EliminationSummary>,
    /// Total debits.
    pub total_debits: Decimal,
    /// Total credits.
    pub total_credits: Decimal,
    /// Is journal balanced?
    pub is_balanced: bool,
    /// Created date.
    pub created_date: NaiveDate,
    /// Last modified date.
    pub modified_date: NaiveDate,
    /// Approved by (if applicable).
    pub approved_by: Option<String>,
    /// Approval date.
    pub approved_date: Option<NaiveDate>,
}

impl ConsolidationJournal {
    /// Create a new consolidation journal.
    pub fn new(
        consolidation_entity: String,
        fiscal_period: String,
        created_date: NaiveDate,
    ) -> Self {
        Self {
            consolidation_entity,
            fiscal_period,
            status: ConsolidationStatus::Draft,
            entries: Vec::new(),
            summary: HashMap::new(),
            total_debits: Decimal::ZERO,
            total_credits: Decimal::ZERO,
            is_balanced: true,
            created_date,
            modified_date: created_date,
            approved_by: None,
            approved_date: None,
        }
    }

    /// Add an elimination entry.
    pub fn add_entry(&mut self, entry: EliminationEntry) {
        self.total_debits += entry.total_debit;
        self.total_credits += entry.total_credit;
        self.is_balanced = self.total_debits == self.total_credits;

        // Update summary
        let summary = self
            .summary
            .entry(entry.elimination_type)
            .or_insert_with(|| EliminationSummary {
                elimination_type: entry.elimination_type,
                entry_count: 0,
                total_amount: Decimal::ZERO,
            });
        summary.entry_count += 1;
        summary.total_amount += entry.total_debit;

        self.entries.push(entry);
        self.modified_date = chrono::Utc::now().date_naive();
    }

    /// Submit for approval.
    pub fn submit(&mut self) {
        if self.is_balanced {
            self.status = ConsolidationStatus::PendingApproval;
        }
    }

    /// Approve the journal.
    pub fn approve(&mut self, approved_by: String) {
        self.status = ConsolidationStatus::Approved;
        self.approved_by = Some(approved_by);
        self.approved_date = Some(chrono::Utc::now().date_naive());
    }

    /// Post the journal.
    pub fn post(&mut self) {
        if self.status == ConsolidationStatus::Approved {
            self.status = ConsolidationStatus::Posted;
        }
    }
}

/// Status of consolidation journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationStatus {
    /// Draft - still being prepared.
    #[default]
    Draft,
    /// Submitted for approval.
    PendingApproval,
    /// Approved.
    Approved,
    /// Posted to consolidated financials.
    Posted,
    /// Reversed.
    Reversed,
}

/// Summary statistics for an elimination type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliminationSummary {
    /// Elimination type.
    pub elimination_type: EliminationType,
    /// Number of entries.
    pub entry_count: usize,
    /// Total amount eliminated.
    pub total_amount: Decimal,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_elimination_type_properties() {
        assert!(EliminationType::ICRevenueExpense.affects_pnl());
        assert!(!EliminationType::ICBalances.affects_pnl());

        assert!(EliminationType::ICBalances.is_recurring());
        assert!(!EliminationType::InvestmentEquity.is_recurring());
    }

    #[test]
    fn test_ic_balance_elimination() {
        let entry = EliminationEntry::create_ic_balance_elimination(
            "ELIM001".to_string(),
            "GROUP".to_string(),
            "202206".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            "1000",
            "1100",
            "1310",
            "2110",
            dec!(50000),
            "USD".to_string(),
        );

        assert_eq!(entry.lines.len(), 2);
        assert!(entry.is_balanced());
        assert_eq!(entry.total_debit, dec!(50000));
        assert_eq!(entry.total_credit, dec!(50000));
    }

    #[test]
    fn test_ic_revenue_expense_elimination() {
        let entry = EliminationEntry::create_ic_revenue_expense_elimination(
            "ELIM002".to_string(),
            "GROUP".to_string(),
            "202206".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            "1000",
            "1100",
            "4100",
            "5100",
            dec!(100000),
            "USD".to_string(),
        );

        assert!(entry.is_balanced());
        assert!(entry.elimination_type.affects_pnl());
    }

    #[test]
    fn test_aggregated_balance() {
        let mut balance = ICAggregatedBalance::new(
            "1000".to_string(),
            "1100".to_string(),
            "1310".to_string(),
            "2110".to_string(),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
        );

        balance.set_balances(dec!(50000), dec!(50000));
        assert!(balance.is_matched);
        assert_eq!(balance.elimination_amount(), dec!(50000));

        balance.set_balances(dec!(50000), dec!(48000));
        assert!(!balance.is_matched);
        assert_eq!(balance.difference, dec!(2000));
        assert_eq!(balance.elimination_amount(), dec!(48000));
    }

    #[test]
    fn test_consolidation_journal() {
        let mut journal = ConsolidationJournal::new(
            "GROUP".to_string(),
            "202206".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
        );

        let entry = EliminationEntry::create_ic_balance_elimination(
            "ELIM001".to_string(),
            "GROUP".to_string(),
            "202206".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            "1000",
            "1100",
            "1310",
            "2110",
            dec!(50000),
            "USD".to_string(),
        );

        journal.add_entry(entry);

        assert_eq!(journal.entries.len(), 1);
        assert!(journal.is_balanced);
        assert_eq!(journal.status, ConsolidationStatus::Draft);

        journal.submit();
        assert_eq!(journal.status, ConsolidationStatus::PendingApproval);
    }
}
