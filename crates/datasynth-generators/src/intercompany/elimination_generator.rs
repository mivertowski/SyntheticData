//! Consolidation elimination entry generator.
//!
//! Generates elimination entries for intercompany balances, revenue/expense,
//! unrealized profits, and investment/equity eliminations.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::intercompany::{
    ConsolidationJournal, ConsolidationMethod, ConsolidationStatus, EliminationEntry,
    EliminationType, ICAggregatedBalance, ICMatchedPair, ICTransactionType, OwnershipStructure,
};

/// Configuration for elimination generation.
#[derive(Debug, Clone)]
pub struct EliminationConfig {
    /// Consolidation entity code.
    pub consolidation_entity: String,
    /// Base currency for eliminations.
    pub base_currency: String,
    /// Generate IC balance eliminations.
    pub eliminate_ic_balances: bool,
    /// Generate IC revenue/expense eliminations.
    pub eliminate_ic_revenue_expense: bool,
    /// Generate unrealized profit eliminations.
    pub eliminate_unrealized_profit: bool,
    /// Generate investment/equity eliminations.
    pub eliminate_investment_equity: bool,
    /// Average markup rate for unrealized profit calculation.
    pub average_markup_rate: Decimal,
    /// Percentage of IC inventory remaining at period end.
    pub ic_inventory_percent: Decimal,
}

impl Default for EliminationConfig {
    fn default() -> Self {
        Self {
            consolidation_entity: "GROUP".to_string(),
            base_currency: "USD".to_string(),
            eliminate_ic_balances: true,
            eliminate_ic_revenue_expense: true,
            eliminate_unrealized_profit: true,
            eliminate_investment_equity: true,
            average_markup_rate: dec!(0.05),
            ic_inventory_percent: dec!(0.20),
        }
    }
}

/// Generator for consolidation elimination entries.
pub struct EliminationGenerator {
    /// Configuration.
    config: EliminationConfig,
    /// Ownership structure.
    ownership_structure: OwnershipStructure,
    /// Entry counter.
    entry_counter: u64,
    /// Generated elimination journals.
    journals: HashMap<String, ConsolidationJournal>,
}

impl EliminationGenerator {
    /// Create a new elimination generator.
    pub fn new(config: EliminationConfig, ownership_structure: OwnershipStructure) -> Self {
        Self {
            config,
            ownership_structure,
            entry_counter: 0,
            journals: HashMap::new(),
        }
    }

    /// Generate entry ID.
    fn generate_entry_id(&mut self, elim_type: EliminationType) -> String {
        self.entry_counter += 1;
        let prefix = match elim_type {
            EliminationType::ICBalances => "EB",
            EliminationType::ICRevenueExpense => "ER",
            EliminationType::ICProfitInInventory => "EP",
            EliminationType::ICProfitInFixedAssets => "EA",
            EliminationType::InvestmentEquity => "EI",
            EliminationType::ICDividends => "ED",
            EliminationType::ICLoans => "EL",
            EliminationType::ICInterest => "EN",
            EliminationType::MinorityInterest => "EM",
            EliminationType::Goodwill => "EG",
            EliminationType::CurrencyTranslation => "EC",
        };
        format!("{}{:06}", prefix, self.entry_counter)
    }

    /// Get or create consolidation journal for a period.
    fn get_or_create_journal(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
    ) -> &mut ConsolidationJournal {
        self.journals
            .entry(fiscal_period.to_string())
            .or_insert_with(|| {
                ConsolidationJournal::new(
                    self.config.consolidation_entity.clone(),
                    fiscal_period.to_string(),
                    entry_date,
                )
            })
    }

    /// Generate all eliminations for a period.
    pub fn generate_eliminations(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        ic_balances: &[ICAggregatedBalance],
        ic_transactions: &[ICMatchedPair],
        investment_amounts: &HashMap<String, Decimal>,
        equity_amounts: &HashMap<String, HashMap<String, Decimal>>,
    ) -> &ConsolidationJournal {
        // Generate IC balance eliminations
        if self.config.eliminate_ic_balances {
            self.generate_ic_balance_eliminations(fiscal_period, entry_date, ic_balances);
        }

        // Generate IC revenue/expense eliminations
        if self.config.eliminate_ic_revenue_expense {
            self.generate_ic_revenue_expense_eliminations(
                fiscal_period,
                entry_date,
                ic_transactions,
            );
        }

        // Generate unrealized profit eliminations
        if self.config.eliminate_unrealized_profit {
            self.generate_unrealized_profit_eliminations(
                fiscal_period,
                entry_date,
                ic_transactions,
            );
        }

        // Generate investment/equity eliminations
        if self.config.eliminate_investment_equity {
            self.generate_investment_equity_eliminations(
                fiscal_period,
                entry_date,
                investment_amounts,
                equity_amounts,
            );
        }

        self.get_or_create_journal(fiscal_period, entry_date)
    }

    /// Generate IC balance eliminations (receivables vs payables).
    pub fn generate_ic_balance_eliminations(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        balances: &[ICAggregatedBalance],
    ) {
        for balance in balances {
            if balance.elimination_amount() == Decimal::ZERO {
                continue;
            }

            let entry = EliminationEntry::create_ic_balance_elimination(
                self.generate_entry_id(EliminationType::ICBalances),
                self.config.consolidation_entity.clone(),
                fiscal_period.to_string(),
                entry_date,
                &balance.creditor_company,
                &balance.debtor_company,
                &balance.receivable_account,
                &balance.payable_account,
                balance.elimination_amount(),
                balance.currency.clone(),
            );

            let journal = self.get_or_create_journal(fiscal_period, entry_date);
            journal.add_entry(entry);
        }
    }

    /// Generate IC revenue/expense eliminations.
    pub fn generate_ic_revenue_expense_eliminations(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        transactions: &[ICMatchedPair],
    ) {
        // Aggregate by seller/buyer pair and transaction type
        let mut aggregated: HashMap<(String, String, ICTransactionType), Decimal> = HashMap::new();

        for tx in transactions {
            if tx.transaction_type.affects_pnl() {
                let key = (
                    tx.seller_company.clone(),
                    tx.buyer_company.clone(),
                    tx.transaction_type,
                );
                *aggregated.entry(key).or_insert(Decimal::ZERO) += tx.amount;
            }
        }

        for ((seller, buyer, tx_type), amount) in aggregated {
            if amount == Decimal::ZERO {
                continue;
            }

            let revenue_account = match tx_type {
                ICTransactionType::GoodsSale => "4100",
                ICTransactionType::ServiceProvided => "4200",
                ICTransactionType::ManagementFee => "4300",
                ICTransactionType::Royalty => "4400",
                ICTransactionType::LoanInterest => "4500",
                _ => "4900",
            };

            let expense_account = match tx_type {
                ICTransactionType::GoodsSale => "5100",
                ICTransactionType::ServiceProvided => "5200",
                ICTransactionType::ManagementFee => "5300",
                ICTransactionType::Royalty => "5400",
                ICTransactionType::LoanInterest => "5500",
                _ => "5900",
            };

            let entry = EliminationEntry::create_ic_revenue_expense_elimination(
                self.generate_entry_id(EliminationType::ICRevenueExpense),
                self.config.consolidation_entity.clone(),
                fiscal_period.to_string(),
                entry_date,
                &seller,
                &buyer,
                revenue_account,
                expense_account,
                amount,
                self.config.base_currency.clone(),
            );

            let journal = self.get_or_create_journal(fiscal_period, entry_date);
            journal.add_entry(entry);
        }
    }

    /// Generate unrealized profit in inventory eliminations.
    pub fn generate_unrealized_profit_eliminations(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        transactions: &[ICMatchedPair],
    ) {
        // Calculate unrealized profit from goods sales
        let mut unrealized_by_pair: HashMap<(String, String), Decimal> = HashMap::new();

        for tx in transactions {
            if tx.transaction_type == ICTransactionType::GoodsSale {
                let key = (tx.seller_company.clone(), tx.buyer_company.clone());

                // Unrealized profit = IC sales amount * markup rate * % in inventory
                let unrealized =
                    tx.amount * self.config.average_markup_rate * self.config.ic_inventory_percent;

                *unrealized_by_pair.entry(key).or_insert(Decimal::ZERO) += unrealized;
            }
        }

        for ((seller, buyer), unrealized_profit) in unrealized_by_pair {
            if unrealized_profit < dec!(0.01) {
                continue;
            }

            let entry = EliminationEntry::create_unrealized_profit_elimination(
                self.generate_entry_id(EliminationType::ICProfitInInventory),
                self.config.consolidation_entity.clone(),
                fiscal_period.to_string(),
                entry_date,
                &seller,
                &buyer,
                unrealized_profit.round_dp(2),
                self.config.base_currency.clone(),
            );

            let journal = self.get_or_create_journal(fiscal_period, entry_date);
            journal.add_entry(entry);
        }
    }

    /// Generate investment/equity eliminations.
    pub fn generate_investment_equity_eliminations(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        investment_amounts: &HashMap<String, Decimal>,
        equity_amounts: &HashMap<String, HashMap<String, Decimal>>,
    ) {
        // Collect relationships that need processing to avoid borrow issues
        let relationships_to_process: Vec<_> = self
            .ownership_structure
            .relationships
            .iter()
            .filter(|r| r.consolidation_method == ConsolidationMethod::Full)
            .map(|r| {
                (
                    r.parent_company.clone(),
                    r.subsidiary_company.clone(),
                    r.ownership_percentage,
                )
            })
            .collect();

        for (parent, subsidiary, ownership_pct) in relationships_to_process {
            let investment = investment_amounts
                .get(&format!("{parent}_{subsidiary}"))
                .copied()
                .unwrap_or(Decimal::ZERO);

            if investment == Decimal::ZERO {
                continue;
            }

            // Get equity components
            let equity_components: Vec<(String, Decimal)> = equity_amounts
                .get(&subsidiary)
                .map(|eq| eq.iter().map(|(k, v)| (k.clone(), *v)).collect())
                .unwrap_or_else(|| {
                    // Default equity components if not provided
                    vec![
                        ("3100".to_string(), investment * dec!(0.10)), // Common stock
                        ("3200".to_string(), investment * dec!(0.30)), // APIC
                        ("3300".to_string(), investment * dec!(0.60)), // Retained earnings
                    ]
                });

            let total_equity: Decimal = equity_components.iter().map(|(_, v)| v).sum();

            // Calculate goodwill (investment > equity) or bargain purchase (investment < equity)
            let goodwill = if investment > total_equity {
                Some(investment - total_equity)
            } else {
                None
            };

            // Calculate minority interest for non-100% ownership
            let minority_interest = if ownership_pct < dec!(100) {
                let minority_pct = (dec!(100) - ownership_pct) / dec!(100);
                Some(total_equity * minority_pct)
            } else {
                None
            };

            let entry_id = self.generate_entry_id(EliminationType::InvestmentEquity);
            let consolidation_entity = self.config.consolidation_entity.clone();
            let base_currency = self.config.base_currency.clone();

            let entry = EliminationEntry::create_investment_equity_elimination(
                entry_id,
                consolidation_entity,
                fiscal_period.to_string(),
                entry_date,
                &parent,
                &subsidiary,
                investment,
                equity_components,
                goodwill,
                minority_interest,
                base_currency,
            );

            let journal = self.get_or_create_journal(fiscal_period, entry_date);
            journal.add_entry(entry);
        }
    }

    /// Generate dividend elimination entry.
    pub fn generate_dividend_elimination(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        paying_company: &str,
        receiving_company: &str,
        dividend_amount: Decimal,
    ) -> EliminationEntry {
        let mut entry = EliminationEntry::new(
            self.generate_entry_id(EliminationType::ICDividends),
            EliminationType::ICDividends,
            self.config.consolidation_entity.clone(),
            fiscal_period.to_string(),
            entry_date,
            self.config.base_currency.clone(),
        );

        entry.related_companies = vec![paying_company.to_string(), receiving_company.to_string()];
        entry.description =
            format!("Eliminate IC dividend from {paying_company} to {receiving_company}");

        // Debit dividend income (reduce income)
        entry.add_line(datasynth_core::models::intercompany::EliminationLine {
            line_number: 1,
            company: receiving_company.to_string(),
            account: "4600".to_string(), // Dividend income
            is_debit: true,
            amount: dividend_amount,
            currency: self.config.base_currency.clone(),
            description: "Eliminate dividend income".to_string(),
        });

        // Credit retained earnings (restore to subsidiary)
        entry.add_line(datasynth_core::models::intercompany::EliminationLine {
            line_number: 2,
            company: paying_company.to_string(),
            account: "3300".to_string(), // Retained earnings
            is_debit: false,
            amount: dividend_amount,
            currency: self.config.base_currency.clone(),
            description: "Restore retained earnings".to_string(),
        });

        let journal = self.get_or_create_journal(fiscal_period, entry_date);
        journal.add_entry(entry.clone());

        entry
    }

    /// Generate minority interest allocation for period profit/loss.
    pub fn generate_minority_interest_allocation(
        &mut self,
        fiscal_period: &str,
        entry_date: NaiveDate,
        subsidiary: &str,
        net_income: Decimal,
        minority_percentage: Decimal,
    ) -> Option<EliminationEntry> {
        if minority_percentage <= Decimal::ZERO || minority_percentage >= dec!(100) {
            return None;
        }

        let minority_share = net_income * minority_percentage / dec!(100);

        if minority_share.abs() < dec!(0.01) {
            return None;
        }

        let mut entry = EliminationEntry::new(
            self.generate_entry_id(EliminationType::MinorityInterest),
            EliminationType::MinorityInterest,
            self.config.consolidation_entity.clone(),
            fiscal_period.to_string(),
            entry_date,
            self.config.base_currency.clone(),
        );

        entry.related_companies = vec![subsidiary.to_string()];
        entry.description = format!("Minority interest share of {subsidiary} profit/loss");

        if net_income > Decimal::ZERO {
            // Profit: DR consolidated income, CR NCI
            entry.add_line(datasynth_core::models::intercompany::EliminationLine {
                line_number: 1,
                company: self.config.consolidation_entity.clone(),
                account: "3400".to_string(), // NCI share of income
                is_debit: true,
                amount: minority_share,
                currency: self.config.base_currency.clone(),
                description: "NCI share of net income".to_string(),
            });

            entry.add_line(datasynth_core::models::intercompany::EliminationLine {
                line_number: 2,
                company: self.config.consolidation_entity.clone(),
                account: "3500".to_string(), // Non-controlling interest
                is_debit: false,
                amount: minority_share,
                currency: self.config.base_currency.clone(),
                description: "Increase NCI for share of income".to_string(),
            });
        } else {
            // Loss: DR NCI, CR consolidated loss
            entry.add_line(datasynth_core::models::intercompany::EliminationLine {
                line_number: 1,
                company: self.config.consolidation_entity.clone(),
                account: "3500".to_string(), // Non-controlling interest
                is_debit: true,
                amount: minority_share.abs(),
                currency: self.config.base_currency.clone(),
                description: "Decrease NCI for share of loss".to_string(),
            });

            entry.add_line(datasynth_core::models::intercompany::EliminationLine {
                line_number: 2,
                company: self.config.consolidation_entity.clone(),
                account: "3400".to_string(), // NCI share of income
                is_debit: false,
                amount: minority_share.abs(),
                currency: self.config.base_currency.clone(),
                description: "NCI share of net loss".to_string(),
            });
        }

        let journal = self.get_or_create_journal(fiscal_period, entry_date);
        journal.add_entry(entry.clone());

        Some(entry)
    }

    /// Get consolidation journal for a period.
    pub fn get_journal(&self, fiscal_period: &str) -> Option<&ConsolidationJournal> {
        self.journals.get(fiscal_period)
    }

    /// Get all journals.
    pub fn get_all_journals(&self) -> &HashMap<String, ConsolidationJournal> {
        &self.journals
    }

    /// Finalize and approve a journal.
    pub fn finalize_journal(
        &mut self,
        fiscal_period: &str,
        approved_by: String,
    ) -> Option<&ConsolidationJournal> {
        if let Some(journal) = self.journals.get_mut(fiscal_period) {
            journal.submit();
            journal.approve(approved_by);
            Some(journal)
        } else {
            None
        }
    }

    /// Post a journal.
    pub fn post_journal(&mut self, fiscal_period: &str) -> Option<&ConsolidationJournal> {
        if let Some(journal) = self.journals.get_mut(fiscal_period) {
            journal.post();
            Some(journal)
        } else {
            None
        }
    }

    /// Get elimination summary for a period.
    pub fn get_summary(&self, fiscal_period: &str) -> Option<EliminationSummaryReport> {
        self.journals.get(fiscal_period).map(|journal| {
            let mut by_type: HashMap<EliminationType, (usize, Decimal)> = HashMap::new();

            for entry in &journal.entries {
                let stats = by_type
                    .entry(entry.elimination_type)
                    .or_insert((0, Decimal::ZERO));
                stats.0 += 1;
                stats.1 += entry.total_debit;
            }

            EliminationSummaryReport {
                fiscal_period: fiscal_period.to_string(),
                consolidation_entity: journal.consolidation_entity.clone(),
                total_entries: journal.entries.len(),
                total_debit: journal.total_debits,
                total_credit: journal.total_credits,
                is_balanced: journal.is_balanced,
                status: journal.status,
                by_type,
            }
        })
    }

    /// Reset counters and clear journals.
    pub fn reset(&mut self) {
        self.entry_counter = 0;
        self.journals.clear();
    }
}

/// Summary report for elimination entries.
#[derive(Debug, Clone)]
pub struct EliminationSummaryReport {
    /// Fiscal period.
    pub fiscal_period: String,
    /// Consolidation entity.
    pub consolidation_entity: String,
    /// Total number of entries.
    pub total_entries: usize,
    /// Total debit amount.
    pub total_debit: Decimal,
    /// Total credit amount.
    pub total_credit: Decimal,
    /// Is the journal balanced?
    pub is_balanced: bool,
    /// Journal status.
    pub status: ConsolidationStatus,
    /// Breakdown by elimination type (count, amount).
    pub by_type: HashMap<EliminationType, (usize, Decimal)>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::intercompany::IntercompanyRelationship;
    use rust_decimal_macros::dec;

    fn create_test_ownership_structure() -> OwnershipStructure {
        let mut structure = OwnershipStructure::new("1000".to_string());
        structure.add_relationship(IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure.add_relationship(IntercompanyRelationship::new(
            "REL002".to_string(),
            "1000".to_string(),
            "1200".to_string(),
            dec!(80),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure
    }

    #[test]
    fn test_elimination_generator_creation() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let generator = EliminationGenerator::new(config, structure);

        assert!(generator.journals.is_empty());
    }

    #[test]
    fn test_generate_ic_balance_eliminations() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let balances = vec![ICAggregatedBalance {
            creditor_company: "1000".to_string(),
            debtor_company: "1100".to_string(),
            receivable_account: "1310".to_string(),
            payable_account: "2110".to_string(),
            receivable_balance: dec!(50000),
            payable_balance: dec!(50000),
            difference: Decimal::ZERO,
            currency: "USD".to_string(),
            as_of_date: NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            is_matched: true,
        }];

        generator.generate_ic_balance_eliminations(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            &balances,
        );

        let journal = generator.get_journal("202206").unwrap();
        assert_eq!(journal.entries.len(), 1);
        assert!(journal.is_balanced);
    }

    #[test]
    fn test_generate_ic_revenue_expense_eliminations() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let transactions = vec![ICMatchedPair::new(
            "IC001".to_string(),
            ICTransactionType::ServiceProvided,
            "1000".to_string(),
            "1100".to_string(),
            dec!(25000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 15).unwrap(),
        )];

        generator.generate_ic_revenue_expense_eliminations(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            &transactions,
        );

        let journal = generator.get_journal("202206").unwrap();
        assert_eq!(journal.entries.len(), 1);
        assert!(journal.is_balanced);
    }

    #[test]
    fn test_generate_unrealized_profit_eliminations() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let transactions = vec![ICMatchedPair::new(
            "IC001".to_string(),
            ICTransactionType::GoodsSale,
            "1000".to_string(),
            "1100".to_string(),
            dec!(100000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 15).unwrap(),
        )];

        generator.generate_unrealized_profit_eliminations(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            &transactions,
        );

        let journal = generator.get_journal("202206").unwrap();
        assert_eq!(journal.entries.len(), 1);
        // Unrealized profit = 100000 * 0.05 * 0.20 = 1000
        assert!(journal.is_balanced);
    }

    #[test]
    fn test_generate_dividend_elimination() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let entry = generator.generate_dividend_elimination(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            "1100",
            "1000",
            dec!(50000),
        );

        assert!(entry.is_balanced());
        assert_eq!(entry.elimination_type, EliminationType::ICDividends);
    }

    #[test]
    fn test_generate_minority_interest_allocation() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let entry = generator.generate_minority_interest_allocation(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            "1200",
            dec!(100000),
            dec!(20), // 20% minority
        );

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert!(entry.is_balanced());
        // Minority share = 100000 * 20% = 20000
        assert_eq!(entry.total_debit, dec!(20000));
    }

    #[test]
    fn test_finalize_and_post_journal() {
        let config = EliminationConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = EliminationGenerator::new(config, structure);

        let balances = vec![ICAggregatedBalance {
            creditor_company: "1000".to_string(),
            debtor_company: "1100".to_string(),
            receivable_account: "1310".to_string(),
            payable_account: "2110".to_string(),
            receivable_balance: dec!(50000),
            payable_balance: dec!(50000),
            difference: Decimal::ZERO,
            currency: "USD".to_string(),
            as_of_date: NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            is_matched: true,
        }];

        generator.generate_ic_balance_eliminations(
            "202206",
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
            &balances,
        );

        generator.finalize_journal("202206", "ADMIN".to_string());
        let journal = generator.get_journal("202206").unwrap();
        assert_eq!(journal.status, ConsolidationStatus::Approved);

        generator.post_journal("202206");
        let journal = generator.get_journal("202206").unwrap();
        assert_eq!(journal.status, ConsolidationStatus::Posted);
    }
}
